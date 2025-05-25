use logging::{debug, error, info, trace, warn};
use std::env;

fn main() {
    println!("=== Environment Variable Configuration Demo ===\n");

    // Show current environment variables
    println!("Current environment variables:");
    println!("ARTIFICE_LOG_LEVEL: {:?}", env::var("ARTIFICE_LOG_LEVEL"));
    println!("ARTIFICE_LOG_CONSOLE: {:?}", env::var("ARTIFICE_LOG_CONSOLE"));
    println!("ARTIFICE_LOG_COLORS: {:?}", env::var("ARTIFICE_LOG_COLORS"));
    println!("ARTIFICE_LOG_FILE: {:?}", env::var("ARTIFICE_LOG_FILE"));
    println!();

    // Initialize logger from environment variables
    println!("Initializing logger from environment variables...");
    match logging::init_from_env() {
        Ok(()) => println!("Logger initialized successfully from environment!"),
        Err(e) => {
            eprintln!("Failed to initialize logger: {}", e);
            return;
        }
    }
    println!();

    // Demo logging at all levels
    println!("Demo: Logging at all levels");
    println!("(Level filtering depends on ARTIFICE_LOG_LEVEL environment variable)");
    error!("This is an ERROR message");
    warn!("This is a WARN message");
    info!("This is an INFO message");
    debug!("This is a DEBUG message");
    trace!("This is a TRACE message");
    println!();

    // Show current effective log level
    println!("Current effective log level: {:?}", logging::get_log_level());
    println!();

    // Demo application logging
    println!("Demo: Application logging");
    info!("Application started");
    debug!("Loading configuration...");
    info!("Configuration loaded");
    
    // Simulate some work
    for i in 1..=3 {
        debug!("Processing item {}", i);
        if i == 2 {
            warn!("Item {} took longer than expected", i);
        }
    }
    
    info!("Processing completed");
    println!();

    // Demo error handling
    println!("Demo: Error scenarios");
    warn!("This is a warning about something");
    error!("This is an error that occurred");
    debug!("Additional debug information about the error");
    println!();

    println!("=== Environment Variable Demo Complete ===");
    println!();
    println!("To customize the logging behavior, set these environment variables:");
    println!("export ARTIFICE_LOG_LEVEL=DEBUG     # Set log level (ERROR, WARN, INFO, DEBUG, TRACE)");
    println!("export ARTIFICE_LOG_CONSOLE=true    # Enable/disable console output");
    println!("export ARTIFICE_LOG_COLORS=false    # Enable/disable colors in console");
    println!("export ARTIFICE_LOG_FILE=app.log    # Write logs to a file");
    println!();
    println!("Then run this demo again to see the different behavior!");
    println!();
    println!("Example configurations:");
    println!("1. Debug level with file output:");
    println!("   ARTIFICE_LOG_LEVEL=DEBUG ARTIFICE_LOG_FILE=debug.log cargo run --example env_demo");
    println!();
    println!("2. Error level only, no colors:");
    println!("   ARTIFICE_LOG_LEVEL=ERROR ARTIFICE_LOG_COLORS=false cargo run --example env_demo");
    println!();
    println!("3. File only (no console output):");
    println!("   ARTIFICE_LOG_CONSOLE=false ARTIFICE_LOG_FILE=silent.log cargo run --example env_demo");
}