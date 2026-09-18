use crate::world::CellType;
use crate::world::World;
use crate::world::WorldCoord;
use glam::Vec3;
use std::collections::HashSet;

// 60Hz fixed timestep keeps simulation stable and deterministic regardless of frame rate.
pub const SIMULATION_DT: f32 = 1.0 / 60.0;

// Limit catch up steps to 8 per frame to avoid the 'spiral of death' during stalls.
pub const MAX_PHYSICS_STEPS: u32 = 8;

// Velocity below which a body is considered "at rest" for sleep calculation.
pub const SLEEP_VELOCITY_THRESHOLD: f32 = 0.02;

// Time in seconds a body must stay below the threshold to fall asleep.
pub const SLEEP_TIME_THRESHOLD: f32 = 0.5;

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
        F: FnMut(f32),
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
    pub cell_type: CellType,
    pub position: Vec3,
    pub velocity: Vec3,
    pub size: Vec3,
    pub color_rgb: Vec3,
    pub texture: String,
    pub density: f32,
    pub visible: bool,
    pub anchored: bool,
    pub solid: bool,
    pub gravity_participation: bool,
    pub is_sleeping: bool,

    // Accumulates time spent below the linear velocity threshold.
    pub sleep_timer: f32,

    // Stores the normal of the surface the body is currently resting against.
    // normal points from the static block toward this body.
    pub static_contact_normal: Option<Vec3>,

    // ID and normal of a dynamic body currently supporting this one.
    // normal points from the support toward this body.
    pub dynamic_contact: Option<(PhysicsBodyId, Vec3)>,
}

impl PhysicsBody {
    pub fn new(id: PhysicsBodyId, position: Vec3, size: Vec3) -> Self {
        Self {
            id,
            cell_type: CellType::Empty,
            position,
            velocity: Vec3::ZERO,
            size,
            color_rgb: Vec3::new(0.5, 0.5, 0.5),
            texture: "None".to_string(),
            density: 1.0,
            visible: true,
            anchored: false,
            solid: true,
            gravity_participation: true,
            is_sleeping: false,
            sleep_timer: 0.0,
            static_contact_normal: None,
            dynamic_contact: None,
        }
    }

