//! # Artifice Logging - High-Performance Rust Logging Library
//!
//! A high-performance logging library optimized for both standard and high-throughput applications.
//! Features include batching, file rotation, colored output, and advanced memory optimization techniques.
//!
//! ## Quick Start
//!
//! ```rust
//! use artifice_logging::init;
//! use log::{info, warn, error};
//!
//! // Initialize with default settings
//! init().expect("Failed to initialize logger");
//!
//! // Log messages
//! info!("Application started");
//! warn!("This is a warning");
//! error!("This is an error");
//! ```
//!
//! ## Advanced Usage
//!
//! ```rust
//! use artifice_logging::LoggerBuilder;
//!
//! LoggerBuilder::new()
//!     .console(true)
//!     .file("app.log")
//!     .colors(true)
//!     .batch_size(100)
//!     .init()
//!     .expect("Failed to initialize logger");
//! ```

use log::{Log, Metadata, Record};
use std::fs::File;
use std::sync::mpsc;
use std::sync::Mutex;
use std::thread;
use std::time::Instant;

// Module declarations
pub mod config;
pub mod batching;
pub mod writers;
pub mod benchmarks;

// Re-export public types
pub use config::{LogConfig, BatchConfig, HighPerformanceConfig, LogLevel, LoggerError};
pub use benchmarks::{LoggingBenchmarks, ThroughputMeter};

// Re-export log macros for convenience
pub use log::{trace, debug, info, warn, error};

use batching::{LogMessage, LogCommand, AdvancedLogCommand};
use writers::{file_worker_thread, high_performance_worker_thread, FileWriter, HighPerformanceFileWriter};

/// Main logger implementation supporting both standard and high-performance modes
pub struct ArtificeLogger {
    config: LogConfig,
    batch_config: BatchConfig,
    file_sender: Option<mpsc::Sender<LogCommand>>,
    _file_thread: Option<thread::JoinHandle<()>>,
}

impl ArtificeLogger {
    /// Create a new logger with default configuration
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
            batch_config: BatchConfig::default(),
            file_sender: None,
            _file_thread: None,
        }
    }

    /// Enable file logging with the specified path
    pub fn with_file<P: AsRef<std::path::Path>>(mut self, path: P) -> Result<Self, LoggerError> {
        let file = File::create(path)?;
        let file_writer = FileWriter::new(file, self.batch_config.clone());
        
        let (sender, receiver) = mpsc::channel();
        let thread_handle = thread::spawn(move || {
            file_worker_thread(file_writer, receiver);
        });
        
        self.file_sender = Some(sender);
        self._file_thread = Some(thread_handle);
        self.config.file = true;
        
        Ok(self)
    }

    /// Set batch configuration
    pub fn with_batch_config(mut self, config: BatchConfig) -> Self {
        self.batch_config = config;
        self
    }

    /// Update logger configuration
    pub fn set_config(&mut self, config: LogConfig) {
        self.config = config;
    }

    /// Get current logger configuration
    pub fn get_config(&self) -> &LogConfig {
        &self.config
    }

    /// Get current batch configuration
    pub fn get_batch_config(&self) -> &BatchConfig {
        &self.batch_config
    }

    fn format_message(&self, record: &Record) -> String {
        let level_str = if self.config.colors {
            LogLevel::from(record.level()).as_colored_str()
        } else {
            LogLevel::from(record.level()).as_str()
        };

        format!("[{}] {}: {}", 
                chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                level_str,
                record.args())
    }

    /// Force flush all pending log messages
    pub fn flush(&self) {
        if let Some(sender) = &self.file_sender {
            let _ = sender.send(LogCommand::Flush);
        }
    }
}

impl Log for ArtificeLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let formatted = self.format_message(record);

        if self.config.console {
            println!("{}", formatted);
        }

        if self.config.file {
            if let Some(sender) = &self.file_sender {
                let message = LogMessage {
                    formatted_message: formatted,
                    timestamp: Instant::now(),
                };
                let _ = sender.send(LogCommand::Message(message));
            }
        }
    }

    fn flush(&self) {
        self.flush();
    }
}

