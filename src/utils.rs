use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
use simplelog::*;
use std::fs::File;
use std::io::Write as IoWrite;
use std::io::{stdin, stdout, Read};
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use serde::{
    de::MapAccess, de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

use crate::entities::{Bounds, Cell, InputBounds, ScreenBuffer}; // Update this line to use crate::entities

pub fn get_fg_color(color: &str) -> String {
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

pub fn print_with_color_at(
    y: usize,
    x: usize,
    color: &str,
    text: &str,
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
    buffer: &mut ScreenBuffer,
) {
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
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
    buffer: &mut ScreenBuffer,
) {
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
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
    buffer: &mut ScreenBuffer,
) {
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
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
    buffer: &mut ScreenBuffer,
) {
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
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
    buffer: &mut ScreenBuffer,
) {
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
                screen,
                buffer,
            );
            print_with_color_and_background_at(
                y,
                title_start_position,
                title_fg_color,
                title_bg_color,
                &formatted_title,
                screen,
                buffer,
            );
            draw_horizontal_line(
                y,
                x1 + line_before_title_length + title_length,
                x2,
                fg_color,
                bg_color,
                screen,
                buffer,
            );
        } else {
            // If the title is too long, just draw a line without the title
            draw_horizontal_line(y, x1, x2, fg_color, bg_color, screen, buffer);
        }
    } else {
        // If there is no title, just draw a full horizontal line
        draw_horizontal_line(y, x1, x2, fg_color, bg_color, screen, buffer);
    }
}

static H_SCROLL_CHAR: &str = "|";
static V_SCROLL_CHAR: &str = "-";

pub fn draw_box(
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
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
    buffer: &mut ScreenBuffer,
) {
    log::debug!(
        "Drawing box with bounds '{:?}', border color '{}', background color '{}', parent background color '{}', title '{}', title fg color '{}', title bg color '{}', title position '{}', content '{:?}', fg color '{}', overflow behavior '{}', horizontal scroll '{}', vertical scroll '{}'",
        bounds,
        border_color,
        bg_color.unwrap_or("default"),
        parent_bg_color.unwrap_or("default"),
        title.unwrap_or("None"),
        title_fg_color,
        title_bg_color,
        title_position,
        content,
        fg_color,
        overflow_behavior,
        horizontal_scroll,
        vertical_scroll
    );
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

    log::debug!(
        "Drawing box with title '{}', background color '{}', background color code '{}'",
        title.unwrap_or("None"),
        bg_color.unwrap_or("default"),
        bg_color_code
    );

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
                screen,
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
            screen,
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
                    screen,
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
                    screen,
                    buffer,
                );
            }
        }
    }

    if _overflowing && overflow_behavior != "scroll" {
        if overflow_behavior == "fill" {
            fill_box(
                &bounds,
                true,
                bg_color.unwrap_or("default"),
                '█',
                screen,
                buffer,
            );
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
            fill_box(
                &bounds,
                false,
                parent_bg_color.unwrap_or("default"),
                ' ',
                screen,
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
            screen,
            buffer,
        );
        draw_vertical_line(
            bounds.left(),
            bounds.top() + 1,
            bounds.bottom() - 1,
            border_color,
            bg_color.unwrap_or("default"),
            screen,
            buffer,
        );
        draw_vertical_line(
            bounds.right(),
            bounds.top() + 1,
            bounds.bottom() - 1,
            border_color,
            bg_color.unwrap_or("default"),
            screen,
            buffer,
        );

        print_with_color_and_background_at(
            bounds.top() + 2 + vertical_scrollbar_position,
            bounds.right(),
            border_color,
            bg_color.unwrap_or("default"),
            V_SCROLL_CHAR,
            screen,
            buffer,
        );

        print_with_color_and_background_at(
            bounds.bottom(),
            bounds.left() + 2 + horizontal_scrollbar_position,
            border_color,
            bg_color.unwrap_or("default"),
            H_SCROLL_CHAR,
            screen,
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

pub fn fill_box(
    bounds: &Bounds,
    inside: bool,
    bg_color: &str,
    fill_char: char,
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
    buffer: &mut ScreenBuffer,
) {
    let fg_color_code = get_fg_color(bg_color);
    let bg_color_code = get_bg_color(bg_color);

    log::debug!(
        "Filling box with bounds '{:?}', inside '{}', background color '{}', fill char '{}'",
        bounds,
        inside,
        bg_color,
        fill_char
    );

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
    termion::terminal_size().unwrap().0 as usize
}

pub fn screen_height() -> usize {
    termion::terminal_size().unwrap().1 as usize
}

pub fn screen_bounds() -> Bounds {
    Bounds {
        x1: 0,
        y1: 0,
        x2: screen_width(),
        y2: screen_height(),
    }
}

pub fn input_bounds_to_bounds(input_bounds: &InputBounds, parent_bounds: &Bounds) -> Bounds {
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
    if value.ends_with('%') {
        let percentage = value.trim_end_matches('%').parse::<f64>().unwrap() / 100.0;
        (percentage * total as f64).round() as usize
    } else {
        value.parse::<usize>().unwrap()
    }
}

pub fn content_size(text: &str) -> (usize, usize) {
    let mut width = 0;
    let mut height = 0;
    for line in text.lines() {
        width = width.max(line.len());
        height += 1;
    }
    (width, height)
}
