use super::cell::{Cell, CellType, RuntimeCellState};
use super::coordinate::WorldCoord;
use glam::Vec3;
use std::collections::HashMap;
use crate::scripting::binding::ScriptBinding;

#[derive(Clone)]
pub struct LightingSettings {
    pub shadows_enabled: bool,
    pub global_light_enabled: bool,
    /// The direction light travels through the scene.
    pub global_light_direction: Vec3,
    pub global_light_color: Vec3,
    pub global_light_intensity: f32,
    pub ambient_intensity: f32,
}

impl Default for LightingSettings {
    fn default() -> Self {
        Self {
            shadows_enabled: true,
            global_light_enabled: true,
            // Default downward diagonal.
            global_light_direction: Vec3::new(0.5, -1.0, 0.5).normalize(),
            global_light_color: Vec3::ONE,
            global_light_intensity: 1.0,
            ambient_intensity: 0.20,
        }
    }
}

#[derive(Clone)]
pub struct World {
    // Authored grid data. We use a HashMap because the world is unbounded
    // and most coordinates are empty.
    pub(crate) cells: HashMap<WorldCoord, Cell>,

    // Temporary runtime-only overrides for cell state, keyed by Cell ID.
    pub(crate) runtime_state: HashMap<u64, RuntimeCellState>,

    // Tracks which cells have had physics-relevant properties changed at runtime.
    pub(crate) physics_dirty_cells: std::collections::HashSet<u64>,

    // Optimized lookup for cell coordinates by ID.
    pub(crate) id_to_coord: HashMap<u64, WorldCoord>,

    // The world wide gravity vector used by the physics simulation.
    pub gravity: Vec3,

    // Authoritative scene lighting settings.
    pub lighting: LightingSettings,

    /// Authored script bindings for entities in this world.
    pub script_bindings: Vec<ScriptBinding>,
}

