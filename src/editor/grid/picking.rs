use glam::{Vec3};
use crate::renderer::camera::CameraController;
use crate::editor::GridPlane;
use crate::world::WorldCoord;

pub fn update_hover(
    mx: f32,
    my: f32,
    width: f32,
    height: f32,
    camera: &CameraController,
    anchor: WorldCoord
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
                snapped_hit.z.floor() as i32
            ));
        }
    }

    None
}
