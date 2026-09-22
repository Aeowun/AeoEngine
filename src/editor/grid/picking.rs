use crate::editor::GridPlane;
use crate::renderer::camera::CameraController;
use crate::world::WorldCoord;
use glam::Vec3;

pub fn update_hover(
    mx: f32,
    my: f32,
    width: f32,
    height: f32,
    camera: &CameraController,
    anchor: WorldCoord,
) -> Option<WorldCoord> {
    let camera_pos = camera.get_position();
    let view = camera.get_view_matrix();
    let target = camera.target;

    let projection = glam::camera::rh::proj::opengl::perspective(
        60.0_f32.to_radians(),
        width / height,
        0.1,
        1000.0,
    );

    let plane = super::grid::select_grid_plane(camera_pos, target);

    let x = (2.0 * mx) / width - 1.0;
    let y = 1.0 - (2.0 * my) / height;

    let inv_vp = (projection * view).inverse();
    let near = inv_vp.project_point3(Vec3::new(x, y, -1.0));
    let far = inv_vp.project_point3(Vec3::new(x, y, 1.0));
    let dir = (far - near).normalize();

    let (normal, offset) = match plane {
        GridPlane::Xz => (Vec3::Y, anchor.y as f32),
        GridPlane::Yz => (Vec3::X, anchor.x as f32),
        GridPlane::Xy => (Vec3::Z, anchor.z as f32),
    };

    let denom = dir.dot(normal);
    if denom.abs() > 1e-6 {
        let t = (offset - near.dot(normal)) / denom;
        if t >= 0.0 {
            let hit = near + dir * t;

            // Precision Fix: Lock the coordinate on the plane's axis to the exact anchor offset.
            // This prevents floor() from snapping to -1 or +1 due to float precision (e.g. -0.000001).
            let snapped_hit = match plane {
                GridPlane::Xz => Vec3::new(hit.x, offset, hit.z),
                GridPlane::Yz => Vec3::new(offset, hit.y, hit.z),
                GridPlane::Xy => Vec3::new(hit.x, hit.y, offset),
            };

            return Some(WorldCoord::new(
                snapped_hit.x.floor() as i32,
                snapped_hit.y.floor() as i32,
                snapped_hit.z.floor() as i32,
            ));
        }
    }

    None
}

pub fn raycast_world(
    mx: f32,
    my: f32,
    width: f32,
    height: f32,
    camera: &CameraController,
    world: &crate::world::World,
    include_lights: bool,
) -> Option<(WorldCoord, Vec3)> {
    let view = camera.get_view_matrix();

    let projection = glam::camera::rh::proj::opengl::perspective(
        60.0_f32.to_radians(),
        width / height,
        0.1,
        1000.0,
    );

    let x = (2.0 * mx) / width - 1.0;
    let y = 1.0 - (2.0 * my) / height;

    let inv_vp = (projection * view).inverse();
    let near = inv_vp.project_point3(Vec3::new(x, y, -1.0));
    let far = inv_vp.project_point3(Vec3::new(x, y, 1.0));
    let dir = (far - near).normalize();

    // DDA Algorithm for voxel traversal
    let mut map_pos = WorldCoord::new(
        near.x.floor() as i32,
        near.y.floor() as i32,
        near.z.floor() as i32,
    );

    let delta_dist = Vec3::new(
        (1.0 / dir.x).abs(),
        (1.0 / dir.y).abs(),
        (1.0 / dir.z).abs(),
    );

    let step = Vec3::new(
        if dir.x < 0.0 { -1.0 } else { 1.0 },
        if dir.y < 0.0 { -1.0 } else { 1.0 },
        if dir.z < 0.0 { -1.0 } else { 1.0 },
    );

    let mut side_dist = Vec3::new(
        if dir.x < 0.0 {
            (near.x - map_pos.x as f32) * delta_dist.x
        } else {
            (map_pos.x as f32 + 1.0 - near.x) * delta_dist.x
        },
        if dir.y < 0.0 {
            (near.y - map_pos.y as f32) * delta_dist.y
        } else {
            (map_pos.y as f32 + 1.0 - near.y) * delta_dist.y
        },
        if dir.z < 0.0 {
            (near.z - map_pos.z as f32) * delta_dist.z
        } else {
            (map_pos.z as f32 + 1.0 - near.z) * delta_dist.z
        },
    );

    let max_dist = 200.0;
    let mut dist = 0.0;
    let mut hit_normal = Vec3::ZERO;

    while dist < max_dist {
        if let Some(cell) = world.get(map_pos) {
            // Eligible: Block, SpawnPoint
            // Light is only eligible if include_lights is true.
            let eligible = match cell.cell_type {
                crate::world::CellType::Block | crate::world::CellType::SpawnPoint => true,
                crate::world::CellType::Light => include_lights,
                _ => false,
            };

            // Lights are often set to visible=false, so we check eligibility first.
            // If it's a light and we are including lights, we hit it regardless of visibility flag.
            if eligible
                && (world.is_cell_visible(map_pos)
                    || cell.cell_type == crate::world::CellType::Light)
            {
                return Some((map_pos, hit_normal));
            }
        }

        if side_dist.x < side_dist.y && side_dist.x < side_dist.z {
            dist = side_dist.x;
            side_dist.x += delta_dist.x;
            map_pos.x += step.x as i32;
            hit_normal = Vec3::new(-step.x, 0.0, 0.0);
        } else if side_dist.y < side_dist.z {
            dist = side_dist.y;
            side_dist.y += delta_dist.y;
            map_pos.y += step.y as i32;
            hit_normal = Vec3::new(0.0, -step.y, 0.0);
        } else {
            dist = side_dist.z;
            side_dist.z += delta_dist.z;
            map_pos.z += step.z as i32;
            hit_normal = Vec3::new(0.0, 0.0, -step.z);
        }
    }

    None
}
