//! Pseudo-terminal sessions: a process running in a PTY, its terminal emulator, and the thread
//! that reads its output.

use crate::model::stream::RunId;
use crate::platform::ProcessTree;
use crate::runtime::event::{Event, RunEvent, RunOutcome};
use crate::terminal::Emulator;
use crossbeam_channel::Sender;
use portable_pty::{native_pty_system, CommandBuilder, MasterPty, PtySize};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

/// The size of a PTY in character cells.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TerminalSize {
    pub rows: u16,
    pub cols: u16,
}

impl TerminalSize {
    fn to_pty_size(self) -> PtySize {
        PtySize {
            rows: self.rows.max(1),
            cols: self.cols.max(1),
            pixel_width: 0,
            pixel_height: 0,
        }
    }
}

pub struct PtySession {
    /// The terminal's side of the pseudo-terminal. On Windows it is taken and closed when the
    /// program exits, which ends the terminal's output there.
    master: Arc<Mutex<Option<Box<dyn MasterPty + Send>>>>,
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    emulator: Arc<Mutex<Emulator>>,
    /// Set by the reader when the screen changed and the runtime has not looked at it yet;
    /// one notification is sent per transition to set.
    dirty: Arc<AtomicBool>,
    tree: ProcessTree,
    size: TerminalSize,
}

/// Locks state shared with the reader thread. If the reader panicked while holding the lock,
/// the emulator may be half-updated; the session is unusable and the runtime is told so.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            log::error!("PTY session state was poisoned by a panic in its reader thread");
            poisoned.into_inner()
        }
    }
}

impl PtySession {
    /// Starts `command` in a new PTY of `size`. Output is read on a background thread that
    /// sends [`RunEvent::TerminalUpdated`] when the screen changes; another waits for the
    /// process and its output and sends [`RunEvent::Finished`] once it has exited and its
    /// output is fully read.
    pub fn spawn(
        run: RunId,
        command: CommandBuilder,
        size: TerminalSize,
        events: Sender<Event>,
    ) -> Result<PtySession, String> {
        let pair = native_pty_system()
            .openpty(size.to_pty_size())
            .map_err(|e| format!("cannot open a pseudo-terminal: {}", e))?;
        // Everything that can fail is set up before the process starts, so a failure never
        // leaves a process behind.
        let mut reader = pair
            .master
            .try_clone_reader()
            .map_err(|e| format!("cannot read from the pseudo-terminal: {}", e))?;
        let writer: Arc<Mutex<Box<dyn Write + Send>>> =
            Arc::new(Mutex::new(pair.master.take_writer().map_err(|e| {
                format!("cannot write to the pseudo-terminal: {}", e)
            })?));
        let mut child = pair
            .slave
            .spawn_command(command)
            .map_err(|e| format!("cannot start the process in a pseudo-terminal: {}", e))?;
        // The child holds its own copy of the slave; ours must close so the reader sees EOF
        // when the child exits on Unix, and so the pseudo-console closes with the master on
        // Windows.
        drop(pair.slave);
        let Some(pid) = child.process_id() else {
            let _ = child.kill();
            let _ = child.wait();
            return Err("the PTY process has no process id".to_string());
        };
        let tree = ProcessTree::new(pid);
        let emulator = Arc::new(Mutex::new(Emulator::with_screen_size(
            usize::from(size.cols.max(1)),
            usize::from(size.rows.max(1)),
        )));
        let dirty = Arc::new(AtomicBool::new(false));
        let master = Arc::new(Mutex::new(Some(pair.master)));

        let thread_emulator = emulator.clone();
        let thread_writer = writer.clone();
        let thread_dirty = dirty.clone();
        let reading = std::thread::Builder::new()
            .name(format!("pty-{}", run))
            .spawn({
                let events = events.clone();
                move || {
                    read_output(
                        &mut *reader,
                        &thread_emulator,
                        &thread_writer,
                        &thread_dirty,
                        // Sending fails only when the runtime is gone. The dirty flag then
                        // stays set, so no update is sent again.
                        || {
                            let _ = events.send(Event::Run(RunEvent::TerminalUpdated { run }));
                        },
                    )
                }
            })
            .map_err(|e| {
                // Nothing would read or wait for the process.
                tree.kill();
                format!("cannot start the PTY reader thread: {}", e)
            })?;
        let thread_tree = tree.clone();
        #[cfg(windows)]
        let thread_master = master.clone();
        std::thread::Builder::new()
            .name(format!("pty-wait-{}", run))
            .spawn(move || {
                // The run ends after the last of its output has been shown.
                let read_to_end = move || {
                    if reading.join().is_err() {
                        log::error!("the reader of PTY {} panicked", run);
                    }
                };
                // On Unix the output ends by itself once no process has the terminal open, and
                // the terminal stays resizable for whatever still has it open after the
                // program. The program is reaped only once its output has been read: reaping it
                // first loses the end of the output on FreeBSD.
                #[cfg(unix)]
                let outcome = {
                    read_to_end();
                    outcome_of(thread_tree.wait_with(|| child.wait()))
                };
                // On Windows the console host keeps the terminal's output open until the
                // pseudo-console is closed, whether its program has exited or not; closing it
                // makes the host write out what it has not yet and end the output. It is closed
                // outside the lock, as the host may take a while.
                #[cfg(windows)]
                let outcome = {
                    let outcome = outcome_of(thread_tree.wait_with(|| child.wait()));
                    let console = lock(&thread_master).take();
                    drop(console);
                    read_to_end();
                    outcome
                };
                // Sending fails only when the runtime has shut down.
                let _ = events.send(Event::Run(RunEvent::Finished {
                    run,
                    output: None,
                    outcome,
                }));
            })
            .map_err(|e| {
                // Nothing would wait for the process; the reader ends with its output.
                tree.kill();
                format!("cannot start the PTY waiter thread: {}", e)
            })?;

        Ok(PtySession {
            master,
            writer,
            emulator,
            dirty,
            tree,
            size,
        })
    }

