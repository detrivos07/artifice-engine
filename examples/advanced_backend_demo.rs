extern crate artifice_engine;
extern crate gl;
extern crate glfw;
extern crate artifice_logging;

use std::ffi::CString;
use std::time::{Duration, Instant};

use artifice_engine::events::{
    Event, EventType, KeyAction, KeyCode,
};
use artifice_engine::{Engine, Application, Layer};
use artifice_engine::window::{HotReloadConfig, HotReloadStatus, HotReloadBuilder};
use artifice_engine::io::MetricsConfig;
use artifice_logging::{error, info, warn, debug};

pub struct AdvancedBackendDemo {
    // OpenGL objects
    vertex_array: u32,
    vertex_buffer: u32,
    shader_program: u32,

    // Animation state
    rotation: f32,
    scale_pulse: f32,
    color_cycle: f32,

    // Backend switching
    current_backend: String,
    switch_pending: Option<String>,
    switch_cooldown: f32,
    backend_switch_count: u32,

    // Color schemes for different backends
    background_colors: BackendColors,
    triangle_colors: BackendColors,

    // Performance tracking
    frame_count: u32,
    last_fps_update: Instant,
    current_fps: f32,

    // Shader uniform locations
    rotation_location: i32,
    scale_location: i32,
    color_location: i32,
    time_location: i32,

    // OpenGL availability tracking
    opengl_available: bool,
}

#[derive(Clone)]
struct BackendColors {
    glfw: (f32, f32, f32),
    wayland: (f32, f32, f32),
    x11: (f32, f32, f32),
    default: (f32, f32, f32),
}

impl BackendColors {
    fn new_background() -> Self {
        Self {
            glfw: (0.1, 0.2, 0.4),      // Deep blue for GLFW
            wayland: (0.2, 0.4, 0.2),   // Forest green for Wayland
            x11: (0.4, 0.2, 0.1),       // Brown for X11
            default: (0.2, 0.2, 0.2),   // Gray for unknown
        }
    }
    
    fn new_triangle() -> Self {
        Self {
            glfw: (1.0, 0.6, 0.0),      // Orange for GLFW
            wayland: (0.8, 0.2, 0.8),   // Purple for Wayland
            x11: (0.2, 0.8, 1.0),       // Cyan for X11
            default: (0.5, 0.5, 0.5),   // Gray for unknown
        }
    }
    
    fn get(&self, backend: &str) -> (f32, f32, f32) {
        match backend {
            "glfw" => self.glfw,
            "wayland" => self.wayland,
            "x11" => self.x11,
            _ => self.default,
        }
    }
}

pub struct BackendInfoLayer {
    last_update: Instant,
    start_time: Instant,
}

impl BackendInfoLayer {
    fn new() -> Self {
        Self {
            last_update: Instant::now(),
            start_time: Instant::now(),
        }
    }
}

impl Layer for BackendInfoLayer {
    fn update(&mut self, _delta_time: f32) {
        // Layer update logic can go here
    }
    
    fn render(&mut self) {
        // In a real implementation, this would render UI elements
        // For now, we just log periodically to show the layer is active
        if self.last_update.elapsed() > Duration::from_secs(5) {
            let uptime = self.start_time.elapsed().as_secs();
            info!("Info Layer Active | Uptime: {}s | Use SPACE for current status", uptime);
            self.last_update = Instant::now();
        }
    }
    
    fn get_name(&self) -> &str {
        "Backend Info Layer"
    }
}

impl Application for AdvancedBackendDemo {
    fn new() -> Self {
        AdvancedBackendDemo {
            vertex_array: 0,
            vertex_buffer: 0,
            shader_program: 0,
            rotation: 0.0,
            scale_pulse: 1.0,
            color_cycle: 0.0,
            current_backend: "unknown".to_string(),
            switch_pending: None,
            switch_cooldown: 0.0,
            backend_switch_count: 0,
            background_colors: BackendColors::new_background(),
            triangle_colors: BackendColors::new_triangle(),
            frame_count: 0,
            last_fps_update: Instant::now(),
            current_fps: 0.0,
            rotation_location: -1,
            scale_location: -1,
            color_location: -1,
            time_location: -1,
            opengl_available: false,
        }
    }

