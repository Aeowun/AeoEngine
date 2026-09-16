use glam::Vec3;
use super::rig::EvaluatedPose;
use super::appearance::{AppearanceCustomization, MaterialSlot};

/// Generates a flat vertex buffer matching AeoEngine's 3D layout:
/// 10 floats per vertex: pos(3), normal(3), color(4).
pub fn generate_character_mesh(
    pose: &EvaluatedPose,
    appearance: &AppearanceCustomization,
) -> Vec<f32> {
    let mut vertices = Vec::new();

    // Helper to add a stylized cuboid mesh transformed by a bone's global matrix
    let mut add_cube = |joint_idx: usize, min: Vec3, max: Vec3, slot: MaterialSlot| {
        if joint_idx >= pose.matrices.len() {
            return;
        }
        let mat = pose.matrices[joint_idx];
        let color = appearance.get_color(slot);

        // Define 6 faces of a cube
        let faces = [
            // Front (Normal: 0, 0, 1)
            ([min.x, min.y, max.z], [max.x, min.y, max.z], [max.x, max.y, max.z], [min.x, max.y, max.z], [0.0, 0.0, 1.0]),
            // Back (Normal: 0, 0, -1)
            ([max.x, min.y, min.z], [min.x, min.y, min.z], [min.x, max.y, min.z], [max.x, max.y, min.z], [0.0, 0.0, -1.0]),
            // Left (Normal: -1, 0, 0)
            ([min.x, min.y, min.z], [min.x, min.y, max.z], [min.x, max.y, max.z], [min.x, max.y, min.z], [-1.0, 0.0, 0.0]),
            // Right (Normal: 1, 0, 0)
            ([max.x, min.y, max.z], [max.x, min.y, min.z], [max.x, max.y, min.z], [max.x, max.y, max.z], [1.0, 0.0, 0.0]),
            // Top (Normal: 0, 1, 0)
            ([min.x, max.y, max.z], [max.x, max.y, max.z], [max.x, max.y, min.z], [min.x, max.y, min.z], [0.0, 1.0, 0.0]),
            // Bottom (Normal: 0, -1, 0)
            ([min.x, min.y, min.z], [max.x, min.y, min.z], [max.x, min.y, max.z], [min.x, min.y, max.z], [0.0, -1.0, 0.0]),
        ];

        for (v1, v2, v3, v4, norm) in faces {
            let n_transformed = mat.transform_vector3(Vec3::from_slice(&norm)).normalize();
            let pts = [v1, v2, v3, v4].map(|p| mat.transform_point3(Vec3::from_slice(&p)));

            // Triangle 1
            push_vertex(&mut vertices, pts[0], n_transformed, color);
            push_vertex(&mut vertices, pts[1], n_transformed, color);
            push_vertex(&mut vertices, pts[2], n_transformed, color);

            // Triangle 2
            push_vertex(&mut vertices, pts[0], n_transformed, color);
            push_vertex(&mut vertices, pts[2], n_transformed, color);
            push_vertex(&mut vertices, pts[3], n_transformed, color);
        }
    };

    // 1. Hips / Lower Torso (Bone 1)
    add_cube(1, Vec3::new(-0.25, -0.2, -0.15), Vec3::new(0.25, 0.2, 0.15), MaterialSlot::Armor);

    // 2. Spine / Upper Chest (Bone 2)
    add_cube(2, Vec3::new(-0.28, 0.0, -0.18), Vec3::new(0.28, 0.35, 0.18), MaterialSlot::Cloth);
    // Gold Emblem detail on chest
    add_cube(2, Vec3::new(-0.1, 0.1, 0.181), Vec3::new(0.1, 0.25, 0.20), MaterialSlot::Detail);

    // 3. Head (Bone 3)
    add_cube(3, Vec3::new(-0.2, 0.0, -0.2), Vec3::new(0.2, 0.35, 0.2), MaterialSlot::Skin);
    // Helmet top / hair plate slot
    add_cube(3, Vec3::new(-0.21, 0.2, -0.21), Vec3::new(0.21, 0.37, 0.21), MaterialSlot::Armor);
    // Eyes detail (visor/glasses)
    add_cube(3, Vec3::new(-0.15, 0.12, 0.201), Vec3::new(0.15, 0.18, 0.22), MaterialSlot::Detail);

    // 4. Left Leg & Foot (Bones 4 & 5)
    add_cube(4, Vec3::new(-0.12, -0.3, -0.12), Vec3::new(0.12, 0.0, 0.12), MaterialSlot::Armor);
    add_cube(5, Vec3::new(-0.13, -0.15, -0.2), Vec3::new(0.13, 0.0, 0.1), MaterialSlot::Detail);

    // 5. Right Leg & Foot (Bones 6 & 7)
    add_cube(6, Vec3::new(-0.12, -0.3, -0.12), Vec3::new(0.12, 0.0, 0.12), MaterialSlot::Armor);
    add_cube(7, Vec3::new(-0.13, -0.15, -0.2), Vec3::new(0.13, 0.0, 0.1), MaterialSlot::Detail);

    // 6. Left Arm (Bone 8)
    add_cube(8, Vec3::new(-0.1, -0.4, -0.1), Vec3::new(0.1, 0.1, 0.1), MaterialSlot::Cloth);
    add_cube(8, Vec3::new(-0.11, -0.42, -0.11), Vec3::new(0.11, -0.25, 0.11), MaterialSlot::Armor);

    // 7. Right Arm (Bone 9)
    add_cube(9, Vec3::new(-0.1, -0.4, -0.1), Vec3::new(0.1, 0.1, 0.1), MaterialSlot::Cloth);
    add_cube(9, Vec3::new(-0.11, -0.42, -0.11), Vec3::new(0.11, -0.25, 0.11), MaterialSlot::Armor);

    // 8. Secondary Details / Optional Accessory (Emerald Cape/Shield attached to Spine)
    if appearance.show_accessory {
        // High quality back shield / cape plate attached to upper chest/spine
        add_cube(2, Vec3::new(-0.2, -0.2, -0.24), Vec3::new(0.2, 0.25, -0.181), MaterialSlot::Accessory);
    }

    vertices
}

fn push_vertex(buffer: &mut Vec<f32>, pos: Vec3, normal: Vec3, color: [f32; 4]) {
    buffer.extend_from_slice(&[
        pos.x, pos.y, pos.z,
        normal.x, normal.y, normal.z,
        color[0], color[1], color[2], color[3],
    ]);
}
