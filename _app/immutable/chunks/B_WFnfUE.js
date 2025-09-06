import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as Rn,s as n,a as Ln,b as Dn,d as s,r as a,n as In}from"./E9AOERj2.js";import{h as t}from"./Cc2oKZLy.js";const $={title:"User Guide",description:"Complete guide for building terminal interfaces with BoxMux - from basic concepts to advanced techniques"},{title:Nn,description:Yn}=$;var Bn=Rn('<h2>Table of Contents</h2> <ul><li><a href="#quick-start">Quick Start</a></li> <li><a href="#core-concepts">Core Concepts</a></li> <li><a href="#stream-architecture">Stream Architecture</a></li> <li><a href="#interactive-ui-features">Interactive UI Features</a></li> <li><a href="#building-interfaces">Building Interfaces</a></li> <li><a href="#common-patterns">Common Patterns</a></li> <li><a href="#pty-features">PTY Features</a></li> <li><a href="#data-visualization">Data Visualization</a></li> <li><a href="#plugin-system">Plugin System</a></li> <li><a href="#real-world-examples">Real-World Examples</a></li> <li><a href="#best-practices">Best Practices</a></li> <li><a href="#techniques">Techniques</a></li></ul> <h2>Quick Start</h2> <h3>Your First Interface</h3> <p>Create a simple “Hello World” interface:</p> <pre class="language-yaml"><!></pre> <p>Run it:</p> <pre class="language-bash"><!></pre> <h3>Interactive Menu</h3> <p>Add interactivity with menus:</p> <pre class="language-yaml"><!></pre> <h2>Core Concepts</h2> <h3>1. Layouts and Boxes</h3> <p><strong>Layouts</strong> define the overall structure of your interface. <strong>Boxes</strong> are the building blocks within layouts.</p> <pre class="language-yaml"><!></pre> <h3>2. Positioning System</h3> <p>BoxMux uses percentage-based positioning for responsive design:</p> <ul><li><code>x1, y1</code>: Top-left corner coordinates</li> <li><code>x2, y2</code>: Bottom-right corner coordinates</li> <li>Values are percentages of the parent container</li></ul> <pre class="language-yaml"><!></pre> <h3>3. Interactive Elements</h3> <p>Create interactive menus with choices:</p> <pre class="language-yaml"><!></pre> <h3>4. Real-time Updates</h3> <p>Add live data with refresh intervals:</p> <pre class="language-yaml"><!></pre> <h3>5. Variable System - Dynamic Configuration</h3> <p>BoxMux’s variable system enables template-driven interfaces with environment-specific configuration:</p> <h4>Basic Variable Usage</h4> <pre class="language-yaml"><!></pre> <h4>Environment Integration</h4> <p>Variables seamlessly integrate with environment variables:</p> <pre class="language-yaml"><!></pre> <h4>Hierarchical Variables</h4> <p>Child boxes inherit and can override parent variables:</p> <pre class="language-yaml"><!></pre> <p><strong>Learn more</strong>: See the complete <a href="/docs/variables">Variable System Guide</a> for additional patterns and best practices.</p> <h2>Stream Architecture</h2> <p>BoxMux uses a <strong>stream-based architecture</strong> where boxes can display content from multiple input streams, each appearing as a separate tab.</p> <h3>Understanding Streams</h3> <p>Every box can have multiple content streams:</p> <pre class="language-yaml"><!></pre> <h3>Tab System Usage</h3> <p><strong>Tab Navigation:</strong></p> <ul><li><strong>Click tabs</strong> to switch between streams within a box</li> <li><strong>Close buttons (×)</strong> appear on closeable streams (redirected output, PTY sessions)</li> <li><strong>Background streams</strong> continue updating while inactive</li></ul> <p><strong>Example Workflow:</strong></p> <ol><li>Click “Deploy” choice in control_center → Deploy tab appears in output_display</li> <li>Click “Monitor” choice → Monitor tab appears alongside Deploy tab</li> <li>Click between Deploy/Monitor tabs to view different command outputs</li> <li>Click × on Deploy tab to terminate the deployment process</li></ol> <h3>Multi-Stream Development Setup</h3> <pre class="language-yaml"><!></pre> <p><strong>Result:</strong> Each box shows Content tab initially, then gains Build/Test/Lint tabs as commands execute. Users can monitor multiple processes simultaneously and close completed operations.</p> <h3>PTY Streams</h3> <p>Interactive terminal sessions create dedicated streams:</p> <pre class="language-yaml"><!></pre> <p><strong>Learn more</strong>: See the complete <a href="/docs/stream-architecture">Stream Architecture Guide</a> for advanced stream patterns.</p> <h2>Interactive UI Features</h2> <p>BoxMux supports mouse-driven interface manipulation for dynamic layout editing:</p> <h3>Box Resizing</h3> <ul><li><strong>Drag bottom-right corner</strong> to resize boxes in real-time</li> <li><strong>Auto-save to YAML</strong> preserves changes across restarts</li> <li><strong>Visual feedback</strong> with cursor style changes</li></ul> <h3>Box Movement</h3> <ul><li><strong>Drag title bar</strong> to move boxes to new positions</li> <li><strong>Snap-to-grid</strong> alignment for precise placement</li> <li><strong>Live persistence</strong> to original configuration files</li></ul> <h3>Dynamic Interactions</h3> <pre class="language-yaml"><!></pre> <p><strong>Learn more</strong>: See the complete <a href="/docs/interactive-ui">Interactive UI Guide</a> for advanced interaction patterns.</p> <h2>Building Interfaces</h2> <h3>Step-by-Step Interface Creation</h3> <h4>1. Plan Your Layout</h4> <p>Before writing YAML, sketch your interface:</p> <pre class="language-undefined"><!></pre> <h4>2. Implement the Structure</h4> <pre class="language-yaml"><!></pre> <h4>3. Add Styling</h4> <pre class="language-yaml"><!></pre> <h2>Common Patterns</h2> <h3>Header/Sidebar/Main Layout</h3> <pre class="language-yaml"><!></pre> <h3>Two-Column Layout</h3> <pre class="language-yaml"><!></pre> <h3>Grid Layout (2x2)</h3> <pre class="language-yaml"><!></pre> <h2>PTY Features</h2> <p>PTY (pseudo-terminal) features enable running interactive terminal programs directly within BoxMux boxes.</p> <h3>When to Use PTY</h3> <p>Use PTY for interactive programs that require:</p> <ul><li>Keyboard input (vim, nano, htop)</li> <li>Terminal control sequences (colors, cursor movement)</li> <li>Process interaction (ssh sessions, database shells)</li></ul> <h3>PTY Box Example</h3> <pre class="language-yaml"><!></pre> <h3>PTY Choice Example</h3> <pre class="language-yaml"><!></pre> <h3>PTY vs Regular Execution</h3> <table><thead><tr><th>Feature</th><th>PTY</th><th>Regular</th></tr></thead><tbody><tr><td>Interactive input</td><td>✅ Yes</td><td>❌ No</td></tr><tr><td>ANSI colors/formatting</td><td>✅ Yes</td><td>❌ Limited</td></tr><tr><td>Real-time output</td><td>✅ Yes</td><td>❌ Buffered</td></tr><tr><td>Process control (Ctrl+C)</td><td>✅ Yes</td><td>❌ No</td></tr><tr><td>Terminal programs</td><td>✅ Yes</td><td>❌ No</td></tr><tr><td>Simple commands</td><td>✅ Yes</td><td>✅ Yes</td></tr></tbody></table> <p><strong>Learn more</strong>: See the <a href="/docs/pty-features">PTY Features Guide</a> for detailed configuration and examples.</p> <h2>Data Visualization</h2> <p>BoxMux provides data visualization through charts and tables.</p> <h3>Chart System</h3> <p>BoxMux includes a Unicode-based charting system:</p> <ul><li><strong>Chart Types</strong>: Bar charts, line charts, histograms</li> <li><strong>Smart Layout</strong>: Responsive chart sizing and alignment</li> <li><strong>Real-time Data</strong>: Live data integration with configurable refresh</li></ul> <pre class="language-yaml"><!></pre> <h3>Table System</h3> <p>Table features for structured data:</p> <ul><li><strong>Data Formats</strong>: CSV and JSON parsing</li> <li><strong>Sorting</strong>: Text and numeric sorting with direction control</li> <li><strong>Filtering</strong>: Exact match and case-insensitive search</li> <li><strong>Pagination</strong>: Configurable page sizes with navigation</li> <li><strong>Visual Enhancement</strong>: Zebra striping, row numbers, multiple border styles</li></ul> <pre class="language-yaml"><!></pre> <p><strong>Learn more</strong>: See the <a href="/docs/data-visualization">Data Visualization Guide</a> for chart types, table features, and examples.</p> <h2>Plugin System</h2> <p>BoxMux includes a plugin system for extending functionality with dynamic component loading and security validation.</p> <h3>Plugin Overview</h3> <p>The plugin system supports:</p> <ul><li><strong>Dynamic Component Loading</strong>: Load custom components at runtime using <code>libloading</code></li> <li><strong>Security Validation</strong>: Permission-based access control with manifest validation</li> <li><strong>Fallback System</strong>: Graceful fallback to mock implementations for development/testing</li></ul> <h3>Basic Plugin Usage</h3> <pre class="language-yaml"><!></pre> <p><strong>Learn more</strong>: See the <a href="/docs/plugin-system">Plugin System Guide</a> for development, security model, and examples.</p> <h2>Real-World Examples</h2> <h3>System Monitor</h3> <pre class="language-yaml"><!></pre> <h3>Development Dashboard</h3> <pre class="language-yaml"><!></pre> <h3>Log Monitor</h3> <pre class="language-yaml"><!></pre> <h2>Best Practices</h2> <h3>1. Planning and Design</h3> <ul><li><strong>Sketch first</strong>: Plan your layout before writing YAML</li> <li><strong>Start simple</strong>: Begin with basic boxes and add complexity</li> <li><strong>Consider screen sizes</strong>: Test on different terminal sizes</li> <li><strong>Group related content</strong>: Keep related functions in nearby boxes</li></ul> <h3>2. Configuration Organization</h3> <pre class="language-yaml"><!></pre> <h3>3. Performance Optimization</h3> <pre class="language-yaml"><!></pre> <h3>4. Error Handling</h3> <pre class="language-yaml"><!></pre> <h3>5. User Experience</h3> <pre class="language-yaml"><!></pre> <h2>Techniques</h2> <h3>Dynamic Content with Scripts</h3> <pre class="language-yaml"><!></pre> <h3>Multi-line Scripts</h3> <pre class="language-yaml"><!></pre> <h3>Output Redirection Patterns</h3> <pre class="language-yaml"><!></pre> <h3>Nested Box Hierarchies</h3> <pre class="language-yaml"><!></pre> <h3>Custom Key Bindings</h3> <pre class="language-yaml"><!></pre> <h3>Integration with External APIs</h3> <pre class="language-yaml"><!></pre> <hr/> <h2>Advanced Features</h2> <p>BoxMux includes additional features:</p> <ul><li><strong>Clipboard Integration</strong>: Ctrl+C copies box content with visual feedback</li> <li><strong>Enhanced Scrolling</strong>: Position preservation, page navigation, visual indicators</li> <li><strong>Performance Monitoring</strong>: Built-in benchmarking and performance tracking</li> <li><strong>Schema Validation</strong>: JSON Schema validation with detailed error reporting</li></ul> <p><strong>Learn more</strong>: See the <a href="/docs/advanced-features">Advanced Features Guide</a> for clipboard, scrolling, and performance features.</p> <hr/> <p>For configuration reference, see <a href="/docs/configuration">Configuration Guide</a>.<br/> For data visualization details, see <a href="/docs/data-visualization">Data Visualization Guide</a>.<br/> For plugin development, see <a href="/docs/plugin-system">Plugin System Guide</a>.<br/> For advanced features, see <a href="/docs/advanced-features">Advanced Features Guide</a>.<br/> For programmatic control, see <a href="/docs/api">API Reference</a>.<br/> For troubleshooting, see <a href="/docs/troubleshooting">Troubleshooting Guide</a>.</p>',1);function Un(G){var Y=Bn(),p=n(Ln(Y),10),H=s(p);t(H,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">WELCOME_MSG</span><span class="token punctuation">:</span> <span class="token string">"Welcome to BoxMux!"</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'hello'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'My First BoxMux App'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'greeting'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Hello World'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span>
            <span class="token key atrule">x1</span><span class="token punctuation">:</span> 25%
            <span class="token key atrule">y1</span><span class="token punctuation">:</span> 40%
            <span class="token key atrule">x2</span><span class="token punctuation">:</span> 75%
            <span class="token key atrule">y2</span><span class="token punctuation">:</span> 60%
          <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'$&#123;WELCOME_MSG&#125;'</span>
          <span class="token key atrule">border</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">'green'</span></code>`),a(p);var e=n(p,4),q=s(e);t(q,()=>`<code class="language-bash"><span class="token comment"># If installed via cargo install</span>
boxmux hello.yaml

<span class="token comment"># If built from source</span>
./run_boxmux.sh hello.yaml</code>`),a(e);var o=n(e,6),W=s(o);t(W,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'interactive'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Interactive Menu'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'menu'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Actions'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 20%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 20%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 80%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 80%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">tab_order</span><span class="token punctuation">:</span> <span class="token string">'1'</span>
          <span class="token key atrule">choices</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'hello'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Say Hello'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'echo "Hello, World!"'</span><span class="token punctuation">]</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'date'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Show Date'</span>  
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'date'</span><span class="token punctuation">]</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'files'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'List Files'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'ls -la'</span><span class="token punctuation">]</span></code>`),a(o);var c=n(o,8),j=s(c);t(j,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>                    <span class="token comment"># List of layout definitions</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'main'</span>             <span class="token comment"># Unique layout identifier</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>             <span class="token comment"># This is the main layout</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>              <span class="token comment"># Boxes within this layout</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'box1'</span>       <span class="token comment"># Unique box identifier</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Box'</span>     <span class="token comment"># Box title</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span>          <span class="token comment"># Box position and size</span>
            <span class="token key atrule">x1</span><span class="token punctuation">:</span> 10%          <span class="token comment"># Left edge (percentage of parent)</span>
            <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%          <span class="token comment"># Top edge</span>
            <span class="token key atrule">x2</span><span class="token punctuation">:</span> 90%          <span class="token comment"># Right edge</span>
            <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%          <span class="token comment"># Bottom edge</span></code>`),a(c);var l=n(c,8),K=s(l);t(K,()=>`<code class="language-yaml"><span class="token comment"># Common layout patterns</span>
<span class="token comment"># Full screen</span>
<span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>

<span class="token comment"># Top half  </span>
<span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">&#125;</span>

<span class="token comment"># Left sidebar</span>
<span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 25%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>

<span class="token comment"># Centered window</span>
<span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 25%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 25%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 75%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 75%<span class="token punctuation">&#125;</span></code>`),a(l);var u=n(l,6),J=s(u);t(J,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'menu_box'</span>
  <span class="token key atrule">tab_order</span><span class="token punctuation">:</span> <span class="token string">'1'</span>           <span class="token comment"># Enable keyboard navigation</span>
  <span class="token key atrule">choices</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'action1'</span>
      <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Menu Option 1'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'command to execute'</span><span class="token punctuation">]</span>
      <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'output_box'</span>  <span class="token comment"># Send results to another box</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'action2'</span>
      <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Menu Option 2'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'another command'</span><span class="token punctuation">]</span></code>`),a(u);var k=n(u,6),Q=s(k);t(Q,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'live_box'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Live Data'</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>   <span class="token comment"># Update every 2 seconds</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> date                 <span class="token comment"># Commands to run on each refresh</span>
    <span class="token punctuation">-</span> uptime</code>`),a(k);var i=n(k,8),Z=s(i);t(Z,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">SERVER_NAME</span><span class="token punctuation">:</span> <span class="token string">"production-db"</span>
    <span class="token key atrule">DEFAULT_PORT</span><span class="token punctuation">:</span> <span class="token string">"5432"</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'database_monitor'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Database Monitor - $&#123;SERVER_NAME&#125;'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'connection_status'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Connection Status'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'pg_isready -h $&#123;SERVER_NAME&#125; -p $&#123;DEFAULT_PORT&#125;'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Connected to $&#123;SERVER_NAME&#125;:$&#123;DEFAULT_PORT&#125;"'</span></code>`),a(i);var r=n(i,6),X=s(r);t(X,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">ENVIRONMENT</span><span class="token punctuation">:</span> <span class="token string">"development"</span>  <span class="token comment"># Overridden by $ENVIRONMENT if set</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'deployment_status'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Status - $&#123;ENVIRONMENT&#125;'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'api_check'</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Environment: $&#123;ENVIRONMENT&#125;"'</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "User: $&#123;USER:unknown&#125;"'</span>       <span class="token comment"># Uses $USER or "unknown"</span>
            <span class="token punctuation">-</span> <span class="token string">'echo "Log Level: $&#123;LOG_LEVEL:info&#125;"'</span> <span class="token comment"># Uses $LOG_LEVEL or "info"</span></code>`),a(r);var y=n(r,6),nn=s(y);t(nn,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">REGION</span><span class="token punctuation">:</span> <span class="token string">"us-east-1"</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'infrastructure'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'web_tier'</span>
          <span class="token key atrule">variables</span><span class="token punctuation">:</span>
            <span class="token key atrule">SERVICE_TYPE</span><span class="token punctuation">:</span> <span class="token string">"frontend"</span>
            <span class="token key atrule">PORT</span><span class="token punctuation">:</span> <span class="token string">"80"</span>
          <span class="token key atrule">children</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'nginx_server'</span>
              <span class="token key atrule">variables</span><span class="token punctuation">:</span>
                <span class="token key atrule">SERVICE_NAME</span><span class="token punctuation">:</span> <span class="token string">"nginx"</span>
                <span class="token key atrule">PORT</span><span class="token punctuation">:</span> <span class="token string">"443"</span>  <span class="token comment"># Overrides parent PORT</span>
              <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'$&#123;SERVICE_NAME&#125; ($&#123;SERVICE_TYPE&#125;) - $&#123;REGION&#125;'</span>
              <span class="token comment"># Resolves to: "nginx (frontend) - us-east-1"</span></code>`),a(y);var g=n(y,12),sn=s(g);t(sn,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'control_center'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Control Center'</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'System control panel ready'</span>  <span class="token comment"># → Content tab</span>
  <span class="token key atrule">choices</span><span class="token punctuation">:</span>                               <span class="token comment"># → Control Center tab  </span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'deploy'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'./deploy.sh'</span><span class="token punctuation">]</span>
      <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'output_display'</span>  <span class="token comment"># → Deploy tab in output_display</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'monitor'</span>  
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'./monitor.sh'</span><span class="token punctuation">]</span>
      <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'output_display'</span>  <span class="token comment"># → Monitor tab in output_display</span>

<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'output_display'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Output'</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Command output will appear here...'</span>
  <span class="token comment"># Tabs: [Content] + [Deploy] + [Monitor] as commands run</span></code>`),a(g);var d=n(g,14),an=s(d);t(an,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'dev_commands'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Development'</span>
  <span class="token key atrule">choices</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'build'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'cargo build --release'</span><span class="token punctuation">]</span>
      <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'build_results'</span>
      <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>               <span class="token comment"># Live output streaming</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'test'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'cargo test'</span><span class="token punctuation">]</span>  
      <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'test_results'</span>
      <span class="token key atrule">streaming</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'lint'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'cargo clippy'</span><span class="token punctuation">]</span>
      <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'lint_results'</span>

