use std::{collections::HashMap, io::Write};

use crate::{model::common::{Bounds, Cell, InputBounds, ScreenBuffer}, AppContext, Layout, Panel};
use termion::{color, raw::RawTerminal, screen::AlternateScreen};
use uuid::Uuid;

pub fn get_fg_color(color: &str) -> String {
    //log::debug!("Getting foreground color '{}'", color);
    match color {
        "red" => format!("{}", color::Fg(color::Red)),
        "green" => format!("{}", color::Fg(color::Green)),
        "yellow" => format!("{}", color::Fg(color::Yellow)),
        "blue" => format!("{}", color::Fg(color::Blue)),
        "magenta" => format!("{}", color::Fg(color::Magenta)),
        "cyan" => format!("{}", color::Fg(color::Cyan)),
        "white" => format!("{}", color::Fg(color::White)),
        "black" => format!("{}", color::Fg(color::Black)),
        "reset" => format!("{}", color::Fg(color::Reset)),
        "bright_black" => format!("{}", color::Fg(color::AnsiValue(8))),
        "bright_red" => format!("{}", color::Fg(color::AnsiValue(9))),
        "bright_green" => format!("{}", color::Fg(color::AnsiValue(10))),
        "bright_yellow" => format!("{}", color::Fg(color::AnsiValue(11))),
        "bright_blue" => format!("{}", color::Fg(color::AnsiValue(12))),
        "bright_magenta" => format!("{}", color::Fg(color::AnsiValue(13))),
        "bright_cyan" => format!("{}", color::Fg(color::AnsiValue(14))),
        "bright_white" => format!("{}", color::Fg(color::AnsiValue(15))),
        "default" => format!("{}", color::Fg(color::White)),
        _ => format!("{}", color::Fg(color::Reset)),
    }
}

pub fn get_bg_color(color: &str) -> String {
    //log::debug!("Getting background color '{}'", color);
    match color {
        "red" => format!("{}", color::Bg(color::Red)),
        "green" => format!("{}", color::Bg(color::Green)),
        "yellow" => format!("{}", color::Bg(color::Yellow)),
        "blue" => format!("{}", color::Bg(color::Blue)),
        "magenta" => format!("{}", color::Bg(color::Magenta)),
        "cyan" => format!("{}", color::Bg(color::Cyan)),
        "white" => format!("{}", color::Bg(color::White)),
        "black" => format!("{}", color::Bg(color::Black)),
        "reset" => format!("{}", color::Bg(color::Reset)),
        "bright_black" => format!("{}", color::Bg(color::AnsiValue(8))),
        "bright_red" => format!("{}", color::Bg(color::AnsiValue(9))),
        "bright_green" => format!("{}", color::Bg(color::AnsiValue(10))),
        "bright_yellow" => format!("{}", color::Bg(color::AnsiValue(11))),
        "bright_blue" => format!("{}", color::Bg(color::AnsiValue(12))),
        "bright_magenta" => format!("{}", color::Bg(color::AnsiValue(13))),
        "bright_cyan" => format!("{}", color::Bg(color::AnsiValue(14))),
        "bright_white" => format!("{}", color::Bg(color::AnsiValue(15))),
        "default" => format!("{}", color::Bg(color::AnsiValue(8))),
        _ => format!("{}", color::Bg(color::Reset)),
    }
}

pub fn print_with_color_at(y: usize, x: usize, color: &str, text: &str, buffer: &mut ScreenBuffer) {
    //log::debug!(
    //     "Printing text '{}' with color '{}' at x='{}', y='{}'",
    //     text,
    //     color,
    //     x,
    //     y
    // );
    let color_code = get_fg_color(color);
    for (i, ch) in text.chars().enumerate() {
        let cell = Cell {
            fg_color: color_code.clone(),
            bg_color: get_bg_color("default"),
            ch,
        };
        buffer.update(x + i, y, cell);
    }
}

