#[cfg(feature = "x11")]
extern crate artifice_engine;
#[cfg(feature = "x11")]
extern crate artifice_logging;

#[cfg(feature = "x11")]
use artifice_engine::window::x11::{X11Window, X11WindowFactory};
#[cfg(feature = "x11")]
use artifice_engine::window::factory::WindowFactory;
#[cfg(feature = "x11")]
use artifice_engine::io::Window;
#[cfg(feature = "x11")]
use artifice_logging::{info, error, warn};

#[cfg(feature = "x11")]
fn main() {
    // Initialize logging
    artifice_logging::init_from_env().expect("Failed to initialize logger");
    info!("Starting X11 Test");

    // Check if we're running under X11
    match std::env::var("DISPLAY") {
        Ok(display) => info!("X11 display detected: {}", display),
        Err(_) => {
            warn!("DISPLAY not set - may not be running under X11");
            info!("XDG_SESSION_TYPE: {:?}", std::env::var("XDG_SESSION_TYPE"));
            info!("SESSION_TYPE: {:?}", std::env::var("SESSION_TYPE"));
        }
    }

    info!("Attempting to create X11 window...");

    // Try to create an X11 window directly
    match std::panic::catch_unwind(|| {
        let mut window = X11Window::with_hints(
            800, 
            600, 
            "X11 Test Window", 
            &[]
        );
        info!("✓ X11 window created successfully!");
        
        // Try basic operations
        info!("Window size: {:?}", window.size());
        info!("Window position: {:?}", window.position());
        info!("Window title: {}", window.title());
        
        // Try to change window properties
        window.set_title("X11 Test - Modified Title");
        info!("✓ Window title changed to: {}", window.title());
        
        // Test OpenGL context
        use artifice_engine::io::OpenGLWindow;
        info!("Testing OpenGL context...");
        info!("OpenGL context is current: {}", window.is_current());
        window.make_current();
        info!("✓ OpenGL context made current");
        
        // Try to process events
        info!("Testing event processing...");
        
        // Simple event loop for a few seconds
        let start = std::time::Instant::now();
        let mut should_close = false;
        
        info!("Running event loop for 5 seconds...");
        info!("Try moving the mouse, pressing keys, or clicking in the window!");
        
        while start.elapsed().as_secs() < 5 && !should_close {
            window.process_events();
            should_close = window.should_close();
            
            // Test rendering - clear to blue
            unsafe {
                gl::ClearColor(0.2, 0.4, 0.8, 1.0);
                gl::Clear(gl::COLOR_BUFFER_BIT);
            }
            window.swap_buffers();
            
            if start.elapsed().as_millis() % 1000 < 50 {
                info!("Window still alive after {:.1}s", start.elapsed().as_secs_f32());
            }
            
            std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps
        }
        
        if should_close {
            info!("✓ Window close event detected!");
        } else {
            info!("✓ X11 window test completed successfully!");
        }
        
        window
    }) {
        Ok(_) => info!("X11 backend appears to be working!"),
        Err(panic_info) => {
            error!("X11 window creation failed!");
            if let Some(s) = panic_info.downcast_ref::<String>() {
                error!("Panic message: {}", s);
            } else if let Some(s) = panic_info.downcast_ref::<&str>() {
                error!("Panic message: {}", s);
            } else {
                error!("Panic occurred but no message available");
            }
        }
    }

    // Also test the factory
    info!("Testing X11WindowFactory...");
    match std::panic::catch_unwind(|| {
        let factory = X11WindowFactory {};
        info!("Factory backend name: {}", factory.backend_name());
        info!("Factory backend version: {}", factory.backend_version().unwrap_or_else(|| "Unknown".to_string()));
        
        // Test supported features
        use artifice_engine::window::factory::WindowFeature;
        info!("Supports OpenGL: {}", factory.supports_feature(WindowFeature::OpenGL));
        info!("Supports Vulkan: {}", factory.supports_feature(WindowFeature::Vulkan));
        info!("Supports MultiWindow: {}", factory.supports_feature(WindowFeature::MultiWindow));
        info!("Supports HighDPI: {}", factory.supports_feature(WindowFeature::HighDPI));
        
        // Try creating a window through the factory
        info!("Creating window through factory...");
        let mut factory_window = factory.create_window(400, 300, "Factory Created X11 Window");
        info!("✓ Factory window created successfully!");
        
        // Brief test of factory window
        let start = std::time::Instant::now();
        while start.elapsed().as_secs() < 2 && !factory_window.should_close() {
            factory_window.process_events();
            std::thread::sleep(std::time::Duration::from_millis(16));
        }
        
        info!("✓ X11WindowFactory test completed!");
    }) {
        Ok(_) => info!("X11WindowFactory works correctly"),
        Err(e) => {
            error!("X11WindowFactory failed!");
            if let Some(s) = e.downcast_ref::<String>() {
                error!("Error: {}", s);
            }
        }
    }

    info!("X11 test completed");
}

#[cfg(not(feature = "x11"))]
fn main() {
    println!("X11 test example requires the 'x11' feature to be enabled.");
    println!("Run with: cargo run --example x11_test --features x11");
}