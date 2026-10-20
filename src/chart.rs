//! Text charts drawn from `label,value` data: bar, line, histogram, pie and scatter.

use serde::{Deserialize, Serialize};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

/// The smallest area, in columns and rows, a chart is drawn in.
pub const MIN_WIDTH: usize = 20;
pub const MIN_HEIGHT: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChartType {
    /// One horizontal bar per data point.
    Bar,
    /// The values in order, as points on a grid.
    Line,
    /// How many values fall into each range of values.
    Histogram,
    /// Each value's share of the total.
    Pie,
    /// Points at (label, value); labels that are not all numbers are placed in order.
    Scatter,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DataPoint {
    pub label: String,
    pub value: f64,
}

/// Parses chart data: one `label,value`, `label:value` or `label value` per line. Blank lines
/// and lines starting with `#` are skipped; any other line that is not a label and a finite
/// number is an error naming the line.
pub fn parse_chart_data(content: &str) -> Result<Vec<DataPoint>, String> {
    let mut data = Vec::new();
    for (number, line) in content.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let split = if line.contains(',') {
            line.split_once(',')
        } else if line.contains(':') {
            line.split_once(':')
        } else {
            line.split_once(char::is_whitespace)
        };
        let point = split.and_then(|(label, value)| {
            let value: f64 = value.trim().parse().ok()?;
            value.is_finite().then(|| DataPoint {
                label: label.trim().to_string(),
                value,
            })
        });
        match point {
            Some(point) => data.push(point),
            None => {
                return Err(format!(
                    "chart data line {} is not 'label,value': {}",
                    number + 1,
                    line
                ))
            }
        }
    }
    Ok(data)
}

/// The chart of `kind` for `data`, at most `width` columns by `height` rows.
pub fn chart_text(kind: ChartType, data: &[DataPoint], width: usize, height: usize) -> String {
    if width < MIN_WIDTH || height < MIN_HEIGHT {
        return "too small".chars().take(width).collect();
    }
    if data.is_empty() {
        return "no chart data".to_string();
    }
    let lines = match kind {
        ChartType::Bar => bar_chart(data, width, height),
        ChartType::Line => point_chart(&line_points(data), data, width, height, '●', true),
        ChartType::Scatter => point_chart(&scatter_points(data), data, width, height, '◆', false),
        ChartType::Histogram => histogram(data, width, height),
        ChartType::Pie => pie_chart(data, width, height),
    };
    lines.join("\n")
}

/// A number as short as it can be written: `12`, `3.5`, `0.25`.
fn number(value: f64) -> String {
    let text = format!("{:.2}", value);
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".to_string()
    } else {
        text.to_string()
    }
}

/// `text` cut or padded to exactly `width` columns.
fn pad(text: &str, width: usize) -> String {
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > width {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push_str(&" ".repeat(width - used));
    out
}

fn bar_chart(data: &[DataPoint], width: usize, height: usize) -> Vec<String> {
    let shown = &data[..data.len().min(height)];
    let label_width = shown
        .iter()
        .map(|p| p.label.width())
        .max()
        .unwrap_or(0)
        .min(width / 3);
    let value_width = shown
        .iter()
        .map(|p| number(p.value).width())
        .max()
        .unwrap_or(0)
        .min(width / 3);
    // label, " │", bar, " ", value
    let bar_space = width.saturating_sub(label_width + value_width + 3);
    let max = shown.iter().map(|p| p.value.max(0.0)).fold(0.0, f64::max);
    shown
        .iter()
        .map(|point| {
            let length = if max > 0.0 {
                ((point.value.max(0.0) / max) * bar_space as f64).round() as usize
            } else {
                0
            };
            format!(
                "{} │{}{} {}",
                pad(&point.label, label_width),
                "█".repeat(length),
                " ".repeat(bar_space - length),
                pad(&number(point.value), value_width)
            )
            .trim_end()
            .to_string()
        })
        .collect()
}

/// Points of a line chart: the values in order, evenly spread.
fn line_points(data: &[DataPoint]) -> Vec<(f64, f64)> {
    data.iter()
        .enumerate()
        .map(|(i, point)| (i as f64, point.value))
        .collect()
}

/// Points of a scatter chart: at the label's number when every label is a number, otherwise in
/// order.
fn scatter_points(data: &[DataPoint]) -> Vec<(f64, f64)> {
    let numeric: Option<Vec<f64>> = data.iter().map(|p| p.label.parse::<f64>().ok()).collect();
    match numeric {
        Some(xs) => xs.into_iter().zip(data.iter().map(|p| p.value)).collect(),
        None => line_points(data),
    }
}

fn range(values: impl Iterator<Item = f64>) -> (f64, f64) {
    values.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
        (lo.min(v), hi.max(v))
    })
}

