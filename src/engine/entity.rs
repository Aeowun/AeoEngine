use std::collections::HashMap;
use glam::Vec3;

/// Unique runtime identifier for an entity during a single Play session.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntityId(pub u64);

/// Record for a runtime entity.
pub struct RuntimeEntity {
    pub id: EntityId,
    pub name: String,
    pub position: Vec3,
}

/// Registry for runtime entity identity.
///
/// This registry is valid only for the duration of a Play session. It is
/// intentionally discarded when returning to the Editor.
pub struct EntityManager {
    entities: HashMap<EntityId, RuntimeEntity>,
    name_to_id: HashMap<String, EntityId>,
    next_id: u64,
}

impl EntityManager {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            name_to_id: HashMap::new(),
            next_id: 1,
        }
    }

    /// Creates a new runtime entity and returns its ID.
    pub fn create_entity(&mut self, name: &str) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id += 1;

        let entity = RuntimeEntity {
            id,
            name: name.to_string(),
            position: Vec3::ZERO,
        };

        self.entities.insert(id, entity);
        self.name_to_id.insert(name.to_string(), id);

        id
    }

    /// Looks up an entity ID by its script-facing name.
    pub fn lookup_entity(&self, name: &str) -> Option<EntityId> {
        self.name_to_id.get(name).copied()
    }

    /// Validates that a handle ID refers to a currently active runtime entity.
    pub fn validate_handle(&self, id: u64) -> bool {
        self.entities.contains_key(&EntityId(id))
    }

    /// Returns the name of an entity if it exists.
    pub fn get_name(&self, id: EntityId) -> Option<&str> {
        self.entities.get(&id).map(|e| e.name.as_str())
    }

    /// Returns the position of an entity if it exists.
    pub fn get_position(&self, id: EntityId) -> Option<Vec3> {
        self.entities.get(&id).map(|e| e.position)
    }

    /// Sets the position of an entity if it exists.
    pub fn set_position(&mut self, id: EntityId, position: Vec3) {
        if let Some(entity) = self.entities.get_mut(&id) {
            entity.position = position;
        }
    }

    /// Removes an entity from the registry.
    pub fn remove_entity(&mut self, id: EntityId) {
        if let Some(entity) = self.entities.remove(&id) {
            self.name_to_id.remove(&entity.name);
        }
    }

    /// Clears the registry.
    pub fn clear(&mut self) {
        self.entities.clear();
        self.name_to_id.clear();
        self.next_id = 1;
    }
}

impl Default for EntityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_starts_empty() {
        let manager = EntityManager::new();
        assert!(manager.entities.is_empty());
    }

    #[test]
    fn creating_entity_produces_unique_id() {
        let mut manager = EntityManager::new();
        let id1 = manager.create_entity("Entity1");
        let id2 = manager.create_entity("Entity2");
        assert_ne!(id1, id2);
    }

    #[test]
    fn entity_can_be_looked_up_by_name() {
        let mut manager = EntityManager::new();
        let id = manager.create_entity("Test");
        assert_eq!(manager.lookup_entity("Test"), Some(id));
    }

    #[test]
    fn valid_handle_validates_successfully() {
        let mut manager = EntityManager::new();
        let id = manager.create_entity("Test");
        assert!(manager.validate_handle(id.0));
    }

    #[test]
    fn removing_entity_invalidates_handle() {
        let mut manager = EntityManager::new();
        let id = manager.create_entity("Test");
        manager.remove_entity(id);
        assert!(!manager.validate_handle(id.0));
        assert_eq!(manager.lookup_entity("Test"), None);
    }

    #[test]
    fn nonexistent_entity_lookup_fails_safely() {
        let manager = EntityManager::new();
        assert_eq!(manager.lookup_entity("Missing"), None);
        assert!(!manager.validate_handle(999));
    }

    #[test]
    fn test_single_identity_mapping() {
        let mut manager = EntityManager::new();
        let id = manager.create_entity("Player");
        assert_eq!(manager.lookup_entity("Player"), Some(id));
    }

    #[test]
    fn test_multiple_identities_mapping() {
        let mut manager = EntityManager::new();
        let id_player = manager.create_entity("Player");
        let id_door = manager.create_entity("Door");
        let id_enemy = manager.create_entity("Enemy");

        assert_eq!(manager.lookup_entity("Player"), Some(id_player));
        assert_eq!(manager.lookup_entity("Door"), Some(id_door));
        assert_eq!(manager.lookup_entity("Enemy"), Some(id_enemy));

        assert_ne!(id_player, id_door);
        assert_ne!(id_player, id_enemy);
        assert_ne!(id_door, id_enemy);
    }

    #[test]
    fn test_duplicate_identity_behavior() {
        let mut manager = EntityManager::new();
        let id1 = manager.create_entity("Duplicate");
        let id2 = manager.create_entity("Duplicate");

        // Existing behavior: names are 1-to-1 with IDs in lookup, last one wins.
        // Both entities exist, but only the newest is reachable by name.
        assert_ne!(id1, id2);
        assert_eq!(manager.lookup_entity("Duplicate"), Some(id2));

        assert!(manager.validate_handle(id1.0));
        assert!(manager.validate_handle(id2.0));
    }

    #[test]
    fn test_session_isolation() {
        let mut manager = EntityManager::new();
        let id_session1 = manager.create_entity("Player");
        assert_eq!(manager.lookup_entity("Player"), Some(id_session1));

        // End session
        manager.clear();
        assert_eq!(manager.lookup_entity("Player"), None);
        assert!(!manager.validate_handle(id_session1.0));

        // Start new session
        let id_session2 = manager.create_entity("Player");
        assert_eq!(manager.lookup_entity("Player"), Some(id_session2));

        // IDs might be the same if counter resets, but it's a fresh state.
        // In current implementation next_id resets to 1.
        assert_eq!(id_session1, id_session2);
    }

    #[test]
    fn fresh_registry_does_not_validate_old_handles() {
        let mut manager = EntityManager::new();
        let id = manager.create_entity("Test");

        manager.clear();
        assert!(!manager.validate_handle(id.0));

        let new_id = manager.create_entity("Test");
        // IDs might overlap if we reset the counter, but validate_handle checks the map.
        // If we reset next_id to 1, new_id might be the same as id.
        // However, the registry is cleared, so old entries are gone.
    }
}
