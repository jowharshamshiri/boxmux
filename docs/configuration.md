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
- [Features](#features)
- [Validation Rules](#validation-rules)
- [Examples](#examples)

## File Structure

BoxMux configuration files use YAML format with the following top-level structure:

```yaml
app:
  libs:          # Optional: External script libraries
  layouts:       # Required: Layout definitions
```

## Application Configuration

### Root Application (`app`)

| Property | Type | Required | Description |
|----------|------|----------|-------------|
| `libs` | `array[string]` | No | List of external script library files |
| `layouts` | `array[Layout]` | Yes | List of layout definitions |

```yaml
app:
  libs:
    - lib/utils.sh
    - lib/network.sh
  layouts:
    - id: 'main'
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
| `overflow_behavior` | `string` | No | `"scroll"` | How to handle overflow: "scroll", "fill", "cross_out", "removed" |
| `scroll` | `boolean` | No | `false` | Enable scrolling for content |
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

For more examples and tutorials, see the [Examples](examples.md) document.
