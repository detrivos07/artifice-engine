extern crate artifice_engine;
extern crate gl;
extern crate glfw;
extern crate logging;

use std::ffi::CString;

use artifice_engine::event::{Event, EventType, KeyAction, KeyCode, KeyEvent};
use artifice_engine::{run_application, Application};
use logging::Level;

pub struct TestApplication {
    vertex_array: u32,
    vertex_buffer: u32,
    shader_program: u32,
    rotation: f32,
}

impl Application for TestApplication {
    fn new() -> Self {
        TestApplication {
            vertex_array: 0,
            vertex_buffer: 0,
            shader_program: 0,
            rotation: 0.0,
        }
    }

    fn init(&mut self) {
        logging::set_log_level(Level::DEBUG);
        logging::info("TestApplication initialized!");

        // Define vertex data for a triangle
        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0, // top
            -0.5, -0.5, 0.0, // bottom left
            0.5, -0.5, 0.0, // bottom right
        ];

        // Set up OpenGL objects
        unsafe {
            // Create a vertex array object
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Create a vertex buffer object
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            // Configure vertex attributes
            gl::VertexAttribPointer(
                0,
                3,
                gl::FLOAT,
                gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as i32,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

            // Create and compile the vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let vertex_shader_source = CString::new(
                "#version 330 core
                layout (location = 0) in vec3 aPos;
                uniform float rotation;

                void main() {
                    float angle = rotation;
                    float x = aPos.x * cos(angle) - aPos.y * sin(angle);
                    float y = aPos.x * sin(angle) + aPos.y * cos(angle);
                    gl_Position = vec4(x, y, aPos.z, 1.0);
                }",
            )
            .unwrap();
            gl::ShaderSource(
                vertex_shader,
                1,
                &vertex_shader_source.as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(vertex_shader);
            check_shader_compilation(vertex_shader);

            // Create and compile the fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_shader_source = CString::new(
                "#version 330 core
                out vec4 FragColor;

                void main() {
                    FragColor = vec4(1.0, 0.5, 0.2, 1.0);
                }",
            )
            .unwrap();
            gl::ShaderSource(
                fragment_shader,
                1,
                &fragment_shader_source.as_ptr(),
                std::ptr::null(),
            );
            gl::CompileShader(fragment_shader);
            check_shader_compilation(fragment_shader);

            // Create and link the shader program
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            check_program_linking(shader_program);

            // Delete the shaders as they're linked into the program and no longer needed
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            self.vertex_array = vao;
            self.vertex_buffer = vbo;
            self.shader_program = shader_program;
        }

        logging::info("OpenGL initialized successfully");
    }

    fn update(&mut self, delta_time: f32) {
        // Update rotation
        self.rotation += delta_time * 0.5;

        // Keep rotation within 0-2π
        if self.rotation > std::f32::consts::TAU {
            self.rotation -= std::f32::consts::TAU;
        }
    }

    fn render(&mut self) {
        // Render
        unsafe {
            gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw the triangle
            gl::UseProgram(self.shader_program);

            // Set rotation uniform
            let rotation_name = CString::new("rotation").unwrap();
            let rotation_loc = gl::GetUniformLocation(self.shader_program, rotation_name.as_ptr());
            gl::Uniform1f(rotation_loc, self.rotation);

            gl::BindVertexArray(self.vertex_array);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    fn shutdown(&mut self) {
        // Clean up
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteProgram(self.shader_program);
        }
        logging::info("TestApplication shutdown complete!");
    }

    fn event(&mut self, event: &mut Event) {
        // Check if the event is a keyboard event
        if event.event_type == EventType::Keyboard {
            // Try to get the keyboard event data
            if let Some(key_event) = event.get_data::<KeyEvent>() {
                if key_event.key == KeyCode::R && key_event.action == KeyAction::Press {
                    // Reset rotation on R key press
                    self.rotation = 0.0;
                    logging::info("Rotation reset!");
                    event.mark_handled();
                } else if key_event.key == KeyCode::Escape && key_event.action == KeyAction::Press {
                    // Log escape key press
                    logging::info("Escape key pressed - closing application");
                }
            }
        } else if event.event_type == EventType::Window {
            // Log window events at debug level
            logging::debug(&format!("Window event: {:?}", event.data));
        }
    }

    fn get_name(&self) -> &str {
        "Artifice Engine Demo"
    }
}

fn main() {
    logging::init();
    logging::info("Program has started!");

    // Run the application
    run_application::<TestApplication>();

    logging::info("Program has finished");
}

unsafe fn check_shader_compilation(shader: u32) {
    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut log_length = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);
        let mut log = Vec::with_capacity(log_length as usize);
        log.set_len(log_length as usize);
        gl::GetShaderInfoLog(
            shader,
            log_length,
            std::ptr::null_mut(),
            log.as_mut_ptr() as *mut i8,
        );
        let log_str = std::str::from_utf8(&log).unwrap_or("Unknown error");
        logging::error(&format!("Shader compilation failed: {}", log_str));
    }
}

unsafe fn check_program_linking(program: u32) {
    let mut success = 0;
    gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
    if success == 0 {
        let mut log_length = 0;
        gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);
        let mut log = Vec::with_capacity(log_length as usize);
        log.set_len(log_length as usize);
        gl::GetProgramInfoLog(
            program,
            log_length,
            std::ptr::null_mut(),
            log.as_mut_ptr() as *mut i8,
        );
        let log_str = std::str::from_utf8(&log).unwrap_or("Unknown error");
        logging::error(&format!("Program linking failed: {}", log_str));
    }
}
