//! Streams are the tabs of a box. Each stream holds either text lines or a choice menu, and
//! knows whether a run (a script or PTY process) is currently feeding it.

use crate::model::choice::Choice;
use std::fmt;

/// Identifies one execution of a script or PTY session, unique for the lifetime of the process.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RunId(pub u64);

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "run-{}", self.0)
    }
}

/// What a run was started for. It decides the stream the output lands in, so starting the same
/// origin again reuses its tab instead of opening a new one.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunOrigin {
    /// The box's own `script`, run once or every `refresh_interval`.
    BoxScript { box_id: String },
    /// A choice of `box_id`, activated with Enter, a click or a hotkey.
    Choice { box_id: String, choice_id: String },
    /// An `on_keypress` handler of `box_id` for `key`.
    Keypress { box_id: String, key: String },
    /// A process started with the socket API's `spawn_pty_process` for `box_id`.
    SocketSpawn { box_id: String },
    /// An app- or layout-level `on_keypress` handler. Its output is not shown in any box; a
    /// failure is reported as a notice.
    GlobalKeypress { scope: String, key: String },
}

impl RunOrigin {
    /// The box whose configuration started the run; `None` for app- and layout-level runs.
    pub fn source_box(&self) -> Option<&str> {
        match self {
            RunOrigin::BoxScript { box_id }
            | RunOrigin::Choice { box_id, .. }
            | RunOrigin::Keypress { box_id, .. }
            | RunOrigin::SocketSpawn { box_id } => Some(box_id),
            RunOrigin::GlobalKeypress { .. } => None,
        }
    }

    /// The id of the stream, inside `target_box`, that this origin's output goes to.
    pub fn stream_id(&self, target_box: &str) -> String {
        match self {
            RunOrigin::BoxScript { box_id } if box_id == target_box => {
                format!("{}/script", target_box)
            }
            RunOrigin::BoxScript { box_id } => format!("{}/script:{}", target_box, box_id),
            RunOrigin::Choice { box_id, choice_id } => {
                format!("{}/choice:{}:{}", target_box, box_id, choice_id)
            }
            RunOrigin::Keypress { box_id, key } => {
                format!("{}/key:{}:{}", target_box, box_id, key)
            }
            RunOrigin::SocketSpawn { box_id } => format!("{}/spawn:{}", target_box, box_id),
            RunOrigin::GlobalKeypress { scope, key } => {
                format!("{}/global:{}:{}", target_box, scope, key)
            }
        }
    }
}

/// The source of a stream's content.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamKind {
    /// The box's static `content` from the configuration.
    Content,
    /// The box's `choices` menu.
    Choices,
    /// Output of runs of one origin.
    Run(RunOrigin),
    /// Content pushed through the socket API with `replace_box_content`.
    Socket,
}

/// Lifecycle of the content of a stream.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamStatus {
    /// Static content, or a run that has not started yet.
    Idle,
    /// A run is producing output.
    Running,
    /// The last run exited with this code.
    Exited(i32),
    /// The last run could not be started or was ended by a signal.
    Failed(String),
    /// The last run was killed by the user (closed tab, socket `kill_pty_process`).
    Killed,
}

