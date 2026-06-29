import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as j,s,a as q,b as $,d as n,r as a,n as C}from"./E9AOERj2.js";import{h as e}from"./Cc2oKZLy.js";const S={title:"BoxMux Visual Themes",description:"Customizing colors, styling, and visual appearance in BoxMux terminal applications"},{title:ns,description:as}=S;var Z=j('<h1>BoxMux Visual Themes</h1> <p>BoxMux renders with readable colors out of the box and also supports full theming through YAML configuration for color schemes and styling.</p> <h2>Automatic Light/Dark Theme</h2> <p>You do not need to set any colors to get a good-looking dashboard. BoxMux picks a light or dark theme automatically and applies sensible defaults to every element — panel backgrounds, text, borders, titles, menu selections, and hover highlights — so a configuration with no color fields renders cleanly in both light and dark terminals.</p> <ul><li><p><strong>Auto-detection</strong>: The theme is chosen from the terminal (via <code>COLORFGBG</code>), defaulting to dark when it cannot be determined.</p></li> <li><p><strong>Force a theme</strong>: Pass <code>--light</code> or <code>--dark</code> on the command line to override detection:</p> <pre class="language-bash"><!></pre></li> <li><p><strong>Palette-safe highlights</strong>: Selection, active-tab, and focus colors use fixed 256-color values rather than the base 16 ANSI colors. The base 16 colors are remapped by terminal themes (for example, “bright black” is rendered as a light tint by some palettes), which can make text on a highlight unreadable; the fixed values render consistently across terminals.</p></li> <li><p><strong>Focus indication</strong>: The focused box is marked with a distinct border and tab color, so the active box is always identifiable without any configuration.</p></li> <li><p><strong>Backgrounds are real backgrounds</strong>: Empty space inside a box is painted with the box’s background color (not a foreground block glyph), so the area behind text and the empty area always match — including in selected boxes and tab bars.</p></li></ul> <p>Any color you set explicitly (see below) overrides the theme default for that element, so you can theme as much or as little as you like.</p> <h2>ANSI Color System</h2> <p>BoxMux uses the standard 16-color ANSI palette for all visual elements:</p> <h3>Standard Colors</h3> <pre class="language-yaml"><!></pre> <h3>Bright Colors</h3> <pre class="language-yaml"><!></pre> <h2>Box Styling Properties</h2> <h3>Basic Box Appearance</h3> <pre class="language-yaml"><!></pre> <h3>Focus and State Styling</h3> <pre class="language-yaml"><!></pre> <h2>Theme Examples</h2> <h3>Professional Dark Theme</h3> <pre class="language-yaml"><!></pre> <h3>Retro Terminal Theme</h3> <pre class="language-yaml"><!></pre> <h3>High Contrast Theme</h3> <pre class="language-yaml"><!></pre> <h2>PTY Visual Indicators</h2> <p>PTY-enabled boxes have special visual styling to distinguish them:</p> <pre class="language-yaml"><!></pre> <h2>State-Based Color Coding</h2> <h3>Process States</h3> <pre class="language-yaml"><!></pre> <h3>Content-Based Styling</h3> <pre class="language-yaml"><!></pre> <h2>Layout-Based Theming</h2> <h3>Multi-Layout Color Coordination</h3> <pre class="language-yaml"><!></pre> <h2>Interactive Element Styling</h2> <h3>Menu and Choice Styling</h3> <pre class="language-yaml"><!></pre> <h3>Scrollbar and Interactive Elements</h3> <pre class="language-yaml"><!></pre> <h2>Theming Techniques</h2> <h3>Conditional Color Schemes</h3> <pre class="language-yaml"><!></pre> <h3>Time-Based Themes</h3> <pre class="language-yaml"><!></pre> <h2>Z-Index and Layering</h2> <p>Control visual stacking order for overlapping boxes:</p> <pre class="language-yaml"><!></pre> <h2>Theme Best Practices</h2> <h3>Color Harmony</h3> <pre class="language-yaml"><!></pre> <h3>Accessibility Considerations</h3> <ul><li>Ensure sufficient contrast between text and background</li> <li>Use bright colors for important information</li> <li>Provide non-color indicators (symbols, text) for critical states</li> <li>Test themes in different terminal environments</li> <li>Consider colorblind-friendly color combinations</li></ul> <h3>Performance Optimization</h3> <ul><li>Use consistent colors across related boxes</li> <li>Minimize frequent color changes</li> <li>Choose colors that work well in various terminal emulators</li> <li>Test theme performance with different terminal color capabilities</li></ul> <p>BoxMux uses ANSI colors for terminal application styling.</p>',1);function J(T){var f=Z(),t=s(q(f),8),v=s(n(t),2),x=s(n(v),2),R=n(x);e(R,()=>`<code class="language-bash">boxmux layouts/dashboard.yaml <span class="token parameter variable">--dark</span>
boxmux layouts/dashboard.yaml <span class="token parameter variable">--light</span></code>`),a(x),a(v),C(6),a(t);var o=s(t,10),Y=n(o);e(Y,()=>`<code class="language-yaml"><span class="token comment"># Basic ANSI colors (8 colors)</span>
<span class="token key atrule">standard_colors</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> Black      <span class="token comment"># 0: Pure black</span>
  <span class="token punctuation">-</span> Red        <span class="token comment"># 1: Dark red  </span>
  <span class="token punctuation">-</span> Green      <span class="token comment"># 2: Dark green</span>
  <span class="token punctuation">-</span> Yellow     <span class="token comment"># 3: Dark yellow/brown</span>
  <span class="token punctuation">-</span> Blue       <span class="token comment"># 4: Dark blue</span>
  <span class="token punctuation">-</span> Magenta    <span class="token comment"># 5: Dark magenta/purple</span>
  <span class="token punctuation">-</span> Cyan       <span class="token comment"># 6: Dark cyan</span>
  <span class="token punctuation">-</span> White      <span class="token comment"># 7: Light gray</span></code>`),a(o);var p=s(o,4),A=n(p);e(A,()=>`<code class="language-yaml"><span class="token comment"># Bright ANSI colors (8 additional colors)</span>
<span class="token key atrule">bright_colors</span><span class="token punctuation">:</span>
  <span class="token punctuation">-</span> BrightBlack    <span class="token comment"># 8: Dark gray</span>
  <span class="token punctuation">-</span> BrightRed      <span class="token comment"># 9: Bright red</span>
  <span class="token punctuation">-</span> BrightGreen    <span class="token comment"># 10: Bright green  </span>
  <span class="token punctuation">-</span> BrightYellow   <span class="token comment"># 11: Bright yellow</span>
  <span class="token punctuation">-</span> BrightBlue     <span class="token comment"># 12: Bright blue</span>
  <span class="token punctuation">-</span> BrightMagenta  <span class="token comment"># 13: Bright magenta</span>
  <span class="token punctuation">-</span> BrightCyan     <span class="token comment"># 14: Bright cyan</span>
  <span class="token punctuation">-</span> BrightWhite    <span class="token comment"># 15: Pure white</span></code>`),a(p);var l=s(p,6),M=n(l);e(M,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">styled_box</span><span class="token punctuation">:</span>
    <span class="token comment"># Border and outline styling</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>         <span class="token comment"># Box border color</span>
    <span class="token key atrule">title_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>        <span class="token comment"># Title text color</span>
    
    <span class="token comment"># Content area styling  </span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>    <span class="token comment"># Text color</span>
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>          <span class="token comment"># Background fill</span>
    
    <span class="token comment"># Fill characters for empty space</span>
    <span class="token key atrule">fill_char</span><span class="token punctuation">:</span> <span class="token string">" "</span>                     <span class="token comment"># Default fill character</span>
    <span class="token key atrule">selected_fill_char</span><span class="token punctuation">:</span> <span class="token string">"░"</span>            <span class="token comment"># Fill when box is selected/focused</span></code>`),a(l);var c=s(l,4),G=n(c);e(G,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">interactive_box</span><span class="token punctuation">:</span>
    <span class="token comment"># Base styling</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"White"</span>
    <span class="token key atrule">foreground_color</span><span class="token punctuation">:</span> <span class="token string">"White"</span>
    
    <span class="token comment"># Focus indication (automatic when focusable: true)</span>
    <span class="token key atrule">focusable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    
    <span class="token comment"># Selection state styling</span>
    <span class="token key atrule">selected_fill_char</span><span class="token punctuation">:</span> <span class="token string">"▓"</span>            <span class="token comment"># Highlighted fill pattern</span></code>`),a(c);var r=s(c,6),P=n(r);e(P,()=>`<code class="language-yaml"><span class="token comment"># Professional dark theme with blue accents</span>
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
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span></code>`),a(r);var i=s(r,4),D=n(i);e(D,()=>`<code class="language-yaml"><span class="token comment"># Retro green-on-black terminal theme</span>
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
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span></code>`),a(i);var k=s(i,4),W=n(k);e(W,()=>`<code class="language-yaml"><span class="token comment"># High contrast theme for accessibility</span>
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
    <span class="token key atrule">background_color</span><span class="token punctuation">:</span> <span class="token string">"Black"</span></code>`),a(k);var u=s(k,6),E=n(u);e(E,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">terminal_box</span><span class="token punctuation">:</span>
    <span class="token key atrule">pty</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    
    <span class="token comment"># PTY visual indicators (automatic)</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"⚡ Interactive Terminal"</span>      <span class="token comment"># Lightning bolt prefix</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>           <span class="token comment"># Distinctive cyan border</span>
    
    <span class="token comment"># PTY status color coding (automatic based on state)</span>
    <span class="token comment"># - Running: BrightCyan</span>
    <span class="token comment"># - Error: BrightRed  </span>
    <span class="token comment"># - Finished: BrightBlack</span>
    <span class="token comment"># - Dead: Red</span></code>`),a(u);var g=s(u,6),I=n(g);e(I,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">process_monitor</span><span class="token punctuation">:</span>
    <span class="token comment"># Colors change automatically based on process state</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token string">"some_long_running_process.sh"</span>
    
    <span class="token comment"># State colors (automatic):</span>
    <span class="token comment"># - Running: Normal colors</span>
    <span class="token comment"># - Error: BrightRed border/text</span>
    <span class="token comment"># - Success: BrightGreen accents</span>
    <span class="token comment"># - Waiting: BrightYellow indicators</span></code>`),a(g);var m=s(g,4),N=n(m);e(N,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">log_viewer</span><span class="token punctuation">:</span>
    <span class="token key atrule">script</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      #!/bin/bash
      # Script output colors are preserved
      echo -e "\\033[32mSUCCESS: Operation completed\\033[0m"
      echo -e "\\033[33mWARNING: Check configuration\\033[0m" 
      echo -e "\\033[31mERROR: Connection failed\\033[0m"</span>
      
    <span class="token comment"># ANSI escape sequences in output are processed</span>
    <span class="token comment"># Colors appear correctly in the terminal</span></code>`),a(m);var d=s(m,6),U=n(d);e(U,()=>`<code class="language-yaml"><span class="token comment"># Consistent theming across layouts</span>
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
        <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;error_color&#125;&#125;"</span></code>`),a(d);var h=s(d,6),F=n(h);e(F,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">main_menu</span><span class="token punctuation">:</span>
    <span class="token key atrule">choices</span><span class="token punctuation">:</span>
      <span class="token punctuation">-</span> <span class="token key atrule">name</span><span class="token punctuation">:</span> <span class="token string">"Deploy Application"</span>
        <span class="token comment"># Choice styling is inherited from parent box</span>
        
    <span class="token comment"># Menu-specific colors</span>
    <span class="token key atrule">choice_color</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>          <span class="token comment"># Unselected choices</span>
    <span class="token key atrule">selected_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>       <span class="token comment"># Currently selected choice</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>           <span class="token comment"># Menu border</span></code>`),a(h);var y=s(h,4),L=n(y);e(L,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">scrollable_content</span><span class="token punctuation">:</span>
    <span class="token key atrule">overflow_behavior</span><span class="token punctuation">:</span> <span class="token string">"scroll"</span>
    
    <span class="token comment"># Scrollbar styling (automatic based on box colors)</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightWhite"</span>          <span class="token comment"># Scrollbar uses box border color</span>
    <span class="token comment"># Scrollbar components:</span>
    <span class="token comment"># - Track: Uses fill_char</span>
    <span class="token comment"># - Thumb: Uses selected_fill_char</span>
    <span class="token comment"># - Arrows: Use border_color</span></code>`),a(y);var b=s(y,6),z=n(b);e(z,()=>`<code class="language-yaml"><span class="token comment"># Environment-based theming</span>
<span class="token key atrule">variables</span><span class="token punctuation">:</span>
  <span class="token key atrule">env</span><span class="token punctuation">:</span> <span class="token string">"&#123;&#123;ENV&#125;&#125;"</span>  <span class="token comment"># production, staging, development</span>
  
<span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">status_indicator</span><span class="token punctuation">:</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">"Environment: &#123;&#123;env&#125;&#125;"</span>
    <span class="token comment"># Use different colors based on environment</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
      &#123;% if env == "production" %&#125;BrightRed&#123;% elif env == "staging" %&#125;BrightYellow&#123;% else %&#125;BrightGreen&#123;% endif %&#125;</span></code>`),a(b);var _=s(b,4),O=n(_);e(O,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
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
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>  <span class="token comment"># Could be dynamic based on conditions</span></code>`),a(_);var B=s(_,6),H=n(B);e(H,()=>`<code class="language-yaml"><span class="token key atrule">boxes</span><span class="token punctuation">:</span>
  <span class="token key atrule">background_box</span><span class="token punctuation">:</span>
    <span class="token key atrule">z_index</span><span class="token punctuation">:</span> <span class="token number">1</span>                           <span class="token comment"># Behind other boxes</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlack"</span>
    
  <span class="token key atrule">main_content</span><span class="token punctuation">:</span>
    <span class="token key atrule">z_index</span><span class="token punctuation">:</span> <span class="token number">5</span>                           <span class="token comment"># Normal layer</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>
    
  <span class="token key atrule">popup_overlay</span><span class="token punctuation">:</span>
    <span class="token key atrule">z_index</span><span class="token punctuation">:</span> <span class="token number">10</span>                          <span class="token comment"># Above other boxes</span>
    <span class="token key atrule">border_color</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span></code>`),a(B);var w=s(B,6),V=n(w);e(V,()=>`<code class="language-yaml"><span class="token comment"># Use related colors for cohesive appearance</span>
<span class="token key atrule">theme_palette</span><span class="token punctuation">:</span>
  <span class="token key atrule">primary</span><span class="token punctuation">:</span> <span class="token string">"BrightBlue"</span>        <span class="token comment"># Main accent color</span>
  <span class="token key atrule">secondary</span><span class="token punctuation">:</span> <span class="token string">"BrightCyan"</span>      <span class="token comment"># Related cool color</span>
  <span class="token key atrule">neutral</span><span class="token punctuation">:</span> <span class="token string">"White"</span>             <span class="token comment"># Text and borders</span>
  <span class="token key atrule">background</span><span class="token punctuation">:</span> <span class="token string">"Black"</span>          <span class="token comment"># Consistent background</span>
  <span class="token key atrule">success</span><span class="token punctuation">:</span> <span class="token string">"BrightGreen"</span>       <span class="token comment"># Status indication</span>
  <span class="token key atrule">warning</span><span class="token punctuation">:</span> <span class="token string">"BrightYellow"</span>      <span class="token comment"># Attention grabbing</span>
  <span class="token key atrule">danger</span><span class="token punctuation">:</span> <span class="token string">"BrightRed"</span>          <span class="token comment"># Error states</span></code>`),a(w),C(10),$(T,f)}const es=Object.freeze(Object.defineProperty({__proto__:null,default:J,metadata:S},Symbol.toStringTag,{value:"Module"}));export{es as _};
