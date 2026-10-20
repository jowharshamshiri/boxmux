---
title: Data Visualization
description: Charts and tables in BoxMux boxes - chart types, data formats, table data in CSV or JSON, and every table_config setting.
---


## Table of Contents

- [Charts](#charts)
- [Tables](#tables)
- [Data from scripts](#data-from-scripts)
- [Rules and limits](#rules-and-limits)

A box can show a chart (`chart_type`), a table (`table_data` / `table_config`) or content drawn
by a plugin (`plugin_component`, see [Plugin System](/docs/plugin-system)). A box shows only one
of these. The chart or table is drawn as text inside the border, with a one-column margin on
the left and right, starting at the top-left corner. It uses the box's colors, and when it is
larger than the box it scrolls (or wraps, or is cut off) according to `overflow_behavior`.

## Charts

Set `chart_type` to `bar`, `line`, `histogram`, `pie` or `scatter`, and give the data in
`chart_data`:

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: resources
          title: Resource usage
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
          chart_type: bar
          chart_data: |
            CPU,67
            Memory,45
            Disk,89
            Network,23
```

### Data format

One data point per line, as a label and a number:

- `label,value` (when the line contains a comma),
- `label:value` (when it contains a colon and no comma),
- `label value` (otherwise; the label is the first word).

Empty lines and lines starting with `#` are skipped. Any other line must be a label and a
number; otherwise the box shows an error naming the line (`chart data line 3 is not
'label,value': ...`). Points keep their order.

### Chart types

| `chart_type` | What it draws |
|--------------|---------------|
| `bar` | One horizontal bar per point, with the label on the left and the value on the right. Bars are proportional to the largest value; negative values get no bar. Points beyond the height of the box are not shown. |
| `line` | The points in order, evenly spread from left to right, as `●` joined by a dotted line, with the largest and smallest values on the scale on the left and the labels underneath where they fit. |
| `scatter` | The points as `◆`. When every label is a number, a point's label is its position from left to right; otherwise the points are evenly spread in order, like a line chart without the line. |
| `pie` | A filled circle whose slices (`█`, `▓`, `▒`, `░`, `●`, `○`) are proportional to the values, clockwise from the top, with a legend giving each label's percentage. It shows the first six points; negative values get no slice, and when no value is positive the box shows `no positive values`. |
| `histogram` | How the values are distributed: the range from the smallest to the largest value is cut into equal parts (up to 12, and no more than there are points), and each part is a vertical bar as tall as the number of values in it. Under the bars are the counts and the start of each part. Labels are not shown. |

The chart fills the box and is redrawn at the new size when the box or the terminal is
resized. The chart has no title of its own; the box title serves as one.

```yaml
app:
  layouts:
    - id: charts
      root: true
      title: Chart types
      children:
        - id: trend
          title: Requests per hour (line)
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 50%}
          chart_type: line
          chart_data: |
            00h,120
            04h,80
            08h,340
            12h,510
            16h,470
            20h,260
        - id: share
          title: Disk share (pie)
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 50%}
          chart_type: pie
          chart_data: |
            System,30
            Home,55
            Free,15
        - id: latency
          title: Latency samples (scatter)
          position: {x1: 0%, y1: 50%, x2: 50%, y2: 100%}
          chart_type: scatter
          chart_data: |
            a,12
            b,30
            c,18
            d,25
        - id: sizes
          title: Sizes (histogram)
          position: {x1: 50%, y1: 50%, x2: 100%, y2: 100%}
          chart_type: histogram
          chart_data: |
            v1,5
            v2,12
            v3,8
            v4,15
```

## Tables

`table_data` holds the table as CSV or as a JSON array of objects. `table_config` sets how it
is drawn; it is optional.

### CSV

The first line is the header row; each further line is a row. Fields are separated by commas
and trimmed of surrounding spaces. There is no quoting: a comma always separates two fields.
Empty lines are skipped.

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: staff
          title: Staff
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          table_data: |
            Name,Department,Salary
            Alice,Engineering,75000
            Bob,Marketing,65000
            Carol,Engineering,78000
            David,Sales,72000
          table_config:
            sort_column: Salary
            sort_ascending: false
            border_style: double
```

### JSON

Data that starts with `[` is read as a JSON array of objects. The columns are the keys of the
first object, in the order they are written; keys that only later objects have are not shown,
and a key an object lacks is an empty cell. Strings,
numbers and booleans are shown as written, `null` as an empty cell, and nested arrays or objects
as JSON text. Data that is not a JSON array of objects shows an error message in the box.

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: products
          title: Electronics
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          table_data: |
            [
              {"product": "Laptop", "price": 1299, "category": "Electronics"},
              {"product": "Chair", "price": 299, "category": "Furniture"},
              {"product": "Keyboard", "price": 149, "category": "Electronics"},
              {"product": "Mouse", "price": 79, "category": "Electronics"}
            ]
          table_config:
            sort_column: price
            filters:
              category: electronics
            show_row_numbers: true
```

### `table_config`

All settings are optional. Unknown keys are errors.

| Key | Type | Default | Effect |
|-----|------|---------|--------|
| `show_headers` | boolean | `true` | Draw the header row. |
| `sort_column` | string | none | Header of the column the rows are sorted by. Values that are both numbers compare as numbers; otherwise they compare as text (case-sensitive). A name that matches no header leaves the rows unsorted. |
| `sort_ascending` | boolean | `true` | Sort order for `sort_column`. |
| `filters` | map of column header to text | none | Show only rows whose value in each named column equals the text, ignoring case. Every filter must match, so a filter naming a column the table does not have leaves no rows. |
| `border_style` | `none`, `single`, `double`, `rounded`, `thick` | `single` | Lines drawn around and inside the table. |
| `zebra_striping` | boolean | `false` | Shade every other row with a gray background. |
| `max_column_width` | number | `30` | Widest a column gets, in terminal columns (wide characters count as two); longer values are cut and end with `…`. Columns are at least 3 wide. |
| `show_row_numbers` | boolean | `false` | Add a `#` column numbering the rows from 1, after filtering and sorting; on later pages the numbers continue. |
| `page_size` | number | none | Rows per page. Without it the table is not paged. |
| `current_page` | number | `0` | The page shown, counting from 0. Needs `page_size`. |
| `show_page_info` | boolean | `false` | Add a line `Page N of M (R rows)` under the table, counting the rows left after filtering. Needs `page_size`. |

Filtering happens first, then sorting, then paging. Rows with fewer fields than the header
get empty cells; extra fields are dropped. Data with no header line shows `No data`; a header
with no rows draws just the header.

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: inventory
          title: Inventory, page 1
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          table_data: |
            ID,Item,Price
            1,Widget A,12.99
            2,Widget B,24.99
            3,Gadget X,45.00
            4,Tool Y,78.50
            5,Device Z,199.99
            6,Component 1,8.75
            7,Component 2,15.25
          table_config:
            sort_column: Price
            sort_ascending: false
            border_style: rounded
            max_column_width: 12
            page_size: 3
            current_page: 0
            show_page_info: true
```

Paging and sorting are set in the configuration; there are no keys or clicks that change them
while BoxMux runs. To show another page from a script, replace the box through the control
socket (`boxmux replace_box`) with a different `current_page`.

## Data from scripts

Without `chart_data` (or `table_data`), a chart or table uses the output of the box's script:
the text of the box's visible tab, with ANSI color codes removed. With a `refresh_interval`,
the chart or table is redrawn from each new output.

```yaml
app:
  layouts:
    - id: main
      root: true
      title: Live data
      children:
        - id: disk
          title: Disk use (%)
          position: {x1: 0%, y1: 0%, x2: 50%, y2: 100%}
          chart_type: bar
          refresh_interval: 5000
          script:
            - df -P | awk 'NR > 1 { sub("%", "", $5); print $6 "," $5 }'

        - id: files
          title: Largest files here
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
          refresh_interval: 10000
          table_config:
            sort_column: Bytes
            sort_ascending: false
            page_size: 10
          script:
            - echo "Name,Bytes"
            - ls -l | awk 'NR > 1 { print $9 "," $5 }'
```

With `append_output: true`, each run's output is added to the previous output, so a line chart
can grow over time. Nothing is ever removed from it, so use this for bounded sessions.

```yaml
app:
  layouts:
    - id: main
      root: true
      children:
        - id: samples
          title: Random samples
          position: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}
          chart_type: line
          refresh_interval: 2000
          append_output: true
          script:
            - echo "$(date +%S),$((RANDOM % 100))"
```

The scripts above use `bash`, the default shell on Unix. On Windows scripts run in PowerShell
unless `app.shell` says otherwise.

## Rules and limits

These are checked when the configuration is loaded:

- `chart_data` needs a `chart_type`.
- A chart needs `chart_data` or a `script`.
- A box with `table_config` needs `table_data` or a `script`.
- A box has at most one of `plugin_component`, `chart_type` and `table_data`/`table_config`.

While drawing:

- A chart needs an area of at least 20 columns by 5 rows. The area is the inside of the
  border less a one-column margin on each side, so the box must be at least 24 columns wide
  and 7 rows tall; a smaller box shows `too small`.
- A chart without data points shows `no chart data`.
