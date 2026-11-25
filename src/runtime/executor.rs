//! Starts, tracks and stops runs of scripts and PTY sessions.
//!
//! Every run executes in its own process tree, off the runtime thread, and reports progress as
//! [`RunEvent`]s. Stopping a run (closing its tab, removing its box, quitting) ends the whole
//! tree, including background processes the script started.

use crate::model::common::ExecutionMode;
use crate::model::stream::{RunId, RunOrigin};
use crate::platform::{ProcessTree, Shell};
use crate::runtime::event::{Event, RunEvent, RunOutcome};
use crate::runtime::pty::{PtySession, TerminalSize};
use crossbeam_channel::Sender;
use std::collections::HashMap;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

/// How long a stopped process tree gets to exit after SIGTERM before it is killed.
const TERMINATE_GRACE: Duration = Duration::from_millis(500);

/// What to run, how, and where its output goes.
#[derive(Debug, Clone)]
pub struct RunSpec {
    pub origin: RunOrigin,
    /// The box whose stream receives the output.
    pub target_box: String,
    pub script: Vec<String>,
    pub mode: ExecutionMode,
    /// The terminal size for `Pty` runs. The runtime sets it from the target box each time it
    /// starts the run, a restart included.
    pub terminal_size: Option<TerminalSize>,
    /// Variables this run has in its environment beside those every run has.
    pub environment: Vec<(String, String)>,
}

/// A run that has started and not yet been reported finished.
pub struct ActiveRun {
    pub origin: RunOrigin,
    pub target_box: String,
    pub stream_id: String,
    pub mode: ExecutionMode,
    /// Set when the run was stopped on purpose, so its end is reported as killed.
    pub stop_requested: bool,
    process: Process,
}

enum Process {
    Child(ProcessTree),
    Pty(PtySession),
}

impl ActiveRun {
    pub fn pid(&self) -> u32 {
        match &self.process {
            Process::Child(tree) => tree.pid(),
            Process::Pty(session) => session.process_tree().pid(),
        }
    }

    pub fn pty(&self) -> Option<&PtySession> {
        match &self.process {
            Process::Pty(session) => Some(session),
            Process::Child(_) => None,
        }
    }

    pub fn pty_mut(&mut self) -> Option<&mut PtySession> {
        match &mut self.process {
            Process::Pty(session) => Some(session),
            Process::Child(_) => None,
        }
    }

    fn tree(&self) -> &ProcessTree {
        match &self.process {
            Process::Child(tree) => tree,
            Process::Pty(session) => session.process_tree(),
        }
    }
}

pub struct Executor {
    events: Sender<Event>,
    shell: Shell,
    libs: Vec<PathBuf>,
    environment: Vec<(String, String)>,
    next_id: u64,
    runs: HashMap<RunId, ActiveRun>,
}

impl Executor {
    /// `libs` are sourced before every script; `environment` is added to every run's
    /// environment.
    pub fn new(
        events: Sender<Event>,
        shell: Shell,
        libs: Vec<PathBuf>,
        environment: Vec<(String, String)>,
    ) -> Self {
        Executor {
            events,
            shell,
            libs,
            environment,
            next_id: 1,
            runs: HashMap::new(),
        }
    }

    pub fn get(&self, run: RunId) -> Option<&ActiveRun> {
        self.runs.get(&run)
    }

    pub fn get_mut(&mut self, run: RunId) -> Option<&mut ActiveRun> {
        self.runs.get_mut(&run)
    }

    pub fn runs(&self) -> impl Iterator<Item = (&RunId, &ActiveRun)> {
        self.runs.iter()
    }

    /// The active run started for `origin`, if any.
    pub fn run_for_origin(&self, origin: &RunOrigin) -> Option<RunId> {
        self.runs
            .iter()
            .find(|(_, run)| &run.origin == origin)
            .map(|(id, _)| *id)
    }

