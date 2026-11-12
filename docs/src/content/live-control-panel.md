---
title: Drive BoxMux as a Live Control Panel
description: Let another program keep a BoxMux screen current and hear back from its buttons - live text, generated menus, presses reported through a pipe and a log, confirmations, terminal steps and narrow panes.
---

## What you build

A panel that another program, the **driver**, keeps current while a person uses it: text
boxes that show what the driver writes, a menu whose entries come and go, buttons that tell
the driver they were pressed, a confirmation before anything acts, and a terminal for steps
only the person can take, such as a login. The driver can be any program that writes files
and runs commands; here it is a small shell script.

The repository's `examples/live-panel/` holds a panel built this way:

| File | What it is |
|------|------------|
| `panel.yaml` | The layouts: the panel, a confirmation, a terminal, and a hidden box |
| `driver.sh` | The driver: writes the text files, builds the menu, acts on presses |
| `buttons.sh` | What the menu entries run: confirm, report a press, start a terminal step |
| `common.sh` | Shell functions both scripts use |
| `start.sh` | Starts the driver in the background and BoxMux in the terminal |

It needs `sh`, `awk` and `sed`, and runs on Linux and macOS. Run it with `boxmux` on your
`PATH`, or name the binary:

```bash
examples/live-panel/start.sh
BOXMUX=./target/release/boxmux examples/live-panel/start.sh
```

Choose **✔ Restart the web service**, confirm it, and watch the driver answer in the log at
the bottom; **▶ Sign in to the service** opens a terminal that asks for your name. Press `q`
to quit; the driver stops with BoxMux.

The sections below build each part. Each one stands on its own.

## Give the panel its own socket and lock its file

Start BoxMux with a socket of its own, so that the driver's commands reach this panel and
not another BoxMux instance, and with `--lock`, so that moving a box with the mouse never
writes positions back into a YAML file your program owns:

```bash
export BOXMUX_SOCKET="$PANEL_STATE/boxmux.sock"
boxmux --lock panel.yaml
```

On Unix a socket's path must be short, about 100 bytes; BoxMux refuses a longer one when it
starts. Keep the folder under `/tmp` rather than a deep `$TMPDIR`.

`boxmux` and its control commands read `BOXMUX_SOCKET`, and every script BoxMux runs gets it
in its environment, so neither the driver nor the buttons need `--socket`. The driver can
start before BoxMux; it waits for the socket file to appear before its first command:

```sh
until [ -S "$BOXMUX_SOCKET" ]; do sleep 0.1; done
```

To run the panel beside another program, open it in a split of your terminal, for example
`tmux split-window -h 'examples/live-panel/start.sh'`.

## Show text the driver writes

Have the driver write plain files, and have boxes show them with a script on a short
`refresh_interval`:

```yaml
- id: log
  title: What the driver is doing
  position: {x1: "0%", y1: "80%", x2: "100%", y2: "100%"}
  overflow_behavior: wrap
  auto_scroll_bottom: true
  script: ["tail -n 40 \"$PANEL_STATE/log.txt\""]
  refresh_interval: 500
```

`auto_scroll_bottom` keeps a log at its newest line. The box shows a change within one
interval. Write a file whole (to a temporary name, then `mv`) when a half-written file would
mislead the reader.

`$PANEL_STATE` is a shell variable: BoxMux leaves `$NAME` without braces as written, and the
shell resolves it from the environment when the script runs. `${NAME}` is substituted by
BoxMux when it loads the file (see [Variables](/docs/variables)), but not in boxes sent over
the socket, so the plain `$NAME` form works in both.

`replace_box_content` looks like the direct way to set a box's text, but it does not replace
what the box shows: it adds a tab labelled `Socket` at the end of the box's tabs, and the box
then has two tabs. Use files, or `replace_box` with new `content`, for text that replaces
what is there.

## Replace a menu when its entries change

When the structure of a box changes, such as a menu whose entries come and go, build the
whole box as JSON and replace it in one call:

```sh
boxmux replace_box needs '{"id":"needs","title":"Needs you: 2",
  "position":{"x1":"0%","y1":"5","x2":"100%","y2":"60%"},
  "tab_order":1,"selected":true,
  "choices":[
    {"id":"restart-web","content":"✔ Restart the web service",
     "script":["\"$PANEL_DIR/buttons.sh\" ask restart-web"],"redirect_output":"quiet"},
    {"id":"about","content":"ℹ About this panel",
     "script":["\"$PANEL_DIR/buttons.sh\" show about"],"redirect_output":"quiet"}]}'
```

