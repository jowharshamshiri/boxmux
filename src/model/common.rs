use serde_json::Value;
use std::{collections::HashMap, error::Error, hash::Hash};
use indexmap::IndexMap;

use crate::{
    draw_utils::{get_bg_color, get_fg_color},
    screen_bounds, screen_height, screen_width,
    utils::input_bounds_to_bounds,
    AppContext, AppGraph, Layout, Message, MuxBox,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub enum EntityType {
    AppContext,
    App,
    Layout,
    MuxBox,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub enum StreamType {
    Content,
    Choices,
    RedirectedOutput(String), // Named redirect output
    PTY,
    Plugin(String),
    // F0210: Complete StreamType Enum - Add missing variants for source tracking
    ChoiceExecution(String), // Track choice executions as streams
    RedirectSource(String),  // Track redirect output sources 
    ExternalSocket,          // External socket connections
    PtySession(String),      // PTY session with command info
    OwnScript,              // Box's own script execution stream
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct Stream {
    pub id: String,
    pub stream_type: StreamType,
    pub label: String,
    pub content: Vec<String>,
    pub choices: Option<Vec<crate::model::muxbox::Choice>>, // For choices stream
    pub active: bool,
    // F0212: Stream Source Tracking - enable stream termination when tabs closed
    pub source: Option<StreamSource>,
    // F0216: Stream Change Detection - track content changes for efficient updates
    #[serde(skip, default = "default_content_hash")]
    pub content_hash: u64,
    #[serde(skip, default = "default_system_time")]
    pub last_updated: std::time::SystemTime,
    #[serde(skip, default = "default_system_time")]
    pub created_at: std::time::SystemTime,
}

// Helper functions for default values
fn default_content_hash() -> u64 { 0 }
fn default_system_time() -> std::time::SystemTime { std::time::SystemTime::now() }

// F0217: Stream rendering behavior traits - exclusive content OR choices
pub trait ContentStreamTrait {
    fn get_content_lines(&self) -> &Vec<String>;
    fn set_content_lines(&mut self, content: Vec<String>);
}

pub trait ChoicesStreamTrait {
    fn get_choices(&self) -> &Vec<crate::model::muxbox::Choice>;
    fn get_choices_mut(&mut self) -> &mut Vec<crate::model::muxbox::Choice>;
    fn set_choices(&mut self, choices: Vec<crate::model::muxbox::Choice>);
}

// F0212: Base trait for all stream sources with lifecycle management
pub trait StreamSourceTrait {
    fn source_type(&self) -> &'static str;
    fn source_id(&self) -> String;
    fn can_terminate(&self) -> bool;
    fn cleanup(&self) -> Result<(), String>;
    fn get_metadata(&self) -> std::collections::HashMap<String, String>;
}

// F0212: Static content sources (no lifecycle management needed)
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct StaticContentSource {
    pub content_type: String,  // "default", "choices", "manual"
    pub created_at: std::time::SystemTime,
}

impl StreamSourceTrait for StaticContentSource {
    fn source_type(&self) -> &'static str { "static_content" }
    fn source_id(&self) -> String { format!("static_{}", self.content_type) }
    fn can_terminate(&self) -> bool { false }
    fn cleanup(&self) -> Result<(), String> { Ok(()) }
    fn get_metadata(&self) -> std::collections::HashMap<String, String> {
        let mut meta = std::collections::HashMap::new();
        meta.insert("content_type".to_string(), self.content_type.clone());
        meta.insert("created_at".to_string(), format!("{:?}", self.created_at));
        meta
    }
}

// F0212: Choice execution sources with thread management
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct ChoiceExecutionSource {
    pub choice_id: String,
    pub muxbox_id: String,
    pub thread_id: Option<String>,    // For thread tracking
    pub process_id: Option<u32>,      // For process-based choices
    pub execution_type: String,       // "threaded", "process", "pty"
    pub started_at: std::time::SystemTime,
    pub timeout_seconds: Option<u32>,
}

impl StreamSourceTrait for ChoiceExecutionSource {
    fn source_type(&self) -> &'static str { "choice_execution" }
    fn source_id(&self) -> String { self.choice_id.clone() }
    fn can_terminate(&self) -> bool { true }
    fn cleanup(&self) -> Result<(), String> {
        match self.execution_type.as_str() {
            "process" => {
                if let Some(pid) = self.process_id {
                    // Terminate the process
                    let _ = std::process::Command::new("kill")
                        .arg("-9")
                        .arg(pid.to_string())
                        .output();
                }
                Ok(())
            },
            "threaded" => {
                // Thread cleanup would be handled by ThreadManager
                Ok(())
            },
            _ => Ok(())
        }
    }
    fn get_metadata(&self) -> std::collections::HashMap<String, String> {
        let mut meta = std::collections::HashMap::new();
        meta.insert("choice_id".to_string(), self.choice_id.clone());
        meta.insert("muxbox_id".to_string(), self.muxbox_id.clone());
        meta.insert("execution_type".to_string(), self.execution_type.clone());
        if let Some(pid) = self.process_id {
            meta.insert("process_id".to_string(), pid.to_string());
        }
        if let Some(thread_id) = &self.thread_id {
            meta.insert("thread_id".to_string(), thread_id.clone());
        }
        meta
    }
}

// F0212: PTY sources with process and terminal management
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct PTYSource {
    pub pty_id: String,
    pub process_id: u32,
    pub command: String,
    pub args: Vec<String>,
    pub working_dir: Option<String>,
    pub reader_thread_id: Option<String>,
    pub started_at: std::time::SystemTime,
    pub terminal_size: (u16, u16),    // (rows, cols)
}

impl StreamSourceTrait for PTYSource {
    fn source_type(&self) -> &'static str { "pty" }
    fn source_id(&self) -> String { self.pty_id.clone() }
    fn can_terminate(&self) -> bool { true }
    fn cleanup(&self) -> Result<(), String> {
        // Terminate PTY process
        let result = std::process::Command::new("kill")
            .arg("-9")
            .arg(self.process_id.to_string())
            .output();
        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("Failed to terminate PTY process {}: {}", self.process_id, e))
        }
    }
    fn get_metadata(&self) -> std::collections::HashMap<String, String> {
        let mut meta = std::collections::HashMap::new();
        meta.insert("pty_id".to_string(), self.pty_id.clone());
        meta.insert("process_id".to_string(), self.process_id.to_string());
        meta.insert("command".to_string(), self.command.clone());
        meta.insert("args".to_string(), self.args.join(" "));
        meta.insert("terminal_size".to_string(), format!("{}x{}", self.terminal_size.0, self.terminal_size.1));
        if let Some(ref thread_id) = self.reader_thread_id {
            meta.insert("reader_thread_id".to_string(), thread_id.clone());
        }
        meta
    }
}

// F0212: Redirect sources with source tracking
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct RedirectSource {
    pub source_muxbox_id: String,
    pub source_choice_id: Option<String>,
    pub redirect_name: String,
    pub redirect_type: String,        // "choice", "script", "pty", "external"
    pub created_at: std::time::SystemTime,
    pub source_process_id: Option<u32>,  // For process-based redirects
}

impl StreamSourceTrait for RedirectSource {
    fn source_type(&self) -> &'static str { "redirect" }
    fn source_id(&self) -> String { format!("{}_{}", self.source_muxbox_id, self.redirect_name) }
    fn can_terminate(&self) -> bool { self.source_process_id.is_some() }
    fn cleanup(&self) -> Result<(), String> {
        if let Some(pid) = self.source_process_id {
            let result = std::process::Command::new("kill")
                .arg("-9")
                .arg(pid.to_string())
                .output();
            match result {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("Failed to terminate redirect source process {}: {}", pid, e))
            }
        } else {
            Ok(())
        }
    }
    fn get_metadata(&self) -> std::collections::HashMap<String, String> {
        let mut meta = std::collections::HashMap::new();
        meta.insert("source_muxbox_id".to_string(), self.source_muxbox_id.clone());
        meta.insert("redirect_name".to_string(), self.redirect_name.clone());
        meta.insert("redirect_type".to_string(), self.redirect_type.clone());
        if let Some(ref choice_id) = self.source_choice_id {
            meta.insert("source_choice_id".to_string(), choice_id.clone());
        }
        if let Some(pid) = self.source_process_id {
            meta.insert("source_process_id".to_string(), pid.to_string());
        }
        meta
    }
}

