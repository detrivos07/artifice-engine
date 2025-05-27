# Backend Switching Examples

This document explains how to run the window backend switching demonstrations in the Artifice Engine.

## Overview

The Artifice Engine supports hot-swapping between different window backends during runtime without losing application state. This allows applications to switch between GLFW, Wayland, and potentially other window systems on-the-fly.

## Available Backends

- **GLFW**: Cross-platform windowing library (default)
- **Wayland**: Native Wayland protocol support (Linux only, requires `wayland` feature)

## Building the Examples

### Basic Build (GLFW only)
```bash
cd artifice-engine
cargo build --examples
```

### With Wayland Support
```bash
cd artifice-engine
cargo build --examples --features wayland
```

## Running the Examples

### 1. Basic Demo with Engine Implementation
```bash
cargo run --example basic_demo
```

A simple rotating triangle demo that uses the actual Engine implementation instead of the simplified `run_application` function.

**Controls:**
- `R` - Reset rotation
- `ESC` - Exit

### 2. Simple Backend Switch Demo (Recommended)
```bash
cargo run --example simple_backend_switch
```

A working demonstration of real-time backend switching with visual feedback.

**Controls:**
- `G` - Switch to GLFW backend (teal background)
- `W` - Switch to Wayland backend (purple background)  
- `R` - Reset rotation
- `ESC` - Exit

**Features:**
- Actual runtime backend switching
- Visual feedback through background colors
- Preserved animations during switches
- Simple, clear demonstration

### 3. Backend Switching Demo (Complex)
```bash
cargo run --example backend_switching_demo
```

A more complex demonstration with enhanced features.

**Controls:**
- `G` - Switch to GLFW backend (orange triangle, teal background)
- `W` - Switch to Wayland backend (green triangle, purple background)
- `R` - Reset rotation and colors
- `ESC` - Exit

**Features:**
- Visual feedback shows current backend
- Different colors for each backend
- State preservation during switches
- Event buffering during transitions
- Performance metrics

### 3. Advanced Backend Demo
```bash
cargo run --example advanced_backend_demo
```

A comprehensive demonstration with enhanced animations, performance monitoring, and sophisticated backend management.

**Controls:**
- `G` - Switch to GLFW backend
- `W` - Switch to Wayland backend  
- `R` - Reset all animations
- `SPACE` - Show status information
- `ESC` - Exit

**Features:**
- Enhanced shaders with multiple effects
- Pulsing scale animation
- Color cycling effects
- FPS monitoring
- Switch cooldown prevention
- Comprehensive logging
- Performance metrics reporting
- Layer system demonstration

## What to Expect

### During Backend Switching

1. **Preparation Phase**: The engine validates the target backend and preserves current window state
2. **Transition Phase**: Events are buffered while the new window is created
3. **Completion Phase**: State is transferred to the new backend and buffered events are replayed

### Visual Feedback

- **GLFW Backend**: Blue/teal backgrounds, orange triangles
- **Wayland Backend**: Green/purple backgrounds, purple/green triangles
- Smooth animations continue during switches
- No visual glitches or interruptions

### Performance Impact

Backend switching typically takes 100-500ms depending on:
- System performance
- Graphics driver initialization
- Wayland compositor responsiveness

## Requirements

### System Dependencies

**Ubuntu/Debian:**
```bash
sudo apt install libglfw3-dev libgl1-mesa-dev
```

**For Wayland support:**
```bash
sudo apt install libwayland-dev wayland-protocols
```

**Arch Linux:**
```bash
sudo pacman -S glfw-x11 mesa
```

**For Wayland support:**
```bash
sudo pacman -S wayland wayland-protocols
```

### Runtime Requirements

- OpenGL 3.3+ compatible graphics driver
- X11 or Wayland display server (Linux)
- GLFW 3.3+ (automatically handled by Cargo)

## Troubleshooting

### Common Issues

**"Failed to create window with backend"**
- Ensure the target backend is available on your system
- Check that Wayland is running if trying to switch to Wayland backend
- Verify graphics drivers are properly installed

**"Wayland backend not available"**
- Compile with `--features wayland`
- Ensure you're running on a Linux system with Wayland support

**OpenGL context errors**
- Update graphics drivers
- Ensure OpenGL 3.3+ support
- Try running with `LIBGL_ALWAYS_SOFTWARE=1` for software rendering

### Debug Information

Enable debug logging:
```bash
RUST_LOG=debug cargo run --example backend_switching_demo
```

Enable verbose logging for engine internals:
```bash
RUST_LOG=artifice_engine=trace cargo run --example backend_switching_demo
```

## Technical Details

### Backend Switching Process

1. **Validation**: Target backend availability and feature support
2. **State Capture**: Window size, position, title, and application state
3. **Event Buffering**: Incoming events are queued during transition
4. **Window Creation**: New backend window with captured properties
5. **Context Transfer**: OpenGL context and resources are recreated
6. **Event Replay**: Buffered events are processed in order
7. **Cleanup**: Old backend resources are released

### Performance Considerations

- Graphics resources are automatically recreated after switches
- Application state (animations, logic) is preserved
- Event ordering is maintained through buffering
- Switch operations are atomic (all-or-nothing)

### Architecture

The backend switching system uses:
- **WindowBackendRegistry**: Manages available backend factories
- **HotReloadManager**: Orchestrates the switching process
- **EventBuffer**: Queues events during transitions
- **WindowState**: Captures and restores window properties

This design ensures seamless transitions while maintaining application responsiveness and visual continuity.