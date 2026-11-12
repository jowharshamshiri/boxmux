#!/bin/sh
# What the panel's buttons run: buttons.sh <action> [<id> [<option>]].
#   ask <id>             show the confirmation for an item, with its options and Cancel
#   press <id> <option>  report the confirmed choice to the driver and go back to the panel
#   run <id>             run the item's command in a terminal, for a step only the human can take
#   show <id>            show the item's details
#   back                 go back to the panel
# Its output goes to the hidden `quiet` box, so failures are written to the log instead.
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
    ask)
        label=$(item "$2" 3)
        detail=$(item "$2" 4)
        options=$(item "$2" 5)
        menu=
        for option in $(printf '%s' "$options" | tr ';' '\n' | cut -d= -f1); do
            menu=$menu$(printf '{"id":"%s","content":"%s","script":["\\"$PANEL_DIR/buttons.sh\\" press %s %s"],"redirect_output":"quiet"},' \
                "$option" "$(json "$(option_label "$2" "$option")")" "$2" "$option")
        done
        bm replace_box confirm_text "$(printf '{"id":"confirm_text","title":"Are you sure?","position":{"x1":"0%%","y1":"0","x2":"100%%","y2":"60%%"},"border_color":"bright_yellow","overflow_behavior":"wrap","content":"%s\\n\\n%s"}' \
            "$(json "$label")" "$(json "$detail")")"
        bm replace_box confirm_menu "$(printf '{"id":"confirm_menu","title":"Choose","position":{"x1":"0%%","y1":"60%%","x2":"100%%","y2":"100%%"},"border_color":"bright_green","tab_order":1,"selected":true,"choices":[%s{"id":"cancel","content":"✖ Cancel","script":["\\"$PANEL_DIR/buttons.sh\\" back"],"redirect_output":"quiet"}]}' \
            "$menu")"
        bm switch_active_layout confirm
        ;;
    press)
        report press "$2" "$3"
        bm switch_active_layout main
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
