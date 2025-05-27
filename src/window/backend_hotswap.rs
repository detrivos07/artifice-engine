use crate::events::core::{Event, EventData, EventQueue};
use crate::io::{Window, WindowHint, Size, Position};
use crate::window::factory::{WindowFactory, WindowBackendRegistry};
use artifice_logging::{debug, info, warn, error};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Configuration for window backend hotswap operations
#[derive(Debug, Clone)]
pub struct WindowBackendHotswapConfig {
    /// Maximum time to wait for backend switch completion
    pub switch_timeout: Duration,
    /// Whether to preserve window state during switches
    pub preserve_state: bool,
    /// Whether to buffer events during transitions
    pub buffer_events: bool,
    /// Maximum number of events to buffer during transition
    pub max_buffered_events: usize,
    /// Whether to validate backend before switching
    pub validate_backend: bool,
}

impl Default for WindowBackendHotswapConfig {
    fn default() -> Self {
        Self {
            switch_timeout: Duration::from_secs(5),
            preserve_state: true,
            buffer_events: true,
            max_buffered_events: 1000,
            validate_backend: true,
        }
    }
}

/// Represents the current state of a window for preservation during hot reload
#[derive(Debug, Clone)]
pub struct WindowState {
    pub size: Size,
    pub position: Position,
    pub title: String,
    pub should_close: bool,
    pub hints: Vec<WindowHint>,
}

impl WindowState {
    pub fn capture_from_window(window: &dyn Window) -> Self {
        Self {
            size: *window.size(),
            position: *window.position(),
            title: window.title().to_string(),
            should_close: window.should_close(),
            hints: Vec::new(), // Would need window to expose hints
        }
    }

    pub fn apply_to_window(&self, window: &mut dyn Window) {
        window.set_size(self.size);
        window.set_position(self.position);
        window.set_title(&self.title);
        if self.should_close {
            window.set_should_close();
        }
    }
}

/// Status of a hot reload operation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowBackendHotswapStatus {
    Idle,
    InProgress,
    Completed,
    Failed(String),
    Cancelled,
}

/// Result of a hot reload operation
#[derive(Debug)]
pub struct WindowBackendHotswapResult {
    pub status: WindowBackendHotswapStatus,
    pub old_backend: String,
    pub new_backend: String,
    pub duration: Duration,
    pub events_buffered: usize,
    pub errors: Vec<String>,
}

/// Event buffer for storing events during backend transitions
pub struct EventBuffer {
    events: Vec<Event>,
    max_size: usize,
    enabled: bool,
}

impl EventBuffer {
    pub fn new(max_size: usize) -> Self {
        Self {
            events: Vec::new(),
            max_size,
            enabled: true,
        }
    }

    pub fn push(&mut self, event: Event) -> bool {
        if !self.enabled {
            return false;
        }

        if self.events.len() >= self.max_size {
            warn!("Event buffer full, dropping oldest event");
            self.events.remove(0);
        }

        self.events.push(event);
        true
    }

    pub fn drain(&mut self) -> Vec<Event> {
        std::mem::take(&mut self.events)
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.clear();
        }
    }
}

/// Manages hot reloading of window backends
pub struct WindowBackendHotswapManager {
    registry: WindowBackendRegistry,
    config: WindowBackendHotswapConfig,
    current_backend: Option<String>,
    status: WindowBackendHotswapStatus,
    event_buffer: EventBuffer,
    preserved_state: Option<WindowState>,
    switch_start_time: Option<Instant>,
    validation_cache: HashMap<String, bool>,
}

impl WindowBackendHotswapManager {
    pub fn new(registry: WindowBackendRegistry) -> Self {
        Self::with_config(registry, WindowBackendHotswapConfig::default())
    }

