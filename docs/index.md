# BoxMux Documentation

**YAML-driven terminal UI framework for creating rich, interactive CLI applications and dashboards.**

## Quick Start Guide

### Installation & First Interface
```bash
# Clone and build
git clone https://github.com/jowharshamshiri/boxmux.git && cd boxmux
cargo build --release && chmod +x run_boxmux.sh

# Run example
./run_boxmux.sh layouts/dashboard.yaml
```

### Essential Commands
```bash
cargo build --release    # Production build
cargo test               # Run test suite  
./run_boxmux.sh <config> # Run with configuration
```

### Basic Configuration Template
```yaml
app:
  layouts:
    - id: 'main'
      root: true
      title: 'My Interface'
      children:
        - id: 'panel1'
          title: 'Panel Title'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'Hello, BoxMux!'
```

## Core Documentation

### 📚 Essential References
- **[Configuration Guide](configuration.md)** - Complete YAML configuration reference with all properties
- **[User Guide](user-guide.md)** - Comprehensive tutorials and examples for all use cases  
- **[API Reference](api.md)** - Socket messaging and programmatic control documentation
- **[Troubleshooting](troubleshooting.md)** - Common issues, debugging, and solutions

### 🎯 Development Resources
- **[Contributing](../CONTRIBUTING.md)** - Development setup and contribution guidelines
- **[Roadmap](roadmap.md)** - Current status and planned features
- **[Architecture](../internal/architectural_record.md)** - Technical architecture decisions

## Socket API Quick Reference

```bash
# Update panel content
echo '{"UpdatePanel": {"panel_id": "status", "content": "Online"}}' | nc -U /tmp/boxmux.sock

# Refresh panel
echo '{"RefreshPanel": {"panel_id": "monitor"}}' | nc -U /tmp/boxmux.sock
```

## Key Features

### Core Capabilities
- **YAML Configuration** - Declarative interface design through simple configuration files
- **Real-time Updates** - Live data streams with configurable refresh intervals
- **Interactive Elements** - Keyboard navigation, menus, and user input handling
- **Socket API** - External control and integration via Unix sockets
- **Cross-platform** - Native support for macOS, Linux, and Unix-like systems

### Use Cases
- **System Administration** - Server monitoring, log analysis, deployment dashboards
- **Development Tools** - Build status, code review, testing interfaces
- **DevOps** - Infrastructure monitoring, container management, CI/CD pipelines
- **Data Visualization** - Metrics dashboards, real-time analytics

## Learning Path

### New Users
1. **[User Guide](user-guide.md)** - Complete tutorials and examples
2. **[Configuration Guide](configuration.md)** - Master YAML configuration
3. **[Troubleshooting](troubleshooting.md)** - Common issues and solutions

### Developers  
1. **[API Reference](api.md)** - Socket programming and external control
2. **[Contributing](../CONTRIBUTING.md)** - Development setup and guidelines
3. **[Architecture](../internal/architectural_record.md)** - Technical design decisions

## Project Information

- **Current Version**: 0.76.71205 (Production Ready)
- **License**: MIT License - See [LICENSE](../LICENSE)
- **Repository**: [GitHub](https://github.com/jowharshamshiri/boxmux)
- **Minimum Requirements**: Rust 1.70.0, Unix-like system

## Support and Contributing

- **Issues**: [GitHub Issues](https://github.com/jowharshamshiri/boxmux/issues) for bugs and feature requests
- **Contributing**: See [CONTRIBUTING.md](../CONTRIBUTING.md) for development guidelines  
- **Discussions**: [GitHub Discussions](https://github.com/jowharshamshiri/boxmux/discussions) for questions and ideas

---
*Documentation consolidated for streamlined access - All original docs preserved in archive/*
