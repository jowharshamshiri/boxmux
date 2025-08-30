use crate::model::app::AppContext;
// T0325: ExecuteChoice cleanup - keeping Choice import for ChoiceScriptRunner
use crate::{FieldUpdate, MuxBox, Updatable};
use bincode;
use log::error;
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::mpsc::{self, Sender};
use std::thread;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Exit,
    Terminate,
    Pause,
    Start,
    NextMuxBox(),
    PreviousMuxBox(),
    ScrollMuxBoxDown(),
    ScrollMuxBoxUp(),
    ScrollMuxBoxLeft(),
    ScrollMuxBoxRight(),
    ScrollMuxBoxPageUp(),
    ScrollMuxBoxPageDown(),
    ScrollMuxBoxPageLeft(),
    ScrollMuxBoxPageRight(),
    ScrollMuxBoxToBeginning(), // Home key - scroll to beginning horizontally
    ScrollMuxBoxToEnd(),       // End key - scroll to end horizontally
    ScrollMuxBoxToTop(),       // Ctrl+Home - scroll to top vertically
    ScrollMuxBoxToBottom(),    // Ctrl+End - scroll to bottom vertically
    CopyFocusedMuxBoxContent(),
    Resize,
    RedrawMuxBox(String),
    RedrawApp,
    RedrawAppDiff, // Redraw entire app using diff-based rendering (no screen clear)
    MuxBoxEventRefresh(String),
    MuxBoxScriptUpdate(String, Vec<String>),
    ReplaceMuxBox(String, MuxBox),
    StopBoxRefresh(String),
    StartBoxRefresh(String),
    SwitchActiveLayout(String),
    KeyPress(String),
    ExecuteHotKeyChoice(String),
    MouseClick(u16, u16),                               // x, y coordinates
    MouseDragStart(u16, u16),                           // x, y coordinates - start drag
    MouseDrag(u16, u16),                                // x, y coordinates - continue drag
    MouseDragEnd(u16, u16),                             // x, y coordinates - end drag
    MuxBoxBorderDrag(String, u16, u16), // muxbox_id, x, y coordinates - resize muxbox
    MuxBoxResizeComplete(String),       // muxbox_id - save changes to YAML
    MuxBoxMove(String, u16, u16),       // muxbox_id, x, y coordinates - move muxbox
    MuxBoxMoveComplete(String),         // muxbox_id - save position changes to YAML
    SaveYamlState,                      // F0200: Trigger complete YAML state persistence
    SaveActiveLayout(String),           // F0200: Save active layout to YAML
    SaveMuxBoxContent(String, String),  // F0200: Save muxbox content to YAML
    SaveMuxBoxScroll(String, usize, usize), // F0200: Save muxbox scroll position
    PTYInput(String, String),           // muxbox_id, input_text
    ExternalMessage(String),
    AddBox(String, MuxBox),
    RemoveBox(String),
    // F0203: Multi-Stream Input Tabs messages
    SwitchTab(String, usize),       // muxbox_id, tab_index
    ScrollTabsLeft(String),         // muxbox_id - scroll tabs left
    ScrollTabsRight(String),        // muxbox_id - scroll tabs right
    SwitchToStream(String, String), // muxbox_id, stream_id
    AddStream(String, crate::model::common::StreamSource), // muxbox_id, stream
    RemoveStream(String, String),   // muxbox_id, stream_id
    CloseTab(String, String), // muxbox_id, stream_id - F0219: Close button for redirected tabs
    UpdateStreamContent(String, String, String), // muxbox_id, stream_id, content
    // T0309: UNIFIED EXECUTION ARCHITECTURE - New message types for unified execution system
    ExecuteScriptMessage(crate::model::common::ExecuteScript), // Universal script execution entry point
    StreamUpdateMessage(crate::model::common::StreamUpdate), // Universal stream content updates
    SourceActionMessage(crate::model::common::SourceAction), // Source lifecycle management
}

impl Hash for Message {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Message::Exit => "exit".hash(state),
            Message::Terminate => "terminate".hash(state),
            Message::NextMuxBox() => "next_muxbox".hash(state),
            Message::PreviousMuxBox() => "previous_muxbox".hash(state),
            Message::Resize => "resize".hash(state),
            Message::RedrawMuxBox(muxbox_id) => {
                "redraw_muxbox".hash(state);
                muxbox_id.hash(state);
            }
            Message::RedrawApp => "redraw_app".hash(state),
            Message::RedrawAppDiff => "redraw_app_diff".hash(state),
            Message::SwitchActiveLayout(layout_id) => {
                "switch_active_layout".hash(state);
                layout_id.hash(state);
            }
            Message::MuxBoxEventRefresh(muxbox_id) => {
                "muxbox_event_refresh".hash(state);
                muxbox_id.hash(state);
            }
            Message::ScrollMuxBoxDown() => "scroll_muxbox_down".hash(state),
            Message::ScrollMuxBoxUp() => "scroll_muxbox_up".hash(state),
            Message::ScrollMuxBoxLeft() => "scroll_muxbox_left".hash(state),
            Message::ScrollMuxBoxRight() => "scroll_muxbox_right".hash(state),
            Message::ScrollMuxBoxPageUp() => "scroll_muxbox_page_up".hash(state),
            Message::ScrollMuxBoxPageDown() => "scroll_muxbox_page_down".hash(state),
            Message::ScrollMuxBoxPageLeft() => "scroll_muxbox_page_left".hash(state),
            Message::ScrollMuxBoxPageRight() => "scroll_muxbox_page_right".hash(state),
            Message::ScrollMuxBoxToBeginning() => "scroll_muxbox_to_beginning".hash(state),
            Message::ScrollMuxBoxToEnd() => "scroll_muxbox_to_end".hash(state),
            Message::ScrollMuxBoxToTop() => "scroll_muxbox_to_top".hash(state),
            Message::ScrollMuxBoxToBottom() => "scroll_muxbox_to_bottom".hash(state),
            Message::CopyFocusedMuxBoxContent() => "copy_focused_muxbox_content".hash(state),
            Message::MuxBoxScriptUpdate(muxbox_id, script) => {
                "muxbox_script_update".hash(state);
                muxbox_id.hash(state);
                script.hash(state);
            }
            Message::ReplaceMuxBox(muxbox_id, muxbox) => {
                "replace_muxbox".hash(state);
                muxbox_id.hash(state);
                muxbox.hash(state);
            }
            Message::KeyPress(pressed_key) => {
                "key_press".hash(state);
                pressed_key.hash(state);
            }
            Message::ExecuteHotKeyChoice(choice_id) => {
                "execute_hot_key_choice".hash(state);
                choice_id.hash(state);
            }
            Message::MouseClick(x, y) => {
                "mouse_click".hash(state);
                x.hash(state);
                y.hash(state);
            }
            Message::MouseDragStart(x, y) => {
                "mouse_drag_start".hash(state);
                x.hash(state);
                y.hash(state);
            }
            Message::MouseDrag(x, y) => {
                "mouse_drag".hash(state);
                x.hash(state);
                y.hash(state);
            }
            Message::MouseDragEnd(x, y) => {
                "mouse_drag_end".hash(state);
                x.hash(state);
                y.hash(state);
            }
            Message::MuxBoxBorderDrag(muxbox_id, x, y) => {
                "muxbox_border_drag".hash(state);
                muxbox_id.hash(state);
                x.hash(state);
                y.hash(state);
            }
            Message::MuxBoxResizeComplete(muxbox_id) => {
                "muxbox_resize_complete".hash(state);
                muxbox_id.hash(state);
            }
            Message::MuxBoxMove(muxbox_id, x, y) => {
                "muxbox_move".hash(state);
                muxbox_id.hash(state);
                x.hash(state);
                y.hash(state);
            }
            Message::MuxBoxMoveComplete(muxbox_id) => {
                "muxbox_move_complete".hash(state);
                muxbox_id.hash(state);
            }
            Message::SaveYamlState => "save_yaml_state".hash(state),
            Message::SaveActiveLayout(layout_id) => {
                "save_active_layout".hash(state);
                layout_id.hash(state);
            }
            Message::SaveMuxBoxContent(muxbox_id, content) => {
                "save_muxbox_content".hash(state);
                muxbox_id.hash(state);
                content.hash(state);
            }
            Message::SaveMuxBoxScroll(muxbox_id, x, y) => {
                "save_muxbox_scroll".hash(state);
                muxbox_id.hash(state);
                x.hash(state);
                y.hash(state);
            }
            Message::PTYInput(muxbox_id, input) => {
                "pty_input".hash(state);
                muxbox_id.hash(state);
                input.hash(state);
            }
            Message::Pause => "pause".hash(state),
            Message::ExternalMessage(msg) => {
                "external_message".hash(state);
                msg.hash(state);
            }
            Message::Start => "start".hash(state),
            Message::StopBoxRefresh(box_id) => {
                "stop_box_refresh".hash(state);
                box_id.hash(state);
            }
            Message::StartBoxRefresh(box_id) => {
                "start_box_refresh".hash(state);
                box_id.hash(state);
            }
            Message::AddBox(box_id, muxbox) => {
                "add_box".hash(state);
                box_id.hash(state);
                muxbox.hash(state);
            }
            Message::RemoveBox(box_id) => {
                "remove_box".hash(state);
                box_id.hash(state);
            }
            // F0203: Multi-Stream Input Tabs hash implementations
            Message::SwitchTab(muxbox_id, tab_index) => {
                "switch_tab".hash(state);
                muxbox_id.hash(state);
                tab_index.hash(state);
            }
            Message::ScrollTabsLeft(muxbox_id) => {
                "scroll_tabs_left".hash(state);
                muxbox_id.hash(state);
            }
            Message::ScrollTabsRight(muxbox_id) => {
                "scroll_tabs_right".hash(state);
                muxbox_id.hash(state);
            }
            Message::SwitchToStream(muxbox_id, stream_id) => {
                "switch_to_stream".hash(state);
                muxbox_id.hash(state);
                stream_id.hash(state);
            }
            Message::AddStream(muxbox_id, stream) => {
                "add_stream".hash(state);
                muxbox_id.hash(state);
                stream.hash(state);
            }
            Message::RemoveStream(muxbox_id, stream_id) => {
                "remove_stream".hash(state);
                muxbox_id.hash(state);
                stream_id.hash(state);
            }
            Message::CloseTab(muxbox_id, stream_id) => {
                "close_tab".hash(state);
                muxbox_id.hash(state);
                stream_id.hash(state);
            }
            Message::UpdateStreamContent(muxbox_id, stream_id, content) => {
                "update_stream_content".hash(state);
                muxbox_id.hash(state);
                stream_id.hash(state);
                content.hash(state);
            }
            // T0309: UNIFIED EXECUTION ARCHITECTURE - Hash implementations for new message types
            Message::ExecuteScriptMessage(execute_script) => {
                "execute_script_message".hash(state);
                execute_script.hash(state);
            }
            Message::StreamUpdateMessage(stream_update) => {
                "stream_update_message".hash(state);
                stream_update.hash(state);
            }
            Message::SourceActionMessage(source_action) => {
                "source_action_message".hash(state);
                source_action.hash(state);
            }
        }
    }
}