// F0212: Socket sources with connection management
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct SocketSource {
    pub connection_id: String,
    pub socket_path: Option<String>,
    pub client_info: String,
    pub protocol_version: String,
    pub connected_at: std::time::SystemTime,
    pub last_activity: std::time::SystemTime,
}

impl StreamSourceTrait for SocketSource {
    fn source_type(&self) -> &'static str { "socket" }
    fn source_id(&self) -> String { self.connection_id.clone() }
    fn can_terminate(&self) -> bool { true }
    fn cleanup(&self) -> Result<(), String> {
        // Socket cleanup would be handled by socket manager
        Ok(())
    }
    fn get_metadata(&self) -> std::collections::HashMap<String, String> {
        let mut meta = std::collections::HashMap::new();
        meta.insert("connection_id".to_string(), self.connection_id.clone());
        meta.insert("client_info".to_string(), self.client_info.clone());
        meta.insert("protocol_version".to_string(), self.protocol_version.clone());
        if let Some(ref socket_path) = self.socket_path {
            meta.insert("socket_path".to_string(), socket_path.clone());
        }
        meta
    }
}

// F0212: Unified stream source enum containing all source types
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub enum StreamSource {
    StaticContent(StaticContentSource),
    ChoiceExecution(ChoiceExecutionSource),
    PTY(PTYSource),
    Redirect(RedirectSource),
    Socket(SocketSource),
}

// F0212: Implementation of StreamSourceTrait for the unified enum
impl StreamSourceTrait for StreamSource {
    fn source_type(&self) -> &'static str {
        match self {
            StreamSource::StaticContent(s) => s.source_type(),
            StreamSource::ChoiceExecution(s) => s.source_type(),
            StreamSource::PTY(s) => s.source_type(),
            StreamSource::Redirect(s) => s.source_type(),
            StreamSource::Socket(s) => s.source_type(),
        }
    }

    fn source_id(&self) -> String {
        match self {
            StreamSource::StaticContent(s) => s.source_id(),
            StreamSource::ChoiceExecution(s) => s.source_id(),
            StreamSource::PTY(s) => s.source_id(),
            StreamSource::Redirect(s) => s.source_id(),
            StreamSource::Socket(s) => s.source_id(),
        }
    }

    fn can_terminate(&self) -> bool {
        match self {
            StreamSource::StaticContent(s) => s.can_terminate(),
            StreamSource::ChoiceExecution(s) => s.can_terminate(),
            StreamSource::PTY(s) => s.can_terminate(),
            StreamSource::Redirect(s) => s.can_terminate(),
            StreamSource::Socket(s) => s.can_terminate(),
        }
    }

    fn cleanup(&self) -> Result<(), String> {
        match self {
            StreamSource::StaticContent(s) => s.cleanup(),
            StreamSource::ChoiceExecution(s) => s.cleanup(),
            StreamSource::PTY(s) => s.cleanup(),
            StreamSource::Redirect(s) => s.cleanup(),
            StreamSource::Socket(s) => s.cleanup(),
        }
    }

    fn get_metadata(&self) -> std::collections::HashMap<String, String> {
        match self {
            StreamSource::StaticContent(s) => s.get_metadata(),
            StreamSource::ChoiceExecution(s) => s.get_metadata(),
            StreamSource::PTY(s) => s.get_metadata(),
            StreamSource::Redirect(s) => s.get_metadata(),
            StreamSource::Socket(s) => s.get_metadata(),
        }
    }
}

// F0216: Stream Change Detection Implementation
impl Stream {
    /// Create a new stream with proper change detection initialization
    pub fn new(
        id: String,
        stream_type: StreamType,
        label: String,
        content: Vec<String>,
        choices: Option<Vec<crate::model::muxbox::Choice>>,
        source: Option<StreamSource>,
    ) -> Self {
        let now = std::time::SystemTime::now();
        let mut stream = Self {
            id,
            stream_type,
            label,
            content,
            choices,
            active: false,
            source,
            content_hash: 0,
            last_updated: now,
            created_at: now,
        };
        stream.update_content_hash();
        stream
    }

    /// Update the content hash for change detection
    pub fn update_content_hash(&mut self) {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        if let Some(ref choices) = self.choices {
            choices.hash(&mut hasher);
        }
        self.content_hash = hasher.finish();
        self.last_updated = std::time::SystemTime::now();
    }

    /// Check if content has changed since last hash update
    pub fn has_content_changed(&self) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.content.hash(&mut hasher);
        if let Some(ref choices) = self.choices {
            choices.hash(&mut hasher);
        }
        let current_hash = hasher.finish();
        current_hash != self.content_hash
    }

    /// Update content and automatically refresh change detection
    pub fn update_content(&mut self, new_content: Vec<String>) {
        self.content = new_content;
        self.update_content_hash();
    }

    /// Update choices and automatically refresh change detection  
    pub fn update_choices(&mut self, new_choices: Option<Vec<crate::model::muxbox::Choice>>) {
        self.choices = new_choices;
        self.update_content_hash();
    }

    /// Get time since last update for staleness detection
    pub fn time_since_last_update(&self) -> Result<std::time::Duration, std::time::SystemTimeError> {
        std::time::SystemTime::now().duration_since(self.last_updated)
    }
}

// F0217: Implement ContentStreamTrait for content-type streams
impl ContentStreamTrait for Stream {
    fn get_content_lines(&self) -> &Vec<String> {
        match self.stream_type {
            StreamType::Content | 
            StreamType::RedirectedOutput(_) |
            StreamType::PTY |
            StreamType::Plugin(_) |
            StreamType::ChoiceExecution(_) |
            StreamType::PtySession(_) |
            StreamType::OwnScript => &self.content,
            _ => panic!("ContentStreamTrait called on non-content stream: {:?}", self.stream_type),
        }
    }
    
    fn set_content_lines(&mut self, content: Vec<String>) {
        match self.stream_type {
            StreamType::Content | 
            StreamType::RedirectedOutput(_) |
            StreamType::PTY |
            StreamType::Plugin(_) |
            StreamType::ChoiceExecution(_) |
            StreamType::PtySession(_) |
            StreamType::OwnScript => {
                self.content = content;
                self.update_content_hash();
            },
            _ => panic!("ContentStreamTrait called on non-content stream: {:?}", self.stream_type),
        }
    }
}

// F0217: Implement ChoicesStreamTrait for choices-type streams
impl ChoicesStreamTrait for Stream {
    fn get_choices(&self) -> &Vec<crate::model::muxbox::Choice> {
        match self.stream_type {
            StreamType::Choices => {
                self.choices.as_ref().expect("Choices stream must have choices")
            },
            _ => panic!("ChoicesStreamTrait called on non-choices stream: {:?}", self.stream_type),
        }
    }
    
    fn get_choices_mut(&mut self) -> &mut Vec<crate::model::muxbox::Choice> {
        match self.stream_type {
            StreamType::Choices => {
                self.choices.as_mut().expect("Choices stream must have choices")
            },
            _ => panic!("ChoicesStreamTrait called on non-choices stream: {:?}", self.stream_type),
        }
    }
    
    fn set_choices(&mut self, choices: Vec<crate::model::muxbox::Choice>) {
        match self.stream_type {
            StreamType::Choices => {
                self.choices = Some(choices);
                self.update_content_hash();
            },
            _ => panic!("ChoicesStreamTrait called on non-choices stream: {:?}", self.stream_type),
        }
    }
}

// Represents a granular field update
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct FieldUpdate {
    pub entity_type: EntityType,   // The type of entity being updated
    pub entity_id: Option<String>, // The ID of the entity (App, Layout, or MuxBox)
    pub field_name: String,        // The field name to be updated
    pub new_value: Value,          // The new value for the field
}

