---
layout: default
title: User Guide - BoxMux
---

# BoxMux User Guide

**Guide for building terminal interfaces with BoxMux**

## Table of Contents

- [Quick Start](#quick-start)
- [Core Concepts](#core-concepts)
- [Building Interfaces](#building-interfaces)
- [Common Patterns](#common-patterns)
- [Real-World Examples](#real-world-examples)
- [Best Practices](#best-practices)
- [Techniques](#techniques)

## Quick Start

### Your First Interface

Create a simple "Hello World" interface:

```yaml
app:
  variables:
    WELCOME_MSG: "Welcome to BoxMux!"
  layouts:
    - id: 'hello'
      root: true
      title: 'My First BoxMux App'
      children:
        - id: 'greeting'
          title: 'Hello World'
          position:
            x1: 25%
            y1: 40%
            x2: 75%
            y2: 60%
          content: '${WELCOME_MSG}'
          border: true
          border_color: 'green'
```

Run it:
```bash
# If installed via cargo install
boxmux hello.yaml

# If built from source
./run_boxmux.sh hello.yaml
```

### Interactive Menu

Add interactivity with menus:

```yaml
app:
  layouts:
    - id: 'interactive'
      root: true
      title: 'Interactive Menu'
      children:
        - id: 'menu'
          title: 'Actions'
          position: {x1: 20%, y1: 20%, x2: 80%, y2: 80%}
          tab_order: '1'
          choices:
            - id: 'hello'
              content: 'Say Hello'
              script: ['echo "Hello, World!"']
            - id: 'date'
              content: 'Show Date'  
              script: ['date']
            - id: 'files'
              content: 'List Files'
              script: ['ls -la']
```

## Core Concepts

### 1. Layouts and Panels

**Layouts** define the overall structure of your interface. **Panels** are the building blocks within layouts.

```yaml
app:
  layouts:                    # List of layout definitions
    - id: 'main'             # Unique layout identifier
      root: true             # This is the main layout
      children:              # Panels within this layout
        - id: 'panel1'       # Unique panel identifier
          title: 'Panel'     # Panel title
          position:          # Panel position and size
            x1: 10%          # Left edge (percentage of parent)
            y1: 10%          # Top edge
            x2: 90%          # Right edge
            y2: 90%          # Bottom edge
```

### 2. Positioning System

BoxMux uses percentage-based positioning for responsive design:

- `x1, y1`: Top-left corner coordinates
- `x2, y2`: Bottom-right corner coordinates
- Values are percentages of the parent container

```yaml
# Common layout patterns
# Full screen
position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}

# Top half  
position: {x1: 0%, y1: 0%, x2: 100%, y2: 50%}

# Left sidebar
position: {x1: 0%, y1: 0%, x2: 25%, y2: 100%}

# Centered window
position: {x1: 25%, y1: 25%, x2: 75%, y2: 75%}
```

### 3. Interactive Elements

Create interactive menus with choices:

```yaml
- id: 'menu_panel'
  tab_order: '1'           # Enable keyboard navigation
  choices:
    - id: 'action1'
      content: 'Menu Option 1'
      script: ['command to execute']
      redirect_output: 'output_panel'  # Send results to another panel
    - id: 'action2'
      content: 'Menu Option 2'
      script: ['another command']
```

### 4. Real-time Updates

Add live data with refresh intervals:

```yaml
- id: 'live_panel'
  title: 'Live Data'
  refresh_interval: 2000   # Update every 2 seconds
  script:
    - date                 # Commands to run on each refresh
    - uptime
```

### 5. Variable System - Dynamic Configuration

BoxMux's variable system enables template-driven interfaces with environment-specific configuration:

#### Basic Variable Usage

```yaml
app:
  variables:
    SERVER_NAME: "production-db"
    DEFAULT_PORT: "5432"
  layouts:
    - id: 'database_monitor'
      title: 'Database Monitor - ${SERVER_NAME}'
      children:
        - id: 'connection_status'
          title: 'Connection Status'
          script:
            - 'pg_isready -h ${SERVER_NAME} -p ${DEFAULT_PORT}'
            - 'echo "Connected to ${SERVER_NAME}:${DEFAULT_PORT}"'
```

#### Environment Integration

Variables seamlessly integrate with environment variables:

```yaml
app:
  variables:
    ENVIRONMENT: "development"  # Overridden by $ENVIRONMENT if set
  layouts:
    - id: 'deployment_status'
      title: 'Status - ${ENVIRONMENT}'
      children:
        - id: 'api_check'
          script:
            - 'echo "Environment: ${ENVIRONMENT}"'
            - 'echo "User: ${USER:unknown}"'       # Uses $USER or "unknown"
            - 'echo "Log Level: ${LOG_LEVEL:info}"' # Uses $LOG_LEVEL or "info"
```

#### Hierarchical Variables

Child panels inherit and can override parent variables:

```yaml
app:
  variables:
    REGION: "us-east-1"
  layouts:
    - id: 'infrastructure'
      children:
        - id: 'web_tier'
          variables:
            SERVICE_TYPE: "frontend"
            PORT: "80"
          children:
            - id: 'nginx_server'
              variables:
                SERVICE_NAME: "nginx"
                PORT: "443"  # Overrides parent PORT
              title: '${SERVICE_NAME} (${SERVICE_TYPE}) - ${REGION}'
              # Resolves to: "nginx (frontend) - us-east-1"
```

**Learn more**: See the complete [Variable System Guide](variables.md) for additional patterns and best practices.

## Building Interfaces

### Step-by-Step Interface Creation

#### 1. Plan Your Layout

Before writing YAML, sketch your interface:
```
┌─────────────────────────────┐
│           Header            │
├─────────┬───────────────────┤
│ Sidebar │    Main Content   │
│         │                   │
│         │                   │
├─────────┴───────────────────┤
│           Footer            │
└─────────────────────────────┘
```

#### 2. Implement the Structure

```yaml
app:
  layouts:
    - id: 'dashboard'
      root: true
      title: 'My Dashboard'
      children:
        # Header
        - id: 'header'
          title: 'Dashboard Header'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 15%}
          content: 'Welcome to the Dashboard'
          
        # Sidebar
        - id: 'sidebar'
          title: 'Navigation'
          position: {x1: 0%, y1: 15%, x2: 25%, y2: 85%}
          tab_order: '1'
          choices:
            - id: 'system'
              content: 'System Info'
              script: ['uname -a']
              redirect_output: 'main'
            - id: 'processes'
              content: 'Processes'
              script: ['ps aux | head -10']
              redirect_output: 'main'
              
        # Main content
        - id: 'main'
          title: 'Main Content'
          position: {x1: 25%, y1: 15%, x2: 100%, y2: 85%}
          content: 'Select an option from the sidebar'
          scroll: true
          
        # Footer
        - id: 'footer'
          title: 'Status'
          position: {x1: 0%, y1: 85%, x2: 100%, y2: 100%}
          refresh_interval: 1000
          script: ['date']
```

#### 3. Add Styling

```yaml
# At layout level for consistent styling
- id: 'dashboard'
  root: true
  title: 'Styled Dashboard'
  bg_color: 'black'
  fg_color: 'white'
  title_fg_color: 'bright_yellow'
  title_bg_color: 'blue'
  selected_bg_color: 'bright_blue'
  border_color: 'green'
  # ... children panels inherit these styles
```

## Common Patterns

### Header/Sidebar/Main Layout

```yaml
children:
  - id: 'header'
    position: {x1: 0%, y1: 0%, x2: 100%, y2: 10%}
  - id: 'sidebar'  
    position: {x1: 0%, y1: 10%, x2: 20%, y2: 100%}
  - id: 'main'
    position: {x1: 20%, y1: 10%, x2: 100%, y2: 100%}
```

### Two-Column Layout

```yaml
children:
  - id: 'left'
    position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
  - id: 'right'
    position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
```

### Grid Layout (2x2)

```yaml
children:
  - id: 'top_left'
    position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
  - id: 'top_right'
    position: {x1: 50%, y1: 0%, x2: 100%, y2: 50%}
  - id: 'bottom_left'
    position: {x1: 0%, y1: 50%, x2: 50%, y2: 100%}
  - id: 'bottom_right'
    position: {x1: 50%, y1: 50%, x2: 100%, y2: 100%}
```

## Real-World Examples

### System Monitor

```yaml
app:
  variables:
    REFRESH_FAST: "2000"
    REFRESH_SLOW: "5000"
  layouts:
    - id: 'sysmon'
      root: true
      title: 'System Monitor - ${USER:admin}'
      bg_color: 'black'
      fg_color: 'green'
      children:
        - id: 'cpu'
          title: 'CPU Usage'
          position: {x1: 5%, y1: 10%, x2: 48%, y2: 40%}
          refresh_interval: ${REFRESH_FAST}
          script:
            - top -l 1 | grep "CPU usage"
            
        - id: 'memory'
          title: 'Memory Usage'
          position: {x1: 52%, y1: 10%, x2: 95%, y2: 40%}
          refresh_interval: ${REFRESH_FAST}
          script:
            - top -l 1 | grep "PhysMem"
            
        - id: 'disk'
          title: 'Disk Usage'
          position: {x1: 5%, y1: 45%, x2: 95%, y2: 75%}
          refresh_interval: ${REFRESH_SLOW}
          script:
            - df -h | head -5
            
        - id: 'processes'
          title: 'Top Processes'
          position: {x1: 5%, y1: 80%, x2: 95%, y2: 95%}
          refresh_interval: 3000
          script:
            - ps aux | head -8
```

### Development Dashboard

```yaml
app:
  layouts:
    - id: 'dev_dash'
      root: true
      title: 'Development Dashboard'
      children:
        - id: 'git_status'
          title: 'Git Status'
          position: {x1: 5%, y1: 10%, x2: 95%, y2: 30%}
          refresh_interval: 10000
          script:
            - git status --short
            - echo "---"
            - git log --oneline -3
            
        - id: 'actions'
          title: 'Actions'
          position: {x1: 5%, y1: 35%, x2: 45%, y2: 85%}
          tab_order: '1'
          choices:
            - id: 'build'
              content: 'Build Project'
              script: ['cargo build']
              redirect_output: 'output'
            - id: 'test'
              content: 'Run Tests'
              script: ['cargo test']
              redirect_output: 'output'
            - id: 'lint'
              content: 'Run Linter'
              script: ['cargo clippy']
              redirect_output: 'output'
              
        - id: 'output'
          title: 'Output'
          position: {x1: 50%, y1: 35%, x2: 95%, y2: 85%}
          content: 'Select an action'
          scroll: true
```

### Log Monitor

```yaml
app:
  layouts:
    - id: 'logmon'
      root: true
      title: 'Log Monitor'
      children:
        - id: 'log_selector'
          title: 'Select Log'
          position: {x1: 5%, y1: 10%, x2: 25%, y2: 90%}
          tab_order: '1'
          choices:
            - id: 'syslog'
              content: 'System Log'
              script: ['tail -20 /var/log/system.log']
              redirect_output: 'log_viewer'
            - id: 'error_log'
              content: 'Error Log'
              script: ['tail -20 /var/log/error.log']
              redirect_output: 'log_viewer'
              
        - id: 'log_viewer'
          title: 'Log Contents'
          position: {x1: 30%, y1: 10%, x2: 95%, y2: 90%}
          content: 'Select a log file'
          scroll: true
```

## Best Practices

### 1. Planning and Design

- **Sketch first**: Plan your layout before writing YAML
- **Start simple**: Begin with basic panels and add complexity
- **Consider screen sizes**: Test on different terminal sizes
- **Group related content**: Keep related functions in nearby panels

### 2. Configuration Organization

```yaml
# Use meaningful IDs
- id: 'cpu_monitor'        # Good: descriptive
- id: 'panel1'             # Bad: generic

# Consistent styling
app:
  layouts:
    - id: 'main'
      # Define colors once at layout level
      bg_color: 'black'
      fg_color: 'white'
      # Panels inherit these styles
```

### 3. Performance Optimization

```yaml
# Optimize refresh intervals
- id: 'fast_updates'
  refresh_interval: 1000   # 1 second for critical data

- id: 'slow_updates'  
  refresh_interval: 30000  # 30 seconds for less critical data

# Efficient scripts
script:
  - ps aux | head -10      # Limit output
  - df -h | head -5        # Don't show all filesystems
```

### 4. Error Handling

```yaml
# Robust scripts with fallbacks
script:
  - |
    if command -v docker >/dev/null 2>&1; then
      docker ps
    else
      echo "Docker not available"
    fi
```

### 5. User Experience

```yaml
# Clear navigation with tab_order
- id: 'menu1'
  tab_order: '1'    # First tab stop
- id: 'menu2'  
  tab_order: '2'    # Second tab stop

# Helpful content and titles
- id: 'output'
  title: 'Command Output'
  content: 'Select a command from the menu to see output here'
```

## Techniques

### Dynamic Content with Scripts

```yaml
# Conditional content
script:
  - |
    if systemctl is-active nginx >/dev/null 2>&1; then
      echo "✓ Nginx: Running"
    else
      echo "✗ Nginx: Stopped"
    fi
```

### Multi-line Scripts

```yaml
script:
  - |
    echo "System Information:"
    echo "=================="
    echo "Hostname: $(hostname)"
    echo "Uptime: $(uptime)"
    echo "Load: $(uptime | awk -F'load average:' '{print $2}')"
```

### Output Redirection Patterns

```yaml
# Menu with shared output panel
- id: 'actions'
  choices:
    - id: 'action1'
      script: ['command1']
      redirect_output: 'shared_output'
    - id: 'action2'
      script: ['command2']
      redirect_output: 'shared_output'
      
- id: 'shared_output'
  title: 'Results'
  content: 'Select an action'
```

### Nested Panel Hierarchies

```yaml
# Complex nested structure
- id: 'main_container'
  position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
  children:
    - id: 'left_section'
      position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
      children:
        - id: 'top_left'
          position: {x1: 5%, y1: 5%, x2: 95%, y2: 45%}
        - id: 'bottom_left'
          position: {x1: 5%, y1: 55%, x2: 95%, y2: 95%}
    - id: 'right_section'
      position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
      # ... right side content
```

### Custom Key Bindings

```yaml
# Custom keyboard shortcuts
on_keypress:
  r:                    # Press 'r' to refresh
    - echo 'Refreshing...'
    - date
  q:                    # Press 'q' to quit
    - exit
  'ctrl+c':            # Ctrl+C handling
    - echo 'Interrupted'
```

### Integration with External APIs

```yaml
# API integration example
script:
  - |
    # Fetch weather data
    curl -s "https://api.openweathermap.org/data/2.5/weather?q=London&appid=YOUR_KEY" | 
    jq '.main.temp, .weather[0].description'
```

---

For configuration reference, see [Configuration Guide](configuration.md).  
For programmatic control, see [API Reference](api.md).  
For troubleshooting, see [Troubleshooting Guide](troubleshooting.md).