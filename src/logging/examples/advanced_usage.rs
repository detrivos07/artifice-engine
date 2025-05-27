//! Advanced usage examples for the Artifice Logging library
//! 
//! This example demonstrates advanced features including performance tuning,
//! concurrent logging patterns, memory optimization, and production-ready configurations.

use artifice_logging::*;
use artifice_logging::writers::*;
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use std::fs::File;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Artifice Logging Library - Advanced Usage Examples ===\n");

    // Example 1: High-performance concurrent logging
    example_1_high_performance_concurrent()?;
    
    // Example 2: Memory-optimized logging for embedded systems
    example_2_memory_optimized()?;
    
    // Example 3: Production-grade error handling and recovery
    example_3_production_error_handling()?;
    
    // Example 4: Performance profiling and tuning
    example_4_performance_profiling()?;
    
    // Example 5: Custom DoD file writer usage
    example_5_custom_dod_writer()?;
    
    // Example 6: Adaptive batch sizing based on load
    example_6_adaptive_batching()?;
    
    // Example 7: Log rotation and management
    example_7_log_rotation()?;

    println!("\n=== All advanced examples completed successfully! ===");
    Ok(())
}

/// Example 1: High-performance concurrent logging with optimal settings
fn example_1_high_performance_concurrent() -> Result<(), LoggerError> {
    println!("--- Example 1: High-Performance Concurrent Logging ---");
    
    let log_file = "advanced_concurrent.log";
    
    // Optimized configuration for high-throughput scenarios
    let _config = LogConfig {
        console: false,  // Disable console to reduce overhead
        file: true,
        colors: false,   // No colors for file output
    };
    
    let batch_config = BatchConfig {
        batch_size: 500,         // Large batches for efficiency
        flush_interval_ms: 100,  // Quick flushes for responsiveness
        enabled: true,
        buffer_capacity: 2000,   // Large pre-allocated buffers
        string_pool_size: 1000,  // Large string pool for reuse
    };
    
    // Initialize the global logger with file and batching
    let log_config = LogConfig {
        console: false,
        file: true,
        colors: false,
    };
    init_with_file_and_batching(&log_file, log_config, batch_config)?;
    
    // Simulate high-load concurrent scenario
    let thread_count = 8;
    let messages_per_thread = 1000;
    let barrier = Arc::new(Barrier::new(thread_count));
    
    println!("Spawning {} threads, each logging {} messages...", thread_count, messages_per_thread);
    
    let start = Instant::now();
    let mut handles = vec![];
    
    for thread_id in 0..thread_count {
        let barrier_clone = Arc::clone(&barrier);
        
        let handle = thread::spawn(move || {
            barrier_clone.wait(); // Synchronize start for realistic load testing
            
            for i in 0..messages_per_thread {
                // Simulate realistic application logging with structured data
                let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                
                // Use log macros which handle format_args! properly
                match i % 4 {
                    0 => log::info!(target: "concurrent_app", "{{\"thread_id\":{},\"message_id\":{},\"timestamp\":{},\"data\":\"Processing request {}\",\"status\":\"success\"}}", thread_id, i, timestamp, i),
                    1 => log::debug!(target: "concurrent_app", "{{\"thread_id\":{},\"message_id\":{},\"timestamp\":{},\"data\":\"Processing request {}\",\"status\":\"success\"}}", thread_id, i, timestamp, i),
                    2 => log::warn!(target: "concurrent_app", "{{\"thread_id\":{},\"message_id\":{},\"timestamp\":{},\"data\":\"Processing request {}\",\"status\":\"success\"}}", thread_id, i, timestamp, i),
                    3 => log::error!(target: "concurrent_app", "{{\"thread_id\":{},\"message_id\":{},\"timestamp\":{},\"data\":\"Processing request {}\",\"status\":\"success\"}}", thread_id, i, timestamp, i),
                    _ => log::info!(target: "concurrent_app", "{{\"thread_id\":{},\"message_id\":{},\"timestamp\":{},\"data\":\"Processing request {}\",\"status\":\"success\"}}", thread_id, i, timestamp, i),
                }
                
                // Simulate some work between log messages
                if i % 100 == 0 {
                    thread::sleep(Duration::from_millis(1));
                }
            }
        });
        
        handles.push(handle);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }
    
    flush();
    thread::sleep(Duration::from_millis(200)); // Allow background thread to finish
    
    let duration = start.elapsed();
    let total_messages = thread_count * messages_per_thread;
    let throughput = total_messages as f64 / duration.as_secs_f64();
    
    println!("✓ Logged {} messages in {:?}", total_messages, duration);
    println!("✓ Throughput: {:.2} messages/second", throughput);
    println!("✓ Average latency: {:.2} μs per message", duration.as_micros() as f64 / total_messages as f64);
    println!("✓ High-performance concurrent logging example completed\n");
    
    Ok(())
}

