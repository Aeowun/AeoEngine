use std::collections::HashMap;

use crate::engine::EditorMode;
use crate::world::{CellType, ChunkCoord, World, WorldCoord};

use super::Renderer;
use super::chunk::{
    ChunkMesh, TextureBatchRange, build_cpu_chunk_data, expand_dirty_coords_to_chunks,
    upload_position_only_vertices,
};
use super::mesh::{BLOCK_VERTEX_FLOATS, compute_exposed_faces_main, upload_block_vertices_3d};

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

        self.ghost_cache.borrow_mut().clear();
        *self.last_ghost_revision.borrow_mut() = 0;
        *self.last_ghost_mode.borrow_mut() = None;
    }

    /// Updates the cached editor ghosts only when render-relevant world state
    /// or renderer mode changes.
    ///
    /// Unchanged frames must not rescan the World or recompute exposed faces.
    pub(super) fn update_ghost_cache(&self, world: &World, mode: EditorMode) {
        let revision = world.render_revision();

        let unchanged = *self.last_ghost_revision.borrow() == revision
            && *self.last_ghost_mode.borrow() == Some(mode);

        if unchanged {
            return;
        }

        let mut ghosts = self.ghost_cache.borrow_mut();

        ghosts.clear();

        if mode == EditorMode::Editor {
            for coord in world.iter_active_effective_coords() {
                let Some(cell) = world.get_effective_cell(coord) else {
                    continue;
                };

                if !matches!(cell.cell_type, CellType::Block | CellType::SpawnPoint) {
                    continue;
                }

                if world.is_cell_visible(coord) {
                    continue;
                }

                let mask = compute_exposed_faces_main(world, coord, mode);

                if mask == 0 {
                    continue;
                }

                ghosts.push(super::GhostRenderCell {
                    coord,
                    mask,
                    color: world.get_effective_color(coord),
                    visual_offset: world.get_visual_offset(coord),
                });
            }
        }

        *self.last_ghost_revision.borrow_mut() = revision;
        *self.last_ghost_mode.borrow_mut() = Some(mode);
    }

    pub fn update_chunk_cache(&self, world: &World, mode: EditorMode) {
        self.update_ghost_cache(world, mode);

        let dirty_coords = world.drain_render_dirty_cells();

        let mode_changed = *self.last_render_mode.borrow() != Some(mode);

        let revision_reset = world.render_revision() < *self.last_render_revision.borrow();

        if dirty_coords.is_empty()
            && !mode_changed
            && !revision_reset
            && !self.chunk_cache.borrow().is_empty()
        {
            return;
        }

        let mut cache = self.chunk_cache.borrow_mut();

        if mode_changed || revision_reset || cache.is_empty() {
            for chunk_mesh in cache.values_mut() {
                chunk_mesh.free_gl_resources();
            }

            cache.clear();

            for chunk_coord in world.iter_active_chunks() {
                if let Some(coords) = world.get_active_coords_in_chunk(chunk_coord) {
                    let coords_vec: Vec<WorldCoord> = coords.iter().copied().collect();

                    self.rebuild_single_chunk(world, &mut cache, chunk_coord, &coords_vec, mode);
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
    use super::*;
    use crate::renderer::Renderer;
    use crate::world::DirtyReason;

    #[test]
    fn test_renderer_chunk_cache_cleared_on_project_reset() {
        let renderer = Renderer::new_for_tests(800.0, 600.0);

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

        assert!(
            renderer.ghost_cache.borrow().is_empty(),
            "Ghost cache must be cleared when the renderer cache is cleared"
        );

        assert_eq!(*renderer.last_ghost_revision.borrow(), 0);

        assert_eq!(*renderer.last_ghost_mode.borrow(), None);
    }

    #[test]
    fn test_renderer_chunk_cache_invalidates_on_world_revision_reset() {
        let renderer = Renderer::new_for_tests(800.0, 600.0);

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

    #[test]
    fn test_renderer_selective_chunk_rebuild_preserves_unmodified_chunks() {
        let renderer = Renderer::new_for_tests(800.0, 600.0);

        let mut world = World::new();

        // This represents a renderer whose cache has already been populated
        // for the current world and editor mode.
        *renderer.last_render_mode.borrow_mut() = Some(EditorMode::Editor);
        *renderer.last_render_revision.borrow_mut() = world.render_revision();

        let distant_chunk = ChunkCoord::new(10, 10, 10);

        let mock_mesh = ChunkMesh {
            chunk_coord: distant_chunk,
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
            .insert(distant_chunk, mock_mesh);

        world.mark_render_dirty(WorldCoord::new(0, 0, 0), DirtyReason::Geometry);

        world.bump_render_revision();

        renderer.update_chunk_cache(&world, EditorMode::Editor);

        let cache = renderer.chunk_cache.borrow();

        assert!(
            cache.contains_key(&distant_chunk),
            "Distant chunk mesh should remain cached when an unrelated chunk is dirtied"
        );
    }

    #[test]
    fn test_renderer_ghost_cache_rebuilds_only_when_render_state_changes() {
        let renderer = Renderer::new_for_tests(800.0, 600.0);

        let mut world = World::new();

        let coord = WorldCoord::new(0, 0, 0);

        world.set_cell(coord, CellType::Block);
        world.set_cell_visible_runtime(coord, false);

        renderer.update_ghost_cache(&world, EditorMode::Editor);

        let first_revision = *renderer.last_ghost_revision.borrow();

        let first_cache = renderer.ghost_cache.borrow().clone();

        assert_eq!(first_cache.len(), 1);

        renderer.update_ghost_cache(&world, EditorMode::Editor);

        assert_eq!(*renderer.last_ghost_revision.borrow(), first_revision);

        assert_eq!(*renderer.ghost_cache.borrow(), first_cache);
    }

    #[test]
    fn test_renderer_ghost_cache_updates_when_cell_render_state_changes() {
        use glam::Vec3;

        let renderer = Renderer::new_for_tests(800.0, 600.0);

        let mut world = World::new();

        let coord = WorldCoord::new(0, 0, 0);

        world.set_cell(coord, CellType::Block);
        world.set_cell_visible_runtime(coord, false);

        renderer.update_ghost_cache(&world, EditorMode::Editor);

        let first_revision = *renderer.last_ghost_revision.borrow();

        world.set_cell_color_runtime(coord, Vec3::new(1.0, 0.0, 0.0));

        renderer.update_ghost_cache(&world, EditorMode::Editor);

        let ghosts = renderer.ghost_cache.borrow();

        assert_eq!(ghosts.len(), 1);

        assert_eq!(ghosts[0].color, Vec3::new(1.0, 0.0, 0.0));

        assert!(
            *renderer.last_ghost_revision.borrow() > first_revision,
            "Ghost cache revision must advance after a render-relevant world change"
        );
    }

    #[test]
    fn test_renderer_ghost_cache_clears_outside_editor_mode() {
        let renderer = Renderer::new_for_tests(800.0, 600.0);

        let mut world = World::new();

        let coord = WorldCoord::new(0, 0, 0);

        world.set_cell(coord, CellType::Block);
        world.set_cell_visible_runtime(coord, false);

        renderer.update_ghost_cache(&world, EditorMode::Editor);

        assert_eq!(renderer.ghost_cache.borrow().len(), 1);

        renderer.update_ghost_cache(&world, EditorMode::Play);

        assert!(
            renderer.ghost_cache.borrow().is_empty(),
            "Editor ghost geometry must not remain cached in Play mode"
        );
    }

    #[test]
    fn test_benchmark_large_world_performance_cliff() {
        use std::time::Instant;

        use crate::renderer::chunk::build_cpu_chunk_data;

        for &block_count in &[5_000, 10_000, 20_000] {
            let mut world = World::new();

            let side = (block_count as f32).cbrt().ceil() as i32;

            let mut count = 0;

            'outer: for x in 0..side {
                for y in 0..side {
                    for z in 0..side {
                        world.set_cell(WorldCoord::new(x, y, z), CellType::Block);

                        count += 1;

                        if count >= block_count {
                            break 'outer;
                        }
                    }
                }
            }

            let start_full = Instant::now();

            let mut total_batches = 0;

            for chunk_coord in world.iter_active_chunks() {
                if let Some(coords) = world.get_active_coords_in_chunk(chunk_coord) {
                    let coords_vec: Vec<WorldCoord> = coords.iter().copied().collect();

                    let cpu_data =
                        build_cpu_chunk_data(&world, chunk_coord, &coords_vec, EditorMode::Editor);

                    total_batches += cpu_data.main_texture_vertices.len();
                }
            }

            let full_build_time = start_full.elapsed();

            world.set_cell(WorldCoord::new(0, 0, 0), CellType::Empty);

            let start_edit = Instant::now();

            let dirty_coords = world.drain_render_dirty_cells();

            let dirty_chunks = expand_dirty_coords_to_chunks(&dirty_coords);

            let mut edit_batches = 0;

            for chunk_coord in dirty_chunks {
                let coords_vec: Vec<WorldCoord> = world
                    .get_active_coords_in_chunk(chunk_coord)
                    .map(|set| set.iter().copied().collect())
                    .unwrap_or_default();

                let cpu_data =
                    build_cpu_chunk_data(&world, chunk_coord, &coords_vec, EditorMode::Editor);

                edit_batches += cpu_data.main_texture_vertices.len();
            }

            let edit_time = start_edit.elapsed();

            println!(
                "[BENCHMARK] World size: {} blocks | Full CPU chunk build: {:?} (batches: {}) | Selective edit rebuild: {:?} (batches: {})",
                block_count, full_build_time, total_batches, edit_time, edit_batches
            );
        }
    }
}
