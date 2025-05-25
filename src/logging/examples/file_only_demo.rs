use logging::{debug, error, info, trace, warn, LoggerBuilder};
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== File-Only Logging Demo ===\n");

    // Initialize logger for file output only (no console)
    println!("Initializing file-only logger...");
    LoggerBuilder::new()
        .console(false)
        .file("file_only_demo.log")
        .init()
        .expect("Failed to initialize file-only logger");

    println!("Logger initialized successfully!");
    println!("All log messages will be written to 'file_only_demo.log' only\n");

    // Demo different log levels - these won't appear in console
    println!("Writing log messages to file...");
    error!("This is an ERROR message (goes to file only)");
    warn!("This is a WARN message (goes to file only)");
    info!("This is an INFO message (goes to file only)");
    debug!("This is a DEBUG message (goes to file only)");
    trace!("This is a TRACE message (goes to file only)");

    println!("Wrote 5 log messages at different levels");

    // Demo log level filtering
    println!("\nTesting log level filtering (setting to WARN)...");
    logging::set_log_level(logging::LogLevel::Warn);

    trace!("This trace should NOT appear in file");
    debug!("This debug should NOT appear in file");
    info!("This info should NOT appear in file");
    warn!("This warning SHOULD appear in file");
    error!("This error SHOULD appear in file");

    println!("Wrote filtered log messages (only WARN and ERROR should be in file)");

    // Reset log level
    logging::set_log_level(logging::LogLevel::Trace);

    // Demo batch logging
    println!("\nWriting batch of application logs...");
    info!("Application starting up");
    debug!("Loading configuration from config.toml");
    info!("Configuration loaded successfully");
    debug!("Initializing database connection");
    info!("Database connected");
    warn!("Using default cache size (1024MB)");
    info!("Application ready to serve requests");

    println!("Wrote application startup sequence to file");

    // Demo error scenarios
    println!("\nSimulating error scenarios...");
    warn!("Low disk space detected: 15% remaining");
    error!("Failed to connect to external service: timeout after 30s");
    error!("Database query failed: connection lost");
    info!("Attempting to reconnect to database");
    info!("Database reconnection successful");

    println!("Wrote error simulation sequence to file");

    // Demo performance logging
    println!("\nWriting performance metrics...");
    let start = std::time::Instant::now();
    
    // Simulate some work
    thread::sleep(Duration::from_millis(100));
    
    let duration = start.elapsed();
    info!("Request processed in {:?}", duration);
    debug!("Cache hit rate: 85.7%");
    debug!("Memory usage: 342MB");
    trace!("GC triggered: minor collection");

    println!("Wrote performance metrics to file");

    // Demo structured data logging
    println!("\nWriting structured application data...");
    let request_id = "req_123456";
    let user_id = 789;
    let endpoint = "/api/users";
    
    info!("Request started: id={}, user_id={}, endpoint={}", request_id, user_id, endpoint);
    debug!("Validating request parameters for request_id={}", request_id);
    debug!("User authorization check passed for user_id={}", user_id);
    info!("Request completed: id={}, status=200, duration=45ms", request_id);

    println!("Wrote structured request data to file");

    println!("\n=== File-Only Demo Complete ===");
    println!("Check 'file_only_demo.log' to see all the logged messages");
    println!("Notice that no log messages appeared in this console output - they all went to the file!");
    println!("The file contains timestamps and no color codes since colors are only for console output");
}