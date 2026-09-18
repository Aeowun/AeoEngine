use super::animation::AnimationState;
use super::character::Character;
use super::movement::{JUMP_IMPULSE, MOVE_SPEED, MovementState};
use super::spawning::spawn_at_random_point;
use crate::character_custom::TargetAnimation;
use crate::world::{World, WorldCoord};
use glam::{Quat, Vec2, Vec3};
use std::collections::HashMap;

#[derive(Default)]
pub struct CharacterSystem {
    characters: HashMap<u64, Character>,
    next_id: u64,
}

impl CharacterSystem {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            next_id: 1,
        }
    }

    /// Spawns exactly one player character if a valid spawn point exists.
    pub fn spawn_player(&mut self, world: &World) {
        if let Some(character) = spawn_at_random_point(world, self.next_id) {
            self.characters.insert(character.id, character);
            self.next_id += 1;
        }
    }

    /// Returns an iterator over all active runtime characters.
    pub fn get_active_characters(&self) -> impl Iterator<Item = &Character> {
        self.characters.values()
    }

    /// Discards all runtime character state.
    pub fn clear(&mut self) {
        self.characters.clear();
        // We reset next_id to maintain deterministic numbering for fresh play sessions.
        self.next_id = 1;
    }

    /// Returns true if any characters are currently active.
    pub fn has_characters(&self) -> bool {
        !self.characters.is_empty()
    }

    /// Performs a fixed simulation step for all active characters.
    pub fn update(&mut self, world: &World, dt: f32, input: Vec2, jump_requested: bool) {
        for character in self.characters.values_mut() {
            // 1. Horizontal Movement
            // We apply input directly to horizontal velocity.
            let horizontal_vel = if input.length_squared() > 0.001 {
                input.normalize() * MOVE_SPEED
            } else {
                Vec2::ZERO
            };

            character.movement.velocity.x = horizontal_vel.x;
            character.movement.velocity.z = horizontal_vel.y;

            // 2. Jumping
            // Jump is only accepted if the character is grounded.
            if jump_requested && character.movement.is_grounded {
                character.movement.velocity.y = JUMP_IMPULSE;
                // Character is no longer grounded immediately after jumping.
                character.movement.is_grounded = false;
            }

            // 3. Gravity
            character.movement.velocity += world.gravity * dt;

            // 4. Position Integration
            let next_pos = character.transform.position + character.movement.velocity * dt;
            character.transform.position = next_pos;

            // 5. Orientation
            // Character faces the direction of horizontal movement.
            if horizontal_vel.length_squared() > 0.001 {
                let angle = f32::atan2(horizontal_vel.x, horizontal_vel.y);
                character.transform.rotation = Quat::from_rotation_y(angle);
            }

            // 6. Voxel Collision Resolution
            // Note: We use a static helper function to avoid borrow checker errors
            // when accessing world while iterating characters.
            Self::resolve_voxel_collisions(character, world);

            // 7. Animation State Handoff
            let horizontal_speed = horizontal_vel.length();
            if horizontal_speed > 0.1 {
                character.animation.current_state = AnimationState::Walk;
                character.movement.state = MovementState::Walk;
                character
                    .animation_controller
                    .select_animation(TargetAnimation::Walk);
            } else {
                character.animation.current_state = AnimationState::Idle;
                character.movement.state = MovementState::Idle;
                character
                    .animation_controller
                    .select_animation(TargetAnimation::Idle);
            }

            // 8. Advance Custom Animation Controller
            // This advances time and blends weights.
            character.animation_controller.update(dt);

            // 9. Evaluate Pose for Renderer
            character.current_pose = character.animation_controller.evaluate_pose();
        }
    }

    fn resolve_voxel_collisions(character: &mut Character, world: &World) {
        let pos = character.transform.position;
        let radius = character.collision.radius;
        let height = character.collision.height;

        // Reset grounded state before checking downward collisions.
        character.movement.is_grounded = false;

        // Define search bounds for voxels that could overlap with the character's AABB.
        let min_x = (pos.x - radius).floor() as i32;
        let max_x = (pos.x + radius).ceil() as i32;
        let min_y = pos.y.floor() as i32;
        let max_y = (pos.y + height).ceil() as i32;
        let min_z = (pos.z - radius).floor() as i32;
        let max_z = (pos.z + radius).ceil() as i32;

        for x in min_x..max_x {
            for y in min_y..max_y {
                for z in min_z..max_z {
                    let coord = WorldCoord::new(x, y, z);

                    if let Some(cell) = world.get(coord) {
                        if !cell.solid {
                            continue;
                        }

                        let v_min = Vec3::new(x as f32, y as f32, z as f32);
                        let v_max = v_min + Vec3::ONE;

                        // AABB overlap test
                        let overlap_x = (character.transform.position.x + radius).min(v_max.x)
                            - (character.transform.position.x - radius).max(v_min.x);
                        let overlap_y = (character.transform.position.y + height).min(v_max.y)
                            - character.transform.position.y.max(v_min.y);
                        let overlap_z = (character.transform.position.z + radius).min(v_max.z)
                            - (character.transform.position.z - radius).max(v_min.z);

                        if overlap_x > 0.001 && overlap_y > 0.001 && overlap_z > 0.001 {
                            // Resolve collision on the axis of shallowest penetration.
                            if overlap_y < overlap_x && overlap_y < overlap_z {
                                if character.transform.position.y < v_min.y {
                                    // Pushing down (head hit)
                                    character.transform.position.y -= overlap_y;
                                    if character.movement.velocity.y > 0.0 {
                                        character.movement.velocity.y = 0.0;
                                    }
                                } else {
                                    // Pushing up (floor hit)
                                    character.transform.position.y += overlap_y;
                                    character.movement.is_grounded = true;
                                    if character.movement.velocity.y < 0.0 {
                                        character.movement.velocity.y = 0.0;
                                    }
                                }
                            } else if overlap_x < overlap_z {
                                if character.transform.position.x < v_min.x {
                                    character.transform.position.x -= overlap_x;
                                } else {
                                    character.transform.position.x += overlap_x;
                                }
                                character.movement.velocity.x = 0.0;
                            } else {
                                if character.transform.position.z < v_min.z {
                                    character.transform.position.z -= overlap_z;
                                } else {
                                    character.transform.position.z += overlap_z;
                                }
                                character.movement.velocity.z = 0.0;
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::movement;
    use crate::world::{CellType, World, WorldCoord};

    #[test]
    fn test_character_runtime_starts_empty() {
        let system = CharacterSystem::new();
        assert!(!system.has_characters());
        assert_eq!(system.get_active_characters().count(), 0);
    }

    #[test]
    fn test_adding_character_stores_it() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();
        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        system.spawn_player(&world);
        assert!(system.has_characters());
        assert_eq!(system.get_active_characters().count(), 1);
    }

    #[test]
    fn test_runtime_ids_remain_unique() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();
        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);
        world.set_cell(WorldCoord::new(5, 0, 5), CellType::SpawnPoint);

        system.spawn_player(&world);
        let id1 = system.get_active_characters().next().unwrap().id;

        system.spawn_player(&world);
        let id2 = system
            .get_active_characters()
            .find(|c| c.id != id1)
            .unwrap()
            .id;

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_clear_removes_all_characters() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();
        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        system.spawn_player(&world);
        assert!(system.has_characters());

        system.clear();
        assert!(!system.has_characters());
        assert_eq!(system.next_id, 1);
    }

    #[test]
    fn test_character_gravity() {
        let mut system = CharacterSystem::new();
        let world = World::new(); // empty air
        let character = Character::new(1, Vec3::new(0.0, 10.0, 0.0));
        system.characters.insert(1, character);

        // Update for 1 second
        for _ in 0..60 {
            system.update(&world, 1.0 / 60.0, Vec2::ZERO, false);
        }

        let updated = system.get_active_characters().next().unwrap();
        assert!(updated.transform.position.y < 10.0);
        assert!(updated.movement.velocity.y < 0.0);
    }

    #[test]
    fn test_character_horizontal_movement() {
        let mut system = CharacterSystem::new();
        let world = World::new();
        let character = Character::new(1, Vec3::ZERO);
        system.characters.insert(1, character);

        // Move Right (+X)
        system.update(&world, 0.1, Vec2::new(1.0, 0.0), false);

        let updated = system.get_active_characters().next().unwrap();
        assert!(updated.transform.position.x > 0.0);
        assert_eq!(updated.animation.current_state, AnimationState::Walk);
        assert_eq!(updated.animation_controller.blend_weight() > 0.0, true);
    }

    #[test]
    fn test_character_ground_collision() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();
        // Create solid floor at Y=0
        world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);

        // Character starts just above the floor
        let character = Character::new(1, Vec3::new(0.0, 1.5, 0.0));
        system.characters.insert(1, character);

        // Update until they hit the ground
        for _ in 0..60 {
            system.update(&world, 1.0 / 60.0, Vec2::ZERO, false);
        }

        let updated = system.get_active_characters().next().unwrap();
        // Should stop at top of voxel (Y=1.0)
        assert!((updated.transform.position.y - 1.0).abs() < 0.01);
        assert!(updated.movement.is_grounded);
    }

    #[test]
    fn test_animation_state_switching() {
        let mut system = CharacterSystem::new();
        let world = World::new();
        let character = Character::new(1, Vec3::ZERO);
        system.characters.insert(1, character);

        // 1. Idle (no input)
        system.update(&world, 0.1, Vec2::ZERO, false);
        assert_eq!(
            system
                .get_active_characters()
                .next()
                .unwrap()
                .animation
                .current_state,
            AnimationState::Idle
        );

        // 2. Walk (input)
        system.update(&world, 0.1, Vec2::X, false);
        assert_eq!(
            system
                .get_active_characters()
                .next()
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
        let character = Character::new(1, Vec3::ZERO);
        system.characters.insert(1, character);

        // Move Right (+X, input.y is 0)
        system.update(&world, 0.1, Vec2::new(1.0, 0.0), false);
        let rot1 = system
            .get_active_characters()
            .next()
            .unwrap()
            .transform
            .rotation;

        // Move Left (-X)
        system.update(&world, 0.1, Vec2::new(-1.0, 0.0), false);
        let rot2 = system
            .get_active_characters()
            .next()
            .unwrap()
            .transform
            .rotation;

        assert_ne!(rot1, rot2);

        // Stationary character keeps rotation
        system.update(&world, 0.1, Vec2::ZERO, false);
        let rot3 = system
            .get_active_characters()
            .next()
            .unwrap()
            .transform
            .rotation;
        assert_eq!(rot2, rot3);
    }

    #[test]
    fn test_character_jumping() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();
        world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);

        let character = Character::new(1, Vec3::new(0.0, 1.0, 0.0));
        system.characters.insert(1, character);

        // Ensure grounded first
        system.update(&world, 1.0 / 60.0, Vec2::ZERO, false);
        assert!(
            system
                .get_active_characters()
                .next()
                .unwrap()
                .movement
                .is_grounded
        );

        // Jump
        system.update(&world, 1.0 / 60.0, Vec2::ZERO, true);
        let updated = system.get_active_characters().next().unwrap();
        // Note: Gravity is applied in the same frame as the impulse, so we expect
        // JUMP_IMPULSE + gravity * dt.
        let expected_v = JUMP_IMPULSE + world.gravity.y * (1.0 / 60.0);
        assert!((updated.movement.velocity.y - expected_v).abs() < 0.001);
        assert!(!updated.movement.is_grounded);

        // Airborne character cannot jump again (until grounded)
        system.update(&world, 1.0 / 60.0, Vec2::ZERO, true);
        // Velocity should have decreased due to gravity, not reset to JUMP_IMPULSE
        assert!(
            system
                .get_active_characters()
                .next()
                .unwrap()
                .movement
                .velocity
                .y
                < movement::JUMP_IMPULSE
        );
    }
}
