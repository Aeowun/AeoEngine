use std::collections::BTreeMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use super::WORLD_FORMAT_VERSION;
use crate::world::cell::{Cell, ChunkCoord};
use crate::world::coordinate::WorldCoord;
use crate::world::storage::WorldStorage;
use crate::world::world::World;

/// We save the world to a simple text format. This is easier to debug and
/// version than a binary format for now.
pub fn save_world(world: &World, path: &Path) -> std::io::Result<()> {
    let project_root = path.parent().unwrap_or_else(|| Path::new("."));
    let storage = WorldStorage::new(project_root)?;

    let mut file = File::create(path)?;

    writeln!(file, "WORLD_FORMAT {}", WORLD_FORMAT_VERSION)?;
    writeln!(file, "NEXT_CELL_ID {}", world.next_cell_id)?;
    writeln!(
        file,
        "GRAVITY {} {} {}",
        world.gravity.x, world.gravity.y, world.gravity.z
    )?;

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

    // Group all authored cells by their owning chunk.
    let mut cells_by_chunk: BTreeMap<ChunkCoord, Vec<(WorldCoord, Cell)>> = BTreeMap::new();

    for coord in world.active_blocks() {
        if let Some(cell) = world.get(coord) {
            let chunk_coord = ChunkCoord::from_world_coord(coord);

            cells_by_chunk
                .entry(chunk_coord)
                .or_default()
                .push((coord, cell.clone()));
        }
    }

    // Remove chunk files that no longer have any authored cells.
    let existing_chunks = storage.list_chunks()?;

    for chunk_coord in existing_chunks {
        if !cells_by_chunk.contains_key(&chunk_coord) {
            storage.delete_chunk(chunk_coord)?;
        }
    }

    // Write the current chunks.
    for (chunk_coord, cells) in &cells_by_chunk {
        let stored_chunk = WorldStorage::create_stored_chunk(*chunk_coord, cells.clone())?;

        storage.save_chunk(&stored_chunk)?;
    }

    Ok(())
}