impl Drop for ArtificeLogger {
    fn drop(&mut self) {
        if let Some(sender) = &self.file_sender {
            let _ = sender.send(LogCommand::Shutdown);
        }
    }
}

// Global logger instance
static LOGGER: Mutex<Option<ArtificeLogger>> = Mutex::new(None);

/// Initialize logger with custom configuration
pub fn init_with_config(config: LogConfig) -> Result<(), LoggerError> {
    let mut logger = ArtificeLogger::new();
    logger.set_config(config);
    log::set_logger(Box::leak(Box::new(logger)))?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(())
}

/// Initialize logger with default settings
pub fn init() -> Result<(), LoggerError> {
    init_with_config(LogConfig::default())
}

/// Initialize logger with file output
pub fn init_with_file<P: AsRef<std::path::Path>>(
    path: P, 
    _config: LogConfig
) -> Result<(), LoggerError> {
    let logger = ArtificeLogger::new()
        .with_file(path)?;
    
    log::set_logger(Box::leak(Box::new(logger)))?;
    log::set_max_level(log::LevelFilter::Trace);
    Ok(())
}

/// Initialize logger with file output and custom batching
pub fn init_with_file_and_batching<P: AsRef<std::path::Path>>(
    path: P,
    _config: LogConfig,
    batch_config: BatchConfig,
) -> Result<(), LoggerError> {
    let logger = ArtificeLogger::new()
        .with_batch_config(batch_config)
        .with_file(path)?;
    
    log::set_logger(Box::leak(Box::new(logger)))?;
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

/// Flush all pending log messages
pub fn flush() {
    if let Ok(logger_guard) = LOGGER.lock() {
        if let Some(logger) = logger_guard.as_ref() {
            logger.flush();
        }
    }
}

/// Initialize logger from environment variables
pub fn init_from_env() -> Result<(), LoggerError> {
    let mut config = LogConfig::default();
    let mut batch_config = BatchConfig::default();
    let mut file_path: Option<String> = None;

    // Parse environment variables
    if let Ok(console) = std::env::var("ARTIFICE_LOG_CONSOLE") {
        config.console = console.parse().unwrap_or(true);
    }

    if let Ok(colors) = std::env::var("ARTIFICE_LOG_COLORS") {
        config.colors = colors.parse().unwrap_or(true);
    }

    if let Ok(path) = std::env::var("ARTIFICE_LOG_FILE") {
        file_path = Some(path);
        config.file = true;
    }

    if let Ok(batch_size) = std::env::var("ARTIFICE_LOG_BATCH_SIZE") {
        if let Ok(size) = batch_size.parse() {
            batch_config.batch_size = size;
        }
    }

    if let Ok(flush_interval) = std::env::var("ARTIFICE_LOG_FLUSH_INTERVAL") {
        if let Ok(interval) = flush_interval.parse() {
            batch_config.flush_interval_ms = interval;
        }
    }

    if let Ok(batching) = std::env::var("ARTIFICE_LOG_BATCHING") {
        batch_config.enabled = batching.parse().unwrap_or(true);
    }

    // Initialize logger
    match file_path {
        Some(path) => init_with_file_and_batching(path, config, batch_config),
        None => init_with_config(config),
    }
}

/// Builder pattern for logger configuration
pub struct LoggerBuilder {
    config: LogConfig,
    batch_config: BatchConfig,
    file_path: Option<String>,
}

impl LoggerBuilder {
    /// Create a new logger builder
    pub fn new() -> Self {
        Self {
            config: LogConfig::default(),
            batch_config: BatchConfig::default(),
            file_path: None,
        }
    }

    /// Enable/disable console output
    pub fn console(mut self, enabled: bool) -> Self {
        self.config.console = enabled;
        self
    }

    /// Set file output path
    pub fn file<P: AsRef<str>>(mut self, path: P) -> Self {
        self.file_path = Some(path.as_ref().to_string());
        self.config.file = true;
        self
    }

    /// Enable/disable colored output
    pub fn colors(mut self, enabled: bool) -> Self {
        self.config.colors = enabled;
        self
    }

    /// Set batch size
    pub fn batch_size(mut self, size: usize) -> Self {
        self.batch_config.batch_size = size;
        self
    }

    /// Set flush interval in milliseconds
    pub fn flush_interval_ms(mut self, interval: u64) -> Self {
        self.batch_config.flush_interval_ms = interval;
        self
    }

    /// Enable/disable batching
    pub fn batching(mut self, enabled: bool) -> Self {
        self.batch_config.enabled = enabled;
        self
    }

    /// Set custom batch configuration
    pub fn batch_config(mut self, config: BatchConfig) -> Self {
        self.batch_config = config;
        self
    }

    /// Initialize the logger with the configured settings
    pub fn init(self) -> Result<(), LoggerError> {
        match self.file_path {
            Some(path) => init_with_file_and_batching(path, self.config, self.batch_config),
            None => init_with_config(self.config),
        }
    }
}

impl Default for LoggerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// High-performance logger for maximum throughput scenarios
pub struct HighPerformanceLogger {
    sender: mpsc::Sender<AdvancedLogCommand>,
    _thread: thread::JoinHandle<()>,
}

impl HighPerformanceLogger {
    /// Create a new high-performance logger
    pub fn new<P: AsRef<std::path::Path>>(
        path: P, 
        config: HighPerformanceConfig
    ) -> Result<Self, LoggerError> {
        let file = File::create(path)?;
        let writer = HighPerformanceFileWriter::new(file, config);
        
        let (sender, receiver) = mpsc::channel();
        let thread_handle = thread::spawn(move || {
            high_performance_worker_thread(writer, receiver);
        });
        
        Ok(Self {
            sender,
            _thread: thread_handle,
        })
    }
    
    /// Log a pre-formatted message for maximum performance
    pub fn log_fast(&self, message: String) -> Result<(), LoggerError> {
        self.sender.send(AdvancedLogCommand::Message(message))
            .map_err(|_| LoggerError::ChannelError)
    }
    
    /// Force flush all pending messages
    pub fn flush(&self) -> Result<(), LoggerError> {
        self.sender.send(AdvancedLogCommand::Flush)
            .map_err(|_| LoggerError::ChannelError)
    }
}

impl Drop for HighPerformanceLogger {
    fn drop(&mut self) {
        let _ = self.sender.send(AdvancedLogCommand::Shutdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::writers::HighPerformanceFileWriter;
    use log::Log;
    use std::fs;
    use std::io::Read;
    use std::sync::{Arc, Barrier, Once};
    use std::thread;
    use std::time::{Duration, Instant};


    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            let _ = init();
        });
    }

    #[test]
    fn test_log_levels() {
        setup();
        assert_eq!(LogLevel::Error.as_str(), "ERROR");
        assert_eq!(LogLevel::Info.as_str(), "INFO");
    }

    #[test]
    fn test_batch_config_default() {
        let config = BatchConfig::default();
        assert_eq!(config.batch_size, 50);
        assert!(config.enabled);
    }

    #[test]
    fn test_log_config_default() {
        let config = LogConfig::default();
        assert!(config.console);
        assert!(!config.file);
    }

    #[test]
    fn test_logger_builder() {
        let builder = LoggerBuilder::new()
            .console(false)
            .colors(false)
            .batch_size(100);
        
        assert!(!builder.config.console);
        assert!(!builder.config.colors);
        assert_eq!(builder.batch_config.batch_size, 100);
    }

    #[test]
    fn test_level_conversions() {
        let log_level = LogLevel::Info;
        let log_crate_level: log::Level = log_level.into();
        assert_eq!(log_crate_level, log::Level::Info);
    }

    // Integration Tests
    #[test]
    fn test_basic_console_logging() {
        let _config = LogConfig {
            console: true,
            file: false,
            colors: false,
        };
        
        let mut logger = ArtificeLogger::new();
        logger.set_config(_config);
        assert_eq!(logger.get_config().console, true);
    }

    #[test]
    fn test_file_logging() {
        let temp_dir = std::env::temp_dir();
        let log_file = temp_dir.join("test_file_logging.log");
        
        // Clean up any existing file
        let _ = fs::remove_file(&log_file);
        
        let _config = LogConfig {
            console: false,
            file: true,
            colors: false,
        };
        
        let batch_config = BatchConfig {
            batch_size: 1, // Immediate write for testing
            flush_interval_ms: 10,
            enabled: false, // Disable batching for immediate writes
            buffer_capacity: 100,
            string_pool_size: 50,
        };
        
        // Create logger with file output
        let logger = ArtificeLogger::new()
            .with_batch_config(batch_config)
            .with_file(&log_file)
            .unwrap();
        
        // Simulate logging
        let record = log::Record::builder()
            .args(format_args!("Test message"))
            .level(log::Level::Info)
            .target("test")
            .module_path(Some("test_module"))
            .file(Some("test.rs"))
            .line(Some(42))
            .build();
        
        logger.log(&record);
        logger.flush();
        
        // Give some time for the background thread to write
        thread::sleep(Duration::from_millis(100));
        
        // Verify file was created and contains the message
        assert!(log_file.exists());
        
        let mut content = String::new();
        fs::File::open(&log_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        
        assert!(content.contains("Test message"));
        assert!(content.contains("INFO"));
        
        // Cleanup
        let _ = fs::remove_file(&log_file);
    }

    #[test]
    fn test_batch_processing() {
        let temp_dir = std::env::temp_dir();
        let log_file = temp_dir.join("test_batch_processing.log");
        
        // Clean up any existing file
        let _ = fs::remove_file(&log_file);
        
        let _config = LogConfig {
            console: false,
            file: true,
            colors: false,
        };
        
        let batch_config = BatchConfig {
            batch_size: 5,
            flush_interval_ms: 1000, // Long interval to test batch size trigger
            enabled: true,
            buffer_capacity: 100,
            string_pool_size: 50,
        };
        
        let logger = ArtificeLogger::new()
            .with_batch_config(batch_config)
            .with_file(&log_file)
            .unwrap();
        
        // Send multiple messages to trigger batch write
        for i in 0..10 {
            logger.log(&log::Record::builder()
                .args(format_args!("Batch message {}", i))
                .level(log::Level::Info)
                .target("test")
                .build());
        }
        
        // Force flush to ensure all messages are written
        logger.flush();
        thread::sleep(Duration::from_millis(200));
        
        // Verify all messages were written
        let mut content = String::new();
        fs::File::open(&log_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        
        for i in 0..10 {
            assert!(content.contains(&format!("Batch message {}", i)));
        }
        
        // Cleanup
        let _ = fs::remove_file(&log_file);
    }

    #[test]
    fn test_concurrent_logging() {
        let temp_dir = std::env::temp_dir();
        let log_file = temp_dir.join("test_concurrent.log");
        
        // Clean up any existing file
        let _ = fs::remove_file(&log_file);
        
        let _config = LogConfig {
            console: false,
            file: true,
            colors: false,
        };
        
        let batch_config = BatchConfig {
            batch_size: 10,
            flush_interval_ms: 50,
            enabled: true,
            buffer_capacity: 200,
            string_pool_size: 100,
        };
        
        let logger: Arc<ArtificeLogger> = Arc::new(
            ArtificeLogger::new()
                .with_batch_config(batch_config)
                .with_file(&log_file)
                .unwrap()
        );
        
        // Create multiple threads that log concurrently
        let thread_count = 4;
        let messages_per_thread = 25;
        let barrier = Arc::new(Barrier::new(thread_count));
        
        let mut handles = vec![];
        
        for thread_id in 0..thread_count {
            let logger_clone = Arc::clone(&logger);
            let barrier_clone = Arc::clone(&barrier);
            
            let handle = thread::spawn(move || {
                barrier_clone.wait(); // Synchronize start
                
                for i in 0..messages_per_thread {
                    logger_clone.log(&log::Record::builder()
                        .args(format_args!("Thread {} message {}", thread_id, i))
                        .level(log::Level::Info)
                        .target("concurrent_test")
                        .build());
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // Force flush and wait
        logger.flush();
        thread::sleep(Duration::from_millis(300));
        
        // Verify all messages were written
        let mut content = String::new();
        fs::File::open(&log_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        
        // Check that we have messages from all threads
        for thread_id in 0..thread_count {
            for i in 0..messages_per_thread {
                assert!(content.contains(&format!("Thread {} message {}", thread_id, i)));
            }
        }
        
        // Cleanup
        let _ = fs::remove_file(&log_file);
    }

    #[test]
    fn test_performance_batching() {
        let temp_dir = std::env::temp_dir();
        let log_file = temp_dir.join("test_performance.log");
        
        // Clean up any existing file
        let _ = fs::remove_file(&log_file);
        
        let _config = LogConfig {
            console: false,
            file: true,
            colors: false,
        };
        
        let batch_config = BatchConfig {
            batch_size: 100,
            flush_interval_ms: 50,
            enabled: true,
            buffer_capacity: 1000,
            string_pool_size: 500,
        };
        
        let logger = ArtificeLogger::new()
            .with_batch_config(batch_config)
            .with_file(&log_file)
            .unwrap();
        
        let message_count = 1000;
        let start = Instant::now();
        
        // Log many messages quickly
        for i in 0..message_count {
            logger.log(&log::Record::builder()
                .args(format_args!("Performance test message {} with some additional data", i))
                .level(log::Level::Info)
                .target("perf_test")
                .build());
        }
        
        logger.flush();
        let duration = start.elapsed();
        
        // Wait for background thread to complete
        thread::sleep(Duration::from_millis(200));
        
        // Verify performance (should be fast with batching)
        println!("Logged {} messages in {:?}", message_count, duration);
        assert!(duration < Duration::from_millis(1000)); // Should be much faster
        
        // Verify all messages were written
        let mut content = String::new();
        fs::File::open(&log_file)
            .unwrap()
            .read_to_string(&mut content)
            .unwrap();
        
        let line_count = content.lines().count();
        assert_eq!(line_count, message_count);
        
        // Cleanup
        let _ = fs::remove_file(&log_file);
    }

    #[test]
    fn test_level_filtering() {
        let logger = ArtificeLogger::new();
        
        // Test that logger is enabled for all levels by default
        let metadata = log::Metadata::builder()
            .level(log::Level::Trace)
            .target("test")
            .build();
        assert!(logger.enabled(&metadata));
        
        let metadata = log::Metadata::builder()
            .level(log::Level::Error)
            .target("test")
            .build();
        assert!(logger.enabled(&metadata));
    }

    #[test]
    fn test_error_handling() {
        // Test creating logger with invalid file path
        let invalid_path = "/root/invalid/path/test.log";
        let batch_config = BatchConfig::default();
        
        let result = ArtificeLogger::new()
            .with_batch_config(batch_config)
            .with_file(invalid_path);
        assert!(result.is_err());
    }

    // Benchmark Tests
    #[test]
    fn bench_batch_sizes() {
        println!("=== Batch Size Performance Benchmarks ===");
        
        let batch_sizes = vec![1, 10, 50, 100];
        let message_count = 1000;
        let test_message = "This is a test log message for benchmarking";
        
        for &batch_size in &batch_sizes {
            let temp_dir = std::env::temp_dir();
            let log_file = temp_dir.join(format!("bench_batch_{}.log", batch_size));
            let _ = fs::remove_file(&log_file);
            
            let _config = LogConfig {
                console: false,
                file: true,
                colors: false,
            };
            
            let batch_config = BatchConfig {
                batch_size,
                flush_interval_ms: 10000, // Large interval to test batch size only
                enabled: true,
                buffer_capacity: batch_size * 2,
                string_pool_size: batch_size,
            };
            
            let logger = ArtificeLogger::new()
                .with_batch_config(batch_config)
                .with_file(&log_file)
                .unwrap();
            
            let start = Instant::now();
            
            for i in 0..message_count {
                logger.log(&log::Record::builder()
                    .args(format_args!("{} - message {}", test_message, i))
                    .level(log::Level::Info)
                    .target("bench")
                    .build());
            }
            
            logger.flush();
            thread::sleep(Duration::from_millis(100));
            
            let duration = start.elapsed();
            let throughput = message_count as f64 / duration.as_secs_f64();
            
            println!("Batch size {}: {:.2} messages/sec ({:?} total)", 
                     batch_size, throughput, duration);
            
            let _ = fs::remove_file(&log_file);
        }
    }

    #[test]
    fn bench_memory_patterns() {
        println!("=== Memory Pattern Benchmarks ===");
        
        let message_count = 5000;
        let test_message = "Benchmark message for memory pattern testing";
        
        // Test Structure of Arrays (SoA) - DoD approach
        let start = Instant::now();
        let mut messages = Vec::with_capacity(message_count);
        let mut timestamps = Vec::with_capacity(message_count);
        
        // Allocation phase
        for i in 0..message_count {
            messages.push(format!("{} {}", test_message, i));
            timestamps.push(Instant::now());
        }
        
        // Access phase - sequential, cache-friendly
        let _total_length: usize = messages.iter().map(|msg| msg.len()).sum();
        
        let soa_duration = start.elapsed();
        
        // Test Array of Structures (AoS) - traditional approach
        #[derive(Clone)]
        struct TraditionalMessage {
            content: String,
            timestamp: Instant,
        }
        
        let start = Instant::now();
        let mut traditional_messages = Vec::with_capacity(message_count);
        
        // Allocation phase
        for i in 0..message_count {
            traditional_messages.push(TraditionalMessage {
                content: format!("{} {}", test_message, i),
                timestamp: Instant::now(),
            });
        }
        
        // Access phase - less cache-friendly due to interleaved data
        let mut _total_length = 0usize;
        let mut _timestamp_count = 0usize;
        for msg in &traditional_messages {
            _total_length += msg.content.len();
            _timestamp_count += msg.timestamp.elapsed().as_nanos() as usize;
        }
        
        let aos_duration = start.elapsed();
        
        println!("SoA (DoD) approach: {:?}", soa_duration);
        println!("AoS (traditional) approach: {:?}", aos_duration);
        if aos_duration.as_nanos() > 0 && soa_duration.as_nanos() > 0 {
            println!("DoD is {:.2}x faster", aos_duration.as_nanos() as f64 / soa_duration.as_nanos() as f64);
        }
        
        // Prevent optimization
        assert!(_total_length > 0);
        assert!(_timestamp_count > 0);
    }

    #[test]
    fn bench_string_pool() {
        println!("=== String Pool Efficiency Benchmarks ===");
        
        let iterations = 500;
        let pool_sizes = vec![0, 50, 100];
        
        for &pool_size in &pool_sizes {
            let temp_dir = std::env::temp_dir();
            let log_file = temp_dir.join(format!("bench_pool_{}.log", pool_size));
            let _ = fs::remove_file(&log_file);
            
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_file)
                .unwrap();
            
            let config = HighPerformanceConfig {
                batch_size: 100,
                flush_interval_ms: 1000,
                enabled: true,
                buffer_capacity: 200,
                string_pool_size: pool_size,
            };
            
            let mut writer = HighPerformanceFileWriter::new(file, config);
            
            let start = Instant::now();
            
            for i in 0..iterations {
                let message = format!("String pool test message number {}", i);
                writer.add_message(message).unwrap();
            }
            
            writer.flush().unwrap();
            let duration = start.elapsed();
            
            println!("Pool size {}: {:?} ({:.2} μs per message)", 
                     pool_size, duration, duration.as_micros() as f64 / iterations as f64);
            
            let _ = fs::remove_file(&log_file);
        }
    }

    #[test]
    fn bench_cache_locality() {
        println!("=== Cache Locality Benchmarks ===");
        
        let data_size = 5000;
        
        // Structure of Arrays (better cache locality)
        let start = Instant::now();
        let mut strings = Vec::with_capacity(data_size);
        let mut numbers = Vec::with_capacity(data_size);
        let mut booleans = Vec::with_capacity(data_size);
        
        // Fill arrays
        for i in 0..data_size {
            strings.push(format!("Item {}", i));
            numbers.push(i as u64);
            booleans.push(i % 2 == 0);
        }
        
        // Process each array separately (cache-friendly)
        let mut _string_total = 0;
        let mut _number_total = 0;
        let mut _bool_count = 0;
        
        for s in &strings {
            _string_total += s.len();
        }
            
        for &n in &numbers {
            _number_total += n;
        }
            
        for &b in &booleans {
            if b { _bool_count += 1; }
        }
        
        let soa_time = start.elapsed();
        
        // Array of Structures (worse cache locality)
        #[derive(Clone)]
        struct Item {
            string: String,
            number: u64,
            boolean: bool,
        }
        
        let start = Instant::now();
        let mut items = Vec::with_capacity(data_size);
        
        // Fill array
        for i in 0..data_size {
            items.push(Item {
                string: format!("Item {}", i),
                number: i as u64,
                boolean: i % 2 == 0,
            });
        }
        
        // Process mixed data (cache-unfriendly)
        let mut _string_total = 0;
        let mut _number_total = 0u64;
        let mut _bool_count = 0;
        
        for item in &items {
            _string_total += item.string.len();
            _number_total += item.number;
            if item.boolean { _bool_count += 1; }
        }
        
        let aos_time = start.elapsed();
        
        println!("SoA (cache-friendly): {:?}", soa_time);
        println!("AoS (cache-unfriendly): {:?}", aos_time);
        if aos_time.as_nanos() > 0 && soa_time.as_nanos() > 0 {
            println!("SoA is {:.2}x faster due to better cache locality", 
                     aos_time.as_nanos() as f64 / soa_time.as_nanos() as f64);
            }
        }

        #[test]
        fn test_disabled_batching() {
            let temp_dir = std::env::temp_dir();
            let log_file = temp_dir.join("test_disabled_batching.log");
        
            // Clean up any existing file
            let _ = fs::remove_file(&log_file);
        
            let _config = LogConfig {
                console: false,
                file: true,
                colors: false,
            };
        
            let batch_config = BatchConfig {
                batch_size: 100,
                flush_interval_ms: 1000,
                enabled: false, // Disable batching
                buffer_capacity: 200,
                string_pool_size: 100,
            };
        
            let logger = ArtificeLogger::new()
                .with_batch_config(batch_config)
                .with_file(&log_file)
                .unwrap();
        
            // Send messages
            for i in 0..3 {
                logger.log(&log::Record::builder()
                    .args(format_args!("No batch message {}", i))
                    .level(log::Level::Info)
                    .target("no_batch_test")
                    .build());
            }
        
            // With batching disabled, messages should be written immediately
            thread::sleep(Duration::from_millis(50));
        
            let mut content = String::new();
            fs::File::open(&log_file)
                .unwrap()
                .read_to_string(&mut content)
                .unwrap();
        
            for i in 0..3 {
                assert!(content.contains(&format!("No batch message {}", i)));
            }
        
            // Cleanup
            let _ = fs::remove_file(&log_file);
        }
    }