use super::appearance::{AppearanceCustomization, MaterialSlot};
use super::rig::EvaluatedPose;
use glam::Vec3;

/// Generates a robot character using the AeoEngine rig.
///
/// Existing bone indices:
/// 1 = Hips
/// 2 = Spine
/// 3 = Head
/// 4 = LeftLeg
/// 5 = LeftFoot
/// 6 = RightLeg
/// 7 = RightFoot
/// 8 = LeftArm
/// 9 = RightArm
///
/// Bones 4, 5, 6, and 7 remain part of the rig but are intentionally
/// not rendered for the floating robot. The visual character uses a downward-facing jet
/// mounted to the Hips bone.
///
/// Vertex layout:
/// 10 floats per vertex: pos(3), normal(3), color(4).
pub fn generate_character_mesh(
    pose: &EvaluatedPose,
    appearance: &AppearanceCustomization,
    has_gun: bool,
    mesh_type: &str,
) -> Vec<f32> {
    let mut vertices = Vec::new();

    // ------------------------------------------------------------
    // Dedicated jet colors.
    //
    // These are intentionally NOT tied to MaterialSlot so changing
    // character materials does not unexpectedly change the exhaust.
    // ------------------------------------------------------------
    const JET_OUTER_COLOR: [f32; 4] = [1.0, 0.28, 0.035, 1.0];
    const JET_INNER_COLOR: [f32; 4] = [1.0, 0.88, 0.18, 1.0];

    // ------------------------------------------------------------
    // Cube helper
    // ------------------------------------------------------------
    let add_cube =
        |vertices: &mut Vec<f32>, joint_idx: usize, min: Vec3, max: Vec3, color: [f32; 4]| {
            if joint_idx >= pose.matrices.len() {
                return;
            }

            let mat = pose.matrices[joint_idx];

            let faces = [
                // Front
                (
                    [min.x, min.y, max.z],
                    [max.x, min.y, max.z],
                    [max.x, max.y, max.z],
                    [min.x, max.y, max.z],
                    [0.0, 0.0, 1.0],
                ),
                // Back
                (
                    [max.x, min.y, min.z],
                    [min.x, min.y, min.z],
                    [min.x, max.y, min.z],
                    [max.x, max.y, min.z],
                    [0.0, 0.0, -1.0],
                ),
                // Left
                (
                    [min.x, min.y, min.z],
                    [min.x, min.y, max.z],
                    [min.x, max.y, max.z],
                    [min.x, max.y, min.z],
                    [-1.0, 0.0, 0.0],
                ),
                // Right
                (
                    [max.x, min.y, max.z],
                    [max.x, min.y, min.z],
                    [max.x, max.y, min.z],
                    [max.x, max.y, max.z],
                    [1.0, 0.0, 0.0],
                ),
                // Top
                (
                    [min.x, max.y, max.z],
                    [max.x, max.y, max.z],
                    [max.x, max.y, min.z],
                    [min.x, max.y, min.z],
                    [0.0, 1.0, 0.0],
                ),
                // Bottom
                (
                    [min.x, min.y, min.z],
                    [max.x, min.y, min.z],
                    [max.x, min.y, max.z],
                    [min.x, min.y, max.z],
                    [0.0, -1.0, 0.0],
                ),
            ];

            for (v1, v2, v3, v4, normal) in faces {
                let n = mat.transform_vector3(Vec3::from_slice(&normal)).normalize();

                let points = [v1, v2, v3, v4].map(|p| mat.transform_point3(Vec3::from_slice(&p)));

                push_vertex(vertices, points[0], n, color);
                push_vertex(vertices, points[1], n, color);
                push_vertex(vertices, points[2], n, color);

                push_vertex(vertices, points[0], n, color);
                push_vertex(vertices, points[2], n, color);
                push_vertex(vertices, points[3], n, color);
            }
        };

    // ------------------------------------------------------------
    // Low-poly cylinder helper.
    //
    // Axis = local Y.
    // ------------------------------------------------------------
    let add_cylinder = |vertices: &mut Vec<f32>,
                        joint_idx: usize,
                        radius_x: f32,
                        radius_z: f32,
                        min_y: f32,
                        max_y: f32,
                        segments: usize,
                        color: [f32; 4]| {
        if joint_idx >= pose.matrices.len() || segments < 3 {
            return;
        }

        let mat = pose.matrices[joint_idx];

        let mut bottom = Vec::with_capacity(segments);
        let mut top = Vec::with_capacity(segments);

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;

            let x = angle.cos();
            let z = angle.sin();

            bottom.push(Vec3::new(x * radius_x, min_y, z * radius_z));

            top.push(Vec3::new(x * radius_x, max_y, z * radius_z));
        }

        // Side walls.
        //
        // IMPORTANT:
        // The previous ordering produced inward-facing winding while
        // the normals were outward-facing. With back-face culling enabled,
        // that made portions of the cylinder disappear.
        //
        // These triangles are now wound CCW from the outside.
        for i in 0..segments {
            let next = (i + 1) % segments;

            let b0 = bottom[i];
            let b1 = bottom[next];
            let t1 = top[next];
            let t0 = top[i];

            let local_normal = Vec3::new(b0.x / radius_x, 0.0, b0.z / radius_z).normalize();

            let normal = mat.transform_vector3(local_normal).normalize();

            let p0 = mat.transform_point3(b0);
            let p1 = mat.transform_point3(b1);
            let p2 = mat.transform_point3(t1);
            let p3 = mat.transform_point3(t0);

            // Reversed winding.
            push_vertex(vertices, p0, normal, color);
            push_vertex(vertices, p2, normal, color);
            push_vertex(vertices, p1, normal, color);

            push_vertex(vertices, p0, normal, color);
            push_vertex(vertices, p3, normal, color);
            push_vertex(vertices, p2, normal, color);
        }

        // Top cap.
        {
            let normal = mat.transform_vector3(Vec3::Y).normalize();

            let center = mat.transform_point3(Vec3::new(0.0, max_y, 0.0));

            for i in 0..segments {
                let next = (i + 1) % segments;

                let p1 = mat.transform_point3(top[i]);
                let p2 = mat.transform_point3(top[next]);

                // CCW viewed from above.
                push_vertex(vertices, center, normal, color);
                push_vertex(vertices, p2, normal, color);
                push_vertex(vertices, p1, normal, color);
            }
        }

        // Bottom cap.
        {
            let normal = mat.transform_vector3(-Vec3::Y).normalize();

            let center = mat.transform_point3(Vec3::new(0.0, min_y, 0.0));

            for i in 0..segments {
                let next = (i + 1) % segments;

                let p1 = mat.transform_point3(bottom[i]);
                let p2 = mat.transform_point3(bottom[next]);

                // CCW viewed from below.
                push_vertex(vertices, center, normal, color);
                push_vertex(vertices, p2, normal, color);
                push_vertex(vertices, p1, normal, color);
            }
        }
    };

    // ------------------------------------------------------------
    // Tapered flame helper.
    //
    // Wide at the nozzle and narrow toward the exhaust tip.
    // Axis = local Y.
    //
    // radius_bottom:
    //     radius at the lower/tip end.
    //
    // radius_top:
    //     radius at the nozzle end.
    // ------------------------------------------------------------
    let add_tapered_flame = |vertices: &mut Vec<f32>,
                             joint_idx: usize,
                             radius_bottom: f32,
                             radius_top: f32,
                             min_y: f32,
                             max_y: f32,
                             segments: usize,
                             color: [f32; 4]| {
        if joint_idx >= pose.matrices.len() || segments < 3 {
            return;
        }

        let mat = pose.matrices[joint_idx];

        let mut bottom = Vec::with_capacity(segments);
        let mut top = Vec::with_capacity(segments);

        for i in 0..segments {
            let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;

            let x = angle.cos();
            let z = angle.sin();

            bottom.push(Vec3::new(x * radius_bottom, min_y, z * radius_bottom));

            top.push(Vec3::new(x * radius_top, max_y, z * radius_top));
        }

        let height = (max_y - min_y).max(0.0001);
        let radius_delta = radius_top - radius_bottom;
        let slope = radius_delta / height;

        // Side surface.
        //
        // The flame had the same winding problem as the cylinder.
        // Keep the outward conical normals and reverse the triangle
        // order so the faces survive back-face culling.
        for i in 0..segments {
            let next = (i + 1) % segments;

            let angle = ((i as f32 + 0.5) / segments as f32) * std::f32::consts::TAU;

            let radial = Vec3::new(angle.cos(), 0.0, angle.sin());

            // Outward normal for the conical surface.
            let local_normal = Vec3::new(radial.x, -slope, radial.z).normalize();

            let normal = mat.transform_vector3(local_normal).normalize();

            let p0 = mat.transform_point3(bottom[i]);
            let p1 = mat.transform_point3(bottom[next]);
            let p2 = mat.transform_point3(top[next]);
            let p3 = mat.transform_point3(top[i]);

            // Reversed winding.
            push_vertex(vertices, p0, normal, color);
            push_vertex(vertices, p2, normal, color);
            push_vertex(vertices, p1, normal, color);

            push_vertex(vertices, p0, normal, color);
            push_vertex(vertices, p3, normal, color);
            push_vertex(vertices, p2, normal, color);
        }

        // Top cap where flame meets the nozzle.
        {
            let normal = mat.transform_vector3(Vec3::Y).normalize();

            let center = mat.transform_point3(Vec3::new(0.0, max_y, 0.0));

            for i in 0..segments {
                let next = (i + 1) % segments;

                let p1 = mat.transform_point3(top[i]);
                let p2 = mat.transform_point3(top[next]);

                push_vertex(vertices, center, normal, color);
                push_vertex(vertices, p2, normal, color);
                push_vertex(vertices, p1, normal, color);
            }
        }

        // No bottom cap when radius_bottom is effectively zero.
        if radius_bottom > 0.001 {
            let normal = mat.transform_vector3(-Vec3::Y).normalize();

            let center = mat.transform_point3(Vec3::new(0.0, min_y, 0.0));

            for i in 0..segments {
                let next = (i + 1) % segments;

                let p1 = mat.transform_point3(bottom[i]);
                let p2 = mat.transform_point3(bottom[next]);

                push_vertex(vertices, center, normal, color);
                push_vertex(vertices, p1, normal, color);
                push_vertex(vertices, p2, normal, color);
            }
        }
    };

    // ============================================================
    // ROBOT BODY
    // ============================================================

    // ------------------------------------------------------------
    // 1. Hips / mechanical waist
    // ------------------------------------------------------------
    let hips_color = appearance.get_color(MaterialSlot::Armor);
    let detail_color = appearance.get_color(MaterialSlot::Detail);
    let armor_color = appearance.get_color(MaterialSlot::Armor);
    let accessory_color = appearance.get_color(MaterialSlot::Accessory);

    // Main hip housing.
    add_cylinder(&mut vertices, 1, 0.22, 0.16, -0.12, 0.12, 8, hips_color);

    // Waist ring.
    add_cylinder(&mut vertices, 1, 0.245, 0.18, -0.04, 0.05, 8, detail_color);

    // ------------------------------------------------------------
    // 2. Barrel torso / spine
    // ------------------------------------------------------------
    add_cylinder(&mut vertices, 2, 0.31, 0.21, -0.08, 0.48, 10, armor_color);

    // Lower torso ring.
    add_cylinder(
        &mut vertices,
        2,
        0.315,
        0.215,
        -0.07,
        0.01,
        10,
        detail_color,
    );

    // Chest panel.
    add_cube(
        &mut vertices,
        2,
        Vec3::new(-0.18, 0.08, 0.205),
        Vec3::new(0.18, 0.29, 0.235),
        detail_color,
    );

    // Central chest plate.
    add_cube(
        &mut vertices,
        2,
        Vec3::new(-0.10, 0.13, 0.236),
        Vec3::new(0.10, 0.25, 0.26),
        accessory_color,
    );

    // Chest center strip.
    add_cube(
        &mut vertices,
        2,
        Vec3::new(-0.025, 0.16, 0.261),
        Vec3::new(0.025, 0.24, 0.275),
        detail_color,
    );

    // ------------------------------------------------------------
    // 3. Compact robot head
    // ------------------------------------------------------------
    add_cylinder(&mut vertices, 3, 0.20, 0.17, -0.06, 0.24, 8, armor_color);

    // Side plates.
    add_cube(
        &mut vertices,
        3,
        Vec3::new(-0.215, -0.01, -0.14),
        Vec3::new(-0.19, 0.18, 0.14),
        detail_color,
    );

    add_cube(
        &mut vertices,
        3,
        Vec3::new(0.19, -0.01, -0.14),
        Vec3::new(0.215, 0.18, 0.14),
        detail_color,
    );

    // Visor housing.
    add_cube(
        &mut vertices,
        3,
        Vec3::new(-0.145, 0.045, 0.165),
        Vec3::new(0.145, 0.175, 0.205),
        detail_color,
    );

    // Lower face plate.
    add_cube(
        &mut vertices,
        3,
        Vec3::new(-0.105, -0.04, 0.15),
        Vec3::new(0.105, 0.035, 0.19),
        accessory_color,
    );

    // Antenna stem.
    add_cylinder(
        &mut vertices,
        3,
        0.022,
        0.022,
        0.24,
        0.33,
        8,
        accessory_color,
    );

    // Antenna tip.
    add_cylinder(&mut vertices, 3, 0.042, 0.042, 0.32, 0.35, 8, detail_color);

    // ------------------------------------------------------------
    // 4. LEGS AND FEET
    //
    // Intentionally NOT RENDERED.
    //
    // Bones 4/5/6/7 still exist in the rig and animation system.
    // The visual robot is a hovering/flying machine.
    // ------------------------------------------------------------

    // ------------------------------------------------------------
    // 5. LEFT ARM
    // ------------------------------------------------------------
    add_cylinder(&mut vertices, 8, 0.115, 0.115, -0.08, 0.08, 8, detail_color);

    add_cylinder(&mut vertices, 8, 0.075, 0.075, -0.32, 0.0, 8, armor_color);

    add_cylinder(
        &mut vertices,
        8,
        0.095,
        0.095,
        -0.40,
        -0.30,
        8,
        detail_color,
    );

    add_cylinder(&mut vertices, 8, 0.07, 0.07, -0.58, -0.38, 8, armor_color);

    add_cylinder(
        &mut vertices,
        8,
        0.085,
        0.085,
        -0.66,
        -0.53,
        8,
        detail_color,
    );

    // ------------------------------------------------------------
    // 6. RIGHT ARM
    // ------------------------------------------------------------
    add_cylinder(&mut vertices, 9, 0.115, 0.115, -0.08, 0.08, 8, detail_color);

    add_cylinder(&mut vertices, 9, 0.075, 0.075, -0.32, 0.0, 8, armor_color);

    add_cylinder(
        &mut vertices,
        9,
        0.095,
        0.095,
        -0.40,
        -0.30,
        8,
        detail_color,
    );

    add_cylinder(&mut vertices, 9, 0.07, 0.07, -0.58, -0.38, 8, armor_color);

    add_cylinder(
        &mut vertices,
        9,
        0.085,
        0.085,
        -0.66,
        -0.53,
        8,
        detail_color,
    );

    if has_gun || mesh_type == "soldier" {
        // Left hand glove on Joint 8
        add_cube(
            &mut vertices,
            8,
            Vec3::new(-0.06, -0.72, -0.06),
            Vec3::new(0.06, -0.66, 0.06),
            detail_color,
        );

        // Right hand glove on Joint 9
        add_cube(
            &mut vertices,
            9,
            Vec3::new(-0.06, -0.72, -0.06),
            Vec3::new(0.06, -0.66, 0.06),
            detail_color,
        );

        // Sci-Fi Rifle on Joint 9 (Right Arm/Hand)
        let gun_color = [0.15, 0.18, 0.16, 1.0];
        let barrel_color = [0.08, 0.30, 0.34, 1.0];

        // Rifle receiver/body
        add_cube(
            &mut vertices,
            9,
            Vec3::new(-0.05, -0.75, -0.15),
            Vec3::new(0.05, -0.62, 0.35),
            gun_color,
        );

        // Rifle barrel
        add_cylinder(
            &mut vertices,
            9,
            0.025,
            0.025,
            0.35,
            0.65,
            8,
            barrel_color,
        );

        // Magazine/grip
        add_cube(
            &mut vertices,
            9,
            Vec3::new(-0.03, -0.85, 0.0),
            Vec3::new(0.03, -0.72, 0.12),
            gun_color,
        );
    }

    // ------------------------------------------------------------
    // 7. HIP-MOUNTED JET NOZZLE
    //
    // Attached directly to Hips bone 1.
    // ------------------------------------------------------------

    // Upper mounting collar.
    add_cylinder(&mut vertices, 1, 0.13, 0.13, -0.14, -0.04, 8, detail_color);

    // Main nozzle housing.
    add_cylinder(&mut vertices, 1, 0.105, 0.105, -0.27, -0.13, 8, armor_color);

    // Nozzle rim.
    add_cylinder(
        &mut vertices,
        1,
        0.125,
        0.125,
        -0.31,
        -0.25,
        8,
        detail_color,
    );

    // Dark/secondary exhaust chamber.
    add_cylinder(
        &mut vertices,
        1,
        0.075,
        0.075,
        -0.36,
        -0.30,
        8,
        accessory_color,
    );

    // ------------------------------------------------------------
    // 8. OUTER FLAME
    //
    // Wide where it exits the nozzle and tapered to a point below.
    // ------------------------------------------------------------
    add_tapered_flame(
        &mut vertices,
        1,
        0.015,
        0.085,
        -0.68,
        -0.35,
        10,
        JET_OUTER_COLOR,
    );

    // ------------------------------------------------------------
    // 9. INNER HOT FLAME
    //
    // Smaller, brighter core nested inside the outer flame.
    // ------------------------------------------------------------
    add_tapered_flame(
        &mut vertices,
        1,
        0.001,
        0.055,
        -0.58,
        -0.35,
        10,
        JET_INNER_COLOR,
    );

    // ------------------------------------------------------------
    // 10. OPTIONAL BACK PLATE (From the old humanoid character, thought id leave it here)
    // ------------------------------------------------------------
    //if appearance.show_accessory {
    //    add_cube(
    //        &mut vertices,
    //        2,
    //        Vec3::new(-0.24, -0.02, -0.25),
    //        Vec3::new(0.24, 0.34, -0.205),
    //        accessory_color,
    //    );
    //
    //    add_cube(
    //        &mut vertices,
    //        2,
    //        Vec3::new(-0.05, -0.04, -0.275),
    //        Vec3::new(0.05, 0.32, -0.245),
    //        detail_color,
    //    );
    //}

    vertices
}

fn push_vertex(buffer: &mut Vec<f32>, pos: Vec3, normal: Vec3, color: [f32; 4]) {
    buffer.extend_from_slice(&[
        pos.x, pos.y, pos.z, normal.x, normal.y, normal.z, color[0], color[1], color[2], color[3],
    ]);
}