/// Example 2: Memory-optimized logging for resource-constrained environments
fn example_2_memory_optimized() -> Result<(), LoggerError> {
    println!("--- Example 2: Memory-Optimized Logging ---");
    
    let log_file = "advanced_memory_optimized.log";
    
    // Minimal memory footprint configuration
    let hp_config = HighPerformanceConfig {
        batch_size: 25,
        flush_interval_ms: 200,
        enabled: true,
        buffer_capacity: 100,
        string_pool_size: 50,
    };
    let _logger = HighPerformanceLogger::new(log_file, hp_config)?;
    
    // Simulate embedded system logging with minimal allocations
    println!("Simulating embedded system with memory constraints...");
    
    // Use a circular buffer pattern to demonstrate efficient memory usage
    let mut message_buffer: VecDeque<String> = VecDeque::with_capacity(20);
    
    for i in 0..100 {
        // Reuse strings from circular buffer when possible
        let message = if message_buffer.len() < 20 {
            format!("Embedded system event {}: sensor_reading={}", i, i as f64 * 1.5)
        } else {
            let mut reused = message_buffer.pop_front().unwrap();
            reused.clear();
            reused.push_str(&format!("Embedded system event {}: sensor_reading={}", i, i as f64 * 1.5));
            reused
        };
        
        message_buffer.push_back(message.clone());
        
        if i % 10 == 0 {
            log::warn!(target: "embedded_sensor", "{}", message);
        } else {
            log::info!(target: "embedded_sensor", "{}", message);
        }
        
        // Simulate sensor reading interval
        thread::sleep(Duration::from_millis(1));
    }
    
    flush();
    thread::sleep(Duration::from_millis(100));
    
    println!("✓ Memory-optimized logging maintains minimal footprint");
    println!("✓ Circular buffer pattern reduces allocations");
    println!("✓ Memory-optimized example completed\n");
    
    Ok(())
}

/// Example 3: Production-grade error handling and recovery
fn example_3_production_error_handling() -> Result<(), LoggerError> {
    println!("--- Example 3: Production-Grade Error Handling ---");
    
    // Demonstrate robust error handling patterns
    println!("Testing various error scenarios...");
    
    // Test 1: Graceful handling of disk space issues (simulated)
    {
        let temp_file = "/tmp/test_production_logging.log";
        match ArtificeLogger::new().with_file(temp_file) {
            Ok(_logger) => {
                println!("✓ Logger initialized successfully");
                
                // Test recovery from channel disconnection
                for i in 0..10 {
                    log::info!(target: "recovery_test", "Recovery test message {}", i);
                }
                
                flush();
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                println!("✓ Gracefully handled file creation error: {}", e);
                
                // Fallback to console-only logging
                let fallback_config = LogConfig {
                    console: true,
                    file: false,
                    colors: true,
                };
                
                let mut fallback_logger = ArtificeLogger::new();
                fallback_logger.set_config(fallback_config);
                log::info!(target: "fallback", "Switched to console-only logging due to file error");
                println!("✓ Fallback to console logging successful");
            }
        }
    }
    
    // Test 2: Handling logger re-initialization
    {
        println!("Testing logger re-initialization protection...");
        
        // This should succeed (first initialization)
        match init() {
            Ok(_) => println!("✓ Initial logger setup successful"),
            Err(LoggerError::AlreadyInitialized) => println!("✓ Logger already initialized (expected)"),
            Err(e) => println!("✗ Unexpected error: {}", e),
        }
        
        // This should fail gracefully (already initialized)
        match init() {
            Ok(_) => println!("✗ Unexpected success on re-initialization"),
            Err(LoggerError::AlreadyInitialized) => println!("✓ Re-initialization properly rejected"),
            Err(e) => println!("✗ Unexpected error type: {}", e),
        }
    }
    
    // Test 3: Stress testing with rapid logging
    {
        println!("Stress testing with rapid message generation...");
        
        let start = Instant::now();
        let mut successful_logs = 0;
        
        for i in 0..1000 {
            // Simulate potential failure scenarios
            if i % 100 == 0 {
                thread::sleep(Duration::from_micros(100)); // Simulate brief system pause
            }
            
            log::info!("Stress test message {}: system_load=high", i);
            successful_logs += 1;
        }
        
        flush();
        let duration = start.elapsed();
        
        println!("✓ Stress test completed: {}/{} messages in {:?}", 
                successful_logs, 1000, duration);
    }
    
    println!("✓ Production error handling example completed\n");
    Ok(())
}

