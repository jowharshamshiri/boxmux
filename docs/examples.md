# BoxMux Examples

This document provides real-world examples and tutorials for creating various types of interfaces with BoxMux.

## Table of Contents

- [Basic Examples](#basic-examples)
- [System Monitoring](#system-monitoring)
- [Development Tools](#development-tools)
- [DevOps Interfaces](#devops-interfaces)
- [Data Visualization](#data-visualization)
- [Interactive Applications](#interactive-applications)
- [Patterns](#patterns)
- [Integration Examples](#integration-examples)

## Basic Examples

### Hello World

The simplest possible BoxMux interface:

```yaml
app:
  layouts:
    - id: 'hello'
      root: true
      children:
        - id: 'greeting'
          title: 'Hello World'
          position:
            x1: 25%
            y1: 40%
            x2: 75%
            y2: 60%
          content: 'Welcome to BoxMux!'
          border: true
```

### Simple Menu

Basic interactive menu:

```yaml
app:
  layouts:
    - id: 'menu'
      root: true
      title: 'Simple Menu'
      children:
        - id: 'options'
          title: 'Choose an option'
          position:
            x1: 20%
            y1: 20%
            x2: 80%
            y2: 80%
          tab_order: '1'
          choices:
            - id: 'option1'
              content: 'Say Hello'
              script:
                - echo 'Hello, World!'
            - id: 'option2'
              content: 'Show Date'
              script:
                - date
            - id: 'option3'
              content: 'List Files'
              script:
                - ls -la
```

### Two-Panel Layout

Split screen with menu and output:

```yaml
app:
  layouts:
    - id: 'split'
      root: true
      title: 'Split View'
      children:
        - id: 'menu'
          title: 'Actions'
          position:
            x1: 5%
            y1: 10%
            x2: 45%
            y2: 90%
          tab_order: '1'
          choices:
            - id: 'uptime'
              content: 'System Uptime'
              script:
                - uptime
              redirect_output: 'output'
            - id: 'whoami'
              content: 'Current User'
              script:
                - whoami
              redirect_output: 'output'
            - id: 'pwd'
              content: 'Current Directory'
              script:
                - pwd
              redirect_output: 'output'
        
        - id: 'output'
          title: 'Output'
          position:
            x1: 55%
            y1: 10%
            x2: 95%
            y2: 90%
          content: 'Select an action from the menu'
```

## System Monitoring

### CPU & Memory Monitor

Real-time system resource monitoring:

```yaml
app:
  layouts:
    - id: 'sysmon'
      root: true
      title: 'System Monitor'
      bg_color: 'black'
      fg_color: 'green'
      title_fg_color: 'bright_white'
      title_bg_color: 'blue'
      children:
        - id: 'cpu'
          title: 'CPU Usage'
          position:
            x1: 5%
            y1: 10%
            x2: 48%
            y2: 40%
          refresh_interval: 2000
          script:
            - top -l 1 | grep "CPU usage" | head -1
            - echo "---"
            - ps -A -o %cpu | awk '{s+=$1} END {print "Total CPU: " s "%"}'
        
        - id: 'memory'
          title: 'Memory Usage'
          position:
            x1: 52%
            y1: 10%
            x2: 95%
            y2: 40%
          refresh_interval: 2000
          script:
            - top -l 1 | grep "PhysMem" | head -1
            - echo "---"
            - vm_stat | grep -E "(free|active|inactive|wired)" | head -4
        
        - id: 'disk'
          title: 'Disk Usage'
          position:
            x1: 5%
            y1: 45%
            x2: 48%
            y2: 75%
          refresh_interval: 5000
          script:
            - df -h | head -6
        
        - id: 'network'
          title: 'Network Stats'
          position:
            x1: 52%
            y1: 45%
            x2: 95%
            y2: 75%
          refresh_interval: 3000
          script:
            - netstat -i | head -5
        
        - id: 'processes'
          title: 'Top Processes'
          position:
            x1: 5%
            y1: 80%
            x2: 95%
            y2: 95%
          refresh_interval: 5000
          script:
            - ps aux | head -10 | awk '{print $2, $3, $4, $11}'
```

### Log Monitor

Real-time log monitoring interface:

```yaml
app:
  layouts:
    - id: 'logmon'
      root: true
      title: 'Log Monitor'
      children:
        - id: 'log_selector'
          title: 'Select Log'
          position:
            x1: 5%
            y1: 10%
            x2: 25%
            y2: 90%
          tab_order: '1'
          choices:
            - id: 'syslog'
              content: 'System Log'
              script:
                - tail -f /var/log/system.log
              redirect_output: 'log_viewer'
              thread: true
            - id: 'install'
              content: 'Install Log'
              script:
                - tail -f /var/log/install.log
              redirect_output: 'log_viewer'
              thread: true
            - id: 'wifi'
              content: 'WiFi Log'
              script:
                - tail -f /var/log/wifi.log
              redirect_output: 'log_viewer'
              thread: true
        
        - id: 'log_viewer'
          title: 'Log Contents'
          position:
            x1: 30%
            y1: 10%
            x2: 95%
            y2: 90%
          content: 'Select a log file to view'
          scroll: true
          overflow_behavior: 'scroll'
```

## Development Tools

### Git Dashboard

Git repository management interface:

```yaml
app:
  layouts:
    - id: 'git_dash'
      root: true
      title: 'Git Dashboard'
      bg_color: 'black'
      fg_color: 'white'
      selected_bg_color: 'blue'
      children:
        - id: 'status'
          title: 'Repository Status'
          position:
            x1: 5%
            y1: 10%
            x2: 95%
            y2: 30%
          refresh_interval: 5000
          script:
            - git status --short
            - echo "---"
            - git log --oneline -5
        
        - id: 'actions'
          title: 'Git Actions'
          position:
            x1: 5%
            y1: 35%
            x2: 45%
            y2: 85%
          tab_order: '1'
          choices:
            - id: 'status_full'
              content: 'Full Status'
              script:
                - git status
              redirect_output: 'output'
            - id: 'diff'
              content: 'Show Diff'
              script:
                - git diff
              redirect_output: 'output'
            - id: 'log'
              content: 'Recent Commits'
              script:
                - git log --oneline -20
              redirect_output: 'output'
            - id: 'branches'
              content: 'List Branches'
              script:
                - git branch -a
              redirect_output: 'output'
            - id: 'remotes'
              content: 'List Remotes'
              script:
                - git remote -v
              redirect_output: 'output'
        
        - id: 'output'
          title: 'Command Output'
          position:
            x1: 50%
            y1: 35%
            x2: 95%
            y2: 85%
          content: 'Select an action from the menu'
          scroll: true
```

### Build Monitor

Continuous integration and build monitoring:

```yaml
app:
  layouts:
    - id: 'build_monitor'
      root: true
      title: 'Build Monitor'
      children:
        - id: 'build_controls'
          title: 'Build Controls'
          position:
            x1: 5%
            y1: 10%
            x2: 30%
            y2: 50%
          tab_order: '1'
          choices:
            - id: 'build'
              content: 'Run Build'
              script:
                - echo 'Starting build...'
                - cargo build
                - echo 'Build completed!'
              redirect_output: 'build_output'
              thread: true
            - id: 'test'
              content: 'Run Tests'
              script:
                - echo 'Running tests...'
                - cargo test
                - echo 'Tests completed!'
              redirect_output: 'build_output'
              thread: true
            - id: 'lint'
              content: 'Run Linter'
              script:
                - echo 'Running linter...'
                - cargo clippy
                - echo 'Linting completed!'
              redirect_output: 'build_output'
              thread: true
        
        - id: 'build_output'
          title: 'Build Output'
          position:
            x1: 35%
            y1: 10%
            x2: 95%
            y2: 70%
          content: 'Select a build action'
          scroll: true
        
        - id: 'build_status'
          title: 'Build Status'
          position:
            x1: 5%
            y1: 55%
            x2: 30%
            y2: 90%
          refresh_interval: 10000
          script:
            - echo "Last build: $(date)"
            - echo "Branch: $(git branch --show-current)"
            - echo "Commit: $(git log --oneline -1)"
        
        - id: 'coverage'
          title: 'Code Coverage'
          position:
            x1: 35%
            y1: 75%
            x2: 95%
            y2: 90%
          refresh_interval: 30000
          script:
            - echo "Coverage stats would go here"
            - echo "Lines: 85% | Branches: 78% | Functions: 92%"
```

## DevOps Interfaces

### Server Management

Server administration dashboard:

```yaml
app:
  layouts:
    - id: 'server_mgmt'
      root: true
      title: 'Server Management'
      bg_color: 'black'
      fg_color: 'cyan'
      children:
        - id: 'server_info'
          title: 'Server Information'
          position:
            x1: 5%
            y1: 10%
            x2: 95%
            y2: 25%
          refresh_interval: 10000
          script:
            - echo "Hostname: $(hostname)"
            - echo "Uptime: $(uptime)"
            - echo "Load: $(uptime | awk -F'load average:' '{print $2}')"
        
        - id: 'services'
          title: 'Services'
          position:
            x1: 5%
            y1: 30%
            x2: 45%
            y2: 70%
          tab_order: '1'
          choices:
            - id: 'nginx_status'
              content: 'Nginx Status'
              script:
                - systemctl status nginx
              redirect_output: 'service_output'
            - id: 'nginx_restart'
              content: 'Restart Nginx'
              script:
                - sudo systemctl restart nginx
                - echo 'Nginx restarted'
              redirect_output: 'service_output'
            - id: 'docker_status'
              content: 'Docker Status'
              script:
                - docker ps
              redirect_output: 'service_output'
            - id: 'docker_images'
              content: 'Docker Images'
              script:
                - docker images
              redirect_output: 'service_output'
        
        - id: 'service_output'
          title: 'Service Output'
          position:
            x1: 50%
            y1: 30%
            x2: 95%
            y2: 70%
          content: 'Select a service action'
          scroll: true
        
        - id: 'logs'
          title: 'System Logs'
          position:
            x1: 5%
            y1: 75%
            x2: 95%
            y2: 90%
          refresh_interval: 5000
          script:
            - tail -5 /var/log/syslog
```

### Deployment Pipeline

Deployment management interface:

```yaml
app:
  layouts:
    - id: 'deploy_pipeline'
      root: true
      title: 'Deployment Pipeline'
      children:
        - id: 'environments'
          title: 'Environments'
          position:
            x1: 5%
            y1: 10%
            x2: 25%
            y2: 60%
          tab_order: '1'
          choices:
            - id: 'deploy_dev'
              content: 'Deploy to Dev'
              script:
                - echo 'Deploying to development...'
                - ./deploy.sh dev
              redirect_output: 'deploy_output'
              thread: true
            - id: 'deploy_staging'
              content: 'Deploy to Staging'
              script:
                - echo 'Deploying to staging...'
                - ./deploy.sh staging
              redirect_output: 'deploy_output'
              thread: true
            - id: 'deploy_prod'
              content: 'Deploy to Production'
              script:
                - echo 'Deploying to production...'
                - ./deploy.sh production
              redirect_output: 'deploy_output'
              thread: true
        
        - id: 'deploy_output'
          title: 'Deployment Output'
          position:
            x1: 30%
            y1: 10%
            x2: 95%
            y2: 60%
          content: 'Select deployment target'
          scroll: true
        
        - id: 'status'
          title: 'Environment Status'
          position:
            x1: 5%
            y1: 65%
            x2: 95%
            y2: 90%
          refresh_interval: 10000
          script:
            - echo "Dev: $(curl -s -o /dev/null -w '%{http_code}' http://dev.example.com)"
            - echo "Staging: $(curl -s -o /dev/null -w '%{http_code}' http://staging.example.com)"
            - echo "Production: $(curl -s -o /dev/null -w '%{http_code}' http://production.example.com)"
```

## Data Visualization

### Metrics Dashboard

ASCII charts and metrics visualization:

```yaml
app:
  layouts:
    - id: 'metrics'
      root: true
      title: 'Metrics Dashboard'
      children:
        - id: 'cpu_chart'
          title: 'CPU Usage Chart'
          position:
            x1: 5%
            y1: 10%
            x2: 48%
            y2: 45%
          refresh_interval: 2000
          script:
            - |
              # Generate CPU usage data
              for i in {1..10}; do
                echo "$i,$((RANDOM % 100))"
              done > /tmp/cpu_data.csv
              
              # Create ASCII chart
              gnuplot -e "
                set terminal dumb size 40,10;
                set style data lines;
                set datafile separator ',';
                plot '/tmp/cpu_data.csv' using 1:2 title 'CPU %' with linespoints;
              "
        
        - id: 'memory_chart'
          title: 'Memory Usage Chart'
          position:
            x1: 52%
            y1: 10%
            x2: 95%
            y2: 45%
          refresh_interval: 2000
          script:
            - |
              # Generate memory usage data
              for i in {1..10}; do
                echo "$i,$((RANDOM % 100 + 20))"
              done > /tmp/mem_data.csv
              
              # Create ASCII chart
              gnuplot -e "
                set terminal dumb size 40,10;
                set style data lines;
                set datafile separator ',';
                plot '/tmp/mem_data.csv' using 1:2 title 'Memory %' with linespoints;
              "
        
        - id: 'network_chart'
          title: 'Network Traffic'
          position:
            x1: 5%
            y1: 50%
            x2: 48%
            y2: 85%
          refresh_interval: 3000
          script:
            - |
              # Generate network data
              for i in {1..15}; do
                echo "$i,$((RANDOM % 1000))"
              done > /tmp/net_data.csv
              
              # Create ASCII chart
              gnuplot -e "
                set terminal dumb size 40,12;
                set style data lines;
                set datafile separator ',';
                plot '/tmp/net_data.csv' using 1:2 title 'KB/s' with linespoints;
              "
        
        - id: 'stats'
          title: 'Quick Stats'
          position:
            x1: 52%
            y1: 50%
            x2: 95%
            y2: 85%
          refresh_interval: 5000
          script:
            - echo "Current Stats:"
            - echo "CPU: $((RANDOM % 100))%"
            - echo "Memory: $((RANDOM % 100))%"
            - echo "Disk: $((RANDOM % 100))%"
            - echo "Network: $((RANDOM % 1000)) KB/s"
            - echo "Processes: $(ps aux | wc -l)"
```

### Database Monitor

Database performance monitoring:

```yaml
app:
  layouts:
    - id: 'db_monitor'
      root: true
      title: 'Database Monitor'
      children:
        - id: 'db_actions'
          title: 'Database Actions'
          position:
            x1: 5%
            y1: 10%
            x2: 30%
            y2: 70%
          tab_order: '1'
          choices:
            - id: 'connections'
              content: 'Active Connections'
              script:
                - psql -c "SELECT count(*) as connections FROM pg_stat_activity;"
              redirect_output: 'db_output'
            - id: 'queries'
              content: 'Running Queries'
              script:
                - psql -c "SELECT query, state, query_start FROM pg_stat_activity WHERE state = 'active';"
              redirect_output: 'db_output'
            - id: 'locks'
              content: 'Table Locks'
              script:
                - psql -c "SELECT schemaname, tablename, attname, n_distinct, correlation FROM pg_stats LIMIT 10;"
              redirect_output: 'db_output'
            - id: 'size'
              content: 'Database Size'
              script:
                - psql -c "SELECT datname, pg_size_pretty(pg_database_size(datname)) FROM pg_database;"
              redirect_output: 'db_output'
        
        - id: 'db_output'
          title: 'Query Results'
          position:
            x1: 35%
            y1: 10%
            x2: 95%
            y2: 70%
          content: 'Select a database query'
          scroll: true
        
        - id: 'db_stats'
          title: 'Database Stats'
          position:
            x1: 5%
            y1: 75%
            x2: 95%
            y2: 90%
          refresh_interval: 10000
          script:
            - echo "DB Status: $(pg_isready && echo 'Online' || echo 'Offline')"
            - echo "Uptime: $(psql -t -c "SELECT date_trunc('second', now() - pg_postmaster_start_time());")"
```

## Interactive Applications

### File Manager

Simple file management interface:

```yaml
app:
  layouts:
    - id: 'file_manager'
      root: true
      title: 'File Manager'
      children:
        - id: 'navigation'
          title: 'Navigation'
          position:
            x1: 5%
            y1: 10%
            x2: 30%
            y2: 70%
          tab_order: '1'
          choices:
            - id: 'list'
              content: 'List Files'
              script:
                - ls -la
              redirect_output: 'file_view'
            - id: 'home'
              content: 'Go Home'
              script:
                - cd ~ && pwd && ls -la
              redirect_output: 'file_view'
            - id: 'parent'
              content: 'Parent Directory'
              script:
                - cd .. && pwd && ls -la
              redirect_output: 'file_view'
            - id: 'disk_usage'
              content: 'Disk Usage'
              script:
                - du -sh * | sort -hr | head -10
              redirect_output: 'file_view'
        
        - id: 'file_view'
          title: 'File Contents'
          position:
            x1: 35%
            y1: 10%
            x2: 95%
            y2: 70%
          content: 'Select a navigation option'
          scroll: true
        
        - id: 'current_path'
          title: 'Current Path'
          position:
            x1: 5%
            y1: 75%
            x2: 95%
            y2: 90%
          refresh_interval: 5000
          script:
            - pwd
            - echo "Files: $(ls -1 | wc -l)"
            - echo "Size: $(du -sh . | cut -f1)"
```

### Process Manager

Process monitoring and management:

```yaml
app:
  layouts:
    - id: 'process_manager'
      root: true
      title: 'Process Manager'
      children:
        - id: 'process_actions'
          title: 'Process Actions'
          position:
            x1: 5%
            y1: 10%
            x2: 30%
            y2: 50%
          tab_order: '1'
          choices:
            - id: 'top_cpu'
              content: 'Top CPU'
              script:
                - ps aux --sort=-%cpu | head -15
              redirect_output: 'process_view'
            - id: 'top_memory'
              content: 'Top Memory'
              script:
                - ps aux --sort=-%mem | head -15
              redirect_output: 'process_view'
            - id: 'all_processes'
              content: 'All Processes'
              script:
                - ps aux
              redirect_output: 'process_view'
            - id: 'process_tree'
              content: 'Process Tree'
              script:
                - pstree
              redirect_output: 'process_view'
        
        - id: 'process_view'
          title: 'Process List'
          position:
            x1: 35%
            y1: 10%
            x2: 95%
            y2: 70%
          content: 'Select a process view'
          scroll: true
        
        - id: 'system_stats'
          title: 'System Stats'
          position:
            x1: 5%
            y1: 55%
            x2: 30%
            y2: 90%
          refresh_interval: 3000
          script:
            - echo "Processes: $(ps aux | wc -l)"
            - echo "Load: $(uptime | awk -F'load average:' '{print $2}')"
            - echo "Memory: $(free -h | grep Mem | awk '{print $3 "/" $2}')"
        
        - id: 'real_time'
          title: 'Real-time Activity'
          position:
            x1: 35%
            y1: 75%
            x2: 95%
            y2: 90%
          refresh_interval: 2000
          script:
            - ps aux --sort=-%cpu | head -5 | tail -4
```

## Patterns

### Multi-layout Application

Application with multiple layouts:

```yaml
app:
  layouts:
    - id: 'main_menu'
      root: true
      title: 'Main Menu'
      children:
        - id: 'menu_choices'
          title: 'Select Application'
          position:
            x1: 25%
            y1: 25%
            x2: 75%
            y2: 75%
          tab_order: '1'
          choices:
            - id: 'sysmon'
              content: 'System Monitor'
              script:
                - echo 'Loading System Monitor...'
                # Switch to system monitor layout
            - id: 'filemanager'
              content: 'File Manager'
              script:
                - echo 'Loading File Manager...'
                # Switch to file manager layout
            - id: 'logs'
              content: 'Log Viewer'
              script:
                - echo 'Loading Log Viewer...'
                # Switch to log viewer layout
    
    - id: 'system_monitor'
      title: 'System Monitor'
      # ... system monitor layout configuration
    
    - id: 'file_manager'
      title: 'File Manager'
      # ... file manager layout configuration
```

### Nested Panel Hierarchy

Complex nested panel structure:

```yaml
app:
  layouts:
    - id: 'complex'
      root: true
      title: 'Complex Layout'
      children:
        - id: 'header'
          title: 'Header'
          position:
            x1: 0%
            y1: 0%
            x2: 100%
            y2: 10%
          children:
            - id: 'logo'
              position:
                x1: 5%
                y1: 20%
                x2: 30%
                y2: 80%
              content: 'MyApp v1.0'
            - id: 'status'
              position:
                x1: 70%
                y1: 20%
                x2: 95%
                y2: 80%
              content: 'Status: Online'
              
        - id: 'main_area'
          position:
            x1: 0%
            y1: 10%
            x2: 100%
            y2: 90%
          children:
            - id: 'sidebar'
              position:
                x1: 0%
                y1: 0%
                x2: 20%
                y2: 100%
              children:
                - id: 'nav_menu'
                  title: 'Navigation'
                  position:
                    x1: 5%
                    y1: 5%
                    x2: 95%
                    y2: 50%
                  # ... navigation menu configuration
                - id: 'quick_stats'
                  title: 'Quick Stats'
                  position:
                    x1: 5%
                    y1: 55%
                    x2: 95%
                    y2: 95%
                  # ... quick stats configuration
                  
            - id: 'content_area'
              position:
                x1: 20%
                y1: 0%
                x2: 100%
                y2: 100%
              children:
                - id: 'main_content'
                  position:
                    x1: 2%
                    y1: 2%
                    x2: 98%
                    y2: 98%
                  # ... main content configuration
```

### Conditional Content

Dynamic content based on conditions:

```yaml
app:
  layouts:
    - id: 'conditional'
      root: true
      title: 'Conditional Interface'
      children:
        - id: 'status_check'
          title: 'System Status'
          position:
            x1: 5%
            y1: 10%
            x2: 95%
            y2: 30%
          refresh_interval: 5000
          script:
            - |
              if systemctl is-active nginx >/dev/null 2>&1; then
                echo "✓ Nginx: Running"
              else
                echo "✗ Nginx: Stopped"
              fi
              
              if systemctl is-active postgresql >/dev/null 2>&1; then
                echo "✓ PostgreSQL: Running"
              else
                echo "✗ PostgreSQL: Stopped"
              fi
              
              if docker ps >/dev/null 2>&1; then
                echo "✓ Docker: Running"
              else
                echo "✗ Docker: Stopped"
              fi
        
        - id: 'conditional_actions'
          title: 'Available Actions'
          position:
            x1: 5%
            y1: 35%
            x2: 95%
            y2: 80%
          script:
            - |
              echo "Available actions based on system state:"
              if systemctl is-active nginx >/dev/null 2>&1; then
                echo "• Restart Nginx"
                echo "• View Nginx logs"
              else
                echo "• Start Nginx"
              fi
              
              if systemctl is-active postgresql >/dev/null 2>&1; then
                echo "• Database backup"
                echo "• View DB connections"
              else
                echo "• Start PostgreSQL"
              fi
```

## Integration Examples

### API Integration

Interface that consumes REST APIs:

```yaml
app:
  layouts:
    - id: 'api_dashboard'
      root: true
      title: 'API Dashboard'
      children:
        - id: 'api_calls'
          title: 'API Calls'
          position:
            x1: 5%
            y1: 10%
            x2: 30%
            y2: 70%
          tab_order: '1'
          choices:
            - id: 'weather'
              content: 'Weather Data'
              script:
                - curl -s "https://api.openweathermap.org/data/2.5/weather?q=London&appid=YOUR_API_KEY" | jq .
              redirect_output: 'api_output'
            - id: 'github'
              content: 'GitHub Status'
              script:
                - curl -s "https://api.github.com/repos/octocat/Hello-World" | jq '.name, .description, .stargazers_count'
              redirect_output: 'api_output'
            - id: 'system_time'
              content: 'World Clock'
              script:
                - curl -s "http://worldtimeapi.org/api/timezone/Europe/London" | jq '.datetime'
              redirect_output: 'api_output'
        
        - id: 'api_output'
          title: 'API Response'
          position:
            x1: 35%
            y1: 10%
            x2: 95%
            y2: 70%
          content: 'Select an API call'
          scroll: true
```

### Docker Integration

Docker container management:

```yaml
app:
  layouts:
    - id: 'docker_mgmt'
      root: true
      title: 'Docker Management'
      children:
        - id: 'docker_actions'
          title: 'Docker Actions'
          position:
            x1: 5%
            y1: 10%
            x2: 30%
            y2: 80%
          tab_order: '1'
          choices:
            - id: 'containers'
              content: 'List Containers'
              script:
                - docker ps -a
              redirect_output: 'docker_output'
            - id: 'images'
              content: 'List Images'
              script:
                - docker images
              redirect_output: 'docker_output'
            - id: 'stats'
              content: 'Container Stats'
              script:
                - docker stats --no-stream
              redirect_output: 'docker_output'
            - id: 'logs'
              content: 'Container Logs'
              script:
                - docker logs $(docker ps -q | head -1)
              redirect_output: 'docker_output'
        
        - id: 'docker_output'
          title: 'Docker Output'
          position:
            x1: 35%
            y1: 10%
            x2: 95%
            y2: 80%
          content: 'Select a Docker action'
          scroll: true
```

These examples demonstrate the flexibility and power of BoxMux for creating various types of terminal interfaces. You can mix and match these patterns to create your own custom applications.

For more usage, see the [API Reference](api.md).
