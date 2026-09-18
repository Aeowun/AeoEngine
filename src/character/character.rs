use super::animation::CharacterAnimation;
use super::collision::CharacterCollision;
use super::movement::CharacterMovement;
use super::transform::CharacterTransform;
use crate::character_custom::{AppearanceCustomization, CharacterAnimationController};
use glam::Vec3;

pub struct Character {
    pub id: u64,
    pub transform: CharacterTransform,
    pub movement: CharacterMovement,
    pub collision: CharacterCollision,
    pub animation: CharacterAnimation,

    // --- Custom Character Package Integration ---
    /// Manages high-level animation state and pose evaluation.
    pub animation_controller: CharacterAnimationController,
    /// Stores the character's visual color settings.
    pub appearance: AppearanceCustomization,
    /// Cached pose for the current frame.
    pub current_pose: crate::character_custom::EvaluatedPose,
}

impl Character {
    pub fn new(id: u64, position: Vec3) -> Self {
        let mut controller = CharacterAnimationController::new();
        let pose = controller.evaluate_pose();

        Self {
            id,
            transform: CharacterTransform::new(position),
            movement: CharacterMovement::new(),
            collision: CharacterCollision::new(),
            animation: CharacterAnimation::new(),
            animation_controller: controller,
            appearance: AppearanceCustomization::new(),
            current_pose: pose,
        }
    }
}
