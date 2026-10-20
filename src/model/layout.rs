use crate::model::muxbox::MuxBox;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize, Serialize, Default, PartialEq, Clone)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub id: String,
    pub title: Option<String>,
    pub refresh_interval: Option<u64>,
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
    pub title_position: Option<crate::model::appearance::TitlePosition>,
    pub selected_title_bg_color: Option<crate::model::appearance::ColorName>,
    pub selected_title_fg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_title_bg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_title_fg_color: Option<crate::model::appearance::ColorName>,
    pub menu_fg_color: Option<crate::model::appearance::ColorName>,
    pub menu_bg_color: Option<crate::model::appearance::ColorName>,
    pub selected_menu_fg_color: Option<crate::model::appearance::ColorName>,
    pub selected_menu_bg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_menu_fg_color: Option<crate::model::appearance::ColorName>,
    pub highlighted_menu_bg_color: Option<crate::model::appearance::ColorName>,
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
    pub overflow_behavior: Option<crate::model::appearance::Overflow>,
    pub root: Option<bool>,
    #[serde(default)]
    pub on_keypress: Option<HashMap<String, Vec<String>>>,
    #[serde(
        default,
        deserialize_with = "crate::model::variables::deserialize_scope"
    )]
    pub variables: Option<crate::model::variables::Scope>,
    /// Whether this layout is the one shown. Runtime state; the configuration picks the first
    /// shown layout with `root`.
    #[serde(skip)]
    pub active: bool,
}

