use crate::model::common::*;
use crate::model::layout::Layout;
use crate::utils::{input_bounds_to_bounds, screen_bounds};
use core::hash::Hash;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::hash::Hasher;
use std::io::Write;

use crate::{utils::*, AppContext, AppGraph};

/// Flexible deserializer for script fields that handles:
/// - Single string (split on newlines)
/// - Array of strings
/// - Mixed array with YAML literal blocks
fn deserialize_script<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ScriptFormat {
        Single(String),
        Multiple(Vec<String>),
        Mixed(Vec<serde_yaml::Value>),
    }

    let script_format = Option::<ScriptFormat>::deserialize(deserializer)?;
    
    match script_format {
        None => Ok(None),
        Some(ScriptFormat::Single(single)) => {
            // Split single string on newlines, filter empty lines
            let commands: Vec<String> = single
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            Ok(Some(commands))
        }
        Some(ScriptFormat::Multiple(multiple)) => Ok(Some(multiple)),
        Some(ScriptFormat::Mixed(mixed)) => {
            // Handle mixed array with literal blocks and simple strings
            let mut commands = Vec::new();
            for value in mixed {
                match value {
                    serde_yaml::Value::String(s) => commands.push(s),
                    serde_yaml::Value::Mapping(_) | serde_yaml::Value::Sequence(_) => {
                        // Convert complex YAML structures to string representation
                        if let Ok(yaml_str) = serde_yaml::to_string(&value) {
                            // For literal blocks, extract the actual content
                            let clean_str = yaml_str.trim_start_matches("---\n").trim().to_string();
                            if !clean_str.is_empty() {
                                commands.push(clean_str);
                            }
                        }
                    }
                    _ => {
                        // For other types, convert to string
                        commands.push(format!("{:?}", value));
                    }
                }
            }
            Ok(Some(commands))
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Choice {
    pub id: String,
    pub content: Option<String>,
    #[serde(deserialize_with = "deserialize_script", default)]
    pub script: Option<Vec<String>>,
    pub thread: Option<bool>,
    pub redirect_output: Option<String>,
    pub append_output: Option<bool>,
    #[serde(skip, default)]
    pub selected: bool,
    #[serde(skip, default)]
    pub waiting: bool,
}

impl Hash for Choice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.content.hash(state);
        self.script.hash(state);
        self.thread.hash(state);
        self.redirect_output.hash(state);
        self.append_output.hash(state);
        self.selected.hash(state);
        self.waiting.hash(state);
    }
}

impl PartialEq for Choice {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.content == other.content
            && self.script == other.script
            && self.thread == other.thread
            && self.redirect_output == other.redirect_output
            && self.append_output == other.append_output
            && self.selected == other.selected
            && self.waiting == other.waiting
    }
}

impl Eq for Choice {}

impl Clone for Choice {
    fn clone(&self) -> Self {
        Choice {
            id: self.id.clone(),
            content: self.content.clone(),
            script: self.script.clone(),
            thread: self.thread,
            redirect_output: self.redirect_output.clone(),
            append_output: self.append_output,
            selected: self.selected,
            waiting: self.waiting,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
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
    pub error_border_color: Option<String>,
    pub error_bg_color: Option<String>,
    pub error_fg_color: Option<String>,
    pub error_title_bg_color: Option<String>,
    pub error_title_fg_color: Option<String>,
    pub error_selected_border_color: Option<String>,
    pub error_selected_bg_color: Option<String>,
    pub error_selected_fg_color: Option<String>,
    pub error_selected_title_bg_color: Option<String>,
    pub error_selected_title_fg_color: Option<String>,
    pub choices: Option<Vec<Choice>>,
    pub menu_fg_color: Option<String>,
    pub menu_bg_color: Option<String>,
    pub selected_menu_fg_color: Option<String>,
    pub selected_menu_bg_color: Option<String>,
    pub redirect_output: Option<String>,
    pub append_output: Option<bool>,
    #[serde(deserialize_with = "deserialize_script", default)]
    pub script: Option<Vec<String>>,
    pub thread: Option<bool>,
    #[serde(default)]
    pub on_keypress: Option<HashMap<String, Vec<String>>>,
    #[serde(default)]
    pub variables: Option<HashMap<String, String>>,
    pub horizontal_scroll: Option<f64>,
    pub vertical_scroll: Option<f64>,
    pub selected: Option<bool>,
    pub content: Option<String>,
    pub save_in_file: Option<String>,
    #[serde(skip)]
    pub output: String,
    #[serde(skip)]
    pub parent_id: Option<String>,
    #[serde(skip)]
    pub parent_layout_id: Option<String>,
    #[serde(skip, default)]
    pub error_state: bool,
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
        self.menu_fg_color.hash(state);
        self.menu_bg_color.hash(state);
        self.selected_menu_fg_color.hash(state);
        self.selected_menu_bg_color.hash(state);
        self.error_border_color.hash(state);
        self.error_bg_color.hash(state);
        self.error_fg_color.hash(state);
        self.error_title_bg_color.hash(state);
        self.error_title_fg_color.hash(state);
        self.error_selected_border_color.hash(state);
        self.error_selected_bg_color.hash(state);
        self.error_selected_fg_color.hash(state);
        self.error_selected_title_bg_color.hash(state);
        self.error_selected_title_fg_color.hash(state);
        if let Some(choices) = &self.choices {
            for choice in choices {
                choice.hash(state);
            }
        }
        self.redirect_output.hash(state);
        self.append_output.hash(state);
        self.script.hash(state);
        self.thread.hash(state);
        self.output.hash(state);
        self.save_in_file.hash(state);
        if let Some(hs) = self.horizontal_scroll {
            hs.to_bits().hash(state);
        }
        if let Some(vs) = self.vertical_scroll {
            vs.to_bits().hash(state);
        }
        self.selected.hash(state);
        self.parent_id.hash(state);
        self.parent_layout_id.hash(state);
        self.error_state.hash(state);
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
            error_border_color: None,
            error_bg_color: None,
            error_fg_color: None,
            error_title_bg_color: None,
            error_title_fg_color: None,
            error_selected_border_color: None,
            error_selected_bg_color: None,
            error_selected_fg_color: None,
            error_selected_title_bg_color: None,
            error_selected_title_fg_color: None,
            choices: None,
            menu_fg_color: None,
            menu_bg_color: None,
            selected_menu_fg_color: None,
            selected_menu_bg_color: None,
            redirect_output: None,
            append_output: None,
            script: None,
            thread: Some(false),
            on_keypress: None,
            variables: None,
            output: "".to_string(),
            save_in_file: None,
            horizontal_scroll: Some(0.0),
            vertical_scroll: Some(0.0),
            selected: Some(false),
            parent_id: None,
            parent_layout_id: None,
            error_state: false,
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
            && self.error_border_color == other.error_border_color
            && self.error_bg_color == other.error_bg_color
            && self.error_fg_color == other.error_fg_color
            && self.error_title_bg_color == other.error_title_bg_color
            && self.error_title_fg_color == other.error_title_fg_color
            && self.error_selected_border_color == other.error_selected_border_color
            && self.error_selected_bg_color == other.error_selected_bg_color
            && self.error_selected_fg_color == other.error_selected_fg_color
            && self.error_selected_title_bg_color == other.error_selected_title_bg_color
            && self.error_selected_title_fg_color == other.error_selected_title_fg_color
            && self.choices == other.choices
            && self.menu_fg_color == other.menu_fg_color
            && self.menu_bg_color == other.menu_bg_color
            && self.selected_menu_fg_color == other.selected_menu_fg_color
            && self.selected_menu_bg_color == other.selected_menu_bg_color
            && self.redirect_output == other.redirect_output
            && self.append_output == other.append_output
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
            && self.save_in_file == other.save_in_file
            && self.error_state == other.error_state
    }
}

impl Eq for Panel {}

impl Clone for Panel {
    fn clone(&self) -> Self {
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
            children: self.children.as_ref().map(|children| children.to_vec()),
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
            error_border_color: self.error_border_color.clone(),
            error_bg_color: self.error_bg_color.clone(),
            error_fg_color: self.error_fg_color.clone(),
            error_title_bg_color: self.error_title_bg_color.clone(),
            error_title_fg_color: self.error_title_fg_color.clone(),
            error_selected_border_color: self.error_selected_border_color.clone(),
            error_selected_bg_color: self.error_selected_bg_color.clone(),
            error_selected_fg_color: self.error_selected_fg_color.clone(),
            error_selected_title_bg_color: self.error_selected_title_bg_color.clone(),
            error_selected_title_fg_color: self.error_selected_title_fg_color.clone(),
            choices: self.choices.clone(),
            menu_fg_color: self.menu_fg_color.clone(),
            menu_bg_color: self.menu_bg_color.clone(),
            selected_menu_fg_color: self.selected_menu_fg_color.clone(),
            selected_menu_bg_color: self.selected_menu_bg_color.clone(),
            redirect_output: self.redirect_output.clone(),
            append_output: self.append_output,
            script: self.script.clone(),
            thread: self.thread,
            on_keypress: self.on_keypress.clone(),
            variables: self.variables.clone(),
            output: self.output.clone(),
            save_in_file: self.save_in_file.clone(),
            horizontal_scroll: self.horizontal_scroll,
            vertical_scroll: self.vertical_scroll,
            selected: self.selected,
            parent_id: self.parent_id.clone(),
            parent_layout_id: self.parent_layout_id.clone(),
            error_state: self.error_state,
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
        app_graph.get_parent(layout_id, &self.id).cloned()
    }

