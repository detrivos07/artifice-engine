extern crate artifice_engine;
extern crate gl;
extern crate artifice_logging;

use std::ffi::CString;
use std::time::Instant;

use artifice_engine::events::{Event, EventType, KeyAction, KeyCode};
use artifice_engine::{Engine, Application};
use artifice_engine::window::HotReloadConfig;
use artifice_engine::io::MetricsConfig;
use artifice_logging::{info, warn, error};

pub struct VisualX11Test {
    vertex_array: u32,
    vertex_buffer: u32,
    shader_program: u32,
    rotation: f32,
    current_backend: String,
    switch_requested: Option<String>,
    start_time: Instant,
    switch_count: u32,
    last_switch_time: Option<Instant>,
    frame_count: u64,
}

impl Application for VisualX11Test {
    fn new() -> Self {
        VisualX11Test {
            vertex_array: 0,
            vertex_buffer: 0,
            shader_program: 0,
            rotation: 0.0,
            current_backend: "glfw".to_string(),
            switch_requested: None,
            start_time: Instant::now(),
            switch_count: 0,
            last_switch_time: None,
            frame_count: 0,
        }
    }

    fn init(&mut self) {
        info!("🔧 VISUAL TEST: Initializing OpenGL for backend: {}", self.current_backend);
        
        // Clean up any existing objects
        self.cleanup_opengl();

        // Verify OpenGL context immediately
        self.verify_opengl_context();

        // Create a more complex shape for better visual feedback
        let vertices: [f32; 18] = [
            // Triangle 1 (top)
            0.0, 0.6, 0.0,
            -0.3, 0.2, 0.0,
            0.3, 0.2, 0.0,
            // Triangle 2 (bottom)  
            -0.3, -0.2, 0.0,
            0.3, -0.2, 0.0,
            0.0, -0.6, 0.0,
        ];

        unsafe {
            // Create VAO
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            // Create VBO
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );

            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, std::ptr::null());
            gl::EnableVertexAttribArray(0);

            // Create shaders with backend-specific colors
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let vertex_source = CString::new(
                "#version 330 core\n\
                layout (location = 0) in vec3 aPos;\n\
                uniform float rotation;\n\
                uniform float time;\n\
                void main() {\n\
                    float angle = rotation + time * 0.5;\n\
                    float x = aPos.x * cos(angle) - aPos.y * sin(angle);\n\
                    float y = aPos.x * sin(angle) + aPos.y * cos(angle);\n\
                    gl_Position = vec4(x, y, aPos.z, 1.0);\n\
                }"
            ).unwrap();
            gl::ShaderSource(vertex_shader, 1, &vertex_source.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_source = CString::new(
                "#version 330 core\n\
                out vec4 FragColor;\n\
                uniform vec3 color;\n\
                uniform float pulse;\n\
                void main() {\n\
                    float intensity = 0.7 + 0.3 * pulse;\n\
                    FragColor = vec4(color * intensity, 1.0);\n\
                }"
            ).unwrap();
            gl::ShaderSource(fragment_shader, 1, &fragment_source.as_ptr(), std::ptr::null());
            gl::CompileShader(fragment_shader);

            // Create program
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            self.vertex_array = vao;
            self.vertex_buffer = vbo;
            self.shader_program = program;

            // Test immediate rendering
            self.test_immediate_render();
        }

