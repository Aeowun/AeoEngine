pub mod host_audio;
pub mod host_camera;
pub mod host_cells;
pub mod host_character;
pub mod host_entities;
pub mod host_input;
pub mod host_ui;

use std::collections::BTreeMap;
use std::sync::Arc;

use glam::Vec3;

use crate::engine::entity::{EntityId, EntityManager};
use crate::engine::mouse::MouseController;
use crate::scripting::api::EngineHost;
use crate::scripting::value::{HandleKind, MapKey, Value};
use crate::world::World;
use crate::world::cell::AttributeValue;

fn norm_path(path: &str) -> String {
    let clean = path.replace('\\', "/");

    if clean.starts_with("scripts/")
        || clean.starts_with("controllers/")
        || clean.starts_with("cameras/")
    {
        clean
    } else {
        format!("scripts/{}", clean)
    }
}

/// Runtime bridge between AeoScript and the live engine.
pub struct ScriptHostBridge<'a> {
    pub entity_manager: &'a mut EntityManager,
    pub world: &'a mut World,
    pub mouse: &'a mut MouseController,

    pub dynamic_properties: &'a mut std::collections::HashMap<
        (HandleKind, u64),
        std::collections::BTreeMap<String, Value>,
    >,

    pub pending_events: &'a mut Vec<(String, Vec<Value>)>,

    pub test_results: &'a mut std::collections::BTreeMap<String, bool>,

    pub pending_enable_scripts: &'a mut Vec<String>,
    pub pending_disable_scripts: &'a mut Vec<String>,

    pub runtime_ui: &'a mut crate::engine::ui::RuntimeUi,

    pub viewport_size: [f32; 2],

    pub move_input: glam::Vec2,

    pub jump_requested: bool,

    pub orbit_delta: [f32; 2],

    pub character_system: Option<&'a mut crate::character::CharacterSystem>,

    pub gameplay_camera: Option<&'a mut crate::renderer::camera::GameplayCamera>,

    pub valid_entity_declarations: Option<Arc<std::collections::HashSet<String>>>,

    pub pending_spawns: &'a mut Vec<(String, u64)>,

    pub project_path: Option<&'a std::path::Path>,
}

impl<'a> ScriptHostBridge<'a> {
    pub fn get_property(
        &self,
        kind: HandleKind,
        id: u64,
        name: &str,
    ) -> Result<Option<Value>, String> {
        match kind {
            HandleKind::Mouse => {
                if id != 0 {
                    return Err("Invalid Mouse handle".to_string());
                }

                match name {
                    "setCursorVisible" => {
                        return Ok(Some(Value::Bool(self.mouse.cursor_visible)));
                    }

                    "setScreenLocked" => {
                        return Ok(Some(Value::Bool(self.mouse.screen_locked)));
                    }

                    _ => {}
                }
            }

            HandleKind::Cell | HandleKind::Light => {
                if let Some(cell) = self.world.get_effective_cell_by_id(id) {
                    match name {
                        "sound" => {
                            return Ok(Some(Value::Handle {
                                kind: HandleKind::Sound,
                                id,
                            }));
                        }

                        "id" => {
                            return Ok(Some(Value::Number(cell.id as f64)));
                        }

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
                            }

                            return Ok(Some(Value::Nil));
                        }

                        "visible" => {
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                return Ok(Some(Value::Bool(self.world.is_cell_visible(coord))));
                            }

                            if let Some(runtime_state) = self.world.runtime_state.get(&id) {
                                if let Some(value) = runtime_state.visible {
                                    return Ok(Some(Value::Bool(value)));
                                }
                            }

                            return Ok(Some(Value::Bool(cell.visible)));
                        }

                        "enabled" => {
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                return Ok(Some(Value::Bool(self.world.is_light_enabled(coord))));
                            }

                            if let Some(runtime_state) = self.world.runtime_state.get(&id) {
                                if let Some(value) = runtime_state.light_enabled {
                                    return Ok(Some(Value::Bool(value)));
                                }
                            }

                            return Ok(Some(Value::Bool(cell.light_enabled)));
                        }

                        "solid" => {
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                return Ok(Some(Value::Bool(self.world.is_cell_solid(coord))));
                            }

                            if let Some(runtime_state) = self.world.runtime_state.get(&id) {
                                if let Some(value) = runtime_state.solid {
                                    return Ok(Some(Value::Bool(value)));
                                }
                            }

                            return Ok(Some(Value::Bool(cell.solid)));
                        }

                        "anchored" => {
                            if let Some(coord) = self.world.resolve_cell_id(id) {
                                return Ok(Some(Value::Bool(self.world.is_cell_anchored(coord))));
                            }

                            if let Some(runtime_state) = self.world.runtime_state.get(&id) {
                                if let Some(value) = runtime_state.anchored {
                                    return Ok(Some(Value::Bool(value)));
                                }
                            }

                            return Ok(Some(Value::Bool(cell.anchored)));
                        }

                        "color" => {
                            let color = if let Some(coord) = self.world.resolve_cell_id(id) {
                                self.world.get_effective_color(coord)
                            } else if let Some(runtime_state) = self.world.runtime_state.get(&id) {
                                runtime_state.color_rgb.unwrap_or(cell.color_rgb)
                            } else {
                                cell.color_rgb
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
                            } else if let Some(runtime_state) = self.world.runtime_state.get(&id) {
                                runtime_state.visual_offset.unwrap_or(Vec3::ZERO)
                            } else {
                                Vec3::ZERO
                            };

                            return Ok(Some(Value::array(vec![
                                Value::Number(offset.x as f64),
                                Value::Number(offset.y as f64),
                                Value::Number(offset.z as f64),
                            ])));
                        }

