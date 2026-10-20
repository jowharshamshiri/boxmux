---
title: Interactive UI Features
description: Mouse and keyboard interaction in BoxMux - focus, menus, tabs, scrolling, moving and resizing boxes, and how new positions are saved to the YAML file
---

## Table of Contents

- [Overview](#overview)
- [Focus and Keyboard](#focus-and-keyboard)
- [Menus](#menus)
- [Tabs](#tabs)
- [Scrolling](#scrolling)
- [Moving and Resizing Boxes](#moving-and-resizing-boxes)
- [Saving Positions to the YAML File](#saving-positions-to-the-yaml-file)
- [Hover Highlighting](#hover-highlighting)
- [Overlapping Boxes](#overlapping-boxes)
- [Terminal Boxes](#terminal-boxes)
- [Copying Text](#copying-text)
- [Notices](#notices)
- [Practical Example](#practical-example)

## Overview

BoxMux reads keyboard and mouse input from the terminal. With the mouse you can:

- click a box to focus it
- click a menu entry to run it
- click tabs to switch between them, and `×` to close one
- scroll with the wheel, the scrollbars, or by dragging a scrollbar knob
- move a box by dragging its top border
- resize a box by dragging its bottom-right corner

Moved and resized boxes are written back to the YAML file, so the new layout is there the next time you start BoxMux. Start with `--lock` to turn moving and resizing off; BoxMux then never writes the file.

The terminal must report mouse events; almost all current terminal emulators do.

## Focus and Keyboard

One box at a time has the focus. Its border uses the focus color (the theme's blue, or `selected_border_color`), and keys act on it.

- At start, the focus is on the box with `selected: true`, or else on the box with the lowest `tab_order`. When neither exists, no box has the focus until you click one.
- Tab moves the focus to the next box in `tab_order` and Shift+Tab to the previous one, wrapping around. Boxes without `tab_order` are skipped (clicking still focuses them).
- A box's `next_focus_id` overrides the order for Tab: pressing Tab in that box moves to the named box, which must be in the same layout.

| Key | Action |
|-----|--------|
| `q` | Quit (without modifiers; a focused terminal box receives it instead) |
| Tab / Shift+Tab | Next / previous box |
| Up / Down | Move the menu selection; in other boxes, scroll one line |
| Enter | Run the selected menu entry |
| Left / Right | Scroll one column |
| PageUp / PageDown | Scroll one page up / down |
| Shift+PageUp / Shift+PageDown | Scroll one page left / right |
| Home / End | Scroll to the left / right end |
| Ctrl+Home / Ctrl+End | Scroll to the top / bottom |
| Ctrl+C (Cmd+C where the terminal passes it) | Copy the focused box's text to the clipboard |
| Ctrl+Z | Suspend BoxMux (Unix); `fg` resumes it |

Keys also run `hot_keys` and `on_keypress` handlers; see [Key Bindings](/docs/user-guide#key-bindings).

```yaml
app:
  layouts:
    - id: 'focus_demo'
      root: true
      children:
        - id: 'menu'
          title: 'Menu (Tab 1)'
          position: {x1: 0%, y1: 0%, x2: 30%, y2: 100%}
          tab_order: 1
          next_focus_id: 'details'   # Tab from here goes to 'details', skipping 'log'
          choices:
            - id: 'one'
              content: 'First'
              script: ['echo first']
              redirect_output: 'details'
        - id: 'log'
          title: 'Log (Tab 2)'
          position: {x1: 30%, y1: 0%, x2: 65%, y2: 100%}
          tab_order: 2
          content: 'Shift+Tab reaches this box from details'
        - id: 'details'
          title: 'Details (Tab 3)'
          position: {x1: 65%, y1: 0%, x2: 100%, y2: 100%}
          tab_order: 3
          content: 'Output appears here'
```

## Menus

A box with `choices` shows them as a menu. The selected entry is marked with `►` and drawn in the selected-menu colors (`selected_menu_fg_color`, `selected_menu_bg_color`).

- Up and Down move the selection, wrapping at the ends, and scroll the menu to keep it in view.
- Enter runs the selected entry. Clicking an entry selects and runs it. Only the label is clickable, not the empty space after it.
- While an entry's script runs, the entry shows ` …`. Running it again while it runs only brings its output tab into view.

With `overflow_behavior: wrap`, long labels wrap and continue under the label, not under the `►` marker. With `truncate` they are cut with `…`.

## Tabs

A box that has more than one stream (its content, its menu, its script output, output of choices and key handlers, socket content) shows a tab bar in its top border instead of the title.

- Click a tab to show it. Hidden tabs keep receiving output.
- Tabs with a `×` can be closed: output of choices, key handlers, redirected scripts, socket content and spawned terminals. Closing a tab stops the process that feeds it. The box's own content, menu and script output cannot be closed.
- When the tabs do not fit, `◀` and `▶` appear at the ends of the bar; click them to scroll the bar. The last tab can always be scrolled into view.
- When a choice or key handler starts, its tab becomes the shown tab.

See [Stream Architecture](/docs/stream-architecture) for how tabs are created and labelled.

## Scrolling

With `overflow_behavior: scroll` (the default), content larger than the box gets scrollbars drawn on the right and bottom border: `█` and `■` are the knobs, `│` and `─` the tracks.

| Mouse action | Result |
|--------------|--------|
| Wheel up / down | Scroll 3 lines |
| Wheel left / right (horizontal wheel or tilt) | Scroll 3 columns |
| Click on a scrollbar track | Jump so the knob is centered on the clicked cell |
| Drag a scrollbar knob | Scroll with the knob |

The wheel scrolls the box under the pointer, whether or not it has the focus.

With `overflow_behavior: wrap` there is only vertical scrolling. With `truncate` there is no scrolling.

`vertical_scroll` and `horizontal_scroll` (0 to 100, in percent) set where a box starts scrolled to. `auto_scroll_bottom: true` scrolls to the bottom whenever new output arrives in the shown tab; terminal tabs always do this.

```yaml
- id: 'long_text'
  title: 'Scroll Me'
  position: {x1: 10%, y1: 10%, x2: 50%, y2: 40%}
  overflow_behavior: scroll
  vertical_scroll: 100       # Start at the bottom
  script:
    - seq 1 200
```

## Moving and Resizing Boxes

### Moving

Press the left button on the box's top border, outside any tab, and drag. The box follows the pointer and stops at the edges of its parent (the enclosing box, or the screen); its size does not change.

### Resizing

The bottom-right corner of a resizable box is drawn as `⋱`. Drag it to change the box's width and height. The top-left corner stays in place, the box cannot become smaller than 3 by 3 cells, and it cannot grow past the edges of its parent.

With `--lock`, the corner is drawn as `┘` and neither moving nor resizing is possible.

Nested boxes are positioned relative to their parent, so they move and scale with it.

## Saving Positions to the YAML File

When you release the mouse after moving or resizing a box, BoxMux writes the box's new `position` into the YAML file it was started with:

```yaml
# Before
- id: 'demo_box'
  position: {x1: 10%, y1: 10%, x2: 50%, y2: 50%}   # main panel

# After resizing the box
- id: 'demo_box'
  position: {x1: "10%", y1: "10%", x2: "75%", y2: "65%"}   # main panel
```

- Only the `position` of that box changes. Comments, key order and the rest of the file are left as they were.
- Each edge keeps its unit: a percentage stays a percentage, a cell count stays a cell count.
- Before writing, BoxMux checks that the edited file still parses and differs from the original only in that position. If the box's id appears more than once in the file, or its `position` cannot be found, nothing is written and a notice says why.
- The file is replaced atomically (written to a temporary file in the same directory, then renamed), with its permissions kept.
- BoxMux does not reload the file while it runs. Edits you make in an editor meanwhile are kept, because BoxMux reads the file again before each save and changes only the position.

While BoxMux runs without `--lock`, it holds an exclusive lock on `<config>.lock` next to the YAML file, so a second instance cannot write the same file. That second instance refuses to start; run it with `--lock` to view the same file without editing it. The lock file is removed when BoxMux exits.

Scroll positions, the shown tab and the focused box are not saved.

## Hover Highlighting

The element under the mouse pointer is highlighted:

- Menu entries, tabs, tab close buttons and the tab bar arrows use the hover colors: black on bright yellow by default, or `highlighted_menu_fg_color` / `highlighted_menu_bg_color`.
- A box under the pointer uses its `highlighted_*` settings (`highlighted_border_color`, `highlighted_bg_color`, `highlighted_fg_color`, `highlighted_title_fg_color`, `highlighted_title_bg_color`, `highlighted_fill_char`) when they are set. Without them, a hovered box looks the same as when it is not hovered.

```yaml
- id: 'hover_demo'
  title: 'Hover Me'
  position: {x1: 10%, y1: 10%, x2: 50%, y2: 50%}
  highlighted_border_color: 'bright_yellow'
  highlighted_title_fg_color: 'bright_yellow'
  content: 'The border turns yellow under the pointer'
```

Run BoxMux with `--calibrate` to highlight the single cell under the pointer. This helps when a terminal reports mouse positions that are off by a cell.

## Overlapping Boxes

Boxes may overlap. Among siblings, boxes are drawn in order of `z_index` (default 0), lowest first; boxes with the same `z_index` are drawn in the order they appear in the file. A box's children are drawn after it, on top of it.

Clicks, hover and the mouse wheel act on the box that is visible at the pointer.

```yaml
- id: 'background_box'
  title: 'Background'
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  content: 'Behind'

- id: 'foreground_box'
  title: 'Foreground'
  z_index: 1
  position: {x1: 20%, y1: 20%, x2: 80%, y2: 80%}
  content: 'On top: drag my top border or corner'
```

`fill: false` leaves a box's background unpainted, so the boxes beneath show through.

## Terminal Boxes

A box whose shown tab is a running terminal program (`execution_mode: Pty`, or a terminal started through the socket) behaves differently while it has the focus:

- Every key except Tab and Shift+Tab goes to the program, including `q`, Ctrl+C and the arrow keys.
- If the program has turned on mouse reporting (as `htop`, `vim` with `mouse=a`, or `mc` do), mouse clicks, drags and wheel events inside the terminal area go to the program, with coordinates relative to the box. A drag that starts in the program stays with it until the button is released. Otherwise the mouse acts on BoxMux as usual, and the wheel scrolls the terminal's scrollback.
- The top border and the resize corner still belong to BoxMux, so the box can be moved and resized. The program is told the new size.

## Copying Text

Ctrl+C copies the text of the focused box's shown tab to the clipboard, without color codes; for a menu, it copies the entry labels. BoxMux sends the text to the terminal with the OSC 52 escape sequence and shows "copied N lines". Whether the text reaches the system clipboard depends on the terminal: some need the feature enabled, and some ignore it (see [Troubleshooting](/docs/troubleshooting#copy-to-clipboard-does-nothing)).

Programs running in terminal boxes can copy text the same way; BoxMux passes their OSC 52 requests on to your terminal. Requests to read the clipboard are refused.

## Notices

Short messages appear on the bottom row of the screen for 5 seconds: confirmations such as "copied 12 lines" and errors (in red) such as a failed save, a handler that failed, or a hot key whose choice is not in the shown layout.

## Practical Example

```yaml
app:
  layouts:
    - id: 'main'
      root: true
      title: 'Interactive Dashboard'
      children:
        - id: 'header'
          title: 'System Overview'
          z_index: 2
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 15%}
          content: 'Drag a top border to move a box, a ⋱ corner to resize it'

        - id: 'sidebar'
          title: 'Controls'
          position: {x1: 0%, y1: 15%, x2: 25%, y2: 100%}
          tab_order: 1
          choices:
            - id: 'processes'
              content: 'Processes'
              script: ['ps aux | head -n 40']
              redirect_output: 'main_display'
            - id: 'cpu_watch'
              content: 'Watch Load'
              execution_mode: Thread
              script:
                - while true; do uptime; sleep 1; done
              redirect_output: 'main_display'

        - id: 'main_display'
          title: 'Output'
          position: {x1: 25%, y1: 15%, x2: 100%, y2: 100%}
          content: 'Pick a control on the left'
          auto_scroll_bottom: true
          highlighted_border_color: 'bright_cyan'
```

Try it:

1. Click "Watch Load": a tab with that name opens in the output box and gains a line every second.
2. Click "Processes": a second tab opens. Click between the two tabs; the hidden one keeps updating.
3. Click `×` on the "Watch Load" tab to stop the loop.
4. Drag the output box's `⋱` corner, then quit with `q` and look at its `position` in the file.
