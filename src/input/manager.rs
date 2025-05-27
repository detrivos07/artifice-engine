use std::sync::Arc;
use crate::events::core::{Event, EventData, EventQueue};
use crate::input::{InputDevice, keyboard::Keyboard, mouse::Mouse, gamepad::GamepadManager};
use crate::events::{GamepadButton, GamepadAxis};
use artifice_logging::{debug, trace, warn};

/// Centralized input state manager
pub struct InputManager {
    keyboard: Keyboard,
    mouse: Mouse,
    gamepad: GamepadManager,
    event_queue: Arc<EventQueue>,
}

impl InputManager {
    pub fn new() -> Self {
        debug!("Creating input manager");
        InputManager {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            gamepad: GamepadManager::new(),
            event_queue: Arc::new(EventQueue::new(1024)), // Configurable size
        }
    }
    
    pub fn with_queue_size(queue_size: usize) -> Self {
        debug!("Creating input manager with queue size: {}", queue_size);
        InputManager {
            keyboard: Keyboard::new(),
            mouse: Mouse::new(),
            gamepad: GamepadManager::new(),
            event_queue: Arc::new(EventQueue::new(queue_size)),
        }
    }
    
    /// Process all queued events and update input device states
    pub fn process_events(&mut self) -> Vec<Event> {
        let mut processed_events = Vec::new();
        
        // Process all queued events
        while let Some(event) = self.event_queue.try_pop() {
            match &event.data {
                EventData::Key(key_event) => {
                    self.keyboard.process_key_event(
                        key_event.key,
                        key_event.action,
                        key_event.mods.clone(),
                    );
                    trace!("Processed key event: {:?}", key_event);
                }
                EventData::MouseMove(move_event) => {
                    self.mouse.process_move_event(move_event.x, move_event.y);
                    trace!("Processed mouse move event: ({:.1}, {:.1})", move_event.x, move_event.y);
                }
                EventData::MouseButton(button_event) => {
                    self.mouse.process_button_event(
                        button_event.button,
                        button_event.action,
                        button_event.mods.clone(),
                    );
                    trace!("Processed mouse button event: {:?}", button_event);
                }
                EventData::MouseScroll(scroll_event) => {
                    self.mouse.process_scroll_event(scroll_event.x_offset, scroll_event.y_offset);
                    trace!("Processed mouse scroll event: ({:.1}, {:.1})", scroll_event.x_offset, scroll_event.y_offset);
                }
                EventData::GamepadButton(button_event) => {
                    // Convert event gamepad types to internal gamepad types
                    let internal_button = self.convert_gamepad_button(button_event.button);
                    self.gamepad.process_button_event(
                        button_event.gamepad_id,
                        internal_button,
                        button_event.action,
                        button_event.mods.clone(),
                    );
                    trace!("Processed gamepad button event: {:?}", button_event);
                }
                EventData::GamepadAxis(axis_event) => {
                    // Convert event gamepad types to internal gamepad types  
                    let internal_axis = self.convert_gamepad_axis(axis_event.axis);
                    self.gamepad.process_axis_event(
                        axis_event.gamepad_id,
                        internal_axis,
                        axis_event.value,
                    );
                    trace!("Processed gamepad axis event: {:?}", axis_event);
                }
                EventData::GamepadConnection(connection_event) => {
                    self.gamepad.process_connection_event(
                        connection_event.gamepad_id,
                        connection_event.connected,
                        connection_event.name.clone(),
                    );
                    trace!("Processed gamepad connection event: {:?}", connection_event);
                }
                _ => {
                    // Other events (window, application) pass through unchanged
                    trace!("Passing through non-input event: {:?}", event.event_type);
                }
            }
            
            processed_events.push(event);
        }
        
        processed_events
    }
    
    /// Update all input devices (should be called once per frame)
    pub fn update(&mut self) {
        self.keyboard.update();
        self.mouse.update();
        self.gamepad.update();
    }
    
    /// Get a reference to the event queue for external event producers
    pub fn get_event_queue(&self) -> Arc<EventQueue> {
        self.event_queue.clone()
    }
    
    /// Get a reference to the keyboard
    pub fn keyboard(&self) -> &Keyboard {
        &self.keyboard
    }
    
    /// Get a reference to the mouse
    pub fn mouse(&self) -> &Mouse {
        &self.mouse
    }
    
