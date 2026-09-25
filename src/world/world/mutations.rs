use crate::world::cell::{Cell, CellType, ChunkCoord};
use crate::world::coordinate::WorldCoord;
use super::dirty::DirtyReason;
use super::World;

impl World {
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
        let id = if cell_type == CellType::Empty {
            if let Some(cell) = self.cells.remove(&coord) {
                self.id_to_coord.remove(&cell.id);
                self.mark_physics_dirty(cell.id);
                self.runtime_state.remove(&cell.id);
                self.audio_emitter_ids.remove(&cell.id);
                cell.id
            } else {
                0
            }
        } else {
            // Default properties for a new cell.
            let mut cell = match cell_type {
                CellType::Block => Cell::new_block(),
                CellType::Light => Cell::new_light(),
                CellType::SpawnPoint => Cell::new_spawn_point(),
                CellType::AudioEmitter => Cell::new_audio_emitter(),
                _ => {
                    let mut c = Cell::default();
                    c.cell_type = cell_type;
                    c
                }
            };

            cell.id = self.generate_unique_id(coord, cell_type);
            let id = cell.id;
            self.id_to_coord.insert(id, coord);
            self.mark_physics_dirty(id);
            if cell_type == CellType::AudioEmitter {
                self.audio_emitter_ids.insert(id);
            }
            self.cells.insert(coord, cell);
            id
        };
        self.update_spatial_index_at(coord);
        id
    }

    /// Internal helper to update the ID index when an ID is changed manually (e.g. during loading).
    pub(crate) fn update_id_mapping(
        &mut self,
        old_id: u64,
        new_id: u64,
        coord: WorldCoord,
    ) {
        self.id_to_coord.remove(&old_id);
        self.id_to_coord.insert(new_id, coord);
        self.observe_authored_id(new_id);
    }

    /// Rebuilds the ID to coordinate index. Call this if the cells map is replaced (e.g. undo/redo).
    pub fn rebuild_id_mapping(&mut self) {
        self.bump_render_revision();
        for &coord in self.cells.keys() {
            self.mark_render_dirty(coord, DirtyReason::Geometry);
        }
        self.id_to_coord.clear();
        for (coord, cell) in &self.cells {
            self.id_to_coord.insert(cell.id, *coord);
        }
        self.rebuild_spatial_index();
        self.rebuild_audio_emitter_index();
        self.dirty_authored_chunks = self.cells.keys()
            .map(|coord| ChunkCoord::from_world_coord(*coord))
            .collect();
    }

    pub(crate) fn rebuild_audio_emitter_index(&mut self) {
        self.audio_emitter_ids.clear();
        self.audio_emitter_ids.extend(
            self.cells.values().chain(self.runtime_cells.values())
                .filter(|cell| cell.cell_type == CellType::AudioEmitter)
                .map(|cell| cell.id),
        );
    }

    pub(crate) fn generate_unique_id(
        &mut self,
        _coord: WorldCoord,
        _cell_type: CellType,
    ) -> u64 {
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
            self.next_cell_id = id
                .checked_add(1)
                .expect("Authored cell ID space exhausted");
        }
    }

    /// Moves a collection of authored cells by a given delta offset, preserving their cell IDs.
    /// Returns Err if the movement destination collides with unrelated cells in the world.
    pub fn move_cells(
        &mut self,
        source_coords: &[WorldCoord],
        delta: WorldCoord,
    ) -> Result<(), String> {
        if delta == WorldCoord::new(0, 0, 0) || source_coords.is_empty() {
            return Ok(());
        }

        let source_set: std::collections::HashSet<WorldCoord> =
            source_coords.iter().cloned().collect();

        // Validate that no destination coordinate collides with an unrelated cell.
        for &src in source_coords {
            let dest = WorldCoord::new(src.x + delta.x, src.y + delta.y, src.z + delta.z);
            if self.cells.contains_key(&dest) && !source_set.contains(&dest) {
                return Err("Destination is occupied by an unrelated cell".to_string());
            }
        }

        self.bump_render_revision();

        // 1. Remove all source cells first to avoid self-overwrite when moving onto own positions.
        let mut moved = Vec::new();
        for &src in source_coords {
            if let Some(cell) = self.cells.remove(&src) {
                self.id_to_coord.remove(&cell.id);
                self.mark_physics_dirty(cell.id);
                self.mark_render_dirty(src, DirtyReason::Geometry);
                moved.push((src, cell));
            }
        }

        // 2. Re-insert cells at destination coordinates.
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
        }

        Ok(())
    }

    /// Pastes a group of copied cells from clipboard at the given target pivot coordinate.
    /// Generates new unique IDs for every pasted cell and remaps script bindings.
    /// Returns Err if any destination coordinate is occupied by an existing cell.
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
        let mut pasted_coords = Vec::new();

        for item in clipboard_cells {
            let dest = WorldCoord::new(
                target_pivot.x + item.offset.x,
                target_pivot.y + item.offset.y,
                target_pivot.z + item.offset.z,
            );

            let mut cell = item.cell.clone();
            let new_id = self.generate_unique_id(dest, cell.cell_type);
            cell.id = new_id;

            self.cells.insert(dest, cell);
            self.id_to_coord.insert(new_id, dest);
            self.mark_physics_dirty(new_id);
            self.mark_render_dirty(dest, DirtyReason::Geometry);

            if let Some(ref binding) = item.script_binding {
                let mut new_binding = binding.clone();
                new_binding.target_identity = new_id;
                self.script_bindings.push(new_binding);
            }

            pasted_coords.push(dest);
        }

        Ok(pasted_coords)
    }
}