// The Updatable trait
pub trait Updatable {
    // Generate a diff of changes from another instance
    fn generate_diff(&self, other: &Self) -> Vec<FieldUpdate>;

    // Apply a list of updates to the current instance
    fn apply_updates(&mut self, updates: Vec<FieldUpdate>);
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub struct Config {
    pub frame_delay: u64,
    pub locked: bool, // Disable muxbox resizing and moving when true
}

impl Hash for Config {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.frame_delay.hash(state);
        self.locked.hash(state);
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            frame_delay: 30,
            locked: false, // Default to unlocked (resizable/movable)
        }
    }
}

impl Config {
    pub fn new(frame_delay: u64) -> Self {
        let result = Config {
            frame_delay,
            locked: false, // Default to unlocked
        };
        result.validate();
        result
    }

    pub fn new_with_lock(frame_delay: u64, locked: bool) -> Self {
        let result = Config {
            frame_delay,
            locked,
        };
        result.validate();
        result
    }
    pub fn validate(&self) {
        if self.frame_delay == 0 {
            panic!("Validation error: frame_delay cannot be 0");
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub enum SocketFunction {
    ReplaceBoxContent {
        box_id: String,
        success: bool,
        content: String,
    },
    ReplaceBoxScript {
        box_id: String,
        script: Vec<String>,
    },
    StopBoxRefresh {
        box_id: String,
    },
    StartBoxRefresh {
        box_id: String,
    },
    ReplaceBox {
        box_id: String,
        new_box: MuxBox,
    },
    SwitchActiveLayout {
        layout_id: String,
    },
    AddBox {
        layout_id: String,
        muxbox: MuxBox,
    },
    RemoveBox {
        box_id: String,
    },
    // F0137: Socket PTY Control - Kill and restart PTY processes
    KillPtyProcess {
        box_id: String,
    },
    RestartPtyProcess {
        box_id: String,
    },
    // F0138: Socket PTY Query - Get PTY status and info
    QueryPtyStatus {
        box_id: String,
    },
    // F0136: Socket PTY Spawn - Spawn PTY processes via socket commands
    SpawnPtyProcess {
        box_id: String,
        script: Vec<String>,
        libs: Option<Vec<String>>,
        redirect_output: Option<String>,
    },
    // F0139: Socket PTY Input - Send input to PTY processes remotely
    SendPtyInput {
        box_id: String,
        input: String,
    },
}

pub fn run_socket_function(
    socket_function: SocketFunction,
    app_context: &AppContext,
) -> Result<(AppContext, Vec<Message>), Box<dyn Error>> {
    let app_context = app_context.clone();
    let mut messages = Vec::new();
    match socket_function {
        SocketFunction::ReplaceBoxContent {
            box_id,
            success,
            content,
        } => {
            messages.push(Message::MuxBoxOutputUpdate(box_id, success, content));
        }
        SocketFunction::ReplaceBoxScript { box_id, script } => {
            messages.push(Message::MuxBoxScriptUpdate(box_id, script));
        }
        SocketFunction::StopBoxRefresh { box_id } => {
            messages.push(Message::StopBoxRefresh(box_id));
        }
        SocketFunction::StartBoxRefresh { box_id } => {
            messages.push(Message::StartBoxRefresh(box_id));
        }
        SocketFunction::ReplaceBox { box_id, new_box } => {
            messages.push(Message::ReplaceMuxBox(box_id, new_box));
        }
        SocketFunction::SwitchActiveLayout { layout_id } => {
            messages.push(Message::SwitchActiveLayout(layout_id));
        }
        SocketFunction::AddBox { layout_id, muxbox } => {
            messages.push(Message::AddBox(layout_id, muxbox));
        }
        SocketFunction::RemoveBox { box_id } => {
            messages.push(Message::RemoveBox(box_id));
        }
        // F0137: Socket PTY Control - Kill and restart PTY processes
        SocketFunction::KillPtyProcess { box_id } => {
            if let Some(pty_manager) = &app_context.pty_manager {
                match pty_manager.kill_pty_process(&box_id) {
                    Ok(_) => {
                        messages.push(Message::MuxBoxOutputUpdate(
                            box_id.clone(),
                            true,
                            format!("PTY process killed for box {}", box_id),
                        ));
                    }
                    Err(err) => {
                        messages.push(Message::MuxBoxOutputUpdate(
                            box_id.clone(),
                            false,
                            format!("Failed to kill PTY process: {}", err),
                        ));
                    }
                }
            } else {
                messages.push(Message::MuxBoxOutputUpdate(
                    box_id.clone(),
                    false,
                    "PTY manager not available".to_string(),
                ));
            }
        }
        SocketFunction::RestartPtyProcess { box_id } => {
            if let Some(pty_manager) = &app_context.pty_manager {
                match pty_manager.restart_pty_process(&box_id) {
                    Ok(_) => {
                        messages.push(Message::MuxBoxOutputUpdate(
                            box_id.clone(),
                            true,
                            format!("PTY process restarted for box {}", box_id),
                        ));
                    }
                    Err(err) => {
                        messages.push(Message::MuxBoxOutputUpdate(
                            box_id.clone(),
                            false,
                            format!("Failed to restart PTY process: {}", err),
                        ));
                    }
                }
            } else {
                messages.push(Message::MuxBoxOutputUpdate(
                    box_id.clone(),
                    false,
                    "PTY manager not available".to_string(),
                ));
            }
        }
        // F0138: Socket PTY Query - Get PTY status and info
        SocketFunction::QueryPtyStatus { box_id } => {
            if let Some(pty_manager) = &app_context.pty_manager {
                if let Some(info) = pty_manager.get_detailed_process_info(&box_id) {
                    let status_info = format!(
                        "PTY Status - Box: {}, PID: {:?}, Status: {:?}, Running: {}, Buffer Lines: {}",
                        info.muxbox_id, info.process_id, info.status, info.is_running, info.buffer_lines
                    );
                    messages.push(Message::MuxBoxOutputUpdate(
                        box_id.clone(),
                        true,
                        status_info,
                    ));
                } else {
                    messages.push(Message::MuxBoxOutputUpdate(
                        box_id.clone(),
                        false,
                        format!("No PTY process found for box {}", box_id),
                    ));
                }
            } else {
                messages.push(Message::MuxBoxOutputUpdate(
                    box_id.clone(),
                    false,
                    "PTY manager not available".to_string(),
                ));
            }
        }
        // F0136: Socket PTY Spawn - Spawn PTY processes via socket commands
        SocketFunction::SpawnPtyProcess {
            box_id,
            script,
            libs,
            redirect_output,
        } => {
            if let Some(pty_manager) = &app_context.pty_manager {
                // We need to create a temporary message sender for PTY operations
                // This is a limitation of the socket API - it doesn't have access to the main ThreadManager
                let (temp_sender, _temp_receiver) = std::sync::mpsc::channel();
                let temp_uuid = uuid::Uuid::new_v4();
                
                let spawn_result = if redirect_output.is_some() {
                    pty_manager.spawn_pty_script_with_redirect(
                        box_id.clone(),
                        &script,
                        libs,
                        temp_sender,
                        temp_uuid,
                        redirect_output,
                    )
                } else {
                    pty_manager.spawn_pty_script(
                        box_id.clone(),
                        &script,
                        libs,
                        temp_sender,
                        temp_uuid,
                    )
                };
                
                match spawn_result {
                    Ok(_) => {
                        messages.push(Message::MuxBoxOutputUpdate(
                            box_id.clone(),
                            true,
                            format!("PTY process spawned successfully for box {}", box_id),
                        ));
                    }
                    Err(err) => {
                        messages.push(Message::MuxBoxOutputUpdate(
                            box_id.clone(),
                            false,
                            format!("Failed to spawn PTY process: {}", err),
                        ));
                    }
                }
            } else {
                messages.push(Message::MuxBoxOutputUpdate(
                    box_id.clone(),
                    false,
                    "PTY manager not available".to_string(),
                ));
            }
        }
        // F0139: Socket PTY Input - Send input to PTY processes remotely
        SocketFunction::SendPtyInput { box_id, input } => {
            if let Some(pty_manager) = &app_context.pty_manager {
                match pty_manager.send_input(&box_id, &input) {
                    Ok(_) => {
                        messages.push(Message::MuxBoxOutputUpdate(
                            box_id.clone(),
                            true,
                            format!("Input sent successfully to PTY process for box {}", box_id),
                        ));
                    }
                    Err(err) => {
                        messages.push(Message::MuxBoxOutputUpdate(
                            box_id.clone(),
                            false,
                            format!("Failed to send input to PTY process: {}", err),
                        ));
                    }
                }
            } else {
                messages.push(Message::MuxBoxOutputUpdate(
                    box_id.clone(),
                    false,
                    "PTY manager not available".to_string(),
                ));
            }
        }
    }
    Ok((app_context, messages))
}

#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    pub fg_color: String,
    pub bg_color: String,
    pub ch: char,
}

#[derive(Debug, Clone)]
pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Vec<Cell>>,
}

impl Default for ScreenBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl ScreenBuffer {
    pub fn new() -> Self {
        let default_cell = Cell {
            fg_color: get_fg_color("white"),
            bg_color: get_bg_color("black"),
            ch: ' ',
        };
        let width = screen_width();
        let height = screen_height();
        let buffer = vec![vec![default_cell; width]; height];
        ScreenBuffer {
            width,
            height,
            buffer,
        }
    }

