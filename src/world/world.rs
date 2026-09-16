use std::collections::HashMap;
use glam::Vec3;
use super::cell::{Cell, CellType};
use super::coordinate::WorldCoord;

pub struct World {
    // Authored grid data. We use a HashMap because the world is unbounded
    // and most coordinates are empty.
    cells: HashMap<WorldCoord, Cell>,

    // The world wide gravity vector used by the physics simulation.
    pub gravity: Vec3,
}

impl World {
    pub fn new() -> Self {
        Self {
            cells: HashMap::new(),
            // We default to Earth standard gravity.
            gravity: Vec3::new(0.0, -9.81, 0.0),
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
            let mut cell = Cell::default();
            cell.cell_type = cell_type;
            if cell_type == CellType::Grass {
                cell = Cell::new_grass();
            }
            self.cells.insert(coord, cell);
        }
    }

    pub fn active_blocks(&self) -> Vec<WorldCoord> {
        self.cells.keys().cloned().collect()
    }
}
