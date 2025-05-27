use artifice_engine::{
    Engine, Application, Layer,
    events::{
        Event, EventData, EventType, EventFilter, EventFilterManager,
        EventTypeFilter, PredicateFilter, CustomEventData,
        KeyEvent, KeyCode, KeyAction, MouseButtonEvent, MouseButton,
        GamepadButtonEvent, GamepadButton
    },
    io::{
        MetricsConfig, MetricsCollector
    },
    input::{
        InputRecorder, InputPlayer, GamepadManager
    },
    window::{
        HotReloadConfig, HotReloadBuilder, WindowFeature
    }
};
use artifice_logging::{info, debug, warn};
use std::time::Duration;
use std::sync::Arc;

// Custom event types for our demo
#[derive(Debug, Clone)]
struct PlayerScoreEvent {
    player_id: u32,
    score: i32,
    timestamp: f64,
}

#[derive(Debug, Clone)]
struct GameStateEvent {
    state: String,
    data: String,
}

#[derive(Debug, Clone)]
struct NetworkEvent {
    message_type: String,
    payload: Vec<u8>,
}

/// Demo application showcasing all new engine features
struct ComprehensiveDemo {
    // Feature demonstration states
    demo_phase: DemoPhase,
    phase_timer: f32,
    
    // Input recording
    input_recorder: Option<InputRecorder>,
    input_player: Option<InputPlayer>,
    recording_active: bool,
    
    // Event filtering
    filter_mode: FilterMode,
    
    // Backend switching
    current_backend: String,
    backend_switch_cooldown: f32,
    
    // Metrics
    metrics_enabled: bool,
    last_metrics_report: f32,
    
    // Game state
    player_score: i32,
    game_objects: Vec<GameObject>,
}

#[derive(Debug, Clone, PartialEq)]
enum DemoPhase {
    Introduction,
    EventFiltering,
    InputRecording,
    GamepadDemo,
    CustomEvents,
    BackendSwitching,
    MetricsDisplay,
    NetworkSimulation,
    Conclusion,
}

#[derive(Debug, Clone)]
enum FilterMode {
    None,
    KeyboardOnly,
    MouseOnly,
    GamepadOnly,
    CustomPredicate,
}

#[derive(Debug, Clone)]
struct GameObject {
    x: f32,
    y: f32,
    velocity_x: f32,
    velocity_y: f32,
    color: (f32, f32, f32),
}

impl Application for ComprehensiveDemo {
    fn new() -> Self {
        info!("=== Artifice Engine Comprehensive Demo ===");
        info!("This demo showcases all new engine features:");
        info!("- Custom event types");
        info!("- Event filtering system");
        info!("- Input recording/playback");
        info!("- Gamepad support");
        info!("- Hot reload backend switching");
        info!("- Performance metrics");
        info!("- Wayland backend support");
        info!("==========================================");

        Self {
            demo_phase: DemoPhase::Introduction,
            phase_timer: 0.0,
            input_recorder: None,
            input_player: None,
            recording_active: false,
            filter_mode: FilterMode::None,
            current_backend: "glfw".to_string(),
            backend_switch_cooldown: 0.0,
            metrics_enabled: true,
            last_metrics_report: 0.0,
            player_score: 0,
            game_objects: vec![
                GameObject {
                    x: 100.0, y: 100.0,
                    velocity_x: 50.0, velocity_y: 30.0,
                    color: (1.0, 0.0, 0.0),
                },
                GameObject {
                    x: 200.0, y: 150.0,
                    velocity_x: -30.0, velocity_y: 40.0,
                    color: (0.0, 1.0, 0.0),
                },
                GameObject {
                    x: 300.0, y: 200.0,
                    velocity_x: 20.0, velocity_y: -60.0,
                    color: (0.0, 0.0, 1.0),
                },
            ],
        }
    }

    fn init(&mut self) {
        info!("Demo initialized - starting introduction phase");
    }

