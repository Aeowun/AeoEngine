use super::runtime::ScriptRuntime;
use super::interpreter::ScriptInstance;
use super::execution::{ScriptTaskId, FiberResult};
use super::value::Value;
use super::ast::{Program, Declaration, EntityMember};
use super::api::HostContext;
use super::binding::ScriptBinding;
use super::lexer::Lexer;
use super::parser::Parser;
use super::source::SourceSpan;
use crate::engine::entity::{EntityManager, EntityId};
use std::path::Path;
use std::collections::HashMap;

/// A single scripted entity's lifecycle state.
#[derive(Debug)]
pub struct ScriptEntity {
    instance: ScriptInstance,
    state: EntityState,
    stopped: bool,
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
    fn new(instance: ScriptInstance) -> Self {
        Self {
            instance,
            state: EntityState::Initial,
            stopped: false,
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
    /// Loads and parses scripts based on authored bindings and prepares the scene.
    pub fn load_from_bindings(
        project_path: &Path,
        bindings: &[ScriptBinding],
        entity_manager: &mut EntityManager,
        delta_time: f64,
    ) -> Result<Self, String> {
        let mut loaded_scripts = HashMap::new();
        let mut entities_to_spawn = Vec::new();

        for binding in bindings {
            if !loaded_scripts.contains_key(&binding.script_path) {
                let script_path = project_path.join(&binding.script_path);
                let source = std::fs::read_to_string(&script_path).map_err(|e| {
                    format!("Failed to read script {:?}: {}", script_path, e)
                })?;

                let tokens = Lexer::new(&source).tokenize().map_err(|e| {
                    format!("Lexer error in {:?}: {:?}", script_path, e)
                })?;

                let program = Parser::new(tokens).parse().map_err(|e| {
                    format!("Parser error in {:?}: {:?}", script_path, e)
                })?;

                loaded_scripts.insert(binding.script_path.clone(), program);
            }

            let program = loaded_scripts.get(&binding.script_path).unwrap();
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

            entities_to_spawn.push(binding.target_identity.clone());
        }

        let mut all_declarations = Vec::new();
        for program in loaded_scripts.into_values() {
            all_declarations.extend(program.declarations);
        }

        let combined_program = Program {
            span: SourceSpan::new(0, 0),
            declarations: all_declarations,
        };

        let mut spawn_params = Vec::new();
        for name in &entities_to_spawn {
            let id = entity_manager.create_entity(name);
            spawn_params.push((name.clone(), id.0));
        }

        let mut host = HostContext {
            delta_time,
            engine: entity_manager,
        };

        Ok(Self::new(combined_program, spawn_params, &mut host))
    }

    pub fn new(program: Program, entities_to_spawn: Vec<(String, u64)>, host: &mut HostContext) -> Self {
        let mut runtime = ScriptRuntime::new(program);
        let mut entities = Vec::new();

        for (name, id) in entities_to_spawn {
            if let Ok(instance) = runtime.interpreter_mut().instantiate_entity(&name, id, host) {
                entities.push(ScriptEntity::new(instance));
            }
        }

        Self {
            runtime,
            entities,
            current_time: 0.0,
        }
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
                    break;
                }
            }
        }

        // Spawn update tasks for active entities that don't currently have one running.
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