    /// Starts a run. Its output goes to `stream_id` of `spec.target_box`.
    pub fn start(&mut self, spec: RunSpec, stream_id: String) -> Result<RunId, String> {
        let run = RunId(self.next_id);
        self.next_id += 1;
        let text = self.shell.script_text(&self.libs, &spec.script);
        let (program, args) = self.shell.invocation(spec.mode == ExecutionMode::Pty);
        let process = match spec.mode {
            ExecutionMode::Pty => {
                let size = spec
                    .terminal_size
                    .ok_or_else(|| "a PTY run needs a terminal size".to_string())?;
                let mut command = portable_pty::CommandBuilder::new(program);
                command.args(args);
                command.arg(&text);
                command.cwd(
                    std::env::current_dir()
                        .map_err(|e| format!("cannot read the current directory: {}", e))?,
                );
                command.env("TERM", "xterm-256color");
                for (name, value) in &self.environment {
                    command.env(name, value);
                }
                for (name, value) in &spec.environment {
                    command.env(name, value);
                }
                command.env("BOXMUX_BOX", &spec.target_box);
                Process::Pty(PtySession::spawn(run, command, size, self.events.clone())?)
            }
            ExecutionMode::Immediate | ExecutionMode::Thread => {
                let mut command = Command::new(program);
                command.args(args).arg(&text);
                crate::platform::detach(&mut command);
                command.stdout(Stdio::piped()).stderr(Stdio::piped());
                for (name, value) in &self.environment {
                    command.env(name, value);
                }
                for (name, value) in &spec.environment {
                    command.env(name, value);
                }
                command.env("BOXMUX_BOX", &spec.target_box);
                let child = command
                    .spawn()
                    .map_err(|e| format!("cannot start {}: {}", program, e))?;
                let tree = ProcessTree::new(child.id());
                if let Err(e) =
                    spawn_waiter(run, child, tree.clone(), spec.mode, self.events.clone())
                {
                    // Nothing would read or wait for the process.
                    tree.kill();
                    return Err(e);
                }
                Process::Child(tree)
            }
        };
        self.runs.insert(
            run,
            ActiveRun {
                origin: spec.origin,
                target_box: spec.target_box,
                stream_id,
                mode: spec.mode,
                stop_requested: false,
                process,
            },
        );
        Ok(run)
    }

    /// Asks a run's process tree to stop. Its `Finished` event still arrives and is reported as
    /// killed. Stopping a run that already finished does nothing.
    pub fn stop(&mut self, run: RunId) {
        if let Some(active) = self.runs.get_mut(&run) {
            active.stop_requested = true;
            active.tree().terminate(TERMINATE_GRACE);
        }
    }

    /// Sends a Unix signal to a run's process tree.
    pub fn signal(&self, run: RunId, signal: i32) -> Result<(), String> {
        if signal <= 0 {
            return Err(format!("signal {} is not a signal number", signal));
        }
        let active = self
            .runs
            .get(&run)
            .ok_or_else(|| format!("{} is not running", run))?;
        crate::platform::send_signal(active.pid(), signal)
            .map_err(|e| format!("cannot send signal {}: {}", signal, e))
    }

    /// Forgets a run after its `Finished` event, returning what is known about it.
    pub fn finish(&mut self, run: RunId) -> Option<ActiveRun> {
        self.runs.remove(&run)
    }
}

/// Dropping the executor (when the runtime quits or fails, or a test ends) kills every run's
/// process tree, so no script outlives boxmux.
impl Drop for Executor {
    fn drop(&mut self) {
        for active in self.runs.values() {
            active.tree().kill();
        }
    }
}

