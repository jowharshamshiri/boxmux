---
title: Advanced Features
description: Variables, script libraries, output redirection and saving, hot keys and key handlers, focus order, layering, anchored sizing, socket control from scripts, embedded terminals and plugins.
---


## Table of Contents

- [Variables](#variables)
- [Script libraries](#script-libraries)
- [Output handling](#output-handling)
- [Hot keys and key handlers](#hot-keys-and-key-handlers)
- [Focus order](#focus-order)
- [Layering with z_index](#layering-with-z_index)
- [Anchors and size limits](#anchors-and-size-limits)
- [Controlling BoxMux from its own scripts](#controlling-boxmux-from-its-own-scripts)
- [Embedded terminals](#embedded-terminals)
- [Plugins](#plugins)

## Variables

`variables` can be set on the app, on a layout and on any box. `${NAME}` is replaced by the
value of `NAME`, looked up in this order:

1. the box the string belongs to,
2. its enclosing boxes, nearest first,
3. the layout,
4. the app,
5. the process environment.

`${NAME:default}` uses `default` when `NAME` is not defined anywhere, and the default may
itself contain references: `${PORT:${DEFAULT_PORT}}`. Values may be strings, numbers or
booleans.

Substitution happens once, in every string of the file (titles, content, scripts, colors,
positions), before the file is checked. Substituted values are not scanned again, so a value
containing `${...}` is inserted as written. The result of a substitution is text, so settings
that take a number or a boolean (`refresh_interval`, `tab_order`, `z_index`, `append_output`
and so on) cannot come from a variable.

Some forms are left for the shell:

- `$${` writes a literal `${`.
- A reference to a name defined nowhere, without a default, is left as written, so the shell
  can expand it when the script runs (for example a loop variable `${i}`).
- Shell syntax such as `${X:-y}`, `${#list[@]}` or `${file%.txt}` is left as written.

```yaml
app:
  variables:
    ENVIRONMENT: staging
    DEFAULT_HOST: localhost
  layouts:
    - id: main
      root: true
      title: Services (${ENVIRONMENT})
      variables:
        REGION: eu-west
      children:
        - id: services
          title: Services
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          variables:
            SERVICE: api
          children:
            - id: api_status
              title: ${SERVICE} on ${HOST:${DEFAULT_HOST}}
              position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
              refresh_interval: 5000
              script:
                - echo "Checking ${SERVICE} in ${ENVIRONMENT} (${REGION})"
                - for i in 1 2 3; do echo "attempt ${i}"; done
            - id: db_status
              title: database
              position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
              variables:
                SERVICE: database
              script:
                - echo "Checking ${SERVICE} for user ${USER:unknown}"
```

Here `api_status` takes `SERVICE` from its enclosing box, `db_status` overrides it, `HOST`
falls back to `DEFAULT_HOST` unless the environment defines it, and `${i}` reaches the shell
unchanged. See [Variables](/docs/variables) for more.

## Script libraries

`app.libs` lists script files that are sourced before every script BoxMux runs (box scripts,
choices, key handlers and terminals). Relative paths are relative to the YAML file.

```yaml
app:
  libs:
    - lib/helpers.sh
  layouts:
    - id: main
      root: true
      children:
        - id: report
          title: Report
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          script:
            - print_header "Disk"
            - df -h
```

with `lib/helpers.sh`:

```bash
print_header() {
    echo "== $1 =="
}
```

Each library is sourced with `. 'path'`, in `bash`, `sh`, `zsh` and PowerShell alike, so write
the libraries for the shell the app uses (`app.shell`).

## Output handling

| Setting | Where | Effect |
|---------|-------|--------|
| `redirect_output` | box, choice | Shows the output in another box (any layout), in a tab of its own. Needs a script. |
| `append_output` | box, choice | Adds each run's output after the previous output instead of replacing it. |
| `auto_scroll_bottom` | box | Scrolls to the newest line when the visible tab gets new output. |
| `save_in_file` | box | After each run of the box's script, writes its output (without color codes) to this file, replacing the file's contents. Relative paths are relative to the YAML file. |

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: actions
          title: Actions
          position: {x1: 0%, y1: 0%, x2: 30%, y2: 100%}
          tab_order: 1
          choices:
            - id: build
              content: Build
              script:
                - cargo build 2>&1
              execution_mode: Thread
              redirect_output: log
            - id: stamp
              content: Add timestamp
              script:
                - date
              redirect_output: log
              append_output: true

        - id: log
          title: Log
          position: {x1: 30%, y1: 0%, x2: 100%, y2: 70%}
          auto_scroll_bottom: true
          content: Output of the actions appears here in new tabs.

        - id: uptime
          title: Uptime (saved to uptime.txt)
          position: {x1: 30%, y1: 70%, x2: 100%, y2: 100%}
          refresh_interval: 10000
          save_in_file: uptime.txt
          script:
            - uptime
```

## Hot keys and key handlers

`app.hot_keys` maps a key to the id of a choice. Pressing the key runs that choice as if it had
been selected, provided the choice is in the layout being shown. The choice must exist, or the
file does not load.

`on_keypress` maps a key to script lines. It can be set on the app, on a layout and on a box:

- App and layout handlers run in the background; their output is not shown, and a failure
  appears as a message on the bottom row. A handler that is still running is not started again.
- A box handler runs when that box has the focus and its visible tab is not a menu. Its output
  appears in a tab of the box, or of its `redirect_output` box, streamed line by line.

```yaml
app:
  hot_keys:
    F5: refresh_now
    Ctrl+t: run_tests
  on_keypress:
    Ctrl+l:
      - date >> keypress.log
  layouts:
    - id: main
      root: true
      children:
        - id: menu
          title: Actions (F5, Ctrl+t)
          position: {x1: 0%, y1: 0%, x2: 40%, y2: 100%}
          tab_order: 1
          choices:
            - id: refresh_now
              content: Show the date
              script:
                - date
            - id: run_tests
              content: Run the tests
              script:
                - cargo test 2>&1
              execution_mode: Thread
        - id: notes
          title: Notes (press n here)
          position: {x1: 40%, y1: 0%, x2: 100%, y2: 100%}
          tab_order: 2
          content: Focus this box and press n.
          on_keypress:
            n:
              - echo "Note taken at $(date)"
```

### Key names

| Kind | Names |
|------|-------|
| Characters | `a`, `A`, `1`, `/` ... (`a` and `A` are different keys) |
| Space | `Space` |
| Special keys | `Enter`, `Tab`, `Backspace`, `Delete`, `Insert`, `Esc`, `Home`, `End`, `PageUp`, `PageDown`, `Up`, `Down`, `Left`, `Right`, `F1` ... `F24` |
| Modifiers | `Ctrl+`, `Alt+`, `Cmd+`, and `Shift+` for keys that are not characters: `Ctrl+r`, `Alt+x`, `Shift+Up`, `Shift+Tab`, `Ctrl+Alt+Delete` |

Modifier and special key names are matched ignoring case (`ctrl+r` = `Ctrl+r`, `enter` =
`Enter`); single characters must match exactly. Modifiers can be written in any order.
`Ctrl++` is Ctrl with the plus key.

Key names in `hot_keys` and every `on_keypress` are checked when the file is loaded, and a
name that could never match a key press is an error naming it: an unknown key or modifier
(`Ctrl+Enterr`, `Hyper+x`), a modifier named twice, or `Shift+` with a character (write the
shifted character, `A`, instead of `Shift+a`).

Things to know:

- A plain `q` always quits, and on Unix `Ctrl+z` suspends BoxMux; binding either is an error.
- Keys that BoxMux uses (Tab, arrows, Enter, Page Up/Down, Home/End, Ctrl+C) do their usual job
  and then also run any binding for them.
- When the focused box shows a running terminal, every key except Tab and Shift+Tab goes to the
  program, and no bindings run.
- Which keys reach BoxMux depends on the terminal; some combinations (for example many `Cmd+`
  keys on macOS) are taken by the terminal application.

## Focus order

- `tab_order` (a whole number) puts a box in the keyboard focus order. Tab moves forward and
  Shift+Tab backward, wrapping around. Boxes without `tab_order` are not reached with the
  keyboard, but can be focused with a click.
- `next_focus_id` names the box Tab moves to from this box, instead of the next one in
  `tab_order`. The target must be in the same layout. Shift+Tab still follows `tab_order`.
- `selected: true` gives a box the focus when its layout is first shown; otherwise the first
  box in `tab_order` gets it.

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: search
          title: Search
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 20%}
          tab_order: 1
          next_focus_id: results
          selected: true
          content: Tab jumps to Results.
        - id: filters
          title: Filters
          position: {x1: 0%, y1: 20%, x2: 30%, y2: 100%}
          tab_order: 2
          content: Reached with Shift+Tab from Search.
        - id: results
          title: Results
          position: {x1: 30%, y1: 20%, x2: 100%, y2: 100%}
          tab_order: 3
          content: Tab from here goes back to Search.
```

## Layering with z_index

Boxes are drawn in configuration order, and each box's children after it. Among siblings,
`z_index` (a whole number, default `0`, negative allowed) changes the order: a higher value is
drawn later, on top. Clicks and the mouse wheel go to the box visible under the pointer.

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: background
          title: Background
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          content: This box is drawn first.
        - id: popup
          title: On top
          position: {x1: 30%, y1: 30%, x2: 70%, y2: 60%}
          z_index: 10
          content: This box covers the middle of the background.
```

## Anchors and size limits

`min_width`, `min_height`, `max_width` and `max_height` (in cells) limit the size a box gets
from its position. When a limit changes the size, `anchor` decides which point of the box stays
where the position put it: `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`, `Center`
(default), `CenterTop`, `CenterBottom`, `CenterLeft` or `CenterRight`. The box is then kept
inside its parent. A minimum larger than the maximum is an error.

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: sidebar
          title: Sidebar
          position: {x1: 0%, y1: 0%, x2: 20%, y2: 100%}
          anchor: TopLeft
          min_width: 24
          max_width: 40
          content: Between 24 and 40 columns wide, growing to the right.
        - id: dialog
          title: Dialog
          position: {x1: 40%, y1: 40%, x2: 60%, y2: 60%}
          anchor: Center
          min_width: 30
          min_height: 7
          z_index: 1
          content: At least 30x7, centered where the position puts it.
```

## Controlling BoxMux from its own scripts

Every script BoxMux runs has two environment variables:

- `BOXMUX_SOCKET`: the control socket of this instance.
- `BOXMUX_BOX`: the id of the box the script's output goes to (its `redirect_output` box when
  it has one; empty for app and layout key handlers).

Scripts run in the directory BoxMux was started from.

The `boxmux` control commands use `BOXMUX_SOCKET` when no `--socket` is given, so a script can
change the interface it runs in (with `boxmux` on its `PATH`):

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: menu
          title: Menu
          position: {x1: 0%, y1: 0%, x2: 40%, y2: 100%}
          tab_order: 1
          choices:
            - id: deploy
              content: Deploy
              script:
                - boxmux replace_box_content status "Deploying..."
                - sleep 2
                - boxmux replace_box_content status "Deployed at $(date +%H:%M)"
            - id: pause
              content: Pause the clock
              script:
                - boxmux stop_box_refresh clock
            - id: resume
              content: Resume the clock
              script:
                - boxmux start_box_refresh clock
        - id: status
          title: Status
          position: {x1: 40%, y1: 0%, x2: 100%, y2: 50%}
          content: Idle
        - id: clock
          title: Clock
          position: {x1: 40%, y1: 50%, x2: 100%, y2: 100%}
          refresh_interval: 1000
          script:
            - date
```

Other programs can do the same with the socket path, or by writing JSON requests to the socket
directly. See the [API Reference](/docs/api) for every command.

## Embedded terminals

`execution_mode: Pty` runs a box's script (or a choice's) in a pseudo-terminal shown in the box,
for programs like `htop`, `vim`, `less` or an interactive shell. The terminal is sized to the
inside of the border and follows resizes. When the box has the focus, keys (except Tab and
Shift+Tab) and pasted text go to the program. A Pty box cannot have `refresh_interval`.
Terminals can also be started, stopped, restarted, queried and typed into through the control
socket (`spawn_pty_process`, `kill_pty_process`, `restart_pty_process`, `query_pty_status`,
`send_pty_input`, `send_signal`).

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: top
          title: top
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          tab_order: 1
          execution_mode: Pty
          script:
            - top
```

See [PTY Features](/docs/pty-features).

## Plugins

Shared libraries listed under `app.plugins` can draw the content of boxes that set
`plugin_component`. Plugins run inside BoxMux with full privileges. See
[Plugin System](/docs/plugin-system).

---

For every setting, see the [Configuration Reference](/docs/configuration).