pub trait Runnable: Send + 'static {
    fn run(&mut self) -> Result<bool, Box<dyn std::error::Error>>;
    fn receive_updates(&mut self) -> (AppContext, Vec<Message>);
    fn process(&mut self, app_context: AppContext, messages: Vec<Message>);

    fn update_app_context(&mut self, app_context: AppContext);
    fn set_uuid(&mut self, uuid: Uuid);
    fn get_uuid(&self) -> Uuid;
    fn set_app_context_sender(
        &mut self,
        app_context_sender: mpsc::Sender<(Uuid, Vec<FieldUpdate>)>,
    );
    fn set_message_sender(&mut self, message_sender: mpsc::Sender<(Uuid, Message)>);
    fn set_app_context_receiver(
        &mut self,
        app_context_receiver: mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>,
    );
    fn set_message_receiver(&mut self, message_receiver: mpsc::Receiver<(Uuid, Message)>);
    fn get_app_context(&self) -> &AppContext;
    fn get_app_context_sender(&self) -> &Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>>;
    fn get_message_sender(&self) -> &Option<mpsc::Sender<(Uuid, Message)>>;

    fn send_app_context_update(&self, old_app_context: AppContext);
    fn send_message(&self, msg: Message);
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunnableState {
    Created,
    Running,
    Paused,
    Terminated,
}

pub struct RunnableImpl {
    pub app_context: AppContext,
    uuid: Uuid,
    running_state: RunnableState,
    app_context_sender: Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>>,
    message_sender: Option<mpsc::Sender<(Uuid, Message)>>,
    app_context_receiver: Option<mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>>,
    message_receiver: Option<mpsc::Receiver<(Uuid, Message)>>,
}

impl RunnableImpl {
    pub fn new(app_context: AppContext) -> Self {
        RunnableImpl {
            app_context,
            uuid: Uuid::new_v4(),
            running_state: RunnableState::Created,
            app_context_sender: None,
            message_sender: None,
            app_context_receiver: None,
            message_receiver: None,
        }
    }

    pub fn get_message_sender(&self) -> Option<&mpsc::Sender<(Uuid, Message)>> {
        self.message_sender.as_ref()
    }

    pub fn get_message_sender_option_ref(&self) -> &Option<mpsc::Sender<(Uuid, Message)>> {
        &self.message_sender
    }

    pub fn _run(
        &mut self,
        process_fn: &mut dyn FnMut(&mut Self, AppContext, Vec<Message>) -> (bool, AppContext),
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let (updated_app_context, new_messages) = self.receive_updates();
        let original_app_context = updated_app_context.clone();
        let mut should_continue = true;
        for message in new_messages.iter() {
            match message {
                Message::Exit => {
                    self.running_state = RunnableState::Terminated;
                    return Ok(false);
                }
                Message::Terminate => {
                    self.running_state = RunnableState::Terminated;
                    return Ok(false);
                }
                Message::Pause => {
                    self.running_state = RunnableState::Paused;
                }
                Message::Start => {
                    self.running_state = RunnableState::Running;
                }
                _ => {}
            }
        }

        //keep app_context in sync even if not running
        if updated_app_context != self.app_context {
            self.app_context = updated_app_context;
        }

        log::trace!(
            "RunnableImpl _run: state={:?}, messages={}",
            self.running_state,
            new_messages.len()
        );
        if self.running_state == RunnableState::Running {
            log::trace!(
                "RunnableImpl calling process function with {} messages",
                new_messages.len()
            );
            let (process_should_continue, result_app_context) =
                process_fn(self, self.app_context.clone(), new_messages);
            if result_app_context != original_app_context {
                self.app_context = result_app_context;
                self.send_app_context_update(original_app_context);
            }
            should_continue = process_should_continue;
            log::trace!(
                "RunnableImpl process function returned should_continue={}",
                should_continue
            );
        } else {
            log::debug!(
                "RunnableImpl NOT calling process function - state is {:?}",
                self.running_state
            );
        }

        if !should_continue {
            return Ok(false);
        }
        Ok(true)
    }
}

impl Runnable for RunnableImpl {
    fn receive_updates(&mut self) -> (AppContext, Vec<Message>) {
        let mut app_context_updates = Vec::new();
        let mut new_messages = Vec::new();

        if let Some(ref app_context_receiver) = self.app_context_receiver {
            while let Ok((_, received_field_updates)) = app_context_receiver.try_recv() {
                if !received_field_updates.is_empty() {
                    log::trace!(
                        "Received app_context update: {:?} in thread {}",
                        received_field_updates,
                        self.uuid
                    );
                    app_context_updates = received_field_updates;
                }
            }
        }

        let mut updated_app_context = self.app_context.clone();
        updated_app_context.apply_updates(app_context_updates);

        if let Some(ref message_receiver) = self.message_receiver {
            while let Ok((_, message)) = message_receiver.try_recv() {
                new_messages.push(message);
            }
        }

        (updated_app_context, new_messages)
    }

    fn process(&mut self, app_context: AppContext, messages: Vec<Message>) {
        // Default implementation: update app context and handle basic messages
        self.update_app_context(app_context);

        // Process any messages that need handling
        for message in messages {
            match message {
                Message::Terminate => {
                    self.running_state = RunnableState::Terminated;
                }
                Message::Pause => {
                    self.running_state = RunnableState::Paused;
                }
                Message::Start => {
                    self.running_state = RunnableState::Running;
                }
                Message::ExecuteScriptMessage(execute_script) => {
                    log::info!("ThreadManager processing ExecuteScript for target_box_id: {}, execution_mode: {:?}", 
                               execute_script.target_box_id, execute_script.execution_mode);
                    
                    self.handle_execute_script(execute_script);
                }
                _ => {
                    // Other messages are handled by specific implementations
                }
            }
        }
    }

    fn update_app_context(&mut self, app_context: AppContext) {
        let old_app_context = self.app_context.clone();
        self.app_context = app_context;
        self.send_app_context_update(old_app_context);
    }

    fn set_uuid(&mut self, uuid: Uuid) {
        self.uuid = uuid;
    }

    fn get_uuid(&self) -> Uuid {
        self.uuid
    }

    fn set_app_context_sender(&mut self, app_context_sender: Sender<(Uuid, Vec<FieldUpdate>)>) {
        self.app_context_sender = Some(app_context_sender);
    }

    fn set_message_sender(&mut self, message_sender: mpsc::Sender<(Uuid, Message)>) {
        self.message_sender = Some(message_sender);
    }

    fn set_app_context_receiver(
        &mut self,
        app_context_receiver: mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>,
    ) {
        self.app_context_receiver = Some(app_context_receiver);
    }

    fn set_message_receiver(&mut self, message_receiver: mpsc::Receiver<(Uuid, Message)>) {
        self.message_receiver = Some(message_receiver);
    }

    fn get_app_context(&self) -> &AppContext {
        &self.app_context
    }

    fn get_app_context_sender(&self) -> &Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>> {
        &self.app_context_sender
    }

    fn get_message_sender(&self) -> &Option<mpsc::Sender<(Uuid, Message)>> {
        &self.message_sender
    }

    fn send_app_context_update(&self, old_app_context: AppContext) {
        if let Some(ref app_context_sender) = self.get_app_context_sender() {
            if let Err(e) = app_context_sender.send((
                self.get_uuid(),
                self.get_app_context().generate_diff(&old_app_context),
            )) {
                error!("Failed to send update to main thread: {}", e);
            }
        }
    }

    fn send_message(&self, msg: Message) {
        if let Some(ref message_sender) = self.get_message_sender() {
            if let Err(e) = message_sender.send((self.get_uuid(), msg)) {
                error!("Failed to send message to main thread: {}", e);
            }
        }
    }


    fn run(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        // This Runnable implementation doesn't need a run loop - it's handled by ThreadManager
        // Return false to indicate no continuous processing needed
        Ok(false)
    }
}

impl RunnableImpl {
    fn handle_execute_script(&mut self, execute_script: crate::model::common::ExecuteScript) {
        use crate::model::common::ExecutionMode;
        
        log::info!("T0315 FIXED: ThreadManager properly handling ExecuteScript for target_box: {}", 
                   execute_script.target_box_id);
        
        // Use UpdateStreamContent to create/update the output stream
        match execute_script.execution_mode {
            ExecutionMode::Immediate => {
                log::info!("T0315: Immediate execution - running script synchronously");
                self.execute_immediate_script(execute_script);
            }
            ExecutionMode::Thread => {
                log::info!("T0315: Thread execution - dispatching to thread pool");  
                self.execute_threaded_script(execute_script);
            }
            ExecutionMode::Pty => {
                log::error!("T0315: PTY ExecuteScript should never reach RunnableImpl - this indicates a routing problem");
                log::error!("PTY execution should be handled directly in DrawLoop, not sent to ThreadManager");
                // This is an error condition - PTY should never reach here
            }
        }
    }
    
