use glam::{Mat4, Vec3};
use std::collections::HashMap;

use crate::engine::EditorMode;
use crate::renderer::mesh::{
    BLOCK_VERTEX_FLOATS, CubeFace, add_block_quad, upload_block_vertices_3d,
};
use crate::world::{CellType, World, WorldCoord};

pub const CHUNK_SIZE: i32 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkCoord {
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_world_coord(coord: WorldCoord) -> Self {
        Self {
            x: coord.x.div_euclid(CHUNK_SIZE),
            y: coord.y.div_euclid(CHUNK_SIZE),
            z: coord.z.div_euclid(CHUNK_SIZE),
        }
    }

    pub fn local_offset(coord: WorldCoord) -> (i32, i32, i32) {
        (
            coord.x.rem_euclid(CHUNK_SIZE),
            coord.y.rem_euclid(CHUNK_SIZE),
            coord.z.rem_euclid(CHUNK_SIZE),
        )
    }

    pub fn world_origin(&self) -> Vec3 {
        Vec3::new(
            (self.x * CHUNK_SIZE) as f32,
            (self.y * CHUNK_SIZE) as f32,
            (self.z * CHUNK_SIZE) as f32,
        )
    }

    pub fn aabb_min_max(&self) -> (Vec3, Vec3) {
        let min = self.world_origin();
        let max = min + Vec3::splat(CHUNK_SIZE as f32);
        (min, max)
    }
}

#[derive(Clone, Debug)]
pub struct TextureBatchRange {
    pub texture_id: String,
    pub start_vertex: i32,
    pub vertex_count: i32,
}

pub struct ChunkMesh {
    pub chunk_coord: ChunkCoord,
    pub main_vao: u32,
    pub main_vbo: u32,
    pub main_texture_batches: Vec<TextureBatchRange>,

    pub shadow_vao: u32,
    pub shadow_vbo: u32,
    pub shadow_vertex_count: i32,
}

impl ChunkMesh {
    pub fn free_gl_resources(&mut self) {
        unsafe {
            if self.main_vao != 0 {
                gl::DeleteVertexArrays(1, &self.main_vao);
                self.main_vao = 0;
            }
            if self.main_vbo != 0 {
                gl::DeleteBuffers(1, &self.main_vbo);
                self.main_vbo = 0;
            }
            if self.shadow_vao != 0 {
                gl::DeleteVertexArrays(1, &self.shadow_vao);
                self.shadow_vao = 0;
            }
            if self.shadow_vbo != 0 {
                gl::DeleteBuffers(1, &self.shadow_vbo);
                self.shadow_vbo = 0;
            }
        }
    }
}

impl Drop for ChunkMesh {
    fn drop(&mut self) {
        self.free_gl_resources();
    }
}

#[derive(Clone, Debug, Default)]
pub struct CpuChunkData {
    pub main_texture_vertices: HashMap<String, Vec<f32>>,
    pub shadow_positions: Vec<f32>,
}

pub fn build_cpu_chunk_data(
    world: &World,
    chunk_coord: ChunkCoord,
    coords_in_chunk: &[WorldCoord],
    mode: EditorMode,
) -> CpuChunkData {
    let mut main_texture_vertices: HashMap<String, Vec<f32>> = HashMap::new();
    let mut shadow_positions: Vec<f32> = Vec::new();

    for &coord in coords_in_chunk {
        let Some(cell) = world.get_effective_cell(coord) else {
            continue;
        };

        let (lx, ly, lz) = ChunkCoord::local_offset(coord);
        let local_pos = Vec3::new(lx as f32, ly as f32, lz as f32);
        let visual_offset = world.get_visual_offset(coord);
        let block_offset = local_pos + visual_offset;

        let should_draw = match mode {
            EditorMode::Editor => true,
            EditorMode::Play => world.is_cell_anchored(coord),
        };

        if !should_draw {
            continue;
        }

        // --- Main Pass Static Geometry ---
        let main_renderable = matches!(
            cell.cell_type,
            CellType::Block | CellType::SpawnPoint | CellType::Light
        );

        if main_renderable && world.is_cell_visible(coord) {
            let mask = crate::renderer::mesh::compute_exposed_faces_main(world, coord, mode);
            if mask != 0 {
                let color = world.get_effective_color(coord);
                let color_arr = [color.x, color.y, color.z, 1.0];
                let texture_id = cell.texture.clone();

                let vertices = main_texture_vertices
                    .entry(texture_id)
                    .or_insert_with(Vec::new);

                append_exposed_faces_main(vertices, block_offset, mask, color_arr);
            }
        }

        // --- Shadow Pass Static Geometry ---
        let shadow_renderable = matches!(cell.cell_type, CellType::Block | CellType::SpawnPoint);

        if shadow_renderable && world.is_cell_visible(coord) && world.is_cell_solid(coord) {
            let mask = crate::renderer::mesh::compute_exposed_faces_shadow(world, coord, mode);
            if mask != 0 {
                append_exposed_faces_shadow(&mut shadow_positions, block_offset, mask);
            }
        }
    }

    CpuChunkData {
        main_texture_vertices,
        shadow_positions,
    }
}

