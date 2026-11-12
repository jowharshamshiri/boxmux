---
title: Configuration Examples
description: Complete BoxMux configurations for monitoring, development, server administration, containers, data display and multi-layout control panels
---


Each example below is a complete configuration: save it to a file and run `boxmux <file>`. The scripts are written for `bash` on Linux; on macOS some commands (`free`, `ps --sort`, `ss`) differ, and on Windows scripts run in PowerShell unless the app sets `shell: bash`.

The repository has more in `layouts/` (for example `showcase_menu.yaml`, `devops_control_center.yaml`, `chart_demo.yaml`, `table_demo.yaml`, `plugin_demo.yaml`) and `examples/` (`pty_states.yaml`, `ansi_colors.yaml`, and `live-panel/`, a panel driven by another program, described in [Drive BoxMux as a Live Control Panel](/docs/live-control-panel)).

## System Monitoring Dashboard

Live system statistics, a process table, and a menu whose output goes to a log box.

```yaml
# system_monitor.yaml
app:
  layouts:
    - id: 'main'
      root: true
      title: 'System Monitor - ${HOSTNAME:this machine}'
      children:
        - id: 'header'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 12%}
          border_color: 'bright_cyan'
          refresh_interval: 1000
          script:
            - echo "$(hostname) | user $(whoami) | $(date '+%Y-%m-%d %H:%M:%S')"

        - id: 'cpu_stats'
          title: 'Load'
          position: {x1: 0%, y1: 12%, x2: 33%, y2: 55%}
          border_color: 'bright_green'
          chart_type: bar
          refresh_interval: 2000
          script:
            - awk '{print "1 min," $1; print "5 min," $2; print "15 min," $3}' /proc/loadavg

        - id: 'memory_stats'
          title: 'Memory'
          position: {x1: 33%, y1: 12%, x2: 66%, y2: 55%}
          border_color: 'bright_blue'
          refresh_interval: 2000
          script:
            - free -h

        - id: 'disk_stats'
          title: 'Disks'
          position: {x1: 66%, y1: 12%, x2: 100%, y2: 55%}
          border_color: 'bright_magenta'
          refresh_interval: 10000
          script:
            - df -h | grep -E '^(Filesystem|/dev/)'

        - id: 'control_panel'
          title: 'Actions'
          position: {x1: 0%, y1: 55%, x2: 30%, y2: 100%}
          border_color: 'bright_yellow'
          tab_order: 1
          choices:
            - id: 'system_info'
              content: 'System Info'
              script:
                - 'echo "System: $(uname -a)"'
                - 'echo "Uptime: $(uptime)"'
              redirect_output: 'log_output'
            - id: 'top_memory'
              content: 'Top Memory Users'
              script:
                - ps -eo pid,pmem,comm --sort=-pmem | head -n 10
              redirect_output: 'log_output'
            - id: 'shell'
              content: 'Interactive Shell'
              execution_mode: Pty
              script: ['bash']
              redirect_output: 'log_output'

        - id: 'log_output'
          title: 'Output'
          position: {x1: 30%, y1: 55%, x2: 100%, y2: 100%}
          border_color: 'bright_white'
          content: 'Action output will appear here'
          auto_scroll_bottom: true
```

Notes:

- `${HOSTNAME:this machine}` in the layout title uses the `HOSTNAME` environment variable if BoxMux can see it (shells often do not export it) and the text after the colon otherwise. The header box asks `hostname` at run time instead.
- The load box is a bar chart drawn from its script's `label,value` output.
- "Interactive Shell" opens a terminal tab in the output box. Focus the box to type into it; close the tab (`×`) to end the shell.

## Development Dashboard

Project status, build and test commands with streamed output, and hot keys.

