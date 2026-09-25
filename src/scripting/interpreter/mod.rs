pub mod interpreter_compile;
pub mod interpreter_eval;
pub mod interpreter_execution;
pub mod interpreter_tests;

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::scripting::api::HostContext;
use crate::scripting::ast::*;
use crate::scripting::execution::FiberResult;
use crate::scripting::log::{LogRecord, LogSeverity};
use crate::scripting::source::SourceSpan;
use crate::scripting::value::{Scope, Value};

const DEFAULT_OPERATION_BUDGET: u64 = 100_000;
const DEFAULT_MAX_CALL_DEPTH: usize = 64;

pub(crate) const FRAME_PUSHED_SENTINEL: &str = "__AEO_FIBER_FRAME_PUSHED__";

/// A live instance of one AeoScript entity definition.
#[derive(Clone, Debug)]
pub struct ScriptInstance {
    pub(super) id: u64,
    pub(super) entity_name: String,
    pub script_path: Option<String>,
    pub(super) fields: BTreeMap<String, Value>,
}

impl ScriptInstance {
    pub fn id(&self) -> u64 {
        self.id
    }

    pub fn new_empty() -> Self {
        Self {
            id: 0,
            entity_name: "Global".to_string(),
            script_path: None,
            fields: BTreeMap::new(),
        }
    }

    pub fn new_entity(entity_name: &str) -> Self {
        Self {
            id: 0,
            entity_name: entity_name.to_string(),
            script_path: None,
            fields: BTreeMap::new(),
        }
    }

    pub fn with_path(mut self, path: String) -> Self {
        self.script_path = Some(path);
        self
    }

    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }

    pub fn get_field(&self, name: &str) -> Option<&Value> {
        self.fields.get(name)
    }

    pub fn set_field(&mut self, name: &str, value: Value) -> Result<(), String> {
        self.fields.insert(name.to_string(), value);
        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(super) enum ExecutionFlow {
    Continue,
    Return(Value),
}

/// One persistent state for a resumable AeoScript execution.
#[derive(Clone, Debug)]
pub(super) struct ForState {
    pub(super) name: String,
    pub(super) values: Vec<Value>,
    pub(super) next_index: usize,
}

#[derive(Clone, Debug)]
pub(super) struct CallFrame {
    pub(super) function: Arc<CompiledFunction>,
    pub(super) pc: usize,
    pub(super) scopes: Vec<Scope>,
    pub(super) for_states: Vec<ForState>,
    pub(super) function_name: String,
    pub(super) captured_scopes: Vec<Scope>,
    pub(super) expects_return_value: bool,
    pub(super) call_site_span: Option<SourceSpan>,
}

/// A cooperative AeoScript execution fiber.
#[derive(Debug)]
pub struct ScriptFiber {
    pub script_path: Option<String>,
    pub entity_name: String,
    pub(super) instance: ScriptInstance,
    pub(super) stack: Vec<CallFrame>,
    pub(super) finished: bool,
    pub(super) result: Option<Value>,
    pub return_value: Option<Value>,
    pub return_site_span: Option<SourceSpan>,
}

impl ScriptFiber {
    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }

    pub fn function_name(&self) -> &str {
        self.stack
            .first()
            .map(|f| f.function_name.as_str())
            .unwrap_or("unknown")
    }

    pub fn program_counter(&self) -> usize {
        self.stack.last().map(|f| f.pc).unwrap_or(0)
    }

    pub fn is_finished(&self) -> bool {
        self.finished
    }

    pub fn instance(&self) -> &ScriptInstance {
        &self.instance
    }

    pub fn instance_mut(&mut self) -> &mut ScriptInstance {
        &mut self.instance
    }

    pub fn result(&self) -> Option<&Value> {
        self.result.as_ref()
    }
}

pub struct Interpreter {
    pub(super) program: Program,
    pub(super) output: Vec<LogRecord>,
    pub(super) operation_budget: u64,
    pub(super) operations_remaining: u64,
    pub(super) max_call_depth: usize,
    pub(super) call_depth: usize,
    pub(super) compiled_functions: BTreeMap<(String, String), Arc<CompiledFunction>>,

    pub(super) current_script_path: Option<String>,
    pub(super) current_entity_name: Option<String>,
    pub(super) current_entity_id: Option<u64>,
    pub(super) current_function_name: Option<String>,
}

impl Clone for Interpreter {
    fn clone(&self) -> Self {
        Self {
            program: self.program.clone(),
            output: self.output.clone(),
            operation_budget: self.operation_budget,
            operations_remaining: self.operations_remaining,
            max_call_depth: self.max_call_depth,
            call_depth: self.call_depth,
            compiled_functions: self.compiled_functions.clone(),
            current_script_path: self.current_script_path.clone(),
            current_entity_name: self.current_entity_name.clone(),
            current_entity_id: self.current_entity_id,
            current_function_name: self.current_function_name.clone(),
        }
    }
}

