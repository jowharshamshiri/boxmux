---
layout: default
title: Advanced Features - BoxMux
---

# Advanced Features

BoxMux includes clipboard integration, scrolling, and performance monitoring.

## Table of Contents

- [Clipboard Integration](#clipboard-integration)
- [Enhanced Scrolling](#enhanced-scrolling)
- [Performance Monitoring](#performance-monitoring)
- [Configuration Schema Validation](#configuration-schema-validation)
- [Manual Socket Implementation](#manual-socket-implementation)
- [Real-World Examples](#real-world-examples)


## Clipboard Integration

BoxMux provides platform-specific clipboard integration with visual feedback.

### Features

- **Ctrl+C Integration**: Copy focused panel content to system clipboard
- **Visual Feedback**: Brief visual indication when content is copied
- **Platform Support**: Works on macOS, Linux, and Windows
- **Content Selection**: Copies complete panel content or selected regions

### Usage

1. **Navigate** to a panel using Tab key or mouse
2. **Press Ctrl+C** to copy panel content to clipboard
3. **Visual flash** indicates successful copy operation
4. **Paste** content in any application using standard clipboard operations

### Configuration

```yaml
# Enable clipboard for specific panels
- id: 'results_panel'
  title: 'Command Results'
  clipboard_enabled: true  # Allow clipboard copying
  script:
    - ps aux | head -20
```

### Implementation Details

- Platform-specific clipboard APIs:
  - **macOS**: `pbcopy` command integration
  - **Linux**: `xclip` or `xsel` command integration
  - **Windows**: Windows clipboard API
- Visual feedback through brief color change
- Error handling for clipboard access failures

## Enhanced Scrolling

BoxMux provides advanced scrolling capabilities with position preservation and navigation controls.

### Features

- **Position Preservation**: Maintains scroll position during auto-refresh
- **Page Navigation**: Page Up/Down keyboard support for efficient scrolling  
- **Visual Indicators**: Scroll position indicators and scrollbar detection
- **Smooth Scrolling**: Smooth scrolling with configurable scroll amounts
- **Auto-sizing**: Automatic scrollbar detection for focusable panels

### Keyboard Controls

- **Arrow Keys**: Line-by-line scrolling (Up/Down)
- **Page Up/Down**: Page-based scrolling (10x scroll amount)
- **Home/End**: Jump to beginning/end of content
- **Mouse Wheel**: Scroll support (where available)

### Configuration

```yaml
# Panel with enhanced scrolling
- id: 'scrollable_output'
  title: 'Large Output'
  position: {x1: 10%, y1: 10%, x2: 90%, y2: 80%}
  scroll: true
  scroll_config:
    preserve_position: true    # Maintain position during refresh
    show_indicators: true      # Show scroll position
    page_size: 10             # Lines per page scroll
    smooth_scrolling: true    # Enable smooth scrolling
  refresh_interval: 2000
  script:
    - ps aux | head -100  # Large output for scrolling demo
```

### Advanced Scrolling Features

```yaml
# Auto-refresh with scroll preservation
- id: 'live_data'
  title: 'Live Data Stream'
  scroll: true
  scroll_config:
    preserve_position: true
    auto_scroll_new: false    # Don't auto-scroll to new content
    scroll_buffer_size: 1000  # Maximum lines to keep
  refresh_interval: 1000
  script:
    - date && ps aux | head -50
```

## Performance Monitoring

BoxMux includes built-in performance monitoring and benchmarking capabilities.

### Performance Benchmarks

BoxMux tracks performance metrics for core operations:

- **ANSI Stripping**: 10k operations in ~1.5s
- **Key Mapping**: 150k operations in ~2.2s  
- **Bounds Calculation**: 100k operations in ~20ms
- **Script Execution**: 100 operations in ~575ms
- **Large Config Processing**: 1k operations in ~38ms

### Monitoring Configuration

```yaml
# Performance monitoring panel
- id: 'performance'
  title: 'System Performance'
  position: {x1: 5%, y1: 10%, x2: 95%, y2: 50%}
  performance_monitoring: true
  refresh_interval: 5000
  script:
    - |
      echo "BoxMux Performance Metrics:"
      echo "=========================="
      echo "Memory Usage: $(ps -o rss= -p $$) KB"
      echo "CPU Usage: $(ps -o %cpu= -p $$)%"
      echo "Uptime: $(uptime -p)"
      echo "Active Panels: $(pgrep -f boxmux | wc -l)"
```

### Performance Testing

```yaml
# Load testing configuration
- id: 'load_test'
  title: 'Load Test Results'
  performance_test: true
  test_config:
    duration: 60        # Test duration in seconds
    refresh_rate: 100   # Fast refresh for stress testing
    data_volume: 'high' # High data volume test
  script:
    - |
      # Generate high-volume output for testing
      for i in {1..1000}; do
        echo "Test line $i: $(date)"
      done
```

## Configuration Schema Validation

BoxMux includes comprehensive JSON Schema validation for YAML configurations.

### Features

- **Automatic Validation**: Configuration files validated on load
- **Detailed Error Messages**: Line/column specific error reporting
- **Schema Coverage**: Complete schema validation for all configuration sections
- **Type Checking**: Strict type validation and required field checking

### Error Reporting

When loading invalid configuration:

```bash
$ boxmux invalid-config.yaml
Error: Configuration validation failed
  --> invalid-config.yaml:15:3
   |
15 |   position: "invalid"
   |   ^^^^^^^^ expected object with x1, y1, x2, y2 properties
   |
   = help: position must be an object with percentage-based coordinates
```

### Schema Validation Configuration

```yaml
# Enable strict validation
app:
  validation:
    schema_validation: true     # Enable JSON schema validation
    strict_mode: true          # Fail on any validation error
    validate_scripts: true     # Validate script syntax
    validate_colors: true      # Validate color names
```

### Custom Validation Rules

```yaml
# Custom validation for specific fields
app:
  validation:
    custom_rules:
      - field: 'refresh_interval'
        min_value: 100           # Minimum refresh interval
        max_value: 300000        # Maximum refresh interval
      - field: 'position'
        validate_bounds: true    # Validate position bounds
```

## Manual Socket Implementation

BoxMux uses a manual Unix socket implementation without external dependencies for maximum control and reliability.

### Features

- **Manual Implementation**: Direct Unix socket handling using `std::os::unix::net`
- **No External Dependencies**: Self-contained socket communication
- **Full Control**: Complete control over socket lifecycle and message handling
- **Error Recovery**: Comprehensive error handling and connection recovery
- **Performance**: Optimized for low-latency communication

### Socket Commands

BoxMux supports comprehensive socket API:

```bash
# Update panel content
echo '{"UpdatePanel": {"panel_id": "status", "content": "Connected"}}' | nc -U /tmp/boxmux.sock

# Replace panel script
echo '{"ReplaceScript": {"panel_id": "monitor", "script": ["uptime"]}}' | nc -U /tmp/boxmux.sock

# Switch layouts
echo '{"SwitchLayout": {"layout_id": "dashboard"}}' | nc -U /tmp/boxmux.sock

# Add new panel
echo '{"AddPanel": {"parent_id": "main", "panel": {...}}}' | nc -U /tmp/boxmux.sock

# Control refresh
echo '{"StartRefresh": {"panel_id": "logs"}}' | nc -U /tmp/boxmux.sock
echo '{"StopRefresh": {"panel_id": "logs"}}' | nc -U /tmp/boxmux.sock
```

### Socket Configuration

```yaml
# Socket server configuration
app:
  socket_config:
    socket_path: '/tmp/boxmux.sock'
    permissions: 0660
    buffer_size: 8192
    timeout: 5000
    max_connections: 10
```

## Real-World Examples

### DevOps Dashboard

```yaml
app:
  layouts:
    - id: 'devops'
      root: true
      title: 'DevOps Command Center'
      children:
        # Live deployment logs
        - id: 'deploy_logs'
          title: 'Deployment Logs'
          position: {x1: 5%, y1: 10%, x2: 60%, y2: 50%}
          clipboard_enabled: true
          scroll: true
          scroll_config:
            preserve_position: true
            show_indicators: true
          script:
            - kubectl logs -f deployment/api-server
            
        # Performance monitoring
        - id: 'cluster_perf'
          title: 'Cluster Performance'
          position: {x1: 65%, y1: 10%, x2: 95%, y2: 50%}
          performance_monitoring: true
          refresh_interval: 2000
          script:
            - kubectl top nodes
            - echo "---"
            - kubectl top pods
            
        # Interactive control panel
        - id: 'controls'
          title: 'Deployment Controls'
          position: {x1: 5%, y1: 55%, x2: 95%, y2: 90%}
          tab_order: '1'
          choices:
            - id: 'scale_up'
              content: 'Scale Up (3→5 replicas)'
              script:
                - kubectl scale deployment/api-server --replicas=5
              redirect_output: 'deploy_logs'
            - id: 'rollback'
              content: 'Rollback to Previous Version'  
              script:
                - kubectl rollout undo deployment/api-server
              redirect_output: 'deploy_logs'
```

### System Monitoring with Advanced Features

```yaml
app:
  layouts:
    - id: 'advanced_monitor'
      root: true
      title: 'Advanced System Monitor'
      children:
        # System logs
        - id: 'system_logs'
          title: 'System Logs'
          position: {x1: 5%, y1: 10%, x2: 48%, y2: 60%}
          clipboard_enabled: true
          scroll: true
          scroll_config:
            preserve_position: true
            auto_scroll_new: true
            scroll_buffer_size: 2000
          script:
            - tail -f /var/log/system.log | grep -E "(error|warning|critical)"
            
        # Performance metrics with clipboard
        - id: 'perf_metrics'
          title: 'Performance Metrics'
          position: {x1: 52%, y1: 10%, x2: 95%, y2: 60%}
          clipboard_enabled: true
          performance_monitoring: true
          refresh_interval: 1000
          script:
            - |
              echo "=== System Performance ==="
              echo "CPU: $(top -l 1 | grep "CPU usage" | awk '{print $3}')"
              echo "Memory: $(free | awk 'NR==2{printf "%.1f%%", $3/$2*100}')"
              echo "Load: $(uptime | awk -F'load average:' '{print $2}')"
              echo "Processes: $(ps aux | wc -l)"
              echo "BoxMux Memory: $(ps -o rss= -p $$) KB"
              
        # Enhanced table with all features
        - id: 'process_table'
          title: 'Top Processes'
          position: {x1: 5%, y1: 65%, x2: 95%, y2: 90%}
          table_config:
            headers: ['Command', 'CPU %', 'Memory %', 'PID', 'User']
            sortable: true
            filterable: true
            page_size: 8
            zebra_striping: true
            show_row_numbers: true
            border_style: 'rounded'
          clipboard_enabled: true
          refresh_interval: 3000
          script:
            - ps aux --no-headers | awk '{printf "%s,%.1f,%.1f,%s,%s\n", $11, $3, $4, $2, $1}' | 
              sort -rn -k2 -t, | head -20
```

### Development Environment Monitor

```yaml
app:
  layouts:
    - id: 'dev_monitor'
      root: true
      title: 'Development Environment'
      children:
        # Build output
        - id: 'build_output'
          title: 'Build Output'
          position: {x1: 5%, y1: 10%, x2: 60%, y2: 70%}
          clipboard_enabled: true
          scroll: true
          scroll_config:
            preserve_position: false  # Always show latest for builds
            auto_scroll_new: true
          script:
            - cargo watch -x build
            
        # Git status with clipboard support
        - id: 'git_status'
          title: 'Git Status'
          position: {x1: 65%, y1: 10%, x2: 95%, y2: 40%}
          clipboard_enabled: true
          refresh_interval: 5000
          script:
            - git status --porcelain
            - echo "---"
            - git log --oneline -5
            
        # Test results with performance monitoring
        - id: 'test_results'
          title: 'Test Suite'
          position: {x1: 65%, y1: 45%, x2: 95%, y2: 70%}
          performance_monitoring: true
          clipboard_enabled: true
          refresh_interval: 10000
          script:
            - |
              echo "Running test suite..."
              start_time=$(date +%s%3N)
              cargo test --quiet 2>&1 | head -20
              end_time=$(date +%s%3N)
              duration=$((end_time - start_time))
              echo "Test duration: ${duration}ms"
```

---

For configuration details, see [Configuration Reference](configuration.md).  
For basic usage, see [User Guide](user-guide.md).  
For plugin development, see [Plugin System](plugin-system.md).