```yaml
# dev_dashboard.yaml
app:
  variables:
    PROJECT_NAME: "My Project"
  hot_keys:
    F5: 'build'
    F6: 'test'
  layouts:
    - id: 'main'
      root: true
      title: '${PROJECT_NAME}'
      children:
        - id: 'header'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 10%}
          border_color: 'bright_cyan'
          refresh_interval: 5000
          script:
            - echo "${PROJECT_NAME} - branch $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo 'none')"

        - id: 'git_status'
          title: 'Git Status'
          position: {x1: 0%, y1: 10%, x2: 30%, y2: 50%}
          border_color: 'bright_green'
          refresh_interval: 3000
          script:
            - git status --porcelain | head -n 20
            - echo "---"
            - git log --oneline -5

        - id: 'actions'
          title: 'Actions (F5 build, F6 test)'
          position: {x1: 30%, y1: 10%, x2: 60%, y2: 50%}
          border_color: 'bright_yellow'
          tab_order: 1
          choices:
            - id: 'build'
              content: 'Build'
              script: ['npm run build 2>&1']
              execution_mode: Thread
              redirect_output: 'build_log'
            - id: 'test'
              content: 'Run Tests'
              script: ['npm test 2>&1']
              execution_mode: Thread
              redirect_output: 'test_log'
            - id: 'install'
              content: 'Install Dependencies'
              script: ['npm install 2>&1']
              execution_mode: Thread
              redirect_output: 'build_log'
            - id: 'dev_server'
              content: 'Start Dev Server'
              script: ['npm run dev']
              execution_mode: Pty
              redirect_output: 'test_log'

        - id: 'package_info'
          title: 'Package'
          position: {x1: 60%, y1: 10%, x2: 100%, y2: 50%}
          border_color: 'bright_blue'
          refresh_interval: 10000
          script:
            - |
              if [ -f package.json ]; then
                echo "Dependencies:"
                jq -r '.dependencies // {} | keys[]' package.json | head -n 10
                echo ""
                echo "Scripts:"
                jq -r '.scripts // {} | keys[]' package.json
              else
                echo "No package.json in $(pwd)"
              fi

        - id: 'build_log'
          title: 'Build Output'
          position: {x1: 0%, y1: 50%, x2: 50%, y2: 90%}
          border_color: 'bright_magenta'
          content: 'Build output will appear here'
          auto_scroll_bottom: true

        - id: 'test_log'
          title: 'Test Results'
          position: {x1: 50%, y1: 50%, x2: 100%, y2: 90%}
          border_color: 'bright_red'
          content: 'Test results will appear here'
          auto_scroll_bottom: true

        - id: 'footer'
          position: {x1: 0%, y1: 90%, x2: 100%, y2: 100%}
          border_color: 'bright_black'
          refresh_interval: 60000
          script:
            - echo "Node $(node --version 2>/dev/null || echo 'not found') | npm $(npm --version 2>/dev/null || echo 'not found')"
```

Notes:

- Scripts run in the directory BoxMux was started from, so start it in the project directory.
- `execution_mode: Thread` shows build and test output line by line. A failing build (non-zero exit status) turns the output box's border red.
- `hot_keys` maps F5 and F6 to the choices `build` and `test`; they work whichever box has the focus, unless that box is showing a running terminal (which receives the keys).

## Server Management Interface

Read-only checks in one area, and interactive tools (`htop`, a file manager, `ssh`) in terminal tabs.

```yaml
# server_admin.yaml
app:
  variables:
    SERVER: "192.168.1.100"
    ADMIN_USER: "admin"
  layouts:
    - id: 'main'
      root: true
      title: 'Server Admin - ${SERVER}'
      children:
        - id: 'server_menu'
          title: 'Server Actions'
          position: {x1: 0%, y1: 0%, x2: 25%, y2: 100%}
          border_color: 'bright_green'
          tab_order: 1
          choices:
            - id: 'status'
              content: 'System Status'
              script:
                - uptime
                - free -h
                - df -h
              redirect_output: 'main_output'
            - id: 'process_monitor'
              content: 'Process Monitor'
              execution_mode: Pty
              script: ['htop']
              redirect_output: 'terminal_area'
            - id: 'file_manager'
              content: 'File Manager'
              execution_mode: Pty
              script: ['ranger']
              redirect_output: 'terminal_area'
            - id: 'network'
              content: 'Listening Ports'
              script: ['ss -tuln']
              redirect_output: 'main_output'
            - id: 'ssh_remote'
              content: 'SSH to ${SERVER}'
              execution_mode: Pty
              script: ['ssh ${ADMIN_USER}@${SERVER}']
              redirect_output: 'terminal_area'
            - id: 'services'
              content: 'Failed Services'
              script: ['systemctl --failed --no-pager']
              redirect_output: 'main_output'

        - id: 'main_output'
          title: 'Command Output'
          position: {x1: 25%, y1: 0%, x2: 60%, y2: 100%}
          border_color: 'bright_blue'
          content: 'Select an action to see its output'

        - id: 'terminal_area'
          title: 'Terminal'
          position: {x1: 60%, y1: 0%, x2: 100%, y2: 80%}
          border_color: 'bright_cyan'
          tab_order: 2
          execution_mode: Pty
          script: ['bash']

        - id: 'system_info'
          title: 'Live Stats'
          position: {x1: 60%, y1: 80%, x2: 100%, y2: 100%}
          border_color: 'bright_yellow'
          refresh_interval: 2000
          script:
            - |
              echo "Load: $(cut -d' ' -f1-3 /proc/loadavg)"
              echo "Memory: $(free | awk '/^Mem:/ {printf "%.1f%%", $3/$2*100}')"
              echo "Disk /: $(df / | awk 'NR==2 {print $5}')"
```

Notes:

