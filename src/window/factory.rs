use crate::io::{Window, WindowHint};
#[cfg(feature = "wayland")]
use crate::window::wayland::WaylandWindowFactory;
use std::collections::HashMap;
use artifice_logging::{debug, info, warn};

/// Trait for creating windows with different backends
pub trait WindowFactory: Send + Sync {
    /// Create a basic window with default settings
    fn create_window(&self, width: u32, height: u32, title: &str) -> Box<dyn Window>;
    
    /// Create a window with specific hints/configuration
    fn create_window_with_hints(&self, width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Box<dyn Window>;
    
    /// Check if this factory supports a specific feature
    fn supports_feature(&self, feature: WindowFeature) -> bool;
    
    /// Get the name of this window backend
    fn backend_name(&self) -> &str;
    
    /// Get version information for this backend
    fn backend_version(&self) -> Option<String> {
        None
    }
}

/// Features that window backends might support
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WindowFeature {
    /// OpenGL rendering context support
    OpenGL,
    /// Vulkan rendering context support
    Vulkan,
    /// DirectX rendering context support (Windows only)
    DirectX,
    /// Multiple window support
    MultiWindow,
    /// High DPI/scaling support
    HighDPI,
    /// Fullscreen support
    Fullscreen,
    /// Window transparency support
    Transparency,
    /// Custom cursor support
    CustomCursor,
    /// Raw input support
    RawInput,
    /// Monitor enumeration
    MonitorInfo,
}

/// GLFW window factory implementation
pub struct GlfwWindowFactory;

impl WindowFactory for GlfwWindowFactory {
    fn create_window(&self, width: u32, height: u32, title: &str) -> Box<dyn Window> {
        info!("Creating GLFW window: {} ({}x{})", title, width, height);
        Box::new(crate::window::artificeglfw::GlfwWindow::new(width, height, title))
    }
    
    fn create_window_with_hints(&self, width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Box<dyn Window> {
        info!("Creating GLFW window with hints: {} ({}x{})", title, width, height);
        Box::new(crate::window::artificeglfw::GlfwWindow::with_hints(width, height, title, hints))
    }
    
    fn supports_feature(&self, feature: WindowFeature) -> bool {
        match feature {
            WindowFeature::OpenGL => true,
            WindowFeature::Vulkan => true, // GLFW supports Vulkan
            WindowFeature::MultiWindow => true,
            WindowFeature::HighDPI => true,
            WindowFeature::Fullscreen => true,
            WindowFeature::Transparency => true,
            WindowFeature::CustomCursor => true,
            WindowFeature::RawInput => true,
            WindowFeature::MonitorInfo => true,
            WindowFeature::DirectX => false, // GLFW doesn't directly support DirectX
        }
    }
    
    fn backend_name(&self) -> &str {
        "GLFW"
    }
    
    fn backend_version(&self) -> Option<String> {
        // This would ideally get the actual GLFW version
        Some("3.3+".to_string())
    }
}

/// Registry for managing different window backends
pub struct WindowBackendRegistry {
    factories: HashMap<String, Box<dyn WindowFactory>>,
    default_backend: Option<String>,
}

impl WindowBackendRegistry {
    /// Create a new registry with default backends
    pub fn new() -> Self {
        let mut registry = WindowBackendRegistry {
            factories: HashMap::new(),
            default_backend: None,
        };
        
        // Register default backends
        registry.register_factory("glfw".to_string(), Box::new(GlfwWindowFactory));
        
        // Register Wayland backend if available
        #[cfg(all(feature = "wayland", target_os = "linux"))]
        registry.register_factory("wayland".to_string(), Box::new(WaylandWindowFactory));
        
        registry.set_default_backend("glfw");
        
        info!("Window backend registry initialized with default backends");
        registry
    }
    
    /// Register a new window factory
    pub fn register_factory(&mut self, name: String, factory: Box<dyn WindowFactory>) {
        info!("Registering window backend: {} ({})", name, factory.backend_name());
        self.factories.insert(name, factory);
    }
    
