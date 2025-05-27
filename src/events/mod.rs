pub mod core;

// Re-export key types for easier access
pub use core::{
    Event, EventData, EventType, EventDispatcher, EventFilterManager,
    KeyAction, KeyCode, KeyEvent, KeyMod, MouseButton, 
    MouseMoveEvent, MouseButtonEvent, MouseScrollEvent,
    GamepadButton, GamepadAxis, GamepadButtonEvent, GamepadAxisEvent, GamepadConnectionEvent,
    WindowResizeEvent, WindowMoveEvent, WindowCloseEvent,
    EventFilter, EventTypeFilter, PredicateFilter, CustomEventData
};