use crate::scripting::api::HostContext;
use crate::scripting::interpreter::Interpreter;
use crate::scripting::value::{HandleKind, Value};

pub fn resolve_stdlib_property(
    _object_name: &str,
    _property_name: &str,
) -> Result<Option<Value>, String> {
    Ok(None)
}

pub fn call_stdlib_function(
    _interpreter: &mut Interpreter,
    _instance: &mut crate::scripting::interpreter::ScriptInstance,
    _scopes: &mut Vec<crate::scripting::value::Scope>,
    object_name: &str,
    method_name: &str,
    args: &[Value],
    _host: &mut HostContext,
) -> Result<Option<Value>, String> {
    match object_name {
        "math" => {
            let res = match method_name {
                "abs" => {
                    if args.len() != 1 {
                        return Err("math.abs expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.abs())
                }
                "min" => {
                    if args.len() != 2 {
                        return Err("math.min expects exactly 2 arguments".to_string());
                    }
                    Value::Number(args[0].as_number()?.min(args[1].as_number()?))
                }
                "max" => {
                    if args.len() != 2 {
                        return Err("math.max expects exactly 2 arguments".to_string());
                    }
                    Value::Number(args[0].as_number()?.max(args[1].as_number()?))
                }
                "floor" => {
                    if args.len() != 1 {
                        return Err("math.floor expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.floor())
                }
                "ceil" => {
                    if args.len() != 1 {
                        return Err("math.ceil expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.ceil())
                }
                "round" => {
                    if args.len() != 1 {
                        return Err("math.round expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.round())
                }
                "sqrt" => {
                    if args.len() != 1 {
                        return Err("math.sqrt expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.sqrt())
                }
                "pow" => {
                    if args.len() != 2 {
                        return Err("math.pow expects exactly 2 arguments".to_string());
                    }
                    Value::Number(args[0].as_number()?.powf(args[1].as_number()?))
                }
                "sin" => {
                    if args.len() != 1 {
                        return Err("math.sin expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.sin())
                }
                "cos" => {
                    if args.len() != 1 {
                        return Err("math.cos expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.cos())
                }
                "tan" => {
                    if args.len() != 1 {
                        return Err("math.tan expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.tan())
                }
                "random" => {
                    use rand::Rng;
                    let mut rng = rand::thread_rng();
                    if args.is_empty() {
                        Value::Number(rng.r#gen::<f64>())
                    } else if args.len() == 2 {
                        let min = args[0].as_number()? as i64;
                        let max = args[1].as_number()? as i64;
                        if min > max {
                            return Err("math.random min cannot be greater than max".to_string());
                        }
                        Value::Number(rng.gen_range(min..=max) as f64)
                    } else {
                        return Err("math.random expects 0 or 2 arguments".to_string());
                    }
                }
                "clamp" => {
                    if args.len() != 3 {
                        return Err("math.clamp expects exactly 3 arguments".to_string());
                    }
                    let val = args[0].as_number()?;
                    let min = args[1].as_number()?;
                    let max = args[2].as_number()?;
                    if min > max {
                        return Err("math.clamp min cannot be greater than max".to_string());
                    }
                    Value::Number(val.clamp(min, max))
                }
                "lerp" => {
                    if args.len() != 3 {
                        return Err("math.lerp expects exactly 3 arguments".to_string());
                    }
                    let a = args[0].as_number()?;
                    let b = args[1].as_number()?;
                    let t = args[2].as_number()?;
                    Value::Number(a + (b - a) * t)
                }
                "deg_to_rad" => {
                    if args.len() != 1 {
                        return Err("math.deg_to_rad expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.to_radians())
                }
                "rad_to_deg" => {
                    if args.len() != 1 {
                        return Err("math.rad_to_deg expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_number()?.to_degrees())
                }
                _ => return Ok(None),
            };
            return Ok(Some(res));
        }
        "cell" => {
            let res = match method_name {
                "new" => {
                    if args.len() != 1 {
                        return Err("cell.new expects exactly 1 argument (cell_type)".to_string());
                    }
                    let cell_type = args[0].as_string()?;
                    let (kind, id) = _host.engine.create_runtime_cell(cell_type)?;
                    Value::Handle { kind, id }
                }
                "delete" => {
                    if args.len() != 1 {
                        return Err("cell.delete expects exactly 1 argument (handle)".to_string());
                    }
                    let (_, id) = args[0].as_handle()?;
                    _host.engine.delete_cell(id)?;
                    Value::Nil
                }
                "get" => {
                    if args.len() != 1 {
                        return Err("cell.get expects exactly 1 argument (id)".to_string());
                    }
                    let id = args[0].as_number()? as u64;
                    if _host.engine.cell_exists(id) {
                        Value::Handle {
                            kind: HandleKind::Cell,
                            id,
                        }
                    } else {
                        Value::Nil
                    }
                }
                "exists" => {
                    if args.len() != 1 {
                        return Err("cell.exists expects exactly 1 argument (id)".to_string());
                    }
                    let id = args[0].as_number()? as u64;
                    Value::Bool(_host.engine.cell_exists(id))
                }
                _ => return Ok(None),
            };
            return Ok(Some(res));
        }
        "entity" => {
            let res = match method_name {
                "get" => {
                    if args.len() != 1 {
                        return Err("entity.get expects exactly 1 argument (id)".to_string());
                    }
                    let id = args[0].as_number()? as u64;
                    if _host.engine.entity_exists(id) {
                        Value::Handle {
                            kind: HandleKind::Entity,
                            id,
                        }
                    } else {
                        Value::Nil
                    }
                }
                "exists" => {
                    if args.len() != 1 {
                        return Err("entity.exists expects exactly 1 argument (id)".to_string());
                    }
                    let id = args[0].as_number()? as u64;
                    Value::Bool(_host.engine.entity_exists(id))
                }
                _ => return Ok(None),
            };
            return Ok(Some(res));
        }
        "basket" => {
            let res = match method_name {
                "len" => {
                    if args.len() != 1 {
                        return Err("basket.len expects 1 argument".to_string());
                    }
                    let basket_val = args[0].as_basket()?;
                    Value::Number(basket_val.borrow().elements.len() as f64)
                }
                "insert" => {
                    if args.len() < 2 || args.len() > 3 {
                        return Err("basket.insert expects 2 or 3 arguments".to_string());
                    }
                    let basket_val = args[0].as_basket()?;
                    let mut borrowed = basket_val.borrow_mut();
                    if borrowed.frozen {
                        return Err("cannot mutate frozen basket".to_string());
                    }
                    if args.len() == 2 {
                        let value = args[1].clone();
                        borrowed.elements.push(value);
                    } else {
                        let pos = args[1].as_index()?;
                        let value = args[2].clone();
                        if pos > borrowed.elements.len() {
                            return Err("basket.insert index out of bounds".to_string());
                        }
                        borrowed.elements.insert(pos, value);
                    }
                    Value::Nil
                }
                "remove" => {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("basket.remove expects 1 or 2 arguments".to_string());
                    }
                    let basket_val = args[0].as_basket()?;
                    let mut borrowed = basket_val.borrow_mut();
                    if borrowed.frozen {
                        return Err("cannot mutate frozen basket".to_string());
                    }
                    if borrowed.elements.is_empty() {
                        return Err("cannot remove from empty basket".to_string());
                    }
                    if args.len() == 1 {
                        borrowed.elements.pop().unwrap_or(Value::Nil)
                    } else {
                        let pos = args[1].as_index()?;
                        if pos >= borrowed.elements.len() {
                            return Err("basket.remove index out of bounds".to_string());
                        }
                        borrowed.elements.remove(pos)
                    }
                }
                "sort" => {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("basket.sort expects 1 or 2 arguments".to_string());
                    }
                    let basket_val = args[0].as_basket()?;
                    if args.len() == 2 {
                        return Err("comparator not supported yet".to_string());
                    }
                    let mut borrowed = basket_val.borrow_mut();
                    if borrowed.frozen {
                        return Err("cannot mutate frozen basket".to_string());
                    }

                    let mut err = None;
                    borrowed.elements.sort_by(|a, b| match (a, b) {
                        (Value::Number(na), Value::Number(nb)) => na.total_cmp(nb),
                        (Value::String(sa), Value::String(sb)) => sa.cmp(sb),
                        _ => {
                            if err.is_none() {
                                err = Some(format!(
                                    "cannot compare {} and {}",
                                    a.type_name(),
                                    b.type_name()
                                ));
                            }
                            std::cmp::Ordering::Equal
                        }
                    });
                    if let Some(e) = err {
                        return Err(e);
                    }
                    Value::Nil
                }
                "clear" => {
                    if args.len() != 1 {
                        return Err("basket.clear expects exactly 1 argument".to_string());
                    }
                    let basket_val = args[0].as_basket()?;
                    let mut borrowed = basket_val.borrow_mut();
                    if borrowed.frozen {
                        return Err("cannot mutate frozen basket".to_string());
                    }
                    borrowed.elements.clear();
                    Value::Nil
                }
                "create" => {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("basket.create expects 1 or 2 arguments".to_string());
                    }
                    let count = args[0].as_index()?;
                    let default_value = if args.len() == 2 {
                        args[1].clone()
                    } else {
                        Value::Nil
                    };
                    let mut elements = Vec::with_capacity(count);
                    for _ in 0..count {
                        elements.push(default_value.clone());
                    }
                    Value::array(elements)
                }
                "find" => {
                    if args.len() < 2 || args.len() > 3 {
                        return Err("basket.find expects 2 or 3 arguments".to_string());
                    }
                    let basket_val = args[0].as_basket()?;
                    let target = &args[1];
                    let init = if args.len() == 3 {
                        args[2].as_index()?
                    } else {
                        0
                    };

                    let borrowed = basket_val.borrow();
                    if init > borrowed.elements.len() {
                        return Err("basket.find init index out of bounds".to_string());
                    }
                    let mut found_idx = -1;
                    for i in init..borrowed.elements.len() {
                        if &borrowed.elements[i] == target {
                            found_idx = i as i32;
                            break;
                        }
                    }
                    if found_idx == -1 {
                        Value::Nil
                    } else {
                        Value::Number(found_idx as f64)
                    }
                }
                "move" => {
                    if args.len() < 4 || args.len() > 5 {
                        return Err("basket.move expects 4 or 5 arguments (src, a, b, t, [dst])"
                            .to_string());
                    }
                    let src_basket = args[0].as_basket()?;
                    let a = args[1].as_index()?;
                    let b = args[2].as_index()?;
                    let t = args[3].as_index()?;

                    let has_dst = args.len() == 5;
                    let dst_basket = if has_dst {
                        args[4].as_basket()?
                    } else {
                        src_basket.clone()
                    };

                    let slice_to_move: Vec<Value> = {
                        let borrowed_src = src_basket.borrow();
                        if a > b || b >= borrowed_src.elements.len() {
                            return Err("invalid source range bounds".to_string());
                        }
                        borrowed_src.elements[a..=b].iter().cloned().collect()
                    };

                    let mut borrowed_dst = dst_basket.borrow_mut();
                    if borrowed_dst.frozen {
                        return Err("cannot mutate frozen destination basket".to_string());
                    }
                    if t > borrowed_dst.elements.len() {
                        return Err("destination index out of bounds".to_string());
                    }

                    if t + slice_to_move.len() > borrowed_dst.elements.len() {
                        borrowed_dst
                            .elements
                            .resize(t + slice_to_move.len(), Value::Nil);
                    }

                    for (i, val) in slice_to_move.into_iter().enumerate() {
                        borrowed_dst.elements[t + i] = val;
                    }

                    Value::Nil
                }
                "concat" => {
                    if args.len() < 1 || args.len() > 4 {
                        return Err("basket.concat expects 1, 2, 3, or 4 arguments".to_string());
                    }
                    let basket_val = args[0].as_basket()?;
                    let sep = if args.len() >= 2 {
                        args[1].as_string()?
                    } else {
                        ""
                    };
                    let borrowed = basket_val.borrow();
                    if borrowed.elements.is_empty() {
                        return Ok(Some(Value::String("".to_string())));
                    }

                    let i = if args.len() >= 3 {
                        args[2].as_index()?
                    } else {
                        0
                    };
                    let j = if args.len() == 4 {
                        args[3].as_index()?
                    } else {
                        borrowed.elements.len().saturating_sub(1)
                    };

                    if i >= borrowed.elements.len() || j >= borrowed.elements.len() || i > j {
                        return Err("invalid bounds for basket.concat".to_string());
                    }

                    let slice = &borrowed.elements[i..=j];
                    let str_parts: Vec<String> = slice.iter().map(|v| v.display_string()).collect();
                    Value::String(str_parts.join(sep))
                }
                "clone" => {
                    if args.len() != 1 {
                        return Err("basket.clone expects exactly 1 argument".to_string());
                    }
                    let basket_val = args[0].as_basket()?;
                    Value::array(basket_val.borrow().elements.clone())
                }
                "freeze" => {
                    if args.len() != 1 {
                        return Err("basket.freeze expects exactly 1 argument".to_string());
                    }
                    let basket_val = args[0].as_basket()?;
                    basket_val.borrow_mut().frozen = true;
                    args[0].clone()
                }
                _ => return Ok(None),
            };
            return Ok(Some(res));
        }
        "string" => {
            let res = match method_name {
                "len" => {
                    if args.len() != 1 {
                        return Err("string.len expects exactly 1 argument".to_string());
                    }
                    Value::Number(args[0].as_string()?.chars().count() as f64)
                }
                "lower" => {
                    if args.len() != 1 {
                        return Err("string.lower expects exactly 1 argument".to_string());
                    }
                    Value::String(args[0].as_string()?.to_lowercase())
                }
                "upper" => {
                    if args.len() != 1 {
                        return Err("string.upper expects exactly 1 argument".to_string());
                    }
                    Value::String(args[0].as_string()?.to_uppercase())
                }
                "reverse" => {
                    if args.len() != 1 {
                        return Err("string.reverse expects exactly 1 argument".to_string());
                    }
                    Value::String(args[0].as_string()?.chars().rev().collect())
                }
                "split" => {
                    if args.len() < 1 || args.len() > 2 {
                        return Err("string.split expects 1 or 2 arguments".to_string());
                    }
                    let s = args[0].as_string()?;
                    let sep = if args.len() == 2 {
                        args[1].as_string()?
                    } else {
                        ""
                    };

                    let parts: Vec<Value> = if sep.is_empty() {
                        s.chars().map(|c| Value::String(c.to_string())).collect()
                    } else {
                        s.split(sep)
                            .map(|part| Value::String(part.to_string()))
                            .collect()
                    };
                    Value::array(parts)
                }
                _ => return Ok(None),
            };
            return Ok(Some(res));
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::entity::EntityManager;
    use crate::scripting::api::HostContext;
    use crate::scripting::ast::Program;
    use crate::scripting::interpreter::{Interpreter, ScriptInstance};
    use crate::scripting::source::SourceSpan;
    use crate::scripting::value::Scope;

    fn setup() -> (Interpreter, ScriptInstance, Vec<Scope>, EntityManager) {
        let program = Program {
            span: SourceSpan::new(0, 0),
            declarations: vec![],
            statements: vec![],
        };
        let interpreter = Interpreter::new(program);
        let instance = ScriptInstance::new_empty();
        let scopes = vec![Scope::new()];
        let em = EntityManager::new();
        (interpreter, instance, scopes, em)
    }

    #[test]
    fn test_math_random_no_args() {
        let (mut interpreter, mut instance, mut scopes, mut em) = setup();
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut em,
        };

        let res = call_stdlib_function(
            &mut interpreter,
            &mut instance,
            &mut scopes,
            "math",
            "random",
            &[],
            &mut host,
        )
        .unwrap()
        .unwrap();
        let val = res.as_number().unwrap();
        assert!(val >= 0.0 && val < 1.0);
    }

    #[test]
    fn test_math_random_range() {
        let (mut interpreter, mut instance, mut scopes, mut em) = setup();
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut em,
        };

        // [1, 1] -> 1
        let res = call_stdlib_function(
            &mut interpreter,
            &mut instance,
            &mut scopes,
            "math",
            "random",
            &[Value::Number(1.0), Value::Number(1.0)],
            &mut host,
        )
        .unwrap()
        .unwrap();
        assert_eq!(res.as_number().unwrap(), 1.0);

        // [1, 3] -> 1, 2, or 3
        for _ in 0..100 {
            let res = call_stdlib_function(
                &mut interpreter,
                &mut instance,
                &mut scopes,
                "math",
                "random",
                &[Value::Number(1.0), Value::Number(3.0)],
                &mut host,
            )
            .unwrap()
            .unwrap();
            let val = res.as_number().unwrap();
            assert!(val == 1.0 || val == 2.0 || val == 3.0);
        }
    }

    #[test]
    fn test_math_random_invalid_range() {
        let (mut interpreter, mut instance, mut scopes, mut em) = setup();
        let mut host = HostContext {
            delta_time: 0.0,
            engine: &mut em,
        };

        let res = call_stdlib_function(
            &mut interpreter,
            &mut instance,
            &mut scopes,
            "math",
            "random",
            &[Value::Number(10.0), Value::Number(1.0)],
            &mut host,
        );
        assert!(res.is_err());
        assert!(res.unwrap_err().contains("min cannot be greater than max"));
    }
}
