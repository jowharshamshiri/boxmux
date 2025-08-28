---
title: Stream Architecture
description: BoxMux stream-based architecture - multiple input streams, tab system, stream switching, and lifecycle management for dynamic content display
---

## Table of Contents

- [Overview](#overview)
- [Stream Types](#stream-types)
- [Tab System](#tab-system)
- [Stream Switching](#stream-switching)
- [Close Buttons and Lifecycle](#close-buttons-and-lifecycle)
- [Practical Examples](#practical-examples)
- [Advanced Usage](#advanced-usage)

## Overview

BoxMux uses a **stream-based architecture** where every box can display content from multiple input streams. Each stream appears as a separate tab in the box title bar, allowing users to switch between different content sources within the same box.

This architecture enables:
- **Multiple content sources** per box (base content, choice outputs, PTY sessions)
- **Dynamic tab creation** based on active streams  
- **Interactive stream switching** by clicking tabs
- **Stream lifecycle management** with automatic cleanup
- **Real-time updates** from background streams

## Stream Types

### Content Stream
The default stream containing the box's base content and script output.

```yaml
- id: 'content_box'
  title: 'System Info'
  content: 'Base system information'
  script: ['uname -a']
  # → Creates "Content" tab
```

### Choices Stream  
Interactive menu options that appear as a separate stream.

```yaml
- id: 'menu_box'
  title: 'Actions Menu'
  choices:
    - id: 'deploy'
      script: ['./deploy.sh']
    - id: 'test'
      script: ['./run-tests.sh']
  # → Creates "Actions Menu" tab (uses box title)
```

### Redirected Output Streams
Output from choice executions in other boxes creates dedicated streams.

```yaml
- id: 'control_box'
  title: 'Controls'
  choices:
    - id: 'deploy'
      script: ['./deploy.sh']
      redirect_output: 'output_box'  # Creates "Deploy" tab in output_box

- id: 'output_box'
  title: 'Output Display'
  content: 'Waiting for operations...'
  # → Shows both "Content" and "Deploy" tabs when deploy runs
```

### PTY Streams
Interactive terminal sessions for running terminal programs.

```yaml
- id: 'terminal_box'  
  title: 'Terminal'
  pty: true
  script: ['bash']
  # → Creates "Terminal" tab with interactive shell
```

## Tab System

Tabs are automatically generated based on available streams in each box:

### Tab Priority and Ordering
1. **Content Stream** - Always first if present
2. **Choices Stream** - Second if present  
3. **Redirected Output Streams** - Ordered by creation time
4. **PTY Streams** - Ordered by creation time

### Tab Labels
- **Content Stream**: "Content"
- **Choices Stream**: Uses box title or "Choices" if no title
- **Redirected Output**: Uses choice ID (e.g., "Deploy", "Test") 
- **PTY Stream**: Uses box title or "PTY" if no title

### Visual Indicators
- **Active tab**: Highlighted with different color
- **Background activity**: Subtle indicator for streams with new content
- **Close buttons**: × symbol on closeable streams (redirected output, PTY)

## Stream Switching

Users can switch between streams by clicking tabs:

### Mouse Interaction
```
[Content] [Deploy] [Test]
    ↑       ↑       ↑
  Click to switch active stream
```

### Keyboard Navigation
- **Tab**: Cycle through boxes
- **Arrow keys**: Navigate within active stream content
- **Mouse click**: Switch streams within focused box

### Active Stream Behavior
- Only the active stream content is displayed in the box
- Scrolling and navigation apply to the active stream
- Background streams continue updating but aren't visible

## Close Buttons and Lifecycle

### Closeable Streams
Streams that can be terminated show close buttons (×):
- **Redirected Output Streams** - From choice executions
- **PTY Streams** - Interactive terminal sessions  
- **Choice Execution Streams** - Running processes
- **External Socket Streams** - Socket-controlled content

### Non-closeable Streams
Core streams remain open:
- **Content Streams** - Base box content
- **Choices Streams** - Menu definitions

### Stream Cleanup
When a stream is closed:
1. **Process termination** - Associated processes are killed
2. **Resource cleanup** - File descriptors and threads cleaned up
3. **Tab removal** - Tab disappears from box title bar
4. **Stream switching** - Active stream switches to remaining stream

## Practical Examples

### Multi-Stream Development Box
```yaml
- id: 'dev_control'
  title: 'Development'
  content: 'Ready for development tasks'
  choices:
    - id: 'build'
      script: ['cargo build --release']  
      redirect_output: 'build_output'
    - id: 'test'
      script: ['cargo test']
      redirect_output: 'test_output' 
    - id: 'lint'
      script: ['cargo clippy']
      redirect_output: 'lint_output'

- id: 'build_output'
  title: 'Build Results'
  content: 'Build output will appear here...'
  # Tabs: [Content] + [Build]/[Test]/[Lint] when commands run

- id: 'terminal'
  title: 'Terminal' 
  pty: true
  script: ['bash']
  # Tab: [Terminal] with interactive shell
```

**Result**: 
- `dev_control` box shows [Content] [Development] tabs
- `build_output` box starts with [Content], adds [Build]/[Test]/[Lint] tabs as commands run
- `terminal` box shows [Terminal] tab with interactive bash session
- Users can close Build/Test/Lint tabs with × button after completion

### Real-time Monitoring Setup
```yaml
- id: 'monitoring'
  title: 'System Monitor'
  content: |
    System monitoring dashboard
    Click choices to start monitoring streams
  choices:
    - id: 'cpu'
      script: ['top -l 0 -s 1']
      redirect_output: 'metrics'
      streaming: true
    - id: 'disk'  
      script: ['iostat 1']
      redirect_output: 'metrics'
      streaming: true
    - id: 'network'
      script: ['netstat -w 1'] 
      redirect_output: 'metrics'
      streaming: true

- id: 'metrics'
  title: 'Live Metrics'
  content: 'Select monitoring options from control panel'
  auto_scroll: true
  # Tabs dynamically created: [Content] [CPU] [Disk] [Network]
```

**Behavior**:
- Users click CPU/Disk/Network choices to start monitoring streams
- Each creates a dedicated tab in the `metrics` box
- Users can switch between different metric streams using tabs
- Close buttons (×) allow stopping individual monitoring streams
- `auto_scroll: true` keeps latest metrics visible

## Advanced Usage

### Stream Configuration Options

```yaml  
- id: 'advanced_box'
  title: 'Advanced Streams'
  content: 'Base content'
  
  # Content stream behavior
  auto_scroll: true          # Auto-scroll to bottom for new content
  
  # Choice stream with streaming output
  choices:
    - id: 'long_process'
      script: ['./long-running-task.sh']
      streaming: true          # Stream output line-by-line
      redirect_output: 'output'
      
  # PTY stream configuration  
  pty: true
  script: ['htop']            # Interactive process monitor
```

### Stream Monitoring and Control

Via Socket API:
```bash
# List all streams for a box
boxmux query-streams --box-id advanced_box

# Switch active stream  
boxmux switch-stream --box-id advanced_box --stream-id long_process

# Close specific stream
boxmux close-stream --box-id advanced_box --stream-id long_process

# Monitor stream status
boxmux stream-status --box-id advanced_box
```

### Error Handling

Streams handle failures gracefully:
- **Process crashes**: Stream shows error state, remains closeable
- **Script failures**: Error output captured in stream  
- **PTY failures**: Automatic fallback to regular execution
- **Socket disconnects**: Stream marked as disconnected but preserved

### Performance Considerations

- **Stream limits**: No hard limit on streams per box
- **Memory usage**: Each stream maintains its own buffer
- **Update frequency**: Background streams throttled to prevent UI lag
- **Cleanup timing**: Streams cleaned up immediately when closed

---

The stream architecture provides a powerful foundation for building complex, multi-source terminal interfaces while maintaining simple YAML configuration syntax.