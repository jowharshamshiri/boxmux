# BoxMux

BoxMux turns a YAML file into a terminal interface. You describe boxes, their positions and
what goes in them: text, the output of shell scripts (run once or on an interval), menus whose
entries run scripts, interactive programs in an embedded terminal, charts, tables, or content
drawn by a plugin. BoxMux draws the boxes, runs the scripts, and handles the keyboard and the
mouse. A running instance can be controlled from other programs through a local socket.

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/boxmux.svg)](https://crates.io/crates/boxmux)
[![Documentation](https://img.shields.io/badge/docs-boxmux.com-blue.svg)](https://boxmux.com)

## Installation

### From crates.io

You need a stable Rust toolchain ([rustup](https://rustup.rs)).

```bash
cargo install boxmux
```

### From source

```bash
git clone https://github.com/jowharshamshiri/boxmux.git
cd boxmux
cargo build --release
./target/release/boxmux layouts/showcase_menu.yaml
```

The `layouts/` and `examples/` directories hold ready-made configurations to try.

### Prebuilt packages

Releases are built with `scripts/release.sh`, which runs the mg-test `release` plan
(`mg-test/release.yaml`). A GitHub release made this way carries:

- `boxmux-<version>-linux-x86_64.tar.gz` and `boxmux-<version>-linux-aarch64.tar.gz`
- `.deb` and `.rpm` packages for Linux x86-64 and arm64
- `boxmux-<version>-windows-x86_64.zip`
- `SHA256SUMS`

## A first interface

Save this as `hello.yaml`:

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
          content: |
            Hello from BoxMux.
            Tab moves the focus, Enter runs a menu entry, q quits.

        - id: clock
          title: Clock
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 40%}
          refresh_interval: 1000
          script:
            - date

        - id: actions
          title: Actions
          position: {x1: 50%, y1: 40%, x2: 100%, y2: 100%}
          tab_order: 1
          choices:
            - id: disk
              content: Disk usage
              script:
                - df -h
            - id: who
              content: Who am I
              script:
                - whoami
```

Run it:

```bash
boxmux hello.yaml
```

The clock box reruns `date` every second. Selecting a menu entry runs its script and shows the
output in a new tab of the Actions box. Positions are percentages of the parent (here the
screen) or cell counts written as strings (`"12"`). Unknown keys and invalid values are load
errors that name the place in the file.

## Command line

```
Usage: boxmux [OPTIONS] <config>
       boxmux [--socket <NAME>] <COMMAND> [ARGS]

Commands:
  replace_box_content   Show text in a box's socket tab
  replace_box_script    Replace a box's script and run it
  stop_box_refresh      Pause a box's periodic refresh
  start_box_refresh     Resume a box's periodic refresh
  replace_box           Replace a box with the one described in JSON
  add_box               Add a box, described in JSON, to a layout
  remove_box            Remove a box
  switch_active_layout  Show another layout
  spawn_pty_process     Run a script in a terminal shown in a box
  kill_pty_process      Stop the process in a box's visible tab
  restart_pty_process   Restart the process in a box's visible tab
  query_pty_status      Report on the process in a box's visible tab
  send_pty_input        Type input into the terminal in a box's visible tab
  send_signal           Send a signal to the process in a box's visible tab
  help                  Print this message or the help of the given subcommand(s)

Arguments:
  <config>  The YAML file describing the interface

Options:
  -d, --frame-delay <MS>   Minimum time between two redraws, in milliseconds [default: 30]
      --log-file <FILE>    Write a log to FILE
      --log-level <LEVEL>  How much to log [default: info] [possible values: error, warn, info, debug, trace]
      --lock               Disable moving and resizing boxes; the YAML file is never written
      --calibrate          Highlight the cell under the mouse pointer
      --dark               Use the dark color theme regardless of the terminal
      --light              Use the light color theme regardless of the terminal
      --socket <NAME>      The control socket (default: $BOXMUX_SOCKET, or a per-user default)
  -h, --help               Print help
  -V, --version            Print version
```

- `boxmux <command> --help` shows the arguments of each control command.
- `boxmux --version` prints one line naming the version and the commit the binary was built
  from:

  ```
  boxmux <version> commit <commit>
  boxmux <version> commit <commit>-dirty
  boxmux <version> commit unknown (built without git metadata)
  ```

  `<commit>` is the full 40-character hash. `-dirty` means the source tree had changes git had
  not committed, untracked files included. A build from a tree without git metadata of its own,
  such as the crates.io package or a source tarball, says `unknown (built without git
  metadata)`. Building from a git checkout needs `git`. The form is stable, and scripts may rely
  on it: the line is exactly one of the three above, with nothing else on it.
- `--log-level` needs `--log-file`. `--dark` and `--light` cannot be combined.
- The interface options cannot be combined with a control command.
- Moving or resizing a box with the mouse writes the new position back into the YAML file,
  keeping comments and units. `--lock` turns this off. Without `--lock`, BoxMux holds a lock
  file (`<config>.lock`) so that a second instance cannot edit the same file.

### Controlling a running instance

A running BoxMux listens on a control socket: `$XDG_RUNTIME_DIR/boxmux.sock` (or
`/tmp/boxmux-<uid>.sock`) on Unix, the named pipe `boxmux-<USERNAME>` on Windows, unless
`--socket` or `$BOXMUX_SOCKET` names another. From another terminal:

```bash
boxmux replace_box_content welcome "Deployment finished"
boxmux stop_box_refresh clock
boxmux switch_active_layout main
```

Scripts started by BoxMux get `BOXMUX_SOCKET` and `BOXMUX_BOX` (the id of the box they run for)
in their environment, so they can control the instance they run in. The protocol is one JSON
line per request; see the [API reference](https://boxmux.com/docs/api).

## Features

- **Layouts and boxes**: several layouts per file (one shown at a time, switchable over the
  socket), nested boxes positioned in percentages or cells, `anchor` with minimum and maximum
  sizes, `z_index` for overlapping boxes.
- **Scripts**: run once or every `refresh_interval` milliseconds; three execution modes:
  `Immediate` (output shown when the script ends), `Thread` (output streamed line by line) and
  `Pty` (an interactive terminal in the box). Output can go to another box
  (`redirect_output`), be appended (`append_output`), follow the newest line
  (`auto_scroll_bottom`) and be saved to a file (`save_in_file`). Shared shell functions can be
  sourced from `libs`.
- **Menus**: `choices` run their scripts with Enter or a click; each run opens its own tab.
  `hot_keys` bind keys to choices, and `on_keypress` binds keys to scripts at app, layout or
  box level.
- **Tabs**: a box with several outputs (its content, its script output, menu runs, socket
  updates) shows them as tabs in its top border; extra tabs can be closed.
- **Embedded terminal**: `execution_mode: Pty` runs programs such as `htop`, `vim` or a shell in
  the box, with 256 and 24-bit color, the alternate screen, mouse reporting and scrollback.
- **Charts and tables**: bar, line, histogram, pie and scatter charts; tables from CSV or JSON
  with sorting, filtering and paging. Data can be inline or come from a script.
- **Plugins**: shared libraries with a small C interface can draw box content.
- **Variables**: `${NAME}` and `${NAME:default}` with values from boxes, layouts, the app or the
  environment.
- **Mouse**: click to focus, run menu entries and switch tabs; drag the top border to move a
  box and the bottom-right corner to resize it; scrollbars and the wheel scroll.
- **Keyboard**: Tab and Shift+Tab move the focus along `tab_order` (or `next_focus_id`), arrows
  and Page Up/Down scroll, Ctrl+C copies the focused box's visible text to the clipboard
  (through the terminal, with OSC 52).
- **Colors**: 16 ANSI color names for borders, text, titles and menus, with separate colors for
  the focused box, the box under the pointer and failed runs. The light or dark default theme
  follows the terminal background, or is set with `--light` / `--dark`.

## Platforms

- Linux (x86-64 and arm64) and FreeBSD: tested.
- macOS: uses the same Unix code, but is not tested on macOS machines.
- Windows 10 and later: terminals use ConPTY; scripts run in PowerShell unless `app.shell`
  says otherwise; `send_signal` is not supported; the terminal background is not queried, so
  the theme comes from `COLORFGBG`, `--light` / `--dark`, or defaults to dark.

On Unix, scripts run in `bash` unless `app.shell` is `sh` or `zsh` (`powershell` and `pwsh` are
also accepted).

## Documentation

The documentation is at [boxmux.com](https://boxmux.com); its source is in
`docs/src/content/`.

- [User Guide](https://boxmux.com/docs/user-guide): building an interface step by step
- [Configuration Reference](https://boxmux.com/docs/configuration): every YAML setting
- [Examples](https://boxmux.com/docs/examples)
- [API Reference](https://boxmux.com/docs/api): the control socket and commands
- [PTY Features](https://boxmux.com/docs/pty-features): interactive programs in boxes
- [Data Visualization](https://boxmux.com/docs/data-visualization): charts and tables
- [Plugin System](https://boxmux.com/docs/plugin-system): writing plugins
- [Variables](https://boxmux.com/docs/variables)
- [Advanced Features](https://boxmux.com/docs/advanced-features)
- [Troubleshooting](https://boxmux.com/docs/troubleshooting)

## Development and testing

```bash
cargo build
cargo test                    # unit tests, runtime tests and end-to-end tests
cargo test --test live_ux     # only the end-to-end tests (the real binary in a pseudo-terminal)
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

Cross-platform runs use [mg-test](mg-test.yaml), which starts virtual machines, copies the
tracked files into them and runs the plans in `mg-test/`:

| Plan | Machines | What runs |
|------|----------|-----------|
| `hosts` | Ubuntu x86-64, Windows 11 x86-64 | formatting (Linux only), clippy and all tests |
| `arm64` | Ubuntu arm64 | formatting, clippy and all tests |
| `freebsd` | FreeBSD x86-64 | formatting, clippy and all tests |
| `release` | Ubuntu x86-64 and arm64, Windows 11 x86-64 | release builds and packages |

```bash
mg-test run mg-test.yaml --plan hosts
```

The runners are `mg-test/test.sh` (Linux and FreeBSD) and `mg-test/test.ps1` (Windows). See
[CONTRIBUTING.md](CONTRIBUTING.md) for the project layout and conventions, and
[CHANGELOG.md](CHANGELOG.md) for the history.

## License

MIT. See [LICENSE](LICENSE).
