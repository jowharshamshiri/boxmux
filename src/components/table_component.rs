//! Table Component - Unified table rendering component
//!
//! This component provides comprehensive table rendering capabilities with borders,
//! pagination, sorting, filtering, and styling. Extracts table rendering logic
//! into a reusable component following the established component architecture.

use crate::model::common::Bounds;
use crate::table::{render_table, TableConfig, TableData, TablePagination};
use crossterm::style::Color;
use crate::components::ComponentDimensions;

/// Configuration for table component styling
#[derive(Debug, Clone, PartialEq)]
pub struct TableComponentConfig {
    /// Table border color
    pub border_color: Color,
    /// Header background color
    pub header_bg_color: Option<Color>,
    /// Header text color
    pub header_text_color: Color,
    /// Data row text color
    pub row_text_color: Color,
    /// Highlighted row background color
    pub highlight_bg_color: Option<Color>,
    /// Zebra striping background color
    pub zebra_bg_color: Option<Color>,
    /// Page info display format
    pub page_info_format: String,
    /// Show page navigation indicators
    pub show_navigation: bool,
    /// Padding inside table cells
    pub cell_padding: usize,
}

impl Default for TableComponentConfig {
    fn default() -> Self {
        Self {
            border_color: Color::White,
            header_bg_color: Some(Color::DarkBlue),
            header_text_color: Color::White,
            row_text_color: Color::White,
            highlight_bg_color: Some(Color::DarkYellow),
            zebra_bg_color: Some(Color::DarkGrey),
            page_info_format: "Page {current} of {total} ({count} rows)".to_string(),
            show_navigation: true,
            cell_padding: 1,
        }
    }
}

/// Table Component for unified table rendering
pub struct TableComponent {
    config: TableComponentConfig,
}

impl TableComponent {
    /// Create new table component with default configuration
    pub fn new() -> Self {
        Self {
            config: TableComponentConfig::default(),
        }
    }

    /// Create table component with custom configuration
    pub fn with_config(config: TableComponentConfig) -> Self {
        Self { config }
    }

    /// Create table component with custom styling
    pub fn with_colors(border_color: Color, header_color: Color, row_color: Color) -> Self {
        Self {
            config: TableComponentConfig {
                border_color,
                header_text_color: header_color,
                row_text_color: row_color,
                ..Default::default()
            },
        }
    }

    /// Render table within specified bounds using table data and configuration
    pub fn render(
        &self,
        table_data: &TableData,
        table_config: &TableConfig,
        bounds: &Bounds,
    ) -> Vec<String> {
        // Adjust table config to fit within bounds
        let mut adjusted_config = table_config.clone();
        adjusted_config.width = ComponentDimensions::new(*bounds).content_bounds().width(); // Account for borders
        adjusted_config.height = ComponentDimensions::new(*bounds).content_bounds().height(); // Account for borders

        // Generate base table content using existing table system
        let table_content = render_table(table_data, &adjusted_config);

        // Split content into lines and apply component styling
        let mut lines: Vec<String> = table_content.lines().map(|s| s.to_string()).collect();

        // Apply component-specific styling and formatting
        self.apply_component_styling(&mut lines, &adjusted_config);

        // Ensure lines fit within bounds
        self.fit_to_bounds(&mut lines, bounds);

        lines
    }

    /// Render table with pagination controls
    pub fn render_with_pagination(
        &self,
        table_data: &TableData,
        table_config: &TableConfig,
        bounds: &Bounds,
    ) -> Vec<String> {
        let mut lines = self.render(table_data, table_config, bounds);

        // Add pagination info if pagination is enabled
        if let Some(pagination) = &table_config.pagination {
            if pagination.show_page_info {
                let page_info = self.generate_page_info(table_data, pagination);
                lines.push(page_info);

                // Add navigation indicators if enabled
                if self.config.show_navigation {
                    let nav_line = self.generate_navigation_line(pagination);
                    lines.push(nav_line);
                }
            }
        }

        lines
    }

    /// Apply component-specific styling to table lines
    fn apply_component_styling(&self, lines: &mut [String], config: &TableConfig) {
        // Apply zebra striping background colors
        if config.zebra_striping {
            self.apply_zebra_styling(lines);
        }

        // Apply row highlighting
        if let Some(highlight_row) = config.highlight_row {
            self.apply_row_highlighting(lines, highlight_row);
        }

        // Apply header styling
        if config.show_headers && !lines.is_empty() {
            self.apply_header_styling(lines);
        }
    }

