use std::collections::BTreeMap;
use glam::Vec3;
use crate::engine::entity::{EntityManager, EntityId};
use crate::world::{World, WorldCoord, CellType};
use crate::world::cell::AttributeValue;
use crate::scripting::api::EngineHost;
use crate::scripting::value::{Value, HandleKind, MapKey};

pub struct ScriptHostBridge<'a> {
    pub entity_manager: &'a mut EntityManager,
    pub world: &'a mut World,
}

impl<'a> EngineHost for ScriptHostBridge<'a> {
    fn entity_manager(&self) -> &EntityManager {
        self.entity_manager
    }

    fn get_position(&self, id: u64) -> Option<Vec3> {
        self.entity_manager.get_position(EntityId(id))
    }

    fn set_position(&mut self, id: u64, position: Vec3) {
        self.entity_manager.set_position(EntityId(id), position);
    }

    fn lookup_light(&self, x: i32, y: i32, z: i32) -> Option<u64> {
        let coord = WorldCoord::new(x, y, z);
        if let Some(cell) = self.world.get(coord) {
            if cell.cell_type == CellType::Light {
                return Some(cell.id);
            }
        }
        None
    }

    fn is_light_enabled(&self, id: u64) -> Option<bool> {
        if let Some(coord) = self.world.resolve_cell_id(id) {
            Some(self.world.is_light_enabled(coord))
        } else {
            None
        }
    }

    fn set_light_enabled(&mut self, id: u64, enabled: bool) {
        if let Some(coord) = self.world.resolve_cell_id(id) {
            self.world.set_light_enabled_runtime(coord, enabled);
        }
    }

