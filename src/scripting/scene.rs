use super::runtime::ScriptRuntime;
use super::interpreter::ScriptInstance;
use super::execution::{ScriptTaskId, FiberResult};
use super::value::Value;
use super::ast::{Program, Declaration, EntityMember};
use super::api::HostContext;

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
pub struct ScriptScene {
    runtime: ScriptRuntime,
    entities: Vec<ScriptEntity>,
    current_time: f64,
}

impl ScriptScene {
    pub fn new(program: Program, host: &HostContext) -> Self {
        let mut runtime = ScriptRuntime::new(program);
        let mut entities = Vec::new();

        let entity_names: Vec<String> = runtime
            .interpreter()
            .program()
            .declarations
            .iter()
            .filter_map(|decl| {
                if let Declaration::Entity(entity) = decl {
                    Some(entity.name.clone())
                } else {
                    None
                }
            })
            .collect();

        for name in entity_names {
            if let Ok(instance) = runtime.interpreter_mut().instantiate_entity(&name, host) {
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
    pub fn start(&mut self, host: &HostContext) -> Result<(), String> {
        for entity in &mut self.entities {
            Self::transition_entity(&mut self.runtime, entity, host)?;
        }
        Ok(())
    }

    /// Advances the scene time and executes script fibers.
    pub fn update(&mut self, dt: f32, host: &HostContext) -> Result<(), String> {
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

    /// Stops all script execution and invokes on_destroy where available.
    pub fn stop(&mut self, host: &HostContext) {
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

    fn transition_entity(runtime: &mut ScriptRuntime, entity: &mut ScriptEntity, host: &HostContext) -> Result<(), String> {
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
    use crate::engine::entity::EntityManager;

    fn create_scene(source: &str, host: &HostContext) -> ScriptScene {
        let tokens = Lexer::new(source).tokenize().unwrap();
        let program = Parser::new(tokens).parse().unwrap();
        ScriptScene::new(program, host)
    }

    fn test_host() -> EntityManager {
        EntityManager::new()
    }

    #[test]
    fn scene_instantiates_entities() {
        let source = "entity Test {}";
        let em = test_host();
        let host = HostContext { delta_time: 1.0, entity_manager: &em };
        let scene = create_scene(source, &host);
        assert_eq!(scene.entities.len(), 1);
    }

    #[test]
    fn scene_lifecycle_order() {
        let source = r#"
entity Test {
    val: number = 0
    fn on_spawn() { val = 1 }
    fn on_ready() { val = 2 }
    fn update(dt: number) { val = 3 }
}
"#;
        let em = test_host();
        let host = HostContext { delta_time: 1.0, entity_manager: &em };
        let mut scene = create_scene(source, &host);
        scene.start(&host).unwrap();

        assert!(matches!(scene.entities[0].state, EntityState::Spawning(Some(_))));

        scene.update(0.0, &host).unwrap();
        assert!(matches!(scene.entities[0].state, EntityState::Readying(Some(_))));

        scene.update(0.0, &host).unwrap();
        assert!(matches!(scene.entities[0].state, EntityState::Active { update_task: Some(_) }));

        scene.update(0.0, &host).unwrap();
    }

    #[test]
    fn update_cooperative_wait() {
        let source = r#"
entity Test {
    val: number = 0
    fn update(dt: number) {
        val += 1
        wait(1)
        val += 1
    }
}
"#;
        let em = test_host();
        let host = HostContext { delta_time: 1.0, entity_manager: &em };
        let mut scene = create_scene(source, &host);
        scene.start(&host).unwrap();

        scene.update(0.0, &host).unwrap();
        let task_id = match scene.entities[0].state {
            EntityState::Active { update_task: Some(tid) } => tid,
            _ => panic!("Expected update task"),
        };

        scene.update(0.1, &host).unwrap();
        // It should yield now.
        let state = scene.runtime.scheduler().state(task_id).unwrap();
        assert!(matches!(state, crate::scripting::execution::ScriptTaskState::Waiting { .. }));
        assert_eq!(scene.entities[0].active_task(), Some(task_id));

        scene.update(0.1, &host).unwrap();
        assert_eq!(scene.entities[0].active_task(), Some(task_id));

        scene.update(1.0, &host).unwrap();
        assert!(scene.entities[0].active_task().is_some());
        assert_ne!(scene.entities[0].active_task(), Some(task_id));
    }

    #[test]
    fn stop_cleans_up() {
        let source = r#"
entity Test {
    fn on_destroy() { debug.log("destroyed") }
}
"#;
        let em = test_host();
        let host = HostContext { delta_time: 1.0, entity_manager: &em };
        let mut scene = create_scene(source, &host);
        scene.start(&host).unwrap();
        scene.stop(&host);
        assert_eq!(scene.entities.len(), 0);
        assert_eq!(scene.runtime.task_count(), 0);
    }

    #[test]
    fn wait_0_fails() {
        let source = r#"
entity Test {
    fn update(dt: number) { wait(0) }
}
"#;
        let em = test_host();
        let host = HostContext { delta_time: 1.0, entity_manager: &em };
        let mut scene = create_scene(source, &host);
        scene.start(&host).unwrap();
        scene.update(0.0, &host).unwrap();
        let result = scene.update(0.0, &host);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("greater than zero"));
    }

    #[test]
    fn wait_small_positive_succeeds() {
        let source = r#"
entity Test {
    fn update(dt: number) { wait(0.0001) }
}
"#;
        let em = test_host();
        let host = HostContext { delta_time: 1.0, entity_manager: &em };
        let mut scene = create_scene(source, &host);
        scene.start(&host).unwrap();
        scene.update(0.0, &host).unwrap();
        let result = scene.update(0.0, &host);
        assert!(result.is_ok());
    }
}
