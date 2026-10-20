---
title: Stream Architecture
description: How BoxMux organizes what a box shows into streams (tabs), where script output goes, and how the runtime's single event loop and executor run scripts
---

## Streams are tabs

Everything a box can show is held in **streams**, and each stream is one tab of the box. A
box shows one stream at a time: its **visible tab**.

Every stream has a kind:

| Kind | Holds | Created |
|------|-------|---------|
| Content | The box's static `content` | At load, when `content` is not empty |
| Choices | The box's `choices` menu | At load, when `choices` is not empty |
| Run | The output of one *origin* (see below): text lines, or a terminal screen for PTY runs | The box's own script tab at load (when the box has a `script`); every other one the first time its origin runs |
| Socket | Text sent with the `replace_box_content` control command | The first time that command targets the box |

Tabs appear in this order: Content, Choices, the box's own script output, then every
stream created while boxmux runs, in the order they were created.

### Origins

A Run stream belongs to one **origin**, the thing that started the run:

| Origin | Started by | Tab label |
|--------|-----------|-----------|
| Box script | The box's `script`, once or every `refresh_interval` | `Output` (or the box title, see below); in another box: the source box's title or id |
| Choice | Enter, a click or a `hot_keys` key on a choice with a `script` | The choice's `content` (its id when it has none) |
| Key handler | A box's `on_keypress` handler | The key name, e.g. `Ctrl+r` |
| Socket spawn | The `spawn_pty_process` control command | `Terminal` |

App- and layout-level `on_keypress` handlers have no stream: their output is not shown, and a
failure appears as a notice at the bottom of the screen.

Running the same origin again writes to the same tab instead of opening a new one. Runs of
different origins never share a tab, even when they write into the same box: two boxes whose
choices redirect to the same `log` box each get their own tab there.

A terminal program that sets a title (OSC 0 or 2) renames its tab to that title.

### Labels and the tab bar

When a box defines exactly one of `content`, `choices` and `script`, that tab is labelled with
the box's `title` (or `Content`, `Choices`, `Output` when there is no title). With more than
one, the tabs are labelled `Content`, `Choices` and `Output`.

The top border shows tabs when the box has more than one stream, or a stream that can be
closed. Otherwise it shows the box's `title`, or the label of its only stream. Click a tab to
show it; ◀ and ▶ scroll the tab bar when the tabs do not fit, and the last tab can always be
scrolled into view.

### Closeable tabs

| Stream | Has a `×` |
|--------|-----------|
| Content, Choices | No |
| The box's own script output | No |
| A box script redirected into another box (shown in that other box) | Yes |
| Choice, key handler and socket spawn output | Yes |
| Socket content | Yes |

Clicking `×` removes the tab and stops the process feeding it, if any. When the closed tab was
visible, the first tab becomes visible. The next run of the same origin creates the tab again.

### Which tab is visible

At start each box shows its first tab. A run started by the user (activating a choice,
pressing a handled key, `replace_box_script`, `spawn_pty_process`, `restart_pty_process`)
makes its tab visible and scrolls the tab bar to it. A box script started when its layout is
shown, or by its `refresh_interval`, does not switch tabs. `replace_box_content` makes the
socket tab visible.

### Status

Each stream records the state of its content:

| Status | Meaning |
|--------|---------|
| Idle | Static content, or nothing has run yet |
| Running | A run is producing output |
| Exited with a code | The last run ended with that exit code |
| Failed | The last run could not start, was ended by a signal not sent by boxmux, or was reported failed through `replace_box_content` with `success: false` |
| Killed | The last run was stopped by boxmux (tab closed, box removed or replaced, `kill_pty_process`, or a restart) |

When the **visible** tab's status is a non-zero exit or a failure, the box is drawn in its
error colors (`error_*` settings, red by default). A terminal tab whose program exited with
code 0 or was killed gets a yellow border. While a choice's run is active, its menu entry
shows ` …`. The `query_pty_status` control command reports the status of the visible tab.

## Where output goes

### redirect_output

By default a run writes into the box it belongs to. `redirect_output: <box id>` sends it to
another box instead:

- on a **box**, it applies to the box's `script` and to its `on_keypress` handlers;
- on a **choice**, it applies to that choice's script;
- for `spawn_pty_process`, the `redirect_output` field does the same.

