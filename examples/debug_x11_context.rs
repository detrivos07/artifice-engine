extern crate artifice_engine;
extern crate gl;
extern crate artifice_logging;

use std::ffi::CString;
use std::ptr;

use artifice_engine::events::{Event, EventType, KeyAction, KeyCode};
use artifice_engine::{Engine, Application};
use artifice_engine::window::HotReloadConfig;
use artifice_engine::io::{MetricsConfig, OpenGLWindow};
use artifice_logging::{info, warn, error, debug};

pub struct X11ContextDebugger {
    current_backend: String,
    switch_requested: Option<String>,
    context_test_results: Vec<String>,
    frame_count: u32,
    last_successful_render: bool,
}

impl Application for X11ContextDebugger {
    fn new() -> Self {
        X11ContextDebugger {
            current_backend: "glfw".to_string(),
            switch_requested: None,
            context_test_results: Vec::new(),
            frame_count: 0,
            last_successful_render: false,
        }
    }

    fn init(&mut self) {
        info!("🔧 DEBUG: Initializing OpenGL context for backend: {}", self.current_backend);
        self.context_test_results.clear();
        
        // Comprehensive OpenGL context testing
        self.test_opengl_context_comprehensive();
        self.test_basic_opengl_operations();
        self.test_minimal_rendering();
    }

    fn update(&mut self, _delta_time: f32) {
        self.frame_count += 1;
        
        // Auto-switch to X11 after 3 seconds for testing
        if self.frame_count == 180 && self.current_backend == "glfw" {
            info!("🔄 DEBUG: Auto-triggering X11 switch for context debugging");
            self.switch_requested = Some("x11".to_string());
        }
    }

    fn render(&mut self) {
        let render_success = self.test_frame_rendering();
        
        if render_success != self.last_successful_render {
            if render_success {
                info!("✅ DEBUG: Rendering restored for backend: {}", self.current_backend);
            } else {
                error!("❌ DEBUG: Rendering failed for backend: {}", self.current_backend);
            }
            self.last_successful_render = render_success;
        }
        
        // Log frame status periodically
        if self.frame_count % 120 == 0 {
            info!("📊 DEBUG: Backend={}, Frame={}, Render={}", 
                  self.current_backend, self.frame_count, 
                  if render_success { "OK" } else { "FAILED" });
        }
    }

    fn shutdown(&mut self) {
        info!("🔄 DEBUG: Shutting down context debugger");
    }

