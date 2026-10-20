---
title: BoxMux Visual Themes
description: Customizing colors, styling, and visual appearance in BoxMux terminal applications
---

# BoxMux Visual Themes

BoxMux draws readable colors without any configuration. Every color can also be set in the YAML file, on a layout or on a box.

## Automatic Light/Dark Theme

A configuration with no color settings works in light and dark terminals. BoxMux picks a light or dark theme and uses it for every color the file does not set.

The theme is chosen in this order:

1. `--light` or `--dark` on the command line.
2. The terminal's background color, which BoxMux asks the terminal for at start (Unix only).
3. The `COLORFGBG` environment variable: background color 7 or 15 means light.
4. Otherwise, dark.

```bash
boxmux layouts/dashboard.yaml --dark
boxmux layouts/dashboard.yaml --light
```

### Theme Colors

| Element | Dark theme | Light theme |
|---------|------------|-------------|
| Text | `white` | `black` |
| Background | `black` | `white` |
| Border | `white` | dark gray (256-color 238) |
| Border of the focused box | blue (256-color 25) | blue (256-color 25) |
| Title | the box's text and background colors | the box's text and background colors |
| Border in the error state | `red`; `bright_red` when focused | same |
| Title in the error state | `bright_white` on `red` | same |
| Selected menu entry | `bright_white` on dark gray (238) | same |
| Menu entry or tab under the mouse pointer | `black` on `bright_yellow` | same |

The focus and selection accents use fixed entries of the 256-color palette. Terminal themes usually remap only the 16 base colors, so these accents look the same in every terminal.

A few elements always use the theme and have no setting:

| Element | Color |
|---------|-------|
| The visible tab | `bright_white` on dark gray (238); on blue (25) in the focused box |
| Scrollbar track | `bright_black` (the knob uses the border color) |
| Border of a terminal whose program ended normally or was stopped | `yellow` |
| Notices on the bottom row | `bright_white` on blue (25); on `red` for errors |

## ANSI Color System

Colors are written as lower-case names. Any other value is a load error.

### Standard Colors

| Name | ANSI color |
|------|------------|
| `black` | 0 |
| `red` | 1 |
| `green` | 2 |
| `yellow` | 3 |
| `blue` | 4 |
| `magenta` | 5 |
| `cyan` | 6 |
| `white` | 7 |

### Bright Colors

| Name | ANSI color |
|------|------------|
| `bright_black` | 8 |
| `bright_red` | 9 |
| `bright_green` | 10 |
| `bright_yellow` | 11 |
| `bright_blue` | 12 |
| `bright_magenta` | 13 |
| `bright_cyan` | 14 |
| `bright_white` | 15 |

`default` is the terminal's own default color.

These are the terminal's palette colors, so their exact look depends on the terminal theme. Some palettes, for example, draw `bright_black` as a light gray.

## Box Styling Properties

### Color Settings

Boxes and layouts accept the same color settings. A box uses its own setting, else the nearest enclosing box's, else the layout's, else the theme's.

| Element | Normal | Focused box | Under the mouse pointer | Error | Error and focused |
|---------|--------|-------------|-------------------------|-------|-------------------|
| Text | `fg_color` | `selected_fg_color` | `highlighted_fg_color` | `error_fg_color` | `error_selected_fg_color` |
| Background | `bg_color` | `selected_bg_color` | `highlighted_bg_color` | `error_bg_color` | `error_selected_bg_color` |
| Border | `border_color` | `selected_border_color` | `highlighted_border_color` | `error_border_color` | `error_selected_border_color` |
| Title text | `title_fg_color` | `selected_title_fg_color` | `highlighted_title_fg_color` | `error_title_fg_color` | `error_selected_title_fg_color` |
| Title background | `title_bg_color` | `selected_title_bg_color` | `highlighted_title_bg_color` | `error_title_bg_color` | `error_selected_title_bg_color` |

Menus have three pairs:

