use regex::Regex;
use serde_json::Value;
use std::{
    collections::HashMap,
    error::Error,
    hash::Hash,
    io::{self, Read, Write},
    os::unix::net::UnixStream,
};

use crate::{
    draw_utils::{get_bg_color, get_fg_color},
    screen_bounds, screen_height, screen_width,
    utils::input_bounds_to_bounds,
    AppContext, AppGraph, Layout, Message, Panel,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub enum EntityType {
    AppContext,
    App,
    Layout,
    Panel,
}

// Represents a granular field update
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct FieldUpdate {
    pub entity_type: EntityType,   // The type of entity being updated
    pub entity_id: Option<String>, // The ID of the entity (App, Layout, or Panel)
    pub field_name: String,        // The field name to be updated
    pub new_value: Value,          // The new value for the field
}

// The Updatable trait
pub trait Updatable {
    // Generate a diff of changes from another instance
    fn generate_diff(&self, other: &Self) -> Vec<FieldUpdate>;

    // Apply a list of updates to the current instance
    fn apply_updates(&mut self, updates: Vec<FieldUpdate>);
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Config {
    pub frame_delay: u64,
}

impl Hash for Config {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.frame_delay.hash(state);
    }
}

impl Default for Config {
    fn default() -> Self {
        Config { frame_delay: 30 }
    }
}

impl Config {
    pub fn new(frame_delay: u64) -> Self {
        let result = Config { frame_delay };
        result.validate();
        result
    }
    pub fn validate(&self) {
        if self.frame_delay == 0 {
            panic!("Validation error: frame_delay cannot be 0");
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub enum SocketFunction {
    ReplacePanelContent {
        panel_id: String,
        content: String,
    },
    ReplacePanelScript {
        panel_id: String,
        script: Vec<String>,
    },
    StopPanelRefresh {
        panel_id: String,
    },
    StartPanelRefresh {
        panel_id: String,
    },
    ReplacePanel {
        panel_id: String,
        new_panel: Panel,
    },
    SwitchActiveLayout {
        layout_id: String,
    },
    AddPanel {
        layout_id: String,
        panel: Panel,
    },
    RemovePanel {
        panel_id: String,
    },
}

pub fn run_socket_function(
    socket_function: SocketFunction,
    app_context: &AppContext,
) -> Result<(AppContext, Vec<Message>), Box<dyn Error>> {
    let mut app_context = app_context.clone();
    let mut messages = Vec::new();
    match socket_function {
        SocketFunction::ReplacePanelContent { panel_id, content } => {
            messages.push(Message::PanelOutputUpdate(panel_id, content));
        }
        SocketFunction::ReplacePanelScript { panel_id, script } => {
            messages.push(Message::PanelScriptUpdate(panel_id, script));
        }
        SocketFunction::StopPanelRefresh { panel_id } => {
            messages.push(Message::StopPanelRefresh(panel_id));
        }
        SocketFunction::StartPanelRefresh { panel_id } => {
            messages.push(Message::StartPanelRefresh(panel_id));
        }
        SocketFunction::ReplacePanel {
            panel_id,
            new_panel,
        } => {
            messages.push(Message::ReplacePanel(panel_id, new_panel));
        }
        SocketFunction::SwitchActiveLayout { layout_id } => {
            messages.push(Message::SwitchActiveLayout(layout_id));
        }
        SocketFunction::AddPanel { layout_id, panel } => {
            messages.push(Message::AddPanel(layout_id, panel));
        }
        SocketFunction::RemovePanel { panel_id } => {
            messages.push(Message::RemovePanel(panel_id));
        }
    }
    Ok((app_context, messages))
}

#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    pub fg_color: String,
    pub bg_color: String,
    pub ch: char,
}

#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Vec<Cell>>,
}

impl ScreenBuffer {
    pub fn new() -> Self {
        let default_cell = Cell {
            fg_color: get_fg_color("white"),
            bg_color: get_bg_color("black"),
            ch: ' ',
        };
        let width = screen_width();
        let height = screen_height();
        let buffer = vec![vec![default_cell; width]; height];
        ScreenBuffer {
            width,
            height,
            buffer,
        }
    }

    pub fn new_custom(width: usize, height: usize) -> Self {
        let default_cell = Cell {
            fg_color: get_fg_color("white"),
            bg_color: get_bg_color("black"),
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
            fg_color: get_fg_color("white"),
            bg_color: get_bg_color("black"),
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

    pub fn resize(&mut self, width: usize, height: usize) {
        // First handle shrinking the buffer if necessary
        if height < self.height {
            self.buffer.truncate(height);
        }
        if width < self.width {
            for row in &mut self.buffer {
                row.truncate(width);
            }
        }

        // Now handle expanding the buffer if necessary
        if height > self.height {
            let default_row = vec![
                Cell {
                    fg_color: get_fg_color("white"),
                    bg_color: get_bg_color("black"),
                    ch: ' ',
                };
                width
            ];

            self.buffer.resize_with(height, || default_row.clone());
        }
        if width > self.width {
            for row in &mut self.buffer {
                row.resize_with(width, || Cell {
                    fg_color: get_fg_color("white"),
                    bg_color: get_bg_color("black"),
                    ch: ' ',
                });
            }
        }

        // Update the dimensions
        self.width = width;
        self.height = height;
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct InputBounds {
    pub x1: String,
    pub y1: String,
    pub x2: String,
    pub y2: String,
}

impl InputBounds {
    pub fn to_bounds(&self, parent_bounds: &Bounds) -> Bounds {
        input_bounds_to_bounds(self, parent_bounds)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Bounds {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl PartialEq for Bounds {
    fn eq(&self, other: &Self) -> bool {
        self.x1 == other.x1 && self.y1 == other.y1 && self.x2 == other.x2 && self.y2 == other.y2
    }
}

impl Eq for Bounds {}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    Center,
    CenterTop,
    CenterBottom,
    CenterLeft,
    CenterRight,
}

impl Default for Anchor {
    fn default() -> Self {
        Anchor::Center
    }
}

impl Bounds {
    pub fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Bounds { x1, y1, x2, y2 }
    }

    pub fn validate(&self) {
        if self.x1 > self.x2 {
            panic!(
                "Validation error: x1 ({}) is greater than x2 ({})",
                self.x1, self.x2
            );
        }
        if self.y1 > self.y2 {
            panic!(
                "Validation error: y1 ({}) is greater than y2 ({})",
                self.y1, self.y2
            );
        }
    }

    pub fn width(&self) -> usize {
        self.x2.saturating_sub(self.x1)
    }

    pub fn height(&self) -> usize {
        self.y2.saturating_sub(self.y1)
    }

    pub fn to_string(&self) -> String {
        format!("({}, {}), ({}, {})", self.x1, self.y1, self.x2, self.y2)
    }

    pub fn extend(&mut self, horizontal_amount: usize, vertical_amount: usize, anchor: Anchor) {
        match anchor {
            Anchor::TopLeft => {
                self.x1 = self.x1.saturating_sub(horizontal_amount);
                self.y1 = self.y1.saturating_sub(vertical_amount);
            }
            Anchor::TopRight => {
                self.x2 += horizontal_amount;
                self.y1 = self.y1.saturating_sub(vertical_amount);
            }
            Anchor::BottomLeft => {
                self.x1 = self.x1.saturating_sub(horizontal_amount);
                self.y2 += vertical_amount;
            }
            Anchor::BottomRight => {
                self.x2 += horizontal_amount;
                self.y2 += vertical_amount;
            }
            Anchor::Center => {
                let half_horizontal = horizontal_amount / 2;
                let half_vertical = vertical_amount / 2;
                self.x1 = self.x1.saturating_sub(half_horizontal);
                self.y1 = self.y1.saturating_sub(half_vertical);
                self.x2 += half_horizontal;
                self.y2 += half_vertical;
            }
            Anchor::CenterTop => {
                let half_horizontal = horizontal_amount / 2;
                self.x1 = self.x1.saturating_sub(half_horizontal);
                self.x2 += half_horizontal;
                self.y1 = self.y1.saturating_sub(vertical_amount);
            }
            Anchor::CenterBottom => {
                let half_horizontal = horizontal_amount / 2;
                self.x1 = self.x1.saturating_sub(half_horizontal);
                self.x2 += half_horizontal;
                self.y2 += vertical_amount;
            }
            Anchor::CenterLeft => {
                let half_vertical = vertical_amount / 2;
                self.x1 = self.x1.saturating_sub(horizontal_amount);
                self.y1 = self.y1.saturating_sub(half_vertical);
                self.y2 += half_vertical;
            }
            Anchor::CenterRight => {
                let half_vertical = vertical_amount / 2;
                self.x2 += horizontal_amount;
                self.y1 = self.y1.saturating_sub(half_vertical);
                self.y2 += half_vertical;
            }
        }
        self.validate();
    }

    pub fn contract(&mut self, horizontal_amount: usize, vertical_amount: usize, anchor: Anchor) {
        match anchor {
            Anchor::TopLeft => {
                self.x1 += horizontal_amount;
                self.y1 += vertical_amount;
            }
            Anchor::TopRight => {
                self.x2 = self.x2.saturating_sub(horizontal_amount);
                self.y1 += vertical_amount;
            }
            Anchor::BottomLeft => {
                self.x1 += horizontal_amount;
                self.y2 = self.y2.saturating_sub(vertical_amount);
            }
            Anchor::BottomRight => {
                self.x2 = self.x2.saturating_sub(horizontal_amount);
                self.y2 = self.y2.saturating_sub(vertical_amount);
            }
            Anchor::Center => {
                let half_horizontal = horizontal_amount / 2;
                let half_vertical = vertical_amount / 2;
                self.x1 += half_horizontal;
                self.y1 += half_vertical;
                self.x2 = self.x2.saturating_sub(half_horizontal);
                self.y2 = self.y2.saturating_sub(half_vertical);
            }
            Anchor::CenterTop => {
                let half_horizontal = horizontal_amount / 2;
                self.x1 += half_horizontal;
                self.x2 = self.x2.saturating_sub(half_horizontal);
                self.y1 += vertical_amount;
            }
            Anchor::CenterBottom => {
                let half_horizontal = horizontal_amount / 2;
                self.x1 += half_horizontal;
                self.x2 = self.x2.saturating_sub(half_horizontal);
                self.y2 = self.y2.saturating_sub(vertical_amount);
            }
            Anchor::CenterLeft => {
                let half_vertical = vertical_amount / 2;
                self.x1 += horizontal_amount;
                self.y1 += half_vertical;
                self.y2 = self.y2.saturating_sub(half_vertical);
            }
            Anchor::CenterRight => {
                let half_vertical = vertical_amount / 2;
                self.x2 = self.x2.saturating_sub(horizontal_amount);
                self.y1 += half_vertical;
                self.y2 = self.y2.saturating_sub(half_vertical);
            }
        }
        self.validate();
    }

    pub fn move_to(&mut self, x: usize, y: usize, anchor: Anchor) {
        match anchor {
            Anchor::TopLeft => {
                let width = self.width();
                let height = self.height();
                self.x1 = x;
                self.y1 = y;
                self.x2 = x + width;
                self.y2 = y + height;
            }
            Anchor::TopRight => {
                let width = self.width();
                let height = self.height();
                self.x2 = x;
                self.y1 = y;
                self.x1 = x - width;
                self.y2 = y + height;
            }
            Anchor::BottomLeft => {
                let width = self.width();
                let height = self.height();
                self.x1 = x;
                self.y2 = y;
                self.x2 = x + width;
                self.y1 = y - height;
            }
            Anchor::BottomRight => {
                let width = self.width();
                let height = self.height();
                self.x2 = x;
                self.y2 = y;
                self.x1 = x - width;
                self.y1 = y - height;
            }
            Anchor::Center => {
                let width = self.width();
                let height = self.height();
                let half_width = width / 2;
                let half_height = height / 2;
                self.x1 = x - half_width;
                self.y1 = y - half_height;
                self.x2 = x + half_width;
                self.y2 = y + half_height;
            }
            Anchor::CenterTop => {
                let width = self.width();
                let height = self.height();
                let half_width = width / 2;
                self.x1 = x - half_width;
                self.x2 = x + half_width;
                self.y1 = y;
                self.y2 = y + height;
            }
            Anchor::CenterBottom => {
                let width = self.width();
                let height = self.height();
                let half_width = width / 2;
                self.x1 = x - half_width;
                self.x2 = x + half_width;
                self.y2 = y;
                self.y1 = y - height;
            }
            Anchor::CenterLeft => {
                let width = self.width();
                let height = self.height();
                let half_height = height / 2;
                self.x1 = x;
                self.x2 = x + width;
                self.y1 = y - half_height;
                self.y2 = y + half_height;
            }
            Anchor::CenterRight => {
                let width = self.width();
                let height = self.height();
                let half_height = height / 2;
                self.x2 = x;
                self.x1 = x - width;
                self.y1 = y - half_height;
                self.y2 = y + half_height;
            }
        }
        self.validate();
    }

    pub fn move_by(&mut self, dx: isize, dy: isize) {
        self.x1 = (self.x1 as isize + dx) as usize;
        self.y1 = (self.y1 as isize + dy) as usize;
        self.x2 = (self.x2 as isize + dx) as usize;
        self.y2 = (self.y2 as isize + dy) as usize;
        self.validate();
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

pub fn calculate_initial_bounds(app_graph: &AppGraph, layout: &Layout) -> HashMap<String, Bounds> {
    let mut bounds_map = HashMap::new();

    fn dfs(
        app_graph: &AppGraph,
        layout_id: &str,
        panel: &Panel,
        parent_bounds: Bounds,
        bounds_map: &mut HashMap<String, Bounds>,
    ) {
        let bounds = panel.absolute_bounds(Some(&parent_bounds));
        bounds_map.insert(panel.id.clone(), bounds.clone());

        if let Some(children) = &panel.children {
            for child in children {
                dfs(app_graph, layout_id, child, bounds.clone(), bounds_map);
            }
        }
    }

    let root_bounds = screen_bounds();
    if let Some(children) = &layout.children {
        for panel in children {
            dfs(
                app_graph,
                &layout.id,
                panel,
                root_bounds.clone(),
                &mut bounds_map,
            );
        }
    }

    bounds_map
}

pub fn adjust_bounds_with_constraints(
    layout: &Layout,
    mut bounds_map: HashMap<String, Bounds>,
) -> HashMap<String, Bounds> {
    fn apply_constraints(panel: &Panel, bounds: &mut Bounds) {
        if let Some(min_width) = panel.min_width {
            if bounds.width() < min_width {
                bounds.extend(min_width - bounds.width(), 0, panel.anchor.clone());
            }
        }
        if let Some(min_height) = panel.min_height {
            if bounds.height() < min_height {
                bounds.extend(0, min_height - bounds.height(), panel.anchor.clone());
            }
        }
        if let Some(max_width) = panel.max_width {
            if bounds.width() > max_width {
                bounds.contract(bounds.width() - max_width, 0, panel.anchor.clone());
            }
        }
        if let Some(max_height) = panel.max_height {
            if bounds.height() > max_height {
                bounds.contract(0, bounds.height() - max_height, panel.anchor.clone());
            }
        }
    }

    fn dfs(panel: &Panel, bounds_map: &mut HashMap<String, Bounds>) -> Bounds {
        let mut bounds = bounds_map.remove(&panel.id).unwrap();
        apply_constraints(panel, &mut bounds);
        bounds_map.insert(panel.id.clone(), bounds.clone());

        if let Some(children) = &panel.children {
            for child in children {
                let child_bounds = dfs(child, bounds_map);
                bounds.x2 = bounds.x2.max(child_bounds.x2);
                bounds.y2 = bounds.y2.max(child_bounds.y2);
            }
        }

        bounds
    }

    fn revalidate_children(
        panel: &Panel,
        bounds_map: &mut HashMap<String, Bounds>,
        parent_bounds: &Bounds,
    ) {
        if let Some(children) = &panel.children {
            for child in children {
                if let Some(child_bounds) = bounds_map.get_mut(&child.id) {
                    // Ensure child bounds are within parent bounds
                    if child_bounds.x2 > parent_bounds.x2 {
                        child_bounds.x2 = parent_bounds.x2;
                    }
                    if child_bounds.y2 > parent_bounds.y2 {
                        child_bounds.y2 = parent_bounds.y2;
                    }
                    if child_bounds.x1 < parent_bounds.x1 {
                        child_bounds.x1 = parent_bounds.x1;
                    }
                    if child_bounds.y1 < parent_bounds.y1 {
                        child_bounds.y1 = parent_bounds.y1;
                    }
                }
                revalidate_children(child, bounds_map, parent_bounds);
            }
        }
    }

    if let Some(children) = &layout.children {
        for panel in children {
            let parent_bounds = dfs(panel, &mut bounds_map);
            revalidate_children(panel, &mut bounds_map, &parent_bounds);
        }
    }

    bounds_map
}

pub fn calculate_bounds_map(app_graph: &AppGraph, layout: &Layout) -> HashMap<String, Bounds> {
    let bounds_map = calculate_initial_bounds(app_graph, layout);
    adjust_bounds_with_constraints(layout, bounds_map)
}

pub fn send_json_to_socket(socket_path: &str, json: &str) -> io::Result<String> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.write_all(json.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenbuffer_new() {
        let screen_buffer = ScreenBuffer::new_custom(5, 5);
        assert_eq!(screen_buffer.width, 5);
        assert_eq!(screen_buffer.height, 5);
        assert_eq!(screen_buffer.buffer.len(), 5);
        assert_eq!(screen_buffer.buffer[0].len(), 5);
    }

    #[test]
    fn test_screenbuffer_clear() {
        let mut screen_buffer = ScreenBuffer::new_custom(5, 5);
        let test_cell = Cell {
            fg_color: String::from("red"),
            bg_color: String::from("blue"),
            ch: 'X',
        };
        screen_buffer.update(2, 2, test_cell.clone());
        screen_buffer.clear();
        for row in screen_buffer.buffer.iter() {
            for cell in row.iter() {
                assert_eq!(cell.fg_color, get_fg_color("default"));
                assert_eq!(cell.bg_color, get_bg_color("default"));
                assert_eq!(cell.ch, ' ');
            }
        }
    }

    #[test]
    fn test_screenbuffer_update() {
        let mut screen_buffer = ScreenBuffer::new_custom(5, 5);
        let test_cell = Cell {
            fg_color: String::from("red"),
            bg_color: String::from("blue"),
            ch: 'X',
        };
        screen_buffer.update(2, 2, test_cell.clone());
        assert_eq!(screen_buffer.get(2, 2).unwrap(), &test_cell);
    }

    #[test]
    fn test_screenbuffer_get() {
        let screen_buffer = ScreenBuffer::new_custom(5, 5);
        assert!(screen_buffer.get(6, 6).is_none());
        assert!(screen_buffer.get(3, 3).is_some());
    }
}
