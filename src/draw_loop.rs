use crate::draw_utils::{draw_app, draw_panel};
use crate::model::panel::Choice;
use crate::thread_manager::Runnable;
use crate::utils::{run_script_with_pty_and_redirect, should_use_pty_for_choice};
use crate::{
    apply_buffer, apply_buffer_if_changed, handle_keypress, run_script, AppContext, Panel,
    ScreenBuffer,
};
use crate::{thread_manager::*, FieldUpdate};
// use crossbeam_channel::Sender; // T311: Removed with ChoiceThreadManager
use crossterm::{
    terminal::{enable_raw_mode, EnterAlternateScreen},
    ExecutableCommand,
};
use std::io::stdout;
use std::io::Stdout;
use std::sync::{mpsc, Mutex};

use uuid::Uuid;

// F0188: Drag state tracking for draggable scroll knobs
#[derive(Debug, Clone)]
struct DragState {
    panel_id: String,
    is_vertical: bool,  // true for vertical scrollbar, false for horizontal
    start_x: u16,
    start_y: u16,
    start_scroll_percentage: f64,
}

static DRAG_STATE: Mutex<Option<DragState>> = Mutex::new(None);

// F0188: Helper functions to determine if click is on scroll knob (not just track)
fn is_on_vertical_knob(panel: &Panel, click_y: usize) -> bool {
    let panel_bounds = panel.bounds();
    let viewable_height = panel_bounds.height().saturating_sub(4);
    
    // Get content dimensions to calculate knob position and size
    let max_content_height = if let Some(content) = &panel.content {
        let lines: Vec<&str> = content.split('\n').collect();
        let mut total_height = lines.len();
        
        // Add choices height if present
        if let Some(choices) = &panel.choices {
            total_height += choices.len();
        }
        total_height
    } else if let Some(choices) = &panel.choices {
        choices.len()
    } else {
        viewable_height // No scrolling needed
    };
    
    if max_content_height <= viewable_height {
        return false; // No scrollbar needed
    }
    
    let track_height = viewable_height.saturating_sub(2);
    if track_height == 0 {
        return false;
    }
    
    // Calculate knob position and size (matching draw_utils.rs logic)
    let content_ratio = viewable_height as f64 / max_content_height as f64;
    let knob_size = std::cmp::max(1, (track_height as f64 * content_ratio).round() as usize);
    let available_track = track_height.saturating_sub(knob_size);
    
    let vertical_scroll = panel.vertical_scroll.unwrap_or(0.0);
    let knob_position = if available_track > 0 {
        ((vertical_scroll / 100.0) * available_track as f64).round() as usize
    } else {
        0
    };
    
    // Check if click is within knob bounds
    let knob_start_y = panel_bounds.top() + 1 + knob_position;
    let knob_end_y = knob_start_y + knob_size;
    
    click_y >= knob_start_y && click_y < knob_end_y
}

fn is_on_horizontal_knob(panel: &Panel, click_x: usize) -> bool {
    let panel_bounds = panel.bounds();
    let viewable_width = panel_bounds.width().saturating_sub(4);
    
    // Get content width to calculate knob position and size
    let max_content_width = if let Some(content) = &panel.content {
        let lines: Vec<&str> = content.split('\n').collect();
        lines.iter().map(|line| line.len()).max().unwrap_or(0)
    } else if let Some(choices) = &panel.choices {
        choices.iter()
            .map(|choice| choice.content.as_ref().map(|c| c.len()).unwrap_or(0))
            .max()
            .unwrap_or(0)
    } else {
        viewable_width // No scrolling needed
    };
    
    if max_content_width <= viewable_width {
        return false; // No scrollbar needed
    }
    
    let track_width = viewable_width.saturating_sub(2);
    if track_width == 0 {
        return false;
    }
    
    // Calculate knob position and size (matching draw_utils.rs logic)
    let content_ratio = viewable_width as f64 / max_content_width as f64;
    let knob_size = std::cmp::max(1, (track_width as f64 * content_ratio).round() as usize);
    let available_track = track_width.saturating_sub(knob_size);
    
    let horizontal_scroll = panel.horizontal_scroll.unwrap_or(0.0);
    let knob_position = if available_track > 0 {
        ((horizontal_scroll / 100.0) * available_track as f64).round() as usize
    } else {
        0
    };
    
    // Check if click is within knob bounds
    let knob_start_x = panel_bounds.left() + 1 + knob_position;
    let knob_end_x = knob_start_x + knob_size;
    
    click_x >= knob_start_x && click_x < knob_end_x
}

