use crate::renderer::Renderer;
use crate::renderer::mesh::upload_block_vertices_3d;
use crate::renderer::shader::create_program;
use crate::world::World;
use glam::{Mat4, Vec3};
use std::ffi::CString;

pub struct SkyRenderer {
    pub(crate) program: u32,
    pub(crate) vao: u32,
    pub(crate) vbo: u32,
}

impl SkyRenderer {
    pub fn new() -> Self {
        let program = create_program(SKY_VERTEX_SHADER, SKY_FRAGMENT_SHADER);

        let mut vertices = Vec::new();
        // A unit cube to sample the cubemap texture.

        // Right face (+X)
        add_face(
            &mut vertices,
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, 0.5, -0.5],
        );
        // Left face (-X)
        add_face(
            &mut vertices,
            [-0.5, -0.5, 0.5],
            [-0.5, -0.5, -0.5],
            [-0.5, 0.5, -0.5],
            [-0.5, 0.5, 0.5],
        );
        // Top face (+Y)
        add_face(
            &mut vertices,
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
        );
        // Bottom face (-Y)
        add_face(
            &mut vertices,
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
        );
        // Front face (-Z)
        add_face(
            &mut vertices,
            [-0.5, -0.5, -0.5],
            [0.5, -0.5, -0.5],
            [0.5, 0.5, -0.5],
            [-0.5, 0.5, -0.5],
        );
        // Back face (+Z)
        add_face(
            &mut vertices,
            [0.5, -0.5, 0.5],
            [-0.5, -0.5, 0.5],
            [-0.5, 0.5, 0.5],
            [0.5, 0.5, 0.5],
        );

        let (vao, vbo, _) = upload_block_vertices_3d(&vertices);

        Self { program, vao, vbo }
    }

    pub fn render(
        &self,
        renderer: &Renderer,
        world: &World,
        camera_pos: Vec3,
        view_projection: Mat4,
    ) {
        if !world.sky.enabled {
            return;
        }

        unsafe {
            gl::Disable(gl::CULL_FACE);
            gl::DepthMask(gl::FALSE);

            gl::UseProgram(self.program);

            let vp_name = CString::new("u_view_projection").unwrap();
            let vp_loc = gl::GetUniformLocation(self.program, vp_name.as_ptr());
            gl::UniformMatrix4fv(
                vp_loc,
                1,
                gl::FALSE,
                view_projection.to_cols_array().as_ptr(),
            );

            let model_name = CString::new("u_model").unwrap();
            let model_loc = gl::GetUniformLocation(self.program, model_name.as_ptr());

            // Render a cube centered on camera.
            let model = Mat4::from_translation(camera_pos);
            gl::UniformMatrix4fv(model_loc, 1, gl::FALSE, model.to_cols_array().as_ptr());

            gl::BindVertexArray(self.vao);
            gl::ActiveTexture(gl::TEXTURE0);

            let tex_loc =
                gl::GetUniformLocation(self.program, CString::new("u_cubemap").unwrap().as_ptr());
            gl::Uniform1i(tex_loc, 0);

            let tex = renderer.get_cubemap(&world.sky.texture);
            if tex != 0 {
                gl::BindTexture(gl::TEXTURE_CUBE_MAP, tex);
                // Draw all 36 vertices
                gl::DrawArrays(gl::TRIANGLES, 0, 36);
            }

            gl::DepthMask(gl::TRUE);
            gl::Enable(gl::CULL_FACE);
        }
    }
}

fn add_face(vertices: &mut Vec<f32>, v1: [f32; 3], v2: [f32; 3], v3: [f32; 3], v4: [f32; 3]) {
    let normal = [0.0, 0.0, 0.0];
    let color = [1.0, 1.0, 1.0, 1.0];
    let dummy_uv = [0.0, 0.0];

    // Triangle 1
    vertices.extend_from_slice(&v1);
    vertices.extend_from_slice(&normal);
    vertices.extend_from_slice(&color);
    vertices.extend_from_slice(&dummy_uv);
    vertices.extend_from_slice(&v2);
    vertices.extend_from_slice(&normal);
    vertices.extend_from_slice(&color);
    vertices.extend_from_slice(&dummy_uv);
    vertices.extend_from_slice(&v3);
    vertices.extend_from_slice(&normal);
    vertices.extend_from_slice(&color);
    vertices.extend_from_slice(&dummy_uv);

    // Triangle 2
    vertices.extend_from_slice(&v1);
    vertices.extend_from_slice(&normal);
    vertices.extend_from_slice(&color);
    vertices.extend_from_slice(&dummy_uv);
    vertices.extend_from_slice(&v3);
    vertices.extend_from_slice(&normal);
    vertices.extend_from_slice(&color);
    vertices.extend_from_slice(&dummy_uv);
    vertices.extend_from_slice(&v4);
    vertices.extend_from_slice(&normal);
    vertices.extend_from_slice(&color);
    vertices.extend_from_slice(&dummy_uv);
}

const SKY_VERTEX_SHADER: &str = r#"
#version 330 core
layout (location = 0) in vec3 a_position;

uniform mat4 u_view_projection;
uniform mat4 u_model;

out vec3 v_pos;

void main() {
    v_pos = a_position;
    gl_Position = u_view_projection * u_model * vec4(a_position, 1.0);
}
"#;

const SKY_FRAGMENT_SHADER: &str = r#"
#version 330 core
in vec3 v_pos;
uniform samplerCube u_cubemap;
out vec4 FragColor;

void main() {
    FragColor = texture(u_cubemap, v_pos);
}
"#;
