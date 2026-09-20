use std::collections::BTreeMap;
use std::sync::Arc;

use super::ast::*;
use super::execution::{FiberResult, YieldReason};
use super::value::{Scope, Value, HandleKind, MapKey};
use super::api::{HostContext, call_host_function, call_host_member, resolve_host_property, resolve_host_member_property, set_host_member_property};
use super::log::{LogSeverity, LogRecord};
use std::cell::RefCell;

const DEFAULT_OPERATION_BUDGET: u64 = 100_000;
const DEFAULT_MAX_CALL_DEPTH: usize = 64;

/// A live instance of one AeoScript entity definition.
///
/// Fields belong to the script instance. Local variables belong to temporary
/// execution scopes created while functions run.
#[derive(Clone, Debug)]
pub struct ScriptInstance {
    id: u64,
    entity_name: String,
    pub script_path: Option<String>,
    fields: BTreeMap<String, Value>,
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
        if !self.fields.contains_key(name) {
            return Err(format!(
                "field '{}' does not exist on entity '{}'",
                name, self.entity_name
            ));
        }

        self.fields.insert(name.to_string(), value);

        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ControlFlow {
    Continue,
}

#[derive(Clone, Debug)]
enum ExecutionFlow {
    Continue,
    Return(Value),
}

/// Internal resumable instruction.
///
/// This is still an interpreter implementation. These instructions are an
/// execution plan used only to preserve control-flow position across yields.
/// They are deliberately small and can later map cleanly to the real AeoScript
/// bytecode VM.
#[derive(Clone, Debug)]
enum Instruction {
    EnterScope,

    ExitScope,

    Variable {
        name: String,
        initializer: Option<Expression>,
        is_const: bool,
    },

    Assignment {
        target: Expression,
        operator: AssignmentOperator,
        value: Expression,
    },

    Evaluate(Expression),

    Jump {
        target: usize,
    },

    JumpIfFalse {
        condition: Expression,
        target: usize,
    },

    ForInit {
        name: String,
        iterable: Expression,
        end: usize,
    },

    ForNext {
        body_start: usize,
        end: usize,
    },

    CallUserFunction {
        name: String,
        arguments: Vec<Expression>,
    },

    Wait {
        arguments: Vec<Expression>,
    },

    Return(Option<Expression>),
}

/// One persistent state for a resumable AeoScript execution.
#[derive(Clone, Debug)]
struct ForState {
    name: String,
    values: Vec<Value>,
    next_index: usize,
}

/// Compiled execution plan for one function.
///
/// The AST remains the source representation. This is only the runtime
/// execution representation used by resumable fibers.
#[derive(Clone, Debug)]
struct CompiledFunction {
    instructions: Vec<Instruction>,
}

#[derive(Clone, Debug)]
struct CallFrame {
    function: Arc<CompiledFunction>,
    pc: usize,
    scopes: Vec<Scope>,
    for_states: Vec<ForState>,
    /// If this frame was called as part of an expression, this stores where to put the result.
    /// Actually, for now let's just support calls as statements.
    function_name: String,
}

/// A cooperative AeoScript execution fiber.
///
/// The fiber owns its runtime execution state but not the interpreter itself.
/// That keeps the interpreter single-threaded while allowing many independent
/// script fibers to exist.
#[derive(Debug)]
pub struct ScriptFiber {
    pub script_path: Option<String>,
    pub entity_name: String,
    instance: ScriptInstance,
    stack: Vec<CallFrame>,
    finished: bool,
    result: Option<Value>,
}

impl ScriptFiber {
    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }

    pub fn function_name(&self) -> &str {
        self.stack.first().map(|f| f.function_name.as_str()).unwrap_or("unknown")
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

    fn fail(&mut self, message: String) -> FiberResult {
        self.finished = true;

        FiberResult::Failed(message)
    }

    fn complete(&mut self, value: Value) -> FiberResult {
        self.finished = true;
        self.result = Some(value);

        FiberResult::Complete
    }
}

pub struct Interpreter {
    program: Program,
    output: Vec<LogRecord>,
    operation_budget: u64,
    operations_remaining: u64,
    max_call_depth: usize,
    call_depth: usize,
    compiled_functions: BTreeMap<(String, String), Arc<CompiledFunction>>,

    // Current execution context for logging
    current_script_path: Option<String>,
    current_entity_name: Option<String>,
    current_entity_id: Option<u64>,
    current_function_name: Option<String>,
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
        let function = self.find_function(&instance.entity_name, name)
            .ok_or_else(|| format!("function '{}' does not exist", name))?;
        self.call_user_function(instance, &function, arguments, host)
    }

    pub fn with_limits(
        program: Program,
        operation_budget: u64,
        max_call_depth: usize,
    ) -> Self {
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
                self.eval_expression(&mut instance, &mut scopes, initializer, host)?
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
        self.reset_execution_budget();

        self.current_script_path = instance.script_path.clone();
        self.current_entity_name = Some(instance.entity_name().to_string());
        self.current_entity_id = Some(instance.id());
        self.current_function_name = Some(function_name.to_string());

        let function = self
            .find_function(&instance.entity_name, function_name)
            .ok_or_else(|| {
                format!(
                    "function '{}' does not exist on entity '{}'",
                    function_name, instance.entity_name
                )
            })?;

        let result = self.call_user_function(instance, &function, arguments, host);

        self.current_script_path = None;
        self.current_entity_name = None;
        self.current_entity_id = None;
        self.current_function_name = None;

        result
    }

    /// Creates a persistent fiber for a function.
    ///
    /// The function body is compiled once into a compact execution plan.
    /// Subsequent resumes only advance the existing program counter and runtime
    /// state.
    pub fn start_fiber(
        &mut self,
        instance: ScriptInstance,
        function_name: &str,
        arguments: Vec<Value>,
    ) -> Result<ScriptFiber, String> {
        let entity_name = instance.entity_name.clone();

        let function = self
            .find_function(&entity_name, function_name)
            .ok_or_else(|| {
                format!(
                    "function '{}' does not exist on entity '{}'",
                    function_name, entity_name
                )
            })?;

        if function.parameters.len() != arguments.len() {
            return Err(format!(
                "function '{}' expected {} argument(s), got {}",
                function.name,
                function.parameters.len(),
                arguments.len()
            ));
        }

        let cache_key = (entity_name.clone(), function_name.to_string());

        let compiled = if let Some(compiled) =
            self.compiled_functions.get(&cache_key)
        {
            Arc::clone(compiled)
        } else {
            let compiled = Arc::new(compile_function(&function));

            self.compiled_functions
                .insert(cache_key, Arc::clone(&compiled));

            compiled
        };

        let mut parameter_scope = Scope::new();

        for (parameter, argument) in function
            .parameters
            .iter()
            .zip(arguments.into_iter())
        {
            parameter_scope
                .declare(parameter.name.clone(), argument, false)
                .map_err(|error| {
                    format!(
                        "failed to bind parameter '{}': {}",
                        parameter.name, error
                    )
                })?;
        }

        Ok(ScriptFiber {
            script_path: instance.script_path.clone(),
            entity_name,
            instance,
            stack: vec![CallFrame {
                function: compiled,
                pc: 0,
                scopes: vec![parameter_scope],
                for_states: Vec::new(),
                function_name: function_name.to_string(),
            }],
            finished: false,
            result: None,
        })
    }

    pub fn start_event_fiber(
        &mut self,
        instance: ScriptInstance,
        event: EventDecl,
        arguments: Vec<Value>,
    ) -> Result<ScriptFiber, String> {
        if event.parameters.len() != arguments.len() {
            return Err(format!(
                "event '{}' expected {} argument(s), got {}",
                event.name,
                event.parameters.len(),
                arguments.len()
            ));
        }

        let compiled = Arc::new(compile_event(&event));

        let mut parameter_scope = Scope::new();

        for (parameter, argument) in event
            .parameters
            .iter()
            .zip(arguments.into_iter())
        {
            parameter_scope
                .declare(parameter.name.clone(), argument, false)
                .map_err(|error| {
                    format!(
                        "failed to bind parameter '{}': {}",
                        parameter.name, error
                    )
                })?;
        }

        Ok(ScriptFiber {
            script_path: instance.script_path.clone(),
            entity_name: instance.entity_name().to_string(),
            instance,
            stack: vec![CallFrame {
                function: compiled,
                pc: 0,
                scopes: vec![parameter_scope],
                for_states: Vec::new(),
                function_name: event.name.clone(),
            }],
            finished: false,
            result: None,
        })
    }

