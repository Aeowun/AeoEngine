pub mod camera;
pub mod mesh;
pub mod shader;
pub mod sky;

use glam::{Mat4, Vec3};
use image::GenericImageView;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::CString;
use std::path::Path;

use crate::editor::{Editor, GridPlane};
use crate::engine::EditorMode;
use crate::engine::physics::PhysicsWorld;
use crate::world::{CellType, World, WorldCoord};

use self::mesh::{add_block_quad, add_line, upload_vertices_3d};
use self::shader::create_program;

/// Owns the OpenGL resources required by the engine renderer.
///
/// The renderer reads engine state and produces graphics. It does not own or
/// modify the World or Editor.
pub struct Renderer {
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

    block_vao: u32,
    block_vbo: u32,
    block_mask_ranges: [(i32, i32); 64],

    billboard_vao: u32,
    billboard_vbo: u32,

    shadow_program: u32,
    shadow_fbo: u32,
    shadow_depth_tex: u32,

    textures: RefCell<HashMap<String, u32>>,
    cubemaps: RefCell<HashMap<String, u32>>,
    fallback_tex: u32,

    sky_renderer: sky::SkyRenderer,

    width: f32,
    height: f32,
}

const SHADOW_RES: i32 = 2048;
const SHADOW_ORTHO_HALF_EXTENT: f32 = 30.0;
const SHADOW_NEAR: f32 = 0.1;
const SHADOW_FAR: f32 = 100.0;
const SHADOW_LIGHT_DISTANCE: f32 = 50.0;

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

        let (grid_vao_xz, grid_vbo_xz, grid_count_xz) =
            create_grid_plane_vao(GridPlane::Xz);
        let (grid_vao_yz, grid_vbo_yz, grid_count_yz) =
            create_grid_plane_vao(GridPlane::Yz);
        let (grid_vao_xy, grid_vbo_xy, grid_count_xy) =
            create_grid_plane_vao(GridPlane::Xy);

        let (axis_vao, axis_vbo, axis_vertex_count) = create_axes();
        let (highlight_vao, highlight_vbo, highlight_vertex_count) = create_highlight_box();
        let (anchor_vao, anchor_vbo, anchor_vertex_count) = create_anchor_marker();
        let (block_vao, block_vbo, block_mask_ranges) = self::mesh::create_block_masks();
        let (billboard_vao, billboard_vbo) = create_billboard_vao();

        let shadow_program = create_program(SHADOW_VERTEX_SHADER, SHADOW_FRAGMENT_SHADER);

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

            // Default blocks use the white fallback texture.
            textures.insert("Block_tx".to_string(), fallback_tex);
        }

        if let Some(tex) = load_texture_from_file(".assets/textures/brick.png") {
            textures.insert("brick".to_string(), tex);
        }

        let renderer = Self {
            grid_program,

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

            billboard_vao,
            billboard_vbo,

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

    pub fn get_block_mask_range(&self, mask: u8) -> (i32, i32) {
        self.block_mask_ranges[mask as usize]
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.width = width.max(1.0);
        self.height = height.max(1.0);

        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
        }
    }

    pub fn get_texture(&self, identifier: &str) -> u32 {
        let mut textures = self.textures.borrow_mut();

        if let Some(&tex) = textures.get(identifier) {
            return tex;
        }

        if !identifier.is_empty() {
            let paths = [
                format!(".assets/textures/{identifier}"),
                format!(".assets/textures/{identifier}.png"),
                format!(".assets/skybox/{identifier}"),
                format!(".assets/skybox/{identifier}.png"),
            ];

            for path in paths {
                if Path::new(&path).exists() {
                    if let Some(tex) = load_texture_from_file(&path) {
                        textures.insert(identifier.to_string(), tex);
                        return tex;
                    }
                }
            }
        }

        self.fallback_tex
    }

    pub fn get_cubemap(&self, identifier: &str) -> u32 {
        let mut cubemaps = self.cubemaps.borrow_mut();

        if let Some(&tex) = cubemaps.get(identifier) {
            return tex;
        }

        if !identifier.is_empty() {
            let paths = [
                format!(".assets/skybox/{identifier}"),
                format!(".assets/skybox/{identifier}.png"),
            ];

            for path in paths {
                if Path::new(&path).exists() {
                    if let Some(tex) = load_cubemap_from_file(&path) {
                        cubemaps.insert(identifier.to_string(), tex);
                        return tex;
                    }
                }
            }
        }

        0
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn render_home(&self) {
        unsafe {
            gl::Viewport(0, 0, self.width as i32, self.height as i32);
            gl::Disable(gl::SCISSOR_TEST);
            gl::Disable(gl::DEPTH_TEST);
            gl::DepthMask(gl::TRUE);

            gl::ClearColor(1.0 / 255.0, 1.0 / 255.0, 2.0 / 255.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);
        }
    }

    pub fn render_editor(
        &self,
        editor: &Editor,
        world: &World,
        physics: &PhysicsWorld,
        character_system: &crate::character::CharacterSystem,
        gameplay_camera: &crate::renderer::camera::GameplayCamera,
        drag_start: Option<WorldCoord>,
    ) {
        let (camera_pos, view, target) =
            if editor.mode == EditorMode::Play && character_system.has_characters() {
                (
                    gameplay_camera.current_position,
                    gameplay_camera.get_view_matrix(),
                    gameplay_camera.current_target,
                )
            } else {
                (
                    editor.camera.get_position(),
                    editor.camera.get_view_matrix(),
                    editor.camera.target,
                )
            };

        let active_blocks = world.active_effective_blocks();

        let ppp = editor.viewport_ppp.max(1.0);
        let rect = editor.viewport_rect;

        let (vp_x, vp_y, vp_w, vp_h, aspect_ratio) = if rect.width() > 1.0 && rect.height() > 1.0
        {
            let x = (rect.min.x * ppp) as i32;
            let w = (rect.width() * ppp) as i32;
            let h = (rect.height() * ppp) as i32;
            let y = self.height as i32 - (rect.max.y * ppp) as i32;

            (x, y, w, h, w as f32 / h.max(1) as f32)
        } else {
            (
                0,
                0,
                self.width as i32,
                self.height as i32,
                self.width / self.height.max(1.0),
            )
        };

        let projection = glam::camera::rh::proj::opengl::perspective(
            60.0_f32.to_radians(),
            aspect_ratio,
            0.1,
            1000.0,
        );

        let view_projection = projection * view;

        let mut light_direction = world.lighting.global_light_direction;

        if light_direction.length_squared() > 0.000001 {
            light_direction = light_direction.normalize();
        } else {
            light_direction = Vec3::new(-0.5, -1.0, -0.5).normalize();
        }

        // The shadow map follows the center of the current camera segment rather
        // than the editor orbit target alone. This prevents the light frustum
        // from being centered on a stale target while the camera moves close to
        // or far from the scene.
        let view_segment = target - camera_pos;
        let shadow_center = if view_segment.length_squared() > 0.000001 {
            camera_pos + view_segment * 0.5
        } else {
            target
        };

        let light_pos = shadow_center - light_direction * SHADOW_LIGHT_DISTANCE;
        let light_view = Mat4::look_at_rh(light_pos, shadow_center, Vec3::Y);

        let light_proj = glam::camera::rh::proj::opengl::orthographic(
            -SHADOW_ORTHO_HALF_EXTENT,
            SHADOW_ORTHO_HALF_EXTENT,
            -SHADOW_ORTHO_HALF_EXTENT,
            SHADOW_ORTHO_HALF_EXTENT,
            SHADOW_NEAR,
            SHADOW_FAR,
        );

        let light_space_matrix = light_proj * light_view;

        unsafe {
            gl::Enable(gl::DEPTH_TEST);
            gl::DepthMask(gl::TRUE);
            gl::DepthFunc(gl::LESS);

            gl::Enable(gl::CULL_FACE);
            gl::CullFace(gl::BACK);
            gl::FrontFace(gl::CCW);

            gl::Disable(gl::BLEND);

            // Shadow pass.
            gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_fbo);
            gl::Viewport(0, 0, SHADOW_RES, SHADOW_RES);
            gl::Clear(gl::DEPTH_BUFFER_BIT);

            gl::UseProgram(self.shadow_program);

            let s_lsm_name = CString::new("u_light_space_matrix").unwrap();
            let s_lsm_location = gl::GetUniformLocation(self.shadow_program, s_lsm_name.as_ptr());

            gl::UniformMatrix4fv(
                s_lsm_location,
                1,
                gl::FALSE,
                light_space_matrix.to_cols_array().as_ptr(),
            );

            let s_model_name = CString::new("u_model").unwrap();
            let s_model_location = gl::GetUniformLocation(self.shadow_program, s_model_name.as_ptr());

            gl::BindVertexArray(self.block_vao);

            for coord in &active_blocks {
                let Some(cell) = world.get_effective_cell(*coord) else {
                    continue;
                };

                if !matches!(cell.cell_type, CellType::Block | CellType::SpawnPoint)
                    || !world.is_cell_visible(*coord)
                    || !world.is_cell_solid(*coord)
                {
                    continue;
                }

                let should_draw = match editor.mode {
                    EditorMode::Editor => true,
                    EditorMode::Play => world.is_cell_anchored(*coord),
                };

                if !should_draw {
                    continue;
                }

                let mask = self::mesh::compute_exposed_faces_shadow(world, *coord, editor.mode);
                if mask == 0 {
                    continue;
                }

                let (first_vertex, count) = self.get_block_mask_range(mask);

                let model = Mat4::from_translation(
                    Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32)
                        + world.get_visual_offset(*coord),
                );

                gl::UniformMatrix4fv(
                    s_model_location,
                    1,
                    gl::FALSE,
                    model.to_cols_array().as_ptr(),
                );

                gl::DrawArrays(gl::TRIANGLES, first_vertex, count);
            }

            if editor.mode == EditorMode::Play {
                let (first_vertex, count) = self.get_block_mask_range(63);

                for body in &physics.bodies {
                    if !body.solid {
                        continue;
                    }

                    let model = Mat4::from_translation(body.position);

                    gl::UniformMatrix4fv(
                        s_model_location,
                        1,
                        gl::FALSE,
                        model.to_cols_array().as_ptr(),
                    );

                    gl::DrawArrays(gl::TRIANGLES, first_vertex, count);
                }
            }

            // Main rendering pass.
            gl::BindFramebuffer(gl::FRAMEBUFFER, 0);

            gl::Viewport(vp_x, vp_y, vp_w, vp_h);
            gl::Enable(gl::SCISSOR_TEST);
            gl::Scissor(vp_x, vp_y, vp_w, vp_h);

            gl::ClearColor(0.0, 0.0, 0.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT | gl::DEPTH_BUFFER_BIT);

            self.sky_renderer
                .render(self, world, camera_pos, view_projection);

            gl::UseProgram(self.grid_program);

            let vp_name = CString::new("u_view_projection").unwrap();
            let vp_location = gl::GetUniformLocation(self.grid_program, vp_name.as_ptr());

            gl::UniformMatrix4fv(
                vp_location,
                1,
                gl::FALSE,
                view_projection.to_cols_array().as_ptr(),
            );

            let model_name = CString::new("u_model").unwrap();
            let model_location = gl::GetUniformLocation(self.grid_program, model_name.as_ptr());

            let base_color_name = CString::new("u_base_color").unwrap();
            let base_color_location =
                gl::GetUniformLocation(self.grid_program, base_color_name.as_ptr());

            gl::Uniform3f(base_color_location, 1.0, 1.0, 1.0);

            let alpha_name = CString::new("u_alpha").unwrap();
            let alpha_location = gl::GetUniformLocation(self.grid_program, alpha_name.as_ptr());

            let use_tex_name = CString::new("u_use_texture").unwrap();
            let use_tex_location = gl::GetUniformLocation(self.grid_program, use_tex_name.as_ptr());

            // Global lighting.
            let ambient_name = CString::new("u_ambient_intensity").unwrap();
            let ambient_location =
                gl::GetUniformLocation(self.grid_program, ambient_name.as_ptr());
            gl::Uniform1f(ambient_location, world.lighting.ambient_intensity);

            let global_enabled_name = CString::new("u_global_light_enabled").unwrap();
            let global_enabled_location =
                gl::GetUniformLocation(self.grid_program, global_enabled_name.as_ptr());
            gl::Uniform1i(
                global_enabled_location,
                if world.lighting.global_light_enabled { 1 } else { 0 },
            );

            let global_dir_name = CString::new("u_global_light_direction").unwrap();
            let global_dir_location =
                gl::GetUniformLocation(self.grid_program, global_dir_name.as_ptr());
            gl::Uniform3f(
                global_dir_location,
                world.lighting.global_light_direction.x,
                world.lighting.global_light_direction.y,
                world.lighting.global_light_direction.z,
            );

            let global_color_name = CString::new("u_global_light_color").unwrap();
            let global_color_location =
                gl::GetUniformLocation(self.grid_program, global_color_name.as_ptr());
            gl::Uniform3f(
                global_color_location,
                world.lighting.global_light_color.x,
                world.lighting.global_light_color.y,
                world.lighting.global_light_color.z,
            );

            let global_intensity_name = CString::new("u_global_light_intensity").unwrap();
            let global_intensity_location =
                gl::GetUniformLocation(self.grid_program, global_intensity_name.as_ptr());
            gl::Uniform1f(
                global_intensity_location,
                world.lighting.global_light_intensity,
            );

            // Shadow uniforms.
            let lsm_name = CString::new("u_light_space_matrix").unwrap();
            let lsm_location = gl::GetUniformLocation(self.grid_program, lsm_name.as_ptr());

            gl::UniformMatrix4fv(
                lsm_location,
                1,
                gl::FALSE,
                light_space_matrix.to_cols_array().as_ptr(),
            );

            let shadows_enabled_name = CString::new("u_shadows_enabled").unwrap();
            let shadows_enabled_location =
                gl::GetUniformLocation(self.grid_program, shadows_enabled_name.as_ptr());

            gl::Uniform1i(
                shadows_enabled_location,
                if world.lighting.shadows_enabled { 1 } else { 0 },
            );

            gl::ActiveTexture(gl::TEXTURE0);
            gl::BindTexture(gl::TEXTURE_2D, self.shadow_depth_tex);

            let shadow_map_name = CString::new("u_shadow_map").unwrap();
            let shadow_map_location =
                gl::GetUniformLocation(self.grid_program, shadow_map_name.as_ptr());
            gl::Uniform1i(shadow_map_location, 0);

            gl::ActiveTexture(gl::TEXTURE1);
            gl::BindTexture(gl::TEXTURE_2D, self.fallback_tex);

            // Point lights.
            let mut point_lights = Vec::new();

            for coord in &active_blocks {
                let Some(cell) = world.get_effective_cell(*coord) else {
                    continue;
                };

                if cell.cell_type == CellType::Light && world.is_light_enabled(*coord) {
                    point_lights.push((*coord, cell));

                    if point_lights.len() >= 16 {
                        break;
                    }
                }
            }

            let count_name = CString::new("u_point_light_count").unwrap();
            let count_location = gl::GetUniformLocation(self.grid_program, count_name.as_ptr());
            gl::Uniform1i(count_location, point_lights.len() as i32);

            for (i, (coord, cell)) in point_lights.iter().enumerate() {
                let base = format!("u_point_lights[{i}]");

                let position_name = CString::new(format!("{base}.position")).unwrap();
                let position_location =
                    gl::GetUniformLocation(self.grid_program, position_name.as_ptr());
                gl::Uniform3f(
                    position_location,
                    coord.x as f32,
                    coord.y as f32,
                    coord.z as f32,
                );

                let color_name = CString::new(format!("{base}.color")).unwrap();
                let color_location =
                    gl::GetUniformLocation(self.grid_program, color_name.as_ptr());
                gl::Uniform3f(
                    color_location,
                    cell.light_color.x,
                    cell.light_color.y,
                    cell.light_color.z,
                );

                let intensity_name = CString::new(format!("{base}.intensity")).unwrap();
                let intensity_location =
                    gl::GetUniformLocation(self.grid_program, intensity_name.as_ptr());
                gl::Uniform1f(intensity_location, cell.light_intensity);

                let range_name = CString::new(format!("{base}.range")).unwrap();
                let range_location =
                    gl::GetUniformLocation(self.grid_program, range_name.as_ptr());
                gl::Uniform1f(range_location, cell.light_range);
            }

            // Authored world cells.
            let cam_right = Vec3::new(view.col(0).x, view.col(1).x, view.col(2).x);
            let cam_up = Vec3::new(view.col(0).y, view.col(1).y, view.col(2).y);

            for coord in &active_blocks {
                let Some(cell) = world.get_effective_cell(*coord) else {
                    continue;
                };

                // Light and audio cells use editor-only billboard markers.
                if editor.mode == EditorMode::Editor
                    && matches!(cell.cell_type, CellType::Light | CellType::AudioEmitter)
                {
                    gl::BindVertexArray(self.billboard_vao);
                    gl::Uniform1i(use_tex_location, 1);

                    let tex_name = match cell.cell_type {
                        CellType::Light => "lightbulb",
                        CellType::AudioEmitter => "speaker",
                        _ => unreachable!(),
                    };

                    let tex = self.get_texture(tex_name);
                    gl::ActiveTexture(gl::TEXTURE1);
                    gl::BindTexture(gl::TEXTURE_2D, tex);

                    let center = Vec3::new(
                        coord.x as f32 + 0.5,
                        coord.y as f32 + 0.5,
                        coord.z as f32 + 0.5,
                    );

                    let scale = 0.35;

                    let model = Mat4::from_cols(
                        cam_right.extend(0.0) * scale,
                        cam_up.extend(0.0) * scale,
                        Vec3::ZERO.extend(0.0),
                        center.extend(1.0),
                    );

                    gl::UniformMatrix4fv(
                        model_location,
                        1,
                        gl::FALSE,
                        model.to_cols_array().as_ptr(),
                    );

                    gl::Uniform3f(base_color_location, 1.0, 1.0, 1.0);
                    gl::DrawArrays(gl::TRIANGLES, 0, 6);
                }

                let renderable = matches!(
                    cell.cell_type,
                    CellType::Block | CellType::SpawnPoint | CellType::Light
                );

                if !renderable || !world.is_cell_visible(*coord) {
                    continue;
                }

                let should_draw = match editor.mode {
                    EditorMode::Editor => true,
                    EditorMode::Play => world.is_cell_anchored(*coord),
                };

                if !should_draw {
                    continue;
                }

                let mask = self::mesh::compute_exposed_faces_main(world, *coord, editor.mode);
                if mask == 0 {
                    continue;
                }

                let (first_vertex, count) = self.get_block_mask_range(mask);

                gl::BindVertexArray(self.block_vao);
                gl::Uniform1i(use_tex_location, 1);

                let tex = self.get_texture(&cell.texture);
                gl::ActiveTexture(gl::TEXTURE1);
                gl::BindTexture(gl::TEXTURE_2D, tex);

                let color = world.get_effective_color(*coord);
                gl::Uniform3f(base_color_location, color.x, color.y, color.z);
                gl::Uniform1f(alpha_location, 1.0);

                let model = Mat4::from_translation(
                    Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32)
                        + world.get_visual_offset(*coord),
                );

                gl::UniformMatrix4fv(
                    model_location,
                    1,
                    gl::FALSE,
                    model.to_cols_array().as_ptr(),
                );

                gl::DrawArrays(gl::TRIANGLES, first_vertex, count);
            }

            gl::Uniform1i(use_tex_location, 0);

            // Runtime physics bodies and characters.
            if editor.mode == EditorMode::Play {
                let (first_vertex, count) = self.get_block_mask_range(63);

                for body in &physics.bodies {
                    if !body.visible {
                        continue;
                    }

                    gl::Uniform3f(
                        base_color_location,
                        body.color_rgb.x,
                        body.color_rgb.y,
                        body.color_rgb.z,
                    );

                    let model = Mat4::from_translation(body.position);

                    gl::UniformMatrix4fv(
                        model_location,
                        1,
                        gl::FALSE,
                        model.to_cols_array().as_ptr(),
                    );

                    gl::DrawArrays(gl::TRIANGLES, first_vertex, count);
                }

                for character in character_system.get_active_characters() {
                    let vertices = crate::character_custom::generate_character_mesh(
                        &character.current_pose,
                        &character.appearance,
                        character.has_gun,
                        &character.mesh_type,
                    );

                    let model = Mat4::from_scale_rotation_translation(
                        character.transform.scale,
                        character.transform.rotation,
                        character.transform.position,
                    );

                    gl::UniformMatrix4fv(
                        model_location,
                        1,
                        gl::FALSE,
                        model.to_cols_array().as_ptr(),
                    );

                    gl::Uniform3f(base_color_location, 1.0, 1.0, 1.0);
                    self::mesh::upload_and_draw_mesh_3d(&vertices);
                }
            }

            // Editor helpers.
            let plane = crate::editor::grid::select_grid_plane(camera_pos, target);

            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

            if editor.mode == EditorMode::Editor && editor.plane_picking {
                let (_normal, anchor_offset) = match plane {
                    GridPlane::Xz => (Vec3::Y, editor.anchor.y as f32),
                    GridPlane::Yz => (Vec3::X, editor.anchor.x as f32),
                    GridPlane::Xy => (Vec3::Z, editor.anchor.z as f32),
                };

                let center = if let Some(hover) = editor.hovered_cell {
                    Vec3::new(hover.x as f32, hover.y as f32, hover.z as f32)
                } else {
                    match plane {
                        GridPlane::Xz => {
                            Vec3::new(target.x.floor(), anchor_offset, target.z.floor())
                        }
                        GridPlane::Yz => {
                            Vec3::new(anchor_offset, target.y.floor(), target.z.floor())
                        }
                        GridPlane::Xy => {
                            Vec3::new(target.x.floor(), target.y.floor(), anchor_offset)
                        }
                    }
                };

                let model = Mat4::from_translation(center);
                gl::UniformMatrix4fv(
                    model_location,
                    1,
                    gl::FALSE,
                    model.to_cols_array().as_ptr(),
                );

                self.bind_grid_vao(plane);
                gl::DrawArrays(gl::LINES, 0, self.get_grid_count(plane));
            }

            if editor.mode == EditorMode::Editor {
                let anchor_pos = Vec3::new(
                    editor.anchor.x as f32,
                    editor.anchor.y as f32,
                    editor.anchor.z as f32,
                );

                let model = Mat4::from_translation(anchor_pos);
                gl::UniformMatrix4fv(
                    model_location,
                    1,
                    gl::FALSE,
                    model.to_cols_array().as_ptr(),
                );

                gl::BindVertexArray(self.anchor_vao);
                gl::DrawArrays(gl::LINES, 0, self.anchor_vertex_count);

                if let Some(hover) = editor.hovered_cell {
                    let model = Mat4::from_translation(Vec3::new(
                        hover.x as f32,
                        hover.y as f32,
                        hover.z as f32,
                    ));

                    gl::UniformMatrix4fv(
                        model_location,
                        1,
                        gl::FALSE,
                        model.to_cols_array().as_ptr(),
                    );

                    gl::BindVertexArray(self.highlight_vao);
                    gl::DrawArrays(gl::LINES, 0, self.highlight_vertex_count);
                }

                gl::BindVertexArray(self.highlight_vao);
                gl::Uniform3f(base_color_location, 0.2, 0.6, 1.0);

                for coord in &editor.selected_coords {
                    let model = Mat4::from_translation(Vec3::new(
                        coord.x as f32,
                        coord.y as f32,
                        coord.z as f32,
                    ));

                    gl::UniformMatrix4fv(
                        model_location,
                        1,
                        gl::FALSE,
                        model.to_cols_array().as_ptr(),
                    );

                    gl::DrawArrays(gl::LINES, 0, self.highlight_vertex_count);
                }

                let model = Mat4::from_translation(anchor_pos);
                gl::UniformMatrix4fv(
                    model_location,
                    1,
                    gl::FALSE,
                    model.to_cols_array().as_ptr(),
                );

                gl::BindVertexArray(self.axis_vao);
                gl::DrawArrays(gl::LINES, 0, self.axis_vertex_count);
            }

            // Ghost rendering.
            if editor.mode == EditorMode::Editor {
                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                gl::DepthMask(gl::FALSE);
                gl::Uniform1f(alpha_location, 0.4);
                gl::Uniform1i(use_tex_location, 0);

                gl::BindVertexArray(self.block_vao);

                for coord in &active_blocks {
                    let Some(cell) = world.get_effective_cell(*coord) else {
                        continue;
                    };

                    if !matches!(cell.cell_type, CellType::Block | CellType::SpawnPoint) {
                        continue;
                    }

                    if world.is_cell_visible(*coord) {
                        continue;
                    }

                    let mask = self::mesh::compute_exposed_faces_main(world, *coord, editor.mode);
                    if mask == 0 {
                        continue;
                    }

                    let (first_vertex, count) = self.get_block_mask_range(mask);
                    let color = world.get_effective_color(*coord);

                    gl::Uniform3f(base_color_location, color.x, color.y, color.z);

                    let model = Mat4::from_translation(
                        Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32)
                            + world.get_visual_offset(*coord),
                    );

                    gl::UniformMatrix4fv(
                        model_location,
                        1,
                        gl::FALSE,
                        model.to_cols_array().as_ptr(),
                    );

                    gl::DrawArrays(gl::TRIANGLES, first_vertex, count);
                }

                gl::DepthMask(gl::TRUE);
                gl::Uniform1f(alpha_location, 1.0);

                // Existing build/erase preview.
                if let (Some(start), Some(end)) = (drag_start, editor.hovered_cell) {
                    let x_min = start.x.min(end.x);
                    let x_max = start.x.max(end.x);

                    let y_min = start.y.min(end.y);
                    let y_max = start.y.max(end.y);

                    let z_min = start.z.min(end.z);
                    let z_max = start.z.max(end.z);

                    gl::Enable(gl::BLEND);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
                    gl::DepthMask(gl::FALSE);
                    gl::Uniform1f(alpha_location, 0.4);

                    match editor.current_tool {
                        crate::editor::EditorTool::Build => {
                            let is_marker = matches!(
                                editor.build_template.cell_type,
                                CellType::Light | CellType::AudioEmitter
                            );

                            if is_marker {
                                gl::BindVertexArray(self.highlight_vao);

                                let (r, g, b) =
                                    if editor.build_template.cell_type == CellType::AudioEmitter {
                                        (0.2, 0.8, 1.0)
                                    } else {
                                        (1.0, 1.0, 0.2)
                                    };

                                gl::Uniform3f(base_color_location, r, g, b);

                                for x in x_min..=x_max {
                                    for y in y_min..=y_max {
                                        for z in z_min..=z_max {
                                            let model = Mat4::from_translation(Vec3::new(
                                                x as f32,
                                                y as f32,
                                                z as f32,
                                            ));

                                            gl::UniformMatrix4fv(
                                                model_location,
                                                1,
                                                gl::FALSE,
                                                model.to_cols_array().as_ptr(),
                                            );

                                            gl::DrawArrays(
                                                gl::LINES,
                                                0,
                                                self.highlight_vertex_count,
                                            );
                                        }
                                    }
                                }
                            } else {
                                gl::BindVertexArray(self.block_vao);

                                let (first_vertex, count) = self.get_block_mask_range(63);
                                let color = editor.build_template.color_rgb;

                                gl::Uniform3f(base_color_location, color.x, color.y, color.z);
                                gl::Uniform1i(use_tex_location, 0);

                                for x in x_min..=x_max {
                                    for y in y_min..=y_max {
                                        for z in z_min..=z_max {
                                            let model = Mat4::from_translation(Vec3::new(
                                                x as f32,
                                                y as f32,
                                                z as f32,
                                            ));

                                            gl::UniformMatrix4fv(
                                                model_location,
                                                1,
                                                gl::FALSE,
                                                model.to_cols_array().as_ptr(),
                                            );

                                            gl::DrawArrays(
                                                gl::TRIANGLES,
                                                first_vertex,
                                                count,
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        crate::editor::EditorTool::Erase => {
                            gl::BindVertexArray(self.highlight_vao);
                            gl::Uniform3f(base_color_location, 1.0, 0.2, 0.2);

                            for x in x_min..=x_max {
                                for y in y_min..=y_max {
                                    for z in z_min..=z_max {
                                        let model = Mat4::from_translation(Vec3::new(
                                            x as f32,
                                            y as f32,
                                            z as f32,
                                        ));

                                        gl::UniformMatrix4fv(
                                            model_location,
                                            1,
                                            gl::FALSE,
                                            model.to_cols_array().as_ptr(),
                                        );

                                        gl::DrawArrays(
                                            gl::LINES,
                                            0,
                                            self.highlight_vertex_count,
                                        );
                                    }
                                }
                            }
                        }

                        _ => {}
                    }

                    gl::DepthMask(gl::TRUE);
                    gl::Uniform1f(alpha_location, 1.0);
                }
            }

            gl::Uniform1i(use_tex_location, 0);
            gl::Uniform1f(alpha_location, 1.0);
            gl::DepthMask(gl::TRUE);
            gl::Enable(gl::DEPTH_TEST);
            gl::Enable(gl::CULL_FACE);
            gl::BindVertexArray(0);
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
            gl::DeleteProgram(self.grid_program);
            gl::DeleteProgram(self.shadow_program);

            gl::DeleteFramebuffers(1, &self.shadow_fbo);
            gl::DeleteTextures(1, &self.shadow_depth_tex);

            // The fallback texture is also cached under "Block_tx". Do not
            // delete that same GL texture twice.
            for &tex in self.textures.borrow().values() {
                if tex != self.fallback_tex {
                    gl::DeleteTextures(1, &tex);
                }
            }
            gl::DeleteTextures(1, &self.fallback_tex);

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

            gl::DeleteVertexArrays(1, &self.block_vao);
            gl::DeleteBuffers(1, &self.block_vbo);

            gl::DeleteVertexArrays(1, &self.billboard_vao);
            gl::DeleteBuffers(1, &self.billboard_vbo);
        }
    }
}

fn create_grid_plane_vao(plane: GridPlane) -> (u32, u32, i32) {
    let vertices = crate::editor::grid::generate_grid_vertices(plane);
    upload_vertices_3d(&vertices)
}

fn create_axes() -> (u32, u32, i32) {
    let length = 3.0;
    let mut vertices = Vec::new();

    add_line(
        &mut vertices,
        [0.0, 0.0, 0.0],
        [length, 0.0, 0.0],
        [1.0, 0.2, 0.2, 1.0],
    );

    add_line(
        &mut vertices,
        [0.0, 0.0, 0.0],
        [0.0, length, 0.0],
        [0.2, 1.0, 0.2, 1.0],
    );

    add_line(
        &mut vertices,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, length],
        [0.2, 0.2, 1.0, 1.0],
    );

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
    use std::f32::consts::PI;

    let mut vertices = Vec::new();

    let color = [0.0, 1.0, 1.0, 1.0];
    let radius = 0.15;
    let center = [0.0, 0.0, 0.0];
    let cross_size = 0.3;

    add_line(
        &mut vertices,
        [-cross_size, 0.0, 0.0],
        [cross_size, 0.0, 0.0],
        color,
    );

    add_line(
        &mut vertices,
        [0.0, -cross_size, 0.0],
        [0.0, cross_size, 0.0],
        color,
    );

    add_line(
        &mut vertices,
        [0.0, 0.0, -cross_size],
        [0.0, 0.0, cross_size],
        color,
    );

    let latitudes = 8;
    let longitudes = 16;

    for i in 0..=latitudes {
        let lat = PI * (i as f32 / latitudes as f32 - 0.5);
        let y = lat.sin() * radius + center[1];
        let r = lat.cos() * radius;

        for j in 0..longitudes {
            let lon1 = 2.0 * PI * (j as f32 / longitudes as f32);
            let lon2 = 2.0 * PI * ((j + 1) as f32 / longitudes as f32);

            add_line(
                &mut vertices,
                [lon1.cos() * r + center[0], y, lon1.sin() * r + center[2]],
                [lon2.cos() * r + center[0], y, lon2.sin() * r + center[2]],
                color,
            );
        }
    }

    for i in 0..longitudes {
        let lon = 2.0 * PI * (i as f32 / longitudes as f32);
        let cos_lon = lon.cos();
        let sin_lon = lon.sin();

        for j in 0..latitudes {
            let lat1 = PI * (j as f32 / latitudes as f32 - 0.5);
            let lat2 = PI * ((j + 1) as f32 / latitudes as f32 - 0.5);

            add_line(
                &mut vertices,
                [
                    cos_lon * lat1.cos() * radius + center[0],
                    lat1.sin() * radius + center[1],
                    sin_lon * lat1.cos() * radius + center[2],
                ],
                [
                    cos_lon * lat2.cos() * radius + center[0],
                    lat2.sin() * radius + center[1],
                    sin_lon * lat2.cos() * radius + center[2],
                ],
                color,
            );
        }
    }

    upload_vertices_3d(&vertices)
}

fn create_billboard_vao() -> (u32, u32) {
    let mut vertices = Vec::new();

    let color = [1.0, 1.0, 1.0, 1.0];
    let normal = [0.0, 0.0, 1.0];

    add_block_quad(
        &mut vertices,
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
        color,
        normal,
    );

    let (vao, vbo, _) = self::mesh::upload_block_vertices_3d(&vertices);
    (vao, vbo)
}

fn load_texture_from_file(path: &str) -> Option<u32> {
    match image::open(path) {
        Ok(image) => {
            let rgba = image.to_rgba8();
            let (width, height) = rgba.dimensions();

            let mut texture = 0;

            unsafe {
                gl::GenTextures(1, &mut texture);
                gl::BindTexture(gl::TEXTURE_2D, texture);

                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    width as i32,
                    height as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    rgba.as_raw().as_ptr() as *const _,
                );

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            Some(texture)
        }

        Err(error) => {
            eprintln!("Failed to load texture {path}: {error:?}");
            None
        }
    }
}

fn load_cubemap_from_file(path: &str) -> Option<u32> {
    match image::open(path) {
        Ok(image) => {
            let (width, height) = image.dimensions();

            if width != height * 4 / 3 {
                eprintln!(
                    "Invalid cubemap dimensions for cross layout: {}x{}. Expected 4:3 ratio.",
                    width, height
                );
                return None;
            }

            let face_size = height / 3;
            let rgba_image = image.to_rgba8();

            let mut texture = 0;

            unsafe {
                gl::GenTextures(1, &mut texture);
                gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);
            }

            // The X arms use the opposite convention from the default
            // OpenGL horizontal-cross layout.
            let faces = [
                (0, 1, gl::TEXTURE_CUBE_MAP_POSITIVE_X),
                (2, 1, gl::TEXTURE_CUBE_MAP_NEGATIVE_X),
                (1, 0, gl::TEXTURE_CUBE_MAP_POSITIVE_Y),
                (1, 2, gl::TEXTURE_CUBE_MAP_NEGATIVE_Y),
                (3, 1, gl::TEXTURE_CUBE_MAP_POSITIVE_Z),
                (1, 1, gl::TEXTURE_CUBE_MAP_NEGATIVE_Z),
            ];

            for (column, row, target) in faces {
                let x = column * face_size;
                let y = row * face_size;
                let face = rgba_image.view(x, y, face_size, face_size).to_image();

                unsafe {
                    gl::TexImage2D(
                        target,
                        0,
                        gl::RGBA as i32,
                        face_size as i32,
                        face_size as i32,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        face.as_raw().as_ptr() as *const _,
                    );
                }
            }

            unsafe {
                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR as i32,
                );
                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_MAG_FILTER,
                    gl::LINEAR as i32,
                );
                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_WRAP_S,
                    gl::CLAMP_TO_EDGE as i32,
                );
                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_WRAP_T,
                    gl::CLAMP_TO_EDGE as i32,
                );
                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_WRAP_R,
                    gl::CLAMP_TO_EDGE as i32,
                );

                gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
            }

            Some(texture)
        }

        Err(error) => {
            eprintln!("Failed to load cubemap {path}: {error:?}");
            None
        }
    }
}

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
        u_model * vec4(a_position, 1.0);

    v_world_pos = world_pos.xyz;

    gl_Position =
        u_view_projection * world_pos;

    v_normal =
        mat3(u_model) * a_normal;

    v_color =
        vec4(
            a_color.rgb * u_base_color,
            a_color.a
        );

    v_uv = a_uv;
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

float calculate_shadow(vec3 world_pos, vec3 normal) {
    vec4 light_space_pos =
        u_light_space_matrix *
        vec4(world_pos, 1.0);

    vec3 proj_coords =
        light_space_pos.xyz /
        light_space_pos.w;

    proj_coords =
        proj_coords * 0.5 + 0.5;

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

    vec3 light_direction =
        normalize(-u_global_light_direction);

    float normal_alignment =
        max(dot(normalize(normal), light_direction), 0.0);

    // The old fixed 0.005 bias was large relative to the current
    // orthographic depth range and could visibly erase nearby shadows.
    // Keep the bias small and increase it only for grazing surfaces.
    float bias =
        mix(0.0005, 0.00005, normal_alignment);

    float shadow = 0.0;

    vec2 texel_size =
        1.0 / textureSize(
            u_shadow_map,
            0
        );

    for (int x = -1; x <= 1; ++x) {
        for (int y = -1; y <= 1; ++y) {
            float pcf_depth =
                texture(
                    u_shadow_map,
                    proj_coords.xy +
                        vec2(x, y) * texel_size
                ).r;

            shadow +=
                current_depth - bias > pcf_depth
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
                ? calculate_shadow(v_world_pos, normal)
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
