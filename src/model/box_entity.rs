use crate::state::SELECTED_BOX;
use crate::utils::{
    draw_box, fill_box, get_bg_color, get_fg_color, input_bounds_to_bounds, screen_bounds,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;

use serde::{de, ser};

use crate::model::common::*;
use crate::model::layout::Layout;

use crate::model::box_entity::BoxEntityWrapper;
use crate::thread_manager::{Data, Runnable};

pub struct BoxRunnable {
    box_entity: BoxEntityWrapper,
}

impl BoxRunnable {
    pub fn new(box_entity: BoxEntityWrapper) -> Self {
        BoxRunnable { box_entity }
    }
}

impl Runnable for BoxRunnable {
    fn run(&mut self, data: Data) {
        let mut box_guard = self.box_entity.0.lock().unwrap();
        match data.value {
            0 => box_guard.execute_refresh(),
            1 => box_guard.execute_enter(),
            2 => box_guard.execute_leave(),
            _ => {}
        }
    }
}

// Newtype wrapper for Arc<Mutex<BoxEntity>>
#[derive(Debug, Clone)]
pub struct BoxEntityWrapper(pub Arc<Mutex<BoxEntity>>);

impl PartialEq<BoxEntity> for BoxEntityWrapper {
    fn eq(&self, other: &BoxEntity) -> bool {
        self.0.lock().unwrap().id == other.id
    }
}

impl PartialEq<BoxEntityWrapper> for BoxEntity {
    fn eq(&self, other: &BoxEntityWrapper) -> bool {
        self.id == other.0.lock().unwrap().id
    }
}

impl PartialEq<BoxEntityWrapper> for BoxEntityWrapper {
    fn eq(&self, other: &BoxEntityWrapper) -> bool {
        self.0.lock().unwrap().id == other.0.lock().unwrap().id
    }
}

// Implement Serialize for BoxEntityWrapper
impl Serialize for BoxEntityWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let lock = self.0.lock().unwrap();
        (*lock).serialize(serializer)
    }
}

// Implement Deserialize for BoxEntityWrapper
impl<'de> Deserialize<'de> for BoxEntityWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let inner = BoxEntity::deserialize(deserializer)?;
        Ok(BoxEntityWrapper(Arc::new(Mutex::new(inner))))
    }
}

// Custom serialization and deserialization functions
pub fn serialize_arc_mutex_vec_bew<S>(
    arc_mutex: &Option<Vec<BoxEntityWrapper>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    arc_mutex.serialize(serializer)
}

pub fn serialize_arc_mutex_bew<S>(
    arc_mutex: &Option<BoxEntityWrapper>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    arc_mutex.serialize(serializer)
}

pub fn deserialize_arc_mutex_vec_bew<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<BoxEntityWrapper>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::deserialize(deserializer)
}

pub fn deserialize_arc_mutex_bew<'de, D>(
    deserializer: D,
) -> Result<Option<BoxEntityWrapper>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::deserialize(deserializer)
}

// Use the custom serialize and deserialize functions in the struct
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    #[serde(
        serialize_with = "serialize_arc_mutex_vec_bew",
        deserialize_with = "deserialize_arc_mutex_vec_bew",
        default
    )]
    pub children: Option<Vec<BoxEntityWrapper>>,
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
    #[serde(skip)]
    pub parent: Option<BoxEntityWrapper>,
    pub parent_layout: Option<Box<Layout>>,
    #[serde(skip)]
    pub horizontal_scroll: Option<f64>,
    #[serde(skip)]
    pub vertical_scroll: Option<f64>,
}

impl Default for BoxEntity {
    fn default() -> Self {
        BoxEntity {
            id: "".to_string(),
            title: None,
            position: InputBounds {
                x1: "0".to_string(),
                y1: "0".to_string(),
                x2: "0".to_string(),
                y2: "0".to_string(),
            },
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            overflow_behavior: None,
            content: None,
            scroll: false,
            refresh_interval: None,
            tab_order: None,
            on_error: None,
            on_enter: None,
            on_leave: None,
            next_focus_id: None,
            children: None,
            fill: None,
            fill_char: None,
            selected_fill_char: None,
            border: None,
            border_color: None,
            selected_border_color: None,
            bg_color: None,
            selected_bg_color: None,
            fg_color: None,
            selected_fg_color: None,
            title_fg_color: None,
            title_bg_color: None,
            selected_title_bg_color: None,
            selected_title_fg_color: None,
            title_position: None,
            on_refresh: None,
            thread: None,
            output: "".to_string(),
            parent: None,
            parent_layout: None,
            horizontal_scroll: None,
            vertical_scroll: None,
        }
    }
}

