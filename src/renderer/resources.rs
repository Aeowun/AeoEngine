use std::ffi::CString;
use std::path::Path;

use image::GenericImageView;

use crate::editor::GridPlane;

use super::mesh::{add_block_quad, add_line, upload_vertices_3d, VERTEX_3D_FLOATS};
use super::Renderer;

impl Renderer {
    pub fn get_texture(&self, identifier: &str) -> u32 {
        let mut textures = self.textures.borrow_mut();

        if let Some(&tex) = textures.get(identifier) {
            return tex;
        }

        if !identifier.is_empty() {
            let paths = [
                format!(".assets/textures/{identifier}"),
                format!(".assets/textures/{identifier}.png"),
                format!(".assets/skybox/{identifier}"),
                format!(".assets/skybox/{identifier}.png"),
            ];

            for path in paths {
                if Path::new(&path).exists() {
                    if let Some(tex) = load_texture_from_file(&path) {
                        textures.insert(identifier.to_string(), tex);
                        return tex;
                    }
                }
            }
        }

        self.fallback_tex
    }

    pub fn get_cubemap(&self, identifier: &str) -> u32 {
        let mut cubemaps = self.cubemaps.borrow_mut();

        if let Some(&tex) = cubemaps.get(identifier) {
            return tex;
        }

        if !identifier.is_empty() {
            let paths = [
                format!(".assets/skybox/{identifier}"),
                format!(".assets/skybox/{identifier}.png"),
            ];

            for path in paths {
                if Path::new(&path).exists() {
                    if let Some(tex) = load_cubemap_from_file(&path) {
                        cubemaps.insert(identifier.to_string(), tex);
                        return tex;
                    }
                }
            }
        }

        0
    }
}

pub(super) fn get_uniform_location(program: u32, name: &str) -> i32 {
    let name = CString::new(name).expect("uniform name must not contain NUL");

    unsafe { gl::GetUniformLocation(program, name.as_ptr()) }
}

pub(super) fn create_character_vao_vbo() -> (u32, u32) {
    unsafe {
        let mut vao = 0;
        let mut vbo = 0;

        gl::GenVertexArrays(1, &mut vao);
        gl::GenBuffers(1, &mut vbo);

        gl::BindVertexArray(vao);
        gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

        let stride = (VERTEX_3D_FLOATS * std::mem::size_of::<f32>()) as i32;

        // Position.
        gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, stride, std::ptr::null());
        gl::EnableVertexAttribArray(0);

        // Normal.
        gl::VertexAttribPointer(
            1,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (3 * std::mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(1);

        // Color.
        gl::VertexAttribPointer(
            2,
            3,
            gl::FLOAT,
            gl::FALSE,
            stride,
            (6 * std::mem::size_of::<f32>()) as *const _,
        );
        gl::EnableVertexAttribArray(2);

        gl::BindVertexArray(0);

        (vao, vbo)
    }
}

pub(super) fn create_grid_plane_vao(plane: GridPlane) -> (u32, u32, i32) {
    let vertices = crate::editor::grid::generate_grid_vertices(plane);
    upload_vertices_3d(&vertices)
}

pub(super) fn create_axes() -> (u32, u32, i32) {
    let length = 3.0;
    let mut vertices = Vec::new();

    add_line(
        &mut vertices,
        [0.0, 0.0, 0.0],
        [length, 0.0, 0.0],
        [1.0, 0.2, 0.2, 1.0],
    );

    add_line(
        &mut vertices,
        [0.0, 0.0, 0.0],
        [0.0, length, 0.0],
        [0.2, 1.0, 0.2, 1.0],
    );

    add_line(
        &mut vertices,
        [0.0, 0.0, 0.0],
        [0.0, 0.0, length],
        [0.2, 0.2, 1.0, 1.0],
    );

    upload_vertices_3d(&vertices)
}

pub(super) fn create_highlight_box() -> (u32, u32, i32) {
    let mut vertices = Vec::new();

    let color = [1.0, 0.8, 0.1, 0.9];

    let min = 0.0;
    let max = 1.0;

    add_line(&mut vertices, [min, min, min], [max, min, min], color);
    add_line(&mut vertices, [max, min, min], [max, min, max], color);
    add_line(&mut vertices, [max, min, max], [min, min, max], color);
    add_line(&mut vertices, [min, min, max], [min, min, min], color);

    add_line(&mut vertices, [min, max, min], [max, max, min], color);
    add_line(&mut vertices, [max, max, min], [max, max, max], color);
    add_line(&mut vertices, [max, max, max], [min, max, max], color);
    add_line(&mut vertices, [min, max, max], [min, max, min], color);

    add_line(&mut vertices, [min, min, min], [min, max, min], color);
    add_line(&mut vertices, [max, min, min], [max, max, min], color);
    add_line(&mut vertices, [max, min, max], [max, max, max], color);
    add_line(&mut vertices, [min, min, max], [min, max, max], color);

    upload_vertices_3d(&vertices)
}

pub(super) fn create_anchor_marker() -> (u32, u32, i32) {
    use std::f32::consts::PI;

    let mut vertices = Vec::new();

    let color = [0.0, 1.0, 1.0, 1.0];

    let radius = 0.15;
    let center = [0.0, 0.0, 0.0];

    let cross_size = 0.3;

    add_line(
        &mut vertices,
        [-cross_size, 0.0, 0.0],
        [cross_size, 0.0, 0.0],
        color,
    );

    add_line(
        &mut vertices,
        [0.0, -cross_size, 0.0],
        [0.0, cross_size, 0.0],
        color,
    );

    add_line(
        &mut vertices,
        [0.0, 0.0, -cross_size],
        [0.0, 0.0, cross_size],
        color,
    );

    let latitudes = 8;
    let longitudes = 16;

    for i in 0..=latitudes {
        let lat = PI * (i as f32 / latitudes as f32 - 0.5);

        let y = lat.sin() * radius + center[1];

        let r = lat.cos() * radius;

        for j in 0..longitudes {
            let lon1 = 2.0 * PI * (j as f32 / longitudes as f32);

            let lon2 = 2.0 * PI * ((j + 1) as f32 / longitudes as f32);

            add_line(
                &mut vertices,
                [lon1.cos() * r + center[0], y, lon1.sin() * r + center[2]],
                [lon2.cos() * r + center[0], y, lon2.sin() * r + center[2]],
                color,
            );
        }
    }

    for i in 0..longitudes {
        let lon = 2.0 * PI * (i as f32 / longitudes as f32);

        let cos_lon = lon.cos();

        let sin_lon = lon.sin();

        for j in 0..latitudes {
            let lat1 = PI * (j as f32 / latitudes as f32 - 0.5);

            let lat2 = PI * ((j + 1) as f32 / latitudes as f32 - 0.5);

            add_line(
                &mut vertices,
                [
                    cos_lon * lat1.cos() * radius + center[0],
                    lat1.sin() * radius + center[1],
                    sin_lon * lat1.cos() * radius + center[2],
                ],
                [
                    cos_lon * lat2.cos() * radius + center[0],
                    lat2.sin() * radius + center[1],
                    sin_lon * lat2.cos() * radius + center[2],
                ],
                color,
            );
        }
    }

    upload_vertices_3d(&vertices)
}

pub(super) fn create_billboard_vao() -> (u32, u32) {
    let mut vertices = Vec::new();

    let color = [1.0, 1.0, 1.0, 1.0];

    let normal = [0.0, 0.0, 1.0];

    add_block_quad(
        &mut vertices,
        [-0.5, -0.5, 0.0],
        [0.5, -0.5, 0.0],
        [0.5, 0.5, 0.0],
        [-0.5, 0.5, 0.0],
        color,
        normal,
    );

    let (vao, vbo, _) = super::mesh::upload_block_vertices_3d(&vertices);

    (vao, vbo)
}

pub(super) fn load_texture_from_file(path: &str) -> Option<u32> {
    match image::open(path) {
        Ok(image) => {
            let rgba = image.to_rgba8();

            let (width, height) = rgba.dimensions();

            let mut texture = 0;

            unsafe {
                gl::GenTextures(1, &mut texture);

                gl::BindTexture(gl::TEXTURE_2D, texture);

                gl::TexImage2D(
                    gl::TEXTURE_2D,
                    0,
                    gl::RGBA as i32,
                    width as i32,
                    height as i32,
                    0,
                    gl::RGBA,
                    gl::UNSIGNED_BYTE,
                    rgba.as_raw().as_ptr() as *const _,
                );

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);

                gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);

                gl::BindTexture(gl::TEXTURE_2D, 0);
            }

            Some(texture)
        }

        Err(error) => {
            eprintln!("Failed to load texture {path}: {error:?}");

            None
        }
    }
}

