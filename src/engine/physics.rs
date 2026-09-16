use glam::Vec3;
use std::collections::HashSet;
use crate::world::World;
use crate::world::WorldCoord;

// 60Hz fixed timestep keeps simulation stable and deterministic regardless of frame rate.
pub const SIMULATION_DT: f32 = 1.0 / 60.0;

// Limit catch up steps to 8 per frame to avoid the 'spiral of death' during stalls.
pub const MAX_PHYSICS_STEPS: u32 = 8;

// Separate runtime ID from grid coordinates because bodies move in continuous space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicsBodyId(pub u64);

// This records a detected overlap between a dynamic body and a static block.
pub struct CollisionRecord {
    pub body_id: PhysicsBodyId,
    pub block_coord: WorldCoord,
}

pub struct PhysicsClock {
    pub accumulator: f32,
}

impl PhysicsClock {
    pub fn new() -> Self {
        Self { accumulator: 0.0 }
    }

    // Reset on exit to prevent simulation jumps when returning to a project.
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }

    pub fn update<F>(&mut self, dt: f32, mut step_fn: F)
    where
        F: FnMut(f32)
    {
        // Clamp huge deltas (e.g. from window drags) to prevent tunneling.
        let dt = dt.min(0.25);
        self.accumulator += dt;

        let mut steps_processed = 0;
        while self.accumulator >= SIMULATION_DT {
            if steps_processed >= MAX_PHYSICS_STEPS {
                // Drop remaining time if we can't keep up; better to slow down than freeze.
                self.accumulator = 0.0;
                break;
            }

            step_fn(SIMULATION_DT);
            self.accumulator -= SIMULATION_DT;
            steps_processed += 1;
        }
    }
}

#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub id: PhysicsBodyId,
    pub position: Vec3,
    pub velocity: Vec3,
    pub size: Vec3,
    pub density: f32,
    pub anchored: bool,
    pub solid: bool,
    pub gravity_participation: bool,
    pub is_sleeping: bool,

    // Stores the normal of the surface the body is currently resting against.
    // This allows the simulation to cancel gravity acceleration into that
    // surface, preventing a repeated cycle of penetration and correction.
    pub static_contact_normal: Option<Vec3>,
}

impl PhysicsBody {
    pub fn new(id: PhysicsBodyId, position: Vec3, size: Vec3) -> Self {
        Self {
            id,
            position,
            velocity: Vec3::ZERO,
            size,
            density: 1.0,
            anchored: false,
            solid: true,
            gravity_participation: true,
            is_sleeping: false,
            static_contact_normal: None,
        }
    }

    // Mass is derived from volume. Anchored objects report 0 (infinite mass).
    pub fn get_mass(&self) -> f32 {
        if self.anchored {
            0.0
        } else {
            let volume = self.size.x * self.size.y * self.size.z;
            volume * self.density
        }
    }

    // Position is the minimum corner (X, Y, Z).
    pub fn min_corner(&self) -> Vec3 {
        self.position
    }

    pub fn max_corner(&self) -> Vec3 {
        self.position + self.size
    }
}

pub struct PhysicsIdGenerator {
    next_id: u64,
}

impl PhysicsIdGenerator {
    pub fn new() -> Self {
        Self { next_id: 1 }
    }

    pub fn next(&mut self) -> PhysicsBodyId {
        let id = PhysicsBodyId(self.next_id);
        self.next_id += 1;
        id
    }
}

pub struct PhysicsWorld {
    pub bodies: Vec<PhysicsBody>,

    // Static colliders are blocks that are solid and anchored.
    pub static_colliders: HashSet<WorldCoord>,

    id_gen: PhysicsIdGenerator,

