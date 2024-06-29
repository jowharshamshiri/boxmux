use crate::utils::{
    draw_panel, fill_panel, get_bg_color, get_fg_color, input_bounds_to_bounds, screen_bounds,
};
use core::hash::Hash;
use std::hash::{DefaultHasher, Hasher};
use lazy_static::lazy_static;
use serde::{de, ser};
use serde::{Deserialize, Serialize};

use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;

use crate::model::common::*;
use crate::model::layout::Layout;

use crate::thread_manager::{Runnable, ThreadManager};

use crate::{utils::*, App, AppContext};
use crate::model::app::AppGraph;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Panel {
    pub id: String,
    pub title: Option<String>,
    pub position: InputBounds,
    pub min_width: Option<usize>,
    pub min_height: Option<usize>,
    pub max_width: Option<usize>,
    pub max_height: Option<usize>,
    pub overflow_behavior: Option<String>,
    pub content: Option<String>,
    pub scroll: bool,
    pub refresh_interval: Option<u64>,
    pub tab_order: Option<String>,
    pub on_error: Option<Vec<String>>,
    pub on_enter: Option<Vec<String>>,
    pub on_leave: Option<Vec<String>>,
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
    pub on_refresh: Option<Vec<String>>,
    pub thread: Option<bool>,
	pub horizontal_scroll: Option<f64>,
	pub vertical_scroll: Option<f64>,
	pub selected: Option<bool>,
    #[serde(skip)]
    pub output: String,
    #[serde(skip)]
	pub parent_id: Option<String>,       
    #[serde(skip)]
	pub parent_layout_id: Option<String> 
}
impl Hash for Panel {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.title.hash(state);
        self.position.hash(state);
        self.min_width.hash(state);
        self.min_height.hash(state);
        self.max_width.hash(state);
        self.max_height.hash(state);
        self.overflow_behavior.hash(state);
        self.content.hash(state);
        self.scroll.hash(state);
        self.refresh_interval.hash(state);
        self.tab_order.hash(state);
        self.on_error.hash(state);
        self.on_enter.hash(state);
        self.on_leave.hash(state);
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
        self.on_refresh.hash(state);
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
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            overflow_behavior: None,
            content: None,
            scroll: false,
            refresh_interval: None,
            tab_order: None,
            on_error: None,
            on_enter: None,
            on_leave: None,
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
            on_refresh: None,
            thread: Some(false),
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
        self.id == other.id &&
        self.title == other.title &&
        self.position == other.position &&
        self.min_width == other.min_width &&
        self.min_height == other.min_height &&
        self.max_width == other.max_width &&
        self.max_height == other.max_height &&
        self.overflow_behavior == other.overflow_behavior &&
        self.content == other.content &&
        self.scroll == other.scroll &&
        self.refresh_interval == other.refresh_interval &&
        self.tab_order == other.tab_order &&
        self.on_error == other.on_error &&
        self.on_enter == other.on_enter &&
        self.on_leave == other.on_leave &&
        self.next_focus_id == other.next_focus_id &&
        self.children == other.children &&
        self.fill == other.fill &&
        self.fill_char == other.fill_char &&
        self.selected_fill_char == other.selected_fill_char &&
        self.border == other.border &&
        self.border_color == other.border_color &&
        self.selected_border_color == other.selected_border_color &&
        self.bg_color == other.bg_color &&
        self.selected_bg_color == other.selected_bg_color &&
        self.fg_color == other.fg_color &&
        self.selected_fg_color == other.selected_fg_color &&
        self.title_fg_color == other.title_fg_color &&
        self.title_bg_color == other.title_bg_color &&
        self.selected_title_bg_color == other.selected_title_bg_color &&
        self.selected_title_fg_color == other.selected_title_fg_color &&
        self.title_position == other.title_position &&
        self.on_refresh == other.on_refresh &&
        self.thread == other.thread &&
        self.horizontal_scroll.map(|hs| hs.to_bits()) == other.horizontal_scroll.map(|hs| hs.to_bits()) &&
        self.vertical_scroll.map(|vs| vs.to_bits()) == other.vertical_scroll.map(|vs| vs.to_bits()) &&
        self.selected == other.selected &&
        self.parent_id == other.parent_id &&
        self.parent_layout_id == other.parent_layout_id &&
        self.output == other.output
    }
}

