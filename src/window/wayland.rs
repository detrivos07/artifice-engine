use crate::events::core::{Event, EventData, KeyEvent, MouseMoveEvent, MouseButtonEvent, MouseScrollEvent, WindowResizeEvent, KeyAction, KeyCode, KeyMod, MouseButton};
use crate::io::{Window, WindowHint, OpenGLWindow, Size, Position};
use crate::window::factory::{WindowFactory, WindowFeature};
use artifice_logging::{debug, info, warn};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::any::Any;

use std::os::unix::io::BorrowedFd;

// Wayland protocol imports
use wayland_client::{
    Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum,
    protocol::{
        wl_compositor::WlCompositor,
        wl_surface::{self, WlSurface},
        wl_shell::WlShell,
        wl_shell_surface::{self, WlShellSurface},
        wl_seat::WlSeat,
        wl_pointer::{self, WlPointer},
        wl_keyboard::{self, WlKeyboard},
        wl_registry::WlRegistry,
        wl_shm::{self, WlShm},
        wl_buffer::{self, WlBuffer},
        wl_shm_pool::WlShmPool,
        wl_output::WlOutput,
    },
    globals::{registry_queue_init, GlobalListContents},
};

/// Wayland window implementation
pub struct WaylandWindow {
    // Core Wayland objects
    connection: Connection,
    event_queue: EventQueue<WaylandState>,
    compositor: WlCompositor,
    surface: WlSurface,
    shell: Option<WlShell>,
    shell_surface: Option<WlShellSurface>,
    seat: Option<WlSeat>,
    pointer: Option<WlPointer>,
    keyboard: Option<WlKeyboard>,
    shm: Option<WlShm>,
    
    // Window properties
    size: Size,
    position: Position,
    title: String,
    should_close: bool,
    
    // Event handling
    event_callback: Option<Arc<Mutex<dyn FnMut(Event) + Send + 'static>>>,
    
    // State tracking
    mouse_x: f64,
    mouse_y: f64,
    keyboard_state: HashMap<u32, bool>,
    modifiers: KeyMod,
    
    // Buffer management
    buffer: Option<WlBuffer>,
    buffer_data: Vec<u8>,
}

/// State object for Wayland event handling
#[derive(Debug)]
pub struct WaylandState {
    window_ref: *mut WaylandWindow,
}

unsafe impl Send for WaylandState {}
unsafe impl Sync for WaylandState {}

impl WaylandState {
    fn new(window: &mut WaylandWindow) -> Self {
        Self {
            window_ref: window as *mut WaylandWindow,
        }
    }

    unsafe fn get_window(&self) -> &mut WaylandWindow {
        &mut *self.window_ref
    }
}

