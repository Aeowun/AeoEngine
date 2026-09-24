use crate::scripting::value::Value;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct UiCommon {
    pub position: [f32; 2],
    pub size: [f32; 2],
    pub visible: bool,
    pub enabled: bool,
    pub color: [f32; 4],
}

impl Default for UiCommon {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0],
            size: [100.0, 50.0],
            visible: true,
            enabled: true,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct UiText {
    pub common: UiCommon,
    pub text: String,
}

#[derive(Clone, Debug)]
pub struct UiButton {
    pub common: UiCommon,
    pub text: String,
    pub on_click: Option<Value>,
}

#[derive(Clone, Debug)]
pub enum UiElement {
    Panel(UiCommon),
    Text(UiText),
    Button(UiButton),
}

impl UiElement {
    pub fn common(&self) -> &UiCommon {
        match self {
            Self::Panel(c) => c,
            Self::Text(t) => &t.common,
            Self::Button(b) => &b.common,
        }
    }

    pub fn common_mut(&mut self) -> &mut UiCommon {
        match self {
            Self::Panel(c) => c,
            Self::Text(t) => &mut t.common,
            Self::Button(b) => &mut b.common,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct RuntimeUi {
    next_id: u32,
    pub elements: HashMap<u32, UiElement>,
    pub pending_clicks: Vec<u32>,
}

impl RuntimeUi {
    pub fn new() -> Self {
        Self {
            next_id: 1,
            elements: HashMap::new(),
            pending_clicks: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.next_id = 1;
        self.elements.clear();
        self.pending_clicks.clear();
    }

    pub fn allocate(&mut self, element_type: &str) -> Result<u64, String> {
        let mut common = UiCommon::default();
        let element = match element_type {
            "Panel" => {
                common.color = [0.12, 0.12, 0.15, 0.85];
                UiElement::Panel(common)
            }
            "Text" => UiElement::Text(UiText {
                common,
                text: String::new(),
            }),
            "Button" => UiElement::Button(UiButton {
                common,
                text: String::new(),
                on_click: None,
            }),
            unknown => return Err(format!("unknown UI element type '{}'", unknown)),
        };

        let id = self.next_id;
        self.next_id += 1;
        self.elements.insert(id, element);
        Ok(id as u64)
    }

    pub fn delete(&mut self, id: u64) -> Result<(), String> {
        let key = id as u32;
        if self.elements.remove(&key).is_some() {
            self.pending_clicks.retain(|&click_id| click_id != key);
            Ok(())
        } else {
            Err(format!("invalid UI handle {}", id))
        }
    }

    pub fn get_property(&self, id: u64, name: &str) -> Result<Option<Value>, String> {
        let key = id as u32;
        let Some(element) = self.elements.get(&key) else {
            return Err(format!("invalid UI handle {}", id));
        };

        let common = element.common();
        match name {
            "position" => Ok(Some(Value::array(vec![
                Value::Number(common.position[0] as f64),
                Value::Number(common.position[1] as f64),
            ]))),
            "size" => Ok(Some(Value::array(vec![
                Value::Number(common.size[0] as f64),
                Value::Number(common.size[1] as f64),
            ]))),
            "visible" => Ok(Some(Value::Bool(common.visible))),
            "enabled" => Ok(Some(Value::Bool(common.enabled))),
            "color" => Ok(Some(Value::array(vec![
                Value::Number(common.color[0] as f64),
                Value::Number(common.color[1] as f64),
                Value::Number(common.color[2] as f64),
                Value::Number(common.color[3] as f64),
            ]))),
            "text" => match element {
                UiElement::Text(t) => Ok(Some(Value::String(t.text.clone()))),
                UiElement::Button(b) => Ok(Some(Value::String(b.text.clone()))),
                UiElement::Panel(_) => Err("property 'text' is not valid on Panel".to_string()),
            },
            "on_click" => match element {
                UiElement::Button(b) => Ok(Some(b.on_click.clone().unwrap_or(Value::Nil))),
                _ => Err(format!(
                    "property 'on_click' is not valid on {}",
                    element_type_name(element)
                )),
            },
            _ => Err(format!("unknown property '{}' on UI element", name)),
        }
    }

    pub fn set_property(&mut self, id: u64, name: &str, value: Value) -> Result<bool, String> {
        let key = id as u32;
        let Some(element) = self.elements.get_mut(&key) else {
            return Err(format!("invalid UI handle {}", id));
        };

        match name {
            "position" => {
                let basket = value.as_basket()?;
                let borrowed = basket.borrow();
                if borrowed.elements.len() != 2 {
                    return Err("position expects an array of 2 numbers [x, y]".to_string());
                }
                let x = borrowed.elements[0].as_number()? as f32;
                let y = borrowed.elements[1].as_number()? as f32;
                element.common_mut().position = [x, y];
                Ok(true)
            }
            "size" => {
                let basket = value.as_basket()?;
                let borrowed = basket.borrow();
                if borrowed.elements.len() != 2 {
                    return Err("size expects an array of 2 numbers [width, height]".to_string());
                }
                let w = borrowed.elements[0].as_number()? as f32;
                let h = borrowed.elements[1].as_number()? as f32;
                element.common_mut().size = [w, h];
                Ok(true)
            }
            "visible" => {
                element.common_mut().visible = value.as_bool()?;
                Ok(true)
            }
            "enabled" => {
                element.common_mut().enabled = value.as_bool()?;
                Ok(true)
            }
            "color" => {
                let basket = value.as_basket()?;
                let borrowed = basket.borrow();
                if borrowed.elements.len() != 4 {
                    return Err("color expects an array of 4 numbers [r, g, b, a]".to_string());
                }
                let r = borrowed.elements[0].as_number()? as f32;
                let g = borrowed.elements[1].as_number()? as f32;
                let b = borrowed.elements[2].as_number()? as f32;
                let a = borrowed.elements[3].as_number()? as f32;
                element.common_mut().color = [r, g, b, a];
                Ok(true)
            }
            "text" => match element {
                UiElement::Text(t) => {
                    t.text = value.as_string()?.to_string();
                    Ok(true)
                }
                UiElement::Button(b) => {
                    b.text = value.as_string()?.to_string();
                    Ok(true)
                }
                UiElement::Panel(_) => Err("property 'text' is not valid on Panel".to_string()),
            },
            "on_click" => match element {
                UiElement::Button(b) => {
                    match value {
                        Value::Function(..) => b.on_click = Some(value),
                        Value::Nil => b.on_click = None,
                        other => {
                            return Err(format!(
                                "on_click expects a function or nil, got {}",
                                other.type_name()
                            ));
                        }
                    }
                    Ok(true)
                }
                _ => Err(format!(
                    "property 'on_click' is not valid on {}",
                    element_type_name(element)
                )),
            },
            _ => Err(format!("unknown property '{}' on UI element", name)),
        }
    }

    pub fn drain_pending_clicks(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.pending_clicks)
            .into_iter()
            .map(|id| id as u64)
            .collect()
    }
}

fn element_type_name(element: &UiElement) -> &'static str {
    match element {
        UiElement::Panel(_) => "Panel",
        UiElement::Text(_) => "Text",
        UiElement::Button(_) => "Button",
    }
}