| Setting | Applies to |
|---------|------------|
| `menu_fg_color`, `menu_bg_color` | Menu entries. Default: the box's text and background. |
| `selected_menu_fg_color`, `selected_menu_bg_color` | The selected entry, marked `►`, which Enter runs. |
| `highlighted_menu_fg_color`, `highlighted_menu_bg_color` | The entry under the mouse pointer. Tabs under the pointer use these colors too. |

When a box has tabs, the titles of the tabs that are not visible use the title colors.

### Basic Box Appearance

```yaml
- id: styled_box
  title: Status
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
  content: 'All systems normal'
  border_color: bright_blue
  title_fg_color: bright_yellow
  fg_color: bright_white
  bg_color: black
```

### Fill

| Setting | Default | Description |
|---------|---------|-------------|
| `fill` | `true` | `false` leaves the box's background unpainted, so whatever is beneath shows through. |
| `fill_char` | `' '` | Character in the empty cells of the box. |
| `selected_fill_char` | - | Fill character while the box has focus. |
| `highlighted_fill_char` | - | Fill character while the mouse pointer is over the box. |

On a layout, `fill: false` leaves the terminal's own background behind the boxes, and `fill_char` fills the screen behind the boxes. The layout's `fg_color` and `bg_color` color that background. Boxes inherit `fill` and the fill characters from enclosing boxes and the layout.

### Title Position

`title_position` puts the title at the `left`, `center` (default) or `right` of the top border.

## States and Fallbacks

A box can be in several states at once:

- **Focused**: the box has the keyboard focus.
- **Highlighted**: the mouse pointer is over the box.
- **Error**: the last run of the visible tab failed, for example with a non-zero exit status. A tab whose process you stopped is not an error.

For each element, BoxMux tries the settings of the box's states in a fixed order. It takes the first one that is set on the box, an enclosing box or the layout. If none is set, the theme decides.

| Element | Order |
|---------|-------|
| Text, background | `error_selected_*` (error and focused), `error_*` (error), `selected_*` (focused), `highlighted_*` (pointer), the plain setting, the theme |
| Border, error state | `error_selected_border_color` (focused), `error_border_color`, the theme's red (`bright_red` when focused) |
| Border, ended terminal | always `yellow` |
| Border, focused | `selected_border_color`, the theme's focus blue |
| Border, otherwise | `highlighted_border_color` (pointer), `border_color`, the theme's border color |
| Title, error state | `error_selected_title_*` (focused), `error_title_*`, `bright_white` on `red` |
| Title, otherwise | `selected_title_*` (focused), `highlighted_title_*` (pointer), `title_*`, the box's text and background |
| Fill character | `selected_fill_char` (focused), `highlighted_fill_char` (pointer), `fill_char`, a space |

Two consequences:

- A focused box does not use `border_color`. Set `selected_border_color` to change the focus border.
- State settings win over plain ones wherever they are set. A `selected_bg_color` on the layout applies to a focused box even when the box sets its own `bg_color`.

```yaml
- id: interactive_box
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
  tab_order: 1
  script: ['./check.sh']
  border_color: white
  selected_border_color: bright_green     # Border while focused
  highlighted_border_color: cyan          # Border while the pointer is over it
  error_border_color: magenta             # Border when check.sh fails
  selected_fill_char: '·'                 # Empty cells while focused
```

## Theme Examples

There is no separate theme file. Set the colors on the layout, and every box in it inherits them.

### Professional Dark Theme

```yaml
app:
  layouts:
    - id: main
      bg_color: black
      fg_color: white
      border_color: bright_blue
      title_fg_color: bright_white
      selected_border_color: bright_cyan
      selected_menu_fg_color: black
      selected_menu_bg_color: bright_cyan
      children:
        - id: header
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 20%}
          border_color: bright_cyan
          content: 'Operations'
        - id: status
          position: {x1: 0%, y1: 20%, x2: 50%, y2: 100%}
          fg_color: bright_green
          content: 'All services running'
        - id: warnings
          position: {x1: 50%, y1: 20%, x2: 100%, y2: 100%}
          border_color: bright_yellow
          fg_color: bright_yellow
          content: '2 warnings'
```

