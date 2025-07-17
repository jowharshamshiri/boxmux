# BoxMux Documentation

Welcome to the docs for BoxMux, the YAML-driven terminal UI framework.

## Quick Navigation

### 🚀 Getting Started

- **[Installation & Setup](getting-started.md)** - Get BoxMux up and running
- **[Your First Interface](getting-started.md#your-first-interface)** - Create your first BoxMux interface
- **[Core Concepts](getting-started.md#core-concepts)** - Understanding layouts, panels, and positioning

### 📚 Documentation

- **[Configuration Reference](configuration.md)** - YAML configuration guide
- **[API Reference](api.md)** - Socket messaging and programmatic control
- **[Examples](examples.md)** - Real-world examples and tutorials
- **[Troubleshooting](troubleshooting.md)** - Common issues and solutions

### 🛠️ Development

- **[Contributing Guidelines](../CONTRIBUTING.md)** - How to contribute to BoxMux
- **[Roadmap](roadmap.md)** - Future plans and features
- **[Architecture](architecture.md)** - Technical architecture overview

## Documentation Structure

### Core Documentation

1. **[Getting Started](getting-started.md)** - New user guide
2. **[Configuration Reference](configuration.md)** - YAML reference
3. **[API Reference](api.md)** - Socket API documentation
4. **[Examples](examples.md)** - Practical examples and tutorials

### Topics

5. **[Architecture](architecture.md)** - System architecture and design
6. **[Performance](performance.md)** - Optimization and best practices
7. **[Security](security.md)** - Security considerations and guidelines
8. **[Deployment](deployment.md)** - Production deployment guide

### Community & Development

9. **[Contributing](../CONTRIBUTING.md)** - Contribution guidelines
10. **[Roadmap](roadmap.md)** - Future development plans
11. **[Changelog](../CHANGELOG.md)** - Version history and changes
12. **[FAQ](faq.md)** - Frequently asked questions

## Quick Reference

### Essential Commands

```bash
# Build BoxMux
cargo build --release

# Run with configuration
./run_boxmux.sh layouts/dashboard.yaml

# Run tests
cargo test

# Format code
cargo fmt
```

### Basic Configuration

```yaml
app:
  layouts:
    - id: 'main'
      root: true
      title: 'My Interface'
      children:
        - id: 'panel1'
          title: 'Panel Title'
          position:
            x1: 10%
            y1: 10%
            x2: 90%
            y2: 90%
          content: 'Hello, BoxMux!'
```

### Socket API

```bash
# Update panel content
echo '{"UpdatePanel": {"panel_id": "status", "content": "Online"}}' | nc -U /tmp/boxmux.sock

# Refresh panel
echo '{"RefreshPanel": {"panel_id": "monitor"}}' | nc -U /tmp/boxmux.sock
```

## Learning Path

### For New Users

1. **Start with [Getting Started](getting-started.md)** - Learn the basics
2. **Follow the tutorials** - Build your first interface
3. **Explore [Examples](examples.md)** - See real-world use cases
4. **Read [Configuration Reference](configuration.md)** - Master YAML configuration

### For Developers

1. **Review [Architecture](architecture.md)** - Understand the system
2. **Read [API Reference](api.md)** - Learn socket programming
3. **Check [Contributing](../CONTRIBUTING.md)** - Join the development
4. **Follow [Roadmap](roadmap.md)** - See future plans

### For Users

1. **Study [Performance](performance.md)** - Optimize your interfaces
2. **Learn [Security](security.md)** - Secure your deployments
3. **Master [Deployment](deployment.md)** - Production best practices
4. **Explore plugins and extensions** - Extend functionality

## Feature Overview

### Core Features

- **YAML Configuration** - Declarative interface design
- **Real-time Updates** - Live data and automatic refresh
- **Interactive Elements** - Menus, navigation, and user input
- **Socket API** - External control and integration
- **Cross-platform** - Works on macOS, Linux, and Windows

### Interface Components

- **Layouts** - Overall interface structure
- **Panels** - Individual interface elements
- **Menus** - Interactive choice selection
- **Charts** - Data visualization
- **Logs** - Real-time log monitoring

### Features

- **Scripting** - Shell script integration
- **Themes** - Visual customization
- **Plugins** - Extended functionality
- **Networking** - Remote data sources
- **Performance** - Optimized rendering

## Use Cases

### System Administration

- Server monitoring dashboards
- Log analysis interfaces
- System health checks
- Deployment pipelines

### Development Tools

- Build status monitoring
- Code review dashboards
- Testing interfaces
- Performance monitoring

### Data Visualization

- Metrics dashboards
- Real-time analytics
- Business intelligence
- Scientific computing

### DevOps

- Infrastructure monitoring
- Container management
- CI/CD pipelines
- Alert management

## Community Resources

### Support Channels

- **[GitHub Issues](https://github.com/jowharshamshiri/boxmux/issues)** - Bug reports and feature requests
- **[GitHub Discussions](https://github.com/jowharshamshiri/boxmux/discussions)** - Community help and ideas
- **[Discord](https://discord.gg/boxmux)** - Real-time chat and support
- **[Email](mailto:support@boxmux.org)** - Direct support

### Contributing

- **[Contribution Guide](../CONTRIBUTING.md)** - How to contribute
- **[Code of Conduct](../CODE_OF_CONDUCT.md)** - Community guidelines
- **[Development Setup](../CONTRIBUTING.md#development-setup)** - Get started developing
- **[Issue Templates](https://github.com/jowharshamshiri/boxmux/issues/new/choose)** - Report bugs or request features

### External Resources

- **[Awesome BoxMux](https://github.com/topics/boxmux)** - Community projects
- **[Stack Overflow](https://stackoverflow.com/questions/tagged/boxmux)** - Q&A
- **[Reddit](https://reddit.com/r/boxmux)** - Community discussions
- **[YouTube](https://youtube.com/boxmux)** - Video tutorials

## License and Legal

BoxMux is released under the **MIT License**. See the [LICENSE](../LICENSE) file for details.

### Third-party Dependencies

- **[Rust](https://www.rust-lang.org/)** - Programming language
- **[Crossterm](https://github.com/crossterm-rs/crossterm)** - Terminal manipulation
- **[Serde](https://serde.rs/)** - Serialization framework
- **[YAML](https://yaml.org/)** - Configuration format

## Version Information

- **Current Version**: 0.76.71205
- **Minimum Rust Version**: 1.70.0
- **Supported Platforms**: macOS, Linux, Windows
- **Latest Release**: [GitHub Releases](https://github.com/jowharshamshiri/boxmux/releases)

## Feedback and Improvements

This documentation is continuously improved based on user feedback. If you find errors, missing information, or have suggestions for improvement:

1. **[Open an issue](https://github.com/jowharshamshiri/boxmux/issues/new)** on GitHub
2. **[Submit a pull request](https://github.com/jowharshamshiri/boxmux/pulls)** with improvements
3. **[Join the discussion](https://github.com/jowharshamshiri/boxmux/discussions)** about documentation
4. **[Contact us directly](mailto:docs@boxmux.org)** with feedback

---

**Welcome to BoxMux! I'm excited to see what you'll build.** 🚀