    pub fn get_parent_layout_clone(&self, app_context: &AppContext) -> Option<Layout> {
        let layout_id = self
            .parent_layout_id
            .as_ref()
            .expect("Parent layout ID missing");
        app_context.app.get_layout_by_id(layout_id).cloned()
    }

    pub fn calc_fg_color<'a>(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        if self.error_state {
            return self.calc_error_fg_color(app_context, app_graph);
        }

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
        if self.error_state {
            return self.calc_error_bg_color(app_context, app_graph);
        }

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
        if self.error_state {
            return self.calc_error_border_color(app_context, app_graph);
        }

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
        if self.error_state {
            return self.calc_error_title_bg_color(app_context, app_graph);
        }

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
        if self.error_state {
            return self.calc_error_title_fg_color(app_context, app_graph);
        }

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

    pub fn calc_menu_fg_color(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_position = self
            .get_parent_clone(app_graph)
            .and_then(|p| p.menu_fg_color.clone());
        let parent_layout_position = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.menu_fg_color.clone());

        inherit_string(
            self.menu_fg_color.as_ref(),
            parent_position.as_ref(),
            parent_layout_position.as_ref(),
            "bright_white",
        )
    }

    pub fn calc_menu_bg_color(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_position = self
            .get_parent_clone(app_graph)
            .and_then(|p| p.menu_bg_color.clone());
        let parent_layout_position = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.menu_bg_color.clone());

        inherit_string(
            self.menu_bg_color.as_ref(),
            parent_position.as_ref(),
            parent_layout_position.as_ref(),
            "black",
        )
    }

    pub fn calc_selected_menu_fg_color(
        &self,
        app_context: &AppContext,
        app_graph: &AppGraph,
    ) -> String {
        let parent_position = self
            .get_parent_clone(app_graph)
            .and_then(|p| p.selected_menu_fg_color.clone());
        let parent_layout_position = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.selected_menu_fg_color.clone());

        inherit_string(
            self.selected_menu_fg_color.as_ref(),
            parent_position.as_ref(),
            parent_layout_position.as_ref(),
            "bright_white",
        )
    }

    pub fn calc_selected_menu_bg_color(
        &self,
        app_context: &AppContext,
        app_graph: &AppGraph,
    ) -> String {
        let parent_position = self
            .get_parent_clone(app_graph)
            .and_then(|p| p.selected_menu_bg_color.clone());
        let parent_layout_position = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.selected_menu_bg_color.clone());

        inherit_string(
            self.selected_menu_bg_color.as_ref(),
            parent_position.as_ref(),
            parent_layout_position.as_ref(),
            "red",
        )
    }

    pub fn calc_error_fg_color(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.error_selected_fg_color.clone()
            } else {
                p.error_fg_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.error_selected_fg_color.clone()
            } else {
                pl.error_fg_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.error_selected_fg_color.as_ref()
        } else {
            self.error_fg_color.as_ref()
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

    pub fn calc_error_bg_color(&self, app_context: &AppContext, app_graph: &AppGraph) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.error_selected_bg_color.clone()
            } else {
                p.error_bg_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.error_selected_bg_color.clone()
            } else {
                pl.error_bg_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.error_selected_bg_color.as_ref()
        } else {
            self.error_bg_color.as_ref()
        };

        let default_color = if self.selected.unwrap_or(false) {
            "bright_red"
        } else {
            "red"
        };

        inherit_string(
            self_color,
            parent_color.as_ref(),
            parent_layout_color.as_ref(),
            default_color,
        )
    }

    pub fn calc_error_title_fg_color(
        &self,
        app_context: &AppContext,
        app_graph: &AppGraph,
    ) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.error_selected_title_fg_color.clone()
            } else {
                p.error_title_fg_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.error_selected_title_fg_color.clone()
            } else {
                pl.error_title_fg_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.error_selected_title_fg_color.as_ref()
        } else {
            self.error_title_fg_color.as_ref()
        };

        let default_color = if self.selected.unwrap_or(false) {
            "bright_red"
        } else {
            "red"
        };

        inherit_string(
            self_color,
            parent_color.as_ref(),
            parent_layout_color.as_ref(),
            default_color,
        )
    }

    pub fn calc_error_title_bg_color(
        &self,
        app_context: &AppContext,
        app_graph: &AppGraph,
    ) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.error_selected_title_bg_color.clone()
            } else {
                p.error_title_bg_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.error_selected_title_bg_color.clone()
            } else {
                pl.error_title_bg_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.error_selected_title_bg_color.as_ref()
        } else {
            self.error_title_bg_color.as_ref()
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

    pub fn calc_error_border_color(
        &self,
        app_context: &AppContext,
        app_graph: &AppGraph,
    ) -> String {
        let parent_color = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.error_selected_border_color.clone()
            } else {
                p.error_border_color.clone()
            }
        });

        let parent_layout_color = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.error_selected_border_color.clone()
            } else {
                pl.error_border_color.clone()
            }
        });

        let self_color = if self.selected.unwrap_or(false) {
            self.error_selected_border_color.as_ref()
        } else {
            self.error_border_color.as_ref()
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

    pub fn calc_fill_char(&self, app_context: &AppContext, app_graph: &AppGraph) -> char {
        let parent_fill_char = self.get_parent_clone(app_graph).and_then(|p| {
            if self.selected.unwrap_or(false) {
                p.fill_char
            } else {
                p.fill_char
            }
        });

        let parent_layout_fill_char = self.get_parent_layout_clone(app_context).and_then(|pl| {
            if self.selected.unwrap_or(false) {
                pl.selected_fill_char
            } else {
                pl.fill_char
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
        let parent_border = self.get_parent_clone(app_graph).and_then(|p| p.border);
        let parent_layout_border = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.border);

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
            .and_then(|p| p.refresh_interval);
        let parent_layout_interval = self
            .get_parent_layout_clone(app_context)
            .and_then(|pl| pl.refresh_interval);

        inherit_u64(
            self.refresh_interval.as_ref(),
            parent_refresh_interval.as_ref(),
            parent_layout_interval.as_ref(),
            0,
        )
    }

    pub fn is_selectable(&self) -> bool {
        self.tab_order.is_some() 
            && self.tab_order.as_ref().unwrap() != "none"
            && !self.tab_order.as_ref().unwrap().is_empty()
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

    fn generate_children_diff(&self, other: &Self) -> Vec<FieldUpdate> {
        let mut updates = Vec::new();

        // Get references to children, defaulting to an empty slice if None
        let self_children = self.children.as_deref().unwrap_or(&[]);
        let other_children = other.children.as_deref().unwrap_or(&[]);
        // Compare each pair of children
        for self_child in self_children {
            for other_child in other_children {
                if self_child.id == other_child.id {
                    updates.extend(self_child.generate_diff(other_child));
                }
            }
        }

        // Handle extra children in other
        if self_children.len() < other_children.len() {
            for other_child in &other_children[self_children.len()..] {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(other_child.id.clone()),
                    field_name: "children".to_string(),
                    new_value: serde_json::to_value(other_child).unwrap(),
                });
            }
        }

        // Handle extra children in self
        if self_children.len() > other_children.len() {
            for self_child in &self_children[other_children.len()..] {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self_child.id.clone()),
                    field_name: "children".to_string(),
                    new_value: Value::Null, // Representing removal
                });
            }
        }

        updates
    }

    fn apply_children_updates(&mut self, updates: Vec<FieldUpdate>) {
        for update in updates {
            if update.entity_type != EntityType::Panel {
                continue;
            }
            if let Some(entity_id) = &update.entity_id {
                // Check if the update is for a child panel
                if self.children.as_ref().map_or(false, |children| {
                    children.iter().any(|p| p.id == *entity_id)
                }) {
                    // Find the child panel and apply the update
                    if let Some(child_panel) = self
                        .children
                        .as_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|p| p.id == *entity_id)
                    {
                        child_panel.apply_updates(vec![FieldUpdate {
                            entity_type: EntityType::Panel,
                            entity_id: Some(child_panel.id.clone()),
                            field_name: update.field_name.clone(),
                            new_value: update.new_value.clone(),
                        }]);
                    }
                }
            }

            // If the entity_id matches the parent itself and field is "children", apply to all children
            if update.field_name == "children" {
                match update.new_value {
                    Value::Null => {
                        // Removing all children
                        self.children = None;
                    }
                    _ => {
                        if let Ok(new_children) =
                            serde_json::from_value::<Vec<Panel>>(update.new_value.clone())
                        {
                            if self.children.is_none() {
                                // Assign new children
                                self.children = Some(new_children);
                            } else {
                                let self_children = self.children.as_mut().unwrap();
                                for new_child in new_children {
                                    if let Some(existing_child) =
                                        self_children.iter_mut().find(|p| p.id == new_child.id)
                                    {
                                        // Update existing child
                                        *existing_child = new_child;
                                    } else {
                                        // Add new child
                                        self_children.push(new_child);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn update_content(&mut self, new_content: &str, append_content: bool, success: bool) {
        // Preserve current scroll position
        let preserved_horizontal_scroll = self.horizontal_scroll;
        let preserved_vertical_scroll = self.vertical_scroll;
        
        let mut formatted_content = new_content.to_string();
        if append_content {
            if let Some(self_content) = &self.content {
                formatted_content = format!(
                    "[{}]\n\n{}\n\n\n\n{}",
                    chrono::Local::now().to_rfc2822(),
                    new_content,
                    self_content
                );
            } else {
                formatted_content =
                    format!("[{}]\n\n{}", chrono::Local::now().to_rfc2822(), new_content);
            }
        }
        self.content = Some(formatted_content);
        self.error_state = !success;
        
        // Restore scroll position after content update
        self.horizontal_scroll = preserved_horizontal_scroll;
        self.vertical_scroll = preserved_vertical_scroll;

        if self.save_in_file.is_some() {
            //include date
            let formatted_content_for_file =
                format!("[{}]\n\n{}", chrono::Local::now().to_rfc2822(), new_content);
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(append_content)
                .open(self.save_in_file.clone().unwrap())
                .unwrap();
            writeln!(file, "{}", formatted_content_for_file).unwrap();
        }
    }
}

impl Updatable for Panel {
    fn generate_diff(&self, other: &Self) -> Vec<FieldUpdate> {
        let mut updates = Vec::new();
        // Compare each field and add to updates if not null and different
        if self.title != other.title {
            if let Some(new_value) = &other.title {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()),
                    field_name: "title".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.position != other.position {
            updates.push(FieldUpdate {
                entity_type: EntityType::Panel,
                entity_id: Some(self.id.clone()),
                field_name: "position".to_string(),
                new_value: serde_json::to_value(&other.position).unwrap(),
            });
        }

        if self.anchor != other.anchor {
            updates.push(FieldUpdate {
                entity_type: EntityType::Panel,
                entity_id: Some(self.id.clone()),
                field_name: "anchor".to_string(),
                new_value: serde_json::to_value(&other.anchor).unwrap(),
            });
        }

        if self.min_height != other.min_height {
            if let Some(new_value) = other.min_height {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "min_height".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.min_width != other.min_width {
            if let Some(new_value) = other.min_width {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "min_width".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.max_height != other.max_height {
            if let Some(new_value) = other.max_height {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "max_height".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.max_width != other.max_width {
            if let Some(new_value) = other.max_width {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "max_width".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.overflow_behavior != other.overflow_behavior {
            if let Some(new_value) = &other.overflow_behavior {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "overflow_behavior".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.scroll != other.scroll {
            updates.push(FieldUpdate {
                entity_type: EntityType::Panel,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "scroll".to_string(),
                new_value: serde_json::to_value(other.scroll).unwrap(),
            });
        }

        if self.refresh_interval != other.refresh_interval {
            if let Some(new_value) = other.refresh_interval {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "refresh_interval".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.tab_order != other.tab_order {
            if let Some(new_value) = &other.tab_order {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "tab_order".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.next_focus_id != other.next_focus_id {
            if let Some(new_value) = &other.next_focus_id {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "next_focus_id".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        updates.extend(self.generate_children_diff(other));

        if self.fill != other.fill {
            updates.push(FieldUpdate {
                entity_type: EntityType::Panel,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "fill".to_string(),
                new_value: serde_json::to_value(other.fill).unwrap(),
            });
        }

        if self.fill_char != other.fill_char {
            if let Some(new_value) = other.fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "fill_char".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_fill_char != other.selected_fill_char {
            if let Some(new_value) = other.selected_fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_fill_char".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.border != other.border {
            updates.push(FieldUpdate {
                entity_type: EntityType::Panel,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "border".to_string(),
                new_value: serde_json::to_value(other.border).unwrap(),
            });
        }

        if self.border_color != other.border_color {
            if let Some(new_value) = &other.border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_border_color != other.selected_border_color {
            if let Some(new_value) = &other.selected_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.bg_color != other.bg_color {
            if let Some(new_value) = &other.bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_bg_color != other.selected_bg_color {
            if let Some(new_value) = &other.selected_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.fg_color != other.fg_color {
            if let Some(new_value) = &other.fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_fg_color != other.selected_fg_color {
            if let Some(new_value) = &other.selected_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_fg_color != other.title_fg_color {
            if let Some(new_value) = &other.title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_bg_color != other.title_bg_color {
            if let Some(new_value) = &other.title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_title_bg_color != other.selected_title_bg_color {
            if let Some(new_value) = &other.selected_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_title_fg_color != other.selected_title_fg_color {
            if let Some(new_value) = &other.selected_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_position != other.title_position {
            if let Some(new_value) = &other.title_position {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "title_position".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_state != other.error_state {
            updates.push(FieldUpdate {
                entity_type: EntityType::Panel,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "error_state".to_string(),
                new_value: serde_json::to_value(other.error_state).unwrap(),
            });
        }

        if self.error_fg_color != other.error_fg_color {
            if let Some(new_value) = &other.error_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_bg_color != other.error_bg_color {
            if let Some(new_value) = &other.error_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_title_fg_color != other.error_title_fg_color {
            if let Some(new_value) = &other.error_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_title_bg_color != other.error_title_bg_color {
            if let Some(new_value) = &other.error_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_border_color != other.error_border_color {
            if let Some(new_value) = &other.error_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_bg_color != other.error_selected_bg_color {
            if let Some(new_value) = &other.error_selected_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_fg_color != other.error_selected_fg_color {
            if let Some(new_value) = &other.error_selected_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_border_color != other.error_selected_border_color {
            if let Some(new_value) = &other.error_selected_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_title_bg_color != other.error_selected_title_bg_color {
            if let Some(new_value) = &other.error_selected_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_title_fg_color != other.error_selected_title_fg_color {
            if let Some(new_value) = &other.error_selected_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.choices != other.choices {
            if let Some(new_value) = &other.choices {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "choices".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.menu_fg_color != other.menu_fg_color {
            if let Some(new_value) = &other.menu_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "menu_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.menu_bg_color != other.menu_bg_color {
            if let Some(new_value) = &other.menu_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "menu_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_menu_fg_color != other.selected_menu_fg_color {
            if let Some(new_value) = &other.selected_menu_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_menu_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_menu_bg_color != other.selected_menu_bg_color {
            if let Some(new_value) = &other.selected_menu_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_menu_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.redirect_output != other.redirect_output {
            if let Some(new_value) = &other.redirect_output {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "redirect_output".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.script != other.script {
            if let Some(new_value) = &other.script {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "script".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.thread != other.thread {
            updates.push(FieldUpdate {
                entity_type: EntityType::Panel,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "thread".to_string(),
                new_value: serde_json::to_value(other.thread).unwrap(),
            });
        }

        if self.on_keypress != other.on_keypress {
            if let Some(new_value) = &other.on_keypress {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "on_keypress".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.horizontal_scroll != other.horizontal_scroll {
            if let Some(new_value) = other.horizontal_scroll {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "horizontal_scroll".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.vertical_scroll != other.vertical_scroll {
            if let Some(new_value) = other.vertical_scroll {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "vertical_scroll".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected != other.selected {
            if let Some(new_value) = other.selected {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.content != other.content {
            if let Some(new_value) = &other.content {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "content".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.output != other.output {
            updates.push(FieldUpdate {
                entity_type: EntityType::Panel,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "output".to_string(),
                new_value: serde_json::to_value(&other.output).unwrap(),
            });
        }

        if self.parent_id != other.parent_id {
            if let Some(new_value) = &other.parent_id {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "parent_id".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.parent_layout_id != other.parent_layout_id {
            if let Some(new_value) = &other.parent_layout_id {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Panel,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "parent_layout_id".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        updates
    }

    fn apply_updates(&mut self, updates: Vec<FieldUpdate>) {
        let updates_for_children = updates.clone();
        for update in updates {
            if update.entity_type != EntityType::Panel {
                continue;
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
                "anchor" => {
                    if let Ok(new_anchor) =
                        serde_json::from_value::<Anchor>(update.new_value.clone())
                    {
                        self.anchor = new_anchor;
                    }
                }
                "min_height" => {
                    if let Ok(new_min_height) =
                        serde_json::from_value::<Option<usize>>(update.new_value.clone())
                    {
                        self.min_height = new_min_height;
                    }
                }
                "min_width" => {
                    if let Ok(new_min_width) =
                        serde_json::from_value::<Option<usize>>(update.new_value.clone())
                    {
                        self.min_width = new_min_width;
                    }
                }
                "max_height" => {
                    if let Ok(new_max_height) =
                        serde_json::from_value::<Option<usize>>(update.new_value.clone())
                    {
                        self.max_height = new_max_height;
                    }
                }
                "max_width" => {
                    if let Ok(new_max_width) =
                        serde_json::from_value::<Option<usize>>(update.new_value.clone())
                    {
                        self.max_width = new_max_width;
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
                "children" => {}
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
                "error_state" => {
                    if let Ok(new_error_state) =
                        serde_json::from_value::<Option<bool>>(update.new_value.clone())
                    {
                        self.error_state = new_error_state.unwrap();
                    }
                }
                "error_fg_color" => {
                    if let Ok(new_error_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_fg_color = new_error_fg_color;
                    }
                }
                "error_bg_color" => {
                    if let Ok(new_error_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_bg_color = new_error_bg_color;
                    }
                }
                "error_title_fg_color" => {
                    if let Ok(new_error_title_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_title_fg_color = new_error_title_fg_color;
                    }
                }
                "error_title_bg_color" => {
                    if let Ok(new_error_title_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_title_bg_color = new_error_title_bg_color;
                    }
                }
                "error_border_color" => {
                    if let Ok(new_error_border_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_border_color = new_error_border_color;
                    }
                }
                "error_selected_bg_color" => {
                    if let Ok(new_error_selected_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_selected_bg_color = new_error_selected_bg_color;
                    }
                }
                "error_selected_fg_color" => {
                    if let Ok(new_error_selected_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_selected_fg_color = new_error_selected_fg_color;
                    }
                }
                "error_selected_border_color" => {
                    if let Ok(new_error_selected_border_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_selected_border_color = new_error_selected_border_color;
                    }
                }
                "error_selected_title_bg_color" => {
                    if let Ok(new_error_selected_title_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_selected_title_bg_color = new_error_selected_title_bg_color;
                    }
                }
                "error_selected_title_fg_color" => {
                    if let Ok(new_error_selected_title_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.error_selected_title_fg_color = new_error_selected_title_fg_color;
                    }
                }
                "choices" => {
                    if let Ok(new_choices) =
                        serde_json::from_value::<Option<Vec<Choice>>>(update.new_value.clone())
                    {
                        self.choices = new_choices;
                    }
                }
                "menu_fg_color" => {
                    if let Ok(new_menu_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.menu_fg_color = new_menu_fg_color;
                    }
                }
                "menu_bg_color" => {
                    if let Ok(new_menu_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.menu_bg_color = new_menu_bg_color;
                    }
                }
                "selected_menu_fg_color" => {
                    if let Ok(new_selected_menu_fg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.selected_menu_fg_color = new_selected_menu_fg_color;
                    }
                }
                "selected_menu_bg_color" => {
                    if let Ok(new_selected_menu_bg_color) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.selected_menu_bg_color = new_selected_menu_bg_color;
                    }
                }
                "redirect_output" => {
                    if let Ok(new_redirect_output) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.redirect_output = new_redirect_output;
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
                "content" => {
                    if let Ok(new_content) =
                        serde_json::from_value::<Option<String>>(update.new_value.clone())
                    {
                        self.content = new_content;
                    }
                }
                "output" => {
                    if let Ok(new_output) =
                        serde_json::from_value::<String>(update.new_value.clone())
                    {
                        self.output = new_output;
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

                // Handle other fields similarly...
                _ => log::warn!("Unknown field name for Panel: {}", update.field_name),
            }
        }
        self.apply_children_updates(updates_for_children);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::app::{App, AppContext};
    use crate::model::layout::Layout;
    use crate::Config;

    // === Helper Functions ===

    /// Creates a basic test panel with minimal required fields.
    /// This helper demonstrates how to create a Panel for testing purposes.
    fn create_test_panel(id: &str) -> Panel {
        Panel {
            id: id.to_string(),
            title: Some(format!("Test Panel {}", id)),
            position: InputBounds {
                x1: "0%".to_string(),
                y1: "0%".to_string(),
                x2: "100%".to_string(),
                y2: "100%".to_string(),
            },
            ..Default::default()
        }
    }

    /// Creates a test Choice with the given id and content.
    /// This helper demonstrates how to create a Choice for testing purposes.
    fn create_test_choice(id: &str, content: &str) -> Choice {
        Choice {
            id: id.to_string(),
            content: Some(content.to_string()),
            script: None,
            thread: Some(false),
            redirect_output: None,
            append_output: None,
            selected: false,
            waiting: false,
        }
    }

    /// Creates a test AppContext with a simple layout for testing.
    /// This helper demonstrates how to create an AppContext for Panel testing.
    fn create_test_app_context() -> AppContext {
        let mut app = App::new();
        let layout = Layout {
            id: "test_layout".to_string(),
            children: Some(vec![create_test_panel("test_panel")]),
            ..Default::default()
        };
        app.layouts.push(layout);
        AppContext::new(app, Config::default())
    }

    // === Panel Default Tests ===

    /// Tests that Panel::default() creates a panel with expected default values.
    /// This test demonstrates the default Panel construction behavior.
    #[test]
    fn test_panel_default() {
        let panel = Panel::default();
        assert_eq!(panel.id, "");
        assert_eq!(panel.title, None);
        assert_eq!(panel.scroll, false);
        assert_eq!(panel.anchor, Anchor::Center);
        assert_eq!(panel.selected, Some(false));
        assert_eq!(panel.thread, Some(false));
        assert_eq!(panel.horizontal_scroll, Some(0.0));
        assert_eq!(panel.vertical_scroll, Some(0.0));
        assert_eq!(panel.error_state, false);
        assert_eq!(panel.output, "");
    }

    // === Panel Creation Tests ===

    /// Tests creating a Panel with specific values.
    /// This test demonstrates how to create a Panel with custom properties.
    #[test]
    fn test_panel_creation() {
        let panel = Panel {
            id: "test_panel".to_string(),
            title: Some("Test Panel".to_string()),
            position: InputBounds {
                x1: "10%".to_string(),
                y1: "20%".to_string(),
                x2: "90%".to_string(),
                y2: "80%".to_string(),
            },
            scroll: true,
            anchor: Anchor::TopLeft,
            selected: Some(true),
            content: Some("Test content".to_string()),
            ..Default::default()
        };
        
        assert_eq!(panel.id, "test_panel");
        assert_eq!(panel.title, Some("Test Panel".to_string()));
        assert_eq!(panel.scroll, true);
        assert_eq!(panel.anchor, Anchor::TopLeft);
        assert_eq!(panel.selected, Some(true));
        assert_eq!(panel.content, Some("Test content".to_string()));
    }

    // === Panel Bounds Tests ===

    /// Tests that Panel::bounds() calculates bounds correctly.
    /// This test demonstrates the bounds calculation feature using screen bounds.
    #[test]
    fn test_panel_bounds() {
        let panel = Panel {
            id: "test".to_string(),
            position: InputBounds {
                x1: "25%".to_string(),
                y1: "50%".to_string(),
                x2: "75%".to_string(),
                y2: "100%".to_string(),
            },
            ..Default::default()
        };
        
        // Get screen bounds and calculated bounds
        let bounds = panel.bounds();
        let screen_bounds = crate::utils::screen_bounds();
        
        // Calculate expected values using the same logic as parse_percentage
        let expected_x1 = (0.25 * screen_bounds.width() as f64).round() as usize;
        let expected_y1 = (0.50 * screen_bounds.height() as f64).round() as usize;
        let expected_x2 = (0.75 * screen_bounds.width() as f64).round() as usize;
        let expected_y2 = (1.00 * screen_bounds.height() as f64).round() as usize;
        
        // Test that bounds are calculated correctly relative to screen size
        assert_eq!(bounds.x1, expected_x1, "x1 should be 25% of screen width");
        assert_eq!(bounds.y1, expected_y1, "y1 should be 50% of screen height");
        assert_eq!(bounds.x2, expected_x2, "x2 should be 75% of screen width");
        assert_eq!(bounds.y2, expected_y2, "y2 should be 100% of screen height");
        
        // Also test that bounds are within screen bounds
        assert!(bounds.x1 <= screen_bounds.width());
        assert!(bounds.y1 <= screen_bounds.height());
        assert!(bounds.x2 <= screen_bounds.width());
        assert!(bounds.y2 <= screen_bounds.height());
        
        // Test that bounds are logically consistent
        assert!(bounds.x1 < bounds.x2, "x1 should be less than x2");
        assert!(bounds.y1 < bounds.y2, "y1 should be less than y2");
    }

    /// Tests that Panel::absolute_bounds() works with parent bounds.
    /// This test demonstrates the absolute bounds calculation feature.
    #[test]
    fn test_panel_absolute_bounds() {
        let panel = Panel {
            id: "test".to_string(),
            position: InputBounds {
                x1: "25%".to_string(),
                y1: "50%".to_string(),
                x2: "75%".to_string(),
                y2: "100%".to_string(),
            },
            ..Default::default()
        };
        
        let parent_bounds = Bounds::new(0, 0, 100, 200);
        let bounds = panel.absolute_bounds(Some(&parent_bounds));
        
        assert_eq!(bounds.x1, 25);
        assert_eq!(bounds.y1, 100);
        assert_eq!(bounds.x2, 75);
        assert_eq!(bounds.y2, 200);
    }

    /// Tests that Panel::update_bounds_absolutely() updates position correctly.
    /// This test demonstrates the bounds update feature.
    #[test]
    fn test_panel_update_bounds_absolutely() {
        let mut panel = Panel {
            id: "test".to_string(),
            position: InputBounds {
                x1: "0%".to_string(),
                y1: "0%".to_string(),
                x2: "100%".to_string(),
                y2: "100%".to_string(),
            },
            ..Default::default()
        };
        
        let new_bounds = Bounds::new(25, 50, 75, 100);
        let parent_bounds = Bounds::new(0, 0, 100, 200);
        
        panel.update_bounds_absolutely(new_bounds, Some(&parent_bounds));
        
        assert_eq!(panel.position.x1, "25%");
        assert_eq!(panel.position.y1, "25%");
        assert_eq!(panel.position.x2, "75%");
        assert_eq!(panel.position.y2, "50%");
    }

    // === Panel Output Tests ===

    /// Tests that Panel::set_output() updates the output field.
    /// This test demonstrates the output setting feature.
    #[test]
    fn test_panel_set_output() {
        let mut panel = Panel::default();
        assert_eq!(panel.output, "");
        
        panel.set_output("Test output");
        assert_eq!(panel.output, "Test output");
    }

    /// Tests that Panel::update_content() updates content and error state.
    /// This test demonstrates the content update feature.
    #[test]
    fn test_panel_update_content() {
        let mut panel = Panel::default();
        
        // Test successful update
        panel.update_content("New content", false, true);
        assert_eq!(panel.content, Some("New content".to_string()));
        assert_eq!(panel.error_state, false);
        
        // Test error update
        panel.update_content("Error content", false, false);
        assert_eq!(panel.content, Some("Error content".to_string()));
        assert_eq!(panel.error_state, true);
    }

    /// Tests that Panel::update_content() handles content appending.
    /// This test demonstrates the content appending feature.
    #[test]
    fn test_panel_update_content_append() {
        let mut panel = Panel {
            content: Some("Existing content".to_string()),
            ..Default::default()
        };
        
        panel.update_content("New content", true, true);
        
        // Check that content was appended with timestamp
        let content = panel.content.unwrap();
        assert!(content.contains("New content"));
        assert!(content.contains("Existing content"));
        assert!(content.contains("[")); // Timestamp format
    }

    /// Tests that Panel::update_content() preserves scroll position during content updates.
    /// This test demonstrates the scroll position preservation feature.
    #[test]
    fn test_panel_update_content_preserves_scroll_position() {
        let mut panel = Panel {
            content: Some("Original content".to_string()),
            horizontal_scroll: Some(25.0),
            vertical_scroll: Some(75.0),
            ..Default::default()
        };
        
        // Update content and verify scroll position is preserved
        panel.update_content("Updated content", false, true);
        
        assert_eq!(panel.content, Some("Updated content".to_string()));
        assert_eq!(panel.horizontal_scroll, Some(25.0));
        assert_eq!(panel.vertical_scroll, Some(75.0));
        assert_eq!(panel.error_state, false);
    }

    /// Tests that Panel::update_content() preserves scroll position during append operations.
    /// This test demonstrates scroll preservation with content appending.
    #[test]
    fn test_panel_update_content_append_preserves_scroll_position() {
        let mut panel = Panel {
            content: Some("Existing content".to_string()),
            horizontal_scroll: Some(10.0),
            vertical_scroll: Some(50.0),
            ..Default::default()
        };
        
        // Append content and verify scroll position is preserved
        panel.update_content("New content", true, true);
        
        // Check scroll position preservation
        assert_eq!(panel.horizontal_scroll, Some(10.0));
        assert_eq!(panel.vertical_scroll, Some(50.0));
        assert_eq!(panel.error_state, false);
        
        // Verify content was appended
        let content = panel.content.unwrap();
        assert!(content.contains("New content"));
        assert!(content.contains("Existing content"));
    }

    /// Tests that Panel::update_content() preserves scroll position even when panel has no initial scroll values.
    /// This test demonstrates scroll preservation with None values.
    #[test]
    fn test_panel_update_content_preserves_none_scroll_values() {
        let mut panel = Panel {
            content: Some("Original content".to_string()),
            horizontal_scroll: None,
            vertical_scroll: None,
            ..Default::default()
        };
        
        // Update content and verify None scroll values are preserved
        panel.update_content("Updated content", false, true);
        
        assert_eq!(panel.content, Some("Updated content".to_string()));
        assert_eq!(panel.horizontal_scroll, None);
        assert_eq!(panel.vertical_scroll, None);
    }

    /// Tests that Panel scrolling supports larger amounts for page-based scrolling.
    /// This test demonstrates the page scrolling feature with larger scroll amounts.
    #[test]
    fn test_panel_page_scrolling_large_amounts() {
        let mut panel = Panel {
            vertical_scroll: Some(50.0),
            horizontal_scroll: Some(30.0),
            ..Default::default()
        };
        
        // Test page down (large scroll down)
        panel.scroll_down(Some(10.0));
        assert_eq!(panel.current_vertical_scroll(), 60.0);
        
        // Test page up (large scroll up)
        panel.scroll_up(Some(10.0));
        assert_eq!(panel.current_vertical_scroll(), 50.0);
        
        // Test page scrolling doesn't exceed bounds
        panel.vertical_scroll = Some(95.0);
        panel.scroll_down(Some(10.0));
        assert_eq!(panel.current_vertical_scroll(), 100.0); // Capped at 100
        
        panel.vertical_scroll = Some(5.0);
        panel.scroll_up(Some(10.0));
        assert_eq!(panel.current_vertical_scroll(), 0.0); // Capped at 0
    }

    /// Tests that Panel page scrolling works correctly with boundary conditions.
    /// This test demonstrates page scrolling boundary handling.
    #[test]
    fn test_panel_page_scrolling_boundaries() {
        let mut panel = Panel::default();
        
        // Test page down from initial position (None -> Some)
        panel.scroll_down(Some(10.0));
        assert_eq!(panel.current_vertical_scroll(), 10.0);
        
        // Test page up from initial position (None -> Some)
        panel.vertical_scroll = None;
        panel.scroll_up(Some(10.0));
        assert_eq!(panel.current_vertical_scroll(), 0.0); // Can't go below 0
        
        // Test large page movements near boundaries
        panel.vertical_scroll = Some(98.0);
        panel.scroll_down(Some(10.0));
        assert_eq!(panel.current_vertical_scroll(), 100.0); // Should cap at 100
        
        panel.vertical_scroll = Some(2.0);
        panel.scroll_up(Some(10.0));
        assert_eq!(panel.current_vertical_scroll(), 0.0); // Should cap at 0
    }

    // === Panel Selection Tests ===

    /// Tests that Panel::is_selectable() correctly identifies selectable panels.
    /// This test demonstrates the panel selectability feature.
    #[test]
    fn test_panel_is_selectable() {
        let mut panel = Panel::default();
        
        // Panel without tab_order is not selectable
        assert!(!panel.is_selectable());
        
        // Panel with tab_order "none" is not selectable
        panel.tab_order = Some("none".to_string());
        assert!(!panel.is_selectable());
        
        // Panel with numeric tab_order is selectable
        panel.tab_order = Some("1".to_string());
        assert!(panel.is_selectable());
    }

    /// Tests that Panel::is_selected() correctly identifies selected panels.
    /// This test demonstrates the panel selection state feature.
    #[test]
    fn test_panel_is_selected() {
        let mut panel = Panel::default();
        
        // Default panel is not selected
        assert!(!panel.is_selected());
        
        // Panel with selected = Some(true) is selected
        panel.selected = Some(true);
        assert!(panel.is_selected());
        
        // Panel with selected = Some(false) is not selected
        panel.selected = Some(false);
        assert!(!panel.is_selected());
    }

    // === Panel Scrolling Tests ===

    /// Tests that Panel::scroll_down() increases vertical scroll.
    /// This test demonstrates the downward scrolling feature.
    #[test]
    fn test_panel_scroll_down() {
        let mut panel = Panel::default();
        
        // Initial scroll should be 0
        assert_eq!(panel.current_vertical_scroll(), 0.0);
        
        // Scroll down by default amount
        panel.scroll_down(None);
        assert_eq!(panel.current_vertical_scroll(), 5.0);
        
        // Scroll down by custom amount
        panel.scroll_down(Some(10.0));
        assert_eq!(panel.current_vertical_scroll(), 15.0);
        
        // Scroll should not exceed 100%
        panel.scroll_down(Some(90.0));
        assert_eq!(panel.current_vertical_scroll(), 100.0);
    }

    /// Tests that Panel::scroll_up() decreases vertical scroll.
    /// This test demonstrates the upward scrolling feature.
    #[test]
    fn test_panel_scroll_up() {
        let mut panel = Panel {
            vertical_scroll: Some(50.0),
            ..Default::default()
        };
        
        // Scroll up by default amount
        panel.scroll_up(None);
        assert_eq!(panel.current_vertical_scroll(), 45.0);
        
        // Scroll up by custom amount
        panel.scroll_up(Some(20.0));
        assert_eq!(panel.current_vertical_scroll(), 25.0);
        
        // Scroll should not go below 0
        panel.scroll_up(Some(50.0));
        assert_eq!(panel.current_vertical_scroll(), 0.0);
    }

    /// Tests that Panel::scroll_right() increases horizontal scroll.
    /// This test demonstrates the rightward scrolling feature.
    #[test]
    fn test_panel_scroll_right() {
        let mut panel = Panel::default();
        
        // Initial scroll should be 0
        assert_eq!(panel.current_horizontal_scroll(), 0.0);
        
        // Scroll right by default amount
        panel.scroll_right(None);
        assert_eq!(panel.current_horizontal_scroll(), 5.0);
        
        // Scroll right by custom amount
        panel.scroll_right(Some(10.0));
        assert_eq!(panel.current_horizontal_scroll(), 15.0);
        
        // Scroll should not exceed 100%
        panel.scroll_right(Some(90.0));
        assert_eq!(panel.current_horizontal_scroll(), 100.0);
    }

    /// Tests that Panel::scroll_left() decreases horizontal scroll.
    /// This test demonstrates the leftward scrolling feature.
    #[test]
    fn test_panel_scroll_left() {
        let mut panel = Panel {
            horizontal_scroll: Some(50.0),
            ..Default::default()
        };
        
        // Scroll left by default amount
        panel.scroll_left(None);
        assert_eq!(panel.current_horizontal_scroll(), 45.0);
        
        // Scroll left by custom amount
        panel.scroll_left(Some(20.0));
        assert_eq!(panel.current_horizontal_scroll(), 25.0);
        
        // Scroll should not go below 0
        panel.scroll_left(Some(50.0));
        assert_eq!(panel.current_horizontal_scroll(), 0.0);
    }

    // === Choice Tests ===

    /// Tests that Choice::new() creates a choice with expected values.
    /// This test demonstrates Choice creation and property access.
    #[test]
    fn test_choice_creation() {
        let choice = create_test_choice("test_choice", "Test Content");
        
        assert_eq!(choice.id, "test_choice");
        assert_eq!(choice.content, Some("Test Content".to_string()));
        assert_eq!(choice.selected, false);
        assert_eq!(choice.waiting, false);
        assert_eq!(choice.thread, Some(false));
    }

    /// Tests that Choice implements Clone correctly.
    /// This test demonstrates Choice cloning behavior.
    #[test]
    fn test_choice_clone() {
        let choice1 = create_test_choice("test", "content");
        let choice2 = choice1.clone();
        
        assert_eq!(choice1, choice2);
        assert_eq!(choice1.id, choice2.id);
        assert_eq!(choice1.content, choice2.content);
    }

    /// Tests that Choice implements PartialEq correctly.
    /// This test demonstrates Choice equality comparison.
    #[test]
    fn test_choice_equality() {
        let choice1 = create_test_choice("test", "content");
        let choice2 = create_test_choice("test", "content");
        let choice3 = create_test_choice("other", "content");
        
        assert_eq!(choice1, choice2);
        assert_ne!(choice1, choice3);
    }

    /// Tests that Choice implements Hash correctly.
    /// This test demonstrates Choice hashing behavior.
    #[test]
    fn test_choice_hash() {
        let choice1 = create_test_choice("test", "content");
        let choice2 = create_test_choice("test", "content");
        let choice3 = create_test_choice("other", "content");
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();
        
        choice1.hash(&mut hasher1);
        choice2.hash(&mut hasher2);
        choice3.hash(&mut hasher3);
        
        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    // === Panel Clone Tests ===

    /// Tests that Panel implements Clone correctly.
    /// This test demonstrates Panel cloning behavior.
    #[test]
    fn test_panel_clone() {
        let panel1 = create_test_panel("test");
        let panel2 = panel1.clone();
        
        assert_eq!(panel1, panel2);
        assert_eq!(panel1.id, panel2.id);
        assert_eq!(panel1.title, panel2.title);
        assert_eq!(panel1.position, panel2.position);
    }

    /// Tests that Panel cloning includes children.
    /// This test demonstrates Panel cloning with nested children.
    #[test]
    fn test_panel_clone_with_children() {
        let child_panel = create_test_panel("child");
        let parent_panel = Panel {
            id: "parent".to_string(),
            children: Some(vec![child_panel]),
            ..Default::default()
        };
        
        let cloned = parent_panel.clone();
        
        assert_eq!(parent_panel.children.as_ref().unwrap().len(), 1);
        assert_eq!(cloned.children.as_ref().unwrap().len(), 1);
        assert_eq!(
            parent_panel.children.as_ref().unwrap()[0].id,
            cloned.children.as_ref().unwrap()[0].id
        );
    }

    // === Panel Hash Tests ===

    /// Tests that Panel implements Hash correctly.
    /// This test demonstrates Panel hashing behavior.
    #[test]
    fn test_panel_hash() {
        let panel1 = create_test_panel("test");
        let panel2 = create_test_panel("test");
        let panel3 = create_test_panel("other");
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();
        
        panel1.hash(&mut hasher1);
        panel2.hash(&mut hasher2);
        panel3.hash(&mut hasher3);
        
        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    // === Panel Validation Tests ===

    /// Tests that Panel handles edge cases in scrolling.
    /// This test demonstrates edge case handling in scrolling methods.
    #[test]
    fn test_panel_scrolling_edge_cases() {
        let mut panel = Panel::default();
        
        // Test scrolling when scroll values are None
        panel.vertical_scroll = None;
        panel.horizontal_scroll = None;
        
        panel.scroll_down(Some(10.0));
        assert_eq!(panel.vertical_scroll, Some(10.0));
        
        panel.scroll_right(Some(15.0));
        assert_eq!(panel.horizontal_scroll, Some(15.0));
        
        panel.scroll_up(Some(5.0));
        assert_eq!(panel.vertical_scroll, Some(5.0));
        
        panel.scroll_left(Some(10.0));
        assert_eq!(panel.horizontal_scroll, Some(5.0));
    }

    /// Tests that Panel handles empty and None values correctly.
    /// This test demonstrates edge case handling in Panel properties.
    #[test]
    fn test_panel_empty_values() {
        let panel = Panel {
            id: "test".to_string(),
            title: Some("".to_string()),
            content: Some("".to_string()),
            tab_order: Some("".to_string()),
            ..Default::default()
        };
        
        assert_eq!(panel.id, "test");
        assert_eq!(panel.title, Some("".to_string()));
        assert_eq!(panel.content, Some("".to_string()));
        assert!(!panel.is_selectable()); // Empty tab_order should not be selectable
    }

    // === Panel with Choices Tests ===

    /// Tests that Panel correctly handles choices.
    /// This test demonstrates Panel choice management.
    #[test]
    fn test_panel_with_choices() {
        let choice1 = create_test_choice("choice1", "First Choice");
        let choice2 = create_test_choice("choice2", "Second Choice");
        
        let panel = Panel {
            id: "test".to_string(),
            choices: Some(vec![choice1, choice2]),
            ..Default::default()
        };
        
        assert_eq!(panel.choices.as_ref().unwrap().len(), 2);
        assert_eq!(panel.choices.as_ref().unwrap()[0].id, "choice1");
        assert_eq!(panel.choices.as_ref().unwrap()[1].id, "choice2");
    }

    /// Tests that Panel correctly handles choice selection.
    /// This test demonstrates Panel choice selection behavior.
    #[test]
    fn test_panel_choice_selection() {
        let mut choice1 = create_test_choice("choice1", "First Choice");
        let mut choice2 = create_test_choice("choice2", "Second Choice");
        
        choice1.selected = true;
        choice2.selected = false;
        
        let panel = Panel {
            id: "test".to_string(),
            choices: Some(vec![choice1, choice2]),
            ..Default::default()
        };
        
        assert_eq!(panel.choices.as_ref().unwrap()[0].selected, true);
        assert_eq!(panel.choices.as_ref().unwrap()[1].selected, false);
    }

    // === Panel PartialEq Tests ===

    /// Tests that Panel implements PartialEq correctly.
    /// This test demonstrates Panel equality comparison.
    #[test]
    fn test_panel_equality() {
        let panel1 = create_test_panel("test");
        let panel2 = create_test_panel("test");
        let panel3 = create_test_panel("other");
        
        assert_eq!(panel1, panel2);
        assert_ne!(panel1, panel3);
    }

    /// Tests that Panel equality considers all fields.
    /// This test demonstrates comprehensive Panel equality checking.
    #[test]
    fn test_panel_equality_comprehensive() {
        let panel1 = Panel {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            scroll: true,
            selected: Some(true),
            ..Default::default()
        };
        
        let panel2 = Panel {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            scroll: true,
            selected: Some(true),
            ..Default::default()
        };
        
        let panel3 = Panel {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            scroll: false, // Different scroll value
            selected: Some(true),
            ..Default::default()
        };
        
        assert_eq!(panel1, panel2);
        assert_ne!(panel1, panel3);
    }

    // === Script Deserializer Tests ===

    /// Tests that script deserializer handles simple string arrays.
    /// This test demonstrates basic script array deserialization.
    #[test]
    fn test_script_deserialize_string_array() {
        let yaml = r#"
            id: "test"
            position:
              x1: "0%"
              y1: "0%"
              x2: "100%"
              y2: "100%"
            script:
              - "echo hello"
              - "echo world"
        "#;
        
        let panel: Panel = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = panel.script.expect("Script should be present");
        
        assert_eq!(script.len(), 2);
        assert_eq!(script[0], "echo hello");
        assert_eq!(script[1], "echo world");
    }

    /// Tests that script deserializer handles YAML literal blocks.
    /// This test demonstrates literal block script deserialization.
    #[test]
    fn test_script_deserialize_literal_block() {
        let yaml = r#"
            id: "test"
            position:
              x1: "0%"
              y1: "0%"
              x2: "100%"
              y2: "100%"
            script: |
              echo "Line 1"
              echo "Line 2"
              echo "Line 3"
        "#;
        
        let panel: Panel = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = panel.script.expect("Script should be present");
        
        assert_eq!(script.len(), 3);
        assert_eq!(script[0], "echo \"Line 1\"");
        assert_eq!(script[1], "echo \"Line 2\"");
        assert_eq!(script[2], "echo \"Line 3\"");
    }

    /// Tests that script deserializer handles mixed arrays with literal blocks.
    /// This test demonstrates mixed script format deserialization.
    #[test] 
    fn test_script_deserialize_mixed_array() {
        let yaml = r#"
            id: "test"
            position:
              x1: "0%"
              y1: "0%"
              x2: "100%"
              y2: "100%"
            script:
              - |
                if command -v free >/dev/null; then
                  echo "Memory available"
                fi
              - "echo 'Simple command'"
        "#;
        
        let panel: Panel = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = panel.script.expect("Script should be present");
        
        assert_eq!(script.len(), 2);
        assert!(script[0].contains("if command -v free"));
        assert!(script[0].contains("echo \"Memory available\""));
        assert_eq!(script[1], "echo 'Simple command'");
    }

    /// Tests that script deserializer handles empty script values.
    /// This test demonstrates empty script handling.
    #[test]
    fn test_script_deserialize_empty_values() {
        let yaml = r#"
            id: "test"
            position:
              x1: "0%"
              y1: "0%"
              x2: "100%"
              y2: "100%"
        "#;
        
        let panel: Panel = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        assert_eq!(panel.script, None);
    }

    /// Tests that script deserializer filters empty lines from literal blocks.
    /// This test demonstrates empty line filtering.
    #[test]
    fn test_script_deserialize_filters_empty_lines() {
        let yaml = r#"
            id: "test"
            position:
              x1: "0%"
              y1: "0%"
              x2: "100%"
              y2: "100%"
            script: |
              echo "First line"

              echo "Third line"
              
              echo "Fifth line"
        "#;
        
        let panel: Panel = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = panel.script.expect("Script should be present");
        
        assert_eq!(script.len(), 3);
        assert_eq!(script[0], "echo \"First line\"");
        assert_eq!(script[1], "echo \"Third line\"");
        assert_eq!(script[2], "echo \"Fifth line\"");
    }

    /// Tests that Choice script deserializer works with all formats.
    /// This test demonstrates Choice script deserialization compatibility.
    #[test]
    fn test_choice_script_deserialize_formats() {
        let yaml = r#"
            id: "test_choice"
            content: "Test Choice"
            script:
              - "echo simple"
              - |
                if true; then
                  echo "complex"
                fi
        "#;
        
        let choice: Choice = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = choice.script.expect("Script should be present");
        
        assert_eq!(script.len(), 2);
        assert_eq!(script[0], "echo simple");
        assert!(script[1].contains("if true"));
        assert!(script[1].contains("echo \"complex\""));
    }

    /// Tests that script deserializer handles single string format.
    /// This test demonstrates single string script deserialization.
    #[test]
    fn test_script_deserialize_single_string() {
        let yaml = r#"
            id: "test"
            position:
              x1: "0%"
              y1: "0%"
              x2: "100%"
              y2: "100%"
            script: "echo single command"
        "#;
        
        let panel: Panel = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = panel.script.expect("Script should be present");
        
        assert_eq!(script.len(), 1);
        assert_eq!(script[0], "echo single command");
    }

    /// Tests that script deserializer handles complex YAML structures gracefully.
    /// This test demonstrates error resilience in script deserialization.
    #[test]
    fn test_script_deserialize_error_handling() {
        // Test with valid YAML that has complex script structures
        let yaml = r#"
            id: "test"
            position:
              x1: "0%"
              y1: "0%"
              x2: "100%"
              y2: "100%"
            script:
              - "echo normal"
              - |
                # Complex multiline script
                for i in {1..3}; do
                  echo "Line $i"
                done
              - "echo final"
        "#;
        
        let result = serde_yaml::from_str::<Panel>(yaml);
        assert!(result.is_ok(), "Should handle complex scripts without error");
        
        let panel = result.unwrap();
        let script = panel.script.expect("Script should be present");
        
        assert_eq!(script.len(), 3);
        assert_eq!(script[0], "echo normal");
        assert!(script[1].contains("for i in"));
        assert!(script[1].contains("echo \"Line $i\""));
        assert_eq!(script[2], "echo final");
    }
}