fn append_exposed_faces_main(vertices: &mut Vec<f32>, offset: Vec3, mask: u8, color: [f32; 4]) {
    for face in &CubeFace::ALL {
        if (mask & face.mask()) != 0 {
            add_face_quad_offset(vertices, offset, *face, color);
        }
    }
}

fn add_face_quad_offset(vertices: &mut Vec<f32>, offset: Vec3, face: CubeFace, color: [f32; 4]) {
    let min = 0.0;
    let max = 1.0;

    let (v1, v2, v3, v4, normal) = match face {
        CubeFace::Top => (
            [min, max, max],
            [max, max, max],
            [max, max, min],
            [min, max, min],
            [0.0, 1.0, 0.0],
        ),
        CubeFace::Bottom => (
            [min, min, min],
            [max, min, min],
            [max, min, max],
            [min, min, max],
            [0.0, -1.0, 0.0],
        ),
        CubeFace::Front => (
            [min, min, max],
            [max, min, max],
            [max, max, max],
            [min, max, max],
            [0.0, 0.0, 1.0],
        ),
        CubeFace::Back => (
            [max, min, min],
            [min, min, min],
            [min, max, min],
            [max, max, min],
            [0.0, 0.0, -1.0],
        ),
        CubeFace::Left => (
            [min, min, min],
            [min, min, max],
            [min, max, max],
            [min, max, min],
            [-1.0, 0.0, 0.0],
        ),
        CubeFace::Right => (
            [max, min, max],
            [max, min, min],
            [max, max, min],
            [max, max, max],
            [1.0, 0.0, 0.0],
        ),
    };

    let o1 = [v1[0] + offset.x, v1[1] + offset.y, v1[2] + offset.z];
    let o2 = [v2[0] + offset.x, v2[1] + offset.y, v2[2] + offset.z];
    let o3 = [v3[0] + offset.x, v3[1] + offset.y, v3[2] + offset.z];
    let o4 = [v4[0] + offset.x, v4[1] + offset.y, v4[2] + offset.z];

    add_block_quad(vertices, o1, o2, o3, o4, color, normal);
}

fn append_exposed_faces_shadow(vertices: &mut Vec<f32>, offset: Vec3, mask: u8) {
    for face in &CubeFace::ALL {
        if (mask & face.mask()) != 0 {
            add_shadow_quad_offset(vertices, offset, *face);
        }
    }
}

fn add_shadow_quad_offset(vertices: &mut Vec<f32>, offset: Vec3, face: CubeFace) {
    let min = 0.0;
    let max = 1.0;

    let (v1, v2, v3, v4) = match face {
        CubeFace::Top => (
            [min, max, max],
            [max, max, max],
            [max, max, min],
            [min, max, min],
        ),
        CubeFace::Bottom => (
            [min, min, min],
            [max, min, min],
            [max, min, max],
            [min, min, max],
        ),
        CubeFace::Front => (
            [min, min, max],
            [max, min, max],
            [max, max, max],
            [min, max, max],
        ),
        CubeFace::Back => (
            [max, min, min],
            [min, min, min],
            [min, max, min],
            [max, max, min],
        ),
        CubeFace::Left => (
            [min, min, min],
            [min, min, max],
            [min, max, max],
            [min, max, min],
        ),
        CubeFace::Right => (
            [max, min, max],
            [max, min, min],
            [max, max, min],
            [max, max, max],
        ),
    };

    let o1 = [v1[0] + offset.x, v1[1] + offset.y, v1[2] + offset.z];
    let o2 = [v2[0] + offset.x, v2[1] + offset.y, v2[2] + offset.z];
    let o3 = [v3[0] + offset.x, v3[1] + offset.y, v3[2] + offset.z];
    let o4 = [v4[0] + offset.x, v4[1] + offset.y, v4[2] + offset.z];

    // Triangle 1
    vertices.extend_from_slice(&o1);
    vertices.extend_from_slice(&o2);
    vertices.extend_from_slice(&o3);

    // Triangle 2
    vertices.extend_from_slice(&o1);
    vertices.extend_from_slice(&o3);
    vertices.extend_from_slice(&o4);
}

