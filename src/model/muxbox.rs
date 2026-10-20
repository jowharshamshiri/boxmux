use crate::model::bounds::{Anchor, InputBounds};
use crate::model::choice::Choice;
use crate::model::common::{deserialize_script, ExecutionMode};
use crate::model::stream::{RunOrigin, Stream, StreamKind};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct MuxBox {
    pub id: String,
    pub title: Option<String>,
    pub position: InputBounds,
    #[serde(default)]
    pub anchor: Anchor,
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
    pub max_width: Option<usize>,
    pub max_height: Option<usize>,
    pub overflow_behavior: Option<crate::model::appearance::Overflow>,
    pub refresh_interval: Option<u64>,
    /// Position of the box in keyboard focus order; boxes without one cannot be focused with Tab.
    pub tab_order: Option<u32>,
    pub next_focus_id: Option<String>,
    pub children: Option<Vec<MuxBox>>,
    pub fill: Option<bool>,
    pub fill_char: Option<char>,
    pub selected_fill_char: Option<char>,
    pub highlighted_fill_char: Option<char>,
    pub border_color: Option<crate::model::appearance::ColorName>,
    pub selected_border_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_border_color: Option<crate::model::appearance::ColorName>,
    pub bg_color: Option<crate::model::appearance::ColorName>,
    pub selected_bg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_bg_color: Option<crate::model::appearance::ColorName>,
    pub fg_color: Option<crate::model::appearance::ColorName>,
    pub selected_fg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_fg_color: Option<crate::model::appearance::ColorName>,
    pub title_fg_color: Option<crate::model::appearance::ColorName>,
    pub title_bg_color: Option<crate::model::appearance::ColorName>,
    pub selected_title_bg_color: Option<crate::model::appearance::ColorName>,
    pub selected_title_fg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_title_bg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_title_fg_color: Option<crate::model::appearance::ColorName>,
    pub title_position: Option<crate::model::appearance::TitlePosition>,
    pub error_border_color: Option<crate::model::appearance::ColorName>,
    pub error_bg_color: Option<crate::model::appearance::ColorName>,
    pub error_fg_color: Option<crate::model::appearance::ColorName>,
    pub error_title_bg_color: Option<crate::model::appearance::ColorName>,
    pub error_title_fg_color: Option<crate::model::appearance::ColorName>,
    pub error_selected_border_color: Option<crate::model::appearance::ColorName>,
    pub error_selected_bg_color: Option<crate::model::appearance::ColorName>,
    pub error_selected_fg_color: Option<crate::model::appearance::ColorName>,
    pub error_selected_title_bg_color: Option<crate::model::appearance::ColorName>,
    pub error_selected_title_fg_color: Option<crate::model::appearance::ColorName>,
    pub choices: Option<Vec<Choice>>,
    pub menu_fg_color: Option<crate::model::appearance::ColorName>,
    pub menu_bg_color: Option<crate::model::appearance::ColorName>,
    pub selected_menu_fg_color: Option<crate::model::appearance::ColorName>,
    pub selected_menu_bg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_menu_fg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_menu_bg_color: Option<crate::model::appearance::ColorName>,
    pub redirect_output: Option<String>,
    pub append_output: Option<bool>,
    #[serde(deserialize_with = "deserialize_script", default)]
    pub script: Option<Vec<String>>,
    #[serde(default)]
    pub on_keypress: Option<HashMap<String, Vec<String>>>,
    #[serde(
        default,
        deserialize_with = "crate::model::variables::deserialize_scope"
    )]
    pub variables: Option<crate::model::variables::Scope>,
    pub horizontal_scroll: Option<f64>,
    pub vertical_scroll: Option<f64>,
    pub selected: Option<bool>,
    pub content: Option<String>,
    pub save_in_file: Option<String>,
    /// The tabs of the box. Runtime state, built from the configuration at load.
    #[serde(skip)]
    pub streams: IndexMap<String, Stream>,
    /// Which tab is visible.
    #[serde(skip)]
    pub selected_stream_id: Option<String>,
    pub chart_type: Option<crate::chart::ChartType>,
    pub chart_data: Option<String>,
    pub plugin_component: Option<String>,
    pub plugin_config: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub table_data: Option<String>,
    pub table_config: Option<crate::table::TableSettings>,
    pub auto_scroll_bottom: Option<bool>,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(default)]
    pub z_index: Option<i32>,
    #[serde(skip)]
    pub tab_scroll_offset: usize,
    /// Id of the enclosing box, set when the configuration is loaded.
    #[serde(skip)]
    pub parent_id: Option<String>,
    /// Id of the enclosing layout, set when the configuration is loaded.
    #[serde(skip)]
    pub parent_layout_id: Option<String>,
}

