use logging;
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

/// The core Event trait that all events must implement
pub trait EventTrait: Send + Sync + Debug + 'static {
    fn event_type(&self) -> EventType;
    fn as_any(&self) -> &dyn Any;
}

/// Represents different categories of events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Window,
    Keyboard,
    Mouse,
    Application,
    Custom(u32),
}

/// The main Event struct that contains the actual event data
#[derive(Debug)]
pub struct Event {
    pub event_type: EventType,
    pub handled: bool,
    pub data: Box<dyn EventTrait>,
}

impl Event {
    pub fn new<T: EventTrait>(data: T) -> Self {
        Event {
            event_type: data.event_type(),
            handled: false,
            data: Box::new(data),
        }
    }

    pub fn is_handled(&self) -> bool {
        self.handled
    }

    pub fn mark_handled(&mut self) {
        self.handled = true;
    }

    pub fn get_data<T: EventTrait + 'static>(&self) -> Option<&T> {
        self.data.as_any().downcast_ref::<T>()
    }
}

/// Event handler trait for handling events
pub trait EventHandler: Send + Sync + std::fmt::Debug {
    fn handle_event(&mut self, event: &mut Event);
}

/// Window Events
#[derive(Debug, Clone)]
pub struct WindowResizeEvent {
    pub width: u32,
    pub height: u32,
}

impl EventTrait for WindowResizeEvent {
    fn event_type(&self) -> EventType {
        EventType::Window
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct WindowCloseEvent;

impl EventTrait for WindowCloseEvent {
    fn event_type(&self) -> EventType {
        EventType::Window
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct WindowMoveEvent {
    pub x: i32,
    pub y: i32,
}

impl EventTrait for WindowMoveEvent {
    fn event_type(&self) -> EventType {
        EventType::Window
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Keyboard Events
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Release,
    Repeat,
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: KeyCode,
    pub action: KeyAction,
    pub mods: KeyMod,
}

impl EventTrait for KeyEvent {
    fn event_type(&self) -> EventType {
        EventType::Keyboard
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Mouse Events
#[derive(Debug, Clone)]
pub struct MouseMoveEvent {
    pub x: f64,
    pub y: f64,
}

impl EventTrait for MouseMoveEvent {
    fn event_type(&self) -> EventType {
        EventType::Mouse
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub action: KeyAction, // Reusing KeyAction for mouse buttons
    pub mods: KeyMod,
}

impl EventTrait for MouseButtonEvent {
    fn event_type(&self) -> EventType {
        EventType::Mouse
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Clone)]
pub struct MouseScrollEvent {
    pub x_offset: f64,
    pub y_offset: f64,
}

impl EventTrait for MouseScrollEvent {
    fn event_type(&self) -> EventType {
        EventType::Mouse
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Application Events
#[derive(Debug, Clone)]
pub struct ApplicationTickEvent {
    pub delta_time: f32,
}

impl EventTrait for ApplicationTickEvent {
    fn event_type(&self) -> EventType {
        EventType::Application
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// Event Dispatcher
#[derive(Debug)]
pub struct EventDispatcher {
    handlers: HashMap<EventType, Vec<Box<dyn EventHandler>>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        logging::debug("Creating event dispatcher");
        EventDispatcher {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, event_type: EventType, handler: Box<dyn EventHandler>) {
        logging::debug(&format!(
            "Registering handler for event type: {:?}",
            event_type
        ));
        self.handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub fn dispatch_event(&mut self, event: &mut Event) {
        logging::trace(&format!("Dispatching event: {:?}", event.event_type));
        if let Some(handlers) = self.handlers.get_mut(&event.event_type) {
            for handler in handlers.iter_mut() {
                handler.handle_event(event);
                if event.is_handled() {
                    break;
                }
            }
        }
    }

    /// Register a closure as an event handler
    pub fn add_event_listener<F>(&mut self, event_type: EventType, listener: F)
    where
        F: Fn(&mut Event) + Send + Sync + Debug + 'static,
    {
        let handler = ClosureEventHandler::new(listener);
        self.register_handler(event_type, Box::new(handler));
    }
}

// Implementation of an event handler that wraps a closure
#[derive(Debug)]
struct ClosureEventHandler<F>
where
    F: Fn(&mut Event) + Send + Sync + Debug + 'static,
{
    callback: F,
}

impl<F> ClosureEventHandler<F>
where
    F: Fn(&mut Event) + Send + Sync + Debug + 'static,
{
    fn new(callback: F) -> Self {
        ClosureEventHandler { callback }
    }
}

impl<F> EventHandler for ClosureEventHandler<F>
where
    F: Fn(&mut Event) + Send + Sync + Debug + 'static,
{
    fn handle_event(&mut self, event: &mut Event) {
        (self.callback)(event);
    }
}

/// Key Codes and Modifiers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    Unknown,
    Space,
    Apostrophe,
    Comma,
    Minus,
    Period,
    Slash,
    Num0,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Semicolon,
    Equal,
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    LeftBracket,
    Backslash,
    RightBracket,
    GraveAccent,
    World1,
    World2,
    Escape,
    Enter,
    Tab,
    Backspace,
    Insert,
    Delete,
    Right,
    Left,
    Down,
    Up,
    PageUp,
    PageDown,
    Home,
    End,
    CapsLock,
    ScrollLock,
    NumLock,
    PrintScreen,
    Pause,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    F13,
    F14,
    F15,
    F16,
    F17,
    F18,
    F19,
    F20,
    F21,
    F22,
    F23,
    F24,
    F25,
    KP0,
    KP1,
    KP2,
    KP3,
    KP4,
    KP5,
    KP6,
    KP7,
    KP8,
    KP9,
    KPDecimal,
    KPDivide,
    KPMultiply,
    KPSubtract,
    KPAdd,
    KPEnter,
    KPEqual,
    LeftShift,
    LeftControl,
    LeftAlt,
    LeftSuper,
    RightShift,
    RightControl,
    RightAlt,
    RightSuper,
    Menu,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KeyMod {
    pub shift: bool,
    pub control: bool,
    pub alt: bool,
    pub super_key: bool,
    pub caps_lock: bool,
    pub num_lock: bool,
}

impl KeyMod {
    pub fn new() -> Self {
        KeyMod {
            shift: false,
            control: false,
            alt: false,
            super_key: false,
            caps_lock: false,
            num_lock: false,
        }
    }
}

impl Default for KeyMod {
    fn default() -> Self {
        Self::new()
    }
}

/// Mouse Buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Button1,
    Button2,
    Button3,
    Button4,
    Button5,
    Button6,
    Button7,
    Button8,
    Left,
    Right,
    Middle,
}
