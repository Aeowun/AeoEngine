use glam::Vec3;
use crate::world::World;

/// Fixed timestep for physics simulation (1/60th of a second).
pub const SIMULATION_DT: f32 = 1.0 / 60.0;

/// Maximum number of simulation steps to process in a single frame to prevent "spiral of death".
pub const MAX_PHYSICS_STEPS: u32 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicsBodyId(pub u64);

pub struct PhysicsClock {
    pub accumulator: f32,
}

impl PhysicsClock {
    pub fn new() -> Self {
        Self { accumulator: 0.0 }
    }

    /// Resets the accumulator. Useful when switching modes.
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }

    /// Advances the clock by the given frame delta time and executes fixed steps.
    pub fn update<F>(&mut self, dt: f32, mut step_fn: F)
    where
        F: FnMut(f32)
    {
        // Cap frame time to prevent massive jumps (e.g. after a pause or stall)
        let dt = dt.min(0.25);
        self.accumulator += dt;

        let mut steps_processed = 0;
        while self.accumulator >= SIMULATION_DT {
            if steps_processed >= MAX_PHYSICS_STEPS {
                // Drop remaining time if we hit the catch-up limit
                self.accumulator = 0.0;
                break;
            }

            step_fn(SIMULATION_DT);
            self.accumulator -= SIMULATION_DT;
            steps_processed += 1;
        }
    }
}

/// Runtime Physics Body representing a physical entity in continuous space.
#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub id: PhysicsBodyId,
    pub position: Vec3,
    pub velocity: Vec3,
    pub size: Vec3,

    // Properties
    pub density: f32,
    pub anchored: bool,
    pub solid: bool,
    pub gravity_participation: bool,

    // State metadata
    pub is_sleeping: bool,
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
        }
    }

    /// Returns the mass calculated from volume and density.
    pub fn get_mass(&self) -> f32 {
        if self.anchored {
            0.0
        } else {
            let volume = self.size.x * self.size.y * self.size.z;
            volume * self.density
        }
    }
}

/// Simple runtime ID generator for physics bodies.
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