    fn event(&mut self, event: &mut Event) {
        if let EventType::Keyboard = event.event_type {
            if let Some(key_event) = event.as_key_event() {
                if key_event.action == KeyAction::Press {
                    match key_event.key {
                        KeyCode::X => {
                            info!("🎯 DEBUG: Manual X11 switch requested");
                            self.switch_requested = Some("x11".to_string());
                            event.mark_handled();
                        }
                        KeyCode::G => {
                            info!("🎯 DEBUG: Manual GLFW switch requested");
                            self.switch_requested = Some("glfw".to_string());
                            event.mark_handled();
                        }
                        KeyCode::T => {
                            info!("🔍 DEBUG: Running immediate context tests");
                            self.test_opengl_context_comprehensive();
                            self.print_test_results();
                            event.mark_handled();
                        }
                        KeyCode::R => {
                            info!("🎨 DEBUG: Testing immediate render");
                            self.test_immediate_render();
                            event.mark_handled();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn get_name(&self) -> &str {
        "🐛 X11 OpenGL Context Debugger - X=X11, G=GLFW, T=Test, R=Render"
    }

    fn get_pending_backend_switch(&self) -> Option<String> {
        self.switch_requested.clone()
    }

    fn clear_pending_backend_switch(&mut self) {
        self.switch_requested = None;
    }

    fn on_backend_switch_completed(&mut self, old_backend: &str, new_backend: &str) {
        info!("🚀 DEBUG: Backend switch completed: {} -> {}", old_backend, new_backend);
        self.current_backend = new_backend.to_string();
        
        // Comprehensive post-switch testing
        info!("🔍 DEBUG: Running post-switch context validation");
        self.test_context_after_switch();
        
        // Re-initialize with full testing
        self.init();
        
        // Test immediate rendering
        self.test_immediate_render();
        
        // Print all test results
        self.print_test_results();
    }
}

impl X11ContextDebugger {
    fn test_opengl_context_comprehensive(&mut self) {
        info!("🔍 DEBUG: Starting comprehensive OpenGL context test for {}", self.current_backend);
        
        unsafe {
            // Test 1: Basic context queries
            let mut result = String::new();
            
            let version = gl::GetString(gl::VERSION);
            if version.is_null() {
                result.push_str("❌ GetString(VERSION) returned NULL - context invalid\n");
                error!("❌ DEBUG: OpenGL VERSION query failed - context is not working");
            } else {
                let version_str = std::ffi::CStr::from_ptr(version as *const i8).to_string_lossy();
                result.push_str(&format!("✅ OpenGL Version: {}\n", version_str));
                info!("✅ DEBUG: OpenGL Version: {}", version_str);
            }
            
            let vendor = gl::GetString(gl::VENDOR);
            if !vendor.is_null() {
                let vendor_str = std::ffi::CStr::from_ptr(vendor as *const i8).to_string_lossy();
                result.push_str(&format!("✅ OpenGL Vendor: {}\n", vendor_str));
            }
            
            let renderer = gl::GetString(gl::RENDERER);
            if !renderer.is_null() {
                let renderer_str = std::ffi::CStr::from_ptr(renderer as *const i8).to_string_lossy();
                result.push_str(&format!("✅ OpenGL Renderer: {}\n", renderer_str));
            }
            
            // Test 2: Error state
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                result.push_str(&format!("⚠️ OpenGL Error State: 0x{:X}\n", error));
                error!("⚠️ DEBUG: OpenGL error state: 0x{:X}", error);
            } else {
                result.push_str("✅ OpenGL Error State: Clean\n");
            }
            
            // Test 3: Basic parameters
            let mut viewport = [0i32; 4];
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
            result.push_str(&format!("✅ Viewport: [{}, {}, {}, {}]\n", 
                viewport[0], viewport[1], viewport[2], viewport[3]));
            
            let mut max_texture_size = 0i32;
            gl::GetIntegerv(gl::MAX_TEXTURE_SIZE, &mut max_texture_size);
            result.push_str(&format!("✅ Max Texture Size: {}\n", max_texture_size));
            
            // Test 4: Context capabilities
            let mut major = 0i32;
            let mut minor = 0i32;
            gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
            gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
            result.push_str(&format!("✅ OpenGL Version Numbers: {}.{}\n", major, minor));
            
            self.context_test_results.push(result);
        }
    }
    
    fn test_basic_opengl_operations(&mut self) {
        info!("🔍 DEBUG: Testing basic OpenGL operations for {}", self.current_backend);
        
        let mut result = String::new();
        
        unsafe {
            // Test basic state changes
            gl::ClearColor(1.0, 0.0, 0.0, 1.0);
            let error1 = gl::GetError();
            if error1 != gl::NO_ERROR {
                result.push_str(&format!("❌ glClearColor failed: 0x{:X}\n", error1));
            } else {
                result.push_str("✅ glClearColor: OK\n");
            }
            
            gl::Clear(gl::COLOR_BUFFER_BIT);
            let error2 = gl::GetError();
            if error2 != gl::NO_ERROR {
                result.push_str(&format!("❌ glClear failed: 0x{:X}\n", error2));
            } else {
                result.push_str("✅ glClear: OK\n");
            }
            
            // Test buffer operations
            let mut buffer = 0u32;
            gl::GenBuffers(1, &mut buffer);
            let error3 = gl::GetError();
            if error3 != gl::NO_ERROR {
                result.push_str(&format!("❌ glGenBuffers failed: 0x{:X}\n", error3));
            } else {
                result.push_str(&format!("✅ glGenBuffers: OK (buffer={})\n", buffer));
                
                gl::BindBuffer(gl::ARRAY_BUFFER, buffer);
                let error4 = gl::GetError();
                if error4 != gl::NO_ERROR {
                    result.push_str(&format!("❌ glBindBuffer failed: 0x{:X}\n", error4));
                } else {
                    result.push_str("✅ glBindBuffer: OK\n");
                }
                
                gl::DeleteBuffers(1, &buffer);
            }
            
            // Test VAO operations
            let mut vao = 0u32;
            gl::GenVertexArrays(1, &mut vao);
            let error5 = gl::GetError();
            if error5 != gl::NO_ERROR {
                result.push_str(&format!("❌ glGenVertexArrays failed: 0x{:X}\n", error5));
            } else {
                result.push_str(&format!("✅ glGenVertexArrays: OK (vao={})\n", vao));
                
                gl::BindVertexArray(vao);
                let error6 = gl::GetError();
                if error6 != gl::NO_ERROR {
                    result.push_str(&format!("❌ glBindVertexArray failed: 0x{:X}\n", error6));
                } else {
                    result.push_str("✅ glBindVertexArray: OK\n");
                }
                
                gl::DeleteVertexArrays(1, &vao);
            }
        }
        
        self.context_test_results.push(result);
    }
    
    fn test_minimal_rendering(&mut self) {
        info!("🔍 DEBUG: Testing minimal rendering for {}", self.current_backend);
        
        let mut result = String::new();
        
        unsafe {
            // Test shader creation
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let error1 = gl::GetError();
            if error1 != gl::NO_ERROR {
                result.push_str(&format!("❌ glCreateShader(vertex) failed: 0x{:X}\n", error1));
                self.context_test_results.push(result);
                return;
            }
            
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let error2 = gl::GetError();
            if error2 != gl::NO_ERROR {
                result.push_str(&format!("❌ glCreateShader(fragment) failed: 0x{:X}\n", error2));
                self.context_test_results.push(result);
                return;
            }
            
            result.push_str(&format!("✅ Shader creation: vertex={}, fragment={}\n", vertex_shader, fragment_shader));
            
            // Test simple vertex shader
            let vertex_source = CString::new(
                "#version 330 core\n\
                layout (location = 0) in vec3 aPos;\n\
                void main() {\n\
                    gl_Position = vec4(aPos, 1.0);\n\
                }"
            ).unwrap();
            
            gl::ShaderSource(vertex_shader, 1, &vertex_source.as_ptr(), ptr::null());
            gl::CompileShader(vertex_shader);
            
            let mut success = 0i32;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                result.push_str("❌ Vertex shader compilation failed\n");
            } else {
                result.push_str("✅ Vertex shader compilation: OK\n");
            }
            
            // Test simple fragment shader
            let fragment_source = CString::new(
                "#version 330 core\n\
                out vec4 FragColor;\n\
                void main() {\n\
                    FragColor = vec4(1.0, 0.0, 1.0, 1.0);\n\
                }"
            ).unwrap();
            
            gl::ShaderSource(fragment_shader, 1, &fragment_source.as_ptr(), ptr::null());
            gl::CompileShader(fragment_shader);
            
            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                result.push_str("❌ Fragment shader compilation failed\n");
            } else {
                result.push_str("✅ Fragment shader compilation: OK\n");
            }
            
            // Test program creation and linking
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);
            
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                result.push_str("❌ Program linking failed\n");
            } else {
                result.push_str(&format!("✅ Program linking: OK (program={})\n", program));
            }
            
            // Cleanup
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            gl::DeleteProgram(program);
        }
        
        self.context_test_results.push(result);
    }
    
    fn test_frame_rendering(&self) -> bool {
        unsafe {
            // Clear with backend-specific color
            match self.current_backend.as_str() {
                "glfw" => gl::ClearColor(0.0, 0.5, 0.0, 1.0), // Green for GLFW
                "x11" => gl::ClearColor(0.5, 0.0, 0.0, 1.0),  // Red for X11
                _ => gl::ClearColor(0.0, 0.0, 0.5, 1.0),      // Blue for unknown
            }
            
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                if self.frame_count % 60 == 0 {
                    error!("❌ DEBUG: Frame render error for {}: 0x{:X}", self.current_backend, error);
                }
                return false;
            }
            
            true
        }
    }
    