    fn update(&mut self, delta_time: f32) {
        self.phase_timer += delta_time;
        self.backend_switch_cooldown = (self.backend_switch_cooldown - delta_time).max(0.0);
        self.last_metrics_report += delta_time;

        // Auto-advance demo phases
        match self.demo_phase {
            DemoPhase::Introduction if self.phase_timer > 5.0 => {
                self.advance_to_phase(DemoPhase::EventFiltering);
            }
            DemoPhase::EventFiltering if self.phase_timer > 10.0 => {
                self.advance_to_phase(DemoPhase::InputRecording);
            }
            DemoPhase::InputRecording if self.phase_timer > 15.0 => {
                self.advance_to_phase(DemoPhase::GamepadDemo);
            }
            DemoPhase::GamepadDemo if self.phase_timer > 20.0 => {
                self.advance_to_phase(DemoPhase::CustomEvents);
            }
            DemoPhase::CustomEvents if self.phase_timer > 25.0 => {
                self.advance_to_phase(DemoPhase::BackendSwitching);
            }
            DemoPhase::BackendSwitching if self.phase_timer > 30.0 => {
                self.advance_to_phase(DemoPhase::MetricsDisplay);
            }
            DemoPhase::MetricsDisplay if self.phase_timer > 35.0 => {
                self.advance_to_phase(DemoPhase::NetworkSimulation);
            }
            DemoPhase::NetworkSimulation if self.phase_timer > 40.0 => {
                self.advance_to_phase(DemoPhase::Conclusion);
            }
            _ => {}
        }

        // Update game objects
        for obj in &mut self.game_objects {
            obj.x += obj.velocity_x * delta_time;
            obj.y += obj.velocity_y * delta_time;

            // Bounce off screen edges
            if obj.x < 0.0 || obj.x > 800.0 {
                obj.velocity_x = -obj.velocity_x;
                obj.x = obj.x.clamp(0.0, 800.0);
            }
            if obj.y < 0.0 || obj.y > 600.0 {
                obj.velocity_y = -obj.velocity_y;
                obj.y = obj.y.clamp(0.0, 600.0);
            }
        }

        // Periodic metrics reporting
        if self.metrics_enabled && self.last_metrics_report > 5.0 {
            info!("=== Periodic Metrics Report ===");
            self.last_metrics_report = 0.0;
        }
    }

    fn event(&mut self, event: &mut Event) {
        match event.event_type {
            EventType::Keyboard => {
                if let Some(key_event) = event.as_key_event() {
                    let key_event_clone = key_event.clone();
                    self.handle_keyboard_event(&key_event_clone, event);
                }
            }
            EventType::Mouse => {
                if let Some(mouse_event) = event.as_mouse_button_event() {
                    let mouse_event_clone = mouse_event.clone();
                    self.handle_mouse_event(&mouse_event_clone, event);
                }
            }
            EventType::Gamepad => {
                if let Some(gamepad_event) = event.as_gamepad_button_event() {
                    let gamepad_event_clone = gamepad_event.clone();
                    self.handle_gamepad_event(&gamepad_event_clone, event);
                }
            }
            EventType::Custom => {
                if let Some(custom_event) = event.as_custom_event() {
                    let custom_event_clone = custom_event.clone();
                    self.handle_custom_event(&custom_event_clone, event);
                }
            }
            _ => {}
        }
    }

    fn get_name(&self) -> &str {
        "Comprehensive Engine Demo"
    }
}

