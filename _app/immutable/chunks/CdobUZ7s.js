import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as W,s as n,a as G,b as j,d as s,r as a,n as q}from"./E9AOERj2.js";import{h as t}from"./Cc2oKZLy.js";const w={title:"Advanced Features",description:"Mouse support, hot keys, clipboard integration, navigation, and performance monitoring."},{title:sn,description:an}=w;var J=W('<h2>Table of Contents</h2> <ul><li><a href="#mouse-click-support">Mouse Click Support</a></li> <li><a href="#hot-key-actions">Hot Key Actions</a></li> <li><a href="#enhanced-navigation">Enhanced Navigation</a></li> <li><a href="#clipboard-integration">Clipboard Integration</a></li> <li><a href="#enhanced-scrolling">Enhanced Scrolling</a></li> <li><a href="#performance-monitoring">Performance Monitoring</a></li> <li><a href="#configuration-schema-validation">Configuration Schema Validation</a></li> <li><a href="#manual-socket-implementation">Manual Socket Implementation</a></li> <li><a href="#real-world-examples">Real-World Examples</a></li></ul> <h2>Mouse Click Support</h2> <p>BoxMux provides mouse interaction for navigation and control.</p> <h3>Features</h3> <ul><li><strong>Box Selection</strong>: Click to select and focus boxes</li> <li><strong>Menu Activation</strong>: Click menu items to trigger actions</li> <li><strong>Parent Box Auto-selection</strong>: Menu clicks automatically select parent box</li> <li><strong>Scrollable Content</strong>: Auto-selectability for boxes with scrollable content</li> <li><strong>Non-blocking Execution</strong>: Threaded execution prevents UI freezing</li> <li><strong>Visual Feedback</strong>: Immediate visual feedback on clicks</li></ul> <h3>Usage</h3> <pre class="language-yaml"><!></pre> <h3>Mouse Interaction Behavior</h3> <ul><li><strong>Single Click</strong>: Select box and focus for keyboard input</li> <li><strong>Menu Clicks</strong>: Execute choice action and redirect output to target box</li> <li><strong>Box Focus</strong>: Enable keyboard scrolling and input routing</li> <li><strong>Visual Indicators</strong>: Focused boxes show distinct border colors</li></ul> <h2>Hot Key Actions</h2> <p>Global keyboard shortcuts to trigger specific choice actions without menu navigation.</p> <h3>Features</h3> <ul><li><strong>Global Shortcuts</strong>: F1-F24 function keys for instant action execution</li> <li><strong>Direct Choice Execution</strong>: Bypass menu navigation for frequently used commands</li> <li><strong>Background Execution</strong>: Actions run in background threads</li> <li><strong>Output Redirection</strong>: Hot key actions support output redirection</li> <li><strong>Visual Feedback</strong>: Hot key mappings shown in box titles</li></ul> <h3>Configuration</h3> <pre class="language-yaml"><!></pre> <h3>Hot Key Best Practices</h3> <pre class="language-yaml"><!></pre> <h2>Enhanced Navigation</h2> <p>Keyboard navigation with Home/End scrolling and proportional scrollbars.</p> <h3>Features</h3> <ul><li><strong>Home/End Horizontal</strong>: Home/End keys scroll to beginning/end of lines (0%/100%)</li> <li><strong>Ctrl+Home/End Vertical</strong>: Ctrl+Home/End scroll to top/bottom of content (0%/100%)</li> <li><strong>Proportional Scrollbars</strong>: Scrollbar knob size reflects content proportions</li> <li><strong>Accurate Positioning</strong>: Knob position accurately represents scroll location</li> <li><strong>Visual Feedback</strong>: Scrollbars show exact scroll state and content ratio</li></ul> <h3>Navigation Keys</h3> <pre class="language-undefined"><!></pre> <h3>Navigation Configuration</h3> <pre class="language-yaml"><!></pre> <h3>Proportional Scrollbar Behavior</h3> <ul><li><strong>Large Knob</strong>: When content is only slightly larger than viewport (e.g., 1 extra line)</li> <li><strong>Small Knob</strong>: When there’s much more content than fits in viewport</li> <li><strong>Reaches End</strong>: Knob reaches bottom/right edge when scrolled to 100%</li> <li><strong>Smooth Movement</strong>: Knob position reflects exact scroll percentage</li></ul> <pre class="language-yaml"><!></pre> <h2>Clipboard Integration</h2> <p>BoxMux provides platform-specific clipboard integration with visual feedback.</p> <h3>Features</h3> <ul><li><strong>Ctrl+C Integration</strong>: Copy focused box content to system clipboard</li> <li><strong>Visual Feedback</strong>: Brief visual indication when content is copied</li> <li><strong>Platform Support</strong>: Works on macOS, Linux, and Windows</li> <li><strong>Content Selection</strong>: Copies complete box content or selected regions</li></ul> <h3>Usage</h3> <ol><li><strong>Navigate</strong> to a box using Tab key or mouse</li> <li><strong>Press Ctrl+C</strong> to copy box content to clipboard</li> <li><strong>Visual flash</strong> indicates successful copy operation</li> <li><strong>Paste</strong> content in any application using standard clipboard operations</li></ol> <h3>Configuration</h3> <pre class="language-yaml"><!></pre> <h3>Implementation Details</h3> <ul><li>Platform-specific clipboard APIs: <ul><li><strong>macOS</strong>: <code>pbcopy</code> command integration</li> <li><strong>Linux</strong>: <code>xclip</code> or <code>xsel</code> command integration</li> <li><strong>Windows</strong>: Windows clipboard API</li></ul></li> <li>Visual feedback through brief color change</li> <li>Error handling for clipboard access failures</li></ul> <h2>Enhanced Scrolling</h2> <p>BoxMux provides advanced scrolling capabilities with position preservation and navigation controls.</p> <h3>Features</h3> <ul><li><strong>Position Preservation</strong>: Maintains scroll position during auto-refresh</li> <li><strong>Page Navigation</strong>: Page Up/Down keyboard support for efficient scrolling</li> <li><strong>Visual Indicators</strong>: Scroll position indicators and scrollbar detection</li> <li><strong>Smooth Scrolling</strong>: Smooth scrolling with configurable scroll amounts</li> <li><strong>Auto-sizing</strong>: Automatic scrollbar detection for focusable boxes</li></ul> <h3>Keyboard Controls</h3> <ul><li><strong>Arrow Keys</strong>: Line-by-line scrolling (Up/Down)</li> <li><strong>Page Up/Down</strong>: Page-based scrolling (10x scroll amount)</li> <li><strong>Home/End</strong>: Jump to beginning/end of content</li> <li><strong>Mouse Wheel</strong>: Scroll support (where available)</li></ul> <h3>Configuration</h3> <pre class="language-yaml"><!></pre> <h3>Advanced Scrolling Features</h3> <pre class="language-yaml"><!></pre> <h2>Performance Monitoring</h2> <p>BoxMux includes built-in performance monitoring and benchmarking capabilities.</p> <h3>Performance Benchmarks</h3> <p>BoxMux tracks performance metrics for core operations:</p> <ul><li><strong>ANSI Stripping</strong>: 10k operations in ~1.5s</li> <li><strong>Key Mapping</strong>: 150k operations in ~2.2s</li> <li><strong>Bounds Calculation</strong>: 100k operations in ~20ms</li> <li><strong>Script Execution</strong>: 100 operations in ~575ms</li> <li><strong>Large Config Processing</strong>: 1k operations in ~38ms</li></ul> <h3>Monitoring Configuration</h3> <pre class="language-yaml"><!></pre> <h3>Performance Testing</h3> <pre class="language-yaml"><!></pre> <h2>Configuration Schema Validation</h2> <p>BoxMux includes comprehensive JSON Schema validation for YAML configurations.</p> <h3>Features</h3> <ul><li><strong>Automatic Validation</strong>: Configuration files validated on load</li> <li><strong>Detailed Error Messages</strong>: Line/column specific error reporting</li> <li><strong>Schema Coverage</strong>: Complete schema validation for all configuration sections</li> <li><strong>Type Checking</strong>: Strict type validation and required field checking</li></ul> <h3>Error Reporting</h3> <p>When loading invalid configuration:</p> <pre class="language-bash"><!></pre> <h3>Schema Validation Configuration</h3> <pre class="language-yaml"><!></pre> <h3>Custom Validation Rules</h3> <pre class="language-yaml"><!></pre> <h2>Manual Socket Implementation</h2> <p>BoxMux uses a manual Unix socket implementation without external dependencies for maximum control and reliability.</p> <h3>Features</h3> <ul><li><strong>Manual Implementation</strong>: Direct Unix socket handling using <code>std::os::unix::net</code></li> <li><strong>No External Dependencies</strong>: Self-contained socket communication</li> <li><strong>Full Control</strong>: Complete control over socket lifecycle and message handling</li> <li><strong>Error Recovery</strong>: Comprehensive error handling and connection recovery</li> <li><strong>Performance</strong>: Optimized for low-latency communication</li></ul> <h3>Socket Commands</h3> <p>BoxMux supports comprehensive socket API:</p> <pre class="language-bash"><!></pre> <h3>Socket Configuration</h3> <pre class="language-yaml"><!></pre> <h2>Real-World Examples</h2> <h3>DevOps Dashboard</h3> <pre class="language-yaml"><!></pre> <h3>System Monitoring with Advanced Features</h3> <pre class="language-yaml"><!></pre> <h3>Development Environment Monitor</h3> <pre class="language-yaml"><!></pre> <hr/> <p>For configuration details, see <a href="configuration.md">Configuration Reference</a>.<br/> For basic usage, see <a href="user-guide.md">User Guide</a>.<br/> For plugin development, see <a href="plugin-system.md">Plugin System</a>.</p>',1);function Q(C){var x=J(),p=n(G(x),14),F=s(p);t(F,()=>`<code class="language-yaml"><span class="token comment"># Enable mouse interaction (enabled by default)</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">mouse_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'interactive'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'sensitive_menu'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Actions (Click to Execute)'</span>
          <span class="token key atrule">choices</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'deploy'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Deploy Application'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'./deploy.sh'</span><span class="token punctuation">]</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'test'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Run Tests'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'cargo test'</span><span class="token punctuation">]</span>
              
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'output_box'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Output (Click to Focus)'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Click this box to focus and enable scrolling'</span></code>`),a(p);var e=n(p,16),M=s(e);t(M,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">hot_keys</span><span class="token punctuation">:</span>
    <span class="token key atrule">'F1'</span><span class="token punctuation">:</span> <span class="token string">'build'</span>           <span class="token comment"># F1 triggers build action</span>
    <span class="token key atrule">'F2'</span><span class="token punctuation">:</span> <span class="token string">'test'</span>            <span class="token comment"># F2 triggers test action</span>
    <span class="token key atrule">'F3'</span><span class="token punctuation">:</span> <span class="token string">'deploy'</span>          <span class="token comment"># F3 triggers deploy action</span>
    <span class="token key atrule">'F5'</span><span class="token punctuation">:</span> <span class="token string">'refresh_all'</span>     <span class="token comment"># F5 refreshes all boxes</span>
    <span class="token key atrule">'F9'</span><span class="token punctuation">:</span> <span class="token string">'git_status'</span>      <span class="token comment"># F9 shows git status</span>
    <span class="token key atrule">'F12'</span><span class="token punctuation">:</span> <span class="token string">'system_info'</span>    <span class="token comment"># F12 shows system info</span>
    
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'development'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Dev Environment (F1=Build, F2=Test, F3=Deploy)'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'menu'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Quick Actions'</span>
          <span class="token key atrule">choices</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'build'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Build Project [F1]'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'cargo build'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'output'</span>
              
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'test'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Run Tests [F2]'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'cargo test'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'output'</span>
              
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'deploy'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Deploy [F3]'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'./deploy.sh'</span><span class="token punctuation">]</span>
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'output'</span>
              
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'output'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Build/Test Output'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 0%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 40%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 100%<span class="token punctuation">&#125;</span></code>`),a(e);var o=n(e,4),P=s(o);t(P,()=>`<code class="language-yaml"><span class="token comment"># Group related actions by function key ranges</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">hot_keys</span><span class="token punctuation">:</span>
    <span class="token comment"># Build/Test (F1-F4)</span>
    <span class="token key atrule">'F1'</span><span class="token punctuation">:</span> <span class="token string">'build_debug'</span>
    <span class="token key atrule">'F2'</span><span class="token punctuation">:</span> <span class="token string">'build_release'</span>
    <span class="token key atrule">'F3'</span><span class="token punctuation">:</span> <span class="token string">'test_unit'</span>
    <span class="token key atrule">'F4'</span><span class="token punctuation">:</span> <span class="token string">'test_integration'</span>
    
    <span class="token comment"># Git operations (F5-F8)</span>
    <span class="token key atrule">'F5'</span><span class="token punctuation">:</span> <span class="token string">'git_status'</span>
    <span class="token key atrule">'F6'</span><span class="token punctuation">:</span> <span class="token string">'git_pull'</span>
    <span class="token key atrule">'F7'</span><span class="token punctuation">:</span> <span class="token string">'git_commit'</span>
    <span class="token key atrule">'F8'</span><span class="token punctuation">:</span> <span class="token string">'git_push'</span>
    
    <span class="token comment"># System monitoring (F9-F12)</span>
    <span class="token key atrule">'F9'</span><span class="token punctuation">:</span> <span class="token string">'cpu_usage'</span>
    <span class="token key atrule">'F10'</span><span class="token punctuation">:</span> <span class="token string">'memory_usage'</span>
    <span class="token key atrule">'F11'</span><span class="token punctuation">:</span> <span class="token string">'disk_usage'</span>
    <span class="token key atrule">'F12'</span><span class="token punctuation">:</span> <span class="token string">'system_logs'</span></code>`),a(o);var l=n(o,12),E=s(l);t(E,()=>`<code class="language-undefined">Home          - Scroll to beginning of current line (horizontal 0%)
End           - Scroll to end of current line (horizontal 100%)
Ctrl+Home     - Scroll to top of content (vertical 0%)
Ctrl+End      - Scroll to bottom of content (vertical 100%)
Arrow Keys    - Line-by-line scrolling
Page Up/Down  - Page-based scrolling</code>`),a(l);var c=n(l,4),B=s(c);t(B,()=>`<code class="language-yaml"><span class="token comment"># Box with enhanced navigation</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'large_content'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Large Content (Home/End/Ctrl+Home/End navigation)'</span>
  <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 80%<span class="token punctuation">&#125;</span>
  <span class="token key atrule">scrollable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">navigation_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
      echo "=== Large Content for Navigation Demo ==="
      for i in &#123;1..100&#125;; do
        echo "Line $i: This is a very long line that extends beyond the box width to demonstrate horizontal scrolling capabilities in BoxMux boxes"
      done</span></code>`),a(c);var u=n(c,6),$=s(u);t($,()=>`<code class="language-yaml"><span class="token comment"># Example: Different content ratios</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'small_overflow'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Small Overflow (Large Knob)'</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'seq 1 25'</span><span class="token punctuation">]</span>  <span class="token comment"># Only 5 extra lines - large scrollbar knob</span>
  
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'large_overflow'</span> 
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Large Overflow (Small Knob)'</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'seq 1 1000'</span><span class="token punctuation">]</span> <span class="token comment"># Many extra lines - small scrollbar knob</span></code>`),a(u);var i=n(u,16),A=s(i);t(A,()=>`<code class="language-yaml"><span class="token comment"># Enable clipboard for specific boxes</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'results_box'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Command Results'</span>
  <span class="token key atrule">clipboard_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>  <span class="token comment"># Allow clipboard copying</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> ps aux <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">20</span></code>`),a(i);var r=n(i,20),U=s(r);t(U,()=>`<code class="language-yaml"><span class="token comment"># Box with enhanced scrolling</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'scrollable_output'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Large Output'</span>
  <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 80%<span class="token punctuation">&#125;</span>
  <span class="token key atrule">scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">scroll_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">preserve_position</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>    <span class="token comment"># Maintain position during refresh</span>
    <span class="token key atrule">show_indicators</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>      <span class="token comment"># Show scroll position</span>
    <span class="token key atrule">page_size</span><span class="token punctuation">:</span> <span class="token number">10</span>             <span class="token comment"># Lines per page scroll</span>
    <span class="token key atrule">smooth_scrolling</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>    <span class="token comment"># Enable smooth scrolling</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> ps aux <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">100</span>  <span class="token comment"># Large output for scrolling demo</span></code>`),a(r);var k=n(r,4),D=s(k);t(D,()=>`<code class="language-yaml"><span class="token comment"># Auto-refresh with scroll preservation</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'live_data'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Live Data Stream'</span>
  <span class="token key atrule">scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">scroll_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">preserve_position</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">auto_scroll_new</span><span class="token punctuation">:</span> <span class="token boolean important">false</span>    <span class="token comment"># Don't auto-scroll to new content</span>
    <span class="token key atrule">scroll_buffer_size</span><span class="token punctuation">:</span> <span class="token number">1000</span>  <span class="token comment"># Maximum lines to keep</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">1000</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> date <span class="token important">&amp;&amp;</span> ps aux <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">50</span></code>`),a(k);var g=n(k,14),L=s(g);t(L,()=>`<code class="language-yaml"><span class="token comment"># Performance monitoring box</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'performance'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Performance'</span>
  <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">&#125;</span>
  <span class="token key atrule">performance_monitoring</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">5000</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
      echo "BoxMux Performance Metrics:"
      echo "=========================="
      echo "Memory Usage: $(ps -o rss= -p $$) KB"
      echo "CPU Usage: $(ps -o %cpu= -p $$)%"
      echo "Uptime: $(uptime -p)"
      echo "Active Boxes: $(pgrep -f boxmux | wc -l)"</span></code>`),a(g);var m=n(g,4),R=s(m);t(R,()=>`<code class="language-yaml"><span class="token comment"># Load testing configuration</span>
<span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'load_test'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Load Test Results'</span>
  <span class="token key atrule">performance_test</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">test_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">duration</span><span class="token punctuation">:</span> <span class="token number">60</span>        <span class="token comment"># Test duration in seconds</span>
    <span class="token key atrule">refresh_rate</span><span class="token punctuation">:</span> <span class="token number">100</span>   <span class="token comment"># Fast refresh for stress testing</span>
    <span class="token key atrule">data_volume</span><span class="token punctuation">:</span> <span class="token string">'high'</span> <span class="token comment"># High data volume test</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
      # Generate high-volume output for testing
      for i in &#123;1..1000&#125;; do
        echo "Test line $i: $(date)"
      done</span></code>`),a(m);var y=n(m,14),T=s(y);t(T,()=>`<code class="language-bash">$ boxmux invalid-config.yaml
Error: Configuration validation failed
  --<span class="token operator">></span> invalid-config.yaml:15:3
   <span class="token operator">|</span>
<span class="token number">15</span> <span class="token operator">|</span>   position: <span class="token string">"invalid"</span>
   <span class="token operator">|</span>   ^^^^^^^^ expected object with x1, y1, x2, y2 properties
   <span class="token operator">|</span>
   <span class="token operator">=</span> help: position must be an object with percentage-based coordinates</code>`),a(y);var d=n(y,4),I=s(d);t(I,()=>`<code class="language-yaml"><span class="token comment"># Enable strict validation</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">validation</span><span class="token punctuation">:</span>
    <span class="token key atrule">schema_validation</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>     <span class="token comment"># Enable JSON schema validation</span>
    <span class="token key atrule">strict_mode</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>          <span class="token comment"># Fail on any validation error</span>
    <span class="token key atrule">validate_scripts</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>     <span class="token comment"># Validate script syntax</span>
    <span class="token key atrule">validate_colors</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>      <span class="token comment"># Validate color names</span></code>`),a(d);var h=n(d,4),K=s(h);t(K,()=>`<code class="language-yaml"><span class="token comment"># Custom validation for specific fields</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">validation</span><span class="token punctuation">:</span>
    <span class="token key atrule">custom_rules</span><span class="token punctuation">:</span>
      <span class="token punctuation">-</span> <span class="token key atrule">field</span><span class="token punctuation">:</span> <span class="token string">'refresh_interval'</span>
        <span class="token key atrule">min_value</span><span class="token punctuation">:</span> <span class="token number">100</span>           <span class="token comment"># Minimum refresh interval</span>
        <span class="token key atrule">max_value</span><span class="token punctuation">:</span> <span class="token number">300000</span>        <span class="token comment"># Maximum refresh interval</span>
      <span class="token punctuation">-</span> <span class="token key atrule">field</span><span class="token punctuation">:</span> <span class="token string">'position'</span>
        <span class="token key atrule">validate_bounds</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>    <span class="token comment"># Validate position bounds</span></code>`),a(h);var b=n(h,14),V=s(b);t(V,()=>`<code class="language-bash"><span class="token comment"># Update box content</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"UpdateBox": &#123;"box_id": "status", "content": "Connected"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock

<span class="token comment"># Replace box script</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"ReplaceScript": &#123;"box_id": "monitor", "script": ["uptime"]&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock

<span class="token comment"># Switch layouts</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"SwitchLayout": &#123;"layout_id": "dashboard"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock

<span class="token comment"># Add new box</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"AddBox": &#123;"parent_id": "main", "box": &#123;...&#125;&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock

<span class="token comment"># Control refresh</span>
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"StartRefresh": &#123;"box_id": "logs"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock
<span class="token builtin class-name">echo</span> <span class="token string">'&#123;"StopRefresh": &#123;"box_id": "logs"&#125;&#125;'</span> <span class="token operator">|</span> <span class="token function">nc</span> <span class="token parameter variable">-U</span> /tmp/boxmux.sock</code>`),a(b);var f=n(b,4),H=s(f);t(H,()=>`<code class="language-yaml"><span class="token comment"># Socket server configuration</span>
<span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">socket_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">socket_path</span><span class="token punctuation">:</span> <span class="token string">'/tmp/boxmux.sock'</span>
    <span class="token key atrule">permissions</span><span class="token punctuation">:</span> <span class="token number">0660</span>
    <span class="token key atrule">buffer_size</span><span class="token punctuation">:</span> <span class="token number">8192</span>
    <span class="token key atrule">timeout</span><span class="token punctuation">:</span> <span class="token number">5000</span>
    <span class="token key atrule">max_connections</span><span class="token punctuation">:</span> <span class="token number">10</span></code>`),a(f);var _=n(f,6),O=s(_);t(O,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'devops'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'DevOps Command Center'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token comment"># Live deployment logs</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'deploy_logs'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Deployment Logs'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 60%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">clipboard_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">scroll_config</span><span class="token punctuation">:</span>
            <span class="token key atrule">preserve_position</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">show_indicators</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> kubectl logs <span class="token punctuation">-</span>f deployment/api<span class="token punctuation">-</span>server
            
        <span class="token comment"># Performance monitoring</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'cluster_perf'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Cluster Performance'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 65%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">performance_monitoring</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> kubectl top nodes
            <span class="token punctuation">-</span> echo "<span class="token punctuation">---</span>"
            <span class="token punctuation">-</span> kubectl top pods
            
        <span class="token comment"># Interactive control box</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'controls'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Deployment Controls'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 55%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">tab_order</span><span class="token punctuation">:</span> <span class="token string">'1'</span>
          <span class="token key atrule">choices</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'scale_up'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Scale Up (3→5 replicas)'</span>
              <span class="token key atrule">script</span><span class="token punctuation">:</span>
                <span class="token punctuation">-</span> kubectl scale deployment/api<span class="token punctuation">-</span>server <span class="token punctuation">-</span><span class="token punctuation">-</span>replicas=5
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'deploy_logs'</span>
            <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'rollback'</span>
              <span class="token key atrule">content</span><span class="token punctuation">:</span> <span class="token string">'Rollback to Previous Version'</span>  
              <span class="token key atrule">script</span><span class="token punctuation">:</span>
                <span class="token punctuation">-</span> kubectl rollout undo deployment/api<span class="token punctuation">-</span>server
              <span class="token key atrule">redirect_output</span><span class="token punctuation">:</span> <span class="token string">'deploy_logs'</span></code>`),a(_);var v=n(_,4),z=s(v);t(z,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'advanced_monitor'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Advanced System Monitor'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token comment"># System logs</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'system_logs'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Logs'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 48%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 60%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">clipboard_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">scroll_config</span><span class="token punctuation">:</span>
            <span class="token key atrule">preserve_position</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">auto_scroll_new</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">scroll_buffer_size</span><span class="token punctuation">:</span> <span class="token number">2000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> tail <span class="token punctuation">-</span>f /var/log/system.log <span class="token punctuation">|</span> grep <span class="token punctuation">-</span>E "(error<span class="token punctuation">|</span>warning<span class="token punctuation">|</span>critical)"
            
        <span class="token comment"># Performance metrics with clipboard</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'perf_metrics'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Performance Metrics'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 52%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 60%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">clipboard_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">performance_monitoring</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">1000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
              echo "=== System Performance ==="
              echo "CPU: $(top -l 1 | grep "CPU usage" | awk '&#123;print $3&#125;')"
              echo "Memory: $(free | awk 'NR==2&#123;printf "%.1f%%", $3/$2*100&#125;')"
              echo "Load: $(uptime | awk -F'load average:' '&#123;print $2&#125;')"
              echo "Processes: $(ps aux | wc -l)"
              echo "BoxMux Memory: $(ps -o rss= -p $$) KB"</span>
              
        <span class="token comment"># Enhanced table with all features</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'process_table'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Top Processes'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 65%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">table_config</span><span class="token punctuation">:</span>
            <span class="token key atrule">headers</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'Command'</span><span class="token punctuation">,</span> <span class="token string">'CPU %'</span><span class="token punctuation">,</span> <span class="token string">'Memory %'</span><span class="token punctuation">,</span> <span class="token string">'PID'</span><span class="token punctuation">,</span> <span class="token string">'User'</span><span class="token punctuation">]</span>
            <span class="token key atrule">sortable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">filterable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">page_size</span><span class="token punctuation">:</span> <span class="token number">8</span>
            <span class="token key atrule">zebra_striping</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">show_row_numbers</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">border_style</span><span class="token punctuation">:</span> <span class="token string">'rounded'</span>
          <span class="token key atrule">clipboard_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">3000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> ps aux <span class="token punctuation">-</span><span class="token punctuation">-</span>no<span class="token punctuation">-</span>headers <span class="token punctuation">|</span> awk '<span class="token punctuation">&#123;</span>printf "%s<span class="token punctuation">,</span>%.1f<span class="token punctuation">,</span>%.1f<span class="token punctuation">,</span>%s<span class="token punctuation">,</span>%s&#92;n"<span class="token punctuation">,</span> $11<span class="token punctuation">,</span> $3<span class="token punctuation">,</span> $4<span class="token punctuation">,</span> $2<span class="token punctuation">,</span> $1<span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> 
              sort <span class="token punctuation">-</span>rn <span class="token punctuation">-</span>k2 <span class="token punctuation">-</span>t<span class="token punctuation">,</span> <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">20</span></code>`),a(v);var S=n(v,4),N=s(S);t(N,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'dev_monitor'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Development Environment'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token comment"># Build output</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'build_output'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Build Output'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 60%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 70%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">clipboard_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">scroll</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">scroll_config</span><span class="token punctuation">:</span>
            <span class="token key atrule">preserve_position</span><span class="token punctuation">:</span> <span class="token boolean important">false</span>  <span class="token comment"># Always show latest for builds</span>
            <span class="token key atrule">auto_scroll_new</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> cargo watch <span class="token punctuation">-</span>x build
            
        <span class="token comment"># Git status with clipboard support</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'git_status'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Git Status'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 65%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 40%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">clipboard_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">5000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> git status <span class="token punctuation">-</span><span class="token punctuation">-</span>porcelain
            <span class="token punctuation">-</span> echo "<span class="token punctuation">---</span>"
            <span class="token punctuation">-</span> git log <span class="token punctuation">-</span><span class="token punctuation">-</span>oneline <span class="token punctuation">-</span><span class="token number">5</span>
            
        <span class="token comment"># Test results with performance monitoring</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'test_results'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Test Suite'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 65%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 45%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 70%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">performance_monitoring</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">clipboard_enabled</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">10000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
              echo "Running test suite..."
              start_time=$(date +%s%3N)
              cargo test --quiet 2>&amp;1 | head -20
              end_time=$(date +%s%3N)
              duration=$((end_time - start_time))
              echo "Test duration: $&#123;duration&#125;ms"</span></code>`),a(S),q(4),j(C,x)}const tn=Object.freeze(Object.defineProperty({__proto__:null,default:Q,metadata:w},Symbol.toStringTag,{value:"Module"}));export{tn as _};
