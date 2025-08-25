import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as H,s,a as L,b as F,d as n,r as a,n as O}from"./E9AOERj2.js";import{h as e}from"./Cc2oKZLy.js";const v={title:"BoxMux Visual Themes",description:"Customizing colors, styling, and visual appearance in BoxMux terminal applications"},{title:K,description:Q}=v;var V=H('<h1>BoxMux Visual Themes</h1> <p>BoxMux supports theming through YAML configuration for terminal applications with color schemes and styling.</p> <h2>ANSI Color System</h2> <p>BoxMux uses the standard 16-color ANSI palette for all visual elements:</p> <h3>Standard Colors</h3> <pre class="language-yaml"><!></pre> <h3>Bright Colors</h3> <pre class="language-yaml"><!></pre> <h2>Box Styling Properties</h2> <h3>Basic Box Appearance</h3> <pre class="language-yaml"><!></pre> <h3>Focus and State Styling</h3> <pre class="language-yaml"><!></pre> <h2>Theme Examples</h2> <h3>Professional Dark Theme</h3> <pre class="language-yaml"><!></pre> <h3>Retro Terminal Theme</h3> <pre class="language-yaml"><!></pre> <h3>High Contrast Theme</h3> <pre class="language-yaml"><!></pre> <h2>PTY Visual Indicators</h2> <p>PTY-enabled boxes have special visual styling to distinguish them:</p> <pre class="language-yaml"><!></pre> <h2>State-Based Color Coding</h2> <h3>Process States</h3> <pre class="language-yaml"><!></pre> <h3>Content-Based Styling</h3> <pre class="language-yaml"><!></pre> <h2>Layout-Based Theming</h2> <h3>Multi-Layout Color Coordination</h3> <pre class="language-yaml"><!></pre> <h2>Interactive Element Styling</h2> <h3>Menu and Choice Styling</h3> <pre class="language-yaml"><!></pre> <h3>Scrollbar and Interactive Elements</h3> <pre class="language-yaml"><!></pre> <h2>Theming Techniques</h2> <h3>Conditional Color Schemes</h3> <pre class="language-yaml"><!></pre> <h3>Time-Based Themes</h3> <pre class="language-yaml"><!></pre> <h2>Z-Index and Layering</h2> <p>Control visual stacking order for overlapping boxes:</p> <pre class="language-yaml"><!></pre> <h2>Theme Best Practices</h2> <h3>Color Harmony</h3> <pre class="language-yaml"><!></pre> <h3>Accessibility Considerations</h3> <ul><li>Ensure sufficient contrast between text and background</li> <li>Use bright colors for important information</li> <li>Provide non-color indicators (symbols, text) for critical states</li> <li>Test themes in different terminal environments</li> <li>Consider colorblind-friendly color combinations</li></ul> <h3>Performance Optimization</h3> <ul><li>Use consistent colors across related boxes</li> <li>Minimize frequent color changes</li> <li>Choose colors that work well in various terminal emulators</li> <li>Test theme performance with different terminal color capabilities</li></ul> <p>BoxMux uses ANSI colors for terminal application styling.</p>',1);function j(x){var B=V(),t=s(L(B),10),C=n(t);e(C,()=>`<code class="language-yaml"><span class="token comment"># Basic ANSI colors (8 colors)</span>
<span class="token key atrule">standard_colors</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> Black      <span class="token comment"># 0: Pure black</span>
  <span class="token punctuation">-</span> Red        <span class="token comment"># 1: Dark red  </span>
  <span class="token punctuation">-</span> Green      <span class="token comment"># 2: Dark green</span>
  <span class="token punctuation">-</span> Yellow     <span class="token comment"># 3: Dark yellow/brown</span>
  <span class="token punctuation">-</span> Blue       <span class="token comment"># 4: Dark blue</span>
  <span class="token punctuation">-</span> Magenta    <span class="token comment"># 5: Dark magenta/purple</span>
  <span class="token punctuation">-</span> Cyan       <span class="token comment"># 6: Dark cyan</span>
  <span class="token punctuation">-</span> White      <span class="token comment"># 7: Light gray</span></code>`),a(t);var o=s(t,4),w=n(o);e(w,()=>`<code class="language-yaml"><span class="token comment"># Bright ANSI colors (8 additional colors)</span>
<span class="token key atrule">bright_colors</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> BrightBlack    <span class="token comment"># 8: Dark gray</span>
  <span class="token punctuation">-</span> BrightRed      <span class="token comment"># 9: Bright red</span>
  <span class="token punctuation">-</span> BrightGreen    <span class="token comment"># 10: Bright green  </span>
  <span class="token punctuation">-</span> BrightYellow   <span class="token comment"># 11: Bright yellow</span>
  <span class="token punctuation">-</span> BrightBlue     <span class="token comment"># 12: Bright blue</span>
  <span class="token punctuation">-</span> BrightMagenta  <span class="token comment"># 13: Bright magenta</span>
  <span class="token punctuation">-</span> BrightCyan     <span class="token comment"># 14: Bright cyan</span>
  <span class="token punctuation">-</span> BrightWhite    <span class="token comment"># 15: Pure white</span></code>`),a(o);var p=s(o,6),S=n(p);e(S,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">styled_box</span><span class="token punctuation">:</span>
    <span class="token comment"># Border and outline styling</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>         <span class="token comment"># Box border color</span>
    <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>        <span class="token comment"># Title text color</span>
    
    <span class="token comment"># Content area styling  </span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>    <span class="token comment"># Text color</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>          <span class="token comment"># Background fill</span>
    
    <span class="token comment"># Fill characters for empty space</span>
    <span class="token key atrule">fill_char</span><span class="token punctuation">:</span> <span class="token string">" "</span>                     <span class="token comment"># Default fill character</span>
    <span class="token key atrule">selected_fill_char</span><span class="token punctuation">:</span> <span class="token string">"░"</span>            <span class="token comment"># Fill when box is selected/focused</span></code>`),a(p);var l=s(p,4),T=n(l);e(T,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">interactive_box</span><span class="token punctuation">:</span>
    <span class="token comment"># Base styling</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"White"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"White"</span>
    
    <span class="token comment"># Focus indication (automatic when focusable: true)</span>
    <span class="token key atrule">focusable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    
    <span class="token comment"># Selection state styling</span>
    <span class="token key atrule">selected_fill_char</span><span class="token punctuation">:</span> <span class="token string">"▓"</span>            <span class="token comment"># Highlighted fill pattern</span></code>`),a(l);var c=s(l,6),R=n(c);e(R,()=>`<code class="language-yaml"><span class="token comment"># Professional dark theme with blue accents</span>
<span class="token key atrule">theme</span><span class="token punctuation">:</span>
  <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"Professional Dark"</span>
  
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">header</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
    <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>
    
  <span class="token key atrule">content</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"White"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>
    
  <span class="token key atrule">status</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>
    
  <span class="token key atrule">warning</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>
    
  <span class="token key atrule">error</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightRed"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightRed"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span></code>`),a(c);var r=s(c,4),Y=n(r);e(Y,()=>`<code class="language-yaml"><span class="token comment"># Retro green-on-black terminal theme</span>
<span class="token key atrule">theme</span><span class="token punctuation">:</span>
  <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"Retro Terminal"</span>
  
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">main</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
    <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"Green"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>
    <span class="token key atrule">fill_char</span><span class="token punctuation">:</span> <span class="token string">" "</span>
    <span class="token key atrule">selected_fill_char</span><span class="token punctuation">:</span> <span class="token string">"░"</span>
    
  <span class="token key atrule">secondary</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"Green"</span>
    <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"Green"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span></code>`),a(r);var k=s(r,4),M=n(k);e(M,()=>`<code class="language-yaml"><span class="token comment"># High contrast theme for accessibility</span>
<span class="token key atrule">theme</span><span class="token punctuation">:</span>
  <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"High Contrast"</span>
  
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">primary</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>
    <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>
    
  <span class="token key atrule">secondary</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
    <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>
    
  <span class="token key atrule">accent</span><span class="token punctuation">:</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span></code>`),a(k);var u=s(k,6),G=n(u);e(G,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">terminal_box</span><span class="token punctuation">:</span>
    <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    
    <span class="token comment"># PTY visual indicators (automatic)</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"⚡ Interactive Terminal"</span>      <span class="token comment"># Lightning bolt prefix</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>           <span class="token comment"># Distinctive cyan border</span>
    
    <span class="token comment"># PTY status color coding (automatic based on state)</span>
    <span class="token comment"># - Running: BrightCyan</span>
    <span class="token comment"># - Error: BrightRed  </span>
    <span class="token comment"># - Finished: BrightBlack</span>
    <span class="token comment"># - Dead: Red</span></code>`),a(u);var i=s(u,6),A=n(i);e(A,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">process_monitor</span><span class="token punctuation">:</span>
    <span class="token comment"># Colors change automatically based on process state</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"some_long_running_process.sh"</span>
    
    <span class="token comment"># State colors (automatic):</span>
    <span class="token comment"># - Running: Normal colors</span>
    <span class="token comment"># - Error: BrightRed border/text</span>
    <span class="token comment"># - Success: BrightGreen accents</span>
    <span class="token comment"># - Waiting: BrightYellow indicators</span></code>`),a(i);var g=s(i,4),P=n(g);e(P,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">log_viewer</span><span class="token punctuation">:</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/bin/bash
      # Script output colors are preserved
      echo -e "\\033[32mSUCCESS: Operation completed\\033[0m"
      echo -e "\\033[33mWARNING: Check configuration\\033[0m" 
      echo -e "\\033[31mERROR: Connection failed\\033[0m"</span>
      
    <span class="token comment"># ANSI escape sequences in output are processed</span>
    <span class="token comment"># Colors appear correctly in the terminal</span></code>`),a(g);var m=s(g,6),W=n(m);e(W,()=>`<code class="language-yaml"><span class="token comment"># Consistent theming across layouts</span>
<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">primary_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
  <span class="token key atrule">accent_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>
  <span class="token key atrule">success_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>
  <span class="token key atrule">warning_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>
  <span class="token key atrule">error_color</span><span class="token punctuation">:</span> <span class="token string">"BrightRed"</span>

<span class="token key atrule">layouts</span><span class="token punctuation">:</span>
  <span class="token key atrule">dashboard</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">header</span><span class="token punctuation">:</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;primary_color&#125;&#125;"</span>
        <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;accent_color&#125;&#125;"</span>
        
      <span class="token key atrule">status</span><span class="token punctuation">:</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;success_color&#125;&#125;"</span>
        
  <span class="token key atrule">monitoring</span><span class="token punctuation">:</span>
    <span class="token key atrule">type</span><span class="token punctuation">:</span> <span class="token string">"MuxBox"</span>
    <span class="token key atrule">children</span><span class="token punctuation">:</span>
      <span class="token key atrule">alerts</span><span class="token punctuation">:</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;warning_color&#125;&#125;"</span>
        
      <span class="token key atrule">errors</span><span class="token punctuation">:</span>
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;error_color&#125;&#125;"</span></code>`),a(m);var y=s(m,6),D=n(y);e(D,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">main_menu</span><span class="token punctuation">:</span>
    <span class="token key atrule">choices</span><span class="token punctuation">:</span>
      <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"Deploy Application"</span>
        <span class="token comment"># Choice styling is inherited from parent box</span>
        
    <span class="token comment"># Menu-specific colors</span>
    <span class="token key atrule">choice_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>          <span class="token comment"># Unselected choices</span>
    <span class="token key atrule">selected_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>       <span class="token comment"># Currently selected choice</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>           <span class="token comment"># Menu border</span></code>`),a(y);var h=s(y,4),E=n(h);e(E,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">scrollable_content</span><span class="token punctuation">:</span>
    <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>
    
    <span class="token comment"># Scrollbar styling (automatic based on box colors)</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>          <span class="token comment"># Scrollbar uses box border color</span>
    <span class="token comment"># Scrollbar components:</span>
    <span class="token comment"># - Track: Uses fill_char</span>
    <span class="token comment"># - Thumb: Uses selected_fill_char</span>
    <span class="token comment"># - Arrows: Use border_color</span></code>`),a(h);var d=s(h,6),I=n(d);e(I,()=>`<code class="language-yaml"><span class="token comment"># Environment-based theming</span>
<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">env</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;ENV&#125;&#125;"</span>  <span class="token comment"># production, staging, development</span>
  
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">status_indicator</span><span class="token punctuation">:</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Environment: &#123;&#123;env&#125;&#125;"</span>
    <span class="token comment"># Use different colors based on environment</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      &#123;% if env == "production" %&#125;BrightRed&#123;% elif env == "staging" %&#125;BrightYellow&#123;% else %&#125;BrightGreen&#123;% endif %&#125;</span></code>`),a(d);var _=s(d,4),N=n(_);e(N,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">time_sensitive</span><span class="token punctuation">:</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/bin/bash
      hour=$(date +%H)
      if [ $hour -lt 6 ] || [ $hour -gt 18 ]; then
        echo "🌙 Night mode active"
      else
        echo "☀️ Day mode active"  
      fi</span>
    
    <span class="token comment"># Adjust colors based on script output or time</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>  <span class="token comment"># Could be dynamic based on conditions</span></code>`),a(_);var b=s(_,6),U=n(b);e(U,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">background_box</span><span class="token punctuation">:</span>
    <span class="token key atrule">z_index</span><span class="token punctuation">:</span> <span class="token number">1</span>                           <span class="token comment"># Behind other boxes</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlack"</span>
    
  <span class="token key atrule">main_content</span><span class="token punctuation">:</span>
    <span class="token key atrule">z_index</span><span class="token punctuation">:</span> <span class="token number">5</span>                           <span class="token comment"># Normal layer</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
    
  <span class="token key atrule">popup_overlay</span><span class="token punctuation">:</span>
    <span class="token key atrule">z_index</span><span class="token punctuation">:</span> <span class="token number">10</span>                          <span class="token comment"># Above other boxes</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span></code>`),a(b);var f=s(b,6),z=n(f);e(z,()=>`<code class="language-yaml"><span class="token comment"># Use related colors for cohesive appearance</span>
<span class="token key atrule">theme_palette</span><span class="token punctuation">:</span>
  <span class="token key atrule">primary</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>        <span class="token comment"># Main accent color</span>
  <span class="token key atrule">secondary</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>      <span class="token comment"># Related cool color</span>
  <span class="token key atrule">neutral</span><span class="token punctuation">:</span> <span class="token string">"White"</span>             <span class="token comment"># Text and borders</span>
  <span class="token key atrule">background</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>          <span class="token comment"># Consistent background</span>
  <span class="token key atrule">success</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>       <span class="token comment"># Status indication</span>
  <span class="token key atrule">warning</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>      <span class="token comment"># Attention grabbing</span>
  <span class="token key atrule">danger</span><span class="token punctuation">:</span> <span class="token string">"BrightRed"</span>          <span class="token comment"># Error states</span></code>`),a(f),O(10),F(x,B)}const X=Object.freeze(Object.defineProperty({__proto__:null,default:j,metadata:v},Symbol.toStringTag,{value:"Module"}));export{X as _};
