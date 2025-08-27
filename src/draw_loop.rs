use crate::draw_utils::{draw_app, draw_muxbox};
use crate::model::app::{save_muxbox_bounds_to_yaml, save_complete_state_to_yaml, save_active_layout_to_yaml, save_muxbox_content_to_yaml, save_muxbox_scroll_to_yaml};
use crate::model::common::{InputBounds, StreamSourceTrait};
use crate::model::muxbox::Choice;
use crate::thread_manager::Runnable;
use crate::utils::{run_script_with_pty_and_redirect, should_use_pty_for_choice};
use crate::{
    apply_buffer, apply_buffer_if_changed, handle_keypress, run_script, AppContext, MuxBox,
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
    muxbox_id: String,
    is_vertical: bool, // true for vertical scrollbar, false for horizontal
    start_x: u16,
    start_y: u16,
    start_scroll_percentage: f64,
}

// F0189: MuxBox resize state tracking for draggable muxbox borders
#[derive(Debug, Clone)]
struct MuxBoxResizeState {
    muxbox_id: String,
    resize_edge: ResizeEdge,
    start_x: u16,
    start_y: u16,
    original_bounds: InputBounds,
}

// F0191: MuxBox move state tracking for draggable muxbox titles/top borders
#[derive(Debug, Clone)]
struct MuxBoxMoveState {
    muxbox_id: String,
    start_x: u16,
    start_y: u16,
    original_bounds: InputBounds,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResizeEdge {
    BottomRight, // Only corner resize allowed
}

static DRAG_STATE: Mutex<Option<DragState>> = Mutex::new(None);
static MUXBOX_RESIZE_STATE: Mutex<Option<MuxBoxResizeState>> = Mutex::new(None);
static MUXBOX_MOVE_STATE: Mutex<Option<MuxBoxMoveState>> = Mutex::new(None);

// F0189: Helper functions to detect muxbox border resize areas (corner-only)
pub fn detect_resize_edge(muxbox: &MuxBox, click_x: u16, click_y: u16) -> Option<ResizeEdge> {
    let bounds = muxbox.bounds();
    let x = click_x as usize;
    let y = click_y as usize;

    // Check for corner resize (bottom-right only) with tolerance for easier clicking
    // Allow clicking within 1 pixel of the exact corner to make it easier to grab
    let corner_tolerance = 1;

    // Standard detection zone - same for all panels including 100% width
    if (x >= bounds.x2.saturating_sub(corner_tolerance) && x <= bounds.x2)
        && (y >= bounds.y2.saturating_sub(corner_tolerance) && y <= bounds.y2)
    {
        return Some(ResizeEdge::BottomRight);
    }

    None
}

// F0191: Helper function to detect muxbox title/top border for movement
pub fn detect_move_area(muxbox: &MuxBox, click_x: u16, click_y: u16) -> bool {
    let bounds = muxbox.bounds();
    let x = click_x as usize;
    let y = click_y as usize;

    // Check for title area or top border (y1 coordinate across muxbox width)
    y == bounds.y1 && x >= bounds.x1 && x <= bounds.x2
}

pub fn calculate_new_bounds(
    original_bounds: &InputBounds,
    resize_edge: &ResizeEdge,
    start_x: u16,
    start_y: u16,
    current_x: u16,
    current_y: u16,
    terminal_width: usize,
    terminal_height: usize,
) -> InputBounds {
    let delta_x = (current_x as i32) - (start_x as i32);
    let delta_y = (current_y as i32) - (start_y as i32);

    let mut new_bounds = original_bounds.clone();

    // F0197: Minimum resize constraints - prevent boxes smaller than 2x2 characters
    let min_width_percent = (2.0 / terminal_width as f32) * 100.0;
    let min_height_percent = (2.0 / terminal_height as f32) * 100.0;

    match resize_edge {
        ResizeEdge::BottomRight => {
            // Update both x2 and y2 coordinates for corner resize
            if let Ok(current_x2_percent) = new_bounds.x2.replace('%', "").parse::<f32>() {
                if let Ok(current_x1_percent) = new_bounds.x1.replace('%', "").parse::<f32>() {
                    let pixel_delta_x = delta_x as f32;
                    let percent_delta_x = (pixel_delta_x / terminal_width as f32) * 100.0;
                    let new_x2_percent =
                        (current_x2_percent + percent_delta_x).max(10.0).min(100.0);

                    // Enforce minimum width constraint
                    let min_x2_for_width = current_x1_percent + min_width_percent;
                    let constrained_x2 = new_x2_percent.max(min_x2_for_width);

                    new_bounds.x2 = format!("{}%", constrained_x2.round() as i32);
                }
            }

            // Also update y2 coordinate for corner resize
            if let Ok(current_y2_percent) = new_bounds.y2.replace('%', "").parse::<f32>() {
                if let Ok(current_y1_percent) = new_bounds.y1.replace('%', "").parse::<f32>() {
                    let pixel_delta_y = delta_y as f32;
                    let percent_delta_y = (pixel_delta_y / terminal_height as f32) * 100.0;
                    let new_y2_percent =
                        (current_y2_percent + percent_delta_y).max(10.0).min(100.0);

                    // Enforce minimum height constraint
                    let min_y2_for_height = current_y1_percent + min_height_percent;
                    let constrained_y2 = new_y2_percent.max(min_y2_for_height);

                    new_bounds.y2 = format!("{}%", constrained_y2.round() as i32);
                }
            }
        }
    }

    new_bounds
}

// F0191: Calculate new muxbox position during drag move
pub fn calculate_new_position(
    original_bounds: &InputBounds,
    start_x: u16,
    start_y: u16,
    current_x: u16,
    current_y: u16,
    terminal_width: usize,
    terminal_height: usize,
) -> InputBounds {
    let delta_x = (current_x as i32) - (start_x as i32);
    let delta_y = (current_y as i32) - (start_y as i32);

    let mut new_bounds = original_bounds.clone();

    // Convert pixel deltas to percentage deltas and update position
    let pixel_delta_x = delta_x as f32;
    let percent_delta_x = (pixel_delta_x / terminal_width as f32) * 100.0;

    let pixel_delta_y = delta_y as f32;
    let percent_delta_y = (pixel_delta_y / terminal_height as f32) * 100.0;

    // Update x1 and x2 (maintain width)
    if let (Ok(current_x1), Ok(current_x2)) = (
        new_bounds.x1.replace('%', "").parse::<f32>(),
        new_bounds.x2.replace('%', "").parse::<f32>(),
    ) {
        let new_x1 = (current_x1 + percent_delta_x).max(0.0).min(90.0);
        let new_x2 = (current_x2 + percent_delta_x).max(10.0).min(100.0);

        // Ensure we don't go beyond boundaries while maintaining muxbox width
        if new_x2 <= 100.0 && new_x1 >= 0.0 {
            new_bounds.x1 = format!("{}%", new_x1.round() as i32);
            new_bounds.x2 = format!("{}%", new_x2.round() as i32);
        }
    }

    // Update y1 and y2 (maintain height)
    if let (Ok(current_y1), Ok(current_y2)) = (
        new_bounds.y1.replace('%', "").parse::<f32>(),
        new_bounds.y2.replace('%', "").parse::<f32>(),
    ) {
        let new_y1 = (current_y1 + percent_delta_y).max(0.0).min(90.0);
        let new_y2 = (current_y2 + percent_delta_y).max(10.0).min(100.0);

        // Ensure we don't go beyond boundaries while maintaining muxbox height
        if new_y2 <= 100.0 && new_y1 >= 0.0 {
            new_bounds.y1 = format!("{}%", new_y1.round() as i32);
            new_bounds.y2 = format!("{}%", new_y2.round() as i32);
        }
    }

    new_bounds
}

// F0188: Helper functions to determine if click is on scroll knob (not just track)
fn is_on_vertical_knob(muxbox: &MuxBox, click_y: usize) -> bool {
    let muxbox_bounds = muxbox.bounds();
    let viewable_height = muxbox_bounds.height().saturating_sub(4);

    // Get content dimensions to calculate knob position and size
    // F0214: Stream-Based Scrollbar Calculations - Use active stream content
    let stream_content = muxbox.get_active_stream_content();
    let stream_choices = muxbox.get_active_stream_choices();
    
    let max_content_height = if !stream_content.is_empty() {
        let mut total_height = stream_content.len();
        // Add choices height if active stream has choices
        if let Some(choices) = stream_choices {
            total_height += choices.len();
        }
        total_height
    } else if let Some(choices) = stream_choices {
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

    let vertical_scroll = muxbox.vertical_scroll.unwrap_or(0.0);
    let knob_position = if available_track > 0 {
        ((vertical_scroll / 100.0) * available_track as f64).round() as usize
    } else {
        0
    };

    // Check if click is within knob bounds
    let knob_start_y = muxbox_bounds.top() + 1 + knob_position;
    let knob_end_y = knob_start_y + knob_size;

    click_y >= knob_start_y && click_y < knob_end_y
}

fn is_on_horizontal_knob(muxbox: &MuxBox, click_x: usize) -> bool {
    let muxbox_bounds = muxbox.bounds();
    let viewable_width = muxbox_bounds.width().saturating_sub(4);

    // Get content width to calculate knob position and size
    // F0214: Stream-Based Scrollbar Calculations - Use active stream content
    let stream_content = muxbox.get_active_stream_content();
    let stream_choices = muxbox.get_active_stream_choices();
    
    let max_content_width = if !stream_content.is_empty() {
        stream_content.iter().map(|line| line.len()).max().unwrap_or(0)
    } else if let Some(choices) = stream_choices {
        choices
            .iter()
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

    let horizontal_scroll = muxbox.horizontal_scroll.unwrap_or(0.0);
    let knob_position = if available_track > 0 {
        ((horizontal_scroll / 100.0) * available_track as f64).round() as usize
    } else {
        0
    };

    // Check if click is within knob bounds
    let knob_start_x = muxbox_bounds.left() + 1 + knob_position;
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
                        Message::ChoiceExecutionComplete(choice_id, muxbox_id, _) => {
                            log::info!(
                                "About to process ChoiceExecutionComplete: {} -> {}",
                                choice_id,
                                muxbox_id
                            );
                        }
                        _ => {}
                    }
                }
            }