    fn execute_immediate_script(&mut self, execute_script: crate::model::common::ExecuteScript) {
        use std::process::Command;
        
        // Run the script synchronously
        let output = Command::new("sh")
            .arg("-c")
            .arg(execute_script.script.join(" "))
            .output();
            
        let content = match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                }
            }
            Err(e) => {
                format!("Error executing script: {}", e)
            }
        };
        
        // Use stream_id from ExecuteScript (already registered in source registry)
        let stream_id = execute_script.stream_id.clone();
        
        // Send result via StreamUpdate with target_box_id for auto-creation
        // REDIRECT FIX: Use redirect destination if specified
        let target_box_id = if let Some(ref redirect_to) = execute_script.redirect_output {
            log::info!("THREADMANAGER REDIRECT FIX UNKNOWN: Using redirect destination: {} (was {})", redirect_to, execute_script.target_box_id);
            redirect_to.clone()
        } else {
            log::info!("THREADMANAGER REDIRECT FIX UNKNOWN: No redirect, using source box: {}", execute_script.target_box_id);
            execute_script.target_box_id.clone()
        };
        
        let stream_update = crate::model::common::StreamUpdate {
            stream_id: stream_id.clone(),
            target_box_id: target_box_id,
            content_update: content,
            source_state: crate::model::common::SourceState::Batch(
                crate::model::common::BatchSourceState {
                    task_id: stream_id.clone(),
                    queue_wait_time: std::time::Duration::from_millis(0),
                    execution_time: std::time::Duration::from_millis(50), // Immediate scripts are very fast
                    exit_code: Some(0),
                    status: crate::model::common::BatchStatus::Completed,
                }
            ),
            execution_mode: execute_script.execution_mode,
        };
        
        self.send_message(Message::StreamUpdateMessage(stream_update));
    }
    
    fn execute_threaded_script(&mut self, execute_script: crate::model::common::ExecuteScript) {
        use std::process::Command;
        use std::thread;
        
        log::info!("T0315: Thread execution - spawning background thread for script");
        
        // Let PTYManager generate and hold its own stream ID - remove ThreadManager stream ID generation
        let target_box_id = execute_script.target_box_id.clone();
        let script = execute_script.script.clone();
        let execution_mode = execute_script.execution_mode.clone();
        let stream_id = execute_script.stream_id.clone();
        
        // Send initial "started" update using stream_id from ExecuteScript
        let start_update = crate::model::common::StreamUpdate {
            stream_id: stream_id.clone(),
            target_box_id: target_box_id.clone(),
            content_update: format!("Starting thread execution: {}\n", script.join(" ")),
            source_state: crate::model::common::SourceState::Thread(
                crate::model::common::ThreadSourceState {
                    thread_id: "pending".to_string(),
                    execution_time: std::time::Duration::from_millis(0),
                    exit_code: None,
                    status: crate::model::common::ExecutionThreadStatus::Running,
                }
            ),
            execution_mode: execution_mode.clone(),
        };
        
        // TODO: Need to send this message back to DrawLoop somehow
        // For now, just log it
        log::info!("ThreadManager would send StreamUpdate message: {:?}", start_update);
        
        // Create a channel to receive messages from the spawned thread
        let (thread_sender, thread_receiver) = std::sync::mpsc::channel::<crate::model::common::StreamUpdate>();
        
        // Store the receiver so we can poll it later (simplified approach)
        // TODO: This is not ideal - should use a proper async mechanism
        
        // Spawn background thread for actual execution
        thread::spawn(move || {
            let thread_id = format!("{:?}", thread::current().id());
            
            // Execute the script
            let output = Command::new("sh")
                .arg("-c")
                .arg(script.join(" "))
                .output();
                
            let (content, exit_code, status) = match output {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let content = if stderr.is_empty() {
                        stdout.to_string()
                    } else {
                        format!("{}\n{}", stdout, stderr)
                    };
                    let exit_code = output.status.code();
                    let status = if output.status.success() {
                        crate::model::common::ExecutionThreadStatus::Completed
                    } else {
                        crate::model::common::ExecutionThreadStatus::Failed("Script execution failed".to_string())
                    };
                    (content, exit_code, status)
                }
                Err(e) => {
                    (format!("Error executing script: {}", e), Some(1), 
                     crate::model::common::ExecutionThreadStatus::Failed(format!("Execution error: {}", e)))
                }
            };
            
            // Send final result back to ThreadManager
            let final_update = crate::model::common::StreamUpdate {
                stream_id,
                target_box_id,
                content_update: content,
                source_state: crate::model::common::SourceState::Thread(
                    crate::model::common::ThreadSourceState {
                        thread_id,
                        execution_time: std::time::Duration::from_millis(100), // approximate
                        exit_code,
                        status,
                    }
                ),
                execution_mode,
            };
            
            // TODO: Need to send this back to DrawLoop
            log::info!("Thread execution completed, would send StreamUpdate: {:?}", final_update);
        });
    }
    
    fn execute_pty_script(&mut self, execute_script: crate::model::common::ExecuteScript) {
        log::info!("T0315: PTY execution - spawning PTY process with real-time streaming");
        
        // Let PTYManager generate and hold its own stream ID - remove ThreadManager stream ID generation
        // REDIRECT FIX: Use redirect destination if specified
        let target_box_id = if let Some(ref redirect_to) = execute_script.redirect_output {
            log::info!("THREADMANAGER REDIRECT FIX PTY1: Using redirect destination: {} (was {})", redirect_to, execute_script.target_box_id);
            redirect_to.clone()
        } else {
            log::info!("THREADMANAGER REDIRECT FIX PTY1: No redirect, using source box: {}", execute_script.target_box_id);
            execute_script.target_box_id.clone()
        };
        let script = execute_script.script.clone();
        let execution_mode = execute_script.execution_mode.clone();
        
        // Use stream_id from source object (from source registry) - no UUID generation here
        let stream_id = execute_script.stream_id.clone();
        let message_sender = if let Some(sender) = &self.message_sender {
            sender.clone()
        } else {
            log::error!("No message sender available for PTY stream: {}", stream_id);
            return;
        };

        // PTYManager will handle all stream creation and updates with its own UUID
        // Just delegate to PTY manager - no ThreadManager stream ID generation needed
        if let Some(pty_manager) = self.app_context.pty_manager.as_ref() {
            let result = if execute_script.redirect_output.is_some() {
                pty_manager.spawn_pty_script_with_redirect(
                    execute_script.target_box_id.clone(),
                    &execute_script.script,
                    Some(execute_script.libs),
                    message_sender.clone(),
                    uuid::Uuid::new_v4(), // Coordination UUID for thread management
                    execute_script.redirect_output,
                    Some(execute_script.stream_id.clone()), // Pass the stream_id from source registry
                )
            } else {
                pty_manager.spawn_pty_script(
                    execute_script.target_box_id.clone(),
                    &execute_script.script,
                    Some(execute_script.libs),
                    message_sender.clone(),
                    uuid::Uuid::new_v4(), // Coordination UUID for thread management
                    Some(execute_script.stream_id.clone()), // Pass the stream_id from source registry
                )
            };
            
            if let Err(e) = result {
                log::error!("Failed to spawn PTY process: {}", e);
            }
        } else {
            log::error!("PTY manager not available for PTY execution");
        }
    }
}

#[derive(Debug)]
pub struct ThreadManager {
    threads: HashMap<Uuid, thread::JoinHandle<()>>,
    app_context_senders: HashMap<Uuid, mpsc::Sender<(Uuid, Vec<FieldUpdate>)>>,
    app_context_receivers: HashMap<Uuid, mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>>,
    message_senders: HashMap<Uuid, mpsc::Sender<(Uuid, Message)>>,
    message_receivers: HashMap<Uuid, mpsc::Receiver<(Uuid, Message)>>,
    app_context: AppContext,
}

impl ThreadManager {
    pub fn new(app_context: AppContext) -> Self {
        ThreadManager {
            threads: HashMap::new(),
            app_context_senders: HashMap::new(),
            message_senders: HashMap::new(),
            app_context_receivers: HashMap::new(),
            message_receivers: HashMap::new(),
            app_context,
        }
    }

    pub fn stop(&self) {
        self.send_message_to_all_threads((Uuid::new_v4(), Message::Exit));
    }

    pub fn pause(&self) {
        self.send_message_to_all_threads((Uuid::new_v4(), Message::Pause));
    }

    pub fn run(&mut self) {
        for message_sender in self.message_senders.values() {
            if let Err(e) = message_sender.send((Uuid::new_v4(), Message::Start)) {
                error!("Failed to send start message to thread: {}", e);
            }
        }
        let mut should_continue: bool = true;
        while should_continue {
            let mut has_updates = false;

            // Handle app_context updates
            for reciever in self.app_context_receivers.values() {
                if let Ok((uuid, app_context_updates)) = reciever.try_recv() {
                    if app_context_updates.is_empty() {
                        // log::trace!("No updates received from thread {}", uuid);
                        continue;
                    } else {
                        let app_context_updates_size_in_bytes =
                            bincode::serialize(&app_context_updates)
                                .unwrap_or_default()
                                .len();
                        log::trace!(
                            "Received {} updates from thread {} with total size {} bytes. Will relay to all other threads.",
                            app_context_updates.len(),
                            uuid,
                            app_context_updates_size_in_bytes
                        );
                    }

                    let original_app_context = self.app_context.clone();

                    // log::trace!(
                    //     "Sending app_context update to all threads: {:?}",
                    //     app_context_updates
                    // );
                    self.app_context.app.apply_updates(app_context_updates);
                    self.send_app_context_update_to_all_threads((
                        uuid,
                        self.app_context.generate_diff(&original_app_context),
                    ));
                    has_updates = true;
                }
            }

            // Handle messages - collect first to avoid borrow conflicts
            let mut messages_to_process = Vec::new();
            for reciever in self.message_receivers.values() {
                if let Ok((uuid, received_msg)) = reciever.try_recv() {
                    messages_to_process.push((uuid, received_msg));
                }
            }

            // Process collected messages
            for (uuid, received_msg) in messages_to_process {
                // log::info!("Received message from thread {}: {:?}", uuid, received_msg);
                match received_msg {
                    Message::Exit => {
                        self.send_message_to_all_threads((Uuid::new_v4(), Message::Terminate));
                        should_continue = false;
                    }
                    Message::ExecuteScriptMessage(execute_script) => {
                        log::info!("ThreadManager processing ExecuteScript from thread {}: target_box_id={}, execution_mode={:?}", 
                                   uuid, execute_script.target_box_id, execute_script.execution_mode);
                        
                        // Handle ExecuteScript directly in ThreadManager, don't broadcast
                        self.handle_execute_script(execute_script);
                        has_updates = true;
                    }
                    _ => {
                        // For all other messages, broadcast to all threads
                        self.send_message_to_all_threads((uuid, received_msg));
                        has_updates = true;
                    }
                }
            }

            // Sleep only if there were no updates to process
            if !has_updates {
                std::thread::sleep(std::time::Duration::from_millis(
                    self.app_context.config.frame_delay,
                ));
            }
        }
    }

    pub fn spawn_thread<R: Runnable + 'static>(&mut self, mut runnable: R) -> Uuid {
        let uuid = Uuid::new_v4();
        let (s_tm_t_s, s_tm_t_r) = mpsc::channel::<(Uuid, Vec<FieldUpdate>)>();
        let (s_t_tm_s, s_t_tm_r) = mpsc::channel::<(Uuid, Vec<FieldUpdate>)>();
        let (m_tm_t_s, m_tm_t_r) = mpsc::channel::<(Uuid, Message)>();
        let (m_t_tm_s, m_t_tm_r) = mpsc::channel::<(Uuid, Message)>();