    // Wake the body and reset its sleep timer.
    pub fn wake(&mut self) {
        self.is_sleeping = false;
        self.sleep_timer = 0.0;
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
    // stability and prevent "jittering" into blocks every step.
    pub fn apply_gravity(&mut self, gravity: Vec3, dt: f32) {
        self.step_count += 1;
        for body in &mut self.bodies {
            if body.is_sleeping || body.anchored || !body.gravity_participation {
                continue;
            }

            let mut effective_gravity = gravity;

            // Remove gravity pointing into an active contact normal if that normal
            // actually opposes the gravity vector.
            let mut combined_normal = Vec3::ZERO;
            if let Some(n) = body.static_contact_normal {
                combined_normal += n;
            }
            if let Some((_, n)) = body.dynamic_contact {
                combined_normal += n;
            }

            if combined_normal != Vec3::ZERO {
                let n = combined_normal.normalize();
                let gravity_into_surface = gravity.dot(n);
                // If gravity points into the surface (dot < 0), remove that component.
                if gravity_into_surface < 0.0 {
                    effective_gravity -= n * gravity_into_surface;
                }
            }

            body.velocity += effective_gravity * dt;
        }

        // Low frequency debug print (approx once per second at 60Hz)
        if self.step_count % 60 == 0 {
            for (i, body) in self.bodies.iter().enumerate() {
                println!(
                    "Physics Debug | Body[{}] Pos: ({:.3}, {:.3}, {:.3}) Vel: ({:.3}, {:.3}, {:.3}) Solid: {} Anchored: {} Sleep: {}",
                    i,
                    body.position.x,
                    body.position.y,
                    body.position.z,
                    body.velocity.x,
                    body.velocity.y,
                    body.velocity.z,
                    body.solid,
                    body.anchored,
                    body.is_sleeping,
                );
            }
        }
    }

    // Position integration moves dynamic bodies that are not sleeping.
    pub fn integrate_positions(&mut self, dt: f32) {
        for body in &mut self.bodies {
            if !body.anchored && !body.is_sleeping {
                body.position += body.velocity * dt;
            }
        }
    }

    /// Resolves overlaps between dynamic bodies and static blocks.
    pub fn resolve_static_collisions(&mut self) {
        for i in 0..self.bodies.len() {
            if !self.bodies[i].solid || self.bodies[i].anchored {
                continue;
            }

            let mut latest_contact_normal: Option<Vec3> = None;

            for _ in 0..4 {
                let body = &self.bodies[i];
                let b_min = body.min_corner();
                let b_max = body.max_corner();
                let b_center = (b_min + b_max) * 0.5;

                let x_start = b_min.x.floor() as i32;
                let x_end = b_max.x.ceil() as i32;
                let y_start = b_min.y.floor() as i32;
                let y_end = b_max.y.ceil() as i32;
                let z_start = b_min.z.floor() as i32;
                let z_end = b_max.z.ceil() as i32;

                let mut best_collision: Option<(Vec3, f32)> = None;
                let mut min_overlap = f32::MAX;

                for x in x_start..x_end {
                    for y in y_start..y_end {
                        for z in z_start..z_end {
                            let coord = WorldCoord::new(x, y, z);
                            if self.static_colliders.contains(&coord) {
                                let v_min = Vec3::new(x as f32, y as f32, z as f32);
                                let v_max = v_min + Vec3::ONE;
                                let v_center = v_min + Vec3::new(0.5, 0.5, 0.5);

                                // Rule: We only resolve actual geometric overlap. Touching is contact.
                                if Self::aabb_overlap_static(b_min, b_max, v_min, v_max) {
                                    let overlap_x = b_max.x.min(v_max.x) - b_min.x.max(v_min.x);
                                    let overlap_y = b_max.y.min(v_max.y) - b_min.y.max(v_min.y);
                                    let overlap_z = b_max.z.min(v_max.z) - b_min.z.max(v_min.z);

                                    // Identify the normal based only on center relative position
                                    // on the smallest overlap axis.
                                    let (axis_normal, depth) =
                                        if overlap_x < overlap_y && overlap_x < overlap_z {
                                            let n = if b_center.x < v_center.x {
                                                Vec3::NEG_X
                                            } else {
                                                Vec3::X
                                            };
                                            (n, overlap_x)
                                        } else if overlap_y < overlap_z {
                                            let n = if b_center.y < v_center.y {
                                                Vec3::NEG_Y
                                            } else {
                                                Vec3::Y
                                            };
                                            (n, overlap_y)
                                        } else {
                                            let n = if b_center.z < v_center.z {
                                                Vec3::NEG_Z
                                            } else {
                                                Vec3::Z
                                            };
                                            (n, overlap_z)
                                        };

                                    if depth < min_overlap {
                                        min_overlap = depth;
                                        best_collision = Some((axis_normal, depth));
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some((normal, penetration)) = best_collision {
                    if penetration > 0.0001 {
                        let body = &mut self.bodies[i];
                        let epsilon = 0.0001;
                        // Correct position along normal (points from voxel to body).
                        body.position += normal * (penetration + epsilon);

                        // Remove only inward velocity using dot product.
                        let v_in = body.velocity.dot(normal);
                        if v_in < 0.0 {
                            body.velocity -= normal * v_in;
                        }

                        latest_contact_normal = Some(normal);
                        body.wake();
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            if let Some(n) = latest_contact_normal {
                self.bodies[i].static_contact_normal = Some(n);
            } else if let Some(n) = self.bodies[i].static_contact_normal {
                if !self.check_body_touching_static(&self.bodies[i], n) {
                    self.bodies[i].static_contact_normal = None;
                }
            }
        }
    }

    /// Resolves overlaps between dynamic bodies.
    pub fn resolve_dynamic_collisions(&mut self) {
        let n = self.bodies.len();
        for i in 0..n {
            for j in (i + 1)..n {
                let (body_a, body_b) = if i < j {
                    let (left, right) = self.bodies.split_at_mut(j);
                    (&mut left[i], &mut right[0])
                } else {
                    continue;
                };

                if !body_a.solid || !body_b.solid || (body_a.anchored && body_b.anchored) {
                    continue;
                }

                let min_a = body_a.min_corner();
                let max_a = body_a.max_corner();
                let center_a = (min_a + max_a) * 0.5;

                let min_b = body_b.min_corner();
                let max_b = body_b.max_corner();
                let center_b = (min_b + max_b) * 0.5;

                // Rule: We only resolve actual geometric overlap.
                if Self::aabb_overlap_static(min_a, max_a, min_b, max_b) {
                    let overlap_x = max_a.x.min(max_b.x) - min_a.x.max(min_b.x);
                    let overlap_y = max_a.y.min(max_b.y) - min_a.y.max(min_b.y);
                    let overlap_z = max_a.z.min(max_b.z) - min_a.z.max(min_b.z);

                    // Normal points from B toward A.
                    let (normal, penetration) = if overlap_x < overlap_y && overlap_x < overlap_z {
                        let n = if center_a.x < center_b.x {
                            Vec3::NEG_X
                        } else {
                            Vec3::X
                        };
                        (n, overlap_x)
                    } else if overlap_y < overlap_z {
                        let n = if center_a.y < center_b.y {
                            Vec3::NEG_Y
                        } else {
                            Vec3::Y
                        };
                        (n, overlap_y)
                    } else {
                        let n = if center_a.z < center_b.z {
                            Vec3::NEG_Z
                        } else {
                            Vec3::Z
                        };
                        (n, overlap_z)
                    };

                    if penetration > 0.0001 {
                        let epsilon = 0.0001;
                        let correction = normal * (penetration * 0.5 + epsilon);

                        if body_a.anchored {
                            body_b.position -= normal * (penetration + epsilon);
                        } else if body_b.anchored {
                            body_a.position += normal * (penetration + epsilon);
                        } else {
                            body_a.position += correction;
                            body_b.position -= correction;
                        }

                        // Remove only inward velocity using dot product with normal.
                        let v_a_in = body_a.velocity.dot(normal);
                        if v_a_in < 0.0 {
                            body_a.velocity -= normal * v_a_in;
                        }

                        let v_b_in = body_b.velocity.dot(-normal);
                        if v_b_in < 0.0 {
                            body_b.velocity -= (-normal) * v_b_in;
                        }

                        // Overlap resolution indicates a physical disturbance; wake both.
                        body_a.wake();
                        body_b.wake();
                    }
                }
            }
        }
    }

    /// Identifies exact touching face contacts and registers support relationships.
    pub fn refresh_dynamic_support(&mut self) {
        let n = self.bodies.len();

        // We use indices to safely manage support assignment.
        let mut new_supports: Vec<Option<(PhysicsBodyId, Vec3)>> = vec![None; n];

        for i in 0..n {
            for j in (i + 1)..n {
                let min_a = self.bodies[i].min_corner();
                let max_a = self.bodies[i].max_corner();
                let center_a = (min_a + max_a) * 0.5;

                let min_b = self.bodies[j].min_corner();
                let max_b = self.bodies[j].max_corner();
                let center_b = (min_b + max_b) * 0.5;

                // Touching is defined as zero overlap on one axis within a tolerance
                // and real overlap on the others.
                let epsilon = 0.01;
                let overlap_x = max_a.x.min(max_b.x) - min_a.x.max(min_b.x);
                let overlap_y = max_a.y.min(max_b.y) - min_a.y.max(min_b.y);
                let overlap_z = max_a.z.min(max_b.z) - min_a.z.max(min_b.z);

                let ax = overlap_x.abs() < epsilon && overlap_y > 0.001 && overlap_z > 0.001;
                let ay = overlap_y.abs() < epsilon && overlap_x > 0.001 && overlap_z > 0.001;
                let az = overlap_z.abs() < epsilon && overlap_x > 0.001 && overlap_y > 0.001;

                if ax || ay || az {
                    // Normal points from support toward supported.
                    let (supported_idx, supporting_idx, support_normal) = if ax {
                        if center_a.x < center_b.x {
                            (j, i, Vec3::X)
                        } else {
                            (i, j, Vec3::NEG_X)
                        }
                    } else if ay {
                        if center_a.y < center_b.y {
                            (j, i, Vec3::Y)
                        } else {
                            (i, j, Vec3::NEG_Y)
                        }
                    } else {
                        if center_a.z < center_b.z {
                            (j, i, Vec3::Z)
                        } else {
                            (i, j, Vec3::NEG_Z)
                        }
                    };

                    new_supports[supported_idx] =
                        Some((self.bodies[supporting_idx].id, support_normal));
                }
            }
        }

        // Apply new supports and wake if the support relationship changed or was lost.
        for i in 0..n {
            let old = self.bodies[i].dynamic_contact;
            let new = new_supports[i];

            if old != new {
                self.bodies[i].wake();
            }

            self.bodies[i].dynamic_contact = new;
        }
    }

    /// Evaluates linear velocity and support to manage body sleep state.
    pub fn update_sleeping(&mut self, gravity: Vec3) {
        for i in 0..self.bodies.len() {
            let mut should_wake = false;

            // Supporting Body Check.
            if let Some((support_id, _normal)) = self.bodies[i].dynamic_contact {
                if let Some(support) = self.bodies.iter().find(|b| b.id == support_id) {
                    // Rule: supported body wakes if support body wakes.
                    if !support.is_sleeping {
                        should_wake = true;
                    }
                } else {
                    should_wake = true; // Support disappeared.
                }
            }

            if should_wake {
                self.bodies[i].wake();
            }
        }

        for body in &mut self.bodies {
            if body.anchored {
                continue;
            }

            if body.is_sleeping {
                body.velocity = Vec3::ZERO;
                continue;
            }

            // Sleep Rule: must stay slow for SLEEP_TIME_THRESHOLD.
            if body.velocity.length() < SLEEP_VELOCITY_THRESHOLD {
                // Rule: Under nonzero gravity, a body can only sleep if it has valid physical
                // support opposing the gravity vector.
                let mut is_supported = false;
                if body.gravity_participation && gravity.length_squared() > 1e-6 {
                    if let Some(n) = body.static_contact_normal {
                        if gravity.dot(n) < -1e-3 {
                            is_supported = true;
                        }
                    }
                    if !is_supported {
                        if let Some((_, n)) = body.dynamic_contact {
                            if gravity.dot(n) < -1e-3 {
                                is_supported = true;
                            }
                        }
                    }
                } else {
                    is_supported = true;
                }

                if is_supported {
                    body.sleep_timer += SIMULATION_DT;
                    if body.sleep_timer >= SLEEP_TIME_THRESHOLD {
                        body.is_sleeping = true;
                        body.velocity = Vec3::ZERO;
                    }
                } else {
                    body.sleep_timer = 0.0;
                }
            } else {
                body.sleep_timer = 0.0;
            }
        }
    }

    // Touching helper for dynamic bodies. normal_other_to_body points from other to body.
    fn check_body_touching_dynamic(
        &self,
        body: &PhysicsBody,
        other: &PhysicsBody,
        normal_other_to_body: Vec3,
    ) -> bool {
        let epsilon = 0.01;
        let test_min = body.min_corner() - normal_other_to_body * epsilon;
        let test_max = body.max_corner() - normal_other_to_body * epsilon;
        Self::aabb_overlap_static(test_min, test_max, other.min_corner(), other.max_corner())
    }

    // Touching helper for static blocks.
    fn check_body_touching_static(&self, body: &PhysicsBody, normal: Vec3) -> bool {
        let epsilon = 0.01;
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
            if !body.solid {
                continue;
            }
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
        a_min.x < b_max.x
            && a_max.x > b_min.x
            && a_min.y < b_max.y
            && a_max.y > b_min.y
            && a_min.z < b_max.z
            && a_max.z > b_min.z
    }

    pub fn register_from_world(&mut self, world: &World) {
        self.bodies.clear();
        self.static_colliders.clear();
        self.id_gen = PhysicsIdGenerator::new();
        self.step_count = 0;
        for coord in world.active_blocks() {
            if let Some(cell) = world.get(coord) {
                if cell.cell_type == CellType::Light {
                    continue;
                }

                if cell.anchored {
                    if cell.solid {
                        self.static_colliders.insert(coord);
                    }
                } else {
                    let id = self.id_gen.next();
                    let pos = Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32);
                    let mut body = PhysicsBody::new(id, pos, Vec3::ONE);

                    // Authoritative Property Transfer
                    body.cell_type = cell.cell_type;
                    body.visible = cell.visible;
                    body.solid = cell.solid;
                    body.anchored = cell.anchored;
                    body.texture = cell.texture.clone();
                    body.color_rgb = cell.color_rgb;

                    self.bodies.push(body);
                }
            }
        }
        println!(
            "Physics: Registered {} dynamic bodies and {} static colliders.",
            self.bodies.len(),
            self.static_colliders.len()
        );
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
        world.set_cell(c1, CellType::Block);
        if let Some(cell) = world.get_mut(c1) {
            cell.anchored = false;
        }
        let c2 = WorldCoord::new(1, 0, 0);
        world.set_cell(c2, CellType::Block);
        if let Some(cell) = world.get_mut(c2) {
            cell.anchored = true;
        }
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
        world.set_cell(coord, CellType::Block);
        if let Some(cell) = world.get_mut(coord) {
            cell.anchored = false;
        }
        let simulate_update =
            |mode: EditorMode, last: &mut EditorMode, p: &mut PhysicsWorld, w: &World| {
                if mode == EditorMode::Play && *last == EditorMode::Editor {
                    p.register_from_world(w);
                }
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
        p_world
            .bodies
            .push(PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE));
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
        p_world
            .bodies
            .push(PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE));
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
        p_world
            .bodies
            .push(PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE));
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
        world.set_cell(c1, CellType::Block);
        if let Some(cell) = world.get_mut(c1) {
            cell.anchored = true;
            cell.solid = true;
        }
        p_world.register_from_world(&world);
        assert!(p_world.static_colliders.contains(&c1));
    }

    #[test]
    fn test_collision_detection() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        p_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(1),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::ONE,
        ));
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
        assert!(p_world.bodies[0].position.y >= 1.0);
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
        assert!((p_world.bodies[0].position.x - (-1.0)).abs() < 0.001);
        assert_eq!(p_world.bodies[0].velocity.x, 0.0);
    }

    #[test]
    fn test_physics_non_solid_ignored() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let c = WorldCoord::new(0, 0, 0);
        world.set_cell(c, CellType::Block);
        if let Some(cell) = world.get_mut(c) {
            cell.anchored = true;
            cell.solid = false;
        }
        p_world.register_from_world(&world);
        p_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(1),
            Vec3::new(0.0, 0.9, 0.0),
            Vec3::ONE,
        ));
        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].position.y, 0.9);
    }

    #[test]
    fn test_physics_touching_no_push() {
        let mut p_world = PhysicsWorld::new();
        p_world.static_colliders.insert(WorldCoord::new(0, 0, 0));
        p_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(1),
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::ONE,
        ));
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
        p_world
            .bodies
            .push(PhysicsBody::new(PhysicsBodyId(1), pos, Vec3::ONE));
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
        p_world.apply_gravity(gravity, dt);
        p_world.integrate_positions(dt);
        p_world.resolve_static_collisions();
        assert!(p_world.bodies[0].static_contact_normal.is_some());
        let pos_after_res = p_world.bodies[0].position;
        assert_eq!(p_world.bodies[0].velocity.y, 0.0);
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

    #[test]
    fn test_physics_unsupported_no_sleep() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        p_world
            .bodies
            .push(PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE));
        for _ in 0..100 {
            p_world.apply_gravity(gravity, SIMULATION_DT);
            p_world.integrate_positions(SIMULATION_DT);
            p_world.update_sleeping(gravity);
        }
        assert!(!p_world.bodies[0].is_sleeping);
        assert!(p_world.bodies[0].velocity.y < -5.0);
    }

    #[test]
    fn test_physics_static_floor_sleep() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        p_world.static_colliders.insert(WorldCoord::new(0, -1, 0));
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 0.0, 0.0), Vec3::ONE);
        body.static_contact_normal = Some(Vec3::Y);
        p_world.bodies.push(body);
        for _ in 0..60 {
            p_world.update_sleeping(gravity);
        }
        assert!(p_world.bodies[0].is_sleeping);
    }

    #[test]
    fn test_physics_dynamic_stack_sleep() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        let mut b1 = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        b1.is_sleeping = true;
        let mut b2 = PhysicsBody::new(PhysicsBodyId(2), Vec3::Y, Vec3::ONE);
        b2.dynamic_contact = Some((b1.id, Vec3::Y));
        p_world.bodies.push(b1);
        p_world.bodies.push(b2);
        for _ in 0..60 {
            p_world.update_sleeping(gravity);
        }
        assert!(p_world.bodies[1].is_sleeping);
    }

    #[test]
    fn test_physics_wake_on_support_wake() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        let mut b1 = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        let mut b2 = PhysicsBody::new(PhysicsBodyId(2), Vec3::Y, Vec3::ONE);
        b2.is_sleeping = true;
        b2.dynamic_contact = Some((b1.id, Vec3::Y));
        p_world.bodies.push(b1);
        p_world.bodies.push(b2);
        p_world.update_sleeping(gravity);
        assert!(!p_world.bodies[1].is_sleeping);
    }

    #[test]
    fn test_physics_wake_on_support_loss() {
        let mut p_world = PhysicsWorld::new();
        let mut b1 = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        b1.is_sleeping = true;
        let mut b2 = PhysicsBody::new(PhysicsBodyId(2), Vec3::Y, Vec3::ONE);
        b2.is_sleeping = true;
        b2.dynamic_contact = Some((b1.id, Vec3::Y));
        p_world.bodies.push(b1);
        p_world.bodies.push(b2);
        p_world.bodies[0].position.x += 10.0;
        p_world.refresh_dynamic_support();
        p_world.update_sleeping(Vec3::new(0.0, -10.0, 0.0));
        assert!(!p_world.bodies[1].is_sleeping);
    }

    #[test]
    fn test_physics_side_contact_no_support() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        body.static_contact_normal = Some(Vec3::X);
        p_world.bodies.push(body);
        for _ in 0..100 {
            p_world.update_sleeping(gravity);
        }
        assert!(!p_world.bodies[0].is_sleeping);
    }

    #[test]
    fn test_physics_arbitrary_gravity_sleep() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(10.0, 0.0, 0.0);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        body.static_contact_normal = Some(Vec3::NEG_X);
        p_world.bodies.push(body);
        for _ in 0..60 {
            p_world.update_sleeping(gravity);
        }
        assert!(p_world.bodies[0].is_sleeping);
    }

    #[test]
    fn test_physics_falling_stack_integration() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        let dt = SIMULATION_DT;

        // Setup: floor, dynamic A at y=5, dynamic B at y=10
        p_world.static_colliders.insert(WorldCoord::new(0, -1, 0));
        p_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(1),
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::ONE,
        ));
        p_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(2),
            Vec3::new(0.0, 10.0, 0.0),
            Vec3::ONE,
        ));

        // Simulate many steps until they stack and sleep.
        for _ in 0..300 {
            p_world.apply_gravity(gravity, dt);
            p_world.integrate_positions(dt);
            p_world.resolve_static_collisions();
            p_world.resolve_dynamic_collisions();
            p_world.refresh_dynamic_support();
            p_world.update_sleeping(gravity);
        }

        // A should be supported by floor.
        assert!(p_world.bodies[0].static_contact_normal.is_some());
        // B should be supported by A.
        assert!(p_world.bodies[1].dynamic_contact.is_some());
        assert_eq!(
            p_world.bodies[1].dynamic_contact.unwrap().0,
            p_world.bodies[0].id
        );

        // Both should be asleep.
        assert!(p_world.bodies[0].is_sleeping);
        assert!(p_world.bodies[1].is_sleeping);
    }

    #[test]
    fn test_physics_exact_touching_support() {
        let mut p_world = PhysicsWorld::new();
        let b1 = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 0.0, 0.0), Vec3::ONE);
        let b2 = PhysicsBody::new(PhysicsBodyId(2), Vec3::new(0.0, 1.0, 0.0), Vec3::ONE);
        p_world.bodies.push(b1);
        p_world.bodies.push(b2);

        // AABB test should report NO penetration.
        let min_a = p_world.bodies[0].min_corner();
        let max_a = p_world.bodies[0].max_corner();
        let min_b = p_world.bodies[1].min_corner();
        let max_b = p_world.bodies[1].max_corner();
        assert!(!PhysicsWorld::aabb_overlap_static(
            min_a, max_a, min_b, max_b
        ));

        // Support logic should detect contact.
        p_world.refresh_dynamic_support();
        assert!(p_world.bodies[1].dynamic_contact.is_some());
        let (support_id, normal) = p_world.bodies[1].dynamic_contact.unwrap();
        assert_eq!(support_id, p_world.bodies[0].id);
        assert_eq!(normal, Vec3::Y);
    }

    #[test]
    fn test_physics_x_axis_support() {
        let mut p_world = PhysicsWorld::new();
        let b1 = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(0.0, 0.0, 0.0), Vec3::ONE);
        let b2 = PhysicsBody::new(PhysicsBodyId(2), Vec3::new(1.0, 0.0, 0.0), Vec3::ONE);
        p_world.bodies.push(b1);
        p_world.bodies.push(b2);
        p_world.refresh_dynamic_support();
        assert_eq!(
            p_world.bodies[1].dynamic_contact,
            Some((p_world.bodies[0].id, Vec3::X))
        );
    }

    #[test]
    fn test_physics_sideways_stack_sleep() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(10.0, 0.0, 0.0); // Gravity points +X
        p_world.static_colliders.insert(WorldCoord::new(2, 0, 0)); // Wall at X=2

        let mut b1 = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(1.0, 0.0, 0.0), Vec3::ONE);
        b1.static_contact_normal = Some(Vec3::NEG_X); // Supported by wall
        b1.is_sleeping = true;

        let mut b2 = PhysicsBody::new(PhysicsBodyId(2), Vec3::new(0.0, 0.0, 0.0), Vec3::ONE);
        b2.dynamic_contact = Some((b1.id, Vec3::NEG_X)); // Supported by b1

        p_world.bodies.push(b1);
        p_world.bodies.push(b2);

        for _ in 0..60 {
            p_world.update_sleeping(gravity);
        }
        assert!(
            p_world.bodies[1].is_sleeping,
            "Body should sleep when supported sideways against sideways gravity"
        );
    }

    #[test]
    fn test_physics_wake_on_support_movement_simulated() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(0.0, -10.0, 0.0);

        p_world
            .bodies
            .push(PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE));
        p_world
            .bodies
            .push(PhysicsBody::new(PhysicsBodyId(2), Vec3::Y, Vec3::ONE));
        p_world.bodies[0].is_sleeping = true;
        p_world.bodies[1].is_sleeping = true;
        p_world.bodies[1].dynamic_contact = Some((p_world.bodies[0].id, Vec3::Y));

        // 1. Move support away via state.
        p_world.bodies[0].position.x += 5.0;

        // 2. Run simulation phases.
        p_world.refresh_dynamic_support();
        p_world.update_sleeping(gravity);

        assert!(
            !p_world.bodies[1].is_sleeping,
            "Upper body must wake when lower support is no longer touching"
        );
    }

    #[test]
    fn test_light_physics_exclusion() {
        // Light cells must never become part of the physical simulation.
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let coord = WorldCoord::new(0, 0, 0);
        world.set_cell(coord, CellType::Light);

        p_world.register_from_world(&world);

        assert_eq!(
            p_world.bodies.len(),
            0,
            "Light cell incorrectly registered as dynamic body"
        );
        assert_eq!(
            p_world.static_colliders.len(),
            0,
            "Light cell incorrectly registered as static collider"
        );
    }

    #[test]
    fn test_physics_body_captures_cell_color() {
        let mut world = World::new();
        let coord = WorldCoord::new(2, 5, 3);

        world.set_cell(coord, CellType::Block);

        let authored_color = Vec3::new(1.0, 0.0, 0.25);
        if let Some(cell) = world.get_mut(coord) {
            cell.anchored = false;
            cell.color_rgb = authored_color;
        }

        let mut p_world = PhysicsWorld::new();
        p_world.register_from_world(&world);

        assert_eq!(p_world.bodies.len(), 1);
        assert_eq!(p_world.bodies[0].color_rgb, authored_color);
    }

    #[test]
    fn test_physics_body_independence_after_registration() {
        let mut world = World::new();
        let coord = WorldCoord::new(0, 0, 0);
        world.set_cell(coord, CellType::Block);

        let original_color = Vec3::new(1.0, 1.0, 1.0);
        if let Some(cell) = world.get_mut(coord) {
            cell.anchored = false;
            cell.color_rgb = original_color;
            cell.visible = true;
        }

        let mut p_world = PhysicsWorld::new();
        p_world.register_from_world(&world);

        // Change authored world after registration
        if let Some(cell) = world.get_mut(coord) {
            cell.color_rgb = Vec3::ZERO;
            cell.visible = false;
        }
        world.set_cell(coord, CellType::Empty);

        // Runtime body must remain unchanged
        assert_eq!(p_world.bodies.len(), 1);
        assert_eq!(p_world.bodies[0].color_rgb, original_color);
        assert_eq!(p_world.bodies[0].visible, true);
    }
}
