use super::appearance::AppearanceCustomization;
use super::rig::EvaluatedPose;
use glam::Vec3;

/// Generates the built-in Hero character.
///
/// The Hero uses the exact same skeleton and animation system as the other
/// character packages. Only the visual geometry is different.
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
/// Vertex layout:
/// 10 floats per vertex:
/// position(3), normal(3), color(4).
pub fn generate_character_mesh(
    pose: &EvaluatedPose,
    appearance: &AppearanceCustomization,
    _has_gun: bool,
    _mesh_type: &str,
) -> Vec<f32> {
    let mut vertices = Vec::new();

    // ------------------------------------------------------------
    // Hero palette.
    // ------------------------------------------------------------

    const SKIN: [f32; 4] = [0.91, 0.62, 0.42, 1.0];
    const SKIN_LIGHT: [f32; 4] = [1.0, 0.76, 0.56, 1.0];

    const HAIR: [f32; 4] = [0.18, 0.075, 0.025, 1.0];
    const EYE_WHITE: [f32; 4] = [0.95, 0.95, 0.90, 1.0];
    const EYE_DARK: [f32; 4] = [0.025, 0.02, 0.018, 1.0];

    const TUNIC: [f32; 4] = [0.12, 0.42, 0.24, 1.0];
    const TUNIC_LIGHT: [f32; 4] = [0.18, 0.56, 0.32, 1.0];

    const BELT: [f32; 4] = [0.35, 0.18, 0.065, 1.0];
    const BELT_METAL: [f32; 4] = [0.88, 0.68, 0.20, 1.0];

    const PANTS: [f32; 4] = [0.12, 0.16, 0.25, 1.0];

    const BOOT: [f32; 4] = [0.22, 0.095, 0.035, 1.0];
    const BOOT_LIGHT: [f32; 4] = [0.34, 0.15, 0.055, 1.0];

    const CAP: [f32; 4] = [0.16, 0.48, 0.22, 1.0];
    const CAP_LIGHT: [f32; 4] = [0.22, 0.62, 0.30, 1.0];

    // ------------------------------------------------------------
    // Cube helper.
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
                let transformed_normal =
                    mat.transform_vector3(Vec3::from_slice(&normal)).normalize();

                let points = [v1, v2, v3, v4].map(|p| mat.transform_point3(Vec3::from_slice(&p)));

                push_vertex(vertices, points[0], transformed_normal, color);
                push_vertex(vertices, points[1], transformed_normal, color);
                push_vertex(vertices, points[2], transformed_normal, color);

                push_vertex(vertices, points[0], transformed_normal, color);
                push_vertex(vertices, points[2], transformed_normal, color);
                push_vertex(vertices, points[3], transformed_normal, color);
            }
        };

    // ------------------------------------------------------------
    // Low-poly ellipsoid.
    //
    // Used for the rounded/chibi head and nose.
    // Axis = local Y.
    // ------------------------------------------------------------

    let add_ellipsoid = |vertices: &mut Vec<f32>,
                         joint_idx: usize,
                         center: Vec3,
                         radius_x: f32,
                         radius_y: f32,
                         radius_z: f32,
                         segments: usize,
                         rings: usize,
                         color: [f32; 4]| {
        if joint_idx >= pose.matrices.len() || segments < 3 || rings < 2 {
            return;
        }

        let mat = pose.matrices[joint_idx];

        for ring in 0..rings {
            let phi0 =
                -std::f32::consts::FRAC_PI_2 + (ring as f32 / rings as f32) * std::f32::consts::PI;
            let phi1 = -std::f32::consts::FRAC_PI_2
                + ((ring + 1) as f32 / rings as f32) * std::f32::consts::PI;

            let y0 = phi0.sin();
            let y1 = phi1.sin();

            let r0 = phi0.cos();
            let r1 = phi1.cos();

            for segment in 0..segments {
                let a0 = (segment as f32 / segments as f32) * std::f32::consts::TAU;
                let a1 = ((segment + 1) as f32 / segments as f32) * std::f32::consts::TAU;

                let p00 = Vec3::new(
                    center.x + a0.cos() * radius_x * r0,
                    center.y + y0 * radius_y,
                    center.z + a0.sin() * radius_z * r0,
                );

                let p01 = Vec3::new(
                    center.x + a1.cos() * radius_x * r0,
                    center.y + y0 * radius_y,
                    center.z + a1.sin() * radius_z * r0,
                );

                let p10 = Vec3::new(
                    center.x + a0.cos() * radius_x * r1,
                    center.y + y1 * radius_y,
                    center.z + a0.sin() * radius_z * r1,
                );

                let p11 = Vec3::new(
                    center.x + a1.cos() * radius_x * r1,
                    center.y + y1 * radius_y,
                    center.z + a1.sin() * radius_z * r1,
                );

                let n00 = Vec3::new(a0.cos() * r0, y0, a0.sin() * r0).normalize();

                let n01 = Vec3::new(a1.cos() * r0, y0, a1.sin() * r0).normalize();

                let n10 = Vec3::new(a0.cos() * r1, y1, a0.sin() * r1).normalize();

                let n11 = Vec3::new(a1.cos() * r1, y1, a1.sin() * r1).normalize();

                let tp00 = mat.transform_point3(p00);
                let tp01 = mat.transform_point3(p01);
                let tp10 = mat.transform_point3(p10);
                let tp11 = mat.transform_point3(p11);

                let tn00 = mat.transform_vector3(n00).normalize();
                let tn01 = mat.transform_vector3(n01).normalize();
                let tn10 = mat.transform_vector3(n10).normalize();
                let tn11 = mat.transform_vector3(n11).normalize();

                push_vertex(vertices, tp00, tn00, color);
                push_vertex(vertices, tp10, tn10, color);
                push_vertex(vertices, tp11, tn11, color);

                push_vertex(vertices, tp00, tn00, color);
                push_vertex(vertices, tp11, tn11, color);
                push_vertex(vertices, tp01, tn01, color);
            }
        }
    };

    // ------------------------------------------------------------
    // Low-poly cylinder.
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
        for i in 0..segments {
            let next = (i + 1) % segments;

            let b0 = bottom[i];
            let b1 = bottom[next];
            let t1 = top[next];
            let t0 = top[i];

            let normal = Vec3::new(b0.x / radius_x, 0.0, b0.z / radius_z).normalize();

            let transformed_normal = mat.transform_vector3(normal).normalize();

            let p0 = mat.transform_point3(b0);
            let p1 = mat.transform_point3(b1);
            let p2 = mat.transform_point3(t1);
            let p3 = mat.transform_point3(t0);

            push_vertex(vertices, p0, transformed_normal, color);
            push_vertex(vertices, p2, transformed_normal, color);
            push_vertex(vertices, p1, transformed_normal, color);

            push_vertex(vertices, p0, transformed_normal, color);
            push_vertex(vertices, p3, transformed_normal, color);
            push_vertex(vertices, p2, transformed_normal, color);
        }

        // Top cap.
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

        // Bottom cap.
        {
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
    // HERO BODY
    // ============================================================

    // ------------------------------------------------------------
    // 1. HIPS
    // ------------------------------------------------------------

    // Small rounded hip section.
    add_cylinder(&mut vertices, 1, 0.23, 0.17, -0.12, 0.10, 8, PANTS);

    // Belt around the waist.
    add_cylinder(&mut vertices, 1, 0.245, 0.18, 0.00, 0.075, 8, BELT);

    // Belt buckle.
    add_cube(
        &mut vertices,
        1,
        Vec3::new(-0.065, 0.005, 0.175),
        Vec3::new(0.065, 0.075, 0.195),
        BELT_METAL,
    );

    // ------------------------------------------------------------
    // 2. TORSO / TUNIC
    // ------------------------------------------------------------

    // Main tunic.
    add_cube(
        &mut vertices,
        2,
        Vec3::new(-0.285, -0.13, -0.19),
        Vec3::new(0.285, 0.32, 0.19),
        TUNIC,
    );

    // Slightly expanded lower tunic.
    add_cube(
        &mut vertices,
        2,
        Vec3::new(-0.32, -0.20, -0.205),
        Vec3::new(0.32, 0.02, 0.205),
        TUNIC_LIGHT,
    );

    // Chest center panel.
    add_cube(
        &mut vertices,
        2,
        Vec3::new(-0.11, 0.05, 0.192),
        Vec3::new(0.11, 0.26, 0.215),
        TUNIC_LIGHT,
    );

    // Small chest emblem.
    add_cube(
        &mut vertices,
        2,
        Vec3::new(-0.065, 0.11, 0.216),
        Vec3::new(0.065, 0.19, 0.23),
        BELT_METAL,
    );

    // ------------------------------------------------------------
    // 3. LEGS
    // ------------------------------------------------------------

    // Left upper/lower leg.
    add_cylinder(&mut vertices, 4, 0.105, 0.105, -0.72, 0.03, 8, PANTS);

    // Left boot attached to the animated foot joint.
    add_cube(
        &mut vertices,
        5,
        Vec3::new(-0.13, -0.13, -0.10),
        Vec3::new(0.13, 0.08, 0.28),
        BOOT,
    );

    add_cube(
        &mut vertices,
        5,
        Vec3::new(-0.135, -0.08, 0.18),
        Vec3::new(0.135, 0.055, 0.30),
        BOOT_LIGHT,
    );

    // Right leg.
    add_cylinder(&mut vertices, 6, 0.105, 0.105, -0.72, 0.03, 8, PANTS);

    // Right boot.
    add_cube(
        &mut vertices,
        7,
        Vec3::new(-0.13, -0.13, -0.10),
        Vec3::new(0.13, 0.08, 0.28),
        BOOT,
    );

    add_cube(
        &mut vertices,
        7,
        Vec3::new(-0.135, -0.08, 0.18),
        Vec3::new(0.135, 0.055, 0.30),
        BOOT_LIGHT,
    );

    // ------------------------------------------------------------
    // 4. LEFT ARM
    // ------------------------------------------------------------

    add_cylinder(&mut vertices, 8, 0.10, 0.10, -0.38, 0.12, 8, TUNIC);

    // Hand.
    add_ellipsoid(
        &mut vertices,
        8,
        Vec3::new(0.0, -0.44, 0.0),
        0.10,
        0.115,
        0.10,
        8,
        5,
        SKIN,
    );

    // ------------------------------------------------------------
    // 5. RIGHT ARM
    // ------------------------------------------------------------

    add_cylinder(&mut vertices, 9, 0.10, 0.10, -0.38, 0.12, 8, TUNIC);

    // Hand.
    add_ellipsoid(
        &mut vertices,
        9,
        Vec3::new(0.0, -0.44, 0.0),
        0.10,
        0.115,
        0.10,
        8,
        5,
        SKIN,
    );

    // ------------------------------------------------------------
    // 6. HEAD
    // ------------------------------------------------------------

    // Large rounded head gives the character the chibi proportions.
    add_ellipsoid(
        &mut vertices,
        3,
        Vec3::new(0.0, 0.045, 0.0),
        0.255,
        0.285,
        0.245,
        12,
        8,
        SKIN,
    );

    // ------------------------------------------------------------
    // 7. HAIR / CAP
    // ------------------------------------------------------------

    // Back/top hair mass.
    add_ellipsoid(
        &mut vertices,
        3,
        Vec3::new(0.0, 0.17, -0.03),
        0.255,
        0.17,
        0.24,
        10,
        5,
        HAIR,
    );

    // Hero cap.
    add_cylinder(
        &mut vertices,
        3,
        0.265,
        0.255,
        0.18,
        0.29,
        10,
        if appearance.show_accessory { CAP } else { HAIR },
    );

    // Cap highlight.
    add_cylinder(
        &mut vertices,
        3,
        0.20,
        0.19,
        0.285,
        0.32,
        10,
        if appearance.show_accessory {
            CAP_LIGHT
        } else {
            HAIR
        },
    );

    // Cap brim.
    add_cube(
        &mut vertices,
        3,
        Vec3::new(-0.22, 0.16, 0.19),
        Vec3::new(0.22, 0.205, 0.34),
        if appearance.show_accessory {
            CAP_LIGHT
        } else {
            HAIR
        },
    );

    // ------------------------------------------------------------
    // 8. FACE
    // ------------------------------------------------------------

    // Left eye.
    add_ellipsoid(
        &mut vertices,
        3,
        Vec3::new(-0.085, 0.06, 0.228),
        0.052,
        0.065,
        0.028,
        8,
        5,
        EYE_WHITE,
    );

    add_ellipsoid(
        &mut vertices,
        3,
        Vec3::new(-0.085, 0.06, 0.252),
        0.024,
        0.036,
        0.016,
        8,
        4,
        EYE_DARK,
    );

    // Right eye.
    add_ellipsoid(
        &mut vertices,
        3,
        Vec3::new(0.085, 0.06, 0.228),
        0.052,
        0.065,
        0.028,
        8,
        5,
        EYE_WHITE,
    );

    add_ellipsoid(
        &mut vertices,
        3,
        Vec3::new(0.085, 0.06, 0.252),
        0.024,
        0.036,
        0.016,
        8,
        4,
        EYE_DARK,
    );

    // Small nose.
    add_ellipsoid(
        &mut vertices,
        3,
        Vec3::new(0.0, -0.015, 0.255),
        0.055,
        0.065,
        0.065,
        8,
        5,
        SKIN_LIGHT,
    );

    // Small mouth.
    add_cube(
        &mut vertices,
        3,
        Vec3::new(-0.055, -0.115, 0.235),
        Vec3::new(0.055, -0.095, 0.255),
        EYE_DARK,
    );

    // ============================================================
    // 9. OPTIONAL SIMPLE CAPE / BACK PLATE
    // ============================================================

    if appearance.show_accessory {
        add_cube(
            &mut vertices,
            2,
            Vec3::new(-0.29, -0.10, -0.245),
            Vec3::new(0.29, 0.27, -0.205),
            CAP,
        );

        add_cube(
            &mut vertices,
            2,
            Vec3::new(-0.18, -0.14, -0.255),
            Vec3::new(0.18, 0.16, -0.245),
            CAP_LIGHT,
        );
    }

    vertices
}

fn push_vertex(buffer: &mut Vec<f32>, pos: Vec3, normal: Vec3, color: [f32; 4]) {
    buffer.extend_from_slice(&[
        pos.x, pos.y, pos.z, normal.x, normal.y, normal.z, color[0], color[1], color[2], color[3],
    ]);
}
