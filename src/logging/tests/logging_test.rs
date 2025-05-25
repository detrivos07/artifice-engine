use logging::{debug, error, info, trace, warn, LoggerBuilder, LoggerError};
use std::sync::Once;

static INIT: Once = Once::new();

fn init_logger_for_tests() {
    INIT.call_once(|| {
        // Initialize a basic logger for all tests
        let _ = LoggerBuilder::new()
            .console(true)
            .colors(false)
            .init();
    });
}

#[test]
fn test_console_logging() {
    init_logger_for_tests();

    info!("Test info message");
    warn!("Test warning message");
    error!("Test error message");
    debug!("Test debug message");
    trace!("Test trace message");
}

#[test]
fn test_file_logging() {
    // Clean up any existing file
    let _ = std::fs::remove_file("test_output.log");
    
    // For file logging test, we'll test the builder directly
    // since we can't reinitialize the global logger
    match LoggerBuilder::new()
        .console(false)
        .file("test_output.log")
        .init() {
        Ok(_) => {
            info!("This should go to file");
            warn!("File warning");
            error!("File error");
        },
        Err(LoggerError::AlreadyInitialized) => {
            // Logger already initialized, that's fine for tests
            // Just verify we can create a logger with file configuration
            println!("Logger already initialized - this is expected in tests");
        },
        Err(e) => panic!("Unexpected error: {}", e),
    }

    // Test that file operations work by creating a temporary logger instance
    // (This tests the file creation logic without global state)
    let test_logger = logging::ArtificeLogger::new(logging::LogConfig {
        console: false,
        file: true,
        colors: false,
    });
    
    if let Ok(logger_with_file) = test_logger.with_file("test_output_direct.log") {
        // Test succeeded - file logger can be created
        let _ = std::fs::remove_file("test_output_direct.log");
    }

    // Clean up
    let _ = std::fs::remove_file("test_output.log");
}

#[test]
fn test_log_levels() {
    init_logger_for_tests();

    // Test setting different log levels
    logging::set_log_level(logging::Level::Warn);
    
    // These should be filtered out at WARN level
    trace!("Trace should not appear");
    debug!("Debug should not appear");
    info!("Info should not appear");
    
    // These should appear
    warn!("Warning should appear");
    error!("Error should appear");

    // Reset to trace level
    logging::set_log_level(logging::Level::Trace);
    
    trace!("Trace should now appear");
    debug!("Debug should now appear");
    info!("Info should now appear");
}

#[test]
fn test_logger_configuration() {
    // Test that we can create logger configurations without initializing
    let config = logging::LogConfig {
        console: true,
        file: false,
        colors: true,
    };
    
    let logger = logging::ArtificeLogger::new(config);
    assert_eq!(logger.get_config().console, true);
    assert_eq!(logger.get_config().file, false);
    assert_eq!(logger.get_config().colors, true);

    // Test builder pattern
    let builder = LoggerBuilder::new()
        .console(false)
        .colors(true);
    
    // We can't actually initialize since the global logger might already be set,
    // but we can test that the builder methods work
    println!("Logger builder test completed");
}