        info!("✅ VISUAL TEST: OpenGL setup complete for {} backend", self.current_backend);
    }

    fn update(&mut self, delta_time: f32) {
        self.rotation += delta_time * 2.0;
        self.frame_count += 1;

        // Auto-switch every 5 seconds for demonstration
        let elapsed = self.start_time.elapsed().as_secs();
        if elapsed > 0 && elapsed % 5 == 0 && self.last_switch_time.map_or(true, |t| t.elapsed().as_secs() >= 5) {
            let target_backend = if self.current_backend == "glfw" { "x11" } else { "glfw" };
            info!("🔄 AUTO-SWITCHING: {} -> {}", self.current_backend, target_backend);
            self.switch_requested = Some(target_backend.to_string());
            self.last_switch_time = Some(Instant::now());
        }
    }

    fn render(&mut self) {
        let time = self.start_time.elapsed().as_secs_f32();
        
        unsafe {
            // Set background color based on backend
            match self.current_backend.as_str() {
                "glfw" => gl::ClearColor(0.1, 0.3, 0.1, 1.0), // Green tint for GLFW
                "x11" => gl::ClearColor(0.3, 0.1, 0.1, 1.0),  // Red tint for X11
                _ => gl::ClearColor(0.1, 0.1, 0.3, 1.0),      // Blue tint for unknown
            }
            gl::Clear(gl::COLOR_BUFFER_BIT);

            if self.vertex_array != 0 && self.shader_program != 0 {
                gl::UseProgram(self.shader_program);

                // Set uniforms
                let rotation_loc = gl::GetUniformLocation(self.shader_program, CString::new("rotation").unwrap().as_ptr());
                gl::Uniform1f(rotation_loc, self.rotation);

                let time_loc = gl::GetUniformLocation(self.shader_program, CString::new("time").unwrap().as_ptr());
                gl::Uniform1f(time_loc, time);

                let pulse_loc = gl::GetUniformLocation(self.shader_program, CString::new("pulse").unwrap().as_ptr());
                gl::Uniform1f(pulse_loc, (time * 3.0).sin());

                // Set color based on backend
                let color_loc = gl::GetUniformLocation(self.shader_program, CString::new("color").unwrap().as_ptr());
                match self.current_backend.as_str() {
                    "glfw" => gl::Uniform3f(color_loc, 1.0, 0.8, 0.2), // Golden for GLFW
                    "x11" => gl::Uniform3f(color_loc, 0.2, 0.8, 1.0),  // Cyan for X11
                    _ => gl::Uniform3f(color_loc, 1.0, 0.2, 1.0),      // Magenta for unknown
                }

                gl::BindVertexArray(self.vertex_array);
                gl::DrawArrays(gl::TRIANGLES, 0, 6);

                // Check for errors
                let error = gl::GetError();
                if error != gl::NO_ERROR {
                    error!("❌ VISUAL TEST: OpenGL error in render: 0x{:X}", error);
                }
            }
        }

        // Log status every 60 frames
        if self.frame_count % 60 == 0 {
            info!("📊 VISUAL TEST: Backend={}, Frame={}, Switches={}", 
                  self.current_backend, self.frame_count, self.switch_count);
        }
    }

    fn shutdown(&mut self) {
        info!("🔄 VISUAL TEST: Shutting down");
        self.cleanup_opengl();
    }

    fn event(&mut self, event: &mut Event) {
        if let EventType::Keyboard = event.event_type {
            if let Some(key_event) = event.as_key_event() {
                if key_event.action == KeyAction::Press {
                    match key_event.key {
                        KeyCode::X => {
                            info!("🎯 MANUAL X11 SWITCH REQUESTED");
                            self.switch_requested = Some("x11".to_string());
                            event.mark_handled();
                        }
                        KeyCode::G => {
                            info!("🎯 MANUAL GLFW SWITCH REQUESTED");
                            self.switch_requested = Some("glfw".to_string());
                            event.mark_handled();
                        }
                        KeyCode::Space => {
                            self.print_status();
                            event.mark_handled();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn get_name(&self) -> &str {
        "🎨 VISUAL X11 TEST - Press X (X11), G (GLFW), SPACE (status) - Auto-switches every 5s"
    }

    fn get_pending_backend_switch(&self) -> Option<String> {
        self.switch_requested.clone()
    }

    fn clear_pending_backend_switch(&mut self) {
        self.switch_requested = None;
    }

    fn on_backend_switch_completed(&mut self, old_backend: &str, new_backend: &str) {
        self.switch_count += 1;
        info!("🚀 VISUAL TEST: BACKEND SWITCH COMPLETED!");
        info!("   📌 Old: {} -> New: {}", old_backend, new_backend);
        info!("   🔢 Switch count: {}", self.switch_count);
        
        self.current_backend = new_backend.to_string();
        
        // Verify OpenGL state after switch
        self.verify_opengl_context();
        
        // Re-initialize (this will test if OpenGL functions work)
        info!("   🔧 Re-initializing OpenGL objects...");
        self.init();
        
        // Test immediate rendering after switch
        info!("   🎨 Testing post-switch rendering...");
        self.test_immediate_render();
        
        info!("✅ VISUAL TEST: Backend switch verification complete!");
        self.print_status();
    }
}

impl VisualX11Test {
    fn cleanup_opengl(&mut self) {
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

    fn verify_opengl_context(&self) {
        info!("🔍 VISUAL TEST: Verifying OpenGL context for {}", self.current_backend);
        
        unsafe {
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                error!("❌ OpenGL error state: 0x{:X}", error);
                return;
            }

            let version = gl::GetString(gl::VERSION);
            if version.is_null() {
                error!("❌ Failed to get OpenGL version - context invalid!");
                return;
            }

            let version_str = std::ffi::CStr::from_ptr(version as *const i8).to_string_lossy();
            let renderer = gl::GetString(gl::RENDERER);
            let renderer_str = if renderer.is_null() {
                "Unknown".to_string()
            } else {
                std::ffi::CStr::from_ptr(renderer as *const i8).to_string_lossy().to_string()
            };

            info!("✅ OpenGL Context Verified:");
            info!("   📋 Version: {}", version_str);
            info!("   🖥️  Renderer: {}", renderer_str);
            info!("   🎯 Backend: {}", self.current_backend);
        }
    }

    fn test_immediate_render(&self) {
        unsafe {
            // Test basic OpenGL calls
            gl::ClearColor(1.0, 0.0, 1.0, 1.0); // Magenta flash
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                error!("❌ Immediate render test failed: 0x{:X}", error);
            } else {
                info!("✅ Immediate render test passed");
            }
        }
    }

    fn print_status(&self) {
        let uptime = self.start_time.elapsed().as_secs();
        info!("📊 === VISUAL TEST STATUS ===");
        info!("   🎯 Current Backend: {}", self.current_backend);
        info!("   🔢 Total Switches: {}", self.switch_count);
        info!("   ⏱️  Uptime: {}s", uptime);
        info!("   🖼️  Frame Count: {}", self.frame_count);
        info!("   🎨 Expected Visual:");
        match self.current_backend.as_str() {
            "glfw" => info!("      🟢 Green background, 🟡 Golden rotating double-triangle"),
            "x11" => info!("      🔴 Red background, 🔵 Cyan rotating double-triangle"),
            _ => info!("      🔵 Blue background, 🟣 Magenta rotating double-triangle"),
        }
        info!("========================");
    }
}

fn main() {
    artifice_logging::init_from_env().expect("Failed to initialize logger");
    
    info!("🎨 === VISUAL X11 BACKEND TEST ===");
    info!("This test provides clear visual feedback for backend switching:");
    info!("  🟢 GLFW: Green background + Golden shapes");
    info!("  🔴 X11:  Red background + Cyan shapes");
    info!("  ⚡ Auto-switches every 5 seconds");
    info!("  🎮 Manual: X=X11, G=GLFW, SPACE=status");
    info!("=====================================");

    let app = VisualX11Test::new();

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
        report_interval: std::time::Duration::from_secs(30),
        max_event_types: 100,
    };

    let mut engine = Engine::with_config(app, "glfw", metrics_config, hot_reload_config);
    
    info!("🚀 Starting visual test engine...");
    engine.run();
    
    info!("🏁 Visual X11 test completed");
}