impl MuxBox {
    pub fn effective_z_index(&self) -> i32 {
        self.z_index.unwrap_or(0)
    }

    pub fn current_vertical_scroll(&self) -> f64 {
        self.vertical_scroll.unwrap_or(0.0)
    }

    pub fn current_horizontal_scroll(&self) -> f64 {
        self.horizontal_scroll.unwrap_or(0.0)
    }

    /// Whether the box shows its error colors: the visible stream's last run failed.
    pub fn in_error_state(&self) -> bool {
        self.get_selected_stream()
            .is_some_and(|stream| stream.status.is_error())
    }

    /// Builds the streams the configuration defines: the static content, the choice menu and the
    /// output tab of the box's own script, in that order. Runtime streams (choice output, socket
    /// content) are added later and never touched here.
    pub fn initialize_streams(&mut self) {
        let has_content = self
            .content
            .as_ref()
            .is_some_and(|content| !content.is_empty());
        let has_choices = self
            .choices
            .as_ref()
            .is_some_and(|choices| !choices.is_empty());
        let has_script = self.script.is_some();
        let defined = [has_content, has_choices, has_script]
            .iter()
            .filter(|defined| **defined)
            .count();
        // A box with a single tab shows its title on that tab; with several, each tab is named
        // for what it holds.
        let sole_label = |kind_label: &str| -> String {
            if defined == 1 {
                self.title.clone().unwrap_or_else(|| kind_label.to_string())
            } else {
                kind_label.to_string()
            }
        };

        let mut streams = IndexMap::new();
        if has_content {
            let content = self.content.as_deref().unwrap_or_default();
            let stream = Stream::content(
                format!("{}/content", self.id),
                sole_label("Content"),
                crate::text::sanitize_output(content),
            );
            streams.insert(stream.id.clone(), stream);
        }
        if has_choices {
            let mut choices = self.choices.clone().unwrap_or_default();
            if let Some(first) = choices.first_mut() {
                first.selected = true;
            }
            let stream = Stream::choices(
                format!("{}/choices", self.id),
                sole_label("Choices"),
                choices,
            );
            streams.insert(stream.id.clone(), stream);
        }
        if has_script {
            let origin = RunOrigin::BoxScript {
                box_id: self.id.clone(),
            };
            let stream = Stream::run(
                origin.stream_id(&self.id),
                sole_label("Output"),
                origin,
                self.execution_mode.is_pty(),
            );
            streams.insert(stream.id.clone(), stream);
        }
        self.selected_stream_id = streams.keys().next().cloned();
        self.streams = streams;
    }

    /// The stream for `origin` in this box, created at the end of the tab list when missing.
    pub fn stream_for_origin(
        &mut self,
        origin: &RunOrigin,
        label: &str,
        terminal: bool,
    ) -> &mut Stream {
        let stream_id = origin.stream_id(&self.id);
        if !self.streams.contains_key(&stream_id) {
            let stream = Stream::run(
                stream_id.clone(),
                label.to_string(),
                origin.clone(),
                terminal,
            );
            self.streams.insert(stream_id.clone(), stream);
        }
        self.streams
            .get_mut(&stream_id)
            .expect("stream was inserted above")
    }

