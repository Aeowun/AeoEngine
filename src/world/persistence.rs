use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use glam::Vec3;

use super::cell::CellType;
use super::coordinate::WorldCoord;
use super::world::World;
use crate::scripting::binding::ScriptBinding;

/// We save the world to a simple text format. This is easier to debug and
/// version than a binary format for now.
pub fn save_world(world: &World, path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    // We save the global gravity setting first so it can be parsed easily
    // before the block data.
    writeln!(
        file,
        "GRAVITY {} {} {}",
        world.gravity.x, world.gravity.y, world.gravity.z
    )?;

    // Save global lighting settings.
    writeln!(
        file,
        "LIGHTING {} {} {} {} {} {} {} {} {} {}",
        world.lighting.shadows_enabled,
        world.lighting.global_light_enabled,
        world.lighting.global_light_direction.x,
        world.lighting.global_light_direction.y,
        world.lighting.global_light_direction.z,
        world.lighting.global_light_color.x,
        world.lighting.global_light_color.y,
        world.lighting.global_light_color.z,
        world.lighting.global_light_intensity,
        world.lighting.ambient_intensity
    )?;

    for binding in &world.script_bindings {
        writeln!(
            file,
            "SCRIPT_BINDING {} {}",
            binding.target_identity, binding.script_path
        )?;
    }

    for coord in world.active_blocks() {
        if let Some(cell) = world.get(coord) {
            match cell.cell_type {
                CellType::Block => {
                    writeln!(
                        file,
                        "BLOCK {} {} {} {} {} {} {} {} {} {}",
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.visible,
                        cell.solid,
                        cell.anchored,
                        cell.texture,
                        cell.color_rgb.x,
                        cell.color_rgb.y,
                        cell.color_rgb.z
                    )?;
                }

                CellType::SpawnPoint => {
                    writeln!(
                        file,
                        "SPAWN_POINT {} {} {} {} {} {} {} {} {} {}",
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.visible,
                        cell.solid,
                        cell.anchored,
                        cell.texture,
                        cell.color_rgb.x,
                        cell.color_rgb.y,
                        cell.color_rgb.z
                    )?;
                }

                CellType::Light => {
                    writeln!(
                        file,
                        "LIGHT {} {} {} {} {} {} {} {} {}",
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.light_color.x,
                        cell.light_color.y,
                        cell.light_color.z,
                        cell.light_intensity,
                        cell.light_range,
                        cell.light_shadows
                    )?;
                }

                _ => {}
            }
        }
    }

    Ok(())
}

