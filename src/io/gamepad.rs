use crate::event::{GamepadButton, GamepadAxis, KeyAction, GamepadButtonEvent, GamepadAxisEvent, Event, EventType};
use crate::io::InputDevice;
use std::collections::{HashMap, HashSet};

/// Represents a connected gamepad/controller
pub struct Gamepad {
    id: u32,
    name: String,
    button_states: HashMap<GamepadButton, KeyAction>,
    pressed_buttons: HashSet<GamepadButton>,
    released_buttons: HashSet<GamepadButton>,
    axis_values: HashMap<GamepadAxis, f32>,
    is_connected: bool,
}

impl Gamepad {
    pub fn new(id: u32, name: String) -> Self {
        Gamepad {
            id,
            name,
            button_states: HashMap::new(),
            pressed_buttons: HashSet::new(),
            released_buttons: HashSet::new(),
            axis_values: HashMap::new(),
            is_connected: true,
        }
    }

    /// Get the gamepad ID
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Get the gamepad name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Process a gamepad button event and update internal state
    pub fn process_button_event(&mut self, button: GamepadButton, action: KeyAction) {
        // Update the button state
        self.button_states.insert(button, action);

        // Track pressed and released buttons for this frame
        match action {
            KeyAction::Press => {
                self.pressed_buttons.insert(button);
            },
            KeyAction::Release => {
                self.released_buttons.insert(button);
            },
            _ => {}
        }
    }

    /// Process a gamepad axis event and update axis value
    pub fn process_axis_event(&mut self, axis: GamepadAxis, value: f32) {
        self.axis_values.insert(axis, value);
    }

    /// Check if a gamepad button is currently pressed
    pub fn is_button_pressed(&self, button: GamepadButton) -> bool {
        matches!(self.button_states.get(&button), Some(KeyAction::Press) | Some(KeyAction::Repeat))
    }

    /// Check if a gamepad button was just pressed this frame
    pub fn is_button_just_pressed(&self, button: GamepadButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    /// Check if a gamepad button was just released this frame
    pub fn is_button_just_released(&self, button: GamepadButton) -> bool {
        self.released_buttons.contains(&button)
    }

    /// Get the current value of a gamepad axis
    pub fn get_axis_value(&self, axis: GamepadAxis) -> f32 {
        *self.axis_values.get(&axis).unwrap_or(&0.0)
    }

    /// Clear the per-frame state (called at the end of each frame)
    pub fn clear_frame_state(&mut self) {
        self.pressed_buttons.clear();
        self.released_buttons.clear();
    }

    /// Set the connection state of the gamepad
    pub fn set_connected(&mut self, connected: bool) {
        self.is_connected = connected;
    }
}

impl InputDevice for Gamepad {
    fn update(&mut self) {
        self.clear_frame_state();
    }

    fn is_connected(&self) -> bool {
        self.is_connected
    }
}

/// Manager for multiple gamepads
pub struct GamepadManager {
    gamepads: HashMap<u32, Gamepad>,
}

impl GamepadManager {
    pub fn new() -> Self {
        GamepadManager {
            gamepads: HashMap::new(),
        }
    }

    /// Register a new gamepad
    pub fn register_gamepad(&mut self, id: u32, name: String) {
        self.gamepads.insert(id, Gamepad::new(id, name));
    }

    /// Remove a gamepad
    pub fn remove_gamepad(&mut self, id: u32) {
        self.gamepads.remove(&id);
    }

    /// Get a gamepad by ID
    pub fn get_gamepad(&self, id: u32) -> Option<&Gamepad> {
        self.gamepads.get(&id)
    }

    /// Get a mutable reference to a gamepad by ID
    pub fn get_gamepad_mut(&mut self, id: u32) -> Option<&mut Gamepad> {
        self.gamepads.get_mut(&id)
    }

    /// Get all connected gamepads
    pub fn get_connected_gamepads(&self) -> Vec<&Gamepad> {
        self.gamepads.values().filter(|g| g.is_connected()).collect()
    }

    /// Update all gamepads
    pub fn update(&mut self) {
        for gamepad in self.gamepads.values_mut() {
            gamepad.update();
        }
    }
}

/// Helper functions for converting platform-specific gamepad inputs
pub mod gamepad_translation {
    use super::{GamepadButton, GamepadAxis};

    /// Convert GLFW joystick button to our abstracted GamepadButton
    pub fn button_from_glfw(button: i32) -> GamepadButton {
        match button {
            0 => GamepadButton::A,
            1 => GamepadButton::B,
            2 => GamepadButton::X,
            3 => GamepadButton::Y,
            4 => GamepadButton::LeftBumper,
            5 => GamepadButton::RightBumper,
            6 => GamepadButton::Back,
            7 => GamepadButton::Start,
            8 => GamepadButton::Guide,
            9 => GamepadButton::LeftThumb,
            10 => GamepadButton::RightThumb,
            11 => GamepadButton::DPadUp,
            12 => GamepadButton::DPadRight,
            13 => GamepadButton::DPadDown,
            14 => GamepadButton::DPadLeft,
            _ => GamepadButton::Unknown(button as u8),
        }
    }

    /// Convert GLFW joystick axis to our abstracted GamepadAxis
    pub fn axis_from_glfw(axis: i32) -> GamepadAxis {
        match axis {
            0 => GamepadAxis::LeftX,
            1 => GamepadAxis::LeftY,
            2 => GamepadAxis::RightX,
            3 => GamepadAxis::RightY,
            4 => GamepadAxis::LeftTrigger,
            5 => GamepadAxis::RightTrigger,
            _ => GamepadAxis::Unknown(axis as u8),
        }
    }
}