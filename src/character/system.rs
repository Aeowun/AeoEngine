use super::animation::AnimationState;
use super::character::Character;
use super::movement::{JUMP_IMPULSE, MOVE_SPEED, MovementState};
use super::spawning::spawn_at_random_point;
use crate::character_custom::TargetAnimation;
use crate::world::World;
use glam::{Quat, Vec2, Vec3};
use std::collections::HashMap;
use std::path::Path;

/// Owns all runtime character state.
///
/// CharacterSystem is authoritative for:
/// - character transform
/// - gravity
/// - movement velocity
/// - grounded state
/// - collision resolution
/// - animation state
///
/// AeoScript may control horizontal velocity, facing, animation selection,
/// jumping, and explicit teleports through the EngineHost bridge.
#[derive(Default)]
pub struct CharacterSystem {
    characters: HashMap<u64, Character>,

    /// Explicit active player identity.
    ///
    /// HashMap iteration order is not a gameplay contract, so runtime player
    /// access should use this ID whenever possible.
    active_player_id: Option<u64>,

    next_id: u64,
}

impl CharacterSystem {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            active_player_id: None,
            next_id: 1,
        }
    }

    /// Spawns a player character at a valid world spawn point.
    ///
    /// `project_path` is currently reserved for project-specific character
    /// configuration. Keeping it on this boundary avoids forcing callers to
    /// change again when character asset selection becomes project-aware.
    pub fn spawn_player(
        &mut self,
        world: &World,
        project_path: Option<&Path>,
    ) -> Option<u64> {
        let character =
            spawn_at_random_point(world, self.next_id, project_path)?;

        let id = character.id;

        self.characters.insert(id, character);
        self.active_player_id = Some(id);
        self.next_id += 1;

        Some(id)
    }

    /// Returns all active runtime characters.
    pub fn get_active_characters(
        &self,
    ) -> impl Iterator<Item = &Character> {
        self.characters.values()
    }

    /// Returns all active runtime characters mutably.
    pub fn get_active_characters_mut(
        &mut self,
    ) -> impl Iterator<Item = &mut Character> {
        self.characters.values_mut()
    }

    /// Returns the authoritative active player.
    ///
    /// The fallback keeps tests and older code that directly insert a
    /// Character into the HashMap working.
    pub fn get_active_player(&self) -> Option<&Character> {
        if let Some(id) = self.active_player_id {
            if let Some(character) = self.characters.get(&id) {
                return Some(character);
            }
        }

        self.characters.values().next()
    }

    /// Returns the authoritative active player mutably.
    pub fn get_active_player_mut(
        &mut self,
    ) -> Option<&mut Character> {
        if let Some(id) = self.active_player_id {
            if self.characters.contains_key(&id) {
                return self.characters.get_mut(&id);
            }
        }

        self.characters.values_mut().next()
    }

    /// Clears all runtime character state.
    pub fn clear(&mut self) {
        self.characters.clear();
        self.active_player_id = None;

        // Keep runtime IDs deterministic across play sessions.
        self.next_id = 1;
    }

    pub fn has_characters(&self) -> bool {
        !self.characters.is_empty()
    }

    /// Explicitly teleports the active player.
    ///
    /// This function is intentionally NOT part of normal synchronization.
    /// It should only be called when gameplay code explicitly requests a
    /// position change through the existing entity.set_position() API.
    pub fn set_active_position(&mut self, position: Vec3) -> bool {
        let Some(character) = self.get_active_player_mut() else {
            return false;
        };

        character.transform.position = position;

        // A teleport invalidates momentum from the old location.
        character.movement.velocity = Vec3::ZERO;

        // Let the next collision pass establish grounded state at the new
        // location instead of inheriting the old floor state.
        character.movement.is_grounded = false;

        true
    }

    /// Legacy engine-controlled movement path.
    ///
    /// The engine supplies directional input. CharacterSystem converts it to
    /// horizontal velocity, handles jumping, then performs normal physics.
    pub fn update(
        &mut self,
        world: &World,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        dt: f32,
        input: Vec2,
        jump_requested: bool,
    ) -> std::collections::HashSet<u64> {
        self.update_internal(
            world,
            physics_world,
            dt,
            Some(input),
            jump_requested,
            false,
        )
    }

    /// AeoScript-controlled movement path.
    ///
    /// The script/controller supplies horizontal velocity and facing.
    /// CharacterSystem remains responsible for gravity, integration,
    /// collision resolution, grounded state, and animation evaluation.
    pub fn update_scripted(
        &mut self,
        world: &World,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        dt: f32,
    ) -> std::collections::HashSet<u64> {
        self.update_internal(
            world,
            physics_world,
            dt,
            None,
            false,
            true,
        )
    }

    fn update_internal(
        &mut self,
        world: &World,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        dt: f32,
        input: Option<Vec2>,
        jump_requested: bool,
        scripted: bool,
    ) -> std::collections::HashSet<u64> {
        let mut contacted_cells =
            std::collections::HashSet::new();

        for character in self.characters.values_mut() {
            if !scripted {
                let horizontal_velocity = match input {
                    Some(input) if input.length_squared() > 0.001 => {
                        input.normalize() * MOVE_SPEED
                    }

                    _ => Vec2::ZERO,
                };

                character.movement.velocity.x =
                    horizontal_velocity.x;
                character.movement.velocity.z =
                    horizontal_velocity.y;

                if jump_requested
                    && character.movement.is_grounded
                {
                    character.movement.velocity.y =
                        JUMP_IMPULSE;

                    character.movement.is_grounded = false;
                }

                // Legacy engine movement controls facing direction.
                if horizontal_velocity.length_squared() > 0.001
                {
                    let angle = f32::atan2(
                        horizontal_velocity.x,
                        horizontal_velocity.y,
                    );

                    character.transform.rotation =
                        Quat::from_rotation_y(angle);
                }
            }

            // Scripted movement already owns X/Z velocity. CharacterSystem
            // remains responsible for vertical gravity.
            character.movement.velocity +=
                world.gravity * dt;

            character.transform.position +=
                character.movement.velocity * dt;

            Self::resolve_static_voxel_collisions(
                character,
                physics_world,
                &mut contacted_cells,
            );

            Self::resolve_dynamic_body_collisions(
                character,
                physics_world,
                &mut contacted_cells,
            );

            // Animation is based on actual resulting horizontal velocity.
            // This keeps animation correct for both engine-controlled and
            // AeoScript-controlled movement.
            let horizontal_speed = Vec2::new(
                character.movement.velocity.x,
                character.movement.velocity.z,
            )
            .length();

            if horizontal_speed > 0.1 {
                character.animation.current_state =
                    AnimationState::Walk;

                character.movement.state =
                    MovementState::Walk;

                character
                    .animation_controller
                    .select_animation(TargetAnimation::Walk);
            } else {
                character.animation.current_state =
                    AnimationState::Idle;

                character.movement.state =
                    MovementState::Idle;

                character
                    .animation_controller
                    .select_animation(TargetAnimation::Idle);
            }

            // Advance animation blending/time independently of who controls
            // movement.
            character.animation_controller.update(dt);

            // Produce the final pose consumed by rendering.
            character.current_pose =
                character.animation_controller.evaluate_pose();
        }

        contacted_cells
    }

    fn resolve_static_voxel_collisions(
        character: &mut Character,
        physics_world: &crate::engine::physics::PhysicsWorld,
        contacted_cells: &mut std::collections::HashSet<u64>,
    ) {
        let radius = character.collision.radius;
        let height = character.collision.height;

        // Grounded state is derived fresh from collision every fixed step.
        character.movement.is_grounded = false;

        for (cell_id, v_min) in &physics_world.static_colliders {
            let v_max = *v_min + Vec3::ONE;

            let overlap_x = (character.transform.position.x + radius)
                .min(v_max.x)
                - (character.transform.position.x - radius)
                    .max(v_min.x);

            let overlap_y = (character.transform.position.y + height)
                .min(v_max.y)
                - character.transform.position.y.max(v_min.y);

            let overlap_z = (character.transform.position.z + radius)
                .min(v_max.z)
                - (character.transform.position.z - radius)
                    .max(v_min.z);

            if overlap_x > 0.001
                && overlap_y > 0.001
                && overlap_z > 0.001
            {
                contacted_cells.insert(*cell_id);

                // Resolve along the shallowest penetration axis.
                if overlap_y < overlap_x
                    && overlap_y < overlap_z
                {
                    if character.transform.position.y < v_min.y {
                        // Head collision.
                        character.transform.position.y -=
                            overlap_y;

                        if character.movement.velocity.y > 0.0 {
                            character.movement.velocity.y = 0.0;
                        }
                    } else {
                        // Floor collision.
                        character.transform.position.y +=
                            overlap_y;

                        character.movement.is_grounded = true;

                        if character.movement.velocity.y < 0.0 {
                            character.movement.velocity.y = 0.0;
                        }
                    }
                } else if overlap_x < overlap_z {
                    if character.transform.position.x < v_min.x {
                        character.transform.position.x -=
                            overlap_x;
                    } else {
                        character.transform.position.x +=
                            overlap_x;
                    }

                    character.movement.velocity.x = 0.0;
                } else {
                    if character.transform.position.z < v_min.z {
                        character.transform.position.z -=
                            overlap_z;
                    } else {
                        character.transform.position.z +=
                            overlap_z;
                    }

                    character.movement.velocity.z = 0.0;
                }
            }
        }
    }

    fn resolve_dynamic_body_collisions(
        character: &mut Character,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        contacted_cells: &mut std::collections::HashSet<u64>,
    ) {
        let radius = character.collision.radius;
        let height = character.collision.height;

        for body in &mut physics_world.bodies {
            if body.anchored || !body.solid {
                continue;
            }

            let v_min = body.min_corner();
            let v_max = body.max_corner();

            let overlap_x = (character.transform.position.x + radius)
                .min(v_max.x)
                - (character.transform.position.x - radius)
                    .max(v_min.x);

            let overlap_y = (character.transform.position.y + height)
                .min(v_max.y)
                - character.transform.position.y.max(v_min.y);

            let overlap_z = (character.transform.position.z + radius)
                .min(v_max.z)
                - (character.transform.position.z - radius)
                    .max(v_min.z);

            if overlap_x > 0.001
                && overlap_y > 0.001
                && overlap_z > 0.001
            {
                if body.cell_id != 0 {
                    contacted_cells.insert(body.cell_id);
                }

                if overlap_y < overlap_x
                    && overlap_y < overlap_z
                {
                    if character.transform.position.y < v_min.y {
                        character.transform.position.y -=
                            overlap_y;

                        if character.movement.velocity.y > 0.0 {
                            character.movement.velocity.y = 0.0;
                        }
                    } else {
                        character.transform.position.y +=
                            overlap_y;

                        character.movement.is_grounded = true;

                        if character.movement.velocity.y < 0.0 {
                            character.movement.velocity.y = 0.0;
                        }
                    }
                } else if overlap_x < overlap_z {
                    if character.transform.position.x < v_min.x {
                        character.transform.position.x -=
                            overlap_x;
                    } else {
                        character.transform.position.x +=
                            overlap_x;
                    }

                    character.movement.velocity.x = 0.0;
                } else {
                    if character.transform.position.z < v_min.z {
                        character.transform.position.z -=
                            overlap_z;
                    } else {
                        character.transform.position.z +=
                            overlap_z;
                    }

                    character.movement.velocity.z = 0.0;
                }

                // Character interaction wakes a dynamic object that was
                // sleeping when contact occurred.
                body.wake();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::movement;
    use crate::world::{CellType, WorldCoord};

    #[test]
    fn test_character_runtime_starts_empty() {
        let system = CharacterSystem::new();

        assert!(!system.has_characters());
        assert_eq!(
            system.get_active_characters().count(),
            0
        );
        assert!(system.get_active_player().is_none());
    }

    #[test]
    fn test_adding_character_stores_it() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::SpawnPoint,
        );

        system.spawn_player(&world, None);

        assert!(system.has_characters());
        assert_eq!(
            system.get_active_characters().count(),
            1
        );
        assert!(system.get_active_player().is_some());
    }

    #[test]
    fn test_runtime_ids_remain_unique() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::SpawnPoint,
        );

        world.set_cell(
            WorldCoord::new(5, 0, 5),
            CellType::SpawnPoint,
        );

        system.spawn_player(&world, None);

        let id1 = system
            .get_active_characters()
            .next()
            .unwrap()
            .id;

        system.spawn_player(&world, None);

        let id2 = system
            .get_active_characters()
            .find(|character| character.id != id1)
            .unwrap()
            .id;

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_clear_removes_all_characters() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::SpawnPoint,
        );

        system.spawn_player(&world, None);

        assert!(system.has_characters());

        system.clear();

        assert!(!system.has_characters());
        assert!(system.get_active_player().is_none());
    }

    #[test]
    fn test_character_teleport() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::SpawnPoint,
        );

        system.spawn_player(&world, None);

        let player = system
            .get_active_player_mut()
            .unwrap();

        player.movement.velocity =
            Vec3::new(4.0, -8.0, 2.0);
        player.movement.is_grounded = true;

        let target = Vec3::new(
            10.0,
            20.0,
            30.0,
        );

        assert!(system.set_active_position(target));

        let player =
            system.get_active_player().unwrap();

        assert_eq!(
            player.transform.position,
            target
        );

        assert_eq!(
            player.movement.velocity,
            Vec3::ZERO
        );

        assert!(!player.movement.is_grounded);
    }

    #[test]
    fn test_character_gravity() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character =
            Character::new(1, Vec3::new(0.0, 10.0, 0.0));

        system.characters.insert(1, character);
        system.active_player_id = Some(1);

        let mut physics_world =
            crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        for _ in 0..60 {
            let _ = system.update(
                &world,
                &mut physics_world,
                1.0 / 60.0,
                Vec2::ZERO,
                false,
            );
        }

        let updated =
            system.get_active_player().unwrap();

        assert!(updated.transform.position.y < 10.0);
        assert!(updated.movement.velocity.y < 0.0);
    }

    #[test]
    fn test_character_horizontal_movement() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character =
            Character::new(1, Vec3::ZERO);

        system.characters.insert(1, character);
        system.active_player_id = Some(1);

        let mut physics_world =
            crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let _ = system.update(
            &world,
            &mut physics_world,
            0.1,
            Vec2::new(1.0, 0.0),
            false,
        );

        let updated =
            system.get_active_player().unwrap();

        assert!(updated.transform.position.x > 0.0);
        assert_eq!(
            updated.animation.current_state,
            AnimationState::Walk
        );
        assert!(
            updated
                .animation_controller
                .blend_weight()
                > 0.0
        );
    }

    #[test]
    fn test_scripted_horizontal_movement_preserves_velocity() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character =
            Character::new(1, Vec3::ZERO);

        system.characters.insert(1, character);
        system.active_player_id = Some(1);

        let mut physics_world =
            crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        {
            let player =
                system.get_active_player_mut().unwrap();

            player.movement.velocity.x = 5.0;
            player.movement.velocity.z = -2.0;
        }

        let _ = system.update_scripted(
            &world,
            &mut physics_world,
            0.1,
        );

        let updated =
            system.get_active_player().unwrap();

        assert!(
            updated.transform.position.x > 0.49
        );

        assert!(
            updated.transform.position.z < -0.19
        );

        assert_eq!(
            updated.animation.current_state,
            AnimationState::Walk
        );
    }

    #[test]
    fn test_character_ground_collision() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::Block,
        );

        let mut physics_world =
            crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let character =
            Character::new(
                1,
                Vec3::new(0.0, 1.5, 0.0),
            );

        system.characters.insert(1, character);
        system.active_player_id = Some(1);

        for _ in 0..60 {
            let _ = system.update(
                &world,
                &mut physics_world,
                1.0 / 60.0,
                Vec2::ZERO,
                false,
            );
        }

        let updated =
            system.get_active_player().unwrap();

        assert!(
            (updated.transform.position.y - 1.0).abs()
                < 0.01
        );

        assert!(updated.movement.is_grounded);
    }

    #[test]
    fn test_animation_state_switching() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character =
            Character::new(1, Vec3::ZERO);

        system.characters.insert(1, character);
        system.active_player_id = Some(1);

        let mut physics_world =
            crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let _ = system.update(
            &world,
            &mut physics_world,
            0.1,
            Vec2::ZERO,
            false,
        );

        assert_eq!(
            system
                .get_active_player()
                .unwrap()
                .animation
                .current_state,
            AnimationState::Idle
        );

        let _ = system.update(
            &world,
            &mut physics_world,
            0.1,
            Vec2::X,
            false,
        );

        assert_eq!(
            system
                .get_active_player()
                .unwrap()
                .animation
                .current_state,
            AnimationState::Walk
        );
    }

    #[test]
    fn test_character_facing_direction() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character =
            Character::new(1, Vec3::ZERO);

        system.characters.insert(1, character);
        system.active_player_id = Some(1);

        let mut physics_world =
            crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let _ = system.update(
            &world,
            &mut physics_world,
            0.1,
            Vec2::new(1.0, 0.0),
            false,
        );

        let rotation_right =
            system
                .get_active_player()
                .unwrap()
                .transform
                .rotation;

        let _ = system.update(
            &world,
            &mut physics_world,
            0.1,
            Vec2::new(-1.0, 0.0),
            false,
        );

        let rotation_left =
            system
                .get_active_player()
                .unwrap()
                .transform
                .rotation;

        assert_ne!(rotation_right, rotation_left);

        let _ = system.update(
            &world,
            &mut physics_world,
            0.1,
            Vec2::ZERO,
            false,
        );

        let rotation_idle =
            system
                .get_active_player()
                .unwrap()
                .transform
                .rotation;

        assert_eq!(rotation_left, rotation_idle);
    }

    #[test]
    fn test_character_jumping() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::Block,
        );

        let mut physics_world =
            crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let character =
            Character::new(
                1,
                Vec3::new(0.0, 1.0, 0.0),
            );

        system.characters.insert(1, character);
        system.active_player_id = Some(1);

        let _ = system.update(
            &world,
            &mut physics_world,
            1.0 / 60.0,
            Vec2::ZERO,
            false,
        );

        assert!(
            system
                .get_active_player()
                .unwrap()
                .movement
                .is_grounded
        );

        let _ = system.update(
            &world,
            &mut physics_world,
            1.0 / 60.0,
            Vec2::ZERO,
            true,
        );

        let updated =
            system.get_active_player().unwrap();

        let expected_velocity =
            JUMP_IMPULSE
                + world.gravity.y * (1.0 / 60.0);

        assert!(
            (updated.movement.velocity.y
                - expected_velocity)
                .abs()
                < 0.001
        );

        assert!(!updated.movement.is_grounded);

        let _ = system.update(
            &world,
            &mut physics_world,
            1.0 / 60.0,
            Vec2::ZERO,
            true,
        );

        assert!(
            system
                .get_active_player()
                .unwrap()
                .movement
                .velocity
                .y
                < movement::JUMP_IMPULSE
        );
    }

    #[test]
    fn test_character_collision_with_dynamic_bodies() {
        use crate::engine::physics::{
            PhysicsBody,
            PhysicsBodyId,
            PhysicsWorld,
        };

        let mut system = CharacterSystem::new();
        let world = World::new();
        let mut physics_world = PhysicsWorld::new();

        let mut character =
            Character::new(1, Vec3::ZERO);

        character.collision.radius = 0.5;
        system.characters.insert(1, character);
        system.active_player_id = Some(1);

        let body_id = PhysicsBodyId(1);

        let mut body = PhysicsBody::new(
            body_id,
            0,
            Vec3::new(0.6, 0.0, -0.5),
            Vec3::ONE,
        );

        body.solid = true;
        body.anchored = false;

        physics_world.bodies.push(body);

        let _ = system.update(
            &world,
            &mut physics_world,
            0.1,
            Vec2::new(1.0, 0.0),
            false,
        );

        let updated =
            system.get_active_player().unwrap();

        assert!(
            updated.transform.position.x < 0.4,
            "Character should be blocked by awake body"
        );

        assert!(
            (updated.transform.position.x - 0.1).abs()
                < 0.01
        );

        physics_world.bodies[0].is_sleeping = true;
        physics_world.bodies[0].position =
            Vec3::new(0.6, 0.0, -0.5);

        system
            .get_active_player_mut()
            .unwrap()
            .transform
            .position = Vec3::ZERO;

        let _ = system.update(
            &world,
            &mut physics_world,
            0.1,
            Vec2::new(1.0, 0.0),
            false,
        );

        let updated =
            system.get_active_player().unwrap();

        assert!(
            updated.transform.position.x < 0.4
        );

        assert!(
            !physics_world.bodies[0].is_sleeping
        );

        physics_world.bodies[0].solid = false;

        system
            .get_active_player_mut()
            .unwrap()
            .transform
            .position = Vec3::ZERO;

        let _ = system.update(
            &world,
            &mut physics_world,
            0.1,
            Vec2::new(1.0, 0.0),
            false,
        );

        let updated =
            system.get_active_player().unwrap();

        assert!(
            (updated.transform.position.x - 0.4).abs()
                < 0.01
        );
    }

    #[test]
    fn test_anchored_cell_runtime_movement_collision() {
        use crate::engine::physics::PhysicsWorld;

        let mut system = CharacterSystem::new();
        let mut world = World::new();
        let mut physics_world = PhysicsWorld::new();

        let coord = WorldCoord::new(0, 0, 0);

        world.set_cell(coord, CellType::Block);

        let cell_id = world.get(coord).unwrap().id;

        physics_world.register_from_world(&world);

        assert_eq!(
            physics_world.static_colliders.len(),
            1
        );

        let character =
            Character::new(
                1,
                Vec3::new(0.0, 1.5, 0.0),
            );

        system.characters.insert(1, character);
        system.active_player_id = Some(1);

        for _ in 0..60 {
            let _ = system.update(
                &world,
                &mut physics_world,
                1.0 / 60.0,
                Vec2::ZERO,
                false,
            );
        }

        let updated =
            system.get_active_player().unwrap();

        assert!(
            (updated.transform.position.y - 1.0).abs()
                < 0.01
        );

        assert!(updated.movement.is_grounded);

        world.set_visual_offset_runtime(
            coord,
            Vec3::new(0.0, 2.0, 0.0),
        );

        physics_world.sync_with_world(&mut world);

        system
            .get_active_player_mut()
            .unwrap()
            .transform
            .position =
            Vec3::new(0.0, 1.5, 0.0);

        system
            .get_active_player_mut()
            .unwrap()
            .movement
            .is_grounded = false;

        system
            .get_active_player_mut()
            .unwrap()
            .movement
            .velocity = Vec3::ZERO;

        for _ in 0..10 {
            let _ = system.update(
                &world,
                &mut physics_world,
                1.0 / 60.0,
                Vec2::ZERO,
                false,
            );
        }

        let updated =
            system.get_active_player().unwrap();

        assert!(
            updated.transform.position.y < 1.5
        );

        assert!(!updated.movement.is_grounded);

        system
            .get_active_player_mut()
            .unwrap()
            .transform
            .position =
            Vec3::new(0.0, 3.5, 0.0);

        for _ in 0..60 {
            let _ = system.update(
                &world,
                &mut physics_world,
                1.0 / 60.0,
                Vec2::ZERO,
                false,
            );
        }

        let updated =
            system.get_active_player().unwrap();

        assert!(
            (updated.transform.position.y - 3.0).abs()
                < 0.01
        );

        assert!(updated.movement.is_grounded);

        world.set_visual_offset_runtime(
            coord,
            Vec3::new(0.0, 4.0, 0.0),
        );

        physics_world.sync_with_world(&mut world);

        system
            .get_active_player_mut()
            .unwrap()
            .transform
            .position =
            Vec3::new(0.0, 5.5, 0.0);

        for _ in 0..60 {
            let _ = system.update(
                &world,
                &mut physics_world,
                1.0 / 60.0,
                Vec2::ZERO,
                false,
            );
        }

        let updated =
            system.get_active_player().unwrap();

        assert!(
            (updated.transform.position.y - 5.0).abs()
                < 0.01
        );

        world.clear_runtime_state();
        physics_world.sync_with_world(&mut world);

        system
            .get_active_player_mut()
            .unwrap()
            .transform
            .position =
            Vec3::new(0.0, 1.5, 0.0);

        for _ in 0..60 {
            let _ = system.update(
                &world,
                &mut physics_world,
                1.0 / 60.0,
                Vec2::ZERO,
                false,
            );
        }

        let updated =
            system.get_active_player().unwrap();

        assert!(
            (updated.transform.position.y - 1.0).abs()
                < 0.01
        );

        assert!(world.get(coord).is_some());
        assert_eq!(world.get(coord).unwrap().id, cell_id);
    }
}