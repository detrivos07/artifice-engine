#![allow(unused)]

pub mod artificeglfw;
pub mod keyboard;
pub mod mouse;

use crate::event::Event;
use std::sync::{Arc, Mutex};

/// Trait representing a window.
///
/// This trait defines the basic functionality of a window, including updating,
/// processing events, setting the should close flag, checking if the window
/// should close, getting the window size, and getting the window title.
pub trait Window {
    fn update(&mut self);
    fn process_events(&mut self);
    fn set_should_close(&mut self);
    fn should_close(&self) -> bool;
    fn set_position(&mut self, position: Position);
    fn position(&self) -> &Position;
    fn set_size(&mut self, size: Size);
    fn size(&self) -> &Size;
    fn title(&self) -> &str;
    fn set_title(&mut self, title: &str);
    fn get_event_callback(&self) -> Option<Arc<Mutex<dyn FnMut(Event) + Send + 'static>>>;
    fn set_event_callback(&mut self, callback: Arc<Mutex<dyn FnMut(Event) + Send + 'static>>);
}

/// Extends the Window trait with OpenGL-specific functionality.
pub trait OpenGLWindow: Window {
    fn make_current(&mut self);
    fn is_current(&self) -> bool;
    fn swap_buffers(&mut self);
}

/// Window hints for configuring window creation
#[derive(Debug, Clone)]
pub enum WindowHint {
    Resizable(bool),
    Visible(bool),
    Decorated(bool),
    Focused(bool),
    AutoIconify(bool),
    Floating(bool),
    Maximized(bool),
    Transparent(bool),
    Samples(u32),
    DoubleBuffer(bool),
    RefreshRate(u32),
    ContextVersion(u32, u32),
    OpenGLProfile(OpenGLProfile),
    OpenGLForwardCompat(bool),
}

#[derive(Debug, Clone, Copy)]
pub enum OpenGLProfile {
    Any,
    Core,
    Compatibility,
}

/// Input device trait for common functionality
pub trait InputDevice {
    fn update(&mut self);
    fn is_connected(&self) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct Size(pub u32, pub u32);

impl Size {
    /// Returns the width and height of the Size.
    pub fn size(&self) -> (u32, u32) {
        (self.0, self.1)
    }

    /// Returns the width of the Size.
    pub fn width(&self) -> u32 {
        self.0
    }

    /// Returns the height of the Size.
    pub fn height(&self) -> u32 {
        self.1
    }
}

impl From<(u32, u32)> for Size {
    /// Converts a `(u32, u32)` into a `Size`.
    fn from((width, height): (u32, u32)) -> Self {
        Size(width, height)
    }
}

impl From<(i32, i32)> for Size {
    /// Converts a `(i32, i32)` into a `Size`.
    fn from((width, height): (i32, i32)) -> Self {
        Size(width as u32, height as u32)
    }
}

impl From<Size> for (u32, u32) {
    /// Converts a `Size` into a tuple of `(u32, u32)`.
    fn from(size: Size) -> Self {
        (size.0, size.1)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position(pub i32, pub i32);

impl Position {
    /// Returns the x and y coordinates of the Position.
    pub fn position(&self) -> (i32, i32) {
        (self.0, self.1)
    }

    /// returns the x of the Position.
    pub fn x(&self) -> i32 {
        self.0
    }

    /// returns the y of the Position.
    pub fn y(&self) -> i32 {
        self.1
    }
}

impl From<(i32, i32)> for Position {
    /// Converts a `(i32, i32)` into a `Position`.
    fn from((x, y): (i32, i32)) -> Self {
        Position(x, y)
    }
}

impl From<Position> for (i32, i32) {
    /// Converts a `Position` into a tuple of `(i32, i32)`.
    fn from(position: Position) -> Self {
        (position.0, position.1)
    }
}
