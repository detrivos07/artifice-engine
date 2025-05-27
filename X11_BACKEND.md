# X11 Window Backend

## Overview

The X11 backend provides native X11 window support for the Artifice Engine. This backend creates windows directly using the X11 protocol and provides OpenGL rendering context through GLX (OpenGL Extension to the X Window System).

## Features

### Supported Features
- ✅ **OpenGL Rendering**: Full OpenGL context support via GLX
- ✅ **Multi-Window Support**: Create and manage multiple windows
- ✅ **High DPI Support**: Handles high DPI displays
- ✅ **Fullscreen Support**: Window fullscreen capabilities
- ✅ **Window Transparency**: Support for transparent windows
- ✅ **Custom Cursor Support**: Custom cursor management
- ✅ **Raw Input Support**: Direct input event handling
- ✅ **Monitor Information**: Multi-monitor support and information

### Unsupported Features
- ❌ **Vulkan**: Not implemented (use GLFW backend for Vulkan)
- ❌ **DirectX**: Not available on Linux/X11

## System Requirements

### Dependencies
- **X11 development libraries**: `libx11-dev`
- **OpenGL libraries**: `libgl1-mesa-dev` or equivalent
- **GLX support**: Usually included with OpenGL drivers

### Installation (Ubuntu/Debian)
```bash
sudo apt install libx11-dev libgl1-mesa-dev
```

### Installation (Arch Linux)
```bash
sudo pacman -S libx11 mesa
```

### Installation (Fedora/RHEL)
```bash
sudo dnf install libX11-devel mesa-libGL-devel
```

## Usage

### Enable X11 Backend
Add the `x11` feature to your `Cargo.toml`:

```toml
[dependencies]
artifice-engine = { version = "0.0.0", features = ["x11"] }
```

### Direct Window Creation
```rust
use artifice_engine::window::x11::X11Window;
use artifice_engine::io::{Window, WindowHint, OpenGLProfile};

// Basic window
let mut window = X11Window::new(800, 600, "My X11 Window");

// Window with OpenGL hints
let hints = [
    WindowHint::ContextVersion(3, 3),
    WindowHint::OpenGLProfile(OpenGLProfile::Core),
    WindowHint::DoubleBuffer(true),
    WindowHint::Samples(4), // 4x MSAA
];
let mut window = X11Window::with_hints(800, 600, "Advanced X11 Window", &hints);
```

### Factory-Based Creation
```rust
use artifice_engine::window::{WindowBackendRegistry, create_default_registry};

let registry = create_default_registry();
let window = registry.create_window("x11", 800, 600, "Factory Window")
    .expect("Failed to create X11 window");
```

### Backend Switching
```rust
use artifice_engine::{Engine, Application};

// The engine supports hot-switching to X11
// In your application's event handler:
fn event(&mut self, event: &mut Event) {
    if let Some(key_event) = event.as_key_event() {
        if key_event.key == KeyCode::X && key_event.action == KeyAction::Press {
            // Request switch to X11 backend
            self.switch_requested = Some("x11".to_string());
        }
    }
}
```

## OpenGL Context Configuration

The X11 backend supports advanced OpenGL context configuration:

```rust
use artifice_engine::io::{WindowHint, OpenGLProfile};

let hints = [
    // OpenGL version
    WindowHint::ContextVersion(4, 5),
    
    // OpenGL profile
    WindowHint::OpenGLProfile(OpenGLProfile::Core),
    
    // Buffer configuration
    WindowHint::DoubleBuffer(true),
    WindowHint::Samples(8), // Anti-aliasing
    
    // Forward compatibility (for newer OpenGL)
    WindowHint::OpenGLForwardCompat(true),
];

let window = X11Window::with_hints(800, 600, "OpenGL Window", &hints);
```

## Event Handling

The X11 backend provides comprehensive event handling:

```rust
use artifice_engine::events::{Event, EventData};
use std::sync::{Arc, Mutex};

// Set up event callback
let callback = Arc::new(Mutex::new(|event: Event| {
    match event.data {
        EventData::Key(key_event) => {
            println!("Key: {:?} Action: {:?}", key_event.key, key_event.action);
        }
        EventData::MouseMove(move_event) => {
            println!("Mouse: ({}, {})", move_event.x, move_event.y);
        }
        EventData::MouseButton(button_event) => {
            println!("Mouse button: {:?}", button_event.button);
        }
        EventData::WindowResize(resize_event) => {
            println!("Window resized: {}x{}", resize_event.width, resize_event.height);
        }
        _ => {}
    }
}));

window.set_event_callback(callback);
```

## Performance Characteristics

### Advantages
- **Low Latency**: Direct X11 protocol communication
- **Efficient**: Minimal abstraction overhead
- **Native Integration**: Works seamlessly with X11 window managers
- **Resource Control**: Fine-grained control over window resources

### Considerations
- **X11 Only**: Limited to X11-based systems (not Wayland)
- **Platform Specific**: Linux/Unix systems only
- **Complexity**: More complex than high-level abstractions like GLFW

## Comparison with Other Backends

