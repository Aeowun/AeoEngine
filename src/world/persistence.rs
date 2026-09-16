use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use glam::Vec3;
use super::world::World;
use super::cell::CellType;
use super::coordinate::WorldCoord;

pub fn save_world(world: &World, path: &Path) -> std::io::Result<()> {
    let mut file = File::create(path)?;

    // Save global settings
    writeln!(file, "GRAVITY {} {} {}", world.gravity.x, world.gravity.y, world.gravity.z)?;

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
        if parts.is_empty() { continue; }

        if parts[0] == "GRAVITY" && parts.len() >= 4 {
            let x = parts[1].parse::<f32>().ok();
            let y = parts[2].parse::<f32>().ok();
            let z = parts[3].parse::<f32>().ok();
            if let (Some(x), Some(y), Some(z)) = (x, y, z) {
                world.gravity = Vec3::new(x, y, z);
            }
            continue;
        }

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use glam::Vec3;

    #[test]
    fn test_world_default_gravity() {
        let world = World::new();
        assert_eq!(world.gravity, Vec3::new(0.0, -9.81, 0.0));
    }

    #[test]
    fn test_world_gravity_persistence() {
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
}