### Retro Terminal Theme

```yaml
app:
  layouts:
    - id: main
      bg_color: black
      fg_color: green
      border_color: green
      title_fg_color: bright_green
      selected_border_color: bright_green
      selected_fill_char: '░'
      menu_fg_color: green
      selected_menu_fg_color: black
      selected_menu_bg_color: bright_green
      children:
        - id: main_menu
          title: Menu
          position: {x1: 0%, y1: 0%, x2: 40%, y2: 100%}
          tab_order: 1
          choices:
            - id: who
              content: 'Who is logged in'
              script: ['who']
        - id: secondary
          title: Notes
          position: {x1: 40%, y1: 0%, x2: 100%, y2: 100%}
          tab_order: 2
          content: 'READY.'
```

### High Contrast Theme

```yaml
app:
  layouts:
    - id: main
      bg_color: black
      fg_color: bright_white
      border_color: bright_white
      title_fg_color: black
      title_bg_color: bright_white
      selected_border_color: bright_yellow
      selected_title_bg_color: bright_yellow
      selected_menu_fg_color: black
      selected_menu_bg_color: bright_yellow
      children:
        - id: primary
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 50%}
          tab_order: 1
          content: 'Primary information'
        - id: accent
          position: {x1: 0%, y1: 50%, x2: 100%, y2: 100%}
          tab_order: 2
          fg_color: bright_cyan
          border_color: bright_cyan
          content: 'Secondary information'
```

### Named Colors with Variables

Variables work in color settings, so a scheme can be defined once:

```yaml
app:
  variables:
    PRIMARY: bright_blue
    ACCENT: bright_cyan
    WARNING: bright_yellow
  layouts:
    - id: dashboard
      border_color: '${PRIMARY}'
      title_fg_color: '${ACCENT}'
      children:
        - id: alerts
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          border_color: '${WARNING}'
          content: 'No alerts'
```

The substituted value must be a valid color name.

## Script Output Colors

Script output keeps its SGR color and style sequences. Boxes show:

- The 16 base colors and their bright versions (`30`-`37`, `90`-`97`, and the background codes).
- 256-color (`38;5;N`) and 24-bit (`38;2;R;G;B`) colors.
- Bold, dim, italic, underline, blink, reverse, hidden and strikethrough.

A reset (`ESC[0m`) returns to the box's own text and background colors. Each line starts in the box's colors; a color set on one line does not continue on the next line. Other escape sequences, such as cursor movement, are removed. A carriage return goes back to the start of the line, so progress bars show their last state.

```yaml
- id: log_viewer
  title: Log
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  script:
    - echo "$(tput setaf 2)SUCCESS$(tput sgr0) operation completed"
    - echo "$(tput setaf 3)WARNING$(tput sgr0) check configuration"
    - echo "$(tput bold)$(tput setaf 1)ERROR$(tput sgr0) connection failed"
```

`tput` prints the escape sequences for the terminal named in `TERM`. Printing the sequences directly, such as `ESC[32m` with `printf`, works the same way.

Programs in a terminal box (`execution_mode: Pty`) draw with their own colors, including 256-color and 24-bit color.

## Z-Index and Layering

`z_index` orders overlapping sibling boxes; higher values are drawn on top. A box with `fill: false` lets the boxes beneath show through its empty cells.

```yaml
- id: background_box
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  z_index: 1
  border_color: bright_black
  content: 'Background'

- id: popup_overlay
  position: {x1: 30%, y1: 30%, x2: 70%, y2: 70%}
  z_index: 10
  border_color: bright_yellow
  content: 'Popup'
```

## Theme Best Practices

- Set colors on the layout and override them only in the boxes that need it.
- Leave `bg_color` and `fg_color` unset if the file should work in both light and dark terminals; the automatic theme then picks readable defaults.
- Check text against its background in both a light and a dark terminal theme, because the 16 named colors change with the terminal palette.
- Do not rely on color alone for important states. Have scripts print a clear message, such as `ERROR: disk full`, as well as exiting with a failure status.
