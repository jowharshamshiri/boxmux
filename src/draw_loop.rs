use crate::draw_utils::{draw_app, draw_muxbox};
use crate::model::app::{
    save_active_layout_to_yaml, save_complete_state_to_yaml, save_muxbox_bounds_to_yaml,
    save_muxbox_content_to_yaml, save_muxbox_scroll_to_yaml,
};
use crate::model::common::{
    ExecutionMode, InputBounds, StreamSourceTrait, StreamType, StreamSource
};
use crate::model::muxbox::Choice;
use crate::thread_manager::Runnable;
// use crate::utils::run_script_with_pty_and_redirect; // F0229: No longer used in unified system
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
                    // T0305-T0308: UNIFIED EXECUTION ARCHITECTURE - Phase 2 handlers
                    Message::ExecuteScriptMessage(execute_script) => {
                        log::info!("Processing ExecuteScript for target_box_id: {}, execution_mode: {:?}", 
                                   execute_script.target_box_id, execute_script.execution_mode);
                        
                        // T0306: Stream-first creation logic - create stream BEFORE execution begins
                        let stream_id = Uuid::new_v4().to_string();
                        let source_id = execute_script.source.source_id.clone();
                        
                        // Create stream in target box first
                        if let Some(target_muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(&execute_script.target_box_id) {
                            // Create stream label based on source type
                            let stream_label = match &execute_script.source.source_type {
                                crate::model::common::SourceType::Choice(choice_id) => choice_id.clone(),
                                crate::model::common::SourceType::StaticScript => "Script".to_string(),
                                crate::model::common::SourceType::SocketUpdate => "Socket".to_string(),
                                crate::model::common::SourceType::RedirectedScript => "Redirect".to_string(),
                                crate::model::common::SourceType::HotkeyScript => "Hotkey".to_string(),
                                crate::model::common::SourceType::ScheduledScript => "Scheduled".to_string(),
                            };
                            
                            // Create execution stream with appropriate type
                            let stream_type = match execute_script.execution_mode {
                                crate::model::common::ExecutionMode::Immediate => StreamType::ChoiceExecution(source_id.clone()),
                                crate::model::common::ExecutionMode::Thread => StreamType::ChoiceExecution(source_id.clone()),
                                crate::model::common::ExecutionMode::Pty => StreamType::PtySession(format!("PTY-{}", source_id)),
                            };
                            
                            let new_stream = crate::model::common::Stream::new(
                                stream_id.clone(),
                                stream_type,
                                stream_label,
                                Vec::new(),
                                None,
                                None, // TODO: Add proper StreamSource in future phase
                            );
                            
                            // Add stream to target muxbox streams HashMap directly
                            target_muxbox.streams.insert(stream_id.clone(), new_stream);
                            
                            log::info!("Created stream {} in box {} for execution", stream_id, execute_script.target_box_id);
                        } else {
                            log::error!("Target box {} not found for ExecuteScript", execute_script.target_box_id);
                            continue;
                        }
                        
                        // T0307: Execution mode dispatch - delegate to appropriate execution handler
                        match execute_script.execution_mode {
                            crate::model::common::ExecutionMode::Immediate => {
                                // Batch mode - queue for dedicated batch thread (future implementation)
                                log::info!("Batch execution not yet implemented - using immediate execution");
                                // For now, execute immediately (will be moved to batch queue in later phase)
                                let execution_result = crate::run_script(Some(execute_script.libs.clone()), &execute_script.script);
                                
                                // Create StreamUpdate message with result
                                let source_state = crate::model::common::SourceState::Batch(crate::model::common::BatchSourceState {
                                    task_id: source_id.clone(),
                                    queue_wait_time: std::time::Duration::from_millis(0),
                                    execution_time: std::time::Duration::from_millis(100), // Placeholder
                                    exit_code: None,
                                    status: match execution_result {
                                        Ok(_) => crate::model::common::BatchStatus::Completed,
                                        Err(ref e) => crate::model::common::BatchStatus::Failed(e.to_string()),
                                    },
                                });
                                
                                let content_update = match execution_result {
                                    Ok(output) => output,
                                    Err(e) => format!("Error: {}", e),
                                };
                                
                                let stream_update = crate::model::common::StreamUpdate {
                                    stream_id: stream_id.clone(),
                                    content_update,
                                    source_state,
                                    execution_mode: execute_script.execution_mode.clone(),
                                };
                                
                                // Send StreamUpdate message back to self
                                inner.send_message(Message::StreamUpdateMessage(stream_update));
                            }
                            crate::model::common::ExecutionMode::Thread => {
                                // Thread mode - dispatch to general thread pool (future implementation)
                                log::info!("Thread execution dispatch not yet implemented - using immediate execution");
                                // Placeholder - will dispatch to ThreadManager in later phase
                            }
                            crate::model::common::ExecutionMode::Pty => {
                                // PTY mode - spawn PTY process (future implementation)
                                log::info!("PTY execution dispatch not yet implemented");
                                // Placeholder - will spawn PTY process in later phase
                            }
                        }
                    }
                    Message::StreamUpdateMessage(stream_update) => {
                        log::info!("Processing StreamUpdate for stream_id: {}, execution_mode: {:?}", 
                                   stream_update.stream_id, stream_update.execution_mode);
                        
                        // T0308: StreamUpdate handler - append content to target stream
                        // Find the target stream across all muxboxes
                        let mut stream_found = false;
                        for layout in &mut app_context_unwrapped.app.layouts {
                            if let Some(children) = &mut layout.children {
                                for muxbox in children {
                                    if let Some(stream) = muxbox.streams.get_mut(&stream_update.stream_id) {
                                        // Append content to stream
                                        if !stream_update.content_update.is_empty() {
                                            stream.content.push(stream_update.content_update.clone());
                                            log::debug!("Appended content to stream {}: {} characters", 
                                                        stream_update.stream_id, stream_update.content_update.len());
                                        }
                                        
                                        // Update source state tracking (future implementation)
                                        // TODO: Store source_state in stream for lifecycle management
                                        
                                        stream_found = true;
                                        
                                        // Trigger redraw of the containing muxbox
                                        inner.send_message(Message::RedrawMuxBox(muxbox.id.clone()));
                                        break;
                                    }
                                }
                                if stream_found {
                                    break;
                                }
                            }
                        }
                        
                        if !stream_found {
                            log::warn!("Stream {} not found for update", stream_update.stream_id);
                        }
                    }
                    Message::SourceActionMessage(source_action) => {
                        log::info!("Processing SourceAction: {:?} for source_id: {}, execution_mode: {:?}", 
                                   source_action.action, source_action.source_id, source_action.execution_mode);
                        
                        // T0320: SourceAction handler - Phase 4 source lifecycle management implementation
                        match source_action.action {
                            crate::model::common::ActionType::Kill => {
                                log::info!("Kill action for source {} (mode: {:?})", source_action.source_id, source_action.execution_mode);
                                
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
                                                    if id.starts_with("PTY-") {
                                                        Some(id[4..].to_string())
                                                    } else {
                                                        Some(id.clone())
                                                    }
                                                },
                                                _ => None,
                                            };
                                            
                                            if let Some(stream_src_id) = stream_source_id {
                                                if stream_src_id == source_action.source_id {
                                                    log::info!("Found stream {} with source_id {} for termination", stream_id, source_action.source_id);
                                                    stream_to_update = Some((stream_id.clone(), muxbox.id.clone()));
                                                    source_terminated = true;
                                                    break;
                                                }
                                            }
                                        }
                                        if source_terminated { break; }
                                    }
                                    if source_terminated { break; }
                                }
                                
                                // Now perform the actual cleanup if we found the stream
                                if let Some((stream_id, muxbox_id)) = stream_to_update {
                                    // Get mutable access to perform cleanup
                                    if let Some(target_muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(&muxbox_id) {
                                        if let Some(stream) = target_muxbox.streams.get(&stream_id) {
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
                                            
                                            let termination_update = crate::model::common::StreamUpdate {
                                                stream_id: stream_id.clone(),
                                                content_update: "\n[Process terminated by user]".to_string(),
                                                source_state: terminated_state,
                                                execution_mode: source_action.execution_mode.clone(),
                                            };
                                            
                                            inner.send_message(Message::StreamUpdateMessage(termination_update));
                                        }
                                    }
                                } else {
                                    log::warn!("Source {} not found for kill action", source_action.source_id);
                                }
                            }
                            crate::model::common::ActionType::Query => {
                                log::info!("Query action for source {} (mode: {:?})", source_action.source_id, source_action.execution_mode);
                                
                                // Find source and return status information
                                let mut found_source = false;
                                for layout in &app_context_unwrapped.app.layouts {
                                    for muxbox in layout.get_all_muxboxes() {
                                        for (stream_id, stream) in &muxbox.streams {
                                            let stream_source_id = match &stream.stream_type {
                                                StreamType::ChoiceExecution(id) => Some(id.clone()),
                                                StreamType::PtySession(id) => {
                                                    if id.starts_with("PTY-") {
                                                        Some(id[4..].to_string())
                                                    } else {
                                                        Some(id.clone())
                                                    }
                                                },
                                                _ => None,
                                            };
                                            
                                            if let Some(stream_src_id) = stream_source_id {
                                                if stream_src_id == source_action.source_id {
                                                    log::info!("Source {} status: stream_id={}, active={}", 
                                                              source_action.source_id, 
                                                              stream_id, 
                                                              stream.source.is_some());
                                                    found_source = true;
                                                    break;
                                                }
                                            }
                                        }
                                        if found_source { break; }
                                    }
                                    if found_source { break; }
                                }
                                
                                if !found_source {
                                    log::info!("Source {} not found for query", source_action.source_id);
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
                    // F0229: CreateChoiceExecutionStream message - unified stream creation for all execution paths
                    Message::CreateChoiceExecutionStream(stream_id, target_muxbox_id, execution_mode, _stream_label) => {
                        // Only create streams for non-PTY modes - PTY creates its own via MuxBoxOutputUpdate
                        if *execution_mode != ExecutionMode::Pty {
                            let choice_id = if let Some(last_underscore) = stream_id.rfind('_') {
                                &stream_id[..last_underscore]
                            } else {
                                &stream_id
                            };
                            
                            create_execution_stream_only(
                                inner,
                                &mut app_context_unwrapped,
                                &target_muxbox_id,
                                &stream_id,
                                choice_id,
                                execution_mode.clone(),
                            );
                        }
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
                                if let Some(choices) = found_muxbox.get_active_stream_choices_mut()
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
                                if let Some(choices) = found_muxbox.get_active_stream_choices_mut()
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
                    Message::MuxBoxOutputUpdate(stream_id, success, output) => {
                        log::info!("F0229: RECEIVED MuxBoxOutputUpdate for stream: {}, success: {}, output_len: {}, preview: {}", 
                                   stream_id, success, output.len(), output.chars().take(50).collect::<String>());
                        
                        // Extract choice_id from stream_id (format: choice_id_suffix)
                        let choice_id = if let Some(last_underscore) = stream_id.rfind('_') {
                            &stream_id[..last_underscore]
                        } else {
                            &stream_id
                        };
                        
                        // Find target muxbox by searching all muxboxes for this stream
                        let mut target_muxbox_id = None;
                        for layout in &app_context_unwrapped.app.layouts {
                            if let Some(children) = &layout.children {
                                for muxbox in children {
                                    if muxbox.streams.contains_key(stream_id) {
                                        target_muxbox_id = Some(muxbox.id.clone());
                                        break;
                                    }
                                }
                            }
                            if target_muxbox_id.is_some() { break; }
                        }
                        
                        if let Some(muxbox_id) = target_muxbox_id {
                            if let Some(target_muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(&muxbox_id) {
                                // Create PTY stream if not exists (should be created by ThreadManager)
                                if !target_muxbox.streams.contains_key(stream_id) {
                                    let stream_label = format!("{} (pty)", choice_id);
                                    let stream = crate::model::common::Stream::new(
                                        stream_id.clone(),
                                        crate::model::common::StreamType::PtySession(choice_id.to_string()),
                                        stream_label,
                                        Vec::new(),
                                        None,
                                        Some(crate::model::common::StreamSource::create_pty_session_execution_source(
                                            choice_id.to_string(),
                                            muxbox_id.clone(),
                                            "sh".to_string(),
                                            Vec::new(),
                                            None,
                                            (80, 24)
                                        ))
                                    );
                                    target_muxbox.streams.insert(stream_id.clone(), stream);
                                }
                            
                                // Update stream content only
                                if let Some(stream) = target_muxbox.streams.get_mut(stream_id) {
                                    let formatted_output = if *success {
                                        output.clone()
                                    } else {
                                        format!("ERROR: {}", output)
                                    };
                                    
                                    // PTY output - append lines
                                    stream.content.push(formatted_output);
                                    stream.active = true;
                                    
                                    log::info!("F0229: Updated stream {} with {} chars of content",
                                              stream_id, output.len());
                                }
                            }
                        } else {
                            log::warn!("F0229: Could not find target muxbox for stream {}", stream_id);
                        }
                        
                        inner.update_app_context(app_context_unwrapped.clone());
                    }
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
                            let active_layout = app_context_unwrapped.app.get_active_layout().unwrap();
                            let libs = app_context_unwrapped.app.libs.clone();

                            // Find the choice by ID in any muxbox
                            log::info!("Searching for choice {} in active layout", choice_id);
                            if let Some(choice_muxbox) = active_layout.find_muxbox_with_choice(&choice_id) {
                                log::info!("Found choice in muxbox: {}", choice_muxbox.id);

                                if let Some(choices) = choice_muxbox.get_active_stream_choices() {
                                    if let Some(choice) = choices.iter().find(|c| c.id == *choice_id) {
                                        log::info!("Hotkey choice config - execution_mode: {:?}, redirect: {:?}, script_lines: {}", 
                                            choice.execution_mode,
                                            choice.redirect_output,
                                            choice.script.as_ref().map(|s| s.len()).unwrap_or(0)
                                        );

                                        if let Some(script) = &choice.script {
                                            (Some((choice.clone(), script.clone())), choice_muxbox.id.clone(), libs)
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
                            log::info!("T0316: Hotkey creating ExecuteScript for choice {} (mode: {:?})", choice_id, choice.execution_mode);
                            
                            // Create ExecuteScript message instead of direct execution
                            use crate::model::common::{ExecuteScript, ExecutionSource, SourceType, SourceReference};
                            
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
                                                    muxbox_mut.get_active_stream_choices_mut()
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
                                            use crate::model::common::{ExecuteScript, ExecutionSource, SourceType, SourceReference};
                                            
                                            // Create choice object for SourceReference
                                            let choice_for_reference = Choice {
                                                id: choice_id.clone(),
                                                content: Some("".to_string()),
                                                selected: false,
                                                script: Some(script.clone()),
                                                pty: None,
                                                thread: None,
                                                execution_mode: execution_mode.clone(),
                                                redirect_output: redirect_output.clone(),
                                                append_output: Some(append_output),
                                                waiting: true,
                                            };

                                            let execute_script = ExecuteScript {
                                                script: script.clone(),
                                                source: ExecutionSource {
                                                    source_type: SourceType::Choice(choice_id.clone()),
                                                    source_id: format!("choice_{}", choice_id),
                                                    source_reference: SourceReference::Choice(choice_for_reference),
                                                },
                                                execution_mode: execution_mode.clone(),
                                                target_box_id: muxbox_id.clone(),
                                                libs: libs_clone.unwrap_or_default(),
                                                redirect_output: redirect_output.clone(),
                                                append_output,
                                            };

                                            // Send ExecuteScript message instead of calling legacy execute_choice_stream_only
                                            inner.send_message(Message::ExecuteScriptMessage(execute_script));

                                            // Update the app context to persist the waiting state change
                                            inner.update_app_context(
                                                app_context_for_keypress.clone(),
                                            );

                                            log::info!(
                                                "T0314: ExecuteScript message sent for choice {} (unified architecture)",
                                                choice_id
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
                                use crate::model::common::{ExecuteScript, ExecutionSource, SourceType, SourceReference, ExecutionMode};
                                
                                let execute_script = ExecuteScript {
                                    script: actions_unwrapped,
                                    source: ExecutionSource {
                                        source_type: SourceType::SocketUpdate,
                                        source_id: format!("muxbox_keypress_{}_{}", muxbox.id, pressed_key),
                                        source_reference: SourceReference::SocketCommand(format!("muxbox {} keypress: {}", muxbox.id, pressed_key)),
                                    },
                                    execution_mode: ExecutionMode::Immediate, // Muxbox-level handlers use immediate execution
                                    target_box_id: muxbox.redirect_output.as_ref().unwrap_or(&muxbox.id).clone(),
                                    libs: libs.unwrap_or_default(),
                                    redirect_output: muxbox.redirect_output.clone(),
                                    append_output: muxbox.append_output.unwrap_or(false),
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
                                            has_border,
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
                                            has_border,
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
                                            if let Some(stream) = clicked_muxbox.streams.get(stream_id) {
                                                if stream.is_closeable() {
                                                    log::info!(
                                                        "Processing close tab for closeable stream {} in muxbox {}",
                                                        stream_id,
                                                        clicked_muxbox.id
                                                    );
                                                    
                                                    // Extract source_id and execution_mode from stream type
                                                    let (source_id, execution_mode) = match &stream.stream_type {
                                                        StreamType::ChoiceExecution(id) => {
                                                            // Choice execution streams - determine execution mode from stream context
                                                            // For now, default to Thread mode for choice executions
                                                            (id.clone(), crate::model::common::ExecutionMode::Thread)
                                                        }
                                                        StreamType::PtySession(id) => {
                                                            // PTY session streams - extract source_id from "PTY-{source_id}" format
                                                            let actual_source_id = if id.starts_with("PTY-") {
                                                                id[4..].to_string()
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
                                                    inner.send_message(Message::SourceActionMessage(kill_action));
                                                    
                                                    // Also directly remove the stream for immediate UI feedback
                                                    // The SourceAction will handle process termination
                                                    let mut app_context_for_close = app_context_unwrapped.clone();
                                                    if let Some(muxbox) = app_context_for_close.app.get_muxbox_by_id_mut(&clicked_muxbox.id) {
                                                        if muxbox.streams.contains_key(stream_id) {
                                                            let _removed_source = muxbox.remove_stream(stream_id);
                                                            log::info!("Stream {} removed from UI (source termination handled by SourceAction)", stream_id);
                                                            
                                                            // Update app context and trigger redraw for immediate UI response
                                                            inner.update_app_context(app_context_for_close.clone());
                                                            inner.send_message(Message::RedrawAppDiff);
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
                                            has_border,
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

                                    // Check if muxbox has choices (menu items)
                                    if let Some(choices) =
                                        clicked_muxbox.get_active_stream_choices()
                                    {
                                        // Calculate which choice was clicked based on y and x offset within muxbox
                                        if let Some(clicked_choice_idx) =
                                            calculate_clicked_choice_index(
                                                clicked_muxbox,
                                                *x,
                                                *y,
                                                choices,
                                            )
                                        {
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
                                                inner.update_app_context(
                                                    app_context_for_click.clone(),
                                                );
                                                inner.send_message(Message::RedrawAppDiff);

                                                // Then activate the clicked choice (same as pressing Enter)
                                                // F0224: Use ExecutionMode to determine execution path for mouse clicks too
                                                if let Some(script) = &clicked_choice.script {
                                                    let libs =
                                                        app_context_unwrapped.app.libs.clone();

                                                    let script_clone = script.clone();
                                                    let choice_id_clone = clicked_choice.id.clone();
                                                    let muxbox_id_clone = clicked_muxbox.id.clone();
                                                    let libs_clone = libs.clone();
                                                    let execution_mode = clicked_choice.execution_mode.clone();
                                                    let redirect_output = clicked_choice.redirect_output.clone();
                                                    let _append_output = clicked_choice.append_output.unwrap_or(false);

                                                    // T0315: UNIFIED ARCHITECTURE - Replace legacy mouse click execution with ExecuteScript message
                                                    log::info!("T0315: Mouse click creating ExecuteScript for choice {} (mode: {:?})", choice_id_clone, execution_mode);

                                                    // Create ExecuteScript message instead of direct execution or legacy message routing
                                                    use crate::model::common::{ExecuteScript, ExecutionSource, SourceType, SourceReference};
                                                    
                                                    // Create choice object for SourceReference
                                                    let choice_for_reference = Choice {
                                                        id: choice_id_clone.clone(),
                                                        content: Some("".to_string()),
                                                        selected: false,
                                                        script: Some(script_clone.clone()),
                                                        pty: None,
                                                        thread: None,
                                                        execution_mode: execution_mode.clone(),
                                                        redirect_output: redirect_output.clone(),
                                                        append_output: Some(_append_output),
                                                        waiting: true,
                                                    };

                                                    let execute_script = ExecuteScript {
                                                        script: script_clone.clone(),
                                                        source: ExecutionSource {
                                                            source_type: SourceType::Choice(choice_id_clone.clone()),
                                                            source_id: format!("mouse_choice_{}", choice_id_clone),
                                                            source_reference: SourceReference::Choice(choice_for_reference),
                                                        },
                                                        execution_mode: execution_mode.clone(),
                                                        target_box_id: muxbox_id_clone.clone(),
                                                        libs: libs_clone.unwrap_or_default(),
                                                        redirect_output: redirect_output.clone(),
                                                        append_output: _append_output,
                                                    };

                                                    // Send ExecuteScript message instead of legacy execution paths
                                                    inner.send_message(Message::ExecuteScriptMessage(execute_script));

                                                    log::info!(
                                                        "T0315: ExecuteScript message sent for mouse-clicked choice {} (unified architecture)",
                                                        choice_id_clone
                                                    );
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
                                                muxbox.calc_border(
                                                    &app_context_unwrapped.clone(),
                                                    &app_graph,
                                                ),
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
                            "=== F0229: CHOICE EXECUTION COMPLETE: {} on muxbox {} ===",
                            choice_id,
                            muxbox_id
                        );
                        match result {
                            Ok(ref output) => log::info!(
                                "F0229: Choice {} success: {} chars",
                                choice_id, output.len()
                            ),
                            Err(ref error) => {
                                log::error!("F0229: Choice {} error: {}", choice_id, error)
                            }
                        }

                        // F0229: Get execution parameters from the choice
                        let (execution_mode, target_muxbox_id, append_output) = {
                            if let Some(source_muxbox) = app_context_unwrapped.app.get_muxbox_by_id(muxbox_id) {
                                if let Some(choices) = source_muxbox.get_active_stream_choices() {
                                    if let Some(choice) = choices.iter().find(|c| c.id == *choice_id) {
                                        let target = choice.redirect_output.as_ref().unwrap_or(muxbox_id).clone();
                                        (choice.execution_mode.clone(), target, choice.append_output.unwrap_or(false))
                                    } else {
                                        (ExecutionMode::Immediate, muxbox_id.clone(), false)
                                    }
                                } else {
                                    (ExecutionMode::Immediate, muxbox_id.clone(), false)
                                }
                            } else {
                                (ExecutionMode::Immediate, muxbox_id.clone(), false)
                            }
                        };

                        // F0229: Use unified stream update approach
                        let stream_id = format!("{}_{}", choice_id, execution_mode.as_stream_suffix());
                        
                        update_execution_stream_directly(
                            inner,
                            &mut app_context_unwrapped,
                            &target_muxbox_id,
                            &stream_id,
                            choice_id,
                            muxbox_id,
                            result.clone().map_err(|e| e.into()),
                            append_output,
                        );
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
                                    crate::model::common::StreamSource::ImmediateExecution(source) => {
                                        if let Err(e) = source.cleanup() {
                                            log::warn!("Failed to cleanup immediate execution source: {}", e);
                                        }
                                    }
                                    crate::model::common::StreamSource::ThreadPoolExecution(source) => {
                                        if let Err(e) = source.cleanup() {
                                            log::warn!("Failed to cleanup thread pool execution source: {}", e);
                                        }
                                    }
                                    crate::model::common::StreamSource::PtySessionExecution(source) => {
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
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(&muxbox_id)
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
                                        }

                                        // Update app context and trigger redraw
                                        inner.update_app_context(app_context_unwrapped.clone());
                                        inner.send_message(Message::RedrawMuxBox(muxbox_id.clone()));
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
                        
                        // First update the muxbox script field
                        if let Some(muxbox) = app_context_unwrapped.app.get_muxbox_by_id_mut(&muxbox_id) {
                            muxbox.script = Some(new_script.clone());
                            
                            // Create ExecuteScript message for socket-triggered script execution
                            use crate::model::common::{ExecuteScript, ExecutionSource, SourceType, SourceReference};
                            
                            let execute_script = ExecuteScript {
                                script: new_script.clone(),
                                source: ExecutionSource {
                                    source_type: SourceType::SocketUpdate,
                                    source_id: format!("socket_script_{}", muxbox_id),
                                    source_reference: SourceReference::SocketCommand(format!("replace-box-script command for {}", muxbox_id)),
                                },
                                execution_mode: muxbox.execution_mode.clone(),
                                target_box_id: muxbox_id.clone(),
                                libs: libs,
                                redirect_output: muxbox.redirect_output.clone(),
                                append_output: muxbox.append_output.unwrap_or(false),
                            };

                            // Send ExecuteScript message instead of direct execution
                            inner.send_message(Message::ExecuteScriptMessage(execute_script));

                            log::info!(
                                "T0318: ExecuteScript message sent for socket-updated muxbox {} script (unified architecture)",
                                muxbox_id
                            );
                            
                            inner.update_app_context(app_context_unwrapped.clone());
                        } else {
                            log::error!("MuxBox {} not found for script update", muxbox_id);
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

// F0229: Stream-Only Architecture - Unified execution functions that only create/update streams
// Never update content fields directly - eliminates architectural inconsistency

/// Create execution stream only - never update content fields
fn create_execution_stream_only(
    inner: &mut RunnableImpl,
    app_context: &mut AppContext,
    target_muxbox_id: &str,
    stream_id: &str,
    choice_id: &str,
    execution_mode: ExecutionMode,
) {
    log::info!(
        "F0229: Creating execution stream {} for choice {} in muxbox {} (mode: {:?})",
        stream_id, choice_id, target_muxbox_id, execution_mode
    );

    if let Some(target_muxbox) = app_context.app.get_muxbox_by_id_mut(target_muxbox_id) {
        // Create execution stream
        let stream_type = match execution_mode {
            ExecutionMode::Immediate => crate::model::common::StreamType::ChoiceExecution(choice_id.to_string()),
            ExecutionMode::Thread => crate::model::common::StreamType::ChoiceExecution(choice_id.to_string()),
            ExecutionMode::Pty => crate::model::common::StreamType::PtySession(choice_id.to_string()),
        };

        let stream_label = format!("{} ({})", choice_id, execution_mode.as_stream_suffix());
        
        let mut stream = crate::model::common::Stream::new(
            stream_id.to_string(),
            stream_type,
            stream_label,
            vec!["Executing...".to_string()],
            None,
            Some(crate::model::common::StreamSource::from_execution_mode(
                &execution_mode,
                choice_id.to_string(),
                target_muxbox_id.to_string(),
                vec!["execution".to_string()],
                None
            ))
        );
        
        stream.active = true;
        target_muxbox.streams.insert(stream_id.to_string(), stream);
        
        log::info!(
            "F0229: Created execution stream {} in muxbox {}",
            stream_id, target_muxbox_id
        );
    } else {
        log::error!(
            "F0229: Target muxbox {} not found for stream creation",
            target_muxbox_id
        );
    }
}

/// Update execution stream content only - never touch muxbox content fields
fn update_execution_stream(
    inner: &mut RunnableImpl,
    app_context: &mut AppContext,
    target_muxbox_id: &str,
    stream_id: &str,
    execution_result: Result<String, Box<dyn std::error::Error + Send + Sync>>,
    append_output: bool,
) {
    log::info!(
        "F0229: Updating execution stream {} in muxbox {}",
        stream_id, target_muxbox_id
    );

    if let Some(target_muxbox) = app_context.app.get_muxbox_by_id_mut(target_muxbox_id) {
        if let Some(stream) = target_muxbox.streams.get_mut(stream_id) {
            match execution_result {
                Ok(output) => {
                    log::info!("F0229: Stream {} completed successfully ({} chars)", stream_id, output.len());
                    
                    if append_output && !stream.content.is_empty() {
                        stream.content.push(output);
                    } else {
                        stream.content = vec![output];
                    }
                }
                Err(error) => {
                    log::error!("F0229: Stream {} failed: {}", stream_id, error);
                    
                    let error_message = format!("ERROR: {}", error);
                    if append_output && !stream.content.is_empty() {
                        stream.content.push(error_message);
                    } else {
                        stream.content = vec![error_message];
                    }
                }
            }
            
            // Ensure stream remains active after update
            stream.active = true;
        } else {
            log::error!("F0229: Execution stream {} not found in muxbox {}", stream_id, target_muxbox_id);
        }
    } else {
        log::error!("F0229: Target muxbox {} not found for stream update", target_muxbox_id);
    }

    // Update app context and redraw
    inner.update_app_context(app_context.clone());
    inner.send_message(Message::RedrawMuxBox(target_muxbox_id.to_string()));
}

/// Clear choice waiting state helper
fn clear_choice_waiting_state(
    app_context: &mut AppContext,
    source_muxbox_id: &str,
    choice_id: &str,
) {
    if let Some(muxbox_mut) = app_context.app.get_muxbox_by_id_mut(source_muxbox_id) {
        if let Some(choices) = muxbox_mut.get_active_stream_choices_mut() {
            if let Some(choice_mut) = choices.iter_mut().find(|c| c.id == choice_id) {
                choice_mut.waiting = false;
            }
        }
    }
}

/// Update muxbox content routing to specific stream in tab system
// F0229: Stream-Only ExecutionMode System - creates execution streams for all modes  
fn execute_choice_stream_only(
    inner: &mut RunnableImpl,
    app_context: &mut AppContext,
    choice: &Choice,
    source_muxbox_id: &str,
    script: &[String],
    libs: &Option<Vec<String>>,
    execution_mode: ExecutionMode,
) {
    log::info!(
        "F0229: Stream-only execution for choice {} (mode: {:?}, redirect: {:?})",
        choice.id, execution_mode, choice.redirect_output
    );

    // Set choice to waiting state before execution
    if let Some(muxbox_mut) = app_context.app.get_muxbox_by_id_mut(source_muxbox_id) {
        if let Some(choices) = muxbox_mut.get_active_stream_choices_mut() {
            if let Some(choice_mut) = choices.iter_mut().find(|c| c.id == choice.id) {
                choice_mut.waiting = true;
            }
        }
    }

    // F0229: All execution modes now use unified ThreadManager path
    // ThreadManager handles stream creation and execution consistently for all modes
    let target_muxbox_id = choice.redirect_output.as_ref().unwrap_or(&source_muxbox_id.to_string()).clone();
    
    log::info!("F0229: Executing choice {} (mode: {:?}) - delegating to unified ThreadManager", choice.id, execution_mode);
    
    // Use unified ThreadManager execution path to avoid split-brain architecture
    inner.send_message(Message::ExecuteChoice(
        choice.clone(),
        target_muxbox_id.clone(),
        libs.clone(),
    ));

    // Update app context and redraw
    inner.update_app_context(app_context.clone());
}

// F0229: Create or update execution stream without complex message routing
fn create_or_update_execution_stream(
    inner: &mut RunnableImpl,
    app_context: &mut AppContext,
    choice_id: &str,
    target_muxbox_id: &str,
    execution_mode: &ExecutionMode,
    stream_id: &str,
) {
    log::info!(
        "F0229: Creating/updating execution stream {} for choice {} in muxbox {}",
        stream_id, choice_id, target_muxbox_id
    );

    if let Some(target_muxbox) = app_context.app.get_muxbox_by_id_mut(target_muxbox_id) {
        // Initialize streams if needed
        target_muxbox.ensure_tabs_initialized();

        // Check if stream already exists
        if !target_muxbox.streams.contains_key(stream_id) {
            // Create new execution stream
            let (stream_type, stream_source) = create_execution_mode_stream_components(
                choice_id,
                target_muxbox_id,
                execution_mode,
            );

            let stream = crate::model::common::Stream::new(
                stream_id.to_string(),
                stream_type,
                choice_id.to_string(), // Use choice_id as label - clean and simple
                vec!["Executing...".to_string()], // Initial execution status
                None,
                Some(stream_source),
            );

            target_muxbox.streams.insert(stream_id.to_string(), stream);
        }

        // Switch to this stream and redraw
        target_muxbox.switch_to_stream(stream_id);
        inner.send_message(Message::RedrawMuxBox(target_muxbox_id.to_string()));
    } else {
        log::error!("F0229: Target muxbox {} not found for execution stream", target_muxbox_id);
    }
}

// F0229: Direct stream update without message routing complexity
fn update_execution_stream_directly(
    inner: &mut RunnableImpl,
    app_context: &mut AppContext,
    target_muxbox_id: &str,
    stream_id: &str,
    choice_id: &str,
    source_muxbox_id: &str,
    execution_result: Result<String, Box<dyn std::error::Error + Send + Sync>>,
    append_output: bool,
) {
    // Clear waiting state first
    if let Some(muxbox_mut) = app_context.app.get_muxbox_by_id_mut(source_muxbox_id) {
        if let Some(choices) = muxbox_mut.get_active_stream_choices_mut() {
            if let Some(choice_mut) = choices.iter_mut().find(|c| c.id == choice_id) {
                choice_mut.waiting = false;
            }
        }
    }

    // Update the stream directly
    if let Some(target_muxbox) = app_context.app.get_muxbox_by_id_mut(target_muxbox_id) {
        if let Some(stream) = target_muxbox.streams.get_mut(stream_id) {
            match execution_result {
                Ok(output) => {
                    log::info!("F0229: Choice {} completed successfully ({} chars)", choice_id, output.len());
                    
                    // F0229: Clean output without timestamps or formatting complexity
                    if append_output {
                        stream.content.push(output);
                    } else {
                        stream.content = vec![output];
                    }
                }
                Err(error) => {
                    log::error!("F0229: Choice {} failed: {}", choice_id, error);
                    
                    let error_message = format!("ERROR: {}", error);
                    if append_output {
                        stream.content.push(error_message);
                    } else {
                        stream.content = vec![error_message];
                    }
                }
            }

            // Ensure the stream is active
            stream.active = true;
        } else {
            log::error!("F0229: Execution stream {} not found in muxbox {}", stream_id, target_muxbox_id);
        }
    }

    // Update app context and redraw
    inner.update_app_context(app_context.clone());
    inner.send_message(Message::RedrawMuxBox(target_muxbox_id.to_string()));
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

            // Check if we need to activate this stream (separate borrow)
            let has_active_streams = found_muxbox
                .streams
                .values()
                .any(|s| s.active && s.id != stream_id);
            let should_activate = !has_active_streams || found_muxbox.streams.len() == 1;

            if should_activate {
                // Deactivate all other streams first
                for (_, other_stream) in found_muxbox.streams.iter_mut() {
                    other_stream.active = other_stream.id == stream_id;
                }
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
        let stream_content = muxbox.get_active_stream_content();
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
                if let Some(choices) = muxbox.get_active_stream_choices() {
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
                if let Some(choices) = muxbox.get_active_stream_choices() {
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

// F0223: ExecutionMode Stream Integration helper functions

/// F0227: Create execution-mode-specific stream components using specialized source traits
fn create_execution_mode_stream_components(
    choice_id: &str,
    target_muxbox_id: &str,
    execution_mode: &ExecutionMode,
) -> (StreamType, StreamSource) {
    match execution_mode {
        ExecutionMode::Immediate => {
            // F0227: Immediate execution - use ImmediateExecutionSource for specialized lifecycle management
            (
                StreamType::ChoiceExecution(choice_id.to_string()),
                StreamSource::create_immediate_execution_source(
                    choice_id.to_string(),
                    target_muxbox_id.to_string(),
                    vec![], // Script will be provided during execution
                ),
            )
        }
        ExecutionMode::Thread => {
            // F0227: Thread execution - use ThreadPoolExecutionSource for background execution
            (
                StreamType::ChoiceExecution(choice_id.to_string()),
                StreamSource::create_thread_pool_execution_source(
                    choice_id.to_string(),
                    target_muxbox_id.to_string(),
                    vec![], // Script will be provided during execution
                    Some(uuid::Uuid::new_v4().to_string()), // Generate unique thread ID
                    None, // No timeout by default - can be set later
                ),
            )
        }
        ExecutionMode::Pty => {
            // F0227: PTY execution - use PtySessionExecutionSource for real-time process execution
            (
                StreamType::PtySession(choice_id.to_string()),
                StreamSource::create_pty_session_execution_source(
                    choice_id.to_string(),
                    target_muxbox_id.to_string(),
                    "sh".to_string(), // Default command - will be updated during execution
                    vec![], // Args will be provided during execution
                    None, // Working directory will be set during execution
                    (24, 80), // Default terminal size - can be resized later
                ),
            )
        }
    }
}

