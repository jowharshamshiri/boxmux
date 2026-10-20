//! Tables drawn as text from CSV or JSON data, filtered, sorted and paged as a box's
//! `table_config` says.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use unicode_width::UnicodeWidthChar;

/// A table: column headers and rows of cells.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

/// How a table's borders are drawn.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TableBorderStyle {
    None,
    Single,
    Double,
    Rounded,
    Thick,
}

/// The `table_config` of a box: how its table is sorted, filtered, paged and drawn.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct TableSettings {
    /// Whether the header row is shown (default: true).
    pub show_headers: Option<bool>,
    /// Header of the column rows are sorted by.
    pub sort_column: Option<String>,
    /// Default: true.
    pub sort_ascending: Option<bool>,
    /// Only rows whose value in the named column equals the given text, ignoring case, are
    /// shown.
    pub filters: Option<BTreeMap<String, String>>,
    /// Default: single.
    pub border_style: Option<TableBorderStyle>,
    /// Shades every other row.
    pub zebra_striping: Option<bool>,
    /// Columns wider than this many cells are cut off with `…` (default: 30).
    pub max_column_width: Option<usize>,
    /// A first column numbering the rows (numbering continues across pages).
    pub show_row_numbers: Option<bool>,
    /// Rows per page; the table is not paged without it.
    pub page_size: Option<usize>,
    /// The page shown, from 0.
    pub current_page: Option<usize>,
    /// A line under the table saying which page is shown.
    pub show_page_info: Option<bool>,
}

const DEFAULT_MAX_COLUMN_WIDTH: usize = 30;
/// A column is never narrower than this, so a cut-off value still shows something.
const MIN_COLUMN_WIDTH: usize = 3;
/// SGR sequences shading a zebra row and ending the shading (back to the box background).
const SHADE_ON: &str = "\x1b[100m";
const SHADE_OFF: &str = "\x1b[49m";

/// The table text for `data` (CSV, or a JSON array of objects).
pub fn table_text(data: &str, settings: &TableSettings) -> String {
    let parsed = if data.trim_start().starts_with('[') {
        match parse_table_data_from_json(data) {
            Ok(parsed) => parsed,
            Err(e) => return format!("table data is not valid JSON: {}", e),
        }
    } else {
        parse_table_data(data, ',')
    };
    render_table(&parsed, settings)
}

/// Parses CSV text: the first line holds the headers, every other non-empty line a row.
/// Values are trimmed; quoting is not supported.
pub fn parse_table_data(content: &str, delimiter: char) -> TableData {
    let split = |line: &str| -> Vec<String> {
        line.split(delimiter)
            .map(|s| s.trim().to_string())
            .collect()
    };
    let mut lines = content.lines().filter(|line| !line.trim().is_empty());
    let headers = lines.next().map(split).unwrap_or_default();
    let rows = lines.map(split).collect();
    TableData { headers, rows }
}

/// Parses a JSON array of objects. The columns are the keys of the first object, in the order
/// they are written; a missing value is empty.
pub fn parse_table_data_from_json(content: &str) -> Result<TableData, String> {
    let objects: Vec<serde_json::Map<String, serde_json::Value>> = serde_json::from_str(content)
        .map_err(|e| format!("{} (expected an array of objects)", e))?;
    let headers: Vec<String> = objects
        .first()
        .map(|first| first.keys().cloned().collect())
        .unwrap_or_default();
    let text = |value: &serde_json::Value| match value {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    };
    let rows = objects
        .iter()
        .map(|object| {
            headers
                .iter()
                .map(|header| object.get(header).map(text).unwrap_or_default())
                .collect()
        })
        .collect();
    Ok(TableData { headers, rows })
}

fn text_width(text: &str) -> usize {
    text.chars().map(|c| c.width().unwrap_or(0)).sum()
}

/// `text` padded with spaces to `width` columns, or cut to `width` columns ending in `…`.
fn fit(text: &str, width: usize) -> String {
    let full = text_width(text);
    if full <= width {
        return format!("{}{}", text, " ".repeat(width - full));
    }
    let mut out = String::new();
    let mut used = 0;
    for ch in text.chars() {
        let w = ch.width().unwrap_or(0);
        if used + w > width.saturating_sub(1) {
            break;
        }
        out.push(ch);
        used += w;
    }
    out.push('…');
    used += 1;
    out.push_str(&" ".repeat(width.saturating_sub(used)));
    out
}

