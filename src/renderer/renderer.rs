use glam::{Mat4, Vec3};
use std::ffi::CString;

use crate::world::{World, CellType};
use crate::editor::{Editor, GridPlane};
use super::shader::create_program;
use super::mesh::{
    upload_vertices_2d,
    upload_vertices_3d,
    add_line,
    add_quad
};

pub struct Renderer {
    home_program: u32,
    home_vao: u32,
    home_vbo: u32,

    grid_program: u32,
    grid_vao_xz: u32,
    grid_vbo_xz: u32,
    grid_count_xz: i32,

    grid_vao_yz: u32,
    grid_vbo_yz: u32,
    grid_count_yz: i32,

    grid_vao_xy: u32,
    grid_vbo_xy: u32,
    grid_count_xy: i32,

    axis_vao: u32,
    axis_vbo: u32,
    axis_vertex_count: i32,

    highlight_vao: u32,
    highlight_vbo: u32,
    highlight_vertex_count: i32,

    anchor_vao: u32,
    anchor_vbo: u32,
    anchor_vertex_count: i32,

    grass_vao: u32,
    grass_vbo: u32,
    grass_vertex_count: i32,

    width: f32,
    height: f32,
}

impl Renderer {
    pub fn new(width: f32, height: f32) -> Self {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(
                gl::SRC_ALPHA,
                gl::ONE_MINUS_SRC_ALPHA,
            );

            gl::LineWidth(1.0);
        }

        let home_program = create_program(HOME_VERTEX_SHADER, HOME_FRAGMENT_SHADER);
        let grid_program = create_program(GRID_VERTEX_SHADER, GRID_FRAGMENT_SHADER);

        let (home_vao, home_vbo) = create_home_triangle();
        let (grid_vao_xz, grid_vbo_xz, grid_count_xz) = create_grid_plane_vao(GridPlane::Xz);
        let (grid_vao_yz, grid_vbo_yz, grid_count_yz) = create_grid_plane_vao(GridPlane::Yz);
        let (grid_vao_xy, grid_vbo_xy, grid_count_xy) = create_grid_plane_vao(GridPlane::Xy);

        let (axis_vao, axis_vbo, axis_vertex_count) = create_axes();
        let (highlight_vao, highlight_vbo, highlight_vertex_count) = create_highlight_box();
        let (anchor_vao, anchor_vbo, anchor_vertex_count) = create_anchor_marker();
        let (grass_vao, grass_vbo, grass_vertex_count) = create_grass_cube();

        Self {
            home_program, home_vao, home_vbo,
            grid_program, grid_vao_xz, grid_vbo_xz, grid_count_xz,
            grid_vao_yz, grid_vbo_yz, grid_count_yz,
            grid_vao_xy, grid_vbo_xy, grid_count_xy,
            axis_vao, axis_vbo, axis_vertex_count,
            highlight_vao, highlight_vbo, highlight_vertex_count,
            anchor_vao, anchor_vbo, anchor_vertex_count,
            grass_vao, grass_vbo, grass_vertex_count,
            width, height,
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width.max(1.0);
        self.height = height.max(1.0);
        unsafe { gl::Viewport(0, 0, self.width as i32, self.height as i32); }
    }

    pub fn width(&self) -> f32 { self.width }
    pub fn height(&self) -> f32 { self.height }

