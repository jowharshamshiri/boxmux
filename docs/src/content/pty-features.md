---
title: PTY Features
description: Interactive terminal emulation in BoxMux boxes with keyboard interaction, ANSI processing, and process management.
---


## Table of Contents

- [Overview](#overview)
- [Basic PTY Configuration](#basic-pty-configuration)
- [Interactive Applications](#interactive-applications)
- [PTY Process Management](#pty-process-management)
- [Input and Navigation](#input-and-navigation)
- [Visual Indicators](#visual-indicators)
- [Socket Control](#socket-control)
- [Error Handling](#error-handling)
- [Best Practices](#best-practices)

## Overview

PTY (Pseudo-Terminal) features allow you to run interactive terminal applications like `vim`, `htop`, `ssh`, `less`, `nano`, and database shells within BoxMux boxes. This provides terminal multiplexing with organized box layouts.

### Key Benefits

- **Interactivity**: Keyboard input routing to terminal applications
- **Process Management**: Kill, restart, and monitor PTY processes
- **ANSI Support**: Handling of colors, cursor movements, and escape sequences
- **Scrollback Buffer**: 10,000-line circular buffer for command history
- **Visual Feedback**: Lightning bolt indicators and color-coded borders
- **Error Recovery**: Fallback to regular execution on PTY failures

## Basic PTY Configuration

Enable PTY for any box by adding `pty: true`:

```yaml
app:
  layouts:
    - id: 'main'
      children:
        - id: 'interactive_box'
          title: 'Interactive Terminal ⚡'
          pty: true
          script:
            - htop
          position:
            x1: 0%
            y1: 0%
            x2: 100%
            y2: 100%
```

### PTY vs Regular Execution

```yaml
# Regular script execution (non-interactive)
- id: 'regular_box'
  title: 'System Info'
  script:
    - ps aux | head -10
    - df -h
  refresh_interval: 5000

# PTY execution (interactive)
- id: 'pty_box'
  title: 'Interactive Top ⚡'
  pty: true
  script:
    - htop
  # No refresh_interval needed - PTY runs continuously
```

## Interactive Applications

### System Monitoring

```yaml
- id: 'htop_monitor'
  title: 'System Monitor ⚡'
  pty: true
  script:
    - htop
  position:
    x1: 0%
    y1: 0%
    x2: 50%
    y2: 50%

- id: 'iotop_monitor'
  title: 'IO Monitor ⚡'
  pty: true
  script:
    - sudo iotop
  position:
    x1: 50%
    y1: 0%
    x2: 100%
    y2: 50%
```

### Text Editors

```yaml
- id: 'vim_editor'
  title: 'Text Editor ⚡'
  pty: true
  script:
    - vim /path/to/config.yaml
  position:
    x1: 0%
    y1: 0%
    x2: 100%
    y2: 70%

- id: 'nano_editor'
  title: 'Simple Editor ⚡'
  pty: true
  script:
    - nano /etc/hosts
```

### Remote Connections

```yaml
- id: 'ssh_session'
  title: 'Production Server ⚡'
  pty: true
  script:
    - ssh user@production-server.com
  position:
    x1: 0%
    y1: 0%
    x2: 50%
    y2: 100%

- id: 'database_shell'
  title: 'Database Console ⚡'
  pty: true
  script:
    - psql -h localhost -U postgres -d myapp
  position:
    x1: 50%
    y1: 0%
    x2: 100%
    y2: 100%
```

### File Navigation

```yaml
- id: 'file_manager'
  title: 'File Manager ⚡'
  pty: true
  script:
    - ranger
    # or: mc (midnight commander)
    # or: nnn
```

## PTY Process Management

### Lifecycle Control

PTY processes have full lifecycle management:

- **Automatic Start**: Process starts when box initializes
- **Process Monitoring**: Track running/stopped/failed states  
- **Manual Control**: Kill, restart via keyboard or socket
- **Resource Cleanup**: Proper cleanup when BoxMux exits

### Process Status

PTY boxes show process information in their titles:

```
Interactive Top ⚡ [PID: 12345, Running]
SSH Session ⚡ [PID: 12346, Connected]  
Text Editor ⚡ [Process Stopped]
Database Shell ⚡ [PID: 12347, Error]
```

### Process Actions

Available process management actions:

```yaml
# In choice menus for PTY control
- id: 'pty_controls'
  title: 'PTY Controls'
  choices:
    - id: 'kill_htop'
      content: 'Kill htop process'
      script:
        - echo '{"Command": {"action": "kill_pty", "box_id": "htop_monitor"}}' | nc -U /tmp/boxmux.sock
    
    - id: 'restart_ssh'
      content: 'Restart SSH session'
      script:
        - echo '{"Command": {"action": "restart_pty", "box_id": "ssh_session"}}' | nc -U /tmp/boxmux.sock
```

## Input and Navigation

### Keyboard Input Routing

When a PTY box is focused, all keyboard input is routed directly to the running process:

- **Regular Keys**: Letters, numbers, symbols sent directly
- **Special Keys**: Arrow keys, function keys (F1-F24), navigation keys
- **Modifier Keys**: Ctrl, Alt, Shift combinations
- **Terminal Keys**: Tab, Backspace, Delete, Enter, Escape

### Navigation Keys Supported

```
Arrow Keys:     ↑ ↓ ← →
Function Keys:  F1 F2 F3 ... F24
Navigation:     Home End PageUp PageDown
Editing:        Insert Delete Backspace
Modifiers:      Ctrl+C Ctrl+Z Ctrl+D etc.
```

### Focus Management

Switch between PTY and regular boxes:

- **Tab/Shift+Tab**: Navigate between focusable boxes
- **Mouse Click**: Focus PTY box and enable input routing
- **Box Selection**: Visual indicators show which box receives input

### Scrollback Navigation

PTY boxes maintain scrollback history:

- **Circular Buffer**: 10,000 lines of command history
- **Memory Efficient**: Automatic cleanup of old content
- **Search Support**: Search through command history
- **Thread Safe**: Concurrent access from PTY reader threads

## Visual Indicators

### Box Title Indicators

PTY boxes display special visual indicators:

```
Regular Box:      "System Info"
PTY Box:          "Interactive Top ⚡" 
PTY with Process:   "SSH Session ⚡ [PID: 12345, Running]"
PTY Error State:    "Failed Process ⚡ [Process Stopped]"
```

### Border Colors

PTY boxes use distinct border colors:

- **PTY Active**: Bright cyan borders
- **PTY Error**: Red borders with error indicators
- **Regular Box**: Standard border colors

### Status Information

Process status appears in box titles:

- **PID**: Process ID when running
- **State**: Running, Stopped, Error, Connected
- **Resource Info**: Memory usage, connection status

## Socket Control

### Remote PTY Management

Control PTY processes via Unix socket:

```bash
# Kill PTY process
echo '{"Command": {"action": "kill_pty", "box_id": "htop_box"}}' | nc -U /tmp/boxmux.sock

# Restart PTY process  
echo '{"Command": {"action": "restart_pty", "box_id": "ssh_session"}}' | nc -U /tmp/boxmux.sock

# Query PTY status
echo '{"Command": {"action": "pty_status", "box_id": "vim_box"}}' | nc -U /tmp/boxmux.sock

# Send input to PTY (for automation)
echo '{"Command": {"action": "pty_input", "box_id": "database", "input": "SELECT * FROM users LIMIT 5;\n"}}' | nc -U /tmp/boxmux.sock
```

### Batch Operations

```bash
# Kill all PTY processes
for box in htop_box ssh_session vim_editor; do
  echo '{"Command": {"action": "kill_pty", "box_id": "'$box'"}}' | nc -U /tmp/boxmux.sock
done

# Restart development environment
echo '{"Command": {"action": "restart_pty", "box_id": "vim_editor"}}' | nc -U /tmp/boxmux.sock
echo '{"Command": {"action": "restart_pty", "box_id": "test_runner"}}' | nc -U /tmp/boxmux.sock
```

## Error Handling

### Automatic Fallback

When PTY fails, BoxMux automatically falls back to regular execution:

```yaml
- id: 'robust_box'
  title: 'System Status'
  pty: true  # Try PTY first
  script:
    - htop  # Interactive if PTY works, static output if PTY fails
```

### Failure Tracking

BoxMux tracks PTY failures and avoids repeated attempts:

- **Failure Threshold**: After 3 consecutive failures, avoid PTY for that box
- **Success Recovery**: Clear failure count on successful PTY startup
- **Box-Specific**: Failure tracking per box, not global

### Error States

PTY boxes can be in various error states:

```yaml
# Error state indicators in titles
"Process Name ⚡ [Process Stopped]"      # Process exited
"Process Name ⚡ [PTY Failed]"           # PTY allocation failed  
"Process Name ⚡ [Connection Lost]"      # SSH/network failure
"Process Name ⚡ [Permission Denied]"    # Insufficient permissions
```

### Error Recovery

Manual recovery options:

- **Box Restart**: Kill and restart PTY process
- **Configuration Reload**: Reload YAML with updated settings
- **Fallback Mode**: Disable PTY and use regular execution

## Best Practices

### Performance Considerations

```yaml
# Good: Specific PTY processes
- id: 'htop_box'
  pty: true
  script:
    - htop

# Avoid: Heavy output in PTY
- id: 'log_box'  
  pty: false  # Use regular execution for high-volume logs
  script:
    - tail -f /var/log/messages
  auto_scroll_bottom: true
```

### Resource Management

```yaml
# Limit concurrent PTY processes
app:
  layouts:
    - id: 'development'
      children:
        # Core interactive tools (keep PTY)
        - id: 'vim_editor'
          pty: true
          script: [vim]
        
        - id: 'htop_monitor'  
          pty: true
          script: [htop]
          
        # Background processes (regular execution)
        - id: 'build_output'
          pty: false
          streaming: true
          script: [./watch-build.sh]
```

### Security Considerations

```yaml
# Be careful with PTY and sensitive operations
- id: 'secure_session'
  title: 'Admin Console ⚡'
  pty: true
  script:
    - ssh -o ServerAliveInterval=60 admin@secure-server
  # Consider: timeout, logging, access controls
```

### Layout Design

```yaml
# Organize PTY boxes logically
app:
  layouts:
    - id: 'development'
      title: 'Development Environment'
      children:
        # Main editor (large space)
        - id: 'editor'
          title: 'Code Editor ⚡'
          pty: true
          script: [vim]
          position: {x1: 0%, y1: 0%, x2: 70%, y2: 70%}
          
        # Monitoring (side box)  
        - id: 'system'
          title: 'System Monitor ⚡'
          pty: true
          script: [htop]
          position: {x1: 70%, y1: 0%, x2: 100%, y2: 50%}
          
        # Terminal (bottom)
        - id: 'shell'
          title: 'Shell ⚡'
          pty: true
          script: [bash]
          position: {x1: 0%, y1: 70%, x2: 100%, y2: 100%}
```

### Troubleshooting

Common PTY issues and solutions:

```yaml
# Issue: PTY not starting
# Solution: Check permissions and binary paths
- id: 'debug_box'
  pty: true
  script:
    - /usr/bin/htop  # Use full path
    
# Issue: Input not working
# Solution: Ensure box is focused and check key bindings

# Issue: Display corruption
# Solution: Use ANSI processing and proper terminal size
- id: 'clean_display'
  pty: true
  script:
    - TERM=xterm-256color htop
```

PTY features provide powerful terminal multiplexing capabilities within BoxMux's organized box system, enabling complex interactive workflows with proper process management and visual organization.