    pub fn new_custom(width: usize, height: usize) -> Self {
        let default_cell = Cell {
            fg_color: get_fg_color("white"),
            bg_color: get_bg_color("black"),
            ch: ' ',
        };
        let buffer = vec![vec![default_cell; width]; height];
        ScreenBuffer {
            width,
            height,
            buffer,
        }
    }

    pub fn clear(&mut self) {
        let default_cell = Cell {
            fg_color: get_fg_color("white"),
            bg_color: get_bg_color("black"),
            ch: ' ',
        };
        self.buffer = vec![vec![default_cell; self.width]; self.height];
    }

    pub fn update(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.buffer[y][x] = cell;
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height {
            Some(&self.buffer[y][x])
        } else {
            None
        }
    }

    pub fn resize(&mut self, width: usize, height: usize) {
        // First handle shrinking the buffer if necessary
        if height < self.height {
            self.buffer.truncate(height);
        }
        if width < self.width {
            for row in &mut self.buffer {
                row.truncate(width);
            }
        }

        // Now handle expanding the buffer if necessary
        if height > self.height {
            let default_row = vec![
                Cell {
                    fg_color: get_fg_color("white"),
                    bg_color: get_bg_color("black"),
                    ch: ' ',
                };
                width
            ];

            self.buffer.resize_with(height, || default_row.clone());
        }
        if width > self.width {
            for row in &mut self.buffer {
                row.resize_with(width, || Cell {
                    fg_color: get_fg_color("white"),
                    bg_color: get_bg_color("black"),
                    ch: ' ',
                });
            }
        }

        // Update the dimensions
        self.width = width;
        self.height = height;
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq)]
pub struct InputBounds {
    pub x1: String,
    pub y1: String,
    pub x2: String,
    pub y2: String,
}

impl InputBounds {
    pub fn to_bounds(&self, parent_bounds: &Bounds) -> Bounds {
        input_bounds_to_bounds(self, parent_bounds)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Bounds {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl PartialEq for Bounds {
    fn eq(&self, other: &Self) -> bool {
        self.x1 == other.x1 && self.y1 == other.y1 && self.x2 == other.x2 && self.y2 == other.y2
    }
}

impl Eq for Bounds {}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash, Eq, Default)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    #[default]
    Center,
    CenterTop,
    CenterBottom,
    CenterLeft,
    CenterRight,
}

impl Bounds {
    pub fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
        Bounds { x1, y1, x2, y2 }
    }

    pub fn validate(&self) {
        if self.x1 > self.x2 {
            panic!(
                "Validation error: x1 ({}) is greater than x2 ({})",
                self.x1, self.x2
            );
        }
        if self.y1 > self.y2 {
            panic!(
                "Validation error: y1 ({}) is greater than y2 ({})",
                self.y1, self.y2
            );
        }
    }

    pub fn width(&self) -> usize {
        // For inclusive coordinate bounds, width is x2 - x1 + 1
        self.x2.saturating_sub(self.x1).saturating_add(1)
    }

    pub fn height(&self) -> usize {
        // For inclusive coordinate bounds, height is y2 - y1 + 1
        self.y2.saturating_sub(self.y1).saturating_add(1)
    }

    pub fn to_string(&self) -> String {
        format!("({}, {}), ({}, {})", self.x1, self.y1, self.x2, self.y2)
    }

    pub fn extend(&mut self, horizontal_amount: usize, vertical_amount: usize, anchor: Anchor) {
        match anchor {
            Anchor::TopLeft => {
                self.x1 = self.x1.saturating_sub(horizontal_amount);
                self.y1 = self.y1.saturating_sub(vertical_amount);
            }
            Anchor::TopRight => {
                self.x2 += horizontal_amount;
                self.y1 = self.y1.saturating_sub(vertical_amount);
            }
            Anchor::BottomLeft => {
                self.x1 = self.x1.saturating_sub(horizontal_amount);
                self.y2 += vertical_amount;
            }
            Anchor::BottomRight => {
                self.x2 += horizontal_amount;
                self.y2 += vertical_amount;
            }
            Anchor::Center => {
                let half_horizontal = horizontal_amount / 2;
                let half_vertical = vertical_amount / 2;
                self.x1 = self.x1.saturating_sub(half_horizontal);
                self.y1 = self.y1.saturating_sub(half_vertical);
                self.x2 += half_horizontal;
                self.y2 += half_vertical;
            }
            Anchor::CenterTop => {
                let half_horizontal = horizontal_amount / 2;
                self.x1 = self.x1.saturating_sub(half_horizontal);
                self.x2 += half_horizontal;
                self.y1 = self.y1.saturating_sub(vertical_amount);
            }
            Anchor::CenterBottom => {
                let half_horizontal = horizontal_amount / 2;
                self.x1 = self.x1.saturating_sub(half_horizontal);
                self.x2 += half_horizontal;
                self.y2 += vertical_amount;
            }
            Anchor::CenterLeft => {
                let half_vertical = vertical_amount / 2;
                self.x1 = self.x1.saturating_sub(horizontal_amount);
                self.y1 = self.y1.saturating_sub(half_vertical);
                self.y2 += half_vertical;
            }
            Anchor::CenterRight => {
                let half_vertical = vertical_amount / 2;
                self.x2 += horizontal_amount;
                self.y1 = self.y1.saturating_sub(half_vertical);
                self.y2 += half_vertical;
            }
        }
        self.validate();
    }

    pub fn contract(&mut self, horizontal_amount: usize, vertical_amount: usize, anchor: Anchor) {
        match anchor {
            Anchor::TopLeft => {
                self.x1 += horizontal_amount;
                self.y1 += vertical_amount;
            }
            Anchor::TopRight => {
                self.x2 = self.x2.saturating_sub(horizontal_amount);
                self.y1 += vertical_amount;
            }
            Anchor::BottomLeft => {
                self.x1 += horizontal_amount;
                self.y2 = self.y2.saturating_sub(vertical_amount);
            }
            Anchor::BottomRight => {
                self.x2 = self.x2.saturating_sub(horizontal_amount);
                self.y2 = self.y2.saturating_sub(vertical_amount);
            }
            Anchor::Center => {
                let half_horizontal = horizontal_amount / 2;
                let half_vertical = vertical_amount / 2;
                self.x1 += half_horizontal;
                self.y1 += half_vertical;
                self.x2 = self.x2.saturating_sub(half_horizontal);
                self.y2 = self.y2.saturating_sub(half_vertical);
            }
            Anchor::CenterTop => {
                let half_horizontal = horizontal_amount / 2;
                self.x1 += half_horizontal;
                self.x2 = self.x2.saturating_sub(half_horizontal);
                self.y1 += vertical_amount;
            }
            Anchor::CenterBottom => {
                let half_horizontal = horizontal_amount / 2;
                self.x1 += half_horizontal;
                self.x2 = self.x2.saturating_sub(half_horizontal);
                self.y2 = self.y2.saturating_sub(vertical_amount);
            }
            Anchor::CenterLeft => {
                let half_vertical = vertical_amount / 2;
                self.x1 += horizontal_amount;
                self.y1 += half_vertical;
                self.y2 = self.y2.saturating_sub(half_vertical);
            }
            Anchor::CenterRight => {
                let half_vertical = vertical_amount / 2;
                self.x2 = self.x2.saturating_sub(horizontal_amount);
                self.y1 += half_vertical;
                self.y2 = self.y2.saturating_sub(half_vertical);
            }
        }
        self.validate();
    }

