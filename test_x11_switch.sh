#!/bin/bash

set -e

echo "=== X11 Backend Switch Test ==="
echo "Building backend switching demo with X11 support..."

# Build the demo
cargo build --example backend_switching_demo --features x11

echo "Starting demo and testing X11 switch..."

# Create a log file for the test
LOG_FILE="/tmp/x11_switch_test.log"

# Function to cleanup background processes
cleanup() {
    if [ ! -z "$DEMO_PID" ]; then
        kill $DEMO_PID 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Start the demo in background with logging
RUST_LOG=info cargo run --example backend_switching_demo --features x11 > "$LOG_FILE" 2>&1 &
DEMO_PID=$!

echo "Demo started with PID: $DEMO_PID"
echo "Waiting 3 seconds for window to appear..."
sleep 3

# Check if demo is still running
if ! kill -0 $DEMO_PID 2>/dev/null; then
    echo "ERROR: Demo crashed or exited early"
    echo "Log output:"
    cat "$LOG_FILE"
    exit 1
fi

echo "Attempting to send X key to trigger X11 switch..."

# Try different methods to send the X key
if command -v xdotool &> /dev/null; then
    # Method 1: Use xdotool to find window and send key
    WINDOW_ID=$(xdotool search --name "Backend Switching Demo" | head -1)
    if [ ! -z "$WINDOW_ID" ]; then
        echo "Found window ID: $WINDOW_ID"
        xdotool windowactivate "$WINDOW_ID"
        sleep 0.5
        xdotool key --window "$WINDOW_ID" x
        echo "Sent X key to window"
    else
        echo "Could not find demo window with xdotool"
    fi
elif command -v wmctrl &> /dev/null; then
    # Method 2: Use wmctrl to activate window and send key
    wmctrl -a "Backend Switching Demo"
    sleep 0.5
    xdotool key x
    echo "Sent X key using wmctrl + xdotool"
else
    echo "WARNING: Neither xdotool nor wmctrl available"
    echo "You need to manually press X in the demo window to test X11 switching"
fi

echo "Waiting 5 seconds for switch to complete..."
sleep 5

# Check if demo is still running
if ! kill -0 $DEMO_PID 2>/dev/null; then
    echo "Demo has exited"
else
    echo "Stopping demo..."
    kill $DEMO_PID 2>/dev/null || true
    sleep 1
fi

echo ""
echo "=== Test Results ==="
echo "Log file: $LOG_FILE"
echo ""

# Analyze the log for key events
echo "=== Key Events Found ==="
if grep -q "switch" "$LOG_FILE"; then
    grep -E "(switch|Switch|X11|GLFW|OpenGL|reload)" "$LOG_FILE" | head -20
else
    echo "No backend switching events found in log"
fi

echo ""
echo "=== Checking for X11 switch completion ==="
if grep -q "Successfully switched to x11 backend" "$LOG_FILE"; then
    echo "✓ X11 switch completed successfully"
elif grep -q "Requesting switch to X11 backend" "$LOG_FILE"; then
    echo "⚠ X11 switch was requested but may not have completed"
    echo "Checking for errors..."
    if grep -q -E "(ERROR|WARN|Failed)" "$LOG_FILE"; then
        echo "Errors/warnings found:"
        grep -E "(ERROR|WARN|Failed)" "$LOG_FILE" | tail -10
    fi
else
    echo "❌ No X11 switch detected"
    echo "This could mean:"
    echo "  1. The X key wasn't received by the application"
    echo "  2. X11 backend switching is not working"
    echo "  3. The application didn't have focus"
fi

echo ""
echo "=== OpenGL Function Reloading ==="
if grep -q "reload.*OpenGL" "$LOG_FILE"; then
    echo "OpenGL function reloading events:"
    grep -E "reload.*OpenGL|OpenGL.*reload" "$LOG_FILE"
else
    echo "❌ No OpenGL function reloading detected"
fi

echo ""
echo "Full log saved to: $LOG_FILE"
echo "To view full log: cat $LOG_FILE"