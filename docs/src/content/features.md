---
title: Features
description: Overview of what BoxMux does, with links to the pages that describe each part.
---


This page lists what BoxMux does today. Each section links to the page with the details.

## Configuration

- One YAML file describes the whole interface: `app` holds `layouts`, and layouts hold boxes,
  which can hold more boxes.
- The file is checked when it is loaded: against a JSON schema, then for consistency (unique
  ids, existing `redirect_output` and `next_focus_id` targets, `min_*` not larger than
  `max_*`, and so on). Unknown keys are errors. Errors name the location, for example
  `app.layouts[0] ('main').children[1] ('logs').border_color`; YAML syntax errors show the line
  and column.
- Variables: `${NAME}` and `${NAME:default}` are substituted in every string, with values from
  the box, its enclosing boxes, the layout, the app and the environment. See
  [Variables](/docs/variables).

See the [Configuration Reference](/docs/configuration).

## Layouts and boxes

- Several layouts per file. The layout marked `root: true` (or the first one) is shown at
  start; another can be shown with the `switch_active_layout` control command.
- Box positions (`x1`, `y1`, `x2`, `y2`) are percentages of the parent (`"50%"`, up to two
  decimals) or cell counts written as strings (`"12"`).
- `anchor`, `min_width`, `min_height`, `max_width` and `max_height` constrain a box's size
  around a fixed point.
- `z_index` decides which of two overlapping sibling boxes is drawn on top.
- `overflow_behavior`: `scroll` (default, with scrollbars on the border), `wrap` or `truncate`.
- `title_position`: `left`, `center` or `right`.
- Colors: 16 ANSI color names for borders, text, titles and menus, with separate settings for
  the focused box (`selected_*`), the box under the pointer (`highlighted_*`) and boxes whose
  last run failed (`error_*`). Colors are inherited from enclosing boxes and the layout.
- A light and a dark default theme. The theme follows the terminal background (asked with
  OSC 11 on Unix, or read from `COLORFGBG`), or is forced with `--light` / `--dark`.

## Scripts

- `script` runs in a shell (`bash` by default on Unix, PowerShell on Windows; set with
  `app.shell`).
- `refresh_interval` (milliseconds) reruns it; a run still in progress is not started again.
  Intervals are inherited from enclosing boxes and the layout.
- `execution_mode`: `Immediate` (default; output shown when the script ends), `Thread` (output
  streamed line by line) or `Pty` (an interactive terminal).
- `redirect_output` sends the output to another box, `append_output` appends instead of
  replacing, `auto_scroll_bottom` keeps the newest line in view, and `save_in_file` writes the
  output to a file.
- `libs` lists script files sourced before every script.
- Scripts get `BOXMUX_SOCKET` and `BOXMUX_BOX` in their environment.
- When BoxMux exits, every process it started is stopped (whole process groups on Unix,
  process trees on Windows).

## Menus, keys and tabs

- `choices` are menu entries. Enter or a click on the label runs the selected entry; its
  output opens a tab in the box (or in the `redirect_output` box), and the entry shows ` …`
  while it runs.
- `hot_keys` bind key names to choice ids; `on_keypress` binds key names to scripts in the
  app, a layout or a box.
- A box with several outputs shows them as tabs in its top border. Tabs opened by runs can be
  closed with `×`, which also stops their process.

See [Interactive UI](/docs/interactive-ui) and [Stream Architecture](/docs/stream-architecture).

## Embedded terminals

- `execution_mode: Pty` runs a program in a pseudo-terminal sized to the inside of the box.
- The focused terminal receives every key except Tab and Shift+Tab, and pasted text. Mouse
  events reach the program when it asks for mouse reporting.
- The emulator handles 256 and 24-bit color, the alternate screen, scroll regions, line drawing
  characters, wide characters, mouse modes and window titles (shown as the tab label), and
  keeps scrollback.
- Terminal processes can be started, stopped, restarted, queried and sent input through the
  control socket.

See [PTY Features](/docs/pty-features).

## Charts, tables and plugins

- Charts: `chart_type` `bar`, `line`, `histogram`, `pie` or `scatter`, with data from
  `chart_data` or the box's script output.
- Tables: `table_data` in CSV or JSON (or the script output), with `table_config` for
  sorting, filtering, paging, border styles, row numbers and column widths.
- Plugins: shared libraries with a C interface draw the content of boxes that set
  `plugin_component`.

See [Data Visualization](/docs/data-visualization) and [Plugin System](/docs/plugin-system).

## Mouse and keyboard

- Click a box to focus it, a menu label to run it, a tab to show it, `×` to close it.
- Drag a box's top border to move it, and its bottom-right corner (`⋱`) to resize it. The new
  position is written back into the YAML file, keeping comments and each edge's unit. `--lock`
  disables this.
- Click a scrollbar track to jump, drag its knob to scroll; the wheel scrolls the box under the
  pointer.
- Tab and Shift+Tab move the focus along `tab_order`; `next_focus_id` overrides the next box.
- Up and Down move the menu selection or scroll; Left and Right scroll sideways; Page Up and
  Page Down scroll a page (with Shift, sideways); Home and End go to the start and end of the
  lines; Ctrl+Home and Ctrl+End go to the top and bottom.
- Ctrl+C copies the text of the focused box's visible tab to the clipboard through the
  terminal (OSC 52).
- `q` quits.

## Control socket

- A running instance listens on a local socket (a Unix socket file, or a named pipe on
  Windows).
- Commands: `replace_box_content`, `replace_box_script`, `stop_box_refresh`,
  `start_box_refresh`, `replace_box`, `add_box`, `remove_box`, `switch_active_layout`,
  `spawn_pty_process`, `kill_pty_process`, `restart_pty_process`, `query_pty_status`,
  `send_pty_input` and `send_signal` (Unix only).
- Each is available as a `boxmux` subcommand and as a JSON request. Boxes added or replaced
  through the socket are checked like boxes in the YAML file.

See the [API Reference](/docs/api).

## Command line and diagnostics

- `--frame-delay` sets the minimum time between redraws; the screen is only redrawn when
  something changed.
- `--log-file` and `--log-level` write a log.
- `--calibrate` highlights the cell under the mouse pointer.
- Errors and short messages appear on the bottom row of the screen for a few seconds.

## Platforms

Linux (x86-64 and arm64), FreeBSD, macOS (same code as other Unix systems, not tested on
macOS machines) and Windows 10 or later (ConPTY; `send_signal` is not available).
