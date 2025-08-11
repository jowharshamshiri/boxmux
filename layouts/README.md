# BoxMux Professional Dashboard Showcases

This directory contains impressive showcase layouts that demonstrate BoxMux's capabilities for real-world professional scenarios.

## Available Showcases

### 🎯 Showcase Menu
**File**: `showcase_menu.yaml`  
**Purpose**: Interactive menu to explore all available showcases  
**Usage**: `boxmux layouts/showcase_menu.yaml`

---

### 🚀 DevOps Control Center
**File**: `devops_control_center.yaml`  
**Target Audience**: System administrators, DevOps engineers, infrastructure monitoring  

**Features**:
- **Service Management**: Docker, Nginx, database connectivity, process monitoring
- **System Metrics**: Real-time CPU, memory, disk I/O, network interfaces  
- **Live Log Monitoring**: System events, error analysis, security monitoring
- **Quick Actions**: System info, network tests, security scans
- **Professional Theme**: Dark with blue/cyan accents

---

### 💻 Developer Workspace
**File**: `developer_workspace.yaml`  
**Target Audience**: Software developers, project managers, development teams

**Features**:
- **Project Explorer**: Structure analysis, Git integration, dependency management
- **Build & Test Tools**: Multi-language support (Rust, Node.js, Java), automated testing
- **Development Metrics**: Code statistics, Git activity, contributor tracking
- **Integrated Output**: Build results, test execution, real-time feedback
- **Modern Theme**: Dark with green syntax highlighting colors

---

### 📊 System Monitor Pro
**File**: `system_monitor_pro.yaml`  
**Target Audience**: System administrators, NOC operators, performance tuning

**Features**:
- **Performance Metrics**: CPU usage, memory utilization, disk I/O, network activity
- **Process Management**: Resource identification, system/user breakdown
- **Health Controls**: Automated checks, performance profiling, log analysis
- **Alert Integration**: Threshold-based alerting, trend analysis
- **High-Contrast Theme**: Red/yellow for critical monitoring

---

### 🌐 Network Operations Center
**File**: `network_operations.yaml`  
**Target Audience**: Network administrators, security teams, NOC operations

**Features**:
- **Interface Monitoring**: Status tracking, IP management, traffic statistics
- **Connection Analysis**: Active connections, port monitoring, service discovery
- **Network Tools**: Connectivity testing, DNS lookup, port scanning, traceroute
- **Security Monitoring**: Firewall status, SSH security, authentication monitoring
- **Network Theme**: Blue/cyan dominant for network focus

---

### 🗄️ Database Administration Console
**File**: `database_admin.yaml`  
**Target Audience**: Database administrators, data engineers, backend developers

**Features**:
- **Multi-DB Support**: MySQL, PostgreSQL, Redis, MongoDB management
- **Performance Monitoring**: Resource utilization, connection pools, query metrics
- **Admin Tools**: Backup verification, log analysis, maintenance scheduling
- **User Management**: Permissions, security recommendations
- **Data Theme**: Purple/magenta for database operations

---

## BoxMux Framework Features Demonstrated

### ✅ Core Capabilities
- **YAML-Driven Configuration**: Declarative interface design
- **Multi-threaded Architecture**: Efficient concurrent operations
- **Real-time Updates**: Live data refresh with configurable intervals
- **Interactive Panels**: Choice-based navigation and command execution
- **Professional Themes**: Industry-standard color schemes
- **Cross-platform Support**: macOS and Linux compatibility

### ✅ Advanced Features
- **Script Integration**: Shell command execution with output capture
- **Output Redirection**: Route command results between panels
- **Flexible Layouts**: Percentage-based positioning and sizing
- **Tab Navigation**: Keyboard-driven panel selection
- **Border Customization**: Visual panel separation and theming
- **Content Management**: Dynamic text updates and formatting

### ✅ Professional Quality
- **Error Handling**: Graceful degradation and informative messages
- **Performance**: Sub-millisecond input response, efficient rendering
- **Validation**: Comprehensive YAML and configuration validation
- **Documentation**: Extensive inline help and examples
- **Testing**: 309+ tests ensuring reliability and stability

## Usage Examples

```bash
# Start the showcase menu
boxmux layouts/showcase_menu.yaml

# Launch specific dashboards
boxmux layouts/devops_control_center.yaml
boxmux layouts/developer_workspace.yaml
boxmux layouts/system_monitor_pro.yaml
boxmux layouts/network_operations.yaml
boxmux layouts/database_admin.yaml

# Adjust refresh rate (optional)
boxmux layouts/devops_control_center.yaml --frame_delay 50
```

## Creating Custom Showcases

Each showcase demonstrates different BoxMux patterns:

1. **Panel Layouts**: From simple 2-panel to complex multi-nested hierarchies
2. **Color Themes**: Professional color schemes for different domains
3. **Script Integration**: Real system commands with proper error handling
4. **Interactive Navigation**: Choice panels with dynamic content updates
5. **Real-time Monitoring**: Automatic refresh intervals for live data

Use these showcases as templates for your own professional dashboards!

---

*Generated for BoxMux v0.115+ - Professional Terminal UI Framework*