    fn get_all_cells_of_class(&self, class_name: &str) -> Vec<u64> {
        let mut results = Vec::new();
        for (_coord, cell) in &self.world.cells {
            let matches = match class_name {
                "Light" => cell.cell_type == CellType::Light,
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
        results
    }

    fn find_objects(&self, query: &str) -> Vec<(HandleKind, u64)> {
        let mut results = Vec::new();
        if let Some(id) = self.entity_manager.lookup_entity(query) {
            results.push((HandleKind::Entity, id.0));
        }

        // Search for cells with matching entity_identity
        for coord in self.world.active_blocks() {
            if let Some(cell) = self.world.get(coord) {
                if let Some(identity) = &cell.entity_identity {
                    if identity == query {
                        // All Cells are identified by their unique ID in the scripting system.
                        // We use HandleKind::Cell or HandleKind::Light etc. based on cell type.
                        let kind = match cell.cell_type {
                            CellType::Light => HandleKind::Light,
                            _ => HandleKind::Cell,
                        };
                        results.push((kind, cell.id));
                    }
                }
            }
        }

        results
    }

    fn get_children(&self, _kind: HandleKind, _id: u64) -> Vec<(HandleKind, u64)> {
        Vec::new()
    }

    fn get_parent(&self, _kind: HandleKind, _id: u64) -> Option<(HandleKind, u64)> {
        None
    }

    fn get_cell_object(&self, cell_id: u64) -> Option<(HandleKind, u64)> {
        if let Some(coord) = self.world.resolve_cell_id(cell_id) {
            if let Some(cell) = self.world.get(coord) {
                if let Some(identity) = &cell.entity_identity {
                    if let Some(id) = self.entity_manager.lookup_entity(identity) {
                        return Some((HandleKind::Entity, id.0));
                    }
                }
            }
        }
        None
    }

    fn get_property(&self, kind: HandleKind, id: u64, name: &str) -> Result<Option<Value>, String> {
        match kind {
            HandleKind::Cell | HandleKind::Light => {
                if let Some(coord) = self.world.resolve_cell_id(id) {
                    if let Some(cell) = self.world.get(coord) {
                        match name {
                        "id" => return Ok(Some(Value::Number(cell.id as f64))),
                        "name" => return Ok(Some(Value::String(cell.entity_identity.clone().unwrap_or_else(|| "Cell".to_string())))),
                        "cellType" => return Ok(Some(Value::String(format!("{:?}", cell.cell_type)))),
                        "position" => return Ok(Some(Value::array(vec![
                            Value::Number(coord.x as f64),
                            Value::Number(coord.y as f64),
                            Value::Number(coord.z as f64),
                        ]))),
                        "visible" => return Ok(Some(Value::Bool(self.world.is_cell_visible(coord)))),
                        "enabled" => return Ok(Some(Value::Bool(self.world.is_light_enabled(coord)))),
                        "solid" => return Ok(Some(Value::Bool(self.world.is_cell_solid(coord)))),
                        "anchored" => return Ok(Some(Value::Bool(self.world.is_cell_anchored(coord)))),
                        "color" => {
                            let color = self.world.get_effective_color(coord);
                            return Ok(Some(Value::array(vec![
                                Value::Number(color.x as f64),
                                Value::Number(color.y as f64),
                                Value::Number(color.z as f64),
                            ])));
                        }
                        "offset" => {
                            let offset = self.world.get_visual_offset(coord);
                            return Ok(Some(Value::array(vec![
                                Value::Number(offset.x as f64),
                                Value::Number(offset.y as f64),
                                Value::Number(offset.z as f64),
                            ])));
                        }
                        "attributes" => {
                            let mut map = BTreeMap::new();
                            for (key, attr) in self.world.get_effective_attributes(coord) {
                                let val = match attr {
                                    AttributeValue::Number(n) => Value::Number(n),
                                    AttributeValue::Bool(b) => Value::Bool(b),
                                    AttributeValue::String(s) => Value::String(s),
                                };
                                map.insert(MapKey::String(key), val);
                            }
                            return Ok(Some(Value::map(map)));
                        }
                        _ => {}
                    }
                    }
                }
            }
            HandleKind::Entity => {
                let entity_id = EntityId(id);
                match name {
                    "name" => {
                        if let Some(entity_name) = self.entity_manager.get_name(entity_id) {
                            return Ok(Some(Value::String(entity_name.to_string())));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        Ok(None)
    }

    fn set_property(&mut self, kind: HandleKind, id: u64, name: &str, value: Value) -> Result<(), String> {
        match kind {
            HandleKind::Cell | HandleKind::Light => {
                if let Some(coord) = self.world.resolve_cell_id(id) {
                    if self.world.get(coord).is_some() {
                        match name {
                        "visible" => {
                            let visible = value.as_bool()?;
                            self.world.set_cell_visible_runtime(coord, visible);
                            return Ok(());
                        }
                        "enabled" => {
                            let enabled = value.as_bool()?;
                            self.world.set_light_enabled_runtime(coord, enabled);
                            return Ok(());
                        }
                        "solid" => {
                            let solid = value.as_bool()?;
                            self.world.set_cell_solid_runtime(coord, solid);
                            return Ok(());
                        }
                        "anchored" => {
                            let anchored = value.as_bool()?;
                            self.world.set_cell_anchored_runtime(coord, anchored);
                            return Ok(());
                        }
                        "color" => {
                            let basket = value.as_basket()?;
                            let borrowed = basket.borrow();
                            if borrowed.elements.len() != 3 {
                                return Err("color must be a basket of 3 numbers [r, g, b]".to_string());
                            }
                            let r = borrowed.elements[0].as_number()? as f32;
                            let g = borrowed.elements[1].as_number()? as f32;
                            let b = borrowed.elements[2].as_number()? as f32;
                            self.world.set_cell_color_runtime(coord, Vec3::new(r, g, b));
                            return Ok(());
                        }
                        "offset" => {
                            let basket = value.as_basket()?;
                            let borrowed = basket.borrow();
                            if borrowed.elements.len() != 3 {
                                return Err("offset must be a basket of 3 numbers [x, y, z]".to_string());
                            }
                            let x = borrowed.elements[0].as_number()? as f32;
                            let y = borrowed.elements[1].as_number()? as f32;
                            let z = borrowed.elements[2].as_number()? as f32;
                            self.world.set_visual_offset_runtime(coord, Vec3::new(x, y, z));
                            return Ok(());
                        }
                        _ => {}
                    }
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn call_method(&mut self, _kind: HandleKind, _id: u64, _name: &str, _args: &[Value]) -> Result<Option<Value>, String> {
        Ok(None)
    }

    fn set_attribute(&mut self, id: u64, key: String, value: Value) -> Result<(), String> {
        if let Some(coord) = self.world.resolve_cell_id(id) {
            let attr_val = match value {
                Value::Number(n) => AttributeValue::Number(n),
                Value::Bool(b) => AttributeValue::Bool(b),
                Value::String(s) => AttributeValue::String(s),
                _ => return Err(format!("Cell attributes only support Number, Bool, or String. Got {}", value.type_name())),
            };
            self.world.set_attribute_runtime(coord, key, attr_val);
            Ok(())
        } else {
            Err("invalid cell handle for attribute assignment".to_string())
        }
    }

    fn remove_attribute(&mut self, id: u64, key: &str) -> Result<(), String> {
        if let Some(coord) = self.world.resolve_cell_id(id) {
            self.world.remove_attribute_runtime(coord, key);
            Ok(())
        } else {
            Err("invalid cell handle for attribute removal".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::cell::{AttributeValue, CellType};
    use crate::world::WorldCoord;
    use crate::scripting::value::{HandleKind, Value};
    use crate::scripting::api::EngineHost;

    #[test]
    fn test_script_host_bridge_attributes_get() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();

        let coord = WorldCoord::new(5, 5, 5);
        let cell_id = world.set_cell(coord, CellType::Block);

        if let Some(cell) = world.get_mut(coord) {
            cell.attributes.insert("health".to_string(), AttributeValue::Number(100.0));
            cell.attributes.insert("is_boss".to_string(), AttributeValue::Bool(false));
            cell.attributes.insert("tag".to_string(), AttributeValue::String("enemy".to_string()));
        }

        let bridge = ScriptHostBridge {
            entity_manager: &mut entity_manager,
            world: &mut world,
        };

        let result = bridge.get_property(HandleKind::Cell, cell_id, "attributes").unwrap();
        let val = result.expect("attributes property should exist");

        let map_arc = val.as_map().expect("attributes should be a map");
        let map = map_arc.borrow();

        use crate::scripting::value::MapKey;
        assert_eq!(map.get(&MapKey::String("health".to_string())), Some(&Value::Number(100.0)));
        assert_eq!(map.get(&MapKey::String("is_boss".to_string())), Some(&Value::Bool(false)));
        assert_eq!(map.get(&MapKey::String("tag".to_string())), Some(&Value::String("enemy".to_string())));
    }

    #[test]
    fn test_script_host_bridge_runtime_attribute_override() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();

        let coord = WorldCoord::new(0, 0, 0);
        let cell_id = world.set_cell(coord, CellType::Block);

        if let Some(cell) = world.get_mut(coord) {
            cell.attributes.insert("test".to_string(), AttributeValue::String("authored".to_string()));
        }

        let mut bridge = ScriptHostBridge {
            entity_manager: &mut entity_manager,
            world: &mut world,
        };

        // 1. Initial read should be authored value
        let attrs = bridge.get_property(HandleKind::Cell, cell_id, "attributes").unwrap().unwrap();
        let map_arc = attrs.as_map().unwrap();
        assert_eq!(map_arc.borrow().get(&MapKey::String("test".to_string())), Some(&Value::String("authored".to_string())));

        // 2. Runtime write
        bridge.set_attribute(cell_id, "test".to_string(), Value::String("runtime".to_string())).unwrap();

        // 3. Effective read should be runtime value
        let attrs = bridge.get_property(HandleKind::Cell, cell_id, "attributes").unwrap().unwrap();
        let map_arc = attrs.as_map().unwrap();
        assert_eq!(map_arc.borrow().get(&MapKey::String("test".to_string())), Some(&Value::String("runtime".to_string())));

        // 4. Verify authored value in World is UNCHANGED
        assert_eq!(bridge.world.get(coord).unwrap().attributes.get("test"), Some(&AttributeValue::String("authored".to_string())));

        // 5. Runtime removal
        bridge.remove_attribute(cell_id, "test").unwrap();

        // 6. Effective read should fall back to authored value
        let attrs = bridge.get_property(HandleKind::Cell, cell_id, "attributes").unwrap().unwrap();
        let map_arc = attrs.as_map().unwrap();
        assert_eq!(map_arc.borrow().get(&MapKey::String("test".to_string())), Some(&Value::String("authored".to_string())));
    }
}