        runnable.set_uuid(uuid);
        runnable.set_app_context_sender(s_t_tm_s);
        runnable.set_message_sender(m_t_tm_s);
        runnable.set_app_context_receiver(s_tm_t_r);
        runnable.set_message_receiver(m_tm_t_r);

        self.app_context_senders.insert(uuid, s_tm_t_s);
        self.message_senders.insert(uuid, m_tm_t_s);
        self.app_context_receivers.insert(uuid, s_t_tm_r);
        self.message_receivers.insert(uuid, m_t_tm_r);

        let runnable_class_name = std::any::type_name::<R>();
        let thread_name = format!("{}_{}", runnable_class_name, uuid);

        let handle = thread::Builder::new()
            .name(thread_name)
            .spawn(move || {
                let mut continue_running = true;
                while continue_running {
                    let result = runnable.run();
                    if let Err(e) = result {
                        error!("Runnable encountered an error: {}", e);
                        continue_running = false;
                    } else if let Ok(should_continue) = result {
                        continue_running = should_continue;
                        if !continue_running {
                            log::trace!("Stopping thread as directed by run method");
                        }
                    }
                }
            })
            .unwrap();

        self.threads.insert(uuid, handle);

        log::trace!("Thread spawned: {}", uuid);

        uuid
    }

    pub fn send_app_context_update_to_thread(&self, field_updates: Vec<FieldUpdate>, uuid: Uuid) {
        if let Some(sender) = self.app_context_senders.get(&uuid) {
            if let Err(e) = sender.send((uuid, field_updates)) {
                error!("Failed to send data to thread: {}", e);
            }
        }
    }

    pub fn send_app_context_update_to_all_threads(&self, field_updates: (Uuid, Vec<FieldUpdate>)) {
        for (&uuid, sender) in &self.app_context_senders {
            if uuid != field_updates.0 {
                if let Err(e) = sender.send(field_updates.clone()) {
                    error!("Failed to send update to thread: {}", e);
                }
            } else {
                log::trace!("Skipping sending update to thread: {}", uuid);
            }
        }
    }

    pub fn send_message_to_thread(&self, msg: (Uuid, Message), uuid: Uuid) {
        if let Some(sender) = self.message_senders.get(&uuid) {
            log::debug!("ThreadManager found message sender for thread: {}", uuid);
            if let Err(e) = sender.send(msg) {
                log::error!("Failed to send message to thread {}: {}", uuid, e);
            } else {
                log::debug!(
                    "ThreadManager successfully sent message to thread: {}",
                    uuid
                );
            }
        } else {
            log::error!(
                "ThreadManager could not find message sender for thread: {}",
                uuid
            );
        }
    }

    pub fn send_message_to_all_threads(&self, msg: (Uuid, Message)) {
        for (&uuid, sender) in &self.message_senders {
            if uuid != msg.0 {
                if let Err(e) = sender.send(msg.clone()) {
                    error!("Failed to send message to thread: {}", e);
                }
            }
        }
    }

    pub fn join_threads(&mut self) {
        for handle in self.threads.drain() {
            if let Err(e) = handle.1.join() {
                error!("Failed to join thread: {:?}", e);
            }
        }
    }

    pub fn get_hash<T: Hash>(&self, t: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        t.hash(&mut hasher);
        hasher.finish()
    }

    pub fn remove_thread(&mut self, uuid: Uuid) {
        if let Some(handle) = self.threads.remove(&uuid) {
            if let Err(e) = handle.join() {
                error!("Failed to join thread: {:?}", e);
            }
        }
        let msg = (Uuid::new_v4(), Message::Exit);
        self.send_message_to_thread(msg, uuid);
        self.app_context_senders.remove(&uuid);
        self.message_senders.remove(&uuid);
    }

    fn handle_execute_script(&mut self, execute_script: crate::model::common::ExecuteScript) {
        use crate::model::common::ExecutionMode;
        
        log::info!("T0315 FIXED: ThreadManager properly handling ExecuteScript for target_box: {}", 
                   execute_script.target_box_id);
        
        // Use UpdateStreamContent to create/update the output stream
        match execute_script.execution_mode {
            ExecutionMode::Immediate => {
                log::info!("T0315: Immediate execution - running script synchronously");
                self.execute_immediate_script(execute_script);
            }
            ExecutionMode::Thread => {
                log::info!("T0315: Thread execution - dispatching to thread pool");  
                self.execute_threaded_script(execute_script);
            }
            ExecutionMode::Pty => {
                log::error!("T0315: PTY ExecuteScript should never reach ThreadManager - this indicates a routing problem");
                log::error!("PTY execution should be handled directly in DrawLoop, not sent to ThreadManager");
                // This is an error condition - PTY should never reach here
            }
        }
    }

    fn execute_immediate_script(&mut self, execute_script: crate::model::common::ExecuteScript) {
        use std::process::Command;
        
        // Run the script synchronously
        let output = Command::new("sh")
            .arg("-c")
            .arg(execute_script.script.join(" "))
            .output();
            
        let content = match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                }
            }
            Err(e) => {
                format!("Error executing script: {}", e)
            }
        };
        
        // SOURCE OBJECT ARCHITECTURE: Use stream_id from ExecuteScript (from source object)
        let stream_id = execute_script.stream_id.clone();
        
        // Send result via StreamUpdate with target_box_id for auto-creation
        // REDIRECT FIX: Use redirect destination if specified
        let target_box_id = if let Some(ref redirect_to) = execute_script.redirect_output {
            log::info!("THREADMANAGER REDIRECT FIX IMMEDIATE: Using redirect destination: {} (was {})", redirect_to, execute_script.target_box_id);
            redirect_to.clone()
        } else {
            log::info!("THREADMANAGER REDIRECT FIX IMMEDIATE: No redirect, using source box: {}", execute_script.target_box_id);
            execute_script.target_box_id.clone()
        };
        
        let stream_update = crate::model::common::StreamUpdate {
            stream_id: stream_id,
            target_box_id: target_box_id,
            content_update: content,
            source_state: crate::model::common::SourceState::Batch(
                crate::model::common::BatchSourceState {
                    task_id: "immediate".to_string(),
                    queue_wait_time: std::time::Duration::from_millis(0),
                    execution_time: std::time::Duration::from_millis(50), // Immediate scripts are very fast
                    exit_code: Some(0),
                    status: crate::model::common::BatchStatus::Completed,
                }
            ),
            execution_mode: execute_script.execution_mode,
        };
        
        // Broadcast StreamUpdate to all threads for processing
        self.send_message_to_all_threads((uuid::Uuid::new_v4(), Message::StreamUpdateMessage(stream_update)));
    }
    
    fn execute_threaded_script(&mut self, execute_script: crate::model::common::ExecuteScript) {
        log::info!("T0315: Thread execution - using existing thread pool infrastructure");
        
        // SOURCE OBJECT ARCHITECTURE: Use stream_id from ExecuteScript (from source object)
        let stream_id = execute_script.stream_id.clone();
        
        // Use existing utils::run_script_with_pty_and_redirect for Thread execution
        let libs = if execute_script.libs.is_empty() { 
            None 
        } else { 
            Some(execute_script.libs.clone()) 
        };
        
        // REDIRECT FIX: Use redirect destination if specified
        let target_box_id = if let Some(ref redirect_to) = execute_script.redirect_output {
            log::info!("THREADMANAGER REDIRECT FIX THREAD: Using redirect destination: {} (was {})", redirect_to, execute_script.target_box_id);
            redirect_to.clone()
        } else {
            log::info!("THREADMANAGER REDIRECT FIX THREAD: No redirect, using source box: {}", execute_script.target_box_id);
            execute_script.target_box_id.clone()
        };
        let script = execute_script.script.clone();
        let execution_mode = execute_script.execution_mode.clone();
        let redirect_target = execute_script.redirect_output.clone();
        let message_senders = self.message_senders.clone();
        let thread_manager_uuid = uuid::Uuid::new_v4();
        
        // Spawn thread using existing infrastructure pattern
        std::thread::spawn(move || {
            let result = crate::utils::run_script_with_pty_and_redirect(
                libs,
                &script,
                &execution_mode,
                None, // No PTY manager for Thread mode
                None, // No muxbox_id needed for Thread mode
                None, // No message sender needed - we'll send result directly
                redirect_target,
            );
            
            let (content, is_success) = match result {
                Ok(output) => (output, true),
                Err(e) => (format!("Thread execution error: {}", e), false),
            };
            
            // Send result via StreamUpdate
            let final_update = crate::model::common::StreamUpdate {
                stream_id,
                target_box_id,
                content_update: content,
                source_state: crate::model::common::SourceState::Thread(
                    crate::model::common::ThreadSourceState {
                        thread_id: format!("{:?}", std::thread::current().id()),
                        execution_time: std::time::Duration::from_millis(100), // approximate
                        exit_code: Some(if is_success { 0 } else { 1 }), // Success=0, Error=1
                        status: crate::model::common::ExecutionThreadStatus::Completed,
                    }
                ),
                execution_mode,
            };
            
            // Send to all threads via message senders
            for (uuid, sender) in message_senders.iter() {
                if let Err(e) = sender.send((thread_manager_uuid, Message::StreamUpdateMessage(final_update.clone()))) {
                    log::error!("Failed to send thread execution result to thread {}: {}", uuid, e);
                }
            }
        });
    }
    
    fn execute_pty_script(&mut self, execute_script: crate::model::common::ExecuteScript) {
        log::info!("T0315: PTY execution - delegating to PTY manager with unified architecture");
        
        // Use stream_id from source object (from source registry) - no UUID generation here
        let stream_id = execute_script.stream_id.clone();
        
        // TODO T0401: Implement proper message routing back to DrawLoop
        // For now, PTY execution is delegated directly to PTY manager without return messages
        log::warn!("PTY execution: proper message routing needs implementation in T0401");

        // Temporary stub - PTY execution disabled until T0401 complete
        log::error!("PTY execution disabled - broken message routing needs implementation in T0401");
    }
}

