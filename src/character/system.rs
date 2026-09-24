use super::animation::AnimationState;
use super::character::Character;
use super::movement::{JUMP_IMPULSE, MOVE_SPEED, MovementState};
use super::spawning::spawn_at_random_point;
use crate::character_custom::TargetAnimation;
use crate::engine::entity::{EntityId, EntityManager};
use crate::world::{World, WorldCoord};
use glam::{Quat, Vec2, Vec3};
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet};
use std::path::Path;

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
    characters: HashMap<u64, Character>,

    /// Maps EntityId to CharacterId.
    entity_to_character: HashMap<EntityId, u64>,

    /// Maps CharacterId to EntityId.
    character_to_entity: HashMap<u64, EntityId>,

    /// Explicit active player identity.
    ///
    /// HashMap iteration order is not a gameplay contract, so runtime player
    /// access should use this ID whenever possible.
    active_player_id: Option<u64>,

    next_id: u64,
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

    /// Spawns a generic character at an explicitly supplied world position.
    pub fn spawn_character(&mut self, position: Vec3, package_dir: Option<&Path>) -> u64 {
        let id = self.next_id;
        self.next_id += 1;

        let character = Character::new_from_package(id, position, package_dir);

        self.characters.insert(id, character);

        id
    }

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

    /// Returns current health.
    pub fn get_entity_health(&self, entity_id: EntityId) -> Option<f32> {
        self.get_character_for_entity(entity_id)
            .map(|character| character.health)
    }

    /// Returns maximum health.
    pub fn get_entity_max_health(&self, entity_id: EntityId) -> Option<f32> {
        self.get_character_for_entity(entity_id)
            .map(|character| character.max_health)
    }

    /// Sets current health, clamped to [0, max_health].
    pub fn set_entity_health(&mut self, entity_id: EntityId, health: f32) -> bool {
        let Some(character) = self.get_character_for_entity_mut(entity_id) else {
            return false;
        };

        character.health = health.clamp(0.0, character.max_health);

        true
    }

    /// Sets maximum health and clamps current health if necessary.
    pub fn set_entity_max_health(&mut self, entity_id: EntityId, max_health: f32) -> bool {
        let Some(character) = self.get_character_for_entity_mut(entity_id) else {
            return false;
        };

        character.max_health = max_health.max(1.0);

        character.health = character.health.min(character.max_health);

        true
    }

    /// Applies damage and returns the resulting health.
    pub fn damage_entity(&mut self, entity_id: EntityId, amount: f32) -> Option<f32> {
        let character = self.get_character_for_entity_mut(entity_id)?;

        character.health = (character.health - amount.max(0.0)).max(0.0);

        Some(character.health)
    }

    /// Heals the character and returns the resulting health.
    pub fn heal_entity(&mut self, entity_id: EntityId, amount: f32) -> Option<f32> {
        let character = self.get_character_for_entity_mut(entity_id)?;

        character.health = (character.health + amount.max(0.0)).min(character.max_health);

        Some(character.health)
    }

    /// Returns whether the character is alive.
    pub fn is_entity_alive(&self, entity_id: EntityId) -> Option<bool> {
        self.get_character_for_entity(entity_id)
            .map(|character| character.health > 0.0)
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

    /// Destroys the active player character.
    pub fn destroy_active_player(&mut self) -> bool {
        let Some(character_id) = self.active_player_id.take() else {
            return false;
        };

        if let Some(entity_id) = self.character_to_entity.remove(&character_id) {
            self.entity_to_character.remove(&entity_id);
        }

        self.characters.remove(&character_id).is_some()
    }

    /// Returns a navigation path from the character to the target position.
    ///
    /// The result contains world-space voxel-center waypoints.
    /// AeoScript is responsible for deciding how to follow the path.
    pub fn find_path_for_entity(
        &self,
        entity_id: EntityId,
        world: &World,
        target: Vec3,
    ) -> Option<Vec<Vec3>> {
        let character = self.get_character_for_entity(entity_id)?;

        let start = (
            character.transform.position.x.round() as i32,
            character.transform.position.y.round() as i32,
            character.transform.position.z.round() as i32,
        );

        let goal = (
            target.x.round() as i32,
            target.y.round() as i32,
            target.z.round() as i32,
        );

        if start == goal {
            return Some(Vec::new());
        }

        let active = world.active_blocks();

        if active.is_empty() {
            return None;
        }

        let mut min_x = start.0.min(goal.0);
        let mut max_x = start.0.max(goal.0);
        let mut min_y = start.1.min(goal.1);
        let mut max_y = start.1.max(goal.1);
        let mut min_z = start.2.min(goal.2);
        let mut max_z = start.2.max(goal.2);

        for coord in active {
            min_x = min_x.min(coord.x);
            max_x = max_x.max(coord.x);
            min_y = min_y.min(coord.y);
            max_y = max_y.max(coord.y);
            min_z = min_z.min(coord.z);
            max_z = max_z.max(coord.z);
        }

        const PADDING: i32 = 2;
        const MAX_NODES: usize = 8192;

        min_x -= PADDING;
        max_x += PADDING;
        min_y -= PADDING;
        max_y += PADDING;
        min_z -= PADDING;
        max_z += PADDING;

        let mut open: BinaryHeap<(Reverse<i32>, Reverse<i32>, (i32, i32, i32))> = BinaryHeap::new();

        let mut came_from: HashMap<(i32, i32, i32), (i32, i32, i32)> = HashMap::new();

        let mut cost_so_far: HashMap<(i32, i32, i32), i32> = HashMap::new();

        let mut closed: HashSet<(i32, i32, i32)> = HashSet::new();

        let start_cost = 0;
        let start_heuristic = Self::path_heuristic(start, goal);

        cost_so_far.insert(start, start_cost);

        open.push((
            Reverse(start_cost + start_heuristic),
            Reverse(start_cost),
            start,
        ));

        while let Some((Reverse(_f), Reverse(g), current)) = open.pop() {
            if current == goal {
                let mut path = Vec::new();
                let mut cursor = current;

                while cursor != start {
                    path.push(Vec3::new(cursor.0 as f32, cursor.1 as f32, cursor.2 as f32));

                    cursor = *came_from.get(&cursor)?;
                }

                path.reverse();

                return Some(path);
            }

            if !closed.insert(current) {
                continue;
            }

            let (x, y, z) = current;

            const DIRECTIONS: [(i32, i32, i32); 6] = [
                (1, 0, 0),
                (-1, 0, 0),
                (0, 1, 0),
                (0, -1, 0),
                (0, 0, 1),
                (0, 0, -1),
            ];

            for (dx, dy, dz) in DIRECTIONS {
                let next = (x + dx, y + dy, z + dz);

                if next.0 < min_x
                    || next.0 > max_x
                    || next.1 < min_y
                    || next.1 > max_y
                    || next.2 < min_z
                    || next.2 > max_z
                {
                    continue;
                }

                if !Self::path_cell_walkable(world, next) {
                    continue;
                }

                let next_cost = g + 1;

                let Some(existing_cost) = cost_so_far.get(&next) else {
                    cost_so_far.insert(next, next_cost);

                    came_from.insert(next, current);

                    open.push((
                        Reverse(next_cost + Self::path_heuristic(next, goal)),
                        Reverse(next_cost),
                        next,
                    ));

                    continue;
                };

                if next_cost >= *existing_cost {
                    continue;
                }

                cost_so_far.insert(next, next_cost);

                came_from.insert(next, current);

                open.push((
                    Reverse(next_cost + Self::path_heuristic(next, goal)),
                    Reverse(next_cost),
                    next,
                ));
            }

            if cost_so_far.len() >= MAX_NODES {
                return None;
            }
        }

        None
    }

    fn path_heuristic(a: (i32, i32, i32), b: (i32, i32, i32)) -> i32 {
        (a.0 - b.0).abs() + (a.1 - b.1).abs() + (a.2 - b.2).abs()
    }

    fn path_cell_walkable(world: &World, position: (i32, i32, i32)) -> bool {
        let (x, y, z) = position;

        let floor = WorldCoord::new(x, y - 1, z);

        let feet = WorldCoord::new(x, y, z);

        let head = WorldCoord::new(x, y + 1, z);

        world.is_cell_solid(floor) && !world.is_cell_solid(feet) && !world.is_cell_solid(head)
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

    /// Spawns a player character at a valid world spawn point.
    pub fn spawn_player(&mut self, world: &World, project_path: Option<&Path>) -> Option<u64> {
        let character = spawn_at_random_point(world, self.next_id, project_path)?;

        let id = character.id;

        self.characters.insert(id, character);

        self.active_player_id = Some(id);

        self.next_id += 1;

        Some(id)
    }

    /// Returns all active runtime characters.
    pub fn get_active_characters(&self) -> impl Iterator<Item = &Character> {
        self.characters.values()
    }

    /// Returns all active runtime characters mutably.
    pub fn get_active_characters_mut(&mut self) -> impl Iterator<Item = &mut Character> {
        self.characters.values_mut()
    }

    /// Returns the authoritative active player.
    ///
    /// The fallback keeps tests and older code that directly insert a
    /// Character into the HashMap working.
    pub fn get_active_player(&self) -> Option<&Character> {
        if let Some(id) = self.active_player_id {
            if let Some(character) = self.characters.get(&id) {
                return Some(character);
            }
        }

        self.characters.values().next()
    }

    /// Returns the authoritative active player mutably.
    pub fn get_active_player_mut(&mut self) -> Option<&mut Character> {
        if let Some(id) = self.active_player_id {
            if self.characters.contains_key(&id) {
                return self.characters.get_mut(&id);
            }
        }

        self.characters.values_mut().next()
    }

    /// Clears all runtime character state.
    pub fn clear(&mut self) {
        self.characters.clear();
        self.entity_to_character.clear();
        self.character_to_entity.clear();
        self.active_player_id = None;

        self.next_id = 1;
    }

    pub fn has_characters(&self) -> bool {
        !self.characters.is_empty()
    }

    /// Explicitly teleports the active player.
    pub fn set_active_position(&mut self, position: Vec3) -> bool {
        let Some(character) = self.get_active_player_mut() else {
            return false;
        };

        character.transform.position = position;

        character.movement.velocity = Vec3::ZERO;

        character.movement.is_grounded = false;

        true
    }

    /// Legacy engine-controlled movement path.
    pub fn update(
        &mut self,
        world: &World,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        dt: f32,
        input: Vec2,
        jump_requested: bool,
    ) -> std::collections::HashSet<u64> {
        self.update_internal(world, physics_world, dt, Some(input), jump_requested, false)
    }

    /// AeoScript-controlled movement path.
    pub fn update_scripted(
        &mut self,
        world: &World,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        dt: f32,
    ) -> std::collections::HashSet<u64> {
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
    ) -> std::collections::HashSet<u64> {
        let mut contacted_cells = std::collections::HashSet::new();

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

            Self::resolve_static_voxel_collisions(
                character,
                physics_world,
                &mut contacted_cells,
                previous_position,
            );

            Self::resolve_dynamic_body_collisions(
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

        Self::resolve_character_collisions(&mut self.characters);

        contacted_cells
    }

    /// Resolves horizontal collision between runtime characters.
    ///
    /// Character-to-character collision:
    /// - only separates horizontally
    /// - never modifies Y position
    /// - never modifies Y velocity
    /// - never changes grounded state
    fn resolve_character_collisions(characters: &mut HashMap<u64, Character>) {
        const EPSILON: f32 = 0.001;

        for _ in 0..2 {
            let snapshots: Vec<(u64, Vec3, f32, f32, Vec3)> = characters
                .values()
                .map(|character| {
                    (
                        character.id,
                        character.transform.position,
                        character.collision.radius,
                        character.collision.height,
                        character.movement.velocity,
                    )
                })
                .collect();

            let mut position_corrections: HashMap<u64, Vec3> = HashMap::new();

            let mut velocity_corrections: HashMap<u64, Vec3> = HashMap::new();

            for i in 0..snapshots.len() {
                for j in (i + 1)..snapshots.len() {
                    let (id_a, position_a, radius_a, height_a, velocity_a) = snapshots[i];

                    let (id_b, position_b, radius_b, height_b, velocity_b) = snapshots[j];

                    let vertical_overlap = (position_a.y + height_a).min(position_b.y + height_b)
                        - position_a.y.max(position_b.y);

                    if vertical_overlap <= EPSILON {
                        continue;
                    }

                    let delta_x = position_b.x - position_a.x;

                    let delta_z = position_b.z - position_a.z;

                    let distance_squared = delta_x * delta_x + delta_z * delta_z;

                    let combined_radius = radius_a + radius_b;

                    if distance_squared >= combined_radius * combined_radius {
                        continue;
                    }

                    let (normal_x, normal_z, distance) = if distance_squared > 0.000001 {
                        let distance = distance_squared.sqrt();

                        (delta_x / distance, delta_z / distance, distance)
                    } else if id_a < id_b {
                        (1.0, 0.0, 0.0)
                    } else {
                        (-1.0, 0.0, 0.0)
                    };

                    let penetration = combined_radius - distance;

                    if penetration <= EPSILON {
                        continue;
                    }

                    let correction = Vec3::new(normal_x, 0.0, normal_z) * (penetration * 0.5);

                    position_corrections
                        .entry(id_a)
                        .and_modify(|value| {
                            *value -= correction;
                        })
                        .or_insert(-correction);

                    position_corrections
                        .entry(id_b)
                        .and_modify(|value| {
                            *value += correction;
                        })
                        .or_insert(correction);

                    let velocity_a_normal = velocity_a.x * normal_x + velocity_a.z * normal_z;

                    if velocity_a_normal > 0.0 {
                        let correction_velocity = Vec3::new(
                            -normal_x * velocity_a_normal,
                            0.0,
                            -normal_z * velocity_a_normal,
                        );

                        velocity_corrections
                            .entry(id_a)
                            .and_modify(|value| {
                                *value += correction_velocity;
                            })
                            .or_insert(correction_velocity);
                    }

                    let velocity_b_normal = velocity_b.x * normal_x + velocity_b.z * normal_z;

                    if velocity_b_normal < 0.0 {
                        let correction_velocity = Vec3::new(
                            -normal_x * velocity_b_normal,
                            0.0,
                            -normal_z * velocity_b_normal,
                        );

                        velocity_corrections
                            .entry(id_b)
                            .and_modify(|value| {
                                *value += correction_velocity;
                            })
                            .or_insert(correction_velocity);
                    }
                }
            }

            for (id, correction) in position_corrections {
                if let Some(character) = characters.get_mut(&id) {
                    character.transform.position += correction;
                }
            }

            for (id, correction) in velocity_corrections {
                if let Some(character) = characters.get_mut(&id) {
                    character.movement.velocity += correction;
                }
            }
        }
    }

    fn resolve_static_voxel_collisions(
        character: &mut Character,
        physics_world: &crate::engine::physics::PhysicsWorld,
        contacted_cells: &mut std::collections::HashSet<u64>,
        previous_position: Vec3,
    ) {
        let radius = character.collision.radius;
        let height = character.collision.height;

        character.movement.is_grounded = false;

        let min_pos = character.transform.position - Vec3::new(radius + 1.0, 0.5, radius + 1.0);
        let max_pos =
            character.transform.position + Vec3::new(radius + 1.0, height + 1.0, radius + 1.0);

        let candidates = physics_world.query_static_colliders_in_aabb(min_pos, max_pos);

        for (cell_id, v_min) in &candidates {
            let v_max = *v_min + Vec3::ONE;

            let overlap_x = (character.transform.position.x + radius).min(v_max.x)
                - (character.transform.position.x - radius).max(v_min.x);

            let overlap_y = (character.transform.position.y + height).min(v_max.y)
                - character.transform.position.y.max(v_min.y);

            let overlap_z = (character.transform.position.z + radius).min(v_max.z)
                - (character.transform.position.z - radius).max(v_min.z);

            if overlap_x <= 0.001 || overlap_y <= 0.001 || overlap_z <= 0.001 {
                continue;
            }

            contacted_cells.insert(*cell_id);

            let previous_bottom = previous_position.y;

            let previous_top = previous_position.y + height;

            let current_bottom = character.transform.position.y;

            let current_top = character.transform.position.y + height;

            let moving_down = character.movement.velocity.y <= 0.0;

            let moving_up = character.movement.velocity.y >= 0.0;

            let crossed_floor =
                moving_down && previous_bottom >= v_max.y - 0.001 && current_bottom < v_max.y;

            let crossed_ceiling =
                moving_up && previous_top <= v_min.y + 0.001 && current_top > v_min.y;

            if crossed_floor {
                character.transform.position.y = v_max.y;

                character.movement.velocity.y = 0.0;

                character.movement.is_grounded = true;

                continue;
            }

            if crossed_ceiling {
                character.transform.position.y = v_min.y - height;

                character.movement.velocity.y = 0.0;

                continue;
            }

            // If the character is already intersecting the collider without
            // crossing it vertically, resolve horizontally rather than
            // manufacturing an airborne floor/head collision.
            if overlap_x < overlap_z {
                if character.transform.position.x < v_min.x {
                    character.transform.position.x -= overlap_x;
                } else {
                    character.transform.position.x += overlap_x;
                }

                character.movement.velocity.x = 0.0;
            } else {
                if character.transform.position.z < v_min.z {
                    character.transform.position.z -= overlap_z;
                } else {
                    character.transform.position.z += overlap_z;
                }

                character.movement.velocity.z = 0.0;
            }
        }
    }

    fn resolve_dynamic_body_collisions(
        character: &mut Character,
        physics_world: &mut crate::engine::physics::PhysicsWorld,
        contacted_cells: &mut std::collections::HashSet<u64>,
        previous_position: Vec3,
    ) {
        let radius = character.collision.radius;

        let height = character.collision.height;

        for body in &mut physics_world.bodies {
            if body.anchored || !body.solid {
                continue;
            }

            let v_min = body.min_corner();

            let v_max = body.max_corner();

            let overlap_x = (character.transform.position.x + radius).min(v_max.x)
                - (character.transform.position.x - radius).max(v_min.x);

            let overlap_y = (character.transform.position.y + height).min(v_max.y)
                - character.transform.position.y.max(v_min.y);

            let overlap_z = (character.transform.position.z + radius).min(v_max.z)
                - (character.transform.position.z - radius).max(v_min.z);

            if overlap_x <= 0.001 || overlap_y <= 0.001 || overlap_z <= 0.001 {
                continue;
            }

            if body.cell_id != 0 {
                contacted_cells.insert(body.cell_id);
            }

            let previous_bottom = previous_position.y;

            let previous_top = previous_position.y + height;

            let current_bottom = character.transform.position.y;

            let current_top = character.transform.position.y + height;

            let moving_down = character.movement.velocity.y <= 0.0;

            let moving_up = character.movement.velocity.y >= 0.0;

            let crossed_floor =
                moving_down && previous_bottom >= v_max.y - 0.001 && current_bottom < v_max.y;

            let crossed_ceiling =
                moving_up && previous_top <= v_min.y + 0.001 && current_top > v_min.y;

            if crossed_floor {
                character.transform.position.y = v_max.y;

                character.movement.velocity.y = 0.0;

                character.movement.is_grounded = true;

                body.wake();

                continue;
            }

            if crossed_ceiling {
                character.transform.position.y = v_min.y - height;

                character.movement.velocity.y = 0.0;

                body.wake();

                continue;
            }

            if overlap_x < overlap_z {
                if character.transform.position.x < v_min.x {
                    character.transform.position.x -= overlap_x;
                } else {
                    character.transform.position.x += overlap_x;
                }

                character.movement.velocity.x = 0.0;
            } else {
                if character.transform.position.z < v_min.z {
                    character.transform.position.z -= overlap_z;
                } else {
                    character.transform.position.z += overlap_z;
                }

                character.movement.velocity.z = 0.0;
            }

            body.wake();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::character::movement;
    use crate::world::{CellType, WorldCoord};

    #[test]
    fn test_character_runtime_starts_empty() {
        let system = CharacterSystem::new();

        assert!(!system.has_characters());
        assert_eq!(system.get_active_characters().count(), 0);
        assert!(system.get_active_player().is_none());
    }

    #[test]
    fn test_adding_character_stores_it() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        system.spawn_player(&world, None);

        assert!(system.has_characters());
        assert_eq!(system.get_active_characters().count(), 1);
        assert!(system.get_active_player().is_some());
    }

    #[test]
    fn test_runtime_ids_remain_unique() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        world.set_cell(WorldCoord::new(5, 0, 5), CellType::SpawnPoint);

        system.spawn_player(&world, None);

        let id1 = system.get_active_characters().next().unwrap().id;

        system.spawn_player(&world, None);

        let id2 = system
            .get_active_characters()
            .find(|character| character.id != id1)
            .unwrap()
            .id;

        assert_ne!(id1, id2);
    }

    #[test]
    fn test_clear_removes_all_characters() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        system.spawn_player(&world, None);

        assert!(system.has_characters());

        system.clear();

        assert!(!system.has_characters());
        assert!(system.get_active_player().is_none());
        assert_eq!(system.next_id, 1);
    }

    #[test]
    fn test_character_teleport() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::SpawnPoint);

        system.spawn_player(&world, None);

        let player = system.get_active_player_mut().unwrap();

        player.movement.velocity = Vec3::new(4.0, -8.0, 2.0);

        player.movement.is_grounded = true;

        let target = Vec3::new(10.0, 20.0, 30.0);

        assert!(system.set_active_position(target));

        let player = system.get_active_player().unwrap();

        assert_eq!(player.transform.position, target);

        assert_eq!(player.movement.velocity, Vec3::ZERO);

        assert!(!player.movement.is_grounded);
    }

    #[test]
    fn test_character_gravity() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character = Character::new(1, Vec3::new(0.0, 10.0, 0.0));

        system.characters.insert(1, character);

        system.active_player_id = Some(1);

        let mut physics_world = crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        for _ in 0..60 {
            let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, false);
        }

        let updated = system.get_active_player().unwrap();

        assert!(updated.transform.position.y < 10.0);

        assert!(updated.movement.velocity.y < 0.0);
    }

    #[test]
    fn test_character_horizontal_movement() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character = Character::new(1, Vec3::ZERO);

        system.characters.insert(1, character);

        system.active_player_id = Some(1);

        let mut physics_world = crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let _ = system.update(&world, &mut physics_world, 0.1, Vec2::new(1.0, 0.0), false);

        let updated = system.get_active_player().unwrap();

        assert!(updated.transform.position.x > 0.0);

        assert_eq!(updated.animation.current_state, AnimationState::Walk);

        assert!(updated.animation_controller.blend_weight() > 0.0);
    }

    #[test]
    fn test_scripted_horizontal_movement_preserves_velocity() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character = Character::new(1, Vec3::ZERO);

        system.characters.insert(1, character);

        system.active_player_id = Some(1);

        let mut physics_world = crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        {
            let player = system.get_active_player_mut().unwrap();

            player.movement.velocity.x = 5.0;

            player.movement.velocity.z = -2.0;
        }

        let _ = system.update_scripted(&world, &mut physics_world, 0.1);

        let updated = system.get_active_player().unwrap();

        assert!(updated.transform.position.x > 0.49);

        assert!(updated.transform.position.z < -0.19);

        assert_eq!(updated.animation.current_state, AnimationState::Walk);
    }

    #[test]
    fn test_character_ground_collision() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);

        let mut physics_world = crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let character = Character::new(1, Vec3::new(0.0, 1.5, 0.0));

        system.characters.insert(1, character);

        system.active_player_id = Some(1);

        for _ in 0..60 {
            let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, false);
        }

        let updated = system.get_active_player().unwrap();

        assert!((updated.transform.position.y - 1.0).abs() < 0.01);

        assert!(updated.movement.is_grounded);
    }

    #[test]
    fn test_animation_state_switching() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character = Character::new(1, Vec3::ZERO);

        system.characters.insert(1, character);

        system.active_player_id = Some(1);

        let mut physics_world = crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let _ = system.update(&world, &mut physics_world, 0.1, Vec2::ZERO, false);

        assert_eq!(
            system.get_active_player().unwrap().animation.current_state,
            AnimationState::Idle
        );

        let _ = system.update(&world, &mut physics_world, 0.1, Vec2::X, false);

        assert_eq!(
            system.get_active_player().unwrap().animation.current_state,
            AnimationState::Walk
        );
    }

    #[test]
    fn test_character_facing_direction() {
        let mut system = CharacterSystem::new();
        let world = World::new();

        let character = Character::new(1, Vec3::ZERO);

        system.characters.insert(1, character);

        system.active_player_id = Some(1);

        let mut physics_world = crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let _ = system.update(&world, &mut physics_world, 0.1, Vec2::new(1.0, 0.0), false);

        let rotation_right = system.get_active_player().unwrap().transform.rotation;

        let _ = system.update(&world, &mut physics_world, 0.1, Vec2::new(-1.0, 0.0), false);

        let rotation_left = system.get_active_player().unwrap().transform.rotation;

        assert_ne!(rotation_right, rotation_left);

        let _ = system.update(&world, &mut physics_world, 0.1, Vec2::ZERO, false);

        let rotation_idle = system.get_active_player().unwrap().transform.rotation;

        assert_eq!(rotation_left, rotation_idle);
    }

    #[test]
    fn test_character_jumping() {
        let mut system = CharacterSystem::new();
        let mut world = World::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);

        let mut physics_world = crate::engine::physics::PhysicsWorld::new();

        physics_world.register_from_world(&world);

        let character = Character::new(1, Vec3::new(0.0, 1.0, 0.0));

        system.characters.insert(1, character);

        system.active_player_id = Some(1);

        let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, false);

        assert!(system.get_active_player().unwrap().movement.is_grounded);

        let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, true);

        let updated = system.get_active_player().unwrap();

        let expected_velocity = JUMP_IMPULSE + world.gravity.y * (1.0 / 60.0);

        assert!((updated.movement.velocity.y - expected_velocity).abs() < 0.001);

        assert!(!updated.movement.is_grounded);

        let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, true);

        assert!(system.get_active_player().unwrap().movement.velocity.y < movement::JUMP_IMPULSE);
    }

    #[test]
    fn test_character_collision_with_dynamic_bodies() {
        use crate::engine::physics::{PhysicsBody, PhysicsBodyId, PhysicsWorld};

        let mut system = CharacterSystem::new();

        let world = World::new();

        let mut physics_world = PhysicsWorld::new();

        let mut character = Character::new(1, Vec3::ZERO);

        character.collision.radius = 0.5;

        system.characters.insert(1, character);

        system.active_player_id = Some(1);

        let body_id = PhysicsBodyId(1);

        let mut body = PhysicsBody::new(body_id, 0, Vec3::new(0.6, 0.0, -0.5), Vec3::ONE);

        body.solid = true;
        body.anchored = false;

        physics_world.bodies.push(body);

        let _ = system.update(&world, &mut physics_world, 0.1, Vec2::new(1.0, 0.0), false);

        let updated = system.get_active_player().unwrap();

        assert!(
            updated.transform.position.x < 0.4,
            "Character should be blocked by awake body"
        );

        assert!((updated.transform.position.x - 0.1).abs() < 0.01);

        physics_world.bodies[0].is_sleeping = true;

        physics_world.bodies[0].position = Vec3::new(0.6, 0.0, -0.5);

        system.get_active_player_mut().unwrap().transform.position = Vec3::ZERO;

        let _ = system.update(&world, &mut physics_world, 0.1, Vec2::new(1.0, 0.0), false);

        let updated = system.get_active_player().unwrap();

        assert!(updated.transform.position.x < 0.4);

        assert!(!physics_world.bodies[0].is_sleeping);

        physics_world.bodies[0].solid = false;

        system.get_active_player_mut().unwrap().transform.position = Vec3::ZERO;

        let _ = system.update(&world, &mut physics_world, 0.1, Vec2::new(1.0, 0.0), false);

        let updated = system.get_active_player().unwrap();

        assert!((updated.transform.position.x - 0.4).abs() < 0.01);
    }

    #[test]
    fn test_character_collision_with_other_characters() {
        let mut system = CharacterSystem::new();

        let world = World::new();

        let mut physics_world = crate::engine::physics::PhysicsWorld::new();

        let mut character_a = Character::new(1, Vec3::new(0.0, 0.0, 0.0));

        let mut character_b = Character::new(2, Vec3::new(0.5, 0.0, 0.0));

        character_a.movement.velocity = Vec3::new(2.0, 0.0, 0.0);

        character_b.movement.velocity = Vec3::new(-2.0, 0.0, 0.0);

        system.characters.insert(1, character_a);

        system.characters.insert(2, character_b);

        let _ = system.update_scripted(&world, &mut physics_world, 1.0 / 60.0);

        let a = system.characters.get(&1).unwrap();

        let b = system.characters.get(&2).unwrap();

        let dx = b.transform.position.x - a.transform.position.x;

        let dz = b.transform.position.z - a.transform.position.z;

        let horizontal_distance = (dx * dx + dz * dz).sqrt();

        let minimum_distance = a.collision.radius + b.collision.radius;

        assert!(
            horizontal_distance >= minimum_distance - 0.001,
            "Characters still overlap: distance={}, minimum={}",
            horizontal_distance,
            minimum_distance
        );
    }

    #[test]
    fn test_entity_character_relationship_and_position_sync() {
        let mut system = CharacterSystem::new();

        let mut entity_manager = EntityManager::new();

        let pos = Vec3::new(12.0, 3.0, 8.0);

        let char_id = system.spawn_character(pos, None);

        let entity_id = entity_manager.create_entity("Villager");

        entity_manager.set_position(entity_id, pos);

        system.associate_entity(entity_id, char_id);

        assert_eq!(system.get_character_id_for_entity(entity_id), Some(char_id));

        assert_eq!(system.get_entity_for_character(char_id), Some(entity_id));

        let character = system.get_character_for_entity(entity_id).unwrap();

        assert_eq!(character.transform.position, pos);

        let new_pos = Vec3::new(20.0, 3.0, 30.0);

        system.set_entity_position(entity_id, new_pos);

        system.sync_entity_positions(&mut entity_manager);

        assert_eq!(entity_manager.get_position(entity_id), Some(new_pos));
    }

    #[test]
    fn test_multiple_spawned_characters() {
        let mut system = CharacterSystem::new();

        let id1 = system.spawn_character(Vec3::new(10.0, 1.0, 5.0), None);

        let id2 = system.spawn_character(Vec3::new(14.0, 1.0, 5.0), None);

        let id3 = system.spawn_character(Vec3::new(20.0, 1.0, 8.0), None);

        assert_ne!(id1, id2);
        assert_ne!(id2, id3);

        assert_eq!(system.get_active_characters().count(), 3);
    }

    #[test]
    fn test_character_runtime_controls() {
        let mut system = CharacterSystem::new();

        let entity_id = EntityId(100);

        let character_id = system.spawn_character(Vec3::new(0.0, 1.0, 0.0), None);

        system.associate_entity(entity_id, character_id);

        assert!(system.set_entity_velocity(entity_id, Vec3::new(1.0, 2.0, 3.0,)));

        assert_eq!(
            system.get_entity_velocity(entity_id),
            Some(Vec3::new(1.0, 2.0, 3.0,))
        );

        assert!(system.set_entity_facing_direction(entity_id, 1.0, 0.0,));

        let facing = system.get_entity_facing(entity_id).unwrap();

        assert!((facing.x - 1.0).abs() < 0.001);

        assert!(facing.y.abs() < 0.001);

        assert!(system.set_entity_animation(entity_id, "Walk",).unwrap());
    }

    #[test]
    fn test_character_health_controls() {
        let mut system = CharacterSystem::new();

        let entity_id = EntityId(100);

        let character_id = system.spawn_character(Vec3::ZERO, None);

        system.associate_entity(entity_id, character_id);

        assert_eq!(system.get_entity_health(entity_id), Some(100.0));

        assert_eq!(system.damage_entity(entity_id, 25.0), Some(75.0));

        assert_eq!(system.heal_entity(entity_id, 10.0), Some(85.0));

        assert!(system.set_entity_health(entity_id, 0.0));

        assert_eq!(system.is_entity_alive(entity_id), Some(false));
    }

    #[test]
    fn test_character_destroy() {
        let mut system = CharacterSystem::new();

        let entity_id = EntityId(100);

        let character_id = system.spawn_character(Vec3::ZERO, None);

        system.associate_entity(entity_id, character_id);

        assert!(system.destroy_entity(entity_id));

        assert!(system.get_character_for_entity(entity_id).is_none());

        assert!(system.get_entity_for_character(character_id).is_none());
    }

    #[test]
    fn test_character_pathfinding() {
        let mut system = CharacterSystem::new();

        let mut world = World::new();

        world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);

        world.set_cell(WorldCoord::new(1, 0, 0), CellType::Block);

        world.set_cell(WorldCoord::new(2, 0, 0), CellType::Block);

        let entity_id = EntityId(100);

        let character_id = system.spawn_character(Vec3::new(0.0, 1.0, 0.0), None);

        system.associate_entity(entity_id, character_id);

        let path = system.find_path_for_entity(entity_id, &world, Vec3::new(2.0, 1.0, 0.0));

        assert!(path.is_some());
        assert!(!path.unwrap().is_empty());
    }

    #[test]
    fn test_anchored_cell_runtime_movement_collision() {
        use crate::engine::physics::PhysicsWorld;

        let mut system = CharacterSystem::new();

        let mut world = World::new();

        let mut physics_world = PhysicsWorld::new();

        let coord = WorldCoord::new(0, 0, 0);

        world.set_cell(coord, CellType::Block);

        let cell_id = world.get(coord).unwrap().id;

        physics_world.register_from_world(&world);

        assert_eq!(physics_world.static_colliders.len(), 1);

        let character = Character::new(1, Vec3::new(0.0, 1.5, 0.0));

        system.characters.insert(1, character);

        system.active_player_id = Some(1);

        for _ in 0..60 {
            let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, false);
        }

        let updated = system.get_active_player().unwrap();

        assert!((updated.transform.position.y - 1.0).abs() < 0.01);

        assert!(updated.movement.is_grounded);

        world.set_visual_offset_runtime(coord, Vec3::new(0.0, 2.0, 0.0));

        physics_world.sync_with_world(&mut world);

        system.get_active_player_mut().unwrap().transform.position = Vec3::new(0.0, 1.5, 0.0);

        system.get_active_player_mut().unwrap().movement.is_grounded = false;

        system.get_active_player_mut().unwrap().movement.velocity = Vec3::ZERO;

        for _ in 0..10 {
            let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, false);
        }

        let updated = system.get_active_player().unwrap();

        assert!(updated.transform.position.y < 1.5);

        assert!(!updated.movement.is_grounded);

        system.get_active_player_mut().unwrap().transform.position = Vec3::new(0.0, 3.5, 0.0);

        for _ in 0..60 {
            let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, false);
        }

        let updated = system.get_active_player().unwrap();

        assert!((updated.transform.position.y - 3.0).abs() < 0.01);

        assert!(updated.movement.is_grounded);

        world.set_visual_offset_runtime(coord, Vec3::new(0.0, 4.0, 0.0));

        physics_world.sync_with_world(&mut world);

        system.get_active_player_mut().unwrap().transform.position = Vec3::new(0.0, 5.5, 0.0);

        for _ in 0..60 {
            let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, false);
        }

        let updated = system.get_active_player().unwrap();

        assert!((updated.transform.position.y - 5.0).abs() < 0.01);

        assert!(updated.movement.is_grounded);

        world.clear_runtime_state();

        physics_world.sync_with_world(&mut world);

        system.get_active_player_mut().unwrap().transform.position = Vec3::new(0.0, 1.5, 0.0);

        for _ in 0..60 {
            let _ = system.update(&world, &mut physics_world, 1.0 / 60.0, Vec2::ZERO, false);
        }

        let updated = system.get_active_player().unwrap();

        assert!((updated.transform.position.y - 1.0).abs() < 0.01);

        assert!(world.get(coord).is_some());

        assert_eq!(world.get(coord).unwrap().id, cell_id);
    }
}