impl std::fmt::Debug for Interpreter {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Interpreter")
            .field("program", &self.program)
            .field("output", &self.output)
            .field("operation_budget", &self.operation_budget)
            .field("operations_remaining", &self.operations_remaining)
            .field("max_call_depth", &self.max_call_depth)
            .field("call_depth", &self.call_depth)
            .field(
                "compiled_functions",
                &self.compiled_functions.keys().collect::<Vec<_>>(),
            )
            .finish()
    }
}

impl Interpreter {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            output: Vec::new(),
            operation_budget: DEFAULT_OPERATION_BUDGET,
            operations_remaining: DEFAULT_OPERATION_BUDGET,
            max_call_depth: DEFAULT_MAX_CALL_DEPTH,
            call_depth: 0,
            compiled_functions: BTreeMap::new(),
            current_script_path: None,
            current_entity_name: None,
            current_entity_id: None,
            current_function_name: None,
        }
    }

    pub fn call_function_by_name(
        &mut self,
        instance: &mut ScriptInstance,
        name: &str,
        arguments: Vec<Value>,
        host: &mut HostContext,
    ) -> Result<Value, String> {
        self.call(instance, name, arguments, host)
    }

    pub fn with_limits(program: Program, operation_budget: u64, max_call_depth: usize) -> Self {
        Self {
            program,
            output: Vec::new(),
            operation_budget,
            operations_remaining: operation_budget,
            max_call_depth,
            call_depth: 0,
            compiled_functions: BTreeMap::new(),
            current_script_path: None,
            current_entity_name: None,
            current_entity_id: None,
            current_function_name: None,
        }
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn output(&self) -> &[LogRecord] {
        &self.output
    }

    pub fn drain_output(&mut self) -> Vec<LogRecord> {
        std::mem::take(&mut self.output)
    }

    pub fn log_error(&mut self, message: String) {
        let record = LogRecord {
            severity: LogSeverity::Error,
            script_path: self.current_script_path.clone(),
            entity_name: self.current_entity_name.clone(),
            entity_id: self.current_entity_id,
            context_name: self.current_function_name.clone(),
            message,
            line: None,
            column: None,
        };

        self.output.push(record);
    }

    pub fn log_warning(&mut self, message: String) {
        let record = LogRecord {
            severity: LogSeverity::Warning,
            script_path: self.current_script_path.clone(),
            entity_name: self.current_entity_name.clone(),
            entity_id: self.current_entity_id,
            context_name: self.current_function_name.clone(),
            message,
            line: None,
            column: None,
        };

        self.output.push(record);
    }

    pub fn log_system(&mut self, message: String) {
        let record = LogRecord {
            severity: LogSeverity::System,
            script_path: self.current_script_path.clone(),
            entity_name: self.current_entity_name.clone(),
            entity_id: self.current_entity_id,
            context_name: self.current_function_name.clone(),
            message,
            line: None,
            column: None,
        };

        self.output.push(record);
    }

    pub fn instantiate_entity(
        &mut self,
        entity_name: &str,
        id: u64,
        host: &mut HostContext,
    ) -> Result<ScriptInstance, String> {
        self.reset_execution_budget();

        let entity = self
            .find_entity(entity_name)
            .ok_or_else(|| format!("entity '{}' does not exist", entity_name))?;

        let fields = entity
            .members
            .iter()
            .filter_map(|member| match member {
                EntityMember::Field(field) => Some(field.clone()),
                EntityMember::Function(_) => None,
            })
            .collect::<Vec<_>>();

        let mut instance = ScriptInstance {
            id,
            entity_name: entity_name.to_string(),
            script_path: None,
            fields: BTreeMap::new(),
        };

        let mut scopes = Vec::new();

        for field in fields {
            self.tick()?;

            let value = if let Some(initializer) = &field.initializer {
                self.eval_expression(&mut instance, &mut scopes, initializer, host, &[])?
            } else {
                Value::Nil
            };

            instance.fields.insert(field.name, value);
        }

        Ok(instance)
    }

    pub fn call(
        &mut self,
        instance: &mut ScriptInstance,
        function_name: &str,
        arguments: Vec<Value>,
        host: &mut HostContext,
    ) -> Result<Value, String> {
        let function = self
            .find_function(&instance.entity_name, function_name)
            .ok_or_else(|| {
                format!(
                    "function '{}' does not exist on entity '{}'",
                    function_name, instance.entity_name
                )
            })?;

        let old_budget = self.operations_remaining;
        let old_depth = self.call_depth;
        let old_path = self.current_script_path.clone();
        let old_entity = self.current_entity_name.clone();
        let old_id = self.current_entity_id;
        let old_function = self.current_function_name.clone();

        if old_function.is_none() {
            self.operations_remaining = self.operation_budget;
            self.call_depth = 0;
        }

        self.current_script_path = instance.script_path.clone();
        self.current_entity_name = Some(instance.entity_name().to_string());
        self.current_entity_id = Some(instance.id());
        self.current_function_name = Some(function_name.to_string());

        let result = self.call_user_function(instance, &function, arguments, host, &[]);

        self.operations_remaining = old_budget;
        self.call_depth = old_depth;
        self.current_script_path = old_path;
        self.current_entity_name = old_entity;
        self.current_entity_id = old_id;
        self.current_function_name = old_function;

        result
    }

    pub(super) fn call_direct(
        &mut self,
        instance: &mut ScriptInstance,
        compiled: Arc<CompiledFunction>,
        param_names: Vec<String>,
        arguments: Vec<Value>,
        self_value: Option<Value>,
        function_name: String,
        host: &mut HostContext,
        captured_scopes: Vec<Scope>,
    ) -> Result<Value, String> {
        if self.call_depth >= self.max_call_depth {
            return Err(format!(
                "AeoScript call depth exceeded the limit of {}.",
                self.max_call_depth
            ));
        }

        if param_names.len() != arguments.len() {
            return Err(format!(
                "function expected {} argument(s), got {}",
                param_names.len(),
                arguments.len()
            ));
        }

        let parameter_scope = Scope::new();

        if let Some(self_value) = self_value {
            parameter_scope.declare("self", self_value, false)?;
        }

        for (name, argument) in param_names.into_iter().zip(arguments.into_iter()) {
            parameter_scope.declare(name, argument, false)?;
        }

        let mut fiber = ScriptFiber {
            script_path: instance.script_path.clone(),
            entity_name: instance.entity_name().to_string(),
            instance: instance.clone(),
            stack: vec![CallFrame {
                function: compiled,
                pc: 0,
                scopes: vec![parameter_scope],
                for_states: Vec::new(),
                function_name,
                captured_scopes,
                expects_return_value: false,
                call_site_span: None,
            }],
            finished: false,
            result: None,
            return_value: None,
            return_site_span: None,
        };

        self.call_depth += 1;

        let result = self.resume_fiber(&mut fiber, host);

        self.call_depth -= 1;

        *instance = fiber.instance;

        match result {
            FiberResult::Complete => Ok(fiber.result.unwrap_or(Value::Nil)),
            FiberResult::Yield(_) => {
                Err("yielding is not supported in this synchronous call context".to_string())
            }
            FiberResult::Failed(message) => Err(message),
            _ => Ok(Value::Nil),
        }
    }

    pub(super) fn call_user_function(
        &mut self,
        instance: &mut ScriptInstance,
        function: &FunctionDecl,
        arguments: Vec<Value>,
        host: &mut HostContext,
        captured_scopes: &[Scope],
    ) -> Result<Value, String> {
        if self.call_depth >= self.max_call_depth {
            return Err(format!(
                "AeoScript call depth exceeded the limit of {}.",
                self.max_call_depth
            ));
        }

        if function.parameters.len() != arguments.len() {
            return Err(format!(
                "function '{}' expected {} argument(s), got {}",
                function.name,
                function.parameters.len(),
                arguments.len()
            ));
        }

        self.call_depth += 1;

        let result = (|| {
            let parameter_scope = Scope::new();

            for (parameter, argument) in function.parameters.iter().zip(arguments.into_iter()) {
                parameter_scope.declare(parameter.name.clone(), argument, false)?;
            }

            let mut scopes = vec![parameter_scope];

            match self.execute_block(
                instance,
                &mut scopes,
                &function.body,
                host,
                captured_scopes,
            )? {
                ExecutionFlow::Continue => Ok(Value::Nil),
                ExecutionFlow::Return(value) => Ok(value),
            }
        })();

        self.call_depth -= 1;

        result
    }

    pub(super) fn find_entity(&self, name: &str) -> Option<EntityDecl> {
        self.program
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Entity(entity) if entity.name == name => Some(entity.clone()),

                _ => None,
            })
    }

    pub(super) fn find_function(&self, entity_name: &str, function_name: &str) -> Option<FunctionDecl> {
        if let Some(entity) = self.find_entity(entity_name) {
            if let Some(function) = entity.members.into_iter().find_map(|member| match member {
                EntityMember::Function(function) if function.name == function_name => {
                    Some(function)
                }

                _ => None,
            }) {
                return Some(function);
            }
        }

        self.program
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Function(function) if function.name == function_name => {
                    Some(function.clone())
                }

                _ => None,
            })
    }
}
