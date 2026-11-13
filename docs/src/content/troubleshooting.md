---
title: Troubleshooting Guide
description: Solving common BoxMux problems - configuration errors, locked files, the control socket, scripts, plugins, display and clipboard issues, Windows specifics, and logging
---


## Installation Problems

### Rust Not Found

**Problem**: `cargo: command not found`

**Solution**: install Rust with rustup:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
. "$HOME/.cargo/env"
```

### Build Failures

**Problem**: `cargo install boxmux` or `cargo build` fails to compile.

**Solution**: BoxMux needs a recent stable Rust. Update the toolchain and build again:

```bash
rustup update stable
cargo build --release
```

If a build from a source checkout still fails, `cargo clean` removes stale build output.

## Configuration Errors

BoxMux checks the whole configuration before it touches the screen. If anything is wrong it prints the problems to stderr, prefixed with `boxmux:`, and exits with a non-zero status. Nothing is drawn, so the message stays readable in your shell.

### File Not Found

```
boxmux: cannot open 'dashbord.yaml': No such file or directory (os error 2)
```

Check the path. Relative paths are relative to the current directory.

### YAML Syntax Errors

A syntax error names the file, line and column, then shows the line with a caret under the column:

```
boxmux: /home/me/dashboard.yaml:7:18: did not find expected ',' or '}' at line 7 column 18, while parsing a flow mapping at line 6 column 21
7 |           content: hi
  |                  ^