impl ComprehensiveDemo {
    fn advance_to_phase(&mut self, new_phase: DemoPhase) {
        info!("=== Demo Phase: {:?} ===", new_phase);
        self.demo_phase = new_phase.clone();
        self.phase_timer = 0.0;

        match new_phase {
            DemoPhase::Introduction => {
                info!("Welcome! This demo will showcase all engine features.");
                info!("Watch the console for detailed information about each feature.");
            }
            DemoPhase::EventFiltering => {
                info!("EVENT FILTERING DEMO");
                info!("Demonstrating different event filter modes:");
                info!("- Keyboard only filter");
                info!("- Mouse only filter"); 
                info!("- Custom predicate filters");
                self.filter_mode = FilterMode::KeyboardOnly;
            }
            DemoPhase::InputRecording => {
                info!("INPUT RECORDING DEMO");
                info!("Starting input recording...");
                self.start_input_recording();
            }
            DemoPhase::GamepadDemo => {
                info!("GAMEPAD DEMO");
                info!("Connect a gamepad to see gamepad input handling!");
                info!("Gamepad events will be logged below.");
            }
            DemoPhase::CustomEvents => {
                info!("CUSTOM EVENTS DEMO");
                info!("Generating custom application events...");
                self.generate_custom_events();
            }
            DemoPhase::BackendSwitching => {
                info!("BACKEND SWITCHING DEMO");
                info!("Demonstrating hot reload backend switching...");
                info!("Note: Wayland backend requires Linux with Wayland compositor");
            }
            DemoPhase::MetricsDisplay => {
                info!("METRICS DEMO");
                info!("Displaying performance metrics and statistics...");
            }
            DemoPhase::NetworkSimulation => {
                info!("NETWORK EVENTS DEMO");
                info!("Simulating network events using custom event types...");
                self.simulate_network_events();
            }
            DemoPhase::Conclusion => {
                info!("DEMO COMPLETE");
                info!("All engine features have been demonstrated!");
                info!("Check the console output for detailed information.");
                info!("Press ESC to exit the demo.");
            }
        }
    }

    fn handle_keyboard_event(&mut self, key_event: &KeyEvent, event: &mut Event) {
        if key_event.action == KeyAction::Press {
            match key_event.key {
                KeyCode::Escape => {
                    info!("ESC pressed - requesting application shutdown");
                    // In a real engine, this would signal shutdown
                }
                KeyCode::Space => {
                    info!("SPACE pressed - toggling input recording");
                    self.toggle_input_recording();
                }
                KeyCode::F1 => {
                    info!("F1 pressed - switching event filter mode");
                    self.cycle_filter_mode();
                }
                KeyCode::F2 => {
                    info!("F2 pressed - attempting backend switch");
                    self.attempt_backend_switch();
                }
                KeyCode::F3 => {
                    info!("F3 pressed - generating custom events");
                    self.generate_custom_events();
                }
                KeyCode::F4 => {
                    info!("F4 pressed - toggling metrics");
                    self.metrics_enabled = !self.metrics_enabled;
                    info!("Metrics {}", if self.metrics_enabled { "enabled" } else { "disabled" });
                }
                KeyCode::Num1 | KeyCode::Num2 | KeyCode::Num3 | KeyCode::Num4 | KeyCode::Num5 | 
                KeyCode::Num6 | KeyCode::Num7 | KeyCode::Num8 | KeyCode::Num9 => {
                    let score_increment = match key_event.key {
                        KeyCode::Num1 => 1,
                        KeyCode::Num2 => 2,
                        KeyCode::Num3 => 3,
                        KeyCode::Num4 => 4,
                        KeyCode::Num5 => 5,
                        KeyCode::Num6 => 6,
                        KeyCode::Num7 => 7,
                        KeyCode::Num8 => 8,
                        KeyCode::Num9 => 9,
                        _ => 1, // fallback, shouldn't happen
                    };
                    self.player_score += score_increment;
                    info!("Score increased by {} - Total: {}", score_increment, self.player_score);
                    
                    // Generate custom score event
                    let score_event = PlayerScoreEvent {
                        player_id: 1,
                        score: self.player_score,
                        timestamp: self.phase_timer as f64,
                    };
                    self.send_custom_event("PlayerScore", score_event);
                }
                _ => {
                    debug!("Key pressed: {:?} (modifiers: shift={}, ctrl={}, alt={})", 
                           key_event.key, key_event.mods.shift, key_event.mods.control, key_event.mods.alt);
                }
            }
        }
        event.mark_handled();
    }

