---
title: Configuration Reference
description: Complete reference for BoxMux YAML configuration files - file structure, layouts, boxes, positioning, choices, and all configuration options
---


## Table of Contents

- [File Structure](#file-structure)
- [Application Configuration](#application-configuration)
- [Layout Configuration](#layout-configuration)
- [Box Configuration](#box-configuration)
- [Position Configuration](#position-configuration)
- [Content and Tabs](#content-and-tabs)
- [Choice Configuration](#choice-configuration)
- [Script Configuration](#script-configuration)
- [PTY Configuration](#pty-configuration)
- [Keyboard Configuration](#keyboard-configuration)
- [Focus and Navigation](#focus-and-navigation)
- [Overflow and Scrolling](#overflow-and-scrolling)
- [Color Reference](#color-reference)
- [Chart Configuration](#chart-configuration)
- [Table Configuration](#table-configuration)
- [Plugin Configuration](#plugin-configuration)
- [Layering](#layering)
- [Mouse and Clipboard](#mouse-and-clipboard)
- [Variable System](#variable-system)
- [Validation](#validation)
- [Examples](#examples)
- [Common Patterns](#common-patterns)

## File Structure

A BoxMux configuration is one YAML file with a single top-level key, `app`:

```yaml
app:
  layouts:            # Required: one or more layouts
    - id: main
      children:
        - id: hello
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          content: 'Hello'
  # libs:             # Optional: script files sourced before every script
  # variables:        # Optional: values for ${NAME} substitution
  # hot_keys:         # Optional: keys that run menu choices
  # on_keypress:      # Optional: keys that run scripts
  # shell:            # Optional: the shell scripts run in
  # plugins:          # Optional: plugin libraries
```

Every object accepts only the keys listed in this reference. An unknown or misspelled key is a load error; it is never ignored.

Run a configuration with:

```bash
boxmux my-config.yaml
```

## Application Configuration

### Root Application (`app`)

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `layouts` | `array[Layout]` | Yes | - | The layouts. At least one is required. |
| `libs` | `array[string]` | No | - | Script files sourced before every script. Relative paths are relative to the YAML file. |
| `variables` | `object` | No | - | Variables for `${NAME}` substitution in the whole file. |
| `hot_keys` | `object` | No | - | Key name to choice id. The key runs that choice. |
| `on_keypress` | `object` | No | - | Key name to script lines. Runs in the background; output is not shown. |
| `shell` | `string` | No | `bash` (`powershell` on Windows) | The shell scripts run in: `bash`, `sh`, `zsh`, `powershell` or `pwsh`. |
| `plugins` | `array[{path}]` | No | - | Plugin libraries that draw `plugin_component` boxes. |

```yaml
app:
  shell: bash
  libs:
    - lib/utils.sh
  variables:
    APP_NAME: "BoxMux Dashboard"
  hot_keys:
    F5: refresh_status
  layouts:
    - id: main
      title: '${APP_NAME}'
      children:
        - id: actions
          title: Actions
          position: {x1: 0%, y1: 0%, x2: 30%, y2: 100%}
          tab_order: 1
          choices:
            - id: refresh_status
              content: 'Status [F5]'
              script: [uptime]
```

## Layout Configuration

A layout is one screen of boxes. The layout with `root: true` is shown at start; without one, the first layout is shown. At most one layout may be `root`.

### Layout Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `id` | `string` | Yes | - | Unique layout id. |
| `title` | `string` | No | - | Shown as the terminal window title while the layout is shown. |
| `root` | `boolean` | No | `false` | Show this layout at start. |
| `children` | `array[Box]` | No | - | The top-level boxes. |
| `refresh_interval` | `integer` | No | - | Default `refresh_interval` (milliseconds) for the boxes of the layout. |
| `fill` | `boolean` | No | `true` | `false` leaves the terminal's own background behind the boxes. Also the default for boxes. |
| `fill_char` | `char` | No | `' '` | Character that fills the screen behind the boxes. Also the default for boxes. |
| `selected_fill_char`, `highlighted_fill_char` | `char` | No | - | Defaults for the boxes of the layout. |
| `overflow_behavior` | `string` | No | `scroll` | Default for the boxes of the layout. |
| `title_position` | `string` | No | `center` | Default for the boxes of the layout. |
| `on_keypress` | `object` | No | - | Key name to script lines, active while the layout is shown. |
| `variables` | `object` | No | - | Variables for the strings inside this layout. |
| Color settings | `string` | No | theme | Every color setting listed under [Color Reference](#color-reference). Boxes use them when they do not set their own. |

The layout's `fg_color` and `bg_color` also color the screen behind the boxes.

### Example Layout

```yaml
app:
  layouts:
    - id: dashboard
      root: true
      title: 'My Dashboard'
      border_color: green
      title_fg_color: bright_yellow
      children:
        - id: header
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 20%}
          content: 'Welcome'
```

## Box Configuration

Boxes are the building blocks of a layout. A box can show static text, a menu, script output, a terminal, a chart, a table or plugin output, and it can contain other boxes.

### Box Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `id` | `string` | Yes | - | Box id, unique across the whole file. |
| `position` | `Position` | Yes | - | Edges of the box. See [Position Configuration](#position-configuration). |
| `title` | `string` | No | - | Text in the top border. |
| `anchor` | `string` | No | `Center` | The point that stays in place when `min_*`/`max_*` change the size. |
| `min_width`, `min_height`, `max_width`, `max_height` | `integer` | No | - | Size limits in cells (at least 1). |
| `children` | `array[Box]` | No | - | Boxes inside this box. |
| `content` | `string` | No | - | Static text. |
| `choices` | `array[Choice]` | No | - | A menu. See [Choice Configuration](#choice-configuration). |
| `script` | `string` or `array[string]` | No | - | Script whose output the box shows. |
| `execution_mode` | `string` | No | `Immediate` | `Immediate`, `Thread` or `Pty`. |
| `refresh_interval` | `integer` | No | inherited | Milliseconds between runs of the script (at least 1). |
| `redirect_output` | `string` | No | - | Box id that shows the output of this box's script and key handlers. |
| `append_output` | `boolean` | No | `false` | Add new output after the old output instead of replacing it. |
| `save_in_file` | `string` | No | - | File the script output is written to after each run. |
| `auto_scroll_bottom` | `boolean` | No | `false` | Scroll to the bottom when new output arrives. |
| `on_keypress` | `object` | No | - | Key name to script lines, active while the box has focus. |
| `variables` | `object` | No | - | Variables for this box and the boxes inside it. |
| `tab_order` | `integer` | No | - | Position in the Tab focus order (0 or more). Boxes without one are skipped by Tab. |
| `next_focus_id` | `string` | No | - | Box that Tab moves to from this box. |
| `selected` | `boolean` | No | `false` | Give this box the focus when the layout is first shown. |
| `overflow_behavior` | `string` | No | inherited, else `scroll` | `scroll`, `wrap` or `truncate`. |
| `horizontal_scroll`, `vertical_scroll` | `number` | No | `0` | Initial scroll position, in percent (0 to 100). |
| `title_position` | `string` | No | inherited, else `center` | `left`, `center` or `right`. |
| `z_index` | `integer` | No | `0` | Drawing order among sibling boxes. Higher is on top. |
| `fill` | `boolean` | No | inherited, else `true` | `false` leaves the background unpainted, so what is beneath shows through. |
| `fill_char`, `selected_fill_char`, `highlighted_fill_char` | `char` | No | inherited, else `' '` | Character that fills empty cells, normally, when focused, and under the mouse pointer. |
| `chart_type`, `chart_data` | `string` | No | - | See [Chart Configuration](#chart-configuration). |
| `table_data`, `table_config` | `string`, `object` | No | - | See [Table Configuration](#table-configuration). |
| `plugin_component`, `plugin_config` | `string`, `object` | No | - | See [Plugin Configuration](#plugin-configuration). |
| Color settings | `string` | No | inherited, else theme | See [Color Reference](#color-reference). |

"Inherited" means the value comes from the nearest enclosing box that sets it, else from the layout.

A box always has a border when it is at least 2 by 2 cells. There is no setting to turn the border off.

## Position Configuration

A box's `position` has four edges, `x1`, `y1`, `x2` and `y2`. Each edge is a string in one of two forms:

- A percentage with up to two decimals: `"50%"`, `"33.25%"`. `0%` is the first cell of the parent and `100%` is its last cell.
- A cell count: `"12"`. This is an offset in cells from the parent's first cell. It is clamped to the parent's last cell.

Write cell counts in quotes. An unquoted `12` is a YAML number, and the loader rejects it because positions must be strings. Unquoted percentages such as `50%` are already strings in YAML.

Positions are relative to the parent box, including its border, or to the screen for top-level boxes. Both edges are part of the box, so `x1: 0%` and `x2: 50%` cover the first half of the columns plus the middle column. If an edge is given before its opposite edge (for example `x2` left of `x1`), the two are swapped.

### Position Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `x1` | `string` | Yes | Left edge |
| `y1` | `string` | Yes | Top edge |
| `x2` | `string` | Yes | Right edge |
| `y2` | `string` | Yes | Bottom edge |

### Position Formats

```yaml
# Percentages of the parent
- id: percent_box
  position:
    x1: 10%
    y1: 20%
    x2: 90%
    y2: 80%

# Cell offsets inside the parent (quoted)
- id: cell_box
  position: {x1: "2", y1: "1", x2: "40", y2: "10"}

# Mixed units: three rows high, half the width
- id: mixed_box
  position: {x1: "0", y1: "0", x2: 50%, y2: "2"}
```

Boxes that share an edge value, such as `x2: 50%` on one box and `x1: 50%` on the next, overlap by one column, so their borders sit on the same column.

### Size Limits and Anchor

`min_width`, `min_height`, `max_width` and `max_height` (in cells) grow or shrink the box after its position is worked out. The `anchor` is the point that stays in place: `TopLeft`, `TopRight`, `BottomLeft`, `BottomRight`, `Center` (the default), `CenterTop`, `CenterBottom`, `CenterLeft` or `CenterRight`. After resizing, the box is moved back inside its parent if it left it. A minimum larger than the maximum is a load error.

```yaml
- id: dialog
  position: {x1: 30%, y1: 30%, x2: 70%, y2: 70%}
  min_width: 40
  max_height: 12
  anchor: Center
  content: 'At least 40 columns wide, at most 12 rows high'
```

### Position Examples

```yaml
- id: full_screen
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}

- id: top_half
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 50%}

- id: left_sidebar
  position: {x1: 0%, y1: 0%, x2: 20%, y2: 100%}

- id: bottom_rows   # the last three rows
  position: {x1: 0%, y1: 100%, x2: 100%, y2: 100%}
  min_height: 3
  anchor: BottomLeft
```

## Content and Tabs

A box shows one tab for each source of content it has, in this order:

- **Content**: the `content` text.
- **Choices**: the `choices` menu.
- **Output**: the output of the box's `script`.

A box with one tab shows its `title` in the top border. A box with several tabs shows them in the top border instead; click a tab to switch to it. Runs started later get their own tab when they print something (a terminal run: when it starts), which can be closed with its `×`. These include choice runs, box key handler runs, output redirected from other boxes, and processes started through the control socket. Closing such a tab stops its process. When the tabs do not fit, `◀` and `▶` arrows scroll the tab bar.

Text that fits in the box is centered horizontally and vertically. The content area keeps a one-column margin inside the border. A terminal (`execution_mode: Pty`) fills the whole inside of the border.

Script output keeps its SGR color sequences, so colored output is shown in color. Other control sequences are removed.

## Choice Configuration

Choices make a menu inside a box.

### Choice Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `id` | `string` | Yes | - | Choice id, unique across the whole file. |
| `content` | `string` | No | the id | The label shown in the menu. |
| `script` | `string` or `array[string]` | No | - | Script run when the choice is activated. |
| `execution_mode` | `string` | No | `Immediate` | `Immediate`, `Thread` or `Pty`. |
| `redirect_output` | `string` | No | - | Box id that shows the output. Requires a `script`. |
| `append_output` | `boolean` | No | `false` | Add new output after the old output instead of replacing it. |

Up and Down move the selection (marked with `►`) and wrap around. Enter, or a click on a label, runs the selected choice. While the script runs, the label ends with ` …`. Activating a choice that is still running does not start it again.

What a choice's run shows depends on its `execution_mode`:

- `Immediate` and `Thread`: the output opens in its own tab, labelled with the choice's label, in the box itself or in the `redirect_output` box, when the script first prints something (an `Immediate` script prints when it ends). The tab opens beside the others and does not take the box's view: click it to see the output. A script that prints nothing opens no tab, and the box keeps showing what it showed; if it fails, a notice at the bottom of the screen says so.
- `Pty`: the choice's terminal opens in its own tab when the choice starts and is shown at once, since it takes the keyboard. Activating the choice while its program runs shows the terminal again.

### Choice Example

```yaml
choices:
  - id: deploy
    content: 'Deploy Application'
    script:
      - echo 'Starting deployment...'
      - ./deploy.sh
    execution_mode: Thread
    redirect_output: status
    append_output: true

  - id: check_status
    content: 'Check Status'
    script:
      - systemctl status myapp
    redirect_output: status
```

## Script Configuration

### Script Format

A script is either a single string or a list of strings:

```yaml
# A list: one command per element
- id: time
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
  script:
    - echo 'Current time:'
    - date

# A single string: kept exactly as written, including indentation and blank lines
- id: log_sizes
  position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
  script: |
    for f in *.log; do
      wc -l "$f"
    done
```

The elements of a list are joined with newlines and run as one script, in one shell process. The script runs in the shell set by `app.shell`, in the directory boxmux was started from. The files listed in `app.libs` are sourced first. Standard output and standard error are both shown.

Every script gets two extra environment variables: `BOXMUX_SOCKET`, the name of the control socket, and `BOXMUX_BOX`, the id of the box that shows the output. A script can use them to call `boxmux` control commands, such as `boxmux replace_box_content`.

### Execution Modes

| `execution_mode` | Behaviour |
|------------------|-----------|
| `Immediate` (default) | Runs in the background. The output is shown when the script finishes. |
| `Thread` | Runs in the background. The output is shown line by line as it arrives. |
| `Pty` | Runs in a pseudo-terminal shown live in the box. See [PTY Configuration](#pty-configuration). |

A script that exits with a non-zero status puts its tab in the error state, which shows the error colors (a red border by default).

### Running on a Schedule

| Property | Description |
|----------|-------------|
| `refresh_interval` | Milliseconds between runs. Inherited from the nearest enclosing box, else from the layout. |

A box script runs once when its layout is first shown. With a `refresh_interval`, it runs again at that interval while its layout is shown; while another layout is shown it waits, and when its layout is shown again it runs right away if an interval passed. A run that is still in progress is not started a second time; the missed intervals are skipped.

`refresh_interval` needs a `script`, a `plugin_component` (which is then redrawn at that interval), or child boxes that inherit it. A box with `execution_mode: Pty` cannot set `refresh_interval`; an inherited interval does not apply to it.

### Output Settings

| Property | Description |
|----------|-------------|
| `redirect_output` | Show the output in another box, in a tab labelled with this box's title (or id). |
| `append_output` | Add each run's output after the previous output. |
| `save_in_file` | After each run, write the output (without colors) to this file. The file is overwritten. A relative path is relative to the YAML file. |
| `auto_scroll_bottom` | Keep the view at the bottom as output arrives. |

```yaml
- id: clock
  title: Clock
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 30%}
  refresh_interval: 1000
  script:
    - date '+%H:%M:%S'

- id: build_log
  title: Build
  position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
  execution_mode: Thread
  auto_scroll_bottom: true
  save_in_file: build.log
  script:
    - cargo build 2>&1
```

## PTY Configuration

`execution_mode: Pty` runs the script in a pseudo-terminal. Interactive programs such as `htop`, `vim` or a shell work in the box.

```yaml
- id: monitor
  title: 'System Monitor'
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  execution_mode: Pty
  tab_order: 1
  script:
    - htop
```

- The terminal is the inside of the border and follows the box's size.
- While the box has focus, every key except Tab and Shift+Tab goes to the program. This includes `q`.
- Mouse events go to the program only when it turned on mouse reporting.
- A title the program sets becomes the tab label.
- When the program ends with status 0, or is stopped, the border turns yellow. A failure shows the error colors.

A choice can also use `execution_mode: Pty`; its terminal opens, and is shown, in a tab of the box or of its `redirect_output` box when the choice starts.

```yaml
choices:
  - id: edit_config
    content: 'Edit Configuration'
    execution_mode: Pty
    script:
      - vim config.yaml
```

## Keyboard Configuration

### Hot Keys

`app.hot_keys` maps a key name to a choice id. Pressing the key runs that choice, as if it were selected and Enter pressed. The choice must exist (this is checked at load) and must be in the layout that is shown; otherwise a notice says so.

```yaml
app:
  hot_keys:
    F1: build
    Ctrl+t: test
  layouts:
    - id: main
      children:
        - id: actions
          title: Actions
          position: {x1: 0%, y1: 0%, x2: 40%, y2: 100%}
          choices:
            - id: build
              content: 'Build [F1]'
              script: ['cargo build']
            - id: test
              content: 'Test [Ctrl+t]'
              script: ['cargo test']
```

### Key Handlers

`on_keypress` maps a key name to script lines. It can be set on the app, on a layout and on a box.

- App handlers run on every layout. Layout handlers run while their layout is shown. Their output is not shown; a failure appears as a notice.
- Box handlers run while the box has focus and its visible tab is not a menu. Their output opens in a tab named after the key, in the box or in its `redirect_output` box.

```yaml
on_keypress:
  r:
    - echo 'Refreshing...'
    - date
  Ctrl+l:
    - clear-logs.sh
```

All matching bindings run, at every level. They run in addition to what boxmux itself does with the key. The exception is `q` without modifiers, which always quits. A focused box showing a live terminal receives the keys instead, so no bindings run.

### Key Names

| Key | Name |
|-----|------|
| Letters, digits, symbols | The character itself: `a`, `A`, `5`, `?` |
| Space | `Space` |
| Special keys | `Enter`, `Tab`, `Shift+Tab`, `Backspace`, `Delete`, `Insert`, `Esc`, `Home`, `End`, `PageUp`, `PageDown`, `Up`, `Down`, `Left`, `Right` |
| Function keys | `F1`, `F2`, and so on |
| Modifiers | `Ctrl+`, `Alt+`, `Cmd+`, `Shift+` before the key: `Ctrl+r`, `Alt+x`, `Shift+Up` |

Modifier names and special key names are matched without regard to case, and modifiers may be given in any order. A single character must match exactly: `a` and `A` are different keys. Shift is part of the character for printable keys, so write `A`, not `Shift+a`.

## Focus and Navigation

| Property | Description |
|----------|-------------|
| `tab_order` | Position in the Tab order (an integer). Boxes with equal values follow the order they appear in the file. |
| `next_focus_id` | Box that Tab moves to from this box. It must be in the same layout. |
| `selected` | The box that has focus when the layout is first shown. At most one per layout. |

Tab moves the focus to the next box in `tab_order`, wrapping around. Shift+Tab moves backwards and does not follow `next_focus_id`. Boxes without `tab_order` are skipped by Tab, but can still be focused with a click or through `next_focus_id`. Without `selected`, the box with the lowest `tab_order` starts with the focus.

## Overflow and Scrolling

`overflow_behavior` decides what happens to content larger than the box:

| Value | Behaviour |
|-------|-----------|
| `scroll` (default) | Content scrolls in both directions. Scrollbars appear on the border. |
| `wrap` | Long lines break into several rows; rows scroll vertically. A wrapped menu entry continues under its label. |
| `truncate` | Content is cut off, and a cut line ends with `…`. No scrolling. |

`vertical_scroll` and `horizontal_scroll` set the starting scroll position in percent. `auto_scroll_bottom: true` keeps the newest output in view. Terminals always follow their output.

```yaml
- id: notes
  position: {x1: 10%, y1: 10%, x2: 50%, y2: 90%}
  overflow_behavior: wrap
  content: |
    This long line breaks at the box edge instead of needing horizontal scrolling.
```

## Color Reference

### Color Names

| Name | Terminal color |
|------|----------------|
| `default` | The terminal's default color |
| `black`, `red`, `green`, `yellow`, `blue`, `magenta`, `cyan`, `white` | ANSI colors 0-7 |
| `bright_black`, `bright_red`, `bright_green`, `bright_yellow`, `bright_blue`, `bright_magenta`, `bright_cyan`, `bright_white` | ANSI colors 8-15 |

Names are lower case. Any other value is a load error. These colors come from the terminal's palette, so they look different in different terminal themes.

### Color Settings

Boxes and layouts accept the same color settings:

| Element | Normal | Focused box | Under the mouse pointer | Error | Error and focused |
|---------|--------|-------------|-------------------------|-------|-------------------|
| Text | `fg_color` | `selected_fg_color` | `highlighted_fg_color` | `error_fg_color` | `error_selected_fg_color` |
| Background | `bg_color` | `selected_bg_color` | `highlighted_bg_color` | `error_bg_color` | `error_selected_bg_color` |
| Border | `border_color` | `selected_border_color` | `highlighted_border_color` | `error_border_color` | `error_selected_border_color` |
| Title text | `title_fg_color` | `selected_title_fg_color` | `highlighted_title_fg_color` | `error_title_fg_color` | `error_selected_title_fg_color` |
| Title background | `title_bg_color` | `selected_title_bg_color` | `highlighted_title_bg_color` | `error_title_bg_color` | `error_selected_title_bg_color` |

Menu entries have their own settings: `menu_fg_color` and `menu_bg_color` for entries, `selected_menu_fg_color` and `selected_menu_bg_color` for the selected entry (the one Enter runs), and `highlighted_menu_fg_color` and `highlighted_menu_bg_color` for the entry under the mouse pointer.

A box is in the error state when its visible tab's last run failed.

Each setting is looked up on the box, then on the enclosing boxes from the nearest out, then on the layout. When nothing sets it, the [theme](styling/theme) supplies the color. Every color setting is optional. [Visual Themes](styling/theme) describes the theme colors and how the state settings fall back.

```yaml
- id: status
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 20%}
  fg_color: bright_white
  bg_color: blue
  border_color: cyan
  selected_border_color: bright_yellow
  error_border_color: magenta
  script: [./check.sh]
```

## Chart Configuration

| Property | Type | Description |
|----------|------|-------------|
| `chart_type` | `string` | `bar`, `line`, `histogram`, `pie` or `scatter`. |
| `chart_data` | `string` | One data point per line. Without it, the output of the box's `script` is the data. |

Each data line is `label,value`, `label:value` or `label value`. Empty lines and lines starting with `#` are skipped, as are lines whose value is not a number. A chart needs `chart_data` or a `script`. The chart fills the box. The area inside the border and margins must be at least 20 columns by 5 rows; a smaller box shows `too small`.

```yaml
- id: sales
  title: 'Weekly Sales'
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
  chart_type: bar
  chart_data: |
    Mon,45
    Tue,67
    Wed,23

- id: disk
  title: 'Disk Use (%)'
  position: {x1: 50%, y1: 0%, x2: 100%, y2: 50%}
  chart_type: bar
  refresh_interval: 60000
  script: |
    df -P | tail -n +2 | awk '{gsub("%", "", $5); print $6 "," $5}'
```

## Table Configuration

| Property | Type | Description |
|----------|------|-------------|
| `table_data` | `string` | CSV whose first line is the header row, or a JSON array of objects. Without it, the output of the box's `script` is the data. |
| `table_config` | `object` | How the table is sorted, filtered, paged and drawn. |

`table_config` accepts these keys; any other key is a load error:

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `show_headers` | `boolean` | `true` | Show the header row. |
| `sort_column` | `string` | - | Header of the column to sort by. |
| `sort_ascending` | `boolean` | `true` | Sort direction. |
| `filters` | `object` | - | Column header to text. Only rows whose value in that column equals the text, ignoring case, are shown. |
| `border_style` | `string` | `single` | `none`, `single`, `double`, `rounded` or `thick`. |
| `zebra_striping` | `boolean` | `false` | Alternate the row style. |
| `max_column_width` | `integer` | `30` | Widest a column may be. |
| `show_row_numbers` | `boolean` | `false` | Number the rows. |
| `page_size` | `integer` | - | Rows per page. Without it, the table is not paged. |
| `current_page` | `integer` | `0` | Page shown; `0` is the first page. |
| `show_page_info` | `boolean` | `false` | Show the page number under the table. |

A table needs `table_data` or a `script`.

```yaml
- id: services
  title: Services
  position: {x1: 0%, y1: 50%, x2: 100%, y2: 100%}
  table_data: |
    Service,Status,CPU
    nginx,running,2.5
    mysql,running,15.2
    redis,stopped,0.0
  table_config:
    sort_column: CPU
    sort_ascending: false
    border_style: rounded
    zebra_striping: true
```

A box shows only one kind of generated content. Setting more than one of `plugin_component`, `chart_type` and `table_data`/`table_config` on the same box is a load error.

## Plugin Configuration

Plugins are shared libraries that draw the content of boxes. List them under `app.plugins`. A relative path is relative to the YAML file. A path without an extension is expanded to `libNAME.so`, `libNAME.dylib` or `NAME.dll`, depending on the platform.

| Property | Type | Description |
|----------|------|-------------|
| `plugin_component` | `string` | Name of a component a loaded plugin offers. |
| `plugin_config` | `object` | Passed to the plugin as JSON. |

```yaml
app:
  plugins:
    # Built with: cargo build --release --manifest-path examples/plugins/gauge/Cargo.toml
    - path: examples/plugins/gauge/target/release/boxmux_gauge
  layouts:
    - id: main
      children:
        - id: gauges
          title: Gauges
          position: {x1: 0%, y1: 0%, x2: 60%, y2: 40%}
          plugin_component: gauge
          plugin_config:
            gauges:
              - {label: CPU, value: 42}
              - {label: Memory, value: 6.1, max: 16}
```

boxmux refuses to start if a plugin cannot be loaded, if two plugins offer the same component, or if a box uses a component no plugin offers. Plugins run inside the boxmux process with its permissions; there is no sandbox. With `refresh_interval`, a plugin box is redrawn at that interval. See [Plugin System](plugin-system) for how to write a plugin.

## Layering

`z_index` orders sibling boxes that overlap: higher values are drawn on top. Boxes with equal values are drawn in file order, so later boxes are on top. Child boxes are always drawn on top of their parent.

```yaml
- id: background
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  z_index: 1
  content: 'Background'

- id: popup
  position: {x1: 25%, y1: 25%, x2: 75%, y2: 75%}
  z_index: 5
  content: 'On top'
```

## Mouse and Clipboard

The mouse and clipboard need no configuration.

- Click a box to focus it, a choice label to run it, a tab to show it, and `×` to close a tab.
- Drag a box's top border to move it, and its bottom-right corner (`⋱`) to resize it. The new position is written back to the YAML file. Comments are kept, and each edge keeps its unit (percent or cells). Start boxmux with `--lock` to turn moving and resizing off; the file is then never written.
- Click a scrollbar track to jump, drag the knob to scroll, or use the mouse wheel (3 lines per step).
- Ctrl+C (or Cmd+C) copies the text of the focused box's visible tab to the clipboard, using the OSC 52 escape sequence. When the focused box shows a live terminal, Ctrl+C goes to the program instead.

## Variable System

Any string value can use `${NAME}` and `${NAME:default}`. Values are looked up in the box's `variables`, then the enclosing boxes' variables, the layout's, the app's, and finally the environment. When the name is defined nowhere, the default is used.

```yaml
app:
  variables:
    ENVIRONMENT: production
  layouts:
    - id: main
      children:
        - id: web
          variables:
            SERVICE: nginx
          title: '${SERVICE} (${ENVIRONMENT})'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          script:
            - systemctl status ${SERVICE}
            - echo "User: ${USER:unknown}"
```

Substitution happens when the file is loaded, before validation. A reference to an undefined name without a default is left as written, and so is shell syntax such as `${i}`, `${X:-y}` or `${#list[@]}`. `$${` writes a literal `${`.

See the [Variable System Guide](variables) for the details.

## Validation

The file is checked when it is loaded. Any problem stops boxmux before the interface starts: the errors are printed and the exit code is non-zero.

A YAML syntax error names the file, line and column, and shows the line:

```text
boxmux: config.yaml:7:18: did not find expected ',' or '}' at line 7 column 18, while parsing a flow mapping at line 6 column 21
7 |           content: hi
  |                  ^
```

Other errors name the place in the file, with the layout and box ids along the way:

```text
boxmux: config.yaml: invalid configuration:
  app.layouts[0] ('main').children[0] ('logs').border_color: "purple" is not one of ["default","black","red",...]
  app.layouts[0] ('main').children[0] ('logs'): Additional properties are not allowed ('pty' was unexpected)
```

Besides the types and allowed values of every key, the loader checks:

- Layout ids, box ids and choice ids are unique across the file.
- At most one layout is `root`, and at most one box per layout is `selected`.
- `redirect_output` names an existing box, and `next_focus_id` names a box in the same layout.
- Every `hot_keys` entry names an existing choice.
- A choice with `redirect_output` has a `script`.
- `refresh_interval` is at least 1, needs a `script`, a `plugin_component` or children, and is not set on a `Pty` box.
- Key names in `hot_keys` and `on_keypress` are valid key names (see [Key names](/docs/advanced-features#key-names)), and neither `q` nor, on Unix, `Ctrl+z` is bound.
- `min_width`/`min_height` are not larger than `max_width`/`max_height`.
- A chart has `chart_data` or a `script`, `chart_data` has a `chart_type`, and a table has `table_data` or a `script`.
- A box has at most one of `plugin_component`, `chart_type` and `table_data`/`table_config`.

## Examples

### Complete Configuration Example

```yaml
app:
  layouts:
    - id: main
      root: true
      title: 'System Dashboard'
      border_color: green
      title_fg_color: bright_yellow
      children:
        - id: header
          title: 'System Status'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 15%}
          content: 'System Dashboard v1.0'

        - id: sidebar
          title: Navigation
          position: {x1: 0%, y1: 15%, x2: 25%, y2: 85%}
          tab_order: 1
          choices:
            - id: memory
              content: 'Memory Usage'
              script:
                - free -h
              redirect_output: main_content
            - id: disk
              content: 'Disk Usage'
              script:
                - df -h
              redirect_output: main_content

        - id: main_content
          title: Output
          position: {x1: 25%, y1: 15%, x2: 100%, y2: 85%}
          tab_order: 2
          content: 'Select an option from the menu'

        - id: footer
          title: Clock
          position: {x1: 0%, y1: 85%, x2: 100%, y2: 100%}
          refresh_interval: 1000
          script:
            - date
```

### Minimal Configuration Example

```yaml
app:
  layouts:
    - id: simple
      children:
        - id: content
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'Hello, World!'
```

## Best Practices

1. **Use meaningful ids**: ids appear in error messages and are used by `redirect_output`, `hot_keys` and socket commands.
2. **Start small**: load a simple file first, then add boxes, so each load error points at what you just changed.
3. **Set colors at the layout level**: boxes inherit them, so one change restyles the whole layout.
4. **Choose refresh intervals with care**: a short interval starts a new process each time.
5. **Make failures visible**: a script that exits non-zero shows the error colors, so let your scripts fail when something is wrong.
6. **Comment your configuration**: comments are kept when boxmux saves moved or resized boxes.

## Common Patterns

### Dashboard Layout

```yaml
# Header + Sidebar + Main + Footer
children:
  - id: header
    position: {x1: 0%, y1: 0%, x2: 100%, y2: 10%}
  - id: sidebar
    position: {x1: 0%, y1: 10%, x2: 20%, y2: 90%}
  - id: main
    position: {x1: 20%, y1: 10%, x2: 100%, y2: 90%}
  - id: footer
    position: {x1: 0%, y1: 90%, x2: 100%, y2: 100%}
```

### Grid Layout

```yaml
# 2x2 grid
children:
  - id: top_left
    position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
  - id: top_right
    position: {x1: 50%, y1: 0%, x2: 100%, y2: 50%}
  - id: bottom_left
    position: {x1: 0%, y1: 50%, x2: 50%, y2: 100%}
  - id: bottom_right
    position: {x1: 50%, y1: 50%, x2: 100%, y2: 100%}
```

### Modal/Dialog Pattern

```yaml
# Centered box drawn above the others
- id: modal
  position: {x1: 25%, y1: 25%, x2: 75%, y2: 75%}
  z_index: 10
  bg_color: white
  fg_color: black
  border_color: red
  content: 'Are you sure?'
```
