use super::api::HostContext;
use super::ast::{Declaration, EntityMember, Program};
use super::binding::ScriptBinding;
use super::execution::{FiberResult, ScriptTaskId};
use super::interpreter::ScriptInstance;
use super::lexer::Lexer;
use super::parser::Parser;
use super::runtime::ScriptRuntime;
use super::source::SourceSpan;
use super::value::Value;
use crate::engine::entity::{EntityId, EntityManager};
use crate::world::{CellType, World, WorldCoord};
use glam::Vec3;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::Path;

use super::log::LogRecord;

/// A single scripted entity's lifecycle state.
#[derive(Debug)]
pub struct ScriptEntity {
    instance: ScriptInstance,
    state: EntityState,
    stopped: bool,
    script_path: Option<String>,
    pub(crate) associated_cell_id: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum EntityState {
    Initial,
    Spawning(Option<ScriptTaskId>),
    Readying(Option<ScriptTaskId>),
    Active { update_task: Option<ScriptTaskId> },
    Stopped,
}

impl ScriptEntity {
    fn new(instance: ScriptInstance, script_path: Option<String>, cell_id: Option<u64>) -> Self {
        Self {
            instance,
            state: EntityState::Initial,
            stopped: false,
            script_path,
            associated_cell_id: cell_id,
        }
    }