            for message in &messages {
                log::trace!("Processing message: {:?}", message);
                match message {
                    Message::MuxBoxEventRefresh(_) => {
                        log::trace!("MuxBoxEventRefresh");
                    }
                    Message::Exit => should_continue = false,
                    Message::Terminate => should_continue = false,
                    Message::NextMuxBox() => {
                        let active_layout = app_context_unwrapped
                            .app
                            .get_active_layout_mut()
                            .expect("No active layout found!");

                        // First, collect the IDs of currently selected muxboxes before changing the selection.
                        let unselected_muxbox_ids: Vec<String> = active_layout
                            .get_selected_muxboxes()
                            .iter()
                            .map(|muxbox| muxbox.id.clone())
                            .collect();

                        // Now perform the mutation that changes the muxbox selection.
                        active_layout.select_next_muxbox();

                        // After mutation, get the newly selected muxboxes' IDs.
                        let selected_muxbox_ids: Vec<String> = active_layout
                            .get_selected_muxboxes()
                            .iter()
                            .map(|muxbox| muxbox.id.clone())
                            .collect();

                        // Update the application context and issue redraw commands based on the collected IDs.
                        inner.update_app_context(app_context_unwrapped.clone());
                        for muxbox_id in unselected_muxbox_ids {
                            inner.send_message(Message::RedrawMuxBox(muxbox_id));
                        }
                        for muxbox_id in selected_muxbox_ids {
                            inner.send_message(Message::RedrawMuxBox(muxbox_id));
                        }
                    }
                    Message::PreviousMuxBox() => {
                        let active_layout = app_context_unwrapped
                            .app
                            .get_active_layout_mut()
                            .expect("No active layout found!");

                        // First, collect the IDs of currently selected muxboxes before changing the selection.
                        let unselected_muxbox_ids: Vec<String> = active_layout
                            .get_selected_muxboxes()
                            .iter()
                            .map(|muxbox| muxbox.id.clone())
                            .collect();

                        // Now perform the mutation that changes the muxbox selection.
                        active_layout.select_previous_muxbox();

                        // After mutation, get the newly selected muxboxes' IDs.
                        let selected_muxbox_ids: Vec<String> = active_layout
                            .get_selected_muxboxes()
                            .iter()
                            .map(|muxbox| muxbox.id.clone())
                            .collect();

                        // Update the application context and issue redraw commands based on the collected IDs.
                        inner.update_app_context(app_context_unwrapped.clone());
                        for muxbox_id in unselected_muxbox_ids {
                            inner.send_message(Message::RedrawMuxBox(muxbox_id));
                        }
                        for muxbox_id in selected_muxbox_ids {
                            inner.send_message(Message::RedrawMuxBox(muxbox_id));
                        }
                    }
                    Message::ScrollMuxBoxDown() => {
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                // F0215: Stream-Based Choice Navigation - Use active stream choices
                                if let Some(choices) = found_muxbox.get_active_stream_choices_mut() {
                                    //select first or next choice
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

                                    // Auto-scroll to keep selected choice visible
                                    auto_scroll_to_selected_choice(found_muxbox, new_selected_choice);
                                } else {
                                    found_muxbox.scroll_down(Some(1.0));
                                }

                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxUp() => {
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                // F0215: Stream-Based Choice Navigation - Use active stream choices  
                                if let Some(choices) = found_muxbox.get_active_stream_choices_mut() {
                                    //select first or next choice
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

                                    // Auto-scroll to keep selected choice visible
                                    auto_scroll_to_selected_choice(found_muxbox, new_selected_choice);
                                } else {
                                    found_muxbox.scroll_up(Some(1.0));
                                }
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxLeft() => {
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                found_muxbox.scroll_left(Some(1.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxRight() => {
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                found_muxbox.scroll_right(Some(1.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxPageUp() => {
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                // Page up scrolls by larger amount (10 units for page-based scrolling)
                                found_muxbox.scroll_up(Some(10.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxPageDown() => {
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                // Page down scrolls by larger amount (10 units for page-based scrolling)
                                found_muxbox.scroll_down(Some(10.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxPageLeft() => {
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                // Page left scrolls by larger amount (10 units for page-based scrolling)
                                found_muxbox.scroll_left(Some(10.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxPageRight() => {
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                // Page right scrolls by larger amount (10 units for page-based scrolling)
                                found_muxbox.scroll_right(Some(10.0));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxToBeginning() => {
                        // Home key: scroll to beginning horizontally (horizontal_scroll = 0)
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                found_muxbox.horizontal_scroll = Some(0.0);
                                
                                // F0200: Save scroll position to YAML
                                inner.send_message(Message::SaveMuxBoxScroll(
                                    found_muxbox.id.clone(),
                                    0,
                                    (found_muxbox.vertical_scroll.unwrap_or(0.0) * 100.0) as usize,
                                ));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxToEnd() => {
                        // End key: scroll to end horizontally (horizontal_scroll = 100)
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                found_muxbox.horizontal_scroll = Some(100.0);
                                
                                // F0200: Save scroll position to YAML
                                inner.send_message(Message::SaveMuxBoxScroll(
                                    found_muxbox.id.clone(),
                                    100,
                                    (found_muxbox.vertical_scroll.unwrap_or(0.0) * 100.0) as usize,
                                ));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxToTop() => {
                        // Ctrl+Home: scroll to top vertically (vertical_scroll = 0)
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                found_muxbox.vertical_scroll = Some(0.0);
                                
                                // F0200: Save scroll position to YAML
                                inner.send_message(Message::SaveMuxBoxScroll(
                                    found_muxbox.id.clone(),
                                    (found_muxbox.horizontal_scroll.unwrap_or(0.0) * 100.0) as usize,
                                    0,
                                ));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::ScrollMuxBoxToBottom() => {
                        // Ctrl+End: scroll to bottom vertically (vertical_scroll = 100)
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                found_muxbox.vertical_scroll = Some(100.0);
                                
                                // F0200: Save scroll position to YAML
                                inner.send_message(Message::SaveMuxBoxScroll(
                                    found_muxbox.id.clone(),
                                    (found_muxbox.horizontal_scroll.unwrap_or(0.0) * 100.0) as usize,
                                    100,
                                ));
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(selected_id));
                            }
                        }
                    }
                    Message::CopyFocusedMuxBoxContent() => {
                        let selected_muxboxes = app_context_unwrapped
                            .app
                            .get_active_layout()
                            .unwrap()
                            .get_selected_muxboxes();
                        if !selected_muxboxes.is_empty() {
                            let selected_id = selected_muxboxes.first().unwrap().id.clone();
                            let muxbox = app_context_unwrapped.app.get_muxbox_by_id(&selected_id);
                            if let Some(found_muxbox) = muxbox {
                                // Get muxbox content to copy
                                let content_to_copy =
                                    get_muxbox_content_for_clipboard(found_muxbox);

                                // Copy to clipboard
                                if copy_to_clipboard(&content_to_copy).is_ok() {
                                    // Trigger visual flash for the muxbox
                                    trigger_muxbox_flash(&selected_id);
                                    inner.send_message(Message::RedrawMuxBox(selected_id));
                                }
                            }
                        }
                    }
                    Message::RedrawMuxBox(muxbox_id) => {
                        if let Some(mut found_muxbox) = app_context_unwrapped
                            .app
                            .get_muxbox_by_id_mut(muxbox_id)
                            .cloned()
                        {
                            new_buffer = buffer.clone();

                            // Clone the parent layout to avoid mutable borrow conflicts
                            if let Some(parent_layout) =
                                found_muxbox.get_parent_layout_clone(&mut app_context_unwrapped)
                            {
                                draw_muxbox(
                                    &app_context_unwrapped,
                                    &app_graph,
                                    &adjusted_bounds,
                                    &parent_layout,
                                    &mut found_muxbox,
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
                    Message::RedrawAppDiff => {
                        // Redraw entire app with diff-based rendering (no screen clear)
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
                    Message::MuxBoxOutputUpdate(muxbox_id, success, output) => {
                        log::info!("RECEIVED MuxBoxOutputUpdate for muxbox: {}, success: {}, output_len: {}, preview: {}", 
                                   muxbox_id, success, output.len(), output.chars().take(50).collect::<String>());
                        let mut app_context_unwrapped_cloned = app_context_unwrapped.clone();
                        // For PTY streaming output, we need to use a special update method
                        // that doesn't add timestamp formatting. The presence of a newline
                        // at the end of the output indicates it's PTY streaming data.
                        let is_pty_streaming = output.ends_with('\n');

                        if is_pty_streaming {
                            // Use streaming update for PTY output
                            let content_for_save = {
                                let target_muxbox = app_context_unwrapped_cloned
                                    .app
                                    .get_muxbox_by_id_mut(muxbox_id)
                                    .unwrap();
                                target_muxbox.update_streaming_content(output, *success);
                                Some(target_muxbox.get_active_stream_content().join("\n"))
                            };
                            inner.update_app_context(app_context_unwrapped_cloned.clone());
                            inner.send_message(Message::RedrawMuxBox(muxbox_id.to_string()));
                            
                            // F0200: Save PTY streaming content to YAML
                            if let Some(updated_content) = content_for_save {
                                inner.send_message(Message::SaveMuxBoxContent(
                                    muxbox_id.clone(),
                                    updated_content,
                                ));
                            }
                        } else {
                            // Use regular update for non-PTY output
                            let muxbox = app_context_unwrapped
                                .app
                                .get_muxbox_by_id(muxbox_id)
                                .unwrap();
                            
                            // Store content before update for YAML persistence
                            let old_content = Some(muxbox.get_active_stream_content().join("\n"));
                            
                            update_muxbox_content(
                                inner,
                                &mut app_context_unwrapped_cloned,
                                muxbox_id,
                                *success,
                                muxbox.append_output.unwrap_or(false),
                                output,
                            );
                            
                            // F0200: Save updated content to YAML if it changed
                            if let Some(updated_muxbox) = app_context_unwrapped_cloned.app.get_muxbox_by_id(muxbox_id) {
                                if Some(updated_muxbox.get_active_stream_content().join("\n")) != old_content {
                                    let new_content = updated_muxbox.get_active_stream_content().join("\n");
                                    if !new_content.is_empty() {
                                        inner.send_message(Message::SaveMuxBoxContent(
                                            muxbox_id.clone(),
                                            new_content,
                                        ));
                                    }
                                }
                            }
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

                        // Find the choice by ID in any muxbox
                        log::info!("Searching for choice {} in active layout", choice_id);
                        if let Some(choice_muxbox) =
                            active_layout.find_muxbox_with_choice(&choice_id)
                        {
                            log::info!("Found choice in muxbox: {}", choice_muxbox.id);

                            if let Some(choices) = choice_muxbox.get_active_stream_choices() {
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
                                            choice_muxbox.id.clone(),
                                            result.map_err(|e| e.to_string()),
                                        ));
                                    }
                                } else {
                                    log::warn!("Choice {} found in muxbox {} but no matching choice in choices list", choice_id, choice_muxbox.id);
                                }
                            } else {
                                log::warn!("MuxBox {} has no choices list", choice_muxbox.id);
                            }
                        } else {
                            log::error!(
                                "Choice {} not found in any muxbox of active layout",
                                choice_id
                            );
                        }
                    }
                    Message::KeyPress(pressed_key) => {
                        let mut app_context_for_keypress = app_context_unwrapped.clone();
                        let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();

                        let selected_muxboxes: Vec<&MuxBox> = active_layout.get_selected_muxboxes();

                        let selected_muxboxes_with_keypress_events: Vec<&MuxBox> =
                            selected_muxboxes
                                .clone()
                                .into_iter()
                                .filter(|p| p.on_keypress.is_some())
                                .filter(|p| p.get_active_stream_choices().is_none())
                                .collect();

                        let libs = app_context_unwrapped.app.libs.clone();

                        if pressed_key == "Enter" {
                            let selected_muxboxes_with_choices: Vec<&MuxBox> = selected_muxboxes
                                .into_iter()
                                .filter(|p| p.get_active_stream_choices().is_some())
                                .collect();
                            for muxbox in selected_muxboxes_with_choices {
                                // First, extract choice information before any mutable operations
                                let (selected_choice_data, choice_needs_execution) = {
                                    let muxbox_ref = app_context_for_keypress
                                        .app
                                        .get_muxbox_by_id(&muxbox.id)
                                        .unwrap();
                                    if let Some(choices) = muxbox_ref.get_active_stream_choices() {
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
                                                muxbox.id.clone(),
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
                                    muxbox_id,
                                )) = selected_choice_data
                                {
                                    if choice_needs_execution {
                                        log::info!(
                                            "=== ENTER KEY CHOICE EXECUTION: {} (muxbox: {}) ===",
                                            choice_id,
                                            muxbox_id
                                        );
                                        log::info!("Enter choice config - pty: {}, thread: {}, redirect: {:?}", 
                                            use_pty, use_thread, redirect_output
                                        );

                                        if let Some(script) = script_opt {
                                            let libs_clone = libs.clone();

                                            // T312: Execute choice using unified threading system - proper architecture
                                            log::info!("Enter key requesting ThreadManager to execute choice {} (pty: {})", choice_id, use_pty);

                                            // Set choice to waiting state before execution
                                            if let Some(muxbox_mut) = app_context_for_keypress
                                                .app
                                                .get_muxbox_by_id_mut(&muxbox_id)
                                            {
                                                if let Some(choices) = muxbox_mut.get_active_stream_choices_mut() {
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
                                                muxbox_id.clone(),
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

                        for muxbox in selected_muxboxes_with_keypress_events {
                            let actions =
                                handle_keypress(pressed_key, &muxbox.on_keypress.clone().unwrap());
                            if actions.is_none() {
                                if let Some(actions_unwrapped) = actions {
                                    let libs = app_context_unwrapped.app.libs.clone();

                                    match run_script(libs, &actions_unwrapped) {
                                        Ok(output) => {
                                            if muxbox.redirect_output.is_some() {
                                                update_muxbox_content(
                                                    inner,
                                                    &mut app_context_for_keypress,
                                                    muxbox.redirect_output.as_ref().unwrap(),
                                                    true,
                                                    muxbox.append_output.unwrap_or(false),
                                                    &output,
                                                )
                                            } else {
                                                update_muxbox_content(
                                                    inner,
                                                    &mut app_context_for_keypress,
                                                    &muxbox.id,
                                                    true,
                                                    muxbox.append_output.unwrap_or(false),
                                                    &output,
                                                )
                                            }
                                        }
                                        Err(e) => {
                                            if muxbox.redirect_output.is_some() {
                                                update_muxbox_content(
                                                    inner,
                                                    &mut app_context_for_keypress,
                                                    muxbox.redirect_output.as_ref().unwrap(),
                                                    false,
                                                    muxbox.append_output.unwrap_or(false),
                                                    e.to_string().as_str(),
                                                )
                                            } else {
                                                update_muxbox_content(
                                                    inner,
                                                    &mut app_context_for_keypress,
                                                    &muxbox.id,
                                                    false,
                                                    muxbox.append_output.unwrap_or(false),
                                                    e.to_string().as_str(),
                                                )
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Message::PTYInput(muxbox_id, input) => {
                        log::trace!("PTY input for muxbox {}: {}", muxbox_id, input);

                        // Find the target muxbox to verify it exists and has PTY enabled
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id(muxbox_id)
                        {
                            if muxbox.pty.unwrap_or(false) {
                                log::debug!(
                                    "Routing input to PTY muxbox {}: {:?}",
                                    muxbox_id,
                                    input.chars().collect::<Vec<_>>()
                                );

                                // TODO: Write input to PTY process when PTY manager is thread-safe
                                // For now, log the successful routing detection
                                log::info!(
                                    "PTY input ready for routing to muxbox {}: {} chars",
                                    muxbox_id,
                                    input.len()
                                );
                            } else {
                                log::warn!(
                                    "MuxBox {} received PTY input but pty field is false",
                                    muxbox_id
                                );
                            }
                        } else {
                            log::error!(
                                "PTY input received for non-existent muxbox: {}",
                                muxbox_id
                            );
                        }
                    }
                    Message::MouseClick(x, y) => {
                        log::trace!("Mouse click at ({}, {})", x, y);
                        let mut app_context_for_click = app_context_unwrapped.clone();
                        let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();

                        // F0187: Check for scrollbar clicks first
                        let mut handled_scrollbar_click = false;
                        for muxbox in active_layout.get_all_muxboxes() {
                            if muxbox.has_scrollable_content() {
                                let muxbox_bounds = muxbox.bounds();

                                // Check for vertical scrollbar click (right border)
                                if *x as usize == muxbox_bounds.right()
                                    && *y as usize > muxbox_bounds.top()
                                    && (*y as usize) < muxbox_bounds.bottom()
                                {
                                    let track_height =
                                        (muxbox_bounds.height() as isize - 2).max(1) as usize;
                                    let click_position = ((*y as usize) - muxbox_bounds.top() - 1)
                                        as f64
                                        / track_height as f64;
                                    let scroll_percentage =
                                        (click_position * 100.0).min(100.0).max(0.0);

                                    log::trace!(
                                        "Vertical scrollbar click on muxbox {} at {}%",
                                        muxbox.id,
                                        scroll_percentage
                                    );

                                    // Update muxbox vertical scroll
                                    let (muxbox_id, horizontal_scroll) = {
                                        let muxbox_to_update = app_context_for_click
                                            .app
                                            .get_muxbox_by_id_mut(&muxbox.id)
                                            .unwrap();
                                        muxbox_to_update.vertical_scroll = Some(scroll_percentage);
                                        (muxbox_to_update.id.clone(), muxbox_to_update.horizontal_scroll.unwrap_or(0.0))
                                    };

                                    inner.update_app_context(app_context_for_click.clone());
                                    inner.send_message(Message::RedrawAppDiff);
                                    handled_scrollbar_click = true;
                                    
                                    // F0200: Save scroll position to YAML
                                    inner.send_message(Message::SaveMuxBoxScroll(
                                        muxbox_id,
                                        (horizontal_scroll * 100.0) as usize,
                                        (scroll_percentage * 100.0) as usize,
                                    ));
                                    break;
                                }

                                // Check for horizontal scrollbar click (bottom border)
                                if *y as usize == muxbox_bounds.bottom()
                                    && *x as usize > muxbox_bounds.left()
                                    && (*x as usize) < muxbox_bounds.right()
                                {
                                    let track_width =
                                        (muxbox_bounds.width() as isize - 2).max(1) as usize;
                                    let click_position = ((*x as usize) - muxbox_bounds.left() - 1)
                                        as f64
                                        / track_width as f64;
                                    let scroll_percentage =
                                        (click_position * 100.0).min(100.0).max(0.0);

                                    log::trace!(
                                        "Horizontal scrollbar click on muxbox {} at {}%",
                                        muxbox.id,
                                        scroll_percentage
                                    );

                                    // Update muxbox horizontal scroll
                                    let (muxbox_id, vertical_scroll) = {
                                        let muxbox_to_update = app_context_for_click
                                            .app
                                            .get_muxbox_by_id_mut(&muxbox.id)
                                            .unwrap();
                                        muxbox_to_update.horizontal_scroll = Some(scroll_percentage);
                                        (muxbox_to_update.id.clone(), muxbox_to_update.vertical_scroll.unwrap_or(0.0))
                                    };

                                    inner.update_app_context(app_context_for_click.clone());
                                    inner.send_message(Message::RedrawAppDiff);
                                    handled_scrollbar_click = true;
                                    
                                    // F0200: Save scroll position to YAML
                                    inner.send_message(Message::SaveMuxBoxScroll(
                                        muxbox_id,
                                        (scroll_percentage * 100.0) as usize,
                                        (vertical_scroll * 100.0) as usize,
                                    ));
                                    break;
                                }
                            }
                        }

                        // If scrollbar click was handled, skip muxbox selection
                        if handled_scrollbar_click {
                            // Continue to next message
                        } else {
                            // F0203: Check for tab clicks first
                            let mut handled_tab_click = false;
                            for muxbox in active_layout.get_all_muxboxes() {
                                let muxbox_bounds = muxbox.bounds();
                                
                                // Check if click is in title bar area (all boxes have tabs)
                                if *y as usize == muxbox_bounds.top() && 
                                   *x as usize >= muxbox_bounds.left() && 
                                   *x as usize <= muxbox_bounds.right() {
                                    
                                    let tab_labels = muxbox.get_tab_labels();
                                    log::debug!("Title bar click at ({},{}) in muxbox '{}' with {} tabs: {:?}", 
                                        *x, *y, muxbox.id, tab_labels.len(), tab_labels);
                                    
                                    let has_border = muxbox.calc_border(&app_context_unwrapped.clone(), &app_graph);
                                    log::debug!("Muxbox bounds: left={}, right={}, top={}, border={}", 
                                        muxbox_bounds.left(), muxbox_bounds.right(), muxbox_bounds.top(), has_border);
                                    
                                    if let Some(clicked_tab_index) = crate::draw_utils::calculate_tab_click_index(
                                        *x as usize, 
                                        muxbox_bounds.left(), 
                                        muxbox_bounds.right(),
                                        &tab_labels,
                                        has_border
                                    ) {
                                        log::info!("Tab click detected: muxbox {} tab {} ({})", 
                                            muxbox.id, clicked_tab_index, 
                                            tab_labels.get(clicked_tab_index).unwrap_or(&"unknown".to_string()));
                                        
                                        log::info!("Processing SwitchTab directly: muxbox={}, tab_index={}", muxbox.id, clicked_tab_index);
                                        if let Some(muxbox) = app_context_for_click.app.get_muxbox_by_id_mut(&muxbox.id) {
                                            if muxbox.switch_to_tab(clicked_tab_index) {
                                                log::info!("Successfully switched muxbox '{}' to tab {}", muxbox.id, clicked_tab_index);
                                                inner.update_app_context(app_context_for_click.clone());
                                            } else {
                                                log::warn!("Failed to switch muxbox '{}' to tab {} - switch_to_tab returned false", muxbox.id, clicked_tab_index);
                                            }
                                        }
                                        handled_tab_click = true;
                                        break;
                                    } else {
                                        log::debug!("Click in title bar but not on tab area - allowing move/drag operation");
                                    }
                                }
                            }
                            
                            if !handled_tab_click {
                            // F0091: Find which muxbox was clicked based on coordinates
                            if let Some(clicked_muxbox) =
                                active_layout.find_muxbox_at_coordinates(*x, *y)
                            {
                                log::trace!("Clicked on muxbox: {}", clicked_muxbox.id);

                                // Check if muxbox has choices (menu items)
                                if let Some(choices) = clicked_muxbox.get_active_stream_choices() {
                                    // Calculate which choice was clicked based on y and x offset within muxbox
                                    if let Some(clicked_choice_idx) = calculate_clicked_choice_index(
                                        clicked_muxbox,
                                        *x,
                                        *y,
                                        choices,
                                    ) {
                                        if let Some(clicked_choice) =
                                            choices.get(clicked_choice_idx)
                                        {
                                            log::trace!("Clicked on choice: {}", clicked_choice.id);

                                            // First, select the parent muxbox if not already selected
                                            let layout = app_context_for_click
                                                .app
                                                .get_active_layout_mut()
                                                .unwrap();
                                            layout.deselect_all_muxboxes();
                                            layout.select_only_muxbox(&clicked_muxbox.id);

                                            // Then select the clicked choice visually
                                            let muxbox_to_update = app_context_for_click
                                                .app
                                                .get_muxbox_by_id_mut(&clicked_muxbox.id)
                                                .unwrap();
                                            if let Some(muxbox_choices) =
                                                muxbox_to_update.get_active_stream_choices_mut()
                                            {
                                                // Deselect all choices first
                                                for choice in muxbox_choices.iter_mut() {
                                                    choice.selected = false;
                                                }
                                                // Select only the clicked choice
                                                if let Some(selected_choice) =
                                                    muxbox_choices.get_mut(clicked_choice_idx)
                                                {
                                                    selected_choice.selected = true;
                                                }
                                            }

                                            // Update the app context and immediately trigger redraw for responsiveness
                                            inner.update_app_context(app_context_for_click.clone());
                                            inner.send_message(Message::RedrawAppDiff);

                                            // Then activate the clicked choice (same as pressing Enter)
                                            // Force threaded execution for clicked choices to maintain UI responsiveness
                                            if let Some(script) = &clicked_choice.script {
                                                let libs = app_context_unwrapped.app.libs.clone();

                                                // Always use threaded execution for mouse clicks to keep UI responsive
                                                let _script_clone = script.clone();
                                                let _choice_id_clone = clicked_choice.id.clone();
                                                let muxbox_id_clone = clicked_muxbox.id.clone();
                                                let libs_clone = libs.clone();

                                                // T312: Use unified ExecuteChoice message system
                                                inner.send_message(Message::ExecuteChoice(
                                                    clicked_choice.clone(),
                                                    muxbox_id_clone,
                                                    libs_clone,
                                                ));

                                                // Spawn the choice execution in ThreadManager
                                                // TODO: Get ThreadManager reference to spawn the runnable
                                                log::trace!("Mouse click choice {} ready for ThreadManager execution", clicked_choice.id);
                                            }
                                        }
                                    } else {
                                        // Click was on muxbox with choices but not on any specific choice
                                        // Only select the muxbox, don't activate any choice
                                        if clicked_muxbox.tab_order.is_some()
                                            || clicked_muxbox.has_scrollable_content()
                                        {
                                            log::trace!(
                                                "Selecting muxbox (clicked on empty area): {}",
                                                clicked_muxbox.id
                                            );

                                            // Deselect all muxboxes in the layout first
                                            let layout = app_context_for_click
                                                .app
                                                .get_active_layout_mut()
                                                .unwrap();
                                            layout.deselect_all_muxboxes();
                                            layout.select_only_muxbox(&clicked_muxbox.id);

                                            inner.update_app_context(app_context_for_click);
                                            inner.send_message(Message::RedrawAppDiff);
                                        }
                                    }
                                } else {
                                    // MuxBox has no choices - just select it if it's selectable
                                    if clicked_muxbox.tab_order.is_some()
                                        || clicked_muxbox.has_scrollable_content()
                                    {
                                        log::trace!(
                                            "Selecting muxbox (no choices): {}",
                                            clicked_muxbox.id
                                        );

                                        // Deselect all muxboxes in the layout first
                                        let layout = app_context_for_click
                                            .app
                                            .get_active_layout_mut()
                                            .unwrap();
                                        layout.deselect_all_muxboxes();
                                        layout.select_only_muxbox(&clicked_muxbox.id);

                                        inner.update_app_context(app_context_for_click);
                                        inner.send_message(Message::RedrawAppDiff);
                                    }
                                }
                            }
                            } // End of !handled_tab_click
                        }
                    }
                    Message::MouseDragStart(x, y) => {
                        // Check if muxboxes are locked before allowing resize/move
                        if app_context_unwrapped.config.locked {
                            // Skip all resize/move operations when locked
                            log::trace!("MuxBox resize/move blocked: muxboxes are locked");
                        } else {
                            // F0189: Check if drag started on a muxbox border first
                            let active_layout =
                                app_context_unwrapped.app.get_active_layout().unwrap();
                            let mut resize_state = MUXBOX_RESIZE_STATE.lock().unwrap();
                            *resize_state = None; // Clear any previous resize state

                            // Check for muxbox border resize first
                            let mut handled_resize = false;
                            for muxbox in active_layout.get_all_muxboxes() {
                                if let Some(resize_edge) = detect_resize_edge(muxbox, *x, *y) {
                                    *resize_state = Some(MuxBoxResizeState {
                                        muxbox_id: muxbox.id.clone(),
                                        resize_edge,
                                        start_x: *x,
                                        start_y: *y,
                                        original_bounds: muxbox.position.clone(),
                                    });
                                    log::trace!(
                                        "Started resizing muxbox {} via {:?} edge",
                                        muxbox.id,
                                        resize_state.as_ref().unwrap().resize_edge
                                    );
                                    handled_resize = true;
                                    break;
                                }
                            }

                            // F0191: If not a resize, check if drag started on muxbox title/top border for movement
                            let mut handled_move = false;
                            if !handled_resize {
                                let mut move_state = MUXBOX_MOVE_STATE.lock().unwrap();
                                *move_state = None; // Clear any previous move state

                                for muxbox in active_layout.get_all_muxboxes() {
                                    if detect_move_area(muxbox, *x, *y) {
                                        // Check if the drag started on a tab area - if so, don't start move
                                        let tab_labels = muxbox.get_tab_labels();
                                        let muxbox_bounds = muxbox.bounds();
                                        if let Some(_tab_index) = crate::draw_utils::calculate_tab_click_index(
                                            *x as usize, 
                                            muxbox_bounds.left(), 
                                            muxbox_bounds.right(),
                                            &tab_labels,
                                            muxbox.calc_border(&app_context_unwrapped.clone(), &app_graph)
                                        ) {
                                            log::trace!("Drag started on tab area for muxbox {} - skipping move operation", muxbox.id);
                                            // Skip move operation for tab area drags
                                            continue;
                                        }
                                        
                                        *move_state = Some(MuxBoxMoveState {
                                            muxbox_id: muxbox.id.clone(),
                                            start_x: *x,
                                            start_y: *y,
                                            original_bounds: muxbox.position.clone(),
                                        });
                                        log::trace!(
                                            "Started moving muxbox {} via title/top border",
                                            muxbox.id
                                        );
                                        handled_move = true;
                                        break;
                                    }
                                }
                            }
                        }

                        // F0188: Check for scroll knob drag (allowed even when locked)
                        let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();

                        // Check if any resize/move states are active (only possible when unlocked)
                        let has_active_resize = if !app_context_unwrapped.config.locked {
                            let resize_state_guard = MUXBOX_RESIZE_STATE.lock().unwrap();
                            resize_state_guard.is_some()
                        } else {
                            false
                        };

                        let has_active_move = if !app_context_unwrapped.config.locked {
                            let move_state_guard = MUXBOX_MOVE_STATE.lock().unwrap();
                            move_state_guard.is_some()
                        } else {
                            false
                        };

                        // F0188: If no resize or move is active, check if drag started on a scroll knob
                        if !has_active_resize && !has_active_move {
                            let mut drag_state = DRAG_STATE.lock().unwrap();
                            *drag_state = None; // Clear any previous drag state

                            for muxbox in active_layout.get_all_muxboxes() {
                                if muxbox.has_scrollable_content() {
                                    let muxbox_bounds = muxbox.bounds();

                                    // Check if drag started on vertical scroll knob
                                    if *x as usize == muxbox_bounds.right()
                                        && *y as usize > muxbox_bounds.top()
                                        && (*y as usize) < muxbox_bounds.bottom()
                                    {
                                        // Check if we clicked on the actual knob, not just the track
                                        if is_on_vertical_knob(muxbox, *y as usize) {
                                            let current_scroll =
                                                muxbox.vertical_scroll.unwrap_or(0.0);
                                            *drag_state = Some(DragState {
                                                muxbox_id: muxbox.id.clone(),
                                                is_vertical: true,
                                                start_x: *x,
                                                start_y: *y,
                                                start_scroll_percentage: current_scroll,
                                            });
                                            log::trace!("Started dragging vertical scroll knob on muxbox {}", muxbox.id);
                                            break;
                                        }
                                    }

                                    // Check if drag started on horizontal scroll knob
                                    if *y as usize == muxbox_bounds.bottom()
                                        && *x as usize > muxbox_bounds.left()
                                        && (*x as usize) < muxbox_bounds.right()
                                    {
                                        // Check if we clicked on the actual knob, not just the track
                                        if is_on_horizontal_knob(muxbox, *x as usize) {
                                            let current_scroll =
                                                muxbox.horizontal_scroll.unwrap_or(0.0);
                                            *drag_state = Some(DragState {
                                                muxbox_id: muxbox.id.clone(),
                                                is_vertical: false,
                                                start_x: *x,
                                                start_y: *y,
                                                start_scroll_percentage: current_scroll,
                                            });
                                            log::trace!("Started dragging horizontal scroll knob on muxbox {}", muxbox.id);
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Message::MouseDrag(x, y) => {
                        // Skip resize/move operations when muxboxes are locked
                        if !app_context_unwrapped.config.locked {
                            // F0189: Handle muxbox border resize during drag
                            let resize_state_guard = MUXBOX_RESIZE_STATE.lock().unwrap();
                            if let Some(ref resize_state) = *resize_state_guard {
                                let terminal_width = crate::screen_width();
                                let terminal_height = crate::screen_height();

                                // FIXED: Handle 100% width panels where horizontal drag events may not work
                                let (effective_x, effective_y) =
                                    if resize_state.original_bounds.x2 == "100%" {
                                        // For 100% width panels at rightmost edge, if no horizontal movement is detected,
                                        // use the vertical movement as a proxy for horizontal movement to enable resizing
                                        let horizontal_delta =
                                            (*x as i32) - (resize_state.start_x as i32);
                                        let vertical_delta =
                                            (*y as i32) - (resize_state.start_y as i32);

                                        if horizontal_delta == 0 && vertical_delta != 0 {
                                            // No horizontal movement detected but vertical movement exists
                                            // Use diagonal movement: apply vertical delta to horizontal as well
                                            let adjusted_x =
                                                resize_state.start_x as i32 + vertical_delta;
                                            (adjusted_x.max(0) as u16, *y)
                                        } else {
                                            (*x, *y)
                                        }
                                    } else {
                                        (*x, *y)
                                    };

                                let new_bounds = calculate_new_bounds(
                                    &resize_state.original_bounds,
                                    &resize_state.resize_edge,
                                    resize_state.start_x,
                                    resize_state.start_y,
                                    effective_x,
                                    effective_y,
                                    terminal_width,
                                    terminal_height,
                                );

                                // Update the muxbox bounds in real-time
                                if let Some(muxbox) = app_context_unwrapped
                                    .app
                                    .get_muxbox_by_id_mut(&resize_state.muxbox_id)
                                {
                                    muxbox.position = new_bounds;
                                    inner.update_app_context(app_context_unwrapped.clone());
                                    inner.send_message(Message::RedrawAppDiff);
                                }
                            }

                            // F0191: Handle muxbox movement during drag
                            let move_state_guard = MUXBOX_MOVE_STATE.lock().unwrap();
                            if let Some(ref move_state) = *move_state_guard {
                                let terminal_width = crate::screen_width();
                                let terminal_height = crate::screen_height();

                                let new_position = calculate_new_position(
                                    &move_state.original_bounds,
                                    move_state.start_x,
                                    move_state.start_y,
                                    *x,
                                    *y,
                                    terminal_width,
                                    terminal_height,
                                );

                                // Update the muxbox position in real-time
                                if let Some(muxbox) = app_context_unwrapped
                                    .app
                                    .get_muxbox_by_id_mut(&move_state.muxbox_id)
                                {
                                    muxbox.position = new_position;
                                    inner.update_app_context(app_context_unwrapped.clone());
                                    inner.send_message(Message::RedrawAppDiff);
                                }
                            }
                        }

                        // F0188: Handle scroll knob drag (always allowed, even when locked)
                        let drag_state_guard = DRAG_STATE.lock().unwrap();
                        if let Some(ref drag_state) = *drag_state_guard {
                            let muxbox_to_update = app_context_unwrapped
                                .app
                                .get_muxbox_by_id_mut(&drag_state.muxbox_id);

                            if let Some(muxbox) = muxbox_to_update {
                                let muxbox_bounds = muxbox.bounds();

                                if drag_state.is_vertical {
                                    // Calculate new vertical scroll percentage based on drag distance
                                    let track_height =
                                        (muxbox_bounds.height() as isize - 2).max(1) as usize;
                                    let drag_delta = (*y as isize) - (drag_state.start_y as isize);
                                    let percentage_delta =
                                        (drag_delta as f64 / track_height as f64) * 100.0;
                                    let new_percentage = (drag_state.start_scroll_percentage
                                        + percentage_delta)
                                        .min(100.0)
                                        .max(0.0);

                                    muxbox.vertical_scroll = Some(new_percentage);
                                } else {
                                    // Calculate new horizontal scroll percentage based on drag distance
                                    let track_width =
                                        (muxbox_bounds.width() as isize - 2).max(1) as usize;
                                    let drag_delta = (*x as isize) - (drag_state.start_x as isize);
                                    let percentage_delta =
                                        (drag_delta as f64 / track_width as f64) * 100.0;
                                    let new_percentage = (drag_state.start_scroll_percentage
                                        + percentage_delta)
                                        .min(100.0)
                                        .max(0.0);

                                    muxbox.horizontal_scroll = Some(new_percentage);
                                }

                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawAppDiff);
                            }
                        }
                    }
                    Message::MouseDragEnd(x, y) => {
                        // Only handle resize/move end when muxboxes are unlocked
                        if !app_context_unwrapped.config.locked {
                            // F0189: End muxbox resize operation
                            let mut resize_state = MUXBOX_RESIZE_STATE.lock().unwrap();
                            if let Some(ref resize_state_data) = *resize_state {
                                log::trace!(
                                    "Ended muxbox resize at ({}, {}) for muxbox {}",
                                    x,
                                    y,
                                    resize_state_data.muxbox_id
                                );

                                // Trigger YAML persistence
                                inner.send_message(Message::MuxBoxResizeComplete(
                                    resize_state_data.muxbox_id.clone(),
                                ));
                                *resize_state = None; // Clear resize state
                            } else {
                                // F0191: End muxbox move operation
                                let mut move_state = MUXBOX_MOVE_STATE.lock().unwrap();
                                if let Some(ref move_state_data) = *move_state {
                                    log::trace!(
                                        "Ended muxbox move at ({}, {}) for muxbox {}",
                                        x,
                                        y,
                                        move_state_data.muxbox_id
                                    );

                                    // Trigger YAML persistence for new position
                                    inner.send_message(Message::MuxBoxMoveComplete(
                                        move_state_data.muxbox_id.clone(),
                                    ));
                                    *move_state = None; // Clear move state
                                }
                            }
                        }

                        // F0188: End scroll knob drag operation (always allowed, even when locked)
                        let mut drag_state = DRAG_STATE.lock().unwrap();
                        if drag_state.is_some() {
                            log::trace!("Ended scroll knob drag at ({}, {})", x, y);
                            *drag_state = None; // Clear drag state
                        }
                    }
                    Message::ChoiceExecutionComplete(choice_id, muxbox_id, result) => {
                        log::info!(
                            "=== DRAWLOOP RECEIVED CHOICE EXECUTION COMPLETE: {} on muxbox {} ===",
                            choice_id,
                            muxbox_id
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
                        if let Some(muxbox) =
                            app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                        {
                            if let Some(choices) = muxbox.get_active_stream_choices_mut() {
                                if let Some(choice) =
                                    choices.iter_mut().find(|c| c.id == *choice_id)
                                {
                                    choice.waiting = false;
                                }
                            }
                        }

                        // Then handle the output in a separate scope to avoid borrow conflicts
                        let target_muxbox_id = {
                            if let Some(muxbox) =
                                app_context_unwrapped.app.get_muxbox_by_id(muxbox_id)
                            {
                                if let Some(choices) = muxbox.get_active_stream_choices() {
                                    if let Some(choice) =
                                        choices.iter().find(|c| c.id == *choice_id)
                                    {
                                        let redirect_target = choice
                                            .redirect_output
                                            .as_ref()
                                            .unwrap_or(muxbox_id)
                                            .clone();
                                        log::info!(
                                            "Choice {} redirect_output: {:?} -> target muxbox: {}",
                                            choice_id,
                                            choice.redirect_output,
                                            redirect_target
                                        );
                                        redirect_target
                                    } else {
                                        log::warn!(
                                            "Choice {} not found in muxbox {}",
                                            choice_id,
                                            muxbox_id
                                        );
                                        muxbox_id.clone()
                                    }
                                } else {
                                    log::warn!("MuxBox {} has no choices", muxbox_id);
                                    muxbox_id.clone()
                                }
                            } else {
                                log::error!("MuxBox {} not found", muxbox_id);
                                muxbox_id.clone()
                            }
                        };

                        let append = {
                            if let Some(muxbox) =
                                app_context_unwrapped.app.get_muxbox_by_id(muxbox_id)
                            {
                                if let Some(choices) = muxbox.get_active_stream_choices() {
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

                        // F0203: Create stream in target muxbox's tab system for ALL choice executions
                        let is_redirect = &target_muxbox_id != muxbox_id;
                        log::info!(
                            "Creating stream for choice {} in muxbox {} (redirect: {})",
                            choice_id,
                            target_muxbox_id,
                            is_redirect
                        );
                        
                        // Get choice details first (immutable borrow)
                        let choice_details = if let Some(source_muxbox) = app_context_unwrapped.app.get_muxbox_by_id(muxbox_id) {
                            if let Some(choices) = source_muxbox.get_active_stream_choices() {
                                choices.iter().find(|c| c.id == *choice_id).map(|choice| {
                                    (choice.id.clone(), choice.content.as_deref().unwrap_or(&choice.id).to_string())
                                })
                            } else { None }
                        } else { None };
                        
                        // Create stream in target muxbox (mutable borrow) - works for both redirect and local
                        if let Some((choice_id_clone, stream_label)) = choice_details {
                            if let Some(target_muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(&target_muxbox_id) {
                                // Initialize default stream if tab system is empty
                                target_muxbox.ensure_tabs_initialized();
                                
                                // Create choice execution stream
                                let stream_id = target_muxbox.add_input_stream(
                                    crate::model::common::StreamType::RedirectedOutput(choice_id_clone),
                                    stream_label
                                );
                                
                                log::info!(
                                    "Created stream {} in target muxbox {} for choice execution",
                                    stream_id,
                                    target_muxbox_id
                                );
                            }
                        }

                        match result {
                            Ok(output) => {
                                log::info!(
                                    "Choice {} output length: {} chars, updating stream in muxbox: {}",
                                    choice_id,
                                    output.len(),
                                    target_muxbox_id
                                );
                                update_muxbox_content_with_stream(
                                    inner,
                                    &mut app_context_unwrapped,
                                    &target_muxbox_id,
                                    choice_id, // Use choice_id as stream identifier
                                    true,
                                    append,
                                    output,
                                );
                            }
                            Err(error) => {
                                log::error!("Error running choice script: {}", error);
                                update_muxbox_content_with_stream(
                                    inner,
                                    &mut app_context_unwrapped,
                                    &target_muxbox_id,
                                    choice_id, // Use choice_id as stream identifier  
                                    false,
                                    append,
                                    error,
                                );
                            }
                        }
                    }
                    Message::MuxBoxResizeComplete(muxbox_id) => {
                        // F0190: Save muxbox bounds changes to YAML file
                        log::info!(
                            "Saving muxbox resize changes to YAML for muxbox: {}",
                            muxbox_id
                        );

                        // Get the updated muxbox bounds
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id(muxbox_id)
                        {
                            let new_bounds = &muxbox.position;
                            log::debug!(
                                "New bounds for muxbox {}: x1={}, y1={}, x2={}, y2={}",
                                muxbox_id,
                                new_bounds.x1,
                                new_bounds.y1,
                                new_bounds.x2,
                                new_bounds.y2
                            );

                            // Find the original YAML file path
                            if let Some(yaml_path) = &app_context_unwrapped.yaml_file_path {
                                match save_muxbox_bounds_to_yaml(yaml_path, muxbox_id, new_bounds) {
                                    Ok(()) => {
                                        log::info!(
                                            "Successfully saved muxbox {} bounds to YAML file",
                                            muxbox_id
                                        );
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "Failed to save muxbox {} bounds to YAML: {}",
                                            muxbox_id,
                                            e
                                        );
                                    }
                                }
                            } else {
                                log::error!("CRITICAL: No YAML file path available for saving muxbox bounds - resize changes will not persist!");
                            }
                        } else {
                            log::error!("MuxBox {} not found for saving bounds", muxbox_id);
                        }
                    }
                    Message::MuxBoxMoveComplete(muxbox_id) => {
                        // F0191: Save muxbox position changes to YAML file
                        log::info!(
                            "Saving muxbox move changes to YAML for muxbox: {}",
                            muxbox_id
                        );

                        // Get the updated muxbox position
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id(muxbox_id)
                        {
                            let new_position = &muxbox.position;
                            log::debug!(
                                "New position for muxbox {}: x1={}, y1={}, x2={}, y2={}",
                                muxbox_id,
                                new_position.x1,
                                new_position.y1,
                                new_position.x2,
                                new_position.y2
                            );

                            // Find the original YAML file path
                            if let Some(yaml_path) = &app_context_unwrapped.yaml_file_path {
                                match save_muxbox_bounds_to_yaml(yaml_path, muxbox_id, new_position)
                                {
                                    Ok(()) => {
                                        log::info!(
                                            "Successfully saved muxbox {} position to YAML file",
                                            muxbox_id
                                        );
                                    }
                                    Err(e) => {
                                        log::error!(
                                            "Failed to save muxbox {} position to YAML: {}",
                                            muxbox_id,
                                            e
                                        );
                                    }
                                }
                            } else {
                                log::error!("CRITICAL: No YAML file path available for saving muxbox position - move changes will not persist!");
                            }
                        } else {
                            log::error!("MuxBox {} not found for saving position", muxbox_id);
                        }
                    }
                    Message::SaveYamlState => {
                        // F0200: Save complete application state to YAML
                        log::info!("Saving complete application state to YAML");

                        if let Some(yaml_path) = &app_context_unwrapped.yaml_file_path {
                            match save_complete_state_to_yaml(yaml_path, &app_context_unwrapped) {
                                Ok(()) => {
                                    log::info!("Successfully saved complete state to YAML file");
                                }
                                Err(e) => {
                                    log::error!("Failed to save complete state to YAML: {}", e);
                                }
                            }
                        } else {
                            log::error!("CRITICAL: No YAML file path available for saving complete state!");
                        }
                    }
                    Message::SaveActiveLayout(layout_id) => {
                        // F0200: Save active layout to YAML
                        log::info!("Saving active layout '{}' to YAML", layout_id);

                        if let Some(yaml_path) = &app_context_unwrapped.yaml_file_path {
                            match save_active_layout_to_yaml(yaml_path, layout_id) {
                                Ok(()) => {
                                    log::info!("Successfully saved active layout to YAML file");
                                }
                                Err(e) => {
                                    log::error!("Failed to save active layout to YAML: {}", e);
                                }
                            }
                        } else {
                            log::error!("CRITICAL: No YAML file path available for saving active layout!");
                        }
                    }
                    Message::SaveMuxBoxContent(muxbox_id, content) => {
                        // F0200: Save muxbox content changes to YAML
                        log::debug!("Saving content changes to YAML for muxbox: {}", muxbox_id);

                        if let Some(yaml_path) = &app_context_unwrapped.yaml_file_path {
                            match save_muxbox_content_to_yaml(yaml_path, muxbox_id, content) {
                                Ok(()) => {
                                    log::debug!("Successfully saved muxbox {} content to YAML", muxbox_id);
                                }
                                Err(e) => {
                                    log::error!("Failed to save muxbox {} content to YAML: {}", muxbox_id, e);
                                }
                            }
                        } else {
                            log::warn!("No YAML file path available for saving muxbox content");
                        }
                    }
                    Message::SaveMuxBoxScroll(muxbox_id, scroll_x, scroll_y) => {
                        // F0200: Save muxbox scroll position to YAML
                        log::debug!("Saving scroll position to YAML for muxbox: {} ({}, {})", muxbox_id, scroll_x, scroll_y);

                        if let Some(yaml_path) = &app_context_unwrapped.yaml_file_path {
                            match save_muxbox_scroll_to_yaml(yaml_path, muxbox_id, *scroll_x, *scroll_y) {
                                Ok(()) => {
                                    log::debug!("Successfully saved muxbox {} scroll position to YAML", muxbox_id);
                                }
                                Err(e) => {
                                    log::error!("Failed to save muxbox {} scroll position to YAML: {}", muxbox_id, e);
                                }
                            }
                        } else {
                            log::warn!("No YAML file path available for saving muxbox scroll position");
                        }
                    }
                    Message::SwitchActiveLayout(layout_id) => {
                        // F0200: Switch active layout with YAML persistence
                        log::info!("Switching to active layout: {}", layout_id);
                        
                        // Update the active layout in app context
                        let mut app_context_cloned = app_context_unwrapped.clone();
                        match app_context_cloned.app.set_active_layout_with_yaml_save(
                            layout_id,
                            app_context_cloned.yaml_file_path.as_deref()
                        ) {
                            Ok(()) => {
                                inner.update_app_context(app_context_cloned);
                                inner.send_message(Message::RedrawApp);
                                log::info!("Successfully switched to layout '{}' with YAML persistence", layout_id);
                            }
                            Err(e) => {
                                log::error!("Failed to switch layout with YAML persistence: {}", e);
                                // Still update app context without YAML persistence
                                app_context_cloned.app.set_active_layout(layout_id);
                                inner.update_app_context(app_context_cloned);
                                inner.send_message(Message::RedrawApp);
                            }
                        }
                    }
                    // F0203: Multi-Stream Input Tabs message handling
                    Message::SwitchTab(muxbox_id, tab_index) => {
                        log::debug!("Processing SwitchTab message: muxbox={}, tab_index={}", muxbox_id, tab_index);
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id) {
                            log::debug!("Found muxbox '{}', attempting to switch to tab {}", muxbox_id, tab_index);
                            if muxbox.switch_to_tab(*tab_index) {
                                log::info!("Successfully switched muxbox '{}' to tab {}", muxbox_id, tab_index);
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                            } else {
                                log::warn!("Failed to switch muxbox '{}' to tab {} - switch_to_tab returned false", muxbox_id, tab_index);
                            }
                        } else {
                            log::error!("SwitchTab message for non-existent muxbox: {}", muxbox_id);
                        }
                    }
                    Message::SwitchToStream(muxbox_id, stream_id) => {
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id) {
                            if muxbox.switch_to_stream(stream_id) {
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                            }
                        }
                    }
                    Message::AddStream(muxbox_id, stream) => {
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id) {
                            muxbox.tab_system.add_stream(stream.clone());
                            inner.update_app_context(app_context_unwrapped.clone());
                            inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                        }
                    }
                    Message::RemoveStream(muxbox_id, stream_id) => {
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id) {
                            // F0213: Stream Lifecycle Management - cleanup stream sources before removal
                            if let Some(source) = muxbox.remove_stream(stream_id) {
                                log::info!("Cleaning up stream source for stream {}: {:?}", stream_id, source);
                                
                                // Perform source cleanup based on source type
                                match source {
                                    crate::model::common::StreamSource::ChoiceExecution(choice_source) => {
                                        if let Err(e) = choice_source.cleanup() {
                                            log::warn!("Failed to cleanup choice execution source: {}", e);
                                        }
                                    },
                                    crate::model::common::StreamSource::PTY(pty_source) => {
                                        if let Err(e) = pty_source.cleanup() {
                                            log::warn!("Failed to cleanup PTY source: {}", e);
                                        }
                                    },
                                    crate::model::common::StreamSource::Redirect(redirect_source) => {
                                        if let Err(e) = redirect_source.cleanup() {
                                            log::warn!("Failed to cleanup redirect source: {}", e);
                                        }
                                    },
                                    crate::model::common::StreamSource::Socket(socket_source) => {
                                        if let Err(e) = socket_source.cleanup() {
                                            log::warn!("Failed to cleanup socket source: {}", e);
                                        }
                                    },
                                    crate::model::common::StreamSource::StaticContent(_) => {
                                        // Static content sources don't need cleanup
                                        log::debug!("Static content source removed - no cleanup needed");
                                    }
                                }
                                
                                // Remove from tab system after source cleanup
                                if muxbox.tab_system.remove_stream(stream_id) {
                                    inner.update_app_context(app_context_unwrapped.clone());
                                    inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                                }
                            } else {
                                log::warn!("Stream {} not found in muxbox {} for cleanup", stream_id, muxbox_id);
                            }
                        }
                    }
                    Message::UpdateStreamContent(muxbox_id, stream_id, content) => {
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id) {
                            muxbox.update_stream_content_with_tab(stream_id, content.clone());
                            inner.update_app_context(app_context_unwrapped.clone());
                            inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
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

pub fn update_muxbox_content(
    inner: &mut RunnableImpl,
    app_context_unwrapped: &mut AppContext,
    muxbox_id: &str,
    success: bool,
    append_output: bool,
    output: &str,
) {
    log::info!(
        "=== UPDATE MUXBOX CONTENT: {} (success: {}, append: {}, output_len: {}) ===",
        muxbox_id,
        success,
        append_output,
        output.len()
    );

    let mut app_context_unwrapped_cloned = app_context_unwrapped.clone();
    let muxbox = app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id);

    if let Some(found_muxbox) = muxbox {
        log::info!(
            "Found target muxbox: {} (redirect_output: {:?})",
            muxbox_id,
            found_muxbox.redirect_output
        );

        if found_muxbox.redirect_output.is_some()
            && found_muxbox.redirect_output.as_ref().unwrap() != muxbox_id
        {
            log::info!(
                "MuxBox {} has its own redirect to: {}, following redirect chain",
                muxbox_id,
                found_muxbox.redirect_output.as_ref().unwrap()
            );
            update_muxbox_content(
                inner,
                &mut app_context_unwrapped_cloned,
                found_muxbox.redirect_output.as_ref().unwrap(),
                success,
                append_output,
                output,
            );
        } else {
            log::info!(
                "Updating muxbox {} content directly (no redirection)",
                muxbox_id
            );
            log::info!(
                "MuxBox {} current content length: {} chars",
                muxbox_id,
                found_muxbox.get_active_stream_content().join("\n").len()
            );

            // Check if this is PTY streaming output by the newline indicator
            let is_pty_streaming = output.ends_with('\n');

            if is_pty_streaming {
                // Use streaming update for PTY output (no timestamp formatting)
                log::info!("Using streaming update for muxbox {}", muxbox_id);
                found_muxbox.update_streaming_content(output, success);
            } else {
                // Use regular update for non-PTY output
                log::info!(
                    "Using regular update for muxbox {} (append: {})",
                    muxbox_id,
                    append_output
                );
                found_muxbox.update_content(output, append_output, success);
            }

            log::info!(
                "MuxBox {} updated content length: {} chars",
                muxbox_id,
                found_muxbox.get_active_stream_content().join("\n").len()
            );
            inner.update_app_context(app_context_unwrapped.clone());
            inner.send_message(Message::RedrawMuxBox(muxbox_id.to_string()));
            log::info!("Sent RedrawMuxBox message for muxbox: {}", muxbox_id);
        }
    } else {
        log::error!("Could not find muxbox {} for content update.", muxbox_id);
        // List available muxboxes for debugging
        let available_muxboxes: Vec<String> = app_context_unwrapped
            .app
            .get_active_layout()
            .unwrap()
            .get_all_muxboxes()
            .iter()
            .map(|p| p.id.clone())
            .collect();
        log::error!("Available muxboxes: {:?}", available_muxboxes);
    }
}

/// Update muxbox content routing to specific stream in tab system
pub fn update_muxbox_content_with_stream(
    inner: &mut RunnableImpl,
    app_context_unwrapped: &mut AppContext,
    muxbox_id: &str,
    stream_id: &str,
    success: bool,
    append_output: bool,
    output: &str,
) {
    log::info!(
        "=== UPDATE MUXBOX CONTENT WITH STREAM: {} stream: {} (success: {}, append: {}, output_len: {}) ===",
        muxbox_id,
        stream_id,
        success,
        append_output,
        output.len()
    );

    let muxbox = app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id);

    if let Some(found_muxbox) = muxbox {
        log::info!(
            "Found target muxbox: {} with tab system streams: {}",
            muxbox_id,
            found_muxbox.tab_system.streams.len()
        );

        // Format output with timestamp if success (non-PTY format)
        let formatted_output = if success {
            if output.ends_with('\n') {
                // PTY output - don't add timestamp
                output.to_string()
            } else {
                // Regular output - add timestamp
                let timestamp = chrono::Local::now().format("%H:%M:%S").to_string();
                if append_output && found_muxbox.tab_system.get_stream_content(stream_id).is_some() {
                    format!("{}", output)
                } else {
                    format!("[{}] {}", timestamp, output)
                }
            }
        } else {
            format!("ERROR: {}", output)
        };

        // Update the specific stream content
        found_muxbox.tab_system.update_stream_content(stream_id, formatted_output.clone());
        
        // Switch to the updated stream only if this is the first content for this stream
        // This shows new stream results but doesn't disrupt users viewing other tabs
        let should_switch = found_muxbox.tab_system.get_stream_content(stream_id)
            .map(|content| content == &formatted_output) // True if this is the first content
            .unwrap_or(false);
            
        if should_switch {
            found_muxbox.tab_system.switch_to_stream(stream_id);
            log::info!("Switched to new stream {} with first content", stream_id);
        } else {
            log::info!("Updated existing stream {} content without switching", stream_id);
        }

        log::info!(
            "Updated stream {} content in muxbox {}, switched to stream",
            stream_id,
            muxbox_id
        );
        
        inner.update_app_context(app_context_unwrapped.clone());
        inner.send_message(Message::RedrawMuxBox(muxbox_id.to_string()));
        log::info!("Sent RedrawMuxBox message for muxbox: {}", muxbox_id);
    } else {
        log::error!("Could not find muxbox {} for stream content update.", muxbox_id);
    }
}

/// Extract muxbox content for clipboard copy
pub fn get_muxbox_content_for_clipboard(muxbox: &MuxBox) -> String {
    // F0215: Stream-Based Choice Navigation - Use stream content for clipboard
    // Priority order: output > stream content > default message
    if !muxbox.output.is_empty() {
        muxbox.output.clone()
    } else {
        let stream_content = muxbox.get_active_stream_content();
        if !stream_content.is_empty() {
            stream_content.join("\n")
        } else {
            format!("MuxBox '{}': No content", muxbox.id)
        }
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

/// Calculate which choice was clicked based on muxbox bounds and click coordinates
/// T0258: Enhanced to check both X and Y coordinates against actual choice text bounds
/// Only clicks on actual choice text (not empty space after text) trigger choice activation
#[cfg(test)]
pub fn calculate_clicked_choice_index(
    muxbox: &MuxBox,
    click_x: u16,
    click_y: u16,
    choices: &[crate::model::muxbox::Choice],
) -> Option<usize> {
    calculate_clicked_choice_index_impl(muxbox, click_x, click_y, choices)
}

#[cfg(not(test))]
fn calculate_clicked_choice_index(
    muxbox: &MuxBox,
    click_x: u16,
    click_y: u16,
    choices: &[crate::model::muxbox::Choice],
) -> Option<usize> {
    calculate_clicked_choice_index_impl(muxbox, click_x, click_y, choices)
}

fn calculate_clicked_choice_index_impl(
    muxbox: &MuxBox,
    click_x: u16,
    click_y: u16,
    choices: &[crate::model::muxbox::Choice],
) -> Option<usize> {
    let bounds = muxbox.bounds();
    let muxbox_top = bounds.y1 as u16;

    if click_y < muxbox_top || choices.is_empty() {
        return None;
    }

    // Choices start at bounds.top() + 1 (one line below border) as per draw_utils.rs:610
    let choices_start_y = muxbox_top + 1;

    if click_y < choices_start_y {
        return None; // Click was on border or title area
    }

    // Check if this muxbox uses text wrapping by checking overflow_behavior directly
    // Note: We assume "wrap" behavior since this function will be called from contexts
    // where the overflow behavior is already determined to be "wrap"
    if let Some(overflow_behavior) = &muxbox.overflow_behavior {
        if overflow_behavior == "wrap" {
            return calculate_wrapped_choice_click(muxbox, click_x, click_y, choices);
        }
    }

    // Original logic for non-wrapped choices
    let choice_index = (click_y - choices_start_y) as usize;

    // Ensure click is within choice bounds (don't exceed available choices or muxbox height)
    let muxbox_bottom = bounds.y2 as u16;
    if choice_index >= choices.len() || click_y >= muxbox_bottom {
        return None;
    }

    // T0258: Check if click is within the actual text bounds of the choice
    if let Some(choice) = choices.get(choice_index) {
        if let Some(content) = &choice.content {
            // Choices are rendered at bounds.left() + 2 (per draw_utils.rs:636)
            let choice_text_start_x = bounds.left() + 2;

            // Format the content as it appears (including "..." for waiting choices)
            let formatted_content = if choice.waiting {
                format!("{}...", content)
            } else {
                content.clone()
            };

            let choice_text_end_x = choice_text_start_x + formatted_content.len();

            // Check if click X is within the actual text bounds
            if (click_x as usize) >= choice_text_start_x && (click_x as usize) < choice_text_end_x {
                Some(choice_index)
            } else {
                None // Click was after the text on the same line - should only select muxbox
            }
        } else {
            None // Choice has no content to click on
        }
    } else {
        None
    }
}

/// Handle click detection for wrapped choices
fn calculate_wrapped_choice_click(
    muxbox: &MuxBox,
    click_x: u16,
    click_y: u16,
    choices: &[crate::model::muxbox::Choice],
) -> Option<usize> {
    let bounds = muxbox.bounds();
    let viewable_width = bounds.width().saturating_sub(4);
    let choices_start_y = bounds.y1 as u16 + 1;
    let choice_text_start_x = bounds.left() + 2;

    // Create wrapped choice lines (same logic as in draw_utils.rs)
    let mut wrapped_choices = Vec::new();
    
    for (choice_idx, choice) in choices.iter().enumerate() {
        if let Some(content) = &choice.content {
            let formatted_content = if choice.waiting {
                format!("{}...", content)
            } else {
                content.clone()
            };
            
            let wrapped_lines = wrap_text_to_width_simple(&formatted_content, viewable_width);
            
            for wrapped_line in wrapped_lines {
                wrapped_choices.push((choice_idx, wrapped_line));
            }
        }
    }

    if wrapped_choices.is_empty() {
        return None;
    }

    // Calculate which wrapped line was clicked
    let clicked_line_index = (click_y - choices_start_y) as usize;
    
    if clicked_line_index >= wrapped_choices.len() {
        return None;
    }

    let (original_choice_index, line_content) = &wrapped_choices[clicked_line_index];
    let choice_text_end_x = choice_text_start_x + line_content.len();

    // Check if click X is within the actual text bounds of this wrapped line
    if (click_x as usize) >= choice_text_start_x && (click_x as usize) < choice_text_end_x {
        Some(*original_choice_index)
    } else {
        None // Click was after the text on the same line - should only select muxbox
    }
}

/// Simple text wrapping for click detection (matches draw_utils.rs logic)
fn wrap_text_to_width_simple(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut wrapped_lines = Vec::new();
    
    for line in text.lines() {
        if line.len() <= width {
            wrapped_lines.push(line.to_string());
            continue;
        }

        // Split long line into multiple wrapped lines
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in line.split_whitespace() {
            let word_len = word.len();
            
            // If word itself is longer than width, break it
            if word_len > width {
                // Finish current line if it has content
                if !current_line.is_empty() {
                    wrapped_lines.push(current_line.clone());
                    current_line.clear();
                    current_width = 0;
                }
                
                // Break the long word across multiple lines
                let mut remaining_word = word;
                while remaining_word.len() > width {
                    let (chunk, rest) = remaining_word.split_at(width);
                    wrapped_lines.push(chunk.to_string());
                    remaining_word = rest;
                }
                
                if !remaining_word.is_empty() {
                    current_line = remaining_word.to_string();
                    current_width = remaining_word.len();
                }
                continue;
            }

            // Check if adding this word would exceed width
            let space_needed = if current_line.is_empty() { 0 } else { 1 }; // Space before word
            if current_width + space_needed + word_len > width {
                // Start new line with this word
                if !current_line.is_empty() {
                    wrapped_lines.push(current_line.clone());
                }
                current_line = word.to_string();
                current_width = word_len;
            } else {
                // Add word to current line
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += 1;
                }
                current_line.push_str(word);
                current_width += word_len;
            }
        }

        // Add final line if it has content
        if !current_line.is_empty() {
            wrapped_lines.push(current_line);
        }
    }

    if wrapped_lines.is_empty() {
        wrapped_lines.push(String::new());
    }

    wrapped_lines
}

/// Auto-scroll to ensure selected choice is visible
fn auto_scroll_to_selected_choice(muxbox: &mut crate::model::muxbox::MuxBox, selected_choice_index: usize) {
    use crate::draw_utils::wrap_text_to_width;
    
    let bounds = muxbox.bounds();
    let viewable_height = bounds.height().saturating_sub(2); // Account for borders
    
    // Handle different overflow behaviors
    if let Some(overflow_behavior) = &muxbox.overflow_behavior {
        match overflow_behavior.as_str() {
            "wrap" => {
                // Calculate wrapped lines for auto-scroll in wrapped choice mode
                if let Some(choices) = muxbox.get_active_stream_choices() {
                    let viewable_width = bounds.width().saturating_sub(4);
                    let mut total_lines = 0;
                    let mut selected_line_start = 0;
                    let mut selected_line_end = 0;
                    
                    for (i, choice) in choices.iter().enumerate() {
                        if let Some(content) = &choice.content {
                            let formatted_content = if choice.waiting {
                                format!("{}...", content)
                            } else {
                                content.clone()
                            };
                            
                            let wrapped_lines = wrap_text_to_width(&formatted_content, viewable_width);
                            let line_count = wrapped_lines.len();
                            
                            if i == selected_choice_index {
                                selected_line_start = total_lines;
                                selected_line_end = total_lines + line_count - 1;
                            }
                            
                            total_lines += line_count;
                        }
                    }
                    
                    // Adjust scroll to keep selected wrapped lines visible
                    if total_lines > viewable_height {
                        let current_scroll_percent = muxbox.vertical_scroll.unwrap_or(0.0);
                        let current_scroll_offset = ((current_scroll_percent / 100.0) * (total_lines - viewable_height) as f64).floor() as usize;
                        let visible_start = current_scroll_offset;
                        let visible_end = visible_start + viewable_height - 1;
                        
                        let mut new_scroll_percent = current_scroll_percent;
                        
                        // Scroll down if selected choice is below visible area
                        if selected_line_end > visible_end {
                            let new_offset = selected_line_end.saturating_sub(viewable_height - 1);
                            new_scroll_percent = (new_offset as f64 / (total_lines - viewable_height) as f64) * 100.0;
                        }
                        // Scroll up if selected choice is above visible area
                        else if selected_line_start < visible_start {
                            let new_offset = selected_line_start;
                            new_scroll_percent = (new_offset as f64 / (total_lines - viewable_height) as f64) * 100.0;
                        }
                        
                        muxbox.vertical_scroll = Some(new_scroll_percent.clamp(0.0, 100.0));
                    }
                }
            },
            "scroll" => {
                // For scroll mode, use choice index directly
                if let Some(choices) = muxbox.get_active_stream_choices() {
                    let total_choices = choices.len();
                    if total_choices > viewable_height {
                        let current_scroll_percent = muxbox.vertical_scroll.unwrap_or(0.0);
                        let current_scroll_offset = ((current_scroll_percent / 100.0) * (total_choices - viewable_height) as f64).floor() as usize;
                        let visible_start = current_scroll_offset;
                        let visible_end = visible_start + viewable_height - 1;
                        
                        let mut new_scroll_percent = current_scroll_percent;
                        
                        // Scroll down if selected choice is below visible area
                        if selected_choice_index > visible_end {
                            let new_offset = selected_choice_index.saturating_sub(viewable_height - 1);
                            new_scroll_percent = (new_offset as f64 / (total_choices - viewable_height) as f64) * 100.0;
                        }
                        // Scroll up if selected choice is above visible area
                        else if selected_choice_index < visible_start {
                            let new_offset = selected_choice_index;
                            new_scroll_percent = (new_offset as f64 / (total_choices - viewable_height) as f64) * 100.0;
                        }
                        
                        muxbox.vertical_scroll = Some(new_scroll_percent.clamp(0.0, 100.0));
                    }
                }
            },
            _ => {
                // For other overflow behaviors (fill, cross_out, etc.), use simple choice index
                if let Some(choices) = muxbox.get_active_stream_choices() {
                    let total_choices = choices.len();
                    if total_choices > viewable_height {
                        let current_scroll_percent = muxbox.vertical_scroll.unwrap_or(0.0);
                        let current_scroll_offset = ((current_scroll_percent / 100.0) * (total_choices - viewable_height) as f64).floor() as usize;
                        let visible_start = current_scroll_offset;
                        let visible_end = visible_start + viewable_height - 1;
                        
                        let mut new_scroll_percent = current_scroll_percent;
                        
                        if selected_choice_index > visible_end {
                            let new_offset = selected_choice_index.saturating_sub(viewable_height - 1);
                            new_scroll_percent = (new_offset as f64 / (total_choices - viewable_height) as f64) * 100.0;
                        } else if selected_choice_index < visible_start {
                            let new_offset = selected_choice_index;
                            new_scroll_percent = (new_offset as f64 / (total_choices - viewable_height) as f64) * 100.0;
                        }
                        
                        muxbox.vertical_scroll = Some(new_scroll_percent.clamp(0.0, 100.0));
                    }
                }
            }
        }
    }
}

/// Trigger visual flash for muxbox (stub implementation)
fn trigger_muxbox_flash(_muxbox_id: &str) {
    // TODO: Implement visual flash with color inversion
    // This would require storing flash state and modifying muxbox rendering
    // For now, the redraw provides visual feedback
}