    pub fn render_home(&self) {
        unsafe {
            gl::Disable(gl::DEPTH_TEST);
            gl::ClearColor(0.05, 0.05, 0.05, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            gl::UseProgram(self.home_program);
            gl::BindVertexArray(self.home_vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            gl::BindVertexArray(0);
            gl::Enable(gl::DEPTH_TEST);
        }
    }

    pub fn render_editor(&self, editor: &Editor, world: &World) {
        let camera_pos = editor.camera.get_position();
        let view = editor.camera.get_view_matrix();
        let target = editor.camera.target;

        let projection = glam::camera::rh::proj::opengl::perspective(
            60.0_f32.to_radians(),
            self.width / self.height,
            0.1,
            1000.0,
        );

        let view_projection = projection * view;
        let plane = crate::editor::grid::select_grid_plane(camera_pos, target);

        unsafe {
            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
            gl::UseProgram(self.grid_program);

            let vp_name = CString::new("u_view_projection").unwrap();
            let vp_location = gl::GetUniformLocation(self.grid_program, vp_name.as_ptr());
            gl::UniformMatrix4fv(vp_location, 1, gl::FALSE, view_projection.to_cols_array().as_ptr());

            let model_name = CString::new("u_model").unwrap();
            let model_location = gl::GetUniformLocation(self.grid_program, model_name.as_ptr());

            // --- Render World Blocks ---
            gl::BindVertexArray(self.grass_vao);
            for coord in world.active_blocks() {
                if let Some(cell) = world.get(coord) {
                    if cell.cell_type == CellType::Grass && cell.visible {
                        let model = Mat4::from_translation(Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32));
                        gl::UniformMatrix4fv(model_location, 1, gl::FALSE, model.to_cols_array().as_ptr());
                        gl::DrawArrays(gl::TRIANGLES, 0, self.grass_vertex_count);
                    }
                }
            }

            // --- Render Workarea Grid ---
            if editor.mode == crate::engine::EditorMode::Editor {
                let (_normal, anchor_offset) = match plane {
                    GridPlane::Xz => (Vec3::Y, editor.anchor.y as f32),
                    GridPlane::Yz => (Vec3::X, editor.anchor.x as f32),
                    GridPlane::Xy => (Vec3::Z, editor.anchor.z as f32),
                };

                let center = if let Some(hover) = editor.hovered_cell {
                    Vec3::new(hover.x as f32, hover.y as f32, hover.z as f32)
                } else {
                    // Working area follows camera target, but stays on the plane defined by anchor
                    let t = editor.camera.target;
                    match plane {
                        GridPlane::Xz => Vec3::new(t.x.floor(), anchor_offset, t.z.floor()),
                        GridPlane::Yz => Vec3::new(anchor_offset, t.y.floor(), t.z.floor()),
                        GridPlane::Xy => Vec3::new(t.x.floor(), t.y.floor(), anchor_offset),
                    }
                };

                gl::UniformMatrix4fv(model_location, 1, gl::FALSE, Mat4::from_translation(center).to_cols_array().as_ptr());
                self.bind_grid_vao(plane);
                gl::DrawArrays(gl::LINES, 0, self.get_grid_count(plane));
            }

            // --- Render Anchor Marker ---
            if editor.mode == crate::engine::EditorMode::Editor {
                let anchor_pos = Vec3::new(editor.anchor.x as f32, editor.anchor.y as f32, editor.anchor.z as f32);
                gl::UniformMatrix4fv(
                    model_location,
                    1,
                    gl::FALSE,
                    Mat4::from_translation(anchor_pos).to_cols_array().as_ptr()
                );
                gl::BindVertexArray(self.anchor_vao);
                gl::DrawArrays(gl::LINES, 0, self.anchor_vertex_count);
            }

            // --- Render Hover Highlight ---
            if editor.mode == crate::engine::EditorMode::Editor {
                if let Some(hover) = editor.hovered_cell {
                    gl::UniformMatrix4fv(
                        model_location,
                        1,
                        gl::FALSE,
                        Mat4::from_translation(Vec3::new(hover.x as f32, hover.y as f32, hover.z as f32)).to_cols_array().as_ptr()
                    );
                    gl::BindVertexArray(self.highlight_vao);
                    gl::DrawArrays(gl::LINES, 0, self.highlight_vertex_count);
                }
            }

            // --- Render World Axes (follows anchor) ---
            if editor.mode == crate::engine::EditorMode::Editor {
                let anchor_pos = Vec3::new(editor.anchor.x as f32, editor.anchor.y as f32, editor.anchor.z as f32);
                gl::UniformMatrix4fv(
                    model_location,
                    1,
                    gl::FALSE,
                    Mat4::from_translation(anchor_pos).to_cols_array().as_ptr()
                );
                gl::BindVertexArray(self.axis_vao);
                gl::DrawArrays(gl::LINES, 0, self.axis_vertex_count);
            }

            gl::BindVertexArray(0);

            // Explicitly restore GL state for egui
            gl::Disable(gl::DEPTH_TEST);
            gl::Disable(gl::BLEND);
        }
    }

    fn bind_grid_vao(&self, plane: GridPlane) {
        unsafe {
            match plane {
                GridPlane::Xz => gl::BindVertexArray(self.grid_vao_xz),
                GridPlane::Yz => gl::BindVertexArray(self.grid_vao_yz),
                GridPlane::Xy => gl::BindVertexArray(self.grid_vao_xy),
            }
        }
    }

    fn get_grid_count(&self, plane: GridPlane) -> i32 {
        match plane {
            GridPlane::Xz => self.grid_count_xz,
            GridPlane::Yz => self.grid_count_yz,
            GridPlane::Xy => self.grid_count_xy,
        }
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.home_program);
            gl::DeleteVertexArrays(1, &self.home_vao);
            gl::DeleteBuffers(1, &self.home_vbo);

            gl::DeleteProgram(self.grid_program);
            gl::DeleteVertexArrays(1, &self.grid_vao_xz);
            gl::DeleteBuffers(1, &self.grid_vbo_xz);
            gl::DeleteVertexArrays(1, &self.grid_vao_yz);
            gl::DeleteBuffers(1, &self.grid_vbo_yz);
            gl::DeleteVertexArrays(1, &self.grid_vao_xy);
            gl::DeleteBuffers(1, &self.grid_vbo_xy);

            gl::DeleteVertexArrays(1, &self.axis_vao);
            gl::DeleteBuffers(1, &self.axis_vbo);

            gl::DeleteVertexArrays(1, &self.highlight_vao);
            gl::DeleteBuffers(1, &self.highlight_vbo);

            gl::DeleteVertexArrays(1, &self.anchor_vao);
            gl::DeleteBuffers(1, &self.anchor_vbo);

            gl::DeleteVertexArrays(1, &self.grass_vao);
            gl::DeleteBuffers(1, &self.grass_vbo);
        }
    }
}

