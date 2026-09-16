use serde::{Serialize, Deserialize};
use super::transform::CharacterTransform;
use super::movement::CharacterMovement;
use super::collision::CharacterCollision;
use super::animation::CharacterAnimation;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Character {
    pub id: u64,
    pub transform: CharacterTransform,
    pub movement: CharacterMovement,
    pub collision: CharacterCollision,
    pub animation: CharacterAnimation,
}

impl Character {
    pub fn new(id: u64, position: glam::Vec3) -> Self {
        Self {
            id,
            transform: CharacterTransform::new(position),
            movement: CharacterMovement::new(),
            collision: CharacterCollision::new(),
            animation: CharacterAnimation::new(),
        }
    }
}