/// The Physics System state, managing runtime bodies and simulation.
pub struct PhysicsWorld {
    pub bodies: Vec<PhysicsBody>,
    id_gen: PhysicsIdGenerator,
    step_count: u64,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            id_gen: PhysicsIdGenerator::new(),
            step_count: 0,
        }
    }

    /// Applies gravity acceleration to all dynamic bodies.
    pub fn apply_gravity(&mut self, gravity: Vec3, dt: f32) {
        self.step_count += 1;
        for body in &mut self.bodies {
            if !body.anchored && body.gravity_participation {
                body.velocity += gravity * dt;
            }
        }

        // Low-frequency debug print (approx once per second at 60Hz)
        if self.step_count % 60 == 0 {
            if let Some(body) = self.bodies.first() {
                println!("Physics Debug | Body[0] Velocity: ({:.2}, {:.2}, {:.2})",
                    body.velocity.x, body.velocity.y, body.velocity.z);
            }
        }
    }

    /// Synchronizes the physics system with the authoritative world state.
    /// Discovers non-anchored cells and registers them as dynamic physics bodies.
    pub fn register_from_world(&mut self, world: &World) {
        self.bodies.clear();
        self.id_gen = PhysicsIdGenerator::new(); // Reset IDs for consistent session sync
        self.step_count = 0; // Reset debug counter

        for coord in world.active_blocks() {
            if let Some(cell) = world.get(coord) {
                // Rules: Only non-anchored cells become active dynamic bodies.
                if !cell.anchored {
                    let id = self.id_gen.next();
                    // Position is center of voxel (X.0, Y.0, Z.0)
                    let pos = Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32);
                    let mut body = PhysicsBody::new(id, pos, Vec3::ONE);

                    // Map existing cell properties
                    body.solid = cell.solid;
                    // Density/Mass are not yet in Cell data, using default 1.0

                    self.bodies.push(body);
                }
            }
        }

        println!("Physics: Registered {} dynamic bodies from world.", self.bodies.len());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::CellType;
    use crate::world::WorldCoord;

    #[test]
    fn test_physics_clock_accumulation() {
        let mut clock = PhysicsClock::new();
        let mut steps = 0;

        // One step exactly
        clock.update(SIMULATION_DT, |_| steps += 1);
        assert_eq!(steps, 1);
        assert!(clock.accumulator < 0.0001);

        // Half a step, should not tick
        clock.update(SIMULATION_DT / 2.0, |_| steps += 1);
        assert_eq!(steps, 1);
        assert!((clock.accumulator - SIMULATION_DT / 2.0).abs() < 0.0001);

        // Another half, should tick once
        clock.update(SIMULATION_DT / 2.0, |_| steps += 1);
        assert_eq!(steps, 2);
        assert!(clock.accumulator < 0.0001);
    }

    #[test]
    fn test_physics_clock_cap() {
        let mut clock = PhysicsClock::new();
        let mut steps = 0;

        // Provide enough time for 100 steps
        clock.update(1.0, |_| steps += 1);

        // Should be capped at MAX_PHYSICS_STEPS (8)
        assert_eq!(steps, MAX_PHYSICS_STEPS);
        assert_eq!(clock.accumulator, 0.0);
    }

    #[test]
    fn test_physics_body_id_uniqueness() {
        let mut id_gen = PhysicsIdGenerator::new();
        let id1 = id_gen.next();
        let id2 = id_gen.next();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_world_to_physics_registration() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();

        // 1. Non-anchored cell -> Should be registered
        let coord1 = WorldCoord::new(0, 0, 0);
        world.set_cell(coord1, CellType::Grass);
        if let Some(cell) = world.get_mut(coord1) {
            cell.anchored = false;
        }

        // 2. Anchored cell -> Should be ignored
        let coord2 = WorldCoord::new(1, 0, 0);
        world.set_cell(coord2, CellType::Grass);
        if let Some(cell) = world.get_mut(coord2) {
            cell.anchored = true;
        }

        p_world.register_from_world(&world);

        assert_eq!(p_world.bodies.len(), 1, "Only non-anchored cell should be registered");
        assert_eq!(p_world.bodies[0].position, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_app_mode_transition_logic() {
        use crate::engine::EditorMode;

        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let mut last_mode = EditorMode::Editor;
        let mut current_mode = EditorMode::Editor;

        // 1. Put a non-anchored block in the world
        let coord = WorldCoord::new(0, 5, 0);
        world.set_cell(coord, CellType::Grass);
        if let Some(cell) = world.get_mut(coord) {
            cell.anchored = false;
        }

        // 2. Confirm physics world starts empty
        assert_eq!(p_world.bodies.len(), 0);

        // 3. Simulate App::update logic for transition
        let mut simulate_update = |mode: EditorMode, last: &mut EditorMode, p: &mut PhysicsWorld, w: &World| {
            if mode == EditorMode::Play && *last == EditorMode::Editor {
                p.register_from_world(w);
            }
            *last = mode;
        };

        // 4. Set mode to Play and run update logic
        current_mode = EditorMode::Play;
        simulate_update(current_mode, &mut last_mode, &mut p_world, &world);

        // 5. Assert the physics world now contains the expected registered body
        assert_eq!(p_world.bodies.len(), 1);
        let first_id = p_world.bodies[0].id;

        // 6. Run the update again while still in Play
        simulate_update(current_mode, &mut last_mode, &mut p_world, &world);

        // 7. Assert the registration did not happen again (count and ID remain same)
        assert_eq!(p_world.bodies.len(), 1);
        assert_eq!(p_world.bodies[0].id, first_id);
    }

    #[test]
    fn test_physics_gravity_acceleration() {
        let mut p_world = PhysicsWorld::new();
        let id = PhysicsBodyId(1);
        let pos = Vec3::ZERO;
        let size = Vec3::ONE;
        let mut body = PhysicsBody::new(id, pos, size);
        body.velocity = Vec3::ZERO;
        p_world.bodies.push(body);

        let gravity = Vec3::new(0.0, -9.81, 0.0);
        let dt = 1.0 / 60.0;

        p_world.apply_gravity(gravity, dt);

        let expected_velocity = Vec3::new(0.0, -9.81 * dt, 0.0);
        let actual_velocity = p_world.bodies[0].velocity;

        assert!((actual_velocity.x - expected_velocity.x).abs() < 1e-5);
        assert!((actual_velocity.y - expected_velocity.y).abs() < 1e-5);
        assert!((actual_velocity.z - expected_velocity.z).abs() < 1e-5);
    }

    #[test]
    fn test_physics_gravity_anchored_ignored() {
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        body.anchored = true;
        body.velocity = Vec3::ZERO;
        p_world.bodies.push(body);

        p_world.apply_gravity(Vec3::new(0.0, -10.0, 0.0), 0.1);
        assert_eq!(p_world.bodies[0].velocity, Vec3::ZERO);
    }

    #[test]
    fn test_physics_gravity_participation_gating() {
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        body.gravity_participation = false;
        body.velocity = Vec3::ZERO;
        p_world.bodies.push(body);

        p_world.apply_gravity(Vec3::new(0.0, -10.0, 0.0), 0.1);
        assert_eq!(p_world.bodies[0].velocity, Vec3::ZERO);
    }

    #[test]
    fn test_physics_gravity_3d_vector() {
        let mut p_world = PhysicsWorld::new();
        let body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        p_world.bodies.push(body);

        let gravity = Vec3::new(1.0, 2.0, -3.0);
        let dt = 0.5;
        p_world.apply_gravity(gravity, dt);

        let expected = gravity * dt;
        let actual = p_world.bodies[0].velocity;

        assert!((actual.x - expected.x).abs() < 1e-5);
        assert!((actual.y - expected.y).abs() < 1e-5);
        assert!((actual.z - expected.z).abs() < 1e-5);
    }
}
