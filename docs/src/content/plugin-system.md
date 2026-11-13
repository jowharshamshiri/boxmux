---
title: Plugin System
description: Writing BoxMux plugins - shared libraries with a four-function C interface that draw the content of boxes from JSON requests.
---


## Table of Contents

- [Overview](#overview)
- [Security](#security)
- [Using a plugin](#using-a-plugin)
- [The plugin interface](#the-plugin-interface)
- [Errors](#errors)
- [The example plugin](#the-example-plugin)
- [Writing a plugin in C](#writing-a-plugin-in-c)

## Overview

A plugin is a shared library (`.so`, `.dylib` or `.dll`) that draws the content of boxes. The
app lists the libraries to load under `plugins`; a box names the component it wants with
`plugin_component` and passes settings in `plugin_config`. When BoxMux draws the box, it sends
the component name, the settings and the size of the box's content area to the plugin as JSON,
and shows the text the plugin returns.

Only text crosses the library boundary: every call takes and returns UTF-8 JSON in
NUL-terminated strings, so a plugin can be written in any language that can export C functions.

## Security

**A plugin runs inside the BoxMux process with the same privileges as BoxMux.** There is no
sandbox, no permission system and no signature check. Loading a library runs its initialization
code, and the plugin can then do anything BoxMux can: read and write files, start processes,
open network connections, and read the configuration and environment. A crash in a plugin
crashes BoxMux.

Only list libraries you trust, and treat a configuration file that loads plugins like a program
you are about to run.

## Using a plugin

```yaml
app:
  plugins:
    - path: plugins/gauge
  layouts:
    - id: main
      root: true
      children:
        - id: gauges
          title: Gauges
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 50%}
          plugin_component: gauge
          plugin_config:
            gauges:
              - {label: CPU, value: 42}
              - {label: Memory, value: 6.1, max: 16}
          refresh_interval: 1000
```

### `plugins`

A list of entries with one key, `path`:

- A relative path is relative to the directory of the YAML file; an absolute path is used as it
  is.
- When the file name has no extension, it names the library the way the platform does:
  `plugins/gauge` means `plugins/libgauge.so` on Linux and FreeBSD, `plugins/libgauge.dylib` on
  macOS and `plugins\gauge.dll` on Windows. A file name with an extension is used unchanged.
- Plugins are loaded once, when BoxMux starts, in the order listed.

### Box settings

| Key | Meaning |
|-----|---------|
| `plugin_component` | The component that draws the box. Some loaded plugin must offer it. |
| `plugin_config` | A map of settings passed to the plugin as the `config` object. Values can be any YAML that has a JSON equivalent. Optional; `{}` is sent when it is absent. |
| `refresh_interval` | Milliseconds. The box is redrawn at least this often, so the plugin is asked for new content. |

BoxMux asks the plugin for content whenever it draws the screen, which happens after any change
(new script output in any box, focus, resizing and so on). Without `refresh_interval`, nothing
makes a plugin box redraw on its own. Plugin output is shown from the top-left of the content
area and scrolls, wraps or is cut off according to the box's `overflow_behavior`. A box with
`plugin_component` cannot also have `chart_type`, `table_data` or `table_config`.

## The plugin interface

A plugin exports four functions with C linkage:

```c
#include <stdint.h>

uint32_t boxmux_plugin_abi_version(void);
char    *boxmux_plugin_describe(void);
char    *boxmux_plugin_render(const char *request);
void     boxmux_plugin_free(char *text);
```

### `boxmux_plugin_abi_version`

Returns the version of the interface the plugin implements. It must return `1`; any other value
stops BoxMux at start.

### `boxmux_plugin_describe`

Returns a JSON object naming the plugin and the components it offers:

```json
{"name": "gauge", "components": ["gauge"]}
```

Both keys are required and no other keys are allowed. Component names must be unique across all
loaded plugins.

### `boxmux_plugin_render`

Receives a request and returns a reply. The request is:

```json
{"component": "gauge", "config": {"gauges": [{"label": "CPU", "value": 42}]}, "width": 60, "height": 8}
```

- `component`: the box's `plugin_component`.
- `config`: the box's `plugin_config` as a JSON object (`{}` when absent).
- `width`, `height`: the size of the area the content is shown in, in cells: the inside of
  the border less a one-column margin on each side.

The reply is one of:

```json
{"content": "line one\nline two"}
```

```json
{"error": "sensor offline"}
```

Exactly one of `content` and `error` must be present, and no other keys. `content` is the text
to show; lines are separated by `\n`, and SGR color sequences (`ESC [ ... m`) and OSC 8
hyperlinks are kept while other control sequences are removed. Content larger than `width` x `height` is not an error; the
box scrolls or cuts it off as configured.

BoxMux calls `boxmux_plugin_render` from the thread that draws the screen, one call at a time.
Drawing waits for the call to return, so keep it fast: do slow work (network, disk) elsewhere
and return the latest result.

### `boxmux_plugin_free`

BoxMux copies every string returned by `boxmux_plugin_describe` and `boxmux_plugin_render`, then
passes it back to `boxmux_plugin_free` exactly once. The plugin allocates its strings and frees
them in this function, with its own allocator. The `request` string belongs to BoxMux and is
valid only during the `boxmux_plugin_render` call; the plugin must not free or keep it.

## Errors

These errors stop BoxMux at start, with a message naming the library:

- the library cannot be loaded (for example, the file does not exist);
- one of the four functions is missing;
- `boxmux_plugin_abi_version` returns a value other than `1`;
- `boxmux_plugin_describe` returns no string, a string that is not UTF-8, or JSON that is not a
  valid description;
- two plugins offer the same component;
- a box uses a `plugin_component` that no loaded plugin offers.

For example:

```
plugin /home/me/dash/plugins/libgauge.so: speaks plugin interface version 2, boxmux speaks 1
```

Errors while drawing do not stop BoxMux. The box shows `plugin '<component>': <message>`, where
the message is the plugin's `error` text, or a description of what was wrong with the reply
(no string, not UTF-8, not valid JSON, or not exactly one of `content` and `error`).

## The example plugin

The repository has a complete plugin in Rust in `examples/plugins/gauge`. It offers one
component, `gauge`, which draws a horizontal bar for each configured value, sized to the box.
Each gauge has a `label`, a `value` and an optional `max` (default `100`); unknown settings are
reported as errors.

Build it from the repository root:

```bash
cargo build --release --manifest-path examples/plugins/gauge/Cargo.toml
```

This produces `examples/plugins/gauge/target/release/libboxmux_gauge.so` (`.dylib` on macOS,
`boxmux_gauge.dll` on Windows). Then run the demo, which loads it with a path without extension:

```bash
boxmux layouts/plugin_demo.yaml
```

For a Rust plugin, the crate type is `cdylib` and each function is declared
`#[no_mangle] pub extern "C"`. Strings are returned with `CString::into_raw` and freed with
`CString::from_raw` in `boxmux_plugin_free`; see `examples/plugins/gauge/src/lib.rs`.

## Writing a plugin in C

A plugin that offers a `clock` component showing the current time:

```c
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>

/* Returns a copy of text, allocated here and freed by boxmux_plugin_free. The JSON in this
   file is written with single quotes, which the copy turns into double quotes (39 is the
   code of the single quote). */
static char *json(const char *text) {
    size_t length = strlen(text);
    char *out = malloc(length + 1);
    if (out != NULL) {
        for (size_t i = 0; i <= length; i++) {
            out[i] = text[i] == 39 ? '"' : text[i];
        }
    }
    return out;
}

uint32_t boxmux_plugin_abi_version(void) {
    return 1;
}

char *boxmux_plugin_describe(void) {
    return json("{'name': 'clock', 'components': ['clock']}");
}

char *boxmux_plugin_render(const char *request) {
    (void)request; /* this component ignores its configuration and size */
    char time_text[16];
    char reply[64];
    time_t now = time(NULL);
    strftime(time_text, sizeof time_text, "%H:%M:%S", localtime(&now));
    snprintf(reply, sizeof reply, "{'content': '%s'}", time_text);
    return json(reply);
}

void boxmux_plugin_free(char *text) {
    free(text);
}
```

Build it as a shared library:

```bash
cc -shared -fPIC -o libclock.so clock.c        # Linux, FreeBSD
cc -dynamiclib -o libclock.dylib clock.c       # macOS
```

On Windows, mark each function `__declspec(dllexport)` and build a DLL named `clock.dll`.

Use it with a refresh interval so the time updates:

```yaml
app:
  plugins:
    - path: clock
  layouts:
    - id: main
      root: true
      children:
        - id: time
          title: Time
          position: {x1: 0%, y1: 0%, x2: 30%, y2: 20%}
          plugin_component: clock
          refresh_interval: 1000
```

A real plugin reads the request with a JSON parser to get `component`, `config`, `width` and
`height`, and builds its reply with the same parser so that text is escaped correctly.
