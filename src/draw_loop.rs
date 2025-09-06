use crate::draw_utils::{draw_app, draw_muxbox};
use crate::model::app::{
    save_active_layout_to_yaml, save_complete_state_to_yaml, save_muxbox_bounds_to_yaml,
    save_muxbox_content_to_yaml, save_muxbox_scroll_to_yaml,
};
use crate::model::common::{InputBounds, StreamSourceTrait, StreamType};
use crate::model::muxbox::Choice;
use crate::thread_manager::Runnable;
use crate::{
    apply_buffer, apply_buffer_if_changed, handle_keypress, AppContext, MuxBox, ScreenBuffer,
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

// Hover state tracking for sensitive zones
#[derive(Debug, Clone)]
struct HoverState {
    current_zone: Option<String>,       // Currently hovered zone ID
    current_muxbox: Option<String>,     // MuxBox containing hovered zone
    last_position: Option<(u16, u16)>,  // Last mouse position
    hover_start_time: Option<std::time::SystemTime>, // When current hover started
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResizeEdge {
    BottomRight, // Only corner resize allowed
}

static DRAG_STATE: Mutex<Option<DragState>> = Mutex::new(None);
static MUXBOX_RESIZE_STATE: Mutex<Option<MuxBoxResizeState>> = Mutex::new(None);
static MUXBOX_MOVE_STATE: Mutex<Option<MuxBoxMoveState>> = Mutex::new(None);
static HOVER_STATE: Mutex<HoverState> = Mutex::new(HoverState {
    current_zone: None,
    current_muxbox: None,
    last_position: None,
    hover_start_time: None,
});

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
                    let new_x2_percent = (current_x2_percent + percent_delta_x).clamp(10.0, 100.0);

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
                    let new_y2_percent = (current_y2_percent + percent_delta_y).clamp(10.0, 100.0);

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
        let new_x1 = (current_x1 + percent_delta_x).clamp(0.0, 90.0);
        let new_x2 = (current_x2 + percent_delta_x).clamp(10.0, 100.0);

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
        let new_y1 = (current_y1 + percent_delta_y).clamp(0.0, 90.0);
        let new_y2 = (current_y2 + percent_delta_y).clamp(10.0, 100.0);

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
    let stream_content = muxbox
        .get_selected_stream()
        .map_or(Vec::new(), |s| s.content.clone());
    let stream_choices = muxbox.get_selected_stream_choices();

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
    let stream_content = muxbox
        .get_selected_stream()
        .map_or(Vec::new(), |s| s.content.clone());
    let stream_choices = muxbox.get_selected_stream_choices();

    let max_content_width = if !stream_content.is_empty() {
        stream_content
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0)
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
            }

            for message in &messages {
                log::trace!("Processing message: {:?}", message);
                match message {
                    Message::MuxBoxEventRefresh(_) => {
                        log::trace!("MuxBoxEventRefresh");
                    }
                    // ExecuteScript messages sent back from ThreadManager for stream creation + execution
                    Message::ExecuteScriptMessage(execute_script) => {
                        log::info!("Processing ExecuteScript from ThreadManager for target_box_id: {}, execution_mode: {:?}", 
                                   execute_script.target_box_id, execute_script.execution_mode);

                        // ExecuteScript already contains the stream_id from source registry
                        let stream_id = execute_script.stream_id.clone();
                        let source_id = execute_script.source.source_id.clone();

                        if let Some(target_muxbox) = app_context_unwrapped
                            .app
                            .get_muxbox_by_id_mut(&execute_script.target_box_id)
                        {
                            // Create stream label based on source type
                            let stream_label = match &execute_script.source.source_type {
                                crate::model::common::SourceType::Choice(choice_id) => {
                                    choice_id.clone()
                                }
                                crate::model::common::SourceType::StaticScript => {
                                    "Script".to_string()
                                }
                                crate::model::common::SourceType::PeriodicRefresh => {
                                    "Content".to_string()
                                } // Periodic refresh shows as "Content" tab
                                crate::model::common::SourceType::SocketUpdate => {
                                    "Socket".to_string()
                                }
                                crate::model::common::SourceType::RedirectedScript => {
                                    "Redirect".to_string()
                                }
                                crate::model::common::SourceType::HotkeyScript => {
                                    "Hotkey".to_string()
                                }
                                crate::model::common::SourceType::ScheduledScript => {
                                    "Scheduled".to_string()
                                }
                            };

                            // Create execution stream with appropriate type
                            let stream_type = match execute_script.execution_mode {
                                crate::model::common::ExecutionMode::Immediate => {
                                    StreamType::ChoiceExecution(source_id.clone())
                                }
                                crate::model::common::ExecutionMode::Thread => {
                                    StreamType::ChoiceExecution(source_id.clone())
                                }
                                crate::model::common::ExecutionMode::Pty => {
                                    StreamType::PtySession(format!("PTY-{}", source_id))
                                }
                            };

                            let mut new_stream = crate::model::common::Stream::new(
                                stream_id.clone(),
                                stream_type,
                                stream_label,
                                Vec::new(),
                                None,
                                None,
                            );

                            // Set new execution stream as selected so it renders
                            target_muxbox.selected_stream_id = Some(stream_id.clone());

                            // Add stream to target muxbox streams HashMap
                            target_muxbox.streams.insert(stream_id.clone(), new_stream);

                            log::info!(
                                "Created stream {} in box {} for execution",
                                stream_id,
                                execute_script.target_box_id
                            );

                            // T0700: Route ALL execution through ThreadManager for unified architecture
                            // Send ExecuteScript to ThreadManager for consistent handling across all execution modes
                            let mut execute_script_with_stream = execute_script.clone();
                            execute_script_with_stream.stream_id = stream_id;

                            log::info!("T0700: Unified execution - sending {:?} ExecuteScript to ThreadManager", 
                                      execute_script.execution_mode);
                            inner.send_message(Message::ExecuteScriptMessage(
                                execute_script_with_stream,
                            ));
                        } else {
                            log::error!(
                                "Target box {} not found for ExecuteScript",
                                execute_script.target_box_id
                            );
                        }
                    }
                    Message::StreamUpdateMessage(stream_update) => {
                        log::info!("Processing StreamUpdate for stream_id: {}, target_box: {}, execution_mode: {:?}", 
                                   stream_update.stream_id, stream_update.target_box_id, stream_update.execution_mode);

                        // T0308 ENHANCED: StreamUpdate handler with auto-creation - find or create stream
                        let mut stream_found = false;

                        // First, try to find existing stream across all muxboxes
                        for layout in &mut app_context_unwrapped.app.layouts {
                            if let Some(children) = &mut layout.children {
                                for muxbox in children {
                                    if let Some(stream) =
                                        muxbox.streams.get_mut(&stream_update.stream_id)
                                    {
                                        // Handle replace vs append based on content prefix
                                        if !stream_update.content_update.is_empty() {
                                            if stream_update.content_update.starts_with("REPLACE:")
                                            {
                                                // Replace content for full-screen programs
                                                let new_content = stream_update
                                                    .content_update
                                                    .strip_prefix("REPLACE:")
                                                    .unwrap_or(&stream_update.content_update);
                                                stream.content = vec![new_content.to_string()];
                                                log::info!("Replaced content in existing stream {}: {} characters", 
                                                          stream_update.stream_id, new_content.len());
                                            } else {
                                                // Normal append behavior
                                                stream
                                                    .content
                                                    .push(stream_update.content_update.clone());
                                                log::info!("Appended content to existing stream {}: {} characters", 
                                                          stream_update.stream_id, stream_update.content_update.len());
                                            }
                                        }

                                        stream_found = true;
                                        
                                        // AUTO_SCROLL_BOTTOM FIX: Apply auto-scroll when stream content is updated
                                        if muxbox.auto_scroll_bottom == Some(true) {
                                            muxbox.vertical_scroll = Some(100.0);
                                            log::debug!("Applied auto-scroll to bottom for muxbox {} after stream update", muxbox.id);
                                        }
                                        
                                        inner
                                            .send_message(Message::RedrawMuxBox(muxbox.id.clone()));
                                        break;
                                    }
                                }
                                if stream_found {
                                    break;
                                }
                            }
                        }

                        // If stream not found, create it in the target box
                        if !stream_found {
                            if let Some(target_muxbox) = app_context_unwrapped
                                .app
                                .get_muxbox_by_id_mut(&stream_update.target_box_id)
                            {
                                log::info!(
                                    "AUTO-CREATING stream {} in target box {}",
                                    stream_update.stream_id,
                                    stream_update.target_box_id
                                );

                                // Create execution stream with content
                                let stream_label = match stream_update.execution_mode {
                                    crate::model::common::ExecutionMode::Immediate => "Immediate",
                                    crate::model::common::ExecutionMode::Thread => "Thread",
                                    crate::model::common::ExecutionMode::Pty => "PTY",
                                };

                                let stream_id = target_muxbox.add_stream_with_source(
                                    crate::model::common::StreamType::ChoiceExecution(stream_update.stream_id.clone()),
                                    stream_label.to_string(),
                                    crate::model::common::StreamSource::create_immediate_execution_source(
                                        stream_update.stream_id.clone(),
                                        stream_update.target_box_id.clone(),
                                        vec!["executed".to_string()],
                                    )
                                );

                                // Add the content to the newly created stream
                                if let Some(stream) = target_muxbox.streams.get_mut(&stream_id) {
                                    if !stream_update.content_update.is_empty() {
                                        if stream_update.content_update.starts_with("REPLACE:") {
                                            // Replace content for full-screen programs
                                            let new_content = stream_update
                                                .content_update
                                                .strip_prefix("REPLACE:")
                                                .unwrap_or(&stream_update.content_update);
                                            stream.content = vec![new_content.to_string()];
                                            log::info!("Set initial content in new stream {}: {} characters", 
                                                       stream_id, new_content.len());
                                        } else {
                                            // Normal append behavior
                                            stream
                                                .content
                                                .push(stream_update.content_update.clone());
                                            log::info!(
                                                "Added content to new stream {}: {} characters",
                                                stream_id,
                                                stream_update.content_update.len()
                                            );
                                        }
                                    }
                                }

                                // Set the newly created stream as the selected stream so it's visible
                                target_muxbox.selected_stream_id = Some(stream_id.clone());
                                log::info!(
                                    "Set stream {} as selected stream for box {}",
                                    stream_id,
                                    target_muxbox.id
                                );

                                // AUTO_SCROLL_BOTTOM FIX: Apply auto-scroll when new stream content is added
                                if target_muxbox.auto_scroll_bottom == Some(true) {
                                    target_muxbox.vertical_scroll = Some(100.0);
                                    log::debug!("Applied auto-scroll to bottom for muxbox {} after new stream content", target_muxbox.id);
                                }

                                inner.send_message(Message::RedrawMuxBox(target_muxbox.id.clone()));
                            } else {
                                log::error!(
                                    "Target box {} not found for stream creation",
                                    stream_update.target_box_id
                                );
                            }
                        }

                        // Clear waiting state for any choices that were executed (visual feedback completion)
                        for layout in &mut app_context_unwrapped.app.layouts {
                            if let Some(children) = &mut layout.children {
                                for muxbox in children {
                                    if let Some(choices) = muxbox.get_selected_stream_choices_mut()
                                    {
                                        for choice in choices.iter_mut() {
                                            if choice.waiting {
                                                choice.waiting = false;
                                                log::info!(
                                                    "Cleared waiting state for choice: {}",
                                                    choice.id
                                                );
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        // CRITICAL FIX: Update app context to persist all stream changes
                        inner.update_app_context(app_context_unwrapped.clone());
                    }
                    Message::SourceActionMessage(source_action) => {
                        log::info!(
                            "Processing SourceAction: {:?} for source_id: {}, execution_mode: {:?}",
                            source_action.action,
                            source_action.source_id,
                            source_action.execution_mode
                        );

                        // T0320: SourceAction handler - Phase 4 source lifecycle management implementation
                        match source_action.action {
                            crate::model::common::ActionType::Kill => {
                                log::info!(
                                    "Kill action for source {} (mode: {:?})",
                                    source_action.source_id,
                                    source_action.execution_mode
                                );

                                // Find and terminate the source based on execution mode
                                let mut source_terminated = false;
                                let mut stream_to_update: Option<(String, String)> = None; // (stream_id, muxbox_id)

                                // Search all muxboxes for streams with this source_id (read-only first)
                                for layout in &app_context_unwrapped.app.layouts {
                                    for muxbox in layout.get_all_muxboxes() {
                                        for (stream_id, stream) in &muxbox.streams {
                                            // Check if this stream matches the source_id
                                            let stream_source_id = match &stream.stream_type {
                                                StreamType::ChoiceExecution(id) => Some(id.clone()),
                                                StreamType::PtySession(id) => {
                                                    // Extract source_id from PTY session format "PTY-{source_id}"
                                                    if let Some(stripped) = id.strip_prefix("PTY-")
                                                    {
                                                        Some(stripped.to_string())
                                                    } else {
                                                        Some(id.clone())
                                                    }
                                                }
                                                _ => None,
                                            };

                                            if let Some(stream_src_id) = stream_source_id {
                                                if stream_src_id == source_action.source_id {
                                                    log::info!("Found stream {} with source_id {} for termination", stream_id, source_action.source_id);
                                                    stream_to_update = Some((
                                                        stream_id.clone(),
                                                        muxbox.id.clone(),
                                                    ));
                                                    source_terminated = true;
                                                    break;
                                                }
                                            }
                                        }
                                        if source_terminated {
                                            break;
                                        }
                                    }
                                    if source_terminated {
                                        break;
                                    }
                                }

                                // Now perform the actual cleanup if we found the stream
                                if let Some((stream_id, muxbox_id)) = stream_to_update {
                                    // Get mutable access to perform cleanup
                                    if let Some(target_muxbox) =
                                        app_context_unwrapped.app.get_muxbox_by_id_mut(&muxbox_id)
                                    {
                                        if let Some(stream) = target_muxbox.streams.get(&stream_id)
                                        {
                                            // Attempt source cleanup based on execution mode
                                            if let Some(ref stream_source) = stream.source {
                                                match source_action.execution_mode {
                                                    crate::model::common::ExecutionMode::Immediate => {
                                                        // Batch mode - attempt to cancel queued task
                                                        if let Err(e) = stream_source.cleanup() {
                                                            log::warn!("Failed to cleanup batch source {}: {}", source_action.source_id, e);
                                                        } else {
                                                            log::info!("Successfully terminated batch source {}", source_action.source_id);
                                                        }
                                                    }
                                                    crate::model::common::ExecutionMode::Thread => {
                                                        // Thread mode - interrupt thread execution
                                                        if let Err(e) = stream_source.cleanup() {
                                                            log::warn!("Failed to cleanup thread source {}: {}", source_action.source_id, e);
                                                        } else {
                                                            log::info!("Successfully terminated thread source {}", source_action.source_id);
                                                        }
                                                    }
                                                    crate::model::common::ExecutionMode::Pty => {
                                                        // PTY mode - kill PTY process
                                                        if let Err(e) = stream_source.cleanup() {
                                                            log::warn!("Failed to cleanup PTY source {}: {}", source_action.source_id, e);
                                                        } else {
                                                            log::info!("Successfully terminated PTY source {}", source_action.source_id);
                                                        }
                                                    }
                                                }
                                            } else {
                                                log::info!("Stream {} source already terminated or inactive", stream_id);
                                            }

                                            // Send StreamUpdate with terminated status
                                            let terminated_state = match source_action.execution_mode {
                                                crate::model::common::ExecutionMode::Immediate => {
                                                    crate::model::common::SourceState::Batch(
                                                        crate::model::common::BatchSourceState {
                                                            task_id: source_action.source_id.clone(),
                                                            queue_wait_time: std::time::Duration::from_secs(0),
                                                            execution_time: std::time::Duration::from_secs(0),
                                                            exit_code: Some(1),
                                                            status: crate::model::common::BatchStatus::Failed("Killed by user".to_string())
                                                        }
                                                    )
                                                }
                                                crate::model::common::ExecutionMode::Thread => {
                                                    crate::model::common::SourceState::Thread(
                                                        crate::model::common::ThreadSourceState {
                                                            thread_id: source_action.source_id.clone(),
                                                            execution_time: std::time::Duration::from_secs(0),
                                                            exit_code: Some(1),
                                                            status: crate::model::common::ExecutionThreadStatus::Failed("Killed by user".to_string())
                                                        }
                                                    )
                                                }
                                                crate::model::common::ExecutionMode::Pty => {
                                                    crate::model::common::SourceState::Pty(
                                                        crate::model::common::PtySourceState {
                                                            process_id: 0, // Will be updated by actual PTY termination
                                                            runtime: std::time::Duration::from_secs(0),
                                                            exit_code: Some(1),
                                                            status: crate::model::common::ExecutionPtyStatus::Terminated
                                                        }
                                                    )
                                                }
                                            };

                                            let termination_update =
                                                crate::model::common::StreamUpdate {
                                                    stream_id: stream_id.clone(),
                                                    target_box_id: muxbox_id.clone(),
                                                    content_update:
                                                        "\n[Process terminated by user]".to_string(),
                                                    source_state: terminated_state,
                                                    execution_mode: source_action
                                                        .execution_mode
                                                        .clone(),
                                                };

                                            inner.send_message(Message::StreamUpdateMessage(
                                                termination_update,
                                            ));
                                        }
                                    }
                                } else {
                                    log::warn!(
                                        "Source {} not found for kill action",
                                        source_action.source_id
                                    );
                                }
                            }
                            crate::model::common::ActionType::Query => {
                                log::info!(
                                    "Query action for source {} (mode: {:?})",
                                    source_action.source_id,
                                    source_action.execution_mode
                                );

                                // Find source and return status information
                                let mut found_source = false;
                                for layout in &app_context_unwrapped.app.layouts {
                                    for muxbox in layout.get_all_muxboxes() {
                                        for (stream_id, stream) in &muxbox.streams {
                                            let stream_source_id = match &stream.stream_type {
                                                StreamType::ChoiceExecution(id) => Some(id.clone()),
                                                StreamType::PtySession(id) => {
                                                    if let Some(stripped) = id.strip_prefix("PTY-")
                                                    {
                                                        Some(stripped.to_string())
                                                    } else {
                                                        Some(id.clone())
                                                    }
                                                }
                                                _ => None,
                                            };

                                            if let Some(stream_src_id) = stream_source_id {
                                                if stream_src_id == source_action.source_id {
                                                    log::info!(
                                                        "Source {} status: stream_id={}, active={}",
                                                        source_action.source_id,
                                                        stream_id,
                                                        stream.source.is_some()
                                                    );
                                                    found_source = true;
                                                    break;
                                                }
                                            }
                                        }
                                        if found_source {
                                            break;
                                        }
                                    }
                                    if found_source {
                                        break;
                                    }
                                }

                                if !found_source {
                                    log::info!(
                                        "Source {} not found for query",
                                        source_action.source_id
                                    );
                                }
                            }
                            crate::model::common::ActionType::Pause => {
                                log::info!("Pause action for source {} (mode: {:?}) - not supported in current implementation", 
                                          source_action.source_id, source_action.execution_mode);
                                // Pause/Resume not implemented in Phase 4 - future enhancement
                            }
                            crate::model::common::ActionType::Resume => {
                                log::info!("Resume action for source {} (mode: {:?}) - not supported in current implementation", 
                                          source_action.source_id, source_action.execution_mode);
                                // Pause/Resume not implemented in Phase 4 - future enhancement
                            }
                        }
                    }
                    // T0326: REMOVED CreateChoiceExecutionStream handler - replaced by ExecuteScript handler
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
                                if let Some(choices) =
                                    found_muxbox.get_selected_stream_choices_mut()
                                {
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
                                    auto_scroll_to_selected_choice(
                                        found_muxbox,
                                        new_selected_choice,
                                    );
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
                                if let Some(choices) =
                                    found_muxbox.get_selected_stream_choices_mut()
                                {
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
                                    auto_scroll_to_selected_choice(
                                        found_muxbox,
                                        new_selected_choice,
                                    );
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
                                    (found_muxbox.horizontal_scroll.unwrap_or(0.0) * 100.0)
                                        as usize,
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
                                    (found_muxbox.horizontal_scroll.unwrap_or(0.0) * 100.0)
                                        as usize,
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
                        if let Some(found_muxbox) = app_context_unwrapped
                            .app
                            .get_muxbox_by_id_mut(muxbox_id)
                            .cloned()
                        {
                            new_buffer = buffer.clone();

                            // Clone the parent layout to avoid mutable borrow conflicts
                            if let Some(parent_layout) =
                                found_muxbox.get_parent_layout_clone(&app_context_unwrapped)
                            {
                                draw_muxbox(
                                    &app_context_unwrapped,
                                    &app_graph,
                                    &adjusted_bounds,
                                    &parent_layout,
                                    &found_muxbox,
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
                    // T0328: REMOVED MuxBoxOutputUpdate handler - replaced by StreamUpdateMessage handler
                    // ExternalMessage handling is now done by RSJanusComms library
                    // Messages are converted to appropriate internal messages by the socket handler
                    Message::ExternalMessage(_) => {
                        // This should no longer be used - socket handler converts messages directly
                        log::warn!("Received deprecated ExternalMessage - should be converted by socket handler");
                    }
                    Message::ExecuteHotKeyChoice(choice_id) => {
                        log::info!("=== EXECUTING HOT KEY CHOICE: {} ===", choice_id);

                        // F0229: Unified ExecutionMode System - all execution paths use same unified stream approach
                        // First extract the data we need without borrowing app_context_unwrapped
                        let (choice_data, muxbox_id, libs) = {
                            let active_layout =
                                app_context_unwrapped.app.get_active_layout().unwrap();
                            let libs = app_context_unwrapped.app.libs.clone();

                            // Find the choice by ID in any muxbox
                            log::info!("Searching for choice {} in active layout", choice_id);
                            if let Some(choice_muxbox) =
                                active_layout.find_muxbox_with_choice(choice_id)
                            {
                                log::info!("Found choice in muxbox: {}", choice_muxbox.id);

                                if let Some(choices) = choice_muxbox.get_selected_stream_choices() {
                                    if let Some(choice) =
                                        choices.iter().find(|c| c.id == *choice_id)
                                    {
                                        log::info!("Hotkey choice config - execution_mode: {:?}, redirect: {:?}, script_lines: {}", 
                                            choice.execution_mode,
                                            choice.redirect_output,
                                            choice.script.as_ref().map(|s| s.len()).unwrap_or(0)
                                        );

                                        if let Some(script) = &choice.script {
                                            (
                                                Some((choice.clone(), script.clone())),
                                                choice_muxbox.id.clone(),
                                                libs,
                                            )
                                        } else {
                                            (None, choice_muxbox.id.clone(), libs)
                                        }
                                    } else {
                                        log::warn!("Choice {} found in muxbox {} but no matching choice in choices list", choice_id, choice_muxbox.id);
                                        (None, choice_muxbox.id.clone(), libs)
                                    }
                                } else {
                                    log::warn!("MuxBox {} has no choices list", choice_muxbox.id);
                                    (None, choice_muxbox.id.clone(), libs)
                                }
                            } else {
                                log::error!(
                                    "Choice {} not found in any muxbox of active layout",
                                    choice_id
                                );
                                (None, String::new(), libs)
                            }
                        };

                        // T0316: UNIFIED ARCHITECTURE - Replace legacy hotkey execution with ExecuteScript message
                        if let Some((choice, script)) = choice_data {
                            log::info!(
                                "T0316: Hotkey creating ExecuteScript for choice {} (mode: {:?})",
                                choice_id,
                                choice.execution_mode
                            );

                            // Create ExecuteScript message instead of direct execution
                            use crate::model::common::{
                                ExecuteScript, ExecutionSource, SourceReference, SourceType,
                            };

                            // Register execution source and get stream_id
                            let source_type =
                                crate::model::common::ExecutionSourceType::HotkeyScript {
                                    hotkey: format!("hotkey_for_{}", choice_id), // We don't have the actual key here, use placeholder
                                    script: script.clone(),
                                };
                            let stream_id = app_context_unwrapped
                                .app
                                .register_execution_source(source_type, muxbox_id.clone());

                            let execute_script = ExecuteScript {
                                script: script.clone(),
                                source: ExecutionSource {
                                    source_type: SourceType::HotkeyScript,
                                    source_id: format!("hotkey_choice_{}", choice_id),
                                    source_reference: SourceReference::Choice(choice.clone()),
                                },
                                execution_mode: choice.execution_mode.clone(),
                                target_box_id: muxbox_id.clone(),
                                libs: libs.unwrap_or_default(),
                                redirect_output: choice.redirect_output.clone(),
                                append_output: choice.append_output.unwrap_or(false),
                                stream_id,
                                target_bounds: app_context_unwrapped
                                    .app
                                    .get_active_layout()
                                    .and_then(|layout| {
                                        layout
                                            .children
                                            .as_ref()?
                                            .iter()
                                            .find(|mb| mb.id == *muxbox_id)
                                    })
                                    .map(|mb| mb.bounds()),
                            };

                            // Send ExecuteScript message instead of calling legacy execute_choice_stream_only
                            inner.send_message(Message::ExecuteScriptMessage(execute_script));

                            log::info!(
                                "T0316: ExecuteScript message sent for hotkey choice {} (unified architecture)",
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
                                .filter(|p| p.get_selected_stream_choices().is_none())
                                .collect();

                        let libs = app_context_unwrapped.app.libs.clone();

                        if pressed_key == "Enter" {
                            let selected_muxboxes_with_choices: Vec<&MuxBox> = selected_muxboxes
                                .into_iter()
                                .filter(|p| p.get_selected_stream_choices().is_some())
                                .collect();
                            for muxbox in selected_muxboxes_with_choices {
                                // First, extract choice information before any mutable operations
                                let (selected_choice_data, choice_needs_execution) = {
                                    let muxbox_ref = app_context_for_keypress
                                        .app
                                        .get_muxbox_by_id(&muxbox.id)
                                        .unwrap();
                                    if let Some(choices) = muxbox_ref.get_selected_stream_choices()
                                    {
                                        if let Some(selected_choice) =
                                            choices.iter().find(|c| c.selected)
                                        {
                                            let choice_data = (
                                                selected_choice.id.clone(),
                                                selected_choice.script.clone(),
                                                selected_choice.execution_mode.clone(), // Use ExecutionMode directly instead of boolean flags
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
                                    execution_mode,
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
                                        log::info!("Enter choice config - execution_mode: {:?}, redirect: {:?}", 
                                            execution_mode, redirect_output
                                        );

                                        if let Some(script) = script_opt {
                                            let libs_clone = libs.clone();

                                            // Set choice to waiting state before execution
                                            if let Some(muxbox_mut) = app_context_for_keypress
                                                .app
                                                .get_muxbox_by_id_mut(&muxbox_id)
                                            {
                                                if let Some(choices) =
                                                    muxbox_mut.get_selected_stream_choices_mut()
                                                {
                                                    if let Some(choice) = choices
                                                        .iter_mut()
                                                        .find(|c| c.id == choice_id)
                                                    {
                                                        choice.waiting = true;
                                                    }
                                                }
                                            }

                                            // T0314: UNIFIED ARCHITECTURE - Replace legacy execution with ExecuteScript message
                                            log::info!("T0314: Enter key creating ExecuteScript for choice {} (mode: {:?})", choice_id, execution_mode);

                                            // Create ExecuteScript message instead of direct execution
                                            use crate::model::common::{
                                                ExecuteScript, ExecutionSource, SourceReference,
                                                SourceType,
                                            };

                                            // Create choice object for SourceReference
                                            let choice_for_reference = Choice {
                                                id: choice_id.clone(),
                                                content: Some("".to_string()),
                                                selected: false,
                                                script: Some(script.clone()),
                                                execution_mode: execution_mode.clone(),
                                                redirect_output: redirect_output.clone(),
                                                append_output: Some(append_output),
                                                waiting: true,
                                                hovered: false,
                                            };

                                            // Register execution source and get stream_id
                                            let source_type = crate::model::common::ExecutionSourceType::ChoiceExecution {
                                                choice_id: choice_id.clone(),
                                                script: script.clone(),
                                                redirect_output: redirect_output.clone(),
                                            };
                                            // Restructure to avoid borrow conflicts - get stream_id before holding references
                                            let stream_id = {
                                                let mut app_for_registration =
                                                    app_context_unwrapped.clone();
                                                app_for_registration.app.register_execution_source(
                                                    source_type,
                                                    muxbox_id.clone(),
                                                )
                                            };

                                            let execute_script = ExecuteScript {
                                                script: script.clone(),
                                                source: ExecutionSource {
                                                    source_type: SourceType::Choice(
                                                        choice_id.clone(),
                                                    ),
                                                    source_id: format!("choice_{}", choice_id),
                                                    source_reference: SourceReference::Choice(
                                                        choice_for_reference,
                                                    ),
                                                },
                                                execution_mode: execution_mode.clone(),
                                                target_box_id: muxbox_id.clone(),
                                                libs: libs_clone.unwrap_or_default(),
                                                redirect_output: redirect_output.clone(),
                                                append_output,
                                                stream_id: stream_id.clone(),
                                                target_bounds: Some(muxbox.bounds()),
                                            };

                                            // UNIFIED EXECUTION ARCHITECTURE: Route ExecuteScript based on execution mode
                                            match execution_mode {
                                                crate::model::common::ExecutionMode::Immediate
                                                | crate::model::common::ExecutionMode::Thread => {
                                                    // Send to ThreadManager for Immediate/Thread execution
                                                    inner.send_message(
                                                        Message::ExecuteScriptMessage(
                                                            execute_script,
                                                        ),
                                                    );
                                                    log::info!(
                                                        "T0314: ExecuteScript message sent to ThreadManager for choice {} (mode: {:?})",
                                                        choice_id, execution_mode
                                                    );
                                                }
                                                crate::model::common::ExecutionMode::Pty => {
                                                    // FIXED: Route PTY execution to PTYManager, never to ThreadManager
                                                    log::info!(
                                                        "T0314 FIXED: Routing PTY ExecuteScript to PTYManager for choice {} (mode: {:?})",
                                                        choice_id, execution_mode
                                                    );

                                                    // Route to PTYManager instead of ThreadManager
                                                    if let Some(pty_manager) =
                                                        &app_context_unwrapped.pty_manager
                                                    {
                                                        // Get message sender for PTY communication
                                                        if let Some(sender) =
                                                            inner.get_message_sender()
                                                        {
                                                            let uuid = uuid::Uuid::new_v4();

                                                            // Call PTYManager's ExecuteScript handler
                                                            if let Err(e) = pty_manager
                                                                .handle_execute_script(
                                                                    &execute_script,
                                                                    sender.clone(),
                                                                    uuid,
                                                                )
                                                            {
                                                                log::error!(
                                                                    "T0314: PTYManager failed to handle ExecuteScript for choice {}: {}",
                                                                    choice_id, e
                                                                );
                                                            } else {
                                                                log::info!(
                                                                    "T0314 FIXED: PTYManager successfully handling ExecuteScript for choice {} (architecture compliant)",
                                                                    choice_id
                                                                );
                                                            }
                                                        } else {
                                                            log::error!("No message sender available for PTY execution - choice {}", choice_id);
                                                        }
                                                    } else {
                                                        log::error!("No PTYManager available - PTY execution failed for choice {}", choice_id);
                                                    }
                                                    log::info!(
                                                        "T0314 FIXED: PTY execution routed to PTYManager for choice {} (never sent to ThreadManager)",
                                                        choice_id
                                                    );
                                                }
                                            }

                                            // Update the app context to persist the waiting state change
                                            inner.update_app_context(
                                                app_context_for_keypress.clone(),
                                            );
                                        }
                                    }
                                }
                            }
                        }

                        // T0317: UNIFIED ARCHITECTURE - Replace muxbox keypress run_script with ExecuteScript messages
                        for muxbox in selected_muxboxes_with_keypress_events {
                            let actions =
                                handle_keypress(pressed_key, &muxbox.on_keypress.clone().unwrap());
                            if let Some(actions_unwrapped) = actions {
                                let libs = app_context_unwrapped.app.libs.clone();

                                log::info!("T0317: Creating ExecuteScript for muxbox keypress handler {} ({})", muxbox.id, pressed_key);

                                // Create ExecuteScript message for muxbox-level keypress handlers
                                use crate::model::common::{
                                    ExecuteScript, ExecutionMode, ExecutionSource, SourceReference,
                                    SourceType,
                                };

                                // Register execution source and get stream_id
                                let target_box_id = muxbox
                                    .redirect_output
                                    .as_ref()
                                    .unwrap_or(&muxbox.id)
                                    .clone();
                                let source_type =
                                    crate::model::common::ExecutionSourceType::SocketUpdate {
                                        command_type: format!("keypress_{}", pressed_key),
                                    };
                                // Restructure to avoid borrow conflicts - get stream_id with separate context
                                let stream_id = {
                                    let mut app_for_registration = app_context_unwrapped.clone();
                                    app_for_registration.app.register_execution_source(
                                        source_type,
                                        target_box_id.clone(),
                                    )
                                };

                                let execute_script = ExecuteScript {
                                    script: actions_unwrapped,
                                    source: ExecutionSource {
                                        source_type: SourceType::SocketUpdate,
                                        source_id: format!(
                                            "muxbox_keypress_{}_{}",
                                            muxbox.id, pressed_key
                                        ),
                                        source_reference: SourceReference::SocketCommand(format!(
                                            "muxbox {} keypress: {}",
                                            muxbox.id, pressed_key
                                        )),
                                    },
                                    execution_mode: ExecutionMode::Immediate, // Muxbox-level handlers use immediate execution
                                    target_box_id,
                                    libs: libs.unwrap_or_default(),
                                    redirect_output: muxbox.redirect_output.clone(),
                                    append_output: muxbox.append_output.unwrap_or(false),
                                    stream_id,
                                    target_bounds: Some(muxbox.bounds()),
                                };

                                inner.send_message(Message::ExecuteScriptMessage(execute_script));
                                log::info!("T0317: ExecuteScript message sent for muxbox {} keypress handler ({})", muxbox.id, pressed_key);
                            }
                        }
                    }
                    Message::PTYInput(muxbox_id, input) => {
                        log::trace!("PTY input for muxbox {}: {}", muxbox_id, input);

                        // Find the target muxbox to verify it exists and has PTY enabled
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id(muxbox_id)
                        {
                            // F0229: Use ExecutionMode instead of legacy pty field
                            if muxbox.execution_mode == crate::model::common::ExecutionMode::Pty {
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
                    Message::PTYMouseEvent(muxbox_id, kind, column, row, modifiers) => {
                        log::trace!(
                            "PTY mouse event for muxbox {}: {:?} at ({}, {})",
                            muxbox_id,
                            kind,
                            column,
                            row
                        );

                        // Find the target muxbox to verify it exists and has PTY enabled
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id(muxbox_id)
                        {
                            if muxbox.execution_mode == crate::model::common::ExecutionMode::Pty {
                                // Generate terminal state-aware mouse sequence using PTY manager
                                if let Some(pty_manager) = &app_context_unwrapped.pty_manager {
                                    if let Some(mouse_sequence) = pty_manager
                                        .generate_mouse_sequence(
                                            muxbox_id, *kind, *column, *row, *modifiers,
                                        )
                                    {
                                        match pty_manager.send_input(muxbox_id, &mouse_sequence) {
                                            Ok(_) => {
                                                log::debug!(
                                                    "Sent PTY mouse sequence to muxbox {}: {}",
                                                    muxbox_id,
                                                    mouse_sequence
                                                );
                                            }
                                            Err(e) => {
                                                log::error!("Failed to send PTY mouse sequence to muxbox {}: {}", muxbox_id, e);
                                            }
                                        }
                                    } else {
                                        log::trace!(
                                            "Mouse reporting disabled for muxbox {}",
                                            muxbox_id
                                        );
                                    }
                                } else {
                                    log::error!(
                                        "Could not lock PTY manager for mouse event processing"
                                    );
                                }
                            } else {
                                log::warn!("MuxBox {} received PTY mouse event but execution_mode is not Pty", muxbox_id);
                            }
                        } else {
                            log::error!(
                                "PTY mouse event received for non-existent muxbox: {}",
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
                                    && (*y as usize) < muxbox_bounds.bottom().saturating_sub(1)
                                {
                                    let track_height =
                                        (muxbox_bounds.height() as isize - 2).max(1) as usize;
                                    let click_position = ((*y as usize) - muxbox_bounds.top() - 1)
                                        as f64
                                        / track_height as f64;
                                    let scroll_percentage =
                                        (click_position * 100.0).clamp(0.0, 100.0);

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
                                        (
                                            muxbox_to_update.id.clone(),
                                            muxbox_to_update.horizontal_scroll.unwrap_or(0.0),
                                        )
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

                                // Check for horizontal scrollbar click (on bottom border)
                                if *y as usize == muxbox_bounds.bottom()
                                    && *x as usize > muxbox_bounds.left()
                                    && (*x as usize) < muxbox_bounds.right().saturating_sub(1)
                                {
                                    let track_width =
                                        (muxbox_bounds.width() as isize - 2).max(1) as usize;
                                    let click_position = ((*x as usize) - muxbox_bounds.left() - 1)
                                        as f64
                                        / track_width as f64;
                                    let scroll_percentage =
                                        (click_position * 100.0).clamp(0.0, 100.0);

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
                                        muxbox_to_update.horizontal_scroll =
                                            Some(scroll_percentage);
                                        (
                                            muxbox_to_update.id.clone(),
                                            muxbox_to_update.vertical_scroll.unwrap_or(0.0),
                                        )
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
                            // F0203: Check for tab clicks first using proper z-index ordering
                            let mut handled_tab_click = false;

                            // Find the top-most muxbox at the click coordinates (respects z-index)
                            if let Some(clicked_muxbox) =
                                active_layout.find_muxbox_at_coordinates(*x, *y)
                            {
                                let muxbox_bounds = clicked_muxbox.bounds();

                                // Check if click is in title bar area specifically
                                if *y as usize == muxbox_bounds.top() {
                                    let tab_labels = clicked_muxbox.get_tab_labels();
                                    log::debug!("Title bar click at ({},{}) in top-most muxbox '{}' with {} tabs: {:?}", 
                                        *x, *y, clicked_muxbox.id, tab_labels.len(), tab_labels);

                                    let has_border = clicked_muxbox
                                        .calc_border(&app_context_unwrapped.clone(), &app_graph);
                                    log::debug!(
                                        "Muxbox bounds: left={}, right={}, top={}, border={}",
                                        muxbox_bounds.left(),
                                        muxbox_bounds.right(),
                                        muxbox_bounds.top(),
                                        has_border
                                    );

                                    // Check for navigation arrow clicks first
                                    if let Some(nav_action) =
                                        crate::draw_utils::calculate_tab_navigation_click(
                                            *x as usize,
                                            muxbox_bounds.left(),
                                            muxbox_bounds.right(),
                                            &tab_labels,
                                            clicked_muxbox.tab_scroll_offset,
                                            &clicked_muxbox.calc_border_color(
                                                &app_context_unwrapped,
                                                &app_graph,
                                            ),
                                            &clicked_muxbox.bg_color,
                                        )
                                    {
                                        log::info!(
                                            "Tab navigation clicked: muxbox {} action {:?}",
                                            clicked_muxbox.id,
                                            nav_action
                                        );
                                        if let Some(muxbox) = app_context_for_click
                                            .app
                                            .get_muxbox_by_id_mut(&clicked_muxbox.id)
                                        {
                                            match nav_action {
                                                crate::draw_utils::TabNavigationAction::ScrollLeft => {
                                                    muxbox.scroll_tabs_left();
                                                    log::info!("Scrolled tabs left for muxbox '{}', new offset: {}", muxbox.id, muxbox.tab_scroll_offset);
                                                },
                                                crate::draw_utils::TabNavigationAction::ScrollRight => {
                                                    muxbox.scroll_tabs_right();
                                                    log::info!("Scrolled tabs right for muxbox '{}', new offset: {}", muxbox.id, muxbox.tab_scroll_offset);
                                                },
                                            }
                                            inner.update_app_context(app_context_for_click.clone());
                                        }
                                        handled_tab_click = true;
                                    } else if let Some(close_tab_index) =
                                        crate::draw_utils::calculate_tab_close_click(
                                            *x as usize,
                                            muxbox_bounds.left(),
                                            muxbox_bounds.right(),
                                            &tab_labels,
                                            &clicked_muxbox.get_tab_close_buttons(),
                                            clicked_muxbox.tab_scroll_offset,
                                            &clicked_muxbox.calc_border_color(
                                                &app_context_unwrapped,
                                                &app_graph,
                                            ),
                                            &clicked_muxbox.bg_color,
                                        )
                                    {
                                        // T0323: Tab close integration with unified execution architecture
                                        let stream_ids = clicked_muxbox.get_tab_stream_ids();
                                        if let Some(stream_id) = stream_ids.get(close_tab_index) {
                                            log::info!(
                                                "Close button clicked for tab {} (stream: {})",
                                                close_tab_index,
                                                stream_id
                                            );

                                            // Get stream info to determine source_id and execution_mode
                                            if let Some(stream) =
                                                clicked_muxbox.streams.get(stream_id)
                                            {
                                                if stream.is_closeable() {
                                                    log::info!(
                                                        "Processing close tab for closeable stream {} in muxbox {}",
                                                        stream_id,
                                                        clicked_muxbox.id
                                                    );

                                                    // Extract source_id and execution_mode from stream type
                                                    let (source_id, execution_mode) = match &stream
                                                        .stream_type
                                                    {
                                                        StreamType::ChoiceExecution(id) => {
                                                            // Choice execution streams - determine execution mode from stream context
                                                            // For now, default to Thread mode for choice executions
                                                            (id.clone(), crate::model::common::ExecutionMode::Thread)
                                                        }
                                                        StreamType::PtySession(id) => {
                                                            // PTY session streams - extract source_id from "PTY-{source_id}" format
                                                            let actual_source_id =
                                                                if let Some(stripped) =
                                                                    id.strip_prefix("PTY-")
                                                                {
                                                                    stripped.to_string()
                                                                } else {
                                                                    id.clone()
                                                                };
                                                            (actual_source_id, crate::model::common::ExecutionMode::Pty)
                                                        }
                                                        StreamType::RedirectedOutput(_) => {
                                                            // Redirected output streams - use stream_id as source_id
                                                            (stream_id.clone(), crate::model::common::ExecutionMode::Thread)
                                                        }
                                                        StreamType::ExternalSocket => {
                                                            // External socket streams - use stream_id as source_id
                                                            (stream_id.clone(), crate::model::common::ExecutionMode::Thread)
                                                        }
                                                        _ => {
                                                            // Content/Choices streams are not closeable - this shouldn't happen
                                                            log::warn!("Unexpected closeable stream type: {:?}", stream.stream_type);
                                                            (stream_id.clone(), crate::model::common::ExecutionMode::Thread)
                                                        }
                                                    };

                                                    // Create SourceAction Kill message to terminate the source
                                                    let kill_action = crate::model::common::SourceAction {
                                                        action: crate::model::common::ActionType::Kill,
                                                        source_id,
                                                        execution_mode,
                                                    };

                                                    log::info!("Sending SourceAction Kill for source {} (mode: {:?})", 
                                                              kill_action.source_id, kill_action.execution_mode);

                                                    // Send SourceAction message - this will be handled by the unified architecture
                                                    inner.send_message(
                                                        Message::SourceActionMessage(kill_action),
                                                    );

                                                    // Also directly remove the stream for immediate UI feedback
                                                    // The SourceAction will handle process termination
                                                    let mut app_context_for_close =
                                                        app_context_unwrapped.clone();
                                                    if let Some(muxbox) = app_context_for_close
                                                        .app
                                                        .get_muxbox_by_id_mut(&clicked_muxbox.id)
                                                    {
                                                        if muxbox.streams.contains_key(stream_id) {
                                                            let _removed_source =
                                                                muxbox.remove_stream(stream_id);
                                                            log::info!("Stream {} removed from UI (source termination handled by SourceAction)", stream_id);

                                                            // Update app context and trigger redraw for immediate UI response
                                                            inner.update_app_context(
                                                                app_context_for_close.clone(),
                                                            );
                                                            inner.send_message(
                                                                Message::RedrawAppDiff,
                                                            );
                                                        }
                                                    }
                                                } else {
                                                    log::info!(
                                                        "Stream {} in muxbox {} is not closeable",
                                                        stream_id,
                                                        clicked_muxbox.id
                                                    );
                                                }
                                            }
                                        }
                                        handled_tab_click = true;
                                    } else if let Some(clicked_tab_index) =
                                        crate::draw_utils::calculate_tab_click_index(
                                            *x as usize,
                                            muxbox_bounds.left(),
                                            muxbox_bounds.right(),
                                            &tab_labels,
                                            clicked_muxbox.tab_scroll_offset,
                                            &clicked_muxbox.calc_border_color(
                                                &app_context_unwrapped,
                                                &app_graph,
                                            ),
                                            &clicked_muxbox.bg_color,
                                        )
                                    {
                                        log::info!(
                                            "Tab click detected: muxbox {} tab {} ({})",
                                            clicked_muxbox.id,
                                            clicked_tab_index,
                                            tab_labels
                                                .get(clicked_tab_index)
                                                .unwrap_or(&"unknown".to_string())
                                        );

                                        log::info!("Processing SwitchTab directly: muxbox={}, tab_index={}", clicked_muxbox.id, clicked_tab_index);
                                        if let Some(muxbox) = app_context_for_click
                                            .app
                                            .get_muxbox_by_id_mut(&clicked_muxbox.id)
                                        {
                                            if muxbox.switch_to_tab(clicked_tab_index) {
                                                log::info!(
                                                    "Successfully switched muxbox '{}' to tab {}",
                                                    muxbox.id,
                                                    clicked_tab_index
                                                );
                                                inner.update_app_context(
                                                    app_context_for_click.clone(),
                                                );
                                            } else {
                                                log::warn!("Failed to switch muxbox '{}' to tab {} - switch_to_tab returned false", muxbox.id, clicked_tab_index);
                                            }
                                        }
                                        handled_tab_click = true;
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

                                    // FORMALIZED COORDINATE SYSTEM: Use BoxDimensions for all coordinate translation
                                    log::info!("CLICK: Processing click on muxbox '{}' at screen ({}, {})", 
                                             clicked_muxbox.id, *x, *y);

                                    // UNIVERSAL BOX SELECTION: Select any muxbox that is clicked, regardless of specific actions
                                    log::trace!("Selecting muxbox on click: {}", clicked_muxbox.id);
                                    let layout = app_context_for_click.app.get_active_layout_mut().unwrap();
                                    layout.deselect_all_muxboxes();
                                    layout.select_only_muxbox(&clicked_muxbox.id);
                                    inner.update_app_context(app_context_for_click.clone());
                                    inner.send_message(Message::RedrawAppDiff);

                                    // Check if muxbox has choices (menu items) in the currently selected stream
                                    log::info!("CLICK DEBUG: Checking selected stream for muxbox '{}'", clicked_muxbox.id);
                                    if let Some(selected_stream) = clicked_muxbox.get_selected_stream() {
                                        log::info!("CLICK DEBUG: Found selected stream type: {:?}, has choices: {}", 
                                                 selected_stream.stream_type, 
                                                 selected_stream.choices.as_ref().map(|c| c.len()).unwrap_or(0));
                                        if let Some(choices) = selected_stream.choices.as_ref() {
                                            if !choices.is_empty() {
                                        use crate::components::box_renderer::{BoxRenderer, BoxDimensions};
                                        use crate::components::choice_menu::ChoiceMenu;
                                        use crate::components::renderable_content::RenderableContent;

                                        // Create BoxRenderer and ChoiceMenu
                                        let mut box_renderer = BoxRenderer::new(
                                            clicked_muxbox,
                                            format!("{}_click_renderer", clicked_muxbox.id),
                                        );
                                        
                                        let choice_menu = ChoiceMenu::new(
                                            format!("{}_choice_menu", clicked_muxbox.id),
                                            choices,
                                        )
                                        .with_selection(clicked_muxbox.selected_choice_index())
                                        .with_focus(clicked_muxbox.focused_choice_index());

                                        // Create formalized BoxDimensions
                                        let bounds = clicked_muxbox.bounds();
                                        let (content_width, content_height) = choice_menu.get_dimensions();
                                        let dimensions = BoxDimensions::new(
                                            clicked_muxbox, 
                                            &bounds, 
                                            content_width, 
                                            content_height
                                        );

                                        // Generate sensitive zones using formalized system with wrapping support
                                        let box_relative_zones = choice_menu.get_box_relative_sensitive_zones_with_width(dimensions.viewable_width);
                                        let translated_zones = box_renderer.translate_box_relative_zones_to_absolute(
                                            &box_relative_zones,
                                            &bounds,
                                            content_width,
                                            content_height,
                                            dimensions.viewable_width,
                                            dimensions.viewable_height,
                                            dimensions.horizontal_scroll,
                                            dimensions.vertical_scroll,
                                            false,
                                        );
                                        box_renderer.store_translated_sensitive_zones(translated_zones);

                                        // Handle click using formalized coordinate system
                                        if let Some(clicked_choice_idx) = box_renderer.handle_click_with_dimensions(
                                            *x as usize,
                                            *y as usize,
                                            &dimensions,
                                        ) {
                                            log::info!("CLICK: BoxRenderer detected click on choice {} for muxbox '{}'", clicked_choice_idx, clicked_muxbox.id);

                                            // Extract choice execution using formalized coordinate translation
                                            if let Some(choices) = clicked_muxbox.get_selected_stream_choices() {
                                                // Find clicked choice using screen-to-inbox coordinate translation
                                                let zones = box_renderer.get_sensitive_zones();
                                                let screen_x = *x as usize;
                                                let screen_y = *y as usize;
                                                
                                                // Convert to inbox coordinates for logging
                                                if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(screen_x, screen_y) {
                                                    log::info!("CLICK TRANSLATION: Screen ({},{}) -> Inbox ({},{})", 
                                                             screen_x, screen_y, inbox_x, inbox_y);
                                                }

                                                // Find clicked zone using screen coordinates (zones are stored in screen coords)
                                                if let Some(clicked_zone) = zones.iter().find(|z| {
                                                    z.bounds.contains_point(screen_x, screen_y)
                                                }) {
                                                    if let Some(idx_str) = clicked_zone.content_id.strip_prefix("choice_") {
                                                        if let Ok(clicked_choice_idx) = idx_str.parse::<usize>() {
                                                            log::info!("CLICK HANDLING: Successfully detected click on choice index {}", clicked_choice_idx);
                                                            if let Some(clicked_choice) =
                                                                choices.get(clicked_choice_idx)
                                                            {
                                                                log::trace!(
                                                                    "Clicked on choice: {}",
                                                                    clicked_choice.id
                                                                );

                                                                // First, select the parent muxbox if not already selected
                                                                let layout = app_context_for_click
                                                                    .app
                                                                    .get_active_layout_mut()
                                                                    .unwrap();
                                                                layout.deselect_all_muxboxes();
                                                                layout.select_only_muxbox(
                                                                    &clicked_muxbox.id,
                                                                );

                                                                // Then select the clicked choice visually
                                                                let muxbox_to_update =
                                                                    app_context_for_click
                                                                        .app
                                                                        .get_muxbox_by_id_mut(
                                                                            &clicked_muxbox.id,
                                                                        )
                                                                        .unwrap();
                                                                if let Some(muxbox_choices) =
                                                    muxbox_to_update.get_selected_stream_choices_mut()
                                                {
                                                    // Deselect all choices first
                                                    for choice in muxbox_choices.iter_mut() {
                                                        choice.selected = false;
                                                    }
                                                    // Select only the clicked choice and set waiting state for visual feedback
                                                    if let Some(selected_choice) =
                                                        muxbox_choices.get_mut(clicked_choice_idx)
                                                    {
                                                        selected_choice.selected = true;
                                                        selected_choice.waiting = true;
                                                        // Visual feedback consistency with Enter key
                                                    }
                                                }

                                                                // Update the app context and immediately trigger redraw for responsiveness
                                                                inner.update_app_context(
                                                                    app_context_for_click.clone(),
                                                                );
                                                                inner.send_message(
                                                                    Message::RedrawAppDiff,
                                                                );

                                                                // Then activate the clicked choice (same as pressing Enter)
                                                                // F0224: Use ExecutionMode to determine execution path for mouse clicks too
                                                                if let Some(script) =
                                                                    &clicked_choice.script
                                                                {
                                                                    let libs =
                                                                        app_context_unwrapped
                                                                            .app
                                                                            .libs
                                                                            .clone();

                                                                    let script_clone =
                                                                        script.clone();
                                                                    let choice_id_clone =
                                                                        clicked_choice.id.clone();
                                                                    let muxbox_id_clone =
                                                                        clicked_muxbox.id.clone();
                                                                    let libs_clone = libs.clone();
                                                                    let execution_mode =
                                                                        clicked_choice
                                                                            .execution_mode
                                                                            .clone();
                                                                    let redirect_output =
                                                                        clicked_choice
                                                                            .redirect_output
                                                                            .clone();
                                                                    let _append_output =
                                                                        clicked_choice
                                                                            .append_output
                                                                            .unwrap_or(false);

                                                                    // T0315: UNIFIED ARCHITECTURE - Replace legacy mouse click execution with ExecuteScript message
                                                                    log::info!("T0315: Mouse click creating ExecuteScript for choice {} (mode: {:?})", choice_id_clone, execution_mode);

                                                                    // Create ExecuteScript message instead of direct execution or legacy message routing
                                                                    use crate::model::common::{
                                                                        ExecuteScript,
                                                                        ExecutionSource,
                                                                        SourceReference,
                                                                        SourceType,
                                                                    };

                                                                    // Create choice object for SourceReference
                                                                    let choice_for_reference =
                                                                        Choice {
                                                                            id: choice_id_clone
                                                                                .clone(),
                                                                            content: Some(
                                                                                "".to_string(),
                                                                            ),
                                                                            selected: false,
                                                                            script: Some(
                                                                                script_clone
                                                                                    .clone(),
                                                                            ),
                                                                            execution_mode:
                                                                                execution_mode
                                                                                    .clone(),
                                                                            redirect_output:
                                                                                redirect_output
                                                                                    .clone(),
                                                                            append_output: Some(
                                                                                _append_output,
                                                                            ),
                                                                            waiting: true,
                                                                            hovered: false,
                                                                        };

                                                                    // Register execution source and get stream_id
                                                                    let source_type = crate::model::common::ExecutionSourceType::ChoiceExecution {
                                                        choice_id: choice_id_clone.clone(),
                                                        script: script_clone.clone(),
                                                        redirect_output: redirect_output.clone(),
                                                    };
                                                                    let stream_id = app_context_unwrapped
                                                        .app
                                                        .register_execution_source(
                                                            source_type,
                                                            muxbox_id_clone.clone(),
                                                        );

                                                                    let execute_script = ExecuteScript {
                                                        script: script_clone.clone(),
                                                        source: ExecutionSource {
                                                            source_type: SourceType::Choice(
                                                                choice_id_clone.clone(),
                                                            ),
                                                            source_id: format!(
                                                                "mouse_choice_{}",
                                                                choice_id_clone
                                                            ),
                                                            source_reference:
                                                                SourceReference::Choice(
                                                                    choice_for_reference,
                                                                ),
                                                        },
                                                        execution_mode: execution_mode.clone(),
                                                        target_box_id: muxbox_id_clone.clone(),
                                                        libs: libs_clone.unwrap_or_default(),
                                                        redirect_output: redirect_output.clone(),
                                                        append_output: _append_output,
                                                        stream_id: stream_id.clone(),
                                                        target_bounds: app_context_unwrapped.app.get_active_layout()
                                                            .and_then(|layout| layout.children.as_ref()?.iter().find(|mb| mb.id == *muxbox_id_clone))
                                                            .map(|mb| mb.bounds()),
                                                    };

                                                                    // Route ExecuteScript based on execution mode
                                                                    match execution_mode {
                                                        crate::model::common::ExecutionMode::Immediate |
                                                        crate::model::common::ExecutionMode::Thread => {
                                                            // Send to ThreadManager for Immediate/Thread execution
                                                            inner.send_message(Message::ExecuteScriptMessage(execute_script));
                                                            log::info!(
                                                                "T0315: ExecuteScript message sent to ThreadManager for mouse-clicked choice {} (mode: {:?})",
                                                                choice_id_clone, execution_mode
                                                            );
                                                        }
                                                        crate::model::common::ExecutionMode::Pty => {
                                                            // FIXED: Route PTY execution to PTYManager, never to ThreadManager
                                                            log::info!(
                                                                "T0315 FIXED: Routing PTY ExecuteScript to PTYManager for mouse-clicked choice {} (mode: {:?})",
                                                                choice_id_clone, execution_mode
                                                            );
                                                            // Route to PTYManager instead of ThreadManager
                                                            if let Some(pty_manager) = &app_context_unwrapped.pty_manager {
                                                                // Get message sender for PTY communication
                                                                if let Some(sender) = inner.get_message_sender() {
                                                                let uuid = uuid::Uuid::new_v4();
                                                                // Call PTYManager's ExecuteScript handler
                                                                if let Err(e) = pty_manager.handle_execute_script(&execute_script, sender.clone(), uuid) {
                                                                    log::error!(
                                                                        "T0315: PTYManager failed to handle ExecuteScript for choice {}: {}",
                                                                        choice_id_clone, e
                                                                    );
                                                                } else {
                                                                    log::info!(
                                                                        "T0315 FIXED: PTYManager successfully handling ExecuteScript for choice {} (architecture compliant)",
                                                                        choice_id_clone
                                                                    );
                                                                }
                                                                } else {
                                                                    log::error!("No message sender available for PTY execution - choice {}", choice_id_clone);
                                                                }
                                                            } else {
                                                                log::error!("No PTYManager available - PTY execution failed for choice {}", choice_id_clone);
                                                            }
                                                        }
                                                    }
                                                                }
                                                            } // Close if let Some(clicked_choice)
                                                        } // Close if let Ok(clicked_choice_idx)
                                                    } // Close if let Some(idx_str)
                                                } // Close if let Some(clicked_zone)
                                        } else {
                                            log::info!("NEW ARCH: Click at ({}, {}) did not hit any sensitive zone - muxbox already selected", *x, *y);
                                        }
                                            } // Close if !choices.is_empty()
                                        } else {
                                            // Selected stream has empty choices - muxbox already selected
                                            log::info!("CLICK DEBUG: MuxBox '{}' selected stream has empty choices - muxbox already selected", clicked_muxbox.id);
                                        }
                                        } // Close if let Some(choices)
                                    } else {
                                        // No selected stream - muxbox already selected
                                        log::info!("CLICK DEBUG: MuxBox '{}' has no selected stream - muxbox already selected", clicked_muxbox.id);
                                    } // Close if let Some(selected_stream)
                                } // End of clicked_muxbox
                            } // End of !handled_tab_click
                        }
                    }
                    Message::MouseMove(x, y) => {
                        // Handle mouse movement for hover detection on sensitive zones
                        // Use the EXACT same pattern as MouseClick handling
                        let screen_x = *x as usize;
                        let screen_y = *y as usize;
                        
                        let mut hover_state = HOVER_STATE.lock().unwrap();
                        let current_time = std::time::SystemTime::now();
                        
                        // Update position tracking
                        hover_state.last_position = Some((*x, *y));
                        
                        // Check if mouse is over any sensitive zones
                        let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();
                        let mut new_hovered_zone: Option<String> = None;
                        let mut new_hovered_muxbox: Option<String> = None;
                        
                        // Iterate through all muxboxes to find sensitive zones (same as click handling)
                        for hovered_muxbox in active_layout.get_all_muxboxes() {
                            let muxbox_bounds = hovered_muxbox.bounds();
                            
                            // Check if mouse is within this muxbox bounds first
                            if screen_x >= muxbox_bounds.left() && screen_x <= muxbox_bounds.right()
                                && screen_y >= muxbox_bounds.top() && screen_y <= muxbox_bounds.bottom()
                            {
                                // Check if this muxbox has choices (same condition as click handling)
                                if let Some(selected_stream) = hovered_muxbox.get_selected_stream() {
                                    if let Some(choices) = selected_stream.choices.as_ref() {
                                        if !choices.is_empty() {
                                            // Use EXACT same pattern as click detection
                                            use crate::components::box_renderer::{BoxRenderer, BoxDimensions};
                                            use crate::components::choice_menu::ChoiceMenu;
                                            use crate::components::renderable_content::RenderableContent;

                                            // Create BoxRenderer and ChoiceMenu (same as click handling)
                                            let mut box_renderer = BoxRenderer::new(
                                                hovered_muxbox,
                                                format!("{}_hover_renderer", hovered_muxbox.id),
                                            );
                                            
                                            let choice_menu = ChoiceMenu::new(
                                                format!("{}_choice_menu", hovered_muxbox.id),
                                                choices,
                                            )
                                            .with_selection(hovered_muxbox.selected_choice_index())
                                            .with_focus(hovered_muxbox.focused_choice_index());

                                            // Create formalized BoxDimensions (same as click handling)
                                            let bounds = hovered_muxbox.bounds();
                                            let (content_width, content_height) = choice_menu.get_dimensions();
                                            let dimensions = BoxDimensions::new(
                                                hovered_muxbox, 
                                                &bounds, 
                                                content_width, 
                                                content_height
                                            );

                                            // Generate sensitive zones using formalized system (same as click handling)
                                            let box_relative_zones = choice_menu.get_box_relative_sensitive_zones_with_width(dimensions.viewable_width);
                                            let translated_zones = box_renderer.translate_box_relative_zones_to_absolute(
                                                &box_relative_zones,
                                                &bounds,
                                                content_width,
                                                content_height,
                                                dimensions.viewable_width,
                                                dimensions.viewable_height,
                                                dimensions.horizontal_scroll,
                                                dimensions.vertical_scroll,
                                                false,
                                            );
                                            box_renderer.store_translated_sensitive_zones(translated_zones);

                                            // Use EXACT same zone detection as click handling (no coordinate translation)
                                            let zones = box_renderer.get_sensitive_zones();
                                            if let Some(hovered_zone) = zones.iter().find(|z| {
                                                z.bounds.contains_point(screen_x, screen_y)
                                            }) {
                                                new_hovered_zone = Some(hovered_zone.content_id.clone());
                                                new_hovered_muxbox = Some(hovered_muxbox.id.clone());
                                                break;
                                            }
                                        }
                                    }
                                }
                                
                                if new_hovered_zone.is_some() {
                                    break;
                                }
                            }
                        }
                        
                        // Check for hover state changes
                        let previous_zone = hover_state.current_zone.clone();
                        let zone_changed = previous_zone != new_hovered_zone;
                        
                        if zone_changed {
                            let mut app_context_for_hover = app_context_unwrapped.clone();
                            
                            // Clear previous hover state from choices
                            if let Some(prev_zone) = &previous_zone {
                                if let Some(prev_muxbox_id) = &hover_state.current_muxbox {
                                    if let Some(prev_muxbox) = app_context_for_hover.app.get_muxbox_by_id_mut(prev_muxbox_id) {
                                        if let Some(choices) = prev_muxbox.get_selected_stream_choices_mut() {
                                            // Find and unhover the previous choice
                                            if let Some(idx_str) = prev_zone.strip_prefix("choice_") {
                                                if let Ok(choice_idx) = idx_str.parse::<usize>() {
                                                    if let Some(choice) = choices.get_mut(choice_idx) {
                                                        choice.hovered = false;
                                                        log::trace!("Hover leave: {} (choice_{})", prev_zone, choice_idx);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            
                            // Set new hover state on choices
                            if let Some(new_zone) = &new_hovered_zone {
                                if let Some(new_muxbox_id) = &new_hovered_muxbox {
                                    if let Some(new_muxbox) = app_context_for_hover.app.get_muxbox_by_id_mut(new_muxbox_id) {
                                        if let Some(choices) = new_muxbox.get_selected_stream_choices_mut() {
                                            // Find and hover the new choice
                                            if let Some(idx_str) = new_zone.strip_prefix("choice_") {
                                                if let Ok(choice_idx) = idx_str.parse::<usize>() {
                                                    if let Some(choice) = choices.get_mut(choice_idx) {
                                                        choice.hovered = true;
                                                        log::trace!("Hover enter: {} (choice_{})", new_zone, choice_idx);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                hover_state.hover_start_time = Some(current_time);
                            } else {
                                hover_state.hover_start_time = None;
                            }
                            
                            // Update hover state and trigger redraw
                            hover_state.current_zone = new_hovered_zone;
                            hover_state.current_muxbox = new_hovered_muxbox;
                            
                            // Update app context and trigger redraw to show hover effects
                            inner.update_app_context(app_context_for_hover);
                            inner.send_message(Message::RedrawAppDiff);
                        } else if hover_state.current_zone.is_some() {
                            // Mouse moved within the same zone - could generate HoverState::Move event
                            log::trace!("Hover move within zone: {:?} at ({}, {})", hover_state.current_zone, screen_x, screen_y);
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
                            let mut _handled_move = false;
                            if !handled_resize {
                                let mut move_state = MUXBOX_MOVE_STATE.lock().unwrap();
                                *move_state = None; // Clear any previous move state

                                for muxbox in active_layout.get_all_muxboxes() {
                                    if detect_move_area(muxbox, *x, *y) {
                                        // Check if the drag started on a tab area - if so, don't start move
                                        let tab_labels = muxbox.get_tab_labels();
                                        let muxbox_bounds = muxbox.bounds();
                                        if let Some(_tab_index) =
                                            crate::draw_utils::calculate_tab_click_index(
                                                *x as usize,
                                                muxbox_bounds.left(),
                                                muxbox_bounds.right(),
                                                &tab_labels,
                                                muxbox.tab_scroll_offset,
                                                &muxbox.calc_border_color(
                                                    &app_context_unwrapped,
                                                    &app_graph,
                                                ),
                                                &muxbox.bg_color,
                                            )
                                        {
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
                                        _handled_move = true;
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
                                        .clamp(0.0, 100.0);

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
                                        .clamp(0.0, 100.0);

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
                            log::error!(
                                "CRITICAL: No YAML file path available for saving complete state!"
                            );
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
                            log::error!(
                                "CRITICAL: No YAML file path available for saving active layout!"
                            );
                        }
                    }
                    Message::SaveMuxBoxContent(muxbox_id, content) => {
                        // F0200: Save muxbox content changes to YAML
                        log::debug!("Saving content changes to YAML for muxbox: {}", muxbox_id);

                        if let Some(yaml_path) = &app_context_unwrapped.yaml_file_path {
                            match save_muxbox_content_to_yaml(yaml_path, muxbox_id, content) {
                                Ok(()) => {
                                    log::debug!(
                                        "Successfully saved muxbox {} content to YAML",
                                        muxbox_id
                                    );
                                }
                                Err(e) => {
                                    log::error!(
                                        "Failed to save muxbox {} content to YAML: {}",
                                        muxbox_id,
                                        e
                                    );
                                }
                            }
                        } else {
                            log::warn!("No YAML file path available for saving muxbox content");
                        }
                    }
                    Message::SaveMuxBoxScroll(muxbox_id, scroll_x, scroll_y) => {
                        // F0200: Save muxbox scroll position to YAML
                        log::debug!(
                            "Saving scroll position to YAML for muxbox: {} ({}, {})",
                            muxbox_id,
                            scroll_x,
                            scroll_y
                        );

                        if let Some(yaml_path) = &app_context_unwrapped.yaml_file_path {
                            match save_muxbox_scroll_to_yaml(
                                yaml_path, muxbox_id, *scroll_x, *scroll_y,
                            ) {
                                Ok(()) => {
                                    log::debug!(
                                        "Successfully saved muxbox {} scroll position to YAML",
                                        muxbox_id
                                    );
                                }
                                Err(e) => {
                                    log::error!(
                                        "Failed to save muxbox {} scroll position to YAML: {}",
                                        muxbox_id,
                                        e
                                    );
                                }
                            }
                        } else {
                            log::warn!(
                                "No YAML file path available for saving muxbox scroll position"
                            );
                        }
                    }
                    Message::SwitchActiveLayout(layout_id) => {
                        // F0200: Switch active layout with YAML persistence
                        log::info!("Switching to active layout: {}", layout_id);

                        // Update the active layout in app context
                        let mut app_context_cloned = app_context_unwrapped.clone();
                        match app_context_cloned.app.set_active_layout_with_yaml_save(
                            layout_id,
                            app_context_cloned.yaml_file_path.as_deref(),
                        ) {
                            Ok(()) => {
                                inner.update_app_context(app_context_cloned);
                                inner.send_message(Message::RedrawApp);
                                log::info!(
                                    "Successfully switched to layout '{}' with YAML persistence",
                                    layout_id
                                );
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
                        log::debug!(
                            "Processing SwitchTab message: muxbox={}, tab_index={}",
                            muxbox_id,
                            tab_index
                        );
                        if let Some(muxbox) =
                            app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                        {
                            log::debug!(
                                "Found muxbox '{}', attempting to switch to tab {}",
                                muxbox_id,
                                tab_index
                            );
                            if muxbox.switch_to_tab(*tab_index) {
                                log::info!(
                                    "Successfully switched muxbox '{}' to tab {}",
                                    muxbox_id,
                                    tab_index
                                );
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                            } else {
                                log::warn!("Failed to switch muxbox '{}' to tab {} - switch_to_tab returned false", muxbox_id, tab_index);
                            }
                        } else {
                            log::error!("SwitchTab message for non-existent muxbox: {}", muxbox_id);
                        }
                    }
                    Message::ScrollTabsLeft(muxbox_id) => {
                        log::debug!("Processing ScrollTabsLeft message: muxbox={}", muxbox_id);
                        if let Some(muxbox) =
                            app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                        {
                            muxbox.scroll_tabs_left();
                            log::info!(
                                "Scrolled tabs left for muxbox '{}', new offset: {}",
                                muxbox_id,
                                muxbox.tab_scroll_offset
                            );
                            inner.update_app_context(app_context_unwrapped.clone());
                            inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                        } else {
                            log::error!(
                                "ScrollTabsLeft message for non-existent muxbox: {}",
                                muxbox_id
                            );
                        }
                    }
                    Message::ScrollTabsRight(muxbox_id) => {
                        log::debug!("Processing ScrollTabsRight message: muxbox={}", muxbox_id);
                        if let Some(muxbox) =
                            app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                        {
                            muxbox.scroll_tabs_right();
                            log::info!(
                                "Scrolled tabs right for muxbox '{}', new offset: {}",
                                muxbox_id,
                                muxbox.tab_scroll_offset
                            );
                            inner.update_app_context(app_context_unwrapped.clone());
                            inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                        } else {
                            log::error!(
                                "ScrollTabsRight message for non-existent muxbox: {}",
                                muxbox_id
                            );
                        }
                    }
                    Message::SwitchToStream(muxbox_id, stream_id) => {
                        if let Some(muxbox) =
                            app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                        {
                            if muxbox.switch_to_stream(stream_id) {
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                            }
                        }
                    }
                    Message::AddStream(muxbox_id, _stream) => {
                        if let Some(_muxbox) =
                            app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                        {
                            // AddStream is deprecated - use add_input_stream() method directly on muxbox instead
                            log::warn!("AddStream message is deprecated - use muxbox.add_input_stream() instead");
                            inner.update_app_context(app_context_unwrapped.clone());
                            inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                        }
                    }
                    Message::RemoveStream(muxbox_id, stream_id) => {
                        if let Some(muxbox) =
                            app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                        {
                            // F0213: Stream Lifecycle Management - cleanup stream sources before removal
                            if let Some(source) = muxbox.remove_stream(stream_id) {
                                log::info!(
                                    "Cleaning up stream source for stream {}: {:?}",
                                    stream_id,
                                    source
                                );

                                // Perform source cleanup based on source type
                                match source {
                                    crate::model::common::StreamSource::ChoiceExecution(
                                        choice_source,
                                    ) => {
                                        if let Err(e) = choice_source.cleanup() {
                                            log::warn!(
                                                "Failed to cleanup choice execution source: {}",
                                                e
                                            );
                                        }
                                    }
                                    // F0227: ExecutionMode-specific source cleanup
                                    crate::model::common::StreamSource::ImmediateExecution(
                                        source,
                                    ) => {
                                        if let Err(e) = source.cleanup() {
                                            log::warn!(
                                                "Failed to cleanup immediate execution source: {}",
                                                e
                                            );
                                        }
                                    }
                                    crate::model::common::StreamSource::ThreadPoolExecution(
                                        source,
                                    ) => {
                                        if let Err(e) = source.cleanup() {
                                            log::warn!("Failed to cleanup thread pool execution source: {}", e);
                                        }
                                    }
                                    crate::model::common::StreamSource::PtySessionExecution(
                                        source,
                                    ) => {
                                        if let Err(e) = source.cleanup() {
                                            log::warn!("Failed to cleanup PTY session execution source: {}", e);
                                        }
                                    }
                                    crate::model::common::StreamSource::PTY(pty_source) => {
                                        if let Err(e) = pty_source.cleanup() {
                                            log::warn!("Failed to cleanup PTY source: {}", e);
                                        }
                                    }
                                    crate::model::common::StreamSource::Redirect(
                                        redirect_source,
                                    ) => {
                                        if let Err(e) = redirect_source.cleanup() {
                                            log::warn!("Failed to cleanup redirect source: {}", e);
                                        }
                                    }
                                    crate::model::common::StreamSource::Socket(socket_source) => {
                                        if let Err(e) = socket_source.cleanup() {
                                            log::warn!("Failed to cleanup socket source: {}", e);
                                        }
                                    }
                                    crate::model::common::StreamSource::StaticContent(_) => {
                                        // Static content sources don't need cleanup
                                        log::debug!(
                                            "Static content source removed - no cleanup needed"
                                        );
                                    }
                                    crate::model::common::StreamSource::PeriodicRefresh(
                                        periodic_source,
                                    ) => {
                                        if let Err(e) = periodic_source.cleanup() {
                                            log::warn!(
                                                "Failed to cleanup periodic refresh source: {}",
                                                e
                                            );
                                        }
                                    }
                                }

                                // Stream already removed by muxbox.remove_stream() call above
                                inner.update_app_context(app_context_unwrapped.clone());
                                inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                            } else {
                                log::warn!(
                                    "Stream {} not found in muxbox {} for cleanup",
                                    stream_id,
                                    muxbox_id
                                );
                            }
                        }
                    }
                    Message::CloseTab(muxbox_id, stream_id) => {
                        // F0219: Handle close tab request - handle terminated sources gracefully
                        log::info!(
                            "Close tab requested for stream {} in muxbox {}",
                            stream_id,
                            muxbox_id
                        );

                        // Check if stream is closeable before closing
                        if let Some(muxbox) =
                            app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                        {
                            if let Some(stream) = muxbox.streams.get(stream_id) {
                                if stream.is_closeable() {
                                    // Handle stream removal directly - source threads may be terminated
                                    log::info!(
                                        "Processing close tab for closeable stream {} in muxbox {}",
                                        stream_id,
                                        muxbox_id
                                    );

                                    // Remove stream and handle source cleanup
                                    if let Some(source) = muxbox.remove_stream(stream_id) {
                                        log::info!(
                                            "Stream {} removed from muxbox {}, attempting source cleanup",
                                            stream_id,
                                            muxbox_id
                                        );

                                        // Attempt source cleanup - ignore failures for terminated threads
                                        match source {
                                            crate::model::common::StreamSource::ChoiceExecution(
                                                choice_source,
                                            ) => {
                                                if let Err(e) = choice_source.cleanup() {
                                                    log::info!(
                                                        "Choice execution source cleanup failed (likely already terminated): {}",
                                                        e
                                                    );
                                                }
                                            }
                                            // F0227: ExecutionMode-specific source cleanup
                                            crate::model::common::StreamSource::ImmediateExecution(source) => {
                                                if let Err(e) = source.cleanup() {
                                                    log::info!("Immediate execution source cleanup failed: {}", e);
                                                }
                                            }
                                            crate::model::common::StreamSource::ThreadPoolExecution(source) => {
                                                if let Err(e) = source.cleanup() {
                                                    log::info!("Thread pool execution source cleanup failed: {}", e);
                                                }
                                            }
                                            crate::model::common::StreamSource::PtySessionExecution(source) => {
                                                if let Err(e) = source.cleanup() {
                                                    log::info!("PTY session execution source cleanup failed: {}", e);
                                                }
                                            }
                                            crate::model::common::StreamSource::PTY(pty_source) => {
                                                if let Err(e) = pty_source.cleanup() {
                                                    log::info!("PTY source cleanup failed (likely already terminated): {}", e);
                                                }
                                            }
                                            crate::model::common::StreamSource::Redirect(
                                                redirect_source,
                                            ) => {
                                                if let Err(e) = redirect_source.cleanup() {
                                                    log::info!("Redirect source cleanup failed (likely already terminated): {}", e);
                                                }
                                            }
                                            crate::model::common::StreamSource::Socket(socket_source) => {
                                                if let Err(e) = socket_source.cleanup() {
                                                    log::info!("Socket source cleanup failed (likely already terminated): {}", e);
                                                }
                                            }
                                            crate::model::common::StreamSource::StaticContent(_) => {
                                                log::debug!("Static content source removed - no cleanup needed");
                                            }
                                            crate::model::common::StreamSource::PeriodicRefresh(periodic_source) => {
                                                if let Err(e) = periodic_source.cleanup() {
                                                    log::info!("Periodic refresh source cleanup failed (likely already terminated): {}", e);
                                                }
                                            }
                                        }

                                        // Update app context and trigger redraw
                                        inner.update_app_context(app_context_unwrapped.clone());
                                        inner
                                            .send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                                        log::info!("Stream {} successfully closed and removed from muxbox {}", stream_id, muxbox_id);
                                    } else {
                                        log::warn!(
                                            "Stream {} not found in muxbox {} during removal",
                                            stream_id,
                                            muxbox_id
                                        );
                                    }
                                } else {
                                    log::warn!(
                                        "Attempted to close non-closeable stream {} in muxbox {}",
                                        stream_id,
                                        muxbox_id
                                    );
                                }
                            } else {
                                log::warn!(
                                    "Stream {} not found in muxbox {} for close operation",
                                    stream_id,
                                    muxbox_id
                                );
                            }
                        } else {
                            log::warn!("MuxBox {} not found for close tab operation", muxbox_id);
                        }
                    }
                    Message::UpdateStreamContent(muxbox_id, stream_id, content) => {
                        if let Some(muxbox) =
                            app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                        {
                            // Update stream content directly using new stream system
                            if let Some(stream) = muxbox.streams.get_mut(stream_id) {
                                stream.content = content.lines().map(|s| s.to_string()).collect();
                                
                                // AUTO_SCROLL_BOTTOM FIX: Apply auto-scroll after stream content update
                                if muxbox.auto_scroll_bottom == Some(true) {
                                    muxbox.vertical_scroll = Some(100.0);
                                    log::debug!("Applied auto-scroll to bottom for muxbox {} after UpdateStreamContent", muxbox_id);
                                }
                            } else {
                                log::warn!(
                                    "Stream {} not found in muxbox {} for content update",
                                    stream_id,
                                    muxbox_id
                                );
                            }
                            inner.update_app_context(app_context_unwrapped.clone());
                            inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
                        }
                    }
                    // T0318: UNIFIED ARCHITECTURE - Replace legacy socket script execution with ExecuteScript message
                    Message::MuxBoxScriptUpdate(muxbox_id, new_script) => {
                        log::info!("T0318: Socket script update creating ExecuteScript for muxbox {} ({} commands)", muxbox_id, new_script.len());

                        // Clone libs before mutable borrow to avoid borrowing conflict
                        let libs = app_context_unwrapped.app.libs.clone().unwrap_or_default();

                        // Collect all needed data in one scope to avoid borrow conflicts
                        let (execution_mode, redirect_output, append_output) = {
                            if let Some(muxbox) =
                                app_context_unwrapped.app.get_muxbox_by_id_mut(muxbox_id)
                            {
                                // Update the muxbox script field and collect needed data
                                muxbox.script = Some(new_script.clone());
                                (
                                    muxbox.execution_mode.clone(),
                                    muxbox.redirect_output.clone(),
                                    muxbox.append_output.unwrap_or(false),
                                )
                            } else {
                                log::warn!("Muxbox {} not found for script update", muxbox_id);
                                continue;
                            }
                        };

                        // Now register execution source with a fresh mutable borrow
                        // Create ExecuteScript message for socket-triggered script execution
                        use crate::model::common::{
                            ExecuteScript, ExecutionSource, SourceReference, SourceType,
                        };

                        // Register execution source and get stream_id
                        let source_type = crate::model::common::ExecutionSourceType::SocketUpdate {
                            command_type: "replace-box-script".to_string(),
                        };
                        let stream_id = app_context_unwrapped
                            .app
                            .register_execution_source(source_type, muxbox_id.clone());

                        let execute_script = ExecuteScript {
                            script: new_script.clone(),
                            source: ExecutionSource {
                                source_type: SourceType::SocketUpdate,
                                source_id: format!("socket_script_{}", muxbox_id),
                                source_reference: SourceReference::SocketCommand(format!(
                                    "replace-box-script command for {}",
                                    muxbox_id
                                )),
                            },
                            execution_mode,
                            target_box_id: muxbox_id.clone(),
                            libs,
                            redirect_output,
                            append_output,
                            stream_id,
                            target_bounds: app_context_unwrapped
                                .app
                                .get_active_layout()
                                .and_then(|layout| {
                                    layout
                                        .children
                                        .as_ref()?
                                        .iter()
                                        .find(|mb| mb.id == *muxbox_id)
                                })
                                .map(|mb| mb.bounds()),
                        };

                        // Send ExecuteScript message instead of direct execution
                        inner.send_message(Message::ExecuteScriptMessage(execute_script));

                        log::info!(
                                "T0318: ExecuteScript message sent for socket-updated muxbox {} script (unified architecture)",
                                muxbox_id
                            );

                        inner.update_app_context(app_context_unwrapped.clone());
                    }
                    _ => {}
                }
            }

            // T311: Choice execution now handled via ChoiceExecutionComplete messages
            // Old POOL-based choice results processing removed
        }

        // Ensure the loop continues by sleeping briefly
        std::thread::sleep(std::time::Duration::from_millis(
            app_context.config.frame_delay,
        ));

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
                found_muxbox
                    .get_selected_stream()
                    .map_or(0, |s| s.content.join("\n").len())
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
                found_muxbox
                    .get_selected_stream()
                    .map_or(0, |s| s.content.join("\n").len())
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
            "Found target muxbox: {} with streams: {}",
            muxbox_id,
            found_muxbox.streams.len()
        );

        // F0229: Clean output formatting without unwanted timestamps
        let formatted_output = if success {
            output.to_string()
        } else {
            format!("ERROR: {}", output)
        };

        // Update the specific stream content - find stream by actual stream ID
        let mut stream_updated = false;

        // Check if stream exists and update content
        if found_muxbox.streams.contains_key(stream_id) {
            // Update the stream content
            if let Some(stream) = found_muxbox.streams.get_mut(stream_id) {
                if append_output {
                    stream.content.push(formatted_output.clone());
                } else {
                    stream.content = vec![formatted_output.clone()];
                }
                stream_updated = true;
            }

            // Check if we need to select this stream (if no stream is currently selected)
            let should_activate = found_muxbox.selected_stream_id.is_none() || found_muxbox.streams.len() == 1;

            if should_activate {
                // Set this stream as selected
                found_muxbox.selected_stream_id = Some(stream_id.to_string());
            }

            log::info!("Updated stream {} with new content", stream_id);
        }

        if !stream_updated {
            log::warn!(
                "Stream {} not found in muxbox {}, fallback to updating content",
                stream_id,
                muxbox_id
            );
            // Fallback to updating the muxbox content directly
            found_muxbox.update_content(&formatted_output, append_output, success);
        }

        log::info!(
            "Updated stream {} content in muxbox {}",
            stream_id,
            muxbox_id
        );

        inner.update_app_context(app_context_unwrapped.clone());
        inner.send_message(Message::RedrawMuxBox(muxbox_id.to_string()));
        log::info!("Sent RedrawMuxBox message for muxbox: {}", muxbox_id);
    } else {
        log::error!(
            "Could not find muxbox {} for stream content update.",
            muxbox_id
        );
    }
}

/// Extract muxbox content for clipboard copy
pub fn get_muxbox_content_for_clipboard(muxbox: &MuxBox) -> String {
    // F0215: Stream-Based Choice Navigation - Use stream content for clipboard
    // Priority order: output > stream content > static content > default message
    if !muxbox.output.is_empty() {
        muxbox.output.clone()
    } else {
        let stream_content = muxbox
            .get_selected_stream()
            .map_or(Vec::new(), |s| s.content.clone());
        if !stream_content.is_empty() {
            stream_content.join("\n")
        } else if let Some(ref content) = muxbox.content {
            content.clone()
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

// REMOVED: Legacy calculate_clicked_choice_index - replaced by BoxDimensions coordinate system

// REMOVED: Legacy calculate_clicked_choice_index_impl - replaced by BoxDimensions coordinate system

// REMOVED: Legacy calculate_wrapped_choice_click - replaced by BoxDimensions coordinate system

// REMOVED: Legacy wrap_text_to_width_simple - text wrapping handled by RenderableContent components

/// Auto-scroll to ensure selected choice is visible
fn auto_scroll_to_selected_choice(
    muxbox: &mut crate::model::muxbox::MuxBox,
    selected_choice_index: usize,
) {
    use crate::draw_utils::wrap_text_to_width;

    let bounds = muxbox.bounds();
    let viewable_height = bounds.height().saturating_sub(2); // Account for borders

    // Handle different overflow behaviors
    if let Some(overflow_behavior) = &muxbox.overflow_behavior {
        match overflow_behavior.as_str() {
            "wrap" => {
                // Calculate wrapped lines for auto-scroll in wrapped choice mode
                if let Some(choices) = muxbox.get_selected_stream_choices() {
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

                            let wrapped_lines =
                                wrap_text_to_width(&formatted_content, viewable_width);
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
                        let current_scroll_offset = ((current_scroll_percent / 100.0)
                            * (total_lines - viewable_height) as f64)
                            .floor() as usize;
                        let visible_start = current_scroll_offset;
                        let visible_end = visible_start + viewable_height - 1;

                        let mut new_scroll_percent = current_scroll_percent;

                        // Scroll down if selected choice is below visible area
                        if selected_line_end > visible_end {
                            let new_offset = selected_line_end.saturating_sub(viewable_height - 1);
                            new_scroll_percent = (new_offset as f64
                                / (total_lines - viewable_height) as f64)
                                * 100.0;
                        }
                        // Scroll up if selected choice is above visible area
                        else if selected_line_start < visible_start {
                            let new_offset = selected_line_start;
                            new_scroll_percent = (new_offset as f64
                                / (total_lines - viewable_height) as f64)
                                * 100.0;
                        }

                        muxbox.vertical_scroll = Some(new_scroll_percent.clamp(0.0, 100.0));
                    }
                }
            }
            "scroll" => {
                // For scroll mode, use choice index directly
                if let Some(choices) = muxbox.get_selected_stream_choices() {
                    let total_choices = choices.len();
                    if total_choices > viewable_height {
                        let current_scroll_percent = muxbox.vertical_scroll.unwrap_or(0.0);
                        let current_scroll_offset = ((current_scroll_percent / 100.0)
                            * (total_choices - viewable_height) as f64)
                            .floor() as usize;
                        let visible_start = current_scroll_offset;
                        let visible_end = visible_start + viewable_height - 1;

                        let mut new_scroll_percent = current_scroll_percent;

                        // Scroll down if selected choice is below visible area
                        if selected_choice_index > visible_end {
                            let new_offset =
                                selected_choice_index.saturating_sub(viewable_height - 1);
                            new_scroll_percent = (new_offset as f64
                                / (total_choices - viewable_height) as f64)
                                * 100.0;
                        }
                        // Scroll up if selected choice is above visible area
                        else if selected_choice_index < visible_start {
                            let new_offset = selected_choice_index;
                            new_scroll_percent = (new_offset as f64
                                / (total_choices - viewable_height) as f64)
                                * 100.0;
                        }

                        muxbox.vertical_scroll = Some(new_scroll_percent.clamp(0.0, 100.0));
                    }
                }
            }
            _ => {
                // For other overflow behaviors (fill, cross_out, etc.), use simple choice index
                if let Some(choices) = muxbox.get_selected_stream_choices() {
                    let total_choices = choices.len();
                    if total_choices > viewable_height {
                        let current_scroll_percent = muxbox.vertical_scroll.unwrap_or(0.0);
                        let current_scroll_offset = ((current_scroll_percent / 100.0)
                            * (total_choices - viewable_height) as f64)
                            .floor() as usize;
                        let visible_start = current_scroll_offset;
                        let visible_end = visible_start + viewable_height - 1;

                        let mut new_scroll_percent = current_scroll_percent;

                        if selected_choice_index > visible_end {
                            let new_offset =
                                selected_choice_index.saturating_sub(viewable_height - 1);
                            new_scroll_percent = (new_offset as f64
                                / (total_choices - viewable_height) as f64)
                                * 100.0;
                        } else if selected_choice_index < visible_start {
                            let new_offset = selected_choice_index;
                            new_scroll_percent = (new_offset as f64
                                / (total_choices - viewable_height) as f64)
                                * 100.0;
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