    fn handle_mouse_event(&mut self, mouse_event: &MouseButtonEvent, event: &mut Event) {
        if mouse_event.action == KeyAction::Press {
            match mouse_event.button {
                MouseButton::Left => {
                    info!("Left mouse button clicked");
                    self.spawn_game_object();
                }
                MouseButton::Right => {
                    info!("Right mouse button clicked - advancing demo phase");
                    self.advance_to_next_phase();
                }
                MouseButton::Middle => {
                    info!("Middle mouse button clicked - resetting demo");
                    self.reset_demo();
                }
                _ => {
                    debug!("Mouse button {:?} pressed", mouse_event.button);
                }
            }
        }
        event.mark_handled();
    }

    fn handle_gamepad_event(&mut self, gamepad_event: &GamepadButtonEvent, event: &mut Event) {
        if gamepad_event.action == KeyAction::Press {
            info!("Gamepad {} button {:?} pressed", gamepad_event.gamepad_id, gamepad_event.button);
            
            match gamepad_event.button {
                GamepadButton::A => {
                    info!("Gamepad A button - jumping!");
                    self.player_score += 10;
                }
                GamepadButton::B => {
                    info!("Gamepad B button - attacking!");
                    self.spawn_game_object();
                }
                GamepadButton::X => {
                    info!("Gamepad X button - using item!");
                }
                GamepadButton::Y => {
                    info!("Gamepad Y button - special ability!");
                    self.generate_custom_events();
                }
                GamepadButton::Start => {
                    info!("Gamepad Start - pausing game");
                    self.send_custom_event("GameState", GameStateEvent {
                        state: "paused".to_string(),
                        data: "user_requested".to_string(),
                    });
                }
                _ => {}
            }
        }
        event.mark_handled();
    }

    fn handle_custom_event(&mut self, custom_event: &CustomEventData, event: &mut Event) {
        info!("Received custom event: {}", custom_event.type_name());
        
        match custom_event.type_name() {
            "PlayerScore" => {
                if let Some(score_event) = custom_event.get_data::<PlayerScoreEvent>() {
                    info!("Player {} scored {} points at time {:.2}", 
                          score_event.player_id, score_event.score, score_event.timestamp);
                }
            }
            "GameState" => {
                if let Some(state_event) = custom_event.get_data::<GameStateEvent>() {
                    info!("Game state changed to: {} ({})", state_event.state, state_event.data);
                }
            }
            "Network" => {
                if let Some(network_event) = custom_event.get_data::<NetworkEvent>() {
                    info!("Network event: {} ({} bytes)", 
                          network_event.message_type, network_event.payload.len());
                }
            }
            _ => {
                info!("Unknown custom event type: {}", custom_event.type_name());
            }
        }
        event.mark_handled();
    }

    fn toggle_input_recording(&mut self) {
        if self.recording_active {
            self.stop_input_recording();
        } else {
            self.start_input_recording();
        }
    }

    fn start_input_recording(&mut self) {
        if !self.recording_active {
            let mut recorder = InputRecorder::new("demo_recording")
                .with_description("Comprehensive demo input recording");
            recorder.start_recording();
            self.input_recorder = Some(recorder);
            self.recording_active = true;
            info!("Input recording started");
        }
    }

    fn stop_input_recording(&mut self) {
        if let Some(mut recorder) = self.input_recorder.take() {
            let recording = recorder.finish();
            info!("Input recording stopped - {} events recorded over {:.2}s", 
                  recording.metadata.event_count, 
                  recording.metadata.duration_ms as f32 / 1000.0);
            
            // Create player for playback
            let player = InputPlayer::new(recording);
            self.input_player = Some(player);
            self.recording_active = false;
        }
    }

