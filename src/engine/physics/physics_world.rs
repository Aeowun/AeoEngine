use super::physics_body::{CollisionRecord, PhysicsBody, PhysicsBodyId, PhysicsIdGenerator};
use crate::world::{CellType, ChunkCoord, World, WorldCoord};
use glam::Vec3;
use std::collections::HashMap;

pub struct PhysicsWorld {
    pub bodies: Vec<PhysicsBody>,

    // Static colliders are blocks that are solid and anchored.
    // They are stored with their effective runtime position.
    pub static_colliders: Vec<(u64, Vec3)>,
    pub static_colliders_by_chunk: HashMap<ChunkCoord, Vec<(u64, Vec3)>>,

    pub(crate) id_gen: PhysicsIdGenerator,

    // Step counter used to throttle console output.
    pub(crate) step_count: u64,
}

impl PhysicsWorld {
    pub fn new() -> Self {
        Self {
            bodies: Vec::new(),
            static_colliders: Vec::new(),
            static_colliders_by_chunk: HashMap::new(),
            id_gen: PhysicsIdGenerator::new(),
            step_count: 0,
        }
    }

    pub fn rebuild_static_chunk_index(&mut self) {
        self.static_colliders_by_chunk.clear();
        for &(cell_id, pos) in &self.static_colliders {
            let coord = WorldCoord::from_vec3(pos);
            let chunk_coord = ChunkCoord::from_world_coord(coord);
            self.static_colliders_by_chunk
                .entry(chunk_coord)
                .or_default()
                .push((cell_id, pos));
        }
    }

    pub fn query_static_colliders_in_aabb(&self, min_pos: Vec3, max_pos: Vec3) -> Vec<(u64, Vec3)> {
        let min_coord = WorldCoord::from_vec3(min_pos);
        let max_coord = WorldCoord::from_vec3(max_pos);

        let min_chunk = ChunkCoord::from_world_coord(min_coord);
        let max_chunk = ChunkCoord::from_world_coord(max_coord);

        let mut results = Vec::new();

        for cx in min_chunk.x..=max_chunk.x {
            for cy in min_chunk.y..=max_chunk.y {
                for cz in min_chunk.z..=max_chunk.z {
                    let chunk_coord = ChunkCoord::new(cx, cy, cz);
                    if let Some(list) = self.static_colliders_by_chunk.get(&chunk_coord) {
                        for &(id, pos) in list {
                            if pos.x >= min_pos.x - 1.0
                                && pos.x <= max_pos.x + 1.0
                                && pos.y >= min_pos.y - 1.0
                                && pos.y <= max_pos.y + 1.0
                                && pos.z >= min_pos.z - 1.0
                                && pos.z <= max_pos.z + 1.0
                            {
                                results.push((id, pos));
                            }
                        }
                    }
                }
            }
        }

        results
    }

    // Touching helper for dynamic bodies. normal_other_to_body points from other to body.
    pub(crate) fn check_body_touching_dynamic(
        &self,
        body: &PhysicsBody,
        other: &PhysicsBody,
        normal_other_to_body: Vec3,
    ) -> bool {
        let epsilon = 0.01;
        let test_min = body.min_corner() - normal_other_to_body * epsilon;
        let test_max = body.max_corner() - normal_other_to_body * epsilon;
        Self::aabb_overlap_static(test_min, test_max, other.min_corner(), other.max_corner())
    }

    // Touching helper for static blocks.
    pub(crate) fn check_body_touching_static(&self, body: &PhysicsBody, normal: Vec3) -> bool {
        let epsilon = 0.01;
        let test_min = body.min_corner() - normal * epsilon;
        let test_max = body.max_corner() - normal * epsilon;
        let candidates = self.query_static_colliders_in_aabb(test_min, test_max);
        for (_cell_id, v_min) in &candidates {
            let v_max = v_min + Vec3::ONE;
            if Self::aabb_overlap_static(test_min, test_max, *v_min, v_max) {
                return true;
            }
        }
        false
    }

    pub fn check_static_collisions(&self) -> Vec<CollisionRecord> {
        let mut collisions = Vec::new();
        for body in &self.bodies {
            if !body.solid {
                continue;
            }
            let min = body.min_corner();
            let max = body.max_corner();
            let candidates = self.query_static_colliders_in_aabb(min, max);
            for (_cell_id, v_min) in &candidates {
                let voxel_max = v_min + Vec3::ONE;
                if Self::aabb_overlap_static(min, max, *v_min, voxel_max) {
                    collisions.push(CollisionRecord {
                        body_id: body.id,
                        block_coord: WorldCoord::new(
                            v_min.x.round() as i32,
                            v_min.y.round() as i32,
                            v_min.z.round() as i32,
                        ),
                    });
                }
            }
        }
        collisions
    }