    pub fn process_tree(&self) -> &ProcessTree {
        &self.tree
    }

    pub fn size(&self) -> TerminalSize {
        self.size
    }

    /// Sends keyboard (or other) input to the process.
    pub fn write_input(&self, input: &[u8]) -> Result<(), String> {
        let mut writer = lock(&self.writer);
        writer
            .write_all(input)
            .and_then(|_| writer.flush())
            .map_err(|e| format!("cannot write to the PTY process: {}", e))
    }

    /// Changes the terminal size seen by the process (it receives SIGWINCH) and the emulator.
    pub fn resize(&mut self, size: TerminalSize) -> Result<(), String> {
        if size == self.size {
            return Ok(());
        }
        // A pseudo-console closed at its program's exit has no process left to tell; the
        // emulator still shows the program's last screen at the box's size.
        if let Some(master) = lock(&self.master).as_ref() {
            master
                .resize(size.to_pty_size())
                .map_err(|e| format!("cannot resize the pseudo-terminal: {}", e))?;
        }
        lock(&self.emulator)
            .resize_screen(usize::from(size.cols.max(1)), usize::from(size.rows.max(1)));
        self.size = size;
        Ok(())
    }

    /// The lines to show for the session: scrollback and screen, or only the screen while the
    /// program uses the alternate screen. Clears the dirty flag first, so output that arrives
    /// while the snapshot is taken triggers another update.
    pub fn snapshot(&self) -> TerminalSnapshot {
        self.dirty.store(false, Ordering::Release);
        let mut emulator = lock(&self.emulator);
        TerminalSnapshot {
            clipboard: emulator.take_clipboard_writes(),
            lines: emulator.display_lines(),
            title: emulator.get_current_terminal_title(),
            mouse_reporting: emulator.get_terminal_state().is_mouse_reporting_enabled(),
            application_cursor_keys: emulator.get_terminal_state().mode.cursor_key_mode,
            application_keypad: emulator.get_terminal_state().mode.keypad_mode,
        }
    }

    /// The bytes that deliver pasted `text` to the program: wrapped in the bracketed paste
    /// markers when it asked for them (mode 2004), so it can tell a paste from typing. The
    /// end marker is taken out of the text, so a paste cannot end the bracket early.
    pub fn paste_bytes(&self, text: &str) -> Vec<u8> {
        let bracketed = lock(&self.emulator)
            .get_terminal_state()
            .mode
            .bracketed_paste_mode;
        if bracketed {
            format!("\x1b[200~{}\x1b[201~", text.replace("\x1b[201~", "")).into_bytes()
        } else {
            text.as_bytes().to_vec()
        }
    }

    /// Whether the program switched the arrow keys to application mode (DECCKM).
    pub fn application_cursor_keys(&self) -> bool {
        lock(&self.emulator)
            .get_terminal_state()
            .mode
            .cursor_key_mode
    }

    /// Whether the program asked to receive mouse events.
    pub fn mouse_reporting(&self) -> bool {
        lock(&self.emulator)
            .get_terminal_state()
            .is_mouse_reporting_enabled()
    }

    /// Encodes a mouse event for the program, in the reporting mode it enabled; `None` when the
    /// program did not ask for mouse events. `column` and `row` are 0-based within the terminal.
    pub fn mouse_report(
        &self,
        event_type: crate::terminal::MouseEventType,
        button: Option<crate::terminal::MouseButton>,
        column: usize,
        row: usize,
        modifiers: crate::terminal::MouseModifiers,
    ) -> Option<String> {
        lock(&self.emulator)
            .get_terminal_state()
            .generate_mouse_report(event_type, column, row, button, modifiers)
    }
}

