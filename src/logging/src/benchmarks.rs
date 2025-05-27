use std::time::Instant;
use crate::batching::LogBatch;

/// Performance benchmarking utilities for the logging system
pub struct LoggingBenchmarks;

impl LoggingBenchmarks {
    /// Benchmark different batch sizes to find optimal configuration
    pub fn benchmark_batch_sizes() {
        println!("=== Batch Size Performance Benchmarks ===");
        
        let batch_sizes = vec![1, 10, 50, 100, 500, 1000, 2000];
        let message_count = 10000;
        let test_message = "This is a test log message for performance benchmarking".to_string();
        
        for &batch_size in &batch_sizes {
            let start = Instant::now();
            
            // Simulate batching
            let mut batch = LogBatch::new(batch_size);
            let mut write_operations = 0;
            
            for _i in 0..message_count {
                batch.push(test_message.clone(), Instant::now());
                
                if batch.len() >= batch_size {
                    // Simulate bulk write
                    let _bulk_content = batch.format_bulk();
                    write_operations += 1;
                    batch.clear();
                }
            }
            
            // Handle remaining messages
            if !batch.is_empty() {
                let _bulk_content = batch.format_bulk();
                write_operations += 1;
            }
            
            let duration = start.elapsed();
            
            println!("Batch size {}: {} write operations, {:?} total time",
                     batch_size, write_operations, duration);
            println!("  Avg per message: {:.2} μs", 
                     duration.as_micros() as f64 / message_count as f64);
        }
    }
    
    /// Compare Structure of Arrays vs Array of Structures memory patterns
    pub fn benchmark_memory_patterns() {
        println!("=== Memory Pattern Performance Comparison ===");
        
        let message_count = 10000;
        let test_message = "Test message for memory pattern analysis".to_string();
        
        // Test SoA vs AoS patterns
        println!("Testing Structure of Arrays (SoA) vs Array of Structures (AoS)");
        
        // SoA test - separate vectors for each data type
        let start = Instant::now();
        let mut messages = Vec::with_capacity(message_count);
        let mut timestamps = Vec::with_capacity(message_count);
        
        for _i in 0..message_count {
            messages.push(test_message.clone());
            timestamps.push(Instant::now());
        }
        
        // Sequential access - cache friendly
        let mut _total_length = 0;
        for msg in &messages {
            _total_length += msg.len();
        }
        
        let soa_duration = start.elapsed();
        
        // AoS test - traditional approach with structs
        #[derive(Clone)]
        struct TraditionalLogMessage {
            message: String,
            timestamp: Instant,
        }
        
        let start = Instant::now();
        let mut traditional_messages = Vec::with_capacity(message_count);
        
        for _i in 0..message_count {
            traditional_messages.push(TraditionalLogMessage {
                message: test_message.clone(),
                timestamp: Instant::now(),
            });
        }
        
        let mut _total_length = 0;
        let mut _timestamp_count = 0usize;
        for msg in &traditional_messages {
            _total_length += msg.message.len();
            _timestamp_count += msg.timestamp.elapsed().as_nanos() as usize;
        }
        
        let aos_duration = start.elapsed();
        
        println!("SoA pattern: {:?}", soa_duration);
        println!("AoS pattern: {:?}", aos_duration);
        if aos_duration.as_nanos() > 0 && soa_duration.as_nanos() > 0 {
            println!("SoA is {:.2}x faster", aos_duration.as_nanos() as f64 / soa_duration.as_nanos() as f64);
        }
    }
    
    /// Benchmark string allocation patterns
    pub fn benchmark_string_pooling() {
        println!("=== String Pooling Performance Benchmark ===");
        
        let iterations = 10000;
        let message_template = "Log message number: ";
        
        // Test without string pooling
        let start = Instant::now();
        for i in 0..iterations {
            let _message = format!("{}{}", message_template, i);
        }
        let no_pooling_duration = start.elapsed();
        
        // Test with string pooling simulation
        let start = Instant::now();
        let mut string_pool = Vec::with_capacity(100);
        for _ in 0..100 {
            string_pool.push(String::with_capacity(64));
        }
        
        let mut pool_index = 0;
        for i in 0..iterations {
            let mut s = if pool_index < string_pool.len() {
                std::mem::take(&mut string_pool[pool_index])
            } else {
                String::with_capacity(64)
            };
            s.clear();
            s.push_str(message_template);
            s.push_str(&i.to_string());
            
            // Return to pool
            if pool_index < string_pool.len() {
                string_pool[pool_index] = s;
                pool_index += 1;
            }
            
            if pool_index >= string_pool.len() {
                pool_index = 0;
            }
        }
        let pooling_duration = start.elapsed();
        
        println!("Without pooling: {:?}", no_pooling_duration);
        println!("With pooling: {:?}", pooling_duration);
        if no_pooling_duration.as_nanos() > 0 && pooling_duration.as_nanos() > 0 {
            println!("Pooling is {:.2}x faster", 
                     no_pooling_duration.as_nanos() as f64 / pooling_duration.as_nanos() as f64);
        }
    }
    
    /// Comprehensive performance test suite
    pub fn run_all_benchmarks() {
        println!("Running comprehensive logging performance benchmarks...\n");
        
        Self::benchmark_batch_sizes();
        println!();
        
        Self::benchmark_memory_patterns();
        println!();
        
        Self::benchmark_string_pooling();
        println!();
        
        println!("Benchmark suite completed.");
    }
}

/// Utility for measuring operation throughput
pub struct ThroughputMeter {
    start_time: Instant,
    operation_count: usize,
}

impl ThroughputMeter {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            operation_count: 0,
        }
    }
    
    pub fn record_operation(&mut self) {
        self.operation_count += 1;
    }
    
    pub fn record_operations(&mut self, count: usize) {
        self.operation_count += count;
    }
    
    pub fn throughput_per_second(&self) -> f64 {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            self.operation_count as f64 / elapsed
        } else {
            0.0
        }
    }
    
    pub fn report(&self, operation_name: &str) {
        let throughput = self.throughput_per_second();
        let elapsed = self.start_time.elapsed();
        
        println!("{}: {} operations in {:?} ({:.2} ops/sec)", 
                 operation_name, self.operation_count, elapsed, throughput);
    }
}

impl Default for ThroughputMeter {
    fn default() -> Self {
        Self::new()
    }
}