pub fn print_with_color_and_background_at(
    y: usize,
    x: usize,
    fg_color: &str,
    bg_color: &str,
    text: &str,
    buffer: &mut ScreenBuffer,
) {
    //log::debug!(
    //     "Printing text '{}' with foreground color '{}' and background color '{}' at x='{}', y='{}'",
    //     text,
    //     fg_color,
    //     bg_color,
    //     x,
    //     y
    // );
    let fg_color_code = get_fg_color(fg_color);
    let bg_color_code = get_bg_color(bg_color);
    for (i, ch) in text.chars().enumerate() {
        let cell = Cell {
            fg_color: fg_color_code.clone(),
            bg_color: bg_color_code.clone(),
            ch,
        };
        buffer.update(x + i, y, cell);
    }
}

pub fn draw_horizontal_line(
    y: usize,
    x1: usize,
    x2: usize,
    border_color: &str,
    bg_color: &str,
    buffer: &mut ScreenBuffer,
) {
    //log::debug!(
    //     "Drawing horizontal line with border color '{}', background color '{}', x1='{}', x2='{}', y='{}'",
    //     border_color,
    //     bg_color,
    //     x1,
    //     x2,
    //     y
    // );
    let border_color_code = get_fg_color(border_color);
    let bg_color_code = get_bg_color(bg_color);
    for x in x1..=x2 {
        let cell = Cell {
            fg_color: border_color_code.clone(),
            bg_color: bg_color_code.clone(),
            ch: '─',
        };
        buffer.update(x, y, cell);
    }
}

pub fn draw_vertical_line(
    x: usize,
    y1: usize,
    y2: usize,
    border_color: &str,
    bg_color: &str,
    buffer: &mut ScreenBuffer,
) {
    //log::debug!(
    //     "Drawing vertical line with border color '{}', background color '{}', x='{}', y1='{}', y2='{}'",
    //     border_color,
    //     bg_color,
    //     x,
    //     y1,
    //     y2
    // );
    let border_color_code = get_fg_color(border_color);
    let bg_color_code = get_bg_color(bg_color);
    for y in y1..=y2 {
        let cell = Cell {
            fg_color: border_color_code.clone(),
            bg_color: bg_color_code.clone(),
            ch: '│',
        };
        buffer.update(x, y, cell);
    }
}

pub fn draw_horizontal_line_with_title(
    y: usize,
    x1: usize,
    x2: usize,
    fg_color: &str,
    bg_color: &str,
    title: Option<&str>,
    title_fg_color: &str,
    title_bg_color: &str,
    title_position: &str,
    buffer: &mut ScreenBuffer,
) {
    //log::debug!(
    //     "Drawing horizontal line with title '{:?}', foreground color '{}', background color '{}', title foreground color '{}', title background color '{}', title position '{}', x1='{}', x2='{}', y='{}'",
    //     title,
    //     fg_color,
    //     bg_color,
    //     title_fg_color,
    //     title_bg_color,
    //     title_position,
    //     x1,
    //     x2,
    //     y
    // );
    let width = x2.saturating_sub(x1);
    let title_padding = 2; // Adjust padding if needed

    if let Some(title) = title {
        let formatted_title = format!(" {} ", title);
        let title_length = formatted_title.len();

        if title_length <= width {
            let (title_start_position, line_before_title_length, line_after_title_length) =
                match title_position {
                    "start" => {
                        let title_start_position = x1 + title_padding;
                        let line_before_title_length = title_start_position.saturating_sub(x1);
                        let line_after_title_length =
                            width.saturating_sub(line_before_title_length + title_length);
                        (
                            title_start_position,
                            line_before_title_length,
                            line_after_title_length,
                        )
                    }
                    "center" => {
                        let title_start_position = x1 + (width.saturating_sub(title_length)) / 2;
                        let line_before_title_length = title_start_position.saturating_sub(x1);
                        let line_after_title_length =
                            width.saturating_sub(line_before_title_length + title_length);
                        (
                            title_start_position,
                            line_before_title_length,
                            line_after_title_length,
                        )
                    }
                    "end" => {
                        let title_start_position = x2.saturating_sub(title_length + title_padding);
                        let line_before_title_length = title_start_position.saturating_sub(x1);
                        let line_after_title_length =
                            x2.saturating_sub(title_start_position + title_length);
                        (
                            title_start_position,
                            line_before_title_length,
                            line_after_title_length,
                        )
                    }
                    _ => (x1, width, 0), // Default to no title
                };

            draw_horizontal_line(
                y,
                x1,
                x1 + line_before_title_length,
                fg_color,
                bg_color,
                buffer,
            );
            print_with_color_and_background_at(
                y,
                title_start_position,
                title_fg_color,
                title_bg_color,
                &formatted_title,
                buffer,
            );
            draw_horizontal_line(
                y,
                x1 + line_before_title_length + title_length,
                x2,
                fg_color,
                bg_color,
                buffer,
            );
        } else {
            // If the title is too long, just draw a line without the title
            draw_horizontal_line(y, x1, x2, fg_color, bg_color, buffer);
        }
    } else {
        // If there is no title, just draw a full horizontal line
        draw_horizontal_line(y, x1, x2, fg_color, bg_color, buffer);
    }
}

