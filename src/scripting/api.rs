use super::value::{HandleKind, Value};
use crate::engine::entity::{EntityId, EntityManager};
use glam::Vec3;

/// Abstraction for engine services exposed to AeoScript.
///
/// This bridge allows the scripting runtime to interact with engine systems
/// without being coupled to the full App or Renderer implementation.
pub trait EngineHost {
    fn entity_manager(&self) -> &EntityManager;
    fn get_position(&self, id: u64) -> Option<Vec3>;
    fn set_position(&mut self, id: u64, position: Vec3);

    fn lookup_light(&self, x: i32, y: i32, z: i32) -> Option<u64>;
    fn is_light_enabled(&self, id: u64) -> Option<bool>;
    fn set_light_enabled(&mut self, id: u64, enabled: bool);
    fn is_collision_events_enabled(&self, id: u64) -> bool;

    // Generic object model support
    fn get_all_cells_of_class(&self, class_name: &str) -> Vec<u64>;
    fn find_objects(&self, query: &str) -> Vec<(HandleKind, u64)>;
    fn get_children(&self, kind: HandleKind, id: u64) -> Vec<(HandleKind, u64)>;
    fn get_parent(&self, kind: HandleKind, id: u64) -> Option<(HandleKind, u64)>;
    fn get_cell_object(&self, cell_id: u64) -> Option<(HandleKind, u64)>;

    fn get_property(&self, kind: HandleKind, id: u64, name: &str) -> Result<Option<Value>, String>;
    fn set_property(
        &mut self,
        kind: HandleKind,
        id: u64,
        name: &str,
        value: Value,
    ) -> Result<bool, String>;
    fn call_method(
        &mut self,
        kind: HandleKind,
        id: u64,
        name: &str,
        args: &[Value],
    ) -> Result<Option<Value>, String>;

    fn set_attribute(&mut self, id: u64, key: String, value: Value) -> Result<(), String>;
    fn remove_attribute(&mut self, id: u64, key: &str) -> Result<(), String>;

    fn cell_exists(&self, id: u64) -> bool;
    fn entity_exists(&self, id: u64) -> bool;

    fn get_script_property(&self, kind: HandleKind, id: u64, name: &str) -> Option<Value>;
    fn set_script_property(&mut self, kind: HandleKind, id: u64, name: String, value: Value);

    fn create_runtime_cell(&mut self, cell_type: &str) -> Result<(HandleKind, u64), String>;
    fn move_runtime_cell(&mut self, id: u64, x: i32, y: i32, z: i32) -> Result<(), String>;
    fn delete_cell(&mut self, id: u64) -> Result<(), String>;

    // Runtime UI support
    fn create_ui_element(&mut self, _element_type: &str) -> Result<u64, String> {
        Err("Runtime UI is not supported by this host".to_string())
    }
    fn delete_ui_element(&mut self, _id: u64) -> Result<(), String> {
        Err("Runtime UI is not supported by this host".to_string())
    }
    fn get_ui_property(&self, _id: u64, _name: &str) -> Result<Option<Value>, String> {
        Ok(None)
    }
    fn set_ui_property(&mut self, _id: u64, _name: &str, _value: Value) -> Result<bool, String> {
        Ok(false)
    }
    fn drain_ui_clicks(&mut self) -> Vec<u64> {
        Vec::new()
    }

    // Runtime script management, event firing, and test orchestration
    fn enable_script(&mut self, _path: &str) {}
    fn disable_script(&mut self, _path: &str) {}
    fn is_script_enabled(&self, _path: &str) -> bool {
        true
    }
    fn drain_enabled_scripts(&mut self) -> Vec<String> {
        Vec::new()
    }
    fn drain_disabled_scripts(&mut self) -> Vec<String> {
        Vec::new()
    }
    fn fire_event(&mut self, _event_name: &str, _args: Vec<Value>) {}
    fn drain_pending_events(&mut self) -> Vec<(String, Vec<Value>)> {
        Vec::new()
    }
    fn complete_test(&mut self, _test_name: &str, _passed: bool) {}
    fn is_test_completed(&self, _test_name: &str) -> Option<bool> {
        None
    }
    fn get_test_results(&self) -> (usize, usize, usize) {
        (0, 0, 0)
    }
}

/// A simple implementation of EngineHost that just wraps an EntityManager.
/// Useful for tests and the initial bridge implementation.
impl EngineHost for EntityManager {
    fn entity_manager(&self) -> &EntityManager {
        self
    }

    fn get_position(&self, id: u64) -> Option<Vec3> {
        EntityManager::get_position(self, EntityId(id))
    }

    fn set_position(&mut self, id: u64, position: Vec3) {
        EntityManager::set_position(self, EntityId(id), position);
    }

