use super::CharacterSystem;
use crate::character::animation::AnimationState;
use crate::character::character::Character;
use crate::character_custom::TargetAnimation;
use crate::engine::entity::{EntityId, EntityManager};
use glam::{Quat, Vec2, Vec3};

impl CharacterSystem {
    /// Establishes an explicit mapping between a script EntityId and a
    /// runtime Character ID.
    pub fn associate_entity(&mut self, entity_id: EntityId, character_id: u64) {
        self.entity_to_character.insert(entity_id, character_id);

        self.character_to_entity.insert(character_id, entity_id);
    }

    /// Returns the character associated with a specific script EntityId.
    pub fn get_character_for_entity(&self, entity_id: EntityId) -> Option<&Character> {
        let character_id = self.entity_to_character.get(&entity_id)?;

        self.characters.get(character_id)
    }

    /// Returns the character associated with a specific script EntityId
    /// mutably.
    pub fn get_character_for_entity_mut(&mut self, entity_id: EntityId) -> Option<&mut Character> {
        let character_id = self.entity_to_character.get(&entity_id)?;

        self.characters.get_mut(character_id)
    }

    /// Returns the character ID associated with a specific script EntityId.
    pub fn get_character_id_for_entity(&self, entity_id: EntityId) -> Option<u64> {
        self.entity_to_character.get(&entity_id).copied()
    }

    /// Returns the script EntityId associated with a specific character ID.
    pub fn get_entity_for_character(&self, character_id: u64) -> Option<EntityId> {
        self.character_to_entity.get(&character_id).copied()
    }

    /// Sets the complete runtime velocity of a character.
    pub fn set_entity_velocity(&mut self, entity_id: EntityId, velocity: Vec3) -> bool {
        let Some(character) = self.get_character_for_entity_mut(entity_id) else {
            return false;
        };

        character.movement.velocity = velocity;
        true
    }

    /// Sets only horizontal velocity.
    pub fn set_entity_horizontal_velocity(&mut self, entity_id: EntityId, x: f32, z: f32) -> bool {
        let Some(character) = self.get_character_for_entity_mut(entity_id) else {
            return false;
        };

        character.movement.velocity.x = x;
        character.movement.velocity.z = z;

        true
    }

    /// Returns the current runtime velocity.
    pub fn get_entity_velocity(&self, entity_id: EntityId) -> Option<Vec3> {
        self.get_character_for_entity(entity_id)
            .map(|character| character.movement.velocity)
    }

    /// Sets the character's horizontal facing direction.
    pub fn set_entity_facing_direction(&mut self, entity_id: EntityId, x: f32, z: f32) -> bool {
        let direction = Vec2::new(x, z);

        if direction.length_squared() <= 0.000001 {
            return false;
        }

        let direction = direction.normalize();

        let Some(character) = self.get_character_for_entity_mut(entity_id) else {
            return false;
        };

        character.facing = direction;

        let angle = f32::atan2(direction.x, direction.y);

        character.transform.rotation = Quat::from_rotation_y(angle);

        true
    }

    /// Returns the current horizontal facing direction.
    pub fn get_entity_facing(&self, entity_id: EntityId) -> Option<Vec2> {
        self.get_character_for_entity(entity_id)
            .map(|character| character.facing)
    }

    /// Explicitly selects an animation.
    ///
    /// `Auto` returns control to movement-driven animation.
    pub fn set_entity_animation(
        &mut self,
        entity_id: EntityId,
        animation: &str,
    ) -> Result<bool, String> {
        let Some(character) = self.get_character_for_entity_mut(entity_id) else {
            return Ok(false);
        };

        character.animation_override = match animation {
            "Auto" => None,

            "Idle" => Some(TargetAnimation::Idle),

            "Walk" => Some(TargetAnimation::Walk),

            _ => {
                return Err(format!("unsupported character animation '{}'", animation));
            }
        };

        Ok(true)
    }

    /// Returns the currently active animation state.
    pub fn get_entity_animation(&self, entity_id: EntityId) -> Option<&'static str> {
        let character = self.get_character_for_entity(entity_id)?;

        Some(match character.animation.current_state {
            AnimationState::Idle => "Idle",
            AnimationState::Walk => "Walk",
        })
    }

    /// Returns whether the character is currently grounded.
    pub fn is_entity_grounded(&self, entity_id: EntityId) -> Option<bool> {
        self.get_character_for_entity(entity_id)
            .map(|character| character.movement.is_grounded)
    }

    /// Applies a jump impulse only when grounded.
    pub fn jump_entity(&mut self, entity_id: EntityId, impulse: f32) -> bool {
        let Some(character) = self.get_character_for_entity_mut(entity_id) else {
            return false;
        };

        if !character.movement.is_grounded {
            return false;
        }

        character.movement.velocity.y = impulse;
        character.movement.is_grounded = false;

        true
    }

    /// Explicitly sets the position of a character associated with a script
    /// EntityId.
    ///
    /// Returns true if an associated character was moved, or false if no
    /// mapping exists.
    pub fn set_entity_position(&mut self, entity_id: EntityId, position: Vec3) -> bool {
        if let Some(&character_id) = self.entity_to_character.get(&entity_id) {
            if let Some(character) = self.characters.get_mut(&character_id) {
                character.transform.position = position;

                character.movement.velocity = Vec3::ZERO;

                character.movement.is_grounded = false;

                return true;
            }
        }

        false
    }

    /// Synchronizes all runtime character transforms back to their
    /// corresponding EntityManager entities.
    pub fn sync_entity_positions(&self, entity_manager: &mut EntityManager) {
        for (&entity_id, &character_id) in &self.entity_to_character {
            if let Some(character) = self.characters.get(&character_id) {
                entity_manager.set_position(entity_id, character.transform.position);
            }
        }

        if let Some(player_id) = entity_manager.lookup_entity("Player") {
            if !self.entity_to_character.contains_key(&player_id) {
                if let Some(player) = self.get_active_player() {
                    entity_manager.set_position(player_id, player.transform.position);
                }
            }
        }
    }

    /// Destroys a mapped runtime character.
    ///
    /// EntityManager removal is handled by the scripting host because
    /// CharacterSystem does not own the EntityManager registry.
    pub fn destroy_entity(&mut self, entity_id: EntityId) -> bool {
        let Some(character_id) = self.entity_to_character.remove(&entity_id) else {
            return false;
        };

        self.character_to_entity.remove(&character_id);

        if self.active_player_id == Some(character_id) {
            self.active_player_id = None;
        }

        self.characters.remove(&character_id).is_some()
    }
}