static H_SCROLL_CHAR: &str = "|";
static V_SCROLL_CHAR: &str = "-";

pub fn draw_panel(
    bounds: &Bounds,
    border_color: &str,
    bg_color: Option<&str>,
    parent_bg_color: Option<&str>,
    title: Option<&str>,
    title_fg_color: &str,
    title_bg_color: &str,
    title_position: &str,
    content: Option<&str>,
    fg_color: &str,
    overflow_behavior: &str,
    horizontal_scroll: f64,
    vertical_scroll: f64,
    buffer: &mut ScreenBuffer,
) {
    //log::debug!(
    //     "Drawing panel with bounds '{:?}', border color '{}', background color '{}', parent background color '{}', title '{}', title fg color '{}', title bg color '{}', title position '{}', content '{:?}', fg color '{}', overflow behavior '{}', horizontal scroll '{}', vertical scroll '{}'",
    //     bounds,
    //     border_color,
    //     bg_color.unwrap_or("default"),
    //     parent_bg_color.unwrap_or("default"),
    //     title.unwrap_or("None"),
    //     title_fg_color,
    //     title_bg_color,
    //     title_position,
    //     content,
    //     fg_color,
    //     overflow_behavior,
    //     horizontal_scroll,
    //     vertical_scroll
    // );
    let border_color_code = get_fg_color(border_color);
    let title_fg_color_code = get_fg_color(title_fg_color);
    let title_bg_color_code = get_bg_color(title_bg_color);
    let fg_color_code = get_fg_color(fg_color);
    let bg_color_code = get_bg_color(bg_color.unwrap_or("default"));
    let parent_bg_color_code = get_bg_color(parent_bg_color.unwrap_or("default"));
    let mut _title_overflowing = false;
    let mut _x_offset = 0;
    let mut _y_offset = 0;
    let mut _overflowing = false;
    let mut horizontal_scrollbar_position = 0;
    let mut vertical_scrollbar_position = 0;

    //log::debug!(
    //     "Drawing panel with title '{}', background color '{}', background color code '{}'",
    //     title.unwrap_or("None"),
    //     bg_color.unwrap_or("default"),
    //     bg_color_code
    // );

    // Ensure bounds stay within screen limits
    let screen_bounds = screen_bounds();
    let bounds = bounds
        .intersection(&screen_bounds)
        .unwrap_or_else(|| bounds.clone());

    // Draw top border with title
    let top_border_length = bounds.width();
    if let Some(title) = title {
        let mut formatted_title = format!(" {} ", title);
        let title_length = formatted_title.len();
        let mut max_title_length = title_length;

        if title_length < top_border_length {
            if top_border_length < 5 {
                _title_overflowing = true;
            } else {
                if max_title_length > top_border_length.saturating_sub(3) {
                    max_title_length = top_border_length.saturating_sub(3);
                }

                if title_length > max_title_length {
                    formatted_title = format!("{}...", &formatted_title[..max_title_length]);
                }
            }
        }

        if !_title_overflowing {
            draw_horizontal_line_with_title(
                bounds.top(),
                bounds.left(),
                bounds.right(),
                border_color,
                bg_color.unwrap_or("default"),
                Some(&formatted_title),
                title_fg_color,
                title_bg_color,
                title_position,
                buffer,
            );
        }
    } else {
        draw_horizontal_line(
            bounds.top(),
            bounds.left(),
            bounds.right(),
            border_color,
            bg_color.unwrap_or("default"),
            buffer,
        );
    }

    if let Some(content) = content {
        let (content_width, content_height) = content_size(content);
        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        let content_lines: Vec<&str> = content.lines().collect();

        let max_content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        let max_content_height = content_lines.len();

        if content_width > viewable_width {
            _x_offset = (max_content_width as f64 * horizontal_scroll / 100.0).round() as usize;
        }

        if content_height > viewable_height {
            _y_offset = (max_content_height as f64 * vertical_scroll / 100.0).round() as usize;
        }

        _overflowing = content_width > viewable_width || content_height > viewable_height;

        if _overflowing && overflow_behavior == "scroll" {
            for (i, line) in content_lines
                .iter()
                .enumerate()
                .skip(_y_offset)
                .take(viewable_height)
            {
                let visible_line = &line
                    .chars()
                    .skip(_x_offset)
                    .take(viewable_width)
                    .collect::<String>();
                print_with_color_at(
                    bounds.top() + 2 + i,
                    bounds.left() + 2,
                    fg_color,
                    visible_line,
                    buffer,
                );
            }

            // Draw vertical scrollbar
            if content_height > viewable_height {
                let scrollbar_height = bounds.height().saturating_sub(4);
                let scroll_ratio =
                    vertical_scroll as f64 / (content_height - viewable_height) as f64;
                vertical_scrollbar_position =
                    (scroll_ratio * scrollbar_height as f64).round() as usize;

                // for i in 0..scrollbar_height {
                //     let ch = if i == scrollbar_position { '█' } else { ' ' };
                //     buffer.update(
                //         bounds.right() - 1,
                //         bounds.top() + 2 + i,
                //         Cell {
                //             fg_color: border_color_code.clone(),
                //             bg_color: bg_color_code.clone(),
                //             ch,
                //         },
                //     );
                // }
            }

            // Draw horizontal scrollbar
            if content_width > viewable_width {
                let scrollbar_width = bounds.width().saturating_sub(4);
                let scroll_ratio =
                    horizontal_scroll as f64 / (content_width - viewable_width) as f64;
                horizontal_scrollbar_position =
                    (scroll_ratio * scrollbar_width as f64).round() as usize;

                // for i in 0..scrollbar_width {
                //     let ch = if i == scrollbar_position { '█' } else { ' ' };
                //     buffer.update(
                //         bounds.left() + 2 + i,
                //         bounds.bottom() - 1,
                //         Cell {
                //             fg_color: border_color_code.clone(),
                //             bg_color: bg_color_code.clone(),
                //             ch,
                //         },
                //     );
                // }
            }
        } else if !_overflowing {
            for (i, line) in content_lines.iter().enumerate().take(viewable_height) {
                let visible_line = &line
                    .chars()
                    .skip(_x_offset)
                    .take(viewable_width)
                    .collect::<String>();
                print_with_color_at(
                    bounds.top() + 2 + i,
                    bounds.left() + 2,
                    fg_color,
                    visible_line,
                    buffer,
                );
            }
        }
    }

    if _overflowing && overflow_behavior != "scroll" {
        if overflow_behavior == "fill" {
            fill_panel(&bounds, true, bg_color.unwrap_or("default"), '█', buffer);
        } else if overflow_behavior == "cross_out" {
            for i in 0..bounds.width() {
                let cell = Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: parent_bg_color_code.clone(),
                    ch: 'X',
                };
                buffer.update(bounds.left() + i, bounds.top() + i, cell);
            }
        } else if overflow_behavior == "removed" {
            fill_panel(
                &bounds,
                false,
                parent_bg_color.unwrap_or("default"),
                ' ',
                buffer,
            );
        }
    } else {
        draw_horizontal_line(
            bounds.bottom(),
            bounds.left(),
            bounds.right(),
            border_color,
            bg_color.unwrap_or("default"),
            buffer,
        );
        draw_vertical_line(
            bounds.left(),
            bounds.top() + 1,
            bounds.bottom() - 1,
            border_color,
            bg_color.unwrap_or("default"),
            buffer,
        );
        draw_vertical_line(
            bounds.right(),
            bounds.top() + 1,
            bounds.bottom() - 1,
            border_color,
            bg_color.unwrap_or("default"),
            buffer,
        );

        print_with_color_and_background_at(
            bounds.top() + 2 + vertical_scrollbar_position,
            bounds.right(),
            border_color,
            bg_color.unwrap_or("default"),
            V_SCROLL_CHAR,
            buffer,
        );

        print_with_color_and_background_at(
            bounds.bottom(),
            bounds.left() + 2 + horizontal_scrollbar_position,
            border_color,
            bg_color.unwrap_or("default"),
            H_SCROLL_CHAR,
            buffer,
        );

        // Draw corners
        buffer.update(
            bounds.left(),
            bounds.top(),
            Cell {
                fg_color: border_color_code.clone(),
                bg_color: bg_color_code.clone(),
                ch: '┌',
            },
        );
        buffer.update(
            bounds.right(),
            bounds.top(),
            Cell {
                fg_color: border_color_code.clone(),
                bg_color: bg_color_code.clone(),
                ch: '┐',
            },
        );
        buffer.update(
            bounds.left(),
            bounds.bottom(),
            Cell {
                fg_color: border_color_code.clone(),
                bg_color: bg_color_code.clone(),
                ch: '└',
            },
        );
        buffer.update(
            bounds.right(),
            bounds.bottom(),
            Cell {
                fg_color: border_color_code.clone(),
                bg_color: bg_color_code.clone(),
                ch: '┘',
            },
        );
    }
}