```

The mistake is often on the line before the one shown. Here, line 6 opens a `{ ... }` mapping that is never closed. Other common causes are tabs used for indentation (YAML needs spaces), inconsistent indentation, and unquoted values that contain `: ` or start with a special character (`*`, `&`, `!`, `{`, `[`, `%`, `@`).

### Invalid Settings

Every key and value is checked against the schema, and every problem found is listed with its location. The location names each layout and box by its index and id:

```
boxmux: /home/me/dashboard.yaml: invalid configuration:
  app.layouts[0] ('main').children[1] ('logs').border_color: "BrightRed" is not one of ["default","black","red","green","yellow","blue","magenta","cyan","white","bright_black","bright_red","bright_green","bright_yellow","bright_blue","bright_magenta","bright_cyan","bright_white"]
  app.layouts[0] ('main').children[1] ('logs').refresh_interval: "fast" is not of type "integer"
  app.layouts[0] ('main').children[1] ('logs'): Additional properties are not allowed ('pty' was unexpected)
```

Frequent causes:

| Message | Cause and fix |
|---------|---------------|
| `Additional properties are not allowed ('pty' was unexpected)` | Unknown key. There is no `pty: true` or `thread: true`; use `execution_mode: Pty` or `execution_mode: Thread`. Check spelling against the [configuration reference](/docs/configuration). |
| `... is not one of ["default","black",...]` | Color names are lower case with underscores: `bright_red`, not `BrightRed`. |
| `position.x1: 10 is not of type "string"` | An unquoted number. Write a percentage (`10%`) or a quoted cell count (`"10"`). |
| `"2000" is not of type "integer"` | A number setting written as text, often through a variable (`refresh_interval: ${FAST}`). Variables always produce text; write the number directly. |
| `tab_order: "1" is not of type "integer"` | `tab_order` is a number: `tab_order: 1`. |
| `"position" is a required property` | Every box needs a `position`. |

### Inconsistent Configuration

After the schema check, BoxMux checks that the parts fit together:

```
boxmux: /home/me/dashboard.yaml: invalid configuration:
  app.layouts[0] ('main').children[0] ('a'): box id 'a' is also used at app.layouts[0] ('main').children[1] ('a')
  app.layouts[0] ('main').children[1] ('a').refresh_interval: a PTY box runs its script once as an interactive session; refresh_interval does not apply
  app.layouts[0] ('main').children[0] ('a').redirect_output: there is no box 'nowhere'
  app.hot_keys.F5: there is no choice 'nothing'
```

Other checks in this group:

- Box ids must be unique across all layouts, layout ids must be unique, and choice ids must be unique across the whole app.
- Only one layout can be `root`, and only one box per layout can be `selected`.
- `next_focus_id` must name a box in the same layout.
- `refresh_interval` needs a `script` (or a `plugin_component`) on the same box.
- `chart_type` needs `chart_data` or a script; `chart_data` needs a `chart_type`; `table_config` needs `table_data` or a script.
- A box can show only one of a plugin component, a chart or a table.
- A choice with `redirect_output` needs a `script`.
- `min_width` / `min_height` cannot be larger than `max_width` / `max_height`.

### Variables Not Replaced

`${NAME}` stays in the text when `NAME` is not defined in the box, its parents, the layout, the app or the environment, and has no default. Add the variable or a default (`${NAME:fallback}`). Shell forms such as `${NAME:-x}` are deliberately left for the shell. A `${...}` written inside a variable's own value is not expanded. See [Variables](/docs/variables).

## Starting BoxMux

### The File Is Already Open

```
boxmux: '/home/me/dashboard.yaml' is open in another boxmux instance; start this one with --lock to open it read-only
```

Without `--lock`, BoxMux saves moved and resized boxes into the YAML file, so it takes an exclusive lock on `<config>.lock` next to it. Only one such instance per file can run. Either close the other instance, or start this one with `--lock`: moving and resizing are then disabled and the file is never written.

The lock is held by the operating system and released when the process ends, even after a crash. A leftover `dashboard.yaml.lock` file does not block anything and can be deleted while BoxMux is not running.

If BoxMux cannot create the lock file (for example, the directory is read-only), it says `cannot create '<config>.lock': ...`. Use `--lock`, or copy the file somewhere writable.

### Another Instance Uses the Socket

```
boxmux: another boxmux instance is already listening on '/run/user/1000/boxmux.sock'; start this one with --socket <name>
```

Every instance listens on a control socket, by default `$XDG_RUNTIME_DIR/boxmux.sock` (or `/tmp/boxmux-<uid>.sock`) on Unix and the named pipe `boxmux-<username>` on Windows. To run two instances at once, give the second its own socket:

```bash
boxmux --socket /tmp/boxmux-logs.sock logs.yaml
```

A socket file left behind by an instance that crashed is replaced automatically.

### A Plugin Fails to Load

Plugins are loaded at start, and BoxMux does not start if one cannot be used:

```
boxmux: plugin /home/me/plugins/libgauge.so: /home/me/plugins/libgauge.so: cannot open shared object file: No such file or directory
boxmux: box 'cpu' uses plugin component 'gauge', which no loaded plugin offers
```

- Plugin paths are relative to the YAML file. A path without an extension becomes `libNAME.so` (Linux, FreeBSD), `libNAME.dylib` (macOS) or `NAME.dll` (Windows).
- Build the plugin before starting BoxMux. For the example plugin: `cargo build --release --manifest-path examples/plugins/gauge/Cargo.toml`.
- A plugin built for another plugin interface version, or two plugins offering the same component, are also refused at start.

See [Plugin System](/docs/plugin-system).

### BoxMux Starts Slowly

On Unix, BoxMux asks the terminal for its background color at start, to pick a light or dark theme. A terminal that answers nothing at all costs up to one second. Pass `--dark` or `--light` to skip the question.

## Control Commands

### Cannot Reach a BoxMux Instance

```
boxmux: cannot reach a boxmux instance at '/run/user/1000/boxmux.sock': No such file or directory (os error 2) (is boxmux running, and is BOXMUX_SOCKET or --socket set to its socket?)
```

- Check that BoxMux is running.
- The command must use the same socket as the instance. If the instance was started with `--socket NAME`, pass the same `--socket NAME` (or set `BOXMUX_SOCKET=NAME`).
- The default socket depends on `XDG_RUNTIME_DIR` and your user id. A command run under `sudo`, in another user's session, or over an SSH session without `XDG_RUNTIME_DIR` looks for a different socket; pass `--socket` explicitly.
- Scripts started by BoxMux have `BOXMUX_SOCKET` set to the right socket already.

### Errors from Commands

A command that reaches BoxMux but fails prints the reason and exits with a non-zero status, for example `boxmux: there is no box 'statsu'`. Interface options cannot be combined with a control command: `boxmux --lock replace_box_content ...` is rejected with a usage error (exit status 2). `send_signal` is not available on Windows; use `kill_pty_process` there. See the [API Reference](/docs/api).

## Scripts

### A Script Shows No Output or the Wrong Output

- Run the script in a shell first: `bash -c 'your script'` (or `powershell -Command '...'` on Windows). Scripts run with the app's `shell`, `bash` by default.
- Scripts run in the directory BoxMux was started from, not the directory of the YAML file. Use absolute paths, or `cd` at the start of the script.
- With the default `execution_mode: Immediate`, output appears only when the script ends. A script that never ends (`tail -f`, a `while true` loop) shows nothing. Use `execution_mode: Thread` to see lines as they arrive.
- A script that starts a background process (`server &`) counts as running until that process closes its output. Redirect the background process's output (`server >/dev/null 2>&1 &`).
- Commands that need a terminal (`top`, `htop`, `vim`, `less`, `ssh` asking for a password) need `execution_mode: Pty`.

### The Box Turns Red

The border and title turn red when the last run of the shown tab failed: a non-zero exit status, a script that could not start, or one ended by a signal. The output is still shown; read it for the error. `grep` that finds nothing, for example, exits with status 1; add `|| true` when that is not a failure.

### A Box Does Not Refresh

- `refresh_interval` is in milliseconds: `2000` is two seconds.
- A run that is still going is not started again, so a slow script refreshes less often than its interval.
- `stop_box_refresh` pauses a box until `start_box_refresh`.
- Box scripts start when their layout is first shown; scripts of other layouts wait until you switch to them.

### Key Bindings Do Nothing

- `q` always quits; a binding for `q` never runs.
- A box's `on_keypress` handlers run only while that box has the focus and its shown tab is not a menu.
- When the focused box shows a running terminal program, all keys except Tab and Shift+Tab go to the program, and no bindings run.
- Some terminals do not send certain combinations (for example Ctrl+Shift+letter, or F-keys taken by the terminal or desktop). Try another key.
- App and layout handlers run in the background and show no output; a failure appears as a red notice on the bottom row.

### Windows: Scripts Fail

On Windows, scripts run in PowerShell (`powershell -NoLogo -NoProfile -NonInteractive -Command ...`, or without `-NonInteractive` in a PTY) unless the app sets `shell`. Scripts written for bash (`ls -la`, `$(date)`, `grep`) fail there.

- Write the scripts in PowerShell, or
- set `shell: bash` when bash is on `PATH` (for example from Git for Windows), or `shell: pwsh` for PowerShell 7.

```yaml
app:
  shell: pwsh
  layouts:
    - id: 'main'
      children:
        - id: 'processes'
          title: 'Processes'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          refresh_interval: 5000
          script:
            - Get-Process | Sort-Object CPU -Descending | Select-Object -First 10 Name, CPU | Format-Table -AutoSize | Out-String -Width 200
```

Running a `.ps1` file from a script can be blocked by the PowerShell execution policy. Call it as `powershell -ExecutionPolicy Bypass -File .\script.ps1`, or set a policy that allows local scripts. Files in `libs` are dot-sourced into each script, so they must be PowerShell files when the shell is PowerShell.

## Display Problems

### Corrupted Display

If a program run by BoxMux resets the terminal modes, BoxMux restores raw mode and mouse capture on the next frame. Resizing the terminal window redraws the whole screen.

If BoxMux was killed (for example with `kill -9`) and your shell is left echoing mouse escape codes or not showing typed text, run `reset`.

### Colors Look Wrong

BoxMux chooses a light or dark theme: `--light` or `--dark` if given; otherwise, on Unix, the background color the terminal reports; otherwise the `COLORFGBG` environment variable; otherwise dark. On Windows the background color is not queried. If the wrong theme is chosen (common inside `tmux` or `screen`, which may not pass the question on), force it:

```bash
boxmux dashboard.yaml --dark
boxmux dashboard.yaml --light
```

Colors in the configuration are the 16 ANSI colors, so their exact shade comes from your terminal's palette. If `TERM` or the terminal does not support colors, nothing BoxMux sets will show; try another terminal emulator.

### Text Is Cut Off or a Box Shows "too small"

- `overflow_behavior: truncate` cuts lines with `…`; use `scroll` (the default) or `wrap` to see everything.
- Charts need an area of at least 20 columns by 5 rows inside the border; smaller ones show "too small".
- A terminal box needs room inside its border; in a box that is too small, starting the program fails with "box '...' is too small to show a terminal".

### Lines Shift or Leave Stray Characters Around Emoji or Arrows

BoxMux gives every character the columns described in [Text Width](configuration.md#text-width). A terminal that draws a character wider than that shifts the rest of its line:

- Arrows, `·`, `×`, `…` and box-drawing characters are East Asian *ambiguous*: BoxMux draws them one column wide. Set the terminal to draw them narrow (in WezTerm, `treat_east_asian_ambiguous_width_as_wide = false`, the default; outside a CJK locale in most others).
- A terminal that draws an emoji narrower than BoxMux does shows it followed by a blank, with everything else in place.
- On Windows, BoxMux's output reaches the terminal through the console host (ConPTY), which redraws the screen itself and drops combining marks and skin tones: `é` written as `e` + U+0301 shows as `e`, and 👍🏽 as 👍. Columns stay where BoxMux put them.

### The Mouse Does Not Work or Clicks Land in the Wrong Place

- Use a terminal that reports mouse events (most do). Inside `tmux`, enable `set -g mouse on`.
- If clicks are off by a cell, start BoxMux with `--calibrate`: it highlights the cell the terminal reports under the pointer.
- In a box showing a terminal program that turned on mouse reporting, clicks inside the box go to that program.

### Moved or Resized Boxes Are Not Saved

- With `--lock`, boxes cannot be moved or resized and nothing is saved.
- A red notice such as "box 'x' appears more than once in the file; its position was not saved" or "cannot find the position of box 'x' in the file" means BoxMux could not find exactly one `id: x` with a `position` in the file. This happens when a box's id or position comes from a YAML anchor or alias, or a variable.
- "cannot write '...'" means the file or its directory is not writable.

### Copy to Clipboard Does Nothing

Ctrl+C sends the text to the terminal with the OSC 52 escape sequence and shows "copied N lines"; the terminal puts it on the system clipboard. BoxMux cannot tell whether the terminal did.

- Some terminals support OSC 52 only after it is enabled in their settings, and some do not support it at all. Check your terminal's documentation for "OSC 52" or "clipboard access".
- Inside `tmux`, enable `set -g set-clipboard on`. Inside GNU `screen`, OSC 52 is usually not passed through.
- Over SSH, the local terminal still does the copying, so it works when the local terminal supports OSC 52.
- In a box showing a running terminal program, Ctrl+C goes to the program (usually stopping it). Move the focus to another box first.

## Debugging

### Write a Log

BoxMux writes a log only when asked:

```bash
boxmux dashboard.yaml --log-file boxmux.log --log-level debug
```

`--log-level` is `error`, `warn`, `info` (default), `debug` or `trace`, and requires `--log-file`. Follow the log from another terminal while BoxMux runs:

```bash
tail -f boxmux.log
grep -E 'ERROR|WARN' boxmux.log
```

### Check a Configuration

Starting BoxMux is the check: it validates the whole file before drawing anything and prints every problem it finds. To check a file without opening the interface for long, start it and press `q`.

### Test Scripts on Their Own

```bash
bash -c 'df -h | head -n 5'
echo $?    # 0 means BoxMux will not show the box as failed
```

### Inspect a Running Instance

```bash
# State of the process shown in a box
boxmux query_pty_status shell

# Replace a box's text to check that the socket works
boxmux replace_box_content status "socket works"
```

## Getting Help

### Before Asking

1. Read the error message: it names the location of the problem.
2. Reduce the configuration to the smallest one that still shows the problem.
3. Search the existing issues on GitHub.

### What to Include

- BoxMux version: `boxmux --version`
- Operating system: `uname -a`, or the Windows version
- Terminal emulator and version, and whether `tmux` or `screen` is involved
- The configuration (reduced), and the exact command line
- What you expected and what happened
- Error messages, and a log written with `--log-file ... --log-level debug`

### A Minimal Configuration

```yaml
app:
  layouts:
    - id: 'test'
      root: true
      children:
        - id: 'problem_box'
          title: 'Problem Box'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'This demonstrates the issue'
```

## Recovery

### Undo Saved Positions

Moving and resizing boxes rewrites only their `position` lines. Keep the configuration in version control (or keep a copy) to undo layout changes, and use `--lock` while you only want to look.

### Stop BoxMux

- Press `q` (move the focus out of a terminal box first, since terminal boxes receive `q`).
- From another terminal: `pkill -TERM -x boxmux`. BoxMux quits cleanly on SIGTERM, SIGHUP and SIGINT.
- Ctrl+Z suspends BoxMux on Unix; `fg` brings it back.

When BoxMux exits, it stops every process it started, including the processes those started. If BoxMux was killed with `kill -9`, nothing could clean up: look for leftover processes with `ps` and run `reset` if the terminal misbehaves.
