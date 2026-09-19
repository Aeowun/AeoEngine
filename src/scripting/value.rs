use std::collections::BTreeMap;

/// Kinds of opaque engine handles.
///
/// Handles are intentionally lightweight. The scripting runtime does not own
/// the underlying engine object; it only carries an identifier that the host
/// runtime can resolve later.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HandleKind {
    Entity,
    Light,
    Cell,
}

impl HandleKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Entity => "Entity",
            Self::Light => "Light",
            Self::Cell => "Cell",
        }
    }
}

/// Values that can exist inside the AeoScript runtime.
#[derive(Clone, Debug)]
pub enum Value {
    Number(f64),
    Bool(bool),
    String(String),
    Nil,
    Array(Vec<Value>),
    Map(BTreeMap<String, Value>),

    /// Opaque engine object reference.
    ///
    /// The scripting VM never receives a pointer or a Rust object directly.
    Handle {
        kind: HandleKind,
        id: u64,
    },
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Number(_) => "number",
            Self::Bool(_) => "bool",
            Self::String(_) => "string",
            Self::Nil => "nil",
            Self::Array(_) => "basket",
            Self::Map(_) => "map",
            Self::Handle { .. } => "handle",
        }
    }

    /// AeoScript conditions are strict for booleans but nullable values are
    /// naturally useful in gameplay code:
    ///
    ///     if light { ... }
    ///
    /// becomes true for a valid handle and false for nil.
    pub fn is_truthy(&self) -> Result<bool, String> {
        match self {
            Self::Bool(value) => Ok(*value),
            Self::Nil => Ok(false),
            Self::Handle { .. } => Ok(true),
            other => Err(format!(
                "expected bool or optional value in condition, got {}",
                other.type_name()
            )),
        }
    }

    pub fn as_number(&self) -> Result<f64, String> {
        match self {
            Self::Number(value) => Ok(*value),
            other => Err(format!("expected number, got {}", other.type_name())),
        }
    }

    pub fn as_bool(&self) -> Result<bool, String> {
        match self {
            Self::Bool(value) => Ok(*value),
            other => Err(format!("expected bool, got {}", other.type_name())),
        }
    }

    pub fn as_string(&self) -> Result<&str, String> {
        match self {
            Self::String(value) => Ok(value.as_str()),
            other => Err(format!("expected string, got {}", other.type_name())),
        }
    }

    pub fn as_basket(&self) -> Result<&Vec<Value>, String> {
        match self {
            Self::Array(value) => Ok(value),
            other => Err(format!("expected basket, got {}", other.type_name())),
        }
    }

    pub fn as_handle(&self) -> Result<(HandleKind, u64), String> {
        match self {
            Self::Handle { kind, id } => Ok((*kind, *id)),
            other => Err(format!("expected engine handle, got {}", other.type_name())),
        }
    }

    pub fn display_string(&self) -> String {
        match self {
            Self::Number(value) => {
                if value.fract() == 0.0 {
                    format!("{:.0}", value)
                } else {
                    value.to_string()
                }
            }

            Self::Bool(value) => value.to_string(),

            Self::String(value) => value.clone(),

            Self::Nil => "nil".to_string(),

            Self::Array(values) => {
                let items = values
                    .iter()
                    .map(Value::display_string)
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("[{}]", items)
            }

            Self::Map(values) => {
                let items = values
                    .iter()
                    .map(|(key, value)| format!("{}: {}", key, value.display_string()))
                    .collect::<Vec<_>>()
                    .join(", ");

                format!("{{{}}}", items)
            }

            Self::Handle { kind, id } => {
                format!("{}#{}", kind.name(), id)
            }
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Nil, Self::Nil) => true,
            (Self::Array(a), Self::Array(b)) => a == b,
            (Self::Map(a), Self::Map(b)) => a == b,

            (
                Self::Handle {
                    kind: left_kind,
                    id: left_id,
                },
                Self::Handle {
                    kind: right_kind,
                    id: right_id,
                },
            ) => left_kind == right_kind && left_id == right_id,

            _ => false,
        }
    }
}

impl Eq for Value {}

impl std::fmt::Display for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.display_string())
    }
}

/// A single lexical scope.
#[derive(Clone, Debug, Default)]
pub struct Scope {
    values: BTreeMap<String, Value>,
    constants: BTreeMap<String, ()>,
}

