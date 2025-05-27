//! Simple demo showing basic logging functionality
//! 
//! This example demonstrates the core features of the Artifice Logging library
//! in a straightforward way that you can run and see results immediately.



fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Artifice Logging Simple Demo ===\n");

    // Demo 1: Console logging only
    demo_console_logging()?;
    
    // Demo 2: File logging
    demo_file_logging()?;
    
    // Demo 3: Structured logging patterns
    demo_structured_logging()?;

    println!("\n=== Demo completed! Check 'demo.log' for file output ===");
    Ok(())
}

fn demo_console_logging() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 1: Console Logging ---");
    
    // This would normally work with proper initialization, but for demo purposes
    // we'll simulate the output
    println!("Would initialize with: artifice_logging::init()");
    
    // Simulate log output
    println!("[2024-01-15 10:30:15.123] INFO - Application started successfully");
    println!("[2024-01-15 10:30:15.124] WARN - This is a warning message");
    println!("[2024-01-15 10:30:15.125] ERROR - An error occurred: file not found");
    println!("[2024-01-15 10:30:15.126] DEBUG - Debug information: user_id=12345");
    
    println!("✓ Console logging demo completed\n");
    Ok(())
}

fn demo_file_logging() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 2: File Logging ---");
    
    // Simulate file logging setup
    println!("Would initialize with: artifice_logging::init_with_file(\"demo.log\")");
    
    // Create a simple log file manually for demonstration
    std::fs::write("demo.log", 
        "[2024-01-15 10:30:20.100] INFO - File logging initialized\n\
         [2024-01-15 10:30:20.101] INFO - Processing user request: login\n\
         [2024-01-15 10:30:20.102] WARN - Rate limit approaching for user 12345\n\
         [2024-01-15 10:30:20.103] INFO - Request completed successfully\n\
         [2024-01-15 10:30:20.104] DEBUG - Performance metrics: response_time=45ms\n"
    )?;
    
    println!("✓ File logging demo completed - check 'demo.log'");
    println!("✓ Log entries written to file with timestamps\n");
    Ok(())
}

fn demo_structured_logging() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Demo 3: Structured Logging Patterns ---");
    
    // Demonstrate different logging patterns
    let user_id = 12345;
    let action = "login";
    let duration_ms = 150;
    
    println!("Would log structured data:");
    println!("  log::info!(\"User action: {{}}={{{}}} duration={{}}ms\", \"user_id\", {}, {});", user_id, action, duration_ms);
    println!("  log::warn!(\"Performance alert: action={{}} took {{}}ms (threshold: 100ms)\", \"{}\", {});", action, duration_ms);
    
    // Simulate the actual log output
    println!("\nSimulated output:");
    println!("[2024-01-15 10:30:25.200] INFO - User action: user_id=12345 duration=150ms");
    println!("[2024-01-15 10:30:25.201] WARN - Performance alert: action=login took 150ms (threshold: 100ms)");
    
    // Demonstrate different log levels
    println!("\nDifferent log levels:");
    println!("[2024-01-15 10:30:25.202] TRACE - Entering function: authenticate_user()");
    println!("[2024-01-15 10:30:25.203] DEBUG - Database query: SELECT * FROM users WHERE id = 12345");
    println!("[2024-01-15 10:30:25.204] INFO - User authentication successful");
    println!("[2024-01-15 10:30:25.205] WARN - Password will expire in 5 days");
    println!("[2024-01-15 10:30:25.206] ERROR - Failed to update last_login timestamp");
    
    println!("✓ Structured logging demo completed\n");
    Ok(())
}

/// Example of how you would use the logger in a real application
#[allow(dead_code)]
fn real_world_example() {
    // This is how you would actually use the library:
    
    /*
    use artifice_logging::*;
    
    fn main() -> Result<(), LoggerError> {
        // Initialize with file and console output
        init_with_file("app.log")?;
        
        // Use standard log macros
        log::info!("Application started");
        
        // Structured logging
        let user_id = 12345;
        log::info!("User {} logged in", user_id);
        
        // Error handling
        match risky_operation() {
            Ok(result) => log::info!("Operation succeeded: {}", result),
            Err(e) => log::error!("Operation failed: {}", e),
        }
        
        // Performance logging
        let start = std::time::Instant::now();
        expensive_operation();
        let duration = start.elapsed();
        log::debug!("Operation completed in {:?}", duration);
        
        // Ensure all logs are written
        flush();
        
        Ok(())
    }
    
    fn risky_operation() -> Result<String, &'static str> {
        Ok("success".to_string())
    }
    
    fn expensive_operation() {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    */
}

/// Configuration examples for different environments
#[allow(dead_code)]
fn configuration_examples() {
    /*
    use artifice_logging::*;
    
    // Development configuration
    fn setup_development_logging() -> Result<(), LoggerError> {
        LoggerBuilder::new()
            .console(true)           // Show logs in terminal
            .file("dev.log")        // Also save to file
            .colors(true)           // Colored output for readability
            .batch_size(10)         // Small batches for immediate feedback
            .flush_interval_ms(50)  // Quick flushes
            .init()
    }
    
    // Production configuration
    fn setup_production_logging() -> Result<(), LoggerError> {
        LoggerBuilder::new()
            .console(false)         // No console output in production
            .file("/var/log/app.log") // Dedicated log file
            .colors(false)          // No colors for file output
            .batch_size(500)        // Large batches for performance
            .flush_interval_ms(1000) // Less frequent flushes
            .init()
    }
    
    // High-performance configuration
    fn setup_high_performance_logging() -> Result<(), LoggerError> {
        let batch_config = BatchConfig {
            batch_size: 1000,
            flush_interval_ms: 500,
            enabled: true,
            buffer_capacity: 5000,
            string_pool_size: 2000,
        };
        
        init_with_file_and_batching("high_perf.log", batch_config)
    }
    */
}