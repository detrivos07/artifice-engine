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
            echo "  R - Reset rotation and colors"
            echo "  ESC - Exit"
            ;;
        "advanced_backend_demo")
            print_info "Controls for Advanced Backend Demo:"
            echo "  G - Switch to GLFW backend"
            echo "  W - Switch to Wayland backend"
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
    echo "  basic           Run basic demo"
    echo "  switching       Run backend switching demo"
    echo "  advanced        Run advanced backend demo"
    echo "  all             Run all demos in sequence"
    echo "  check           Check system dependencies"
    echo "  help            Show this help message"
    echo ""
    echo "Options:"
    echo "  --wayland       Enable Wayland support for demos"
    echo "  --debug         Enable debug logging"
    echo "  --trace         Enable trace logging"
    echo ""
    echo "Examples:"
    echo "  $0 build-wayland          # Build with Wayland support"
    echo "  $0 switching --wayland     # Run switching demo with Wayland"
    echo "  $0 advanced --debug        # Run advanced demo with debug logging"
}

# Parse command line arguments
COMMAND=""
WAYLAND_SUPPORT=""
DEBUG_LEVEL=""

while [[ $# -gt 0 ]]; do
    case $1 in
        build|build-wayland|basic|switching|advanced|all|check|help)
            COMMAND="$1"
            ;;
        --wayland)
            WAYLAND_SUPPORT="wayland"
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

# Handle special build command
if [ "$COMMAND" = "build-wayland" ]; then
    WAYLAND_SUPPORT="wayland"
    COMMAND="build"
fi

# Execute command
case "$COMMAND" in
    "build")
        check_dependencies "$WAYLAND_SUPPORT"
        build_examples "$WAYLAND_SUPPORT"
        print_success "Build complete! Use '$0 help' to see available demos."
        ;;
        
    "basic")
        build_examples "$WAYLAND_SUPPORT"
        show_controls "basic_demo"
        run_example "basic_demo" "$WAYLAND_SUPPORT"
        ;;
        
    "switching")
        build_examples "$WAYLAND_SUPPORT"
        show_controls "backend_switching_demo"
        run_example "backend_switching_demo" "$WAYLAND_SUPPORT"
        ;;
        
    "advanced")
        build_examples "$WAYLAND_SUPPORT"
        show_controls "advanced_backend_demo"
        run_example "advanced_backend_demo" "$WAYLAND_SUPPORT"
        ;;
        
    "all")
        build_examples "$WAYLAND_SUPPORT"
        
        print_info "Running all demos in sequence..."
        print_info "Press Ctrl+C to skip to next demo, or ESC in demo to continue"
        
        for demo in "basic_demo" "backend_switching_demo" "advanced_backend_demo"; do
            print_info "Starting $demo..."
            show_controls "$demo"
            read -p "Press Enter to start $demo (or Ctrl+C to skip)..."
            run_example "$demo" "$WAYLAND_SUPPORT" || true
            echo ""
        done
        
        print_success "All demos completed!"
        ;;
        
    "check")
        check_dependencies "$WAYLAND_SUPPORT"
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