                        "attributes" => {
                            let mut map = BTreeMap::new();

                            // Begin with authored attributes.
                            for (key, attribute) in &cell.attributes {
                                let value = match attribute {
                                    AttributeValue::Number(n) => Value::Number(*n),
                                    AttributeValue::Bool(b) => Value::Bool(*b),
                                    AttributeValue::String(s) => Value::String(s.clone()),
                                };

                                map.insert(MapKey::String(key.clone()), value);
                            }

                            // Runtime overrides replace authored values.
                            if let Some(runtime_state) = self.world.runtime_state.get(&id) {
                                for (key, attribute) in &runtime_state.attribute_overrides {
                                    let value = match attribute {
                                        AttributeValue::Number(n) => Value::Number(*n),
                                        AttributeValue::Bool(b) => Value::Bool(*b),
                                        AttributeValue::String(s) => Value::String(s.clone()),
                                    };

                                    map.insert(MapKey::String(key.clone()), value);
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

                if !self.entity_manager.validate_handle(id) {
                    return Ok(None);
                }

                match name {
                    "sound" => {
                        return Ok(Some(Value::Handle {
                            kind: HandleKind::Sound,
                            id,
                        }));
                    }

                    "id" => {
                        return Ok(Some(Value::Number(id as f64)));
                    }

                    "name" => {
                        if let Some(entity_name) = self.entity_manager.get_name(entity_id) {
                            return Ok(Some(Value::String(entity_name.to_string())));
                        }

                        return Ok(Some(Value::Nil));
                    }

                    "position" => {
                        if let Some(position) = self.get_position(id) {
                            return Ok(Some(Value::array(vec![
                                Value::Number(position.x as f64),
                                Value::Number(position.y as f64),
                                Value::Number(position.z as f64),
                            ])));
                        }

                        return Ok(Some(Value::Nil));
                    }

                    "velocity" => {
                        if let Some(character_system) = self.character_system.as_deref() {
                            if let Some(velocity) = character_system.get_entity_velocity(entity_id)
                            {
                                return Ok(Some(Value::array(vec![
                                    Value::Number(velocity.x as f64),
                                    Value::Number(velocity.y as f64),
                                    Value::Number(velocity.z as f64),
                                ])));
                            }
                        }

                        return Ok(Some(Value::Nil));
                    }

                    "facing" => {
                        if let Some(character_system) = self.character_system.as_deref() {
                            if let Some(facing) = character_system.get_entity_facing(entity_id) {
                                return Ok(Some(Value::array(vec![
                                    Value::Number(facing.x as f64),
                                    Value::Number(facing.y as f64),
                                ])));
                            }
                        }

                        return Ok(Some(Value::Nil));
                    }

                    "grounded" => {
                        if let Some(character_system) = self.character_system.as_deref() {
                            if let Some(grounded) = character_system.is_entity_grounded(entity_id) {
                                return Ok(Some(Value::Bool(grounded)));
                            }
                        }

                        return Ok(Some(Value::Nil));
                    }

                    "animation" => {
                        if let Some(character_system) = self.character_system.as_deref() {
                            if let Some(animation) =
                                character_system.get_entity_animation(entity_id)
                            {
                                return Ok(Some(Value::String(animation.to_string())));
                            }
                        }

                        return Ok(Some(Value::Nil));
                    }

                    "health" => {
                        if let Some(character_system) = self.character_system.as_deref() {
                            if let Some(health) = character_system.get_entity_health(entity_id) {
                                return Ok(Some(Value::Number(health as f64)));
                            }
                        }

                        return Ok(Some(Value::Nil));
                    }

                    "max_health" => {
                        if let Some(character_system) = self.character_system.as_deref() {
                            if let Some(max_health) =
                                character_system.get_entity_max_health(entity_id)
                            {
                                return Ok(Some(Value::Number(max_health as f64)));
                            }
                        }

                        return Ok(Some(Value::Nil));
                    }

                    "alive" => {
                        if let Some(character_system) = self.character_system.as_deref() {
                            if let Some(alive) = character_system.is_entity_alive(entity_id) {
                                return Ok(Some(Value::Bool(alive)));
                            }
                        }

                        return Ok(Some(Value::Bool(true)));
                    }

                    _ => {}
                }
            }

            HandleKind::Sound => match name {
                "playing" => {
                    return Ok(Some(Value::Bool(
                        self.world.is_audio_playing(id) && !self.world.is_audio_paused(id),
                    )));
                }

                "looped" => {
                    return Ok(Some(Value::Bool(self.world.is_audio_looped(id))));
                }

                "volume" => {
                    return Ok(Some(Value::Number(self.world.get_audio_volume(id) as f64)));
                }

                _ => {}
            },

            HandleKind::Ui => {}
        }

        Ok(None)
    }