    fn lookup_light(&self, _x: i32, _y: i32, _z: i32) -> Option<u64> {
        None
    }

    fn is_light_enabled(&self, _id: u64) -> Option<bool> {
        None
    }

    fn set_light_enabled(&mut self, _id: u64, _enabled: bool) {}

    fn is_collision_events_enabled(&self, _id: u64) -> bool {
        true
    }

    fn get_all_cells_of_class(&self, _class_name: &str) -> Vec<u64> {
        Vec::new()
    }

    fn find_objects(&self, _query: &str) -> Vec<(HandleKind, u64)> {
        Vec::new()
    }

    fn get_children(&self, _kind: HandleKind, _id: u64) -> Vec<(HandleKind, u64)> {
        Vec::new()
    }

    fn get_parent(&self, _kind: HandleKind, _id: u64) -> Option<(HandleKind, u64)> {
        None
    }

    fn get_cell_object(&self, _cell_id: u64) -> Option<(HandleKind, u64)> {
        None
    }

    fn get_property(
        &self,
        _kind: HandleKind,
        _id: u64,
        _name: &str,
    ) -> Result<Option<Value>, String> {
        Ok(None)
    }

    fn set_property(
        &mut self,
        _kind: HandleKind,
        _id: u64,
        _name: &str,
        _value: Value,
    ) -> Result<bool, String> {
        Ok(false)
    }

    fn get_script_property(&self, _kind: HandleKind, _id: u64, _name: &str) -> Option<Value> {
        None
    }

    fn set_script_property(&mut self, _kind: HandleKind, _id: u64, _name: String, _value: Value) {}

    fn call_method(
        &mut self,
        _kind: HandleKind,
        _id: u64,
        _name: &str,
        _args: &[Value],
    ) -> Result<Option<Value>, String> {
        Ok(None)
    }

    fn set_attribute(&mut self, _id: u64, _key: String, _value: Value) -> Result<(), String> {
        Ok(())
    }

    fn remove_attribute(&mut self, _id: u64, _key: &str) -> Result<(), String> {
        Ok(())
    }

    fn cell_exists(&self, _id: u64) -> bool {
        false
    }

    fn entity_exists(&self, id: u64) -> bool {
        EntityManager::validate_handle(self, id)
    }

    fn create_runtime_cell(&mut self, _cell_type: &str) -> Result<(HandleKind, u64), String> {
        Err("Runtime cell creation not supported in this host".to_string())
    }

    fn move_runtime_cell(&mut self, _id: u64, _x: i32, _y: i32, _z: i32) -> Result<(), String> {
        Err("Runtime cell movement not supported in this host".to_string())
    }

    fn delete_cell(&mut self, _id: u64) -> Result<(), String> {
        Err("Cell deletion not supported in this host".to_string())
    }
}

/// Context provided by the engine when executing AeoScript host operations.
pub struct HostContext<'a> {
    pub delta_time: f64,
    pub engine: &'a mut dyn EngineHost,
}

