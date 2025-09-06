import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as E,s,a as V,b as N,d as n,r as a,n as j}from"./E9AOERj2.js";import{h as t}from"./Cc2oKZLy.js";const B={title:"Customization Guide",description:"How to customize BoxMux applications through YAML configuration, styling, and features"},{title:X,description:ss}=B;var q=E('<h2>YAML Configuration Structure</h2> <p>BoxMux applications are defined using hierarchical YAML configuration files with the following top-level structure:</p> <pre class="language-yaml"><!></pre> <h3>Application-level Customization</h3> <p>Configure global application behavior:</p> <pre class="language-yaml"><!></pre> <h2>Box Configuration & Styling</h2> <h3>Basic Box Properties</h3> <pre class="language-yaml"><!></pre> <h3>Advanced Box Styling</h3> <pre class="language-yaml"><!></pre> <h2>Content Types & Behavior</h2> <h3>Script Execution Boxes</h3> <pre class="language-yaml"><!></pre> <h3>Interactive Menu Boxes</h3> <pre class="language-yaml"><!></pre> <h3>PTY (Interactive Terminal) Boxes</h3> <pre class="language-yaml"><!></pre> <h2>Layout Management</h2> <h3>Multi-Layout Applications</h3> <pre class="language-yaml"><!></pre> <h3>Nested Box Hierarchies</h3> <pre class="language-yaml"><!></pre> <h2>Advanced Customization Features</h2> <h3>Variable System & Templates</h3> <pre class="language-yaml"><!></pre> <p>Environment variables are automatically available:</p> <pre class="language-yaml"><!></pre> <h3>Data Visualization</h3> <pre class="language-yaml"><!></pre> <h3>Content Overflow Handling</h3> <pre class="language-yaml"><!></pre> <h3>Interactive Features</h3> <pre class="language-yaml"><!></pre> <h2>Color Schemes & Themes</h2> <p>BoxMux supports the full 16-color ANSI palette:</p> <pre class="language-yaml"><!></pre> <h2>Performance & Optimization</h2> <h3>Refresh Rate Tuning</h3> <pre class="language-yaml"><!></pre> <h3>Memory Management</h3> <pre class="language-yaml"><!></pre> <h2>Plugin Integration</h2> <pre class="language-yaml"><!></pre> <p>BoxMux uses YAML configuration files to define terminal applications with nested components and variable substitution.</p>',1);function $(M){var v=q(),p=s(V(v),4),w=n(p);t(w,()=>`<code class="language-yaml"><span class="token comment"># Application-level configuration</span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"My Application"</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">100</span>
<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">app_var</span><span class="token punctuation">:</span> <span class="token string">"global_value"</span>

<span class="token comment"># Layout definitions</span>
<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">main</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Welcome to my app"</span>
    <span class="token comment"># ... box configuration</span>
  
  <span class="token key atrule">debug</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span> 
    <span class="token comment"># ... alternative layout</span>

<span class="token comment"># Active layout selection</span>
<span class="token key atrule">active</span><span class="token punctuation">:</span> <span class="token string">"main"</span></code>`),a(p);var e=s(p,6),C=n(e);t(C,()=>`<code class="language-yaml"><span class="token comment"># Global application settings</span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"DevOps Dashboard"</span>           <span class="token comment"># Application title</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">50</span>                    <span class="token comment"># Global refresh interval (ms)</span>
<span class="token key atrule">frame_delay</span><span class="token punctuation">:</span> <span class="token number">16</span>                     <span class="token comment"># Rendering frame delay (ms)</span>

<span class="token comment"># Global variables available to all boxes</span>
<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">environment</span><span class="token punctuation">:</span> <span class="token string">"production"</span>
  <span class="token key atrule">api_endpoint</span><span class="token punctuation">:</span> <span class="token string">"https://api.example.com"</span>
  <span class="token key atrule">log_level</span><span class="token punctuation">:</span> <span class="token string">"info"</span>

<span class="token comment"># Global key bindings</span>
<span class="token key atrule">on_keypress</span><span class="token punctuation">:</span>
  <span class="token key atrule">"Ctrl+r"</span><span class="token punctuation">:</span> <span class="token string">"refresh_all"</span>
  <span class="token key atrule">"Ctrl+q"</span><span class="token punctuation">:</span> <span class="token string">"quit"</span>
  <span class="token key atrule">"F1"</span><span class="token punctuation">:</span> <span class="token string">"show_help"</span></code>`),a(e);var o=s(e,6),S=n(o);t(S,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">header</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"System Monitor"</span>
    
    <span class="token comment"># Positioning (percentage-based or absolute)</span>
    <span class="token key atrule">bounds</span><span class="token punctuation">:</span>
      <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span>      <span class="token comment"># Left edge</span>
      <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span>      <span class="token comment"># Top edge  </span>
      <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span>    <span class="token comment"># Right edge</span>
      <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"20%"</span>     <span class="token comment"># Bottom edge</span>
      <span class="token key atrule">anchor</span><span class="token punctuation">:</span> <span class="token string">"TopLeft"</span>   <span class="token comment"># Anchor point</span>
    
    <span class="token comment"># Visual styling</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"White"</span>
    <span class="token key atrule">fill_char</span><span class="token punctuation">:</span> <span class="token string">" "</span>
    <span class="token key atrule">selected_fill_char</span><span class="token punctuation">:</span> <span class="token string">"█"</span></code>`),a(o);var c=s(o,4),T=n(c);t(T,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">styled_box</span><span class="token punctuation">:</span>
    <span class="token comment"># Border and color options</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>      <span class="token comment"># 16 ANSI colors available</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>       <span class="token comment"># Background fill color</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span> <span class="token comment"># Text color</span>
    
    <span class="token comment"># Fill characters for content areas</span>
    <span class="token key atrule">fill_char</span><span class="token punctuation">:</span> <span class="token string">" "</span>                  <span class="token comment"># Default fill character</span>
    <span class="token key atrule">selected_fill_char</span><span class="token punctuation">:</span> <span class="token string">"░"</span>         <span class="token comment"># Fill when selected/focused</span>
    
    <span class="token comment"># Title positioning and styling</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Custom Box"</span>
    <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
    
    <span class="token comment"># Focus and interaction</span>
    <span class="token key atrule">focusable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">next_focus_id</span><span class="token punctuation">:</span> <span class="token string">"next_box"</span>
    <span class="token key atrule">tab_order</span><span class="token punctuation">:</span> <span class="token number">1</span></code>`),a(c);var l=s(c,6),A=n(l);t(A,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">system_stats</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/bin/bash
      echo "CPU Usage: $(top -l1 | grep "CPU usage")"
      echo "Memory: $(vm_stat | grep "Pages active")"
      echo "Disk: $(df -h / | tail -1)"</span>
    
    <span class="token comment"># Execution configuration</span>
    <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>          <span class="token comment"># Auto-refresh every 2 seconds  </span>
    <span class="token key atrule">thread</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>                    <span class="token comment"># Run in background thread</span>
    <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>                 <span class="token comment"># Stream output as generated</span>
    <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">false</span>                      <span class="token comment"># Use regular process execution</span>
    
    <span class="token comment"># Output redirection</span>
    <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"log_box"</span>      <span class="token comment"># Send output to another box</span>
    <span class="token key atrule">append_output</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>             <span class="token comment"># Append instead of replace</span>
    
    <span class="token comment"># Error handling</span>
    <span class="token key atrule">save_in_file</span><span class="token punctuation">:</span> <span class="token string">"/tmp/stats.log"</span>  <span class="token comment"># Save output to file</span>
    <span class="token key atrule">libs</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">"utils.sh"</span><span class="token punctuation">]</span>              <span class="token comment"># Include script libraries</span></code>`),a(l);var u=s(l,4),P=n(u);t(P,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">main_menu</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">choices</span><span class="token punctuation">:</span>
      <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"Deploy Application"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"./deploy.sh"</span>
        <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">"deployment_log"</span>
        <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        
      <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"Run Tests"</span> 
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"npm test"</span>
        <span class="token key atrule">thread</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>                   <span class="token comment"># Use PTY for interactive output</span>
        
      <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"View Logs"</span>
        <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"tail -f /var/log/app.log"</span>
        <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    
    <span class="token comment"># Menu styling</span>
    <span class="token key atrule">selected_index</span><span class="token punctuation">:</span> <span class="token number">0</span>
    <span class="token key atrule">choice_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
    <span class="token key atrule">selected_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span></code>`),a(u);var k=s(u,4),z=n(k);t(z,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">terminal</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>                       <span class="token comment"># Enable PTY mode</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"bash"</span>                  <span class="token comment"># Start interactive bash</span>
    
    <span class="token comment"># PTY-specific configuration  </span>
    <span class="token key atrule">pty_buffer_size</span><span class="token punctuation">:</span> <span class="token number">10000</span>          <span class="token comment"># Scrollback buffer size</span>
    <span class="token key atrule">pty_scroll_position</span><span class="token punctuation">:</span> <span class="token string">"bottom"</span>   <span class="token comment"># Auto-scroll to bottom</span>
    
    <span class="token comment"># Visual indicators for PTY boxes</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"⚡ Interactive Terminal"</span>  <span class="token comment"># Lightning bolt prefix</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>       <span class="token comment"># Distinctive border color</span></code>`),a(k);var i=s(k,6),L=n(i);t(L,()=>`<code class="language-yaml"><span class="token comment"># Define multiple layouts for different views</span>
<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">dashboard</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">header</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"10%"</span> <span class="token punctuation">&#125;</span><span class="token punctuation">&#125;</span>
      <span class="token key atrule">stats</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span><span class="token punctuation">&#125;</span>
      <span class="token key atrule">logs</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span><span class="token punctuation">&#125;</span>
  
  <span class="token key atrule">monitoring</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span> 
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">cpu_graph</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"33%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span> <span class="token punctuation">&#125;</span><span class="token punctuation">&#125;</span>
      <span class="token key atrule">memory_graph</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"33%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"66%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span> <span class="token punctuation">&#125;</span><span class="token punctuation">&#125;</span>
      <span class="token key atrule">disk_graph</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"66%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"50%"</span> <span class="token punctuation">&#125;</span><span class="token punctuation">&#125;</span>
      <span class="token key atrule">alerts</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"50%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span><span class="token punctuation">&#125;</span>

<span class="token comment"># Set the active layout</span>
<span class="token key atrule">active</span><span class="token punctuation">:</span> <span class="token string">"dashboard"</span>

<span class="token comment"># Switch between layouts using key bindings</span>
<span class="token key atrule">on_keypress</span><span class="token punctuation">:</span>
  <span class="token key atrule">"F2"</span><span class="token punctuation">:</span> <span class="token string">"switch_layout:monitoring"</span>
  <span class="token key atrule">"F3"</span><span class="token punctuation">:</span> <span class="token string">"switch_layout:dashboard"</span></code>`),a(i);var r=s(i,4),Y=n(r);t(Y,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">main_container</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">sidebar</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"25%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token key atrule">children</span><span class="token punctuation">:</span>
          <span class="token key atrule">menu</span><span class="token punctuation">:</span> 
            <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"80%"</span> <span class="token punctuation">&#125;</span>
            <span class="token key atrule">choices</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token punctuation">...</span><span class="token punctuation">]</span>
          <span class="token key atrule">status</span><span class="token punctuation">:</span>
            <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"80%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
            <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"uptime"</span>
      
      <span class="token key atrule">content</span><span class="token punctuation">:</span>
        <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"25%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
        <span class="token comment"># Main content area</span></code>`),a(r);var y=s(r,6),R=n(y);t(R,()=>`<code class="language-yaml"><span class="token comment"># Hierarchical variable definition</span>
<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token comment"># Application-level variables</span>
  <span class="token key atrule">app_name</span><span class="token punctuation">:</span> <span class="token string">"DevOps Dashboard"</span>
  <span class="token key atrule">version</span><span class="token punctuation">:</span> <span class="token string">"1.0.0"</span>
  
<span class="token comment"># Box-level variables (override app-level)</span>
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">header</span><span class="token punctuation">:</span>
    <span class="token key atrule">variables</span><span class="token punctuation">:</span>
      <span class="token key atrule">title_text</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;app_name&#125;&#125; v&#123;&#123;version&#125;&#125;"</span>
    <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Welcome to &#123;&#123;title_text&#125;&#125;"</span>
    
  <span class="token key atrule">api_status</span><span class="token punctuation">:</span>
    <span class="token key atrule">variables</span><span class="token punctuation">:</span>
      <span class="token key atrule">endpoint</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;api_endpoint&#125;&#125;/status"</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"curl -s &#123;&#123;endpoint&#125;&#125; | jq '.status'"</span></code>`),a(y);var g=s(y,4),G=n(g);t(G,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">env_info</span><span class="token punctuation">:</span>
    <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      Current user: &#123;&#123;USER&#125;&#125;
      Home directory: &#123;&#123;HOME&#125;&#125;
      Path: &#123;&#123;PATH&#125;&#125;</span></code>`),a(g);var m=s(g,4),U=n(m);t(U,()=>`<code class="language-yaml"><span class="token comment"># Table display with CSV/JSON data</span>
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">data_table</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">table</span><span class="token punctuation">:</span>
      <span class="token key atrule">data_source</span><span class="token punctuation">:</span> <span class="token string">"csv"</span>
      <span class="token key atrule">data</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
        Name,CPU,Memory,Status
        web-01,45%,67%,Running
        web-02,23%,54%,Running
        db-01,78%,89%,Warning</span>
      
      <span class="token comment"># Table styling</span>
      <span class="token key atrule">headers</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">sortable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">filterable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">zebra_striping</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">border_style</span><span class="token punctuation">:</span> <span class="token string">"rounded"</span>

<span class="token comment"># Chart visualization  </span>
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">cpu_chart</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">chart</span><span class="token punctuation">:</span>
      <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"line"</span>
      <span class="token key atrule">data</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token number">45</span><span class="token punctuation">,</span> <span class="token number">52</span><span class="token punctuation">,</span> <span class="token number">48</span><span class="token punctuation">,</span> <span class="token number">65</span><span class="token punctuation">,</span> <span class="token number">71</span><span class="token punctuation">,</span> <span class="token number">58</span><span class="token punctuation">,</span> <span class="token number">62</span><span class="token punctuation">]</span>
      <span class="token key atrule">labels</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">"Mon"</span><span class="token punctuation">,</span> <span class="token string">"Tue"</span><span class="token punctuation">,</span> <span class="token string">"Wed"</span><span class="token punctuation">,</span> <span class="token string">"Thu"</span><span class="token punctuation">,</span> <span class="token string">"Fri"</span><span class="token punctuation">,</span> <span class="token string">"Sat"</span><span class="token punctuation">,</span> <span class="token string">"Sun"</span><span class="token punctuation">]</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"CPU Usage Over Time"</span>
      <span class="token key atrule">y_axis_label</span><span class="token punctuation">:</span> <span class="token string">"Percentage"</span></code>`),a(m);var d=s(m,4),D=n(d);t(D,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">log_viewer</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>     <span class="token comment"># scroll | wrap | fill | cross_out</span>
    
    <span class="token comment"># Scrolling configuration</span>
    <span class="token key atrule">scrollable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>               <span class="token comment"># Auto-scroll to bottom</span>
    <span class="token key atrule">vertical_scroll</span><span class="token punctuation">:</span> <span class="token string">"0%"</span>           <span class="token comment"># Initial scroll position</span>
    <span class="token key atrule">horizontal_scroll</span><span class="token punctuation">:</span> <span class="token string">"0%"</span>
    
    <span class="token comment"># Content that exceeds box bounds will be scrollable</span>
    <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      Very long log content that exceeds the box dimensions...
      Line 1 of many log entries
      Line 2 of many log entries
      ...many more lines...</span></code>`),a(d);var h=s(d,4),F=n(h);t(F,()=>`<code class="language-yaml"><span class="token comment"># Mouse interaction support</span>
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">interactive_box</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">focusable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    
    <span class="token comment"># Click handlers</span>
    <span class="token key atrule">on_click</span><span class="token punctuation">:</span> <span class="token string">"handle_click_action"</span>
    
    <span class="token comment"># Resize behavior</span>
    <span class="token key atrule">resizable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>                 <span class="token comment"># Allow mouse resize</span>
    <span class="token key atrule">min_width</span><span class="token punctuation">:</span> <span class="token string">"10%"</span>               <span class="token comment"># Minimum dimensions</span>
    <span class="token key atrule">min_height</span><span class="token punctuation">:</span> <span class="token string">"5%"</span>
    
    <span class="token comment"># Z-index for overlapping boxes</span>
    <span class="token key atrule">z_index</span><span class="token punctuation">:</span> <span class="token number">10</span>                    <span class="token comment"># Higher values appear on top</span>

<span class="token comment"># Global hot keys for instant actions  </span>
<span class="token key atrule">hot_keys</span><span class="token punctuation">:</span>
  <span class="token key atrule">"Ctrl+d"</span><span class="token punctuation">:</span> 
    <span class="token key atrule">choice_id</span><span class="token punctuation">:</span> <span class="token string">"deploy_action"</span>
    <span class="token key atrule">box_id</span><span class="token punctuation">:</span> <span class="token string">"main_menu"</span>
  <span class="token key atrule">"Ctrl+t"</span><span class="token punctuation">:</span>
    <span class="token key atrule">choice_id</span><span class="token punctuation">:</span> <span class="token string">"run_tests"</span> 
    <span class="token key atrule">box_id</span><span class="token punctuation">:</span> <span class="token string">"actions"</span></code>`),a(h);var b=s(h,6),O=n(b);t(O,()=>`<code class="language-yaml"><span class="token comment"># Standard colors</span>
<span class="token key atrule">colors</span><span class="token punctuation">:</span>
  <span class="token key atrule">normal</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">"Black"</span><span class="token punctuation">,</span> <span class="token string">"Red"</span><span class="token punctuation">,</span> <span class="token string">"Green"</span><span class="token punctuation">,</span> <span class="token string">"Yellow"</span><span class="token punctuation">,</span> <span class="token string">"Blue"</span><span class="token punctuation">,</span> <span class="token string">"Magenta"</span><span class="token punctuation">,</span> <span class="token string">"Cyan"</span><span class="token punctuation">,</span> <span class="token string">"White"</span><span class="token punctuation">]</span>
  <span class="token key atrule">bright</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">"BrightBlack"</span><span class="token punctuation">,</span> <span class="token string">"BrightRed"</span><span class="token punctuation">,</span> <span class="token string">"BrightGreen"</span><span class="token punctuation">,</span> <span class="token string">"BrightYellow"</span><span class="token punctuation">,</span> 
           <span class="token string">"BrightBlue"</span><span class="token punctuation">,</span> <span class="token string">"BrightMagenta"</span><span class="token punctuation">,</span> <span class="token string">"BrightCyan"</span><span class="token punctuation">,</span> <span class="token string">"BrightWhite"</span><span class="token punctuation">]</span>

<span class="token comment"># Theme example</span>
<span class="token key atrule">theme</span><span class="token punctuation">:</span>
  <span class="token key atrule">primary</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
  <span class="token key atrule">secondary</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span> 
  <span class="token key atrule">success</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
  <span class="token key atrule">warning</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
  <span class="token key atrule">danger</span><span class="token punctuation">:</span> <span class="token string">"BrightRed"</span>
  <span class="token key atrule">muted</span><span class="token punctuation">:</span> <span class="token string">"BrightBlack"</span>

<span class="token comment"># Apply theme to boxes</span>
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">success_box</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;theme.success&#125;&#125;"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;theme.success&#125;&#125;"</span>
  <span class="token key atrule">warning_box</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;theme.warning&#125;&#125;"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;theme.warning&#125;&#125;"</span></code>`),a(b);var _=s(b,6),I=n(_);t(I,()=>`<code class="language-yaml"><span class="token comment"># Global refresh settings</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">50</span>                    <span class="token comment"># Global refresh interval (ms)</span>
<span class="token key atrule">frame_delay</span><span class="token punctuation">:</span> <span class="token number">16</span>                     <span class="token comment"># Target 60 FPS rendering</span>

<span class="token comment"># Per-box refresh rates</span>
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">fast_updates</span><span class="token punctuation">:</span>
    <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">100</span>           <span class="token comment"># Update every 100ms</span>
    
  <span class="token key atrule">slow_updates</span><span class="token punctuation">:</span>
    <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">5000</span>          <span class="token comment"># Update every 5 seconds</span>
    
  <span class="token key atrule">on_demand</span><span class="token punctuation">:</span>
    <span class="token comment"># No refresh_interval = manual refresh only</span></code>`),a(_);var x=s(_,4),H=n(x);t(H,()=>`<code class="language-yaml"><span class="token comment"># PTY buffer management</span>
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">terminal</span><span class="token punctuation">:</span>
    <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">pty_buffer_size</span><span class="token punctuation">:</span> <span class="token number">5000</span>           <span class="token comment"># Limit scrollback to 5k lines</span>
    
<span class="token comment"># Large content handling</span>
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">large_logs</span><span class="token punctuation">:</span>
    <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>
    <span class="token key atrule">max_content_lines</span><span class="token punctuation">:</span> <span class="token number">1000</span>         <span class="token comment"># Limit content buffer size</span></code>`),a(x);var f=s(x,4),W=n(f);t(W,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">custom_component</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">plugin</span><span class="token punctuation">:</span> <span class="token string">"my_custom_plugin"</span>      <span class="token comment"># Load custom plugin</span>
    <span class="token key atrule">plugin_config</span><span class="token punctuation">:</span>
      <span class="token key atrule">setting1</span><span class="token punctuation">:</span> <span class="token string">"value1"</span>
      <span class="token key atrule">setting2</span><span class="token punctuation">:</span> <span class="token string">"value2"</span></code>`),a(f),j(2),N(M,v)}const ns=Object.freeze(Object.defineProperty({__proto__:null,default:$,metadata:B},Symbol.toStringTag,{value:"Module"}));export{ns as _};