    pub fn resume_fiber(&mut self, fiber: &mut ScriptFiber, host: &mut HostContext) -> FiberResult {
        if fiber.finished {
            return FiberResult::Failed(
                "cannot resume a completed AeoScript fiber.".to_string(),
            );
        }

        self.operations_remaining = self.operation_budget;
        self.call_depth = 0;

        self.current_script_path = fiber.script_path.clone();
        self.current_entity_name = Some(fiber.entity_name.clone());
        self.current_entity_id = Some(fiber.instance().id());

        let result = loop {
            let Some(frame) = fiber.stack.last_mut() else {
                break fiber.complete(Value::Nil);
            };

            self.current_function_name = Some(frame.function_name.clone());

            if frame.pc >= frame.function.instructions.len() {
                fiber.stack.pop();
                if fiber.stack.is_empty() {
                    break fiber.complete(Value::Nil);
                }
                continue;
            }

            if let Err(error) = self.tick() {
                self.log_error(error.clone());
                break fiber.fail(error);
            }

            let instruction = frame.function.instructions[frame.pc].clone();
            match instruction {
                Instruction::EnterScope => {
                    frame.scopes.push(Scope::new());

                    frame.pc += 1;
                }

                Instruction::ExitScope => {
                    if frame.scopes.len() <= 1 {
                        let err = "AeoScript runtime scope underflow.".to_string();
                        self.log_error(err.clone());
                        break fiber.fail(err);
                    }

                    frame.scopes.pop();

                    frame.pc += 1;
                }

                Instruction::Variable {
                    name,
                    initializer,
                    is_const,
                } => {
                    let value = match initializer {
                        Some(initializer) => {
                            match self.eval_expression(
                                &mut fiber.instance,
                                &mut frame.scopes,
                                &initializer,
                                host,
                            ) {
                                Ok(value) => value,

                                Err(error) => {
                                    self.log_error(error.clone());
                                    break fiber.fail(error);
                                }
                            }
                        }

                        None => Value::Nil,
                    };

                    let scope = match frame.scopes.last_mut() {
                        Some(scope) => scope,

                        None => {
                            let err = "AeoScript runtime has no active scope.".to_string();
                            self.log_error(err.clone());
                            break fiber.fail(err);
                        }
                    };

                    if let Err(error) =
                        scope.declare(name.clone(), value, is_const)
                    {
                        self.log_error(error.clone());
                        break fiber.fail(error);
                    }

                    frame.pc += 1;
                }

                Instruction::Assignment {
                    target,
                    operator,
                    value,
                } => {
                    let right = match self.eval_expression(
                        &mut fiber.instance,
                        &mut frame.scopes,
                        &value,
                        host,
                    ) {
                        Ok(value) => value,

                        Err(error) => {
                            self.log_error(error.clone());
                            break fiber.fail(error);
                        }
                    };

                    if let Err(error) = self.assign_target(
                        &mut fiber.instance,
                        &mut frame.scopes,
                        &target,
                        operator,
                        right,
                        host,
                    ) {
                        self.log_error(error.clone());
                        break fiber.fail(error);
                    }

                    frame.pc += 1;
                }

                Instruction::Evaluate(expression) => {
                    if let Err(error) = self.eval_expression(
                        &mut fiber.instance,
                        &mut frame.scopes,
                        &expression,
                        host,
                    ) {
                        self.log_error(error.clone());
                        break fiber.fail(error);
                    }

                    frame.pc += 1;
                }

                Instruction::Jump { target } => {
                    frame.pc = target;
                }

                Instruction::JumpIfFalse {
                    condition,
                    target,
                } => {
                    let value = match self.eval_expression(
                        &mut fiber.instance,
                        &mut frame.scopes,
                        &condition,
                        host,
                    ) {
                        Ok(value) => value,

                        Err(error) => {
                            self.log_error(error.clone());
                            break fiber.fail(error);
                        }
                    };

                    match value.is_truthy() {
                        Ok(true) => frame.pc += 1,

                        Ok(false) => frame.pc = target,

                        Err(error) => {
                            self.log_error(error.clone());
                            break fiber.fail(error);
                        }
                    }
                }

                Instruction::ForInit {
                    name,
                    iterable,
                    end,
                } => {
                    let iterable_value = match self.eval_expression(
                        &mut fiber.instance,
                        &mut frame.scopes,
                        &iterable,
                        host,
                    ) {
                        Ok(value) => value,

                        Err(error) => {
                            self.log_error(error.clone());
                            break fiber.fail(error);
                        }
                    };

                    let values = match iterable_value {
                        Value::Array(values) => values.borrow().elements.clone(),

                        other => {
                            let err = format!(
                                "cannot iterate over {} in for loop",
                                other.type_name()
                            );
                            self.log_error(err.clone());
                            break fiber.fail(err);
                        }
                    };

                    if values.is_empty() {
                        frame.pc = end;
                        continue;
                    }

                    let first_value = values[0].clone();

                    frame.for_states.push(ForState {
                        name: name.clone(),
                        values,
                        next_index: 1,
                    });

                    let scope = match frame.scopes.last_mut() {
                        Some(scope) => scope,

                        None => {
                            let err = "AeoScript runtime has no active scope.".to_string();
                            self.log_error(err.clone());
                            break fiber.fail(err);
                        }
                    };

                    if let Err(error) =
                        scope.declare(name.clone(), first_value, false)
                    {
                        self.log_error(error.clone());
                        break fiber.fail(error);
                    }

                    frame.pc += 1;
                }

                Instruction::ForNext { body_start, end } => {
                    let Some(loop_state) = frame.for_states.last_mut() else {
                        let err = "AeoScript runtime for-loop state underflow.".to_string();
                        self.log_error(err.clone());
                        break fiber.fail(err);
                    };

                    if loop_state.next_index < loop_state.values.len() {
                        let value =
                            loop_state.values[loop_state.next_index].clone();

                        loop_state.next_index += 1;

                        let name = loop_state.name.clone();

                        let scope = match frame.scopes.last_mut() {
                            Some(scope) => scope,

                            None => {
                                let err = "AeoScript runtime has no active scope.".to_string();
                                self.log_error(err.clone());
                                break fiber.fail(err);
                            }
                        };

                        if let Err(error) =
                        scope.set_or_declare(name.clone(), value)
                    {
                            self.log_error(error.clone());
                            break fiber.fail(error);
                        }

                        frame.pc = body_start;
                    } else {
                        frame.for_states.pop();

                        frame.pc = end;
                    }
                }

                Instruction::CallUserFunction { name, arguments } => {
                    let values = match self.eval_arguments(&mut fiber.instance, &mut frame.scopes, &arguments, host) {
                        Ok(v) => v,
                        Err(e) => {
                            self.log_error(e.clone());
                            break fiber.fail(e);
                        }
                    };

                    // Check if it's a host function
                    if let Ok(Some(res)) = call_host_function(host, &name, &values) {
                        // For now we don't store the result because CallUserFunction is only for statements.
                        let _ = res;
                        frame.pc += 1;
                        continue;
                    }

                    // Built-ins
                    match name.as_str() {
                        "print" => {
                            self.log_values(&values);
                            frame.pc += 1;
                            continue;
                        }
                        "get_parent" => {
                            // Only statements, discard result
                            frame.pc += 1;
                            continue;
                        }
                        "wait" => {
                            // Should have been handled by Instruction::Wait, but if we're here:
                            if values.len() != 1 {
                                let err = "wait() expects exactly one argument.".to_string();
                                self.log_error(err.clone());
                                break fiber.fail(err);
                            }
                            let seconds = values[0].as_number().unwrap_or(0.0);
                            frame.pc += 1;
                            break FiberResult::Yield(YieldReason::WaitSeconds(seconds));
                        }
                        _ => {}
                    }

                    let function = match self.find_function(&fiber.instance.entity_name, &name) {
                        Some(f) => f,
                        None => {
                            let err = format!("function '{}' does not exist", name);
                            self.log_error(err.clone());
                            break fiber.fail(err);
                        }
                    };

                    if function.parameters.len() != values.len() {
                        let err = format!(
                            "function '{}' expected {} argument(s), got {}",
                            function.name,
                            function.parameters.len(),
                            values.len()
                        );
                        self.log_error(err.clone());
                        break fiber.fail(err);
                    }

                    let cache_key = (fiber.instance.entity_name.clone(), name.clone());
                    let compiled = if let Some(compiled) = self.compiled_functions.get(&cache_key) {
                        Arc::clone(compiled)
                    } else {
                        let compiled = Arc::new(compile_function(&function));
                        self.compiled_functions.insert(cache_key, Arc::clone(&compiled));
                        compiled
                    };

                    let mut parameter_scope = Scope::new();
                    for (parameter, argument) in function.parameters.iter().zip(values.into_iter()) {
                        parameter_scope.declare(parameter.name.clone(), argument, false).unwrap();
                    }

                    frame.pc += 1; // Advance caller PC BEFORE pushing new frame

                    fiber.stack.push(CallFrame {
                        function: compiled,
                        pc: 0,
                        scopes: vec![parameter_scope],
                        for_states: Vec::new(),
                        function_name: name,
                    });
                }

                Instruction::Wait { arguments } => {
                    if arguments.len() != 1 {
                        let err = "wait() expects exactly one argument.".to_string();
                        self.log_error(err.clone());
                        break fiber.fail(err);
                    }

                    let seconds = match self.eval_expression(
                        &mut fiber.instance,
                        &mut frame.scopes,
                        &arguments[0],
                        host,
                    ) {
                        Ok(value) => match value.as_number() {
                            Ok(value) => value,

                            Err(error) => {
                                self.log_error(error.clone());
                                break fiber.fail(error);
                            }
                        },

                        Err(error) => {
                            self.log_error(error.clone());
                            break fiber.fail(error);
                        }
                    };

                    if !seconds.is_finite() {
                        let err = "AeoScript wait duration must be finite.".to_string();
                        self.log_error(err.clone());
                        break fiber.fail(err);
                    }

                    if seconds <= 0.0 {
                        let err = "AeoScript wait duration must be greater than zero.".to_string();
                        self.log_error(err.clone());
                        break fiber.fail(err);
                    }

                    frame.pc += 1;

                    break FiberResult::Yield(
                        YieldReason::WaitSeconds(seconds),
                    );
                }

                Instruction::Return(expression) => {
                    let value = match expression {
                        Some(expression) => match self.eval_expression(
                            &mut fiber.instance,
                            &mut frame.scopes,
                            &expression,
                            host,
                        ) {
                            Ok(value) => value,

                            Err(error) => {
                                self.log_error(error.clone());
                                break fiber.fail(error);
                            }
                        },

                        None => Value::Nil,
                    };

                    fiber.stack.pop();
                    if fiber.stack.is_empty() {
                        break fiber.complete(value);
                    }
                    // For now we only support calls as statements, so we ignore 'value' here
                    // unless we implement result passing.
                }
            }
        };

        self.current_script_path = None;
        self.current_entity_name = None;
        self.current_entity_id = None;
        self.current_function_name = None;

        result
    }

    fn reset_execution_budget(&mut self) {
        self.operations_remaining = self.operation_budget;
        self.call_depth = 0;
    }

    fn tick(&mut self) -> Result<(), String> {
        if self.operations_remaining == 0 {
            return Err("AeoScript execution budget exhausted.".to_string());
        }

        self.operations_remaining -= 1;

        Ok(())
    }