impl WaylandWindow {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        Self::with_hints(width, height, title, &[])
    }

    pub fn with_hints(width: u32, height: u32, title: &str, _hints: &[WindowHint]) -> Self {
        info!("Creating Wayland window: {} ({}x{})", title, width, height);

        // Connect to Wayland compositor
        let connection = Connection::connect_to_env()
            .expect("Failed to connect to Wayland compositor");

        // Create initial event queue
        let (globals, event_queue) = registry_queue_init::<WaylandState>(&connection)
            .expect("Failed to initialize Wayland registry");

        // Get required globals
        let compositor: WlCompositor = globals
            .bind(&event_queue.handle(), 1..=4, ())
            .expect("Failed to bind compositor");

        let shell: Option<WlShell> = globals
            .bind(&event_queue.handle(), 1..=1, ())
            .ok();

        let seat: Option<WlSeat> = globals
            .bind(&event_queue.handle(), 1..=7, ())
            .ok();

        let shm: Option<WlShm> = globals
            .bind(&event_queue.handle(), 1..=1, ())
            .ok();

        // Create surface
        let surface = compositor.create_surface(&event_queue.handle(), ());

        let mut window = Self {
            connection,
            event_queue,
            compositor,
            surface: surface.clone(),
            shell: shell.clone(),
            shell_surface: None,
            seat: seat.clone(),
            pointer: None,
            keyboard: None,
            shm,
            size: Size(width, height),
            position: Position(0, 0),
            title: title.to_string(),
            should_close: false,
            event_callback: None,
            mouse_x: 0.0,
            mouse_y: 0.0,
            keyboard_state: HashMap::new(),
            modifiers: KeyMod::default(),
            buffer: None,
            buffer_data: Vec::new(),
        };

        // Set up shell surface if shell is available
        if let Some(ref shell) = shell {
            let shell_surface = shell.get_shell_surface(&surface, &window.event_queue.handle(), ());
            shell_surface.set_title(title.to_string());
            shell_surface.set_toplevel();
            window.shell_surface = Some(shell_surface);
        }

        // Set up input devices
        if let Some(ref seat) = seat {
            // Get pointer
            if seat.version() >= 3 {
                let pointer = seat.get_pointer(&window.event_queue.handle(), ());
                window.pointer = Some(pointer);
            }

            // Get keyboard
            if seat.version() >= 3 {
                let keyboard = seat.get_keyboard(&window.event_queue.handle(), ());
                window.keyboard = Some(keyboard);
            }
        }

        // Initialize buffer
        window.create_buffer(width, height);

        window
    }



    fn create_buffer(&mut self, width: u32, height: u32) {
        if let Some(ref shm) = self.shm {
            let stride = width * 4; // 4 bytes per pixel (ARGB)
            let size = stride * height;
            
            // Create shared memory file
            let fd = create_anonymous_file(size as usize)
                .expect("Failed to create shared memory file");
            
            // Create shm pool
            let borrowed_fd = unsafe { BorrowedFd::borrow_raw(fd) };
            let pool = shm.create_pool(borrowed_fd, size as i32, &self.event_queue.handle(), ());
            
            // Create buffer
            let buffer = pool.create_buffer(
                0,
                width as i32,
                height as i32,
                stride as i32,
                wl_shm::Format::Argb8888,
                &self.event_queue.handle(),
                ()
            );
            
            // Initialize buffer data
            self.buffer_data = vec![0xFF; size as usize]; // White background
            self.buffer = Some(buffer);
            
            // Clean up pool
            pool.destroy();
        }
    }

    fn send_event(&mut self, event: Event) {
        if let Some(ref callback) = self.event_callback {
            if let Ok(mut cb) = callback.lock() {
                cb(event);
            }
        }
    }

    fn map_wayland_key_to_keycode(key: u32) -> KeyCode {
        // Basic key mapping - would need to be expanded for full support
        match key {
            // Letters
            30 => KeyCode::A, 48 => KeyCode::B, 46 => KeyCode::C, 32 => KeyCode::D,
            18 => KeyCode::E, 33 => KeyCode::F, 34 => KeyCode::G, 35 => KeyCode::H,
            23 => KeyCode::I, 36 => KeyCode::J, 37 => KeyCode::K, 38 => KeyCode::L,
            50 => KeyCode::M, 49 => KeyCode::N, 24 => KeyCode::O, 25 => KeyCode::P,
            16 => KeyCode::Q, 19 => KeyCode::R, 31 => KeyCode::S, 20 => KeyCode::T,
            22 => KeyCode::U, 47 => KeyCode::V, 17 => KeyCode::W, 45 => KeyCode::X,
            21 => KeyCode::Y, 44 => KeyCode::Z,
            
            // Numbers
            11 => KeyCode::Num0, 2 => KeyCode::Num1, 3 => KeyCode::Num2, 4 => KeyCode::Num3,
            5 => KeyCode::Num4, 6 => KeyCode::Num5, 7 => KeyCode::Num6, 8 => KeyCode::Num7,
            9 => KeyCode::Num8, 10 => KeyCode::Num9,
            
            // Special keys
            57 => KeyCode::Space,
            28 => KeyCode::Enter,
            1 => KeyCode::Escape,
            14 => KeyCode::Backspace,
            15 => KeyCode::Tab,
            
            // Arrow keys
            103 => KeyCode::Up, 108 => KeyCode::Down, 105 => KeyCode::Left, 106 => KeyCode::Right,
            
            // Function keys
            59 => KeyCode::F1, 60 => KeyCode::F2, 61 => KeyCode::F3, 62 => KeyCode::F4,
            63 => KeyCode::F5, 64 => KeyCode::F6, 65 => KeyCode::F7, 66 => KeyCode::F8,
            67 => KeyCode::F9, 68 => KeyCode::F10, 87 => KeyCode::F11, 88 => KeyCode::F12,
            
            // Modifiers
            42 | 54 => KeyCode::LeftShift, // Left/Right Shift
            29 | 97 => KeyCode::LeftControl, // Left/Right Control
            56 | 100 => KeyCode::LeftAlt, // Left/Right Alt
            
            _ => KeyCode::Unknown,
        }
    }

    fn map_wayland_mouse_button(button: u32) -> MouseButton {
        match button {
            0x110 => MouseButton::Left,   // BTN_LEFT
            0x111 => MouseButton::Right,  // BTN_RIGHT
            0x112 => MouseButton::Middle, // BTN_MIDDLE
            0x113 => MouseButton::Button4,
            0x114 => MouseButton::Button5,
            0x115 => MouseButton::Button6,
            0x116 => MouseButton::Button7,
            0x117 => MouseButton::Button8,
            _ => MouseButton::Button1,
        }
    }
}

