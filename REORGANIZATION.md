# Artifice Engine Reorganization

This document outlines the major reorganization of the Artifice Engine codebase to improve structure, clarity, and maintainability.

## Overview

The engine has been restructured from a flat `io` module into specialized, focused modules that better reflect their purpose and reduce ambiguity.

## New Module Structure

### Before
```
src/
├── event.rs
├── io/
│   ├── artificeglfw.rs
│   ├── gamepad.rs
│   ├── hot_reload.rs
│   ├── input_manager.rs
│   ├── keyboard.rs
│   ├── metrics.rs
│   ├── mouse.rs
│   ├── recording.rs
│   ├── wayland.rs
│   └── window_factory.rs
├── lib.rs
└── main.rs
```

### After
```
src/
├── events/
│   ├── mod.rs
│   └── core.rs (was event.rs)
├── input/
│   ├── mod.rs
│   ├── gamepad.rs
│   ├── keyboard.rs
│   ├── mouse.rs
│   ├── manager.rs (was input_manager.rs)
│   └── recording.rs
├── window/
│   ├── mod.rs
│   ├── artificeglfw.rs
│   ├── wayland.rs
│   ├── factory.rs (was window_factory.rs)
│   └── backend_hotswap.rs (was hot_reload.rs)
├── io/
│   ├── mod.rs
│   └── metrics.rs
├── lib.rs
└── examples/
    ├── basic_demo.rs (was main.rs)
    ├── comprehensive_demo.rs
    ├── event_system_demo.rs
    └── dod_performance_demo.rs (was dod_demo.rs)
```

## Key Changes

### 1. Module Reorganization

#### Events Module (`src/events/`)
- **Purpose**: Event system, event types, and event handling
- **Contents**: Core event types, event dispatcher, event filters
- **Moved from**: `src/event.rs` → `src/events/core.rs`

#### Input Module (`src/input/`)
- **Purpose**: All input-related functionality
- **Contents**: Keyboard, mouse, gamepad input handling, input recording
- **Moved from**: Various files in `src/io/`

#### Window Module (`src/window/`)
- **Purpose**: Window management and backend systems
- **Contents**: Window factories, GLFW/Wayland implementations, backend hotswapping
- **Moved from**: Various files in `src/io/`

#### IO Module (`src/io/`)
- **Purpose**: General I/O utilities and metrics
- **Contents**: Metrics system, core window/input traits
- **Remaining**: Only `metrics.rs` and core traits

### 2. File Renames for Clarity

| Old Name | New Name | Reason |
|----------|----------|--------|
| `hot_reload.rs` | `backend_hotswap.rs` | "Hot reload" is ambiguous in game engines |
| `input_manager.rs` | `manager.rs` | Shorter name within input module |
| `window_factory.rs` | `factory.rs` | Shorter name within window module |
| `main.rs` | `examples/basic_demo.rs` | main.rs should not contain examples |
| `dod_demo.rs` | `examples/dod_performance_demo.rs` | Better location and naming |

### 3. Type Renames for Specificity

All "HotReload" types have been renamed to "WindowBackendHotswap" for clarity:

| Old Type | New Type |
|----------|----------|
| `HotReloadManager` | `WindowBackendHotswapManager` |
| `HotReloadConfig` | `WindowBackendHotswapConfig` |
| `HotReloadStatus` | `WindowBackendHotswapStatus` |
| `HotReloadResult` | `WindowBackendHotswapResult` |
| `HotReloadBuilder` | `WindowBackendHotswapBuilder` |
| `HotReloadFactory` | `WindowBackendHotswapFactory` |
| `HotReloadStats` | `WindowBackendHotswapStats` |

For backward compatibility, the old names are re-exported from the window module.

## Migration Guide

### Import Changes

#### Events
```rust
// Old
use artifice_engine::event::{Event, EventType, KeyCode};

// New
use artifice_engine::events::{Event, EventType, KeyCode};
```

#### Input
```rust
// Old
use artifice_engine::io::{InputManager, Keyboard, Mouse, GamepadManager};

// New
use artifice_engine::input::{InputManager, Keyboard, Mouse, GamepadManager};
```

#### Window
```rust
// Old
use artifice_engine::io::{WindowFactory, HotReloadManager};

// New
use artifice_engine::window::{WindowFactory, HotReloadManager};
```

#### Metrics
```rust
// Old
use artifice_engine::io::{MetricsCollector, MetricsConfig};

// New
use artifice_engine::io::{MetricsCollector, MetricsConfig};
// Metrics remain in io module
```

### Code Changes

Most existing code will continue to work due to re-exports, but for clarity and future-proofing, update imports to use the new module structure.

### Example Updates

The `main.rs` example has been moved to `examples/basic_demo.rs`. To run it:

```bash
cargo run --example basic_demo
```

## Benefits

1. **Clearer Organization**: Related functionality is grouped together
2. **Reduced Ambiguity**: Names like "hot reload" are now specific to their purpose
3. **Better Discoverability**: Developers can easily find input, window, or event-related code
4. **Maintainability**: Smaller, focused modules are easier to maintain
5. **Scalability**: New features can be added to appropriate modules without cluttering

## Backward Compatibility

The reorganization maintains backward compatibility through re-exports in the old module locations. However, it's recommended to update imports to use the new structure for future compatibility.

## Future Considerations

- The `io` module may be further specialized or renamed as the engine evolves
- Additional modules may be created for rendering, audio, or other engine subsystems
- The module structure provides a foundation for plugin architectures