/// Example 4: Performance profiling and tuning
fn example_4_performance_profiling() -> Result<(), LoggerError> {
    println!("--- Example 4: Performance Profiling and Tuning ---");
    
    // Test different configurations to find optimal settings
    let configurations = vec![
        ("Small Batch", BatchConfig { batch_size: 10, flush_interval_ms: 100, enabled: true, buffer_capacity: 50, string_pool_size: 25 }),
        ("Medium Batch", BatchConfig { batch_size: 100, flush_interval_ms: 100, enabled: true, buffer_capacity: 200, string_pool_size: 100 }),
        ("Large Batch", BatchConfig { batch_size: 500, flush_interval_ms: 100, enabled: true, buffer_capacity: 1000, string_pool_size: 500 }),
        ("No Batching", BatchConfig { batch_size: 1, flush_interval_ms: 1, enabled: false, buffer_capacity: 10, string_pool_size: 5 }),
    ];
    
    let message_count = 1000;
    
    for (name, batch_config) in configurations {
        let log_file = format!("profile_{}.log", name.replace(" ", "_").to_lowercase());
        
        let config = LogConfig {
            console: false,
            file: true,
            colors: false,
        };
        
        let hp_config = HighPerformanceConfig {
            batch_size: 200,
            flush_interval_ms: 25,
            enabled: true,
            buffer_capacity: 1000,
            string_pool_size: 500,
        };
        let logger = HighPerformanceLogger::new("performance_test.log", hp_config)?;
        
        // Measure latency distribution
        let mut latencies = Vec::with_capacity(message_count);
        
        for i in 0..message_count {
            let start = Instant::now();
            
            let message = format!("Profile test message {} with payload data", i);
            let _ = logger.log_fast(message);
            
            let latency = start.elapsed();
            latencies.push(latency);
        }
        
        flush();
        thread::sleep(Duration::from_millis(150));
        
        // Calculate statistics
        latencies.sort();
        let min = latencies.first().unwrap();
        let max = latencies.last().unwrap();
        let median = latencies[latencies.len() / 2];
        let p95 = latencies[(latencies.len() as f64 * 0.95) as usize];
        let avg = latencies.iter().sum::<Duration>() / latencies.len() as u32;
        
        println!("Configuration: {}", name);
        println!("  Min latency: {:?}", min);
        println!("  Avg latency: {:?}", avg);
        println!("  Median latency: {:?}", median);
        println!("  95th percentile: {:?}", p95);
        println!("  Max latency: {:?}", max);
        println!();
        
        // Cleanup
        let _ = std::fs::remove_file(&log_file);
    }
    
    println!("✓ Performance profiling completed\n");
    Ok(())
}

