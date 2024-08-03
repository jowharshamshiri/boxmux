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
            recursive_collect(&children, &mut selected_panels);
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
                    .cmp(&b.tab_order.as_ref().unwrap())
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
            recursive_collect(&children, &mut all_panels);
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
            cloned_children = Some(children.iter().map(|panel| panel.clone()).collect());
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
                    new_value: serde_json::to_value(&new_value).unwrap(),
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
                    new_value: serde_json::to_value(&new_value).unwrap(),
                });
            }
        }

        if self.fill_char != other.fill_char {
            if let Some(new_value) = other.fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "fill_char".to_string(),
                    new_value: serde_json::to_value(&new_value).unwrap(),
                });
            }
        }

        if self.selected_fill_char != other.selected_fill_char {
            if let Some(new_value) = other.selected_fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "selected_fill_char".to_string(),
                    new_value: serde_json::to_value(&new_value).unwrap(),
                });
            }
        }

        if self.border != other.border {
            if let Some(new_value) = other.border {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the panel
                    field_name: "border".to_string(),
                    new_value: serde_json::to_value(&new_value).unwrap(),
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
                    new_value: serde_json::to_value(&new_value).unwrap(),
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
                    new_value: serde_json::to_value(&new_value).unwrap(),
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
