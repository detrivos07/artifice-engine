//! Basic usage examples for the Artifice Logging library
//! 
//! This example demonstrates the most common ways to use the logging library,
//! including console logging, file logging, and different configuration options.

use artifice_logging::*;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Artifice Logging Library - Basic Usage Examples ===\n");

    // Example 1: Simple console logging
    example_1_simple_console()?;
    
    // Example 2: File logging
    example_2_file_logging()?;
    
    // Example 3: Console + File logging
    example_3_console_and_file()?;
    
    // Example 4: Custom configuration
    example_4_custom_config()?;
    
    // Example 5: Builder pattern
    example_5_builder_pattern()?;
    
    // Example 6: Environment configuration
    example_6_environment_config()?;

    println!("\n=== All examples completed successfully! ===");
    Ok(())
}

/// Example 1: Simple console logging with default settings
fn example_1_simple_console() -> Result<(), LoggerError> {
    println!("--- Example 1: Simple Console Logging ---");
    
    // Initialize with default settings (console only, with colors)
    let _ = init();
    
    // Use standard log macros
    log::info!("This is an info message");
    log::warn!("This is a warning message");
    log::error!("This is an error message");
    log::debug!("This is a debug message");
    log::trace!("This is a trace message");
    
    println!("✓ Console logging example completed\n");
    Ok(())
}

/// Example 2: File logging only
fn example_2_file_logging() -> Result<(), LoggerError> {
    println!("--- Example 2: File Logging ---");
    
    let log_file = "example_file_only.log";
    
    // Initialize with file logging
    let config = LogConfig::default();
    let _ = init_with_file(log_file, config);
    
    log::info!("This message will be written to {}", log_file);
    log::warn!("File logging is useful for production systems");
    log::error!("Errors are captured in the log file");
    
    // Force flush to ensure messages are written
    flush();
    
    // Give background thread time to write
    thread::sleep(Duration::from_millis(100));
    
    println!("✓ Check '{}' for the logged messages", log_file);
    println!("✓ File logging example completed\n");
    Ok(())
}

/// Example 3: Both console and file logging
fn example_3_console_and_file() -> Result<(), LoggerError> {
    println!("--- Example 3: Console + File Logging ---");
    
    let log_file = "example_console_and_file.log";
    
    // Create custom configuration
    let config = LogConfig {
        console: true,  // Enable console output
        file: true,     // Enable file output
        colors: true,   // Enable colored console output
    };
    
    // Initialize with custom config and file
    let _ = init_with_config(config);
    
    log::info!("This appears in both console and file: {}", log_file);
    log::warn!("Dual output is great for development");
    log::error!("You can see errors immediately and review them later");
    
    flush();
    thread::sleep(Duration::from_millis(100));
    
    println!("✓ Console + file logging example completed\n");
    Ok(())
}

/// Example 4: Custom configuration with batch settings
fn example_4_custom_config() -> Result<(), LoggerError> {
    println!("--- Example 4: Custom Configuration ---");
    
    let log_file = "example_custom_config.log";
    
    // Custom batch configuration for high performance
    let batch_config = BatchConfig {
        batch_size: 50,        // Write after 50 messages
        flush_interval_ms: 100, // Or flush every 100ms
        enabled: true,         // Enable batching
        buffer_capacity: 200,  // Pre-allocate buffer for 200 messages
        string_pool_size: 100, // Pool size for string reuse
    };
    
    let config = LogConfig::default();
    let _ = init_with_file_and_batching(log_file, config, batch_config);
    
    // Log many messages quickly to demonstrate batching
    for i in 0..25 {
        log::info!("Batched message {}: High-performance logging in action", i);
    }
    
    log::warn!("Batching improves performance for high-volume logging");
    log::error!("Critical messages are still processed efficiently");
    
    flush();
    thread::sleep(Duration::from_millis(200));
    
    println!("✓ Custom configuration example completed\n");
    Ok(())
}