The new box takes the old one's place and focus. It replaces everything: repeat the
position, colors and `tab_order`, since nothing is kept from the old box. The example keeps
its items in one file, a line per item, and `render` in `driver.sh` turns them into choices,
each marked by kind: `✔` asks for approval, `▶` runs a terminal step, `ℹ` shows details.
Keep each entry's text short and put the long text in a details box.

Replace a menu only when its entries change: the replacement moves the person's selection
back to the first entry, even when they were reading another one. A button's script must not
replace the box it was started from, because replacing a box stops everything running for
it, the script included; let the driver replace the menu after the button has reported.

## Report presses back to the driver

The control socket takes commands in but sends no events out, so a button reports its own
press. Each press does two things:

1. It appends a line to an events log. The log keeps every press, including those made
   while the driver is busy or not running.
2. It writes a line to a named pipe the driver waits on, which wakes the driver at once.

The button side, in `buttons.sh`:

```sh
report() {
    printf '%s %s\n' "$(date +%s)" "$*" >> "$EVENTS"
    if kill -0 "$(cat "$PANEL_STATE/driver.pid" 2>/dev/null)" 2>/dev/null; then
        printf '%s\n' "$*" 1<> "$FIFO"
    else
        say "Kept for the driver, which is not running: $*"
    fi
}
```

The driver side, in `driver.sh`, keeps how many lines of the log it has handled, and handles
the rest every time it wakes:

```sh
exec 3<> "$FIFO"
while :; do
    while [ "$handled" -lt "$(($(wc -l < "$EVENTS")))" ]; do
        handled=$((handled + 1))
        set -- $(sed -n "${handled}p" "$EVENTS")
        shift                    # the time of the press
        handle "$@"
    done
    read -r _ <&3
done
```

Both sides open the pipe for reading and writing (`<>`), which never blocks. The driver holds
it open the whole time, so the pipe always has a reader while the driver runs: a button never
waits for the driver, and a press made while the driver is busy waits in the pipe until its
next `read`. A button that opened the pipe for writing only would hang whenever the driver is
not running; opened for reading and writing, its line is simply dropped then.

The log is what the driver trusts; the pipe only wakes it. A driver that restarts reads the
presses it missed from the log, starting from the count it kept.

## Keep a button's output out of the menu

Every run of a choice's script opens a tab, labelled with the choice, in the box the choice
belongs to, and shows it, even when the script prints nothing. In a menu, the person then
sees an empty tab where the menu was, and may think something happened that did not.

Send buttons' output to a box nobody sees: a box in a layout that is never shown.

```yaml
- id: hidden
  title: Hidden
  children:
    - id: quiet
      position: {x1: "0%", y1: "0", x2: "100%", y2: "100%"}
      content: Output of the buttons
```

and give each choice `redirect_output: quiet`. The tabs then open in `quiet`, out of sight.
Errors go there too, so a button must write its own failures where the person sees them; the
example's `bm` function writes a refused control command to the log box:

```sh
bm() {
    if ! err=$(boxmux "$@" 2>&1 >/dev/null); then
        say "boxmux $1 failed: $err"
        return 1
    fi
}
```

## Confirm before acting

A button that acts at once gets pressed by mistake. Put a confirmation between the press and
the report: a second layout with a text box and a menu, both filled for the item before the
layout is shown.

```yaml
- id: confirm
  title: Confirm
  children:
    - id: confirm_text
      title: Are you sure?
      position: {x1: "0%", y1: "0", x2: "100%", y2: "60%"}
      overflow_behavior: wrap
      content: buttons.sh ask fills this in
    - id: confirm_menu
      title: Choose
      position: {x1: "0%", y1: "60%", x2: "100%", y2: "100%"}
      tab_order: 1
      selected: true
      choices:
        - id: cancel
          content: "✖ Cancel"
          script: ["\"$PANEL_DIR/buttons.sh\" back"]
          redirect_output: quiet
```

The menu entry for an item runs `buttons.sh ask <id>`, which:

