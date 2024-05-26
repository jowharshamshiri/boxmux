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

enum BoxEvent {
    Refresh(Arc<Mutex<BoxEntity>>),
    EnterLeave(Arc<Mutex<BoxEntity>>),
}

#[derive(Clone, PartialEq)]
struct Cell {
    fg_color: String,
    bg_color: String,
    ch: char,
}

#[derive(Clone)]
struct ScreenBuffer {
    width: usize,
    height: usize,
    buffer: Vec<Vec<Cell>>,
}

impl ScreenBuffer {
    fn new(width: usize, height: usize) -> Self {
        let default_cell = Cell {
            fg_color: get_fg_color("default"),
            bg_color: get_bg_color("default"),
            ch: ' ',
        };
        let buffer = vec![vec![default_cell; width]; height];
        ScreenBuffer {
            width,
            height,
            buffer,
        }
    }

    fn clear(&mut self) {
        let default_cell = Cell {
            fg_color: get_fg_color("default"),
            bg_color: get_bg_color("default"),
            ch: ' ',
        };
        self.buffer = vec![vec![default_cell; self.width]; self.height];
    }

    fn update(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.buffer[y][x] = cell;
        }
    }

    fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height {
            Some(&self.buffer[y][x])
        } else {
            None
        }
    }
}

// BOX_EVENTS! {
//     "on_error",
//     "on_enter",
//     "on_leave",
//     "on_refresh",
// }

