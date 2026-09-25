use std::cell::RefCell;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::scripting::api::{
    call_host_function, call_host_member, resolve_host_member_property, resolve_host_property,
    set_host_member_property, HostContext,
};
use crate::scripting::ast::*;
use crate::scripting::log::{LogRecord, LogSeverity};
use crate::scripting::source::SourceSpan;
use crate::scripting::value::{HandleKind, MapKey, Scope, Value};

use super::interpreter_compile::{compile_function, FunctionCompiler};
use super::{CallFrame, Interpreter, ScriptFiber, ScriptInstance, FRAME_PUSHED_SENTINEL};

impl Interpreter {
    pub(super) fn assign_target(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        target: &Expression,
        operator: AssignmentOperator,
        right: Value,
        host: &mut HostContext,
        captured_scopes: &[Scope],
    ) -> Result<(), String> {
        match &target.kind {
            ExpressionKind::Identifier(name) => {
                let value = if operator == AssignmentOperator::Assign {
                    right
                } else {
                    let left = self.resolve_identifier(instance, scopes, name, captured_scopes)?;

                    self.apply_assignment(operator, left, right)?
                };

                for scope in scopes.iter_mut().rev() {
                    if scope.contains(name) {
                        eprintln!(
                            "[INTERPRETER][ASSIGN] local '{}' = {:?} function={:?}",
                            name, value, self.current_function_name
                        );

                        return scope.set(name, value);
                    }
                }

                for scope in captured_scopes.iter().rev() {
                    if scope.contains(name) {
                        eprintln!(
                            "[INTERPRETER][ASSIGN] captured '{}' = {:?} function={:?} script={:?}",
                            name, value, self.current_function_name, self.current_script_path
                        );

                        return scope.set(name, value);
                    }
                }

                if self.current_function_name.as_deref() == Some("top-level") {
                    let Some(scope) = scopes.last() else {
                        let error =
                            "AeoScript top-level assignment has no active scope.".to_string();

                        eprintln!(
                            "[INTERPRETER][ERROR] {} variable='{}' script={:?}",
                            error, name, self.current_script_path
                        );

                        return Err(error);
                    };

                    eprintln!(
                        "[INTERPRETER][TOP-LEVEL-ASSIGN] creating '{}' = {:?} script={:?}",
                        name, value, self.current_script_path
                    );

                    scope.set_or_declare(name.to_string(), value)?;

                    eprintln!(
                        "[INTERPRETER][TOP-LEVEL-ASSIGN] '{}' now exists={} script={:?}",
                        name,
                        scope.contains(name),
                        self.current_script_path
                    );

                    return Ok(());
                }

                eprintln!(
                    "[INTERPRETER][FIELD-ASSIGN] '{}' = {:?} entity='{}' function={:?}",
                    name,
                    value,
                    instance.entity_name(),
                    self.current_function_name
                );

                instance.set_field(name, value)
            }

            ExpressionKind::Member { object, name } => {
                let object_value =
                    self.eval_expression(instance, scopes, object, host, captured_scopes)?;

                let value = if operator == AssignmentOperator::Assign {
                    right
                } else {
                    let left = match &object_value {
                        Value::Map(map) => map
                            .borrow()
                            .get(&MapKey::String(name.clone()))
                            .cloned()
                            .unwrap_or(Value::Nil),

                        Value::Handle { kind, id } => {
                            if let Some(v) = resolve_host_member_property(host, *kind, *id, name)? {
                                v
                            } else {
                                return Err(format!(
                                    "engine member '{}' is not available on handle kind {} yet",
                                    name,
                                    kind.name()
                                ));
                            }
                        }

                        Value::Object(instance_arc) => instance_arc
                            .borrow()
                            .get_field(name)
                            .cloned()
                            .unwrap_or(Value::Nil),

                        _ => {
                            return Err(format!(
                                "cannot access member on type {}",
                                object_value.type_name()
                            ));
                        }
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

                    Value::Object(instance_arc) => {
                        if matches!(value, Value::Nil) {
                            instance_arc.borrow_mut().set_field(name, Value::Nil)?;
                        } else {
                            instance_arc.borrow_mut().set_field(name, value)?;
                        }

                        Ok(())
                    }

                    _ => Err(format!(
                        "cannot assign to member of type {}",
                        object_value.type_name()
                    )),
                }
            }

            ExpressionKind::Index {
                object: target_object,
                index,
            } => {
                if let ExpressionKind::Member {
                    object: handle_expr,
                    name,
                } = &target_object.kind
                {
                    if name == "attributes" {
                        let handle_value = self.eval_expression(
                            instance,
                            scopes,
                            handle_expr,
                            host,
                            captured_scopes,
                        )?;

                        if let Value::Handle { kind, id } = handle_value {
                            if kind == HandleKind::Cell || kind == HandleKind::Light {
                                let key_value = self.eval_expression(
                                    instance,
                                    scopes,
                                    index,
                                    host,
                                    captured_scopes,
                                )?;

                                let key = key_value.as_string()?;

                                let value_to_assign = if operator == AssignmentOperator::Assign {
                                    right
                                } else {
                                    let current = host
                                        .engine
                                        .get_property(kind, id, "attributes")?
                                        .ok_or_else(|| "could not read attributes".to_string())?;

                                    let map_arc = current.as_map()?;

                                    let left = map_arc
                                        .borrow()
                                        .get(&MapKey::String(key.to_string()))
                                        .cloned()
                                        .unwrap_or(Value::Nil);

                                    self.apply_assignment(operator, left, right)?
                                };

                                if matches!(value_to_assign, Value::Nil) {
                                    host.engine.remove_attribute(id, key)?;
                                } else {
                                    host.engine.set_attribute(
                                        id,
                                        key.to_string(),
                                        value_to_assign,
                                    )?;
                                }

                                return Ok(());
                            }
                        }
                    }
                }

                let object_value =
                    self.eval_expression(instance, scopes, target_object, host, captured_scopes)?;

                let index_value =
                    self.eval_expression(instance, scopes, index, host, captured_scopes)?;

                let value = if operator == AssignmentOperator::Assign {
                    right
                } else {
                    let left = match &object_value {
                        Value::Array(array) => {
                            let idx = index_value.as_index()?;

                            array
                                .borrow()
                                .elements
                                .get(idx)
                                .cloned()
                                .ok_or_else(|| format!("index {} out of bounds", idx))?
                        }

                        Value::Map(map) => {
                            let key = index_value.as_map_key()?;

                            map.borrow().get(&key).cloned().unwrap_or(Value::Nil)
                        }

                        _ => {
                            return Err(format!("cannot index type {}", object_value.type_name()));
                        }
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

                    _ => Err(format!(
                        "cannot assign to index of type {}",
                        object_value.type_name()
                    )),
                }
            }

            _ => Err(format!(
                "cannot assign to expression of type {:?}",
                target.kind
            )),
        }
    }

    pub(super) fn apply_assignment(
        &self,
        operator: AssignmentOperator,
        left: Value,
        right: Value,
    ) -> Result<Value, String> {
        match operator {
            AssignmentOperator::Assign => Ok(right),

            AssignmentOperator::Add => self.apply_binary(BinaryOperator::Add, left, right),

            AssignmentOperator::Subtract => {
                self.apply_binary(BinaryOperator::Subtract, left, right)
            }

            AssignmentOperator::Multiply => {
                self.apply_binary(BinaryOperator::Multiply, left, right)
            }

            AssignmentOperator::Divide => self.apply_binary(BinaryOperator::Divide, left, right),
        }
    }

    pub(super) fn invoke_callable_opt_fiber(
        &mut self,
        instance: &mut ScriptInstance,
        compiled: Arc<CompiledFunction>,
        params: Vec<String>,
        arguments: Vec<Value>,
        self_value: Option<Value>,
        function_name: String,
        host: &mut HostContext,
        captures: Vec<Scope>,
        call_site_span: Option<SourceSpan>,
        opt_fiber: Option<&mut ScriptFiber>,
    ) -> Result<Value, String> {
        if let Some(fiber) = opt_fiber {
            if fiber.stack.len() >= self.max_call_depth {
                return Err(format!(
                    "AeoScript call depth exceeded the limit of {}.",
                    self.max_call_depth
                ));
            }

            if params.len() != arguments.len() {
                return Err(format!(
                    "function expected {} argument(s), got {}",
                    params.len(),
                    arguments.len()
                ));
            }

            let parameter_scope = Scope::new();

            if let Some(sv) = self_value {
                parameter_scope.declare("self", sv, false)?;
            }

            for (name, argument) in params.into_iter().zip(arguments.into_iter()) {
                parameter_scope.declare(name, argument, false)?;
            }

            fiber.stack.push(CallFrame {
                function: compiled,
                pc: 0,
                scopes: vec![parameter_scope],
                for_states: Vec::new(),
                function_name,
                captured_scopes: captures,
                expects_return_value: true,
                call_site_span,
            });

            Err(FRAME_PUSHED_SENTINEL.to_string())
        } else {
            self.call_direct(
                instance,
                compiled,
                params,
                arguments,
                self_value,
                function_name,
                host,
                captures,
            )
        }
    }

    pub(super) fn eval_expression_on_fiber(
        &mut self,
        fiber: &mut ScriptFiber,
        scopes: &mut Vec<Scope>,
        expression: &Expression,
        host: &mut HostContext,
        captured_scopes: &[Scope],
    ) -> Result<Value, String> {
        let mut instance = std::mem::replace(&mut fiber.instance, ScriptInstance::new_empty());

        let result = self.eval_expression_opt_fiber(
            &mut instance,
            scopes,
            expression,
            host,
            captured_scopes,
            Some(fiber),
        );

        fiber.instance = instance;

        result
    }

    pub(super) fn eval_expression(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        expression: &Expression,
        host: &mut HostContext,
        captured_scopes: &[Scope],
    ) -> Result<Value, String> {
        self.eval_expression_opt_fiber(instance, scopes, expression, host, captured_scopes, None)
    }

    pub(super) fn eval_expression_opt_fiber(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        expression: &Expression,
        host: &mut HostContext,
        captured_scopes: &[Scope],
        mut opt_fiber: Option<&mut ScriptFiber>,
    ) -> Result<Value, String> {
        self.tick()?;

        if matches!(
            expression.kind,
            ExpressionKind::Call { .. } | ExpressionKind::MethodCall { .. }
        ) {
            if let Some(fiber) = opt_fiber.as_deref_mut() {
                if fiber.return_site_span == Some(expression.span) {
                    fiber.return_site_span = None;

                    if let Some(val) = fiber.return_value.take() {
                        return Ok(val);
                    }
                }
            }
        }

        match &expression.kind {
            ExpressionKind::Literal(val) => Ok(val.clone()),

            ExpressionKind::Number(value) => Ok(Value::Number(*value)),

            ExpressionKind::String(value) => Ok(Value::String(value.clone())),

            ExpressionKind::Bool(value) => Ok(Value::Bool(*value)),

            ExpressionKind::Nil => Ok(Value::Nil),

            ExpressionKind::Identifier(name) => {
                self.resolve_identifier(instance, scopes, name, captured_scopes)
            }

            ExpressionKind::Unary {
                operator,
                expression,
            } => {
                let value = self.eval_expression_opt_fiber(
                    instance,
                    scopes,
                    expression,
                    host,
                    captured_scopes,
                    opt_fiber,
                )?;

                match operator {
                    UnaryOperator::Negate => Ok(Value::Number(-value.as_number()?)),

                    UnaryOperator::Not => Ok(Value::Bool(!value.is_truthy()?)),
                }
            }

            ExpressionKind::Binary {
                left,
                operator,
                right,
            } => {
                if *operator == BinaryOperator::And {
                    let left_value = self.eval_expression_opt_fiber(
                        instance,
                        scopes,
                        left,
                        host,
                        captured_scopes,
                        opt_fiber.as_deref_mut(),
                    )?;

                    if !left_value.is_truthy()? {
                        return Ok(Value::Bool(false));
                    }

                    let right_value = self.eval_expression_opt_fiber(
                        instance,
                        scopes,
                        right,
                        host,
                        captured_scopes,
                        opt_fiber,
                    )?;

                    return Ok(Value::Bool(right_value.is_truthy()?));
                }

                if *operator == BinaryOperator::Or {
                    let left_value = self.eval_expression_opt_fiber(
                        instance,
                        scopes,
                        left,
                        host,
                        captured_scopes,
                        opt_fiber.as_deref_mut(),
                    )?;

                    if left_value.is_truthy()? {
                        return Ok(Value::Bool(true));
                    }

                    let right_value = self.eval_expression_opt_fiber(
                        instance,
                        scopes,
                        right,
                        host,
                        captured_scopes,
                        opt_fiber,
                    )?;

                    return Ok(Value::Bool(right_value.is_truthy()?));
                }

                let left_value = self.eval_expression_opt_fiber(
                    instance,
                    scopes,
                    left,
                    host,
                    captured_scopes,
                    opt_fiber.as_deref_mut(),
                )?;

                let right_value = self.eval_expression_opt_fiber(
                    instance,
                    scopes,
                    right,
                    host,
                    captured_scopes,
                    opt_fiber,
                )?;

                self.apply_binary(*operator, left_value, right_value)
            }

            ExpressionKind::Call { callee, arguments } => self.eval_call_opt_fiber(
                instance,
                scopes,
                expression.span,
                callee,
                arguments,
                host,
                captured_scopes,
                opt_fiber,
            ),

            ExpressionKind::MethodCall {
                object,
                method,
                arguments,
            } => {
                let object_value = self.eval_expression_opt_fiber(
                    instance,
                    scopes,
                    object,
                    host,
                    captured_scopes,
                    opt_fiber.as_deref_mut(),
                )?;

                if let Value::Namespace(ref name) = object_value {
                    let arg_values = self.eval_arguments_opt_fiber(
                        instance,
                        scopes,
                        arguments,
                        host,
                        captured_scopes,
                        opt_fiber.as_deref_mut(),
                    )?;

                    if let Some(res) = crate::scripting::stdlib::call_stdlib_function(
                        self,
                        instance,
                        scopes,
                        name,
                        method,
                        &arg_values,
                        host,
                    )? {
                        return Ok(res);
                    }
                }

                if let Value::Namespace(ref name) = object_value {
                    if name == "debug" && method == "log" {
                        let arg_values = self.eval_arguments_opt_fiber(
                            instance,
                            scopes,
                            arguments,
                            host,
                            captured_scopes,
                            opt_fiber,
                        )?;

                        self.log_values(&arg_values);

                        return Ok(Value::Nil);
                    }
                }

                let arg_values = self.eval_arguments_opt_fiber(
                    instance,
                    scopes,
                    arguments,
                    host,
                    captured_scopes,
                    opt_fiber.as_deref_mut(),
                )?;

                match object_value {
                    Value::Handle { kind, id } => {
                        if let Some(result) =
                            call_host_member(host, kind, id, method, &arg_values)?
                        {
                            Ok(result)
                        } else if let Some(Value::Function(compiled, params, caps)) =
                            host.engine.get_script_property(kind, id, method)
                        {
                            self.invoke_callable_opt_fiber(
                                instance,
                                compiled,
                                params,
                                arg_values,
                                Some(object_value),
                                method.clone(),
                                host,
                                caps,
                                Some(expression.span),
                                opt_fiber,
                            )
                        } else {
                            Err(format!(
                                "engine method '{}' is not available on handle kind {}",
                                method,
                                kind.name()
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
                let object_value = self.eval_expression_opt_fiber(
                    instance,
                    scopes,
                    object,
                    host,
                    captured_scopes,
                    opt_fiber,
                )?;

                if let Value::Namespace(ref namespace) = object_value {
                    if let Some(value) = resolve_host_property(host, namespace, name)? {
                        return Ok(value);
                    }

                    if let Some(value) =
                        crate::scripting::stdlib::resolve_stdlib_property(namespace, name, host)?
                    {
                        return Ok(value);
                    }
                }

                match object_value {
                    Value::Map(map) => {
                        let borrowed = map.borrow();

                        Ok(borrowed
                            .get(&MapKey::String(name.clone()))
                            .cloned()
                            .unwrap_or(Value::Nil))
                    }

                    Value::Handle { kind, id } => {
                        if let Some(value) = resolve_host_member_property(host, kind, id, name)? {
                            Ok(value)
                        } else {
                            Err(format!(
                                "engine member '{}' is not available on handle kind {} yet",
                                name,
                                kind.name()
                            ))
                        }
                    }

                    Value::Object(ref instance_arc) => {
                        let borrowed = instance_arc.borrow();

                        if let Some(val) = borrowed.get_field(name) {
                            Ok(val.clone())
                        } else if let Some(function) =
                            self.find_function(borrowed.entity_name(), name)
                        {
                            let cache_key = (borrowed.entity_name().to_string(), name.clone());

                            let compiled =
                                if let Some(compiled) = self.compiled_functions.get(&cache_key) {
                                    Arc::clone(compiled)
                                } else {
                                    Arc::new(compile_function(&function))
                                };

                            let param_names =
                                function.parameters.iter().map(|p| p.name.clone()).collect();

                            Ok(Value::Function(compiled, param_names, vec![]))
                        } else {
                            Ok(Value::Nil)
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
                let object = self.eval_expression_opt_fiber(
                    instance,
                    scopes,
                    object,
                    host,
                    captured_scopes,
                    opt_fiber.as_deref_mut(),
                )?;

                let index = self.eval_expression_opt_fiber(
                    instance,
                    scopes,
                    index,
                    host,
                    captured_scopes,
                    opt_fiber,
                )?;

                self.eval_index(object, index)
            }

            ExpressionKind::Array(expressions) => {
                let mut values = Vec::with_capacity(expressions.len());

                for expression in expressions {
                    values.push(self.eval_expression_opt_fiber(
                        instance,
                        scopes,
                        expression,
                        host,
                        captured_scopes,
                        opt_fiber.as_deref_mut(),
                    )?);
                }

                Ok(Value::array(values))
            }

            ExpressionKind::Map(entries) => {
                let mut values = BTreeMap::new();

                for (key_expr, expression) in entries {
                    let key_value = self.eval_expression_opt_fiber(
                        instance,
                        scopes,
                        key_expr,
                        host,
                        captured_scopes,
                        opt_fiber.as_deref_mut(),
                    )?;

                    let key = key_value.as_map_key()?;

                    values.insert(
                        key,
                        self.eval_expression_opt_fiber(
                            instance,
                            scopes,
                            expression,
                            host,
                            captured_scopes,
                            opt_fiber.as_deref_mut(),
                        )?,
                    );
                }

                Ok(Value::Map(Arc::new(RefCell::new(values))))
            }

            ExpressionKind::AnonymousFunction {
                parameters,
                return_type: _,
                body,
            } => {
                let mut compiler = FunctionCompiler {
                    instructions: Vec::new(),
                };

                compiler.compile_block(body);

                let compiled = Arc::new(CompiledFunction {
                    instructions: compiler.instructions,
                });

                let param_names = parameters
                    .iter()
                    .map(|parameter| parameter.name.clone())
                    .collect();

                let mut captured_env = Vec::with_capacity(captured_scopes.len() + scopes.len());

                for scope in captured_scopes.iter().chain(scopes.iter()) {
                    if !captured_env
                        .iter()
                        .any(|existing: &Scope| existing == scope)
                    {
                        captured_env.push(scope.clone());
                    }
                }

                Ok(Value::Function(compiled, param_names, captured_env))
            }
        }
    }

    pub(super) fn eval_call(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        call_span: SourceSpan,
        callee: &Expression,
        arguments: &[Expression],
        host: &mut HostContext,
        captured_scopes: &[Scope],
    ) -> Result<Value, String> {
        self.eval_call_opt_fiber(
            instance,
            scopes,
            call_span,
            callee,
            arguments,
            host,
            captured_scopes,
            None,
        )
    }

    pub fn call_object_method_direct(
        &mut self,
        object: &Value,
        method_name: &str,
        args: Vec<Value>,
        host: &mut HostContext,
    ) -> Result<Value, String> {
        let Value::Object(instance_arc) = object else {
            return Err(format!(
                "expected Value::Object, got {}",
                object.type_name()
            ));
        };

        let entity_name = instance_arc.borrow().entity_name().to_string();

        let Some(function) = self.find_function(&entity_name, method_name) else {
            return Err(format!(
                "cannot access method '{}' on {}",
                method_name, entity_name
            ));
        };

        let cache_key = (entity_name.clone(), method_name.to_string());

        let compiled = if let Some(compiled) = self.compiled_functions.get(&cache_key) {
            Arc::clone(compiled)
        } else {
            let compiled = Arc::new(compile_function(&function));

            self.compiled_functions
                .insert(cache_key.clone(), Arc::clone(&compiled));

            compiled
        };

        let param_names = function.parameters.iter().map(|p| p.name.clone()).collect();

        let mut target_instance = instance_arc.borrow().clone();

        let result = self.invoke_callable_opt_fiber(
            &mut target_instance,
            compiled,
            param_names,
            args,
            Some(Value::Object(Arc::clone(instance_arc))),
            method_name.to_string(),
            host,
            vec![],
            None,
            None,
        )?;

        *instance_arc.borrow_mut() = target_instance;

        Ok(result)
    }

    pub(crate) fn eval_call_opt_fiber(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        call_span: SourceSpan,
        callee: &Expression,
        arguments: &[Expression],
        host: &mut HostContext,
        captured_scopes: &[Scope],
        mut opt_fiber: Option<&mut ScriptFiber>,
    ) -> Result<Value, String> {
        if let ExpressionKind::Member { object, name } = &callee.kind {
            let object_value = self.eval_expression_opt_fiber(
                instance,
                scopes,
                object,
                host,
                captured_scopes,
                opt_fiber.as_deref_mut(),
            )?;

            if let Value::Namespace(ref namespace) = object_value {
                let arg_values = self.eval_arguments_opt_fiber(
                    instance,
                    scopes,
                    arguments,
                    host,
                    captured_scopes,
                    opt_fiber.as_deref_mut(),
                )?;

                if namespace == "debug" && name == "log" {
                    self.log_values(&arg_values);
                    return Ok(Value::Nil);
                }

                if let Some(res) = crate::scripting::stdlib::call_stdlib_function(
                    self, instance, scopes, namespace, name, &arg_values, host,
                )? {
                    return Ok(res);
                }

                if let Some(res) = call_host_function(host, name, &arg_values)? {
                    return Ok(res);
                }

                return Err(format!("unknown function '{}.{}'", namespace, name));
            }

            let arg_values = self.eval_arguments_opt_fiber(
                instance,
                scopes,
                arguments,
                host,
                captured_scopes,
                opt_fiber.as_deref_mut(),
            )?;

            match &object_value {
                Value::Handle { kind, id } => {
                    if let Some(result) = call_host_member(host, *kind, *id, name, &arg_values)? {
                        return Ok(result);
                    }

                    if let Some(Value::Function(compiled, params, caps)) =
                        host.engine.get_script_property(*kind, *id, name)
                    {
                        return self.invoke_callable_opt_fiber(
                            instance,
                            compiled,
                            params,
                            arg_values,
                            Some(object_value),
                            name.clone(),
                            host,
                            caps,
                            Some(call_span),
                            opt_fiber,
                        );
                    }

                    return Err(format!(
                        "engine member '{}' is not available on handle kind {}",
                        name,
                        kind.name()
                    ));
                }

                Value::Array(_) | Value::Map(_) | Value::String(_) => {
                    let type_name = object_value.type_name();

                    let stdlib_ns = match type_name {
                        "basket" => "basket",
                        "map" => "map",
                        "string" => "string",
                        _ => type_name,
                    };

                    let mut full_args = Vec::with_capacity(arg_values.len() + 1);

                    full_args.push(object_value.clone());
                    full_args.extend(arg_values.clone());

                    if let Some(res) = crate::scripting::stdlib::call_stdlib_function(
                        self, instance, scopes, stdlib_ns, name, &full_args, host,
                    )? {
                        return Ok(res);
                    }

                    if let Value::Map(map) = &object_value {
                        if let Some(Value::Function(compiled, params, caps)) =
                            map.borrow().get(&MapKey::String(name.clone())).cloned()
                        {
                            return self.invoke_callable_opt_fiber(
                                instance,
                                compiled,
                                params,
                                arg_values,
                                None,
                                name.clone(),
                                host,
                                caps,
                                Some(call_span),
                                opt_fiber,
                            );
                        }
                    }

                    return Err(format!("cannot access member '{}' on {}", name, type_name));
                }

                Value::Object(instance_arc) => {
                    let entity_name = instance_arc.borrow().entity_name().to_string();

                    if let Some(function) = self.find_function(&entity_name, name) {
                        let cache_key = (entity_name.clone(), name.clone());

                        let compiled =
                            if let Some(compiled) = self.compiled_functions.get(&cache_key) {
                                Arc::clone(compiled)
                            } else {
                                let compiled = Arc::new(compile_function(&function));

                                self.compiled_functions
                                    .insert(cache_key.clone(), Arc::clone(&compiled));

                                compiled
                            };

                        let param_names =
                            function.parameters.iter().map(|p| p.name.clone()).collect();

                        let mut target_instance = instance_arc.borrow().clone();

                        let result = self.invoke_callable_opt_fiber(
                            &mut target_instance,
                            compiled,
                            param_names,
                            arg_values,
                            Some(Value::Object(Arc::clone(instance_arc))),
                            name.clone(),
                            host,
                            vec![],
                            Some(call_span),
                            opt_fiber,
                        )?;

                        *instance_arc.borrow_mut() = target_instance;

                        return Ok(result);
                    } else if let Some(val) = instance_arc.borrow().get_field(name).cloned() {
                        if let Value::Function(compiled, params, caps) = val {
                            let mut target_instance = instance_arc.borrow().clone();

                            let result = self.invoke_callable_opt_fiber(
                                &mut target_instance,
                                compiled,
                                params,
                                arg_values,
                                Some(Value::Object(Arc::clone(instance_arc))),
                                name.clone(),
                                host,
                                caps,
                                Some(call_span),
                                opt_fiber,
                            )?;

                            *instance_arc.borrow_mut() = target_instance;

                            return Ok(result);
                        }
                    }

                    return Err(format!(
                        "cannot access method '{}' on {}",
                        name, entity_name
                    ));
                }

                _ => {}
            }
        }

        if let ExpressionKind::Identifier(ref type_name) = callee.kind {
            if let Some(entity) = self.find_entity(type_name) {
                let arg_values = self.eval_arguments_opt_fiber(
                    instance,
                    scopes,
                    arguments,
                    host,
                    captured_scopes,
                    opt_fiber.as_deref_mut(),
                )?;

                let mut obj_instance = ScriptInstance::new_entity(type_name);

                for member in &entity.members {
                    if let EntityMember::Field(field) = member {
                        let init_val = if let Some(ref init_expr) = field.initializer {
                            self.eval_expression_opt_fiber(
                                &mut obj_instance,
                                scopes,
                                init_expr,
                                host,
                                captured_scopes,
                                opt_fiber.as_deref_mut(),
                            )?
                        } else {
                            Value::Nil
                        };

                        obj_instance.set_field(&field.name, init_val)?;
                    }
                }

                let obj_arc = Arc::new(RefCell::new(obj_instance));

                if let Some(constructor) = entity.members.iter().find_map(|m| match m {
                    EntityMember::Function(f) if f.name == "constructor" => Some(f.clone()),

                    _ => None,
                }) {
                    let cache_key = (type_name.clone(), "constructor".to_string());

                    let compiled = if let Some(compiled) = self.compiled_functions.get(&cache_key) {
                        Arc::clone(compiled)
                    } else {
                        let compiled = Arc::new(compile_function(&constructor));

                        self.compiled_functions
                            .insert(cache_key, Arc::clone(&compiled));

                        compiled
                    };

                    let param_names = constructor
                        .parameters
                        .iter()
                        .map(|p| p.name.clone())
                        .collect();

                    let mut target_instance = obj_arc.borrow().clone();

                    let _ = self.invoke_callable_opt_fiber(
                        &mut target_instance,
                        compiled,
                        param_names,
                        arg_values,
                        Some(Value::Object(Arc::clone(&obj_arc))),
                        "constructor".to_string(),
                        host,
                        vec![],
                        Some(call_span),
                        opt_fiber,
                    )?;

                    *obj_arc.borrow_mut() = target_instance;
                }

                return Ok(Value::Object(obj_arc));
            }
        }

        let resolved = match self.eval_expression_opt_fiber(
            instance,
            scopes,
            callee,
            host,
            captured_scopes,
            opt_fiber.as_deref_mut(),
        ) {
            Ok(value) => Some(value),

            Err(e) if e == FRAME_PUSHED_SENTINEL => {
                return Err(e);
            }

            Err(_) if matches!(callee.kind, ExpressionKind::Identifier(_)) => None,

            Err(error) => return Err(error),
        };

        match resolved {
            Some(Value::Function(compiled, params, captures)) => {
                let values = self.eval_arguments_opt_fiber(
                    instance,
                    scopes,
                    arguments,
                    host,
                    captured_scopes,
                    opt_fiber.as_deref_mut(),
                )?;

                self.invoke_callable_opt_fiber(
                    instance,
                    compiled,
                    params,
                    values,
                    None,
                    "anonymous".to_string(),
                    host,
                    captures,
                    Some(call_span),
                    opt_fiber,
                )
            }

            Some(value) => Err(format!("cannot call a {} value", value.type_name())),

            None => {
                let ExpressionKind::Identifier(name) = &callee.kind else {
                    return Err("that expression cannot be called yet.".to_string());
                };

                let values = self.eval_arguments_opt_fiber(
                    instance,
                    scopes,
                    arguments,
                    host,
                    captured_scopes,
                    opt_fiber.as_deref_mut(),
                )?;

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
                            return Err(
                                "get_parent() expects no arguments"
                                    .to_string(),
                            );
                        }

                        if instance.id() == 0 {
                            return Err(
                                "get_parent() called from an unattached script"
                                    .to_string(),
                            );
                        }

                        if let Some((kind, id)) =
                            host.engine.get_parent(
                                HandleKind::Entity,
                                instance.id(),
                            )
                        {
                            Ok(Value::Handle { kind, id })
                        } else {
                            Ok(Value::Nil)
                        }
                    }

                    _ => {
                        let function = self
                            .find_function(
                                &instance.entity_name,
                                name,
                            )
                            .ok_or_else(|| {
                                format!(
                                    "function '{}' does not exist",
                                    name
                                )
                            })?;

                        let cache_key = (
                            instance.entity_name.clone(),
                            name.clone(),
                        );

                        let compiled =
                            if let Some(compiled) =
                                self.compiled_functions
                                    .get(&cache_key)
                            {
                                Arc::clone(compiled)
                            } else {
                                let compiled =
                                    Arc::new(
                                        compile_function(
                                            &function,
                                        ),
                                    );

                                self.compiled_functions
                                    .insert(
                                        cache_key,
                                        Arc::clone(&compiled),
                                    );

                                compiled
                            };

                        let param_names = function
                            .parameters
                            .iter()
                            .map(|p| p.name.clone())
                            .collect();

                        self.invoke_callable_opt_fiber(
                            instance,
                            compiled,
                            param_names,
                            values,
                            None,
                            name.clone(),
                            host,
                            vec![],
                            Some(call_span),
                            opt_fiber,
                        )
                    }
                }
            }
        }
    }

    pub(super) fn eval_arguments(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        arguments: &[Expression],
        host: &mut HostContext,
        captured_scopes: &[Scope],
    ) -> Result<Vec<Value>, String> {
        self.eval_arguments_opt_fiber(instance, scopes, arguments, host, captured_scopes, None)
    }

    pub(super) fn eval_arguments_opt_fiber(
        &mut self,
        instance: &mut ScriptInstance,
        scopes: &mut Vec<Scope>,
        arguments: &[Expression],
        host: &mut HostContext,
        captured_scopes: &[Scope],
        mut opt_fiber: Option<&mut ScriptFiber>,
    ) -> Result<Vec<Value>, String> {
        let mut values = Vec::with_capacity(arguments.len());

        for argument in arguments {
            values.push(self.eval_expression_opt_fiber(
                instance,
                scopes,
                argument,
                host,
                captured_scopes,
                opt_fiber.as_deref_mut(),
            )?);
        }

        Ok(values)
    }

    pub(super) fn log_values(&mut self, values: &[Value]) {
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

    pub(super) fn eval_index(&self, object: Value, index: Value) -> Result<Value, String> {
        match object {
            Value::Array(values) => {
                let index = index.as_index()?;

                let borrowed = values.borrow();

                borrowed
                    .elements
                    .get(index)
                    .cloned()
                    .ok_or_else(|| format!("array index {} is out of bounds", index))
            }

            Value::Map(map) => {
                let key = index.as_map_key()?;

                let borrowed = map.borrow();

                Ok(borrowed.get(&key).cloned().unwrap_or(Value::Nil))
            }

            other => Err(format!("cannot index {}", other.type_name())),
        }
    }

    pub(super) fn apply_binary(
        &self,
        operator: BinaryOperator,
        left: Value,
        right: Value,
    ) -> Result<Value, String> {
        match operator {
            BinaryOperator::Add => match (left, right) {
                (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),

                (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),

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
                    return Err("division by zero.".to_string());
                }

                Ok(Value::Number(left / right))
            }

            BinaryOperator::Modulo => {
                let left = left.as_number()?;
                let right = right.as_number()?;

                if right == 0.0 {
                    return Err("modulo by zero.".to_string());
                }

                Ok(Value::Number(left % right))
            }

            BinaryOperator::Equal => Ok(Value::Bool(left == right)),

            BinaryOperator::NotEqual => Ok(Value::Bool(left != right)),

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
                unreachable!("logical operators are handled with short-circuit evaluation")
            }
        }
    }

    pub(super) fn resolve_identifier(
        &self,
        instance: &ScriptInstance,
        scopes: &[Scope],
        name: &str,
        captured_scopes: &[Scope],
    ) -> Result<Value, String> {
        for scope in scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value);
            }
        }

        for scope in captured_scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value);
            }
        }

        if let Some(value) = instance.get_field(name) {
            return Ok(value.clone());
        }

        if name == "math"
            || name == "basket"
            || name == "string"
            || name == "cell"
            || name == "debug"
            || name == "entity"
            || name == "script"
            || name == "event"
            || name == "test"
            || name == "time"
            || name == "ui"
            || name == "input"
            || name == "camera"
            || name == "physics"
            || name == "player"
            || name == "get"
        {
            return Ok(Value::Namespace(name.to_string()));
        }

        if let Some(function) = self.find_function(instance.entity_name(), name) {
            let cache_key = (instance.entity_name().to_string(), name.to_string());

            let compiled = if let Some(compiled) = self.compiled_functions.get(&cache_key) {
                Arc::clone(compiled)
            } else {
                Arc::new(compile_function(&function))
            };

            let param_names = function.parameters.iter().map(|p| p.name.clone()).collect();

            let captured_env = if instance.entity_name() == "Global" {
                if self.current_function_name.as_deref() == Some("top-level") {
                    scopes.to_vec()
                } else {
                    captured_scopes.to_vec()
                }
            } else {
                Vec::new()
            };

            return Ok(Value::Function(compiled, param_names, captured_env));
        }

        eprintln!(
            "[INTERPRETER][ERROR] unknown variable='{}' script={:?} entity={:?} function={:?} local_scopes={} captured_scopes={}",
            name,
            self.current_script_path,
            self.current_entity_name,
            self.current_function_name,
            scopes.len(),
            captured_scopes.len()
        );

        Err(format!("unknown variable '{}'", name))
    }
}