    fn call_user_function(
        &mut self,
        instance: &mut ScriptInstance,
        function: &FunctionDecl,
        arguments: Vec<Value>,
        host: &mut HostContext,
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

        let result = {
            let mut scopes = vec![Scope::new()];

            for (parameter, argument) in function
                .parameters
                .iter()
                .zip(arguments.into_iter())
            {
                scopes[0]
                    .declare(parameter.name.clone(), argument, false)
                    .map_err(|error| {
                        format!(
                            "failed to bind parameter '{}': {}",
                            parameter.name, error
                        )
                    })?;
            }

            match self.execute_block(
                instance,
                &mut scopes,
                &function.body,
                host,
            )? {
                ExecutionFlow::Continue => Ok(Value::Nil),

                ExecutionFlow::Return(value) => Ok(value),
            }
        };

        self.call_depth -= 1;

        result
    }

    fn execute_block(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        block: &Block,
        host: &mut HostContext,
    ) -> Result<ExecutionFlow, String> {
        scopes.push(Scope::new());

        let result = (|| {
            for statement in &block.statements {
                match self.execute_statement(
                    instance,
                    scopes,
                    statement,
                    host,
                )? {
                    ExecutionFlow::Continue => {}

                    ExecutionFlow::Return(value) => {
                        return Ok(ExecutionFlow::Return(value));
                    }
                }
            }

            Ok(ExecutionFlow::Continue)
        })();

        scopes.pop();

        result
    }

    fn execute_statement(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        statement: &Statement,
        host: &mut HostContext,
    ) -> Result<ExecutionFlow, String> {
        self.tick()?;

        match &statement.kind {
            StatementKind::Variable {
                name,
                initializer,
                is_const,
                ..
            } => {
                let value = if let Some(initializer) = initializer {
                    self.eval_expression(
                        instance,
                        scopes,
                        initializer,
                        host,
                    )?
                } else {
                    Value::Nil
                };

                scopes
                    .last_mut()
                    .expect("function execution always has a scope")
                    .declare(name.clone(), value, *is_const)?;

                Ok(ExecutionFlow::Continue)
            }

            StatementKind::Assignment {
                target,
                operator,
                value,
            } => {
                let right = self.eval_expression(
                    instance,
                    scopes,
                    value,
                    host,
                )?;

                self.assign_target(
                    instance,
                    scopes,
                    target,
                    *operator,
                    right,
                    host,
                )?;

                Ok(ExecutionFlow::Continue)
            }

            StatementKind::If {
                condition,
                then_block,
                else_if,
                else_block,
            } => {
                let condition_value =
                    self.eval_expression(instance, scopes, condition, host)?;

                if condition_value.is_truthy()? {
                    return self.execute_block(
                        instance,
                        scopes,
                        then_block,
                        host,
                    );
                }

                for (else_if_condition, block) in else_if {
                    let value = self.eval_expression(
                        instance,
                        scopes,
                        else_if_condition,
                        host,
                    )?;

                    if value.is_truthy()? {
                        return self.execute_block(
                            instance,
                            scopes,
                            block,
                            host,
                        );
                    }
                }

                if let Some(block) = else_block {
                    return self.execute_block(
                        instance,
                        scopes,
                        block,
                        host,
                    );
                }

                Ok(ExecutionFlow::Continue)
            }

            StatementKind::While { condition, body } => {
                loop {
                    self.tick()?;

                    let value =
                        self.eval_expression(instance, scopes, condition, host)?;

                    if !value.is_truthy()? {
                        break;
                    }

                    match self.execute_block(
                        instance,
                        scopes,
                        body,
                        host,
                    )? {
                        ExecutionFlow::Continue => {}

                        ExecutionFlow::Return(value) => {
                            return Ok(ExecutionFlow::Return(value));
                        }
                    }
                }

                Ok(ExecutionFlow::Continue)
            }

            StatementKind::For {
                name,
                iterable,
                body,
            } => {
                let iterable_value =
                    self.eval_expression(instance, scopes, iterable, host)?;

                let values = match iterable_value {
                    Value::Array(values) => values.borrow().elements.clone(),

                    other => {
                        return Err(format!(
                            "cannot iterate over {} in for loop",
                            other.type_name()
                        ));
                    }
                };

                scopes.push(Scope::new());

                let result = (|| {
                    for value in values {
                        self.tick()?;

                        scopes
                            .last_mut()
                            .expect("loop scope exists")
                            .set_or_declare(name.to_string(), value)?;

                        match self.execute_block(
                            instance,
                            scopes,
                            body,
                            host,
                        )? {
                            ExecutionFlow::Continue => {}

                            ExecutionFlow::Return(value) => {
                                return Ok(
                                    ExecutionFlow::Return(value)
                                );
                            }
                        }
                    }

                    Ok(ExecutionFlow::Continue)
                })();

                scopes.pop();

                result
            }

            StatementKind::Return(value) => {
                let result = if let Some(expression) = value {
                    self.eval_expression(
                        instance,
                        scopes,
                        expression,
                        host,
                    )?
                } else {
                    Value::Nil
                };

                Ok(ExecutionFlow::Return(result))
            }

            StatementKind::Expression(expression) => {
                self.eval_expression(instance, scopes, expression, host)?;

                Ok(ExecutionFlow::Continue)
            }
        }
    }

    fn assign_target(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        target: &Expression,
        operator: AssignmentOperator,
        right: Value,
        host: &mut HostContext,
    ) -> Result<(), String> {
        match &target.kind {
            ExpressionKind::Identifier(name) => {
                let value = if operator == AssignmentOperator::Assign {
                    right
                } else {
                    let left =
                        self.resolve_identifier(instance, scopes, name)?;

                    self.apply_assignment(operator, left, right)?
                };

                for scope in scopes.iter_mut().rev() {
                    if scope.contains(name) {
                        return scope.set(name, value);
                    }
                }

                instance.set_field(name, value)
            }

            ExpressionKind::Member { object, name } => {
                let object_value = self.eval_expression(instance, scopes, object, host)?;

                let value = if operator == AssignmentOperator::Assign {
                    right
                } else {
                    let left = match &object_value {
                        Value::Map(map) => map.borrow().get(&MapKey::String(name.clone())).cloned().unwrap_or(Value::Nil),
                        Value::Handle { kind, id } => {
                            if let Some(v) = resolve_host_member_property(host, *kind, *id, name)? {
                                v
                            } else {
                                return Err(format!("engine member '{}' is not available on handle kind {} yet", name, kind.name()));
                            }
                        }
                        _ => return Err(format!("cannot access member on type {}", object_value.type_name())),
                    };
                    self.apply_assignment(operator, left, right)?
                };

                match object_value {
                    Value::Map(map) => {
                        if matches!(value, Value::Nil) {
                            map.borrow_mut().remove(&MapKey::String(name.clone()));
                        } else {
                            map.borrow_mut().insert(MapKey::String(name.clone()), value);
                        }
                        Ok(())
                    }
                    Value::Handle { kind, id } => {
                        set_host_member_property(host, kind, id, name, value)
                    }
                    _ => Err(format!("cannot assign to member of type {}", object_value.type_name())),
                }
            }

            ExpressionKind::Index { object, index } => {
                let object_value = self.eval_expression(instance, scopes, object, host)?;
                let index_value = self.eval_expression(instance, scopes, index, host)?;

                let value = if operator == AssignmentOperator::Assign {
                    right
                } else {
                    let left = match &object_value {
                        Value::Array(array) => {
                            let idx = index_value.as_index()?;
                            array.borrow().elements.get(idx).cloned().ok_or_else(|| format!("index {} out of bounds", idx))?
                        }
                        Value::Map(map) => {
                            let key = index_value.as_map_key()?;
                            map.borrow().get(&key).cloned().unwrap_or(Value::Nil)
                        }
                        _ => return Err(format!("cannot index type {}", object_value.type_name())),
                    };
                    self.apply_assignment(operator, left, right)?
                };

                match object_value {
                    Value::Array(array) => {
                        let idx = index_value.as_index()?;
                        let mut borrowed = array.borrow_mut();
                        if borrowed.frozen {
                            return Err("cannot mutate frozen basket".to_string());
                        }
                        if idx >= borrowed.elements.len() {
                            return Err(format!("index {} out of bounds", idx));
                        }
                        borrowed.elements[idx] = value;
                        Ok(())
                    }
                    Value::Map(map) => {
                        let key = index_value.as_map_key()?;
                        if matches!(value, Value::Nil) {
                            map.borrow_mut().remove(&key);
                        } else {
                            map.borrow_mut().insert(key, value);
                        }
                        Ok(())
                    }
                    _ => Err(format!("cannot assign to index of type {}", object_value.type_name())),
                }
            }

            _ => Err(format!("cannot assign to expression of type {:?}", target.kind)),
        }
    }

    fn apply_assignment(
        &self,
        operator: AssignmentOperator,
        left: Value,
        right: Value,
    ) -> Result<Value, String> {
        match operator {
            AssignmentOperator::Assign => Ok(right),

            AssignmentOperator::Add => {
                self.apply_binary(BinaryOperator::Add, left, right)
            }

            AssignmentOperator::Subtract => {
                self.apply_binary(
                    BinaryOperator::Subtract,
                    left,
                    right,
                )
            }

            AssignmentOperator::Multiply => {
                self.apply_binary(
                    BinaryOperator::Multiply,
                    left,
                    right,
                )
            }

            AssignmentOperator::Divide => {
                self.apply_binary(
                    BinaryOperator::Divide,
                    left,
                    right,
                )
            }
        }
    }