/// Dispatches a global host function call.
pub fn call_host_function(
    context: &mut HostContext,
    name: &str,
    arguments: &[Value],
) -> Result<Option<Value>, String> {
    match name {
        "get_entity" => {
            if arguments.len() != 1 {
                return Err("get_entity expects exactly 1 argument (name)".to_string());
            }

            let entity_name = arguments[0].as_string()?;

            if let Some(id) = context.engine.entity_manager().lookup_entity(entity_name) {
                Ok(Some(Value::Handle {
                    kind: HandleKind::Entity,
                    id: id.0,
                }))
            } else {
                Ok(Some(Value::Nil))
            }
        }

        "get_light" => {
            if arguments.len() != 3 {
                return Err("get_light expects exactly 3 arguments (x, y, z)".to_string());
            }

            let x = arguments[0].as_number()? as i32;
            let y = arguments[1].as_number()? as i32;
            let z = arguments[2].as_number()? as i32;

            if let Some(id) = context.engine.lookup_light(x, y, z) {
                Ok(Some(Value::Handle {
                    kind: HandleKind::Light,
                    id,
                }))
            } else {
                Ok(Some(Value::Nil))
            }
        }

        "getAllCellsOfClass" => {
            if arguments.len() != 1 {
                return Err(
                    "getAllCellsOfClass expects exactly 1 argument (class_name)".to_string()
                );
            }

            let class_name = arguments[0].as_string()?;
            let cells = context.engine.get_all_cells_of_class(class_name);

            let handles = cells
                .into_iter()
                .map(|id| Value::Handle {
                    kind: HandleKind::Cell,
                    id,
                })
                .collect();
            Ok(Some(Value::array(handles)))
        }

        "find" => {
            if arguments.len() != 1 {
                return Err("find expects exactly 1 argument (query)".to_string());
            }

            let query = arguments[0].as_string()?;
            let objects = context.engine.find_objects(query);
            let handles = objects
                .into_iter()
                .map(|(kind, id)| Value::Handle { kind, id })
                .collect();
            Ok(Some(Value::array(handles)))
        }

        "enable_script" => {
            if arguments.len() != 1 {
                return Err("enable_script expects 1 argument (path)".to_string());
            }
            let path = arguments[0].as_string()?;
            context.engine.enable_script(path);
            Ok(Some(Value::Nil))
        }

        "disable_script" => {
            if arguments.len() != 1 {
                return Err("disable_script expects 1 argument (path)".to_string());
            }
            let path = arguments[0].as_string()?;
            context.engine.disable_script(path);
            Ok(Some(Value::Nil))
        }

        "is_script_enabled" => {
            if arguments.len() != 1 {
                return Err("is_script_enabled expects 1 argument (path)".to_string());
            }
            let path = arguments[0].as_string()?;
            let enabled = context.engine.is_script_enabled(path);
            Ok(Some(Value::Bool(enabled)))
        }

        "fire_event" => {
            if arguments.is_empty() {
                return Err("fire_event expects at least 1 argument (event_name)".to_string());
            }
            let event_name = arguments[0].as_string()?;
            let event_args = arguments[1..].to_vec();
            context.engine.fire_event(event_name, event_args);
            Ok(Some(Value::Nil))
        }

        "complete_test" => {
            if arguments.len() < 2 {
                return Err("complete_test expects 2 arguments (test_name, passed)".to_string());
            }
            let test_name = arguments[0].as_string()?;
            let passed = arguments[1].as_bool()?;
            context.engine.complete_test(test_name, passed);
            Ok(Some(Value::Nil))
        }

        "is_test_completed" => {
            if arguments.len() != 1 {
                return Err("is_test_completed expects 1 argument (test_name)".to_string());
            }
            let test_name = arguments[0].as_string()?;
            if let Some(_passed) = context.engine.is_test_completed(test_name) {
                Ok(Some(Value::Bool(true)))
            } else {
                Ok(Some(Value::Bool(false)))
            }
        }

        "test_passed" => {
            if arguments.len() != 1 {
                return Err("test_passed expects 1 argument (test_name)".to_string());
            }
            let test_name = arguments[0].as_string()?;
            if let Some(passed) = context.engine.is_test_completed(test_name) {
                Ok(Some(Value::Bool(passed)))
            } else {
                Ok(Some(Value::Nil))
            }
        }

        "test_summary" => {
            let (p, f, t) = context.engine.get_test_results();
            Ok(Some(Value::array(vec![
                Value::Number(p as f64),
                Value::Number(f as f64),
                Value::Number(t as f64),
            ])))
        }

        _ => Ok(None),
    }
}

/// Resolves a property on a host object (e.g. time.delta).
pub fn resolve_host_property(
    context: &mut HostContext,
    object_name: &str,
    property_name: &str,
) -> Result<Option<Value>, String> {
    match object_name {
        "time" => {
            if property_name == "delta" {
                return Ok(Some(Value::Number(context.delta_time)));
            }
        }

        _ => {}
    }

    Ok(None)
}

/// Resolves a property on an engine handle (e.g. entity.position).
pub fn resolve_host_member_property(
    context: &mut HostContext,
    handle_kind: HandleKind,
    handle_id: u64,
    property_name: &str,
) -> Result<Option<Value>, String> {
    if handle_kind == HandleKind::Ui {
        return context.engine.get_ui_property(handle_id, property_name);
    }

    if let Some(value) = context
        .engine
        .get_property(handle_kind, handle_id, property_name)?
    {
        return Ok(Some(value));
    }

    if let Some(value) = context
        .engine
        .get_script_property(handle_kind, handle_id, property_name)
    {
        return Ok(Some(value));
    }

    match handle_kind {
        HandleKind::Entity => match property_name {
            "name" => {
                if let Some(name) = context
                    .engine
                    .entity_manager()
                    .get_name(EntityId(handle_id))
                {
                    Ok(Some(Value::String(name.to_string())))
                } else {
                    Ok(Some(Value::Nil))
                }
            }
            "position" => {
                if let Some(pos) = context.engine.get_position(handle_id) {
                    Ok(Some(Value::array(vec![
                        Value::Number(pos.x as f64),
                        Value::Number(pos.y as f64),
                        Value::Number(pos.z as f64),
                    ])))
                } else {
                    Ok(Some(Value::Nil))
                }
            }
            _ => Ok(None),
        },
        _ => Ok(None),
    }
}

