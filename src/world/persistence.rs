use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

use glam::Vec3;
use serde_json;

use super::cell::{Cell, CellType};
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

    // Save global sky settings.
    writeln!(
        file,
        "SKY {} {}",
        world.sky.enabled,
        if world.sky.texture.is_empty() {
            "None"
        } else {
            &world.sky.texture
        }
    )?;

    // Save global mouse settings.
    writeln!(
        file,
        "MOUSE {} {}",
        world.cursor_visible, world.screen_locked
    )?;

    for binding in &world.script_bindings {
        writeln!(
            file,
            "SCRIPT_BINDING {} {} {}",
            binding.target_identity, binding.script_path, binding.enabled
        )?;
    }

    for path in &world.disabled_scripts {
        writeln!(file, "DISABLED_SCRIPT {}", path)?;
    }

    writeln!(file, "SELECTED_CHARACTER {}", world.selected_character)?;
    writeln!(file, "SELECTED_CONTROLLER {}", world.selected_controller)?;
    writeln!(file, "SELECTED_CAMERA {}", world.selected_camera)?;

    for coord in world.active_blocks() {
        if let Some(cell) = world.get(coord) {
            match cell.cell_type {
                CellType::Block => {
                    writeln!(
                        file,
                        "BLOCK {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        cell.id,
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.visible,
                        cell.solid,
                        cell.anchored,
                        cell.texture,
                        cell.color_rgb.x,
                        cell.color_rgb.y,
                        cell.color_rgb.z,
                        cell.entity_identity.as_deref().unwrap_or("None"),
                        cell.collision_events_enabled
                    )?;
                }

                CellType::FxBlock => {
                    writeln!(
                        file,
                        "FX_BLOCK {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        cell.id,
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.visible,
                        cell.solid,
                        cell.anchored,
                        cell.texture,
                        cell.color_rgb.x,
                        cell.color_rgb.y,
                        cell.color_rgb.z,
                        cell.entity_identity.as_deref().unwrap_or("None"),
                        cell.collision_events_enabled
                    )?;
                }

                CellType::SpawnPoint => {
                    writeln!(
                        file,
                        "SPAWN_POINT {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        cell.id,
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.visible,
                        cell.solid,
                        cell.anchored,
                        cell.texture,
                        cell.color_rgb.x,
                        cell.color_rgb.y,
                        cell.color_rgb.z,
                        cell.entity_identity.as_deref().unwrap_or("None"),
                        cell.collision_events_enabled
                    )?;
                }

                CellType::Light => {
                    writeln!(
                        file,
                        "LIGHT {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        cell.id,
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.light_color.x,
                        cell.light_color.y,
                        cell.light_color.z,
                        cell.light_intensity,
                        cell.light_range,
                        cell.light_shadows,
                        cell.light_enabled,
                        cell.entity_identity.as_deref().unwrap_or("None"),
                        cell.collision_events_enabled
                    )?;
                }

                CellType::AudioEmitter => {
                    writeln!(
                        file,
                        "AUDIO_EMITTER {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        cell.id,
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.visible,
                        cell.solid,
                        cell.anchored,
                        cell.texture,
                        cell.color_rgb.x,
                        cell.color_rgb.y,
                        cell.color_rgb.z,
                        cell.entity_identity.as_deref().unwrap_or("None"),
                        cell.collision_events_enabled,
                        cell.playing,
                        cell.looped,
                        cell.volume,
                        if cell.audio.is_empty() {
                            "None"
                        } else {
                            &cell.audio
                        }
                    )?;
                }

                CellType::Player => {
                    writeln!(
                        file,
                        "PLAYER {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        cell.id,
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.visible,
                        cell.solid,
                        cell.anchored,
                        cell.texture,
                        cell.color_rgb.x,
                        cell.color_rgb.y,
                        cell.color_rgb.z,
                        cell.entity_identity.as_deref().unwrap_or("None"),
                        cell.collision_events_enabled
                    )?;
                }

                CellType::NPC => {
                    writeln!(
                        file,
                        "NPC {} {} {} {} {} {} {} {} {} {} {} {} {}",
                        cell.id,
                        coord.x,
                        coord.y,
                        coord.z,
                        cell.visible,
                        cell.solid,
                        cell.anchored,
                        cell.texture,
                        cell.color_rgb.x,
                        cell.color_rgb.y,
                        cell.color_rgb.z,
                        cell.entity_identity.as_deref().unwrap_or("None"),
                        cell.collision_events_enabled
                    )?;
                }

                _ => {}
            }

            if !cell.attributes.is_empty() {
                let json = serde_json::to_string(&cell.attributes)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
                writeln!(file, "ATTRIBUTES {} {}", cell.id, json)?;
            }
        }
    }

    Ok(())
}

