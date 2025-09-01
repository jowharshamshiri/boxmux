//! Chart Component System - Unified chart rendering component with multiple chart types
//! 
//! This module provides a comprehensive chart rendering component that encapsulates
//! all chart generation logic while providing a clean, reusable interface.

use crate::model::common::{Bounds, ScreenBuffer};
use crate::draw_utils::print_with_color_and_background_at;

/// Chart data point
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
}

/// Supported chart types
#[derive(Debug, Clone, PartialEq)]
pub enum ChartType {
    Bar,
    Line,
    Histogram,
    Pie,
    Scatter,
}

impl std::str::FromStr for ChartType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "bar" => Ok(ChartType::Bar),
            "line" => Ok(ChartType::Line),
            "histogram" => Ok(ChartType::Histogram),
            "pie" => Ok(ChartType::Pie),
            "scatter" => Ok(ChartType::Scatter),
            _ => Err(format!("Unknown chart type: {}", s)),
        }
    }
}

/// Chart configuration
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub chart_type: ChartType,
    pub title: Option<String>,
    pub width: usize,
    pub height: usize,
    pub color: String,
    pub show_title: bool,
    pub show_values: bool,
    pub show_grid: bool,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            chart_type: ChartType::Bar,
            title: None,
            width: 40,
            height: 10,
            color: "blue".to_string(),
            show_title: true,
            show_values: true,
            show_grid: false,
        }
    }
}

/// Chart layout dimensions and positioning
#[derive(Debug, Clone)]
struct ChartLayout {
    /// Total available width
    pub total_width: usize,
    /// Total available height
    pub _total_height: usize,
    /// Width for chart content (excluding labels)
    pub chart_width: usize,
    /// Height for chart content (excluding title/axes)
    pub chart_height: usize,
    /// Width reserved for Y-axis labels
    pub y_label_width: usize,
    /// Height reserved for title
    pub title_height: usize,
    /// Height reserved for X-axis labels
    pub x_label_height: usize,
}

/// Chart rendering component with comprehensive chart type support
#[derive(Debug, Clone)]
pub struct ChartComponent {
    /// Unique identifier for this chart component instance
    id: String,
    /// Chart configuration
    config: ChartConfig,
    /// Chart data points
    data: Vec<DataPoint>,
}

impl ChartComponent {
    /// Create a new ChartComponent with default configuration
    pub fn new(id: String) -> Self {
        Self {
            id,
            config: ChartConfig::default(),
            data: Vec::new(),
        }
    }

    /// Create a ChartComponent with custom configuration
    pub fn with_config(id: String, config: ChartConfig) -> Self {
        Self {
            id,
            config,
            data: Vec::new(),
        }
    }

    /// Create a ChartComponent with data and configuration
    pub fn with_data_and_config(id: String, data: Vec<DataPoint>, config: ChartConfig) -> Self {
        Self { id, config, data }
    }

    /// Set chart data
    pub fn set_data(&mut self, data: Vec<DataPoint>) {
        self.data = data;
    }

    /// Get chart data reference
    pub fn get_data(&self) -> &[DataPoint] {
        &self.data
    }

    /// Set chart configuration
    pub fn set_config(&mut self, config: ChartConfig) {
        self.config = config;
    }

    /// Get chart configuration reference
    pub fn get_config(&self) -> &ChartConfig {
        &self.config
    }

    /// Parse chart data from text content
    pub fn parse_data_from_content(&mut self, content: &str) {
        self.data = Self::parse_chart_data(content);
    }

    /// Generate chart content as string (for legacy compatibility)
    pub fn generate(&self) -> String {
        self.generate_with_muxbox_title(None)
    }

    /// Generate chart content with muxbox title context to avoid duplication
    pub fn generate_with_muxbox_title(&self, muxbox_title: Option<&str>) -> String {
        if self.data.is_empty() {
            return "No chart data".to_string();
        }

        // Update config dimensions if not set
        let effective_config = ChartConfig {
            width: self.config.width.max(20),
            height: self.config.height.max(5),
            ..self.config.clone()
        };

        // Calculate smart layout based on chart type and data
        let layout = self.calculate_chart_layout(&effective_config, muxbox_title);

        match effective_config.chart_type {
            ChartType::Bar => self.generate_bar_chart(&effective_config, &layout, muxbox_title),
            ChartType::Line => self.generate_line_chart(&effective_config, &layout, muxbox_title),
            ChartType::Histogram => self.generate_histogram(&effective_config, &layout, muxbox_title),
            ChartType::Pie => self.generate_pie_chart(&effective_config, &layout, muxbox_title),
            ChartType::Scatter => self.generate_scatter_chart(&effective_config, &layout, muxbox_title),
        }
    }

    /// Render chart directly to screen buffer at specified bounds
    pub fn render(
        &self,
        bounds: &Bounds,
        buffer: &mut ScreenBuffer,
    ) {
        self.render_with_colors(bounds, &self.config.color, "black", buffer);
    }

