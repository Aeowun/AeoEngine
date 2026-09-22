use serde::{Deserialize, Serialize};

/// Represents an authored binding between an engine target and a script asset.
///
/// This data model links an authored object identity (such as an entity name)
/// to an AeoScript asset path. It is intended for inclusion in project or
/// world persistence.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScriptBinding {
    /// The unique persistent ID of the authored cell.
    pub target_identity: u64,

    /// The relative path to the script asset (e.g. "scripts/player.aeo").
    pub script_path: String,

    /// Whether this script binding is currently active.
    pub enabled: bool,
}

impl ScriptBinding {
    pub fn new(target_identity: u64, script_path: impl Into<String>) -> Self {
        Self {
            target_identity,
            script_path: script_path.into(),
            enabled: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binding_construction() {
        let binding = ScriptBinding::new(1001, "scripts/player.aeo");
        assert_eq!(binding.target_identity, 1001);
        assert_eq!(binding.script_path, "scripts/player.aeo");
    }

    #[test]
    fn test_binding_equality() {
        let b1 = ScriptBinding::new(1, "path/a.aeo");
        let b2 = ScriptBinding::new(1, "path/a.aeo");
        let b3 = ScriptBinding::new(2, "path/a.aeo");
        let b4 = ScriptBinding::new(1, "path/b.aeo");

        assert_eq!(b1, b2);
        assert_ne!(b1, b3);
        assert_ne!(b1, b4);
    }

    #[test]
    fn test_serialization_round_trip() {
        let original = ScriptBinding::new(12345678, "assets/enemy.aeo");
        let serialized = serde_json::to_string(&original).expect("Should serialize");
        let deserialized: ScriptBinding =
            serde_json::from_str(&serialized).expect("Should deserialize");

        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_multiple_bindings_in_collection() {
        let mut bindings = Vec::new();
        bindings.push(ScriptBinding::new(1001, "script1.aeo"));
        bindings.push(ScriptBinding::new(2002, "script2.aeo"));

        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].target_identity, 1001);
        assert_eq!(bindings[1].target_identity, 2002);

        let serialized = serde_json::to_string(&bindings).unwrap();
        let deserialized: Vec<ScriptBinding> = serde_json::from_str(&serialized).unwrap();

        assert_eq!(bindings, deserialized);
    }
}
