use super::animation::CharacterAnimation;
use super::animation::{CharacterAnimationController, EvaluatedPose, TargetAnimation};
use super::collision::CharacterCollision;
use super::movement::CharacterMovement;
use super::transform::CharacterTransform;
use super::{builtin_hero, builtin_robot};
use glam::{Vec2, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum MaterialSlot {
    Skin,
    Armor,
    Cloth,
    Detail,
    Accessory,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AppearanceCustomization {
    pub skin_color: [f32; 4],
    pub armor_color: [f32; 4],
    pub cloth_color: [f32; 4],
    pub detail_color: [f32; 4],
    pub accessory_color: [f32; 4],
    pub show_accessory: bool,
}

impl Default for AppearanceCustomization {
    fn default() -> Self {
        Self {
            skin_color: [0.70, 0.71, 0.70, 1.0],
            armor_color: [0.46, 0.47, 0.45, 1.0],
            cloth_color: [0.10, 0.10, 0.10, 1.0],
            detail_color: [0.12, 0.13, 0.15, 1.0],
            accessory_color: [0.08, 0.30, 0.34, 1.0],
            show_accessory: true,
        }
    }
}

impl AppearanceCustomization {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_color(&self, slot: MaterialSlot) -> [f32; 4] {
        match slot {
            MaterialSlot::Skin => self.skin_color,
            MaterialSlot::Armor => self.armor_color,
            MaterialSlot::Cloth => self.cloth_color,
            MaterialSlot::Detail => self.detail_color,
            MaterialSlot::Accessory => self.accessory_color,
        }
    }

    fn to_robot(&self) -> builtin_robot::AppearanceCustomization {
        builtin_robot::AppearanceCustomization {
            skin_color: self.skin_color,
            armor_color: self.armor_color,
            cloth_color: self.cloth_color,
            detail_color: self.detail_color,
            accessory_color: self.accessory_color,
            show_accessory: self.show_accessory,
        }
    }

    fn to_hero(&self) -> builtin_hero::AppearanceCustomization {
        builtin_hero::AppearanceCustomization {
            skin_color: self.skin_color,
            armor_color: self.armor_color,
            cloth_color: self.cloth_color,
            detail_color: self.detail_color,
            accessory_color: self.accessory_color,
            show_accessory: self.show_accessory,
        }
    }
}

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
    pub current_pose: EvaluatedPose,
}

impl Character {
    pub fn new(id: u64, position: Vec3) -> Self {
        Self::new_from_package(id, position, "character_robot")
    }

    pub fn new_from_package(id: u64, position: Vec3, requested_package: &str) -> Self {
        let package_name = match requested_package {
            "character_hero" => "character_hero",
            _ => "character_robot",
        };

        let mut controller = CharacterAnimationController::new_for_package(package_name);

        let pose = controller.evaluate_pose();

        Self {
            id,
            transform: CharacterTransform::new(position),
            movement: CharacterMovement::new(),
            collision: CharacterCollision::for_package(package_name),
            animation: CharacterAnimation::new(),

            facing: Vec2::new(0.0, 1.0),

            health: 100.0,
            max_health: 100.0,

            animation_override: None,

            package_name: package_name.to_string(),
            mesh_type: package_name
                .strip_prefix("character_")
                .unwrap_or(package_name)
                .to_string(),
            has_gun: false,

            animation_controller: controller,
            appearance: AppearanceCustomization::new(),
            current_pose: pose,
        }
    }
}

pub fn generate_character_mesh(character: &Character) -> Vec<f32> {
    match character.package_name.as_str() {
        "character_hero" => {
            let pose = builtin_hero::EvaluatedPose {
                matrices: character.current_pose.matrices.clone(),
            };

            let appearance = character.appearance.to_hero();

            builtin_hero::generate_character_mesh(
                &pose,
                &appearance,
                character.has_gun,
                &character.mesh_type,
            )
        }

        _ => {
            let pose = builtin_robot::EvaluatedPose {
                matrices: character.current_pose.matrices.clone(),
            };

            let appearance = character.appearance.to_robot();

            builtin_robot::generate_character_mesh(
                &pose,
                &appearance,
                character.has_gun,
                &character.mesh_type,
            )
        }
    }
}
