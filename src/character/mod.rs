pub mod animation;
pub mod character;
pub mod collision;
pub mod movement;
pub mod spawning;
pub mod system;
pub mod transform;

pub use animation::{AnimationState, CharacterAnimation};
pub use character::Character;
pub use collision::CharacterCollision;
pub use movement::{CharacterMovement, MovementState};
pub use spawning::spawn_at_random_point;
pub use system::CharacterSystem;
pub use transform::CharacterTransform;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::{CellType, World, WorldCoord};
    use glam::{Quat, Vec3};

    #[test]
    fn test_character_transform_defaults() {
        let t = CharacterTransform::default();
        assert_eq!(t.position, Vec3::ZERO);
        assert_eq!(t.rotation, Quat::IDENTITY);
        assert_eq!(t.scale, Vec3::ONE);
    }

    #[test]
    fn test_character_transform_construction() {
        let pos = Vec3::new(1.0, 2.0, 3.0);
        let t = CharacterTransform::new(pos);
        assert_eq!(t.position, pos);
    }

    #[test]
    fn test_character_movement_defaults() {
        let m = CharacterMovement::default();
        assert_eq!(m.velocity, Vec3::ZERO);
        assert!(!m.is_grounded);
        assert_eq!(m.state, MovementState::Idle);
    }

    #[test]
    fn test_character_collision_defaults() {
        let c = CharacterCollision::default();
        // Standard character capsule dimensions
        assert_eq!(c.radius, 0.4);
        assert_eq!(c.height, 1.8);
    }

    #[test]
    fn test_character_animation_defaults() {
        let a = CharacterAnimation::default();
        assert_eq!(a.current_state, AnimationState::Idle);
    }

    #[test]
    fn test_character_construction() {
        let id = 42;
        let pos = Vec3::new(10.0, 5.0, -2.0);
        let c = Character::new(id, pos);

        assert_eq!(c.id, id);
        assert_eq!(c.transform.position, pos);
        assert_eq!(c.movement.state, MovementState::Idle);
        assert_eq!(c.animation.current_state, AnimationState::Idle);
    }

    #[test]
    fn test_spawn_no_points_returns_none() {
        let world = World::new();
        let result = spawn_at_random_point(&world, 1);
        assert!(result.is_none());
    }

    #[test]
    fn test_spawn_valid_solid_spawn_point() {
        let mut world = World::new();
        let coord = WorldCoord::new(10, 0, 10);
        world.set_cell(coord, CellType::SpawnPoint);
        // Default spawn point is solid.

        let id = 123;
        let character = spawn_at_random_point(&world, id).expect("Should spawn character");

        assert_eq!(character.id, id);
        // Should be at (10, 1, 10) because block is at (10, 0, 10) and is solid.
        assert_eq!(character.transform.position, Vec3::new(10.0, 1.0, 10.0));
        assert_eq!(character.collision, CharacterCollision::default());
    }

    #[test]
    fn test_spawn_non_solid_spawn_point() {
        let mut world = World::new();
        let coord = WorldCoord::new(5, 5, 5);
        world.set_cell(coord, CellType::SpawnPoint);
        if let Some(cell) = world.get_mut(coord) {
            cell.solid = false;
        }

        let character = spawn_at_random_point(&world, 1).expect("Should spawn character");
        // Should be at (5, 5, 5) because block is non-solid.
        assert_eq!(character.transform.position, Vec3::new(5.0, 5.0, 5.0));
    }

    #[test]
    fn test_spawn_blocked_searches_nearby() {
        let mut world = World::new();
        let coord = WorldCoord::new(0, 0, 0);
        world.set_cell(coord, CellType::SpawnPoint);

        // Place a solid block exactly where the character would spawn (0, 1, 0)
        world.set_cell(WorldCoord::new(0, 1, 0), CellType::Block);

        let character = spawn_at_random_point(&world, 1).expect("Should find alternative spawn");
        // Should not be at (0, 1, 0)
        assert_ne!(character.transform.position, Vec3::new(0.0, 1.0, 0.0));
        // It should have found a valid spot within the search radius.
        // (0, 1, 0) is blocked, so it would check radius 1.
        // e.g. (1, 1, 0) or (0, 1, 1) etc.
    }
}
