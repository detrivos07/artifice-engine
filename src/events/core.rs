use artifice_logging::{debug, trace};
use std::any::Any;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Represents different categories of events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Window,
    Keyboard,
    Mouse,
    Gamepad,
    Application,
    Custom,
}

/// More efficient event representation using enums instead of trait objects
#[derive(Debug, Clone)]
pub enum EventData {
    Key(KeyEvent),
    MouseMove(MouseMoveEvent),
    MouseButton(MouseButtonEvent),
    MouseScroll(MouseScrollEvent),
    GamepadButton(GamepadButtonEvent),
    GamepadAxis(GamepadAxisEvent),
    GamepadConnection(GamepadConnectionEvent),
    WindowResize(WindowResizeEvent),
    WindowMove(WindowMoveEvent),
    WindowClose(WindowCloseEvent),
    ApplicationTick(ApplicationTickEvent),
    Custom(CustomEventData),
}

impl EventData {
    pub fn event_type(&self) -> EventType {
        match self {
            EventData::Key(_) => EventType::Keyboard,
            EventData::MouseMove(_) | EventData::MouseButton(_) | EventData::MouseScroll(_) => {
                EventType::Mouse
            }
            EventData::GamepadButton(_)
            | EventData::GamepadAxis(_)
            | EventData::GamepadConnection(_) => EventType::Gamepad,
            EventData::WindowResize(_) | EventData::WindowMove(_) | EventData::WindowClose(_) => {
                EventType::Window
            }
            EventData::ApplicationTick(_) => EventType::Application,
            EventData::Custom(_) => EventType::Custom,
        }
    }
}

/// The main Event struct that contains the actual event data
#[derive(Debug, Clone)]
pub struct Event {
    pub event_type: EventType,
    pub handled: bool,
    pub data: EventData,
    pub timestamp: u64,
}

