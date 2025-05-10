pub mod event;
pub mod io;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::event::{Event, EventDispatcher};
use crate::io::{Window, WindowFactory, InputDevice};
use crate::io::artificeglfw::GlfwWindowFactory;
use crate::io::keyboard::Keyboard;
use crate::io::mouse::Mouse;
use crate::io::gamepad::GamepadManager;

/// The core Application trait that all applications must implement
pub trait Application {
    /// Create a new instance of the application
    fn new() -> Self where Self: Sized;
    
    /// Called once when the application starts
    fn on_init(&mut self) {}
    
    /// Called once per frame to update the application state
    fn on_update(&mut self, _delta_time: f32) {}
    
    /// Called once per frame after update to render the application
    fn on_render(&mut self) {}
    
    /// Called when the application is about to close
    fn on_shutdown(&mut self) {}
    
    /// Called for each event that occurs
    fn on_event(&mut self, _event: &mut Event) {}
    
    /// Get the application name
    fn get_name(&self) -> &str {
        "Artifice Application"
    }
}

/// A layer that can be added to the application stack
pub trait Layer {
    /// Called once when the layer is attached to the application
    fn on_attach(&mut self) {}
    
    /// Called once when the layer is detached from the application
    fn on_detach(&mut self) {}
    
    /// Called once per frame to update the layer state
    fn on_update(&mut self, _delta_time: f32) {}
    
    /// Called once per frame after update to render the layer
    fn on_render(&mut self) {}
    
    /// Called for each event that occurs
    fn on_event(&mut self, _event: &mut Event) {}
    
    /// Get the layer name
    fn get_name(&self) -> &str {
        "Layer"
    }
}

/// The main engine class that runs the application
pub struct Engine<T: Application> {
    application: T,
    window: Box<dyn Window>,
    event_dispatcher: EventDispatcher,
    keyboard: Keyboard,
    mouse: Mouse,
    gamepad_manager: GamepadManager,
    layers: Vec<Box<dyn Layer>>,
    running: bool,
    last_frame_time: Instant,
}

impl<T: Application> Engine<T> {
    /// Create a new engine instance with the given application
        pub fn new(application: T) -> Self {
            // Create the window
            let mut window = GlfwWindowFactory::create_window(800, 600, application.get_name());
        
            // Create a simple event callback that just logs events for now
            // We'll replace this with proper event handling in the run method
            let event_callback = Arc::new(Mutex::new(|event: Event| {
                logging::debug(&format!("Event received: {:?}", event.event_type));
            }));
        
            window.set_event_callback(event_callback);
        
            Engine {
                application,
                window: Box::new(window),
                event_dispatcher: EventDispatcher::new(),
                keyboard: Keyboard::new(),
                mouse: Mouse::new(),
                gamepad_manager: GamepadManager::new(),
                layers: Vec::new(),
                running: false,
                last_frame_time: Instant::now(),
            }
        }
    
    /// Run the application
    pub fn run(&mut self) {
        self.running = true;
        self.last_frame_time = Instant::now();
        
        // Set up a proper event callback
        let event_callback = Arc::new(Mutex::new(|mut event: Event| {
            logging::trace(&format!("Event received: {:?}", event.event_type));
            // Note: We can't directly call application methods here due to ownership issues
            // Events will be processed in the main loop
        }));
        
        self.window.set_event_callback(event_callback);
        
        // Initialize the application
        self.application.on_init();
        
        // Main loop
        while self.running && !self.window.should_close() {
            // Calculate delta time
            let current_time = Instant::now();
            let delta_time = current_time.duration_since(self.last_frame_time).as_secs_f32();
            self.last_frame_time = current_time;
            
            // Update input devices
            self.keyboard.update();
            self.mouse.update();
            self.gamepad_manager.update();
            
            // Update layers
            for layer in &mut self.layers {
                layer.on_update(delta_time);
            }
            
            // Update application
            self.application.on_update(delta_time);
            
            // Render layers
            for layer in &mut self.layers {
                layer.on_render();
            }
            
            // Render application
            self.application.on_render();
            
            // Update window (swap buffers and poll events)
            self.window.update();
        }
        
        // Shutdown the application
        self.application.on_shutdown();
    }
    
    /// Stop the application
    pub fn stop(&mut self) {
        self.running = false;
    }
    
    /// Add a layer to the application
    pub fn push_layer(&mut self, mut layer: Box<dyn Layer>) {
        layer.on_attach();
        self.layers.push(layer);
    }
    
    /// Remove a layer from the application
    pub fn pop_layer(&mut self) {
        if let Some(mut layer) = self.layers.pop() {
            layer.on_detach();
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
    
    /// Get the gamepad manager
    pub fn get_gamepad_manager(&self) -> &GamepadManager {
        &self.gamepad_manager
    }
}

/// Run an application
pub fn run_application<T: Application + 'static>() {
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
}