impl World {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            runtime_state: HashMap::new(),
            physics_dirty_cells: std::collections::HashSet::new(),
            id_to_coord: HashMap::new(),
            // We default to Earth standard gravity.
            gravity: Vec3::new(0.0, -9.81, 0.0),
            lighting: LightingSettings::default(),
            script_bindings: Vec::new(),
        }
    }

    pub fn get(&self, coord: WorldCoord) -> Option<&Cell> {
        self.cells.get(&coord)
    }

    pub fn get_mut(&mut self, coord: WorldCoord) -> Option<&mut Cell> {
        self.cells.get_mut(&coord)
    }

    /// Finds a cell's current coordinate by its unique ID.
    /// This is an O(1) lookup using the runtime index.
    pub fn resolve_cell_id(&self, id: u64) -> Option<WorldCoord> {
        self.id_to_coord.get(&id).cloned()
    }

    /// Marks a cell as needing physics reconciliation.
    pub fn mark_physics_dirty(&mut self, cell_id: u64) {
        self.physics_dirty_cells.insert(cell_id);
    }

    /// Returns the effective value of light_enabled for a coordinate,
    /// accounting for any runtime overrides.
    pub fn is_light_enabled(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.cells.get(&coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(enabled) = rs.light_enabled {
                    return enabled;
                }
            }
            return cell.light_enabled;
        }
        false
    }

    /// Sets a runtime-only override for light_enabled.
    /// This does not modify the authored Cell data.
    pub fn set_light_enabled_runtime(&mut self, coord: WorldCoord, enabled: bool) {
        if let Some(cell) = self.cells.get(&coord) {
            self.runtime_state
                .entry(cell.id)
                .or_default()
                .light_enabled = Some(enabled);
        }
    }

    /// Returns the effective value of visible for a coordinate,
    /// accounting for any runtime overrides.
    pub fn is_cell_visible(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.cells.get(&coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(visible) = rs.visible {
                    return visible;
                }
            }
            return cell.visible;
        }
        false
    }

    /// Sets a runtime-only override for cell visibility.
    /// This does not modify the authored Cell data.
    pub fn set_cell_visible_runtime(&mut self, coord: WorldCoord, visible: bool) {
        if let Some(cell) = self.cells.get(&coord) {
            self.runtime_state
                .entry(cell.id)
                .or_default()
                .visible = Some(visible);
        }
    }

    /// Returns the effective color of a cell, accounting for runtime overrides.
    pub fn get_effective_color(&self, coord: WorldCoord) -> Vec3 {
        if let Some(cell) = self.cells.get(&coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(color) = rs.color_rgb {
                    return color;
                }
            }
            return cell.color_rgb;
        }
        Vec3::ZERO
    }

    /// Sets a runtime-only override for cell color.
    pub fn set_cell_color_runtime(&mut self, coord: WorldCoord, color: Vec3) {
        if let Some(cell) = self.cells.get(&coord) {
            self.runtime_state
                .entry(cell.id)
                .or_default()
                .color_rgb = Some(color);
        }
    }

    /// Returns whether a cell is effectively solid.
    pub fn is_cell_solid(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.cells.get(&coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(solid) = rs.solid {
                    return solid;
                }
            }
            return cell.solid;
        }
        false
    }

    /// Sets a runtime-only override for cell solidity.
    pub fn set_cell_solid_runtime(&mut self, coord: WorldCoord, solid: bool) {
        if let Some(cell) = self.cells.get(&coord) {
            let id = cell.id;
            self.runtime_state
                .entry(id)
                .or_default()
                .solid = Some(solid);
            self.mark_physics_dirty(id);
        }
    }

    /// Returns whether a cell is effectively anchored.
    pub fn is_cell_anchored(&self, coord: WorldCoord) -> bool {
        if let Some(cell) = self.cells.get(&coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(anchored) = rs.anchored {
                    return anchored;
                }
            }
            return cell.anchored;
        }
        false
    }

    /// Sets a runtime-only override for cell anchored state.
    pub fn set_cell_anchored_runtime(&mut self, coord: WorldCoord, anchored: bool) {
        if let Some(cell) = self.cells.get(&coord) {
            let id = cell.id;
            self.runtime_state
                .entry(id)
                .or_default()
                .anchored = Some(anchored);
            self.mark_physics_dirty(id);
        }
    }

    /// Returns the effective visual offset of a cell.
    pub fn get_visual_offset(&self, coord: WorldCoord) -> Vec3 {
        if let Some(cell) = self.cells.get(&coord) {
            if let Some(rs) = self.runtime_state.get(&cell.id) {
                if let Some(offset) = rs.visual_offset {
                    return offset;
                }
            }
        }
        Vec3::ZERO
    }

    /// Sets a runtime-only visual offset for a cell.
    pub fn set_visual_offset_runtime(&mut self, coord: WorldCoord, offset: Vec3) {
        if let Some(cell) = self.cells.get(&coord) {
            let id = cell.id;
            self.runtime_state
                .entry(id)
                .or_default()
                .visual_offset = Some(offset);
            self.mark_physics_dirty(id);
        }
    }

    /// Discards all runtime state modifications.
    pub fn clear_runtime_state(&mut self) {
        for id in self.runtime_state.keys() {
            self.physics_dirty_cells.insert(*id);
        }
        self.runtime_state.clear();
    }

    pub fn set_cell(&mut self, coord: WorldCoord, cell_type: CellType) -> u64 {
        if cell_type == CellType::Empty {
            if let Some(cell) = self.cells.remove(&coord) {
                self.id_to_coord.remove(&cell.id);
                self.mark_physics_dirty(cell.id);
                self.runtime_state.remove(&cell.id);
                return cell.id;
            }
            0
        } else {
            // Default properties for a new cell.
            let mut cell = match cell_type {
                CellType::Block => Cell::new_block(),
                CellType::Light => Cell::new_light(),
                CellType::SpawnPoint => Cell::new_spawn_point(),
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
            self.cells.insert(coord, cell);
            id
        }
    }

    /// Internal helper to update the ID index when an ID is changed manually (e.g. during loading).
    pub(crate) fn update_id_mapping(&mut self, old_id: u64, new_id: u64, coord: WorldCoord) {
        self.id_to_coord.remove(&old_id);
        self.id_to_coord.insert(new_id, coord);
    }

    /// Rebuilds the ID to coordinate index. Call this if the cells map is replaced (e.g. undo/redo).
    pub fn rebuild_id_mapping(&mut self) {
        self.id_to_coord.clear();
        for (coord, cell) in &self.cells {
            self.id_to_coord.insert(cell.id, *coord);
        }
    }

    pub(crate) fn generate_unique_id(
        &mut self,
        coord: WorldCoord,
        cell_type: CellType,
    ) -> u64 {
        use chrono::Local;
        use std::hash::{Hash, Hasher};

        let mut retry_count = 0;

        loop {
            let mut hasher = std::collections::hash_map::DefaultHasher::new();

            let timestamp = Local::now()
                .format("%Y:%m:%d:%S")
                .to_string();

            timestamp.hash(&mut hasher);
            coord.hash(&mut hasher);
            (cell_type as u32).hash(&mut hasher);
            retry_count.hash(&mut hasher);

            let hash = hasher.finish();

            // Map to 8 digit range: 10,000,000 to 99,999,999.
            let id = 10_000_000 + (hash % 90_000_000);

            // Only currently existing cells participate in collision checks.
            if !self.cells.values().any(|cell| cell.id == id) {
                return id;
            }

            retry_count += 1;
        }
    }

    pub fn active_blocks(&self) -> Vec<WorldCoord> {
        self.cells.keys().cloned().collect()
    }
}