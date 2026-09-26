//! Renderer ownership and OpenGL resource initialization.
//!
//! Owns the OpenGL programs, buffers, textures, framebuffers, cached chunk
//! meshes, cached editor ghost data, and other GPU resources used by the
//! engine renderer. The renderer reads engine state and produces graphics;
//! it does not own or modify the World or Editor.

pub mod camera;
pub mod character_render;
pub mod chunk;
pub mod chunk_cache;
pub mod editor_render;
pub mod home_render;
pub mod mesh;
pub mod resources;
pub mod shader;
pub mod sky;

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;

use crate::editor::GridPlane;
use crate::engine::EditorMode;
use crate::world::ChunkCoord;

use self::chunk::{ChunkMesh, GhostChunkMesh};
use self::resources::{
    create_anchor_marker, create_axes, create_billboard_vao, create_character_vao_vbo,
    create_grid_plane_vao, create_highlight_box, get_uniform_location, load_texture_from_file,
};
use self::shader::create_program;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct PointLightUniformLocations {
    pub(super) position: i32,
    pub(super) color: i32,
    pub(super) intensity: i32,
    pub(super) range: i32,
}

/// Owns the OpenGL resources required by the engine renderer.
///
/// The renderer reads engine state and produces graphics. It does not own or
/// modify the World or Editor.
pub struct Renderer {
    pub(super) grid_program: u32,
    pub(super) ghost_program: u32,

    pub(super) ghost_view_projection_location: i32,
    pub(super) ghost_model_location: i32,
    pub(super) ghost_color_location: i32,
    pub(super) ghost_alpha_location: i32,

    pub(super) grid_view_projection_location: i32,
    pub(super) grid_model_location: i32,
    pub(super) grid_base_color_location: i32,
    pub(super) grid_alpha_location: i32,
    pub(super) grid_use_texture_location: i32,
    pub(super) grid_ambient_intensity_location: i32,
    pub(super) grid_global_light_enabled_location: i32,
    pub(super) grid_global_light_direction_location: i32,
    pub(super) grid_global_light_color_location: i32,
    pub(super) grid_global_light_intensity_location: i32,
    pub(super) grid_light_space_matrix_location: i32,
    pub(super) grid_shadows_enabled_location: i32,
    pub(super) grid_shadow_map_location: i32,
    pub(super) grid_point_light_count_location: i32,
    pub(super) grid_point_light_uniforms: [PointLightUniformLocations; 16],

    pub(super) shadow_light_space_matrix_location: i32,
    pub(super) shadow_model_location: i32,

    pub(super) grid_vao_xz: u32,
    pub(super) grid_vbo_xz: u32,
    pub(super) grid_count_xz: i32,

    pub(super) grid_vao_yz: u32,
    pub(super) grid_vbo_yz: u32,
    pub(super) grid_count_yz: i32,

    pub(super) grid_vao_xy: u32,
    pub(super) grid_vbo_xy: u32,
    pub(super) grid_count_xy: i32,

    pub(super) axis_vao: u32,
    pub(super) axis_vbo: u32,
    pub(super) axis_vertex_count: i32,

    pub(super) highlight_vao: u32,
    pub(super) highlight_vbo: u32,
    pub(super) highlight_vertex_count: i32,

    pub(super) anchor_vao: u32,
    pub(super) anchor_vbo: u32,
    pub(super) anchor_vertex_count: i32,

    pub(super) block_vao: u32,
    pub(super) block_vbo: u32,
    pub(super) block_mask_ranges: [(i32, i32); 64],

    pub(super) chunk_cache: RefCell<HashMap<ChunkCoord, ChunkMesh>>,
    pub(super) last_render_revision: RefCell<u64>,
    pub(super) last_render_mode: RefCell<Option<EditorMode>>,

    pub(super) ghost_cache: RefCell<HashMap<ChunkCoord, GhostChunkMesh>>,
    pub(super) last_ghost_revision: RefCell<u64>,
    pub(super) last_ghost_mode: RefCell<Option<EditorMode>>,

    pub(super) billboard_vao: u32,
    pub(super) billboard_vbo: u32,

    pub(super) character_vao: u32,
    pub(super) character_vbo: u32,

    pub(super) shadow_program: u32,
    pub(super) shadow_fbo: u32,
    pub(super) shadow_depth_tex: u32,

    pub(super) textures: RefCell<HashMap<String, u32>>,
    pub(super) cubemaps: RefCell<HashMap<String, u32>>,
    pub(super) fallback_tex: u32,