    /// Removes a stream. When it was the visible one, the first remaining stream becomes visible.
    pub fn remove_stream(&mut self, stream_id: &str) -> Option<Stream> {
        let removed = self.streams.shift_remove(stream_id)?;
        if self.selected_stream_id.as_deref() == Some(stream_id) {
            self.selected_stream_id = self.streams.keys().next().cloned();
        }
        let max_offset = self.streams.len().saturating_sub(1);
        self.tab_scroll_offset = self.tab_scroll_offset.min(max_offset);
        Some(removed)
    }

    /// The visible stream: the selected one, or the first one when nothing is selected.
    pub fn get_selected_stream(&self) -> Option<&Stream> {
        self.selected_stream_id
            .as_ref()
            .and_then(|id| self.streams.get(id))
            .or_else(|| self.streams.values().next())
    }

    pub fn get_selected_stream_mut(&mut self) -> Option<&mut Stream> {
        let selected = self
            .selected_stream_id
            .clone()
            .filter(|id| self.streams.contains_key(id));
        match selected {
            Some(id) => self.streams.get_mut(&id),
            None => self.streams.values_mut().next(),
        }
    }

    /// The menu of the visible stream, when the visible stream is a menu.
    pub fn get_selected_stream_choices(&self) -> Option<&Vec<Choice>> {
        self.get_selected_stream().and_then(|s| s.choices.as_ref())
    }

    pub fn get_selected_stream_choices_mut(&mut self) -> Option<&mut Vec<Choice>> {
        self.get_selected_stream_mut()
            .and_then(|s| s.choices.as_mut())
    }

    /// The menu of the box, whichever tab is visible.
    pub fn choices_stream(&self) -> Option<&Stream> {
        self.streams
            .values()
            .find(|s| matches!(s.kind, StreamKind::Choices))
    }

    pub fn choices_stream_mut(&mut self) -> Option<&mut Stream> {
        self.streams
            .values_mut()
            .find(|s| matches!(s.kind, StreamKind::Choices))
    }

    /// Finds a choice of this box's menu by id.
    pub fn choice_mut(&mut self, choice_id: &str) -> Option<&mut Choice> {
        self.choices_stream_mut()
            .and_then(|stream| stream.choices.as_mut())
            .and_then(|choices| choices.iter_mut().find(|c| c.id == choice_id))
    }

    pub fn switch_to_tab(&mut self, tab_index: usize) -> bool {
        match self.streams.get_index(tab_index) {
            Some((id, _)) => {
                self.selected_stream_id = Some(id.clone());
                true
            }
            None => false,
        }
    }

    pub fn switch_to_stream(&mut self, stream_id: &str) -> bool {
        if self.streams.contains_key(stream_id) {
            self.selected_stream_id = Some(stream_id.to_string());
            true
        } else {
            false
        }
    }

    /// Index of the visible stream's tab.
    pub fn get_active_tab_index(&self) -> usize {
        self.selected_stream_id
            .as_ref()
            .and_then(|id| self.streams.get_index_of(id))
            .unwrap_or(0)
    }

    pub fn get_tab_labels(&self) -> Vec<String> {
        self.streams.values().map(|s| s.label.clone()).collect()
    }

    /// Whether each tab (in tab order) has a close button.
    pub fn get_tab_close_buttons(&self) -> Vec<bool> {
        self.streams
            .values()
            .map(|s| s.is_closeable(&self.id))
            .collect()
    }

    /// Stream ids in tab order.
    pub fn get_tab_stream_ids(&self) -> Vec<String> {
        self.streams.keys().cloned().collect()
    }

    pub fn scroll_tabs_left(&mut self) {
        self.tab_scroll_offset = self.tab_scroll_offset.saturating_sub(1);
    }

    /// Scrolls the tab bar right, never past the point where the last tab is the first visible.
    pub fn scroll_tabs_right(&mut self) {
        if self.tab_scroll_offset + 1 < self.streams.len() {
            self.tab_scroll_offset += 1;
        }
    }

