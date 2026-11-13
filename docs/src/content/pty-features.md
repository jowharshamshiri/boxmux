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

boxmux turns on bracketed paste in your terminal, so a paste arrives as one piece rather than
as key presses. It goes to the focused terminal: wrapped in `ESC [ 200 ~` and `ESC [ 201 ~`
when the program asked for bracketed paste (mode 2004, as shells and editors do), so the
program can tell it from typing, and as plain text otherwise. A pasted Tab goes to the program;
it does not move the focus. When no terminal has the focus, a paste does nothing.

On Windows the console has no bracketed paste: pasted text arrives as key presses and is
sent to the focused terminal key by key, like typing, so a pasted Tab moves the focus.

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

What is reported depends on the mode the program chose:

| Mode | Reported |
|------|----------|
| 9 (X10) | Button presses, without modifier keys |
| 1000 | Presses and releases |
| 1002 | Presses, releases, and motion while a button is held |
| 1003 | Presses, releases and all motion |

Wheel turns are reported as buttons 64 and up. Shift, Alt (or Cmd) and Ctrl are reported with
the xterm modifier bits. The classic format names no button on release and cannot express
positions past column or row 223; such events are not sent. The SGR format names the released
button and has no position limit.

Otherwise the mouse works as for any box: a click on an unfocused terminal box focuses it,
the top border moves the box, the corner resizes it, and the wheel scrolls the box's
content.

## The screen

The tab shows the program's scrollback followed by its screen, or only the screen while the
program uses the alternate screen (as full-screen programs do). The scrollback keeps up to
10,000 lines. Whenever the screen changes and the tab is visible, the box scrolls to the
bottom. While the program does not use the mouse, the wheel and the scrollbar scroll back
through earlier output.

The program's cursor is drawn as a cell in reverse video, unless the program hid it
(`ESC [ ? 25 l`). Cursor shapes (DECSCUSR) are not drawn.

When the program ends, its last screen stays in the tab.

### Terminal emulator

boxmux contains its own terminal emulator. It handles:

- **Text**: UTF-8 in grapheme clusters: a combining mark, variation selector, skin tone or
  joined emoji goes into the cell of the character before it, and each cell takes the columns
  [Text Width](configuration.md#text-width) gives it (wide characters and emoji two); line
  wrapping with auto-wrap on or off (DECAWM), insert mode (IRM), linefeed/newline mode
  (LNM), tab stops (HTS, TBC, CHT, CBT, default stops every 8 columns), repeat of the last character (`b`).
- **Character sets**: G0 and G1 with shift in and shift out; the DEC special graphics set
  (`ESC ( 0`), whose letters become line-drawing characters (`lqqk` is `┌──┐`), and the UK
  set (`#` is `£`).
- **Attributes**: bold, dim, italic, underline (every underline style is drawn as underline),
  blink, reverse, hidden, strikethrough; the 16 ANSI colors, the 256-color palette and
  24-bit color, for foreground and background, with parameters separated by semicolons or
  colons (`38;2;r;g;b`, `38:2::r:g:b`).
- **Cursor movement**: CUP/HVP (`H`, `f`), CUU/CUD/CUF/CUB (`A`-`D`), HPR/VPR (`a`, `e`),
  CNL/CPL (`E`, `F`), CHA/HPA (`G`, `` ` ``), VPA (`d`), save and restore with `ESC 7` /
  `ESC 8` and `CSI s` / `CSI u` (position, attributes, character sets, origin and wrap
  modes), index, reverse index and next line (`ESC D`, `ESC M`, `ESC E`), origin mode
  (DECOM). Up and down movement stops at the scroll region's margins, as in xterm.
- **Editing**: erase in display and in line (`J`, `K`, including `3J` to clear the
  scrollback), insert, delete and erase characters (`@`, `P`, `X`), insert and delete lines
  within the scroll region (`L`, `M`), scroll up and down (`S`, `T`), scroll regions
  (DECSTBM), screen alignment (`ESC # 8`). Erased cells take the current background color.
- **Screens**: the alternate screen through modes 47, 1047 and 1049 (1049 also saves the
  cursor; leaving 1047 clears it), full reset (`ESC c`), reverse video (mode 5).
- **Modes**: application cursor keys (DECCKM), application keypad, mouse reporting (9, 1000,
  1002, 1003, with the SGR format 1006), bracketed paste (2004). A `CSI ? ... h/l` sequence
  may set several modes at once.
- **Replies to queries**: device status and cursor position (`CSI 5 n`, `CSI 6 n`,
  `CSI ? 6 n`), primary and secondary device attributes (`CSI c`, `ESC Z`, `CSI > c`), mode
  queries (DECRQM, `CSI ? n $ p`), the screen size (`CSI 18 t`, `CSI 14 t`). Requests for
  settings (DECRQSS) and termcap entries (XTGETTCAP) are answered as not available, so
  programs go on with their defaults. Replies are written back to the program.
- **OSC**: window title (OSC 0 and 2), hyperlinks (OSC 8) and clipboard writes (OSC 52),
  described below.

Not implemented: cursor shapes, double-width and double-height lines (`ESC # 3` to `6`; such
lines are shown at normal size), left and right margins, sixel images, and combining
characters (they are dropped). Programs that rely on these may draw incorrectly.

### Title

When the program sets a title with OSC 0 or OSC 2 (`ESC ] 2 ; title BEL`), the title becomes
the label of its tab. A box showing a single tab displays its own `title` when it has one,
so the program's title is visible when the box has no `title` or shows several tabs.

### Hyperlinks (OSC 8)

Text a program prints between `ESC ] 8 ; params ; URI ST` and `ESC ] 8 ; ; ST` is a link in
each cell it goes in, as `ls --hyperlink`, `gcc` and `systemd` print them. BoxMux writes the
link to your terminal around those cells, so a line the program's text wrapped onto is the
same link, with the same id. A color reset does not end a link; erasing a cell takes its link
away. A malformed link (a parameter that is not `key=value`, a URI with a character outside
printable ASCII) changes nothing. See [Hyperlinks](styling/theme#hyperlinks).

### Clipboard (OSC 52)

A program can copy text to the clipboard with OSC 52 (`ESC ] 52 ; c ; <base64> ST`, or ended
with BEL), as `tmux` and `vim` with OSC 52 support do. boxmux passes the text on to your
terminal with its own OSC 52 sequence, so your terminal must support OSC 52 for it to reach the
system clipboard, and shows a notice such as `shell copied "make test"` or `shell copied 3
lines`. A copy whose text is not base64, or not UTF-8 once decoded, is not passed on: an error
notice such as `shell cannot copy to the clipboard: its text is not base64` says so.

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
- Scripts run in Windows PowerShell (`powershell -NoLogo -NoProfile -Command`; scripts
  outside a PTY also get `-NonInteractive`) unless the app sets `shell: pwsh` (PowerShell 7).
  Write PTY scripts for that shell.
- `send_signal` is not supported; it always fails with
  `signal <n> cannot be sent on Windows; use kill_pty_process`.
- Stopping a program ends its whole process tree with `taskkill`.
- Ctrl+Z does not suspend boxmux.
- Pasted text arrives as key presses (see [Paste](#paste)).