    pub(super) sky_renderer: sky::SkyRenderer,

    pub(super) width: f32,
    pub(super) height: f32,
}

pub(super) const SHADOW_RES: i32 = 2048;
pub(super) const SHADOW_ORTHO_HALF_EXTENT: f32 = 30.0;
pub(super) const SHADOW_NEAR: f32 = 0.1;
pub(super) const SHADOW_FAR: f32 = 100.0;
pub(super) const SHADOW_LIGHT_DISTANCE: f32 = 50.0;

impl Renderer {
    pub fn new(width: f32, height: f32) -> Self {
        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthFunc(gl::LESS);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            gl::LineWidth(1.0);
        }

        let grid_program = create_program(GRID_VERTEX_SHADER, GRID_FRAGMENT_SHADER);

        let ghost_program = create_program(GHOST_VERTEX_SHADER, GHOST_FRAGMENT_SHADER);

        let shadow_program = create_program(SHADOW_VERTEX_SHADER, SHADOW_FRAGMENT_SHADER);

        let grid_view_projection_location = get_uniform_location(grid_program, "u_view_projection");
        let grid_model_location = get_uniform_location(grid_program, "u_model");
        let grid_base_color_location = get_uniform_location(grid_program, "u_base_color");
        let grid_alpha_location = get_uniform_location(grid_program, "u_alpha");
        let grid_use_texture_location = get_uniform_location(grid_program, "u_use_texture");
        let grid_ambient_intensity_location =
            get_uniform_location(grid_program, "u_ambient_intensity");
        let grid_global_light_enabled_location =
            get_uniform_location(grid_program, "u_global_light_enabled");
        let grid_global_light_direction_location =
            get_uniform_location(grid_program, "u_global_light_direction");
        let grid_global_light_color_location =
            get_uniform_location(grid_program, "u_global_light_color");
        let grid_global_light_intensity_location =
            get_uniform_location(grid_program, "u_global_light_intensity");
        let grid_light_space_matrix_location =
            get_uniform_location(grid_program, "u_light_space_matrix");
        let grid_shadows_enabled_location = get_uniform_location(grid_program, "u_shadows_enabled");
        let grid_shadow_map_location = get_uniform_location(grid_program, "u_shadow_map");
        let grid_point_light_count_location =
            get_uniform_location(grid_program, "u_point_light_count");

        let grid_point_light_uniforms = std::array::from_fn(|i| {
            let base = format!("u_point_lights[{i}]");

            PointLightUniformLocations {
                position: get_uniform_location(grid_program, &format!("{base}.position")),
                color: get_uniform_location(grid_program, &format!("{base}.color")),
                intensity: get_uniform_location(grid_program, &format!("{base}.intensity")),
                range: get_uniform_location(grid_program, &format!("{base}.range")),
            }
        });

        let shadow_light_space_matrix_location =
            get_uniform_location(shadow_program, "u_light_space_matrix");
        let shadow_model_location = get_uniform_location(shadow_program, "u_model");

        let ghost_view_projection_location =
            get_uniform_location(ghost_program, "u_view_projection");
        let ghost_model_location = get_uniform_location(ghost_program, "u_model");
        let ghost_color_location = get_uniform_location(ghost_program, "u_color");
        let ghost_alpha_location = get_uniform_location(ghost_program, "u_alpha");

        let (grid_vao_xz, grid_vbo_xz, grid_count_xz) = create_grid_plane_vao(GridPlane::Xz);

        let (grid_vao_yz, grid_vbo_yz, grid_count_yz) = create_grid_plane_vao(GridPlane::Yz);

        let (grid_vao_xy, grid_vbo_xy, grid_count_xy) = create_grid_plane_vao(GridPlane::Xy);

        let (axis_vao, axis_vbo, axis_vertex_count) = create_axes();

        let (highlight_vao, highlight_vbo, highlight_vertex_count) = create_highlight_box();

        let (anchor_vao, anchor_vbo, anchor_vertex_count) = create_anchor_marker();

        let (block_vao, block_vbo, block_mask_ranges) = self::mesh::create_block_masks();

        let (billboard_vao, billboard_vbo) = create_billboard_vao();

        let (character_vao, character_vbo) = create_character_vao_vbo();

        let mut shadow_fbo = 0;
        let mut shadow_depth_tex = 0;

        let mut textures = HashMap::new();
        let fallback_tex;

