---
title: Interactive UI Features
description: Mouse-driven interface manipulation - box resizing, dragging, cursor styles, scrolling interactions, and real-time YAML persistence for dynamic layouts
---

## Table of Contents

- [Overview](#overview)
- [Box Resizing](#box-resizing)
- [Box Movement](#box-movement)
- [Scrolling Interactions](#scrolling-interactions)
- [Dynamic Cursor Styles](#dynamic-cursor-styles)
- [YAML Persistence](#yaml-persistence)
- [Performance Optimizations](#performance-optimizations)
- [Practical Examples](#practical-examples)

## Overview

BoxMux provides comprehensive mouse-driven interface manipulation, allowing users to modify layouts interactively without editing YAML files. All changes are automatically persisted to the original configuration files.

**Interactive Features:**
- **Box resizing** by dragging corners
- **Box movement** by dragging title bars
- **Scrolling** with sensitive scrollbars and drag-to-scroll
- **Dynamic cursor styles** that change based on interaction zones
- **Real-time YAML synchronization** for persistent changes
- **Performance optimizations** for smooth interactions

## Box Resizing

### Corner Dragging
Resize boxes by clicking and dragging the **bottom-right corner**:

```yaml
- id: 'resizable_box'
  title: 'Drag Corner to Resize'
  position: {x1: 10%, y1: 10%, x2: 60%, y2: 60%}
  content: 'Drag the bottom-right corner to resize this box'
```

**Interaction:**
1. **Hover** over bottom-right corner → cursor changes to resize indicator (■)
2. **Click and drag** → box resizes in real-time with visual feedback  
3. **Release** → new dimensions automatically saved to YAML file

### Size Constraints
- **Minimum size**: 2x2 characters to ensure usability
- **Terminal boundaries**: Cannot resize beyond terminal dimensions
- **Proportional scaling**: Width and height adjust based on drag direction

### Z-Index Layering
Boxes with higher `z_index` values appear on top and receive resize interactions first:

```yaml
- id: 'background_box'
  z_index: 1
  position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
  
- id: 'foreground_box'  
  z_index: 2
  position: {x1: 20%, y1: 20%, x2: 80%, y2: 80%}
  # This box can be resized even when overlapping background_box
```

## Box Movement

### Title Bar Dragging
Move boxes by clicking and dragging the **title bar** or **top border**:

```yaml
- id: 'movable_box'
  title: 'Drag Title to Move'
  position: {x1: 25%, y1: 25%, x2: 75%, y2: 75%}
  content: 'Drag the title bar to move this box around'
```

**Interaction:**
1. **Hover** over title bar → cursor changes to move indicator (_)
2. **Click and drag** → box follows mouse with real-time preview
3. **Release** → new position automatically saved to YAML file

### Movement Constraints  
- **Boundary limits**: Box cannot be moved outside terminal bounds
- **Minimum visibility**: At least title bar must remain visible
- **Grid snapping**: Optional snap-to-grid for precise alignment

## Scrolling Interactions

### Sensitive Scrollbars
Click scrollbar areas to jump to specific positions:

```yaml
- id: 'scrollable_content'
  title: 'Scrollable Box'
  content: |
    Line 1 of many lines
    Line 2 of many lines
    ... many more lines ...
    Line 100 of many lines
  overflow_behavior: 'scroll'
```

**Scrollbar Interactions:**
- **Click scrollbar track** → jump to clicked position
- **Click above thumb** → page up
- **Click below thumb** → page down

### Drag-to-Scroll
Click and drag the scrollbar thumb for smooth scrolling:

**Thumb Dragging:**
1. **Click scrollbar thumb** → cursor changes, thumb highlights
2. **Drag vertically** → content scrolls proportionally
3. **Release** → scroll position maintained

### Mouse Wheel Support
- **Wheel up/down** → scroll content vertically
- **Shift + wheel** → scroll content horizontally (where applicable)
- **Works in any focusable scrollable box**

## Dynamic Cursor Styles

The cursor automatically changes based on the mouse position to indicate available interactions:

### Cursor Types

| Cursor Style | Interaction Zone | Purpose |
|--------------|------------------|---------|
| **■ (BlinkingBlock)** | Bottom-right corner | Box resizing |  
| **_ (BlinkingUnderScore)** | Title bar/top border | Box movement |
| **\| (BlinkingBar)** | Sensitive choices/buttons | Interactive elements |
| **→ (DefaultUserShape)** | Default areas | No special interaction |

### Edge Case Handling
- **100% width panels**: Enhanced detection for resize corners at terminal edge
- **Overlapping boxes**: Cursor reflects topmost (highest z-index) box interaction
- **Small boxes**: Cursor zones adapt to available space

## YAML Persistence

### Automatic Synchronization
All interactive changes are immediately saved to the original YAML configuration:

```yaml
# Original configuration
- id: 'demo_box' 
  position: {x1: 10%, y1: 10%, x2: 50%, y2: 50%}

# After user resizes box → automatically becomes:
- id: 'demo_box'
  position: {x1: 10%, y1: 10%, x2: 75%, y2: 65%}
```

### Preserved Formatting
- **YAML structure**: Original file formatting and comments preserved
- **Field order**: Existing field ordering maintained
- **Indentation**: Consistent with original file style

### Live Synchronization Features
- **Bounds changes**: Position and size updates
- **Active layout**: Layout switching persisted
- **Scroll positions**: Scroll state maintained across restarts
- **Box properties**: Dynamic property changes saved

### Atomic Operations
- **File locking**: Prevents corruption during concurrent access
- **Backup creation**: Temporary backup before writing changes
- **Validation**: Configuration validated before saving
- **Rollback**: Automatic rollback on validation failures

## Performance Optimizations

### Smooth Dragging
BoxMux implements dual-layer optimization for lag-free interactions:

**Message Coalescing:**
- Multiple rapid mouse movements consolidated into single updates
- Intermediate drag events skipped when queue is full
- Reduces CPU load during high-frequency mouse input

**60 FPS Throttling:**
- Screen redraws limited to 16ms intervals during drag operations
- Provides smooth visual feedback without excessive rendering
- Automatically disables throttling when interaction ends

### Memory Management
- **Efficient redraws**: Only affected screen regions updated
- **Background processing**: Non-critical updates deferred during interactions
- **Resource cleanup**: Temporary UI state cleaned up after interactions

## Practical Examples

### Dashboard Layout Editor
```yaml
app:
  title: 'Interactive Dashboard'
  layouts:
    - id: 'main'
      active: true
      boxes:
        # Header bar - movable and resizable
        - id: 'header'
          title: 'System Overview'
          z_index: 2
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 15%}
          content: 'Drag title to move, drag corner to resize'
          
        # Left sidebar - can be resized horizontally  
        - id: 'sidebar'
          title: 'Controls'
          z_index: 1
          position: {x1: 0%, y1: 15%, x2: 25%, y2: 100%}
          choices:
            - id: 'monitor_cpu'
              script: ['top -l 0 -s 1']
              redirect_output: 'main_display'
              
        # Main display area - resizable content area
        - id: 'main_display'
          title: 'Output'
          z_index: 1
          position: {x1: 25%, y1: 15%, x2: 100%, y2: 100%}
          content: 'Main content area - fully interactive'
          overflow_behavior: 'scroll'
          auto_scroll: true
```

**Interactive Features:**
- **Resize sidebar**: Drag right edge to adjust sidebar width
- **Move header**: Drag title bar to reposition header
- **Scroll main area**: Use scrollbars or mouse wheel in main display
- **Auto-save**: All layout changes saved to `dashboard.yaml`

### Development Environment
```yaml
- id: 'editor_area'
  title: 'Editor Simulation'
  z_index: 3
  position: {x1: 5%, y1: 5%, x2: 70%, y2: 85%}  
  content: 'Main editor area (drag to move/resize)'
  
- id: 'file_tree'
  title: 'Files'
  z_index: 2  
  position: {x1: 70%, y1: 5%, x2: 95%, y2: 50%}
  choices:
    - id: 'src'
      script: ['find src -name "*.rs"']
      redirect_output: 'editor_area'
      
- id: 'terminal'
  title: 'Terminal'
  z_index: 2
  position: {x1: 70%, y1: 50%, x2: 95%, y2: 85%}
  pty: true
  script: ['bash']
  
- id: 'status'
  title: 'Status'
  z_index: 1
  position: {x1: 5%, y1: 85%, x2: 95%, y2: 95%}
  content: 'Ready - layout changes auto-saved'
```

**Workflow:**
1. **Adjust layout**: Resize editor, file tree, and terminal panes by dragging corners
2. **Reposition panels**: Move panels by dragging title bars  
3. **Layer management**: Higher z_index panels can be resized over lower ones
4. **Persistent setup**: Layout changes saved to config file for next session

### Multi-Monitor Simulation
```yaml
# Simulate multiple monitor regions with interactive boundaries
- id: 'monitor1'
  title: 'Monitor 1 (Primary)'
  z_index: 1
  position: {x1: 0%, y1: 0%, x2: 60%, y2: 100%}
  content: 'Primary display - drag border to adjust split'
  
- id: 'monitor2' 
  title: 'Monitor 2 (Secondary)'
  z_index: 1
  position: {x1: 60%, y1: 0%, x2: 100%, y2: 100%}
  content: 'Secondary display'
  
- id: 'floating_window'
  title: 'Floating Window'
  z_index: 5
  position: {x1: 30%, y1: 20%, x2: 80%, y2: 60%}
  content: 'Movable window that can float over both monitors'
```

**Usage:**
- **Adjust monitor split**: Resize monitor1 to change the split point
- **Float windows**: Move floating_window between "monitor" regions
- **Z-index layering**: Floating window appears over monitor backgrounds

---

The interactive UI system makes BoxMux layouts fully dynamic while maintaining the simplicity of YAML configuration through automatic persistence.