use logging::{debug, error, info, trace, warn, LoggerBuilder};

fn main() {
    println!("=== Console-Only Logging Demo ===\n");

    // Initialize logger for console output only (no file)
    println!("Initializing console-only logger with colors...");
    LoggerBuilder::new()
        .console(true)
        .colors(true)
        .init()
        .expect("Failed to initialize console-only logger");

    println!("Logger initialized successfully!\n");

    // Demo different log levels with colors
    println!("Demo: All log levels with colors");
    error!("This is an ERROR message (red)");
    warn!("This is a WARN message (yellow)");
    info!("This is an INFO message (green)");
    debug!("This is a DEBUG message (cyan)");
    trace!("This is a TRACE message (magenta)");

    println!("\nDemo: Log level filtering");
    println!("Setting log level to INFO - only INFO, WARN, and ERROR should show:");
    logging::set_log_level(logging::LogLevel::Info);

    trace!("This trace should NOT appear");
    debug!("This debug should NOT appear");
    info!("This info SHOULD appear");
    warn!("This warning SHOULD appear");
    error!("This error SHOULD appear");

    println!("\nResetting log level to TRACE");
    logging::set_log_level(logging::LogLevel::Trace);

    println!("\nDemo: Rapid logging");
    for i in 1..=5 {
        info!("Rapid log message #{}", i);
    }

    println!("\nDemo: Structured logging");
    let user_id = 42;
    let session_id = "abc123";
    info!("User login: user_id={}, session_id={}", user_id, session_id);
    debug!("Processing user data for user_id={}", user_id);
    info!("User login completed: user_id={}", user_id);

    println!("\n=== Console-Only Demo Complete ===");
    println!("All log output went to console only - no files were created");
    println!("Notice the colored output in your terminal!");
}