- `terminal_area` starts with a shell. The tools started from the menu open as further tabs in it; each tab is labelled with the choice (or with the title the program sets).
- Tab moves the focus between the menu and the terminal; inside the terminal every other key, including `q`, goes to the program.

## Docker Container Manager

Container list, actions, Compose commands and a log stream.

```yaml
# docker_manager.yaml
app:
  layouts:
    - id: 'main'
      root: true
      title: 'Docker Manager'
      children:
        - id: 'header'
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 10%}
          border_color: 'bright_blue'
          content: 'Docker Manager - host ${DOCKER_HOST:local socket}'

        - id: 'container_list'
          title: 'Running Containers'
          position: {x1: 0%, y1: 10%, x2: 40%, y2: 100%}
          border_color: 'bright_green'
          refresh_interval: 3000
          script:
            - docker ps --format 'table {{.Names}}\t{{.Status}}\t{{.Ports}}'

        - id: 'container_actions'
          title: 'Containers'
          position: {x1: 40%, y1: 10%, x2: 60%, y2: 55%}
          border_color: 'bright_yellow'
          tab_order: 1
          choices:
            - id: 'list_all'
              content: 'List All'
              script: ['docker ps -a']
              redirect_output: 'docker_output'
            - id: 'stats'
              content: 'Stats'
              script: ['docker stats --no-stream']
              redirect_output: 'docker_output'
            - id: 'images'
              content: 'Images'
              script: ['docker images']
              redirect_output: 'docker_output'
            - id: 'prune'
              content: 'Clean Up'
              script:
                - docker system prune -f 2>&1
                - docker image prune -f 2>&1
              execution_mode: Thread
              redirect_output: 'docker_output'
            - id: 'disk'
              content: 'Disk Usage'
              script: ['docker system df']
              redirect_output: 'docker_output'

        - id: 'docker_compose'
          title: 'Compose'
          position: {x1: 40%, y1: 55%, x2: 60%, y2: 100%}
          border_color: 'bright_magenta'
          tab_order: 2
          choices:
            - id: 'compose_up'
              content: 'Up'
              script: ['docker compose up -d 2>&1']
              execution_mode: Thread
              redirect_output: 'docker_output'
            - id: 'compose_down'
              content: 'Down'
              script: ['docker compose down 2>&1']
              redirect_output: 'docker_output'
            - id: 'compose_ps'
              content: 'Status'
              script: ['docker compose ps']
              redirect_output: 'docker_output'
            - id: 'compose_logs'
              content: 'Follow Logs'
              script: ['docker compose logs --tail=50 -f']
              execution_mode: Thread
              redirect_output: 'docker_output'

        - id: 'docker_output'
          title: 'Docker Output'
          position: {x1: 60%, y1: 10%, x2: 100%, y2: 100%}
          border_color: 'bright_white'
          content: 'Command output will appear here'
          auto_scroll_bottom: true
```

Notes:

- `{{.Names}}` is Docker's template syntax; BoxMux only substitutes `${...}`, so it reaches Docker unchanged.
- "Follow Logs" keeps running in its own tab; close the tab to stop `docker compose logs`.
- Choice ids are unique across the whole configuration, which is why the Compose entries are prefixed.

## Data Visualization Dashboard

Charts and a table from static data, and a chart fed by a script.

```yaml
# data_dashboard.yaml
app:
  layouts:
    - id: 'main'
      root: true
      title: 'Data Visualization'
      children:
        - id: 'cpu_chart'
          title: 'CPU Usage Over Time (%)'
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 45%}
          border_color: 'bright_green'
          chart_type: line
          chart_data: |
            10s,45
            20s,52
            30s,48
            40s,65
            50s,71
            60s,58
            70s,62
            80s,69

        - id: 'memory_chart'
          title: 'Memory by Application (GB)'
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 45%}
          border_color: 'bright_blue'
          chart_type: bar
          chart_data: |
            App1,2.1
            App2,3.4
            App3,2.8
            App4,4.1
            App5,3.2

        - id: 'process_table'
          title: 'Processes'
          position: {x1: 0%, y1: 45%, x2: 60%, y2: 100%}
          border_color: 'bright_magenta'
          table_data: |
            Process,PID,CPU,Memory,Status
            nginx,1234,2.3,45MB,Running
            postgres,5678,15.7,256MB,Running
            redis,9012,0.8,12MB,Running
            node,3456,8.4,128MB,Running
            docker,7890,3.2,89MB,Running
          table_config:
            sort_column: 'CPU'
            sort_ascending: false
            zebra_striping: true
            border_style: rounded

        - id: 'disk_chart'
          title: 'Disk Use by Mount (%)'
          position: {x1: 60%, y1: 45%, x2: 100%, y2: 100%}
          border_color: 'bright_yellow'
          chart_type: bar
          refresh_interval: 30000
          script:
            - df -P | awk 'NR>1 && $1 ~ /^\/dev\// {gsub("%", "", $5); print $6 "," $5}'
```

