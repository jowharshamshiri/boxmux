use crate::{model::muxbox::MuxBox, EntityType, FieldUpdate, Updatable};
use core::hash::Hash;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, hash::Hasher};

#[derive(Debug, Deserialize, Serialize, Default, PartialEq)]
pub struct Layout {
    pub id: String,
    pub title: Option<String>,
    pub refresh_interval: Option<u64>,
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
    pub active: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub muxbox_ids_in_tab_order: Option<Vec<String>>,
}

impl Hash for Layout {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.title.hash(state);
        self.refresh_interval.hash(state);
        if let Some(children) = &self.children {
            for muxbox in children {
                muxbox.hash(state);
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
            muxbox_ids_in_tab_order: None,
        }
    }

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

    pub fn get_muxboxes_in_tab_order(&mut self) -> Vec<&MuxBox> {
        fn collect_muxboxes_recursive<'a>(muxbox: &'a MuxBox, muxboxes: &mut Vec<&'a MuxBox>) {
            // Check if muxbox has a tab order and add it to the list
            if muxbox.tab_order.is_some() {
                muxboxes.push(muxbox);
            }

            // If children exist, iterate over them recursively
            if let Some(children) = &muxbox.children {
                for child in children {
                    collect_muxboxes_recursive(child, muxboxes);
                }
            }
        }

        if self.muxbox_ids_in_tab_order.is_some() {
            let mut muxboxes = Vec::new();
            for muxbox_id in self.muxbox_ids_in_tab_order.as_ref().unwrap() {
                if let Some(muxbox) = self.get_muxbox_by_id(muxbox_id) {
                    muxboxes.push(muxbox);
                }
            }
            muxboxes
        } else {
            let mut muxboxes = Vec::new();
            // Start recursion for each top-level child
            if let Some(children) = &self.children {
                for muxbox in children {
                    collect_muxboxes_recursive(muxbox, &mut muxboxes);
                }
            }

            // Sort muxboxes by their tab order
            muxboxes.sort_by(|a, b| {
                a.tab_order
                    .as_ref()
                    .unwrap()
                    .cmp(b.tab_order.as_ref().unwrap())
            });

            self.muxbox_ids_in_tab_order = Some(muxboxes.iter().map(|p| p.id.clone()).collect());

            muxboxes
        }
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
        let muxboxes = self.get_muxboxes_in_tab_order();
        if muxboxes.is_empty() {
            return; // Early return if there are no muxboxes
        }

        let selected_muxbox_index = muxboxes.iter().position(|p| p.selected.unwrap_or(false));

        let next_muxbox_index = match selected_muxbox_index {
            Some(index) => (index + 1) % muxboxes.len(), // Get next muxbox, wrap around if at the end
            None => 0, // No muxbox is selected, select the first one
        };

