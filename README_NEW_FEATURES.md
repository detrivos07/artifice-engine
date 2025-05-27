# Artifice Engine - New Features Documentation

## Overview

This document describes the major new features and improvements added to the Artifice Engine. All backwards compatibility code has been removed in favor of cleaner, more performant implementations.

## Table of Contents

- [Breaking Changes](#breaking-changes)
- [Custom Event Types](#custom-event-types)
- [Event Filtering System](#event-filtering-system)
- [Input Recording & Playback](#input-recording--playback)
- [Gamepad Support](#gamepad-support)
- [Window Backend System](#window-backend-system)
- [Hot Reload (Backend Switching)](#hot-reload-backend-switching)
- [Performance Metrics](#performance-metrics)
- [Extensibility Points](#extensibility-points)
- [Examples](#examples)
- [Migration Guide](#migration-guide)

## Breaking Changes

### Removed Legacy Code

- **`Event::get_data<T>()`** method has been removed
- Use type-safe accessor methods instead: `event.as_key_event()`, `event.as_mouse_move_event()`, etc.
- All unsafe transmutation code has been eliminated

### Before (Removed):
```rust
if let Some(key_event) = event.get_data::<KeyEvent>() {
    // Handle key event
}
```

### After (Required):
```rust
if let Some(key_event) = event.as_key_event() {
    // Handle key event - type-safe and more efficient
}
```

## Custom Event Types

The engine now supports user-defined custom event types through a type-safe system.

### Creating Custom Events

```rust
use artifice_engine::event::{Event, EventData, CustomEventData};

// Define your custom event type
#[derive(Debug, Clone)]
struct PlayerScoreEvent {
    player_id: u32,
    score: i32,
    timestamp: f64,
}

// Create and send the event
let score_event = PlayerScoreEvent {
    player_id: 1,
    score: 100,
    timestamp: 123.45,
};

let custom_data = CustomEventData::new("PlayerScore", score_event);
let event = Event::new(EventData::Custom(custom_data));
```

### Handling Custom Events

```rust
fn handle_event(&mut self, event: &mut Event) {
    if let Some(custom_event) = event.as_custom_event() {
        match custom_event.type_name() {
            "PlayerScore" => {
                if let Some(score_event) = custom_event.get_data::<PlayerScoreEvent>() {
                    println!("Player {} scored {} points", score_event.player_id, score_event.score);
                }
            }
            _ => {}
        }
    }
}
```

## Event Filtering System

Configurable event filters allow for fine-grained control over which events are processed.

### Filter Types

#### Event Type Filter
```rust
use artifice_engine::event::{EventTypeFilter, EventType};

// Allow only keyboard and mouse events
let filter = EventTypeFilter::new("input_only", vec![
    EventType::Keyboard,
    EventType::Mouse,
]).with_priority(10);

engine.get_event_filter_manager_mut().add_filter(Box::new(filter));
```

#### Predicate Filter
```rust
use artifice_engine::event::PredicateFilter;

// Block all F5 key presses
let filter = PredicateFilter::new("block_f5", |event| {
    if let Some(key_event) = event.as_key_event() {
        key_event.key != KeyCode::F5
    } else {
        true
    }
});

engine.get_event_filter_manager_mut().add_filter(Box::new(filter));
```

#### Block Filter
```rust
use artifice_engine::event::EventTypeBlockFilter;

// Block all window events
let filter = EventTypeBlockFilter::new("no_window_events", vec![
    EventType::Window,
]);

engine.get_event_filter_manager_mut().add_filter(Box::new(filter));
```

### Managing Filters

```rust
let filter_manager = engine.get_event_filter_manager_mut();

// Add filter
filter_manager.add_filter(Box::new(my_filter));

// Remove filter by name
filter_manager.remove_filter("filter_name");

// Clear all filters
filter_manager.clear_filters();

// Enable/disable filtering
filter_manager.set_enabled(false);
```

## Input Recording & Playback

Record and replay input sequences for testing, demos, or gameplay features.

### Recording Input

```rust
use artifice_engine::io::{InputRecorder, InputRecordingManager};

// Create and start a recorder
let mut recorder = InputRecorder::new("my_recording")
    .with_description("Demo recording for testing");
recorder.start_recording();

// In your event handler
recorder.record_event(&event);

// Stop recording
let recording = recorder.finish();
recording.save_to_file("my_recording.json").unwrap();
```

### Playing Back Input

```rust
use artifice_engine::io::InputPlayer;

// Load and play a recording
let mut player = InputPlayer::load_from_file("my_recording.json").unwrap();
player.set_playback_speed(1.5); // 1.5x speed
player.set_loop_playback(true);
player.start_playback();

// In your update loop
let events = player.get_current_events();
for event in events {
    // Process replayed events
}
```

### Recording Manager

```rust
use artifice_engine::io::InputRecordingManager;

let mut manager = InputRecordingManager::new();

// Start recording
manager.start_recording("test_sequence");

// Record events
manager.record_event(&event);

// Stop and get recording
let recording = manager.stop_recording();

// Start playback
manager.start_playback("test_sequence");

// Get playback events
let events = manager.get_playback_events();
```

## Gamepad Support

Full gamepad/controller input support with multiple device management.

### Basic Gamepad Usage

```rust
// Get gamepad manager
let gamepad_manager = engine.get_gamepad_manager();

// Check if any gamepads are connected
if gamepad_manager.connected_count() > 0 {
    // Get primary gamepad
    if let Some(gamepad) = gamepad_manager.primary_gamepad() {
        // Check button states
        if gamepad.is_button_pressed(GamepadButton::A) {
            println!("A button is held down");
        }
        
        if gamepad.is_button_just_pressed(GamepadButton::Start) {
            println!("Start button was just pressed");
        }
        
        // Get analog stick values
        let (left_x, left_y) = gamepad.left_stick();
        let (right_x, right_y) = gamepad.right_stick();
        
        // Get trigger values
        let (left_trigger, right_trigger) = gamepad.triggers();
    }
}
```

### Gamepad Events

```rust
fn handle_gamepad_events(&mut self, event: &Event) {
    if let Some(button_event) = event.as_gamepad_button_event() {
        println!("Gamepad {} button {:?} {:?}", 
                 button_event.gamepad_id, 
                 button_event.button, 
                 button_event.action);
    }
    
    if let Some(axis_event) = event.as_gamepad_axis_event() {
        println!("Gamepad {} axis {:?} = {:.3}", 
                 axis_event.gamepad_id, 
                 axis_event.axis, 
                 axis_event.value);
    }
    
    if let Some(connection_event) = event.as_gamepad_connection_event() {
        if connection_event.connected {
            println!("Gamepad {} connected: {}", 
                     connection_event.gamepad_id, 
                     connection_event.name);
        } else {
            println!("Gamepad {} disconnected", connection_event.gamepad_id);
        }
    }
}
```

### Multiple Gamepads

```rust
// Get all connected gamepad IDs
let gamepad_ids = gamepad_manager.connected_gamepad_ids();

for id in gamepad_ids {
    if let Some(gamepad) = gamepad_manager.gamepad(id) {
        // Process each gamepad individually
        if gamepad.is_button_just_pressed(GamepadButton::A) {
            println!("Player {} pressed A", id);
        }
    }
}

// Check if any gamepad has a button pressed
if gamepad_manager.any_button_pressed(GamepadButton::Start) {
    println!("Someone pressed Start");
}
```

## Window Backend System

Pluggable window backend system supporting multiple implementations.

### Available Backends

- **GLFW** - Cross-platform OpenGL library (default)
- **Wayland** - Linux Wayland compositor support
- **Extensible** - Easy to add new backends

### Using Specific Backends

```rust
// Create engine with specific backend
let app = MyApplication::new();
let engine = Engine::with_backend(app, "wayland");

// Or use the registry directly
use artifice_engine::io::{WindowBackendRegistry, WindowFeature};

let mut registry = WindowBackendRegistry::new();
let backends = registry.available_backends();
println!("Available backends: {:?}", backends);

// Check backend capabilities
if registry.backend_supports_feature("wayland", WindowFeature::HighDPI) {
    println!("Wayland supports high DPI");
}

// Create window with specific backend
let window = registry.create_window("wayland", 800, 600, "My Window")
    .expect("Failed to create Wayland window");
```

### Backend Information

```rust
let registry = WindowBackendRegistry::new();

for backend_name in registry.available_backends() {
    if let Some(info) = registry.get_backend_info(backend_name) {
        println!("Backend: {} v{:?}", info.name, info.version);
        println!("Features: {:?}", info.supported_features);
    }
}
```

## Hot Reload (Backend Switching)

Switch window backends at runtime without losing application state.

### Basic Backend Switching

```rust
// Switch to a different backend
match engine.switch_backend("wayland") {
    Ok(()) => println!("Switched to Wayland successfully"),
    Err(e) => println!("Failed to switch: {}", e),
}
```

### Advanced Hot Reload

```rust
use artifice_engine::io::{HotReloadBuilder, HotReloadConfig};

// Configure hot reload behavior
let result = HotReloadBuilder::new("wayland")
    .preserve_state(true)
    .buffer_events(true)
    .timeout(Duration::from_secs(5))
    .validate_backend(true)
    .execute(engine.get_hot_reload_manager_mut(), engine.get_window());

match result {
    Ok(()) => println!("Hot reload initiated"),
    Err(e) => println!("Hot reload failed: {}", e),
}
```

### Hot Reload Manager

```rust
let hot_reload_manager = engine.get_hot_reload_manager_mut();

// Check available backends
let backends = hot_reload_manager.available_backends();
println!("Can switch to: {:?}", backends);

// Validate backend before switching
if hot_reload_manager.validate_backend("wayland") {
    // Start reload process
    hot_reload_manager.start_reload("wayland", engine.get_window())?;
    
    // Create new window
    let new_window = hot_reload_manager.create_window_with_backend(
        "wayland", 800, 600, "My App", &[]
    )?;
    
    // Complete the reload
    let result = hot_reload_manager.complete_reload("wayland", new_window.as_mut());
    println!("Reload result: {:?}", result);
}
```

## Performance Metrics

Built-in performance monitoring and metrics collection.

### Basic Metrics

```rust
// Enable metrics
engine.set_metrics_enabled(true);

// Get current metrics
if let Some(metrics) = engine.get_metrics() {
    println!("Events processed: {}", metrics.events_processed);
    println!("Events per second: {:.2}", metrics.events_per_second);
    println!("Average processing time: {:.2}μs", metrics.avg_processing_time_us);
    println!("Queue utilization: {:.1}%", metrics.queue_utilization * 100.0);
}

// Generate a metrics report
engine.report_metrics();
```

### Advanced Metrics Configuration

```rust
use artifice_engine::io::{MetricsConfig, MetricsFactory};
use std::time::Duration;

let metrics_config = MetricsConfig {
    enabled: true,
    auto_reporting: true,
    report_interval: Duration::from_secs(30),
    max_event_types: 100,
};

let engine = Engine::with_config(app, "glfw", metrics_config, HotReloadConfig::default());
```

### Custom Metrics Collection

```rust
use artifice_engine::io::{MetricsCollector, MetricsTimer};

let collector = MetricsCollector::new();
let handle = collector.get_handle();

// Manual timing
let timer = MetricsTimer::new(handle.clone(), "custom_operation");
// ... do work ...
timer.finish(); // Records the elapsed time

// Or automatic timing (records on drop)
{
    let _timer = MetricsTimer::new(handle, "auto_operation");
    // ... do work ...
} // Timer automatically records when dropped
```

## Extensibility Points

### Adding Custom Input Devices

```rust
use artifice_engine::io::InputDevice;

struct CustomInputDevice {
    connected: bool,
    // ... device-specific fields
}

impl InputDevice for CustomInputDevice {
    fn update(&mut self) {
        // Update device state
    }
    
    fn is_connected(&self) -> bool {
        self.connected
    }
}

// Add to input manager
let mut custom_device = CustomInputDevice { connected: true };
// Integration would require extending InputManager
```

### Custom Window Backends

```rust
use artifice_engine::io::{Window, WindowFactory, WindowFeature};

struct MyCustomWindow {
    // Window implementation
}

impl Window for MyCustomWindow {
    // Implement all required methods
    fn update(&mut self) { /* ... */ }
    fn process_events(&mut self) { /* ... */ }
    // ... other methods
}

struct MyCustomWindowFactory;

impl WindowFactory for MyCustomWindowFactory {
    fn create_window(&self, width: u32, height: u32, title: &str) -> Box<dyn Window> {
        Box::new(MyCustomWindow::new(width, height, title))
    }
    
    fn supports_feature(&self, feature: WindowFeature) -> bool {
        // Return supported features
        matches!(feature, WindowFeature::OpenGL | WindowFeature::MultiWindow)
    }
    
    fn backend_name(&self) -> &str {
        "MyCustomBackend"
    }
}

// Register with the engine
let mut registry = WindowBackendRegistry::new();
registry.register_factory("custom".to_string(), Box::new(MyCustomWindowFactory));
```

### Network Events

```rust
// Define network event types
#[derive(Debug, Clone)]
struct NetworkEvent {
    message_type: String,
    payload: Vec<u8>,
    source: std::net::SocketAddr,
}

// Send network events through the custom event system
let network_event = NetworkEvent {
    message_type: "player_update".to_string(),
    payload: vec![1, 2, 3, 4],
    source: "127.0.0.1:8080".parse().unwrap(),
};

let custom_data = CustomEventData::new("Network", network_event);
let event = Event::new(EventData::Custom(custom_data));
```

## Examples

### Complete Application Example

```rust
use artifice_engine::{Engine, Application, event::*};

struct MyApp {
    score: i32,
}

impl Application for MyApp {
    fn new() -> Self {
        Self { score: 0 }
    }
    
    fn event(&mut self, event: &mut Event) {
        match &event.data {
            EventData::Key(key_event) => {
                if key_event.key == KeyCode::Space && key_event.action == KeyAction::Press {
                    self.score += 10;
                    println!("Score: {}", self.score);
                }
            }
            EventData::GamepadButton(gamepad_event) => {
                if gamepad_event.button == GamepadButton::A && gamepad_event.action == KeyAction::Press {
                    self.score += 5;
                    println!("Gamepad score: {}", self.score);
                }
            }
            EventData::Custom(custom_event) => {
                if custom_event.type_name() == "ResetScore" {
                    self.score = 0;
                    println!("Score reset!");
                }
            }
            _ => {}
        }
    }
    
    fn get_name(&self) -> &str {
        "My Awesome Game"
    }
}

fn main() {
    let app = MyApp::new();
    let mut engine = Engine::new(app);
    
    // Enable all features
    engine.set_metrics_enabled(true);
    
    // Add event filters
    let filter = EventTypeFilter::new("gameplay", vec![
        EventType::Keyboard,
        EventType::Gamepad,
        EventType::Custom,
    ]);
    engine.get_event_filter_manager_mut().add_filter(Box::new(filter));
    
    engine.run();
}
```

### Input Recording Example

```rust
use artifice_engine::io::*;

fn main() {
    let mut recording_manager = InputRecordingManager::new();
    
    // Start recording
    recording_manager.start_recording("player_actions");
    
    // In your event loop:
    // recording_manager.record_event(&event);
    
    // Stop and save
    if let Some(recording) = recording_manager.stop_recording() {
        recording.save_to_file("player_actions.json").unwrap();
        println!("Recorded {} events", recording.metadata.event_count);
    }
    
    // Later, load and play back
    recording_manager.load_recording_from_file("player_actions.json").unwrap();
    recording_manager.start_playback("player_actions");
    
    // In your update loop:
    let playback_events = recording_manager.get_playback_events();
    for event in playback_events {
        // Process replayed events
    }
}
```

## Migration Guide

### From Legacy Event System

**Old:**
```rust
fn event(&mut self, event: &mut Event) {
    if let Some(key_event) = event.get_data::<KeyEvent>() {
        // Handle key event
    }
}
```

**New:**
```rust
fn event(&mut self, event: &mut Event) {
    if let Some(key_event) = event.as_key_event() {
        // Handle key event - more efficient and type-safe
    }
}
```

### Engine Creation

**Old:**
```rust
let engine = Engine::new(app);
```

**New (Enhanced):**
```rust
// Basic usage (same as before)
let engine = Engine::new(app);

// With specific backend
let engine = Engine::with_backend(app, "wayland");

// With full configuration
let metrics_config = MetricsConfig::default();
let hot_reload_config = HotReloadConfig::default();
let engine = Engine::with_config(app, "glfw", metrics_config, hot_reload_config);
```

### Best Practices

1. **Use type-safe event accessors** instead of legacy `get_data()`
2. **Enable metrics** for performance monitoring
3. **Use event filters** to optimize performance in high-frequency scenarios
4. **Record input sequences** for testing and debugging
5. **Validate backends** before switching to prevent runtime errors
6. **Use custom events** for application-specific event types

### Performance Considerations

- Event filtering can significantly improve performance in scenarios with high event frequency
- The lock-free event queue provides better throughput than the old channel-based system
- Metrics collection has minimal overhead when disabled
- Hot reload preserves application state but has a brief transition period

## Conclusion

These new features provide a solid foundation for building sophisticated applications with the Artifice Engine. The removal of legacy code ensures better performance and maintainability, while the new extensibility points allow for easy customization and expansion.

For more examples and detailed API documentation, see the `examples/` directory and the generated rustdoc documentation.