    fn eval_expression(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        expression: &Expression,
        host: &mut HostContext,
    ) -> Result<Value, String> {
        self.tick()?;

        match &expression.kind {
            ExpressionKind::Number(value) => {
                Ok(Value::Number(*value))
            }

            ExpressionKind::String(value) => {
                Ok(Value::String(value.clone()))
            }

            ExpressionKind::Bool(value) => Ok(Value::Bool(*value)),

            ExpressionKind::Nil => Ok(Value::Nil),

            ExpressionKind::Identifier(name) => {
                self.resolve_identifier(instance, scopes, name)
            }

            ExpressionKind::Unary {
                operator,
                expression,
            } => {
                let value =
                    self.eval_expression(instance, scopes, expression, host)?;

                match operator {
                    UnaryOperator::Negate => {
                        Ok(Value::Number(-value.as_number()?))
                    }

                    UnaryOperator::Not => {
                        Ok(Value::Bool(!value.is_truthy()?))
                    }
                }
            }

            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                if *operator == BinaryOperator::And {
                    let left_value =
                        self.eval_expression(instance, scopes, left, host)?;

                    if !left_value.is_truthy()? {
                        return Ok(Value::Bool(false));
                    }

                    let right_value =
                        self.eval_expression(instance, scopes, right, host)?;

                    return Ok(Value::Bool(
                        right_value.is_truthy()?
                    ));
                }

                if *operator == BinaryOperator::Or {
                    let left_value =
                        self.eval_expression(instance, scopes, left, host)?;

                    if left_value.is_truthy()? {
                        return Ok(Value::Bool(true));
                    }

                    let right_value =
                        self.eval_expression(instance, scopes, right, host)?;

                    return Ok(Value::Bool(
                        right_value.is_truthy()?
                    ));
                }

                let left_value =
                    self.eval_expression(instance, scopes, left, host)?;

                let right_value =
                    self.eval_expression(instance, scopes, right, host)?;

                self.apply_binary(
                    *operator,
                    left_value,
                    right_value,
                )
            }

            ExpressionKind::Call {
                callee,
                arguments,
            } => {
                self.eval_call(
                    instance,
                    scopes,
                    callee,
                    arguments,
                    host,
                )
            }

            ExpressionKind::MethodCall {
                object,
                method,
                arguments,
            } => {
                if let ExpressionKind::Identifier(ref object_name) = object.kind {
                    if object_name == "debug" && method == "log" {
                        let arg_values = self.eval_arguments(instance, scopes, arguments, host)?;
                        self.log_values(&arg_values);
                        return Ok(Value::Nil);
                    }
                    if object_name == "math" || object_name == "basket" || object_name == "string" {
                        let arg_values = self.eval_arguments(instance, scopes, arguments, host)?;
                        if let Some(res) = super::stdlib::call_stdlib_function(self, instance, scopes, object_name, method, &arg_values, host)? {
                            return Ok(res);
                        }
                    }
                }

                let object_value = self.eval_expression(instance, scopes, object, host)?;
                let arg_values = self.eval_arguments(instance, scopes, arguments, host)?;

                match object_value {
                    Value::Handle { kind, id } => {
                        if let Some(result) = call_host_member(host, kind, id, method, &arg_values)? {
                            Ok(result)
                        } else {
                            Err(format!(
                                "engine method '{}' is not available on handle kind {}",
                                method, kind.name()
                            ))
                        }
                    }

                    _ => Err(format!(
                        "cannot call method '{}' on {}",
                        method,
                        object_value.type_name()
                    )),
                }
            }

            ExpressionKind::Member { object, name } => {
                if let ExpressionKind::Identifier(ref object_name) = object.kind {
                    if let Some(value) = resolve_host_property(host, object_name, name)? {
                        return Ok(value);
                    }
                    if let Some(value) = super::stdlib::resolve_stdlib_property(object_name, name)? {
                        return Ok(value);
                    }
                }

                let value =
                    self.eval_expression(instance, scopes, object, host)?;

                match value {
                    Value::Map(map) => {
                        let borrowed = map.borrow();
                        Ok(borrowed.get(&MapKey::String(name.clone()))
                        .cloned()
                        .unwrap_or(Value::Nil))
                    }

                    Value::Handle { kind, id } => {
                        if let Some(value) = resolve_host_member_property(host, kind, id, name)? {
                            Ok(value)
                        } else {
                            Err(format!(
                                "engine member '{}' is not available on handle kind {} yet",
                                name, kind.name()
                            ))
                        }
                    }

                    other => Err(format!(
                        "cannot access member '{}' on {}",
                        name,
                        other.type_name()
                    )),
                }
            }

            ExpressionKind::Index { object, index } => {
                let object =
                    self.eval_expression(instance, scopes, object, host)?;

                let index =
                    self.eval_expression(instance, scopes, index, host)?;

                self.eval_index(object, index)
            }

            ExpressionKind::Array(expressions) => {
                let mut values =
                    Vec::with_capacity(expressions.len());

                for expression in expressions {
                    values.push(
                        self.eval_expression(
                            instance,
                            scopes,
                            expression,
                            host,
                        )?,
                    );
                }

                Ok(Value::array(values))
            }

            ExpressionKind::Map(entries) => {
                let mut values = BTreeMap::new();

                for (key_expr, expression) in entries {
                    let key_value = self.eval_expression(instance, scopes, key_expr, host)?;
                    let key = key_value.as_map_key()?;

                    values.insert(
                        key,
                        self.eval_expression(
                            instance,
                            scopes,
                            expression,
                            host,
                        )?,
                    );
                }

                Ok(Value::Map(Arc::new(RefCell::new(values))))
            }
        }
    }

    fn eval_call(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        callee: &Expression,
        arguments: &[Expression],
        host: &mut HostContext,
    ) -> Result<Value, String> {
        match &callee.kind {
            ExpressionKind::Identifier(name) => {
                let values =
                    self.eval_arguments(instance, scopes, arguments, host)?;

                if let Some(result) = call_host_function(host, name, &values)? {
                    return Ok(result);
                }

                match name.as_str() {
                    "print" => {
                        self.log_values(&values);

                        Ok(Value::Nil)
                    }

                    "wait" => Err(
                        "wait() is a yielding operation and must be used as a standalone statement on a resumable AeoScript fiber."
                            .to_string(),
                    ),

                    "get_parent" => {
                        if !values.is_empty() {
                            return Err("get_parent() expects no arguments".to_string());
                        }

                        if instance.id() == 0 {
                            return Err("get_parent() called from an unattached script".to_string());
                        }

                        if let Some((kind, id)) = host.engine.get_parent(HandleKind::Entity, instance.id()) {
                            Ok(Value::Handle { kind, id })
                        } else {
                            Ok(Value::Nil)
                        }
                    }

                    _ => {
                        let function =
                            self.find_function(
                                &instance.entity_name,
                                name,
                            )
                            .ok_or_else(|| {
                                format!(
                                    "function '{}' does not exist",
                                    name
                                )
                            })?;

                        self.call_user_function(
                            instance,
                            &function,
                            values,
                            host,
                        )
                    }
                }
            }

            ExpressionKind::Member { object, name } => {
                if let ExpressionKind::Identifier(ref object_name) = object.kind {
                    if object_name == "debug" && name == "log" {
                        let values = self.eval_arguments(instance, scopes, arguments, host)?;
                        self.log_values(&values);
                        return Ok(Value::Nil);
                    }

                    if object_name == "time" && name == "delta" {
                        if !arguments.is_empty() {
                            return Err("time.delta() expects no arguments".to_string());
                        }
                        return Ok(Value::Number(host.delta_time));
                    }

                    if object_name == "math" || object_name == "basket" || object_name == "string" {
                        let values = self.eval_arguments(instance, scopes, arguments, host)?;
                        if let Some(res) = super::stdlib::call_stdlib_function(self, instance, scopes, object_name, name, &values, host)? {
                            return Ok(res);
                        }
                    }
                }

                let object_value = self.eval_expression(instance, scopes, object, host)?;
                let values = self.eval_arguments(instance, scopes, arguments, host)?;

                match &object_value {
                    Value::Array(items) => {
                        if name == "len" {
                            if !values.is_empty() {
                                return Err("array.len() expects no arguments".to_string());
                            }
                            return Ok(Value::Number(items.borrow().elements.len() as f64));
                        }

                        let mut all_args = vec![object_value.clone()];
                        all_args.extend(values.iter().cloned());
                        if let Some(res) = super::stdlib::call_stdlib_function(self, instance, scopes, "basket", name, &all_args, host)? {
                            return Ok(res);
                        }
                    }

                    Value::Handle { kind, id } => {
                        if let Some(result) = call_host_member(host, *kind, *id, name, &values)? {
                            return Ok(result);
                        }
                    }
                    _ => {}
                }

                if let ExpressionKind::Identifier(ref object_name) = object.kind {
                     if object_name == "debug" {
                        return Err(format!("unknown debug function '{}'", name));
                     }
                }

                Err(format!("cannot call '{}' on type {}", name, object_value.type_name()))
            }

            _ => Err(
                "that expression cannot be called yet.".to_string()
            ),
        }
    }

    fn eval_arguments(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        arguments: &[Expression],
        host: &mut HostContext,
    ) -> Result<Vec<Value>, String> {
        let mut values = Vec::with_capacity(arguments.len());

        for argument in arguments {
            values.push(
                self.eval_expression(
                    instance,
                    scopes,
                    argument,
                    host,
                )?,
            );
        }

        Ok(values)
    }

    fn log_values(&mut self, values: &[Value]) {
        let message = values
            .iter()
            .map(Value::display_string)
            .collect::<Vec<_>>()
            .join(" ");

        let record = LogRecord {
            severity: LogSeverity::Script,
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

    fn eval_index(
        &self,
        object: Value,
        index: Value,
    ) -> Result<Value, String> {
        match object {
            Value::Array(values) => {
                let index = index.as_index()?;
                let borrowed = values.borrow();

                borrowed.elements
                    .get(index)
                    .cloned()
                    .ok_or_else(|| {
                        format!(
                            "array index {} is out of bounds",
                            index
                        )
                    })
            }

            Value::Map(map) => {
                let key = index.as_map_key()?;
                let borrowed = map.borrow();

                Ok(borrowed.get(&key)
                    .cloned()
                    .unwrap_or(Value::Nil))
            }

            other => Err(format!(
                "cannot index {}",
                other.type_name()
            )),
        }
    }

    fn apply_binary(
        &self,
        operator: BinaryOperator,
        left: Value,
        right: Value,
    ) -> Result<Value, String> {
        match operator {
            BinaryOperator::Add => match (left, right) {
                (Value::Number(a), Value::Number(b)) => {
                    Ok(Value::Number(a + b))
                }

                (Value::String(a), Value::String(b)) => {
                    Ok(Value::String(format!("{}{}", a, b)))
                }

                (Value::String(a), other) => {
                    Ok(Value::String(format!("{}{}", a, other.display_string())))
                }

                (other, Value::String(b)) => {
                    Ok(Value::String(format!("{}{}", other.display_string(), b)))
                }

                (left, right) => Err(format!(
                    "cannot add {} and {}",
                    left.type_name(),
                    right.type_name()
                )),
            },

            BinaryOperator::Subtract => {
                let left = left.as_number()?;
                let right = right.as_number()?;

                Ok(Value::Number(left - right))
            }

            BinaryOperator::Multiply => {
                let left = left.as_number()?;
                let right = right.as_number()?;

                Ok(Value::Number(left * right))
            }

            BinaryOperator::Divide => {
                let left = left.as_number()?;
                let right = right.as_number()?;

                if right == 0.0 {
                    return Err(
                        "division by zero.".to_string()
                    );
                }

                Ok(Value::Number(left / right))
            }

            BinaryOperator::Modulo => {
                let left = left.as_number()?;
                let right = right.as_number()?;

                if right == 0.0 {
                    return Err(
                        "modulo by zero.".to_string()
                    );
                }

                Ok(Value::Number(left % right))
            }

            BinaryOperator::Equal => {
                Ok(Value::Bool(left == right))
            }

            BinaryOperator::NotEqual => {
                Ok(Value::Bool(left != right))
            }

            BinaryOperator::Less => {
                let left = left.as_number()?;
                let right = right.as_number()?;

                Ok(Value::Bool(left < right))
            }

            BinaryOperator::LessEqual => {
                let left = left.as_number()?;
                let right = right.as_number()?;

                Ok(Value::Bool(left <= right))
            }

            BinaryOperator::Greater => {
                let left = left.as_number()?;
                let right = right.as_number()?;

                Ok(Value::Bool(left > right))
            }

            BinaryOperator::GreaterEqual => {
                let left = left.as_number()?;
                let right = right.as_number()?;

                Ok(Value::Bool(left >= right))
            }

            BinaryOperator::And | BinaryOperator::Or => {
                unreachable!(
                    "logical operators are handled with short-circuit evaluation"
                )
            }
        }
    }

    fn resolve_identifier(
        &self,
        instance: &ScriptInstance,
        scopes: &[Scope],
        name: &str,
    ) -> Result<Value, String> {
        if name == "math" || name == "basket" || name == "string" {
            return Ok(Value::String(name.to_string()));
        }
        for scope in scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }

        if let Some(value) = instance.fields.get(name) {
            return Ok(value.clone());
        }

        Err(format!("unknown variable '{}'", name))
    }

    fn find_entity(&self, name: &str) -> Option<EntityDecl> {
        self.program
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Entity(entity) if entity.name == name => {
                    Some(entity.clone())
                }

                _ => None,
            })
    }

    fn find_function(
        &self,
        entity_name: &str,
        function_name: &str,
    ) -> Option<FunctionDecl> {
        if let Some(entity) = self.find_entity(entity_name) {
            if let Some(function) =
                entity.members.into_iter().find_map(
                    |member| match member {
                        EntityMember::Function(function)
                            if function.name == function_name =>
                        {
                            Some(function)
                        }

                        _ => None,
                    },
                )
            {
                return Some(function);
            }
        }

        self.program
            .declarations
            .iter()
            .find_map(|declaration| match declaration {
                Declaration::Function(function)
                    if function.name == function_name =>
                {
                    Some(function.clone())
                }

                _ => None,
            })
    }
}

