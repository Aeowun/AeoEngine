use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterCollision {
    pub radius: f32,
    pub height: f32,
}

impl CharacterCollision {
    pub fn new() -> Self {
        Self {
            radius: 0.4,
            height: 1.8,
        }
    }
}