    pub fn set_property(
        &mut self,
        kind: HandleKind,
        id: u64,
        name: &str,
        value: Value,
    ) -> Result<bool, String> {
        match kind {
            HandleKind::Mouse => {
                if id != 0 {
                    return Err("Invalid Mouse handle".to_string());
                }

                match name {
                    "setCursorVisible" => {
                        self.mouse.cursor_visible = value.as_bool()?;
                        return Ok(true);
                    }

                    "setScreenLocked" => {
                        self.mouse.screen_locked = value.as_bool()?;
                        return Ok(true);
                    }

                    _ => {}
                }
            }

            HandleKind::Cell | HandleKind::Light => {
                if self.world.get_effective_cell_by_id(id).is_some() {
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

                            let color = Vec3::new(
                                borrowed.elements[0].as_number()? as f32,
                                borrowed.elements[1].as_number()? as f32,
                                borrowed.elements[2].as_number()? as f32,
                            );

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

                            let offset = Vec3::new(
                                borrowed.elements[0].as_number()? as f32,
                                borrowed.elements[1].as_number()? as f32,
                                borrowed.elements[2].as_number()? as f32,
                            );

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

            HandleKind::Entity => {
                let entity_id = EntityId(id);

                if !self.entity_manager.validate_handle(id) {
                    return Err("Invalid Entity handle".to_string());
                }

                match name {
                    "position" => {
                        let basket = value.as_basket()?;
                        let borrowed = basket.borrow();

                        if borrowed.elements.len() != 3 {
                            return Err("Entity.position must be [x, y, z]".to_string());
                        }

                        let position = Vec3::new(
                            borrowed.elements[0].as_number()? as f32,
                            borrowed.elements[1].as_number()? as f32,
                            borrowed.elements[2].as_number()? as f32,
                        );

                        self.set_position(id, position);

                        return Ok(true);
                    }

                    "velocity" => {
                        let basket = value.as_basket()?;
                        let borrowed = basket.borrow();

                        if borrowed.elements.len() != 3 {
                            return Err("Entity.velocity must be [x, y, z]".to_string());
                        }

                        let velocity = Vec3::new(
                            borrowed.elements[0].as_number()? as f32,
                            borrowed.elements[1].as_number()? as f32,
                            borrowed.elements[2].as_number()? as f32,
                        );

                        let Some(character_system) = self.character_system.as_deref_mut() else {
                            return Err("CharacterSystem not available".to_string());
                        };

                        if !character_system.set_entity_velocity(entity_id, velocity) {
                            return Err("Entity does not reference a runtime character".to_string());
                        }

                        return Ok(true);
                    }

                    "facing" => {
                        let basket = value.as_basket()?;
                        let borrowed = basket.borrow();

                        if borrowed.elements.len() != 2 {
                            return Err("Entity.facing must be [x, z]".to_string());
                        }

                        let x = borrowed.elements[0].as_number()? as f32;
                        let z = borrowed.elements[1].as_number()? as f32;

                        let Some(character_system) = self.character_system.as_deref_mut() else {
                            return Err("CharacterSystem not available".to_string());
                        };

                        if !character_system.set_entity_facing_direction(entity_id, x, z) {
                            return Err("Entity does not reference a runtime character".to_string());
                        }

                        return Ok(true);
                    }

                    "animation" => {
                        let animation = value.as_string()?;

                        let Some(character_system) = self.character_system.as_deref_mut() else {
                            return Err("CharacterSystem not available".to_string());
                        };

                        character_system.set_entity_animation(entity_id, animation)?;

                        return Ok(true);
                    }

                    "health" => {
                        let health = value.as_number()? as f32;

                        let Some(character_system) = self.character_system.as_deref_mut() else {
                            return Err("CharacterSystem not available".to_string());
                        };

                        if !character_system.set_entity_health(entity_id, health) {
                            return Err("Entity does not reference a runtime character".to_string());
                        }

                        return Ok(true);
                    }

                    "max_health" => {
                        let max_health = value.as_number()? as f32;

                        let Some(character_system) = self.character_system.as_deref_mut() else {
                            return Err("CharacterSystem not available".to_string());
                        };

                        if !character_system.set_entity_max_health(entity_id, max_health) {
                            return Err("Entity does not reference a runtime character".to_string());
                        }

                        return Ok(true);
                    }

                    "id" | "name" | "grounded" | "alive" => {
                        return Err(format!("Entity.{} is read-only", name));
                    }

                    _ => {}
                }
            }

            HandleKind::Sound => match name {
                "playing" => {
                    let playing = value.as_bool()?;
                    self.world.set_audio_playing_runtime(id, playing);
                    return Ok(true);
                }

                "looped" => {
                    let looped = value.as_bool()?;
                    self.world.set_audio_looped_runtime(id, looped);
                    return Ok(true);
                }

                "volume" => {
                    let vol = value.as_number()? as f32;
                    self.world.set_audio_volume_runtime(id, vol);
                    return Ok(true);
                }

                _ => {}
            },

            _ => {}
        }

        Ok(false)
    }

    pub fn call_method(
        &mut self,
        kind: HandleKind,
        id: u64,
        name: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, String> {
        if kind == HandleKind::Sound {
            match name {
                "play" => {
                    self.world.audio_play_runtime(id);
                    return Ok(Some(Value::Nil));
                }

                "stop" => {
                    self.world.audio_stop_runtime(id);
                    return Ok(Some(Value::Nil));
                }

                "pause" => {
                    self.world.audio_pause_runtime(id);
                    return Ok(Some(Value::Nil));
                }

                _ => {}
            }
        }

        match kind {
            HandleKind::Cell | HandleKind::Light => match name {
                "getObject" => {
                    if !arguments.is_empty() {
                        return Err(format!("{}.getObject() expects 0 arguments", kind.name()));
                    }

                    if let Some((kind, id)) = self.get_cell_object(id) {
                        Ok(Some(Value::Handle { kind, id }))
                    } else {
                        Ok(Some(Value::Nil))
                    }
                }

                "is_enabled" => {
                    if !arguments.is_empty() {
                        return Err(format!("{}.is_enabled() expects 0 arguments", kind.name()));
                    }

                    Ok(self
                        .is_light_enabled(id)
                        .map(Value::Bool)
                        .or(Some(Value::Nil)))
                }

                "set_enabled" => {
                    if arguments.len() != 1 {
                        return Err(format!(
                            "{}.set_enabled() expects 1 argument (bool)",
                            kind.name()
                        ));
                    }

                    let enabled = arguments[0].as_bool()?;
                    self.set_light_enabled(id, enabled);

                    Ok(Some(Value::Nil))
                }

                _ => Ok(None),
            },

            HandleKind::Entity => {
                let entity_id = EntityId(id);

                if !self.entity_manager.validate_handle(id) {
                    return Err("entity method called on an invalid handle".to_string());
                }

                match name {
                    "is_valid" => {
                        if !arguments.is_empty() {
                            return Err("entity.is_valid() expects 0 arguments".to_string());
                        }

                        Ok(Some(Value::Bool(self.entity_manager.validate_handle(id))))
                    }

                    "name" => {
                        if !arguments.is_empty() {
                            return Err("entity.name() expects 0 arguments".to_string());
                        }

                        if let Some(entity_name) = self.entity_manager.get_name(entity_id) {
                            Ok(Some(Value::String(entity_name.to_string())))
                        } else {
                            Ok(Some(Value::Nil))
                        }
                    }

                    "set_position" => {
                        if arguments.len() != 3 {
                            return Err(
                                "entity.set_position() expects 3 arguments (x, y, z)".to_string()
                            );
                        }

                        let x = arguments[0].as_number()? as f32;
                        let y = arguments[1].as_number()? as f32;
                        let z = arguments[2].as_number()? as f32;

                        self.set_position(id, Vec3::new(x, y, z));

                        Ok(Some(Value::Nil))
                    }

                    "translate" => {
                        if arguments.len() != 3 {
                            return Err(
                                "entity.translate() expects 3 arguments (x, y, z)".to_string()
                            );
                        }

                        let dx = arguments[0].as_number()? as f32;
                        let dy = arguments[1].as_number()? as f32;
                        let dz = arguments[2].as_number()? as f32;

                        if let Some(position) = self.get_position(id) {
                            self.set_position(id, position + Vec3::new(dx, dy, dz));
                        }

                        Ok(Some(Value::Nil))
                    }

                    "jump" => {
                        if arguments.len() > 1 {
                            return Err(
                                "entity.jump() expects 0 or 1 argument (impulse)".to_string()
                            );
                        }

                        let impulse = if arguments.is_empty() {
                            crate::character::movement::JUMP_IMPULSE
                        } else {
                            arguments[0].as_number()? as f32
                        };

                        let Some(character_system) = self.character_system.as_deref_mut() else {
                            return Err("CharacterSystem not available".to_string());
                        };

                        if !character_system.jump_entity(entity_id, impulse) {
                            return Err("Entity does not reference a runtime character".to_string());
                        }

                        Ok(Some(Value::Nil))
                    }

                    "damage" => {
                        if arguments.len() != 1 {
                            return Err("entity.damage() expects 1 argument (amount)".to_string());
                        }

                        let amount = arguments[0].as_number()? as f32;

                        let Some(character_system) = self.character_system.as_deref_mut() else {
                            return Err("CharacterSystem not available".to_string());
                        };

                        let Some(health) = character_system.damage_entity(entity_id, amount) else {
                            return Err("Entity does not reference a runtime character".to_string());
                        };

                        Ok(Some(Value::Number(health as f64)))
                    }

                    "heal" => {
                        if arguments.len() != 1 {
                            return Err("entity.heal() expects 1 argument (amount)".to_string());
                        }

                        let amount = arguments[0].as_number()? as f32;

                        let Some(character_system) = self.character_system.as_deref_mut() else {
                            return Err("CharacterSystem not available".to_string());
                        };

                        let Some(health) = character_system.heal_entity(entity_id, amount) else {
                            return Err("Entity does not reference a runtime character".to_string());
                        };

                        Ok(Some(Value::Number(health as f64)))
                    }

                    "destroy" => {
                        if !arguments.is_empty() {
                            return Err("entity.destroy() expects 0 arguments".to_string());
                        }

                        if let Some(character_system) = self.character_system.as_deref_mut() {
                            character_system.destroy_entity(entity_id);
                        }

                        self.pending_spawns
                            .retain(|(_, pending_id)| *pending_id != id);

                        self.entity_manager.remove_entity(entity_id);

                        Ok(Some(Value::Bool(true)))
                    }

                    "find_path" => {
                        if arguments.len() != 1 {
                            return Err(
                                "entity.find_path() expects 1 argument ([x, y, z])".to_string()
                            );
                        }

                        let basket = arguments[0].as_basket()?;
                        let borrowed = basket.borrow();

                        if borrowed.elements.len() != 3 {
                            return Err("entity.find_path() target must be [x, y, z]".to_string());
                        }

                        let target = Vec3::new(
                            borrowed.elements[0].as_number()? as f32,
                            borrowed.elements[1].as_number()? as f32,
                            borrowed.elements[2].as_number()? as f32,
                        );

                        let Some(character_system) = self.character_system.as_deref() else {
                            return Err("CharacterSystem not available".to_string());
                        };

                        let Some(path) =
                            character_system.find_path_for_entity(entity_id, self.world, target)
                        else {
                            return Ok(Some(Value::Nil));
                        };

                        let points = path
                            .into_iter()
                            .map(|point| {
                                Value::array(vec![
                                    Value::Number(point.x as f64),
                                    Value::Number(point.y as f64),
                                    Value::Number(point.z as f64),
                                ])
                            })
                            .collect();

                        Ok(Some(Value::array(points)))
                    }

                    _ => Ok(None),
                }
            }

            HandleKind::Ui | HandleKind::Sound | HandleKind::Mouse => Ok(None),
        }
    }

    pub fn get_script_property(&self, kind: HandleKind, id: u64, name: &str) -> Option<Value> {
        self.dynamic_properties
            .get(&(kind, id))
            .and_then(|properties| properties.get(name))
            .cloned()
    }

    pub fn set_script_property(&mut self, kind: HandleKind, id: u64, name: String, value: Value) {
        self.dynamic_properties
            .entry((kind, id))
            .or_default()
            .insert(name, value);
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

    fn get_viewport_size(&self) -> [f32; 2] {
        self.viewport_size
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
        ScriptHostBridge::get_position(self, id)
    }

    fn set_position(&mut self, id: u64, position: Vec3) {
        ScriptHostBridge::set_position(self, id, position);
    }

    fn is_entity_declaration_valid(&self, entity_name: &str) -> bool {
        ScriptHostBridge::is_entity_declaration_valid(self, entity_name)
    }

    fn spawn_character(&mut self, entity_name: &str, position: Vec3) -> Result<u64, String> {
        ScriptHostBridge::spawn_character(self, entity_name, position)
    }

    fn set_valid_entity_declarations(&mut self, decls: Arc<std::collections::HashSet<String>>) {
        ScriptHostBridge::set_valid_entity_declarations(self, decls)
    }

    fn drain_pending_spawns(&mut self) -> Vec<(String, u64)> {
        ScriptHostBridge::drain_pending_spawns(self)
    }

    fn lookup_light(&self, x: i32, y: i32, z: i32) -> Option<u64> {
        ScriptHostBridge::lookup_light(self, x, y, z)
    }

    fn is_light_enabled(&self, id: u64) -> Option<bool> {
        ScriptHostBridge::is_light_enabled(self, id)
    }

    fn set_light_enabled(&mut self, id: u64, enabled: bool) {
        ScriptHostBridge::set_light_enabled(self, id, enabled);
    }

    fn is_collision_events_enabled(&self, id: u64) -> bool {
        ScriptHostBridge::is_collision_events_enabled(self, id)
    }

    fn create_runtime_cell(&mut self, cell_type: &str) -> Result<(HandleKind, u64), String> {
        ScriptHostBridge::create_runtime_cell(self, cell_type)
    }

    fn move_runtime_cell(&mut self, id: u64, x: i32, y: i32, z: i32) -> Result<(), String> {
        ScriptHostBridge::move_runtime_cell(self, id, x, y, z)
    }

    fn delete_cell(&mut self, id: u64) -> Result<(), String> {
        ScriptHostBridge::delete_cell(self, id)
    }

    fn get_all_cells_of_class(&self, class_name: &str) -> Vec<u64> {
        ScriptHostBridge::get_all_cells_of_class(self, class_name)
    }

    fn find_objects(&self, query: &str) -> Vec<(HandleKind, u64)> {
        ScriptHostBridge::find_objects(self, query)
    }

    fn get_children(&self, kind: HandleKind, id: u64) -> Vec<(HandleKind, u64)> {
        ScriptHostBridge::get_children(self, kind, id)
    }

    fn get_parent(&self, kind: HandleKind, id: u64) -> Option<(HandleKind, u64)> {
        ScriptHostBridge::get_parent(self, kind, id)
    }

    fn get_cell_object(&self, cell_id: u64) -> Option<(HandleKind, u64)> {
        ScriptHostBridge::get_cell_object(self, cell_id)
    }

    fn get_property(&self, kind: HandleKind, id: u64, name: &str) -> Result<Option<Value>, String> {
        ScriptHostBridge::get_property(self, kind, id, name)
    }

    fn set_property(
        &mut self,
        kind: HandleKind,
        id: u64,
        name: &str,
        value: Value,
    ) -> Result<bool, String> {
        ScriptHostBridge::set_property(self, kind, id, name, value)
    }

    fn call_method(
        &mut self,
        kind: HandleKind,
        id: u64,
        name: &str,
        arguments: &[Value],
    ) -> Result<Option<Value>, String> {
        ScriptHostBridge::call_method(self, kind, id, name, arguments)
    }

    fn is_audio_playing(&self, id: u64) -> Option<bool> {
        ScriptHostBridge::is_audio_playing(self, id)
    }

    fn is_audio_paused(&self, id: u64) -> Option<bool> {
        ScriptHostBridge::is_audio_paused(self, id)
    }

    fn is_audio_looped(&self, id: u64) -> Option<bool> {
        ScriptHostBridge::is_audio_looped(self, id)
    }

    fn get_audio_volume(&self, id: u64) -> Option<f32> {
        ScriptHostBridge::get_audio_volume(self, id)
    }

    fn set_audio_playing(&mut self, id: u64, playing: bool) {
        ScriptHostBridge::set_audio_playing(self, id, playing);
    }

    fn set_audio_looped(&mut self, id: u64, looped: bool) {
        ScriptHostBridge::set_audio_looped(self, id, looped);
    }

    fn set_audio_volume(&mut self, id: u64, volume: f32) {
        ScriptHostBridge::set_audio_volume(self, id, volume);
    }

    fn audio_play(&mut self, id: u64) {
        ScriptHostBridge::audio_play(self, id);
    }

    fn audio_stop(&mut self, id: u64) {
        ScriptHostBridge::audio_stop(self, id);
    }

    fn audio_pause(&mut self, id: u64) {
        ScriptHostBridge::audio_pause(self, id);
    }

    fn set_attribute(&mut self, id: u64, key: String, value: Value) -> Result<(), String> {
        ScriptHostBridge::set_attribute(self, id, key, value)
    }

    fn remove_attribute(&mut self, id: u64, key: &str) -> Result<(), String> {
        ScriptHostBridge::remove_attribute(self, id, key)
    }

    fn cell_exists(&self, id: u64) -> bool {
        ScriptHostBridge::cell_exists(self, id)
    }

    fn entity_exists(&self, id: u64) -> bool {
        ScriptHostBridge::entity_exists(self, id)
    }

    fn get_script_property(&self, kind: HandleKind, id: u64, name: &str) -> Option<Value> {
        ScriptHostBridge::get_script_property(self, kind, id, name)
    }

    fn set_script_property(&mut self, kind: HandleKind, id: u64, name: String, value: Value) {
        ScriptHostBridge::set_script_property(self, kind, id, name, value);
    }

    fn create_ui_element(&mut self, element_type: &str) -> Result<u64, String> {
        ScriptHostBridge::create_ui_element(self, element_type)
    }

    fn delete_ui_element(&mut self, id: u64) -> Result<(), String> {
        ScriptHostBridge::delete_ui_element(self, id)
    }

    fn get_ui_property(&self, id: u64, name: &str) -> Result<Option<Value>, String> {
        ScriptHostBridge::get_ui_property(self, id, name)
    }

    fn set_ui_property(&mut self, id: u64, name: &str, value: Value) -> Result<bool, String> {
        ScriptHostBridge::set_ui_property(self, id, name, value)
    }

    fn drain_ui_clicks(&mut self) -> Vec<u64> {
        ScriptHostBridge::drain_ui_clicks(self)
    }

    fn get_input_move_vector(&self) -> [f32; 2] {
        ScriptHostBridge::get_input_move_vector(self)
    }

    fn is_input_jump_pressed(&self) -> bool {
        ScriptHostBridge::is_input_jump_pressed(self)
    }

    fn get_input_orbit_delta(&self) -> [f32; 2] {
        ScriptHostBridge::get_input_orbit_delta(self)
    }

    fn get_camera_horizontal_basis(&self) -> ([f32; 3], [f32; 3]) {
        ScriptHostBridge::get_camera_horizontal_basis(self)
    }

    fn set_camera_position(&mut self, position: [f32; 3]) {
        ScriptHostBridge::set_camera_position(self, position);
    }

    fn set_camera_target(&mut self, target: [f32; 3]) {
        ScriptHostBridge::set_camera_target(self, target);
    }

    fn set_camera_orientation(&mut self, yaw: f32, pitch: f32) {
        ScriptHostBridge::set_camera_orientation(self, yaw, pitch);
    }

    fn resolve_camera_collision(&self, target: [f32; 3], desired: [f32; 3]) -> [f32; 3] {
        ScriptHostBridge::resolve_camera_collision(self, target, desired)
    }

    fn get_player_position(&self) -> Option<[f32; 3]> {
        ScriptHostBridge::get_player_position(self)
    }

    fn set_player_horizontal_velocity(&mut self, velocity_x: f32, velocity_z: f32) {
        ScriptHostBridge::set_player_horizontal_velocity(self, velocity_x, velocity_z);
    }

    fn set_player_facing_direction(&mut self, direction_x: f32, direction_z: f32) {
        ScriptHostBridge::set_player_facing_direction(self, direction_x, direction_z);
    }

    fn select_player_animation(&mut self, animation: &str) {
        ScriptHostBridge::select_player_animation(self, animation);
    }

    fn is_player_grounded(&self) -> bool {
        ScriptHostBridge::is_player_grounded(self)
    }

    fn apply_player_vertical_impulse(&mut self, impulse: f32) {
        ScriptHostBridge::apply_player_vertical_impulse(self, impulse);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::CharacterSystem;
    use crate::scripting::api::EngineHost;
    use crate::scripting::value::{HandleKind, Value};
    use crate::world::WorldCoord;
    use crate::world::cell::{AttributeValue, CellType};

    fn make_bridge<'a>(
        entity_manager: &'a mut EntityManager,
        world: &'a mut World,
        mouse: &'a mut MouseController,
        dynamic_properties: &'a mut std::collections::HashMap<
            (HandleKind, u64),
            std::collections::BTreeMap<String, Value>,
        >,
        pending_events: &'a mut Vec<(String, Vec<Value>)>,
        test_results: &'a mut std::collections::BTreeMap<String, bool>,
        pending_enable_scripts: &'a mut Vec<String>,
        pending_disable_scripts: &'a mut Vec<String>,
        runtime_ui: &'a mut crate::engine::ui::RuntimeUi,
        pending_spawns: &'a mut Vec<(String, u64)>,
        character_system: Option<&'a mut CharacterSystem>,
    ) -> ScriptHostBridge<'a> {
        ScriptHostBridge {
            entity_manager,
            world,
            mouse,
            dynamic_properties,
            pending_events,
            test_results,
            pending_enable_scripts,
            pending_disable_scripts,
            runtime_ui,
            viewport_size: [1280.0, 720.0],
            move_input: glam::Vec2::ZERO,
            jump_requested: false,
            orbit_delta: [0.0, 0.0],
            character_system,
            gameplay_camera: None,
            valid_entity_declarations: None,
            pending_spawns,
            project_path: None,
        }
    }

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
        let mut mouse = MouseController::default();
        let mut pending_spawns = Vec::new();

        let bridge = make_bridge(
            &mut entity_manager,
            &mut world,
            &mut mouse,
            &mut dynamic_properties,
            &mut pending_events,
            &mut test_results,
            &mut pending_enable_scripts,
            &mut pending_disable_scripts,
            &mut runtime_ui,
            &mut pending_spawns,
            None,
        );

        let value = bridge
            .get_property(HandleKind::Cell, cell_id, "attributes")
            .unwrap()
            .expect("attributes property should exist");

        let map = value.as_map().unwrap();
        let map = map.borrow();

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
        let mut mouse = MouseController::default();
        let mut pending_spawns = Vec::new();

        let mut bridge = make_bridge(
            &mut entity_manager,
            &mut world,
            &mut mouse,
            &mut dynamic_properties,
            &mut pending_events,
            &mut test_results,
            &mut pending_enable_scripts,
            &mut pending_disable_scripts,
            &mut runtime_ui,
            &mut pending_spawns,
            None,
        );

        let attrs = bridge
            .get_property(HandleKind::Cell, cell_id, "attributes")
            .unwrap()
            .unwrap();

        let map = attrs.as_map().unwrap();

        assert_eq!(
            map.borrow().get(&MapKey::String("test".to_string())),
            Some(&Value::String("authored".to_string()))
        );

        bridge
            .set_attribute(
                cell_id,
                "test".to_string(),
                Value::String("runtime".to_string()),
            )
            .unwrap();

        let attrs = bridge
            .get_property(HandleKind::Cell, cell_id, "attributes")
            .unwrap()
            .unwrap();

        let map = attrs.as_map().unwrap();

        assert_eq!(
            map.borrow().get(&MapKey::String("test".to_string())),
            Some(&Value::String("runtime".to_string()))
        );

        assert_eq!(
            bridge.world.get(coord).unwrap().attributes.get("test"),
            Some(&AttributeValue::String("authored".to_string()))
        );

        bridge.remove_attribute(cell_id, "test").unwrap();

        let attrs = bridge
            .get_property(HandleKind::Cell, cell_id, "attributes")
            .unwrap()
            .unwrap();

        let map = attrs.as_map().unwrap();

        assert_eq!(
            map.borrow().get(&MapKey::String("test".to_string())),
            Some(&Value::String("authored".to_string()))
        );
    }

    #[test]
    fn test_player_set_position_updates_character_once() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();
        let mut character_system = CharacterSystem::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        let player_char_id = character_system
            .spawn_player(&world, None)
            .expect("player should spawn");

        let player_id = entity_manager.create_entity("Player");
        character_system.associate_entity(player_id, player_char_id);

        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui = crate::engine::ui::RuntimeUi::new();
        let mut mouse = MouseController::default();
        let mut pending_spawns = Vec::new();

        let mut bridge = make_bridge(
            &mut entity_manager,
            &mut world,
            &mut mouse,
            &mut dynamic_properties,
            &mut pending_events,
            &mut test_results,
            &mut pending_enable_scripts,
            &mut pending_disable_scripts,
            &mut runtime_ui,
            &mut pending_spawns,
            Some(&mut character_system),
        );

        let target = Vec3::new(10.0, 20.0, 30.0);

        bridge.set_position(player_id.0, target);

        assert_eq!(bridge.entity_manager.get_position(player_id), Some(target));

        assert_eq!(
            bridge.character_system.as_deref().and_then(|system| {
                system
                    .get_active_player()
                    .map(|player| player.transform.position)
            }),
            Some(target)
        );
    }

    #[test]
    fn test_non_player_set_position_does_not_move_character() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();
        let mut character_system = CharacterSystem::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        character_system
            .spawn_player(&world, None)
            .expect("player should spawn");

        let original_position = character_system
            .get_active_player()
            .unwrap()
            .transform
            .position;

        let entity_id = entity_manager.create_entity("SomethingElse");

        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui = crate::engine::ui::RuntimeUi::new();
        let mut mouse = MouseController::default();
        let mut pending_spawns = Vec::new();

        let mut bridge = make_bridge(
            &mut entity_manager,
            &mut world,
            &mut mouse,
            &mut dynamic_properties,
            &mut pending_events,
            &mut test_results,
            &mut pending_enable_scripts,
            &mut pending_disable_scripts,
            &mut runtime_ui,
            &mut pending_spawns,
            Some(&mut character_system),
        );

        bridge.set_position(entity_id.0, Vec3::new(100.0, 100.0, 100.0));

        assert_eq!(
            bridge.character_system.as_deref().and_then(|system| {
                system
                    .get_active_player()
                    .map(|player| player.transform.position)
            }),
            Some(original_position)
        );
    }

