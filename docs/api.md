---
layout: default
title: API Reference - BoxMux
---

# BoxMux API Reference

This document describes the BoxMux API for programmatic control via socket messaging, external integrations, and box management.

## Table of Contents

- [Socket API](#socket-api)
- [Message Format](#message-format)
- [Command Reference](#command-reference)
- [MuxBox Operations](#box-operations)
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
echo '{"UpdateMuxBox": {"box_id": "status", "content": "Hello World"}}' | nc -U /tmp/boxmux.sock

# Check if BoxMux is running
nc -U /tmp/boxmux.sock -w 1 < /dev/null && echo "BoxMux is running"
```

### Connection

The socket accepts JSON messages terminated by newlines. Each message should be a single JSON object.

```bash
# Multiple commands
{
  echo '{"UpdateMuxBox": {"box_id": "box1", "content": "Line 1"}}'
  echo '{"UpdateMuxBox": {"box_id": "box2", "content": "Line 2"}}'
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

### UpdateMuxBox

Update the content of a specific box.

```json
{
  "UpdateMuxBox": {
    "box_id": "target_box",
    "content": "New content for the box"
  }
}
```

**Parameters:**

- `box_id` (string): ID of the box to update
- `content` (string): New content to display

**Example:**

```bash
echo '{"UpdateMuxBox": {"box_id": "status", "content": "System Online"}}' | nc -U /tmp/boxmux.sock
```

### AppendMuxBox

Append content to a specific box.

```json
{
  "AppendMuxBox": {
    "box_id": "target_box",
    "content": "Content to append"
  }
}
```

**Parameters:**

- `box_id` (string): ID of the box to update
- `content` (string): Content to append

**Example:**

```bash
echo '{"AppendMuxBox": {"box_id": "logs", "content": "New log entry"}}' | nc -U /tmp/boxmux.sock
```

### RefreshMuxBox

Trigger a refresh of a specific box (executes its script).

```json
{
  "RefreshMuxBox": {
    "box_id": "target_box"
  }
}
```

**Parameters:**

- `box_id` (string): ID of the box to refresh

**Example:**

```bash
echo '{"RefreshMuxBox": {"box_id": "cpu_monitor"}}' | nc -U /tmp/boxmux.sock
```

### SetMuxBoxProperty

Update a specific property of a box.

```json
{
  "SetMuxBoxProperty": {
    "box_id": "target_box",
    "property": "bg_color",
    "value": "red"
  }
}
```

**Parameters:**

- `box_id` (string): ID of the box to update
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
echo '{"SetMuxBoxProperty": {"box_id": "alert", "property": "bg_color", "value": "red"}}' | nc -U /tmp/boxmux.sock
```

### ExecuteScript

Execute a script in the context of a box.

```json
{
  "ExecuteScript": {
    "box_id": "target_box",
    "script": ["echo 'Hello'", "date"],
    "append": false
  }
}
```

**Parameters:**

- `box_id` (string): ID of the box to execute script in
- `script` (array[string]): Commands to execute
- `append` (boolean): Whether to append or replace output

**Example:**

```bash
echo '{"ExecuteScript": {"box_id": "output", "script": ["ps aux | head -5"], "append": true}}' | nc -U /tmp/boxmux.sock
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

### FocusMuxBox

Focus a specific box.

```json
{
  "FocusMuxBox": {
    "box_id": "target_box"
  }
}
```

**Parameters:**

- `box_id` (string): ID of the box to focus

**Example:**

```bash
echo '{"FocusMuxBox": {"box_id": "menu"}}' | nc -U /tmp/boxmux.sock
```

## MuxBox Operations

### Getting MuxBox Information

```json
{
  "GetMuxBoxInfo": {
    "box_id": "target_box"
  }
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "id": "target_box",
    "title": "MuxBox Title",
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

### Listing All MuxBoxes

```json
{
  "ListMuxBoxes": {}
}
```

**Response:**

```json
{
  "success": true,
  "data": {
    "boxes": [
      {
        "id": "box1",
        "title": "MuxBox 1",
        "parent": "layout1"
      },
      {
        "id": "box2",
        "title": "MuxBox 2",
        "parent": "layout1"
      }
    ]
  }
}
```

### MuxBox State Management

```json
{
  "SetMuxBoxState": {
    "box_id": "target_box",
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
    "boxes": ["box1", "box2", "box3"]
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
    "active_boxes": 5,
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
    "events": ["box_update", "key_press", "focus_change"],
    "callback_url": "http://localhost:8080/events"
  }
}
```

### Event Types

- `box_update`: MuxBox content changed
- `key_press`: Key was pressed
- `focus_change`: Focus moved to different box
- `script_complete`: Script execution completed
- `error`: Error occurred

### Event Format

```json
{
  "event_type": "box_update",
  "timestamp": "2024-01-01T12:00:00Z",
  "data": {
    "box_id": "status",
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
    
    def update_box(self, box_id, content):
        command = {
            "UpdateMuxBox": {
                "box_id": box_id,
                "content": content
            }
        }
        return self.send_command(command)
    
    def refresh_box(self, box_id):
        command = {
            "RefreshMuxBox": {
                "box_id": box_id
            }
        }
        return self.send_command(command)

# Usage
client = BoxMuxClient()
client.update_box("status", "System Online")
client.refresh_box("cpu_monitor")
```

### Bash Client

```bash
#!/bin/bash

SOCKET_PATH="/tmp/boxmux.sock"

send_command() {
    local command="$1"
    echo "$command" | nc -U "$SOCKET_PATH"
}

update_box() {
    local box_id="$1"
    local content="$2"
    local command="{\"UpdateMuxBox\": {\"box_id\": \"$box_id\", \"content\": \"$content\"}}"
    send_command "$command"
}

refresh_box() {
    local box_id="$1"
    local command="{\"RefreshMuxBox\": {\"box_id\": \"$box_id\"}}"
    send_command "$command"
}

# Usage
update_box "status" "System Online"
refresh_box "cpu_monitor"
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
    
    updateMuxBox(boxId, content) {
        return this.sendCommand({
            UpdateMuxBox: {
                box_id: boxId,
                content: content
            }
        });
    }
    
    refreshMuxBox(boxId) {
        return this.sendCommand({
            RefreshMuxBox: {
                box_id: boxId
            }
        });
    }
}

// Usage
const client = new BoxMuxClient();
client.updateMuxBox('status', 'System Online');
client.refreshMuxBox('cpu_monitor');
```

## Integration Examples

### CI/CD Integration

```bash
#!/bin/bash
# deploy.sh - Update BoxMux during deployment

BOXMUX_SOCKET="/tmp/boxmux.sock"

update_status() {
    local status="$1"
    echo "{\"UpdateMuxBox\": {\"box_id\": \"deploy_status\", \"content\": \"$status\"}}" | nc -U "$BOXMUX_SOCKET"
}

update_status "Starting deployment..."

# Run deployment steps
./build.sh
update_status "Build completed"

./test.sh
update_status "Tests passed"

./deploy.sh
update_status "Deployment completed"

# Refresh monitoring boxes
echo '{"RefreshMuxBox": {"box_id": "server_status"}}' | nc -U "$BOXMUX_SOCKET"
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
        
        # Update BoxMux box
        command = {
            "UpdateMuxBox": {
                "box_id": "log_viewer",
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
app.post('/api/update-box', (req, res) => {
    const { box_id, content } = req.body;
    
    const command = {
        UpdateMuxBox: {
            box_id: box_id,
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
    echo "{\"UpdateMuxBox\": {\"box_id\": \"cpu_box\", \"content\": \"CPU: $CPU%\"}}" | nc -U "$SOCKET"
    
    # Memory usage
    MEM=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//')
    echo "{\"UpdateMuxBox\": {\"box_id\": \"mem_box\", \"content\": \"Memory: ${MEM}MB free\"}}" | nc -U "$SOCKET"
    
    # Disk usage
    DISK=$(df -h / | tail -1 | awk '{print $5}')
    echo "{\"UpdateMuxBox\": {\"box_id\": \"disk_box\", \"content\": \"Disk: $DISK used\"}}" | nc -U "$SOCKET"
    
    sleep 5
done
```

## Error Handling

### Error Response Format

```json
{
  "success": false,
  "error": "MuxBox not found",
  "error_code": "MUXBOX_NOT_FOUND",
  "details": {
    "box_id": "nonexistent_box",
    "available_boxes": ["box1", "box2"]
  }
}
```

### Common Error Codes

- `MUXBOX_NOT_FOUND`: Specified box ID does not exist
- `INVALID_COMMAND`: Unknown command type
- `INVALID_PARAMETER`: Invalid parameter value
- `SCRIPT_ERROR`: Script execution failed
- `PERMISSION_DENIED`: Insufficient permissions
- `SOCKET_ERROR`: Socket communication error

### Error Handling in Clients

```python
try:
    response = client.update_box("nonexistent", "content")
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
5. **Validation**: Validate box IDs and parameters before sending
6. **Logging**: Log API calls for debugging
7. **Timeouts**: Set appropriate timeouts for socket operations
