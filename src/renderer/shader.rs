use std::ffi::CString;
use std::ptr;

pub fn create_program(vertex_source: &str, fragment_source: &str) -> u32 {
    let vertex_shader = compile_shader(gl::VERTEX_SHADER, vertex_source);
    let fragment_shader = compile_shader(gl::FRAGMENT_SHADER, fragment_source);
    unsafe {
        let program = gl::CreateProgram();
        gl::AttachShader(program, vertex_shader);
        gl::AttachShader(program, fragment_shader);
        gl::LinkProgram(program);
        let mut success = 0;
        gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
        if success == 0 {
            panic!("Failed to link OpenGL shader program.");
        }
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);
        program
    }
}

fn compile_shader(kind: u32, source: &str) -> u32 {
    let source = CString::new(source).unwrap();
    unsafe {
        let shader = gl::CreateShader(kind);
        gl::ShaderSource(shader, 1, &source.as_ptr(), ptr::null());
        gl::CompileShader(shader);
        let mut success = 0;
        gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
        if success == 0 {
            panic!("OpenGL shader compilation failed.");
        }
        shader
    }
}