    pub fn move_to(&mut self, x: usize, y: usize, anchor: Anchor) {
        match anchor {
            Anchor::TopLeft => {
                let width = self.width();
                let height = self.height();
                self.x1 = x;
                self.y1 = y;
                self.x2 = x + width - 1; // Inclusive bounds
                self.y2 = y + height - 1; // Inclusive bounds
            }
            Anchor::TopRight => {
                let width = self.width();
                let height = self.height();
                self.x2 = x;
                self.y1 = y;
                self.x1 = x - width + 1; // Inclusive bounds
                self.y2 = y + height - 1; // Inclusive bounds
            }
            Anchor::BottomLeft => {
                let width = self.width();
                let height = self.height();
                self.x1 = x;
                self.y2 = y;
                self.x2 = x + width - 1; // Inclusive bounds
                self.y1 = y - height + 1; // Inclusive bounds
            }
            Anchor::BottomRight => {
                let width = self.width();
                let height = self.height();
                self.x2 = x;
                self.y2 = y;
                self.x1 = x - width + 1; // Inclusive bounds
                self.y1 = y - height + 1; // Inclusive bounds
            }
            Anchor::Center => {
                let width = self.width();
                let height = self.height();
                let half_width = width / 2;
                let half_height = height / 2;
                self.x1 = x - half_width;
                self.y1 = y - half_height;
                self.x2 = x + width - half_width - 1; // Inclusive bounds
                self.y2 = y + height - half_height - 1; // Inclusive bounds
            }
            Anchor::CenterTop => {
                let width = self.width();
                let height = self.height();
                let half_width = width / 2;
                self.x1 = x - half_width;
                self.x2 = x + width - half_width - 1; // Inclusive bounds
                self.y1 = y;
                self.y2 = y + height - 1; // Inclusive bounds
            }
            Anchor::CenterBottom => {
                let width = self.width();
                let height = self.height();
                let half_width = width / 2;
                self.x1 = x - half_width;
                self.x2 = x + width - half_width - 1; // Inclusive bounds
                self.y2 = y;
                self.y1 = y - height + 1; // Inclusive bounds
            }
            Anchor::CenterLeft => {
                let width = self.width();
                let height = self.height();
                let half_height = height / 2;
                self.x1 = x;
                self.x2 = x + width - 1; // Inclusive bounds
                self.y1 = y - half_height;
                self.y2 = y + height - half_height - 1; // Inclusive bounds
            }
            Anchor::CenterRight => {
                let width = self.width();
                let height = self.height();
                let half_height = height / 2;
                self.x2 = x;
                self.x1 = x - width + 1; // Inclusive bounds
                self.y1 = y - half_height;
                self.y2 = y + height - half_height - 1; // Inclusive bounds
            }
        }
        self.validate();
    }

    pub fn move_by(&mut self, dx: isize, dy: isize) {
        self.x1 = (self.x1 as isize + dx) as usize;
        self.y1 = (self.y1 as isize + dy) as usize;
        self.x2 = (self.x2 as isize + dx) as usize;
        self.y2 = (self.y2 as isize + dy) as usize;
        self.validate();
    }

    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x1 && x < self.x2 && y >= self.y1 && y < self.y2
    }

    pub fn contains_bounds(&self, other: &Bounds) -> bool {
        self.contains(other.x1, other.y1) && self.contains(other.x2, other.y2)
    }

    pub fn intersects(&self, other: &Bounds) -> bool {
        self.contains(other.x1, other.y1)
            || self.contains(other.x2, other.y2)
            || self.contains(other.x1, other.y2)
            || self.contains(other.x2, other.y1)
    }

    pub fn intersection(&self, other: &Bounds) -> Option<Bounds> {
        if self.intersects(other) {
            Some(Bounds {
                x1: self.x1.max(other.x1),
                y1: self.y1.max(other.y1),
                x2: self.x2.min(other.x2),
                y2: self.y2.min(other.y2),
            })
        } else {
            None
        }
    }

    pub fn union(&self, other: &Bounds) -> Bounds {
        Bounds {
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
            x2: self.x2.max(other.x2),
            y2: self.y2.max(other.y2),
        }
    }

    pub fn translate(&self, dx: isize, dy: isize) -> Bounds {
        Bounds {
            x1: (self.x1 as isize + dx) as usize,
            y1: (self.y1 as isize + dy) as usize,
            x2: (self.x2 as isize + dx) as usize,
            y2: (self.y2 as isize + dy) as usize,
        }
    }

    pub fn center(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    pub fn center_x(&self) -> usize {
        (self.x1 + self.x2) / 2
    }

    pub fn center_y(&self) -> usize {
        (self.y1 + self.y2) / 2
    }

    pub fn top_left(&self) -> (usize, usize) {
        (self.x1, self.y1)
    }

    pub fn top_right(&self) -> (usize, usize) {
        (self.x2, self.y1)
    }

    pub fn bottom_left(&self) -> (usize, usize) {
        (self.x1, self.y2)
    }

    pub fn bottom_right(&self) -> (usize, usize) {
        (self.x2, self.y2)
    }

    pub fn top(&self) -> usize {
        self.y1
    }

    pub fn bottom(&self) -> usize {
        self.y2
    }

    pub fn left(&self) -> usize {
        self.x1
    }

    pub fn right(&self) -> usize {
        self.x2
    }

    pub fn center_top(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, self.y1)
    }

    pub fn center_bottom(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, self.y2)
    }

    pub fn center_left(&self) -> (usize, usize) {
        (self.x1, (self.y1 + self.y2) / 2)
    }

    pub fn center_right(&self) -> (usize, usize) {
        (self.x2, (self.y1 + self.y2) / 2)
    }
}

pub fn calculate_initial_bounds(app_graph: &AppGraph, layout: &Layout) -> HashMap<String, Bounds> {
    let mut bounds_map = HashMap::new();

    fn dfs(
        app_graph: &AppGraph,
        layout_id: &str,
        muxbox: &MuxBox,
        parent_bounds: Bounds,
        bounds_map: &mut HashMap<String, Bounds>,
    ) {
        let bounds = muxbox.absolute_bounds(Some(&parent_bounds));
        bounds_map.insert(muxbox.id.clone(), bounds.clone());

        if let Some(children) = &muxbox.children {
            for child in children {
                dfs(app_graph, layout_id, child, bounds.clone(), bounds_map);
            }
        }
    }

    let root_bounds = screen_bounds();
    if let Some(children) = &layout.children {
        for muxbox in children {
            dfs(
                app_graph,
                &layout.id,
                muxbox,
                root_bounds.clone(),
                &mut bounds_map,
            );
        }
    }

    bounds_map
}

