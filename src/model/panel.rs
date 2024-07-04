use crate::utils::{input_bounds_to_bounds, screen_bounds};
use core::hash::Hash;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::hash::Hasher;

use crate::model::common::*;
use crate::model::layout::Layout;

use crate::{utils::*, AppContext, AppGraph};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Panel {
    pub id: String,
    pub title: Option<String>,
    pub position: InputBounds,
    #[serde(default)]
    pub anchor: Anchor,
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
    pub max_width: Option<usize>,
    pub max_height: Option<usize>,
    pub overflow_behavior: Option<String>,
    #[serde(default)]
    pub scroll: bool,
    pub refresh_interval: Option<u64>,
    pub tab_order: Option<String>,
    pub next_focus_id: Option<String>,
    pub children: Option<Vec<Panel>>,
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
    pub script: Option<Vec<String>>,
    pub thread: Option<bool>,
    #[serde(default)]
    pub on_keypress: Option<HashMap<String, Vec<String>>>,
    pub horizontal_scroll: Option<f64>,
    pub vertical_scroll: Option<f64>,
    pub selected: Option<bool>,
    #[serde(skip)]
    pub content: Option<String>,
    #[serde(skip)]
    pub output: String,
    #[serde(skip)]
    pub parent_id: Option<String>,
    #[serde(skip)]
    pub parent_layout_id: Option<String>,
}

impl Hash for Panel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.title.hash(state);
        self.position.hash(state);
        self.anchor.hash(state);
        self.min_width.hash(state);
        self.min_height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);
        self.overflow_behavior.hash(state);
        self.content.hash(state);
        self.scroll.hash(state);
        self.refresh_interval.hash(state);
        self.tab_order.hash(state);
        self.next_focus_id.hash(state);
        if let Some(children) = &self.children {
            for child in children {
                child.hash(state);
            }
        }
        self.fill.hash(state);
        self.fill_char.hash(state);
        self.selected_fill_char.hash(state);
        self.border.hash(state);
        self.border_color.hash(state);
        self.selected_border_color.hash(state);
        self.bg_color.hash(state);
        self.selected_bg_color.hash(state);
        self.fg_color.hash(state);
        self.selected_fg_color.hash(state);
        self.title_fg_color.hash(state);
        self.title_bg_color.hash(state);
        self.selected_title_bg_color.hash(state);
        self.selected_title_fg_color.hash(state);
        self.title_position.hash(state);
        self.script.hash(state);
        self.thread.hash(state);
        self.output.hash(state);
        if let Some(hs) = self.horizontal_scroll {
            hs.to_bits().hash(state);
        }
        if let Some(vs) = self.vertical_scroll {
            vs.to_bits().hash(state);
        }
        self.selected.hash(state);
        self.parent_id.hash(state);
        self.parent_layout_id.hash(state);
    }
}

impl Default for Panel {
    fn default() -> Self {
        Panel {
            id: "".to_string(),
            title: None,
            position: InputBounds {
                x1: "0".to_string(),
                y1: "0".to_string(),
                x2: "0".to_string(),
                y2: "0".to_string(),
            },
            anchor: Anchor::Center,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            overflow_behavior: None,
            content: None,
            scroll: false,
            refresh_interval: None,
            tab_order: None,
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
            script: None,
            thread: Some(false),
            on_keypress: None,
            output: "".to_string(),
            horizontal_scroll: Some(0.0),
            vertical_scroll: Some(0.0),
            selected: Some(false),
            parent_id: None,
            parent_layout_id: None,
        }
    }
}

