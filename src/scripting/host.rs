use crate::engine::entity::{EntityId, EntityManager};
use crate::scripting::api::EngineHost;
use crate::scripting::value::{HandleKind, MapKey, Value};
use crate::world::cell::AttributeValue;
use crate::world::{CellType, World, WorldCoord};
use glam::Vec3;
use std::collections::BTreeMap;

pub struct ScriptHostBridge<'a> {
    pub entity_manager: &'a mut EntityManager,
    pub world: &'a mut World,
    pub dynamic_properties: &'a mut std::collections::HashMap<
        (HandleKind, u64),
        std::collections::BTreeMap<String, Value>,
    >,
    pub pending_events: &'a mut Vec<(String, Vec<Value>)>,
    pub test_results: &'a mut std::collections::BTreeMap<String, bool>,
    pub pending_enable_scripts: &'a mut Vec<String>,
    pub pending_disable_scripts: &'a mut Vec<String>,
    pub runtime_ui: &'a mut crate::engine::ui::RuntimeUi,
}

fn norm_path(path: &str) -> String {
    let clean = path.replace("\\", "/");
    if clean.starts_with("scripts/") {
        clean
    } else {
        format!("scripts/{}", clean)
    }
}

impl<'a> EngineHost for ScriptHostBridge<'a> {
    fn enable_script(&mut self, path: &str) {
        let norm = norm_path(path);
        self.world.disabled_scripts.retain(|s| s != &norm);
        if !self.pending_enable_scripts.contains(&norm) {
            self.pending_enable_scripts.push(norm);
        }
    }

    fn disable_script(&mut self, path: &str) {
        let norm = norm_path(path);
        if !self.world.disabled_scripts.contains(&norm) {
            self.world.disabled_scripts.push(norm.clone());
        }
        if !self.pending_disable_scripts.contains(&norm) {
            self.pending_disable_scripts.push(norm);
        }
    }

    fn is_script_enabled(&self, path: &str) -> bool {
        let norm = norm_path(path);
        !self.world.disabled_scripts.contains(&norm)
    }

    fn drain_enabled_scripts(&mut self) -> Vec<String> {
        std::mem::take(self.pending_enable_scripts)
    }

    fn drain_disabled_scripts(&mut self) -> Vec<String> {
        std::mem::take(self.pending_disable_scripts)
    }

    fn fire_event(&mut self, event_name: &str, args: Vec<Value>) {
        self.pending_events.push((event_name.to_string(), args));
    }

    fn drain_pending_events(&mut self) -> Vec<(String, Vec<Value>)> {
        std::mem::take(self.pending_events)
    }

    fn complete_test(&mut self, test_name: &str, passed: bool) {
        self.test_results.insert(test_name.to_string(), passed);
    }

    fn is_test_completed(&self, test_name: &str) -> Option<bool> {
        self.test_results.get(test_name).copied()
    }

    fn get_test_results(&self) -> (usize, usize, usize) {
        let passed = self.test_results.values().filter(|&&v| v).count();
        let total = self.test_results.len();
        let failed = total - passed;
        (passed, failed, total)
    }

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

    fn is_collision_events_enabled(&self, id: u64) -> bool {
        if let Some(coord) = self.world.resolve_cell_id(id) {
            if let Some(rs) = self.world.runtime_state.get(&id) {
                if let Some(v) = rs.collision_events_enabled {
                    return v;
                }
            }
            if let Some(cell) = self.world.get_effective_cell(coord) {
                return cell.collision_events_enabled;
            }
        }
        true
    }

    fn create_runtime_cell(&mut self, cell_type: &str) -> Result<(HandleKind, u64), String> {
        let ct = match cell_type {
            "Block" => CellType::Block,
            "FxBlock" => CellType::FxBlock,
            "Player" => CellType::Player,
            "NPC" => CellType::NPC,
            "Light" => CellType::Light,
            "SpawnPoint" => CellType::SpawnPoint,
            "Empty" => return Err("Cannot create Empty cell".to_string()),
            _ => return Err(format!("Unknown cell type: {}", cell_type)),
        };

        let id = self.world.create_runtime_cell(ct);
        let kind = if ct == CellType::Light {
            HandleKind::Light
        } else {
            HandleKind::Cell
        };

        Ok((kind, id))
    }