/// Where `value` falls among `cells` cells spanning `min..=max`; the middle cell when the span
/// is empty.
fn cell_of(value: f64, min: f64, max: f64, cells: usize) -> usize {
    if max > min {
        (((value - min) / (max - min)) * (cells - 1) as f64).round() as usize
    } else {
        (cells - 1) / 2
    }
}

/// A grid with a value axis on the left and the points marked with `mark`, joined in order by a
/// dotted line when `connect` is set, and the data labels under their points where they fit
/// without overlapping.
fn point_chart(
    points: &[(f64, f64)],
    data: &[DataPoint],
    width: usize,
    height: usize,
    mark: char,
    connect: bool,
) -> Vec<String> {
    let (min_y, max_y) = range(points.iter().map(|p| p.1));
    let (min_x, max_x) = range(points.iter().map(|p| p.0));
    let axis_width = number(min_y)
        .width()
        .max(number(max_y).width())
        .min(width / 3);
    // axis labels, " ┤", plot
    let plot_width = width - axis_width - 2;
    let plot_height = height - 1;
    let mut grid = vec![vec![' '; plot_width]; plot_height];
    let place = |x: f64, y: f64| {
        (
            plot_height - 1 - cell_of(y, min_y, max_y, plot_height),
            cell_of(x, min_x, max_x, plot_width),
        )
    };
    if connect {
        for pair in points.windows(2) {
            let ((x0, y0), (x1, y1)) = (pair[0], pair[1]);
            let (_, from) = place(x0, y0);
            let (_, to) = place(x1, y1);
            // Each column between the points covers the rows the line passes through within
            // it, so steep lines stay unbroken.
            let row_at = |column: f64| {
                let t = ((column - from as f64) / (to - from) as f64).clamp(0.0, 1.0);
                place(x0, y0 + (y1 - y0) * t).0
            };
            for column in from + 1..to {
                let (a, b) = (row_at(column as f64 - 0.5), row_at(column as f64 + 0.5));
                for line in &mut grid[a.min(b)..=a.max(b)] {
                    line[column] = '·';
                }
            }
        }
    }
    for &(x, y) in points {
        let (row, column) = place(x, y);
        grid[row][column] = mark;
    }
    let mut lines: Vec<String> = grid
        .into_iter()
        .enumerate()
        .map(|(row, cells)| {
            let label = if row == 0 {
                number(max_y)
            } else if row == plot_height - 1 {
                number(min_y)
            } else {
                String::new()
            };
            let label = pad(&label, axis_width);
            let label = format!("{:>w$}", label.trim_end(), w = axis_width);
            format!("{} ┤{}", label, cells.into_iter().collect::<String>())
                .trim_end()
                .to_string()
        })
        .collect();
    let mut axis = String::new();
    let mut used = 0;
    for (&(x, _), point) in points.iter().zip(data) {
        let label = pad(&point.label, point.label.width().min(6));
        let label_width = label.width();
        if label_width == 0 || label_width > plot_width {
            continue;
        }
        let start = cell_of(x, min_x, max_x, plot_width).min(plot_width - label_width);
        // One blank column between labels.
        if used > 0 && start <= used {
            continue;
        }
        axis.push_str(&" ".repeat(start - used));
        axis.push_str(&label);
        used = start + label_width;
    }
    lines.push(
        format!("{}  {}", " ".repeat(axis_width), axis)
            .trim_end()
            .to_string(),
    );
    lines
}

/// Counts of the values in equal ranges between the smallest and the largest, as vertical bars
/// with each count and each range's start under its bar.
fn histogram(data: &[DataPoint], width: usize, height: usize) -> Vec<String> {
    let (min, max) = range(data.iter().map(|p| p.value));
    let bins = (width / 6).clamp(1, 12).min(data.len());
    let span = if max > min { max - min } else { 1.0 };
    let mut counts = vec![0usize; bins];
    for point in data {
        let bin = (((point.value - min) / span) * bins as f64) as usize;
        counts[bin.min(bins - 1)] += 1;
    }
    // Each bin is a column of `column` cells: the bar and one blank.
    let column = width / bins;
    let bar_width = column - 1;
    let plot_height = height - 2;
    let most = counts.iter().copied().max().unwrap_or(1).max(1);
    let mut lines = Vec::new();
    for row in (0..plot_height).rev() {
        let line: String = counts
            .iter()
            .map(|&count| {
                let bar = (count as f64 / most as f64 * plot_height as f64).ceil() as usize;
                let cell = if row < bar { "█" } else { " " };
                format!("{} ", cell.repeat(bar_width))
            })
            .collect();
        lines.push(line.trim_end().to_string());
    }
    let row_of = |texts: Vec<String>| -> String {
        texts
            .iter()
            .map(|t| pad(t, column))
            .collect::<String>()
            .trim_end()
            .to_string()
    };
    lines.push(row_of(counts.iter().map(|c| c.to_string()).collect()));
    lines.push(row_of(
        (0..bins)
            .map(|i| number(min + span * i as f64 / bins as f64))
            .collect(),
    ));
    lines
}

