---
title: BoxMux Visual Themes
description: Customizing colors, styling, and visual appearance in BoxMux terminal applications
---

# BoxMux Visual Themes

BoxMux renders with readable colors out of the box and also supports full theming through YAML configuration for color schemes and styling.

## Automatic Light/Dark Theme

You do not need to set any colors to get a good-looking dashboard. BoxMux picks a light or dark theme automatically and applies sensible defaults to every element — panel backgrounds, text, borders, titles, menu selections, and hover highlights — so a configuration with no color fields renders cleanly in both light and dark terminals.

- **Auto-detection**: The theme is chosen from the terminal (via `COLORFGBG`), defaulting to dark when it cannot be determined.
- **Force a theme**: Pass `--light` or `--dark` on the command line to override detection:

  ```bash
  boxmux layouts/dashboard.yaml --dark
  boxmux layouts/dashboard.yaml --light
  ```

- **Palette-safe highlights**: Selection, active-tab, and focus colors use fixed 256-color values rather than the base 16 ANSI colors. The base 16 colors are remapped by terminal themes (for example, "bright black" is rendered as a light tint by some palettes), which can make text on a highlight unreadable; the fixed values render consistently across terminals.
- **Focus indication**: The focused box is marked with a distinct border and tab color, so the active box is always identifiable without any configuration.
- **Backgrounds are real backgrounds**: Empty space inside a box is painted with the box's background color (not a foreground block glyph), so the area behind text and the empty area always match — including in selected boxes and tab bars.

Any color you set explicitly (see below) overrides the theme default for that element, so you can theme as much or as little as you like.

## ANSI Color System

BoxMux uses the standard 16-color ANSI palette for all visual elements:

### Standard Colors
```yaml
# Basic ANSI colors (8 colors)
standard_colors:
  - Black      # 0: Pure black
  - Red        # 1: Dark red  
  - Green      # 2: Dark green
  - Yellow     # 3: Dark yellow/brown
  - Blue       # 4: Dark blue
  - Magenta    # 5: Dark magenta/purple
  - Cyan       # 6: Dark cyan
  - White      # 7: Light gray
```

### Bright Colors
```yaml
# Bright ANSI colors (8 additional colors)
bright_colors:
  - BrightBlack    # 8: Dark gray
  - BrightRed      # 9: Bright red
  - BrightGreen    # 10: Bright green  
  - BrightYellow   # 11: Bright yellow
  - BrightBlue     # 12: Bright blue
  - BrightMagenta  # 13: Bright magenta
  - BrightCyan     # 14: Bright cyan
  - BrightWhite    # 15: Pure white
```

## Box Styling Properties

### Basic Box Appearance
```yaml
boxes:
  styled_box:
    # Border and outline styling
    border_color: "BrightBlue"         # Box border color
    title_color: "BrightYellow"        # Title text color
    
    # Content area styling  
    foreground_color: "BrightWhite"    # Text color
    background_color: "Black"          # Background fill
    
    # Fill characters for empty space
    fill_char: " "                     # Default fill character
    selected_fill_char: "░"            # Fill when box is selected/focused
```

### Focus and State Styling
```yaml
boxes:
  interactive_box:
    # Base styling
    border_color: "White"
    foreground_color: "White"
    
    # Focus indication (automatic when focusable: true)
    focusable: true
    
    # Selection state styling
    selected_fill_char: "▓"            # Highlighted fill pattern
```

## Theme Examples

### Professional Dark Theme
```yaml
# Professional dark theme with blue accents
theme:
  name: "Professional Dark"
  
boxes:
  header:
    border_color: "BrightCyan"
    title_color: "BrightWhite"
    foreground_color: "BrightWhite"
    background_color: "Black"
    
  content:
    border_color: "BrightBlue"
    foreground_color: "White"
    background_color: "Black"
    
  status:
    border_color: "BrightGreen"
    foreground_color: "BrightGreen"
    background_color: "Black"
    
  warning:
    border_color: "BrightYellow"
    foreground_color: "BrightYellow"
    background_color: "Black"
    
  error:
    border_color: "BrightRed"
    foreground_color: "BrightRed"
    background_color: "Black"
```

### Retro Terminal Theme
```yaml
# Retro green-on-black terminal theme
theme:
  name: "Retro Terminal"
  
boxes:
  main:
    border_color: "BrightGreen"
    title_color: "BrightGreen"
    foreground_color: "Green"
    background_color: "Black"
    fill_char: " "
    selected_fill_char: "░"
    
  secondary:
    border_color: "Green"
    title_color: "BrightGreen"
    foreground_color: "Green"
    background_color: "Black"
```

### High Contrast Theme
```yaml
# High contrast theme for accessibility
theme:
  name: "High Contrast"
  
boxes:
  primary:
    border_color: "BrightWhite"
    title_color: "BrightWhite"
    foreground_color: "BrightWhite"
    background_color: "Black"
    
  secondary:
    border_color: "BrightYellow"
    title_color: "BrightYellow"
    foreground_color: "BrightYellow"
    background_color: "Black"
    
  accent:
    border_color: "BrightCyan"
    foreground_color: "BrightCyan"
    background_color: "Black"
```

