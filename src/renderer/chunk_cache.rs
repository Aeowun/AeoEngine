use std::collections::HashMap;

use crate::engine::EditorMode;
use crate::world::{ChunkCoord, World, WorldCoord};

use super::Renderer;
use super::chunk::{build_cpu_chunk_data, expand_dirty_coords_to_chunks, upload_position_only_vertices, ChunkMesh, TextureBatchRange};
use super::mesh::{upload_block_vertices_3d, BLOCK_VERTEX_FLOATS};

impl Renderer {
    /// Clears and frees all GPU resources for cached voxel chunk meshes.
    ///
    /// Must be called when transitioning project boundaries or resetting the world
    /// to ensure stale project render state does not survive.
    pub fn clear_chunk_cache(&self) {
        let mut cache = self.chunk_cache.borrow_mut();

        for chunk_mesh in cache.values_mut() {
            chunk_mesh.free_gl_resources();
        }

        cache.clear();

        *self.last_render_mode.borrow_mut() = None;
        *self.last_render_revision.borrow_mut() = 0;
    }

    pub fn update_chunk_cache(&self, world: &World, mode: EditorMode) {
        let dirty_coords = world.drain_render_dirty_cells();

        let mode_changed = *self.last_render_mode.borrow() != Some(mode);

        let revision_changed =
            *self.last_render_revision.borrow() != world.render_revision();

        if dirty_coords.is_empty()
            && !mode_changed
            && !revision_changed
            && !self.chunk_cache.borrow().is_empty()
        {
            return;
        }

        let mut cache = self.chunk_cache.borrow_mut();

        if mode_changed || revision_changed || cache.is_empty() {
            for chunk_mesh in cache.values_mut() {
                chunk_mesh.free_gl_resources();
            }

            cache.clear();

            for chunk_coord in world.iter_active_chunks() {
                if let Some(coords) = world.get_active_coords_in_chunk(chunk_coord) {
                    let coords_vec: Vec<WorldCoord> = coords.iter().copied().collect();

                    self.rebuild_single_chunk(
                        world,
                        &mut cache,
                        chunk_coord,
                        &coords_vec,
                        mode,
                    );
                }
            }

            *self.last_render_mode.borrow_mut() = Some(mode);

            *self.last_render_revision.borrow_mut() = world.render_revision();

            return;
        }

        let dirty_chunks = expand_dirty_coords_to_chunks(&dirty_coords);

        for chunk_coord in dirty_chunks {
            let coords_vec: Vec<WorldCoord> = world
                .get_active_coords_in_chunk(chunk_coord)
                .map(|set| set.iter().copied().collect())
                .unwrap_or_default();

            self.rebuild_single_chunk(world, &mut cache, chunk_coord, &coords_vec, mode);
        }

        *self.last_render_revision.borrow_mut() = world.render_revision();
    }

