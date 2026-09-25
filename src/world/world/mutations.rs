//! World mutation and cell-index maintenance.
//!
//! Handles authored cell creation, replacement, deletion, movement, pasting,
//! ID remapping, and maintenance of the specialized Light, AudioEmitter,
//! and editor-marker lookup indices.
use crate::world::cell::{Cell, CellType, ChunkCoord};
use crate::world::coordinate::WorldCoord;

use super::World;
use super::dirty::DirtyReason;

impl World {
    fn index_special_cell(&mut self, id: u64, cell_type: CellType) {
        match cell_type {
            CellType::Light => {
                self.light_ids.insert(id);
                self.editor_marker_ids.insert(id);
            }

            CellType::AudioEmitter => {
                self.audio_emitter_ids.insert(id);
                self.editor_marker_ids.insert(id);
            }

            _ => {}
        }
    }

    fn unindex_special_cell(&mut self, id: u64, cell_type: CellType) {
        match cell_type {
            CellType::Light => {
                self.light_ids.remove(&id);
                self.editor_marker_ids.remove(&id);
            }

            CellType::AudioEmitter => {
                self.audio_emitter_ids.remove(&id);
                self.editor_marker_ids.remove(&id);
            }

            _ => {}
        }
    }

    pub fn get_mut(&mut self, coord: WorldCoord) -> Option<&mut Cell> {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);
        self.dirty_authored_chunks
            .insert(ChunkCoord::from_world_coord(coord));