## PTY Visual Indicators

PTY-enabled boxes have special visual styling to distinguish them:

```yaml
boxes:
  terminal_box:
    pty: true
    
    # PTY visual indicators (automatic)
    title: "⚡ Interactive Terminal"      # Lightning bolt prefix
    border_color: "BrightCyan"           # Distinctive cyan border
    
    # PTY status color coding (automatic based on state)
    # - Running: BrightCyan
    # - Error: BrightRed  
    # - Finished: BrightBlack
    # - Dead: Red
```

## State-Based Color Coding

### Process States
```yaml
boxes:
  process_monitor:
    # Colors change automatically based on process state
    script: "some_long_running_process.sh"
    
    # State colors (automatic):
    # - Running: Normal colors
    # - Error: BrightRed border/text
    # - Success: BrightGreen accents
    # - Waiting: BrightYellow indicators
```

### Content-Based Styling
```yaml
boxes:
  log_viewer:
    script: |
      #!/bin/bash
      # Script output colors are preserved
      echo -e "\\033[32mSUCCESS: Operation completed\\033[0m"
      echo -e "\\033[33mWARNING: Check configuration\\033[0m" 
      echo -e "\\033[31mERROR: Connection failed\\033[0m"
      
    # ANSI escape sequences in output are processed
    # Colors appear correctly in the terminal
```

## Layout-Based Theming

### Multi-Layout Color Coordination
```yaml
# Consistent theming across layouts
variables:
  primary_color: "BrightBlue"
  accent_color: "BrightCyan"
  success_color: "BrightGreen"
  warning_color: "BrightYellow"
  error_color: "BrightRed"

layouts:
  dashboard:
    type: "MuxBox"
    children:
      header:
        border_color: "{{primary_color}}"
        title_color: "{{accent_color}}"
        
      status:
        border_color: "{{success_color}}"
        
  monitoring:
    type: "MuxBox"
    children:
      alerts:
        border_color: "{{warning_color}}"
        
      errors:
        border_color: "{{error_color}}"
```

## Interactive Element Styling

### Menu and Choice Styling
```yaml
boxes:
  main_menu:
    choices:
      - name: "Deploy Application"
        # Choice styling is inherited from parent box
        
    # Menu-specific colors
    choice_color: "BrightGreen"          # Unselected choices
    selected_color: "BrightYellow"       # Currently selected choice
    border_color: "BrightBlue"           # Menu border
```

### Scrollbar and Interactive Elements
```yaml
boxes:
  scrollable_content:
    overflow_behavior: "scroll"
    
    # Scrollbar styling (automatic based on box colors)
    border_color: "BrightWhite"          # Scrollbar uses box border color
    # Scrollbar components:
    # - Track: Uses fill_char
    # - Thumb: Uses selected_fill_char
    # - Arrows: Use border_color
```

## Theming Techniques

### Conditional Color Schemes
```yaml
# Environment-based theming
variables:
  env: "{{ENV}}"  # production, staging, development
  
boxes:
  status_indicator:
    title: "Environment: {{env}}"
    # Use different colors based on environment
    border_color: |
      {% if env == "production" %}BrightRed{% elif env == "staging" %}BrightYellow{% else %}BrightGreen{% endif %}
```

### Time-Based Themes
```yaml
boxes:
  time_sensitive:
    script: |
      #!/bin/bash
      hour=$(date +%H)
      if [ $hour -lt 6 ] || [ $hour -gt 18 ]; then
        echo "🌙 Night mode active"
      else
        echo "☀️ Day mode active"  
      fi
    
    # Adjust colors based on script output or time
    border_color: "BrightBlue"  # Could be dynamic based on conditions
```

## Z-Index and Layering

Control visual stacking order for overlapping boxes:

```yaml
boxes:
  background_box:
    z_index: 1                           # Behind other boxes
    border_color: "BrightBlack"
    
  main_content:
    z_index: 5                           # Normal layer
    border_color: "BrightBlue"
    
  popup_overlay:
    z_index: 10                          # Above other boxes
    border_color: "BrightYellow"
```

## Theme Best Practices

### Color Harmony
```yaml
# Use related colors for cohesive appearance
theme_palette:
  primary: "BrightBlue"        # Main accent color
  secondary: "BrightCyan"      # Related cool color
  neutral: "White"             # Text and borders
  background: "Black"          # Consistent background
  success: "BrightGreen"       # Status indication
  warning: "BrightYellow"      # Attention grabbing
  danger: "BrightRed"          # Error states
```

### Accessibility Considerations
- Ensure sufficient contrast between text and background
- Use bright colors for important information
- Provide non-color indicators (symbols, text) for critical states
- Test themes in different terminal environments
- Consider colorblind-friendly color combinations

### Performance Optimization
- Use consistent colors across related boxes
- Minimize frequent color changes
- Choose colors that work well in various terminal emulators
- Test theme performance with different terminal color capabilities

BoxMux uses ANSI colors for terminal application styling.