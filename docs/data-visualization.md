---
layout: default
title: Data Visualization - BoxMux
---

# Data Visualization

BoxMux provides data visualization through charts and tables with features for displaying, sorting, and filtering structured data, including real-time updates.

## Table of Contents

- [Chart System](#chart-system)
- [Table System](#table-system)
- [Performance Features](#performance-features)
- [Real-World Examples](#real-world-examples)
- [Best Practices](#best-practices)

## Chart System

BoxMux includes a Unicode-based charting system with responsive layout and multiple chart types.

### Chart Types

#### Bar Charts

Perfect for categorical data comparison:

```yaml
- id: 'resource_usage'
  title: 'Resource Usage'
  chart_config:
    chart_type: 'bar'
    width: 50
    height: 15
    title: 'System Resources'
    x_label: 'Resource'
    y_label: 'Usage %'
  chart_data: |
    CPU,67
    Memory,45
    Disk,89
    Network,23
```

#### Line Charts

Ideal for time-series data and trends:

```yaml
- id: 'cpu_trend'
  title: 'CPU Usage Over Time'
  chart_config:
    chart_type: 'line'
    width: 60
    height: 12
    title: 'CPU Trend'
  refresh_interval: 2000
  script:
    - top -l 1 | grep "CPU usage" | awk '{print NR","$3}' | sed 's/%//'
```

#### Histograms

Great for distribution visualization:

```yaml
- id: 'response_times'
  title: 'Response Time Distribution'
  chart_config:
    chart_type: 'histogram'
    width: 45
    height: 10
    title: 'API Response Times'
  chart_data: |
    0-100ms,45
    100-200ms,67
    200-500ms,23
    500ms+,5
```

### Chart Layout System

BoxMux includes a smart chart layout engine with responsive design:

- **Automatic sizing**: Charts adapt to box dimensions
- **Smart alignment**: Proper axis alignment and labeling
- **Responsive scaling**: Charts scale with terminal size changes
- **Unicode rendering**: High-quality Unicode-based chart display

### Chart Configuration Options

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `chart_type` | string | - | Chart type: 'bar', 'line', 'histogram' |
| `width` | number | 40 | Chart width in characters |
| `height` | number | 10 | Chart height in lines |
| `title` | string | - | Chart title |
| `x_label` | string | - | X-axis label |
| `y_label` | string | - | Y-axis label |

### Dynamic Chart Data

Charts can display live data from scripts:

```yaml
- id: 'memory_chart'
  title: 'Live Memory Usage'
  chart_config:
    chart_type: 'line'
    width: 50
    height: 15
  refresh_interval: 1000
  script:
    - |
      # Generate timestamp and memory usage
      timestamp=$(date +%s)
      memory=$(free | awk 'NR==2{printf "%.1f", $3/$2 * 100.0}')
      echo "${timestamp},${memory}"
```

## Table System

BoxMux provides a table system for displaying structured data.

### Table Features

- **Data Parsing**: Support for CSV and JSON data formats
- **Sorting**: Text and numeric sorting with ascending/descending support
- **Filtering**: Exact match and case-insensitive search
- **Pagination**: Configurable page sizes with navigation info
- **Visual Enhancement**: Zebra striping, row numbers, multiple border styles
- **Column Management**: Width calculation with max width constraints

### Table Configuration

| Property | Type | Default | Description |
|----------|------|---------|-------------|
| `headers` | array[string] | - | Column headers |
| `sortable` | boolean | false | Enable column sorting |
| `filterable` | boolean | false | Enable row filtering |
| `page_size` | number | 10 | Rows per page |
| `show_row_numbers` | boolean | false | Display row numbers |
| `zebra_striping` | boolean | false | Alternating row colors |
| `border_style` | string | 'single' | Border style |

### Border Styles

BoxMux supports multiple table border styles:

- `none`: No borders
- `single`: Single-line borders (default)
- `double`: Double-line borders
- `rounded`: Rounded corner borders
- `thick`: Thick-line borders
- `custom`: Custom border characters

### Data Formats

#### CSV Format

```yaml
- id: 'process_table'
  title: 'System Processes'
  table_config:
    headers: ['Process', 'CPU %', 'Memory', 'PID']
    sortable: true
    filterable: true
    page_size: 15
    zebra_striping: true
  refresh_interval: 5000
  script:
    - ps aux --no-headers | awk '{printf "%s,%.1f,%s,%s\n", $11, $3, $4, $2}' | head -20
```

#### JSON Format

```yaml
- id: 'service_status'
  title: 'Service Status'
  table_config:
    headers: ['Service', 'Status', 'Port', 'Uptime']
    sortable: true
    show_row_numbers: true
    border_style: 'double'
  table_data: |
    [
      {"Service": "nginx", "Status": "running", "Port": "80", "Uptime": "5d 3h"},
      {"Service": "mysql", "Status": "running", "Port": "3306", "Uptime": "2d 1h"},
      {"Service": "redis", "Status": "stopped", "Port": "6379", "Uptime": "-"}
    ]
```

### Table Operations

#### Sorting

Tables support intelligent sorting:

- **Numeric sorting**: Automatically detects and sorts numeric values
- **Text sorting**: Case-insensitive text sorting
- **Ascending/Descending**: Toggle sort direction
- **Multi-column**: Sort by multiple columns (planned feature)

#### Filtering

Tables include flexible filtering:

- **Exact match**: Find exact value matches
- **Case-insensitive**: Search ignoring case
- **Column-specific**: Filter specific columns
- **Real-time**: Filter results update immediately

#### Pagination

Table pagination features:

- **Configurable page size**: Set rows per page
- **Navigation info**: Current page and total pages
- **Keyboard navigation**: Page Up/Down support
- **Auto-sizing**: Page size adapts to box height

## Performance Features

### Efficient Rendering

- **Unicode optimization**: Fast Unicode character rendering
- **Memory efficiency**: Minimal memory usage for large datasets
- **Incremental updates**: Only redraw changed content
- **Viewport culling**: Only render visible content

### Data Processing

- **Stream processing**: Handle large datasets efficiently
- **Lazy loading**: Load data on demand
- **Caching**: Cache parsed data for faster access
- **Incremental parsing**: Parse data incrementally

## Real-World Examples

### System Monitor Dashboard

```yaml
app:
  layouts:
    - id: 'monitoring'
      root: true
      title: 'System Monitoring Dashboard'
      children:
        # CPU usage chart
        - id: 'cpu_chart'
          title: 'CPU Usage'
          position: {x1: 5%, y1: 10%, x2: 48%, y2: 45%}
          chart_config:
            chart_type: 'line'
            width: 35
            height: 12
            title: 'CPU %'
          refresh_interval: 2000
          script:
            - top -l 1 | grep "CPU usage" | awk '{print NR","$3}' | sed 's/%//'
            
        # Memory usage bar chart
        - id: 'memory_chart'
          title: 'Memory Breakdown'
          position: {x1: 52%, y1: 10%, x2: 95%, y2: 45%}
          chart_config:
            chart_type: 'bar'
            width: 35
            height: 12
          refresh_interval: 5000
          script:
            - |
              free | awk 'NR==2{printf "Used,%.1f\nFree,%.1f\nCached,%.1f\n", 
                                    $3/$2*100, $4/$2*100, $6/$2*100}'
        
        # Process table
        - id: 'process_table'
          title: 'Top Processes'
          position: {x1: 5%, y1: 50%, x2: 95%, y2: 90%}
          table_config:
            headers: ['Command', 'CPU %', 'Memory %', 'PID']
            sortable: true
            filterable: true
            page_size: 12
            zebra_striping: true
            show_row_numbers: true
            border_style: 'rounded'
          refresh_interval: 3000
          script:
            - ps aux --no-headers | awk '{printf "%s,%.1f,%.1f,%s\n", $11, $3, $4, $2}' | 
              sort -rn -k2 -t, | head -25
```

### Network Monitoring

```yaml
- id: 'network_dashboard'
  title: 'Network Monitoring'
  children:
    # Network traffic chart
    - id: 'traffic_chart'
      title: 'Network Traffic'
      position: {x1: 5%, y1: 10%, x2: 60%, y2: 50%}
      chart_config:
        chart_type: 'line'
        width: 45
        height: 15
        title: 'Network I/O (MB/s)'
      refresh_interval: 1000
      script:
        - |
          # Network traffic monitoring script
          interface=$(route get default | awk '/interface:/ {print $2}')
          netstat -ibn | grep $interface | awk '{print NR","$7/1024/1024}'
    
    # Connection table
    - id: 'connections'
      title: 'Active Connections'
      position: {x1: 65%, y1: 10%, x2: 95%, y2: 90%}
      table_config:
        headers: ['Protocol', 'Local', 'Remote', 'State']
        sortable: true
        filterable: true
        page_size: 20
        border_style: 'double'
      refresh_interval: 2000
      script:
        - netstat -an | grep -E '^(tcp|udp)' | 
          awk '{printf "%s,%s,%s,%s\n", $1, $4, $5, $6}' | head -30
```

### Database Analytics

```yaml
- id: 'db_analytics'
  title: 'Database Analytics'
  children:
    # Query performance histogram
    - id: 'query_perf'
      title: 'Query Performance'
      position: {x1: 5%, y1: 10%, x2: 50%, y2: 50%}
      chart_config:
        chart_type: 'histogram'
        width: 35
        height: 15
        title: 'Query Duration Distribution'
      refresh_interval: 10000
      script:
        - |
          # PostgreSQL query duration analysis
          psql -t -c "
            SELECT 
              CASE 
                WHEN total_time < 100 THEN '<100ms'
                WHEN total_time < 1000 THEN '100ms-1s'
                WHEN total_time < 5000 THEN '1s-5s'
                ELSE '>5s'
              END as duration_range,
              COUNT(*)
            FROM pg_stat_statements 
            GROUP BY 1
            ORDER BY 1" | 
          sed 's/|/,/g'
    
    # Active queries table
    - id: 'active_queries'
      title: 'Active Queries'
      position: {x1: 55%, y1: 10%, x2: 95%, y2: 90%}
      table_config:
        headers: ['PID', 'Duration', 'State', 'Query']
        sortable: true
        filterable: true
        page_size: 15
        zebra_striping: true
      refresh_interval: 5000
      script:
        - |
          psql -t -c "
            SELECT 
              pid,
              EXTRACT(EPOCH FROM (NOW() - query_start))::int as duration_sec,
              state,
              LEFT(query, 50) as query_preview
            FROM pg_stat_activity 
            WHERE state != 'idle' AND pid != pg_backend_pid()
            ORDER BY query_start" |
          sed 's/[[:space:]]*|[[:space:]]*/,/g'
```

## Best Practices

### Chart Design

1. **Choose appropriate chart types**: Bar charts for comparisons, line charts for trends, histograms for distributions
2. **Size appropriately**: Balance chart detail with box space
3. **Use meaningful titles**: Clear chart and axis titles
4. **Consider refresh rates**: Balance real-time updates with performance
5. **Test with real data**: Verify charts with actual data ranges

### Table Design

1. **Limit column count**: Keep tables readable with appropriate column counts
2. **Use appropriate page sizes**: Balance data visibility with performance
3. **Enable sorting for numeric data**: Make numeric columns sortable
4. **Consider zebra striping**: Improve readability for wide tables
5. **Use filtering for large datasets**: Help users find relevant data quickly

### Performance Optimization

1. **Optimize data scripts**: Use efficient commands to generate data
2. **Cache expensive operations**: Cache slow database queries
3. **Limit data volume**: Use appropriate limits in data generation scripts
4. **Use appropriate refresh intervals**: Balance freshness with system load
5. **Monitor memory usage**: Watch memory consumption with large datasets

### Data Quality

1. **Validate data formats**: Ensure scripts produce consistent formats
2. **Handle missing data**: Gracefully handle missing or invalid data
3. **Error handling**: Include error handling in data generation scripts
4. **Test edge cases**: Verify behavior with empty or malformed data
5. **Document data sources**: Clear documentation for data generation logic

---

For configuration details, see [Configuration Reference](configuration.md).  
For implementation examples, see [User Guide](user-guide.md).