        unsafe {
            gl::GenFramebuffers(1, &mut shadow_fbo);
            gl::GenTextures(1, &mut shadow_depth_tex);

            gl::BindTexture(gl::TEXTURE_2D, shadow_depth_tex);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::DEPTH_COMPONENT as i32,
                SHADOW_RES,
                SHADOW_RES,
                0,
                gl::DEPTH_COMPONENT,
                gl::FLOAT,
                std::ptr::null(),
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_S,
                gl::CLAMP_TO_BORDER as i32,
            );

            gl::TexParameteri(
                gl::TEXTURE_2D,
                gl::TEXTURE_WRAP_T,
                gl::CLAMP_TO_BORDER as i32,
            );

            let border_color = [1.0, 1.0, 1.0, 1.0];

            gl::TexParameterfv(
                gl::TEXTURE_2D,
                gl::TEXTURE_BORDER_COLOR,
                border_color.as_ptr(),
            );

            gl::BindFramebuffer(gl::FRAMEBUFFER, shadow_fbo);

            gl::FramebufferTexture2D(
                gl::FRAMEBUFFER,
                gl::DEPTH_ATTACHMENT,
                gl::TEXTURE_2D,
                shadow_depth_tex,
                0,
            );

            gl::DrawBuffer(gl::NONE);
            gl::ReadBuffer(gl::NONE);

            let framebuffer_status = gl::CheckFramebufferStatus(gl::FRAMEBUFFER);

            assert_eq!(
                framebuffer_status,
                gl::FRAMEBUFFER_COMPLETE,
                "shadow framebuffer is incomplete: 0x{framebuffer_status:04X}"
            );

            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            // 1x1 white fallback texture.
            let mut fallback = 0;

            gl::GenTextures(1, &mut fallback);
            gl::BindTexture(gl::TEXTURE_2D, fallback);

