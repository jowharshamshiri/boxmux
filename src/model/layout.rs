
use crate::utils::*;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;
use core::hash::Hash;
use std::hash::{DefaultHasher, Hasher};

use serde::{de, ser};

use crate::model::common::{ScreenBuffer};
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
			if Some(true) == panel.selected {
				selected_panels.push(panel);
			}
		}
		selected_panels
	}

	pub fn select_only_panel(&mut self, id: &str) {
		for panel in &mut self.children {
			panel.selected = Some(false);
			if panel.id == id {
				panel.selected = Some(true);
			}
		}
	}

	pub fn get_panels_in_tab_order(&self) -> Vec<&Panel> {
		let mut panels = Vec::new();
		for panel in &self.children {
			panels.push(panel);
		}
		panels.sort_by(|a, b| a.tab_order.cmp(&b.tab_order));
		panels
	}

	pub fn select_next_panel(&mut self) {
		let panels = self.get_panels_in_tab_order();
		let mut selected_panel_index = None;
		for (index, panel) in panels.iter().enumerate() {
			if Some(true) == panel.selected {
				selected_panel_index = Some(index);
				break;
			}
		}
		if let Some(selected_panel_index) = selected_panel_index {
			let next_panel_index = (selected_panel_index + 1) % panels.len();
			for panel in &mut self.children {
				panel.selected = Some(false);
			}
			self.children[next_panel_index].selected = Some(true);
		}
	}

	pub fn select_previous_panel(&mut self) {
		let panels = self.get_panels_in_tab_order();
		let mut selected_panel_index = None;
		for (index, panel) in panels.iter().enumerate() {
			if Some(true) == panel.selected {
				selected_panel_index = Some(index);
				break;
			}
		}
		if let Some(selected_panel_index) = selected_panel_index {
			let previous_panel_index = if selected_panel_index == 0 {
				panels.len() - 1
			} else {
				selected_panel_index - 1
			};
			for panel in &mut self.children {
				panel.selected = Some(false);
			}
			self.children[previous_panel_index].selected = Some(true);
		}
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
            children: self.children.iter().map(|panel| panel.deep_clone()).collect(),
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
        }
    }
}