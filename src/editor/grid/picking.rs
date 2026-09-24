use crate::editor::GridPlane;
use crate::renderer::camera::CameraController;
use crate::world::{CellType, World, WorldCoord};
use glam::Vec3;

const PICK_FOV: f32 = 60.0_f32.to_radians();
const PICK_NEAR: f32 = 0.1;
const PICK_FAR: f32 = 1000.0;
const RAYCAST_MAX_DISTANCE: f32 = 200.0;

pub fn update_hover(
    mx: f32,
    my: f32,
    width: f32,
    height: f32,
    camera: &CameraController,
    anchor: WorldCoord,
) -> Option<WorldCoord> {
    let (near, dir) = screen_ray(mx, my, width, height, camera)?;

    let plane = super::grid::select_grid_plane(camera.get_position(), camera.target);

    let (normal, offset) = match plane {
        GridPlane::Xz => (Vec3::Y, anchor.y as f32),
        GridPlane::Yz => (Vec3::X, anchor.x as f32),
        GridPlane::Xy => (Vec3::Z, anchor.z as f32),
    };

    let denominator = dir.dot(normal);

    if denominator.abs() <= f32::EPSILON {
        return None;
    }

    let distance = (offset - near.dot(normal)) / denominator;

    if distance < 0.0 {
        return None;
    }

    let hit = near + dir * distance;

    // Lock the plane axis to the exact anchor value before flooring.
    // This avoids floating-point drift around integer boundaries.
    let hit = match plane {
        GridPlane::Xz => Vec3::new(hit.x, offset, hit.z),
        GridPlane::Yz => Vec3::new(offset, hit.y, hit.z),
        GridPlane::Xy => Vec3::new(hit.x, hit.y, offset),
    };

    Some(WorldCoord::new(
        hit.x.floor() as i32,
        hit.y.floor() as i32,
        hit.z.floor() as i32,
    ))
}

pub fn raycast_world(
    mx: f32,
    my: f32,
    width: f32,
    height: f32,
    camera: &CameraController,
    world: &World,
    include_markers: bool,
) -> Option<(WorldCoord, Vec3)> {
    let (near, dir) = screen_ray(mx, my, width, height, camera)?;

    // Start DDA traversal at the voxel containing the ray origin.
    let mut map_pos = WorldCoord::new(
        near.x.floor() as i32,
        near.y.floor() as i32,
        near.z.floor() as i32,
    );

    let delta_dist = Vec3::new(
        reciprocal_abs(dir.x),
        reciprocal_abs(dir.y),
        reciprocal_abs(dir.z),
    );

    let step = WorldCoord::new(
        if dir.x < 0.0 { -1 } else { 1 },
        if dir.y < 0.0 { -1 } else { 1 },
        if dir.z < 0.0 { -1 } else { 1 },
    );

    let mut side_dist = Vec3::new(
        initial_side_distance(near.x, map_pos.x, dir.x, delta_dist.x),
        initial_side_distance(near.y, map_pos.y, dir.y, delta_dist.y),
        initial_side_distance(near.z, map_pos.z, dir.z, delta_dist.z),
    );

    let mut distance = 0.0;
    let mut hit_normal = Vec3::ZERO;

    while distance < RAYCAST_MAX_DISTANCE {
        if let Some(cell) = world.get(map_pos) {
            let eligible = match cell.cell_type {
                CellType::Block | CellType::SpawnPoint => true,
                CellType::Light | CellType::AudioEmitter => include_markers,
                _ => false,
            };

            if eligible {
                let selectable = world.is_cell_visible(map_pos) || include_markers;

                if selectable {
                    return Some((map_pos, hit_normal));
                }
            }
        }

        if side_dist.x < side_dist.y && side_dist.x < side_dist.z {
            distance = side_dist.x;
            side_dist.x += delta_dist.x;
            map_pos.x += step.x;
            hit_normal = Vec3::new(-step.x as f32, 0.0, 0.0);
        } else if side_dist.y < side_dist.z {
            distance = side_dist.y;
            side_dist.y += delta_dist.y;
            map_pos.y += step.y;
            hit_normal = Vec3::new(0.0, -step.y as f32, 0.0);
        } else {
            distance = side_dist.z;
            side_dist.z += delta_dist.z;
            map_pos.z += step.z;
            hit_normal = Vec3::new(0.0, 0.0, -step.z as f32);
        }
    }

    None
}

fn screen_ray(
    mx: f32,
    my: f32,
    width: f32,
    height: f32,
    camera: &CameraController,
) -> Option<(Vec3, Vec3)> {
    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    let view = camera.get_view_matrix();

    let projection =
        glam::camera::rh::proj::opengl::perspective(PICK_FOV, width / height, PICK_NEAR, PICK_FAR);

    let normalized_x = (2.0 * mx) / width - 1.0;

    let normalized_y = 1.0 - (2.0 * my) / height;

    let inverse_view_projection = (projection * view).inverse();

    let near = inverse_view_projection.project_point3(Vec3::new(normalized_x, normalized_y, -1.0));

    let far = inverse_view_projection.project_point3(Vec3::new(normalized_x, normalized_y, 1.0));

    let direction = far - near;

    if direction.length_squared() <= f32::EPSILON {
        return None;
    }

    Some((near, direction.normalize()))
}

fn reciprocal_abs(value: f32) -> f32 {
    if value.abs() <= f32::EPSILON {
        f32::INFINITY
    } else {
        1.0 / value.abs()
    }
}

fn initial_side_distance(
    coordinate: f32,
    cell_coordinate: i32,
    direction: f32,
    delta_distance: f32,
) -> f32 {
    if direction < 0.0 {
        (coordinate - cell_coordinate as f32) * delta_distance
    } else {
        (cell_coordinate as f32 + 1.0 - coordinate) * delta_distance
    }
}
