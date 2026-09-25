//! Bounded authored-world streaming.
//!
//! `World` is the resident query cache used by rendering, physics, and editor
//! code. This module keeps prefetched chunks separate until promotion.

use std::collections::{HashMap, HashSet};
use std::io;
use std::path::Path;

use super::{Cell, ChunkCoord, World, WorldCoord, WorldStorage};

#[derive(Clone, Debug)]
pub struct WorldChunk {
    pub coord: ChunkCoord,
    pub cells: HashMap<WorldCoord, Cell>,
}

impl WorldChunk {
    pub fn new(coord: ChunkCoord) -> Self {
        Self { coord, cells: HashMap::new() }
    }

    pub fn from_cells(
        coord: ChunkCoord,
        cells: impl IntoIterator<Item = (WorldCoord, Cell)>,
    ) -> io::Result<Self> {
        let mut chunk = Self::new(coord);
        for (cell_coord, cell) in cells {
            if ChunkCoord::from_world_coord(cell_coord) != coord {
                return Err(io::Error::new(io::ErrorKind::InvalidInput, "cell belongs to another chunk"));
            }
            chunk.cells.insert(cell_coord, cell);
        }
        Ok(chunk)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChunkResidency {
    Resident,
    Prefetched,
}

/// Policy and state for a bounded authored chunk cache. Distances use the
/// Chebyshev metric, producing a cube around the streaming focus.
#[derive(Clone, Debug)]
pub struct WorldStreamer {
    storage: WorldStorage,
    load_radius: i32,
    prefetch_radius: i32,
    unload_radius: i32,
    /// Chunks currently promoted into `World.cells`.
    resident: HashSet<ChunkCoord>,
    /// Loaded but not resident authored data. This cache never participates in
    /// renderer, physics, scripting, or normal World queries.
    prefetched: HashMap<ChunkCoord, WorldChunk>,
    residency: HashMap<ChunkCoord, ChunkResidency>,
    last_focus_chunk: Option<ChunkCoord>,
}

impl WorldStreamer {
    pub fn new(project_root: impl AsRef<Path>) -> io::Result<Self> {
        Ok(Self::with_radii(project_root, 4, 5, 7)?)
    }

    pub fn with_radii(
        project_root: impl AsRef<Path>, load_radius: i32, prefetch_radius: i32, unload_radius: i32,
    ) -> io::Result<Self> {
        if load_radius < 0 || load_radius > prefetch_radius || prefetch_radius > unload_radius {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "stream radii must satisfy 0 <= load <= prefetch <= unload"));
        }
        Ok(Self {
            storage: WorldStorage::new(project_root)?, load_radius, prefetch_radius, unload_radius,
            resident: HashSet::new(), prefetched: HashMap::new(),
            residency: HashMap::new(), last_focus_chunk: None,
        })
    }

