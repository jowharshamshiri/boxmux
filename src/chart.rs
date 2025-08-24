// Chart rendering functionality

/// Chart data point
#[derive(Debug, Clone)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
}

/// Chart configuration
#[derive(Debug, Clone)]
pub struct ChartConfig {
    pub chart_type: ChartType,
    pub title: Option<String>,
    pub width: usize,
    pub height: usize,
    pub color: String,
}

/// Chart layout dimensions and positioning
#[derive(Debug, Clone)]
struct ChartLayout {
    /// Total available width
    pub total_width: usize,
    /// Total available height
    pub total_height: usize,
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

/// Supported chart types
#[derive(Debug, Clone)]
pub enum ChartType {
    Bar,
    Line,
    Histogram,
}

/// Generate ASCII+ chart using Unicode block characters
pub fn generate_chart(data: &[DataPoint], config: &ChartConfig) -> String {
    generate_chart_with_panel_title(data, config, None)
}

/// Generate chart with panel title context to avoid duplication
pub fn generate_chart_with_panel_title(
    data: &[DataPoint],
    config: &ChartConfig,
    panel_title: Option<&str>,
) -> String {
    if data.is_empty() {
        return "No chart data".to_string();
    }

    // Calculate smart layout based on chart type and data
    let layout = calculate_chart_layout(data, config, panel_title);

    match config.chart_type {
        ChartType::Bar => generate_bar_chart(data, config, &layout, panel_title),
        ChartType::Line => generate_line_chart(data, config, &layout, panel_title),
        ChartType::Histogram => generate_histogram(data, config, &layout, panel_title),
    }
}

/// Calculate optimal layout dimensions for chart
fn calculate_chart_layout(
    data: &[DataPoint],
    config: &ChartConfig,
    panel_title: Option<&str>,
) -> ChartLayout {
    let total_width = config.width.max(20); // Minimum width
    let total_height = config.height.max(5); // Minimum height

    // Reserve space for title if present and different from panel title
    let title_height = if let Some(title) = &config.title {
        let should_show_title = panel_title.map_or(true, |panel_title| panel_title != title);
        if should_show_title {
            2
        } else {
            0
        }
    } else {
        0
    };

    match config.chart_type {
        ChartType::Bar => {
            // Calculate Y-axis label width based on data labels
            let y_label_width = data.iter().map(|p| p.label.len()).max().unwrap_or(0).max(3); // Minimum for values

            ChartLayout {
                total_width,
                total_height,
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
                total_height,
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
                total_height,
                chart_width: total_width,
                chart_height: total_height.saturating_sub(title_height + x_label_height),
                y_label_width: 0,
                title_height,
                x_label_height,
            }
        }
    }
}

fn generate_bar_chart(
    data: &[DataPoint],
    config: &ChartConfig,
    layout: &ChartLayout,
    panel_title: Option<&str>,
) -> String {
    let max_value = data.iter().map(|p| p.value).fold(0.0, f64::max);
    let mut result = String::new();

    // Only add title if it's different from the panel title
    if let Some(title) = &config.title {
        let should_show_title = panel_title.map_or(true, |panel_title| panel_title != title);
        if should_show_title {
            let title_centered = center_text(title, layout.total_width);
            result.push_str(&format!("{}\n", title_centered));
            if layout.title_height > 1 {
                result.push('\n');
            }
        }
    }

    // Calculate optimal bar width
    let bar_width = layout.chart_width.saturating_sub(2); // Reserve space for separator and value

    // Fill available height by distributing bars vertically
    let lines_per_bar = if data.is_empty() {
        1
    } else {
        (layout.chart_height / data.len()).max(1)
    };
    let total_lines_needed = data.len() * lines_per_bar;

    for (_i, point) in data.iter().enumerate() {
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
        let value_str = if point.value.fract() == 0.0 {
            format!("{:.0}", point.value)
        } else {
            format!("{:.1}", point.value)
        };

        // Add the main bar line
        result.push_str(&format!("{} │{}{} {}\n", label, bar, padding, value_str));

        // Add additional lines for this bar to fill vertical space
        for _ in 1..lines_per_bar {
            let empty_label = " ".repeat(layout.y_label_width);
            result.push_str(&format!(
                "{} │{}\n",
                empty_label,
                " ".repeat(bar_width + value_str.len() + 1)
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

fn generate_line_chart(
    data: &[DataPoint],
    config: &ChartConfig,
    layout: &ChartLayout,
    panel_title: Option<&str>,
) -> String {
    if data.len() < 2 {
        return "Need at least 2 data points for line chart".to_string();
    }

    let max_value = data.iter().map(|p| p.value).fold(0.0, f64::max);
    let min_value = data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);
    let range = max_value - min_value;

    let mut result = String::new();

    // Only add title if it's different from the panel title
    if let Some(title) = &config.title {
        let should_show_title = panel_title.map_or(true, |panel_title| panel_title != title);
        if should_show_title {
            let title_centered = center_text(title, layout.total_width);
            result.push_str(&format!("{}\n", title_centered));
            if layout.title_height > 1 {
                result.push('\n');
            }
        }
    }

    // Create grid with proper dimensions
    let mut grid = vec![vec![' '; layout.chart_width]; layout.chart_height];

    // Plot data points and lines first
    for (i, point) in data.iter().enumerate() {
        let x = if data.len() > 1 {
            (i as f64 / (data.len() - 1) as f64 * (layout.chart_width - 1) as f64) as usize
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

        // Lines removed for cleaner appearance - data points only
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
    if layout.x_label_height > 0 && !data.is_empty() {
        let padding = " ".repeat(layout.y_label_width + 1); // Align with chart
        result.push_str(&padding);

        // Show labels for each data point, but space them out to avoid crowding
        let max_labels = layout.chart_width / 6; // Each label needs ~6 chars
        let step = if data.len() > max_labels {
            data.len() / max_labels.max(1)
        } else {
            1
        };

        for (i, point) in data.iter().enumerate() {
            if i % step == 0 || i == data.len() - 1 {
                let x = if data.len() > 1 {
                    (i as f64 / (data.len() - 1) as f64 * (layout.chart_width - 1) as f64) as usize
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

// Line drawing functions removed - chart now shows data points only for cleaner appearance

fn generate_histogram(
    data: &[DataPoint],
    config: &ChartConfig,
    layout: &ChartLayout,
    panel_title: Option<&str>,
) -> String {
    // Create bins based on value ranges
    let max_value = data.iter().map(|p| p.value).fold(0.0, f64::max);
    let min_value = data.iter().map(|p| p.value).fold(f64::INFINITY, f64::min);

    // Calculate optimal number of bins based on available width
    let max_bins = layout.chart_width / 2; // Each bin needs at least 2 chars width
    let bins = if data.len() <= max_bins {
        data.len() // Use one bin per data point if we have space
    } else {
        max_bins.min(12).max(6) // Otherwise use traditional histogram bins
    };

    let bin_size = if max_value > min_value {
        (max_value - min_value) / bins as f64
    } else {
        1.0
    };

    // For discrete data points, show each value as its own bar
    let histogram: Vec<usize> = if data.len() <= bins {
        // Use actual data values directly
        data.iter().map(|p| p.value as usize).collect()
    } else {
        // Traditional histogram with value range bins
        let mut hist = vec![0; bins];
        for point in data {
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

    // Only add title if it's different from the panel title
    if let Some(title) = &config.title {
        let should_show_title = panel_title.map_or(true, |panel_title| panel_title != title);
        if should_show_title {
            let title_centered = center_text(title, layout.total_width);
            result.push_str(&format!("{}\n", title_centered));
            if layout.title_height > 1 {
                result.push('\n');
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

    // Add simplified X-axis labels - show only a few key values to avoid crowding
    if layout.x_label_height > 0 {
        result.push('\n'); // Extra line before labels

        // Show more labels when using individual data points as bins
        let mut label_line = " ".repeat(layout.chart_width);

        if data.len() <= bins {
            // Show labels for actual data points with overlap prevention
            let min_label_spacing = 4; // Minimum characters between label starts
            let max_labels_for_width = layout.chart_width / min_label_spacing;
            let labels_to_show = data.len().min(max_labels_for_width).min(bins);

            for i in 0..labels_to_show {
                let data_idx = if labels_to_show == data.len() {
                    i // Show all labels
                } else {
                    (i * (data.len() - 1)) / (labels_to_show - 1).max(1) // Sample evenly
                };

                let point = &data[data_idx.min(data.len() - 1)];
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
            // Traditional histogram range labels - show more labels
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

/// Parse chart data from text content
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_chart_data() {
        let content = "Jan,10\nFeb,20\nMar,15";
        let data = parse_chart_data(content);

        assert_eq!(data.len(), 3);
        assert_eq!(data[0].label, "Jan");
        assert_eq!(data[0].value, 10.0);
        assert_eq!(data[2].value, 15.0);
    }

    #[test]
    fn test_bar_chart_generation() {
        let data = vec![
            DataPoint {
                label: "Item1".to_string(),
                value: 10.0,
            },
            DataPoint {
                label: "Item2".to_string(),
                value: 25.0,
            },
        ];

        let config = ChartConfig {
            chart_type: ChartType::Bar,
            title: Some("Test Chart".to_string()),
            width: 30,
            height: 10,
            color: "blue".to_string(),
        };

        let result = generate_chart(&data, &config);
        assert!(result.contains("Test Chart"));
        assert!(result.contains("Item1"));
        assert!(result.contains("█"));
    }
}
