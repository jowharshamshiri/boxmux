---
title: Colored Output and Syntax Highlighting
description: Showing syntax-highlighted and colored program output in BoxMux boxes, writing scripts in YAML, and editor support for configuration files
---

# Colored Output and Syntax Highlighting

BoxMux does not highlight code itself. It shows the colors that programs write, so any highlighter that prints ANSI colors works in a box: `bat`, `pygmentize`, `highlight`, `delta`, or colored `grep`, `ls`, `git` and `jq`.

## Colored Script Output

A box script's output keeps its SGR color and style sequences: the 16 base colors, 256 colors, 24-bit colors, bold, italic, underline and the other text attributes. Each line starts in the box's own colors. Other escape sequences are removed. See [Script Output Colors](theme#script-output-colors).

Scripts in the default `Immediate` and `Thread` modes write into a pipe, not a terminal. Many programs turn their colors off in that case, so ask for color explicitly:

```yaml
- id: highlighted_source
  title: 'main.rs'
  position: {x1: 0%, y1: 0%, x2: 60%, y2: 100%}
  script:
    - bat --color=always --style=plain src/main.rs

- id: recent_commits
  title: 'Recent Commits'
  position: {x1: 60%, y1: 0%, x2: 100%, y2: 50%}
  script:
    - git -c color.ui=always log --oneline -20

- id: errors
  title: 'Errors'
  position: {x1: 60%, y1: 50%, x2: 100%, y2: 100%}
  script:
    - grep --color=always -n ERROR app.log | tail -20
```

Other common switches: `ls --color=always`, `jq -C .`, `pygmentize -f terminal256 file.py`, `diff --color=always`.

## Interactive Programs with Their Own Colors

With `execution_mode: Pty`, the script runs in a terminal inside the box. Programs see a color terminal (`TERM=xterm-256color`) and draw with their own colors, including 256-color and 24-bit color. Editors and pagers with syntax highlighting work as they do in a normal terminal:

```yaml
- id: editor
  title: 'Editor'
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  execution_mode: Pty
  tab_order: 1
  script: ['vim README.md']
```

While the box has focus, all keys except Tab and Shift+Tab go to the program.

## Writing Scripts in YAML

### Single String or List

```yaml
# A list: each element is one line of the script
- id: overview
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
  script:
    - echo "=== System Overview ==="
    - uname -a
    - uptime

# A block string: kept exactly as written, including indentation and blank lines
- id: disk_report
  position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
  script: |
    echo "=== Disk Usage ==="
    df -h | grep -v tmpfs

    echo "=== Largest Directories ==="
    du -sh /var/* 2>/dev/null | sort -h | tail -5
```

Both forms run as one script in one shell process, so variables and the current directory carry over from one line to the next. The shell is `bash` by default (`powershell` on Windows), or the one set with `app.shell`.

A list element that contains `: ` (colon and space) without quotes is read by YAML as a mapping. BoxMux turns such an element back into text, but quoting the element avoids surprises: `- 'echo "Status: ok"'`.

### Other Languages

The script is passed to the shell, so a `#!` line at the top is only a comment. To run another language, call its interpreter from the script, for example with a heredoc:

```yaml
- id: python_example
  title: Python
  position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
  script: |
    python3 - <<'EOF'
    import json, platform
    print(json.dumps({"python": platform.python_version()}, indent=2))
    EOF

- id: node_example
  title: Node.js
  position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
  script: |
    node - <<'EOF'
    const pkg = require('./package.json');
    console.log(`Project: ${pkg.name} v${pkg.version}`);
    EOF
```

Or keep longer programs in their own files and run them: `script: ['python3 tools/report.py']`.

### Variables and `${...}` in Scripts

BoxMux replaces `${NAME}` and `${NAME:default}` in scripts when it loads the file, if `NAME` is defined in the file or in the environment. Shell syntax such as `${i}` for an undefined name, `${X:-y}` or `${#list[@]}` is left for the shell, and so is `${pkg.name}` above. Write `$${NAME}` to pass `${NAME}` to the shell unchanged, or use `$NAME` without braces. See [Variables in Scripts](../variables#variables-in-scripts).

```yaml
- id: deployment
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  variables:
    SERVER_HOST: production-server.example.com
    SSH_USER: deploy
    APP_PATH: /var/www/myapp
  script:
    - 'echo "Deploying to ${SERVER_HOST}"'
    - 'ssh ${SSH_USER}@${SERVER_HOST} "cd ${APP_PATH} && git pull"'
```

Go templates such as `docker ps --format '{{.Names}}'` contain no `${` and pass through unchanged.

## Editor Support for Configuration Files

The configuration schema is in [`schemas/app_schema.json`](https://github.com/jowharshamshiri/boxmux/blob/main/schemas/app_schema.json) (JSON Schema draft 7). Editors that use the YAML language server can check a file and complete keys with it:

```yaml
# yaml-language-server: $schema=../schemas/app_schema.json
app:
  layouts:
    - id: main
      children:
        - id: hello
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          content: 'Hello'
```

BoxMux substitutes variables before it validates, but an editor checks the file as written. A setting that only becomes valid after substitution, such as `border_color: '${ACCENT}'`, is flagged by the editor but loads in BoxMux. BoxMux also performs checks a schema cannot express, such as unique ids and existing `redirect_output` targets.

## Best Practices

### YAML Formatting
- Use 2-space indentation consistently.
- Quote strings that contain `: `, `#`, or start with a special character such as `*`, `&`, `!` or `{`.
- Use `|` for scripts that span several lines.
- Write cell positions in quotes (`"12"`); unquoted numbers are rejected.

### Script Organization
- Keep long scripts in files and call them, or share functions through `app.libs`.
- Test scripts on their own before putting them in a box.
- Exit with a non-zero status on failure; the box then shows its error colors.
- Force color output (`--color=always`) when you want colors in a non-terminal box.