trait ScopeExt {
    fn set_or_declare(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<(), String>;
}

impl ScopeExt for Scope {
    fn set_or_declare(
        &mut self,
        name: &str,
        value: Value,
    ) -> Result<(), String> {
        if self.contains(name) {
            self.set(name, value)
        } else {
            self.declare(
                name.to_string(),
                value,
                false,
            )
        }
    }
}

/// Compiles one function body into a resumable execution plan.
fn compile_function(function: &FunctionDecl) -> CompiledFunction {
    let mut compiler = FunctionCompiler {
        instructions: Vec::new(),
    };

    compiler.compile_block(&function.body);

    CompiledFunction {
        instructions: compiler.instructions,
    }
}

/// Compiles one event body into a resumable execution plan.
fn compile_event(event: &EventDecl) -> CompiledFunction {
    let mut compiler = FunctionCompiler {
        instructions: Vec::new(),
    };

    compiler.compile_block(&event.body);

    CompiledFunction {
        instructions: compiler.instructions,
    }
}

struct FunctionCompiler {
    instructions: Vec<Instruction>,
}

impl FunctionCompiler {
    fn emit(&mut self, instruction: Instruction) -> usize {
        let index = self.instructions.len();

        self.instructions.push(instruction);

        index
    }

    fn compile_block(&mut self, block: &Block) {
        self.emit(Instruction::EnterScope);

        for statement in &block.statements {
            self.compile_statement(statement);
        }

        self.emit(Instruction::ExitScope);
    }

    fn compile_statement(&mut self, statement: &Statement) {
        match &statement.kind {
            StatementKind::Variable {
                name,
                initializer,
                is_const,
                ..
            } => {
                self.emit(Instruction::Variable {
                    name: name.clone(),
                    initializer: initializer.clone(),
                    is_const: *is_const,
                });
            }

            StatementKind::Assignment {
                target,
                operator,
                value,
            } => {
                self.emit(Instruction::Assignment {
                    target: target.clone(),
                    operator: *operator,
                    value: value.clone(),
                });
            }

            StatementKind::If {
                condition,
                then_block,
                else_if,
                else_block,
            } => {
                self.compile_if(
                    condition,
                    then_block,
                    else_if,
                    else_block,
                );
            }

            StatementKind::While { condition, body } => {
                self.compile_while(condition, body);
            }

            StatementKind::For {
                name,
                iterable,
                body,
            } => {
                self.compile_for(name, iterable, body);
            }

            StatementKind::Return(value) => {
                self.emit(Instruction::Return(value.clone()));
            }

            StatementKind::Expression(expression) => {
                if let Some(arguments) = standalone_wait_arguments(expression)
                {
                    self.emit(Instruction::Wait {
                        arguments,
                    });
                } else if let Some((name, arguments)) = standalone_call(expression) {
                    self.emit(Instruction::CallUserFunction {
                        name,
                        arguments,
                    });
                } else {
                    self.emit(Instruction::Evaluate(
                        expression.clone(),
                    ));
                }
            }
        }
    }

    fn compile_if(
        &mut self,
        condition: &Expression,
        then_block: &Block,
        else_if: &[(Expression, Block)],
        else_block: &Option<Block>,
    ) {
        let first_condition_jump = self.emit(
            Instruction::JumpIfFalse {
                condition: condition.clone(),
                target: 0,
            },
        );

        self.compile_block(then_block);

        let mut end_jumps = Vec::new();

        end_jumps.push(self.emit(Instruction::Jump { target: 0 }));

        let mut next_condition =
            self.instructions.len();

        self.patch_jump_if_false(
            first_condition_jump,
            next_condition,
        );

        for (condition, block) in else_if {
            let condition_jump = self.emit(
                Instruction::JumpIfFalse {
                    condition: condition.clone(),
                    target: 0,
                },
            );

            self.compile_block(block);

            end_jumps.push(
                self.emit(Instruction::Jump { target: 0 }),
            );

            next_condition = self.instructions.len();

            self.patch_jump_if_false(
                condition_jump,
                next_condition,
            );
        }

        if let Some(block) = else_block {
            self.compile_block(block);
        }

        let end = self.instructions.len();

        for jump in end_jumps {
            self.patch_jump(jump, end);
        }
    }

    fn compile_while(
        &mut self,
        condition: &Expression,
        body: &Block,
    ) {
        let loop_start = self.instructions.len();

        let condition_jump = self.emit(
            Instruction::JumpIfFalse {
                condition: condition.clone(),
                target: 0,
            },
        );

        self.compile_block(body);

        self.emit(Instruction::Jump {
            target: loop_start,
        });

        let end = self.instructions.len();

        self.patch_jump_if_false(
            condition_jump,
            end,
        );
    }

    fn compile_for(
        &mut self,
        name: &str,
        iterable: &Expression,
        body: &Block,
    ) {
        // The loop-variable scope remains alive across all iterations.
        self.emit(Instruction::EnterScope);

        let init = self.emit(
            Instruction::ForInit {
                name: name.to_string(),
                iterable: iterable.clone(),
                end: 0,
            },
        );

        let body_start = self.instructions.len();

        self.compile_block(body);

        let next = self.emit(
            Instruction::ForNext {
                body_start,
                end: 0,
            },
        );

        let exit_scope = self.emit(Instruction::ExitScope);

        self.patch_for_init(init, exit_scope);

        self.patch_for_next(next, exit_scope);
    }

    fn patch_jump(
        &mut self,
        index: usize,
        target: usize,
    ) {
        match &mut self.instructions[index] {
            Instruction::Jump {
                target: jump_target,
            } => {
                *jump_target = target;
            }

            _ => {
                panic!(
                    "AeoScript compiler attempted to patch a non-jump instruction"
                );
            }
        }
    }