fn create_grid_plane_vao(plane: GridPlane) -> (u32, u32, i32) {
    let vertices = crate::editor::grid::generate_grid_vertices(plane);
    upload_vertices_3d(&vertices)
}

fn create_axes() -> (u32, u32, i32) {
    let length = 3.0; // Smaller axes
    let mut vertices = Vec::new();
    add_line(&mut vertices, [0.0, 0.0, 0.0], [ length, 0.0, 0.0], [1.0, 0.2, 0.2, 1.0]);
    add_line(&mut vertices, [0.0, 0.0, 0.0], [0.0,  length, 0.0], [0.2, 1.0, 0.2, 1.0]);
    add_line(&mut vertices, [0.0, 0.0, 0.0], [0.0, 0.0,  length], [0.2, 0.2, 1.0, 1.0]);
    upload_vertices_3d(&vertices)
}

fn create_highlight_box() -> (u32, u32, i32) {
    let mut vertices = Vec::new();
    let color = [1.0, 0.8, 0.1, 0.9];
    let min = 0.0;
    let max = 1.0;
    add_line(&mut vertices, [min, min, min], [max, min, min], color);
    add_line(&mut vertices, [max, min, min], [max, min, max], color);
    add_line(&mut vertices, [max, min, max], [min, min, max], color);
    add_line(&mut vertices, [min, min, max], [min, min, min], color);
    add_line(&mut vertices, [min, max, min], [max, max, min], color);
    add_line(&mut vertices, [max, max, min], [max, max, max], color);
    add_line(&mut vertices, [max, max, max], [min, max, max], color);
    add_line(&mut vertices, [min, max, max], [min, max, min], color);
    add_line(&mut vertices, [min, min, min], [min, max, min], color);
    add_line(&mut vertices, [max, min, min], [max, max, min], color);
    add_line(&mut vertices, [max, min, max], [max, max, max], color);
    add_line(&mut vertices, [min, min, max], [min, max, max], color);
    upload_vertices_3d(&vertices)
}