    pub fn storage(&self) -> &WorldStorage { &self.storage }
    pub fn loaded_chunks(&self) -> impl Iterator<Item = ChunkCoord> + '_ {
        self.resident.iter().copied().chain(self.prefetched.keys().copied())
    }
    pub fn residency(&self, coord: ChunkCoord) -> Option<ChunkResidency> { self.residency.get(&coord).copied() }
    pub fn is_loaded(&self, coord: ChunkCoord) -> bool {
        self.resident.contains(&coord) || self.prefetched.contains_key(&coord)
    }
    pub fn is_resident(&self, coord: ChunkCoord) -> bool { self.resident.contains(&coord) }
    pub fn is_prefetched(&self, coord: ChunkCoord) -> bool { self.prefetched.contains_key(&coord) }

    /// Sets the resident distance in chunks and preserves the one/two-chunk
    /// prefetch/unload hysteresis. The next update reevaluates immediately.
    pub fn set_max_distance(&mut self, resident_radius: i32) {
        let resident_radius = resident_radius.max(0);
        if self.load_radius != resident_radius {
            self.load_radius = resident_radius;
            self.prefetch_radius = resident_radius + 1;
            self.unload_radius = resident_radius + 3;
            self.last_focus_chunk = None;
        }
    }

    /// Updates cache membership. I/O is deliberately synchronous at this
    /// boundary; callers can run this on an I/O worker without sharing `World`
    /// across threads. Only the resident radius enters `World`.
    pub fn update(&mut self, world: &mut World, focus: WorldCoord) -> io::Result<()> {
        let center = ChunkCoord::from_world_coord(focus);
        if self.last_focus_chunk == Some(center) {
            return Ok(());
        }
        let stored: HashSet<_> = self.storage.list_chunks()?.into_iter().collect();

        // Newly authored chunks originate in World rather than on disk. They
        // are already resident and must participate in later demotion/flush.
        for chunk in world.authored_chunk_coords() {
            if !self.prefetched.contains_key(&chunk) {
                self.resident.insert(chunk);
            }
        }

        for chunk in &stored {
            let distance = chunk_distance(*chunk, center);
            if distance <= self.prefetch_radius
                && !self.resident.contains(chunk)
                && !self.prefetched.contains_key(chunk)
            {
                self.prefetch_chunk(*chunk)?;
            }
        }

        let prefetched: Vec<_> = self.prefetched.keys().copied().collect();
        for chunk in prefetched {
            if chunk_distance(chunk, center) <= self.load_radius {
                self.promote_chunk(world, chunk)?;
            }
        }

        // Resident chunks that have just moved outside the resident radius
        // are demoted to the prefetch cache. Their authored edits remain in
        // memory until the chunk crosses the unload radius.
        let resident: Vec<_> = self.resident.iter().copied().collect();
        for chunk in resident {
            let distance = chunk_distance(chunk, center);
            if distance > self.load_radius && distance <= self.prefetch_radius {
                self.demote_chunk(world, chunk)?;
            }
        }

        let loaded: Vec<_> = self.loaded_chunks().collect();
        for chunk in loaded {
            if chunk_distance(chunk, center) > self.unload_radius {
                self.flush_and_evict(world, chunk)?;
            }
        }

        self.residency.clear();
        for chunk in &self.resident {
            let state = if chunk_distance(*chunk, center) <= self.load_radius {
                ChunkResidency::Resident
            } else {
                ChunkResidency::Prefetched
            };
            self.residency.insert(*chunk, state);
        }
        for chunk in self.prefetched.keys() {
            self.residency.insert(*chunk, ChunkResidency::Prefetched);
        }
        self.last_focus_chunk = Some(center);
        Ok(())
    }

    pub fn prefetch_chunk(&mut self, coord: ChunkCoord) -> io::Result<()> {
        if self.is_loaded(coord) { return Ok(()); }
        if let Some(stored) = self.storage.load_chunk(coord)? {
            let chunk = WorldChunk::from_cells(coord, stored.into_cells())?;
            self.prefetched.insert(coord, chunk);
        }
        Ok(())
    }

    pub fn promote_chunk(&mut self, world: &mut World, coord: ChunkCoord) -> io::Result<()> {
        if self.resident.contains(&coord) { return Ok(()); }
        if !self.prefetched.contains_key(&coord) { self.prefetch_chunk(coord)?; }
        if let Some(chunk) = self.prefetched.remove(&coord) {
            world.insert_loaded_chunk(chunk.cells);
        }
        self.resident.insert(coord);
        Ok(())
    }

    fn demote_chunk(&mut self, world: &mut World, coord: ChunkCoord) -> io::Result<()> {
        if !self.resident.remove(&coord) { return Ok(()); }
        let chunk = WorldChunk::from_cells(coord, world.evict_authored_chunk(coord))?;
        self.prefetched.insert(coord, chunk);
        Ok(())
    }

    pub fn flush_chunk(&self, world: &mut World, coord: ChunkCoord) -> io::Result<()> {
        if !world.is_authored_chunk_dirty(coord) { return Ok(()); }
        let cells: Vec<(WorldCoord, Cell)> = if let Some(chunk) = self.prefetched.get(&coord) {
            chunk.cells.iter().map(|(coord, cell)| (*coord, cell.clone())).collect()
        } else {
            world.authored_cells_in_chunk(coord)
        };
        let chunk = WorldChunk::from_cells(coord, cells)?;
        if chunk.cells.is_empty() { self.storage.delete_chunk(coord)?; }
        else { self.storage.save_chunk(&WorldStorage::create_stored_chunk(coord, chunk.cells)? )?; }
        world.mark_authored_chunk_clean(coord);
        Ok(())
    }

    pub fn flush_all(&self, world: &mut World) -> io::Result<()> {
        // A newly authored chunk may not have been loaded from disk first, so
        // dirty chunks—not only the current cache membership—must be flushed.
        let dirty: Vec<_> = world.dirty_authored_chunks.iter().copied().collect();
        for coord in dirty { self.flush_chunk(world, coord)?; }
        Ok(())
    }

    pub fn flush_and_evict(&mut self, world: &mut World, coord: ChunkCoord) -> io::Result<()> {
        self.flush_chunk(world, coord)?;
        if self.resident.remove(&coord) {
            world.evict_authored_chunk(coord);
        }
        self.prefetched.remove(&coord);
        world.mark_authored_chunk_clean(coord);
        self.residency.remove(&coord);
        Ok(())
    }

    /// Explicitly deletes every persisted authored chunk. This is only for a
    /// user-requested "clear world" operation, never ordinary eviction.
    pub fn delete_all_chunks(&mut self) -> io::Result<()> {
        for coord in self.storage.list_chunks()? { self.storage.delete_chunk(coord)?; }
        self.resident.clear();
        self.prefetched.clear();
        self.residency.clear();
        self.last_focus_chunk = None;
        Ok(())
    }
}

