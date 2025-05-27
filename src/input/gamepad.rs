use crate::events::core::{KeyAction, KeyMod};
use crate::input::InputDevice;
use artifice_logging::{debug, trace, warn};
use std::collections::HashMap;

/// Standard gamepad buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    // Face buttons (Xbox layout names)
    A,              // Bottom face button
    B,              // Right face button  
    X,              // Left face button
    Y,              // Top face button
    
    // Shoulder buttons
    LeftBumper,     // L1/LB
    RightBumper,    // R1/RB
    LeftTrigger,    // L2/LT (digital)
    RightTrigger,   // R2/RT (digital)
    
    // D-pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    
    // Special buttons
    Start,          // Start/Menu/Options
    Select,         // Back/View/Share
    Guide,          // Xbox/PS/Home button
    
    // Stick buttons
    LeftStick,      // L3
    RightStick,     // R3
    
    // Additional buttons for extended controllers
    Paddle1,
    Paddle2,
    Paddle3,
    Paddle4,
    
    // Generic buttons for non-standard controllers
    Button16,
    Button17,
    Button18,
    Button19,
    Button20,
}

/// Gamepad analog axes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadAxis {
    // Left stick
    LeftStickX,
    LeftStickY,
    
    // Right stick  
    RightStickX,
    RightStickY,
    
    // Triggers (analog)
    LeftTriggerAnalog,   // L2/LT analog value
    RightTriggerAnalog,  // R2/RT analog value
    
    // Additional axes for extended controllers
    Axis6,
    Axis7,
    Axis8,
    Axis9,
    Axis10,
    Axis11,
}

/// Represents a gamepad input event
#[derive(Debug, Clone)]
pub struct GamepadButtonEvent {
    pub gamepad_id: u32,
    pub button: GamepadButton,
    pub action: KeyAction,
    pub mods: KeyMod,
}

#[derive(Debug, Clone)]
pub struct GamepadAxisEvent {
    pub gamepad_id: u32,
    pub axis: GamepadAxis,
    pub value: f32,
}

#[derive(Debug, Clone)]
pub struct GamepadConnectionEvent {
    pub gamepad_id: u32,
    pub connected: bool,
    pub name: String,
}

/// State of a single gamepad
#[derive(Debug, Clone)]
pub struct GamepadState {
    pub id: u32,
    pub name: String,
    pub connected: bool,
    button_states: HashMap<GamepadButton, bool>,
    button_just_pressed: HashMap<GamepadButton, bool>,
    button_just_released: HashMap<GamepadButton, bool>,
    axis_values: HashMap<GamepadAxis, f32>,
    deadzone: f32,
}

impl GamepadState {
    pub fn new(id: u32, name: String) -> Self {
        Self {
            id,
            name,
            connected: true,
            button_states: HashMap::new(),
            button_just_pressed: HashMap::new(),
            button_just_released: HashMap::new(),
            axis_values: HashMap::new(),
            deadzone: 0.1, // Default deadzone
        }
    }

    /// Set the deadzone for analog sticks (0.0 to 1.0)
    pub fn set_deadzone(&mut self, deadzone: f32) {
        self.deadzone = deadzone.clamp(0.0, 1.0);
    }

    /// Get the current deadzone value
    pub fn deadzone(&self) -> f32 {
        self.deadzone
    }