    pub fn with_config(registry: WindowBackendRegistry, config: WindowBackendHotswapConfig) -> Self {
        let event_buffer = EventBuffer::new(config.max_buffered_events);

        Self {
            registry,
            config,
            current_backend: None,
            status: WindowBackendHotswapStatus::Idle,
            event_buffer,
            preserved_state: None,
            switch_start_time: None,
            validation_cache: HashMap::new(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &WindowBackendHotswapConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: WindowBackendHotswapConfig) {
        self.event_buffer = EventBuffer::new(config.max_buffered_events);
        self.event_buffer.set_enabled(config.buffer_events);
        self.config = config;
    }

    /// Get the current backend name
    pub fn current_backend(&self) -> Option<&String> {
        self.current_backend.as_ref()
    }

    /// Get the current hot reload status
    pub fn status(&self) -> &WindowBackendHotswapStatus {
        &self.status
    }

    /// Check if a hot reload operation is in progress
    pub fn is_reloading(&self) -> bool {
        matches!(self.status, WindowBackendHotswapStatus::InProgress)
    }

    /// Get available backend names
    pub fn available_backends(&self) -> Vec<&String> {
        self.registry.available_backends()
    }

    /// Validate that a backend is available and functional
    pub fn validate_backend(&mut self, backend_name: &str) -> bool {
        if let Some(&cached_result) = self.validation_cache.get(backend_name) {
            return cached_result;
        }

        let is_valid = self.registry.get_backend_info(backend_name).is_some();
        
        // Additional validation could include:
        // - Testing window creation
        // - Checking system requirements
        // - Verifying driver availability
        
        self.validation_cache.insert(backend_name.to_string(), is_valid);
        is_valid
    }

    /// Start a hot reload operation to switch to a new backend
    pub fn start_reload(&mut self, target_backend: &str, current_window: &dyn Window) -> Result<(), String> {
        if self.is_reloading() {
            return Err("Hot reload already in progress".to_string());
        }

        // Validate target backend
        if self.config.validate_backend && !self.validate_backend(target_backend) {
            return Err(format!("Backend '{}' is not available or invalid", target_backend));
        }

        // Don't reload to the same backend
        if let Some(ref current) = self.current_backend {
            if current == target_backend {
                return Err("Already using the specified backend".to_string());
            }
        }

        info!("Starting hot reload to backend: {}", target_backend);

        // Preserve current window state
        if self.config.preserve_state {
            self.preserved_state = Some(WindowState::capture_from_window(current_window));
            debug!("Preserved window state for hot reload");
        }

        // Enable event buffering
        if self.config.buffer_events {
            self.event_buffer.set_enabled(true);
            self.event_buffer.clear();
            debug!("Event buffering enabled for hot reload");
        }

        // Update status
        self.status = WindowBackendHotswapStatus::InProgress;
        self.switch_start_time = Some(Instant::now());

        Ok(())
    }

    /// Complete a hot reload operation with the new window
    pub fn complete_reload(&mut self, target_backend: &str, new_window: &mut dyn Window) -> WindowBackendHotswapResult {
        let start_time = self.switch_start_time.unwrap_or_else(Instant::now);
        let duration = start_time.elapsed();
        let events_buffered = self.event_buffer.len();
        let old_backend = self.current_backend.clone().unwrap_or_else(|| "unknown".to_string());

        // Check for timeout
        if duration > self.config.switch_timeout {
            let error_msg = format!("Hot reload timed out after {:?}", duration);
            warn!("{}", error_msg);
            
            self.status = WindowBackendHotswapStatus::Failed(error_msg.clone());
            return WindowBackendHotswapResult {
                status: self.status.clone(),
                old_backend,
                new_backend: target_backend.to_string(),
                duration,
                events_buffered,
                errors: vec![error_msg],
            };
        }

        let mut errors = Vec::new();

        // Apply preserved state
        if let Some(ref state) = self.preserved_state {
            state.apply_to_window(new_window);
            debug!("Applied preserved window state");
        }

        // Replay buffered events
        if self.config.buffer_events {
            let buffered_events = self.event_buffer.drain();
            if !buffered_events.is_empty() {
                // Set up event callback to replay events
                if let Some(callback) = new_window.get_event_callback() {
                    for event in buffered_events {
                        if let Ok(mut cb) = callback.lock() {
                            cb(event);
                        }
                    }
                }
                debug!("Replayed {} buffered events", events_buffered);
            }
        }

        // Update state
        self.current_backend = Some(target_backend.to_string());
        self.status = WindowBackendHotswapStatus::Completed;
        self.preserved_state = None;
        self.switch_start_time = None;
        self.event_buffer.set_enabled(false);

        info!("Hot reload completed successfully: {} -> {} ({:?})", 
              old_backend, target_backend, duration);

        WindowBackendHotswapResult {
            status: self.status.clone(),
            old_backend,
            new_backend: target_backend.to_string(),
            duration,
            events_buffered,
            errors,
        }
    }

    /// Cancel an in-progress hot reload operation
    pub fn cancel_reload(&mut self) -> bool {
        if !self.is_reloading() {
            return false;
        }

        warn!("Hot reload operation cancelled");

        self.status = WindowBackendHotswapStatus::Cancelled;
        self.preserved_state = None;
        self.switch_start_time = None;
        self.event_buffer.set_enabled(false);
        self.event_buffer.clear();

        true
    }

    /// Handle an event during hot reload (for buffering)
    pub fn handle_event(&mut self, event: Event) -> bool {
        if self.is_reloading() && self.config.buffer_events {
            return self.event_buffer.push(event);
        }
        false
    }

    /// Create a new window with the specified backend
    pub fn create_window_with_backend(
        &self,
        backend_name: &str,
        width: u32,
        height: u32,
        title: &str,
        hints: &[WindowHint],
    ) -> Result<Box<dyn Window>, String> {
        if hints.is_empty() {
            self.registry
                .create_window(backend_name, width, height, title)
                .ok_or_else(|| format!("Failed to create window with backend '{}'", backend_name))
        } else {
            self.registry
                .create_window_with_hints(backend_name, width, height, title, hints)
                .ok_or_else(|| format!("Failed to create window with backend '{}'", backend_name))
        }
    }

    /// Get statistics about the hot reload manager
    pub fn get_stats(&self) -> WindowBackendHotswapStats {
        WindowBackendHotswapStats {
            current_backend: self.current_backend.clone(),
            status: self.status.clone(),
            available_backends: self.available_backends().len(),
            validation_cache_size: self.validation_cache.len(),
            buffered_events: self.event_buffer.len(),
            buffer_enabled: self.config.buffer_events,
            switch_in_progress: self.is_reloading(),
            switch_duration: self.switch_start_time.map(|start| start.elapsed()),
        }
    }

    /// Clear the validation cache
    pub fn clear_validation_cache(&mut self) {
        self.validation_cache.clear();
        debug!("Validation cache cleared");
    }

    /// Get the window backend registry
    pub fn registry(&self) -> &WindowBackendRegistry {
        &self.registry
    }

    /// Get a mutable reference to the window backend registry
    pub fn registry_mut(&mut self) -> &mut WindowBackendRegistry {
        &mut self.registry
    }
}

/// Statistics about the hot reload manager
#[derive(Debug, Clone)]
pub struct WindowBackendHotswapStats {
    pub current_backend: Option<String>,
    pub status: WindowBackendHotswapStatus,
    pub available_backends: usize,
    pub validation_cache_size: usize,
    pub buffered_events: usize,
    pub buffer_enabled: bool,
    pub switch_in_progress: bool,
    pub switch_duration: Option<Duration>,
}

/// Convenient builder for hot reload operations
pub struct WindowBackendHotswapBuilder {
    target_backend: String,
    preserve_state: Option<bool>,
    buffer_events: Option<bool>,
    timeout: Option<Duration>,
    validate: Option<bool>,
}

impl WindowBackendHotswapBuilder {
    pub fn new(target_backend: impl Into<String>) -> Self {
        Self {
            target_backend: target_backend.into(),
            preserve_state: None,
            buffer_events: None,
            timeout: None,
            validate: None,
        }
    }

    pub fn preserve_state(mut self, preserve: bool) -> Self {
        self.preserve_state = Some(preserve);
        self
    }

    pub fn buffer_events(mut self, buffer: bool) -> Self {
        self.buffer_events = Some(buffer);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn validate_backend(mut self, validate: bool) -> WindowBackendHotswapBuilder {
        self.validate = Some(validate);
        self
    }

    pub fn execute(self, manager: &mut WindowBackendHotswapManager, current_window: &dyn Window) -> Result<(), String> {
        // Apply temporary config changes
        let original_config = manager.config().clone();
        let mut temp_config = original_config.clone();

        if let Some(preserve) = self.preserve_state {
            temp_config.preserve_state = preserve;
        }
        if let Some(buffer) = self.buffer_events {
            temp_config.buffer_events = buffer;
        }
        if let Some(timeout) = self.timeout {
            temp_config.switch_timeout = timeout;
        }
        if let Some(validate) = self.validate {
            temp_config.validate_backend = validate;
        }

        // Apply temporary config
        manager.set_config(temp_config);

        // Execute the reload
        let result = manager.start_reload(&self.target_backend, current_window);

        // Restore original config
        manager.set_config(original_config);

        result
    }
}

/// Factory for creating hot reload managers with different configurations
pub struct WindowBackendHotswapFactory;

impl WindowBackendHotswapFactory {
    /// Create a hot reload manager with default configuration
    pub fn create_default(registry: WindowBackendRegistry) -> WindowBackendHotswapManager {
        WindowBackendHotswapManager::new(registry)
    }

    /// Create a hot reload manager optimized for fast switching
    pub fn create_fast_switch(registry: WindowBackendRegistry) -> WindowBackendHotswapManager {
        let config = WindowBackendHotswapConfig {
            switch_timeout: Duration::from_millis(500),
            preserve_state: false,
            buffer_events: false,
            max_buffered_events: 0,
            validate_backend: false,
        };
        WindowBackendHotswapManager::with_config(registry, config)
    }

    /// Create a hot reload manager optimized for reliability
    pub fn create_reliable(registry: WindowBackendRegistry) -> WindowBackendHotswapManager {
        let config = WindowBackendHotswapConfig {
            switch_timeout: Duration::from_secs(10),
            preserve_state: true,
            buffer_events: true,
            max_buffered_events: 2000,
            validate_backend: true,
        };
        WindowBackendHotswapManager::with_config(registry, config)
    }

    /// Create a hot reload manager with custom configuration
    pub fn create_custom(registry: WindowBackendRegistry, config: WindowBackendHotswapConfig) -> WindowBackendHotswapManager {
        WindowBackendHotswapManager::with_config(registry, config)
    }
}