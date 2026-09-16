use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimationState {
    Idle,
    Walk,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterAnimation {
    pub current_state: AnimationState,
}

impl CharacterAnimation {
    pub fn new() -> Self {
        Self {
            current_state: AnimationState::Idle,
        }
    }
}
