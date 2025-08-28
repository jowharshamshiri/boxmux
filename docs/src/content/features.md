---
title: Features
description: Overview of BoxMux capabilities for building terminal user interfaces
---


## Core Framework Features

### YAML Configuration System
- **Interface Design**: Define terminal layouts using YAML configuration files
- **Nested Box Hierarchy**: Create layouts with parent-child box relationships and automatic bounds calculation
- **Multi-Layout Support**: Switch between different layouts dynamically with root/active layout management
- **Schema Validation**: Built-in JSON schema validation ensures configuration correctness
- **Live Configuration Updates**: Modify YAML files and see changes reflected in real-time

### Interactive Box Components
- **Flexible Positioning**: Percentage-based and absolute positioning with anchor system support (TopLeft, Center, etc.)
- **Dynamic Resizing**: Interactive box resizing with mouse drag support and automatic YAML persistence
- **Border Styling**: Customizable borders with 16 ANSI colors and bright variants
- **Focus Management**: Tab order navigation with configurable focus chains
- **Overflow Handling**: Multiple overflow behaviors including scroll, wrap, fill, and cross_out modes

## Script Execution & Automation

### Real-time Script Execution
- **Background Threading**: Execute shell scripts in dedicated threads with configurable refresh intervals
- **Live Output Streaming**: Stream script output incrementally as it's generated
- **Output Redirection**: Route script output between different boxes with `redirect_output` and `append_output`
- **Error Handling**: Graceful handling of script failures with visual error state indication
- **Library Support**: Include external script libraries with the `libs` field

### Execution Features
- **Choice-based Actions**: Execute scripts from interactive menu selections
- **Hot Key Actions**: Global keyboard shortcuts to trigger actions
- **Conditional Execution**: Script execution based on system state and environment variables
- **Multi-threaded Processing**: Concurrent script execution across multiple boxes

## PTY (Pseudo-Terminal) Integration

### Interactive Terminal Support
- **PTY Integration**: Run interactive programs like vim, htop, SSH, and bash sessions
- **ANSI Processing**: ANSI escape sequence handling with color support and cursor management
- **Input Routing**: Direct keyboard input to focused PTY processes with special key support
- **Process Management**: Process lifecycle control including spawn, kill, and restart operations

### PTY Features
- **Scrollback Buffer**: Circular buffer storage for PTY output with configurable capacity (default 10,000 lines)
- **Error Recovery**: Fallback system that avoids problematic PTY usage after failures
- **Visual Indicators**: Lightning bolt (⚡) prefix and color-coded borders for PTY-enabled boxes
- **Process Information**: Display of running process details in box titles

## Socket API & Remote Control

### Unix Socket Server
- **Remote Control Interface**: Control BoxMux applications via Unix socket commands
- **CLI**: Command-line interface for socket operations
- **Real-time Updates**: Update box content, scripts, and configurations remotely
- **Layout Management**: Switch layouts and manage box hierarchies via socket commands

### Socket API Operations
- **Content Management**: `replace-box-content`, `replace-box-script` commands
- **Box Management**: `add-box`, `remove-box`, `replace-box` operations  
- **PTY Control**: `spawn-pty`, `kill-pty`, `restart-pty`, `send-pty-input` commands
- **Process Monitoring**: `query-pty-status` for detailed process information
- **Refresh Control**: `start-box-refresh`, `stop-box-refresh` commands
- **Stream Control**: `close-stream`, `switch-stream`, `list-streams` commands

## User Interface & Interaction

### Mouse Support
- **Click Interactions**: Click to select boxes, activate menu items, and trigger actions
- **Scrollbar Controls**: Interactive scrollbars with click-to-jump and drag-to-scroll support
- **Border Dragging**: Resize boxes interactively by dragging borders with real-time feedback
- **Dynamic Cursor Styles**: Context-sensitive cursor shapes for different interactive elements
- **Performance Optimized**: Message coalescing and 60 FPS throttling for smooth drag operations

### Keyboard Navigation
- **Key Support**: Arrow keys, page up/down, home/end with modifier key combinations
- **Focus Chain Navigation**: Tab/Shift+Tab navigation through focusable elements
- **Special Key Handling**: F1-F24, Ctrl/Alt/Shift combinations for PTY applications
- **Global Shortcuts**: Configurable application-level key bindings
- **Platform-specific Support**: Key handling for macOS (Cmd) and Linux/Windows (Ctrl)

## Data Visualization & Content

### Content Types
- **Table System**: CSV/JSON data parsing with sorting, filtering, and pagination support
- **Chart Library**: Unicode-based charts including bar, line, and histogram visualizations
- **Plugin System**: Dynamic loading of custom components with security validation
- **Text Processing**: Text wrapping with word boundary preservation
- **Content Streaming**: Auto-scroll to bottom for log files and streaming output

### Variable & Template System
- **Hierarchical Variables**: Environment, application, layout, and box-level variable support
- **Template Substitution**: Dynamic content generation using variable substitution
- **Environment Integration**: Seamless integration with system environment variables
- **Context-aware Resolution**: Intelligent variable precedence resolution

## Testing & Quality Assurance

### Test Coverage
- **Unit Tests**: 437+ tests covering framework components with 99.8% success rate
- **Integration Tests**: End-to-end application testing with real-world scenarios
- **PTY Tests**: Testing for pseudo-terminal functionality
- **Socket API Tests**: Validation of remote control operations
- **Performance Tests**: Benchmarking and optimization validation

### Development Features
- **Hot Reload**: Live reloading of configuration changes during development
- **Debug Logging**: Logging system with configurable verbosity levels
- **Error Reporting**: Error context with stack traces and execution breadcrumbs
- **Performance Monitoring**: Built-in metrics collection and analysis tools

## Architecture & Performance

### Multi-threaded Architecture
- **Unified Threading System**: Single ThreadManager coordinating all background operations
- **Message Passing**: Efficient inter-thread communication using structured message system
- **Resource Management**: Proper cleanup and resource leak prevention
- **Thread Safety**: Thread-safe implementation with Arc&lt;Mutex&lt;&gt;&gt; patterns

### Performance Optimizations
- **Efficient Rendering**: Diff-based screen updates minimizing terminal I/O overhead
- **Memory Management**: Circular buffers and bounded storage preventing memory leaks
- **Input Optimization**: Message coalescing and event throttling for responsive interactions
- **Cross-platform Support**: Optimized implementations for macOS, Linux, and Windows

## Configuration & Customization

### YAML Schema Features
- **Type Safety**: JSON schema validation for configuration elements  
- **Nested Structures**: Support for nested box hierarchies and relationships
- **Dynamic Updates**: Live synchronization of configuration changes to YAML files
- **Validation Feedback**: Error messages for configuration issues
- **Documentation Integration**: Schema-driven documentation generation

### Extensibility
- **Plugin Architecture**: Dynamic component loading with fallback to mock implementations
- **Custom Components**: Create specialized box types for specific use cases
- **Theme System**: Customizable color schemes and visual styling
- **Layout Templates**: Reusable layout patterns and component libraries

BoxMux uses YAML configuration to define terminal UI applications with interactive components and real-time updates.