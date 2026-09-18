use crate::editor::GridPlane;
use crate::renderer::mesh::add_line;

pub const GRID_RADIUS: i32 = 2;

pub fn generate_grid_vertices(plane: GridPlane) -> Vec<f32> {
    let mut vertices = Vec::new();
    let primary_color = [0.45, 0.45, 0.45, 0.75];
    let secondary_color = [0.25, 0.25, 0.25, 0.35];
    let r = GRID_RADIUS as f32;

    add_plane_lines(&mut vertices, plane, 0.0, primary_color, r);
    add_plane_lines(&mut vertices, plane, -1.0, secondary_color, r);

    vertices
}

fn add_plane_lines(
    vertices: &mut Vec<f32>,
    plane: GridPlane,
    offset: f32,
    color: [f32; 4],
    r: f32,
) {
    for i in -GRID_RADIUS..=GRID_RADIUS {
        let p = i as f32;
        match plane {
            GridPlane::Xz => {
                add_line(vertices, [-r, offset, p], [r, offset, p], color);
                add_line(vertices, [p, offset, -r], [p, offset, r], color);
            }
            GridPlane::Yz => {
                add_line(vertices, [offset, -r, p], [offset, r, p], color);
                add_line(vertices, [offset, p, -r], [offset, p, r], color);
            }
            GridPlane::Xy => {
                add_line(vertices, [-r, p, offset], [r, p, offset], color);
                add_line(vertices, [p, -r, offset], [p, r, offset], color);
            }
        }
    }
}

pub fn select_grid_plane(camera_pos: glam::Vec3, target: glam::Vec3) -> GridPlane {
    let dir = (camera_pos - target).normalize();
    let x = dir.x.abs();
    let y = dir.y.abs();
    let z = dir.z.abs();

    if y >= x && y >= z {
        GridPlane::Xz
    } else if x >= z {
        GridPlane::Yz
    } else {
        GridPlane::Xy
    }
}