impl PartialEq for Panel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.title == other.title
            && self.position == other.position
            && self.min_width == other.min_width
            && self.min_height == other.min_height
            && self.max_width == other.max_width
            && self.max_height == other.max_height
            && self.overflow_behavior == other.overflow_behavior
            && self.content == other.content
            && self.scroll == other.scroll
            && self.refresh_interval == other.refresh_interval
            && self.tab_order == other.tab_order
            && self.next_focus_id == other.next_focus_id
            && self.children == other.children
            && self.fill == other.fill
            && self.fill_char == other.fill_char
            && self.selected_fill_char == other.selected_fill_char
            && self.border == other.border
            && self.border_color == other.border_color
            && self.selected_border_color == other.selected_border_color
            && self.bg_color == other.bg_color
            && self.selected_bg_color == other.selected_bg_color
            && self.fg_color == other.fg_color
            && self.selected_fg_color == other.selected_fg_color
            && self.title_fg_color == other.title_fg_color
            && self.title_bg_color == other.title_bg_color
            && self.selected_title_bg_color == other.selected_title_bg_color
            && self.selected_title_fg_color == other.selected_title_fg_color
            && self.title_position == other.title_position
            && self.script == other.script
            && self.thread == other.thread
            && self.horizontal_scroll.map(|hs| hs.to_bits())
                == other.horizontal_scroll.map(|hs| hs.to_bits())
            && self.vertical_scroll.map(|vs| vs.to_bits())
                == other.vertical_scroll.map(|vs| vs.to_bits())
            && self.selected == other.selected
            && self.parent_id == other.parent_id
            && self.parent_layout_id == other.parent_layout_id
            && self.output == other.output
    }
}

impl Eq for Panel {}

impl DeepClone for Panel {
    fn deep_clone(&self) -> Self {
        Panel {
            id: self.id.clone(),
            title: self.title.clone(),
            position: self.position.clone(),
            anchor: self.anchor.clone(),
            min_width: self.min_width,
            min_height: self.min_height,
            max_width: self.max_width,
            max_height: self.max_height,
            overflow_behavior: self.overflow_behavior.clone(),
            content: self.content.clone(),
            scroll: self.scroll,
            refresh_interval: self.refresh_interval,
            tab_order: self.tab_order.clone(),
            next_focus_id: self.next_focus_id.clone(),
            children: self
                .children
                .as_ref()
                .map(|children| children.iter().map(|panel| panel.deep_clone()).collect()),
            fill: self.fill,
            fill_char: self.fill_char,
            selected_fill_char: self.selected_fill_char,
            border: self.border,
            border_color: self.border_color.clone(),
            selected_border_color: self.selected_border_color.clone(),
            bg_color: self.bg_color.clone(),
            selected_bg_color: self.selected_bg_color.clone(),
            fg_color: self.fg_color.clone(),
            selected_fg_color: self.selected_fg_color.clone(),
            title_fg_color: self.title_fg_color.clone(),
            title_bg_color: self.title_bg_color.clone(),
            selected_title_bg_color: self.selected_title_bg_color.clone(),
            selected_title_fg_color: self.selected_title_fg_color.clone(),
            title_position: self.title_position.clone(),
            script: self.script.clone(),
            thread: self.thread,
            on_keypress: self.on_keypress.clone(),
            output: self.output.clone(),
            horizontal_scroll: self.horizontal_scroll,
            vertical_scroll: self.vertical_scroll,
            selected: self.selected,
            parent_id: self.parent_id.clone(),
            parent_layout_id: self.parent_layout_id.clone(),
        }
    }
}

impl Panel {
    pub fn bounds(&self) -> Bounds {
        input_bounds_to_bounds(&self.position, &screen_bounds())
    }

    pub fn absolute_bounds(&self, parent_bounds: Option<&Bounds>) -> Bounds {
        let screen_bounds_value = screen_bounds();
        let actual_parent_bounds = parent_bounds.unwrap_or(&screen_bounds_value);
        input_bounds_to_bounds(&self.position, actual_parent_bounds)
    }

    pub fn update_bounds_absolutely(&mut self, bounds: Bounds, parent_bounds: Option<&Bounds>) {
        let screen_bounds_value = screen_bounds();
        let actual_parent_bounds = parent_bounds.unwrap_or(&screen_bounds_value);
        self.position = bounds_to_input_bounds(&bounds, actual_parent_bounds);
    }

