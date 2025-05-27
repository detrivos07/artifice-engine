extern crate artifice_engine;
extern crate gl;
extern crate glfw;
extern crate artifice_logging;

use artifice_engine::events::{Event, EventType, KeyAction, KeyCode, MouseButton};
use artifice_engine::{run_application, Application};
use artifice_logging::{info, warn};

pub struct EventSystemDemoApp {
    frame_count: u64,
    total_events_processed: u64,
    last_mouse_position: (f64, f64),
    keys_pressed_this_frame: Vec<KeyCode>,
}

impl Application for EventSystemDemoApp {
    fn new() -> Self {
        EventSystemDemoApp {
            frame_count: 0,
            total_events_processed: 0,
            last_mouse_position: (0.0, 0.0),
            keys_pressed_this_frame: Vec::new(),
        }
    }

    fn init(&mut self) {
        info!("Event System Demo Application initialized!");
        info!("Press keys, move mouse, and interact with the window to see events in action");
        info!("Press ESC to exit, R to reset counters, SPACE for input state info");
        
        unsafe {
            gl::ClearColor(0.1, 0.1, 0.2, 1.0);
        }
    }

    fn update(&mut self, _delta_time: f32) {
        self.frame_count += 1;
        self.keys_pressed_this_frame.clear();
        
        // Log stats every 300 frames (approximately every 5 seconds at 60 FPS)
        if self.frame_count % 300 == 0 {
            info!("Frame {}: Total events processed: {}", self.frame_count, self.total_events_processed);
        }
    }