    fn patch_jump_if_false(
        &mut self,
        index: usize,
        target: usize,
    ) {
        match &mut self.instructions[index] {
            Instruction::JumpIfFalse {
                target: jump_target,
                ..
            } => {
                *jump_target = target;
            }

            _ => {
                panic!(
                    "AeoScript compiler attempted to patch a non-conditional jump"
                );
            }
        }
    }

    fn patch_for_init(
        &mut self,
        index: usize,
        target: usize,
    ) {
        match &mut self.instructions[index] {
            Instruction::ForInit {
                end,
                ..
            } => {
                *end = target;
            }

            _ => {
                panic!(
                    "AeoScript compiler attempted to patch a non-for-init instruction"
                );
            }
        }
    }

    fn patch_for_next(
        &mut self,
        index: usize,
        target: usize,
    ) {
        match &mut self.instructions[index] {
            Instruction::ForNext {
                end,
                ..
            } => {
                *end = target;
            }

            _ => {
                panic!(
                    "AeoScript compiler attempted to patch a non-for-next instruction"
                );
            }
        }
    }
}

fn standalone_call(
    expression: &Expression,
) -> Option<(String, Vec<Expression>)> {
    match &expression.kind {
        ExpressionKind::Call {
            callee,
            arguments,
        } => match &callee.kind {
            ExpressionKind::Identifier(name) => {
                Some((name.clone(), arguments.clone()))
            }

            _ => None,
        },

        _ => None,
    }
}

fn standalone_wait_arguments(
    expression: &Expression,
) -> Option<Vec<Expression>> {
    match &expression.kind {
        ExpressionKind::Call {
            callee,
            arguments,
        } => match &callee.kind {
            ExpressionKind::Identifier(name)
                if name == "wait" =>
            {
                Some(arguments.clone())
            }

            _ => None,
        },

        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scripting::execution::{
        ScriptScheduler,
        ScriptTaskState,
    };
    use crate::scripting::lexer::Lexer;
    use crate::scripting::parser::Parser;
    use crate::scripting::value::{
        HandleKind,
        Value,
    };
    use crate::scripting::api::EngineHost;
    use crate::engine::entity::{EntityManager, EntityManager as _};

    fn interpreter(source: &str) -> Interpreter {
        let tokens = Lexer::new(source)
            .tokenize()
            .expect("lexer should succeed");

        let program = Parser::new(tokens)
            .parse()
            .expect("parser should succeed");

        Interpreter::new(program)
    }

    fn test_host() -> EntityManager {
        EntityManager::new()
    }

    fn run(
        source: &str,
    ) -> (Interpreter, ScriptInstance, Value) {
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let mut instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let result = interpreter
            .call(
                &mut instance,
                "update",
                vec![Value::Number(1.0)],
                &mut host
            )
            .expect("update should execute");

        (interpreter, instance, result)
    }

    fn drive_fiber_once(
        interpreter: &mut Interpreter,
        scheduler: &mut ScriptScheduler,
        task_id: crate::scripting::execution::ScriptTaskId,
        fiber: &mut ScriptFiber,
    ) -> FiberResult {
        let ready = scheduler
            .pop_ready()
            .expect("task should be ready");

        assert_eq!(ready, task_id);

        scheduler
            .begin_running(task_id)
            .expect("task should start");

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let result =
            interpreter.resume_fiber(fiber, &mut host);

        scheduler
            .apply_result(
                task_id,
                result.clone(),
            )
            .expect("scheduler should accept fiber result");

        result
    }

    #[test]
    fn executes_field_arithmetic() {
        let source = r#"
entity Test {

    value: number = 10

    fn update(dt: number) {
        value += dt
    }
}
"#;

        let (_interpreter, instance, _result) =
            run(source);

        assert_eq!(
            instance.get_field("value"),
            Some(&Value::Number(11.0))
        );
    }

    #[test]
    fn executes_local_variables() {
        let source = r#"
entity Test {

    value: number = 0

    fn update(dt: number) {
        amount: number = 10
        amount += dt
        value = amount
    }
}
"#;

        let (_interpreter, instance, _result) =
            run(source);

        assert_eq!(
            instance.get_field("value"),
            Some(&Value::Number(11.0))
        );
    }

    #[test]
    fn executes_if_else() {
        let source = r#"
entity Test {

    value: number = 0

    fn update(dt: number) {
        if dt > 0 {
            value = 1
        } else {
            value = 2
        }
    }
}
"#;

        let (_interpreter, instance, _result) =
            run(source);

        assert_eq!(
            instance.get_field("value"),
            Some(&Value::Number(1.0))
        );
    }

    #[test]
    fn executes_while_loop() {
        let source = r#"
entity Test {

    value: number = 0

    fn update(dt: number) {
        counter: number = 0

        while counter < 5 {
            counter += 1
        }

        value = counter
    }
}
"#;

        let (_interpreter, instance, _result) =
            run(source);

        assert_eq!(
            instance.get_field("value"),
            Some(&Value::Number(5.0))
        );
    }

    #[test]
    fn executes_for_loop_over_array() {
        let source = r#"
entity Test {

    value: number = 0

    fn update(dt: number) {
        for item in [1, 2, 3, 4] {
            value += item
        }
    }
}
"#;

        let (_interpreter, instance, _result) =
            run(source);

        assert_eq!(
            instance.get_field("value"),
            Some(&Value::Number(10.0))
        );
    }

    #[test]
    fn executes_return_values() {
        let source = r#"
entity Test {

    value: number = 0

    fn add(a: number, b: number): number {
        return a + b
    }

    fn update(dt: number) {
        value = add(4, 7)
    }
}
"#;

        let (_interpreter, instance, _result) =
            run(source);

        assert_eq!(
            instance.get_field("value"),
            Some(&Value::Number(11.0))
        );
    }

    #[test]
    fn debug_log_captures_output() {
        let source = r#"
entity Test {

    value: number = 42

    fn update(dt: number) {
        debug.log(value)
        debug.log("hello", value)
    }
}
"#;

        let (interpreter, _instance, _result) =
            run(source);

        assert_eq!(
            interpreter.output()[0].message,
            "42".to_string()
        );
        assert_eq!(
            interpreter.output()[1].message,
            "hello 42".to_string()
        );
    }

    #[test]
    fn nil_can_be_used_as_an_optional_condition() {
        let source = r#"
entity Test {

    value: number = 0
    light: Light?

    fn update(dt: number) {
        if light {
            value = 1
        } else {
            value = 2
        }
    }
}
"#;

        let (_interpreter, instance, _result) =
            run(source);

        assert_eq!(
            instance.get_field("value"),
            Some(&Value::Number(2.0))
        );
    }

    #[test]
    fn handles_are_truthy() {
        let value = Value::Handle {
            kind: HandleKind::Light,
            id: 25,
        };

        assert_eq!(
            value.is_truthy()
                .expect("handle should be truthy"),
            true
        );
    }

    #[test]
    fn operation_budget_stops_infinite_loops() {
        let source = r#"
entity Test {

    fn update(dt: number) {
        while true {
        }
    }
}
"#;

        let tokens = Lexer::new(source)
            .tokenize()
            .expect("lexer should succeed");

        let program = Parser::new(tokens)
            .parse()
            .expect("parser should succeed");

        let mut interpreter =
            Interpreter::with_limits(
                program,
                50,
                16,
            );

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let mut instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let result = interpreter.call(
            &mut instance,
            "update",
            vec![Value::Number(1.0)],
            &mut host
        );

        assert!(result.is_err());

        assert!(
            result
                .expect_err("budget should fail")
                .contains(
                    "execution budget exhausted"
                )
        );
    }

    #[test]
    fn call_depth_is_limited() {
        let source = r#"
entity Test {

    fn recurse(value: number): number {
        return recurse(value + 1)
    }

    fn update(dt: number) {
        recurse(0)
    }
}
"#;

        let tokens = Lexer::new(source)
            .tokenize()
            .expect("lexer should succeed");

        let program = Parser::new(tokens)
            .parse()
            .expect("parser should succeed");

        let mut interpreter =
            Interpreter::with_limits(
                program,
                100_000,
                4,
            );

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let mut instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let result = interpreter.call(
            &mut instance,
            "update",
            vec![Value::Number(1.0)],
            &mut host,
        );

        assert!(result.is_err());

        assert!(
            result
                .expect_err(
                    "call depth should fail"
                )
                .contains(
                    "call depth exceeded"
                )
        );
    }

    #[test]
    fn fiber_resumes_after_first_wait() {
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

        let mut interpreter =
            interpreter(source);

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let mut fiber =
            interpreter
                .start_fiber(
                    instance,
                    "update",
                    vec![Value::Number(1.0)],
                )
                .expect("fiber should start");

        let mut scheduler =
            ScriptScheduler::new();

        let task_id = scheduler.spawn();

        let result = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            result,
            FiberResult::Yield(
                YieldReason::WaitSeconds(1.0)
            )
        );

        assert_eq!(
            fiber.instance()
                .get_field("value"),
            Some(&Value::Number(1.0))
        );

        assert_eq!(
            fiber.program_counter(),
            3
        );

        scheduler
            .tick(0.99)
            .expect("time should advance");

        assert_eq!(
            scheduler.state(task_id),
            Some(
                ScriptTaskState::Waiting {
                    wake_at: 1.0
                }
            )
        );

        scheduler
            .tick(1.0)
            .expect("time should advance");

        let result = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            result,
            FiberResult::Complete
        );

        assert_eq!(
            fiber.instance()
                .get_field("value"),
            Some(&Value::Number(2.0))
        );
    }

    #[test]
    fn fiber_resumes_two_waits_in_order() {
        let source = r#"
entity Test {

    value: number = 0

    fn update(dt: number) {
        value = 1
        wait(1)
        value = 2
        wait(1)
        value = 3
    }
}
"#;

        let mut interpreter =
            interpreter(source);

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let mut fiber =
            interpreter
                .start_fiber(
                    instance,
                    "update",
                    vec![Value::Number(1.0)],
                )
                .expect("fiber should start");

        let mut scheduler =
            ScriptScheduler::new();

        let task_id = scheduler.spawn();

        let result = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            result,
            FiberResult::Yield(
                YieldReason::WaitSeconds(1.0)
            )
        );

