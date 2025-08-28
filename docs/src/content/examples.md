---
title: Configuration Examples
description: BoxMux configuration examples for various use cases and application types
---


## System Monitoring Dashboard

A system monitoring application with real-time stats and interactive controls.

```yaml
# system_monitor.yaml
title: "System Monitor Dashboard"
refresh_rate: 50

variables:
  hostname: "{{HOSTNAME}}"
  user: "{{USER}}"

layouts:
  main:
    type: "MuxBox"
    bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "100%" }
    children:
      header:
        bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "15%" }
        content: "📊 {{hostname}} System Monitor - User: {{user}}"
        border_color: "BrightCyan"
        title_color: "BrightYellow"
        
      cpu_stats:
        bounds: { x1: "0%", y1: "15%", x2: "33%", y2: "60%" }
        title: "CPU Usage"
        border_color: "BrightGreen"
        script: |
          #!/bin/bash
          while true; do
            top -l1 | grep "CPU usage" | awk '{print $3, $5}'
            sleep 1
          done
        refresh_interval: 1000
        thread: true
        streaming: true
        
      memory_stats:
        bounds: { x1: "33%", y1: "15%", x2: "66%", y2: "60%" }
        title: "Memory Usage"
        border_color: "BrightBlue" 
        script: |
          #!/bin/bash
          vm_stat | grep -E "(Pages free|Pages active|Pages wired)" | 
          awk '{printf "%s: %d MB\n", $1$2, $3*4/1024}'
        refresh_interval: 2000
        thread: true
        
      disk_stats:
        bounds: { x1: "66%", y1: "15%", x2: "100%", y2: "60%" }
        title: "Disk Usage"
        border_color: "BrightMagenta"
        script: "df -h | grep -E '^/dev/'"
        refresh_interval: 5000
        
      control_panel:
        bounds: { x1: "0%", y1: "60%", x2: "50%", y2: "100%" }
        title: "Actions"
        border_color: "BrightYellow"
        choices:
          - name: "🔄 Refresh All Stats"
            script: "echo 'Refreshing all statistics...'"
            redirect_output: "log_output"
            
          - name: "📋 Copy System Info"
            script: |
              echo "System: $(uname -a)"
              echo "Uptime: $(uptime)"
              echo "Load: $(uptime | awk '{print $NF}')"
            redirect_output: "log_output"
            
          - name: "⚡ Interactive Terminal"
            script: "bash"
            pty: true
            redirect_output: "terminal_box"
            
      log_output:
        bounds: { x1: "50%", y1: "60%", x2: "100%", y2: "85%" }
        title: "Output Log"
        border_color: "BrightWhite"
        content: "Action output will appear here..."
        auto_scroll: true
        
      status_bar:
        bounds: { x1: "50%", y1: "85%", x2: "100%", y2: "100%" }
        border_color: "BrightBlack"
        script: "date '+Last Updated: %Y-%m-%d %H:%M:%S'"
        refresh_interval: 1000

active: "main"

# Global key bindings
on_keypress:
  "Ctrl+r": "refresh_all"
  "Ctrl+q": "quit"
  "F1": "show_help"
```

## Development Dashboard

A developer-focused dashboard for project management, testing, and deployment.