    fn init(&mut self) {
        info!("Initializing AdvancedBackendDemo with {} backend", self.current_backend);

        // Check if OpenGL is available
        self.opengl_available = self.check_opengl_availability();
        
        if !self.opengl_available {
            info!("OpenGL not available for {} backend, using software fallback", self.current_backend);
            return;
        }

        // Enhanced vertex data for a more interesting triangle
        let vertices: [f32; 9] = [
            0.0, 0.6, 0.0,    // top
            -0.5, -0.3, 0.0,  // bottom left
            0.5, -0.3, 0.0,   // bottom right
        ];

        unsafe {
            // Create VAO and VBO
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

            // Enhanced shaders with more uniforms
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let vertex_shader_source = CString::new(
                "#version 330 core
                layout (location = 0) in vec3 aPos;
                uniform float rotation;
                uniform float scale;
                uniform float time;

                void main() {
                    // Apply scale
                    vec3 pos = aPos * scale;
                    
                    // Apply rotation
                    float angle = rotation;
                    float x = pos.x * cos(angle) - pos.y * sin(angle);
                    float y = pos.x * sin(angle) + pos.y * cos(angle);
                    
                    // Add subtle wobble based on time
                    float wobble = sin(time * 3.0) * 0.05;
                    x += wobble * pos.y;
                    y += wobble * pos.x;
                    
                    gl_Position = vec4(x, y, pos.z, 1.0);
                }",
            ).unwrap();
            gl::ShaderSource(vertex_shader, 1, &vertex_shader_source.as_ptr(), std::ptr::null());
            gl::CompileShader(vertex_shader);
            check_shader_compilation(vertex_shader);

            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_shader_source = CString::new(
                "#version 330 core
                out vec4 FragColor;
                uniform vec3 triangleColor;
                uniform float time;

                void main() {
                    // Add subtle color variation based on fragment position
                    vec2 uv = gl_FragCoord.xy / 800.0; // Assuming 800x600 window
                    float colorMod = sin(time + uv.x * 10.0 + uv.y * 10.0) * 0.1 + 0.9;
                    FragColor = vec4(triangleColor * colorMod, 1.0);
                }",
            ).unwrap();
            gl::ShaderSource(fragment_shader, 1, &fragment_shader_source.as_ptr(), std::ptr::null());
            gl::CompileShader(fragment_shader);
            check_shader_compilation(fragment_shader);

            // Link shader program
            let shader_program = gl::CreateProgram();
            gl::AttachShader(shader_program, vertex_shader);
            gl::AttachShader(shader_program, fragment_shader);
            gl::LinkProgram(shader_program);
            check_program_linking(shader_program);

            // Get uniform locations
            let rotation_name = CString::new("rotation").unwrap();
            let scale_name = CString::new("scale").unwrap();
            let color_name = CString::new("triangleColor").unwrap();
            let time_name = CString::new("time").unwrap();
            
            self.rotation_location = gl::GetUniformLocation(shader_program, rotation_name.as_ptr());
            self.scale_location = gl::GetUniformLocation(shader_program, scale_name.as_ptr());
            self.color_location = gl::GetUniformLocation(shader_program, color_name.as_ptr());
            self.time_location = gl::GetUniformLocation(shader_program, time_name.as_ptr());

            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);

            self.vertex_array = vao;
            self.vertex_buffer = vbo;
            self.shader_program = shader_program;

            // Enable blending for smoother visuals
            gl::Enable(gl::BLEND);
            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);
        }

        info!("OpenGL setup complete for {} backend", self.current_backend);
    }



    fn update(&mut self, delta_time: f32) {
        // Update animations
        self.rotation += delta_time * 2.0;
        self.color_cycle += delta_time * 1.5;
        
        // Pulsing scale effect
        self.scale_pulse = 0.8 + 0.3 * (self.color_cycle * 2.0).sin();
        
        // Update switch cooldown
        if self.switch_cooldown > 0.0 {
            self.switch_cooldown -= delta_time;
        }
        
        // Keep values in reasonable ranges
        if self.rotation > std::f32::consts::TAU {
            self.rotation -= std::f32::consts::TAU;
        }
        
        // FPS calculation
        self.frame_count += 1;
        if self.last_fps_update.elapsed() >= Duration::from_secs(1) {
            self.current_fps = self.frame_count as f32 / self.last_fps_update.elapsed().as_secs_f32();
            self.frame_count = 0;
            self.last_fps_update = Instant::now();
        }
    }

    fn render(&mut self) {
        if !self.opengl_available {
            // For non-OpenGL backends like Wayland, we could implement
            // software rendering here, but for now just return
            return;
        }

        unsafe {
            let bg_color = self.background_colors.get(&self.current_backend);
            gl::ClearColor(bg_color.0, bg_color.1, bg_color.2, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);

            gl::UseProgram(self.shader_program);

            // Set uniforms
            gl::Uniform1f(self.rotation_location, self.rotation);
            gl::Uniform1f(self.scale_location, self.scale_pulse);
            gl::Uniform1f(self.time_location, self.color_cycle);
            
            let triangle_color = self.triangle_colors.get(&self.current_backend);
            let color_mod = (self.color_cycle * 0.5).sin() * 0.3 + 0.7;
            gl::Uniform3f(
                self.color_location,
                triangle_color.0 * color_mod,
                triangle_color.1 * color_mod,
                triangle_color.2 * color_mod,
            );

            gl::BindVertexArray(self.vertex_array);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
        }
    }

    fn shutdown(&mut self) {
        if self.opengl_available {
            unsafe {
                gl::DeleteVertexArrays(1, &self.vertex_array);
                gl::DeleteBuffers(1, &self.vertex_buffer);
                gl::DeleteProgram(self.shader_program);
            }
        }
        info!("AdvancedBackendDemo shutdown complete");
    }

    fn event(&mut self, event: &mut Event) {
        match event.event_type {
            EventType::Keyboard => {
                if let Some(key_event) = event.as_key_event() {
                    if key_event.action == KeyAction::Press {
                        match key_event.key {
                            KeyCode::G => {
                                if self.switch_cooldown <= 0.0 && self.current_backend != "glfw" {
                                    info!("Requesting switch to GLFW backend");
                                    self.switch_pending = Some("glfw".to_string());
                                    self.switch_cooldown = 2.0; // 2 second cooldown
                                    event.mark_handled();
                                } else if self.current_backend == "glfw" {
                                    info!("Already using GLFW backend");
                                } else {
                                    info!("Switch cooldown active, please wait...");
                                }
                            }
                            KeyCode::W => {
                                #[cfg(feature = "wayland")]
                                {
                                    if self.switch_cooldown <= 0.0 && self.current_backend != "wayland" {
                                        info!("Requesting switch to Wayland backend");
                                        self.switch_pending = Some("wayland".to_string());
                                        self.switch_cooldown = 2.0;
                                    } else if self.current_backend == "wayland" {
                                        info!("Already using Wayland backend");
                                    } else {
                                        info!("Switch cooldown active, please wait...");
                                    }
                                }
                                #[cfg(not(feature = "wayland"))]
                                {
                                    warn!("Wayland backend not available - compile with --features wayland");
                                }
                                event.mark_handled();
                            }
                            KeyCode::X => {
                                #[cfg(feature = "x11")]
                                {
                                    if self.switch_cooldown <= 0.0 && self.current_backend != "x11" {
                                        info!("Requesting switch to X11 backend");
                                        self.switch_pending = Some("x11".to_string());
                                        self.switch_cooldown = 2.0; // 2 second cooldown
                                        event.mark_handled();
                                    } else if self.current_backend == "x11" {
                                        info!("Already using X11 backend");
                                    } else {
                                        info!("Switch cooldown active, please wait...");
                                    }
                                }
                                #[cfg(not(feature = "x11"))]
                                {
                                    warn!("X11 backend not available - compile with --features x11");
                                }
                                event.mark_handled();
                            }
                            KeyCode::R => {
                                self.rotation = 0.0;
                                self.color_cycle = 0.0;
                                self.scale_pulse = 1.0;
                                info!("Reset all animations!");
                                event.mark_handled();
                            }
                            KeyCode::Space => {
                                info!("=== BACKEND STATUS ===");
                                info!("Current Backend: {}", self.current_backend);
                                info!("Switch Count: {}", self.backend_switch_count);
                                info!("Current FPS: {:.1}", self.current_fps);
                                info!("Switch Cooldown: {:.1}s", self.switch_cooldown.max(0.0));
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
        "Advanced Backend Demo - G:GLFW W:Wayland X:X11 R:Reset Space:Status ESC:Exit"
    }

    fn get_pending_backend_switch(&self) -> Option<String> {
        self.switch_pending.clone()
    }

    fn clear_pending_backend_switch(&mut self) {
        self.switch_pending = None;
    }

    fn on_backend_switch_completed(&mut self, old_backend: &str, new_backend: &str) {
        self.current_backend = new_backend.to_string();
        self.backend_switch_count += 1;
        
        // Clean up old OpenGL resources if they exist and we're not switching to a non-OpenGL backend
        let new_backend_supports_opengl = new_backend == "glfw" || new_backend == "x11"; // GLFW and X11 support OpenGL
        
        if self.opengl_available && new_backend_supports_opengl {
            unsafe {
                if self.vertex_array != 0 {
                    gl::DeleteVertexArrays(1, &self.vertex_array);
                }
                if self.vertex_buffer != 0 {
                    gl::DeleteBuffers(1, &self.vertex_buffer);
                }
                if self.shader_program != 0 {
                    gl::DeleteProgram(self.shader_program);
                }
            }
        }
        
        // Reset OpenGL objects
        self.vertex_array = 0;
        self.vertex_buffer = 0;
        self.shader_program = 0;
        self.opengl_available = false;
        
        // Re-initialize graphics after backend switch
        self.init();
        
        info!("✓ Backend switch completed: {} → {} (Total switches: {})", 
              old_backend, new_backend, self.backend_switch_count);
    }
}

impl AdvancedBackendDemo {
    fn get_backend_switch_count(&self) -> u32 {
        self.backend_switch_count
    }

    fn get_current_fps(&self) -> f32 {
        self.current_fps
    }

    fn check_opengl_availability(&self) -> bool {
        // Quick check: if we know the backend doesn't support OpenGL, return false immediately
        if self.current_backend == "wayland" {
            return false;
        }
        
        // Known OpenGL-supporting backends should have OpenGL available
        if self.current_backend == "glfw" || self.current_backend == "x11" {
            unsafe {
                // Clear any previous OpenGL errors
                while gl::GetError() != gl::NO_ERROR {}
                
                // Try to call a basic OpenGL function to see if we have a valid context
                let version = gl::GetString(gl::VERSION);
                
                if version.is_null() {
                    warn!("OpenGL version string is null for {} backend", self.current_backend);
                    return false;
                }
                
                // Get the version string for logging
                let version_str = std::ffi::CStr::from_ptr(version as *const i8).to_string_lossy();
                debug!("OpenGL version detected: {}", version_str);
                
                // Additional check: try to create a simple buffer
                let mut test_buffer = 0;
                gl::GenBuffers(1, &mut test_buffer);
                let error = gl::GetError();
                
                if test_buffer != 0 && error == gl::NO_ERROR {
                    gl::DeleteBuffers(1, &test_buffer);
                    debug!("OpenGL context validation successful for {} backend", self.current_backend);
                    return true;
                } else {
                    warn!("OpenGL buffer test failed for {} backend - error: 0x{:X}", self.current_backend, error);
                    return false;
                }
            }
        }
        
        // Unknown backend, assume no OpenGL support
        false
    }
}

fn main() {
    artifice_logging::init_from_env().expect("Failed to initialize logger");
    info!("Starting Advanced Backend Switching Demo");

    let app = AdvancedBackendDemo::new();

    // Configure comprehensive hot reload
    let hot_reload_config = HotReloadConfig {
        switch_timeout: Duration::from_secs(15),
        preserve_state: true,
        buffer_events: true,
        max_buffered_events: 2000,
        validate_backend: true,
    };

    // Enhanced metrics configuration
    let metrics_config = MetricsConfig {
        enabled: true,
        auto_reporting: true,
        report_interval: Duration::from_secs(10),
        max_event_types: 100,
    };

    // Create engine with initial backend
    let mut engine = Engine::with_config(app, "glfw", metrics_config, hot_reload_config);

    // Add info layer
    let info_layer = Box::new(BackendInfoLayer::new());
    engine.push_layer(info_layer);

    info!("=== ADVANCED BACKEND SWITCHING DEMO ===");
    info!("Controls:");
    info!("  G - Switch to GLFW backend (orange triangle, blue background)");
    info!("  W - Switch to Wayland backend (purple triangle, green background)");
    info!("  X - Switch to X11 backend (cyan triangle, brown background)");
    info!("  R - Reset animations");
    info!("  SPACE - Show status information");
    info!("  ESC - Exit application");
    info!("Features:");
    info!("  - Real-time backend switching with visual feedback");
    info!("  - Animation preservation during switches");
    info!("  - Performance monitoring and metrics");
    info!("  - Switch cooldown to prevent rapid switching");
    info!("==========================================");

    // OpenGL debug setup
    unsafe {
        if gl::DebugMessageCallback::is_loaded() {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        }
    }

    // Run the engine normally
    engine.run();

    info!("Advanced Backend Switching Demo completed successfully");
}

unsafe fn check_shader_compilation(shader: u32) {
    let mut success = 0;
    gl::GetShaderiv(shader, gl::COMPILE_STATUS, &mut success);
    if success == 0 {
        let mut log_length = 0;
        gl::GetShaderiv(shader, gl::INFO_LOG_LENGTH, &mut log_length);
        let mut log = Vec::with_capacity(log_length as usize);
        log.set_len(log_length as usize);
        gl::GetShaderInfoLog(shader, log_length, std::ptr::null_mut(), log.as_mut_ptr() as *mut i8);
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
        gl::GetProgramInfoLog(program, log_length, std::ptr::null_mut(), log.as_mut_ptr() as *mut i8);
        let log_str = std::str::from_utf8(&log).unwrap_or("Unknown error");
        error!("Program linking failed: {}", log_str);
    }
}