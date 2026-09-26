use glam::{Mat4, Vec3};

use crate::editor::{Editor, GridPlane};
use crate::engine::EditorMode;
use crate::engine::physics::PhysicsWorld;
use crate::world::{CellType, World, WorldCoord};

use super::Renderer;
use super::chunk::CameraFrustum;
use super::{SHADOW_FAR, SHADOW_LIGHT_DISTANCE, SHADOW_NEAR, SHADOW_ORTHO_HALF_EXTENT, SHADOW_RES};

impl Renderer {
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

        let ppp = editor.viewport_ppp.max(1.0);

        let rect = editor.viewport_rect;

        let (vp_x, vp_y, vp_w, vp_h, aspect_ratio) = if rect.width() > 1.0 && rect.height() > 1.0 {
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

        let view_segment = target - camera_pos;

        const CHARACTER_CAMERA_HIDE_DISTANCE: f32 = 2.0;

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

            self.update_chunk_cache(world, editor.mode);

            // Shadow pass.
            {
                gl::BindFramebuffer(gl::FRAMEBUFFER, self.shadow_fbo);

                gl::Viewport(0, 0, SHADOW_RES, SHADOW_RES);

                gl::Clear(gl::DEPTH_BUFFER_BIT);

                gl::UseProgram(self.shadow_program);

                gl::UniformMatrix4fv(
                    self.shadow_light_space_matrix_location,
                    1,
                    gl::FALSE,
                    light_space_matrix.to_cols_array().as_ptr(),
                );

                let cache = self.chunk_cache.borrow();

                for chunk_mesh in cache.values() {
                    if chunk_mesh.shadow_vao != 0 && chunk_mesh.shadow_vertex_count > 0 {
                        let origin = chunk_mesh.chunk_coord.world_origin();

                        let model = Mat4::from_translation(origin);

                        gl::UniformMatrix4fv(
                            self.shadow_model_location,
                            1,
                            gl::FALSE,
                            model.to_cols_array().as_ptr(),
                        );

                        gl::BindVertexArray(chunk_mesh.shadow_vao);

                        gl::DrawArrays(gl::TRIANGLES, 0, chunk_mesh.shadow_vertex_count);
                    }
                }

                if editor.mode == EditorMode::Play {
                    let (first_vertex, count) = self.get_block_mask_range(63);

                    gl::BindVertexArray(self.block_vao);

                    for body in &physics.bodies {
                        if !body.solid {
                            continue;
                        }

                        let model = Mat4::from_translation(body.position);

                        gl::UniformMatrix4fv(
                            self.shadow_model_location,
                            1,
                            gl::FALSE,
                            model.to_cols_array().as_ptr(),
                        );

                        gl::DrawArrays(gl::TRIANGLES, first_vertex, count);
                    }
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

            let vp_location = self.grid_view_projection_location;
            let model_location = self.grid_model_location;
            let base_color_location = self.grid_base_color_location;
            let alpha_location = self.grid_alpha_location;
            let use_tex_location = self.grid_use_texture_location;

            gl::UniformMatrix4fv(
                vp_location,
                1,
                gl::FALSE,
                view_projection.to_cols_array().as_ptr(),
            );

            gl::Uniform3f(base_color_location, 1.0, 1.0, 1.0);

            gl::Uniform1f(alpha_location, 1.0);

            // Global lighting.
            gl::Uniform1f(
                self.grid_ambient_intensity_location,
                world.lighting.ambient_intensity,
            );

            gl::Uniform1i(
                self.grid_global_light_enabled_location,
                if world.lighting.global_light_enabled {
                    1
                } else {
                    0
                },
            );

            gl::Uniform3f(
                self.grid_global_light_direction_location,
                world.lighting.global_light_direction.x,
                world.lighting.global_light_direction.y,
                world.lighting.global_light_direction.z,
            );

            gl::Uniform3f(
                self.grid_global_light_color_location,
                world.lighting.global_light_color.x,
                world.lighting.global_light_color.y,
                world.lighting.global_light_color.z,
            );

            gl::Uniform1f(
                self.grid_global_light_intensity_location,
                world.lighting.global_light_intensity,
            );

            // Shadow uniforms.
            gl::UniformMatrix4fv(
                self.grid_light_space_matrix_location,
                1,
                gl::FALSE,
                light_space_matrix.to_cols_array().as_ptr(),
            );

            gl::Uniform1i(
                self.grid_shadows_enabled_location,
                if world.lighting.shadows_enabled { 1 } else { 0 },
            );

            gl::ActiveTexture(gl::TEXTURE0);

            gl::BindTexture(gl::TEXTURE_2D, self.shadow_depth_tex);

            gl::Uniform1i(self.grid_shadow_map_location, 0);

            gl::ActiveTexture(gl::TEXTURE1);

            gl::BindTexture(gl::TEXTURE_2D, self.fallback_tex);

            // Point lights.
            let mut point_lights = Vec::new();

            for light_id in world.iter_light_ids() {
                let Some(coord) = world.resolve_cell_id(light_id) else {
                    continue;
                };

                let Some(cell) = world.get_effective_cell(coord) else {
                    continue;
                };

                if world.is_light_enabled(coord) {
                    point_lights.push((coord, cell));

                    if point_lights.len() >= 16 {
                        break;
                    }
                }
            }

            gl::Uniform1i(
                self.grid_point_light_count_location,
                point_lights.len() as i32,
            );

            for (i, (coord, cell)) in point_lights.iter().enumerate() {
                let uniforms = self.grid_point_light_uniforms[i];

                gl::Uniform3f(
                    uniforms.position,
                    coord.x as f32,
                    coord.y as f32,
                    coord.z as f32,
                );

                gl::Uniform3f(
                    uniforms.color,
                    cell.light_color.x,
                    cell.light_color.y,
                    cell.light_color.z,
                );

                gl::Uniform1f(uniforms.intensity, cell.light_intensity);

                gl::Uniform1f(uniforms.range, cell.light_range);
            }

            // Authored world editor markers (Light and AudioEmitter).
            let cam_right = Vec3::new(view.col(0).x, view.col(1).x, view.col(2).x);

            let cam_up = Vec3::new(view.col(0).y, view.col(1).y, view.col(2).y);

            if editor.mode == EditorMode::Editor {
                for marker_id in world.iter_editor_marker_ids() {
                    let Some(coord) = world.resolve_cell_id(marker_id) else {
                        continue;
                    };

                    let Some(cell) = world.get_effective_cell(coord) else {
                        continue;
                    };

                    gl::BindVertexArray(self.billboard_vao);

                    gl::Uniform1i(use_tex_location, 1);

                    let tex_name = match cell.cell_type {
                        CellType::Light => "lightbulb",
                        CellType::AudioEmitter => "speaker",
                        _ => continue,
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
            }

            // Static voxel chunk rendering.
            let camera_frustum = CameraFrustum::from_view_projection(view_projection);

            gl::Uniform1i(use_tex_location, 1);

            let cache = self.chunk_cache.borrow();

            for chunk_mesh in cache.values() {
                if chunk_mesh.main_vao == 0 || chunk_mesh.main_texture_batches.is_empty() {
                    continue;
                }

                let (aabb_min, aabb_max) = chunk_mesh.chunk_coord.aabb_min_max();

                if !camera_frustum.intersects_aabb(aabb_min, aabb_max) {
                    continue;
                }

                let origin = chunk_mesh.chunk_coord.world_origin();

                let model = Mat4::from_translation(origin);

                gl::UniformMatrix4fv(model_location, 1, gl::FALSE, model.to_cols_array().as_ptr());

                gl::BindVertexArray(chunk_mesh.main_vao);

                for batch in &chunk_mesh.main_texture_batches {
                    let tex = self.get_texture(&batch.texture_id);

                    gl::ActiveTexture(gl::TEXTURE1);

                    gl::BindTexture(gl::TEXTURE_2D, tex);

                    gl::DrawArrays(gl::TRIANGLES, batch.start_vertex, batch.vertex_count);
                }
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
                    let camera_distance_squared = character
                        .transform
                        .position
                        .distance_squared(gameplay_camera.current_position);

                    if camera_distance_squared < CHARACTER_CAMERA_HIDE_DISTANCE.powi(2) {
                        continue;
                    }

                    let vertices = crate::character::generate_character_mesh(character);

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

                    self.draw_character_mesh_persistent(&vertices);
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

                gl::UniformMatrix4fv(model_location, 1, gl::FALSE, model.to_cols_array().as_ptr());

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

                gl::UniformMatrix4fv(model_location, 1, gl::FALSE, model.to_cols_array().as_ptr());

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

                if let Some(ref grab) = editor.grab_state {
                    let color = if grab.valid {
                        (0.2, 0.6, 1.0)
                    } else {
                        (1.0, 0.2, 0.2)
                    };

                    gl::Uniform3f(base_color_location, color.0, color.1, color.2);

                    for coord in &grab.source_coords {
                        let model = Mat4::from_translation(Vec3::new(
                            (coord.x + grab.delta.x) as f32,
                            (coord.y + grab.delta.y) as f32,
                            (coord.z + grab.delta.z) as f32,
                        ));

                        gl::UniformMatrix4fv(
                            model_location,
                            1,
                            gl::FALSE,
                            model.to_cols_array().as_ptr(),
                        );

                        gl::DrawArrays(gl::LINES, 0, self.highlight_vertex_count);
                    }
                } else {
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
                }

                let model = Mat4::from_translation(anchor_pos);

                gl::UniformMatrix4fv(model_location, 1, gl::FALSE, model.to_cols_array().as_ptr());

                gl::BindVertexArray(self.axis_vao);

                gl::DrawArrays(gl::LINES, 0, self.axis_vertex_count);
            }

            // Editor ghost rendering.
            if editor.mode == EditorMode::Editor {
                // Grab preview.
                if let Some(ref grab) = editor.grab_state {
                    gl::Enable(gl::BLEND);
                    gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

                    gl::DepthMask(gl::FALSE);

                    gl::Uniform1f(alpha_location, 0.5);

                    for coord in &grab.source_coords {
                        let Some(cell) = world.get(*coord) else {
                            continue;
                        };

                        let preview_coord = WorldCoord::new(
                            coord.x + grab.delta.x,
                            coord.y + grab.delta.y,
                            coord.z + grab.delta.z,
                        );

                        let is_marker =
                            matches!(cell.cell_type, CellType::Light | CellType::AudioEmitter);

                        if is_marker {
                            gl::BindVertexArray(self.highlight_vao);

                            let (r, g, b) = if !grab.valid {
                                (1.0, 0.2, 0.2)
                            } else if cell.cell_type == CellType::AudioEmitter {
                                (0.2, 0.8, 1.0)
                            } else {
                                (1.0, 1.0, 0.2)
                            };

                            gl::Uniform3f(base_color_location, r, g, b);

                            let model = Mat4::from_translation(Vec3::new(
                                preview_coord.x as f32,
                                preview_coord.y as f32,
                                preview_coord.z as f32,
                            ));

                            gl::UniformMatrix4fv(
                                model_location,
                                1,
                                gl::FALSE,
                                model.to_cols_array().as_ptr(),
                            );

                            gl::DrawArrays(gl::LINES, 0, self.highlight_vertex_count);
                        } else {
                            gl::BindVertexArray(self.block_vao);

                            let (first_vertex, count) = self.get_block_mask_range(63);

                            let color = if grab.valid {
                                cell.color_rgb
                            } else {
                                Vec3::new(1.0, 0.2, 0.2)
                            };

                            gl::Uniform3f(base_color_location, color.x, color.y, color.z);

                            gl::Uniform1i(use_tex_location, 0);

                            let model = Mat4::from_translation(Vec3::new(
                                preview_coord.x as f32,
                                preview_coord.y as f32,
                                preview_coord.z as f32,
                            ));

                            gl::UniformMatrix4fv(
                                model_location,
                                1,
                                gl::FALSE,
                                model.to_cols_array().as_ptr(),
                            );

                            gl::DrawArrays(gl::TRIANGLES, first_vertex, count);
                        }
                    }

                    gl::DepthMask(gl::TRUE);

                    gl::Uniform1f(alpha_location, 1.0);
                }

                // Invisible-cell ghosts are cached by Renderer. The cache is
                // rebuilt only when render-relevant World state or editor mode changes.
                gl::UseProgram(self.ghost_program);

                gl::UniformMatrix4fv(
                    self.ghost_view_projection_location,
                    1,
                    gl::FALSE,
                    view_projection.to_cols_array().as_ptr(),
                );

                gl::Uniform1f(self.ghost_alpha_location, 0.4);

                gl::Uniform3f(self.ghost_color_location, 1.0, 1.0, 1.0);

                gl::Enable(gl::BLEND);
                gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

                gl::DepthMask(gl::FALSE);

                let ghost_chunks = self.ghost_cache.borrow();

                for ghost_mesh in ghost_chunks.values() {
                    if ghost_mesh.vao == 0 || ghost_mesh.vertex_count <= 0 {
                        continue;
                    }

                    let model = Mat4::from_translation(ghost_mesh.chunk_coord.world_origin());

                    gl::UniformMatrix4fv(
                        self.ghost_model_location,
                        1,
                        gl::FALSE,
                        model.to_cols_array().as_ptr(),
                    );

                    gl::BindVertexArray(ghost_mesh.vao);

                    gl::DrawArrays(gl::TRIANGLES, 0, ghost_mesh.vertex_count);
                }

                drop(ghost_chunks);

                // Return to the normal world shader before any
                // of the remaining editor previews use its uniforms.
                gl::UseProgram(self.grid_program);

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
                                                x as f32, y as f32, z as f32,
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
                                                x as f32, y as f32, z as f32,
                                            ));

                                            gl::UniformMatrix4fv(
                                                model_location,
                                                1,
                                                gl::FALSE,
                                                model.to_cols_array().as_ptr(),
                                            );

                                            gl::DrawArrays(gl::TRIANGLES, first_vertex, count);
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
                                            x as f32, y as f32, z as f32,
                                        ));

                                        gl::UniformMatrix4fv(
                                            model_location,
                                            1,
                                            gl::FALSE,
                                            model.to_cols_array().as_ptr(),
                                        );

                                        gl::DrawArrays(gl::LINES, 0, self.highlight_vertex_count);
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

            gl::UseProgram(self.grid_program);

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