impl Window for WaylandWindow {
    fn update(&mut self) {
        // Commit surface changes
        self.surface.commit();
        
        // Process pending events
        self.process_events();
    }

    fn process_events(&mut self) {
        // Create state object for event dispatching
        let mut state = WaylandState::new(self);
        
        // Dispatch only pending events (non-blocking)
        if let Err(e) = self.event_queue.dispatch_pending(&mut state) {
            warn!("Failed to dispatch Wayland events: {}", e);
        }
    }

    fn set_should_close(&mut self) {
        self.should_close = true;
    }

    fn should_close(&self) -> bool {
        self.should_close
    }

    fn set_position(&mut self, position: Position) {
        self.position = position;
        // Note: Wayland doesn't allow clients to set window position directly
        // This would typically be handled by the compositor/window manager
    }

    fn position(&self) -> &Position {
        &self.position
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        self.create_buffer(size.0, size.1);
        
        // Send resize event
        let event = Event::new(EventData::WindowResize(WindowResizeEvent {
            width: size.0,
            height: size.1,
        }));
        
        if let Some(ref callback) = self.event_callback {
            if let Ok(mut cb) = callback.lock() {
                cb(event);
            }
        }
    }

    fn size(&self) -> &Size {
        &self.size
    }

    fn title(&self) -> &str {
        &self.title
    }

    fn set_title(&mut self, title: &str) {
        self.title = title.to_string();
        if let Some(ref shell_surface) = self.shell_surface {
            shell_surface.set_title(title.to_string());
        }
    }

    fn get_event_callback(&self) -> Option<Arc<Mutex<dyn FnMut(Event) + Send + 'static>>> {
        self.event_callback.clone()
    }

    fn set_event_callback(&mut self, callback: Arc<Mutex<dyn FnMut(Event) + Send + 'static>>) {
        self.event_callback = Some(callback);
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl OpenGLWindow for WaylandWindow {
    fn make_current(&mut self) {
        // Wayland OpenGL context management would require EGL integration
        warn!("OpenGL context management not implemented for Wayland backend");
    }

    fn is_current(&self) -> bool {
        false // Not implemented
    }

    fn swap_buffers(&mut self) {
        // For a basic implementation, just attach the buffer to the surface
        if let Some(ref buffer) = self.buffer {
            self.surface.attach(Some(buffer), 0, 0);
            self.surface.damage(0, 0, self.size.0 as i32, self.size.1 as i32);
            self.surface.commit();
        }
    }

    fn reload_opengl_functions(&mut self) {
        // Wayland OpenGL context management would require EGL integration
        warn!("OpenGL function reloading not implemented for Wayland backend - requires EGL integration");
    }
}

// Wayland event dispatch implementations
impl Dispatch<WlRegistry, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        _event: <WlRegistry as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        // Registry events are handled during initialization
    }
}

