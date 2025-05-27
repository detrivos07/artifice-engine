use crate::events::core::{Event, EventData, KeyEvent, MouseMoveEvent, MouseButtonEvent, MouseScrollEvent, WindowResizeEvent, WindowMoveEvent, WindowCloseEvent, KeyAction, KeyCode, KeyMod, MouseButton};
use crate::io::{Window, WindowHint, OpenGLWindow, Size, Position, OpenGLProfile};
use crate::window::factory::{WindowFactory, WindowFeature};
use artifice_logging::{debug, info, warn, error};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::ffi::{CString, CStr};
use std::ptr;
use std::mem;
use std::any::Any;

// X11 and GLX bindings
use x11::xlib::{self, Display, Window as XWindow, XEvent, XSetWindowAttributes, XWindowAttributes};
use x11::glx::{self, GLXContext, GLXFBConfig};

/// X11 window implementation
pub struct X11Window {
    // X11 core objects
    display: *mut Display,
    window: XWindow,
    screen: i32,
    visual_info: *mut xlib::XVisualInfo,
    
    // GLX objects
    glx_context: GLXContext,
    glx_fbconfig: GLXFBConfig,
    
    // Window properties
    size: Size,
    position: Position,
    title: String,
    should_close: bool,
    
    // Event handling
    event_callback: Option<Arc<Mutex<dyn FnMut(Event) + Send + 'static>>>,
    
    // State tracking
    key_map: HashMap<u32, KeyCode>,
    button_map: HashMap<u32, MouseButton>,
    modifiers: KeyMod,
    
    // Atoms for window management
    wm_delete_window: xlib::Atom,
    wm_protocols: xlib::Atom,
}

unsafe impl Send for X11Window {}
unsafe impl Sync for X11Window {}

impl X11Window {
    pub fn new(width: u32, height: u32, title: &str) -> Self {
        Self::with_hints(width, height, title, &[])
    }

