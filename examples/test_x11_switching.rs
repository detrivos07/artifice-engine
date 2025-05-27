extern crate artifice_engine;
extern crate gl;
extern crate artifice_logging;

use std::ffi::CString;
use std::time::Instant;

use artifice_engine::events::{Event, EventType, KeyAction, KeyCode};
use artifice_engine::{Engine, Application};
use artifice_engine::window::HotReloadConfig;
use artifice_engine::io::MetricsConfig;
use artifice_logging::{error, info, warn, debug};

pub struct X11SwitchTest {
    vertex_array: u32,
    vertex_buffer: u32,
    shader_program: u32,
    current_backend: String,
    switch_requested: Option<String>,
    test_phase: TestPhase,
    switch_time: Option<Instant>,
}

#[derive(Debug, Clone)]
enum TestPhase {
    InitialRender,
    RequestingSwitch,
    SwitchCompleted,
    TestingOpenGL,
    TestComplete,
}

impl Application for X11SwitchTest {
    fn new() -> Self {
        X11SwitchTest {
            vertex_array: 0,
            vertex_buffer: 0,
            shader_program: 0,
            current_backend: "glfw".to_string(),
            switch_requested: None,
            test_phase: TestPhase::InitialRender,
            switch_time: None,
        }
    }

    fn init(&mut self) {
        info!("=== X11SwitchTest::init() called for backend: {} ===", self.current_backend);
        
        // Clean up any existing objects first
        self.cleanup_opengl();

        // Test basic OpenGL state
        self.test_opengl_context();

        // Define vertex data for a simple triangle
        let vertices: [f32; 9] = [
            0.0, 0.5, 0.0,   // top
            -0.5, -0.5, 0.0, // bottom left
            0.5, -0.5, 0.0,  // bottom right
        ];

        unsafe {
            // Create vertex array object
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            if vao == 0 {
                error!("Failed to generate vertex array object");
                return;
            }
            gl::BindVertexArray(vao);
            info!("Created VAO: {}", vao);

            // Create vertex buffer object
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            if vbo == 0 {
                error!("Failed to generate vertex buffer object");
                return;
            }
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            info!("Created VBO: {}", vbo);

            // Configure vertex attributes
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            // Create vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let vertex_source = CString::new(
                "#version 330 core\n\
                layout (location = 0) in vec3 aPos;\n\
                void main() {\n\
                    gl_Position = vec4(aPos.x, aPos.y, aPos.z, 1.0);\n\
                }"
            ).unwrap();
            gl::ShaderSource(vertex_shader, 1, &vertex_source.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);
            
            if !self.check_shader_compilation(vertex_shader, "vertex") {
                return;
            }

            // Create fragment shader
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_source = CString::new(
                "#version 330 core\n\
                out vec4 FragColor;\n\
                void main() {\n\
                    FragColor = vec4(1.0, 0.5, 0.2, 1.0);\n\
                }"
            ).unwrap();
            gl::ShaderSource(fragment_shader, 1, &fragment_source.as_ptr(), std::ptr::null());
            gl::CompileShader(fragment_shader);
            
            if !self.check_shader_compilation(fragment_shader, "fragment") {
                return;
            }

            // Create shader program
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            
            if !self.check_program_linking(shader_program) {
                return;
            }

            // Clean up shaders
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            self.vertex_array = vao;
            self.vertex_buffer = vbo;
            self.shader_program = shader_program;
        }

        info!("OpenGL objects created successfully for {}: VAO={}, VBO={}, Program={}", 
              self.current_backend, self.vertex_array, self.vertex_buffer, self.shader_program);

        // Test rendering immediately after init
        self.test_render();
    }

    fn update(&mut self, _delta_time: f32) {
        // Auto-trigger X11 switch after 2 seconds
        if matches!(self.test_phase, TestPhase::InitialRender) {
            if self.switch_time.is_none() {
                self.switch_time = Some(Instant::now());
            } else if self.switch_time.unwrap().elapsed().as_secs() >= 2 {
                info!("=== AUTO-TRIGGERING X11 SWITCH ===");
                self.switch_requested = Some("x11".to_string());
                self.test_phase = TestPhase::RequestingSwitch;
            }
        }
    }

    fn render(&mut self) {
        unsafe {
            match self.current_backend.as_str() {
                "glfw" => gl::ClearColor(0.2, 0.3, 0.3, 1.0),
                "x11" => gl::ClearColor(0.4, 0.3, 0.2, 1.0),
                _ => gl::ClearColor(0.1, 0.1, 0.1, 1.0),
            }
            gl::Clear(gl::COLOR_BUFFER_BIT);

            if self.vertex_array != 0 && self.shader_program != 0 {
                gl::UseProgram(self.shader_program);
                gl::BindVertexArray(self.vertex_array);
                gl::DrawArrays(gl::TRIANGLES, 0, 3);
                
                // Check for OpenGL errors
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    warn!("OpenGL error during render: 0x{:X}", error);
                }
            } else {
                warn!("Skipping render - OpenGL objects not initialized (VAO={}, Program={})", 
                      self.vertex_array, self.shader_program);
            }
        }
    }

    fn shutdown(&mut self) {
        info!("X11SwitchTest shutdown");
        self.cleanup_opengl();
    }

    fn event(&mut self, event: &mut Event) {
        if let EventType::Keyboard = event.event_type {
            if let Some(key_event) = event.as_key_event() {
                if key_event.action == KeyAction::Press && key_event.key == KeyCode::X {
                    info!("Manual X11 switch requested");
                    self.switch_requested = Some("x11".to_string());
                    self.test_phase = TestPhase::RequestingSwitch;
                    event.mark_handled();
                }
            }
        }
    }

    fn get_name(&self) -> &str {
        "X11 Switch Test - Press X to switch to X11"
    }

    fn get_pending_backend_switch(&self) -> Option<String> {
        self.switch_requested.clone()
    }

    fn clear_pending_backend_switch(&mut self) {
        self.switch_requested = None;
    }

    fn on_backend_switch_completed(&mut self, old_backend: &str, new_backend: &str) {
        info!("=== BACKEND SWITCH COMPLETED: {} -> {} ===", old_backend, new_backend);
        self.current_backend = new_backend.to_string();
        self.test_phase = TestPhase::SwitchCompleted;
        
        // Test OpenGL immediately after switch
        self.test_opengl_context();
        
        // Re-initialize OpenGL objects
        info!("Re-initializing OpenGL objects after backend switch");
        self.init();
        
        self.test_phase = TestPhase::TestingOpenGL;
        
        // Test rendering after switch
        info!("Testing render after backend switch");
        self.test_render();
        
        self.test_phase = TestPhase::TestComplete;
        info!("=== X11 SWITCH TEST COMPLETED ===");
    }
}