#[macro_export]
macro_rules! create_runnable {
    ($name:ident, $init_body:expr, $process_body:expr) => {
        pub struct $name {
            inner: RunnableImpl,
        }

        impl $name {
            pub fn new(app_context: AppContext) -> Self {
                $name {
                    inner: RunnableImpl::new(app_context),
                }
            }
        }

        impl Runnable for $name {
            fn run(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
                // Call the init block before the loop
                {
                    let inner = &mut self.inner;
                    let app_context = inner.app_context.clone();
                    let messages = Vec::new();
                    let init_result = $init_body(inner, app_context, messages);
                    if !init_result {
                        return Ok(false);
                    }
                }
                self.inner._run(&mut |inner, app_context, messages| {
                    $process_body(inner, app_context, messages)
                })
            }

            fn receive_updates(&mut self) -> (AppContext, Vec<Message>) {
                self.inner.receive_updates()
            }

            fn process(&mut self, app_context: AppContext, messages: Vec<Message>) {
                self.inner.process(app_context, messages)
            }

            fn update_app_context(&mut self, app_context: AppContext) {
                self.inner.update_app_context(app_context)
            }

            fn set_uuid(&mut self, uuid: Uuid) {
                self.inner.set_uuid(uuid)
            }

            fn get_uuid(&self) -> Uuid {
                self.inner.get_uuid()
            }

            fn set_app_context_sender(
                &mut self,
                app_context_sender: mpsc::Sender<(Uuid, Vec<FieldUpdate>)>,
            ) {
                self.inner.set_app_context_sender(app_context_sender)
            }

            fn set_message_sender(&mut self, message_sender: mpsc::Sender<(Uuid, Message)>) {
                self.inner.set_message_sender(message_sender)
            }

            fn set_app_context_receiver(
                &mut self,
                app_context_receiver: mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>,
            ) {
                self.inner.set_app_context_receiver(app_context_receiver)
            }

            fn set_message_receiver(&mut self, message_receiver: mpsc::Receiver<(Uuid, Message)>) {
                self.inner.set_message_receiver(message_receiver)
            }

            fn get_app_context(&self) -> &AppContext {
                self.inner.get_app_context()
            }

            fn get_app_context_sender(&self) -> &Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>> {
                self.inner.get_app_context_sender()
            }

            fn get_message_sender(&self) -> &Option<mpsc::Sender<(Uuid, Message)>> {
                self.inner.get_message_sender_option_ref()
            }

            fn send_app_context_update(&self, old_app_context: AppContext) {
                self.inner.send_app_context_update(old_app_context)
            }

            fn send_message(&self, msg: Message) {
                self.inner.send_message(msg)
            }
        }
    };
}

// T312: ChoiceExecutionRunnable - Convert choice execution to Runnable pattern
create_runnable!(
    ChoiceExecutionRunnable,
    |inner: &mut RunnableImpl, _app_context: AppContext, _messages: Vec<Message>| -> bool {
        // Initialize - no setup needed for choice execution
        log::debug!(
            "ChoiceExecutionRunnable initialization: state={:?}",
            inner.running_state
        );
        inner.running_state = RunnableState::Running;
        log::debug!("ChoiceExecutionRunnable set state to Running");
        true
    },
    |_inner: &mut RunnableImpl,
     app_context: AppContext,
     messages: Vec<Message>|
     -> (bool, AppContext) {
        // Debug: Always log to see if processing function is called
        let message_count = messages.len();
        log::debug!(
            "ChoiceExecutionRunnable processing function called with {} messages",
            message_count
        );
        for message in messages {
            // T0325: ExecuteChoice message removed - Phase 5 cleanup complete
            // All message handling now goes through unified ExecuteScript architecture
            match message {
                // All legacy ExecuteChoice handling removed - use ExecuteScript instead
                _ => {
                    log::debug!("ChoiceExecutionRunnable: Unhandled message type: {:?}", message);
                }
            }
        }

        // T0325: ExecuteChoice message removed - no choice execution logic needed
        // All execution now flows through unified ExecuteScript architecture
        let should_continue = true; // Continue message processing
        log::debug!(
            "ChoiceExecutionRunnable: messages={}, should_continue={}",
            message_count,
            should_continue
        );
        (should_continue, app_context)
    }
);

// ChoiceExecutionRunnable implementation removed - using unified message system

#[macro_export]
macro_rules! create_runnable_with_dynamic_input {
    ($name:ident, $vec_fn:expr, $init_body:expr, $process_body:expr) => {
        pub struct $name {
            inner: RunnableImpl,
            vec_fn: Box<dyn Fn() -> Vec<String> + Send>,
        }

        impl $name {
            pub fn new(
                app_context: AppContext,
                vec_fn: Box<dyn Fn() -> Vec<String> + Send>,
            ) -> Self {
                $name {
                    inner: RunnableImpl::new(app_context),
                    vec_fn,
                }
            }
        }

        impl Runnable for $name {
            fn run(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
                // Call the init block before the loop
                {
                    let inner = &mut self.inner;
                    let app_context = inner.app_context.clone();
                    let messages = Vec::new();
                    let vec = (self.vec_fn)();
                    let init_result = $init_body(inner, app_context, messages, vec);
                    if !init_result {
                        return Ok(false);
                    }
                }
                self.inner._run(&mut |inner, app_context, messages| {
                    let vec = (self.vec_fn)();
                    $process_body(inner, app_context, messages, vec)
                })
            }

            fn receive_updates(&mut self) -> (AppContext, Vec<Message>) {
                self.inner.receive_updates()
            }

            fn process(&mut self, app_context: AppContext, messages: Vec<Message>) {
                self.inner.process(app_context.clone(), messages.clone());
            }

            fn update_app_context(&mut self, app_context: AppContext) {
                self.inner.update_app_context(app_context)
            }

            fn set_uuid(&mut self, uuid: Uuid) {
                self.inner.set_uuid(uuid)
            }

            fn get_uuid(&self) -> Uuid {
                self.inner.get_uuid()
            }

            fn set_app_context_sender(
                &mut self,
                app_context_sender: mpsc::Sender<(Uuid, Vec<FieldUpdate>)>,
            ) {
                self.inner.set_app_context_sender(app_context_sender)
            }

            fn set_message_sender(&mut self, message_sender: mpsc::Sender<(Uuid, Message)>) {
                self.inner.set_message_sender(message_sender)
            }

            fn set_app_context_receiver(
                &mut self,
                app_context_receiver: mpsc::Receiver<(Uuid, Vec<FieldUpdate>)>,
            ) {
                self.inner.set_app_context_receiver(app_context_receiver)
            }

            fn set_message_receiver(&mut self, message_receiver: mpsc::Receiver<(Uuid, Message)>) {
                self.inner.set_message_receiver(message_receiver)
            }

            fn get_app_context(&self) -> &AppContext {
                self.inner.get_app_context()
            }

            fn get_app_context_sender(&self) -> &Option<mpsc::Sender<(Uuid, Vec<FieldUpdate>)>> {
                self.inner.get_app_context_sender()
            }

            fn get_message_sender(&self) -> &Option<mpsc::Sender<(Uuid, Message)>> {
                self.inner.get_message_sender_option_ref()
            }

            fn send_app_context_update(&self, old_app_context: AppContext) {
                self.inner.send_app_context_update(old_app_context)
            }

            fn send_message(&self, msg: Message) {
                self.inner.send_message(msg)
            }
        }
    };
}

pub fn run_script_in_thread(
    app_context: AppContext,
    manager: &mut ThreadManager,
    muxbox_id: String,
    choice_id: String,
) -> Uuid {
    let vec_fn = move || vec![muxbox_id.clone(), choice_id.clone()];

    create_runnable_with_dynamic_input!(
        ChoiceScriptRunner,
        Box::new(vec_fn),
        |_inner: &mut RunnableImpl,
         _app_context: AppContext,
         _messages: Vec<Message>,
         _vec: Vec<String>|
         -> bool { true },
        |inner: &mut RunnableImpl,
         app_context: AppContext,
         _messages: Vec<Message>,
         vec: Vec<String>|
         -> (bool, AppContext) {
            let mut app_context_unwrapped = app_context.clone();
            let app_graph = app_context_unwrapped.app.generate_graph();
            let libs = app_context_unwrapped.app.libs.clone();
            let muxbox = app_context_unwrapped
                .app
                .get_muxbox_by_id_mut(&vec[0])
                .unwrap();
            let choice = muxbox
                .choices
                .as_mut()
                .unwrap()
                .iter_mut()
                .find(|c| c.id == vec[1])
                .unwrap();
            // T0330: Remove legacy thread/pty fields - use ExecutionMode
            let execution_mode = &choice.execution_mode;
            let pty_manager = app_context_unwrapped.pty_manager.as_ref();
            let message_sender = if let Some(sender) = inner.get_message_sender() {
                Some((sender.clone(), inner.get_uuid()))
            } else {
                None
            };

            match crate::utils::run_script_with_pty(
                libs,
                choice.script.clone().unwrap().as_ref(),
                execution_mode, // F0228: Use ExecutionMode directly
                pty_manager.map(|arc| arc.as_ref()),
                Some(choice.id.clone()),
                message_sender,
            ) {
                Ok(output) => {
                    // Create stream_id for proper stream targeting
                    let stream_id = format!("{}_{}", choice.id, execution_mode.as_stream_suffix());
                    // T0328: Replace MuxBoxOutputUpdate with StreamUpdateMessage
                    let stream_update = crate::model::common::StreamUpdate {
                        stream_id,
                        target_box_id: vec[0].clone(),
                        content_update: output,
                        source_state: crate::model::common::SourceState::Thread(
                            crate::model::common::ThreadSourceState {
                                thread_id: format!("{:?}", std::thread::current().id()),
                                execution_time: std::time::Duration::from_millis(0),
                                exit_code: Some(0),
                                status: crate::model::common::ExecutionThreadStatus::Completed,
                            }
                        ),
                        execution_mode: execution_mode.clone(),
                    };
                    inner.send_message(Message::StreamUpdateMessage(stream_update))
                }
                Err(e) => {
                    let stream_id = format!("{}_{}", choice.id, execution_mode.as_stream_suffix());
                    // T0328: Replace MuxBoxOutputUpdate with StreamUpdateMessage
                    let stream_update = crate::model::common::StreamUpdate {
                        stream_id,
                        target_box_id: vec[0].clone(),
                        content_update: e.to_string(),
                        source_state: crate::model::common::SourceState::Thread(
                            crate::model::common::ThreadSourceState {
                                thread_id: format!("{:?}", std::thread::current().id()),
                                execution_time: std::time::Duration::from_millis(0),
                                exit_code: Some(1),
                                status: crate::model::common::ExecutionThreadStatus::Failed(e.to_string()),
                            }
                        ),
                        execution_mode: execution_mode.clone(),
                    };
                    inner.send_message(Message::StreamUpdateMessage(stream_update))
                },
            }
            std::thread::sleep(std::time::Duration::from_millis(
                muxbox.calc_refresh_interval(&app_context, &app_graph),
            ));
            (false, app_context_unwrapped)
        }
    );

    let choice_refresh_loop = ChoiceScriptRunner::new(app_context.clone(), Box::new(vec_fn));
    manager.spawn_thread(choice_refresh_loop)
}

