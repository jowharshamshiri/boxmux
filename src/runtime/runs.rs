//! Starting runs for boxes, choices and key handlers, and applying their output to streams.

use super::event::{RunEvent, RunOutcome};
use super::executor::RunSpec;
use super::{terminal_size, Runtime};
use crate::model::common::ExecutionMode;
use crate::model::stream::{RunId, RunOrigin, StreamStatus};
use crate::runtime::display::Display;

impl<D: Display> Runtime<D> {
    /// Starts a run whose output goes to the stream for `spec.origin` in `spec.target_box`.
    /// `reveal` makes that stream the visible tab (for runs the user started). Fails when the
    /// target box does not exist or a run for the same origin is still active.
    pub(super) fn start_run(
        &mut self,
        mut spec: RunSpec,
        label: &str,
        reveal: bool,
        append: bool,
    ) -> Result<RunId, String> {
        if let Some(active) = self.executor.run_for_origin(&spec.origin) {
            return Err(format!("{} is still running", active));
        }
        let terminal = spec.mode == ExecutionMode::Pty;
        if terminal && spec.terminal_size.is_none() {
            let bounds = self.ctx.app.active_bounds();
            let box_bounds = bounds.get(&spec.target_box).ok_or_else(|| {
                format!(
                    "box '{}' is not in the shown layout, so it has no size for a terminal",
                    spec.target_box
                )
            })?;
            spec.terminal_size = Some(terminal_size(box_bounds).ok_or_else(|| {
                format!("box '{}' is too small to show a terminal", spec.target_box)
            })?);
        }
        let target = self
            .ctx
            .app
            .get_muxbox_by_id_mut(&spec.target_box)
            .ok_or_else(|| format!("there is no box '{}'", spec.target_box))?;
        let stream_id = target
            .stream_for_origin(&spec.origin, label, terminal)
            .id
            .clone();
        let run = self.executor.start(spec.clone(), stream_id.clone())?;

        let target = self
            .ctx
            .app
            .get_muxbox_by_id_mut(&spec.target_box)
            .expect("the target box was found above");
        let stream = target
            .streams
            .get_mut(&stream_id)
            .expect("the stream was created above");
        stream.run = Some(run);
        stream.status = StreamStatus::Running;
        stream.terminal = terminal;
        if spec.mode == ExecutionMode::Thread && !append || terminal {
            stream.content.clear();
        }
        if reveal {
            target.selected_stream_id = Some(stream_id.clone());
            // Scroll the tab bar so the revealed tab is the first one shown; the renderer keeps
            // the offset within the bar's range.
            target.tab_scroll_offset = target.get_active_tab_index();
        }
        if let RunOrigin::Choice { box_id, choice_id } = &spec.origin {
            if let Some(choice) = self
                .ctx
                .app
                .get_muxbox_by_id_mut(box_id)
                .and_then(|b| b.choice_mut(choice_id))
            {
                choice.waiting = true;
            }
        }
        self.last_spec.insert(stream_id, spec);
        self.dirty = true;
        Ok(run)
    }

    /// Runs the box's own `script` into its output tab.
    pub(super) fn start_box_script(&mut self, box_id: &str, reveal: bool) -> Result<RunId, String> {
        let muxbox = self
            .ctx
            .app
            .get_muxbox_by_id(box_id)
            .ok_or_else(|| format!("there is no box '{}'", box_id))?;
        let script = muxbox
            .script
            .clone()
            .ok_or_else(|| format!("box '{}' has no script", box_id))?;
        let target_box = muxbox
            .redirect_output
            .clone()
            .unwrap_or_else(|| box_id.to_string());
        let label = if target_box == box_id {
            muxbox
                .streams
                .get(
                    &RunOrigin::BoxScript {
                        box_id: box_id.to_string(),
                    }
                    .stream_id(box_id),
                )
                .map(|s| s.label.clone())
                .unwrap_or_else(|| "Output".to_string())
        } else {
            muxbox.title.clone().unwrap_or_else(|| box_id.to_string())
        };
        let append = muxbox.append_output.unwrap_or(false);
        let mode = muxbox.execution_mode;
        self.start_run(
            RunSpec {
                origin: RunOrigin::BoxScript {
                    box_id: box_id.to_string(),
                },
                target_box,
                script,
                mode,
                terminal_size: None,
            },
            &label,
            reveal,
            append,
        )
    }

