use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use super::world::World;
use super::cell::CellType;
use super::coordinate::WorldCoord;

pub fn save_world(world: &World, path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    for coord in world.active_blocks() {
        if let Some(cell) = world.get(coord) {
            match cell.cell_type {
                CellType::Grass => {
                    writeln!(file, "GRASS {} {} {} {} {} {} {}",
                        coord.x, coord.y, coord.z,
                        cell.visible, cell.solid, cell.anchored, cell.texture
                    )?;
                }
                _ => {}
            }
        }
    }

    Ok(())
}

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
        if parts.len() >= 4 {
            let block_type = parts[0];
            let x = parts[1].parse::<i32>().ok();
            let y = parts[2].parse::<i32>().ok();
            let z = parts[3].parse::<i32>().ok();

            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                let coord = WorldCoord::new(x, y, z);
                if block_type == "GRASS" {
                    world.set_cell(coord, CellType::Grass);
                    if let Some(cell) = world.get_mut(coord) {
                        if parts.len() >= 8 {
                            cell.visible = parts[4].parse::<bool>().unwrap_or(true);
                            cell.solid = parts[5].parse::<bool>().unwrap_or(true);
                            cell.anchored = parts[6].parse::<bool>().unwrap_or(true);
                            cell.texture = parts[7].to_string();
                        } else if parts.len() >= 7 {
                            // Format without anchored
                            cell.visible = parts[4].parse::<bool>().unwrap_or(true);
                            cell.solid = parts[5].parse::<bool>().unwrap_or(true);
                            cell.anchored = true;
                            cell.texture = parts[6].to_string();
                        } else {
                            // Upgrade old format
                            cell.visible = true;
                            cell.solid = true;
                            cell.anchored = true;
                            cell.texture = "Grass_tx".to_string();
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