fn get_fg_color(color: &str) -> String {
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

fn get_bg_color(color: &str) -> String {
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

fn print_with_color_at(
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

fn print_with_color_and_background_at(
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

fn draw_horizontal_line(
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

fn draw_vertical_line(
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

fn draw_horizontal_line_with_title(
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

fn draw_box(
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
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
    buffer: &mut ScreenBuffer,
) {
    let border_color_code = get_fg_color(border_color);
    let title_fg_color_code = get_fg_color(title_fg_color);
    let title_bg_color_code = get_bg_color(title_bg_color);
    let fg_color_code = get_fg_color(fg_color);
    let bg_color_code = get_bg_color(bg_color.unwrap_or("default"));
    let parent_bg_color_code = get_bg_color(parent_bg_color.unwrap_or("default"));
    let mut _overflowing = false;

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
                _overflowing = true;
            } else {
                if max_title_length > top_border_length.saturating_sub(3) {
                    max_title_length = top_border_length.saturating_sub(3);
                }

                if title_length > max_title_length {
                    formatted_title = format!("{}...", &formatted_title[..max_title_length]);
                }
            }
        }

        if !_overflowing {
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
        if content_width > bounds.width().saturating_sub(4)
            || content_height > bounds.height().saturating_sub(4)
        {
            _overflowing = true;
        } else {
            print_with_color_at(
                bounds.top() + 2,
                bounds.left() + 2,
                fg_color,
                content,
                screen,
                buffer,
            );
        }
    }

    if _overflowing {
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

fn fill_box(
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

fn screen_width() -> usize {
    termion::terminal_size().unwrap().0 as usize
}

fn screen_height() -> usize {
    termion::terminal_size().unwrap().1 as usize
}

fn screen_bounds() -> Bounds {
    Bounds {
        x1: 0,
        y1: 0,
        x2: screen_width(),
        y2: screen_height(),
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
struct InputBounds {
    x1: String,
    y1: String,
    x2: String,
    y2: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Bounds {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

impl InputBounds {
    fn to_bounds(&self, parent_bounds: &Bounds) -> Bounds {
        input_bounds_to_bounds(self, parent_bounds)
    }
}

impl Bounds {
    fn width(&self) -> usize {
        self.x2.saturating_sub(self.x1)
    }

    fn height(&self) -> usize {
        self.y2.saturating_sub(self.y1)
    }

    fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x1 && x < self.x2 && y >= self.y1 && y < self.y2
    }

    fn contains_bounds(&self, other: &Bounds) -> bool {
        self.contains(other.x1, other.y1) && self.contains(other.x2, other.y2)
    }

    fn intersects(&self, other: &Bounds) -> bool {
        self.contains(other.x1, other.y1)
            || self.contains(other.x2, other.y2)
            || self.contains(other.x1, other.y2)
            || self.contains(other.x2, other.y1)
    }

    fn intersection(&self, other: &Bounds) -> Option<Bounds> {
        if self.intersects(other) {
            Some(Bounds {
                x1: self.x1.max(other.x1),
                y1: self.y1.max(other.y1),
                x2: self.x2.min(other.x2),
                y2: self.y2.min(other.y2),
            })
        } else {
            None
        }
    }

    fn union(&self, other: &Bounds) -> Bounds {
        Bounds {
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
            x2: self.x2.max(other.x2),
            y2: self.y2.max(other.y2),
        }
    }

    fn translate(&self, dx: isize, dy: isize) -> Bounds {
        Bounds {
            x1: (self.x1 as isize + dx) as usize,
            y1: (self.y1 as isize + dy) as usize,
            x2: (self.x2 as isize + dx) as usize,
            y2: (self.y2 as isize + dy) as usize,
        }
    }

    fn center(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    fn center_x(&self) -> usize {
        (self.x1 + self.x2) / 2
    }

    fn center_y(&self) -> usize {
        (self.y1 + self.y2) / 2
    }

    fn top_left(&self) -> (usize, usize) {
        (self.x1, self.y1)
    }

    fn top_right(&self) -> (usize, usize) {
        (self.x2, self.y1)
    }

    fn bottom_left(&self) -> (usize, usize) {
        (self.x1, self.y2)
    }

    fn bottom_right(&self) -> (usize, usize) {
        (self.x2, self.y2)
    }

    fn top(&self) -> usize {
        self.y1
    }

    fn bottom(&self) -> usize {
        self.y2
    }

    fn left(&self) -> usize {
        self.x1
    }

    fn right(&self) -> usize {
        self.x2
    }

    fn center_top(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, self.y1)
    }

    fn center_bottom(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, self.y2)
    }

    fn center_left(&self) -> (usize, usize) {
        (self.x1, (self.y1 + self.y2) / 2)
    }

    fn center_right(&self) -> (usize, usize) {
        (self.x2, (self.y1 + self.y2) / 2)
    }
}

fn input_bounds_to_bounds(input_bounds: &InputBounds, parent_bounds: &Bounds) -> Bounds {
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

fn parse_percentage(value: &str, total: usize) -> usize {
    if value.ends_with('%') {
        let percentage = value.trim_end_matches('%').parse::<f64>().unwrap() / 100.0;
        (percentage * total as f64).round() as usize
    } else {
        value.parse::<usize>().unwrap()
    }
}

fn content_size(text: &str) -> (usize, usize) {
    let mut width = 0;
    let mut height = 0;
    for line in text.lines() {
        width = width.max(line.len());
        height += 1;
    }
    (width, height)
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BoxEntity {
    id: String,
    title: Option<String>,
    position: InputBounds,
    min_width: Option<usize>,
    min_height: Option<usize>,
    max_width: Option<usize>,
    max_height: Option<usize>,
    overflow_behavior: Option<String>,
    content: Option<String>,
    scroll: bool,
    refresh_interval: Option<u64>,
    tab_order: Option<String>,
    on_error: Option<Vec<String>>,
    on_enter: Option<Vec<String>>,
    on_leave: Option<Vec<String>>,
    next_focus_id: Option<String>,
    children: Option<Vec<BoxEntity>>,
    fill: Option<bool>,
    fill_char: Option<char>,
    selected_fill_char: Option<char>,
    border: Option<bool>,
    border_color: Option<String>,
    selected_border_color: Option<String>,
    bg_color: Option<String>,
    selected_bg_color: Option<String>,
    fg_color: Option<String>,
    selected_fg_color: Option<String>,
    title_fg_color: Option<String>,
    title_bg_color: Option<String>,
    selected_title_bg_color: Option<String>,
    selected_title_fg_color: Option<String>,
    title_position: Option<String>,
    on_refresh: Option<Vec<String>>,
    thread: Option<bool>,
    #[serde(skip)]
    output: String,
    parent: Option<Box<BoxEntity>>,
    parent_layout: Option<Box<Layout>>,
}

impl PartialEq for BoxEntity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct App {
    layouts: Vec<Layout>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Layout {
    id: String,
    title: String,
    refresh_interval: Option<u64>,
    children: Vec<BoxEntity>,
    fill: Option<bool>,
    fill_char: Option<char>,
    selected_fill_char: Option<char>,
    border: Option<bool>,
    border_color: Option<String>,
    selected_border_color: Option<String>,
    bg_color: Option<String>,
    selected_bg_color: Option<String>,
    fg_color: Option<String>,
    selected_fg_color: Option<String>,
    title_fg_color: Option<String>,
    title_bg_color: Option<String>,
    title_position: Option<String>,
    selected_title_bg_color: Option<String>,
    selected_title_fg_color: Option<String>,
    overflow_behavior: Option<String>,
    #[serde(skip)]
    box_list_in_tab_order: Vec<BoxEntity>,
}

impl BoxEntity {
    fn bounds(&self) -> Bounds {
        input_bounds_to_bounds(&self.position, &screen_bounds())
    }

    fn absolute_bounds(&self, parent_bounds: Option<&Bounds>) -> Bounds {
        let screen_bounds_value = screen_bounds();
        let actual_parent_bounds = parent_bounds.unwrap_or(&screen_bounds_value);
        input_bounds_to_bounds(&self.position, actual_parent_bounds)
    }

    fn set_output(&mut self, output: &str) {
        self.output = output.to_string();
        log::debug!("Setting output for box '{}' to '{}'", self.id, output);
    }

    fn inherit_property<T: Clone>(child_value: Option<T>, parent_value: Option<T>) -> Option<T> {
        if child_value.is_some() {
            child_value
        } else {
            parent_value
        }
    }

    fn calc_border_color(&self) -> &str {
        if self.is_selected() {
            if let Some(ref selected_border_color) = self.selected_border_color {
                if !selected_border_color.is_empty() {
                    return selected_border_color;
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(ref selected_border_color) = parent.selected_border_color {
                    if !selected_border_color.is_empty() {
                        return selected_border_color;
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref selected_border_color) = parent_layout.selected_border_color {
                    if !selected_border_color.is_empty() {
                        return selected_border_color;
                    }
                }
            }
        } else {
            if let Some(ref border_color) = self.border_color {
                return border_color;
            }
            if let Some(parent) = &self.parent {
                if let Some(ref border_color) = parent.border_color {
                    return border_color;
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref border_color) = parent_layout.border_color {
                    return border_color;
                }
            }
        }
        "default"
    }

    fn calc_fg_color(&self) -> &str {
        if self.is_selected() {
            if let Some(ref selected_fg_color) = self.selected_fg_color {
                if !selected_fg_color.is_empty() {
                    return selected_fg_color;
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(ref selected_fg_color) = parent.selected_fg_color {
                    if !selected_fg_color.is_empty() {
                        return selected_fg_color;
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref selected_fg_color) = parent_layout.selected_fg_color {
                    if !selected_fg_color.is_empty() {
                        return selected_fg_color;
                    }
                }
            }
        } else {
            if let Some(ref fg_color) = self.fg_color {
                return fg_color;
            }
            if let Some(parent) = &self.parent {
                if let Some(ref fg_color) = parent.fg_color {
                    return fg_color;
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref fg_color) = parent_layout.fg_color {
                    return fg_color;
                }
            }
        }
        "default"
    }

    fn calc_bg_color(&self) -> &str {
        log::debug!("Calculating background color for box '{}'", self.id);

        if self.is_selected() {
            log::debug!("Box '{}' is selected", self.id);
            if let Some(ref selected_bg_color) = self.selected_bg_color {
                if !selected_bg_color.is_empty() {
                    log::debug!(
                        "Using box's selected background color: '{}'",
                        selected_bg_color
                    );
                    return selected_bg_color;
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(ref selected_bg_color) = parent.selected_bg_color {
                    if !selected_bg_color.is_empty() {
                        log::debug!(
                            "Using parent's selected background color: '{}'",
                            selected_bg_color
                        );
                        return selected_bg_color;
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref selected_bg_color) = parent_layout.selected_bg_color {
                    if !selected_bg_color.is_empty() {
                        log::debug!(
                            "Using parent layout's selected background color: '{}'",
                            selected_bg_color
                        );
                        return selected_bg_color;
                    }
                }
            }
        } else {
            log::debug!("Box '{}' is not selected", self.id);
            if let Some(ref bg_color) = self.bg_color {
                log::debug!("Using box's background color: '{}'", bg_color);
                return bg_color;
            }
            if let Some(parent) = &self.parent {
                if let Some(ref bg_color) = parent.bg_color {
                    log::debug!("Using parent's background color: '{}'", bg_color);
                    return bg_color;
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref bg_color) = parent_layout.bg_color {
                    log::debug!("Using parent layout's background color: '{}'", bg_color);
                    return bg_color;
                }
            }
        }

        log::debug!("Using default background color");
        "default"
    }

    fn calc_title_bg_color(&self) -> &str {
        if self.is_selected() {
            if let Some(ref selected_title_bg_color) = self.selected_title_bg_color {
                if !selected_title_bg_color.is_empty() {
                    return selected_title_bg_color;
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(ref selected_title_bg_color) = parent.selected_title_bg_color {
                    if !selected_title_bg_color.is_empty() {
                        return selected_title_bg_color;
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref selected_title_bg_color) = parent_layout.selected_title_bg_color {
                    if !selected_title_bg_color.is_empty() {
                        return selected_title_bg_color;
                    }
                }
            }
        } else {
            if let Some(ref title_bg_color) = self.title_bg_color {
                return title_bg_color;
            }
            if let Some(parent) = &self.parent {
                if let Some(ref title_bg_color) = parent.title_bg_color {
                    return title_bg_color;
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref title_bg_color) = parent_layout.title_bg_color {
                    return title_bg_color;
                }
            }
        }
        "default"
    }

    fn calc_title_fg_color(&self) -> &str {
        if self.is_selected() {
            if let Some(ref selected_title_fg_color) = self.selected_title_fg_color {
                if !selected_title_fg_color.is_empty() {
                    return selected_title_fg_color;
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(ref selected_title_fg_color) = parent.selected_title_fg_color {
                    if !selected_title_fg_color.is_empty() {
                        return selected_title_fg_color;
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref selected_title_fg_color) = parent_layout.selected_title_fg_color {
                    if !selected_title_fg_color.is_empty() {
                        return selected_title_fg_color;
                    }
                }
            }
        } else {
            if let Some(ref title_fg_color) = self.title_fg_color {
                return title_fg_color;
            }
            if let Some(parent) = &self.parent {
                if let Some(ref title_fg_color) = parent.title_fg_color {
                    return title_fg_color;
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(ref title_fg_color) = parent_layout.title_fg_color {
                    return title_fg_color;
                }
            }
        }
        "default"
    }

    fn calc_title_position(&self) -> &str {
        if let Some(title_position) = &self.title_position {
            return title_position;
        }
        if let Some(parent) = &self.parent {
            if let Some(title_position) = parent.title_position.as_deref() {
                return title_position;
            }
        }
        if let Some(parent_layout) = &self.parent_layout {
            if let Some(title_position) = parent_layout.title_position.as_deref() {
                return title_position;
            }
        }
        "start"
    }

    fn calc_fill_char(&self) -> char {
        if self.is_selected() {
            if let Some(selected_fill_char) = self.selected_fill_char {
                return selected_fill_char;
            }
            if let Some(parent) = &self.parent {
                if let Some(selected_fill_char) = parent.selected_fill_char {
                    return selected_fill_char;
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(selected_fill_char) = parent_layout.selected_fill_char {
                    return selected_fill_char;
                }
            }
        } else {
            if let Some(fill_char) = self.fill_char {
                return fill_char;
            }
            if let Some(parent) = &self.parent {
                if let Some(fill_char) = parent.fill_char {
                    return fill_char;
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(fill_char) = parent_layout.fill_char {
                    return fill_char;
                }
            }
        }
        '█'
    }

    fn calc_border(&self) -> bool {
        if let Some(border) = self.border {
            return border;
        }
        if let Some(parent) = &self.parent {
            if let Some(border) = parent.border {
                return border;
            }
        }
        if let Some(parent_layout) = &self.parent_layout {
            if let Some(border) = parent_layout.border {
                return border;
            }
        }
        true
    }

    fn calc_overflow_behavior(&self) -> &str {
        if let Some(ref overflow_behavior) = self.overflow_behavior {
            return overflow_behavior;
        }
        if let Some(parent) = &self.parent {
            if let Some(ref overflow_behavior) = parent.overflow_behavior {
                return overflow_behavior;
            }
        }
        if let Some(parent_layout) = &self.parent_layout {
            if let Some(ref overflow_behavior) = parent_layout.overflow_behavior {
                return overflow_behavior;
            }
        }
        "removed"
    }

    fn calc_refresh_interval(&self) -> Option<u64> {
        if let Some(refresh_interval) = self.refresh_interval {
            return Some(refresh_interval);
        }
        if let Some(parent) = &self.parent {
            if let Some(refresh_interval) = parent.refresh_interval {
                return Some(refresh_interval);
            }
        }
        if let Some(parent_layout) = &self.parent_layout {
            if let Some(refresh_interval) = parent_layout.refresh_interval {
                return Some(refresh_interval);
            }
        }
        None
    }

    fn draw(
        &mut self,
        screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
        buffer: &mut ScreenBuffer,
    ) {
        let parent_bounds = if self.parent.is_none() {
            Some(screen_bounds())
        } else {
            Some(self.parent.as_ref().unwrap().bounds())
        };

        // Calculate properties before borrowing self mutably
        let bounds = self.absolute_bounds(parent_bounds.as_ref());
        let bg_color = self.calc_bg_color().to_string();
        let parent_bg_color = if self.parent.is_none() {
            "default".to_string()
        } else {
            self.parent.as_ref().unwrap().calc_bg_color().to_string()
        };
        let fg_color = self.calc_fg_color().to_string();
        let title_bg_color = self.calc_title_bg_color().to_string();
        let title_fg_color = self.calc_title_fg_color().to_string();
        let border = self.calc_border();
        let border_color = self.calc_border_color().to_string();
        let fill_char = self.calc_fill_char();

        // Draw fill
        fill_box(&bounds, border, &bg_color, fill_char, screen, buffer);

        let mut content = self.content.as_deref();
        // check output is not null or empty
        if !self.output.is_empty() {
            content = Some(&self.output);
        }

        // Draw border with title
        draw_box(
            &bounds,
            &border_color,
            Some(&bg_color),
            Some(&parent_bg_color),
            self.title.as_deref(),
            &title_fg_color,
            &title_bg_color,
            self.calc_title_position(),
            content,
            &fg_color,
            self.calc_overflow_behavior(),
            screen,
            buffer,
        );

        // Draw children
        if let Some(children) = &mut self.children {
            for child in children {
                child.draw(screen, buffer);
            }
        }
    }

    fn is_selectable(&self) -> bool {
        self.tab_order.is_some() && self.tab_order.as_ref().unwrap() != "none"
    }

    fn is_selected(&self) -> bool {
        *SELECTED_BOX.lock().unwrap() == Some(self.clone())
    }

    fn has_events(&self) -> bool {
        for event in &[
            &self.on_enter,
            &self.on_leave,
            &self.on_error,
            &self.on_refresh,
        ] {
            if event.is_some() {
                return true;
            }
        }
        false
    }

    fn has_refresh(&self) -> bool {
        self.on_refresh.is_some()
    }

    fn has_enter(&self) -> bool {
        self.on_enter.is_some()
    }

    fn has_leave(&self) -> bool {
        self.on_leave.is_some()
    }

    fn has_error(&self) -> bool {
        self.on_error.is_some()
    }

    fn execute_refresh(&mut self) {
        log::info!("Executing refresh for box '{}'", self.id);
        if let Some(commands) = &self.on_refresh {
            let output = commands
                .iter()
                .map(|cmd| {
                    let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
                    String::from_utf8_lossy(&output.stdout).to_string()
                })
                .collect::<Vec<_>>()
                .join("\n");

            self.set_output(&output);
        }
    }

    fn execute_enter(&mut self) {
        if let Some(commands) = &self.on_enter {
            let output = commands
                .iter()
                .map(|cmd| {
                    let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
                    String::from_utf8_lossy(&output.stdout).to_string()
                })
                .collect::<Vec<_>>()
                .join("\n");

            self.set_output(&output);
        }
    }

    fn execute_leave(&mut self) {
        if let Some(commands) = &self.on_leave {
            let output = commands
                .iter()
                .map(|cmd| {
                    let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
                    String::from_utf8_lossy(&output.stdout).to_string()
                })
                .collect::<Vec<_>>()
                .join("\n");

            self.set_output(&output);
        }
    }

    fn execute_error(&mut self) {
        if let Some(commands) = &self.on_error {
            let output = commands
                .iter()
                .map(|cmd| {
                    let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
                    String::from_utf8_lossy(&output.stdout).to_string()
                })
                .collect::<Vec<_>>()
                .join("\n");

            self.set_output(&output);
        }
    }
}

impl Layout {
    fn populate_tab_order(&mut self) {
        let mut tab_order_list = Vec::new();
        self.collect_tab_order_boxes(&mut tab_order_list);

        // Ensure the boxes are sorted by tab_order
        tab_order_list.sort_by_key(|box_entity| {
            box_entity
                .tab_order
                .as_ref()
                .and_then(|order| order.parse::<usize>().ok())
                .unwrap_or(usize::MAX)
        });

        self.box_list_in_tab_order = tab_order_list;
    }

    fn collect_tab_order_boxes(&self, tab_order_list: &mut Vec<BoxEntity>) {
        for child in &self.children {
            self.recursively_collect_selectable_boxes(child, tab_order_list);
        }
    }

    fn recursively_collect_selectable_boxes(
        &self,
        box_entity: &BoxEntity,
        tab_order_list: &mut Vec<BoxEntity>,
    ) {
        if box_entity.is_selectable() {
            tab_order_list.push(box_entity.clone());
            log::info!(
                "Added box '{}' with tab order '{}'",
                box_entity.id,
                box_entity.tab_order.as_ref().unwrap()
            );
        }
        if let Some(children) = &box_entity.children {
            for child in children {
                self.recursively_collect_selectable_boxes(child, tab_order_list);
            }
        }
    }

    fn draw(
        &mut self,
        screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
        buffer: &mut ScreenBuffer,
    ) {
        if let Some(bg_color) = &self.bg_color {
            fill_box(
                &screen_bounds(),
                self.border.unwrap_or(true),
                bg_color,
                self.fill_char.unwrap_or('█'),
                screen,
                buffer,
            );
        }
        for child in &mut self.children {
            child.draw(screen, buffer);
        }
    }
}

impl App {
    fn draw(
        &mut self,
        screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
        buffer: &mut ScreenBuffer,
    ) {
        for layout in &mut self.layouts {
            layout.draw(screen, buffer);
        }
    }

    fn start_event_threads(&mut self) {
        let (main_tx, main_rx) = mpsc::channel();
        for layout in &mut self.layouts {
            for box_entity in &mut layout.children {
                if box_entity.has_events() {
                    let box_clone = Arc::new(Mutex::new(box_entity.clone()));
                    let thread = box_entity.thread.unwrap_or(false);
                    log::info!("box {} thread {}", box_entity.id, thread);
                    if thread {
                        log::info!("Starting thread for box '{}'", box_entity.id);
                        // Single-threaded box event handling
                        if box_entity.has_refresh() {
                            let box_clone = Arc::clone(&box_clone);
                            thread::spawn(move || {
                                let refresh_interval = box_clone
                                    .lock()
                                    .unwrap()
                                    .calc_refresh_interval()
                                    .unwrap_or(1);
                                loop {
                                    {
                                        let mut box_guard = box_clone.lock().unwrap(); // Make box_guard mutable
                                        box_guard.execute_refresh();
                                    }
                                    thread::sleep(Duration::from_secs(refresh_interval));
                                }
                            });
                        }

                        if box_entity.has_enter() || box_entity.has_leave() {
                            let box_clone = Arc::clone(&box_clone);
                            thread::spawn(move || loop {
                                {
                                    let mut box_guard = box_clone.lock().unwrap(); // Make box_guard mutable
                                    if box_guard.is_selected() {
                                        box_guard.execute_enter();
                                    } else {
                                        box_guard.execute_leave();
                                    }
                                }
                                thread::sleep(Duration::from_millis(100));
                            });
                        }
                    } else {
                        // Main-threaded box event handling
                        let tx = main_tx.clone();
                        if box_entity.has_refresh() {
                            let box_clone = Arc::clone(&box_clone);
                            tx.send(BoxEvent::Refresh(box_clone)).unwrap();
                        }

                        if box_entity.has_enter() || box_entity.has_leave() {
                            let box_clone = Arc::clone(&box_clone);
                            tx.send(BoxEvent::EnterLeave(box_clone)).unwrap();
                        }
                    }
                }
            }
        }

        // Main event loop handling for boxes with thread=false
        thread::spawn(move || {
            for event in main_rx {
                match event {
                    BoxEvent::Refresh(box_clone) => {
                        let refresh_interval = box_clone
                            .lock()
                            .unwrap()
                            .calc_refresh_interval()
                            .unwrap_or(1);
                        loop {
                            {
                                let mut box_guard = box_clone.lock().unwrap(); // Make box_guard mutable
                                box_guard.execute_refresh();
                            }
                            thread::sleep(Duration::from_secs(refresh_interval));
                        }
                    }
                    BoxEvent::EnterLeave(box_clone) => loop {
                        {
                            let mut box_guard = box_clone.lock().unwrap(); // Make box_guard mutable
                            if box_guard.is_selected() {
                                box_guard.execute_enter();
                            } else {
                                box_guard.execute_leave();
                            }
                        }
                        thread::sleep(Duration::from_millis(100));
                    },
                }
            }
        });
    }
}

fn get_box_list_in_tab_order(box_list: &Vec<BoxEntity>, tab_order: &str) -> Vec<BoxEntity> {
    let mut tab_order_list = tab_order
        .split(',')
        .map(|id| {
            box_list
                .iter()
                .find(|b| b.id == id)
                .expect(&format!("Box with id {} not found", id))
                .clone()
        })
        .collect::<Vec<BoxEntity>>();

    // Sort by tab order
    tab_order_list.sort_by(|a, b| {
        let a_index = tab_order.split(',').position(|id| id == a.id).unwrap();
        let b_index = tab_order.split(',').position(|id| id == b.id).unwrap();
        a_index.cmp(&b_index)
    });

    tab_order_list
}

fn load_app_from_yaml(file_path: &str) -> Result<App, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut app: App = serde_yaml::from_str(&contents)?;

    // Populate parent fields recursively
    for layout in &mut app.layouts {
        let layout_clone = layout.clone();
        load_layout(&mut layout.children, &layout_clone, None);
    }

    Ok(app)
}

fn load_layout(
    children: &mut Vec<BoxEntity>,
    parent_layout: &Layout,
    parent: Option<Box<BoxEntity>>,
) {
    for child in children.iter_mut() {
        child.parent_layout = Some(Box::new(parent_layout.clone()));
        child.parent = parent.clone();

        if let Some(mut child_children) = child.children.take() {
            let parent_clone = Box::new(child.clone());
            load_layout(&mut child_children, parent_layout, Some(parent_clone));
            child.children = Some(child_children);
        }
    }
}

static SELECTED_BOX: Mutex<Option<BoxEntity>> = Mutex::new(None);
fn select_next_box(layout: &Layout) {
    let mut selected_box_guard = SELECTED_BOX.lock().unwrap();
    let tab_order_list = &layout.box_list_in_tab_order;

    if tab_order_list.is_empty() {
        return;
    }

    let next_index = match selected_box_guard.as_ref() {
        Some(selected_box) => {
            let selected_index = tab_order_list
                .iter()
                .position(|b| b == selected_box)
                .unwrap_or(0);
            (selected_index + 1) % tab_order_list.len()
        }
        None => 0,
    };

    *selected_box_guard = Some(tab_order_list[next_index].clone());
}

fn select_previous_box(layout: &Layout) {
    let mut selected_box_guard = SELECTED_BOX.lock().unwrap();
    let tab_order_list = &layout.box_list_in_tab_order;

    if tab_order_list.is_empty() {
        return;
    }

    let prev_index = match selected_box_guard.as_ref() {
        Some(selected_box) => {
            let selected_index = tab_order_list
                .iter()
                .position(|b| b == selected_box)
                .unwrap_or(0);
            if selected_index == 0 {
                tab_order_list.len() - 1
            } else {
                selected_index - 1
            }
        }
        None => tab_order_list.len() - 1,
    };

    *selected_box_guard = Some(tab_order_list[prev_index].clone());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("/Users/bahram/ws/prj/machinegenesis/crossbash/trash/app.log").unwrap(),
    )])
    .unwrap();

    log::info!("Starting app");
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode()?);

    // Load layout from YAML
    let mut app =
        load_app_from_yaml("/Users/bahram/ws/prj/machinegenesis/crossbash/layouts/dashboard.yaml")?;

    for layout in &mut app.layouts {
        layout.populate_tab_order();
        if !layout.box_list_in_tab_order.is_empty() {
            let first_box = layout.box_list_in_tab_order[0].clone();
            println!("Initially selecting box: {}", first_box.id); // Debug print
            *SELECTED_BOX.lock().unwrap() = Some(first_box);
        }
    }

    // Channel to communicate between input handling and drawing
    let (tx, rx) = mpsc::channel();

    // Handle input in a separate thread
    let input_tx = tx.clone();
    thread::spawn(move || {
        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => {
                    input_tx.send("exit").unwrap();
                    break;
                }
                // tab
                Key::Char('\t') => {
                    input_tx.send("next_box").unwrap();
                }
                // shift-tab
                Key::BackTab => {
                    input_tx.send("previous_box").unwrap();
                }
                _ => {}
            }
        }
    });

    // Start refresh threads for each box with a refresh interval
    app.start_event_threads();

    // Handle terminal resize
    let resize_tx = tx.clone();
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGWINCH]).unwrap();
        for _ in signals.forever() {
            resize_tx.send("resize").unwrap();
        }
    });

    // Main drawing loop
    let mut screen_buffer = ScreenBuffer::new(screen_width(), screen_height());
    let mut prev_screen_buffer = screen_buffer.clone();
    loop {
        app.draw(&mut stdout, &mut screen_buffer);
        // Compare screen buffers and update only the differences
        for y in 0..screen_buffer.height {
            for x in 0..screen_buffer.width {
                if screen_buffer.get(x, y) != prev_screen_buffer.get(x, y) {
                    if let Some(cell) = screen_buffer.get(x, y) {
                        write!(
                            stdout,
                            "{}{}{}{}",
                            cursor::Goto((x + 1) as u16, (y + 1) as u16),
                            cell.bg_color,
                            cell.fg_color,
                            cell.ch
                        )
                        .unwrap();
                    }
                }
            }
        }
        stdout.flush().unwrap();
        prev_screen_buffer = screen_buffer.clone();

        // Check for input
        if let Ok(msg) = rx.try_recv() {
            match msg {
                "exit" => break,
                "resize" => {
                    write!(stdout, "{}", termion::clear::All).unwrap();
                    screen_buffer = ScreenBuffer::new(screen_width(), screen_height());
                    prev_screen_buffer = screen_buffer.clone();
                    app.draw(&mut stdout, &mut screen_buffer);
                }
                "next_box" => {
                    for layout in &app.layouts {
                        select_next_box(layout);
                    }
                }
                "previous_box" => {
                    for layout in &app.layouts {
                        select_previous_box(layout);
                    }
                }
                _ => {}
            }
        }
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}