    /// Activates a choice of `box_id`: selects it and runs its script, if it has one. A choice
    /// whose previous run is still active is not started again; a PTY choice brings its session
    /// into view instead.
    pub(super) fn activate_choice(&mut self, box_id: &str, choice_id: &str) {
        let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(box_id) else {
            return;
        };
        if let Some(choices) = muxbox
            .choices_stream_mut()
            .and_then(|stream| stream.choices.as_mut())
        {
            for choice in choices.iter_mut() {
                choice.selected = choice.id == choice_id;
            }
        }
        self.dirty = true;
        let Some(choice) = muxbox.choice_mut(choice_id).map(|c| c.clone()) else {
            return;
        };
        let Some(script) = choice.script.clone() else {
            return;
        };
        let origin = RunOrigin::Choice {
            box_id: box_id.to_string(),
            choice_id: choice_id.to_string(),
        };
        let target_box = choice
            .redirect_output
            .clone()
            .unwrap_or_else(|| box_id.to_string());
        if let Some(run) = self.executor.run_for_origin(&origin) {
            // Already running: show it rather than starting a second copy.
            let stream_id = origin.stream_id(&target_box);
            if let Some(target) = self.ctx.app.get_muxbox_by_id_mut(&target_box) {
                target.switch_to_stream(&stream_id);
            }
            log::debug!("choice '{}' is still running as {}", choice_id, run);
            return;
        }
        let label = choice.content.clone().unwrap_or_else(|| choice.id.clone());
        let result = self.start_run(
            RunSpec {
                origin,
                target_box,
                script,
                mode: choice.execution_mode,
                terminal_size: None,
            },
            &label,
            true,
            choice.append_output.unwrap_or(false),
        );
        if let Err(e) = result {
            self.notify(format!("choice '{}': {}", choice_id, e), true);
        }
    }

    /// Runs an `on_keypress` handler of a box into the box (or its `redirect_output`).
    pub(super) fn start_box_keypress(&mut self, box_id: &str, key: &str, script: Vec<String>) {
        let Some(muxbox) = self.ctx.app.get_muxbox_by_id(box_id) else {
            return;
        };
        let target_box = muxbox
            .redirect_output
            .clone()
            .unwrap_or_else(|| box_id.to_string());
        let append = muxbox.append_output.unwrap_or(false);
        let result = self.start_run(
            RunSpec {
                origin: RunOrigin::Keypress {
                    box_id: box_id.to_string(),
                    key: key.to_string(),
                },
                target_box,
                script,
                mode: ExecutionMode::Thread,
                terminal_size: None,
            },
            key,
            true,
            append,
        );
        if let Err(e) = result {
            self.notify(format!("key {} in box '{}': {}", key, box_id, e), true);
        }
    }

    /// Runs an app- or layout-level `on_keypress` handler. Its output is not shown; a failure
    /// is reported as a notice.
    pub(super) fn start_global_keypress(&mut self, scope: &str, key: &str, script: Vec<String>) {
        let origin = RunOrigin::GlobalKeypress {
            scope: scope.to_string(),
            key: key.to_string(),
        };
        if self.executor.run_for_origin(&origin).is_some() {
            return;
        }
        let spec = RunSpec {
            origin,
            target_box: String::new(),
            script,
            mode: ExecutionMode::Immediate,
            terminal_size: None,
        };
        if let Err(e) = self.executor.start(spec, String::new()) {
            self.notify(format!("key {}: {}", key, e), true);
        }
    }

    pub(super) fn handle_run_event(&mut self, event: RunEvent) {
        match event {
            RunEvent::Output { run, text } => {
                let Some((box_id, stream_id)) = self.run_target(run) else {
                    return;
                };
                let lines = crate::text::sanitize_output(&text);
                if let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(&box_id) {
                    if let Some(stream) = muxbox.streams.get_mut(&stream_id) {
                        stream.content.extend(lines);
                    }
                    follow_output(muxbox, &stream_id);
                }
                self.dirty = true;
            }
            RunEvent::TerminalUpdated { run } => self.refresh_terminal(run),
            RunEvent::Finished {
                run,
                output,
                outcome,
            } => self.finish_run(run, output, outcome),
        }
    }

    /// The box and stream a run writes to, while both exist.
    fn run_target(&self, run: RunId) -> Option<(String, String)> {
        let active = self.executor.get(run)?;
        if active.target_box.is_empty() {
            return None;
        }
        Some((active.target_box.clone(), active.stream_id.clone()))
    }

    /// Copies a PTY session's screen into its stream.
    fn refresh_terminal(&mut self, run: RunId) {
        let Some(active) = self.executor.get(run) else {
            return;
        };
        let Some(session) = active.pty() else {
            return;
        };
        let snapshot = session.snapshot();
        let (box_id, stream_id) = (active.target_box.clone(), active.stream_id.clone());
        if let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(&box_id) {
            if let Some(stream) = muxbox.streams.get_mut(&stream_id) {
                stream.content = snapshot.lines;
                if let Some(title) = snapshot.title.filter(|t| !t.trim().is_empty()) {
                    stream.label = title;
                }
            }
            follow_output(muxbox, &stream_id);
        }
        self.dirty = true;
    }