pub fn upload_position_only_vertices(vertices: &[f32]) -> (u32, u32, i32) {
    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * std::mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        let stride = (3 * std::mem::size_of::<f32>()) as i32;

        // Position at location 0.
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
        gl::EnableVertexAttribArray(0);

        gl::BindVertexArray(0);
    }

    (vao, vbo, (vertices.len() / 3) as i32)
}

#[derive(Clone, Copy, Debug)]
pub struct FrustumPlane {
    pub normal: Vec3,
    pub d: f32,
}

impl FrustumPlane {
    pub fn new(a: f32, b: f32, c: f32, d: f32) -> Self {
        let length = Vec3::new(a, b, c).length();
        if length > 0.00001 {
            Self {
                normal: Vec3::new(a / length, b / length, c / length),
                d: d / length,
            }
        } else {
            Self {
                normal: Vec3::ZERO,
                d: 0.0,
            }
        }
    }

    pub fn distance_to_point(&self, point: Vec3) -> f32 {
        self.normal.dot(point) + self.d
    }
}

pub struct CameraFrustum {
    pub planes: [FrustumPlane; 6],
}

impl CameraFrustum {
    pub fn from_view_projection(vp: Mat4) -> Self {
        let m = vp.to_cols_array_2d();

        let row0 = [m[0][0], m[1][0], m[2][0], m[3][0]];
        let row1 = [m[0][1], m[1][1], m[2][1], m[3][1]];
        let row2 = [m[0][2], m[1][2], m[2][2], m[3][2]];
        let row3 = [m[0][3], m[1][3], m[2][3], m[3][3]];

        let left = FrustumPlane::new(
            row3[0] + row0[0],
            row3[1] + row0[1],
            row3[2] + row0[2],
            row3[3] + row0[3],
        );
        let right = FrustumPlane::new(
            row3[0] - row0[0],
            row3[1] - row0[1],
            row3[2] - row0[2],
            row3[3] - row0[3],
        );
        let bottom = FrustumPlane::new(
            row3[0] + row1[0],
            row3[1] + row1[1],
            row3[2] + row1[2],
            row3[3] + row1[3],
        );
        let top = FrustumPlane::new(
            row3[0] - row1[0],
            row3[1] - row1[1],
            row3[2] - row1[2],
            row3[3] - row1[3],
        );
        let near = FrustumPlane::new(
            row3[0] + row2[0],
            row3[1] + row2[1],
            row3[2] + row2[2],
            row3[3] + row2[3],
        );
        let far = FrustumPlane::new(
            row3[0] - row2[0],
            row3[1] - row2[1],
            row3[2] - row2[2],
            row3[3] - row2[3],
        );

        Self {
            planes: [left, right, bottom, top, near, far],
        }
    }

