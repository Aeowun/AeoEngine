use super::animation::CharacterAnimation;
use super::collision::CharacterCollision;
use super::movement::CharacterMovement;
use super::transform::CharacterTransform;
use crate::character_custom::{
    AppearanceCustomization, CharacterAnimationController,
    CharacterPackageConfig, TargetAnimation,
};
use glam::{Vec2, Vec3};
use std::path::Path;

pub struct Character {
    pub id: u64,
    pub transform: CharacterTransform,
    pub movement: CharacterMovement,
    pub collision: CharacterCollision,
    pub animation: CharacterAnimation,

    /// Current horizontal facing direction.
    pub facing: Vec2,

    /// Runtime combat state.
    pub health: f32,
    pub max_health: f32,

    /// Optional script-selected animation.
    ///
    /// `None` means animation follows movement automatically.
    pub animation_override: Option<TargetAnimation>,

    // --- Custom Character Package Integration ---
    pub package_name: String,
    pub mesh_type: String,
    pub has_gun: bool,

    /// Manages high-level animation state and pose evaluation.
    pub animation_controller: CharacterAnimationController,

    /// Stores the character's visual color settings.
    pub appearance: AppearanceCustomization,

    /// Cached pose for the current frame.
    pub current_pose: crate::character_custom::EvaluatedPose,
}

impl Character {
    pub fn new(id: u64, position: Vec3) -> Self {
        Self::new_from_package(id, position, None)
    }

    pub fn new_from_package(
        id: u64,
        position: Vec3,
        package_dir: Option<&Path>,
    ) -> Self {
        let (config, controller) = if let Some(dir) = package_dir {
            let cfg = CharacterPackageConfig::load_from_dir(dir);
            let ctrl =
                CharacterAnimationController::new_for_character(
                    Some(dir),
                    &cfg.mesh_type,
                );

            (cfg, ctrl)
        } else {
            (
                CharacterPackageConfig {
                    name: "custom".to_string(),
                    mesh_type: "robot".to_string(),
                    has_gun: false,
                    appearance: AppearanceCustomization::new(),
                },
                CharacterAnimationController::new(),
            )
        };

        let mut controller = controller;
        let pose = controller.evaluate_pose();

        Self {
            id,
            transform: CharacterTransform::new(position),
            movement: CharacterMovement::new(),
            collision: CharacterCollision::new(),
            animation: CharacterAnimation::new(),

            facing: Vec2::new(0.0, 1.0),

            health: 100.0,
            max_health: 100.0,

            animation_override: None,

            package_name: config.name,
            mesh_type: config.mesh_type,
            has_gun: config.has_gun,

            animation_controller: controller,
            appearance: config.appearance,
            current_pose: pose,
        }
    }
}