# Logging System Examples

This directory contains comprehensive examples demonstrating the Artifice Engine logging system capabilities.

## Overview

The Artifice Engine includes a flexible logging system that supports:
- Console and file output (simultaneously or separately)
- Colored console output
- Log level filtering
- Thread-safe operation
- Environment variable configuration
- Builder pattern for easy configuration

## Examples

### 1. Comprehensive Demo (`logging_demo.rs`)

**Purpose**: Demonstrates all major logging features in a single comprehensive example.

**Features Shown**:
- Basic logging at all levels (ERROR, WARN, INFO, DEBUG, TRACE)
- Log level filtering
- Multi-threaded logging
- Structured logging with context
- Error handling patterns
- Performance logging
- Builder pattern usage examples

**Run**: `cargo run --example logging_demo`

**Output**: Creates `comprehensive_demo.log` with all log messages (no colors in file)

### 2. Console-Only Demo (`console_only_demo.rs`)

**Purpose**: Shows console-only logging with colored output.

**Features Shown**:
- Console output with colors
- Log level filtering
- Rapid logging sequences
- Structured data logging

**Run**: `cargo run --example console_only_demo`

**Output**: Colored console output only, no files created

### 3. File-Only Demo (`file_only_demo.rs`)

**Purpose**: Demonstrates file-only logging (no console output).

**Features Shown**:
- File-only output
- Application startup/shutdown logging patterns
- Error scenario logging
- Performance metrics logging
- Structured request logging

**Run**: `cargo run --example file_only_demo`

**Output**: Creates `file_only_demo.log`, no console log output

### 4. Environment Variable Demo (`env_demo.rs`)

**Purpose**: Shows how to configure logging using environment variables.

**Features Shown**:
- Environment variable detection
- Dynamic configuration from environment
- All standard logging patterns

**Run**: `cargo run --example env_demo`

**Environment Variables Supported**:
- `ARTIFICE_LOG_LEVEL`: Set log level (ERROR, WARN, INFO, DEBUG, TRACE)
- `ARTIFICE_LOG_CONSOLE`: Enable/disable console output (true/false)
- `ARTIFICE_LOG_COLORS`: Enable/disable colors (true/false)  
- `ARTIFICE_LOG_FILE`: Specify log file path

## Usage Examples

### Basic Console Logging
```bash
cargo run --example console_only_demo
```

### File-Only Logging
```bash
cargo run --example file_only_demo
```

### Environment Variable Configuration

**Debug level with file output**:
```bash
ARTIFICE_LOG_LEVEL=DEBUG ARTIFICE_LOG_FILE=debug.log cargo run --example env_demo
```

**Error level only, no colors**:
```bash
ARTIFICE_LOG_LEVEL=ERROR ARTIFICE_LOG_COLORS=false cargo run --example env_demo
```

**File only (no console output)**:
```bash
ARTIFICE_LOG_CONSOLE=false ARTIFICE_LOG_FILE=silent.log cargo run --example env_demo
```

**Info level with both console and file**:
```bash
ARTIFICE_LOG_LEVEL=INFO ARTIFICE_LOG_FILE=app.log cargo run --example env_demo
```

## Generated Files

After running the examples, you'll find these log files:
- `comprehensive_demo.log` - Complete logging demo output
- `file_only_demo.log` - File-only demo output  
- `env_test.log` (or custom name) - Environment variable demo output

## Key Features Demonstrated

### Log Levels
- **ERROR**: Critical errors that need immediate attention
- **WARN**: Warning conditions that should be noted
- **INFO**: General informational messages
- **DEBUG**: Detailed information for debugging
- **TRACE**: Very detailed trace information

### Output Formats
- **Console**: Colored output with timestamps and module information
- **File**: Plain text with timestamps (no color codes)

### Configuration Patterns
- **Builder Pattern**: `LoggerBuilder::new().console(true).file("app.log").init()`
- **Direct Initialization**: `logging::init()` or `logging::init_with_file()`
- **Environment Variables**: `logging::init_from_env()`

### Thread Safety
All examples demonstrate that the logging system is thread-safe and can handle concurrent logging from multiple threads without issues.

## Integration in Your Application

To use the logging system in your application:

1. **Simple setup**:
   ```rust
   logging::init().expect("Failed to initialize logger");
   ```

2. **With file output**:
   ```rust
   LoggerBuilder::new()
       .console(true)
       .file("app.log")
       .colors(true)
       .init()
       .expect("Failed to initialize logger");
   ```

3. **From environment**:
   ```rust
   logging::init_from_env().expect("Failed to initialize logger");
   ```

4. **Use logging macros**:
   ```rust
   use logging::{error, warn, info, debug, trace};
   
   info!("Application started");
   debug!("Processing user_id: {}", user_id);
   error!("Failed to connect: {}", error_msg);
   ```