impl X11SwitchTest {
    fn test_opengl_context(&self) {
        info!("=== Testing OpenGL context for backend: {} ===", self.current_backend);
        
        unsafe {
            // Test basic OpenGL calls
            let version = gl::GetString(gl::VERSION);
            if version.is_null() {
                error!("Failed to get OpenGL version - context is not working!");
            } else {
                let version_str = std::ffi::CStr::from_ptr(version as *const i8).to_string_lossy();
                info!("OpenGL version: {}", version_str);
            }
            
            let renderer = gl::GetString(gl::RENDERER);
            if !renderer.is_null() {
                let renderer_str = std::ffi::CStr::from_ptr(renderer as *const i8).to_string_lossy();
                info!("OpenGL renderer: {}", renderer_str);
            }
            
            let vendor = gl::GetString(gl::VENDOR);
            if !vendor.is_null() {
                let vendor_str = std::ffi::CStr::from_ptr(vendor as *const i8).to_string_lossy();
                info!("OpenGL vendor: {}", vendor_str);
            }
            
            // Test error state
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                error!("OpenGL error state: 0x{:X}", error);
            } else {
                info!("OpenGL error state: OK");
            }
        }
    }
    
    fn test_render(&self) {
        info!("=== Testing immediate render ===");
        unsafe {
            gl::ClearColor(1.0, 0.0, 0.0, 1.0); // Red clear
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                error!("OpenGL error during test render: 0x{:X}", error);
            } else {
                info!("Test render successful");
            }
        }
    }
    
    fn cleanup_opengl(&mut self) {
        if self.vertex_array != 0 || self.vertex_buffer != 0 || self.shader_program != 0 {
            info!("Cleaning up OpenGL objects: VAO={}, VBO={}, Program={}", 
                  self.vertex_array, self.vertex_buffer, self.shader_program);
            
            unsafe {
                if self.vertex_array != 0 {
                    gl::DeleteVertexArrays(1, &self.vertex_array);
                    self.vertex_array = 0;
                }
                if self.vertex_buffer != 0 {
                    gl::DeleteBuffers(1, &self.vertex_buffer);
                    self.vertex_buffer = 0;
                }
                if self.shader_program != 0 {
                    gl::DeleteProgram(self.shader_program);
                    self.shader_program = 0;
                }
            }
        }
    }
    
    fn check_shader_compilation(&self, shader: u32, shader_type: &str) -> bool {
        unsafe {
            let mut success = 0;
            gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                let mut log_length = 0;
                gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);
                let mut log = Vec::with_capacity(log_length as usize);
                log.set_len(log_length as usize);
                gl::GetShaderInfoLog(shader, log_length, std::ptr::null_mut(), log.as_mut_ptr() as *mut i8);
                let log_str = std::str::from_utf8(&log).unwrap_or("Unknown error");
                error!("{} shader compilation failed: {}", shader_type, log_str);
                false
            } else {
                info!("{} shader compiled successfully", shader_type);
                true
            }
        }
    }
    
    fn check_program_linking(&self, program: u32) -> bool {
        unsafe {
            let mut success = 0;
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                let mut log_length = 0;
                gl::GetProgramiv(program, gl::INFO_LOG_LENGTH, &mut log_length);
                let mut log = Vec::with_capacity(log_length as usize);
                log.set_len(log_length as usize);
                gl::GetProgramInfoLog(program, log_length, std::ptr::null_mut(), log.as_mut_ptr() as *mut i8);
                let log_str = std::str::from_utf8(&log).unwrap_or("Unknown error");
                error!("Shader program linking failed: {}", log_str);
                false
            } else {
                info!("Shader program linked successfully");
                true
            }
        }
    }
}

fn main() {
    artifice_logging::init_from_env().expect("Failed to initialize logger");
    info!("=== Starting X11 Switch Test ===");

    let app = X11SwitchTest::new();

    let hot_reload_config = HotReloadConfig {
        switch_timeout: std::time::Duration::from_secs(10),
        preserve_state: true,
        buffer_events: true,
        max_buffered_events: 1000,
        validate_backend: true,
    };

    let metrics_config = MetricsConfig {
        enabled: true,
        auto_reporting: false,
        report_interval: std::time::Duration::from_secs(60),
        max_event_types: 100,
    };

    let mut engine = Engine::with_config(app, "glfw", metrics_config, hot_reload_config);
    
    info!("Starting engine - will auto-switch to X11 after 2 seconds");
    info!("Or press X to manually trigger switch");
    
    engine.run();
    
    info!("=== X11 Switch Test Completed ===");
}