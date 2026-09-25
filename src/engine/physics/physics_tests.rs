#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::world::{CellType, World, WorldCoord};
    use glam::Vec3;

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
            .push(PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE));
        let gravity = Vec3::new(0.0, -9.81, 0.0);
        p_world.apply_gravity(gravity, SIMULATION_DT);
        let expected = Vec3::new(0.0, -9.81 * SIMULATION_DT, 0.0);
        assert!((p_world.bodies[0].velocity.y - expected.y).abs() < 1e-5);
    }

    #[test]
    fn test_physics_gravity_anchored_ignored() {
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE);
        body.anchored = true;
        p_world.bodies.push(body);
        p_world.apply_gravity(Vec3::new(0.0, -10.0, 0.0), 0.1);
        assert_eq!(p_world.bodies[0].velocity, Vec3::ZERO);
    }

    #[test]
    fn test_physics_gravity_participation_gating() {
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE);
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
            .push(PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE));
        let gravity = Vec3::new(1.0, 2.0, -3.0);
        p_world.apply_gravity(gravity, 0.5);
        assert!((p_world.bodies[0].velocity - Vec3::new(0.5, 1.0, -1.5)).length() < 1e-5);
    }

    #[test]
    fn test_physics_position_integration() {
        let mut p_world = PhysicsWorld::new();
        let mut body =
            PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(10.0, 10.0, 10.0), Vec3::ONE);
        body.velocity = Vec3::new(1.0, 2.0, 3.0);
        p_world.bodies.push(body);
        p_world.integrate_positions(0.1);
        assert!((p_world.bodies[0].position - Vec3::new(10.1, 10.2, 10.3)).length() < 1e-5);
    }

    #[test]
    fn test_physics_integration_anchored_ignored() {
        let mut p_world = PhysicsWorld::new();
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE);
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
            .push(PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE));
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
        assert!(
            p_world
                .static_colliders
                .iter()
                .any(|(_, pos)| *pos == Vec3::new(c1.x as f32, c1.y as f32, c1.z as f32))
        );
    }

    #[test]
    fn test_collision_detection() {
        let mut p_world = PhysicsWorld::new();
        p_world.add_static_collider(0, Vec3::ZERO);
        p_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(1),
            0,
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::ONE,
        ));
        assert_eq!(p_world.check_static_collisions().len(), 1);
    }

    #[test]
    fn test_physics_floor_collision_resolution() {
        let mut p_world = PhysicsWorld::new();
        p_world.add_static_collider(0, Vec3::ZERO);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(0.0, 0.9, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(0.0, -10.0, 0.0);
        p_world.bodies.push(body);
        p_world.resolve_static_collisions();
        assert!(p_world.bodies[0].position.y >= 1.0);
        assert_eq!(p_world.bodies[0].velocity.y, 0.0);
    }

    #[test]
    fn test_physics_tangential_velocity_preserved() {
        let mut p_world = PhysicsWorld::new();
        p_world.add_static_collider(0, Vec3::ZERO);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(0.0, 0.9, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(5.0, -10.0, 0.0);
        p_world.bodies.push(body);
        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].velocity.y, 0.0);
        assert_eq!(p_world.bodies[0].velocity.x, 5.0);
    }

    #[test]
    fn test_physics_wall_collision_resolution() {
        let mut p_world = PhysicsWorld::new();
        p_world.add_static_collider(0, Vec3::ZERO);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(-0.1, 0.0, 0.0), Vec3::ONE);
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
            0,
            Vec3::new(0.0, 0.9, 0.0),
            Vec3::ONE,
        ));
        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].position.y, 0.9);
    }

    #[test]
    fn test_physics_touching_no_push() {
        let mut p_world = PhysicsWorld::new();
        p_world.add_static_collider(0, Vec3::ZERO);
        p_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(1),
            0,
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::ONE,
        ));
        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].position.y, 1.0);
    }

    #[test]
    fn test_physics_adjacent_floors_stable() {
        let mut p_world = PhysicsWorld::new();
        p_world.add_static_collider(0, Vec3::ZERO);
        p_world.add_static_collider(0, Vec3::ZERO);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(0.5, 0.9, 0.0), Vec3::ONE);
        body.velocity = Vec3::new(0.0, -10.0, 0.0);
        p_world.bodies.push(body);
        p_world.resolve_static_collisions();
        assert!((p_world.bodies[0].position.y - 1.0).abs() < 0.001);
        assert_eq!(p_world.bodies[0].velocity.y, 0.0);
    }

    #[test]
    fn test_physics_touching_adjacent_regression() {
        let mut p_world = PhysicsWorld::new();
        p_world.add_static_collider(0, Vec3::ZERO);
        let pos = Vec3::new(0.0, 0.0, -1.0);
        p_world
            .bodies
            .push(PhysicsBody::new(PhysicsBodyId(1), 0, pos, Vec3::ONE));
        p_world.resolve_static_collisions();
        assert_eq!(p_world.bodies[0].position, pos);
    }

    #[test]
    fn test_physics_resting_contact_stability() {
        let mut p_world = PhysicsWorld::new();
        let gravity = Vec3::new(0.0, -10.0, 0.0);
        let dt = SIMULATION_DT;
        p_world.add_static_collider(0, Vec3::ZERO);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(0.0, 0.9, 0.0), Vec3::ONE);
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
        p_world.add_static_collider(0, Vec3::ZERO);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(0.0, 1.0, 0.0), Vec3::ONE);
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
        p_world.add_static_collider(0, Vec3::ZERO);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(0.0, 1.0, 0.0), Vec3::ONE);
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
        p_world.add_static_collider(0, Vec3::ZERO);
        let body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(-0.15, 0.0, 0.0), Vec3::ONE);
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
            .push(PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE));
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
        p_world.add_static_collider(0, Vec3::ZERO);
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(0.0, 0.0, 0.0), Vec3::ONE);
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
        let mut b1 = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE);
        b1.is_sleeping = true;
        let mut b2 = PhysicsBody::new(PhysicsBodyId(2), 0, Vec3::Y, Vec3::ONE);
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
        let b1 = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE);
        let mut b2 = PhysicsBody::new(PhysicsBodyId(2), 0, Vec3::Y, Vec3::ONE);
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
        let mut b1 = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE);
        b1.is_sleeping = true;
        let mut b2 = PhysicsBody::new(PhysicsBodyId(2), 0, Vec3::Y, Vec3::ONE);
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
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE);
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
        let mut body = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE);
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
        p_world.add_static_collider(0, Vec3::ZERO);
        p_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(1),
            0,
            Vec3::new(0.0, 5.0, 0.0),
            Vec3::ONE,
        ));
        p_world.bodies.push(PhysicsBody::new(
            PhysicsBodyId(2),
            0,
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
        let b1 = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(0.0, 0.0, 0.0), Vec3::ONE);
        let b2 = PhysicsBody::new(PhysicsBodyId(2), 0, Vec3::new(0.0, 1.0, 0.0), Vec3::ONE);
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
        let b1 = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(0.0, 0.0, 0.0), Vec3::ONE);
        let b2 = PhysicsBody::new(PhysicsBodyId(2), 0, Vec3::new(1.0, 0.0, 0.0), Vec3::ONE);
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
        p_world.add_static_collider(0, Vec3::ZERO); // Wall at X=2

        let mut b1 = PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::new(1.0, 0.0, 0.0), Vec3::ONE);
        b1.static_contact_normal = Some(Vec3::NEG_X); // Supported by wall
        b1.is_sleeping = true;

        let mut b2 = PhysicsBody::new(PhysicsBodyId(2), 0, Vec3::new(0.0, 0.0, 0.0), Vec3::ONE);
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
            .push(PhysicsBody::new(PhysicsBodyId(1), 0, Vec3::ZERO, Vec3::ONE));
        p_world
            .bodies
            .push(PhysicsBody::new(PhysicsBodyId(2), 0, Vec3::Y, Vec3::ONE));
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

    #[test]
    fn test_sync_no_dirty_no_scan() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);
        p_world.register_from_world(&world);

        // Drain any initial dirty marks from registration.
        world.physics_dirty_cells.clear();

        let initial_count = p_world.static_colliders.len();
        p_world.sync_with_world(&mut world);

        assert_eq!(p_world.static_colliders.len(), initial_count);
        // Since dirty set is empty, it shouldn't have changed anything.
    }

    #[test]
    fn test_sync_offset_movement() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let coord = WorldCoord::new(10, 10, 10);
        world.set_cell(coord, CellType::Block);
        p_world.register_from_world(&world);

        // Move via runtime offset
        world.set_visual_offset_runtime(coord, Vec3::new(0.0, 5.0, 0.0));
        p_world.sync_with_world(&mut world);

        let (_, pos) = p_world.static_colliders[0];
        assert_eq!(pos, Vec3::new(10.0, 15.0, 10.0));
    }

    #[test]
    fn test_sync_offset_reset() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let coord = WorldCoord::new(0, 0, 0);
        world.set_cell(coord, CellType::Block);
        world.set_visual_offset_runtime(coord, Vec3::new(0.0, 10.0, 0.0));
        p_world.register_from_world(&world);

        // Clear runtime state
        world.clear_runtime_state();
        p_world.sync_with_world(&mut world);

        let (_, pos) = p_world.static_colliders[0];
        assert_eq!(pos, Vec3::ZERO);
    }

    #[test]
    fn test_sync_solid_toggle() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let coord = WorldCoord::new(0, 0, 0);
        world.set_cell(coord, CellType::Block);
        p_world.register_from_world(&world);
        assert_eq!(p_world.static_colliders.len(), 1);

        // Toggle solid off
        world.set_cell_solid_runtime(coord, false);
        p_world.sync_with_world(&mut world);
        assert_eq!(p_world.static_colliders.len(), 0);

        // Toggle solid back on
        world.set_cell_solid_runtime(coord, true);
        p_world.sync_with_world(&mut world);
        assert_eq!(p_world.static_colliders.len(), 1);
    }

    #[test]
    fn test_sync_anchored_toggle() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let coord = WorldCoord::new(0, 0, 0);
        world.set_cell(coord, CellType::Block);
        p_world.register_from_world(&world);
        assert_eq!(p_world.static_colliders.len(), 1);
        assert_eq!(p_world.bodies.len(), 0);

        // Toggle anchored off (becomes dynamic)
        world.set_cell_anchored_runtime(coord, false);
        p_world.sync_with_world(&mut world);
        assert_eq!(p_world.static_colliders.len(), 0);
        assert_eq!(p_world.bodies.len(), 1);

        // Toggle anchored back on (becomes static)
        world.set_cell_anchored_runtime(coord, true);
        p_world.sync_with_world(&mut world);
        assert_eq!(p_world.static_colliders.len(), 1);
        assert_eq!(p_world.bodies.len(), 0);
    }

    #[test]
    fn test_sync_multiple_mutations() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let coord = WorldCoord::new(0, 0, 0);
        world.set_cell(coord, CellType::Block);
        p_world.register_from_world(&world);

        // Multiple offsets before sync
        world.set_visual_offset_runtime(coord, Vec3::new(0.0, 1.0, 0.0));
        world.set_visual_offset_runtime(coord, Vec3::new(0.0, 2.0, 0.0));
        world.set_visual_offset_runtime(coord, Vec3::new(0.0, 3.0, 0.0));

        p_world.sync_with_world(&mut world);
        assert_eq!(p_world.static_colliders[0].1, Vec3::new(0.0, 3.0, 0.0));
    }

    #[test]
    fn test_sync_deleted_cell() {
        let mut world = World::new();
        let mut p_world = PhysicsWorld::new();
        let coord = WorldCoord::new(0, 0, 0);
        world.set_cell(coord, CellType::Block);
        let cell_id = world.get(coord).unwrap().id;
        p_world.register_from_world(&world);
        assert_eq!(p_world.static_colliders.len(), 1);

        // Mark dirty then delete
        world.mark_physics_dirty(cell_id);
        world.set_cell(coord, CellType::Empty);

        p_world.sync_with_world(&mut world);
        assert_eq!(p_world.static_colliders.len(), 0);
    }
}
