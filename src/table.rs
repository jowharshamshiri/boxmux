use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Table data structure for enhanced data visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub metadata: HashMap<String, String>,
}

/// Table configuration for rendering and behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    pub title: Option<String>,
    pub width: usize,
    pub height: usize,
    pub show_headers: bool,
    pub sort_column: Option<String>,
    pub sort_ascending: bool,
    pub filters: HashMap<String, String>,
    pub column_widths: Option<Vec<usize>>,
    pub border_style: TableBorderStyle,
    pub zebra_striping: bool,
    pub highlight_row: Option<usize>,
    pub max_column_width: Option<usize>,
    pub show_row_numbers: bool,
    pub pagination: Option<TablePagination>,
}

/// Border styles for tables
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TableBorderStyle {
    None,
    Single,
    Double,
    Rounded,
    Thick,
    Custom {
        horizontal: char,
        vertical: char,
        top_left: char,
        top_right: char,
        bottom_left: char,
        bottom_right: char,
        cross: char,
    },
}

/// Pagination configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePagination {
    pub page_size: usize,
    pub current_page: usize,
    pub show_page_info: bool,
}

/// Sort direction for columns
#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Data type for column sorting
#[derive(Debug, Clone)]
pub enum ColumnType {
    Text,
    Number,
    Date,
    Boolean,
}

impl Default for TableConfig {
    fn default() -> Self {
        Self {
            title: None,
            width: 80,
            height: 20,
            show_headers: true,
            sort_column: None,
            sort_ascending: true,
            filters: HashMap::new(),
            column_widths: None,
            border_style: TableBorderStyle::Single,
            zebra_striping: false,
            highlight_row: None,
            max_column_width: Some(30),
            show_row_numbers: false,
            pagination: None,
        }
    }
}

/// Generate table display string from data and configuration
pub fn render_table(data: &TableData, config: &TableConfig) -> String {
    if data.headers.is_empty() || data.rows.is_empty() {
        return "No data".to_string();
    }

    // Apply filters first
    let filtered_rows = apply_filters(data, &config.filters);

    // Apply sorting
    let sorted_rows = apply_sorting(
        &filtered_rows,
        &data.headers,
        &config.sort_column,
        config.sort_ascending,
    );

    // Apply pagination
    let (paginated_rows, page_info) = apply_pagination(&sorted_rows, &config.pagination);

    // Calculate column widths
    let column_widths = calculate_column_widths(data, config, &paginated_rows);

    let mut result = String::new();

    // Add title if present
    if let Some(title) = &config.title {
        result.push_str(&format!("{}\n", title));
    }

    // Render table structure
    result.push_str(&render_top_border(&column_widths, &config.border_style));

    // Render headers
    if config.show_headers {
        result.push_str(&render_header_row(
            &data.headers,
            &column_widths,
            &config.border_style,
            config.show_row_numbers,
        ));
        result.push_str(&render_separator(&column_widths, &config.border_style));
    }

    // Render data rows
    for (index, row) in paginated_rows.iter().enumerate() {
        let is_highlighted = config.highlight_row == Some(index);
        let is_zebra = config.zebra_striping && index % 2 == 1;
        result.push_str(&render_data_row(
            row,
            &column_widths,
            &config.border_style,
            config.show_row_numbers,
            index,
            is_highlighted,
            is_zebra,
        ));
    }

    // Render bottom border
    result.push_str(&render_bottom_border(&column_widths, &config.border_style));

    // Add pagination info
    if let Some(info) = page_info {
        result.push_str(&format!("\n{}", info));
    }

    result
}

/// Apply filters to table data
fn apply_filters(data: &TableData, filters: &HashMap<String, String>) -> Vec<Vec<String>> {
    if filters.is_empty() {
        return data.rows.clone();
    }

    let mut filtered = Vec::new();

    for row in &data.rows {
        let mut matches = true;

        for (column_name, filter_value) in filters {
            if let Some(column_index) = data.headers.iter().position(|h| h == column_name) {
                if column_index < row.len() {
                    let cell_value = &row[column_index];
                    if cell_value.to_lowercase() != filter_value.to_lowercase() {
                        matches = false;
                        break;
                    }
                }
            }
        }

        if matches {
            filtered.push(row.clone());
        }
    }

    filtered
}

