pub mod events;
pub mod input;
pub mod window;
pub mod io;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::events::{Event, EventDispatcher, EventFilterManager};
use crate::input::InputManager;
use crate::io::{
    Window, MetricsCollector, MetricsReporter, MetricsConfig, MetricsFactory
};
use crate::window::{
    HotReloadManager, HotReloadConfig, WindowBackendRegistry
};
use artifice_logging::{debug, info, warn};

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
    event_filter_manager: EventFilterManager,
    input_manager: InputManager,
    metrics_collector: Option<Arc<MetricsCollector>>,
    metrics_reporter: Option<MetricsReporter>,
    hot_reload_manager: HotReloadManager,
    layers: Vec<Box<dyn Layer>>,
    running: bool,
    last_frame_time: Instant,
}

impl<T: Application> Engine<T> {
    /// Create a new engine instance with the given application
    pub fn new(application: T) -> Self {
        Self::with_backend(application, "glfw")
    }
    
    /// Create a new engine instance with a specific backend
    pub fn with_backend(application: T, backend: &str) -> Self {
        Self::with_config(application, backend, MetricsConfig::default(), HotReloadConfig::default())
    }

    /// Create a new engine instance with full configuration
    pub fn with_config(application: T, backend: &str, metrics_config: MetricsConfig, hot_reload_config: HotReloadConfig) -> Self {
        info!("Creating Engine instance with {} backend", backend);

        // Create window backend registry
        let mut registry = WindowBackendRegistry::new();
        
        // Create window
        let mut window = registry.create_window(backend, 800, 600, application.get_name())
            .unwrap_or_else(|| {
                warn!("Failed to create window with backend '{}', falling back to default", backend);
                registry.create_default_window(800, 600, application.get_name())
                    .expect("Failed to create window with default backend")
            });

        // Create input manager
        let input_manager = InputManager::new();

        // Set up metrics if enabled
        let (metrics_collector, metrics_reporter) = if metrics_config.enabled {
            let (collector, reporter) = MetricsFactory::create_system(&metrics_config);
            (Some(collector), Some(reporter))
        } else {
            (None, None)
        };

        // Create hot reload manager
        let hot_reload_manager = HotReloadManager::with_config(registry, hot_reload_config);

        // Set up the event callback to use our lock-free queue
        let event_queue = input_manager.get_event_queue();
        let metrics_handle = metrics_collector.as_ref().map(|c| c.get_handle());
        
        let event_callback = Arc::new(Mutex::new(move |event: Event| {
            // Record metrics if enabled
            if let Some(ref handle) = metrics_handle {
                let _timer = crate::io::MetricsTimer::new(handle.clone(), format!("{:?}", event.event_type));
            }
            
            if let Err(rejected_event) = event_queue.try_push(event) {
                warn!("Event queue full, dropping event: {:?}", rejected_event);
                if let Some(ref handle) = metrics_handle {
                    handle.record_event_dropped(&format!("{:?}", rejected_event.event_type));
                }
            }
        }));

        window.set_event_callback(event_callback);

        Engine {
            application: Box::new(application),
            window,
            event_dispatcher: EventDispatcher::new(),
            event_filter_manager: EventFilterManager::new(),
            input_manager,
            metrics_collector,
            metrics_reporter,
            hot_reload_manager,
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

            // Process window events first - this will call our callback if events occur
            self.window.process_events();

            // Process input events and update input state
            let mut events = self.input_manager.process_events();

            // Apply event filters
            events = self.event_filter_manager.filter_events(events);

            // Forward events to layers and application
            for mut event in events {
                // Record event processing metrics
                let _timer = if let Some(ref metrics) = self.metrics_collector {
                    crate::io::MetricsTimer::new(metrics.get_handle(), format!("{:?}", event.event_type))
                } else {
                    crate::io::MetricsTimer::disabled()
                };

                // Forward to layers (in reverse order)
                for layer in self.layers.iter_mut().rev() {
                    if !event.handled {
                        layer.event(&mut event);
                    }
                }

                // Forward to application
                if !event.handled {
                    self.application.event(&mut event);
                }
            }

            // Update input devices
            self.input_manager.update();

            // Update metrics reporter
            if let Some(ref mut reporter) = self.metrics_reporter {
                reporter.update();
            }

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

    /// Get the input manager
    pub fn get_input_manager(&self) -> &InputManager {
        &self.input_manager
    }

    /// Get the input manager (mutable)
    pub fn get_input_manager_mut(&mut self) -> &mut InputManager {
        &mut self.input_manager
    }

    /// Get the keyboard
    pub fn get_keyboard(&self) -> &crate::input::keyboard::Keyboard {
        self.input_manager.keyboard()
    }

    /// Get the mouse
    pub fn get_mouse(&self) -> &crate::input::mouse::Mouse {
        self.input_manager.mouse()
    }

    /// Get the gamepad manager
    pub fn get_gamepad_manager(&self) -> &crate::input::gamepad::GamepadManager {
        self.input_manager.gamepad()
    }

    /// Get the gamepad manager (mutable)
    pub fn get_gamepad_manager_mut(&mut self) -> &mut crate::input::gamepad::GamepadManager {
        self.input_manager.gamepad_mut()
    }

    /// Get the event filter manager
    pub fn get_event_filter_manager(&self) -> &EventFilterManager {
        &self.event_filter_manager
    }

    /// Get the event filter manager (mutable)
    pub fn get_event_filter_manager_mut(&mut self) -> &mut EventFilterManager {
        &mut self.event_filter_manager
    }

    /// Get the metrics collector
    pub fn get_metrics_collector(&self) -> Option<&Arc<MetricsCollector>> {
        self.metrics_collector.as_ref()
    }

    /// Get the hot reload manager
    pub fn get_hot_reload_manager(&self) -> &HotReloadManager {
        &self.hot_reload_manager
    }

    /// Get the hot reload manager (mutable)
    pub fn get_hot_reload_manager_mut(&mut self) -> &mut HotReloadManager {
        &mut self.hot_reload_manager
    }

    /// Switch to a different window backend using hot reload
    pub fn switch_backend(&mut self, backend_name: &str) -> Result<(), String> {
        // Start the hot reload process
        self.hot_reload_manager.start_reload(backend_name, self.window.as_ref())?;

        // Create new window with the target backend
        let mut new_window = self.hot_reload_manager.create_window_with_backend(
            backend_name,
            self.window.size().0,
            self.window.size().1,
            self.window.title(),
            &[],
        )?;

        // Set up event callback for new window
        let event_queue = self.input_manager.get_event_queue();
        let metrics_handle = self.metrics_collector.as_ref().map(|c| c.get_handle());
        
        let event_callback = Arc::new(Mutex::new(move |event: Event| {
            if let Some(ref handle) = metrics_handle {
                let _timer = crate::io::MetricsTimer::new(handle.clone(), format!("{:?}", event.event_type));
            }
            
            if let Err(rejected_event) = event_queue.try_push(event) {
                warn!("Event queue full, dropping event: {:?}", rejected_event);
                if let Some(ref handle) = metrics_handle {
                    handle.record_event_dropped(&format!("{:?}", rejected_event.event_type));
                }
            }
        }));

        new_window.set_event_callback(event_callback);

        // Complete the hot reload
        let result = self.hot_reload_manager.complete_reload(backend_name, new_window.as_mut());
        
        // Replace the window
        self.window = new_window;

        info!("Backend switch completed: {:?}", result);
        Ok(())
    }

    /// Get current metrics snapshot
    pub fn get_metrics(&self) -> Option<crate::io::EventSystemMetrics> {
        self.metrics_collector.as_ref().map(|collector| collector.get_metrics())
    }

    /// Enable or disable metrics collection
    pub fn set_metrics_enabled(&mut self, enabled: bool) {
        if let Some(ref collector) = self.metrics_collector {
            collector.set_enabled(enabled);
        }
        if let Some(ref mut reporter) = self.metrics_reporter {
            if enabled {
                reporter.enable();
            } else {
                reporter.disable();
            }
        }
    }

    /// Generate a metrics report
    pub fn report_metrics(&self) {
        if let Some(ref collector) = self.metrics_collector {
            collector.log_metrics_summary();
        }
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