pub fn fill_panel(
    bounds: &Bounds,
    inside: bool,
    bg_color: &str,
    fill_char: char,
    buffer: &mut ScreenBuffer,
) {
    let fg_color_code = get_fg_color(bg_color);
    let bg_color_code = get_bg_color(bg_color);

    //log::debug!(
    //     "Filling panel with bounds '{:?}', inside '{}', background color '{}', fill char '{}'",
    //     bounds,
    //     inside,
    //     bg_color,
    //     fill_char
    // );

    let (top, bottom) = if inside {
        (bounds.top(), bounds.bottom())
    } else {
        (bounds.top(), bounds.bottom())
    };

    let (left, right) = if inside {
        (bounds.left(), bounds.right())
    } else {
        (bounds.left(), bounds.right())
    };

    for y in top..bottom {
        for x in left..right {
            let cell = Cell {
                fg_color: fg_color_code.clone(),
                bg_color: bg_color_code.clone(),
                ch: fill_char,
            };
            buffer.update(x, y, cell);
        }
    }
}

pub fn screen_width() -> usize {
    //log::debug!("Getting screen width");
    termion::terminal_size().unwrap().0 as usize
}

pub fn screen_height() -> usize {
    //log::debug!("Getting screen height");
    termion::terminal_size().unwrap().1 as usize
}