    pub fn intersects_aabb(&self, min: Vec3, max: Vec3) -> bool {
        for plane in &self.planes {
            let p = Vec3::new(
                if plane.normal.x >= 0.0 { max.x } else { min.x },
                if plane.normal.y >= 0.0 { max.y } else { min.y },
                if plane.normal.z >= 0.0 { max.z } else { min.z },
            );

            if plane.distance_to_point(p) < 0.0 {
                return false;
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EditorMode;
    use crate::world::{CellType, World, WorldCoord};

    #[test]
    fn test_chunk_coord_mapping_positive() {
        let c0 = WorldCoord::new(0, 0, 0);
        let chunk0 = ChunkCoord::from_world_coord(c0);
        assert_eq!(chunk0, ChunkCoord::new(0, 0, 0));
        assert_eq!(ChunkCoord::local_offset(c0), (0, 0, 0));

        let c15 = WorldCoord::new(15, 15, 15);
        let chunk15 = ChunkCoord::from_world_coord(c15);
        assert_eq!(chunk15, ChunkCoord::new(0, 0, 0));
        assert_eq!(ChunkCoord::local_offset(c15), (15, 15, 15));

        let c16 = WorldCoord::new(16, 0, 0);
        let chunk16 = ChunkCoord::from_world_coord(c16);
        assert_eq!(chunk16, ChunkCoord::new(1, 0, 0));
        assert_eq!(ChunkCoord::local_offset(c16), (0, 0, 0));
    }

    #[test]
    fn test_chunk_coord_mapping_negative() {
        let c_neg1 = WorldCoord::new(-1, 0, 0);
        let chunk_neg1 = ChunkCoord::from_world_coord(c_neg1);
        assert_eq!(chunk_neg1, ChunkCoord::new(-1, 0, 0));
        assert_eq!(ChunkCoord::local_offset(c_neg1), (15, 0, 0));

        let c_neg16 = WorldCoord::new(-16, 0, 0);
        let chunk_neg16 = ChunkCoord::from_world_coord(c_neg16);
        assert_eq!(chunk_neg16, ChunkCoord::new(-1, 0, 0));
        assert_eq!(ChunkCoord::local_offset(c_neg16), (0, 0, 0));

        let c_neg17 = WorldCoord::new(-17, 0, 0);
        let chunk_neg17 = ChunkCoord::from_world_coord(c_neg17);
        assert_eq!(chunk_neg17, ChunkCoord::new(-2, 0, 0));
        assert_eq!(ChunkCoord::local_offset(c_neg17), (15, 0, 0));
    }

    #[test]
    fn test_chunk_boundary_occlusion() {
        let mut world = World::new();
        let c1 = WorldCoord::new(15, 0, 0); // Chunk (0,0,0)
        let c2 = WorldCoord::new(16, 0, 0); // Chunk (1,0,0)
        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::Block);

        let mask1 =
            crate::renderer::mesh::compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        let mask2 =
            crate::renderer::mesh::compute_exposed_faces_main(&world, c2, EditorMode::Editor);

        // Right face (+X) of c1 must be occluded
        assert_eq!(mask1 & CubeFace::Right.mask(), 0);
        // Left face (-X) of c2 must be occluded
        assert_eq!(mask2 & CubeFace::Left.mask(), 0);
    }

    #[test]
    fn test_texture_batching_inside_chunk() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(1, 0, 0);

        let id1 = world.set_cell(c1, CellType::Block);
        let id2 = world.set_cell(c2, CellType::Block);

        world.get_mut(c1).unwrap().texture = "Block_tx".to_string();
        world.get_mut(c2).unwrap().texture = "brick".to_string();

        let chunk_coord = ChunkCoord::new(0, 0, 0);
        let cpu_data = build_cpu_chunk_data(&world, chunk_coord, &[c1, c2], EditorMode::Editor);

        assert_eq!(cpu_data.main_texture_vertices.len(), 2);
        assert!(cpu_data.main_texture_vertices.contains_key("Block_tx"));
        assert!(cpu_data.main_texture_vertices.contains_key("brick"));
    }

    #[test]
    fn test_shadow_mesh_eligibility() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(1, 0, 0);
        let c3 = WorldCoord::new(2, 0, 0);

        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::SpawnPoint);
        world.set_cell(c3, CellType::Light);
        world.set_cell_visible_runtime(c3, true);

        let chunk_coord = ChunkCoord::new(0, 0, 0);
        let cpu_data = build_cpu_chunk_data(&world, chunk_coord, &[c1, c2, c3], EditorMode::Editor);

        // c1 and c2 cast shadows (Block, SpawnPoint). c3 (Light) does NOT.
        // c1 and c2 are adjacent along X axis, so their shared face is culled.
        // Each has 5 exposed faces -> 10 faces * 6 vertices * 3 floats = 180 floats.
        assert_eq!(cpu_data.shadow_positions.len(), 180);
    }

    #[test]
    fn test_frustum_culling_aabb() {
        let proj =
            glam::camera::rh::proj::opengl::perspective(60.0_f32.to_radians(), 1.0, 0.1, 100.0);
        let view = Mat4::look_at_rh(Vec3::new(0.0, 0.0, 10.0), Vec3::new(0.0, 0.0, 0.0), Vec3::Y);
        let vp = proj * view;

        let frustum = CameraFrustum::from_view_projection(vp);

        let chunk_in_front = ChunkCoord::new(0, 0, 0);
        let (min1, max1) = chunk_in_front.aabb_min_max();
        assert!(frustum.intersects_aabb(min1, max1));

        let chunk_far_away = ChunkCoord::new(100, 100, 100);
        let (min2, max2) = chunk_far_away.aabb_min_max();
        assert!(!frustum.intersects_aabb(min2, max2));
    }
}