    /// Apply zebra striping background colors
    fn apply_zebra_styling(&self, lines: &mut [String]) {
        // Skip border and header lines when applying zebra striping
        let data_start = self.find_data_row_start(lines);

        for (index, line) in lines.iter_mut().enumerate().skip(data_start) {
            if self.is_data_row(line) && (index - data_start) % 2 == 1 {
                // Apply zebra background to every other data row
                *line = format!("\x1b[48;5;8m{}\x1b[0m", line); // Dark grey background
            }
        }
    }

    /// Apply highlighting to specific row
    fn apply_row_highlighting(&self, lines: &mut [String], highlight_row: usize) {
        let data_start = self.find_data_row_start(lines);
        let target_index = data_start + highlight_row;

        if let Some(line) = lines.get_mut(target_index) {
            if self.is_data_row(line) {
                // Apply highlight background
                *line = format!("\x1b[48;5;11m{}\x1b[0m", line); // Bright yellow background
            }
        }
    }

    /// Apply header styling
    fn apply_header_styling(&self, lines: &mut [String]) {
        // Find header row (typically after top border)
        for (index, line) in lines.iter_mut().enumerate() {
            if self.is_header_row(line, index) {
                // Apply header background and text color
                *line = format!("\x1b[48;5;4m\x1b[97m{}\x1b[0m", line); // Blue background, white text
                break;
            }
        }
    }

    /// Find the starting index of data rows
    fn find_data_row_start(&self, lines: &[String]) -> usize {
        for (index, line) in lines.iter().enumerate() {
            if self.is_data_row(line) {
                return index;
            }
        }
        lines.len()
    }

    /// Check if line is a data row (not border or header)
    fn is_data_row(&self, line: &str) -> bool {
        !line.chars().all(|c| matches!(c, '┌' | '┐' | '└' | '┘' | '├' | '┤' | '┬' | '┴' | '┼' | '─' | '│' | '+' | '-' | '|' | ' '))
            && line.contains('│') // Contains vertical border character
            && !line.trim().is_empty()
    }

    /// Check if line is a header row
    fn is_header_row(&self, line: &str, index: usize) -> bool {
        // Header row is typically the first data-like row after borders
        index > 0 && self.is_data_row(line)
    }

    /// Generate pagination info string
    fn generate_page_info(&self, table_data: &TableData, pagination: &TablePagination) -> String {
        let total_rows = table_data.rows.len();
        let total_pages = total_rows.div_ceil(pagination.page_size);

        self.config
            .page_info_format
            .replace("{current}", &pagination.current_page.to_string())
            .replace("{total}", &total_pages.to_string())
            .replace("{count}", &total_rows.to_string())
    }

    /// Generate navigation line with indicators
    fn generate_navigation_line(&self, pagination: &TablePagination) -> String {
        let mut nav_line = String::new();

        // Previous page indicator
        if pagination.current_page > 1 {
            nav_line.push_str("◄ Prev  ");
        } else {
            nav_line.push_str("       ");
        }

        // Page numbers (show current ± 2 pages)
        let start_page = pagination.current_page.saturating_sub(2).max(1);
        let end_page = (pagination.current_page + 2).min(10); // Assume reasonable page limit

        for page in start_page..=end_page {
            if page == pagination.current_page {
                nav_line.push_str(&format!("[{}] ", page));
            } else {
                nav_line.push_str(&format!("{} ", page));
            }
        }

        // Next page indicator
        nav_line.push_str("  Next ►");

        nav_line
    }

    /// Fit lines to bounds by truncating or padding
    fn fit_to_bounds(&self, lines: &mut Vec<String>, bounds: &Bounds) {
        // Truncate lines that are too long
        for line in lines.iter_mut() {
            if line.chars().count() > bounds.width() {
                let truncated: String = line.chars().take(bounds.width()).collect();
                *line = truncated;
                if bounds.width() > 3 {
                    let ellipsis_pos = bounds.width().saturating_sub(3);
                    let mut chars: Vec<char> = line.chars().collect();
                    if chars.len() > ellipsis_pos {
                        chars.truncate(ellipsis_pos);
                        chars.extend("...".chars());
                        *line = chars.into_iter().collect();
                    }
                }
            }
        }

        // Limit number of lines to bounds height
        if lines.len() > bounds.height() {
            lines.truncate(bounds.height());
        }

        // Pad with empty lines if needed
        while lines.len() < bounds.height() {
            lines.push(" ".repeat(bounds.width()));
        }
    }

    /// Get current configuration
    pub fn get_config(&self) -> &TableComponentConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: TableComponentConfig) {
        self.config = config;
    }
}

