use std::io;

/// Configuration for console and file logging output
#[derive(Debug, Clone)]
pub struct LogConfig {
    /// Enable console output
    pub console: bool,
    /// Enable file output
    pub file: bool,
    /// Enable colored output
    pub colors: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            console: true,
            file: false,
            colors: true,
        }
    }
}

/// Configuration for standard batching operations
#[derive(Debug, Clone)]
pub struct BatchConfig {
    /// Number of messages to batch before writing
    pub batch_size: usize,
    /// Maximum time to wait before flushing (milliseconds)
    pub flush_interval_ms: u64,
    /// Whether batching is enabled
    pub enabled: bool,
    /// Initial buffer capacity
    pub buffer_capacity: usize,
    /// String pool size for reusing allocations
    pub string_pool_size: usize,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            batch_size: 50,
            flush_interval_ms: 100,
            enabled: true,
            buffer_capacity: 256,
            string_pool_size: 128,
        }
    }
}

/// High-performance batch configuration optimized for maximum throughput
/// 
/// This configuration is designed for applications that require the highest
/// possible logging performance with minimal allocation overhead.
#[derive(Debug, Clone)]
pub struct HighPerformanceConfig {
    /// Large batch sizes for fewer write operations
    pub batch_size: usize,
    /// Longer flush intervals for better batching
    pub flush_interval_ms: u64,
    /// Batching enabled by default for performance
    pub enabled: bool,
    /// Pre-allocated larger buffers to avoid reallocations
    pub buffer_capacity: usize,
    /// Larger string pool for reusing allocations
    pub string_pool_size: usize,
}

impl Default for HighPerformanceConfig {
    fn default() -> Self {
        Self {
            batch_size: 100,
            flush_interval_ms: 100,
            enabled: true,
            buffer_capacity: 1024,
            string_pool_size: 512,
        }
    }
}

/// Log level enumeration with color support
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
    /// Error level logging
    Error,
    /// Warning level logging
    Warn,
    /// Info level logging
    Info,
    /// Debug level logging
    Debug,
    /// Trace level logging
    Trace,
}

impl LogLevel {
    /// Get the string representation of the log level
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }

    /// Get colored string representation for console output
    pub fn as_colored_str(&self) -> &'static str {
        match self {
            LogLevel::Error => "\x1b[31mERROR\x1b[0m",
            LogLevel::Warn => "\x1b[33mWARN\x1b[0m",
            LogLevel::Info => "\x1b[32mINFO\x1b[0m",
            LogLevel::Debug => "\x1b[36mDEBUG\x1b[0m",
            LogLevel::Trace => "\x1b[37mTRACE\x1b[0m",
        }
    }
}

impl From<log::Level> for LogLevel {
    fn from(level: log::Level) -> Self {
        match level {
            log::Level::Error => LogLevel::Error,
            log::Level::Warn => LogLevel::Warn,
            log::Level::Info => LogLevel::Info,
            log::Level::Debug => LogLevel::Debug,
            log::Level::Trace => LogLevel::Trace,
        }
    }
}

impl From<LogLevel> for log::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => log::Level::Error,
            LogLevel::Warn => log::Level::Warn,
            LogLevel::Info => log::Level::Info,
            LogLevel::Debug => log::Level::Debug,
            LogLevel::Trace => log::Level::Trace,
        }
    }
}

impl From<log::LevelFilter> for LogLevel {
    fn from(filter: log::LevelFilter) -> Self {
        match filter {
            log::LevelFilter::Error => LogLevel::Error,
            log::LevelFilter::Warn => LogLevel::Warn,
            log::LevelFilter::Info => LogLevel::Info,
            log::LevelFilter::Debug => LogLevel::Debug,
            log::LevelFilter::Trace => LogLevel::Trace,
            log::LevelFilter::Off => LogLevel::Error, // Default fallback
        }
    }
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

/// Logger error types
#[derive(Debug)]
pub enum LoggerError {
    /// IO error during file operations
    Io(io::Error),
    /// Error setting the global logger
    SetLogger(log::SetLoggerError),
    /// Logger already initialized
    AlreadyInitialized,
    /// Channel communication error
    ChannelError,
}

impl From<io::Error> for LoggerError {
    fn from(err: io::Error) -> Self {
        LoggerError::Io(err)
    }
}

impl From<log::SetLoggerError> for LoggerError {
    fn from(err: log::SetLoggerError) -> Self {
        LoggerError::SetLogger(err)
    }
}

impl std::fmt::Display for LoggerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoggerError::Io(err) => write!(f, "IO error: {}", err),
            LoggerError::SetLogger(err) => write!(f, "Set logger error: {}", err),
            LoggerError::AlreadyInitialized => write!(f, "Logger already initialized"),
            LoggerError::ChannelError => write!(f, "Channel communication error"),
        }
    }
}

impl std::error::Error for LoggerError {}