    fn active_task(&self) -> Option<ScriptTaskId> {
        match self.state {
            EntityState::Spawning(Some(tid)) => Some(tid),
            EntityState::Readying(Some(tid)) => Some(tid),
            EntityState::Active {
                update_task: Some(tid),
            } => Some(tid),
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
            EntityState::Active {
                update_task: Some(tid),
            } if tid == id => {
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
    pub(crate) entities: Vec<ScriptEntity>,
    pub disabled_scripts: Vec<String>,
    pub loaded_scripts: HashMap<String, Program>,
    pub top_level_spawned: HashSet<String>,
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
        let mut valid_bindings = Vec::new();
        let mut stale_warnings = Vec::new();
        let mut seen_binding_targets = HashSet::new();

        for binding in bindings {
            if world.disabled_scripts.contains(&binding.script_path) {
                continue;
            }

            if !seen_binding_targets.insert(binding.target_identity) {
                return Err(format!(
                    "Multiple script bindings found for target cell ID '{}'. Each authored cell can only have one binding.",
                    binding.target_identity
                ));
            }

            if world.resolve_cell_id(binding.target_identity).is_none() {
                stale_warnings.push(format!(
                    "Stale script binding found: target cell ID '{}' does not exist in the world as an authored cell.",
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
            for entry in std::fs::read_dir(scripts_dir)
                .map_err(|e| format!("Failed to read scripts directory: {}", e))?
                .flatten()
            {
                let path = entry.path();
                if path.is_file() && path.extension().map_or(false, |ext| ext == "aeo") {
                    let source = std::fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read script {:?}: {}", path, e))?;

                    let tokens = Lexer::new(&source)
                        .tokenize()
                        .map_err(|e| format!("Lexer error in {:?}: {:?}", path, e))?;

                    let program = Parser::new(tokens)
                        .parse()
                        .map_err(|e| format!("Parser error in {:?}: {:?}", path, e))?;

                    // Use relative path from project root as the key
                    let relative_path = path
                        .strip_prefix(project_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    // Normalize separators for cross-platform matching with bindings
                    let relative_path = relative_path.replace("\\", "/");

                    loaded_scripts.insert(relative_path, program);
                }
            }
        }

        // 2. Validate explicit bindings.
        for binding in &valid_bindings {
            let coord = world
                .resolve_cell_id(binding.target_identity)
                .expect("binding target existence was verified above");
            let cell = world.get(coord).expect("cell existence was verified above");
            let identity = if let Some(ref id) = cell.entity_identity {
                id.clone()
            } else {
                format!("{:?}", cell.cell_type)
            };

            // Normalize binding path
            let normalized_binding_path = binding.script_path.replace("\\", "/");

            let program = loaded_scripts
                .get(&normalized_binding_path)
                .ok_or_else(|| {
                    format!(
                        "Bound script '{}' not found in scripts directory.",
                        binding.script_path
                    )
                })?;

            let has_entity = program.declarations.iter().any(|decl| {
                if let Declaration::Entity(entity) = decl {
                    entity.name == identity
                } else {
                    false
                }
            });

            if !has_entity {
                return Err(format!(
                    "Script '{}' does not contain an entity declaration for '{}'",
                    binding.script_path, identity
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
            statements: Vec::new(),
        };

        let spawn_params: Vec<(String, u64, Option<String>, Option<u64>)> = valid_bindings
            .iter()
            .filter(|binding| binding.enabled)
            .map(|binding| {
                let coord = world.resolve_cell_id(binding.target_identity).unwrap();
                let cell = world.get(coord).unwrap();
                let identity = if let Some(ref id) = cell.entity_identity {
                    id.clone()
                } else {
                    format!("{:?}", cell.cell_type)
                };

                let id = entity_manager.create_entity(&identity);
                entity_manager.set_position(
                    id,
                    Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32),
                );
                (
                    identity,
                    id.0,
                    Some(binding.script_path.clone()),
                    Some(binding.target_identity),
                )
            })
            .collect();

        let created_ids: Vec<EntityId> = spawn_params
            .iter()
            .map(|(_, raw_id, _, _)| EntityId(*raw_id))
            .collect();

        let scene_res = {
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
        scene.disabled_scripts = world.disabled_scripts.clone();

        // Add event handlers from all loaded scripts with their correct paths.
        // We clear the default auto-registered handlers (which have UNKNOWN path)
        // first to avoid duplication.
        scene.runtime.clear_event_handlers();
        for (path, program) in &loaded_scripts {
            scene.runtime.add_event_handlers(path.clone(), program);
        }

        let mut top_level_spawned = HashSet::new();

        // Spawn top-level fibers for each script.
        for (path, program) in &loaded_scripts {
            if world.disabled_scripts.contains(path) {
                continue;
            }
            top_level_spawned.insert(path.clone());
            if let Err(e) = scene
                .runtime
                .spawn_top_level_custom(path.clone(), &program.statements)
            {
                scene.runtime.interpreter_mut().log_error(format!(
                    "Failed to spawn top-level fiber for {}: {}",
                    path, e
                ));
            }
        }

        scene.loaded_scripts = loaded_scripts.clone();
        scene.top_level_spawned = top_level_spawned;

        // Lifecycle hook warnings for unattached entities.
        let bound_entities: HashSet<(String, String)> = valid_bindings
            .iter()
            .map(|binding| {
                let coord = world.resolve_cell_id(binding.target_identity).unwrap();
                let cell = world.get(coord).unwrap();
                let identity = cell
                    .entity_identity
                    .clone()
                    .unwrap_or_else(|| format!("{:?}", cell.cell_type));
                (binding.script_path.clone(), identity)
            })
            .collect();

        for (path, program) in &loaded_scripts {
            for decl in &program.declarations {
                if let Declaration::Entity(entity) = decl {
                    let hooks: Vec<&str> = entity
                        .members
                        .iter()
                        .filter_map(|m| {
                            if let EntityMember::Function(f) = m {
                                if matches!(
                                    f.name.as_str(),
                                    "on_spawn" | "on_ready" | "update" | "on_destroy"
                                ) {
                                    return Some(f.name.as_str());
                                }
                            }
                            None
                        })
                        .collect();

                    if !hooks.is_empty()
                        && !bound_entities.contains(&(path.clone(), entity.name.clone()))
                    {
                        let warning = format!(
                            "Script '{}' declares lifecycle hooks for entity '{}', but no script binding exists for that entity. Hooks: {}",
                            path,
                            entity.name,
                            hooks.join(", ")
                        );
                        scene.runtime.interpreter_mut().log_warning(warning);
                    }
                }
            }
        }

        // Log stale bindings as warnings
        for warning in stale_warnings {
            scene.runtime.interpreter_mut().log_warning(warning);
        }

        Ok(scene)
    }

    pub fn new(
        program: Program,
        entities_to_spawn: Vec<(String, u64, Option<String>, Option<u64>)>,
        host: &mut HostContext,
    ) -> Result<Self, String> {
        let has_top_level = !program.statements.is_empty();
        let mut runtime = ScriptRuntime::new(program);
        let mut entities = Vec::with_capacity(entities_to_spawn.len());

        for (name, id, script_path, cell_id) in entities_to_spawn {
            let mut instance = runtime
                .interpreter_mut()
                .instantiate_entity(&name, id, host)
                .map_err(|e| format!("Failed to instantiate script entity '{}': {}", name, e))?;
            instance.script_path = script_path.clone();
            entities.push(ScriptEntity::new(instance, script_path, cell_id));
        }

        // Spawn top-level fiber if there are any statements in the program.
        if has_top_level {
            let statements = runtime.interpreter().program().statements.clone();
            runtime.spawn_top_level_custom("UNKNOWN".to_string(), &statements)?;
        }

        Ok(Self {
            runtime,
            entities,
            disabled_scripts: Vec::new(),
            loaded_scripts: HashMap::new(),
            top_level_spawned: HashSet::new(),
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

    pub fn normalize_script_path(path: &str) -> String {
        let clean = path.replace("\\", "/");
        if clean.starts_with("scripts/") {
            clean
        } else {
            format!("scripts/{}", clean)
        }
    }

    pub fn enable_script(&mut self, path: &str) {
        let norm = Self::normalize_script_path(path);
        self.disabled_scripts.retain(|s| s != &norm);
        eprintln!("[RUNTIME] script enabled: {}", norm);

        if !self.top_level_spawned.contains(&norm) {
            if let Some(program) = self.loaded_scripts.get(&norm).cloned() {
                self.top_level_spawned.insert(norm.clone());
                if let Err(e) = self
                    .runtime
                    .spawn_top_level_custom(norm.clone(), &program.statements)
                {
                    self.runtime.interpreter_mut().log_error(format!(
                        "Failed to spawn top-level fiber for {}: {}",
                        norm, e
                    ));
                }
            }
        }
    }

    pub fn disable_script(&mut self, path: &str) {
        let norm = Self::normalize_script_path(path);
        if !self.disabled_scripts.contains(&norm) {
            self.disabled_scripts.push(norm.clone());
        }
        self.top_level_spawned.remove(&norm);
        eprintln!("[RUNTIME] script disabled: {}", norm);
        self.runtime.cancel_fibers_for_script(&norm);
    }

    pub fn sync_script_states(&mut self, host: &mut HostContext) {
        let enabled = host.engine.drain_enabled_scripts();
        for path in enabled {
            self.enable_script(&path);
        }

        let disabled = host.engine.drain_disabled_scripts();
        for path in disabled {
            self.disable_script(&path);
        }
    }

    /// Advances the scene time and executes script fibers.
    pub fn update(&mut self, dt: f32, host: &mut HostContext) -> Result<(), String> {
        self.current_time += dt as f64;

        // Sync runtime enable/disable requests
        self.sync_script_states(host);

        // Drain and dispatch pending script events (e.g. from event.fire or fire_event)
        let pending = host.engine.drain_pending_events();
        for (event_name, args) in pending {
            let _ = self.dispatch_event(&event_name, args, host);
        }

        let tick_results = self.runtime.tick(self.current_time, host)?;

        for (task_id, result) in tick_results {
            let mut handled = false;
            for entity in &mut self.entities {
                if entity.active_task() == Some(task_id) {
                    // Synchronize the mutated instance state back to the persistent entity.
                    Self::sync_entity_instance_from_task(&self.runtime, entity, task_id)?;

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

    /// Handles player contact events for Cells.
    pub fn on_player_contact(
        &mut self,
        contacted_cells: &HashSet<u64>,
        host: &mut HostContext,
    ) -> Result<(), String> {
        // Contact collection order must not determine script execution order.
        // Sort the IDs so multiple simultaneous contacts are deterministic.
        let mut cell_ids = contacted_cells.iter().copied().collect::<Vec<_>>();
        cell_ids.sort_unstable();

        for cell_id in cell_ids {
            if !host.engine.is_collision_events_enabled(cell_id) {
                continue;
            }

            let cell_handle = Value::Handle {
                kind: crate::scripting::value::HandleKind::Cell,
                id: cell_id,
            };

            // 1. Global handlers.
            self.dispatch_event("on_touch", vec![cell_handle.clone()], host)?;

            // 2. Dynamic handle handler. Closures retain the lexical scopes
            // captured when the function value was assigned to the handle.
            self.spawn_dynamic_handler(
                crate::scripting::value::HandleKind::Cell,
                cell_id,
                "on_touch",
                vec![cell_handle.clone()],
                Some(cell_handle.clone()),
                host,
            )?;

            // 3. Bound entity handler.
            for entity in &mut self.entities {
                if entity.associated_cell_id == Some(cell_id) && !entity.stopped {
                    if Self::has_function(&self.runtime, entity, "on_touch") {
                        self.runtime.spawn(
                            entity.instance.clone(),
                            "on_touch",
                            vec![cell_handle.clone()],
                        )?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Handles player overlap events for Cells.
    pub fn on_player_overlap(
        &mut self,
        cell_id: u64,
        overlapping: bool,
        host: &mut HostContext,
    ) -> Result<(), String> {
        if !host.engine.is_collision_events_enabled(cell_id) {
            return Ok(());
        }

        let cell_handle = Value::Handle {
            kind: crate::scripting::value::HandleKind::Cell,
            id: cell_id,
        };
        let overlapping_value = Value::Bool(overlapping);

        // 1. Global handlers.
        self.dispatch_event(
            "on_overlap",
            vec![overlapping_value.clone(), cell_handle.clone()],
            host,
        )?;

        // 2. Dynamic handle handler with its captured lexical environment.
        self.spawn_dynamic_handler(
            crate::scripting::value::HandleKind::Cell,
            cell_id,
            "on_overlap",
            vec![overlapping_value.clone(), cell_handle.clone()],
            Some(cell_handle.clone()),
            host,
        )?;

        // 3. Bound entity handler.
        for entity in &mut self.entities {
            if entity.associated_cell_id == Some(cell_id) && !entity.stopped {
                if Self::has_function(&self.runtime, entity, "on_overlap") {
                    self.runtime.spawn(
                        entity.instance.clone(),
                        "on_overlap",
                        vec![overlapping_value.clone(), cell_handle.clone()],
                    )?;
                }
            }
        }

        Ok(())
    }

    fn spawn_dynamic_handler(
        &mut self,
        kind: crate::scripting::value::HandleKind,
        id: u64,
        name: &str,
        arguments: Vec<Value>,
        self_value: Option<Value>,
        host: &mut HostContext,
    ) -> Result<(), String> {
        let Some(Value::Function(compiled, params, captured_scopes)) =
            host.engine.get_script_property(kind, id, name)
        else {
            return Ok(());
        };

        let instance = ScriptInstance::new_empty();
        let fiber = self.runtime.interpreter_mut().start_direct_fiber(
            instance,
            compiled,
            params,
            arguments,
            self_value,
            name.to_string(),
            captured_scopes,
        )?;

        let task_id = self.runtime.scheduler_mut().spawn();
        self.runtime.fibers_mut().insert(task_id, fiber);

        Ok(())
    }

    /// Dispatches a global event to all scripts in the scene.
    pub fn dispatch_event(
        &mut self,
        name: &str,
        arguments: Vec<Value>,
        host: &mut HostContext,
    ) -> Result<(), String> {
        eprintln!(
            "[RUNTIME] dispatching event '{}' (disabled_scripts: {:?})",
            name, self.disabled_scripts
        );
        self.runtime
            .dispatch_event(name, arguments, host, &self.disabled_scripts)
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
                let _ =
                    self.runtime
                        .interpreter_mut()
                        .call(&mut instance, "on_destroy", vec![], host);
            }

            entity.state = EntityState::Stopped;
        }
        self.entities.clear();
    }

    fn sync_entity_instance_from_task(
        runtime: &ScriptRuntime,
        entity: &mut ScriptEntity,
        task_id: ScriptTaskId,
    ) -> Result<(), String> {
        if let Some(fiber) = runtime.fiber(task_id) {
            entity.instance = fiber.instance().clone();
            Ok(())
        } else {
            Err(format!(
                "Failed to sync instance: task {} not found",
                task_id.value()
            ))
        }
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
                        let task_id = runtime.spawn(entity.instance.clone(), "on_spawn", vec![])?;
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
                        let task_id = runtime.spawn(entity.instance.clone(), "on_ready", vec![])?;
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
pub(crate) mod tests {
    use super::*;
    use crate::engine::entity::{EntityId, EntityManager};
    use crate::scripting::api::EngineHost;
    use crate::scripting::lexer::Lexer;
    use crate::scripting::parser::Parser;
    use glam::Vec3;

    pub(crate) struct TestHost {
        pub(crate) entity_manager: EntityManager,
        pub(crate) world: World,
        pub(crate) dynamic_properties: HashMap<
            (crate::scripting::value::HandleKind, u64),
            BTreeMap<String, crate::scripting::value::Value>,
        >,
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
                                crate::world::CellType::Light => {
                                    crate::scripting::value::HandleKind::Light
                                }
                                _ => crate::scripting::value::HandleKind::Cell,
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
            _kind: crate::scripting::value::HandleKind,
            _id: u64,
        ) -> Vec<(crate::scripting::value::HandleKind, u64)> {
            Vec::new()
        }

        fn get_parent(
            &self,
            _kind: crate::scripting::value::HandleKind,
            _id: u64,
        ) -> Option<(crate::scripting::value::HandleKind, u64)> {
            None
        }

        fn get_cell_object(
            &self,
            cell_id: u64,
        ) -> Option<(crate::scripting::value::HandleKind, u64)> {
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

        fn get_property(
            &self,
            kind: crate::scripting::value::HandleKind,
            id: u64,
            name: &str,
        ) -> Result<Option<crate::scripting::value::Value>, String> {
            use crate::scripting::value::{HandleKind, MapKey, Value};
            use crate::world::cell::AttributeValue;
            use std::collections::BTreeMap;

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
                                    return Ok(Some(Value::Bool(
                                        self.world.is_cell_visible(coord),
                                    )));
                                } else {
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
                                    return Ok(Some(Value::Bool(
                                        self.world.is_light_enabled(coord),
                                    )));
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
                                    return Ok(Some(Value::Bool(
                                        self.world.is_cell_anchored(coord),
                                    )));
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
                                for (key, attr) in &cell.attributes {
                                    let val = match attr {
                                        AttributeValue::Number(n) => Value::Number(*n),
                                        AttributeValue::Bool(b) => Value::Bool(*b),
                                        AttributeValue::String(s) => Value::String(s.clone()),
                                    };
                                    map.insert(MapKey::String(key.clone()), val);
                                }
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
            kind: crate::scripting::value::HandleKind,
            id: u64,
            name: &str,
            value: crate::scripting::value::Value,
        ) -> Result<bool, String> {
            use crate::scripting::value::HandleKind;
            if let Some(_cell) = self.world.get_effective_cell_by_id(id) {
                match kind {
                    HandleKind::Cell | HandleKind::Light => match name {
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
                            if borrowed.elements.len() == 3 {
                                let r = borrowed.elements[0].as_number()? as f32;
                                let g = borrowed.elements[1].as_number()? as f32;
                                let b = borrowed.elements[2].as_number()? as f32;
                                let color = glam::Vec3::new(r, g, b);
                                if let Some(coord) = self.world.resolve_cell_id(id) {
                                    self.world.set_cell_color_runtime(coord, color);
                                } else {
                                    self.world.runtime_state.entry(id).or_default().color_rgb =
                                        Some(color);
                                }
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
                        _ => {}
                    },
                    _ => {}
                }
            }
            Ok(false)
        }

        fn cell_exists(&self, id: u64) -> bool {
            self.world.get_effective_cell_by_id(id).is_some()
        }

        fn entity_exists(&self, id: u64) -> bool {
            self.entity_manager.validate_handle(id)
        }

        fn get_script_property(
            &self,
            kind: crate::scripting::value::HandleKind,
            id: u64,
            name: &str,
        ) -> Option<crate::scripting::value::Value> {
            self.dynamic_properties
                .get(&(kind, id))
                .and_then(|m| m.get(name))
                .cloned()
        }
        fn set_script_property(
            &mut self,
            kind: crate::scripting::value::HandleKind,
            id: u64,
            name: String,
            value: crate::scripting::value::Value,
        ) {
            self.dynamic_properties
                .entry((kind, id))
                .or_default()
                .insert(name, value);
        }

        fn call_method(
            &mut self,
            _kind: crate::scripting::value::HandleKind,
            _id: u64,
            _name: &str,
            _args: &[crate::scripting::value::Value],
        ) -> Result<Option<crate::scripting::value::Value>, String> {
            Ok(None)
        }

        fn set_attribute(
            &mut self,
            id: u64,
            key: String,
            value: crate::scripting::value::Value,
        ) -> Result<(), String> {
            if self.world.get_effective_cell_by_id(id).is_some() {
                use crate::scripting::value::Value;
                use crate::world::cell::AttributeValue;
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

        fn create_runtime_cell(
            &mut self,
            cell_type: &str,
        ) -> Result<(crate::scripting::value::HandleKind, u64), String> {
            let ct = match cell_type {
                "Block" => crate::world::CellType::Block,
                "FxBlock" => crate::world::CellType::FxBlock,
                "Player" => crate::world::CellType::Player,
                "NPC" => crate::world::CellType::NPC,
                "Light" => crate::world::CellType::Light,
                "SpawnPoint" => crate::world::CellType::SpawnPoint,
                "Empty" => return Err("Cannot create Empty cell".to_string()),
                _ => return Err(format!("Unknown cell type: {}", cell_type)),
            };

            let id = self.world.create_runtime_cell(ct);
            let kind = if ct == crate::world::CellType::Light {
                crate::scripting::value::HandleKind::Light
            } else {
                crate::scripting::value::HandleKind::Cell
            };

            Ok((kind, id))
        }

        fn move_runtime_cell(&mut self, id: u64, x: i32, y: i32, z: i32) -> Result<(), String> {
            self.world
                .move_runtime_cell(id, crate::world::WorldCoord::new(x, y, z))
        }

        fn delete_cell(&mut self, id: u64) -> Result<(), String> {
            self.world.delete_cell_runtime(id);
            Ok(())
        }
    }

    pub(crate) fn test_host() -> TestHost {
        TestHost {
            entity_manager: EntityManager::new(),
            world: World::new(),
            dynamic_properties: HashMap::new(),        }
    }

    pub(crate) fn create_scene(source: &str, host: &mut HostContext) -> ScriptScene {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let entities_to_spawn: Vec<(String, u64, Option<String>, Option<u64>)> = program
            .declarations
            .iter()
            .filter_map(|decl| {
                if let Declaration::Entity(entity) = decl {
                    let id = host
                        .engine
                        .entity_manager()
                        .lookup_entity(&entity.name)
                        .unwrap_or(EntityId(0));
                    Some((entity.name.clone(), id.0, None, None))
                } else {
                    None
                }
            })
            .collect();

        ScriptScene::new(program, entities_to_spawn, host).unwrap()
    }

    pub(crate) fn add_authored_entity(
        world: &mut World,
        coord: WorldCoord,
        cell_type: CellType,
        identity: &str,
    ) -> u64 {
        let id = world.set_cell(coord, cell_type);
        if let Some(cell) = world.get_mut(coord) {
            cell.entity_identity = Some(identity.to_string());
        }
        id
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
        assert_eq!(th.entity_manager.get_position(id_other), Some(Vec3::ZERO));
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
        let scene = ScriptScene::new(
            program,
            vec![("Player".to_string(), id.0, None, None)],
            &mut host,
        )
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
        let mut scene = ScriptScene::new(
            program,
            vec![("Bound".to_string(), id.0, None, None)],
            &mut host,
        )
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
            vec![
                ("A".to_string(), id_a.0, None, None),
                ("B".to_string(), id_b.0, None, None),
            ],
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
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::NPC,
            "Missing",
        );
        let temp_dir = std::env::temp_dir().join("aeo_test_missing_entity_decl");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        let script_path = "test.aeo";
        std::fs::write(temp_dir.join("scripts").join(script_path), source).unwrap();

        let bindings = vec![ScriptBinding::new(
            cell_id,
            format!("scripts/{}", script_path),
        )];
        let result = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            1.0,
        );

        assert!(result.is_err());
        assert!(result.unwrap_err().contains(
            "Script 'scripts/test.aeo' does not contain an entity declaration for 'Missing'"
        ));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_from_bindings_missing_script_fails() {
        let mut th = test_host();
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::Player,
            "Player",
        );
        let temp_dir = std::env::temp_dir().join("aeo_test_missing");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();

        let bindings = vec![ScriptBinding::new(cell_id, "nonexistent.aeo")];
        let result = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            1.0,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Bound script 'nonexistent.aeo' not found")
        );
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_from_bindings_invalid_script_fails() {
        let mut th = test_host();
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::NPC,
            "Broken",
        );
        let temp_dir = std::env::temp_dir().join("aeo_test_invalid");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();

        let script_path = "broken.aeo";
        std::fs::write(
            temp_dir.join("scripts").join(script_path),
            "entity Broken { !!! }",
        )
        .unwrap();
        let bindings = vec![ScriptBinding::new(cell_id, script_path)];
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
        th.world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);
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

        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message == "refetch_observed_off")
        );
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
        let cell_id = add_authored_entity(&mut th.world, coord, CellType::NPC, "Guard01");

        let temp_dir = std::env::temp_dir().join("aeo_test_single_authored_entity");
        write_test_script(&temp_dir, "guard.aeo", source);
        let bindings = vec![ScriptBinding::new(cell_id, "scripts/guard.aeo")];

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
        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message == "spawned Guard01")
        );

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
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(10, 5, 20),
            CellType::Player,
            "Player",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_authored_position_transfer");
        write_test_script(&temp_dir, "player.aeo", source);
        let bindings = vec![ScriptBinding::new(cell_id, "scripts/player.aeo")];

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
        let id1 = add_authored_entity(
            &mut th.world,
            WorldCoord::new(10, 5, 10),
            CellType::NPC,
            "Guard01",
        );
        let id2 = add_authored_entity(
            &mut th.world,
            WorldCoord::new(30, 5, 10),
            CellType::NPC,
            "Guard02",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_multiple_instances");
        write_test_script(&temp_dir, "guard.aeo", source);
        let bindings = vec![
            ScriptBinding::new(id1, "scripts/guard.aeo"),
            ScriptBinding::new(id2, "scripts/guard.aeo"),
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
        let rid1 = th.entity_manager.lookup_entity("Guard01").unwrap();
        let rid2 = th.entity_manager.lookup_entity("Guard02").unwrap();
        assert_ne!(rid1, rid2);
        assert_eq!(
            th.entity_manager.get_position(rid1),
            Some(Vec3::new(10.0, 5.0, 10.0))
        );
        assert_eq!(
            th.entity_manager.get_position(rid2),
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
        let cell_id1 = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::NPC,
            "Guard01",
        );
        let _cell_id2 = add_authored_entity(
            &mut th.world,
            WorldCoord::new(1, 1, 1),
            CellType::NPC,
            "Guard02",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_identity_isolation");
        write_test_script(&temp_dir, "guards.aeo", source);
        let bindings = vec![ScriptBinding::new(cell_id1, "scripts/guards.aeo")];

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

        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message == "spawned Guard01")
        );
        assert!(
            !scene
                .output()
                .iter()
                .any(|r| r.message == "spawned Guard02")
        );

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
    fn test_cells_without_identity_do_not_participate() {
        let mut th = test_host();
        th.world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);
        th.world.set_cell(WorldCoord::new(1, 1, 1), CellType::Light);
        th.world
            .set_cell(WorldCoord::new(2, 2, 2), CellType::SpawnPoint);

        let temp_dir = std::env::temp_dir().join("aeo_test_non_entity_cells");
        write_test_script(&temp_dir, "test.aeo", "entity Test {}");

        let scene =
            ScriptScene::load_from_bindings(&temp_dir, &th.world, &[], &mut th.entity_manager, 0.0)
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

        // This is now fine. Multiple cells can have the same identity.
        // We just verify we can load without error.
        let result =
            ScriptScene::load_from_bindings(&temp_dir, &th.world, &[], &mut th.entity_manager, 0.0);

        assert!(result.is_ok());
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_duplicate_binding_target_identity() {
        let mut th = test_host();
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::NPC,
            "Guard01",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_duplicate_binding_target");
        write_test_script(&temp_dir, "guard.aeo", "entity Guard01 {}");
        let bindings = vec![
            ScriptBinding::new(cell_id, "guard.aeo"),
            ScriptBinding::new(cell_id, "guard.aeo"),
        ];

        let result = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Multiple script bindings found for target cell ID")
        );
        assert!(th.entity_manager.lookup_entity("Guard01").is_none());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_stale_binding() {
        let mut th = test_host();
        let temp_dir = std::env::temp_dir().join("aeo_test_stale_binding");
        write_test_script(&temp_dir, "test.aeo", "entity NonExistent {}");
        let bindings = vec![ScriptBinding::new(99999999, "test.aeo")];

        let scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .expect("Stale binding should not prevent scene construction");

        assert!(scene.output().iter().any(|r| r.message.contains(
            "Stale script binding found: target cell ID '99999999' does not exist in the world as an authored cell."
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

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
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
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
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

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(
            output
                .iter()
                .any(|r| r.message == "Discovered light disabled")
        );
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
            dynamic_properties: HashMap::new(),
        };

        let temp_dir = std::env::temp_dir().join("aeo_test_unattached");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/global.aeo"), source).unwrap();

        let mut scene =
            ScriptScene::load_from_bindings(&temp_dir, &th.world, &[], &mut th.entity_manager, 0.0)
                .unwrap();

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();

        scene
            .dispatch_event("GlobalEvent", vec![Value::Number(42.0)], &mut host)
            .unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message.contains("Received: 42"))
        );
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
            dynamic_properties: HashMap::new(),
        };

        let temp_dir = std::env::temp_dir().join("aeo_test_duplication");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/test.aeo"), source).unwrap();

        let mut scene =
            ScriptScene::load_from_bindings(&temp_dir, &th.world, &[], &mut th.entity_manager, 0.0)
                .unwrap();

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();

        scene
            .dispatch_event("TestEvent", vec![], &mut host)
            .unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        let occurrences: Vec<_> = output
            .iter()
            .filter(|r| r.message == "Event Triggered")
            .collect();

        assert_eq!(occurrences.len(), 1, "Event should only be triggered once");
        assert_eq!(
            occurrences[0].script_path.as_deref(),
            Some("scripts/test.aeo"),
            "Event should have correct script path"
        );

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
        let handle = Value::Handle {
            kind: HandleKind::Cell,
            id: cell_id,
        };

        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();

        let mut instance = scene
            .runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();
        scene
            .runtime
            .interpreter_mut()
            .call(&mut instance, "main", vec![handle], &mut host)
            .unwrap();

        let runtime_state = th.world.runtime_state.get(&cell_id).unwrap();
        assert_eq!(
            runtime_state.color_rgb,
            Some(glam::Vec3::new(1.0, 0.2, 0.2))
        );
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

    #[test]
    fn test_save_change_restore_workflow() {
        use crate::scripting::value::HandleKind;
        let source = r#"
entity Test {
    fn main() {
        const blocks = getAllCellsOfClass("Block")
        const saved = {}

        for b in blocks {
            saved[b.id] = b.color
        }

        for b in blocks {
            b.color = [1, 0, 0]
        }

        // Verify they changed to red
        for b in blocks {
            if b.color != [1, 0, 0] {
                debug.log("Error: not red")
            }
        }

        for b in blocks {
            b.color = saved[b.id]
        }
    }
}
"#;
        let mut th = test_host();
        let c1 = WorldCoord::new(1, 1, 1);
        let c2 = WorldCoord::new(2, 2, 2);
        th.world.set_cell(c1, crate::world::CellType::Block);
        th.world.set_cell(c2, crate::world::CellType::Block);
        let id1 = th.world.get(c1).unwrap().id;
        let id2 = th.world.get(c2).unwrap().id;
        th.world.get_mut(c1).unwrap().color_rgb = glam::Vec3::new(0.1, 0.2, 0.3);
        th.world.get_mut(c2).unwrap().color_rgb = glam::Vec3::new(0.4, 0.5, 0.6);

        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        let mut instance = scene
            .runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();
        scene
            .runtime
            .interpreter_mut()
            .call(&mut instance, "main", vec![], &mut host)
            .unwrap();

        let rs1 = th.world.runtime_state.get(&id1).unwrap();
        let rs2 = th.world.runtime_state.get(&id2).unwrap();
        assert_eq!(rs1.color_rgb, Some(glam::Vec3::new(0.1, 0.2, 0.3)));
        assert_eq!(rs2.color_rgb, Some(glam::Vec3::new(0.4, 0.5, 0.6)));
        assert!(!scene.output().iter().any(|r| r.message.contains("Error")));
    }

    #[test]
    fn test_compound_assignment() {
        let source = r#"
entity Test {
    fn main() {
        x_loc: number = 10
        x_loc += 5
        debug.log("x", x_loc)

        const a = [1, 2, 3]
        a[0] *= 10
        debug.log("a[0]", a[0])

        const m = {"val": 100}
        m["val"] -= 50
        debug.log("m.val", m["val"])
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
        let mut instance = scene
            .runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();
        scene
            .runtime
            .interpreter_mut()
            .call(&mut instance, "main", vec![], &mut host)
            .unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "x 15"));
        assert!(output.iter().any(|r| r.message == "a[0] 10"));
        assert!(output.iter().any(|r| r.message == "m.val 50"));
    }

    #[test]
    fn test_nested_mutations() {
        let source = r#"
entity Test {
    fn main() {
        const data = {
            "colors": [[1, 0, 0], [0, 1, 0]],
            "meta": {"id": 42}
        }

        data["colors"][0][0] = 0.5
        data["meta"]["name"] = "Ghost"

        debug.log("data.colors[0][0]", data["colors"][0][0])
        debug.log("data.meta.name", data["meta"]["name"])
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
        let mut instance = scene
            .runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();
        scene
            .runtime
            .interpreter_mut()
            .call(&mut instance, "main", vec![], &mut host)
            .unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "data.colors[0][0] 0.5"));
        assert!(output.iter().any(|r| r.message == "data.meta.name Ghost"));
    }

    #[test]
    fn test_map_key_types() {
        let source = r#"
entity Test {
    fn main() {
        const m_loc = {}
        m_loc[1] = "number"
        m_loc["1"] = "string"

        debug.log("m[1]", m_loc[1])
        debug.log("m['1']", m_loc["1"])
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
        let mut instance = scene
            .runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();
        scene
            .runtime
            .interpreter_mut()
            .call(&mut instance, "main", vec![], &mut host)
            .unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "m[1] number"));
        assert!(output.iter().any(|r| r.message == "m['1'] string"));
    }

    #[test]
    fn test_truthiness() {
        use crate::scripting::value::HandleKind;
        let source = r#"
entity Test {
    fn main(h: handle) {
        if nil { debug.log("Error: nil is true") } else { debug.log("nil is false") }
        if true { debug.log("true is true") }
        if false { debug.log("Error: false is true") }
        if h { debug.log("handle is true") }
    }
}
"#;
        let mut th = test_host();
        let handle = Value::Handle {
            kind: HandleKind::Entity,
            id: 1,
        };
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        let mut instance = scene
            .runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();
        scene
            .runtime
            .interpreter_mut()
            .call(&mut instance, "main", vec![handle], &mut host)
            .unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "nil is false"));
        assert!(output.iter().any(|r| r.message == "true is true"));
        assert!(output.iter().any(|r| r.message == "handle is true"));
        assert!(!output.iter().any(|r| r.message.contains("Error")));
    }

    #[test]
    fn test_array_indexing_and_mutation() {
        let source = r#"
entity Test {
    fn main() {
        const a = [10, 20, 30]

        // Read
        debug.log("a[0]", a[0])

        // Write
        a[1] = 50
        debug.log("a[1]", a[1])

        // Compound Write
        a[2] += 10
        debug.log("a[2]", a[2])

        // Nested
        const nested = [[1, 2], [3, 4]]
        debug.log("nested[1][0]", nested[1][0])
        nested[1][0] = 99
        debug.log("nested[1][0] mutated", nested[1][0])
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
        let mut instance = scene
            .runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();
        scene
            .runtime
            .interpreter_mut()
            .call(&mut instance, "main", vec![], &mut host)
            .unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "a[0] 10"));
        assert!(output.iter().any(|r| r.message == "a[1] 50"));
        assert!(output.iter().any(|r| r.message == "a[2] 40"));
        assert!(output.iter().any(|r| r.message == "nested[1][0] 3"));
        assert!(
            output
                .iter()
                .any(|r| r.message == "nested[1][0] mutated 99")
        );
    }

    #[test]
    fn test_array_indexing_errors() {
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 1.0,
            engine: &mut th,
        };

        let cases = [
            ("const a = [1]\na[1.5]", "non-negative integer"),
            ("const a = [1]\na[-1]", "non-negative integer"),
            ("const a = [1]\na[\"0\"]", "expected number"),
            ("const a = [1]\na[2]", "out of bounds"),
        ];

        for (code, expected_err) in cases {
            let source = format!("entity Test {{ fn main() {{\n{}\n }} }}", code);
            let mut scene = create_scene(&source, &mut host);
            scene.start(&mut host).unwrap();
            let mut instance = scene
                .runtime
                .interpreter_mut()
                .instantiate_entity("Test", 1, &mut host)
                .unwrap();
            let result =
                scene
                    .runtime
                    .interpreter_mut()
                    .call(&mut instance, "main", vec![], &mut host);
            assert!(result.is_err(), "Expected error for: {}", code);
            assert!(result.unwrap_err().contains(expected_err));
        }
    }

    #[test]
    fn test_collection_reference_semantics() {
        let source = r#"
entity Test {
    fn main() {
        const a = [1, 2, 3]
        const b = a
        b[0] = 99
        debug.log("a[0]", a[0])

        const m = {}
        const n = m
        n["x"] = 10
        debug.log("m.x", m["x"])
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
        let mut instance = scene
            .runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();
        scene
            .runtime
            .interpreter_mut()
            .call(&mut instance, "main", vec![], &mut host)
            .unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "a[0] 99"));
        assert!(output.iter().any(|r| r.message == "m.x 10"));
    }

    #[test]
    fn test_map_literals_generalized() {
        let source = r#"
entity Test {
    fn main() {
        const m = { 1: "number", "1": "string" }
        debug.log("m[1]", m[1])
        debug.log("m['1']", m["1"])
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
        let mut instance = scene
            .runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();
        scene
            .runtime
            .interpreter_mut()
            .call(&mut instance, "main", vec![], &mut host)
            .unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "m[1] number"));
        assert!(output.iter().any(|r| r.message == "m['1'] string"));
    }

    #[test]
    fn test_stale_binding_regression_block_identity() {
        let mut th = test_host();
        let coord = WorldCoord::new(1, 1, 1);

        // 1. Setup a Block with an identity.
        let cell_id = add_authored_entity(
            &mut th.world,
            coord,
            CellType::Block,
            "AeoScriptIntegration",
        );

        let temp_dir = std::env::temp_dir().join("aeo_test_stale_regression");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();

        // 2. Write a script containing the entity declaration.
        std::fs::write(
            temp_dir.join("scripts/test.aeo"),
            "entity AeoScriptIntegration {}",
        )
        .unwrap();

        // 3. Create a binding for that Cell ID.
        let bindings = vec![ScriptBinding::new(cell_id, "scripts/test.aeo")];

        // 4. Load the scene.
        let scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .expect("Binding should be valid for Block with identity");

        // 5. Verify NO stale warning was emitted for this cell ID.
        let output = scene.output();
        let has_stale_warning = output.iter().any(|r| {
            r.message.contains("Stale script binding") && r.message.contains(&cell_id.to_string())
        });
        assert!(
            !has_stale_warning,
            "Should not report stale binding for valid block identity"
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_lifecycle_field_persistence() {
        let source = r#"
entity LifecycleTest {
    ready_set: bool = false
    update_count: number = 0

    fn on_ready() {
        ready_set = true
    }

    fn update(dt: number) {
        if ready_set {
            update_count += 1
        }
    }
}
"#;
        let mut th = test_host();
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::Block,
            "LifecycleTest",
        );
        let temp_dir = std::env::temp_dir().join("aeo_test_lifecycle_persistence");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/test.aeo"), source).unwrap();
        let bindings = vec![ScriptBinding::new(cell_id, "scripts/test.aeo")];

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();

        let mut host = HostContext {
            delta_time: 1.0 / 60.0,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();

        // Frame 1: on_ready completes. Transitions to Active. Spawns update task.
        scene.update(1.0 / 60.0, &mut host).unwrap();

        // Frame 2: first update task ticks. update_count = 1. Spawns next update task.
        scene.update(1.0 / 60.0, &mut host).unwrap();

        // Frame 3: second update task ticks. update_count = 2.
        scene.update(1.0 / 60.0, &mut host).unwrap();

        let entity = &scene.entities[0];
        assert_eq!(
            entity.instance.get_field("ready_set"),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            entity.instance.get_field("update_count"),
            Some(&Value::Number(2.0))
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_update_increment_persistence() {
        let source = r#"
entity Counter {
    ticks: number = 0
    fn update(dt: number) {
        ticks += 1
    }
}
"#;
        let mut th = test_host();
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::Block,
            "Counter",
        );
        let temp_dir = std::env::temp_dir().join("aeo_test_counter_persistence");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/test.aeo"), source).unwrap();
        let bindings = vec![ScriptBinding::new(cell_id, "scripts/test.aeo")];

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();
        let mut host = HostContext {
            delta_time: 0.1,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();

        // Frame 1: Spawns update task.
        scene.update(0.1, &mut host).unwrap();
        // Frame 2: Ticks first update task. ticks = 1. Spawns second.
        scene.update(0.1, &mut host).unwrap();
        assert_eq!(
            scene.entities[0].instance.get_field("ticks"),
            Some(&Value::Number(1.0))
        );

        // Frame 3: Ticks second update task. ticks = 2.
        scene.update(0.1, &mut host).unwrap();
        assert_eq!(
            scene.entities[0].instance.get_field("ticks"),
            Some(&Value::Number(2.0))
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_update_wait_persistence() {
        let source = r#"
entity WaitTest {
    finished: bool = false
    fn update(dt: number) {
        wait(0.05)
        finished = true
    }
}
"#;
        let mut th = test_host();
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::Block,
            "WaitTest",
        );
        let temp_dir = std::env::temp_dir().join("aeo_test_update_wait");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/test.aeo"), source).unwrap();
        let bindings = vec![ScriptBinding::new(cell_id, "scripts/test.aeo")];

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();

        let mut host = HostContext {
            delta_time: 0.1,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();

        // F1: current_time=0.1. tick runs, nothing. Spawns update task.
        scene.update(0.1, &mut host).unwrap();

        // F2: current_time=0.2. update runs, calls wait(0.05), yields. wake_at=0.25.
        scene.update(0.1, &mut host).unwrap();
        assert_eq!(
            scene.entities[0].instance.get_field("finished"),
            Some(&Value::Bool(false))
        );

        // F3: current_time=0.3. wakes, finished=true, complete.
        scene.update(0.1, &mut host).unwrap();

        assert_eq!(
            scene.entities[0].instance.get_field("finished"),
            Some(&Value::Bool(true))
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_update_nested_wait_persistence() {
        let source = r#"
entity NestedWait {
    val: number = 0
    fn sub() {
        wait(0.05)
        val = 1
    }
    fn update(dt: number) {
        if val == 0 {
            sub()
        }
    }
}
"#;
        let mut th = test_host();
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::Block,
            "NestedWait",
        );
        let temp_dir = std::env::temp_dir().join("aeo_test_nested_wait");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/test.aeo"), source).unwrap();
        let bindings = vec![ScriptBinding::new(cell_id, "scripts/test.aeo")];

        let mut scene = ScriptScene::load_from_bindings(
            &temp_dir,
            &th.world,
            &bindings,
            &mut th.entity_manager,
            0.0,
        )
        .unwrap();
        let mut host = HostContext {
            delta_time: 0.1,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();

        // F1: spawns update
        scene.update(0.1, &mut host).unwrap();

        // F2: update -> sub -> wait(0.05). yields. wake_at=0.25.
        scene.update(0.1, &mut host).unwrap();
        assert_eq!(
            scene.entities[0].instance.get_field("val"),
            Some(&Value::Number(0.0))
        );

        // F3: current_time=0.3. wakes, sub sets val=1. complete.
        scene.update(0.1, &mut host).unwrap();
        assert_eq!(
            scene.entities[0].instance.get_field("val"),
            Some(&Value::Number(1.0))
        );

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_cell_attributes_aeoscript_access() {
        use crate::world::cell::AttributeValue;
        let source = r#"
entity Test {
    fn on_spawn() {
        const chests = find("Chest")
        for chest in chests {
            debug.log(chest.attributes["coins"])
            debug.log(chest.attributes["difficulty"])
            debug.log(chest.attributes["locked"])
        }
    }
}
"#;
        let mut th = test_host();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(1, 1, 1);

        add_authored_entity(&mut th.world, c1, CellType::Block, "Chest");
        if let Some(cell) = th.world.get_mut(c1) {
            cell.attributes
                .insert("coins".to_string(), AttributeValue::Number(100.0));
            cell.attributes.insert(
                "difficulty".to_string(),
                AttributeValue::String("hard".to_string()),
            );
            cell.attributes
                .insert("locked".to_string(), AttributeValue::Bool(true));
        }

        add_authored_entity(&mut th.world, c2, CellType::Block, "Chest");
        if let Some(cell) = th.world.get_mut(c2) {
            cell.attributes
                .insert("coins".to_string(), AttributeValue::Number(50.0));
            cell.attributes.insert(
                "difficulty".to_string(),
                AttributeValue::String("easy".to_string()),
            );
            cell.attributes
                .insert("locked".to_string(), AttributeValue::Bool(false));
        }

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();

        // Check for chest 1 values
        assert!(output.iter().any(|r| r.message == "100"));
        assert!(output.iter().any(|r| r.message == "hard"));
        assert!(output.iter().any(|r| r.message == "true"));

        // Check for chest 2 values
        assert!(output.iter().any(|r| r.message == "50"));
        assert!(output.iter().any(|r| r.message == "easy"));
        assert!(output.iter().any(|r| r.message == "false"));
    }

    #[test]
    fn test_top_level_scene_execution() {
        let source = r#"
debug.log("global startup")
"#;
        let mut th = test_host();
        let temp_dir = std::env::temp_dir().join("aeo_test_top_level");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/startup.aeo"), source).unwrap();

        let mut scene =
            ScriptScene::load_from_bindings(&temp_dir, &th.world, &[], &mut th.entity_manager, 0.0)
                .unwrap();

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(scene.output().iter().any(|r| r.message == "global startup"));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_unattached_lifecycle_warning() {
        let source = r#"
entity Unattached {
    fn on_ready() {
        debug.log("ready")
    }
    fn update(dt: number) {}
}
"#;
        let mut th = test_host();
        let temp_dir = std::env::temp_dir().join("aeo_test_unattached_warning");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(temp_dir.join("scripts")).unwrap();
        std::fs::write(temp_dir.join("scripts/unattached.aeo"), source).unwrap();

        let mut scene =
            ScriptScene::load_from_bindings(&temp_dir, &th.world, &[], &mut th.entity_manager, 0.0)
                .unwrap();

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(
            |r| r.severity == crate::scripting::log::LogSeverity::Warning
                && r.message.contains("no script binding exists")
        ));
        assert!(!output.iter().any(|r| r.message == "ready"));
        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_authored_attribute_preservation_regression() {
        use crate::world::cell::AttributeValue;
        let source = r#"
entity Test {
    fn on_spawn() {
        const c = find("Test")[0]
        c.attributes["testBool"] = false
        c.attributes["testNum"] = 42
        c.attributes["Test"] = "runtime"

        debug.log("runtime_bool", c.attributes["testBool"])
        debug.log("runtime_num", c.attributes["testNum"])
        debug.log("runtime_str", c.attributes["Test"])
    }
}
"#;
        let mut th = test_host();
        let coord = WorldCoord::new(0, 0, 0);
        add_authored_entity(&mut th.world, coord, CellType::Block, "Test");

        if let Some(cell) = th.world.get_mut(coord) {
            cell.attributes
                .insert("testBool".to_string(), AttributeValue::Bool(true));
            cell.attributes
                .insert("testNum".to_string(), AttributeValue::Number(10.0));
            cell.attributes.insert(
                "Test".to_string(),
                AttributeValue::String("original".to_string()),
            );
        }

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.iter().any(|r| r.message == "runtime_bool false"));
        assert!(output.iter().any(|r| r.message == "runtime_num 42"));
        assert!(output.iter().any(|r| r.message == "runtime_str runtime"));

        // Verify AUTHORED state in Cell is untouched
        let cell_id = th.world.get(coord).unwrap().id;
        {
            let cell = th.world.get(coord).unwrap();
            assert_eq!(
                cell.attributes.get("testBool"),
                Some(&AttributeValue::Bool(true))
            );
            assert_eq!(
                cell.attributes.get("testNum"),
                Some(&AttributeValue::Number(10.0))
            );
            assert_eq!(
                cell.attributes.get("Test"),
                Some(&AttributeValue::String("original".to_string()))
            );
        }

        // Clear runtime state
        th.world.clear_runtime_state();

        // Effective read should now be authored values again for all types
        let mut dynamic_properties = std::collections::HashMap::new();
        let mut pending_events = Vec::new();
        let mut test_results = std::collections::BTreeMap::new();
        let mut pending_enable_scripts = Vec::new();
        let mut pending_disable_scripts = Vec::new();
        let mut bridge = crate::scripting::host::ScriptHostBridge {
            entity_manager: &mut th.entity_manager,
            world: &mut th.world,
            dynamic_properties: &mut dynamic_properties,
            pending_events: &mut pending_events,
            test_results: &mut test_results,
            pending_enable_scripts: &mut pending_enable_scripts,
            pending_disable_scripts: &mut pending_disable_scripts,
        };
        let effective = bridge
            .get_property(
                crate::scripting::value::HandleKind::Cell,
                cell_id,
                "attributes",
            )
            .unwrap()
            .unwrap();
        let map = effective.as_map().unwrap();
        let borrowed = map.borrow();
        assert_eq!(
            borrowed.get(&crate::scripting::value::MapKey::String(
                "testBool".to_string()
            )),
            Some(&Value::Bool(true))
        );
        assert_eq!(
            borrowed.get(&crate::scripting::value::MapKey::String(
                "testNum".to_string()
            )),
            Some(&Value::Number(10.0))
        );
        assert_eq!(
            borrowed.get(&crate::scripting::value::MapKey::String("Test".to_string())),
            Some(&Value::String("original".to_string()))
        );
    }

    #[test]
    fn test_math_random_api() {
        let source = r#"
debug.log("r0", math.random())
debug.log("r1", math.random(1, 1))
debug.log("r3", math.random(1, 3))
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(program, vec![], &mut host).unwrap();
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();

        // math.random() in [0, 1)
        let r0_str = output
            .iter()
            .find(|r| r.message.starts_with("r0"))
            .unwrap()
            .message
            .split_whitespace()
            .last()
            .unwrap();
        let r0: f64 = r0_str.parse().unwrap();
        assert!(r0 >= 0.0 && r0 < 1.0);

        // math.random(1, 1) -> 1
        assert!(output.iter().any(|r| r.message == "r1 1"));

        // math.random(1, 3) -> 1, 2, or 3
        let r3_str = output
            .iter()
            .find(|r| r.message.starts_with("r3"))
            .unwrap()
            .message
            .split_whitespace()
            .last()
            .unwrap();
        let r3: f64 = r3_str.parse().unwrap();
        assert!(r3 == 1.0 || r3 == 2.0 || r3 == 3.0);
    }

    #[test]
    fn test_attribute_random_integration() {
        let source = r#"
const tests = find("Test")
const winner_idx = math.random(0, tests.len() - 1)
const winner_cell = tests[winner_idx]

for test in tests {
    test.attributes["winner"] = false
}

winner_cell.attributes["winner"] = true
debug.log("winner_id", winner_cell.id)
"#;
        let mut th = test_host();
        let c0 = WorldCoord::new(0, 0, 0);
        let c1 = WorldCoord::new(1, 1, 1);
        let c2 = WorldCoord::new(2, 2, 2);
        let id0 = add_authored_entity(&mut th.world, c0, CellType::Block, "Test");
        let id1 = add_authored_entity(&mut th.world, c1, CellType::Block, "Test");
        let id2 = add_authored_entity(&mut th.world, c2, CellType::Block, "Test");

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(program, vec![], &mut host).unwrap();
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        let winner_id_str = output
            .iter()
            .find(|r| r.message.starts_with("winner_id"))
            .unwrap()
            .message
            .split_whitespace()
            .last()
            .unwrap();
        let winner_id: u64 = winner_id_str.parse::<f64>().unwrap() as u64;

        let ids = [id0, id1, id2];
        let mut true_count = 0;
        for &id in &ids {
            let coord = th.world.resolve_cell_id(id).unwrap();
            let effective_attr = th.world.get_effective_attribute(coord, "winner").unwrap();
            use crate::world::cell::AttributeValue;

            // Verify effective behavior
            if id == winner_id {
                assert_eq!(effective_attr, AttributeValue::Bool(true));
                true_count += 1;
            } else {
                assert_eq!(effective_attr, AttributeValue::Bool(false));
            }

            // Verify AUTHORED state remains EMPTY (the key was only added at runtime)
            let cell = th.world.get(coord).unwrap();
            assert!(
                !cell.attributes.contains_key("winner"),
                "Authored attributes must not be modified by runtime writes"
            );
        }
        assert_eq!(true_count, 1);
    }

    #[test]
    fn test_runtime_cell_creation_basic() {
        let source = r#"
const block = cell.new("Block")
block.position = [4, 5, 6]
block.color = [1, 0, 0]
block.visible = true
block.solid = true
block.anchored = true
block.attributes["owner"] = "Alpha"
block.name = "RuntimeBlock"
debug.log("created_id", block.id)
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(program, vec![], &mut host).unwrap();
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        let id_str = output
            .iter()
            .find(|r| r.message.starts_with("created_id"))
            .unwrap()
            .message
            .split_whitespace()
            .last()
            .unwrap();
        let runtime_id: u64 = id_str.parse::<f64>().unwrap() as u64;

        // Verify in World
        assert!(th.world.runtime_cells.contains_key(&runtime_id));
        let coord = WorldCoord::new(4, 5, 6);
        assert_eq!(th.world.runtime_id_to_coord.get(&runtime_id), Some(&coord));

        assert_eq!(
            th.world.get_effective_color(coord),
            Vec3::new(1.0, 0.0, 0.0)
        );

        let cell = th.world.runtime_cells.get(&runtime_id).unwrap();
        assert_eq!(cell.entity_identity, Some("RuntimeBlock".to_string()));

        assert_eq!(
            th.world.get_effective_attribute(coord, "owner"),
            Some(crate::world::cell::AttributeValue::String(
                "Alpha".to_string()
            ))
        );
        assert!(th.world.is_cell_visible(coord));
        assert!(th.world.is_cell_solid(coord));
        assert!(th.world.is_cell_anchored(coord));

        // Verify authored world is EMPTY
        assert!(th.world.cells.is_empty());
    }

    #[test]
    fn test_runtime_cell_types() {
        let cases = [
            ("Block", crate::scripting::value::HandleKind::Cell),
            ("FxBlock", crate::scripting::value::HandleKind::Cell),
            ("Player", crate::scripting::value::HandleKind::Cell),
            ("NPC", crate::scripting::value::HandleKind::Cell),
            ("Light", crate::scripting::value::HandleKind::Light),
            ("SpawnPoint", crate::scripting::value::HandleKind::Cell),
        ];

        for (type_name, _) in cases {
            let mut th = test_host();
            let mut host = HostContext {
                delta_time: 0.0,
                engine: &mut th,
            };

            let source = format!(
                "const c = cell.new(\"{}\")\ndebug.log(\"kind\", c.id)",
                type_name
            );
            let tokens = Lexer::new(&source).tokenize().unwrap();
            let program = Parser::new(tokens).parse().unwrap();
            let mut scene = ScriptScene::new(program, vec![], &mut host).unwrap();
            scene.start(&mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();

            let id_str = scene
                .output()
                .iter()
                .find(|r| r.message.starts_with("kind"))
                .unwrap()
                .message
                .split_whitespace()
                .last()
                .unwrap();
            let id: u64 = id_str.parse::<f64>().unwrap() as u64;
            let cell = th.world.runtime_cells.get(&id).expect(type_name);
            assert_eq!(format!("{:?}", cell.cell_type), type_name);
        }

        // Empty should fail
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let source = "cell.new(\"Empty\")";
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(program, vec![], &mut host).unwrap();
        scene.start(&mut host).unwrap();
        let res = scene.update(0.0, &mut host);
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("Cannot create Empty cell"));
    }

    #[test]
    fn test_runtime_reset_creation() {
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let source = "const c = cell.new(\"Block\")\nc.position = [1, 2, 3]";
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(program, vec![], &mut host).unwrap();
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(!th.world.runtime_cells.is_empty());

        th.world.clear_runtime_state();
        assert!(th.world.runtime_cells.is_empty());
        assert!(th.world.runtime_id_to_coord.is_empty());
        assert!(th.world.coord_to_runtime_id.is_empty());
    }

    #[test]
    fn test_runtime_cell_deletion_authored() {
        use crate::world::cell::AttributeValue;
        let source = r#"
const green = find("Green")[0]
cell.delete(green)
debug.log("deleted")
"#;
        let mut th = test_host();
        let coord = WorldCoord::new(-5, 18, 1);
        let cell_id = add_authored_entity(&mut th.world, coord, CellType::Block, "Green");

        if let Some(cell) = th.world.get_mut(coord) {
            cell.attributes
                .insert("test".to_string(), AttributeValue::Bool(true));
        }

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(program, vec![], &mut host).unwrap();
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(scene.output().iter().any(|r| r.message == "deleted"));

        // Verify effective world: gone
        assert!(th.world.resolve_cell_id(cell_id).is_none());
        assert!(th.world.get_effective_cell(coord).is_none());
        assert!(!th.world.active_effective_blocks().contains(&coord));

        // Verify authored world: still there
        assert!(th.world.cells.contains_key(&coord));
        let auth_cell = th.world.get(coord).unwrap();
        assert_eq!(auth_cell.entity_identity, Some("Green".to_string()));
        assert_eq!(
            auth_cell.attributes.get("test"),
            Some(&AttributeValue::Bool(true))
        );

        // Stop Play (clear runtime state)
        th.world.clear_runtime_state();

        // Verify effective world: restored
        assert_eq!(th.world.resolve_cell_id(cell_id), Some(coord));
        assert!(th.world.get_effective_cell(coord).is_some());
    }

    #[test]
    fn test_runtime_cell_deletion_runtime_created() {
        let source = r#"
const block = cell.new("Block")
debug.log("created_id", block.id)
cell.delete(block)
debug.log("deleted")
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(program, vec![], &mut host).unwrap();
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        let id_str = output
            .iter()
            .find(|r| r.message.starts_with("created_id"))
            .unwrap()
            .message
            .split_whitespace()
            .last()
            .unwrap();
        let runtime_id: u64 = id_str.parse::<f64>().unwrap() as u64;

        assert!(output.iter().any(|r| r.message == "deleted"));

        // Verify gone from everywhere
        assert!(!th.world.runtime_cells.contains_key(&runtime_id));
        assert!(th.world.resolve_cell_id(runtime_id).is_none());
    }

    #[test]
    fn test_runtime_cell_deletion_repeated() {
        let source = r#"
const block = cell.new("Block")
cell.delete(block)
cell.delete(block)
debug.log("ok")
"#;
        let mut th = test_host();
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let mut scene = ScriptScene::new(program, vec![], &mut host).unwrap();
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(scene.output().iter().any(|r| r.message == "ok"));
    }

    #[test]
    fn test_cell_contact_event_authored() {
        let source = r#"
entity Spike {
    fn on_touch(c) {
        debug.log("Touched Spike:", c.id)
    }
}
"#;
        let mut th = test_host();
        let coord = WorldCoord::new(0, 0, 0);
        let cell_id = add_authored_entity(&mut th.world, coord, CellType::Block, "Spike");

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.entities[0].associated_cell_id = Some(cell_id);

        scene.start(&mut host).unwrap();

        let mut contacted = HashSet::new();
        contacted.insert(cell_id);

        scene.on_player_contact(&contacted, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message == format!("Touched Spike: {}", cell_id))
        );
    }

    #[test]
    fn test_cell_contact_event_runtime_created_global() {
        let source = r#"
on on_touch(c) {
    debug.log("Touched Cell:", c.id)
}
"#;
        let mut th = test_host();
        let cell_id = th.world.create_runtime_cell(CellType::Block);

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();

        // Register event handler correctly (create_scene doesn't add global handlers from program)
        // Actually ScriptRuntime::new does it.
        // Wait, ScriptScene::new calls ScriptRuntime::new(program).
        // ScriptRuntime::new collects Declaration::Event.

        let mut contacted = HashSet::new();
        contacted.insert(cell_id);

        scene.on_player_contact(&contacted, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message == format!("Touched Cell: {}", cell_id))
        );
    }

    #[test]
    fn test_cell_contact_event_script_deleted_during_callback() {
        let source = r#"
entity Trap {
    fn on_touch(c) {
        debug.log("Trap touched")
        cell.delete(c)
    }
}
"#;
        let mut th = test_host();
        let coord = WorldCoord::new(0, 0, 0);
        let cell_id = add_authored_entity(&mut th.world, coord, CellType::Block, "Trap");

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.entities[0].associated_cell_id = Some(cell_id);
        scene.start(&mut host).unwrap();

        let mut contacted = HashSet::new();
        contacted.insert(cell_id);

        scene.on_player_contact(&contacted, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(scene.output().iter().any(|r| r.message == "Trap touched"));
        assert!(
            th.world.resolve_cell_id(cell_id).is_none(),
            "Cell should be deleted"
        );
    }

    #[test]
    fn test_cell_contact_event_no_on_touch() {
        let source = r#"
entity Silent {
    fn on_ready() { debug.log("ready") }
}
"#;
        let mut th = test_host();
        let cell_id = add_authored_entity(
            &mut th.world,
            WorldCoord::new(0, 0, 0),
            CellType::Block,
            "Silent",
        );

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.entities[0].associated_cell_id = Some(cell_id);
        scene.start(&mut host).unwrap();

        let mut contacted = HashSet::new();
        contacted.insert(cell_id);

        // Should not error or crash
        assert!(scene.on_player_contact(&contacted, &mut host).is_ok());
        scene.update(0.0, &mut host).unwrap();
    }

    #[test]
    fn test_dynamic_event_handler_assignment() {
        let mut th = test_host();
        let cell_id = th.world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);

        let source = format!(
            r#"
const door = cell.get({})
door.on_touch = fn(c) {{
    debug.log("Dynamic Touch:", c.id)
    c.visible = false
}}
"#,
            cell_id
        );

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(&source, &mut host);
        scene.start(&mut host).unwrap();

        // 1. Initial top-level runs and assigns door.on_touch
        scene.update(0.0, &mut host).unwrap();

        // 2. Simulate contact
        let mut contacted = HashSet::new();
        contacted.insert(cell_id);
        scene.on_player_contact(&contacted, &mut host).unwrap();

        // 3. Update to run the spawned fiber
        scene.update(0.0, &mut host).unwrap();

        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message == format!("Dynamic Touch: {}", cell_id))
        );
        assert_eq!(th.world.is_cell_visible(WorldCoord::new(0, 0, 0)), false);
    }

    #[test]
    fn test_dynamic_method_call_with_self() {
        let mut th = test_host();
        let cell_id = th.world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);

        let source = format!(
            r#"
const door = cell.get({})
door.open = fn() {{
    debug.log("Opening door:", self.id)
    self.visible = false
}}
door:open()
"#,
            cell_id
        );

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(&source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message == format!("Opening door: {}", cell_id))
        );
        assert_eq!(th.world.is_cell_visible(WorldCoord::new(0, 0, 0)), false);
    }

    #[test]
    fn test_dynamic_touch_handler_captures_lexical_scope() {
        let source = r#"
const doors = find("Door")

for door in doors {
    door.on_touch = fn(c) {
        debug.log("Door touched:", c.id)

        for target in doors {
            target.visible = false
            target.solid = false
        }

        wait(1)

        for target in doors {
            target.visible = true
            target.solid = true
        }
    }
}
"#;

        let mut th = test_host();
        let first = th.world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);
        th.world
            .get_mut(WorldCoord::new(0, 0, 0))
            .unwrap()
            .entity_identity = Some("Door".to_string());
        let _second = th.world.set_cell(WorldCoord::new(1, 0, 0), CellType::Block);
        th.world
            .get_mut(WorldCoord::new(1, 0, 0))
            .unwrap()
            .entity_identity = Some("Door".to_string());

        let first_coord = WorldCoord::new(0, 0, 0);
        let second_coord = WorldCoord::new(1, 0, 0);

        let mut scene = {
            let mut host = HostContext {
                delta_time: 0.0,
                engine: &mut th,
            };

            let mut scene = create_scene(source, &mut host);
            scene.start(&mut host).unwrap();

            // Run the top-level once so both dynamic handlers are installed.
            scene.update(0.0, &mut host).unwrap();

            let mut contacted = HashSet::new();
            contacted.insert(first);
            scene.on_player_contact(&contacted, &mut host).unwrap();

            // The callback should see the captured `doors` basket and mutate both
            // cells before yielding.
            scene.update(0.0, &mut host).unwrap();
            scene
        };

        assert!(!th.world.is_cell_visible(first_coord));
        assert!(!th.world.is_cell_visible(second_coord));
        assert!(!th.world.is_cell_solid(first_coord));
        assert!(!th.world.is_cell_solid(second_coord));
        assert!(
            scene
                .output()
                .iter()
                .any(|record| record.message == format!("Door touched: {}", first))
        );

        // Resume the same closure after its wait.
        {
            let mut host = HostContext {
                delta_time: 0.0,
                engine: &mut th,
            };
            scene.update(1.0, &mut host).unwrap();
        }

        assert!(th.world.is_cell_visible(first_coord));
        assert!(th.world.is_cell_visible(second_coord));
        assert!(th.world.is_cell_solid(first_coord));
        assert!(th.world.is_cell_solid(second_coord));
    }

    #[test]
    fn test_cell_overlap_events() {
        let source = r#"
entity Trigger {
    fn on_overlap(overlapping, c) {
        if overlapping {
            debug.log("Entered:", c.id)
        } else {
            debug.log("Exited:", c.id)
        }
    }
}
"#;
        let mut th = test_host();
        let cell_id = th.world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);

        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut th,
        };
        let mut scene = create_scene(source, &mut host);
        scene.entities[0].associated_cell_id = Some(cell_id);
        scene.start(&mut host).unwrap();

        // 1. Start Overlap
        scene.on_player_overlap(cell_id, true, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message == format!("Entered: {}", cell_id))
        );

        // 2. End Overlap
        scene.on_player_overlap(cell_id, false, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        assert!(
            scene
                .output()
                .iter()
                .any(|r| r.message == format!("Exited: {}", cell_id))
        );
    }
}
