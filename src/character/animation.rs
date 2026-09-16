#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationState {
    Idle,
    Walk,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CharacterAnimation {
    pub current_state: AnimationState,
}

impl Default for CharacterAnimation {
    fn default() -> Self {
        Self {
            current_state: AnimationState::Idle,
        }
    }
}

impl CharacterAnimation {
    pub fn new() -> Self {
        Self::default()
    }
}