lazy_static! {
    static ref GLOBAL_SCREEN: Mutex<Option<Stdout>> = Mutex::new(None);
    static ref GLOBAL_BUFFER: Mutex<Option<ScreenBuffer>> = Mutex::new(None);
}

create_runnable!(
    DrawLoop,
    |_inner: &mut RunnableImpl, app_context: AppContext, _messages: Vec<Message>| -> bool {
        let mut global_screen = GLOBAL_SCREEN.lock().unwrap();
        let mut global_buffer = GLOBAL_BUFFER.lock().unwrap();
        let mut app_context_unwrapped = app_context.clone();
        let (adjusted_bounds, app_graph) = app_context_unwrapped
            .app
            .get_adjusted_bounds_and_app_graph(Some(true));

        let is_first_render = global_screen.is_none();
        if is_first_render {
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
            if is_first_render {
                // Force full render on first run to ensure everything is drawn
                apply_buffer(&mut new_buffer, screen);
            } else {
                apply_buffer_if_changed(buffer, &new_buffer, screen);
            }
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
            let mut new_buffer;
            let mut app_context_unwrapped = app_context.clone();
            let (adjusted_bounds, app_graph) = app_context_unwrapped
                .app
                .get_adjusted_bounds_and_app_graph(Some(true));
            // T311: choice_ids_now_waiting removed - no longer needed with unified threading

            if !messages.is_empty() {
                log::info!("DrawLoop processing {} messages", messages.len());
                for msg in &messages {
                    match msg {
                        Message::ChoiceExecutionComplete(choice_id, panel_id, _) => {
                            log::info!(
                                "About to process ChoiceExecutionComplete: {} -> {}",
                                choice_id,
                                panel_id
                            );
                        }
                        _ => {}
                    }
                }
            }

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
                    Message::ScrollPanelPageUp() => {
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                // Page up scrolls by larger amount (10 units for page-based scrolling)
                                found_panel.scroll_up(Some(10.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelPageDown() => {
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                // Page down scrolls by larger amount (10 units for page-based scrolling)
                                found_panel.scroll_down(Some(10.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelPageLeft() => {
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                // Page left scrolls by larger amount (10 units for page-based scrolling)
                                found_panel.scroll_left(Some(10.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelPageRight() => {
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                // Page right scrolls by larger amount (10 units for page-based scrolling)
                                found_panel.scroll_right(Some(10.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelToBeginning() => {
                        // Home key: scroll to beginning horizontally (horizontal_scroll = 0)
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.horizontal_scroll = Some(0.0);
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelToEnd() => {
                        // End key: scroll to end horizontally (horizontal_scroll = 100)
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.horizontal_scroll = Some(100.0);
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelToTop() => {
                        // Ctrl+Home: scroll to top vertically (vertical_scroll = 0)
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.vertical_scroll = Some(0.0);
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::ScrollPanelToBottom() => {
                        // Ctrl+End: scroll to bottom vertically (vertical_scroll = 100)
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id_mut(&selected_id);
                            if let Some(found_panel) = panel {
                                found_panel.vertical_scroll = Some(100.0);
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawPanel(selected_id));
                            }
                        }
                    }
                    Message::CopyFocusedPanelContent() => {
                        let selected_panels = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_panels();
                        if !selected_panels.is_empty() {
                            let selected_id = selected_panels.first().unwrap().id.clone();
                            let panel = app_context_unwrapped.app.get_panel_by_id(&selected_id);
                            if let Some(found_panel) = panel {
                                // Get panel content to copy
                                let content_to_copy = get_panel_content_for_clipboard(found_panel);

                                // Copy to clipboard
                                if copy_to_clipboard(&content_to_copy).is_ok() {
                                    // Trigger visual flash for the panel
                                    trigger_panel_flash(&selected_id);
                                    inner.send_message(Message::RedrawPanel(selected_id));
                                }
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
                        screen
                            .execute(crossterm::terminal::Clear(
                                crossterm::terminal::ClearType::All,
                            ))
                            .unwrap();
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
                        log::info!("RECEIVED PanelOutputUpdate for panel: {}, success: {}, output_len: {}, preview: {}", 
                                   panel_id, success, output.len(), output.chars().take(50).collect::<String>());
                        let mut app_context_unwrapped_cloned = app_context_unwrapped.clone();
                        // For PTY streaming output, we need to use a special update method
                        // that doesn't add timestamp formatting. The presence of a newline
                        // at the end of the output indicates it's PTY streaming data.
                        let is_pty_streaming = output.ends_with('\n');

                        if is_pty_streaming {
                            // Use streaming update for PTY output
                            let target_panel = app_context_unwrapped_cloned
                                .app
                                .get_panel_by_id_mut(panel_id)
                                .unwrap();
                            target_panel.update_streaming_content(output, *success);
                            inner.update_app_context(app_context_unwrapped_cloned.clone());
                            inner.send_message(Message::RedrawPanel(panel_id.to_string()));
                        } else {
                            // Use regular update for non-PTY output
                            let panel =
                                app_context_unwrapped.app.get_panel_by_id(panel_id).unwrap();
                            update_panel_content(
                                inner,
                                &mut app_context_unwrapped_cloned,
                                panel_id,
                                *success,
                                panel.append_output.unwrap_or(false),
                                output,
                            );
                        }
                    }
                    // ExternalMessage handling is now done by RSJanusComms library
                    // Messages are converted to appropriate internal messages by the socket handler
                    Message::ExternalMessage(_) => {
                        // This should no longer be used - socket handler converts messages directly
                        log::warn!("Received deprecated ExternalMessage - should be converted by socket handler");
                    }
                    Message::ExecuteHotKeyChoice(choice_id) => {
                        log::info!("=== EXECUTING HOT KEY CHOICE: {} ===", choice_id);

                        let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();

                        // Find the choice by ID in any panel
                        log::info!("Searching for choice {} in active layout", choice_id);
                        if let Some(choice_panel) = active_layout.find_panel_with_choice(&choice_id)
                        {
                            log::info!("Found choice in panel: {}", choice_panel.id);

                            if let Some(choices) = &choice_panel.choices {
                                if let Some(choice) = choices.iter().find(|c| c.id == *choice_id) {
                                    // T315: Unified choice execution - thread field no longer affects execution path
                                    log::info!("Executing choice config - pty: {}, redirect: {:?}, script_lines: {}", 
                                        choice.pty.unwrap_or(false),
                                        choice.redirect_output,
                                        choice.script.as_ref().map(|s| s.len()).unwrap_or(0)
                                    );

                                    if let Some(script) = &choice.script {
                                        let libs = app_context_unwrapped.app.libs.clone();
                                        let use_pty = should_use_pty_for_choice(choice);
                                        let pty_manager =
                                            app_context_unwrapped.pty_manager.as_ref();
                                        let message_sender = Some((
                                            inner.get_message_sender().as_ref().unwrap().clone(),
                                            inner.get_uuid(),
                                        ));

                                        log::info!("Unified execution - use_pty: {}, has_manager: {}, redirect: {:?}", 
                                            use_pty, pty_manager.is_some(), choice.redirect_output);

                                        let result = run_script_with_pty_and_redirect(
                                            libs,
                                            script,
                                            use_pty,
                                            pty_manager.map(|arc| arc.as_ref()),
                                            Some(choice.id.clone()),
                                            message_sender,
                                            choice.redirect_output.clone(),
                                        );

                                        // Send completion message via unified system
                                        inner.send_message(Message::ChoiceExecutionComplete(
                                            choice_id.clone(),
                                            choice_panel.id.clone(),
                                            result.map_err(|e| e.to_string()),
                                        ));
                                    }
                                } else {
                                    log::warn!("Choice {} found in panel {} but no matching choice in choices list", choice_id, choice_panel.id);
                                }
                            } else {
                                log::warn!("Panel {} has no choices list", choice_panel.id);
                            }
                        } else {
                            log::error!(
                                "Choice {} not found in any panel of active layout",
                                choice_id
                            );
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
                                // First, extract choice information before any mutable operations
                                let (selected_choice_data, choice_needs_execution) = {
                                    let panel_ref = app_context_for_keypress
                                        .app
                                        .get_panel_by_id(&panel.id)
                                        .unwrap();
                                    if let Some(ref choices) = panel_ref.choices {
                                        if let Some(selected_choice) =
                                            choices.iter().find(|c| c.selected)
                                        {
                                            let choice_data = (
                                                selected_choice.id.clone(),
                                                selected_choice.script.clone(),
                                                selected_choice.pty.unwrap_or(false),
                                                selected_choice.thread.unwrap_or(false),
                                                selected_choice.redirect_output.clone(),
                                                selected_choice.append_output.unwrap_or(false),
                                                panel.id.clone(),
                                            );
                                            (Some(choice_data), selected_choice.script.is_some())
                                        } else {
                                            (None, false)
                                        }
                                    } else {
                                        (None, false)
                                    }
                                };

                                if let Some((
                                    choice_id,
                                    script_opt,
                                    use_pty,
                                    use_thread,
                                    redirect_output,
                                    append_output,
                                    panel_id,
                                )) = selected_choice_data
                                {
                                    if choice_needs_execution {
                                        log::info!(
                                            "=== ENTER KEY CHOICE EXECUTION: {} (panel: {}) ===",
                                            choice_id,
                                            panel_id
                                        );
                                        log::info!("Enter choice config - pty: {}, thread: {}, redirect: {:?}", 
                                            use_pty, use_thread, redirect_output
                                        );

                                        if let Some(script) = script_opt {
                                            let libs_clone = libs.clone();

                                            // T312: Execute choice using unified threading system - proper architecture
                                            log::info!("Enter key requesting ThreadManager to execute choice {} (pty: {})", choice_id, use_pty);

                                            // Set choice to waiting state before execution
                                            if let Some(panel_mut) = app_context_for_keypress
                                                .app
                                                .get_panel_by_id_mut(&panel_id)
                                            {
                                                if let Some(ref mut choices) = panel_mut.choices {
                                                    if let Some(choice) = choices
                                                        .iter_mut()
                                                        .find(|c| c.id == choice_id)
                                                    {
                                                        choice.waiting = true;
                                                    }
                                                }
                                            }

                                            // Create the choice object for execution
                                            let choice_for_execution = Choice {
                                                id: choice_id.clone(),
                                                content: Some("".to_string()), // Not needed for execution
                                                selected: false, // Not needed for execution
                                                script: Some(script.clone()),
                                                pty: Some(use_pty),
                                                thread: Some(use_thread),
                                                redirect_output: redirect_output.clone(),
                                                append_output: Some(append_output),
                                                waiting: true,
                                            };

                                            // Send ExecuteChoice message to ThreadManager (proper architecture)
                                            log::info!("Sending ExecuteChoice message for choice {} (pty: {}, thread: {})", 
                                            choice_id, use_pty, use_thread);
                                            inner.send_message(Message::ExecuteChoice(
                                                choice_for_execution,
                                                panel_id.clone(),
                                                libs_clone,
                                            ));

                                            // Update the app context to persist the waiting state change
                                            inner.update_app_context(
                                                app_context_for_keypress.clone(),
                                            );

                                            log::trace!(
                                                "ExecuteChoice message sent for choice {}",
                                                choice_id
                                            );
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
                    Message::PTYInput(panel_id, input) => {
                        log::trace!("PTY input for panel {}: {}", panel_id, input);

                        // Find the target panel to verify it exists and has PTY enabled
                        if let Some(panel) = app_context_unwrapped.app.get_panel_by_id(panel_id) {
                            if panel.pty.unwrap_or(false) {
                                log::debug!(
                                    "Routing input to PTY panel {}: {:?}",
                                    panel_id,
                                    input.chars().collect::<Vec<_>>()
                                );

                                // TODO: Write input to PTY process when PTY manager is thread-safe
                                // For now, log the successful routing detection
                                log::info!(
                                    "PTY input ready for routing to panel {}: {} chars",
                                    panel_id,
                                    input.len()
                                );
                            } else {
                                log::warn!(
                                    "Panel {} received PTY input but pty field is false",
                                    panel_id
                                );
                            }
                        } else {
                            log::error!("PTY input received for non-existent panel: {}", panel_id);
                        }
                    }
                    Message::MouseClick(x, y) => {
                        log::trace!("Mouse click at ({}, {})", x, y);
                        let mut app_context_for_click = app_context_unwrapped.clone();
                        let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();

                        // F0187: Check for scrollbar clicks first
                        let mut handled_scrollbar_click = false;
                        for panel in active_layout.get_all_panels() {
                            if panel.has_scrollable_content() {
                                let panel_bounds = panel.bounds();
                                
                                // Check for vertical scrollbar click (right border)
                                if *x as usize == panel_bounds.right() && 
                                   *y as usize > panel_bounds.top() && (*y as usize) < panel_bounds.bottom() {
                                    let track_height = (panel_bounds.height() as isize - 2).max(1) as usize;
                                    let click_position = ((*y as usize) - panel_bounds.top() - 1) as f64 / track_height as f64;
                                    let scroll_percentage = (click_position * 100.0).min(100.0).max(0.0);
                                    
                                    log::trace!("Vertical scrollbar click on panel {} at {}%", panel.id, scroll_percentage);
                                    
                                    // Update panel vertical scroll
                                    let panel_to_update = app_context_for_click
                                        .app
                                        .get_panel_by_id_mut(&panel.id)
                                        .unwrap();
                                    panel_to_update.vertical_scroll = Some(scroll_percentage);
                                    
                                    inner.update_app_context(app_context_for_click.clone());
                                    inner.send_message(Message::RedrawApp);
                                    handled_scrollbar_click = true;
                                    break;
                                }
                                
                                // Check for horizontal scrollbar click (bottom border)
                                if *y as usize == panel_bounds.bottom() && 
                                   *x as usize > panel_bounds.left() && (*x as usize) < panel_bounds.right() {
                                    let track_width = (panel_bounds.width() as isize - 2).max(1) as usize;
                                    let click_position = ((*x as usize) - panel_bounds.left() - 1) as f64 / track_width as f64;
                                    let scroll_percentage = (click_position * 100.0).min(100.0).max(0.0);
                                    
                                    log::trace!("Horizontal scrollbar click on panel {} at {}%", panel.id, scroll_percentage);
                                    
                                    // Update panel horizontal scroll
                                    let panel_to_update = app_context_for_click
                                        .app
                                        .get_panel_by_id_mut(&panel.id)
                                        .unwrap();
                                    panel_to_update.horizontal_scroll = Some(scroll_percentage);
                                    
                                    inner.update_app_context(app_context_for_click.clone());
                                    inner.send_message(Message::RedrawApp);
                                    handled_scrollbar_click = true;
                                    break;
                                }
                            }
                        }
                        
                        // If scrollbar click was handled, skip panel selection
                        if handled_scrollbar_click {
                            // Continue to next message
                        } else {
                            // F0091: Find which panel was clicked based on coordinates
                            if let Some(clicked_panel) = active_layout.find_panel_at_coordinates(*x, *y)
                        {
                            log::trace!("Clicked on panel: {}", clicked_panel.id);

                            // Check if panel has choices (menu items)
                            if let Some(choices) = &clicked_panel.choices {
                                // Calculate which choice was clicked based on y offset within panel
                                if let Some(clicked_choice_idx) =
                                    calculate_clicked_choice_index(clicked_panel, *y, choices.len())
                                {
                                    if let Some(clicked_choice) = choices.get(clicked_choice_idx) {
                                        log::trace!("Clicked on choice: {}", clicked_choice.id);

                                        // First, select the parent panel if not already selected
                                        let layout = app_context_for_click
                                            .app
                                            .get_active_layout_mut()
                                            .unwrap();
                                        layout.deselect_all_panels();
                                        layout.select_only_panel(&clicked_panel.id);

                                        // Then select the clicked choice visually
                                        let panel_to_update = app_context_for_click
                                            .app
                                            .get_panel_by_id_mut(&clicked_panel.id)
                                            .unwrap();
                                        if let Some(ref mut panel_choices) = panel_to_update.choices
                                        {
                                            // Deselect all choices first
                                            for choice in panel_choices.iter_mut() {
                                                choice.selected = false;
                                            }
                                            // Select only the clicked choice
                                            if let Some(selected_choice) =
                                                panel_choices.get_mut(clicked_choice_idx)
                                            {
                                                selected_choice.selected = true;
                                            }
                                        }

                                        // Update the app context and immediately trigger redraw for responsiveness
                                        inner.update_app_context(app_context_for_click.clone());
                                        inner.send_message(Message::RedrawApp);

                                        // Then activate the clicked choice (same as pressing Enter)
                                        // Force threaded execution for clicked choices to maintain UI responsiveness
                                        if let Some(script) = &clicked_choice.script {
                                            let libs = app_context_unwrapped.app.libs.clone();

                                            // Always use threaded execution for mouse clicks to keep UI responsive
                                            let _script_clone = script.clone();
                                            let _choice_id_clone = clicked_choice.id.clone();
                                            let panel_id_clone = clicked_panel.id.clone();
                                            let libs_clone = libs.clone();

                                            // T312: Use unified ExecuteChoice message system
                                            inner.send_message(Message::ExecuteChoice(
                                                clicked_choice.clone(),
                                                panel_id_clone,
                                                libs_clone,
                                            ));

                                            // Spawn the choice execution in ThreadManager
                                            // TODO: Get ThreadManager reference to spawn the runnable
                                            log::trace!("Mouse click choice {} ready for ThreadManager execution", clicked_choice.id);
                                        }
                                    }
                                } else {
                                    // Click was on panel with choices but not on any specific choice
                                    // Only select the panel, don't activate any choice
                                    if clicked_panel.tab_order.is_some()
                                        || clicked_panel.has_scrollable_content()
                                    {
                                        log::trace!(
                                            "Selecting panel (clicked on empty area): {}",
                                            clicked_panel.id
                                        );

                                        // Deselect all panels in the layout first
                                        let layout = app_context_for_click
                                            .app
                                            .get_active_layout_mut()
                                            .unwrap();
                                        layout.deselect_all_panels();
                                        layout.select_only_panel(&clicked_panel.id);

                                        inner.update_app_context(app_context_for_click);
                                        inner.send_message(Message::RedrawApp);
                                    }
                                }
                            } else {
                                // Panel has no choices - just select it if it's selectable
                                if clicked_panel.tab_order.is_some()
                                    || clicked_panel.has_scrollable_content()
                                {
                                    log::trace!(
                                        "Selecting panel (no choices): {}",
                                        clicked_panel.id
                                    );

                                    // Deselect all panels in the layout first
                                    let layout =
                                        app_context_for_click.app.get_active_layout_mut().unwrap();
                                    layout.deselect_all_panels();
                                    layout.select_only_panel(&clicked_panel.id);

                                    inner.update_app_context(app_context_for_click);
                                    inner.send_message(Message::RedrawApp);
                                }
                            }
                        }
                        }
                    }
                    Message::MouseDragStart(x, y) => {
                        // F0188: Check if drag started on a scroll knob
                        let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();
                        let mut drag_state = DRAG_STATE.lock().unwrap();
                        *drag_state = None; // Clear any previous drag state

                        for panel in active_layout.get_all_panels() {
                            if panel.has_scrollable_content() {
                                let panel_bounds = panel.bounds();
                                
                                // Check if drag started on vertical scroll knob
                                if *x as usize == panel_bounds.right() && 
                                   *y as usize > panel_bounds.top() && (*y as usize) < panel_bounds.bottom() {
                                    // Check if we clicked on the actual knob, not just the track
                                    if is_on_vertical_knob(panel, *y as usize) {
                                        let current_scroll = panel.vertical_scroll.unwrap_or(0.0);
                                        *drag_state = Some(DragState {
                                            panel_id: panel.id.clone(),
                                            is_vertical: true,
                                            start_x: *x,
                                            start_y: *y,
                                            start_scroll_percentage: current_scroll,
                                        });
                                        log::trace!("Started dragging vertical scroll knob on panel {}", panel.id);
                                        break;
                                    }
                                }
                                
                                // Check if drag started on horizontal scroll knob
                                if *y as usize == panel_bounds.bottom() && 
                                   *x as usize > panel_bounds.left() && (*x as usize) < panel_bounds.right() {
                                    // Check if we clicked on the actual knob, not just the track
                                    if is_on_horizontal_knob(panel, *x as usize) {
                                        let current_scroll = panel.horizontal_scroll.unwrap_or(0.0);
                                        *drag_state = Some(DragState {
                                            panel_id: panel.id.clone(),
                                            is_vertical: false,
                                            start_x: *x,
                                            start_y: *y,
                                            start_scroll_percentage: current_scroll,
                                        });
                                        log::trace!("Started dragging horizontal scroll knob on panel {}", panel.id);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                    Message::MouseDrag(x, y) => {
                        // F0188: Update scroll position during drag
                        let drag_state_guard = DRAG_STATE.lock().unwrap();
                        if let Some(ref drag_state) = *drag_state_guard {
                            let panel_to_update = app_context_unwrapped
                                .app
                                .get_panel_by_id_mut(&drag_state.panel_id);
                                
                            if let Some(panel) = panel_to_update {
                                let panel_bounds = panel.bounds();
                                
                                if drag_state.is_vertical {
                                    // Calculate new vertical scroll percentage based on drag distance
                                    let track_height = (panel_bounds.height() as isize - 2).max(1) as usize;
                                    let drag_delta = (*y as isize) - (drag_state.start_y as isize);
                                    let percentage_delta = (drag_delta as f64 / track_height as f64) * 100.0;
                                    let new_percentage = (drag_state.start_scroll_percentage + percentage_delta)
                                        .min(100.0).max(0.0);
                                    
                                    panel.vertical_scroll = Some(new_percentage);
                                } else {
                                    // Calculate new horizontal scroll percentage based on drag distance
                                    let track_width = (panel_bounds.width() as isize - 2).max(1) as usize;
                                    let drag_delta = (*x as isize) - (drag_state.start_x as isize);
                                    let percentage_delta = (drag_delta as f64 / track_width as f64) * 100.0;
                                    let new_percentage = (drag_state.start_scroll_percentage + percentage_delta)
                                        .min(100.0).max(0.0);
                                    
                                    panel.horizontal_scroll = Some(new_percentage);
                                }
                                
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawApp);
                            }
                        }
                    }
                    Message::MouseDragEnd(x, y) => {
                        // F0188: End drag operation
                        let mut drag_state = DRAG_STATE.lock().unwrap();
                        if drag_state.is_some() {
                            log::trace!("Ended scroll knob drag at ({}, {})", x, y);
                            *drag_state = None; // Clear drag state
                        }
                    }
                    Message::ChoiceExecutionComplete(choice_id, panel_id, result) => {
                        log::info!(
                            "=== DRAWLOOP RECEIVED CHOICE EXECUTION COMPLETE: {} on panel {} ===",
                            choice_id,
                            panel_id
                        );
                        match result {
                            Ok(ref output) => log::info!(
                                "DrawLoop processing choice success: {} chars of output",
                                output.len()
                            ),
                            Err(ref error) => {
                                log::error!("DrawLoop processing choice error: {}", error)
                            }
                        }

                        // First update the choice waiting state
                        if let Some(panel) = app_context_unwrapped.app.get_panel_by_id_mut(panel_id)
                        {
                            if let Some(ref mut choices) = panel.choices {
                                if let Some(choice) =
                                    choices.iter_mut().find(|c| c.id == *choice_id)
                                {
                                    choice.waiting = false;
                                }
                            }
                        }

                        // Then handle the output in a separate scope to avoid borrow conflicts
                        let target_panel_id = {
                            if let Some(panel) = app_context_unwrapped.app.get_panel_by_id(panel_id)
                            {
                                if let Some(ref choices) = panel.choices {
                                    if let Some(choice) =
                                        choices.iter().find(|c| c.id == *choice_id)
                                    {
                                        let redirect_target = choice
                                            .redirect_output
                                            .as_ref()
                                            .unwrap_or(panel_id)
                                            .clone();
                                        log::info!(
                                            "Choice {} redirect_output: {:?} -> target panel: {}",
                                            choice_id,
                                            choice.redirect_output,
                                            redirect_target
                                        );
                                        redirect_target
                                    } else {
                                        log::warn!(
                                            "Choice {} not found in panel {}",
                                            choice_id,
                                            panel_id
                                        );
                                        panel_id.clone()
                                    }
                                } else {
                                    log::warn!("Panel {} has no choices", panel_id);
                                    panel_id.clone()
                                }
                            } else {
                                log::error!("Panel {} not found", panel_id);
                                panel_id.clone()
                            }
                        };

                        let append = {
                            if let Some(panel) = app_context_unwrapped.app.get_panel_by_id(panel_id)
                            {
                                if let Some(ref choices) = panel.choices {
                                    if let Some(choice) =
                                        choices.iter().find(|c| c.id == *choice_id)
                                    {
                                        choice.append_output.unwrap_or(false)
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };

                        match result {
                            Ok(output) => {
                                log::info!(
                                    "Choice {} output length: {} chars, redirecting to panel: {}",
                                    choice_id,
                                    output.len(),
                                    target_panel_id
                                );
                                update_panel_content(
                                    inner,
                                    &mut app_context_unwrapped,
                                    &target_panel_id,
                                    true,
                                    append,
                                    output,
                                );
                            }
                            Err(error) => {
                                log::error!("Error running choice script: {}", error);
                                update_panel_content(
                                    inner,
                                    &mut app_context_unwrapped,
                                    &target_panel_id,
                                    false,
                                    append,
                                    error,
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }

            // T311: Choice execution now handled via ChoiceExecutionComplete messages
            // Old POOL-based choice results processing removed

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
    success: bool,
    append_output: bool,
    output: &str,
) {
    log::info!(
        "=== UPDATE PANEL CONTENT: {} (success: {}, append: {}, output_len: {}) ===",
        panel_id,
        success,
        append_output,
        output.len()
    );

    let mut app_context_unwrapped_cloned = app_context_unwrapped.clone();
    let panel = app_context_unwrapped.app.get_panel_by_id_mut(panel_id);

    if let Some(found_panel) = panel {
        log::info!(
            "Found target panel: {} (redirect_output: {:?})",
            panel_id,
            found_panel.redirect_output
        );

        if found_panel.redirect_output.is_some()
            && found_panel.redirect_output.as_ref().unwrap() != panel_id
        {
            log::info!(
                "Panel {} has its own redirect to: {}, following redirect chain",
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
            log::info!(
                "Updating panel {} content directly (no redirection)",
                panel_id
            );
            log::info!(
                "Panel {} current content length: {} chars",
                panel_id,
                found_panel.content.as_ref().map_or(0, |c| c.len())
            );

            // Check if this is PTY streaming output by the newline indicator
            let is_pty_streaming = output.ends_with('\n');

            if is_pty_streaming {
                // Use streaming update for PTY output (no timestamp formatting)
                log::info!("Using streaming update for panel {}", panel_id);
                found_panel.update_streaming_content(output, success);
            } else {
                // Use regular update for non-PTY output
                log::info!(
                    "Using regular update for panel {} (append: {})",
                    panel_id,
                    append_output
                );
                found_panel.update_content(output, append_output, success);
            }

            log::info!(
                "Panel {} updated content length: {} chars",
                panel_id,
                found_panel.content.as_ref().map_or(0, |c| c.len())
            );
            inner.update_app_context(app_context_unwrapped.clone());
            inner.send_message(Message::RedrawPanel(panel_id.to_string()));
            log::info!("Sent RedrawPanel message for panel: {}", panel_id);
        }
    } else {
        log::error!("Could not find panel {} for content update.", panel_id);
        // List available panels for debugging
        let available_panels: Vec<String> = app_context_unwrapped
            .app
            .get_active_layout()
            .unwrap()
            .get_all_panels()
            .iter()
            .map(|p| p.id.clone())
            .collect();
        log::error!("Available panels: {:?}", available_panels);
    }
}

/// Extract panel content for clipboard copy
pub fn get_panel_content_for_clipboard(panel: &Panel) -> String {
    // Priority order: output > content > default message
    if !panel.output.is_empty() {
        panel.output.clone()
    } else if let Some(content) = &panel.content {
        content.clone()
    } else {
        format!("Panel '{}': No content", panel.id)
    }
}

/// Copy text to system clipboard
pub fn copy_to_clipboard(content: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::process::Command;

    // Platform-specific clipboard commands
    #[cfg(target_os = "macos")]
    {
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.take() {
            use std::io::Write;
            let mut stdin = stdin;
            stdin.write_all(content.as_bytes())?;
        }

        child.wait()?;
    }

    #[cfg(target_os = "linux")]
    {
        // Try xclip first, then xsel as fallback
        let result = Command::new("xclip")
            .arg("-selection")
            .arg("clipboard")
            .stdin(std::process::Stdio::piped())
            .spawn();

        match result {
            Ok(mut child) => {
                if let Some(stdin) = child.stdin.take() {
                    use std::io::Write;
                    let mut stdin = stdin;
                    stdin.write_all(content.as_bytes())?;
                }
                child.wait()?;
            }
            Err(_) => {
                // Fallback to xsel
                let mut child = Command::new("xsel")
                    .arg("--clipboard")
                    .arg("--input")
                    .stdin(std::process::Stdio::piped())
                    .spawn()?;

                if let Some(stdin) = child.stdin.take() {
                    use std::io::Write;
                    let mut stdin = stdin;
                    stdin.write_all(content.as_bytes())?;
                }
                child.wait()?;
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        let mut child = Command::new("clip")
            .stdin(std::process::Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.take() {
            use std::io::Write;
            let mut stdin = stdin;
            stdin.write_all(content.as_bytes())?;
        }

        child.wait()?;
    }

    Ok(())
}

/// Calculate which choice was clicked based on panel bounds and click coordinates
/// Matches the actual choice rendering: one choice per line starting at bounds.top() + 1
#[cfg(test)]
pub fn calculate_clicked_choice_index(
    panel: &Panel,
    click_y: u16,
    num_choices: usize,
) -> Option<usize> {
    calculate_clicked_choice_index_impl(panel, click_y, num_choices)
}

#[cfg(not(test))]
fn calculate_clicked_choice_index(
    panel: &Panel,
    click_y: u16,
    num_choices: usize,
) -> Option<usize> {
    calculate_clicked_choice_index_impl(panel, click_y, num_choices)
}

fn calculate_clicked_choice_index_impl(
    panel: &Panel,
    click_y: u16,
    num_choices: usize,
) -> Option<usize> {
    let bounds = panel.bounds();
    let panel_top = bounds.y1 as u16;

    if click_y < panel_top || num_choices == 0 {
        return None;
    }

    // Choices start at bounds.top() + 1 (one line below border) as per draw_utils.rs:553
    let choices_start_y = panel_top + 1;

    if click_y < choices_start_y {
        return None; // Click was on border or title area
    }

    // Each choice occupies exactly 1 line, so choice index = relative y offset
    let choice_index = (click_y - choices_start_y) as usize;

    // Ensure click is within choice bounds (don't exceed available choices or panel height)
    let panel_bottom = bounds.y2 as u16;
    if choice_index < num_choices && click_y < panel_bottom {
        Some(choice_index)
    } else {
        None
    }
}

/// Trigger visual flash for panel (stub implementation)
fn trigger_panel_flash(_panel_id: &str) {
    // TODO: Implement visual flash with color inversion
    // This would require storing flash state and modifying panel rendering
    // For now, the redraw provides visual feedback
}
