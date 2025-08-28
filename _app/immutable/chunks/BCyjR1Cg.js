import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as h,s,a as x,b as v,d as n,r as a,n as f}from"./E9AOERj2.js";import{h as t}from"./Cc2oKZLy.js";const i={title:"Configuration Examples",description:"BoxMux configuration examples for various use cases and application types"},{title:A,description:P}=i;var B=h('<h2>System Monitoring Dashboard</h2> <p>A system monitoring application with real-time stats and interactive controls.</p> <pre class="language-yaml"><!></pre> <h2>Development Dashboard</h2> <p>A developer-focused dashboard for project management, testing, and deployment.</p> <pre class="language-yaml"><!></pre> <h2>Server Management Interface</h2> <p>A server administration interface with PTY terminals and monitoring.</p> <pre class="language-yaml"><!></pre> <h2>Docker Container Manager</h2> <p>A Docker management interface with container controls and log monitoring.</p> <pre class="language-yaml"><!></pre> <h2>Data Visualization Dashboard</h2> <p>A dashboard showcasing BoxMux’s data visualization capabilities with charts and tables.</p> <pre class="language-yaml"><!></pre> <h2>Multi-Environment DevOps Control</h2> <p>A DevOps control panel for managing multiple environments.</p> <pre class="language-yaml"><!></pre> <p>These examples show different BoxMux features:</p> <ul><li>Real-time monitoring with streaming output</li> <li>Interactive terminals using PTY integration</li> <li>Multi-layout applications with environment switching</li> <li>Data visualization with charts and tables</li> <li>Socket-based remote control capabilities</li> <li>Scripting with output redirection</li> <li>Mouse interactions and keyboard shortcuts</li></ul> <p>Use these as starting points for your own BoxMux applications.</p>',1);function w(r){var u=B(),p=s(x(u),4),y=n(p);t(y,()=>`<code class="language-yaml"><span class="token comment"># system_monitor.yaml</span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"System Monitor Dashboard"</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">50</span>

<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">hostname</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;HOSTNAME&#125;&#125;"</span>
  <span class="token key atrule">user</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;USER&#125;&#125;"</span>

<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">main</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">header</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"15%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"📊 &#123;&#123;hostname&#125;&#125; System Monitor - User: &#123;&#123;user&#125;&#125;"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
        <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
        
      <span class="token key atrule">cpu_stats</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"15%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"33%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"60%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"CPU Usage"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
          #!/bin/bash
          while true; do
            top -l1 | grep "CPU usage" | awk '&#123;print $3, $5&#125;'
            sleep 1
          done</span>
        <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">1000</span>
        <span class="token key atrule">thread</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        
      <span class="token key atrule">memory_stats</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"33%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"15%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"66%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"60%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Memory Usage"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span> 
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
          #!/bin/bash
          vm_stat | grep -E "(Pages free|Pages active|Pages wired)" | 
          awk '&#123;printf "%s: %d MB&#92;n", $1$2, $3*4/1024&#125;'</span>
        <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>
        <span class="token key atrule">thread</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        
      <span class="token key atrule">disk_stats</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"66%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"15%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"60%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Disk Usage"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightMagenta"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"df -h | grep -E '^/dev/'"</span>
        <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">5000</span>
        
      <span class="token key atrule">control_panel</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"60%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Actions"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
        <span class="token key atrule">choices</span><span class="token punctuation">:</span>
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🔄 Refresh All Stats"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"echo 'Refreshing all statistics...'"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"log_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📋 Copy System Info"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
              echo "System: $(uname -a)"
              echo "Uptime: $(uptime)"
              echo "Load: $(uptime | awk '&#123;print $NF&#125;')"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"log_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"⚡ Interactive Terminal"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"bash"</span>
            <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"terminal_box"</span>
            
      <span class="token key atrule">log_output</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"60%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"85%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Output Log"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Action output will appear here..."</span>
        <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        
      <span class="token key atrule">status_bar</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"85%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlack"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"date '+Last Updated: %Y-%m-%d %H:%M:%S'"</span>
        <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">1000</span>

<span class="token key atrule">active</span><span class="token punctuation">:</span> <span class="token string">"main"</span>

<span class="token comment"># Global key bindings</span>
<span class="token key atrule">on_keypress</span><span class="token punctuation">:</span>
  <span class="token key atrule">"Ctrl+r"</span><span class="token punctuation">:</span> <span class="token string">"refresh_all"</span>
  <span class="token key atrule">"Ctrl+q"</span><span class="token punctuation">:</span> <span class="token string">"quit"</span>
  <span class="token key atrule">"F1"</span><span class="token punctuation">:</span> <span class="token string">"show_help"</span></code>`),a(p);var e=s(p,6),g=n(e);t(g,()=>`<code class="language-yaml"><span class="token comment"># dev_dashboard.yaml</span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Development Dashboard"</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">100</span>

<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">project_name</span><span class="token punctuation">:</span> <span class="token string">"My Awesome Project"</span>
  <span class="token key atrule">git_branch</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;GIT_BRANCH&#125;&#125;"</span>
  <span class="token key atrule">node_env</span><span class="token punctuation">:</span> <span class="token string">"development"</span>

<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">main</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">header</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"10%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"🚀 &#123;&#123;project_name&#125;&#125; - Branch: &#123;&#123;git_branch&#125;&#125;"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
        
      <span class="token key atrule">git_status</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"30%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Git Status"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
          git status --porcelain | head -20
          echo "---"
          git log --oneline -5</span>
        <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">3000</span>
        
      <span class="token key atrule">actions</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"30%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"60%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Quick Actions"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
        <span class="token key atrule">choices</span><span class="token punctuation">:</span>
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🔨 Build Project"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"npm run build"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"build_log"</span>
            <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">thread</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🧪 Run Tests"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"npm test"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"test_log"</span>
            <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📦 Install Dependencies"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"npm install"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"build_log"</span>
            <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🌐 Start Dev Server"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"npm run dev"</span>
            <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📝 Open Editor"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"code ."</span>
            
      <span class="token key atrule">package_info</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"60%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Package Info"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
          echo "Dependencies:"
          cat package.json | jq -r '.dependencies | keys[]' | head -10
          echo ""
          echo "Scripts:"
          cat package.json | jq -r '.scripts | keys[]'</span>
        <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">10000</span>
        
      <span class="token key atrule">build_log</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"85%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Build Output"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightMagenta"</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Build output will appear here..."</span>
        <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>
        <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        
      <span class="token key atrule">test_log</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"85%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Test Results"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightRed"</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Test results will appear here..."</span>
        <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>
        <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        
      <span class="token key atrule">footer</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"85%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlack"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
          echo "Node: $(node --version) | NPM: $(npm --version) | $(date)"</span>

<span class="token key atrule">active</span><span class="token punctuation">:</span> <span class="token string">"main"</span>

<span class="token comment"># Hot keys for common actions</span>
<span class="token key atrule">hot_keys</span><span class="token punctuation">:</span>
  <span class="token key atrule">"Ctrl+b"</span><span class="token punctuation">:</span> 
    <span class="token key atrule">choice_id</span><span class="token punctuation">:</span> <span class="token string">"build"</span>
    <span class="token key atrule">box_id</span><span class="token punctuation">:</span> <span class="token string">"actions"</span>
  <span class="token key atrule">"Ctrl+t"</span><span class="token punctuation">:</span>
    <span class="token key atrule">choice_id</span><span class="token punctuation">:</span> <span class="token string">"test"</span>
    <span class="token key atrule">box_id</span><span class="token punctuation">:</span> <span class="token string">"actions"</span></code>`),a(e);var o=s(e,6),m=n(o);t(m,()=>`<code class="language-yaml"><span class="token comment"># server_admin.yaml  </span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Server Administration"</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">75</span>

<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">server_ip</span><span class="token punctuation">:</span> <span class="token string">"192.168.1.100"</span>
  <span class="token key atrule">admin_user</span><span class="token punctuation">:</span> <span class="token string">"admin"</span>

<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">main</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">title_bar</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"8%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"🖥️  Server Admin Panel - &#123;&#123;server_ip&#125;&#125;"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
        
      <span class="token key atrule">server_menu</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"8%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"25%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Server Actions"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
        <span class="token key atrule">choices</span><span class="token punctuation">:</span>
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📊 System Status"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
              uptime
              free -h
              df -h</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"main_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🔍 Process Monitor"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"htop"</span>
            <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"terminal_area"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📁 File Manager"</span> 
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"ranger"</span>
            <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"terminal_area"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🌐 Network Status"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
              netstat -tulpn | head -20
              echo "---"
              ss -tuln</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"main_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🔐 SSH to Remote"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"ssh &#123;&#123;admin_user&#125;&#125;@&#123;&#123;server_ip&#125;&#125;"</span>
            <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"terminal_area"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📋 Service Status"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"systemctl status"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"main_output"</span>
            
      <span class="token key atrule">main_output</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"25%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"8%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"65%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Command Output"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Select an action to see output..."</span>
        <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>
        <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        
      <span class="token key atrule">terminal_area</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"65%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"8%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"80%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"⚡ Interactive Terminal"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
        <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"bash"</span>
        
      <span class="token key atrule">system_info</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"65%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"80%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Live Stats"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
          echo "Load: $(uptime | awk '&#123;print $NF&#125;')"
          echo "Memory: $(free | awk '/^Mem:/&#123;printf "%.1f%%", $3/$2*100&#125;')"
          echo "Disk: $(df / | awk 'NR==2&#123;print $5&#125;')"</span>
        <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>
        <span class="token key atrule">thread</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>

<span class="token key atrule">active</span><span class="token punctuation">:</span> <span class="token string">"main"</span></code>`),a(o);var c=s(o,6),d=n(c);t(d,()=>`<code class="language-yaml"><span class="token comment"># docker_manager.yaml</span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Docker Container Manager"</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">100</span>

<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">docker_host</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;DOCKER_HOST&#125;&#125;"</span>

<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">main</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">header</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"10%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"🐳 Docker Manager - Host: &#123;&#123;docker_host&#125;&#125;"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
        
      <span class="token key atrule">container_list</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"40%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Running Containers"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"docker ps --format 'table &#123;&#123;.Names&#125;&#125;&#92;t&#123;&#123;.Status&#125;&#125;&#92;t&#123;&#123;.Ports&#125;&#125;'"</span>
        <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">3000</span>
        <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>
        
      <span class="token key atrule">container_actions</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"40%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"60%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"60%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Container Actions"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
        <span class="token key atrule">choices</span><span class="token punctuation">:</span>
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📋 List All Containers"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"docker ps -a"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"docker_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🔍 Container Stats"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"docker stats --no-stream"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"docker_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🗄️ List Images"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"docker images"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"docker_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🧹 Clean Up"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
              docker system prune -f
              docker image prune -f</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"docker_output"</span>
            <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📊 System Info"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"docker system df"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"docker_output"</span>
            
      <span class="token key atrule">docker_compose</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"40%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"60%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"60%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Docker Compose"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightMagenta"</span>
        <span class="token key atrule">choices</span><span class="token punctuation">:</span>
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🚀 Compose Up"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"docker-compose up -d"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"docker_output"</span>
            <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"⏹️ Compose Down"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"docker-compose down"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"docker_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📋 Compose Status"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"docker-compose ps"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"docker_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📄 View Logs"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"docker-compose logs --tail=50 -f"</span>
            <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"docker_output"</span>
            
      <span class="token key atrule">docker_output</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"60%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Docker Output"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Docker command output will appear here..."</span>
        <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>
        <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>

<span class="token key atrule">active</span><span class="token punctuation">:</span> <span class="token string">"main"</span>

<span class="token comment"># Quick docker commands via hot keys</span>
<span class="token key atrule">hot_keys</span><span class="token punctuation">:</span>
  <span class="token key atrule">"Ctrl+l"</span><span class="token punctuation">:</span> 
    <span class="token key atrule">choice_id</span><span class="token punctuation">:</span> <span class="token string">"list_containers"</span>
    <span class="token key atrule">box_id</span><span class="token punctuation">:</span> <span class="token string">"container_actions"</span>
  <span class="token key atrule">"Ctrl+s"</span><span class="token punctuation">:</span>
    <span class="token key atrule">choice_id</span><span class="token punctuation">:</span> <span class="token string">"stats"</span>
    <span class="token key atrule">box_id</span><span class="token punctuation">:</span> <span class="token string">"container_actions"</span></code>`),a(c);var l=s(c,6),b=n(l);t(b,()=>`<code class="language-yaml"><span class="token comment"># data_dashboard.yaml</span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Data Visualization Dashboard"</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">200</span>

<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">data_source</span><span class="token punctuation">:</span> <span class="token string">"/tmp/metrics.json"</span>

<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">main</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"10%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"📈 Data Visualization Dashboard"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
        
      <span class="token key atrule">cpu_chart</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"CPU Usage Trend"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
        <span class="token key atrule">chart</span><span class="token punctuation">:</span>
          <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"line"</span>
          <span class="token key atrule">data</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token number">45</span><span class="token punctuation">,</span> <span class="token number">52</span><span class="token punctuation">,</span> <span class="token number">48</span><span class="token punctuation">,</span> <span class="token number">65</span><span class="token punctuation">,</span> <span class="token number">71</span><span class="token punctuation">,</span> <span class="token number">58</span><span class="token punctuation">,</span> <span class="token number">62</span><span class="token punctuation">,</span> <span class="token number">69</span><span class="token punctuation">,</span> <span class="token number">73</span><span class="token punctuation">,</span> <span class="token number">67</span><span class="token punctuation">]</span>
          <span class="token key atrule">labels</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">"10s"</span><span class="token punctuation">,</span> <span class="token string">"20s"</span><span class="token punctuation">,</span> <span class="token string">"30s"</span><span class="token punctuation">,</span> <span class="token string">"40s"</span><span class="token punctuation">,</span> <span class="token string">"50s"</span><span class="token punctuation">,</span> <span class="token string">"60s"</span><span class="token punctuation">,</span> <span class="token string">"70s"</span><span class="token punctuation">,</span> <span class="token string">"80s"</span><span class="token punctuation">,</span> <span class="token string">"90s"</span><span class="token punctuation">,</span> <span class="token string">"100s"</span><span class="token punctuation">]</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"CPU Usage Over Time"</span>
          <span class="token key atrule">y_axis_label</span><span class="token punctuation">:</span> <span class="token string">"Percentage"</span>
          
      <span class="token key atrule">memory_chart</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Memory Usage"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
        <span class="token key atrule">chart</span><span class="token punctuation">:</span>
          <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"bar"</span>
          <span class="token key atrule">data</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token number">2.1</span><span class="token punctuation">,</span> <span class="token number">3.4</span><span class="token punctuation">,</span> <span class="token number">2.8</span><span class="token punctuation">,</span> <span class="token number">4.1</span><span class="token punctuation">,</span> <span class="token number">3.2</span><span class="token punctuation">]</span>
          <span class="token key atrule">labels</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">"App1"</span><span class="token punctuation">,</span> <span class="token string">"App2"</span><span class="token punctuation">,</span> <span class="token string">"App3"</span><span class="token punctuation">,</span> <span class="token string">"App4"</span><span class="token punctuation">,</span> <span class="token string">"App5"</span><span class="token punctuation">]</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Memory Usage by Application"</span>
          <span class="token key atrule">y_axis_label</span><span class="token punctuation">:</span> <span class="token string">"GB"</span>
          
      <span class="token key atrule">process_table</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Process Table"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightMagenta"</span>
        <span class="token key atrule">table</span><span class="token punctuation">:</span>
          <span class="token key atrule">data_source</span><span class="token punctuation">:</span> <span class="token string">"csv"</span>
          <span class="token key atrule">data</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
            Process,PID,CPU%,Memory,Status
            nginx,1234,2.3%,45MB,Running
            postgres,5678,15.7%,256MB,Running
            redis,9012,0.8%,12MB,Running
            node,3456,8.4%,128MB,Running
            docker,7890,3.2%,89MB,Running</span>
          <span class="token key atrule">headers</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">sortable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">zebra_striping</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">border_style</span><span class="token punctuation">:</span> <span class="token string">"rounded"</span>

<span class="token key atrule">active</span><span class="token punctuation">:</span> <span class="token string">"main"</span></code>`),a(l);var k=s(l,6),_=n(k);t(_,()=>`<code class="language-yaml"><span class="token comment"># devops_control.yaml</span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"DevOps Control Center"</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">150</span>

<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">env</span><span class="token punctuation">:</span> <span class="token string">"production"</span>
  <span class="token key atrule">kubectl_context</span><span class="token punctuation">:</span> <span class="token string">"prod-cluster"</span>
  <span class="token key atrule">aws_profile</span><span class="token punctuation">:</span> <span class="token string">"production"</span>

<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">main</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">header</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"8%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"☸️ DevOps Control - Environment: &#123;&#123;env&#125;&#125;"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
        
      <span class="token key atrule">kubernetes_menu</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"8%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"20%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Kubernetes"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
        <span class="token key atrule">choices</span><span class="token punctuation">:</span>
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📊 Pod Status"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"kubectl get pods -A"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"main_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🔧 Services"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"kubectl get svc -A"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"main_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📈 Top Nodes"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"kubectl top nodes"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"main_output"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📋 Events"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"kubectl get events --sort-by='.lastTimestamp'"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"main_output"</span>
            
      <span class="token key atrule">deployment_menu</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"20%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Deployments"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
        <span class="token key atrule">choices</span><span class="token punctuation">:</span>
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🚀 Deploy App"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"./deploy.sh &#123;&#123;env&#125;&#125;"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"deployment_log"</span>
            <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"🔄 Rolling Update"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"kubectl rollout restart deployment/app"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"deployment_log"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"⏪ Rollback"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"kubectl rollout undo deployment/app"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"deployment_log"</span>
            
          <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"📊 Rollout Status"</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"kubectl rollout status deployment/app"</span>
            <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"deployment_log"</span>
            
      <span class="token key atrule">main_output</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"20%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"8%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"70%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"60%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Command Output"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Select a command to see output..."</span>
        <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>
        <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        
      <span class="token key atrule">deployment_log</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"20%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"60%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"70%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Deployment Log"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
        <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Deployment output will appear here..."</span>
        <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span> 
        <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        
      <span class="token key atrule">monitoring</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"70%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"8%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Live Monitoring"</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightRed"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
          echo "=== Cluster Status ==="
          kubectl cluster-info --context=&#123;&#123;kubectl_context&#125;&#125; | head -3
          echo ""
          echo "=== Resource Usage ==="
          kubectl top nodes --context=&#123;&#123;kubectl_context&#125;&#125; | head -5
          echo ""
          echo "=== Recent Events ==="
          kubectl get events --context=&#123;&#123;kubectl_context&#125;&#125; --sort-by='.lastTimestamp' | tail -5</span>
        <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">10000</span>
        <span class="token key atrule">thread</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>

<span class="token key atrule">active</span><span class="token punctuation">:</span> <span class="token string">"main"</span>

<span class="token comment"># Environment switching</span>
<span class="token key atrule">on_keypress</span><span class="token punctuation">:</span>
  <span class="token key atrule">"F2"</span><span class="token punctuation">:</span> <span class="token string">"switch_layout:staging"</span>
  <span class="token key atrule">"F3"</span><span class="token punctuation">:</span> <span class="token string">"switch_layout:production"</span></code>`),a(k),f(6),v(r,u)}const R=Object.freeze(Object.defineProperty({__proto__:null,default:w,metadata:i},Symbol.toStringTag,{value:"Module"}));export{R as _};
