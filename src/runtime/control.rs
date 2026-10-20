//! Commands from the control socket.

use super::executor::RunSpec;
use super::{Refresh, Runtime};
use crate::model::common::ExecutionMode;
use crate::model::stream::{RunId, RunOrigin, Stream, StreamKind, StreamStatus};
use crate::runtime::display::Display;
use crate::socket::protocol::{ProcessStatus, Reply, SocketFunction};
use crate::{App, MuxBox};
use std::time::Instant;

impl<D: Display> Runtime<D> {
    pub(super) fn handle_socket(&mut self, function: SocketFunction) -> Reply {
        let result = match function {
            SocketFunction::ReplaceBoxContent {
                box_id,
                content,
                success,
            } => self.replace_box_content(&box_id, &content, success),
            SocketFunction::ReplaceBoxScript { box_id, script } => {
                self.replace_box_script(&box_id, script)
            }
            SocketFunction::StopBoxRefresh { box_id } => self.set_refresh_paused(&box_id, true),
            SocketFunction::StartBoxRefresh { box_id } => self.set_refresh_paused(&box_id, false),
            SocketFunction::ReplaceBox { box_id, new_box } => {
                self.replace_box(&box_id, new_box.into_muxbox())
            }
            SocketFunction::AddBox {
                layout_id,
                parent_id,
                muxbox,
            } => self.add_box(&layout_id, parent_id.as_deref(), muxbox.into_muxbox()),
            SocketFunction::RemoveBox { box_id } => self.remove_box(&box_id),
            SocketFunction::SwitchActiveLayout { layout_id } => self.switch_layout(&layout_id),
            SocketFunction::SpawnPtyProcess {
                box_id,
                script,
                redirect_output,
            } => self.spawn_pty(&box_id, script, redirect_output),
            SocketFunction::KillPtyProcess { box_id } => self.visible_run(&box_id).map(|run| {
                self.executor.stop(run);
                Reply::ok()
            }),
            SocketFunction::RestartPtyProcess { box_id } => self.restart_visible_run(&box_id),
            SocketFunction::QueryPtyStatus { box_id } => self.process_status(&box_id),
            SocketFunction::SendPtyInput { box_id, input } => self
                .visible_run(&box_id)
                .and_then(|run| {
                    self.executor
                        .get(run)
                        .and_then(|r| r.pty())
                        .ok_or_else(|| {
                            format!("the process in box '{}' is not a PTY process", box_id)
                        })?
                        .write_input(input.as_bytes())
                })
                .map(|_| Reply::ok()),
            SocketFunction::SendSignal { box_id, signal } => self
                .visible_run(&box_id)
                .and_then(|run| self.executor.signal(run, signal))
                .map(|_| Reply::ok()),
        };
        self.dirty = true;
        result.unwrap_or_else(Reply::error)
    }

    fn box_mut(&mut self, box_id: &str) -> Result<&mut MuxBox, String> {
        self.ctx
            .app
            .get_muxbox_by_id_mut(box_id)
            .ok_or_else(|| format!("there is no box '{}'", box_id))
    }

    fn replace_box_content(
        &mut self,
        box_id: &str,
        content: &str,
        success: bool,
    ) -> Result<Reply, String> {
        let muxbox = self.box_mut(box_id)?;
        let stream_id = format!("{}/socket", box_id);
        let stream = muxbox
            .streams
            .entry(stream_id.clone())
            .or_insert_with(|| Stream::socket(stream_id.clone()));
        stream.content = crate::text::sanitize_output(content);
        stream.status = if success {
            StreamStatus::Exited(0)
        } else {
            StreamStatus::Failed("the sender reported a failure".to_string())
        };
        muxbox.selected_stream_id = Some(stream_id);
        Ok(Reply::ok())
    }