    #[test]
    fn test_mouse_controller_script_properties() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();
        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui = crate::engine::ui::RuntimeUi::new();
        let mut mouse = MouseController::default();
        let mut pending_spawns = Vec::new();

        let mut bridge = make_bridge(
            &mut entity_manager,
            &mut world,
            &mut mouse,
            &mut dynamic_properties,
            &mut pending_events,
            &mut test_results,
            &mut pending_enable_scripts,
            &mut pending_disable_scripts,
            &mut runtime_ui,
            &mut pending_spawns,
            None,
        );

        bridge.mouse.set_play_defaults();

        assert_eq!(bridge.mouse.cursor_visible, false);
        assert_eq!(bridge.mouse.screen_locked, true);

        assert_eq!(
            bridge
                .get_property(HandleKind::Mouse, 0, "setCursorVisible")
                .unwrap(),
            Some(Value::Bool(false))
        );

        assert_eq!(
            bridge
                .get_property(HandleKind::Mouse, 0, "setScreenLocked")
                .unwrap(),
            Some(Value::Bool(true))
        );

        assert!(
            bridge
                .set_property(HandleKind::Mouse, 0, "setCursorVisible", Value::Bool(true),)
                .unwrap()
        );

        assert!(
            bridge
                .set_property(HandleKind::Mouse, 0, "setScreenLocked", Value::Bool(false),)
                .unwrap()
        );

