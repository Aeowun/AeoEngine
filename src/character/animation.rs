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

use super::{builtin_hero, builtin_robot};
use glam::Mat4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TargetAnimation {
    Idle,
    Walk,
}

#[derive(Clone, Debug)]
pub struct EvaluatedPose {
    pub matrices: Vec<Mat4>,
}

pub enum CharacterAnimationController {
    Robot(builtin_robot::CharacterAnimationController),
    Hero(builtin_hero::CharacterAnimationController),
}

impl CharacterAnimationController {
    pub fn new() -> Self {
        Self::Robot(builtin_robot::CharacterAnimationController::new())
    }

    pub fn new_for_package(package_name: &str) -> Self {
        match package_name {
            "character_hero" => Self::Hero(
                builtin_hero::CharacterAnimationController::new_for_character(None, "hero"),
            ),

            _ => Self::Robot(
                builtin_robot::CharacterAnimationController::new_for_character(None, "robot"),
            ),
        }
    }

    pub fn select_animation(&mut self, animation: TargetAnimation) {
        match self {
            Self::Robot(controller) => {
                controller.select_animation(match animation {
                    TargetAnimation::Idle => builtin_robot::TargetAnimation::Idle,
                    TargetAnimation::Walk => builtin_robot::TargetAnimation::Walk,
                });
            }

            Self::Hero(controller) => {
                controller.select_animation(match animation {
                    TargetAnimation::Idle => builtin_hero::TargetAnimation::Idle,
                    TargetAnimation::Walk => builtin_hero::TargetAnimation::Walk,
                });
            }
        }
    }

    pub fn update(&mut self, dt: f32) {
        match self {
            Self::Robot(controller) => controller.update(dt),
            Self::Hero(controller) => controller.update(dt),
        }
    }

    pub fn evaluate_pose(&mut self) -> EvaluatedPose {
        match self {
            Self::Robot(controller) => {
                let pose = controller.evaluate_pose();
                EvaluatedPose {
                    matrices: pose.matrices.clone(),
                }
            }

            Self::Hero(controller) => {
                let pose = controller.evaluate_pose();
                EvaluatedPose {
                    matrices: pose.matrices.clone(),
                }
            }
        }
    }

    pub fn blend_weight(&self) -> f32 {
        match self {
            Self::Robot(controller) => controller.blend_weight(),
            Self::Hero(controller) => controller.blend_weight(),
        }
    }
}
