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

use crate::state::SELECTED_BOX;

use crate::utils::{
    draw_box, fill_box, get_bg_color, get_fg_color, input_bounds_to_bounds, screen_bounds,
}; // Import necessary functions

pub enum BoxEvent {
    Refresh(Arc<Mutex<BoxEntity>>),
    EnterLeave(Arc<Mutex<BoxEntity>>),
}

pub enum InputMessage {
    Exit,
    NextBox,
    PreviousBox,
    Resize,
    RedrawBox(BoxEntity),
    RedrawApp,
}

#[derive(Clone, PartialEq)]
pub struct Cell {
    pub fg_color: String,
    pub bg_color: String,
    pub ch: char,
}

#[derive(Clone)]
pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Vec<Cell>>,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
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

    pub fn clear(&mut self) {
        let default_cell = Cell {
            fg_color: get_fg_color("default"),
            bg_color: get_bg_color("default"),
            ch: ' ',
        };
        self.buffer = vec![vec![default_cell; self.width]; self.height];
    }

    pub fn update(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.buffer[y][x] = cell;
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height {
            Some(&self.buffer[y][x])
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct InputBounds {
    pub x1: String,
    pub y1: String,
    pub x2: String,
    pub y2: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Bounds {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl InputBounds {
    pub fn to_bounds(&self, parent_bounds: &Bounds) -> Bounds {
        input_bounds_to_bounds(self, parent_bounds)
    }
}

impl Bounds {
    pub fn width(&self) -> usize {
        self.x2.saturating_sub(self.x1)
    }

    pub fn height(&self) -> usize {
        self.y2.saturating_sub(self.y1)
    }

    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x1 && x < self.x2 && y >= self.y1 && y < self.y2
    }

    pub fn contains_bounds(&self, other: &Bounds) -> bool {
        self.contains(other.x1, other.y1) && self.contains(other.x2, other.y2)
    }

    pub fn intersects(&self, other: &Bounds) -> bool {
        self.contains(other.x1, other.y1)
            || self.contains(other.x2, other.y2)
            || self.contains(other.x1, other.y2)
            || self.contains(other.x2, other.y1)
    }

    pub fn intersection(&self, other: &Bounds) -> Option<Bounds> {
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

    pub fn union(&self, other: &Bounds) -> Bounds {
        Bounds {
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
            x2: self.x2.max(other.x2),
            y2: self.y2.max(other.y2),
        }
    }

    pub fn translate(&self, dx: isize, dy: isize) -> Bounds {
        Bounds {
            x1: (self.x1 as isize + dx) as usize,
            y1: (self.y1 as isize + dy) as usize,
            x2: (self.x2 as isize + dx) as usize,
            y2: (self.y2 as isize + dy) as usize,
        }
    }

    pub fn center(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    pub fn center_x(&self) -> usize {
        (self.x1 + self.x2) / 2
    }

    pub fn center_y(&self) -> usize {
        (self.y1 + self.y2) / 2
    }

    pub fn top_left(&self) -> (usize, usize) {
        (self.x1, self.y1)
    }

    pub fn top_right(&self) -> (usize, usize) {
        (self.x2, self.y1)
    }

    pub fn bottom_left(&self) -> (usize, usize) {
        (self.x1, self.y2)
    }

    pub fn bottom_right(&self) -> (usize, usize) {
        (self.x2, self.y2)
    }

    pub fn top(&self) -> usize {
        self.y1
    }

    pub fn bottom(&self) -> usize {
        self.y2
    }

    pub fn left(&self) -> usize {
        self.x1
    }

    pub fn right(&self) -> usize {
        self.x2
    }

    pub fn center_top(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, self.y1)
    }

    pub fn center_bottom(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, self.y2)
    }

    pub fn center_left(&self) -> (usize, usize) {
        (self.x1, (self.y1 + self.y2) / 2)
    }

    pub fn center_right(&self) -> (usize, usize) {
        (self.x2, (self.y1 + self.y2) / 2)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BoxEntity {
    pub id: String,
    pub title: Option<String>,
    pub position: InputBounds,
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
    pub max_width: Option<usize>,
    pub max_height: Option<usize>,
    pub overflow_behavior: Option<String>,
    pub content: Option<String>,
    pub scroll: bool,
    pub refresh_interval: Option<u64>,
    pub tab_order: Option<String>,
    pub on_error: Option<Vec<String>>,
    pub on_enter: Option<Vec<String>>,
    pub on_leave: Option<Vec<String>>,
    pub next_focus_id: Option<String>,
    pub children: Option<Vec<BoxEntity>>,
    pub fill: Option<bool>,
    pub fill_char: Option<char>,
    pub selected_fill_char: Option<char>,
    pub border: Option<bool>,
    pub border_color: Option<String>,
    pub selected_border_color: Option<String>,
    pub bg_color: Option<String>,
    pub selected_bg_color: Option<String>,
    pub fg_color: Option<String>,
    pub selected_fg_color: Option<String>,
    pub title_fg_color: Option<String>,
    pub title_bg_color: Option<String>,
    pub selected_title_bg_color: Option<String>,
    pub selected_title_fg_color: Option<String>,
    pub title_position: Option<String>,
    pub on_refresh: Option<Vec<String>>,
    pub thread: Option<bool>,
    #[serde(skip)]
    pub output: String,
    pub parent: Option<Box<BoxEntity>>,
    pub parent_layout: Option<Box<Layout>>,
    #[serde(skip)]
    pub horizontal_scroll: Option<f64>,
    #[serde(skip)]
    pub vertical_scroll: Option<f64>,
}

impl PartialEq for BoxEntity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct App {
    pub layouts: Vec<Layout>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Layout {
    pub id: String,
    pub title: String,
    pub refresh_interval: Option<u64>,
    pub children: Vec<BoxEntity>,
    pub fill: Option<bool>,
    pub fill_char: Option<char>,
    pub selected_fill_char: Option<char>,
    pub border: Option<bool>,
    pub border_color: Option<String>,
    pub selected_border_color: Option<String>,
    pub bg_color: Option<String>,
    pub selected_bg_color: Option<String>,
    pub fg_color: Option<String>,
    pub selected_fg_color: Option<String>,
    pub title_fg_color: Option<String>,
    pub title_bg_color: Option<String>,
    pub title_position: Option<String>,
    pub selected_title_bg_color: Option<String>,
    pub selected_title_fg_color: Option<String>,
    pub overflow_behavior: Option<String>,
    #[serde(skip)]
    pub box_list_in_tab_order: Vec<BoxEntity>,
}

impl BoxEntity {
    pub fn bounds(&self) -> Bounds {
        input_bounds_to_bounds(&self.position, &screen_bounds())
    }

    pub fn absolute_bounds(&self, parent_bounds: Option<&Bounds>) -> Bounds {
        let screen_bounds_value = screen_bounds();
        let actual_parent_bounds = parent_bounds.unwrap_or(&screen_bounds_value);
        input_bounds_to_bounds(&self.position, actual_parent_bounds)
    }

    pub fn set_output(&mut self, output: &str) {
        self.output = output.to_string();
        log::debug!("Setting output for box '{}' to '{}'", self.id, output);
    }

    pub fn inherit_property<T: Clone>(
        child_value: Option<T>,
        parent_value: Option<T>,
    ) -> Option<T> {
        if child_value.is_some() {
            child_value
        } else {
            parent_value
        }
    }

    pub fn calc_border_color(&self) -> &str {
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

    pub fn calc_fg_color(&self) -> &str {
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

    pub fn calc_bg_color(&self) -> &str {
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

    pub fn calc_title_bg_color(&self) -> &str {
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

    pub fn calc_title_fg_color(&self) -> &str {
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

    pub fn calc_title_position(&self) -> &str {
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

    pub fn calc_fill_char(&self) -> char {
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

    pub fn calc_border(&self) -> bool {
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

    pub fn calc_overflow_behavior(&self) -> &str {
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
        "scroll"
    }

    pub fn calc_refresh_interval(&self) -> Option<u64> {
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

    pub fn draw(
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

        log::info!(
            "Drawing box '{}' with horizontal scroll '{}', vertical scroll '{}'",
            self.id,
            self.current_horizontal_scroll(),
            self.current_vertical_scroll()
        );
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
            self.current_horizontal_scroll(),
            self.current_vertical_scroll(),
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

    pub fn is_selectable(&self) -> bool {
        self.tab_order.is_some() && self.tab_order.as_ref().unwrap() != "none"
    }

    pub fn is_selected(&self) -> bool {
        *SELECTED_BOX.lock().unwrap() == Some(self.clone())
    }

    pub fn scroll_down(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(5.0);
        if let Some(scroll) = self.vertical_scroll {
            self.vertical_scroll = Some((scroll + amount).min(100.0));
        } else {
            self.vertical_scroll = Some(amount.min(100.0));
        }
    }

    pub fn scroll_up(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(5.0);
        if let Some(scroll) = self.vertical_scroll {
            self.vertical_scroll = Some((scroll - amount).max(0.0));
        } else {
            self.vertical_scroll = Some(0.0);
        }
    }

    pub fn scroll_right(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(5.0);
        if let Some(scroll) = self.horizontal_scroll {
            self.horizontal_scroll = Some((scroll + amount).min(100.0));
        } else {
            self.horizontal_scroll = Some(amount.min(100.0));
        }
    }

    pub fn scroll_left(&mut self, amount: Option<f64>) {
        let amount = amount.unwrap_or(5.0);
        if let Some(scroll) = self.horizontal_scroll {
            self.horizontal_scroll = Some((scroll - amount).max(0.0));
        } else {
            self.horizontal_scroll = Some(0.0);
        }
    }

    pub fn current_vertical_scroll(&self) -> f64 {
        self.vertical_scroll.unwrap_or(0.0)
    }

    pub fn current_horizontal_scroll(&self) -> f64 {
        self.horizontal_scroll.unwrap_or(0.0)
    }

    pub fn has_events(&self) -> bool {
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

    pub fn has_refresh(&self) -> bool {
        self.on_refresh.is_some()
    }

    pub fn has_enter(&self) -> bool {
        self.on_enter.is_some()
    }

    pub fn has_leave(&self) -> bool {
        self.on_leave.is_some()
    }

    pub fn has_error(&self) -> bool {
        self.on_error.is_some()
    }

    pub fn execute_refresh(&mut self) {
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

    pub fn execute_enter(&mut self) {
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

    pub fn execute_leave(&mut self) {
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

    pub fn execute_error(&mut self) {
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
    pub fn populate_tab_order(&mut self) {
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

    pub fn collect_tab_order_boxes(&self, tab_order_list: &mut Vec<BoxEntity>) {
        for child in &self.children {
            self.recursively_collect_selectable_boxes(child, tab_order_list);
        }
    }

    pub fn recursively_collect_selectable_boxes(
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

    pub fn draw(
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
    pub fn draw(
        &mut self,
        screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
        buffer: &mut ScreenBuffer,
    ) {
        for layout in &mut self.layouts {
            layout.draw(screen, buffer);
        }
    }

    pub fn start_event_threads(&mut self) {
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