impl StreamStatus {
    /// Whether the content reflects a failure the box should show in its error colors.
    pub fn is_error(&self) -> bool {
        match self {
            StreamStatus::Exited(code) => *code != 0,
            StreamStatus::Failed(_) => true,
            StreamStatus::Idle | StreamStatus::Running | StreamStatus::Killed => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stream {
    pub id: String,
    pub kind: StreamKind,
    pub label: String,
    /// Text lines, possibly containing SGR escape sequences. Empty for choice streams.
    pub content: Vec<String>,
    /// The menu of a `Choices` stream; `None` for every other kind.
    pub choices: Option<Vec<Choice>>,
    pub status: StreamStatus,
    /// The run currently feeding this stream.
    pub run: Option<RunId>,
    /// Whether the content is a terminal screen rendered by a PTY session.
    pub terminal: bool,
}

impl Stream {
    pub fn content(id: String, label: String, lines: Vec<String>) -> Self {
        Stream {
            id,
            kind: StreamKind::Content,
            label,
            content: lines,
            choices: None,
            status: StreamStatus::Idle,
            run: None,
            terminal: false,
        }
    }

    pub fn choices(id: String, label: String, choices: Vec<Choice>) -> Self {
        Stream {
            id,
            kind: StreamKind::Choices,
            label,
            content: Vec::new(),
            choices: Some(choices),
            status: StreamStatus::Idle,
            run: None,
            terminal: false,
        }
    }

    pub fn run(id: String, label: String, origin: RunOrigin, terminal: bool) -> Self {
        Stream {
            id,
            kind: StreamKind::Run(origin),
            label,
            content: Vec::new(),
            choices: None,
            status: StreamStatus::Idle,
            run: None,
            terminal,
        }
    }

    pub fn socket(id: String) -> Self {
        Stream {
            id,
            kind: StreamKind::Socket,
            label: "Socket".to_string(),
            content: Vec::new(),
            choices: None,
            status: StreamStatus::Idle,
            run: None,
            terminal: false,
        }
    }

    /// Whether the tab of this stream, shown in box `owner`, has a close button.
    /// Configuration-defined content and menus, and a box's own script output, stay for the life
    /// of the box.
    pub fn is_closeable(&self, owner: &str) -> bool {
        match &self.kind {
            StreamKind::Content | StreamKind::Choices => false,
            StreamKind::Run(RunOrigin::BoxScript { box_id }) => box_id != owner,
            StreamKind::Run(_) | StreamKind::Socket => true,
        }
    }

    /// Replaces the content with `text`, split into lines and cleaned of control characters
    /// (see [`crate::text`]).
    pub fn set_text(&mut self, text: &str) {
        self.content = crate::text::sanitize_output(text);
    }

    /// Appends `text`, split into lines and cleaned like [`Stream::set_text`].
    pub fn append_text(&mut self, text: &str) {
        self.content.extend(crate::text::sanitize_output(text));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origins_of_one_box_get_distinct_stable_stream_ids() {
        let script = RunOrigin::BoxScript { box_id: "a".into() };
        let choice = RunOrigin::Choice {
            box_id: "a".into(),
            choice_id: "c1".into(),
        };
        let key = RunOrigin::Keypress {
            box_id: "a".into(),
            key: "r".into(),
        };
        let ids = [
            script.stream_id("a"),
            choice.stream_id("a"),
            key.stream_id("a"),
        ];
        assert_eq!(ids[0], "a/script");
        assert_ne!(ids[0], ids[1]);
        assert_ne!(ids[1], ids[2]);
        // Starting the same origin again must land in the same tab.
        assert_eq!(choice.stream_id("a"), choice.clone().stream_id("a"));
    }

    #[test]
    fn redirected_output_of_two_boxes_does_not_share_a_tab() {
        let from_a = RunOrigin::Choice {
            box_id: "a".into(),
            choice_id: "go".into(),
        };
        let from_b = RunOrigin::Choice {
            box_id: "b".into(),
            choice_id: "go".into(),
        };
        assert_ne!(from_a.stream_id("log"), from_b.stream_id("log"));
    }

    #[test]
    fn own_script_tab_is_permanent_but_redirected_script_tab_closes() {
        let own = Stream::run(
            RunOrigin::BoxScript { box_id: "a".into() }.stream_id("a"),
            "Output".into(),
            RunOrigin::BoxScript { box_id: "a".into() },
            false,
        );
        assert!(!own.is_closeable("a"));

        let redirected = Stream::run(
            RunOrigin::BoxScript { box_id: "a".into() }.stream_id("log"),
            "a".into(),
            RunOrigin::BoxScript { box_id: "a".into() },
            false,
        );
        assert!(redirected.is_closeable("log"));

        let choice = Stream::run(
            "x".into(),
            "x".into(),
            RunOrigin::Choice {
                box_id: "a".into(),
                choice_id: "c".into(),
            },
            false,
        );
        assert!(choice.is_closeable("a"));
        assert!(!Stream::content("c".into(), "C".into(), vec![]).is_closeable("a"));
    }

    #[test]
    fn failure_statuses_are_errors_and_others_are_not() {
        assert!(StreamStatus::Exited(2).is_error());
        assert!(StreamStatus::Failed("no shell".into()).is_error());
        assert!(!StreamStatus::Exited(0).is_error());
        assert!(!StreamStatus::Killed.is_error());
        assert!(!StreamStatus::Running.is_error());
    }
}
