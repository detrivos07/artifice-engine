use std::time::Instant;

/// Internal log message structure
#[derive(Debug, Clone)]
pub struct LogMessage {
    pub formatted_message: String,
    pub timestamp: Instant,
}

/// Commands for controlling the logging worker thread
#[derive(Debug)]
pub enum LogCommand {
    /// Log a message
    Message(LogMessage),
    /// Force flush all pending messages
    Flush,
    /// Shutdown the worker thread
    Shutdown,
}

/// High-performance commands for the advanced writer
#[derive(Debug)]
pub enum AdvancedLogCommand {
    /// Pre-formatted message for high-performance logging
    Message(String),
    /// Force flush all pending messages
    Flush,
    /// Shutdown the worker thread
    Shutdown,
}

/// Structure of Arrays (SoA) batch for better cache locality
/// 
/// Instead of storing Vec<LogMessage>, we separate different data types
/// into their own vectors for better CPU cache utilization.
pub struct LogBatch {
    /// Message strings stored separately
    pub messages: Vec<String>,
    /// Timestamps stored separately
    pub timestamps: Vec<Instant>,
    /// Pre-allocated capacity to avoid reallocations
    pub capacity: usize,
    /// Bulk write buffer - reused across flushes
    pub write_buffer: String,
}

impl LogBatch {
    /// Create a new batch with the specified capacity
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: Vec::with_capacity(capacity),
            timestamps: Vec::with_capacity(capacity),
            capacity,
            // Pre-allocate write buffer - estimate ~100 chars per message
            write_buffer: String::with_capacity(capacity * 100),
        }
    }
    
    /// Add a message to the batch
    pub fn push(&mut self, message: String, timestamp: Instant) {
        self.messages.push(message);
        self.timestamps.push(timestamp);
    }
    
    /// Get the number of messages in the batch
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    /// Check if the batch is empty
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    /// Clear the batch while maintaining capacity
    pub fn clear(&mut self) {
        self.messages.clear();
        self.timestamps.clear();
        self.write_buffer.clear();
        
        // Maintain capacity to avoid future allocations
        self.messages.reserve(self.capacity.saturating_sub(self.messages.capacity()));
        self.timestamps.reserve(self.capacity.saturating_sub(self.timestamps.capacity()));
    }
    
    /// Bulk format all messages into a single string buffer
    /// This reduces the number of system calls from N to 1
    pub fn format_bulk(&mut self) -> &str {
        self.write_buffer.clear();
        
        // Reserve space to avoid reallocations during formatting
        let estimated_size = self.messages.iter()
            .map(|msg| msg.len() + 1) // +1 for newline
            .sum::<usize>();
        self.write_buffer.reserve(estimated_size);
        
        // Sequential memory access pattern - cache friendly
        for message in &self.messages {
            self.write_buffer.push_str(message);
            self.write_buffer.push('\n');
        }
        
        &self.write_buffer
    }
}

/// Simple batch implementation for standard performance requirements
pub struct SimpleBatch {
    messages: Vec<LogMessage>,
    capacity: usize,
}

impl SimpleBatch {
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: Vec::with_capacity(capacity),
            capacity,
        }
    }
    
    pub fn push(&mut self, message: LogMessage) {
        self.messages.push(message);
    }
    
    pub fn len(&self) -> usize {
        self.messages.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.messages.is_empty()
    }
    
    pub fn clear(&mut self) {
        self.messages.clear();
        self.messages.reserve(self.capacity.saturating_sub(self.messages.capacity()));
    }
    
    pub fn messages(&self) -> &[LogMessage] {
        &self.messages
    }
}