#!/bin/bash

# PTY Control Demo Script
# Demonstrates socket-based PTY process management commands

echo "=== BoxMux PTY Control Demo ==="
echo "This script demonstrates the new socket-based PTY control commands"
echo ""

# Start BoxMux in background with PTY demo config
echo "1. Starting BoxMux with PTY demo configuration..."
./target/release/boxmux examples/pty_error_states_demo.yaml &
BOXMUX_PID=$!
echo "BoxMux started with PID: $BOXMUX_PID"
echo ""

# Wait for BoxMux to start
sleep 2

echo "2. Querying PTY status for all muxboxes..."
echo "   Query normal PTY muxbox:"
./target/release/boxmux query_pty_status normal_pty 2>/dev/null || echo "   (MuxBox may not be started yet)"

echo "   Query error PTY muxbox:"
./target/release/boxmux query_pty_status error_pty 2>/dev/null || echo "   (MuxBox may not be started yet)"

echo "   Query dead PTY muxbox:"
./target/release/boxmux query_pty_status dead_pty 2>/dev/null || echo "   (MuxBox may not be started yet)"
echo ""

echo "3. Demonstrating PTY process control..."
echo "   Kill PTY process in normal_pty muxbox:"
./target/release/boxmux kill_pty_process normal_pty 2>/dev/null || echo "   (May fail if process not running)"

echo "   Restart PTY process in normal_pty muxbox:"
./target/release/boxmux restart_pty_process normal_pty 2>/dev/null || echo "   (Restart command sent)"
echo ""

echo "4. Query status after restart..."
sleep 1
./target/release/boxmux query_pty_status normal_pty 2>/dev/null || echo "   (Query after restart)"
echo ""

echo "5. Cleanup..."
echo "   Stopping BoxMux..."
kill $BOXMUX_PID 2>/dev/null
echo ""

echo "=== PTY Control Commands Available ==="
echo "kill_pty_process <muxbox_id>    - Kill PTY process for a muxbox"
echo "restart_pty_process <muxbox_id> - Restart PTY process for a muxbox" 
echo "query_pty_status <muxbox_id>    - Get PTY process status and info"
echo ""

echo "=== PTY Error State Indicators ==="
echo "⚡ [PID:xxx Running] - Normal PTY (Bright Cyan border)"
echo "⚠️ [PID:xxx Error:msg] - Error PTY (Yellow border)"  
echo "💀 [PID:xxx Dead:reason] - Dead PTY (Red border)"
echo "⚠️ [PID:xxx Fallback] - Fallback PTY (Yellow border)"
echo ""

echo "Demo complete!"