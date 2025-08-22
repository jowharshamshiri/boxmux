---
layout: default
title: Configuration Reference - BoxMux
---

# Configuration Reference

This document provides a reference for BoxMux YAML configuration files.

## Table of Contents

- [File Structure](#file-structure)
- [Application Configuration](#application-configuration)
- [Layout Configuration](#layout-configuration)
- [Panel Configuration](#panel-configuration)
- [Position Configuration](#position-configuration)
- [Choice Configuration](#choice-configuration)
- [Color Reference](#color-reference)
- [Script Configuration](#script-configuration)
- [Chart Configuration](#chart-configuration)
- [Table Configuration](#table-configuration)
- [Plugin Configuration](#plugin-configuration)
- [Clipboard Configuration](#clipboard-configuration)
- [Scrolling Configuration](#scrolling-configuration)
- [Performance Configuration](#performance-configuration)
- [Variable System](#variable-system)
- [Schema Validation](#schema-validation)
- [Features](#features)
- [Validation Rules](#validation-rules)
- [Examples](#examples)

## File Structure

BoxMux configuration files use YAML format with the following top-level structure:

```yaml
app:
  libs:          # Optional: External script libraries
  variables:     # Optional: Global variables for template substitution
  layouts:       # Required: Layout definitions
```

## Application Configuration

### Root Application (`app`)

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `libs` | `array[string]` | No | List of external script library files |
| `variables` | `object` | No | Global variables for template substitution |
| `layouts` | `array[Layout]` | Yes | List of layout definitions |

```yaml
app:
  libs:
    - lib/utils.sh
    - lib/network.sh
  variables:
    APP_NAME: "BoxMux Dashboard"
    DEFAULT_USER: "admin"
  layouts:
    - id: 'main'
      title: '${APP_NAME}'
      # ... layout configuration
```

## Layout Configuration

Layouts define the overall structure and appearance of your interface.

### Layout Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `id` | `string` | Yes | - | Unique identifier for the layout |
| `root` | `boolean` | No | `false` | Whether this is the root/main layout |
| `title` | `string` | No | - | Layout title (shown in terminal title bar) |
| `bg_color` | `string` | No | `"black"` | Background color |
| `fg_color` | `string` | No | `"white"` | Foreground/text color |
| `title_fg_color` | `string` | No | `"white"` | Title text color |
| `title_bg_color` | `string` | No | `"black"` | Title background color |
| `selected_fg_color` | `string` | No | `"black"` | Selected element text color |
| `selected_bg_color` | `string` | No | `"white"` | Selected element background color |
| `selected_title_fg_color` | `string` | No | `"black"` | Selected title text color |
| `selected_title_bg_color` | `string` | No | `"white"` | Selected title background color |
| `border_color` | `string` | No | `"white"` | Border color |
| `selected_border_color` | `string` | No | `"yellow"` | Selected border color |
| `menu_fg_color` | `string` | No | `"white"` | Menu item text color |
| `menu_bg_color` | `string` | No | `"black"` | Menu item background color |
| `selected_menu_fg_color` | `string` | No | `"black"` | Selected menu item text color |
| `selected_menu_bg_color` | `string` | No | `"white"` | Selected menu item background color |
| `fill_char` | `char` | No | `' '` | Character used to fill empty space |
| `children` | `array[Panel]` | No | `[]` | List of child panels |

### Example Layout

```yaml
app:
  layouts:
    - id: 'dashboard'
      root: true
      title: 'My Dashboard'
      bg_color: 'black'
      fg_color: 'white'
      title_fg_color: 'yellow'
      title_bg_color: 'blue'
      selected_bg_color: 'blue'
      border_color: 'green'
      children:
        - id: 'header'
          # ... panel configuration
```

## Panel Configuration

Panels are the building blocks of your interface. They can contain content, menus, or other panels.

### Panel Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `id` | `string` | Yes | - | Unique identifier for the panel |
| `title` | `string` | No | - | Panel title shown in title bar |
| `position` | `Position` | Yes | - | Panel position and size |
| `content` | `string` | No | - | Static text content |
| `border` | `boolean` | No | `true` | Whether to show border |
| `tab_order` | `string` | No | - | Tab navigation order (numeric string) |
| `next_focus_id` | `string` | No | - | ID of next panel for custom navigation |
| `refresh_interval` | `number` | No | - | Auto-refresh interval in milliseconds |
| `script` | `array[string]` | No | - | Shell commands to execute |
| `choices` | `array[Choice]` | No | - | Interactive menu choices |
| `redirect_output` | `string` | No | - | Panel ID to redirect script output to |
| `append_output` | `boolean` | No | `false` | Whether to append or replace output |
| `on_keypress` | `object` | No | - | Keyboard event handlers |
| `variables` | `object` | No | - | Panel-local variables for template substitution |
| `overflow_behavior` | `string` | No | `"scroll"` | How to handle overflow: "scroll", "fill", "cross_out", "removed" |
| `scroll` | `boolean` | No | `false` | Enable scrolling for content |
| `auto_scroll_bottom` | `boolean` | No | `false` | Automatically scroll to bottom when new content arrives |
| `clipboard_enabled` | `boolean` | No | `false` | Enable Ctrl+C clipboard copying |
| `performance_monitoring` | `boolean` | No | `false` | Enable performance monitoring |
| `scroll_config` | `object` | No | - | Enhanced scrolling configuration |
| `min_width` | `number` | No | - | Minimum width in characters |
| `min_height` | `number` | No | - | Minimum height in characters |
| `max_width` | `number` | No | - | Maximum width in characters |
| `max_height` | `number` | No | - | Maximum height in characters |
| `children` | `array[Panel]` | No | `[]` | List of child panels |

### Styling Properties

All layout-level styling properties can be overridden at the panel level:

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `bg_color` | `string` | Inherited | Background color |
| `fg_color` | `string` | Inherited | Text color |
| `title_fg_color` | `string` | Inherited | Title text color |
| `title_bg_color` | `string` | Inherited | Title background color |
| `selected_bg_color` | `string` | Inherited | Selected background color |
| `selected_fg_color` | `string` | Inherited | Selected text color |
| `selected_title_fg_color` | `string` | Inherited | Selected title text color |
| `selected_title_bg_color` | `string` | Inherited | Selected title background color |
| `border_color` | `string` | Inherited | Border color |
| `selected_border_color` | `string` | Inherited | Selected border color |
| `fill_char` | `char` | Inherited | Fill character |
| `selected_fill_char` | `char` | Inherited | Selected fill character |

## Position Configuration

Positions define where panels appear on screen using percentage-based coordinates.

### Position Properties

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `x1` | `string` | Yes | Left edge (percentage or absolute) |
| `y1` | `string` | Yes | Top edge (percentage or absolute) |
| `x2` | `string` | Yes | Right edge (percentage or absolute) |
| `y2` | `string` | Yes | Bottom edge (percentage or absolute) |

### Position Formats

```yaml
# Percentage-based (recommended)
position:
  x1: 10%      # 10% from left edge
  y1: 20%      # 20% from top edge
  x2: 90%      # 90% from left edge (width = 80%)
  y2: 80%      # 80% from top edge (height = 60%)

# Absolute positioning (not recommended)
position:
  x1: 10       # 10 characters from left
  y1: 5        # 5 lines from top
  x2: 80       # 80 characters from left
  y2: 24       # 24 lines from top
```

### Position Examples

```yaml
# Full screen
position: { x1: 0%, y1: 0%, x2: 100%, y2: 100% }

# Top half
position: { x1: 0%, y1: 0%, x2: 100%, y2: 50% }

# Bottom half
position: { x1: 0%, y1: 50%, x2: 100%, y2: 100% }

# Left sidebar
position: { x1: 0%, y1: 0%, x2: 20%, y2: 100% }

# Centered box
position: { x1: 25%, y1: 25%, x2: 75%, y2: 75% }
```

## Choice Configuration

Choices create interactive menu items within panels.

### Choice Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `id` | `string` | Yes | - | Unique identifier for the choice |
| `content` | `string` | Yes | - | Text displayed in the menu |
| `script` | `array[string]` | No | - | Commands to execute when selected |
| `thread` | `boolean` | No | `false` | Whether to run script in background thread |
| `redirect_output` | `string` | No | - | Panel ID to send output to |
| `append_output` | `boolean` | No | `false` | Whether to append or replace output |

### Choice Example

```yaml
choices:
  - id: 'deploy'
    content: 'Deploy Application'
    script:
      - echo 'Starting deployment...'
      - ./deploy.sh
      - echo 'Deployment complete!'
    redirect_output: 'status'
    append_output: true
    
  - id: 'status'
    content: 'Check Status'
    script:
      - systemctl status myapp
    redirect_output: 'output'
```

## Color Reference

BoxMux supports the following color names:

### Basic Colors

- `black`
- `red`
- `green`
- `yellow`
- `blue`
- `magenta`
- `cyan`
- `white`

### Bright Colors

- `bright_black`
- `bright_red`
- `bright_green`
- `bright_yellow`
- `bright_blue`
- `bright_magenta`
- `bright_cyan`
- `bright_white`

### Special Colors

- `reset` - Reset to default
- `default` - Use default color

### Color Usage Examples

```yaml
# Basic usage
bg_color: 'black'
fg_color: 'white'

# Bright colors for emphasis
title_fg_color: 'bright_yellow'
selected_bg_color: 'bright_blue'

# Mixed color scheme
bg_color: 'black'
fg_color: 'green'
title_fg_color: 'bright_white'
title_bg_color: 'blue'
border_color: 'cyan'
```

## Script Configuration

Scripts define commands that panels can execute.

### Script Properties

Scripts are defined as arrays of shell commands:

```yaml
script:
  - echo 'Hello World'
  - date
  - ls -la
```

### Script Features

- **Multi-line commands**: Use YAML block syntax for complex commands
- **Environment variables**: Access environment variables in scripts
- **Pipes and redirection**: Use shell features within commands
- **Background execution**: Use `thread: true` for long-running scripts

### Script Examples

```yaml
# Simple commands
script:
  - echo 'Current time:'
  - date

# Multi-line command
script:
  - >
    echo "System info:"
    uname -a
    df -h

# Complex pipeline
script:
  - ps aux | grep -v grep | grep nginx | wc -l

# Background script
script:
  - tail -f /var/log/syslog
thread: true
```

## Chart Configuration

Charts visualize data using Unicode-based rendering.

### Chart Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `chart_type` | `string` | Yes | - | Chart type: 'bar', 'line', 'histogram' |
| `width` | `number` | No | 40 | Chart width in characters |
| `height` | `number` | No | 10 | Chart height in lines |
| `title` | `string` | No | - | Chart title |
| `x_label` | `string` | No | - | X-axis label |
| `y_label` | `string` | No | - | Y-axis label |

### Chart Data Format

Charts accept data in CSV format:

```yaml
chart_data: |
  1,10
  2,15
  3,8
  4,20
  5,25
```

### Chart Examples

```yaml
# Bar chart
- id: 'cpu_chart'
  title: 'CPU Usage'
  chart_config:
    chart_type: 'bar'
    width: 50
    height: 15
    title: 'CPU Usage Over Time'
  chart_data: |
    Mon,45
    Tue,67
    Wed,23
    Thu,89
    Fri,56

# Line chart with live data
- id: 'memory_trend'
  title: 'Memory Trend'
  refresh_interval: 2000
  chart_config:
    chart_type: 'line'
    width: 60
    height: 12
  script:
    - free | awk 'NR==2{printf "%.1f\n", $3/$2 * 100.0}'
```

## Table Configuration

Tables display structured data with sorting, filtering, and pagination.

### Table Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `headers` | `array[string]` | No | - | Column headers |
| `sortable` | `boolean` | No | false | Enable column sorting |
| `filterable` | `boolean` | No | false | Enable row filtering |
| `page_size` | `number` | No | 10 | Rows per page |
| `show_row_numbers` | `boolean` | No | false | Display row numbers |
| `zebra_striping` | `boolean` | No | false | Alternating row colors |
| `border_style` | `string` | No | 'single' | Border style: 'none', 'single', 'double', 'rounded', 'thick' |

### Table Data Format

Tables accept CSV or JSON data:

```yaml
# CSV format
table_data: |
  nginx,2.5,45MB
  mysql,15.2,312MB
  redis,0.8,28MB

# JSON format
table_data: |
  [
    {"process": "nginx", "cpu": 2.5, "memory": "45MB"},
    {"process": "mysql", "cpu": 15.2, "memory": "312MB"},
    {"process": "redis", "cpu": 0.8, "memory": "28MB"}
  ]
```

### Table Examples

```yaml
# Process monitoring table
- id: 'process_table'
  title: 'System Processes'
  table_config:
    headers: ['Process', 'CPU %', 'Memory']
    sortable: true
    filterable: true
    page_size: 15
    zebra_striping: true
    border_style: 'double'
  refresh_interval: 5000
  script:
    - ps aux --no-headers | awk '{printf "%s,%.1f,%s\n", $11, $3, $4}' | head -20

# Static data table
- id: 'config_table'
  title: 'Configuration'
  table_config:
    headers: ['Setting', 'Value', 'Description']
    show_row_numbers: true
  table_data: |
    refresh_rate,1000ms,Panel refresh interval
    socket_path,/tmp/boxmux.sock,Unix socket location
    log_level,info,Application log level
```

## Plugin Configuration

Plugins enable dynamic component loading with security validation.

### Plugin Properties

| Property | Type | Required | Default | Description |
|----------|------|----------|---------|-------------|
| `plugin_type` | `string` | Yes | - | Plugin type identifier |
| `plugin_config` | `object` | No | {} | Plugin-specific configuration |
| `security_permissions` | `array[string]` | No | [] | Required permissions |

### Plugin Examples

```yaml
# Custom visualization plugin
- id: 'custom_viz'
  title: 'Custom Visualization'
  plugin_type: 'data_visualizer'
  plugin_config:
    data_source: '/tmp/metrics.json'
    visualization_type: 'heatmap'
  security_permissions:
    - 'filesystem_read'

# External API plugin
- id: 'api_monitor'
  title: 'API Status'
  plugin_type: 'http_monitor'
  plugin_config:
    endpoints:
      - 'https://api.example.com/health'
      - 'https://api.example.com/status'
  security_permissions:
    - 'network_access'
```

## Schema Validation

BoxMux includes JSON Schema validation for configuration files.

### Validation Features

- Automatic validation on configuration load
- Detailed error messages with line/column information
- Schema validation for all configuration sections
- Type checking and required field validation

### Validation Example

When loading invalid configuration:

```bash
$ boxmux invalid-config.yaml
Error: Configuration validation failed
  --> invalid-config.yaml:15:3
   |
15 |   position: "invalid"
   |   ^^^^^^^^ expected object with x1, y1, x2, y2 properties
   |
   = help: position must be an object with percentage-based coordinates
```

## Features

### Keyboard Event Handlers

Define custom keyboard shortcuts:

```yaml
on_keypress:
  r:                    # Press 'r' key
    - echo 'Refreshing...'
    - date
  q:                    # Press 'q' key
    - exit
  'ctrl+c':            # Ctrl+C combination
    - echo 'Interrupted'
```

### Title Positioning

Control where titles appear:

```yaml
title_position: 'start'    # Left-aligned
title_position: 'center'   # Centered
title_position: 'end'      # Right-aligned
```

### Anchoring

Control how panels are anchored:

```yaml
anchor: 'TopLeft'      # Default
anchor: 'TopRight'
anchor: 'BottomLeft'
anchor: 'BottomRight'
anchor: 'Center'
```

### Overflow Behavior

Control how content overflow is handled:

```yaml
overflow_behavior: 'scroll'      # Default: enable scrolling
overflow_behavior: 'fill'        # Fill with solid block
overflow_behavior: 'cross_out'   # Cross out with X's
overflow_behavior: 'removed'     # Hide the panel
```

## Variable System

BoxMux includes hierarchical variable substitution for dynamic configuration, template-driven interfaces, and environment-specific deployments.

### Variable Syntax

Variables use the following patterns:

- `${VARIABLE}` - Standard variable substitution
- `${VARIABLE:default_value}` - Variable with fallback default
- `${VARIABLE:}` - Variable with empty default
- `$SIMPLE_VAR` - Simple environment variable (legacy support)

### Variable Precedence (Hierarchical Resolution)

Variables are resolved in strict hierarchical order:

1. **Panel-specific variables** (highest precedence, most granular)
2. **Parent panel variables** (inherited through panel hierarchy) 
3. **Layout-level variables** (layout scope)
4. **Application-global variables** (app-wide scope)
5. **Environment variables** (system fallback)
6. **Default values** (built-in fallbacks, lowest precedence)

This hierarchical approach allows child panels to override parent settings while providing sensible fallbacks.

### Variable Declaration

#### Application-level Variables

Define global variables that apply across all layouts:

```yaml
app:
  variables:
    APP_NAME: "Production Dashboard"
    SERVER_HOST: "prod-server.company.com"
    LOG_LEVEL: "info"
    DEFAULT_USER: "admin"
```

#### Panel-level Variables

Define panel-specific variables that override global settings:

```yaml
- id: 'web_server_panel'
  variables:
    SERVICE_NAME: "nginx"
    PORT: "80"
    CONFIG_PATH: "/etc/nginx"
  title: '${SERVICE_NAME} Server Status'
  script:
    - echo "Checking ${SERVICE_NAME} on port ${PORT}"
    - systemctl status ${SERVICE_NAME}
```

### Variable Inheritance

Child panels automatically inherit variables from their parents, creating a natural configuration hierarchy:

```yaml
app:
  variables:
    ENVIRONMENT: "production"
    
  layouts:
    - id: 'monitoring'
      children:
        - id: 'parent_panel'
          variables:
            SERVICE_GROUP: "web-services"
          children:
            - id: 'child_panel'
              variables:
                SERVICE_NAME: "api-gateway"
              title: '${SERVICE_NAME} in ${SERVICE_GROUP} (${ENVIRONMENT})'
              # Resolves to: "api-gateway in web-services (production)"
```

### Variable Fields

Variables work in all configuration fields:

```yaml
- id: 'dynamic_panel'
  variables:
    SERVICE: "database"
    LOG_FILE: "/var/log/postgresql.log"
  title: '${SERVICE} Monitor'           # Title substitution
  content: 'Monitoring ${SERVICE}'      # Content substitution
  script:                               # Script substitution
    - echo "Starting ${SERVICE} check"
    - tail -f ${LOG_FILE}
  redirect_output: '${SERVICE}_output'  # Redirect target substitution
  choices:
    - id: 'restart'
      content: 'Restart ${SERVICE}'     # Choice content substitution
      script:
        - systemctl restart ${SERVICE}  # Choice script substitution
```

### Advanced Examples

#### Environment-specific Configuration

```yaml
app:
  variables:
    ENVIRONMENT: "staging"
    DB_HOST: "staging-db.internal"
    API_URL: "https://api-staging.company.com"
    
  layouts:
    - id: 'deployment_dashboard'
      title: 'Deployment Dashboard - ${ENVIRONMENT}'
      children:
        - id: 'database_panel'
          variables:
            SERVICE_NAME: "PostgreSQL"
          title: '${SERVICE_NAME} Status'
          script:
            - echo "Connecting to ${DB_HOST}..."
            - pg_isready -h ${DB_HOST}
            
        - id: 'api_panel'
          variables:
            SERVICE_NAME: "API Gateway"
          title: '${SERVICE_NAME} Health'
          script:
            - curl -s ${API_URL}/health | jq .
```

#### Multi-service Monitoring

```yaml
app:
  variables:
    DEFAULT_USER: "monitor"
    SSH_KEY: "~/.ssh/monitoring_key"
    
  layouts:
    - id: 'infrastructure'
      children:
        - id: 'web_servers'
          variables:
            SERVER_TYPE: "web"
          children:
            - id: 'web1'
              variables:
                HOST: "web1.company.com"
                SERVICE: "nginx"
              title: '${SERVICE}@${HOST}'
              script:
                - ssh -i ${SSH_KEY} ${USER:${DEFAULT_USER}}@${HOST} 'systemctl status ${SERVICE}'
                
            - id: 'web2'
              variables:
                HOST: "web2.company.com" 
                SERVICE: "apache2"
              title: '${SERVICE}@${HOST}'
              script:
                - ssh -i ${SSH_KEY} ${USER:${DEFAULT_USER}}@${HOST} 'systemctl status ${SERVICE}'
```

#### Template-driven Deployment

```yaml
app:
  variables:
    DEPLOYMENT_ENV: "production"
    NAMESPACE: "default"
    
  layouts:
    - id: 'kubernetes_deploy'
      title: 'K8s Deployment - ${DEPLOYMENT_ENV}'
      children:
        - id: 'frontend_deploy'
          variables:
            APP_NAME: "frontend"
            REPLICAS: "3"
            IMAGE_TAG: "v1.2.0"
          title: 'Deploy ${APP_NAME}'
          script:
            - echo "Deploying ${APP_NAME} to ${DEPLOYMENT_ENV}"
            - kubectl set image deployment/${APP_NAME} ${APP_NAME}=${APP_NAME}:${IMAGE_TAG} -n ${NAMESPACE}
            - kubectl scale deployment/${APP_NAME} --replicas=${REPLICAS} -n ${NAMESPACE}
```

### Environment Variable Integration

Seamlessly integrate with existing environment variables:

```yaml
# These variables will use environment values if set,
# otherwise fall back to YAML-defined defaults
app:
  variables:
    LOG_LEVEL: "info"  # Overridden by $LOG_LEVEL if set
    
  layouts:
    - id: 'app_panel'
      script:
        - echo "Running with LOG_LEVEL=${LOG_LEVEL}"
        - echo "User: ${USER:unknown}"          # Uses $USER or "unknown"
        - echo "Home: ${HOME:/tmp}"             # Uses $HOME or "/tmp"
```

### Error Handling

The variable system provides clear error messages for common issues:

- **Nested variables**: `${USER:${DEFAULT}}` will show a descriptive error
- **Malformed syntax**: Missing closing braces are detected and reported
- **Circular references**: Prevented with clear error messages

### Best Practices

1. **Use hierarchical variables** for environment-specific configurations
2. **Provide meaningful defaults** to prevent empty substitutions
3. **Group related variables** at appropriate levels (app/layout/panel)
4. **Use descriptive variable names** that indicate their scope and purpose
5. **Leverage inheritance** to reduce configuration duplication

### Example Usage

```yaml
app:
  variables:
    APP_NAME: "My Dashboard"
    REFRESH_RATE: "1000"
  layouts:
    - id: 'main'
      title: '${APP_NAME} v${VERSION:1.0}'
      children:
        - id: 'panel1'
          variables:
            LOCAL_VAR: "panel-specific"
          script:
            - echo "User: ${USER:unknown}"
            - echo "App: ${APP_NAME}"
            - echo "Local: ${LOCAL_VAR}"
            - echo "Home: $HOME"
```

For detailed documentation, see [Variable System Guide](variables.md).

## Validation Rules

### Required Fields

- `app.layouts` must contain at least one layout
- Each layout must have a unique `id`
- Each panel must have a unique `id` within its layout
- Each panel must have a `position`
- Each choice must have a unique `id` and `content`

### ID Naming Rules

- IDs must be strings
- IDs should be unique within their scope
- IDs should be descriptive and meaningful
- Avoid special characters and spaces

### Position Validation

- All position values must be valid percentages or numbers
- `x1` must be less than `x2`
- `y1` must be less than `y2`
- Positions should not create zero-width or zero-height panels

### Color Validation

- All color values must be valid color names
- Colors are case-sensitive
- Invalid colors will fall back to defaults

## Examples

### Configuration Example

```yaml
app:
  libs:
    - lib/system.sh
    - lib/network.sh
  layouts:
    - id: 'main'
      root: true
      title: 'System Dashboard'
      bg_color: 'black'
      fg_color: 'white'
      title_fg_color: 'bright_yellow'
      title_bg_color: 'blue'
      selected_bg_color: 'bright_blue'
      border_color: 'green'
      children:
        - id: 'header'
          title: 'System Status'
          position:
            x1: 0%
            y1: 0%
            x2: 100%
            y2: 15%
          content: 'System Dashboard v1.0'
          border: true
          
        - id: 'sidebar'
          title: 'Navigation'
          position:
            x1: 0%
            y1: 15%
            x2: 25%
            y2: 85%
          tab_order: '1'
          choices:
            - id: 'cpu'
              content: 'CPU Usage'
              script:
                - top -l 1 | grep "CPU usage"
              redirect_output: 'main_content'
            - id: 'memory'
              content: 'Memory Usage'
              script:
                - top -l 1 | grep "PhysMem"
              redirect_output: 'main_content'
            - id: 'disk'
              content: 'Disk Usage'
              script:
                - df -h
              redirect_output: 'main_content'
              
        - id: 'main_content'
          title: 'Output'
          position:
            x1: 25%
            y1: 15%
            x2: 100%
            y2: 85%
          content: 'Select an option from the menu'
          scroll: true
          
        - id: 'footer'
          title: 'Status'
          position:
            x1: 0%
            y1: 85%
            x2: 100%
            y2: 100%
          refresh_interval: 1000
          script:
            - date
          border: false
```

### Minimal Configuration Example

```yaml
app:
  layouts:
    - id: 'simple'
      root: true
      children:
        - id: 'content'
          position:
            x1: 10%
            y1: 10%
            x2: 90%
            y2: 90%
          content: 'Hello, World!'
```

## Best Practices

1. **Use meaningful IDs**: Choose descriptive names for layouts, panels, and choices
2. **Plan your layout**: Sketch your interface before writing YAML
3. **Test incrementally**: Start with simple configurations and add complexity
4. **Use consistent styling**: Define colors at the layout level when possible
5. **Optimize refresh intervals**: Balance real-time updates with performance
6. **Handle errors gracefully**: Consider what happens when scripts fail
7. **Document your configuration**: Use YAML comments to explain complex sections

## Common Patterns

### Dashboard Layout

```yaml
# Header + Sidebar + Main + Footer
children:
  - id: 'header'
    position: { x1: 0%, y1: 0%, x2: 100%, y2: 10% }
  - id: 'sidebar'
    position: { x1: 0%, y1: 10%, x2: 20%, y2: 90% }
  - id: 'main'
    position: { x1: 20%, y1: 10%, x2: 100%, y2: 90% }
  - id: 'footer'
    position: { x1: 0%, y1: 90%, x2: 100%, y2: 100% }
```

### Grid Layout

```yaml
# 2x2 grid
children:
  - id: 'top_left'
    position: { x1: 0%, y1: 0%, x2: 50%, y2: 50% }
  - id: 'top_right'
    position: { x1: 50%, y1: 0%, x2: 100%, y2: 50% }
  - id: 'bottom_left'
    position: { x1: 0%, y1: 50%, x2: 50%, y2: 100% }
  - id: 'bottom_right'
    position: { x1: 50%, y1: 50%, x2: 100%, y2: 100% }
```

### Modal/Dialog Pattern

```yaml
# Centered modal
- id: 'modal'
  position: { x1: 25%, y1: 25%, x2: 75%, y2: 75% }
  bg_color: 'white'
  fg_color: 'black'
  border_color: 'red'
```


## Clipboard Configuration

Enable clipboard integration for copying panel content:

```yaml
# Enable clipboard for specific panel
- id: 'results_panel'
  title: 'Command Results'
  clipboard_enabled: true
  script:
    - ps aux | head -20

# Global clipboard configuration
app:
  clipboard_config:
    enabled: true
    copy_keybind: 'ctrl+c'     # Keyboard shortcut
    visual_feedback: true       # Show visual confirmation
    max_copy_size: 1048576     # Maximum bytes to copy (1MB)
```

## Scrolling Configuration

Advanced scrolling features with position preservation and navigation:

```yaml
# Enhanced scrolling panel
- id: 'scrollable_content'
  title: 'Large Output'
  scroll: true
  scroll_config:
    preserve_position: true     # Maintain position during refresh
    show_indicators: true       # Show scroll position indicators
    page_size: 10              # Lines per page scroll
    smooth_scrolling: true     # Enable smooth scrolling
    auto_scroll_new: false     # Don't auto-scroll to new content
    scroll_buffer_size: 2000   # Maximum lines to keep
  script:
    - ps aux | head -100

# Minimal scrolling setup
- id: 'simple_scroll'
  scroll: true
  script:
    - find / -name '*.log' 2>/dev/null
```

## Performance Configuration

Configure performance monitoring and optimization:

```yaml
# Performance monitoring panel
- id: 'performance'
  title: 'System Performance'
  performance_monitoring: true
  perf_config:
    collect_metrics: true       # Collect performance metrics
    show_benchmarks: true      # Display benchmark results
    memory_tracking: true      # Track memory usage
    cpu_profiling: false       # CPU profiling (development only)
  script:
    - |  
      echo "Performance Metrics:"
      echo "Memory: $(ps -o rss= -p $$) KB"
      echo "CPU: $(ps -o %cpu= -p $$)%"

# Global performance settings
app:
  performance:
    enable_benchmarks: true
    benchmark_threshold_ms: 1000  # Warn if operations exceed 1s
    memory_limit_mb: 512         # Memory limit for BoxMux
    max_refresh_rate: 10         # Maximum refreshes per second
```

For more examples and tutorials, see the [Examples](examples.md) document.
