import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as I,s,a as M,b as U,d as a,r as n}from"./E9AOERj2.js";import{h as e}from"./Cc2oKZLy.js";const k={title:"Troubleshooting Guide",description:"Comprehensive troubleshooting guide for BoxMux - common issues, debugging techniques, performance problems, and recovery procedures"},{title:F,description:H}=k;var w=I('<h2>Common Issues</h2> <h3>Installation Problems</h3> <h4>Rust Not Found</h4> <p><strong>Problem</strong>: <code>cargo: command not found</code><br/> <strong>Solution</strong>:</p> <pre class="language-bash"><!></pre> <h4>Build Failures</h4> <p><strong>Problem</strong>: Compilation errors during <code>cargo build</code><br/> <strong>Solution</strong>:</p> <pre class="language-bash"><!></pre> <h3>Runtime Issues</h3> <h4>Configuration File Not Found</h4> <p><strong>Problem</strong>: <code>Error: Configuration file not found</code><br/> <strong>Solution</strong>:</p> <ul><li>Verify file path: <code>ls -la layouts/dashboard.yaml</code></li> <li>Use absolute path if installed via cargo: <code>boxmux /full/path/to/config.yaml</code></li> <li>Use absolute path if built from source: <code>./run_boxmux.sh /full/path/to/config.yaml</code></li> <li>Check file permissions: <code>chmod 644 layouts/dashboard.yaml</code></li></ul> <h4>YAML Syntax Errors</h4> <p><strong>Problem</strong>: <code>Error parsing YAML</code><br/> <strong>Solution</strong>:</p> <pre class="language-bash"><!></pre> <h4>Script Execution Fails</h4> <p><strong>Problem</strong>: Box scripts don’t execute<br/> <strong>Solution</strong>:</p> <ul><li>Check script permissions: <code>chmod +x script.sh</code></li> <li>Verify shell path: Use absolute paths like <code>/bin/bash</code></li> <li>Test script independently: <code>bash -c "your_script_here"</code></li></ul> <h3>Performance Issues</h3> <h4>High CPU Usage</h4> <p><strong>Problem</strong>: BoxMux uses too much CPU<br/> <strong>Solution</strong>:</p> <ul><li>Increase refresh intervals in configuration</li> <li>Optimize scripts to run faster</li> <li>Reduce number of boxes with scripts</li> <li>Check for infinite loops in scripts</li></ul> <h3>Display Issues</h3> <h4>Corrupted Display</h4> <p><strong>Problem</strong>: Interface appears broken or corrupted<br/> <strong>Solution</strong>:</p> <ul><li>Clear terminal: <code>clear</code> or <code>Ctrl+L</code></li> <li>Resize terminal window</li> <li>Restart BoxMux</li> <li>Check terminal compatibility</li></ul> <h4>Colors Not Working</h4> <p><strong>Problem</strong>: Colors appear incorrect or missing<br/> <strong>Solution</strong>:</p> <ul><li>Check terminal color support: <code>echo $TERM</code></li> <li>Use standard color names</li> <li>Test with different terminal emulators</li> <li>Check terminal theme settings</li></ul> <h2>Debugging</h2> <h3>Enable Debug Logging</h3> <pre class="language-bash"><!></pre> <h3>Check Application Logs</h3> <pre class="language-bash"><!></pre> <h3>Validate Configuration</h3> <pre class="language-bash"><!></pre> <h3>Test Individual Components</h3> <pre class="language-bash"><!></pre> <h2>Getting Help</h2> <h3>Before Asking for Help</h3> <ol><li><strong>Check this troubleshooting guide</strong></li> <li><strong>Search existing GitHub issues</strong></li> <li><strong>Try the solution with minimal configuration</strong></li> <li><strong>Gather relevant information</strong></li></ol> <h3>Information to Include</h3> <p>When reporting issues, include:</p> <ul><li>BoxMux version: <code>cargo --version</code></li> <li>Operating system: <code>uname -a</code></li> <li>Terminal emulator and version</li> <li>Configuration file (if relevant)</li> <li>Steps to reproduce the issue</li> <li>Expected vs. actual behavior</li> <li>Error messages and logs</li></ul> <h3>Where to Get Help</h3> <ul><li><strong>GitHub Issues</strong>: Bug reports and feature requests</li> <li><strong>Documentation</strong>: Check all docs files</li> <li><strong>Examples</strong>: Review example configurations</li></ul> <h3>Creating a Minimal Example</h3> <p>When reporting issues, create a minimal configuration that reproduces the problem:</p> <pre class="language-yaml"><!></pre> <h2>Recovery Procedures</h2> <h3>Reset Configuration</h3> <pre class="language-bash"><!></pre> <h3>Clear Application State</h3> <pre class="language-bash"><!></pre> <h3>Emergency Stop</h3> <pre class="language-bash"><!></pre>',1);function B(b){var d=w(),o=s(M(d),8),f=a(o);e(f,()=>`<code class="language-bash"><span class="token comment"># Install Rust</span>
<span class="token function">curl</span> <span class="token parameter variable">--proto</span> <span class="token string">'=https'</span> <span class="token parameter variable">--tlsv1.2</span> <span class="token parameter variable">-sSf</span> https://sh.rustup.rs <span class="token operator">|</span> <span class="token function">sh</span>
<span class="token builtin class-name">source</span> ~/.cargo/env</code>`),n(o);var t=s(o,6),y=a(t);e(y,()=>`<code class="language-bash"><span class="token comment"># Update Rust</span>
rustup update

<span class="token comment"># Clean and rebuild</span>
<span class="token function">cargo</span> clean
<span class="token function">cargo</span> build</code>`),n(t);var l=s(t,14),v=a(l);e(v,()=>`<code class="language-bash"><span class="token comment"># Validate YAML syntax</span>
yamllint layouts/dashboard.yaml

<span class="token comment"># Common fixes:</span>
<span class="token comment"># - Check indentation (use spaces, not tabs)</span>
<span class="token comment"># - Ensure proper quoting of strings</span>
<span class="token comment"># - Verify array and object syntax</span></code>`),n(l);var r=s(l,34),x=a(r);e(x,()=>`<code class="language-bash"><span class="token comment"># Run with debug output</span>
<span class="token assign-left variable">RUST_LOG</span><span class="token operator">=</span>debug ./run_boxmux.sh layouts/dashboard.yaml

<span class="token comment"># Filter specific modules</span>
<span class="token assign-left variable">RUST_LOG</span><span class="token operator">=</span>boxmux::draw_utils<span class="token operator">=</span>debug ./run_boxmux.sh layouts/dashboard.yaml</code>`),n(r);var p=s(r,4),_=a(p);e(_,()=>`<code class="language-bash"><span class="token comment"># View application logs</span>
<span class="token function">tail</span> <span class="token parameter variable">-f</span> app.log

<span class="token comment"># Search for errors</span>
<span class="token function">grep</span> <span class="token parameter variable">-i</span> error app.log</code>`),n(p);var c=s(p,4),C=a(c);e(C,()=>`<code class="language-bash"><span class="token comment"># Check YAML syntax</span>
python <span class="token parameter variable">-c</span> <span class="token string">"import yaml; yaml.safe_load(open('layouts/dashboard.yaml'))"</span>

<span class="token comment"># Or use online validator</span>
<span class="token function">cat</span> layouts/dashboard.yaml <span class="token operator">|</span> <span class="token function">curl</span> <span class="token parameter variable">-X</span> POST <span class="token parameter variable">-H</span> <span class="token string">"Content-Type: application/yaml"</span> <span class="token parameter variable">-d</span> @- https://yaml-validator.com/</code>`),n(c);var i=s(c,4),S=a(i);e(S,()=>`<code class="language-bash"><span class="token comment"># Test script execution</span>
<span class="token function">bash</span> <span class="token parameter variable">-c</span> <span class="token string">"your_script_command_here"</span>

<span class="token comment"># Test socket connection</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"GetStatus": &#123;&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),n(i);var u=s(i,22),P=a(u);e(P,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'test'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'problem_box'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Problem Box'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span>
            <span class="token key atrule">x1</span><span class="token punctuation">:</span> 10%
            <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%
            <span class="token key atrule">x2</span><span class="token punctuation">:</span> 90%
            <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%
          <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'This demonstrates the issue'</span></code>`),n(u);var m=s(u,6),T=a(m);e(T,()=>`<code class="language-bash"><span class="token comment"># Backup current configuration</span>
<span class="token function">cp</span> layouts/dashboard.yaml layouts/dashboard.yaml.backup

<span class="token comment"># Use example configuration</span>
<span class="token function">cp</span> layouts/dashboard.yaml.example layouts/dashboard.yaml</code>`),n(m);var g=s(m,4),R=a(g);e(R,()=>`<code class="language-bash"><span class="token comment"># Remove temporary files</span>
<span class="token function">rm</span> <span class="token parameter variable">-f</span> /tmp/boxmux.sock
<span class="token function">rm</span> <span class="token parameter variable">-f</span> app.log

<span class="token comment"># Clear terminal</span>
<span class="token function">clear</span></code>`),n(g);var h=s(g,4),E=a(h);e(E,()=>`<code class="language-bash"><span class="token comment"># Force quit BoxMux</span>
<span class="token function">pkill</span> <span class="token parameter variable">-f</span> boxmux

<span class="token comment"># Or use Ctrl+C in terminal</span>
<span class="token comment"># Or close terminal window</span></code>`),n(h),U(b,d)}const V=Object.freeze(Object.defineProperty({__proto__:null,default:B,metadata:k},Symbol.toStringTag,{value:"Module"}));export{V as _};
