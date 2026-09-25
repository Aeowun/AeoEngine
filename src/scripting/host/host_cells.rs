use crate::scripting::value::{HandleKind, Value};
use crate::world::cell::AttributeValue;
use crate::world::{CellType, WorldCoord};

use super::ScriptHostBridge;

impl<'a> ScriptHostBridge<'a> {
    pub fn lookup_light(&self, x: i32, y: i32, z: i32) -> Option<u64> {
        let coord = WorldCoord::new(x, y, z);

        if let Some(cell) = self.world.get(coord) {
            if cell.cell_type == CellType::Light {
                return Some(cell.id);
            }
        }

        None
    }

    pub fn is_light_enabled(&self, id: u64) -> Option<bool> {
        self.world
            .resolve_cell_id(id)
            .map(|coord| self.world.is_light_enabled(coord))
    }

    pub fn set_light_enabled(&mut self, id: u64, enabled: bool) {
        if let Some(coord) = self.world.resolve_cell_id(id) {
            self.world.set_light_enabled_runtime(coord, enabled);
        }
    }

    pub fn is_collision_events_enabled(&self, id: u64) -> bool {
        if let Some(coord) = self.world.resolve_cell_id(id) {
            if let Some(runtime_state) = self.world.runtime_state.get(&id) {
                if let Some(enabled) = runtime_state.collision_events_enabled {
                    return enabled;
                }
            }

            if let Some(cell) = self.world.get_effective_cell(coord) {
                return cell.collision_events_enabled;
            }
        }

        true
    }

    pub fn create_runtime_cell(&mut self, cell_type: &str) -> Result<(HandleKind, u64), String> {
        let cell_type = match cell_type {
            "Block" => CellType::Block,
            "FxBlock" => CellType::FxBlock,
            "Player" => CellType::Player,
            "NPC" => CellType::NPC,
            "Light" => CellType::Light,
            "SpawnPoint" => CellType::SpawnPoint,
            "AudioEmitter" => CellType::AudioEmitter,
            "Empty" => {
                return Err("Cannot create Empty cell".to_string());
            }
            _ => {
                return Err(format!("Unknown cell type: {}", cell_type));
            }
        };

        let id = self.world.create_runtime_cell(cell_type);

        let kind = if cell_type == CellType::Light {
            HandleKind::Light
        } else {
            HandleKind::Cell
        };

        Ok((kind, id))
    }

    pub fn move_runtime_cell(&mut self, id: u64, x: i32, y: i32, z: i32) -> Result<(), String> {
        let coord = WorldCoord::new(x, y, z);
        self.world.move_runtime_cell(id, coord)
    }

    pub fn delete_cell(&mut self, id: u64) -> Result<(), String> {
        self.world.delete_cell_runtime(id);
        Ok(())
    }

    pub fn get_all_cells_of_class(&self, class_name: &str) -> Vec<u64> {
        let mut results = Vec::new();

        for coord in self.world.iter_active_effective_coords() {
            if let Some(cell) = self.world.get_effective_cell(coord) {
                let matches = match class_name {
                    "Light" => cell.cell_type == CellType::Light,
                    "AudioEmitter" => cell.cell_type == CellType::AudioEmitter,
                    "Block" => cell.cell_type == CellType::Block,
                    "FxBlock" => cell.cell_type == CellType::FxBlock,
                    "SpawnPoint" => cell.cell_type == CellType::SpawnPoint,
                    "Player" => cell.cell_type == CellType::Player,
                    "NPC" => cell.cell_type == CellType::NPC,
                    _ => false,
                };

                if matches {
                    results.push(cell.id);
                }
            }
        }

        results
    }

    pub fn set_attribute(&mut self, id: u64, key: String, value: Value) -> Result<(), String> {
        if self.world.get_effective_cell_by_id(id).is_some() {
            let attribute = match value {
                Value::Number(n) => AttributeValue::Number(n),
                Value::Bool(b) => AttributeValue::Bool(b),
                Value::String(s) => AttributeValue::String(s),

                _ => {
                    return Err(format!(
                        "Cell attributes only support Number, Bool, or String. Got {}",
                        value.type_name()
                    ));
                }
            };

            self.world
                .runtime_state
                .entry(id)
                .or_default()
                .attribute_overrides
                .insert(key, attribute);

            Ok(())
        } else {
            Err("invalid cell handle for attribute assignment".to_string())
        }
    }

    pub fn remove_attribute(&mut self, id: u64, key: &str) -> Result<(), String> {
        if self.world.get_effective_cell_by_id(id).is_some() {
            if let Some(runtime_state) = self.world.runtime_state.get_mut(&id) {
                runtime_state.attribute_overrides.remove(key);
            }

            Ok(())
        } else {
            Err("invalid cell handle for attribute removal".to_string())
        }
    }

    pub fn cell_exists(&self, id: u64) -> bool {
        self.world.get_effective_cell_by_id(id).is_some()
    }
}
