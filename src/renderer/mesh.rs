use std::mem;
use std::ptr;

const VERTEX_3D_FLOATS: usize = 10;
const BLOCK_VERTEX_FLOATS: usize = 12;

pub fn upload_vertices_2d(vertices: &[f32]) -> (u32, u32) {
    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        let stride = (5 * mem::size_of::<f32>()) as i32;

        // Position.
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        // Color.
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (2 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(1);

        gl::BindVertexArray(0);
    }

    (vao, vbo)
}

pub fn upload_vertices_3d(vertices: &[f32]) -> (u32, u32, i32) {
    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        let stride = (VERTEX_3D_FLOATS * mem::size_of::<f32>()) as i32;

        // Position.
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        // Normal.
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(1);

        // Color.
        gl::VertexAttribPointer(
            2,
            4,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(2);

        gl::BindVertexArray(0);
    }

    (vao, vbo, (vertices.len() / VERTEX_3D_FLOATS) as i32)
}

pub fn add_line(vertices: &mut Vec<f32>, a: [f32; 3], b: [f32; 3], color: [f32; 4]) {
    // Lines do not use lighting, so the normal is only present to match the
    // shared 3D vertex format.
    let normal = [0.0, 1.0, 0.0];

    add_vertex(vertices, a, normal, color);
    add_vertex(vertices, b, normal, color);
}

pub fn add_quad(
    vertices: &mut Vec<f32>,
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
    v4: [f32; 3],
    color: [f32; 4],
    normal: [f32; 3],
) {
    add_vertex(vertices, v1, normal, color);
    add_vertex(vertices, v2, normal, color);
    add_vertex(vertices, v3, normal, color);

    add_vertex(vertices, v1, normal, color);
    add_vertex(vertices, v3, normal, color);
    add_vertex(vertices, v4, normal, color);
}

fn add_vertex(vertices: &mut Vec<f32>, position: [f32; 3], normal: [f32; 3], color: [f32; 4]) {
    vertices.extend_from_slice(&[
        position[0],
        position[1],
        position[2],
        normal[0],
        normal[1],
        normal[2],
        color[0],
        color[1],
        color[2],
        color[3],
    ]);
}

pub fn add_block_quad(
    vertices: &mut Vec<f32>,
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
    v4: [f32; 3],
    color: [f32; 4],
    normal: [f32; 3],
) {
    add_block_vertex(vertices, v1, normal, color, [0.0, 1.0]);
    add_block_vertex(vertices, v2, normal, color, [1.0, 1.0]);
    add_block_vertex(vertices, v3, normal, color, [1.0, 0.0]);

    add_block_vertex(vertices, v1, normal, color, [0.0, 1.0]);
    add_block_vertex(vertices, v3, normal, color, [1.0, 0.0]);
    add_block_vertex(vertices, v4, normal, color, [0.0, 0.0]);
}

fn add_block_vertex(
    vertices: &mut Vec<f32>,
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
    uv: [f32; 2],
) {
    vertices.extend_from_slice(&[
        position[0],
        position[1],
        position[2],
        normal[0],
        normal[1],
        normal[2],
        color[0],
        color[1],
        color[2],
        color[3],
        uv[0],
        uv[1],
    ]);
}

