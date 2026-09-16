pub mod rig;
pub mod animation;
pub mod appearance;
pub mod collision;
pub mod blend;
pub mod geometry;

pub use rig::EvaluatedPose;
pub use appearance::{AppearanceCustomization, MaterialSlot};
pub use collision::CharacterCollision;
pub use blend::{CharacterAnimationController, TargetAnimation};
pub use geometry::generate_character_mesh;