            let white_data = [255u8, 255, 255, 255];

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                1,
                1,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                white_data.as_ptr() as *const _,
            );

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

            gl::BindTexture(gl::TEXTURE_2D, 0);

            fallback_tex = fallback;

            textures.insert("Block_tx".to_string(), fallback_tex);
        }

        if let Some(tex) = load_texture_from_file(".assets/textures/brick.png") {
            textures.insert("brick".to_string(), tex);
        }

        let renderer = Self {
            grid_program,
            ghost_program,

            ghost_view_projection_location,
            ghost_model_location,
            ghost_color_location,
            ghost_alpha_location,

            grid_view_projection_location,
            grid_model_location,
            grid_base_color_location,
            grid_alpha_location,
            grid_use_texture_location,
            grid_ambient_intensity_location,
            grid_global_light_enabled_location,
            grid_global_light_direction_location,
            grid_global_light_color_location,
            grid_global_light_intensity_location,
            grid_light_space_matrix_location,
            grid_shadows_enabled_location,
            grid_shadow_map_location,
            grid_point_light_count_location,
            grid_point_light_uniforms,

            shadow_light_space_matrix_location,
            shadow_model_location,

            grid_vao_xz,
            grid_vbo_xz,
            grid_count_xz,

            grid_vao_yz,
            grid_vbo_yz,
            grid_count_yz,

            grid_vao_xy,
            grid_vbo_xy,
            grid_count_xy,

            axis_vao,
            axis_vbo,
            axis_vertex_count,

            highlight_vao,
            highlight_vbo,
            highlight_vertex_count,

            anchor_vao,
            anchor_vbo,
            anchor_vertex_count,

            block_vao,
            block_vbo,
            block_mask_ranges,

            chunk_cache: RefCell::new(HashMap::new()),
            last_render_revision: RefCell::new(0),
            last_render_mode: RefCell::new(None),

            ghost_cache: RefCell::new(HashMap::new()),
            last_ghost_revision: RefCell::new(0),
            last_ghost_mode: RefCell::new(None),

            billboard_vao,
            billboard_vbo,

            character_vao,
            character_vbo,

            shadow_program,
            shadow_fbo,
            shadow_depth_tex,

            textures: RefCell::new(textures),
            cubemaps: RefCell::new(HashMap::new()),
            fallback_tex,

            sky_renderer: sky::SkyRenderer::new(),

            width: width.max(1.0),
            height: height.max(1.0),
        };

        unsafe {
            gl::UseProgram(grid_program);

            let tex_name = CString::new("u_texture").unwrap();

            let tex_location = gl::GetUniformLocation(grid_program, tex_name.as_ptr());

            if tex_location != -1 {
                gl::Uniform1i(tex_location, 1);
            }

            gl::UseProgram(0);
        }

        renderer
    }

    /// Creates a renderer with no OpenGL resources for unit tests.
    ///
    /// This constructor must remain free of OpenGL calls so application-state
    /// tests can construct an App without creating a graphics context.
    #[cfg(test)]
    pub(crate) fn new_for_tests(width: f32, height: f32) -> Self {
        Self {
            grid_program: 0,
            ghost_program: 0,

            ghost_view_projection_location: -1,
            ghost_model_location: -1,
            ghost_color_location: -1,
            ghost_alpha_location: -1,

            grid_view_projection_location: -1,
            grid_model_location: -1,
            grid_base_color_location: -1,
            grid_alpha_location: -1,
            grid_use_texture_location: -1,
            grid_ambient_intensity_location: -1,
            grid_global_light_enabled_location: -1,
            grid_global_light_direction_location: -1,
            grid_global_light_color_location: -1,
            grid_global_light_intensity_location: -1,
            grid_light_space_matrix_location: -1,
            grid_shadows_enabled_location: -1,
            grid_shadow_map_location: -1,
            grid_point_light_count_location: -1,

            grid_point_light_uniforms: [PointLightUniformLocations {
                position: -1,
                color: -1,
                intensity: -1,
                range: -1,
            }; 16],

            shadow_light_space_matrix_location: -1,
            shadow_model_location: -1,

            grid_vao_xz: 0,
            grid_vbo_xz: 0,
            grid_count_xz: 0,

            grid_vao_yz: 0,
            grid_vbo_yz: 0,
            grid_count_yz: 0,

            grid_vao_xy: 0,
            grid_vbo_xy: 0,
            grid_count_xy: 0,

            axis_vao: 0,
            axis_vbo: 0,
            axis_vertex_count: 0,

            highlight_vao: 0,
            highlight_vbo: 0,
            highlight_vertex_count: 0,

            anchor_vao: 0,
            anchor_vbo: 0,
            anchor_vertex_count: 0,

            block_vao: 0,
            block_vbo: 0,
            block_mask_ranges: [(0, 0); 64],

            chunk_cache: RefCell::new(HashMap::new()),
            last_render_revision: RefCell::new(0),
            last_render_mode: RefCell::new(None),

            ghost_cache: RefCell::new(HashMap::new()),
            last_ghost_revision: RefCell::new(0),
            last_ghost_mode: RefCell::new(None),

            billboard_vao: 0,
            billboard_vbo: 0,

            character_vao: 0,
            character_vbo: 0,

            shadow_program: 0,
            shadow_fbo: 0,
            shadow_depth_tex: 0,

            textures: RefCell::new(HashMap::new()),
            cubemaps: RefCell::new(HashMap::new()),
            fallback_tex: 0,

            sky_renderer: sky::SkyRenderer {
                program: 0,
                vao: 0,
                vbo: 0,
            },

            width: width.max(1.0),
            height: height.max(1.0),
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width.max(1.0);
        self.height = height.max(1.0);

        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
        }
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }
}

