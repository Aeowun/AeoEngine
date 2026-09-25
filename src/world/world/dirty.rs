use std::collections::HashMap;
use crate::world::cell::ChunkCoord;
use crate::world::coordinate::WorldCoord;
use super::World;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DirtyReason {
    /// Geometry, occupancy, visibility, solidity, anchored, or cell_type changed.
    /// Invalidation neighborhood: own chunk + 6 neighboring chunks.
    Geometry,
    /// Color, texture, or visual offset changed.
    /// Invalidation neighborhood: own chunk only.
    MaterialOrOffset,
}

impl World {
    pub fn render_revision(&self) -> u64 {
        self.render_revision
    }

    pub(crate) fn bump_render_revision(&mut self) {
        self.render_revision = self.render_revision.wrapping_add(1);
    }

    pub fn mark_render_dirty(&self, coord: WorldCoord, reason: DirtyReason) {
        let mut map = self.render_dirty_cells.borrow_mut();
        match map.get(&coord) {
            Some(&DirtyReason::Geometry) => {
                // Geometry is already the broader invalidation
            }
            _ => {
                map.insert(coord, reason);
            }
        }
    }

    pub fn drain_render_dirty_cells(&self) -> HashMap<WorldCoord, DirtyReason> {
        std::mem::take(&mut *self.render_dirty_cells.borrow_mut())
    }

    /// Marks a cell as needing physics reconciliation.
    pub fn mark_physics_dirty(&mut self, cell_id: u64) {
        self.physics_dirty_cells.insert(cell_id);
    }

    pub(crate) fn authored_chunk_coords(&self) -> std::collections::HashSet<ChunkCoord> {
        self.cells
            .keys()
            .map(|coord| ChunkCoord::from_world_coord(*coord))
            .collect()
    }

    pub fn is_authored_chunk_dirty(&self, chunk_coord: ChunkCoord) -> bool {
        self.dirty_authored_chunks.contains(&chunk_coord)
    }

    pub(crate) fn mark_authored_chunk_clean(&mut self, chunk_coord: ChunkCoord) {
        self.dirty_authored_chunks.remove(&chunk_coord);
    }
}