/// Example 5: Using the builder pattern for complex setups
fn example_5_builder_pattern() -> Result<(), LoggerError> {
    println!("--- Example 5: Builder Pattern ---");
    
    let log_file = "example_builder.log";
    
    // Use builder pattern for fluent configuration
    let _ = LoggerBuilder::new()
        .console(true)              // Enable console
        .file(log_file)            // Set log file
        .colors(false)             // Disable colors for this example
        .batch_size(25)            // Small batch size
        .flush_interval_ms(50)     // Quick flush interval
        .batching(true)            // Enable batching
        .init();                   // Initialize
    
    log::info!("Builder pattern makes configuration clean and readable");
    log::warn!("You can chain multiple configuration calls");
    log::error!("Perfect for complex logging setups");
    
    // Demonstrate performance with quick logging
    let start = std::time::Instant::now();
    for i in 0..100 {
        log::debug!("Performance test message {}", i);
    }
    let duration = start.elapsed();
    
    log::info!("Logged 100 messages in {:?}", duration);
    
    flush();
    thread::sleep(Duration::from_millis(150));
    
    println!("✓ Builder pattern example completed\n");
    Ok(())
}

/// Example 6: Environment-based configuration
fn example_6_environment_config() -> Result<(), LoggerError> {
    println!("--- Example 6: Environment Configuration ---");
    
    // Set environment variables for this example
    std::env::set_var("ARTIFICE_LOG_LEVEL", "debug");
    std::env::set_var("ARTIFICE_LOG_FILE", "example_env_config.log");
    std::env::set_var("ARTIFICE_LOG_CONSOLE", "true");
    std::env::set_var("ARTIFICE_LOG_COLORS", "true");
    std::env::set_var("ARTIFICE_LOG_BATCH_SIZE", "30");
    std::env::set_var("ARTIFICE_LOG_FLUSH_INTERVAL", "75");
    
    // Initialize from environment variables
    let _ = init_from_env();
    
    log::trace!("This trace message might not appear depending on log level");
    log::debug!("Environment configuration loaded successfully");
    log::info!("Log level: {:?}", get_log_level());
    log::warn!("Environment variables make deployment configuration easy");
    log::error!("Different environments can have different log settings");
    
    flush();
    thread::sleep(Duration::from_millis(100));
    
    // Clean up environment variables
    std::env::remove_var("ARTIFICE_LOG_LEVEL");
    std::env::remove_var("ARTIFICE_LOG_FILE");
    std::env::remove_var("ARTIFICE_LOG_CONSOLE");
    std::env::remove_var("ARTIFICE_LOG_COLORS");
    std::env::remove_var("ARTIFICE_LOG_BATCH_SIZE");
    std::env::remove_var("ARTIFICE_LOG_FLUSH_INTERVAL");
    
    println!("✓ Environment configuration example completed\n");
    Ok(())
}

/// Helper function to demonstrate different log levels
#[allow(dead_code)]
fn demonstrate_log_levels() {
    // Set different log levels to see filtering in action
    set_log_level(LogLevel::Info);
    log::trace!("This trace won't appear (level too low)");
    log::debug!("This debug won't appear (level too low)");
    log::info!("This info will appear");
    log::warn!("This warning will appear");
    log::error!("This error will appear");
    
    set_log_level(LogLevel::Trace);
    log::trace!("Now trace messages appear");
    log::debug!("And debug messages too");
}

/// Helper function to demonstrate performance
#[allow(dead_code)]
fn demonstrate_performance() -> Result<(), LoggerError> {
    let log_file = "performance_test.log";
    
    // High-performance configuration
    let batch_config = BatchConfig {
        batch_size: 1000,
        flush_interval_ms: 50,
        enabled: true,
        buffer_capacity: 2000,
        string_pool_size: 1000,
    };
    
    let config = LogConfig::default();
    init_with_file_and_batching(log_file, config, batch_config)?;
    
    let message_count = 10000;
    let start = std::time::Instant::now();
    
    for i in 0..message_count {
        log::info!("High-performance message {} with data: {}", i, i * 42);
    }
    
    flush();
    let duration = start.elapsed();
    
    log::info!("Logged {} messages in {:?} ({:.2} msg/sec)", 
              message_count, duration, message_count as f64 / duration.as_secs_f64());
    
    Ok(())
}