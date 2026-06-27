#!/bin/bash

# Test script to verify child process output capture
# This simulates the cargo build -> binary execution scenario

echo "=== Parent Process Starting ==="
echo "Parent PID: $$"
echo "Parent PGID: $(ps -o pgid= -p $$)"

echo "=== Spawning Child Process ==="
# Simulate what cargo build does - spawn a child process
(
    sleep 0.5
    echo "Child process output - this should be captured!"
    echo "Child PID: $$"
    echo "Child PGID: $(ps -o pgid= -p $$)"
    
    # Spawn a grandchild to really test the scenario
    (
        sleep 0.5
        echo "Grandchild process output - this is often lost!"
        echo "Grandchild PID: $$"
        echo "Grandchild PGID: $(ps -o pgid= -p $$)"
    ) &
    
    wait
) &

echo "=== Waiting for Child Process ==="
wait

echo "=== Parent Process Complete ==="