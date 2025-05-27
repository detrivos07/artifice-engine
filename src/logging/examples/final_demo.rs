//! Final working demo of the Artifice Logging library
//! 
//! This example demonstrates the library functionality with code that actually compiles.

use artifice_logging::*;

use log::Log;
use std::thread;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Artifice Logging Final Demo ===\n");

    // Demo 1: Basic console logger
    demo_console_logger()?;
    
    // Demo 2: File logger with batching
    demo_file_logger()?;
    
    // Demo 3: DoD writer directly
    demo_high_performance_writer()?;

    println!("\n=== All demos completed successfully! ===");
    Ok(())
}

fn demo_console_logger() -> Result<(), LoggerError> {
    println!("--- Demo 1: Console Logger ---");
    
    let logger = ArtificeLogger::new();
    
    // Create log records manually to avoid borrowing issues
    let info_record = log::Record::builder()
        .args(format_args!("Application started successfully"))
        .level(log::Level::Info)
        .target("demo")
        .module_path(Some("final_demo"))
        .file(Some("final_demo.rs"))
        .line(Some(30))
        .build();
    
    let warn_record = log::Record::builder()
        .args(format_args!("This is a warning message"))
        .level(log::Level::Warn)
        .target("demo")
        .module_path(Some("final_demo"))
        .file(Some("final_demo.rs"))
        .line(Some(40))
        .build();
    
    let error_record = log::Record::builder()
        .args(format_args!("This is an error message"))
        .level(log::Level::Error)
        .target("demo")
        .module_path(Some("final_demo"))
        .file(Some("final_demo.rs"))
        .line(Some(50))
        .build();
    
    logger.log(&info_record);
    logger.log(&warn_record);
    logger.log(&error_record);
    
    println!("✓ Console logging completed\n");
    Ok(())
}

fn demo_file_logger() -> Result<(), LoggerError> {
    println!("--- Demo 2: File Logger with Batching ---");
    
    let log_file = "final_demo.log";
    
    let config = LogConfig {
        console: true,
        file: true,
        colors: false,
    };
    
    let batch_config = BatchConfig {
        batch_size: 3,
        flush_interval_ms: 100,
        enabled: true,
        buffer_capacity: 10,
        string_pool_size: 5,
    };
    
    let logger = ArtificeLogger::new()
        .with_batch_config(batch_config)
        .with_file(log_file)?;
    
    // Create multiple log records
    let records = vec![
        ("File logging initialized", log::Level::Info),
        ("Processing first request", log::Level::Debug),
        ("Memory usage is high", log::Level::Warn),
        ("Database connection failed", log::Level::Error),
        ("Retrying connection", log::Level::Info),
        ("Connection restored", log::Level::Info),
    ];
    
    for (i, (message, level)) in records.iter().enumerate() {
        match level {
            log::Level::Error => log::error!(target: "file_demo", "{}", message),
            log::Level::Warn => log::warn!(target: "file_demo", "{}", message),
            log::Level::Info => log::info!(target: "file_demo", "{}", message),
            log::Level::Debug => log::debug!(target: "file_demo", "{}", message),
            log::Level::Trace => log::trace!(target: "file_demo", "{}", message),
        }
        thread::sleep(Duration::from_millis(50));
    }
    
    logger.flush();
    thread::sleep(Duration::from_millis(200));
    
    println!("✓ File logging completed");
    println!("✓ Check '{}' for output", log_file);
    
    // Show file contents if it exists
    if let Ok(content) = std::fs::read_to_string(log_file) {
        println!("\n--- Log file content ---");
        print!("{}", content);
        println!("--- End of log file ---\n");
    }
    
    Ok(())
}

fn demo_high_performance_writer() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 3: High-Performance Writer Direct Usage ---");
    
    let config = HighPerformanceConfig {
        batch_size: 5,
        flush_interval_ms: 100,
        enabled: true,
        buffer_capacity: 20,
        string_pool_size: 10,
    };
    
    let logger = HighPerformanceLogger::new("high_performance_demo.log", config)?;
    
    // Write messages directly using high-performance logger
    let messages = vec![
        "High-performance message 1: High performance logging",
        "High-performance message 2: Structure of Arrays optimization",
        "High-performance message 3: String pooling in action",
        "High-performance message 4: Bulk write operations",
        "High-performance message 5: Memory efficient design",
        "High-performance message 6: Cache friendly access patterns",
        "High-performance message 7: Reduced system calls",
        "High-performance message 8: Pre-allocated buffers",
    ];
    
    for message in messages {
        logger.log_fast(message.to_string())?;
    }
    
    logger.flush()?;
    
    println!("✓ High-performance writer demo completed");
    println!("✓ Check 'high_performance_demo.log' for output");
    
    // Show high-performance log file contents
    if let Ok(content) = std::fs::read_to_string("high_performance_demo.log") {
        println!("\n--- High-performance log file content ---");
        print!("{}", content);
        println!("--- End of high-performance log file ---\n");
    }
    
    Ok(())
}

fn demo_configuration_patterns() {
    println!("--- Configuration Patterns ---");
    
    // Development configuration
    let _dev_config = LogConfig {
        console: true,   // See logs immediately
        file: true,      // Keep history
        colors: true,    // Better readability
    };
    
    let _dev_batch = BatchConfig {
        batch_size: 10,        // Small batches for quick feedback
        flush_interval_ms: 50, // Quick flushes
        enabled: true,
        buffer_capacity: 50,
        string_pool_size: 25,
    };
    
    // Production configuration
    let _prod_config = LogConfig {
        console: false,  // No console output
        file: true,      // File only
        colors: false,   // No colors
    };
    
    let _prod_batch = BatchConfig {
        batch_size: 500,         // Large batches for performance
        flush_interval_ms: 1000, // Less frequent flushes
        enabled: true,
        buffer_capacity: 2000,
        string_pool_size: 1000,
    };
    
    // High-performance configuration
    let _perf_batch = BatchConfig {
        batch_size: 1000,
        flush_interval_ms: 500,
        enabled: true,
        buffer_capacity: 5000,
        string_pool_size: 2000,
    };
    
    println!("✓ Configuration patterns demonstrated");
}