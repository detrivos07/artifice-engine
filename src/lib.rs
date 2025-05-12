pub mod event;
pub mod io;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::event::{Event, EventDispatcher, EventType};
use crate::io::artificeglfw::GlfwWindow;
use crate::io::keyboard::Keyboard;
use crate::io::mouse::Mouse;
use crate::io::{InputDevice, Window};
use logging::*;

/// The core Application trait that all applications must implement
pub trait Application: Send + 'static {
    /// Create a new instance of the application
    fn new() -> Self
    where
        Self: Sized;

    /// Called once when the application starts
    fn init(&mut self) {}

    /// Called once per frame to update the application state
    fn update(&mut self, _delta_time: f32) {}

    /// Called once per frame after update to render the application
    fn render(&mut self) {}

    /// Called when the application is about to close
    fn shutdown(&mut self) {}

    /// Called for each event that occurs
    fn event(&mut self, _event: &mut Event) {}

    /// Get the application name
    fn get_name(&self) -> &str {
        "Artifice Application"
    }
}

/// A layer that can be added to the application stack
pub trait Layer: Send + 'static {
    /// Called once when the layer is attached to the application
    fn attach(&mut self) {}

    /// Called once when the layer is detached from the application
    fn detach(&mut self) {}

    /// Called once per frame to update the layer state
    fn update(&mut self, _delta_time: f32) {}

    /// Called once per frame after update to render the layer
    fn render(&mut self) {}

    /// Called for each event that occurs
    fn event(&mut self, _event: &mut Event) {}

    /// Get the layer name
    fn get_name(&self) -> &str {
        "Layer"
    }
}

/// The main engine class that runs the application
pub struct Engine<T: Application> {
    application: Box<T>,
    window: Box<dyn Window>,
    event_dispatcher: EventDispatcher,
    keyboard: Keyboard,
    mouse: Mouse,
    layers: Vec<Box<dyn Layer>>,
    running: bool,
    last_frame_time: Instant,
}

impl<T: Application> Engine<T> {
    /// Create a new engine instance with the given application
    pub fn new(application: T) -> Self {
        info!("Creating Engine instance");

        // Create the window directly
        let window = GlfwWindow::new(800, 600, application.get_name());

        Engine {
            application: Box::new(application),
            window: Box::new(window),
            event_dispatcher: EventDispatcher::new(),
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            layers: Vec::new(),
            running: false,
            last_frame_time: Instant::now(),
        }
    }

    /// Run the application
    pub fn run(&mut self) {
        info!("Engine starting");
        self.running = true;
        self.last_frame_time = Instant::now();

        // Create an event channel for thread-safe event passing
        let (event_sender, event_receiver) = std::sync::mpsc::channel::<Event>();

        // Create a thread-safe callback that sends events through the channel
        let event_sender_clone = event_sender.clone();
        let event_callback = Arc::new(Mutex::new(move |event: Event| {
            // Send the event to the main thread for processing
            let _ = event_sender_clone.send(event);
        }));

        // Set the event callback on the window
        self.window.set_event_callback(event_callback);

        // Initialize the application
        self.application.init();

        // Initialize layers
        for layer in &mut self.layers {
            layer.attach();
        }

        info!("Starting main loop");

        // Main loop
        while self.running && !self.window.should_close() {
            // Calculate delta time
            let current_time = Instant::now();
            let delta_time = current_time
                .duration_since(self.last_frame_time)
                .as_secs_f32();
            self.last_frame_time = current_time;

            // Process window events - this will call our callback if events occur
            self.window.process_events();

            // Process any pending events that came from the callback
            while let Ok(event) = event_receiver.try_recv() {
                // Process input state
                match event.event_type {
                    EventType::Keyboard => {
                        if let Some(key_event) = event.get_data::<crate::event::KeyEvent>() {
                            self.keyboard.process_key_event(
                                key_event.key,
                                key_event.action,
                                key_event.mods.clone(),
                            );
                        }
                    }
                    EventType::Mouse => {
                        if let Some(move_event) = event.get_data::<crate::event::MouseMoveEvent>() {
                            self.mouse.process_move_event(move_event.x, move_event.y);
                        } else if let Some(button_event) =
                            event.get_data::<crate::event::MouseButtonEvent>()
                        {
                            self.mouse.process_button_event(
                                button_event.button,
                                button_event.action,
                                button_event.mods.clone(),
                            );
                        } else if let Some(scroll_event) =
                            event.get_data::<crate::event::MouseScrollEvent>()
                        {
                            self.mouse
                                .process_scroll_event(scroll_event.x_offset, scroll_event.y_offset);
                        }
                    }
                    _ => {}
                }

                // Forward events to layers
                let mut event_copy = event;
                for layer in self.layers.iter_mut().rev() {
                    if !event_copy.is_handled() {
                        layer.event(&mut event_copy);
                    }
                }

                // Forward events to application
                if !event_copy.is_handled() {
                    self.application.event(&mut event_copy);
                }
            }

            // Update input devices
            self.keyboard.update();
            self.mouse.update();

            // Update layers
            for layer in &mut self.layers {
                layer.update(delta_time);
            }

            // Update application
            self.application.update(delta_time);

            // Render layers
            for layer in &mut self.layers {
                layer.render();
            }

            // Render application
            self.application.render();

            // Update window (swap buffers)
            self.window.update();
        }

        info!("Engine shutdown initiated");

        // Detach layers in reverse order
        for layer in self.layers.iter_mut().rev() {
            layer.detach();
        }

        // Shutdown the application
        self.application.shutdown();

        info!("Engine shutdown complete");
    }

    /// Stop the application
    pub fn stop(&mut self) {
        info!("Engine stop requested");
        self.running = false;
    }

    /// Add a layer to the application
    pub fn push_layer(&mut self, mut layer: Box<dyn Layer>) {
        debug!("Adding layer: {}", layer.get_name());
        layer.attach();
        self.layers.push(layer);
    }

    /// Remove a layer from the application
    pub fn pop_layer(&mut self) {
        if let Some(mut layer) = self.layers.pop() {
            debug!("Removing layer: {}", layer.get_name());
            layer.detach();
        }
    }

    /// Get the window
    pub fn get_window(&self) -> &dyn Window {
        self.window.as_ref()
    }

    /// Get the window (mutable)
    pub fn get_window_mut(&mut self) -> &mut dyn Window {
        self.window.as_mut()
    }

    /// Get the keyboard
    pub fn get_keyboard(&self) -> &Keyboard {
        &self.keyboard
    }

    /// Get the mouse
    pub fn get_mouse(&self) -> &Mouse {
        &self.mouse
    }
}

/// Run an application
pub fn run_application<T: Application>() {
    info!("Starting application");
    let app = T::new();
    let mut engine = Engine::new(app);

    // Set up OpenGL debug output if available
    unsafe {
        if gl::DebugMessageCallback::is_loaded() {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);
        }
    }

    engine.run();
    info!("Application terminated");
}