    fn cycle_filter_mode(&mut self) {
        self.filter_mode = match self.filter_mode {
            FilterMode::None => FilterMode::KeyboardOnly,
            FilterMode::KeyboardOnly => FilterMode::MouseOnly,
            FilterMode::MouseOnly => FilterMode::GamepadOnly,
            FilterMode::GamepadOnly => FilterMode::CustomPredicate,
            FilterMode::CustomPredicate => FilterMode::None,
        };
        
        info!("Event filter mode changed to: {:?}", self.filter_mode);
        // Note: In a real implementation, you would update the engine's filter manager here
    }

    fn attempt_backend_switch(&mut self) {
        if self.backend_switch_cooldown > 0.0 {
            info!("Backend switch on cooldown, please wait...");
            return;
        }

        let target_backend = match self.current_backend.as_str() {
            "glfw" => "wayland",
            "wayland" => "glfw",
            _ => "glfw",
        };

        info!("Attempting to switch from {} to {} backend", self.current_backend, target_backend);
        
        // Note: In a real implementation, you would call engine.switch_backend(target_backend)
        // For demo purposes, we'll simulate the switch
        self.current_backend = target_backend.to_string();
        self.backend_switch_cooldown = 3.0; // 3 second cooldown
        
        info!("Backend switch to {} completed (simulated)", target_backend);
    }

    fn generate_custom_events(&mut self) {
        info!("Generating sample custom events...");
        
        // Player score event
        let score_event = PlayerScoreEvent {
            player_id: 42,
            score: 1337,
            timestamp: self.phase_timer as f64,
        };
        self.send_custom_event("PlayerScore", score_event);

        // Game state event
        let state_event = GameStateEvent {
            state: "level_complete".to_string(),
            data: "level_3_boss_defeated".to_string(),
        };
        self.send_custom_event("GameState", state_event);
    }

    fn simulate_network_events(&mut self) {
        info!("Simulating network events...");
        
        let events = vec![
            NetworkEvent {
                message_type: "player_join".to_string(),
                payload: vec![1, 2, 3, 4, 5],
            },
            NetworkEvent {
                message_type: "chat_message".to_string(),
                payload: "Hello, world!".as_bytes().to_vec(),
            },
            NetworkEvent {
                message_type: "game_update".to_string(),
                payload: vec![0xFF; 64], // Simulate larger payload
            },
        ];

        for network_event in events {
            self.send_custom_event("Network", network_event);
        }
    }

    fn send_custom_event<T: Send + Sync + 'static>(&self, type_name: &str, data: T) {
        let custom_data = CustomEventData::new(type_name, data);
        let event = Event::new(EventData::Custom(custom_data));
        // Note: In a real implementation, you would send this through the engine's event system
        debug!("Custom event '{}' would be sent to engine", type_name);
    }

    fn spawn_game_object(&mut self) {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        
        let new_object = GameObject {
            x: rng.gen_range(0.0..800.0),
            y: rng.gen_range(0.0..600.0),
            velocity_x: rng.gen_range(-100.0..100.0),
            velocity_y: rng.gen_range(-100.0..100.0),
            color: (rng.gen(), rng.gen(), rng.gen()),
        };
        
        self.game_objects.push(new_object);
        info!("Spawned new game object - total: {}", self.game_objects.len());
    }

    fn advance_to_next_phase(&mut self) {
        let next_phase = match self.demo_phase {
            DemoPhase::Introduction => DemoPhase::EventFiltering,
            DemoPhase::EventFiltering => DemoPhase::InputRecording,
            DemoPhase::InputRecording => DemoPhase::GamepadDemo,
            DemoPhase::GamepadDemo => DemoPhase::CustomEvents,
            DemoPhase::CustomEvents => DemoPhase::BackendSwitching,
            DemoPhase::BackendSwitching => DemoPhase::MetricsDisplay,
            DemoPhase::MetricsDisplay => DemoPhase::NetworkSimulation,
            DemoPhase::NetworkSimulation => DemoPhase::Conclusion,
            DemoPhase::Conclusion => DemoPhase::Introduction,
        };
        self.advance_to_phase(next_phase);
    }

    fn reset_demo(&mut self) {
        info!("Resetting demo to beginning...");
        self.advance_to_phase(DemoPhase::Introduction);
        self.player_score = 0;
        self.game_objects.clear();
        self.spawn_game_object(); // Start with one object
        self.filter_mode = FilterMode::None;
        self.metrics_enabled = true;
    }
}

