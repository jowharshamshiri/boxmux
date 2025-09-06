import"./DsnmJJEf.js";import"./8zndd0xi.js";import{f as P,s as n,a as S,b as D,d as s,r as a,n as T}from"./E9AOERj2.js";import{h as t}from"./Cc2oKZLy.js";const g={title:"Data Visualization",description:"Data visualization through charts and tables with features for displaying, sorting, and filtering structured data, including real-time updates."},{title:M,description:O}=g;var U=P('<h2>Table of Contents</h2> <ul><li><a href="#chart-system">Chart System</a></li> <li><a href="#table-system">Table System</a></li> <li><a href="#performance-features">Performance Features</a></li> <li><a href="#real-world-examples">Real-World Examples</a></li> <li><a href="#best-practices">Best Practices</a></li></ul> <h2>Chart System</h2> <p>BoxMux includes a Unicode-based charting system with responsive layout and multiple chart types.</p> <h3>Chart Types</h3> <h4>Bar Charts</h4> <p>Perfect for categorical data comparison:</p> <pre class="language-yaml"><!></pre> <h4>Line Charts</h4> <p>Ideal for time-series data and trends:</p> <pre class="language-yaml"><!></pre> <h4>Histograms</h4> <p>Great for distribution visualization:</p> <pre class="language-yaml"><!></pre> <h3>Chart Layout System</h3> <p>BoxMux includes a smart chart layout engine with responsive design:</p> <ul><li><strong>Automatic sizing</strong>: Charts adapt to box dimensions</li> <li><strong>Smart alignment</strong>: Proper axis alignment and labeling</li> <li><strong>Responsive scaling</strong>: Charts scale with terminal size changes</li> <li><strong>Unicode rendering</strong>: High-quality Unicode-based chart display</li></ul> <h3>Chart Configuration Options</h3> <table><thead><tr><th>Property</th><th>Type</th><th>Default</th><th>Description</th></tr></thead><tbody><tr><td><code>chart_type</code></td><td>string</td><td>-</td><td>Chart type: ‘bar’, ‘line’, ‘histogram’</td></tr><tr><td><code>width</code></td><td>number</td><td>40</td><td>Chart width in characters</td></tr><tr><td><code>height</code></td><td>number</td><td>10</td><td>Chart height in lines</td></tr><tr><td><code>title</code></td><td>string</td><td>-</td><td>Chart title</td></tr><tr><td><code>x_label</code></td><td>string</td><td>-</td><td>X-axis label</td></tr><tr><td><code>y_label</code></td><td>string</td><td>-</td><td>Y-axis label</td></tr></tbody></table> <h3>Dynamic Chart Data</h3> <p>Charts can display live data from scripts:</p> <pre class="language-yaml"><!></pre> <h2>Table System</h2> <p>BoxMux provides a table system for displaying structured data.</p> <h3>Table Features</h3> <ul><li><strong>Data Parsing</strong>: Support for CSV and JSON data formats</li> <li><strong>Sorting</strong>: Text and numeric sorting with ascending/descending support</li> <li><strong>Filtering</strong>: Exact match and case-insensitive search</li> <li><strong>Pagination</strong>: Configurable page sizes with navigation info</li> <li><strong>Visual Enhancement</strong>: Zebra striping, row numbers, multiple border styles</li> <li><strong>Column Management</strong>: Width calculation with max width constraints</li></ul> <h3>Table Configuration</h3> <table><thead><tr><th>Property</th><th>Type</th><th>Default</th><th>Description</th></tr></thead><tbody><tr><td><code>headers</code></td><td>array[string]</td><td>-</td><td>Column headers</td></tr><tr><td><code>sortable</code></td><td>boolean</td><td>false</td><td>Enable column sorting</td></tr><tr><td><code>filterable</code></td><td>boolean</td><td>false</td><td>Enable row filtering</td></tr><tr><td><code>page_size</code></td><td>number</td><td>10</td><td>Rows per page</td></tr><tr><td><code>show_row_numbers</code></td><td>boolean</td><td>false</td><td>Display row numbers</td></tr><tr><td><code>zebra_striping</code></td><td>boolean</td><td>false</td><td>Alternating row colors</td></tr><tr><td><code>border_style</code></td><td>string</td><td>‘single’</td><td>Border style</td></tr></tbody></table> <h3>Border Styles</h3> <p>BoxMux supports multiple table border styles:</p> <ul><li><code>none</code>: No borders</li> <li><code>single</code>: Single-line borders (default)</li> <li><code>double</code>: Double-line borders</li> <li><code>rounded</code>: Rounded corner borders</li> <li><code>thick</code>: Thick-line borders</li> <li><code>custom</code>: Custom border characters</li></ul> <h3>Data Formats</h3> <h4>CSV Format</h4> <pre class="language-yaml"><!></pre> <h4>JSON Format</h4> <pre class="language-yaml"><!></pre> <h3>Table Operations</h3> <h4>Sorting</h4> <p>Tables support intelligent sorting:</p> <ul><li><strong>Numeric sorting</strong>: Automatically detects and sorts numeric values</li> <li><strong>Text sorting</strong>: Case-insensitive text sorting</li> <li><strong>Ascending/Descending</strong>: Toggle sort direction</li> <li><strong>Multi-column</strong>: Sort by multiple columns (planned feature)</li></ul> <h4>Filtering</h4> <p>Tables include flexible filtering:</p> <ul><li><strong>Exact match</strong>: Find exact value matches</li> <li><strong>Case-insensitive</strong>: Search ignoring case</li> <li><strong>Column-specific</strong>: Filter specific columns</li> <li><strong>Real-time</strong>: Filter results update immediately</li></ul> <h4>Pagination</h4> <p>Table pagination features:</p> <ul><li><strong>Configurable page size</strong>: Set rows per page</li> <li><strong>Navigation info</strong>: Current page and total pages</li> <li><strong>Keyboard navigation</strong>: Page Up/Down support</li> <li><strong>Auto-sizing</strong>: Page size adapts to box height</li></ul> <h2>Performance Features</h2> <h3>Efficient Rendering</h3> <ul><li><strong>Unicode optimization</strong>: Fast Unicode character rendering</li> <li><strong>Memory efficiency</strong>: Minimal memory usage for large datasets</li> <li><strong>Incremental updates</strong>: Only redraw changed content</li> <li><strong>Viewport culling</strong>: Only render visible content</li></ul> <h3>Data Processing</h3> <ul><li><strong>Stream processing</strong>: Handle large datasets efficiently</li> <li><strong>Lazy loading</strong>: Load data on demand</li> <li><strong>Caching</strong>: Cache parsed data for faster access</li> <li><strong>Incremental parsing</strong>: Parse data incrementally</li></ul> <h2>Real-World Examples</h2> <h3>System Monitor Dashboard</h3> <pre class="language-yaml"><!></pre> <h3>Network Monitoring</h3> <pre class="language-yaml"><!></pre> <h3>Database Analytics</h3> <pre class="language-yaml"><!></pre> <h2>Best Practices</h2> <h3>Chart Design</h3> <ol><li><strong>Choose appropriate chart types</strong>: Bar charts for comparisons, line charts for trends, histograms for distributions</li> <li><strong>Size appropriately</strong>: Balance chart detail with box space</li> <li><strong>Use meaningful titles</strong>: Clear chart and axis titles</li> <li><strong>Consider refresh rates</strong>: Balance real-time updates with performance</li> <li><strong>Test with real data</strong>: Verify charts with actual data ranges</li></ol> <h3>Table Design</h3> <ol><li><strong>Limit column count</strong>: Keep tables readable with appropriate column counts</li> <li><strong>Use appropriate page sizes</strong>: Balance data visibility with performance</li> <li><strong>Enable sorting for numeric data</strong>: Make numeric columns sortable</li> <li><strong>Consider zebra striping</strong>: Improve readability for wide tables</li> <li><strong>Use filtering for large datasets</strong>: Help users find relevant data quickly</li></ol> <h3>Performance Optimization</h3> <ol><li><strong>Optimize data scripts</strong>: Use efficient commands to generate data</li> <li><strong>Cache expensive operations</strong>: Cache slow database queries</li> <li><strong>Limit data volume</strong>: Use appropriate limits in data generation scripts</li> <li><strong>Use appropriate refresh intervals</strong>: Balance freshness with system load</li> <li><strong>Monitor memory usage</strong>: Watch memory consumption with large datasets</li></ol> <h3>Data Quality</h3> <ol><li><strong>Validate data formats</strong>: Ensure scripts produce consistent formats</li> <li><strong>Handle missing data</strong>: Gracefully handle missing or invalid data</li> <li><strong>Error handling</strong>: Include error handling in data generation scripts</li> <li><strong>Test edge cases</strong>: Verify behavior with empty or malformed data</li> <li><strong>Document data sources</strong>: Clear documentation for data generation logic</li></ol> <hr/> <p>For configuration details, see <a href="configuration.md">Configuration Reference</a>.<br/> For implementation examples, see <a href="user-guide.md">User Guide</a>.</p>',1);function E(y){var k=U(),e=n(S(k),14),h=s(e);t(h,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'resource_usage'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Resource Usage'</span>
  <span class="token key atrule">chart_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">chart_type</span><span class="token punctuation">:</span> <span class="token string">'bar'</span>
    <span class="token key atrule">width</span><span class="token punctuation">:</span> <span class="token number">50</span>
    <span class="token key atrule">height</span><span class="token punctuation">:</span> <span class="token number">15</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Resources'</span>
    <span class="token key atrule">x_label</span><span class="token punctuation">:</span> <span class="token string">'Resource'</span>
    <span class="token key atrule">y_label</span><span class="token punctuation">:</span> <span class="token string">'Usage %'</span>
  <span class="token key atrule">chart_data</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
    CPU,67
    Memory,45
    Disk,89
    Network,23</span></code>`),a(e);var p=n(e,6),m=s(p);t(m,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'cpu_trend'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'CPU Usage Over Time'</span>
  <span class="token key atrule">chart_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">chart_type</span><span class="token punctuation">:</span> <span class="token string">'line'</span>
    <span class="token key atrule">width</span><span class="token punctuation">:</span> <span class="token number">60</span>
    <span class="token key atrule">height</span><span class="token punctuation">:</span> <span class="token number">12</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'CPU Trend'</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> top <span class="token punctuation">-</span>l 1 <span class="token punctuation">|</span> grep "CPU usage" <span class="token punctuation">|</span> awk '<span class="token punctuation">&#123;</span>print NR"<span class="token punctuation">,</span>"$3<span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> sed 's/%//'</code>`),a(p);var o=n(p,6),b=s(o);t(b,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'response_times'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Response Time Distribution'</span>
  <span class="token key atrule">chart_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">chart_type</span><span class="token punctuation">:</span> <span class="token string">'histogram'</span>
    <span class="token key atrule">width</span><span class="token punctuation">:</span> <span class="token number">45</span>
    <span class="token key atrule">height</span><span class="token punctuation">:</span> <span class="token number">10</span>
    <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'API Response Times'</span>
  <span class="token key atrule">chart_data</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
    0-100ms,45
    100-200ms,67
    200-500ms,23
    500ms+,5</span></code>`),a(o);var l=n(o,16),f=s(l);t(f,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'memory_chart'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Live Memory Usage'</span>
  <span class="token key atrule">chart_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">chart_type</span><span class="token punctuation">:</span> <span class="token string">'line'</span>
    <span class="token key atrule">width</span><span class="token punctuation">:</span> <span class="token number">50</span>
    <span class="token key atrule">height</span><span class="token punctuation">:</span> <span class="token number">15</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">1000</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
      # Generate timestamp and memory usage
      timestamp=$(date +%s)
      memory=$(free | awk 'NR==2&#123;printf "%.1f", $3/$2 * 100.0&#125;')
      echo "$&#123;timestamp&#125;,$&#123;memory&#125;"</span></code>`),a(l);var c=n(l,24),_=s(c);t(_,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'process_table'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Processes'</span>
  <span class="token key atrule">table_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">headers</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'Process'</span><span class="token punctuation">,</span> <span class="token string">'CPU %'</span><span class="token punctuation">,</span> <span class="token string">'Memory'</span><span class="token punctuation">,</span> <span class="token string">'PID'</span><span class="token punctuation">]</span>
    <span class="token key atrule">sortable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">filterable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">page_size</span><span class="token punctuation">:</span> <span class="token number">15</span>
    <span class="token key atrule">zebra_striping</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
  <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">5000</span>
  <span class="token key atrule">script</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> ps aux <span class="token punctuation">-</span><span class="token punctuation">-</span>no<span class="token punctuation">-</span>headers <span class="token punctuation">|</span> awk '<span class="token punctuation">&#123;</span>printf "%s<span class="token punctuation">,</span>%.1f<span class="token punctuation">,</span>%s<span class="token punctuation">,</span>%s&#92;n"<span class="token punctuation">,</span> $11<span class="token punctuation">,</span> $3<span class="token punctuation">,</span> $4<span class="token punctuation">,</span> $2<span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">20</span></code>`),a(c);var i=n(c,4),v=s(i);t(v,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'service_status'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Service Status'</span>
  <span class="token key atrule">table_config</span><span class="token punctuation">:</span>
    <span class="token key atrule">headers</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'Service'</span><span class="token punctuation">,</span> <span class="token string">'Status'</span><span class="token punctuation">,</span> <span class="token string">'Port'</span><span class="token punctuation">,</span> <span class="token string">'Uptime'</span><span class="token punctuation">]</span>
    <span class="token key atrule">sortable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">show_row_numbers</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
    <span class="token key atrule">border_style</span><span class="token punctuation">:</span> <span class="token string">'double'</span>
  <span class="token key atrule">table_data</span><span class="token punctuation">:</span> <span class="token punctuation">|</span><span class="token scalar string">
    [
      &#123;"Service": "nginx", "Status": "running", "Port": "80", "Uptime": "5d 3h"&#125;,
      &#123;"Service": "mysql", "Status": "running", "Port": "3306", "Uptime": "2d 1h"&#125;,
      &#123;"Service": "redis", "Status": "stopped", "Port": "6379", "Uptime": "-"&#125;
    ]</span></code>`),a(i);var u=n(i,36),w=s(u);t(w,()=>`<code class="language-yaml"><span class="token key atrule">app</span><span class="token punctuation">:</span>
  <span class="token key atrule">layouts</span><span class="token punctuation">:</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'monitoring'</span>
      <span class="token key atrule">root</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'System Monitoring Dashboard'</span>
      <span class="token key atrule">children</span><span class="token punctuation">:</span>
        <span class="token comment"># CPU usage chart</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'cpu_chart'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'CPU Usage'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 48%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 45%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">chart_config</span><span class="token punctuation">:</span>
            <span class="token key atrule">chart_type</span><span class="token punctuation">:</span> <span class="token string">'line'</span>
            <span class="token key atrule">width</span><span class="token punctuation">:</span> <span class="token number">35</span>
            <span class="token key atrule">height</span><span class="token punctuation">:</span> <span class="token number">12</span>
            <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'CPU %'</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> top <span class="token punctuation">-</span>l 1 <span class="token punctuation">|</span> grep "CPU usage" <span class="token punctuation">|</span> awk '<span class="token punctuation">&#123;</span>print NR"<span class="token punctuation">,</span>"$3<span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> sed 's/%//'
            
        <span class="token comment"># Memory usage bar chart</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'memory_chart'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Memory Breakdown'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 52%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 45%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">chart_config</span><span class="token punctuation">:</span>
            <span class="token key atrule">chart_type</span><span class="token punctuation">:</span> <span class="token string">'bar'</span>
            <span class="token key atrule">width</span><span class="token punctuation">:</span> <span class="token number">35</span>
            <span class="token key atrule">height</span><span class="token punctuation">:</span> <span class="token number">12</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">5000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
              free | awk 'NR==2&#123;printf "Used,%.1f&#92;nFree,%.1f&#92;nCached,%.1f&#92;n", 
                                    $3/$2*100, $4/$2*100, $6/$2*100&#125;'</span>
        
        <span class="token comment"># Process table</span>
        <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'process_table'</span>
          <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Top Processes'</span>
          <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">&#125;</span>
          <span class="token key atrule">table_config</span><span class="token punctuation">:</span>
            <span class="token key atrule">headers</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'Command'</span><span class="token punctuation">,</span> <span class="token string">'CPU %'</span><span class="token punctuation">,</span> <span class="token string">'Memory %'</span><span class="token punctuation">,</span> <span class="token string">'PID'</span><span class="token punctuation">]</span>
            <span class="token key atrule">sortable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">filterable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">page_size</span><span class="token punctuation">:</span> <span class="token number">12</span>
            <span class="token key atrule">zebra_striping</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">show_row_numbers</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
            <span class="token key atrule">border_style</span><span class="token punctuation">:</span> <span class="token string">'rounded'</span>
          <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">3000</span>
          <span class="token key atrule">script</span><span class="token punctuation">:</span>
            <span class="token punctuation">-</span> ps aux <span class="token punctuation">-</span><span class="token punctuation">-</span>no<span class="token punctuation">-</span>headers <span class="token punctuation">|</span> awk '<span class="token punctuation">&#123;</span>printf "%s<span class="token punctuation">,</span>%.1f<span class="token punctuation">,</span>%.1f<span class="token punctuation">,</span>%s&#92;n"<span class="token punctuation">,</span> $11<span class="token punctuation">,</span> $3<span class="token punctuation">,</span> $4<span class="token punctuation">,</span> $2<span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> 
              sort <span class="token punctuation">-</span>rn <span class="token punctuation">-</span>k2 <span class="token punctuation">-</span>t<span class="token punctuation">,</span> <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">25</span></code>`),a(u);var r=n(u,4),C=s(r);t(C,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'network_dashboard'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Network Monitoring'</span>
  <span class="token key atrule">children</span><span class="token punctuation">:</span>
    <span class="token comment"># Network traffic chart</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'traffic_chart'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Network Traffic'</span>
      <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 60%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">&#125;</span>
      <span class="token key atrule">chart_config</span><span class="token punctuation">:</span>
        <span class="token key atrule">chart_type</span><span class="token punctuation">:</span> <span class="token string">'line'</span>
        <span class="token key atrule">width</span><span class="token punctuation">:</span> <span class="token number">45</span>
        <span class="token key atrule">height</span><span class="token punctuation">:</span> <span class="token number">15</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Network I/O (MB/s)'</span>
      <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">1000</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
          # Network traffic monitoring script
          interface=$(route get default | awk '/interface:/ &#123;print $2&#125;')
          netstat -ibn | grep $interface | awk '&#123;print NR","$7/1024/1024&#125;'</span>
    
    <span class="token comment"># Connection table</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'connections'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Active Connections'</span>
      <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 65%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">&#125;</span>
      <span class="token key atrule">table_config</span><span class="token punctuation">:</span>
        <span class="token key atrule">headers</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'Protocol'</span><span class="token punctuation">,</span> <span class="token string">'Local'</span><span class="token punctuation">,</span> <span class="token string">'Remote'</span><span class="token punctuation">,</span> <span class="token string">'State'</span><span class="token punctuation">]</span>
        <span class="token key atrule">sortable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        <span class="token key atrule">filterable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        <span class="token key atrule">page_size</span><span class="token punctuation">:</span> <span class="token number">20</span>
        <span class="token key atrule">border_style</span><span class="token punctuation">:</span> <span class="token string">'double'</span>
      <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">2000</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> netstat <span class="token punctuation">-</span>an <span class="token punctuation">|</span> grep <span class="token punctuation">-</span>E '^(tcp<span class="token punctuation">|</span>udp)' <span class="token punctuation">|</span> 
          awk '<span class="token punctuation">&#123;</span>printf "%s<span class="token punctuation">,</span>%s<span class="token punctuation">,</span>%s<span class="token punctuation">,</span>%s&#92;n"<span class="token punctuation">,</span> $1<span class="token punctuation">,</span> $4<span class="token punctuation">,</span> $5<span class="token punctuation">,</span> $6<span class="token punctuation">&#125;</span>' <span class="token punctuation">|</span> head <span class="token punctuation">-</span><span class="token number">30</span></code>`),a(r);var d=n(r,4),x=s(d);t(x,()=>`<code class="language-yaml"><span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'db_analytics'</span>
  <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Database Analytics'</span>
  <span class="token key atrule">children</span><span class="token punctuation">:</span>
    <span class="token comment"># Query performance histogram</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'query_perf'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Query Performance'</span>
      <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 5%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 50%<span class="token punctuation">&#125;</span>
      <span class="token key atrule">chart_config</span><span class="token punctuation">:</span>
        <span class="token key atrule">chart_type</span><span class="token punctuation">:</span> <span class="token string">'histogram'</span>
        <span class="token key atrule">width</span><span class="token punctuation">:</span> <span class="token number">35</span>
        <span class="token key atrule">height</span><span class="token punctuation">:</span> <span class="token number">15</span>
        <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Query Duration Distribution'</span>
      <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">10000</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
          # PostgreSQL query duration analysis
          psql -t -c "
            SELECT 
              CASE 
                WHEN total_time &lt; 100 THEN '&lt;100ms'
                WHEN total_time &lt; 1000 THEN '100ms-1s'
                WHEN total_time &lt; 5000 THEN '1s-5s'
                ELSE '>5s'
              END as duration_range,
              COUNT(*)
            FROM pg_stat_statements 
            GROUP BY 1
            ORDER BY 1" | 
          sed 's/|/,/g'</span>
    
    <span class="token comment"># Active queries table</span>
    <span class="token punctuation">-</span> <span class="token key atrule">id</span><span class="token punctuation">:</span> <span class="token string">'active_queries'</span>
      <span class="token key atrule">title</span><span class="token punctuation">:</span> <span class="token string">'Active Queries'</span>
      <span class="token key atrule">position</span><span class="token punctuation">:</span> <span class="token punctuation">&#123;</span><span class="token key atrule">x1</span><span class="token punctuation">:</span> 55%<span class="token punctuation">,</span> <span class="token key atrule">y1</span><span class="token punctuation">:</span> 10%<span class="token punctuation">,</span> <span class="token key atrule">x2</span><span class="token punctuation">:</span> 95%<span class="token punctuation">,</span> <span class="token key atrule">y2</span><span class="token punctuation">:</span> 90%<span class="token punctuation">&#125;</span>
      <span class="token key atrule">table_config</span><span class="token punctuation">:</span>
        <span class="token key atrule">headers</span><span class="token punctuation">:</span> <span class="token punctuation">[</span><span class="token string">'PID'</span><span class="token punctuation">,</span> <span class="token string">'Duration'</span><span class="token punctuation">,</span> <span class="token string">'State'</span><span class="token punctuation">,</span> <span class="token string">'Query'</span><span class="token punctuation">]</span>
        <span class="token key atrule">sortable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        <span class="token key atrule">filterable</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
        <span class="token key atrule">page_size</span><span class="token punctuation">:</span> <span class="token number">15</span>
        <span class="token key atrule">zebra_striping</span><span class="token punctuation">:</span> <span class="token boolean important">true</span>
      <span class="token key atrule">refresh_interval</span><span class="token punctuation">:</span> <span class="token number">5000</span>
      <span class="token key atrule">script</span><span class="token punctuation">:</span>
        <span class="token punctuation">-</span> <span class="token punctuation">|</span><span class="token scalar string">
          psql -t -c "
            SELECT 
              pid,
              EXTRACT(EPOCH FROM (NOW() - query_start))::int as duration_sec,
              state,
              LEFT(query, 50) as query_preview
            FROM pg_stat_activity 
            WHERE state != 'idle' AND pid != pg_backend_pid()
            ORDER BY query_start" |
          sed 's/[[:space:]]*|[[:space:]]*/,/g'</span></code>`),a(d),T(22),D(y,k)}const B=Object.freeze(Object.defineProperty({__proto__:null,default:E,metadata:g},Symbol.toStringTag,{value:"Module"}));export{B as _};
