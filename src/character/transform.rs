use glam::{Vec3, Quat};
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CharacterTransform {
    pub position: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}

impl CharacterTransform {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            rotation: Quat::IDENTITY,
            scale: Vec3::ONE,
        }
    }
}
