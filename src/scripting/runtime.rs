use std::collections::HashMap;

use super::ast::Program;
use super::execution::{
    FiberResult,
    ScriptScheduler,
    ScriptTaskId,
};
use super::interpreter::{
    Interpreter,
    ScriptFiber,
    ScriptInstance,
};
use super::value::Value;
use super::api::HostContext;

/// Runtime owner for all live AeoScript execution.
///
/// The runtime connects the interpreter's persistent fibers to the cooperative
/// scheduler. It remains single-threaded and never blocks the engine thread.
#[derive(Debug)]
pub struct ScriptRuntime {
    interpreter: Interpreter,
    scheduler: ScriptScheduler,
    fibers: HashMap<ScriptTaskId, ScriptFiber>,
}

impl ScriptRuntime {
    pub fn new(program: Program) -> Self {
        Self {
            interpreter: Interpreter::new(program),
            scheduler: ScriptScheduler::new(),
            fibers: HashMap::new(),
        }
    }

    pub fn with_interpreter(interpreter: Interpreter) -> Self {
        Self {
            interpreter,
            scheduler: ScriptScheduler::new(),
            fibers: HashMap::new(),
        }
    }

    pub fn interpreter(&self) -> &Interpreter {
        &self.interpreter
    }

    pub fn interpreter_mut(&mut self) -> &mut Interpreter {
        &mut self.interpreter
    }

    pub fn scheduler(&self) -> &ScriptScheduler {
        &self.scheduler
    }

    pub fn scheduler_mut(&mut self) -> &mut ScriptScheduler {
        &mut self.scheduler
    }

    pub fn task_count(&self) -> usize {
        self.fibers.len()
    }

    pub fn fiber(&self, id: ScriptTaskId) -> Option<&ScriptFiber> {
        self.fibers.get(&id)
    }

    pub fn fiber_mut(
        &mut self,
        id: ScriptTaskId,
    ) -> Option<&mut ScriptFiber> {
        self.fibers.get_mut(&id)
    }

    /// Starts a new persistent script fiber and schedules it immediately.
    pub fn spawn(
        &mut self,
        instance: ScriptInstance,
        function_name: &str,
        arguments: Vec<Value>,
    ) -> Result<ScriptTaskId, String> {
        let fiber = self
            .interpreter
            .start_fiber(
                instance,
                function_name,
                arguments,
            )?;

        let task_id = self.scheduler.spawn();

        self.fibers.insert(task_id, fiber);

        Ok(task_id)
    }

    /// Advances runtime time and executes the fibers that were ready for this
    /// tick.
    ///
    /// Each task receives at most one execution slice. This prevents a task
    /// returning `Continue` from consuming the whole tick by immediately
    /// re-entering itself.
    pub fn tick(
        &mut self,
        new_time: f64,
        host: &mut HostContext,
    ) -> Result<Vec<(ScriptTaskId, FiberResult)>, String> {
        self.scheduler.tick(new_time)?;

        let ready_count = self.scheduler.ready_count();

        let mut results = Vec::with_capacity(ready_count);

        for _ in 0..ready_count {
            let task_id = match self.scheduler.pop_ready() {
                Some(task_id) => task_id,

                None => break,
            };

            self.scheduler.begin_running(task_id)?;

            let result = {
                let fiber = self.fibers.get_mut(&task_id).ok_or_else(|| {
                    format!(
                        "AeoScript runtime has no fiber for task {}.",
                        task_id.value()
                    )
                })?;

                self.interpreter.resume_fiber(fiber, host)
            };

            self.scheduler
                .apply_result(task_id, result.clone())?;

            results.push((task_id, result));
        }

        Ok(results)
    }

    pub fn cancel(
        &mut self,
        task_id: ScriptTaskId,
    ) -> Result<(), String> {
        self.scheduler.cancel(task_id)
    }

