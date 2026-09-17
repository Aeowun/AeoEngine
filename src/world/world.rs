use std::collections::HashMap;
use glam::Vec3;
use super::cell::{Cell, CellType};
use super::coordinate::WorldCoord;

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

    // The world wide gravity vector used by the physics simulation.
    pub gravity: Vec3,

    // Authoritative scene lighting settings.
    pub lighting: LightingSettings,
}

impl World {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            // We default to Earth standard gravity.
            gravity: Vec3::new(0.0, -9.81, 0.0),
            lighting: LightingSettings::default(),
        }
    }

    pub fn get(&self, coord: WorldCoord) -> Option<&Cell> {
        self.cells.get(&coord)
    }

    pub fn get_mut(&mut self, coord: WorldCoord) -> Option<&mut Cell> {
        self.cells.get_mut(&coord)
    }

    pub fn set_cell(&mut self, coord: WorldCoord, cell_type: CellType) {
        if cell_type == CellType::Empty {
            self.cells.remove(&coord);
        } else {
            // Default properties for a new cell
            let cell = match cell_type {
                CellType::Block => Cell::new_block(),
                CellType::Light => Cell::new_light(),
                CellType::SpawnPoint => Cell::new_spawn_point(),
                _ => {
                    let mut c = Cell::default();
                    c.cell_type = cell_type;
                    c
                }
            };
            self.cells.insert(coord, cell);
        }
    }

    pub fn active_blocks(&self) -> Vec<WorldCoord> {
        self.cells.keys().cloned().collect()
    }
}
