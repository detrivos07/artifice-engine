use crate::event::{MouseButton, KeyAction, KeyMod, MouseMoveEvent, MouseButtonEvent, MouseScrollEvent, Event, EventType};
use crate::io::InputDevice;
use std::collections::{HashMap, HashSet};

/// Mouse state tracking and input handling
pub struct Mouse {
    button_states: HashMap<MouseButton, KeyAction>,
    pressed_buttons: HashSet<MouseButton>,
    released_buttons: HashSet<MouseButton>,
    position: (f64, f64),
    previous_position: (f64, f64),
    movement: (f64, f64),
    scroll_offset: (f64, f64),
    is_connected: bool,
}

impl Mouse {
    pub fn new() -> Self {
        Mouse {
            button_states: HashMap::new(),
            pressed_buttons: HashSet::new(),
            released_buttons: HashSet::new(),
            position: (0.0, 0.0),
            previous_position: (0.0, 0.0),
            movement: (0.0, 0.0),
            scroll_offset: (0.0, 0.0),
            is_connected: true,
        }
    }

    /// Process a mouse button event and update internal state
    pub fn process_button_event(&mut self, button: MouseButton, action: KeyAction, mods: KeyMod) {
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

    /// Process a mouse move event and update position
    pub fn process_move_event(&mut self, x: f64, y: f64) {
        self.previous_position = self.position;
        self.position = (x, y);
        self.movement = (
            self.position.0 - self.previous_position.0,
            self.position.1 - self.previous_position.1
        );
    }

    /// Process a mouse scroll event
    pub fn process_scroll_event(&mut self, x_offset: f64, y_offset: f64) {
        self.scroll_offset = (x_offset, y_offset);
    }

    /// Check if a mouse button is currently pressed
    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        matches!(self.button_states.get(&button), Some(KeyAction::Press) | Some(KeyAction::Repeat))
    }

    /// Check if a mouse button was just pressed this frame
    pub fn is_button_just_pressed(&self, button: MouseButton) -> bool {
        self.pressed_buttons.contains(&button)
    }

    /// Check if a mouse button was just released this frame
    pub fn is_button_just_released(&self, button: MouseButton) -> bool {
        self.released_buttons.contains(&button)
    }

    /// Get the current mouse position
    pub fn get_position(&self) -> (f64, f64) {
        self.position
    }

    /// Get the mouse movement since last update
    pub fn get_movement(&self) -> (f64, f64) {
        self.movement
    }

    /// Get the current scroll offset
    pub fn get_scroll_offset(&self) -> (f64, f64) {
        self.scroll_offset
    }

    /// Clear the per-frame state (called at the end of each frame)
    pub fn clear_frame_state(&mut self) {
        self.pressed_buttons.clear();
        self.released_buttons.clear();
        self.movement = (0.0, 0.0);
        self.scroll_offset = (0.0, 0.0);
    }
}

impl InputDevice for Mouse {
    fn update(&mut self) {
        self.clear_frame_state();
    }

    fn is_connected(&self) -> bool {
        self.is_connected
    }
}

/// Helper functions for converting platform-specific mouse inputs
pub mod mouse_translation {
    use super::MouseButton;
    use glfw::MouseButton as GlfwMouseButton;

    /// Convert GLFW mouse button to our abstracted MouseButton
    pub fn from_glfw_button(button: GlfwMouseButton) -> MouseButton {
        match button {
            GlfwMouseButton::Button1 => MouseButton::Left,
            GlfwMouseButton::Button2 => MouseButton::Right,
            GlfwMouseButton::Button3 => MouseButton::Middle,
            GlfwMouseButton::Button4 => MouseButton::Button4,
            GlfwMouseButton::Button5 => MouseButton::Button5,
            GlfwMouseButton::Button6 => MouseButton::Button6,
            GlfwMouseButton::Button7 => MouseButton::Button7,
            GlfwMouseButton::Button8 => MouseButton::Button8,
        }
    }
}