1. replaces `confirm_text` with the item's label and details as its `content`;
2. replaces `confirm_menu` with one choice per option of the item, each running
   `buttons.sh press <id> <option>`, and **✖ Cancel**, which runs `buttons.sh back`;
3. runs `boxmux switch_active_layout confirm`.

Only `press` reports to the driver, then switches back with
`boxmux switch_active_layout main`. Cancel only switches back. Options let one confirmation
offer several actions, such as **Deploy to staging** and **Deploy to production**, and the
report names the one chosen.

## Hand a step to the person in a terminal

Some steps only the person can take: a login, a password, a `sudo` prompt. Give them a
layout with a terminal box and a way back:

```yaml
- id: term
  title: Terminal
  children:
    - id: term_bar
      title: Terminal
      position: {x1: "0%", y1: "0", x2: "100%", y2: "3"}
      tab_order: 2
      choices:
        - id: term_back
          content: "◀ Back to the panel"
          script: ["\"$PANEL_DIR/buttons.sh\" back"]
          redirect_output: quiet
    - id: term_box
      title: Run
      position: {x1: "0%", y1: "3", x2: "100%", y2: "100%"}
      tab_order: 1
      selected: true
      content: buttons.sh run starts the step here
```

and start the command in it:

```sh
boxmux switch_active_layout term
boxmux replace_box term_box '{"id":"term_box","title":"Run",
  "position":{"x1":"0%","y1":"3","x2":"100%","y2":"100%"},
  "tab_order":1,"selected":true,"content":"Starting..."}'
boxmux spawn_pty_process term_box "$command; status=\$?; echo; echo \"Finished with exit status \$status. Choose Back above.\""
```

Switch the layout first: a terminal takes its size from its box, so the box must be in the
shown layout. Replacing `term_box` before each step removes the previous step's terminal tab.
While the terminal has the focus every key goes to the program except Tab and Shift+Tab,
which move the focus to the Back entry; a click on it works too. See
[PTY Features](/docs/pty-features).

Keep this for steps that need the person. A command whose result is only text should not be
a terminal step: have the driver run it and write what the result means into the log or the
details box, rather than showing the person raw output they have to interpret.

## Fit a narrow pane

A panel beside another program often gets 36 to 60 columns. Lay it out as a stack:

- a short header box for status, a few rows high;
- the menu, taking most of the height;
- a details box for the long text of the selected item;
- the log at the bottom.

Give fixed rows in cells and let the rest share percentages, as `panel.yaml` does:

```yaml
position: {x1: "0%", y1: "0", x2: "100%", y2: "5"}    # header: rows 0 to 5
position: {x1: "0%", y1: "5", x2: "100%", y2: "60%"}  # menu: from row 5 to 60%
position: {x1: "0%", y1: "60%", x2: "100%", y2: "80%"}
position: {x1: "0%", y1: "80%", x2: "100%", y2: "100%"}
```

Count three rows of every box for its frame: the top border, an empty row under it, and the
bottom border, which the box below draws its own top border over. The header above, from row
0 to row 5, has room for three lines of text, on rows 2 to 4. Keep menu entries short, and use
`overflow_behavior: wrap` on menus and details so that nothing is cut off at the right edge.

## Things to know

| What happens | What to do |
|--------------|------------|
| The socket sends no events: a press reaches the driver only if the button's script reports it | Report through a log and a pipe, as above |
| Every run of a choice opens a tab in its box and shows it | `redirect_output` to a box in a layout that is never shown, and write errors to the log |
| Replacing a menu with `replace_box` moves the selection back to the first entry | Replace the menu only when its entries change |
| `replace_box_content` adds a `Socket` tab instead of replacing the box's text | Show files with a refreshing script, or replace the box with new `content` |
| A box showing a file rereads it every `refresh_interval`, and lags behind changes by up to one interval | Keep the interval short (500 to 2000 ms) |
| There is no built-in confirmation for a choice | A confirmation layout, as above |
| Every box draws an empty row under its top border | Count it when sizing boxes for a small pane |

## See also

- [API Reference](/docs/api): `replace_box`, `switch_active_layout`, `spawn_pty_process` and
  the environment of scripts
- [Configuration Reference](/docs/configuration): `choices`, `redirect_output`,
  `refresh_interval`, `auto_scroll_bottom`
- [PTY Features](/docs/pty-features): terminals in boxes