        self.cells.get_mut(&coord)
    }

    pub fn set_cell(&mut self, coord: WorldCoord, cell_type: CellType) -> u64 {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);
        self.dirty_authored_chunks
            .insert(ChunkCoord::from_world_coord(coord));

        // Remove whatever currently occupies this coordinate first.
        let old_cell = self.cells.remove(&coord);

        if let Some(old_cell) = old_cell {
            self.id_to_coord.remove(&old_cell.id);
            self.runtime_state.remove(&old_cell.id);
            self.unindex_special_cell(old_cell.id, old_cell.cell_type);
            self.mark_physics_dirty(old_cell.id);

            // Empty means delete the cell entirely.
            if cell_type == CellType::Empty {
                self.update_spatial_index_at(coord);
                return old_cell.id;
            }
        } else if cell_type == CellType::Empty {
            self.update_spatial_index_at(coord);
            return 0;
        }

        let mut cell = match cell_type {
            CellType::Block => Cell::new_block(),
            CellType::Light => Cell::new_light(),
            CellType::SpawnPoint => Cell::new_spawn_point(),
            CellType::AudioEmitter => Cell::new_audio_emitter(),
            _ => {
                let mut cell = Cell::default();
                cell.cell_type = cell_type;
                cell
            }
        };

        cell.id = self.generate_unique_id(coord, cell_type);

        let id = cell.id;

        self.id_to_coord.insert(id, coord);
        self.mark_physics_dirty(id);
        self.index_special_cell(id, cell_type);

        self.cells.insert(coord, cell);

        self.update_spatial_index_at(coord);

        id
    }

    /// Internal helper to update the ID index when an ID is changed manually
    /// (for example while loading persisted cells).
    pub(crate) fn update_id_mapping(&mut self, old_id: u64, new_id: u64, coord: WorldCoord) {
        let cell_type = self.cells.get(&coord).map(|cell| cell.cell_type);

        if let Some(cell_type) = cell_type {
            self.unindex_special_cell(old_id, cell_type);
            self.index_special_cell(new_id, cell_type);
        }

        self.id_to_coord.remove(&old_id);
        self.id_to_coord.insert(new_id, coord);
        self.observe_authored_id(new_id);
    }

    /// Rebuilds the ID and specialized lookup indices.
    ///
    /// Call this if the authored cell map is replaced directly,
    /// such as during undo/redo or world loading.
    pub fn rebuild_id_mapping(&mut self) {
        self.bump_render_revision();

        let coords: Vec<WorldCoord> = self.cells.keys().copied().collect();

        for coord in coords {
            self.mark_render_dirty(coord, DirtyReason::Geometry);
        }

        self.id_to_coord.clear();

        for (coord, cell) in &self.cells {
            self.id_to_coord.insert(cell.id, *coord);
        }

        self.rebuild_spatial_index();
        self.rebuild_light_and_marker_indices();

        self.dirty_authored_chunks = self
            .cells
            .keys()
            .map(|coord| ChunkCoord::from_world_coord(*coord))
            .collect();
    }

    pub(crate) fn rebuild_light_and_marker_indices(&mut self) {
        self.light_ids.clear();
        self.editor_marker_ids.clear();
        self.audio_emitter_ids.clear();

        let entries: Vec<(u64, CellType)> = self
            .cells
            .values()
            .chain(self.runtime_cells.values())
            .map(|cell| (cell.id, cell.cell_type))
            .collect();

        for (id, cell_type) in entries {
            self.index_special_cell(id, cell_type);
        }
    }

    pub(crate) fn rebuild_audio_emitter_index(&mut self) {
        self.rebuild_light_and_marker_indices();
    }

    pub(crate) fn generate_unique_id(&mut self, _coord: WorldCoord, _cell_type: CellType) -> u64 {
        let id = self.next_cell_id;

        self.next_cell_id = self
            .next_cell_id
            .checked_add(1)
            .expect("Authored cell ID space exhausted");

        id
    }

    /// Advances the allocator past an existing authored ID.
    ///
    /// Used when loading IDs that were already assigned on disk.
    pub(crate) fn observe_authored_id(&mut self, id: u64) {
        if id >= self.next_cell_id {
            self.next_cell_id = id.checked_add(1).expect("Authored cell ID space exhausted");
        }
    }

    /// Moves a collection of authored cells by a given delta offset,
    /// preserving their cell IDs.
    ///
    /// Returns Err if the movement destination collides with unrelated cells.
    pub fn move_cells(
        &mut self,
        source_coords: &[WorldCoord],
        delta: WorldCoord,
    ) -> Result<(), String> {
        if delta == WorldCoord::new(0, 0, 0) || source_coords.is_empty() {
            return Ok(());
        }

        let source_set: std::collections::HashSet<WorldCoord> =
            source_coords.iter().copied().collect();

        // Validate that no destination coordinate collides with an unrelated cell.
        for &src in source_coords {
            let dest = WorldCoord::new(src.x + delta.x, src.y + delta.y, src.z + delta.z);

            if self.cells.contains_key(&dest) && !source_set.contains(&dest) {
                return Err("Destination is occupied by an unrelated cell".to_string());
            }
        }

        self.bump_render_revision();

        // Remove all source cells first to avoid self-overwrite when
        // moving onto positions belonging to the same selection.
        let mut moved = Vec::new();

        for &src in source_coords {
            if let Some(cell) = self.cells.remove(&src) {
                self.id_to_coord.remove(&cell.id);
                self.mark_physics_dirty(cell.id);
                self.mark_render_dirty(src, DirtyReason::Geometry);

                self.dirty_authored_chunks
                    .insert(ChunkCoord::from_world_coord(src));

                self.update_spatial_index_at(src);

                moved.push((src, cell));
            }
        }

        // Reinsert cells at their destination coordinates.
        for (old_coord, cell) in moved {
            let dest = WorldCoord::new(
                old_coord.x + delta.x,
                old_coord.y + delta.y,
                old_coord.z + delta.z,
            );

            let id = cell.id;

            self.cells.insert(dest, cell);
            self.id_to_coord.insert(id, dest);
            self.mark_physics_dirty(id);
            self.mark_render_dirty(dest, DirtyReason::Geometry);

            self.dirty_authored_chunks
                .insert(ChunkCoord::from_world_coord(dest));

            self.update_spatial_index_at(dest);
        }

        Ok(())
    }

    /// Pastes a group of copied cells from the clipboard at the given target pivot.
    ///
    /// Generates new unique IDs for every pasted cell and remaps script bindings.
    pub fn paste_cells(
        &mut self,
        clipboard_cells: &[crate::editor::ClipboardCell],
        target_pivot: WorldCoord,
    ) -> Result<Vec<WorldCoord>, String> {
        if clipboard_cells.is_empty() {
            return Ok(Vec::new());
        }

        // Validate that no destination coordinate is occupied.
        for item in clipboard_cells {
            let dest = WorldCoord::new(
                target_pivot.x + item.offset.x,
                target_pivot.y + item.offset.y,
                target_pivot.z + item.offset.z,
            );

            if self.cells.contains_key(&dest) {
                return Err("Destination is occupied".to_string());
            }
        }

        self.bump_render_revision();

        let mut pasted_coords = Vec::with_capacity(clipboard_cells.len());

        for item in clipboard_cells {
            let dest = WorldCoord::new(
                target_pivot.x + item.offset.x,
                target_pivot.y + item.offset.y,
                target_pivot.z + item.offset.z,
            );

            let mut cell = item.cell.clone();
            let cell_type = cell.cell_type;

            let new_id = self.generate_unique_id(dest, cell_type);
            cell.id = new_id;

            self.cells.insert(dest, cell);
            self.id_to_coord.insert(new_id, dest);
            self.mark_physics_dirty(new_id);
            self.mark_render_dirty(dest, DirtyReason::Geometry);

            self.index_special_cell(new_id, cell_type);

            self.dirty_authored_chunks
                .insert(ChunkCoord::from_world_coord(dest));

            if let Some(binding) = &item.script_binding {
                let mut new_binding = binding.clone();
                new_binding.target_identity = new_id;
                self.script_bindings.push(new_binding);
            }

            self.update_spatial_index_at(dest);

            pasted_coords.push(dest);
        }

        Ok(pasted_coords)
    }
}