    /// Removes a completed or failed fiber from runtime storage.
    ///
    /// The scheduler state is intentionally left untouched; callers should
    /// remove fibers only after observing the terminal state.
    pub fn remove_fiber(
        &mut self,
        task_id: ScriptTaskId,
    ) -> Option<ScriptFiber> {
        self.fibers.remove(&task_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::scripting::execution::{
        ScriptTaskState,
        YieldReason,
    };
    use crate::scripting::lexer::Lexer;
    use crate::scripting::parser::Parser;
    use crate::engine::entity::EntityManager;

    fn runtime(source: &str) -> ScriptRuntime {
        let tokens = Lexer::new(source)
            .tokenize()
            .expect("lexer should succeed");

        let program = Parser::new(tokens)
            .parse()
            .expect("parser should succeed");

        ScriptRuntime::new(program)
    }

    fn test_host() -> EntityManager {
        EntityManager::new()
    }

    #[test]
    fn runtime_spawns_ready_fiber() {
        let source = r#"
entity Test {
    value: number = 0

    fn update(dt: number) {
        value = 1
    }
}
"#;

        let mut runtime = runtime(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let task_id = runtime
            .spawn(
                instance,
                "update",
                vec![Value::Number(1.0)],
            )
            .expect("fiber should spawn");

        assert_eq!(runtime.task_count(), 1);

        assert_eq!(
            runtime.scheduler().state(task_id),
            Some(ScriptTaskState::Ready)
        );
    }

    #[test]
    fn runtime_connects_wait_to_scheduler() {
        let source = r#"
entity Test {
    value: number = 0

    fn update(dt: number) {
        value = 1
        wait(1)
        value = 2
    }
}
"#;

        let mut runtime = runtime(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let task_id = runtime
            .spawn(
                instance,
                "update",
                vec![Value::Number(1.0)],
            )
            .expect("fiber should spawn");

        let results = runtime
            .tick(0.0, &mut host)
            .expect("first tick should succeed");

        assert_eq!(
            results,
            vec![(
                task_id,
                FiberResult::Yield(
                    YieldReason::WaitSeconds(1.0)
                )
            )]
        );

        assert_eq!(
            runtime.scheduler().state(task_id),
            Some(ScriptTaskState::Waiting {
                wake_at: 1.0
            })
        );

        assert_eq!(
            runtime
                .fiber(task_id)
                .expect("fiber should exist")
                .instance()
                .get_field("value"),
            Some(&Value::Number(1.0))
        );

        let results = runtime
            .tick(0.5, &mut host)
            .expect("second tick should succeed");

        assert!(results.is_empty());

        assert_eq!(
            runtime
                .fiber(task_id)
                .expect("fiber should exist")
                .instance()
                .get_field("value"),
            Some(&Value::Number(1.0))
        );

        let results = runtime
            .tick(1.0, &mut host)
            .expect("wake tick should succeed");

        assert_eq!(
            results,
            vec![(task_id, FiberResult::Complete)]
        );

        assert_eq!(
            runtime.scheduler().state(task_id),
            Some(ScriptTaskState::Completed)
        );

        assert_eq!(
            runtime
                .fiber(task_id)
                .expect("fiber should exist")
                .instance()
                .get_field("value"),
            Some(&Value::Number(2.0))
        );
    }

    #[test]
    fn runtime_does_not_immediately_rerun_continuing_task() {
        let source = r#"
entity Test {
    value: number = 0

    fn update(dt: number) {
        value += 1
    }
}
"#;

        let mut runtime = runtime(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let task_id = runtime
            .spawn(
                instance,
                "update",
                vec![Value::Number(1.0)],
            )
            .expect("fiber should spawn");

        let results = runtime
            .tick(0.0, &mut host)
            .expect("tick should succeed");

        assert_eq!(
            results,
            vec![(task_id, FiberResult::Complete)]
        );
    }

    #[test]
    fn runtime_supports_multiple_waiting_scripts() {
        let source = r#"
entity Test {
    value: number = 0

    fn update(dt: number) {
        value = 1
        wait(1)
        value = 2
    }
}
"#;

        let mut runtime = runtime(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let first_instance = runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .expect("first entity should instantiate");

        let second_instance = runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .expect("second entity should instantiate");

        let first = runtime
            .spawn(
                first_instance,
                "update",
                vec![Value::Number(1.0)],
            )
            .expect("first fiber should spawn");

        let second = runtime
            .spawn(
                second_instance,
                "update",
                vec![Value::Number(1.0)],
            )
            .expect("second fiber should spawn");

        let results = runtime
            .tick(0.0, &mut host)
            .expect("tick should succeed");

        assert_eq!(results.len(), 2);

        assert_eq!(
            runtime.scheduler().state(first),
            Some(ScriptTaskState::Waiting {
                wake_at: 1.0
            })
        );

        assert_eq!(
            runtime.scheduler().state(second),
            Some(ScriptTaskState::Waiting {
                wake_at: 1.0
            })
        );

        let results = runtime
            .tick(1.0, &mut host)
            .expect("wake tick should succeed");

        assert_eq!(results.len(), 2);

        assert!(
            results
                .iter()
                .all(|(_, result)| *result
                    == FiberResult::Complete)
        );
    }

    #[test]
    fn runtime_rejects_zero_wait() {
        let source = r#"
entity Test {
    fn update(dt: number) {
        wait(0)
    }
}
"#;

        let mut runtime = runtime(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let task_id = runtime
            .spawn(
                instance,
                "update",
                vec![Value::Number(1.0)],
            )
            .expect("fiber should spawn");

        let results = runtime
            .tick(0.0, &mut host)
            .expect("runtime tick should itself succeed");

        assert_eq!(results.len(), 1);

        assert!(matches!(
            results[0],
            (
                id,
                FiberResult::Failed(ref message)
            ) if id == task_id
                && message.contains(
                    "greater than zero"
                )
        ));

        assert_eq!(
            runtime.scheduler().state(task_id),
            Some(ScriptTaskState::Failed)
        );
    }

    #[test]
    fn runtime_accepts_small_positive_wait() {
        let source = r#"
entity Test {
    value: number = 0

    fn update(dt: number) {
        value = 1
        wait(0.0001)
        value = 2
    }
}
"#;

        let mut runtime = runtime(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let task_id = runtime
            .spawn(
                instance,
                "update",
                vec![Value::Number(1.0)],
            )
            .expect("fiber should spawn");

        let results = runtime
            .tick(0.0, &mut host)
            .expect("runtime tick should succeed");

        assert_eq!(
            results,
            vec![(
                task_id,
                FiberResult::Yield(
                    YieldReason::WaitSeconds(
                        0.0001
                    )
                )
            )]
        );

        assert_eq!(
            runtime.scheduler().state(task_id),
            Some(ScriptTaskState::Waiting {
                wake_at: 0.0001
            })
        );

        let results = runtime
            .tick(0.0001, &mut host)
            .expect("wake tick should succeed");

        assert_eq!(
            results,
            vec![(task_id, FiberResult::Complete)]
        );

        assert_eq!(
            runtime
                .fiber(task_id)
                .expect("fiber should exist")
                .instance()
                .get_field("value"),
            Some(&Value::Number(2.0))
        );
    }

    #[test]
    fn runtime_execution_smoke_test() {
        let source = r#"
entity Test {
    value: number = 0
    fn main() {
        value = 100
    }
}
"#;
        let mut runtime = runtime(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();

        let task_id = runtime.spawn(instance, "main", vec![]).unwrap();
        let results = runtime.tick(0.0, &mut host).unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, task_id);
        assert_eq!(results[0].1, FiberResult::Complete);

        assert_eq!(
            runtime.fiber(task_id).unwrap().instance().get_field("value"),
            Some(&Value::Number(100.0))
        );
    }

    #[test]
    fn runtime_failure_is_observable() {
        let source = r#"
entity Test {
    fn fail() {
        const x = 1 / 0
    }
}
"#;
        let mut runtime = runtime(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = runtime
            .interpreter_mut()
            .instantiate_entity("Test", 1, &mut host)
            .unwrap();

        let task_id = runtime.spawn(instance, "fail", vec![]).unwrap();
        let results = runtime.tick(0.0, &mut host).unwrap();

        assert_eq!(results.len(), 1);
        match &results[0].1 {
            FiberResult::Failed(msg) => {
                assert!(msg.contains("division by zero"));
            }
            _ => panic!("Expected FiberResult::Failed"),
        }

        assert_eq!(
            runtime.scheduler().state(task_id),
            Some(ScriptTaskState::Failed)
        );
    }

    #[test]
    fn runtime_multiple_entities_independent() {
        let source = r#"
entity Test {
    value: number = 0
    fn set(v: number) {
        value = v
    }
}
"#;
        let mut runtime = runtime(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let inst1 = runtime.interpreter_mut().instantiate_entity("Test", 1, &mut host).unwrap();
        let inst2 = runtime.interpreter_mut().instantiate_entity("Test", 2, &mut host).unwrap();

        let t1 = runtime.spawn(inst1, "set", vec![Value::Number(10.0)]).unwrap();
        let t2 = runtime.spawn(inst2, "set", vec![Value::Number(20.0)]).unwrap();

        runtime.tick(0.0, &mut host).unwrap();

        assert_eq!(
            runtime.fiber(t1).unwrap().instance().get_field("value"),
            Some(&Value::Number(10.0))
        );
        assert_eq!(
            runtime.fiber(t2).unwrap().instance().get_field("value"),
            Some(&Value::Number(20.0))
        );
    }
}