/// Apply sorting to table data
fn apply_sorting(
    rows: &[Vec<String>],
    headers: &[String],
    sort_column: &Option<String>,
    ascending: bool,
) -> Vec<Vec<String>> {
    if let Some(column_name) = sort_column {
        if let Some(column_index) = headers.iter().position(|h| h == column_name) {
            let mut sorted = rows.to_vec();

            sorted.sort_by(|a, b| {
                let default_string = String::new();
                let a_val = a.get(column_index).unwrap_or(&default_string);
                let b_val = b.get(column_index).unwrap_or(&default_string);

                // Try to parse as numbers first
                if let (Ok(a_num), Ok(b_num)) = (a_val.parse::<f64>(), b_val.parse::<f64>()) {
                    if ascending {
                        a_num
                            .partial_cmp(&b_num)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    } else {
                        b_num
                            .partial_cmp(&a_num)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }
                } else {
                    // Fall back to string comparison
                    if ascending {
                        a_val.cmp(b_val)
                    } else {
                        b_val.cmp(a_val)
                    }
                }
            });

            return sorted;
        }
    }

    rows.to_vec()
}

/// Apply pagination to table data
fn apply_pagination(
    rows: &[Vec<String>],
    pagination: &Option<TablePagination>,
) -> (Vec<Vec<String>>, Option<String>) {
    if let Some(pag) = pagination {
        let total_rows = rows.len();
        let total_pages = total_rows.div_ceil(pag.page_size);
        let start_index = pag.current_page * pag.page_size;
        let end_index = std::cmp::min(start_index + pag.page_size, total_rows);

        let paginated = if start_index < total_rows {
            rows[start_index..end_index].to_vec()
        } else {
            Vec::new()
        };

        let page_info = if pag.show_page_info {
            Some(format!(
                "Page {} of {} ({} total rows)",
                pag.current_page + 1,
                total_pages,
                total_rows
            ))
        } else {
            None
        };

        (paginated, page_info)
    } else {
        (rows.to_vec(), None)
    }
}

/// Calculate optimal column widths
fn calculate_column_widths(
    data: &TableData,
    config: &TableConfig,
    rows: &[Vec<String>],
) -> Vec<usize> {
    if let Some(widths) = &config.column_widths {
        return widths.clone();
    }

    let mut widths = Vec::new();
    let max_width = config.max_column_width.unwrap_or(30);

    for (i, header) in data.headers.iter().enumerate() {
        let mut max_len = header.len();

        // Check all rows for this column
        for row in rows {
            if let Some(cell) = row.get(i) {
                max_len = max_len.max(cell.len());
            }
        }

        // Apply max width constraint
        max_len = max_len.min(max_width);

        // Minimum width of 3 for readability
        max_len = max_len.max(3);

        widths.push(max_len);
    }

    widths
}

/// Render top border of table
fn render_top_border(widths: &[usize], style: &TableBorderStyle) -> String {
    match style {
        TableBorderStyle::None => String::new(),
        TableBorderStyle::Single => {
            let mut border = String::from("┌");
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&"─".repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push('┬');
                }
            }
            border.push('┐');
            border.push('\n');
            border
        }
        TableBorderStyle::Double => {
            let mut border = String::from("╔");
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&"═".repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push('╦');
                }
            }
            border.push('╗');
            border.push('\n');
            border
        }
        TableBorderStyle::Rounded => {
            let mut border = String::from("╭");
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&"─".repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push('┬');
                }
            }
            border.push('╮');
            border.push('\n');
            border
        }
        TableBorderStyle::Thick => {
            let mut border = String::from("┏");
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&"━".repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push('┳');
                }
            }
            border.push('┓');
            border.push('\n');
            border
        }
        TableBorderStyle::Custom {
            horizontal,
            vertical: _,
            top_left,
            top_right,
            bottom_left: _,
            bottom_right: _,
            cross,
        } => {
            let mut border = String::from(*top_left);
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&horizontal.to_string().repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push(*cross);
                }
            }
            border.push(*top_right);
            border.push('\n');
            border
        }
    }
}

