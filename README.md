# BoxMux

**Automate tasks and put automation into terminal dashboards with minimal effort. Use YAML to transform Unix commands into interactive programs running in organized, threaded interfaces.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Version](https://img.shields.io/badge/version-0.76.71205-blue.svg)](https://github.com/jowharshamshiri/boxmux)

![BoxMux Dashboard](./docs/screenshot.png)
![BoxMux Dashboard](./docs/code.png)
![BoxMux Dashboard](./docs/screenshot_2.png)
![BoxMux Dashboard](./docs/socket_control.png)

## What is BoxMux?

BoxMux lets you automate tasks and immediately visualize that automation in terminal interfaces. Define shell commands and scripts in YAML configuration to create monitoring dashboards, system administration tools, and interactive applications. Commands execute in separate threads with clean process management and real-time output display.

### Key Features

- Unix Command Integration: Transform shell commands into dashboard panels
- Multi-threaded Execution: Each command runs in isolated threads
- YAML Configuration: Define command pipelines and layouts declaratively
- Real-time Updates: Configurable refresh intervals for command execution
- Interactive Elements: Navigate between commands and control execution
- Socket Communication: External control and data injection via Unix sockets
- Process Management: Clean handling of long-running and periodic commands
- Layout System: Organize command outputs in structured layouts
- Data Visualization: Display command output as charts and logs
- Cross-platform: Works with standard Unix tooling

## Use Cases

- System Monitoring: Combine `top`, `df`, `iostat` into unified dashboards
- DevOps Tools: Orchestrate deployment scripts with real-time feedback
- Log Analysis: Run `tail`, `grep`, `awk` commands with structured output
- Network Monitoring: Execute `netstat`, `ss`, `ping` with visual organization
- Database Operations: Run queries and maintenance scripts with progress tracking
- Development Workflows: Combine build, test, and deployment commands

## Quick Start

### Prerequisites

- Rust (latest stable version) - [Install Rust](https://rustup.rs/)
- Shell access (bash/zsh) for script execution
- Optional: `gnuplot` for charting features

### Installation

1. Clone the repository

   ```bash
   git clone https://github.com/jowharshamshiri/boxmux.git
   cd boxmux
   ```

2. Build BoxMux

   ```bash
   cargo build --release
   ```

3. Run the example dashboard

   ```bash
   ./run_boxmux.sh layouts/dashboard.yaml
   ```

### Your First Interface

Create a simple interface with a single panel:

```yaml
# my-interface.yaml
app:
  layouts:
    - id: 'main'
      root: true
      title: 'My First Interface'
      bg_color: 'black'
      children:
        - id: 'welcome'
          title: 'Welcome Panel'
          position:
            x1: 10%
            y1: 20%
            x2: 90%
            y2: 80%
          content: 'Hello, BoxMux!'
          border: true
```

Run it:

```bash
./run_boxmux.sh my-interface.yaml
```

## Documentation

### Core Concepts

- [Getting Started](docs/getting-started.md) - Step-by-step guide to your first interface
- [Configuration Reference](docs/configuration.md) - YAML configuration guide
- [Examples](docs/examples.md) - Real-world examples and use cases
- [API Reference](docs/api.md) - Socket messaging and programmatic control

### Topics

- [Layouts & Positioning](docs/layouts.md) - Creating layouts
- [Scripting & Automation](docs/scripting.md) - Integrating shell scripts
- [Themes & Styling](docs/themes.md) - Customizing appearance
- [Performance & Optimization](docs/performance.md) - Best practices

## Interface Components

### Panels

- Content Panels: Display static or dynamic text
- Interactive Menus: Navigate and select options
- Chart Panels: Visualize data with ASCII charts
- Log Panels: Monitor log files and streams
- Input Panels: Handle user input and commands

### Features

- Tab Navigation: Move between interactive elements
- Keyboard Shortcuts: Custom keybindings and actions
- Real-time Updates: Automatic refresh intervals
- Scrolling: Handle large content with scroll support
- Borders & Styling: Customize appearance
- Color Themes: Color customization

## Configuration Structure

BoxMux uses a hierarchical YAML structure:

```yaml
app:
  libs:                    # External script libraries
    - lib/utils.sh
  layouts:                 # Layout definitions
    - id: 'dashboard'
      root: true
      title: 'Dashboard'
      children:              # Nested panels
        - id: 'header'
          title: 'Header'
          position:          # Percentage-based positioning
            x1: 0%
            y1: 0%
            x2: 100%
            y2: 10%
          content: 'Welcome'
        - id: 'menu'
          title: 'Menu'
          choices:           # Interactive menu items
            - id: 'option1'
              content: 'Option 1'
              script:
                - echo 'Selected option 1'
```

## Socket Integration

BoxMux supports real-time communication via Unix sockets:

```bash
# Update panel content
echo '{"UpdatePanel": {"panel_id": "status", "content": "Connected"}}' | nc -U /tmp/boxmux.sock

# Send commands
echo '{"Command": {"action": "refresh", "panel_id": "logs"}}' | nc -U /tmp/boxmux.sock
```

## Example Gallery

### System Monitor

```yaml
# Real-time system monitoring dashboard
- id: 'cpu_chart'
  title: 'CPU Usage'
  refresh_interval: 1000
  script:
    - top -l 1 | grep "CPU usage" | awk '{print $3}' | sed 's/%//'
```

### Interactive Menu

```yaml
# Navigation menu with actions
- id: 'main_menu'
  title: 'Actions'
  tab_order: 1
  choices:
    - id: 'deploy'
      content: 'Deploy Application'
      script:
        - ./deploy.sh
    - id: 'logs'
      content: 'View Logs'
      script:
        - tail -f /var/log/app.log
```

### Data Visualization

```yaml
# Chart with live data
- id: 'metrics_chart'
  title: 'Performance Metrics'
  refresh_interval: 5000
  script:
    - gnuplot -e "set terminal dumb; plot '/tmp/metrics.dat' with lines"
```

## Development

### Building from Source

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Run with specific layout
cargo run -- layouts/dashboard.yaml
```

### Project Structure

```
boxmux/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library exports
│   ├── draw_utils.rs        # Rendering utilities
│   ├── thread_manager.rs    # Thread management
│   └── model/               # Data structures
├── layouts/                 # Example configurations
├── docs/                    # Documentation
└── examples/               # Example interfaces
```

## Contributing

Contributions welcome. Please read our [Contributing Guidelines](CONTRIBUTING.md) for details.

### Development Setup

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes
4. Add tests for new functionality
5. Run tests: `cargo test`
6. Commit your changes: `git commit -m 'Add amazing feature'`
7. Push to the branch: `git push origin feature/amazing-feature`
8. Open a Pull Request

## Performance

BoxMux performance characteristics:

- Low Memory: Minimal memory footprint
- Fast Rendering: Optimized screen updates
- Efficient Threading: Multi-threaded architecture
- Responsive: Sub-millisecond input handling

## Troubleshooting

### Common Issues

**Installation Problems**

- Ensure Rust is installed: `rustc --version`
- Update Rust: `rustup update`

**Runtime Issues**

- Check YAML syntax with `yamllint`
- Verify script permissions
- Check terminal compatibility

**Performance Issues**

- Reduce refresh intervals
- Optimize scripts
- Monitor memory usage

For more help, see our [Troubleshooting Guide](docs/troubleshooting.md).

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---
