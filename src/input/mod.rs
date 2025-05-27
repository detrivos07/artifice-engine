pub mod gamepad;
pub mod keyboard;
pub mod mouse;
pub mod manager;
pub mod recording;

// Re-export key types for easier access
pub use gamepad::{
    GamepadManager, GamepadState, GamepadButton, GamepadAxis,
    GamepadButtonEvent, GamepadAxisEvent, GamepadConnectionEvent
};
pub use keyboard::Keyboard;
pub use mouse::Mouse;
pub use manager::{InputManager, InputQueueStats};
pub use recording::{
    InputRecorder, InputPlayer, InputRecording, InputRecordingManager,
    RecordedEvent, RecordingMetadata, SerializableEventData
};

/// Input device trait for common functionality
pub trait InputDevice {
    fn update(&mut self);
    fn is_connected(&self) -> bool;
}