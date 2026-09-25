use std::collections::BTreeMap;
use crate::world::cell::{AttributeValue, Cell, ChunkCoord};
use crate::world::coordinate::WorldCoord;
use super::World;
use glam::Vec3;

impl World {
    pub fn get(&self, coord: WorldCoord) -> Option<&Cell> {
        self.cells.get(&coord)
    }

    pub fn resident_authored_cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Finds a cell's current coordinate by its unique ID.
    /// This is an O(1) lookup using the runtime index.
    pub fn resolve_cell_id(&self, id: u64) -> Option<WorldCoord> {
        if let Some(coord) = self.id_to_coord.get(&id) {
            if let Some(rs) = self.runtime_state.get(&id) {
                if rs.is_deleted {
                    return None;
                }
            }
            return Some(*coord);
        }
        self.runtime_id_to_coord.get(&id).cloned()
    }

    /// Returns the effective value of light_enabled for a coordinate,
    /// accounting for any runtime overrides.
    pub fn is_light_enabled(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(enabled) = rs.light_enabled {
                    return enabled;
                }
            }
            return cell.light_enabled;
        }
        false
    }

    /// Returns the effective value of visible for a coordinate,
    /// accounting for any runtime overrides.
    pub fn is_cell_visible(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(visible) = rs.visible {
                    return visible;
                }
            }
            return cell.visible;
        }
        false
    }

    // --- Audio Runtime Queries ---

    pub fn is_audio_playing(&self, cell_id: u64) -> bool {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(playing) = rs.audio_playing {
                    return playing;
                }
            }
            return cell.playing;
        }
        false
    }

    pub fn is_audio_paused(&self, cell_id: u64) -> bool {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(paused) = rs.audio_paused {
                    return paused;
                }
            }
        }
        false
    }

    pub fn is_audio_looped(&self, cell_id: u64) -> bool {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(looped) = rs.audio_looped {
                    return looped;
                }
            }
            return cell.looped;
        }
        false
    }

    pub fn get_audio_volume(&self, cell_id: u64) -> f32 {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(vol) = rs.audio_volume {
                    return vol;
                }
            }
            return cell.volume;
        }
        1.0
    }

    pub fn get_audio_path(&self, cell_id: u64) -> String {
        if let Some(cell) = self.get_effective_cell_by_id(cell_id) {
            return cell.audio.clone();
        }
        String::new()
    }

    /// Returns the effective color of a cell, accounting for runtime overrides.
    pub fn get_effective_color(&self, coord: WorldCoord) -> Vec3 {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(color) = rs.color_rgb {
                    return color;
                }
            }
            return cell.color_rgb;
        }
        Vec3::ZERO
    }

    /// Returns whether a cell is effectively solid.
    pub fn is_cell_solid(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(solid) = rs.solid {
                    return solid;
                }
            }
            return cell.solid;
        }
        false
    }

    /// Returns whether a cell is effectively anchored.
    pub fn is_cell_anchored(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(anchored) = rs.anchored {
                    return anchored;
                }
            }
            return cell.anchored;
        }
        false
    }

    /// Returns the effective visual offset of a cell.
    pub fn get_visual_offset(&self, coord: WorldCoord) -> Vec3 {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(offset) = rs.visual_offset {
                    return offset;
                }
            }
        }
        Vec3::ZERO
    }

    pub fn get_effective_attributes(
        &self,
        coord: WorldCoord,
    ) -> BTreeMap<String, AttributeValue> {
        let mut result = BTreeMap::new();
        if let Some(cell) = self.cells.get(&coord) {
            result.extend(cell.attributes.clone());
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                result.extend(rs.attribute_overrides.clone());
            }
        }
        result
    }

    pub fn get_effective_attribute(
        &self,
        coord: WorldCoord,
        key: &str,
    ) -> Option<AttributeValue> {
        if let Some(cell) = self.get_effective_cell(coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(val) = rs.attribute_overrides.get(key) {
                    return Some(val.clone());
                }
            }
            return cell.attributes.get(key).cloned();
        }
        None
    }

    pub fn get_effective_cell(&self, coord: WorldCoord) -> Option<&Cell> {
        if let Some(id) = self.coord_to_runtime_id.get(&coord) {
            return self.runtime_cells.get(id);
        }
        if let Some(cell) = self.cells.get(&coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if rs.is_deleted {
                    return None;
                }
            }
            return Some(cell);
        }
        None
    }

    pub fn get_effective_cell_by_id(&self, id: u64) -> Option<&Cell> {
        if let Some(cell) = self.runtime_cells.get(&id) {
            return Some(cell);
        }
        if let Some(coord) = self.id_to_coord.get(&id) {
            if let Some(rs) = self.runtime_state.get(&id) {
                if rs.is_deleted {
                    return None;
                }
            }
            return self.cells.get(coord);
        }
        None
    }

    pub fn active_effective_blocks(&self) -> Vec<WorldCoord> {
        let mut coords: std::collections::HashSet<WorldCoord> = std::collections::HashSet::new();
        for (coord, cell) in &self.cells {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if rs.is_deleted {
                    continue;
                }
            }
            coords.insert(*coord);
        }
        coords.extend(self.runtime_id_to_coord.values().cloned());
        coords.into_iter().collect()
    }

    /// Resident-only effective coordinates without allocating a temporary
    /// whole-world vector. Runtime coordinates follow authored coordinates.
    pub fn iter_active_effective_coords(&self) -> impl Iterator<Item = WorldCoord> + '_ {
        self.cells.iter().filter_map(|(coord, cell)| {
            (!self.runtime_state.get(&cell.id).is_some_and(|state| state.is_deleted))
                .then_some(*coord)
        }).chain(self.runtime_id_to_coord.values().copied())
    }

    pub fn iter_audio_emitter_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.audio_emitter_ids.iter().copied()
    }

    pub fn active_blocks(&self) -> Vec<WorldCoord> {
        self.cells.keys().cloned().collect()
    }

    pub fn authored_cells_in_chunk(&self, chunk_coord: ChunkCoord) -> Vec<(WorldCoord, Cell)> {
        self.cells.iter()
            .filter(|(coord, _)| ChunkCoord::from_world_coord(**coord) == chunk_coord)
            .map(|(coord, cell)| (*coord, cell.clone()))
            .collect()
    }
}