        assert_eq!(bridge.mouse.cursor_visible, true);
        assert_eq!(bridge.mouse.screen_locked, false);
    }

    #[test]
    fn test_entity_runtime_properties() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();
        let mut character_system = CharacterSystem::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        let character_id = character_system
            .spawn_player(&world, None)
            .expect("character should spawn");

        let entity_id = entity_manager.create_entity("Villager");
        entity_manager.set_position(entity_id, Vec3::new(1.0, 2.0, 3.0));

        character_system.associate_entity(entity_id, character_id);

        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui = crate::engine::ui::RuntimeUi::new();
        let mut mouse = MouseController::default();
        let mut pending_spawns = Vec::new();

        let mut bridge = make_bridge(
            &mut entity_manager,
            &mut world,
            &mut mouse,
            &mut dynamic_properties,
            &mut pending_events,
            &mut test_results,
            &mut pending_enable_scripts,
            &mut pending_disable_scripts,
            &mut runtime_ui,
            &mut pending_spawns,
            Some(&mut character_system),
        );

        let id = bridge
            .get_property(HandleKind::Entity, entity_id.0, "id")
            .unwrap();

        assert_eq!(id, Some(Value::Number(entity_id.0 as f64)));

        let name = bridge
            .get_property(HandleKind::Entity, entity_id.0, "name")
            .unwrap();

        assert_eq!(name, Some(Value::String("Villager".to_string())));

        let position = bridge
            .get_property(HandleKind::Entity, entity_id.0, "position")
            .unwrap()
            .unwrap();

        let position = position.as_basket().unwrap();
        let position = position.borrow();

        assert_eq!(position.elements.len(), 3);

        let velocity = bridge
            .get_property(HandleKind::Entity, entity_id.0, "velocity")
            .unwrap()
            .unwrap();

        let velocity = velocity.as_basket().unwrap();
        let velocity = velocity.borrow();

        assert_eq!(velocity.elements.len(), 3);

        let grounded = bridge
            .get_property(HandleKind::Entity, entity_id.0, "grounded")
            .unwrap();

        assert!(grounded.is_some());

        let animation = bridge
            .get_property(HandleKind::Entity, entity_id.0, "animation")
            .unwrap();

        assert!(animation.is_some());
    }

    #[test]
    fn test_entity_velocity_facing_and_health_controls() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();
        let mut character_system = CharacterSystem::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        let character_id = character_system
            .spawn_player(&world, None)
            .expect("character should spawn");

        let entity_id = entity_manager.create_entity("Villager");

        character_system.associate_entity(entity_id, character_id);

        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui = crate::engine::ui::RuntimeUi::new();
        let mut mouse = MouseController::default();
        let mut pending_spawns = Vec::new();

        let mut bridge = make_bridge(
            &mut entity_manager,
            &mut world,
            &mut mouse,
            &mut dynamic_properties,
            &mut pending_events,
            &mut test_results,
            &mut pending_enable_scripts,
            &mut pending_disable_scripts,
            &mut runtime_ui,
            &mut pending_spawns,
            Some(&mut character_system),
        );

        assert!(
            bridge
                .set_property(
                    HandleKind::Entity,
                    entity_id.0,
                    "velocity",
                    Value::array(vec![
                        Value::Number(2.0),
                        Value::Number(0.0),
                        Value::Number(-3.0),
                    ]),
                )
                .unwrap()
        );

        assert!(
            bridge
                .set_property(
                    HandleKind::Entity,
                    entity_id.0,
                    "facing",
                    Value::array(vec![Value::Number(1.0), Value::Number(0.0),]),
                )
                .unwrap()
        );

        assert!(
            bridge
                .set_property(
                    HandleKind::Entity,
                    entity_id.0,
                    "health",
                    Value::Number(75.0),
                )
                .unwrap()
        );

        let damage = bridge
            .call_method(
                HandleKind::Entity,
                entity_id.0,
                "damage",
                &[Value::Number(10.0)],
            )
            .unwrap()
            .unwrap();

        assert_eq!(damage, Value::Number(65.0));

        let health = bridge
            .get_property(HandleKind::Entity, entity_id.0, "health")
            .unwrap()
            .unwrap();

        assert_eq!(health, Value::Number(65.0));
    }

    #[test]
    fn test_entity_jump_and_destroy() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();
        let mut character_system = CharacterSystem::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        let character_id = character_system
            .spawn_player(&world, None)
            .expect("character should spawn");

        let entity_id = entity_manager.create_entity("Villager");

        character_system.associate_entity(entity_id, character_id);

        if let Some(character) = character_system
            .get_active_characters_mut()
            .find(|character| character.id == character_id)
        {
            character.movement.is_grounded = true;
        }

        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui = crate::engine::ui::RuntimeUi::new();
        let mut mouse = MouseController::default();
        let mut pending_spawns = Vec::new();

        let mut bridge = make_bridge(
            &mut entity_manager,
            &mut world,
            &mut mouse,
            &mut dynamic_properties,
            &mut pending_events,
            &mut test_results,
            &mut pending_enable_scripts,
            &mut pending_disable_scripts,
            &mut runtime_ui,
            &mut pending_spawns,
            Some(&mut character_system),
        );

        let jump = bridge
            .call_method(
                HandleKind::Entity,
                entity_id.0,
                "jump",
                &[Value::Number(8.0)],
            )
            .unwrap();

        assert_eq!(jump, Some(Value::Nil));

        let velocity = bridge
            .get_property(HandleKind::Entity, entity_id.0, "velocity")
            .unwrap()
            .unwrap();

        let velocity = velocity.as_basket().unwrap();
        let velocity = velocity.borrow();

        assert_eq!(velocity.elements[1], Value::Number(8.0));

        let destroyed = bridge
            .call_method(HandleKind::Entity, entity_id.0, "destroy", &[])
            .unwrap();

        assert_eq!(destroyed, Some(Value::Bool(true)));

        assert!(!bridge.entity_manager.validate_handle(entity_id.0));
    }

    #[test]
    fn test_entity_find_path_dispatch() {
        let mut world = World::new();

        for x in 0..4 {
            world.set_cell(WorldCoord::new(x, 0, 0), CellType::Block);
        }

        let mut entity_manager = EntityManager::new();
        let mut character_system = CharacterSystem::new();

        let character_id = character_system.spawn_character(Vec3::new(0.0, 1.0, 0.0), None);

        let entity_id = entity_manager.create_entity("Villager");

        character_system.associate_entity(entity_id, character_id);

        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui = crate::engine::ui::RuntimeUi::new();
        let mut mouse = MouseController::default();
        let mut pending_spawns = Vec::new();

        let mut bridge = make_bridge(
            &mut entity_manager,
            &mut world,
            &mut mouse,
            &mut dynamic_properties,
            &mut pending_events,
            &mut test_results,
            &mut pending_enable_scripts,
            &mut pending_disable_scripts,
            &mut runtime_ui,
            &mut pending_spawns,
            Some(&mut character_system),
        );

        let result = bridge
            .call_method(
                HandleKind::Entity,
                entity_id.0,
                "find_path",
                &[Value::array(vec![
                    Value::Number(3.0),
                    Value::Number(1.0),
                    Value::Number(0.0),
                ])],
            )
            .unwrap();

        assert!(result.is_some());
        assert!(!matches!(result, Some(Value::Nil)));
    }
}