    /// Set the default backend to use when none is specified
    pub fn set_default_backend(&mut self, backend_name: &str) {
        if self.factories.contains_key(backend_name) {
            self.default_backend = Some(backend_name.to_string());
            info!("Set default window backend to: {}", backend_name);
        } else {
            warn!("Attempted to set unknown backend '{}' as default", backend_name);
        }
    }
    
    /// Create a window using the specified backend
    pub fn create_window(&self, backend: &str, width: u32, height: u32, title: &str) -> Option<Box<dyn Window>> {
        if let Some(factory) = self.factories.get(backend) {
            Some(factory.create_window(width, height, title))
        } else {
            warn!("Unknown window backend requested: {}", backend);
            None
        }
    }
    
    /// Create a window with hints using the specified backend
    pub fn create_window_with_hints(&self, backend: &str, width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Option<Box<dyn Window>> {
        if let Some(factory) = self.factories.get(backend) {
            Some(factory.create_window_with_hints(width, height, title, hints))
        } else {
            warn!("Unknown window backend requested: {}", backend);
            None
        }
    }
    
    /// Create a window using the default backend
    pub fn create_default_window(&self, width: u32, height: u32, title: &str) -> Option<Box<dyn Window>> {
        if let Some(default_backend) = &self.default_backend {
            self.create_window(default_backend, width, height, title)
        } else {
            warn!("No default backend set");
            None
        }
    }
    
    /// Create a window with hints using the default backend
    pub fn create_default_window_with_hints(&self, width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Option<Box<dyn Window>> {
        if let Some(default_backend) = &self.default_backend {
            self.create_window_with_hints(default_backend, width, height, title, hints)
        } else {
            warn!("No default backend set");
            None
        }
    }
    
    /// Get a list of available backends
    pub fn available_backends(&self) -> Vec<&String> {
        self.factories.keys().collect()
    }
    
    /// Check if a backend supports a specific feature
    pub fn backend_supports_feature(&self, backend: &str, feature: WindowFeature) -> bool {
        if let Some(factory) = self.factories.get(backend) {
            factory.supports_feature(feature)
        } else {
            false
        }
    }
    
    /// Get information about a backend
    pub fn get_backend_info(&self, backend: &str) -> Option<BackendInfo> {
        if let Some(factory) = self.factories.get(backend) {
            Some(BackendInfo {
                name: factory.backend_name().to_string(),
                version: factory.backend_version(),
                supported_features: WindowFeature::all()
                    .into_iter()
                    .filter(|&feature| factory.supports_feature(feature))
                    .collect(),
            })
        } else {
            None
        }
    }
    
    /// Get the name of the default backend
    pub fn default_backend(&self) -> Option<&String> {
        self.default_backend.as_ref()
    }
}

impl Default for WindowBackendRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a window backend
#[derive(Debug, Clone)]
pub struct BackendInfo {
    pub name: String,
    pub version: Option<String>,
    pub supported_features: Vec<WindowFeature>,
}

impl WindowFeature {
    /// Get all available window features
    pub fn all() -> Vec<WindowFeature> {
        vec![
            WindowFeature::OpenGL,
            WindowFeature::Vulkan,
            WindowFeature::DirectX,
            WindowFeature::MultiWindow,
            WindowFeature::HighDPI,
            WindowFeature::Fullscreen,
            WindowFeature::Transparency,
            WindowFeature::CustomCursor,
            WindowFeature::RawInput,
            WindowFeature::MonitorInfo,
        ]
    }
}

/// Helper function to create the default window backend registry
pub fn create_default_registry() -> WindowBackendRegistry {
    WindowBackendRegistry::new()
}

/// Helper function to create a window with automatic backend selection
pub fn create_window_auto(width: u32, height: u32, title: &str) -> Option<Box<dyn Window>> {
    let registry = create_default_registry();
    registry.create_default_window(width, height, title)
}

/// Helper function to create a window with hints and automatic backend selection
pub fn create_window_auto_with_hints(width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Option<Box<dyn Window>> {
    let registry = create_default_registry();
    registry.create_default_window_with_hints(width, height, title, hints)
}