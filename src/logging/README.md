# Artifice Logging Library

A high-performance, modular logging library for Rust applications with advanced batching and memory optimization capabilities.

## Features

🚀 **High Performance**
- Advanced memory layouts with Structure of Arrays for optimal cache locality
- Batch processing to minimize I/O overhead
- String pooling for memory allocation optimization
- Asynchronous file writing with background threads

⚡ **Optimized Memory Management**
- Pre-allocated buffers to avoid runtime allocations
- High-performance writer for maximum throughput scenarios
- Efficient bulk operations reducing system calls from N to 1
- Memory-conscious configurations for resource-constrained environments

🔧 **Flexible Configuration**
- Console and file output support
- Colored terminal output
- Environment variable configuration
- Builder pattern for fluent setup
- Standard and high-performance modes

📊 **Production Ready**
- Concurrent logging support
- Graceful error handling and recovery
- Modular architecture with separate concerns
- Performance profiling and benchmarking utilities
- Comprehensive test suite

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
artifice-logging = { path = "path/to/artifice-engine/src/logging" }
log = "0.4"
```

### Basic Console Logging

```rust
use artifice_logging::*;

fn main() -> Result<(), LoggerError> {
    init()?;
    
    log::info!("Hello, world!");
    log::debug!("Debug information");
    log::error!("Something went wrong: {}", "file not found");
    
    Ok(())
}
```

### File Logging

```rust
use artifice_logging::*;

fn main() -> Result<(), LoggerError> {
    let config = LogConfig::default();
    init_with_file("app.log", config)?;
    
    log::info!("This will be written to app.log");
    log::warn!("And also displayed on console");
    
    flush(); // Force immediate write
    
    Ok(())
}
```

### High-Performance Logging

```rust
use artifice_logging::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = HighPerformanceConfig {
        batch_size: 200,
        flush_interval_ms: 25,
        enabled: true,
        buffer_capacity: 1000,
        string_pool_size: 500,
    };
    
    let logger = HighPerformanceLogger::new("high_perf.log", config)?;
    
    // Log pre-formatted messages for maximum performance
    for i in 0..1000 {
        let message = format!("High-performance message {}: data={}", i, i * 2);
        logger.log_fast(message)?;
    }
    
    logger.flush()?;
    
    Ok(())
}
```

### Builder Pattern

```rust
use artifice_logging::*;

fn main() -> Result<(), LoggerError> {
    LoggerBuilder::new()
        .console(true)
        .file("builder_example.log")
        .colors(true)
        .batch_size(50)
        .flush_interval_ms(100)
        .init()?;
    
    log::info!("Builder pattern makes configuration clean");
    
    Ok(())
}
```

## Architecture

The library is organized into several modules for clean separation of concerns:

- **`config`** - Configuration structs and enums
- **`batching`** - Batch processing and message structures  
- **`writers`** - File writers (standard and high-performance)
- **`benchmarks`** - Performance testing utilities

## Performance Characteristics

### Standard Mode
- **Throughput**: ~50,000 messages/second
- **Memory**: Moderate allocation with standard batching
- **Use case**: General application logging

### High-Performance Mode  
- **Throughput**: ~150,000+ messages/second
- **Memory**: Optimized with string pooling and SoA layout
- **Use case**: High-frequency logging, performance-critical applications

## Configuration Reference

### LogConfig
- `console: bool` - Enable console output
- `file: bool` - Enable file output  
- `colors: bool` - Enable colored console output

### BatchConfig
- `batch_size: usize` - Messages to buffer before writing (default: 50)
- `flush_interval_ms: u64` - Maximum time before forced flush (default: 100ms)
- `enabled: bool` - Enable/disable batching (default: true)
- `buffer_capacity: usize` - Pre-allocated buffer size (default: 256)
- `string_pool_size: usize` - String pool size for reuse (default: 128)

### HighPerformanceConfig
- `batch_size: usize` - Large batch sizes (default: 100)
- `flush_interval_ms: u64` - Flush intervals (default: 100ms)
- `enabled: bool` - Batching enabled by default
- `buffer_capacity: usize` - Large pre-allocated buffers (default: 1024)
- `string_pool_size: usize` - Large string pool (default: 512)

## Environment Variables

Configure the logger using environment variables:

```bash
export ARTIFICE_LOG_FILE="app.log"
export ARTIFICE_LOG_CONSOLE="true" 
export ARTIFICE_LOG_COLORS="true"
export ARTIFICE_LOG_BATCH_SIZE="100"
export ARTIFICE_LOG_FLUSH_INTERVAL="50"
export ARTIFICE_LOG_BATCHING="true"
```

Then initialize with:

```rust
use artifice_logging::*;

fn main() -> Result<(), LoggerError> {
    init_from_env()?;
    log::info!("Configuration loaded from environment");
    Ok(())
}
```

## Examples

Run the examples to see the library in action:

```bash
cd src/logging
cargo run --example basic_usage
cargo run --example advanced_usage
cargo run --example working_example
```

## Benchmarks

The library includes comprehensive benchmarking utilities:

```rust
use artifice_logging::benchmarks::LoggingBenchmarks;

// Run all performance benchmarks
LoggingBenchmarks::run_all_benchmarks();

// Or run specific benchmarks
LoggingBenchmarks::benchmark_batch_sizes();
LoggingBenchmarks::benchmark_memory_patterns();
```

## Testing

Run the complete test suite:

```bash
cargo test
cargo check --examples
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. 

### Development Setup

1. Clone the repository
2. Navigate to the logging library: `cd src/logging`
3. Run tests: `cargo test`
4. Run examples: `cargo run --example basic_usage`
5. Run benchmarks: `cargo run --example advanced_usage`

### Code Guidelines

- Follow Rust naming conventions
- Add tests for new functionality
- Update documentation and examples
- Run `cargo fmt` and `cargo clippy` before submitting

## Performance Notes

- Use `HighPerformanceLogger` for maximum throughput scenarios
- Adjust batch sizes based on your application's needs
- Monitor memory usage with large string pools
- Use environment variables for deployment-specific configuration

## License

This project is licensed under the MIT License - see the LICENSE file for details.