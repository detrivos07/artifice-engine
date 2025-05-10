///! Artifice GLFW library
///!
///! This library provides a GLFW window and input handling for the Artifice engine.
use crate::io::*;
use crate::event::*;
use crate::io::keyboard::key_translation;
use crate::io::mouse::mouse_translation;
use glfw::{Action, Context, Key, WindowHint as GlfwWindowHint};
use std::sync::{Arc, Mutex};

pub struct GlfwWindow {
    size: Size,
    position: Position,
    title: String,
    glfw: glfw::Glfw,
    glfw_window: glfw::PWindow,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
    event_callback: Option<Arc<Mutex<dyn FnMut(Event) + Send + 'static>>>,
}

impl GlfwWindow {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        // Initialize GLFW
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to initialize GLFW");

        //Create a new GLFW window
        let (mut glfw_window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        //Make the window context current
        glfw_window.show();
        glfw_window.make_current(); // Makes GLFW context current with the GLFW instance
        glfw_window.set_key_polling(true);
        glfw_window.set_cursor_pos_polling(true);
        glfw_window.set_framebuffer_size_polling(true);
        glfw_window.set_mouse_button_polling(true);
        glfw_window.set_scroll_polling(true);

        // Initialize OpenGL
        gl::load_with(|symbol| glfw_window.get_proc_address(symbol) as *const std::os::raw::c_void);

        GlfwWindow {
            size: Size::from((width, height)),
            position: Position::default(),
            title: title.to_string(),
            glfw,
            glfw_window,
            events,
            event_callback: None,
        }
    }
    
    pub fn with_hints(width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Self {
        // Initialize GLFW
        let mut glfw = glfw::init(glfw::fail_on_errors).expect("Failed to initialize GLFW");
        
        // Apply window hints
        for hint in hints {
            match hint {
                WindowHint::Resizable(value) => glfw.window_hint(GlfwWindowHint::Resizable(*value)),
                WindowHint::Visible(value) => glfw.window_hint(GlfwWindowHint::Visible(*value)),
                WindowHint::Decorated(value) => glfw.window_hint(GlfwWindowHint::Decorated(*value)),
                WindowHint::Focused(value) => glfw.window_hint(GlfwWindowHint::Focused(*value)),
                WindowHint::AutoIconify(value) => glfw.window_hint(GlfwWindowHint::AutoIconify(*value)),
                WindowHint::Floating(value) => glfw.window_hint(GlfwWindowHint::Floating(*value)),
                WindowHint::Maximized(value) => glfw.window_hint(GlfwWindowHint::Maximized(*value)),
                WindowHint::Transparent(value) => glfw.window_hint(GlfwWindowHint::TransparentFramebuffer(*value)),
                WindowHint::Samples(value) => glfw.window_hint(GlfwWindowHint::Samples(Some(*value))),
                WindowHint::DoubleBuffer(value) => glfw.window_hint(GlfwWindowHint::DoubleBuffer(*value)),
                WindowHint::RefreshRate(value) => glfw.window_hint(GlfwWindowHint::RefreshRate(Some(*value))),
                WindowHint::ContextVersion(major, minor) => {
                    glfw.window_hint(GlfwWindowHint::ContextVersion(*major, *minor));
                },
                WindowHint::OpenGLProfile(profile) => {
                    let glfw_profile = match profile {
                        OpenGLProfile::Any => glfw::OpenGlProfileHint::Any,
                        OpenGLProfile::Core => glfw::OpenGlProfileHint::Core,
                        OpenGLProfile::Compatibility => glfw::OpenGlProfileHint::Compat,
                    };
                    glfw.window_hint(GlfwWindowHint::OpenGlProfile(glfw_profile));
                },
                WindowHint::OpenGLForwardCompat(value) => glfw.window_hint(GlfwWindowHint::OpenGlForwardCompat(*value)),
            }
        }

        //Create a new GLFW window
        let (mut glfw_window, events) = glfw
            .create_window(width, height, title, glfw::WindowMode::Windowed)
            .expect("Failed to create GLFW window");

        //Make the window context current
        glfw_window.show();
        glfw_window.make_current(); // Makes GLFW context current with the GLFW instance
        glfw_window.set_key_polling(true);
        glfw_window.set_cursor_pos_polling(true);
        glfw_window.set_framebuffer_size_polling(true);
        glfw_window.set_mouse_button_polling(true);
        glfw_window.set_scroll_polling(true);

        // Initialize OpenGL
        gl::load_with(|symbol| glfw_window.get_proc_address(symbol) as *const std::os::raw::c_void);

        GlfwWindow {
            size: Size::from((width, height)),
            position: Position::default(),
            title: title.to_string(),
            glfw,
            glfw_window,
            events,
            event_callback: None,
        }
    }
}

impl Window for GlfwWindow {
    /// Swaps buffers, Polls events, and processes events
    fn update(&mut self) {
        self.glfw_window.swap_buffers();
        self.glfw.poll_events();
        self.process_events();
    }

