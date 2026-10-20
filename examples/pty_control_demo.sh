#!/bin/sh
# Drives a running `boxmux examples/pty_states.yaml` through its control socket.
# Run it from a second terminal while the example is shown. Every command must succeed, so the
# script stops at the first failure.
set -eu

boxmux=${BOXMUX:-boxmux}

echo "== status of the interactive shell"
"$boxmux" query_pty_status shell

echo "== typing a command into it"
"$boxmux" send_pty_input shell "echo hello from the control socket
"

echo "== restarting the box that fails"
"$boxmux" restart_pty_process fails
sleep 1
"$boxmux" query_pty_status fails

echo "== starting a new terminal in the processes box"
"$boxmux" spawn_pty_process processes top
sleep 2
"$boxmux" kill_pty_process processes
"$boxmux" query_pty_status processes
