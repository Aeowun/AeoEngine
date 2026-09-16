use glam::Vec3;
use crate::world::World;

/// We use a fixed physics timestep of 1/60 second to keep the simulation
/// deterministic across different hardware and frame rates.
pub const SIMULATION_DT: f32 = 1.0 / 60.0;

/// We cap the number of physics steps per frame to 8. This ensures that if the
/// renderer stalls or the window is dragged, the engine does not get stuck in
/// an infinite catch up loop trying to simulate seconds of missed time.
pub const MAX_PHYSICS_STEPS: u32 = 8;

/// This unique ID identifies a body during a single simulation session.
/// It is separate from world coordinates because physics bodies move in
/// continuous space and can exist at any position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PhysicsBodyId(pub u64);

/// The clock tracks how much time has passed since the last physics update.
/// It accumulates frame deltas and triggers fixed size steps.
pub struct PhysicsClock {
    pub accumulator: f32,
}

impl PhysicsClock {
    pub fn new() -> Self {
        Self { accumulator: 0.0 }
    }

    /// Resets the clock. We call this when leaving Play mode so that
    /// the simulation does not jump forward when the user returns.
    pub fn reset(&mut self) {
        self.accumulator = 0.0;
    }

    /// This consumes the frame delta and runs the provided physics logic in
    /// fixed size chunks. If the frame time is too large (over 0.25s), we
    /// clamp it to prevent objects from teleporting through the world.
    pub fn update<F>(&mut self, dt: f32, mut step_fn: F)
    where
        F: FnMut(f32)
    {
        let dt = dt.min(0.25);
        self.accumulator += dt;

        let mut steps_processed = 0;
        while self.accumulator >= SIMULATION_DT {
            // If the simulation falls too far behind the renderer, we drop
            // the excess time. This protects the engine from freezing if
            // the physics calculations become too heavy.
            if steps_processed >= MAX_PHYSICS_STEPS {
                self.accumulator = 0.0;
                break;
            }

            step_fn(SIMULATION_DT);
            self.accumulator -= SIMULATION_DT;
            steps_processed += 1;
        }
    }
}

/// A physics body is a dynamic object in the simulation.
/// Unlike World cells which are locked to a grid, bodies move in continuous
/// world space using floating point coordinates.
#[derive(Debug, Clone)]
pub struct PhysicsBody {
    pub id: PhysicsBodyId,
    pub position: Vec3,
    pub velocity: Vec3,
    pub size: Vec3,

    // The density is used to calculate mass based on the body's volume.
    pub density: f32,

    // Anchored bodies are physically frozen and do not move or fall.
    pub anchored: bool,

    // Solid bodies will eventually participate in collision detection.
    pub solid: bool,

    // Some bodies might ignore world gravity (like trigger volumes).
    pub gravity_participation: bool,

    // Sleeping bodies are skipped by the solver to save time.
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

    /// We calculate mass from volume and density. Anchored objects return zero
    /// mass because they are treated as infinite immovable masses by physics
    /// solvers.
    pub fn get_mass(&self) -> f32 {
        if self.anchored {
            0.0
        } else {
            let volume = self.size.x * self.size.y * self.size.z;
            volume * self.density
        }
    }
}

/// The generator provides a new unique ID for every body created in a session.
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

/// This owns the runtime physics state.
/// It is temporary and exists only while the simulation is running.
pub struct PhysicsWorld {
    pub bodies: Vec<PhysicsBody>,
    id_gen: PhysicsIdGenerator,

    // Used for low frequency debug logging.
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