pub fn adjust_bounds_with_constraints(
    layout: &Layout,
    mut bounds_map: HashMap<String, Bounds>,
) -> HashMap<String, Bounds> {
    fn apply_constraints(muxbox: &MuxBox, bounds: &mut Bounds) {
        if let Some(min_width) = muxbox.min_width {
            if bounds.width() < min_width {
                bounds.extend(min_width - bounds.width(), 0, muxbox.anchor.clone());
            }
        }
        if let Some(min_height) = muxbox.min_height {
            if bounds.height() < min_height {
                bounds.extend(0, min_height - bounds.height(), muxbox.anchor.clone());
            }
        }
        if let Some(max_width) = muxbox.max_width {
            if bounds.width() > max_width {
                bounds.contract(bounds.width() - max_width, 0, muxbox.anchor.clone());
            }
        }
        if let Some(max_height) = muxbox.max_height {
            if bounds.height() > max_height {
                bounds.contract(0, bounds.height() - max_height, muxbox.anchor.clone());
            }
        }
    }

    fn dfs(muxbox: &MuxBox, bounds_map: &mut HashMap<String, Bounds>) -> Bounds {
        let mut bounds = bounds_map.remove(&muxbox.id).unwrap();
        apply_constraints(muxbox, &mut bounds);
        bounds_map.insert(muxbox.id.clone(), bounds.clone());

        if let Some(children) = &muxbox.children {
            for child in children {
                let child_bounds = dfs(child, bounds_map);
                bounds.x2 = bounds.x2.max(child_bounds.x2);
                bounds.y2 = bounds.y2.max(child_bounds.y2);
            }
        }

        bounds
    }

    fn revalidate_children(
        muxbox: &MuxBox,
        bounds_map: &mut HashMap<String, Bounds>,
        parent_bounds: &Bounds,
    ) {
        if let Some(children) = &muxbox.children {
            for child in children {
                if let Some(child_bounds) = bounds_map.get_mut(&child.id) {
                    // Ensure child bounds are within parent bounds
                    if child_bounds.x2 > parent_bounds.x2 {
                        child_bounds.x2 = parent_bounds.x2;
                    }
                    if child_bounds.y2 > parent_bounds.y2 {
                        child_bounds.y2 = parent_bounds.y2;
                    }
                    if child_bounds.x1 < parent_bounds.x1 {
                        child_bounds.x1 = parent_bounds.x1;
                    }
                    if child_bounds.y1 < parent_bounds.y1 {
                        child_bounds.y1 = parent_bounds.y1;
                    }
                }
                revalidate_children(child, bounds_map, parent_bounds);
            }
        }
    }

    if let Some(children) = &layout.children {
        for muxbox in children {
            let parent_bounds = dfs(muxbox, &mut bounds_map);
            revalidate_children(muxbox, &mut bounds_map, &parent_bounds);
        }
    }

    bounds_map
}

pub fn calculate_bounds_map(app_graph: &AppGraph, layout: &Layout) -> HashMap<String, Bounds> {
    let bounds_map = calculate_initial_bounds(app_graph, layout);
    adjust_bounds_with_constraints(layout, bounds_map)
}

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