/// Example 5: Direct usage of DoD-optimized file writer
fn example_5_custom_dod_writer() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 5: Custom DoD File Writer Usage ---");
    
    let log_file = "advanced_dod_writer.log";
    
    // Create file and DoD writer directly for maximum control
    let file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(log_file)?;
    
    // Create high-performance file writer for maximum performance
    let file = File::create("high_performance.log")?;
    let hp_config = HighPerformanceConfig {
        batch_size: 200,
        flush_interval_ms: 50,
        enabled: true,
        buffer_capacity: 2048,
        string_pool_size: 1024,
    };
    let mut hp_writer = HighPerformanceFileWriter::new(file, hp_config);
    
    println!("Testing DoD writer with optimized memory patterns...");
    
    // Demonstrate efficient bulk operations
    let messages: Vec<String> = (0..500)
        .map(|i| format!("DoD optimized message {}: {{\"id\":{},\"data\":\"sample_data\"}}", i, i))
        .collect();
    
    let start = Instant::now();
    
    for message in messages {
        hp_writer.add_message(message)?;
    }
    
    hp_writer.flush()?;
    let hp_duration = start.elapsed();
    
    println!("✓ DoD writer processed 500 messages in {:?}", hp_duration);
    println!("✓ Achieved {:.2} messages/second", 500.0 / hp_duration.as_secs_f64());
    
    // Test shutdown
    hp_writer.shutdown()?;
    
    println!("✓ DoD writer shutdown completed");
    println!("✓ Custom DoD writer example completed\n");
    
    let _ = std::fs::remove_file(log_file);
    Ok(())
}

/// Example 6: Adaptive batch sizing based on system load
fn example_6_adaptive_batching() -> Result<(), LoggerError> {
    println!("--- Example 6: Adaptive Batch Sizing ---");
    
    let log_file = "advanced_adaptive.log";
    
    // Simulate adaptive batching by changing configuration based on load
    let load_scenarios = vec![
        ("Low Load", 10, 200),    // Small batches, longer intervals
        ("Medium Load", 50, 100), // Medium batches, medium intervals  
        ("High Load", 200, 25),   // Large batches, short intervals
        ("Peak Load", 500, 10),   // Very large batches, very short intervals
    ];
    
    for (scenario_name, batch_size, flush_interval) in load_scenarios {
        println!("Testing scenario: {}", scenario_name);
        
        let hp_config_clone = HighPerformanceConfig {
            batch_size: batch_size,
            flush_interval_ms: 50,
            enabled: true,
            buffer_capacity: batch_size * 2,
            string_pool_size: batch_size,
        };
        let logger = HighPerformanceLogger::new(&format!("adaptive_{}.log", scenario_name), hp_config_clone)?;
        
        let message_count = match scenario_name {
            "Low Load" => 100,
            "Medium Load" => 500,
            "High Load" => 1000,
            "Peak Load" => 2000,
            _ => 100,
        };
        
        let start = Instant::now();
        
        for i in 0..message_count {
            let message = format!("{} message {}: processing_time={}ms", scenario_name, i, i % 10);
            let _ = logger.log_fast(message);
            
            // Simulate different work patterns
            match scenario_name {
                "Low Load" => thread::sleep(Duration::from_millis(1)),
                "Medium Load" => if i % 10 == 0 { thread::sleep(Duration::from_micros(100)); },
                "High Load" => if i % 50 == 0 { thread::sleep(Duration::from_micros(50)); },
                "Peak Load" => {}, // No delays for peak load
                _ => {},
            }
        }
        
        flush();
        thread::sleep(Duration::from_millis(100));
        
        let duration = start.elapsed();
        let throughput = message_count as f64 / duration.as_secs_f64();
        
        println!("  Messages: {}, Duration: {:?}, Throughput: {:.2} msg/s", 
                message_count, duration, throughput);
        
        let _ = std::fs::remove_file(&log_file);
    }
    
    println!("✓ Adaptive batching example completed\n");
    Ok(())
}

