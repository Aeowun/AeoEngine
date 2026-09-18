use std::collections::BTreeMap;
use std::sync::Arc;

use super::ast::*;
use super::execution::{FiberResult, YieldReason};
use super::value::{Scope, Value, HandleKind};
use super::api::{HostContext, call_host_function, call_host_member, resolve_host_property, resolve_host_member_property};

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
    fields: BTreeMap<String, Value>,
}

impl ScriptInstance {
    pub fn id(&self) -> u64 {
        self.id
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

/// A cooperative AeoScript execution fiber.
///
/// The fiber owns its runtime execution state but not the interpreter itself.
/// That keeps the interpreter single-threaded while allowing many independent
/// script fibers to exist.
#[derive(Debug)]
pub struct ScriptFiber {
    entity_name: String,
    function_name: String,
    instance: ScriptInstance,
    function: Arc<CompiledFunction>,
    pc: usize,
    scopes: Vec<Scope>,
    for_states: Vec<ForState>,
    finished: bool,
    result: Option<Value>,
}

impl ScriptFiber {
    pub fn entity_name(&self) -> &str {
        &self.entity_name
    }

    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    pub fn program_counter(&self) -> usize {
        self.pc
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
    output: Vec<String>,
    operation_budget: u64,
    operations_remaining: u64,
    max_call_depth: usize,
    call_depth: usize,
    compiled_functions: BTreeMap<(String, String), Arc<CompiledFunction>>,
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
        }
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
        }
    }

    pub fn program(&self) -> &Program {
        &self.program
    }

    pub fn output(&self) -> &[String] {
        &self.output
    }

    pub fn drain_output(&mut self) -> Vec<String> {
        std::mem::take(&mut self.output)
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
            fields: BTreeMap::new(),
        };

        let mut scopes = Vec::new();