impl Layout {
    pub fn get_muxbox_by_id(&self, id: &str) -> Option<&MuxBox> {
        fn recursive_search<'a>(muxboxes: &'a [MuxBox], id: &str) -> Option<&'a MuxBox> {
            for muxbox in muxboxes {
                if muxbox.id == id {
                    return Some(muxbox);
                }
                if let Some(ref children) = muxbox.children {
                    if let Some(found) = recursive_search(children, id) {
                        return Some(found);
                    }
                }
            }
            None
        }

        if let Some(ref children) = self.children {
            recursive_search(children, id)
        } else {
            None
        }
    }

    pub fn get_muxbox_by_id_mut(&mut self, id: &str) -> Option<&mut MuxBox> {
        fn recursive_search<'a>(muxboxes: &'a mut [MuxBox], id: &str) -> Option<&'a mut MuxBox> {
            for muxbox in muxboxes {
                if muxbox.id == id {
                    return Some(muxbox);
                }
                if let Some(ref mut children) = muxbox.children {
                    if let Some(found) = recursive_search(children, id) {
                        return Some(found);
                    }
                }
            }
            None
        }

        if let Some(ref mut children) = self.children {
            recursive_search(children, id)
        } else {
            None
        }
    }

    /// The boxes enclosing `id`, nearest first; `None` when the layout has no such box.
    pub fn ancestors_of(&self, id: &str) -> Option<Vec<&MuxBox>> {
        fn search<'a>(muxboxes: &'a [MuxBox], id: &str, path: &mut Vec<&'a MuxBox>) -> bool {
            for muxbox in muxboxes {
                if muxbox.id == id {
                    return true;
                }
                path.push(muxbox);
                if search(muxbox.children.as_deref().unwrap_or_default(), id, path) {
                    return true;
                }
                path.pop();
            }
            false
        }
        let mut path = Vec::new();
        search(self.children.as_deref()?, id, &mut path).then(|| {
            path.reverse();
            path
        })
    }

    pub fn get_selected_muxboxes(&self) -> Vec<&MuxBox> {
        fn recursive_collect<'a>(muxboxes: &'a [MuxBox], selected_muxboxes: &mut Vec<&'a MuxBox>) {
            for muxbox in muxboxes {
                if muxbox.selected.unwrap_or(false) {
                    selected_muxboxes.push(muxbox);
                }
                if let Some(ref children) = muxbox.children {
                    recursive_collect(children, selected_muxboxes);
                }
            }
        }

        let mut selected_muxboxes = Vec::new();

        if let Some(ref children) = self.children {
            recursive_collect(children, &mut selected_muxboxes);
        }
        selected_muxboxes
    }

    pub fn select_only_muxbox(&mut self, id: &str) {
        fn recursive_select(muxboxes: &mut [MuxBox], id: &str) {
            for muxbox in muxboxes {
                muxbox.selected = Some(muxbox.id == id);
                if let Some(ref mut children) = muxbox.children {
                    recursive_select(children, id);
                }
            }
        }

        if let Some(ref mut children) = self.children {
            recursive_select(children, id);
        }
    }

    /// Calls `visit` on every box of the layout, nested ones included, parents first.
    pub fn for_each_muxbox_mut(&mut self, visit: &mut dyn FnMut(&mut MuxBox)) {
        fn walk(muxboxes: &mut [MuxBox], visit: &mut dyn FnMut(&mut MuxBox)) {
            for muxbox in muxboxes {
                visit(muxbox);
                if let Some(children) = muxbox.children.as_deref_mut() {
                    walk(children, visit);
                }
            }
        }
        if let Some(children) = self.children.as_deref_mut() {
            walk(children, visit);
        }
    }

    /// The focusable boxes of the layout, in focus order: ascending `tab_order`, and boxes with
    /// the same `tab_order` in the order they appear in the configuration.
    pub fn get_muxboxes_in_tab_order(&self) -> Vec<&MuxBox> {
        let mut focusable: Vec<&MuxBox> = self
            .get_all_muxboxes()
            .into_iter()
            .filter(|muxbox| muxbox.tab_order.is_some())
            .collect();
        // sort_by_key is stable, which keeps configuration order among equal tab orders.
        focusable.sort_by_key(|muxbox| muxbox.tab_order);
        focusable
    }

    pub fn get_all_muxboxes(&self) -> Vec<&MuxBox> {
        fn recursive_collect<'a>(muxboxes: &'a [MuxBox], all_muxboxes: &mut Vec<&'a MuxBox>) {
            for muxbox in muxboxes {
                all_muxboxes.push(muxbox);
                if let Some(ref children) = muxbox.children {
                    recursive_collect(children, all_muxboxes);
                }
            }
        }

        let mut all_muxboxes = Vec::new();
        if let Some(ref children) = self.children {
            recursive_collect(children, &mut all_muxboxes);
        }
        all_muxboxes
    }

    pub fn select_next_muxbox(&mut self) {
        self.step_focus(true);
    }

    pub fn select_previous_muxbox(&mut self) {
        self.step_focus(false);
    }

    /// Moves the focus one step: to the focused box's `next_focus_id` when Tab is pressed on a
    /// box that has one, otherwise to the next (previous) box in tab order, wrapping around.
    fn step_focus(&mut self, forward: bool) {
        let order: Vec<String> = self
            .get_muxboxes_in_tab_order()
            .iter()
            .map(|muxbox| muxbox.id.clone())
            .collect();
        if order.is_empty() {
            return;
        }
        let focused = self
            .get_selected_muxboxes()
            .first()
            .map(|muxbox| (muxbox.id.clone(), muxbox.next_focus_id.clone()));
        if forward {
            // A box can name the box Tab moves to, overriding the tab order.
            if let Some((_, Some(target))) = &focused {
                if self.get_muxbox_by_id(target).is_some() {
                    self.select_only_muxbox(target);
                    return;
                }
            }
        }
        let current = focused
            .as_ref()
            .and_then(|(id, _)| order.iter().position(|candidate| candidate == id));
        let next = match (current, forward) {
            (Some(i), true) => (i + 1) % order.len(),
            (Some(i), false) => (i + order.len() - 1) % order.len(),
            (None, true) => 0,
            (None, false) => order.len() - 1,
        };
        self.select_only_muxbox(&order[next]);
    }

    pub fn deselect_all_muxboxes(&mut self) {
        fn deselect(muxboxes: &mut [MuxBox]) {
            for muxbox in muxboxes {
                muxbox.selected = Some(false);
                if let Some(children) = muxbox.children.as_mut() {
                    deselect(children);
                }
            }
        }
        if let Some(children) = self.children.as_mut() {
            deselect(children);
        }
    }

    /// Replaces the box with the same id as `replacement_muxbox`, wherever it is nested.
    /// Returns whether a box was replaced.
    pub fn replace_muxbox_recursive(&mut self, replacement_muxbox: &MuxBox) -> bool {
        fn replace_in_muxboxes(muxboxes: &mut [MuxBox], replacement: &MuxBox) -> bool {
            for muxbox in muxboxes {
                if muxbox.id == replacement.id {
                    *muxbox = replacement.clone();
                    return true;
                }
                if let Some(ref mut children) = muxbox.children {
                    if replace_in_muxboxes(children, replacement) {
                        return true;
                    }
                }
            }
            false
        }

        self.children
            .as_deref_mut()
            .is_some_and(|children| replace_in_muxboxes(children, replacement_muxbox))
    }

    pub fn find_muxbox_with_choice(&self, choice_id: &str) -> Option<&MuxBox> {
        fn find_in_muxboxes<'a>(muxboxes: &'a [MuxBox], choice_id: &str) -> Option<&'a MuxBox> {
            for muxbox in muxboxes {
                // Check all streams, not just active stream, since choice execution
                // should be possible regardless of which stream/tab is currently active
                for stream in muxbox.streams.values() {
                    if let Some(choices) = &stream.choices {
                        if choices.iter().any(|c| c.id == choice_id) {
                            return Some(muxbox);
                        }
                    }
                }
                if let Some(ref children) = muxbox.children {
                    if let Some(found) = find_in_muxboxes(children, choice_id) {
                        return Some(found);
                    }
                }
            }
            None
        }

        if let Some(ref children) = self.children {
            find_in_muxboxes(children, choice_id)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn muxbox(id: &str, tab_order: Option<u32>, children: Option<Vec<MuxBox>>) -> MuxBox {
        MuxBox {
            id: id.to_string(),
            tab_order,
            children,
            ..MuxBox::default()
        }
    }

    fn layout(children: Vec<MuxBox>) -> Layout {
        Layout {
            id: "layout".to_string(),
            children: Some(children),
            ..Layout::default()
        }
    }

    fn selected_ids(layout: &Layout) -> Vec<String> {
        layout
            .get_selected_muxboxes()
            .iter()
            .map(|muxbox| muxbox.id.clone())
            .collect()
    }

    fn focus_order(layout: &Layout) -> Vec<String> {
        layout
            .get_muxboxes_in_tab_order()
            .iter()
            .map(|muxbox| muxbox.id.clone())
            .collect()
    }

    #[test]
    fn tab_order_is_numeric_and_includes_nested_boxes() {
        let layout = layout(vec![
            muxbox("ten", Some(10), None),
            muxbox(
                "parent",
                None,
                Some(vec![
                    muxbox("two", Some(2), None),
                    muxbox("nine", Some(9), None),
                ]),
            ),
            muxbox("unfocusable", None, None),
        ]);
        // As strings, "10" would sort before "2" and "9".
        assert_eq!(focus_order(&layout), vec!["two", "nine", "ten"]);
    }

    #[test]
    fn equal_tab_orders_keep_configuration_order() {
        let layout = layout(vec![
            muxbox("b", Some(1), None),
            muxbox("a", Some(1), None),
            muxbox("c", Some(0), None),
        ]);
        assert_eq!(focus_order(&layout), vec!["c", "b", "a"]);
    }

    #[test]
    fn focus_steps_wrap_around_in_both_directions() {
        let mut layout = layout(vec![
            muxbox("a", Some(1), None),
            muxbox("p", None, Some(vec![muxbox("b", Some(2), None)])),
            muxbox("c", Some(3), None),
        ]);
        layout.select_next_muxbox();
        assert_eq!(
            selected_ids(&layout),
            vec!["a"],
            "nothing focused: Tab focuses the first"
        );
        layout.select_next_muxbox();
        assert_eq!(selected_ids(&layout), vec!["b"], "nested boxes take part");
        layout.select_next_muxbox();
        layout.select_next_muxbox();
        assert_eq!(selected_ids(&layout), vec!["a"], "wraps to the first");
        layout.select_previous_muxbox();
        assert_eq!(selected_ids(&layout), vec!["c"], "wraps to the last");
    }

    #[test]
    fn next_focus_id_overrides_the_tab_order_going_forward() {
        let mut first = muxbox("first", Some(1), None);
        first.next_focus_id = Some("third".into());
        let mut layout = layout(vec![
            first,
            muxbox("second", Some(2), None),
            // Not in the tab order at all, but reachable through next_focus_id.
            muxbox("third", None, None),
        ]);
        layout.select_only_muxbox("first");
        layout.select_next_muxbox();
        assert_eq!(selected_ids(&layout), vec!["third"]);
        // From a box outside the tab order, Tab starts over at the first box.
        layout.select_next_muxbox();
        assert_eq!(selected_ids(&layout), vec!["first"]);
        // Shift+Tab ignores next_focus_id and follows the tab order backwards.
        layout.select_previous_muxbox();
        assert_eq!(selected_ids(&layout), vec!["second"]);
    }

    #[test]
    fn ancestors_are_listed_nearest_first() {
        let layout = layout(vec![muxbox(
            "outer",
            None,
            Some(vec![muxbox(
                "inner",
                None,
                Some(vec![muxbox("leaf", None, None)]),
            )]),
        )]);
        let ids = |id: &str| {
            layout
                .ancestors_of(id)
                .map(|a| a.iter().map(|b| b.id.clone()).collect::<Vec<_>>())
        };
        assert_eq!(
            ids("leaf"),
            Some(vec!["inner".to_string(), "outer".to_string()])
        );
        assert_eq!(ids("outer"), Some(vec![]));
        assert_eq!(ids("missing"), None);
    }

    #[test]
    fn focus_steps_without_focusable_boxes_change_nothing() {
        let mut layout = layout(vec![muxbox("a", None, None)]);
        layout.select_next_muxbox();
        layout.select_previous_muxbox();
        assert!(selected_ids(&layout).is_empty());
    }

    #[test]
    fn select_only_and_deselect_reach_nested_boxes() {
        let mut layout = layout(vec![
            muxbox("a", None, None),
            muxbox("p", None, Some(vec![muxbox("nested", None, None)])),
        ]);
        layout.select_only_muxbox("nested");
        assert_eq!(selected_ids(&layout), vec!["nested"]);
        layout.select_only_muxbox("a");
        assert_eq!(selected_ids(&layout), vec!["a"]);
        layout.get_muxbox_by_id_mut("nested").unwrap().selected = Some(true);
        layout.deselect_all_muxboxes();
        assert!(selected_ids(&layout).is_empty());
    }

    #[test]
    fn replacing_finds_nested_boxes_and_reports_misses() {
        let mut layout = layout(vec![muxbox(
            "p",
            None,
            Some(vec![muxbox("nested", None, None)]),
        )]);
        let mut replacement = muxbox("nested", Some(4), None);
        replacement.title = Some("new".into());
        assert!(layout.replace_muxbox_recursive(&replacement));
        assert_eq!(
            layout.get_muxbox_by_id("nested").unwrap().title.as_deref(),
            Some("new")
        );
        assert!(!layout.replace_muxbox_recursive(&muxbox("missing", None, None)));
        assert!(!Layout::default().replace_muxbox_recursive(&replacement));
    }

    #[test]
    fn choices_are_found_in_any_tab_of_nested_boxes() {
        let mut nested = muxbox("nested", None, None);
        nested.content = Some("text".into());
        nested.choices = Some(vec![crate::model::choice::Choice {
            id: "deploy".into(),
            content: Some("Deploy".into()),
            script: None,
            redirect_output: None,
            append_output: None,
            execution_mode: Default::default(),
            selected: false,
            waiting: false,
        }]);
        nested.initialize_streams();
        let layout = layout(vec![muxbox("p", None, Some(vec![nested]))]);
        assert_eq!(
            layout
                .find_muxbox_with_choice("deploy")
                .map(|b| b.id.as_str()),
            Some("nested")
        );
        assert!(layout.find_muxbox_with_choice("other").is_none());
    }

    #[test]
    fn unknown_layout_keys_and_the_runtime_active_flag_are_rejected() {
        for key in ["active: true", "border: true"] {
            let yaml = format!("id: l\n{}\nchildren: []\n", key);
            assert!(
                serde_yaml::from_str::<Layout>(&yaml).is_err(),
                "`{}` must not be accepted",
                key
            );
        }
    }
}