impl Drop for Renderer {
    fn drop(&mut self) {
        unsafe {
            for chunk_mesh in self.chunk_cache.borrow_mut().values_mut() {
                chunk_mesh.free_gl_resources();
            }

            for ghost_mesh in self.ghost_cache.borrow_mut().values_mut() {
                ghost_mesh.free_gl_resources();
            }

            if self.grid_program != 0 {
                gl::DeleteProgram(self.grid_program);
            }

            if self.ghost_program != 0 {
                gl::DeleteProgram(self.ghost_program);
            }

            if self.shadow_program != 0 {
                gl::DeleteProgram(self.shadow_program);
            }

            if self.shadow_fbo != 0 {
                gl::DeleteFramebuffers(1, &self.shadow_fbo);
            }

            if self.shadow_depth_tex != 0 {
                gl::DeleteTextures(1, &self.shadow_depth_tex);
            }

            for &tex in self.textures.borrow().values() {
                if tex != 0 && tex != self.fallback_tex {
                    gl::DeleteTextures(1, &tex);
                }
            }

            if self.fallback_tex != 0 {
                gl::DeleteTextures(1, &self.fallback_tex);
            }

            if self.grid_vao_xz != 0 {
                gl::DeleteVertexArrays(1, &self.grid_vao_xz);
            }

            if self.grid_vbo_xz != 0 {
                gl::DeleteBuffers(1, &self.grid_vbo_xz);
            }

            if self.grid_vao_yz != 0 {
                gl::DeleteVertexArrays(1, &self.grid_vao_yz);
            }

            if self.grid_vbo_yz != 0 {
                gl::DeleteBuffers(1, &self.grid_vbo_yz);
            }

            if self.grid_vao_xy != 0 {
                gl::DeleteVertexArrays(1, &self.grid_vao_xy);
            }

            if self.grid_vbo_xy != 0 {
                gl::DeleteBuffers(1, &self.grid_vbo_xy);
            }

            if self.axis_vao != 0 {
                gl::DeleteVertexArrays(1, &self.axis_vao);
            }

            if self.axis_vbo != 0 {
                gl::DeleteBuffers(1, &self.axis_vbo);
            }

            if self.highlight_vao != 0 {
                gl::DeleteVertexArrays(1, &self.highlight_vao);
            }

            if self.highlight_vbo != 0 {
                gl::DeleteBuffers(1, &self.highlight_vbo);
            }

            if self.anchor_vao != 0 {
                gl::DeleteVertexArrays(1, &self.anchor_vao);
            }

            if self.anchor_vbo != 0 {
                gl::DeleteBuffers(1, &self.anchor_vbo);
            }

            if self.block_vao != 0 {
                gl::DeleteVertexArrays(1, &self.block_vao);
            }

            if self.block_vbo != 0 {
                gl::DeleteBuffers(1, &self.block_vbo);
            }

            if self.billboard_vao != 0 {
                gl::DeleteVertexArrays(1, &self.billboard_vao);
            }

            if self.billboard_vbo != 0 {
                gl::DeleteBuffers(1, &self.billboard_vbo);
            }

            if self.character_vao != 0 {
                gl::DeleteVertexArrays(1, &self.character_vao);
            }

            if self.character_vbo != 0 {
                gl::DeleteBuffers(1, &self.character_vbo);
            }
        }
    }
}

const GHOST_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec3 a_position;
layout (location = 2) in vec4 a_color;

uniform mat4 u_view_projection;
uniform mat4 u_model;

out vec4 v_color;

void main() {
    gl_Position =
        u_view_projection *
        u_model *
        vec4(a_position, 1.0);

    v_color = a_color;
}
"#;

const GHOST_FRAGMENT_SHADER: &str = r#"
#version 330 core

in vec4 v_color;

uniform vec3 u_color;
uniform float u_alpha;

out vec4 FragColor;

void main() {
    FragColor = vec4(v_color.rgb * u_color, u_alpha);
}
"#;

const GRID_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec3 a_position;
layout (location = 1) in vec3 a_normal;
layout (location = 2) in vec4 a_color;
layout (location = 3) in vec2 a_uv;

uniform mat4 u_view_projection;
uniform mat4 u_model;
uniform vec3 u_base_color;

out vec3 v_normal;
out vec4 v_color;
out vec3 v_world_pos;
out vec2 v_uv;

void main() {
    vec4 world_pos =
        u_model *
        vec4(a_position, 1.0);

    v_world_pos =
        world_pos.xyz;

    gl_Position =
        u_view_projection *
        world_pos;

    v_normal =
        mat3(u_model) *
        a_normal;

    v_color =
        vec4(
            a_color.rgb *
            u_base_color,
            a_color.a
        );

    v_uv =
        a_uv;
}
"#;

const GRID_FRAGMENT_SHADER: &str = r#"
#version 330 core

in vec3 v_normal;
in vec4 v_color;
in vec3 v_world_pos;
in vec2 v_uv;

uniform float u_ambient_intensity;

uniform bool u_global_light_enabled;
uniform vec3 u_global_light_direction;
uniform vec3 u_global_light_color;
uniform float u_global_light_intensity;

struct PointLight {
    vec3 position;
    vec3 color;
    float intensity;
    float range;
};

#define MAX_POINT_LIGHTS 16

uniform int u_point_light_count;
uniform PointLight u_point_lights[MAX_POINT_LIGHTS];

uniform sampler2D u_shadow_map;
uniform mat4 u_light_space_matrix;
uniform bool u_shadows_enabled;
uniform float u_alpha;

uniform sampler2D u_texture;
uniform bool u_use_texture;

out vec4 FragColor;

const float SHADOW_MIN_BIAS = 0.0008;
const float SHADOW_SLOPE_BIAS_SCALE = 3.0;
const float SHADOW_NORMAL_OFFSET = 0.005;

