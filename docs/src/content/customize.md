---
title: Customization Guide
description: How to customize BoxMux applications through YAML configuration, styling, and features
---


## YAML Configuration Structure

BoxMux applications are defined using hierarchical YAML configuration files with the following top-level structure:

```yaml
# Application-level configuration
title: "My Application"
refresh_rate: 100
variables:
  app_var: "global_value"

# Layout definitions
layouts:
  main:
    type: "MuxBox"
    content: "Welcome to my app"
    # ... box configuration
  
  debug:
    type: "MuxBox" 
    # ... alternative layout

# Active layout selection
active: "main"
```

### Application-level Customization

Configure global application behavior:

```yaml
# Global application settings
title: "DevOps Dashboard"           # Application title
refresh_rate: 50                    # Global refresh interval (ms)
frame_delay: 16                     # Rendering frame delay (ms)

# Global variables available to all boxes
variables:
  environment: "production"
  api_endpoint: "https://api.example.com"
  log_level: "info"

# Global key bindings
on_keypress:
  "Ctrl+r": "refresh_all"
  "Ctrl+q": "quit"
  "F1": "show_help"
```

## Box Configuration & Styling

### Basic Box Properties

```yaml
boxes:
  header:
    type: "MuxBox"
    content: "System Monitor"
    
    # Positioning (percentage-based or absolute)
    bounds:
      x1: "0%"      # Left edge
      y1: "0%"      # Top edge  
      x2: "100%"    # Right edge
      y2: "20%"     # Bottom edge
      anchor: "TopLeft"   # Anchor point
    
    # Visual styling
    border_color: "BrightCyan"
    background_color: "Black"
    foreground_color: "White"
    fill_char: " "
    selected_fill_char: "█"
```

### Advanced Box Styling

```yaml
boxes:
  styled_box:
    # Border and color options
    border_color: "BrightBlue"      # 16 ANSI colors available
    background_color: "Black"       # Background fill color
    foreground_color: "BrightWhite" # Text color
    
    # Fill characters for content areas
    fill_char: " "                  # Default fill character
    selected_fill_char: "░"         # Fill when selected/focused
    
    # Title positioning and styling
    title: "Custom Box"
    title_color: "BrightYellow"
    
    # Focus and interaction
    focusable: true
    next_focus_id: "next_box"
    tab_order: 1
```

## Content Types & Behavior

### Script Execution Boxes

```yaml
boxes:
  system_stats:
    type: "MuxBox"
    script: |
      #!/bin/bash
      echo "CPU Usage: $(top -l1 | grep "CPU usage")"
      echo "Memory: $(vm_stat | grep "Pages active")"
      echo "Disk: $(df -h / | tail -1)"
    
    # Execution configuration
    refresh_interval: 2000          # Auto-refresh every 2 seconds  
    thread: true                    # Run in background thread
    streaming: true                 # Stream output as generated
    pty: false                      # Use regular process execution
    
    # Output redirection
    redirect_output: "log_box"      # Send output to another box
    append_output: true             # Append instead of replace
    
    # Error handling
    save_in_file: "/tmp/stats.log"  # Save output to file
    libs: ["utils.sh"]              # Include script libraries
```

### Interactive Menu Boxes

```yaml
boxes:
  main_menu:
    type: "MuxBox"
    choices:
      - name: "Deploy Application"
        script: "./deploy.sh"
        redirect_output: "deployment_log"
        streaming: true
        
      - name: "Run Tests" 
        script: "npm test"
        thread: true
        pty: true                   # Use PTY for interactive output
        
      - name: "View Logs"
        script: "tail -f /var/log/app.log"
        pty: true
        streaming: true
    
    # Menu styling
    selected_index: 0
    choice_color: "BrightGreen"
    selected_color: "BrightYellow"
```

### PTY (Interactive Terminal) Boxes

```yaml
boxes:
  terminal:
    type: "MuxBox"
    pty: true                       # Enable PTY mode
    script: "bash"                  # Start interactive bash
    
    # PTY-specific configuration  
    pty_buffer_size: 10000          # Scrollback buffer size
    pty_scroll_position: "bottom"   # Auto-scroll to bottom
    
    # Visual indicators for PTY boxes
    title: "⚡ Interactive Terminal"  # Lightning bolt prefix
    border_color: "BrightCyan"       # Distinctive border color
```

## Layout Management

### Multi-Layout Applications

```yaml
# Define multiple layouts for different views
layouts:
  dashboard:
    type: "MuxBox"
    children:
      header: { bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "10%" }}
      stats: { bounds: { x1: "0%", y1: "10%", x2: "50%", y2: "100%" }}
      logs: { bounds: { x1: "50%", y1: "10%", x2: "100%", y2: "100%" }}
  
  monitoring:
    type: "MuxBox" 
    children:
      cpu_graph: { bounds: { x1: "0%", y1: "0%", x2: "33%", y2: "50%" }}
      memory_graph: { bounds: { x1: "33%", y1: "0%", x2: "66%", y2: "50%" }}
      disk_graph: { bounds: { x1: "66%", y1: "0%", x2: "100%", y2: "50%" }}
      alerts: { bounds: { x1: "0%", y1: "50%", x2: "100%", y2: "100%" }}

# Set the active layout
active: "dashboard"

# Switch between layouts using key bindings
on_keypress:
  "F2": "switch_layout:monitoring"
  "F3": "switch_layout:dashboard"
```