impl Event {
    pub fn new(data: EventData) -> Self {
        Event {
            event_type: data.event_type(),
            handled: false,
            data,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    pub fn is_handled(&self) -> bool {
        self.handled
    }

    pub fn mark_handled(&mut self) {
        self.handled = true;
    }

    // Type-safe event data access methods
    pub fn as_key_event(&self) -> Option<&KeyEvent> {
        match &self.data {
            EventData::Key(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_mouse_move_event(&self) -> Option<&MouseMoveEvent> {
        match &self.data {
            EventData::MouseMove(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_mouse_button_event(&self) -> Option<&MouseButtonEvent> {
        match &self.data {
            EventData::MouseButton(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_mouse_scroll_event(&self) -> Option<&MouseScrollEvent> {
        match &self.data {
            EventData::MouseScroll(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_window_resize_event(&self) -> Option<&WindowResizeEvent> {
        match &self.data {
            EventData::WindowResize(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_window_move_event(&self) -> Option<&WindowMoveEvent> {
        match &self.data {
            EventData::WindowMove(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_window_close_event(&self) -> Option<&WindowCloseEvent> {
        match &self.data {
            EventData::WindowClose(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_application_tick_event(&self) -> Option<&ApplicationTickEvent> {
        match &self.data {
            EventData::ApplicationTick(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_custom_event(&self) -> Option<&CustomEventData> {
        match &self.data {
            EventData::Custom(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_gamepad_button_event(&self) -> Option<&GamepadButtonEvent> {
        match &self.data {
            EventData::GamepadButton(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_gamepad_axis_event(&self) -> Option<&GamepadAxisEvent> {
        match &self.data {
            EventData::GamepadAxis(event) => Some(event),
            _ => None,
        }
    }

    pub fn as_gamepad_connection_event(&self) -> Option<&GamepadConnectionEvent> {
        match &self.data {
            EventData::GamepadConnection(event) => Some(event),
            _ => None,
        }
    }

    // Legacy method for backward compatibility
}

/// Lock-free ring buffer for high-performance event queuing
pub struct EventQueue {
    events: Vec<std::sync::atomic::AtomicPtr<Event>>,
    capacity: usize,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
}

impl EventQueue {
    pub fn new(capacity: usize) -> Self {
        let mut events = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            events.push(std::sync::atomic::AtomicPtr::new(std::ptr::null_mut()));
        }

        EventQueue {
            events,
            capacity,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
        }
    }

    pub fn try_push(&self, event: Event) -> Result<(), Event> {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let next_write = (write_pos + 1) % self.capacity;

        if next_write == self.read_pos.load(Ordering::Acquire) {
            return Err(event); // Queue full
        }

        let event_ptr = Box::into_raw(Box::new(event));

        match self.events[write_pos].compare_exchange_weak(
            std::ptr::null_mut(),
            event_ptr,
            Ordering::Release,
            Ordering::Relaxed,
        ) {
            Ok(_) => {
                self.write_pos.store(next_write, Ordering::Release);
                Ok(())
            }
            Err(_) => {
                // Cleanup and retry
                let event = unsafe { Box::from_raw(event_ptr) };
                Err(*event)
            }
        }
    }

    pub fn try_pop(&self) -> Option<Event> {
        let read_pos = self.read_pos.load(Ordering::Acquire);

        if read_pos == self.write_pos.load(Ordering::Acquire) {
            return None; // Queue empty
        }

        let event_ptr = self.events[read_pos].swap(std::ptr::null_mut(), Ordering::Acquire);

        if event_ptr.is_null() {
            return None;
        }

        let next_read = (read_pos + 1) % self.capacity;
        self.read_pos.store(next_read, Ordering::Release);

        Some(*unsafe { Box::from_raw(event_ptr) })
    }

    pub fn is_empty(&self) -> bool {
        self.read_pos.load(Ordering::Acquire) == self.write_pos.load(Ordering::Acquire)
    }

    pub fn is_full(&self) -> bool {
        let write_pos = self.write_pos.load(Ordering::Acquire);
        let next_write = (write_pos + 1) % self.capacity;
        next_write == self.read_pos.load(Ordering::Acquire)
    }
}

unsafe impl Send for EventQueue {}
unsafe impl Sync for EventQueue {}

impl Drop for EventQueue {
    fn drop(&mut self) {
        // Clean up any remaining events
        while let Some(_) = self.try_pop() {
            // Events are automatically dropped
        }
    }
}

/// Event handler trait for handling events
pub trait EventHandler: Send + std::fmt::Debug {
    fn handle_event(&mut self, event: &mut Event);
}

/// Event Dispatcher - now using the more efficient event system
#[derive(Debug)]
pub struct EventDispatcher {
    handlers: HashMap<EventType, Vec<Box<dyn EventHandler>>>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        debug!("Creating event dispatcher");
        EventDispatcher {
            handlers: HashMap::new(),
        }
    }

    pub fn register_handler(&mut self, event_type: EventType, handler: Box<dyn EventHandler>) {
        debug!("Registering handler for event type: {:?}", event_type);
        self.handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }

    pub fn dispatch_event(&mut self, event: &mut Event) {
        trace!("Dispatching event: {:?}", event.event_type);
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
        F: FnMut(&mut Event) + Send + 'static,
    {
        let handler = ClosureEventHandler::new(listener);
        self.register_handler(event_type, Box::new(handler));
    }
}

// Implementation of an event handler that wraps a closure
struct ClosureEventHandler<F>
where
    F: FnMut(&mut Event) + Send + 'static,
{
    callback: F,
}

impl<F> ClosureEventHandler<F>
where
    F: FnMut(&mut Event) + Send + 'static,
{
    fn new(callback: F) -> Self {
        Self { callback }
    }
}

impl<F> std::fmt::Debug for ClosureEventHandler<F>
where
    F: FnMut(&mut Event) + Send + 'static,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClosureEventHandler")
            .field("callback", &"<closure>")
            .finish()
    }
}

impl<F> EventHandler for ClosureEventHandler<F>
where
    F: FnMut(&mut Event) + Send + 'static,
{
    fn handle_event(&mut self, event: &mut Event) {
        (self.callback)(event);
    }
}

/// Event filter trait for configurable event filtering
pub trait EventFilter: Send + Sync {
    /// Check if an event should be allowed through the filter
    fn should_allow(&self, event: &Event) -> bool;

    /// Get the name of this filter
    fn name(&self) -> &str;

    /// Get the priority of this filter (higher = processed first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Filter that allows only specific event types
pub struct EventTypeFilter {
    name: String,
    allowed_types: Vec<EventType>,
    priority: i32,
}

impl EventTypeFilter {
    pub fn new(name: impl Into<String>, allowed_types: Vec<EventType>) -> Self {
        Self {
            name: name.into(),
            allowed_types,
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl EventFilter for EventTypeFilter {
    fn should_allow(&self, event: &Event) -> bool {
        self.allowed_types.contains(&event.event_type)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

/// Filter that blocks specific event types
pub struct EventTypeBlockFilter {
    name: String,
    blocked_types: Vec<EventType>,
    priority: i32,
}

impl EventTypeBlockFilter {
    pub fn new(name: impl Into<String>, blocked_types: Vec<EventType>) -> Self {
        Self {
            name: name.into(),
            blocked_types,
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl EventFilter for EventTypeBlockFilter {
    fn should_allow(&self, event: &Event) -> bool {
        !self.blocked_types.contains(&event.event_type)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

/// Filter based on custom predicates
pub struct PredicateFilter<F>
where
    F: Fn(&Event) -> bool + Send + Sync,
{
    name: String,
    predicate: F,
    priority: i32,
}

impl<F> PredicateFilter<F>
where
    F: Fn(&Event) -> bool + Send + Sync,
{
    pub fn new(name: impl Into<String>, predicate: F) -> Self {
        Self {
            name: name.into(),
            predicate,
            priority: 0,
        }
    }

    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }
}

impl<F> EventFilter for PredicateFilter<F>
where
    F: Fn(&Event) -> bool + Send + Sync,
{
    fn should_allow(&self, event: &Event) -> bool {
        (self.predicate)(event)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> i32 {
        self.priority
    }
}

/// Manages event filters and applies them to events
pub struct EventFilterManager {
    filters: Vec<Box<dyn EventFilter>>,
    enabled: bool,
}

impl EventFilterManager {
    pub fn new() -> Self {
        Self {
            filters: Vec::new(),
            enabled: true,
        }
    }

    /// Add a filter to the manager
    pub fn add_filter(&mut self, filter: Box<dyn EventFilter>) {
        debug!("Adding event filter: {}", filter.name());
        self.filters.push(filter);
        // Sort by priority (highest first)
        self.filters.sort_by(|a, b| b.priority().cmp(&a.priority()));
    }

    /// Remove a filter by name
    pub fn remove_filter(&mut self, name: &str) -> bool {
        if let Some(pos) = self.filters.iter().position(|f| f.name() == name) {
            self.filters.remove(pos);
            debug!("Removed event filter: {}", name);
            true
        } else {
            false
        }
    }

    /// Clear all filters
    pub fn clear_filters(&mut self) {
        debug!("Clearing all event filters");
        self.filters.clear();
    }

    /// Enable or disable filtering
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        debug!(
            "Event filtering {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Check if filtering is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Apply all filters to an event
    pub fn should_allow_event(&self, event: &Event) -> bool {
        if !self.enabled {
            return true;
        }

        for filter in &self.filters {
            if !filter.should_allow(event) {
                trace!("Event blocked by filter: {}", filter.name());
                return false;
            }
        }

        true
    }

    /// Filter a list of events, returning only allowed events
    pub fn filter_events(&self, events: Vec<Event>) -> Vec<Event> {
        if !self.enabled {
            return events;
        }

        events
            .into_iter()
            .filter(|event| self.should_allow_event(event))
            .collect()
    }

    /// Get the names of all registered filters
    pub fn get_filter_names(&self) -> Vec<&str> {
        self.filters.iter().map(|f| f.name()).collect()
    }

    /// Get the number of registered filters
    pub fn filter_count(&self) -> usize {
        self.filters.len()
    }
}

impl Default for EventFilterManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Window Events
#[derive(Debug, Clone)]
pub struct WindowResizeEvent {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct WindowCloseEvent;

#[derive(Debug, Clone)]
pub struct WindowMoveEvent {
    pub x: i32,
    pub y: i32,
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

/// Mouse Events
#[derive(Debug, Clone)]
pub struct MouseMoveEvent {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone)]
pub struct MouseButtonEvent {
    pub button: MouseButton,
    pub action: KeyAction, // Reusing KeyAction for mouse buttons
    pub mods: KeyMod,
}

#[derive(Debug, Clone)]
pub struct MouseScrollEvent {
    pub x_offset: f64,
    pub y_offset: f64,
}

/// Application Events
#[derive(Debug, Clone)]
pub struct ApplicationTickEvent {
    pub delta_time: f32,
}

/// Custom event data that can hold any user-defined event type
#[derive(Debug)]
pub struct CustomEventData {
    pub type_name: String,
    pub data: std::sync::Arc<dyn Any + Send + Sync>,
}

impl CustomEventData {
    pub fn new<T: Any + Send + Sync>(type_name: impl Into<String>, data: T) -> Self {
        Self {
            type_name: type_name.into(),
            data: std::sync::Arc::new(data),
        }
    }

    pub fn get_data<T: Any>(&self) -> Option<&T> {
        self.data.downcast_ref::<T>()
    }

    pub fn is_type<T: Any>(&self) -> bool {
        self.data.is::<T>()
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }
}

impl Clone for CustomEventData {
    fn clone(&self) -> Self {
        Self {
            type_name: self.type_name.clone(),
            data: self.data.clone(),
        }
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

/// Standard gamepad buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GamepadButton {
    // Face buttons (Xbox layout names)
    A, // Bottom face button
    B, // Right face button
    X, // Left face button
    Y, // Top face button

    // Shoulder buttons
    LeftBumper,   // L1/LB
    RightBumper,  // R1/RB
    LeftTrigger,  // L2/LT (digital)
    RightTrigger, // R2/RT (digital)

    // D-pad
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,

    // Special buttons
    Start,  // Start/Menu/Options
    Select, // Back/View/Share
    Guide,  // Xbox/PS/Home button

    // Stick buttons
    LeftStick,  // L3
    RightStick, // R3

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
    LeftTriggerAnalog,  // L2/LT analog value
    RightTriggerAnalog, // R2/RT analog value

    // Additional axes for extended controllers
    Axis6,
    Axis7,
    Axis8,
    Axis9,
    Axis10,
    Axis11,
}

/// Represents a gamepad button input event
#[derive(Debug, Clone)]
pub struct GamepadButtonEvent {
    pub gamepad_id: u32,
    pub button: GamepadButton,
    pub action: KeyAction,
    pub mods: KeyMod,
}

/// Represents a gamepad axis input event
#[derive(Debug, Clone)]
pub struct GamepadAxisEvent {
    pub gamepad_id: u32,
    pub axis: GamepadAxis,
    pub value: f32,
}

/// Represents a gamepad connection/disconnection event
#[derive(Debug, Clone)]
pub struct GamepadConnectionEvent {
    pub gamepad_id: u32,
    pub connected: bool,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_creation_and_type_safety() {
        // Test keyboard event
        let key_event = KeyEvent {
            key: KeyCode::A,
            action: KeyAction::Press,
            mods: KeyMod::new(),
        };
        let event = Event::new(EventData::Key(key_event.clone()));

        assert_eq!(event.event_type, EventType::Keyboard);
        assert!(!event.is_handled());
        assert!(event.as_key_event().is_some());
        assert_eq!(event.as_key_event().unwrap().key, KeyCode::A);
        assert!(event.as_mouse_move_event().is_none());

        // Test mouse move event
        let mouse_move = MouseMoveEvent { x: 100.0, y: 200.0 };
        let event = Event::new(EventData::MouseMove(mouse_move.clone()));

        assert_eq!(event.event_type, EventType::Mouse);
        assert!(event.as_mouse_move_event().is_some());
        assert_eq!(event.as_mouse_move_event().unwrap().x, 100.0);
        assert!(event.as_key_event().is_none());

        // Test window resize event
        let resize_event = WindowResizeEvent {
            width: 800,
            height: 600,
        };
        let event = Event::new(EventData::WindowResize(resize_event.clone()));

        assert_eq!(event.event_type, EventType::Window);
        assert!(event.as_window_resize_event().is_some());
        assert_eq!(event.as_window_resize_event().unwrap().width, 800);
    }

    #[test]
    fn test_event_handling() {
        let key_event = KeyEvent {
            key: KeyCode::Escape,
            action: KeyAction::Press,
            mods: KeyMod::new(),
        };
        let mut event = Event::new(EventData::Key(key_event));

        assert!(!event.is_handled());
        event.mark_handled();
        assert!(event.is_handled());
    }

    #[test]
    fn test_event_queue_basic_operations() {
        let queue = EventQueue::new(4);

        assert!(queue.is_empty());
        assert!(!queue.is_full());

        // Test pushing events
        let event1 = Event::new(EventData::Key(KeyEvent {
            key: KeyCode::A,
            action: KeyAction::Press,
            mods: KeyMod::new(),
        }));

        assert!(queue.try_push(event1).is_ok());
        assert!(!queue.is_empty());

        // Test popping events
        let popped_event = queue.try_pop();
        assert!(popped_event.is_some());
        assert!(queue.is_empty());

        let popped = popped_event.unwrap();
        assert_eq!(popped.event_type, EventType::Keyboard);
        assert!(popped.as_key_event().is_some());
        assert_eq!(popped.as_key_event().unwrap().key, KeyCode::A);
    }

    #[test]
    fn test_event_queue_full_behavior() {
        let queue = EventQueue::new(3); // Use size 3 to allow 2 events

        let event1 = Event::new(EventData::Key(KeyEvent {
            key: KeyCode::A,
            action: KeyAction::Press,
            mods: KeyMod::new(),
        }));

        let event2 = Event::new(EventData::Key(KeyEvent {
            key: KeyCode::B,
            action: KeyAction::Press,
            mods: KeyMod::new(),
        }));

        // Fill the queue
        assert!(queue.try_push(event1).is_ok());
        assert!(!queue.is_full()); // Not full yet

        assert!(queue.try_push(event2).is_ok());
        assert!(queue.is_full()); // Now it should be full

        // This should be rejected when full
        let event3 = Event::new(EventData::Key(KeyEvent {
            key: KeyCode::C,
            action: KeyAction::Press,
            mods: KeyMod::new(),
        }));

        let result = queue.try_push(event3);
        assert!(result.is_err());
        let rejected_event = result.unwrap_err();
        assert_eq!(rejected_event.as_key_event().unwrap().key, KeyCode::C);
    }

    #[test]
    fn test_event_data_type_matching() {
        // Test that EventData correctly reports its type
        let key_data = EventData::Key(KeyEvent {
            key: KeyCode::Enter,
            action: KeyAction::Press,
            mods: KeyMod::new(),
        });
        assert_eq!(key_data.event_type(), EventType::Keyboard);

        let mouse_data = EventData::MouseMove(MouseMoveEvent { x: 0.0, y: 0.0 });
        assert_eq!(mouse_data.event_type(), EventType::Mouse);

        let window_data = EventData::WindowClose(WindowCloseEvent);
        assert_eq!(window_data.event_type(), EventType::Window);

        let app_data = EventData::ApplicationTick(ApplicationTickEvent { delta_time: 0.016 });
        assert_eq!(app_data.event_type(), EventType::Application);
    }

    #[test]
    fn test_key_mod_creation() {
        let mut mods = KeyMod::new();
        assert!(!mods.shift);
        assert!(!mods.control);
        assert!(!mods.alt);

        mods.shift = true;
        mods.control = true;
        assert!(mods.shift);
        assert!(mods.control);
        assert!(!mods.alt);
    }

    #[test]
    fn test_event_timestamp() {
        let event = Event::new(EventData::WindowClose(WindowCloseEvent));
        assert!(event.timestamp > 0);

        // Create another event and ensure timestamp is different (or at least not zero)
        std::thread::sleep(std::time::Duration::from_millis(1));
        let event2 = Event::new(EventData::WindowClose(WindowCloseEvent));
        assert!(event2.timestamp >= event.timestamp);
    }
}
