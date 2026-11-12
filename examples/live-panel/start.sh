#!/bin/sh
# Start the live panel: driver.sh in the background, BoxMux in this terminal. Quitting BoxMux
# (q) stops the driver. BOXMUX=/path/to/boxmux picks the binary; by default, boxmux on PATH.
set -eu

PANEL_DIR=$(cd "$(dirname "$0")" && pwd)
binary=$(command -v "${BOXMUX:-boxmux}") || {
    echo "start.sh: there is no '${BOXMUX:-boxmux}'; put boxmux on PATH or set BOXMUX" >&2
    exit 1
}
# The driver and the buttons run `boxmux` too: the same binary.
PATH=$(cd "$(dirname "$binary")" && pwd):$PATH

# Everything the run writes, and BoxMux's own control socket, so that the panel does not meet
# another BoxMux instance. In /tmp rather than $TMPDIR: a socket's path must be short (about
# 100 bytes), and $TMPDIR can be long.
PANEL_STATE=$(mktemp -d /tmp/boxmux-panel.XXXXXX)
BOXMUX_SOCKET=$PANEL_STATE/boxmux.sock
export PANEL_DIR PANEL_STATE BOXMUX_SOCKET PATH

: > "$PANEL_STATE/log.txt"
: > "$PANEL_STATE/events.log"
printf 'driver.sh is starting\n' > "$PANEL_STATE/status.txt"
printf 'Choose "ℹ About this panel" to read about it here.\n' > "$PANEL_STATE/detail.txt"
mkfifo "$PANEL_STATE/events.fifo"

sh "$PANEL_DIR/driver.sh" 2> "$PANEL_STATE/driver.err" &
driver=$!

status=0
"$binary" --lock "$PANEL_DIR/panel.yaml" || status=$?

kill "$driver" 2> /dev/null || true
wait "$driver" 2> /dev/null || true
if [ -s "$PANEL_STATE/driver.err" ]; then
    echo "driver.sh reported:" >&2
    cat "$PANEL_STATE/driver.err" >&2
fi
rm -rf "$PANEL_STATE"
exit "$status"