    /// Index of the selected choice of the box's menu.
    pub fn selected_choice_index(&self) -> Option<usize> {
        self.choices_stream()
            .and_then(|stream| stream.choices.as_ref())
            .and_then(|choices| choices.iter().position(|choice| choice.selected))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::stream::StreamStatus;

    fn choice(id: &str) -> Choice {
        Choice {
            id: id.to_string(),
            content: Some(format!("Choice {}", id)),
            script: Some(vec![format!("echo {}", id)]),
            redirect_output: None,
            append_output: None,
            execution_mode: ExecutionMode::default(),
            selected: false,
            waiting: false,
        }
    }

    fn muxbox_from_yaml(yaml: &str) -> MuxBox {
        let mut muxbox: MuxBox = serde_yaml::from_str(yaml).expect("valid box");
        muxbox.initialize_streams();
        muxbox
    }

    #[test]
    fn unknown_keys_are_rejected_instead_of_ignored() {
        let result: Result<MuxBox, _> = serde_yaml::from_str(
            "id: a\nposition: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}\npty: true\n",
        );
        let message = result.expect_err("`pty` is not a box field").to_string();
        assert!(message.contains("pty"), "{}", message);
    }

    #[test]
    fn runtime_state_is_never_written_back_to_the_configuration() {
        let mut muxbox = muxbox_from_yaml(
            "id: a\nposition: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}\ncontent: hi\nscript: [date]\n",
        );
        muxbox.parent_layout_id = Some("layout".into());
        muxbox.tab_scroll_offset = 1;
        let yaml = serde_yaml::to_string(&muxbox).unwrap();
        for runtime_key in [
            "streams",
            "selected_stream_id",
            "tab_scroll_offset",
            "parent_id",
            "parent_layout_id",
        ] {
            assert!(
                !yaml.contains(runtime_key),
                "{} leaked into:\n{}",
                runtime_key,
                yaml
            );
        }
        let reloaded: MuxBox = serde_yaml::from_str(&yaml).expect("serialized box loads again");
        assert_eq!(reloaded.id, "a");
    }

    #[test]
    fn a_box_with_one_source_names_its_tab_after_the_title() {
        let muxbox = muxbox_from_yaml(
            "id: a\ntitle: Clock\nposition: {x1: 0%, y1: 0%, x2: 100%, y2: 100%}\nscript: [date]\n",
        );
        assert_eq!(muxbox.get_tab_labels(), vec!["Clock"]);
        assert_eq!(muxbox.get_tab_stream_ids(), vec!["a/script"]);
    }

    #[test]
    fn content_choices_and_script_get_one_tab_each_in_configuration_order() {
        let mut muxbox = MuxBox {
            id: "a".into(),
            title: Some("Box".into()),
            content: Some("line 1\nline 2\n".into()),
            choices: Some(vec![choice("c1"), choice("c2")]),
            script: Some(vec!["date".into()]),
            ..MuxBox::default()
        };
        muxbox.initialize_streams();
        assert_eq!(
            muxbox.get_tab_labels(),
            vec!["Content", "Choices", "Output"]
        );
        assert_eq!(
            muxbox.get_selected_stream().unwrap().content,
            vec!["line 1", "line 2"]
        );
        // None of the configuration-defined tabs can be closed.
        assert_eq!(muxbox.get_tab_close_buttons(), vec![false, false, false]);
        // The first choice starts selected so Enter has a target.
        assert_eq!(muxbox.selected_choice_index(), Some(0));
    }

    #[test]
    fn reinitializing_does_not_keep_stale_streams() {
        let mut muxbox = MuxBox {
            id: "a".into(),
            script: Some(vec!["date".into()]),
            ..MuxBox::default()
        };
        muxbox.initialize_streams();
        muxbox.stream_for_origin(
            &RunOrigin::SocketSpawn { box_id: "a".into() },
            "spawned",
            true,
        );
        assert_eq!(muxbox.streams.len(), 2);
        muxbox.script = None;
        muxbox.content = Some("x".into());
        muxbox.initialize_streams();
        assert_eq!(muxbox.get_tab_stream_ids(), vec!["a/content"]);
    }

    #[test]
    fn a_rerun_of_the_same_origin_reuses_its_tab() {
        let mut muxbox = MuxBox {
            id: "a".into(),
            choices: Some(vec![choice("c1")]),
            ..MuxBox::default()
        };
        muxbox.initialize_streams();
        let origin = RunOrigin::Choice {
            box_id: "a".into(),
            choice_id: "c1".into(),
        };
        muxbox
            .stream_for_origin(&origin, "c1", false)
            .set_text("first");
        let stream = muxbox.stream_for_origin(&origin, "c1", false);
        assert_eq!(
            stream.content,
            vec!["first"],
            "the existing tab is returned, not a new one"
        );
        assert_eq!(muxbox.streams.len(), 2);
        assert_eq!(muxbox.get_tab_close_buttons(), vec![false, true]);
    }

    #[test]
    fn removing_the_visible_stream_shows_the_first_remaining_one() {
        let mut muxbox = MuxBox {
            id: "a".into(),
            content: Some("static".into()),
            ..MuxBox::default()
        };
        muxbox.initialize_streams();
        let spawned = muxbox
            .stream_for_origin(&RunOrigin::SocketSpawn { box_id: "a".into() }, "top", true)
            .id
            .clone();
        assert!(muxbox.switch_to_stream(&spawned));
        muxbox.tab_scroll_offset = 1;
        assert!(muxbox.remove_stream(&spawned).is_some());
        assert_eq!(muxbox.get_selected_stream().unwrap().id, "a/content");
        assert_eq!(muxbox.tab_scroll_offset, 0);
        assert!(
            muxbox.remove_stream(&spawned).is_none(),
            "removing twice reports nothing removed"
        );
    }

    #[test]
    fn switching_to_a_missing_tab_or_stream_changes_nothing() {
        let mut muxbox = MuxBox {
            id: "a".into(),
            content: Some("static".into()),
            ..MuxBox::default()
        };
        muxbox.initialize_streams();
        assert!(!muxbox.switch_to_tab(3));
        assert!(!muxbox.switch_to_stream("a/nope"));
        assert_eq!(muxbox.get_active_tab_index(), 0);
    }

    #[test]
    fn the_tab_bar_cannot_scroll_past_the_last_tab() {
        let mut muxbox = MuxBox {
            id: "a".into(),
            content: Some("x".into()),
            script: Some(vec!["date".into()]),
            ..MuxBox::default()
        };
        muxbox.initialize_streams();
        for _ in 0..5 {
            muxbox.scroll_tabs_right();
        }
        assert_eq!(muxbox.tab_scroll_offset, 1);
        for _ in 0..5 {
            muxbox.scroll_tabs_left();
        }
        assert_eq!(muxbox.tab_scroll_offset, 0);
    }

    #[test]
    fn error_colors_follow_the_visible_stream_status() {
        let mut muxbox = MuxBox {
            id: "a".into(),
            content: Some("x".into()),
            script: Some(vec!["false".into()]),
            ..MuxBox::default()
        };
        muxbox.initialize_streams();
        muxbox.streams.get_mut("a/script").unwrap().status = StreamStatus::Exited(1);
        assert!(!muxbox.in_error_state(), "the failed tab is not visible");
        muxbox.switch_to_stream("a/script");
        assert!(muxbox.in_error_state());
        muxbox.streams.get_mut("a/script").unwrap().status = StreamStatus::Killed;
        assert!(
            !muxbox.in_error_state(),
            "a tab the user killed is not an error"
        );
    }

    #[test]
    fn choices_are_found_by_id_whichever_tab_is_visible() {
        let mut muxbox = MuxBox {
            id: "a".into(),
            content: Some("x".into()),
            choices: Some(vec![choice("c1"), choice("c2")]),
            ..MuxBox::default()
        };
        muxbox.initialize_streams();
        assert!(
            muxbox.get_selected_stream_choices().is_none(),
            "content tab is visible"
        );
        muxbox.choice_mut("c2").unwrap().waiting = true;
        assert!(muxbox.choice_mut("missing").is_none());
        let choices = muxbox.choices_stream().unwrap().choices.as_ref().unwrap();
        assert!(choices[1].waiting);
    }
}
