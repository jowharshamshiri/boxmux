---
title: API Reference
description: The BoxMux control socket and the CLI control commands - transport, every command with its fields and errors, reply format, and the environment given to scripts
---

## Overview

A running boxmux instance listens on a **control socket**. Other programs, including the
scripts boxmux itself runs, use it to change what the instance shows: put text in a box,
replace a box's script, add or remove boxes, switch layouts, and start and control
processes in terminal boxes.

There are two ways to send a command:

- the `boxmux` binary itself: `boxmux <command> [ARGS]` sends one command and prints the reply;
- any program that can write a line of JSON to a local socket (`socat`, `nc -U`, Python...).

The JSON protocol is described by a JSON Schema in
[`api-spec.json`](https://github.com/jowharshamshiri/boxmux/blob/main/api-spec.json) at the
root of the repository. A test checks that the schema matches the code.

## The socket

### Which socket

boxmux and its control commands pick the socket the same way, first match wins:

1. the `--socket <NAME>` option;
2. the `BOXMUX_SOCKET` environment variable, when it is set and not empty;
3. the per-user default:

| Platform | Default socket |
|----------|----------------|
| Linux, macOS, FreeBSD | `$XDG_RUNTIME_DIR/boxmux.sock`, or `/tmp/boxmux-<uid>.sock` when `XDG_RUNTIME_DIR` is not set |
| Windows | the named pipe `\\.\pipe\boxmux-<USERNAME>` |

On Unix `NAME` is the path of a socket file. On Windows it is a pipe name: `--socket work`
means `\\.\pipe\work`.

To run several instances, give each its own socket:

```bash
boxmux --socket /tmp/dash-a.sock a.yaml
boxmux --socket /tmp/dash-b.sock b.yaml
boxmux --socket /tmp/dash-b.sock switch_active_layout logs
```

### Starting and stopping

When boxmux starts it first tries to talk to the socket. If another boxmux instance answers,
it refuses to start:

```
boxmux: another boxmux instance is already listening on '/run/user/1000/boxmux.sock'; start this one with --socket <name>
```

A socket file left behind by an instance that crashed does not answer, and is replaced. On
Unix the socket file is removed when boxmux exits. boxmux does not change the permissions of
the socket file; it gets the ones your umask gives.

### Protocol

Each connection carries exactly one command:

1. The client connects and writes one JSON object on a single line, **ending with a newline**
   (`\n`).
2. boxmux writes one JSON object on a single line, ending with a newline.
3. The connection is closed.

A request has a `command` field naming the command, plus that command's fields:

```json
{"command":"replace_box_content","box_id":"status","content":"Deploy finished"}
```

The reply is either

```json
{"ok":true}
```

or, for the commands that return data (`query_pty_status`, `spawn_pty_process`),

```json
{"ok":true,"result":{"run":4}}
```

or, when the command failed,

```json
{"ok":false,"error":"there is no box 'staus'"}
```

`result` and `error` are left out when they have no value.

Rules:

- Unknown commands and unknown fields are errors. Field types are checked (a number where a
  string is expected is an error).
- A request that does not end with a newline, or is larger than 16 MiB, gets the error
  `requests must be a single line of at most 16777216 bytes`.
- A request that is not valid JSON, or not a valid command, gets `invalid request: <details>`.
- Commands are carried out one at a time, in the order they arrive, by the same loop that
  handles the keyboard and draws the screen. The screen is redrawn afterwards.
- If boxmux does not answer within 10 seconds the reply is `boxmux did not answer in time`.
  The command can still take effect afterwards.
- While boxmux is quitting the reply is `boxmux is shutting down`.
- Commands change only the running instance. The YAML file is never written by a control
  command.

### The visible tab

Several commands act on the **visible tab** of a box: the tab whose content the box shows
right now. A box has a tab for its static `content`, its `choices`, the output of its
`script`, and one for every other run that writes into it (choices, key handlers, redirected
output, `spawn_pty_process`) and for socket content. See
[Stream Architecture](/docs/stream-architecture). `kill_pty_process`, `restart_pty_process`,
`query_pty_status`, `send_pty_input` and `send_signal` all act on the process that feeds the
visible tab of the box you name.

## The CLI

```
boxmux [--socket <NAME>] <COMMAND> [ARGS]
```

`--socket` may come before or after the command. The options that only make sense for an
interface (`<config>`, `-d`/`--frame-delay`, `--log-file`, `--log-level`, `--lock`,
`--calibrate`, `--dark`, `--light`) cannot be combined with a control command.

| Outcome | stdout | stderr | Exit code |
|---------|--------|--------|-----------|
| Success without a result | nothing | nothing | 0 |
| Success with a result | the `result` as indented JSON | nothing | 0 |
| The command failed, or no instance answers | nothing | `boxmux: <error>` | 1 |
| Invalid arguments | nothing | a usage error | 2 |

`boxmux <command> --help` describes each command's arguments.

```bash
$ boxmux remove_box nope
boxmux: there is no box 'nope'
$ boxmux --socket /tmp/none.sock remove_box nope
boxmux: cannot reach a boxmux instance at '/tmp/none.sock': No such file or directory (os error 2) (is boxmux running, and is BOXMUX_SOCKET or --socket set to its socket?)
```

## Commands

Every command naming a box fails with `there is no box '<id>'` when no layout has a box with
that id. Unless stated otherwise, the box can be in any layout, shown or not.

### replace_box_content

Shows text in the box's **socket tab**. The first call creates the tab, labelled `Socket`,
at the end of the box's tabs. Each call replaces the tab's text and makes it the visible tab.

| Field | Type | Required | Default | Meaning |
|-------|------|----------|---------|---------|
| `box_id` | string | yes | | The box |
| `content` | string | yes | | The text. Lines are split on newlines; SGR color sequences are kept, other control sequences are removed |
| `success` | boolean | no | `true` | `false` draws the box in its error colors while this tab is visible |

```bash
boxmux replace_box_content status "Deploy finished"
boxmux replace_box_content status "Deploy failed" --failed
```

```json
{"command":"replace_box_content","box_id":"status","content":"Deploy failed","success":false}
```

The socket tab can be closed with its `×`. It does not change the box's `content`, which keeps
its own tab.

### replace_box_script

Replaces the box's `script` and runs it now, making its output tab the visible tab. Output
goes where the box's script output goes (the box, or its `redirect_output` target), in the
box's `execution_mode`.

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `box_id` | string | yes | The box |
| `script` | array of strings | yes | The script lines; at least one must not be blank |

```bash
boxmux replace_box_script clock "date '+%H:%M:%S'"
boxmux replace_box_script report "cd /var/log" "tail -n 50 syslog"   # one argument per line
```

```json
{"command":"replace_box_script","box_id":"clock","script":["date '+%H:%M:%S'"]}
```

If the old script is still running it is stopped, and the new one starts once it has ended.
The box's other tabs (content, menu, socket, terminals) stay. A periodic refresh the box
already has keeps its interval and its pause; a box that had no script refreshes from now on
like a box configured with one: with its own `refresh_interval`, or the one it inherits from
an enclosing box or its layout.

Errors: `the script is empty`; a box in `Pty` mode whose layout is not shown cannot start
(`box '<id>' is not in the shown layout, so it has no size for a terminal`).

### stop_box_refresh and start_box_refresh

`stop_box_refresh` pauses the periodic run of the box's script; `start_box_refresh` resumes
it and runs the script right away (unless it is still running). For a plugin box with a
`refresh_interval` they pause and resume its periodic redraw. A box of a layout that is not
shown can be paused or resumed too; refreshes only run while their layout is shown.

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `box_id` | string | yes | The box |

```bash
boxmux stop_box_refresh clock
boxmux start_box_refresh clock
```

```json
{"command":"stop_box_refresh","box_id":"clock"}
```

Errors:

- `there is no box '<id>'`;
- `box '<id>' has no script` - nor a plugin with a `refresh_interval`;
- `box '<id>' does not refresh: it has no refresh_interval, or its script runs in a PTY` - the
  box runs its script only once.

### replace_box

Replaces the configuration of a box with a new one.

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `box_id` | string | yes | The box to replace |
| `new_box` | box object | yes | The new box, written like a box in the YAML file but as JSON. Its `id` must equal `box_id` |

```bash
boxmux replace_box status '{"id":"status","title":"Status","position":{"x1":"0%","y1":"0%","x2":"50%","y2":"30%"},"content":"Ready"}'
```

```json
{"command":"replace_box","box_id":"status","new_box":{"id":"status","position":{"x1":"0%","y1":"0%","x2":"50%","y2":"30%"},"content":"Ready"}}
```

The new box takes the old one's place in the layout (same parent, same focus state). Its
`children` replace the old box's children. Everything running for the old box or its
children is stopped and their tabs are gone. If the box's layout is shown, the new box's
script starts.

Errors: `the new box has id '<a>' but replaces '<b>'; ids must match`, and validation errors
(see [Box objects](#box-objects)).

### add_box

Adds a box to a layout, after the boxes already there.

| Field | Type | Required | Default | Meaning |
|-------|------|----------|---------|---------|
| `layout_id` | string | yes | | The layout |
| `parent_id` | string or null | no | `null` | A box of that layout to nest the new box in; `null` or absent adds it at the top level |
| `muxbox` | box object | yes | | The new box (with its children, if any) |

The CLI takes the parent with `--parent`:

```bash
boxmux add_box main '{"id":"alerts","position":{"x1":"50%","y1":"0%","x2":"100%","y2":"30%"},"script":["tail -n 20 /var/log/syslog"],"refresh_interval":5000}'
boxmux add_box main '{"id":"note","position":{"x1":"0%","y1":"0%","x2":"100%","y2":"3"},"content":"hi"}' --parent sidebar
```

```json
{"command":"add_box","layout_id":"main","parent_id":"sidebar","muxbox":{"id":"note","position":{"x1":"0%","y1":"0%","x2":"100%","y2":"3"},"content":"hi"}}
```

The new box is not focused. If the layout is shown, its script starts.

Errors: `a box with id '<id>' already exists`, `there is no layout '<id>'`,
`there is no box '<parent>' in layout '<layout>'`, and validation errors.

### remove_box

Removes a box and its children. Everything running for them, or shown in them, is stopped.
If the removed box had the focus, the focus moves to the next box.

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `box_id` | string | yes | The box |

```bash
boxmux remove_box alerts
```

```json
{"command":"remove_box","box_id":"alerts"}
```

Removing a box that other boxes refer to (for example in `redirect_output` or
`next_focus_id`) fails with the validation error that the same YAML file would produce.

### switch_active_layout

Shows another layout. Scripts of its boxes that have not run yet start now. Scripts of the
previously shown layout keep running, and periodic refreshes that already started keep
running too.

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `layout_id` | string | yes | The layout to show |

```bash
boxmux switch_active_layout monitoring
```

```json
{"command":"switch_active_layout","layout_id":"monitoring"}
```

Error: `no layout with id '<id>'`.

### spawn_pty_process

Runs a script in a pseudo-terminal and shows the terminal in a new tab of the box. The tab is
labelled `Terminal` (until the program sets a title), can be closed with `×`, and becomes the
visible tab. See [PTY Features](/docs/pty-features).

| Field | Type | Required | Default | Meaning |
|-------|------|----------|---------|---------|
| `box_id` | string | yes | | The box the process belongs to; it is shown there unless `redirect_output` is set |
| `script` | array of strings | yes | | The script lines, run by the app's shell |
| `redirect_output` | string or null | no | `null` | Show the terminal in this box instead |

```bash
boxmux spawn_pty_process shell htop
boxmux spawn_pty_process menu top --redirect-output shell
```

```json
{"command":"spawn_pty_process","box_id":"shell","script":["htop"]}
```

Reply:

```json
{"ok":true,"result":{"run":16}}
```

`run` identifies the run within this boxmux instance.

The box that shows the terminal must be in the shown layout, since the terminal size comes
from its size. Only one process started this way for a given `box_id` runs at a time; spawn
again after it has ended, or use `restart_pty_process`.

Errors: `box '<id>' is not in the shown layout, so it has no size for a terminal`,
`box '<id>' is too small to show a terminal`, `run-<n> is still running`, and errors starting
the process.

### kill_pty_process

Stops the process feeding the box's visible tab, together with every process it started. On
Unix the process group gets `SIGTERM`, then `SIGKILL` if it is still there after 500 ms. On
Windows the process tree is ended with `taskkill /T`, then forcibly. This works for any run
(PTY or not). The tab stays, with status `killed`.

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `box_id` | string | yes | The box |

```bash
boxmux kill_pty_process shell
```

```json
{"command":"kill_pty_process","box_id":"shell"}
```

Error: `nothing is running in the visible tab of box '<id>'`.

### restart_pty_process

Runs again the last thing that ran in the box's visible tab, with the same script, mode and
target. If it is still running it is stopped first, and started again once it has ended. This
works for any run (PTY or not).

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `box_id` | string | yes | The box |

```bash
boxmux restart_pty_process shell
```

```json
{"command":"restart_pty_process","box_id":"shell"}
```

Errors: `box '<id>' has no tabs`, `the visible tab of box '<id>' has never run anything`
(for example a content, menu or socket tab).

### query_pty_status

Reports on the box's visible tab and the process feeding it.

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `box_id` | string | yes | The box |

```bash
boxmux query_pty_status shell
```

```json
{"command":"query_pty_status","box_id":"shell"}
```

Result while a PTY process runs:

```json
{"box_id":"shell","cols":118,"pid":402986,"rows":18,"running":true,"status":"running","stream":"Terminal"}
```

and after it was killed:

```json
{"box_id":"shell","running":false,"status":"killed","stream":"Terminal"}
```

| Field | Present | Meaning |
|-------|---------|---------|
| `box_id` | always | The box |
| `stream` | always | The label of the visible tab |
| `running` | always | Whether a process is feeding the visible tab |
| `status` | always | `idle`, `running`, `exited with code <n>`, `failed: <reason>` or `killed` |
| `pid` | while running | The process id (on Unix also its process group id) |
| `rows`, `cols` | while a PTY process runs | The terminal size |

A script tab that has not run yet reports `idle`. A process ended by a signal it did not get
from boxmux reports `failed: ended by ...`.

Errors: `box '<id>' has no tabs`, and
`the visible tab of box '<id>' ('<label>') does not show a process` when the visible tab is
static content, a menu or a socket tab.

### send_pty_input

Writes text to the PTY process feeding the box's visible tab, as if it had been typed.

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `box_id` | string | yes | The box |
| `input` | string | yes | Written byte for byte (UTF-8). Enter is a carriage return, `\r` |

The CLI sends its argument verbatim; use your shell's quoting for control characters (in bash,
`$'...'` quoting understands backslash escapes, for example a carriage return or Ctrl+C written
as a hex escape):

```bash
boxmux send_pty_input shell $'ls -la\r'
```

```json
{"command":"send_pty_input","box_id":"shell","input":"ls -la\r"}
```

Errors: `nothing is running in the visible tab of box '<id>'`,
`the process in box '<id>' is not a PTY process`, and write errors.

### send_signal

Sends a signal to the process group of the process feeding the box's visible tab. Unix only.

| Field | Type | Required | Meaning |
|-------|------|----------|---------|
| `box_id` | string | yes | The box |
| `signal` | integer | yes | The signal number, e.g. `2` (SIGINT), `15` (SIGTERM), `19` (SIGSTOP on Linux) |

The signal number must be 1 or more; 0 and negative numbers are refused
(`signal <n> is not a signal number`).

```bash
boxmux send_signal shell 2
```

```json
{"command":"send_signal","box_id":"shell","signal":2}
```

A process ended by a signal sent this way is reported as `failed: ended by ...`, not as
`killed`.

Errors: `nothing is running in the visible tab of box '<id>'`,
`cannot send signal <n>: <reason>`. On Windows every signal fails with
`cannot send signal <n>: signal <n> cannot be sent on Windows; use kill_pty_process`.

## Box objects

`replace_box` and `add_box` take a box written as JSON with the same fields as a box in the
YAML file (see [Configuration](/docs/configuration) and `schemas/app_schema.json`). A few
things to keep in mind:

- Positions are strings, as in YAML: `"50%"` or a cell count such as `"12"`. A JSON number is
  an error (`invalid type: integer `10`, expected a string`).
- `script` may be an array of lines or a single string.
- The box is checked against the box definition of the schema when the request is read, like
  a box in a configuration file: `id` and `position` are required, and unknown fields and
  unknown color names are errors. The error names the field, for example
  `invalid request: box: "position" is a required property` or
  `box.children[0] ('c').border_color: "pink" is not one of ...`. The CLI checks the box
  before sending it.
- A `plugin_component` must be offered by a plugin the instance loaded.
- Variables (`${NAME}`) are not substituted in boxes sent through the socket; such text
  reaches the shell (in scripts) or the screen as written.

The application with the change applied is checked by the same schema and semantic rules as
the YAML file before anything changes. A failure leaves the instance unchanged and the reply
names the problem with its location; several problems are joined with `; `:

```json
{"ok":false,"error":"app.layouts[0] ('main').children[4] ('pp').refresh_interval: a PTY box runs its script once as an interactive session; refresh_interval does not apply"}
```

## Environment of scripts

Every script boxmux runs (box scripts, choices, `on_keypress` handlers, PTY sessions) gets
these environment variables in addition to boxmux's own environment:

| Variable | Value |
|----------|-------|
| `BOXMUX_SOCKET` | The socket this instance listens on, exactly as boxmux was given or chose it |
| `BOXMUX_BOX` | The id of the box whose tab receives the script's output: the box itself, or its `redirect_output` target. Empty for app- and layout-level `on_keypress` handlers |
| `TERM` | `xterm-256color`, in PTY sessions only. Other scripts keep boxmux's `TERM` |

Because `BOXMUX_SOCKET` is set, a script can control the instance that started it without
naming the socket, provided the `boxmux` binary is on its `PATH`:

```yaml
app:
  layouts:
    - id: main
      children:
        - id: status
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 20%}
          content: Waiting
        - id: actions
          position: {x1: 0%, y1: 20%, x2: 100%, y2: 100%}
          choices:
            - id: build
              content: Build
              script:
                - boxmux replace_box_content status "Building..."
                - if make; then boxmux replace_box_content status "Build passed"; else boxmux replace_box_content status "Build failed" --failed; fi
```

## Examples

### socat and nc

Each connection takes one request. `echo` adds the newline the request needs.

```bash
SOCK=${BOXMUX_SOCKET:-${XDG_RUNTIME_DIR:-/tmp}/boxmux.sock}   # adjust for the /tmp/boxmux-<uid>.sock fallback

echo '{"command":"query_pty_status","box_id":"shell"}' | socat - UNIX-CONNECT:"$SOCK"

# OpenBSD netcat; -N closes the connection after sending
echo '{"command":"switch_active_layout","layout_id":"logs"}' | nc -U -N "$SOCK"
```

Build requests with `jq` to get the quoting right:

```bash
jq -cn --arg text "$(uptime)" '{command:"replace_box_content",box_id:"status",content:$text}' \
  | socat - UNIX-CONNECT:"$SOCK"
```

### Python

```python
import json, os, socket

def boxmux(request, path=os.environ.get("BOXMUX_SOCKET")):
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as s:
        s.connect(path)
        s.sendall((json.dumps(request) + "\n").encode())
        reply = json.loads(s.makefile().readline())
    if not reply["ok"]:
        raise RuntimeError(reply["error"])
    return reply.get("result")

boxmux({"command": "replace_box_content", "box_id": "status", "content": "OK"})
print(boxmux({"command": "query_pty_status", "box_id": "shell"}))
```

### Driving a terminal box

```bash
boxmux spawn_pty_process shell bash
boxmux send_pty_input shell $'cd /var/log && ls\r'
boxmux query_pty_status shell
boxmux kill_pty_process shell
```