    fn render(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    fn event(&mut self, event: &mut Event) {
        self.total_events_processed += 1;
        
        match event.event_type {
            EventType::Keyboard => {
                if let Some(key_event) = event.as_key_event() {
                    let action_str = match key_event.action {
                        KeyAction::Press => "pressed",
                        KeyAction::Release => "released",
                        KeyAction::Repeat => "repeated",
                    };

                    info!("Keyboard Event: Key {:?} {} (mods: shift={}, ctrl={}, alt={})", 
                        key_event.key, 
                        action_str,
                        key_event.mods.shift,
                        key_event.mods.control,
                        key_event.mods.alt
                    );

                    if key_event.action == KeyAction::Press {
                        self.keys_pressed_this_frame.push(key_event.key);
                    }

                    // Handle special keys
                    match key_event.key {
                        KeyCode::R if key_event.action == KeyAction::Press => {
                            info!("Resetting counters!");
                            self.total_events_processed = 0;
                            self.frame_count = 0;
                            event.mark_handled();
                        }
                        KeyCode::Space if key_event.action == KeyAction::Press => {
                            info!("=== INPUT STATE INFO ===");
                            info!("Last mouse position: ({:.1}, {:.1})", 
                                self.last_mouse_position.0, self.last_mouse_position.1);
                            info!("Keys pressed this frame: {:?}", self.keys_pressed_this_frame);
                            info!("========================");
                            event.mark_handled();
                        }
                        KeyCode::Escape if key_event.action == KeyAction::Press => {
                            info!("Escape pressed - application will close");
                        }
                        _ => {}
                    }
                }
            }

            EventType::Mouse => {
                if let Some(move_event) = event.as_mouse_move_event() {
                    // Only log significant mouse movements to avoid spam
                    let distance = ((move_event.x - self.last_mouse_position.0).powi(2) + 
                                   (move_event.y - self.last_mouse_position.1).powi(2)).sqrt();
                    
                    if distance > 10.0 {
                        info!("Mouse moved to ({:.1}, {:.1})", move_event.x, move_event.y);
                    }
                    self.last_mouse_position = (move_event.x, move_event.y);
                } 
                else if let Some(button_event) = event.as_mouse_button_event() {
                    let action_str = match button_event.action {
                        KeyAction::Press => "pressed",
                        KeyAction::Release => "released",
                        KeyAction::Repeat => "repeated",
                    };
                    
                    let button_name = match button_event.button {
                        MouseButton::Left => "Left",
                        MouseButton::Right => "Right", 
                        MouseButton::Middle => "Middle",
                        MouseButton::Button1 => "Button1",
                        MouseButton::Button2 => "Button2",
                        MouseButton::Button3 => "Button3",
                        MouseButton::Button4 => "Button4",
                        MouseButton::Button5 => "Button5",
                        MouseButton::Button6 => "Button6",
                        MouseButton::Button7 => "Button7",
                        MouseButton::Button8 => "Button8",
                    };

                    info!("Mouse Button Event: {} button {} at ({:.1}, {:.1})", 
                        button_name, action_str, self.last_mouse_position.0, self.last_mouse_position.1);
                } 
                else if let Some(scroll_event) = event.as_mouse_scroll_event() {
                    info!("Mouse Scroll: x={:.1}, y={:.1}", scroll_event.x_offset, scroll_event.y_offset);
                }
            }

            EventType::Gamepad => {
                if let Some(button_event) = event.as_gamepad_button_event() {
                    let action_str = match button_event.action {
                        KeyAction::Press => "pressed",
                        KeyAction::Release => "released",
                        KeyAction::Repeat => "repeated",
                    };
                    info!("Gamepad Button Event: {:?} {} (gamepad {})", 
                        button_event.button, action_str, button_event.gamepad_id);
                }
                else if let Some(axis_event) = event.as_gamepad_axis_event() {
                    info!("Gamepad Axis Event: {:?} = {:.3} (gamepad {})", 
                        axis_event.axis, axis_event.value, axis_event.gamepad_id);
                }
                else if let Some(connection_event) = event.as_gamepad_connection_event() {
                    info!("Gamepad Connection Event: gamepad {} {}", 
                        connection_event.gamepad_id, 
                        if connection_event.connected { "connected" } else { "disconnected" });
                }
            }

            EventType::Window => {
                if let Some(resize_event) = event.as_window_resize_event() {
                    info!("Window resized to {}x{}", resize_event.width, resize_event.height);
                    
                    // Update OpenGL viewport
                    unsafe {
                        gl::Viewport(0, 0, resize_event.width as i32, resize_event.height as i32);
                    }
                } 
                else if let Some(move_event) = event.as_window_move_event() {
                    info!("Window moved to ({}, {})", move_event.x, move_event.y);
                } 
                else if let Some(_close_event) = event.as_window_close_event() {
                    info!("Window close event received!");
                }
            }

            EventType::Application => {
                if let Some(tick_event) = event.as_application_tick_event() {
                    // Don't log every tick as it would be too verbose
                    if self.frame_count % 600 == 0 { // Every ~10 seconds
                        info!("Application tick: delta_time = {:.3}s", tick_event.delta_time);
                    }
                }
            }

            EventType::Custom => {
                info!("Custom event received with timestamp: {}", event.timestamp);
            }
        }
    }

    fn shutdown(&mut self) {
        info!("Event System Demo shutting down");
        info!("Final stats: {} frames, {} total events processed", 
            self.frame_count, self.total_events_processed);
    }

    fn get_name(&self) -> &str {
        "Event System Demo"
    }
}

fn main() {
    // Initialize logging first
    if let Err(e) = artifice_logging::init_from_env() {
        eprintln!("Failed to initialize logger: {}", e);
        return;
    }

    info!("Starting Event System Demo");
    info!("This demo showcases the new event system architecture:");
    info!("- Type-safe event handling with pattern matching");
    info!("- Lock-free event queue for high performance");
    info!("- Centralized input management");
    info!("- Pluggable window backends");
    
    // Demonstrate creating an engine with a specific backend
    info!("Creating engine with GLFW backend...");
    
    // You could also create the engine like this to specify a backend:
    // let app = EventSystemDemoApp::new();
    // let mut engine = Engine::with_backend(app, "glfw");
    // engine.run();
    
    // But for simplicity, we'll use the convenience function
    run_application::<EventSystemDemoApp>();

    info!("Event System Demo completed");
}

// Example of how you might use the input manager directly
#[allow(dead_code)]
fn demonstrate_input_manager_usage() {
    use artifice_engine::input::InputManager;
    
    let mut input_manager = InputManager::with_queue_size(512);
    
    // You can query input state directly
    let keyboard = input_manager.keyboard();
    let mouse = input_manager.mouse();
    
    // Check if specific keys are pressed
    if keyboard.is_key_pressed(KeyCode::W) {
        info!("W key is currently pressed");
    }
    
    if keyboard.is_key_just_pressed(KeyCode::Space) {
        info!("Space key was just pressed this frame");
    }
    
    // Check mouse state
    let (mouse_x, mouse_y) = mouse.get_position();
    info!("Mouse position: ({:.1}, {:.1})", mouse_x, mouse_y);
    
    if mouse.is_button_pressed(MouseButton::Left) {
        info!("Left mouse button is currently pressed");
    }
    
    // Process any queued events
    let events = input_manager.process_events();
    info!("Processed {} events this frame", events.len());
    
    // Update input state (should be called once per frame)
    input_manager.update();
    
    // Get queue statistics
    let stats = input_manager.get_queue_stats();
    if stats.is_full {
        warn!("Input event queue is full!");
    }
}