| Feature | X11 | GLFW | Wayland |
|---------|-----|------|---------|
| OpenGL | ✅ Native GLX | ✅ Abstracted | ❌ |
| Vulkan | ❌ | ✅ | ❌ |
| Platform | Linux/Unix | Cross-platform | Linux |
| Performance | High | Medium | High |
| Complexity | Medium | Low | High |
| Dependencies | libX11 | GLFW | libwayland |

## Testing

### Run X11 Test Example
```bash
# Basic test
cargo run --example x11_test --features x11

# Backend switching demo with X11
cargo run --example backend_switching_demo --features x11

# Advanced demo with all backends
cargo run --example advanced_backend_demo --features "x11,wayland"
```

### Using the Demo Script
```bash
# Build with X11 support
./run_backend_demos.sh build-x11

# Run backend switching demo
./run_backend_demos.sh switching --x11

# Run with all backends
./run_backend_demos.sh advanced --all-backends
```

## Troubleshooting

### Common Issues

#### "Failed to open X11 display"
```
ERROR: Failed to open X11 display
```
**Solution**: Ensure you're running in an X11 environment and `$DISPLAY` is set:
```bash
echo $DISPLAY  # Should show something like :0 or :1
export DISPLAY=:0  # If not set
```

#### "Failed to find suitable GLX framebuffer config"
```
ERROR: Failed to find suitable GLX framebuffer config
```
**Solution**: 
1. Install OpenGL drivers: `sudo apt install mesa-libgl1-mesa-dev`
2. Check if hardware acceleration is available: `glxinfo | grep "direct rendering"`
3. Reduce OpenGL requirements (lower version, fewer samples)

#### "X11 libraries not found"
```
ERROR: x11-sys link error
```
**Solution**: Install X11 development headers:
```bash
# Ubuntu/Debian
sudo apt install libx11-dev

# Arch Linux  
sudo pacman -S libx11

# Fedora
sudo dnf install libX11-devel
```

#### Window doesn't appear
**Possible causes**:
1. Window manager not running
2. Window created off-screen
3. X11 permissions issue

**Solutions**:
```rust
// Ensure window is visible and properly positioned
window.set_position(Position::from((100, 100)));
window.set_size(Size::from((800, 600)));
```

#### Backend Switching OpenGL Issues
```
ERROR: OpenGL functions not working after switching to X11
```
**Problem**: When switching from another backend (like GLFW) to X11, OpenGL rendering may not work properly due to context switching issues.

**Root Cause**: OpenGL function pointers are backend-specific and must be reloaded when switching between different windowing systems.

**Solutions**:
1. **Automatic Solution**: The engine automatically calls `reload_opengl_functions()` during backend switches. If this fails, check the logs for downcast errors.

2. **Manual Verification**: In your application's `on_backend_switch_completed` callback, verify OpenGL state:
```rust
fn on_backend_switch_completed(&mut self, _old_backend: &str, new_backend: &str) {
    self.current_backend = new_backend.to_string();
    
    // Verify OpenGL context is working
    unsafe {
        let version = gl::GetString(gl::VERSION);
        if version.is_null() {
            error!("OpenGL context not working after backend switch!");
        } else {
            let version_str = std::ffi::CStr::from_ptr(version as *const i8)
                .to_string_lossy();
            info!("OpenGL working: {}", version_str);
        }
    }
    
    // Re-initialize OpenGL objects
    self.init();
}
```

3. **Debug Backend Detection**: Enable debug logging to see if backend detection is working:
```bash
RUST_LOG=debug cargo run --example backend_switching_demo --features x11
```

4. **Force Context Activation**: If automatic reloading fails, manually ensure context is current:
```rust
// In your application after backend switch
if let Some(opengl_window) = window.as_any_mut().downcast_mut::<X11Window>() {
    opengl_window.make_current();
    opengl_window.reload_opengl_functions();
}
```

**Prevention**: Always re-initialize OpenGL objects (VAOs, VBOs, shaders, etc.) in your `on_backend_switch_completed` callback, as these are not automatically preserved across backend switches.

### Debug Information
Enable debug logging to troubleshoot issues:
```bash
RUST_LOG=debug cargo run --example x11_test --features x11
```

### Performance Optimization

#### For High-Performance Applications
```rust
// Disable VSync for maximum framerate
let hints = [
    WindowHint::DoubleBuffer(true),
    WindowHint::Samples(0), // Disable anti-aliasing
];

// Use efficient event processing
while !window.should_close() {
    window.process_events(); // Process all pending events
    // ... render ...
    window.swap_buffers();
}
```

#### For Visual Quality
```rust
// Enable anti-aliasing and high-quality rendering
let hints = [
    WindowHint::Samples(8), // 8x MSAA
    WindowHint::DoubleBuffer(true),
    WindowHint::ContextVersion(4, 5), // Latest OpenGL
];
```

## Contributing

The X11 backend is actively maintained. To contribute:

1. **Issues**: Report bugs or feature requests on the issue tracker
2. **Testing**: Test on different X11 configurations and window managers
3. **Features**: Implement missing features like Vulkan support
4. **Documentation**: Improve this documentation

### Development Setup
```bash
# Clone and build with X11 support
git clone <repository>
cd artifice-engine
cargo build --features x11

# Run tests
cargo test --features x11

# Build documentation
cargo doc --features x11 --open
```