/// Sets a property on an engine handle (e.g. entity.position = ...).
pub fn set_host_member_property(
    context: &mut HostContext,
    handle_kind: HandleKind,
    handle_id: u64,
    property_name: &str,
    value: Value,
) -> Result<(), String> {
    if handle_kind == HandleKind::Ui {
        if context.engine.set_ui_property(handle_id, property_name, value)? {
            return Ok(());
        } else {
            return Err(format!("failed to set UI property '{}'", property_name));
        }
    }

    if context
        .engine
        .set_property(handle_kind, handle_id, property_name, value.clone())?
    {
        Ok(())
    } else {
        context.engine.set_script_property(
            handle_kind,
            handle_id,
            property_name.to_string(),
            value,
        );
        Ok(())
    }
}

/// Dispatches a member function call on an engine handle.
pub fn call_host_member(
    context: &mut HostContext,
    handle_kind: HandleKind,
    handle_id: u64,
    name: &str,
    arguments: &[Value],
) -> Result<Option<Value>, String> {
    if let Some(result) = context
        .engine
        .call_method(handle_kind, handle_id, name, arguments)?
    {
        return Ok(Some(result));
    }

    match name {
        "get_children" => {
            let children = context.engine.get_children(handle_kind, handle_id);
            let handles: Vec<Value> = children
                .into_iter()
                .map(|(kind, id)| Value::Handle { kind, id })
                .collect();
            return Ok(Some(Value::array(handles)));
        }
        "get_parent" => {
            if let Some((kind, id)) = context.engine.get_parent(handle_kind, handle_id) {
                return Ok(Some(Value::Handle { kind, id }));
            } else {
                return Ok(Some(Value::Nil));
            }
        }
        _ => {}
    }

    match handle_kind {
        HandleKind::Cell | HandleKind::Light => match name {
            "getObject" => {
                if !arguments.is_empty() {
                    return Err(format!(
                        "{}.getObject() expects 0 arguments",
                        handle_kind.name()
                    ));
                }
                if let Some((kind, id)) = context.engine.get_cell_object(handle_id) {
                    Ok(Some(Value::Handle { kind, id }))
                } else {
                    Ok(Some(Value::Nil))
                }
            }

            "is_enabled" => {
                if !arguments.is_empty() {
                    return Err(format!(
                        "{}.is_enabled() expects 0 arguments",
                        handle_kind.name()
                    ));
                }

                if let Some(enabled) = context.engine.is_light_enabled(handle_id) {
                    Ok(Some(Value::Bool(enabled)))
                } else {
                    Ok(Some(Value::Nil))
                }
            }

            "set_enabled" => {
                if arguments.len() != 1 {
                    return Err(format!(
                        "{}.set_enabled() expects 1 argument (bool)",
                        handle_kind.name()
                    ));
                }

                let enabled = arguments[0].as_bool()?;
                context.engine.set_light_enabled(handle_id, enabled);

                Ok(Some(Value::Nil))
            }

            _ => Ok(None),
        },

        HandleKind::Entity => match name {
            "is_valid" => {
                if !arguments.is_empty() {
                    return Err("entity.is_valid() expects 0 arguments".to_string());
                }

                Ok(Some(Value::Bool(
                    context.engine.entity_manager().validate_handle(handle_id),
                )))
            }
            // ... (keep existing name, set_position, translate)
            "name" => {
                if !arguments.is_empty() {
                    return Err("entity.name() expects 0 arguments".to_string());
                }

                if let Some(entity_name) = context
                    .engine
                    .entity_manager()
                    .get_name(EntityId(handle_id))
                {
                    Ok(Some(Value::String(entity_name.to_string())))
                } else {
                    Err("entity.name() called on an invalid handle".to_string())
                }
            }

            "set_position" => {
                if arguments.len() != 3 {
                    return Err("entity.set_position() expects 3 arguments (x, y, z)".to_string());
                }

                let x = arguments[0].as_number()? as f32;
                let y = arguments[1].as_number()? as f32;
                let z = arguments[2].as_number()? as f32;

                if context.engine.entity_manager().validate_handle(handle_id) {
                    context.engine.set_position(handle_id, Vec3::new(x, y, z));
                }

                Ok(Some(Value::Nil))
            }

            "translate" => {
                if arguments.len() != 3 {
                    return Err("entity.translate() expects 3 arguments (x, y, z)".to_string());
                }

                let dx = arguments[0].as_number()? as f32;
                let dy = arguments[1].as_number()? as f32;
                let dz = arguments[2].as_number()? as f32;

                if let Some(pos) = context.engine.get_position(handle_id) {
                    let new_pos = pos + Vec3::new(dx, dy, dz);
                    context.engine.set_position(handle_id, new_pos);
                }

                Ok(Some(Value::Nil))
            }

            _ => Ok(None),
        },

        HandleKind::Ui => Ok(None),
    }
}
