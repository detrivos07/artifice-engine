#!/bin/bash

# Backend Switching Demo Runner Script
# Builds and runs the Artifice Engine backend switching examples

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ] || [ ! -d "examples" ]; then
    print_error "Please run this script from the artifice-engine directory"
    exit 1
fi

# Function to check system dependencies
check_dependencies() {
    print_info "Checking system dependencies..."
    
    # Check for basic OpenGL libraries
    if ! ldconfig -p | grep -q libGL; then
        print_warning "OpenGL libraries may not be installed"
        print_info "Try: sudo apt install libgl1-mesa-dev (Ubuntu/Debian)"
        print_info "Or: sudo pacman -S mesa (Arch Linux)"
    fi
    
    # Check for GLFW
    if ! ldconfig -p | grep -q libglfw; then
        print_warning "GLFW libraries may not be installed"
        print_info "Try: sudo apt install libglfw3-dev (Ubuntu/Debian)"
        print_info "Or: sudo pacman -S glfw-x11 (Arch Linux)"
    fi
    
    # Check for Wayland (if requested)
    if [ "$1" = "wayland" ]; then
        if ! ldconfig -p | grep -q libwayland; then
            print_warning "Wayland libraries not found"
            print_info "Try: sudo apt install libwayland-dev wayland-protocols (Ubuntu/Debian)"
            print_info "Or: sudo pacman -S wayland wayland-protocols (Arch Linux)"
        fi
        
        if [ -z "$WAYLAND_DISPLAY" ] && [ -z "$DISPLAY" ]; then
            print_warning "No display server detected. Make sure X11 or Wayland is running."
        fi
    fi
    
    # Check for X11 (if requested)
    if [ "$1" = "x11" ]; then
        if ! ldconfig -p | grep -q libX11; then
            print_warning "X11 libraries not found"
            print_info "Try: sudo apt install libx11-dev libgl1-mesa-dev (Ubuntu/Debian)"
            print_info "Or: sudo pacman -S libx11 mesa (Arch Linux)"
        fi
        
        if [ -z "$DISPLAY" ]; then
            print_warning "No X11 display detected. Make sure X11 is running."
        fi
    fi
}

# Function to build examples
build_examples() {
    local features="$1"
    
    print_info "Building examples..."
    if [ -n "$features" ]; then
        print_info "Features: $features"
        cargo build --examples --features "$features"
    else
        cargo build --examples
    fi
    
    if [ $? -eq 0 ]; then
        print_success "Examples built successfully"
    else
        print_error "Failed to build examples"
        exit 1
    fi
}

# Function to run an example
run_example() {
    local example="$1"
    local features="$2"
    
    print_info "Running example: $example"
    
    # Set logging level for better output
    export RUST_LOG=info
    
    if [ -n "$features" ]; then
        cargo run --example "$example" --features "$features"
    else
        cargo run --example "$example"
    fi
}

# Function to show example controls
show_controls() {
    local example="$1"
    
    case "$example" in
        "basic_demo")
            print_info "Controls for Basic Demo:"
            echo "  R - Reset rotation"
            echo "  ESC - Exit"
            ;;
        "backend_switching_demo")
            print_info "Controls for Backend Switching Demo:"
            echo "  G - Switch to GLFW backend"
            echo "  W - Switch to Wayland backend"
            echo "  X - Switch to X11 backend"
            echo "  R - Reset rotation and colors"
            echo "  ESC - Exit"
            ;;
        "advanced_backend_demo")
            print_info "Controls for Advanced Backend Demo:"
            echo "  G - Switch to GLFW backend"
            echo "  W - Switch to Wayland backend"
            echo "  X - Switch to X11 backend"
            echo "  R - Reset animations"
            echo "  SPACE - Show status"
            echo "  ESC - Exit"
            ;;
    esac
    echo ""
}