/// Reads the terminal's output into `emulator` until the output ends, answering the program's
/// queries through `writer` and calling `notify` each time the screen changes while `dirty` is
/// clear. The output is read to its end even when nobody is notified any more: on Windows the
/// console host cannot close while its output is not read.
fn read_output(
    reader: &mut dyn Read,
    emulator: &Mutex<Emulator>,
    writer: &Mutex<Box<dyn Write + Send>>,
    dirty: &AtomicBool,
    notify: impl Fn(),
) {
    let mut buffer = [0u8; 16 * 1024];
    loop {
        match reader.read(&mut buffer) {
            Ok(0) => return,
            Ok(n) => {
                let responses = {
                    let mut emulator = lock(emulator);
                    emulator.process_bytes(&buffer[..n]);
                    emulator.take_responses()
                };
                if !responses.is_empty() {
                    let mut writer = lock(writer);
                    for response in responses {
                        // A process that stopped reading its terminal cannot get the answer;
                        // its output keeps being shown regardless.
                        let _ = writer.write_all(response.as_bytes());
                    }
                    let _ = writer.flush();
                }
                if !dirty.swap(true, Ordering::AcqRel) {
                    notify();
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
            // EIO is how Linux reports that the child side closed, a broken pipe how Windows
            // reports that the console host closed.
            Err(_) => return,
        }
    }
}

/// How a terminal program ended, from waiting for it.
fn outcome_of(waited: std::io::Result<portable_pty::ExitStatus>) -> RunOutcome {
    match waited {
        Ok(status) => {
            if status.success() {
                RunOutcome::Exited(0)
            } else {
                match status.signal() {
                    Some(signal) => RunOutcome::Failed(format!("ended by {}", signal)),
                    None => RunOutcome::Exited(status.exit_code() as i32),
                }
            }
        }
        Err(e) => RunOutcome::Failed(format!("cannot wait for the process: {}", e)),
    }
}

/// What the runtime needs from a PTY session's terminal state at one point in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSnapshot {
    /// Text the program put on the clipboard (OSC 52) since the last snapshot.
    pub clipboard: Vec<String>,
    pub lines: Vec<String>,
    pub title: Option<String>,
    pub mouse_reporting: bool,
    pub application_cursor_keys: bool,
    pub application_keypad: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::HANG_BOUND;
    use crossbeam_channel::{unbounded, Receiver};
    use std::time::{Duration, Instant};

    /// A terminal program that runs `script` in the platform's shell.
    fn program(script: &str) -> CommandBuilder {
        let mut command = if cfg!(windows) {
            let mut command = CommandBuilder::new("powershell");
            command.args(["-NoLogo", "-NoProfile", "-Command"]);
            command
        } else {
            let mut command = CommandBuilder::new("sh");
            command.arg("-c");
            command
        };
        command.arg(script);
        command.cwd(std::env::current_dir().unwrap());
        command
    }

    fn start(script: &str) -> (PtySession, Receiver<Event>) {
        let (tx, rx) = unbounded();
        let session = PtySession::spawn(
            RunId(1),
            program(script),
            TerminalSize { rows: 10, cols: 60 },
            tx,
        )
        .unwrap();
        (session, rx)
    }

    /// Waits, as the runtime does, looking at the terminal after each update, until the run
    /// reports its end, and returns how it ended with what its terminal showed then.
    fn wait_for_end(session: &PtySession, events: &Receiver<Event>) -> (RunOutcome, String) {
        let deadline = Instant::now() + HANG_BOUND;
        loop {
            match events.recv_deadline(deadline) {
                Ok(Event::Run(RunEvent::TerminalUpdated { .. })) => {
                    session.snapshot();
                }
                Ok(Event::Run(RunEvent::Finished { outcome, .. })) => {
                    let shown = session.snapshot().lines.join("\n");
                    return (outcome, crate::utils::strip_ansi_codes(&shown));
                }
                Ok(_) => {}
                Err(_) => panic!(
                    "the terminal run reported no end within {:?}; its terminal shows:\n{}",
                    HANG_BOUND,
                    session.snapshot().lines.join("\n")
                ),
            }
        }
    }

    #[test]
    fn a_terminal_run_ends_when_its_program_exits_with_its_code_and_its_last_output_shown() {
        let script = if cfg!(windows) {
            "Write-Output 'last words'; exit 3"
        } else {
            "echo 'last words'; exit 3"
        };
        let (session, events) = start(script);
        let (outcome, shown) = wait_for_end(&session, &events);
        assert_eq!(
            outcome,
            RunOutcome::Exited(3),
            "its terminal shows:\n{}",
            shown
        );
        assert!(
            shown.contains("last words"),
            "the program's last output is not shown:\n{}",
            shown
        );
    }

    #[test]
    fn a_stopped_terminal_run_ends() {
        let script = if cfg!(windows) {
            "Write-Output started; Start-Sleep 600"
        } else {
            "echo started; sleep 600"
        };
        let (session, events) = start(script);
        let deadline = Instant::now() + HANG_BOUND;
        while !crate::utils::strip_ansi_codes(&session.snapshot().lines.join("\n"))
            .contains("started")
        {
            match events.recv_deadline(deadline) {
                Ok(Event::Run(RunEvent::Finished { outcome, .. })) => {
                    panic!("the program ended ({:?}) before it was stopped", outcome)
                }
                Ok(_) => {}
                Err(_) => panic!("the program printed nothing within {:?}", HANG_BOUND),
            }
        }
        session.process_tree().terminate(Duration::from_secs(5));
        let (outcome, shown) = wait_for_end(&session, &events);
        assert_ne!(
            outcome,
            RunOutcome::Exited(0),
            "its terminal shows:\n{}",
            shown
        );
    }
}