/// Reads a run's output and waits for it, then reports the outcome. `Thread` runs report each
/// complete line as it arrives; `Immediate` runs report the whole output at the end.
fn spawn_waiter(
    run: RunId,
    mut child: std::process::Child,
    tree: ProcessTree,
    mode: ExecutionMode,
    events: Sender<Event>,
) -> Result<(), String> {
    let stdout = child.stdout.take().expect("stdout is piped");
    let stderr = child.stderr.take().expect("stderr is piped");
    let streaming = mode == ExecutionMode::Thread;
    let collected = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));

    let spawn_reader = |name: &str, mut source: Box<dyn Read + Send>| {
        let events = events.clone();
        let collected = collected.clone();
        std::thread::Builder::new()
            .name(format!("{}-{}", name, run))
            .spawn(move || {
                let mut pending = Vec::new();
                let mut buffer = [0u8; 8192];
                loop {
                    let n = match source.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => n,
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        Err(_) => break,
                    };
                    if !streaming {
                        collected
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .extend_from_slice(&buffer[..n]);
                        continue;
                    }
                    pending.extend_from_slice(&buffer[..n]);
                    if let Some(last_newline) = pending.iter().rposition(|b| *b == b'\n') {
                        let complete: Vec<u8> = pending.drain(..=last_newline).collect();
                        let text = String::from_utf8_lossy(&complete).into_owned();
                        if events
                            .send(Event::Run(RunEvent::Output { run, text }))
                            .is_err()
                        {
                            return;
                        }
                    }
                }
                if streaming && !pending.is_empty() {
                    let text = String::from_utf8_lossy(&pending).into_owned();
                    let _ = events.send(Event::Run(RunEvent::Output { run, text }));
                }
            })
            .map_err(|e| format!("cannot start an output reader: {}", e))
    };
    let out_reader = spawn_reader("stdout", Box::new(stdout))?;
    let err_reader = spawn_reader("stderr", Box::new(stderr))?;

    std::thread::Builder::new()
        .name(format!("wait-{}", run))
        .spawn(move || {
            // Output is complete once both pipes closed; only then is the exit reported, so
            // the runtime never sees output after the run finished.
            let _ = out_reader.join();
            let _ = err_reader.join();
            let outcome = match tree.wait_with(|| child.wait()) {
                Ok(status) => match status.code() {
                    Some(code) => RunOutcome::Exited(code),
                    None => RunOutcome::Signaled(exit_signal(&status)),
                },
                Err(e) => RunOutcome::Failed(format!("cannot wait for the script: {}", e)),
            };
            let output = (!streaming).then(|| {
                let bytes = collected
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                String::from_utf8_lossy(&bytes).into_owned()
            });
            let _ = events.send(Event::Run(RunEvent::Finished {
                run,
                output,
                outcome,
            }));
        })
        .map_err(|e| format!("cannot start the run waiter: {}", e))?;
    Ok(())
}

#[cfg(unix)]
fn exit_signal(status: &std::process::ExitStatus) -> i32 {
    use std::os::unix::process::ExitStatusExt;
    status.signal().unwrap_or(0)
}