/// Glyphs telling the slices of a pie apart; data beyond them is left out.
const SLICE_GLYPHS: [char; 6] = ['█', '▓', '▒', '░', '●', '○'];

/// A filled circle whose slices show each value's share, with a legend beside it. Negative
/// values have no share.
fn pie_chart(data: &[DataPoint], width: usize, height: usize) -> Vec<String> {
    let shown = &data[..data.len().min(SLICE_GLYPHS.len())];
    let total: f64 = shown.iter().map(|p| p.value.max(0.0)).sum();
    if total <= 0.0 {
        return vec!["no positive values".to_string()];
    }
    // Cells are about twice as tall as wide, so the circle is twice as many columns as rows.
    // It takes at most half the width; the legend gets the rest.
    let rows = height.min(width / 4).max(2);
    let columns = rows * 2;
    let radius = rows as f64 / 2.0;
    let shares: Vec<f64> = shown.iter().map(|p| p.value.max(0.0) / total).collect();
    let slice_at = |turn: f64| -> usize {
        let mut end = 0.0;
        for (i, share) in shares.iter().enumerate() {
            end += share;
            if turn < end {
                return i;
            }
        }
        shares.len() - 1
    };
    let circle: Vec<String> = (0..rows)
        .map(|row| {
            let y = (row as f64 + 0.5 - radius) / radius;
            (0..columns)
                .map(|col| {
                    let x = (col as f64 + 0.5 - radius * 2.0) / (radius * 2.0);
                    if x * x + y * y > 1.0 {
                        ' '
                    } else {
                        // Clockwise from twelve o'clock, as a fraction of a full turn.
                        let turn = (x.atan2(-y) / std::f64::consts::TAU).rem_euclid(1.0);
                        SLICE_GLYPHS[slice_at(turn)]
                    }
                })
                .collect()
        })
        .collect();
    let legend_width = width - columns - 2;
    let legend: Vec<String> = shown
        .iter()
        .zip(&shares)
        .enumerate()
        .map(|(i, (point, share))| {
            let percent = format!(" {:.0}%", share * 100.0);
            let label_width = legend_width.saturating_sub(2 + percent.len());
            format!(
                "{} {}{}",
                SLICE_GLYPHS[i],
                pad(&point.label, point.label.width().min(label_width)),
                percent
            )
        })
        .collect();
    (0..rows.max(legend.len()).min(height))
        .map(|row| {
            let left = circle
                .get(row)
                .cloned()
                .unwrap_or_else(|| " ".repeat(columns));
            let right = legend.get(row).cloned().unwrap_or_default();
            format!("{}  {}", left, right).trim_end().to_string()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(pairs: &[(&str, f64)]) -> Vec<DataPoint> {
        pairs
            .iter()
            .map(|(label, value)| DataPoint {
                label: label.to_string(),
                value: *value,
            })
            .collect()
    }

    const ALL: [ChartType; 5] = [
        ChartType::Bar,
        ChartType::Line,
        ChartType::Histogram,
        ChartType::Pie,
        ChartType::Scatter,
    ];

    #[test]
    fn data_lines_accept_three_separators_and_reject_anything_else() {
        let parsed = parse_chart_data("# sales\nJan,10\nFeb: 12.5\nMar 7\n\n").unwrap();
        assert_eq!(parsed, data(&[("Jan", 10.0), ("Feb", 12.5), ("Mar", 7.0)]));
        let error = parse_chart_data("Jan,10\nFeb,lots\n").unwrap_err();
        assert!(
            error.contains("line 2") && error.contains("Feb,lots"),
            "{}",
            error
        );
        assert!(parse_chart_data("x,inf").is_err());
    }

    #[test]
    fn every_chart_fits_its_area() {
        let points = data(&[
            ("January and February", 10.0),
            ("Feb", 8.0),
            ("Mar", 15.0),
            ("Apr", 20.5),
            ("May", -3.0),
            ("Jun", 123456.25),
            ("日本", 4.0),
        ]);
        for kind in ALL {
            for (width, height) in [(20, 5), (21, 6), (33, 8), (48, 12), (80, 24), (200, 5)] {
                let text = chart_text(kind, &points, width, height);
                let lines: Vec<&str> = text.lines().collect();
                assert!(
                    lines.len() <= height,
                    "{:?} {}x{}:\n{}",
                    kind,
                    width,
                    height,
                    text
                );
                for line in lines {
                    assert!(
                        line.width() <= width,
                        "{:?} {}x{}: {:?} is {} wide",
                        kind,
                        width,
                        height,
                        line,
                        line.width()
                    );
                }
            }
        }
    }

    #[test]
    fn single_points_and_equal_values_draw() {
        for kind in ALL {
            for points in [data(&[("only", 5.0)]), data(&[("a", 2.0), ("b", 2.0)])] {
                let text = chart_text(kind, &points, 30, 8);
                assert!(
                    text.contains(|c: char| "█●◆▓".contains(c)),
                    "{:?}:\n{}",
                    kind,
                    text
                );
            }
        }
    }

    #[test]
    fn too_small_and_empty_charts_say_so() {
        let points = data(&[("a", 1.0)]);
        assert_eq!(chart_text(ChartType::Bar, &points, 19, 10), "too small");
        assert_eq!(chart_text(ChartType::Bar, &points, 30, 4), "too small");
        assert_eq!(chart_text(ChartType::Line, &[], 30, 10), "no chart data");
        assert_eq!(
            chart_text(ChartType::Pie, &data(&[("a", -1.0)]), 30, 10),
            "no positive values"
        );
    }

    #[test]
    fn bars_are_proportional_to_their_values() {
        let text = chart_text(ChartType::Bar, &data(&[("a", 10.0), ("b", 5.0)]), 30, 5);
        let bars: Vec<usize> = text.lines().map(|l| l.matches('█').count()).collect();
        assert_eq!(bars[0], bars[1] * 2, "{}", text);
        assert!(text.lines().next().unwrap().ends_with(" 10"), "{}", text);
    }

    #[test]
    fn line_points_rise_with_their_values() {
        let text = chart_text(ChartType::Line, &data(&[("a", 1.0), ("b", 5.0)]), 30, 6);
        let lines: Vec<&str> = text.lines().collect();
        // The larger value is on the top row, at the right; the smaller at the bottom, left.
        assert!(
            lines[0].starts_with("5 ┤") && lines[0].ends_with('●'),
            "{}",
            text
        );
        assert!(lines[4].starts_with("1 ┤●·"), "{}", text);
        assert!(
            lines[5].trim_start().starts_with('a') && lines[5].ends_with('b'),
            "{}",
            text
        );
    }

    #[test]
    fn a_histogram_counts_values_per_range() {
        // Five values near 1 and one at 10: the first range holds five, the last one.
        let values = data(&[
            ("", 1.0),
            ("", 1.2),
            ("", 1.1),
            ("", 1.3),
            ("", 1.4),
            ("", 10.0),
        ]);
        let text = chart_text(ChartType::Histogram, &values, 24, 8);
        let lines: Vec<&str> = text.lines().collect();
        let counts = lines[lines.len() - 2];
        assert!(counts.starts_with('5') && counts.ends_with('1'), "{}", text);
        assert!(lines[lines.len() - 1].starts_with('1'), "{}", text);
        // The tallest bar reaches the top row.
        assert!(lines[0].starts_with('█'), "{}", text);
    }

    #[test]
    fn pie_slices_follow_the_shares_and_the_legend_lists_them() {
        let text = chart_text(
            ChartType::Pie,
            &data(&[("big", 3.0), ("small", 1.0)]),
            40,
            10,
        );
        let big = text.matches('█').count() as f64 - 1.0;
        let small = text.matches('▓').count() as f64 - 1.0;
        assert!((big / (big + small) - 0.75).abs() < 0.1, "{}", text);
        assert!(
            text.contains("█ big 75%") && text.contains("▓ small 25%"),
            "{}",
            text
        );
    }

    #[test]
    fn scatter_places_points_at_numeric_labels() {
        // x = 0 and x = 10 are at the edges; x = 1 is near the left, not in the middle.
        let text = chart_text(
            ChartType::Scatter,
            &data(&[("0", 1.0), ("1", 2.0), ("10", 3.0)]),
            40,
            6,
        );
        let middle = text
            .lines()
            .find(|l| l.contains('◆') && !l.contains("3 ┤") && !l.contains("1 ┤"))
            .expect("the middle point has a row of its own");
        let axis = middle.find('┤').unwrap();
        let point = middle.find('◆').unwrap();
        assert!(point - axis < 10, "{}", text);
    }

    #[test]
    fn chart_types_are_written_in_lowercase() {
        for (name, kind) in ["bar", "line", "histogram", "pie", "scatter"]
            .into_iter()
            .zip(ALL)
        {
            assert_eq!(serde_yaml::from_str::<ChartType>(name).unwrap(), kind);
        }
        assert!(serde_yaml::from_str::<ChartType>("Bar").is_err());
    }
}
