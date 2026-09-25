use std::sync::Arc;

use glam::Vec3;

use super::ScriptHostBridge;

impl<'a> ScriptHostBridge<'a> {
    pub fn is_entity_declaration_valid(&self, entity_name: &str) -> bool {
        !entity_name.trim().is_empty()
    }

    pub fn spawn_character(&mut self, entity_name: &str, position: Vec3) -> Result<u64, String> {
        if !self.is_entity_declaration_valid(entity_name) {
            return Err("Entity name cannot be empty".to_string());
        }

        let Some(character_system) = self.character_system.as_deref_mut() else {
            return Err("CharacterSystem not available for spawning".to_string());
        };

        let custom_dir = self
            .project_path
            .map(|p| p.join("characters").join("custom"));
        let package_dir = custom_dir.as_deref().filter(|p| p.exists());

        let character_id = character_system.spawn_character(position, package_dir);
        let entity_id = self.entity_manager.create_entity(entity_name);
        self.entity_manager.set_position(entity_id, position);
        character_system.associate_entity(entity_id, character_id);

        self.pending_spawns
            .push((entity_name.to_string(), entity_id.0));

        Ok(entity_id.0)
    }

    pub fn set_valid_entity_declarations(&mut self, decls: Arc<std::collections::HashSet<String>>) {
        self.valid_entity_declarations = Some(decls);
    }

    pub fn drain_pending_spawns(&mut self) -> Vec<(String, u64)> {
        std::mem::take(self.pending_spawns)
    }

    pub fn get_player_position(&self) -> Option<[f32; 3]> {
        let system = self.character_system.as_deref()?;
        let player = system.get_active_player()?;

        let position = player.transform.position;

        Some([position.x, position.y, position.z])
    }

    pub fn set_player_horizontal_velocity(&mut self, velocity_x: f32, velocity_z: f32) {
        if let Some(system) = self.character_system.as_deref_mut() {
            if let Some(player) = system.get_active_player_mut() {
                player.movement.velocity.x = velocity_x;
                player.movement.velocity.z = velocity_z;
            }
        }
    }

    pub fn set_player_facing_direction(&mut self, direction_x: f32, direction_z: f32) {
        if let Some(system) = self.character_system.as_deref_mut() {
            if let Some(player) = system.get_active_player_mut() {
                let angle = f32::atan2(direction_x, direction_z);

                player.transform.rotation = glam::Quat::from_rotation_y(angle);
            }
        }
    }

    pub fn select_player_animation(&mut self, animation: &str) {
        if let Some(system) = self.character_system.as_deref_mut() {
            if let Some(player) = system.get_active_player_mut() {
                let target_animation = match animation {
                    "Walk" => crate::character_custom::TargetAnimation::Walk,
                    _ => crate::character_custom::TargetAnimation::Idle,
                };

                player
                    .animation_controller
                    .select_animation(target_animation);
            }
        }
    }

    pub fn is_player_grounded(&self) -> bool {
        if let Some(system) = self.character_system.as_deref() {
            if let Some(player) = system.get_active_player() {
                return player.movement.is_grounded;
            }
        }

        true
    }

    pub fn apply_player_vertical_impulse(&mut self, impulse: f32) {
        if let Some(system) = self.character_system.as_deref_mut() {
            if let Some(player) = system.get_active_player_mut() {
                player.movement.velocity.y = impulse;
                player.movement.is_grounded = false;
            }
        }
    }
}