        assert_eq!(
            fiber.instance()
                .get_field("value"),
            Some(&Value::Number(1.0))
        );

        scheduler
            .tick(1.0)
            .expect("time should advance");

        let result = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            result,
            FiberResult::Yield(
                YieldReason::WaitSeconds(1.0)
            )
        );

        assert_eq!(
            fiber.instance()
                .get_field("value"),
            Some(&Value::Number(2.0))
        );

        scheduler
            .tick(1.99)
            .expect("time should advance");

        assert_eq!(
            scheduler.state(task_id),
            Some(
                ScriptTaskState::Waiting {
                    wake_at: 2.0
                }
            )
        );

        scheduler
            .tick(2.0)
            .expect("time should advance");

        let result = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            result,
            FiberResult::Complete
        );

        assert_eq!(
            fiber.instance()
                .get_field("value"),
            Some(&Value::Number(3.0))
        );
    }

    #[test]
    fn fiber_wait_inside_if_resumes_inside_if() {
        let source = r#"
entity Test {

    value: number = 0

    fn update(dt: number) {
        if true {
            value = 1
            wait(1)
            value = 2
        }
    }
}
"#;

        let mut interpreter =
            interpreter(source);

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let mut fiber =
            interpreter
                .start_fiber(
                    instance,
                    "update",
                    vec![Value::Number(1.0)],
                )
                .expect("fiber should start");

        let mut scheduler =
            ScriptScheduler::new();

        let task_id = scheduler.spawn();

        let result = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            result,
            FiberResult::Yield(
                YieldReason::WaitSeconds(1.0)
            )
        );

        assert_eq!(
            fiber.instance()
                .get_field("value"),
            Some(&Value::Number(1.0))
        );

        scheduler
            .tick(1.0)
            .expect("time should advance");

        let result = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            result,
            FiberResult::Complete
        );

        assert_eq!(
            fiber.instance()
                .get_field("value"),
            Some(&Value::Number(2.0))
        );
    }

    #[test]
    fn fiber_wait_inside_while_preserves_loop_position() {
        let source = r#"
entity Test {

    counter: number = 0

    fn update(dt: number) {
        while true {
            counter += 1
            wait(1)
        }
    }
}
"#;

        let mut interpreter =
            interpreter(source);

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let mut fiber =
            interpreter
                .start_fiber(
                    instance,
                    "update",
                    vec![Value::Number(1.0)],
                )
                .expect("fiber should start");

        let mut scheduler =
            ScriptScheduler::new();

        let task_id = scheduler.spawn();

        let result = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            result,
            FiberResult::Yield(
                YieldReason::WaitSeconds(1.0)
            )
        );

        assert_eq!(
            fiber.instance()
                .get_field("counter"),
            Some(&Value::Number(1.0))
        );

        scheduler
            .tick(1.0)
            .expect("time should advance");

        let result = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            result,
            FiberResult::Yield(
                YieldReason::WaitSeconds(1.0)
            )
        );

        assert_eq!(
            fiber.instance()
                .get_field("counter"),
            Some(&Value::Number(2.0))
        );
    }

    #[test]
    fn fiber_budget_resets_between_cooperative_resumes() {
        let source = r#"
entity Test {

    counter: number = 0

    fn update(dt: number) {
        while true {
            counter += 1
            wait(1)
        }
    }
}
"#;

        let tokens = Lexer::new(source)
            .tokenize()
            .expect("lexer should succeed");

        let program = Parser::new(tokens)
            .parse()
            .expect("parser should succeed");

        let mut interpreter =
            Interpreter::with_limits(
                program,
                100,
                16,
            );

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let mut fiber =
            interpreter
                .start_fiber(
                    instance,
                    "update",
                    vec![Value::Number(1.0)],
                )
                .expect("fiber should start");

        let mut scheduler =
            ScriptScheduler::new();

        let task_id = scheduler.spawn();

        let first = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            first,
            FiberResult::Yield(
                YieldReason::WaitSeconds(1.0)
            )
        );

        scheduler
            .tick(1.0)
            .expect("time should advance");

        let second = drive_fiber_once(
            &mut interpreter,
            &mut scheduler,
            task_id,
            &mut fiber,
        );

        assert_eq!(
            second,
            FiberResult::Yield(
                YieldReason::WaitSeconds(1.0)
            )
        );

        assert_eq!(
            fiber.instance()
                .get_field("counter"),
            Some(&Value::Number(2.0))
        );
    }

    #[test]
    fn fiber_rejects_negative_wait() {
        let source = r#"
entity Test {

    fn update(dt: number) {
        wait(-1)
    }
}
"#;

        let mut interpreter =
            interpreter(source);

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let mut fiber =
            interpreter
                .start_fiber(
                    instance,
                    "update",
                    vec![Value::Number(1.0)],
                )
                .expect("fiber should start");

        let result =
            interpreter.resume_fiber(&mut fiber, &mut host);

        assert!(matches!(
            result,
            FiberResult::Failed(message)
                if message.contains(
                    "greater than zero"
                )
        ));

        assert!(fiber.is_finished());
    }

    #[test]
    fn fiber_rejects_non_finite_wait() {
        let source = r#"
entity Test {

    fn update(dt: number) {
        wait(dt)
    }
}
"#;

        let mut interpreter =
            interpreter(source);

        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };

        let instance = interpreter
            .instantiate_entity("Test", 1, &mut host)
            .expect("entity should instantiate");

        let mut fiber =
            interpreter
                .start_fiber(
                    instance,
                    "update",
                    vec![Value::Number(
                        f64::INFINITY
                    )],
                )
                .expect("fiber should start");

        let result =
            interpreter.resume_fiber(&mut fiber, &mut host);

        assert!(matches!(
            result,
            FiberResult::Failed(message)
                if message.contains(
                    "must be finite"
                )
        ));

        assert!(fiber.is_finished());
    }
    #[test]
    fn basket_len_property_works() {
        let source = r#"
entity Test {
    count: number = 0
    fn main() {
        b: basket = [1, 2, 3]
        count = b.len()
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();
        assert_eq!(instance.get_field("count"), Some(&Value::Number(3.0)));
    }

    #[test]
    fn method_call_syntax_works() {
        let source = r#"
entity Test {
    fn main(e: Entity) {
        e:set_position(10, 20, 30)
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let id = em.create_entity("Other");
        let handle = Value::Handle { kind: HandleKind::Entity, id: id.0 };

        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();

        interpreter.call(&mut instance, "main", vec![handle], &mut host).unwrap();
        assert_eq!(em.get_position(id), Some(glam::Vec3::new(10.0, 20.0, 30.0)));
    }

    #[test]
    fn get_parent_as_global_works() {
        let source = r#"
entity Test {
    parent_name: string = ""
    fn main() {
        p: Entity? = get_parent()
        if p != nil {
            parent_name = p.name
        }
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();

        struct ParentHost {
            em: EntityManager,
        }
        impl EngineHost for ParentHost {
            fn entity_manager(&self) -> &EntityManager { &self.em }
            fn get_position(&self, id: u64) -> Option<glam::Vec3> { self.em.get_position(crate::engine::entity::EntityId(id)) }
            fn set_position(&mut self, id: u64, pos: glam::Vec3) { self.em.set_position(crate::engine::entity::EntityId(id), pos); }
            fn lookup_light(&self, _: i32, _: i32, _: i32) -> Option<u64> { None }
            fn is_light_enabled(&self, _: u64) -> Option<bool> { None }
            fn set_light_enabled(&mut self, _: u64, _: bool) {}
            fn get_all_cells_of_class(&self, _: &str) -> Vec<u64> { vec![] }
            fn find_objects(&self, _: &str) -> Vec<(HandleKind, u64)> { vec![] }
            fn get_children(&self, _: HandleKind, _: u64) -> Vec<(HandleKind, u64)> { vec![] }
            fn get_parent(&self, kind: HandleKind, id: u64) -> Option<(HandleKind, u64)> {
                if kind == HandleKind::Entity && id == 2 {
                    Some((HandleKind::Entity, 1))
                } else {
                    None
                }
            }
            fn get_cell_object(&self, _: u64) -> Option<(HandleKind, u64)> { None }
            fn get_property(&self, kind: HandleKind, id: u64, name: &str) -> Result<Option<Value>, String> {
                if kind == HandleKind::Entity && id == 1 && name == "name" {
                    Ok(Some(Value::String("Parent".to_string())))
                } else {
                    Ok(None)
                }
            }
            fn set_property(&mut self, _: HandleKind, _: u64, _: &str, _: Value) -> Result<(), String> { Ok(()) }
            fn call_method(&mut self, _: HandleKind, _: u64, _: &str, _: &[Value]) -> Result<Option<Value>, String> { Ok(None) }
        }

        let mut ph = ParentHost { em: test_host() };
        ph.em.create_entity("Parent"); // id 1
        ph.em.create_entity("Test");   // id 2

        let mut host = HostContext { delta_time: 1.0, engine: &mut ph };
        let mut instance = interpreter.instantiate_entity("Test", 2, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();
        assert_eq!(instance.get_field("parent_name"), Some(&Value::String("Parent".to_string())));
    }

    #[test]
    fn test_string_concatenation() {
        let source = r#"
entity Test {
    r1: string = ""
    r2: string = ""
    r3: string = ""
    r4: string = ""
    r5: string = ""

    fn main() {
        r1 = "count = " + 5
        r2 = "enabled = " + true
        r3 = "value = " + nil
        r4 = 10 + " items"
        r5 = "basket = " + [1, 2]
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();

        assert_eq!(instance.get_field("r1"), Some(&Value::String("count = 5".to_string())));
        assert_eq!(instance.get_field("r2"), Some(&Value::String("enabled = true".to_string())));
        assert_eq!(instance.get_field("r3"), Some(&Value::String("value = nil".to_string())));
        assert_eq!(instance.get_field("r4"), Some(&Value::String("10 items".to_string())));
        assert_eq!(instance.get_field("r5"), Some(&Value::String("basket = [1, 2]".to_string())));
    }

    #[test]
    fn stdlib_math_works() {
        let source = r#"
entity Test {
    r1: number = 0
    r2: number = 0
    r3: number = 0
    r4: number = 0
    r5: number = 0

    fn main() {
        r1 = math.abs(-10.5)
        r2 = math.max(5, 10)
        r3 = math.clamp(15, 0, 10)
        r4 = math.floor(3.7)
        r5 = math.lerp(10, 20, 0.5)
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();

        assert_eq!(instance.get_field("r1"), Some(&Value::Number(10.5)));
        assert_eq!(instance.get_field("r2"), Some(&Value::Number(10.0)));
        assert_eq!(instance.get_field("r3"), Some(&Value::Number(10.0)));
        assert_eq!(instance.get_field("r4"), Some(&Value::Number(3.0)));
        assert_eq!(instance.get_field("r5"), Some(&Value::Number(15.0)));
    }

    #[test]
    fn stdlib_basket_works() {
        let source = r#"
entity Test {
    r1: number = 0
    r2: string = ""
    r3: number = 0
    r4: number = 0

    fn main() {
        b: basket = [3, 1, 2]
        b.sort()
        r1 = b[0]
        r2 = b.concat("-")

        b2: basket = basket.create(3, 5)
        r3 = b2[2]

        b.insert(0, 10)
        r4 = b[0]
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();

        assert_eq!(instance.get_field("r1"), Some(&Value::Number(1.0)));
        assert_eq!(instance.get_field("r2"), Some(&Value::String("1-2-3".to_string())));
        assert_eq!(instance.get_field("r3"), Some(&Value::Number(5.0)));
        assert_eq!(instance.get_field("r4"), Some(&Value::Number(10.0)));
    }

    #[test]
    fn stdlib_basket_move_works() {
        let source = r#"
entity Test {
    r1: string = ""
    r2: string = ""

    fn main() {
        b: basket = [0, 1, 2, 3]
        // Copy [1, 2] to start at 3
        basket.move(b, 1, 2, 3)
        r1 = b.concat(",")

        b2: basket = [10, 20]
        // Copy [1, 2] from b to b2 at 1
        basket.move(b, 1, 2, 1, b2)
        r2 = b2.concat(",")
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();

        assert_eq!(instance.get_field("r1"), Some(&Value::String("0,1,2,1,2".to_string())));
        assert_eq!(instance.get_field("r2"), Some(&Value::String("10,1,2".to_string())));
    }

    #[test]
    fn stdlib_string_works() {
        let source = r#"
entity Test {
    r1: number = 0
    r2: string = ""
    r3: string = ""
    r4: string = ""

    fn main() {
        r1 = string.len("hello")
        r2 = string.upper("world")
        r3 = string.reverse("abc")
        s: basket = string.split("a,b,c", ",")
        r4 = s.concat("|")
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();

        assert_eq!(instance.get_field("r1"), Some(&Value::Number(5.0)));
        assert_eq!(instance.get_field("r2"), Some(&Value::String("WORLD".to_string())));
        assert_eq!(instance.get_field("r3"), Some(&Value::String("cba".to_string())));
        assert_eq!(instance.get_field("r4"), Some(&Value::String("a|b|c".to_string())));
    }

    #[test]
    fn frozen_basket_mutation_fails() {
        let source = r#"
entity Test {
    fn main() {
        b: basket = [1, 2, 3]
        b.freeze()
        b[0] = 10
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        let res = interpreter.call(&mut instance, "main", vec![], &mut host);
        assert!(res.is_err());
        assert!(res.unwrap_err().to_lowercase().contains("frozen"));
    }

    #[test]
    fn map_read_missing_key_returns_nil() {
        let source = r#"
entity Test {
    r1: bool = false
    r2: bool = false
    r3: bool = false
    r4: bool = false
    r5: bool = false

    fn main() {
        const values = {}
        r1 = (values["missing"] == nil)
        r2 = (values[123] == nil)
        r3 = (values["1"] == nil)

        values[1] = "numeric"
        r4 = (values["1"] == nil)
        r5 = (values[1] == "numeric")
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();

        assert_eq!(instance.get_field("r1"), Some(&Value::Bool(true)));
        assert_eq!(instance.get_field("r2"), Some(&Value::Bool(true)));
        assert_eq!(instance.get_field("r3"), Some(&Value::Bool(true)));
        assert_eq!(instance.get_field("r4"), Some(&Value::Bool(true)));
        assert_eq!(instance.get_field("r5"), Some(&Value::Bool(true)));
    }

    #[test]
    fn map_deletion_semantics() {
        let source = r#"
entity Test {
    r1: bool = false

    fn main() {
        const values = {}
        values["name"] = "test"
        values["name"] = nil
        r1 = (values["name"] == nil)
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();

        assert_eq!(instance.get_field("r1"), Some(&Value::Bool(true)));
    }

    #[test]
    fn nested_map_read_missing_key() {
        let source = r#"
entity Test {
    r1: bool = false

    fn main() {
        const outer = {}
        outer["inner"] = {}
        r1 = (outer["inner"]["missing"] == nil)
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();

        assert_eq!(instance.get_field("r1"), Some(&Value::Bool(true)));
    }

    #[test]
    fn unicode_string_literals_work() {
        let source = r#"
entity Test {
    r1: number = 0
    r2: number = 0
    r3: number = 0
    r4: string = ""
    r5: string = ""

    fn main() {
        r1 = string.len("é")
        r2 = string.len("你好")
        r3 = string.len("😀")
        r4 = string.reverse("é")

        s: basket = string.split("é")
        r5 = s[0]
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let mut instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        interpreter.call(&mut instance, "main", vec![], &mut host).unwrap();

        assert_eq!(instance.get_field("r1"), Some(&Value::Number(1.0)));
        assert_eq!(instance.get_field("r2"), Some(&Value::Number(2.0)));
        assert_eq!(instance.get_field("r3"), Some(&Value::Number(1.0)));
        assert_eq!(instance.get_field("r4"), Some(&Value::String("é".to_string())));
        assert_eq!(instance.get_field("r5"), Some(&Value::String("é".to_string())));
    }

    #[test]
    fn wait_inside_called_function_works() {
        let source = r#"
entity Test {
    value: number = 0
    fn sub() {
        wait(0.1)
        value = 1
    }
    fn main() {
        sub()
        value = 2
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();

        let mut fiber = interpreter.start_fiber(instance, "main", vec![]).unwrap();

        // 1. Initial run: enters main, calls sub, sub calls wait and yields.
        let result = interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(result, FiberResult::Yield(YieldReason::WaitSeconds(0.1)));
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(0.0)));

        // 2. Resume after wait: sub finishes (sets value=1), returns to main, main sets value=2 and completes.
        let result = interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(result, FiberResult::Complete);
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(2.0)));
    }

    #[test]
    fn wait_preserves_locals() {
        let source = r#"
entity Test {
    value: number = 0
    fn main() {
        local_val: number = 42
        wait(0.1)
        value = local_val
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        let mut fiber = interpreter.start_fiber(instance, "main", vec![]).unwrap();

        interpreter.resume_fiber(&mut fiber, &mut host);
        interpreter.resume_fiber(&mut fiber, &mut host);

        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(42.0)));
    }

    #[test]
    fn wait_inside_loop_preserves_state() {
        let source = r#"
entity Test {
    value: number = 0
    fn main() {
        for i in [1, 2, 3] {
            value += i
            wait(0.1)
        }
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        let mut fiber = interpreter.start_fiber(instance, "main", vec![]).unwrap();

        // Iteration 1
        interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(1.0)));

        // Iteration 2
        interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(3.0)));

        // Iteration 3
        interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(6.0)));

        let res = interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(res, FiberResult::Complete);
    }

    #[test]
    fn nested_calls_with_wait() {
        let source = r#"
entity Test {
    value: number = 0
    fn inner() {
        wait(0.1)
        value += 1
    }
    fn middle() {
        inner()
        wait(0.1)
        value += 10
    }
    fn main() {
        middle()
        value += 100
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();
        let mut fiber = interpreter.start_fiber(instance, "main", vec![]).unwrap();

        // Enters main -> middle -> inner -> wait(0.1)
        interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(0.0)));

        // Resumes inner: value += 1, returns to middle -> wait(0.1)
        interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(1.0)));

        // Resumes middle: value += 10, returns to main: value += 100, complete
        interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(111.0)));
    }

    #[test]
    fn update_callback_with_wait_logic() {
        let source = r#"
entity Test {
    value: number = 0
    started: bool = false
    fn sub() {
        wait(0.1)
        value = 1
    }
    fn update() {
        if !started {
            started = true
            sub()
        }
    }
}
"#;
        let mut interpreter = interpreter(source);
        let mut em = test_host();
        let mut host = HostContext { delta_time: 1.0, engine: &mut em };
        let instance = interpreter.instantiate_entity("Test", 1, &mut host).unwrap();

        // Simulating the ScriptScene logic:
        // Frame 1: update() starts
        let mut fiber = interpreter.start_fiber(instance, "update", vec![]).unwrap();
        let res = interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(res, FiberResult::Yield(YieldReason::WaitSeconds(0.1)));
        assert_eq!(fiber.instance().get_field("started"), Some(&Value::Bool(true)));
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(0.0)));

        // Frame 2: Scheduler resumes fiber
        let res = interpreter.resume_fiber(&mut fiber, &mut host);
        assert_eq!(res, FiberResult::Complete);
        assert_eq!(fiber.instance().get_field("value"), Some(&Value::Number(1.0)));
    }
}