    fn replace_box_script(&mut self, box_id: &str, script: Vec<String>) -> Result<Reply, String> {
        if script.iter().all(|line| line.trim().is_empty()) {
            return Err("the script is empty".to_string());
        }
        self.box_mut(box_id)?.script = Some(script);
        let origin = RunOrigin::BoxScript {
            box_id: box_id.to_string(),
        };
        if let Some(run) = self.executor.run_for_origin(&origin) {
            // The new script starts when the old one has stopped.
            let spec = self.box_script_spec(box_id)?;
            self.restart_after_stop
                .insert(origin.stream_id(&spec.target_box), spec);
            self.executor.stop(run);
        } else {
            self.start_box_script(box_id, true)?;
        }
        // A box that had no script now refreshes like one configured with this script; an
        // existing schedule (and its pause) is kept.
        if !self.refresh.contains_key(box_id) {
            let (action, interval) = self
                .refresh_settings(box_id)
                .expect("the box has a script now");
            self.refresh.insert(
                box_id.to_string(),
                Refresh {
                    action,
                    interval,
                    next_due: interval.map(|interval| Instant::now() + interval),
                    paused: false,
                },
            );
        }
        Ok(Reply::ok())
    }

    fn box_script_spec(&self, box_id: &str) -> Result<RunSpec, String> {
        let muxbox = self
            .ctx
            .app
            .get_muxbox_by_id(box_id)
            .ok_or_else(|| format!("there is no box '{}'", box_id))?;
        Ok(RunSpec {
            origin: RunOrigin::BoxScript {
                box_id: box_id.to_string(),
            },
            target_box: muxbox
                .redirect_output
                .clone()
                .unwrap_or_else(|| box_id.to_string()),
            script: muxbox
                .script
                .clone()
                .ok_or_else(|| format!("box '{}' has no script", box_id))?,
            mode: muxbox.execution_mode,
            terminal_size: None,
        })
    }

    fn set_refresh_paused(&mut self, box_id: &str, paused: bool) -> Result<Reply, String> {
        if self.ctx.app.get_muxbox_by_id(box_id).is_none() {
            return Err(format!("there is no box '{}'", box_id));
        }
        let (action, interval) = self
            .refresh_settings(box_id)
            .ok_or_else(|| format!("box '{}' has no script", box_id))?;
        if interval.is_none() {
            return Err(format!(
                "box '{}' does not refresh: it has no refresh_interval, or its script runs in a PTY",
                box_id
            ));
        }
        // A box of a layout that was never shown has no schedule yet.
        let refresh = self.refresh.entry(box_id.to_string()).or_insert(Refresh {
            action,
            interval,
            next_due: Some(Instant::now()),
            paused,
        });
        refresh.paused = paused;
        if !paused {
            refresh.next_due = Some(Instant::now());
        }
        Ok(Reply::ok())
    }

    /// Applies a change to a copy of the app, checks the result, and only then commits it.
    fn change_structure(
        &mut self,
        change: impl FnOnce(&mut App) -> Result<(), String>,
    ) -> Result<(), String> {
        let mut candidate = self.ctx.app.clone();
        change(&mut candidate)?;
        let app_value = serde_json::to_value(&candidate).map_err(|e| e.to_string())?;
        let document = serde_json::json!({ "app": without_nulls(app_value) });
        crate::validation::validate_schema(&document)
            .and_then(|_| crate::validation::validate_semantics(&candidate))
            .map_err(|errors| {
                errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            })?;
        crate::model::app::check_plugin_components(&candidate, &self.ctx.plugins)?;
        self.ctx.app = candidate;
        Ok(())
    }

    fn replace_box(&mut self, box_id: &str, mut new_box: MuxBox) -> Result<Reply, String> {
        if new_box.id != box_id {
            return Err(format!(
                "the new box has id '{}' but replaces '{}'; ids must match",
                new_box.id, box_id
            ));
        }
        let old = self
            .ctx
            .app
            .get_muxbox_by_id(box_id)
            .ok_or_else(|| format!("there is no box '{}'", box_id))?;
        new_box.parent_id = old.parent_id.clone();
        new_box.parent_layout_id = old.parent_layout_id.clone();
        new_box.selected = old.selected;
        prepare_new_box(&mut new_box);
        self.change_structure(|app| {
            app.replace_muxbox(new_box);
            Ok(())
        })?;
        self.stop_runs_of_box(box_id);
        self.refresh.remove(box_id);
        self.ctx.app.invalidate_layout_cache();
        self.activate_layout_runs();
        Ok(Reply::ok())
    }