    /// Get a reference to the gamepad manager
    pub fn gamepad(&self) -> &GamepadManager {
        &self.gamepad
    }
    
    /// Get a mutable reference to the keyboard (for advanced usage)
    pub fn keyboard_mut(&mut self) -> &mut Keyboard {
        &mut self.keyboard
    }
    
    /// Get a mutable reference to the mouse (for advanced usage)
    pub fn mouse_mut(&mut self) -> &mut Mouse {
        &mut self.mouse
    }
    
    /// Get a mutable reference to the gamepad manager (for advanced usage)
    pub fn gamepad_mut(&mut self) -> &mut GamepadManager {
        &mut self.gamepad
    }
    
    /// Check if the input manager is ready to receive events
    pub fn is_ready(&self) -> bool {
        self.keyboard.is_connected() && self.mouse.is_connected() && self.gamepad.is_connected()
    }
    
    /// Get statistics about the event queue
    pub fn get_queue_stats(&self) -> InputQueueStats {
        InputQueueStats {
            is_empty: self.event_queue.is_empty(),
            is_full: self.event_queue.is_full(),
        }
    }

    /// Convert event gamepad button to internal gamepad button
    fn convert_gamepad_button(&self, button: crate::events::GamepadButton) -> crate::input::gamepad::GamepadButton {
        use crate::events::GamepadButton as EventButton;
        use crate::input::gamepad::GamepadButton as InternalButton;
        
        match button {
            EventButton::A => InternalButton::A,
            EventButton::B => InternalButton::B,
            EventButton::X => InternalButton::X,
            EventButton::Y => InternalButton::Y,
            EventButton::LeftBumper => InternalButton::LeftBumper,
            EventButton::RightBumper => InternalButton::RightBumper,
            EventButton::LeftTrigger => InternalButton::LeftTrigger,
            EventButton::RightTrigger => InternalButton::RightTrigger,
            EventButton::DPadUp => InternalButton::DPadUp,
            EventButton::DPadDown => InternalButton::DPadDown,
            EventButton::DPadLeft => InternalButton::DPadLeft,
            EventButton::DPadRight => InternalButton::DPadRight,
            EventButton::Start => InternalButton::Start,
            EventButton::Select => InternalButton::Select,
            EventButton::Guide => InternalButton::Guide,
            EventButton::LeftStick => InternalButton::LeftStick,
            EventButton::RightStick => InternalButton::RightStick,
            EventButton::Paddle1 => InternalButton::Paddle1,
            EventButton::Paddle2 => InternalButton::Paddle2,
            EventButton::Paddle3 => InternalButton::Paddle3,
            EventButton::Paddle4 => InternalButton::Paddle4,
            EventButton::Button16 => InternalButton::Button16,
            EventButton::Button17 => InternalButton::Button17,
            EventButton::Button18 => InternalButton::Button18,
            EventButton::Button19 => InternalButton::Button19,
            EventButton::Button20 => InternalButton::Button20,
        }
    }

    /// Convert event gamepad axis to internal gamepad axis
    fn convert_gamepad_axis(&self, axis: crate::events::GamepadAxis) -> crate::input::gamepad::GamepadAxis {
        use crate::events::GamepadAxis as EventAxis;
        use crate::input::gamepad::GamepadAxis as InternalAxis;
        
        match axis {
            EventAxis::LeftStickX => InternalAxis::LeftStickX,
            EventAxis::LeftStickY => InternalAxis::LeftStickY,
            EventAxis::RightStickX => InternalAxis::RightStickX,
            EventAxis::RightStickY => InternalAxis::RightStickY,
            EventAxis::LeftTriggerAnalog => InternalAxis::LeftTriggerAnalog,
            EventAxis::RightTriggerAnalog => InternalAxis::RightTriggerAnalog,
            EventAxis::Axis6 => InternalAxis::Axis6,
            EventAxis::Axis7 => InternalAxis::Axis7,
            EventAxis::Axis8 => InternalAxis::Axis8,
            EventAxis::Axis9 => InternalAxis::Axis9,
            EventAxis::Axis10 => InternalAxis::Axis10,
            EventAxis::Axis11 => InternalAxis::Axis11,
        }
    }
}

impl Default for InputManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the input event queue
#[derive(Debug, Clone)]
pub struct InputQueueStats {
    pub is_empty: bool,
    pub is_full: bool,
}