    /// World gravity changes velocity but does not decide where the body ends up.
    /// Position is updated separately so forces and movement remain separate parts
    /// of the simulation.
    ///
    /// This uses the fixed physics timestep supplied by PhysicsClock. It must not
    /// use frame time because physics behavior should not change just because the
    /// renderer produced a different number of frames.
    pub fn apply_gravity(&mut self, gravity: Vec3, dt: f32) {
        self.step_count += 1;
        for body in &mut self.bodies {
            // Anchored bodies ignore gravity because they are physically frozen.
            if !body.anchored && body.gravity_participation {
                body.velocity += gravity * dt;
            }
        }

        // Temporary debug output used while verifying runtime gravity.
        // Remove this once physics state can be inspected directly in the UI.
        if self.step_count % 60 == 0 {
            if let Some(body) = self.bodies.first() {
                println!("Physics Debug | Body[0] Vel: ({:.2}, {:.2}, {:.2}) | Pos: ({:.2}, {:.2}, {:.2})",
                    body.velocity.x, body.velocity.y, body.velocity.z,
                    body.position.x, body.position.y, body.position.z);
            }
        }
    }

    /// We update position based on velocity. This is called after gravity so
    /// the velocity change from this step is included in the movement.
    pub fn integrate_positions(&mut self, dt: f32) {
        for body in &mut self.bodies {
            if !body.anchored {
                body.position += body.velocity * dt;
            }
        }
    }

    /// This function populates the physics simulation from the grid world.
    /// We keep World as the authoring truth and use PhysicsWorld for runtime simulation.
    /// We do not write simulated positions back into World here because the editor
    /// must be able to restore the exact authored state when Play stops.
    pub fn register_from_world(&mut self, world: &World) {
        self.bodies.clear();
        self.id_gen = PhysicsIdGenerator::new();
        self.step_count = 0;

        for coord in world.active_blocks() {
            if let Some(cell) = world.get(coord) {
                // Only non anchored blocks participate in physics movement.
                if !cell.anchored {
                    let id = self.id_gen.next();
                    // We treat the grid coordinate as the center of the voxel.
                    let pos = Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32);
                    let mut body = PhysicsBody::new(id, pos, Vec3::ONE);

                    body.solid = cell.solid;

                    self.bodies.push(body);
                }
            }
        }

        // Temporary confirmation print for the mode transition phase.
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
        // This proves that the clock correctly accumulates time across
        // several small frame deltas until the fixed threshold is reached.
        let mut clock = PhysicsClock::new();
        let mut steps = 0;

        // One step exactly should trigger one update.
        clock.update(SIMULATION_DT, |_| steps += 1);
        assert_eq!(steps, 1);
        assert!(clock.accumulator < 0.0001);

        // Half a step should not trigger an update yet.
        clock.update(SIMULATION_DT / 2.0, |_| steps += 1);
        assert_eq!(steps, 1);
        assert!((clock.accumulator - SIMULATION_DT / 2.0).abs() < 0.0001);

