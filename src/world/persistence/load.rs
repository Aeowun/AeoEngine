use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use glam::Vec3;
use serde_json;

use crate::scripting::binding::ScriptBinding;
use crate::world::cell::{Cell, CellType};
use crate::world::coordinate::WorldCoord;
use crate::world::storage::WorldStorage;
use crate::world::world::World;

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
    let mut world_format_version = 1;
    if !path.exists() {
        return Ok(());
    }

    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut legacy_bindings = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let parts: Vec<&str> = line.split_whitespace().collect();

        if parts.is_empty() {
            continue;
        }
        if parts[0] == "WORLD_FORMAT" && parts.len() >= 2 {
            world_format_version =
                parts[1].parse::<u32>().unwrap_or(1);
            continue;
        }
        if parts[0] == "NEXT_CELL_ID" && parts.len() >= 2 {
            if let Ok(next_id) = parts[1].parse::<u64>() {
                world.next_cell_id = next_id;
            }
            continue;
        }
        // We check for global settings first.
        if parts[0] == "GRAVITY" && parts.len() >= 4 {
            let x = parts[1].parse::<f32>().ok();
            let y = parts[2].parse::<f32>().ok();
            let z = parts[3].parse::<f32>().ok();

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

                if !is_legacy {
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
    if world_format_version >= 2 {
        let project_root =
            path.parent().unwrap_or_else(|| Path::new("."));

        let storage = WorldStorage::new(project_root)?;

        for chunk_coord in storage.list_chunks()? {
            let Some(stored_chunk) =
                storage.load_chunk(chunk_coord)?
            else {
                continue;
            };

            for (coord, cell) in stored_chunk.into_cells() {
                world.observe_authored_id(cell.id);
                world.cells.insert(coord, cell);
            }
        }

        world.rebuild_id_mapping();

        if !has_sky_line {
            world.sky.enabled = false;
        }

        return Ok(());
    }
    // Migration: Assign IDs to legacy cells and rebuild index.
    // Legacy records without IDs already received fresh IDs through set_cell().
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
