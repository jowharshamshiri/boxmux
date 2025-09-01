use crate::model::common::*;
use crate::model::layout::Layout;
use crate::utils::{input_bounds_to_bounds, screen_bounds};
use core::hash::Hash;
use indexmap::IndexMap;

/// Priority order for stream types in tab display
pub fn stream_type_priority(stream_type: &crate::model::common::StreamType) -> u8 {
    use crate::model::common::StreamType;
    match stream_type {
        StreamType::Content => 0,             // Main content first
        StreamType::Choices => 1,             // Choices second
        StreamType::OwnScript => 2,           // Own script execution
        StreamType::PTY => 3,                 // PTY sessions
        StreamType::PtySession(_) => 3,       // PTY with command info
        StreamType::RedirectedOutput(_) => 4, // Redirected outputs
        StreamType::ChoiceExecution(_) => 5,  // Choice executions
        StreamType::Plugin(_) => 6,           // Plugin outputs
        StreamType::RedirectSource(_) => 7,   // Redirect sources
        StreamType::ExternalSocket => 8,      // Socket connections
    }
}

// Helper function for serde skip_serializing_if
fn is_zero(n: &usize) -> bool {
    *n == 0
}
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::hash::Hasher;
use std::io::Write;

use crate::{utils::*, AppContext, AppGraph};
use uuid;

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

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Choice {
    pub id: String,
    pub content: Option<String>,
    #[serde(deserialize_with = "deserialize_script", default)]
    pub script: Option<Vec<String>>,
    pub redirect_output: Option<String>,
    pub append_output: Option<bool>,
    // F0222: Choice ExecutionMode Field - Replace thread+pty boolean flags with single execution_mode enum
    #[serde(default)]
    pub execution_mode: ExecutionMode,
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
        self.redirect_output.hash(state);
        self.append_output.hash(state);
        // F0222: Hash ExecutionMode field
        self.execution_mode.hash(state);
        self.selected.hash(state);
        self.waiting.hash(state);
    }
}

impl PartialEq for Choice {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.content == other.content
            && self.script == other.script
            && self.redirect_output == other.redirect_output
            && self.append_output == other.append_output
            // F0222: Compare ExecutionMode field
            && self.execution_mode == other.execution_mode
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
            redirect_output: self.redirect_output.clone(),
            append_output: self.append_output,
            // F0222: Clone ExecutionMode field
            execution_mode: self.execution_mode.clone(),
            selected: self.selected,
            waiting: self.waiting,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct MuxBox {
    pub id: String,
    pub title: Option<String>,
    #[serde(alias = "bounds")]
    pub position: InputBounds,
    #[serde(default)]
    pub anchor: Anchor,
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
    pub max_width: Option<usize>,
    pub max_height: Option<usize>,
    pub overflow_behavior: Option<String>,
    pub refresh_interval: Option<u64>,
    pub tab_order: Option<String>,
    pub next_focus_id: Option<String>,
    pub children: Option<Vec<MuxBox>>,
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
    #[serde(default)]
    pub on_keypress: Option<HashMap<String, Vec<String>>>,
    #[serde(default)]
    pub variables: Option<HashMap<String, String>>,
    pub horizontal_scroll: Option<f64>,
    pub vertical_scroll: Option<f64>,
    pub selected: Option<bool>,
    pub content: Option<String>,
    pub save_in_file: Option<String>,
    #[serde(skip, default)]
    pub streams: IndexMap<String, Stream>,
    pub chart_type: Option<String>,
    pub chart_data: Option<String>,
    pub plugin_component: Option<String>,
    pub plugin_config: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub table_data: Option<String>,
    pub table_config: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub auto_scroll_bottom: Option<bool>,
    // F0221: MuxBox ExecutionMode Field - Replace thread+pty boolean flags with single execution_mode enum
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(default)]
    pub z_index: Option<i32>,
    #[serde(default)]
    pub output: String,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub scroll_x: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub scroll_y: usize,
    #[serde(skip, default)]
    pub tab_scroll_offset: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_layout_id: Option<String>,
    #[serde(skip, default)]
    pub error_state: bool,
}

impl Hash for MuxBox {
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
        self.output.hash(state);
        self.save_in_file.hash(state);
        self.chart_type.hash(state);
        self.chart_data.hash(state);
        self.plugin_component.hash(state);
        // Hash plugin_config by serializing to string (HashMap<String, serde_json::Value> doesn't implement Hash)
        if let Some(ref config) = self.plugin_config {
            serde_json::to_string(config)
                .unwrap_or_default()
                .hash(state);
        }
        self.table_data.hash(state);
        // Hash table_config by serializing to string (HashMap<String, serde_json::Value> doesn't implement Hash)
        if let Some(ref config) = self.table_config {
            serde_json::to_string(config)
                .unwrap_or_default()
                .hash(state);
        }
        self.auto_scroll_bottom.hash(state);
        // F0221: Hash ExecutionMode field
        self.execution_mode.hash(state);
        self.z_index.hash(state);
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
        // Hash streams by collecting and hashing their keys and values
        for (key, stream) in &self.streams {
            key.hash(state);
            stream.hash(state);
        }
    }
}

impl Default for MuxBox {
    fn default() -> Self {
        MuxBox {
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
            on_keypress: None,
            variables: None,
            output: "".to_string(),
            save_in_file: None,
            chart_type: None,
            chart_data: None,
            plugin_component: None,
            plugin_config: None,
            table_data: None,
            table_config: None,
            auto_scroll_bottom: None,
            // F0221: Default ExecutionMode to Immediate
            execution_mode: ExecutionMode::default(),
            z_index: None,
            horizontal_scroll: Some(0.0),
            vertical_scroll: Some(0.0),
            selected: Some(false),
            scroll_x: 0,
            scroll_y: 0,
            tab_scroll_offset: 0,
            parent_id: None,
            parent_layout_id: None,
            error_state: false,
            streams: IndexMap::new(),
        }
    }
}

impl PartialEq for MuxBox {
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
            && self.horizontal_scroll.map(|hs| hs.to_bits())
                == other.horizontal_scroll.map(|hs| hs.to_bits())
            && self.vertical_scroll.map(|vs| vs.to_bits())
                == other.vertical_scroll.map(|vs| vs.to_bits())
            && self.selected == other.selected
            && self.parent_id == other.parent_id
            && self.parent_layout_id == other.parent_layout_id
            && self.output == other.output
            && self.save_in_file == other.save_in_file
            && self.chart_type == other.chart_type
            && self.chart_data == other.chart_data
            && self.plugin_component == other.plugin_component
            && self.plugin_config == other.plugin_config
            && self.table_data == other.table_data
            && self.table_config == other.table_config
            && self.auto_scroll_bottom == other.auto_scroll_bottom
            // F0221: Compare ExecutionMode field
            && self.execution_mode == other.execution_mode
            && self.z_index == other.z_index
            && self.error_state == other.error_state
            && self.streams == other.streams
    }
}

impl Eq for MuxBox {}

impl Clone for MuxBox {
    fn clone(&self) -> Self {
        MuxBox {
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
            on_keypress: self.on_keypress.clone(),
            variables: self.variables.clone(),
            output: self.output.clone(),
            scroll_x: self.scroll_x,
            scroll_y: self.scroll_y,
            tab_scroll_offset: self.tab_scroll_offset,
            save_in_file: self.save_in_file.clone(),
            chart_type: self.chart_type.clone(),
            chart_data: self.chart_data.clone(),
            plugin_component: self.plugin_component.clone(),
            plugin_config: self.plugin_config.clone(),
            table_data: self.table_data.clone(),
            table_config: self.table_config.clone(),
            auto_scroll_bottom: self.auto_scroll_bottom,
            // F0221: Clone ExecutionMode field
            execution_mode: self.execution_mode.clone(),
            z_index: self.z_index,
            horizontal_scroll: self.horizontal_scroll,
            vertical_scroll: self.vertical_scroll,
            selected: self.selected,
            parent_id: self.parent_id.clone(),
            parent_layout_id: self.parent_layout_id.clone(),
            error_state: self.error_state,
            streams: self.streams.clone(),
        }
    }
}

impl MuxBox {
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

    pub fn effective_z_index(&self) -> i32 {
        self.z_index.unwrap_or(0)
    }

    pub fn set_output(&mut self, output: &str) {
        self.output = output.to_string();
    }

    pub fn get_parent_clone(&self, app_graph: &AppGraph) -> Option<MuxBox> {
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
        // MuxBox is selectable if it has explicit tab order OR has scrollable content
        let has_tab_order = self.tab_order.is_some()
            && self.tab_order.as_ref().unwrap() != "none"
            && !self.tab_order.as_ref().unwrap().is_empty();

        has_tab_order || self.has_scrollable_content()
    }

    pub fn is_selected(&self) -> bool {
        self.selected.unwrap_or(false)
    }

