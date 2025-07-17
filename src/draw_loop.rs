use crate::choice_threads::{ChoiceResult, ChoiceResultPacket, ChoiceThreadManager, JobStatus};
use crate::draw_utils::{draw_app, draw_panel};
use crate::thread_manager::Runnable;
use crate::{
    apply_buffer, apply_buffer_if_changed, handle_keypress, run_script, run_socket_function,
    AppContext, Panel, ScreenBuffer, SocketFunction,
};
use crate::{thread_manager::*, FieldUpdate};
use crossbeam_channel::Sender;
use serde_json;
use std::io::stdout;
use std::io::Stdout;
use std::sync::{mpsc, Mutex};
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};

use uuid::Uuid;

lazy_static! {
    static ref GLOBAL_SCREEN: Mutex<Option<Stdout>> = Mutex::new(None);
    static ref GLOBAL_BUFFER: Mutex<Option<ScreenBuffer>> = Mutex::new(None);
    static ref POOL: ChoiceThreadManager = ChoiceThreadManager::new(4);
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
            let mut stdout = stdout();
            enable_raw_mode().unwrap();
            stdout.execute(EnterAlternateScreen).unwrap();
            *global_screen = Some(stdout);
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
            let mut choice_ids_now_waiting: Vec<(String, String)> = vec![];

            for message in &messages {
                match message {
                    Message::PanelEventRefresh(_) => {
                        log::trace!("PanelEventRefresh");
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
                                    let selected_choice_unwrapped =
                                        selected_choice.unwrap_or_default();
                                    let new_selected_choice =
                                        if selected_choice_unwrapped + 1 < choices.len() {
                                            selected_choice_unwrapped + 1
                                        } else {
                                            0
                                        };
                                    for (i, choice) in choices.iter_mut().enumerate() {
                                        choice.selected = i == new_selected_choice;
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
                                    let selected_choice_unwrapped =
                                        selected_choice.unwrap_or_default();
                                    let new_selected_choice = if selected_choice_unwrapped > 0 {
                                        selected_choice_unwrapped - 1
                                    } else {
                                        choices.len() - 1
                                    };
                                    for (i, choice) in choices.iter_mut().enumerate() {
                                        choice.selected = i == new_selected_choice;
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
                            .get_panel_by_id_mut(panel_id)
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
                        screen.execute(crossterm::terminal::Clear(crossterm::terminal::ClearType::All)).unwrap();
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
                    Message::PanelOutputUpdate(panel_id, success, output) => {
                        let mut app_context_unwrapped_cloned = app_context_unwrapped.clone();
                        let panel = app_context_unwrapped.app.get_panel_by_id(panel_id).unwrap();
                        update_panel_content(
                            inner,
                            &mut app_context_unwrapped_cloned,
                            panel_id,
                            *success,
                            panel.append_output.unwrap_or(false),
                            output,
                        );
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
                        let mut app_context_for_keypress = app_context_unwrapped.clone();
                        let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();

                        let selected_panels: Vec<&Panel> = active_layout.get_selected_panels();

                        let selected_panels_with_keypress_events: Vec<&Panel> = selected_panels
                            .clone()
                            .into_iter()
                            .filter(|p| p.on_keypress.is_some())
                            .filter(|p| p.choices.is_none())
                            .collect();

                        let libs = app_context_unwrapped.app.libs.clone();

                        if pressed_key == "Enter" {
                            let selected_panels_with_choices: Vec<&Panel> = selected_panels
                                .into_iter()
                                .filter(|p| p.choices.is_some())
                                .collect();
                            for panel in selected_panels_with_choices {
                                let panel_mut = app_context_for_keypress
                                    .app
                                    .get_panel_by_id_mut(panel.id.as_str())
                                    .unwrap();
                                let choices = panel_mut.choices.as_mut().unwrap();
                                let selected_choice = choices.iter_mut().find(|c| c.selected);
                                if let Some(selected_choice_unwrapped) = selected_choice {
                                    let script_clone = selected_choice_unwrapped.script.clone();
                                    if let Some(script_clone_unwrapped) = script_clone {
                                        let libs_clone = libs.clone();
                                        let job = move |sender: Sender<
                                            Result<ChoiceResult<String>, String>,
                                        >| {
                                            let result =
                                                run_script(libs_clone, &script_clone_unwrapped);
                                            let mut success = false;
                                            let result_string = match result {
                                                Ok(output) => {
                                                    success = true;
                                                    output
                                                }
                                                Err(e) => e.to_string(),
                                            };

                                            sender
                                                .send(Ok(ChoiceResult::new(success, result_string)))
                                                .unwrap();
                                        };

                                        let job_execution = POOL.execute(
                                            selected_choice_unwrapped.id.clone(),
                                            panel.id.clone(),
                                            job,
                                        );

                                        match job_execution {
                                            Ok(job_id) => {
                                                choice_ids_now_waiting.push((
                                                    panel.id.clone(),
                                                    selected_choice_unwrapped.id.clone(),
                                                ));
                                                log::trace!(
                                                    "Dispatched choice {:?} as job: {:?}",
                                                    selected_choice_unwrapped.id,
                                                    job_id
                                                );
                                                log::debug!(
                                                        "Queued jobs: {:?}, executing jobs: {:?}, finished jobs: {:?}",
                                                        POOL.get_queued_jobs().len(),
                                                        POOL.get_executing_jobs().len(),
                                                        POOL.get_finished_jobs().len()
                                                    );
                                            }
                                            Err(e) => {
                                                log::error!(
                                                    "Error dispatching choice script: {}",
                                                    e
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        for panel in selected_panels_with_keypress_events {
                            let actions =
                                handle_keypress(pressed_key, &panel.on_keypress.clone().unwrap());
                            if actions.is_none() {
                                if let Some(actions_unwrapped) = actions {
                                    let libs = app_context_unwrapped.app.libs.clone();

                                    match run_script(libs, &actions_unwrapped) {
                                        Ok(output) => {
                                            if panel.redirect_output.is_some() {
                                                update_panel_content(
                                                    inner,
                                                    &mut app_context_for_keypress,
                                                    panel.redirect_output.as_ref().unwrap(),
                                                    true,
                                                    panel.append_output.unwrap_or(false),
                                                    &output,
                                                )
                                            } else {
                                                update_panel_content(
                                                    inner,
                                                    &mut app_context_for_keypress,
                                                    &panel.id,
                                                    true,
                                                    panel.append_output.unwrap_or(false),
                                                    &output,
                                                )
                                            }
                                        }
                                        Err(e) => {
                                            if panel.redirect_output.is_some() {
                                                update_panel_content(
                                                    inner,
                                                    &mut app_context_for_keypress,
                                                    panel.redirect_output.as_ref().unwrap(),
                                                    false,
                                                    panel.append_output.unwrap_or(false),
                                                    e.to_string().as_str(),
                                                )
                                            } else {
                                                update_panel_content(
                                                    inner,
                                                    &mut app_context_for_keypress,
                                                    &panel.id,
                                                    false,
                                                    panel.append_output.unwrap_or(false),
                                                    e.to_string().as_str(),
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            let choice_results: Vec<ChoiceResultPacket<ChoiceResult<String>, String>> =
                POOL.get_results();
            let mut app_context_unwrapped_cloned = app_context_unwrapped.clone();
            for (panel_id, choice_id) in choice_ids_now_waiting {
                app_context_unwrapped_cloned
                    .app
                    .get_panel_by_id_mut(panel_id.as_str())
                    .unwrap()
                    .choices
                    .as_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|c| c.id == choice_id)
                    .unwrap()
                    .waiting = true;
            }

            for choice_result in choice_results {
                let panel = app_context_unwrapped
                    .app
                    .get_panel_by_id_mut(choice_result.panel_id.as_str())
                    .unwrap();
                let selected_choice = panel
                    .choices
                    .as_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|c| c.id == choice_result.choice_id)
                    .unwrap();

                log::trace!(
                    "received choice result for panel: {} choice: {}",
                    choice_result.panel_id,
                    choice_result.choice_id
                );

                log::trace!(
                    "Queued jobs: {:?}, executing jobs: {:?}, finished jobs: {:?}",
                    POOL.get_queued_jobs().len(),
                    POOL.get_executing_jobs().len(),
                    POOL.get_finished_jobs().len()
                );

                if POOL
                    .get_jobs_for_choice_id(&selected_choice.id, JobStatus::Executing)
                    .is_empty()
                {
                    log::trace!(
                        "Choice {:?} has finished executing, removing from waiting state.",
                        selected_choice.id
                    );

                    app_context_unwrapped_cloned
                        .app
                        .get_panel_by_id_mut(choice_result.panel_id.as_str())
                        .unwrap()
                        .choices
                        .as_mut()
                        .unwrap()
                        .iter_mut()
                        .find(|c| c.id == choice_result.choice_id)
                        .unwrap()
                        .waiting = false;
                }

                match choice_result.result {
                    Ok(output) => {
                        let cloned_output = output.clone();
                        log::trace!("Choice script output: {}", cloned_output.result);
                        if selected_choice.redirect_output.is_some() {
                            update_panel_content(
                                inner,
                                &mut app_context_unwrapped_cloned,
                                selected_choice.redirect_output.as_ref().unwrap(),
                                cloned_output.success,
                                selected_choice.append_output.unwrap_or(false),
                                cloned_output.result.as_str(),
                            )
                        } else {
                            let cloned_output = output.clone();
                            update_panel_content(
                                inner,
                                &mut app_context_unwrapped_cloned,
                                panel.id.as_ref(),
                                cloned_output.success,
                                selected_choice.append_output.unwrap_or(false),
                                cloned_output.result.as_str(),
                            )
                        }
                    }
                    Err(e) => {
                        log::error!("Error running choice script: {}", e);
                        if selected_choice.redirect_output.is_some() {
                            update_panel_content(
                                inner,
                                &mut app_context_unwrapped_cloned,
                                selected_choice.redirect_output.as_ref().unwrap(),
                                false,
                                selected_choice.append_output.unwrap_or(false),
                                e.to_string().as_str(),
                            )
                        } else {
                            update_panel_content(
                                inner,
                                &mut app_context_unwrapped_cloned,
                                panel.id.as_ref(),
                                false,
                                selected_choice.append_output.unwrap_or(false),
                                e.to_string().as_str(),
                            )
                        }
                    }
                }
            }

            // Ensure the loop continues by sleeping briefly
            std::thread::sleep(std::time::Duration::from_millis(
                app_context.config.frame_delay,
            ));
            return (should_continue, app_context_unwrapped_cloned);
        }

        (should_continue, app_context)
    }
);

pub fn update_panel_content(
    inner: &mut RunnableImpl,
    app_context_unwrapped: &mut AppContext,
    panel_id: &str,
    success: bool,
    append_output: bool,
    output: &str,
) {
    let mut app_context_unwrapped_cloned = app_context_unwrapped.clone();
    let panel = app_context_unwrapped.app.get_panel_by_id_mut(panel_id);

    log::trace!(
        "Updating panel content: {}, redirection: {:?}",
        panel_id,
        panel.as_ref().unwrap().redirect_output
    );
    if let Some(found_panel) = panel {
        if found_panel.redirect_output.is_some()
            && found_panel.redirect_output.as_ref().unwrap() != panel_id
        {
            log::trace!(
                "Redirecting output from panel {} to panel: {}",
                panel_id,
                found_panel.redirect_output.as_ref().unwrap()
            );
            update_panel_content(
                inner,
                &mut app_context_unwrapped_cloned,
                found_panel.redirect_output.as_ref().unwrap(),
                success,
                append_output,
                output,
            );
        } else {
            log::trace!("Updating panel {} content with no redirection.", panel_id);
            found_panel.update_content(output, append_output, success);
            inner.update_app_context(app_context_unwrapped.clone());
            inner.send_message(Message::RedrawPanel(panel_id.to_string()));
        }
    }
}
