use glam::Vec3;

use crate::engine::entity::{EntityId, EntityManager};
use crate::scripting::value::HandleKind;
use crate::world::CellType;

use super::ScriptHostBridge;

impl<'a> ScriptHostBridge<'a> {
    pub fn entity_manager_ref(&self) -> &EntityManager {
        self.entity_manager
    }

    pub fn get_position(&self, id: u64) -> Option<Vec3> {
        let entity_id = EntityId(id);

        // CharacterSystem is authoritative for runtime character position.
        if let Some(character_system) = self.character_system.as_deref() {
            if let Some(character) = character_system
                .get_active_characters()
                .find(|character| character.id == id)
            {
                return Some(character.transform.position);
            }
        }

        // Non-character entities use the script-facing EntityManager state.
        self.entity_manager.get_position(entity_id)
    }

    /// Handles the existing generic AeoScript `entity.set_position()` API.
    ///
    /// This is the ONLY place where a script-requested entity position change
    /// is forwarded into CharacterSystem.
    ///
    /// Normal character movement never comes through this function.
    pub fn set_position(&mut self, id: u64, position: Vec3) {
        let entity_id = EntityId(id);

        // Always update the script-facing entity representation.
        self.entity_manager.set_position(entity_id, position);

        // An entity explicitly calling set_position() is a deliberate
        // teleport request. Forward that operation to the authoritative
        // CharacterSystem representation.
        if let Some(character_system) = self.character_system.as_deref_mut() {
            character_system.set_entity_position(entity_id, position);
        }
    }

    pub fn entity_exists(&self, id: u64) -> bool {
        self.entity_manager.validate_handle(id)
    }

    pub fn entity_character_id(&self, entity_id: EntityId) -> Option<u64> {
        self.character_system.as_deref().and_then(|system| {
            system
                .get_active_characters()
                .find(|character| character.id == entity_id.0)
                .map(|character| character.id)
        })
    }

    pub fn get_children(&self, _kind: HandleKind, _id: u64) -> Vec<(HandleKind, u64)> {
        Vec::new()
    }

    pub fn get_parent(&self, _kind: HandleKind, _id: u64) -> Option<(HandleKind, u64)> {
        None
    }

    pub fn get_cell_object(&self, cell_id: u64) -> Option<(HandleKind, u64)> {
        let coord = self.world.resolve_cell_id(cell_id)?;

        let cell = self.world.get_effective_cell(coord)?;

        let identity = cell.entity_identity.as_ref()?;

        let entity_id = self.entity_manager.lookup_entity(identity)?;

        Some((HandleKind::Entity, entity_id.0))
    }

    pub fn find_objects(&self, query: &str) -> Vec<(HandleKind, u64)> {
        let mut results = Vec::new();

        if let Some(id) = self.entity_manager.lookup_entity(query) {
            results.push((HandleKind::Entity, id.0));
        }

        // Search authored/runtime cells by their script-facing identity.
        for coord in self.world.iter_active_effective_coords() {
            if let Some(cell) = self.world.get_effective_cell(coord) {
                if let Some(identity) = &cell.entity_identity {
                    if identity == query {
                        let kind = match cell.cell_type {
                            CellType::Light => HandleKind::Light,
                            _ => HandleKind::Cell,
                        };

                        results.push((kind, cell.id));
                    }
                }
            }
        }

        results
    }
}
