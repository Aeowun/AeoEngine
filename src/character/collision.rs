#[derive(Clone, Debug, PartialEq)]
pub struct CharacterCollision {
    pub radius: f32,
    pub height: f32,
}

impl Default for CharacterCollision {
    fn default() -> Self {
        Self {
            radius: 0.4,
            height: 1.8,
        }
    }
}

impl CharacterCollision {
    pub fn new() -> Self {
        Self::default()
    }
}