    fn move_runtime_cell(&mut self, id: u64, x: i32, y: i32, z: i32) -> Result<(), String> {
        let coord = WorldCoord::new(x, y, z);
        self.world.move_runtime_cell(id, coord)
    }

    fn delete_cell(&mut self, id: u64) -> Result<(), String> {
        self.world.delete_cell_runtime(id);
        Ok(())
    }

    fn get_all_cells_of_class(&self, class_name: &str) -> Vec<u64> {
        let mut results = Vec::new();
        for coord in self.world.active_effective_blocks() {
            if let Some(cell) = self.world.get_effective_cell(coord) {
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
        }
        results
    }

    fn find_objects(&self, query: &str) -> Vec<(HandleKind, u64)> {
        let mut results = Vec::new();
        if let Some(id) = self.entity_manager.lookup_entity(query) {
            results.push((HandleKind::Entity, id.0));
        }

        // Search for cells with matching entity_identity
        for coord in self.world.active_effective_blocks() {
            if let Some(cell) = self.world.get_effective_cell(coord) {
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
            if let Some(cell) = self.world.get_effective_cell(coord) {
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
                if let Some(cell) = self.world.get_effective_cell_by_id(id) {
                    match name {
                        "id" => return Ok(Some(Value::Number(cell.id as f64))),
                        "name" => {
                            return Ok(Some(Value::String(
                                cell.entity_identity
                                    .clone()
                                    .unwrap_or_else(|| "Cell".to_string()),
                            )));
                        }
                        "cellType" => {
                            return Ok(Some(Value::String(format!("{:?}", cell.cell_type))));
                        }
                        "position" => {
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                return Ok(Some(Value::array(vec![
                                    Value::Number(coord.x as f64),
                                    Value::Number(coord.y as f64),
                                    Value::Number(coord.z as f64),
                                ])));
                            } else {
                                return Ok(Some(Value::Nil));
                            }
                        }
                        "visible" => {
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                return Ok(Some(Value::Bool(self.world.is_cell_visible(coord))));
                            } else {
                                // Default to cell property if no coord (i.e. no runtime override possible yet via coord-based API)
                                // Actually RuntimeCellState is ID-based.
                                if let Some(rs) = self.world.runtime_state.get(&id) {
                                    if let Some(v) = rs.visible {
                                        return Ok(Some(Value::Bool(v)));
                                    }
                                }
                                return Ok(Some(Value::Bool(cell.visible)));
                            }
                        }
                        "enabled" => {
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                return Ok(Some(Value::Bool(self.world.is_light_enabled(coord))));
                            } else {
                                if let Some(rs) = self.world.runtime_state.get(&id) {
                                    if let Some(v) = rs.light_enabled {
                                        return Ok(Some(Value::Bool(v)));
                                    }
                                }
                                return Ok(Some(Value::Bool(cell.light_enabled)));
                            }
                        }
                        "solid" => {
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                return Ok(Some(Value::Bool(self.world.is_cell_solid(coord))));
                            } else {
                                if let Some(rs) = self.world.runtime_state.get(&id) {
                                    if let Some(v) = rs.solid {
                                        return Ok(Some(Value::Bool(v)));
                                    }
                                }
                                return Ok(Some(Value::Bool(cell.solid)));
                            }
                        }
                        "anchored" => {
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                return Ok(Some(Value::Bool(self.world.is_cell_anchored(coord))));
                            } else {
                                if let Some(rs) = self.world.runtime_state.get(&id) {
                                    if let Some(v) = rs.anchored {
                                        return Ok(Some(Value::Bool(v)));
                                    }
                                }
                                return Ok(Some(Value::Bool(cell.anchored)));
                            }
                        }
                        "color" => {
                            let color = if let Some(coord) = self.world.resolve_cell_id(id) {
                                self.world.get_effective_color(coord)
                            } else {
                                if let Some(rs) = self.world.runtime_state.get(&id) {
                                    if let Some(v) = rs.color_rgb {
                                        v
                                    } else {
                                        cell.color_rgb
                                    }
                                } else {
                                    cell.color_rgb
                                }
                            };
                            return Ok(Some(Value::array(vec![
                                Value::Number(color.x as f64),
                                Value::Number(color.y as f64),
                                Value::Number(color.z as f64),
                            ])));
                        }
                        "offset" => {
                            let offset = if let Some(coord) = self.world.resolve_cell_id(id) {
                                self.world.get_visual_offset(coord)
                            } else {
                                if let Some(rs) = self.world.runtime_state.get(&id) {
                                    rs.visual_offset.unwrap_or(Vec3::ZERO)
                                } else {
                                    Vec3::ZERO
                                }
                            };
                            return Ok(Some(Value::array(vec![
                                Value::Number(offset.x as f64),
                                Value::Number(offset.y as f64),
                                Value::Number(offset.z as f64),
                            ])));
                        }
                        "attributes" => {
                            let mut map = BTreeMap::new();
                            // Effective resolution for attributes:
                            // Authored
                            for (key, attr) in &cell.attributes {
                                let val = match attr {
                                    AttributeValue::Number(n) => Value::Number(*n),
                                    AttributeValue::Bool(b) => Value::Bool(*b),
                                    AttributeValue::String(s) => Value::String(s.clone()),
                                };
                                map.insert(MapKey::String(key.clone()), val);
                            }
                            // Runtime overrides
                            if let Some(rs) = self.world.runtime_state.get(&id) {
                                for (key, attr) in &rs.attribute_overrides {
                                    let val = match attr {
                                        AttributeValue::Number(n) => Value::Number(*n),
                                        AttributeValue::Bool(b) => Value::Bool(*b),
                                        AttributeValue::String(s) => Value::String(s.clone()),
                                    };
                                    map.insert(MapKey::String(key.clone()), val);
                                }
                            }

                            return Ok(Some(Value::map(map)));
                        }
                        _ => {}
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

    fn set_property(
        &mut self,
        kind: HandleKind,
        id: u64,
        name: &str,
        value: Value,
    ) -> Result<bool, String> {
        match kind {
            HandleKind::Cell | HandleKind::Light => {
                if let Some(_cell) = self.world.get_effective_cell_by_id(id) {
                    match name {
                        "position" => {
                            let basket = value.as_basket()?;
                            let borrowed = basket.borrow();
                            if borrowed.elements.len() != 3 {
                                return Err(
                                    "position must be a basket of 3 numbers [x, y, z]".to_string()
                                );
                            }
                            let x = borrowed.elements[0].as_number()? as i32;
                            let y = borrowed.elements[1].as_number()? as i32;
                            let z = borrowed.elements[2].as_number()? as i32;
                            self.move_runtime_cell(id, x, y, z)?;
                            return Ok(true);
                        }
                        "name" => {
                            let name = value.as_string()?;
                            if let Some(cell) = self.world.runtime_cells.get_mut(&id) {
                                cell.entity_identity = Some(name.to_string());
                            } else {
                                return Err(
                                    "Cannot change name of an authored cell at runtime".to_string()
                                );
                            }
                            return Ok(true);
                        }
                        "visible" => {
                            let visible = value.as_bool()?;
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                self.world.set_cell_visible_runtime(coord, visible);
                            } else {
                                self.world.runtime_state.entry(id).or_default().visible =
                                    Some(visible);
                            }
                            return Ok(true);
                        }
                        "enabled" => {
                            let enabled = value.as_bool()?;
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                self.world.set_light_enabled_runtime(coord, enabled);
                            } else {
                                self.world
                                    .runtime_state
                                    .entry(id)
                                    .or_default()
                                    .light_enabled = Some(enabled);
                            }
                            return Ok(true);
                        }
                        "solid" => {
                            let solid = value.as_bool()?;
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                self.world.set_cell_solid_runtime(coord, solid);
                            } else {
                                self.world.runtime_state.entry(id).or_default().solid = Some(solid);
                            }
                            return Ok(true);
                        }
                        "anchored" => {
                            let anchored = value.as_bool()?;
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                self.world.set_cell_anchored_runtime(coord, anchored);
                            } else {
                                self.world.runtime_state.entry(id).or_default().anchored =
                                    Some(anchored);
                            }
                            return Ok(true);
                        }
                        "color" => {
                            let basket = value.as_basket()?;
                            let borrowed = basket.borrow();
                            if borrowed.elements.len() != 3 {
                                return Err(
                                    "color must be a basket of 3 numbers [r, g, b]".to_string()
                                );
                            }
                            let r = borrowed.elements[0].as_number()? as f32;
                            let g = borrowed.elements[1].as_number()? as f32;
                            let b = borrowed.elements[2].as_number()? as f32;
                            let color = Vec3::new(r, g, b);
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                self.world.set_cell_color_runtime(coord, color);
                            } else {
                                self.world.runtime_state.entry(id).or_default().color_rgb =
                                    Some(color);
                            }
                            return Ok(true);
                        }
                        "offset" => {
                            let basket = value.as_basket()?;
                            let borrowed = basket.borrow();
                            if borrowed.elements.len() != 3 {
                                return Err(
                                    "offset must be a basket of 3 numbers [x, y, z]".to_string()
                                );
                            }
                            let x = borrowed.elements[0].as_number()? as f32;
                            let y = borrowed.elements[1].as_number()? as f32;
                            let z = borrowed.elements[2].as_number()? as f32;
                            let offset = Vec3::new(x, y, z);
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                self.world.set_visual_offset_runtime(coord, offset);
                            } else {
                                self.world
                                    .runtime_state
                                    .entry(id)
                                    .or_default()
                                    .visual_offset = Some(offset);
                            }
                            return Ok(true);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        Ok(false)
    }

    fn call_method(
        &mut self,
        _kind: HandleKind,
        _id: u64,
        _name: &str,
        _args: &[Value],
    ) -> Result<Option<Value>, String> {
        Ok(None)
    }

    fn set_attribute(&mut self, id: u64, key: String, value: Value) -> Result<(), String> {
        if self.world.get_effective_cell_by_id(id).is_some() {
            let attr_val = match value {
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
                .insert(key, attr_val);
            Ok(())
        } else {
            Err("invalid cell handle for attribute assignment".to_string())
        }
    }

    fn remove_attribute(&mut self, id: u64, key: &str) -> Result<(), String> {
        if self.world.get_effective_cell_by_id(id).is_some() {
            if let Some(rs) = self.world.runtime_state.get_mut(&id) {
                rs.attribute_overrides.remove(key);
            }
            Ok(())
        } else {
            Err("invalid cell handle for attribute removal".to_string())
        }
    }

    fn cell_exists(&self, id: u64) -> bool {
        self.world.get_effective_cell_by_id(id).is_some()
    }

    fn entity_exists(&self, id: u64) -> bool {
        self.entity_manager.validate_handle(id)
    }

    fn get_script_property(&self, kind: HandleKind, id: u64, name: &str) -> Option<Value> {
        self.dynamic_properties
            .get(&(kind, id))
            .and_then(|m| m.get(name))
            .cloned()
    }

    fn set_script_property(&mut self, kind: HandleKind, id: u64, name: String, value: Value) {
        self.dynamic_properties
            .entry((kind, id))
            .or_default()
            .insert(name, value);
    }

    fn create_ui_element(&mut self, element_type: &str) -> Result<u64, String> {
        self.runtime_ui.allocate(element_type)
    }

    fn delete_ui_element(&mut self, id: u64) -> Result<(), String> {
        self.runtime_ui.delete(id)
    }

    fn get_ui_property(&self, id: u64, name: &str) -> Result<Option<Value>, String> {
        self.runtime_ui.get_property(id, name)
    }

    fn set_ui_property(&mut self, id: u64, name: &str, value: Value) -> Result<bool, String> {
        self.runtime_ui.set_property(id, name, value)
    }

    fn drain_ui_clicks(&mut self) -> Vec<u64> {
        self.runtime_ui.drain_pending_clicks()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::api::EngineHost;
    use crate::scripting::value::{HandleKind, Value};
    use crate::world::WorldCoord;
    use crate::world::cell::{AttributeValue, CellType};

    #[test]
    fn test_script_host_bridge_attributes_get() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();

        let coord = WorldCoord::new(5, 5, 5);
        let cell_id = world.set_cell(coord, CellType::Block);

        if let Some(cell) = world.get_mut(coord) {
            cell.attributes
                .insert("health".to_string(), AttributeValue::Number(100.0));
            cell.attributes
                .insert("is_boss".to_string(), AttributeValue::Bool(false));
            cell.attributes.insert(
                "tag".to_string(),
                AttributeValue::String("enemy".to_string()),
            );
        }

        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui = crate::engine::ui::RuntimeUi::new();
        let bridge = ScriptHostBridge {
            entity_manager: &mut entity_manager,
            world: &mut world,
            dynamic_properties: &mut dynamic_properties,
            pending_events: &mut pending_events,
            test_results: &mut test_results,
            pending_enable_scripts: &mut pending_enable_scripts,
            pending_disable_scripts: &mut pending_disable_scripts,
            runtime_ui: &mut runtime_ui,
        };

        let result = bridge
            .get_property(HandleKind::Cell, cell_id, "attributes")
            .unwrap();
        let val = result.expect("attributes property should exist");

        let map_arc = val.as_map().expect("attributes should be a map");
        let map = map_arc.borrow();

        use crate::scripting::value::MapKey;
        assert_eq!(
            map.get(&MapKey::String("health".to_string())),
            Some(&Value::Number(100.0))
        );
        assert_eq!(
            map.get(&MapKey::String("is_boss".to_string())),
            Some(&Value::Bool(false))
        );
        assert_eq!(
            map.get(&MapKey::String("tag".to_string())),
            Some(&Value::String("enemy".to_string()))
        );
    }

    #[test]
    fn test_script_host_bridge_runtime_attribute_override() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();

        let coord = WorldCoord::new(0, 0, 0);
        let cell_id = world.set_cell(coord, CellType::Block);

        if let Some(cell) = world.get_mut(coord) {
            cell.attributes.insert(
                "test".to_string(),
                AttributeValue::String("authored".to_string()),
            );
        }

        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui = crate::engine::ui::RuntimeUi::new();
        let mut bridge = ScriptHostBridge {
            entity_manager: &mut entity_manager,
            world: &mut world,
            dynamic_properties: &mut dynamic_properties,
            pending_events: &mut pending_events,
            test_results: &mut test_results,
            pending_enable_scripts: &mut pending_enable_scripts,
            pending_disable_scripts: &mut pending_disable_scripts,
            runtime_ui: &mut runtime_ui,
        };

        // 1. Initial read should be authored value
        let attrs = bridge
            .get_property(HandleKind::Cell, cell_id, "attributes")
            .unwrap()
            .unwrap();
        let map_arc = attrs.as_map().unwrap();
        assert_eq!(
            map_arc.borrow().get(&MapKey::String("test".to_string())),
            Some(&Value::String("authored".to_string()))
        );

        // 2. Runtime write
        bridge
            .set_attribute(
                cell_id,
                "test".to_string(),
                Value::String("runtime".to_string()),
            )
            .unwrap();

        // 3. Effective read should be runtime value
        let attrs = bridge
            .get_property(HandleKind::Cell, cell_id, "attributes")
            .unwrap()
            .unwrap();
        let map_arc = attrs.as_map().unwrap();
        assert_eq!(
            map_arc.borrow().get(&MapKey::String("test".to_string())),
            Some(&Value::String("runtime".to_string()))
        );

        // 4. Verify authored value in World is UNCHANGED
        assert_eq!(
            bridge.world.get(coord).unwrap().attributes.get("test"),
            Some(&AttributeValue::String("authored".to_string()))
        );

        // 5. Runtime removal
        bridge.remove_attribute(cell_id, "test").unwrap();

        // 6. Effective read should fall back to authored value
        let attrs = bridge
            .get_property(HandleKind::Cell, cell_id, "attributes")
            .unwrap()
            .unwrap();
        let map_arc = attrs.as_map().unwrap();
        assert_eq!(
            map_arc.borrow().get(&MapKey::String("test".to_string())),
            Some(&Value::String("authored".to_string()))
        );
    }
}
