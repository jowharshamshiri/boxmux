use crate::model::app::App;
use crate::thread_manager::{self, Runnable};
use crate::{
    apply_buffer_if_changed, get_bg_color, get_fg_color, screen_height, screen_width, AppContext, Bounds, Cell, Layout, Panel, ScreenBuffer
};
use log::{error, info};
use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
use simplelog::*;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::{stdin, stdout, Read};
use std::io::{Stdout, Write as IoWrite};
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use serde::{
    de::MapAccess, de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

use crate::thread_manager::*;

use std::sync::atomic::{AtomicBool, Ordering};

use crate::utils::{draw_panel as util_draw_panel, fill_panel, screen_bounds};
use uuid::Uuid;

lazy_static! {
    static ref GLOBAL_SCREEN: Mutex<Option<AlternateScreen<RawTerminal<Stdout>>>> =
        Mutex::new(None);
    static ref GLOBAL_BUFFER: Mutex<Option<ScreenBuffer>> = Mutex::new(None);
}

create_runnable!(
    DrawLoop,
    |inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>| -> bool {
        let mut global_screen = GLOBAL_SCREEN.lock().unwrap();
        let mut global_buffer = GLOBAL_BUFFER.lock().unwrap();

        if global_screen.is_none() {
            *global_screen = Some(AlternateScreen::from(stdout().into_raw_mode().unwrap()));
            *global_buffer = Some(ScreenBuffer::new(screen_width(), screen_height()));
        }

        if let (Some(ref mut screen), Some(ref mut buffer)) =
            (&mut *global_screen, &mut *global_buffer)
        {
			let mut new_buffer = ScreenBuffer::new(screen_width(), screen_height());
            draw_app(&state, &mut new_buffer);
            apply_buffer_if_changed(buffer, &new_buffer, screen);
			*buffer = new_buffer;
        }

        true
    },
    |inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>| -> bool {
        let mut global_screen = GLOBAL_SCREEN.lock().unwrap();
        let mut global_buffer = GLOBAL_BUFFER.lock().unwrap();
        let mut should_continue = true;

        if let (Some(ref mut screen), Some(ref mut buffer)) =
            (&mut *global_screen, &mut *global_buffer)
        {
			let mut new_buffer = ScreenBuffer::new(screen_width(), screen_height());
            let mut state_unwrapped = state.deep_clone();

            for message in &messages {
                match message {
                    Message::PanelEventRefresh(_) => {
                        log::info!("PanelEventRefresh");
                    }
                    Message::PanelEventEnter(_) => {
                        log::info!("PanelEventEnter");
                    }
                    Message::PanelEventLeave(_) => {
                        log::info!("PanelEventLeave");
                    }
                    Message::PanelEventError(_) => {
                        log::info!("PanelEventError");
                    }
                    Message::Exit => should_continue = false,
                    Message::Die => should_continue = false,
                    Message::Resize => {
                        write!(screen, "{}", termion::clear::All).unwrap();
                        new_buffer = ScreenBuffer::new(screen_width(), screen_height());
                        draw_app(&state_unwrapped, &mut new_buffer);
                        apply_buffer_if_changed(buffer, &new_buffer, screen);
						*buffer = new_buffer;
                    }
                    Message::NextPanel(layout_id) => {
						let mut active_layout = state_unwrapped.app.get_active_layout_mut().expect("No active layout found!");
					
						// First, collect the IDs of currently selected panels before changing the selection.
						let unselected_panel_ids: Vec<String> = active_layout.get_selected_panels()
																			.iter()
																			.map(|panel| panel.id.clone())
																			.collect();
					
						// Now perform the mutation that changes the panel selection.
						active_layout.select_next_panel();
					
						// After mutation, get the newly selected panels' IDs.
						let selected_panel_ids: Vec<String> = active_layout.get_selected_panels()
							.iter()
							.map(|panel| panel.id.clone())
							.collect();
					
						// Update the application context and issue redraw commands based on the collected IDs.
						inner.update_app_context(state_unwrapped.deep_clone());
						for panel_id in unselected_panel_ids {
							inner.send_message(Message::RedrawPanel(panel_id));
						}
						for panel_id in selected_panel_ids {
							inner.send_message(Message::RedrawPanel(panel_id));
						}
					}
                    Message::PreviousPanel(layout_id) => {
						let mut active_layout = state_unwrapped.app.get_active_layout_mut().expect("No active layout found!");
					
						// First, collect the IDs of currently selected panels before changing the selection.
						let unselected_panel_ids: Vec<String> = active_layout.get_selected_panels()
																			.iter()
																			.map(|panel| panel.id.clone())
																			.collect();
					
						// Now perform the mutation that changes the panel selection.
						active_layout.select_previous_panel();
					
						// After mutation, get the newly selected panels' IDs.
						let selected_panel_ids: Vec<String> = active_layout.get_selected_panels()
							.iter()
							.map(|panel| panel.id.clone())
							.collect();
					
						// Update the application context and issue redraw commands based on the collected IDs.
						inner.update_app_context(state_unwrapped.deep_clone());
						for panel_id in unselected_panel_ids {
							inner.send_message(Message::RedrawPanel(panel_id));
						}
						for panel_id in selected_panel_ids {
							inner.send_message(Message::RedrawPanel(panel_id));
						}
                    }
					Message::ScrollPanelDown(panel_id) => {
						let panel = state_unwrapped.app.get_panel_by_id_mut(&panel_id);
						if let Some(found_panel) = panel {
							found_panel.scroll_down(Some(1.0));
							inner.update_app_context(state_unwrapped.deep_clone());
							inner.send_message(Message::RedrawPanel(panel_id.clone()));
						}
					},
					Message::ScrollPanelUp(panel_id) => {
						let panel = state_unwrapped.app.get_panel_by_id_mut(&panel_id);
						if let Some(found_panel) = panel {
							found_panel.scroll_up(Some(1.0));
							inner.update_app_context(state_unwrapped.deep_clone());
							inner.send_message(Message::RedrawPanel(panel_id.clone()));
						}
					},
					Message::ScrollPanelLeft(panel_id) => {
						let panel = state_unwrapped.app.get_panel_by_id_mut(&panel_id);
						if let Some(found_panel) = panel {
							found_panel.scroll_left(Some(1.0));
							inner.update_app_context(state_unwrapped.deep_clone());
							inner.send_message(Message::RedrawPanel(panel_id.clone()));
						}
					},
					Message::ScrollPanelRight(panel_id) => {
						let panel = state_unwrapped.app.get_panel_by_id_mut(&panel_id);
						if let Some(found_panel) = panel {
							found_panel.scroll_right(Some(1.0));
							inner.update_app_context(state_unwrapped.deep_clone());
							inner.send_message(Message::RedrawPanel(panel_id.clone()));
						}
					},
                    Message::RedrawPanel(panel_id) => {
                        let panel = state_unwrapped.app.get_panel_by_id(&panel_id);
                        if let Some(found_panel) = panel {
                            new_buffer = buffer.clone();
                            draw_panel(
                                &state_unwrapped,
                                &found_panel.get_parent_layout_clone(&state_unwrapped).unwrap(),
                                &found_panel,
                                &mut new_buffer,
                            );
                            apply_buffer_if_changed(buffer, &new_buffer, screen);
							*buffer = new_buffer;
                        }
                    }
                    Message::RedrawApp => {
                        new_buffer = ScreenBuffer::new(screen_width(), screen_height());
                        draw_app(&state_unwrapped, &mut new_buffer);
                        apply_buffer_if_changed(buffer, &new_buffer, screen);
						*buffer = new_buffer;
                    }
                    Message::UpdatePanel(panel_id) => todo!(),
                }
            }
            // Ensure the loop continues by sleeping briefly
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        should_continue
    }
);

pub fn draw_app(app_context: &AppContext, buffer: &mut ScreenBuffer) {
    let app = &app_context.app;
	let active_layout = app.get_active_layout().expect("No active layout found!");
	
	draw_layout(app_context, active_layout, buffer);
}

pub fn draw_layout(app_context: &AppContext, layout: &Layout, buffer: &mut ScreenBuffer) {
	let bg_color = app_context.app.get_active_layout().unwrap().bg_color.clone().unwrap_or_else(|| get_bg_color("default"));
	let fill_char = app_context.app.get_active_layout().unwrap().fill_char.unwrap_or(' ');
	
	//background
	fill_panel(&screen_bounds(), false, bg_color.as_str(),fill_char, buffer);

    for panel in &layout.children {
        draw_panel(app_context, layout, panel, buffer);
    }
}

pub fn draw_panel(
    app_context: &AppContext,
    layout: &Layout,
    panel: &Panel,
    buffer: &mut ScreenBuffer,
) {
	let app_graph = app_context.app.generate_graph();

    let panel_parent = app_graph.get_parent(&layout.id, &panel.id);

    let parent_bounds = if panel_parent.is_none() {
        Some(screen_bounds())
    } else {
        Some(panel_parent.unwrap().bounds())
    };

    // Calculate properties before borrowing self mutably
    let bounds = panel.absolute_bounds(parent_bounds.as_ref());

    let mut bg_color = panel.calc_bg_color(app_context).to_string();
    let parent_bg_color = if panel_parent.is_none() {
        "default".to_string()
    } else {
        panel_parent.unwrap().calc_bg_color(app_context).to_string()
    };
    let mut fg_color = panel.calc_fg_color(app_context).to_string();
	
    let title_bg_color = panel.calc_title_bg_color(app_context).to_string();
    let title_fg_color = panel.calc_title_fg_color(app_context).to_string();
    let border = panel.calc_border(app_context);
    let border_color = panel.calc_border_color(app_context).to_string();
    let fill_char = panel.calc_fill_char(app_context);

	// if panel.selected.unwrap_or(false) {
	// 	log::info!("Panel '{}' is selected", panel.id);
	// 	bg_color = "red".to_owned();
	// }
    // Draw fill
    fill_panel(&bounds, border, &bg_color, fill_char, buffer);

    let mut content = panel.content.as_deref();
    // check output is not null or empty
    if !panel.output.is_empty() {
        content = Some(&panel.output);
    }

    log::info!(
        "Drawing panel '{}' with horizontal scroll '{}', vertical scroll '{}'",
        panel.id,
        panel.current_horizontal_scroll(),
        panel.current_vertical_scroll()
    );

    // Draw border with title
    util_draw_panel(
        &bounds,
        &border_color,
        Some(&bg_color),
        Some(&parent_bg_color),
        panel.title.as_deref(),
        &title_fg_color,
        &title_bg_color,
        &panel.calc_title_position(app_context),
        content,
        &fg_color,
        &panel.calc_overflow_behavior(app_context),
        panel.current_horizontal_scroll(),
        panel.current_vertical_scroll(),
        buffer,
    );

    // Draw children
    if let Some(children) = &panel.children {
        for child in children {
            draw_panel(app_context, layout, child, buffer);
        }
    }
    log::debug!("Finished drawing panel '{}'", panel.id);
}
