use crate::draw_utils::{draw_app, draw_panel};
use crate::thread_manager::Runnable;
use crate::{
    apply_buffer, apply_buffer_if_changed, handle_keypress, run_script, run_socket_function,
    AppContext, ScreenBuffer, SocketFunction,
};
use crate::{thread_manager::*, FieldUpdate};
use clap::App;
use serde_json;
use std::io::stdout;
use std::io::{Stdout, Write as IoWrite};
use std::sync::{mpsc, Mutex};
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

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
        let mut app_context_unwrapped = app_context.clone();
        let (adjusted_bounds, app_graph) = app_context_unwrapped
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
                &app_context_unwrapped,
                &app_graph,
                &adjusted_bounds,
                &mut new_buffer,
            );
            apply_buffer_if_changed(buffer, &new_buffer, screen);
            *buffer = new_buffer;
        }

        true
    },
    |inner: &mut RunnableImpl,
     app_context: AppContext,
     messages: Vec<Message>|
     -> (bool, AppContext) {
        let mut global_screen = GLOBAL_SCREEN.lock().unwrap();
        let mut global_buffer = GLOBAL_BUFFER.lock().unwrap();
        let mut should_continue = true;

        if let (Some(ref mut screen), Some(ref mut buffer)) =
            (&mut *global_screen, &mut *global_buffer)
        {
            let mut new_buffer = ScreenBuffer::new();
            let mut app_context_unwrapped = app_context.clone();
            let (adjusted_bounds, app_graph) = app_context_unwrapped
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
                        let active_layout = app_context_unwrapped
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
                        inner.update_app_context(app_context_unwrapped.clone());
                        for panel_id in unselected_panel_ids {
                            inner.send_message(Message::RedrawPanel(panel_id));
                        }
                        for panel_id in selected_panel_ids {
                            inner.send_message(Message::RedrawPanel(panel_id));
                        }
                    }
                    Message::PreviousPanel() => {
                        let active_layout = app_context_unwrapped
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
                        inner.update_app_context(app_context_unwrapped.clone());
                        for panel_id in unselected_panel_ids {
                            inner.send_message(Message::RedrawPanel(panel_id));
                        }
                        for panel_id in selected_panel_ids {
                            inner.send_message(Message::RedrawPanel(panel_id));
                        }
                    }
                    Message::ScrollPanelDown() => {
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                if found_panel.choices.is_some() {
                                    //select first or next choice
                                    let choices = found_panel.choices.as_mut().unwrap();
                                    let selected_choice = choices.iter().position(|c| c.selected);
                                    let selected_choice_unwrapped = match selected_choice {
                                        Some(selected_choice) => selected_choice,
                                        None => 0,
                                    };
                                    let new_selected_choice =
                                        if selected_choice_unwrapped + 1 < choices.len() {
                                            selected_choice_unwrapped + 1
                                        } else {
                                            0
                                        };
                                    for (i, choice) in choices.iter_mut().enumerate() {
                                        if i == new_selected_choice {
                                            choice.selected = true;
                                        } else {
                                            choice.selected = false;
                                        }
                                    }
                                } else {
                                    found_panel.scroll_down(Some(1.0));
                                }

                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelUp() => {
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                if found_panel.choices.is_some() {
                                    //select first or next choice
                                    let choices = found_panel.choices.as_mut().unwrap();
                                    let selected_choice = choices.iter().position(|c| c.selected);
                                    let selected_choice_unwrapped = match selected_choice {
                                        Some(selected_choice) => selected_choice,
                                        None => 0,
                                    };
                                    let new_selected_choice = if selected_choice_unwrapped > 0 {
                                        selected_choice_unwrapped - 1
                                    } else {
                                        choices.len() - 1
                                    };
                                    for (i, choice) in choices.iter_mut().enumerate() {
                                        if i == new_selected_choice {
                                            choice.selected = true;
                                        } else {
                                            choice.selected = false;
                                        }
                                    }
                                } else {
                                    found_panel.scroll_up(Some(1.0));
                                }
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelLeft() => {
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.scroll_left(Some(1.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelRight() => {
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.scroll_right(Some(1.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::RedrawPanel(panel_id) => {
                        if let Some(mut found_panel) = app_context_unwrapped
                            .app
                            .get_panel_by_id_mut(&panel_id)
                            .cloned()
                        {
                            new_buffer = buffer.clone();

                            // Clone the parent layout to avoid mutable borrow conflicts
                            if let Some(parent_layout) =
                                found_panel.get_parent_layout_clone(&mut app_context_unwrapped)
                            {
                                draw_panel(
                                    &app_context_unwrapped,
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
                            &app_context_unwrapped,
                            &app_graph,
                            &adjusted_bounds,
                            &mut new_buffer,
                        );
                        apply_buffer(&mut new_buffer, screen);
                        *buffer = new_buffer;
                    }
                    Message::PanelOutputUpdate(panel_id, output) => {
                        update_panel_content(inner, &mut app_context_unwrapped, panel_id, output);
                    }
                    Message::ExternalMessage(json_message) => {
                        let message_result: Result<SocketFunction, _> =
                            serde_json::from_str(json_message.trim());
                        match message_result {
                            Ok(socket_function) => {
                                match run_socket_function(socket_function, &app_context_unwrapped) {
                                    Ok((new_app_context, messages)) => {
                                        app_context_unwrapped = new_app_context;
                                        for message in messages {
                                            inner.send_message(message);
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Error running socket function: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Error reading socket message: {}", e);
                            }
                        }
                    }
                    Message::KeyPress(pressed_key) => {
                        let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();

                        let selected_panel_ids: Vec<String> = active_layout
                            .get_selected_panels()
                            .into_iter()
                            .filter(|p| p.on_keypress.is_some())
                            .map(|p| p.id.clone())
                            .collect();

                        for panel_id in selected_panel_ids {
                            let mut app_context_unwrapped_cloned = app_context_unwrapped.clone();
                            let panel = app_context_unwrapped
                                .app
                                .get_panel_by_id(&panel_id)
                                .unwrap();

                            let actions =
                                handle_keypress(&pressed_key, &panel.on_keypress.clone().unwrap());
                            if !actions.is_some()
                                || (panel.choices.is_some() && pressed_key == "Enter")
                            {
                                if panel.choices.is_some() && pressed_key == "Enter" {
                                    let libs = app_context_unwrapped.app.libs.clone();
                                    let choices = panel.choices.as_ref().unwrap();
                                    let selected_choice = choices.iter().position(|c| c.selected);
                                    if let Some(selected_choice_unwrapped) = selected_choice {
                                        let selected_choice = &choices[selected_choice_unwrapped];
                                        if let Some(script) = &selected_choice.script {
                                            let new_output = run_script(libs, script);
                                            if let Ok(new_output) = new_output {
                                                if selected_choice.redirect_output.is_some() {
                                                    update_panel_content(
                                                        inner,
                                                        &mut app_context_unwrapped_cloned,
                                                        selected_choice
                                                            .redirect_output
                                                            .as_ref()
                                                            .unwrap(),
                                                        &new_output,
                                                    )
                                                } else {
                                                    update_panel_content(
                                                        inner,
                                                        &mut app_context_unwrapped_cloned,
                                                        panel_id.as_ref(),
                                                        &new_output,
                                                    )
                                                }
                                            } else {
                                                inner.send_message(Message::PanelOutputUpdate(
                                                    panel_id.clone(),
                                                    "Error running script".to_string(),
                                                ));
                                            }
                                        }
                                    }
                                }
                                if let Some(actions_unwrapped) = actions {
                                    let libs = app_context_unwrapped.app.libs.clone();
                                    // Perform mutable operations outside the loop that borrows immutably
                                    let panel_mut = app_context_unwrapped
                                        .app
                                        .get_panel_by_id_mut(&panel_id)
                                        .unwrap();
                                    let new_output = run_script(libs, &actions_unwrapped);

                                    if let Ok(new_output) = new_output {
                                        update_panel_content(
                                            inner,
                                            &mut app_context_unwrapped_cloned,
                                            panel_mut.id.as_ref(),
                                            &new_output,
                                        )
                                    } else {
                                        inner.send_message(Message::PanelOutputUpdate(
                                            panel_id.clone(),
                                            "Error running script".to_string(),
                                        ));
                                    }
                                }
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
            return (should_continue, app_context_unwrapped);
        }

        (should_continue, app_context)
    }
);

pub fn update_panel_content(
    inner: &mut RunnableImpl,
    app_context_unwrapped: &mut AppContext,
    panel_id: &str,
    output: &str,
) {
    let mut app_context_unwrapped_cloned = app_context_unwrapped.clone();
    let panel = app_context_unwrapped.app.get_panel_by_id_mut(panel_id);

    log::info!(
        "Updating panel content: {}, redirection: {:?}",
        panel_id,
        panel.as_ref().unwrap().redirect_output
    );
    if let Some(found_panel) = panel {
        if found_panel.redirect_output.is_some()
            && found_panel.redirect_output.as_ref().unwrap() != panel_id
        {
            log::info!(
                "Redirecting output from panel {} to panel: {}",
                panel_id,
                found_panel.redirect_output.as_ref().unwrap()
            );
            update_panel_content(
                inner,
                &mut app_context_unwrapped_cloned,
                found_panel.redirect_output.as_ref().unwrap(),
                output,
            );
        } else {
            log::info!("Updating panel {} content with no redirection.", panel_id);
            found_panel.content = Some(output.to_string());
            inner.update_app_context(app_context_unwrapped.clone());
            inner.send_message(Message::RedrawPanel(panel_id.to_string()));
        }
    }
}
