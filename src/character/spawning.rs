use super::character::Character;
use super::collision::CharacterCollision;
use crate::world::{CellType, World, WorldCoord};
use glam::Vec3;

pub fn spawn_at_random_point(world: &World, next_id: u64) -> Option<Character> {
    let spawn_points: Vec<WorldCoord> = world
        .active_blocks()
        .iter()
        .filter_map(|&coord| {
            world
                .get(coord)
                .filter(|cell| cell.cell_type == CellType::SpawnPoint)
                .map(|_| coord)
        })
        .collect();

    if spawn_points.is_empty() {
        return None;
    }

    // Keep the current deterministic behavior for now.
    // Spawn selection can become random later without changing the
    // clearance rules used to validate a candidate position.
    let coord = spawn_points[0];

    let spawn_y = if let Some(_cell) = world.get(coord) {
        if world.is_cell_solid(coord) {
            coord.y as f32 + 1.0
        } else {
            coord.y as f32
        }
    } else {
        return None;
    };

    let collision = CharacterCollision::new();

    let base_position = Vec3::new(coord.x as f32, spawn_y, coord.z as f32);

    if has_character_clearance(world, base_position, &collision) {
        return Some(Character::new(next_id, base_position));
    }

    // Search outward for the closest available position.
    // The character keeps the same spawn height while searching nearby
    // X and Z positions for enough room for its collision shape.
    for radius in 1..=10 {
        let radius = radius as i32;

        for x in -radius..=radius {
            for z in -radius..=radius {
                if x.abs() != radius && z.abs() != radius {
                    continue;
                }

                let candidate = Vec3::new(
                    coord.x as f32 + x as f32,
                    spawn_y,
                    coord.z as f32 + z as f32,
                );

                if has_character_clearance(world, candidate, &collision) {
                    return Some(Character::new(next_id, candidate));
                }
            }
        }
    }

    None
}

fn has_character_clearance(world: &World, position: Vec3, collision: &CharacterCollision) -> bool {
    let radius = collision.radius;
    let height = collision.height;

    let min_x = (position.x - radius).floor() as i32;
    let max_x = (position.x + radius).ceil() as i32;
    let min_y = position.y.floor() as i32;
    let max_y = (position.y + height).ceil() as i32;
    let min_z = (position.z - radius).floor() as i32;
    let max_z = (position.z + radius).ceil() as i32;

    for x in min_x..max_x {
        for y in min_y..max_y {
            for z in min_z..max_z {
                let coord = WorldCoord::new(x, y, z);

                if let Some(_cell) = world.get(coord) {
                    if world.is_cell_solid(coord) {
                        return false;
                    }
                }
            }
        }
    }

    true
}
