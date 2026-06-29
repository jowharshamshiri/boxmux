import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as V,s as n,a as j,b as W,d as s,r as a,n as G}from"./E9AOERj2.js";import{h as t}from"./Cc2oKZLy.js";const T={title:"PTY Features",description:"Interactive terminal emulation in BoxMux boxes with keyboard interaction, ANSI processing, and process management."},{title:sn,description:an}=T;var Q=V('<h2>Table of Contents</h2> <ul><li><a href="#overview">Overview</a></li> <li><a href="#basic-pty-configuration">Basic PTY Configuration</a></li> <li><a href="#interactive-applications">Interactive Applications</a></li> <li><a href="#pty-process-management">PTY Process Management</a></li> <li><a href="#input-and-navigation">Input and Navigation</a></li> <li><a href="#visual-indicators">Visual Indicators</a></li> <li><a href="#socket-control">Socket Control</a></li> <li><a href="#error-handling">Error Handling</a></li> <li><a href="#best-practices">Best Practices</a></li></ul> <h2>Overview</h2> <p>PTY (Pseudo-Terminal) features allow you to run interactive terminal applications like <code>vim</code>, <code>htop</code>, <code>ssh</code>, <code>less</code>, <code>nano</code>, and database shells within BoxMux boxes. This provides terminal multiplexing with organized box layouts.</p> <h3>Key Benefits</h3> <ul><li><strong>Interactivity</strong>: Keyboard input routing to terminal applications</li> <li><strong>Process Management</strong>: Kill, restart, and monitor PTY processes</li> <li><strong>ANSI Support</strong>: Handling of colors, cursor movements, and escape sequences</li> <li><strong>Scrollback Buffer</strong>: 10,000-line circular buffer for command history</li> <li><strong>Visual Feedback</strong>: Lightning bolt indicators and color-coded borders</li> <li><strong>Error Recovery</strong>: Fallback to regular execution on PTY failures</li></ul> <h2>Basic PTY Configuration</h2> <p>Enable PTY for any box by adding <code>pty: true</code>:</p> <pre class="language-yaml"><!></pre> <h3>PTY vs Regular Execution</h3> <pre class="language-yaml"><!></pre> <h2>Interactive Applications</h2> <h3>System Monitoring</h3> <pre class="language-yaml"><!></pre> <h3>Text Editors</h3> <pre class="language-yaml"><!></pre> <h3>Remote Connections</h3> <pre class="language-yaml"><!></pre> <h3>File Navigation</h3> <pre class="language-yaml"><!></pre> <h2>PTY Process Management</h2> <h3>Lifecycle Control</h3> <p>PTY processes have full lifecycle management:</p> <ul><li><strong>Automatic Start</strong>: Process starts when box initializes</li> <li><strong>Process Monitoring</strong>: Track running/stopped/failed states</li> <li><strong>Manual Control</strong>: Kill, restart via keyboard or socket</li> <li><strong>Resource Cleanup</strong>: Proper cleanup when BoxMux exits</li></ul> <h3>Process Status</h3> <p>PTY boxes show process information in their titles:</p> <pre class="language-undefined"><!></pre> <h3>Process Actions</h3> <p>Available process management actions:</p> <pre class="language-yaml"><!></pre> <h2>Input and Navigation</h2> <h3>Keyboard Input Routing</h3> <p>When a PTY box is focused, all keyboard input is routed directly to the running process:</p> <ul><li><strong>Regular Keys</strong>: Letters, numbers, symbols sent directly</li> <li><strong>Special Keys</strong>: Arrow keys, function keys (F1-F24), navigation keys</li> <li><strong>Modifier Keys</strong>: Ctrl, Alt, Shift combinations</li> <li><strong>Terminal Keys</strong>: Tab, Backspace, Delete, Enter, Escape</li></ul> <h3>Navigation Keys Supported</h3> <pre class="language-undefined"><!></pre> <h3>Focus Management</h3> <p>Switch between PTY and regular boxes:</p> <ul><li><strong>Tab/Shift+Tab</strong>: Navigate between focusable boxes</li> <li><strong>Mouse Click</strong>: Focus PTY box and enable input routing</li> <li><strong>Box Selection</strong>: Visual indicators show which box receives input</li></ul> <h3>Scrollback Navigation</h3> <p>PTY boxes maintain scrollback history:</p> <ul><li><strong>Circular Buffer</strong>: 10,000 lines of command history</li> <li><strong>Memory Efficient</strong>: Automatic cleanup of old content</li> <li><strong>Search Support</strong>: Search through command history</li> <li><strong>Thread Safe</strong>: Concurrent access from PTY reader threads</li></ul> <h2>Visual Indicators</h2> <h3>Box Title Indicators</h3> <p>PTY boxes display special visual indicators:</p> <pre class="language-undefined"><!></pre> <h3>Border Colors</h3> <p>PTY boxes use distinct border colors:</p> <ul><li><strong>PTY Active</strong>: Bright cyan borders</li> <li><strong>PTY Error</strong>: Red borders with error indicators</li> <li><strong>Regular Box</strong>: Standard border colors</li></ul> <h3>Status Information</h3> <p>Process status appears in box titles:</p> <ul><li><strong>PID</strong>: Process ID when running</li> <li><strong>State</strong>: Running, Stopped, Error, Connected</li> <li><strong>Resource Info</strong>: Memory usage, connection status</li></ul> <h2>Socket Control</h2> <h3>Remote PTY Management</h3> <p>Control PTY processes via Unix socket:</p> <pre class="language-bash"><!></pre> <h3>Batch Operations</h3> <pre class="language-bash"><!></pre> <h2>Error Handling</h2> <h3>Automatic Fallback</h3> <p>When PTY fails, BoxMux automatically falls back to regular execution:</p> <pre class="language-yaml"><!></pre> <h3>Failure Tracking</h3> <p>BoxMux tracks PTY failures and avoids repeated attempts:</p> <ul><li><strong>Failure Threshold</strong>: After 3 consecutive failures, avoid PTY for that box</li> <li><strong>Success Recovery</strong>: Clear failure count on successful PTY startup</li> <li><strong>Box-Specific</strong>: Failure tracking per box, not global</li></ul> <h3>Error States</h3> <p>PTY boxes can be in various error states:</p> <pre class="language-yaml"><!></pre> <h3>Error Recovery</h3> <p>Manual recovery options:</p> <ul><li><strong>Box Restart</strong>: Kill and restart PTY process</li> <li><strong>Configuration Reload</strong>: Reload YAML with updated settings</li> <li><strong>Fallback Mode</strong>: Disable PTY and use regular execution</li></ul> <h2>Best Practices</h2> <h3>Performance Considerations</h3> <pre class="language-yaml"><!></pre> <h3>Resource Management</h3> <pre class="language-yaml"><!></pre> <h3>Security Considerations</h3> <pre class="language-yaml"><!></pre> <h3>Layout Design</h3> <pre class="language-yaml"><!></pre> <h3>Troubleshooting</h3> <p>Common PTY issues and solutions:</p> <pre class="language-yaml"><!></pre> <p>PTY features provide powerful terminal multiplexing capabilities within BoxMux’s organized box system, enabling complex interactive workflows with proper process management and visual organization.</p>',1);function Z(S){var _=Q(),e=n(j(_),16),Y=s(e);t(Y,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'main'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'interactive_box'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Interactive Terminal ⚡'</span>
          <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> htop
          <span class="token key atrule">position</span><span class="token punctuation">:</span>
            <span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%
            <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%
            <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%
            <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%</code>`),a(e);var p=n(e,4),C=s(p);t(C,()=>`<code class="language-yaml"><span class="token comment"># Regular script execution (non-interactive)</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'regular_box'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Info'</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> ps aux <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">10</span>
    <span class="token punctuation">-</span> df <span class="token punctuation">-</span>h
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">5000</span>

<span class="token comment"># PTY execution (interactive)</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'pty_box'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Interactive Top ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> htop
  <span class="token comment"># No refresh_interval needed - PTY runs continuously</span></code>`),a(p);var o=n(p,6),I=s(o);t(I,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'htop_monitor'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Monitor ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> htop
  <span class="token key atrule">position</span><span class="token punctuation">:</span>
    <span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%
    <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%
    <span class="token key atrule">x2</span><span class="token punctuation">:</span> 50%
    <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%

<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'iotop_monitor'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'IO Monitor ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> sudo iotop
  <span class="token key atrule">position</span><span class="token punctuation">:</span>
    <span class="token key atrule">x1</span><span class="token punctuation">:</span> 50%
    <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%
    <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%
    <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%</code>`),a(o);var c=n(o,4),w=s(c);t(w,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'vim_editor'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Text Editor ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> vim /path/to/config.yaml
  <span class="token key atrule">position</span><span class="token punctuation">:</span>
    <span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%
    <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%
    <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%
    <span class="token key atrule">y2</span><span class="token punctuation">:</span> 70%

<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'nano_editor'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Simple Editor ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> nano /etc/hosts</code>`),a(c);var l=n(c,4),M=s(l);t(M,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'ssh_session'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Production Server ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> ssh user@production<span class="token punctuation">-</span>server.com
  <span class="token key atrule">position</span><span class="token punctuation">:</span>
    <span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%
    <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%
    <span class="token key atrule">x2</span><span class="token punctuation">:</span> 50%
    <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%

<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'database_shell'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Database Console ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> psql <span class="token punctuation">-</span>h localhost <span class="token punctuation">-</span>U postgres <span class="token punctuation">-</span>d myapp
  <span class="token key atrule">position</span><span class="token punctuation">:</span>
    <span class="token key atrule">x1</span><span class="token punctuation">:</span> 50%
    <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%
    <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%
    <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%</code>`),a(l);var i=n(l,4),B=s(i);t(B,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'file_manager'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'File Manager ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> ranger
    <span class="token comment"># or: mc (midnight commander)</span>
    <span class="token comment"># or: nnn</span></code>`),a(i);var u=n(i,14),E=s(u);t(E,()=>`<code class="language-undefined">Interactive Top ⚡ [PID: 12345, Running]
SSH Session ⚡ [PID: 12346, Connected]  
Text Editor ⚡ [Process Stopped]
Database Shell ⚡ [PID: 12347, Error]</code>`),a(u);var r=n(u,6),R=s(r);t(R,()=>`<code class="language-yaml"><span class="token comment"># In choice menus for PTY control</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'pty_controls'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'PTY Controls'</span>
  <span class="token key atrule">choices</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'kill_htop'</span>
      <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Kill htop process'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> echo '<span class="token punctuation">&#123;</span><span class="token key atrule">"Command"</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">"action"</span><span class="token punctuation">:</span> <span class="token string">"kill_pty"</span><span class="token punctuation">,</span> <span class="token key atrule">"box_id"</span><span class="token punctuation">:</span> <span class="token string">"htop_monitor"</span><span class="token punctuation">&#125;</span><span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> nc <span class="token punctuation">-</span>U /tmp/boxmux.sock
    
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'restart_ssh'</span>
      <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Restart SSH session'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> echo '<span class="token punctuation">&#123;</span><span class="token key atrule">"Command"</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">"action"</span><span class="token punctuation">:</span> <span class="token string">"restart_pty"</span><span class="token punctuation">,</span> <span class="token key atrule">"box_id"</span><span class="token punctuation">:</span> <span class="token string">"ssh_session"</span><span class="token punctuation">&#125;</span><span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> nc <span class="token punctuation">-</span>U /tmp/boxmux.sock</code>`),a(r);var k=n(r,12),F=s(k);t(F,()=>`<code class="language-undefined">Arrow Keys:     ↑ ↓ ← →
Function Keys:  F1 F2 F3 ... F24
Navigation:     Home End PageUp PageDown
Editing:        Insert Delete Backspace
Modifiers:      Ctrl+C Ctrl+Z Ctrl+D etc.</code>`),a(k);var m=n(k,20),A=s(m);t(A,()=>`<code class="language-undefined">Regular Box:      &quot;System Info&quot;
PTY Box:          &quot;Interactive Top ⚡&quot; 
PTY with Process:   &quot;SSH Session ⚡ [PID: 12345, Running]&quot;
PTY Error State:    &quot;Failed Process ⚡ [Process Stopped]&quot;</code>`),a(m);var y=n(m,20),D=s(y);t(D,()=>`<code class="language-bash"><span class="token comment"># Kill PTY process</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"Command": &#123;"action": "kill_pty", "box_id": "htop_box"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock

<span class="token comment"># Restart PTY process  </span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"Command": &#123;"action": "restart_pty", "box_id": "ssh_session"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock

<span class="token comment"># Query PTY status</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"Command": &#123;"action": "pty_status", "box_id": "vim_box"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock

<span class="token comment"># Send input to PTY (for automation)</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"Command": &#123;"action": "pty_input", "box_id": "database", "input": "SELECT * FROM users LIMIT 5;&#92;n"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(y);var g=n(y,4),K=s(g);t(K,()=>`<code class="language-bash"><span class="token comment"># Kill all PTY processes</span>
<span class="token keyword">for</span> <span class="token for-or-select variable">box</span> <span class="token keyword">in</span> htop_box ssh_session vim_editor<span class="token punctuation">;</span> <span class="token keyword">do</span>
  <span class="token builtin class-name">echo</span> <span class="token string">'&#123;"Command": &#123;"action": "kill_pty", "box_id": "'</span><span class="token variable">$box</span><span class="token string">'"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock
<span class="token keyword">done</span>

<span class="token comment"># Restart development environment</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"Command": &#123;"action": "restart_pty", "box_id": "vim_editor"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"Command": &#123;"action": "restart_pty", "box_id": "test_runner"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(g);var d=n(g,8),N=s(d);t(N,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'robust_box'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Status'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>  <span class="token comment"># Try PTY first</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> htop  <span class="token comment"># Interactive if PTY works, static output if PTY fails</span></code>`),a(d);var h=n(d,12),U=s(h);t(U,()=>`<code class="language-yaml"><span class="token comment"># Error state indicators in titles</span>
"Process Name ⚡ <span class="token punctuation">[</span>Process Stopped<span class="token punctuation">]</span>"      <span class="token comment"># Process exited</span>
"Process Name ⚡ <span class="token punctuation">[</span>PTY Failed<span class="token punctuation">]</span>"           <span class="token comment"># PTY allocation failed  </span>
"Process Name ⚡ <span class="token punctuation">[</span>Connection Lost<span class="token punctuation">]</span>"      <span class="token comment"># SSH/network failure</span>
"Process Name ⚡ <span class="token punctuation">[</span>Permission Denied<span class="token punctuation">]</span>"    <span class="token comment"># Insufficient permissions</span></code>`),a(h);var b=n(h,12),q=s(b);t(q,()=>`<code class="language-yaml"><span class="token comment"># Good: Specific PTY processes</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'htop_box'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> htop

<span class="token comment"># Avoid: Heavy output in PTY</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'log_box'</span>  
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">false</span>  <span class="token comment"># Use regular execution for high-volume logs</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> tail <span class="token punctuation">-</span>f /var/log/messages
  <span class="token key atrule">auto_scroll_bottom</span><span class="token punctuation">:</span> <span class="token boolean important">true</span></code>`),a(b);var v=n(b,4),H=s(v);t(H,()=>`<code class="language-yaml"><span class="token comment"># Limit concurrent PTY processes</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'development'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token comment"># Core interactive tools (keep PTY)</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'vim_editor'</span>
          <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span>vim<span class="token punctuation">]</span>
        
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'htop_monitor'</span>  
          <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span>htop<span class="token punctuation">]</span>
          
        <span class="token comment"># Background processes (regular execution)</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'build_output'</span>
          <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">false</span>
          <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span>./watch<span class="token punctuation">-</span>build.sh<span class="token punctuation">]</span></code>`),a(v);var x=n(v,4),L=s(x);t(L,()=>`<code class="language-yaml"><span class="token comment"># Be careful with PTY and sensitive operations</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'secure_session'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Admin Console ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> ssh <span class="token punctuation">-</span>o ServerAliveInterval=60 admin@secure<span class="token punctuation">-</span>server
  <span class="token comment"># Consider: timeout, logging, access controls</span></code>`),a(x);var f=n(x,4),O=s(f);t(O,()=>`<code class="language-yaml"><span class="token comment"># Organize PTY boxes logically</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'development'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Development Environment'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token comment"># Main editor (large space)</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'editor'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Code Editor ⚡'</span>
          <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span>vim<span class="token punctuation">]</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 70%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 70%<span class="token punctuation">&#125;</span>
          
        <span class="token comment"># Monitoring (side box)  </span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'system'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Monitor ⚡'</span>
          <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span>htop<span class="token punctuation">]</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 70%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">&#125;</span>
          
        <span class="token comment"># Terminal (bottom)</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'shell'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Shell ⚡'</span>
          <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span>bash<span class="token punctuation">]</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 70%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span></code>`),a(f);var P=n(f,6),z=s(P);t(z,()=>`<code class="language-yaml"><span class="token comment"># Issue: PTY not starting</span>
<span class="token comment"># Solution: Check permissions and binary paths</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'debug_box'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> /usr/bin/htop  <span class="token comment"># Use full path</span>
    
<span class="token comment"># Issue: Input not working</span>
<span class="token comment"># Solution: Ensure box is focused and check key bindings</span>

<span class="token comment"># Issue: Display corruption</span>
<span class="token comment"># Solution: Use ANSI processing and proper terminal size</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'clean_display'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> TERM=xterm<span class="token punctuation">-</span>256color htop</code>`),a(P),G(2),W(S,_)}const tn=Object.freeze(Object.defineProperty({__proto__:null,default:Z,metadata:T},Symbol.toStringTag,{value:"Module"}));export{tn as _};
