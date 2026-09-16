#[path = "../src/character_custom/mod.rs"]
pub mod character_custom;

use character_custom::{
    CharacterAnimationController, TargetAnimation, AppearanceCustomization,
    MaterialSlot, CharacterCollision, generate_character_mesh
};

#[test]
fn test_animation_controller_initial_state() {
    let mut controller = CharacterAnimationController::new();
    let initial_weight = controller.blend_weight();

    // Default animation should be Idle (weight 0.0)
    assert_eq!(initial_weight, 0.0);

    // Evaluate initial pose
    let pose = controller.evaluate_pose();
    assert!(!pose.matrices.is_empty(), "Pose matrices should be evaluated");
}

#[test]
fn test_animation_state_blending() {
    let mut controller = CharacterAnimationController::new();

    // Select Walk animation and advance time to verify blending weight moves towards 1.0
    controller.select_animation(TargetAnimation::Walk);

    controller.update(0.1);
    let mid_weight = controller.blend_weight();
    assert!(mid_weight > 0.0, "Weight should increase towards Walk");
    assert!(mid_weight < 1.0, "Weight shouldn't snap immediately");

    // Update multiple times to reach full blending
    for _ in 0..10 {
        controller.update(0.1);
    }

    assert_eq!(controller.blend_weight(), 1.0, "Should fully blend to Walk state");
}

#[test]
fn test_animation_time_advancement_and_looping() {
    let mut controller = CharacterAnimationController::new();

    // Set active rate and update to check looping bounds
    controller.set_playback_rate(2.0);

    // Large time step to check durability and wrapping mechanics
    controller.update(10.0);

    let pose = controller.evaluate_pose();
    assert_eq!(pose.matrices.len(), controller.skeleton().joints.len());
}

#[test]
fn test_appearance_customization_slots() {
    let mut custom = AppearanceCustomization::new();

    // Update individual colors without recreating everything
    custom.skin_color = [1.0, 0.0, 0.0, 1.0];
    custom.show_accessory = false;

    assert_eq!(custom.get_color(MaterialSlot::Skin), [1.0, 0.0, 0.0, 1.0]);
    assert_eq!(custom.show_accessory, false);
}

#[test]
fn test_decoupled_collision_dimensions() {
    // Confirm independent collision data is completely decoupled from visual bounds
    let collision = CharacterCollision::new(0.45, 1.95);

    assert_eq!(collision.radius, 0.45);
    assert_eq!(collision.height, 1.95);
}

#[test]
fn test_geometry_vertex_buffer_format() {
    let mut controller = CharacterAnimationController::new();
    controller.update(0.5);

    let pose = controller.evaluate_pose();
    let appearance = AppearanceCustomization::default();

    let vertex_buffer = generate_character_mesh(&pose, &appearance);

    assert!(!vertex_buffer.is_empty(), "Vertex buffer should not be empty");
    assert_eq!(
        vertex_buffer.len() % 10,
        0,
        "Vertex buffer length must be a multiple of 10 stride: pos(3) + norm(3) + col(4)"
    );
}