impl PartialEq for BoxEntity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl BoxEntity {
    pub fn start_event_thread(&self, thread_manager: &mut ThreadManager) {
        if self.has_refresh() {
            let runnable = BoxRunnable::new(BoxEntityWrapper::clone(self));
            thread_manager.spawn_thread(runnable);
        }

        if self.has_enter() || self.has_leave() {
            let runnable = BoxRunnable::new(BoxEntityWrapper::clone(self));
            thread_manager.spawn_thread(runnable);
        }
    }

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
    pub fn calc_border_color(&self) -> String {
        if self.is_selected() {
            if let Some(ref selected_border_color) = self.selected_border_color {
                if !selected_border_color.is_empty() {
                    return selected_border_color.clone();
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(selected_border_color) = &parent.0.lock().unwrap().selected_border_color
                {
                    if !selected_border_color.is_empty() {
                        return selected_border_color.clone();
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(selected_border_color) = &parent_layout.selected_border_color {
                    if !selected_border_color.is_empty() {
                        return selected_border_color.clone();
                    }
                }
            }
        } else {
            if let Some(ref border_color) = self.border_color {
                return border_color.clone();
            }
            if let Some(parent) = &self.parent {
                if let Some(border_color) = &parent.0.lock().unwrap().border_color {
                    return border_color.clone();
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(border_color) = &parent_layout.border_color {
                    return border_color.clone();
                }
            }
        }
        "default".to_string()
    }

    pub fn calc_fg_color(&self) -> String {
        if self.is_selected() {
            if let Some(ref selected_fg_color) = self.selected_fg_color {
                if !selected_fg_color.is_empty() {
                    return selected_fg_color.clone();
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(selected_fg_color) = &parent.0.lock().unwrap().selected_fg_color {
                    if !selected_fg_color.is_empty() {
                        return selected_fg_color.clone();
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(selected_fg_color) = &parent_layout.selected_fg_color {
                    if !selected_fg_color.is_empty() {
                        return selected_fg_color.clone();
                    }
                }
            }
        } else {
            if let Some(ref fg_color) = self.fg_color {
                return fg_color.clone();
            }
            if let Some(parent) = &self.parent {
                if let Some(fg_color) = &parent.0.lock().unwrap().fg_color {
                    return fg_color.clone();
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(fg_color) = &parent_layout.fg_color {
                    return fg_color.clone();
                }
            }
        }
        "default".to_string()
    }

    pub fn calc_bg_color(&self) -> String {
        log::debug!("Calculating background color for box '{}'", self.id);

        if self.is_selected() {
            log::debug!("Box '{}' is selected", self.id);
            if let Some(ref selected_bg_color) = self.selected_bg_color {
                if !selected_bg_color.is_empty() {
                    log::debug!(
                        "Using box's selected background color: '{}'",
                        selected_bg_color
                    );
                    return selected_bg_color.clone();
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(selected_bg_color) = &parent.0.lock().unwrap().selected_bg_color {
                    if !selected_bg_color.is_empty() {
                        log::debug!(
                            "Using parent's selected background color: '{}'",
                            selected_bg_color
                        );
                        return selected_bg_color.clone();
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(selected_bg_color) = &parent_layout.selected_bg_color {
                    if !selected_bg_color.is_empty() {
                        log::debug!(
                            "Using parent layout's selected background color: '{}'",
                            selected_bg_color
                        );
                        return selected_bg_color.clone();
                    }
                }
            }
        } else {
            log::debug!("Box '{}' is not selected", self.id);
            if let Some(ref bg_color) = self.bg_color {
                log::debug!("Using box's background color: '{}'", bg_color);
                return bg_color.clone();
            }
            if let Some(parent) = &self.parent {
                if let Some(bg_color) = &parent.0.lock().unwrap().bg_color {
                    log::debug!("Using parent's background color: '{}'", bg_color);
                    return bg_color.clone();
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(bg_color) = &parent_layout.bg_color {
                    log::debug!("Using parent layout's background color: '{}'", bg_color);
                    return bg_color.clone();
                }
            }
        }

        log::debug!("Using default background color");
        "default".to_string()
    }

    pub fn calc_title_bg_color(&self) -> String {
        if self.is_selected() {
            if let Some(ref selected_title_bg_color) = self.selected_title_bg_color {
                if !selected_title_bg_color.is_empty() {
                    return selected_title_bg_color.clone();
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(selected_title_bg_color) =
                    &parent.0.lock().unwrap().selected_title_bg_color
                {
                    if !selected_title_bg_color.is_empty() {
                        return selected_title_bg_color.clone();
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(selected_title_bg_color) = &parent_layout.selected_title_bg_color {
                    if !selected_title_bg_color.is_empty() {
                        return selected_title_bg_color.clone();
                    }
                }
            }
        } else {
            if let Some(ref title_bg_color) = self.title_bg_color {
                return title_bg_color.clone();
            }
            if let Some(parent) = &self.parent {
                if let Some(title_bg_color) = &parent.0.lock().unwrap().title_bg_color {
                    return title_bg_color.clone();
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(title_bg_color) = &parent_layout.title_bg_color {
                    return title_bg_color.clone();
                }
            }
        }
        "default".to_string()
    }

    pub fn calc_title_fg_color(&self) -> String {
        if self.is_selected() {
            if let Some(ref selected_title_fg_color) = self.selected_title_fg_color {
                if !selected_title_fg_color.is_empty() {
                    return selected_title_fg_color.clone();
                }
            }
            if let Some(parent) = &self.parent {
                if let Some(selected_title_fg_color) =
                    &parent.0.lock().unwrap().selected_title_fg_color
                {
                    if !selected_title_fg_color.is_empty() {
                        return selected_title_fg_color.clone();
                    }
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(selected_title_fg_color) = &parent_layout.selected_title_fg_color {
                    if !selected_title_fg_color.is_empty() {
                        return selected_title_fg_color.clone();
                    }
                }
            }
        } else {
            if let Some(ref title_fg_color) = self.title_fg_color {
                return title_fg_color.clone();
            }
            if let Some(parent) = &self.parent {
                if let Some(title_fg_color) = &parent.0.lock().unwrap().title_fg_color {
                    return title_fg_color.clone();
                }
            }
            if let Some(parent_layout) = &self.parent_layout {
                if let Some(title_fg_color) = &parent_layout.title_fg_color {
                    return title_fg_color.clone();
                }
            }
        }
        "default".to_string()
    }

    pub fn calc_title_position(&self) -> String {
        if let Some(title_position) = &self.title_position {
            return title_position.clone();
        }
        if let Some(parent) = &self.parent {
            if let Some(title_position) = parent.0.lock().unwrap().title_position.clone() {
                return title_position;
            }
        }
        if let Some(parent_layout) = &self.parent_layout {
            if let Some(title_position) = parent_layout.title_position.clone() {
                return title_position;
            }
        }
        "start".to_string()
    }

    pub fn calc_fill_char(&self) -> char {
        if self.is_selected() {
            if let Some(selected_fill_char) = self.selected_fill_char {
                return selected_fill_char;
            }
            if let Some(parent) = &self.parent {
                if let Some(selected_fill_char) = parent.0.lock().unwrap().selected_fill_char {
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
                if let Some(fill_char) = parent.0.lock().unwrap().fill_char {
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
            if let Some(border) = parent.0.lock().unwrap().border {
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

    pub fn calc_overflow_behavior(&self) -> String {
        if let Some(ref overflow_behavior) = self.overflow_behavior {
            return overflow_behavior.clone();
        }
        if let Some(parent) = &self.parent {
            if let Some(overflow_behavior) = &parent.0.lock().unwrap().overflow_behavior {
                return overflow_behavior.clone();
            }
        }
        if let Some(parent_layout) = &self.parent_layout {
            if let Some(overflow_behavior) = &parent_layout.overflow_behavior {
                return overflow_behavior.clone();
            }
        }
        "scroll".to_string()
    }

    pub fn calc_refresh_interval(&self) -> Option<u64> {
        if let Some(refresh_interval) = self.refresh_interval {
            return Some(refresh_interval);
        }
        if let Some(parent) = &self.parent {
            if let Some(refresh_interval) = parent.0.lock().unwrap().refresh_interval {
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
        log::debug!("Drawing box '{}'", self.id);
        let parent_bounds = if self.parent.is_none() {
            Some(screen_bounds())
        } else {
            Some(self.parent.as_ref().unwrap().0.lock().unwrap().bounds())
        };

        // Calculate properties before borrowing self mutably
        let bounds = self.absolute_bounds(parent_bounds.as_ref());
        let bg_color = self.calc_bg_color().to_string();
        let parent_bg_color = if self.parent.is_none() {
            "default".to_string()
        } else {
            self.parent
                .as_ref()
                .unwrap()
                .0
                .lock()
                .unwrap()
                .calc_bg_color()
                .to_string()
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
            &self.calc_title_position(),
            content,
            &fg_color,
            &self.calc_overflow_behavior(),
            self.current_horizontal_scroll(),
            self.current_vertical_scroll(),
            screen,
            buffer,
        );

        // Draw children
        if let Some(children) = &mut self.children {
            for child in children {
                child.0.lock().unwrap().draw(screen, buffer);
            }
        }
        log::debug!("Finished drawing box '{}'", self.id);
    }

    pub fn is_selectable(&self) -> bool {
        self.tab_order.is_some() && self.tab_order.as_ref().unwrap() != "none"
    }

    pub fn is_selected(&self) -> bool {
        *SELECTED_BOX.lock().unwrap() == Some(BoxEntityWrapper(Arc::new(Mutex::new(self.clone()))))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;
    use std::sync::Mutex;

    // Test serialization and deserialization of BoxEntityWrapper
    #[test]
    fn test_serialize_deserialize_box_entity_wrapper() {
        let box_entity = BoxEntity {
            id: String::from("box1"),
            title: Some(String::from("Test Box")),
            position: InputBounds {
                x1: String::from("0"),
                y1: String::from("0"),
                x2: String::from("10"),
                y2: String::from("10"),
            },
            min_width: Some(10),
            min_height: Some(10),
            max_width: Some(20),
            max_height: Some(20),
            overflow_behavior: Some(String::from("scroll")),
            content: Some(String::from("Content")),
            scroll: true,
            refresh_interval: Some(1),
            tab_order: Some(String::from("1")),
            on_error: None,
            on_enter: None,
            on_leave: None,
            next_focus_id: None,
            children: None,
            fill: None,
            fill_char: None,
            selected_fill_char: None,
            border: Some(true),
            border_color: Some(String::from("white")),
            selected_border_color: Some(String::from("red")),
            bg_color: Some(String::from("black")),
            selected_bg_color: Some(String::from("bright_black")),
            fg_color: Some(String::from("white")),
            selected_fg_color: Some(String::from("bright_white")),
            title_fg_color: Some(String::from("green")),
            title_bg_color: Some(String::from("blue")),
            selected_title_bg_color: Some(String::from("yellow")),
            selected_title_fg_color: Some(String::from("cyan")),
            title_position: Some(String::from("start")),
            on_refresh: None,
            thread: Some(false),
            output: String::new(),
            parent: None,
            parent_layout: None,
            horizontal_scroll: Some(0.0),
            vertical_scroll: Some(0.0),
        };

        let box_entity_wrapper = BoxEntityWrapper(Arc::new(Mutex::new(box_entity.clone())));

        // Serialize
        let serialized = serde_json::to_string(&box_entity_wrapper).unwrap();
        // Deserialize
        let deserialized: BoxEntityWrapper = serde_json::from_str(&serialized).unwrap();

        // Check if the original and deserialized objects are equal
        assert_eq!(box_entity_wrapper, deserialized);
    }

    // Test scrolling methods
    #[test]
    fn test_scroll_methods() {
        let mut box_entity = BoxEntity {
            id: String::from("box1"),
            // Initialize other fields as needed
            ..Default::default()
        };

        box_entity.scroll_down(Some(10.0));
        assert_eq!(box_entity.current_vertical_scroll(), 10.0);

        box_entity.scroll_up(Some(5.0));
        assert_eq!(box_entity.current_vertical_scroll(), 5.0);

        box_entity.scroll_right(Some(10.0));
        assert_eq!(box_entity.current_horizontal_scroll(), 10.0);

        box_entity.scroll_left(Some(5.0));
        assert_eq!(box_entity.current_horizontal_scroll(), 5.0);
    }

    // Test color calculation methods
    #[test]
    fn test_color_calculations() {
        let box_entity = BoxEntity {
            id: String::from("box1"),
            selected_bg_color: Some(String::from("red")),
            bg_color: Some(String::from("blue")),
            ..Default::default()
        };

        assert_eq!(box_entity.calc_bg_color(), "blue");

        // Mock selected state
        let selected_box = BoxEntityWrapper(Arc::new(Mutex::new(box_entity.clone())));
        *SELECTED_BOX.lock().unwrap() = Some(selected_box);
        assert_eq!(box_entity.calc_bg_color(), "red");
    }
}