    /// Check if a button is currently pressed
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        self.button_states.get(&button).copied().unwrap_or(false)
    }

    /// Check if a button was just pressed this frame
    pub fn is_button_just_pressed(&self, button: GamepadButton) -> bool {
        self.button_just_pressed.get(&button).copied().unwrap_or(false)
    }

    /// Check if a button was just released this frame
    pub fn is_button_just_released(&self, button: GamepadButton) -> bool {
        self.button_just_released.get(&button).copied().unwrap_or(false)
    }

    /// Get the current value of an analog axis (-1.0 to 1.0)
    pub fn axis_value(&self, axis: GamepadAxis) -> f32 {
        let value = self.axis_values.get(&axis).copied().unwrap_or(0.0);
        
        // Apply deadzone
        if value.abs() < self.deadzone {
            0.0
        } else {
            // Scale the value to account for deadzone
            let sign = value.signum();
            let scaled = (value.abs() - self.deadzone) / (1.0 - self.deadzone);
            sign * scaled.clamp(0.0, 1.0)
        }
    }

    /// Get the raw axis value without deadzone applied
    pub fn raw_axis_value(&self, axis: GamepadAxis) -> f32 {
        self.axis_values.get(&axis).copied().unwrap_or(0.0)
    }

    /// Get left stick as a 2D vector (x, y)
    pub fn left_stick(&self) -> (f32, f32) {
        (
            self.axis_value(GamepadAxis::LeftStickX),
            self.axis_value(GamepadAxis::LeftStickY),
        )
    }

    /// Get right stick as a 2D vector (x, y)
    pub fn right_stick(&self) -> (f32, f32) {
        (
            self.axis_value(GamepadAxis::RightStickX),
            self.axis_value(GamepadAxis::RightStickY),
        )
    }

    /// Get trigger values (0.0 to 1.0)
    pub fn triggers(&self) -> (f32, f32) {
        (
            self.axis_value(GamepadAxis::LeftTriggerAnalog).max(0.0),
            self.axis_value(GamepadAxis::RightTriggerAnalog).max(0.0),
        )
    }

    /// Check if left stick is pressed beyond deadzone
    pub fn is_left_stick_active(&self) -> bool {
        let (x, y) = self.left_stick();
        (x * x + y * y).sqrt() > 0.0
    }

    /// Check if right stick is pressed beyond deadzone
    pub fn is_right_stick_active(&self) -> bool {
        let (x, y) = self.right_stick();
        (x * x + y * y).sqrt() > 0.0
    }

    /// Process a button event
    pub fn process_button_event(&mut self, button: GamepadButton, action: KeyAction, _mods: KeyMod) {
        let was_pressed = self.is_button_pressed(button);
        let is_pressed = match action {
            KeyAction::Press => true,
            KeyAction::Release => false,
            KeyAction::Repeat => return, // Ignore repeat for gamepads
        };

        self.button_states.insert(button, is_pressed);

        if is_pressed && !was_pressed {
            self.button_just_pressed.insert(button, true);
            trace!("Gamepad {} button {:?} pressed", self.id, button);
        } else if !is_pressed && was_pressed {
            self.button_just_released.insert(button, true);
            trace!("Gamepad {} button {:?} released", self.id, button);
        }
    }

    /// Process an axis event
    pub fn process_axis_event(&mut self, axis: GamepadAxis, value: f32) {
        let clamped_value = value.clamp(-1.0, 1.0);
        self.axis_values.insert(axis, clamped_value);
        trace!("Gamepad {} axis {:?} = {:.3}", self.id, axis, clamped_value);
    }

    /// Update state for new frame (clear just_pressed/just_released flags)
    pub fn update(&mut self) {
        self.button_just_pressed.clear();
        self.button_just_released.clear();
    }

    /// Disconnect the gamepad
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.button_states.clear();
        self.button_just_pressed.clear();
        self.button_just_released.clear();
        self.axis_values.clear();
        debug!("Gamepad {} disconnected", self.id);
    }
}

/// Manages multiple gamepad inputs
pub struct GamepadManager {
    gamepads: HashMap<u32, GamepadState>,
    connected: bool,
}

impl GamepadManager {
    pub fn new() -> Self {
        Self {
            gamepads: HashMap::new(),
            connected: true,
        }
    }

    /// Get a gamepad by ID
    pub fn gamepad(&self, id: u32) -> Option<&GamepadState> {
        self.gamepads.get(&id)
    }

    /// Get a mutable reference to a gamepad by ID
    pub fn gamepad_mut(&mut self, id: u32) -> Option<&mut GamepadState> {
        self.gamepads.get_mut(&id)
    }

    /// Get the first connected gamepad
    pub fn primary_gamepad(&self) -> Option<&GamepadState> {
        self.gamepads
            .values()
            .find(|gamepad| gamepad.connected)
    }

    /// Get all connected gamepad IDs
    pub fn connected_gamepad_ids(&self) -> Vec<u32> {
        self.gamepads
            .iter()
            .filter(|(_, gamepad)| gamepad.connected)
            .map(|(id, _)| *id)
            .collect()
    }

    /// Get the number of connected gamepads
    pub fn connected_count(&self) -> usize {
        self.gamepads
            .values()
            .filter(|gamepad| gamepad.connected)
            .count()
    }

    /// Check if any gamepad has a button pressed
    pub fn any_button_pressed(&self, button: GamepadButton) -> bool {
        self.gamepads
            .values()
            .filter(|gamepad| gamepad.connected)
            .any(|gamepad| gamepad.is_button_pressed(button))
    }

    /// Check if any gamepad has a button just pressed
    pub fn any_button_just_pressed(&self, button: GamepadButton) -> bool {
        self.gamepads
            .values()
            .filter(|gamepad| gamepad.connected)
            .any(|gamepad| gamepad.is_button_just_pressed(button))
    }

    /// Process a gamepad connection event
    pub fn process_connection_event(&mut self, id: u32, connected: bool, name: String) {
        if connected {
            let gamepad = GamepadState::new(id, name.clone());
            self.gamepads.insert(id, gamepad);
            debug!("Gamepad {} connected: {}", id, name);
        } else {
            if let Some(gamepad) = self.gamepads.get_mut(&id) {
                gamepad.disconnect();
            }
            debug!("Gamepad {} disconnected", id);
        }
    }

    /// Process a gamepad button event
    pub fn process_button_event(&mut self, id: u32, button: GamepadButton, action: KeyAction, mods: KeyMod) {
        if let Some(gamepad) = self.gamepads.get_mut(&id) {
            if gamepad.connected {
                gamepad.process_button_event(button, action, mods);
            }
        } else {
            warn!("Received button event for unknown gamepad: {}", id);
        }
    }

