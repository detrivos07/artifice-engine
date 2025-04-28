//! ArtificeCore Logging Library
//!
//! This library provides logging functionality for Artifice-Engine applications.

/// An enumeration representing different log levels.
enum LogLevel {
    FATAL,
    ERROR,
    WARN,
    INFO,
    DEBUG,
    TRACE,
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
    let now = chrono::offset::Local::now();
    match level {
        LogLevel::FATAL => println!("[FATAL]:[{}]: {}", now, message),
        LogLevel::ERROR => println!("[ERROR]:[{}]: {}", now, message),
        LogLevel::WARN => println!("[WARN]:[{}]: {}", now, message),
        LogLevel::INFO => println!("[INFO]:[{}]: {}", now, message),
        LogLevel::DEBUG => println!("[DEBUG]:[{}]: {}", now, message),
        LogLevel::TRACE => println!("[TRACE]:[{}]: {}", now, message),
    }
}
