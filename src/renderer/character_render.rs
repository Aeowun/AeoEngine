use super::mesh::VERTEX_3D_FLOATS;
use super::Renderer;

impl Renderer {
    pub fn get_block_mask_range(&self, mask: u8) -> (i32, i32) {
        self.block_mask_ranges[mask as usize]
    }

    pub fn draw_character_mesh_persistent(&self, vertices: &[f32]) {
        if vertices.is_empty() {
            return;
        }

        unsafe {
            gl::BindVertexArray(self.character_vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, self.character_vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STREAM_DRAW,
            );

            let count = (vertices.len() / VERTEX_3D_FLOATS) as i32;

            gl::DrawArrays(gl::TRIANGLES, 0, count);

            gl::BindVertexArray(0);
        }
    }
}