<span class="token comment"># Each command creates dedicated tabs in result boxes</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'build_results'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Build Output'</span>
  <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Build results appear here...'</span>

<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'test_results'</span>  
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Test Output'</span>
  <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Test results appear here...'</span>

<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'lint_results'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Lint Output'</span> 
  <span class="token key atrule">auto_scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Lint results appear here...'</span></code>`),a(d);var m=n(d,8),tn=s(m);t(tn,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'terminal'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Terminal'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'bash'</span><span class="token punctuation">]</span>
  <span class="token comment"># → Creates Terminal tab with interactive shell</span>

<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'monitor'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Monitor'</span>  
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'htop'</span><span class="token punctuation">]</span>
  <span class="token comment"># → Creates System Monitor tab with interactive htop</span></code>`),a(m);var h=n(m,18),pn=s(h);t(pn,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'resizable_demo'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Drag Corner to Resize →'</span>
  <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 60%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 60%<span class="token punctuation">&#125;</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
    This box demonstrates interactive features:</span>
    
    • Drag bottom<span class="token punctuation">-</span>right corner to resize
    • Drag title bar to move  
    • Changes auto<span class="token punctuation">-</span>save to YAML file
    • Try it now<span class="token tag">!</span>
    
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'movable_demo'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'← Drag Title to Move'</span>  
  <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 40%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 40%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">&#125;</span>
  <span class="token key atrule">z_index</span><span class="token punctuation">:</span> <span class="token number">2</span>  <span class="token comment"># Appears on top for clear interaction</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
    Interactive UI features:</span>
    
    ✓ Real<span class="token punctuation">-</span>time visual feedback
    ✓ Automatic YAML persistence  
    ✓ Boundary constraint enforcement
    ✓ Smooth 60 FPS performance optimization</code>`),a(h);var b=n(h,12),en=s(b);t(en,()=>`<code class="language-undefined">┌─────────────────────────────┐
