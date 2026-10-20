//! The control socket protocol: a client sends one JSON request on one line and reads one JSON
//! reply on one line, then the connection closes.
//!
//! Request: `{"command": "replace_box_content", "box_id": "logs", "content": "..."}`
//! Reply:   `{"ok": true, "result": ...}` or `{"ok": false, "error": "..."}`

use crate::MuxBox;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case", deny_unknown_fields)]
pub enum SocketFunction {
    /// Shows `content` in the box's socket tab. `success: false` shows it in error colors.
    ReplaceBoxContent {
        box_id: String,
        content: String,
        #[serde(default = "default_true")]
        success: bool,
    },
    /// Replaces the box's script and runs it now.
    ReplaceBoxScript { box_id: String, script: Vec<String> },
    /// Pauses the periodic refresh of the box's script.
    StopBoxRefresh { box_id: String },
    /// Resumes the periodic refresh of the box's script, running it now.
    StartBoxRefresh { box_id: String },
    /// Replaces the configuration of a box. `new_box.id` must equal `box_id`.
    ReplaceBox {
        box_id: String,
        new_box: Box<MuxBox>,
    },
    /// Adds a box to a layout, at the top level or inside `parent_id`.
    AddBox {
        layout_id: String,
        #[serde(default)]
        parent_id: Option<String>,
        muxbox: Box<MuxBox>,
    },
    /// Removes a box and stops everything running for it.
    RemoveBox { box_id: String },
    /// Shows another layout.
    SwitchActiveLayout { layout_id: String },
    /// Starts `script` in a PTY whose screen is shown in `box_id` (or `redirect_output`).
    SpawnPtyProcess {
        box_id: String,
        script: Vec<String>,
        #[serde(default)]
        redirect_output: Option<String>,
    },
    /// Stops the process shown in the box's visible tab.
    KillPtyProcess { box_id: String },
    /// Stops and starts again the process shown in the box's visible tab.
    RestartPtyProcess { box_id: String },
    /// Reports on the process shown in the box's visible tab.
    QueryPtyStatus { box_id: String },
    /// Types `input` into the PTY process shown in the box's visible tab.
    SendPtyInput { box_id: String, input: String },
    /// Sends a Unix signal (by number) to the process shown in the box's visible tab.
    SendSignal { box_id: String, signal: i32 },
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reply {
    pub ok: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl Reply {
    pub fn ok() -> Self {
        Reply {
            ok: true,
            result: None,
            error: None,
        }
    }

    pub fn with_result(result: serde_json::Value) -> Self {
        Reply {
            ok: true,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Reply {
            ok: false,
            result: None,
            error: Some(message.into()),
        }
    }
}

/// Status of a process, as returned by `query_pty_status`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProcessStatus {
    pub box_id: String,
    pub stream: String,
    pub running: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pid: Option<u32>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rows: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cols: Option<u16>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_use_snake_case_command_names() {
        let request: SocketFunction =
            serde_json::from_str(r#"{"command":"switch_active_layout","layout_id":"x"}"#).unwrap();
        assert_eq!(
            request,
            SocketFunction::SwitchActiveLayout {
                layout_id: "x".into()
            }
        );
    }

    #[test]
    fn unknown_commands_and_fields_are_rejected() {
        assert!(serde_json::from_str::<SocketFunction>(r#"{"command":"explode"}"#).is_err());
        assert!(serde_json::from_str::<SocketFunction>(
            r#"{"command":"remove_box","box_id":"a","force":true}"#
        )
        .is_err());
    }

    #[test]
    fn replace_box_content_defaults_to_success() {
        let request: SocketFunction = serde_json::from_str(
            r#"{"command":"replace_box_content","box_id":"a","content":"hi"}"#,
        )
        .unwrap();
        assert!(matches!(
            request,
            SocketFunction::ReplaceBoxContent { success: true, .. }
        ));
    }
}
