import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as E,s,a as $,b as q,d as a,r as n,n as O}from"./E9AOERj2.js";import{h as t}from"./Cc2oKZLy.js";const v={title:"BoxMux Code Highlighting",description:"Syntax highlighting and code formatting in BoxMux YAML configurations"},{title:z,description:G}=v;var D=E('<h1>BoxMux Code Highlighting</h1> <p>BoxMux configurations use YAML syntax and support embedded shell scripts with proper syntax highlighting in documentation and examples.</p> <h2>YAML Configuration Syntax</h2> <p>BoxMux uses standard YAML syntax with specific structures for terminal UI definition:</p> <pre class="language-yaml"><!></pre> <h2>Shell Script Highlighting</h2> <p>BoxMux configurations often contain embedded shell scripts with proper syntax highlighting:</p> <pre class="language-yaml"><!></pre> <h2>Configuration Examples by Language</h2> <h3>Bash Scripts</h3> <pre class="language-yaml"><!></pre> <h3>Python Scripts</h3> <pre class="language-yaml"><!></pre> <h3>Node.js Scripts</h3> <pre class="language-yaml"><!></pre> <h3>Docker Commands</h3> <pre class="language-yaml"><!></pre> <h2>Advanced Configuration Patterns</h2> <h3>Multi-line YAML Values</h3> <pre class="language-yaml"><!></pre> <h3>Variable Substitution Patterns</h3> <pre class="language-yaml"><!></pre> <h2>PTY Integration Examples</h2> <h3>Interactive Terminal Sessions</h3> <pre class="language-yaml"><!></pre> <h3>Interactive Applications</h3> <pre class="language-yaml"><!></pre> <h2>Socket API Command Syntax</h2> <h3>CLI Command Examples</h3> <pre class="language-bash"><!></pre> <h3>Socket Function Configuration</h3> <pre class="language-yaml"><!></pre> <h2>Color and Styling Syntax</h2> <h3>ANSI Color Support</h3> <pre class="language-yaml"><!></pre> <h3>Available Colors</h3> <pre class="language-yaml"><!></pre> <h2>Configuration Validation</h2> <p>BoxMux includes JSON schema validation for YAML files:</p> <pre class="language-json"><!></pre> <h2>Best Practices</h2> <h3>YAML Formatting</h3> <ul><li>Use 2-space indentation consistently</li> <li>Quote string values that contain special characters</li> <li>Use <code>|</code> for multi-line script content</li> <li>Validate YAML syntax before running</li></ul> <h3>Script Organization</h3> <ul><li>Use shebang lines for clarity (<code>#!/bin/bash</code>)</li> <li>Add comments to explain complex operations</li> <li>Test scripts independently before embedding</li> <li>Handle errors gracefully with appropriate exit codes</li></ul> <h3>Variable Usage</h3> <ul><li>Use descriptive variable names</li> <li>Document variable purposes in comments</li> <li>Prefer environment variables for sensitive data</li> <li>Use hierarchical variable scoping appropriately</li></ul> <p>BoxMux uses YAML configuration files to define terminal applications with syntax highlighting and validation support.</p>',1);function F(_){var b=D(),e=s($(b),8),f=a(e);t(f,()=>`<code class="language-yaml"><span class="token comment"># Basic BoxMux configuration structure</span>
<span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"My Application"</span>
<span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">100</span>

<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">app_name</span><span class="token punctuation">:</span> <span class="token string">"MyApp"</span>
  <span class="token key atrule">version</span><span class="token punctuation">:</span> <span class="token string">"1.0.0"</span>

<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">main</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">bounds</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span> <span class="token key atrule">x1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> <span class="token string">"0%"</span><span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span><span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> <span class="token string">"100%"</span> <span class="token punctuation">&#125;</span>
    <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"Welcome to &#123;&#123;app_name&#125;&#125; v&#123;&#123;version&#125;&#125;"</span></code>`),n(e);var p=s(e,6),S=a(p);t(S,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">system_monitor</span><span class="token punctuation">:</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"System Stats"</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/bin/bash
      echo "CPU Usage:"
      top -l1 | grep "CPU usage" | awk '&#123;print $3, $5&#125;'
      echo "Memory:"
      vm_stat | grep -E "(Pages active)" | awk '&#123;print $3&#125;'</span>
    <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>
    <span class="token key atrule">thread</span><span class="token punctuation">:</span> <span class="token boolean important">true</span></code>`),n(p);var o=s(p,6),B=a(o);t(B,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">bash_example</span><span class="token punctuation">:</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/bin/bash
      for i in &#123;1..5&#125;; do
        echo "Count: $i"
        sleep 1
      done</span></code>`),n(o);var c=s(o,4),M=a(c);t(M,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">python_example</span><span class="token punctuation">:</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/usr/bin/env python3
      import json
      import requests</span>
      
      response = requests.get('https<span class="token punctuation">:</span>//api.github.com/user')
      print(json.dumps(response.json()<span class="token punctuation">,</span> indent=2))</code>`),n(c);var l=s(c,4),C=a(l);t(C,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">nodejs_example</span><span class="token punctuation">:</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/usr/bin/env node
      const fs = require('fs');
      const package = JSON.parse(fs.readFileSync('package.json', 'utf8'));
      console.log(&#96;Project: $&#123;package.name&#125; v$&#123;package.version&#125;&#96;);</span></code>`),n(l);var r=s(l,4),w=a(r);t(w,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">docker_example</span><span class="token punctuation">:</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/bin/bash
      echo "Running Containers:"
      docker ps --format "table &#123;&#123;.Names&#125;&#125;&#92;t&#123;&#123;.Status&#125;&#125;&#92;t&#123;&#123;.Ports&#125;&#125;"
      echo ""
      echo "Images:"
      docker images --format "table &#123;&#123;.Repository&#125;&#125;&#92;t&#123;&#123;.Tag&#125;&#125;&#92;t&#123;&#123;.Size&#125;&#125;"</span></code>`),n(r);var i=s(r,6),P=a(i);t(P,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">complex_script</span><span class="token punctuation">:</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/bin/bash
      # This is a complex monitoring script
      echo "=== System Overview ==="
      uname -a</span>
      
      echo "=== Load Average ==="
      uptime <span class="token punctuation">|</span> awk '<span class="token punctuation">&#123;</span>print $NF<span class="token punctuation">&#125;</span>'
      
      echo "=== Disk Usage ==="
      df <span class="token punctuation">-</span>h <span class="token punctuation">|</span> grep <span class="token punctuation">-</span>v tmpfs <span class="token punctuation">|</span> grep <span class="token punctuation">-</span>v devtmpfs
      
      echo "=== Network Connections ==="
      netstat <span class="token punctuation">-</span>tuln <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">10</span></code>`),n(i);var u=s(i,4),A=a(u);t(A,()=>`<code class="language-yaml"><span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">server_host</span><span class="token punctuation">:</span> <span class="token string">"production-server.com"</span>
  <span class="token key atrule">ssh_user</span><span class="token punctuation">:</span> <span class="token string">"deploy"</span>
  <span class="token key atrule">app_path</span><span class="token punctuation">:</span> <span class="token string">"/var/www/myapp"</span>

<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">deployment</span><span class="token punctuation">:</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/bin/bash
      echo "Deploying to &#123;&#123;server_host&#125;&#125;"
      ssh &#123;&#123;ssh_user&#125;&#125;@&#123;&#123;server_host&#125;&#125; "cd &#123;&#123;app_path&#125;&#125; &amp;&amp; git pull &amp;&amp; npm install"
      echo "Deployment complete!"</span></code>`),n(u);var k=s(u,6),Y=a(k);t(Y,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">terminal</span><span class="token punctuation">:</span>
    <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>                    <span class="token comment"># Enable pseudo-terminal</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"bash"</span>               <span class="token comment"># Start interactive bash</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"⚡ Interactive Terminal"</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span></code>`),n(k);var g=s(k,4),T=a(g);t(T,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">file_manager</span><span class="token punctuation">:</span>
    <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"ranger"</span>             <span class="token comment"># File manager requires PTY</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"📁 File Manager"</span>
    
  <span class="token key atrule">process_monitor</span><span class="token punctuation">:</span>
    <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"htop"</span>               <span class="token comment"># Process monitor requires PTY</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"📊 Process Monitor"</span>
    
  <span class="token key atrule">text_editor</span><span class="token punctuation">:</span>
    <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"vim README.md"</span>      <span class="token comment"># Text editor requires PTY</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"📝 Editor"</span></code>`),n(g);var m=s(g,6),j=a(m);t(j,()=>`<code class="language-bash"><span class="token comment"># Update box content via socket</span>
boxmux replace-box-content <span class="token string">"status_box"</span> <span class="token string">"New status: Online"</span>

<span class="token comment"># Execute PTY commands</span>
boxmux spawn-pty <span class="token string">"terminal_box"</span> <span class="token parameter variable">--script</span><span class="token operator">=</span><span class="token string">"vim file.txt"</span> <span class="token parameter variable">--pty</span>

<span class="token comment"># Send input to PTY processes  </span>
boxmux send-pty-input <span class="token string">"terminal_box"</span> <span class="token string">"Hello World<span class="token entity" title="&#92;n">&#92;n</span>"</span>

<span class="token comment"># Query PTY status</span>
boxmux query-pty-status <span class="token string">"terminal_box"</span></code>`),n(m);var y=s(m,4),L=a(y);t(L,()=>`<code class="language-yaml"><span class="token comment"># Remote control via socket commands</span>
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">remote_controlled</span><span class="token punctuation">:</span>
    <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">"This box can be updated via socket"</span>
    <span class="token comment"># Content will be replaced via socket commands:</span>
    <span class="token comment"># boxmux replace-box-content "remote_controlled" "New content"</span></code>`),n(y);var h=s(y,6),U=a(h);t(U,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">colored_box</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>       <span class="token comment"># 16 ANSI colors</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>  <span class="token comment"># Text color</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>        <span class="token comment"># Background color</span>
    <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>      <span class="token comment"># Title color</span></code>`),n(h);var d=s(h,4),I=a(d);t(I,()=>`<code class="language-yaml"><span class="token comment"># Standard ANSI colors</span>
<span class="token key atrule">colors</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> "Black"       <span class="token punctuation">-</span> "Red"         <span class="token punctuation">-</span> "Green"       <span class="token punctuation">-</span> <span class="token string">"Yellow"</span>
  <span class="token punctuation">-</span> "Blue"        <span class="token punctuation">-</span> "Magenta"     <span class="token punctuation">-</span> "Cyan"        <span class="token punctuation">-</span> <span class="token string">"White"</span>
  <span class="token punctuation">-</span> "BrightBlack" <span class="token punctuation">-</span> "BrightRed"   <span class="token punctuation">-</span> "BrightGreen" <span class="token punctuation">-</span> <span class="token string">"BrightYellow"</span>  
  <span class="token punctuation">-</span> "BrightBlue"  <span class="token punctuation">-</span> "BrightMagenta" <span class="token punctuation">-</span> "BrightCyan" <span class="token punctuation">-</span> <span class="token string">"BrightWhite"</span></code>`),n(d);var x=s(d,6),N=a(x);t(N,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"$schema"</span><span class="token operator">:</span> <span class="token string">"https://raw.githubusercontent.com/jowharshamshiri/boxmux/main/schemas/app_schema.json"</span><span class="token punctuation">,</span>
  <span class="token property">"type"</span><span class="token operator">:</span> <span class="token string">"object"</span><span class="token punctuation">,</span>
  <span class="token property">"properties"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"title"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span> <span class="token property">"type"</span><span class="token operator">:</span> <span class="token string">"string"</span> <span class="token punctuation">&#125;</span><span class="token punctuation">,</span>
    <span class="token property">"refresh_rate"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span> <span class="token property">"type"</span><span class="token operator">:</span> <span class="token string">"integer"</span><span class="token punctuation">,</span> <span class="token property">"minimum"</span><span class="token operator">:</span> <span class="token number">1</span> <span class="token punctuation">&#125;</span><span class="token punctuation">,</span>
    <span class="token property">"layouts"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
      <span class="token property">"type"</span><span class="token operator">:</span> <span class="token string">"object"</span><span class="token punctuation">,</span>
      <span class="token property">"additionalProperties"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span> <span class="token property">"$ref"</span><span class="token operator">:</span> <span class="token string">"#/definitions/MuxBox"</span> <span class="token punctuation">&#125;</span>
    <span class="token punctuation">&#125;</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),n(x),O(16),q(_,b)}const J=Object.freeze(Object.defineProperty({__proto__:null,default:F,metadata:v},Symbol.toStringTag,{value:"Module"}));export{J as _};