        let next_muxbox_id = muxboxes[next_muxbox_index].id.clone();
        self.select_only_muxbox(&next_muxbox_id);
    }

    pub fn select_previous_muxbox(&mut self) {
        let muxboxes = self.get_muxboxes_in_tab_order();
        if muxboxes.is_empty() {
            return; // Early return if there are no muxboxes
        }

        let selected_muxbox_index = muxboxes.iter().position(|p| p.selected.unwrap_or(false));

        let previous_muxbox_index = match selected_muxbox_index {
            Some(index) => {
                if index == 0 {
                    muxboxes.len() - 1 // Wrap around to the last muxbox if the first one is currently selected
                } else {
                    index - 1 // Select the previous muxbox
                }
            }
            None => muxboxes.len() - 1, // No muxbox is selected, select the last one
        };

        let previous_muxbox_id = muxboxes[previous_muxbox_index].id.clone();
        self.select_only_muxbox(&previous_muxbox_id);
    }

    pub fn deselect_all_muxboxes(&mut self) {
        if let Some(children) = &mut self.children {
            for muxbox in children {
                muxbox.selected = Some(false);
            }
        }
    }

    pub fn replace_muxbox_recursive(&mut self, replacement_muxbox: &MuxBox) -> Option<bool> {
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

        if let Some(ref mut children) = self.children {
            Some(replace_in_muxboxes(children, replacement_muxbox))
        } else {
            Some(false)
        }
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

    pub fn find_muxbox_at_coordinates(&self, x: u16, y: u16) -> Option<&MuxBox> {
        fn find_in_muxboxes_at_coords<'a>(
            muxboxes: &'a [MuxBox],
            x: u16,
            y: u16,
        ) -> Option<&'a MuxBox> {
            // Collect matching boxes and their children, then sort by z_index (highest first)
            let mut candidates: Vec<&MuxBox> = Vec::new();
            
            // Find all boxes that contain the click coordinates
            for muxbox in muxboxes {
                let bounds = muxbox.bounds();
                if x >= bounds.x1 as u16
                    && x <= bounds.x2 as u16
                    && y >= bounds.y1 as u16
                    && y <= bounds.y2 as u16
                {
                    candidates.push(muxbox);
                }
            }
            
            // Sort candidates by z_index (highest first for click priority)
            candidates.sort_by_key(|muxbox| std::cmp::Reverse(muxbox.effective_z_index()));
            
            // Check candidates in z_index order (highest z_index first)
            for muxbox in candidates {
                // Check children first (they take priority over parent)
                if let Some(ref children) = muxbox.children {
                    if let Some(child_muxbox) = find_in_muxboxes_at_coords(children, x, y) {
                        return Some(child_muxbox);
                    }
                }
                // Return this muxbox (highest z_index among candidates)
                return Some(muxbox);
            }
            None
        }

        if let Some(ref children) = self.children {
            find_in_muxboxes_at_coords(children, x, y)
        } else {
            None
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
            muxbox_ids_in_tab_order: self.muxbox_ids_in_tab_order.clone(),
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
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
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
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "fill".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.fill_char != other.fill_char {
            if let Some(new_value) = other.fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "fill_char".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_fill_char != other.selected_fill_char {
            if let Some(new_value) = other.selected_fill_char {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "selected_fill_char".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.border != other.border {
            if let Some(new_value) = other.border {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "border".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.border_color != other.border_color {
            if let Some(new_value) = &other.border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_border_color != other.selected_border_color {
            if let Some(new_value) = &other.selected_border_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "selected_border_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.bg_color != other.bg_color {
            if let Some(new_value) = &other.bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_bg_color != other.selected_bg_color {
            if let Some(new_value) = &other.selected_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "selected_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.fg_color != other.fg_color {
            if let Some(new_value) = &other.fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_fg_color != other.selected_fg_color {
            if let Some(new_value) = &other.selected_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "selected_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_fg_color != other.title_fg_color {
            if let Some(new_value) = &other.title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "title_fg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_bg_color != other.title_bg_color {
            if let Some(new_value) = &other.title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.title_position != other.title_position {
            if let Some(new_value) = &other.title_position {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "title_position".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_title_bg_color != other.selected_title_bg_color {
            if let Some(new_value) = &other.selected_title_bg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "selected_title_bg_color".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.selected_title_fg_color != other.selected_title_fg_color {
            if let Some(new_value) = &other.selected_title_fg_color {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
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
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "overflow_behavior".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.root != other.root {
            if let Some(new_value) = other.root {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "root".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.on_keypress != other.on_keypress {
            if let Some(new_value) = &other.on_keypress {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "on_keypress".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.active != other.active {
            if let Some(new_value) = other.active {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "active".to_string(),
                    new_value: serde_json::to_value(new_value).unwrap(),
                });
            }
        }

        if self.muxbox_ids_in_tab_order != other.muxbox_ids_in_tab_order {
            if let Some(new_value) = &other.muxbox_ids_in_tab_order {
                updates.push(FieldUpdate {
                    entity_type: EntityType::Layout,
                    entity_id: Some(self.id.clone()), // This is the entity id of the layout, not the muxbox
                    field_name: "muxbox_ids_in_tab_order".to_string(),
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
                "muxbox_ids_in_tab_order" => {
                    if let Some(new_muxbox_ids_in_tab_order) = update.new_value.as_array() {
                        self.muxbox_ids_in_tab_order = Some(
                            new_muxbox_ids_in_tab_order
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
    use crate::model::muxbox::MuxBox;

    // === Helper Functions ===

    /// Creates a basic test muxbox with the given id.
    /// This helper demonstrates how to create a MuxBox for Layout testing.
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
            tab_order: Some(id.to_string()),
            selected: Some(false),
            ..Default::default()
        }
    }

    /// Creates a test Layout with the given id and optional children.
    /// This helper demonstrates how to create a Layout for testing.
    fn create_test_layout(id: &str, children: Option<Vec<MuxBox>>) -> Layout {
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
        assert_eq!(layout.root, None);
        assert_eq!(layout.active, None);
        assert_eq!(layout.refresh_interval, None);
        assert_eq!(layout.muxbox_ids_in_tab_order, None);
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
        assert_eq!(layout.muxbox_ids_in_tab_order, None);
    }

    // === Layout Creation Tests ===

    /// Tests creating a Layout with specific values.
    /// This test demonstrates how to create a Layout with custom properties.
    #[test]
    fn test_layout_creation() {
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox2 = create_test_muxbox("muxbox2");
        let children = vec![muxbox1, muxbox2];

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

    // === Layout MuxBox Management Tests ===

    /// Tests that Layout::get_muxbox_by_id() finds muxboxes correctly.
    /// This test demonstrates the muxbox retrieval feature.
    #[test]
    fn test_layout_get_muxbox_by_id() {
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox2 = create_test_muxbox("muxbox2");
        let layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        let found_muxbox = layout.get_muxbox_by_id("muxbox1");
        assert!(found_muxbox.is_some());
        assert_eq!(found_muxbox.unwrap().id, "muxbox1");

        let not_found = layout.get_muxbox_by_id("nonexistent");
        assert!(not_found.is_none());
    }

    /// Tests that Layout::get_muxbox_by_id() finds nested muxboxes correctly.
    /// This test demonstrates the recursive muxbox retrieval feature.
    #[test]
    fn test_layout_get_muxbox_by_id_nested() {
        let child_muxbox = create_test_muxbox("child");
        let parent_muxbox = MuxBox {
            id: "parent".to_string(),
            children: Some(vec![child_muxbox]),
            ..Default::default()
        };
        let layout = create_test_layout("test", Some(vec![parent_muxbox]));

        let found_child = layout.get_muxbox_by_id("child");
        assert!(found_child.is_some());
        assert_eq!(found_child.unwrap().id, "child");
    }

    /// Tests that Layout::get_muxbox_by_id_mut() finds and allows modification.
    /// This test demonstrates the mutable muxbox retrieval feature.
    #[test]
    fn test_layout_get_muxbox_by_id_mut() {
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox2 = create_test_muxbox("muxbox2");
        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        let found_muxbox = layout.get_muxbox_by_id_mut("muxbox1");
        assert!(found_muxbox.is_some());

        // Modify the muxbox
        found_muxbox.unwrap().title = Some("Modified Title".to_string());

        // Verify the modification
        let verified_muxbox = layout.get_muxbox_by_id("muxbox1");
        assert_eq!(
            verified_muxbox.unwrap().title,
            Some("Modified Title".to_string())
        );
    }

    /// Tests that Layout::get_muxbox_by_id_mut() handles empty layout.
    /// This test demonstrates edge case handling in mutable muxbox retrieval.
    #[test]
    fn test_layout_get_muxbox_by_id_mut_empty() {
        let mut layout = create_test_layout("test", None);

        let found_muxbox = layout.get_muxbox_by_id_mut("nonexistent");
        assert!(found_muxbox.is_none());
    }

    // === Layout MuxBox Selection Tests ===

    /// Tests that Layout::get_selected_muxboxes() returns selected muxboxes.
    /// This test demonstrates the selected muxbox retrieval feature.
    #[test]
    fn test_layout_get_selected_muxboxes() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");
        let mut muxbox3 = create_test_muxbox("muxbox3");

        muxbox1.selected = Some(true);
        muxbox2.selected = Some(false);
        muxbox3.selected = Some(true);

        let layout = create_test_layout("test", Some(vec![muxbox1, muxbox2, muxbox3]));

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 2);
        assert_eq!(selected[0].id, "muxbox1");
        assert_eq!(selected[1].id, "muxbox3");
    }

    /// Tests that Layout::get_selected_muxboxes() handles no selected muxboxes.
    /// This test demonstrates edge case handling in selected muxbox retrieval.
    #[test]
    fn test_layout_get_selected_muxboxes_none() {
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox2 = create_test_muxbox("muxbox2");
        let layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 0);
    }

    /// Tests that Layout::select_only_muxbox() selects only the specified muxbox.
    /// This test demonstrates the exclusive muxbox selection feature.
    #[test]
    fn test_layout_select_only_muxbox() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");
        let mut muxbox3 = create_test_muxbox("muxbox3");

        muxbox1.selected = Some(true);
        muxbox2.selected = Some(true);
        muxbox3.selected = Some(false);

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2, muxbox3]));

        layout.select_only_muxbox("muxbox2");

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "muxbox2");
    }

    /// Tests that Layout::select_only_muxbox() handles nonexistent muxbox.
    /// This test demonstrates edge case handling in muxbox selection.
    #[test]
    fn test_layout_select_only_muxbox_nonexistent() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        muxbox1.selected = Some(true);

        let mut layout = create_test_layout("test", Some(vec![muxbox1]));

        layout.select_only_muxbox("nonexistent");

        // All muxboxes should be deselected
        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 0);
    }

    /// Tests that Layout::deselect_all_muxboxes() deselects all muxboxes.
    /// This test demonstrates the muxbox deselection feature.
    #[test]
    fn test_layout_deselect_all_muxboxes() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");

        muxbox1.selected = Some(true);
        muxbox2.selected = Some(true);

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        layout.deselect_all_muxboxes();

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 0);
    }

    // === Layout Tab Order Tests ===

    /// Tests that Layout::get_muxboxes_in_tab_order() returns muxboxes in tab order.
    /// This test demonstrates the tab order retrieval feature.
    #[test]
    fn test_layout_get_muxboxes_in_tab_order() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");
        let mut muxbox3 = create_test_muxbox("muxbox3");

        muxbox1.tab_order = Some("3".to_string());
        muxbox2.tab_order = Some("1".to_string());
        muxbox3.tab_order = Some("2".to_string());

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2, muxbox3]));

        let muxboxes_in_order = layout.get_muxboxes_in_tab_order();
        assert_eq!(muxboxes_in_order.len(), 3);
        assert_eq!(muxboxes_in_order[0].id, "muxbox2"); // tab_order: "1"
        assert_eq!(muxboxes_in_order[1].id, "muxbox3"); // tab_order: "2"
        assert_eq!(muxboxes_in_order[2].id, "muxbox1"); // tab_order: "3"
    }

    /// Tests that Layout::get_muxboxes_in_tab_order() ignores muxboxes without tab_order.
    /// This test demonstrates tab order filtering behavior.
    #[test]
    fn test_layout_get_muxboxes_in_tab_order_filtered() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");
        let mut muxbox3 = create_test_muxbox("muxbox3");

        muxbox1.tab_order = Some("1".to_string());
        muxbox2.tab_order = None; // No tab order
        muxbox3.tab_order = Some("2".to_string());

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2, muxbox3]));

        let muxboxes_in_order = layout.get_muxboxes_in_tab_order();
        assert_eq!(muxboxes_in_order.len(), 2);
        assert_eq!(muxboxes_in_order[0].id, "muxbox1");
        assert_eq!(muxboxes_in_order[1].id, "muxbox3");
    }

    /// Tests that Layout::get_muxboxes_in_tab_order() handles nested muxboxes.
    /// This test demonstrates recursive tab order retrieval.
    #[test]
    fn test_layout_get_muxboxes_in_tab_order_nested() {
        let mut child_muxbox = create_test_muxbox("child");
        child_muxbox.tab_order = Some("2".to_string());

        let mut parent_muxbox = create_test_muxbox("parent");
        parent_muxbox.tab_order = Some("1".to_string());
        parent_muxbox.children = Some(vec![child_muxbox]);

        let mut layout = create_test_layout("test", Some(vec![parent_muxbox]));

        let muxboxes_in_order = layout.get_muxboxes_in_tab_order();
        assert_eq!(muxboxes_in_order.len(), 2);
        assert_eq!(muxboxes_in_order[0].id, "parent");
        assert_eq!(muxboxes_in_order[1].id, "child");
    }

    /// Tests that Layout::select_next_muxbox() advances selection correctly.
    /// This test demonstrates the next muxbox selection feature.
    #[test]
    fn test_layout_select_next_muxbox() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");
        let mut muxbox3 = create_test_muxbox("muxbox3");

        muxbox1.tab_order = Some("1".to_string());
        muxbox2.tab_order = Some("2".to_string());
        muxbox3.tab_order = Some("3".to_string());

        muxbox1.selected = Some(true);
        muxbox2.selected = Some(false);
        muxbox3.selected = Some(false);

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2, muxbox3]));

        layout.select_next_muxbox();

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "muxbox2");
    }

    /// Tests that Layout::select_next_muxbox() wraps around to first muxbox.
    /// This test demonstrates the wrap-around behavior in next muxbox selection.
    #[test]
    fn test_layout_select_next_muxbox_wrap_around() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");

        muxbox1.tab_order = Some("1".to_string());
        muxbox2.tab_order = Some("2".to_string());

        muxbox1.selected = Some(false);
        muxbox2.selected = Some(true); // Last muxbox selected

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        layout.select_next_muxbox();

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "muxbox1"); // Wrapped to first
    }

    /// Tests that Layout::select_next_muxbox() handles no selection.
    /// This test demonstrates next muxbox selection with no current selection.
    #[test]
    fn test_layout_select_next_muxbox_no_selection() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");

        muxbox1.tab_order = Some("1".to_string());
        muxbox2.tab_order = Some("2".to_string());

        muxbox1.selected = Some(false);
        muxbox2.selected = Some(false);

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        layout.select_next_muxbox();

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "muxbox1"); // First muxbox selected
    }

    /// Tests that Layout::select_previous_muxbox() moves selection backwards.
    /// This test demonstrates the previous muxbox selection feature.
    #[test]
    fn test_layout_select_previous_muxbox() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");
        let mut muxbox3 = create_test_muxbox("muxbox3");

        muxbox1.tab_order = Some("1".to_string());
        muxbox2.tab_order = Some("2".to_string());
        muxbox3.tab_order = Some("3".to_string());

        muxbox1.selected = Some(false);
        muxbox2.selected = Some(true);
        muxbox3.selected = Some(false);

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2, muxbox3]));

        layout.select_previous_muxbox();

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "muxbox1");
    }

    /// Tests that Layout::select_previous_muxbox() wraps around to last muxbox.
    /// This test demonstrates the wrap-around behavior in previous muxbox selection.
    #[test]
    fn test_layout_select_previous_muxbox_wrap_around() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");

        muxbox1.tab_order = Some("1".to_string());
        muxbox2.tab_order = Some("2".to_string());

        muxbox1.selected = Some(true); // First muxbox selected
        muxbox2.selected = Some(false);

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        layout.select_previous_muxbox();

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "muxbox2"); // Wrapped to last
    }

    /// Tests that Layout::select_previous_muxbox() handles no selection.
    /// This test demonstrates previous muxbox selection with no current selection.
    #[test]
    fn test_layout_select_previous_muxbox_no_selection() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");

        muxbox1.tab_order = Some("1".to_string());
        muxbox2.tab_order = Some("2".to_string());

        muxbox1.selected = Some(false);
        muxbox2.selected = Some(false);

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        layout.select_previous_muxbox();

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].id, "muxbox2"); // Last muxbox selected
    }

    /// Tests that Layout navigation handles empty muxbox lists.
    /// This test demonstrates edge case handling in muxbox navigation.
    #[test]
    fn test_layout_navigation_empty_muxboxes() {
        let mut layout = create_test_layout("test", None);

        // These should not panic
        layout.select_next_muxbox();
        layout.select_previous_muxbox();

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 0);
    }

    /// Tests that Layout navigation handles muxboxes without tab order.
    /// This test demonstrates edge case handling with non-tabbable muxboxes.
    #[test]
    fn test_layout_navigation_no_tab_order() {
        let mut muxbox1 = create_test_muxbox("muxbox1");
        let mut muxbox2 = create_test_muxbox("muxbox2");

        muxbox1.tab_order = None;
        muxbox2.tab_order = None;

        let mut layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        // These should not panic
        layout.select_next_muxbox();
        layout.select_previous_muxbox();

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 0);
    }

    // === Layout All MuxBoxes Tests ===

    /// Tests that Layout::get_all_muxboxes() returns all muxboxes.
    /// This test demonstrates the all muxboxes retrieval feature.
    #[test]
    fn test_layout_get_all_muxboxes() {
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox2 = create_test_muxbox("muxbox2");
        let layout = create_test_layout("test", Some(vec![muxbox1, muxbox2]));

        let all_muxboxes = layout.get_all_muxboxes();
        assert_eq!(all_muxboxes.len(), 2);
        assert_eq!(all_muxboxes[0].id, "muxbox1");
        assert_eq!(all_muxboxes[1].id, "muxbox2");
    }

    /// Tests that Layout::get_all_muxboxes() includes nested muxboxes.
    /// This test demonstrates recursive muxbox retrieval.
    #[test]
    fn test_layout_get_all_muxboxes_nested() {
        let child_muxbox = create_test_muxbox("child");
        let parent_muxbox = MuxBox {
            id: "parent".to_string(),
            children: Some(vec![child_muxbox]),
            ..Default::default()
        };
        let layout = create_test_layout("test", Some(vec![parent_muxbox]));

        let all_muxboxes = layout.get_all_muxboxes();
        assert_eq!(all_muxboxes.len(), 2);
        assert_eq!(all_muxboxes[0].id, "parent");
        assert_eq!(all_muxboxes[1].id, "child");
    }

    /// Tests that Layout::get_all_muxboxes() handles empty layout.
    /// This test demonstrates edge case handling in all muxboxes retrieval.
    #[test]
    fn test_layout_get_all_muxboxes_empty() {
        let layout = create_test_layout("test", None);

        let all_muxboxes = layout.get_all_muxboxes();
        assert_eq!(all_muxboxes.len(), 0);
    }

    // === Layout Clone Tests ===

    /// Tests that Layout implements Clone correctly.
    /// This test demonstrates Layout cloning behavior.
    #[test]
    fn test_layout_clone() {
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox2 = create_test_muxbox("muxbox2");
        let layout1 = create_test_layout("test", Some(vec![muxbox1, muxbox2]));
        let layout2 = layout1.clone();

        assert_eq!(layout1.id, layout2.id);
        assert_eq!(layout1.title, layout2.title);
        assert_eq!(
            layout1.children.as_ref().unwrap().len(),
            layout2.children.as_ref().unwrap().len()
        );
        assert_eq!(layout1.root, layout2.root);
        assert_eq!(layout1.active, layout2.active);
    }

    /// Tests that Layout cloning includes nested muxboxes.
    /// This test demonstrates Layout cloning with nested structure.
    #[test]
    fn test_layout_clone_nested() {
        let child_muxbox = create_test_muxbox("child");
        let parent_muxbox = MuxBox {
            id: "parent".to_string(),
            children: Some(vec![child_muxbox]),
            ..Default::default()
        };
        let layout1 = create_test_layout("test", Some(vec![parent_muxbox]));
        let layout2 = layout1.clone();

        assert_eq!(
            layout1.children.as_ref().unwrap()[0]
                .children
                .as_ref()
                .unwrap()
                .len(),
            layout2.children.as_ref().unwrap()[0]
                .children
                .as_ref()
                .unwrap()
                .len()
        );
        assert_eq!(
            layout1.children.as_ref().unwrap()[0]
                .children
                .as_ref()
                .unwrap()[0]
                .id,
            layout2.children.as_ref().unwrap()[0]
                .children
                .as_ref()
                .unwrap()[0]
                .id
        );
    }

    // === Layout Hash Tests ===

    /// Tests that Layout implements Hash correctly.
    /// This test demonstrates Layout hashing behavior.
    #[test]
    fn test_layout_hash() {
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox2 = create_test_muxbox("muxbox2");
        let layout1 = create_test_layout("test", Some(vec![muxbox1.clone(), muxbox2.clone()]));
        let layout2 = create_test_layout("test", Some(vec![muxbox1, muxbox2]));
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
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox2 = create_test_muxbox("muxbox2");
        let layout1 = create_test_layout("test", Some(vec![muxbox1.clone(), muxbox2.clone()]));
        let layout2 = create_test_layout("test", Some(vec![muxbox1, muxbox2]));
        let layout3 = create_test_layout("other", Some(vec![]));

        assert_eq!(layout1, layout2);
        assert_ne!(layout1, layout3);
    }

    /// Tests that Layout equality considers all fields.
    /// This test demonstrates comprehensive Layout equality checking.
    #[test]
    fn test_layout_equality_comprehensive() {
        let muxbox = create_test_muxbox("muxbox");

        let layout1 = Layout {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            children: Some(vec![muxbox.clone()]),
            root: Some(true),
            active: Some(false),
            ..Default::default()
        };

        let layout2 = Layout {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            children: Some(vec![muxbox.clone()]),
            root: Some(true),
            active: Some(false),
            ..Default::default()
        };

        let layout3 = Layout {
            id: "test".to_string(),
            title: Some("Test".to_string()),
            children: Some(vec![muxbox]),
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
        let muxboxes = layout.get_all_muxboxes();
        assert_eq!(muxboxes.len(), 0);

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 0);

        let tab_ordered = layout.get_muxboxes_in_tab_order();
        assert_eq!(tab_ordered.len(), 0);

        layout.select_next_muxbox();
        layout.select_previous_muxbox();
        layout.select_only_muxbox("nonexistent");
        layout.deselect_all_muxboxes();
    }

    /// Tests that Layout handles None children gracefully.
    /// This test demonstrates edge case handling with None children.
    #[test]
    fn test_layout_none_children_operations() {
        let mut layout = create_test_layout("test", None);

        // These should not panic
        let muxboxes = layout.get_all_muxboxes();
        assert_eq!(muxboxes.len(), 0);

        let selected = layout.get_selected_muxboxes();
        assert_eq!(selected.len(), 0);

        let tab_ordered = layout.get_muxboxes_in_tab_order();
        assert_eq!(tab_ordered.len(), 0);

        layout.select_next_muxbox();
        layout.select_previous_muxbox();
        layout.select_only_muxbox("nonexistent");
        layout.deselect_all_muxboxes();
    }
}
