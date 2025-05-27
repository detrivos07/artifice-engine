extern crate artifice_engine;
extern crate gl;
extern crate artifice_logging;

use std::ffi::CString;
use std::ptr;

use artifice_engine::events::{Event, EventType, KeyAction, KeyCode};
use artifice_engine::{Engine, Application};
use artifice_engine::window::HotReloadConfig;
use artifice_engine::io::MetricsConfig;
use artifice_logging::{info, warn, error};

pub struct BlackScreenDebugger {
    current_backend: String,
    switch_requested: Option<String>,
    opengl_objects: OpenGLObjects,
    test_stage: TestStage,
    frame_count: u32,
}

#[derive(Debug, Clone, Copy)]
enum TestStage {
    Initial,
    PreSwitch,
    PostSwitch,
    Testing,
}

struct OpenGLObjects {
    vao: u32,
    vbo: u32,
    program: u32,
    created_successfully: bool,
}

impl OpenGLObjects {
    fn new() -> Self {
        Self {
            vao: 0,
            vbo: 0,
            program: 0,
            created_successfully: false,
        }
    }

    fn cleanup(&mut self) {
        unsafe {
            if self.vao != 0 {
                info!("🧹 Cleaning up VAO: {}", self.vao);
                gl::DeleteVertexArrays(1, &self.vao);
                self.vao = 0;
            }
            if self.vbo != 0 {
                info!("🧹 Cleaning up VBO: {}", self.vbo);
                gl::DeleteBuffers(1, &self.vbo);
                self.vbo = 0;
            }
            if self.program != 0 {
                info!("🧹 Cleaning up Program: {}", self.program);
                gl::DeleteProgram(self.program);
                self.program = 0;
            }
        }
        self.created_successfully = false;
    }

    fn is_valid(&self) -> bool {
        self.vao != 0 && self.vbo != 0 && self.program != 0 && self.created_successfully
    }
}

impl Application for BlackScreenDebugger {
    fn new() -> Self {
        BlackScreenDebugger {
            current_backend: "glfw".to_string(),
            switch_requested: None,
            opengl_objects: OpenGLObjects::new(),
            test_stage: TestStage::Initial,
            frame_count: 0,
        }
    }

    fn init(&mut self) {
        info!("🔧 BLACK SCREEN DEBUG: Initializing for backend: {}", self.current_backend);
        
        // Clean up any existing objects first
        self.opengl_objects.cleanup();
        
        // Verify context is working
        self.verify_context_basic();
        
        // Create new OpenGL objects
        self.create_opengl_objects();
        
        // Test immediate rendering with these objects
        self.test_objects_immediately();
    }

    fn update(&mut self, _delta_time: f32) {
        self.frame_count += 1;
        
        // Auto-trigger X11 switch after 180 frames (3 seconds at 60fps)
        if self.frame_count == 180 && matches!(self.test_stage, TestStage::Initial) {
            info!("🔄 BLACK SCREEN DEBUG: Auto-triggering X11 switch");
            self.test_stage = TestStage::PreSwitch;
            self.switch_requested = Some("x11".to_string());
        }
    }

    fn render(&mut self) {
        // Test each render step individually
        let clear_success = self.test_clear();
        let objects_valid = self.opengl_objects.is_valid();
        let render_success = if objects_valid {
            self.test_triangle_render()
        } else {
            false
        };
        
        // Log status every 60 frames but only when significant
        if self.frame_count % 60 == 0 {
            info!("🎨 RENDER STATUS: Backend={}, Clear={}, Objects={}, Render={}", 
                  self.current_backend, clear_success, objects_valid, render_success);
            
            if !clear_success {
                error!("❌ BLACK SCREEN: Clear operation failed!");
            }
            if !objects_valid {
                error!("❌ BLACK SCREEN: OpenGL objects invalid!");
            }
            if objects_valid && !render_success {
                error!("❌ BLACK SCREEN: Triangle render failed despite valid objects!");
            }
        }
    }

    fn shutdown(&mut self) {
        info!("🔄 BLACK SCREEN DEBUG: Shutting down");
        self.opengl_objects.cleanup();
    }

