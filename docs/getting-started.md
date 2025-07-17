# Getting Started with BoxMux

Welcome to BoxMux! This guide will help you create your first terminal interface and understand the core concepts.

## Prerequisites

Before starting, ensure you have:

- **Rust** (latest stable version) - [Install Rust](https://rustup.rs/)
- **Git** for cloning the repository
- **Basic terminal knowledge** - familiarity with command line
- **Text editor** - any editor that supports YAML syntax

## Installation

### Step 1: Clone and Build

```bash
# Clone the repository
git clone https://github.com/jowharshamshiri/boxmux.git
cd boxmux

# Build the project
cargo build --release

# Make the runner script executable
chmod +x run_boxmux.sh
```

### Step 2: Test the Installation

```bash
# Run the example dashboard
./run_boxmux.sh layouts/dashboard.yaml
```

If successful, you should see a rich terminal interface with multiple panels, menus, and real-time updates. Use:

- **Tab** to navigate between panels
- **Arrow keys** to navigate menu items
- **Enter** to select menu items
- **Ctrl+C** to exit

## Your First Interface

Let's create a simple interface to understand the basics.

### Hello World Example

Create a file called `hello.yaml`:

```yaml
app:
  layouts:
    - id: 'main'
      root: true
      title: 'My First BoxMux App'
      bg_color: 'black'
      title_fg_color: 'white'
      title_bg_color: 'blue'
      children:
        - id: 'welcome'
          title: 'Welcome'
          position:
            x1: 10%
            y1: 20%
            x2: 90%
            y2: 80%
          content: 'Hello, BoxMux World!'
          border: true
          border_color: 'green'
```

Run it:

```bash
./run_boxmux.sh hello.yaml
```

### Understanding the Structure

```yaml
app:                    # Root application configuration
  layouts:              # List of layout definitions
    - id: 'main'        # Unique identifier for this layout
      root: true        # This is the main/root layout
      title: 'My App'   # Title shown in the interface
      children:         # List of child panels
        - id: 'panel1'  # Unique panel identifier
          title: 'Panel Title'
          position:     # Panel positioning
            x1: 10%     # Left edge (10% from screen left)
            y1: 20%     # Top edge (20% from screen top)
            x2: 90%     # Right edge (90% from screen left)
            y2: 80%     # Bottom edge (80% from screen top)
          content: 'Text content'
```

## Core Concepts

### 1. Layouts

A layout is a interface definition. Each layout can contain multiple panels arranged in a hierarchy.

### 2. Panels

Panels are the building blocks of your interface. They can contain:

- Static text content
- Dynamic content from scripts
- Interactive menus
- Nested child panels

### 3. Positioning

BoxMux uses percentage-based positioning:

- `x1, y1`: Top-left corner
- `x2, y2`: Bottom-right corner
- Values are percentages of the parent container

### 4. Hierarchy

Panels can contain child panels, creating nested layouts:

```yaml
children:
  - id: 'parent'
    children:
      - id: 'child1'
      - id: 'child2'
```

## Interactive Elements

### Adding a Menu

```yaml
- id: 'menu'
  title: 'Options'
  position:
    x1: 10%
    y1: 10%
    x2: 50%
    y2: 60%
  tab_order: 1              # Makes this panel navigable
  choices:
    - id: 'option1'
      content: 'Say Hello'
      script:
        - echo 'Hello from script!'
    - id: 'option2'
      content: 'Show Date'
      script:
        - date
```

### Adding Real-time Updates

```yaml
- id: 'clock'
  title: 'Current Time'
  position:
    x1: 60%
    y1: 10%
    x2: 90%
    y2: 30%
  refresh_interval: 1000    # Update every second
  script:
    - date '+%H:%M:%S'
```

## Step-by-Step Tutorial

### Tutorial 1: System Monitor

Let's create a simple system monitor:

```yaml
app:
  layouts:
    - id: 'monitor'
      root: true
      title: 'System Monitor'
      bg_color: 'black'
      children:
        - id: 'cpu'
          title: 'CPU Usage'
          position:
            x1: 5%
            y1: 10%
            x2: 45%
            y2: 50%
          refresh_interval: 2000
          script:
            - top -l 1 | grep "CPU usage" | head -1
        
        - id: 'memory'
          title: 'Memory Usage'
          position:
            x1: 55%
            y1: 10%
            x2: 95%
            y2: 50%
          refresh_interval: 2000
          script:
            - top -l 1 | grep "PhysMem" | head -1
        
        - id: 'disk'
          title: 'Disk Usage'
          position:
            x1: 5%
            y1: 60%
            x2: 95%
            y2: 90%
          refresh_interval: 5000
          script:
            - df -h | head -5
```

### Tutorial 2: Interactive Dashboard

```yaml
app:
  layouts:
    - id: 'dashboard'
      root: true
      title: 'Interactive Dashboard'
      children:
        - id: 'menu'
          title: 'Actions'
          position:
            x1: 5%
            y1: 10%
            x2: 30%
            y2: 80%
          tab_order: 1
          choices:
            - id: 'status'
              content: 'System Status'
              script:
                - uname -a
              redirect_output: 'output'
            - id: 'processes'
              content: 'Top Processes'
              script:
                - ps aux | head -10
              redirect_output: 'output'
            - id: 'network'
              content: 'Network Info'
              script:
                - ifconfig | grep inet
              redirect_output: 'output'
        
        - id: 'output'
          title: 'Output'
          position:
            x1: 35%
            y1: 10%
            x2: 95%
            y2: 80%
          content: 'Select an option from the menu'
```

## Key Features to Explore

### 1. Styling and Colors

```yaml
bg_color: 'black'
fg_color: 'white'
title_fg_color: 'yellow'
title_bg_color: 'blue'
border_color: 'green'
selected_bg_color: 'blue'
```

### 2. Keyboard Shortcuts

```yaml
on_keypress:
  r:                    # Press 'r' to refresh
    - echo 'Refreshed!'
  q:                    # Press 'q' to quit
    - exit
```

### 3. Script Integration

```yaml
script:
  - echo 'Multi-line'
  - echo 'script'
  - date
  - ls -la
```

### 4. Output Redirection

```yaml
redirect_output: 'target_panel_id'    # Send output to another panel
append_output: true                   # Append instead of replace
```

## Common Patterns

### Header/Footer Layout

```yaml
children:
  - id: 'header'
    position: { x1: 0%, y1: 0%, x2: 100%, y2: 10% }
  - id: 'content'
    position: { x1: 0%, y1: 10%, x2: 100%, y2: 90% }
  - id: 'footer'
    position: { x1: 0%, y1: 90%, x2: 100%, y2: 100% }
```

### Sidebar Layout

```yaml
children:
  - id: 'sidebar'
    position: { x1: 0%, y1: 0%, x2: 20%, y2: 100% }
  - id: 'main'
    position: { x1: 20%, y1: 0%, x2: 100%, y2: 100% }
```

### Grid Layout

```yaml
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

## Tips for Success

1. **Start Simple**: Begin with basic layouts and add complexity gradually
2. **Test Frequently**: Run your interface after each change to catch issues early
3. **Use Proper Positioning**: Ensure panels don't overlap unintentionally
4. **Optimize Scripts**: Long-running scripts can freeze the interface
5. **Use Meaningful IDs**: Choose descriptive names for panels and layouts
6. **Consider Screen Sizes**: Test on different terminal sizes
7. **Debug with Logs**: Check the application logs for errors

## Troubleshooting

### Common Issues

**YAML Syntax Errors**

```bash
# Validate YAML syntax
yamllint your-file.yaml
```

**Script Execution Problems**

- Check script permissions
- Test scripts independently
- Use absolute paths for commands

**Layout Issues**

- Verify position percentages add up correctly
- Check for overlapping panels
- Ensure all required fields are present

**Performance Problems**

- Reduce refresh intervals
- Optimize script execution
- Limit output size

## Next Steps

Now that you understand the basics:

1. **Read the [Configuration Reference](configuration.md)** for YAML documentation
2. **Explore [Examples](examples.md)** for real-world use cases
3. **Check the [API Reference](api.md)** for socket messaging
4. **Learn about [Layouts & Positioning](layouts.md)** for layouts

## Need Help?

- Check the [Troubleshooting Guide](troubleshooting.md)
- Review the [FAQ](faq.md)
- Open an issue on GitHub