    fn rebuild_single_chunk(
        &self,
        world: &World,
        cache: &mut HashMap<ChunkCoord, ChunkMesh>,
        chunk_coord: ChunkCoord,
        coords_in_chunk: &[WorldCoord],
        mode: EditorMode,
    ) {
        let cpu_data = build_cpu_chunk_data(world, chunk_coord, coords_in_chunk, mode);

        // A streamed chunk can become empty when it is evicted. Remove its
        // previous GPU mesh instead of leaving stale geometry visible.
        if cpu_data.main_texture_vertices.is_empty() && cpu_data.shadow_positions.is_empty() {
            if let Some(mut old_mesh) = cache.remove(&chunk_coord) {
                old_mesh.free_gl_resources();
            }
            return;
        }

        let mut main_vao = 0;
        let mut main_vbo = 0;
        let mut main_texture_batches = Vec::new();

        let mut combined_main_vertices = Vec::new();

        let mut current_vertex_offset = 0i32;

        let mut texture_ids: Vec<_> = cpu_data.main_texture_vertices.keys().cloned().collect();

        texture_ids.sort();

        for tex_id in texture_ids {
            if let Some(verts) = cpu_data.main_texture_vertices.get(&tex_id) {
                if verts.is_empty() {
                    continue;
                }

                let count = (verts.len() / BLOCK_VERTEX_FLOATS) as i32;

                main_texture_batches.push(TextureBatchRange {
                    texture_id: tex_id,
                    start_vertex: current_vertex_offset,
                    vertex_count: count,
                });

                combined_main_vertices.extend_from_slice(verts);

                current_vertex_offset += count;
            }
        }

        if !combined_main_vertices.is_empty() {
            let (vao, vbo, _) = upload_block_vertices_3d(&combined_main_vertices);

            main_vao = vao;
            main_vbo = vbo;
        }

        let mut shadow_vao = 0;
        let mut shadow_vbo = 0;
        let mut shadow_vertex_count = 0;

        if !cpu_data.shadow_positions.is_empty() {
            let (s_vao, s_vbo, count) = upload_position_only_vertices(&cpu_data.shadow_positions);

            shadow_vao = s_vao;
            shadow_vbo = s_vbo;
            shadow_vertex_count = count;
        }

        if main_vao != 0 || shadow_vao != 0 {
            if let Some(mut old_mesh) = cache.remove(&chunk_coord) {
                old_mesh.free_gl_resources();
            }

            let chunk_mesh = ChunkMesh {
                chunk_coord,
                main_vao,
                main_vbo,
                main_texture_batches,
                shadow_vao,
                shadow_vbo,
                shadow_vertex_count,
            };

            cache.insert(chunk_coord, chunk_mesh);
        } else if let Some(mut old_mesh) = cache.remove(&chunk_coord) {
            old_mesh.free_gl_resources();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::collections::HashMap;

    use super::*;
    use crate::engine::EditorMode;
    use crate::renderer::chunk::ChunkMesh;
    use crate::renderer::sky;
    use crate::world::{World, WorldCoord};

    #[test]
    fn test_renderer_chunk_cache_cleared_on_project_reset() {
        let renderer = Renderer {
            chunk_cache: RefCell::new(HashMap::new()),
            last_render_mode: RefCell::new(Some(EditorMode::Editor)),
            last_render_revision: RefCell::new(10),
            grid_program: 0,
            ghost_program: 0,
            ghost_view_projection_location: -1,
            ghost_model_location: -1,
            ghost_color_location: -1,
            ghost_alpha_location: -1,
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
            width: 800.0,
            height: 600.0,
        };

        let mock_mesh = ChunkMesh {
            chunk_coord: ChunkCoord::new(0, 0, 0),
            main_vao: 0,
            main_vbo: 0,
            main_texture_batches: Vec::new(),
            shadow_vao: 0,
            shadow_vbo: 0,
            shadow_vertex_count: 0,
        };

        renderer
            .chunk_cache
            .borrow_mut()
            .insert(ChunkCoord::new(0, 0, 0), mock_mesh);

        assert!(!renderer.chunk_cache.borrow().is_empty());

        renderer.clear_chunk_cache();

        assert!(
            renderer.chunk_cache.borrow().is_empty(),
            "Chunk cache must be completely empty after clear_chunk_cache()"
        );

        assert_eq!(*renderer.last_render_revision.borrow(), 0);

        assert_eq!(*renderer.last_render_mode.borrow(), None);
    }

    #[test]
    fn test_renderer_chunk_cache_invalidates_on_world_revision_change() {
        let renderer = Renderer {
            chunk_cache: RefCell::new(HashMap::new()),
            last_render_mode: RefCell::new(Some(EditorMode::Editor)),
            last_render_revision: RefCell::new(999),
            grid_program: 0,
            ghost_program: 0,
            ghost_view_projection_location: -1,
            ghost_model_location: -1,
            ghost_color_location: -1,
            ghost_alpha_location: -1,
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
            width: 800.0,
            height: 600.0,
        };

        let mock_mesh = ChunkMesh {
            chunk_coord: ChunkCoord::new(0, 0, 0),
            main_vao: 0,
            main_vbo: 0,
            main_texture_batches: Vec::new(),
            shadow_vao: 0,
            shadow_vbo: 0,
            shadow_vertex_count: 0,
        };

        renderer
            .chunk_cache
            .borrow_mut()
            .insert(ChunkCoord::new(0, 0, 0), mock_mesh);

        let new_blank_world = World::new();

        renderer.update_chunk_cache(&new_blank_world, EditorMode::Editor);

        assert!(
            renderer.chunk_cache.borrow().is_empty(),
            "Chunk cache must be empty for blank world with 0 blocks"
        );

        assert_eq!(
            *renderer.last_render_revision.borrow(),
            new_blank_world.render_revision()
        );
    }
}