pub fn screen_bounds() -> Bounds {
    //log::debug!("Getting screen bounds");
    Bounds {
        x1: 0,
        y1: 0,
        x2: screen_width(),
        y2: screen_height(),
    }
}

pub fn input_bounds_to_bounds(input_bounds: &InputBounds, parent_bounds: &Bounds) -> Bounds {
    //log::debug!(
    //     "Converting input bounds '{:?}' to absolute bounds with parent bounds '{:?}'",
    //     input_bounds,
    //     parent_bounds
    // );
    let bx1 = parse_percentage(&input_bounds.x1, parent_bounds.width());
    let by1 = parse_percentage(&input_bounds.y1, parent_bounds.height());
    let bx2 = parse_percentage(&input_bounds.x2, parent_bounds.width());
    let by2 = parse_percentage(&input_bounds.y2, parent_bounds.height());
    let abs_x1 = parent_bounds.x1 + bx1;
    let abs_y1 = parent_bounds.y1 + by1;
    let abs_x2 = parent_bounds.x1 + bx2;
    let abs_y2 = parent_bounds.y1 + by2;
    Bounds {
        x1: abs_x1,
        y1: abs_y1,
        x2: abs_x2,
        y2: abs_y2,
    }
}

pub fn parse_percentage(value: &str, total: usize) -> usize {
    //log::debug!("Parsing percentage value '{}'", value);
    if value.ends_with('%') {
        let percentage = value.trim_end_matches('%').parse::<f64>().unwrap() / 100.0;
        (percentage * total as f64).round() as usize
    } else {
        value.parse::<usize>().unwrap()
    }
}

pub fn content_size(text: &str) -> (usize, usize) {
    //log::debug!("Getting content size for text '{}'", text);
    let mut width = 0;
    let mut height = 0;
    for line in text.lines() {
        width = width.max(line.len());
        height += 1;
    }
    (width, height)
}

