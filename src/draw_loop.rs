use crate::model::app;
use crate::thread_manager::Runnable;
use crate::{
    apply_buffer, apply_buffer_if_changed, calculate_bounds_map, execute_commands, handle_keypress, screen_height, screen_width, Anchor, AppContext, AppGraph, Bounds, Layout, Panel, ScreenBuffer
};
use std::collections::HashMap;
use std::io::stdout;
use std::io::{Stdout, Write as IoWrite};
use std::sync::{mpsc, Mutex};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use crate::thread_manager::*;

use crate::utils::{render_panel, fill_panel, screen_bounds};
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
        let mut state_unwrapped = state.deep_clone();

        if global_screen.is_none() {
            *global_screen = Some(AlternateScreen::from(stdout().into_raw_mode().unwrap()));
            *global_buffer = Some(ScreenBuffer::new(screen_width(), screen_height()));
        }

        if let (Some(ref mut screen), Some(ref mut buffer)) =
            (&mut *global_screen, &mut *global_buffer)
        {
            let mut new_buffer = ScreenBuffer::new(screen_width(), screen_height());
            draw_app(&mut state_unwrapped, &mut new_buffer);
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
                    Message::Exit => should_continue = false,
                    Message::Die => should_continue = false,
                    Message::NextPanel() => {
                        let mut active_layout = state_unwrapped
                            .app
                            .get_active_layout_mut()
                            .expect("No active layout found!");

                        // First, collect the IDs of currently selected panels before changing the selection.
                        let unselected_panel_ids: Vec<String> = active_layout
                            .get_selected_panels()
                            .iter()
                            .map(|panel| panel.id.clone())
                            .collect();

                        // Now perform the mutation that changes the panel selection.
                        active_layout.select_next_panel();

                        // After mutation, get the newly selected panels' IDs.
                        let selected_panel_ids: Vec<String> = active_layout
                            .get_selected_panels()
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
                    Message::PreviousPanel() => {
                        let mut active_layout = state_unwrapped
                            .app
                            .get_active_layout_mut()
                            .expect("No active layout found!");

                        // First, collect the IDs of currently selected panels before changing the selection.
                        let unselected_panel_ids: Vec<String> = active_layout
                            .get_selected_panels()
                            .iter()
                            .map(|panel| panel.id.clone())
                            .collect();

                        // Now perform the mutation that changes the panel selection.
                        active_layout.select_previous_panel();

                        // After mutation, get the newly selected panels' IDs.
                        let selected_panel_ids: Vec<String> = active_layout
                            .get_selected_panels()
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
                    Message::ScrollPanelDown() => {
                        let selected_panels = state_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = state_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.scroll_down(Some(1.0));
                                inner.update_app_context(state_unwrapped.deep_clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelUp() => {
                        let selected_panels = state_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = state_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.scroll_up(Some(1.0));
                                inner.update_app_context(state_unwrapped.deep_clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelLeft() => {
                        let selected_panels = state_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = state_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.scroll_left(Some(1.0));
                                inner.update_app_context(state_unwrapped.deep_clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelRight() => {
                        let selected_panels = state_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = state_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.scroll_right(Some(1.0));
                                inner.update_app_context(state_unwrapped.deep_clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::RedrawPanel(panel_id) => {
                        if let Some(mut found_panel) =
                            state_unwrapped.app.get_panel_by_id_mut(&panel_id).cloned()
                        {
                            new_buffer = buffer.clone();

                            // Clone the parent layout to avoid mutable borrow conflicts
                            if let Some(parent_layout) =
                                found_panel.get_parent_layout_clone(&mut state_unwrapped)
                            {
                                draw_panel(
                                    &mut state_unwrapped,
                                    &parent_layout,
                                    &mut found_panel,
                                    &mut new_buffer,
                                );
                                apply_buffer_if_changed(buffer, &new_buffer, screen);
                                *buffer = new_buffer;
                            }
                        }
                    }
                    Message::RedrawApp => {
						state_unwrapped.recalculate_bounds();
                        write!(screen, "{}", termion::clear::All).unwrap();
                        new_buffer = ScreenBuffer::new(screen_width(), screen_height());
                        draw_app(&mut state_unwrapped, &mut new_buffer);
                        apply_buffer(&mut new_buffer, screen);
                        *buffer = new_buffer;
                    }
					Message::Resize => {
						state_unwrapped.recalculate_bounds();
                        write!(screen, "{}", termion::clear::All).unwrap();
                        new_buffer = ScreenBuffer::new(screen_width(), screen_height());
                        draw_app(&mut state_unwrapped, &mut new_buffer);
                        apply_buffer(&mut new_buffer, screen);
                        *buffer = new_buffer;
                    }
                    Message::PanelOutputUpdate(panel_id, output) => {
                        let panel = state_unwrapped.app.get_panel_by_id_mut(&panel_id);
                        if let Some(found_panel) = panel {
                            found_panel.content = Some(output.clone());
                            inner.update_app_context(state_unwrapped.deep_clone());
                            inner.send_message(Message::RedrawPanel(panel_id.clone()));
                        }
                    }
                    Message::KeyPress(pressed_key) => {
                        let active_layout = state_unwrapped.app.get_active_layout().unwrap();

                        let selected_panel_ids: Vec<String> = active_layout
                            .get_selected_panels()
                            .into_iter()
                            .filter(|p| p.on_keypress.is_some())
                            .map(|p| p.id.clone())
                            .collect();

                        for panel_id in selected_panel_ids {
                            let panel = state_unwrapped.app.get_panel_by_id(&panel_id).unwrap();

                            if let Some(actions) =
                                handle_keypress(&pressed_key, &panel.on_keypress.clone().unwrap())
                            {
                                // Perform mutable operations outside the loop that borrows immutably
                                let panel_mut =
                                    state_unwrapped.app.get_panel_by_id_mut(&panel_id).unwrap();
                                let new_output = execute_commands(&actions);

                                panel_mut.content = Some(new_output.clone());
                                inner.update_app_context(state_unwrapped.deep_clone());
                                inner.send_message(Message::RedrawPanel(panel_id.clone()));
                            }
                        }
                    }
                    _ => {}
                }
            }
            // Ensure the loop continues by sleeping briefly
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        should_continue
    }
);

pub fn draw_app(app_context: &mut AppContext, buffer: &mut ScreenBuffer) {
    let mut active_layout = app_context
        .app
        .get_active_layout()
        .expect("No active layout found!")
        .clone();

    draw_layout(app_context, &mut active_layout, buffer);
}

pub fn draw_layout(app_context: &mut AppContext, layout: &mut Layout, buffer: &mut ScreenBuffer) {
	let cloned_layout = layout.clone();
    let bg_color = cloned_layout.bg_color.unwrap_or("black".to_string());
    let fill_char = cloned_layout.fill_char.unwrap_or(' ');
	
    // Set the background for the layout
    fill_panel(
        &screen_bounds(),
        false,
        &bg_color,
        fill_char,
        buffer,
    );

	let layout_children = layout.children.clone();

    for mut panel in layout_children {
        draw_panel(app_context, &layout, &mut panel, buffer);
    }
}

pub fn draw_panel(
    app_context: &mut AppContext,
    layout: &Layout,
    panel: &mut Panel,
    buffer: &mut ScreenBuffer,
) {
    let app_graph = app_context.app.generate_graph();

    let panel_parent = app_graph.get_parent(&layout.id, &panel.id);

	let calculated_bounds=app_context.calculated_bounds.clone().expect("Calculated bounds should not be none.");

	let layout_calculated_bounds= calculated_bounds.get(&layout.id);

	let mut panel_calculated_bounds = None;
	match layout_calculated_bounds {
        Some(value) => panel_calculated_bounds= value.get(&panel.id),
        None => println!("Calculated bounds for layout {} not found", &layout.id),
    }

	match panel_calculated_bounds {
		Some(value) => {let bg_color = panel.calc_bg_color(app_context).to_string();
			let parent_bg_color = if panel_parent.is_none() {
				layout.bg_color.clone().unwrap_or("black".to_string())
			} else {
				panel_parent.unwrap().calc_bg_color(app_context).to_string()
			};
			let fg_color = panel.calc_fg_color(app_context).to_string();
		
			let title_bg_color = panel.calc_title_bg_color(app_context).to_string();
			let title_fg_color = panel.calc_title_fg_color(app_context).to_string();
			let border = panel.calc_border(app_context);
			let border_color = panel.calc_border_color(app_context).to_string();
			let fill_char = panel.calc_fill_char(app_context);
		
			// Draw fill
			fill_panel(&value, border, &bg_color, fill_char, buffer);
		
			let mut content = panel.content.as_deref();
			// check output is not null or empty
			if !panel.output.is_empty() {
				content = Some(&panel.output);
			}

			render_panel(
				&value,
				&border_color,
				&bg_color,
				&parent_bg_color,
				panel.title.as_deref(),
				&title_fg_color,
				&title_bg_color,
				&panel.calc_title_position(app_context),
				content,
				&fg_color,
				&panel.calc_overflow_behavior(app_context),
				Some(&panel.calc_border(app_context)),
				panel.current_horizontal_scroll(),
				panel.current_vertical_scroll(),
				buffer,
			);
		
			// Draw children
			if let Some(children) = &mut panel.children {
				for child in children.iter_mut() {
					draw_panel(app_context, layout, child, buffer);
				}
			}},
		None => println!("Calculated bounds for panel {} not found", &panel.id),
	}
}