    fn add_box(
        &mut self,
        layout_id: &str,
        parent_id: Option<&str>,
        mut muxbox: MuxBox,
    ) -> Result<Reply, String> {
        if self.ctx.app.get_muxbox_by_id(&muxbox.id).is_some() {
            return Err(format!("a box with id '{}' already exists", muxbox.id));
        }
        muxbox.parent_layout_id = Some(layout_id.to_string());
        muxbox.parent_id = parent_id.map(str::to_string);
        muxbox.selected = Some(false);
        prepare_new_box(&mut muxbox);
        self.change_structure(|app| {
            let layout = app
                .get_layout_by_id_mut(layout_id)
                .ok_or_else(|| format!("there is no layout '{}'", layout_id))?;
            let siblings = match parent_id {
                Some(parent) => layout
                    .get_muxbox_by_id_mut(parent)
                    .ok_or_else(|| {
                        format!("there is no box '{}' in layout '{}'", parent, layout_id)
                    })?
                    .children
                    .get_or_insert_with(Vec::new),
                None => layout.children.get_or_insert_with(Vec::new),
            };
            siblings.push(muxbox);
            Ok(())
        })?;
        self.ctx.app.invalidate_layout_cache();
        self.activate_layout_runs();
        Ok(Reply::ok())
    }

    fn remove_box(&mut self, box_id: &str) -> Result<Reply, String> {
        let layout_id = self
            .ctx
            .app
            .layout_of_box(box_id)
            .map(|layout| layout.id.clone())
            .ok_or_else(|| format!("there is no box '{}'", box_id))?;
        let removed_ids: Vec<String> = {
            let muxbox = self.ctx.app.get_muxbox_by_id(box_id).expect("found above");
            let mut ids = vec![muxbox.id.clone()];
            fn collect(muxbox: &MuxBox, ids: &mut Vec<String>) {
                for child in muxbox.children.iter().flatten() {
                    ids.push(child.id.clone());
                    collect(child, ids);
                }
            }
            collect(muxbox, &mut ids);
            ids
        };
        self.change_structure(|app| {
            let layout = app
                .get_layout_by_id_mut(&layout_id)
                .expect("layout found above");
            fn remove(muxboxes: &mut Vec<MuxBox>, id: &str) -> bool {
                if let Some(index) = muxboxes.iter().position(|b| b.id == id) {
                    muxboxes.remove(index);
                    return true;
                }
                muxboxes.iter_mut().any(|b| {
                    b.children
                        .as_mut()
                        .is_some_and(|children| remove(children, id))
                })
            }
            if !layout
                .children
                .as_mut()
                .is_some_and(|children| remove(children, box_id))
            {
                return Err(format!("there is no box '{}'", box_id));
            }
            Ok(())
        })?;
        for id in &removed_ids {
            self.stop_runs_of_box(id);
            self.refresh.remove(id);
        }
        let layout = self
            .ctx
            .app
            .get_layout_by_id_mut(&layout_id)
            .expect("layout exists");
        if layout.get_selected_muxboxes().is_empty() {
            layout.select_next_muxbox();
        }
        self.ctx.app.invalidate_layout_cache();
        Ok(Reply::ok())
    }

    fn switch_layout(&mut self, layout_id: &str) -> Result<Reply, String> {
        self.ctx.app.set_active_layout(layout_id)?;
        self.screen_resized();
        self.activate_layout_runs();
        Ok(Reply::ok())
    }

    fn spawn_pty(
        &mut self,
        box_id: &str,
        script: Vec<String>,
        redirect_output: Option<String>,
    ) -> Result<Reply, String> {
        if self.ctx.app.get_muxbox_by_id(box_id).is_none() {
            return Err(format!("there is no box '{}'", box_id));
        }
        let target_box = redirect_output.unwrap_or_else(|| box_id.to_string());
        let run = self.start_run(
            RunSpec {
                origin: RunOrigin::SocketSpawn {
                    box_id: box_id.to_string(),
                },
                target_box,
                script,
                mode: ExecutionMode::Pty,
                terminal_size: None,
            },
            "Terminal",
            true,
            false,
        )?;
        Ok(Reply::with_result(serde_json::json!({ "run": run.0 })))
    }

