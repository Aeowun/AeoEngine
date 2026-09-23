use crate::scripting::api::HostContext;
use crate::scripting::interpreter::Interpreter;
use crate::scripting::value::{HandleKind, Value};

pub fn resolve_stdlib_property(
    object_name: &str,
    property_name: &str,
    host: &mut HostContext,
) -> Result<Option<Value>, String> {
    if object_name == "player" && property_name == "position" {
        if let Some(pos) = host.engine.get_player_position() {
            return Ok(Some(Value::array(vec![
                Value::Number(pos[0] as f64),
                Value::Number(pos[1] as f64),
                Value::Number(pos[2] as f64),
            ])));
        }
    }
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
        "input" => {
            match method_name {
                "get_move_vector" => {
                    let mv = _host.engine.get_input_move_vector();
                    Ok(Some(Value::array(vec![
                        Value::Number(mv[0] as f64),
                        Value::Number(mv[1] as f64),
                    ])))
                }
                "is_jump_pressed" => Ok(Some(Value::Bool(_host.engine.is_input_jump_pressed()))),
                "get_orbit_delta" => {
                    let orbit = _host.engine.get_input_orbit_delta();
                    Ok(Some(Value::array(vec![
                        Value::Number(orbit[0] as f64),
                        Value::Number(orbit[1] as f64),
                    ])))
                }
                _ => Err(format!("unknown function 'input.{}'", method_name)),
            }
        }
        "camera" => {
            match method_name {
                "get_horizontal_basis" => {
                    let (f, r) = _host.engine.get_camera_horizontal_basis();
                    Ok(Some(Value::array(vec![
                        Value::array(vec![
                            Value::Number(f[0] as f64),
                            Value::Number(f[1] as f64),
                            Value::Number(f[2] as f64),
                        ]),
                        Value::array(vec![
                            Value::Number(r[0] as f64),
                            Value::Number(r[1] as f64),
                            Value::Number(r[2] as f64),
                        ]),
                    ])))
                }
                "set_position" => {
                    if args.len() == 3 {
                        let x = args[0].as_number()? as f32;
                        let y = args[1].as_number()? as f32;
                        let z = args[2].as_number()? as f32;
                        _host.engine.set_camera_position([x, y, z]);
                        Ok(Some(Value::Nil))
                    } else if args.len() == 1 {
                        let basket = args[0].as_basket()?;
                        let borrowed = basket.borrow();
                        if borrowed.elements.len() == 3 {
                            let x = borrowed.elements[0].as_number()? as f32;
                            let y = borrowed.elements[1].as_number()? as f32;
                            let z = borrowed.elements[2].as_number()? as f32;
                            _host.engine.set_camera_position([x, y, z]);
                            return Ok(Some(Value::Nil));
                        }
                        Err("camera.set_position expects 3 numbers or a 3D array".to_string())
                    } else {
                        Err("camera.set_position expects 3 numbers or a 3D array".to_string())
                    }
                }
                "set_target" => {
                    if args.len() == 3 {
                        let x = args[0].as_number()? as f32;
                        let y = args[1].as_number()? as f32;
                        let z = args[2].as_number()? as f32;
                        _host.engine.set_camera_target([x, y, z]);
                        Ok(Some(Value::Nil))
                    } else if args.len() == 1 {
                        let basket = args[0].as_basket()?;
                        let borrowed = basket.borrow();
                        if borrowed.elements.len() == 3 {
                            let x = borrowed.elements[0].as_number()? as f32;
                            let y = borrowed.elements[1].as_number()? as f32;
                            let z = borrowed.elements[2].as_number()? as f32;
                            _host.engine.set_camera_target([x, y, z]);
                            return Ok(Some(Value::Nil));
                        }
                        Err("camera.set_target expects 3 numbers or a 3D array".to_string())
                    } else {
                        Err("camera.set_target expects 3 numbers or a 3D array".to_string())
                    }
                }
                "set_orientation" => {
                    if args.len() != 2 {
                        return Err("camera.set_orientation expects 2 numbers (yaw, pitch)".to_string());
                    }
                    let yaw = args[0].as_number()? as f32;
                    let pitch = args[1].as_number()? as f32;
                    _host.engine.set_camera_orientation(yaw, pitch);
                    Ok(Some(Value::Nil))
                }
                _ => Err(format!("unknown function 'camera.{}'", method_name)),
            }
        }
        "physics" => {
            match method_name {
                "resolve_camera_collision" => {
                    if args.len() != 2 {
                        return Err("physics.resolve_camera_collision expects target and desired arrays".to_string());
                    }
                    let t_basket = args[0].as_basket()?;
                    let d_basket = args[1].as_basket()?;
                    let tb = t_basket.borrow();
                    let db = d_basket.borrow();
                    if tb.elements.len() != 3 || db.elements.len() != 3 {
                        return Err("target and desired must be 3-element arrays".to_string());
                    }
                    let target = [
                        tb.elements[0].as_number()? as f32,
                        tb.elements[1].as_number()? as f32,
                        tb.elements[2].as_number()? as f32,
                    ];
                    let desired = [
                        db.elements[0].as_number()? as f32,
                        db.elements[1].as_number()? as f32,
                        db.elements[2].as_number()? as f32,
                    ];
                    let actual = _host.engine.resolve_camera_collision(target, desired);
                    Ok(Some(Value::array(vec![
                        Value::Number(actual[0] as f64),
                        Value::Number(actual[1] as f64),
                        Value::Number(actual[2] as f64),
                    ])))
                }
                _ => Err(format!("unknown function 'physics.{}'", method_name)),
            }
        }
        "player" => {
            match method_name {
                "set_horizontal_velocity" => {
                    if args.len() != 2 {
                        return Err("player.set_horizontal_velocity expects 2 numbers (vx, vz)".to_string());
                    }
                    let vx = args[0].as_number()? as f32;
                    let vz = args[1].as_number()? as f32;
                    _host.engine.set_player_horizontal_velocity(vx, vz);
                    Ok(Some(Value::Nil))
                }
                "set_facing_direction" => {
                    if args.len() != 2 {
                        return Err("player.set_facing_direction expects 2 numbers (dx, dz)".to_string());
                    }
                    let dx = args[0].as_number()? as f32;
                    let dz = args[1].as_number()? as f32;
                    _host.engine.set_player_facing_direction(dx, dz);
                    Ok(Some(Value::Nil))
                }
                "select_animation" => {
                    if args.len() != 1 {
                        return Err("player.select_animation expects 1 string".to_string());
                    }
                    let anim = args[0].as_string()?;
                    _host.engine.select_player_animation(anim);
                    Ok(Some(Value::Nil))
                }
                "is_grounded" => Ok(Some(Value::Bool(_host.engine.is_player_grounded()))),
                "apply_vertical_impulse" => {
                    if args.len() != 1 {
                        return Err("player.apply_vertical_impulse expects 1 number".to_string());
                    }
                    let impulse = args[0].as_number()? as f32;
                    _host.engine.apply_player_vertical_impulse(impulse);
                    Ok(Some(Value::Nil))
                }
                _ => Err(format!("unknown function 'player.{}'", method_name)),
            }
        }
        "ui" => {
            match method_name {
                "new" => {
                    if args.len() != 1 {
                        return Err("ui.new expects exactly 1 argument (element_type)".to_string());
                    }
                    let element_type = args[0].as_string()?;
                    let id = _host.engine.create_ui_element(element_type)?;
                    Ok(Some(Value::Handle {
                        kind: HandleKind::Ui,
                        id,
                    }))
                }
                "delete" => {
                    if args.len() != 1 {
                        return Err("ui.delete expects exactly 1 argument (handle)".to_string());
                    }
                    let (kind, id) = args[0].as_handle()?;
                    if kind != HandleKind::Ui {
                        return Err(format!(
                            "ui.delete expects a Ui handle, got {}",
                            kind.name()
                        ));
                    }
                    _host.engine.delete_ui_element(id)?;
                    Ok(Some(Value::Nil))
                }
                _ => Err(format!("unknown function 'ui.{}'", method_name)),
            }
        }
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
        "script" => {
            let res = match method_name {
                "enable" => {
                    if args.len() != 1 {
                        return Err("script.enable expects 1 argument (path)".to_string());
                    }
                    let path = args[0].as_string()?;
                    _host.engine.enable_script(path);
                    Value::Nil
                }
                "disable" => {
                    if args.len() != 1 {
                        return Err("script.disable expects 1 argument (path)".to_string());
                    }
                    let path = args[0].as_string()?;
                    _host.engine.disable_script(path);
                    Value::Nil
                }
                "is_enabled" => {
                    if args.len() != 1 {
                        return Err("script.is_enabled expects 1 argument (path)".to_string());
                    }
                    let path = args[0].as_string()?;
                    let enabled = _host.engine.is_script_enabled(path);
                    Value::Bool(enabled)
                }
                _ => return Ok(None),
            };
            return Ok(Some(res));
        }
        "event" => {
            let res = match method_name {
                "fire" => {
                    if args.is_empty() {
                        return Err(
                            "event.fire expects at least 1 argument (event_name)".to_string()
                        );
                    }
                    let event_name = args[0].as_string()?;
                    let event_args = args[1..].to_vec();
                    _host.engine.fire_event(event_name, event_args);
                    Value::Nil
                }
                _ => return Ok(None),
            };
            return Ok(Some(res));
        }
        "test" => {
            let res = match method_name {
                "complete" => {
                    if args.len() < 2 {
                        return Err(
                            "test.complete expects 2 arguments (test_name, passed)".to_string()
                        );
                    }
                    let test_name = args[0].as_string()?;
                    let passed = args[1].as_bool()?;
                    _host.engine.complete_test(test_name, passed);
                    Value::Nil
                }
                "is_completed" => {
                    if args.len() != 1 {
                        return Err("test.is_completed expects 1 argument (test_name)".to_string());
                    }
                    let test_name = args[0].as_string()?;
                    if let Some(_passed) = _host.engine.is_test_completed(test_name) {
                        Value::Bool(true)
                    } else {
                        Value::Bool(false)
                    }
                }
                "passed" => {
                    if args.len() != 1 {
                        return Err("test.passed expects 1 argument (test_name)".to_string());
                    }
                    let test_name = args[0].as_string()?;
                    if let Some(passed) = _host.engine.is_test_completed(test_name) {
                        Value::Bool(passed)
                    } else {
                        Value::Nil
                    }
                }
                "summary" => {
                    let (p, f, t) = _host.engine.get_test_results();
                    Value::array(vec![
                        Value::Number(p as f64),
                        Value::Number(f as f64),
                        Value::Number(t as f64),
                    ])
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