float calculate_shadow(
    vec3 world_pos,
    vec3 normal
) {
    vec3 N =
        normalize(normal);

    vec3 L =
        normalize(
            -u_global_light_direction
        );

    float normal_alignment =
        max(
            dot(N, L),
            0.0
        );

    if (normal_alignment <= 0.0) {
        return 0.0;
    }

    ivec2 shadow_texture_size =
        textureSize(
            u_shadow_map,
            0
        );

    vec2 texel_size =
        vec2(1.0) /
        vec2(shadow_texture_size);

    float slope =
        1.0 -
        normal_alignment;

    float bias =
        max(
            SHADOW_MIN_BIAS,
            texel_size.x *
                (
                    1.0 +
                    SHADOW_SLOPE_BIAS_SCALE *
                    slope
                )
        );

    vec3 shadow_world_pos =
        world_pos +
        N *
        SHADOW_NORMAL_OFFSET;

    vec4 light_space_pos =
        u_light_space_matrix *
        vec4(
            shadow_world_pos,
            1.0
        );

    if (abs(light_space_pos.w) <= 0.000001) {
        return 0.0;
    }

    vec3 proj_coords =
        light_space_pos.xyz /
        light_space_pos.w;

    proj_coords =
        proj_coords * 0.5 +
        0.5;

    if (
        proj_coords.z < 0.0 ||
        proj_coords.z > 1.0 ||
        proj_coords.x < 0.0 ||
        proj_coords.x > 1.0 ||
        proj_coords.y < 0.0 ||
        proj_coords.y > 1.0
    ) {
        return 0.0;
    }

    float current_depth =
        proj_coords.z;

    float shadow = 0.0;

    for (
        int x = -1;
        x <= 1;
        ++x
    ) {
        for (
            int y = -1;
            y <= 1;
            ++y
        ) {
            float pcf_depth =
                texture(
                    u_shadow_map,
                    proj_coords.xy
                        + vec2(
                            x,
                            y
                        ) * texel_size
                ).r;

            shadow +=
                current_depth
                    - bias
                    > pcf_depth
                    ? 1.0
                    : 0.0;
        }
    }

    return shadow / 9.0;
}

void main() {
    vec3 normal =
        normalize(v_normal);

    vec4 tex_color =
        vec4(1.0);

    if (u_use_texture) {
        tex_color =
            texture(
                u_texture,
                v_uv
            );
    }

    vec3 color =
        v_color.rgb *
        tex_color.rgb;

    vec3 ambient =
        color *
        u_ambient_intensity;

    vec3 diffuse =
        vec3(0.0);

    if (u_global_light_enabled) {
        vec3 L =
            normalize(
                -u_global_light_direction
            );

        float diff =
            max(
                dot(normal, L),
                0.0
            );

        float shadow =
            u_shadows_enabled
                ? calculate_shadow(
                    v_world_pos,
                    normal
                )
                : 0.0;

        diffuse =
            (1.0 - shadow) *
            color *
            diff *
            u_global_light_color *
            u_global_light_intensity;
    }

    vec3 point_contribution =
        vec3(0.0);

    for (
        int i = 0;
        i < u_point_light_count;
        i++
    ) {
        vec3 L_vec =
            u_point_lights[i].position -
            v_world_pos;

        float dist =
            length(L_vec);

        if (
            dist <
            u_point_lights[i].range
        ) {
            vec3 L =
                normalize(L_vec);

            float diff =
                max(
                    dot(normal, L),
                    0.0
                );

            float attenuation =
                1.0 -
                (
                    dist /
                    u_point_lights[i].range
                );

            attenuation *=
                attenuation;

            point_contribution +=
                color *
                diff *
                u_point_lights[i].color *
                u_point_lights[i].intensity *
                attenuation;
        }
    }

    vec3 result =
        ambient +
        diffuse +
        point_contribution;

    FragColor =
        vec4(
            result,
            v_color.a *
            tex_color.a *
            u_alpha
        );
}
"#;

const SHADOW_VERTEX_SHADER: &str = r#"
#version 330 core

layout (location = 0) in vec3 a_position;

uniform mat4 u_light_space_matrix;
uniform mat4 u_model;

void main() {
    gl_Position =
        u_light_space_matrix *
        u_model *
        vec4(a_position, 1.0);
}
"#;

const SHADOW_FRAGMENT_SHADER: &str = r#"
#version 330 core

void main() {
    // Depth is written automatically.
}
"#;