    // Step counter used to throttle console output.
    step_count: u64,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            static_colliders: HashSet::new(),
            id_gen: PhysicsIdGenerator::new(),
            step_count: 0,
        }
    }

    // World gravity changes velocity but does not decide where the body ends up.
    // We cancel the gravity component pointing into a resting surface to ensure
    // stability and prevent "jittering" into static blocks every step.
    pub fn apply_gravity(&mut self, gravity: Vec3, dt: f32) {
        self.step_count += 1;
        for body in &mut self.bodies {
            if !body.anchored && body.gravity_participation {
                let mut effective_gravity = gravity;

                // If resting against a surface, remove gravity pointing into it.
                if let Some(normal) = body.static_contact_normal {
                    let gravity_into_surface = gravity.dot(normal);
                    if gravity_into_surface < 0.0 {
                        // Project gravity onto the tangential plane.
                        effective_gravity -= normal * gravity_into_surface;
                    }
                }

                body.velocity += effective_gravity * dt;
            }
        }

        // Temporary diagnostic output used while verifying runtime gravity.
        if self.step_count % 60 == 0 {
            if let Some(body) = self.bodies.first() {
                println!("Physics Debug | Body[0] Vel: ({:.2}, {:.2}, {:.2}) | Pos: ({:.2}, {:.2}, {:.2})",
                    body.velocity.x, body.velocity.y, body.velocity.z,
                    body.position.x, body.position.y, body.position.z);
            }
        }
    }

    // Position integration happens after gravity so the new velocity is used immediately.
    pub fn integrate_positions(&mut self, dt: f32) {
        for body in &mut self.bodies {
            if !body.anchored {
                body.position += body.velocity * dt;
            }
        }
    }

    /// Resolves overlaps between dynamic bodies and static blocks.
    /// This is the point where we move bodies out of solid blocks and adjust
    /// their velocities so they don't keep falling through the floor.
    pub fn resolve_static_collisions(&mut self) {
        // Iterate by index to avoid borrow checker conflicts when calling check_body_touching.
        for i in 0..self.bodies.len() {
            if !self.bodies[i].solid || self.bodies[i].anchored {
                continue;
            }

            let mut latest_contact_normal: Option<Vec3> = None;

            // We perform multiple resolution passes if the body hits multiple blocks.
            for _ in 0..4 {
                let body = &self.bodies[i];
                let min = body.min_corner();
                let max = body.max_corner();
                let body_center = (min + max) * 0.5;

                let x_start = min.x.floor() as i32;
                let x_end = max.x.ceil() as i32;
                let y_start = min.y.floor() as i32;
                let y_end = max.y.ceil() as i32;
                let z_start = min.z.floor() as i32;
                let z_end = max.z.ceil() as i32;

                let mut best_collision: Option<(Vec3, f32)> = None;
                let mut min_p = f32::MAX;

                for x in x_start..x_end {
                    for y in y_start..y_end {
                        for z in z_start..z_end {
                            let coord = WorldCoord::new(x, y, z);
                            if self.static_colliders.contains(&coord) {
                                let voxel_min = Vec3::new(x as f32, y as f32, z as f32);
                                let voxel_max = voxel_min + Vec3::ONE;
                                let voxel_center = voxel_min + Vec3::new(0.5, 0.5, 0.5);

                                // Rule: We only resolve actual geometric penetration.
                                if Self::aabb_overlap_static(min, max, voxel_min, voxel_max) {
                                    // Geometric overlap depths.
                                    let overlap_x = max.x.min(voxel_max.x) - min.x.max(voxel_min.x);
                                    let overlap_y = max.y.min(voxel_max.y) - min.y.max(voxel_min.y);
                                    let overlap_z = max.z.min(voxel_max.z) - min.z.max(voxel_min.z);

                                    // Identify the smallest penetration axis.
                                    let (axis_normal, penetration) = if overlap_x < overlap_y && overlap_x < overlap_z {
                                        let n = if body_center.x < voxel_center.x { Vec3::NEG_X } else { Vec3::X };
                                        (n, overlap_x)
                                    } else if overlap_y < overlap_z {
                                        let n = if body_center.y < voxel_center.y { Vec3::NEG_Y } else { Vec3::Y };
                                        (n, overlap_y)
                                    } else {
                                        let n = if body_center.z < voxel_center.z { Vec3::NEG_Z } else { Vec3::Z };
                                        (n, overlap_z)
                                    };

                                    if penetration < min_p {
                                        min_p = penetration;
                                        best_collision = Some((axis_normal, penetration));
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some((normal, penetration)) = best_collision {
                    let body = &mut self.bodies[i];
                    let epsilon = 0.0001;
                    // Move out along normal (points voxel -> body).
                    body.position += normal * (penetration + epsilon);

                    // Remove velocity component pointing into the surface.
                    let velocity_into_surface = body.velocity.dot(normal);
                    if velocity_into_surface < 0.0 {
                        body.velocity -= normal * velocity_into_surface;
                    }

                    latest_contact_normal = Some(normal);
                } else {
                    break;
                }
            }

            // After resolution, we update the resting contact state.
            if let Some(n) = latest_contact_normal {
                self.bodies[i].static_contact_normal = Some(n);
            } else {
                let n_opt = self.bodies[i].static_contact_normal;
                if let Some(n) = n_opt {
                    // If no penetration this step, check if body is still touching the surface.
                    if !self.check_body_touching(&self.bodies[i], n) {
                        self.bodies[i].static_contact_normal = None;
                    }
                }
            }
        }
    }

    // Checks if the body's AABB is touching a static block in the support direction.
    fn check_body_touching(&self, body: &PhysicsBody, normal: Vec3) -> bool {
        let epsilon = 0.001;
        let test_min = body.min_corner() - normal * epsilon;
        let test_max = body.max_corner() - normal * epsilon;

        let x_start = test_min.x.floor() as i32;
        let x_end = test_max.x.ceil() as i32;
        let y_start = test_min.y.floor() as i32;
        let y_end = test_max.y.ceil() as i32;
        let z_start = test_min.z.floor() as i32;
        let z_end = test_max.z.ceil() as i32;

        for x in x_start..x_end {
            for y in y_start..y_end {
                for z in z_start..z_end {
                    if self.static_colliders.contains(&WorldCoord::new(x, y, z)) {
                        let v_min = Vec3::new(x as f32, y as f32, z as f32);
                        let v_max = v_min + Vec3::ONE;
                        if Self::aabb_overlap_static(test_min, test_max, v_min, v_max) {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }

    pub fn check_static_collisions(&self) -> Vec<CollisionRecord> {
        let mut collisions = Vec::new();
        for body in &self.bodies {
            if !body.solid { continue; }
            let min = body.min_corner();
            let max = body.max_corner();
            let x_start = min.x.floor() as i32;
            let x_end = max.x.ceil() as i32;
            let y_start = min.y.floor() as i32;
            let y_end = max.y.ceil() as i32;
            let z_start = min.z.floor() as i32;
            let z_end = max.z.ceil() as i32;
            for x in x_start..x_end {
                for y in y_start..y_end {
                    for z in z_start..z_end {
                        let coord = WorldCoord::new(x, y, z);
                        if self.static_colliders.contains(&coord) {
                            let voxel_min = Vec3::new(x as f32, y as f32, z as f32);
                            let voxel_max = voxel_min + Vec3::ONE;
                            if Self::aabb_overlap_static(min, max, voxel_min, voxel_max) {
                                collisions.push(CollisionRecord {
                                    body_id: body.id,
                                    block_coord: coord,
                                });
                            }
                        }
                    }
                }
            }
        }
        collisions
    }

    fn aabb_overlap_static(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> bool {
        a_min.x < b_max.x && a_max.x > b_min.x &&
        a_min.y < b_max.y && a_max.y > b_min.y &&
        a_min.z < b_max.z && a_max.z > b_min.z
    }

    pub fn register_from_world(&mut self, world: &World) {
        self.bodies.clear();
        self.static_colliders.clear();
        self.id_gen = PhysicsIdGenerator::new();
        self.step_count = 0;
        for coord in world.active_blocks() {
            if let Some(cell) = world.get(coord) {
                if cell.anchored {
                    if cell.solid { self.static_colliders.insert(coord); }
                } else {
                    let id = self.id_gen.next();
                    let pos = Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32);
                    let mut body = PhysicsBody::new(id, pos, Vec3::ONE);
                    body.solid = cell.solid;
                    self.bodies.push(body);
                }
            }
        }
        println!("Physics: Registered {} dynamic bodies and {} static colliders.",
            self.bodies.len(), self.static_colliders.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::CellType;

    #[test]
    fn test_physics_clock_accumulation() {
        let mut clock = PhysicsClock::new();
        let mut steps = 0;
        clock.update(SIMULATION_DT, |_| steps += 1);
        assert_eq!(steps, 1);
        assert!(clock.accumulator < 0.0001);
        clock.update(SIMULATION_DT / 2.0, |_| steps += 1);
        assert_eq!(steps, 1);
        clock.update(SIMULATION_DT / 2.0, |_| steps += 1);
        assert_eq!(steps, 2);
    }

    #[test]
    fn test_physics_clock_cap() {
        let mut clock = PhysicsClock::new();
        let mut steps = 0;
        clock.update(1.0, |_| steps += 1);
        assert_eq!(steps, MAX_PHYSICS_STEPS);
        assert_eq!(clock.accumulator, 0.0);
    }

    #[test]
    fn test_physics_body_id_uniqueness() {
        let mut id_gen = PhysicsIdGenerator::new();
        assert_ne!(id_gen.next(), id_gen.next());
    }

    #[test]
    fn test_world_to_physics_registration() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let c1 = WorldCoord::new(0, 0, 0);
        world.set_cell(c1, CellType::Grass);
        if let Some(cell) = world.get_mut(c1) { cell.anchored = false; }
        let c2 = WorldCoord::new(1, 0, 0);
        world.set_cell(c2, CellType::Grass);
        if let Some(cell) = world.get_mut(c2) { cell.anchored = true; }
        p_world.register_from_world(&world);
        assert_eq!(p_world.bodies.len(), 1);
    }

    #[test]
    fn test_app_mode_transition_logic() {
        use crate::engine::EditorMode;
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let mut last_mode = EditorMode::Editor;
        let coord = WorldCoord::new(0, 5, 0);
        world.set_cell(coord, CellType::Grass);
        if let Some(cell) = world.get_mut(coord) { cell.anchored = false; }
        let simulate_update = |mode: EditorMode, last: &mut EditorMode, p: &mut PhysicsWorld, w: &World| {
            if mode == EditorMode::Play && *last == EditorMode::Editor { p.register_from_world(w); }
            *last = mode;
        };
        simulate_update(EditorMode::Play, &mut last_mode, &mut p_world, &world);
        assert_eq!(p_world.bodies.len(), 1);
        simulate_update(EditorMode::Play, &mut last_mode, &mut p_world, &world);
        assert_eq!(p_world.bodies.len(), 1);
    }

    #[test]
    fn test_physics_gravity_acceleration() {
        let mut p_world = PhysicsWorld::new();
        p_world.bodies.push(PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE));
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        p_world.apply_gravity(gravity, SIMULATION_DT);
        let expected = Vec3::new(0.0, -9.81 * SIMULATION_DT, 0.0);
        assert!((p_world.bodies[0].velocity.y - expected.y).abs() < 1e-5);
    }

    #[test]
    fn test_physics_gravity_anchored_ignored() {
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        body.anchored = true;
        p_world.bodies.push(body);
        p_world.apply_gravity(Vec3::new(0.0, -10.0, 0.0), 0.1);
        assert_eq!(p_world.bodies[0].velocity, Vec3::ZERO);
    }

    #[test]
    fn test_physics_gravity_participation_gating() {
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        body.gravity_participation = false;
        p_world.bodies.push(body);
        p_world.apply_gravity(Vec3::new(0.0, -10.0, 0.0), 0.1);
        assert_eq!(p_world.bodies[0].velocity, Vec3::ZERO);
    }

    #[test]
    fn test_physics_gravity_3d_vector() {
        let mut p_world = PhysicsWorld::new();
        p_world.bodies.push(PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE));
        let gravity = Vec3::new(1.0, 2.0, -3.0);
        p_world.apply_gravity(gravity, 0.5);
        assert!((p_world.bodies[0].velocity - Vec3::new(0.5, 1.0, -1.5)).length() < 1e-5);
    }

    #[test]
    fn test_physics_position_integration() {
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(10.0, 10.0, 10.0), Vec3::ONE);
        body.velocity = Vec3::new(1.0, 2.0, 3.0);
        p_world.bodies.push(body);
        p_world.integrate_positions(0.1);
        assert!((p_world.bodies[0].position - Vec3::new(10.1, 10.2, 10.3)).length() < 1e-5);
    }

    #[test]
    fn test_physics_integration_anchored_ignored() {
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        body.anchored = true;
        body.velocity = Vec3::ONE;
        p_world.bodies.push(body);
        p_world.integrate_positions(0.1);
        assert_eq!(p_world.bodies[0].position, Vec3::ZERO);
    }

    #[test]
    fn test_physics_gravity_integration_ordering() {
        let mut p_world = PhysicsWorld::new();
        p_world.bodies.push(PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE));
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        let dt = 0.1;
        p_world.apply_gravity(gravity, dt);
        p_world.integrate_positions(dt);
        assert!((p_world.bodies[0].velocity.y - (-1.0)).abs() < 1e-5);
        assert!((p_world.bodies[0].position.y - (-0.1)).abs() < 1e-5);
    }

    #[test]
    fn test_static_collider_inclusion_rules() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let c1 = WorldCoord::new(0, 0, 0);
        world.set_cell(c1, CellType::Grass);
        if let Some(cell) = world.get_mut(c1) { cell.anchored = true; cell.solid = true; }
        let c2 = WorldCoord::new(1, 0, 0);
        world.set_cell(c2, CellType::Grass);
        if let Some(cell) = world.get_mut(c2) { cell.anchored = true; cell.solid = false; }
        p_world.register_from_world(&world);
        assert!(p_world.static_colliders.contains(&c1));
        assert!(!p_world.static_colliders.contains(&c2));
    }

    #[test]
    fn test_collision_detection() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        p_world.bodies.push(PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.5, 0.5, 0.5), Vec3::ONE));
        assert_eq!(p_world.check_static_collisions().len(), 1);
    }

    #[test]
    fn test_physics_floor_collision_resolution() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 0.9, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(0.0, -10.0, 0.0);
        p_world.bodies.push(body);
        p_world.resolve_static_collisions();
        // Resting position should be Y=1.0 (top of voxel [0,1])
        assert!(p_world.bodies[0].position.y >= 1.0);
        assert!((p_world.bodies[0].position.y - 1.0).abs() < 0.001);
        assert_eq!(p_world.bodies[0].velocity.y, 0.0);
    }

    #[test]
    fn test_physics_tangential_velocity_preserved() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 0.9, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(5.0, -10.0, 0.0);
        p_world.bodies.push(body);
        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].velocity.y, 0.0);
        assert_eq!(p_world.bodies[0].velocity.x, 5.0);
    }

    #[test]
    fn test_physics_wall_collision_resolution() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(-0.1, 0.0, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(10.0, 0.0, 0.0);
        p_world.bodies.push(body);
        p_world.resolve_static_collisions();
        // Pushed back to X=-1.0 (left of voxel [0,1])
        assert!((p_world.bodies[0].position.x - (-1.0)).abs() < 0.001);
        assert_eq!(p_world.bodies[0].velocity.x, 0.0);
    }

    #[test]
    fn test_physics_non_solid_ignored() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let c = WorldCoord::new(0, 0, 0);
        world.set_cell(c, CellType::Grass);
        if let Some(cell) = world.get_mut(c) { cell.anchored = true; cell.solid = false; }
        p_world.register_from_world(&world);
        p_world.bodies.push(PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 0.9, 0.0), Vec3::ONE));
        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].position.y, 0.9);
    }

    #[test]
    fn test_physics_touching_no_push() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        p_world.bodies.push(PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 1.0, 0.0), Vec3::ONE));
        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].position.y, 1.0);
    }

    #[test]
    fn test_physics_adjacent_floors_stable() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        p_world.static_colliders.insert(WorldCoord::new(1, 0, 0));
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.5, 0.9, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(0.0, -10.0, 0.0);
        p_world.bodies.push(body);
        p_world.resolve_static_collisions();
        assert!((p_world.bodies[0].position.y - 1.0).abs() < 0.001);
        assert_eq!(p_world.bodies[0].velocity.y, 0.0);
    }

    #[test]
    fn test_physics_touching_adjacent_regression() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, -1, -1));
        let pos = Vec3::new(0.0, 0.0, -1.0);
        p_world.bodies.push(PhysicsBody::new(PhysicsBodyId(1), pos, Vec3::ONE));
        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].position, pos);
    }

    #[test]
    fn test_physics_resting_contact_stability() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        let dt = SIMULATION_DT;

        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 0.9, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(0.0, -10.0, 0.0);
        p_world.bodies.push(body);

        // Step 1: Hit floor and resolve.
        p_world.apply_gravity(gravity, dt);
        p_world.integrate_positions(dt);
        p_world.resolve_static_collisions();

        assert!(p_world.bodies[0].static_contact_normal.is_some());
        let pos_after_res = p_world.bodies[0].position;
        assert_eq!(p_world.bodies[0].velocity.y, 0.0);

        // Step 2: Resting. Gravity must be cancelled.
        p_world.apply_gravity(gravity, dt);
        assert_eq!(p_world.bodies[0].velocity.y, 0.0);

        p_world.integrate_positions(dt);
        assert_eq!(p_world.bodies[0].position, pos_after_res);

        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].position, pos_after_res);
    }

    #[test]
    fn test_physics_tangential_sliding_while_resting() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        p_world.static_colliders.insert(WorldCoord::new(1, 0, 0));

        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 1.0, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(5.0, 0.0, 0.0);
        body.static_contact_normal = Some(Vec3::Y);
        p_world.bodies.push(body);

        p_world.apply_gravity(Vec3::new(0.0, -10.0, 0.0), 0.1);
        p_world.integrate_positions(0.1);

        assert_eq!(p_world.bodies[0].velocity.y, 0.0);
        assert!((p_world.bodies[0].position.x - 0.5).abs() < 1e-5);
    }

    #[test]
    fn test_physics_contact_clearing_when_moving_away() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));

        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 1.0, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(0.0, 10.0, 0.0);
        body.static_contact_normal = Some(Vec3::Y);
        p_world.bodies.push(body);

        p_world.integrate_positions(0.1);
        p_world.resolve_static_collisions();

        assert!(p_world.bodies[0].static_contact_normal.is_none());
    }

    #[test]
    fn test_physics_arbitrary_gravity_contact() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(1, 0, 0));

        let body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.1, 0.0, 0.0), Vec3::ONE);
        let gravity = Vec3::new(10.0, 0.0, 0.0);
        p_world.bodies.push(body);

        p_world.apply_gravity(gravity, 0.1);
        p_world.integrate_positions(0.1);
        p_world.resolve_static_collisions();

        assert_eq!(p_world.bodies[0].static_contact_normal, Some(Vec3::NEG_X));

        p_world.apply_gravity(gravity, 0.1);
        assert_eq!(p_world.bodies[0].velocity.x, 0.0);
    }
}