### Nested Box Hierarchies

```yaml
boxes:
  main_container:
    type: "MuxBox"
    bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "100%" }
    children:
      sidebar:
        bounds: { x1: "0%", y1: "0%", x2: "25%", y2: "100%" }
        children:
          menu: 
            bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "80%" }
            choices: [...]
          status:
            bounds: { x1: "0%", y1: "80%", x2: "100%", y2: "100%" }
            script: "uptime"
      
      content:
        bounds: { x1: "25%", y1: "0%", x2: "100%", y2: "100%" }
        # Main content area
```

## Advanced Customization Features

### Variable System & Templates

```yaml
# Hierarchical variable definition
variables:
  # Application-level variables
  app_name: "DevOps Dashboard"
  version: "1.0.0"
  
# Box-level variables (override app-level)
boxes:
  header:
    variables:
      title_text: "{{app_name}} v{{version}}"
    content: "Welcome to {{title_text}}"
    
  api_status:
    variables:
      endpoint: "{{api_endpoint}}/status"
    script: "curl -s {{endpoint}} | jq '.status'"
```

Environment variables are automatically available:

```yaml
boxes:
  env_info:
    content: |
      Current user: {{USER}}
      Home directory: {{HOME}}
      Path: {{PATH}}
```

### Data Visualization

```yaml
# Table display with CSV/JSON data
boxes:
  data_table:
    type: "MuxBox"
    table:
      data_source: "csv"
      data: |
        Name,CPU,Memory,Status
        web-01,45%,67%,Running
        web-02,23%,54%,Running
        db-01,78%,89%,Warning
      
      # Table styling
      headers: true
      sortable: true
      filterable: true
      zebra_striping: true
      border_style: "rounded"

# Chart visualization  
boxes:
  cpu_chart:
    type: "MuxBox"
    chart:
      type: "line"
      data: [45, 52, 48, 65, 71, 58, 62]
      labels: ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
      title: "CPU Usage Over Time"
      y_axis_label: "Percentage"
```

### Content Overflow Handling

```yaml
boxes:
  log_viewer:
    type: "MuxBox"
    overflow_behavior: "scroll"     # scroll | wrap | fill | cross_out
    
    # Scrolling configuration
    scrollable: true
    auto_scroll: true               # Auto-scroll to bottom
    vertical_scroll: "0%"           # Initial scroll position
    horizontal_scroll: "0%"
    
    # Content that exceeds box bounds will be scrollable
    content: |
      Very long log content that exceeds the box dimensions...
      Line 1 of many log entries
      Line 2 of many log entries
      ...many more lines...
```

### Interactive Features

```yaml
# Mouse interaction support
boxes:
  interactive_box:
    type: "MuxBox"
    focusable: true
    
    # Click handlers
    on_click: "handle_click_action"
    
    # Resize behavior
    resizable: true                 # Allow mouse resize
    min_width: "10%"               # Minimum dimensions
    min_height: "5%"
    
    # Z-index for overlapping boxes
    z_index: 10                    # Higher values appear on top

# Global hot keys for instant actions  
hot_keys:
  "Ctrl+d": 
    choice_id: "deploy_action"
    box_id: "main_menu"
  "Ctrl+t":
    choice_id: "run_tests" 
    box_id: "actions"
```

## Color Schemes & Themes

BoxMux supports the full 16-color ANSI palette:

```yaml
# Standard colors
colors:
  normal: ["Black", "Red", "Green", "Yellow", "Blue", "Magenta", "Cyan", "White"]
  bright: ["BrightBlack", "BrightRed", "BrightGreen", "BrightYellow", 
           "BrightBlue", "BrightMagenta", "BrightCyan", "BrightWhite"]

# Theme example
theme:
  primary: "BrightBlue"
  secondary: "BrightCyan" 
  success: "BrightGreen"
  warning: "BrightYellow"
  danger: "BrightRed"
  muted: "BrightBlack"

# Apply theme to boxes
boxes:
  success_box:
    border_color: "{{theme.success}}"
    foreground_color: "{{theme.success}}"
  warning_box:
    border_color: "{{theme.warning}}"
    foreground_color: "{{theme.warning}}"
```

## Performance & Optimization

### Refresh Rate Tuning

```yaml
# Global refresh settings
refresh_rate: 50                    # Global refresh interval (ms)
frame_delay: 16                     # Target 60 FPS rendering

# Per-box refresh rates
boxes:
  fast_updates:
    refresh_interval: 100           # Update every 100ms
    
  slow_updates:
    refresh_interval: 5000          # Update every 5 seconds
    
  on_demand:
    # No refresh_interval = manual refresh only
```

### Memory Management

```yaml
# PTY buffer management
boxes:
  terminal:
    pty: true
    pty_buffer_size: 5000           # Limit scrollback to 5k lines
    
# Large content handling
boxes:
  large_logs:
    overflow_behavior: "scroll"
    max_content_lines: 1000         # Limit content buffer size
```

## Plugin Integration

```yaml
boxes:
  custom_component:
    type: "MuxBox"
    plugin: "my_custom_plugin"      # Load custom plugin
    plugin_config:
      setting1: "value1"
      setting2: "value2"
```

BoxMux uses YAML configuration files to define terminal applications with nested components and variable substitution.