/// Render bottom border of table
fn render_bottom_border(widths: &[usize], style: &TableBorderStyle) -> String {
    match style {
        TableBorderStyle::None => String::new(),
        TableBorderStyle::Single => {
            let mut border = String::from("└");
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&"─".repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push('┴');
                }
            }
            border.push('┘');
            border.push('\n');
            border
        }
        TableBorderStyle::Double => {
            let mut border = String::from("╚");
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&"═".repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push('╩');
                }
            }
            border.push('╝');
            border.push('\n');
            border
        }
        TableBorderStyle::Rounded => {
            let mut border = String::from("╰");
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&"─".repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push('┴');
                }
            }
            border.push('╯');
            border.push('\n');
            border
        }
        TableBorderStyle::Thick => {
            let mut border = String::from("┗");
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&"━".repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push('┻');
                }
            }
            border.push('┛');
            border.push('\n');
            border
        }
        TableBorderStyle::Custom {
            horizontal,
            vertical: _,
            top_left: _,
            top_right: _,
            bottom_left,
            bottom_right,
            cross,
        } => {
            let mut border = String::from(*bottom_left);
            for (i, &width) in widths.iter().enumerate() {
                border.push_str(&horizontal.to_string().repeat(width + 2));
                if i < widths.len() - 1 {
                    border.push(*cross);
                }
            }
            border.push(*bottom_right);
            border.push('\n');
            border
        }
    }
}

/// Render separator line between header and data
fn render_separator(widths: &[usize], style: &TableBorderStyle) -> String {
    match style {
        TableBorderStyle::None => String::new(),
        TableBorderStyle::Single => {
            let mut sep = String::from("├");
            for (i, &width) in widths.iter().enumerate() {
                sep.push_str(&"─".repeat(width + 2));
                if i < widths.len() - 1 {
                    sep.push('┼');
                }
            }
            sep.push('┤');
            sep.push('\n');
            sep
        }
        TableBorderStyle::Double => {
            let mut sep = String::from("╠");
            for (i, &width) in widths.iter().enumerate() {
                sep.push_str(&"═".repeat(width + 2));
                if i < widths.len() - 1 {
                    sep.push('╬');
                }
            }
            sep.push('╣');
            sep.push('\n');
            sep
        }
        TableBorderStyle::Rounded | TableBorderStyle::Thick => {
            // Use single line separator for rounded and thick styles
            let mut sep = String::from("├");
            for (i, &width) in widths.iter().enumerate() {
                sep.push_str(&"─".repeat(width + 2));
                if i < widths.len() - 1 {
                    sep.push('┼');
                }
            }
            sep.push('┤');
            sep.push('\n');
            sep
        }
        TableBorderStyle::Custom {
            horizontal,
            vertical,
            top_left: _,
            top_right: _,
            bottom_left: _,
            bottom_right: _,
            cross,
        } => {
            let mut sep = String::from(*vertical);
            for (i, &width) in widths.iter().enumerate() {
                sep.push_str(&horizontal.to_string().repeat(width + 2));
                if i < widths.len() - 1 {
                    sep.push(*cross);
                }
            }
            sep.push(*vertical);
            sep.push('\n');
            sep
        }
    }
}

/// Render header row
fn render_header_row(
    headers: &[String],
    widths: &[usize],
    style: &TableBorderStyle,
    show_row_numbers: bool,
) -> String {
    let vertical_char = match style {
        TableBorderStyle::None => ' ',
        TableBorderStyle::Single | TableBorderStyle::Rounded | TableBorderStyle::Thick => '│',
        TableBorderStyle::Double => '║',
        TableBorderStyle::Custom { vertical, .. } => *vertical,
    };

    let mut row = String::new();

    if style != &TableBorderStyle::None {
        row.push(vertical_char);
    }

    // Add row number column header if enabled
    if show_row_numbers {
        row.push_str(" # ");
        if style != &TableBorderStyle::None {
            row.push(vertical_char);
        }
    }

    for (i, header) in headers.iter().enumerate() {
        let width = widths[i];
        let truncated = if header.len() > width {
            format!("{}…", &header[..width.saturating_sub(1)])
        } else {
            header.clone()
        };

        row.push(' ');
        row.push_str(&format!("{:width$}", truncated, width = width));
        row.push(' ');

        if i < headers.len() - 1 && style != &TableBorderStyle::None {
            row.push(vertical_char);
        }
    }

    if style != &TableBorderStyle::None {
        row.push(vertical_char);
    }

    row.push('\n');
    row
}