    /// Returns the combined debug output from all scripts in the scene.
    pub fn output(&self) -> &[String] {
        self.runtime.interpreter().output()
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
                let _ = self.runtime.interpreter_mut().call(&mut instance, "on_destroy", vec![], host);
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

    fn transition_entity(runtime: &mut ScriptRuntime, entity: &mut ScriptEntity, host: &mut HostContext) -> Result<(), String> {
        if entity.stopped {
            return Ok(());
        }

        loop {
            match entity.state {
                EntityState::Initial => {
                    if Self::has_function(runtime, entity, "on_spawn") {
                        let task_id =
                            runtime.spawn(entity.instance.clone(), "on_spawn", vec![])?;
                        entity.state = EntityState::Spawning(Some(task_id));
                        break;
                    } else {
                        entity.state = EntityState::Spawning(None);
                    }
                }
                EntityState::Spawning(_) => {
                    if let Some(task_id) = entity.active_task() {
                        // Already has a task running (on_spawn)
                        let _ = task_id;
                        break;
                    }
                    if Self::has_function(runtime, entity, "on_ready") {
                        let task_id =
                            runtime.spawn(entity.instance.clone(), "on_ready", vec![])?;
                        entity.state = EntityState::Readying(Some(task_id));
                        break;
                    } else {
                        entity.state = EntityState::Readying(None);
                    }
                }
                EntityState::Readying(_) => {
                    if let Some(task_id) = entity.active_task() {
                        // Already has a task running (on_ready)
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
    use crate::scripting::lexer::Lexer;
    use crate::scripting::parser::Parser;
    use crate::engine::entity::{EntityManager, EntityId};
    use crate::scripting::api::EngineHost;
    use glam::Vec3;

    struct TestHost {
        entity_manager: EntityManager,
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
    }

    fn create_scene(source: &str, host: &mut HostContext) -> ScriptScene {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        let entities_to_spawn: Vec<(String, u64)> = program
            .declarations
            .iter()
            .filter_map(|decl| {
                if let Declaration::Entity(entity) = decl {
                    let id = host.engine.entity_manager().lookup_entity(&entity.name)
                        .unwrap_or(EntityId(0));
                    Some((entity.name.clone(), id.0))
                } else {
                    None
                }
            })
            .collect();
        ScriptScene::new(program, entities_to_spawn, host)
    }

    fn test_host() -> TestHost {
        TestHost {
            entity_manager: EntityManager::new(),
        }
    }

    #[test]
    fn test_host_context_construction() {
        let mut th = test_host();
        let mut host = HostContext { delta_time: 0.1, engine: &mut th };
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

        let mut scene = {
            let mut host = HostContext { delta_time: 0.0, engine: &mut th };
            let mut scene = create_scene(source, &mut host);
            scene.start(&mut host).unwrap();
            scene.update(0.0, &mut host).unwrap(); // on_spawn spawned
            scene.update(0.0, &mut host).unwrap(); // on_spawn runs
            scene
        };

        let output = scene.output();
        assert!(output.contains(&"1".to_string()));
        assert!(output.contains(&"2".to_string()));
        assert!(output.contains(&"3".to_string()));
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

        {
            let mut host = HostContext { delta_time: 0.0, engine: &mut th };
            let mut scene = create_scene(source, &mut host);
            scene.start(&mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
        }

        assert_eq!(th.entity_manager.get_position(id), Some(Vec3::new(10.0, 20.0, 30.0)));
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

        {
            let mut host = HostContext { delta_time: 0.0, engine: &mut th };
            let mut scene = create_scene(source, &mut host);
            scene.start(&mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
        }

        assert_eq!(th.entity_manager.get_position(id), Some(Vec3::new(6.0, 6.0, 6.0)));
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

        {
            let mut host = HostContext { delta_time: 0.0, engine: &mut th };
            let mut scene = create_scene(source, &mut host);
            scene.start(&mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
            scene.update(0.0, &mut host).unwrap();
        }

        assert_eq!(th.entity_manager.get_position(id_target), Some(Vec3::new(100.0, 100.0, 100.0)));
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
            scene.update(0.0, &mut host).unwrap(); // on_spawn starts and yields at wait(1)

            scene
        };

        // Invalidate the runtime entity after the script captured its handle.
        th.entity_manager.remove_entity(id);

        {
            let mut host = HostContext {
                delta_time: 1.0,
                engine: &mut th,
            };

            // The stale handle must not panic or crash the interpreter.
            let result = scene.update(1.0, &mut host);
            assert!(result.is_ok());
        }
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
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);

        scene.start(&mut host).unwrap();
        assert_eq!(scene.output(), Vec::<String>::new());

        scene.update(0.0, &mut host).unwrap(); // on_spawn
        scene.update(0.0, &mut host).unwrap(); // on_ready
        scene.update(0.0, &mut host).unwrap(); // update

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
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.5, &mut host).unwrap(); // spawns update
        scene.update(0.0, &mut host).unwrap(); // runs update

        assert!(scene.output().contains(&"0.5".to_string()));
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
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap(); // spawns update
        let result = scene.update(0.0, &mut host); // runs update, FAILS

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("division by zero"));
    }

    #[test]
    fn scene_invalid_entity_reference() {
        let source = r#"
entity Test {
    fn update(dt: number) {
        const e = get_entity("Missing")
        if e == null {
            debug.log("not_found")
        }
    }
}
"#;
        let mut th = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap(); // spawns update
        scene.update(0.0, &mut host).unwrap(); // runs update

        assert!(scene.output().contains(&"not_found".to_string()));
    }

    #[test]
    fn scene_entity_validity() {
        let source = r#"
entity Test {
    fn update(dt: number) {
        const e = get_entity("Test")
        if e != null {
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
            let mut host = HostContext { delta_time: 1.0, engine: &mut th };
            let mut scene = create_scene(source, &mut host);
            scene.start(&mut host).unwrap();
            scene.update(0.0, &mut host).unwrap(); // spawns update
            scene.update(0.0, &mut host).unwrap(); // runs update
            scene
        };

        assert!(scene.output().contains(&"valid".to_string()));

        // Now invalidate it.
        th.entity_manager.remove_entity(id);
        scene.runtime.interpreter_mut().drain_output();

        {
            let mut host = HostContext { delta_time: 1.0, engine: &mut th };
            scene.update(0.0, &mut host).unwrap(); // runs NEXT update
        }
    }

    #[test]
    fn scene_stop_prevents_execution() {
        let source = r#"
entity Test {
    fn update(dt: number) { debug.log("update") }
}
"#;
        let mut th = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.stop(&mut host);

        scene.update(0.1, &mut host).unwrap();
        assert!(!scene.output().contains(&"update".to_string()));
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
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();

        scene.update(0.0, &mut host).unwrap(); // spawns update
        scene.update(0.0, &mut host).unwrap(); // update runs, yields at wait(1)
        assert!(scene.output().contains(&"start".to_string()));
        assert!(!scene.output().contains(&"end".to_string()));

        scene.update(0.5, &mut host).unwrap();
        assert!(!scene.output().contains(&"end".to_string()));

        scene.update(0.5, &mut host).unwrap();
        // It should wake up and complete.
        assert!(scene.output().contains(&"end".to_string()));
    }

    #[test]
    fn test_explicit_binding_instantiates_only_requested() {
        let source = r#"
entity Player {}
entity Enemy {}
"#;
        let mut th = test_host();
        let id = th.entity_manager.create_entity("Player");
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };

        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();

        // Only bind Player
        let scene = ScriptScene::new(program, vec![("Player".to_string(), id.0)], &mut host);

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
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };

        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();

        let mut scene = ScriptScene::new(program, vec![("Bound".to_string(), id.0)], &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.contains(&"bound".to_string()));
        assert!(!output.contains(&"unbound".to_string()));
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
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();

        let mut scene = ScriptScene::new(program, vec![("A".to_string(), id_a.0), ("B".to_string(), id_b.0)], &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();
        scene.update(0.0, &mut host).unwrap();

        let output = scene.output();
        assert!(output.contains(&"A".to_string()));
        assert!(output.contains(&"B".to_string()));
    }

    #[test]
    fn test_missing_entity_declaration_fails_safely_explicit() {
        let source = "entity Found {}";
        let mut th = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut th };
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();

        // Requesting "Missing" which is not in the program.
        let scene = ScriptScene::new(program, vec![("Missing".to_string(), 0)], &mut host);

        assert_eq!(scene.entities.len(), 0);
    }

    #[test]
    fn test_load_from_bindings_missing_script_fails() {
        let mut th = test_host();
        let temp_dir = std::env::temp_dir().join("aeo_test_missing");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let bindings = vec![ScriptBinding::new("Player", "nonexistent.aeo")];
        let result = ScriptScene::load_from_bindings(&temp_dir, &bindings, &mut th.entity_manager, 1.0);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to read script"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_load_from_bindings_invalid_script_fails() {
        let mut th = test_host();
        let temp_dir = std::env::temp_dir().join("aeo_test_invalid");
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let script_path = "broken.aeo";
        std::fs::write(temp_dir.join(script_path), "entity Broken { !!! }").unwrap();

        let bindings = vec![ScriptBinding::new("Broken", script_path)];
        let result = ScriptScene::load_from_bindings(&temp_dir, &bindings, &mut th.entity_manager, 1.0);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Parser error"));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn wait_0_fails() {
        let source = r#"
entity Test {
    fn update(dt: number) { wait(0) }
}
"#;
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut scene = create_scene(source, &mut host);
        scene.start(&mut host).unwrap();
        scene.update(0.0, &mut host).unwrap(); // Initial -> Active, spawns update
        let result = scene.update(0.0, &mut host); // runs update, FAILS
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than zero"));
    }
}