/// Demo layer that shows additional functionality
struct MetricsOverlayLayer {
    update_timer: f32,
}

impl Layer for MetricsOverlayLayer {
    fn attach(&mut self) {
        info!("Metrics overlay layer attached");
    }

    fn update(&mut self, delta_time: f32) {
        self.update_timer += delta_time;
        
        // Update metrics display every 2 seconds
        if self.update_timer > 2.0 {
            self.update_timer = 0.0;
            // In a real implementation, you would update UI here
            debug!("Metrics overlay updated");
        }
    }

    fn event(&mut self, event: &mut Event) {
        // Layer can intercept and modify events
        if let EventData::Key(key_event) = &event.data {
            if key_event.key == KeyCode::F12 && key_event.action == KeyAction::Press {
                info!("F12 pressed in overlay layer - toggling debug mode");
                event.mark_handled(); // Prevent further processing
            }
        }
    }

    fn get_name(&self) -> &str {
        "Metrics Overlay"
    }
}

fn main() {
    // Initialize logging
    artifice_logging::init().expect("Failed to initialize logging");

    info!("Starting Comprehensive Engine Demo");
    info!("Controls:");
    info!("  ESC     - Exit demo");
    info!("  SPACE   - Toggle input recording");
    info!("  F1      - Cycle event filter modes");
    info!("  F2      - Switch window backend");
    info!("  F3      - Generate custom events");
    info!("  F4      - Toggle metrics");
    info!("  1-9     - Increase score");
    info!("  LMB     - Spawn object");
    info!("  RMB     - Next demo phase");
    info!("  MMB     - Reset demo");

    // Create application
    let app = ComprehensiveDemo::new();
    
    // Configure engine with metrics and hot reload
    let metrics_config = MetricsConfig {
        enabled: true,
        auto_reporting: true,
        report_interval: Duration::from_secs(10),
        max_event_types: 50,
    };
    
    let hot_reload_config = HotReloadConfig {
        switch_timeout: Duration::from_secs(3),
        preserve_state: true,
        buffer_events: true,
        max_buffered_events: 500,
        validate_backend: true,
    };

    // Create engine with configuration
    let mut engine = Engine::with_config(app, "glfw", metrics_config, hot_reload_config);
    
    // Add demo layer
    let overlay_layer = Box::new(MetricsOverlayLayer {
        update_timer: 0.0,
    });
    engine.push_layer(overlay_layer);

    // Set up event filters based on demo requirements
    let filter_manager = engine.get_event_filter_manager_mut();
    
    // Example: Add a filter that blocks F5 key (reserved for engine)
    let f5_block_filter = PredicateFilter::new("block_f5", |event| {
        if let Some(key_event) = event.as_key_event() {
            key_event.key != KeyCode::F5
        } else {
            true
        }
    });
    filter_manager.add_filter(Box::new(f5_block_filter));

    // Example: Add a filter for performance-sensitive scenarios
    let performance_filter = EventTypeFilter::new("performance_mode", vec![
        EventType::Keyboard,
        EventType::Mouse,
        EventType::Gamepad,
        EventType::Application,
    ]).with_priority(10);
    filter_manager.add_filter(Box::new(performance_filter));

    // Enable metrics
    engine.set_metrics_enabled(true);

    info!("Engine configured and ready - starting main loop");

    // Run the application
    engine.run();

    info!("Demo completed - goodbye!");
}