    fn process_events(&mut self) {
        for (_, event) in glfw::flush_messages(&self.events) {
            match event {
                glfw::WindowEvent::Key(key, _, action, mods) => {
                    // Convert GLFW key to our key code
                    let key_code = key_translation::from_glfw_key(key);
                    let key_action = key_translation::from_glfw_action(action);
                    let key_mods = key_translation::from_glfw_mods(mods);
                    
                    // Check for escape key to close window
                    if key == Key::Escape && action == Action::Press {
                        self.glfw_window.set_should_close(true);
                    }
                    
                    // Create the key event
                    let key_event = KeyEvent {
                        key: key_code,
                        action: key_action,
                        mods: key_mods,
                    };
                    
                    // Dispatch the event
                    if let Some(callback) = &self.event_callback {
                        let mut event = Event::new(key_event);
                        let mut callback = callback.lock().unwrap();
                        callback(event);
                    }
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    // Update internal size
                    self.size = Size::from((width, height));
                    
                    // Update OpenGL viewport
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
                    
                    // Create the resize event
                    let resize_event = WindowResizeEvent {
                        width: width as u32,
                        height: height as u32,
                    };
                    
                    // Dispatch the event
                    if let Some(callback) = &self.event_callback {
                        let mut event = Event::new(resize_event);
                        let mut callback = callback.lock().unwrap();
                        callback(event);
                    }
                }
                glfw::WindowEvent::CursorPos(x, y) => {
                    // Create mouse move event
                    let move_event = MouseMoveEvent { x, y };
                    
                    // Dispatch the event
                    if let Some(callback) = &self.event_callback {
                        let mut event = Event::new(move_event);
                        let mut callback = callback.lock().unwrap();
                        callback(event);
                    }
                }
                glfw::WindowEvent::MouseButton(button, action, mods) => {
                    // Convert GLFW mouse button to our mouse button
                    let mouse_button = mouse_translation::from_glfw_button(button);
                    let button_action = key_translation::from_glfw_action(action);
                    let key_mods = key_translation::from_glfw_mods(mods);
                    
                    // Create mouse button event
                    let button_event = MouseButtonEvent {
                        button: mouse_button,
                        action: button_action,
                        mods: key_mods,
                    };
                    
                    // Dispatch the event
                    if let Some(callback) = &self.event_callback {
                        let mut event = Event::new(button_event);
                        let mut callback = callback.lock().unwrap();
                        callback(event);
                    }
                }
                glfw::WindowEvent::Scroll(x_offset, y_offset) => {
                    // Create scroll event
                    let scroll_event = MouseScrollEvent {
                        x_offset,
                        y_offset,
                    };
                    
                    // Dispatch the event
                    if let Some(callback) = &self.event_callback {
                        let mut event = Event::new(scroll_event);
                        let mut callback = callback.lock().unwrap();
                        callback(event);
                    }
                }
                glfw::WindowEvent::Close => {
                    // Create close event
                    let close_event = WindowCloseEvent;
                    
                    // Dispatch the event
                    if let Some(callback) = &self.event_callback {
                        let mut event = Event::new(close_event);
                        let mut callback = callback.lock().unwrap();
                        callback(event);
                    }
                    
                    self.glfw_window.set_should_close(true);
                }
                _ => {}
            }
        }
    }

    fn set_should_close(&mut self) {
        self.glfw_window.set_should_close(true);
    }

    fn should_close(&self) -> bool {
        self.glfw_window.should_close()
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
        self.glfw_window.set_pos(position.0, position.1);
    }

    fn position(&self) -> &Position {
        &self.position
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.glfw_window
            .set_size(self.size.0 as i32, self.size.1 as i32);
    }

    fn size(&self) -> &Size {
        &self.size
    }

    fn title(&self) -> &str {
        &self.title
    }
    
    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        self.glfw_window.set_title(title);
    }
    
    fn get_event_callback(&self) -> Option<Arc<Mutex<dyn FnMut(Event) + Send + 'static>>> {
        self.event_callback.clone()
    }
    
    fn set_event_callback(&mut self, callback: Arc<Mutex<dyn FnMut(Event) + Send + 'static>>) {
        self.event_callback = Some(callback);
    }
}

impl OpenGLWindow for GlfwWindow {
    fn is_current(&self) -> bool {
        self.glfw_window.is_current()
    }

    fn make_current(&mut self) {
        self.glfw_window.make_current();
    }
    
    fn swap_buffers(&mut self) {
        self.glfw_window.swap_buffers();
    }
}

pub struct GlfwWindowFactory;

impl WindowFactory for GlfwWindowFactory {
    type WindowType = GlfwWindow;
    
    fn create_window(width: u32, height: u32, title: &str) -> Self::WindowType {
        GlfwWindow::new(width, height, title)
    }
    
    fn create_window_with_hints(width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Self::WindowType {
        GlfwWindow::with_hints(width, height, title, hints)
    }
}