    fn finish_run(&mut self, run: RunId, output: Option<String>, outcome: RunOutcome) {
        // The final screen of a PTY session is copied before the session is dropped.
        self.refresh_terminal(run);
        let Some(active) = self.executor.finish(run) else {
            return;
        };
        let status = if active.stop_requested {
            StreamStatus::Killed
        } else {
            match &outcome {
                RunOutcome::Exited(code) => StreamStatus::Exited(*code),
                RunOutcome::Signaled(signal) => {
                    StreamStatus::Failed(format!("ended by signal {}", signal))
                }
                RunOutcome::Failed(reason) => StreamStatus::Failed(reason.clone()),
            }
        };
        if let RunOrigin::Choice { box_id, choice_id } = &active.origin {
            if let Some(choice) = self
                .ctx
                .app
                .get_muxbox_by_id_mut(box_id)
                .and_then(|b| b.choice_mut(choice_id))
            {
                choice.waiting = false;
            }
        }
        if let RunOrigin::GlobalKeypress { key, .. } = &active.origin {
            if status.is_error() {
                self.notify(
                    format!("the handler for key {} failed: {:?}", key, status),
                    true,
                );
            }
            return;
        }

        let append = self.append_mode(&active.origin);
        let mut saved_content = None;
        if let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(&active.target_box) {
            if let Some(stream) = muxbox.streams.get_mut(&active.stream_id) {
                if let Some(output) = output {
                    let lines = crate::text::sanitize_output(&output);
                    if append {
                        stream.content.extend(lines);
                    } else {
                        stream.content = lines;
                    }
                }
                stream.status = status;
                if stream.run == Some(run) {
                    stream.run = None;
                }
                if matches!(active.origin, RunOrigin::BoxScript { .. }) && !stream.terminal {
                    saved_content = Some(stream.content.join("\n"));
                }
            }
            follow_output(muxbox, &active.stream_id);
        }
        if let (Some(content), RunOrigin::BoxScript { box_id }) = (saved_content, &active.origin) {
            self.save_output_to_file(box_id, &content);
        }
        if let Some(spec) = self.restart_after_stop.remove(&active.stream_id) {
            let label = self.stream_label(&spec.target_box, &active.stream_id);
            if let Err(e) = self.start_run(spec, &label, true, append) {
                self.notify(format!("cannot restart: {}", e), true);
            }
        }
        self.dirty = true;
    }

    fn stream_label(&self, box_id: &str, stream_id: &str) -> String {
        self.ctx
            .app
            .get_muxbox_by_id(box_id)
            .and_then(|b| b.streams.get(stream_id))
            .map(|s| s.label.clone())
            .unwrap_or_else(|| "Output".to_string())
    }

    /// Whether output of runs for `origin` is appended to (rather than replacing) the stream.
    fn append_mode(&self, origin: &RunOrigin) -> bool {
        match origin {
            RunOrigin::BoxScript { box_id } | RunOrigin::Keypress { box_id, .. } => self
                .ctx
                .app
                .get_muxbox_by_id(box_id)
                .and_then(|b| b.append_output)
                .unwrap_or(false),
            RunOrigin::Choice { box_id, choice_id } => self
                .ctx
                .app
                .get_muxbox_by_id(box_id)
                .and_then(|b| b.choices.as_ref())
                .and_then(|choices| choices.iter().find(|c| &c.id == choice_id))
                .and_then(|c| c.append_output)
                .unwrap_or(false),
            RunOrigin::SocketSpawn { .. } | RunOrigin::GlobalKeypress { .. } => false,
        }
    }

    /// Writes a box's script output to its `save_in_file`, relative to the configuration file.
    fn save_output_to_file(&mut self, box_id: &str, content: &str) {
        let Some(target) = self
            .ctx
            .app
            .get_muxbox_by_id(box_id)
            .and_then(|b| b.save_in_file.clone())
        else {
            return;
        };
        let mut path = std::path::PathBuf::from(&target);
        if path.is_relative() {
            if let Some(dir) = self.ctx.yaml_file_path.as_ref().and_then(|p| p.parent()) {
                path = dir.join(path);
            }
        }
        let mut text = content.to_string();
        text.push('\n');
        if let Err(e) = std::fs::write(&path, crate::utils::strip_ansi_codes(&text)) {
            self.notify(
                format!(
                    "box '{}': cannot save output to '{}': {}",
                    box_id,
                    path.display(),
                    e
                ),
                true,
            );
        }
    }

    /// Stops every run that targets or was started by `box_id`.
    pub(super) fn stop_runs_of_box(&mut self, box_id: &str) {
        let runs: Vec<RunId> = self
            .executor
            .runs()
            .filter(|(_, run)| run.target_box == box_id || run.origin.source_box() == Some(box_id))
            .map(|(id, _)| *id)
            .collect();
        for run in runs {
            self.executor.stop(run);
        }
    }
}

/// Keeps the newest output in view: boxes with `auto_scroll_bottom`, and terminal streams,
/// scroll to the bottom when their visible stream changes.
fn follow_output(muxbox: &mut crate::MuxBox, stream_id: &str) {
    let visible = muxbox
        .get_selected_stream()
        .is_some_and(|s| s.id == stream_id);
    let terminal = muxbox.streams.get(stream_id).is_some_and(|s| s.terminal);
    if visible && (muxbox.auto_scroll_bottom == Some(true) || terminal) {
        muxbox.vertical_scroll = Some(100.0);
    }
}
