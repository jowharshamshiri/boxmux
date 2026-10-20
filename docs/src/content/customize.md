---
title: Customization Guide
description: How to customize BoxMux applications through YAML configuration, styling, and features
---


## YAML Configuration Structure

A BoxMux application is one YAML file. Everything sits under the `app` key. An app has one or more layouts, and each layout has boxes, which can contain more boxes:

```yaml
app:
  variables:
    APP_NAME: "My Application"
  layouts:
    - id: main
      root: true                 # Shown at start
      title: '${APP_NAME}'       # Terminal window title
      children:
        - id: welcome
          title: Welcome
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          content: 'Welcome to my app'

    - id: debug                  # A second layout
      children:
        - id: debug_info
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          content: 'Debug view'
```

Unknown keys are load errors, so a typo is reported instead of silently ignored. The [Configuration Reference](configuration) lists every key.

### Application-level Customization

```yaml
app:
  shell: bash                    # bash, sh, zsh, powershell or pwsh
  libs:
    - lib/helpers.sh             # Sourced before every script
  variables:
    ENVIRONMENT: production
    API_ENDPOINT: https://api.example.com
  hot_keys:
    F5: refresh                  # Key runs a choice
  on_keypress:
    Ctrl+l:                      # Key runs a script in the background
      - ./rotate-logs.sh
  layouts:
    - id: main
      children:
        - id: menu
          title: Menu
          position: {x1: 0%, y1: 0%, x2: 30%, y2: 100%}
          tab_order: 1
          choices:
            - id: refresh
              content: 'Refresh [F5]'
              script: ['curl -s ${API_ENDPOINT}/status']
```

Settings that are not part of the file are command-line options: `--frame-delay` (minimum milliseconds between redraws), `--lock` (no moving or resizing boxes), `--dark` / `--light` (force a theme), and `--log-file` / `--log-level`.

## Box Configuration & Styling

### Basic Box Properties

```yaml
- id: header
  title: 'System Monitor'
  position:
    x1: 0%       # Left edge
    y1: 0%       # Top edge
    x2: 100%     # Right edge
    y2: 20%      # Bottom edge
  content: 'All systems normal'
  border_color: bright_cyan
  bg_color: black
  fg_color: white
  title_position: left
```