    pub fn set_output(&mut self, output: &str) {
        self.output = output.to_string();
    }

    pub fn get_parent_clone(&self, app_graph: &AppGraph) -> Option<Panel> {
        let layout_id = self
            .parent_layout_id
            .as_ref()
            .expect("Parent layout ID missing");
        if let Some(parent) = app_graph.get_parent(layout_id, &self.id) {
            Some(parent.clone()) // Clone the result to break the lifetime dependency
        } else {
            None
        }
    }

    pub fn get_parent_layout_clone(&self, app_context: &AppContext) -> Option<Layout> {
        let layout_id = self
            .parent_layout_id
            .as_ref()
            .expect("Parent layout ID missing");
        if let Some(parent_layout) = app_context.app.get_layout_by_id(layout_id) {
            Some(parent_layout.clone()) // Clone the result to break the lifetime dependency
        } else {
            None
        }
    }

    pub fn calc_fg_color<'a>(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.selected_fg_color.clone()
            } else {
                p.fg_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.selected_fg_color.clone()
            } else {
                pl.fg_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.selected_fg_color.as_ref()
        } else {
            self.fg_color.as_ref()
        };

        let default_color = if self.selected.unwrap_or(false) {
            "bright_white"
        } else {
            "white"
        };

        inherit_string(
            self_color,
            parent_color.as_ref(),
            parent_layout_color.as_ref(),
            default_color,
        )
    }

    pub fn calc_bg_color<'a>(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.selected_bg_color.clone()
            } else {
                p.bg_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.selected_bg_color.clone()
            } else {
                pl.bg_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.selected_bg_color.as_ref()
        } else {
            self.bg_color.as_ref()
        };

        let default_color = if self.selected.unwrap_or(false) {
            "bright_black"
        } else {
            "black"
        };

        inherit_string(
            self_color,
            parent_color.as_ref(),
            parent_layout_color.as_ref(),
            default_color,
        )
    }

    pub fn calc_border_color<'a>(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.selected_border_color.clone()
            } else {
                p.border_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.selected_border_color.clone()
            } else {
                pl.border_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.selected_border_color.as_ref()
        } else {
            self.border_color.as_ref()
        };

        let default_color = if self.selected.unwrap_or(false) {
            "bright_white"
        } else {
            "white"
        };

        inherit_string(
            self_color,
            parent_color.as_ref(),
            parent_layout_color.as_ref(),
            default_color,
        )
    }

    pub fn calc_title_bg_color<'a>(
        &self,
        app_context: &AppContext,
        app_graph: &AppGraph,
    ) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.selected_title_bg_color.clone()
            } else {
                p.title_bg_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.selected_title_bg_color.clone()
            } else {
                pl.title_bg_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.selected_title_bg_color.as_ref()
        } else {
            self.title_bg_color.as_ref()
        };

        let default_color = if self.selected.unwrap_or(false) {
            "black"
        } else {
            "bright_black"
        };

        inherit_string(
            self_color,
            parent_color.as_ref(),
            parent_layout_color.as_ref(),
            default_color,
        )
    }

    pub fn calc_title_fg_color(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.selected_title_fg_color.clone()
            } else {
                p.title_fg_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.selected_title_fg_color.clone()
            } else {
                pl.title_fg_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.selected_title_fg_color.as_ref()
        } else {
            self.title_fg_color.as_ref()
        };

        let default_color = if self.selected.unwrap_or(false) {
            "bright_white"
        } else {
            "white"
        };

        inherit_string(
            self_color,
            parent_color.as_ref(),
            parent_layout_color.as_ref(),
            default_color,
        )
    }

    pub fn calc_title_position(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_position = self
            .get_parent_clone(app_graph)
            .and_then(|p| p.title_position.clone());
        let parent_layout_position = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.title_position.clone());

        inherit_string(
            self.title_position.as_ref(),
            parent_position.as_ref(),
            parent_layout_position.as_ref(),
            "center",
        )
    }

    pub fn calc_fill_char(&self, app_context: &AppContext, app_graph: &AppGraph) -> char {
        let parent_fill_char = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.fill_char.clone()
            } else {
                p.fill_char.clone()
            }
        });

        let parent_layout_fill_char = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.selected_fill_char.clone()
            } else {
                pl.fill_char.clone()
            }
        });

        let self_char = if self.selected.unwrap_or(false) {
            self.selected_fill_char.as_ref()
        } else {
            self.fill_char.as_ref()
        };

        inherit_char(
            self_char,
            parent_fill_char.as_ref(),
            parent_layout_fill_char.as_ref(),
            '█',
        )
    }

    pub fn calc_border(&self, app_context: &AppContext, app_graph: &AppGraph) -> bool {
        let parent_border = self
            .get_parent_clone(app_graph)
            .and_then(|p| p.border.clone());
        let parent_layout_border = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.border.clone());

        inherit_bool(
            self.border.as_ref(),
            parent_border.as_ref(),
            parent_layout_border.as_ref(),
            true,
        )
    }

    pub fn calc_overflow_behavior(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_overflow_behavior = self
            .get_parent_clone(app_graph)
            .and_then(|p| p.overflow_behavior.clone());
        let parent_layout_overflow = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.overflow_behavior.clone());

        inherit_string(
            self.overflow_behavior.as_ref(),
            parent_overflow_behavior.as_ref(),
            parent_layout_overflow.as_ref(),
            "scroll",
        )
    }

    pub fn calc_refresh_interval(&self, app_context: &AppContext, app_graph: &AppGraph) -> u64 {
        let parent_refresh_interval = self
            .get_parent_clone(app_graph)
            .and_then(|p| p.refresh_interval.clone());
        let parent_layout_interval = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.refresh_interval.clone());

        inherit_u64(
            self.refresh_interval.as_ref(),
            parent_refresh_interval.as_ref(),
            parent_layout_interval.as_ref(),
            0,
        )
    }

    pub fn is_selectable(&self) -> bool {
        self.tab_order.is_some() && self.tab_order.as_ref().unwrap() != "none"
    }

    pub fn is_selected(&self) -> bool {
        self.selected.unwrap_or(false)
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
}

