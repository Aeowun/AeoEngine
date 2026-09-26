/// Decoupled gameplay collision dimensions for the character.
/// These are independent of visual proportions or scale customizations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterCollision {
    pub radius: f32,
    pub height: f32,
}

impl Default for CharacterCollision {
    fn default() -> Self {
        Self {
            radius: 0.35,
            height: 1.8,
        }
    }
}

impl CharacterCollision {
    pub fn new(radius: f32, height: f32) -> Self {
        Self { radius, height }
    }
}