fn chunk_distance(a: ChunkCoord, b: ChunkCoord) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs()).max((a.z - b.z).abs())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::world::CellType;
    use std::fs;

    fn root(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("aeo_streamer_{}_{}", name, std::process::id()));
        fs::remove_dir_all(&root).ok();
        fs::create_dir_all(&root).unwrap(); root
    }

    #[test]
    fn streams_and_evicts_without_losing_authored_changes() {
        let root = root("evict");
        let storage = WorldStorage::new(&root).unwrap();
        let near = ChunkCoord::new(0, 0, 0);
        let far = ChunkCoord::new(10, 0, 0);
        let mut seed = World::new();
        seed.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);
        storage.save_chunk(&WorldStorage::create_stored_chunk(near, seed.authored_cells_in_chunk(near)).unwrap()).unwrap();
        seed.set_cell(WorldCoord::new(160, 0, 0), CellType::Light);
        storage.save_chunk(&WorldStorage::create_stored_chunk(far, seed.authored_cells_in_chunk(far)).unwrap()).unwrap();

        let mut world = World::new();
        let mut streamer = WorldStreamer::with_radii(&root, 0, 0, 1).unwrap();
        streamer.update(&mut world, WorldCoord::new(0, 0, 0)).unwrap();
        assert!(world.get(WorldCoord::new(0, 0, 0)).is_some());
        assert_eq!(world.resident_authored_cell_count(), 1);
        world.get_mut(WorldCoord::new(0, 0, 0)).unwrap().texture = "saved".into();

        streamer.update(&mut world, WorldCoord::new(160, 0, 0)).unwrap();
        assert!(world.get(WorldCoord::new(0, 0, 0)).is_none());
        assert!(world.get(WorldCoord::new(160, 0, 0)).is_some());
        assert_eq!(storage.load_chunk(near).unwrap().unwrap().into_cells()[0].1.texture, "saved");
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn update_is_a_noop_until_focus_crosses_a_chunk_boundary() {
        let root = root("focus_chunk");
        let storage = WorldStorage::new(&root).unwrap();
        let coord = ChunkCoord::new(0, 0, 0);
        let mut seed = World::new();
        seed.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);
        storage.save_chunk(&WorldStorage::create_stored_chunk(coord, seed.authored_cells_in_chunk(coord)).unwrap()).unwrap();

        let mut world = World::new();
        let mut streamer = WorldStreamer::with_radii(&root, 0, 0, 0).unwrap();
        streamer.update(&mut world, WorldCoord::new(0, 0, 0)).unwrap();
        assert!(streamer.is_loaded(coord));

        // A manually evicted chunk proves that a same-chunk update does not
        // perform another disk scan/load.
        streamer.flush_and_evict(&mut world, coord).unwrap();
        streamer.update(&mut world, WorldCoord::new(15, 15, 15)).unwrap();
        assert!(!streamer.is_loaded(coord));

        streamer.update(&mut world, WorldCoord::new(16, 0, 0)).unwrap();
        assert!(!streamer.is_loaded(coord));
        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn prefetch_cache_does_not_enter_world_until_promoted() {
        let root = root("prefetch");
        let storage = WorldStorage::new(&root).unwrap();
        let resident = ChunkCoord::new(0, 0, 0);
        let prefetched = ChunkCoord::new(1, 0, 0);
        let mut seed = World::new();
        seed.set_cell(WorldCoord::new(0, 0, 0), CellType::Block);
        storage.save_chunk(&WorldStorage::create_stored_chunk(resident, seed.authored_cells_in_chunk(resident)).unwrap()).unwrap();
        seed.set_cell(WorldCoord::new(16, 0, 0), CellType::Light);
        storage.save_chunk(&WorldStorage::create_stored_chunk(prefetched, seed.authored_cells_in_chunk(prefetched)).unwrap()).unwrap();

        let mut world = World::new();
        let mut streamer = WorldStreamer::with_radii(&root, 0, 1, 2).unwrap();
        streamer.update(&mut world, WorldCoord::new(0, 0, 0)).unwrap();
        assert!(streamer.is_resident(resident));
        assert!(streamer.is_prefetched(prefetched));
        assert!(world.get(WorldCoord::new(0, 0, 0)).is_some());
        assert!(world.get(WorldCoord::new(16, 0, 0)).is_none());
        assert!(world.physics_dirty_cells.is_empty());
        assert!(world.streaming_physics_rebuild_requested);

        streamer.update(&mut world, WorldCoord::new(16, 0, 0)).unwrap();
        assert!(streamer.is_resident(prefetched));
        assert!(world.get(WorldCoord::new(16, 0, 0)).is_some());
        assert!(world.physics_dirty_cells.is_empty());
        fs::remove_dir_all(root).ok();
    }
}
