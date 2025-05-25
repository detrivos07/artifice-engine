use logging::{debug, error, info, trace, warn, LoggerBuilder, LogLevel};
use std::thread;
use std::time::Duration;

fn main() {
    println!("=== Artifice Logging System Demo ===\n");

    // Initialize the logger once with comprehensive settings
    // Note: The global logger can only be initialized once per process
    println!("Initializing logger with console + file output and colors enabled...");
    LoggerBuilder::new()
        .console(true)
        .file("comprehensive_demo.log")
        .colors(true)
        .init()
        .expect("Failed to initialize logger");

    println!("Logger initialized successfully!\n");

    // Demo 1: Basic logging at different levels
    println!("Demo 1: Basic logging at different levels");
    info!("This is an info message - shows in console with green color and in file");
    warn!("This is a warning message - shows in console with yellow color and in file");
    error!("This is an error message - shows in console with red color and in file");
    debug!("This is a debug message - shows in console with cyan color and in file");
    trace!("This is a trace message - shows in console with magenta color and in file");
    println!();

    // Demo 2: Log level filtering
    println!("Demo 2: Log level filtering");
    println!("Current log level: {:?}", logging::get_log_level());
    
    println!("\nSetting log level to WARN - only WARN and ERROR should show:");
    logging::set_log_level(LogLevel::Warn);
    
    trace!("This trace should NOT appear");
    debug!("This debug should NOT appear");
    info!("This info should NOT appear");
    warn!("This warning SHOULD appear");
    error!("This error SHOULD appear");

    println!("\nSetting log level back to TRACE - all should show:");
    logging::set_log_level(LogLevel::Trace);
    
    trace!("Now trace appears");
    debug!("Now debug appears");
    info!("Now info appears");
    warn!("Warning still appears");
    error!("Error still appears");
    println!();

    // Demo 3: Logging with different modules
    println!("Demo 3: Logging from different modules");
    info!("This message comes from the main module");
    warn!("This warning demonstrates standard logging");
    error!("This error shows how errors are logged");
    println!();

    // Demo 4: Multi-threaded logging
    println!("Demo 4: Multi-threaded logging");
    let handles: Vec<_> = (0..3)
        .map(|i| {
            thread::spawn(move || {
                for j in 0..3 {
                    info!("Thread {} message {}", i, j);
                    thread::sleep(Duration::from_millis(100));
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
    println!();

    // Demo 5: Structured logging with context
    println!("Demo 5: Structured logging with context");
    let user_id = 12345;
    let operation = "user_login";
    
    info!("User operation started: user_id={}, operation={}", user_id, operation);
    debug!("Validating user credentials for user_id={}", user_id);
    info!("User operation completed successfully: user_id={}, operation={}", user_id, operation);
    println!();

    // Demo 6: Error handling and logging
    println!("Demo 6: Error handling and logging");
    fn simulate_operation(success: bool) -> Result<String, String> {
        if success {
            info!("Operation succeeded");
            Ok("Success".to_string())
        } else {
            error!("Operation failed: Invalid input provided");
            Err("Invalid input".to_string())
        }
    }

    match simulate_operation(true) {
        Ok(result) => info!("Got result: {}", result),
        Err(e) => error!("Operation failed: {}", e),
    }

    match simulate_operation(false) {
        Ok(result) => info!("Got result: {}", result),
        Err(e) => error!("Operation failed: {}", e),
    }
    println!();

    // Demo 7: Performance logging
    println!("Demo 7: Performance logging");
    let start = std::time::Instant::now();
    
    // Simulate some work
    thread::sleep(Duration::from_millis(150));
    
    let duration = start.elapsed();
    info!("Operation completed in {:?}", duration);
    
    if duration > Duration::from_millis(100) {
        warn!("Operation took longer than expected: {:?}", duration);
    }
    println!();

    // Demo 8: Different builder configurations (shown as examples)
    println!("Demo 8: Other LoggerBuilder configurations");
    println!("Note: These show API usage but can't be applied since logger is already initialized");
    println!();
    
    println!("Console-only logging (no file):");
    println!("LoggerBuilder::new().console(true).colors(true).init()");
    println!();
    
    println!("File-only logging (no console):");
    println!("LoggerBuilder::new().console(false).file(\"app.log\").init()");
    println!();
    
    println!("Console without colors:");
    println!("LoggerBuilder::new().console(true).colors(false).init()");
    println!();

    // Demo 9: Environment variable configuration info
    println!("Demo 9: Environment variable configuration");
    println!("The logger supports these environment variables:");
    println!("- ARTIFICE_LOG_LEVEL: ERROR, WARN, INFO, DEBUG, TRACE");
    println!("- ARTIFICE_LOG_CONSOLE: true/false");
    println!("- ARTIFICE_LOG_COLORS: true/false");
    println!("- ARTIFICE_LOG_FILE: path to log file");
    println!();
    println!("Use logging::init_from_env() to configure from environment variables");
    println!();

    // Final messages
    info!("Demo completed successfully!");
    println!("\n=== Demo Complete ===");
    println!("Check 'comprehensive_demo.log' to see the file output");
    println!("Notice that file output has no color codes, while console output does");
}