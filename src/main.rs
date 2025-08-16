use std::{ffi::CString, mem, os::raw::c_void, ptr, sync::mpsc::Receiver};

use gl::{
    ARRAY_BUFFER, COMPILE_STATUS, FRAGMENT_SHADER, STATIC_DRAW, VERTEX_SHADER,
    types::{GLchar, GLfloat, GLint, GLsizei, GLsizeiptr},
};
use glfw::{Action, Context, Key, OpenGlProfileHint, WindowEvent, WindowHint, WindowMode};

const SCR_WIDTH: u32 = 800;
const SCR_HEIGHT: u32 = 600;

const VERTICES: [f32; 9] = [
    -0.5, -0.5, 0.0, // 1
    0.5, -0.5, 0.0, // 2
    0.0, 0.5, 0.0, // 3
];

const VERTEX: &str = include_str!("vertex.glsl");
const FRAGMENT: &str = include_str!("fragment.glsl");

fn main() {
    // Init GLFW
    let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
    // Use OpenGL 3.3
    glfw.window_hint(WindowHint::ContextVersionMajor(3));
    glfw.window_hint(WindowHint::ContextVersionMinor(3));
    // Use core profile mode
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));

    // Create window
    let (mut window, events) = glfw
        .create_window(SCR_WIDTH, SCR_HEIGHT, "GL Learning", WindowMode::Windowed)
        .expect("Failed to create window!");
    window.make_current();
    window.set_key_polling(true);
    window.set_framebuffer_size_polling(true);

    // Load OpenGL function pointers
    gl::load_with(|symbol| window.get_proc_address(symbol) as *const _);

    let (shader_program, vao) = unsafe {
        // Compile the vertex shader
        let vertex_shader = gl::CreateShader(VERTEX_SHADER);
        let c_str_vert = CString::new(VERTEX.as_bytes()).unwrap();
        gl::ShaderSource(vertex_shader, 1, &c_str_vert.as_ptr(), ptr::null());
        gl::CompileShader(vertex_shader);

        // Check for compilation errors
        let mut success = gl::FALSE as GLint;
        let mut info_log = Vec::with_capacity(512);
        info_log.set_len(512 - 1);
        gl::GetShaderiv(vertex_shader, COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                vertex_shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "Vertex shader compilation failed: {}",
                str::from_utf8(&info_log).unwrap()
            );
        }

        // Compile fragment shader
        let fragment_shader = gl::CreateShader(FRAGMENT_SHADER);
        let c_str_frag = CString::new(FRAGMENT.as_bytes()).unwrap();
        gl::ShaderSource(fragment_shader, 1, &c_str_frag.as_ptr(), ptr::null());
        gl::CompileShader(fragment_shader);

        // Check for compilation errors
        gl::GetShaderiv(fragment_shader, COMPILE_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetShaderInfoLog(
                fragment_shader,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "Fragment shader compilation failed: {}",
                str::from_utf8(&info_log).unwrap()
            );
        }

        // Link shaders together
        let shader_program = gl::CreateProgram();
        gl::AttachShader(shader_program, vertex_shader);
        gl::AttachShader(shader_program, fragment_shader);
        gl::LinkProgram(shader_program);

        // Check for linking errors
        gl::GetProgramiv(shader_program, gl::LINK_STATUS, &mut success);
        if success != gl::TRUE as GLint {
            gl::GetProgramInfoLog(
                shader_program,
                512,
                ptr::null_mut(),
                info_log.as_mut_ptr() as *mut GLchar,
            );
            println!(
                "Program linking failed: {}",
                str::from_utf8(&info_log).unwrap()
            )
        }

        // Shader objects aren't needed anymore, so we can delete them
        gl::DeleteShader(vertex_shader);
        gl::DeleteShader(fragment_shader);

        // Configure buffers and array
        let (mut vbo, mut vao) = (0, 0);

        // Generate array
        gl::GenVertexArrays(1, &mut vao);
        // Generate buffer
        gl::GenBuffers(1, &mut vbo);

        // Bind array first, so that buffers can be in the array
        gl::BindVertexArray(vao);

        gl::BindBuffer(ARRAY_BUFFER, vbo);
        gl::BufferData(
            ARRAY_BUFFER,
            (VERTICES.len() * mem::size_of::<GLfloat>()) as GLsizeiptr,
            &VERTICES[0] as *const f32 as *const c_void,
            STATIC_DRAW,
        );

        gl::VertexAttribPointer(
            0,
            3,
            gl::FLOAT,
            gl::FALSE,
            3 * mem::size_of::<GLfloat>() as GLsizei,
            ptr::null(),
        );
        gl::EnableVertexAttribArray(0);

        (shader_program, vao)
    };

    // Render Loop
    while !window.should_close() {
        // Events
        process_events(&mut window, &events);

        unsafe {
            gl::ClearColor(0.2, 0.2, 0.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Tell OpenGL to use the linked program
            gl::UseProgram(shader_program);
            gl::BindVertexArray(vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }

        // Swap buffers and poll IO events
        window.swap_buffers();
        glfw.poll_events();
    }
}

fn process_events(window: &mut glfw::Window, events: &Receiver<(f64, glfw::WindowEvent)>) {
    for (_, event) in glfw::flush_messages(events) {
        match event {
            WindowEvent::FramebufferSize(width, height) => unsafe {
                gl::Viewport(0, 0, width, height);
            },
            WindowEvent::Key(Key::Escape, _, Action::Press, _) => window.set_should_close(true),
            _ => {}
        }
    }
}