Positions are percentages of the parent (`"50%"`, up to two decimals) or cell offsets written as quoted numbers (`"12"`). `min_width`, `max_width`, `min_height`, `max_height` and `anchor` limit the size. See [Position Configuration](configuration#position-configuration).

### Advanced Box Styling

```yaml
- id: styled_box
  title: 'Custom Box'
  position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
  content: 'Styled content'

  # Colors: the 16 ANSI color names, or default
  border_color: bright_blue
  bg_color: black
  fg_color: bright_white
  title_fg_color: bright_yellow

  # Colors while the box has focus, and while the mouse is over it
  selected_border_color: bright_green
  highlighted_border_color: cyan

  # Characters that fill empty cells
  fill_char: ' '
  selected_fill_char: '·'

  # Title placement: left, center or right
  title_position: left

  # Keyboard focus
  tab_order: 1
  next_focus_id: next_box

- id: next_box
  position: {x1: 0%, y1: 90%, x2: 100%, y2: 100%}
  tab_order: 2
  content: 'Tab moves here'
```

Every color setting is optional. Without them, the automatic light or dark theme gives readable colors. See [Visual Themes](styling/theme).

## Content Types & Behavior

### Script Execution Boxes

```yaml
- id: system_stats
  title: 'System Stats'
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
  script: |
    echo "Load: $(uptime | sed 's/.*load average: //')"
    echo "Disk: $(df -h / | tail -1)"
  refresh_interval: 2000          # Run again every 2 seconds
  execution_mode: Immediate       # Show the output when the script ends (the default)

- id: build_output
  title: 'Build'
  position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
  script:
    - cargo build 2>&1
  execution_mode: Thread          # Show output line by line as it arrives
  auto_scroll_bottom: true        # Follow the newest line
  save_in_file: build.log         # Also write the output to a file (relative to the YAML file)
  redirect_output: log_box        # Show the output in another box

- id: log_box
  title: Log
  position: {x1: 0%, y1: 50%, x2: 50%, y2: 100%}
  append_output: true
  content: 'Waiting for output'
```

A script is a single string or a list of commands. Lists are joined with newlines and run as one script. A failed script (non-zero exit) shows the box in its error colors.

### Interactive Menu Boxes

```yaml
- id: main_menu
  title: Actions
  position: {x1: 0%, y1: 0%, x2: 30%, y2: 100%}
  tab_order: 1
  choices:
    - id: deploy
      content: 'Deploy Application'
      script: ['./deploy.sh']
      execution_mode: Thread
      redirect_output: deployment_log

    - id: run_tests
      content: 'Run Tests'
      script: ['npm test']
      execution_mode: Pty          # Run in a terminal inside the box

    - id: view_logs
      content: 'View Logs'
      script: ['tail -f /var/log/app.log']
      execution_mode: Thread
  # Menu colors
  menu_fg_color: bright_green
  selected_menu_fg_color: black
  selected_menu_bg_color: bright_yellow

- id: deployment_log
  title: 'Deployment'
  position: {x1: 30%, y1: 0%, x2: 100%, y2: 100%}
  content: 'Deployment output appears here'
```

Up and Down move the selection, and Enter or a click runs the selected choice. Each run opens its own tab, which can be closed with its `×`; closing it stops the process.

### PTY (Interactive Terminal) Boxes

```yaml
- id: terminal
  title: 'Shell'
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  execution_mode: Pty
  tab_order: 1
  script: ['bash']
```

The terminal fills the inside of the border and resizes with the box. While the box has focus, all keys except Tab and Shift+Tab go to the program. When the program ends, the border turns yellow (or shows the error colors if it failed).

## Layout Management

### Multi-Layout Applications

An app can have several layouts. The one marked `root: true` is shown first. A script can switch layouts with the `switch_active_layout` control command. Scripts started by boxmux get the control socket in `BOXMUX_SOCKET`, so the command reaches the running instance (assuming `boxmux` is on your `PATH`):

```yaml
app:
  hot_keys:
    F2: show_monitoring
    F3: show_dashboard
  layouts:
    - id: dashboard
      root: true
      children:
        - id: nav
          title: 'Dashboard [F2: monitoring]'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          choices:
            - id: show_monitoring
              content: 'Open monitoring'
              script: ['boxmux switch_active_layout monitoring']

    - id: monitoring
      children:
        - id: back
          title: 'Monitoring [F3: dashboard]'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          choices:
            - id: show_dashboard
              content: 'Back to dashboard'
              script: ['boxmux switch_active_layout dashboard']
```

A hot key only runs a choice of the layout that is shown, so each layout above has its own choice.

### Nested Box Hierarchies

A box's `children` are positioned inside it. Their percentages refer to the parent box, including its border, so children usually start a little inside the edges:

```yaml
- id: main_container
  title: Main
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  children:
    - id: sidebar
      position: {x1: 1%, y1: 3%, x2: 25%, y2: 97%}
      children:
        - id: menu
          position: {x1: 5%, y1: 5%, x2: 95%, y2: 80%}
          choices:
            - id: show_uptime
              content: 'Uptime'
              script: ['uptime']
        - id: status
          position: {x1: 5%, y1: 80%, x2: 95%, y2: 95%}
          script: ['date']

    - id: content
      position: {x1: 25%, y1: 3%, x2: 99%, y2: 97%}
      content: 'Main content area'
```

Colors, `fill`, fill characters, `overflow_behavior`, `title_position`, `refresh_interval` and `variables` are inherited by nested boxes unless they set their own.

## Advanced Customization Features

### Variable System & Templates

```yaml
app:
  variables:
    APP_NAME: "DevOps Dashboard"
    VERSION: "1.0.0"
  layouts:
    - id: main
      children:
        - id: header
          variables:
            TITLE_TEXT: "Status"          # Box variables override outer ones
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 20%}
          title: '${APP_NAME} v${VERSION}'
          content: 'Welcome to ${TITLE_TEXT}'

        - id: api_status
          position: {x1: 0%, y1: 20%, x2: 100%, y2: 100%}
          script:
            - 'curl -s ${API_ENDPOINT:http://localhost:8080}/status'
```

`${NAME}` is looked up on the box, enclosing boxes, the layout, the app and then the environment. `${NAME:default}` gives a fallback. See the [Variable System Guide](variables).

### Data Visualization

```yaml
- id: data_table
  title: Servers
  position: {x1: 0%, y1: 0%, x2: 60%, y2: 50%}
  table_data: |
    Name,CPU,Memory,Status
    web-01,45,67,Running
    web-02,23,54,Running
    db-01,78,89,Warning
  table_config:
    sort_column: CPU
    sort_ascending: false
    zebra_striping: true
    border_style: rounded

- id: cpu_chart
  title: 'CPU Usage Over Time'
  position: {x1: 60%, y1: 0%, x2: 100%, y2: 50%}
  chart_type: line
  chart_data: |
    Mon,45
    Tue,52
    Wed,48
    Thu,65
    Fri,71
```

Without `table_data` or `chart_data`, the box's script output is the data. Chart types are `bar`, `line`, `histogram`, `pie` and `scatter`. See [Data Visualization](data-visualization).

### Content Overflow Handling

```yaml
- id: log_viewer
  title: Log
  position: {x1: 0%, y1: 50%, x2: 100%, y2: 100%}
  overflow_behavior: scroll       # scroll (default), wrap or truncate
  auto_scroll_bottom: true        # Follow new output
  vertical_scroll: 100            # Start at the bottom (percent)
  script:
    - journalctl -n 500 --no-pager
```

With `scroll`, scrollbars appear on the border when the content is larger than the box. `wrap` breaks long lines and scrolls vertically. `truncate` cuts the content and marks cut lines with `…`.

### Interactive Features

```yaml
app:
  hot_keys:
    Ctrl+d: deploy_action           # Key runs a choice in the shown layout
  layouts:
    - id: main
      children:
        - id: background_box
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          z_index: 1
          tab_order: 2
          content: 'Background'
          on_keypress:
            r: ['date']               # Runs while this box has focus; output opens in a tab

        - id: actions
          title: Actions
          position: {x1: 20%, y1: 20%, x2: 80%, y2: 80%}
          z_index: 10                 # Drawn above background_box
          min_width: 30               # Never narrower than 30 cells
          min_height: 6
          tab_order: 1
          choices:
            - id: deploy_action
              content: 'Deploy [Ctrl+d]'
              script: ['./deploy.sh']
```

The mouse works without configuration: click to focus and run choices, drag the top border to move a box, drag the bottom-right corner to resize it. Moves and resizes are saved to the YAML file unless boxmux runs with `--lock`.

## Color Schemes & Themes

BoxMux uses the 16 ANSI colors, written in lower case with underscores, plus `default`:

| Normal | Bright |
|--------|--------|
| `black` | `bright_black` |
| `red` | `bright_red` |
| `green` | `bright_green` |
| `yellow` | `bright_yellow` |
| `blue` | `bright_blue` |
| `magenta` | `bright_magenta` |
| `cyan` | `bright_cyan` |
| `white` | `bright_white` |

There is no separate theme file. To apply a scheme, set the colors on the layout; every box inherits them. Use variables to name the colors you reuse:

```yaml
app:
  variables:
    PRIMARY: bright_blue
    SUCCESS: bright_green
    WARNING: bright_yellow
  layouts:
    - id: main
      border_color: '${PRIMARY}'
      title_fg_color: bright_white
      children:
        - id: success_box
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
          border_color: '${SUCCESS}'
          fg_color: '${SUCCESS}'
          content: 'All checks passed'
        - id: warning_box
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
          border_color: '${WARNING}'
          fg_color: '${WARNING}'
          content: '2 warnings'
```

## Performance & Optimization

### Refresh Rate Tuning

```yaml
- id: fast_updates
  position: {x1: 0%, y1: 0%, x2: 33%, y2: 100%}
  refresh_interval: 500             # Every half second
  script: ['date +%T']

- id: slow_updates
  position: {x1: 33%, y1: 0%, x2: 66%, y2: 100%}
  refresh_interval: 60000           # Every minute
  script: ['df -h']

- id: on_demand
  position: {x1: 66%, y1: 0%, x2: 100%, y2: 100%}
  script: ['uname -a']              # No refresh_interval: runs once
```

- Each refresh starts a new process, so use the longest interval that is still useful.
- A run that is still going is not started again; missed intervals are skipped.
- A layout's `refresh_interval` is the default for all its boxes that have a script.
- `--frame-delay <MS>` sets the minimum time between two redraws (default 30). A larger value draws less often.

## Plugin Integration

Plugins are shared libraries that draw box content. Load them under `app.plugins`, and name a component in a box:

```yaml
app:
  plugins:
    - path: plugins/boxmux_gauge   # Extension added for the platform
  layouts:
    - id: main
      children:
        - id: custom_component
          title: Gauges
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 50%}
          plugin_component: gauge
          plugin_config:
            gauges:
              - {label: CPU, value: 42}
```

boxmux refuses to start when a plugin is missing or a box names a component no plugin offers. See [Plugin System](plugin-system).