fn parse_block_properties(cell: &mut Cell, parts: &[&str], offset: usize) {
    cell.visible = parts[offset + 4].parse::<bool>().unwrap_or(true);
    cell.solid = parts[offset + 5].parse::<bool>().unwrap_or(true);
    cell.anchored = parts[offset + 6].parse::<bool>().unwrap_or(true);
    cell.texture = parts[offset + 7].to_string();
    let r = parts[offset + 8].parse::<f32>().unwrap_or(0.5);
    let g = parts[offset + 9].parse::<f32>().unwrap_or(0.5);
    let b = parts[offset + 10].parse::<f32>().unwrap_or(0.5);
    cell.color_rgb = Vec3::new(r, g, b);
}

/// We clear and reload the world from disk. This supports the standard project
/// save format used by the engine.
pub fn load_world(world: &mut World, path: &Path) -> std::io::Result<()> {
    *world = World::new();
    let mut has_sky_line = false;

    if !path.exists() {
        return Ok(());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut legacy_cells = Vec::new();
    let mut legacy_bindings = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }

        // We check for global settings first.
        if parts[0] == "GRAVITY" && parts.len() >= 4 {
            // ... (keep gravity parsing)
            let x = parts[1].parse::<f32>().ok();
            let y = parts[2].parse::<f32>().ok();
            let z = parts[3].parse::<f32>().ok();

            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                world.gravity = Vec3::new(x, y, z);
            }
            continue;
        }

        if parts[0] == "LIGHTING" && parts.len() >= 11 {
            // ... (keep lighting parsing)
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

        if parts[0] == "MOUSE" && parts.len() >= 3 {
            world.cursor_visible = parts[1].parse::<bool>().unwrap_or(false);
            world.screen_locked = parts[2].parse::<bool>().unwrap_or(true);
            continue;
        }

        if parts[0] == "SKY" && parts.len() >= 2 {
            has_sky_line = true;
            world.sky.enabled = parts[1].parse::<bool>().unwrap_or(false);
            if parts.len() >= 3 {
                let tex = parts[2];
                world.sky.texture = if tex == "None" {
                    String::new()
                } else {
                    tex.to_string()
                };
            }

            // Infer preset from texture name
            if world.sky.texture.contains("Tropical") {
                world.sky.preset = "Tropical".to_string();
            } else if world.sky.texture.contains("Desert") {
                world.sky.preset = "Desert".to_string();
            } else if world.sky.texture.contains("Snowy") {
                world.sky.preset = "Snowy".to_string();
            } else if world.sky.texture.contains("Mars") {
                world.sky.preset = "Mars".to_string();
            } else {
                world.sky.preset = "Temperate".to_string();
            }
            continue;
        }

        if parts[0] == "SCRIPT_BINDING" && parts.len() >= 3 {
            let target_str = parts[1];
            let script_path = parts[2].to_string();

            if let Ok(id) = target_str.parse::<u64>() {
                let mut binding = ScriptBinding::new(id, script_path);
                if parts.len() >= 4 {
                    binding.enabled = parts[3].parse::<bool>().unwrap_or(true);
                }
                world.script_bindings.push(binding);
            } else {
                legacy_bindings.push((target_str.to_string(), script_path));
            }
            continue;
        }

        if parts[0] == "DISABLED_SCRIPT" && parts.len() >= 2 {
            world.disabled_scripts.push(parts[1].to_string());
            continue;
        }

        if parts[0] == "SELECTED_CHARACTER" && parts.len() >= 2 {
            world.selected_character = parts[1].to_string();
            continue;
        }

        if parts[0] == "SELECTED_CONTROLLER" && parts.len() >= 2 {
            world.selected_controller = parts[1].to_string();
            continue;
        }

        if parts[0] == "SELECTED_CAMERA" && parts.len() >= 2 {
            world.selected_camera = parts[1].to_string();
            continue;
        }

        if parts[0] == "ATTRIBUTES" && parts.len() >= 3 {
            let id_val = parts[1].parse::<u64>().unwrap_or(0);
            // We use splitn to ensure we preserve spaces in the JSON payload.
            let sub_parts: Vec<&str> = line.splitn(3, ' ').collect();
            if sub_parts.len() == 3 {
                let json_str = sub_parts[2];
                if let Some(coord) = world.resolve_cell_id(id_val) {
                    if let Some(cell) = world.get_mut(coord) {
                        if let Ok(attrs) = serde_json::from_str(json_str) {
                            cell.attributes = attrs;
                        }
                    }
                }
            }
            continue;
        }

        if parts.len() >= 4 {
            let block_type = parts[0];

            let has_id = if block_type == "BLOCK"
                || block_type == "SPAWN_POINT"
                || block_type == "LIGHT"
                || block_type == "PLAYER"
                || block_type == "NPC"
                || block_type == "FX_BLOCK"
                || block_type == "AUDIO_EMITTER"
            {
                let legacy_len = match block_type {
                    "BLOCK" | "SPAWN_POINT" | "LIGHT" => 11,
                    "PLAYER" | "NPC" | "FX_BLOCK" => 12,
                    "AUDIO_EMITTER" => 13,
                    _ => 0,
                };
                parts.len() > legacy_len
            } else {
                false
            };

            let offset = if has_id { 1 } else { 0 };

            let x = parts[offset + 1].parse::<i32>().ok();
            let y = parts[offset + 2].parse::<i32>().ok();
            let z = parts[offset + 3].parse::<i32>().ok();

            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                let coord = WorldCoord::new(x, y, z);
                let id_val = if has_id {
                    parts[1].parse::<u64>().unwrap_or(0)
                } else {
                    0
                };
                let is_legacy = id_val == 0;

                let old_id = if block_type == "BLOCK" {
                    world.set_cell(coord, CellType::Block)
                } else if block_type == "FX_BLOCK" {
                    world.set_cell(coord, CellType::FxBlock)
                } else if block_type == "SPAWN_POINT" {
                    world.set_cell(coord, CellType::SpawnPoint)
                } else if block_type == "PLAYER" {
                    world.set_cell(coord, CellType::Player)
                } else if block_type == "NPC" {
                    world.set_cell(coord, CellType::NPC)
                } else if block_type == "LIGHT" {
                    world.set_cell(coord, CellType::Light)
                } else if block_type == "AUDIO_EMITTER" {
                    world.set_cell(coord, CellType::AudioEmitter)
                } else {
                    0
                };

                if is_legacy {
                    legacy_cells.push(coord);
                } else {
                    world.update_id_mapping(old_id, id_val, coord);
                }

                if let Some(cell) = world.get_mut(coord) {
                    if !is_legacy {
                        cell.id = id_val;
                    }

                    if block_type == "LIGHT" {
                        let r = parts[offset + 4].parse::<f32>().unwrap_or(1.0);
                        let g = parts[offset + 5].parse::<f32>().unwrap_or(1.0);
                        let b = parts[offset + 6].parse::<f32>().unwrap_or(1.0);
                        cell.light_color = Vec3::new(r, g, b);
                        cell.light_intensity = parts[offset + 7].parse::<f32>().unwrap_or(5.0);
                        cell.light_range = parts[offset + 8].parse::<f32>().unwrap_or(10.0);
                        cell.light_shadows = parts[offset + 9].parse::<bool>().unwrap_or(true);
                        if parts.len() >= offset + 11 {
                            cell.light_enabled = parts[offset + 10].parse::<bool>().unwrap_or(true);
                        }
                        if parts.len() >= offset + 12 {
                            let identity = parts[offset + 11];
                            if identity != "None" {
                                cell.entity_identity = Some(identity.to_string());
                            }
                        }
                        if parts.len() >= offset + 13 {
                            cell.collision_events_enabled =
                                parts[offset + 12].parse::<bool>().unwrap_or(true);
                        }
                    } else if block_type == "AUDIO_EMITTER" {
                        parse_block_properties(cell, &parts, offset);
                        if parts.len() >= offset + 12 {
                            let identity = parts[offset + 11];
                            if identity != "None" {
                                cell.entity_identity = Some(identity.to_string());
                            }
                        }
                        if parts.len() >= offset + 13 {
                            cell.collision_events_enabled =
                                parts[offset + 12].parse::<bool>().unwrap_or(true);
                        }
                        if parts.len() >= offset + 17 {
                            cell.playing = parts[offset + 13].parse::<bool>().unwrap_or(false);
                            cell.looped = parts[offset + 14].parse::<bool>().unwrap_or(false);
                            cell.volume = parts[offset + 15].parse::<f32>().unwrap_or(1.0);
                            let audio_val = parts[offset + 16..].join(" ");
                            cell.audio = if audio_val == "None" {
                                String::new()
                            } else {
                                audio_val
                            };
                        } else if parts.len() >= offset + 14 {
                            let audio_val = parts[offset + 13..].join(" ");
                            cell.audio = if audio_val == "None" {
                                String::new()
                            } else {
                                audio_val
                            };
                        }
                    } else {
                        parse_block_properties(cell, &parts, offset);
                        if parts.len() >= offset + 12 {
                            let identity = parts[offset + 11];
                            if identity != "None" {
                                cell.entity_identity = Some(identity.to_string());
                            }
                        }
                        if parts.len() >= offset + 13 {
                            cell.collision_events_enabled =
                                parts[offset + 12].parse::<bool>().unwrap_or(true);
                        }
                    }
                }
            }
        }
    }

    // Migration: Assign IDs to legacy cells and rebuild index.
    for coord in legacy_cells {
        let cell_type = world.get(coord).unwrap().cell_type;
        let new_id = world.generate_unique_id(coord, cell_type);
        if let Some(cell) = world.get_mut(coord) {
            cell.id = new_id;
        }
    }
    world.rebuild_id_mapping();

    // Migration: Resolve legacy script bindings.
    for (name, script_path) in legacy_bindings {
        let mut matches = Vec::new();
        for cell in world.cells.values() {
            if let Some(ref identity) = cell.entity_identity {
                if identity == &name {
                    matches.push(cell.id);
                }
            }
        }

        if matches.len() == 1 {
            world
                .script_bindings
                .push(ScriptBinding::new(matches[0], script_path));
        } else if matches.is_empty() {
            println!("Stale legacy script binding found for identity '{}'", name);
        } else {
            eprintln!(
                "Migration ambiguity: multiple cells found for legacy script binding identity '{}'.",
                name
            );
        }
    }

    if !has_sky_line {
        world.sky.enabled = false;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::Cell;
    use crate::world::cell::AttributeValue;
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
            cell.light_enabled = false;
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
        assert_eq!(loaded_cell.light_enabled, false);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_light_default_enabled_legacy_load() {
        let path = Path::new("test_light_legacy.dat");
        {
            let mut file = File::create(path).unwrap();
            // Old format: 10 parts (LIGHT x y z r g b intensity range shadows)
            // Missing 11th part (enabled)
            writeln!(file, "LIGHT 5 5 5 1 1 1 5 10 true").unwrap();
        }

        let mut world = World::new();
        load_world(&mut world, path).unwrap();

        let cell = world.get(WorldCoord::new(5, 5, 5)).unwrap();
        assert_eq!(cell.cell_type, CellType::Light);
        assert_eq!(cell.light_enabled, true); // Should default to true

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
        world
            .script_bindings
            .push(ScriptBinding::new(12345678, "scripts/player.aeo"));

        let path = Path::new("test_one_binding.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(loaded_world.script_bindings.len(), 1);
        assert_eq!(loaded_world.script_bindings[0].target_identity, 12345678);
        assert_eq!(
            loaded_world.script_bindings[0].script_path,
            "scripts/player.aeo"
        );
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_multiple_bindings_persistence() {
        let mut world = World::new();
        world
            .script_bindings
            .push(ScriptBinding::new(10000001, "scripts/player.aeo"));
        world
            .script_bindings
            .push(ScriptBinding::new(10000002, "scripts/door.aeo"));
        world
            .script_bindings
            .push(ScriptBinding::new(10000003, "scripts/enemy.aeo"));

        let path = Path::new("test_multi_bindings.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(loaded_world.script_bindings.len(), 3);
        assert_eq!(loaded_world.script_bindings[0].target_identity, 10000001);
        assert_eq!(loaded_world.script_bindings[1].target_identity, 10000002);
        assert_eq!(loaded_world.script_bindings[2].target_identity, 10000003);
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_legacy_binding_migration() {
        let mut world = World::new();
        let coord = WorldCoord::new(1, 1, 1);
        // We use a fixed ID to ensure it matches the file content we'll write.
        let cell_id = 55555555;

        let path = Path::new("test_legacy_migration.dat");
        {
            let mut file = File::create(path).unwrap();
            writeln!(file, "GRAVITY 0 -9.81 0").unwrap();
            writeln!(file, "LIGHTING true true 0.5 -1.0 0.5 1 1 1 1 0.2").unwrap();
            writeln!(file, "SCRIPT_BINDING PlayerOne scripts/player.aeo").unwrap();
            // Format: PLAYER <id> <x> <y> <z> <visible> <solid> <anchored> <texture> <r> <g> <b> <identity>
            writeln!(
                file,
                "PLAYER {} 1 1 1 true true false Block_tx 0.5 0.5 0.5 PlayerOne",
                cell_id
            )
            .unwrap();
        }

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(loaded_world.script_bindings.len(), 1);
        assert_eq!(loaded_world.script_bindings[0].target_identity, cell_id);
        assert_eq!(
            loaded_world.script_bindings[0].script_path,
            "scripts/player.aeo"
        );

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_existing_data_and_bindings_persistence() {
        let mut world = World::new();
        let coord = WorldCoord::new(1, 2, 3);
        let cell_id = world.set_cell(coord, CellType::Block);
        world
            .script_bindings
            .push(ScriptBinding::new(cell_id, "scripts/box.aeo"));

        let path = Path::new("test_mixed_data.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert!(loaded_world.get(coord).is_some());
        assert_eq!(loaded_world.script_bindings.len(), 1);
        assert_eq!(loaded_world.script_bindings[0].target_identity, cell_id);
        fs::remove_file(path).ok();
    }

    #[test]
    fn test_entity_identity_persistence() {
        let mut world = World::new();
        let p_coord = WorldCoord::new(1, 1, 1);
        let n_coord = WorldCoord::new(2, 2, 2);
        let b_coord = WorldCoord::new(3, 3, 3);

        world.set_cell(p_coord, CellType::Player);
        if let Some(cell) = world.get_mut(p_coord) {
            cell.entity_identity = Some("Hero".to_string());
        }

        world.set_cell(n_coord, CellType::NPC);
        if let Some(cell) = world.get_mut(n_coord) {
            cell.entity_identity = Some("Merchant".to_string());
        }

        world.set_cell(b_coord, CellType::Block);
        // Blocks should have None (non-entity)

        let path = Path::new("test_identity.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        let p_cell = loaded_world.get(p_coord).unwrap();
        assert_eq!(p_cell.cell_type, CellType::Player);
        assert_eq!(p_cell.entity_identity, Some("Hero".to_string()));

        let n_cell = loaded_world.get(n_coord).unwrap();
        assert_eq!(n_cell.cell_type, CellType::NPC);
        assert_eq!(n_cell.entity_identity, Some("Merchant".to_string()));

        let b_cell = loaded_world.get(b_coord).unwrap();
        assert_eq!(b_cell.cell_type, CellType::Block);
        assert_eq!(b_cell.entity_identity, None);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_identity_not_derived_dynamically() {
        let mut world = World::new();
        let coord = WorldCoord::new(10, 20, 30);
        world.set_cell(coord, CellType::NPC);

        if let Some(cell) = world.get_mut(coord) {
            cell.entity_identity = Some("SpecificGuard".to_string());
        }

        // 1. Stability across coordinate change (not derived from coords)
        let new_coord = WorldCoord::new(40, 50, 60);
        let cell_id = world.get(coord).unwrap().id;
        let cell = world.cells.remove(&coord).unwrap();
        world.cells.insert(new_coord, cell);
        world.rebuild_id_mapping();

        assert_eq!(
            world.get(new_coord).unwrap().entity_identity,
            Some("SpecificGuard".to_string())
        );
        assert_eq!(world.resolve_cell_id(cell_id), Some(new_coord));

        // 2. Stability across entity-type transition (Player <-> NPC)
        if let Some(cell) = world.get_mut(new_coord) {
            cell.cell_type = CellType::Player;
        }
        assert_eq!(
            world.get(new_coord).unwrap().entity_identity,
            Some("SpecificGuard".to_string())
        );

        // 3. Invariant: non-entity cells MUST have None identity.
        // If we manually change type to a non-entity, we must also clear identity.
        // This exercises the expected model invariant that only Player/NPC hold identities.
        if let Some(cell) = world.get_mut(new_coord) {
            cell.cell_type = CellType::Block;
            cell.entity_identity = None;
        }
        assert_eq!(world.get(new_coord).unwrap().entity_identity, None);
    }

    #[test]
    fn test_legacy_identity_load() {
        let path = Path::new("test_legacy_identity.dat");
        {
            let mut file = File::create(path).unwrap();
            // Old format: 11 parts for PLAYER/NPC (if they existed, but here we simulate a missing 12th part)
            writeln!(file, "PLAYER 0 0 0 true true true Default 0.5 0.5 0.5").unwrap();
        }

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        let cell = loaded_world.get(WorldCoord::new(0, 0, 0)).unwrap();
        assert_eq!(cell.cell_type, CellType::Player);
        assert_eq!(cell.entity_identity, None);

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

    #[test]
    fn test_build_path_id_preservation() {
        // This exercises the fix for the Build tool bug where generated IDs
        // were overwritten by the template's zero ID.
        let mut world = World::new();
        let coord = WorldCoord::new(5, 5, 5);

        // 1. Template has ID 0.
        let template = Cell::new_block();
        assert_eq!(template.id, 0);

        // 2. World generates a real ID.
        world.set_cell(coord, template.cell_type);
        let generated_id = world.get(coord).unwrap().id;
        assert_ne!(generated_id, 0);

        // 3. Apply fix-pattern (simulating EditorTool::Build logic in App::on_mouse_button).
        if let Some(target) = world.get_mut(coord) {
            let id = target.id;
            *target = template.clone();
            target.id = id;
        }

        // 4. Verify the World cell kept the real ID.
        assert_eq!(world.get(coord).unwrap().id, generated_id);
    }

    #[test]
    fn test_corrupted_id_migration() {
        // Verify that loading a file with zero IDs (produced by the Build bug)
        // correctly migrates them to new unique non-zero IDs while preserving
        // valid existing IDs.
        let path = Path::new("test_corrupted_ids.dat");
        {
            let mut file = File::create(path).unwrap();
            // Corrupted record 1 (id=0)
            writeln!(file, "BLOCK 0 10 10 10 true true true Default 0.5 0.5 0.5").unwrap();
            // Corrupted record 2 (id=0)
            writeln!(file, "LIGHT 0 20 20 20 1 1 1 5 10 true true").unwrap();
            // Valid record (id=88888888)
            writeln!(
                file,
                "BLOCK 88888888 30 30 30 true true true Default 0.1 0.2 0.3"
            )
            .unwrap();
        }

        let mut world = World::new();
        load_world(&mut world, path).unwrap();

        let c1 = world.get(WorldCoord::new(10, 10, 10)).unwrap();
        let c2 = world.get(WorldCoord::new(20, 20, 20)).unwrap();
        let c3 = world.get(WorldCoord::new(30, 30, 30)).unwrap();

        // All zero IDs become non-zero.
        assert_ne!(c1.id, 0);
        assert_ne!(c2.id, 0);

        // Resulting IDs are unique.
        assert_ne!(c1.id, c2.id);
        assert_ne!(c1.id, c3.id);
        assert_ne!(c2.id, c3.id);

        // Valid pre-existing IDs remain unchanged.
        assert_eq!(c3.id, 88888888);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_cell_attributes_defaults() {
        let cell = Cell::default();
        assert!(cell.attributes.is_empty());

        let light = Cell::new_light();
        assert!(light.attributes.is_empty());
    }

    #[test]
    fn test_cell_attributes_clone() {
        let mut cell = Cell::default();
        cell.attributes
            .insert("health".to_string(), AttributeValue::Number(100.0));
        cell.attributes.insert(
            "name".to_string(),
            AttributeValue::String("Player".to_string()),
        );

        let cloned = cell.clone();
        assert_eq!(cloned.attributes, cell.attributes);
        assert_eq!(
            cloned.attributes.get("health"),
            Some(&AttributeValue::Number(100.0))
        );
    }

    #[test]
    fn test_world_attributes_persistence() {
        let mut world = World::new();
        let coord = WorldCoord::new(1, 1, 1);
        let id = world.set_cell(coord, CellType::Block);

        if let Some(cell) = world.get_mut(coord) {
            cell.attributes
                .insert("score".to_string(), AttributeValue::Number(123.45));
            cell.attributes
                .insert("is_active".to_string(), AttributeValue::Bool(true));
            cell.attributes.insert(
                "description".to_string(),
                AttributeValue::String("A block with spaces".to_string()),
            );
        }

        let path = Path::new("test_attributes.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        let loaded_cell = loaded_world.get(coord).unwrap();
        assert_eq!(loaded_cell.id, id);
        assert_eq!(
            loaded_cell.attributes.get("score"),
            Some(&AttributeValue::Number(123.45))
        );
        assert_eq!(
            loaded_cell.attributes.get("is_active"),
            Some(&AttributeValue::Bool(true))
        );
        assert_eq!(
            loaded_cell.attributes.get("description"),
            Some(&AttributeValue::String("A block with spaces".to_string()))
        );

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_world_multiple_attributes_persistence() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(1, 1, 1);

        world.set_cell(c1, CellType::Player);
        if let Some(cell) = world.get_mut(c1) {
            cell.attributes
                .insert("speed".to_string(), AttributeValue::Number(5.0));
        }

        world.set_cell(c2, CellType::NPC);
        if let Some(cell) = world.get_mut(c2) {
            cell.attributes
                .insert("aggro".to_string(), AttributeValue::Bool(false));
            cell.attributes.insert(
                "greeting".to_string(),
                AttributeValue::String("Hello traveler".to_string()),
            );
        }

        let path = Path::new("test_multi_attributes.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(
            loaded_world.get(c1).unwrap().attributes.get("speed"),
            Some(&AttributeValue::Number(5.0))
        );
        assert_eq!(
            loaded_world.get(c2).unwrap().attributes.get("aggro"),
            Some(&AttributeValue::Bool(false))
        );
        assert_eq!(
            loaded_world.get(c2).unwrap().attributes.get("greeting"),
            Some(&AttributeValue::String("Hello traveler".to_string()))
        );

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_legacy_load_empty_attributes() {
        let path = Path::new("test_legacy_no_attrs.dat");
        {
            let mut file = File::create(path).unwrap();
            writeln!(file, "GRAVITY 0 -9.81 0").unwrap();
            writeln!(file, "BLOCK 12345 0 0 0 true true true Default 0.5 0.5 0.5").unwrap();
        }

        let mut world = World::new();
        load_world(&mut world, path).unwrap();

        let cell = world.get(WorldCoord::new(0, 0, 0)).unwrap();
        assert!(cell.attributes.is_empty());

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_world_mouse_settings_persistence() {
        let mut world = World::new();
        world.cursor_visible = true;
        world.screen_locked = false;

        let path = Path::new("test_mouse_settings.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        assert_eq!(loaded_world.cursor_visible, true);
        assert_eq!(loaded_world.screen_locked, false);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_empty_attributes_no_line() {
        let mut world = World::new();
        world.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);

        let path = Path::new("test_empty_attr_line.dat");
        save_world(&world, path).unwrap();

        let content = fs::read_to_string(path).unwrap();
        assert!(!content.contains("ATTRIBUTES"));

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_persistence_firewall_regression() {
        // 1. Create an authored Cell.
        let mut world = World::new();
        let auth_coord = WorldCoord::new(1, 1, 1);
        let auth_id = world.set_cell(auth_coord, CellType::Block);

        // 2. Create a runtime Cell through the runtime API.
        let runtime_id = world.create_runtime_cell(CellType::Block);
        let runtime_coord = WorldCoord::new(2, 2, 2);
        world.move_runtime_cell(runtime_id, runtime_coord).unwrap();

        // 3. Change the runtime Cell's properties.
        if let Some(cell) = world.runtime_cells.get_mut(&runtime_id) {
            cell.color_rgb = Vec3::new(1.0, 0.0, 0.0);
            cell.attributes
                .insert("temp".to_string(), AttributeValue::Bool(true));
        }

        // 4. Call save_world(...).
        let path = Path::new("test_persistence_firewall.dat");
        save_world(&world, path).unwrap();

        // 5. Read the saved file.
        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        // 6. Verify the runtime Cell does NOT appear.
        assert!(loaded_world.resolve_cell_id(runtime_id).is_none());
        assert!(loaded_world.get(runtime_coord).is_none());

        // 7. Verify the authored Cell still appears normally.
        assert!(loaded_world.resolve_cell_id(auth_id).is_some());
        assert!(loaded_world.get(auth_coord).is_some());

        // 8. Clear runtime state.
        world.clear_runtime_state();

        // 9. Verify the runtime Cell is gone.
        assert!(world.runtime_cells.is_empty());
        assert!(world.resolve_cell_id(runtime_id).is_none());

        // 10. Verify the authored Cell remains unchanged.
        assert!(world.get(auth_coord).is_some());
        assert_eq!(world.get(auth_coord).unwrap().id, auth_id);

        fs::remove_file(path).ok();
    }

    #[test]
    fn test_audio_emitter_persistence() {
        let mut world = World::new();
        let coord = WorldCoord::new(2, 4, 6);
        let id = world.set_cell(coord, CellType::AudioEmitter);

        if let Some(cell) = world.get_mut(coord) {
            cell.audio = "battle/sword-unsheathe.wav".to_string();
        }

        let path = Path::new("test_audio_emitter.dat");
        save_world(&world, path).unwrap();

        let mut loaded_world = World::new();
        load_world(&mut loaded_world, path).unwrap();

        let loaded_cell = loaded_world.get(coord).unwrap();
        assert_eq!(loaded_cell.id, id);
        assert_eq!(loaded_cell.cell_type, CellType::AudioEmitter);
        assert_eq!(loaded_cell.audio, "battle/sword-unsheathe.wav");

        fs::remove_file(path).ok();
    }
}
