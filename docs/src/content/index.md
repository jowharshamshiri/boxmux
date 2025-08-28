---
title: BoxMux - YAML-driven Terminal UI Framework
description: Terminal applications and dashboards with data visualization, plugin system, and socket API. Build rich, interactive CLI applications through simple YAML configuration files.
---


## Quick Installation

Get started with BoxMux in minutes:

```bash
# Install BoxMux
cargo install boxmux
```

```bash
# Try example configs
boxmux examples/dashboard.yaml
```

## Core Features

BoxMux provides everything you need to build rich terminal interfaces:

### Core Framework
YAML configuration system with multi-layout support, box hierarchy, and real-time rendering.

### UI Components  
Flexible box positioning, 16 ANSI colors, borders, text rendering, interactive menus, and focus management.

### Scripting & Automation
Multi-threaded script execution, PTY support, and output redirection.

### Socket API
Unix socket server for remote control - update boxes, switch layouts, manage refresh cycles.

### Data Visualization
Unicode charts (bar/line/histogram), layout engine, table boxes with CSV/JSON parsing, sorting, filtering.

### Variable System
Hierarchical variable substitution with precedence: environment > child > parent > layout > app > default.

### Plugin System
Dynamic component loading with security validation, manifest parsing, and access control.

### Enhanced Features
Mouse clicks, hot keys (F1-F24), clipboard integration (Ctrl+C), scrolling, proportional scrollbars.

### Performance & Quality
527/528 tests passing, performance benchmarking, cross-platform compatibility (macOS/Linux).

## Getting Started

Ready to build your first terminal application? Check out our comprehensive guides:

- **[User Guide](/docs/user-guide)** - Installation and basic configuration
- **[Configuration Reference](/docs/configuration)** - YAML configuration options and examples  
- **[API Reference](/docs/api)** - Socket API documentation and function reference

## Advanced Topics

Explore powerful features for complex applications:

- **[Data Visualization](/docs/data-visualization)** - Charts, tables, and data display features
- **[Plugin System](/docs/plugin-system)** - Dynamic component loading and plugin development
- **[PTY Features](/docs/pty-features)** - Interactive terminal emulation and process management
- **[Advanced Features](/docs/advanced-features)** - Mouse support, hot keys, streaming output, and enhanced navigation
- **[Variables](/docs/variables)** - Variable system and template substitution

## Resources

- **[Troubleshooting](/docs/troubleshooting)** - Common issues and solutions
- **[Roadmap](/docs/roadmap)** - Planned features and development timeline
- **[GitHub Repository](https://github.com/jowharshamshiri/boxmux)** - Source code and issues