/// Render data row
fn render_data_row(
    row: &[String],
    widths: &[usize],
    style: &TableBorderStyle,
    show_row_numbers: bool,
    row_index: usize,
    _is_highlighted: bool,
    _is_zebra: bool,
) -> String {
    let vertical_char = match style {
        TableBorderStyle::None => ' ',
        TableBorderStyle::Single | TableBorderStyle::Rounded | TableBorderStyle::Thick => '│',
        TableBorderStyle::Double => '║',
        TableBorderStyle::Custom { vertical, .. } => *vertical,
    };

    let mut result = String::new();

    if style != &TableBorderStyle::None {
        result.push(vertical_char);
    }

    // Add row number if enabled
    if show_row_numbers {
        result.push_str(&format!(" {} ", row_index + 1));
        if style != &TableBorderStyle::None {
            result.push(vertical_char);
        }
    }

    for (i, cell) in row.iter().enumerate() {
        let width = widths.get(i).copied().unwrap_or(10);
        let truncated = if cell.len() > width {
            format!("{}…", &cell[..width.saturating_sub(1)])
        } else {
            cell.clone()
        };

        result.push(' ');
        result.push_str(&format!("{:width$}", truncated, width = width));
        result.push(' ');

        if i < row.len() - 1 && style != &TableBorderStyle::None {
            result.push(vertical_char);
        }
    }

    if style != &TableBorderStyle::None {
        result.push(vertical_char);
    }

    result.push('\n');
    result
}

/// Parse table data from CSV-like text content
pub fn parse_table_data(content: &str, delimiter: Option<char>) -> TableData {
    let delimiter = delimiter.unwrap_or(',');
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return TableData {
            headers: Vec::new(),
            rows: Vec::new(),
            metadata: HashMap::new(),
        };
    }

    // First line is headers
    let headers: Vec<String> = lines[0]
        .split(delimiter)
        .map(|s| s.trim().to_string())
        .collect();

    // Remaining lines are data rows
    let mut rows = Vec::new();
    for line in lines.iter().skip(1) {
        if line.trim().is_empty() {
            continue;
        }

        let row: Vec<String> = line
            .split(delimiter)
            .map(|s| s.trim().to_string())
            .collect();

        rows.push(row);
    }

    let mut metadata = HashMap::new();
    metadata.insert("source".to_string(), "parsed_csv".to_string());
    metadata.insert("delimiter".to_string(), delimiter.to_string());
    metadata.insert("rows_count".to_string(), rows.len().to_string());
    metadata.insert("columns_count".to_string(), headers.len().to_string());

    TableData {
        headers,
        rows,
        metadata,
    }
}