pub fn inherit_string(
    child_value: Option<&String>,
    parent_value: Option<&String>,
    parent_layout_value: Option<&String>,
    default_value: &str,
) -> String {
    if let Some(value) = child_value {
        if !value.is_empty() {
            return value.clone();
        }
    }
    if let Some(value) = parent_value {
        if !value.is_empty() {
            return value.clone();
        }
    }
    if let Some(value) = parent_layout_value {
        if !value.is_empty() {
            return value.clone();
        }
    }
    default_value.to_string()
}

pub fn inherit_char(
    child_char: Option<&char>,
    parent_char: Option<&char>,
    parent_layout_char: Option<&char>,
    default_char: char,
) -> char {
    if let Some(&char) = child_char {
        return char;
    }
    if let Some(&char) = parent_char {
        return char;
    }
    if let Some(&char) = parent_layout_char {
        return char;
    }
    default_char
}

pub fn inherit_bool(
    child_bool: Option<&bool>,
    parent_bool: Option<&bool>,
    parent_layout_bool: Option<&bool>,
    default_bool: bool,
) -> bool {
    if let Some(bool) = child_bool {
        return *bool;
    }
    if let Some(bool) = parent_bool {
        return *bool;
    }
    if let Some(bool) = parent_layout_bool {
        return *bool;
    }
    default_bool
}

pub fn inherit_u64(
    child_value: Option<&u64>,
    parent_value: Option<&u64>,
    parent_layout_value: Option<&u64>,
    default_value: u64,
) -> u64 {
    if let Some(value) = child_value {
        return *value;
    }
    if let Some(value) = parent_value {
        return *value;
    }
    if let Some(value) = parent_layout_value {
        return *value;
    }
    default_value
}

pub fn inherit_i64(
    child_value: Option<&i64>,
    parent_value: Option<&i64>,
    parent_layout_value: Option<&i64>,
    default_value: i64,
) -> i64 {
    if let Some(value) = child_value {
        return *value;
    }
    if let Some(value) = parent_value {
        return *value;
    }
    if let Some(value) = parent_layout_value {
        return *value;
    }
    default_value
}

pub fn inherit_f64(
    child_value: Option<&f64>,
    parent_value: Option<&f64>,
    parent_layout_value: Option<&f64>,
    default_value: f64,
) -> f64 {
    if let Some(value) = child_value {
        return *value;
    }
    if let Some(value) = parent_value {
        return *value;
    }
    if let Some(value) = parent_layout_value {
        return *value;
    }
    default_value
}

pub fn inherit_optional_string(
    child_value: Option<&String>,
    parent_value: Option<&String>,
    parent_layout_value: Option<&String>,
    default_value: Option<String>,
) -> Option<String> {
    if let Some(value) = child_value {
        if !value.is_empty() {
            return Some(value.clone());
        }
    }
    if let Some(value) = parent_value {
        if !value.is_empty() {
            return Some(value.clone());
        }
    }
    if let Some(value) = parent_layout_value {
        if !value.is_empty() {
            return Some(value.clone());
        }
    }
    default_value
}

pub fn inherit_optional_char(
    child_char: Option<&char>,
    parent_char: Option<&char>,
    parent_layout_char: Option<&char>,
    default_char: Option<char>,
) -> Option<char> {
    if let Some(&char) = child_char {
        return Some(char);
    }
    if let Some(&char) = parent_char {
        return Some(char);
    }
    if let Some(&char) = parent_layout_char {
        return Some(char);
    }
    default_char
}

pub fn inherit_optional_bool(
    child_bool: Option<&bool>,
    parent_bool: Option<&bool>,
    parent_layout_bool: Option<&bool>,
    default_bool: Option<bool>,
) -> Option<bool> {
    if let Some(bool) = child_bool {
        return Some(*bool);
    }
    if let Some(bool) = parent_bool {
        return Some(*bool);
    }
    if let Some(bool) = parent_layout_bool {
        return Some(*bool);
    }
    default_bool
}

