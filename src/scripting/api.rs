use glam::Vec3;
use super::value::{Value, HandleKind};
use crate::engine::entity::{EntityManager, EntityId};
use crate::world::WorldCoord;

/// Packs a 3D coordinate into a u64 for use as a handle ID.
pub fn pack_coord(coord: WorldCoord) -> u64 {
    let x = (coord.x as u64) & 0xFFFFF;
    let y = (coord.y as u64) & 0xFFFFF;
    let z = (coord.z as u64) & 0xFFFFF;
    x | (y << 20) | (z << 40)
}

/// Unpacks a u64 handle ID back into a 3D coordinate.
pub fn unpack_coord(id: u64) -> WorldCoord {
    let mut x = (id & 0xFFFFF) as i32;
    if x >= 0x80000 { x -= 0x100000; }
    let mut y = ((id >> 20) & 0xFFFFF) as i32;
    if y >= 0x80000 { y -= 0x100000; }
    let mut z = ((id >> 40) & 0xFFFFF) as i32;
    if z >= 0x80000 { z -= 0x100000; }
    WorldCoord::new(x, y, z)
}

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
        HandleKind::Light => match property_name {
            "is_enabled" => {
                // We could expose this as a property or a method.
                // The task says light.is_enabled() which is a method call.
                // But it's good to have properties if needed.
                // The task specifically asked for methods.
                Ok(None)
            }
            _ => Ok(None),
        },
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

        HandleKind::Light => match name {
            "is_enabled" => {
                if !arguments.is_empty() {
                    return Err("light.is_enabled() expects 0 arguments".to_string());
                }

                if let Some(enabled) = context.engine.is_light_enabled(handle_id) {
                    Ok(Some(Value::Bool(enabled)))
                } else {
                    // Returns null or error if handle is invalid.
                    // Scripting convention usually prefers null for invalid/not found if it's an optional-like check.
                    Ok(Some(Value::Null))
                }
            }

            "set_enabled" => {
                if arguments.len() != 1 {
                    return Err("light.set_enabled() expects 1 argument (bool)".to_string());
                }

                let enabled = arguments[0].as_bool()?;
                context.engine.set_light_enabled(handle_id, enabled);

                Ok(Some(Value::Null))
            }

            _ => Ok(None),
        },
    }
}
