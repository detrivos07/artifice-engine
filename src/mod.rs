//! # Artifice Engine
//! 
//! A modular game engine built in Rust with abstracted input and window handling.
//! 
//! This crate provides a framework for building games and interactive applications
//! with support for multiple backends (currently GLFW with OpenGL).

// Re-export the lib.rs components
pub use crate::{
    Application,
    Layer,
    Engine,
    run_application,
};

// Re-export event system
pub use crate::event::{
    Event,
    EventType,
    EventTrait,
    EventHandler,
    EventDispatcher,
    WindowResizeEvent,
    WindowCloseEvent,
    WindowMoveEvent,
    KeyEvent,
    KeyAction,
    KeyCode,
    KeyMod,
    MouseMoveEvent,
    MouseButtonEvent,
    MouseScrollEvent,
    MouseButton,
    GamepadButtonEvent,
    GamepadAxisEvent,
    GamepadButton,
    GamepadAxis,
};

// Re-export I/O system
pub use crate::io::{
    Window,
    OpenGLWindow,
    WindowFactory,
    WindowHint,
    OpenGLProfile,
    Size,
    Position,
    InputDevice,
};

// Re-export input devices
pub use crate::io::keyboard::Keyboard;
pub use crate::io::mouse::Mouse;
pub use crate::io::gamepad::{
    Gamepad,
    GamepadManager,
};

// Re-export GLFW implementation
pub use crate::io::artificeglfw::{
    GlfwWindow,
    GlfwWindowFactory,
};

// Re-export logging
pub use logging;