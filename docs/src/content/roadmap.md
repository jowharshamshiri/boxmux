---
title: Roadmap
description: What BoxMux does today and ideas that are not built yet.
---


BoxMux aims to make terminal dashboards and tools something you describe in a configuration
file rather than program. This page separates what exists from ideas that have not been built.
There are no dates.

## What exists

The [Features](/docs/features) page has the full list. In short:

- One YAML file per interface, checked against a schema and for consistency when loaded,
  with errors that name the location.
- Several layouts per file; nested boxes positioned in percentages or cells, with anchors,
  minimum and maximum sizes, and `z_index`.
- Scripts run once or on an interval, in three execution modes (`Immediate`, `Thread`, `Pty`),
  with output redirection, appending, auto-scrolling and saving to a file; `libs` sourced
  before every script.
- Menus (`choices`), `hot_keys` and `on_keypress` handlers; tabs for the outputs of a box.
- Embedded terminals with a terminal emulator (256 and 24-bit color, alternate screen, mouse
  reporting, scrollback).
- Charts (bar, line, histogram, pie, scatter) and tables (CSV or JSON, sorting, filtering,
  paging).
- Plugins loaded from shared libraries through a C interface.
- Variables with defaults, resolved from boxes, layouts, the app and the environment.
- A control socket and matching `boxmux` subcommands for changing a running instance.
- Mouse support: focus, menus, tabs, scrollbars, moving and resizing boxes with the result
  written back to the YAML file.
- Light and dark themes chosen from the terminal background or forced with a flag.
- Linux, FreeBSD, macOS (untested) and Windows; tests run with `cargo test` and on virtual
  machines with mg-test.

## Ideas not built yet

None of the following exists in BoxMux today. They are possible directions, not commitments.

- **Configuration reload**: watching the YAML file and applying changes to a running instance.
  Today the file is read once at start; only positions changed with the mouse are written back.
- **Automatic layouts**: grid or flex-style placement of boxes. Today every box has explicit
  edges.
- **Multi-file configuration**: including or importing other YAML files. Today one file holds
  the whole interface (`libs` shares shell code, not configuration).
- **Network control**: TCP or WebSocket access to a running instance. Today control is through
  a local socket or named pipe only, without authentication beyond file and pipe permissions.
- **Images**: sixel or other terminal graphics in boxes. Today boxes show text only.
- **Plugin isolation**: running plugins out of process or limiting what they can do. Today a
  plugin runs inside BoxMux with full privileges.
- **More chart and table interaction**: sorting or paging a table with the keyboard or mouse.
  Today `table_config` is fixed in the configuration (or replaced through the socket).
- **Tooling**: a layout editor, a configuration generator, or editor integration based on the
  JSON schema in `schemas/app_schema.json`.
- **macOS testing**: macOS uses the Unix code path but is not tested on macOS machines.

## Suggesting changes

Open an issue on [GitHub](https://github.com/jowharshamshiri/boxmux/issues) describing the use
case. For code contributions, see `CONTRIBUTING.md` in the repository.
