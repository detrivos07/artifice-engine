use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use crate::batching::{LogBatch, LogCommand, AdvancedLogCommand, LogMessage, SimpleBatch};
use crate::config::{BatchConfig, HighPerformanceConfig};

/// Standard file writer with basic batching capabilities
pub struct FileWriter {
    writer: BufWriter<File>,
    batch: SimpleBatch,
    batch_config: BatchConfig,
    last_flush: Instant,
}

impl FileWriter {
    pub fn new(file: File, batch_config: BatchConfig) -> Self {
        let writer = BufWriter::with_capacity(8 * 1024, file); // 8KB buffer
        let batch = SimpleBatch::new(batch_config.buffer_capacity);
        
        Self {
            writer,
            batch,
            batch_config,
            last_flush: Instant::now(),
        }
    }
    
    fn should_flush(&self) -> bool {
        if !self.batch_config.enabled {
            return true;
        }
        
        self.batch.len() >= self.batch_config.batch_size ||
        self.last_flush.elapsed() >= Duration::from_millis(self.batch_config.flush_interval_ms)
    }
    
    pub fn add_message(&mut self, message: LogMessage) -> io::Result<()> {
        if !self.batch_config.enabled {
            // Immediate write for non-batched mode
            writeln!(self.writer, "{}", message.formatted_message)?;
            return self.writer.flush();
        }
        
        self.batch.push(message);
        
        if self.should_flush() {
            self.flush()?;
        }
        
        Ok(())
    }
    
    pub fn flush(&mut self) -> io::Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }
        
        for message in self.batch.messages() {
            writeln!(self.writer, "{}", message.formatted_message)?;
        }
        
        self.writer.flush()?;
        self.batch.clear();
        self.last_flush = Instant::now();
        
        Ok(())
    }
    
    pub fn shutdown(&mut self) -> io::Result<()> {
        self.flush()
    }
}

/// High-performance file writer optimized for maximum throughput
/// 
/// This writer uses advanced techniques including:
/// - Structure of Arrays (SoA) for better cache locality
/// - String pooling to reduce allocations
/// - Bulk write operations to minimize system calls
/// - Pre-allocated buffers to avoid runtime allocations
pub struct HighPerformanceFileWriter {
    writer: BufWriter<File>,
    batch: LogBatch,
    config: HighPerformanceConfig,
    last_flush: Instant,
    
    // String pool for reusing allocations
    string_pool: Vec<String>,
    pool_index: usize,
}

impl HighPerformanceFileWriter {
    pub fn new(file: File, config: HighPerformanceConfig) -> Self {
        let writer = BufWriter::with_capacity(64 * 1024, file); // 64KB buffer
        let batch = LogBatch::new(config.buffer_capacity);
        
        // Initialize string pool
        let mut string_pool = Vec::with_capacity(config.string_pool_size);
        for _ in 0..config.string_pool_size {
            string_pool.push(String::with_capacity(256)); // Pre-allocate reasonable size
        }
        
        Self {
            writer,
            batch,
            config,
            last_flush: Instant::now(),
            string_pool,
            pool_index: 0,
        }
    }
    
    /// Get a string from the pool to reduce allocations
    #[allow(dead_code)]
    fn get_pooled_string(&mut self) -> String {
        if self.pool_index < self.string_pool.len() {
            let mut s = std::mem::take(&mut self.string_pool[self.pool_index]);
            s.clear();
            self.pool_index += 1;
            s
        } else {
            String::with_capacity(256)
        }
    }
    
    /// Return strings to the pool for reuse
    fn reset_string_pool(&mut self) {
        self.pool_index = 0;
    }
    
    fn should_flush(&self) -> bool {
        if !self.config.enabled {
            return true;
        }
        
        self.batch.len() >= self.config.batch_size ||
        self.last_flush.elapsed() >= Duration::from_millis(self.config.flush_interval_ms)
    }
    
