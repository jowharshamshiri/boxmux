#!/bin/sh
# The program that drives the panel. It keeps the text boxes' files current, builds the
# `needs` menu from its list of items and replaces it over the control socket, and acts on the
# presses that buttons.sh reports. start.sh runs it in the background.
set -eu
. "$PANEL_DIR/common.sh"

echo $$ > "$PANEL_STATE/driver.pid"
trap 'say "The driver stopped."' EXIT

cat > "$ITEMS" <<'EOF'
restart-web|approve|Restart the web service|Restarts the web service. Visitors see errors for about ten seconds.|yes=✔ Yes, restart it
deploy|approve|Deploy release 2.4|Deploys release 2.4. Choose where it goes.|staging=Deploy to staging;production=Deploy to production
sign-in|run|Sign in to the service|A step only you can take: it asks for your name in a terminal.|printf 'Your name: '; read -r name; echo "Signed in as $name."
about|info|About this panel|Every press goes to driver.sh, which answers in the log below.|
EOF

handled=0

status() {
    waiting=$(grep -c -v '^[^|]*|info|' "$ITEMS" || true)
    printf 'driver.sh (pid %s) is listening\n%s waiting for you, %s presses handled\n' \
        "$$" "$waiting" "$handled" > "$PANEL_STATE/status.txt"
}

# Replace the whole `needs` box with one built from the items, in one call.
render() {
    choices= waiting=0
    while IFS='|' read -r id kind label detail extra; do
        case $kind in
            approve) mark='✔' action=ask waiting=$((waiting + 1)) ;;
            run) mark='▶' action=run waiting=$((waiting + 1)) ;;
            info) mark='ℹ' action=show ;;
            *) say "Item '$id' has the unknown kind '$kind'"; return 1 ;;
        esac
        choice=$(printf '{"id":"%s","content":"%s %s","script":["\\"$PANEL_DIR/buttons.sh\\" %s %s"]}' \
            "$id" "$mark" "$(json "$label")" "$action" "$id")
        choices=$choices${choices:+,}$choice
    done < "$ITEMS"
    [ -n "$choices" ] || choices='{"id":"none","content":"Nothing needs you"}'
    bm replace_box needs "$(printf '{"id":"needs","title":"Needs you: %s","position":{"x1":"0%%","y1":"5","x2":"100%%","y2":"60%%"},"border_color":"bright_yellow","tab_order":1,"selected":true,"overflow_behavior":"wrap","choices":[%s]}' \
        "$waiting" "$choices")"
}

# Act on one press: `press <id> <option>` or `run <id>`.
handle() {
    # Shell variables are global: render's loop sets `label` too.
    if ! title=$(field "$2" 3); then
        say "A press for '$2', which is no longer on the list"
        return 0
    fi
    case $1 in
        press)
            say "You chose: $title: $(option_label "$2" "$3")"
            say "Working on it..."
            sleep 1
            grep -v "^$2|" "$ITEMS" > "$ITEMS.new" || true
            mv "$ITEMS.new" "$ITEMS"
            render
            say "Done: $title"
            ;;
        run)
            say "You started: $title"
            ;;
        *)
            say "Unknown event '$1'"
            ;;
    esac
}

# Handle every line of the events log past the ones already handled.
handle_new() {
    while [ "$handled" -lt "$(($(wc -l < "$EVENTS")))" ]; do
        handled=$((handled + 1))
        # The line's words are the event's fields.
        set -- $(sed -n "${handled}p" "$EVENTS")
        shift # the time of the press
        handle "$@"
    done
    status
}

# Opened for reading and writing, the pipe always has a reader while the driver runs, so a
# press never waits for the driver and none is lost while it is busy.
exec 3<> "$FIFO"

tries=0
until [ -S "$BOXMUX_SOCKET" ]; do
    tries=$((tries + 1))
    if [ "$tries" -gt 100 ]; then
        echo "driver.sh: boxmux did not open $BOXMUX_SOCKET within 10 seconds" >&2
        exit 1
    fi
    sleep 0.1
done

render
status
say "The driver is listening. Choose an entry above."
while :; do
    handle_new
    read -r _ <&3
done
