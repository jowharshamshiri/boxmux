---
title: BoxMux - YAML-driven Terminal UI Framework
description: Build terminal dashboards and tools from YAML files - boxes, scripts, menus, embedded terminals, charts, tables, plugins and a control socket.
---


## What BoxMux is

BoxMux reads a YAML file and draws a terminal interface from it. Each box shows text, the
output of a shell script (run once or on an interval), a menu whose entries run scripts, an
interactive program in an embedded terminal, a chart, a table, or content drawn by a plugin.
BoxMux handles the keyboard and the mouse, and a running instance can be controlled by other
programs through a local socket.

## Installation

```bash
cargo install boxmux
```

Or build from source:

```bash
git clone https://github.com/jowharshamshiri/boxmux.git
cd boxmux
cargo build --release
./target/release/boxmux layouts/showcase_menu.yaml
```

The repository's `layouts/` and `examples/` directories hold configurations to try. Press `q`
to quit.

## A minimal interface

```yaml
app:
  layouts:
    - id: main
      root: true
      title: Hello BoxMux
      children:
        - id: welcome
          title: Welcome
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
          content: Hello from BoxMux. Press q to quit.
        - id: clock
          title: Clock
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
          refresh_interval: 1000
          script:
            - date
```

Save it as `hello.yaml` and run `boxmux hello.yaml`.

## What it can do

- **Layouts and boxes**: several layouts per file, nested boxes positioned in percentages or
  cells, anchors with minimum and maximum sizes, stacking with `z_index`.
- **Scripts**: run once or on a `refresh_interval`, with output shown when the script ends,
  streamed line by line, or in an interactive terminal (`execution_mode`); output can be
  redirected to another box, appended, or saved to a file.
- **Menus and keys**: `choices` that run scripts, `hot_keys` that trigger choices, and
  `on_keypress` handlers at app, layout and box level.
- **Tabs**: a box keeps each output (content, script output, menu runs, socket updates) in its
  own tab.
- **Embedded terminals**: programs such as `htop`, `vim` or a shell run inside a box.
- **Charts and tables**: bar, line, histogram, pie and scatter charts; tables from CSV or JSON
  with sorting, filtering and paging.
- **Plugins**: shared libraries with a small C interface draw box content.
- **Variables**: `${NAME}` and `${NAME:default}` resolved from boxes, layouts, the app and the
  environment.
- **Control socket**: update content, replace scripts, add and remove boxes, switch layouts and
  manage terminal processes from scripts or the `boxmux` command line.
- **Mouse**: focus, run menu entries, switch and close tabs, move and resize boxes (saved back
  to the YAML file), scroll.
- **Platforms**: Linux, FreeBSD, macOS (not tested on macOS machines) and Windows 10 or later.

## Getting started

- **[User Guide](/docs/user-guide)**: building an interface step by step
- **[Configuration Reference](/docs/configuration)**: every YAML setting
- **[API Reference](/docs/api)**: the control socket and the command line

## How-to guides

- **[Drive BoxMux as a Live Control Panel](/docs/live-control-panel)**: another program keeps
  the screen current and hears back from its buttons

## More topics

- **[Data Visualization](/docs/data-visualization)**: charts and tables
- **[Plugin System](/docs/plugin-system)**: writing plugins
- **[PTY Features](/docs/pty-features)**: interactive programs in boxes
- **[Advanced Features](/docs/advanced-features)**: variables, output handling, keys, focus,
  layering, sizing, sockets and plugins together
- **[Variables](/docs/variables)**: substitution rules

## Resources

- **[Troubleshooting](/docs/troubleshooting)**
- **[Roadmap](/docs/roadmap)**: what exists and what is not built yet
- **[GitHub Repository](https://github.com/jowharshamshiri/boxmux)**: source code and issues