```yaml
# dev_dashboard.yaml
title: "Development Dashboard"
refresh_rate: 100

variables:
  project_name: "My Awesome Project"
  git_branch: "{{GIT_BRANCH}}"
  node_env: "development"

layouts:
  main:
    type: "MuxBox"
    bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "100%" }
    children:
      header:
        bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "10%" }
        content: "🚀 {{project_name}} - Branch: {{git_branch}}"
        border_color: "BrightCyan"
        
      git_status:
        bounds: { x1: "0%", y1: "10%", x2: "30%", y2: "50%" }
        title: "Git Status"
        border_color: "BrightGreen"
        script: |
          git status --porcelain | head -20
          echo "---"
          git log --oneline -5
        refresh_interval: 3000
        
      actions:
        bounds: { x1: "30%", y1: "10%", x2: "60%", y2: "50%" }
        title: "Quick Actions"
        border_color: "BrightYellow"
        choices:
          - name: "🔨 Build Project"
            script: "npm run build"
            redirect_output: "build_log"
            streaming: true
            thread: true
            
          - name: "🧪 Run Tests"
            script: "npm test"
            redirect_output: "test_log"
            streaming: true
            pty: true
            
          - name: "📦 Install Dependencies"
            script: "npm install"
            redirect_output: "build_log"
            streaming: true
            
          - name: "🌐 Start Dev Server"
            script: "npm run dev"
            pty: true
            streaming: true
            
          - name: "📝 Open Editor"
            script: "code ."
            
      package_info:
        bounds: { x1: "60%", y1: "10%", x2: "100%", y2: "50%" }
        title: "Package Info"
        border_color: "BrightBlue"
        script: |
          echo "Dependencies:"
          cat package.json | jq -r '.dependencies | keys[]' | head -10
          echo ""
          echo "Scripts:"
          cat package.json | jq -r '.scripts | keys[]'
        refresh_interval: 10000
        
      build_log:
        bounds: { x1: "0%", y1: "50%", x2: "50%", y2: "85%" }
        title: "Build Output"
        border_color: "BrightMagenta"
        content: "Build output will appear here..."
        overflow_behavior: "scroll"
        auto_scroll: true
        
      test_log:
        bounds: { x1: "50%", y1: "50%", x2: "100%", y2: "85%" }
        title: "Test Results"
        border_color: "BrightRed"
        content: "Test results will appear here..."
        overflow_behavior: "scroll"
        auto_scroll: true
        
      footer:
        bounds: { x1: "0%", y1: "85%", x2: "100%", y2: "100%" }
        border_color: "BrightBlack"
        script: |
          echo "Node: $(node --version) | NPM: $(npm --version) | $(date)"

active: "main"

# Hot keys for common actions
hot_keys:
  "Ctrl+b": 
    choice_id: "build"
    box_id: "actions"
  "Ctrl+t":
    choice_id: "test"
    box_id: "actions"
```

## Server Management Interface

A server administration interface with PTY terminals and monitoring.

```yaml
# server_admin.yaml  
title: "Server Administration"
refresh_rate: 75

variables:
  server_ip: "192.168.1.100"
  admin_user: "admin"

layouts:
  main:
    type: "MuxBox"
    bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "100%" }
    children:
      title_bar:
        bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "8%" }
        content: "🖥️  Server Admin Panel - {{server_ip}}"
        border_color: "BrightCyan"
        
      server_menu:
        bounds: { x1: "0%", y1: "8%", x2: "25%", y2: "100%" }
        title: "Server Actions"
        border_color: "BrightGreen"
        choices:
          - name: "📊 System Status"
            script: |
              uptime
              free -h
              df -h
            redirect_output: "main_output"
            
          - name: "🔍 Process Monitor"
            script: "htop"
            pty: true
            redirect_output: "terminal_area"
            
          - name: "📁 File Manager" 
            script: "ranger"
            pty: true
            redirect_output: "terminal_area"
            
          - name: "🌐 Network Status"
            script: |
              netstat -tulpn | head -20
              echo "---"
              ss -tuln
            redirect_output: "main_output"
            
          - name: "🔐 SSH to Remote"
            script: "ssh {{admin_user}}@{{server_ip}}"
            pty: true
            redirect_output: "terminal_area"
            
          - name: "📋 Service Status"
            script: "systemctl status"
            redirect_output: "main_output"
            
      main_output:
        bounds: { x1: "25%", y1: "8%", x2: "65%", y2: "100%" }
        title: "Command Output"
        border_color: "BrightBlue"
        content: "Select an action to see output..."
        overflow_behavior: "scroll"
        auto_scroll: true
        
      terminal_area:
        bounds: { x1: "65%", y1: "8%", x2: "100%", y2: "80%" }
        title: "⚡ Interactive Terminal"
        border_color: "BrightCyan"
        pty: true
        script: "bash"
        
      system_info:
        bounds: { x1: "65%", y1: "80%", x2: "100%", y2: "100%" }
        title: "Live Stats"
        border_color: "BrightYellow"
        script: |
          echo "Load: $(uptime | awk '{print $NF}')"
          echo "Memory: $(free | awk '/^Mem:/{printf "%.1f%%", $3/$2*100}')"
          echo "Disk: $(df / | awk 'NR==2{print $5}')"
        refresh_interval: 2000
        thread: true

active: "main"
```

