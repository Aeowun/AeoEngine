use super::value::{Value, HandleKind};
use crate::engine::entity::{EntityManager, EntityId};

/// Context provided by the engine when executing AeoScript host operations.
pub struct HostContext<'a> {
    pub delta_time: f64,
    pub entity_manager: &'a EntityManager,
}

/// Dispatches a global host function call.
pub fn call_host_function(
    context: &HostContext,
    name: &str,
    arguments: &[Value],
) -> Result<Option<Value>, String> {
    match name {
        "get_entity" => {
            if arguments.len() != 1 {
                return Err("get_entity expects exactly 1 argument (name)".to_string());
            }

            let entity_name = arguments[0].as_string()?;

            if let Some(id) = context.entity_manager.lookup_entity(entity_name) {
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
    context: &HostContext,
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

/// Dispatches a member function call on an engine handle.
pub fn call_host_member(
    context: &HostContext,
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
                    context.entity_manager.validate_handle(handle_id),
                )))
            }

            "name" => {
                if !arguments.is_empty() {
                    return Err("entity.name() expects 0 arguments".to_string());
                }

                if let Some(entity_name) = context.entity_manager.get_name(EntityId(handle_id)) {
                    Ok(Some(Value::String(entity_name.to_string())))
                } else {
                    Err("entity.name() called on an invalid handle".to_string())
                }
            }

            _ => Ok(None),
        },

        _ => Ok(None),
    }
}
