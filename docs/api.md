---
layout: default
title: API Reference - BoxMux
---

# BoxMux API Reference

This document describes the BoxMux API for programmatic control via socket messaging and external integrations.

## Table of Contents

- [Socket API](#socket-api)
- [Message Format](#message-format)
- [Command Reference](#command-reference)
- [Panel Operations](#panel-operations)
- [Layout Operations](#layout-operations)
- [System Operations](#system-operations)
- [Event Handling](#event-handling)
- [Client Libraries](#client-libraries)
- [Integration Examples](#integration-examples)

## Socket API

BoxMux provides a Unix socket interface for real-time communication and control. The socket is created at `/tmp/boxmux.sock` by default.

### Basic Usage

```bash
# Send a command to BoxMux
echo '{"UpdatePanel": {"panel_id": "status", "content": "Hello World"}}' | nc -U /tmp/boxmux.sock

# Check if BoxMux is running
nc -U /tmp/boxmux.sock -w 1 < /dev/null && echo "BoxMux is running"
```

### Connection

The socket accepts JSON messages terminated by newlines. Each message should be a single JSON object.

```bash
# Multiple commands
{
  echo '{"UpdatePanel": {"panel_id": "panel1", "content": "Line 1"}}'
  echo '{"UpdatePanel": {"panel_id": "panel2", "content": "Line 2"}}'
} | nc -U /tmp/boxmux.sock
```

## Message Format

All messages are JSON objects with a command type as the root key:

```json
{
  "CommandType": {
    "parameter1": "value1",
    "parameter2": "value2"
  }
}
```

### Response Format

BoxMux may send responses for certain commands:

```json
{
  "success": true,
  "message": "Command executed successfully",
  "data": {
    "additional": "information"
  }
}
```

## Command Reference

### UpdatePanel

Update the content of a specific panel.

```json
{
  "UpdatePanel": {
    "panel_id": "target_panel",
    "content": "New content for the panel"
  }
}
```

**Parameters:**

- `panel_id` (string): ID of the panel to update
- `content` (string): New content to display

**Example:**

```bash
echo '{"UpdatePanel": {"panel_id": "status", "content": "System Online"}}' | nc -U /tmp/boxmux.sock
```

### AppendPanel

Append content to a specific panel.

```json
{
  "AppendPanel": {
    "panel_id": "target_panel",
    "content": "Content to append"
  }
}
```

**Parameters:**

- `panel_id` (string): ID of the panel to update
- `content` (string): Content to append

**Example:**

```bash
echo '{"AppendPanel": {"panel_id": "logs", "content": "New log entry"}}' | nc -U /tmp/boxmux.sock
```

### RefreshPanel

Trigger a refresh of a specific panel (executes its script).

```json
{
  "RefreshPanel": {
    "panel_id": "target_panel"
  }
}
```

**Parameters:**

- `panel_id` (string): ID of the panel to refresh

**Example:**

```bash
echo '{"RefreshPanel": {"panel_id": "cpu_monitor"}}' | nc -U /tmp/boxmux.sock
```

### SetPanelProperty

Update a specific property of a panel.

```json
{
  "SetPanelProperty": {
    "panel_id": "target_panel",
    "property": "bg_color",
    "value": "red"
  }
}
```

**Parameters:**

- `panel_id` (string): ID of the panel to update
- `property` (string): Property name to update
- `value` (any): New value for the property

**Supported Properties:**

- `bg_color`, `fg_color`
- `title_bg_color`, `title_fg_color`
- `border_color`, `selected_border_color`
- `refresh_interval`
- `content`
- `title`

**Example:**

```bash
echo '{"SetPanelProperty": {"panel_id": "alert", "property": "bg_color", "value": "red"}}' | nc -U /tmp/boxmux.sock
```

### ExecuteScript

Execute a script in the context of a panel.

```json
{
  "ExecuteScript": {
    "panel_id": "target_panel",
    "script": ["echo 'Hello'", "date"],
    "append": false
  }
}
```

**Parameters:**

- `panel_id` (string): ID of the panel to execute script in
- `script` (array[string]): Commands to execute
- `append` (boolean): Whether to append or replace output

**Example:**

```bash
echo '{"ExecuteScript": {"panel_id": "output", "script": ["ps aux | head -5"], "append": true}}' | nc -U /tmp/boxmux.sock
```

### SendKey

Send a key press to BoxMux.

```json
{
  "SendKey": {
    "key": "Tab"
  }
}
```

**Parameters:**

- `key` (string): Key to send (e.g., "Tab", "Enter", "Escape", "a", "1")

**Example:**

```bash
echo '{"SendKey": {"key": "Tab"}}' | nc -U /tmp/boxmux.sock
```

### FocusPanel

Focus a specific panel.

```json
{
  "FocusPanel": {
    "panel_id": "target_panel"
  }
}
```

**Parameters:**

- `panel_id` (string): ID of the panel to focus

**Example:**

```bash
echo '{"FocusPanel": {"panel_id": "menu"}}' | nc -U /tmp/boxmux.sock
```

## Panel Operations

### Getting Panel Information

```json
{
  "GetPanelInfo": {
    "panel_id": "target_panel"
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "target_panel",
    "title": "Panel Title",
    "content": "Current content",
    "position": {"x1": "10%", "y1": "10%", "x2": "90%", "y2": "90%"},
    "properties": {
      "bg_color": "black",
      "fg_color": "white",
      "refresh_interval": 5000
    }
  }
}
```

### Listing All Panels

```json
{
  "ListPanels": {}
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "panels": [
      {
        "id": "panel1",
        "title": "Panel 1",
        "parent": "layout1"
      },
      {
        "id": "panel2",
        "title": "Panel 2",
        "parent": "layout1"
      }
    ]
  }
}
```

### Panel State Management

```json
{
  "SetPanelState": {
    "panel_id": "target_panel",
    "state": {
      "visible": true,
      "enabled": true,
      "selected": false
    }
  }
}
```

## Layout Operations

### Switch Layout

```json
{
  "SwitchLayout": {
    "layout_id": "new_layout"
  }
}
```

### Get Current Layout

```json
{
  "GetCurrentLayout": {}
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "layout_id": "current_layout",
    "title": "Current Layout Title",
    "panels": ["panel1", "panel2", "panel3"]
  }
}
```

### Reload Configuration

```json
{
  "ReloadConfig": {
    "config_file": "/path/to/new/config.yaml"
  }
}
```

## System Operations

### Get Application Status

```json
{
  "GetStatus": {}
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "version": "0.76.71205",
    "uptime": "00:15:32",
    "current_layout": "dashboard",
    "active_panels": 5,
    "socket_path": "/tmp/boxmux.sock",
    "config_file": "/path/to/config.yaml"
  }
}
```

### Shutdown Application

```json
{
  "Shutdown": {}
}
```

### Restart Application

```json
{
  "Restart": {}
}
```

## Event Handling

### Subscribe to Events

```json
{
  "Subscribe": {
    "events": ["panel_update", "key_press", "focus_change"],
    "callback_url": "http://localhost:8080/events"
  }
}
```

### Event Types

- `panel_update`: Panel content changed
- `key_press`: Key was pressed
- `focus_change`: Focus moved to different panel
- `script_complete`: Script execution completed
- `error`: Error occurred

### Event Format

```json
{
  "event_type": "panel_update",
  "timestamp": "2024-01-01T12:00:00Z",
  "data": {
    "panel_id": "status",
    "old_content": "Offline",
    "new_content": "Online"
  }
}
```

## Client Libraries

### Python Client

```python
import socket
import json

class BoxMuxClient:
    def __init__(self, socket_path='/tmp/boxmux.sock'):
        self.socket_path = socket_path
    
    def send_command(self, command):
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            sock.connect(self.socket_path)
            sock.send(json.dumps(command).encode() + b'\n')
            response = sock.recv(1024)
            return json.loads(response.decode())
        finally:
            sock.close()
    
    def update_panel(self, panel_id, content):
        command = {
            "UpdatePanel": {
                "panel_id": panel_id,
                "content": content
            }
        }
        return self.send_command(command)
    
    def refresh_panel(self, panel_id):
        command = {
            "RefreshPanel": {
                "panel_id": panel_id
            }
        }
        return self.send_command(command)

# Usage
client = BoxMuxClient()
client.update_panel("status", "System Online")
client.refresh_panel("cpu_monitor")
```

### Bash Client

```bash
#!/bin/bash

SOCKET_PATH="/tmp/boxmux.sock"

send_command() {
    local command="$1"
    echo "$command" | nc -U "$SOCKET_PATH"
}

update_panel() {
    local panel_id="$1"
    local content="$2"
    local command="{\"UpdatePanel\": {\"panel_id\": \"$panel_id\", \"content\": \"$content\"}}"
    send_command "$command"
}

refresh_panel() {
    local panel_id="$1"
    local command="{\"RefreshPanel\": {\"panel_id\": \"$panel_id\"}}"
    send_command "$command"
}

# Usage
update_panel "status" "System Online"
refresh_panel "cpu_monitor"
```

### Node.js Client

```javascript
const net = require('net');

class BoxMuxClient {
    constructor(socketPath = '/tmp/boxmux.sock') {
        this.socketPath = socketPath;
    }
    
    sendCommand(command) {
        return new Promise((resolve, reject) => {
            const client = net.createConnection(this.socketPath, () => {
                client.write(JSON.stringify(command) + '\n');
            });
            
            client.on('data', (data) => {
                try {
                    const response = JSON.parse(data.toString());
                    resolve(response);
                } catch (e) {
                    reject(e);
                }
                client.end();
            });
            
            client.on('error', reject);
        });
    }
    
    updatePanel(panelId, content) {
        return this.sendCommand({
            UpdatePanel: {
                panel_id: panelId,
                content: content
            }
        });
    }
    
    refreshPanel(panelId) {
        return this.sendCommand({
            RefreshPanel: {
                panel_id: panelId
            }
        });
    }
}

// Usage
const client = new BoxMuxClient();
client.updatePanel('status', 'System Online');
client.refreshPanel('cpu_monitor');
```

## Integration Examples

### CI/CD Integration

```bash
#!/bin/bash
# deploy.sh - Update BoxMux during deployment

BOXMUX_SOCKET="/tmp/boxmux.sock"

update_status() {
    local status="$1"
    echo "{\"UpdatePanel\": {\"panel_id\": \"deploy_status\", \"content\": \"$status\"}}" | nc -U "$BOXMUX_SOCKET"
}

update_status "Starting deployment..."

# Run deployment steps
./build.sh
update_status "Build completed"

./test.sh
update_status "Tests passed"

./deploy.sh
update_status "Deployment completed"

# Refresh monitoring panels
echo '{"RefreshPanel": {"panel_id": "server_status"}}' | nc -U "$BOXMUX_SOCKET"
```

### Log Monitoring

```python
#!/usr/bin/env python3
import time
import json
import socket
from watchdog.observers import Observer
from watchdog.events import FileSystemEventHandler

class LogHandler(FileSystemEventHandler):
    def __init__(self, socket_path):
        self.socket_path = socket_path
    
    def on_modified(self, event):
        if event.is_directory:
            return
        
        # Read last lines of log file
        with open(event.src_path, 'r') as f:
            lines = f.readlines()
            recent_lines = ''.join(lines[-10:])
        
        # Update BoxMux panel
        command = {
            "UpdatePanel": {
                "panel_id": "log_viewer",
                "content": recent_lines
            }
        }
        
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        try:
            sock.connect(self.socket_path)
            sock.send(json.dumps(command).encode() + b'\n')
        finally:
            sock.close()

# Monitor log file
observer = Observer()
handler = LogHandler('/tmp/boxmux.sock')
observer.schedule(handler, '/var/log/myapp.log', recursive=False)
observer.start()

try:
    while True:
        time.sleep(1)
except KeyboardInterrupt:
    observer.stop()
observer.join()
```

### Web Dashboard Integration

```javascript
// Express.js endpoint to update BoxMux
app.post('/api/update-panel', (req, res) => {
    const { panel_id, content } = req.body;
    
    const command = {
        UpdatePanel: {
            panel_id: panel_id,
            content: content
        }
    };
    
    const client = net.createConnection('/tmp/boxmux.sock', () => {
        client.write(JSON.stringify(command) + '\n');
    });
    
    client.on('data', (data) => {
        const response = JSON.parse(data.toString());
        res.json(response);
        client.end();
    });
    
    client.on('error', (err) => {
        res.status(500).json({ error: err.message });
    });
});
```

### System Monitoring Integration

```bash
#!/bin/bash
# system_monitor.sh - Send system metrics to BoxMux

SOCKET="/tmp/boxmux.sock"

while true; do
    # CPU usage
    CPU=$(top -l 1 | grep "CPU usage" | awk '{print $3}' | sed 's/%//')
    echo "{\"UpdatePanel\": {\"panel_id\": \"cpu_panel\", \"content\": \"CPU: $CPU%\"}}" | nc -U "$SOCKET"
    
    # Memory usage
    MEM=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//')
    echo "{\"UpdatePanel\": {\"panel_id\": \"mem_panel\", \"content\": \"Memory: ${MEM}MB free\"}}" | nc -U "$SOCKET"
    
    # Disk usage
    DISK=$(df -h / | tail -1 | awk '{print $5}')
    echo "{\"UpdatePanel\": {\"panel_id\": \"disk_panel\", \"content\": \"Disk: $DISK used\"}}" | nc -U "$SOCKET"
    
    sleep 5
done
```

## Error Handling

### Error Response Format

```json
{
  "success": false,
  "error": "Panel not found",
  "error_code": "PANEL_NOT_FOUND",
  "details": {
    "panel_id": "nonexistent_panel",
    "available_panels": ["panel1", "panel2"]
  }
}
```

### Common Error Codes

- `PANEL_NOT_FOUND`: Specified panel ID does not exist
- `INVALID_COMMAND`: Unknown command type
- `INVALID_PARAMETER`: Invalid parameter value
- `SCRIPT_ERROR`: Script execution failed
- `PERMISSION_DENIED`: Insufficient permissions
- `SOCKET_ERROR`: Socket communication error

### Error Handling in Clients

```python
try:
    response = client.update_panel("nonexistent", "content")
    if not response.get("success", False):
        print(f"Error: {response.get('error', 'Unknown error')}")
except Exception as e:
    print(f"Connection error: {e}")
```

## Best Practices

1. **Connection Management**: Close socket connections promptly
2. **Error Handling**: Always check response status
3. **Rate Limiting**: Avoid overwhelming BoxMux with too many requests
4. **Batching**: Group related updates when possible
5. **Validation**: Validate panel IDs and parameters before sending
6. **Logging**: Log API calls for debugging
7. **Timeouts**: Set appropriate timeouts for socket operations
