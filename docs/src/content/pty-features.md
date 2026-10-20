---
title: PTY Features
description: Interactive programs in BoxMux boxes - execution_mode Pty, keyboard, paste and mouse forwarding, terminal size, the built-in terminal emulator, status colors, clipboard, socket control and Windows notes
---

## Overview

A box can show a live terminal: the script runs in a pseudo-terminal (PTY), and boxmux
draws the program's screen inside the box. Interactive programs such as `htop`, `vim`,
`less`, `ssh` or a shell work there, and you type into them when the box has the focus.

There are three ways to start a program in a PTY:

| How | Where the terminal appears |
|-----|---------------------------|
| A box with `execution_mode: Pty` and a `script` | The box's output tab. The script starts when its layout is first shown and runs once |
| A choice with `execution_mode: Pty` | A closeable tab, labelled with the choice's `content`, in the box (or in its `redirect_output` box) |
| The `spawn_pty_process` control command | A closeable tab labelled `Terminal` (see [API Reference](/docs/api#spawn_pty_process)) |

```yaml
app:
  layouts:
    - id: main
      children:
        - id: top
          title: Processes
          position: {x1: 0%, y1: 0%, x2: 60%, y2: 100%}
          execution_mode: Pty
          script:
            - htop
        - id: tools
          title: Tools
          position: {x1: 60%, y1: 0%, x2: 100%, y2: 100%}
          choices:
            - id: shell
              content: Shell
              execution_mode: Pty
              script:
                - bash
            - id: logs
              content: Follow syslog
              execution_mode: Pty
              redirect_output: top
              script:
                - less +F /var/log/syslog
```

There is no `pty:` key; `execution_mode` is the only switch, and an unknown key is a load
error.

### Rules

- A `Pty` box cannot have a `refresh_interval`. Loading fails with
  `a PTY box runs its script once as an interactive session; refresh_interval does not apply`.
  A `refresh_interval` inherited from an enclosing box or the layout is ignored for it.
- The box that shows the terminal must be in the shown layout when the program starts,
  because the terminal size comes from the box. A choice or `spawn_pty_process` that targets a
  box of another layout fails with
  `box '<id>' is not in the shown layout, so it has no size for a terminal`. A box with no
  room inside its border fails with `box '<id>' is too small to show a terminal`.
- One run per origin: activating a PTY choice while its program still runs does not start a
  second copy; it brings the existing tab into view.
- Programs keep running when you switch to another layout.

## Environment and terminal size

- The script runs in the app's shell (`bash` by default, `powershell` on Windows), in
  boxmux's working directory, with boxmux's environment plus `TERM=xterm-256color`,
  `BOXMUX_SOCKET` and `BOXMUX_BOX` (see [API Reference](/docs/api#environment-of-scripts)).
- The terminal size is the inside of the box's border: the whole area within the border,
  without the one-column margin that text boxes have.
- When the box changes size (terminal window resized, box moved or resized with the mouse,
  layout switched), the PTY is resized and the program is told (on Unix it receives
  `SIGWINCH`).

## Input

### Keyboard

When the focused box's visible tab is a terminal whose program is running, **every key goes
to the program except Tab and Shift+Tab**, which always move the focus so you can leave the
terminal. That includes `q`, Ctrl+C, Enter and the arrow keys; boxmux's own bindings for them
do not apply while the terminal has the focus.

Keys are sent as an xterm would send them:

| Key | Bytes |
|-----|-------|
| Characters | UTF-8 |
| Ctrl+letter, Ctrl+`@` `[` `\` `]` `^` `_` `?` (and Ctrl+2..8) | The control character (Ctrl+C is `0x03`) |
| Alt+key | `ESC` followed by the key |
| Enter, Backspace, Esc | `\r`, `0x7f` (Ctrl+Backspace `0x08`, Alt+Backspace `ESC 0x7f`), `ESC` |
| Arrows, Home, End | `ESC [ A`... or, after the program enabled application cursor keys (DECCKM), `ESC O A`... |
| Arrows etc. with Shift/Alt/Ctrl | `ESC [ 1 ; <mod> A` (xterm modifier codes) |
| Insert, Delete, PageUp, PageDown | `ESC [ 2 ~`, `ESC [ 3 ~`, `ESC [ 5 ~`, `ESC [ 6 ~` (with `; <mod>` when modified) |
| F1-F4 | `ESC O P`..`ESC O S` (`ESC [ 1 ; <mod> P`.. when modified) |
| F5-F12 | `ESC [ 15 ~` .. `ESC [ 24 ~` |

Keys with no encoding here (for example F13 and up) are not sent. To send a Tab to the
program, use the socket: `boxmux send_pty_input <box> $'\t'`.

When the program ends, the keyboard goes back to boxmux.

### Paste

boxmux does not turn on bracketed paste for its own terminal, so pasted text reaches boxmux
as ordinary key presses and is sent to the focused terminal key by key, like typing. A tab
character in pasted text moves the focus, like the Tab key.

### Mouse

Mouse events are sent to the program only when all of these hold:

- the box has the focus and its visible tab is a running terminal;
- the program has turned on mouse reporting (mode 1000, 1002 or 1003);
- the pointer is inside the terminal area of that box, not on its border and not on a box
  drawn above it.

Positions are relative to the terminal's top-left cell. Reports use the SGR format
(`ESC [ < b ; x ; y M/m`) when the program enabled mode 1006, otherwise the classic
`ESC [ M` format. A drag that starts in the terminal stays with it until the button is
released, with positions held to the terminal's edges.

Limitations of the current emulator: in the classic format, button releases are not reported
in mode 1000, and pointer motion (including dragging with a button held) is reported only in
mode 1003. In the SGR format, motion is reported in every mode but carries the motion flag
only in mode 1003, so in modes 1000 and 1002 it looks like repeated button presses.

Otherwise the mouse works as for any box: a click on an unfocused terminal box focuses it,
the top border moves the box, the corner resizes it, and the wheel scrolls the box's
content.

## The screen

The tab shows the program's scrollback followed by its screen, or only the screen while the
program uses the alternate screen (as full-screen programs do). The scrollback keeps up to
10,000 lines. Whenever the screen changes and the tab is visible, the box scrolls to the
bottom. While the program does not use the mouse, the wheel and the scrollbar scroll back
through earlier output.

The program's cursor is not drawn.

When the program ends, its last screen stays in the tab.

### Terminal emulator

boxmux contains its own terminal emulator. It handles:

- **Text**: UTF-8, wide (two-column) characters, line wrapping with auto-wrap on or off
  (DECAWM), insert mode (IRM), linefeed/newline mode (LNM), tab stops (HTS, TBC, CHT, CBT,
  default stops every 8 columns).
- **Attributes**: bold, dim, italic, underline (double underline is drawn as underline),
  blink, reverse, hidden, strikethrough; the 16 ANSI colors, the 256-color palette and
  24-bit color, for foreground and background.
- **Cursor movement**: CUP/HVP (`H`, `f`), CUU/CUD/CUF/CUB (`A`-`D`), save and restore with
  `ESC 7` / `ESC 8`, index, reverse index and next line (`ESC D`, `ESC M`, `ESC E`), origin
  mode (DECOM).
- **Editing**: erase in display and in line (`J`, `K`, including `3J` to clear the
  scrollback), insert and delete lines (`L`, `M`), scroll up and down (`S`, `T`), scroll
  regions (DECSTBM), `P` (clears the given number of cells at the cursor; it does not shift
  the rest of the line).
- **Screens**: the alternate screen through modes 47, 1047 and 1049 (1049 also saves the
  cursor), full reset (`ESC c`).
- **Modes**: application cursor keys (DECCKM), mouse reporting (1000, 1002, 1003, 1006).
  Only the first mode of a `CSI ? ... h/l` sequence with several modes takes effect.
- **Replies to queries**: device status and cursor position (`CSI 5 n`, `CSI 6 n`), primary
  device attributes (`CSI c`, `ESC Z`), terminal parameters (DECREQTPARM) and mode queries
  (DECRQM). Replies are written back to the program.
- **OSC**: window title (OSC 0 and 2) and clipboard writes (OSC 52), described below.

Not implemented, among others: cursor horizontal/vertical absolute (`G`, `d`), cursor
next/previous line (`E`, `F`), insert and erase characters (`@`, `X`), repeat (`b`),
`CSI s` / `CSI u`, cursor shapes, the DEC special graphics character set (line-drawing
characters selected with `ESC ( 0` are shown as plain letters), double-width and
double-height lines, combining characters (they are dropped), and secondary device
attributes (`CSI > c` gets the primary answer). Programs that rely on these may draw
incorrectly.

### Title

When the program sets a title with OSC 0 or OSC 2 (`ESC ] 2 ; title BEL`), the title becomes
the label of its tab. A box showing a single tab displays its own `title` when it has one,
so the program's title is visible when the box has no `title` or shows several tabs.

### Clipboard (OSC 52)

A program can copy text to the clipboard with OSC 52 (`ESC ] 52 ; c ; <base64> BEL`), as
`tmux` and `vim` with OSC 52 support do. boxmux passes the text on to your terminal with its
own OSC 52 sequence, so your terminal must support OSC 52 for it to reach the system
clipboard, and shows a notice such as `shell copied 3 lines`. Text that is not valid base64
UTF-8 is ignored.

Requests to read the clipboard (`ESC ] 52 ; c ; ? BEL`) are refused: programs never get the
clipboard's contents.

## Status and colors

While the program runs, the box uses its normal colors. When it ends, the border shows how:

| How the program ended | Border | `query_pty_status` |
|-----------------------|--------|--------------------|
| Exit code 0 | Yellow | `exited with code 0` |
| Stopped by boxmux (tab closed, `kill_pty_process`) | Yellow | `killed` |
| Non-zero exit code | The box's error colors (`error_border_color`, default red; bright red when focused) | `exited with code <n>` |
| Ended by a signal from elsewhere (including `send_signal`) | Error colors | `failed: ended by <signal>` |
| Could not be started | Error colors | `failed: <reason>` |

The yellow border is fixed and takes precedence over the focus color. Error colors follow the
`error_*` and `error_selected_*` settings (see [Configuration](/docs/configuration)).

## Stopping and restarting

- **Closing the tab** (`×`) of a choice or socket terminal stops the program and every
  process it started. A box's own script tab has no `×`.
- **Activating the choice again** after its program ended starts it again in the same tab.
- **Removing or replacing the box** through the socket stops it.
- **Quitting boxmux** stops every program it started.

Stopping means `SIGTERM` to the program's process group, then `SIGKILL` after 500 ms if it is
still there (Unix), or `taskkill /T` followed by a forced `taskkill /T /F` (Windows).

## Socket control

The control socket works on the process feeding a box's **visible tab**:

```bash
boxmux query_pty_status top          # running?, pid, size, status
boxmux send_pty_input top q          # type into it
boxmux send_pty_input shell $'make\r'
boxmux send_signal top 19            # Unix only
boxmux kill_pty_process top
boxmux restart_pty_process top       # run the same script again
boxmux spawn_pty_process tools 'ssh build-host' --redirect-output top
```

See the [API Reference](/docs/api) for fields, results and errors.

## Windows

- PTYs use the Windows pseudo-console (ConPTY), Windows 10 and later.
- Scripts run in Windows PowerShell (`powershell -NoLogo -NoProfile -NonInteractive -Command`)
  unless the app sets `shell: pwsh` (PowerShell 7). Write PTY scripts for that shell.
- `send_signal` is not supported; it always fails with
  `signal <n> cannot be sent on Windows; use kill_pty_process`.
- Stopping a program ends its whole process tree with `taskkill`.
- Ctrl+Z does not suspend boxmux.