/// Characters of one border style: the left end, joint and right end of the top, separator
/// and bottom lines, their horizontal line, and the vertical line.
struct Glyphs {
    top: [char; 3],
    separator: [char; 3],
    bottom: [char; 3],
    horizontal: char,
    vertical: char,
}

fn glyphs(style: TableBorderStyle) -> Option<Glyphs> {
    let (top, separator, bottom, horizontal, vertical) = match style {
        TableBorderStyle::None => return None,
        TableBorderStyle::Single => (['┌', '┬', '┐'], ['├', '┼', '┤'], ['└', '┴', '┘'], '─', '│'),
        TableBorderStyle::Double => (['╔', '╦', '╗'], ['╠', '╬', '╣'], ['╚', '╩', '╝'], '═', '║'),
        TableBorderStyle::Rounded => (['╭', '┬', '╮'], ['├', '┼', '┤'], ['╰', '┴', '╯'], '─', '│'),
        TableBorderStyle::Thick => (['┏', '┳', '┓'], ['┣', '╋', '┫'], ['┗', '┻', '┛'], '━', '┃'),
    };
    Some(Glyphs {
        top,
        separator,
        bottom,
        horizontal,
        vertical,
    })
}

/// A horizontal border line across columns of `widths` (each with one space of padding on
/// both sides).
fn border_line(widths: &[usize], [left, joint, right]: [char; 3], horizontal: char) -> String {
    let segments: Vec<String> = widths
        .iter()
        .map(|w| horizontal.to_string().repeat(w + 2))
        .collect();
    format!("{}{}{}", left, segments.join(&joint.to_string()), right)
}

/// A row of cells, each already fitted to its column.
fn cell_line(cells: &[String], vertical: Option<char>) -> String {
    match vertical {
        Some(v) => format!("{v} {} {v}", cells.join(&format!(" {} ", v))),
        None => cells.join("  "),
    }
}

/// The table as lines of text, with SGR shading for zebra rows.
pub fn render_table(data: &TableData, settings: &TableSettings) -> String {
    if data.headers.is_empty() {
        return "No data".to_string();
    }
    let columns = data.headers.len();
    // Every row has exactly one cell per header.
    let normalized: Vec<Vec<String>> = data
        .rows
        .iter()
        .map(|row| {
            let mut row: Vec<String> = row.iter().take(columns).cloned().collect();
            row.resize(columns, String::new());
            row
        })
        .collect();

    let filtered = apply_filters(&data.headers, normalized, settings.filters.as_ref());
    let sorted = apply_sorting(
        &data.headers,
        filtered,
        settings.sort_column.as_deref(),
        settings.sort_ascending.unwrap_or(true),
    );
    let total_rows = sorted.len();
    let (first_row, page) = match settings.page_size {
        Some(size) => {
            let start = settings.current_page.unwrap_or(0).saturating_mul(size);
            let page: Vec<Vec<String>> = sorted.into_iter().skip(start).take(size).collect();
            (start, page)
        }
        None => (0, sorted),
    };

    // Row numbers form the first column when shown.
    let show_headers = settings.show_headers.unwrap_or(true);
    let mut headers: Vec<String> = data.headers.clone();
    let mut rows: Vec<Vec<String>> = page;
    if settings.show_row_numbers.unwrap_or(false) {
        headers.insert(0, "#".to_string());
        for (i, row) in rows.iter_mut().enumerate() {
            row.insert(0, (first_row + i + 1).to_string());
        }
    }

    let max_width = settings
        .max_column_width
        .unwrap_or(DEFAULT_MAX_COLUMN_WIDTH)
        .max(MIN_COLUMN_WIDTH);
    let widths: Vec<usize> = (0..headers.len())
        .map(|i| {
            let header_width = if show_headers {
                text_width(&headers[i])
            } else {
                0
            };
            rows.iter()
                .map(|row| text_width(&row[i]))
                .chain(std::iter::once(header_width))
                .max()
                .unwrap_or(0)
                .clamp(MIN_COLUMN_WIDTH, max_width)
        })
        .collect();
    let fitted = |row: &[String]| -> Vec<String> {
        row.iter()
            .zip(&widths)
            .map(|(cell, w)| fit(cell, *w))
            .collect()
    };

    let glyphs = glyphs(settings.border_style.unwrap_or(TableBorderStyle::Single));
    let vertical = glyphs.as_ref().map(|g| g.vertical);
    let mut lines = Vec::new();
    if let Some(g) = &glyphs {
        lines.push(border_line(&widths, g.top, g.horizontal));
    }
    if show_headers {
        lines.push(cell_line(&fitted(&headers), vertical));
        if let Some(g) = &glyphs {
            lines.push(border_line(&widths, g.separator, g.horizontal));
        }
    }
    let zebra = settings.zebra_striping.unwrap_or(false);
    for (i, row) in rows.iter().enumerate() {
        let line = cell_line(&fitted(row), vertical);
        if zebra && i % 2 == 1 {
            lines.push(format!("{}{}{}", SHADE_ON, line, SHADE_OFF));
        } else {
            lines.push(line);
        }
    }
    if let Some(g) = &glyphs {
        lines.push(border_line(&widths, g.bottom, g.horizontal));
    }
    if let (Some(size), true) = (settings.page_size, settings.show_page_info.unwrap_or(false)) {
        let pages = total_rows.div_ceil(size.max(1)).max(1);
        lines.push(format!(
            "Page {} of {} ({} rows)",
            settings.current_page.unwrap_or(0) + 1,
            pages,
            total_rows
        ));
    }
    lines.join("\n")
}

