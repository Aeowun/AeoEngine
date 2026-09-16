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

        let stride = (7 * mem::size_of::<f32>()) as i32;
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, ptr::null());
        gl::EnableVertexAttribArray(0);
        gl::VertexAttribPointer(1, 4, gl::FLOAT, gl::FALSE, stride, (3 * mem::size_of::<f32>()) as *const _);
        gl::EnableVertexAttribArray(1);
        gl::BindVertexArray(0);
    }
    (vao, vbo, (vertices.len() / 7) as i32)
}

pub fn add_line(
    vertices: &mut Vec<f32>,
    a: [f32; 3],
    b: [f32; 3],
    color: [f32; 4],
) {
    vertices.extend_from_slice(&[
        a[0], a[1], a[2],
        color[0], color[1],
        color[2], color[3],

        b[0], b[1], b[2],
        color[0], color[1],
        color[2], color[3],
    ]);
}

pub fn add_quad(
    vertices: &mut Vec<f32>,
    v1: [f32; 3],
    v2: [f32; 3],
    v3: [f32; 3],
    v4: [f32; 3],
    color: [f32; 4],
) {
    add_vertex(vertices, v1, color);
    add_vertex(vertices, v2, color);
    add_vertex(vertices, v3, color);

    add_vertex(vertices, v1, color);
    add_vertex(vertices, v3, color);
    add_vertex(vertices, v4, color);
}

fn add_vertex(
    vertices: &mut Vec<f32>,
    pos: [f32; 3],
    color: [f32; 4],
) {
    vertices.extend_from_slice(&[
        pos[0], pos[1], pos[2],
        color[0], color[1],
        color[2], color[3],
    ]);
}