pub fn inherit_optional_u64(
    child_value: Option<&u64>,
    parent_value: Option<&u64>,
    parent_layout_value: Option<&u64>,
    default_value: Option<u64>,
) -> Option<u64> {
    if let Some(value) = child_value {
        return Some(*value);
    }
    if let Some(value) = parent_value {
        return Some(*value);
    }
    if let Some(value) = parent_layout_value {
        return Some(*value);
    }
    default_value
}

pub fn inherit_optional_i64(
    child_value: Option<&i64>,
    parent_value: Option<&i64>,
    parent_layout_value: Option<&i64>,
    default_value: Option<i64>,
) -> Option<i64> {
    if let Some(value) = child_value {
        return Some(*value);
    }
    if let Some(value) = parent_value {
        return Some(*value);
    }
    if let Some(value) = parent_layout_value {
        return Some(*value);
    }
    default_value
}

pub fn inherit_optional_f64(
    child_value: Option<&f64>,
    parent_value: Option<&f64>,
    parent_layout_value: Option<&f64>,
    default_value: Option<f64>,
) -> Option<f64> {
    if let Some(value) = child_value {
        return Some(*value);
    }
    if let Some(value) = parent_value {
        return Some(*value);
    }
    if let Some(value) = parent_layout_value {
        return Some(*value);
    }
    default_value
}

pub fn apply_buffer(
    screen_buffer: &mut ScreenBuffer,
    alternate_screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
) {
    for y in 0..screen_buffer.height {
        for x in 0..screen_buffer.width {
            if let Some(cell) = screen_buffer.get(x, y) {
                write!(
                    alternate_screen,
                    "{}{}{}{}",
                    termion::cursor::Goto((x + 1) as u16, (y + 1) as u16),
                    cell.bg_color,
                    cell.fg_color,
                    cell.ch
                )
                .unwrap();
            }
        }
    }
    alternate_screen.flush().unwrap();
}

pub fn apply_buffer_if_changed(
    previous_buffer: &ScreenBuffer,
    current_buffer: &ScreenBuffer,
    alternate_screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
) {
    for y in 0..current_buffer.height {
        for x in 0..current_buffer.width {
            if current_buffer.get(x, y) != previous_buffer.get(x, y) {
                if let Some(cell) = current_buffer.get(x, y) {
                    write!(
                        alternate_screen,
                        "{}{}{}{}",
                        termion::cursor::Goto((x + 1) as u16, (y + 1) as u16),
                        cell.bg_color,
                        cell.fg_color,
                        cell.ch
                    )
                    .unwrap();
                }
            }
        }
    }
    alternate_screen.flush().unwrap();
}

pub fn find_selected_panel_uuid(layout: &Layout) -> Option<String> {
	for panel in &layout.children {
		if Some(true) == panel.selected {
			return Some(panel.id.clone());
		}
	}

    None
}

pub fn calculate_tab_order(layout: &Layout) -> Vec<String> {
	let mut result:HashMap<String, i32>= HashMap::new();

	for panel in &layout.children {
		let tab_order=panel.tab_order.clone();
		if tab_order.is_some() {
            result.insert(panel.id.clone(), tab_order.unwrap().parse::<i32>().expect("Invalid tab order"));
		}
	}

	// Sort the hashmap by value
	let mut sorted_result: Vec<(String, i32)> = result.into_iter().collect();
	sorted_result.sort_by(|a, b| a.1.cmp(&b.1));

	let mut tab_order: Vec<String> = Vec::new();
	for (key, _) in sorted_result {
		tab_order.push(key);
	}

	tab_order
}

pub fn find_next_panel_uuid(layout: &Layout, current_panel_uuid: &str) -> Option<String> {
	let tab_order = calculate_tab_order(layout);
	let mut found_current_panel = false;

	for panel_uuid in tab_order {
		if found_current_panel {
			return Some(panel_uuid);
		}

		if panel_uuid == current_panel_uuid {
			found_current_panel = true;
		}
	}

	None
}

pub fn find_previous_panel_uuid(layout: &Layout, current_panel_uuid: &str) -> Option<String> {
	let tab_order = calculate_tab_order(layout);
	let mut previous_panel_uuid: Option<String> = None;

	for panel_uuid in tab_order {
		if panel_uuid == current_panel_uuid {
			return previous_panel_uuid;
		}

		previous_panel_uuid = Some(panel_uuid);
	}

	None
}