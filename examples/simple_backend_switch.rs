extern crate artifice_engine;
extern crate gl;
extern crate glfw;
extern crate artifice_logging;

use std::ffi::CString;

use artifice_engine::events::{
    Event, EventType, KeyAction, KeyCode,
};
use artifice_engine::{Engine, Application};
use artifice_engine::window::HotReloadConfig;
use artifice_engine::io::MetricsConfig;
use artifice_logging::{error, info, warn};

pub struct SimpleBackendSwitchDemo {
    vertex_array: u32,
    vertex_buffer: u32,
    shader_program: u32,
    rotation: f32,
    current_backend: String,
    switch_cooldown: f32,
    switch_pending: Option<String>,
}

impl Application for SimpleBackendSwitchDemo {
    fn new() -> Self {
        SimpleBackendSwitchDemo {
            vertex_array: 0,
            vertex_buffer: 0,
            shader_program: 0,
            rotation: 0.0,
            current_backend: "glfw".to_string(),
            switch_cooldown: 0.0,
            switch_pending: None,
        }
    }

    fn init(&mut self) {
        info!("Initializing SimpleBackendSwitchDemo with {} backend", self.current_backend);

        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0,
            -0.5, -0.5, 0.0,
            0.5, -0.5, 0.0,
        ];

        unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(
                0, 3, gl::FLOAT, gl::FALSE,
                (3 * std::mem::size_of::<f32>()) as i32,
                std::ptr::null(),
            );
            gl::EnableVertexAttribArray(0);

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
            ).unwrap();
            gl::ShaderSource(vertex_shader, 1, &vertex_shader_source.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_shader_source = CString::new(
                "#version 330 core
                out vec4 FragColor;
                void main() {
                    FragColor = vec4(1.0, 0.5, 0.2, 1.0);
                }",
            ).unwrap();
            gl::ShaderSource(fragment_shader, 1, &fragment_shader_source.as_ptr(), std::ptr::null());
            gl::CompileShader(fragment_shader);

            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            self.vertex_array = vao;
            self.vertex_buffer = vbo;
            self.shader_program = shader_program;
        }

        info!("OpenGL setup complete for {} backend", self.current_backend);
    }

    fn update(&mut self, delta_time: f32) {
        self.rotation += delta_time * 2.0;
        
        if self.switch_cooldown > 0.0 {
            self.switch_cooldown -= delta_time;
        }
        
        if self.rotation > std::f32::consts::TAU {
            self.rotation -= std::f32::consts::TAU;
        }
    }

    fn render(&mut self) {
        unsafe {
            if self.current_backend == "glfw" {
                gl::ClearColor(0.2, 0.3, 0.3, 1.0);
            } else {
                gl::ClearColor(0.3, 0.2, 0.4, 1.0);
            }
            
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.shader_program);

            let rotation_name = CString::new("rotation").unwrap();
            let rotation_loc = gl::GetUniformLocation(self.shader_program, rotation_name.as_ptr());
            gl::Uniform1f(rotation_loc, self.rotation);

            gl::BindVertexArray(self.vertex_array);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    fn shutdown(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array);
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteProgram(self.shader_program);
        }
        info!("SimpleBackendSwitchDemo shutdown complete!");
    }

    fn event(&mut self, event: &mut Event) {
        match event.event_type {
            EventType::Keyboard => {
                if let Some(key_event) = event.as_key_event() {
                    if key_event.action == KeyAction::Press && self.switch_cooldown <= 0.0 {
                        match key_event.key {
                            KeyCode::G => {
                                if self.current_backend != "glfw" {
                                    info!("Requesting switch to GLFW backend...");
                                    self.switch_pending = Some("glfw".to_string());
                                    self.switch_cooldown = 1.0;
                                    event.mark_handled();
                                }
                            }
                            KeyCode::W => {
                                #[cfg(feature = "wayland")]
                                {
                                    if self.current_backend != "wayland" {
                                        info!("Requesting switch to Wayland backend...");
                                        self.switch_pending = Some("wayland".to_string());
                                        self.switch_cooldown = 1.0;
                                        event.mark_handled();
                                    }
                                }
                                #[cfg(not(feature = "wayland"))]
                                {
                                    warn!("Wayland backend not available");
                                }
                            }
                            KeyCode::R => {
                                self.rotation = 0.0;
                                info!("Reset rotation!");
                                event.mark_handled();
                            }
                            KeyCode::Escape => {
                                info!("Escape pressed - exiting");
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
        "Simple Backend Switch Demo - G:GLFW W:Wayland R:Reset ESC:Exit"
    }

    fn get_pending_backend_switch(&self) -> Option<String> {
        self.switch_pending.clone()
    }

    fn clear_pending_backend_switch(&mut self) {
        self.switch_pending = None;
    }

    fn on_backend_switch_completed(&mut self, old_backend: &str, new_backend: &str) {
        self.current_backend = new_backend.to_string();
        
        // Re-initialize graphics after backend switch
        self.init();
        
        info!("✓ Backend switch completed: {} → {}", old_backend, new_backend);
    }
}

fn main() {
    artifice_logging::init_from_env().expect("Failed to initialize logger");
    info!("Starting Simple Backend Switch Demo");

    let app = SimpleBackendSwitchDemo::new();

    let hot_reload_config = HotReloadConfig {
        switch_timeout: std::time::Duration::from_secs(5),
        preserve_state: true,
        buffer_events: true,
        max_buffered_events: 1000,
        validate_backend: true,
    };

    let metrics_config = MetricsConfig {
        enabled: true,
        auto_reporting: false,
        report_interval: std::time::Duration::from_secs(30),
        max_event_types: 50,
    };

    let mut engine = Engine::with_config(app, "glfw", metrics_config, hot_reload_config);

    info!("Controls:");
    info!("  G - Switch to GLFW backend (teal background)");
    info!("  W - Switch to Wayland backend (purple background)");
    info!("  R - Reset rotation");
    info!("  ESC - Exit");

    unsafe {
        if gl::DebugMessageCallback::is_loaded() {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        }
    }

    engine.run();

    info!("Simple Backend Switch Demo completed");
}