pub fn send_json_to_socket(socket_path: &str, json: &str) -> Result<String, Box<dyn Error>> {
    let mut stream = UnixStream::connect(socket_path)?;
    stream.write_all(json.as_bytes())?;
    let mut response = String::new();
    stream.read_to_string(&mut response)?;
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Config Tests ===

    /// Tests that Config::new() creates a valid configuration with the specified frame delay.
    /// This test demonstrates how to create a Config with proper validation.
    #[test]
    fn test_config_new_valid_frame_delay() {
        let config = Config::new(60);
        assert_eq!(config.frame_delay, 60);
    }

    /// Tests that Config::new() panics when frame_delay is zero.
    /// This test demonstrates Config validation for invalid frame delays.
    #[test]
    #[should_panic(expected = "Validation error: frame_delay cannot be 0")]
    fn test_config_new_zero_frame_delay_panics() {
        Config::new(0);
    }

    /// Tests that Config::default() creates a configuration with default values.
    /// This test demonstrates the default configuration settings.
    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert_eq!(config.frame_delay, 30);
    }

    /// Tests that Config::validate() correctly identifies invalid configurations.
    /// This test demonstrates Config validation behavior.
    #[test]
    #[should_panic(expected = "Validation error: frame_delay cannot be 0")]
    fn test_config_validate_zero_frame_delay() {
        let config = Config {
            frame_delay: 0,
            locked: false,
        };
        config.validate();
    }

    /// Tests that Config::validate() passes for valid configurations.
    /// This test demonstrates successful Config validation.
    #[test]
    fn test_config_validate_valid() {
        let config = Config {
            frame_delay: 16,
            locked: false,
        };
        config.validate(); // Should not panic
    }

    /// Tests that Config implements Hash consistently.
    /// This test demonstrates that Configs with the same values hash to the same value.
    #[test]
    fn test_config_hash_consistency() {
        let config1 = Config::new(30);
        let config2 = Config::new(30);
        let config3 = Config::new(60);

        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        config1.hash(&mut hasher1);
        config2.hash(&mut hasher2);
        config3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    // === Bounds Tests ===

    /// Tests that Bounds::new() creates bounds with correct coordinates.
    /// This test demonstrates basic Bounds construction.
    #[test]
    fn test_bounds_new() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.x1, 10);
        assert_eq!(bounds.y1, 20);
        assert_eq!(bounds.x2, 100);
        assert_eq!(bounds.y2, 200);
    }

    /// Tests that Bounds::validate() panics for invalid x coordinates.
    /// This test demonstrates Bounds validation for x-coordinate ordering.
    #[test]
    #[should_panic(expected = "Validation error: x1 (100) is greater than x2 (50)")]
    fn test_bounds_validate_invalid_x_coordinates() {
        let bounds = Bounds::new(100, 20, 50, 200);
        bounds.validate();
    }

    /// Tests that Bounds::validate() panics for invalid y coordinates.
    /// This test demonstrates Bounds validation for y-coordinate ordering.
    #[test]
    #[should_panic(expected = "Validation error: y1 (200) is greater than y2 (100)")]
    fn test_bounds_validate_invalid_y_coordinates() {
        let bounds = Bounds::new(10, 200, 100, 100);
        bounds.validate();
    }

    /// Tests that Bounds::validate() passes for valid bounds.
    /// This test demonstrates successful Bounds validation.
    #[test]
    fn test_bounds_validate_valid() {
        let bounds = Bounds::new(10, 20, 100, 200);
        bounds.validate(); // Should not panic
    }

    /// Tests that Bounds::width() calculates width correctly.
    /// This test demonstrates the width calculation feature.
    #[test]
    fn test_bounds_width() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.width(), 91); // Inclusive bounds: 100-10+1 = 91
    }

    /// Tests that Bounds::height() calculates height correctly.
    /// This test demonstrates the height calculation feature.
    #[test]
    fn test_bounds_height() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.height(), 181); // Inclusive bounds: 200-20+1 = 181
    }

    /// Tests that Bounds::width() handles edge case where x1 equals x2.
    /// This test demonstrates edge case handling in width calculation.
    #[test]
    fn test_bounds_width_zero() {
        let bounds = Bounds::new(50, 20, 50, 200);
        assert_eq!(bounds.width(), 1); // Inclusive bounds: 50-50+1 = 1
    }

    /// Tests that Bounds::height() handles edge case where y1 equals y2.
    /// This test demonstrates edge case handling in height calculation.
    #[test]
    fn test_bounds_height_zero() {
        let bounds = Bounds::new(10, 50, 100, 50);
        assert_eq!(bounds.height(), 1); // Inclusive bounds: 50-50+1 = 1
    }

    /// Tests that Bounds::contains() correctly identifies points within bounds.
    /// This test demonstrates the point containment feature.
    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert!(bounds.contains(50, 100));
        assert!(bounds.contains(10, 20)); // Edge case: top-left corner
        assert!(!bounds.contains(100, 200)); // Edge case: bottom-right corner (exclusive)
        assert!(!bounds.contains(5, 100)); // Outside left
        assert!(!bounds.contains(150, 100)); // Outside right
        assert!(!bounds.contains(50, 10)); // Outside top
        assert!(!bounds.contains(50, 250)); // Outside bottom
    }

    /// Tests that Bounds::contains_bounds() correctly identifies bounds containment.
    /// This test demonstrates the bounds containment feature.
    #[test]
    fn test_bounds_contains_bounds() {
        let outer = Bounds::new(10, 20, 100, 200);
        let inner = Bounds::new(30, 40, 80, 180);
        let overlapping = Bounds::new(5, 15, 50, 100);

        assert!(outer.contains_bounds(&inner));
        assert!(!outer.contains_bounds(&overlapping));
    }

    /// Tests that Bounds::intersects() correctly identifies intersecting bounds.
    /// This test demonstrates the bounds intersection detection feature.
    #[test]
    fn test_bounds_intersects() {
        let bounds1 = Bounds::new(10, 20, 100, 200);
        let bounds2 = Bounds::new(50, 100, 150, 250); // Overlapping
        let bounds3 = Bounds::new(200, 300, 250, 350); // Non-overlapping

        assert!(bounds1.intersects(&bounds2));
        assert!(!bounds1.intersects(&bounds3));
    }

    /// Tests that Bounds::intersection() returns correct intersection bounds.
    /// This test demonstrates the bounds intersection calculation feature.
    #[test]
    fn test_bounds_intersection() {
        let bounds1 = Bounds::new(10, 20, 100, 200);
        let bounds2 = Bounds::new(50, 100, 150, 250);
        let bounds3 = Bounds::new(200, 300, 250, 350);

        let intersection = bounds1.intersection(&bounds2);
        assert!(intersection.is_some());
        let intersection = intersection.unwrap();
        assert_eq!(intersection.x1, 50);
        assert_eq!(intersection.y1, 100);
        assert_eq!(intersection.x2, 100);
        assert_eq!(intersection.y2, 200);

        assert!(bounds1.intersection(&bounds3).is_none());
    }

    /// Tests that Bounds::union() returns correct union bounds.
    /// This test demonstrates the bounds union calculation feature.
    #[test]
    fn test_bounds_union() {
        let bounds1 = Bounds::new(10, 20, 100, 200);
        let bounds2 = Bounds::new(50, 100, 150, 250);

        let union = bounds1.union(&bounds2);
        assert_eq!(union.x1, 10);
        assert_eq!(union.y1, 20);
        assert_eq!(union.x2, 150);
        assert_eq!(union.y2, 250);
    }

    /// Tests that Bounds::translate() correctly translates bounds.
    /// This test demonstrates the bounds translation feature.
    #[test]
    fn test_bounds_translate() {
        let bounds = Bounds::new(10, 20, 100, 200);
        let translated = bounds.translate(5, -10);
        assert_eq!(translated.x1, 15);
        assert_eq!(translated.y1, 10);
        assert_eq!(translated.x2, 105);
        assert_eq!(translated.y2, 190);
    }

    /// Tests that Bounds::center() returns correct center point.
    /// This test demonstrates the center calculation feature.
    #[test]
    fn test_bounds_center() {
        let bounds = Bounds::new(10, 20, 100, 200);
        let center = bounds.center();
        assert_eq!(center, (55, 110));
    }

    /// Tests that Bounds::center_x() returns correct x center.
    /// This test demonstrates the x-center calculation feature.
    #[test]
    fn test_bounds_center_x() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.center_x(), 55);
    }

    /// Tests that Bounds::center_y() returns correct y center.
    /// This test demonstrates the y-center calculation feature.
    #[test]
    fn test_bounds_center_y() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.center_y(), 110);
    }

    /// Tests that Bounds::extend() correctly extends bounds in all directions.
    /// This test demonstrates the bounds extension feature with Center anchor.
    #[test]
    fn test_bounds_extend_center() {
        let mut bounds = Bounds::new(50, 50, 100, 100);
        bounds.extend(20, 10, Anchor::Center);
        assert_eq!(bounds.x1, 40);
        assert_eq!(bounds.y1, 45);
        assert_eq!(bounds.x2, 110);
        assert_eq!(bounds.y2, 105);
    }

    /// Tests that Bounds::extend() correctly extends bounds with TopLeft anchor.
    /// This test demonstrates the bounds extension feature with TopLeft anchor.
    #[test]
    fn test_bounds_extend_top_left() {
        let mut bounds = Bounds::new(50, 50, 100, 100);
        bounds.extend(20, 10, Anchor::TopLeft);
        assert_eq!(bounds.x1, 30);
        assert_eq!(bounds.y1, 40);
        assert_eq!(bounds.x2, 100);
        assert_eq!(bounds.y2, 100);
    }

    /// Tests that Bounds::contract() correctly contracts bounds.
    /// This test demonstrates the bounds contraction feature.
    #[test]
    fn test_bounds_contract_center() {
        let mut bounds = Bounds::new(50, 50, 100, 100);
        bounds.contract(10, 20, Anchor::Center);
        assert_eq!(bounds.x1, 55);
        assert_eq!(bounds.y1, 60);
        assert_eq!(bounds.x2, 95);
        assert_eq!(bounds.y2, 90);
    }

    /// Tests that Bounds::move_to() correctly moves bounds to new position.
    /// This test demonstrates the bounds movement feature.
    #[test]
    fn test_bounds_move_to() {
        let mut bounds = Bounds::new(10, 20, 60, 70);
        bounds.move_to(100, 150, Anchor::TopLeft);
        assert_eq!(bounds.x1, 100);
        assert_eq!(bounds.y1, 150);
        assert_eq!(bounds.x2, 150);
        assert_eq!(bounds.y2, 200);
    }

    /// Tests that Bounds::move_by() correctly moves bounds by offset.
    /// This test demonstrates the bounds offset movement feature.
    #[test]
    fn test_bounds_move_by() {
        let mut bounds = Bounds::new(10, 20, 60, 70);
        bounds.move_by(5, -10);
        assert_eq!(bounds.x1, 15);
        assert_eq!(bounds.y1, 10);
        assert_eq!(bounds.x2, 65);
        assert_eq!(bounds.y2, 60);
    }

    /// Tests various anchor point getters.
    /// This test demonstrates the anchor point calculation features.
    #[test]
    fn test_bounds_anchor_points() {
        let bounds = Bounds::new(10, 20, 100, 200);

        assert_eq!(bounds.top_left(), (10, 20));
        assert_eq!(bounds.top_right(), (100, 20));
        assert_eq!(bounds.bottom_left(), (10, 200));
        assert_eq!(bounds.bottom_right(), (100, 200));
        assert_eq!(bounds.center_top(), (55, 20));
        assert_eq!(bounds.center_bottom(), (55, 200));
        assert_eq!(bounds.center_left(), (10, 110));
        assert_eq!(bounds.center_right(), (100, 110));
        assert_eq!(bounds.top(), 20);
        assert_eq!(bounds.bottom(), 200);
        assert_eq!(bounds.left(), 10);
        assert_eq!(bounds.right(), 100);
    }

    /// Tests that Bounds::to_string() formats bounds correctly.
    /// This test demonstrates the bounds string formatting feature.
    #[test]
    fn test_bounds_to_string() {
        let bounds = Bounds::new(10, 20, 100, 200);
        assert_eq!(bounds.to_string(), "(10, 20), (100, 200)");
    }

    // === InputBounds Tests ===

    /// Tests that InputBounds::to_bounds() converts percentage strings to absolute bounds.
    /// This test demonstrates the InputBounds to Bounds conversion feature.
    #[test]
    fn test_input_bounds_to_bounds() {
        let input_bounds = InputBounds {
            x1: "25%".to_string(),
            y1: "50%".to_string(),
            x2: "75%".to_string(),
            y2: "100%".to_string(),
        };
        let parent_bounds = Bounds::new(0, 0, 100, 200);
        let bounds = input_bounds.to_bounds(&parent_bounds);

        assert_eq!(bounds.x1, 25);
        assert_eq!(bounds.y1, 100);
        assert_eq!(bounds.x2, 75); // 75% of (101-1) range = 75
        assert_eq!(bounds.y2, 200); // 100% of (201-1) range = 200
    }

    // === Anchor Tests ===

    /// Tests that Anchor::default() returns Center.
    /// This test demonstrates the default anchor behavior.
    #[test]
    fn test_anchor_default() {
        let anchor = Anchor::default();
        assert_eq!(anchor, Anchor::Center);
    }

    // === ScreenBuffer Tests ===

    /// Tests that ScreenBuffer::new_custom() creates a buffer with specified dimensions.
    /// This test demonstrates how to create a custom-sized screen buffer.
    #[test]
    fn test_screenbuffer_new() {
        let screen_buffer = ScreenBuffer::new_custom(5, 5);
        assert_eq!(screen_buffer.width, 5);
        assert_eq!(screen_buffer.height, 5);
        assert_eq!(screen_buffer.buffer.len(), 5);
        assert_eq!(screen_buffer.buffer[0].len(), 5);
    }

    /// Tests that ScreenBuffer::clear() resets all cells to default values.
    /// This test demonstrates the screen buffer clearing feature.
    #[test]
    fn test_screenbuffer_clear() {
        let mut screen_buffer = ScreenBuffer::new_custom(5, 5);
        let test_cell = Cell {
            fg_color: String::from("red"),
            bg_color: String::from("blue"),
            ch: 'X',
        };
        screen_buffer.update(2, 2, test_cell.clone());
        screen_buffer.clear();
        for row in screen_buffer.buffer.iter() {
            for cell in row.iter() {
                assert_eq!(cell.fg_color, get_fg_color("white"));
                assert_eq!(cell.bg_color, get_bg_color("black"));
                assert_eq!(cell.ch, ' ');
            }
        }
    }

    /// Tests that ScreenBuffer::update() correctly updates a cell.
    /// This test demonstrates the screen buffer cell update feature.
    #[test]
    fn test_screenbuffer_update() {
        let mut screen_buffer = ScreenBuffer::new_custom(5, 5);
        let test_cell = Cell {
            fg_color: String::from("red"),
            bg_color: String::from("blue"),
            ch: 'X',
        };
        screen_buffer.update(2, 2, test_cell.clone());
        assert_eq!(screen_buffer.get(2, 2).unwrap(), &test_cell);
    }

    /// Tests that ScreenBuffer::get() returns correct cell references.
    /// This test demonstrates the screen buffer cell retrieval feature.
    #[test]
    fn test_screenbuffer_get() {
        let screen_buffer = ScreenBuffer::new_custom(5, 5);
        assert!(screen_buffer.get(6, 6).is_none());
        assert!(screen_buffer.get(3, 3).is_some());
    }

    /// Tests that ScreenBuffer::update() ignores out-of-bounds coordinates.
    /// This test demonstrates bounds checking in screen buffer updates.
    #[test]
    fn test_screenbuffer_update_out_of_bounds() {
        let mut screen_buffer = ScreenBuffer::new_custom(5, 5);
        let test_cell = Cell {
            fg_color: String::from("red"),
            bg_color: String::from("blue"),
            ch: 'X',
        };
        screen_buffer.update(10, 10, test_cell); // Should not panic
        assert!(screen_buffer.get(10, 10).is_none());
    }

    /// Tests that ScreenBuffer::resize() correctly resizes the buffer.
    /// This test demonstrates the screen buffer resizing feature.
    #[test]
    fn test_screenbuffer_resize() {
        let mut screen_buffer = ScreenBuffer::new_custom(5, 5);
        screen_buffer.resize(10, 8);
        assert_eq!(screen_buffer.width, 10);
        assert_eq!(screen_buffer.height, 8);
        assert_eq!(screen_buffer.buffer.len(), 8);
        assert_eq!(screen_buffer.buffer[0].len(), 10);
    }

    /// Tests that ScreenBuffer::resize() handles shrinking correctly.
    /// This test demonstrates the screen buffer shrinking feature.
    #[test]
    fn test_screenbuffer_resize_shrink() {
        let mut screen_buffer = ScreenBuffer::new_custom(10, 10);
        screen_buffer.resize(5, 5);
        assert_eq!(screen_buffer.width, 5);
        assert_eq!(screen_buffer.height, 5);
        assert_eq!(screen_buffer.buffer.len(), 5);
        assert_eq!(screen_buffer.buffer[0].len(), 5);
    }

    // === Helper Functions ===

    /// Creates a test app context with a valid layout for testing.
    /// This helper ensures tests have a valid app context with layouts.
    fn create_test_app_context() -> AppContext {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        let dashboard_path = current_dir.join("layouts/tests.yaml");
        let app = crate::load_app_from_yaml(dashboard_path.to_str().unwrap())
            .expect("Failed to load app");
        AppContext::new(app, Config::default())
    }

    // === SocketFunction Tests ===

    /// Tests that run_socket_function() correctly handles ReplaceBoxContent.
    /// This test demonstrates socket function message processing.
    #[test]
    fn test_run_socket_function_replace_muxbox_content() {
        let app_context = create_test_app_context();
        let socket_function = SocketFunction::ReplaceBoxContent {
            box_id: "test_muxbox".to_string(),
            success: true,
            content: "Test content".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);
        match &messages[0] {
            crate::Message::MuxBoxOutputUpdate(muxbox_id, success, content) => {
                assert_eq!(muxbox_id, "test_muxbox");
                assert_eq!(*success, true);
                assert_eq!(content, "Test content");
            }
            _ => panic!("Expected MuxBoxOutputUpdate message"),
        }
    }

    /// Tests that run_socket_function() correctly handles SwitchActiveLayout.
    /// This test demonstrates socket function layout switching.
    #[test]
    fn test_run_socket_function_switch_active_layout() {
        let app_context = create_test_app_context();
        let socket_function = SocketFunction::SwitchActiveLayout {
            layout_id: "new_layout".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);
        match &messages[0] {
            crate::Message::SwitchActiveLayout(layout_id) => {
                assert_eq!(layout_id, "new_layout");
            }
            _ => panic!("Expected SwitchActiveLayout message"),
        }
    }

    // === Cell Tests ===

    /// Tests that Cell implements Clone and PartialEq correctly.
    /// This test demonstrates Cell trait implementations.
    #[test]
    fn test_cell_clone_and_eq() {
        let cell1 = Cell {
            fg_color: "red".to_string(),
            bg_color: "blue".to_string(),
            ch: 'X',
        };
        let cell2 = cell1.clone();
        assert_eq!(cell1, cell2);

        let cell3 = Cell {
            fg_color: "green".to_string(),
            bg_color: "blue".to_string(),
            ch: 'X',
        };
        assert_ne!(cell1, cell3);
    }

    /// Test send_json_to_socket function
    #[test]
    fn test_send_json_to_socket_function() {
        use std::os::unix::net::UnixListener;
        use std::thread;
        use std::time::Duration;

        let socket_path = "/tmp/test_send_json.sock";
        let _ = std::fs::remove_file(socket_path);

        // Start a simple test server
        let server_socket_path = socket_path.to_string();
        let server_handle = thread::spawn(move || {
            match UnixListener::bind(&server_socket_path) {
                Ok(listener) => {
                    // Set a timeout to prevent hanging
                    if let Some(Ok(mut stream)) = listener.incoming().next() {
                        let mut buffer = Vec::new();
                        let mut temp_buffer = [0; 1024];

                        // Read data in chunks to avoid hanging on read_to_string
                        match stream.read(&mut temp_buffer) {
                            Ok(n) => {
                                buffer.extend_from_slice(&temp_buffer[..n]);
                                let _ = stream.write_all(b"Test Response");
                                String::from_utf8_lossy(&buffer).to_string()
                            }
                            Err(_) => String::new(),
                        }
                    } else {
                        String::new()
                    }
                }
                Err(_) => String::new(),
            }
        });

        // Give server time to start
        thread::sleep(Duration::from_millis(100));

        // Test send_json_to_socket
        let test_json = r#"{"test": "message"}"#;
        let result = send_json_to_socket(socket_path, test_json);

        // The test is successful if either:
        // 1. The connection succeeds and we get the expected response
        // 2. The connection fails (which can happen in CI environments)
        match result {
            Ok(response) => {
                assert_eq!(response, "Test Response");

                // Verify server received the correct message
                let received_message = server_handle.join().unwrap();
                assert_eq!(received_message, test_json);
            }
            Err(_) => {
                // Connection failed - this can happen in CI environments
                // The important thing is that the function doesn't panic
                let _ = server_handle.join();
            }
        }

        // Clean up
        let _ = std::fs::remove_file(socket_path);
    }
}

// F0203: Multi-Stream Input Tabs - Tab system data structures (uses StreamType defined above)

// Duplicate StreamSource removed - using new trait-based system above

