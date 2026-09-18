use std::mem;
use std::ptr;

pub fn upload_vertices_2d(
    vertices: &[f32]
) -> (u32, u32) {
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
        gl::VertexAttribPointer(0, 2, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (2 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);
        gl::BindVertexArray(0);
    }
    (vao, vbo)
}

pub fn upload_vertices_3d(
    vertices: &[f32]
) -> (u32, u32, i32) {
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

        // Stride is 10: pos(3), normal(3), color(4)
        let stride = (10 * mem::size_of::<f32>()) as i32;
        // Position
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        // Normal
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);
        // Color
        gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(2);

        gl::BindVertexArray(0);
    }
    (vao, vbo, (vertices.len() / 10) as i32)
}

pub fn add_line(
    vertices: &mut Vec<f32>,
    a: [f32; 3],
    b: [f32; 3],
    color: [f32; 4],
) {
    let normal = [0.0, 1.0, 0.0]; // Dummy normal for lines
    vertices.extend_from_slice(&[
        a[0], a[1], a[2],
        normal[0], normal[1], normal[2],
        color[0], color[1], color[2], color[3],

        b[0], b[1], b[2],
        normal[0], normal[1], normal[2],
        color[0], color[1], color[2], color[3],
    ]);
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

fn add_vertex(
    vertices: &mut Vec<f32>,
    pos: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
) {
    vertices.extend_from_slice(&[
        pos[0], pos[1], pos[2],
        normal[0], normal[1], normal[2],
        color[0], color[1], color[2], color[3],
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
    pos: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
    uv: [f32; 2],
) {
    vertices.extend_from_slice(&[
        pos[0], pos[1], pos[2],
        normal[0], normal[1], normal[2],
        color[0], color[1], color[2], color[3],
        uv[0], uv[1],
    ]);
}

pub fn upload_block_vertices_3d(
    vertices: &[f32]
) -> (u32, u32, i32) {
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

        // Stride is 12: pos(3), normal(3), color(4), uv(2)
        let stride = (12 * mem::size_of::<f32>()) as i32;
        // Position
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        // Normal
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);
        // Color
        gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(2);
        // UV
        gl::VertexAttribPointer(3, 2, gl::FLOAT, gl::FALSE, stride, (10 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(3);

        gl::BindVertexArray(0);
    }
    (vao, vbo, (vertices.len() / 12) as i32)
}

/// Dynamic mesh upload and draw for character pose updates.
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

        let stride = (10 * mem::size_of::<f32>()) as i32;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 3, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);
        gl::VertexAttribPointer(2, 4, gl::FLOAT, gl::FALSE, stride, (6 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(2);

        gl::DrawArrays(gl::TRIANGLES, 0, (vertices.len() / 10) as i32);

        gl::BindVertexArray(0);
        gl::DeleteBuffers(1, &vbo);
        gl::DeleteVertexArrays(1, &vao);
    }
}
