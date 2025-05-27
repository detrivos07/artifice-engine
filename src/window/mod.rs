pub mod artificeglfw;
#[cfg(feature = "wayland")]
pub mod wayland;
pub mod factory;
pub mod backend_hotswap;

// Re-export key types for easier access
pub use artificeglfw::GlfwWindow;
pub use factory::{
    WindowFactory, WindowFeature, WindowBackendRegistry, BackendInfo,
    GlfwWindowFactory, create_default_registry, create_window_auto, create_window_auto_with_hints
};

#[cfg(feature = "wayland")]
pub use wayland::{WaylandWindow, WaylandWindowFactory};

pub use backend_hotswap::{
    WindowBackendHotswapManager as HotReloadManager,
    WindowBackendHotswapConfig as HotReloadConfig, 
    WindowBackendHotswapStatus as HotReloadStatus,
    WindowBackendHotswapResult as HotReloadResult,
    WindowBackendHotswapBuilder as HotReloadBuilder,
    WindowBackendHotswapFactory as HotReloadFactory,
    WindowBackendHotswapStats as HotReloadStats,
    WindowState,
    EventBuffer
};