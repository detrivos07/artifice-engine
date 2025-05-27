#[cfg(feature = "wayland")]
extern crate artifice_engine;
#[cfg(feature = "wayland")]
extern crate artifice_logging;

#[cfg(feature = "wayland")]
use artifice_engine::window::wayland::{WaylandWindow, WaylandWindowFactory};
#[cfg(feature = "wayland")]
use artifice_engine::window::factory::WindowFactory;
#[cfg(feature = "wayland")]
use artifice_engine::io::Window;
#[cfg(feature = "wayland")]
use artifice_logging::{info, error, warn};

#[cfg(feature = "wayland")]
fn main() {
    // Initialize logging
    artifice_logging::init_from_env().expect("Failed to initialize logger");
    info!("Starting Wayland Test");

    // Check if we're running under Wayland
    match std::env::var("WAYLAND_DISPLAY") {
        Ok(display) => info!("Wayland display detected: {}", display),
        Err(_) => {
            warn!("WAYLAND_DISPLAY not set - may not be running under Wayland");
            info!("XDG_SESSION_TYPE: {:?}", std::env::var("XDG_SESSION_TYPE"));
            info!("SESSION_TYPE: {:?}", std::env::var("SESSION_TYPE"));
        }
    }

    info!("Attempting to create Wayland window...");

    // Try to create a Wayland window directly
    match std::panic::catch_unwind(|| {
        let mut window = WaylandWindow::with_hints(
            800, 
            600, 
            "Wayland Test Window", 
            &[]
        );
        info!("✓ Wayland window created successfully!");
        
        // Try basic operations
        info!("Window size: {:?}", window.size());
        info!("Window position: {:?}", window.position());
        info!("Window title: {}", window.title());
        
        // Try to process events (this might fail)
        info!("Testing event processing...");
        
        // Simple event loop for a few seconds
        let start = std::time::Instant::now();
        let mut should_close = false;
        
        while start.elapsed().as_secs() < 5 && !should_close {
            window.process_events();
            should_close = window.should_close();
            
            if start.elapsed().as_millis() % 1000 < 50 {
                info!("Window still alive after {:.1}s", start.elapsed().as_secs_f32());
            }
            
            std::thread::sleep(std::time::Duration::from_millis(16)); // ~60fps
        }
        
        info!("✓ Wayland window test completed successfully!");
        window
    }) {
        Ok(_) => info!("Wayland backend appears to be working!"),
        Err(panic_info) => {
            error!("Wayland window creation failed!");
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
    info!("Testing WaylandWindowFactory...");
    match std::panic::catch_unwind(|| {
        let factory = WaylandWindowFactory {};
        info!("Factory backend name: {}", factory.backend_name());
        info!("Factory backend version: {}", factory.backend_version().unwrap_or_else(|| "Unknown".to_string()));
        
        // Test supported features
        use artifice_engine::window::factory::WindowFeature;
        info!("Supports OpenGL: {}", factory.supports_feature(WindowFeature::OpenGL));
        info!("Supports Vulkan: {}", factory.supports_feature(WindowFeature::Vulkan));
        
        info!("✓ WaylandWindowFactory test completed!");
    }) {
        Ok(_) => info!("WaylandWindowFactory works correctly"),
        Err(e) => {
            error!("WaylandWindowFactory failed!");
            if let Some(s) = e.downcast_ref::<String>() {
                error!("Error: {}", s);
            }
        }
    }

    info!("Wayland test completed");
}

#[cfg(not(feature = "wayland"))]
fn main() {
    println!("Wayland test example requires the 'wayland' feature to be enabled.");
    println!("Run with: cargo run --example wayland_test --features wayland");
}