    fn event(&mut self, event: &mut Event) {
        if let EventType::Keyboard = event.event_type {
            if let Some(key_event) = event.as_key_event() {
                if key_event.action == KeyAction::Press {
                    match key_event.key {
                        KeyCode::X => {
                            info!("🎯 Manual X11 switch requested");
                            self.test_stage = TestStage::PreSwitch;
                            self.switch_requested = Some("x11".to_string());
                            event.mark_handled();
                        }
                        KeyCode::G => {
                            info!("🎯 Manual GLFW switch requested");
                            self.switch_requested = Some("glfw".to_string());
                            event.mark_handled();
                        }
                        KeyCode::T => {
                            info!("🔍 Running immediate tests");
                            self.run_immediate_tests();
                            event.mark_handled();
                        }
                        KeyCode::R => {
                            info!("🔧 Recreating OpenGL objects");
                            self.init();
                            event.mark_handled();
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    fn get_name(&self) -> &str {
        "🖤 Black Screen Debugger - X=X11, G=GLFW, T=Test, R=Recreate"
    }

    fn get_pending_backend_switch(&self) -> Option<String> {
        self.switch_requested.clone()
    }

    fn clear_pending_backend_switch(&mut self) {
        self.switch_requested = None;
    }

    fn on_backend_switch_completed(&mut self, old_backend: &str, new_backend: &str) {
        info!("🚀 BLACK SCREEN DEBUG: Backend switch completed: {} -> {}", old_backend, new_backend);
        self.current_backend = new_backend.to_string();
        self.test_stage = TestStage::PostSwitch;
        
        info!("🔍 Testing context immediately after switch...");
        self.verify_context_basic();
        
        info!("🔧 Re-initializing OpenGL objects...");
        self.init();
        
        info!("🎨 Testing immediate render after initialization...");
        self.test_immediate_full_render();
        
        self.test_stage = TestStage::Testing;
        info!("✅ Post-switch initialization complete");
    }
}

impl BlackScreenDebugger {
    fn verify_context_basic(&self) {
        info!("🔍 Verifying basic OpenGL context for {}", self.current_backend);
        
        unsafe {
            // Check error state first
            let initial_error = gl::GetError();
            if initial_error != gl::NO_ERROR {
                error!("❌ OpenGL error state before tests: 0x{:X}", initial_error);
            }
            
            // Test version query
            let version = gl::GetString(gl::VERSION);
            if version.is_null() {
                error!("❌ CRITICAL: OpenGL context invalid - GetString(VERSION) failed");
                return;
            }
            
            let version_str = std::ffi::CStr::from_ptr(version as *const i8).to_string_lossy();
            info!("✅ OpenGL Version: {}", version_str);
            
            // Test viewport
            let mut viewport = [0i32; 4];
            gl::GetIntegerv(gl::VIEWPORT, viewport.as_mut_ptr());
            info!("✅ Viewport: [{}, {}, {}, {}]", viewport[0], viewport[1], viewport[2], viewport[3]);
            
            if viewport[2] == 0 || viewport[3] == 0 {
                error!("❌ Invalid viewport size: {}x{}", viewport[2], viewport[3]);
            }
        }
    }
    
    fn create_opengl_objects(&mut self) {
        info!("🔧 Creating OpenGL objects for {}", self.current_backend);
        
        let vertices: [f32; 9] = [
             0.0,  0.5, 0.0,   // top
            -0.5, -0.5, 0.0,   // bottom left  
             0.5, -0.5, 0.0,   // bottom right
        ];
        
        unsafe {
            // Create VAO
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            if vao == 0 {
                error!("❌ Failed to generate VAO");
                return;
            }
            gl::BindVertexArray(vao);
            info!("✅ Created VAO: {}", vao);
            
            // Create VBO
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);
            if vbo == 0 {
                error!("❌ Failed to generate VBO");
                return;
            }
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                vertices.as_ptr() as *const _,
                gl::STATIC_DRAW,
            );
            info!("✅ Created VBO: {} with {} vertices", vbo, vertices.len() / 3);
            
            // Set up vertex attributes
            gl::VertexAttribPointer(0, 3, gl::FLOAT, gl::FALSE, 0, ptr::null());
            gl::EnableVertexAttribArray(0);
            
            // Create shaders
            let program = self.create_shader_program();
            if program == 0 {
                error!("❌ Failed to create shader program");
                return;
            }
            
            self.opengl_objects.vao = vao;
            self.opengl_objects.vbo = vbo;
            self.opengl_objects.program = program;
            self.opengl_objects.created_successfully = true;
            
            info!("✅ All OpenGL objects created successfully");
            
            // Test using the objects immediately
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                error!("❌ OpenGL error after object creation: 0x{:X}", error);
                self.opengl_objects.created_successfully = false;
            }
        }
    }
    
    fn create_shader_program(&self) -> u32 {
        info!("🔧 Creating shader program for {}", self.current_backend);
        
        unsafe {
            // Vertex shader
            let vertex_shader = gl::CreateShader(gl::VERTEX_SHADER);
            let vertex_source = CString::new(
                "#version 330 core\n\
                layout (location = 0) in vec3 aPos;\n\
                void main() {\n\
                    gl_Position = vec4(aPos, 1.0);\n\
                }"
            ).unwrap();
            gl::ShaderSource(vertex_shader, 1, &vertex_source.as_ptr(), ptr::null());
            gl::CompileShader(vertex_shader);
            
            let mut success = 0;
            gl::GetShaderiv(vertex_shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                error!("❌ Vertex shader compilation failed");
                return 0;
            }
            info!("✅ Vertex shader compiled");
            
            // Fragment shader with backend-specific color
            let frag_color = match self.current_backend.as_str() {
                "glfw" => "vec4(0.0, 1.0, 0.0, 1.0)", // Green for GLFW
                "x11" => "vec4(1.0, 0.0, 0.0, 1.0)",  // Red for X11
                _ => "vec4(1.0, 0.0, 1.0, 1.0)",      // Magenta for unknown
            };
            
            let fragment_shader = gl::CreateShader(gl::FRAGMENT_SHADER);
            let fragment_source = CString::new(format!(
                "#version 330 core\n\
                out vec4 FragColor;\n\
                void main() {{\n\
                    FragColor = {};\n\
                }}", frag_color
            )).unwrap();
            gl::ShaderSource(fragment_shader, 1, &fragment_source.as_ptr(), ptr::null());
            gl::CompileShader(fragment_shader);
            
            gl::GetShaderiv(fragment_shader, gl::COMPILE_STATUS, &mut success);
            if success == 0 {
                error!("❌ Fragment shader compilation failed");
                return 0;
            }
            info!("✅ Fragment shader compiled with {} color", self.current_backend);
            
            // Link program
            let program = gl::CreateProgram();
            gl::AttachShader(program, vertex_shader);
            gl::AttachShader(program, fragment_shader);
            gl::LinkProgram(program);
            
            gl::GetProgramiv(program, gl::LINK_STATUS, &mut success);
            if success == 0 {
                error!("❌ Shader program linking failed");
                return 0;
            }
            
            gl::DeleteShader(vertex_shader);
            gl::DeleteShader(fragment_shader);
            
            info!("✅ Shader program created: {}", program);
            program
        }
    }
    
    fn test_clear(&self) -> bool {
        unsafe {
            // Set clear color based on backend
            match self.current_backend.as_str() {
                "glfw" => gl::ClearColor(0.0, 0.2, 0.0, 1.0), // Dark green for GLFW
                "x11" => gl::ClearColor(0.2, 0.0, 0.0, 1.0),  // Dark red for X11
                _ => gl::ClearColor(0.0, 0.0, 0.2, 1.0),      // Dark blue for unknown
            }
            
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                if self.frame_count % 60 == 0 {
                    error!("❌ Clear failed with error: 0x{:X}", error);
                }
                false
            } else {
                true
            }
        }
    }
    
    fn test_triangle_render(&self) -> bool {
        if !self.opengl_objects.is_valid() {
            return false;
        }
        
        unsafe {
            gl::UseProgram(self.opengl_objects.program);
            let error1 = gl::GetError();
            if error1 != gl::NO_ERROR {
                if self.frame_count % 60 == 0 {
                    error!("❌ UseProgram failed: 0x{:X}", error1);
                }
                return false;
            }
            
            gl::BindVertexArray(self.opengl_objects.vao);
            let error2 = gl::GetError();
            if error2 != gl::NO_ERROR {
                if self.frame_count % 60 == 0 {
                    error!("❌ BindVertexArray failed: 0x{:X}", error2);
                }
                return false;
            }
            
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            let error3 = gl::GetError();
            if error3 != gl::NO_ERROR {
                if self.frame_count % 60 == 0 {
                    error!("❌ DrawArrays failed: 0x{:X}", error3);
                }
                return false;
            }
            
            true
        }
    }
    
    fn test_objects_immediately(&self) {
        info!("🎨 Testing OpenGL objects immediately after creation");
        
        if !self.opengl_objects.is_valid() {
            error!("❌ Objects not valid for immediate test");
            return;
        }
        
        unsafe {
            // Clear to distinctive color
            gl::ClearColor(1.0, 1.0, 0.0, 1.0); // Yellow
            gl::Clear(gl::COLOR_BUFFER_BIT);
            
            // Try to render triangle
            gl::UseProgram(self.opengl_objects.program);
            gl::BindVertexArray(self.opengl_objects.vao);
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            
            let error = gl::GetError();
            if error != gl::NO_ERROR {
                error!("❌ Immediate object test failed: 0x{:X}", error);
            } else {
                info!("✅ Immediate object test passed");
            }
        }
    }
    
    fn test_immediate_full_render(&self) {
        info!("🎨 Testing full immediate render sequence");
        
        unsafe {
            // Test 1: Clear to bright magenta
            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
            info!("✅ Step 1: Clear to magenta");
            
            if !self.opengl_objects.is_valid() {
                error!("❌ Step 2: Objects not valid");
                return;
            }
            
            // Test 2: Use shader program
            gl::UseProgram(self.opengl_objects.program);
            let error1 = gl::GetError();
            if error1 != gl::NO_ERROR {
                error!("❌ Step 2: UseProgram failed: 0x{:X}", error1);
                return;
            }
            info!("✅ Step 2: Shader program bound");
            
            // Test 3: Bind VAO
            gl::BindVertexArray(self.opengl_objects.vao);
            let error2 = gl::GetError();
            if error2 != gl::NO_ERROR {
                error!("❌ Step 3: BindVertexArray failed: 0x{:X}", error2);
                return;
            }
            info!("✅ Step 3: VAO bound");
            
            // Test 4: Draw triangle
            gl::DrawArrays(gl::TRIANGLES, 0, 3);
            let error3 = gl::GetError();
            if error3 != gl::NO_ERROR {
                error!("❌ Step 4: DrawArrays failed: 0x{:X}", error3);
                return;
            }
            info!("✅ Step 4: Triangle drawn successfully");
            
            info!("🎉 Full render sequence completed successfully!");
        }
    }
    
    fn run_immediate_tests(&self) {
        info!("🔍 Running comprehensive immediate tests");
        self.verify_context_basic();
        
        if self.opengl_objects.is_valid() {
            self.test_immediate_full_render();
        } else {
            error!("❌ Cannot test rendering - objects invalid");
            info!("🔧 Attempting to recreate objects...");
        }
    }
}

fn main() {
    artifice_logging::init_from_env().expect("Failed to initialize logger");
    
    info!("🖤 === BLACK SCREEN DEBUGGER ===");
    info!("This tool will identify why the screen goes black after X11 switch");
    info!("🎮 Controls:");
    info!("  X = Switch to X11 (test for black screen)");
    info!("  G = Switch to GLFW");  
    info!("  T = Run immediate tests");
    info!("  R = Recreate OpenGL objects");
    info!("🔍 Expected colors:");
    info!("  GLFW: Dark green background + Green triangle");
    info!("  X11:  Dark red background + Red triangle");
    info!("  Black screen = PROBLEM DETECTED");
    info!("=====================================");

    let app = BlackScreenDebugger::new();

    let hot_reload_config = HotReloadConfig {
        switch_timeout: std::time::Duration::from_secs(10),
        preserve_state: true,
        buffer_events: true,
        max_buffered_events: 1000,
        validate_backend: true,
    };

    let metrics_config = MetricsConfig {
        enabled: false,
        auto_reporting: false,
        report_interval: std::time::Duration::from_secs(60),
        max_event_types: 100,
    };

    let mut engine = Engine::with_config(app, "glfw", metrics_config, hot_reload_config);
    
    info!("🚀 Starting black screen debugger...");
    info!("🕐 Auto-switch to X11 will happen in 3 seconds");
    
    engine.run();
    
    info!("🏁 Black screen debugging completed");
}