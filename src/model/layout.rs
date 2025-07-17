use crate::{model::panel::Panel, EntityType, FieldUpdate, Updatable};
use core::hash::Hash;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, hash::Hasher};

#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct Layout {
    pub id: String,
    pub title: Option<String>,
    pub refresh_interval: Option<u64>,
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
    pub title_position: Option<String>,
    pub selected_title_bg_color: Option<String>,
    pub selected_title_fg_color: Option<String>,
    pub menu_fg_color: Option<String>,
    pub menu_bg_color: Option<String>,
    pub selected_menu_fg_color: Option<String>,
    pub selected_menu_bg_color: Option<String>,
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
    pub overflow_behavior: Option<String>,
    pub root: Option<bool>,
    #[serde(default)]
    pub on_keypress: Option<HashMap<String, Vec<String>>>,
    #[serde(skip)]
    pub active: Option<bool>,
    #[serde(skip)]
    pub panel_ids_in_tab_order: Option<Vec<String>>,
}

impl Hash for Layout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.title.hash(state);
        self.refresh_interval.hash(state);
        for panel in &self.children {
            panel.hash(state);
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
        self.title_position.hash(state);
        self.selected_title_bg_color.hash(state);
        self.selected_title_fg_color.hash(state);
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
        self.overflow_behavior.hash(state);
        self.root.hash(state);
        self.active.hash(state);
    }
}

impl Layout {
    pub fn new() -> Self {
        Layout {
            id: String::new(),
            title: None,
            refresh_interval: None,
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
            title_position: None,
            selected_title_bg_color: None,
            selected_title_fg_color: None,
            menu_fg_color: None,
            menu_bg_color: None,
            selected_menu_fg_color: None,
            selected_menu_bg_color: None,
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
            overflow_behavior: None,
            root: Some(false),
            on_keypress: None,
            active: Some(false),
            panel_ids_in_tab_order: None,
        }
    }