## Docker Container Manager

A Docker management interface with container controls and log monitoring.

```yaml
# docker_manager.yaml
title: "Docker Container Manager"
refresh_rate: 100

variables:
  docker_host: "{{DOCKER_HOST}}"

layouts:
  main:
    type: "MuxBox"
    bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "100%" }
    children:
      header:
        bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "10%" }
        content: "🐳 Docker Manager - Host: {{docker_host}}"
        border_color: "BrightBlue"
        
      container_list:
        bounds: { x1: "0%", y1: "10%", x2: "40%", y2: "100%" }
        title: "Running Containers"
        border_color: "BrightGreen"
        script: "docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'"
        refresh_interval: 3000
        overflow_behavior: "scroll"
        
      container_actions:
        bounds: { x1: "40%", y1: "10%", x2: "60%", y2: "60%" }
        title: "Container Actions"
        border_color: "BrightYellow"
        choices:
          - name: "📋 List All Containers"
            script: "docker ps -a"
            redirect_output: "docker_output"
            
          - name: "🔍 Container Stats"
            script: "docker stats --no-stream"
            redirect_output: "docker_output"
            
          - name: "🗄️ List Images"
            script: "docker images"
            redirect_output: "docker_output"
            
          - name: "🧹 Clean Up"
            script: |
              docker system prune -f
              docker image prune -f
            redirect_output: "docker_output"
            streaming: true
            
          - name: "📊 System Info"
            script: "docker system df"
            redirect_output: "docker_output"
            
      docker_compose:
        bounds: { x1: "40%", y1: "60%", x2: "60%", y2: "100%" }
        title: "Docker Compose"
        border_color: "BrightMagenta"
        choices:
          - name: "🚀 Compose Up"
            script: "docker-compose up -d"
            redirect_output: "docker_output"
            streaming: true
            
          - name: "⏹️ Compose Down"
            script: "docker-compose down"
            redirect_output: "docker_output"
            
          - name: "📋 Compose Status"
            script: "docker-compose ps"
            redirect_output: "docker_output"
            
          - name: "📄 View Logs"
            script: "docker-compose logs --tail=50 -f"
            pty: true
            redirect_output: "docker_output"
            
      docker_output:
        bounds: { x1: "60%", y1: "10%", x2: "100%", y2: "100%" }
        title: "Docker Output"
        border_color: "BrightWhite"
        content: "Docker command output will appear here..."
        overflow_behavior: "scroll"
        auto_scroll: true

active: "main"

# Quick docker commands via hot keys
hot_keys:
  "Ctrl+l": 
    choice_id: "list_containers"
    box_id: "container_actions"
  "Ctrl+s":
    choice_id: "stats"
    box_id: "container_actions"
```

## Data Visualization Dashboard

A dashboard showcasing BoxMux's data visualization capabilities with charts and tables.

```yaml
# data_dashboard.yaml
title: "Data Visualization Dashboard"
refresh_rate: 200

variables:
  data_source: "/tmp/metrics.json"

layouts:
  main:
    type: "MuxBox"
    bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "100%" }
    children:
      title:
        bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "10%" }
        content: "📈 Data Visualization Dashboard"
        border_color: "BrightCyan"
        
      cpu_chart:
        bounds: { x1: "0%", y1: "10%", x2: "50%", y2: "50%" }
        title: "CPU Usage Trend"
        border_color: "BrightGreen"
        chart:
          type: "line"
          data: [45, 52, 48, 65, 71, 58, 62, 69, 73, 67]
          labels: ["10s", "20s", "30s", "40s", "50s", "60s", "70s", "80s", "90s", "100s"]
          title: "CPU Usage Over Time"
          y_axis_label: "Percentage"
          
      memory_chart:
        bounds: { x1: "50%", y1: "10%", x2: "100%", y2: "50%" }
        title: "Memory Usage"
        border_color: "BrightBlue"
        chart:
          type: "bar"
          data: [2.1, 3.4, 2.8, 4.1, 3.2]
          labels: ["App1", "App2", "App3", "App4", "App5"]
          title: "Memory Usage by Application"
          y_axis_label: "GB"
          
      process_table:
        bounds: { x1: "0%", y1: "50%", x2: "100%", y2: "100%" }
        title: "Process Table"
        border_color: "BrightMagenta"
        table:
          data_source: "csv"
          data: |
            Process,PID,CPU%,Memory,Status
            nginx,1234,2.3%,45MB,Running
            postgres,5678,15.7%,256MB,Running
            redis,9012,0.8%,12MB,Running
            node,3456,8.4%,128MB,Running
            docker,7890,3.2%,89MB,Running
          headers: true
          sortable: true
          zebra_striping: true
          border_style: "rounded"

active: "main"
```