/// We clear and reload the world from disk. This supports the standard project
/// save format used by the engine.
pub fn load_world(world: &mut World, path: &Path) -> std::io::Result<()> {
    *world = World::new();

    if !path.exists() {
        return Ok(());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        // We check for global settings first.
        if parts[0] == "GRAVITY" && parts.len() >= 4 {
            let x = parts[1].parse::<f32>().ok();
            let y = parts[2].parse::<f32>().ok();
            let z = parts[3].parse::<f32>().ok();

            // If the tag exists but the numbers are invalid, we keep the default
            // gravity from World::new().
            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                world.gravity = Vec3::new(x, y, z);
            }

            continue;
        }

        if parts[0] == "LIGHTING" && parts.len() >= 11 {
            world.lighting.shadows_enabled = parts[1].parse::<bool>().unwrap_or(true);

            world.lighting.global_light_enabled = parts[2].parse::<bool>().unwrap_or(true);

            let dx = parts[3].parse::<f32>().ok();
            let dy = parts[4].parse::<f32>().ok();
            let dz = parts[5].parse::<f32>().ok();

            if let (Some(x), Some(y), Some(z)) = (dx, dy, dz) {
                world.lighting.global_light_direction = Vec3::new(x, y, z);
            }

            let cr = parts[6].parse::<f32>().ok();
            let cg = parts[7].parse::<f32>().ok();
            let cb = parts[8].parse::<f32>().ok();

            if let (Some(r), Some(g), Some(b)) = (cr, cg, cb) {
                world.lighting.global_light_color = Vec3::new(r, g, b);
            }

            world.lighting.global_light_intensity = parts[9].parse::<f32>().unwrap_or(1.0);

            world.lighting.ambient_intensity = parts[10].parse::<f32>().unwrap_or(0.2);

            continue;
        }

        if parts[0] == "SCRIPT_BINDING" && parts.len() >= 3 {
            world.script_bindings.push(ScriptBinding::new(
                parts[1].to_string(),
                parts[2].to_string(),
            ));
            continue;
        }

        if parts.len() >= 4 {
            let block_type = parts[0];

            let x = parts[1].parse::<i32>().ok();
            let y = parts[2].parse::<i32>().ok();
            let z = parts[3].parse::<i32>().ok();

            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                let coord = WorldCoord::new(x, y, z);

                if block_type == "BLOCK" {
                    world.set_cell(coord, CellType::Block);

                    if let Some(cell) = world.get_mut(coord) {
                        if parts.len() >= 11 {
                            cell.visible = parts[4].parse::<bool>().unwrap_or(true);

                            cell.solid = parts[5].parse::<bool>().unwrap_or(true);

                            cell.anchored = parts[6].parse::<bool>().unwrap_or(true);

                            cell.texture = parts[7].to_string();

                            let r = parts[8].parse::<f32>().unwrap_or(0.5);
                            let g = parts[9].parse::<f32>().unwrap_or(0.5);
                            let b = parts[10].parse::<f32>().unwrap_or(0.5);

                            cell.color_rgb = Vec3::new(r, g, b);
                        }
                    }
                } else if block_type == "SPAWN_POINT" {
                    world.set_cell(coord, CellType::SpawnPoint);

                    if let Some(cell) = world.get_mut(coord) {
                        if parts.len() >= 11 {
                            cell.visible = parts[4].parse::<bool>().unwrap_or(true);

                            cell.solid = parts[5].parse::<bool>().unwrap_or(true);

                            cell.anchored = parts[6].parse::<bool>().unwrap_or(true);

                            cell.texture = parts[7].to_string();

                            let r = parts[8].parse::<f32>().unwrap_or(0.5);
                            let g = parts[9].parse::<f32>().unwrap_or(0.5);
                            let b = parts[10].parse::<f32>().unwrap_or(0.5);

                            cell.color_rgb = Vec3::new(r, g, b);
                        }
                    }
                } else if block_type == "GRASS" {
                    world.set_cell(coord, CellType::Block);

                    if let Some(cell) = world.get_mut(coord) {
                        if parts.len() >= 8 {
                            cell.visible = parts[4].parse::<bool>().unwrap_or(true);

                            cell.solid = parts[5].parse::<bool>().unwrap_or(true);

                            cell.anchored = parts[6].parse::<bool>().unwrap_or(true);

                            cell.texture = parts[7].to_string();
                        } else if parts.len() >= 7 {
                            // Format without anchored.
                            cell.visible = parts[4].parse::<bool>().unwrap_or(true);

                            cell.solid = parts[5].parse::<bool>().unwrap_or(true);

                            cell.anchored = true;
                            cell.texture = parts[6].to_string();
                        } else {
                            // Upgrade old format.
                            cell.visible = true;
                            cell.solid = true;
                            cell.anchored = true;
                            cell.texture = "Block_tx".to_string();
                        }

                        // Default color for legacy grass.
                        cell.color_rgb = Vec3::new(0.5, 0.5, 0.5);
                    }
                } else if block_type == "LIGHT" {
                    world.set_cell(coord, CellType::Light);

                    if let Some(cell) = world.get_mut(coord) {
                        if parts.len() >= 10 {
                            let r = parts[4].parse::<f32>().unwrap_or(1.0);
                            let g = parts[5].parse::<f32>().unwrap_or(1.0);
                            let b = parts[6].parse::<f32>().unwrap_or(1.0);

                            cell.light_color = Vec3::new(r, g, b);

                            cell.light_intensity = parts[7].parse::<f32>().unwrap_or(5.0);

                            cell.light_range = parts[8].parse::<f32>().unwrap_or(10.0);

                            cell.light_shadows = parts[9].parse::<bool>().unwrap_or(true);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Cell;
    use glam::Vec3;
    use std::fs;

    #[test]
    fn test_world_default_gravity() {
        // A new world must start with standard gravity.
        let world = World::new();

        assert_eq!(world.gravity, Vec3::new(0.0, -9.81, 0.0));
    }

    #[test]
    fn test_world_gravity_persistence() {
        // This proves that custom gravity vectors survive the save and load
        // cycle without loss of precision.
        let mut world = World::new();
        let custom_gravity = Vec3::new(1.2, 3.4, -5.6);

        world.gravity = custom_gravity;

        let path = Path::new("test_gravity.dat");

        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(loaded_world.gravity, custom_gravity);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_world_lighting_defaults() {
        // Verify that a new World has the documented lighting defaults.
        let world = World::new();

        assert_eq!(world.lighting.shadows_enabled, true);
        assert_eq!(world.lighting.global_light_enabled, true);
        assert!((world.lighting.global_light_intensity - 1.0).abs() < 1e-5);
        assert!((world.lighting.ambient_intensity - 0.2).abs() < 1e-5);
    }

    #[test]
    fn test_light_cell_construction() {
        // Verify that Cell::new_light() produces a valid authored Light cell.
        let cell = Cell::new_light();

        assert_eq!(cell.cell_type, CellType::Light);
        assert_eq!(cell.visible, false);
        assert_eq!(cell.solid, false);
        assert_eq!(cell.light_intensity, 5.0);
    }

    #[test]
    fn test_world_lighting_persistence() {
        // Verify that global lighting settings survive the save/load cycle.
        let mut world = World::new();

        world.lighting.shadows_enabled = false;
        world.lighting.global_light_enabled = false;
        world.lighting.global_light_direction = Vec3::new(0.0, 1.0, 0.0);
        world.lighting.global_light_color = Vec3::new(1.0, 0.5, 0.2);
        world.lighting.global_light_intensity = 4.2;
        world.lighting.ambient_intensity = 0.88;

        let path = Path::new("test_lighting.dat");

        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(loaded_world.lighting.shadows_enabled, false);
        assert_eq!(loaded_world.lighting.global_light_enabled, false);

        assert_eq!(
            loaded_world.lighting.global_light_direction,
            Vec3::new(0.0, 1.0, 0.0)
        );

        assert_eq!(
            loaded_world.lighting.global_light_color,
            Vec3::new(1.0, 0.5, 0.2)
        );

        assert!((loaded_world.lighting.global_light_intensity - 4.2).abs() < 1e-5);

        assert!((loaded_world.lighting.ambient_intensity - 0.88).abs() < 1e-5);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_light_cell_persistence() {
        // Verify that Light cell properties are correctly persisted and reloaded.
        let mut world = World::new();

        let coord = WorldCoord::new(10, 20, 30);

        world.set_cell(coord, CellType::Light);

        if let Some(cell) = world.get_mut(coord) {
            cell.light_color = Vec3::new(0.1, 0.2, 0.3);
            cell.light_intensity = 99.0;
            cell.light_range = 50.0;
            cell.light_shadows = false;
        }

        let path = Path::new("test_light_cell.dat");

        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        let loaded_cell = loaded_world.get(coord).unwrap();

        assert_eq!(loaded_cell.cell_type, CellType::Light);
        assert_eq!(loaded_cell.light_color, Vec3::new(0.1, 0.2, 0.3));
        assert_eq!(loaded_cell.light_intensity, 99.0);
        assert_eq!(loaded_cell.light_range, 50.0);
        assert_eq!(loaded_cell.light_shadows, false);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_spawn_point_persistence() {
        // Spawn points must retain the same authored block properties after
        // saving and loading.
        let mut world = World::new();

        let coord = WorldCoord::new(4, 8, 12);

        world.set_cell(coord, CellType::SpawnPoint);

        if let Some(cell) = world.get_mut(coord) {
            cell.visible = false;
            cell.solid = true;
            cell.anchored = false;
            cell.texture = "Block_tx".to_string();
            cell.color_rgb = Vec3::new(0.2, 0.4, 0.8);
        }

        let path = Path::new("test_spawn_point.dat");

        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        let loaded_cell = loaded_world.get(coord).unwrap();

        assert_eq!(loaded_cell.cell_type, CellType::SpawnPoint);
        assert_eq!(loaded_cell.visible, false);
        assert_eq!(loaded_cell.solid, true);
        assert_eq!(loaded_cell.anchored, false);
        assert_eq!(loaded_cell.texture, "Block_tx");
        assert_eq!(loaded_cell.color_rgb, Vec3::new(0.2, 0.4, 0.8));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_empty_bindings_persistence() {
        let mut world = World::new();
        world.script_bindings.clear();

        let path = Path::new("test_empty_bindings.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert!(loaded_world.script_bindings.is_empty());
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_one_binding_persistence() {
        let mut world = World::new();
        world.script_bindings.push(ScriptBinding::new("Player", "scripts/player.aeo"));

        let path = Path::new("test_one_binding.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(loaded_world.script_bindings.len(), 1);
        assert_eq!(loaded_world.script_bindings[0].target_identity, "Player");
        assert_eq!(loaded_world.script_bindings[0].script_path, "scripts/player.aeo");
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_multiple_bindings_persistence() {
        let mut world = World::new();
        world.script_bindings.push(ScriptBinding::new("Player", "scripts/player.aeo"));
        world.script_bindings.push(ScriptBinding::new("Door", "scripts/door.aeo"));
        world.script_bindings.push(ScriptBinding::new("Enemy", "scripts/enemy.aeo"));

        let path = Path::new("test_multi_bindings.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(loaded_world.script_bindings.len(), 3);
        assert_eq!(loaded_world.script_bindings[0].target_identity, "Player");
        assert_eq!(loaded_world.script_bindings[1].target_identity, "Door");
        assert_eq!(loaded_world.script_bindings[2].target_identity, "Enemy");
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_existing_data_and_bindings_persistence() {
        let mut world = World::new();
        let coord = WorldCoord::new(1, 2, 3);
        world.set_cell(coord, CellType::Block);
        world.script_bindings.push(ScriptBinding::new("Box", "scripts/box.aeo"));

        let path = Path::new("test_mixed_data.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert!(loaded_world.get(coord).is_some());
        assert_eq!(loaded_world.script_bindings.len(), 1);
        assert_eq!(loaded_world.script_bindings[0].target_identity, "Box");
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_older_data_loadability() {
        // Create a file manually WITHOUT SCRIPT_BINDING tags
        let path = Path::new("test_old_format.dat");
        {
            let mut file = File::create(path).unwrap();
            writeln!(file, "GRAVITY 0 -9.81 0").unwrap();
            writeln!(file, "BLOCK 0 0 0 true true true Default 0.5 0.5 0.5").unwrap();
        }

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(loaded_world.gravity.y, -9.81);
        assert!(loaded_world.get(WorldCoord::new(0, 0, 0)).is_some());
        assert!(loaded_world.script_bindings.is_empty());

        fs::remove_file(path).ok();
    }
}
