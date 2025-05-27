extern crate artifice_engine;
extern crate gl;
extern crate glfw;
extern crate artifice_logging;

use std::ffi::CString;

use artifice_engine::events::{
    Event, EventType, KeyAction, KeyCode,
};
use artifice_engine::{Engine, Application};
use artifice_engine::window::{HotReloadConfig, HotReloadStatus};
use artifice_engine::io::MetricsConfig;
use artifice_logging::{error, info, warn};

pub struct BackendSwitchingDemo {
    vertex_array: u32,
    vertex_buffer: u32,
    shader_program: u32,
    rotation: f32,
    current_backend: String,
    switch_requested: Option<String>,
    background_color: (f32, f32, f32),
    triangle_color_location: i32,
    color_cycle_time: f32,
}

impl Application for BackendSwitchingDemo {
    fn new() -> Self {
        BackendSwitchingDemo {
            vertex_array: 0,
            vertex_buffer: 0,
            shader_program: 0,
            rotation: 0.0,
            current_backend: "glfw".to_string(),
            switch_requested: None,
            background_color: (0.2, 0.3, 0.3),
            triangle_color_location: -1,
            color_cycle_time: 0.0,
        }
    }

    fn init(&mut self) {
        info!("BackendSwitchingDemo initialized with backend: {}", self.current_backend);

        // Define vertex data for a triangle
        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0,   // top
            -0.5, -0.5, 0.0, // bottom left
            0.5, -0.5, 0.0,  // bottom right
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
                uniform vec3 triangleColor;

                void main() {
                    FragColor = vec4(triangleColor, 1.0);
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

            // Get uniform locations
            let color_name = CString::new("triangleColor").unwrap();
            self.triangle_color_location = gl::GetUniformLocation(shader_program, color_name.as_ptr());

            // Delete the shaders as they're linked into the program and no longer needed
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            self.vertex_array = vao;
            self.vertex_buffer = vbo;
            self.shader_program = shader_program;
        }

        info!("OpenGL initialized successfully for backend: {}", self.current_backend);
        self.set_backend_colors(&self.current_backend.clone());
    }

    fn update(&mut self, delta_time: f32) {
        // Update rotation
        self.rotation += delta_time * 1.5;
        self.color_cycle_time += delta_time;

        // Keep rotation within 0-2π
        if self.rotation > std::f32::consts::TAU {
            self.rotation -= std::f32::consts::TAU;
        }
    }

    fn render(&mut self) {
        // Render
        unsafe {
            gl::ClearColor(
                self.background_color.0,
                self.background_color.1,
                self.background_color.2,
                1.0
            );
            gl::Clear(gl::COLOR_BUFFER_BIT);

            // Draw the triangle
            gl::UseProgram(self.shader_program);

            // Set rotation uniform
            let rotation_name = CString::new("rotation").unwrap();
            let rotation_loc = gl::GetUniformLocation(self.shader_program, rotation_name.as_ptr());
            gl::Uniform1f(rotation_loc, self.rotation);

            // Set triangle color with cycling effect
            let cycle = (self.color_cycle_time * 2.0).sin() * 0.3 + 0.7;
            if self.current_backend == "glfw" {
                gl::Uniform3f(self.triangle_color_location, 1.0 * cycle, 0.5 * cycle, 0.2 * cycle);
            } else {
                gl::Uniform3f(self.triangle_color_location, 0.2 * cycle, 1.0 * cycle, 0.5 * cycle);
            }

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
        info!("BackendSwitchingDemo shutdown complete!");
    }

    fn event(&mut self, event: &mut Event) {
        match event.event_type {
            EventType::Keyboard => {
                if let Some(key_event) = event.as_key_event() {
                    if key_event.action == KeyAction::Press {
                        match key_event.key {
                            KeyCode::G => {
                                info!("Requesting switch to GLFW backend");
                                self.switch_requested = Some("glfw".to_string());
                                event.mark_handled();
                            }
                            KeyCode::W => {
                                #[cfg(feature = "wayland")]
                                {
                                    info!("Requesting switch to Wayland backend");
                                    self.switch_requested = Some("wayland".to_string());
                                }
                                #[cfg(not(feature = "wayland"))]
                                {
                                    warn!("Wayland backend not available - compile with --features wayland");
                                }
                                event.mark_handled();
                            }
                            KeyCode::R => {
                                // Reset rotation on R key press
                                self.rotation = 0.0;
                                self.color_cycle_time = 0.0;
                                info!("Reset rotation and colors!");
                                event.mark_handled();
                            }
                            KeyCode::Escape => {
                                info!("Escape key pressed - closing application");
                            }
                            _ => {}
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn get_name(&self) -> &str {
        "Backend Switching Demo - Press G for GLFW, W for Wayland, R to reset"
    }
}

impl BackendSwitchingDemo {
    fn set_backend_colors(&mut self, backend: &str) {
        match backend {
            "glfw" => {
                self.background_color = (0.2, 0.3, 0.3); // Dark teal for GLFW
                info!("Set GLFW colors: dark teal background, orange triangle");
            }
            "wayland" => {
                self.background_color = (0.3, 0.2, 0.4); // Dark purple for Wayland
                info!("Set Wayland colors: dark purple background, green triangle");
            }
            _ => {
                self.background_color = (0.1, 0.1, 0.1); // Dark gray for unknown
                warn!("Unknown backend '{}', using default colors", backend);
            }
        }
    }

    fn on_backend_switched(&mut self, new_backend: &str) {
        self.current_backend = new_backend.to_string();
        self.set_backend_colors(new_backend);
        
        // Re-initialize OpenGL objects after backend switch
        self.init();
        
        info!("Successfully switched to {} backend!", new_backend);
    }
}

fn main() {
    artifice_logging::init_from_env().expect("Failed to initialize logger");
    info!("Starting Backend Switching Demo");

    // Create application
    let app = BackendSwitchingDemo::new();

    // Configure hot reload with reasonable timeouts
    let hot_reload_config = HotReloadConfig {
        switch_timeout: std::time::Duration::from_secs(10),
        preserve_state: true,
        buffer_events: true,
        max_buffered_events: 1000,
        validate_backend: true,
    };

    // Configure metrics for monitoring the switches
    let metrics_config = MetricsConfig {
        enabled: true,
        auto_reporting: true,
        report_interval: std::time::Duration::from_secs(5),
        max_event_types: 100,
    };

    // Create engine with GLFW as initial backend
    let mut engine = Engine::with_config(app, "glfw", metrics_config, hot_reload_config);

    info!("Engine created with GLFW backend");
    info!("Controls:");
    info!("  G - Switch to GLFW backend");
    info!("  W - Switch to Wayland backend (if available)");
    info!("  R - Reset rotation and colors");
    info!("  ESC - Exit application");

    // Set up OpenGL debug output if available
    unsafe {
        if gl::DebugMessageCallback::is_loaded() {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        }
    }

    // Run the engine normally - backend switching will be handled in event() method
    engine.run();

    info!("Backend Switching Demo completed");
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
        error!("Shader compilation failed: {}", log_str);
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
        error!("Program linking failed: {}", log_str);
    }
}