    /// Render chart with custom colors
    pub fn render_with_colors(
        &self,
        bounds: &Bounds,
        fg_color: &str,
        bg_color: &str,
        buffer: &mut ScreenBuffer,
    ) {
        // Update chart dimensions based on actual bounds
        let mut render_config = self.config.clone();
        render_config.width = bounds.width();
        render_config.height = bounds.height();

        // Generate chart content
        let chart_content = if self.data.is_empty() {
            "No chart data".to_string()
        } else {
            let layout = self.calculate_chart_layout(&render_config, None);
            match render_config.chart_type {
                ChartType::Bar => self.generate_bar_chart(&render_config, &layout, None),
                ChartType::Line => self.generate_line_chart(&render_config, &layout, None),
                ChartType::Histogram => self.generate_histogram(&render_config, &layout, None),
                ChartType::Pie => self.generate_pie_chart(&render_config, &layout, None),
                ChartType::Scatter => self.generate_scatter_chart(&render_config, &layout, None),
            }
        };

        // Render chart content line by line
        let lines: Vec<&str> = chart_content.lines().collect();
        for (line_idx, &line) in lines.iter().take(bounds.height()).enumerate() {
            let y_pos = bounds.top() + line_idx;
            // Safe UTF-8 truncation to avoid splitting multi-byte characters
            let display_line = if line.chars().count() > bounds.width() {
                line.chars().take(bounds.width()).collect::<String>()
            } else {
                line.to_string()
            };
            
            print_with_color_and_background_at(
                bounds.left(),
                y_pos,
                &display_line,
                fg_color,
                bg_color,
                buffer,
            );
        }
    }

    /// Parse chart data from text content (static version for external use)
    pub fn parse_chart_data(content: &str) -> Vec<DataPoint> {
        let mut data = Vec::new();

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Support formats: "label,value" or "label:value" or "label value"
            let parts: Vec<&str> = if line.contains(',') {
                line.split(',').collect()
            } else if line.contains(':') {
                line.split(':').collect()
            } else {
                line.split_whitespace().collect()
            };

            if parts.len() >= 2 {
                let label = parts[0].trim().to_string();
                if let Ok(value) = parts[1].trim().parse::<f64>() {
                    data.push(DataPoint { label, value });
                }
            }
        }

