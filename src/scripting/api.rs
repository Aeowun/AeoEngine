use glam::Vec3;
use super::value::{Value, HandleKind};
use crate::engine::entity::{EntityManager, EntityId};

/// Abstraction for engine services exposed to AeoScript.
///
/// This bridge allows the scripting runtime to interact with engine systems
/// without being coupled to the full App or Renderer implementation.
pub trait EngineHost {
    fn entity_manager(&self) -> &EntityManager;
    fn get_position(&self, id: u64) -> Option<Vec3>;
    fn set_position(&mut self, id: u64, position: Vec3);
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
                Ok(Some(Value::Null))
            }
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
    match handle_kind {
        HandleKind::Entity => match property_name {
            "position" => {
                if let Some(pos) = context.engine.get_position(handle_id) {
                    Ok(Some(Value::Array(vec![
                        Value::Number(pos.x as f64),
                        Value::Number(pos.y as f64),
                        Value::Number(pos.z as f64),
                    ])))
                } else {
                    Ok(Some(Value::Null))
                }
            }
            _ => Ok(None),
        },
        _ => Ok(None),
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
    match handle_kind {
        HandleKind::Entity => match name {
            "is_valid" => {
                if !arguments.is_empty() {
                    return Err("entity.is_valid() expects 0 arguments".to_string());
                }

                Ok(Some(Value::Bool(
                    context.engine.entity_manager().validate_handle(handle_id),
                )))
            }

            "name" => {
                if !arguments.is_empty() {
                    return Err("entity.name() expects 0 arguments".to_string());
                }

                if let Some(entity_name) = context.engine.entity_manager().get_name(EntityId(handle_id)) {
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

                Ok(Some(Value::Null))
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

                Ok(Some(Value::Null))
            }

            _ => Ok(None),
        },

        _ => Ok(None),
    }
}
