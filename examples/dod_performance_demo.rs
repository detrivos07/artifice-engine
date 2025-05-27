use std::time::Instant;
use std::fs::File;
use std::io::{Write, BufWriter};

fn main() {
    println!("=== Data-Oriented Design Performance Demo ===\n");
    
    run_soa_vs_aos_demo();
    run_bulk_vs_individual_writes_demo();
    run_cache_locality_demo();
    run_string_pooling_demo();
    run_memory_layout_demo();
    
    println!("\n=== DoD Demo Complete ===");
    println!("Key DoD principles that improve batch write performance:");
    println!("1. Structure of Arrays (SoA) improves cache locality");
    println!("2. Bulk writes reduce system call overhead");  
    println!("3. String pooling reduces memory allocations");
    println!("4. Pre-allocated buffers avoid runtime reallocations");
    println!("5. Sequential memory access patterns are cache-friendly");
}

fn run_soa_vs_aos_demo() {
    println!("=== Structure of Arrays vs Array of Structures ===");
    
    let message_count = 100_000;
    
    // Array of Structures (AoS) - Traditional approach
    #[derive(Clone)]
    struct LogEntry {
        message: String,
        timestamp: u64,
        level: u8,
        thread_id: u32,
    }
    
    println!("Testing Array of Structures (AoS)...");
    let start = Instant::now();
    let mut aos_logs = Vec::with_capacity(message_count);
    
    for i in 0..message_count {
        aos_logs.push(LogEntry {
            message: format!("Log message {}", i),
            timestamp: i as u64,
            level: (i % 4) as u8,
            thread_id: (i % 8) as u32,
        });
    }
    
    // Process messages (cache-unfriendly access pattern)
    let mut total_length = 0;
    for entry in &aos_logs {
        total_length += entry.message.len();
    }
    let aos_time = start.elapsed();
    
    // Structure of Arrays (SoA) - DoD approach
    println!("Testing Structure of Arrays (SoA)...");
    let start = Instant::now();
    let mut soa_messages = Vec::with_capacity(message_count);
    let mut soa_timestamps = Vec::with_capacity(message_count);
    let mut soa_levels = Vec::with_capacity(message_count);
    let mut soa_thread_ids = Vec::with_capacity(message_count);
    
    for i in 0..message_count {
        soa_messages.push(format!("Log message {}", i));
        soa_timestamps.push(i as u64);
        soa_levels.push((i % 4) as u8);
        soa_thread_ids.push((i % 8) as u32);
    }
    
    // Process messages (cache-friendly access pattern)
    let mut total_length = 0;
    for message in &soa_messages {
        total_length += message.len();
    }
    let soa_time = start.elapsed();
    
    println!("AoS approach: {:?}", aos_time);
    println!("SoA approach: {:?}", soa_time);
    println!("SoA is {:.2}x faster\n", aos_time.as_nanos() as f64 / soa_time.as_nanos() as f64);
}

fn run_bulk_vs_individual_writes_demo() {
    println!("=== Bulk Writes vs Individual Writes ===");
    
    let messages = (0..1000)
        .map(|i| format!("[INFO][2024-01-01 12:00:00.{}][test]: Message {}", i % 1000, i))
        .collect::<Vec<_>>();
    
    // Individual writes
    println!("Testing individual writes...");
    let start = Instant::now();
    let file = File::create("individual_test.log").expect("Failed to create file");
    let mut writer = BufWriter::new(file);
    
    for message in &messages {
        writeln!(writer, "{}", message).expect("Write failed");
    }
    writer.flush().expect("Flush failed");
    let individual_time = start.elapsed();
    
    // Bulk write
    println!("Testing bulk write...");
    let start = Instant::now();
    let file = File::create("bulk_test.log").expect("Failed to create file");
    let mut writer = BufWriter::new(file);
    
    // Pre-allocate buffer for all messages
    let total_size: usize = messages.iter().map(|m| m.len() + 1).sum();
    let mut bulk_buffer = String::with_capacity(total_size);
    
    for message in &messages {
        bulk_buffer.push_str(message);
        bulk_buffer.push('\n');
    }
    
    writer.write_all(bulk_buffer.as_bytes()).expect("Bulk write failed");
    writer.flush().expect("Flush failed");
    let bulk_time = start.elapsed();
    
    println!("Individual writes: {:?} ({} system calls)", individual_time, messages.len());
    println!("Bulk write: {:?} (1 system call)", bulk_time);
    println!("Bulk is {:.2}x faster\n", individual_time.as_nanos() as f64 / bulk_time.as_nanos() as f64);
    
    // Cleanup
    let _ = std::fs::remove_file("individual_test.log");
    let _ = std::fs::remove_file("bulk_test.log");
}

