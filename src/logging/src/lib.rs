use log::{Log, Metadata, Record};
use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Mutex, OnceLock};

/// Custom error type for logger initialization
#[derive(Debug)]
pub enum LoggerError {
    Io(io::Error),
    SetLogger(log::SetLoggerError),
    AlreadyInitialized,
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
            LoggerError::SetLogger(err) => write!(f, "Logger setup error: {}", err),
            LoggerError::AlreadyInitialized => write!(f, "Logger already initialized"),
        }
    }
}

impl std::error::Error for LoggerError {}

/// Configuration for logging output destinations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LogConfig {
    pub console: bool,
    pub file: bool,
    pub colors: bool,
}

impl Default for LogConfig {
    fn default() -> Self {
        LogConfig {
            console: true,
            file: false,
            colors: true,
        }
    }
}

/// An enumeration representing different log levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}

impl LogLevel {
    /// Returns the log level as a string.
    fn as_str(&self) -> &str {
        match self {
            LogLevel::Error => "ERROR",
            LogLevel::Warn => "WARN",
            LogLevel::Info => "INFO",
            LogLevel::Debug => "DEBUG",
            LogLevel::Trace => "TRACE",
        }
    }

    /// Returns the log level as a colored string
    fn as_colored_str(&self) -> String {
        match self {
            LogLevel::Error => format!("\x1b[31m{}\x1b[0m", self.as_str()),   // Red
            LogLevel::Warn => format!("\x1b[33m{}\x1b[0m", self.as_str()),    // Yellow
            LogLevel::Info => format!("\x1b[32m{}\x1b[0m", self.as_str()),    // Green
            LogLevel::Debug => format!("\x1b[36m{}\x1b[0m", self.as_str()),   // Cyan
            LogLevel::Trace => format!("\x1b[35m{}\x1b[0m", self.as_str()),   // Magenta
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

/// The main logger implementation
pub struct ArtificeLogger {
    config: LogConfig,
    log_file: Mutex<Option<File>>,
}

impl ArtificeLogger {
    pub fn new(config: LogConfig) -> Self {
        ArtificeLogger {
            config,
            log_file: Mutex::new(None),
        }
    }

    pub fn with_file<P: AsRef<Path>>(mut self, path: P) -> Result<Self, LoggerError> {
        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(path)?;
        
        *self.log_file.lock().unwrap() = Some(file);
        self.config.file = true;
        Ok(self)
    }

    pub fn set_config(&mut self, config: LogConfig) {
        self.config = config;
    }

    pub fn get_config(&self) -> LogConfig {
        self.config
    }

    fn format_message(&self, record: &Record, use_colors: bool) -> String {
        let now = chrono::offset::Local::now();
        let level = LogLevel::from(record.level());
        
        let level_str = if use_colors {
            level.as_colored_str()
        } else {
            level.as_str().to_string()
        };

        let target = if record.target().is_empty() {
            record.module_path().unwrap_or("unknown")
        } else {
            record.target()
        };

        format!(
            "[{}][{}][{}]: {}",
            level_str,
            now.format("%Y-%m-%d %H:%M:%S%.3f"),
            target,
            record.args()
        )
    }
}

impl Log for ArtificeLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Output to console
        if self.config.console {
            let message = self.format_message(record, self.config.colors);
            println!("{}", message);
        }

        // Output to file
        if self.config.file {
            if let Ok(mut file_guard) = self.log_file.lock() {
                if let Some(file) = file_guard.as_mut() {
                    let message = self.format_message(record, false); // No colors in file
                    let _ = writeln!(file, "{}", message);
                    let _ = file.flush();
                }
            }
        }
    }

    fn flush(&self) {
        if self.config.file {
            if let Ok(mut file_guard) = self.log_file.lock() {
                if let Some(file) = file_guard.as_mut() {
                    let _ = file.flush();
                }
            }
        }
    }
}

// Global logger instance
static LOGGER: OnceLock<ArtificeLogger> = OnceLock::new();

/// Initialize the logger with the given configuration
pub fn init_with_config(config: LogConfig) -> Result<(), LoggerError> {
    let logger = ArtificeLogger::new(config);
    LOGGER.set(logger).map_err(|_| LoggerError::AlreadyInitialized)?;
    
    log::set_logger(LOGGER.get().unwrap())?;
    log::set_max_level(log::LevelFilter::Trace);
    
    Ok(())
}

/// Initialize the logger with console output only
pub fn init() -> Result<(), LoggerError> {
    init_with_config(LogConfig::default())
}

/// Initialize the logger with both console and file output
pub fn init_with_file<P: AsRef<Path>>(path: P) -> Result<(), LoggerError> {
    let logger = ArtificeLogger::new(LogConfig {
        console: true,
        file: true,
        colors: true,
    }).with_file(path)?;
    
    LOGGER.set(logger).map_err(|_| LoggerError::AlreadyInitialized)?;
    
    log::set_logger(LOGGER.get().unwrap())?;
    log::set_max_level(log::LevelFilter::Trace);
    
    Ok(())
}

/// Set the global log level
pub fn set_log_level(level: LogLevel) {
    log::set_max_level(level.into());
}

/// Get the current global log level
pub fn get_log_level() -> LogLevel {
    log::max_level().into()
}

impl From<log::LevelFilter> for LogLevel {
    fn from(filter: log::LevelFilter) -> Self {
        match filter {
            log::LevelFilter::Off => LogLevel::Error, // Default to Error if logging is off
            log::LevelFilter::Error => LogLevel::Error,
            log::LevelFilter::Warn => LogLevel::Warn,
            log::LevelFilter::Info => LogLevel::Info,
            log::LevelFilter::Debug => LogLevel::Debug,
            log::LevelFilter::Trace => LogLevel::Trace,
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

/// Initialize logger from environment variables
pub fn init_from_env() -> Result<(), LoggerError> {
    let mut config = LogConfig::default();
    
    // Set log level from environment variable if available
    if let Ok(level) = std::env::var("ARTIFICE_LOG_LEVEL") {
        let log_level = match level.to_uppercase().as_str() {
            "ERROR" => LogLevel::Error,
            "WARN" => LogLevel::Warn,
            "INFO" => LogLevel::Info,
            "DEBUG" => LogLevel::Debug,
            "TRACE" => LogLevel::Trace,
            _ => LogLevel::Info,
        };
        set_log_level(log_level);
    }

    // Configure console output
    if let Ok(console) = std::env::var("ARTIFICE_LOG_CONSOLE") {
        config.console = match console.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => true,
        };
    }

    // Enable/disable colors from environment variable if available
    if let Ok(colors) = std::env::var("ARTIFICE_LOG_COLORS") {
        config.colors = match colors.to_lowercase().as_str() {
            "true" | "1" | "yes" | "on" => true,
            "false" | "0" | "no" | "off" => false,
            _ => true,
        };
    }

    // Check for log file path in environment
    if let Ok(log_path) = std::env::var("ARTIFICE_LOG_FILE") {
        if !log_path.is_empty() {
            config.file = true;
            let logger = ArtificeLogger::new(config)
                .with_file(log_path)?;
            
            LOGGER.set(logger).map_err(|_| LoggerError::AlreadyInitialized)?;
            log::set_logger(LOGGER.get().unwrap())?;
            log::set_max_level(log::LevelFilter::Trace);
            return Ok(());
        }
    }

    init_with_config(config)
}

/// Builder for configuring the logger
pub struct LoggerBuilder {
    config: LogConfig,
    file_path: Option<String>,
}

impl LoggerBuilder {
    pub fn new() -> Self {
        LoggerBuilder {
            config: LogConfig::default(),
            file_path: None,
        }
    }

    pub fn console(mut self, enabled: bool) -> Self {
        self.config.console = enabled;
        self
    }

    pub fn file<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.file = true;
        self.file_path = Some(path.as_ref().to_string_lossy().into_owned());
        self
    }

    pub fn colors(mut self, enabled: bool) -> Self {
        self.config.colors = enabled;
        self
    }

    pub fn init(self) -> Result<(), LoggerError> {
        if let Some(path) = self.file_path {
            let logger = ArtificeLogger::new(self.config)
                .with_file(path)?;
            
            LOGGER.set(logger).map_err(|_| LoggerError::AlreadyInitialized)?;
        } else {
            let logger = ArtificeLogger::new(self.config);
            LOGGER.set(logger).map_err(|_| LoggerError::AlreadyInitialized)?;
        }
        
        log::set_logger(LOGGER.get().unwrap())?;
        log::set_max_level(log::LevelFilter::Trace);
        Ok(())
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// Re-export standard log macros for convenience
pub use log::{debug, error, info, trace, warn};

// Re-export the LogLevel for external use
pub use LogLevel as Level;