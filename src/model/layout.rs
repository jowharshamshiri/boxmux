use crate::utils::*;
use core::hash::Hash;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::hash::{DefaultHasher, Hasher};
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;

use serde::{de, ser};

use crate::model::common::ScreenBuffer;
use crate::model::panel::Panel;

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct Layout {
    pub id: String,
    pub title: String,
    pub refresh_interval: Option<u64>,
    pub children: Vec<Panel>,
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
    pub overflow_behavior: Option<String>,
    pub root: Option<bool>,

    #[serde(skip)]
    pub active: Option<bool>,
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
        self.overflow_behavior.hash(state);
        self.root.hash(state);
        self.active.hash(state);
    }
}

impl Layout {
    pub fn new() -> Self {
        Layout {
            id: String::new(),
            title: String::new(),
            refresh_interval: None,
            children: Vec::new(),
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
            overflow_behavior: None,
            root: Some(false),
            active: Some(false),
        }
    }

    pub fn get_panel_by_id(&self, id: &str) -> Option<&Panel> {
        for panel in &self.children {
            if panel.id == id {
                return Some(panel);
            }
        }
        None
    }

    pub fn get_panel_by_id_mut(&mut self, id: &str) -> Option<&mut Panel> {
        for panel in &mut self.children {
            if panel.id == id {
                return Some(panel);
            }
        }
        None
    }

    pub fn get_selected_panels(&self) -> Vec<&Panel> {
        let mut selected_panels = Vec::new();
        for panel in &self.children {
            if panel.selected.unwrap_or(false) {
                selected_panels.push(panel);
            }
        }
        selected_panels
    }

    pub fn select_only_panel(&mut self, id: &str) {
        for panel in &mut self.children {
            if panel.id == id {
                panel.selected = Some(true);
            } else {
                panel.selected = Some(false);
            }
        }
    }

	pub fn get_panels_in_tab_order(&self) -> Vec<&Panel> {
		let mut panels: Vec<&Panel> = self.children.iter()
			.filter(|panel| panel.tab_order.is_some()) // Filter out panels without a tab_order
			.collect();
	
		// Sort only the panels with a valid tab_order
		panels.sort_by(|a, b| a.tab_order.as_ref().unwrap().cmp(&b.tab_order.as_ref().unwrap()));
	
		panels
	}

	pub fn select_next_panel(&mut self) {
		let panels = self.get_panels_in_tab_order();
		if panels.is_empty() {
			return; // Early return if there are no panels
		}

		let selected_panel_index = panels.iter()
			.position(|p| p.selected.unwrap_or(false));

		let next_panel_index = match selected_panel_index {
			Some(index) => (index + 1) % panels.len(), // Get next panel, wrap around if at the end
			None => 0, // No panel is selected, select the first one
		};

		let next_panel_id = panels[next_panel_index].id.clone(); 
		self.select_only_panel(&next_panel_id);
	}

    pub fn select_previous_panel(&mut self) {
		let panels = self.get_panels_in_tab_order();
		if panels.is_empty() {
			return; // Early return if there are no panels
		}

		let selected_panel_index = panels.iter()
			.position(|p| p.selected.unwrap_or(false));

		let previous_panel_index = match selected_panel_index {
			Some(index) => {
				if index == 0 {
					panels.len() - 1 // Wrap around to the last panel if the first one is currently selected
				} else {
					index - 1 // Select the previous panel
				}
			},
			None => panels.len() - 1, // No panel is selected, select the last one
		};

		let previous_panel_id = panels[previous_panel_index].id.clone(); 
		self.select_only_panel(&previous_panel_id);
	}

    pub fn deselect_all_panels(&mut self) {
        for panel in &mut self.children {
            panel.selected = Some(false);
        }
    }
}

impl Layout {
    pub fn deep_clone(&self) -> Self {
        Layout {
            id: self.id.clone(),
            title: self.title.clone(),
            refresh_interval: self.refresh_interval,
            children: self
                .children
                .iter()
                .map(|panel| panel.deep_clone())
                .collect(),
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
            overflow_behavior: self.overflow_behavior.clone(),
            root: self.root,
            active: self.active,
        }
    }
}