pub(super) fn load_cubemap_from_file(path: &str) -> Option<u32> {
    match image::open(path) {
        Ok(image) => {
            let (width, height) = image.dimensions();

            if width != height * 4 / 3 {
                eprintln!(
                    "Invalid cubemap dimensions for cross layout: {}x{}. Expected 4:3 ratio.",
                    width, height
                );

                return None;
            }

            let face_size = height / 3;

            let rgba_image = image.to_rgba8();

            let mut texture = 0;

            unsafe {
                gl::GenTextures(1, &mut texture);

                gl::BindTexture(gl::TEXTURE_CUBE_MAP, texture);
            }

            let faces = [
                (0, 1, gl::TEXTURE_CUBE_MAP_POSITIVE_X),
                (2, 1, gl::TEXTURE_CUBE_MAP_NEGATIVE_X),
                (1, 0, gl::TEXTURE_CUBE_MAP_POSITIVE_Y),
                (1, 2, gl::TEXTURE_CUBE_MAP_NEGATIVE_Y),
                (3, 1, gl::TEXTURE_CUBE_MAP_POSITIVE_Z),
                (1, 1, gl::TEXTURE_CUBE_MAP_NEGATIVE_Z),
            ];

            for (column, row, target) in faces {
                let x = column * face_size;

                let y = row * face_size;

                let face = rgba_image.view(x, y, face_size, face_size).to_image();

                unsafe {
                    gl::TexImage2D(
                        target,
                        0,
                        gl::RGBA as i32,
                        face_size as i32,
                        face_size as i32,
                        0,
                        gl::RGBA,
                        gl::UNSIGNED_BYTE,
                        face.as_raw().as_ptr() as *const _,
                    );
                }
            }

            unsafe {
                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_MIN_FILTER,
                    gl::LINEAR as i32,
                );

                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_MAG_FILTER,
                    gl::LINEAR as i32,
                );

                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_WRAP_S,
                    gl::CLAMP_TO_EDGE as i32,
                );

                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_WRAP_T,
                    gl::CLAMP_TO_EDGE as i32,
                );

                gl::TexParameteri(
                    gl::TEXTURE_CUBE_MAP,
                    gl::TEXTURE_WRAP_R,
                    gl::CLAMP_TO_EDGE as i32,
                );

                gl::BindTexture(gl::TEXTURE_CUBE_MAP, 0);
            }

            Some(texture)
        }

        Err(error) => {
            eprintln!("Failed to load cubemap {path}: {error:?}");

            None
        }
    }
}
