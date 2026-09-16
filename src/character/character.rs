use super::animation::CharacterAnimation;
use super::collision::CharacterCollision;
use super::movement::CharacterMovement;
use super::transform::CharacterTransform;
use glam::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub struct Character {
    pub id: u64,
    pub transform: CharacterTransform,
    pub movement: CharacterMovement,
    pub collision: CharacterCollision,
    pub animation: CharacterAnimation,
}

impl Character {
    pub fn new(id: u64, position: Vec3) -> Self {
        Self {
            id,
            transform: CharacterTransform::new(position),
            movement: CharacterMovement::new(),
            collision: CharacterCollision::new(),
            animation: CharacterAnimation::new(),
        }
    }
}