The output then appears in a tab of the target box, labelled as described above. A box whose
script is redirected keeps its own (empty) `Output` tab.

The target must be an existing box; a choice can only have `redirect_output` when it has a
script. Both are checked when the file is loaded.

### How runs write to their tab

| `execution_mode` | While running | When the run ends |
|------------------|---------------|-------------------|
| `Immediate` (default) | The tab keeps its previous content | The whole output (stdout and stderr together) replaces the content, or is added after it with `append_output: true` |
| `Thread` | The tab is cleared when the run starts (not with `append_output: true`); each complete line of stdout or stderr is added as it arrives | Any last partial line is added |
| `Pty` | The tab shows the program's terminal: scrollback plus screen, or only the screen while the program uses the alternate screen | The last screen stays |

Box `on_keypress` handlers always run in `Thread` mode. Socket spawns always run in `Pty` mode.

Output is cleaned before it is shown: a carriage return goes back to the start of the line
(later text overwrites earlier text, as progress bars expect), backspace steps back, tabs
advance to the next 8-column stop, SGR color sequences are kept, and other control characters
and escape sequences are removed. Invalid UTF-8 is shown with replacement characters.

### append_output

`append_output: true` keeps earlier output and adds new output after it. It is read from the
**box** for the box's script and its key handlers, and from the **choice** for a choice's runs.
Socket spawns never append.

### auto_scroll_bottom

With `auto_scroll_bottom: true`, a box scrolls to its last line whenever its visible tab
receives output. Terminal tabs always do this.

### save_in_file

`save_in_file: <path>` writes the content of the box's script tab to a file after each run of
the box script, replacing the file, with color sequences removed. A relative path is relative
to the YAML file. It does not apply to `Pty` scripts or to other origins.

## The runtime

### One event loop

One thread owns the whole application state: the layouts, boxes, streams and runs. Every
other thread only sends it events over a single channel:

| Thread | Sends |
|--------|-------|
| Input reader | Keys, mouse events, paste, terminal resizes |
| Signal listener | Termination (SIGTERM, SIGHUP, SIGINT), suspend and resume on Unix; Ctrl+C, Ctrl+Break and console close on Windows |
| Control socket | One thread per connection forwards the command and waits up to 10 s for the answer |
| Run readers | Output lines of `Thread` runs, the end of every run (with the whole output of `Immediate` runs), and "screen changed" notifications from PTY sessions |

The loop takes the next event (or wakes up on time for a due refresh, an expiring notice or a
pending frame), handles it and any others already waiting, starts box scripts whose refresh
is due, and draws a frame if something changed. Frames are drawn at most once per
`--frame-delay` milliseconds (30 by default), and only the cells that changed are written to
the terminal.

Because only this thread touches the state, control commands, key presses and script output
are applied one after another, in the order they arrive.

### The executor

The executor starts runs and keeps track of the active ones:

- Every run executes in the app's `shell` (`bash -c` by default; `powershell -NoLogo
  -NoProfile -NonInteractive -Command` on Windows, without `-NonInteractive` for PTY runs so
  they can read what is typed), with the app's `libs` sourced first and the script lines joined
  by newlines.
- Non-PTY runs get no input (stdin is empty) and are detached from boxmux's terminal: on Unix
  each runs in a new session, as the leader of its own process group; on Windows in a new
  process group without a console window. stdout and stderr are read on background threads.
- PTY runs get a pseudo-terminal sized to the box (see [PTY Features](/docs/pty-features)).
- An origin has at most one active run. A box script whose previous run is still going when
  its refresh is due is skipped for that interval, and the refresh keeps its cadence.
- Stopping a run stops its whole process tree, including background processes the script
  started: on Unix `SIGTERM` to the process group, then `SIGKILL` after 500 ms; on Windows
  `taskkill /T`, then `taskkill /T /F`.
- A restart (`replace_box_script` while the old script runs, `restart_pty_process`) stops the
  run and starts the new one when the old one's end has been reported, so the two never
  write to the tab at the same time.
- When boxmux quits, every remaining process tree is killed at once.

### Layouts and runs

Box scripts start the first time their layout is shown (at start, or after
`switch_active_layout`). Once started, runs and periodic refreshes continue while another
layout is shown. Removing or replacing a box through the control socket stops everything
running for it.