    pub fn get_panel_by_id(&self, id: &str) -> Option<&Panel> {
        fn recursive_search<'a>(panels: &'a [Panel], id: &str) -> Option<&'a Panel> {
            for panel in panels {
                if panel.id == id {
                    return Some(panel);
                }
                if let Some(ref children) = panel.children {
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

    pub fn get_panel_by_id_mut(&mut self, id: &str) -> Option<&mut Panel> {
        fn recursive_search<'a>(panels: &'a mut [Panel], id: &str) -> Option<&'a mut Panel> {
            for panel in panels {
                if panel.id == id {
                    return Some(panel);
                }
                if let Some(ref mut children) = panel.children {
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

    pub fn get_selected_panels(&self) -> Vec<&Panel> {
        fn recursive_collect<'a>(panels: &'a [Panel], selected_panels: &mut Vec<&'a Panel>) {
            for panel in panels {
                if panel.selected.unwrap_or(false) {
                    selected_panels.push(panel);
                }
                if let Some(ref children) = panel.children {
                    recursive_collect(children, selected_panels);
                }
            }
        }

        let mut selected_panels = Vec::new();

        if let Some(ref children) = self.children {
            recursive_collect(children, &mut selected_panels);
        }
        selected_panels
    }

    pub fn select_only_panel(&mut self, id: &str) {
        fn recursive_select(panels: &mut [Panel], id: &str) {
            for panel in panels {
                panel.selected = Some(panel.id == id);
                if let Some(ref mut children) = panel.children {
                    recursive_select(children, id);
                }
            }
        }

        if let Some(ref mut children) = self.children {
            recursive_select(children, id);
        }
    }

    pub fn get_panels_in_tab_order(&mut self) -> Vec<&Panel> {
        fn collect_panels_recursive<'a>(panel: &'a Panel, panels: &mut Vec<&'a Panel>) {
            // Check if panel has a tab order and add it to the list
            if panel.tab_order.is_some() {
                panels.push(panel);
            }

            // If children exist, iterate over them recursively
            if let Some(children) = &panel.children {
                for child in children {
                    collect_panels_recursive(child, panels);
                }
            }
        }

        if self.panel_ids_in_tab_order.is_some() {
            let mut panels = Vec::new();
            for panel_id in self.panel_ids_in_tab_order.as_ref().unwrap() {
                if let Some(panel) = self.get_panel_by_id(panel_id) {
                    panels.push(panel);
                }
            }
            panels
        } else {
            let mut panels = Vec::new();
            // Start recursion for each top-level child
            if let Some(children) = &self.children {
                for panel in children {
                    collect_panels_recursive(panel, &mut panels);
                }
            }

            // Sort panels by their tab order
            panels.sort_by(|a, b| {
                a.tab_order
                    .as_ref()
                    .unwrap()
                    .cmp(b.tab_order.as_ref().unwrap())
            });

            self.panel_ids_in_tab_order = Some(panels.iter().map(|p| p.id.clone()).collect());

            panels
        }
    }

    pub fn get_all_panels(&self) -> Vec<&Panel> {
        fn recursive_collect<'a>(panels: &'a [Panel], all_panels: &mut Vec<&'a Panel>) {
            for panel in panels {
                all_panels.push(panel);
                if let Some(ref children) = panel.children {
                    recursive_collect(children, all_panels);
                }
            }
        }

        let mut all_panels = Vec::new();
        if let Some(ref children) = self.children {
            recursive_collect(children, &mut all_panels);
        }
        all_panels
    }

    pub fn select_next_panel(&mut self) {
        let panels = self.get_panels_in_tab_order();
        if panels.is_empty() {
            return; // Early return if there are no panels
        }

        let selected_panel_index = panels.iter().position(|p| p.selected.unwrap_or(false));

        let next_panel_index = match selected_panel_index {
            Some(index) => (index + 1) % panels.len(), // Get next panel, wrap around if at the end
            None => 0,                                 // No panel is selected, select the first one
        };

        let next_panel_id = panels[next_panel_index].id.clone();
        self.select_only_panel(&next_panel_id);
    }

    pub fn select_previous_panel(&mut self) {
        let panels = self.get_panels_in_tab_order();
        if panels.is_empty() {
            return; // Early return if there are no panels
        }

        let selected_panel_index = panels.iter().position(|p| p.selected.unwrap_or(false));

        let previous_panel_index = match selected_panel_index {
            Some(index) => {
                if index == 0 {
                    panels.len() - 1 // Wrap around to the last panel if the first one is currently selected
                } else {
                    index - 1 // Select the previous panel
                }
            }
            None => panels.len() - 1, // No panel is selected, select the last one
        };

        let previous_panel_id = panels[previous_panel_index].id.clone();
        self.select_only_panel(&previous_panel_id);
    }

    pub fn deselect_all_panels(&mut self) {
        if let Some(children) = &mut self.children {
            for panel in children {
                panel.selected = Some(false);
            }
        }
    }

    fn generate_children_diff(&self, other: &Self) -> Vec<FieldUpdate> {
        let mut updates = Vec::new();

        // Get references to children, defaulting to an empty slice if None
        let self_children = self.children.as_deref().unwrap_or(&[]);
        let other_children = other.children.as_deref().unwrap_or(&[]);

        // Compare each pair of children
        for (self_child, other_child) in self_children.iter().zip(other_children) {
            let child_diffs = self_child.generate_diff(other_child);
            updates.extend(child_diffs.into_iter());
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
}

impl Clone for Layout {
    fn clone(&self) -> Self {
        let mut cloned_children = None;
        if let Some(ref children) = self.children {
            cloned_children = Some(children.to_vec());
        }

        Layout {
            id: self.id.clone(),
            title: self.title.clone(),
            refresh_interval: self.refresh_interval,
            children: cloned_children,
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
            title_position: self.title_position.clone(),
            selected_title_bg_color: self.selected_title_bg_color.clone(),
            selected_title_fg_color: self.selected_title_fg_color.clone(),
            menu_fg_color: self.menu_fg_color.clone(),
            menu_bg_color: self.menu_bg_color.clone(),
            selected_menu_fg_color: self.selected_menu_fg_color.clone(),
            selected_menu_bg_color: self.selected_menu_bg_color.clone(),
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
            overflow_behavior: self.overflow_behavior.clone(),
            root: self.root,
            on_keypress: self.on_keypress.clone(),
            active: self.active,
            panel_ids_in_tab_order: self.panel_ids_in_tab_order.clone(),
        }
    }
}

// Implement Updatable for Layout
impl Updatable for Layout {
    fn generate_diff(&self, other: &Self) -> Vec<FieldUpdate> {
        let mut updates = Vec::new();

        // Compare each field
        if self.title != other.title {
            if let Some(new_value) = &other.title {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "title".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }
        if self.refresh_interval != other.refresh_interval {
            if let Some(new_value) = other.refresh_interval {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "refresh_interval".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        updates.extend(self.generate_children_diff(other));

        // Compare other fields similarly...
        if self.fill != other.fill {
            if let Some(new_value) = other.fill {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "fill".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.fill_char != other.fill_char {
            if let Some(new_value) = other.fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "fill_char".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_fill_char != other.selected_fill_char {
            if let Some(new_value) = other.selected_fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "selected_fill_char".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.border != other.border {
            if let Some(new_value) = other.border {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "border".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.border_color != other.border_color {
            if let Some(new_value) = &other.border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_border_color != other.selected_border_color {
            if let Some(new_value) = &other.selected_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "selected_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.bg_color != other.bg_color {
            if let Some(new_value) = &other.bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_bg_color != other.selected_bg_color {
            if let Some(new_value) = &other.selected_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "selected_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.fg_color != other.fg_color {
            if let Some(new_value) = &other.fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_fg_color != other.selected_fg_color {
            if let Some(new_value) = &other.selected_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "selected_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_fg_color != other.title_fg_color {
            if let Some(new_value) = &other.title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_bg_color != other.title_bg_color {
            if let Some(new_value) = &other.title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_position != other.title_position {
            if let Some(new_value) = &other.title_position {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "title_position".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_title_bg_color != other.selected_title_bg_color {
            if let Some(new_value) = &other.selected_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "selected_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_title_fg_color != other.selected_title_fg_color {
            if let Some(new_value) = &other.selected_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "selected_title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.menu_fg_color != other.menu_fg_color {
            if let Some(new_value) = &other.menu_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "menu_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.menu_bg_color != other.menu_bg_color {
            if let Some(new_value) = &other.menu_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "menu_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_menu_fg_color != other.selected_menu_fg_color {
            if let Some(new_value) = &other.selected_menu_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "selected_menu_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_menu_bg_color != other.selected_menu_bg_color {
            if let Some(new_value) = &other.selected_menu_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "selected_menu_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_border_color != other.error_border_color {
            if let Some(new_value) = &other.error_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_bg_color != other.error_bg_color {
            if let Some(new_value) = &other.error_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_fg_color != other.error_fg_color {
            if let Some(new_value) = &other.error_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_title_bg_color != other.error_title_bg_color {
            if let Some(new_value) = &other.error_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_title_fg_color != other.error_title_fg_color {
            if let Some(new_value) = &other.error_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_border_color != other.error_selected_border_color {
            if let Some(new_value) = &other.error_selected_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_selected_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_bg_color != other.error_selected_bg_color {
            if let Some(new_value) = &other.error_selected_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_selected_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_fg_color != other.error_selected_fg_color {
            if let Some(new_value) = &other.error_selected_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_selected_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_title_bg_color != other.error_selected_title_bg_color {
            if let Some(new_value) = &other.error_selected_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_selected_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.error_selected_title_fg_color != other.error_selected_title_fg_color {
            if let Some(new_value) = &other.error_selected_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()),
                    field_name: "error_selected_title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.overflow_behavior != other.overflow_behavior {
            if let Some(new_value) = &other.overflow_behavior {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "overflow_behavior".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.root != other.root {
            if let Some(new_value) = other.root {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "root".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.on_keypress != other.on_keypress {
            if let Some(new_value) = &other.on_keypress {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "on_keypress".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.active != other.active {
            if let Some(new_value) = other.active {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "active".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.panel_ids_in_tab_order != other.panel_ids_in_tab_order {
            if let Some(new_value) = &other.panel_ids_in_tab_order {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "panel_ids_in_tab_order".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        updates
    }

    fn apply_updates(&mut self, updates: Vec<FieldUpdate>) {
        let updates_for_children = updates.clone();

        for update in updates {
            if update.entity_type != EntityType::Layout {
                continue;
            }
            match update.field_name.as_str() {
                "title" => {
                    if let Some(new_title) = update.new_value.as_str() {
                        self.title = Some(new_title.to_string());
                    }
                }
                "refresh_interval" => {
                    if let Some(new_refresh_interval) = update.new_value.as_u64() {
                        self.refresh_interval = Some(new_refresh_interval);
                    }
                }
                "fill" => {
                    if let Some(new_fill) = update.new_value.as_bool() {
                        self.fill = Some(new_fill);
                    }
                }
                "fill_char" => {
                    if let Some(new_fill_char) = update.new_value.as_str() {
                        self.fill_char = new_fill_char.chars().next();
                    }
                }
                "selected_fill_char" => {
                    if let Some(new_selected_fill_char) = update.new_value.as_str() {
                        self.selected_fill_char = new_selected_fill_char.chars().next();
                    }
                }
                "border" => {
                    if let Some(new_border) = update.new_value.as_bool() {
                        self.border = Some(new_border);
                    }
                }
                "border_color" => {
                    if let Some(new_border_color) = update.new_value.as_str() {
                        self.border_color = Some(new_border_color.to_string());
                    }
                }
                "selected_border_color" => {
                    if let Some(new_selected_border_color) = update.new_value.as_str() {
                        self.selected_border_color = Some(new_selected_border_color.to_string());
                    }
                }
                "bg_color" => {
                    if let Some(new_bg_color) = update.new_value.as_str() {
                        self.bg_color = Some(new_bg_color.to_string());
                    }
                }
                "selected_bg_color" => {
                    if let Some(new_selected_bg_color) = update.new_value.as_str() {
                        self.selected_bg_color = Some(new_selected_bg_color.to_string());
                    }
                }
                "fg_color" => {
                    if let Some(new_fg_color) = update.new_value.as_str() {
                        self.fg_color = Some(new_fg_color.to_string());
                    }
                }
                "selected_fg_color" => {
                    if let Some(new_selected_fg_color) = update.new_value.as_str() {
                        self.selected_fg_color = Some(new_selected_fg_color.to_string());
                    }
                }
                "title_fg_color" => {
                    if let Some(new_title_fg_color) = update.new_value.as_str() {
                        self.title_fg_color = Some(new_title_fg_color.to_string());
                    }
                }
                "title_bg_color" => {
                    if let Some(new_title_bg_color) = update.new_value.as_str() {
                        self.title_bg_color = Some(new_title_bg_color.to_string());
                    }
                }
                "title_position" => {
                    if let Some(new_title_position) = update.new_value.as_str() {
                        self.title_position = Some(new_title_position.to_string());
                    }
                }
                "selected_title_bg_color" => {
                    if let Some(new_selected_title_bg_color) = update.new_value.as_str() {
                        self.selected_title_bg_color =
                            Some(new_selected_title_bg_color.to_string());
                    }
                }
                "selected_title_fg_color" => {
                    if let Some(new_selected_title_fg_color) = update.new_value.as_str() {
                        self.selected_title_fg_color =
                            Some(new_selected_title_fg_color.to_string());
                    }
                }
                "menu_fg_color" => {
                    if let Some(new_menu_fg_color) = update.new_value.as_str() {
                        self.menu_fg_color = Some(new_menu_fg_color.to_string());
                    }
                }
                "menu_bg_color" => {
                    if let Some(new_menu_bg_color) = update.new_value.as_str() {
                        self.menu_bg_color = Some(new_menu_bg_color.to_string());
                    }
                }
                "selected_menu_fg_color" => {
                    if let Some(new_selected_menu_fg_color) = update.new_value.as_str() {
                        self.selected_menu_fg_color = Some(new_selected_menu_fg_color.to_string());
                    }
                }
                "selected_menu_bg_color" => {
                    if let Some(new_selected_menu_bg_color) = update.new_value.as_str() {
                        self.selected_menu_bg_color = Some(new_selected_menu_bg_color.to_string());
                    }
                }
                "error_border_color" => {
                    if let Some(new_error_border_color) = update.new_value.as_str() {
                        self.error_border_color = Some(new_error_border_color.to_string());
                    }
                }
                "error_bg_color" => {
                    if let Some(new_error_bg_color) = update.new_value.as_str() {
                        self.error_bg_color = Some(new_error_bg_color.to_string());
                    }
                }
                "error_fg_color" => {
                    if let Some(new_error_fg_color) = update.new_value.as_str() {
                        self.error_fg_color = Some(new_error_fg_color.to_string());
                    }
                }
                "error_title_bg_color" => {
                    if let Some(new_error_title_bg_color) = update.new_value.as_str() {
                        self.error_title_bg_color = Some(new_error_title_bg_color.to_string());
                    }
                }
                "error_title_fg_color" => {
                    if let Some(new_error_title_fg_color) = update.new_value.as_str() {
                        self.error_title_fg_color = Some(new_error_title_fg_color.to_string());
                    }
                }
                "error_selected_border_color" => {
                    if let Some(new_error_selected_border_color) = update.new_value.as_str() {
                        self.error_selected_border_color =
                            Some(new_error_selected_border_color.to_string());
                    }
                }
                "error_selected_bg_color" => {
                    if let Some(new_error_selected_bg_color) = update.new_value.as_str() {
                        self.error_selected_bg_color =
                            Some(new_error_selected_bg_color.to_string());
                    }
                }
                "error_selected_fg_color" => {
                    if let Some(new_error_selected_fg_color) = update.new_value.as_str() {
                        self.error_selected_fg_color =
                            Some(new_error_selected_fg_color.to_string());
                    }
                }
                "error_selected_title_bg_color" => {
                    if let Some(new_error_selected_title_bg_color) = update.new_value.as_str() {
                        self.error_selected_title_bg_color =
                            Some(new_error_selected_title_bg_color.to_string());
                    }
                }
                "error_selected_title_fg_color" => {
                    if let Some(new_error_selected_title_fg_color) = update.new_value.as_str() {
                        self.error_selected_title_fg_color =
                            Some(new_error_selected_title_fg_color.to_string());
                    }
                }
                "overflow_behavior" => {
                    if let Some(new_overflow_behavior) = update.new_value.as_str() {
                        self.overflow_behavior = Some(new_overflow_behavior.to_string());
                    }
                }
                "root" => {
                    if let Some(new_root) = update.new_value.as_bool() {
                        self.root = Some(new_root);
                    }
                }
                "on_keypress" => {
                    if let Some(new_on_keypress) = update.new_value.as_object() {
                        self.on_keypress = Some(
                            new_on_keypress
                                .iter()
                                .map(|(k, v)| {
                                    (
                                        k.clone(),
                                        v.as_array()
                                            .unwrap()
                                            .iter()
                                            .map(|v| v.as_str().unwrap().to_string())
                                            .collect(),
                                    )
                                })
                                .collect(),
                        );
                    }
                }
                "active" => {
                    if let Some(new_active) = update.new_value.as_bool() {
                        self.active = Some(new_active);
                    }
                }
                "panel_ids_in_tab_order" => {
                    if let Some(new_panel_ids_in_tab_order) = update.new_value.as_array() {
                        self.panel_ids_in_tab_order = Some(
                            new_panel_ids_in_tab_order
                                .iter()
                                .map(|v| v.as_str().unwrap().to_string())
                                .collect(),
                        );
                    }
                }

                _ => {
                    log::warn!(
                        "Layout::apply_updates: Ignoring unknown field name: {}",
                        update.field_name
                    );
                }
            }
        }
        self.apply_children_updates(updates_for_children);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::common::InputBounds;
    use crate::model::panel::Panel;

    // === Helper Functions ===

    /// Creates a basic test panel with the given id.
    /// This helper demonstrates how to create a Panel for Layout testing.
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
            tab_order: Some(id.to_string()),
            selected: Some(false),
            ..Default::default()
        }
    }

    /// Creates a test Layout with the given id and optional children.
    /// This helper demonstrates how to create a Layout for testing.
    fn create_test_layout(id: &str, children: Option<Vec<Panel>>) -> Layout {
        Layout {
            id: id.to_string(),
            title: Some(format!("Test Layout {}", id)),
            children,
            root: Some(false),
            active: Some(false),
            ..Default::default()
        }
    }

    // === Layout Default Tests ===

    /// Tests that Layout::default() creates a layout with expected default values.
    /// This test demonstrates the default Layout construction behavior.
    #[test]
    fn test_layout_default() {
        let layout = Layout::default();
        assert_eq!(layout.id, "");
        assert_eq!(layout.title, None);
        assert_eq!(layout.children, None);
        assert_eq!(layout.root, Some(false));
        assert_eq!(layout.active, Some(false));
        assert_eq!(layout.refresh_interval, None);
        assert_eq!(layout.panel_ids_in_tab_order, None);
    }

    /// Tests that Layout::new() creates a layout with expected default values.
    /// This test demonstrates the Layout::new() construction behavior.
    #[test]
    fn test_layout_new() {
        let layout = Layout::new();
        assert_eq!(layout.id, "");
        assert_eq!(layout.title, None);
        assert_eq!(layout.children, None);
        assert_eq!(layout.root, Some(false));
        assert_eq!(layout.active, Some(false));
        assert_eq!(layout.refresh_interval, None);
        assert_eq!(layout.panel_ids_in_tab_order, None);
    }

    // === Layout Creation Tests ===

    /// Tests creating a Layout with specific values.
    /// This test demonstrates how to create a Layout with custom properties.
    #[test]
    fn test_layout_creation() {
        let panel1 = create_test_panel("panel1");
        let panel2 = create_test_panel("panel2");
        let children = vec![panel1, panel2];
        
        let layout = Layout {
            id: "test_layout".to_string(),
            title: Some("Test Layout".to_string()),
            children: Some(children),
            root: Some(true),
            active: Some(true),
            refresh_interval: Some(1000),
            ..Default::default()
        };
        
        assert_eq!(layout.id, "test_layout");
        assert_eq!(layout.title, Some("Test Layout".to_string()));
        assert_eq!(layout.children.as_ref().unwrap().len(), 2);
        assert_eq!(layout.root, Some(true));
        assert_eq!(layout.active, Some(true));
        assert_eq!(layout.refresh_interval, Some(1000));
    }

    // === Layout Panel Management Tests ===

    /// Tests that Layout::get_panel_by_id() finds panels correctly.
    /// This test demonstrates the panel retrieval feature.
    #[test]
    fn test_layout_get_panel_by_id() {
        let panel1 = create_test_panel("panel1");
        let panel2 = create_test_panel("panel2");
        let layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        let found_panel = layout.get_panel_by_id("panel1");
        assert!(found_panel.is_some());
        assert_eq!(found_panel.unwrap().id, "panel1");
        
        let not_found = layout.get_panel_by_id("nonexistent");
        assert!(not_found.is_none());
    }

    /// Tests that Layout::get_panel_by_id() finds nested panels correctly.
    /// This test demonstrates the recursive panel retrieval feature.
    #[test]
    fn test_layout_get_panel_by_id_nested() {
        let child_panel = create_test_panel("child");
        let parent_panel = Panel {
            id: "parent".to_string(),
            children: Some(vec![child_panel]),
            ..Default::default()
        };
        let layout = create_test_layout("test", Some(vec![parent_panel]));
        
        let found_child = layout.get_panel_by_id("child");
        assert!(found_child.is_some());
        assert_eq!(found_child.unwrap().id, "child");
    }

    /// Tests that Layout::get_panel_by_id_mut() finds and allows modification.
    /// This test demonstrates the mutable panel retrieval feature.
    #[test]
    fn test_layout_get_panel_by_id_mut() {
        let panel1 = create_test_panel("panel1");
        let panel2 = create_test_panel("panel2");
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        let found_panel = layout.get_panel_by_id_mut("panel1");
        assert!(found_panel.is_some());
        
        // Modify the panel
        found_panel.unwrap().title = Some("Modified Title".to_string());
        
        // Verify the modification
        let verified_panel = layout.get_panel_by_id("panel1");
        assert_eq!(verified_panel.unwrap().title, Some("Modified Title".to_string()));
    }

    /// Tests that Layout::get_panel_by_id_mut() handles empty layout.
    /// This test demonstrates edge case handling in mutable panel retrieval.
    #[test]
    fn test_layout_get_panel_by_id_mut_empty() {
        let mut layout = create_test_layout("test", None);
        
        let found_panel = layout.get_panel_by_id_mut("nonexistent");
        assert!(found_panel.is_none());
    }

    // === Layout Panel Selection Tests ===

    /// Tests that Layout::get_selected_panels() returns selected panels.
    /// This test demonstrates the selected panel retrieval feature.
    #[test]
    fn test_layout_get_selected_panels() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        let mut panel3 = create_test_panel("panel3");
        
        panel1.selected = Some(true);
        panel2.selected = Some(false);
        panel3.selected = Some(true);
        
        let layout = create_test_layout("test", Some(vec![panel1, panel2, panel3]));
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, "panel1");
        assert_eq!(selected[1].id, "panel3");
    }

    /// Tests that Layout::get_selected_panels() handles no selected panels.
    /// This test demonstrates edge case handling in selected panel retrieval.
    #[test]
    fn test_layout_get_selected_panels_none() {
        let panel1 = create_test_panel("panel1");
        let panel2 = create_test_panel("panel2");
        let layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 0);
    }

    /// Tests that Layout::select_only_panel() selects only the specified panel.
    /// This test demonstrates the exclusive panel selection feature.
    #[test]
    fn test_layout_select_only_panel() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        let mut panel3 = create_test_panel("panel3");
        
        panel1.selected = Some(true);
        panel2.selected = Some(true);
        panel3.selected = Some(false);
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2, panel3]));
        
        layout.select_only_panel("panel2");
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "panel2");
    }

    /// Tests that Layout::select_only_panel() handles nonexistent panel.
    /// This test demonstrates edge case handling in panel selection.
    #[test]
    fn test_layout_select_only_panel_nonexistent() {
        let mut panel1 = create_test_panel("panel1");
        panel1.selected = Some(true);
        
        let mut layout = create_test_layout("test", Some(vec![panel1]));
        
        layout.select_only_panel("nonexistent");
        
        // All panels should be deselected
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 0);
    }

    /// Tests that Layout::deselect_all_panels() deselects all panels.
    /// This test demonstrates the panel deselection feature.
    #[test]
    fn test_layout_deselect_all_panels() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        
        panel1.selected = Some(true);
        panel2.selected = Some(true);
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        layout.deselect_all_panels();
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 0);
    }

    // === Layout Tab Order Tests ===

    /// Tests that Layout::get_panels_in_tab_order() returns panels in tab order.
    /// This test demonstrates the tab order retrieval feature.
    #[test]
    fn test_layout_get_panels_in_tab_order() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        let mut panel3 = create_test_panel("panel3");
        
        panel1.tab_order = Some("3".to_string());
        panel2.tab_order = Some("1".to_string());
        panel3.tab_order = Some("2".to_string());
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2, panel3]));
        
        let panels_in_order = layout.get_panels_in_tab_order();
        assert_eq!(panels_in_order.len(), 3);
        assert_eq!(panels_in_order[0].id, "panel2"); // tab_order: "1"
        assert_eq!(panels_in_order[1].id, "panel3"); // tab_order: "2"
        assert_eq!(panels_in_order[2].id, "panel1"); // tab_order: "3"
    }

    /// Tests that Layout::get_panels_in_tab_order() ignores panels without tab_order.
    /// This test demonstrates tab order filtering behavior.
    #[test]
    fn test_layout_get_panels_in_tab_order_filtered() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        let mut panel3 = create_test_panel("panel3");
        
        panel1.tab_order = Some("1".to_string());
        panel2.tab_order = None; // No tab order
        panel3.tab_order = Some("2".to_string());
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2, panel3]));
        
        let panels_in_order = layout.get_panels_in_tab_order();
        assert_eq!(panels_in_order.len(), 2);
        assert_eq!(panels_in_order[0].id, "panel1");
        assert_eq!(panels_in_order[1].id, "panel3");
    }

    /// Tests that Layout::get_panels_in_tab_order() handles nested panels.
    /// This test demonstrates recursive tab order retrieval.
    #[test]
    fn test_layout_get_panels_in_tab_order_nested() {
        let mut child_panel = create_test_panel("child");
        child_panel.tab_order = Some("2".to_string());
        
        let mut parent_panel = create_test_panel("parent");
        parent_panel.tab_order = Some("1".to_string());
        parent_panel.children = Some(vec![child_panel]);
        
        let mut layout = create_test_layout("test", Some(vec![parent_panel]));
        
        let panels_in_order = layout.get_panels_in_tab_order();
        assert_eq!(panels_in_order.len(), 2);
        assert_eq!(panels_in_order[0].id, "parent");
        assert_eq!(panels_in_order[1].id, "child");
    }

    /// Tests that Layout::select_next_panel() advances selection correctly.
    /// This test demonstrates the next panel selection feature.
    #[test]
    fn test_layout_select_next_panel() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        let mut panel3 = create_test_panel("panel3");
        
        panel1.tab_order = Some("1".to_string());
        panel2.tab_order = Some("2".to_string());
        panel3.tab_order = Some("3".to_string());
        
        panel1.selected = Some(true);
        panel2.selected = Some(false);
        panel3.selected = Some(false);
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2, panel3]));
        
        layout.select_next_panel();
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "panel2");
    }

    /// Tests that Layout::select_next_panel() wraps around to first panel.
    /// This test demonstrates the wrap-around behavior in next panel selection.
    #[test]
    fn test_layout_select_next_panel_wrap_around() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        
        panel1.tab_order = Some("1".to_string());
        panel2.tab_order = Some("2".to_string());
        
        panel1.selected = Some(false);
        panel2.selected = Some(true); // Last panel selected
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        layout.select_next_panel();
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "panel1"); // Wrapped to first
    }

    /// Tests that Layout::select_next_panel() handles no selection.
    /// This test demonstrates next panel selection with no current selection.
    #[test]
    fn test_layout_select_next_panel_no_selection() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        
        panel1.tab_order = Some("1".to_string());
        panel2.tab_order = Some("2".to_string());
        
        panel1.selected = Some(false);
        panel2.selected = Some(false);
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        layout.select_next_panel();
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "panel1"); // First panel selected
    }

    /// Tests that Layout::select_previous_panel() moves selection backwards.
    /// This test demonstrates the previous panel selection feature.
    #[test]
    fn test_layout_select_previous_panel() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        let mut panel3 = create_test_panel("panel3");
        
        panel1.tab_order = Some("1".to_string());
        panel2.tab_order = Some("2".to_string());
        panel3.tab_order = Some("3".to_string());
        
        panel1.selected = Some(false);
        panel2.selected = Some(true);
        panel3.selected = Some(false);
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2, panel3]));
        
        layout.select_previous_panel();
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "panel1");
    }

    /// Tests that Layout::select_previous_panel() wraps around to last panel.
    /// This test demonstrates the wrap-around behavior in previous panel selection.
    #[test]
    fn test_layout_select_previous_panel_wrap_around() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        
        panel1.tab_order = Some("1".to_string());
        panel2.tab_order = Some("2".to_string());
        
        panel1.selected = Some(true); // First panel selected
        panel2.selected = Some(false);
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        layout.select_previous_panel();
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "panel2"); // Wrapped to last
    }

    /// Tests that Layout::select_previous_panel() handles no selection.
    /// This test demonstrates previous panel selection with no current selection.
    #[test]
    fn test_layout_select_previous_panel_no_selection() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        
        panel1.tab_order = Some("1".to_string());
        panel2.tab_order = Some("2".to_string());
        
        panel1.selected = Some(false);
        panel2.selected = Some(false);
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        layout.select_previous_panel();
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "panel2"); // Last panel selected
    }

    /// Tests that Layout navigation handles empty panel lists.
    /// This test demonstrates edge case handling in panel navigation.
    #[test]
    fn test_layout_navigation_empty_panels() {
        let mut layout = create_test_layout("test", None);
        
        // These should not panic
        layout.select_next_panel();
        layout.select_previous_panel();
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 0);
    }

    /// Tests that Layout navigation handles panels without tab order.
    /// This test demonstrates edge case handling with non-tabbable panels.
    #[test]
    fn test_layout_navigation_no_tab_order() {
        let mut panel1 = create_test_panel("panel1");
        let mut panel2 = create_test_panel("panel2");
        
        panel1.tab_order = None;
        panel2.tab_order = None;
        
        let mut layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        // These should not panic
        layout.select_next_panel();
        layout.select_previous_panel();
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 0);
    }

    // === Layout All Panels Tests ===

    /// Tests that Layout::get_all_panels() returns all panels.
    /// This test demonstrates the all panels retrieval feature.
    #[test]
    fn test_layout_get_all_panels() {
        let panel1 = create_test_panel("panel1");
        let panel2 = create_test_panel("panel2");
        let layout = create_test_layout("test", Some(vec![panel1, panel2]));
        
        let all_panels = layout.get_all_panels();
        assert_eq!(all_panels.len(), 2);
        assert_eq!(all_panels[0].id, "panel1");
        assert_eq!(all_panels[1].id, "panel2");
    }

    /// Tests that Layout::get_all_panels() includes nested panels.
    /// This test demonstrates recursive panel retrieval.
    #[test]
    fn test_layout_get_all_panels_nested() {
        let child_panel = create_test_panel("child");
        let parent_panel = Panel {
            id: "parent".to_string(),
            children: Some(vec![child_panel]),
            ..Default::default()
        };
        let layout = create_test_layout("test", Some(vec![parent_panel]));
        
        let all_panels = layout.get_all_panels();
        assert_eq!(all_panels.len(), 2);
        assert_eq!(all_panels[0].id, "parent");
        assert_eq!(all_panels[1].id, "child");
    }

    /// Tests that Layout::get_all_panels() handles empty layout.
    /// This test demonstrates edge case handling in all panels retrieval.
    #[test]
    fn test_layout_get_all_panels_empty() {
        let layout = create_test_layout("test", None);
        
        let all_panels = layout.get_all_panels();
        assert_eq!(all_panels.len(), 0);
    }

    // === Layout Clone Tests ===

    /// Tests that Layout implements Clone correctly.
    /// This test demonstrates Layout cloning behavior.
    #[test]
    fn test_layout_clone() {
        let panel1 = create_test_panel("panel1");
        let panel2 = create_test_panel("panel2");
        let layout1 = create_test_layout("test", Some(vec![panel1, panel2]));
        let layout2 = layout1.clone();
        
        assert_eq!(layout1.id, layout2.id);
        assert_eq!(layout1.title, layout2.title);
        assert_eq!(layout1.children.as_ref().unwrap().len(), 
                   layout2.children.as_ref().unwrap().len());
        assert_eq!(layout1.root, layout2.root);
        assert_eq!(layout1.active, layout2.active);
    }

    /// Tests that Layout cloning includes nested panels.
    /// This test demonstrates Layout cloning with nested structure.
    #[test]
    fn test_layout_clone_nested() {
        let child_panel = create_test_panel("child");
        let parent_panel = Panel {
            id: "parent".to_string(),
            children: Some(vec![child_panel]),
            ..Default::default()
        };
        let layout1 = create_test_layout("test", Some(vec![parent_panel]));
        let layout2 = layout1.clone();
        
        assert_eq!(layout1.children.as_ref().unwrap()[0].children.as_ref().unwrap().len(),
                   layout2.children.as_ref().unwrap()[0].children.as_ref().unwrap().len());
        assert_eq!(layout1.children.as_ref().unwrap()[0].children.as_ref().unwrap()[0].id,
                   layout2.children.as_ref().unwrap()[0].children.as_ref().unwrap()[0].id);
    }

    // === Layout Hash Tests ===

    /// Tests that Layout implements Hash correctly.
    /// This test demonstrates Layout hashing behavior.
    #[test]
    fn test_layout_hash() {
        let panel1 = create_test_panel("panel1");
        let panel2 = create_test_panel("panel2");
        let layout1 = create_test_layout("test", Some(vec![panel1.clone(), panel2.clone()]));
        let layout2 = create_test_layout("test", Some(vec![panel1, panel2]));
        let layout3 = create_test_layout("other", Some(vec![]));
        
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();
        
        layout1.hash(&mut hasher1);
        layout2.hash(&mut hasher2);
        layout3.hash(&mut hasher3);
        
        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    // === Layout PartialEq Tests ===

    /// Tests that Layout implements PartialEq correctly.
    /// This test demonstrates Layout equality comparison.
    #[test]
    fn test_layout_equality() {
        let panel1 = create_test_panel("panel1");
        let panel2 = create_test_panel("panel2");
        let layout1 = create_test_layout("test", Some(vec![panel1.clone(), panel2.clone()]));
        let layout2 = create_test_layout("test", Some(vec![panel1, panel2]));
        let layout3 = create_test_layout("other", Some(vec![]));
        
        assert_eq!(layout1, layout2);
        assert_ne!(layout1, layout3);
    }

    /// Tests that Layout equality considers all fields.
    /// This test demonstrates comprehensive Layout equality checking.
    #[test]
    fn test_layout_equality_comprehensive() {
        let panel = create_test_panel("panel");
        
        let layout1 = Layout {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            children: Some(vec![panel.clone()]),
            root: Some(true),
            active: Some(false),
            ..Default::default()
        };
        
        let layout2 = Layout {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            children: Some(vec![panel.clone()]),
            root: Some(true),
            active: Some(false),
            ..Default::default()
        };
        
        let layout3 = Layout {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            children: Some(vec![panel]),
            root: Some(false), // Different root value
            active: Some(false),
            ..Default::default()
        };
        
        assert_eq!(layout1, layout2);
        assert_ne!(layout1, layout3);
    }

    // === Layout Edge Cases ===

    /// Tests that Layout handles operations on empty children gracefully.
    /// This test demonstrates edge case handling with empty children.
    #[test]
    fn test_layout_empty_children_operations() {
        let mut layout = create_test_layout("test", Some(vec![]));
        
        // These should not panic
        let panels = layout.get_all_panels();
        assert_eq!(panels.len(), 0);
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 0);
        
        let tab_ordered = layout.get_panels_in_tab_order();
        assert_eq!(tab_ordered.len(), 0);
        
        layout.select_next_panel();
        layout.select_previous_panel();
        layout.select_only_panel("nonexistent");
        layout.deselect_all_panels();
    }

    /// Tests that Layout handles None children gracefully.
    /// This test demonstrates edge case handling with None children.
    #[test]
    fn test_layout_none_children_operations() {
        let mut layout = create_test_layout("test", None);
        
        // These should not panic
        let panels = layout.get_all_panels();
        assert_eq!(panels.len(), 0);
        
        let selected = layout.get_selected_panels();
        assert_eq!(selected.len(), 0);
        
        let tab_ordered = layout.get_panels_in_tab_order();
        assert_eq!(tab_ordered.len(), 0);
        
        layout.select_next_panel();
        layout.select_previous_panel();
        layout.select_only_panel("nonexistent");
        layout.deselect_all_panels();
    }
}