impl Eq for Panel {}

impl Panel {
    pub fn deep_clone(&self) -> Self {
        Panel {
            id: self.id.clone(),
            title: self.title.clone(),
            position: self.position.clone(),
            min_width: self.min_width,
            min_height: self.min_height,
            max_width: self.max_width,
            max_height: self.max_height,
            overflow_behavior: self.overflow_behavior.clone(),
            content: self.content.clone(),
            scroll: self.scroll,
            refresh_interval: self.refresh_interval,
            tab_order: self.tab_order.clone(),
            on_error: self.on_error.clone(),
            on_enter: self.on_enter.clone(),
            on_leave: self.on_leave.clone(),
            next_focus_id: self.next_focus_id.clone(),
            children: self.children.as_ref().map(|children| children.iter().map(|panel| panel.deep_clone()).collect()),
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
            on_refresh: self.on_refresh.clone(),
            thread: self.thread,
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

    pub fn set_output(&mut self, output: &str) {
        self.output = output.to_string();
    }

	pub fn get_parent_clone(&self, app_context: &AppContext) -> Option<Panel> {
        let layout_id = self.parent_layout_id.as_ref().expect("Parent layout ID missing");
        let app_graph = app_context.app.generate_graph(); 
        if let Some(parent) = app_graph.get_parent(layout_id, &self.id) {
            Some(parent.clone())  // Clone the result to break the lifetime dependency
        } else {
            None
        }
    }

	pub fn get_parent_layout_clone(&self, app_context: &AppContext) -> Option<Layout> {
		let layout_id = self.parent_layout_id.as_ref().expect("Parent layout ID missing");
		if let Some(parent_layout) = app_context.app.get_layout_by_id(layout_id) {
			Some(parent_layout.clone())  // Clone the result to break the lifetime dependency
		} else {
			None
		}
	}

    pub fn calc_fg_color<'a>(&self, app_context: &AppContext) -> String {
        let parent_color = self.get_parent_clone(app_context).and_then(|p| {
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

        inherit_string(self_color, parent_color.as_ref(), parent_layout_color.as_ref(), "default")
    }

    pub fn calc_bg_color<'a>(&self, app_context: &AppContext) -> String {
		let parent_color = self.get_parent_clone(app_context).and_then(|p| {
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

        inherit_string(self_color, parent_color.as_ref(), parent_layout_color.as_ref(), "default")
    }

    pub fn calc_border_color<'a>(&self, app_context: &AppContext) -> String {
        let parent_color = self.get_parent_clone(app_context).and_then(|p| {
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

        inherit_string(self_color, parent_color.as_ref(), parent_layout_color.as_ref(), "default")
    }

    pub fn calc_title_bg_color<'a>(&self, app_context: &AppContext) -> String {
        let parent_color = self.get_parent_clone(app_context).and_then(|p| {
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

        inherit_string(self_color, parent_color.as_ref(), parent_layout_color.as_ref(), "default")
    }

    pub fn calc_title_fg_color(&self, app_context: &AppContext) -> String {
        let parent_color = self.get_parent_clone(app_context).and_then(|p| {
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

        inherit_string(self_color, parent_color.as_ref(), parent_layout_color.as_ref(), "default")
    }

    pub fn calc_title_position(&self, app_context: &AppContext) -> String {
		let parent_position = self.get_parent_clone(app_context).and_then(|p| {
			p.title_position.clone()
        });
        let parent_layout_position = self.get_parent_layout_clone(app_context).and_then(|pl| pl.title_position.clone());

        inherit_string(
            self.title_position.as_ref(),
            parent_position.as_ref(),
            parent_layout_position.as_ref(),
            "center",
        )
    }

    pub fn calc_fill_char(&self, app_context: &AppContext) -> char {
        let parent_fill_char = self.get_parent_clone(app_context).and_then(|p| {
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

        inherit_char(self_char, parent_fill_char.as_ref(), parent_layout_fill_char.as_ref(), '█')
    }

    pub fn calc_border(&self, app_context: &AppContext) -> bool {
		let parent_border = self.get_parent_clone(app_context).and_then(|p| {
			p.border.clone()
        });
        let parent_layout_border = self.get_parent_layout_clone(app_context).and_then(|pl| pl.border.clone());

        inherit_bool(self.border.as_ref(), parent_border.as_ref(), parent_layout_border.as_ref(), true)
    }

    pub fn calc_overflow_behavior(&self, app_context: &AppContext) -> String {
        let parent_overflow_behavior = self.get_parent_clone(app_context).and_then(|p| {
            p.overflow_behavior.clone()
        });
        let parent_layout_overflow = self.get_parent_layout_clone(app_context).and_then(|pl| pl.overflow_behavior.clone());

        inherit_string(self.overflow_behavior.as_ref(), parent_overflow_behavior.as_ref(), parent_layout_overflow.as_ref(), "scroll")
    }

    pub fn calc_refresh_interval(&self, app_context: &AppContext) -> u64 {
        let parent_refresh_interval = self.get_parent_clone(app_context).and_then(|p| {
            p.refresh_interval.clone()
        });
        let parent_layout_interval = self.get_parent_layout_clone(app_context).and_then(|pl| pl.refresh_interval.clone());

        inherit_u64(self.refresh_interval.as_ref(), parent_refresh_interval.as_ref(), parent_layout_interval.as_ref(), 0)
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

    pub fn has_events(&self) -> bool {
        for event in &[
            &self.on_enter,
            &self.on_leave,
            &self.on_error,
            &self.on_refresh,
        ] {
            if event.is_some() {
                return true;
            }
        }
        false
    }

    pub fn has_refresh(&self) -> bool {
        self.on_refresh.is_some()
    }

    pub fn has_enter(&self) -> bool {
        self.on_enter.is_some()
    }

    pub fn has_leave(&self) -> bool {
        self.on_leave.is_some()
    }

    pub fn has_error(&self) -> bool {
        self.on_error.is_some()
    }
}

// pub struct PanelDriver {
//     panel: Panel,
//     tx: mpsc::Sender<Panel>,
//     rx: mpsc::Receiver<Panel>,
// }

// impl PanelDriver {
//     pub fn new(panel: Panel) -> Self {
//         let (tx, rx) = mpsc::channel::<Panel>();
//         PanelDriver { panel, tx, rx }
//     }

//     pub fn run(&mut self) {
//         loop {
//             if let Ok(panel) = self.rx.recv() {
//                 self.panel = panel;
//             }
//         }
//     }

//     pub fn send(&self, panel: Panel) {
//         if let Err(e) = self.tx.send(panel) {
//             log::error!("Failed to send panel to driver: {}", e);
//         }
//     }

//     pub fn start_event_thread(&self, thread_manager: &mut ThreadManager) {
//         if self.panel.has_refresh() {
//             let runnable = BoxRunnable::new(Panel::clone(self));
//             thread_manager.spawn_thread(runnable);
//         }

//         if self.panel.has_enter() || self.panel.has_leave() {
//             let runnable = BoxRunnable::new(Panel::clone(self));
//             thread_manager.spawn_thread(runnable);
//         }
//     }

//     pub fn draw(
//         &mut self,
//         screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
//         buffer: &mut ScreenBuffer,
//     ) {
//         log::debug!("Drawing panel '{}'", self.panel.id);


//         let parent_bounds = if self.panel.parent.is_none() {
//             Some(screen_bounds())
//         } else {
//             Some(self.panel.parent.unwrap().bounds())
//         };

//         // Calculate properties before borrowing self mutably
//         let bounds = self.panel.absolute_bounds(parent_bounds.as_ref());

//         let bg_color = self.panel.calc_bg_color().to_string();
//         let parent_bg_color = if self.panel.parent.is_none() {
//             "default".to_string()
//         } else {
//             self.panel.parent
//                 .unwrap()
//                 .calc_bg_color()
//                 .to_string()
//         };
//         let fg_color = self.panel.calc_fg_color().to_string();
//         let title_bg_color = self.panel.calc_title_bg_color().to_string();
//         let title_fg_color = self.panel.calc_title_fg_color().to_string();
//         let border = self.panel.calc_border();
//         let border_color = self.panel.calc_border_color().to_string();
//         let fill_char = self.panel.calc_fill_char();

//         // Draw fill
//         fill_panel(&bounds, border, &bg_color, fill_char, screen, buffer);

//         let mut content = self.panel.content.as_deref();
//         // check output is not null or empty
//         if !self.panel.output.is_empty() {
//             content = Some(&self.panel.output);
//         }

//         log::info!(
//             "Drawing panel '{}' with horizontal scroll '{}', vertical scroll '{}'",
//             self.panel.id,
//             self.panel.current_horizontal_scroll(),
//             self.panel.current_vertical_scroll()
//         );

//         // Draw border with title
//         draw_panel(
//             &bounds,
//             &border_color,
//             Some(&bg_color),
//             Some(&parent_bg_color),
//             self.panel.title.as_deref(),
//             &title_fg_color,
//             &title_bg_color,
//             &self.panel.calc_title_position(),
//             content,
//             &fg_color,
//             &self.panel.calc_overflow_behavior(),
//             self.panel.current_horizontal_scroll(),
//             self.panel.current_vertical_scroll(),
//             screen,
//             buffer,
//         );

//         // Draw children
//         if let Some(children) = &mut self.panel.children {
//             for child in children {
//                 child.0.lock().unwrap().draw(screen, buffer);
//             }
//         }
//         log::debug!("Finished drawing panel '{}'", self.panel.id);
//     }

//     pub fn execute_refresh(&mut self) {
//         log::info!("Executing refresh for panel '{}'", self.panel.id);
//         if let Some(commands) = &self.panel.on_refresh {
//             let output = commands
//                 .iter()
//                 .map(|cmd| {
//                     let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
//                     String::from_utf8_lossy(&output.stdout).to_string()
//                 })
//                 .collect::<Vec<_>>()
//                 .join("\n");

//             self.panel.set_output(&output);
//         }
//     }

//     pub fn execute_enter(&mut self) {
//         if let Some(commands) = &self.panel.on_enter {
//             let output = commands
//                 .iter()
//                 .map(|cmd| {
//                     let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
//                     String::from_utf8_lossy(&output.stdout).to_string()
//                 })
//                 .collect::<Vec<_>>()
//                 .join("\n");

//             self.panel.set_output(&output);
//         }
//     }

//     pub fn execute_leave(&mut self) {
//         if let Some(commands) = &self.panel.on_leave {
//             let output = commands
//                 .iter()
//                 .map(|cmd| {
//                     let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
//                     String::from_utf8_lossy(&output.stdout).to_string()
//                 })
//                 .collect::<Vec<_>>()
//                 .join("\n");

//             self.panel.set_output(&output);
//         }
//     }

//     pub fn execute_error(&mut self) {
//         if let Some(commands) = &self.panel.on_error {
//             let output = commands
//                 .iter()
//                 .map(|cmd| {
//                     let output = Command::new("sh").arg("-c").arg(cmd).output().unwrap();
//                     String::from_utf8_lossy(&output.stdout).to_string()
//                 })
//                 .collect::<Vec<_>>()
//                 .join("\n");

//             self.panel.set_output(&output);
//         }
//     }
// }