    pub fn with_hints(width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Self {
        info!("Creating X11 window: {} ({}x{})", title, width, height);

        unsafe {
            // Open display connection
            let display = xlib::XOpenDisplay(ptr::null());
            if display.is_null() {
                panic!("Failed to open X11 display");
            }

            let screen = xlib::XDefaultScreen(display);
            let root_window = xlib::XRootWindow(display, screen);

            // Parse window hints for GLX configuration
            let mut context_major = 3;
            let mut context_minor = 3;
            let mut samples = 0;
            let mut double_buffer = true;
            let mut opengl_profile = OpenGLProfile::Core;

            for hint in hints {
                match hint {
                    WindowHint::ContextVersion(major, minor) => {
                        context_major = *major;
                        context_minor = *minor;
                    }
                    WindowHint::Samples(sample_count) => {
                        samples = *sample_count as i32;
                    }
                    WindowHint::DoubleBuffer(db) => {
                        double_buffer = *db;
                    }
                    WindowHint::OpenGLProfile(profile) => {
                        opengl_profile = *profile;
                    }
                    _ => {} // Other hints can be handled later
                }
            }

            // Define GLX attributes
            let mut glx_attrs = vec![
                glx::GLX_X_RENDERABLE, 1,
                glx::GLX_DRAWABLE_TYPE, glx::GLX_WINDOW_BIT,
                glx::GLX_RENDER_TYPE, glx::GLX_RGBA_BIT,
                glx::GLX_X_VISUAL_TYPE, glx::GLX_TRUE_COLOR,
                glx::GLX_RED_SIZE, 8,
                glx::GLX_GREEN_SIZE, 8,
                glx::GLX_BLUE_SIZE, 8,
                glx::GLX_ALPHA_SIZE, 8,
                glx::GLX_DEPTH_SIZE, 24,
                glx::GLX_STENCIL_SIZE, 8,
            ];

            if double_buffer {
                glx_attrs.extend_from_slice(&[glx::GLX_DOUBLEBUFFER, 1]);
            }

            if samples > 0 {
                glx_attrs.extend_from_slice(&[
                    glx::GLX_SAMPLE_BUFFERS, 1,
                    glx::GLX_SAMPLES, samples,
                ]);
            }

            glx_attrs.push(0); // Null terminate

            // Get framebuffer config
            let mut fb_count = 0;
            let fb_configs = glx::glXChooseFBConfig(
                display,
                screen,
                glx_attrs.as_ptr(),
                &mut fb_count
            );

            if fb_configs.is_null() || fb_count == 0 {
                panic!("Failed to find suitable GLX framebuffer config");
            }

            let fb_config = *fb_configs;

            // Get visual info
            let visual_info = glx::glXGetVisualFromFBConfig(display, fb_config);
            if visual_info.is_null() {
                panic!("Failed to get visual info from framebuffer config");
            }

            // Create colormap
            let colormap = xlib::XCreateColormap(
                display,
                root_window,
                (*visual_info).visual,
                xlib::AllocNone
            );

            // Set window attributes
            let mut swa = mem::zeroed::<XSetWindowAttributes>();
            swa.colormap = colormap;
            swa.event_mask = xlib::ExposureMask 
                | xlib::KeyPressMask | xlib::KeyReleaseMask
                | xlib::ButtonPressMask | xlib::ButtonReleaseMask
                | xlib::PointerMotionMask
                | xlib::StructureNotifyMask
                | xlib::FocusChangeMask;

            // Create window
            let window = xlib::XCreateWindow(
                display,
                root_window,
                0, 0,
                width, height,
                0,
                (*visual_info).depth,
                xlib::InputOutput as u32,
                (*visual_info).visual,
                xlib::CWColormap | xlib::CWEventMask,
                &mut swa
            );

            // Set window title
            let c_title = CString::new(title).unwrap();
            xlib::XStoreName(display, window, c_title.as_ptr());

            // Set up window manager protocols
            let wm_protocols_str = CString::new("WM_PROTOCOLS").unwrap();
            let wm_delete_window_str = CString::new("WM_DELETE_WINDOW").unwrap();
            
            let wm_protocols = xlib::XInternAtom(display, wm_protocols_str.as_ptr(), 0);
            let wm_delete_window = xlib::XInternAtom(display, wm_delete_window_str.as_ptr(), 0);
            
            let mut protocols = [wm_delete_window];
            xlib::XSetWMProtocols(display, window, protocols.as_mut_ptr(), 1);

            // Create OpenGL context
            let context_attribs = match opengl_profile {
                OpenGLProfile::Core => [
                    glx::arb::GLX_CONTEXT_MAJOR_VERSION_ARB, context_major as i32,
                    glx::arb::GLX_CONTEXT_MINOR_VERSION_ARB, context_minor as i32,
                    glx::arb::GLX_CONTEXT_PROFILE_MASK_ARB, glx::arb::GLX_CONTEXT_CORE_PROFILE_BIT_ARB,
                    0
                ],
                OpenGLProfile::Compatibility => [
                    glx::arb::GLX_CONTEXT_MAJOR_VERSION_ARB, context_major as i32,
                    glx::arb::GLX_CONTEXT_MINOR_VERSION_ARB, context_minor as i32,
                    glx::arb::GLX_CONTEXT_PROFILE_MASK_ARB, glx::arb::GLX_CONTEXT_COMPATIBILITY_PROFILE_BIT_ARB,
                    0
                ],
                OpenGLProfile::Any => [
                    glx::arb::GLX_CONTEXT_MAJOR_VERSION_ARB, context_major as i32,
                    glx::arb::GLX_CONTEXT_MINOR_VERSION_ARB, context_minor as i32,
                    0, 0, 0
                ]
            };

            // Try to create context with ARB extension first
            let glx_context = if let Some(create_context_attribs) = Self::get_glx_create_context_attribs_arb(display) {
                create_context_attribs(display, fb_config, ptr::null_mut(), 1, context_attribs.as_ptr())
            } else {
                // Fallback to legacy context creation
                glx::glXCreateNewContext(display, fb_config, glx::GLX_RGBA_TYPE, ptr::null_mut(), 1)
            };

            if glx_context.is_null() {
                panic!("Failed to create OpenGL context");
            }

            // Map the window
            xlib::XMapWindow(display, window);

            // Get initial window position
            let mut window_attrs = mem::zeroed::<XWindowAttributes>();
            xlib::XGetWindowAttributes(display, window, &mut window_attrs);

            // Free the framebuffer configs list
            xlib::XFree(fb_configs as *mut _);

            let mut x11_window = X11Window {
                display,
                window,
                screen,
                visual_info,
                glx_context,
                glx_fbconfig: fb_config,
                size: Size::from((width, height)),
                position: Position::from((window_attrs.x, window_attrs.y)),
                title: title.to_string(),
                should_close: false,
                event_callback: None,
                key_map: Self::create_key_map(),
                button_map: Self::create_button_map(),
                modifiers: KeyMod::new(),
                wm_delete_window,
                wm_protocols,
            };

            // Make context current
            x11_window.make_current();

            // Load OpenGL functions - this is critical for proper OpenGL operation
            info!("Loading OpenGL function pointers for X11 backend");
            gl::load_with(|symbol| {
                let symbol_cstr = CString::new(symbol).unwrap();
                match glx::glXGetProcAddress(symbol_cstr.as_ptr() as *const u8) {
                    Some(proc_addr) => proc_addr as *const _,
                    None => {
                        // Try alternative function loading method
                        match glx::glXGetProcAddressARB(symbol_cstr.as_ptr() as *const u8) {
                            Some(alt_proc_addr) => alt_proc_addr as *const _,
                            None => {
                                debug!("Failed to load OpenGL function: {}", symbol);
                                ptr::null()
                            }
                        }
                    }
                }
            });

            // Verify OpenGL context is working
            let version = unsafe {
                let version_ptr = gl::GetString(gl::VERSION);
                if version_ptr.is_null() {
                    warn!("Failed to get OpenGL version - context may not be properly initialized");
                    "Unknown".to_string()
                } else {
                    CStr::from_ptr(version_ptr as *const i8).to_string_lossy().to_string()
                }
            };
            info!("OpenGL version: {}", version);

            info!("X11 window created successfully");

            x11_window
        }
    }

    fn get_glx_create_context_attribs_arb(display: *mut Display) -> Option<unsafe extern "C" fn(*mut Display, GLXFBConfig, GLXContext, i32, *const i32) -> GLXContext> {
        unsafe {
            let proc_name = CString::new("glXCreateContextAttribsARB").unwrap();
            if let Some(proc_addr) = glx::glXGetProcAddress(proc_name.as_ptr() as *const u8) {
                Some(mem::transmute(proc_addr))
            } else {
                None
            }
        }
    }

    fn create_key_map() -> HashMap<u32, KeyCode> {
        let mut map = HashMap::new();
        
        // Letters
        map.insert(24, KeyCode::Q);  // XK_q
        map.insert(25, KeyCode::W);  // XK_w
        map.insert(26, KeyCode::E);  // XK_e
        map.insert(27, KeyCode::R);  // XK_r
        map.insert(28, KeyCode::T);  // XK_t
        map.insert(29, KeyCode::Y);  // XK_y
        map.insert(30, KeyCode::U);  // XK_u
        map.insert(31, KeyCode::I);  // XK_i
        map.insert(32, KeyCode::O);  // XK_o
        map.insert(33, KeyCode::P);  // XK_p
        map.insert(38, KeyCode::A);  // XK_a
        map.insert(39, KeyCode::S);  // XK_s
        map.insert(40, KeyCode::D);  // XK_d
        map.insert(41, KeyCode::F);  // XK_f
        map.insert(42, KeyCode::G);  // XK_g
        map.insert(43, KeyCode::H);  // XK_h
        map.insert(44, KeyCode::J);  // XK_j
        map.insert(45, KeyCode::K);  // XK_k
        map.insert(46, KeyCode::L);  // XK_l
        map.insert(52, KeyCode::Z);  // XK_z
        map.insert(53, KeyCode::X);  // XK_x
        map.insert(54, KeyCode::C);  // XK_c
        map.insert(55, KeyCode::V);  // XK_v
        map.insert(56, KeyCode::B);  // XK_b
        map.insert(57, KeyCode::N);  // XK_n
        map.insert(58, KeyCode::M);  // XK_m

        // Numbers
        map.insert(10, KeyCode::Num1);  // XK_1
        map.insert(11, KeyCode::Num2);  // XK_2
        map.insert(12, KeyCode::Num3);  // XK_3
        map.insert(13, KeyCode::Num4);  // XK_4
        map.insert(14, KeyCode::Num5);  // XK_5
        map.insert(15, KeyCode::Num6);  // XK_6
        map.insert(16, KeyCode::Num7);  // XK_7
        map.insert(17, KeyCode::Num8);  // XK_8
        map.insert(18, KeyCode::Num9);  // XK_9
        map.insert(19, KeyCode::Num0);  // XK_0

        // Special keys
        map.insert(9, KeyCode::Escape);     // XK_Escape
        map.insert(36, KeyCode::Enter);     // XK_Return
        map.insert(65, KeyCode::Space);     // XK_space
        map.insert(22, KeyCode::Backspace); // XK_BackSpace
        map.insert(23, KeyCode::Tab);       // XK_Tab
        map.insert(50, KeyCode::LeftShift); // XK_Shift_L
        map.insert(62, KeyCode::RightShift);// XK_Shift_R
        map.insert(37, KeyCode::LeftControl);  // XK_Control_L
        map.insert(105, KeyCode::RightControl);// XK_Control_R
        map.insert(64, KeyCode::LeftAlt);   // XK_Alt_L
        map.insert(108, KeyCode::RightAlt); // XK_Alt_R

        // Arrow keys
        map.insert(111, KeyCode::Up);       // XK_Up
        map.insert(116, KeyCode::Down);     // XK_Down
        map.insert(113, KeyCode::Left);     // XK_Left
        map.insert(114, KeyCode::Right);    // XK_Right

        // Function keys
        map.insert(67, KeyCode::F1);        // XK_F1
        map.insert(68, KeyCode::F2);        // XK_F2
        map.insert(69, KeyCode::F3);        // XK_F3
        map.insert(70, KeyCode::F4);        // XK_F4
        map.insert(71, KeyCode::F5);        // XK_F5
        map.insert(72, KeyCode::F6);        // XK_F6
        map.insert(73, KeyCode::F7);        // XK_F7
        map.insert(74, KeyCode::F8);        // XK_F8
        map.insert(75, KeyCode::F9);        // XK_F9
        map.insert(76, KeyCode::F10);       // XK_F10
        map.insert(95, KeyCode::F11);       // XK_F11
        map.insert(96, KeyCode::F12);       // XK_F12

        map
    }

    fn create_button_map() -> HashMap<u32, MouseButton> {
        let mut map = HashMap::new();
        map.insert(1, MouseButton::Left);
        map.insert(2, MouseButton::Middle);
        map.insert(3, MouseButton::Right);
        map
    }

    fn translate_key(&self, keycode: u32) -> KeyCode {
        self.key_map.get(&keycode).copied().unwrap_or(KeyCode::Unknown)
    }

    fn translate_button(&self, button: u32) -> MouseButton {
        self.button_map.get(&button).copied().unwrap_or(MouseButton::Button1) // Default fallback
    }

    fn update_modifiers(&mut self, state: u32) {
        self.modifiers = KeyMod::new();
        
        if state & xlib::ShiftMask != 0 {
            self.modifiers.shift = true;
        }
        if state & xlib::ControlMask != 0 {
            self.modifiers.control = true;
        }
        if state & xlib::Mod1Mask != 0 {  // Alt
            self.modifiers.alt = true;
        }
        if state & xlib::Mod4Mask != 0 {  // Super (Windows key)
            self.modifiers.super_key = true;
        }
    }

    /// Reload OpenGL function pointers - critical for backend switching
    pub fn reload_opengl_functions(&mut self) {
        info!("Reloading OpenGL function pointers for X11 backend after context switch");
        
        unsafe {
            // Ensure context is current first
            self.make_current();
            
            // Reload all OpenGL function pointers
            gl::load_with(|symbol| {
                let symbol_cstr = CString::new(symbol).unwrap();
                match glx::glXGetProcAddress(symbol_cstr.as_ptr() as *const u8) {
                    Some(proc_addr) => proc_addr as *const _,
                    None => {
                        // Try alternative function loading method
                        match glx::glXGetProcAddressARB(symbol_cstr.as_ptr() as *const u8) {
                            Some(alt_proc_addr) => alt_proc_addr as *const _,
                            None => {
                                debug!("Failed to reload OpenGL function: {}", symbol);
                                ptr::null()
                            }
                        }
                    }
                }
            });

            // Verify the context is working after reload
            let version = unsafe {
                let version_ptr = gl::GetString(gl::VERSION);
                if version_ptr.is_null() {
                    warn!("Failed to get OpenGL version after function reload");
                    "Unknown".to_string()
                } else {
                    CStr::from_ptr(version_ptr as *const i8).to_string_lossy().to_string()
                }
            };
            info!("OpenGL function pointers reloaded successfully, version: {}", version);
        }
    }
}

impl Window for X11Window {
    fn update(&mut self) {
        unsafe {
            xlib::XFlush(self.display);
        }
        // Swap buffers to display rendered content (fixes black screen issue)
        self.swap_buffers();
    }