    pub(crate) fn aabb_overlap_static(a_min: Vec3, a_max: Vec3, b_min: Vec3, b_max: Vec3) -> bool {
        a_min.x < b_max.x
            && a_max.x > b_min.x
            && a_min.y < b_max.y
            && a_max.y > b_min.y
            && a_min.z < b_max.z
            && a_max.z > b_min.z
    }

    pub fn register_from_world(&mut self, world: &World) {
        self.bodies.clear();
        self.static_colliders.clear();
        self.id_gen = PhysicsIdGenerator::new();
        self.step_count = 0;
        for coord in world.iter_active_effective_coords() {
            if let Some(cell) = world.get_effective_cell(coord) {
                if cell.cell_type == CellType::Light {
                    continue;
                }

                let anchored = world.is_cell_anchored(coord);
                let solid = world.is_cell_solid(coord);

                if anchored {
                    if solid {
                        let pos = Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32)
                            + world.get_visual_offset(coord);
                        self.add_static_collider(cell.id, pos);
                    }
                } else {
                    let id = self.id_gen.next();
                    let pos = Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32);
                    let mut body = PhysicsBody::new(id, cell.id, pos, Vec3::ONE);

                    // Authoritative Property Transfer
                    body.cell_type = cell.cell_type;
                    body.visible = world.is_cell_visible(coord);
                    body.solid = solid;
                    body.anchored = anchored;
                    body.texture = cell.texture.clone();
                    body.color_rgb = world.get_effective_color(coord);

                    self.bodies.push(body);
                }
            }
        }
        self.rebuild_static_chunk_index();
        println!(
            "Physics: Registered {} dynamic bodies and {} static colliders.",
            self.bodies.len(),
            self.static_colliders.len()
        );
    }

    pub fn add_static_collider(&mut self, cell_id: u64, position: Vec3) {
        self.static_colliders.push((cell_id, position));
        self.rebuild_static_chunk_index();
    }

    /// Reconciles the physics simulation state with the World's effective state
    /// (authored data + runtime overrides).
    pub fn sync_with_world(&mut self, world: &mut World) {
        // 1. Process dirty cells (Create/Update/Remove static colliders and dynamic bodies).
        // This is O(number of changes) and handles all static-collider transitions.
        let dirty_ids: Vec<u64> = world.physics_dirty_cells.drain().collect();
        if dirty_ids.is_empty() {
            return;
        }
        for cell_id in dirty_ids {
            if let Some(coord) = world.resolve_cell_id(cell_id) {
                if let Some(cell) = world.get_effective_cell_by_id(cell_id) {
                    if cell.cell_type == CellType::Light {
                        self.static_colliders.retain(|(id, _)| *id != cell_id);
                        self.bodies.retain(|b| b.cell_id != cell_id);
                        continue;
                    }

                    let anchored = world.is_cell_anchored(coord);
                    let solid = world.is_cell_solid(coord);

                    if anchored {
                        // Transition/Reconcile Static
                        self.bodies.retain(|b| b.cell_id != cell_id);
                        if solid {
                            let pos = Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32)
                                + world.get_visual_offset(coord);
                            if let Some(existing) = self
                                .static_colliders
                                .iter_mut()
                                .find(|(id, _)| *id == cell_id)
                            {
                                existing.1 = pos;
                            } else {
                                self.static_colliders.push((cell_id, pos));
                            }
                        } else {
                            self.static_colliders.retain(|(id, _)| *id != cell_id);
                        }
                    } else {
                        // Transition/Reconcile Dynamic
                        self.static_colliders.retain(|(id, _)| *id != cell_id);
                        if solid {
                            if let Some(body) =
                                self.bodies.iter_mut().find(|b| b.cell_id == cell_id)
                            {
                                body.solid = true;
                                body.visible = world.is_cell_visible(coord);
                                body.color_rgb = world.get_effective_color(coord);
                            } else {
                                let id = self.id_gen.next();
                                let pos = Vec3::new(coord.x as f32, coord.y as f32, coord.z as f32);
                                let mut body = PhysicsBody::new(id, cell_id, pos, Vec3::ONE);
                                body.cell_type = cell.cell_type;
                                body.visible = world.is_cell_visible(coord);
                                body.solid = true;
                                body.anchored = false;
                                body.texture = cell.texture.clone();
                                body.color_rgb = world.get_effective_color(coord);
                                self.bodies.push(body);
                            }
                        } else {
                            self.bodies.retain(|b| b.cell_id != cell_id);
                        }
                    }
                } else {
                    // ID no longer resolves to a cell (e.g. Empty or removed runtime cell)
                    self.static_colliders.retain(|(id, _)| *id != cell_id);
                    self.bodies.retain(|b| b.cell_id != cell_id);
                }
            } else {
                // Cell was deleted
                self.static_colliders.retain(|(id, _)| *id != cell_id);
                self.bodies.retain(|b| b.cell_id != cell_id);
            }
        }

        self.rebuild_static_chunk_index();
    }
}
