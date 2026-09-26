pub mod character_state;
pub mod entity_control;
pub mod health;
pub mod pathfinding;

#[cfg(test)]
mod system_tests;

use super::animation::AnimationState;
use super::animation::TargetAnimation;
use super::character::Character;
use super::collision::{
    resolve_character_collisions, resolve_dynamic_body_collisions, resolve_static_voxel_collisions,
};
use super::movement::{JUMP_IMPULSE, MOVE_SPEED, MovementState};
use crate::engine::entity::EntityId;
use crate::world::World;
use glam::{Quat, Vec2};
use std::collections::{HashMap, HashSet};

/// Owns all runtime character state.
///
/// CharacterSystem is authoritative for:
/// - character transform
/// - gravity
/// - movement velocity
/// - grounded state
/// - collision resolution
/// - animation state
/// - character health
///
/// AeoScript may control horizontal velocity, facing, animation selection,
/// jumping, health, damage, path queries, and explicit teleports through
/// the EngineHost bridge.
#[derive(Default)]
pub struct CharacterSystem {
    pub(crate) characters: HashMap<u64, Character>,

    /// Maps EntityId to CharacterId.
    pub(crate) entity_to_character: HashMap<EntityId, u64>,

    /// Maps CharacterId to EntityId.
    pub(crate) character_to_entity: HashMap<u64, EntityId>,

    /// Explicit active player identity.
    ///
    /// HashMap iteration order is not a gameplay contract, so runtime player
    /// access should use this ID whenever possible.
    pub(crate) active_player_id: Option<u64>,

    pub(crate) next_id: u64,
}

impl CharacterSystem {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            entity_to_character: HashMap::new(),
            character_to_entity: HashMap::new(),
            active_player_id: None,
            next_id: 1,
        }
    }

    /// Legacy engine-controlled movement path.
    pub fn update(
        &mut self,
        world: &World,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        dt: f32,
        input: Vec2,
        jump_requested: bool,
    ) -> HashSet<u64> {
        self.update_internal(world, physics_world, dt, Some(input), jump_requested, false)
    }

    /// AeoScript-controlled movement path.
    pub fn update_scripted(
        &mut self,
        world: &World,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        dt: f32,
    ) -> HashSet<u64> {
        self.update_internal(world, physics_world, dt, None, false, true)
    }

    fn update_internal(
        &mut self,
        world: &World,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        dt: f32,
        input: Option<Vec2>,
        jump_requested: bool,
        scripted: bool,
    ) -> HashSet<u64> {
        let mut contacted_cells = HashSet::new();

        for character in self.characters.values_mut() {
            if !scripted {
                let horizontal_velocity = match input {
                    Some(input) if input.length_squared() > 0.001 => input.normalize() * MOVE_SPEED,

                    _ => Vec2::ZERO,
                };

                character.movement.velocity.x = horizontal_velocity.x;

                character.movement.velocity.z = horizontal_velocity.y;

                if jump_requested && character.movement.is_grounded {
                    character.movement.velocity.y = JUMP_IMPULSE;

                    character.movement.is_grounded = false;
                }

                // Engine-controlled movement owns facing.
                if horizontal_velocity.length_squared() > 0.001 {
                    let facing = horizontal_velocity.normalize();

                    character.facing = facing;

                    let angle = f32::atan2(facing.x, facing.y);

                    character.transform.rotation = Quat::from_rotation_y(angle);
                }
            }

            // Gravity is always authoritative.
            character.movement.velocity += world.gravity * dt;

            // Preserve the previous position so collision resolution can
            // distinguish vertical crossings from horizontal contacts.
            let previous_position = character.transform.position;

            character.transform.position += character.movement.velocity * dt;

            resolve_static_voxel_collisions(
                character,
                physics_world,
                &mut contacted_cells,
                previous_position,
            );

            resolve_dynamic_body_collisions(
                character,
                physics_world,
                &mut contacted_cells,
                previous_position,
            );

            // Animation follows movement unless explicitly overridden.
            let horizontal_speed =
                Vec2::new(character.movement.velocity.x, character.movement.velocity.z).length();

            let target_animation = character.animation_override.unwrap_or_else(|| {
                if horizontal_speed > 0.1 {
                    TargetAnimation::Walk
                } else {
                    TargetAnimation::Idle
                }
            });

            match target_animation {
                TargetAnimation::Walk => {
                    character.animation.current_state = AnimationState::Walk;

                    character.movement.state = MovementState::Walk;
                }

                TargetAnimation::Idle => {
                    character.animation.current_state = AnimationState::Idle;

                    character.movement.state = MovementState::Idle;
                }
            }

            character
                .animation_controller
                .select_animation(target_animation);

            character.animation_controller.update(dt);

            character.current_pose = character.animation_controller.evaluate_pose();
        }

        resolve_character_collisions(&mut self.characters);

        contacted_cells
    }
}