    pub fn add_message(&mut self, message: String) -> io::Result<()> {
        if !self.config.enabled {
            // Immediate write for non-batched mode
            writeln!(self.writer, "{}", message)?;
            return self.writer.flush();
        }
        
        let timestamp = Instant::now();
        self.batch.push(message, timestamp);
        
        if self.should_flush() {
            self.flush()?;
        }
        
        Ok(())
    }
    
    /// High-performance bulk flush operation
    pub fn flush(&mut self) -> io::Result<()> {
        if self.batch.is_empty() {
            return Ok(());
        }
        
        // Single bulk write operation instead of multiple writes
        let bulk_content = self.batch.format_bulk();
        self.writer.write_all(bulk_content.as_bytes())?;
        self.writer.flush()?;
        
        // Reset batch and string pool
        self.batch.clear();
        self.reset_string_pool();
        self.last_flush = Instant::now();
        
        Ok(())
    }
    
    pub fn shutdown(&mut self) -> io::Result<()> {
        self.flush()
    }
}

/// Standard file worker thread for regular performance requirements
pub fn file_worker_thread(
    mut file_writer: FileWriter,
    receiver: mpsc::Receiver<LogCommand>,
) {
    let mut should_shutdown = false;
    
    while !should_shutdown {
        let timeout = Duration::from_millis(file_writer.batch_config.flush_interval_ms);
        
        match receiver.recv_timeout(timeout) {
            Ok(command) => {
                match command {
                    LogCommand::Message(message) => {
                        if let Err(e) = file_writer.add_message(message) {
                            eprintln!("Failed to write log message: {}", e);
                        }
                    }
                    LogCommand::Flush => {
                        if let Err(e) = file_writer.flush() {
                            eprintln!("Failed to flush log messages: {}", e);
                        }
                    }
                    LogCommand::Shutdown => {
                        should_shutdown = true;
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Periodic flush check
                if file_writer.should_flush() {
                    if let Err(e) = file_writer.flush() {
                        eprintln!("Failed to periodic flush log messages: {}", e);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                should_shutdown = true;
            }
        }
    }
    
    // Final flush on shutdown
    if let Err(e) = file_writer.shutdown() {
        eprintln!("Failed to shutdown file writer: {}", e);
    }
}

/// High-performance worker thread optimized for maximum throughput
pub fn high_performance_worker_thread(
    mut file_writer: HighPerformanceFileWriter,
    receiver: mpsc::Receiver<AdvancedLogCommand>,
) {
    let mut should_shutdown = false;
    
    // Message batch for receiving multiple messages at once
    let mut command_batch = Vec::with_capacity(100);
    
    while !should_shutdown {
        let timeout = Duration::from_millis(file_writer.config.flush_interval_ms / 2);
        
        // Try to receive first message
        match receiver.recv_timeout(timeout) {
            Ok(command) => {
                command_batch.push(command);
                
                // Try to receive more messages without blocking (burst handling)
                while let Ok(cmd) = receiver.try_recv() {
                    command_batch.push(cmd);
                    if command_batch.len() >= 50 { // Limit batch size
                        break;
                    }
                }
                
                // Process all commands in batch
                for cmd in command_batch.drain(..) {
                    match cmd {
                        AdvancedLogCommand::Message(message) => {
                            if let Err(e) = file_writer.add_message(message) {
                                eprintln!("Failed to write log message: {}", e);
                            }
                        }
                        AdvancedLogCommand::Flush => {
                            if let Err(e) = file_writer.flush() {
                                eprintln!("Failed to flush log messages: {}", e);
                            }
                        }
                        AdvancedLogCommand::Shutdown => {
                            should_shutdown = true;
                            break;
                        }
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // Periodic flush check
                if file_writer.should_flush() {
                    if let Err(e) = file_writer.flush() {
                        eprintln!("Failed to periodic flush log messages: {}", e);
                    }
                }
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                should_shutdown = true;
            }
        }
    }
    
    // Final flush on shutdown
    if let Err(e) = file_writer.shutdown() {
        eprintln!("Failed to shutdown file writer: {}", e);
    }
}