    fn process_events(&mut self) {
        unsafe {
            while xlib::XPending(self.display) > 0 {
                let mut event = mem::zeroed::<XEvent>();
                xlib::XNextEvent(self.display, &mut event);

                match event.get_type() {
                    xlib::KeyPress => {
                        let key_event = xlib::XKeyEvent::from(event);
                        self.update_modifiers(key_event.state);
                        
                        let key_code = self.translate_key(key_event.keycode);
                        
                        // Handle escape key
                        if key_code == KeyCode::Escape {
                            self.should_close = true;
                        }

                        if let Some(callback) = &self.event_callback {
                            let key_event = KeyEvent {
                                key: key_code,
                                action: KeyAction::Press,
                                mods: self.modifiers,
                            };
                            let event = Event::new(EventData::Key(key_event));
                            let mut callback = callback.lock().unwrap();
                            callback(event);
                        }
                    }
                    xlib::KeyRelease => {
                        let key_event = xlib::XKeyEvent::from(event);
                        self.update_modifiers(key_event.state);
                        
                        let key_code = self.translate_key(key_event.keycode);

                        if let Some(callback) = &self.event_callback {
                            let key_event = KeyEvent {
                                key: key_code,
                                action: KeyAction::Release,
                                mods: self.modifiers,
                            };
                            let event = Event::new(EventData::Key(key_event));
                            let mut callback = callback.lock().unwrap();
                            callback(event);
                        }
                    }
                    xlib::ButtonPress => {
                        let button_event = xlib::XButtonEvent::from(event);
                        self.update_modifiers(button_event.state);

                        match button_event.button {
                            4 => {
                                // Scroll up
                                if let Some(callback) = &self.event_callback {
                                    let scroll_event = MouseScrollEvent { 
                                        x_offset: 0.0, 
                                        y_offset: 1.0 
                                    };
                                    let event = Event::new(EventData::MouseScroll(scroll_event));
                                    let mut callback = callback.lock().unwrap();
                                    callback(event);
                                }
                            }
                            5 => {
                                // Scroll down
                                if let Some(callback) = &self.event_callback {
                                    let scroll_event = MouseScrollEvent { 
                                        x_offset: 0.0, 
                                        y_offset: -1.0 
                                    };
                                    let event = Event::new(EventData::MouseScroll(scroll_event));
                                    let mut callback = callback.lock().unwrap();
                                    callback(event);
                                }
                            }
                            _ => {
                                // Regular mouse button
                                let mouse_button = self.translate_button(button_event.button);

                                if let Some(callback) = &self.event_callback {
                                    let button_event = MouseButtonEvent {
                                        button: mouse_button,
                                        action: KeyAction::Press,
                                        mods: self.modifiers,
                                    };
                                    let event = Event::new(EventData::MouseButton(button_event));
                                    let mut callback = callback.lock().unwrap();
                                    callback(event);
                                }
                            }
                        }
                    }
                    xlib::ButtonRelease => {
                        let button_event = xlib::XButtonEvent::from(event);
                        self.update_modifiers(button_event.state);

                        // Only handle regular buttons for release (not scroll)
                        if button_event.button <= 3 {
                            let mouse_button = self.translate_button(button_event.button);

                            if let Some(callback) = &self.event_callback {
                                let button_event = MouseButtonEvent {
                                    button: mouse_button,
                                    action: KeyAction::Release,
                                    mods: self.modifiers,
                                };
                                let event = Event::new(EventData::MouseButton(button_event));
                                let mut callback = callback.lock().unwrap();
                                callback(event);
                            }
                        }
                    }
                    xlib::MotionNotify => {
                        let motion_event = xlib::XMotionEvent::from(event);

                        if let Some(callback) = &self.event_callback {
                            let move_event = MouseMoveEvent {
                                x: motion_event.x as f64,
                                y: motion_event.y as f64,
                            };
                            let event = Event::new(EventData::MouseMove(move_event));
                            let mut callback = callback.lock().unwrap();
                            callback(event);
                        }
                    }
                    xlib::ConfigureNotify => {
                        let configure_event = xlib::XConfigureEvent::from(event);
                        
                        // Check for size change
                        let new_size = Size::from((configure_event.width as u32, configure_event.height as u32));
                        if new_size.0 != self.size.0 || new_size.1 != self.size.1 {
                            self.size = new_size;

                            // Update OpenGL viewport
                            gl::Viewport(0, 0, new_size.0 as i32, new_size.1 as i32);

                            if let Some(callback) = &self.event_callback {
                                let resize_event = WindowResizeEvent {
                                    width: new_size.0,
                                    height: new_size.1,
                                };
                                let event = Event::new(EventData::WindowResize(resize_event));
                                let mut callback = callback.lock().unwrap();
                                callback(event);
                            }
                        }

                        // Check for position change
                        let new_position = Position::from((configure_event.x, configure_event.y));
                        if new_position.0 != self.position.0 || new_position.1 != self.position.1 {
                            self.position = new_position;

                            if let Some(callback) = &self.event_callback {
                                let move_event = WindowMoveEvent {
                                    x: new_position.0,
                                    y: new_position.1,
                                };
                                let event = Event::new(EventData::WindowMove(move_event));
                                let mut callback = callback.lock().unwrap();
                                callback(event);
                            }
                        }
                    }
                    xlib::ClientMessage => {
                        let client_message = xlib::XClientMessageEvent::from(event);
                        if client_message.message_type == self.wm_protocols {
                            if client_message.data.get_long(0) as xlib::Atom == self.wm_delete_window {
                                self.should_close = true;

                                if let Some(callback) = &self.event_callback {
                                    let close_event = WindowCloseEvent;
                                    let event = Event::new(EventData::WindowClose(close_event));
                                    let mut callback = callback.lock().unwrap();
                                    callback(event);
                                }
                            }
                        }
                    }
                    xlib::Expose => {
                        // Window needs to be redrawn
                        // The application will handle this in its render loop
                    }
                    _ => {}
                }
            }
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
        unsafe {
            xlib::XMoveWindow(self.display, self.window, position.0, position.1);
        }
    }

    fn position(&self) -> &Position {
        &self.position
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
        unsafe {
            xlib::XResizeWindow(self.display, self.window, size.0, size.1);
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
        unsafe {
            let c_title = CString::new(title).unwrap();
            xlib::XStoreName(self.display, self.window, c_title.as_ptr());
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

impl OpenGLWindow for X11Window {
    fn is_current(&self) -> bool {
        unsafe {
            glx::glXGetCurrentContext() == self.glx_context
        }
    }

    fn make_current(&mut self) {
        unsafe {
            // Check if context is already current to avoid unnecessary calls
            if glx::glXGetCurrentContext() == self.glx_context {
                debug!("OpenGL context already current");
                return;
            }

            // Validate context before making it current
            if self.glx_context.is_null() {
                error!("Cannot make current: OpenGL context is null");
                return;
            }

            if self.display.is_null() {
                error!("Cannot make current: X11 display is null");
                return;
            }

            if self.window == 0 {
                error!("Cannot make current: X11 window is invalid");
                return;
            }

            debug!("Making X11 OpenGL context current");
            let result = glx::glXMakeCurrent(self.display, self.window, self.glx_context);
            if result == 0 {
                error!("Failed to make OpenGL context current - glXMakeCurrent returned 0");
                
                // Try to get more specific error information
                let current_context = glx::glXGetCurrentContext();
                let current_drawable = glx::glXGetCurrentDrawable();
                error!("Current context: {:?}, Current drawable: {:?}", current_context, current_drawable);
            } else {
                debug!("OpenGL context made current successfully");
            }
        }
    }

    fn swap_buffers(&mut self) {
        unsafe {
            glx::glXSwapBuffers(self.display, self.window);
        }
    }

    fn reload_opengl_functions(&mut self) {
        info!("Reloading OpenGL function pointers for X11 backend after context switch");
        
        unsafe {
            // Validate context state before attempting reload
            if self.glx_context.is_null() {
                error!("Cannot reload OpenGL functions: GLX context is null");
                return;
            }

            if self.display.is_null() {
                error!("Cannot reload OpenGL functions: X11 display is null");
                return;
            }

            // Ensure context is current first
            self.make_current();
            
            // Verify context is actually current after make_current call
            let current_context = glx::glXGetCurrentContext();
            if current_context != self.glx_context {
                error!("Failed to make X11 OpenGL context current for function reload");
                error!("Expected context: {:?}, Current context: {:?}", self.glx_context, current_context);
                return;
            }

            debug!("X11 OpenGL context is current, proceeding with function reload");
            
            // Track failed function loads for diagnostics
            let mut failed_functions = 0;
            let mut total_functions = 0;
            
            // Reload all OpenGL function pointers
            gl::load_with(|symbol| {
                total_functions += 1;
                let symbol_cstr = CString::new(symbol).unwrap();
                match glx::glXGetProcAddress(symbol_cstr.as_ptr() as *const u8) {
                    Some(proc_addr) => proc_addr as *const _,
                    None => {
                        // Try alternative function loading method
                        match glx::glXGetProcAddressARB(symbol_cstr.as_ptr() as *const u8) {
                            Some(alt_proc_addr) => alt_proc_addr as *const _,
                            None => {
                                failed_functions += 1;
                                debug!("Failed to reload OpenGL function: {}", symbol);
                                ptr::null()
                            }
                        }
                    }
                }
            });

            // Report function loading statistics
            if failed_functions > 0 {
                warn!("Failed to load {} out of {} OpenGL functions", failed_functions, total_functions);
            } else {
                debug!("Successfully loaded all {} OpenGL functions", total_functions);
            }

            // Verify the context is working after reload
            let version = gl::GetString(gl::VERSION);
            if version.is_null() {
                error!("Failed to get OpenGL version after function reload - context may be invalid");
                error!("This indicates a serious OpenGL context issue with X11 backend");
            } else {
                let version_str = CStr::from_ptr(version as *const i8).to_string_lossy();
                info!("✓ OpenGL function pointers reloaded successfully for X11, version: {}", version_str);
                
                // Additional context validation
                let renderer = gl::GetString(gl::RENDERER);
                if !renderer.is_null() {
                    let renderer_str = CStr::from_ptr(renderer as *const i8).to_string_lossy();
                    debug!("OpenGL renderer: {}", renderer_str);
                }
            }
        }
    }
}

impl Drop for X11Window {
    fn drop(&mut self) {
        debug!("Dropping X11Window and cleaning up resources");
        
        unsafe {
            // Cleanup OpenGL context first
            if !self.glx_context.is_null() {
                debug!("Cleaning up OpenGL context");
                
                // Make sure we're not leaving a dangling current context
                let current_context = glx::glXGetCurrentContext();
                if current_context == self.glx_context {
                    debug!("Releasing current OpenGL context");
                    let result = glx::glXMakeCurrent(self.display, 0, ptr::null_mut());
                    if result == 0 {
                        warn!("Failed to release current OpenGL context during cleanup");
                    }
                }
                
                // Destroy the context
                glx::glXDestroyContext(self.display, self.glx_context);
                debug!("OpenGL context destroyed");
            }

            // Cleanup X11 window
            if self.window != 0 {
                debug!("Destroying X11 window");
                xlib::XDestroyWindow(self.display, self.window);
                self.window = 0;
            }

            // Free visual info
            if !self.visual_info.is_null() {
                debug!("Freeing visual info");
                xlib::XFree(self.visual_info as *mut _);
                self.visual_info = ptr::null_mut();
            }

            // Close display connection last
            if !self.display.is_null() {
                debug!("Closing X11 display connection");
                // Flush any remaining requests before closing
                xlib::XFlush(self.display);
                xlib::XCloseDisplay(self.display);
                self.display = ptr::null_mut();
            }
            
            debug!("X11Window cleanup completed");
        }
    }
}

/// X11 window factory implementation
pub struct X11WindowFactory;

impl WindowFactory for X11WindowFactory {
    fn create_window(&self, width: u32, height: u32, title: &str) -> Box<dyn Window> {
        info!("Creating X11 window: {} ({}x{})", title, width, height);
        Box::new(X11Window::new(width, height, title))
    }
    
    fn create_window_with_hints(&self, width: u32, height: u32, title: &str, hints: &[WindowHint]) -> Box<dyn Window> {
        info!("Creating X11 window with hints: {} ({}x{})", title, width, height);
        Box::new(X11Window::with_hints(width, height, title, hints))
    }
    
    fn supports_feature(&self, feature: WindowFeature) -> bool {
        match feature {
            WindowFeature::OpenGL => true,
            WindowFeature::Vulkan => false, // Not implemented yet
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
        "X11"
    }
    
    fn backend_version(&self) -> Option<String> {
        Some("X11R6+".to_string())
    }
}