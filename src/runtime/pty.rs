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
    master: Box<dyn MasterPty + Send>,
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
    /// sends [`RunEvent::TerminalUpdated`] when the screen changes and [`RunEvent::Finished`]
    /// when the process has exited and its output is fully read.
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
        // when the child exits.
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

        let thread_emulator = emulator.clone();
        let thread_writer = writer.clone();
        let thread_dirty = dirty.clone();
        let thread_tree = tree.clone();
        std::thread::Builder::new()
            .name(format!("pty-{}", run))
            .spawn(move || {
                let mut buffer = [0u8; 16 * 1024];
                loop {
                    match reader.read(&mut buffer) {
                        Ok(0) => break,
                        Ok(n) => {
                            let responses = {
                                let mut emulator = lock(&thread_emulator);
                                emulator.process_bytes(&buffer[..n]);
                                emulator.take_responses()
                            };
                            if !responses.is_empty() {
                                let mut writer = lock(&thread_writer);
                                for response in responses {
                                    // A process that stopped reading its terminal cannot get
                                    // the answer; its output keeps being shown regardless.
                                    let _ = writer.write_all(response.as_bytes());
                                }
                                let _ = writer.flush();
                            }
                            if !thread_dirty.swap(true, Ordering::AcqRel)
                                && events
                                    .send(Event::Run(RunEvent::TerminalUpdated { run }))
                                    .is_err()
                            {
                                // The runtime is gone; nobody will read the output.
                                break;
                            }
                        }
                        Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                        // EIO is how Linux reports that the child side closed.
                        Err(_) => break,
                    }
                }
                let outcome = match thread_tree.wait_with(|| child.wait()) {
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
                };
                // Sending fails only when the runtime has shut down.
                let _ = events.send(Event::Run(RunEvent::Finished {
                    run,
                    output: None,
                    outcome,
                }));
            })
            .map_err(|e| {
                // Nothing would read or wait for the process.
                tree.kill();
                format!("cannot start the PTY reader thread: {}", e)
            })?;

        Ok(PtySession {
            master: pair.master,
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
        self.master
            .resize(size.to_pty_size())
            .map_err(|e| format!("cannot resize the pseudo-terminal: {}", e))?;
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

/// What the runtime needs from a PTY session's terminal state at one point in time.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSnapshot {
    /// The program's clipboard writes (OSC 52) since the last snapshot: the text, or why a
    /// write cannot be copied.
    pub clipboard: Vec<Result<String, String>>,
    pub lines: Vec<String>,
    pub title: Option<String>,
    pub mouse_reporting: bool,
    pub application_cursor_keys: bool,
    pub application_keypad: bool,
}
