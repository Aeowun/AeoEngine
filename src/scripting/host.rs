use crate::engine::entity::{EntityId, EntityManager};
use crate::scripting::api::EngineHost;
use crate::scripting::value::{HandleKind, MapKey, Value};
use crate::world::cell::AttributeValue;
use crate::world::{CellType, World, WorldCoord};
use glam::Vec3;
use std::collections::BTreeMap;

/// Runtime bridge between AeoScript and the live engine.
///
/// The bridge owns references to the systems that scripts are allowed to
/// interact with during a particular script execution phase.
///
/// Important ownership rule:
/// - EntityManager stores the script-facing entity state.
/// - CharacterSystem owns the authoritative runtime player movement state.
/// - `set_position()` is the explicit bridge between those two systems for
///   intentional player teleports.
pub struct ScriptHostBridge<'a> {
    pub entity_manager: &'a mut EntityManager,
    pub world: &'a mut World,

    pub dynamic_properties:
        &'a mut std::collections::HashMap<
            (HandleKind, u64),
            std::collections::BTreeMap<String, Value>,
        >,

    pub pending_events: &'a mut Vec<(String, Vec<Value>)>,

    pub test_results:
        &'a mut std::collections::BTreeMap<String, bool>,

    pub pending_enable_scripts: &'a mut Vec<String>,
    pub pending_disable_scripts: &'a mut Vec<String>,

    pub runtime_ui: &'a mut crate::engine::ui::RuntimeUi,

    pub viewport_size: [f32; 2],

    /// Raw movement input exposed to controller objects.
    ///
    /// Controllers interpret this input themselves. This keeps the scripting
    /// API generic instead of baking a particular movement model into it.
    pub move_input: glam::Vec2,

    /// One-frame jump request exposed to controller objects.
    pub jump_requested: bool,

    /// Mouse orbit delta exposed to scripted camera/controller objects.
    pub orbit_delta: [f32; 2],

    /// Runtime character system used by player-specific script APIs.
    pub character_system:
        Option<&'a mut crate::character::CharacterSystem>,

    /// Runtime gameplay camera used by scripted camera objects.
    pub gameplay_camera:
        Option<&'a mut crate::renderer::camera::GameplayCamera>,
}

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
        self.pending_events
            .push((event_name.to_string(), args));
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

    /// Handles the existing generic AeoScript `entity.set_position()` API.
    ///
    /// This is the ONLY place where a script-requested entity position change
    /// is forwarded into CharacterSystem.
    ///
    /// Normal character movement never comes through this function.
    fn set_position(&mut self, id: u64, position: Vec3) {
        let entity_id = EntityId(id);

        // Determine whether this is the runtime player before taking the
        // mutable EntityManager borrow below.
        let is_player = self
            .entity_manager
            .get_name(entity_id)
            .map(|name| name == "Player")
            .unwrap_or(false);

        // Always update the script-facing entity representation.
        self.entity_manager.set_position(entity_id, position);

        // A Player entity explicitly calling set_position() is a deliberate
        // teleport request. Forward that single operation to the authoritative
        // CharacterSystem representation.
        if is_player {
            if let Some(character_system) =
                self.character_system.as_deref_mut()
            {
                character_system.set_active_position(position);
            }
        }
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
        self.world
            .resolve_cell_id(id)
            .map(|coord| self.world.is_light_enabled(coord))
    }

    fn set_light_enabled(&mut self, id: u64, enabled: bool) {
        if let Some(coord) = self.world.resolve_cell_id(id) {
            self.world.set_light_enabled_runtime(coord, enabled);
        }
    }

    fn is_collision_events_enabled(&self, id: u64) -> bool {
        if let Some(coord) = self.world.resolve_cell_id(id) {
            if let Some(runtime_state) =
                self.world.runtime_state.get(&id)
            {
                if let Some(enabled) =
                    runtime_state.collision_events_enabled
                {
                    return enabled;
                }
            }

            if let Some(cell) =
                self.world.get_effective_cell(coord)
            {
                return cell.collision_events_enabled;
            }
        }

        true
    }

    fn create_runtime_cell(
        &mut self,
        cell_type: &str,
    ) -> Result<(HandleKind, u64), String> {
        let cell_type = match cell_type {
            "Block" => CellType::Block,
            "FxBlock" => CellType::FxBlock,
            "Player" => CellType::Player,
            "NPC" => CellType::NPC,
            "Light" => CellType::Light,
            "SpawnPoint" => CellType::SpawnPoint,
            "AudioEmitter" => CellType::AudioEmitter,
            "Empty" => {
                return Err(
                    "Cannot create Empty cell".to_string()
                );
            }
            _ => {
                return Err(format!(
                    "Unknown cell type: {}",
                    cell_type
                ));
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

    fn move_runtime_cell(
        &mut self,
        id: u64,
        x: i32,
        y: i32,
        z: i32,
    ) -> Result<(), String> {
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
                    "AudioEmitter" => cell.cell_type == CellType::AudioEmitter,
                    "Block" => cell.cell_type == CellType::Block,
                    "FxBlock" => cell.cell_type == CellType::FxBlock,
                    "SpawnPoint" => {
                        cell.cell_type == CellType::SpawnPoint
                    }
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

        // Search authored/runtime cells by their script-facing identity.
        for coord in self.world.active_effective_blocks() {
            if let Some(cell) = self.world.get_effective_cell(coord) {
                if let Some(identity) = &cell.entity_identity {
                    if identity == query {
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

    fn get_children(
        &self,
        _kind: HandleKind,
        _id: u64,
    ) -> Vec<(HandleKind, u64)> {
        Vec::new()
    }

    fn get_parent(
        &self,
        _kind: HandleKind,
        _id: u64,
    ) -> Option<(HandleKind, u64)> {
        None
    }

    fn get_cell_object(
        &self,
        cell_id: u64,
    ) -> Option<(HandleKind, u64)> {
        let coord = self.world.resolve_cell_id(cell_id)?;

        let cell = self.world.get_effective_cell(coord)?;

        let identity = cell.entity_identity.as_ref()?;

        let entity_id =
            self.entity_manager.lookup_entity(identity)?;

        Some((HandleKind::Entity, entity_id.0))
    }

    fn get_property(
        &self,
        kind: HandleKind,
        id: u64,
        name: &str,
    ) -> Result<Option<Value>, String> {
        match kind {
            HandleKind::Cell | HandleKind::Light => {
                if let Some(cell) =
                    self.world.get_effective_cell_by_id(id)
                {
                    match name {
                        "sound" => {
                            return Ok(Some(Value::Handle {
                                kind: HandleKind::Sound,
                                id,
                            }));
                        }
                        "id" => {
                            return Ok(Some(Value::Number(
                                cell.id as f64,
                            )));
                        }

                        "name" => {
                            return Ok(Some(Value::String(
                                cell.entity_identity
                                    .clone()
                                    .unwrap_or_else(|| {
                                        "Cell".to_string()
                                    }),
                            )));
                        }

                        "cellType" => {
                            return Ok(Some(Value::String(
                                format!("{:?}", cell.cell_type),
                            )));
                        }

                        "position" => {
                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                return Ok(Some(Value::array(vec![
                                    Value::Number(coord.x as f64),
                                    Value::Number(coord.y as f64),
                                    Value::Number(coord.z as f64),
                                ])));
                            }

                            return Ok(Some(Value::Nil));
                        }

                        "visible" => {
                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                return Ok(Some(Value::Bool(
                                    self.world.is_cell_visible(coord),
                                )));
                            }

                            if let Some(runtime_state) =
                                self.world.runtime_state.get(&id)
                            {
                                if let Some(value) =
                                    runtime_state.visible
                                {
                                    return Ok(Some(Value::Bool(value)));
                                }
                            }

                            return Ok(Some(Value::Bool(cell.visible)));
                        }

                        "enabled" => {
                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                return Ok(Some(Value::Bool(
                                    self.world
                                        .is_light_enabled(coord),
                                )));
                            }

                            if let Some(runtime_state) =
                                self.world.runtime_state.get(&id)
                            {
                                if let Some(value) =
                                    runtime_state.light_enabled
                                {
                                    return Ok(Some(Value::Bool(value)));
                                }
                            }

                            return Ok(Some(Value::Bool(
                                cell.light_enabled,
                            )));
                        }

                        "solid" => {
                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                return Ok(Some(Value::Bool(
                                    self.world.is_cell_solid(coord),
                                )));
                            }

                            if let Some(runtime_state) =
                                self.world.runtime_state.get(&id)
                            {
                                if let Some(value) =
                                    runtime_state.solid
                                {
                                    return Ok(Some(Value::Bool(value)));
                                }
                            }

                            return Ok(Some(Value::Bool(cell.solid)));
                        }

                        "anchored" => {
                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                return Ok(Some(Value::Bool(
                                    self.world
                                        .is_cell_anchored(coord),
                                )));
                            }

                            if let Some(runtime_state) =
                                self.world.runtime_state.get(&id)
                            {
                                if let Some(value) =
                                    runtime_state.anchored
                                {
                                    return Ok(Some(Value::Bool(value)));
                                }
                            }

                            return Ok(Some(Value::Bool(
                                cell.anchored,
                            )));
                        }

                        "color" => {
                            let color = if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                self.world.get_effective_color(coord)
                            } else if let Some(runtime_state) =
                                self.world.runtime_state.get(&id)
                            {
                                runtime_state
                                    .color_rgb
                                    .unwrap_or(cell.color_rgb)
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
                            let offset = if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                self.world.get_visual_offset(coord)
                            } else if let Some(runtime_state) =
                                self.world.runtime_state.get(&id)
                            {
                                runtime_state
                                    .visual_offset
                                    .unwrap_or(Vec3::ZERO)
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
                            for (key, attribute) in
                                &cell.attributes
                            {
                                let value = match attribute {
                                    AttributeValue::Number(n) => {
                                        Value::Number(*n)
                                    }
                                    AttributeValue::Bool(b) => {
                                        Value::Bool(*b)
                                    }
                                    AttributeValue::String(s) => {
                                        Value::String(s.clone())
                                    }
                                };

                                map.insert(
                                    MapKey::String(key.clone()),
                                    value,
                                );
                            }

                            // Runtime overrides replace authored values.
                            if let Some(runtime_state) =
                                self.world.runtime_state.get(&id)
                            {
                                for (key, attribute) in
                                    &runtime_state.attribute_overrides
                                {
                                    let value = match attribute {
                                        AttributeValue::Number(n) => {
                                            Value::Number(*n)
                                        }
                                        AttributeValue::Bool(b) => {
                                            Value::Bool(*b)
                                        }
                                        AttributeValue::String(s) => {
                                            Value::String(s.clone())
                                        }
                                    };

                                    map.insert(
                                        MapKey::String(key.clone()),
                                        value,
                                    );
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

                if name == "sound" {
                    return Ok(Some(Value::Handle {
                        kind: HandleKind::Sound,
                        id,
                    }));
                }

                if name == "name" {
                    if let Some(entity_name) =
                        self.entity_manager.get_name(entity_id)
                    {
                        return Ok(Some(Value::String(
                            entity_name.to_string(),
                        )));
                    }
                }
            }

            HandleKind::Sound => match name {
                "playing" => {
                    return Ok(Some(Value::Bool(
                        self.world.is_audio_playing(id)
                            && !self.world.is_audio_paused(id),
                    )));
                }
                "looped" => {
                    return Ok(Some(Value::Bool(
                        self.world.is_audio_looped(id),
                    )));
                }
                "volume" => {
                    return Ok(Some(Value::Number(
                        self.world.get_audio_volume(id) as f64,
                    )));
                }
                _ => {}
            },

            HandleKind::Ui => {}
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
                if self
                    .world
                    .get_effective_cell_by_id(id)
                    .is_some()
                {
                    match name {
                        "position" => {
                            let basket = value.as_basket()?;
                            let borrowed = basket.borrow();

                            if borrowed.elements.len() != 3 {
                                return Err(
                                    "position must be a basket of 3 numbers [x, y, z]"
                                        .to_string(),
                                );
                            }

                            let x =
                                borrowed.elements[0].as_number()? as i32;
                            let y =
                                borrowed.elements[1].as_number()? as i32;
                            let z =
                                borrowed.elements[2].as_number()? as i32;

                            self.move_runtime_cell(id, x, y, z)?;

                            return Ok(true);
                        }

                        "name" => {
                            let name = value.as_string()?;

                            if let Some(cell) =
                                self.world.runtime_cells.get_mut(&id)
                            {
                                cell.entity_identity =
                                    Some(name.to_string());
                            } else {
                                return Err(
                                    "Cannot change name of an authored cell at runtime"
                                        .to_string(),
                                );
                            }

                            return Ok(true);
                        }

                        "visible" => {
                            let visible = value.as_bool()?;

                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                self.world
                                    .set_cell_visible_runtime(
                                        coord, visible,
                                    );
                            } else {
                                self.world
                                    .runtime_state
                                    .entry(id)
                                    .or_default()
                                    .visible = Some(visible);
                            }

                            return Ok(true);
                        }

                        "enabled" => {
                            let enabled = value.as_bool()?;

                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                self.world
                                    .set_light_enabled_runtime(
                                        coord, enabled,
                                    );
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

                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                self.world
                                    .set_cell_solid_runtime(
                                        coord, solid,
                                    );
                            } else {
                                self.world
                                    .runtime_state
                                    .entry(id)
                                    .or_default()
                                    .solid = Some(solid);
                            }

                            return Ok(true);
                        }

                        "anchored" => {
                            let anchored = value.as_bool()?;

                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                self.world
                                    .set_cell_anchored_runtime(
                                        coord, anchored,
                                    );
                            } else {
                                self.world
                                    .runtime_state
                                    .entry(id)
                                    .or_default()
                                    .anchored = Some(anchored);
                            }

                            return Ok(true);
                        }

                        "color" => {
                            let basket = value.as_basket()?;
                            let borrowed = basket.borrow();

                            if borrowed.elements.len() != 3 {
                                return Err(
                                    "color must be a basket of 3 numbers [r, g, b]"
                                        .to_string(),
                                );
                            }

                            let color = Vec3::new(
                                borrowed.elements[0].as_number()? as f32,
                                borrowed.elements[1].as_number()? as f32,
                                borrowed.elements[2].as_number()? as f32,
                            );

                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                self.world
                                    .set_cell_color_runtime(
                                        coord, color,
                                    );
                            } else {
                                self.world
                                    .runtime_state
                                    .entry(id)
                                    .or_default()
                                    .color_rgb = Some(color);
                            }

                            return Ok(true);
                        }

                        "offset" => {
                            let basket = value.as_basket()?;
                            let borrowed = basket.borrow();

                            if borrowed.elements.len() != 3 {
                                return Err(
                                    "offset must be a basket of 3 numbers [x, y, z]"
                                        .to_string(),
                                );
                            }

                            let offset = Vec3::new(
                                borrowed.elements[0].as_number()? as f32,
                                borrowed.elements[1].as_number()? as f32,
                                borrowed.elements[2].as_number()? as f32,
                            );

                            if let Some(coord) =
                                self.world.resolve_cell_id(id)
                            {
                                self.world
                                    .set_visual_offset_runtime(
                                        coord, offset,
                                    );
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

    fn call_method(
        &mut self,
        kind: HandleKind,
        id: u64,
        name: &str,
        _args: &[Value],
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
        Ok(None)
    }

    fn is_audio_playing(&self, id: u64) -> Option<bool> {
        Some(self.world.is_audio_playing(id))
    }
    fn is_audio_paused(&self, id: u64) -> Option<bool> {
        Some(self.world.is_audio_paused(id))
    }
    fn is_audio_looped(&self, id: u64) -> Option<bool> {
        Some(self.world.is_audio_looped(id))
    }
    fn get_audio_volume(&self, id: u64) -> Option<f32> {
        Some(self.world.get_audio_volume(id))
    }
    fn set_audio_playing(&mut self, id: u64, playing: bool) {
        self.world.set_audio_playing_runtime(id, playing);
    }
    fn set_audio_looped(&mut self, id: u64, looped: bool) {
        self.world.set_audio_looped_runtime(id, looped);
    }
    fn set_audio_volume(&mut self, id: u64, volume: f32) {
        self.world.set_audio_volume_runtime(id, volume);
    }
    fn audio_play(&mut self, id: u64) {
        self.world.audio_play_runtime(id);
    }
    fn audio_stop(&mut self, id: u64) {
        self.world.audio_stop_runtime(id);
    }
    fn audio_pause(&mut self, id: u64) {
        self.world.audio_pause_runtime(id);
    }

    fn set_attribute(
        &mut self,
        id: u64,
        key: String,
        value: Value,
    ) -> Result<(), String> {
        if self
            .world
            .get_effective_cell_by_id(id)
            .is_some()
        {
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
            Err(
                "invalid cell handle for attribute assignment"
                    .to_string(),
            )
        }
    }

    fn remove_attribute(
        &mut self,
        id: u64,
        key: &str,
    ) -> Result<(), String> {
        if self
            .world
            .get_effective_cell_by_id(id)
            .is_some()
        {
            if let Some(runtime_state) =
                self.world.runtime_state.get_mut(&id)
            {
                runtime_state.attribute_overrides.remove(key);
            }

            Ok(())
        } else {
            Err(
                "invalid cell handle for attribute removal"
                    .to_string(),
            )
        }
    }

    fn cell_exists(&self, id: u64) -> bool {
        self.world.get_effective_cell_by_id(id).is_some()
    }

    fn entity_exists(&self, id: u64) -> bool {
        self.entity_manager.validate_handle(id)
    }

    fn get_script_property(
        &self,
        kind: HandleKind,
        id: u64,
        name: &str,
    ) -> Option<Value> {
        self.dynamic_properties
            .get(&(kind, id))
            .and_then(|properties| properties.get(name))
            .cloned()
    }

    fn set_script_property(
        &mut self,
        kind: HandleKind,
        id: u64,
        name: String,
        value: Value,
    ) {
        self.dynamic_properties
            .entry((kind, id))
            .or_default()
            .insert(name, value);
    }

    fn create_ui_element(
        &mut self,
        element_type: &str,
    ) -> Result<u64, String> {
        self.runtime_ui.allocate(element_type)
    }

    fn delete_ui_element(&mut self, id: u64) -> Result<(), String> {
        self.runtime_ui.delete(id)
    }

    fn get_ui_property(
        &self,
        id: u64,
        name: &str,
    ) -> Result<Option<Value>, String> {
        self.runtime_ui.get_property(id, name)
    }

    fn set_ui_property(
        &mut self,
        id: u64,
        name: &str,
        value: Value,
    ) -> Result<bool, String> {
        self.runtime_ui.set_property(id, name, value)
    }

    fn drain_ui_clicks(&mut self) -> Vec<u64> {
        self.runtime_ui.drain_pending_clicks()
    }

    fn get_input_move_vector(&self) -> [f32; 2] {
        [self.move_input.x, self.move_input.y]
    }

    fn is_input_jump_pressed(&self) -> bool {
        self.jump_requested
    }

    fn get_input_orbit_delta(&self) -> [f32; 2] {
        self.orbit_delta
    }

    fn get_camera_horizontal_basis(
        &self,
    ) -> ([f32; 3], [f32; 3]) {
        if let Some(ref camera) = self.gameplay_camera {
            let (forward, right) = camera.get_horizontal_basis();

            (
                [forward.x, forward.y, forward.z],
                [right.x, right.y, right.z],
            )
        } else {
            ([0.0, 0.0, -1.0], [1.0, 0.0, 0.0])
        }
    }

    fn set_camera_position(&mut self, position: [f32; 3]) {
        if let Some(ref mut camera) = self.gameplay_camera {
            camera.current_position = Vec3::new(
                position[0],
                position[1],
                position[2],
            );
        }
    }

    fn set_camera_target(&mut self, target: [f32; 3]) {
        if let Some(ref mut camera) = self.gameplay_camera {
            camera.current_target = Vec3::new(
                target[0],
                target[1],
                target[2],
            );
        }
    }

    fn set_camera_orientation(
        &mut self,
        yaw: f32,
        pitch: f32,
    ) {
        if let Some(ref mut camera) = self.gameplay_camera {
            camera.yaw = yaw;
            camera.pitch = pitch;
        }
    }

    fn resolve_camera_collision(
        &self,
        target: [f32; 3],
        desired: [f32; 3],
    ) -> [f32; 3] {
        if let Some(ref camera) = self.gameplay_camera {
            let target = Vec3::new(
                target[0],
                target[1],
                target[2],
            );

            let desired = Vec3::new(
                desired[0],
                desired[1],
                desired[2],
            );

            let resolved =
                camera.resolve_collision(
                    target,
                    desired,
                    self.world,
                );

            [resolved.x, resolved.y, resolved.z]
        } else {
            desired
        }
    }

    fn get_player_position(&self) -> Option<[f32; 3]> {
        let system = self.character_system.as_deref()?;
        let player = system.get_active_player()?;

        let position = player.transform.position;

        Some([
            position.x,
            position.y,
            position.z,
        ])
    }

    fn set_player_horizontal_velocity(
        &mut self,
        velocity_x: f32,
        velocity_z: f32,
    ) {
        if let Some(system) =
            self.character_system.as_deref_mut()
        {
            if let Some(player) =
                system.get_active_player_mut()
            {
                player.movement.velocity.x = velocity_x;
                player.movement.velocity.z = velocity_z;
            }
        }
    }

    fn set_player_facing_direction(
        &mut self,
        direction_x: f32,
        direction_z: f32,
    ) {
        if let Some(system) =
            self.character_system.as_deref_mut()
        {
            if let Some(player) =
                system.get_active_player_mut()
            {
                let angle =
                    f32::atan2(direction_x, direction_z);

                player.transform.rotation =
                    glam::Quat::from_rotation_y(angle);
            }
        }
    }

    fn select_player_animation(&mut self, animation: &str) {
        if let Some(system) =
            self.character_system.as_deref_mut()
        {
            if let Some(player) =
                system.get_active_player_mut()
            {
                let target_animation = match animation {
                    "Walk" => {
                        crate::character_custom::TargetAnimation::Walk
                    }

                    _ => {
                        crate::character_custom::TargetAnimation::Idle
                    }
                };

                player
                    .animation_controller
                    .select_animation(target_animation);
            }
        }
    }

    fn is_player_grounded(&self) -> bool {
        if let Some(system) = self.character_system.as_deref() {
            if let Some(player) = system.get_active_player() {
                return player.movement.is_grounded;
            }
        }

        true
    }

    fn apply_player_vertical_impulse(&mut self, impulse: f32) {
        if let Some(system) =
            self.character_system.as_deref_mut()
        {
            if let Some(player) =
                system.get_active_player_mut()
            {
                player.movement.velocity.y = impulse;
                player.movement.is_grounded = false;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::CharacterSystem;
    use crate::scripting::api::EngineHost;
    use crate::scripting::value::{HandleKind, Value};
    use crate::world::cell::{AttributeValue, CellType};
    use crate::world::WorldCoord;

    #[test]
    fn test_script_host_bridge_attributes_get() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();

        let coord = WorldCoord::new(5, 5, 5);
        let cell_id = world.set_cell(coord, CellType::Block);

        if let Some(cell) = world.get_mut(coord) {
            cell.attributes.insert(
                "health".to_string(),
                AttributeValue::Number(100.0),
            );

            cell.attributes.insert(
                "is_boss".to_string(),
                AttributeValue::Bool(false),
            );

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
        let mut runtime_ui =
            crate::engine::ui::RuntimeUi::new();

        let bridge = ScriptHostBridge {
            entity_manager: &mut entity_manager,
            world: &mut world,
            dynamic_properties: &mut dynamic_properties,
            pending_events: &mut pending_events,
            test_results: &mut test_results,
            pending_enable_scripts: &mut pending_enable_scripts,
            pending_disable_scripts: &mut pending_disable_scripts,
            runtime_ui: &mut runtime_ui,
            viewport_size: [0.0, 0.0],
            move_input: glam::Vec2::ZERO,
            jump_requested: false,
            orbit_delta: [0.0, 0.0],
            character_system: None,
            gameplay_camera: None,
        };

        let value = bridge
            .get_property(
                HandleKind::Cell,
                cell_id,
                "attributes",
            )
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
        let cell_id =
            world.set_cell(coord, CellType::Block);

        if let Some(cell) = world.get_mut(coord) {
            cell.attributes.insert(
                "test".to_string(),
                AttributeValue::String(
                    "authored".to_string(),
                ),
            );
        }

        let mut dynamic_properties =
            std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results =
            std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui =
            crate::engine::ui::RuntimeUi::new();

        let mut bridge = ScriptHostBridge {
            entity_manager: &mut entity_manager,
            world: &mut world,
            dynamic_properties: &mut dynamic_properties,
            pending_events: &mut pending_events,
            test_results: &mut test_results,
            pending_enable_scripts: &mut pending_enable_scripts,
            pending_disable_scripts: &mut pending_disable_scripts,
            runtime_ui: &mut runtime_ui,
            viewport_size: [1280.0, 720.0],
            move_input: glam::Vec2::ZERO,
            jump_requested: false,
            orbit_delta: [0.0, 0.0],
            character_system: None,
            gameplay_camera: None,
        };

        let attrs = bridge
            .get_property(
                HandleKind::Cell,
                cell_id,
                "attributes",
            )
            .unwrap()
            .unwrap();

        let map = attrs.as_map().unwrap();

        assert_eq!(
            map.borrow().get(&MapKey::String(
                "test".to_string()
            )),
            Some(&Value::String(
                "authored".to_string()
            ))
        );

        bridge
            .set_attribute(
                cell_id,
                "test".to_string(),
                Value::String("runtime".to_string()),
            )
            .unwrap();

        let attrs = bridge
            .get_property(
                HandleKind::Cell,
                cell_id,
                "attributes",
            )
            .unwrap()
            .unwrap();

        let map = attrs.as_map().unwrap();

        assert_eq!(
            map.borrow().get(&MapKey::String(
                "test".to_string()
            )),
            Some(&Value::String(
                "runtime".to_string()
            ))
        );

        assert_eq!(
            bridge
                .world
                .get(coord)
                .unwrap()
                .attributes
                .get("test"),
            Some(&AttributeValue::String(
                "authored".to_string()
            ))
        );

        bridge
            .remove_attribute(cell_id, "test")
            .unwrap();

        let attrs = bridge
            .get_property(
                HandleKind::Cell,
                cell_id,
                "attributes",
            )
            .unwrap()
            .unwrap();

        let map = attrs.as_map().unwrap();

        assert_eq!(
            map.borrow().get(&MapKey::String(
                "test".to_string()
            )),
            Some(&Value::String(
                "authored".to_string()
            ))
        );
    }

    #[test]
    fn test_player_set_position_updates_character_once() {
        let mut world = World::new();
        let mut entity_manager = EntityManager::new();
        let mut character_system = CharacterSystem::new();

        world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::SpawnPoint,
        );

        character_system
            .spawn_player(&world, None)
            .expect("player should spawn");

        let player_id =
            entity_manager.create_entity("Player");

        let mut dynamic_properties =
            std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results =
            std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui =
            crate::engine::ui::RuntimeUi::new();

        let mut bridge = ScriptHostBridge {
            entity_manager: &mut entity_manager,
            world: &mut world,
            dynamic_properties: &mut dynamic_properties,
            pending_events: &mut pending_events,
            test_results: &mut test_results,
            pending_enable_scripts: &mut pending_enable_scripts,
            pending_disable_scripts: &mut pending_disable_scripts,
            runtime_ui: &mut runtime_ui,
            viewport_size: [1280.0, 720.0],
            move_input: glam::Vec2::ZERO,
            jump_requested: false,
            orbit_delta: [0.0, 0.0],
            character_system: Some(&mut character_system),
            gameplay_camera: None,
        };

        let target = Vec3::new(10.0, 20.0, 30.0);

        bridge.set_position(player_id.0, target);

        assert_eq!(
            bridge
                .entity_manager
                .get_position(player_id),
            Some(target)
        );

        assert_eq!(
            bridge
                .character_system
                .as_deref()
                .and_then(|system| {
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

        world.set_cell(
            WorldCoord::new(0, 0, 0),
            CellType::SpawnPoint,
        );

        character_system
            .spawn_player(&world, None)
            .expect("player should spawn");

        let original_position = character_system
            .get_active_player()
            .unwrap()
            .transform
            .position;

        let entity_id =
            entity_manager.create_entity("SomethingElse");

        let mut dynamic_properties =
            std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results =
            std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut runtime_ui =
            crate::engine::ui::RuntimeUi::new();

        let mut bridge = ScriptHostBridge {
            entity_manager: &mut entity_manager,
            world: &mut world,
            dynamic_properties: &mut dynamic_properties,
            pending_events: &mut pending_events,
            test_results: &mut test_results,
            pending_enable_scripts: &mut pending_enable_scripts,
            pending_disable_scripts: &mut pending_disable_scripts,
            runtime_ui: &mut runtime_ui,
            viewport_size: [1280.0, 720.0],
            move_input: glam::Vec2::ZERO,
            jump_requested: false,
            orbit_delta: [0.0, 0.0],
            character_system: Some(&mut character_system),
            gameplay_camera: None,
        };

        bridge.set_position(
            entity_id.0,
            Vec3::new(100.0, 100.0, 100.0),
        );

        assert_eq!(
            bridge
                .character_system
                .as_deref()
                .and_then(|system| {
                    system
                        .get_active_player()
                        .map(|player| player.transform.position)
                }),
            Some(original_position)
        );
    }
}