impl Scope {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn declare(
        &mut self,
        name: impl Into<String>,
        value: Value,
        is_const: bool,
    ) -> Result<(), String> {
        let name = name.into();

        if self.values.contains_key(&name) {
            return Err(format!(
                "variable '{}' is already declared in this scope",
                name
            ));
        }

        self.values.insert(name.clone(), value);

        if is_const {
            self.constants.insert(name, ());
        }

        Ok(())
    }

    pub fn contains(&self, name: &str) -> bool {
        self.values.contains_key(name)
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn set(&mut self, name: &str, value: Value) -> Result<(), String> {
        if self.constants.contains_key(name) {
            return Err(format!("cannot assign to const variable '{}'", name));
        }

        match self.values.get_mut(name) {
            Some(existing) => {
                *existing = value;
                Ok(())
            }

            None => Err(format!("variable '{}' does not exist in this scope", name)),
        }
    }

    pub fn is_const(&self, name: &str) -> bool {
        self.constants.contains_key(name)
    }
}

#[cfg(test)]
mod tests {
    use super::{HandleKind, Scope, Value};
    use std::collections::BTreeMap;

    #[test]
    fn value_types_are_reported() {
        assert_eq!(Value::Number(10.0).type_name(), "number");

        assert_eq!(Value::Bool(true).type_name(), "bool");

        assert_eq!(Value::String("hello".to_string()).type_name(), "string");

        assert_eq!(Value::Nil.type_name(), "nil");

        assert_eq!(Value::Array(vec![Value::Number(1.0)]).type_name(), "basket");

        assert_eq!(Value::Map(BTreeMap::new()).type_name(), "map");

        assert_eq!(
            Value::Handle {
                kind: HandleKind::Light,
                id: 7,
            }
            .type_name(),
            "handle"
        );
    }

    #[test]
    fn handle_values_display_cleanly() {
        let value = Value::Handle {
            kind: HandleKind::Light,
            id: 42,
        };

        assert_eq!(value.display_string(), "Light#42");

        assert_eq!(
            value.as_handle().expect("handle should resolve"),
            (HandleKind::Light, 42)
        );
    }

    #[test]
    fn nil_is_falsey() {
        assert_eq!(
            Value::Nil.is_truthy().expect("nil should be valid"),
            false
        );
    }

    #[test]
    fn handles_are_truthy() {
        let value = Value::Handle {
            kind: HandleKind::Entity,
            id: 1,
        };

        assert_eq!(value.is_truthy().expect("handle should be valid"), true);
    }

    #[test]
    fn non_boolean_non_optional_values_are_rejected_as_conditions() {
        assert!(Value::Number(1.0).is_truthy().is_err());

        assert!(Value::String("yes".to_string()).is_truthy().is_err());
    }

    #[test]
    fn arrays_and_maps_display() {
        let array = Value::Array(vec![
            Value::Number(1.0),
            Value::Bool(true),
            Value::String("x".to_string()),
        ]);

        assert_eq!(array.display_string(), "[1, true, x]");

        let mut map = BTreeMap::new();

        map.insert("hp".to_string(), Value::Number(100.0));

        map.insert("alive".to_string(), Value::Bool(true));

        let map = Value::Map(map);

        assert_eq!(map.display_string(), "{alive: true, hp: 100}");
    }

    #[test]
    fn scope_declares_and_updates_variables() {
        let mut scope = Scope::new();

        scope
            .declare("value", Value::Number(10.0), false)
            .expect("variable should declare");

        assert!(scope.contains("value"));

        assert_eq!(scope.get("value"), Some(&Value::Number(10.0)));

        scope
            .set("value", Value::Number(15.0))
            .expect("variable should update");

        assert_eq!(scope.get("value"), Some(&Value::Number(15.0)));
    }

    #[test]
    fn const_variables_cannot_change() {
        let mut scope = Scope::new();

        scope
            .declare("limit", Value::Number(10.0), true)
            .expect("const should declare");

        let result = scope.set("limit", Value::Number(20.0));

        assert!(result.is_err());
        assert!(scope.is_const("limit"));

        assert_eq!(scope.get("limit"), Some(&Value::Number(10.0)));
    }

    #[test]
    fn duplicate_declarations_are_rejected() {
        let mut scope = Scope::new();

        scope
            .declare("value", Value::Number(10.0), false)
            .expect("first declaration should work");

        let result = scope.declare("value", Value::Number(20.0), false);

        assert!(result.is_err());
    }
}