        // The second half should push it over the threshold and trigger the tick.
        clock.update(SIMULATION_DT / 2.0, |_| steps += 1);
        assert_eq!(steps, 2);
        assert!(clock.accumulator < 0.0001);
    }

    #[test]
    fn test_physics_clock_cap() {
        // This ensures the engine remains responsive during heavy stalls
        // by capping the number of catch up steps we perform in one frame.
        let mut clock = PhysicsClock::new();
        let mut steps = 0;

        // Provide enough time for 100 steps.
        clock.update(1.0, |_| steps += 1);

        // It must be capped at MAX_PHYSICS_STEPS (8) and drop the excess.
        assert_eq!(steps, MAX_PHYSICS_STEPS);
        assert_eq!(clock.accumulator, 0.0);
    }

    #[test]
    fn test_physics_body_id_uniqueness() {
        // IDs must stay unique within a session so we can track bodies correctly.
        let mut id_gen = PhysicsIdGenerator::new();
        let id1 = id_gen.next();
        let id2 = id_gen.next();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_world_to_physics_registration() {
        // This verifies the rule that anchored blocks ignore simulation
        // while non anchored blocks are correctly discovered.
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();

        // One non anchored block should be registered.
        let coord1 = WorldCoord::new(0, 0, 0);
        world.set_cell(coord1, CellType::Grass);
        if let Some(cell) = world.get_mut(coord1) {
            cell.anchored = false;
        }

        // One anchored block should be ignored.
        let coord2 = WorldCoord::new(1, 0, 0);
        world.set_cell(coord2, CellType::Grass);
        if let Some(cell) = world.get_mut(coord2) {
            cell.anchored = true;
        }

        p_world.register_from_world(&world);

        assert_eq!(p_world.bodies.len(), 1);
        assert_eq!(p_world.bodies[0].position, Vec3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_app_mode_transition_logic() {
        // This proves that registration only happens on the edge of the mode change.
        use crate::engine::EditorMode;

        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let mut last_mode = EditorMode::Editor;
        let mut current_mode = EditorMode::Editor;

        let coord = WorldCoord::new(0, 5, 0);
        world.set_cell(coord, CellType::Grass);
        if let Some(cell) = world.get_mut(coord) {
            cell.anchored = false;
        }

        assert_eq!(p_world.bodies.len(), 0);

        let simulate_update = |mode: EditorMode, last: &mut EditorMode, p: &mut PhysicsWorld, w: &World| {
            if mode == EditorMode::Play && *last == EditorMode::Editor {
                p.register_from_world(w);
            }
            *last = mode;
        };

        // Transition from Editor to Play.
        current_mode = EditorMode::Play;
        simulate_update(current_mode, &mut last_mode, &mut p_world, &world);
        assert_eq!(p_world.bodies.len(), 1);
        let first_id = p_world.bodies[0].id;

        // Run again while already in Play. Registration should not repeat.
        simulate_update(current_mode, &mut last_mode, &mut p_world, &world);
        assert_eq!(p_world.bodies.len(), 1);
        assert_eq!(p_world.bodies[0].id, first_id);
    }

    #[test]
    fn test_physics_gravity_acceleration() {
        // This verifies that world gravity correctly changes velocity over time.
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
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
        // Frozen objects must never receive acceleration from gravity.
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
        // Some bodies might be configured to ignore the gravity field entirely.
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
        // Gravity is a full 3D vector and can pull in any direction.
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

    #[test]
    fn test_physics_position_integration() {
        // Proves that position correctly follows velocity over time.
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::new(10.0, 10.0, 10.0), Vec3::ONE);
        body.velocity = Vec3::new(1.0, 2.0, 3.0);
        p_world.bodies.push(body);

        let dt = 0.1;
        p_world.integrate_positions(dt);

        let expected_pos = Vec3::new(10.1, 10.2, 10.3);
        let actual_pos = p_world.bodies[0].position;
        assert!((actual_pos.x - expected_pos.x).abs() < 1e-5);
        assert!((actual_pos.y - expected_pos.y).abs() < 1e-5);
        assert!((actual_pos.z - expected_pos.z).abs() < 1e-5);
    }

    #[test]
    fn test_physics_integration_anchored_ignored() {
        // Anchored objects are physically static and must not move even if they have velocity.
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        body.velocity = Vec3::new(10.0, 10.0, 10.0);
        body.anchored = true;
        p_world.bodies.push(body);

        p_world.integrate_positions(0.1);
        assert_eq!(p_world.bodies[0].position, Vec3::ZERO);
    }

    #[test]
    fn test_physics_gravity_integration_ordering() {
        // Verifies that applying gravity then integrating position works correctly
        // in a single simulation step.
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), Vec3::ZERO, Vec3::ONE);
        body.velocity = Vec3::ZERO;
        p_world.bodies.push(body);

        let gravity = Vec3::new(0.0, -10.0, 0.0);
        let dt = 0.1;

        // Step simulation: apply gravity THEN integrate
        p_world.apply_gravity(gravity, dt);
        p_world.integrate_positions(dt);

        let expected_vel = Vec3::new(0.0, -1.0, 0.0);
        let expected_pos = Vec3::new(0.0, -0.1, 0.0);

        let actual_vel = p_world.bodies[0].velocity;
        let actual_pos = p_world.bodies[0].position;

        assert!((actual_vel.y - expected_vel.y).abs() < 1e-5);
        assert!((actual_pos.y - expected_pos.y).abs() < 1e-5);
    }
}
