# Shared by driver.sh and buttons.sh, which source it. start.sh exports PANEL_DIR, PANEL_STATE
# and BOXMUX_SOCKET, so `boxmux` reaches this panel's instance.

: "${PANEL_STATE:?is not set: start the panel with start.sh}"
: "${BOXMUX_SOCKET:?is not set: start the panel with start.sh}"

# One line per item: id|kind|label|detail|extra. kind is approve, run or info; extra is
# key=label;key=label for approve (the confirmation's options) and the command for run.
ITEMS=$PANEL_STATE/items
# Every press, one line each, kept for the driver even while it is not listening.
EVENTS=$PANEL_STATE/events.log
# The driver waits on this pipe; a press writes a line to it to wake the driver at once.
FIFO=$PANEL_STATE/events.fifo

# A line in the log box at the bottom of the panel.
say() {
    printf '%s  %s\n' "$(date +%H:%M:%S)" "$1" >> "$PANEL_STATE/log.txt"
}

# A control command. A refusal is written to the log, where the human sees it.
bm() {
    if ! err=$(boxmux "$@" 2>&1 >/dev/null); then
        say "boxmux $1 failed: $err"
        return 1
    fi
}

# Text made safe inside a JSON string. Items hold no newlines or control characters.
json() {
    printf '%s' "$1" | sed -e 's/\\/\\\\/g' -e 's/"/\\"/g'
}

# field <id> <n>: field n of the item; fails when there is no such item.
field() {
    awk -F'|' -v id="$1" -v n="$2" '$1 == id { print $n; found = 1 } END { exit !found }' "$ITEMS"
}

# option_label <id> <key>: the label of one of an approve item's options.
option_label() {
    field "$1" 5 | tr ';' '\n' | sed -n "s/^$2=//p"
}