## Multi-Environment DevOps Control

A DevOps control panel for managing multiple environments.

```yaml
# devops_control.yaml
title: "DevOps Control Center"
refresh_rate: 150

variables:
  env: "production"
  kubectl_context: "prod-cluster"
  aws_profile: "production"

layouts:
  main:
    type: "MuxBox"
    bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "100%" }
    children:
      header:
        bounds: { x1: "0%", y1: "0%", x2: "100%", y2: "8%" }
        content: "☸️ DevOps Control - Environment: {{env}}"
        border_color: "BrightCyan"
        
      kubernetes_menu:
        bounds: { x1: "0%", y1: "8%", x2: "20%", y2: "50%" }
        title: "Kubernetes"
        border_color: "BrightBlue"
        choices:
          - name: "📊 Pod Status"
            script: "kubectl get pods -A"
            redirect_output: "main_output"
            
          - name: "🔧 Services"
            script: "kubectl get svc -A"
            redirect_output: "main_output"
            
          - name: "📈 Top Nodes"
            script: "kubectl top nodes"
            redirect_output: "main_output"
            
          - name: "📋 Events"
            script: "kubectl get events --sort-by='.lastTimestamp'"
            redirect_output: "main_output"
            
      deployment_menu:
        bounds: { x1: "0%", y1: "50%", x2: "20%", y2: "100%" }
        title: "Deployments"
        border_color: "BrightGreen"
        choices:
          - name: "🚀 Deploy App"
            script: "./deploy.sh {{env}}"
            redirect_output: "deployment_log"
            streaming: true
            
          - name: "🔄 Rolling Update"
            script: "kubectl rollout restart deployment/app"
            redirect_output: "deployment_log"
            
          - name: "⏪ Rollback"
            script: "kubectl rollout undo deployment/app"
            redirect_output: "deployment_log"
            
          - name: "📊 Rollout Status"
            script: "kubectl rollout status deployment/app"
            redirect_output: "deployment_log"
            
      main_output:
        bounds: { x1: "20%", y1: "8%", x2: "70%", y2: "60%" }
        title: "Command Output"
        border_color: "BrightWhite"
        content: "Select a command to see output..."
        overflow_behavior: "scroll"
        auto_scroll: true
        
      deployment_log:
        bounds: { x1: "20%", y1: "60%", x2: "70%", y2: "100%" }
        title: "Deployment Log"
        border_color: "BrightYellow"
        content: "Deployment output will appear here..."
        overflow_behavior: "scroll" 
        auto_scroll: true
        
      monitoring:
        bounds: { x1: "70%", y1: "8%", x2: "100%", y2: "100%" }
        title: "Live Monitoring"
        border_color: "BrightRed"
        script: |
          echo "=== Cluster Status ==="
          kubectl cluster-info --context={{kubectl_context}} | head -3
          echo ""
          echo "=== Resource Usage ==="
          kubectl top nodes --context={{kubectl_context}} | head -5
          echo ""
          echo "=== Recent Events ==="
          kubectl get events --context={{kubectl_context}} --sort-by='.lastTimestamp' | tail -5
        refresh_interval: 10000
        thread: true
        streaming: true

active: "main"

# Environment switching
on_keypress:
  "F2": "switch_layout:staging"
  "F3": "switch_layout:production"
```

These examples show different BoxMux features:

- Real-time monitoring with streaming output
- Interactive terminals using PTY integration  
- Multi-layout applications with environment switching
- Data visualization with charts and tables
- Socket-based remote control capabilities
- Scripting with output redirection
- Mouse interactions and keyboard shortcuts

Use these as starting points for your own BoxMux applications.