impl Dispatch<WlRegistry, GlobalListContents> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlRegistry,
        _event: <WlRegistry as Proxy>::Event,
        _data: &GlobalListContents,
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        // Registry events for global list
    }
}

impl Dispatch<WlShell, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShell,
        _event: <WlShell as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        // Shell events
    }
}

impl Dispatch<WlCompositor, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlCompositor,
        _event: <WlCompositor as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        // Compositor events
    }
}

impl Dispatch<WlSurface, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSurface,
        event: <WlSurface as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        match event {
            wl_surface::Event::Enter { output: _ } => {
                debug!("Surface entered output");
            }
            wl_surface::Event::Leave { output: _ } => {
                debug!("Surface left output");
            }
            _ => {}
        }
    }
}

impl Dispatch<WlShellSurface, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &WlShellSurface,
        event: <WlShellSurface as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        unsafe {
            let window = state.get_window();
            match event {
                wl_shell_surface::Event::Configure { edges: _, width, height } => {
                    if width > 0 && height > 0 {
                        window.set_size(Size(width as u32, height as u32));
                    }
                }
                wl_shell_surface::Event::PopupDone => {
                    // Handle popup done
                }
                wl_shell_surface::Event::Ping { serial } => {
                    // Respond to ping
                    if let Some(ref shell_surface) = window.shell_surface {
                        shell_surface.pong(serial);
                    }
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSeat,
        _event: <WlSeat as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        // Seat capability changes
    }
}

impl Dispatch<WlPointer, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &WlPointer,
        event: <WlPointer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        unsafe {
            let window = state.get_window();
            match event {
                wl_pointer::Event::Enter { serial: _, surface: _, surface_x, surface_y } => {
                    window.mouse_x = surface_x;
                    window.mouse_y = surface_y;
                }
                wl_pointer::Event::Leave { serial: _, surface: _ } => {
                    // Mouse left the surface
                }
                wl_pointer::Event::Motion { time: _, surface_x, surface_y } => {
                    window.mouse_x = surface_x;
                    window.mouse_y = surface_y;
                    
                    let event = Event::new(EventData::MouseMove(MouseMoveEvent {
                        x: surface_x,
                        y: surface_y,
                    }));
                    window.send_event(event);
                }
                wl_pointer::Event::Button { serial: _, time: _, button, state } => {
                    let mouse_button = WaylandWindow::map_wayland_mouse_button(button);
                    let action = match state {
                        WEnum::Value(wl_pointer::ButtonState::Pressed) => KeyAction::Press,
                        WEnum::Value(wl_pointer::ButtonState::Released) => KeyAction::Release,
                        _ => return,
                    };
                    
                    let event = Event::new(EventData::MouseButton(MouseButtonEvent {
                        button: mouse_button,
                        action,
                        mods: window.modifiers.clone(),
                    }));
                    window.send_event(event);
                }
                wl_pointer::Event::Axis { time: _, axis, value } => {
                    let (x_offset, y_offset) = match axis {
                        WEnum::Value(wl_pointer::Axis::VerticalScroll) => (0.0, value / 10.0), // Scale down
                        WEnum::Value(wl_pointer::Axis::HorizontalScroll) => (value / 10.0, 0.0), // Scale down
                        _ => return,
                    };
                    
                    let event = Event::new(EventData::MouseScroll(MouseScrollEvent {
                        x_offset,
                        y_offset,
                    }));
                    window.send_event(event);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<WlKeyboard, ()> for WaylandState {
    fn event(
        state: &mut Self,
        _proxy: &WlKeyboard,
        event: <WlKeyboard as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        unsafe {
            let window = state.get_window();
            match event {
                wl_keyboard::Event::Enter { serial: _, surface: _, keys: _ } => {
                    // Keyboard focus gained
                }
                wl_keyboard::Event::Leave { serial: _, surface: _ } => {
                    // Keyboard focus lost
                }
                wl_keyboard::Event::Key { serial: _, time: _, key, state } => {
                    let keycode = WaylandWindow::map_wayland_key_to_keycode(key);
                    let action = match state {
                        WEnum::Value(wl_keyboard::KeyState::Pressed) => {
                            window.keyboard_state.insert(key, true);
                            KeyAction::Press
                        }
                        WEnum::Value(wl_keyboard::KeyState::Released) => {
                            window.keyboard_state.remove(&key);
                            KeyAction::Release
                        }
                        _ => return,
                    };
                    
                    let event = Event::new(EventData::Key(KeyEvent {
                        key: keycode,
                        action,
                        mods: window.modifiers.clone(),
                    }));
                    window.send_event(event);
                }
                wl_keyboard::Event::Modifiers { serial: _, mods_depressed, mods_latched: _, mods_locked: _, group: _ } => {
                    // Update modifier state
                    window.modifiers.shift = (mods_depressed & 0x01) != 0;
                    window.modifiers.control = (mods_depressed & 0x04) != 0;
                    window.modifiers.alt = (mods_depressed & 0x08) != 0;
                    window.modifiers.super_key = (mods_depressed & 0x40) != 0;
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<WlShm, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShm,
        _event: <WlShm as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        // Shared memory events
    }
}

impl Dispatch<WlShmPool, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlShmPool,
        _event: <WlShmPool as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        // Shared memory pool events
    }
}

impl Dispatch<WlBuffer, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlBuffer,
        event: <WlBuffer as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        match event {
            wl_buffer::Event::Release => {
                // Buffer has been released by the compositor
            }
            _ => {}
        }
    }
}

impl Dispatch<WlOutput, ()> for WaylandState {
    fn event(
        _state: &mut Self,
        _proxy: &WlOutput,
        _event: <WlOutput as Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qhandle: &QueueHandle<WaylandState>,
    ) {
        // Output events (monitor info)
    }
}

/// Wayland window factory
pub struct WaylandWindowFactory;

impl WindowFactory for WaylandWindowFactory {
    fn create_window(&self, width: u32, height: u32, title: &str) -> Box<dyn Window> {
        Box::new(WaylandWindow::new(width, height, title))
    }
    
    fn create_window_with_hints(&self, width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Box<dyn Window> {
        Box::new(WaylandWindow::with_hints(width, height, title, hints))
    }
    
    fn supports_feature(&self, feature: WindowFeature) -> bool {
        match feature {
            WindowFeature::OpenGL => false, // Would require EGL integration
            WindowFeature::Vulkan => true,  // Wayland supports Vulkan
            WindowFeature::MultiWindow => true,
            WindowFeature::HighDPI => true,
            WindowFeature::Fullscreen => true,
            WindowFeature::Transparency => true,
            WindowFeature::CustomCursor => true,
            WindowFeature::RawInput => true,
            WindowFeature::MonitorInfo => true,
            WindowFeature::DirectX => false,
        }
    }
    
    fn backend_name(&self) -> &str {
        "Wayland"
    }
    
    fn backend_version(&self) -> Option<String> {
        Some("1.0".to_string())
    }
}

/// Helper function to create an anonymous file for shared memory
fn create_anonymous_file(size: usize) -> Result<std::os::unix::io::RawFd, std::io::Error> {
    use std::ffi::CString;

    
    let name = CString::new("artifice-wayland-shm").unwrap();
    
    unsafe {
        let fd = libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC);
        if fd == -1 {
            return Err(std::io::Error::last_os_error());
        }
        
        if libc::ftruncate(fd, size as i64) == -1 {
            libc::close(fd);
            return Err(std::io::Error::last_os_error());
        }
        
        Ok(fd)
    }
}