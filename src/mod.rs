//! # Artifice Engine
//!
//! A modular game engine built in Rust with abstracted input and window handling.
//!
//! This crate provides a framework for building games and interactive applications
//! with support for multiple backends (currently GLFW with OpenGL).

// Re-export the lib.rs components
pub use crate::{run_application, Application, Engine, Layer};

// Re-export event system
pub use crate::event::{
    ApplicationTickEvent, Event, EventDispatcher, EventHandler, EventTrait, EventType, KeyAction,
    KeyCode, KeyEvent, KeyMod, MouseButton, MouseButtonEvent, MouseMoveEvent, MouseScrollEvent,
    WindowCloseEvent, WindowMoveEvent, WindowResizeEvent,
};

// Re-export I/O system
pub use crate::io::{InputDevice, OpenGLProfile, OpenGLWindow, Position, Size, Window, WindowHint};

// Re-export input devices
pub use crate::io::keyboard::Keyboard;
pub use crate::io::mouse::Mouse;

// Re-export GLFW implementation
pub use crate::io::artificeglfw::GlfwWindow;

// Re-export logging
pub use logging;
