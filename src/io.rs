pub mod artificeglfw;
mod button;
mod keyboard;

/// Trait representing a window.
///
/// This trait defines the basic functionality of a window, including updating,
/// processing events, setting the should close flag, checking if the window
/// should close, getting the window size, and getting the window title.
#[allow(dead_code)]
pub trait Window {
    fn update(&mut self);
    fn process_events(&mut self);
    fn set_should_close(&mut self);
    fn should_close(&self) -> bool;
    fn size(&self) -> &Size;
    fn title(&self) -> &str;
}

pub struct Size(u32, u32);

impl Size {
    #[allow(dead_code)]
    pub fn size(&self) -> u32 {
        self.0 + self.1
    }
}

impl From<(u32, u32)> for Size {
    fn from((width, height): (u32, u32)) -> Self {
        Size(width, height)
    }
}

impl From<(i32, i32)> for Size {
    fn from((width, height): (i32, i32)) -> Self {
        Size(width as u32, height as u32)
    }
}

impl From<Size> for (u32, u32) {
    fn from(size: Size) -> Self {
        (size.0, size.1)
    }
}
