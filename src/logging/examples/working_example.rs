//! Simple working example of the Artifice Logging library
//! 
//! This example demonstrates basic usage that actually compiles and runs.

use artifice_logging::*;

use std::thread;
use std::time::Duration;

fn main() -> Result<(), LoggerError> {
    println!("=== Artifice Logging Simple Working Example ===\n");

    // Example 1: Create logger and demonstrate basic functionality
    example_basic_usage()?;
    
    println!("\n=== Example completed successfully! ===");
    Ok(())
}

fn example_basic_usage() -> Result<(), LoggerError> {
    println!("--- Basic Usage Example ---");
    
    let log_file = "simple_example.log";
    
    let config = LogConfig {
        console: true,
        file: true,
        colors: false,
    };
    
    let batch_config = BatchConfig {
        batch_size: 5,
        flush_interval_ms: 100,
        enabled: true,
        buffer_capacity: 20,
        string_pool_size: 10,
    };
    
    // Create logger with file output
    let logger = ArtificeLogger::new()
        .with_batch_config(batch_config)
        .with_file(log_file)?;
    
    // Create some log messages manually
    let messages = vec![
        ("Application started", log::Level::Info),
        ("Configuration loaded", log::Level::Debug),
        ("Processing request 1", log::Level::Info),
        ("Warning: high memory usage", log::Level::Warn),
        ("Error: connection failed", log::Level::Error),
        ("Request completed", log::Level::Info),
    ];
    
    for (i, (msg, level)) in messages.iter().enumerate() {
        let message = format!("{} (id: {})", msg, i + 1);
        match level {
            log::Level::Error => log::error!(target: "example", "{}", message),
            log::Level::Warn => log::warn!(target: "example", "{}", message),
            log::Level::Info => log::info!(target: "example", "{}", message),
            log::Level::Debug => log::debug!(target: "example", "{}", message),
            log::Level::Trace => log::trace!(target: "example", "{}", message),
        }
    }
    
    // Force flush to ensure all messages are written
    logger.flush();
    thread::sleep(Duration::from_millis(200));
    
    println!("✓ Generated {} log messages", messages.len());
    println!("✓ Check '{}' for the log output", log_file);
    
    // Show the log file content
    if let Ok(content) = std::fs::read_to_string(log_file) {
        println!("\n--- Log file content ---");
        println!("{}", content);
        println!("--- End of log file ---");
    }
    
    Ok(())
}

// Demonstrate high-performance logger directly
#[allow(dead_code)]
fn high_performance_example() -> Result<(), Box<dyn std::error::Error>> {
    let config = HighPerformanceConfig {
        batch_size: 3,
        flush_interval_ms: 50,
        enabled: true,
        buffer_capacity: 10,
        string_pool_size: 5,
    };
    
    let logger = HighPerformanceLogger::new("high_performance_example.log", config)?;
    
    for i in 0..10 {
        let message = format!("High-performance message {}: efficient logging", i);
        logger.log_fast(message)?;
    }
    
    logger.flush()?;
    
    println!("✓ DoD writer example completed");
    Ok(())
}