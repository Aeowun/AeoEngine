use super::ast::{Declaration, EntityMember, Program};
use super::binding::ScriptBinding;
use super::execution::{FiberResult, ScriptTaskId};
use super::interpreter::ScriptInstance;
use super::lexer::Lexer;
use super::parser::Parser;
use super::runtime::ScriptRuntime;
use super::source::SourceSpan;
use super::value::Value;
use super::api::HostContext;
use crate::engine::entity::{EntityId, EntityManager};
use crate::world::{CellType, World, WorldCoord};
use glam::Vec3;
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::log::LogRecord;

/// A single scripted entity's lifecycle state.
#[derive(Debug)]
pub struct ScriptEntity {
    instance: ScriptInstance,
    state: EntityState,
    stopped: bool,
    script_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EntityState {
    Initial,
    Spawning(Option<ScriptTaskId>),
    Readying(Option<ScriptTaskId>),
    Active {
        update_task: Option<ScriptTaskId>,
    },
    Stopped,
}

impl ScriptEntity {
    fn new(instance: ScriptInstance, script_path: Option<String>) -> Self {
        Self {
            instance,
            state: EntityState::Initial,
            stopped: false,
            script_path,
        }
    }

    fn active_task(&self) -> Option<ScriptTaskId> {
        match self.state {
            EntityState::Spawning(Some(tid)) => Some(tid),
            EntityState::Readying(Some(tid)) => Some(tid),
            EntityState::Active { update_task: Some(tid) } => Some(tid),
            _ => None,
        }
    }

    fn complete_task(&mut self, id: ScriptTaskId) {
        match self.state {
            EntityState::Spawning(Some(tid)) if tid == id => {
                self.state = EntityState::Spawning(None);
            }
            EntityState::Readying(Some(tid)) if tid == id => {
                self.state = EntityState::Readying(None);
            }
            EntityState::Active { update_task: Some(tid) } if tid == id => {
                self.state = EntityState::Active { update_task: None };
            }
            _ => {}
        }
    }
}

/// Owns the live AeoScript scene state and coordinates the lifecycle of scripted entities.
#[derive(Debug)]
pub struct ScriptScene {
    runtime: ScriptRuntime,
    entities: Vec<ScriptEntity>,
    current_time: f64,
}

impl ScriptScene {
    /// Loads and parses scripts from the project's scripts directory and applies explicit bindings.
    pub fn load_from_bindings(
        project_path: &Path,
        world: &World,
        bindings: &[ScriptBinding],
        entity_manager: &mut EntityManager,
        delta_time: f64,
    ) -> Result<Self, String> {
        let mut authored_entities: HashMap<String, WorldCoord> = HashMap::new();

        for (coord, cell) in world.cells.iter() {
            if !cell.cell_type.is_entity() {
                continue;
            }

            if let Some(identity) = &cell.entity_identity {
                if authored_entities.insert(identity.clone(), *coord).is_some() {
                    return Err(format!(
                        "Duplicate authored entity identity found: '{}'",
                        identity
                    ));
                }
            }
        }

        let mut valid_bindings = Vec::new();
        let mut stale_warnings = Vec::new();
        let mut seen_binding_targets = HashSet::new();

        for binding in bindings {
            if !seen_binding_targets.insert(binding.target_identity.clone()) {
                return Err(format!(
                    "Multiple script bindings found for target identity '{}'. Each authored entity can only have one binding.",
                    binding.target_identity
                ));
            }

            if !authored_entities.contains_key(&binding.target_identity) {
                stale_warnings.push(format!(
                    "Stale script binding found: target identity '{}' does not exist in the world as an authored entity.",
                    binding.target_identity
                ));
                continue;
            }

            valid_bindings.push(binding.clone());
        }

        let mut loaded_scripts: HashMap<String, Program> = HashMap::new();

        // 1. Load ALL scripts from the project's scripts directory.
        // This ensures unattached event handlers are registered.
        let scripts_dir = project_path.join("scripts");
        if scripts_dir.exists() && scripts_dir.is_dir() {
            for entry in std::fs::read_dir(scripts_dir).map_err(|e| format!("Failed to read scripts directory: {}", e))?.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "aeo") {
                    let source = std::fs::read_to_string(&path).map_err(|e| {
                        format!("Failed to read script {:?}: {}", path, e)
                    })?;

                    let tokens = Lexer::new(&source).tokenize().map_err(|e| {
                        format!("Lexer error in {:?}: {:?}", path, e)
                    })?;

                    let program = Parser::new(tokens).parse().map_err(|e| {
                        format!("Parser error in {:?}: {:?}", path, e)
                    })?;

                    // Use relative path from project root as the key
                    let relative_path = path.strip_prefix(project_path).unwrap_or(&path).to_string_lossy().to_string();
                    // Normalize separators for cross-platform matching with bindings
                    let relative_path = relative_path.replace("\\", "/");

                    loaded_scripts.insert(relative_path, program);
                }
            }
        }

        // 2. Validate explicit bindings.
        for binding in &valid_bindings {
            // Normalize binding path
            let normalized_binding_path = binding.script_path.replace("\\", "/");

            let program = loaded_scripts
                .get(&normalized_binding_path)
                .ok_or_else(|| format!("Bound script '{}' not found in scripts directory.", binding.script_path))?;

            let has_entity = program.declarations.iter().any(|decl| {
                if let Declaration::Entity(entity) = decl {
                    entity.name == binding.target_identity
                } else {
                    false
                }
            });

            if !has_entity {
                return Err(format!(
                    "Script '{}' does not contain an entity declaration for '{}'",
                    binding.script_path, binding.target_identity
                ));
            }
        }

        let mut all_declarations = Vec::new();
        for program in loaded_scripts.values() {
            all_declarations.extend(program.declarations.clone());
        }