impl Default for TableComponent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_table_data() -> TableData {
        TableData {
            headers: vec!["Name".to_string(), "Age".to_string(), "Status".to_string()],
            rows: vec![
                vec!["Alice".to_string(), "25".to_string(), "Active".to_string()],
                vec!["Bob".to_string(), "30".to_string(), "Inactive".to_string()],
                vec![
                    "Charlie".to_string(),
                    "35".to_string(),
                    "Active".to_string(),
                ],
            ],
            metadata: HashMap::new(),
        }
    }

    fn create_test_bounds() -> Bounds {
        Bounds::new(0, 0, 50, 10)
    }

    #[test]
    fn test_table_component_creation() {
        let component = TableComponent::new();
        assert_eq!(component.config.border_color, Color::White);
        assert_eq!(component.config.header_text_color, Color::White);
    }

    #[test]
    fn test_table_component_with_config() {
        let config = TableComponentConfig {
            border_color: Color::Red,
            header_text_color: Color::Yellow,
            row_text_color: Color::Green,
            ..Default::default()
        };

        let component = TableComponent::with_config(config.clone());
        assert_eq!(component.config.border_color, Color::Red);
        assert_eq!(component.config.header_text_color, Color::Yellow);
        assert_eq!(component.config.row_text_color, Color::Green);
    }

    #[test]
    fn test_table_component_with_colors() {
        let component = TableComponent::with_colors(Color::Blue, Color::Cyan, Color::Magenta);
        assert_eq!(component.config.border_color, Color::Blue);
        assert_eq!(component.config.header_text_color, Color::Cyan);
        assert_eq!(component.config.row_text_color, Color::Magenta);
    }

    #[test]
    fn test_basic_table_rendering() {
        let component = TableComponent::new();
        let table_data = create_test_table_data();
        let table_config = TableConfig::default();
        let bounds = create_test_bounds();

        let lines = component.render(&table_data, &table_config, &bounds);

        assert!(!lines.is_empty());
        assert!(lines.len() <= bounds.height());

        // Check that all lines fit within width bounds
        for line in &lines {
            assert!(line.chars().count() <= bounds.width());
        }
    }

    #[test]
    fn test_table_rendering_with_pagination() {
        let component = TableComponent::new();
        let table_data = create_test_table_data();
        let mut table_config = TableConfig::default();
        table_config.pagination = Some(TablePagination {
            page_size: 2,
            current_page: 1,
            show_page_info: true,
        });
        let bounds = create_test_bounds();

        let lines = component.render_with_pagination(&table_data, &table_config, &bounds);

        assert!(!lines.is_empty());

        // Should have pagination info
        let has_page_info = lines.iter().any(|line| line.contains("Page"));
        assert!(has_page_info);
    }

    #[test]
    fn test_zebra_striping_detection() {
        let component = TableComponent::new();

        // Test data row detection
        assert!(component.is_data_row("│ Alice   │ 25  │ Active   │"));
        assert!(!component.is_data_row("├─────────┼─────┼──────────┤"));
        assert!(!component.is_data_row("┌─────────┬─────┬──────────┐"));
    }

    #[test]
    fn test_page_info_generation() {
        let component = TableComponent::new();
        let table_data = create_test_table_data();
        let pagination = TablePagination {
            page_size: 2,
            current_page: 2,
            show_page_info: true,
        };

        let page_info = component.generate_page_info(&table_data, &pagination);

        assert!(page_info.contains("Page 2"));
        assert!(page_info.contains("3 rows"));
    }

    #[test]
    fn test_navigation_line_generation() {
        let component = TableComponent::new();
        let pagination = TablePagination {
            page_size: 2,
            current_page: 2,
            show_page_info: true,
        };

        let nav_line = component.generate_navigation_line(&pagination);

        assert!(nav_line.contains("◄ Prev"));
        assert!(nav_line.contains("[2]")); // Current page in brackets
        assert!(nav_line.contains("Next ►"));
    }

    #[test]
    fn test_bounds_fitting() {
        let component = TableComponent::new();
        let table_data = create_test_table_data();
        let table_config = TableConfig::default();
        let small_bounds = Bounds::new(0, 0, 20, 5);

        let lines = component.render(&table_data, &table_config, &small_bounds);

        assert_eq!(lines.len(), small_bounds.height());
        for line in &lines {
            assert!(line.chars().count() <= small_bounds.width());
        }
    }

    #[test]
    fn test_config_update() {
        let mut component = TableComponent::new();
        let original_color = component.config.border_color;

        let mut new_config = component.config.clone();
        new_config.border_color = Color::Red;

        component.update_config(new_config);

        assert_ne!(component.config.border_color, original_color);
        assert_eq!(component.config.border_color, Color::Red);
    }
}