    pub fn has_scrollable_content(&self) -> bool {
        let bounds = self.bounds();
        let viewable_width = bounds.width().saturating_sub(4); // Account for borders and padding
        let viewable_height = bounds.height().saturating_sub(4);

        // Check for choice/menu overflow first - choices take priority in rendering
        if let Some(choices) = &self.choices {
            let choice_count = choices.len();
            if choice_count > viewable_height {
                return true; // Choices overflow vertically
            }

            // Check if any choice content is too wide
            let max_choice_width = choices
                .iter()
                .map(|choice| choice.content.as_ref().map(|c| c.len()).unwrap_or(0))
                .max()
                .unwrap_or(0);

            if max_choice_width > viewable_width {
                return true; // Choice content overflows horizontally
            }
        }

        // Check text content for overflow from streams (F0214: Stream-based scrollbar fix)
        let content = {
            // Use get_active_stream_content() to match how scrollbars are drawn in draw_utils.rs
            if let Some(stream) = self.get_active_stream() {
                match stream.stream_type {
                    crate::model::common::StreamType::Content
                    | crate::model::common::StreamType::RedirectedOutput(_)
                    | crate::model::common::StreamType::PTY
                    | crate::model::common::StreamType::Plugin(_)
                    | crate::model::common::StreamType::ChoiceExecution(_)
                    | crate::model::common::StreamType::PtySession(_)
                    | crate::model::common::StreamType::OwnScript => {
                        use crate::model::common::ContentStreamTrait;
                        Some(stream.get_content_lines().join("\n"))
                    }
                    _ => None,
                }
            } else {
                // Fallback to old field-based approach for backward compatibility
                if !self.output.is_empty() {
                    Some(self.output.as_str())
                } else {
                    self.content.as_deref()
                }.map(|s| s.to_string())
            }
        };

        if let Some(content_str) = content {
            // Empty content is not scrollable
            if content_str.trim().is_empty() {
                return false;
            }

            // Use the same content_size logic as draw_utils.rs
            let lines: Vec<&str> = content_str.split('\n').collect();
            let content_height = lines.len();
            let content_width = lines.iter().map(|line| line.len()).max().unwrap_or(0);

            // Account for choice height if both choices and content exist
            let total_content_height = if let Some(choices) = &self.choices {
                content_height + choices.len()
            } else {
                content_height
            };

            // Content overflows if it's larger than viewable area
            content_width > viewable_width || total_content_height > viewable_height
        } else {
            false
        }
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
                    entity_type: EntityType::MuxBox,
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
                    entity_type: EntityType::MuxBox,
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
            if update.entity_type != EntityType::MuxBox {
                continue;
            }
            if let Some(entity_id) = &update.entity_id {
                // Check if the update is for a child muxbox
                if self.children.as_ref().map_or(false, |children| {
                    children.iter().any(|p| p.id == *entity_id)
                }) {
                    // Find the child muxbox and apply the update
                    if let Some(child_muxbox) = self
                        .children
                        .as_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|p| p.id == *entity_id)
                    {
                        child_muxbox.apply_updates(vec![FieldUpdate {
                            entity_type: EntityType::MuxBox,
                            entity_id: Some(child_muxbox.id.clone()),
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
                            serde_json::from_value::<Vec<MuxBox>>(update.new_value.clone())
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
        // Update active stream content
        if let Some(active_stream) = self.get_active_stream_mut() {
            let current_content = active_stream.content.join("\n");
            let formatted_content = if append_content {
                format!(
                    "[{}]\n\n{}\n\n\n\n{}",
                    chrono::Local::now().to_rfc2822(),
                    current_content,
                    new_content
                )
            } else {
                format!("[{}]\n\n{}", chrono::Local::now().to_rfc2822(), new_content)
            };
            active_stream.content = formatted_content.lines().map(|s| s.to_string()).collect();
            self.error_state = !success;
            return;
        }

        // Fallback to regular content update if no streams
        // Preserve current scroll position
        let preserved_horizontal_scroll = self.horizontal_scroll;
        let preserved_vertical_scroll = self.vertical_scroll;

        let mut formatted_content = new_content.to_string();
        if append_content {
            if let Some(self_content) = &self.content {
                formatted_content = format!(
                    "[{}]\n\n{}\n\n\n\n{}",
                    chrono::Local::now().to_rfc2822(),
                    self_content,
                    new_content
                );
            } else {
                formatted_content =
                    format!("[{}]\n\n{}", chrono::Local::now().to_rfc2822(), new_content);
            }
        }
        self.content = Some(formatted_content);
        self.error_state = !success;

        // Handle auto-scroll to bottom or restore scroll position
        if self.auto_scroll_bottom == Some(true) {
            // Auto-scroll to bottom - set vertical scroll to maximum
            self.vertical_scroll = Some(100.0);
            self.horizontal_scroll = preserved_horizontal_scroll;
        } else {
            // Restore scroll position after content update
            self.horizontal_scroll = preserved_horizontal_scroll;
            self.vertical_scroll = preserved_vertical_scroll;
        }

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

    /// Update content for streaming output (like PTY) without timestamp formatting
    pub fn update_streaming_content(&mut self, new_content: &str, success: bool) {
        // Preserve current scroll position
        let preserved_horizontal_scroll = self.horizontal_scroll;
        let preserved_vertical_scroll = self.vertical_scroll;

        let formatted_content = if let Some(self_content) = &self.content {
            // Simple append without timestamp formatting for streaming
            format!("{}{}", self_content, new_content)
        } else {
            new_content.to_string()
        };

        self.content = Some(formatted_content);
        self.error_state = !success;

        // Handle auto-scroll to bottom or restore scroll position
        if self.auto_scroll_bottom == Some(true) {
            // Auto-scroll to bottom - set vertical scroll to maximum
            self.vertical_scroll = Some(100.0);
            self.horizontal_scroll = preserved_horizontal_scroll;
        } else {
            // Restore scroll position after content update
            self.horizontal_scroll = preserved_horizontal_scroll;
            self.vertical_scroll = preserved_vertical_scroll;
        }
    }

    /// Get scrollback content for PTY muxboxes using the circular buffer
    /// F0120: PTY Scrollback - Access full scrollback history beyond visible area
    pub fn get_scrollback_content(
        &self,
        pty_manager: &crate::pty_manager::PtyManager,
    ) -> Option<String> {
        if self.execution_mode == ExecutionMode::Pty {
            pty_manager.get_scrollback_content(&self.id)
        } else {
            self.content.clone()
        }
    }

    /// Get scrollback lines for PTY muxboxes with range support
    /// F0120: PTY Scrollback - Access specific range of scrollback lines
    pub fn get_scrollback_lines(
        &self,
        pty_manager: &crate::pty_manager::PtyManager,
        start_line: usize,
        line_count: usize,
    ) -> Option<Vec<String>> {
        if self.execution_mode == ExecutionMode::Pty {
            if let Some(buffer) = pty_manager.get_output_buffer(&self.id) {
                if let Ok(buffer_lock) = buffer.lock() {
                    let total_lines = buffer_lock.len();
                    if start_line < total_lines {
                        let _end_line = std::cmp::min(start_line + line_count, total_lines);
                        return Some(buffer_lock.get_lines_range(start_line, line_count));
                    }
                }
            }
            None
        } else {
            // For regular muxboxes, split content by lines and return range
            if let Some(content) = &self.content {
                let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
                let total_lines = lines.len();
                if start_line < total_lines {
                    let end_line = std::cmp::min(start_line + line_count, total_lines);
                    return Some(lines[start_line..end_line].to_vec());
                }
            }
            None
        }
    }

    /// Get total available scrollback lines for a muxbox
    /// F0120: PTY Scrollback - Get total scrollback line count for scroll calculations
    pub fn get_scrollback_line_count(&self, pty_manager: &crate::pty_manager::PtyManager) -> usize {
        if self.execution_mode == ExecutionMode::Pty {
            if let Some(buffer) = pty_manager.get_output_buffer(&self.id) {
                if let Ok(buffer_lock) = buffer.lock() {
                    return buffer_lock.len();
                }
            }
            0
        } else {
            // For regular muxboxes, count lines in content
            self.content
                .as_ref()
                .map(|c| c.lines().count())
                .unwrap_or(0)
        }
    }

    /// Generate plugin content for the muxbox
    pub fn generate_plugin_content(
        &self,
        app_context: &AppContext,
        bounds: &Bounds,
    ) -> Option<String> {
        use crate::plugin::{ComponentConfig, PluginContext, PluginRegistry};
        use std::collections::HashMap;

        if let Some(component_type) = &self.plugin_component {
            let registry = PluginRegistry::new();
            let context = PluginContext {
                app_context: app_context.clone(),
                muxbox_bounds: bounds.clone(),
                plugin_data: HashMap::new(),
                permissions: Vec::new(),
            };
            let config = ComponentConfig {
                component_type: component_type.clone(),
                properties: self.plugin_config.clone().unwrap_or_default(),
                data_source: None,
                refresh_interval: None,
            };

            match registry.render_component(component_type, &context, &config) {
                Ok(content) => Some(content),
                Err(_) => {
                    // Return mock content for testing when component is not found
                    Some(self.generate_mock_plugin_content(component_type, bounds))
                }
            }
        } else {
            None
        }
    }

    /// Generate mock plugin content for testing
    fn generate_mock_plugin_content(&self, component_type: &str, bounds: &Bounds) -> String {
        let bounds_str = format!("{}x{}", bounds.width(), bounds.height());

        match component_type {
            "custom_chart" => {
                format!(
                    "Mock Plugin: custom_chart\nTest Chart\nBounds: {}",
                    bounds_str
                )
            }
            "data_table" => {
                if let Some(config) = &self.plugin_config {
                    let mut content = format!("Mock Plugin: data_table\n");

                    // Check for columns configuration
                    if let Some(columns) = config.get("columns") {
                        if let Some(cols_array) = columns.as_array() {
                            for col in cols_array {
                                if let Some(col_str) = col.as_str() {
                                    content.push_str(&format!("{}: Sample\n", col_str));
                                }
                            }
                        }
                    }

                    // Include page size if specified
                    if let Some(page_size) = config.get("page_size") {
                        if let Some(size) = page_size.as_u64() {
                            content.push_str(&format!("Page Size: {}\n", size));
                        }
                    }

                    content.push_str(&format!("Bounds: {}", bounds_str));
                    content
                } else {
                    format!("Mock Plugin: data_table\nBounds: {}", bounds_str)
                }
            }
            "dashboard_widget" => {
                if let Some(config) = &self.plugin_config {
                    if let Some(widget_type) = config.get("widget_type") {
                        format!(
                            "Mock Plugin: dashboard_widget\nType: {}\nBounds: {}",
                            widget_type.as_str().unwrap_or("unknown"),
                            bounds_str
                        )
                    } else {
                        format!(
                            "Mock Plugin: dashboard_widget\nmetrics\nBounds: {}",
                            bounds_str
                        )
                    }
                } else {
                    format!(
                        "Mock Plugin: dashboard_widget\nmetrics\nBounds: {}",
                        bounds_str
                    )
                }
            }
            _ => {
                format!("Mock Plugin: {}\nBounds: {}", component_type, bounds_str)
            }
        }
    }

    /// Generate chart content for the muxbox
    pub fn generate_chart_content(&self, bounds: &Bounds) -> Option<String> {
        use crate::chart::{generate_chart, parse_chart_data, ChartConfig, ChartType};

        if let (Some(chart_type_str), Some(chart_data)) = (&self.chart_type, &self.chart_data) {
            let data = parse_chart_data(chart_data);
            let chart_type = match chart_type_str.as_str() {
                "bar" => ChartType::Bar,
                "line" => ChartType::Line,
                "histogram" => ChartType::Histogram,
                _ => ChartType::Bar,
            };
            let config = ChartConfig {
                chart_type,
                width: bounds.width().saturating_sub(4),
                height: bounds.height().saturating_sub(4),
                title: None, // Don't show chart title since muxbox already has the title
                color: "blue".to_string(),
            };
            Some(generate_chart(&data, &config))
        } else {
            None
        }
    }

    /// Generate table content for the muxbox
    pub fn generate_table_content(&self, bounds: &Bounds) -> Option<String> {
        use crate::table::{
            parse_table_data, parse_table_data_from_json, render_table, TableBorderStyle,
            TableConfig,
        };
        use std::collections::HashMap;

        if let Some(table_data) = &self.table_data {
            // Parse table data
            let data = if table_data.trim().starts_with('[') {
                // JSON format
                match parse_table_data_from_json(table_data) {
                    Ok(d) => d,
                    Err(e) => return Some(format!("Table JSON error: {}", e)),
                }
            } else {
                // CSV format
                parse_table_data(table_data, None)
            };

            // Create table configuration
            let mut config = TableConfig {
                width: bounds.width().saturating_sub(4),
                height: bounds.height().saturating_sub(4),
                title: self.title.clone(),
                ..Default::default()
            };

            // Apply table configuration from muxbox
            if let Some(table_config_map) = &self.table_config {
                if let Some(show_headers) = table_config_map
                    .get("show_headers")
                    .and_then(|v| v.as_bool())
                {
                    config.show_headers = show_headers;
                }

                if let Some(sort_column) =
                    table_config_map.get("sort_column").and_then(|v| v.as_str())
                {
                    config.sort_column = Some(sort_column.to_string());
                }

                if let Some(sort_ascending) = table_config_map
                    .get("sort_ascending")
                    .and_then(|v| v.as_bool())
                {
                    config.sort_ascending = sort_ascending;
                }

                if let Some(zebra_striping) = table_config_map
                    .get("zebra_striping")
                    .and_then(|v| v.as_bool())
                {
                    config.zebra_striping = zebra_striping;
                }

                if let Some(show_row_numbers) = table_config_map
                    .get("show_row_numbers")
                    .and_then(|v| v.as_bool())
                {
                    config.show_row_numbers = show_row_numbers;
                }

                if let Some(border_style) = table_config_map
                    .get("border_style")
                    .and_then(|v| v.as_str())
                {
                    config.border_style = match border_style {
                        "none" => TableBorderStyle::None,
                        "single" => TableBorderStyle::Single,
                        "double" => TableBorderStyle::Double,
                        "rounded" => TableBorderStyle::Rounded,
                        "thick" => TableBorderStyle::Thick,
                        _ => TableBorderStyle::Single,
                    };
                }

                // Apply filters
                if let Some(filters_obj) =
                    table_config_map.get("filters").and_then(|v| v.as_object())
                {
                    let mut filters = HashMap::new();
                    for (key, value) in filters_obj {
                        if let Some(val_str) = value.as_str() {
                            filters.insert(key.clone(), val_str.to_string());
                        }
                    }
                    config.filters = filters;
                }

                if let Some(max_column_width) = table_config_map
                    .get("max_column_width")
                    .and_then(|v| v.as_u64())
                {
                    config.max_column_width = Some(max_column_width as usize);
                }

                // Pagination configuration
                if let Some(page_size) = table_config_map.get("page_size").and_then(|v| v.as_u64())
                {
                    let current_page = table_config_map
                        .get("current_page")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as usize;
                    let show_page_info = table_config_map
                        .get("show_page_info")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    config.pagination = Some(crate::table::TablePagination {
                        page_size: page_size as usize,
                        current_page,
                        show_page_info,
                    });
                }
            }

            Some(render_table(&data, &config))
        } else {
            None
        }
    }

    // F0203: Multi-Stream Input Tabs - Unified tab system initialization
    pub fn ensure_tabs_initialized(&mut self) {
        // Every box must have streams - if empty, initialize them
        if self.streams.is_empty() {
            self.initialize_streams();
        }
    }

    /// Add a new input stream
    pub fn add_input_stream(
        &mut self,
        source_type: crate::model::common::StreamType,
        label: String,
    ) -> String {
        // F0212: Stream Source Tracking - Add stream to streams HashMap
        let stream_id = format!("{}_{}", self.id, uuid::Uuid::new_v4());
        let stream = Stream::new(
            stream_id.clone(),
            source_type,
            label,
            vec![],
            None,
            None, // F0212: No source tracking for manually added streams
        );

        self.streams.insert(stream_id.clone(), stream);
        stream_id
    }

    pub fn switch_to_tab(&mut self, tab_index: usize) -> bool {
        // F0218: Stream Tab Integration - tabs control active stream
        // Get stream IDs in natural insertion order (same as tab drawing)
        let stream_ids: Vec<String> = self.streams.keys().cloned().collect();

        if tab_index < stream_ids.len() {
            let target_stream_id = &stream_ids[tab_index];

            // Deactivate all streams
            for stream in self.streams.values_mut() {
                stream.active = false;
            }

            // Activate target stream
            if let Some(target_stream) = self.streams.get_mut(target_stream_id) {
                target_stream.active = true;
                log::info!(
                    "Successfully switched to tab {} (stream {})",
                    tab_index,
                    target_stream_id
                );
                return true;
            }
        }
        log::warn!(
            "Failed to switch to tab {} - index out of range or stream not found",
            tab_index
        );
        false
    }

    /// Check if content was defined in YAML vs populated by redirect output
    fn is_yaml_defined_content(&self, content: &str) -> bool {
        // F0229: Simple heuristic - redirect output typically has timestamps, ERROR prefixes, or execution indicators
        // YAML content is usually static documentation, descriptions, or simple text
        let lines: Vec<&str> = content.lines().collect();

        // Empty content is not YAML-defined
        if lines.is_empty() {
            return false;
        }

        // Check for common redirect output patterns
        let has_redirect_patterns = lines.iter().any(|line| {
            line.starts_with("ERROR:") ||
            line.starts_with("===") ||
            line.starts_with("Executing") ||
            line.contains("completed successfully") ||
            line.contains("failed:") ||
            // Check for timestamp-like patterns (crude but effective)
            line.contains(" [") && line.contains("] ")
        });

        // If no redirect patterns found, likely YAML-defined
        !has_redirect_patterns
    }

    /// Initialize streams for a muxbox - F0229: Only create streams for YAML-defined content/choices
    pub fn initialize_streams(&mut self) {
        self.streams.clear();

        // F0229: Check if content was defined in YAML vs populated by redirect
        let has_yaml_content = self.content.as_ref().map_or(false, |c| {
            !c.trim().is_empty() && self.is_yaml_defined_content(c)
        });
        let has_choices = self
            .choices
            .as_ref()
            .map_or(false, |choices| !choices.is_empty());

        // F0229: Check if this is an action-only box (choices but no YAML content)
        let is_action_only_box = has_choices && !has_yaml_content;

        // F0229: Only create content stream for YAML-defined content (not redirect output)
        if has_yaml_content && !is_action_only_box {
            let content_title = if has_choices {
                // If both content and choices exist, content tab gets box title or box ID
                self.title.clone().unwrap_or_else(|| self.id.clone())
            } else {
                // Content is the only stream, use default "Content" title
                "Content".to_string()
            };

            let mut content_stream = Stream::new(
                format!("{}_content", self.id),
                StreamType::Content,
                content_title,
                self.content
                    .as_ref()
                    .unwrap()
                    .lines()
                    .map(|s| s.to_string())
                    .collect(),
                None,
                Some(crate::model::common::StreamSource::StaticContent(
                    crate::model::common::StaticContentSource {
                        content_type: "default".to_string(),
                        created_at: std::time::SystemTime::now(),
                    },
                )),
            );
            content_stream.active = true;
            self.streams
                .insert(content_stream.id.clone(), content_stream);
        }

        // Create choices stream only if choices exist and are non-empty
        if has_choices {
            let choices_title = if has_yaml_content && !is_action_only_box {
                // If both content and choices exist, choices get "Choices" title
                "Choices".to_string()
            } else {
                // Choices is the only stream (action-only box), use box title or box ID
                self.title.clone().unwrap_or_else(|| self.id.clone())
            };

            let mut choices_stream = Stream::new(
                format!("{}_choices", self.id),
                StreamType::Choices,
                choices_title,
                vec![], // Choices stream doesn't use content field
                Some(self.choices.as_ref().unwrap().clone()), // Store actual Choice objects
                Some(crate::model::common::StreamSource::StaticContent(
                    crate::model::common::StaticContentSource {
                        content_type: "choices".to_string(),
                        created_at: std::time::SystemTime::now(),
                    },
                )),
            );

            // If no content stream exists (action-only box), choices stream becomes active
            if is_action_only_box || !has_yaml_content {
                choices_stream.active = true;
            }

            self.streams
                .insert(choices_stream.id.clone(), choices_stream);
        }
    }

    /// Add a redirected output stream
    pub fn add_redirected_stream(&mut self, source_name: String, content: Vec<String>) {
        let stream_id = format!("{}_{}", self.id, source_name);
        let stream = Stream::new(
            stream_id.clone(),
            StreamType::RedirectedOutput(source_name.clone()),
            source_name.clone(),
            content,
            None,
            Some(crate::model::common::StreamSource::Redirect(
                crate::model::common::RedirectSource {
                    source_muxbox_id: "unknown".to_string(), // Will be filled by caller
                    source_choice_id: None,
                    redirect_name: source_name,
                    redirect_type: "external".to_string(),
                    created_at: std::time::SystemTime::now(),
                    source_process_id: None,
                },
            )),
        );
        self.streams.insert(stream_id, stream);
    }

    /// F0212: Add stream with source tracking
    pub fn add_stream_with_source(
        &mut self,
        stream_type: StreamType,
        label: String,
        source: crate::model::common::StreamSource,
    ) -> String {
        // SOURCE OBJECT ARCHITECTURE: Extract stream_id from StreamType (which contains source object stream_id)
        let stream_id = match &stream_type {
            StreamType::Content => format!("{}_content", self.id),
            StreamType::Choices => format!("{}_choices", self.id),
            StreamType::PTY => format!("{}_pty", self.id),
            StreamType::ExternalSocket => format!("{}_external", self.id),
            StreamType::OwnScript => format!("{}_script", self.id),
            StreamType::ChoiceExecution(source_stream_id) => source_stream_id.clone(),
            StreamType::PtySession(source_stream_id) => source_stream_id.clone(),
            StreamType::RedirectedOutput(source_stream_id) => source_stream_id.clone(),
            StreamType::RedirectSource(source_stream_id) => source_stream_id.clone(),
            StreamType::Plugin(plugin_name) => format!("{}_{}", self.id, plugin_name),
        };
        let stream = Stream::new(
            stream_id.clone(),
            stream_type,
            label,
            vec![],
            None,
            Some(source),
        );
        self.streams.insert(stream_id.clone(), stream);
        stream_id
    }

    /// F0212: Get streams that need cleanup (have sources)
    pub fn get_streams_requiring_cleanup(
        &self,
    ) -> Vec<(&String, &crate::model::common::StreamSource)> {
        self.streams
            .iter()
            .filter_map(|(stream_id, stream)| {
                stream.source.as_ref().map(|source| (stream_id, source))
            })
            .collect()
    }

    /// F0212: Remove stream and return its source for cleanup
    pub fn remove_stream(&mut self, stream_id: &str) -> Option<crate::model::common::StreamSource> {
        // Check if the stream being removed is currently active
        let was_active = self
            .streams
            .get(stream_id)
            .map(|stream| stream.active)
            .unwrap_or(false);

        // Remove the stream and get its source
        let removed_source = self
            .streams
            .shift_remove(stream_id)
            .and_then(|stream| stream.source);

        // If the removed stream was active, switch to the first remaining stream
        if was_active && !self.streams.is_empty() {
            // Set the first remaining stream as active
            if let Some((_, first_stream)) = self.streams.get_index_mut(0) {
                first_stream.active = true;
                log::info!(
                    "Stream '{}' was active and removed, switched to first remaining stream: '{}'",
                    stream_id,
                    first_stream.id
                );
            }
        }

        removed_source
    }

    /// F0212: Find streams by source type
    pub fn find_streams_by_source(&self, source_pattern: &str) -> Vec<String> {
        self.streams
            .iter()
            .filter_map(|(stream_id, stream)| {
                if let Some(ref source) = stream.source {
                    match source {
                        crate::model::common::StreamSource::ChoiceExecution(choice_source) => {
                            if choice_source.choice_id.contains(source_pattern) {
                                Some(stream_id.clone())
                            } else {
                                None
                            }
                        }
                        crate::model::common::StreamSource::PTY(pty_source) => {
                            if pty_source.process_id.to_string() == source_pattern {
                                Some(stream_id.clone())
                            } else {
                                None
                            }
                        }
                        crate::model::common::StreamSource::Redirect(redirect_source) => {
                            if redirect_source.source_muxbox_id.contains(source_pattern) {
                                Some(stream_id.clone())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect()
    }

    /// Get all streams as tab labels
    pub fn get_stream_tabs(&self) -> Vec<String> {
        // Use exact same ordering as get_tab_labels() - IndexMap insertion order
        self.streams.iter().map(|(_, s)| s.label.clone()).collect()
    }

    /// Get the active stream (unified approach)
    pub fn get_active_stream(&self) -> Option<&crate::model::common::Stream> {
        self.streams.values().find(|s| s.active)
    }

    /// Get mutable active stream (unified approach)
    pub fn get_active_stream_mut(&mut self) -> Option<&mut crate::model::common::Stream> {
        self.streams.values_mut().find(|s| s.active)
    }

    /// Get active stream content (deprecated - use get_active_stream + ContentStreamTrait)
    pub fn get_active_stream_content(&self) -> Vec<String> {
        self.streams
            .values()
            .find(|s| s.active)
            .map(|s| s.content.clone())
            .unwrap_or_else(|| vec![])
    }

    /// Get active stream choices (deprecated - use get_active_stream + ChoicesStreamTrait)
    pub fn get_active_stream_choices(&self) -> Option<&Vec<Choice>> {
        self.streams
            .values()
            .find(|s| s.active)
            .and_then(|s| s.choices.as_ref())
    }

    /// Get mutable active stream choices (deprecated - use get_active_stream_mut + ChoicesStreamTrait)
    pub fn get_active_stream_choices_mut(&mut self) -> Option<&mut Vec<Choice>> {
        self.streams
            .values_mut()
            .find(|s| s.active)
            .and_then(|s| s.choices.as_mut())
    }

    /// Switch to a specific stream by index
    pub fn switch_to_stream_by_index(&mut self, stream_index: usize) -> bool {
        // Use exact same ordering as all other tab functions - IndexMap insertion order
        let stream_ids: Vec<String> = self.streams.keys().cloned().collect();

        if stream_index >= stream_ids.len() {
            return false;
        }

        // Set all streams to inactive
        for stream in self.streams.values_mut() {
            stream.active = false;
        }

        // Activate the selected stream
        let target_id = &stream_ids[stream_index];
        if let Some(stream) = self.streams.get_mut(target_id) {
            stream.active = true;
            return true;
        }

        false
    }

    /// Update content for a specific stream
    pub fn update_stream_content(&mut self, stream_id: &str, content: Vec<String>) {
        if let Some(stream) = self.streams.get_mut(stream_id) {
            stream.content = content;
        }
    }

    /// Get active tab index for rendering
    pub fn get_active_tab_index(&self) -> usize {
        if self.streams.is_empty() {
            return 0;
        }

        // Use exact same ordering as get_tab_labels() and switch_to_tab() - IndexMap insertion order
        let stream_ids: Vec<String> = self.streams.keys().cloned().collect();

        // Find the index of the active stream
        for (index, stream_id) in stream_ids.iter().enumerate() {
            if let Some(stream) = self.streams.get(stream_id) {
                if stream.active {
                    return index;
                }
            }
        }

        0 // Fallback to first tab
    }

    /// Switch to a specific stream by ID
    pub fn switch_to_stream(&mut self, stream_id: &str) -> bool {
        // F0218: Stream Tab Integration - Switch using streams architecture
        if !self.streams.contains_key(stream_id) {
            return false;
        }

        // Set all streams to inactive
        for stream in self.streams.values_mut() {
            stream.active = false;
        }

        // Activate the specified stream
        if let Some(stream) = self.streams.get_mut(stream_id) {
            stream.active = true;
            return true;
        }

        false
    }

    /// Get current active stream ID
    pub fn get_current_stream_id(&self) -> Option<String> {
        // F0218: Stream Tab Integration - Get from streams architecture
        self.streams
            .values()
            .find(|s| s.active)
            .map(|s| s.id.clone())
    }

    /// All boxes need streams - check if they're missing
    pub fn needs_tab_initialization(&self) -> bool {
        self.streams.is_empty()
    }

    pub fn get_tab_labels(&self) -> Vec<String> {
        // F0218: Stream Tab Integration - generate tabs from streams
        log::info!(
            "TAB DEBUG: get_tab_labels() called for box {}, streams count: {}",
            self.id,
            self.streams.len()
        );
        let mut labels = Vec::new();

        // Use exact same ordering as switch_to_tab() - natural IndexMap insertion order
        let stream_ids: Vec<String> = self.streams.keys().cloned().collect();

        for stream_id in stream_ids {
            let stream = &self.streams[&stream_id];
            // Use the stream's actual label instead of hardcoded titles
            let label = match &stream.stream_type {
                crate::model::common::StreamType::Content
                | crate::model::common::StreamType::Choices => stream.label.clone(),
                crate::model::common::StreamType::RedirectedOutput(name) => format!("→{}", name),
                crate::model::common::StreamType::PTY => "PTY".to_string(),
                crate::model::common::StreamType::Plugin(name) => format!("Plugin:{}", name),
                crate::model::common::StreamType::ChoiceExecution(choice_id) => {
                    format!("Choice:{}", choice_id)
                }
                crate::model::common::StreamType::RedirectSource(source) => {
                    format!("From:{}", source)
                }
                crate::model::common::StreamType::ExternalSocket => "Socket".to_string(),
                crate::model::common::StreamType::PtySession(name) => format!("PTY:{}", name),
                crate::model::common::StreamType::OwnScript => "Script".to_string(),
            };
            labels.push(label);
        }

        log::info!(
            "TAB DEBUG: get_tab_labels() returning {} labels for box {}: {:?}",
            labels.len(),
            self.id,
            labels
        );

        // No default tab - empty boxes have no tabs
        labels
    }

    /// F0219: Get close button information for each tab (same order as get_tab_labels)
    pub fn get_tab_close_buttons(&self) -> Vec<bool> {
        let mut close_buttons = Vec::new();

        // Use exact same ordering as get_tab_labels() - natural IndexMap insertion order
        let stream_ids: Vec<String> = self.streams.keys().cloned().collect();

        for stream_id in stream_ids {
            let stream = &self.streams[&stream_id];
            close_buttons.push(stream.is_closeable());
        }

        close_buttons
    }

    /// F0219: Get stream IDs in tab order (for close button click handling)
    pub fn get_tab_stream_ids(&self) -> Vec<String> {
        // Use exact same ordering as get_tab_labels() - natural IndexMap insertion order
        self.streams.keys().cloned().collect()
    }

    /// Check if tabs need scrolling (overflow available width)
    pub fn tabs_need_scrolling(&self, available_width: usize) -> bool {
        let tab_labels = self.get_tab_labels();
        if tab_labels.is_empty() {
            return false;
        }

        // Calculate required width for all tabs
        let max_tab_width = 16;
        let min_tab_width = 6;
        let calculated_tab_width = available_width / tab_labels.len().max(1);
        let tab_width = calculated_tab_width.clamp(min_tab_width, max_tab_width);

        let required_width = tab_labels.len() * tab_width + 4; // 4 for border/separators
        required_width > available_width
    }

    /// Scroll tabs left (decrease offset)
    pub fn scroll_tabs_left(&mut self) {
        if self.tab_scroll_offset > 0 {
            self.tab_scroll_offset -= 1;
        }
    }

    /// Scroll tabs right (increase offset)
    pub fn scroll_tabs_right(&mut self) {
        let tab_count = self.get_tab_labels().len();
        if self.tab_scroll_offset < tab_count.saturating_sub(1) {
            self.tab_scroll_offset += 1;
        }
    }

    /// Get maximum valid tab scroll offset
    pub fn max_tab_scroll_offset(&self, available_width: usize) -> usize {
        let tab_labels = self.get_tab_labels();
        if tab_labels.is_empty() || !self.tabs_need_scrolling(available_width) {
            return 0;
        }

        let max_tab_width = 16;
        let min_tab_width = 6;
        let calculated_tab_width = available_width / tab_labels.len().max(1);
        let tab_width = calculated_tab_width.clamp(min_tab_width, max_tab_width);

        // Calculate how many tabs fit in available width (reserve 6 chars for nav arrows)
        let nav_arrow_width = 6; // "< " and " >"
        let usable_width = available_width.saturating_sub(nav_arrow_width);
        let tabs_that_fit = usable_width / tab_width.max(1);

        tab_labels.len().saturating_sub(tabs_that_fit)
    }

    // F0317: Terminal Window Integration Methods - PTY boxes act like terminal windows

    /// Calculate terminal character dimensions from pixel bounds
    /// F0317: PTY boxes should have dimensions that match their terminal content
    pub fn calculate_terminal_dimensions(&self, bounds: &Bounds) -> (u16, u16) {
        // F0316: Performance Optimization - Improved terminal dimension calculation
        // Convert inclusive bounds to character dimensions with proper adjustments
        let mut cols = bounds.width() as u16;
        let mut rows = bounds.height() as u16;
        
        // Account for border space if present
        if self.border.unwrap_or(false) {
            // Borders take 2 characters (left + right) and 2 rows (top + bottom)
            cols = cols.saturating_sub(2);
            rows = rows.saturating_sub(2);
        }
        
        // Account for title bar space (1 row for title)
        if self.title.as_ref().map_or(false, |t| !t.is_empty()) {
            rows = rows.saturating_sub(1);
        }
        
        // Account for tab bar space if multiple streams exist
        let stream_count = self.streams.len();
        if stream_count > 1 {
            rows = rows.saturating_sub(1); // Tab bar takes 1 row
        }
        
        // Ensure minimum viable terminal size (htop/terminal applications need reasonable space)
        let min_cols = 40u16;  // Minimum for readable terminal applications
        let min_rows = 10u16;  // Minimum for practical terminal usage
        
        let final_cols = cols.max(min_cols);
        let final_rows = rows.max(min_rows);
        
        log::debug!("Terminal dimension calculation for box {}: bounds={}x{}, border={}, title_present={}, streams={} -> {}x{} characters", 
            self.id, bounds.width(), bounds.height(), self.border.unwrap_or(false), self.title.as_ref().map_or(false, |t| !t.is_empty()), stream_count, final_cols, final_rows);
        
        (final_cols, final_rows)
    }

    /// Update MuxBox bounds and resize associated PTY to match
    /// F0317: When PTY box resizes, the underlying PTY terminal should resize too
    pub fn update_bounds_with_pty_resize(&mut self, bounds: &Bounds, pty_manager: &mut crate::pty_manager::PtyManager) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Only handle PTY boxes - others use normal bounds update
        if !matches!(self.execution_mode, ExecutionMode::Pty) {
            return Ok(()); // Non-PTY boxes don't need PTY resize
        }

        // Calculate terminal dimensions from new bounds
        let (cols, rows) = self.calculate_terminal_dimensions(bounds);
        
        // Resize the PTY to match the new terminal dimensions
        if let Err(e) = pty_manager.resize_pty(&self.id, rows, cols) {
            log::warn!("PTY resize failed for box {}: {}", self.id, e);
            // Don't fail the operation - PTY might not exist yet or might be finished
        } else {
            log::debug!("PTY resized for box {} to {}x{} characters", self.id, cols, rows);
        }

        Ok(())
    }

    /// Update MuxBox title from PTY terminal title changes
    /// F0315 + F0317: PTY terminal title changes should propagate to the containing box
    pub fn update_title_from_pty(&mut self, terminal_title: Option<String>) {
        if matches!(self.execution_mode, ExecutionMode::Pty) {
            self.title = terminal_title;
        }
    }
}

impl Choice {}

impl Updatable for MuxBox {
    fn generate_diff(&self, other: &Self) -> Vec<FieldUpdate> {
        let mut updates = Vec::new();
        // Compare each field and add to updates if not null and different
        if self.title != other.title {
            if let Some(new_value) = &other.title {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()),
                    field_name: "title".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.position != other.position {
            updates.push(FieldUpdate {
                entity_type: EntityType::MuxBox,
                entity_id: Some(self.id.clone()),
                field_name: "position".to_string(),
                new_value: serde_json::to_value(&other.position).unwrap(),
            });
        }

        if self.anchor != other.anchor {
            updates.push(FieldUpdate {
                entity_type: EntityType::MuxBox,
                entity_id: Some(self.id.clone()),
                field_name: "anchor".to_string(),
                new_value: serde_json::to_value(&other.anchor).unwrap(),
            });
        }

        if self.min_height != other.min_height {
            if let Some(new_value) = other.min_height {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "min_height".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.min_width != other.min_width {
            if let Some(new_value) = other.min_width {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "min_width".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.max_height != other.max_height {
            if let Some(new_value) = other.max_height {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "max_height".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.max_width != other.max_width {
            if let Some(new_value) = other.max_width {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "max_width".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.overflow_behavior != other.overflow_behavior {
            if let Some(new_value) = &other.overflow_behavior {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "overflow_behavior".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.refresh_interval != other.refresh_interval {
            if let Some(new_value) = other.refresh_interval {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "refresh_interval".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.tab_order != other.tab_order {
            if let Some(new_value) = &other.tab_order {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "tab_order".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.next_focus_id != other.next_focus_id {
            if let Some(new_value) = &other.next_focus_id {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "next_focus_id".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        updates.extend(self.generate_children_diff(other));

        if self.fill != other.fill {
            updates.push(FieldUpdate {
                entity_type: EntityType::MuxBox,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "fill".to_string(),
                new_value: serde_json::to_value(other.fill).unwrap(),
            });
        }

        if self.fill_char != other.fill_char {
            if let Some(new_value) = other.fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "fill_char".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_fill_char != other.selected_fill_char {
            if let Some(new_value) = other.selected_fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_fill_char".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.border != other.border {
            updates.push(FieldUpdate {
                entity_type: EntityType::MuxBox,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "border".to_string(),
                new_value: serde_json::to_value(other.border).unwrap(),
            });
        }

        if self.border_color != other.border_color {
            if let Some(new_value) = &other.border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_border_color != other.selected_border_color {
            if let Some(new_value) = &other.selected_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.bg_color != other.bg_color {
            if let Some(new_value) = &other.bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_bg_color != other.selected_bg_color {
            if let Some(new_value) = &other.selected_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.fg_color != other.fg_color {
            if let Some(new_value) = &other.fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_fg_color != other.selected_fg_color {
            if let Some(new_value) = &other.selected_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_fg_color != other.title_fg_color {
            if let Some(new_value) = &other.title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_bg_color != other.title_bg_color {
            if let Some(new_value) = &other.title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_title_bg_color != other.selected_title_bg_color {
            if let Some(new_value) = &other.selected_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_title_fg_color != other.selected_title_fg_color {
            if let Some(new_value) = &other.selected_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_position != other.title_position {
            if let Some(new_value) = &other.title_position {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "title_position".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_state != other.error_state {
            updates.push(FieldUpdate {
                entity_type: EntityType::MuxBox,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "error_state".to_string(),
                new_value: serde_json::to_value(other.error_state).unwrap(),
            });
        }

        if self.error_fg_color != other.error_fg_color {
            if let Some(new_value) = &other.error_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_bg_color != other.error_bg_color {
            if let Some(new_value) = &other.error_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_title_fg_color != other.error_title_fg_color {
            if let Some(new_value) = &other.error_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_title_bg_color != other.error_title_bg_color {
            if let Some(new_value) = &other.error_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_border_color != other.error_border_color {
            if let Some(new_value) = &other.error_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_bg_color != other.error_selected_bg_color {
            if let Some(new_value) = &other.error_selected_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_fg_color != other.error_selected_fg_color {
            if let Some(new_value) = &other.error_selected_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_border_color != other.error_selected_border_color {
            if let Some(new_value) = &other.error_selected_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_title_bg_color != other.error_selected_title_bg_color {
            if let Some(new_value) = &other.error_selected_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_title_fg_color != other.error_selected_title_fg_color {
            if let Some(new_value) = &other.error_selected_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "error_selected_title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.choices != other.choices {
            if let Some(new_value) = &other.choices {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "choices".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.menu_fg_color != other.menu_fg_color {
            if let Some(new_value) = &other.menu_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "menu_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.menu_bg_color != other.menu_bg_color {
            if let Some(new_value) = &other.menu_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "menu_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_menu_fg_color != other.selected_menu_fg_color {
            if let Some(new_value) = &other.selected_menu_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_menu_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_menu_bg_color != other.selected_menu_bg_color {
            if let Some(new_value) = &other.selected_menu_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected_menu_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.redirect_output != other.redirect_output {
            if let Some(new_value) = &other.redirect_output {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "redirect_output".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.script != other.script {
            if let Some(new_value) = &other.script {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "script".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        // T0330: thread field removed - tracked via execution_mode instead

        if self.on_keypress != other.on_keypress {
            if let Some(new_value) = &other.on_keypress {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "on_keypress".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.horizontal_scroll != other.horizontal_scroll {
            if let Some(new_value) = other.horizontal_scroll {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "horizontal_scroll".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.vertical_scroll != other.vertical_scroll {
            if let Some(new_value) = other.vertical_scroll {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "vertical_scroll".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected != other.selected {
            if let Some(new_value) = other.selected {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "selected".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.content != other.content {
            if let Some(new_value) = &other.content {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "content".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.output != other.output {
            updates.push(FieldUpdate {
                entity_type: EntityType::MuxBox,
                entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                field_name: "output".to_string(),
                new_value: serde_json::to_value(&other.output).unwrap(),
            });
        }

        if self.parent_id != other.parent_id {
            if let Some(new_value) = &other.parent_id {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
                    entity_id: Some(self.id.clone()), // Use clone to break the lifetime dependency
                    field_name: "parent_id".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.parent_layout_id != other.parent_layout_id {
            if let Some(new_value) = &other.parent_layout_id {
                updates.push(FieldUpdate {
                    entity_type: EntityType::MuxBox,
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
            if update.entity_type != EntityType::MuxBox {
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
                // T0330: "thread" field removed - use "execution_mode" field instead
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
                "auto_scroll_bottom" => {
                    if let Ok(new_auto_scroll_bottom) =
                        serde_json::from_value::<Option<bool>>(update.new_value.clone())
                    {
                        self.auto_scroll_bottom = new_auto_scroll_bottom;
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
                _ => log::warn!("Unknown field name for MuxBox: {}", update.field_name),
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

    /// Creates a basic test muxbox with minimal required fields.
    /// This helper demonstrates how to create a MuxBox for testing purposes.
    fn create_test_muxbox(id: &str) -> MuxBox {
        MuxBox {
            id: id.to_string(),
            title: Some(format!("Test MuxBox {}", id)),
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
            redirect_output: None,
            append_output: None,
            execution_mode: ExecutionMode::default(),
            selected: false,
            waiting: false,
        }
    }

    /// Creates a test AppContext with a simple layout for testing.
    /// This helper demonstrates how to create an AppContext for MuxBox testing.
    fn create_test_app_context() -> AppContext {
        let mut app = App::new();
        let layout = Layout {
            id: "test_layout".to_string(),
            children: Some(vec![create_test_muxbox("test_muxbox")]),
            ..Default::default()
        };
        app.layouts.push(layout);
        AppContext::new(app, Config::default())
    }

    // === MuxBox Default Tests ===

    /// Tests that MuxBox::default() creates a muxbox with expected default values.
    /// This test demonstrates the default MuxBox construction behavior.
    #[test]
    fn test_muxbox_default() {
        let muxbox = MuxBox::default();
        assert_eq!(muxbox.id, "");
        assert_eq!(muxbox.title, None);
        assert_eq!(muxbox.anchor, Anchor::Center);
        assert_eq!(muxbox.selected, Some(false));
        assert_eq!(
            muxbox.execution_mode,
            crate::model::common::ExecutionMode::default()
        );
        assert_eq!(muxbox.horizontal_scroll, Some(0.0));
        assert_eq!(muxbox.vertical_scroll, Some(0.0));
        assert_eq!(muxbox.error_state, false);
        assert_eq!(muxbox.output, "");
    }

    // === MuxBox Creation Tests ===

    /// Tests creating a MuxBox with specific values.
    /// This test demonstrates how to create a MuxBox with custom properties.
    #[test]
    fn test_muxbox_creation() {
        let muxbox = MuxBox {
            id: "test_muxbox".to_string(),
            title: Some("Test MuxBox".to_string()),
            position: InputBounds {
                x1: "10%".to_string(),
                y1: "20%".to_string(),
                x2: "90%".to_string(),
                y2: "80%".to_string(),
            },
            anchor: Anchor::TopLeft,
            selected: Some(true),
            content: Some("Test content".to_string()),
            ..Default::default()
        };

        assert_eq!(muxbox.id, "test_muxbox");
        assert_eq!(muxbox.title, Some("Test MuxBox".to_string()));
        assert_eq!(muxbox.anchor, Anchor::TopLeft);
        assert_eq!(muxbox.selected, Some(true));
        assert_eq!(muxbox.content, Some("Test content".to_string()));
    }

    // === MuxBox Bounds Tests ===

    /// Tests that MuxBox::bounds() calculates bounds correctly.
    /// This test demonstrates the bounds calculation feature using screen bounds.
    #[test]
    fn test_muxbox_bounds() {
        let muxbox = MuxBox {
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
        let bounds = muxbox.bounds();
        let screen_bounds = crate::utils::screen_bounds();

        // Calculate expected values using the new coordinate mapping logic
        let expected_x1 = (0.25 * (screen_bounds.width() - 1) as f64).round() as usize;
        let expected_y1 = (0.50 * (screen_bounds.height() - 1) as f64).round() as usize;
        let expected_x2 = (0.75 * (screen_bounds.width() - 1) as f64).round() as usize;
        let expected_y2 = screen_bounds.height() - 1; // 100% maps to last coordinate

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

    /// Tests that MuxBox::absolute_bounds() works with parent bounds.
    /// This test demonstrates the absolute bounds calculation feature.
    #[test]
    fn test_muxbox_absolute_bounds() {
        let muxbox = MuxBox {
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
        let bounds = muxbox.absolute_bounds(Some(&parent_bounds));

        assert_eq!(bounds.x1, 25);
        assert_eq!(bounds.y1, 100);
        assert_eq!(bounds.x2, 75); // 75% of 0-99 range = 75
        assert_eq!(bounds.y2, 200); // 100% of 0-200 range = 200
    }

    /// Tests that MuxBox::update_bounds_absolutely() updates position correctly.
    /// This test demonstrates the bounds update feature.
    #[test]
    fn test_muxbox_update_bounds_absolutely() {
        let mut muxbox = MuxBox {
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

        muxbox.update_bounds_absolutely(new_bounds, Some(&parent_bounds));

        assert_eq!(muxbox.position.x1, "25%");
        assert_eq!(muxbox.position.y1, "25%");
        assert_eq!(muxbox.position.x2, "75%");
        assert_eq!(muxbox.position.y2, "50%");
    }

    // === MuxBox Output Tests ===

    /// Tests that MuxBox::set_output() updates the output field.
    /// This test demonstrates the output setting feature.
    #[test]
    fn test_muxbox_set_output() {
        let mut muxbox = MuxBox::default();
        assert_eq!(muxbox.output, "");

        muxbox.set_output("Test output");
        assert_eq!(muxbox.output, "Test output");
    }

    /// Tests that MuxBox::update_content() updates content and error state.
    /// This test demonstrates the content update feature.
    #[test]
    fn test_muxbox_update_content() {
        let mut muxbox = MuxBox::default();

        // Test successful update
        muxbox.update_content("New content", false, true);
        assert_eq!(muxbox.content, Some("New content".to_string()));
        assert_eq!(muxbox.error_state, false);

        // Test error update
        muxbox.update_content("Error content", false, false);
        assert_eq!(muxbox.content, Some("Error content".to_string()));
        assert_eq!(muxbox.error_state, true);
    }

    /// Tests that MuxBox::update_content() handles content appending.
    /// This test demonstrates the content appending feature.
    #[test]
    fn test_muxbox_update_content_append() {
        let mut muxbox = MuxBox {
            content: Some("Existing content".to_string()),
            ..Default::default()
        };

        muxbox.update_content("New content", true, true);

        // Check that content was appended with timestamp
        let content = muxbox.content.unwrap();
        assert!(content.contains("New content"));
        assert!(content.contains("Existing content"));
        assert!(content.contains("[")); // Timestamp format
    }

    /// Tests that MuxBox::update_content() preserves scroll position during content updates.
    /// This test demonstrates the scroll position preservation feature.
    #[test]
    fn test_muxbox_update_content_preserves_scroll_position() {
        let mut muxbox = MuxBox {
            content: Some("Original content".to_string()),
            horizontal_scroll: Some(25.0),
            vertical_scroll: Some(75.0),
            ..Default::default()
        };

        // Update content and verify scroll position is preserved
        muxbox.update_content("Updated content", false, true);

        assert_eq!(muxbox.content, Some("Updated content".to_string()));
        assert_eq!(muxbox.horizontal_scroll, Some(25.0));
        assert_eq!(muxbox.vertical_scroll, Some(75.0));
        assert_eq!(muxbox.error_state, false);
    }

    /// Tests that MuxBox::update_content() preserves scroll position during append operations.
    /// This test demonstrates scroll preservation with content appending.
    #[test]
    fn test_muxbox_update_content_append_preserves_scroll_position() {
        let mut muxbox = MuxBox {
            content: Some("Existing content".to_string()),
            horizontal_scroll: Some(10.0),
            vertical_scroll: Some(50.0),
            ..Default::default()
        };

        // Append content and verify scroll position is preserved
        muxbox.update_content("New content", true, true);

        // Check scroll position preservation
        assert_eq!(muxbox.horizontal_scroll, Some(10.0));
        assert_eq!(muxbox.vertical_scroll, Some(50.0));
        assert_eq!(muxbox.error_state, false);

        // Verify content was appended
        let content = muxbox.content.unwrap();
        assert!(content.contains("New content"));
        assert!(content.contains("Existing content"));
    }

    /// Tests that MuxBox::update_content() preserves scroll position even when muxbox has no initial scroll values.
    /// This test demonstrates scroll preservation with None values.
    #[test]
    fn test_muxbox_update_content_preserves_none_scroll_values() {
        let mut muxbox = MuxBox {
            content: Some("Original content".to_string()),
            horizontal_scroll: None,
            vertical_scroll: None,
            ..Default::default()
        };

        // Update content and verify None scroll values are preserved
        muxbox.update_content("Updated content", false, true);

        assert_eq!(muxbox.content, Some("Updated content".to_string()));
        assert_eq!(muxbox.horizontal_scroll, None);
        assert_eq!(muxbox.vertical_scroll, None);
    }

    /// Tests that MuxBox scrolling supports larger amounts for page-based scrolling.
    /// This test demonstrates the page scrolling feature with larger scroll amounts.
    #[test]
    fn test_muxbox_page_scrolling_large_amounts() {
        let mut muxbox = MuxBox {
            vertical_scroll: Some(50.0),
            horizontal_scroll: Some(30.0),
            ..Default::default()
        };

        // Test page down (large scroll down)
        muxbox.scroll_down(Some(10.0));
        assert_eq!(muxbox.current_vertical_scroll(), 60.0);

        // Test page up (large scroll up)
        muxbox.scroll_up(Some(10.0));
        assert_eq!(muxbox.current_vertical_scroll(), 50.0);

        // Test page scrolling doesn't exceed bounds
        muxbox.vertical_scroll = Some(95.0);
        muxbox.scroll_down(Some(10.0));
        assert_eq!(muxbox.current_vertical_scroll(), 100.0); // Capped at 100

        muxbox.vertical_scroll = Some(5.0);
        muxbox.scroll_up(Some(10.0));
        assert_eq!(muxbox.current_vertical_scroll(), 0.0); // Capped at 0
    }

    /// Tests that MuxBox page scrolling works correctly with boundary conditions.
    /// This test demonstrates page scrolling boundary handling.
    #[test]
    fn test_muxbox_page_scrolling_boundaries() {
        let mut muxbox = MuxBox::default();

        // Test page down from initial position (None -> Some)
        muxbox.scroll_down(Some(10.0));
        assert_eq!(muxbox.current_vertical_scroll(), 10.0);

        // Test page up from initial position (None -> Some)
        muxbox.vertical_scroll = None;
        muxbox.scroll_up(Some(10.0));
        assert_eq!(muxbox.current_vertical_scroll(), 0.0); // Can't go below 0

        // Test large page movements near boundaries
        muxbox.vertical_scroll = Some(98.0);
        muxbox.scroll_down(Some(10.0));
        assert_eq!(muxbox.current_vertical_scroll(), 100.0); // Should cap at 100

        muxbox.vertical_scroll = Some(2.0);
        muxbox.scroll_up(Some(10.0));
        assert_eq!(muxbox.current_vertical_scroll(), 0.0); // Should cap at 0
    }

    // === MuxBox Selection Tests ===

    /// Tests that MuxBox::is_selectable() correctly identifies selectable muxboxes.
    /// This test demonstrates the muxbox selectability feature.
    #[test]
    fn test_muxbox_is_selectable() {
        let mut muxbox = MuxBox::default();

        // MuxBox without tab_order is not selectable
        assert!(!muxbox.is_selectable());

        // MuxBox with tab_order "none" is not selectable
        muxbox.tab_order = Some("none".to_string());
        assert!(!muxbox.is_selectable());

        // MuxBox with numeric tab_order is selectable
        muxbox.tab_order = Some("1".to_string());
        assert!(muxbox.is_selectable());
    }

    /// Tests that MuxBox::is_selected() correctly identifies selected muxboxes.
    /// This test demonstrates the muxbox selection state feature.
    #[test]
    fn test_muxbox_is_selected() {
        let mut muxbox = MuxBox::default();

        // Default muxbox is not selected
        assert!(!muxbox.is_selected());

        // MuxBox with selected = Some(true) is selected
        muxbox.selected = Some(true);
        assert!(muxbox.is_selected());

        // MuxBox with selected = Some(false) is not selected
        muxbox.selected = Some(false);
        assert!(!muxbox.is_selected());
    }

    // === MuxBox Scrolling Tests ===

    /// Tests that MuxBox::scroll_down() increases vertical scroll.
    /// This test demonstrates the downward scrolling feature.
    #[test]
    fn test_muxbox_scroll_down() {
        let mut muxbox = MuxBox::default();

        // Initial scroll should be 0
        assert_eq!(muxbox.current_vertical_scroll(), 0.0);

        // Scroll down by default amount
        muxbox.scroll_down(None);
        assert_eq!(muxbox.current_vertical_scroll(), 5.0);

        // Scroll down by custom amount
        muxbox.scroll_down(Some(10.0));
        assert_eq!(muxbox.current_vertical_scroll(), 15.0);

        // Scroll should not exceed 100%
        muxbox.scroll_down(Some(90.0));
        assert_eq!(muxbox.current_vertical_scroll(), 100.0);
    }

    /// Tests that MuxBox::scroll_up() decreases vertical scroll.
    /// This test demonstrates the upward scrolling feature.
    #[test]
    fn test_muxbox_scroll_up() {
        let mut muxbox = MuxBox {
            vertical_scroll: Some(50.0),
            ..Default::default()
        };

        // Scroll up by default amount
        muxbox.scroll_up(None);
        assert_eq!(muxbox.current_vertical_scroll(), 45.0);

        // Scroll up by custom amount
        muxbox.scroll_up(Some(20.0));
        assert_eq!(muxbox.current_vertical_scroll(), 25.0);

        // Scroll should not go below 0
        muxbox.scroll_up(Some(50.0));
        assert_eq!(muxbox.current_vertical_scroll(), 0.0);
    }

    /// Tests that MuxBox::scroll_right() increases horizontal scroll.
    /// This test demonstrates the rightward scrolling feature.
    #[test]
    fn test_muxbox_scroll_right() {
        let mut muxbox = MuxBox::default();

        // Initial scroll should be 0
        assert_eq!(muxbox.current_horizontal_scroll(), 0.0);

        // Scroll right by default amount
        muxbox.scroll_right(None);
        assert_eq!(muxbox.current_horizontal_scroll(), 5.0);

        // Scroll right by custom amount
        muxbox.scroll_right(Some(10.0));
        assert_eq!(muxbox.current_horizontal_scroll(), 15.0);

        // Scroll should not exceed 100%
        muxbox.scroll_right(Some(90.0));
        assert_eq!(muxbox.current_horizontal_scroll(), 100.0);
    }

    /// Tests that MuxBox::scroll_left() decreases horizontal scroll.
    /// This test demonstrates the leftward scrolling feature.
    #[test]
    fn test_muxbox_scroll_left() {
        let mut muxbox = MuxBox {
            horizontal_scroll: Some(50.0),
            ..Default::default()
        };

        // Scroll left by default amount
        muxbox.scroll_left(None);
        assert_eq!(muxbox.current_horizontal_scroll(), 45.0);

        // Scroll left by custom amount
        muxbox.scroll_left(Some(20.0));
        assert_eq!(muxbox.current_horizontal_scroll(), 25.0);

        // Scroll should not go below 0
        muxbox.scroll_left(Some(50.0));
        assert_eq!(muxbox.current_horizontal_scroll(), 0.0);
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
        assert_eq!(
            choice.execution_mode,
            crate::model::common::ExecutionMode::default()
        );
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

    // === MuxBox Clone Tests ===

    /// Tests that MuxBox implements Clone correctly.
    /// This test demonstrates MuxBox cloning behavior.
    #[test]
    fn test_muxbox_clone() {
        let muxbox1 = create_test_muxbox("test");
        let muxbox2 = muxbox1.clone();

        assert_eq!(muxbox1, muxbox2);
        assert_eq!(muxbox1.id, muxbox2.id);
        assert_eq!(muxbox1.title, muxbox2.title);
        assert_eq!(muxbox1.position, muxbox2.position);
    }

    /// Tests that MuxBox cloning includes children.
    /// This test demonstrates MuxBox cloning with nested children.
    #[test]
    fn test_muxbox_clone_with_children() {
        let child_muxbox = create_test_muxbox("child");
        let parent_muxbox = MuxBox {
            id: "parent".to_string(),
            children: Some(vec![child_muxbox]),
            ..Default::default()
        };

        let cloned = parent_muxbox.clone();

        assert_eq!(parent_muxbox.children.as_ref().unwrap().len(), 1);
        assert_eq!(cloned.children.as_ref().unwrap().len(), 1);
        assert_eq!(
            parent_muxbox.children.as_ref().unwrap()[0].id,
            cloned.children.as_ref().unwrap()[0].id
        );
    }

    // === MuxBox Hash Tests ===

    /// Tests that MuxBox implements Hash correctly.
    /// This test demonstrates MuxBox hashing behavior.
    #[test]
    fn test_muxbox_hash() {
        let muxbox1 = create_test_muxbox("test");
        let muxbox2 = create_test_muxbox("test");
        let muxbox3 = create_test_muxbox("other");

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        muxbox1.hash(&mut hasher1);
        muxbox2.hash(&mut hasher2);
        muxbox3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    // === MuxBox Validation Tests ===

    /// Tests that MuxBox handles edge cases in scrolling.
    /// This test demonstrates edge case handling in scrolling methods.
    #[test]
    fn test_muxbox_scrolling_edge_cases() {
        let mut muxbox = MuxBox::default();

        // Test scrolling when scroll values are None
        muxbox.vertical_scroll = None;
        muxbox.horizontal_scroll = None;

        muxbox.scroll_down(Some(10.0));
        assert_eq!(muxbox.vertical_scroll, Some(10.0));

        muxbox.scroll_right(Some(15.0));
        assert_eq!(muxbox.horizontal_scroll, Some(15.0));

        muxbox.scroll_up(Some(5.0));
        assert_eq!(muxbox.vertical_scroll, Some(5.0));

        muxbox.scroll_left(Some(10.0));
        assert_eq!(muxbox.horizontal_scroll, Some(5.0));
    }

    /// Tests that MuxBox handles empty and None values correctly.
    /// This test demonstrates edge case handling in MuxBox properties.
    #[test]
    fn test_muxbox_empty_values() {
        let muxbox = MuxBox {
            id: "test".to_string(),
            title: Some("".to_string()),
            content: Some("".to_string()),
            tab_order: Some("".to_string()),
            ..Default::default()
        };

        assert_eq!(muxbox.id, "test");
        assert_eq!(muxbox.title, Some("".to_string()));
        assert_eq!(muxbox.content, Some("".to_string()));
        assert!(!muxbox.is_selectable()); // Empty tab_order should not be selectable
    }

    #[test]
    fn test_muxbox_scrollable_content_selectability() {
        // Test that muxboxes with scrollable content become selectable even without tab_order
        let mut muxbox = MuxBox {
            id: "test_scrollable".to_string(),
            content: Some("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nVery long line that would exceed normal muxbox width to trigger horizontal scrolling".to_string()),
            position: InputBounds {
                x1: "0".to_string(),
                y1: "0".to_string(), 
                x2: "20".to_string(),  // Small width to trigger overflow
                y2: "5".to_string(),   // Small height to trigger overflow
            },
            anchor: Anchor::TopLeft,
            tab_order: None, // No explicit tab order
            ..Default::default()
        };

        assert!(
            muxbox.has_scrollable_content(),
            "MuxBox with large content should have scrollable content"
        );
        assert!(
            muxbox.is_selectable(),
            "MuxBox with scrollable content should be selectable even without tab_order"
        );

        // Test that empty content is not scrollable
        muxbox.content = Some("".to_string());
        assert!(
            !muxbox.has_scrollable_content(),
            "MuxBox with empty content should not have scrollable content"
        );
        assert!(
            !muxbox.is_selectable(),
            "MuxBox with empty content should not be selectable without tab_order"
        );

        // Test that normal sized content is not scrollable
        muxbox.content = Some("Short".to_string());
        muxbox.position = InputBounds {
            x1: "0".to_string(),
            y1: "0".to_string(),
            x2: "100".to_string(), // Large enough to fit content
            y2: "50".to_string(),  // Large enough to fit content
        };
        assert!(
            !muxbox.has_scrollable_content(),
            "MuxBox with content that fits should not have scrollable content"
        );
        assert!(
            !muxbox.is_selectable(),
            "MuxBox with non-scrollable content should not be selectable without tab_order"
        );

        // But with tab_order it should still be selectable
        muxbox.tab_order = Some("1".to_string());
        assert!(
            muxbox.is_selectable(),
            "MuxBox with tab_order should always be selectable"
        );
    }

    #[test]
    fn test_muxbox_scrollable_content_with_choices() {
        // Test that muxboxes with many choices are correctly detected as scrollable
        let mut muxbox = MuxBox {
            id: "test_choice_scroll".to_string(),
            position: InputBounds {
                x1: "0".to_string(),
                y1: "0".to_string(),
                x2: "50".to_string(), // Moderate width
                y2: "20".to_string(), // Moderate height
            },
            anchor: Anchor::TopLeft,
            tab_order: None,
            choices: None,
            content: None,
            ..Default::default()
        };

        // Test: MuxBox with no choices should not be scrollable
        assert!(
            !muxbox.has_scrollable_content(),
            "Empty muxbox should not be scrollable"
        );

        // Test: MuxBox with few choices that fit should not be scrollable
        muxbox.choices = Some(vec![
            create_test_choice("choice1", "Item 1"),
            create_test_choice("choice2", "Item 2"),
            create_test_choice("choice3", "Item 3"),
        ]);
        assert!(
            !muxbox.has_scrollable_content(),
            "MuxBox with few choices should not be scrollable"
        );

        // Test: MuxBox with many choices should be scrollable (vertical overflow)
        let many_choices: Vec<Choice> = (0..25)
            .map(|i| create_test_choice(&format!("choice{}", i), &format!("Menu Item {}", i)))
            .collect();
        muxbox.choices = Some(many_choices);
        assert!(
            muxbox.has_scrollable_content(),
            "MuxBox with many choices should be scrollable"
        );

        // Test: MuxBox with wide choice content should be scrollable (horizontal overflow)
        muxbox.choices = Some(vec![
            create_test_choice("wide_choice", "This is a very long menu choice that definitely exceeds the muxbox width and should trigger horizontal scrolling")
        ]);
        assert!(
            muxbox.has_scrollable_content(),
            "MuxBox with wide choice content should be scrollable"
        );

        // Test: Large muxbox with choices should not be scrollable
        muxbox.position = InputBounds {
            x1: "0".to_string(),
            y1: "0".to_string(),
            x2: "200".to_string(), // Very large width
            y2: "100".to_string(), // Very large height
        };
        muxbox.choices = Some(vec![
            create_test_choice("choice1", "Item 1"),
            create_test_choice("choice2", "Item 2"),
        ]);
        muxbox.content = None;
        assert!(
            !muxbox.has_scrollable_content(),
            "Large muxbox with few choices should not be scrollable"
        );
    }

    // === MuxBox with Choices Tests ===

    /// Tests that MuxBox correctly handles choices.
    /// This test demonstrates MuxBox choice management.
    #[test]
    fn test_muxbox_with_choices() {
        let choice1 = create_test_choice("choice1", "First Choice");
        let choice2 = create_test_choice("choice2", "Second Choice");

        let muxbox = MuxBox {
            id: "test".to_string(),
            choices: Some(vec![choice1, choice2]),
            ..Default::default()
        };

        assert_eq!(muxbox.choices.as_ref().unwrap().len(), 2);
        assert_eq!(muxbox.choices.as_ref().unwrap()[0].id, "choice1");
        assert_eq!(muxbox.choices.as_ref().unwrap()[1].id, "choice2");
    }

    /// Tests that MuxBox correctly handles choice selection.
    /// This test demonstrates MuxBox choice selection behavior.
    #[test]
    fn test_muxbox_choice_selection() {
        let mut choice1 = create_test_choice("choice1", "First Choice");
        let mut choice2 = create_test_choice("choice2", "Second Choice");

        choice1.selected = true;
        choice2.selected = false;

        let muxbox = MuxBox {
            id: "test".to_string(),
            choices: Some(vec![choice1, choice2]),
            ..Default::default()
        };

        assert_eq!(muxbox.choices.as_ref().unwrap()[0].selected, true);
        assert_eq!(muxbox.choices.as_ref().unwrap()[1].selected, false);
    }

    // === MuxBox PartialEq Tests ===

    /// Tests that MuxBox implements PartialEq correctly.
    /// This test demonstrates MuxBox equality comparison.
    #[test]
    fn test_muxbox_equality() {
        let muxbox1 = create_test_muxbox("test");
        let muxbox2 = create_test_muxbox("test");
        let muxbox3 = create_test_muxbox("other");

        assert_eq!(muxbox1, muxbox2);
        assert_ne!(muxbox1, muxbox3);
    }

    /// Tests that MuxBox equality considers all fields.
    /// This test demonstrates comprehensive MuxBox equality checking.
    #[test]
    fn test_muxbox_equality_comprehensive() {
        let muxbox1 = MuxBox {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            selected: Some(true),
            ..Default::default()
        };

        let muxbox2 = MuxBox {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            selected: Some(true),
            ..Default::default()
        };

        let muxbox3 = MuxBox {
            id: "test".to_string(),
            title: Some("Different Title".to_string()), // Make it different
            selected: Some(true),
            ..Default::default()
        };

        assert_eq!(muxbox1, muxbox2);
        assert_ne!(muxbox1, muxbox3);
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

        let muxbox: MuxBox = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = muxbox.script.expect("Script should be present");

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

        let muxbox: MuxBox = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = muxbox.script.expect("Script should be present");

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

        let muxbox: MuxBox = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = muxbox.script.expect("Script should be present");

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

        let muxbox: MuxBox = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        assert_eq!(muxbox.script, None);
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

        let muxbox: MuxBox = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = muxbox.script.expect("Script should be present");

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

        let muxbox: MuxBox = serde_yaml::from_str(yaml).expect("Should deserialize successfully");
        let script = muxbox.script.expect("Script should be present");

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

        let result = serde_yaml::from_str::<MuxBox>(yaml);
        assert!(
            result.is_ok(),
            "Should handle complex scripts without error"
        );

        let muxbox = result.unwrap();
        let script = muxbox.script.expect("Script should be present");

        assert_eq!(script.len(), 3);
        assert_eq!(script[0], "echo normal");
        assert!(script[1].contains("for i in"));
        assert!(script[1].contains("echo \"Line $i\""));
        assert_eq!(script[2], "echo final");
    }
}