    fn test_context_after_switch(&mut self) {
        info!("🔍 DEBUG: Testing context state after switch to {}", self.current_backend);
        
        let mut result = String::new();
        result.push_str(&format!("=== Context Test After Switch to {} ===\n", self.current_backend));
        
        unsafe {
            // Check if context is actually current
            let error_before = gl::GetError();
            result.push_str(&format!("Error state before tests: 0x{:X}\n", error_before));
            
            // Test immediate context validity
            let version = gl::GetString(gl::VERSION);
            if version.is_null() {
                result.push_str("❌ CRITICAL: Context appears to be invalid after switch\n");
                error!("❌ DEBUG: Context is invalid after switch to {}", self.current_backend);
            } else {
                let version_str = std::ffi::CStr::from_ptr(version as *const i8).to_string_lossy();
                result.push_str(&format!("✅ Context valid, version: {}\n", version_str));
            }
            
            // Test basic rendering capability
            gl::ClearColor(1.0, 1.0, 0.0, 1.0); // Yellow test
            gl::Clear(gl::COLOR_BUFFER_BIT);
            let render_error = gl::GetError();
            if render_error != gl::NO_ERROR {
                result.push_str(&format!("❌ Basic render test failed: 0x{:X}\n", render_error));
            } else {
                result.push_str("✅ Basic render test passed\n");
            }
        }
        
        self.context_test_results.push(result);
    }
    