        let combined_program = Program {
            span: SourceSpan::new(0, 0),
            declarations: all_declarations,
        };

        let spawn_params: Vec<(String, u64, Option<String>)> = valid_bindings
            .iter()
            .map(|binding| {
                let coord = authored_entities
                    .get(&binding.target_identity)
                    .expect("binding target was validated above");
                let id = entity_manager.create_entity(&binding.target_identity);
                entity_manager.set_position(
                    id,
                    Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32),
                );
                (binding.target_identity.clone(), id.0, Some(binding.script_path.clone()))
            })
            .collect();

        let created_ids: Vec<EntityId> = spawn_params
            .iter()
            .map(|(_, raw_id, _)| EntityId(*raw_id))
            .collect();

        let mut scene_res = {
            let mut host = HostContext {
                delta_time,
                engine: entity_manager,
            };

            Self::new(combined_program, spawn_params, &mut host)
        };

        if let Err(e) = &scene_res {
            for id in created_ids {
                entity_manager.remove_entity(id);
            }
            return Err(e.clone());
        }

        let mut scene = scene_res.unwrap();

        // Add event handlers from all loaded scripts with their correct paths.
        // We clear the default auto-registered handlers (which have UNKNOWN path)
        // first to avoid duplication.
        scene.runtime.clear_event_handlers();
        for (path, program) in &loaded_scripts {
            scene.runtime.add_event_handlers(path.clone(), program);
        }

        // Log stale bindings as warnings
        for warning in stale_warnings {
            scene.runtime.interpreter_mut().log_warning(warning);
        }

        Ok(scene)
    }

    pub fn new(
        program: Program,
        entities_to_spawn: Vec<(String, u64, Option<String>)>,
        host: &mut HostContext,
    ) -> Result<Self, String> {
        let mut runtime = ScriptRuntime::new(program);
        let mut entities = Vec::with_capacity(entities_to_spawn.len());

        for (name, id, script_path) in entities_to_spawn {
            let mut instance = runtime
                .interpreter_mut()
                .instantiate_entity(&name, id, host)
                .map_err(|e| format!("Failed to instantiate script entity '{}': {}", name, e))?;
            instance.script_path = script_path.clone();
            entities.push(ScriptEntity::new(instance, script_path));
        }

        Ok(Self {
            runtime,
            entities,
            current_time: 0.0,
        })
    }

    /// Starts the lifecycle for all entities in the scene.
    pub fn start(&mut self, host: &mut HostContext) -> Result<(), String> {
        for entity in &mut self.entities {
            Self::transition_entity(&mut self.runtime, entity, host)?;
        }
        Ok(())
    }

    /// Advances the scene time and executes script fibers.
    pub fn update(&mut self, dt: f32, host: &mut HostContext) -> Result<(), String> {
        self.current_time += dt as f64;
        let tick_results = self.runtime.tick(self.current_time, host)?;

        for (task_id, result) in tick_results {
            let mut handled = false;
            for entity in &mut self.entities {
                if entity.active_task() == Some(task_id) {
                    match result {
                        FiberResult::Complete => {
                            self.runtime.remove_fiber(task_id);
                            entity.complete_task(task_id);
                            Self::transition_entity(&mut self.runtime, entity, host)?;
                        }
                        FiberResult::Failed(err) => {
                            self.runtime.remove_fiber(task_id);
                            entity.state = EntityState::Stopped;
                            entity.stopped = true;
                            return Err(format!(
                                "Script error in entity {}: {}",
                                entity.instance.entity_name(),
                                err
                            ));
                        }
                        _ => {}
                    }
                    handled = true;
                    break;
                }
            }

            if !handled {
                // Handle global (unattached) tasks
                match result {
                    FiberResult::Complete => {
                        self.runtime.remove_fiber(task_id);
                    }
                    FiberResult::Failed(err) => {
                        self.runtime.remove_fiber(task_id);
                        return Err(format!("Global script error: {}", err));
                    }
                    _ => {}
                }
            }
        }

        for entity in &mut self.entities {
            if let EntityState::Active { update_task: None } = entity.state {
                if !entity.stopped && Self::has_function(&self.runtime, entity, "update") {
                    let task_id = self.runtime.spawn(
                        entity.instance.clone(),
                        "update",
                        vec![Value::Number(dt as f64)],
                    )?;
                    entity.state = EntityState::Active {
                        update_task: Some(task_id),
                    };
                }
            }
        }

        Ok(())
    }

    pub fn output(&self) -> &[LogRecord] {
        self.runtime.interpreter().output()
    }

    pub fn drain_output(&mut self) -> Vec<LogRecord> {
        self.runtime.interpreter_mut().drain_output()
    }

    /// Dispatches a global event to all scripts in the scene.
    pub fn dispatch_event(&mut self, name: &str, arguments: Vec<Value>, host: &mut HostContext) -> Result<(), String> {
        self.runtime.dispatch_event(name, arguments, host)
    }

    /// Stops all script execution and invokes on_destroy where available.
    pub fn stop(&mut self, host: &mut HostContext) {
        for entity in &mut self.entities {
            if entity.stopped {
                continue;
            }
            entity.stopped = true;

            if let Some(task_id) = entity.active_task() {
                let _ = self.runtime.cancel(task_id);
                let _ = self.runtime.remove_fiber(task_id);
            }

            if Self::has_function(&self.runtime, entity, "on_destroy") {
                let mut instance = entity.instance.clone();
                let _ = self
                    .runtime
                    .interpreter_mut()
                    .call(&mut instance, "on_destroy", vec![], host);
            }

            entity.state = EntityState::Stopped;
        }
        self.entities.clear();
    }

    fn has_function(runtime: &ScriptRuntime, entity: &ScriptEntity, name: &str) -> bool {
        let entity_name = entity.instance.entity_name();
        runtime
            .interpreter()
            .program()
            .declarations
            .iter()
            .any(|decl| {
                if let Declaration::Entity(e) = decl {
                    if e.name == entity_name {
                        return e.members.iter().any(|m| {
                            if let EntityMember::Function(f) = m {
                                f.name == name
                            } else {
                                false
                            }
                        });
                    }
                }
                false
            })
    }

    fn transition_entity(
        runtime: &mut ScriptRuntime,
        entity: &mut ScriptEntity,
        _host: &mut HostContext,
    ) -> Result<(), String> {
        if entity.stopped {
            return Ok(());
        }

        loop {
            match entity.state {
                EntityState::Initial => {
                    if Self::has_function(runtime, entity, "on_spawn") {
                        let task_id = runtime.spawn(
                            entity.instance.clone(),
                            "on_spawn",
                            vec![],
                        )?;
                        entity.state = EntityState::Spawning(Some(task_id));
                        break;
                    } else {
                        entity.state = EntityState::Spawning(None);
                    }
                }
                EntityState::Spawning(_) => {
                    if let Some(task_id) = entity.active_task() {
                        let _ = task_id;
                        break;
                    }
                    if Self::has_function(runtime, entity, "on_ready") {
                        let task_id = runtime.spawn(
                            entity.instance.clone(),
                            "on_ready",
                            vec![],
                        )?;
                        entity.state = EntityState::Readying(Some(task_id));
                        break;
                    } else {
                        entity.state = EntityState::Readying(None);
                    }
                }
                EntityState::Readying(_) => {
                    if let Some(task_id) = entity.active_task() {
                        let _ = task_id;
                        break;
                    }
                    entity.state = EntityState::Active { update_task: None };
                    break;
                }
                _ => break,
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::entity::{EntityId, EntityManager};
    use crate::scripting::api::EngineHost;
    use crate::scripting::lexer::Lexer;
    use crate::scripting::parser::Parser;
    use glam::Vec3;

    struct TestHost {
        entity_manager: EntityManager,
        world: World,
    }

    impl EngineHost for TestHost {
        fn entity_manager(&self) -> &EntityManager {
            &self.entity_manager
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
                return Some(self.world.is_light_enabled(coord));
            }
            None
        }

        fn set_light_enabled(&mut self, id: u64, enabled: bool) {
            if let Some(coord) = self.world.resolve_cell_id(id) {
                self.world.set_light_enabled_runtime(coord, enabled);
            }
        }

        fn get_all_cells_of_class(&self, class_name: &str) -> Vec<u64> {
            let mut results = Vec::new();
            for cell in self.world.cells.values() {
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

        fn find_objects(&self, query: &str) -> Vec<(crate::scripting::value::HandleKind, u64)> {
            let mut results = Vec::new();
            if let Some(id) = self.entity_manager.lookup_entity(query) {
                results.push((crate::scripting::value::HandleKind::Entity, id.0));
            }

            for coord in self.world.active_blocks() {
                if let Some(cell) = self.world.get(coord) {
                    if let Some(identity) = &cell.entity_identity {
                        if identity == query {
                            let kind = match cell.cell_type {
                                crate::world::CellType::Light => crate::scripting::value::HandleKind::Light,
                                _ => crate::scripting::value::HandleKind::Cell,
                            };
                            results.push((kind, cell.id));
                        }
                    }
                }
            }
            results
        }

        fn get_children(&self, _kind: crate::scripting::value::HandleKind, _id: u64) -> Vec<(crate::scripting::value::HandleKind, u64)> {
            Vec::new()
        }

        fn get_parent(&self, _kind: crate::scripting::value::HandleKind, _id: u64) -> Option<(crate::scripting::value::HandleKind, u64)> {
            None
        }

        fn get_cell_object(&self, cell_id: u64) -> Option<(crate::scripting::value::HandleKind, u64)> {
            if let Some(coord) = self.world.resolve_cell_id(cell_id) {
                if let Some(cell) = self.world.get(coord) {
                    if let Some(identity) = &cell.entity_identity {
                        if let Some(id) = self.entity_manager.lookup_entity(identity) {
                            return Some((crate::scripting::value::HandleKind::Entity, id.0));
                        }
                    }
                }
            }
            None
        }

        fn get_property(&self, kind: crate::scripting::value::HandleKind, id: u64, name: &str) -> Result<Option<crate::scripting::value::Value>, String> {
            use crate::scripting::value::{Value, HandleKind};
            match kind {
                HandleKind::Cell | HandleKind::Light => {
                    if let Some(coord) = self.world.resolve_cell_id(id) {
                        if let Some(cell) = self.world.get(coord) {
                            match name {
                                "id" => return Ok(Some(Value::Number(cell.id as f64))),
                                "name" => return Ok(Some(Value::String(cell.entity_identity.clone().unwrap_or_else(|| "Cell".to_string())))),
                                "cellType" => return Ok(Some(Value::String(format!("{:?}", cell.cell_type)))),
                                "position" => return Ok(Some(Value::Array(vec![
                                    Value::Number(coord.x as f64),
                                    Value::Number(coord.y as f64),
                                    Value::Number(coord.z as f64),
                                ]))),
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

        fn set_property(&mut self, kind: crate::scripting::value::HandleKind, id: u64, name: &str, value: crate::scripting::value::Value) -> Result<(), String> {
            use crate::scripting::value::HandleKind;
            match kind {
                HandleKind::Cell | HandleKind::Light => {
                    if let Some(coord) = self.world.resolve_cell_id(id) {
                        match name {
                            "visible" => {
                                let visible = value.as_bool()?;
                                self.world.set_cell_visible_runtime(coord, visible);
                            }
                            "solid" => {
                                let solid = value.as_bool()?;
                                self.world.set_cell_solid_runtime(coord, solid);
                            }
                            "anchored" => {
                                let anchored = value.as_bool()?;
                                self.world.set_cell_anchored_runtime(coord, anchored);
                            }
                            "color" => {
                                let basket = value.as_basket()?;
                                if basket.len() == 3 {
                                    let r = basket[0].as_number()? as f32;
                                    let g = basket[1].as_number()? as f32;
                                    let b = basket[2].as_number()? as f32;
                                    self.world.set_cell_color_runtime(coord, glam::Vec3::new(r, g, b));
                                }
                            }
                            "enabled" => {
                                let enabled = value.as_bool()?;
                                self.world.set_light_enabled_runtime(coord, enabled);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
            Ok(())
        }

        fn call_method(&mut self, _kind: crate::scripting::value::HandleKind, _id: u64, _name: &str, _args: &[crate::scripting::value::Value]) -> Result<Option<crate::scripting::value::Value>, String> {
            Ok(None)
        }
    }

    fn test_host() -> TestHost {
        TestHost {
            entity_manager: EntityManager::new(),
            world: World::new(),
        }
    }

    fn create_scene(source: &str, host: &mut HostContext) -> ScriptScene {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let entities_to_spawn: Vec<(String, u64, Option<String>)> = program
            .declarations
            .iter()
            .filter_map(|decl| {
                if let Declaration::Entity(entity) = decl {
                    let id = host
                        .engine
                        .entity_manager()
                        .lookup_entity(&entity.name)
                        .unwrap_or(EntityId(0));
                    Some((entity.name.clone(), id.0, None))
                } else {
                    None
                }
            })
            .collect();

        ScriptScene::new(program, entities_to_spawn, host).unwrap()
    }

    fn add_authored_entity(world: &mut World, coord: WorldCoord, cell_type: CellType, identity: &str) {
        let mut cell = crate::world::Cell::default();
        cell.cell_type = cell_type;
        cell.entity_identity = Some(identity.to_string());
        world.cells.insert(coord, cell);
    }

    #[test]
    fn test_host_context_construction() {
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 0.1,
            engine: &mut th,
        };
        assert_eq!(host.delta_time, 0.1);
        assert!(host.engine.entity_manager().lookup_entity("Any").is_none());
    }

    #[test]
    fn test_entity_position_read() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const e = get_entity("Test")
        const p = e.position
        debug.log(p[0])
        debug.log(p[1])
        debug.log(p[2])
    }
}
"#;
        let mut th = test_host();
        let id = th.entity_manager.create_entity("Test");
        th.entity_manager.set_position(id, Vec3::new(1.0, 2.0, 3.0));
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "1"));
        assert!(output.iter().any(|r| r.message == "2"));
        assert!(output.iter().any(|r| r.message == "3"));
    }

    #[test]
    fn test_entity_set_position() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const e = get_entity("Test")
        e.set_position(10.0, 20.0, 30.0)
    }
}
"#;
        let mut th = test_host();
        let id = th.entity_manager.create_entity("Test");
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert_eq!(
            th.entity_manager.get_position(id),
            Some(Vec3::new(10.0, 20.0, 30.0))
        );
    }

    #[test]
    fn test_entity_translate() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const e = get_entity("Test")
        e.translate(1.0, 1.0, 1.0)
    }
}
"#;
        let mut th = test_host();
        let id = th.entity_manager.create_entity("Test");
        th.entity_manager.set_position(id, Vec3::new(5.0, 5.0, 5.0));
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert_eq!(
            th.entity_manager.get_position(id),
            Some(Vec3::new(6.0, 6.0, 6.0))
        );
    }

    #[test]
    fn test_transform_correct_target() {
        let source = r#"
entity Target {
    fn on_spawn() {
        const e = get_entity("Target")
        e.set_position(100.0, 100.0, 100.0)
    }
}
entity Other {}
"#;
        let mut th = test_host();
        let id_target = th.entity_manager.create_entity("Target");
        let id_other = th.entity_manager.create_entity("Other");
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert_eq!(
            th.entity_manager.get_position(id_target),
            Some(Vec3::new(100.0, 100.0, 100.0))
        );
        assert_eq!(
            th.entity_manager.get_position(id_other),
            Some(Vec3::ZERO)
        );
    }

    #[test]
    fn test_transform_invalid_handle_fails_safely() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const e = get_entity("Test")
        wait(1)
        e.set_position(10.0, 20.0, 30.0)
    }
}
"#;
        let mut th = test_host();
        let id = th.entity_manager.create_entity("Test");
        let mut scene = {
            let mut host = HostContext {
                delta_time: 1.0,
                engine: &mut th,
            };
            let mut scene = create_scene(source, &mut host);
            scene.start(&mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
            scene
        };

        th.entity_manager.remove_entity(id);
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        assert!(scene.update(1.0, &mut host).is_ok());
    }

    #[test]
    fn scene_lifecycle_full() {
        let source = r#"
entity Test {
    fn on_spawn() { debug.log("spawn") }
    fn on_ready() { debug.log("ready") }
    fn update(dt: number) { debug.log("update") }
    fn on_destroy() { debug.log("destroy") }
}
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        assert!(scene.output().is_empty());

        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.stop(&mut host);
    }

    #[test]
    fn scene_update_receives_dt() {
        let source = r#"
entity Test {
    fn update(dt: number) { debug.log(dt) }
}
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.5, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "0.5"));
    }

    #[test]
    fn scene_runtime_failure_observable() {
        let source = r#"
entity Test {
    fn update(dt: number) {
        const x = 1 / 0
    }
}
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        let result = scene.update(0.0, &mut host);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("division by zero"));
    }

    #[test]
    fn scene_invalid_entity_reference() {
        let source = r#"
entity Test {
    fn update(dt: number) {
        const e = get_entity("Missing")
        if e == nil {
            debug.log("not_found")
        }
    }
}
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        assert!(scene.output().iter().any(|r| r.message == "not_found"));
    }

    #[test]
    fn scene_entity_validity() {
        let source = r#"
entity Test {
    fn update(dt: number) {
        const e = get_entity("Test")
        if e != nil {
            if e.is_valid() {
                debug.log("valid")
            }
        }
    }
}
"#;
        let mut th = test_host();
        let id = th.entity_manager.create_entity("Test");
        let mut scene = {
            let mut host = HostContext {
                delta_time: 1.0,
                engine: &mut th,
            };
            let mut scene = create_scene(source, &mut host);
            scene.start(&mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
            scene
        };

        assert!(scene.output().iter().any(|r| r.message == "valid"));
        th.entity_manager.remove_entity(id);
        scene.runtime.interpreter_mut().drain_output();

        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        scene.update(0.0, &mut host).unwrap();
    }

    #[test]
    fn scene_stop_prevents_execution() {
        let source = r#"
entity Test {
    fn update(dt: number) { debug.log("update") }
}
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.stop(&mut host);

        scene.update(0.1, &mut host).unwrap();
        assert!(!scene.output().iter().any(|r| r.message == "update"));
    }

    #[test]
    fn update_cooperative_wait() {
        let source = r#"
entity Test {
    fn update(dt: number) {
        debug.log("start")
        wait(1)
        debug.log("end")
    }
}
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        assert!(scene.output().iter().any(|r| r.message == "start"));
        assert!(!scene.output().iter().any(|r| r.message == "end"));

        scene.update(0.5, &mut host).unwrap();
        assert!(!scene.output().iter().any(|r| r.message == "end"));
        scene.update(0.5, &mut host).unwrap();
        assert!(scene.output().iter().any(|r| r.message == "end"));
    }

    #[test]
    fn test_explicit_binding_instantiates_only_requested() {
        let source = r#"
entity Player {}
entity Enemy {}
"#;
        let mut th = test_host();
        let id = th.entity_manager.create_entity("Player");
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let scene = ScriptScene::new(program, vec![("Player".to_string(), id.0, None)], &mut host)
            .unwrap();

        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].instance.entity_name(), "Player");
    }

    #[test]
    fn test_unassigned_script_does_not_execute() {
        let source = r#"
entity Bound {
    fn on_spawn() { debug.log("bound") }
}
entity Unbound {
    fn on_spawn() { debug.log("unbound") }
}
"#;
        let mut th = test_host();
        let id = th.entity_manager.create_entity("Bound");
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };

        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(program, vec![("Bound".to_string(), id.0, None)], &mut host)
            .unwrap();
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "bound"));
        assert!(!output.iter().any(|r| r.message == "unbound"));
    }

    #[test]
    fn test_multiple_bindings_remain_independent_bridge() {
        let source = r#"
entity A { fn on_spawn() { debug.log("A") } }
entity B { fn on_spawn() { debug.log("B") } }
"#;
        let mut th = test_host();
        let id_a = th.entity_manager.create_entity("A");
        let id_b = th.entity_manager.create_entity("B");
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(
            program,
            vec![("A".to_string(), id_a.0, None), ("B".to_string(), id_b.0, None)],
            &mut host,
        )
        .unwrap();
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "A"));
        assert!(output.iter().any(|r| r.message == "B"));
    }

    #[test]
    fn test_missing_entity_declaration_fails_safely_explicit() {
        let source = "entity Found {}";
        let mut th = test_host();
        add_authored_entity(&mut th.world, WorldCoord::new(0, 0, 0), CellType::NPC, "Missing");
        let temp_dir = std::env::temp_dir().join("aeo_test_missing_entity_decl");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        let script_path = "test.aeo";
        std::fs::write(temp_dir.join("scripts").join(script_path), source).unwrap();

        let bindings = vec![ScriptBinding::new("Missing", format!("scripts/{}", script_path))];
        let result = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            1.0,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Script 'scripts/test.aeo' does not contain an entity declaration for 'Missing'"));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_from_bindings_missing_script_fails() {
        let mut th = test_host();
        add_authored_entity(&mut th.world, WorldCoord::new(0, 0, 0), CellType::Player, "Player");
        let temp_dir = std::env::temp_dir().join("aeo_test_missing");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();

        let bindings = vec![ScriptBinding::new("Player", "nonexistent.aeo")];
        let result = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            1.0,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Bound script 'nonexistent.aeo' not found"));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_from_bindings_invalid_script_fails() {
        let mut th = test_host();
        add_authored_entity(&mut th.world, WorldCoord::new(0, 0, 0), CellType::NPC, "Broken");
        let temp_dir = std::env::temp_dir().join("aeo_test_invalid");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();

        let script_path = "broken.aeo";
        std::fs::write(temp_dir.join("scripts").join(script_path), "entity Broken { !!! }").unwrap();
        let bindings = vec![ScriptBinding::new("Broken", script_path)];
        let result = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            1.0,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Parser error"));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_get_light_api() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const l1 = get_light(10, 10, 10)
        if l1 != nil { debug.log("found_l1") }
        const l2 = get_light(0, 0, 0)
        if l2 == nil { debug.log("nil_l2") }
        const l3 = get_light(100, 100, 100)
        if l3 == nil { debug.log("nil_l3") }
    }
}
"#;
        let mut th = test_host();
        th.world
            .set_cell(WorldCoord::new(10, 10, 10), CellType::Light);
        th.world
            .set_cell(WorldCoord::new(0, 0, 0), CellType::Block);
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "found_l1"));
        assert!(output.iter().any(|r| r.message == "nil_l2"));
        assert!(output.iter().any(|r| r.message == "nil_l3"));
    }

    #[test]
    fn test_light_enable_api() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const l = get_light(10, 10, 10)
        if l.is_enabled() { debug.log("enabled_init") }
        l.set_enabled(false)
        if l.is_enabled() == false { debug.log("disabled_after_set") }
        l.set_enabled(true)
        if l.is_enabled() { debug.log("enabled_after_set") }
    }
}
"#;
        let mut th = test_host();
        let coord = WorldCoord::new(10, 10, 10);
        th.world.set_cell(coord, CellType::Light);
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "enabled_init"));
        assert!(output.iter().any(|r| r.message == "disabled_after_set"));
        assert!(output.iter().any(|r| r.message == "enabled_after_set"));
        assert_eq!(th.world.get(coord).unwrap().light_enabled, true);
    }

    #[test]
    fn test_light_independent() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const l1 = get_light(10, 10, 10)
        const l2 = get_light(20, 20, 20)
        l1.set_enabled(false)
        if l1.is_enabled() == false { debug.log("l1_off") }
        if l2.is_enabled() == true { debug.log("l2_on") }
    }
}
"#;
        let mut th = test_host();
        th.world
            .set_cell(WorldCoord::new(10, 10, 10), CellType::Light);
        th.world
            .set_cell(WorldCoord::new(20, 20, 20), CellType::Light);
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "l1_off"));
        assert!(output.iter().any(|r| r.message == "l2_on"));
    }

    #[test]
    fn test_light_wait_loop() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const l = get_light(10, 10, 10)
        for i in [1, 2] {
            l.set_enabled(false)
            wait(1)
            l.set_enabled(true)
            wait(1)
        }
        debug.log("done")
    }
}
"#;
        let mut th = test_host();
        let coord = WorldCoord::new(10, 10, 10);
        th.world.set_cell(coord, CellType::Light);
        let mut scene = {
            let mut host = HostContext {
                delta_time: 1.0,
                engine: &mut th,
            };
            let mut scene = create_scene(source, &mut host);
            scene.start(&mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
            scene
        };

        {
            let mut host = HostContext {
                delta_time: 1.0,
                engine: &mut th,
            };
            scene.update(0.0, &mut host).unwrap();
        }
        assert_eq!(th.world.is_light_enabled(coord), false);

        {
            let mut host = HostContext {
                delta_time: 1.0,
                engine: &mut th,
            };
            scene.update(1.0, &mut host).unwrap();
        }
        assert_eq!(th.world.is_light_enabled(coord), true);

        {
            let mut host = HostContext {
                delta_time: 1.0,
                engine: &mut th,
            };
            scene.update(1.0, &mut host).unwrap();
        }
        assert_eq!(th.world.is_light_enabled(coord), false);

        {
            let mut host = HostContext {
                delta_time: 1.0,
                engine: &mut th,
            };
            scene.update(1.0, &mut host).unwrap();
        }
        assert_eq!(th.world.is_light_enabled(coord), true);

        {
            let mut host = HostContext {
                delta_time: 1.0,
                engine: &mut th,
            };
            scene.update(1.0, &mut host).unwrap();
        }
        assert!(scene.output().iter().any(|r| r.message == "done"));
    }

    #[test]
    fn test_light_refetch_state() {
        let source = r#"
entity Test {
    fn update(dt: number) {
        const l1 = get_light(10, 10, 10)
        l1.set_enabled(false)
        const l2 = get_light(10, 10, 10)
        if l2.is_enabled() == false { debug.log("refetch_observed_off") }
    }
}
"#;
        let mut th = test_host();
        th.world
            .set_cell(WorldCoord::new(10, 10, 10), CellType::Light);
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(scene.output().iter().any(|r| r.message == "refetch_observed_off"));
    }

    // -------------------------------------------------------------------------
    // Authored World → ScriptScene → RuntimeEntity integration tests
    // -------------------------------------------------------------------------

    fn write_test_script(temp_dir: &Path, file_name: &str, source: &str) {
        let _ = std::fs::remove_dir_all(temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts").join(file_name), source).unwrap();
    }

    #[test]
    fn test_single_authored_entity() {
        let source = r#"
entity Guard01 {
    fn on_spawn() { debug.log("spawned Guard01") }
}
"#;
        let mut th = test_host();
        let coord = WorldCoord::new(10, 5, 10);
        add_authored_entity(&mut th.world, coord, CellType::NPC, "Guard01");

        let temp_dir = std::env::temp_dir().join("aeo_test_single_authored_entity");
        write_test_script(&temp_dir, "guard.aeo", source);
        let bindings = vec![ScriptBinding::new("Guard01", "scripts/guard.aeo")];

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();

        assert_eq!(scene.entities.len(), 1);
        let id = th.entity_manager.lookup_entity("Guard01").unwrap();
        assert_eq!(
            th.entity_manager.get_position(id),
            Some(Vec3::new(10.0, 5.0, 10.0))
        );

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        assert!(scene.output().iter().any(|r| r.message == "spawned Guard01"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_authored_position_transfer() {
        let source = r#"
entity Player {
    fn on_spawn() {
        const e = get_entity("Player")
        const p = e.position
        debug.log(p[0])
        debug.log(p[1])
        debug.log(p[2])
    }
}
"#;
        let mut th = test_host();
        add_authored_entity(
            &mut th.world,
            WorldCoord::new(10, 5, 20),
            CellType::Player,
            "Player",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_authored_position_transfer");
        write_test_script(&temp_dir, "player.aeo", source);
        let bindings = vec![ScriptBinding::new("Player", "scripts/player.aeo")];

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();

        let id = th.entity_manager.lookup_entity("Player").unwrap();
        assert_eq!(
            th.entity_manager.get_position(id),
            Some(Vec3::new(10.0, 5.0, 20.0))
        );

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "10"));
        assert!(output.iter().any(|r| r.message == "5"));
        assert!(output.iter().any(|r| r.message == "20"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_multiple_instances() {
        let source = r#"
entity Guard01 {
    fn on_spawn() {
        const e = get_entity("Guard01")
        const p = e.position
        debug.log(e.name)
        debug.log(p[0])
        debug.log(p[1])
        debug.log(p[2])
    }
}
entity Guard02 {
    fn on_spawn() {
        const e = get_entity("Guard02")
        const p = e.position
        debug.log(e.name)
        debug.log(p[0])
        debug.log(p[1])
        debug.log(p[2])
    }
}
"#;
        let mut th = test_host();
        add_authored_entity(
            &mut th.world,
            WorldCoord::new(10, 5, 10),
            CellType::NPC,
            "Guard01",
        );
        add_authored_entity(
            &mut th.world,
            WorldCoord::new(30, 5, 10),
            CellType::NPC,
            "Guard02",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_multiple_instances");
        write_test_script(&temp_dir, "guard.aeo", source);
        let bindings = vec![
            ScriptBinding::new("Guard01", "scripts/guard.aeo"),
            ScriptBinding::new("Guard02", "scripts/guard.aeo"),
        ];

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();

        assert_eq!(scene.entities.len(), 2);
        let id1 = th.entity_manager.lookup_entity("Guard01").unwrap();
        let id2 = th.entity_manager.lookup_entity("Guard02").unwrap();
        assert_ne!(id1, id2);
        assert_eq!(
            th.entity_manager.get_position(id1),
            Some(Vec3::new(10.0, 5.0, 10.0))
        );
        assert_eq!(
            th.entity_manager.get_position(id2),
            Some(Vec3::new(30.0, 5.0, 10.0))
        );

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "Guard01"));
        assert!(output.iter().any(|r| r.message == "Guard02"));
        assert!(output.iter().any(|r| r.message == "30"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_identity_isolation() {
        let source = r#"
entity Guard01 {
    fn on_spawn() { debug.log("spawned Guard01") }
}
entity Guard02 {
    fn on_spawn() { debug.log("spawned Guard02") }
}
"#;
        let mut th = test_host();
        add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::NPC,
            "Guard01",
        );
        add_authored_entity(
            &mut th.world,
            WorldCoord::new(1, 1, 1),
            CellType::NPC,
            "Guard02",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_identity_isolation");
        write_test_script(&temp_dir, "guards.aeo", source);
        let bindings = vec![ScriptBinding::new("Guard01", "scripts/guards.aeo")];

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();

        assert_eq!(scene.entities.len(), 1);
        assert_eq!(scene.entities[0].instance.entity_name(), "Guard01");
        assert!(th.entity_manager.lookup_entity("Guard02").is_none());

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(scene.output().iter().any(|r| r.message == "spawned Guard01"));
        assert!(!scene.output().iter().any(|r| r.message == "spawned Guard02"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_unbound_authored_entity_does_not_spawn() {
        let mut th = test_host();
        add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::NPC,
            "Unbound",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_unbound_authored_entity");
        write_test_script(
            &temp_dir,
            "other.aeo",
            "entity Other { fn on_spawn() { debug.log(\"other\") } }",
        );

        let bindings: Vec<ScriptBinding> = Vec::new();
        let scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();

        assert!(scene.entities.is_empty());
        assert!(th.entity_manager.lookup_entity("Unbound").is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_non_entity_cells_do_not_participate() {
        let mut th = test_host();
        th.world
            .set_cell(WorldCoord::new(0, 0, 0), CellType::Block);
        th.world
            .set_cell(WorldCoord::new(1, 1, 1), CellType::Light);
        th.world
            .set_cell(WorldCoord::new(2, 2, 2), CellType::SpawnPoint);

        let temp_dir = std::env::temp_dir().join("aeo_test_non_entity_cells");
        write_test_script(&temp_dir, "test.aeo", "entity Test {}");

        let scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &[],
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();

        assert!(scene.entities.is_empty());
        assert!(th.entity_manager.lookup_entity("Test").is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_duplicate_authored_identity() {
        let mut th = test_host();
        add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::Player,
            "Duplicate",
        );
        add_authored_entity(
            &mut th.world,
            WorldCoord::new(1, 1, 1),
            CellType::NPC,
            "Duplicate",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_duplicate_authored_identity");
        write_test_script(&temp_dir, "duplicate.aeo", "entity Duplicate {}");
        let bindings = vec![ScriptBinding::new("Duplicate", "duplicate.aeo")];

        let result = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Duplicate authored entity identity found: 'Duplicate'"));
        assert!(th.entity_manager.lookup_entity("Duplicate").is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_duplicate_binding_target_identity() {
        let mut th = test_host();
        add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::NPC,
            "Guard01",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_duplicate_binding_target");
        write_test_script(&temp_dir, "guard.aeo", "entity Guard01 {}");
        let bindings = vec![
            ScriptBinding::new("Guard01", "guard.aeo"),
            ScriptBinding::new("Guard01", "guard.aeo"),
        ];

        let result = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        );

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .contains("Multiple script bindings found for target identity 'Guard01'"));
        assert!(th.entity_manager.lookup_entity("Guard01").is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_stale_binding() {
        let mut th = test_host();
        let temp_dir = std::env::temp_dir().join("aeo_test_stale_binding");
        write_test_script(&temp_dir, "test.aeo", "entity NonExistent {}");
        let bindings = vec![ScriptBinding::new("NonExistent", "test.aeo")];

        let scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        ).expect("Stale binding should not prevent scene construction");

        assert!(scene.output().iter().any(|r| r.message.contains(
            "Stale script binding found: target identity 'NonExistent' does not exist in the world as an authored entity."
        )));
        assert!(th.entity_manager.lookup_entity("NonExistent").is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_discovery_and_bridge() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const lights = getAllCellsOfClass("Light")
        debug.log("Lights:", lights.len())
        if lights.len() > 0 {
            const first = lights[0]
            const obj = first:getObject()
            if obj != nil {
                debug.log("Found object:", obj.name)
            }
        }
    }
}
"#;
        let mut th = test_host();
        let coord1 = WorldCoord::new(10, 10, 10);
        let coord2 = WorldCoord::new(20, 20, 20);
        add_authored_entity(&mut th.world, coord1, CellType::Light, "Light1");
        add_authored_entity(&mut th.world, coord2, CellType::Light, "Light2");

        let mut host = HostContext { delta_time: 0.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "Lights: 2"));
    }

    #[test]
    fn test_get_all_cells_of_class_empty() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const lights = getAllCellsOfClass("Light")
        debug.log("Lights:", lights.len())
    }
}
"#;
        let mut th = test_host();
        let mut host = HostContext { delta_time: 0.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "Lights: 0"));
    }

    #[test]
    fn test_light_set_enabled_on_discovered_cell() {
        let source = r#"
entity Test {
    fn on_spawn() {
        const lights = getAllCellsOfClass("Light")
        if lights.len() > 0 {
            const l = lights[0]
            l.set_enabled(false)
            if l.is_enabled() == false {
                debug.log("Discovered light disabled")
            }
        }
    }
}
"#;
        let mut th = test_host();
        let coord = WorldCoord::new(10, 10, 10);
        add_authored_entity(&mut th.world, coord, CellType::Light, "Light1");

        let mut host = HostContext { delta_time: 0.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "Discovered light disabled"));
        assert_eq!(th.world.is_light_enabled(coord), false);
    }

    #[test]
    fn test_unattached_event_handler_loading() {
        let source = r#"
on GlobalEvent(val) {
    debug.log("Received:", val)
}
"#;
        let mut th = TestHost {
            entity_manager: EntityManager::new(),
            world: World::new(),
        };

        let temp_dir = std::env::temp_dir().join("aeo_test_unattached");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/global.aeo"), source).unwrap();

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &[],
            &mut th.entity_manager,
            0.0,
        ).unwrap();

        let mut host = HostContext { delta_time: 0.0, engine: &mut th };
        scene.start(&mut host).unwrap();

        scene.dispatch_event("GlobalEvent", vec![Value::Number(42.0)], &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(scene.output().iter().any(|r| r.message.contains("Received: 42")));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_event_handler_not_duplicated() {
        let source = r#"
on TestEvent() {
    debug.log("Event Triggered")
}
"#;
        let mut th = TestHost {
            entity_manager: EntityManager::new(),
            world: World::new(),
        };

        let temp_dir = std::env::temp_dir().join("aeo_test_duplication");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/test.aeo"), source).unwrap();

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &[],
            &mut th.entity_manager,
            0.0,
        ).unwrap();

        let mut host = HostContext { delta_time: 0.0, engine: &mut th };
        scene.start(&mut host).unwrap();

        scene.dispatch_event("TestEvent", vec![], &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        let occurrences: Vec<_> = output.iter().filter(|r| r.message == "Event Triggered").collect();

        assert_eq!(occurrences.len(), 1, "Event should only be triggered once");
        assert_eq!(occurrences[0].script_path.as_deref(), Some("scripts/test.aeo"), "Event should have correct script path");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_member_assignment_bridge() {
        use crate::scripting::value::HandleKind;
        let source = r#"
entity Test {
    fn main(b: Cell) {
        b.color = [1.0, 0.2, 0.2]
        b.visible = false
        b.solid = false
    }
}
"#;
        let mut th = test_host();
        let coord = WorldCoord::new(1, 2, 3);
        th.world.set_cell(coord, crate::world::CellType::Block);
        let cell_id = th.world.get(coord).unwrap().id;
        let handle = Value::Handle { kind: HandleKind::Cell, id: cell_id };

        let mut host = HostContext { delta_time: 1.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();

        let mut instance = scene.runtime.interpreter_mut().instantiate_entity("Test", 1, &mut host).unwrap();
        scene.runtime.interpreter_mut().call(&mut instance, "main", vec![handle], &mut host).unwrap();

        let runtime_state = th.world.runtime_state.get(&cell_id).unwrap();
        assert_eq!(runtime_state.color_rgb, Some(glam::Vec3::new(1.0, 0.2, 0.2)));
        assert_eq!(runtime_state.visible, Some(false));
        assert_eq!(runtime_state.solid, Some(false));
    }

    #[test]
    fn test_find_objects_universal_identity() {
        use crate::scripting::api::EngineHost;
        let mut th = test_host();

        let c1 = WorldCoord::new(1, 1, 1);
        let c2 = WorldCoord::new(2, 2, 2);
        let c3 = WorldCoord::new(3, 3, 3);

        th.world.set_cell(c1, crate::world::CellType::Block);
        th.world.get_mut(c1).unwrap().entity_identity = Some("Ghost".to_string());

        th.world.set_cell(c2, crate::world::CellType::Light);
        th.world.get_mut(c2).unwrap().entity_identity = Some("Ghost".to_string());

        th.world.set_cell(c3, crate::world::CellType::NPC);
        th.world.get_mut(c3).unwrap().entity_identity = Some("Ghost".to_string());

        let results = th.find_objects("Ghost");
        assert_eq!(results.len(), 3);

        let mut kinds = results.iter().map(|(k, _)| *k).collect::<Vec<_>>();
        kinds.sort_by_key(|k| format!("{:?}", k));

        // Block -> Cell, Light -> Light, NPC -> Cell
        use crate::scripting::value::HandleKind;
        assert!(kinds.contains(&HandleKind::Cell));
        assert!(kinds.contains(&HandleKind::Light));

        let cell_count = kinds.iter().filter(|&&k| k == HandleKind::Cell).count();
        let light_count = kinds.iter().filter(|&&k| k == HandleKind::Light).count();
        assert_eq!(cell_count, 2);
        assert_eq!(light_count, 1);
    }
}
