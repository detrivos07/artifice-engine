// src/logging/src/lib.rs
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Mutex, OnceLock};

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

    /// Returns the log level as a colored string
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

// Global log file
static LOG_FILE: OnceLock<Mutex<Option<File>>> = OnceLock::new();

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

/// Initialize a log file
pub fn init_log_file<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .append(true)
        .open(path)?;

    LOG_FILE.get_or_init(|| Mutex::new(Some(file)));
    Ok(())
}

/// Close the log file if it's open
pub fn close_log_file() -> io::Result<()> {
    if let Some(log_file) = LOG_FILE.get() {
        let mut file_guard = log_file.lock().unwrap();
        *file_guard = None;
    }
    Ok(())
}

/// Internal log function used by macros
pub fn _log_internal(level: LogLevel, message: &str) {
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

    // Format the log message
    let now = chrono::offset::Local::now();
    let level_str = if are_colors_enabled() {
        level.as_colored_str()
    } else {
        level.as_str().to_string()
    };

    let formatted_message = format!(
        "[{}][{}]: {}\n",
        level_str,
        now.format("%Y-%m-%d %H:%M:%S%.3f"),
        message
    );

    // Output to console
    if are_colors_enabled() {
        print!("{}", formatted_message);
    } else {
        // Remove color codes for non-color output
        print!(
            "[{}][{}]: {}\n",
            level.as_str(),
            now.format("%Y-%m-%d %H:%M:%S%.3f"),
            message
        );
    }

    // Output to file if enabled
    if let Some(log_file) = LOG_FILE.get() {
        if let Ok(mut file_guard) = log_file.lock() {
            if let Some(file) = file_guard.as_mut() {
                // Write without color codes to file
                let file_message = format!(
                    "[{}][{}]: {}\n",
                    level.as_str(),
                    now.format("%Y-%m-%d %H:%M:%S%.3f"),
                    message
                );
                let _ = file.write_all(file_message.as_bytes());
                let _ = file.flush();
            }
        }
    }
}

/// Function to initialize the logger - called manually rather than via a feature
pub fn init() {
    // Initialize the log file cell
    LOG_FILE.get_or_init(|| Mutex::new(None));

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

    // Check for log file path in environment
    if let Ok(log_path) = std::env::var("ARTIFICE_LOG_FILE") {
        if !log_path.is_empty() {
            let _ = init_log_file(log_path);
        }
    }
}

// Logging macros
#[macro_export]
macro_rules! fatal {
    ($($arg:tt)*) => ($crate::_log_internal($crate::LogLevel::FATAL, &format!($($arg)*)));
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => ($crate::_log_internal($crate::LogLevel::ERROR, &format!($($arg)*)));
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => ($crate::_log_internal($crate::LogLevel::WARN, &format!($($arg)*)));
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => ($crate::_log_internal($crate::LogLevel::INFO, &format!($($arg)*)));
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => ($crate::_log_internal($crate::LogLevel::DEBUG, &format!($($arg)*)));
}

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => ($crate::_log_internal($crate::LogLevel::TRACE, &format!($($arg)*)));
}

// Re-export the LogLevel for external use
pub use LogLevel as Level;
