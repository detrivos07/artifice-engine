///! Artifice GLFW library
///!
///! This library provides a GLFW window and input handling for the Artifice engine.
use crate::io::*;
use glfw::{Action, Context, Key, WindowHint};

pub struct GlfwWindow {
    size: Size,
    position: Position,
    title: String,
    glfw: glfw::Glfw,
    glfw_window: glfw::PWindow,
    events: glfw::GlfwReceiver<(f64, glfw::WindowEvent)>,
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

        // Initialize OpenGL
        gl::load_with(|symbol| glfw_window.get_proc_address(symbol) as *const std::os::raw::c_void);

        GlfwWindow {
            size: Size::from((width, height)),
            position: Position::default(),
            title: title.to_string(),
            glfw,
            glfw_window,
            events,
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
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    self.glfw_window.set_should_close(true)
                }
                glfw::WindowEvent::FramebufferSize(width, height) => {
                    self.size = Size::from((width, height));
                    unsafe {
                        gl::Viewport(0, 0, width, height);
                    }
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
}