Notes:

- `chart_data` and `table_data` are text. Charts take one `label,value` (or `label:value`, `label value`) per line; tables take CSV with a header row, or a JSON array of objects.
- Without `chart_data`, the chart is drawn from the output of the box's script, which must print the same format.

## Multi-Layout DevOps Control

Two layouts, switched with F2 and F3 through the control socket.

```yaml
# devops_control.yaml
app:
  variables:
    KUBE_CONTEXT: "prod-cluster"
  on_keypress:
    F2: ['boxmux switch_active_layout cluster']
    F3: ['boxmux switch_active_layout deploy']
  layouts:
    - id: 'cluster'
      root: true
      title: 'Cluster - ${KUBE_CONTEXT} (F3: deployments)'
      children:
        - id: 'kubernetes_menu'
          title: 'Kubernetes'
          position: {x1: 0%, y1: 0%, x2: 25%, y2: 100%}
          border_color: 'bright_blue'
          tab_order: 1
          choices:
            - id: 'pods'
              content: 'Pods'
              script: ['kubectl --context=${KUBE_CONTEXT} get pods -A']
              redirect_output: 'cluster_output'
            - id: 'services'
              content: 'Services'
              script: ['kubectl --context=${KUBE_CONTEXT} get svc -A']
              redirect_output: 'cluster_output'
            - id: 'top_nodes'
              content: 'Top Nodes'
              script: ['kubectl --context=${KUBE_CONTEXT} top nodes']
              redirect_output: 'cluster_output'
            - id: 'events'
              content: 'Events'
              script: ["kubectl --context=${KUBE_CONTEXT} get events -A --sort-by='.lastTimestamp'"]
              redirect_output: 'cluster_output'

        - id: 'cluster_output'
          title: 'Command Output'
          position: {x1: 25%, y1: 0%, x2: 70%, y2: 100%}
          content: 'Select a command'

        - id: 'monitoring'
          title: 'Live Monitoring'
          position: {x1: 70%, y1: 0%, x2: 100%, y2: 100%}
          border_color: 'bright_red'
          refresh_interval: 10000
          script:
            - echo "=== Cluster ==="
            - kubectl --context=${KUBE_CONTEXT} cluster-info 2>&1 | head -n 3
            - echo ""
            - echo "=== Nodes ==="
            - kubectl --context=${KUBE_CONTEXT} top nodes 2>&1 | head -n 5

    - id: 'deploy'
      title: 'Deployments - ${KUBE_CONTEXT} (F2: cluster)'
      children:
        - id: 'deployment_menu'
          title: 'Deployments'
          position: {x1: 0%, y1: 0%, x2: 25%, y2: 100%}
          border_color: 'bright_green'
          tab_order: 1
          choices:
            - id: 'restart'
              content: 'Rolling Restart'
              script: ['kubectl --context=${KUBE_CONTEXT} rollout restart deployment/app 2>&1']
              redirect_output: 'deployment_log'
            - id: 'rollout_status'
              content: 'Rollout Status'
              script: ['kubectl --context=${KUBE_CONTEXT} rollout status deployment/app 2>&1']
              execution_mode: Thread
              redirect_output: 'deployment_log'
            - id: 'rollback'
              content: 'Roll Back'
              script: ['kubectl --context=${KUBE_CONTEXT} rollout undo deployment/app 2>&1']
              redirect_output: 'deployment_log'

        - id: 'deployment_log'
          title: 'Deployment Log'
          position: {x1: 25%, y1: 0%, x2: 100%, y2: 100%}
          border_color: 'bright_yellow'
          content: 'Deployment output will appear here'
          auto_scroll_bottom: true
```

Notes:

- App-level `on_keypress` handlers run the `boxmux` command in the background. Scripts started by BoxMux have `BOXMUX_SOCKET` set, so `boxmux switch_active_layout` reaches this instance. The `boxmux` binary must be on `PATH`.
- A layout's box scripts start the first time the layout is shown.
- The layout `title` is also the terminal window title.

## What These Examples Use

- Live output with `refresh_interval`, and streamed output with `execution_mode: Thread`
- Interactive programs in terminal tabs with `execution_mode: Pty`
- Output redirection to other boxes with `redirect_output`
- Charts (`chart_type`, `chart_data`) and tables (`table_data`, `table_config`)
- Variables with defaults from the environment (`${NAME:default}`)
- Keyboard shortcuts with `hot_keys` and `on_keypress`
- Layout switching through the control socket

See the [User Guide](/docs/user-guide) for how each feature works and the [Configuration Reference](/docs/configuration) for every setting.