fn create_anchor_marker() -> (u32, u32, i32) {
    let mut vertices = Vec::new();
    let color = [0.0, 1.0, 1.0, 1.0]; // Cyan
    let radius = 0.1; // Smaller ball
    let center = [0.0, 0.0, 0.0]; // Local origin for ball, translated in render
    let latitudes = 8;
    let longitudes = 16;
    use std::f32::consts::PI;
    for i in 0..=latitudes {
        let lat = PI * (i as f32 / latitudes as f32 - 0.5);
        let y = lat.sin() * radius + center[1];
        let r = lat.cos() * radius;
        for j in 0..longitudes {
            let lon1 = 2.0 * PI * (j as f32 / longitudes as f32);
            let lon2 = 2.0 * PI * ((j + 1) as f32 / longitudes as f32);
            add_line(&mut vertices, [lon1.cos() * r + center[0], y, lon1.sin() * r + center[2]], [lon2.cos() * r + center[0], y, lon2.sin() * r + center[2]], color);
        }
    }
    for i in 0..longitudes {
        let lon = 2.0 * PI * (i as f32 / longitudes as f32);
        let cos_lon = lon.cos();
        let sin_lon = lon.sin();
        for j in 0..latitudes {
            let lat1 = PI * (j as f32 / latitudes as f32 - 0.5);
            let lat2 = PI * ((j + 1) as f32 / latitudes as f32 - 0.5);
            add_line(&mut vertices, [cos_lon * lat1.cos() * radius + center[0], lat1.sin() * radius + center[1], sin_lon * lat1.cos() * radius + center[2]], [cos_lon * lat2.cos() * radius + center[0], lat2.sin() * radius + center[1], sin_lon * lat2.cos() * radius + center[2]], color);
        }
    }
    upload_vertices_3d(&vertices)
}

fn create_grass_cube() -> (u32, u32, i32) {
    let mut vertices = Vec::new();
    let color = [0.2, 0.8, 0.2, 1.0];
    let min = 0.0;
    let max = 1.0;
    add_quad(&mut vertices, [min, min, min], [max, min, min], [max, min, max], [min, min, max], color);
    add_quad(&mut vertices, [min, max, min], [min, max, max], [max, max, max], [max, max, min], color);
    add_quad(&mut vertices, [min, min, max], [max, min, max], [max, max, max], [min, max, max], color);
    add_quad(&mut vertices, [min, min, min], [min, max, min], [max, max, min], [max, min, min], color);
    add_quad(&mut vertices, [min, min, min], [min, min, max], [min, max, max], [min, max, min], color);
    add_quad(&mut vertices, [max, min, min], [max, max, min], [max, max, max], [max, min, max], color);
    upload_vertices_3d(&vertices)
}

fn create_home_triangle() -> (u32, u32) {
    let vertices: [f32; 15] = [
         0.0,  0.65, 1.0, 0.1, 0.1,
        -0.65, -0.65, 0.1, 1.0, 0.1,
         0.65, -0.65, 0.1, 0.3, 1.0,
    ];
    upload_vertices_2d(&vertices)
}

const HOME_VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec2 a_position;
layout (location = 1) in vec3 a_color;
out vec3 v_color;
void main() {
    gl_Position = vec4(a_position, 0.0, 1.0);
    v_color = a_color;
}
"#;

const HOME_FRAGMENT_SHADER: &str = r#"
#version 330 core
in vec3 v_color;
out vec4 FragColor;
void main() {
    FragColor = vec4(v_color, 1.0);
}
"#;

const GRID_VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 a_position;
layout (location = 1) in vec4 a_color;
uniform mat4 u_view_projection;
uniform mat4 u_model;
out vec4 v_color;
void main() {
    gl_Position = u_view_projection * u_model * vec4(a_position, 1.0);
    v_color = a_color;
}
"#;

const GRID_FRAGMENT_SHADER: &str = r#"
#version 330 core
in vec4 v_color;
out vec4 FragColor;
void main() {
    FragColor = v_color;
}
"#;