        data
    }

    /// Calculate optimal layout dimensions for chart
    fn calculate_chart_layout(&self, config: &ChartConfig, muxbox_title: Option<&str>) -> ChartLayout {
        let total_width = config.width.max(20); // Minimum width
        let total_height = config.height.max(5); // Minimum height

        // Reserve space for title if present and different from muxbox title
        let title_height = if config.show_title {
            if let Some(title) = &config.title {
                let should_show_title = muxbox_title.map_or(true, |muxbox_title| muxbox_title != title);
                if should_show_title {
                    2
                } else {
                    0
                }
            } else {
                0
            }
        } else {
            0
        };

        match config.chart_type {
            ChartType::Bar => {
                // Calculate Y-axis label width based on data labels
                let y_label_width = self.data.iter().map(|p| p.label.len()).max().unwrap_or(0).max(3); // Minimum for values

                ChartLayout {
                    total_width,
                    _total_height: total_height,
                    chart_width: total_width.saturating_sub(y_label_width + 4), // +4 for separator and padding
                    chart_height: total_height.saturating_sub(title_height),
                    y_label_width,
                    title_height,
                    x_label_height: 0,
                }
            }
            ChartType::Line => {
                // Line charts need space for Y-axis values and X-axis labels
                let y_label_width = 6; // Space for numeric values like "100.0"
                let x_label_height = 1; // Space for X-axis values

                ChartLayout {
                    total_width,
                    _total_height: total_height,
                    chart_width: total_width.saturating_sub(y_label_width + 2),
                    chart_height: total_height.saturating_sub(title_height + x_label_height + 1),
                    y_label_width,
                    title_height,
                    x_label_height,
                }
            }
            ChartType::Histogram => {
                // Histograms need space for X-axis labels at bottom
                let x_label_height = 2; // Space for bin labels

                ChartLayout {
                    total_width,
                    _total_height: total_height,
                    chart_width: total_width,
                    chart_height: total_height.saturating_sub(title_height + x_label_height),
                    y_label_width: 0,
                    title_height,
                    x_label_height,
                }
            }
            ChartType::Pie => {
                // Pie charts need square space and legend area
                let legend_width = 15; // Space for legend on the right
                let chart_size = (total_width.saturating_sub(legend_width)).min(total_height.saturating_sub(title_height));
                
                ChartLayout {
                    total_width,
                    _total_height: total_height,
                    chart_width: chart_size,
                    chart_height: chart_size,
                    y_label_width: legend_width,
                    title_height,
                    x_label_height: 0,
                }
            }
            ChartType::Scatter => {
                // Scatter plots need space for X and Y axis labels
                let y_label_width = 8; // Space for Y-axis numeric values
                let x_label_height = 2; // Space for X-axis labels
                
                ChartLayout {
                    total_width,
                    _total_height: total_height,
                    chart_width: total_width.saturating_sub(y_label_width + 2),
                    chart_height: total_height.saturating_sub(title_height + x_label_height + 1),
                    y_label_width,
                    title_height,
                    x_label_height,
                }
            }
        }
    }

    /// Generate bar chart
    fn generate_bar_chart(&self, config: &ChartConfig, layout: &ChartLayout, muxbox_title: Option<&str>) -> String {
        let max_value = self.data.iter().map(|p| p.value).fold(0.0, f64::max);
        let mut result = String::new();

        // Only add title if it's different from the muxbox title
        if config.show_title {
            if let Some(title) = &config.title {
                let should_show_title = muxbox_title.map_or(true, |muxbox_title| muxbox_title != title);
                if should_show_title {
                    let title_centered = Self::center_text(title, layout.total_width);
                    result.push_str(&format!("{}\n", title_centered));
                    if layout.title_height > 1 {
                        result.push('\n');
                    }
                }
            }
        }

        // Calculate optimal bar width
        let bar_width = layout.chart_width.saturating_sub(2); // Reserve space for separator and value

        // Fill available height by distributing bars vertically
        let lines_per_bar = if self.data.is_empty() {
            1
        } else {
            (layout.chart_height / self.data.len()).max(1)
        };
        let total_lines_needed = self.data.len() * lines_per_bar;

        for (_i, point) in self.data.iter().enumerate() {
            let bar_length = if max_value > 0.0 {
                ((point.value / max_value) * bar_width as f64).round() as usize
            } else {
                0
            };

            // Right-align labels for better alignment
            let label = format!("{:>width$}", point.label, width = layout.y_label_width);

            // Create bar with proper alignment
            let bar = "█".repeat(bar_length);
            let padding = " ".repeat(bar_width.saturating_sub(bar_length));

            // Format value with consistent decimal places
            let value_str = if config.show_values {
                if point.value.fract() == 0.0 {
                    format!(" {:.0}", point.value)
                } else {
                    format!(" {:.1}", point.value)
                }
            } else {
                String::new()
            };

            // Add the main bar line
            result.push_str(&format!("{} │{}{}{}\n", label, bar, padding, value_str));

            // Add additional lines for this bar to fill vertical space
            for _ in 1..lines_per_bar {
                let empty_label = " ".repeat(layout.y_label_width);
                result.push_str(&format!(
                    "{} │{}\n",
                    empty_label,
                    " ".repeat(bar_width + value_str.len())
                ));
            }
        }

        // Fill remaining vertical space if needed
        let lines_used = total_lines_needed;
        for _ in lines_used..layout.chart_height {
            result.push_str(&" ".repeat(layout.y_label_width + bar_width + 10));
            result.push('\n');
        }

        result.trim_end().to_string() // Remove trailing newline
    }

    /// Generate line chart
    fn generate_line_chart(&self, config: &ChartConfig, layout: &ChartLayout, muxbox_title: Option<&str>) -> String {
        if self.data.len() < 2 {
            return "Need at least 2 data points for line chart".to_string();
        }

        let max_value = self.data.iter().map(|p| p.value).fold(0.0, f64::max);
        let min_value = self.data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
        let range = max_value - min_value;

        let mut result = String::new();

        // Only add title if it's different from the muxbox title
        if config.show_title {
            if let Some(title) = &config.title {
                let should_show_title = muxbox_title.map_or(true, |muxbox_title| muxbox_title != title);
                if should_show_title {
                    let title_centered = Self::center_text(title, layout.total_width);
                    result.push_str(&format!("{}\n", title_centered));
                    if layout.title_height > 1 {
                        result.push('\n');
                    }
                }
            }
        }

        // Create grid with proper dimensions
        let mut grid = vec![vec![' '; layout.chart_width]; layout.chart_height];

        // Plot data points
        for (i, point) in self.data.iter().enumerate() {
            let x = if self.data.len() > 1 {
                (i as f64 / (self.data.len() - 1) as f64 * (layout.chart_width - 1) as f64) as usize
            } else {
                layout.chart_width / 2
            };

            let y = if range > 0.0 {
                layout.chart_height
                    - 1
                    - ((point.value - min_value) / range * (layout.chart_height - 1) as f64) as usize
            } else {
                layout.chart_height / 2
            };

            if x < layout.chart_width && y < layout.chart_height {
                grid[y][x] = '●';
            }
        }

        // Convert grid to string with proper Y-axis labels
        for (row_idx, row) in grid.iter().enumerate() {
            // Add Y-axis value labels that correspond to actual data
            let y_label = if layout.y_label_width > 0 {
                // Calculate the actual Y value for this row
                let row_from_bottom = (layout.chart_height - 1).saturating_sub(row_idx);
                let y_value = if range > 0.0 {
                    min_value + (row_from_bottom as f64 / (layout.chart_height - 1) as f64) * range
                } else {
                    min_value
                };

                // Only show labels at specific intervals for readability
                let label_interval = layout.chart_height / 4; // Show ~4 labels
                if row_idx % label_interval.max(1) == 0 || row_idx == layout.chart_height - 1 {
                    format!("{:>width$.1}", y_value, width = layout.y_label_width)
                } else {
                    " ".repeat(layout.y_label_width)
                }
            } else {
                String::new()
            };

            result.push_str(&format!("{} {}\n", y_label, row.iter().collect::<String>()));
        }

        // Add X-axis labels showing actual data point labels
        if layout.x_label_height > 0 && !self.data.is_empty() {
            let padding = " ".repeat(layout.y_label_width + 1); // Align with chart
            result.push_str(&padding);

            // Show labels for each data point, but space them out to avoid crowding
            let max_labels = layout.chart_width / 6; // Each label needs ~6 chars
            let step = if self.data.len() > max_labels {
                self.data.len() / max_labels.max(1)
            } else {
                1
            };

            for (i, point) in self.data.iter().enumerate() {
                if i % step == 0 || i == self.data.len() - 1 {
                    let x = if self.data.len() > 1 {
                        (i as f64 / (self.data.len() - 1) as f64 * (layout.chart_width - 1) as f64) as usize
                    } else {
                        layout.chart_width / 2
                    };

                    // Position label under the data point
                    let spaces_before = x.saturating_sub(
                        result
                            .lines()
                            .last()
                            .unwrap_or("")
                            .len()
                            .saturating_sub(layout.y_label_width + 1),
                    );
                    if spaces_before < layout.chart_width {
                        result.push_str(&" ".repeat(spaces_before));
                        result.push_str(&point.label.chars().take(4).collect::<String>());
                    }
                }
            }
            result.push('\n');
        }

        result.trim_end().to_string()
    }

    /// Generate histogram
    fn generate_histogram(&self, config: &ChartConfig, layout: &ChartLayout, muxbox_title: Option<&str>) -> String {
        // Create bins based on value ranges
        let max_value = self.data.iter().map(|p| p.value).fold(0.0, f64::max);
        let min_value = self.data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);

        // Calculate optimal number of bins based on available width
        let max_bins = layout.chart_width / 2; // Each bin needs at least 2 chars width
        let bins = if self.data.len() <= max_bins {
            self.data.len() // Use one bin per data point if we have space
        } else {
            max_bins.min(12).max(6) // Otherwise use traditional histogram bins
        };

        let bin_size = if max_value > min_value {
            (max_value - min_value) / bins as f64
        } else {
            1.0
        };

        // For discrete data points, show each value as its own bar
        let histogram: Vec<usize> = if self.data.len() <= bins {
            // Use actual data values directly
            self.data.iter().map(|p| p.value as usize).collect()
        } else {
            // Traditional histogram with value range bins
            let mut hist = vec![0; bins];
            for point in &self.data {
                let bin_index = if bin_size > 0.0 && max_value > min_value {
                    let normalized = (point.value - min_value) / (max_value - min_value);
                    (normalized * (bins - 1) as f64).round() as usize
                } else {
                    0
                };
                let bin_index = bin_index.min(bins - 1);
                hist[bin_index] += 1;
            }
            hist
        };

        let max_count = *histogram.iter().max().unwrap_or(&1);

        let mut result = String::new();

        // Only add title if it's different from the muxbox title
        if config.show_title {
            if let Some(title) = &config.title {
                let should_show_title = muxbox_title.map_or(true, |muxbox_title| muxbox_title != title);
                if should_show_title {
                    let title_centered = Self::center_text(title, layout.total_width);
                    result.push_str(&format!("{}\n", title_centered));
                    if layout.title_height > 1 {
                        result.push('\n');
                    }
                }
            }
        }

        // Draw histogram bars from top to bottom, using full width
        for row in (0..layout.chart_height).rev() {
            let mut row_chars = 0;

            for (bin_idx, &count) in histogram.iter().enumerate() {
                let bar_height_needed = if max_count > 0 {
                    (count as f64 / max_count as f64 * layout.chart_height as f64) as usize
                } else {
                    0
                };

                if row < bar_height_needed {
                    result.push('█');
                } else {
                    result.push(' ');
                }
                row_chars += 1;

                // Calculate spacing to distribute bars across full width
                if bin_idx < bins - 1 {
                    let remaining_bins = bins - bin_idx - 1;
                    let remaining_width = layout.chart_width.saturating_sub(row_chars);
                    let spaces_needed = if remaining_bins > 0 {
                        (remaining_width / remaining_bins).max(1)
                    } else {
                        0
                    };

                    for _ in 0..spaces_needed {
                        result.push(' ');
                        row_chars += 1;
                        if row_chars >= layout.chart_width {
                            break;
                        }
                    }
                }

                if row_chars >= layout.chart_width {
                    break;
                }
            }

            // Fill remaining width with spaces
            while row_chars < layout.chart_width {
                result.push(' ');
                row_chars += 1;
            }

            result.push('\n');
        }

        // Add X-axis labels if enabled
        if layout.x_label_height > 0 {
            result.push('\n'); // Extra line before labels

            // Show more labels when using individual data points as bins
            let mut label_line = " ".repeat(layout.chart_width);

            if self.data.len() <= bins {
                // Show labels for actual data points with overlap prevention
                let min_label_spacing = 4; // Minimum characters between label starts
                let max_labels_for_width = layout.chart_width / min_label_spacing;
                let labels_to_show = self.data.len().min(max_labels_for_width).min(bins);

                for i in 0..labels_to_show {
                    let data_idx = if labels_to_show == self.data.len() {
                        i // Show all labels
                    } else {
                        (i * (self.data.len() - 1)) / (labels_to_show - 1).max(1) // Sample evenly
                    };

                    let point = &self.data[data_idx.min(self.data.len() - 1)];
                    let label = if point.value.fract() == 0.0 {
                        format!("{:.0}", point.value)
                    } else {
                        format!("{:.1}", point.value)
                    };

                    // Calculate position with better spacing
                    let label_position = if labels_to_show > 1 {
                        (i * (layout.chart_width - label.len())) / (labels_to_show - 1)
                    } else {
                        layout.chart_width / 2
                    };

                    // Check for overlap with previous labels
                    let start_pos = label_position;
                    let end_pos = start_pos + label.len();

                    if end_pos <= layout.chart_width {
                        // Check for overlap by ensuring this position doesn't overwrite existing labels
                        let line_chars: Vec<char> = label_line.chars().collect();
                        let can_place = (start_pos..end_pos)
                            .all(|pos| pos >= line_chars.len() || line_chars[pos] == ' ');

                        if can_place {
                            // Place the label
                            let label_chars: Vec<char> = label.chars().collect();
                            let mut line_chars = line_chars;
                            line_chars.resize(layout.chart_width, ' ');
                            for (j, &ch) in label_chars.iter().enumerate() {
                                if start_pos + j < line_chars.len() {
                                    line_chars[start_pos + j] = ch;
                                }
                            }
                            label_line = line_chars.into_iter().collect();
                        }
                    }
                }
            } else {
                // Traditional histogram range labels
                let num_labels = if layout.chart_width > 80 {
                    8
                } else if layout.chart_width > 60 {
                    6
                } else if layout.chart_width > 40 {
                    4
                } else {
                    3
                };

                for i in 0..num_labels {
                    let bin_idx = if num_labels == 1 {
                        0
                    } else {
                        (i * (bins - 1)) / (num_labels - 1)
                    };
                    let bin_start = min_value + bin_idx as f64 * bin_size;
                    let label = if bin_start.fract() == 0.0 {
                        format!("{:.0}", bin_start)
                    } else {
                        format!("{:.1}", bin_start)
                    };

                    // Calculate position across the full width
                    let label_position = if bins > 1 {
                        (bin_idx * layout.chart_width) / bins
                    } else {
                        layout.chart_width / 2
                    };

                    // Place label if it fits
                    let start_pos = label_position.saturating_sub(label.len() / 2);
                    let end_pos = start_pos + label.len();

                    if end_pos <= layout.chart_width {
                        // Replace spaces with the label characters
                        let label_chars: Vec<char> = label.chars().collect();
                        let mut line_chars: Vec<char> = label_line.chars().collect();
                        for (j, &ch) in label_chars.iter().enumerate() {
                            if start_pos + j < line_chars.len() {
                                line_chars[start_pos + j] = ch;
                            }
                        }
                        label_line = line_chars.into_iter().collect();
                    }
                }
            }

            result.push_str(&label_line);
        }

        result.trim_end().to_string()
    }

    /// Generate pie chart
    fn generate_pie_chart(&self, config: &ChartConfig, layout: &ChartLayout, muxbox_title: Option<&str>) -> String {
        let total_value: f64 = self.data.iter().map(|p| p.value).sum();
        if total_value == 0.0 {
            return "No data for pie chart".to_string();
        }

        let mut result = String::new();

        // Only add title if it's different from the muxbox title
        if config.show_title {
            if let Some(title) = &config.title {
                let should_show_title = muxbox_title.map_or(true, |muxbox_title| muxbox_title != title);
                if should_show_title {
                    let title_centered = Self::center_text(title, layout.total_width);
                    result.push_str(&format!("{}\n", title_centered));
                    if layout.title_height > 1 {
                        result.push('\n');
                    }
                }
            }
        }

        // Calculate pie chart dimensions - make it as circular as possible
        let radius = (layout.chart_width.min(layout.chart_height) / 2).max(3);
        let center_x = layout.chart_width / 2;
        let center_y = layout.chart_height / 2;

        // Create grid for pie chart
        let mut grid = vec![vec![' '; layout.chart_width]; layout.chart_height];

        // Calculate angles for each slice
        let mut current_angle = 0.0;
        let pie_chars = ['█', '▓', '▒', '░', '●', '◐', '◑', '◒'];
        
        for (slice_idx, point) in self.data.iter().enumerate() {
            let slice_angle = (point.value / total_value) * 2.0 * std::f64::consts::PI;
            let slice_char = pie_chars[slice_idx % pie_chars.len()];
            
            // Fill the slice using simple sector approximation
            let angle_steps = (slice_angle * radius as f64 / 2.0).ceil() as usize;
            for step in 0..angle_steps.max(1) {
                let angle = current_angle + (step as f64 / angle_steps.max(1) as f64) * slice_angle;
                let end_angle = current_angle + slice_angle;
                
                // Draw lines from center to edge for this slice
                for r in 1..=radius {
                    let x = center_x as f64 + (r as f64 * angle.cos());
                    let y = center_y as f64 + (r as f64 * angle.sin() / 2.0); // Adjust for character aspect ratio
                    
                    let grid_x = x.round() as usize;
                    let grid_y = y.round() as usize;
                    
                    if grid_x < layout.chart_width && grid_y < layout.chart_height {
                        grid[grid_y][grid_x] = slice_char;
                    }
                }
            }
            current_angle += slice_angle;
        }

        // Convert grid to string
        for row in &grid {
            result.push_str(&row.iter().collect::<String>());
            result.push('\n');
        }

        // Add legend on the right side
        if layout.y_label_width > 0 {
            let legend_lines: Vec<String> = result.lines().collect::<Vec<_>>().into_iter().map(|s| s.to_string()).collect();
            result.clear();
            
            for (line_idx, line) in legend_lines.iter().enumerate() {
                result.push_str(line);
                
                // Add legend entries
                if line_idx < self.data.len() {
                    let point = &self.data[line_idx];
                    let percentage = (point.value / total_value) * 100.0;
                    let slice_char = pie_chars[line_idx % pie_chars.len()];
                    let legend_text = format!(" {} {}: {:.1}%", slice_char, point.label, percentage);
                    
                    // Pad line to fit legend
                    let padding = layout.total_width.saturating_sub(line.len() + legend_text.len());
                    result.push_str(&" ".repeat(padding));
                    result.push_str(&legend_text);
                }
                result.push('\n');
            }
        }

        result.trim_end().to_string()
    }

    /// Generate scatter chart
    fn generate_scatter_chart(&self, config: &ChartConfig, layout: &ChartLayout, muxbox_title: Option<&str>) -> String {
        if self.data.len() < 2 {
            return "Need at least 2 data points for scatter chart".to_string();
        }

        // For scatter plots, we need X and Y values. We'll use the index as X and value as Y
        // In a real implementation, DataPoint might have both x and y values
        let max_y = self.data.iter().map(|p| p.value).fold(0.0, f64::max);
        let min_y = self.data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
        let max_x = self.data.len() as f64;
        let min_x = 0.0;
        
        let x_range = max_x - min_x;
        let y_range = max_y - min_y;

        let mut result = String::new();

        // Only add title if it's different from the muxbox title
        if config.show_title {
            if let Some(title) = &config.title {
                let should_show_title = muxbox_title.map_or(true, |muxbox_title| muxbox_title != title);
                if should_show_title {
                    let title_centered = Self::center_text(title, layout.total_width);
                    result.push_str(&format!("{}\n", title_centered));
                    if layout.title_height > 1 {
                        result.push('\n');
                    }
                }
            }
        }

        // Create grid for scatter plot
        let mut grid = vec![vec![' '; layout.chart_width]; layout.chart_height];
        
        // Draw grid lines if enabled
        if config.show_grid {
            // Vertical grid lines
            for x in (0..layout.chart_width).step_by(layout.chart_width / 5) {
                for y in 0..layout.chart_height {
                    grid[y][x] = '│';
                }
            }
            // Horizontal grid lines
            for y in (0..layout.chart_height).step_by(layout.chart_height / 4) {
                for x in 0..layout.chart_width {
                    grid[y][x] = '─';
                }
            }
        }

        // Plot scatter points with different symbols for variety
        let scatter_symbols = ['●', '◆', '▲', '■', '♦', '✦', '⬢', '⬣'];
        
        for (point_idx, point) in self.data.iter().enumerate() {
            let x_pos = if x_range > 0.0 {
                ((point_idx as f64 - min_x) / x_range * (layout.chart_width - 1) as f64) as usize
            } else {
                layout.chart_width / 2
            };
            
            let y_pos = if y_range > 0.0 {
                layout.chart_height - 1 - 
                ((point.value - min_y) / y_range * (layout.chart_height - 1) as f64) as usize
            } else {
                layout.chart_height / 2
            };

            if x_pos < layout.chart_width && y_pos < layout.chart_height {
                let symbol = scatter_symbols[point_idx % scatter_symbols.len()];
                grid[y_pos][x_pos] = symbol;
            }
        }

        // Convert grid to string with Y-axis labels
        for (row_idx, row) in grid.iter().enumerate() {
            // Add Y-axis value labels
            let y_label = if layout.y_label_width > 0 {
                let row_from_bottom = (layout.chart_height - 1).saturating_sub(row_idx);
                let y_value = if y_range > 0.0 {
                    min_y + (row_from_bottom as f64 / (layout.chart_height - 1) as f64) * y_range
                } else {
                    min_y
                };
                
                // Show labels at intervals
                let label_interval = (layout.chart_height / 4).max(1);
                if row_idx % label_interval == 0 || row_idx == layout.chart_height - 1 {
                    format!("{:>width$.1}", y_value, width = layout.y_label_width)
                } else {
                    " ".repeat(layout.y_label_width)
                }
            } else {
                String::new()
            };

            result.push_str(&format!("{} {}\n", y_label, row.iter().collect::<String>()));
        }

        // Add X-axis labels
        if layout.x_label_height > 0 && !self.data.is_empty() {
            let padding = " ".repeat(layout.y_label_width + 1);
            result.push_str(&padding);
            
            // Show X labels (indices or labels)
            let step = if self.data.len() > 10 { self.data.len() / 8 } else { 1 };
            for (i, point) in self.data.iter().enumerate().step_by(step) {
                let x_pos = if x_range > 0.0 {
                    ((i as f64 - min_x) / x_range * (layout.chart_width - 1) as f64) as usize
                } else {
                    layout.chart_width / 2
                };
                
                // Position label under the point
                let spaces_before = x_pos.saturating_sub(
                    result.lines().last().unwrap_or("").len().saturating_sub(layout.y_label_width + 1)
                );
                if spaces_before < layout.chart_width {
                    result.push_str(&" ".repeat(spaces_before));
                    result.push_str(&point.label.chars().take(4).collect::<String>());
                }
            }
            result.push('\n');
        }

        result.trim_end().to_string()
    }

    /// Center text within given width
    fn center_text(text: &str, width: usize) -> String {
        if text.len() >= width {
            return text.to_string();
        }

        let padding = width - text.len();
        let left_pad = padding / 2;
        let right_pad = padding - left_pad;

        format!("{}{}{}", " ".repeat(left_pad), text, " ".repeat(right_pad))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Test helper function
    fn create_test_buffer() -> ScreenBuffer {
        ScreenBuffer::new()
    }

    fn create_test_data() -> Vec<DataPoint> {
        vec![
            DataPoint { label: "Jan".to_string(), value: 10.0 },
            DataPoint { label: "Feb".to_string(), value: 20.0 },
            DataPoint { label: "Mar".to_string(), value: 15.0 },
        ]
    }

    #[test]
    fn test_chart_component_creation() {
        let chart = ChartComponent::new("test_chart".to_string());
        assert_eq!(chart.id, "test_chart");
        assert!(chart.data.is_empty());
        assert_eq!(chart.config.chart_type, ChartType::Bar);
    }

    #[test]
    fn test_chart_component_with_config() {
        let config = ChartConfig {
            chart_type: ChartType::Line,
            title: Some("Test Chart".to_string()),
            width: 60,
            height: 20,
            color: "red".to_string(),
            show_title: true,
            show_values: false,
            show_grid: true,
        };
        
        let chart = ChartComponent::with_config("test".to_string(), config.clone());
        assert_eq!(chart.config.chart_type, ChartType::Line);
        assert_eq!(chart.config.width, 60);
        assert_eq!(chart.config.height, 20);
        assert_eq!(chart.config.color, "red");
        assert!(!chart.config.show_values);
    }

    #[test]
    fn test_data_parsing_from_content() {
        let mut chart = ChartComponent::new("test".to_string());
        let content = "Jan,10\nFeb,20\nMar,15";
        chart.parse_data_from_content(content);
        
        assert_eq!(chart.data.len(), 3);
        assert_eq!(chart.data[0].label, "Jan");
        assert_eq!(chart.data[0].value, 10.0);
        assert_eq!(chart.data[2].value, 15.0);
    }

    #[test]
    fn test_bar_chart_generation() {
        let data = create_test_data();
        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: Some("Test Bar Chart".to_string()),
            width: 30,
            height: 10,
            color: "blue".to_string(),
            show_title: true,
            show_values: true,
            show_grid: false,
        };
        
        let chart = ChartComponent::with_data_and_config("test".to_string(), data, config);
        let result = chart.generate();
        
        assert!(result.contains("Test Bar Chart"));
        assert!(result.contains("Jan"));
        assert!(result.contains("█")); // Bar characters
        assert!(result.contains("10")); // Values should be shown
    }

    #[test]
    fn test_line_chart_generation() {
        let data = create_test_data();
        let config = ChartConfig {
            chart_type: ChartType::Line,
            title: Some("Test Line Chart".to_string()),
            width: 40,
            height: 15,
            color: "green".to_string(),
            show_title: true,
            show_values: true,
            show_grid: false,
        };
        
        let chart = ChartComponent::with_data_and_config("test".to_string(), data, config);
        let result = chart.generate();
        
        assert!(result.contains("Test Line Chart"));
        assert!(result.contains("●")); // Data points
    }

    #[test]
    fn test_histogram_generation() {
        let data = create_test_data();
        let config = ChartConfig {
            chart_type: ChartType::Histogram,
            title: Some("Test Histogram".to_string()),
            width: 50,
            height: 12,
            color: "yellow".to_string(),
            show_title: true,
            show_values: false,
            show_grid: false,
        };
        
        let chart = ChartComponent::with_data_and_config("test".to_string(), data, config);
        let result = chart.generate();
        
        assert!(result.contains("Test Histogram"));
        assert!(result.contains("█")); // Histogram bars
    }

    #[test]
    fn test_chart_type_from_string() {
        assert_eq!("bar".parse::<ChartType>().unwrap(), ChartType::Bar);
        assert_eq!("line".parse::<ChartType>().unwrap(), ChartType::Line);
        assert_eq!("histogram".parse::<ChartType>().unwrap(), ChartType::Histogram);
        assert_eq!("pie".parse::<ChartType>().unwrap(), ChartType::Pie);
        assert_eq!("scatter".parse::<ChartType>().unwrap(), ChartType::Scatter);
        assert!("invalid".parse::<ChartType>().is_err());
    }

    #[test]
    fn test_title_suppression_with_muxbox_title() {
        let data = create_test_data();
        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: Some("Same Title".to_string()),
            width: 30,
            height: 10,
            color: "blue".to_string(),
            show_title: true,
            show_values: true,
            show_grid: false,
        };
        
        let chart = ChartComponent::with_data_and_config("test".to_string(), data, config);
        let result = chart.generate_with_muxbox_title(Some("Same Title"));
        
        // Title should be suppressed when it matches muxbox title
        let title_count = result.matches("Same Title").count();
        assert_eq!(title_count, 0);
    }

    #[test]
    fn test_empty_data_handling() {
        let chart = ChartComponent::new("test".to_string());
        let result = chart.generate();
        assert_eq!(result, "No chart data");
    }

    #[test]
    fn test_render_to_buffer() {
        let data = create_test_data();
        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: None,
            width: 30,
            height: 8,
            color: "blue".to_string(),
            show_title: false,
            show_values: true,
            show_grid: false,
        };
        
        let chart = ChartComponent::with_data_and_config("test".to_string(), data, config);
        let bounds = Bounds::new(5, 5, 30, 8);
        let mut buffer = create_test_buffer();
        
        // Should not panic and should render without issues
        chart.render(&bounds, &mut buffer);
        
        // Test rendering with custom colors
        chart.render_with_colors(&bounds, "red", "black", &mut buffer);
    }

    #[test]
    fn test_parse_chart_data_formats() {
        // Test comma format
        let data1 = ChartComponent::parse_chart_data("A,10\nB,20");
        assert_eq!(data1.len(), 2);
        assert_eq!(data1[0].label, "A");
        assert_eq!(data1[0].value, 10.0);
        
        // Test colon format
        let data2 = ChartComponent::parse_chart_data("C:30\nD:40");
        assert_eq!(data2.len(), 2);
        assert_eq!(data2[0].label, "C");
        assert_eq!(data2[0].value, 30.0);
        
        // Test space format
        let data3 = ChartComponent::parse_chart_data("E 50\nF 60");
        assert_eq!(data3.len(), 2);
        assert_eq!(data3[0].label, "E");
        assert_eq!(data3[0].value, 50.0);
        
        // Test comments and empty lines
        let data4 = ChartComponent::parse_chart_data("# Comment\n\nG,70\n");
        assert_eq!(data4.len(), 1);
        assert_eq!(data4[0].label, "G");
        assert_eq!(data4[0].value, 70.0);
    }

    #[test]
    fn test_chart_layout_calculation() {
        let data = create_test_data();
        let chart = ChartComponent::with_data_and_config(
            "test".to_string(), 
            data, 
            ChartConfig::default()
        );
        
        let config = ChartConfig {
            chart_type: ChartType::Bar,
            width: 50,
            height: 20,
            ..ChartConfig::default()
        };
        
        let layout = chart.calculate_chart_layout(&config, None);
        assert_eq!(layout.total_width, 50);
        assert!(layout.chart_width < layout.total_width); // Should reserve space for labels
        assert!(layout.y_label_width > 0); // Should calculate label width from data
    }

    #[test]
    fn test_pie_chart_generation() {
        let data = create_test_data();
        let config = ChartConfig {
            chart_type: ChartType::Pie,
            title: Some("Test Pie Chart".to_string()),
            width: 50,
            height: 20,
            color: "blue".to_string(),
            show_title: true,
            show_values: true,
            show_grid: false,
        };
        
        let chart = ChartComponent::with_data_and_config("test".to_string(), data, config);
        let result = chart.generate();
        
        assert!(result.contains("Test Pie Chart"));
        // Should contain pie chart characters
        assert!(result.chars().any(|c| matches!(c, '█' | '▓' | '▒' | '░' | '●')));
        // Should contain percentage in legend
        assert!(result.contains("%"));
    }

    #[test]
    fn test_scatter_chart_generation() {
        let data = create_test_data();
        let config = ChartConfig {
            chart_type: ChartType::Scatter,
            title: Some("Test Scatter Plot".to_string()),
            width: 60,
            height: 25,
            color: "green".to_string(),
            show_title: true,
            show_values: false,
            show_grid: true,
        };
        
        let chart = ChartComponent::with_data_and_config("test".to_string(), data, config);
        let result = chart.generate();
        
        assert!(result.contains("Test Scatter Plot"));
        // Should contain scatter plot symbols
        assert!(result.chars().any(|c| matches!(c, '●' | '◆' | '▲' | '■')));
        // Should contain grid lines when enabled
        assert!(result.contains('│') || result.contains('─'));
    }

    #[test]
    fn test_pie_chart_empty_data() {
        let chart = ChartComponent::with_config(
            "test".to_string(),
            ChartConfig {
                chart_type: ChartType::Pie,
                ..ChartConfig::default()
            }
        );
        let result = chart.generate();
        assert_eq!(result, "No chart data");
    }

    #[test]
    fn test_pie_chart_zero_values() {
        let data = vec![
            DataPoint { label: "A".to_string(), value: 0.0 },
            DataPoint { label: "B".to_string(), value: 0.0 },
        ];
        
        let chart = ChartComponent::with_data_and_config(
            "test".to_string(),
            data,
            ChartConfig {
                chart_type: ChartType::Pie,
                ..ChartConfig::default()
            }
        );
        
        let result = chart.generate();
        assert_eq!(result, "No data for pie chart");
    }

    #[test]
    fn test_scatter_chart_insufficient_data() {
        let data = vec![
            DataPoint { label: "Single".to_string(), value: 10.0 },
        ];
        
        let chart = ChartComponent::with_data_and_config(
            "test".to_string(),
            data,
            ChartConfig {
                chart_type: ChartType::Scatter,
                ..ChartConfig::default()
            }
        );
        
        let result = chart.generate();
        assert_eq!(result, "Need at least 2 data points for scatter chart");
    }

    #[test]
    fn test_all_chart_types_with_same_data() {
        let data = create_test_data();
        
        let chart_types = [ChartType::Bar, ChartType::Line, ChartType::Histogram, ChartType::Pie, ChartType::Scatter];
        
        for chart_type in &chart_types {
            let config = ChartConfig {
                chart_type: chart_type.clone(),
                title: Some(format!("{:?} Chart", chart_type)),
                width: 40,
                height: 15,
                color: "blue".to_string(),
                show_title: true,
                show_values: true,
                show_grid: false,
            };
            
            let chart = ChartComponent::with_data_and_config("test".to_string(), data.clone(), config);
            let result = chart.generate();
            
            // All chart types should generate some content
            assert!(!result.is_empty(), "Chart type {:?} generated empty content", chart_type);
            assert!(result.contains(&format!("{:?} Chart", chart_type)), "Chart type {:?} missing title", chart_type);
        }
    }

    #[test]
    fn test_chart_layout_calculation_new_types() {
        let data = create_test_data();
        let chart = ChartComponent::with_data_and_config(
            "test".to_string(), 
            data, 
            ChartConfig::default()
        );
        
        // Test pie chart layout
        let pie_config = ChartConfig {
            chart_type: ChartType::Pie,
            width: 50,
            height: 20,
            ..ChartConfig::default()
        };
        let pie_layout = chart.calculate_chart_layout(&pie_config, None);
        assert!(pie_layout.y_label_width > 0); // Should have legend space
        assert_eq!(pie_layout.chart_width, pie_layout.chart_height); // Should be square-ish
        
        // Test scatter chart layout
        let scatter_config = ChartConfig {
            chart_type: ChartType::Scatter,
            width: 60,
            height: 25,
            ..ChartConfig::default()
        };
        let scatter_layout = chart.calculate_chart_layout(&scatter_config, None);
        assert!(scatter_layout.y_label_width > 0); // Should have Y-axis labels
        assert!(scatter_layout.x_label_height > 0); // Should have X-axis labels
        assert!(scatter_layout.chart_width < scatter_layout.total_width); // Should reserve space for labels
    }

    #[test]
    fn test_scatter_chart_with_grid() {
        let data = create_test_data();
        let config = ChartConfig {
            chart_type: ChartType::Scatter,
            title: None,
            width: 30,
            height: 15,
            color: "red".to_string(),
            show_title: false,
            show_values: false,
            show_grid: true,
        };
        
        let chart = ChartComponent::with_data_and_config("test".to_string(), data, config);
        let result = chart.generate();
        
        // Should contain grid characters when show_grid is true
        assert!(result.contains('│') || result.contains('─'), "Scatter chart with grid should contain grid lines");
    }

    #[test]
    fn test_center_text() {
        assert_eq!(ChartComponent::center_text("test", 10), "   test   ");
        assert_eq!(ChartComponent::center_text("hello", 9), "  hello  ");
        assert_eq!(ChartComponent::center_text("toolong", 5), "toolong");
    }

    #[test]
    fn test_data_setters_and_getters() {
        let mut chart = ChartComponent::new("test".to_string());
        let data = create_test_data();
        
        chart.set_data(data.clone());
        assert_eq!(chart.get_data().len(), 3);
        assert_eq!(chart.get_data()[0].label, "Jan");
        
        let new_config = ChartConfig {
            chart_type: ChartType::Histogram,
            ..ChartConfig::default()
        };
        
        chart.set_config(new_config.clone());
        assert_eq!(chart.get_config().chart_type, ChartType::Histogram);
    }
}