    /// Process a gamepad axis event
    pub fn process_axis_event(&mut self, id: u32, axis: GamepadAxis, value: f32) {
        if let Some(gamepad) = self.gamepads.get_mut(&id) {
            if gamepad.connected {
                gamepad.process_axis_event(axis, value);
            }
        } else {
            warn!("Received axis event for unknown gamepad: {}", id);
        }
    }

    /// Set deadzone for all connected gamepads
    pub fn set_global_deadzone(&mut self, deadzone: f32) {
        for gamepad in self.gamepads.values_mut() {
            gamepad.set_deadzone(deadzone);
        }
        debug!("Set global gamepad deadzone to {:.3}", deadzone);
    }

    /// Set deadzone for a specific gamepad
    pub fn set_gamepad_deadzone(&mut self, id: u32, deadzone: f32) {
        if let Some(gamepad) = self.gamepads.get_mut(&id) {
            gamepad.set_deadzone(deadzone);
            debug!("Set gamepad {} deadzone to {:.3}", id, deadzone);
        }
    }

    /// Remove disconnected gamepads from memory
    pub fn cleanup_disconnected(&mut self) {
        let disconnected_ids: Vec<u32> = self.gamepads
            .iter()
            .filter(|(_, gamepad)| !gamepad.connected)
            .map(|(id, _)| *id)
            .collect();

        for id in disconnected_ids {
            self.gamepads.remove(&id);
            debug!("Cleaned up disconnected gamepad: {}", id);
        }
    }

    /// Get gamepad info for debugging
    pub fn get_gamepad_info(&self) -> Vec<(u32, String, bool)> {
        self.gamepads
            .iter()
            .map(|(id, gamepad)| (*id, gamepad.name.clone(), gamepad.connected))
            .collect()
    }
}

impl InputDevice for GamepadManager {
    fn update(&mut self) {
        for gamepad in self.gamepads.values_mut() {
            gamepad.update();
        }
    }

    fn is_connected(&self) -> bool {
        self.connected && self.connected_count() > 0
    }
}

impl Default for GamepadManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Convenience functions for common gamepad button mappings
impl GamepadButton {
    /// Get all face buttons
    pub fn face_buttons() -> [GamepadButton; 4] {
        [GamepadButton::A, GamepadButton::B, GamepadButton::X, GamepadButton::Y]
    }

    /// Get all shoulder buttons
    pub fn shoulder_buttons() -> [GamepadButton; 4] {
        [
            GamepadButton::LeftBumper,
            GamepadButton::RightBumper,
            GamepadButton::LeftTrigger,
            GamepadButton::RightTrigger,
        ]
    }

    /// Get all D-pad buttons
    pub fn dpad_buttons() -> [GamepadButton; 4] {
        [
            GamepadButton::DPadUp,
            GamepadButton::DPadDown,
            GamepadButton::DPadLeft,
            GamepadButton::DPadRight,
        ]
    }

    /// Check if this is a D-pad button
    pub fn is_dpad(&self) -> bool {
        matches!(
            self,
            GamepadButton::DPadUp | GamepadButton::DPadDown | GamepadButton::DPadLeft | GamepadButton::DPadRight
        )
    }

    /// Check if this is a face button
    pub fn is_face_button(&self) -> bool {
        matches!(
            self,
            GamepadButton::A | GamepadButton::B | GamepadButton::X | GamepadButton::Y
        )
    }

    /// Check if this is a shoulder button
    pub fn is_shoulder_button(&self) -> bool {
        matches!(
            self,
            GamepadButton::LeftBumper | GamepadButton::RightBumper | GamepadButton::LeftTrigger | GamepadButton::RightTrigger
        )
    }
}

impl GamepadAxis {
    /// Get all stick axes
    pub fn stick_axes() -> [GamepadAxis; 4] {
        [
            GamepadAxis::LeftStickX,
            GamepadAxis::LeftStickY,
            GamepadAxis::RightStickX,
            GamepadAxis::RightStickY,
        ]
    }

    /// Get trigger axes
    pub fn trigger_axes() -> [GamepadAxis; 2] {
        [GamepadAxis::LeftTriggerAnalog, GamepadAxis::RightTriggerAnalog]
    }

    /// Check if this is a stick axis
    pub fn is_stick_axis(&self) -> bool {
        matches!(
            self,
            GamepadAxis::LeftStickX | GamepadAxis::LeftStickY | GamepadAxis::RightStickX | GamepadAxis::RightStickY
        )
    }

    /// Check if this is a trigger axis
    pub fn is_trigger_axis(&self) -> bool {
        matches!(self, GamepadAxis::LeftTriggerAnalog | GamepadAxis::RightTriggerAnalog)
    }

    /// Check if this is a left stick axis
    pub fn is_left_stick(&self) -> bool {
        matches!(self, GamepadAxis::LeftStickX | GamepadAxis::LeftStickY)
    }

    /// Check if this is a right stick axis
    pub fn is_right_stick(&self) -> bool {
        matches!(self, GamepadAxis::RightStickX | GamepadAxis::RightStickY)
    }
}