│           Header            │
├─────────┬───────────────────┤
│ Sidebar │    Main Content   │
│         │                   │
│         │                   │
├─────────┴───────────────────┤
│           Footer            │
└─────────────────────────────┘</code>`),a(b);var _=n(b,4),on=s(_);t(on,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'dashboard'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'My Dashboard'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token comment"># Header</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'header'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Dashboard Header'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 15%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Welcome to the Dashboard'</span>
          
        <span class="token comment"># Sidebar</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'sidebar'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Navigation'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 15%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 25%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 85%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">tab_order</span><span class="token punctuation">:</span> <span class="token string">'1'</span>
          <span class="token key atrule">choices</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'system'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'System Info'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'uname -a'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'main'</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'processes'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Processes'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'ps aux | head -10'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'main'</span>
              
        <span class="token comment"># Main content</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'main'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Main Content'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 25%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 15%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 85%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Select an option from the sidebar'</span>
          <span class="token key atrule">scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          
        <span class="token comment"># Footer</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'footer'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Status'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 85%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">1000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'date'</span><span class="token punctuation">]</span></code>`),a(_);var v=n(_,4),cn=s(v);t(cn,()=>`<code class="language-yaml"><span class="token comment"># At layout level for consistent styling</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'dashboard'</span>
  <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Styled Dashboard'</span>
  <span class="token key atrule">bg_color</span><span class="token punctuation">:</span> <span class="token string">'black'</span>
  <span class="token key atrule">fg_color</span><span class="token punctuation">:</span> <span class="token string">'white'</span>
  <span class="token key atrule">title_fg_color</span><span class="token punctuation">:</span> <span class="token string">'bright_yellow'</span>
  <span class="token key atrule">title_bg_color</span><span class="token punctuation">:</span> <span class="token string">'blue'</span>
  <span class="token key atrule">selected_bg_color</span><span class="token punctuation">:</span> <span class="token string">'bright_blue'</span>
  <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">'green'</span>
  <span class="token comment"># ... children boxes inherit these styles</span></code>`),a(v);var f=n(v,6),ln=s(f);t(ln,()=>`<code class="language-yaml"><span class="token key atrule">children</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'header'</span>
    <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 10%<span class="token punctuation">&#125;</span>
  <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'sidebar'</span>  
    <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 20%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>
  <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'main'</span>
    <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 20%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span></code>`),a(f);var x=n(f,4),un=s(x);t(un,()=>`<code class="language-yaml"><span class="token key atrule">children</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'left'</span>
    <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>
  <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'right'</span>
    <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span></code>`),a(x);var S=n(x,4),kn=s(S);t(kn,()=>`<code class="language-yaml"><span class="token key atrule">children</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'top_left'</span>
    <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">&#125;</span>
  <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'top_right'</span>
    <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">&#125;</span>
  <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'bottom_left'</span>
    <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>
  <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'bottom_right'</span>
    <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span></code>`),a(S);var C=n(S,14),rn=s(C);t(rn,()=>`<code class="language-yaml"><span class="token comment"># Interactive system monitor</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'system_monitor'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Monitor ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> htop
  <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>

<span class="token comment"># Text editor</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'editor'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Configuration Editor ⚡'</span>
  <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> vim /etc/app/config.yaml
  <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span></code>`),a(C);var E=n(C,4),yn=s(E);t(yn,()=>`<code class="language-yaml"><span class="token comment"># Interactive menu choices</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'admin_box'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Administration'</span>
  <span class="token key atrule">choices</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'edit_config'</span>
      <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Edit Config File'</span>
      <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> vim /etc/app/config.yaml
    
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'database_shell'</span>
      <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Database Console'</span>
      <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> psql <span class="token punctuation">-</span>U postgres <span class="token punctuation">-</span>d myapp
        
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'ssh_server'</span>
      <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Connect to Server'</span>
      <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> ssh admin@production<span class="token punctuation">-</span>server</code>`),a(E);var w=n(E,18),gn=s(w);t(gn,()=>`<code class="language-yaml"><span class="token comment"># Live CPU usage chart with smart layout</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'cpu_chart'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'CPU Usage Over Time'</span>
  <span class="token key atrule">chart_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">chart_type</span><span class="token punctuation">:</span> <span class="token string">'line'</span>
    <span class="token key atrule">width</span><span class="token punctuation">:</span> <span class="token number">50</span>
    <span class="token key atrule">height</span><span class="token punctuation">:</span> <span class="token number">15</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'CPU Usage Trend'</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> top <span class="token punctuation">-</span>l 1 <span class="token punctuation">|</span> grep "CPU usage" <span class="token punctuation">|</span> awk '<span class="token punctuation">&#123;</span>print $3<span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> sed 's/%//'</code>`),a(w);var M=n(w,8),dn=s(M);t(dn,()=>`<code class="language-yaml"><span class="token comment"># Advanced process table with all features</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'process_table'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Top Processes'</span>
  <span class="token key atrule">table_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">headers</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'Process'</span><span class="token punctuation">,</span> <span class="token string">'CPU %'</span><span class="token punctuation">,</span> <span class="token string">'Memory'</span><span class="token punctuation">,</span> <span class="token string">'PID'</span><span class="token punctuation">]</span>
    <span class="token key atrule">sortable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">filterable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">page_size</span><span class="token punctuation">:</span> <span class="token number">10</span>
    <span class="token key atrule">zebra_striping</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">show_row_numbers</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">border_style</span><span class="token punctuation">:</span> <span class="token string">'double'</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">5000</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> ps aux <span class="token punctuation">-</span><span class="token punctuation">-</span>no<span class="token punctuation">-</span>headers <span class="token punctuation">|</span> awk '<span class="token punctuation">&#123;</span>printf "%s<span class="token punctuation">,</span>%.1f<span class="token punctuation">,</span>%.1f<span class="token punctuation">,</span>%s&#92;n"<span class="token punctuation">,</span> $11<span class="token punctuation">,</span> $3<span class="token punctuation">,</span> $4<span class="token punctuation">,</span> $2<span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> sort <span class="token punctuation">-</span>rn <span class="token punctuation">-</span>k2 <span class="token punctuation">-</span>t<span class="token punctuation">,</span> <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">15</span></code>`),a(M);var T=n(M,16),mn=s(T);t(mn,()=>`<code class="language-yaml"><span class="token comment"># Custom data visualization plugin</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'custom_viz'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Custom Metrics'</span>
  <span class="token key atrule">plugin_type</span><span class="token punctuation">:</span> <span class="token string">'metrics_visualizer'</span>
  <span class="token key atrule">plugin_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">metric_source</span><span class="token punctuation">:</span> <span class="token string">'/var/log/metrics.json'</span>
    <span class="token key atrule">visualization_type</span><span class="token punctuation">:</span> <span class="token string">'heatmap'</span>
    <span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">1000</span>
  <span class="token key atrule">security_permissions</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token string">'filesystem_read'</span>
    <span class="token punctuation">-</span> <span class="token string">'process_spawn'</span></code>`),a(T);var P=n(T,8),hn=s(P);t(hn,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">variables</span><span class="token punctuation">:</span>
    <span class="token key atrule">REFRESH_FAST</span><span class="token punctuation">:</span> <span class="token string">"2000"</span>
    <span class="token key atrule">REFRESH_SLOW</span><span class="token punctuation">:</span> <span class="token string">"5000"</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'sysmon'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Monitor - $&#123;USER:admin&#125;'</span>
      <span class="token key atrule">bg_color</span><span class="token punctuation">:</span> <span class="token string">'black'</span>
      <span class="token key atrule">fg_color</span><span class="token punctuation">:</span> <span class="token string">'green'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'cpu'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'CPU Usage'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 48%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 40%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> $<span class="token punctuation">&#123;</span>REFRESH_FAST<span class="token punctuation">&#125;</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> top <span class="token punctuation">-</span>l 1 <span class="token punctuation">|</span> grep "CPU usage"
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'memory'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Memory Usage'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 52%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 40%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> $<span class="token punctuation">&#123;</span>REFRESH_FAST<span class="token punctuation">&#125;</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> top <span class="token punctuation">-</span>l 1 <span class="token punctuation">|</span> grep "PhysMem"
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'disk'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Disk Usage'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 45%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 75%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> $<span class="token punctuation">&#123;</span>REFRESH_SLOW<span class="token punctuation">&#125;</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> df <span class="token punctuation">-</span>h <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">5</span>
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'processes'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Top Processes'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 80%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">3000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> ps aux <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">8</span></code>`),a(P);var R=n(P,4),bn=s(R);t(bn,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'dev_dash'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Development Dashboard'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'git_status'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Git Status'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 30%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">10000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> git status <span class="token punctuation">-</span><span class="token punctuation">-</span>short
            <span class="token punctuation">-</span> echo "<span class="token punctuation">---</span>"
            <span class="token punctuation">-</span> git log <span class="token punctuation">-</span><span class="token punctuation">-</span>oneline <span class="token punctuation">-</span><span class="token number">3</span>
            
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'actions'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Actions'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 35%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 45%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 85%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">tab_order</span><span class="token punctuation">:</span> <span class="token string">'1'</span>
          <span class="token key atrule">choices</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'build'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Build Project'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'cargo build'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'output'</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'test'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Run Tests'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'cargo test'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'output'</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'lint'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Run Linter'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'cargo clippy'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'output'</span>
              
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'output'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Output'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 35%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 85%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Select an action'</span>
          <span class="token key atrule">scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span></code>`),a(R);var L=n(R,4),_n=s(L);t(_n,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'logmon'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Log Monitor'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'log_selector'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Select Log'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 25%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">tab_order</span><span class="token punctuation">:</span> <span class="token string">'1'</span>
          <span class="token key atrule">choices</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'syslog'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'System Log'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'tail -20 /var/log/system.log'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'log_viewer'</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'error_log'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Error Log'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'tail -20 /var/log/error.log'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'log_viewer'</span>
              
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'log_viewer'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Log Contents'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 30%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Select a log file'</span>
          <span class="token key atrule">scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span></code>`),a(L);var D=n(L,10),vn=s(D);t(vn,()=>`<code class="language-yaml"><span class="token comment"># Use meaningful IDs</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'cpu_monitor'</span>        <span class="token comment"># Good: descriptive</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'box1'</span>             <span class="token comment"># Bad: generic</span>

<span class="token comment"># Consistent styling</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'main'</span>
      <span class="token comment"># Define colors once at layout level</span>
      <span class="token key atrule">bg_color</span><span class="token punctuation">:</span> <span class="token string">'black'</span>
      <span class="token key atrule">fg_color</span><span class="token punctuation">:</span> <span class="token string">'white'</span>
      <span class="token comment"># Boxes inherit these styles</span></code>`),a(D);var I=n(D,4),fn=s(I);t(fn,()=>`<code class="language-yaml"><span class="token comment"># Optimize refresh intervals</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'fast_updates'</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">1000</span>   <span class="token comment"># 1 second for critical data</span>

<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'slow_updates'</span>  
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">30000</span>  <span class="token comment"># 30 seconds for less critical data</span>

<span class="token comment"># Efficient scripts</span>
<span class="token key atrule">script</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> ps aux <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">10</span>      <span class="token comment"># Limit output</span>
  <span class="token punctuation">-</span> df <span class="token punctuation">-</span>h <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">5</span>        <span class="token comment"># Don't show all filesystems</span></code>`),a(I);var B=n(I,4),xn=s(B);t(xn,()=>`<code class="language-yaml"><span class="token comment"># Robust scripts with fallbacks</span>
<span class="token key atrule">script</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
    if command -v docker >/dev/null 2>&amp;1; then
      docker ps
    else
      echo "Docker not available"
    fi</span></code>`),a(B);var U=n(B,4),Sn=s(U);t(Sn,()=>`<code class="language-yaml"><span class="token comment"># Clear navigation with tab_order</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'menu1'</span>
  <span class="token key atrule">tab_order</span><span class="token punctuation">:</span> <span class="token string">'1'</span>    <span class="token comment"># First tab stop</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'menu2'</span>  
  <span class="token key atrule">tab_order</span><span class="token punctuation">:</span> <span class="token string">'2'</span>    <span class="token comment"># Second tab stop</span>

<span class="token comment"># Helpful content and titles</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'output'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Command Output'</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Select a command from the menu to see output here'</span></code>`),a(U);var F=n(U,6),Cn=s(F);t(Cn,()=>`<code class="language-yaml"><span class="token comment"># Conditional content</span>
<span class="token key atrule">script</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
    if systemctl is-active nginx >/dev/null 2>&amp;1; then
      echo "✓ Nginx: Running"
    else
      echo "✗ Nginx: Stopped"
    fi</span></code>`),a(F);var A=n(F,4),En=s(A);t(En,()=>`<code class="language-yaml"><span class="token key atrule">script</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
    echo "System Information:"
    echo "=================="
    echo "Hostname: $(hostname)"
    echo "Uptime: $(uptime)"
    echo "Load: $(uptime | awk -F'load average:' '&#123;print $2&#125;')"</span></code>`),a(A);var O=n(A,4),wn=s(O);t(wn,()=>`<code class="language-yaml"><span class="token comment"># Menu with shared output box</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'actions'</span>
  <span class="token key atrule">choices</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'action1'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'command1'</span><span class="token punctuation">]</span>
      <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'shared_output'</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'action2'</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'command2'</span><span class="token punctuation">]</span>
      <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'shared_output'</span>
      
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'shared_output'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Results'</span>
  <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Select an action'</span></code>`),a(O);var z=n(O,4),Mn=s(z);t(Mn,()=>`<code class="language-yaml"><span class="token comment"># Complex nested structure</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'main_container'</span>
  <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">&#125;</span>
  <span class="token key atrule">children</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'left_section'</span>
      <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'top_left'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 45%<span class="token punctuation">&#125;</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'bottom_left'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 55%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">&#125;</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'right_section'</span>
      <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>
      <span class="token comment"># ... right side content</span></code>`),a(z);var N=n(z,4),Tn=s(N);t(Tn,()=>`<code class="language-yaml"><span class="token comment"># Custom keyboard shortcuts</span>
<span class="token key atrule">on_keypress</span><span class="token punctuation">:</span>
  <span class="token key atrule">r</span><span class="token punctuation">:</span>                    <span class="token comment"># Press 'r' to refresh</span>
    <span class="token punctuation">-</span> echo 'Refreshing<span class="token punctuation">...</span>'
    <span class="token punctuation">-</span> date
  <span class="token key atrule">q</span><span class="token punctuation">:</span>                    <span class="token comment"># Press 'q' to quit</span>
    <span class="token punctuation">-</span> exit
  <span class="token key atrule">'ctrl+c'</span><span class="token punctuation">:</span>            <span class="token comment"># Ctrl+C handling</span>
    <span class="token punctuation">-</span> echo 'Interrupted'</code>`),a(N);var V=n(N,4),Pn=s(V);t(Pn,()=>`<code class="language-yaml"><span class="token comment"># API integration example</span>
<span class="token key atrule">script</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
    # Fetch weather data
    curl -s "https://api.openweathermap.org/data/2.5/weather?q=London&amp;appid=YOUR_KEY" | 
    jq '.main.temp, .weather[0].description'</span></code>`),a(V),In(14),Dn(G,Y)}const Vn=Object.freeze(Object.defineProperty({__proto__:null,default:Un,metadata:$},Symbol.toStringTag,{value:"Module"}));export{Vn as _};