fn column_of(headers: &[String], name: &str) -> Option<usize> {
    headers.iter().position(|h| h == name)
}

/// The rows whose value in each filtered column equals the filter text, ignoring case. A
/// filter on a column the data does not have keeps no rows.
fn apply_filters(
    headers: &[String],
    rows: Vec<Vec<String>>,
    filters: Option<&BTreeMap<String, String>>,
) -> Vec<Vec<String>> {
    let Some(filters) = filters.filter(|f| !f.is_empty()) else {
        return rows;
    };
    let wanted: Vec<(Option<usize>, String)> = filters
        .iter()
        .map(|(column, value)| (column_of(headers, column), value.to_lowercase()))
        .collect();
    rows.into_iter()
        .filter(|row| {
            wanted
                .iter()
                .all(|(column, value)| column.is_some_and(|c| row[c].to_lowercase() == *value))
        })
        .collect()
}

/// Rows sorted by `column`: numerically when both values are numbers, otherwise as text. An
/// unknown column leaves the order unchanged.
fn apply_sorting(
    headers: &[String],
    mut rows: Vec<Vec<String>>,
    column: Option<&str>,
    ascending: bool,
) -> Vec<Vec<String>> {
    let Some(index) = column.and_then(|name| column_of(headers, name)) else {
        return rows;
    };
    rows.sort_by(|a, b| {
        let order = match (a[index].parse::<f64>(), b[index].parse::<f64>()) {
            (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
            _ => a[index].cmp(&b[index]),
        };
        if ascending {
            order
        } else {
            order.reverse()
        }
    });
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(yaml: &str) -> TableSettings {
        crate::yaml::from_str(yaml).unwrap()
    }

    #[test]
    fn csv_rows_are_trimmed_and_blank_lines_skipped() {
        let table = parse_table_data("Name, Age\n\nAlice, 30\nBob,25\n", ',');
        assert_eq!(table.headers, vec!["Name", "Age"]);
        assert_eq!(table.rows, vec![vec!["Alice", "30"], vec!["Bob", "25"]]);
    }

    #[test]
    fn json_columns_keep_the_order_of_the_first_object() {
        let table =
            parse_table_data_from_json(r#"[{"name":"a","id":1,"ok":true},{"id":2}]"#).unwrap();
        assert_eq!(table.headers, vec!["name", "id", "ok"]);
        assert_eq!(table.rows[0], vec!["a", "1", "true"]);
        assert_eq!(table.rows[1], vec!["", "2", ""]);
        assert!(parse_table_data_from_json(r#"{"a": 1}"#).is_err());
    }

    #[test]
    fn every_line_of_a_table_has_the_same_width() {
        let data = parse_table_data("Name,City\nZoë,Zürich\n日本語の名前,東京\nBob,Paris\n", ',');
        for yaml in [
            "{}",
            "show_row_numbers: true",
            "border_style: double\nshow_row_numbers: true",
            "border_style: none",
            "max_column_width: 4",
        ] {
            let text = render_table(&data, &settings(yaml));
            let widths: Vec<usize> = text.lines().map(crate::utils::display_width).collect();
            assert!(
                widths.windows(2).all(|w| w[0] == w[1]),
                "{}:\n{}\n{:?}",
                yaml,
                text,
                widths
            );
        }
    }

    #[test]
    fn long_values_are_cut_at_character_boundaries() {
        // Multi-byte text longer than the column used to be sliced by bytes and panic.
        let data = parse_table_data("Text\nÄÖÜäöüß日本語テキスト\n", ',');
        let text = render_table(&data, &settings("max_column_width: 5"));
        assert!(text.contains("ÄÖÜä…"), "{}", text);
        let text = render_table(&data, &settings("max_column_width: 4"));
        assert!(text.contains("ÄÖÜ…"), "{}", text);
        let data = parse_table_data("Text\n日本語テキスト\n", ',');
        let text = render_table(&data, &settings("max_column_width: 4"));
        // Wide characters take two columns: one fits, then the ellipsis, then padding.
        assert!(text.contains("│ 日…  │"), "{}", text);
    }

    #[test]
    fn row_numbers_continue_across_pages() {
        let data = parse_table_data("N\na\nb\nc\nd\ne\n", ',');
        let text = render_table(
            &data,
            &settings(
                "show_row_numbers: true\npage_size: 2\ncurrent_page: 1\nshow_page_info: true",
            ),
        );
        assert!(text.contains("│ 3   │ c   │"), "{}", text);
        assert!(text.contains("│ 4   │ d   │"), "{}", text);
        assert!(!text.contains(" a "), "{}", text);
        assert!(text.ends_with("Page 2 of 3 (5 rows)"), "{}", text);
    }

    #[test]
    fn zebra_striping_shades_every_other_row() {
        let data = parse_table_data("N\na\nb\nc\n", ',');
        let text = render_table(&data, &settings("zebra_striping: true\nborder_style: none"));
        let lines: Vec<&str> = text.lines().collect();
        assert!(!lines[1].contains(SHADE_ON));
        assert!(lines[2].starts_with(SHADE_ON) && lines[2].ends_with(SHADE_OFF));
        assert!(!lines[3].contains(SHADE_ON));
    }

    #[test]
    fn filters_match_whole_values_ignoring_case_and_unknown_columns_match_nothing() {
        let data = parse_table_data("Name,Dept\nA,Eng\nB,engineering\nC,ENG\n", ',');
        let text = render_table(&data, &settings("filters: {Dept: eng}"));
        assert!(
            text.contains(" A ") && text.contains(" C ") && !text.contains(" B "),
            "{}",
            text
        );
        let text = render_table(&data, &settings("filters: {Team: x}"));
        assert!(!text.contains(" A "), "{}", text);
    }

    #[test]
    fn sorting_is_numeric_for_numbers_and_textual_otherwise() {
        let data = parse_table_data("Name,Age\nCarol,9\nAlice,30\nBob,100\n", ',');
        let text = render_table(&data, &settings("sort_column: Age\nsort_ascending: false"));
        let bob = text.find("Bob").unwrap();
        let alice = text.find("Alice").unwrap();
        let carol = text.find("Carol").unwrap();
        assert!(bob < alice && alice < carol, "{}", text);
        let text = render_table(&data, &settings("sort_column: Name"));
        assert!(text.find("Alice").unwrap() < text.find("Bob").unwrap());
    }

    #[test]
    fn short_and_long_rows_fit_the_headers() {
        let data = parse_table_data("A,B\n1\n2,3,4\n", ',');
        let text = render_table(&data, &settings("{}"));
        assert!(!text.contains('4'), "extra cells are dropped: {}", text);
        assert_eq!(text.lines().count(), 6, "{}", text);
    }
}
