# BoxMux

**BoxMux** is an advanced terminal-based dashboard application for managing and visualizing real-time system metrics, logs, and custom data. It supports dynamic configurations through socket messaging, enabling real-time updates and complex data visualizations.

![BoxMux Dashboard Screenshot](./assets/boxmux_screenshot.png)

## Features

- **Dynamic Layouts:** Configure multiple layouts with intricate panels.
- **Real-Time Updates:** Real-time data visualization and log monitoring.
- **Socket Messaging:** Send and receive commands through Unix sockets.
- **Customizable Widgets:** Easily extend with new data sources and visualizations.
- **Platform Support:** Works seamlessly on macOS and Linux.

## Table of Contents

1. [Installation](#installation)
2. [Configuration](#configuration)
3. [Usage](#usage)
4. [Socket Messaging](#socket-messaging)
5. [Customization](#customization)
6. [Contributing](#contributing)
7. [License](#license)

## Installation

To install BoxMux, follow these steps:

### Prerequisites

- **Rust**: Ensure you have Rust installed. [Install Rust](https://www.rust-lang.org/tools/install)
- **nc (netcat)**: Required for socket communication. Install via package manager (e.g., `brew install netcat` for macOS).

### Clone the Repository

```bash
git clone https://github.com/your-username/boxmux.git
cd boxmux
```

### Build the Application

```bash
cargo build --release
```

### Running the Application

```bash
./target/release/boxmux
```

## Configuration

BoxMux configuration is defined in a YAML file. The default configuration file is `config.yaml`. Here’s a sample configuration:

```yaml
layouts:
  - id: 'dashboard'
    root: true
    title: 'Advanced Dashboard Layout'
    selected_fg_color: 'bright_yellow'
    bg_color: 'bright_black'
    selected_bg_color: 'bright_black'
    title_fg_color: 'white'
    selected_title_fg_color: 'black'
    title_bg_color: 'black'
    selected_title_bg_color: 'bright_yellow'
    border_color: 'bright_white'
    selected_border_color: 'bright_yellow'
    children:
      - id: 'header'
        title: 'Header'
        position:
          x1: 0%
          y1: 0%
          x2: 100%
          y2: 5%
        children:
          - id: 'title'
            title: 'Dashboard Title'
            position:
              x1: 2%
              y1: 0%
              x2: 30%
              y2: 100%
            script:
              - echo 'My Awesome Dash'
          - id: 'status'
            title: 'Status'
            position:
              x1: 70%
              y1: 0%
              x2: 98%
              y2: 100%
            script:
              - echo 'Status Running'
      - id: 'sidebar'
        title: 'Sidebar'
        position:
          x1: 0%
          y1: 5%
          x2: 20%
          y2: 95%
        children:
          - id: 'menu1'
            position:
              x1: 10%
              y1: 5%
              x2: 90%
              y2: 15%
            tab_order: 1
            script:
              - echo 'Option 1'
          # More configurations...
```

### Customizing Configuration

Modify the `config.yaml` file to suit your needs. The `children` key allows you to add nested components like charts, logs, and buttons.

## Usage

### Basic Usage

After configuring, start BoxMux with:

```bash
./target/release/boxmux --config /path/to/your/config.yaml
```

Replace `/path/to/your/config.yaml` with the path to your customized configuration file.

### Sending Commands via Sockets

BoxMux listens on a Unix socket for commands. You can send JSON-formatted messages to the socket. For example:

```bash
echo -n '{"UpdatePanel": {"panel_id": "menu2", "content": "hey"}}' | nc -U /tmp/boxmux.sock
```

Ensure the message format matches the expected structure defined in your application logic.

## Socket Messaging

BoxMux uses socket messaging to receive updates. Here's how to send a JSON message to update a panel:

### Example Command

```json
{
  "UpdatePanel": {
    "panel_id": "menu2",
    "content": "hey"
  }
}
```

### Sending Messages

Use `nc` (netcat) to send commands to the BoxMux socket:

```bash
echo -n '{"UpdatePanel": {"panel_id": "menu2", "content": "hey"}}' | nc -U /tmp/boxmux.sock
```

## Customization

### Adding New Panels

You can add new panels by modifying the `config.yaml` file. For example, to add a new chart, define it under the `children` of the desired layout:

```yaml
- id: 'chart5'
  title: 'New Chart'
  position:
    x1: 50%
    y1: 0%
    x2: 100%
    y2: 50%
  script:
    - some_script_to_generate_chart
```

### Extending Functionality

Extend BoxMux by adding new functions and scripts to handle more complex interactions and visualizations. Refer to the project's documentation for guidelines on extending the app.

## Contributing

We welcome contributions to BoxMux! Here’s how you can help:

1. **Fork the repository**
2. **Create a new branch** (`git checkout -b feature/your-feature`)
3. **Commit your changes** (`git commit -am 'Add new feature'`)
4. **Push to the branch** (`git push origin feature/your-feature`)
5. **Open a pull request**

Please read the [CONTRIBUTING.md](CONTRIBUTING.md) for more details on our code of conduct and the process for submitting pull requests.

## License

BoxMux is licensed under the [MIT License](LICENSE).
