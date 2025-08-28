---
title: BoxMux Code Highlighting
description: Syntax highlighting and code formatting in BoxMux YAML configurations
---

# BoxMux Code Highlighting

BoxMux configurations use YAML syntax and support embedded shell scripts with proper syntax highlighting in documentation and examples.

## YAML Configuration Syntax

BoxMux uses standard YAML syntax with specific structures for terminal UI definition:

```yaml
# Basic BoxMux configuration structure
title: "My Application"
refresh_rate: 100

variables:
  app_name: "MyApp"
  version: "1.0.0"

layouts:
  main:
    type: "MuxBox"
    bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "100%" }
    content: "Welcome to {{app_name}} v{{version}}"
```

## Shell Script Highlighting

BoxMux configurations often contain embedded shell scripts with proper syntax highlighting:

```yaml
boxes:
  system_monitor:
    title: "System Stats"
    script: |
      #!/bin/bash
      echo "CPU Usage:"
      top -l1 | grep "CPU usage" | awk '{print $3, $5}'
      echo "Memory:"
      vm_stat | grep -E "(Pages active)" | awk '{print $3}'
    refresh_interval: 2000
    thread: true
```

## Configuration Examples by Language

### Bash Scripts
```yaml
boxes:
  bash_example:
    script: |
      #!/bin/bash
      for i in {1..5}; do
        echo "Count: $i"
        sleep 1
      done
```

### Python Scripts
```yaml
boxes:
  python_example:
    script: |
      #!/usr/bin/env python3
      import json
      import requests
      
      response = requests.get('https://api.github.com/user')
      print(json.dumps(response.json(), indent=2))
```

### Node.js Scripts
```yaml
boxes:
  nodejs_example:
    script: |
      #!/usr/bin/env node
      const fs = require('fs');
      const package = JSON.parse(fs.readFileSync('package.json', 'utf8'));
      console.log(`Project: ${package.name} v${package.version}`);
```

### Docker Commands
```yaml
boxes:
  docker_example:
    script: |
      #!/bin/bash
      echo "Running Containers:"
      docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
      echo ""
      echo "Images:"
      docker images --format "table {{.Repository}}\t{{.Tag}}\t{{.Size}}"
```

## Advanced Configuration Patterns

### Multi-line YAML Values
```yaml
boxes:
  complex_script:
    script: |
      #!/bin/bash
      # This is a complex monitoring script
      echo "=== System Overview ==="
      uname -a
      
      echo "=== Load Average ==="
      uptime | awk '{print $NF}'
      
      echo "=== Disk Usage ==="
      df -h | grep -v tmpfs | grep -v devtmpfs
      
      echo "=== Network Connections ==="
      netstat -tuln | head -10
```

### Variable Substitution Patterns
```yaml
variables:
  server_host: "production-server.com"
  ssh_user: "deploy"
  app_path: "/var/www/myapp"

boxes:
  deployment:
    script: |
      #!/bin/bash
      echo "Deploying to {{server_host}}"
      ssh {{ssh_user}}@{{server_host}} "cd {{app_path}} && git pull && npm install"
      echo "Deployment complete!"
```

## PTY Integration Examples

### Interactive Terminal Sessions
```yaml
boxes:
  terminal:
    pty: true                    # Enable pseudo-terminal
    script: "bash"               # Start interactive bash
    title: "⚡ Interactive Terminal"
    border_color: "BrightCyan"
```

### Interactive Applications
```yaml
boxes:
  file_manager:
    pty: true
    script: "ranger"             # File manager requires PTY
    title: "📁 File Manager"
    
  process_monitor:
    pty: true
    script: "htop"               # Process monitor requires PTY
    title: "📊 Process Monitor"
    
  text_editor:
    pty: true
    script: "vim README.md"      # Text editor requires PTY
    title: "📝 Editor"
```

## Socket API Command Syntax

### CLI Command Examples
```bash
# Update box content via socket
boxmux replace-box-content "status_box" "New status: Online"

# Execute PTY commands
boxmux spawn-pty "terminal_box" --script="vim file.txt" --pty

# Send input to PTY processes  
boxmux send-pty-input "terminal_box" "Hello World\n"

# Query PTY status
boxmux query-pty-status "terminal_box"
```

### Socket Function Configuration
```yaml
# Remote control via socket commands
boxes:
  remote_controlled:
    content: "This box can be updated via socket"
    # Content will be replaced via socket commands:
    # boxmux replace-box-content "remote_controlled" "New content"
```

## Color and Styling Syntax

### ANSI Color Support
```yaml
boxes:
  colored_box:
    border_color: "BrightCyan"       # 16 ANSI colors
    foreground_color: "BrightWhite"  # Text color
    background_color: "Black"        # Background color
    title_color: "BrightYellow"      # Title color
```

### Available Colors
```yaml
# Standard ANSI colors
colors:
  - "Black"       - "Red"         - "Green"       - "Yellow"
  - "Blue"        - "Magenta"     - "Cyan"        - "White"
  - "BrightBlack" - "BrightRed"   - "BrightGreen" - "BrightYellow"  
  - "BrightBlue"  - "BrightMagenta" - "BrightCyan" - "BrightWhite"
```

## Configuration Validation

BoxMux includes JSON schema validation for YAML files:

```json
{
  "$schema": "https://raw.githubusercontent.com/jowharshamshiri/boxmux/main/schemas/app_schema.json",
  "type": "object",
  "properties": {
    "title": { "type": "string" },
    "refresh_rate": { "type": "integer", "minimum": 1 },
    "layouts": {
      "type": "object",
      "additionalProperties": { "$ref": "#/definitions/MuxBox" }
    }
  }
}
```

## Best Practices

### YAML Formatting
- Use 2-space indentation consistently
- Quote string values that contain special characters
- Use `|` for multi-line script content
- Validate YAML syntax before running

### Script Organization
- Use shebang lines for clarity (`#!/bin/bash`)
- Add comments to explain complex operations
- Test scripts independently before embedding
- Handle errors gracefully with appropriate exit codes

### Variable Usage
- Use descriptive variable names
- Document variable purposes in comments
- Prefer environment variables for sensitive data
- Use hierarchical variable scoping appropriately

BoxMux uses YAML configuration files to define terminal applications with syntax highlighting and validation support.