/// Parse table data from JSON content
pub fn parse_table_data_from_json(content: &str) -> Result<TableData, serde_json::Error> {
    // Try to parse as array of objects first
    if let Ok(objects) = serde_json::from_str::<Vec<serde_json::Value>>(content) {
        if objects.is_empty() {
            return Ok(TableData {
                headers: Vec::new(),
                rows: Vec::new(),
                metadata: HashMap::new(),
            });
        }

        // Extract headers from first object
        let mut headers = Vec::new();
        if let Some(serde_json::Value::Object(map)) = objects.first() {
            for key in map.keys() {
                headers.push(key.clone());
            }
        }

        // Extract rows
        let mut rows = Vec::new();
        for obj in &objects {
            if let serde_json::Value::Object(map) = obj {
                let mut row = Vec::new();
                for header in &headers {
                    let value = map
                        .get(header)
                        .map(|v| match v {
                            serde_json::Value::String(s) => s.clone(),
                            serde_json::Value::Number(n) => n.to_string(),
                            serde_json::Value::Bool(b) => b.to_string(),
                            serde_json::Value::Null => "".to_string(),
                            _ => format!("{}", v),
                        })
                        .unwrap_or_default();
                    row.push(value);
                }
                rows.push(row);
            }
        }

        let mut metadata = HashMap::new();
        metadata.insert("source".to_string(), "parsed_json".to_string());
        metadata.insert("format".to_string(), "array_of_objects".to_string());
        metadata.insert("rows_count".to_string(), rows.len().to_string());
        metadata.insert("columns_count".to_string(), headers.len().to_string());

        return Ok(TableData {
            headers,
            rows,
            metadata,
        });
    }

    // Try to parse as structured table data
    serde_json::from_str(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_csv_data() {
        let csv_content = "Name,Age,City\nJohn,25,New York\nJane,30,Los Angeles\nBob,35,Chicago";
        let table = parse_table_data(csv_content, None);

        assert_eq!(table.headers, vec!["Name", "Age", "City"]);
        assert_eq!(table.rows.len(), 3);
        assert_eq!(table.rows[0], vec!["John", "25", "New York"]);
        assert_eq!(table.rows[2], vec!["Bob", "35", "Chicago"]);
    }

    #[test]
    fn test_table_rendering() {
        let data = TableData {
            headers: vec!["ID".to_string(), "Name".to_string(), "Status".to_string()],
            rows: vec![
                vec!["1".to_string(), "Alice".to_string(), "Active".to_string()],
                vec!["2".to_string(), "Bob".to_string(), "Inactive".to_string()],
                vec!["3".to_string(), "Charlie".to_string(), "Active".to_string()],
            ],
            metadata: HashMap::new(),
        };

        let config = TableConfig::default();
        let result = render_table(&data, &config);

        assert!(result.contains("ID"));
        assert!(result.contains("Alice"));
        assert!(result.contains("┌"));
        assert!(result.contains("└"));
    }

    #[test]
    fn test_table_filtering() {
        let data = TableData {
            headers: vec!["Name".to_string(), "Status".to_string()],
            rows: vec![
                vec!["Alice".to_string(), "Active".to_string()],
                vec!["Bob".to_string(), "Inactive".to_string()],
                vec!["Charlie".to_string(), "Active".to_string()],
            ],
            metadata: HashMap::new(),
        };

        let mut filters = HashMap::new();
        filters.insert("Status".to_string(), "Active".to_string());

        let filtered = apply_filters(&data, &filters);
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().all(|row| row[1] == "Active"));
    }

    #[test]
    fn test_table_sorting() {
        let rows = vec![
            vec!["Charlie".to_string(), "30".to_string()],
            vec!["Alice".to_string(), "25".to_string()],
            vec!["Bob".to_string(), "35".to_string()],
        ];
        let headers = vec!["Name".to_string(), "Age".to_string()];

        // Sort by name
        let sorted = apply_sorting(&rows, &headers, &Some("Name".to_string()), true);
        assert_eq!(sorted[0][0], "Alice");
        assert_eq!(sorted[2][0], "Charlie");

        // Sort by age (numeric)
        let sorted = apply_sorting(&rows, &headers, &Some("Age".to_string()), true);
        assert_eq!(sorted[0][1], "25");
        assert_eq!(sorted[2][1], "35");
    }

    #[test]
    fn test_json_parsing() {
        let json_content = r#"[
            {"id": 1, "name": "Alice", "active": true},
            {"id": 2, "name": "Bob", "active": false}
        ]"#;

        let table = parse_table_data_from_json(json_content).unwrap();
        assert_eq!(table.rows.len(), 2);
        assert!(table.headers.contains(&"id".to_string()));
        assert!(table.headers.contains(&"name".to_string()));
    }

    #[test]
    fn test_pagination() {
        let rows = vec![
            vec!["1".to_string()],
            vec!["2".to_string()],
            vec!["3".to_string()],
            vec!["4".to_string()],
            vec!["5".to_string()],
        ];

        let pagination = TablePagination {
            page_size: 2,
            current_page: 1,
            show_page_info: true,
        };

        let (paginated, info) = apply_pagination(&rows, &Some(pagination));
        assert_eq!(paginated.len(), 2);
        assert_eq!(paginated[0][0], "3");
        assert_eq!(paginated[1][0], "4");
        assert!(info.unwrap().contains("Page 2 of 3"));
    }
}
