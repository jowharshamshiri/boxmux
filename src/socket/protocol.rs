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
        new_box: BoxDefinition,
    },
    /// Adds a box to a layout, at the top level or inside `parent_id`.
    AddBox {
        layout_id: String,
        #[serde(default)]
        parent_id: Option<String>,
        muxbox: BoxDefinition,
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

/// A box sent in a request, kept as the JSON the sender wrote. Reading one checks it against
/// the schema like a box in a configuration file, so a missing `position` or a misspelled
/// field is an error rather than a default.
#[derive(Debug, Clone, PartialEq)]
pub struct BoxDefinition {
    json: serde_json::Value,
    muxbox: Box<MuxBox>,
}

impl BoxDefinition {
    pub fn into_muxbox(self) -> MuxBox {
        *self.muxbox
    }
}

impl std::str::FromStr for BoxDefinition {
    type Err = String;

    fn from_str(json: &str) -> Result<Self, String> {
        let value: serde_json::Value = serde_json::from_str(json).map_err(|e| e.to_string())?;
        BoxDefinition::try_from(value)
    }
}

impl TryFrom<serde_json::Value> for BoxDefinition {
    type Error = String;

    fn try_from(json: serde_json::Value) -> Result<Self, String> {
        crate::validation::validate_box(&json).map_err(|errors| {
            errors
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("; ")
        })?;
        let muxbox = serde_json::from_value(json.clone()).map_err(|e| e.to_string())?;
        Ok(BoxDefinition {
            json,
            muxbox: Box::new(muxbox),
        })
    }
}

impl Serialize for BoxDefinition {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.json.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for BoxDefinition {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let json = serde_json::Value::deserialize(deserializer)?;
        BoxDefinition::try_from(json).map_err(serde::de::Error::custom)
    }
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
    fn boxes_in_requests_are_checked_like_configuration_boxes() {
        let request = |muxbox: &str| {
            serde_json::from_str::<SocketFunction>(&format!(
                r#"{{"command":"add_box","layout_id":"l","muxbox":{}}}"#,
                muxbox
            ))
        };
        let missing_position = request(r#"{"id": "b"}"#).unwrap_err().to_string();
        assert!(
            missing_position.contains("position"),
            "{}",
            missing_position
        );
        let misspelled = request(
            r#"{"id": "b", "position": {"x1": "0", "y1": "0", "x2": "9", "y2": "9"}, "titel": "x"}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(misspelled.contains("titel"), "{}", misspelled);
        let nested = request(
            r#"{"id": "b", "position": {"x1": "0", "y1": "0", "x2": "9", "y2": "9"},
                "children": [{"id": "c", "position": {"x1": "0", "y1": "0", "x2": "9", "y2": "9"}, "border_color": "pink"}]}"#,
        )
        .unwrap_err()
        .to_string();
        assert!(
            nested.contains("children[0] ('c').border_color"),
            "{}",
            nested
        );
        // A valid box is sent on as written: unset fields stay unset rather than null.
        let valid =
            request(r#"{"id": "b", "position": {"x1": "0", "y1": "0", "x2": "9", "y2": "9"}}"#)
                .unwrap();
        let sent = serde_json::to_value(&valid).unwrap();
        assert_eq!(sent["muxbox"].as_object().unwrap().len(), 2);
    }

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

    /// The published protocol schema, `api-spec.json` at the repository root.
    fn api_spec() -> serde_json::Value {
        serde_json::from_str(include_str!("../../api-spec.json"))
            .expect("api-spec.json must be valid JSON")
    }

    /// The command name of a request. The match lists every variant, so adding one fails to
    /// compile here until it is also added to `samples` (and then to api-spec.json).
    fn command_name(function: &SocketFunction) -> &'static str {
        match function {
            SocketFunction::ReplaceBoxContent { .. } => "replace_box_content",
            SocketFunction::ReplaceBoxScript { .. } => "replace_box_script",
            SocketFunction::StopBoxRefresh { .. } => "stop_box_refresh",
            SocketFunction::StartBoxRefresh { .. } => "start_box_refresh",
            SocketFunction::ReplaceBox { .. } => "replace_box",
            SocketFunction::AddBox { .. } => "add_box",
            SocketFunction::RemoveBox { .. } => "remove_box",
            SocketFunction::SwitchActiveLayout { .. } => "switch_active_layout",
            SocketFunction::SpawnPtyProcess { .. } => "spawn_pty_process",
            SocketFunction::KillPtyProcess { .. } => "kill_pty_process",
            SocketFunction::RestartPtyProcess { .. } => "restart_pty_process",
            SocketFunction::QueryPtyStatus { .. } => "query_pty_status",
            SocketFunction::SendPtyInput { .. } => "send_pty_input",
            SocketFunction::SendSignal { .. } => "send_signal",
        }
    }

    /// One request of every command, with every optional field set.
    fn samples() -> Vec<SocketFunction> {
        let muxbox = || {
            r#"{"id": "b", "position": {"x1": "0", "y1": "0", "x2": "10", "y2": "5"}}"#
                .parse::<BoxDefinition>()
                .unwrap()
        };
        let id = || "b".to_string();
        vec![
            SocketFunction::ReplaceBoxContent {
                box_id: id(),
                content: "text".into(),
                success: false,
            },
            SocketFunction::ReplaceBoxScript {
                box_id: id(),
                script: vec!["date".into()],
            },
            SocketFunction::StopBoxRefresh { box_id: id() },
            SocketFunction::StartBoxRefresh { box_id: id() },
            SocketFunction::ReplaceBox {
                box_id: id(),
                new_box: muxbox(),
            },
            SocketFunction::AddBox {
                layout_id: "l".into(),
                parent_id: Some("p".into()),
                muxbox: muxbox(),
            },
            SocketFunction::RemoveBox { box_id: id() },
            SocketFunction::SwitchActiveLayout {
                layout_id: "l".into(),
            },
            SocketFunction::SpawnPtyProcess {
                box_id: id(),
                script: vec!["top".into()],
                redirect_output: Some("o".into()),
            },
            SocketFunction::KillPtyProcess { box_id: id() },
            SocketFunction::RestartPtyProcess { box_id: id() },
            SocketFunction::QueryPtyStatus { box_id: id() },
            SocketFunction::SendPtyInput {
                box_id: id(),
                input: "q".into(),
            },
            SocketFunction::SendSignal {
                box_id: id(),
                signal: 2,
            },
        ]
    }

    /// The command names serde accepts, read from its error for an unknown command. This
    /// catches a variant that `samples` leaves out.
    fn accepted_command_names() -> std::collections::BTreeSet<String> {
        let error = serde_json::from_str::<SocketFunction>(r#"{"command":"?"}"#)
            .expect_err("'?' is not a command")
            .to_string();
        let list = error
            .split("expected one of ")
            .nth(1)
            .unwrap_or_else(|| panic!("unexpected serde error format: {}", error));
        list.split(", ")
            .map(|name| {
                name.trim_start_matches('`')
                    .split('`')
                    .next()
                    .expect("split yields at least one part")
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn api_spec_lists_exactly_the_commands_and_fields_of_the_protocol() {
        use std::collections::{BTreeMap, BTreeSet};
        let spec = api_spec();

        let mut expected: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for sample in samples() {
            let value = serde_json::to_value(&sample).unwrap();
            let object = value.as_object().expect("requests serialize as objects");
            assert_eq!(object["command"], command_name(&sample));
            let fields = object.keys().filter(|k| *k != "command").cloned().collect();
            let previous = expected.insert(command_name(&sample).to_string(), fields);
            assert!(
                previous.is_none(),
                "two samples of {}",
                command_name(&sample)
            );
        }
        assert_eq!(
            expected.keys().cloned().collect::<BTreeSet<_>>(),
            accepted_command_names(),
            "samples() must contain one request of every command"
        );

        let mut documented: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for reference in spec["oneOf"].as_array().expect("the root is a oneOf") {
            let pointer = reference["$ref"]
                .as_str()
                .expect("each alternative is a $ref")
                .trim_start_matches('#');
            let definition = spec
                .pointer(pointer)
                .unwrap_or_else(|| panic!("{} is not defined", pointer));
            let properties = definition["properties"]
                .as_object()
                .unwrap_or_else(|| panic!("{} has no properties", pointer));
            let name = properties["command"]["const"]
                .as_str()
                .unwrap_or_else(|| panic!("{} has no command const", pointer))
                .to_string();
            let fields = properties
                .keys()
                .filter(|k| *k != "command")
                .cloned()
                .collect();
            assert_eq!(definition["additionalProperties"], false, "{}", name);
            documented.insert(name, fields);
        }
        assert_eq!(documented, expected);
    }

    #[test]
    fn api_spec_examples_are_valid_requests() {
        let spec = api_spec();
        let schema = jsonschema::validator_for(&spec).expect("api-spec.json must compile");
        let examples = spec["examples"]
            .as_array()
            .expect("api-spec.json has examples");
        let mut covered = std::collections::BTreeSet::new();
        for example in examples {
            let request: SocketFunction = serde_json::from_value(example.clone())
                .unwrap_or_else(|e| panic!("example {} is not a request: {}", example, e));
            covered.insert(command_name(&request));
            let errors: Vec<String> = schema.iter_errors(example).map(|e| e.to_string()).collect();
            assert!(
                errors.is_empty(),
                "example {} fails the schema: {:?}",
                example,
                errors
            );
        }
        for sample in samples() {
            let value = serde_json::to_value(&sample).unwrap();
            assert!(
                covered.contains(command_name(&sample)),
                "no example of {}",
                command_name(&sample)
            );
            assert!(schema.is_valid(&value), "{} fails the schema", value);
        }
    }

    #[test]
    fn api_spec_describes_the_replies() {
        let spec = api_spec();
        let reply_schema = serde_json::json!({
            "$ref": "#/definitions/reply",
            "definitions": spec["definitions"].clone(),
        });
        let schema = jsonschema::validator_for(&reply_schema).expect("reply schema compiles");
        let status = ProcessStatus {
            box_id: "b".into(),
            stream: "Terminal".into(),
            running: true,
            pid: Some(42),
            status: "running".into(),
            rows: Some(24),
            cols: Some(80),
        };
        let replies = [
            Reply::ok(),
            Reply::error("there is no box 'x'"),
            Reply::with_result(serde_json::to_value(status).unwrap()),
            Reply::with_result(serde_json::json!({ "run": 7 })),
        ];
        for reply in replies {
            let value = serde_json::to_value(&reply).unwrap();
            assert!(schema.is_valid(&value), "{} fails the reply schema", value);
        }
    }
}