# Function to show usage
show_usage() {
    echo "Backend Switching Demo Runner"
    echo ""
    echo "Usage: $0 [command] [options]"
    echo ""
    echo "Commands:"
    echo "  build           Build all examples (GLFW only)"
    echo "  build-wayland   Build all examples with Wayland support"
    echo "  build-x11       Build all examples with X11 support"
    echo "  build-all       Build all examples with all backends"
    echo "  basic           Run basic demo"
    echo "  switching       Run backend switching demo"
    echo "  advanced        Run advanced backend demo"
    echo "  all             Run all demos in sequence"
    echo "  check           Check system dependencies"
    echo "  help            Show this help message"
    echo ""
    echo "Options:"
    echo "  --wayland       Enable Wayland support for demos"
    echo "  --x11           Enable X11 support for demos"
    echo "  --all-backends  Enable all backend support"
    echo "  --debug         Enable debug logging"
    echo "  --trace         Enable trace logging"
    echo ""
    echo "Examples:"
    echo "  $0 build-wayland          # Build with Wayland support"
    echo "  $0 build-x11              # Build with X11 support"
    echo "  $0 build-all              # Build with all backends"
    echo "  $0 switching --wayland     # Run switching demo with Wayland"
    echo "  $0 switching --x11         # Run switching demo with X11"
    echo "  $0 advanced --all-backends # Run advanced demo with all backends"
    echo "  $0 advanced --debug        # Run advanced demo with debug logging"
}

# Parse command line arguments
COMMAND=""
BACKEND_FEATURES=""
DEBUG_LEVEL=""

while [[ $# -gt 0 ]]; do
    case $1 in
        build|build-wayland|build-x11|build-all|basic|switching|advanced|all|check|help)
            COMMAND="$1"
            ;;
        --wayland)
            BACKEND_FEATURES="wayland"
            ;;
        --x11)
            BACKEND_FEATURES="x11"
            ;;
        --all-backends)
            BACKEND_FEATURES="wayland,x11"
            ;;
        --debug)
            DEBUG_LEVEL="debug"
            ;;
        --trace)
            DEBUG_LEVEL="trace"
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
    shift
done

# Set logging level
if [ "$DEBUG_LEVEL" = "debug" ]; then
    export RUST_LOG=debug
elif [ "$DEBUG_LEVEL" = "trace" ]; then
    export RUST_LOG=trace
fi

# Handle special build commands
if [ "$COMMAND" = "build-wayland" ]; then
    BACKEND_FEATURES="wayland"
    COMMAND="build"
elif [ "$COMMAND" = "build-x11" ]; then
    BACKEND_FEATURES="x11"
    COMMAND="build"
elif [ "$COMMAND" = "build-all" ]; then
    BACKEND_FEATURES="wayland,x11"
    COMMAND="build"
fi

# Execute command
case "$COMMAND" in
    "build")
        check_dependencies "$BACKEND_FEATURES"
        build_examples "$BACKEND_FEATURES"
        print_success "Build complete! Use '$0 help' to see available demos."
        ;;
        
    "basic")
        build_examples "$BACKEND_FEATURES"
        show_controls "basic_demo"
        run_example "basic_demo" "$BACKEND_FEATURES"
        ;;
        
    "switching")
        build_examples "$BACKEND_FEATURES"
        show_controls "backend_switching_demo"
        run_example "backend_switching_demo" "$BACKEND_FEATURES"
        ;;
        
    "advanced")
        build_examples "$BACKEND_FEATURES"
        show_controls "advanced_backend_demo"
        run_example "advanced_backend_demo" "$BACKEND_FEATURES"
        ;;
        
    "all")
        build_examples "$BACKEND_FEATURES"
        
        print_info "Running all demos in sequence..."
        print_info "Press Ctrl+C to skip to next demo, or ESC in demo to continue"
        
        for demo in "basic_demo" "backend_switching_demo" "advanced_backend_demo"; do
            print_info "Starting $demo..."
            show_controls "$demo"
            read -p "Press Enter to start $demo (or Ctrl+C to skip)..."
            run_example "$demo" "$BACKEND_FEATURES" || true
            echo ""
        done
        
        print_success "All demos completed!"
        ;;
        
    "check")
        check_dependencies "$BACKEND_FEATURES"
        print_info "Dependency check complete"
        ;;
        
    "help"|"")
        show_usage
        ;;
        
    *)
        print_error "Unknown command: $COMMAND"
        show_usage
        exit 1
        ;;
esac