use crate::event::{Event, EventType, KeyAction, KeyCode, KeyEvent, KeyMod};
use crate::io::InputDevice;
use logging::{debug, error, info, trace, warn};
use std::collections::{HashMap, HashSet};

/// Keyboard state tracking and input handling
pub struct Keyboard {
    key_states: HashMap<KeyCode, KeyAction>,
    pressed_keys: HashSet<KeyCode>,
    released_keys: HashSet<KeyCode>,
    key_mods: KeyMod,
    is_connected: bool,
}

impl Keyboard {
    pub fn new() -> Self {
        debug!("Creating keyboard input handler");
        Keyboard {
            key_states: HashMap::new(),
            pressed_keys: HashSet::new(),
            released_keys: HashSet::new(),
            key_mods: KeyMod::new(),
            is_connected: true,
        }
    }

    /// Process a key event and update internal state
    pub fn process_key_event(&mut self, key: KeyCode, action: KeyAction, mods: KeyMod) {
        // Update the key state
        self.key_states.insert(key, action);
        self.key_mods = mods;

        // Track pressed and released keys for this frame
        match action {
            KeyAction::Press => {
                self.pressed_keys.insert(key);
                trace!("Key pressed: {:?}", key);
            }
            KeyAction::Release => {
                self.released_keys.insert(key);
                trace!("Key released: {:?}", key);
            }
            _ => {}
        }
    }

    /// Check if a key is currently pressed
    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        matches!(
            self.key_states.get(&key),
            Some(KeyAction::Press) | Some(KeyAction::Repeat)
        )
    }

    /// Check if a key was just pressed this frame
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.pressed_keys.contains(&key)
    }

    /// Check if a key was just released this frame
    pub fn is_key_just_released(&self, key: KeyCode) -> bool {
        self.released_keys.contains(&key)
    }

    /// Get the current key modifiers state
    pub fn get_key_mods(&self) -> &KeyMod {
        &self.key_mods
    }

    /// Clear the per-frame state (called at the end of each frame)
    pub fn clear_frame_state(&mut self) {
        self.pressed_keys.clear();
        self.released_keys.clear();
    }
}

impl InputDevice for Keyboard {
    fn update(&mut self) {
        self.clear_frame_state();
    }

    fn is_connected(&self) -> bool {
        self.is_connected
    }
}

/// Helper functions for converting platform-specific key codes
pub mod key_translation {
    use crate::event::{KeyAction, KeyCode, KeyMod};
    use glfw::{Action, Key, Modifiers};