#[cfg(not(unix))]
fn exit_signal(_status: &std::process::ExitStatus) -> i32 {
    0
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use crate::tests::HANG_BOUND;
    use crossbeam_channel::{unbounded, Receiver};
    use std::time::Instant;

    fn executor() -> (Executor, Receiver<Event>) {
        let (tx, rx) = unbounded();
        (Executor::new(tx, Shell::Sh, Vec::new(), Vec::new()), rx)
    }

    fn spec(script: &[&str], mode: ExecutionMode) -> RunSpec {
        RunSpec {
            origin: RunOrigin::BoxScript { box_id: "b".into() },
            target_box: "b".into(),
            script: script.iter().map(|s| s.to_string()).collect(),
            mode,
            terminal_size: Some(TerminalSize { rows: 10, cols: 40 }),
            environment: Vec::new(),
        }
    }

    /// The next event, failing as hung, naming `what` the test waits for, when none comes
    /// within the hang bound.
    fn next_event(rx: &Receiver<Event>, what: &str) -> Event {
        rx.recv_timeout(HANG_BOUND).unwrap_or_else(|_| {
            panic!(
                "timed out after {:?} waiting for {}: the run hangs",
                HANG_BOUND, what
            )
        })
    }

    /// Collects a script run's streamed lines, its collected output and its outcome.
    fn finished(rx: &Receiver<Event>) -> (Vec<String>, Option<String>, RunOutcome) {
        let mut lines = Vec::new();
        loop {
            match next_event(rx, "the script's end") {
                Event::Run(RunEvent::Output { text, .. }) => lines.push(text),
                Event::Run(RunEvent::Finished {
                    output, outcome, ..
                }) => return (lines, output, outcome),
                _ => {}
            }
        }
    }

    /// A PTY run watched as the runtime watches it: its terminal is looked at after each update,
    /// which lets the next update be reported. Each wait ends at what it waits for, and fails,
    /// naming it, when the run ends without it or `bound` passes without it.
    struct PtyWatch<'a> {
        executor: &'a Executor,
        events: &'a Receiver<Event>,
        run: RunId,
        bound: Duration,
        outcome: Option<RunOutcome>,
    }

    impl<'a> PtyWatch<'a> {
        fn new(
            executor: &'a Executor,
            events: &'a Receiver<Event>,
            run: RunId,
            bound: Duration,
        ) -> Self {
            PtyWatch {
                executor,
                events,
                run,
                bound,
                outcome: None,
            }
        }

        /// What the terminal shows, without colours and other escape codes. Taking it clears
        /// the session's update flag, so output after it is reported again.
        fn screen(&self) -> String {
            let snapshot = self
                .executor
                .get(self.run)
                .expect("the run is known until the test forgets it")
                .pty()
                .expect("the run is a PTY run")
                .snapshot();
            crate::utils::strip_ansi_codes(&snapshot.lines.join("\n"))
        }

        /// Waits until the terminal shows `text`, which is `what` the program prints.
        fn wait_for_text(&mut self, what: &str, text: &str) {
            let deadline = Instant::now() + self.bound;
            loop {
                let shown = self.screen();
                if shown.contains(text) {
                    return;
                }
                if let Some(outcome) = &self.outcome {
                    panic!(
                        "the run ended ({:?}) before {} (\"{}\") was shown; \
                         its terminal shows:\n{}",
                        outcome, what, text, shown
                    );
                }
                self.next_until(deadline, what, &shown);
            }
        }

        /// Waits until the run has ended and returns how.
        fn wait_for_end(&mut self) -> RunOutcome {
            let deadline = Instant::now() + self.bound;
            loop {
                if let Some(outcome) = &self.outcome {
                    return outcome.clone();
                }
                let shown = self.screen();
                self.next_until(deadline, "the run's end", &shown);
            }
        }

        fn next_until(&mut self, deadline: Instant, what: &str, shown: &str) {
            match self.events.recv_deadline(deadline) {
                Ok(Event::Run(RunEvent::Finished { run, outcome, .. })) if run == self.run => {
                    self.outcome = Some(outcome)
                }
                Ok(_) => {}
                Err(_) => panic!(
                    "timed out after {:?} waiting for {}: the run hangs; its terminal shows:\n{}",
                    self.bound, what, shown
                ),
            }
        }
    }

    #[test]
    fn immediate_run_keeps_multiline_scripts_intact_and_reports_exit_code() {
        let (mut ex, rx) = executor();
        let id = ex
            .start(
                spec(
                    &["for i in 1 2; do", "  echo line $i", "done", "exit 3"],
                    ExecutionMode::Immediate,
                ),
                "s".into(),
            )
            .unwrap();
        let (lines, output, outcome) = finished(&rx);
        assert!(lines.is_empty());
        assert_eq!(output.as_deref(), Some("line 1\nline 2\n"));
        assert_eq!(outcome, RunOutcome::Exited(3));
        assert!(ex.finish(id).is_some());
    }

    #[test]
    fn thread_run_streams_lines_as_they_arrive() {
        let (mut ex, rx) = executor();
        ex.start(spec(&["echo a; echo b"], ExecutionMode::Thread), "s".into())
            .unwrap();
        let (lines, output, outcome) = finished(&rx);
        assert_eq!(lines.concat(), "a\nb\n");
        assert!(output.is_none());
        assert_eq!(outcome, RunOutcome::Exited(0));
    }

    #[test]
    fn invalid_utf8_output_is_shown_with_replacement_characters() {
        let (mut ex, rx) = executor();
        ex.start(
            spec(&["printf 'ok \\377\\n'"], ExecutionMode::Immediate),
            "s".into(),
        )
        .unwrap();
        let (_, output, _) = finished(&rx);
        assert_eq!(output.as_deref(), Some("ok \u{fffd}\n"));
    }

    #[test]
    fn stopping_a_run_kills_its_background_children_too() {
        let (mut ex, rx) = executor();
        let marker = tempfile::NamedTempFile::new().unwrap();
        let path = marker.path().display().to_string();
        let script = format!(
            "(sleep 30; echo late > '{}') & echo started; sleep 30",
            path
        );
        let id = ex
            .start(spec(&[&script], ExecutionMode::Thread), "s".into())
            .unwrap();
        // The background child is started once the script says so.
        loop {
            if let Event::Run(RunEvent::Output { text, .. }) =
                next_event(&rx, "the script to start its background child")
            {
                assert_eq!(text, "started\n");
                break;
            }
        }
        let pgid = ex.get(id).unwrap().pid();
        ex.stop(id);
        let (_, _, outcome) = finished(&rx);
        assert_eq!(outcome, RunOutcome::Signaled(libc::SIGTERM));
        // Nothing of the process group is left once its killed members are reaped: signal 0
        // then fails with ESRCH.
        let deadline = Instant::now() + HANG_BOUND;
        while unsafe { libc::killpg(pgid as i32, 0) } == 0 {
            assert!(
                Instant::now() < deadline,
                "process group {} survived its run's stop for {:?}",
                pgid,
                HANG_BOUND
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::ESRCH)
        );
        assert_eq!(std::fs::read_to_string(marker.path()).unwrap(), "");
    }

    #[test]
    fn pty_run_renders_terminal_output_and_answers_cursor_queries() {
        let (mut ex, rx) = executor();
        // `dd` waits for the cursor position reply to the DSR query; without a reply the
        // script would hang until the hang bound.
        let id = ex
            .start(
                spec(
                    &[
                        // Raw input without echo first, so the reply is neither shown nor held
                        // back until a newline. The reply is ESC [ 1 ; 3 R: six bytes.
                        "stty -icanon -echo",
                        "printf 'hi\\033[6n'",
                        "reply=$(dd bs=1 count=6 2>/dev/null)",
                        "stty icanon echo",
                        "printf ' got:%s' \"${reply#??}\"",
                    ],
                    ExecutionMode::Pty,
                ),
                "s".into(),
            )
            .unwrap();
        let mut watch = PtyWatch::new(&ex, &rx, id, HANG_BOUND);
        watch.wait_for_text("the run's output", "hi");
        watch.wait_for_text("the answer to its cursor position query", "hi got:1;3");
        assert_eq!(watch.wait_for_end(), RunOutcome::Exited(0));
    }

    #[test]
    #[should_panic(expected = "timed out after 1s waiting for the run's output: the run hangs")]
    fn a_wait_on_a_terminal_program_that_never_prints_fails_naming_what_it_waited_for() {
        let (mut ex, rx) = executor();
        let id = ex
            .start(spec(&["exec sleep 30"], ExecutionMode::Pty), "s".into())
            .unwrap();
        // The program prints nothing whatever the bound, so a short one is enough.
        PtyWatch::new(&ex, &rx, id, Duration::from_secs(1)).wait_for_text("the run's output", "hi");
    }

    #[test]
    #[should_panic(
        expected = "the run ended (Exited(0)) before the run's output (\"hi\") was shown"
    )]
    fn a_wait_on_a_terminal_program_that_ends_without_printing_fails_at_its_end() {
        let (mut ex, rx) = executor();
        let id = ex
            .start(spec(&["exit 0"], ExecutionMode::Pty), "s".into())
            .unwrap();
        PtyWatch::new(&ex, &rx, id, HANG_BOUND).wait_for_text("the run's output", "hi");
    }
}