// Implement Updatable for Panel
impl Updatable for Panel {
    fn generate_diff(&self, other: &Self) -> Vec<FieldUpdate> {
        let mut updates = Vec::new();
        // Compare each field
        if self.title != other.title {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "title".to_string(),
                new_value: serde_json::to_value(&other.title).unwrap(),
            });
        }
        if self.position != other.position {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "position".to_string(),
                new_value: serde_json::to_value(&other.position).unwrap(),
            });
        }

        // Compare other fields similarly...
        if self.anchor != other.anchor {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "anchor".to_string(),
                new_value: serde_json::to_value(&other.anchor).unwrap(),
            });
        }

        if self.min_width != other.min_width {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "min_width".to_string(),
                new_value: serde_json::to_value(&other.min_width).unwrap(),
            });
        }

        if self.min_height != other.min_height {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "min_height".to_string(),
                new_value: serde_json::to_value(&other.min_height).unwrap(),
            });
        }

        if self.max_width != other.max_width {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "max_width".to_string(),
                new_value: serde_json::to_value(&other.max_width).unwrap(),
            });
        }

        if self.max_height != other.max_height {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "max_height".to_string(),
                new_value: serde_json::to_value(&other.max_height).unwrap(),
            });
        }

        if self.overflow_behavior != other.overflow_behavior {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),

                field_name: "overflow_behavior".to_string(),
                new_value: serde_json::to_value(&other.overflow_behavior).unwrap(),
            });
        }

        if self.scroll != other.scroll {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "scroll".to_string(),
                new_value: serde_json::to_value(&other.scroll).unwrap(),
            });
        }

        if self.refresh_interval != other.refresh_interval {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "refresh_interval".to_string(),
                new_value: serde_json::to_value(&other.refresh_interval).unwrap(),
            });
        }

        if self.tab_order != other.tab_order {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "tab_order".to_string(),
                new_value: serde_json::to_value(&other.tab_order).unwrap(),
            });
        }

        if self.next_focus_id != other.next_focus_id {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "next_focus_id".to_string(),
                new_value: serde_json::to_value(&other.next_focus_id).unwrap(),
            });
        }

        if self.children != other.children {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "children".to_string(),
                new_value: serde_json::to_value(&other.children).unwrap(),
            });
        }

        if self.fill != other.fill {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "fill".to_string(),
                new_value: serde_json::to_value(&other.fill).unwrap(),
            });
        }

        if self.fill_char != other.fill_char {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "fill_char".to_string(),
                new_value: serde_json::to_value(&other.fill_char).unwrap(),
            });
        }

        if self.selected_fill_char != other.selected_fill_char {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "selected_fill_char".to_string(),
                new_value: serde_json::to_value(&other.selected_fill_char).unwrap(),
            });
        }

        if self.border != other.border {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "border".to_string(),
                new_value: serde_json::to_value(&other.border).unwrap(),
            });
        }

        if self.border_color != other.border_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "border_color".to_string(),
                new_value: serde_json::to_value(&other.border_color).unwrap(),
            });
        }

        if self.selected_border_color != other.selected_border_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "selected_border_color".to_string(),
                new_value: serde_json::to_value(&other.selected_border_color).unwrap(),
            });
        }

        if self.bg_color != other.bg_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "bg_color".to_string(),
                new_value: serde_json::to_value(&other.bg_color).unwrap(),
            });
        }

        if self.selected_bg_color != other.selected_bg_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "selected_bg_color".to_string(),
                new_value: serde_json::to_value(&other.selected_bg_color).unwrap(),
            });
        }

        if self.fg_color != other.fg_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "fg_color".to_string(),
                new_value: serde_json::to_value(&other.fg_color).unwrap(),
            });
        }

        if self.selected_fg_color != other.selected_fg_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "selected_fg_color".to_string(),
                new_value: serde_json::to_value(&other.selected_fg_color).unwrap(),
            });
        }

        if self.title_fg_color != other.title_fg_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "title_fg_color".to_string(),
                new_value: serde_json::to_value(&other.title_fg_color).unwrap(),
            });
        }

        if self.title_bg_color != other.title_bg_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "title_bg_color".to_string(),
                new_value: serde_json::to_value(&other.title_bg_color).unwrap(),
            });
        }

        if self.selected_title_bg_color != other.selected_title_bg_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "selected_title_bg_color".to_string(),
                new_value: serde_json::to_value(&other.selected_title_bg_color).unwrap(),
            });
        }

        if self.selected_title_fg_color != other.selected_title_fg_color {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "selected_title_fg_color".to_string(),
                new_value: serde_json::to_value(&other.selected_title_fg_color).unwrap(),
            });
        }

        if self.title_position != other.title_position {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "title_position".to_string(),
                new_value: serde_json::to_value(&other.title_position).unwrap(),
            });
        }

        if self.script != other.script {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "script".to_string(),
                new_value: serde_json::to_value(&other.script).unwrap(),
            });
        }

        if self.thread != other.thread {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "thread".to_string(),
                new_value: serde_json::to_value(&other.thread).unwrap(),
            });
        }

        if self.on_keypress != other.on_keypress {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "on_keypress".to_string(),
                new_value: serde_json::to_value(&other.on_keypress).unwrap(),
            });
        }

        if self.horizontal_scroll != other.horizontal_scroll {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "horizontal_scroll".to_string(),

                new_value: serde_json::to_value(&other.horizontal_scroll).unwrap(),
            });
        }

        if self.vertical_scroll != other.vertical_scroll {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "vertical_scroll".to_string(),
                new_value: serde_json::to_value(&other.vertical_scroll).unwrap(),
            });
        }

        if self.selected != other.selected {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "selected".to_string(),
                new_value: serde_json::to_value(&other.selected).unwrap(),
            });
        }

        if self.content != other.content {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "content".to_string(),
                new_value: serde_json::to_value(&other.content).unwrap(),
            });
        }

        if self.output != other.output {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "output".to_string(),
                new_value: serde_json::to_value(&other.output).unwrap(),
            });
        }

        if self.parent_id != other.parent_id {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "parent_id".to_string(),
                new_value: serde_json::to_value(&other.parent_id).unwrap(),
            });
        }

        if self.parent_layout_id != other.parent_layout_id {
            updates.push(FieldUpdate {
                entity_id: Some(self.id.clone()),
                field_name: "parent_layout_id".to_string(),
                new_value: serde_json::to_value(&other.parent_layout_id).unwrap(),
            });
        }

        updates
    }

    fn apply_updates(&mut self, updates: &[FieldUpdate]) {
        for update in updates {
            if let Some(ref entity_id) = update.entity_id {
                if entity_id != &self.id {
                    continue;
                }
            }
            match update.field_name.as_str() {
                "title" => {
                    if let Ok(new_title) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.title = new_title;
                    }
                }
                "position" => {
                    if let Ok(new_position) =
                        serde_json::from_value::<InputBounds>(update.new_value.clone())
                    {
                        self.position = new_position;
                    }
                }
                // Handle other fields similarly...
                "anchor" => {
                    if let Ok(new_anchor) =
                        serde_json::from_value::<Anchor>(update.new_value.clone())
                    {
                        self.anchor = new_anchor;
                    }
                }
                "min_width" => {
                    if let Ok(new_min_width) =
                        serde_json::from_value::<Option<usize>>(update.new_value.clone())
                    {
                        self.min_width = new_min_width;
                    }
                }
                "min_height" => {
                    if let Ok(new_min_height) =
                        serde_json::from_value::<Option<usize>>(update.new_value.clone())
                    {
                        self.min_height = new_min_height;
                    }
                }
                "max_width" => {
                    if let Ok(new_max_width) =
                        serde_json::from_value::<Option<usize>>(update.new_value.clone())
                    {
                        self.max_width = new_max_width;
                    }
                }
                "max_height" => {
                    if let Ok(new_max_height) =
                        serde_json::from_value::<Option<usize>>(update.new_value.clone())
                    {
                        self.max_height = new_max_height;
                    }
                }
                "overflow_behavior" => {
                    if let Ok(new_overflow_behavior) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.overflow_behavior = new_overflow_behavior;
                    }
                }
                "scroll" => {
                    if let Ok(new_scroll) = serde_json::from_value::<bool>(update.new_value.clone())
                    {
                        self.scroll = new_scroll;
                    }
                }
                "refresh_interval" => {
                    if let Ok(new_refresh_interval) =
                        serde_json::from_value::<Option<u64>>(update.new_value.clone())
                    {
                        self.refresh_interval = new_refresh_interval;
                    }
                }
                "tab_order" => {
                    if let Ok(new_tab_order) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.tab_order = new_tab_order;
                    }
                }
                "next_focus_id" => {
                    if let Ok(new_next_focus_id) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.next_focus_id = new_next_focus_id;
                    }
                }
                "children" => {
                    if let Ok(new_children) =
                        serde_json::from_value::<Option<Vec<Panel>>>(update.new_value.clone())
                    {
                        self.children = new_children;
                    }
                }
                "fill" => {
                    if let Ok(new_fill) =
                        serde_json::from_value::<Option<bool>>(update.new_value.clone())
                    {
                        self.fill = new_fill;
                    }
                }
                "fill_char" => {
                    if let Ok(new_fill_char) =
                        serde_json::from_value::<Option<char>>(update.new_value.clone())
                    {
                        self.fill_char = new_fill_char;
                    }
                }
                "selected_fill_char" => {
                    if let Ok(new_selected_fill_char) =
                        serde_json::from_value::<Option<char>>(update.new_value.clone())
                    {
                        self.selected_fill_char = new_selected_fill_char;
                    }
                }
                "border" => {
                    if let Ok(new_border) =
                        serde_json::from_value::<Option<bool>>(update.new_value.clone())
                    {
                        self.border = new_border;
                    }
                }
                "border_color" => {
                    if let Ok(new_border_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.border_color = new_border_color;
                    }
                }
                "selected_border_color" => {
                    if let Ok(new_selected_border_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.selected_border_color = new_selected_border_color;
                    }
                }
                "bg_color" => {
                    if let Ok(new_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.bg_color = new_bg_color;
                    }
                }
                "selected_bg_color" => {
                    if let Ok(new_selected_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.selected_bg_color = new_selected_bg_color;
                    }
                }
                "fg_color" => {
                    if let Ok(new_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.fg_color = new_fg_color;
                    }
                }
                "selected_fg_color" => {
                    if let Ok(new_selected_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.selected_fg_color = new_selected_fg_color;
                    }
                }
                "title_fg_color" => {
                    if let Ok(new_title_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.title_fg_color = new_title_fg_color;
                    }
                }
                "title_bg_color" => {
                    if let Ok(new_title_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.title_bg_color = new_title_bg_color;
                    }
                }
                "selected_title_bg_color" => {
                    if let Ok(new_selected_title_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.selected_title_bg_color = new_selected_title_bg_color;
                    }
                }
                "selected_title_fg_color" => {
                    if let Ok(new_selected_title_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.selected_title_fg_color = new_selected_title_fg_color;
                    }
                }
                "title_position" => {
                    if let Ok(new_title_position) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.title_position = new_title_position;
                    }
                }
                "script" => {
                    if let Ok(new_script) =
                        serde_json::from_value::<Option<Vec<String>>>(update.new_value.clone())
                    {
                        self.script = new_script;
                    }
                }
                "thread" => {
                    if let Ok(new_thread) =
                        serde_json::from_value::<Option<bool>>(update.new_value.clone())
                    {
                        self.thread = new_thread;
                    }
                }
                "on_keypress" => {
                    if let Ok(new_on_keypress) = serde_json::from_value::<
                        Option<HashMap<String, Vec<String>>>,
                    >(update.new_value.clone())
                    {
                        self.on_keypress = new_on_keypress;
                    }
                }
                "output" => {
                    if let Ok(new_output) =
                        serde_json::from_value::<String>(update.new_value.clone())
                    {
                        self.output = new_output;
                    }
                }
                "horizontal_scroll" => {
                    if let Ok(new_horizontal_scroll) =
                        serde_json::from_value::<Option<f64>>(update.new_value.clone())
                    {
                        self.horizontal_scroll = new_horizontal_scroll;
                    }
                }
                "vertical_scroll" => {
                    if let Ok(new_vertical_scroll) =
                        serde_json::from_value::<Option<f64>>(update.new_value.clone())
                    {
                        self.vertical_scroll = new_vertical_scroll;
                    }
                }
                "selected" => {
                    if let Ok(new_selected) =
                        serde_json::from_value::<Option<bool>>(update.new_value.clone())
                    {
                        self.selected = new_selected;
                    }
                }
                "parent_id" => {
                    if let Ok(new_parent_id) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.parent_id = new_parent_id;
                    }
                }
                "parent_layout_id" => {
                    if let Ok(new_parent_layout_id) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.parent_layout_id = new_parent_layout_id;
                    }
                }
                _ => log::warn!("Unknown field name for Panel: {}", update.field_name),
            }
        }
    }
}