pub fn upload_block_vertices_3d(vertices: &[f32]) -> (u32, u32, i32) {
    let mut vao = 0;
    let mut vbo = 0;

    unsafe {
        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STATIC_DRAW,
        );

        let stride = (BLOCK_VERTEX_FLOATS * mem::size_of::<f32>()) as i32;

        // Position.
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        // Normal.
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(1);

        // Color.
        gl::VertexAttribPointer(
            2,
            4,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(2);

        // UV.
        gl::VertexAttribPointer(
            3,
            2,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (10 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(3);

        gl::BindVertexArray(0);
    }

    (vao, vbo, (vertices.len() / BLOCK_VERTEX_FLOATS) as i32)
}

/// Uploads and draws a dynamic mesh used by runtime character rendering.
///
/// The mesh is intentionally short-lived: the VAO and VBO are created for
/// the draw and released immediately afterward.
pub fn upload_and_draw_mesh_3d(vertices: &[f32]) {
    unsafe {
        let mut vao = 0;
        let mut vbo = 0;

        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        gl::BufferData(
            gl::ARRAY_BUFFER,
            (vertices.len() * mem::size_of::<f32>()) as isize,
            vertices.as_ptr() as *const _,
            gl::STREAM_DRAW,
        );

        let stride = (VERTEX_3D_FLOATS * mem::size_of::<f32>()) as i32;

        // Position.
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);

        // Normal.
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(1);

        // Color.
        gl::VertexAttribPointer(
            2,
            4,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(2);

        gl::DrawArrays(gl::TRIANGLES, 0, (vertices.len() / VERTEX_3D_FLOATS) as i32);

        gl::BindVertexArray(0);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CubeFace {
    Top,    // +Y (bit 0)
    Bottom, // -Y (bit 1)
    Front,  // +Z (bit 2)
    Back,   // -Z (bit 3)
    Left,   // -X (bit 4)
    Right,  // +X (bit 5)
}

impl CubeFace {
    pub const ALL: [CubeFace; 6] = [
        CubeFace::Top,
        CubeFace::Bottom,
        CubeFace::Front,
        CubeFace::Back,
        CubeFace::Left,
        CubeFace::Right,
    ];

    pub fn mask(&self) -> u8 {
        match self {
            CubeFace::Top => 1 << 0,
            CubeFace::Bottom => 1 << 1,
            CubeFace::Front => 1 << 2,
            CubeFace::Back => 1 << 3,
            CubeFace::Left => 1 << 4,
            CubeFace::Right => 1 << 5,
        }
    }

    pub fn neighbor_coord(&self, coord: crate::world::WorldCoord) -> crate::world::WorldCoord {
        match self {
            CubeFace::Top => crate::world::WorldCoord::new(coord.x, coord.y + 1, coord.z),
            CubeFace::Bottom => crate::world::WorldCoord::new(coord.x, coord.y - 1, coord.z),
            CubeFace::Front => crate::world::WorldCoord::new(coord.x, coord.y, coord.z + 1),
            CubeFace::Back => crate::world::WorldCoord::new(coord.x, coord.y, coord.z - 1),
            CubeFace::Left => crate::world::WorldCoord::new(coord.x - 1, coord.y, coord.z),
            CubeFace::Right => crate::world::WorldCoord::new(coord.x + 1, coord.y, coord.z),
        }
    }

    pub fn add_face_quad(&self, vertices: &mut Vec<f32>) {
        let color = [1.0, 1.0, 1.0, 1.0];
        let min = 0.0;
        let max = 1.0;

        match self {
            CubeFace::Top => add_block_quad(
                vertices,
                [min, max, max],
                [max, max, max],
                [max, max, min],
                [min, max, min],
                color,
                [0.0, 1.0, 0.0],
            ),
            CubeFace::Bottom => add_block_quad(
                vertices,
                [min, min, min],
                [max, min, min],
                [max, min, max],
                [min, min, max],
                color,
                [0.0, -1.0, 0.0],
            ),
            CubeFace::Front => add_block_quad(
                vertices,
                [min, min, max],
                [max, min, max],
                [max, max, max],
                [min, max, max],
                color,
                [0.0, 0.0, 1.0],
            ),
            CubeFace::Back => add_block_quad(
                vertices,
                [max, min, min],
                [min, min, min],
                [min, max, min],
                [max, max, min],
                color,
                [0.0, 0.0, -1.0],
            ),
            CubeFace::Left => add_block_quad(
                vertices,
                [min, min, min],
                [min, min, max],
                [min, max, max],
                [min, max, min],
                color,
                [-1.0, 0.0, 0.0],
            ),
            CubeFace::Right => add_block_quad(
                vertices,
                [max, min, max],
                [max, min, min],
                [max, max, min],
                [max, max, max],
                color,
                [1.0, 0.0, 0.0],
            ),
        }
    }
}

pub fn compute_exposed_faces_main(
    world: &crate::world::World,
    coord: crate::world::WorldCoord,
    mode: crate::engine::EditorMode,
) -> u8 {
    let mut mask = 0u8;

    for face in &CubeFace::ALL {
        let neighbor_coord = face.neighbor_coord(coord);
        let occluded = if let Some(neighbor_cell) = world.get_effective_cell(neighbor_coord) {
            let renderable = matches!(
                neighbor_cell.cell_type,
                crate::world::CellType::Block
                    | crate::world::CellType::SpawnPoint
                    | crate::world::CellType::Light
            );
            if renderable && world.is_cell_visible(neighbor_coord) {
                match mode {
                    crate::engine::EditorMode::Editor => true,
                    crate::engine::EditorMode::Play => world.is_cell_anchored(neighbor_coord),
                }
            } else {
                false
            }
        } else {
            false
        };

        if !occluded {
            mask |= face.mask();
        }
    }

    mask
}

pub fn compute_exposed_faces_shadow(
    world: &crate::world::World,
    coord: crate::world::WorldCoord,
    mode: crate::engine::EditorMode,
) -> u8 {
    let mut mask = 0u8;

    for face in &CubeFace::ALL {
        let neighbor_coord = face.neighbor_coord(coord);
        let occluded = if let Some(neighbor_cell) = world.get_effective_cell(neighbor_coord) {
            let renderable = matches!(
                neighbor_cell.cell_type,
                crate::world::CellType::Block | crate::world::CellType::SpawnPoint
            );
            if renderable
                && world.is_cell_visible(neighbor_coord)
                && world.is_cell_solid(neighbor_coord)
            {
                match mode {
                    crate::engine::EditorMode::Editor => true,
                    crate::engine::EditorMode::Play => world.is_cell_anchored(neighbor_coord),
                }
            } else {
                false
            }
        } else {
            false
        };

        if !occluded {
            mask |= face.mask();
        }
    }

    mask
}

pub fn create_block_masks() -> (u32, u32, [(i32, i32); 64]) {
    let mut all_vertices = Vec::new();
    let mut mask_ranges = [(0i32, 0i32); 64];

    for mask in 0u8..64 {
        let first_vertex = (all_vertices.len() / BLOCK_VERTEX_FLOATS) as i32;
        let mut count = 0i32;

        for face in &CubeFace::ALL {
            if (mask & face.mask()) != 0 {
                face.add_face_quad(&mut all_vertices);
                count += 6;
            }
        }

        mask_ranges[mask as usize] = (first_vertex, count);
    }

    let (vao, vbo, _) = upload_block_vertices_3d(&all_vertices);

    (vao, vbo, mask_ranges)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EditorMode;
    use crate::world::{CellType, World, WorldCoord};

    #[test]
    fn test_isolated_voxel_exposes_all_six_faces() {
        let mut world = World::new();
        let coord = WorldCoord::new(0, 0, 0);
        world.set_cell(coord, CellType::Block);

        let mask = compute_exposed_faces_main(&world, coord, EditorMode::Editor);
        assert_eq!(mask, 63);

        let mut vertices = Vec::new();
        for face in &CubeFace::ALL {
            if (mask & face.mask()) != 0 {
                face.add_face_quad(&mut vertices);
            }
        }
        let vertex_count = (vertices.len() / BLOCK_VERTEX_FLOATS) as i32;
        assert_eq!(vertex_count, 36);
    }

    #[test]
    fn test_voxel_with_one_neighbor_exposes_five_faces() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(0, 1, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::Block);

        let mask = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        assert_eq!(mask, 63 & !CubeFace::Top.mask());

        let mut vertices = Vec::new();
        for face in &CubeFace::ALL {
            if (mask & face.mask()) != 0 {
                face.add_face_quad(&mut vertices);
            }
        }
        let vertex_count = (vertices.len() / BLOCK_VERTEX_FLOATS) as i32;
        assert_eq!(vertex_count, 30);
    }

    #[test]
    fn test_two_neighboring_occluders_hide_two_faces() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(WorldCoord::new(0, 1, 0), CellType::Block);
        world.set_cell(WorldCoord::new(1, 0, 0), CellType::Block);

        let mask = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        let expected = 63 & !CubeFace::Top.mask() & !CubeFace::Right.mask();
        assert_eq!(mask, expected);

        let mut vertices = Vec::new();
        for face in &CubeFace::ALL {
            if (mask & face.mask()) != 0 {
                face.add_face_quad(&mut vertices);
            }
        }
        let vertex_count = (vertices.len() / BLOCK_VERTEX_FLOATS) as i32;
        assert_eq!(vertex_count, 24);
    }

    #[test]
    fn test_opposite_neighbors_hide_correct_two_faces() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(WorldCoord::new(0, 1, 0), CellType::Block);
        world.set_cell(WorldCoord::new(0, -1, 0), CellType::Block);

        let mask = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        let expected = 63 & !CubeFace::Top.mask() & !CubeFace::Bottom.mask();
        assert_eq!(mask, expected);
    }

    #[test]
    fn test_six_occluding_neighbors_produce_zero_face_mask() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(WorldCoord::new(0, 1, 0), CellType::Block);
        world.set_cell(WorldCoord::new(0, -1, 0), CellType::Block);
        world.set_cell(WorldCoord::new(0, 0, 1), CellType::Block);
        world.set_cell(WorldCoord::new(0, 0, -1), CellType::Block);
        world.set_cell(WorldCoord::new(-1, 0, 0), CellType::Block);
        world.set_cell(WorldCoord::new(1, 0, 0), CellType::Block);

        let mask = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        assert_eq!(mask, 0);

        let mut vertices = Vec::new();
        for face in &CubeFace::ALL {
            if (mask & face.mask()) != 0 {
                face.add_face_quad(&mut vertices);
            }
        }
        assert_eq!(vertices.len(), 0);
    }

    #[test]
    fn test_missing_neighbor_is_treated_as_exposed() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        world.set_cell(c1, CellType::Block);

        for face in &CubeFace::ALL {
            assert!(world.get_effective_cell(face.neighbor_coord(c1)).is_none());
        }
        let mask = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        assert_eq!(mask, 63);
    }

    #[test]
    fn test_neighbor_lookup_uses_integer_world_coord() {
        let c1 = WorldCoord::new(10, -5, 20);
        assert_eq!(
            CubeFace::Top.neighbor_coord(c1),
            WorldCoord::new(10, -4, 20)
        );
        assert_eq!(
            CubeFace::Bottom.neighbor_coord(c1),
            WorldCoord::new(10, -6, 20)
        );
        assert_eq!(
            CubeFace::Front.neighbor_coord(c1),
            WorldCoord::new(10, -5, 21)
        );
        assert_eq!(
            CubeFace::Back.neighbor_coord(c1),
            WorldCoord::new(10, -5, 19)
        );
        assert_eq!(
            CubeFace::Left.neighbor_coord(c1),
            WorldCoord::new(9, -5, 20)
        );
        assert_eq!(
            CubeFace::Right.neighbor_coord(c1),
            WorldCoord::new(11, -5, 20)
        );
    }

    #[test]
    fn test_runtime_created_neighboring_cells_participate() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        world.set_cell(c1, CellType::Block);

        let id = world.create_runtime_cell(CellType::Block);
        world
            .move_runtime_cell(id, WorldCoord::new(0, 1, 0))
            .unwrap();

        let mask = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        assert_eq!(mask, 63 & !CubeFace::Top.mask());
    }

    #[test]
    fn test_runtime_deleted_authored_cells_no_longer_occlude() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(0, 1, 0);
        world.set_cell(c1, CellType::Block);
        let id2 = world.set_cell(c2, CellType::Block);

        assert_eq!(
            compute_exposed_faces_main(&world, c1, EditorMode::Editor),
            63 & !CubeFace::Top.mask()
        );

        world.delete_cell_runtime(id2);

        assert_eq!(
            compute_exposed_faces_main(&world, c1, EditorMode::Editor),
            63
        );
    }

    #[test]
    fn test_runtime_visibility_overrides_affect_exposure() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(0, 1, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::Block);

        world.set_cell_visible_runtime(c2, false);

        let mask = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        assert_eq!(mask, 63);
    }

    #[test]
    fn test_main_pass_occlusion_does_not_acquire_solidity_requirement() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(0, 1, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::Block);

        world.set_cell_solid_runtime(c2, false);

        let mask_main = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        assert_eq!(mask_main, 63 & !CubeFace::Top.mask());
    }

    #[test]
    fn test_shadow_pass_eligibility_requires_shadow_conditions() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(0, 1, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::Block);

        let mask_shadow_1 = compute_exposed_faces_shadow(&world, c1, EditorMode::Editor);
        assert_eq!(mask_shadow_1, 63 & !CubeFace::Top.mask());

        world.set_cell_solid_runtime(c2, false);

        let mask_shadow_2 = compute_exposed_faces_shadow(&world, c1, EditorMode::Editor);
        assert_eq!(mask_shadow_2, 63);
    }

    #[test]
    fn test_fxblock_does_not_become_voxel_occluder() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(0, 1, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::FxBlock);

        let mask_main = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        assert_eq!(mask_main, 63);

        let mask_shadow = compute_exposed_faces_shadow(&world, c1, EditorMode::Editor);
        assert_eq!(mask_shadow, 63);
    }

    #[test]
    fn test_audio_emitter_does_not_become_voxel_occluder() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(0, 1, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::AudioEmitter);

        let mask_main = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        assert_eq!(mask_main, 63);

        let mask_shadow = compute_exposed_faces_shadow(&world, c1, EditorMode::Editor);
        assert_eq!(mask_shadow, 63);
    }

    #[test]
    fn test_light_does_not_become_part_of_shadow_voxel_path() {
        let mut world = World::new();
        let c1 = WorldCoord::new(0, 0, 0);
        let c2 = WorldCoord::new(0, 1, 0);
        world.set_cell(c1, CellType::Block);
        world.set_cell(c2, CellType::Light);
        world.set_cell_visible_runtime(c2, true);

        let mask_main = compute_exposed_faces_main(&world, c1, EditorMode::Editor);
        assert_eq!(mask_main, 63 & !CubeFace::Top.mask());

        let mask_shadow = compute_exposed_faces_shadow(&world, c1, EditorMode::Editor);
        assert_eq!(mask_shadow, 63);
    }

    #[test]
    fn test_face_winding_normals_and_uv_coordinates() {
        let mut vertices = Vec::new();

        CubeFace::Top.add_face_quad(&mut vertices);
        assert_eq!(vertices.len(), 6 * BLOCK_VERTEX_FLOATS);

        assert_eq!(vertices[3], 0.0);
        assert_eq!(vertices[4], 1.0);
        assert_eq!(vertices[5], 0.0);

        assert_eq!(&vertices[10..12], &[0.0, 1.0]);
        assert_eq!(&vertices[22..24], &[1.0, 1.0]);
        assert_eq!(&vertices[34..36], &[1.0, 0.0]);
    }

    #[test]
    fn test_geometry_counts_for_all_exposed_face_counts() {
        for num_faces in 0..=6 {
            let mask = (1u8 << num_faces) - 1;
            let mut vertices = Vec::new();
            for face in &CubeFace::ALL {
                if (mask & face.mask()) != 0 {
                    face.add_face_quad(&mut vertices);
                }
            }
            let count = (vertices.len() / BLOCK_VERTEX_FLOATS) as i32;
            assert_eq!(count, num_faces * 6);
        }
    }
}