        for field in fields {
            self.tick()?;

            let value = if let Some(initializer) = &field.initializer {
                self.eval_expression(&mut instance, &mut scopes, initializer, host)?
            } else {
                Value::Null
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

        let function = self
            .find_function(&instance.entity_name, function_name)
            .ok_or_else(|| {
                format!(
                    "function '{}' does not exist on entity '{}'",
                    function_name, instance.entity_name
                )
            })?;

        self.call_user_function(instance, &function, arguments, host)
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
            entity_name,
            function_name: function_name.to_string(),
            instance,
            function: compiled,
            pc: 0,
            scopes: vec![parameter_scope],
            for_states: Vec::new(),
            finished: false,
            result: None,
        })
    }

    /// Resumes a persistent fiber until it yields, completes, or fails.
    ///
    /// The operation budget is per resume slice. A cooperative wait therefore
    /// allows a long-running script to continue indefinitely while a script
    /// that spins without yielding still hits the execution budget.
    pub fn resume_fiber(&mut self, fiber: &mut ScriptFiber, host: &mut HostContext) -> FiberResult {
        if fiber.finished {
            return FiberResult::Failed(
                "cannot resume a completed AeoScript fiber.".to_string(),
            );
        }

        self.operations_remaining = self.operation_budget;
        self.call_depth = 0;

        loop {
            if fiber.pc >= fiber.function.instructions.len() {
                return fiber.complete(Value::Null);
            }

            if let Err(error) = self.tick() {
                return fiber.fail(error);
            }

            let instruction = fiber.function.instructions[fiber.pc].clone();
            match instruction {
                Instruction::EnterScope => {
                    fiber.scopes.push(Scope::new());

                    fiber.pc += 1;
                }

                Instruction::ExitScope => {
                    if fiber.scopes.len() <= 1 {
                        return fiber.fail(
                            "AeoScript runtime scope underflow.".to_string()
                        );
                    }

                    fiber.scopes.pop();

                    fiber.pc += 1;
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
                                &mut fiber.scopes,
                                &initializer,
                                host,
                            ) {
                                Ok(value) => value,

                                Err(error) => return fiber.fail(error),
                            }
                        }

                        None => Value::Null,
                    };

                    let scope = match fiber.scopes.last_mut() {
                        Some(scope) => scope,

                        None => {
                            return fiber.fail(
                                "AeoScript runtime has no active scope."
                                    .to_string(),
                            )
                        }
                    };

                    if let Err(error) =
                        scope.declare(name.clone(), value, is_const)
                    {
                        return fiber.fail(error);
                    }

                    fiber.pc += 1;
                }

                Instruction::Assignment {
                    target,
                    operator,
                    value,
                } => {
                    let right = match self.eval_expression(
                        &mut fiber.instance,
                        &mut fiber.scopes,
                        &value,
                        host,
                    ) {
                        Ok(value) => value,

                        Err(error) => return fiber.fail(error),
                    };

                    if let Err(error) = self.assign_target(
                        &mut fiber.instance,
                        &mut fiber.scopes,
                        &target,
                        operator,
                        right,
                        host,
                    ) {
                        return fiber.fail(error);
                    }

                    fiber.pc += 1;
                }

                Instruction::Evaluate(expression) => {
                    if let Err(error) = self.eval_expression(
                        &mut fiber.instance,
                        &mut fiber.scopes,
                        &expression,
                        host,
                    ) {
                        return fiber.fail(error);
                    }

                    fiber.pc += 1;
                }

                Instruction::Jump { target } => {
                    fiber.pc = target;
                }

                Instruction::JumpIfFalse {
                    condition,
                    target,
                } => {
                    let value = match self.eval_expression(
                        &mut fiber.instance,
                        &mut fiber.scopes,
                        &condition,
                        host,
                    ) {
                        Ok(value) => value,

                        Err(error) => return fiber.fail(error),
                    };

                    match value.is_truthy() {
                        Ok(true) => fiber.pc += 1,

                        Ok(false) => fiber.pc = target,

                        Err(error) => return fiber.fail(error),
                    }
                }

                Instruction::ForInit {
                    name,
                    iterable,
                    end,
                } => {
                    let iterable_value = match self.eval_expression(
                        &mut fiber.instance,
                        &mut fiber.scopes,
                        &iterable,
                        host,
                    ) {
                        Ok(value) => value,

                        Err(error) => return fiber.fail(error),
                    };

                    let values = match iterable_value {
                        Value::Array(values) => values,

                        other => {
                            return fiber.fail(format!(
                                "cannot iterate over {} in for loop",
                                other.type_name()
                            ))
                        }
                    };

                    if values.is_empty() {
                        fiber.pc = end;
                        continue;
                    }

                    let first_value = values[0].clone();

                    fiber.for_states.push(ForState {
                        name: name.clone(),
                        values,
                        next_index: 1,
                    });

                    let scope = match fiber.scopes.last_mut() {
                        Some(scope) => scope,

                        None => {
                            return fiber.fail(
                                "AeoScript runtime has no active scope."
                                    .to_string(),
                            )
                        }
                    };

                    if let Err(error) =
                        scope.declare(name.clone(), first_value, false)
                    {
                        return fiber.fail(error);
                    }

                    fiber.pc += 1;
                }

                Instruction::ForNext { body_start, end } => {
                    let Some(loop_state) = fiber.for_states.last_mut() else {
                        return fiber.fail(
                            "AeoScript runtime for-loop state underflow."
                                .to_string(),
                        );
                    };

                    if loop_state.next_index < loop_state.values.len() {
                        let value =
                            loop_state.values[loop_state.next_index].clone();

                        loop_state.next_index += 1;

                        let name = loop_state.name.clone();

                        let scope = match fiber.scopes.last_mut() {
                            Some(scope) => scope,

                            None => {
                                return fiber.fail(
                                    "AeoScript runtime has no active scope."
                                        .to_string(),
                                )
                            }
                        };

                        if let Err(error) =
                            scope.set_or_declare(&name, value)
                        {
                            return fiber.fail(error);
                        }

                        fiber.pc = body_start;
                    } else {
                        fiber.for_states.pop();

                        fiber.pc = end;
                    }
                }

                Instruction::Wait { arguments } => {
                    if arguments.len() != 1 {
                        return fiber.fail(
                            "wait() expects exactly one argument.".to_string(),
                        );
                    }

                    let seconds = match self.eval_expression(
                        &mut fiber.instance,
                        &mut fiber.scopes,
                        &arguments[0],
                        host,
                    ) {
                        Ok(value) => match value.as_number() {
                            Ok(value) => value,

                            Err(error) => return fiber.fail(error),
                        },

                        Err(error) => return fiber.fail(error),
                    };

                    if !seconds.is_finite() {
                        return fiber.fail(
                            "AeoScript wait duration must be finite."
                                .to_string(),
                        );
                    }

                    if seconds <= 0.0 {
                        return fiber.fail(
                            "AeoScript wait duration must be greater than zero.".to_string(),
                        );
                    }

                    fiber.pc += 1;

                    return FiberResult::Yield(
                        YieldReason::WaitSeconds(seconds),
                    );
                }

                Instruction::Return(expression) => {
                    let value = match expression {
                        Some(expression) => match self.eval_expression(
                            &mut fiber.instance,
                            &mut fiber.scopes,
                            &expression,
                            host,
                        ) {
                            Ok(value) => value,

                            Err(error) => return fiber.fail(error),
                        },

                        None => Value::Null,
                    };

                    return fiber.complete(value);
                }
            }
        }
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
                ExecutionFlow::Continue => Ok(Value::Null),

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
                    Value::Null
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
                    Value::Array(values) => values,

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
                            .set_or_declare(name, value)?;

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
                    Value::Null
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

            _ => Err(
                "only identifier assignment is supported by the interpreter yet."
                    .to_string(),
            ),
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

            ExpressionKind::Null => Ok(Value::Null),

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

            ExpressionKind::Member { object, name } => {
                if let ExpressionKind::Identifier(ref object_name) = object.kind {
                    if let Some(value) = resolve_host_property(host, object_name, name)? {
                        return Ok(value);
                    }
                }

                let value =
                    self.eval_expression(instance, scopes, object, host)?;

                match value {
                    Value::Map(map) => map
                        .get(name)
                        .cloned()
                        .ok_or_else(|| {
                            format!("map has no key '{}'", name)
                        }),

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

                Ok(Value::Array(values))
            }

            ExpressionKind::Map(entries) => {
                let mut values = BTreeMap::new();

                for (key, expression) in entries {
                    values.insert(
                        key.clone(),
                        self.eval_expression(
                            instance,
                            scopes,
                            expression,
                            host,
                        )?,
                    );
                }

                Ok(Value::Map(values))
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

                        Ok(Value::Null)
                    }

                    "wait" => Err(
                        "wait() is a yielding operation and must be used as a standalone statement on a resumable AeoScript fiber."
                            .to_string(),
                    ),

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
                if matches!(
                    object.kind,
                    ExpressionKind::Identifier(
                        ref object_name
                    ) if object_name == "debug"
                ) && name == "log"
                {
                    let values =
                        self.eval_arguments(
                            instance,
                            scopes,
                            arguments,
                            host,
                        )?;

                    self.log_values(&values);

                    return Ok(Value::Null);
                }

                if matches!(
                    object.kind,
                    ExpressionKind::Identifier(
                        ref object_name
                    ) if object_name == "time"
                ) && name == "delta"
                {
                    let values =
                        self.eval_arguments(
                            instance,
                            scopes,
                            arguments,
                            host,
                        )?;

                    if !values.is_empty() {
                        return Err("time.delta() expects no arguments".to_string());
                    }

                    return Ok(Value::Number(host.delta_time));
                }

                let object_value = self.eval_expression(instance, scopes, object, host)?;
                let values = self.eval_arguments(instance, scopes, arguments, host)?;

                if let Value::Handle { kind, id } = object_value {
                    if let Some(result) = call_host_member(host, kind, id, name, &values)? {
                        return Ok(result);
                    }
                }

                if matches!(
                    object.kind,
                    ExpressionKind::Identifier(
                        ref object_name
                    ) if object_name == "debug"
                ) {
                    return Err(format!(
                        "unknown debug function '{}'",
                        name
                    ));
                }

                Err(
                    "engine/native member calls are not connected to the interpreter yet."
                        .to_string(),
                )
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
        let line = values
            .iter()
            .map(Value::display_string)
            .collect::<Vec<_>>()
            .join(" ");

        self.output.push(line);
    }

    fn eval_index(
        &self,
        object: Value,
        index: Value,
    ) -> Result<Value, String> {
        match object {
            Value::Array(values) => {
                let number = index.as_number()?;

                if number.fract() != 0.0 || number < 0.0 {
                    return Err(
                        "array index must be a non-negative integer."
                            .to_string(),
                    );
                }

                let index = number as usize;

                values
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
                let key = index.as_string()?;

                map.get(key)
                    .cloned()
                    .ok_or_else(|| {
                        format!("map has no key '{}'", key)
                    })
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
    use crate::engine::entity::EntityManager;

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
            interpreter.output(),
            &[
                "42".to_string(),
                "hello 42".to_string()
            ]
        );
    }

    #[test]
    fn null_can_be_used_as_an_optional_condition() {
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
fn fiber_rejects_zero_wait() {
    let source = r#"
entity Test {

    fn update(dt: number) {
        wait(0)
    }
}
"#;

    let mut interpreter = interpreter(source);

    let mut em = test_host();
    let mut host = HostContext { delta_time: 1.0, engine: &mut em };

    let instance = interpreter
        .instantiate_entity("Test", 1, &mut host)
        .expect("entity should instantiate");

    let mut fiber = interpreter
        .start_fiber(
            instance,
            "update",
            vec![Value::Number(1.0)],
        )
        .expect("fiber should start");

    let result = interpreter.resume_fiber(&mut fiber, &mut host);

    assert!(matches!(
        result,
        FiberResult::Failed(message)
            if message.contains("greater than zero")
    ));

    assert!(fiber.is_finished());
}
}