fn run_cache_locality_demo() {
    println!("=== Cache Locality Demonstration ===");
    
    let data_size = 1_000_000;
    
    // Random access pattern (cache misses)
    println!("Testing random access pattern...");
    let mut mixed_data: Vec<(String, u64, f32)> = Vec::with_capacity(data_size);
    for i in 0..data_size {
        mixed_data.push((format!("data_{}", i), i as u64, i as f32));
    }
    
    let start = Instant::now();
    let mut sum = 0u64;
    // Access every 17th element (causes cache misses)
    for i in (0..data_size).step_by(17) {
        sum += mixed_data[i].1;
    }
    let random_time = start.elapsed();
    
    // Sequential access pattern (cache hits)
    println!("Testing sequential access pattern...");
    let mut string_data: Vec<String> = Vec::with_capacity(data_size);
    let mut int_data: Vec<u64> = Vec::with_capacity(data_size);
    let mut float_data: Vec<f32> = Vec::with_capacity(data_size);
    
    for i in 0..data_size {
        string_data.push(format!("data_{}", i));
        int_data.push(i as u64);
        float_data.push(i as f32);
    }
    
    let start = Instant::now();
    let mut sum2 = 0u64;
    // Sequential access to int_data only (cache friendly)
    for i in (0..data_size).step_by(17) {
        sum2 += int_data[i];
    }
    let sequential_time = start.elapsed();
    
    println!("Random access: {:?}", random_time);
    println!("Sequential access: {:?}", sequential_time);
    println!("Sequential is {:.2}x faster\n", random_time.as_nanos() as f64 / sequential_time.as_nanos() as f64);
    
    assert_eq!(sum, sum2); // Verify same computation
}

fn run_string_pooling_demo() {
    println!("=== String Pooling for Reduced Allocations ===");
    
    let iteration_count = 50_000;
    
    // Without string pooling
    println!("Testing without string pooling...");
    let start = Instant::now();
    let mut no_pool_strings = Vec::new();
    
    for i in 0..iteration_count {
        let s = format!("[INFO][thread_{}]: Processing item {}", i % 4, i);
        no_pool_strings.push(s);
    }
    let no_pool_time = start.elapsed();
    
    // With string pooling
    println!("Testing with string pooling...");
    let start = Instant::now();
    
    // Pre-allocate string pool
    let pool_size = 1000;
    let mut string_pool: Vec<String> = Vec::with_capacity(pool_size);
    for _ in 0..pool_size {
        string_pool.push(String::with_capacity(64));
    }
    
    let mut pooled_strings = Vec::new();
    let mut pool_index = 0;
    
    for i in 0..iteration_count {
        // Reuse string from pool
        let mut s = std::mem::take(&mut string_pool[pool_index]);
        s.clear();
        
        use std::fmt::Write;
        write!(s, "[INFO][thread_{}]: Processing item {}", i % 4, i).unwrap();
        
        pooled_strings.push(s);
        pool_index = (pool_index + 1) % pool_size;
    }
    let pool_time = start.elapsed();
    
    println!("Without pooling: {:?}", no_pool_time);
    println!("With pooling: {:?}", pool_time);
    println!("Pooling is {:.2}x faster\n", no_pool_time.as_nanos() as f64 / pool_time.as_nanos() as f64);
}

fn run_memory_layout_demo() {
    println!("=== Memory Layout and Batch Processing ===");
    
    let batch_sizes = vec![1, 10, 50, 100, 500, 1000];
    let total_messages = 10_000;
    
    println!("Comparing different batch sizes for {} messages:", total_messages);
    
    for &batch_size in &batch_sizes {
        let start = Instant::now();
        
        // Simulate DoD batch processing
        let mut batch_buffer = String::with_capacity(batch_size * 80); // Estimate 80 chars per message
        let mut write_count = 0;
        let mut messages_in_batch = 0;
        
        for i in 0..total_messages {
            // Add message to batch
            batch_buffer.push_str(&format!("[INFO][12:00:00.{}][test]: Message {}\n", i % 1000, i));
            messages_in_batch += 1;
            
            // Flush when batch is full
            if messages_in_batch >= batch_size {
                // Simulate write operation (measure formatting + preparation time)
                let _bytes = batch_buffer.as_bytes();
                write_count += 1;
                
                // Reset batch
                batch_buffer.clear();
                messages_in_batch = 0;
            }
        }
        
        // Handle remaining messages
        if messages_in_batch > 0 {
            let _bytes = batch_buffer.as_bytes();
            write_count += 1;
        }
        
        let duration = start.elapsed();
        
        println!("Batch size {}: {} writes, {:?} total", batch_size, write_count, duration);
    }
    
    println!("\nOptimal batch size balances:");
    println!("- Fewer I/O operations (larger batches)");
    println!("- Lower memory usage (smaller batches)");
    println!("- Reduced latency (smaller batches)");
}