// create_runnable!(
//     ExampleRunnable,z
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool {
//         // Initialization block
//         info!("Initializing ExampleRunnable with app_context: {:?}", app_context);
//         inner.update_app_context(app_context);
//         true // Initialization complete, continue running
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool {
//         // Processing block
//         info!("Processing in ExampleRunnable with app_context: {:?} and messages: {:?}", app_context, messages);

//         for message in messages {
//             match message {
//                 Message::Exit => return false, // Stop running
//                 Message::NextMuxBox(muxbox_id) => {
//                     info!("Next muxbox: {}", muxbox_id);
//                     // Handle NextMuxBox logic
//                 },
//                 Message::PreviousMuxBox(muxbox_id) => {
//                     info!("Previous muxbox: {}", muxbox_id);
//                     // Handle PreviousMuxBox logic
//                 },
//                 _ => {
//                     info!("Unhandled message: {:?}", message);
//                     // Handle other messages
//                 },
//             }
//         }

//         true // Continue running
//     }
// );

// create_runnable!(
//     TestRunnableOne,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne initialization");
//         true  // Assuming initialization is always successful
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableOne received message: {:?}", message);
//         }
//         info!("TestRunnableOne running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::RedrawApp);
//         false  // Intentionally stopping the thread for demonstration
//     }
// );

// create_runnable!(
//     TestRunnableTwo,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne initialization");
//         true  // Assuming initialization is always successful
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableOne received message: {:?}", message);
//         }
//         info!("TestRunnableOne running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::RedrawApp);
//         false  // Intentionally stopping the thread for demonstration
//     }
// );

// create_runnable!(
//     TestRunnableThree,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne initialization");
//         true  // Assuming initialization is always successful
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableOne received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableOne received message: {:?}", message);
//         }
//         info!("TestRunnableOne running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::RedrawApp);
//         false  // Intentionally stopping the thread for demonstration
//     }
// );

// create_runnable!(
//     TestRunnableTwo,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| {
//         info!("TestRunnableTwo initialization");
//         true  // Initialization success
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message)| {
//         info!("TestRunnableTwo received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableTwo received message: {:?}", message);
//         }
//         info!("TestRunnableTwo running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::ReplaceMuxBox("MuxBox2".to_string()));
//         true  // Continue running
//     }
// );

// create_runnable!(
//     TestRunnableThree,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message)| {
//         info!("TestRunnableThree initialization");
//         true  // Initialization success
//     },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message)| {
//         info!("TestRunnableThree received app_context: {:?}", app_context);
//         for message in messages.iter() {
//             info!("TestRunnableThree received message: {:?}", message);
//         }
//         info!("TestRunnableThree running with data: {:?}", inner.get_app_context());
//         inner.send_message(Message::MuxBoxEventEnter("MuxBox3".to_string()));
//         true  // Continue running
//     }
// );

// #[cfg(test)]
// mod tests {
//     use crate::App;

//     use super::*;
//     use std::sync::mpsc::TryRecvError;

//     #[test]
//     fn test_message_delivery() {
//         let app_context = AppContext::new(App::new());
//         let mut manager = ThreadManager::new(app_context.clone());
//         let uuid1 = manager.spawn_thread(TestRunnableOne::new(app_context.clone()));
//         let uuid2 = manager.spawn_thread(TestRunnableTwo::new(app_context.clone()));
//         let uuid3 = manager.spawn_thread(TestRunnableThree::new(app_context.clone()));

//         let data = AppContext::new(App::new());
//         manager.send_app_context_update_to_thread(data.clone(), uuid1);
//         manager.send_app_context_update_to_thread(data.clone(), uuid2);
//         manager.send_app_context_update_to_thread(data.clone(), uuid3);

//         manager.send_message_to_all_threads((uuid1, Message::NextMuxBox("MuxBox1".to_string())));

//         // Run the manager's loop in a separate thread to allow message handling
//         let manager = manager;
//         // let manager_clone = Arc::clone(&manager);

//         let handle = thread::spawn(move || {
//             manager_clone.run();
//         });

//         // Give the threads some time to process the messages
//         thread::sleep(std::time::Duration::from_secs(1));

//         // Ensure that each runnable received the message
//         let runnables = manager.runnables.clone();
//         for (_, runnable) in runnables.iter() {
//             let mut runnable = runnable.lock().unwrap();
//             let (_, messages) = runnable.receive_updates();
//             assert!(messages.iter().any(|msg| matches!(msg, Message::NextMuxBox(muxbox_id) if muxbox_id == "MuxBox1")));
//         }
//         manager.stop();
//         handle.join().unwrap();
//     }

//     #[test]
//     fn test_state_update_propagation() {
//         let app_context = AppContext::new(App::new());
//         let mut manager = ThreadManager::new(app_context.clone());
//         let uuid1 = manager.spawn_thread(TestRunnableOne::new(app_context.clone()));
//         let uuid2 = manager.spawn_thread(TestRunnableTwo::new(app_context.clone()));
//         let uuid3 = manager.spawn_thread(TestRunnableThree::new(app_context.clone()));

//         let data = AppContext::new(App::new());
//         manager.send_app_context_update_to_thread(data.clone(), uuid1);

//         // Run the manager's loop in a separate thread to allow app_context handling
//         let manager = Arc::new(manager);
//         let manager_clone = Arc::clone(&manager);

//         let handle = thread::spawn(move || {
//             manager_clone.run();
//         });

//         // Give the threads some time to process the app_context update
//         thread::sleep(std::time::Duration::from_secs(1));

//         // Ensure that the app_context was propagated to all runnables
//         let runnables = manager.runnables.clone();
//         for (_, runnable) in runnables.iter() {
//             let mut runnable = runnable.lock().unwrap();
//             let (app_context, _) = runnable.receive_updates();
//             assert_eq!(app_context, data);
//         }
//         manager.stop();
//         handle.join().unwrap();
//     }

//     #[test]
//     fn test_concurrent_message_handling() {
//         let app_context = AppContext::new(App::new());
//         let mut manager = ThreadManager::new(app_context.clone());
//         let uuid1 = manager.spawn_thread(TestRunnableOne::new(app_context.clone()));
//         let uuid2 = manager.spawn_thread(TestRunnableTwo::new(app_context.clone()));
//         let uuid3 = manager.spawn_thread(TestRunnableThree::new(app_context.clone()));

//         let data = AppContext::new(App::new());
//         manager.send_app_context_update_to_thread(data.clone(), uuid1);
//         manager.send_app_context_update_to_thread(data.clone(), uuid2);
//         manager.send_app_context_update_to_thread(data.clone(), uuid3);

//         manager.send_message_to_all_threads((uuid1, Message::RedrawApp));
//         manager.send_message_to_all_threads((uuid2, Message::ReplaceMuxBox("MuxBox2".to_string())));
//         manager.send_message_to_all_threads((uuid3, Message::MuxBoxEventEnter("MuxBox3".to_string())));

//         // Run the manager's loop in a separate thread to allow message handling
//         let manager = Arc::new(manager);
//         let manager_clone = Arc::clone(&manager);

//         let handle = thread::spawn(move || {
//             manager_clone.run();
//         });

//         // Give the threads some time to process the messages
//         thread::sleep(std::time::Duration::from_secs(1));

//         // Ensure that each runnable received the messages
//         let runnables = manager.runnables.clone();
//         for (_, runnable) in runnables.iter() {
//             let mut runnable = runnable.lock().unwrap();
//             let (_, messages) = runnable.receive_updates();
//             assert!(messages.iter().any(|msg| matches!(msg, Message::RedrawApp)));
//             assert!(messages.iter().any(|msg| matches!(msg, Message::ReplaceMuxBox(muxbox_id) if muxbox_id == "MuxBox2")));
//             assert!(messages.iter().any(|msg| matches!(msg, Message::MuxBoxEventEnter(muxbox_id) if muxbox_id == "MuxBox3")));
//         }
//         manager.stop();
//         handle.join().unwrap();
//     }

//     #[test]
//     fn test_message_delivery_once() {
//         let app_context = AppContext::new(App::new());
//         let mut manager = ThreadManager::new(app_context.clone());
//         let uuid1 = manager.spawn_thread(TestRunnableOne::new(app_context.clone()));
//         let uuid2 = manager.spawn_thread(TestRunnableTwo::new(app_context.clone()));
//         let uuid3 = manager.spawn_thread(TestRunnableThree::new(app_context.clone()));

//         let data = AppContext::new(App::new());
//         manager.send_app_context_update_to_thread(data.clone(), uuid1);
//         manager.send_app_context_update_to_thread(data.clone(), uuid2);
//         manager.send_app_context_update_to_thread(data.clone(), uuid3);

//         manager.send_message_to_all_threads((uuid1, Message::RedrawApp));

//         // Run the manager's loop in a separate thread to allow message handling
//         let manager = Arc::new(manager);
//         let manager_clone = Arc::clone(&manager);

//         let handle = thread::spawn(move || {
//             manager_clone.run();
//         });

//         // Give the threads some time to process the messages
//         thread::sleep(std::time::Duration::from_secs(1));

//         // Ensure that the message was delivered only once
//         let runnables = manager.runnables.clone();
//         for (_, runnable) in runnables.iter() {
//             let mut runnable = runnable.lock().unwrap();
//             let (_, messages) = runnable.receive_updates();
//             let count = messages.iter().filter(|msg| matches!(msg, Message::RedrawApp)).count();
//             assert_eq!(count, 1);
//         }
//         manager.stop();
//         handle.join().unwrap();
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::app::App;
    use crate::model::common::{Config, EntityType, FieldUpdate};
    use crate::model::layout::Layout;
    use crate::model::muxbox::MuxBox;
    use serde_json::Value;
    use std::sync::mpsc;

    // Helper function to create test AppContext
    fn create_test_app_context() -> AppContext {
        let mut app = App::new();
        let layout = create_test_layout("test_layout");
        app.layouts.push(layout);
        let config = Config::default();
        AppContext::new(app, config)
    }

    // Helper function to create test MuxBox
    fn create_test_muxbox(id: &str) -> MuxBox {
        MuxBox {
            id: id.to_string(),
            position: crate::model::common::InputBounds {
                x1: "0%".to_string(),
                y1: "0%".to_string(),
                x2: "100%".to_string(),
                y2: "100%".to_string(),
            },
            ..Default::default()
        }
    }

    // Helper function to create test Layout
    fn create_test_layout(id: &str) -> Layout {
        Layout {
            id: id.to_string(),
            ..Default::default()
        }
    }

    // Helper function to create test FieldUpdate
    fn create_test_field_update(
        entity_type: EntityType,
        entity_id: &str,
        field_name: &str,
        value: Value,
    ) -> FieldUpdate {
        FieldUpdate {
            entity_type,
            entity_id: Some(entity_id.to_string()),
            field_name: field_name.to_string(),
            new_value: value,
        }
    }

    /// Tests that Message enum implements Hash correctly for different variants.
    /// This test demonstrates the message hashing feature.
    #[test]
    fn test_message_hash() {
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        let msg1 = Message::Exit;
        let msg2 = Message::Exit;
        msg1.hash(&mut hasher1);
        msg2.hash(&mut hasher2);
        assert_eq!(hasher1.finish(), hasher2.finish());

        let msg3 = Message::RedrawMuxBox("muxbox1".to_string());
        let msg4 = Message::RedrawMuxBox("muxbox1".to_string());
        let mut hasher3 = DefaultHasher::new();
        let mut hasher4 = DefaultHasher::new();
        msg3.hash(&mut hasher3);
        msg4.hash(&mut hasher4);
        assert_eq!(hasher3.finish(), hasher4.finish());
    }

    /// Tests that Message enum implements PartialEq correctly.
    /// This test demonstrates the message equality comparison feature.
    #[test]
    fn test_message_equality() {
        assert_eq!(Message::Exit, Message::Exit);
        assert_eq!(Message::Terminate, Message::Terminate);
        assert_eq!(
            Message::RedrawMuxBox("muxbox1".to_string()),
            Message::RedrawMuxBox("muxbox1".to_string())
        );
        assert_ne!(
            Message::RedrawMuxBox("muxbox1".to_string()),
            Message::RedrawMuxBox("muxbox2".to_string())
        );
        assert_ne!(Message::Exit, Message::Terminate);
    }

    /// Tests that Message enum correctly hashes different message types.
    /// This test demonstrates the message type differentiation feature.
    #[test]
    fn test_message_hash_different_types() {
        let msg1 = Message::KeyPress("a".to_string());
        let msg2 = Message::KeyPress("a".to_string());
        let msg3 = Message::KeyPress("b".to_string());

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        msg1.hash(&mut hasher1);
        msg2.hash(&mut hasher2);
        msg3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    /// Tests that RunnableState enum implements Clone and PartialEq.
    /// This test demonstrates the runnable state management feature.
    #[test]
    fn test_runnable_state() {
        let state1 = RunnableState::Created;
        let state2 = state1.clone();
        assert_eq!(state1, state2);

        let state3 = RunnableState::Running;
        assert_ne!(state1, state3);

        let state4 = RunnableState::Paused;
        let state5 = RunnableState::Terminated;
        assert_ne!(state4, state5);
    }

    /// Tests that RunnableImpl can be created with an AppContext.
    /// This test demonstrates the runnable implementation creation feature.
    #[test]
    fn test_runnable_impl_new() {
        let app_context = create_test_app_context();
        let runnable = RunnableImpl::new(app_context.clone());

        assert_eq!(runnable.get_app_context(), &app_context);
        assert_eq!(runnable.running_state, RunnableState::Created);
        assert!(runnable.app_context_sender.is_none());
        assert!(runnable.message_sender.is_none());
        assert!(runnable.app_context_receiver.is_none());
        assert!(runnable.message_receiver.is_none());
    }

    /// Tests that RunnableImpl can set and get UUID.
    /// This test demonstrates the runnable UUID management feature.
    #[test]
    fn test_runnable_impl_uuid() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);

        let original_uuid = runnable.get_uuid();
        let new_uuid = Uuid::new_v4();
        runnable.set_uuid(new_uuid);

        assert_eq!(runnable.get_uuid(), new_uuid);
        assert_ne!(runnable.get_uuid(), original_uuid);
    }

    /// Tests that RunnableImpl can set and get senders and receivers.
    /// This test demonstrates the runnable channel management feature.
    #[test]
    fn test_runnable_impl_channels() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);

        let (app_context_sender, app_context_receiver) = mpsc::channel();
        let (message_sender, message_receiver) = mpsc::channel();

        runnable.set_app_context_sender(app_context_sender);
        runnable.set_message_sender(message_sender);
        runnable.set_app_context_receiver(app_context_receiver);
        runnable.set_message_receiver(message_receiver);

        assert!(runnable.get_app_context_sender().is_some());
        assert!(runnable.get_message_sender().is_some());
        assert!(runnable.app_context_receiver.is_some());
        assert!(runnable.message_receiver.is_some());
    }

    /// Tests that RunnableImpl can update app context.
    /// This test demonstrates the app context update feature.
    #[test]
    fn test_runnable_impl_update_app_context() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);

        let mut new_app_context = create_test_app_context();
        new_app_context.config.frame_delay = 100;

        runnable.update_app_context(new_app_context.clone());
        assert_eq!(runnable.get_app_context(), &new_app_context);
    }

    /// Tests that RunnableImpl can receive updates from channels.
    /// This test demonstrates the message receiving feature.
    #[test]
    fn test_runnable_impl_receive_updates() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);

        let (app_context_sender, app_context_receiver) = mpsc::channel();
        let (message_sender, message_receiver) = mpsc::channel();

        runnable.set_app_context_receiver(app_context_receiver);
        runnable.set_message_receiver(message_receiver);

        // Send test data
        let field_update = create_test_field_update(
            EntityType::App,
            "test",
            "field",
            Value::String("value".to_string()),
        );
        app_context_sender
            .send((Uuid::new_v4(), vec![field_update]))
            .unwrap();
        message_sender
            .send((Uuid::new_v4(), Message::RedrawApp))
            .unwrap();

        let (updated_app_context, messages) = runnable.receive_updates();
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0], Message::RedrawApp);
    }

    /// Tests that RunnableImpl processes messages correctly.
    /// This test demonstrates the message processing feature.
    #[test]
    fn test_runnable_impl_process_messages() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context.clone());

        let messages = vec![Message::Start, Message::Pause, Message::Terminate];

        runnable.process(app_context.clone(), vec![Message::Start]);
        assert_eq!(runnable.running_state, RunnableState::Running);

        runnable.process(app_context.clone(), vec![Message::Pause]);
        assert_eq!(runnable.running_state, RunnableState::Paused);

        runnable.process(app_context.clone(), vec![Message::Terminate]);
        assert_eq!(runnable.running_state, RunnableState::Terminated);
    }

    /// Tests that RunnableImpl can send messages through channels.
    /// This test demonstrates the message sending feature.
    #[test]
    fn test_runnable_impl_send_message() {
        let app_context = create_test_app_context();
        let mut runnable = RunnableImpl::new(app_context);

        let (message_sender, message_receiver) = mpsc::channel();
        runnable.set_message_sender(message_sender);

        runnable.send_message(Message::RedrawApp);

        let (uuid, received_message) = message_receiver.recv().unwrap();
        assert_eq!(received_message, Message::RedrawApp);
        assert_eq!(uuid, runnable.get_uuid());
    }

    /// Tests that ThreadManager can be created with an AppContext.
    /// This test demonstrates the thread manager creation feature.
    #[test]
    fn test_thread_manager_new() {
        let app_context = create_test_app_context();
        let manager = ThreadManager::new(app_context.clone());

        assert_eq!(manager.app_context, app_context);
        assert!(manager.threads.is_empty());
        assert!(manager.app_context_senders.is_empty());
        assert!(manager.message_senders.is_empty());
        assert!(manager.app_context_receivers.is_empty());
        assert!(manager.message_receivers.is_empty());
    }

    /// Tests that ThreadManager can spawn threads.
    /// This test demonstrates the thread spawning feature.
    #[test]
    fn test_thread_manager_spawn_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);

        let uuid = manager.spawn_thread(runnable);

        assert!(manager.threads.contains_key(&uuid));
        assert!(manager.app_context_senders.contains_key(&uuid));
        assert!(manager.message_senders.contains_key(&uuid));
        assert!(manager.app_context_receivers.contains_key(&uuid));
        assert!(manager.message_receivers.contains_key(&uuid));

        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can remove threads.
    /// This test demonstrates the thread removal feature.
    #[test]
    fn test_thread_manager_remove_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);

        let uuid = manager.spawn_thread(runnable);
        assert!(manager.threads.contains_key(&uuid));

        manager.remove_thread(uuid);
        assert!(!manager.threads.contains_key(&uuid));
        assert!(!manager.app_context_senders.contains_key(&uuid));
        assert!(!manager.message_senders.contains_key(&uuid));
    }

    /// Tests that ThreadManager can send messages to specific threads.
    /// This test demonstrates the targeted message sending feature.
    #[test]
    fn test_thread_manager_send_message_to_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);

        let uuid = manager.spawn_thread(runnable);
        let message = (Uuid::new_v4(), Message::RedrawApp);

        manager.send_message_to_thread(message, uuid);

        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can send messages to all threads.
    /// This test demonstrates the broadcast message sending feature.
    #[test]
    fn test_thread_manager_send_message_to_all_threads() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable1 = RunnableImpl::new(app_context.clone());
        let runnable2 = RunnableImpl::new(app_context.clone());

        let uuid1 = manager.spawn_thread(runnable1);
        let uuid2 = manager.spawn_thread(runnable2);

        let message = (Uuid::new_v4(), Message::RedrawApp);
        manager.send_message_to_all_threads(message);

        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can send app context updates to specific threads.
    /// This test demonstrates the targeted app context update feature.
    #[test]
    fn test_thread_manager_send_app_context_update_to_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);

        let uuid = manager.spawn_thread(runnable);
        let field_update = create_test_field_update(
            EntityType::App,
            "test",
            "field",
            Value::String("value".to_string()),
        );

        manager.send_app_context_update_to_thread(vec![field_update], uuid);

        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can send app context updates to all threads.
    /// This test demonstrates the broadcast app context update feature.
    #[test]
    fn test_thread_manager_send_app_context_update_to_all_threads() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable1 = RunnableImpl::new(app_context.clone());
        let runnable2 = RunnableImpl::new(app_context.clone());

        let uuid1 = manager.spawn_thread(runnable1);
        let uuid2 = manager.spawn_thread(runnable2);

        let field_update = create_test_field_update(
            EntityType::App,
            "test",
            "field",
            Value::String("value".to_string()),
        );
        let sender_uuid = Uuid::new_v4();

        manager.send_app_context_update_to_all_threads((sender_uuid, vec![field_update]));

        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can stop all threads.
    /// This test demonstrates the thread stopping feature.
    #[test]
    fn test_thread_manager_stop() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);

        let uuid = manager.spawn_thread(runnable);
        assert!(manager.threads.contains_key(&uuid));

        manager.stop();
        manager.join_threads();

        assert!(manager.threads.is_empty());
    }

    /// Tests that ThreadManager can pause all threads.
    /// This test demonstrates the thread pausing feature.
    #[test]
    fn test_thread_manager_pause() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);

        let uuid = manager.spawn_thread(runnable);
        manager.pause();

        // Clean up
        manager.stop();
        manager.join_threads();
    }

    /// Tests that ThreadManager can calculate hash values.
    /// This test demonstrates the hash calculation feature.
    #[test]
    fn test_thread_manager_get_hash() {
        let app_context = create_test_app_context();
        let manager = ThreadManager::new(app_context);

        let test_string = "test";
        let hash1 = manager.get_hash(&test_string);
        let hash2 = manager.get_hash(&test_string);
        assert_eq!(hash1, hash2);

        let test_string2 = "different";
        let hash3 = manager.get_hash(&test_string2);
        assert_ne!(hash1, hash3);
    }

    /// Tests that ThreadManager can join all threads.
    /// This test demonstrates the thread joining feature.
    #[test]
    fn test_thread_manager_join_threads() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());
        let runnable = RunnableImpl::new(app_context);

        let uuid = manager.spawn_thread(runnable);
        assert!(manager.threads.contains_key(&uuid));

        manager.stop();
        manager.join_threads();

        assert!(manager.threads.is_empty());
    }

    /// Tests that run_script_in_thread spawns a thread correctly.
    /// This test demonstrates the script thread spawning feature.
    #[test]
    fn test_run_script_in_thread() {
        let app_context = create_test_app_context();
        let mut manager = ThreadManager::new(app_context.clone());

        let muxbox_id = "test_muxbox".to_string();
        let choice_id = "test_choice".to_string();

        let uuid = run_script_in_thread(app_context, &mut manager, muxbox_id, choice_id);

        assert!(manager.threads.contains_key(&uuid));
        assert!(manager.app_context_senders.contains_key(&uuid));
        assert!(manager.message_senders.contains_key(&uuid));

        // Clean up
        manager.stop();
        manager.join_threads();
    }

    // T0328: REMOVED test_message_muxbox_output_update - replaced by StreamUpdateMessage tests

    /// Tests that Message::MuxBoxScriptUpdate contains correct data.
    /// This test demonstrates the muxbox script update message feature.
    #[test]
    fn test_message_muxbox_script_update() {
        let muxbox_id = "test_muxbox".to_string();
        let script = vec!["echo 'test'".to_string(), "ls".to_string()];

        let message = Message::MuxBoxScriptUpdate(muxbox_id.clone(), script.clone());

        match message {
            Message::MuxBoxScriptUpdate(id, script_content) => {
                assert_eq!(id, muxbox_id);
                assert_eq!(script_content, script);
            }
            _ => panic!("Expected MuxBoxScriptUpdate message"),
        }
    }

    /// Tests that Message::ReplaceMuxBox contains correct data.
    /// This test demonstrates the muxbox replacement message feature.
    #[test]
    fn test_message_replace_muxbox() {
        let muxbox_id = "test_muxbox".to_string();
        let muxbox = create_test_muxbox("new_muxbox");

        let message = Message::ReplaceMuxBox(muxbox_id.clone(), muxbox.clone());

        match message {
            Message::ReplaceMuxBox(id, new_muxbox) => {
                assert_eq!(id, muxbox_id);
                assert_eq!(new_muxbox.id, muxbox.id);
            }
            _ => panic!("Expected ReplaceMuxBox message"),
        }
    }

    /// Tests that Message::SwitchActiveLayout contains correct data.
    /// This test demonstrates the layout switching message feature.
    #[test]
    fn test_message_switch_active_layout() {
        let layout_id = "test_layout".to_string();
        let message = Message::SwitchActiveLayout(layout_id.clone());

        match message {
            Message::SwitchActiveLayout(id) => {
                assert_eq!(id, layout_id);
            }
            _ => panic!("Expected SwitchActiveLayout message"),
        }
    }

    /// Tests that Message::KeyPress contains correct data.
    /// This test demonstrates the key press message feature.
    #[test]
    fn test_message_key_press() {
        let key = "ctrl+c".to_string();
        let message = Message::KeyPress(key.clone());

        match message {
            Message::KeyPress(pressed_key) => {
                assert_eq!(pressed_key, key);
            }
            _ => panic!("Expected KeyPress message"),
        }
    }

    /// Tests that Message::ExternalMessage contains correct data.
    /// This test demonstrates the external message feature.
    #[test]
    fn test_message_external_message() {
        let external_msg = "external command".to_string();
        let message = Message::ExternalMessage(external_msg.clone());

        match message {
            Message::ExternalMessage(msg) => {
                assert_eq!(msg, external_msg);
            }
            _ => panic!("Expected ExternalMessage message"),
        }
    }

    /// Tests that Message::AddBox contains correct data.
    /// This test demonstrates the box addition message feature.
    #[test]
    fn test_message_add_box() {
        let box_id = "test_box".to_string();
        let muxbox = create_test_muxbox("new_box");

        let message = Message::AddBox(box_id.clone(), muxbox.clone());

        match message {
            Message::AddBox(id, new_muxbox) => {
                assert_eq!(id, box_id);
                assert_eq!(new_muxbox.id, muxbox.id);
            }
            _ => panic!("Expected AddBox message"),
        }
    }

    /// Tests that Message::RemoveBox contains correct data.
    /// This test demonstrates the box removal message feature.
    #[test]
    fn test_message_remove_box() {
        let box_id = "test_box".to_string();
        let message = Message::RemoveBox(box_id.clone());

        match message {
            Message::RemoveBox(id) => {
                assert_eq!(id, box_id);
            }
            _ => panic!("Expected RemoveBox message"),
        }
    }

    /// Tests that simple messages are created correctly.
    /// This test demonstrates the simple message creation feature.
    #[test]
    fn test_simple_messages() {
        let exit_msg = Message::Exit;
        let terminate_msg = Message::Terminate;
        let pause_msg = Message::Pause;
        let start_msg = Message::Start;
        let resize_msg = Message::Resize;
        let redraw_app_msg = Message::RedrawApp;

        // Test that they can be created and compared
        assert_eq!(exit_msg, Message::Exit);
        assert_eq!(terminate_msg, Message::Terminate);
        assert_eq!(pause_msg, Message::Pause);
        assert_eq!(start_msg, Message::Start);
        assert_eq!(resize_msg, Message::Resize);
        assert_eq!(redraw_app_msg, Message::RedrawApp);

        // Test that they are different from each other
        assert_ne!(exit_msg, terminate_msg);
        assert_ne!(pause_msg, start_msg);
        assert_ne!(resize_msg, redraw_app_msg);
    }

    /// Tests that scroll messages are created correctly.
    /// This test demonstrates the scroll message creation feature.
    #[test]
    fn test_scroll_messages() {
        let scroll_down = Message::ScrollMuxBoxDown();
        let scroll_up = Message::ScrollMuxBoxUp();
        let scroll_left = Message::ScrollMuxBoxLeft();
        let scroll_right = Message::ScrollMuxBoxRight();
        let scroll_page_up = Message::ScrollMuxBoxPageUp();
        let scroll_page_down = Message::ScrollMuxBoxPageDown();

        assert_eq!(scroll_down, Message::ScrollMuxBoxDown());
        assert_eq!(scroll_up, Message::ScrollMuxBoxUp());
        assert_eq!(scroll_left, Message::ScrollMuxBoxLeft());
        assert_eq!(scroll_right, Message::ScrollMuxBoxRight());
        assert_eq!(scroll_page_up, Message::ScrollMuxBoxPageUp());
        assert_eq!(scroll_page_down, Message::ScrollMuxBoxPageDown());

        assert_ne!(scroll_down, scroll_up);
        assert_ne!(scroll_left, scroll_right);
    }

    /// Tests that navigation messages are created correctly.
    /// This test demonstrates the navigation message creation feature.
    #[test]
    fn test_navigation_messages() {
        let next_muxbox = Message::NextMuxBox();
        let previous_muxbox = Message::PreviousMuxBox();

        assert_eq!(next_muxbox, Message::NextMuxBox());
        assert_eq!(previous_muxbox, Message::PreviousMuxBox());
        assert_ne!(next_muxbox, previous_muxbox);
    }

    /// Tests that box refresh messages are created correctly.
    /// This test demonstrates the box refresh message feature.
    #[test]
    fn test_box_refresh_messages() {
        let box_id = "test_box".to_string();
        let start_refresh = Message::StartBoxRefresh(box_id.clone());
        let stop_refresh = Message::StopBoxRefresh(box_id.clone());
        let event_refresh = Message::MuxBoxEventRefresh(box_id.clone());

        match start_refresh {
            Message::StartBoxRefresh(id) => assert_eq!(id, box_id),
            _ => panic!("Expected StartBoxRefresh"),
        }

        match stop_refresh {
            Message::StopBoxRefresh(id) => assert_eq!(id, box_id),
            _ => panic!("Expected StopBoxRefresh"),
        }

        match event_refresh {
            Message::MuxBoxEventRefresh(id) => assert_eq!(id, box_id),
            _ => panic!("Expected MuxBoxEventRefresh"),
        }
    }

    /// Tests that RedrawMuxBox message is created correctly.
    /// This test demonstrates the muxbox redraw message feature.
    #[test]
    fn test_redraw_muxbox_message() {
        let muxbox_id = "test_muxbox".to_string();
        let redraw_msg = Message::RedrawMuxBox(muxbox_id.clone());

        match redraw_msg {
            Message::RedrawMuxBox(id) => assert_eq!(id, muxbox_id),
            _ => panic!("Expected RedrawMuxBox message"),
        }
    }
}