    /// The run feeding the visible tab of `box_id`.
    fn visible_run(&self, box_id: &str) -> Result<RunId, String> {
        let muxbox = self
            .ctx
            .app
            .get_muxbox_by_id(box_id)
            .ok_or_else(|| format!("there is no box '{}'", box_id))?;
        muxbox
            .get_selected_stream()
            .and_then(|stream| stream.run)
            .ok_or_else(|| format!("nothing is running in the visible tab of box '{}'", box_id))
    }

    fn restart_visible_run(&mut self, box_id: &str) -> Result<Reply, String> {
        let muxbox = self
            .ctx
            .app
            .get_muxbox_by_id(box_id)
            .ok_or_else(|| format!("there is no box '{}'", box_id))?;
        let stream = muxbox
            .get_selected_stream()
            .ok_or_else(|| format!("box '{}' has no tabs", box_id))?;
        let stream_id = stream.id.clone();
        let running = stream.run;
        let spec =
            self.last_spec.get(&stream_id).cloned().ok_or_else(|| {
                format!("the visible tab of box '{}' has never run anything", box_id)
            })?;
        match running {
            Some(run) => {
                self.restart_after_stop.insert(stream_id, spec);
                self.executor.stop(run);
            }
            None => {
                let label = stream.label.clone();
                self.start_run(spec, &label, true, false)?;
            }
        }
        Ok(Reply::ok())
    }

    fn process_status(&mut self, box_id: &str) -> Result<Reply, String> {
        let muxbox = self
            .ctx
            .app
            .get_muxbox_by_id(box_id)
            .ok_or_else(|| format!("there is no box '{}'", box_id))?;
        let stream = muxbox
            .get_selected_stream()
            .ok_or_else(|| format!("box '{}' has no tabs", box_id))?;
        if !matches!(stream.kind, StreamKind::Run(_)) {
            return Err(format!(
                "the visible tab of box '{}' ('{}') does not show a process",
                box_id, stream.label
            ));
        }
        let active = stream.run.and_then(|run| self.executor.get(run));
        let size = active.and_then(|a| a.pty()).map(|s| s.size());
        let status = ProcessStatus {
            box_id: box_id.to_string(),
            stream: stream.label.clone(),
            running: active.is_some(),
            pid: active.map(|a| a.pid()),
            status: match &stream.status {
                StreamStatus::Idle => "idle".to_string(),
                StreamStatus::Running => "running".to_string(),
                StreamStatus::Exited(code) => format!("exited with code {}", code),
                StreamStatus::Failed(reason) => format!("failed: {}", reason),
                StreamStatus::Killed => "killed".to_string(),
            },
            rows: size.map(|s| s.rows),
            cols: size.map(|s| s.cols),
        };
        Ok(Reply::with_result(
            serde_json::to_value(status).map_err(|e| e.to_string())?,
        ))
    }
}

/// Drops `null` entries from objects: an unset optional field serializes as `null`, which the
/// configuration format expresses by leaving the field out.
fn without_nulls(value: serde_json::Value) -> serde_json::Value {
    match value {
        serde_json::Value::Object(map) => serde_json::Value::Object(
            map.into_iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k, without_nulls(v)))
                .collect(),
        ),
        serde_json::Value::Array(items) => {
            serde_json::Value::Array(items.into_iter().map(without_nulls).collect())
        }
        other => other,
    }
}

/// Builds the tabs of a box (and its children) added or replaced at runtime.
fn prepare_new_box(muxbox: &mut MuxBox) {
    muxbox.initialize_streams();
    let parent_layout = muxbox.parent_layout_id.clone();
    let id = muxbox.id.clone();
    for child in muxbox.children.iter_mut().flatten() {
        child.parent_layout_id = parent_layout.clone();
        child.parent_id = Some(id.clone());
        prepare_new_box(child);
    }
}
