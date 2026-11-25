#!/bin/sh
# What the panel's buttons run: buttons.sh <action> [<id>].
#   press <id>   report to the driver the answer the item's confirmation was given, which BoxMux
#                passes in BOXMUX_OPTION
#   run <id>     run the item's command in a terminal, for a step only the human can take
#   show <id>    show the item's details
#   back         go back to the panel
# Its choices discard its output (`output: none`), so a press opens no tab over the menu: what
# it has to say, its failures included, is written to the log.
set -eu
. "$PANEL_DIR/common.sh"

# Record a press in the events log, then wake the driver through its pipe. Opened for reading
# and writing, the pipe never blocks: with no driver listening the line is dropped, and the log
# keeps the press for when the driver starts.
report() {
    printf '%s %s\n' "$(date +%s)" "$*" >> "$EVENTS"
    if kill -0 "$(cat "$PANEL_STATE/driver.pid" 2>/dev/null)" 2>/dev/null; then
        printf '%s\n' "$*" 1<> "$FIFO"
    else
        say "Kept for the driver, which is not running: $*"
    fi
}

item() {
    if ! field "$1" "$2"; then
        say "There is no item '$1'"
        exit 1
    fi
}

case ${1:-} in
    press)
        if [ -z "${BOXMUX_OPTION:-}" ]; then
            say "buttons.sh press $2: BOXMUX_OPTION is not set; press runs from a confirmed choice"
            exit 1
        fi
        report press "$2" "$BOXMUX_OPTION"
        ;;
    run)
        command=$(item "$2" 5)
        report run "$2"
        bm switch_active_layout term
        # A fresh box: the previous run's terminal tab goes with the old one.
        bm replace_box term_box '{"id":"term_box","title":"Run","position":{"x1":"0%","y1":"3","x2":"100%","y2":"100%"},"border_color":"bright_white","tab_order":1,"selected":true,"content":"Starting..."}'
        bm spawn_pty_process term_box "$command; status=\$?; echo; echo \"Finished with exit status \$status. Choose Back above.\""
        ;;
    show)
        label=$(item "$2" 3)
        detail=$(item "$2" 4)
        printf '%s\n\n%s\n' "$label" "$detail" > "$PANEL_STATE/detail.txt"
        ;;
    back)
        bm switch_active_layout main
        ;;
    *)
        say "buttons.sh: unknown action '${1:-}'"
        exit 2
        ;;
esac
