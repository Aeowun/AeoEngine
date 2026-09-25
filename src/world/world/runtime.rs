use super::World;
use super::dirty::DirtyReason;
use crate::world::cell::{AttributeValue, Cell, CellType};
use crate::world::coordinate::WorldCoord;
use glam::Vec3;

impl World {
    /// Sets a runtime-only override for light_enabled.
    ///
    /// This affects the renderer's point-light selection but does not change
    /// voxel geometry, so it must not invalidate the chunk render cache.
    pub fn set_light_enabled_runtime(&mut self, coord: WorldCoord, enabled: bool) {
        let Some(id) = self.get_effective_cell(coord).map(|cell| cell.id) else {
            return;
        };

        let runtime_state = self.runtime_state.entry(id).or_default();

        if runtime_state.light_enabled == Some(enabled) {
            return;
        }

        runtime_state.light_enabled = Some(enabled);
    }

    /// Sets a runtime-only override for cell visibility.
    /// This does not modify the authored Cell data.
    pub fn set_cell_visible_runtime(&mut self, coord: WorldCoord, visible: bool) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);

        let id = self.get_effective_cell(coord).map(|cell| cell.id);

        if let Some(id) = id {
            let runtime_state = self.runtime_state.entry(id).or_default();

            if runtime_state.visible != Some(visible) {
                runtime_state.visible = Some(visible);
            }
        }

        self.update_spatial_index_at(coord);
    }

    // --- Audio Runtime Overrides ---

    pub fn set_audio_playing_runtime(&mut self, cell_id: u64, playing: bool) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();

            rs.audio_playing = Some(playing);

            if playing {
                rs.audio_paused = Some(false);
            }
        }
    }

    pub fn set_audio_paused_runtime(&mut self, cell_id: u64, paused: bool) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();

            rs.audio_paused = Some(paused);
        }
    }

    pub fn set_audio_looped_runtime(&mut self, cell_id: u64, looped: bool) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();

            rs.audio_looped = Some(looped);
        }
    }

    pub fn set_audio_volume_runtime(&mut self, cell_id: u64, volume: f32) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();

            rs.audio_volume = Some(volume);
        }
    }

    pub fn audio_play_runtime(&mut self, cell_id: u64) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            println!("[AUDIO] play requested for emitter {}", cell_id);

            let rs = self.runtime_state.entry(cell_id).or_default();

            rs.audio_playing = Some(true);
            rs.audio_paused = Some(false);
        }
    }

    pub fn audio_stop_runtime(&mut self, cell_id: u64) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();

            rs.audio_playing = Some(false);
            rs.audio_paused = Some(false);
        }
    }

    pub fn audio_pause_runtime(&mut self, cell_id: u64) {
        if self.get_effective_cell_by_id(cell_id).is_some() {
            let rs = self.runtime_state.entry(cell_id).or_default();

            rs.audio_paused = Some(true);
        }
    }

    /// Sets a runtime-only override for cell color.
    pub fn set_cell_color_runtime(&mut self, coord: WorldCoord, color: Vec3) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::MaterialOrOffset);

        let id = self.get_effective_cell(coord).map(|cell| cell.id);

        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().color_rgb = Some(color);
        }
    }

    /// Sets a runtime-only override for cell solidity.
    pub fn set_cell_solid_runtime(&mut self, coord: WorldCoord, solid: bool) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);

        let id = self.get_effective_cell(coord).map(|cell| cell.id);

        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().solid = Some(solid);

            self.mark_physics_dirty(id);
        }

        self.update_spatial_index_at(coord);
    }

    /// Sets a runtime-only override for cell anchored state.
    pub fn set_cell_anchored_runtime(&mut self, coord: WorldCoord, anchored: bool) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::Geometry);

        let id = self.get_effective_cell(coord).map(|cell| cell.id);

        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().anchored = Some(anchored);

            self.mark_physics_dirty(id);
        }
    }

    /// Sets a runtime-only visual offset for a cell.
    pub fn set_visual_offset_runtime(&mut self, coord: WorldCoord, offset: Vec3) {
        self.bump_render_revision();
        self.mark_render_dirty(coord, DirtyReason::MaterialOrOffset);

        let id = self.get_effective_cell(coord).map(|cell| cell.id);

        if let Some(id) = id {
            self.runtime_state.entry(id).or_default().visual_offset = Some(offset);

            self.mark_physics_dirty(id);
        }
    }

    pub fn create_runtime_cell(&mut self, cell_type: CellType) -> u64 {
        self.bump_render_revision();

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

        let id = self.generate_runtime_unique_id();

        cell.id = id;

        if cell.cell_type == CellType::Light {
            self.light_ids.insert(id);
            self.editor_marker_ids.insert(id);
        } else if cell.cell_type == CellType::AudioEmitter {
            self.audio_emitter_ids.insert(id);
            self.editor_marker_ids.insert(id);
        }

        self.runtime_cells.insert(id, cell);
        self.mark_physics_dirty(id);

        id
    }

    pub fn move_runtime_cell(&mut self, id: u64, new_coord: WorldCoord) -> Result<(), String> {
        if !self.runtime_cells.contains_key(&id) {
            return Err("Not a runtime cell".to_string());
        }

        self.bump_render_revision();

        let old_coord_opt = self.runtime_id_to_coord.remove(&id);

        if let Some(old_coord) = old_coord_opt {
            self.coord_to_runtime_id.remove(&old_coord);
            self.mark_physics_dirty(id);
            self.mark_render_dirty(old_coord, DirtyReason::Geometry);
            self.update_spatial_index_at(old_coord);
        }

        self.runtime_id_to_coord.insert(id, new_coord);
        self.coord_to_runtime_id.insert(new_coord, id);

        self.mark_physics_dirty(id);
        self.mark_render_dirty(new_coord, DirtyReason::Geometry);
        self.update_spatial_index_at(new_coord);

        Ok(())
    }

    pub fn delete_cell_runtime(&mut self, id: u64) {
        self.bump_render_revision();

        let coord_opt = self.resolve_cell_id(id);

        if let Some(coord) = coord_opt {
            self.mark_render_dirty(coord, DirtyReason::Geometry);
        }

        self.light_ids.remove(&id);
        self.editor_marker_ids.remove(&id);
        self.audio_emitter_ids.remove(&id);

        if self.runtime_cells.contains_key(&id) {
            if let Some(coord) = self.runtime_id_to_coord.remove(&id) {
                self.coord_to_runtime_id.remove(&coord);
                self.mark_physics_dirty(id);
            }

            self.runtime_cells.remove(&id);
            self.runtime_state.remove(&id);
        } else if self.id_to_coord.contains_key(&id) {
            self.runtime_state.entry(id).or_default().is_deleted = true;

            self.mark_physics_dirty(id);
        }

        if let Some(coord) = coord_opt {
            self.update_spatial_index_at(coord);
        }
    }

    pub(crate) fn generate_runtime_unique_id(&self) -> u64 {
        use chrono::Local;
        use std::hash::{Hash, Hasher};

        let mut retry_count = 0;

        loop {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();

            let timestamp = Local::now().format("%Y:%m:%d:%S:%f").to_string();

            timestamp.hash(&mut hasher);
            retry_count.hash(&mut hasher);

            let hash = hasher.finish();

            // Map to 9 digit range: 100,000,000 to 999,999,999.
            let id = 100_000_000 + (hash % 900_000_000);

            if !self.runtime_cells.contains_key(&id)
                && !self.cells.values().any(|cell| cell.id == id)
            {
                return id;
            }

            retry_count += 1;
        }
    }

    pub fn set_attribute_runtime(&mut self, coord: WorldCoord, key: String, value: AttributeValue) {
        let id = self.get_effective_cell(coord).map(|cell| cell.id);

        if let Some(id) = id {
            self.runtime_state
                .entry(id)
                .or_default()
                .attribute_overrides
                .insert(key, value);
        }
    }

    pub fn remove_attribute_runtime(&mut self, coord: WorldCoord, key: &str) {
        let id = self.get_effective_cell(coord).map(|cell| cell.id);

        if let Some(id) = id {
            if let Some(rs) = self.runtime_state.get_mut(&id) {
                rs.attribute_overrides.remove(key);
            }
        }
    }

    /// Discards all runtime state modifications.
    pub fn clear_runtime_state(&mut self) {
        self.bump_render_revision();

        for id in self.runtime_state.keys() {
            self.physics_dirty_cells.insert(*id);

            if let Some(coord) = self.id_to_coord.get(id) {
                self.mark_render_dirty(*coord, DirtyReason::Geometry);
            }
        }

        for (id, coord) in &self.runtime_id_to_coord {
            self.physics_dirty_cells.insert(*id);
            self.mark_render_dirty(*coord, DirtyReason::Geometry);
        }

        self.runtime_state.clear();
        self.runtime_cells.clear();
        self.runtime_id_to_coord.clear();
        self.coord_to_runtime_id.clear();

        self.rebuild_light_and_marker_indices();
        self.rebuild_spatial_index();
    }
}