    /// Convert GLFW key to our abstracted KeyCode
    pub fn from_glfw_key(key: Key) -> KeyCode {
        match key {
            Key::Space => KeyCode::Space,
            Key::Apostrophe => KeyCode::Apostrophe,
            Key::Comma => KeyCode::Comma,
            Key::Minus => KeyCode::Minus,
            Key::Period => KeyCode::Period,
            Key::Slash => KeyCode::Slash,
            Key::Num0 => KeyCode::Num0,
            Key::Num1 => KeyCode::Num1,
            Key::Num2 => KeyCode::Num2,
            Key::Num3 => KeyCode::Num3,
            Key::Num4 => KeyCode::Num4,
            Key::Num5 => KeyCode::Num5,
            Key::Num6 => KeyCode::Num6,
            Key::Num7 => KeyCode::Num7,
            Key::Num8 => KeyCode::Num8,
            Key::Num9 => KeyCode::Num9,
            Key::Semicolon => KeyCode::Semicolon,
            Key::Equal => KeyCode::Equal,
            Key::A => KeyCode::A,
            Key::B => KeyCode::B,
            Key::C => KeyCode::C,
            Key::D => KeyCode::D,
            Key::E => KeyCode::E,
            Key::F => KeyCode::F,
            Key::G => KeyCode::G,
            Key::H => KeyCode::H,
            Key::I => KeyCode::I,
            Key::J => KeyCode::J,
            Key::K => KeyCode::K,
            Key::L => KeyCode::L,
            Key::M => KeyCode::M,
            Key::N => KeyCode::N,
            Key::O => KeyCode::O,
            Key::P => KeyCode::P,
            Key::Q => KeyCode::Q,
            Key::R => KeyCode::R,
            Key::S => KeyCode::S,
            Key::T => KeyCode::T,
            Key::U => KeyCode::U,
            Key::V => KeyCode::V,
            Key::W => KeyCode::W,
            Key::X => KeyCode::X,
            Key::Y => KeyCode::Y,
            Key::Z => KeyCode::Z,
            Key::LeftBracket => KeyCode::LeftBracket,
            Key::Backslash => KeyCode::Backslash,
            Key::RightBracket => KeyCode::RightBracket,
            Key::GraveAccent => KeyCode::GraveAccent,
            Key::Escape => KeyCode::Escape,
            Key::Enter => KeyCode::Enter,
            Key::Tab => KeyCode::Tab,
            Key::Backspace => KeyCode::Backspace,
            Key::Insert => KeyCode::Insert,
            Key::Delete => KeyCode::Delete,
            Key::Right => KeyCode::Right,
            Key::Left => KeyCode::Left,
            Key::Down => KeyCode::Down,
            Key::Up => KeyCode::Up,
            Key::PageUp => KeyCode::PageUp,
            Key::PageDown => KeyCode::PageDown,
            Key::Home => KeyCode::Home,
            Key::End => KeyCode::End,
            Key::CapsLock => KeyCode::CapsLock,
            Key::ScrollLock => KeyCode::ScrollLock,
            Key::NumLock => KeyCode::NumLock,
            Key::PrintScreen => KeyCode::PrintScreen,
            Key::Pause => KeyCode::Pause,
            Key::F1 => KeyCode::F1,
            Key::F2 => KeyCode::F2,
            Key::F3 => KeyCode::F3,
            Key::F4 => KeyCode::F4,
            Key::F5 => KeyCode::F5,
            Key::F6 => KeyCode::F6,
            Key::F7 => KeyCode::F7,
            Key::F8 => KeyCode::F8,
            Key::F9 => KeyCode::F9,
            Key::F10 => KeyCode::F10,
            Key::F11 => KeyCode::F11,
            Key::F12 => KeyCode::F12,
            Key::F13 => KeyCode::F13,
            Key::F14 => KeyCode::F14,
            Key::F15 => KeyCode::F15,
            Key::F16 => KeyCode::F16,
            Key::F17 => KeyCode::F17,
            Key::F18 => KeyCode::F18,
            Key::F19 => KeyCode::F19,
            Key::F20 => KeyCode::F20,
            Key::F21 => KeyCode::F21,
            Key::F22 => KeyCode::F22,
            Key::F23 => KeyCode::F23,
            Key::F24 => KeyCode::F24,
            Key::F25 => KeyCode::F25,
            Key::Kp0 => KeyCode::KP0,
            Key::Kp1 => KeyCode::KP1,
            Key::Kp2 => KeyCode::KP2,
            Key::Kp3 => KeyCode::KP3,
            Key::Kp4 => KeyCode::KP4,
            Key::Kp5 => KeyCode::KP5,
            Key::Kp6 => KeyCode::KP6,
            Key::Kp7 => KeyCode::KP7,
            Key::Kp8 => KeyCode::KP8,
            Key::Kp9 => KeyCode::KP9,
            Key::KpDecimal => KeyCode::KPDecimal,
            Key::KpDivide => KeyCode::KPDivide,
            Key::KpMultiply => KeyCode::KPMultiply,
            Key::KpSubtract => KeyCode::KPSubtract,
            Key::KpAdd => KeyCode::KPAdd,
            Key::KpEnter => KeyCode::KPEnter,
            Key::KpEqual => KeyCode::KPEqual,
            Key::LeftShift => KeyCode::LeftShift,
            Key::LeftControl => KeyCode::LeftControl,
            Key::LeftAlt => KeyCode::LeftAlt,
            Key::LeftSuper => KeyCode::LeftSuper,
            Key::RightShift => KeyCode::RightShift,
            Key::RightControl => KeyCode::RightControl,
            Key::RightAlt => KeyCode::RightAlt,
            Key::RightSuper => KeyCode::RightSuper,
            Key::Menu => KeyCode::Menu,
            _ => KeyCode::Unknown,
        }
    }

    // Convert GLFW key action to our abstracted KeyAction
    pub fn from_glfw_action(action: Action) -> KeyAction {
        match action {
            Action::Press => KeyAction::Press,
            Action::Release => KeyAction::Release,
            Action::Repeat => KeyAction::Repeat,
        }
    }

    // Convert GLFW key mods to our abstracted KeyMod
    pub fn from_glfw_mods(mods: Modifiers) -> KeyMod {
        KeyMod {
            shift: mods.contains(Modifiers::Shift),
            control: mods.contains(Modifiers::Control),
            alt: mods.contains(Modifiers::Alt),
            super_key: mods.contains(Modifiers::Super),
            caps_lock: mods.contains(Modifiers::CapsLock),
            num_lock: mods.contains(Modifiers::NumLock),
        }
    }
}
