use glam::Vec3;
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum MovementState {
    Idle,
    Walk,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterMovement {
    pub velocity: Vec3,
    pub is_grounded: bool,
    pub state: MovementState,
}

impl CharacterMovement {
    pub fn new() -> Self {
        Self {
            velocity: Vec3::ZERO,
            is_grounded: false,
            state: MovementState::Idle,
        }
    }
}