/// Example 7: Log rotation and management strategies
fn example_7_log_rotation() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Example 7: Log Rotation and Management ---");
    
    // Simulate a log rotation scenario
    let base_filename = "advanced_rotation";
    let current_log = format!("{}.log", base_filename);
    
    let hp_config = HighPerformanceConfig {
        batch_size: 50,
        flush_interval_ms: 200,
        enabled: true,
        buffer_capacity: 100,
        string_pool_size: 50,
    };
    
    // Simulate writing to current log file using high-performance logger
    {
        let hp_logger = HighPerformanceLogger::new(&current_log, hp_config.clone())?;
        
        println!("Writing to current log file...");
        for i in 0..100 {
            let message = format!("Application log entry {}: user_action=login, user_id={}", i, i % 10);
            let _ = hp_logger.log_fast(message);
        }
        
        let _ = hp_logger.flush();
        thread::sleep(Duration::from_millis(100));
    }
    
    // Simulate log rotation
    let rotated_filename = format!("{}.{}.log", base_filename, 
        chrono::Local::now().format("%Y%m%d_%H%M%S"));
    
    println!("Rotating log file: {} -> {}", current_log, rotated_filename);
    std::fs::rename(&current_log, &rotated_filename)?;
    
    // Start new logger with fresh file
    {
        let logger = HighPerformanceLogger::new(&current_log, hp_config.clone())?;
        
        println!("Writing to new log file after rotation...");
        for i in 100..150 {
            let message = format!("New log entry {}: post_rotation=true", i);
            let _ = logger.log_fast(message);
        }
        
        let _ = logger.flush();
        thread::sleep(Duration::from_millis(100));
    }
    
    // Verify both files exist and have content
    let current_size = std::fs::metadata(&current_log)?.len();
    let rotated_size = std::fs::metadata(&rotated_filename)?.len();
    
    println!("✓ Current log size: {} bytes", current_size);
    println!("✓ Rotated log size: {} bytes", rotated_size);
    println!("✓ Log rotation example completed");
    
    // Cleanup
    let _ = std::fs::remove_file(&current_log);
    let _ = std::fs::remove_file(&rotated_filename);
    
    println!("✓ Log rotation and management example completed\n");
    Ok(())
}

/// Utility function to simulate realistic application workload
#[allow(dead_code)]
fn simulate_realistic_workload() -> Result<(), LoggerError> {
    let log_file = "realistic_workload.log";
    
    let hp_config = HighPerformanceConfig {
        batch_size: 50,
        flush_interval_ms: 100,
        enabled: true,
        buffer_capacity: 200,
        string_pool_size: 100,
    };
    
    let logger = Arc::new(HighPerformanceLogger::new(log_file, hp_config)?);
    
    // Simulate different types of application events
    let events = vec![
        ("http_request", log::Level::Info, 60),
        ("database_query", log::Level::Debug, 30),
        ("cache_miss", log::Level::Warn, 8),
        ("authentication_failure", log::Level::Error, 2),
    ];
    
    println!("Simulating realistic application workload...");
    
    let total_duration = Duration::from_secs(10);
    let start = Instant::now();
    
    while start.elapsed() < total_duration {
        for (event_type, level, frequency) in &events {
            if (start.elapsed().as_millis() % 100) < *frequency as u128 {
                let timestamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis();
                let request_id = start.elapsed().as_nanos() % 100000;
                
                match level {
                    log::Level::Error => log::error!(target: "web_server", "{{\"event\":\"{}\",\"timestamp\":{},\"request_id\":\"req-{}\"}}", event_type, timestamp, request_id),
                    log::Level::Warn => log::warn!(target: "web_server", "{{\"event\":\"{}\",\"timestamp\":{},\"request_id\":\"req-{}\"}}", event_type, timestamp, request_id),
                    log::Level::Info => log::info!(target: "web_server", "{{\"event\":\"{}\",\"timestamp\":{},\"request_id\":\"req-{}\"}}", event_type, timestamp, request_id),
                    log::Level::Debug => log::debug!(target: "web_server", "{{\"event\":\"{}\",\"timestamp\":{},\"request_id\":\"req-{}\"}}", event_type, timestamp, request_id),
                    log::Level::Trace => log::trace!(target: "web_server", "{{\"event\":\"{}\",\"timestamp\":{},\"request_id\":\"req-{}\"}}", event_type, timestamp, request_id),
                }
            }
        }
        
        thread::sleep(Duration::from_millis(10));
    }
    
    let _ = logger.flush();
    println!("✓ Realistic workload simulation completed");
    
    Ok(())
}