    fn test_immediate_render(&self) {
        info!("🎨 DEBUG: Testing immediate render for {}", self.current_backend);
        
        unsafe {
            // Test with distinctive colors
            match self.current_backend.as_str() {
                "glfw" => {
                    gl::ClearColor(0.0, 1.0, 0.0, 1.0); // Bright green
                    info!("🎨 DEBUG: GLFW - Should see BRIGHT GREEN");
                }
                "x11" => {
                    gl::ClearColor(1.0, 0.0, 0.0, 1.0); // Bright red  
                    info!("🎨 DEBUG: X11 - Should see BRIGHT RED");
                }
                _ => {
                    gl::ClearColor(1.0, 0.0, 1.0, 1.0); // Bright magenta
                    info!("🎨 DEBUG: Unknown - Should see BRIGHT MAGENTA");
                }
            }
            
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                error!("❌ DEBUG: Immediate render failed with error: 0x{:X}", error);
            } else {
                info!("✅ DEBUG: Immediate render successful for {}", self.current_backend);
            }
        }
    }
    
    fn print_test_results(&self) {
        info!("📋 DEBUG: ========== TEST RESULTS ==========");
        for (i, result) in self.context_test_results.iter().enumerate() {
            info!("📋 DEBUG: Test Suite {}:\n{}", i + 1, result);
        }
        info!("📋 DEBUG: ===================================");
    }
}

fn main() {
    artifice_logging::init_from_env().expect("Failed to initialize logger");
    
    info!("🐛 === X11 OpenGL Context Debugger ===");
    info!("This tool will comprehensively test OpenGL context functionality");
    info!("🎮 Controls:");
    info!("  X = Switch to X11 (watch for black screen issue)");
    info!("  G = Switch to GLFW");
    info!("  T = Run immediate context tests");
    info!("  R = Test immediate rendering");
    info!("🔍 Expected behavior:");
    info!("  GLFW: GREEN background");
    info!("  X11:  RED background");
    info!("  Black screen = CONTEXT ISSUE");
    info!("=====================================");

    let app = X11ContextDebugger::new();

    let hot_reload_config = HotReloadConfig {
        switch_timeout: std::time::Duration::from_secs(10),
        preserve_state: true,
        buffer_events: true,
        max_buffered_events: 1000,
        validate_backend: true,
    };

    let metrics_config = MetricsConfig {
        enabled: false, // Disable to reduce log noise
        auto_reporting: false,
        report_interval: std::time::Duration::from_secs(60),
        max_event_types: 100,
    };

    let mut engine = Engine::with_config(app, "glfw", metrics_config, hot_reload_config);
    
    info!("🚀 Starting context debugger...");
    info!("🕐 Auto-switch to X11 will happen in 3 seconds");
    
    engine.run();
    
    info!("🏁 Context debugging completed");
}