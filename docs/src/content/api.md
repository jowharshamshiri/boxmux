---
title: API Reference
description: Complete API reference for BoxMux socket messaging, external integrations, and programmatic control - commands, message formats, and integration examples
---


## Table of Contents

- [Socket API](#socket-api)
- [Message Format](#message-format)
- [Command Reference](#command-reference)
- [Box Operations](#box-operations)
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
echo '{"UpdateBox": {"box_id": "status", "content": "Hello World"}}' | nc -U /tmp/boxmux.sock

# Check if BoxMux is running
nc -U /tmp/boxmux.sock -w 1 < /dev/null && echo "BoxMux is running"
```

### Connection

The socket accepts JSON messages terminated by newlines. Each message should be a single JSON object.

```bash
# Multiple commands
{
  echo '{"UpdateBox": {"box_id": "box1", "content": "Line 1"}}'
  echo '{"UpdateBox": {"box_id": "box2", "content": "Line 2"}}'
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

### UpdateBox

Update the content of a specific box.

```json
{
  "UpdateBox": {
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
echo '{"UpdateBox": {"box_id": "status", "content": "System Online"}}' | nc -U /tmp/boxmux.sock
```

### AppendBox

Append content to a specific box.

```json
{
  "AppendBox": {
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
echo '{"AppendBox": {"box_id": "logs", "content": "New log entry"}}' | nc -U /tmp/boxmux.sock
```

### RefreshBox

Trigger a refresh of a specific box (executes its script).

```json
{
  "RefreshBox": {
    "box_id": "target_box"
  }
}
```

**Parameters:**

- `box_id` (string): ID of the box to refresh

**Example:**

```bash
echo '{"RefreshBox": {"box_id": "cpu_monitor"}}' | nc -U /tmp/boxmux.sock
```

### SetBoxProperty

Update a specific property of a box.

```json
{
  "SetBoxProperty": {
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
echo '{"SetBoxProperty": {"box_id": "alert", "property": "bg_color", "value": "red"}}' | nc -U /tmp/boxmux.sock
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

### FocusBox

Focus a specific box.

```json
{
  "FocusBox": {
    "box_id": "target_box"
  }
}
```

**Parameters:**

- `box_id` (string): ID of the box to focus

**Example:**

```bash
echo '{"FocusBox": {"box_id": "menu"}}' | nc -U /tmp/boxmux.sock
```

## Box Operations

### Getting Box Information

```json
{
  "GetBoxInfo": {
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
    "title": "Box Title",
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

### Listing All Boxes

```json
{
  "ListBoxes": {}
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
        "title": "Box 1",
        "layout": "main"
      },
      {
        "id": "box2", 
        "title": "Box 2",
        "layout": "main"
      }
    ]
  }
}
```

### Box State Management

```json
{
  "SetBoxState": {
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

## Event Handling

BoxMux can send event notifications for certain activities:

```json
{
  "event": "box_updated",
  "data": {
    "box_id": "status",
    "timestamp": "2023-01-01T12:00:00Z",
    "content": "New content"
  }
}
```

### Event Types

- `box_updated`: Box content changed
- `layout_switched`: Active layout changed
- `box_focused`: Box focus changed
- `key_pressed`: Key event occurred
- `error`: Error occurred

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
            sock.send((json.dumps(command) + '\n').encode())
            response = sock.recv(4096).decode()
            return json.loads(response) if response else None
        finally:
            sock.close()
    
    def update_box(self, box_id, content):
        return self.send_command({
            "UpdateBox": {
                "box_id": box_id,
                "content": content
            }
        })
    
    def refresh_box(self, box_id):
        return self.send_command({
            "RefreshBox": {"box_id": box_id}
        })

# Usage
client = BoxMuxClient()
client.update_box("status", "System Online")
```

### Shell Script Helper

```bash
#!/bin/bash

SOCKET_PATH="/tmp/boxmux.sock"

boxmux_cmd() {
    echo "$1" | nc -U "$SOCKET_PATH"
}

update_box() {
    local box_id="$1"
    local content="$2"
    boxmux_cmd "{\"UpdateBox\": {\"box_id\": \"$box_id\", \"content\": \"$content\"}}"
}

refresh_box() {
    local box_id="$1"
    boxmux_cmd "{\"RefreshBox\": {\"box_id\": \"$box_id\"}}"
}

# Usage
update_box "status" "System Running"
refresh_box "cpu_monitor"
```

## Integration Examples

### CI/CD Integration

Update build status in real-time:

```bash
#!/bin/bash
# build.sh

# Start build
update_box "build_status" "🔄 Building..."

# Run build
if cargo build --release; then
    update_box "build_status" "✅ Build successful"
    update_box "build_output" "$(cargo build --release 2>&1 | tail -10)"
else
    update_box "build_status" "❌ Build failed"
    update_box "build_output" "$(cargo build --release 2>&1 | tail -10)"
fi
```

### Monitoring Integration

```python
import psutil
import time

def update_system_metrics():
    client = BoxMuxClient()
    
    while True:
        # CPU usage
        cpu = psutil.cpu_percent(interval=1)
        client.update_box("cpu", f"CPU: {cpu}%")
        
        # Memory usage
        memory = psutil.virtual_memory()
        client.update_box("memory", f"Memory: {memory.percent}%")
        
        # Disk usage
        disk = psutil.disk_usage('/')
        client.update_box("disk", f"Disk: {disk.percent}%")
        
        time.sleep(5)
```

### Log Monitoring

```bash
#!/bin/bash
# Monitor log file and update BoxMux

tail -f /var/log/application.log | while read line; do
    if [[ "$line" == *"ERROR"* ]]; then
        update_box "log_status" "❌ Error detected"
        update_box "last_error" "$line"
    elif [[ "$line" == *"WARNING"* ]]; then
        update_box "log_status" "⚠️ Warning"
    else
        update_box "log_status" "✅ Normal"
    fi
    
    # Update recent logs (last 10 lines)
    tail -10 /var/log/application.log | boxmux_cmd '{"UpdateBox": {"box_id": "recent_logs", "content": "'$(cat)'"}}' 
done
```

### Web Hook Integration

```python
from flask import Flask, request
import json

app = Flask(__name__)
client = BoxMuxClient()

@app.route('/webhook', methods=['POST'])
def handle_webhook():
    data = request.json
    
    if data.get('type') == 'deployment':
        status = data.get('status')
        if status == 'success':
            client.update_box("deploy_status", "✅ Deployment successful")
        elif status == 'failed':
            client.update_box("deploy_status", "❌ Deployment failed")
        elif status == 'started':
            client.update_box("deploy_status", "🔄 Deploying...")
    
    return {'status': 'ok'}

if __name__ == '__main__':
    app.run(host='0.0.0.0', port=5000)
```

## Error Handling

### Common Error Responses

```json
{
  "success": false,
  "error": "Box not found",
  "error_code": "BOX_NOT_FOUND"
}
```

### Error Codes

- `BOX_NOT_FOUND`: Specified box ID doesn't exist
- `LAYOUT_NOT_FOUND`: Specified layout ID doesn't exist
- `INVALID_COMMAND`: Command format is invalid
- `EXECUTION_ERROR`: Script execution failed
- `PERMISSION_DENIED`: Operation not permitted
- `SOCKET_ERROR`: Communication error

### Error Handling Example

```python
def safe_update_box(box_id, content):
    try:
        response = client.update_box(box_id, content)
        if not response.get('success'):
            print(f"Error: {response.get('error')}")
            return False
        return True
    except Exception as e:
        print(f"Connection error: {e}")
        return False
```
