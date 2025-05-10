//! ArtificeCore Logging Library
//!
//! This library provides logging functionality for Artifice-Engine applications.
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Mutex;

/// An enumeration representing different log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    FATAL = 0,
    ERROR = 1,
    WARN = 2,
    INFO = 3,
    DEBUG = 4,
    TRACE = 5,
}

impl LogLevel {
    /// Returns the log level as a string.
    fn as_str(&self) -> &str {
        match self {
            LogLevel::FATAL => "FATAL",
            LogLevel::ERROR => "ERROR",
            LogLevel::WARN => "WARN",
            LogLevel::INFO => "INFO",
            LogLevel::DEBUG => "DEBUG",
            LogLevel::TRACE => "TRACE",
        }
    }

    /// Returns the log level as a colored string (for terminals that support ANSI color codes)
    fn as_colored_str(&self) -> String {
        match self {
            LogLevel::FATAL => format!("\x1b[1;31m{}\x1b[0m", self.as_str()), // Bold Red
            LogLevel::ERROR => format!("\x1b[31m{}\x1b[0m", self.as_str()),   // Red
            LogLevel::WARN => format!("\x1b[33m{}\x1b[0m", self.as_str()),    // Yellow
            LogLevel::INFO => format!("\x1b[32m{}\x1b[0m", self.as_str()),    // Green
            LogLevel::DEBUG => format!("\x1b[36m{}\x1b[0m", self.as_str()),   // Cyan
            LogLevel::TRACE => format!("\x1b[35m{}\x1b[0m", self.as_str()),   // Magenta
        }
    }
}

// Global current log level
static CURRENT_LOG_LEVEL: AtomicU8 = AtomicU8::new(LogLevel::INFO as u8);

// Global use colors flag
static USE_COLORS: AtomicU8 = AtomicU8::new(1); // 1 = true, 0 = false

// Custom logger function type
type LoggerFn = dyn Fn(LogLevel, &str) + Send + Sync;

// Global custom logger
static CUSTOM_LOGGER: Mutex<Option<Box<LoggerFn>>> = Mutex::new(None);

/// Set the global log level
pub fn set_log_level(level: LogLevel) {
    CURRENT_LOG_LEVEL.store(level as u8, Ordering::SeqCst);
}

/// Get the current global log level
pub fn get_log_level() -> LogLevel {
    let level = CURRENT_LOG_LEVEL.load(Ordering::SeqCst);
    match level {
        0 => LogLevel::FATAL,
        1 => LogLevel::ERROR,
        2 => LogLevel::WARN,
        3 => LogLevel::INFO,
        4 => LogLevel::DEBUG,
        5 => LogLevel::TRACE,
        _ => LogLevel::INFO, // Default to INFO if unexpected value
    }
}

/// Enable or disable colored log output
pub fn set_colors_enabled(enabled: bool) {
    USE_COLORS.store(if enabled { 1 } else { 0 }, Ordering::SeqCst);
}

/// Check if colors are enabled
fn are_colors_enabled() -> bool {
    USE_COLORS.load(Ordering::SeqCst) == 1
}

/// Set a custom logger function
pub fn set_custom_logger<F>(logger: F)
where
    F: Fn(LogLevel, &str) + Send + Sync + 'static,
{
    let mut custom_logger = CUSTOM_LOGGER.lock().unwrap();
    *custom_logger = Some(Box::new(logger));
}

/// Remove any custom logger
pub fn remove_custom_logger() {
    let mut custom_logger = CUSTOM_LOGGER.lock().unwrap();
    *custom_logger = None;
}

/// Automatically logs a fatal message with the given message.
pub fn fatal(message: &str) {
    log(LogLevel::FATAL, message);
}

/// Automatically logs an error message with the given message.
pub fn error(message: &str) {
    log(LogLevel::ERROR, message);
}

/// Automatically logs a warning message with the given message.
pub fn warn(message: &str) {
    log(LogLevel::WARN, message);
}

/// Automatically logs an informational message with the given message.
pub fn info(message: &str) {
    log(LogLevel::INFO, message);
}

/// Automatically logs a debug message with the given message.
pub fn debug(message: &str) {
    log(LogLevel::DEBUG, message);
}

/// Automatically logs a trace message with the given message.
pub fn trace(message: &str) {
    log(LogLevel::TRACE, message);
}

/// Logging function for logging messages with a specific log level.
/// This will format the log message with the log level, timestamp, and append the message upon that.
fn log(level: LogLevel, message: &str) {
    // Check if this log level should be output
    if level as u8 > CURRENT_LOG_LEVEL.load(Ordering::SeqCst) {
        return;
    }

    // Check if we have a custom logger
    let custom_logger = CUSTOM_LOGGER.lock().unwrap();
    if let Some(logger) = &*custom_logger {
        logger(level, message);
        return;
    }

    // Format and output the log message
    let now = chrono::offset::Local::now();
    let level_str = if are_colors_enabled() {
        level.as_colored_str()
    } else {
        level.as_str().to_string()
    };

    println!(
        "[{}][{}]: {}",
        level_str,
        now.format("%Y-%m-%d %H:%M:%S%.3f"),
        message
    );
}

/// Function to initialize the logger - called manually rather than via a feature
pub fn init() {
    // Set log level from environment variable if available
    if let Ok(level) = std::env::var("ARTIFICE_LOG_LEVEL") {
        match level.to_uppercase().as_str() {
            "FATAL" => set_log_level(LogLevel::FATAL),
            "ERROR" => set_log_level(LogLevel::ERROR),
            "WARN" => set_log_level(LogLevel::WARN),
            "INFO" => set_log_level(LogLevel::INFO),
            "DEBUG" => set_log_level(LogLevel::DEBUG),
            "TRACE" => set_log_level(LogLevel::TRACE),
            _ => set_log_level(LogLevel::INFO),
        }
    }

    // Enable/disable colors from environment variable if available
    if let Ok(colors) = std::env::var("ARTIFICE_LOG_COLORS") {
        match colors.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => set_colors_enabled(true),
            "false" | "0" | "no" | "off" => set_colors_enabled(false),
            _ => set_colors_enabled(true),
        }
    } else {
        // Default to colors enabled
        set_colors_enabled(true);
    }
}

// Re-export the LogLevel for external use
pub use LogLevel as Level;
