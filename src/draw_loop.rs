use crate::draw_utils::{draw_app, draw_panel};
use crate::thread_manager::Runnable;
use crate::{
    apply_buffer, apply_buffer_if_changed, execute_commands, handle_keypress, AppContext,
    ScreenBuffer,
};
use std::io::stdout;
use std::io::{Stdout, Write as IoWrite};
use std::sync::{mpsc, Mutex};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use crate::thread_manager::*;

use uuid::Uuid;

lazy_static! {
    static ref GLOBAL_SCREEN: Mutex<Option<AlternateScreen<RawTerminal<Stdout>>>> =
        Mutex::new(None);
    static ref GLOBAL_BUFFER: Mutex<Option<ScreenBuffer>> = Mutex::new(None);
}

create_runnable!(
    DrawLoop,
    |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool {
        let mut global_screen = GLOBAL_SCREEN.lock().unwrap();
        let mut global_buffer = GLOBAL_BUFFER.lock().unwrap();
        let mut state_unwrapped = app_context.deep_clone();
        let (adjusted_bounds, app_graph) = state_unwrapped
            .app
            .get_adjusted_bounds_and_app_graph(Some(true));

        if global_screen.is_none() {
            *global_screen = Some(AlternateScreen::from(stdout().into_raw_mode().unwrap()));
            *global_buffer = Some(ScreenBuffer::new());
        }

        if let (Some(ref mut screen), Some(ref mut buffer)) =
            (&mut *global_screen, &mut *global_buffer)
        {
            let mut new_buffer = ScreenBuffer::new();
            draw_app(
                &state_unwrapped,
                &app_graph,
                &adjusted_bounds,
                &mut new_buffer,
            );
            apply_buffer_if_changed(buffer, &new_buffer, screen);
            *buffer = new_buffer;
        }

        true
    },
    |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool {
        let mut global_screen = GLOBAL_SCREEN.lock().unwrap();
        let mut global_buffer = GLOBAL_BUFFER.lock().unwrap();
        let mut should_continue = true;

        if let (Some(ref mut screen), Some(ref mut buffer)) =
            (&mut *global_screen, &mut *global_buffer)
        {
            let mut new_buffer = ScreenBuffer::new();
            let mut state_unwrapped = app_context.deep_clone();
            let (adjusted_bounds, app_graph) = state_unwrapped
                .app
                .get_adjusted_bounds_and_app_graph(Some(true));

            for message in &messages {
                match message {
                    Message::PanelEventRefresh(_) => {
                        log::info!("PanelEventRefresh");
                    }
                    Message::Exit => should_continue = false,
                    Message::Terminate => should_continue = false,
                    Message::NextPanel() => {
                        let active_layout = state_unwrapped
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
                        let active_layout = state_unwrapped
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
                                    &state_unwrapped,
                                    &app_graph,
                                    &adjusted_bounds,
                                    &parent_layout,
                                    &mut found_panel,
                                    &mut new_buffer,
                                );
                                apply_buffer_if_changed(buffer, &new_buffer, screen);
                                *buffer = new_buffer;
                            }
                        }
                    }
                    Message::RedrawApp | Message::Resize => {
                        write!(screen, "{}", termion::clear::All).unwrap();
                        let mut new_buffer = ScreenBuffer::new();
                        draw_app(
                            &state_unwrapped,
                            &app_graph,
                            &adjusted_bounds,
                            &mut new_buffer,
                        );
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
            std::thread::sleep(std::time::Duration::from_millis(
                app_context.config.frame_delay,
            ));
        }

        should_continue
    }
);
