import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as Rs,s,a as Is,b as Ms,d as n,r as a}from"./E9AOERj2.js";import{h as p}from"./Cc2oKZLy.js";const K={title:"API Reference",description:"Complete API reference for BoxMux socket messaging, external integrations, and programmatic control - commands, message formats, and integration examples"},{title:Ls,description:Fs}=K;var Ts=Rs('<h2>Table of Contents</h2> <ul><li><a href="#socket-api">Socket API</a></li> <li><a href="#message-format">Message Format</a></li> <li><a href="#command-reference">Command Reference</a></li> <li><a href="#box-operations">Box Operations</a></li> <li><a href="#layout-operations">Layout Operations</a></li> <li><a href="#system-operations">System Operations</a></li> <li><a href="#event-handling">Event Handling</a></li> <li><a href="#client-libraries">Client Libraries</a></li> <li><a href="#integration-examples">Integration Examples</a></li></ul> <h2>Socket API</h2> <p>BoxMux provides a Unix socket interface for real-time communication and control. The socket is created at <code>/tmp/boxmux.sock</code> by default.</p> <h3>Basic Usage</h3> <pre class="language-bash"><!></pre> <h3>Connection</h3> <p>The socket accepts JSON messages terminated by newlines. Each message should be a single JSON object.</p> <pre class="language-bash"><!></pre> <h2>Message Format</h2> <p>All messages are JSON objects with a command type as the root key:</p> <pre class="language-json"><!></pre> <h3>Response Format</h3> <p>BoxMux may send responses for certain commands:</p> <pre class="language-json"><!></pre> <h2>Command Reference</h2> <h3>UpdateBox</h3> <p>Update the content of a specific box.</p> <pre class="language-json"><!></pre> <p><strong>Parameters:</strong></p> <ul><li><code>box_id</code> (string): ID of the box to update</li> <li><code>content</code> (string): New content to display</li></ul> <p><strong>Example:</strong></p> <pre class="language-bash"><!></pre> <h3>AppendBox</h3> <p>Append content to a specific box.</p> <pre class="language-json"><!></pre> <p><strong>Parameters:</strong></p> <ul><li><code>box_id</code> (string): ID of the box to update</li> <li><code>content</code> (string): Content to append</li></ul> <p><strong>Example:</strong></p> <pre class="language-bash"><!></pre> <h3>RefreshBox</h3> <p>Trigger a refresh of a specific box (executes its script).</p> <pre class="language-json"><!></pre> <p><strong>Parameters:</strong></p> <ul><li><code>box_id</code> (string): ID of the box to refresh</li></ul> <p><strong>Example:</strong></p> <pre class="language-bash"><!></pre> <h3>SetBoxProperty</h3> <p>Update a specific property of a box.</p> <pre class="language-json"><!></pre> <p><strong>Parameters:</strong></p> <ul><li><code>box_id</code> (string): ID of the box to update</li> <li><code>property</code> (string): Property name to update</li> <li><code>value</code> (any): New value for the property</li></ul> <p><strong>Supported Properties:</strong></p> <ul><li><code>bg_color</code>, <code>fg_color</code></li> <li><code>title_bg_color</code>, <code>title_fg_color</code></li> <li><code>border_color</code>, <code>selected_border_color</code></li> <li><code>refresh_interval</code></li> <li><code>content</code></li> <li><code>title</code></li></ul> <p><strong>Example:</strong></p> <pre class="language-bash"><!></pre> <h3>ExecuteScript</h3> <p>Execute a script in the context of a box.</p> <pre class="language-json"><!></pre> <p><strong>Parameters:</strong></p> <ul><li><code>box_id</code> (string): ID of the box to execute script in</li> <li><code>script</code> (array[string]): Commands to execute</li> <li><code>append</code> (boolean): Whether to append or replace output</li></ul> <p><strong>Example:</strong></p> <pre class="language-bash"><!></pre> <h3>SendKey</h3> <p>Send a key press to BoxMux.</p> <pre class="language-json"><!></pre> <p><strong>Parameters:</strong></p> <ul><li><code>key</code> (string): Key to send (e.g., “Tab”, “Enter”, “Escape”, “a”, “1”)</li></ul> <p><strong>Example:</strong></p> <pre class="language-bash"><!></pre> <h3>FocusBox</h3> <p>Focus a specific box.</p> <pre class="language-json"><!></pre> <p><strong>Parameters:</strong></p> <ul><li><code>box_id</code> (string): ID of the box to focus</li></ul> <p><strong>Example:</strong></p> <pre class="language-bash"><!></pre> <h2>Box Operations</h2> <h3>Getting Box Information</h3> <pre class="language-json"><!></pre> <p><strong>Response:</strong></p> <pre class="language-json"><!></pre> <h3>Listing All Boxes</h3> <pre class="language-json"><!></pre> <p><strong>Response:</strong></p> <pre class="language-json"><!></pre> <h3>Box State Management</h3> <pre class="language-json"><!></pre> <h2>Layout Operations</h2> <h3>Switch Layout</h3> <pre class="language-json"><!></pre> <h3>Get Current Layout</h3> <pre class="language-json"><!></pre> <p><strong>Response:</strong></p> <pre class="language-json"><!></pre> <h3>Reload Configuration</h3> <pre class="language-json"><!></pre> <h2>System Operations</h2> <h3>Get Application Status</h3> <pre class="language-json"><!></pre> <p><strong>Response:</strong></p> <pre class="language-json"><!></pre> <h3>Shutdown Application</h3> <pre class="language-json"><!></pre> <h2>Event Handling</h2> <p>BoxMux can send event notifications for certain activities:</p> <pre class="language-json"><!></pre> <h3>Event Types</h3> <ul><li><code>box_updated</code>: Box content changed</li> <li><code>layout_switched</code>: Active layout changed</li> <li><code>box_focused</code>: Box focus changed</li> <li><code>key_pressed</code>: Key event occurred</li> <li><code>error</code>: Error occurred</li></ul> <h2>Client Libraries</h2> <h3>Python Client</h3> <pre class="language-python"><!></pre> <h3>Shell Script Helper</h3> <pre class="language-bash"><!></pre> <h2>Integration Examples</h2> <h3>CI/CD Integration</h3> <p>Update build status in real-time:</p> <pre class="language-bash"><!></pre> <h3>Monitoring Integration</h3> <pre class="language-python"><!></pre> <h3>Log Monitoring</h3> <pre class="language-bash"><!></pre> <h3>Web Hook Integration</h3> <pre class="language-python"><!></pre> <h2>Error Handling</h2> <h3>Common Error Responses</h3> <pre class="language-json"><!></pre> <h3>Error Codes</h3> <ul><li><code>BOX_NOT_FOUND</code>: Specified box ID doesn’t exist</li> <li><code>LAYOUT_NOT_FOUND</code>: Specified layout ID doesn’t exist</li> <li><code>INVALID_COMMAND</code>: Command format is invalid</li> <li><code>EXECUTION_ERROR</code>: Script execution failed</li> <li><code>PERMISSION_DENIED</code>: Operation not permitted</li> <li><code>SOCKET_ERROR</code>: Communication error</li></ul> <h3>Error Handling Example</h3> <pre class="language-python"><!></pre>',1);function Ns(G){var $=Ts(),t=s(Is($),10),W=n(t);p(W,()=>`<code class="language-bash"><span class="token comment"># Send a command to BoxMux</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"UpdateBox": &#123;"box_id": "status", "content": "Hello World"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock

<span class="token comment"># Check if BoxMux is running</span>
<span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock <span class="token parameter variable">-w</span> <span class="token number">1</span> <span class="token operator">&lt;</span> /dev/null <span class="token operator">&amp;&amp;</span> <span class="token builtin class-name">echo</span> <span class="token string">"BoxMux is running"</span></code>`),a(t);var o=s(t,6),X=n(o);p(X,()=>`<code class="language-bash"><span class="token comment"># Multiple commands</span>
<span class="token punctuation">&#123;</span>
  <span class="token builtin class-name">echo</span> <span class="token string">'&#123;"UpdateBox": &#123;"box_id": "box1", "content": "Line 1"&#125;&#125;'</span>
  <span class="token builtin class-name">echo</span> <span class="token string">'&#123;"UpdateBox": &#123;"box_id": "box2", "content": "Line 2"&#125;&#125;'</span>
<span class="token punctuation">&#125;</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(o);var e=s(o,6),J=n(e);p(J,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"CommandType"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"parameter1"</span><span class="token operator">:</span> <span class="token string">"value1"</span><span class="token punctuation">,</span>
    <span class="token property">"parameter2"</span><span class="token operator">:</span> <span class="token string">"value2"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(e);var c=s(e,6),z=n(c);p(z,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"success"</span><span class="token operator">:</span> <span class="token boolean">true</span><span class="token punctuation">,</span>
  <span class="token property">"message"</span><span class="token operator">:</span> <span class="token string">"Command executed successfully"</span><span class="token punctuation">,</span>
  <span class="token property">"data"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"additional"</span><span class="token operator">:</span> <span class="token string">"information"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(c);var l=s(c,8),V=n(l);p(V,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"UpdateBox"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"box_id"</span><span class="token operator">:</span> <span class="token string">"target_box"</span><span class="token punctuation">,</span>
    <span class="token property">"content"</span><span class="token operator">:</span> <span class="token string">"New content for the box"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(l);var r=s(l,8),Y=n(r);p(Y,()=>`<code class="language-bash"><span class="token builtin class-name">echo</span> <span class="token string">'&#123;"UpdateBox": &#123;"box_id": "status", "content": "System Online"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(r);var u=s(r,6),Z=n(u);p(Z,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"AppendBox"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"box_id"</span><span class="token operator">:</span> <span class="token string">"target_box"</span><span class="token punctuation">,</span>
    <span class="token property">"content"</span><span class="token operator">:</span> <span class="token string">"Content to append"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(u);var i=s(u,8),Q=n(i);p(Q,()=>`<code class="language-bash"><span class="token builtin class-name">echo</span> <span class="token string">'&#123;"AppendBox": &#123;"box_id": "logs", "content": "New log entry"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(i);var k=s(i,6),ss=n(k);p(ss,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"RefreshBox"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"box_id"</span><span class="token operator">:</span> <span class="token string">"target_box"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(k);var d=s(k,8),ns=n(d);p(ns,()=>`<code class="language-bash"><span class="token builtin class-name">echo</span> <span class="token string">'&#123;"RefreshBox": &#123;"box_id": "cpu_monitor"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(d);var g=s(d,6),as=n(g);p(as,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"SetBoxProperty"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"box_id"</span><span class="token operator">:</span> <span class="token string">"target_box"</span><span class="token punctuation">,</span>
    <span class="token property">"property"</span><span class="token operator">:</span> <span class="token string">"bg_color"</span><span class="token punctuation">,</span>
    <span class="token property">"value"</span><span class="token operator">:</span> <span class="token string">"red"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(g);var b=s(g,12),ps=n(b);p(ps,()=>`<code class="language-bash"><span class="token builtin class-name">echo</span> <span class="token string">'&#123;"SetBoxProperty": &#123;"box_id": "alert", "property": "bg_color", "value": "red"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(b);var m=s(b,6),ts=n(m);p(ts,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"ExecuteScript"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"box_id"</span><span class="token operator">:</span> <span class="token string">"target_box"</span><span class="token punctuation">,</span>
    <span class="token property">"script"</span><span class="token operator">:</span> <span class="token punctuation">[</span><span class="token string">"echo 'Hello'"</span><span class="token punctuation">,</span> <span class="token string">"date"</span><span class="token punctuation">]</span><span class="token punctuation">,</span>
    <span class="token property">"append"</span><span class="token operator">:</span> <span class="token boolean">false</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(m);var _=s(m,8),os=n(_);p(os,()=>`<code class="language-bash"><span class="token builtin class-name">echo</span> <span class="token string">'&#123;"ExecuteScript": &#123;"box_id": "output", "script": ["ps aux | head -5"], "append": true&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(_);var y=s(_,6),es=n(y);p(es,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"SendKey"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"key"</span><span class="token operator">:</span> <span class="token string">"Tab"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(y);var x=s(y,8),cs=n(x);p(cs,()=>`<code class="language-bash"><span class="token builtin class-name">echo</span> <span class="token string">'&#123;"SendKey": &#123;"key": "Tab"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(x);var h=s(x,6),ls=n(h);p(ls,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"FocusBox"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"box_id"</span><span class="token operator">:</span> <span class="token string">"target_box"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(h);var f=s(h,8),rs=n(f);p(rs,()=>`<code class="language-bash"><span class="token builtin class-name">echo</span> <span class="token string">'&#123;"FocusBox": &#123;"box_id": "menu"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(f);var v=s(f,6),us=n(v);p(us,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"GetBoxInfo"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"box_id"</span><span class="token operator">:</span> <span class="token string">"target_box"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(v);var w=s(v,4),is=n(w);p(is,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"success"</span><span class="token operator">:</span> <span class="token boolean">true</span><span class="token punctuation">,</span>
  <span class="token property">"data"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"id"</span><span class="token operator">:</span> <span class="token string">"target_box"</span><span class="token punctuation">,</span>
    <span class="token property">"title"</span><span class="token operator">:</span> <span class="token string">"Box Title"</span><span class="token punctuation">,</span>
    <span class="token property">"content"</span><span class="token operator">:</span> <span class="token string">"Current content"</span><span class="token punctuation">,</span>
    <span class="token property">"position"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span><span class="token property">"x1"</span><span class="token operator">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token property">"y1"</span><span class="token operator">:</span> <span class="token string">"10%"</span><span class="token punctuation">,</span> <span class="token property">"x2"</span><span class="token operator">:</span> <span class="token string">"90%"</span><span class="token punctuation">,</span> <span class="token property">"y2"</span><span class="token operator">:</span> <span class="token string">"90%"</span><span class="token punctuation">&#125;</span><span class="token punctuation">,</span>
    <span class="token property">"properties"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
      <span class="token property">"bg_color"</span><span class="token operator">:</span> <span class="token string">"black"</span><span class="token punctuation">,</span>
      <span class="token property">"fg_color"</span><span class="token operator">:</span> <span class="token string">"white"</span><span class="token punctuation">,</span>
      <span class="token property">"refresh_interval"</span><span class="token operator">:</span> <span class="token number">5000</span>
    <span class="token punctuation">&#125;</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(w);var B=s(w,4),ks=n(B);p(ks,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"ListBoxes"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span><span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(B);var j=s(B,4),ds=n(j);p(ds,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"success"</span><span class="token operator">:</span> <span class="token boolean">true</span><span class="token punctuation">,</span>
  <span class="token property">"data"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"boxes"</span><span class="token operator">:</span> <span class="token punctuation">[</span>
      <span class="token punctuation">&#123;</span>
        <span class="token property">"id"</span><span class="token operator">:</span> <span class="token string">"box1"</span><span class="token punctuation">,</span>
        <span class="token property">"title"</span><span class="token operator">:</span> <span class="token string">"Box 1"</span><span class="token punctuation">,</span>
        <span class="token property">"layout"</span><span class="token operator">:</span> <span class="token string">"main"</span>
      <span class="token punctuation">&#125;</span><span class="token punctuation">,</span>
      <span class="token punctuation">&#123;</span>
        <span class="token property">"id"</span><span class="token operator">:</span> <span class="token string">"box2"</span><span class="token punctuation">,</span> 
        <span class="token property">"title"</span><span class="token operator">:</span> <span class="token string">"Box 2"</span><span class="token punctuation">,</span>
        <span class="token property">"layout"</span><span class="token operator">:</span> <span class="token string">"main"</span>
      <span class="token punctuation">&#125;</span>
    <span class="token punctuation">]</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(j);var S=s(j,4),gs=n(S);p(gs,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"SetBoxState"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"box_id"</span><span class="token operator">:</span> <span class="token string">"target_box"</span><span class="token punctuation">,</span>
    <span class="token property">"state"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
      <span class="token property">"visible"</span><span class="token operator">:</span> <span class="token boolean">true</span><span class="token punctuation">,</span>
      <span class="token property">"enabled"</span><span class="token operator">:</span> <span class="token boolean">true</span><span class="token punctuation">,</span>
      <span class="token property">"selected"</span><span class="token operator">:</span> <span class="token boolean">false</span>
    <span class="token punctuation">&#125;</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(S);var E=s(S,6),bs=n(E);p(bs,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"SwitchLayout"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"layout_id"</span><span class="token operator">:</span> <span class="token string">"new_layout"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(E);var C=s(E,4),ms=n(C);p(ms,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"GetCurrentLayout"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span><span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(C);var U=s(C,4),_s=n(U);p(_s,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"success"</span><span class="token operator">:</span> <span class="token boolean">true</span><span class="token punctuation">,</span>
  <span class="token property">"data"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"layout_id"</span><span class="token operator">:</span> <span class="token string">"current_layout"</span><span class="token punctuation">,</span>
    <span class="token property">"title"</span><span class="token operator">:</span> <span class="token string">"Current Layout Title"</span><span class="token punctuation">,</span>
    <span class="token property">"boxes"</span><span class="token operator">:</span> <span class="token punctuation">[</span><span class="token string">"box1"</span><span class="token punctuation">,</span> <span class="token string">"box2"</span><span class="token punctuation">,</span> <span class="token string">"box3"</span><span class="token punctuation">]</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(U);var O=s(U,4),ys=n(O);p(ys,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"ReloadConfig"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"config_file"</span><span class="token operator">:</span> <span class="token string">"/path/to/new/config.yaml"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(O);var R=s(O,6),xs=n(R);p(xs,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"GetStatus"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span><span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(R);var I=s(R,4),hs=n(I);p(hs,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"success"</span><span class="token operator">:</span> <span class="token boolean">true</span><span class="token punctuation">,</span>
  <span class="token property">"data"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"version"</span><span class="token operator">:</span> <span class="token string">"0.76.71205"</span><span class="token punctuation">,</span>
    <span class="token property">"uptime"</span><span class="token operator">:</span> <span class="token string">"00:15:32"</span><span class="token punctuation">,</span>
    <span class="token property">"current_layout"</span><span class="token operator">:</span> <span class="token string">"dashboard"</span><span class="token punctuation">,</span>
    <span class="token property">"active_boxes"</span><span class="token operator">:</span> <span class="token number">5</span><span class="token punctuation">,</span>
    <span class="token property">"socket_path"</span><span class="token operator">:</span> <span class="token string">"/tmp/boxmux.sock"</span><span class="token punctuation">,</span>
    <span class="token property">"config_file"</span><span class="token operator">:</span> <span class="token string">"/path/to/config.yaml"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(I);var M=s(I,4),fs=n(M);p(fs,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"Shutdown"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span><span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(M);var T=s(M,6),vs=n(T);p(vs,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"event"</span><span class="token operator">:</span> <span class="token string">"box_updated"</span><span class="token punctuation">,</span>
  <span class="token property">"data"</span><span class="token operator">:</span> <span class="token punctuation">&#123;</span>
    <span class="token property">"box_id"</span><span class="token operator">:</span> <span class="token string">"status"</span><span class="token punctuation">,</span>
    <span class="token property">"timestamp"</span><span class="token operator">:</span> <span class="token string">"2023-01-01T12:00:00Z"</span><span class="token punctuation">,</span>
    <span class="token property">"content"</span><span class="token operator">:</span> <span class="token string">"New content"</span>
  <span class="token punctuation">&#125;</span>
<span class="token punctuation">&#125;</span></code>`),a(T);var N=s(T,10),ws=n(N);p(ws,()=>`<code class="language-python"><span class="token keyword">import</span> socket
<span class="token keyword">import</span> json

<span class="token keyword">class</span> <span class="token class-name">BoxMuxClient</span><span class="token punctuation">:</span>
    <span class="token keyword">def</span> <span class="token function">__init__</span><span class="token punctuation">(</span>self<span class="token punctuation">,</span> socket_path<span class="token operator">=</span><span class="token string">'/tmp/boxmux.sock'</span><span class="token punctuation">)</span><span class="token punctuation">:</span>
        self<span class="token punctuation">.</span>socket_path <span class="token operator">=</span> socket_path
    
    <span class="token keyword">def</span> <span class="token function">send_command</span><span class="token punctuation">(</span>self<span class="token punctuation">,</span> command<span class="token punctuation">)</span><span class="token punctuation">:</span>
        sock <span class="token operator">=</span> socket<span class="token punctuation">.</span>socket<span class="token punctuation">(</span>socket<span class="token punctuation">.</span>AF_UNIX<span class="token punctuation">,</span> socket<span class="token punctuation">.</span>SOCK_STREAM<span class="token punctuation">)</span>
        <span class="token keyword">try</span><span class="token punctuation">:</span>
            sock<span class="token punctuation">.</span>connect<span class="token punctuation">(</span>self<span class="token punctuation">.</span>socket_path<span class="token punctuation">)</span>
            sock<span class="token punctuation">.</span>send<span class="token punctuation">(</span><span class="token punctuation">(</span>json<span class="token punctuation">.</span>dumps<span class="token punctuation">(</span>command<span class="token punctuation">)</span> <span class="token operator">+</span> <span class="token string">'&#92;n'</span><span class="token punctuation">)</span><span class="token punctuation">.</span>encode<span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">)</span>
            response <span class="token operator">=</span> sock<span class="token punctuation">.</span>recv<span class="token punctuation">(</span><span class="token number">4096</span><span class="token punctuation">)</span><span class="token punctuation">.</span>decode<span class="token punctuation">(</span><span class="token punctuation">)</span>
            <span class="token keyword">return</span> json<span class="token punctuation">.</span>loads<span class="token punctuation">(</span>response<span class="token punctuation">)</span> <span class="token keyword">if</span> response <span class="token keyword">else</span> <span class="token boolean">None</span>
        <span class="token keyword">finally</span><span class="token punctuation">:</span>
            sock<span class="token punctuation">.</span>close<span class="token punctuation">(</span><span class="token punctuation">)</span>
    
    <span class="token keyword">def</span> <span class="token function">update_box</span><span class="token punctuation">(</span>self<span class="token punctuation">,</span> box_id<span class="token punctuation">,</span> content<span class="token punctuation">)</span><span class="token punctuation">:</span>
        <span class="token keyword">return</span> self<span class="token punctuation">.</span>send_command<span class="token punctuation">(</span><span class="token punctuation">&#123;</span>
            <span class="token string">"UpdateBox"</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span>
                <span class="token string">"box_id"</span><span class="token punctuation">:</span> box_id<span class="token punctuation">,</span>
                <span class="token string">"content"</span><span class="token punctuation">:</span> content
            <span class="token punctuation">&#125;</span>
        <span class="token punctuation">&#125;</span><span class="token punctuation">)</span>
    
    <span class="token keyword">def</span> <span class="token function">refresh_box</span><span class="token punctuation">(</span>self<span class="token punctuation">,</span> box_id<span class="token punctuation">)</span><span class="token punctuation">:</span>
        <span class="token keyword">return</span> self<span class="token punctuation">.</span>send_command<span class="token punctuation">(</span><span class="token punctuation">&#123;</span>
            <span class="token string">"RefreshBox"</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token string">"box_id"</span><span class="token punctuation">:</span> box_id<span class="token punctuation">&#125;</span>
        <span class="token punctuation">&#125;</span><span class="token punctuation">)</span>

<span class="token comment"># Usage</span>
client <span class="token operator">=</span> BoxMuxClient<span class="token punctuation">(</span><span class="token punctuation">)</span>
client<span class="token punctuation">.</span>update_box<span class="token punctuation">(</span><span class="token string">"status"</span><span class="token punctuation">,</span> <span class="token string">"System Online"</span><span class="token punctuation">)</span></code>`),a(N);var P=s(N,4),Bs=n(P);p(Bs,()=>`<code class="language-bash"><span class="token shebang important">#!/bin/bash</span>

<span class="token assign-left variable">SOCKET_PATH</span><span class="token operator">=</span><span class="token string">"/tmp/boxmux.sock"</span>

<span class="token function-name function">boxmux_cmd</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token punctuation">&#123;</span>
    <span class="token builtin class-name">echo</span> <span class="token string">"<span class="token variable">$1</span>"</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> <span class="token string">"<span class="token variable">$SOCKET_PATH</span>"</span>
<span class="token punctuation">&#125;</span>

<span class="token function-name function">update_box</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token punctuation">&#123;</span>
    <span class="token builtin class-name">local</span> <span class="token assign-left variable">box_id</span><span class="token operator">=</span><span class="token string">"<span class="token variable">$1</span>"</span>
    <span class="token builtin class-name">local</span> <span class="token assign-left variable">content</span><span class="token operator">=</span><span class="token string">"<span class="token variable">$2</span>"</span>
    boxmux_cmd <span class="token string">"&#123;<span class="token entity" title="&quot;">"</span>UpdateBox<span class="token entity" title="&quot;">"</span>: &#123;<span class="token entity" title="&quot;">"</span>box_id<span class="token entity" title="&quot;">"</span>: <span class="token entity" title="&quot;">"</span><span class="token variable">$box_id</span><span class="token entity" title="&quot;">"</span>, <span class="token entity" title="&quot;">"</span>content<span class="token entity" title="&quot;">"</span>: <span class="token entity" title="&quot;">"</span><span class="token variable">$content</span><span class="token entity" title="&quot;">"</span>&#125;&#125;"</span>
<span class="token punctuation">&#125;</span>

<span class="token function-name function">refresh_box</span><span class="token punctuation">(</span><span class="token punctuation">)</span> <span class="token punctuation">&#123;</span>
    <span class="token builtin class-name">local</span> <span class="token assign-left variable">box_id</span><span class="token operator">=</span><span class="token string">"<span class="token variable">$1</span>"</span>
    boxmux_cmd <span class="token string">"&#123;<span class="token entity" title="&quot;">"</span>RefreshBox<span class="token entity" title="&quot;">"</span>: &#123;<span class="token entity" title="&quot;">"</span>box_id<span class="token entity" title="&quot;">"</span>: <span class="token entity" title="&quot;">"</span><span class="token variable">$box_id</span><span class="token entity" title="&quot;">"</span>&#125;&#125;"</span>
<span class="token punctuation">&#125;</span>

<span class="token comment"># Usage</span>
update_box <span class="token string">"status"</span> <span class="token string">"System Running"</span>
refresh_box <span class="token string">"cpu_monitor"</span></code>`),a(P);var A=s(P,8),js=n(A);p(js,()=>`<code class="language-bash"><span class="token shebang important">#!/bin/bash</span>
<span class="token comment"># build.sh</span>

<span class="token comment"># Start build</span>
update_box <span class="token string">"build_status"</span> <span class="token string">"🔄 Building..."</span>

<span class="token comment"># Run build</span>
<span class="token keyword">if</span> <span class="token function">cargo</span> build --release<span class="token punctuation">;</span> <span class="token keyword">then</span>
    update_box <span class="token string">"build_status"</span> <span class="token string">"✅ Build successful"</span>
    update_box <span class="token string">"build_output"</span> <span class="token string">"<span class="token variable"><span class="token variable">$(</span><span class="token function">cargo</span> build <span class="token parameter variable">--release</span> <span class="token operator"><span class="token file-descriptor important">2</span>></span><span class="token file-descriptor important">&amp;1</span> <span class="token operator">|</span> <span class="token function">tail</span> <span class="token parameter variable">-10</span><span class="token variable">)</span></span>"</span>
<span class="token keyword">else</span>
    update_box <span class="token string">"build_status"</span> <span class="token string">"❌ Build failed"</span>
    update_box <span class="token string">"build_output"</span> <span class="token string">"<span class="token variable"><span class="token variable">$(</span><span class="token function">cargo</span> build <span class="token parameter variable">--release</span> <span class="token operator"><span class="token file-descriptor important">2</span>></span><span class="token file-descriptor important">&amp;1</span> <span class="token operator">|</span> <span class="token function">tail</span> <span class="token parameter variable">-10</span><span class="token variable">)</span></span>"</span>
<span class="token keyword">fi</span></code>`),a(A);var D=s(A,4),Ss=n(D);p(Ss,()=>`<code class="language-python"><span class="token keyword">import</span> psutil
<span class="token keyword">import</span> time

<span class="token keyword">def</span> <span class="token function">update_system_metrics</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">:</span>
    client <span class="token operator">=</span> BoxMuxClient<span class="token punctuation">(</span><span class="token punctuation">)</span>
    
    <span class="token keyword">while</span> <span class="token boolean">True</span><span class="token punctuation">:</span>
        <span class="token comment"># CPU usage</span>
        cpu <span class="token operator">=</span> psutil<span class="token punctuation">.</span>cpu_percent<span class="token punctuation">(</span>interval<span class="token operator">=</span><span class="token number">1</span><span class="token punctuation">)</span>
        client<span class="token punctuation">.</span>update_box<span class="token punctuation">(</span><span class="token string">"cpu"</span><span class="token punctuation">,</span> <span class="token string-interpolation"><span class="token string">f"CPU: </span><span class="token interpolation"><span class="token punctuation">&#123;</span>cpu<span class="token punctuation">&#125;</span></span><span class="token string">%"</span></span><span class="token punctuation">)</span>
        
        <span class="token comment"># Memory usage</span>
        memory <span class="token operator">=</span> psutil<span class="token punctuation">.</span>virtual_memory<span class="token punctuation">(</span><span class="token punctuation">)</span>
        client<span class="token punctuation">.</span>update_box<span class="token punctuation">(</span><span class="token string">"memory"</span><span class="token punctuation">,</span> <span class="token string-interpolation"><span class="token string">f"Memory: </span><span class="token interpolation"><span class="token punctuation">&#123;</span>memory<span class="token punctuation">.</span>percent<span class="token punctuation">&#125;</span></span><span class="token string">%"</span></span><span class="token punctuation">)</span>
        
        <span class="token comment"># Disk usage</span>
        disk <span class="token operator">=</span> psutil<span class="token punctuation">.</span>disk_usage<span class="token punctuation">(</span><span class="token string">'/'</span><span class="token punctuation">)</span>
        client<span class="token punctuation">.</span>update_box<span class="token punctuation">(</span><span class="token string">"disk"</span><span class="token punctuation">,</span> <span class="token string-interpolation"><span class="token string">f"Disk: </span><span class="token interpolation"><span class="token punctuation">&#123;</span>disk<span class="token punctuation">.</span>percent<span class="token punctuation">&#125;</span></span><span class="token string">%"</span></span><span class="token punctuation">)</span>
        
        time<span class="token punctuation">.</span>sleep<span class="token punctuation">(</span><span class="token number">5</span><span class="token punctuation">)</span></code>`),a(D);var q=s(D,4),Es=n(q);p(Es,()=>`<code class="language-bash"><span class="token shebang important">#!/bin/bash</span>
<span class="token comment"># Monitor log file and update BoxMux</span>

<span class="token function">tail</span> <span class="token parameter variable">-f</span> /var/log/application.log <span class="token operator">|</span> <span class="token keyword">while</span> <span class="token builtin class-name">read</span> line<span class="token punctuation">;</span> <span class="token keyword">do</span>
    <span class="token keyword">if</span> <span class="token punctuation">[</span><span class="token punctuation">[</span> <span class="token string">"<span class="token variable">$line</span>"</span> <span class="token operator">==</span> *<span class="token string">"ERROR"</span>* <span class="token punctuation">]</span><span class="token punctuation">]</span><span class="token punctuation">;</span> <span class="token keyword">then</span>
        update_box <span class="token string">"log_status"</span> <span class="token string">"❌ Error detected"</span>
        update_box <span class="token string">"last_error"</span> <span class="token string">"<span class="token variable">$line</span>"</span>
    <span class="token keyword">elif</span> <span class="token punctuation">[</span><span class="token punctuation">[</span> <span class="token string">"<span class="token variable">$line</span>"</span> <span class="token operator">==</span> *<span class="token string">"WARNING"</span>* <span class="token punctuation">]</span><span class="token punctuation">]</span><span class="token punctuation">;</span> <span class="token keyword">then</span>
        update_box <span class="token string">"log_status"</span> <span class="token string">"⚠️ Warning"</span>
    <span class="token keyword">else</span>
        update_box <span class="token string">"log_status"</span> <span class="token string">"✅ Normal"</span>
    <span class="token keyword">fi</span>
    
    <span class="token comment"># Update recent logs (last 10 lines)</span>
    <span class="token function">tail</span> <span class="token parameter variable">-10</span> /var/log/application.log <span class="token operator">|</span> boxmux_cmd <span class="token string">'&#123;"UpdateBox": &#123;"box_id": "recent_logs", "content": "'</span><span class="token variable"><span class="token variable">$(</span><span class="token function">cat</span><span class="token variable">)</span></span><span class="token string">'"&#125;&#125;'</span> 
<span class="token keyword">done</span></code>`),a(q);var L=s(q,4),Cs=n(L);p(Cs,()=>`<code class="language-python"><span class="token keyword">from</span> flask <span class="token keyword">import</span> Flask<span class="token punctuation">,</span> request
<span class="token keyword">import</span> json

app <span class="token operator">=</span> Flask<span class="token punctuation">(</span>__name__<span class="token punctuation">)</span>
client <span class="token operator">=</span> BoxMuxClient<span class="token punctuation">(</span><span class="token punctuation">)</span>

<span class="token decorator annotation punctuation">@app<span class="token punctuation">.</span>route</span><span class="token punctuation">(</span><span class="token string">'/webhook'</span><span class="token punctuation">,</span> methods<span class="token operator">=</span><span class="token punctuation">[</span><span class="token string">'POST'</span><span class="token punctuation">]</span><span class="token punctuation">)</span>
<span class="token keyword">def</span> <span class="token function">handle_webhook</span><span class="token punctuation">(</span><span class="token punctuation">)</span><span class="token punctuation">:</span>
    data <span class="token operator">=</span> request<span class="token punctuation">.</span>json
    
    <span class="token keyword">if</span> data<span class="token punctuation">.</span>get<span class="token punctuation">(</span><span class="token string">'type'</span><span class="token punctuation">)</span> <span class="token operator">==</span> <span class="token string">'deployment'</span><span class="token punctuation">:</span>
        status <span class="token operator">=</span> data<span class="token punctuation">.</span>get<span class="token punctuation">(</span><span class="token string">'status'</span><span class="token punctuation">)</span>
        <span class="token keyword">if</span> status <span class="token operator">==</span> <span class="token string">'success'</span><span class="token punctuation">:</span>
            client<span class="token punctuation">.</span>update_box<span class="token punctuation">(</span><span class="token string">"deploy_status"</span><span class="token punctuation">,</span> <span class="token string">"✅ Deployment successful"</span><span class="token punctuation">)</span>
        <span class="token keyword">elif</span> status <span class="token operator">==</span> <span class="token string">'failed'</span><span class="token punctuation">:</span>
            client<span class="token punctuation">.</span>update_box<span class="token punctuation">(</span><span class="token string">"deploy_status"</span><span class="token punctuation">,</span> <span class="token string">"❌ Deployment failed"</span><span class="token punctuation">)</span>
        <span class="token keyword">elif</span> status <span class="token operator">==</span> <span class="token string">'started'</span><span class="token punctuation">:</span>
            client<span class="token punctuation">.</span>update_box<span class="token punctuation">(</span><span class="token string">"deploy_status"</span><span class="token punctuation">,</span> <span class="token string">"🔄 Deploying..."</span><span class="token punctuation">)</span>
    
    <span class="token keyword">return</span> <span class="token punctuation">&#123;</span><span class="token string">'status'</span><span class="token punctuation">:</span> <span class="token string">'ok'</span><span class="token punctuation">&#125;</span>

<span class="token keyword">if</span> __name__ <span class="token operator">==</span> <span class="token string">'__main__'</span><span class="token punctuation">:</span>
    app<span class="token punctuation">.</span>run<span class="token punctuation">(</span>host<span class="token operator">=</span><span class="token string">'0.0.0.0'</span><span class="token punctuation">,</span> port<span class="token operator">=</span><span class="token number">5000</span><span class="token punctuation">)</span></code>`),a(L);var F=s(L,6),Us=n(F);p(Us,()=>`<code class="language-json"><span class="token punctuation">&#123;</span>
  <span class="token property">"success"</span><span class="token operator">:</span> <span class="token boolean">false</span><span class="token punctuation">,</span>
  <span class="token property">"error"</span><span class="token operator">:</span> <span class="token string">"Box not found"</span><span class="token punctuation">,</span>
  <span class="token property">"error_code"</span><span class="token operator">:</span> <span class="token string">"BOX_NOT_FOUND"</span>
<span class="token punctuation">&#125;</span></code>`),a(F);var H=s(F,8),Os=n(H);p(Os,()=>`<code class="language-python"><span class="token keyword">def</span> <span class="token function">safe_update_box</span><span class="token punctuation">(</span>box_id<span class="token punctuation">,</span> content<span class="token punctuation">)</span><span class="token punctuation">:</span>
    <span class="token keyword">try</span><span class="token punctuation">:</span>
        response <span class="token operator">=</span> client<span class="token punctuation">.</span>update_box<span class="token punctuation">(</span>box_id<span class="token punctuation">,</span> content<span class="token punctuation">)</span>
        <span class="token keyword">if</span> <span class="token keyword">not</span> response<span class="token punctuation">.</span>get<span class="token punctuation">(</span><span class="token string">'success'</span><span class="token punctuation">)</span><span class="token punctuation">:</span>
            <span class="token keyword">print</span><span class="token punctuation">(</span><span class="token string-interpolation"><span class="token string">f"Error: </span><span class="token interpolation"><span class="token punctuation">&#123;</span>response<span class="token punctuation">.</span>get<span class="token punctuation">(</span><span class="token string">'error'</span><span class="token punctuation">)</span><span class="token punctuation">&#125;</span></span><span class="token string">"</span></span><span class="token punctuation">)</span>
            <span class="token keyword">return</span> <span class="token boolean">False</span>
        <span class="token keyword">return</span> <span class="token boolean">True</span>
    <span class="token keyword">except</span> Exception <span class="token keyword">as</span> e<span class="token punctuation">:</span>
        <span class="token keyword">print</span><span class="token punctuation">(</span><span class="token string-interpolation"><span class="token string">f"Connection error: </span><span class="token interpolation"><span class="token punctuation">&#123;</span>e<span class="token punctuation">&#125;</span></span><span class="token string">"</span></span><span class="token punctuation">)</span>
        <span class="token keyword">return</span> <span class="token boolean">False</span></code>`),a(H),Ms(G,$)}const $s=Object.freeze(Object.defineProperty({__proto__:null,default:Ns,metadata:K},Symbol.toStringTag,{value:"Module"}));export{$s as _};
