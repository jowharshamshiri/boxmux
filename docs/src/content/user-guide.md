---
title: User Guide
description: Building terminal interfaces with BoxMux, from a first box to menus, live output, interactive terminals, charts, tables and plugins
---


## Table of Contents

- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
- [Tabs and Streams](#tabs-and-streams)
- [Mouse and Keyboard](#mouse-and-keyboard)
- [Building an Interface](#building-an-interface)
- [Common Layout Patterns](#common-layout-patterns)
- [Interactive Terminals](#interactive-terminals)
- [Charts and Tables](#charts-and-tables)
- [Plugins](#plugins)
- [Controlling BoxMux from Scripts](#controlling-boxmux-from-scripts)
- [Complete Examples](#complete-examples)
- [Best Practices](#best-practices)
- [Techniques](#techniques)

## Quick Start

### Your First Interface

Save this as `hello.yaml`:

```yaml
app:
  variables:
    WELCOME_MSG: "Welcome to BoxMux!"
  layouts:
    - id: 'hello'
      root: true
      title: 'My First BoxMux App'
      children:
        - id: 'greeting'
          title: 'Hello World'
          position:
            x1: 25%
            y1: 40%
            x2: 75%
            y2: 60%
          content: '${WELCOME_MSG}'
          border_color: 'green'
```

Run it:

```bash
# Installed with cargo install
boxmux hello.yaml

# From a source checkout
cargo run --release -- hello.yaml
```

Press `q` to quit.

The file defines no background or text colors, yet the interface is readable: BoxMux picks a light or dark theme to match your terminal. To force one, pass `--light` or `--dark`:

```bash
boxmux hello.yaml --dark
boxmux hello.yaml --light
```

If the file has a mistake, BoxMux does not start. It prints what is wrong, and where, to stderr; see [Troubleshooting](/docs/troubleshooting#configuration-errors).

### Interactive Menu

A box with `choices` is a menu. Selecting an entry runs its script:

```yaml
app:
  layouts:
    - id: 'interactive'
      root: true
      title: 'Interactive Menu'
      children:
        - id: 'menu'
          title: 'Actions'
          position: {x1: 5%, y1: 10%, x2: 35%, y2: 90%}
          tab_order: 1
          choices:
            - id: 'hello'
              content: 'Say Hello'
              script: ['echo "Hello, World!"']
              redirect_output: 'output'
            - id: 'date'
              content: 'Show Date'
              script: ['date']
              redirect_output: 'output'
            - id: 'files'
              content: 'List Files'
              script: ['ls -la']
              redirect_output: 'output'
        - id: 'output'
          title: 'Output'
          position: {x1: 40%, y1: 10%, x2: 95%, y2: 90%}
          content: 'Pick an action on the left'
```

`tab_order: 1` makes the menu reachable with Tab, and because it is the only box with a `tab_order`, it has the focus when the interface starts. Use Up and Down to move the selection and Enter to run it, or click an entry. Each choice's output opens in its own tab of the `output` box.

## Core Concepts

### 1. Layouts and Boxes

A configuration has one `app` with one or more **layouts**. A layout is a screen of **boxes**; boxes can contain other boxes.

```yaml
app:
  layouts:                   # Required: at least one layout
    - id: 'main'             # Unique layout id
      root: true             # The layout shown at start (default: the first one)
      title: 'Main'          # Also used as the terminal window title
      children:              # The boxes of the layout
        - id: 'box1'         # Unique box id (unique across the whole app)
          title: 'Box'       # Shown in the top border
          position:          # Required for every box
            x1: 10%          # Left edge
            y1: 10%          # Top edge
            x2: 90%          # Right edge
            y2: 90%          # Bottom edge
```

Every key is checked when the file is loaded. A key BoxMux does not know, a value of the wrong type, a reference to a box that does not exist, or a duplicate id stops BoxMux with an error that names the place in the file.

### 2. Positioning

Each edge of `position` is a string in one of two forms:

- A **percentage** with up to two decimals, such as `50%` or `33.25%`. `0%` is the first cell and `100%` the last cell of the area the box sits in.
- A **number of cells**, such as `"12"`, counted from the start of that area and clamped to it. Quote it: an unquoted `12` is a YAML number and is rejected.

Positions are relative to the enclosing box (its outer edge, including the border) or, for top-level boxes, to the screen. If an edge pair is given in reverse order (`x1` larger than `x2`), the two are swapped.

```yaml
# Full screen
position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}

# Top half
position: {x1: 0%, y1: 0%, x2: 100%, y2: 50%}

# Left sidebar, 30 cells wide
position: {x1: "0", y1: 0%, x2: "29", y2: 100%}

# Centered window
position: {x1: 25%, y1: 25%, x2: 75%, y2: 75%}
```

`min_width`, `min_height`, `max_width` and `max_height` (in cells) limit the size a box gets from its position. When a limit changes the size, `anchor` decides which point of the box stays in place: `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`, `Center` (the default), `CenterTop`, `CenterBottom`, `CenterLeft` or `CenterRight`. The box is then kept inside its parent.

Boxes can overlap. `z_index` (default 0) decides which of two overlapping sibling boxes is drawn on top; a box is always drawn over its parent.

### 3. Content, Scripts and Refresh

A box can show static `content`, the output of a `script`, a menu of `choices`, or several of these as tabs.

```yaml
- id: 'live_box'
  title: 'Live Data'
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
  refresh_interval: 2000   # Run the script again every 2 seconds
  script:
    - date
    - uptime
```

- `script` is a list of lines, or one multi-line string. The lines are joined and run as one script by the app's `shell` (`bash` by default; `powershell` on Windows; `sh`, `zsh` and `pwsh` can be chosen). Standard output and standard error both appear in the box.
- Without `refresh_interval`, the script runs once, when its layout is first shown. With it, the script runs again every `refresh_interval` milliseconds. A run that is still going is not started a second time.
- A layout's `refresh_interval` is the default for the boxes in it that have a script.
- `append_output: true` adds each run's output to what is already shown instead of replacing it. `auto_scroll_bottom: true` keeps the newest lines in view.
- `save_in_file: 'output.txt'` writes the output of the box's script to a file (relative to the YAML file) after each run.
- Scripts run in the directory BoxMux was started from. They get two environment variables: `BOXMUX_BOX` (the box the output goes to) and `BOXMUX_SOCKET` (the control socket of this instance).

Text that fits in a box is centered in it. When it does not fit, `overflow_behavior` decides what happens:

| Value | Behavior |
|-------|----------|
| `scroll` (default) | Scrollbars appear on the border; scroll with the mouse or keys |
| `wrap` | Long lines break; only vertical scrolling |
| `truncate` | Lines are cut and end with `…`; no scrolling |

### 4. Execution Modes

`execution_mode` (on a box or a choice) decides how a script runs:

| Mode | Behavior |
|------|----------|
| `Immediate` (default) | Runs in the background; the output appears when the script finishes |
| `Thread` | Output appears line by line while the script runs |
| `Pty` | The script runs in a terminal inside the box, and you can type into it |

```yaml
- id: 'build_log'
  title: 'Build'
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 50%}
  execution_mode: Thread
  auto_scroll_bottom: true
  script:
    - cargo build 2>&1
```

When a run fails (non-zero exit status), the box shows it with its error colors: a red border and title unless `error_border_color` and the other `error_*` settings say otherwise.

### 5. Menus (Choices)

```yaml
- id: 'menu_box'
  title: 'Menu'
  position: {x1: 0%, y1: 0%, x2: 30%, y2: 100%}
  tab_order: 1                      # Reachable with Tab
  choices:
    - id: 'action1'                 # Unique across the whole app
      content: 'Menu Option 1'      # The label (the id is shown when missing)
      script: ['echo one']
      redirect_output: 'output_box' # Show the output in another box
    - id: 'action2'
      content: 'Menu Option 2'
      script: ['echo two']
      execution_mode: Thread
- id: 'output_box'
  title: 'Output'
  position: {x1: 30%, y1: 0%, x2: 100%, y2: 100%}
```

- Enter runs the selected entry; clicking an entry's label runs it.
- While a choice runs, its entry shows ` …`. Selecting it again while it runs does not start a second copy.
- When the script prints something, the output opens in a tab named after the choice: in the box named by `redirect_output`, or in the menu's own box. The tab does not take the box's view, so the menu stays on screen; click the tab to see the output. A script that prints nothing opens no tab; if it fails, a notice says so at the bottom of the screen.
- A choice with `execution_mode: Pty` is a terminal: its tab opens and is shown as soon as it starts, and selecting it again while it runs shows it.
- `append_output: true` on a choice appends each run to its tab.

### 6. Variables

Variables fill in `${NAME}` anywhere in the file:

```yaml
app:
  variables:
    SERVER_NAME: "db.example.com"
    DEFAULT_PORT: 5432
  layouts:
    - id: 'database_monitor'
      title: 'Database Monitor - ${SERVER_NAME}'
      children:
        - id: 'connection_status'
          title: 'Connection Status'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          script:
            - 'pg_isready -h ${SERVER_NAME} -p ${DEFAULT_PORT}'
            - 'echo "User: ${USER:unknown}"'
```

- `${NAME:default}` uses `default` when `NAME` is not defined anywhere.
- A name is looked up in the box's own `variables`, then the enclosing boxes (nearest first), the layout, the app, and finally the environment. A variable defined in the file therefore takes precedence over an environment variable of the same name.
- A reference to a name that is not defined anywhere and has no default is left as written, and so is shell syntax such as `${i}`, `${X:-y}` or `${#list[@]}`. In a script, the shell then expands it. Write `$${` for a literal `${`.
- Values may be strings, numbers or booleans. The result of a substitution is always text, so a variable cannot supply a number setting such as `refresh_interval`.
- Variable values are inserted as written: a `${...}` inside a value is not expanded.

Nested boxes see the variables of the boxes around them and can override them:

```yaml
app:
  variables:
    REGION: "us-east-1"
  layouts:
    - id: 'infrastructure'
      children:
        - id: 'web_tier'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          variables:
            SERVICE_TYPE: "frontend"
            PORT: "80"
          children:
            - id: 'nginx_server'
              position: {x1: 5%, y1: 10%, x2: 95%, y2: 90%}
              variables:
                SERVICE_NAME: "nginx"
                PORT: "443"   # Overrides web_tier's PORT
              title: '${SERVICE_NAME} (${SERVICE_TYPE}) - ${REGION}:${PORT}'
              # Title: "nginx (frontend) - us-east-1:443"
```

**Learn more**: [Variable System Guide](/docs/variables).

## Tabs and Streams

Everything a box can show is a **stream**, and a box with more than one stream shows them as tabs in its top border:

| Tab | Comes from | Can be closed |
|-----|------------|---------------|
| Content | The box's `content` | No |
| Choices | The box's `choices` | No |
| Output | The box's own `script` | No |
| One per choice | A choice whose output goes to this box, once it prints something (a terminal choice: once it starts) | Yes |
| One per key | A box `on_keypress` handler, once it prints something | Yes |
| Source box title | Another box's script redirected here with `redirect_output` | Yes |
| Socket | Text sent with `replace_box_content` | Yes |
| Terminal | A process started with `spawn_pty_process` | Yes |

A box with a single stream shows its `title` (or the stream's name) instead of tabs; in a box with only `choices`, the menu's tab is labelled with the box title.

- Click a tab to show it. Streams that are not shown keep receiving output.
- Click `×` on a tab to close it. Closing a tab stops the process that feeds it.
- When the tabs do not fit, `◀` and `▶` at the ends of the bar scroll it.
- A terminal program can set its tab's label with the usual window-title escape sequence.

```yaml
app:
  layouts:
    - id: 'dev'
      children:
        - id: 'dev_commands'
          title: 'Development'
          position: {x1: 0%, y1: 0%, x2: 30%, y2: 100%}
          tab_order: 1
          choices:
            - id: 'build'
              content: 'Build'
              script: ['cargo build --release 2>&1']
              execution_mode: Thread
              redirect_output: 'results'
            - id: 'test'
              content: 'Test'
              script: ['cargo test 2>&1']
              execution_mode: Thread
              redirect_output: 'results'
            - id: 'lint'
              content: 'Lint'
              script: ['cargo clippy 2>&1']
              redirect_output: 'results'
        - id: 'results'
          title: 'Results'
          position: {x1: 30%, y1: 0%, x2: 100%, y2: 100%}
          auto_scroll_bottom: true
          content: 'Results appear here'
```

The `results` box starts with its content. Running Build adds a "Build" tab once the build prints something, running Test adds a "Test" tab next to it, and so on; the box keeps showing what it showed until you click a tab. Each tab is reused when its choice runs again.

**Learn more**: [Stream Architecture](/docs/stream-architecture).

## Mouse and Keyboard

| Action | How |
|--------|-----|
| Focus a box | Click it, or Tab / Shift+Tab through boxes with a `tab_order` |
| Move the menu selection | Up / Down |
| Run the selected choice | Enter, or click the entry's label |
| Scroll | Mouse wheel over any box (3 lines per step), arrow keys, PageUp / PageDown, Home / End, Ctrl+Home / Ctrl+End |
| Move a box | Drag its top border |
| Resize a box | Drag its bottom-right corner (`⋱`) |
| Copy the focused box's text | Ctrl+C |
| Quit | `q` |

Moved and resized boxes are saved to the YAML file. Start BoxMux with `--lock` to turn moving and resizing off and never write the file.

When a box shows a running terminal program and has the focus, every key except Tab and Shift+Tab goes to that program, including `q` and Ctrl+C.

**Learn more**: [Interactive UI](/docs/interactive-ui).

## Building an Interface

### 1. Sketch the Layout

```
┌─────────────────────────────┐
│           Header            │
├─────────┬───────────────────┤
│ Sidebar │    Main Content   │
│         │                   │
│         │                   │
├─────────┴───────────────────┤
│           Footer            │
└─────────────────────────────┘
```

### 2. Write the Structure

```yaml
app:
  layouts:
    - id: 'dashboard'
      root: true
      title: 'My Dashboard'
      children:
        - id: 'header'
          title: 'Dashboard'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 15%}
          content: 'Welcome to the Dashboard'

        - id: 'sidebar'
          title: 'Navigation'
          position: {x1: 0%, y1: 15%, x2: 25%, y2: 85%}
          tab_order: 1
          choices:
            - id: 'system'
              content: 'System Info'
              script: ['uname -a']
              redirect_output: 'main'
            - id: 'processes'
              content: 'Processes'
              script: ['ps aux | head -10']
              redirect_output: 'main'

        - id: 'main'
          title: 'Main Content'
          position: {x1: 25%, y1: 15%, x2: 100%, y2: 85%}
          content: 'Select an option from the sidebar'

        - id: 'footer'
          title: 'Clock'
          position: {x1: 0%, y1: 85%, x2: 100%, y2: 100%}
          refresh_interval: 1000
          script: ['date']
```

### 3. Add Colors

Colors are optional; without them the theme supplies readable defaults (see [Theme](/docs/styling/theme)). The color names are `default` (the terminal's own color), `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white` and their `bright_` versions (`bright_black` through `bright_white`).

Colors set on a layout apply to all its boxes; colors set on a box apply to it and the boxes inside it, unless they set their own:

```yaml
- id: 'dashboard'
  root: true
  title: 'Styled Dashboard'
  bg_color: 'black'
  fg_color: 'white'
  title_fg_color: 'bright_yellow'
  border_color: 'green'
  selected_border_color: 'bright_cyan'   # Border of the focused box
  selected_menu_bg_color: 'blue'         # The menu entry Enter runs
  children:
    - id: 'note'
      position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
      content: 'Styled with the layout colors'
```

The settings are `fg_color`, `bg_color`, `border_color`, `title_fg_color`, `title_bg_color`, `menu_fg_color` and `menu_bg_color`, each with `selected_` (focused box), `highlighted_` (box under the mouse pointer), `error_` and `error_selected_` (last run of the shown tab failed) variants where the [configuration reference](/docs/configuration) lists them. `fill: false` leaves a box's background unpainted, so whatever is beneath shows through.

## Common Layout Patterns

These are the `children` of a layout; add content to each box.

### Header, Sidebar and Main Area

```yaml
children:
  - id: 'header'
    position: {x1: 0%, y1: 0%, x2: 100%, y2: 10%}
  - id: 'sidebar'
    position: {x1: 0%, y1: 10%, x2: 20%, y2: 100%}
  - id: 'main'
    position: {x1: 20%, y1: 10%, x2: 100%, y2: 100%}
```

### Two Columns

```yaml
children:
  - id: 'left'
    position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
  - id: 'right'
    position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
```

### 2x2 Grid

```yaml
children:
  - id: 'top_left'
    position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
  - id: 'top_right'
    position: {x1: 50%, y1: 0%, x2: 100%, y2: 50%}
  - id: 'bottom_left'
    position: {x1: 0%, y1: 50%, x2: 50%, y2: 100%}
  - id: 'bottom_right'
    position: {x1: 50%, y1: 50%, x2: 100%, y2: 100%}
```

### Fixed-Width Sidebar

Cell counts keep the sidebar 30 columns wide at any terminal size:

```yaml
children:
  - id: 'sidebar'
    position: {x1: "0", y1: 0%, x2: "29", y2: 100%}
  - id: 'main'
    position: {x1: "30", y1: 0%, x2: 100%, y2: 100%}
```

### Bounded Size

A dialog that takes half the screen, but never less than 40 by 10 cells nor more than 100 by 30:

```yaml
children:
  - id: 'dialog'
    position: {x1: 25%, y1: 25%, x2: 75%, y2: 75%}
    min_width: 40
    min_height: 10
    max_width: 100
    max_height: 30
    anchor: Center
```

## Interactive Terminals

With `execution_mode: Pty`, a script runs in a terminal inside the box. Use it for programs that need a terminal: editors, `htop`, shells, `ssh`, database consoles.

```yaml
# A box running htop
- id: 'system_monitor'
  title: 'System Monitor'
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
  execution_mode: Pty
  script:
    - htop

# A shell
- id: 'shell'
  title: 'Shell'
  position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
  tab_order: 1
  execution_mode: Pty
  script:
    - bash
```

Choices can open terminals too. Each one opens in its own tab:

```yaml
- id: 'admin_box'
  title: 'Administration'
  position: {x1: 0%, y1: 0%, x2: 30%, y2: 100%}
  tab_order: 1
  choices:
    - id: 'edit_config'
      content: 'Edit Config File'
      execution_mode: Pty
      script: ['vi /etc/hosts']
      redirect_output: 'workspace'
    - id: 'database_shell'
      content: 'Database Console'
      execution_mode: Pty
      script: ['psql -U postgres -d myapp']
      redirect_output: 'workspace'
- id: 'workspace'
  title: 'Workspace'
  position: {x1: 30%, y1: 0%, x2: 100%, y2: 100%}
  content: 'Pick a tool on the left'
```

- The terminal fills the inside of the box border and is resized with the box. Programs see `TERM=xterm-256color`.
- Focus the box (click it or Tab to it) to type into the program. Tab and Shift+Tab still move the focus.
- A PTY box runs its script once; `refresh_interval` is not allowed on it.
- When the program ends, its last screen stays. The border turns yellow if it exited normally or was stopped, and red if it failed. Closing its tab stops it.

| | Immediate | Thread | Pty |
|---|---|---|---|
| Output shown | When the script ends | Line by line | Live terminal screen |
| Keyboard input to the program | No | No | Yes |
| Full-screen programs (vim, htop) | No | No | Yes |
| Colors (SGR escape sequences) | Yes | Yes | Yes |
| Hyperlinks (OSC 8) | Yes | Yes | Yes |
| `refresh_interval` | Yes | Yes | No |

**Learn more**: [PTY Features](/docs/pty-features).

## Charts and Tables

### Charts

`chart_type` is `bar`, `line`, `histogram`, `pie` or `scatter`. The data is one point per line, written `label,value`, `label:value` or `label value`. It comes from `chart_data`, or from the output of the box's script when there is no `chart_data`:

```yaml
- id: 'requests_chart'
  title: 'Requests per Day'
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
  chart_type: bar
  chart_data: |
    Mon,120
    Tue,180
    Wed,95
    Thu,210
    Fri,160

- id: 'load_chart'
  title: 'Load Average'
  position: {x1: 50%, y1: 0%, x2: 100%, y2: 50%}
  chart_type: line
  refresh_interval: 5000
  script:
    - awk '{print "1min," $1; print "5min," $2; print "15min," $3}' /proc/loadavg
```

A chart needs an area of at least 20 columns by 5 rows inside the border and its one-column margins; a smaller box shows "too small".

### Tables

`table_data` holds CSV (the first row is the header) or a JSON array of objects; without it, the script's output is the data. `table_config` controls how it is shown:

```yaml
- id: 'process_table'
  title: 'Top Processes'
  position: {x1: 0%, y1: 50%, x2: 100%, y2: 100%}
  refresh_interval: 5000
  script:
    # Linux (procps) ps; prints CSV with a header row
    - ps -eo pid,pcpu,pmem,comm --sort=-pcpu | head -n 16 | awk '{print $1 "," $2 "," $3 "," $4}'
  table_config:
    show_headers: true
    zebra_striping: true
    show_row_numbers: true
    border_style: rounded      # none, single, double, rounded or thick
    page_size: 10
    show_page_info: true
```

The other `table_config` keys are `sort_column` (a header), `sort_ascending`, `filters` (column header to text the value must contain), `max_column_width` and `current_page`.

A box shows one chart, one table or one plugin component, not several.

**Learn more**: [Data Visualization](/docs/data-visualization).

## Plugins

A plugin is a shared library that draws the content of boxes. List the libraries under `app.plugins` and name a component in a box:

```yaml
app:
  plugins:
    # Relative to this file. Without an extension, the platform's name is used:
    # libboxmux_gauge.so, libboxmux_gauge.dylib or boxmux_gauge.dll.
    - path: plugins/boxmux_gauge
  layouts:
    - id: 'main'
      children:
        - id: 'gauges'
          title: 'Gauges'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 50%}
          plugin_component: 'gauge'
          plugin_config:          # Passed to the plugin as JSON
            gauges:
              - {label: CPU, value: 42}
              - {label: Memory, value: 6.1, max: 16}
```

BoxMux loads every plugin when it starts and refuses to start if a library cannot be loaded, speaks another plugin interface version, offers a component another plugin already offers, or if a box names a component no plugin offers. A plugin runs inside the BoxMux process with the same rights as BoxMux; only load libraries you trust. With `refresh_interval`, a plugin box is redrawn on that interval.

The repository has a working example in `examples/plugins/gauge` (build it with `cargo build --release --manifest-path examples/plugins/gauge/Cargo.toml`) and a layout that uses it, `layouts/plugin_demo.yaml`.

**Learn more**: [Plugin System](/docs/plugin-system).

## Controlling BoxMux from Scripts

A running BoxMux listens on a control socket. The same `boxmux` binary sends commands to it:

```bash
boxmux replace_box_content status "Deploy finished"
boxmux replace_box_script clock 'date +%T'
boxmux stop_box_refresh clock
boxmux switch_active_layout logs
boxmux send_pty_input shell $'ls -la\n'
```

Scripts started by BoxMux have `BOXMUX_SOCKET` set, so a command in a script or key handler reaches the instance that started it. From another terminal, the per-user default socket is used; pass `--socket NAME` (or set `BOXMUX_SOCKET`) to run several instances side by side.

**Learn more**: [API Reference](/docs/api).

## Complete Examples

### System Monitor (Linux)

```yaml
app:
  layouts:
    - id: 'sysmon'
      root: true
      title: 'System Monitor - ${USER:admin}'
      refresh_interval: 2000
      children:
        - id: 'load'
          title: 'Load'
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 30%}
          script:
            - uptime

        - id: 'memory'
          title: 'Memory'
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 30%}
          script:
            - free -h

        - id: 'disk'
          title: 'Disk Usage'
          position: {x1: 0%, y1: 30%, x2: 100%, y2: 60%}
          refresh_interval: 10000
          script:
            - df -h | head -n 8

        - id: 'processes'
          title: 'Top Processes'
          position: {x1: 0%, y1: 60%, x2: 100%, y2: 100%}
          script:
            - ps -eo pid,pcpu,pmem,comm --sort=-pcpu | head -n 11 | awk '{print $1 "," $2 "," $3 "," $4}'
          table_config:
            border_style: single
            zebra_striping: true
```

### Development Dashboard

```yaml
app:
  hot_keys:
    F5: 'build'
    F6: 'test'
  layouts:
    - id: 'dev_dash'
      root: true
      title: 'Development Dashboard'
      children:
        - id: 'git_status'
          title: 'Git Status'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 30%}
          refresh_interval: 10000
          script:
            - git status --short
            - echo "---"
            - git log --oneline -3

        - id: 'actions'
          title: 'Actions (F5 build, F6 test)'
          position: {x1: 0%, y1: 30%, x2: 30%, y2: 100%}
          tab_order: 1
          choices:
            - id: 'build'
              content: 'Build Project'
              script: ['cargo build 2>&1']
              execution_mode: Thread
              redirect_output: 'output'
            - id: 'test'
              content: 'Run Tests'
              script: ['cargo test 2>&1']
              execution_mode: Thread
              redirect_output: 'output'
            - id: 'lint'
              content: 'Run Linter'
              script: ['cargo clippy 2>&1']
              redirect_output: 'output'

        - id: 'output'
          title: 'Output'
          position: {x1: 30%, y1: 30%, x2: 100%, y2: 100%}
          content: 'Select an action'
          auto_scroll_bottom: true
```

### Log Viewer

```yaml
app:
  layouts:
    - id: 'logmon'
      root: true
      title: 'Log Viewer'
      children:
        - id: 'log_selector'
          title: 'Logs'
          position: {x1: 0%, y1: 0%, x2: 25%, y2: 100%}
          tab_order: 1
          choices:
            - id: 'recent'
              content: 'Last 50 lines'
              script:
                - tail -n 50 /var/log/syslog 2>/dev/null || journalctl -n 50 --no-pager
              redirect_output: 'log_viewer'
            - id: 'follow'
              content: 'Follow'
              execution_mode: Thread
              script:
                - tail -n 20 -f /var/log/syslog 2>/dev/null || journalctl -f -n 20 --no-pager
              redirect_output: 'log_viewer'

        - id: 'log_viewer'
          title: 'Log Contents'
          position: {x1: 25%, y1: 0%, x2: 100%, y2: 100%}
          content: 'Select a log'
          auto_scroll_bottom: true
```

"Follow" keeps running and adds lines as they arrive; close its tab to stop it.

More complete configurations are on the [Examples](/docs/examples) page and in the `layouts/` directory of the repository.

## Best Practices

### Planning

- Sketch the layout before writing YAML, then add boxes one at a time.
- Try the interface in a small terminal as well as a large one; percentages scale, cell counts do not.
- Give boxes descriptive ids (`cpu_monitor`, not `box1`): ids appear in error messages, socket commands and `BOXMUX_BOX`.

### Keep Scripts Cheap

```yaml
- id: 'fast_updates'
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
  refresh_interval: 1000     # Every second, for data that changes quickly
  script: ['date +%T']

- id: 'slow_updates'
  position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
  refresh_interval: 30000    # Every 30 seconds is enough for disk usage
  script: ['df -h | head -n 5']
```

Limit output (`head`, `tail`) and avoid commands that wait for input: a script that never ends keeps its box from refreshing. A script also counts as running while a background process it started (`command &`) still holds its output open; redirect such a process's output (`command >/dev/null 2>&1 &`) if the script should finish without it.

### Handle Missing Tools

```yaml
script:
  - |
    if command -v docker >/dev/null 2>&1; then
      docker ps
    else
      echo "Docker is not installed"
    fi
```

A script that exits with a non-zero status puts its box in the error colors, which is useful for checks: `systemctl is-active nginx` turns the box red when nginx is down.

### Make Navigation Obvious

```yaml
- id: 'menu1'
  position: {x1: 0%, y1: 0%, x2: 30%, y2: 50%}
  tab_order: 1               # First Tab stop, focused at start
  choices:
    - id: 'one'
      content: 'One'
- id: 'menu2'
  position: {x1: 0%, y1: 50%, x2: 30%, y2: 100%}
  tab_order: 2               # Second Tab stop
  choices:
    - id: 'two'
      content: 'Two'
```

Only boxes with a `tab_order` (an integer) are reached with Tab. If no box has one and none sets `selected: true`, nothing has the focus until you click a box.

## Techniques

### Conditional Content

```yaml
script:
  - |
    if systemctl is-active nginx >/dev/null 2>&1; then
      echo "Nginx: running"
    else
      echo "Nginx: stopped"
    fi
```

### Multi-line Scripts

```yaml
script:
  - |
    echo "System Information"
    echo "=================="
    echo "Hostname: $(hostname)"
    echo "Uptime:   $(uptime)"
```

### Shared Shell Functions

`libs` lists script files that are sourced before every script, relative to the YAML file:

```yaml
app:
  libs:
    - lib/helpers.sh
  layouts:
    - id: 'main'
      children:
        - id: 'status'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          script: ['show_status']   # A function defined in lib/helpers.sh
```

### Nested Boxes

Positions of nested boxes are relative to their parent:

```yaml
- id: 'main_container'
  title: 'Container'
  position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
  children:
    - id: 'left_section'
      position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
      children:
        - id: 'top_left'
          position: {x1: 5%, y1: 5%, x2: 95%, y2: 45%}
        - id: 'bottom_left'
          position: {x1: 5%, y1: 55%, x2: 95%, y2: 95%}
    - id: 'right_section'
      position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
```

### Key Bindings

`on_keypress` runs commands when a key is pressed. It can be set on the app, a layout or a box:

```yaml
app:
  on_keypress:
    F2: ['boxmux switch_active_layout main']
    F3: ['boxmux switch_active_layout logs']
  layouts:
    - id: 'main'
      root: true
      children:
        - id: 'clock'
          title: 'Clock (press r)'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          tab_order: 1
          on_keypress:
            r: ['date']
    - id: 'logs'
      children:
        - id: 'log'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          content: 'Logs'
```

- A box's handlers run while it has the focus and its shown tab is not a menu; their output opens in a tab named after the key.
- App and layout handlers run in the background; their output is not shown, and a failure appears as a notice at the bottom of the screen.
- Key names: single characters (`r`, `R`), `Enter`, `Esc`, `Tab`, `Shift+Tab`, `Backspace`, `Delete`, `Insert`, `Space`, `Up`, `Down`, `Left`, `Right`, `Home`, `End`, `PageUp`, `PageDown`, `F1` to `F12`, with `Ctrl+`, `Alt+` and `Shift+` prefixes (for example `Ctrl+r`, `Alt+x`, `Shift+Up`). Modifier and key names are not case-sensitive; single characters are.
- `q` always quits, so a binding for `q` never runs.
- `hot_keys` maps a key to a choice id and runs that choice, as if it were selected, when the choice is in the shown layout.

### Calling Web APIs

```yaml
script:
  - |
    curl -s "https://api.github.com/repos/rust-lang/rust" | jq -r '.stargazers_count'

```

---

For every setting, see the [Configuration Reference](/docs/configuration).
For the socket commands, see the [API Reference](/docs/api).
For problems, see [Troubleshooting](/docs/troubleshooting).
