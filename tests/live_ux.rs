//! End-to-end tests: the real `boxmux` binary in a pseudo-terminal (a ConPTY on Windows),
//! driven with keyboard and SGR mouse input, its screen read back through a small terminal
//! emulator. Scripts in the configurations are written for the platform's default shell.

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthChar;

const WIDTH: u16 = 100;
const HEIGHT: u16 = 30;
/// How long a test waits for something boxmux or a script does before it calls it hung and
/// fails, naming what it waited for. It bounds a hang, not the work: every wait ends at what it
/// waits for, while on a machine loaded well past its cores starting one process can take
/// seconds, and PowerShell takes seconds to start on a cold Windows machine. Programs a test
/// needs running sleep 600 seconds, longer than this, so only being stopped ends them sooner.
const TIMEOUT: Duration = Duration::from_secs(120);

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_boxmux")
}

/// A temporary directory holding one test's configuration, socket, log and marker files.
struct Fixture {
    tmp: TempDir,
}

impl Fixture {
    fn new() -> Fixture {
        let tmp = tempfile::tempdir().expect("temporary directory");
        fs::create_dir(tmp.path().join("markers")).unwrap();
        Fixture { tmp }
    }

    fn path(&self, name: &str) -> PathBuf {
        self.tmp.path().join(name)
    }

    fn marker(&self, name: &str) -> PathBuf {
        self.path("markers").join(name)
    }

    /// This test's control socket: a file on Unix, a pipe name on Windows.
    fn socket(&self) -> String {
        if cfg!(windows) {
            let unique = self
                .tmp
                .path()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned();
            format!("boxmux-test-{}", unique)
        } else {
            self.path("boxmux.sock").to_string_lossy().into_owned()
        }
    }

    fn write_config(&self, yaml: &str) -> PathBuf {
        let path = self.path("app.yaml");
        fs::write(&path, yaml).unwrap();
        path
    }

    fn wait_for_marker(&self, name: &str) -> String {
        let path = self.marker(name);
        let start = Instant::now();
        loop {
            if let Ok(text) = fs::read_to_string(&path) {
                if !text.is_empty() {
                    return text;
                }
            }
            assert!(
                start.elapsed() < TIMEOUT,
                "marker {} was not written\nlog:\n{}",
                name,
                self.log()
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn assert_no_marker_after(&self, name: &str, wait: Duration) {
        thread::sleep(wait);
        assert!(!self.marker(name).exists(), "marker {} was written", name);
    }

    fn log(&self) -> String {
        fs::read_to_string(self.path("boxmux.log")).unwrap_or_default()
    }

    /// Runs `boxmux --socket <socket> <args>` as a control client.
    fn control(&self, args: &[&str]) -> std::process::Output {
        let mut child = Command::new(binary())
            .arg("--socket")
            .arg(self.socket())
            .args(args)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("the control client starts");
        let start = Instant::now();
        loop {
            if child.try_wait().unwrap().is_some() {
                return child.wait_with_output().unwrap();
            }
            if start.elapsed() > TIMEOUT {
                child.kill().unwrap();
                panic!(
                    "`boxmux {}` did not finish: the socket hangs",
                    args.join(" ")
                );
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
}

/// A running boxmux in a pseudo-terminal.
struct LiveBoxMux {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    /// The terminal's side of the PTY, which resizes the terminal; closed once boxmux has
    /// exited (see [`LiveBoxMux::wait_for_exit`]).
    master: Option<Box<dyn portable_pty::MasterPty + Send>>,
    writer: Box<dyn Write + Send>,
    output: Receiver<Vec<u8>>,
    /// Every byte boxmux wrote to the terminal.
    raw: Vec<u8>,
    screen: Screen,
    log: PathBuf,
    _reader: thread::JoinHandle<()>,
}

impl LiveBoxMux {
    fn spawn(fixture: &Fixture, config: &Path) -> LiveBoxMux {
        Self::spawn_with(fixture, config, &[])
    }

    fn spawn_with(fixture: &Fixture, config: &Path, extra: &[&str]) -> LiveBoxMux {
        Self::spawn_in(
            fixture,
            config,
            extra,
            Screen::new(WIDTH as usize, HEIGHT as usize),
        )
    }

    /// Starts boxmux in a terminal that draws like `screen`, with a fixed theme: boxmux does not
    /// ask the terminal for its colors, which this test terminal would not answer.
    fn spawn_in(fixture: &Fixture, config: &Path, extra: &[&str], screen: Screen) -> LiveBoxMux {
        Self::start(fixture, config, extra, screen, true)
    }

    /// Starts boxmux in a terminal that draws like `screen`, with no theme given: boxmux asks the
    /// terminal for its background color, as the handlebar's BoxMux does, and waits up to a
    /// second for an answer this test terminal gives only when the test does.
    #[cfg(unix)]
    fn spawn_asking_background(fixture: &Fixture, config: &Path, screen: Screen) -> LiveBoxMux {
        Self::start(fixture, config, &[], screen, false)
    }

    fn start(
        fixture: &Fixture,
        config: &Path,
        extra: &[&str],
        screen: Screen,
        fixed_theme: bool,
    ) -> LiveBoxMux {
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: screen.height as u16,
                cols: screen.width as u16,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("a PTY");
        let mut command = CommandBuilder::new(binary());
        command.arg(config);
        command.args(extra);
        if fixed_theme {
            command.arg("--dark");
        }
        command.args(["--frame-delay", "10", "--log-level", "debug", "--log-file"]);
        command.arg(fixture.path("boxmux.log"));
        command.arg("--socket");
        command.arg(fixture.socket());
        command.cwd(fixture.tmp.path());
        command.env("TERM", "xterm-256color");
        let child = pair.slave.spawn_command(command).expect("boxmux starts");
        drop(pair.slave);
        let writer = pair.master.take_writer().unwrap();
        let mut reader = pair.master.try_clone_reader().unwrap();
        let (tx, rx) = mpsc::channel();
        let reader_thread = thread::spawn(move || {
            let mut buf = [0u8; 8192];
            while let Ok(n) = reader.read(&mut buf) {
                if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                    break;
                }
            }
        });
        LiveBoxMux {
            child,
            master: Some(pair.master),
            writer,
            output: rx,
            raw: Vec::new(),
            screen,
            log: fixture.path("boxmux.log"),
            _reader: reader_thread,
        }
    }

    /// Gives the terminal `cols` x `rows` cells, as a terminal window or pane resized by its
    /// user does: the PTY reports the new size to boxmux, and the screen keeps what it shows as
    /// a terminal does (see [`Screen::resize`]).
    #[cfg(unix)]
    fn resize(&mut self, cols: u16, rows: u16) {
        // What boxmux wrote before the resize is drawn at the old size.
        self.pump(Duration::from_millis(30));
        self.master
            .as_ref()
            .expect("the terminal is open while boxmux runs")
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("the PTY resizes");
        self.screen.resize(usize::from(cols), usize::from(rows));
    }

    /// Waits until boxmux has written its first frame, and gives the terminal `cols` x `rows`
    /// cells before the screen shows any of it: the frame, drawn for the size the terminal had,
    /// reaches a screen of the new size. A terminal multiplexer does this when it resizes a pane
    /// while the pane's output still waits to be read.
    #[cfg(unix)]
    fn resize_with_first_frame_in_flight(&mut self, cols: u16, rows: u16) {
        let mut held = Vec::new();
        let start = Instant::now();
        while !contains(&held, END_SYNC) {
            assert!(
                start.elapsed() < TIMEOUT,
                "boxmux wrote no frame: {:?}",
                String::from_utf8_lossy(&held)
            );
            if let Ok(bytes) = self.output.recv_timeout(Duration::from_millis(30)) {
                held.extend(bytes);
            }
        }
        self.master
            .as_ref()
            .expect("the terminal is open while boxmux runs")
            .resize(PtySize {
                rows,
                cols,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("the PTY resizes");
        self.screen.resize(usize::from(cols), usize::from(rows));
        self.accept(held);
    }

    #[cfg(unix)]
    fn pid(&self) -> i32 {
        self.child.process_id().expect("boxmux has a pid") as i32
    }

    fn pump(&mut self, wait: Duration) {
        if let Ok(bytes) = self.output.recv_timeout(wait) {
            self.accept(bytes);
            while let Ok(bytes) = self.output.try_recv() {
                self.accept(bytes);
            }
        }
    }

    fn accept(&mut self, bytes: Vec<u8>) {
        self.screen.apply(&bytes);
        let before = self.raw.len();
        self.raw.extend(bytes);
        // Answer cursor position requests like a terminal. ConPTY sends one when it starts and
        // runs nothing until it gets the answer. A request may be split between two reads.
        const REQUEST: &[u8] = b"\x1b[6n";
        let from = before.saturating_sub(REQUEST.len() - 1);
        let requests = self.raw[from..]
            .windows(REQUEST.len())
            .filter(|window| *window == REQUEST)
            .count();
        for _ in 0..requests {
            let reply = format!("\x1b[{};{}R", self.screen.y + 1, self.screen.x + 1);
            self.writer.write_all(reply.as_bytes()).unwrap();
            self.writer.flush().unwrap();
        }
    }

    /// The screen, the end of what boxmux wrote, whether it is still running, and its log: what
    /// a failed wait needs to show to be diagnosed.
    fn report(&mut self) -> String {
        let tail = &self.raw[self.raw.len().saturating_sub(1500)..];
        format!(
            "screen:\n{}\nlinks:\n{}\nlast output ({} bytes in all): {}\nexited: {:?}\nlog:\n{}",
            self.screen.text(),
            self.screen.link_runs(),
            self.raw.len(),
            String::from_utf8_lossy(tail).escape_debug(),
            self.child.try_wait().ok().flatten(),
            fs::read_to_string(&self.log).unwrap_or_default()
        )
    }

    fn wait_for_text(&mut self, needle: &str) -> (usize, usize) {
        let start = Instant::now();
        loop {
            self.pump(Duration::from_millis(30));
            if let Some(at) = self.screen.find(needle) {
                return at;
            }
            if start.elapsed() >= TIMEOUT {
                panic!("{:?} did not appear\n{}", needle, self.report());
            }
        }
    }

    /// Waits until `done` holds for the screen; `what` names it in the failure.
    #[cfg(unix)]
    fn wait_until(&mut self, what: &str, done: impl Fn(&Screen) -> bool) {
        let start = Instant::now();
        loop {
            self.pump(Duration::from_millis(30));
            if done(&self.screen) {
                return;
            }
            if start.elapsed() >= TIMEOUT {
                panic!("{} did not happen\n{}", what, self.report());
            }
        }
    }

    /// Waits until `problems` finds nothing wrong with the screen; `what` names what the screen
    /// should show, and the failure lists what was still wrong.
    fn wait_for_screen(&mut self, what: &str, problems: impl Fn(&Screen) -> Vec<String>) {
        let start = Instant::now();
        loop {
            self.pump(Duration::from_millis(30));
            let found = problems(&self.screen);
            if found.is_empty() {
                return;
            }
            if start.elapsed() >= TIMEOUT {
                panic!(
                    "the screen does not show {}:\n{}\n{}",
                    what,
                    found.join("\n"),
                    self.report()
                );
            }
        }
    }

    fn wait_until_gone(&mut self, needle: &str) {
        let start = Instant::now();
        loop {
            self.pump(Duration::from_millis(30));
            if self.screen.find(needle).is_none() {
                return;
            }
            if start.elapsed() >= TIMEOUT {
                panic!("{:?} is still shown\n{}", needle, self.report());
            }
        }
    }

    fn send(&mut self, bytes: &str) {
        self.writer.write_all(bytes.as_bytes()).unwrap();
        self.writer.flush().unwrap();
    }

    fn press(&mut self, x: usize, y: usize) {
        self.send(&format!("\x1b[<0;{};{}M", x + 1, y + 1));
    }

    fn drag_to(&mut self, x: usize, y: usize) {
        self.send(&format!("\x1b[<32;{};{}M", x + 1, y + 1));
    }

    fn release(&mut self, x: usize, y: usize) {
        self.send(&format!("\x1b[<0;{};{}m", x + 1, y + 1));
    }

    fn click(&mut self, x: usize, y: usize) {
        self.press(x, y);
        self.release(x, y);
    }

    fn click_text(&mut self, text: &str) {
        let (x, y) = self.wait_for_text(text);
        self.click(x, y);
    }

    /// Waits for boxmux to exit, reads everything it wrote, and returns whether it exited
    /// successfully.
    fn wait_for_exit(&mut self) -> bool {
        let start = Instant::now();
        let status = loop {
            self.pump(Duration::from_millis(20));
            if let Some(status) = self.child.try_wait().unwrap() {
                break status;
            }
            assert!(start.elapsed() < TIMEOUT, "boxmux did not exit");
        };
        // What boxmux wrote just before exiting may still be on its way. On Windows the console
        // host between boxmux and the test writes it out and ends the output only when the
        // pseudo-console is closed, which closing the terminal's side does; on Unix closing it
        // changes nothing, as the reader holds its own copy.
        drop(self.master.take());
        // The output ends, and the reader with it, once no process has the terminal open: the
        // scripts boxmux runs read no terminal and write to pipes.
        loop {
            match self.output.recv_timeout(Duration::from_millis(20)) {
                Ok(bytes) => self.accept(bytes),
                Err(mpsc::RecvTimeoutError::Disconnected) => return status.success(),
                Err(mpsc::RecvTimeoutError::Timeout) => assert!(
                    start.elapsed() < TIMEOUT,
                    "boxmux exited, but its terminal's output never ended\n{}",
                    self.report()
                ),
            }
        }
    }
}

impl Drop for LiveBoxMux {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A script that writes `text` to `markers/<name>`.
fn write_marker(name: &str, text: &str) -> String {
    if cfg!(windows) {
        format!(
            "Set-Content -NoNewline -Path markers/{} -Value {}",
            name, text
        )
    } else {
        format!("printf {} > markers/{}", text, name)
    }
}

fn menu() -> String {
    MENU.replace("WRITE_BUILD", &write_marker("build", "build"))
        .replace("WRITE_TEST", &write_marker("test", "test"))
}

const MENU: &str = r#"
app:
  layouts:
    - id: main
      title: Live
      root: true
      children:
        - id: menu
          title: Commands
          position: {x1: "2", y1: "2", x2: "42", y2: "14"}
          tab_order: 1
          choices:
            - id: build
              content: Build Now
              script: ["WRITE_BUILD"]
            - id: test
              content: Run Tests
              script: ["WRITE_TEST"]
            - id: show
              content: Show Output
              redirect_output: output
              script: ["echo REDIRECTED_PAYLOAD"]
        - id: output
          title: Output
          position: {x1: "44", y1: "2", x2: "98", y2: "14"}
          content: READY
"#;

#[test]
fn clicking_a_choice_label_runs_exactly_that_choice() {
    let fixture = Fixture::new();
    let config = fixture.write_config(&menu());
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("Run Tests");
    app.click_text("Build Now");
    assert_eq!(fixture.wait_for_marker("build"), "build");
    fixture.assert_no_marker_after("test", Duration::from_millis(300));
}

#[test]
fn clicking_beside_a_choice_label_runs_nothing() {
    let fixture = Fixture::new();
    let config = fixture.write_config(&menu());
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    let (x, y) = app.wait_for_text("Build Now");
    app.click(x + "Build Now".len() + 4, y);
    fixture.assert_no_marker_after("build", Duration::from_millis(700));
}

/// A script that writes the answer its confirmation was given to `markers/<name>`.
fn write_option_marker(name: &str) -> String {
    if cfg!(windows) {
        format!(
            "Set-Content -NoNewline -Path markers/{} -Value $env:BOXMUX_OPTION",
            name
        )
    } else {
        format!("printf %s \"$BOXMUX_OPTION\" > markers/{}", name)
    }
}

const CONFIRM_MENU: &str = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: menu
          title: Commands
          position: {x1: "2", y1: "2", x2: "42", y2: "8"}
          tab_order: 1
          choices:
            - id: deploy
              content: Deploy
              script: [WRITE_OPTION]
              confirm:
                text: Deploy where?
                options: [{id: staging, content: Staging}, {id: prod, content: Production}, {id: abort, content: Not now}]
                cancel: abort
"#;

#[test]
fn a_choice_asks_first_runs_nothing_on_escape_and_runs_with_the_answer_clicked() {
    let fixture = Fixture::new();
    let script = serde_json::to_string(&write_option_marker("deploy")).unwrap();
    let config = fixture.write_config(&CONFIRM_MENU.replace("WRITE_OPTION", &script));
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.click_text("Deploy");
    app.wait_for_text("Deploy where?");
    app.wait_for_text(" Staging    Production    Not now ");
    app.send("\x1b");
    app.wait_until_gone("Deploy where?");
    fixture.assert_no_marker_after("deploy", Duration::from_millis(700));

    app.click_text("Deploy");
    app.wait_for_text("Deploy where?");
    app.click_text("Production");
    assert_eq!(fixture.wait_for_marker("deploy"), "prod");
    app.wait_until_gone("Deploy where?");
}

#[test]
fn redirected_output_opens_a_tab_shown_when_clicked_that_its_close_button_closes() {
    let fixture = Fixture::new();
    let config = fixture.write_config(&menu());
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.click_text("Show Output");
    // The output's tab opens beside the box's content, which stays in view.
    let tab = app.wait_for_text("Show Output ×");
    assert!(app.screen.find("READY").is_some());
    assert!(app.screen.find("REDIRECTED_PAYLOAD").is_none());
    app.click(tab.0, tab.1);
    app.wait_for_text("REDIRECTED_PAYLOAD");
    let (x, y) = app.wait_for_text("×");
    app.click(x, y);
    app.wait_until_gone("REDIRECTED_PAYLOAD");
    app.wait_for_text("READY");
}

#[test]
fn control_commands_answer_and_change_the_screen() {
    let fixture = Fixture::new();
    let config = fixture.write_config(&menu());
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("READY");

    let output = fixture.control(&["replace_box_content", "output", "FROM_THE_SOCKET"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    app.wait_for_text("FROM_THE_SOCKET");

    let output = fixture.control(&["replace_box_content", "nonexistent", "x"]);
    assert!(!output.status.success(), "an unknown box is an error");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("nonexistent"), "{}", stderr);

    let output = fixture.control(&["query_pty_status", "menu"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stdout.is_empty() || !stderr.is_empty(),
        "query_pty_status answers"
    );
}

#[test]
fn a_file_box_follows_its_file_and_the_content_target_keeps_the_box_to_one_tab() {
    let fixture = Fixture::new();
    fs::write(fixture.path("status.txt"), "STATUS_ONE\n").unwrap();
    let config = fixture.write_config(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: status
          title: Status
          position: {x1: "2", y1: "2", x2: "48", y2: "14"}
          file: status.txt
        - id: note
          title: Note
          position: {x1: "50", y1: "2", x2: "98", y2: "14"}
          content: NOTE_ONE
"#,
    );
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("STATUS_ONE");

    // No refresh_interval: only the platform telling boxmux the file changed redraws it.
    fs::write(fixture.path("status.txt"), "STATUS_TWO\n").unwrap();
    app.wait_for_text("STATUS_TWO");
    fs::remove_file(fixture.path("status.txt")).unwrap();
    app.wait_for_text("Waiting for this file to appear:");

    let output = fixture.control(&[
        "replace_box_content",
        "note",
        "NOTE_TWO",
        "--target",
        "content",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    app.wait_for_text("NOTE_TWO");
    assert!(
        app.screen.find("Socket").is_none() && app.screen.find("NOTE_ONE").is_none(),
        "the note's own content was replaced, with no tab added\n{}",
        app.screen.text()
    );

    let output = fixture.control(&["replace_box_content", "note", "x", "--target", "title"]);
    assert!(!output.status.success(), "an unknown target is an error");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("title"), "{}", stderr);
}

#[test]
fn keys_typed_into_a_focused_terminal_reach_the_program() {
    let fixture = Fixture::new();
    let config = fixture.write_config(
        &r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: shell
          title: Terminal
          position: {x1: "0", y1: "0", x2: "60", y2: "12"}
          tab_order: 1
          execution_mode: Pty
          script: [SCRIPT]
"#
        .replace(
            "SCRIPT",
            if cfg!(windows) {
                r#"'$line = Read-Host "ready>"; Write-Output "got:$line"; Start-Sleep 600'"#
            } else {
                r#"'printf "ready> "; read line; echo "got:$line"; sleep 600'"#
            },
        ),
    );
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("ready>");
    app.send("abc q\r");
    app.wait_for_text("got:abc q");
}

/// A terminal program that writes its process id to `markers/term`, then reads lines and echoes
/// each as `got[<line>]`, tabs and control characters shown as `^I`, `^[`.
#[cfg(unix)]
const LINE_ECHO: &str = r#"'echo $$ > markers/term; echo ready; while IFS= read -r line; do printf "got[%s]\n" "$line" | cat -t; done'"#;

#[test]
#[cfg(unix)]
fn tab_and_shift_tab_typed_into_a_focused_terminal_reach_the_program() {
    let fixture = Fixture::new();
    let config = fixture.write_config(
        &r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: shell
          title: Terminal
          position: {x1: "0", y1: "0", x2: "60", y2: "12"}
          tab_order: 1
          execution_mode: Pty
          script: [SCRIPT]
        - id: notes
          title: Notes
          position: {x1: "62", y1: "0", x2: "98", y2: "12"}
          tab_order: 2
          content: notes
"#
        .replace("SCRIPT", LINE_ECHO),
    );
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("ready");
    // Tab and Shift+Tab as a terminal sends them, then Enter.
    app.send("a\tb\x1b[Z\r");
    app.wait_for_text("got[a^Ib^[[Z]");
}

/// The escape chord Ctrl+], as terminals send it: the control character GS.
#[cfg(unix)]
const ESCAPE_CHORD: &str = "\x1d";

#[test]
#[cfg(unix)]
fn the_escape_chord_leaves_a_focused_terminal_running_and_its_bindings_switch_the_layout() {
    let fixture = Fixture::new();
    let config = fixture.write_config(
        &r#"
app:
  hot_keys:
    "Ctrl+]": needs_you
  layouts:
    - id: main
      root: true
      on_keypress:
        "Ctrl+]": [WRITE_LEFT]
      children:
        - id: shell
          title: Terminal
          position: {x1: "0", y1: "0", x2: "60", y2: "12"}
          tab_order: 1
          execution_mode: Pty
          script: [SCRIPT]
        - id: actions
          title: Actions
          position: {x1: "62", y1: "0", x2: "98", y2: "6"}
          choices:
            - id: needs_you
              content: Back to Needs you
              script: ["'BOXMUX' switch_active_layout queue"]
    - id: queue
      children:
        - id: waiting
          title: Needs you
          position: {x1: "0", y1: "0", x2: "60", y2: "12"}
          content: QUEUE_SHOWN
"#
        .replace("SCRIPT", LINE_ECHO)
        .replace("WRITE_LEFT", &write_marker("left", "left"))
        .replace("BOXMUX", binary()),
    );
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("ready");
    let program: i32 = fixture.wait_for_marker("term").trim().parse().unwrap();
    // The terminal, the only box with a tab_order, starts with the focus.
    app.send("one\r");
    app.wait_for_text("got[one]");

    // The chord leaves the terminal and runs what is bound to it: the hot key shows the other
    // layout, and the layout's handler runs.
    app.send(ESCAPE_CHORD);
    app.wait_for_text("QUEUE_SHOWN");
    assert_eq!(fixture.wait_for_marker("left"), "left");
    assert_eq!(
        unsafe { libc::kill(program, 0) },
        0,
        "the terminal's program still runs"
    );

    // Back on the terminal's layout, the focus is out of the terminal: typing reaches no program.
    let output = fixture.control(&["switch_active_layout", "main"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    app.wait_for_text("got[one]");
    app.send("two\r");

    // Tab takes the focus back into the terminal, whose program reads on.
    app.send("\t");
    app.send("three\r");
    app.wait_for_text("got[three]");
    // The program reads its lines in order: had `two` reached it, it would have been echoed
    // before `three`, in the frame that shows `three` or an earlier one.
    assert!(
        app.screen.find("got[two]").is_none(),
        "the terminal kept the focus\n{}",
        app.report()
    );
}

#[test]
#[cfg(unix)]
fn quitting_leaves_no_process_behind() {
    let fixture = Fixture::new();
    let config = fixture.write_config(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: worker
          position: {x1: "0", y1: "0", x2: "40", y2: "10"}
          execution_mode: Thread
          script: ['sleep 600 & echo $! > markers/child', 'echo started', 'wait']
"#,
    );
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("started");
    let child: i32 = fixture.wait_for_marker("child").trim().parse().unwrap();
    app.send("q");
    assert!(app.wait_for_exit(), "q exits successfully");
    let start = Instant::now();
    while unsafe { libc::kill(child, 0) } == 0 {
        assert!(
            start.elapsed() < TIMEOUT,
            "the script's child {} survived",
            child
        );
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
fn dragging_a_box_saves_its_new_position_to_the_configuration() {
    let fixture = Fixture::new();
    let config = fixture.write_config(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: mover
          title: Mover
          position: {x1: "10", y1: "5", x2: "30", y2: "10"}
          content: drag me
"#,
    );
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("drag me");
    // The top border, away from the title.
    app.press(12, 5);
    app.drag_to(16, 7);
    app.release(16, 7);
    let start = Instant::now();
    loop {
        let text = fs::read_to_string(&config).unwrap();
        if text.contains("x1: \"14\"") {
            assert!(
                text.contains("y1: \"7\"")
                    && text.contains("x2: \"34\"")
                    && text.contains("y2: \"12\""),
                "{}",
                text
            );
            boxmux_lib::load_app_from_yaml(&config).expect("the saved configuration still loads");
            break;
        }
        assert!(
            start.elapsed() < TIMEOUT,
            "position not saved:\n{}\nlog:\n{}",
            text,
            fixture.log()
        );
        app.pump(Duration::from_millis(30));
    }
}

#[test]
#[cfg(unix)]
fn a_termination_signal_restores_the_terminal() {
    let fixture = Fixture::new();
    let config = fixture.write_config(&menu());
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("READY");
    let before = app.raw.len();
    assert_eq!(unsafe { libc::kill(app.pid(), libc::SIGTERM) }, 0);
    app.wait_for_exit();
    let after = String::from_utf8_lossy(&app.raw[before..]).to_string();
    assert!(
        after.contains("\x1b[?1049l"),
        "the alternate screen is left: {:?}",
        after
    );
    assert!(
        after.contains("\x1b[?25h"),
        "the cursor is shown again: {:?}",
        after
    );
    assert!(app.screen.autowrap, "autowrap is on again: {:?}", after);
}

#[test]
fn an_invalid_configuration_is_reported_and_exits_with_failure() {
    let fixture = Fixture::new();
    let config = fixture.write_config("app:\n  layouts:\n    - id: main\n      children:\n        - id: a\n          position: {x1: \"0\", y1: \"0\", x2: \"5\", y2: \"5\"}\n          border_color: purple\n");
    let output = Command::new(binary())
        .arg(&config)
        .arg("--socket")
        .arg(fixture.socket())
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("border_color"), "{}", stderr);
}

/// An OSC 8 hyperlink as a terminal holds it: its URI, and its `id` parameter when it has one.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Link {
    id: Option<String>,
    uri: String,
}

/// A color an SGR sequence selects: an entry of the 256-color palette (the first 16 the basic
/// and bright colors) or a 24-bit color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ink {
    Indexed(u8),
    Rgb(u8, u8, u8),
}

/// What a cell is drawn in, of what SGR sequences set: its colors (`None` the terminal's own)
/// and whether it is bold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Pen {
    fg: Option<Ink>,
    bg: Option<Ink>,
    bold: bool,
}

impl Pen {
    /// Applies the parameters of one SGR sequence, as a terminal does: `0` resets, `1` and `22`
    /// set and clear bold, and `30`–`49` and `90`–`107` set the colors, `38` and `48` with
    /// `5;n` or `2;r;g;b`. Every other parameter changes nothing this terminal keeps.
    fn apply(&mut self, params: &[usize]) {
        let byte = |k: usize| params.get(k).and_then(|&v| u8::try_from(v).ok());
        let mut i = 0;
        while i < params.len() {
            match params[i] {
                0 => *self = Pen::default(),
                1 => self.bold = true,
                22 => self.bold = false,
                n @ 30..=37 => self.fg = Some(Ink::Indexed((n - 30) as u8)),
                n @ 40..=47 => self.bg = Some(Ink::Indexed((n - 40) as u8)),
                n @ 90..=97 => self.fg = Some(Ink::Indexed((n - 90 + 8) as u8)),
                n @ 100..=107 => self.bg = Some(Ink::Indexed((n - 100 + 8) as u8)),
                39 => self.fg = None,
                49 => self.bg = None,
                code @ (38 | 48) => {
                    let (ink, used) = match params.get(i + 1) {
                        Some(5) => (byte(i + 2).map(Ink::Indexed), 2),
                        Some(2) => match (byte(i + 2), byte(i + 3), byte(i + 4)) {
                            (Some(r), Some(g), Some(b)) => (Some(Ink::Rgb(r, g, b)), 4),
                            _ => (None, params.len()),
                        },
                        _ => (None, params.len()),
                    };
                    if let Some(ink) = ink {
                        if code == 38 {
                            self.fg = Some(ink);
                        } else {
                            self.bg = Some(ink);
                        }
                    }
                    i += used;
                }
                _ => {}
            }
            i += 1;
        }
    }

    /// What erasing leaves in a cell while this pen is in force: a blank in its background, as
    /// xterm and the terminals that follow it erase.
    fn erased(self) -> Pen {
        Pen {
            bg: self.bg,
            ..Pen::default()
        }
    }
}

/// Begins a synchronized update (DEC private mode 2026): the terminal keeps showing what it
/// showed until the update ends.
const BEGIN_SYNC: &[u8] = b"\x1b[?2026h";
/// Ends a synchronized update: the terminal shows everything written since it began.
const END_SYNC: &[u8] = b"\x1b[?2026l";

fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// Enough of a terminal to read what boxmux draws: cursor movement, erasing and UTF-8 text.
///
/// It keeps text the way current terminals (WezTerm, kitty, foot) do: each cell holds one
/// grapheme cluster, and a character printed right after another joins its cell when the two
/// are one cluster (a combining mark, a variation selector, a skin tone, a zero-width joiner and
/// what it joins). A cluster's width is the sum of its characters' widths, at most two, with
/// East Asian ambiguous characters one column wide. How it draws emoji variation sequences and
/// which characters it draws narrower than Unicode says are the terminal's own choices, as they
/// are a real terminal's: see [`Screen::new`] and [`Screen::narrow`].
///
/// It keeps OSC 8 hyperlinks as a terminal does: text printed while one is open is that link in
/// each cell it goes in, until `ESC ] 8 ; ; ST` closes it; erasing a cell takes its link away.
///
/// It keeps each cell's colors and bold as SGR sequences set them when the cell was printed
/// (see [`Pen`]).
///
/// It shows a synchronized update as WezTerm, kitty and foot do: what is written between
/// [`BEGIN_SYNC`] and [`END_SYNC`] reaches the screen all at once, when the update ends, however
/// the PTY splits it into reads. An update that never ends is never shown.
///
/// It wraps as a terminal does with autowrap (DECAWM) on, as it starts: a character printed
/// past the last column goes to the start of the next row, and on the last row scrolls the
/// screen up. With autowrap off (`ESC [ ? 7 l`), as WezTerm and xterm have it, such a character
/// is printed in the last column instead, over what was there, and nothing wraps or scrolls.
struct Screen {
    width: usize,
    height: usize,
    /// Each cell's cluster; the empty string is the right half of a two-column cluster.
    cells: Vec<Vec<String>>,
    /// The hyperlink of each cell.
    links: Vec<Vec<Option<Link>>>,
    /// The hyperlink text printed now goes in.
    link: Option<Link>,
    /// What each cell is drawn in.
    pens: Vec<Vec<Pen>>,
    /// What text printed now is drawn in.
    pen: Pen,
    /// Every OSC 8 sequence boxmux wrote that is not a well-formed opening or closing one.
    malformed_links: Vec<String>,
    x: usize,
    y: usize,
    wrap_pending: bool,
    /// Whether autowrap (DECAWM) is on.
    autowrap: bool,
    /// How many times the screen scrolled up: a line feed or a character wrapped past the last
    /// column on the last row moves every row up one and blanks the last, as a terminal does.
    scrolled: usize,
    pending: Vec<u8>,
    /// The cell the last character was printed into, while nothing has happened since.
    last: Option<(usize, usize)>,
    /// Whether an emoji variation selector (U+FE0F) makes its cluster two columns wide.
    emoji_sequences_wide: bool,
    /// Characters this terminal draws one column wide whatever Unicode says.
    narrow_chars: Vec<char>,
}

impl Screen {
    /// A terminal that draws an emoji variation sequence two columns wide, as kitty, foot and
    /// WezTerm with `unicode_version = 14` do.
    fn new(width: usize, height: usize) -> Screen {
        Screen {
            width,
            height,
            cells: vec![vec![" ".to_string(); width]; height],
            links: vec![vec![None; width]; height],
            link: None,
            pens: vec![vec![Pen::default(); width]; height],
            pen: Pen::default(),
            malformed_links: Vec::new(),
            x: 0,
            y: 0,
            wrap_pending: false,
            autowrap: true,
            scrolled: 0,
            pending: Vec::new(),
            last: None,
            emoji_sequences_wide: true,
            narrow_chars: Vec::new(),
        }
    }

    /// A terminal whose widths are narrower than Unicode's: emoji variation sequences take the
    /// width of their base character, as in WezTerm's default `unicode_version = 9`, and each of
    /// `chars` takes one column, as emoji do in terminals that use Unicode 6 tables.
    #[cfg(unix)]
    fn narrow(width: usize, height: usize, chars: &[char]) -> Screen {
        Screen {
            emoji_sequences_wide: false,
            narrow_chars: chars.to_vec(),
            ..Screen::new(width, height)
        }
    }

    fn cluster_width(&self, cluster: &str) -> usize {
        if cluster.chars().any(|c| self.narrow_chars.contains(&c)) {
            return 1;
        }
        if self.emoji_sequences_wide && cluster.contains('\u{fe0f}') {
            return 2;
        }
        cluster
            .chars()
            .map(|c| c.width().unwrap_or(0))
            .sum::<usize>()
            .min(2)
    }

    fn apply(&mut self, bytes: &[u8]) {
        // A held update is looked through only where its end could be: the bytes before were
        // looked through when they came.
        let holding = self.pending.starts_with(BEGIN_SYNC);
        let unseen = self.pending.len().saturating_sub(END_SYNC.len() - 1);
        self.pending.extend_from_slice(bytes);
        if holding && !contains(&self.pending[unseen..], END_SYNC) {
            return;
        }
        let data = std::mem::take(&mut self.pending);
        let mut i = 0;
        while i < data.len() {
            match data[i] {
                0x1b if data[i..].starts_with(BEGIN_SYNC) && !contains(&data[i..], END_SYNC) => {
                    self.pending = data[i..].to_vec();
                    return;
                }
                0x1b => match self.escape(&data, i) {
                    Some(next) => {
                        self.last = None;
                        i = next;
                    }
                    None => {
                        self.pending = data[i..].to_vec();
                        return;
                    }
                },
                b'\r' => {
                    self.x = 0;
                    self.wrap_pending = false;
                    self.last = None;
                    i += 1;
                }
                b'\n' => {
                    self.line_feed();
                    self.wrap_pending = false;
                    self.last = None;
                    i += 1;
                }
                byte if byte < 0x20 => {
                    self.last = None;
                    i += 1;
                }
                _ => {
                    let len = match data[i] {
                        b if b < 0x80 => 1,
                        b if b >= 0xf0 => 4,
                        b if b >= 0xe0 => 3,
                        _ => 2,
                    };
                    if i + len > data.len() {
                        self.pending = data[i..].to_vec();
                        return;
                    }
                    if let Ok(text) = std::str::from_utf8(&data[i..i + len]) {
                        for ch in text.chars() {
                            self.put(ch);
                        }
                    }
                    i += len;
                }
            }
        }
    }

    /// Handles the escape sequence at `i`; `None` when it is not complete yet.
    fn escape(&mut self, data: &[u8], i: usize) -> Option<usize> {
        match *data.get(i + 1)? {
            b'[' => {
                let mut j = i + 2;
                while (0x20..0x40).contains(data.get(j)?) {
                    j += 1;
                }
                let params: Vec<usize> = String::from_utf8_lossy(&data[i + 2..j])
                    .trim_start_matches('?')
                    .split(';')
                    .map(|p| p.parse().unwrap_or(0))
                    .collect();
                let n = |k: usize| params.get(k).copied().filter(|v| *v > 0).unwrap_or(1);
                let private = data[i + 2] == b'?';
                if private && matches!(data[j], b'h' | b'l') && params.contains(&7) {
                    self.autowrap = data[j] == b'h';
                }
                if !private && data[j] == b'm' {
                    self.pen.apply(&params);
                } else if !private {
                    self.wrap_pending = false;
                    match data[j] {
                        b'H' | b'f' => {
                            self.y = (n(0) - 1).min(self.height - 1);
                            self.x = (n(1) - 1).min(self.width - 1);
                        }
                        b'A' => self.y = self.y.saturating_sub(n(0)),
                        b'B' => self.y = (self.y + n(0)).min(self.height - 1),
                        b'C' => self.x = (self.x + n(0)).min(self.width - 1),
                        b'D' => self.x = self.x.saturating_sub(n(0)),
                        b'G' => self.x = (n(0) - 1).min(self.width - 1),
                        b'd' => self.y = (n(0) - 1).min(self.height - 1),
                        b'J' if params.first() == Some(&2) => {
                            for row in &mut self.cells {
                                row.fill(" ".to_string());
                            }
                            for row in &mut self.links {
                                row.fill(None);
                            }
                            for row in &mut self.pens {
                                row.fill(self.pen.erased());
                            }
                        }
                        b'K' => {
                            let (from, to) = match params.first() {
                                Some(1) => (0, self.x + 1),
                                Some(2) => (0, self.width),
                                _ => (self.x, self.width),
                            };
                            self.erase(from, to);
                        }
                        b'X' => self.erase(self.x, self.x + n(0)),
                        _ => {}
                    }
                }
                Some(j + 1)
            }
            b']' => {
                let mut j = i + 2;
                let (end, next) = loop {
                    match *data.get(j)? {
                        0x07 => break (j, j + 1),
                        0x1b if *data.get(j + 1)? == b'\\' => break (j, j + 2),
                        _ => j += 1,
                    }
                };
                self.osc(&String::from_utf8_lossy(&data[i + 2..end]));
                Some(next)
            }
            _ => Some(i + 2),
        }
    }

    /// Handles the text of an OSC sequence, between `ESC ]` and its terminator. `8 ; params ;
    /// URI` opens a hyperlink and `8 ; ; ` closes it, as the OSC 8 specification has them:
    /// params are `key=value` pairs separated by `:`, and the URI is printable ASCII. Any other
    /// OSC 8 is recorded as malformed and changes nothing.
    fn osc(&mut self, text: &str) {
        let Some(rest) = text.strip_prefix("8;") else {
            return;
        };
        let Some((params, uri)) = rest.split_once(';') else {
            self.malformed_links.push(text.to_string());
            return;
        };
        if params.is_empty() && uri.is_empty() {
            self.link = None;
            return;
        }
        let pairs: Option<Vec<(&str, &str)>> = if params.is_empty() {
            Some(Vec::new())
        } else {
            params.split(':').map(|pair| pair.split_once('=')).collect()
        };
        let printable = |s: &str| s.bytes().all(|b| (0x20..=0x7e).contains(&b));
        match pairs {
            Some(pairs) if !uri.is_empty() && printable(uri) => {
                let id = pairs
                    .iter()
                    .find(|(key, _)| *key == "id")
                    .map(|(_, value)| value.to_string());
                self.link = Some(Link {
                    id,
                    uri: uri.to_string(),
                });
            }
            _ => self.malformed_links.push(text.to_string()),
        }
    }

    /// Blanks the cells `from..to` of the cursor's row, and the halves of two-column clusters
    /// the range cuts.
    fn erase(&mut self, from: usize, to: usize) {
        for x in from..to.min(self.width) {
            self.unsplit(x, self.y);
            self.cells[self.y][x] = " ".to_string();
            self.links[self.y][x] = None;
            self.pens[self.y][x] = self.pen.erased();
        }
    }

    /// Blanks the other half of a two-column cluster that writing at (`x`, `y`) cuts, as a
    /// terminal does.
    fn unsplit(&mut self, x: usize, y: usize) {
        if self.cells[y][x].is_empty() && x > 0 {
            self.cells[y][x - 1] = " ".to_string();
        }
        if x + 1 < self.width && self.cells[y][x + 1].is_empty() {
            self.cells[y][x + 1] = " ".to_string();
        }
    }

    fn put(&mut self, ch: char) {
        if let Some((x, y)) = self.last {
            let joined = format!("{}{}", self.cells[y][x], ch);
            if joined.graphemes(true).count() == 1 {
                let before = self.cluster_width(&self.cells[y][x]);
                let after = self.cluster_width(&joined);
                self.cells[y][x] = joined;
                if after > before && x + 1 < self.width {
                    self.unsplit(x + 1, y);
                    self.cells[y][x + 1] = String::new();
                    self.advance(x, after);
                }
                return;
            }
        }
        let width = self.cluster_width(&ch.to_string());
        if width == 0 {
            return;
        }
        // Like a real terminal, a character after the last column goes to the next line, and
        // on the last row scrolls the screen up; with autowrap off it goes in the last column.
        if !self.autowrap {
            self.x = self.x.min(self.width.saturating_sub(width));
        } else if self.wrap_pending || self.x + width > self.width {
            self.x = 0;
            self.line_feed();
            self.wrap_pending = false;
        }
        let (x, y) = (self.x, self.y);
        self.unsplit(x, y);
        self.cells[y][x] = ch.to_string();
        self.links[y][x] = self.link.clone();
        self.pens[y][x] = self.pen;
        if width == 2 && x + 1 < self.width {
            if x + 2 < self.width && self.cells[y][x + 2].is_empty() {
                self.cells[y][x + 2] = " ".to_string();
            }
            self.cells[y][x + 1] = String::new();
            self.links[y][x + 1] = self.link.clone();
            self.pens[y][x + 1] = self.pen;
        }
        self.last = Some((x, y));
        self.advance(x, width);
    }

    /// Moves the cursor past a cluster of `width` columns at column `x`.
    fn advance(&mut self, x: usize, width: usize) {
        if x + width >= self.width {
            self.x = self.width - 1;
            self.wrap_pending = self.autowrap;
        } else {
            self.x = x + width;
        }
    }

    /// Moves the cursor down a row; on the last row, scrolls every row up one instead, the top
    /// row leaving the screen and a blank row coming in at the bottom.
    fn line_feed(&mut self) {
        if self.y + 1 < self.height {
            self.y += 1;
            return;
        }
        self.cells.remove(0);
        self.cells.push(vec![" ".to_string(); self.width]);
        self.links.remove(0);
        self.links.push(vec![None; self.width]);
        self.pens.remove(0);
        self.pens.push(vec![self.pen.erased(); self.width]);
        self.scrolled += 1;
    }

    /// Gives the screen `width` x `height` cells, as WezTerm does to its alternate screen:
    /// rows are cut or padded with blanks on the right; a shorter screen loses the blank rows
    /// below the cursor first, then its top rows, so the cursor's row stays in view; a taller
    /// one gains blank rows at the bottom. A wide cluster the new right edge cuts is blanked.
    #[cfg(unix)]
    fn resize(&mut self, width: usize, height: usize) {
        for y in 0..self.cells.len() {
            let cut = width < self.width && self.cells[y][width].is_empty();
            self.cells[y].resize(width, " ".to_string());
            self.links[y].resize(width, None);
            self.pens[y].resize(width, Pen::default());
            if cut {
                self.cells[y][width - 1] = " ".to_string();
                self.links[y][width - 1] = None;
                self.pens[y][width - 1] = Pen::default();
            }
        }
        while self.cells.len() > height
            && self.cells.len() > self.y + 1
            && self.cells.last().unwrap().iter().all(|c| c == " ")
        {
            self.cells.pop();
            self.links.pop();
            self.pens.pop();
        }
        while self.cells.len() > height {
            self.cells.remove(0);
            self.links.remove(0);
            self.pens.remove(0);
            self.y = self.y.saturating_sub(1);
        }
        while self.cells.len() < height {
            self.cells.push(vec![" ".to_string(); width]);
            self.links.push(vec![None; width]);
            self.pens.push(vec![Pen::default(); width]);
        }
        self.width = width;
        self.height = height;
        self.x = self.x.min(width - 1);
        self.y = self.y.min(height - 1);
        self.wrap_pending = false;
        self.last = None;
    }

    /// The cluster in cell (`x`, `y`): empty for the right half of a two-column one.
    #[cfg(unix)]
    fn cell(&self, x: usize, y: usize) -> &str {
        &self.cells[y][x]
    }

    fn row(&self, y: usize) -> String {
        self.cells[y].concat()
    }

    /// The text of cells `from..=to` of row `y`.
    #[cfg(unix)]
    fn span(&self, y: usize, from: usize, to: usize) -> String {
        self.cells[y][from..=to].concat()
    }

    /// The cell where `needle` starts.
    fn find(&self, needle: &str) -> Option<(usize, usize)> {
        (0..self.height).find_map(|y| {
            (0..self.width)
                .find(|&x| {
                    !self.cells[y][x].is_empty() && self.cells[y][x..].concat().starts_with(needle)
                })
                .map(|x| (x, y))
        })
    }

    fn text(&self) -> String {
        (0..self.height)
            .map(|y| self.row(y))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The screen's text followed by its links, as [`Screen::link_runs`] lists them.
    #[cfg(unix)]
    fn with_links(&self) -> String {
        format!("{}\nlinks:\n{}", self.text(), self.link_runs())
    }

    /// Each run of cells of one hyperlink, a line each: its row, its columns, its id and URI,
    /// and the malformed OSC 8 sequences received.
    fn link_runs(&self) -> String {
        let mut out = Vec::new();
        for (y, row) in self.links.iter().enumerate() {
            let mut x = 0;
            while x < row.len() {
                let Some(link) = &row[x] else {
                    x += 1;
                    continue;
                };
                let end = (x..row.len())
                    .find(|&e| row[e].as_ref() != Some(link))
                    .unwrap_or(row.len());
                out.push(format!(
                    "row {} columns {}..{}: id {:?} {}",
                    y, x, end, link.id, link.uri
                ));
                x = end;
            }
        }
        for sequence in &self.malformed_links {
            out.push(format!("malformed: {:?}", sequence));
        }
        out.join("\n")
    }
}
#[test]
fn the_socket_option_goes_before_or_after_a_control_command_but_interface_options_do_not() {
    let fixture = Fixture::new();
    let socket = fixture.socket();
    for args in [
        vec!["--socket", socket.as_str(), "query_pty_status", "b"],
        vec!["query_pty_status", "--socket", socket.as_str(), "b"],
    ] {
        let output = Command::new(binary()).args(&args).output().unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Nothing listens on the socket: the command was understood and tried to connect.
        assert!(
            stderr.contains("cannot reach a boxmux instance"),
            "{:?}: {}",
            args,
            stderr
        );
    }
    let output = Command::new(binary())
        .args(["--frame-delay", "5", "query_pty_status", "b"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("frame-delay cannot be used"));
}

#[test]
#[cfg(unix)]
fn the_terminal_is_asked_for_its_background_and_the_answer_is_not_taken_for_keys() {
    let fixture = Fixture::new();
    // A key handler that would fire if the reply's characters were read as typed keys.
    let config = fixture.write_config(
        r#"
app:
  on_keypress:
    r: ["printf leaked > markers/leaked"]
    b: ["printf leaked > markers/leaked"]
  layouts:
    - id: main
      root: true
      children:
        - id: a
          position: {x1: "0", y1: "0", x2: "30", y2: "5"}
          content: themed
"#,
    );
    let pair = native_pty_system()
        .openpty(PtySize {
            rows: HEIGHT,
            cols: WIDTH,
            pixel_width: 0,
            pixel_height: 0,
        })
        .unwrap();
    let mut command = CommandBuilder::new(binary());
    command.arg(&config);
    command.arg("--socket");
    command.arg(fixture.socket());
    command.cwd(fixture.tmp.path());
    let mut child = pair.slave.spawn_command(command).unwrap();
    drop(pair.slave);
    let mut writer = pair.master.take_writer().unwrap();
    let mut reader = pair.master.try_clone_reader().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut buf = [0u8; 4096];
        while let Ok(n) = reader.read(&mut buf) {
            if n == 0 || tx.send(buf[..n].to_vec()).is_err() {
                break;
            }
        }
    });
    let mut seen = Vec::new();
    let mut answered = false;
    let mut screen = Screen::new(WIDTH as usize, HEIGHT as usize);
    let start = Instant::now();
    while screen.find("themed").is_none() {
        assert!(start.elapsed() < TIMEOUT, "never drawn:\n{}", screen.text());
        if let Ok(bytes) = rx.recv_timeout(Duration::from_millis(20)) {
            screen.apply(&bytes);
            seen.extend(bytes);
        }
        let text = String::from_utf8_lossy(&seen);
        if !answered && text.contains("\x1b]11;?") && text.contains("\x1b[c") {
            // A light background, then the device attributes.
            writer
                .write_all(b"\x1b]11;rgb:ffff/ffff/ffff\x1b\\\x1b[?62;22c")
                .unwrap();
            writer.flush().unwrap();
            answered = true;
        }
    }
    assert!(
        answered,
        "boxmux asks the terminal for its background color"
    );
    fixture.assert_no_marker_after("leaked", Duration::from_millis(300));
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn q_quits_with_success() {
    let fixture = Fixture::new();
    let config = fixture.write_config(&menu());
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("READY");
    // BoxMux draws with autowrap off, and turns it back on as it leaves. On Windows the console
    // host between boxmux and the test writes its own screen out, in its own modes.
    let unix = cfg!(unix);
    assert!(
        !(unix && app.screen.autowrap),
        "autowrap is on while boxmux runs"
    );
    app.send("q");
    assert!(app.wait_for_exit(), "q exits successfully");
    assert!(
        !unix || app.screen.autowrap,
        "autowrap is not turned back on"
    );
}

#[test]
fn a_locked_interface_cannot_be_rearranged_and_never_writes_the_file() {
    let fixture = Fixture::new();
    let original = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: mover
          title: Mover
          position: {x1: "10", y1: "5", x2: "30", y2: "10"}
          content: locked
"#;
    let config = fixture.write_config(original);
    let mut app = LiveBoxMux::spawn_with(&fixture, &config, &["--lock"]);
    app.wait_for_text("locked");
    // The resize handle is not offered.
    assert_eq!(
        app.screen.row(10).chars().nth(30),
        Some('┘'),
        "{}",
        app.screen.text()
    );
    app.press(12, 5);
    app.drag_to(16, 7);
    app.release(16, 7);
    app.pump(Duration::from_millis(300));
    assert!(
        app.screen.find("locked").is_some_and(|(_, y)| y == 7),
        "{}",
        app.screen.text()
    );
    assert_eq!(fs::read_to_string(&config).unwrap(), original);
    assert!(
        !fixture.path("app.yaml.lock").exists(),
        "no lock file is taken"
    );
}

/// What a terminal shows of the box drawn by [`every_frame_reaches_the_terminal_whole`] while
/// boxmux draws it: nothing, the box whole with its first text, or the box whole with its second.
/// A description of anything else, such as a box missing a corner or a text half replaced.
#[cfg(unix)]
fn torn_frame(screen: &Screen) -> Option<String> {
    // The bottom right corner is the box's resize handle.
    let corners = [(10, 3, "┌"), (60, 3, "┐"), (10, 12, "└"), (60, 12, "⋱")]
        .iter()
        .filter(|(x, y, corner)| screen.cell(*x, *y) == *corner)
        .count();
    let shown = screen.text();
    let texts = ["FIRST FRAME TEXT", "OTHER WORDS HERE"]
        .iter()
        .filter(|text| shown.contains(*text))
        .count();
    match (corners, texts) {
        (0, 0) | (4, 1) => None,
        _ => Some(format!(
            "{} of the box's 4 corners and {} of its texts shown",
            corners, texts
        )),
    }
}

/// A terminal is never shown half a frame: boxmux writes each frame as one synchronized update,
/// so a terminal that supports them (WezTerm, kitty, foot) shows none of it until all of it is
/// there, however the PTY splits it, and a terminal reading the screen as boxmux draws never
/// finds it half drawn. Unix only, as [`display_width`] is: on Windows the console host writes
/// its own rendering in place of boxmux's output.
#[test]
#[cfg(unix)]
fn every_frame_reaches_the_terminal_whole() {
    let fixture = Fixture::new();
    let config = fixture.write_config(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: note
          title: Note
          position: {x1: "10", y1: "3", x2: "60", y2: "12"}
          content: FIRST FRAME TEXT
"#,
    );
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("FIRST FRAME TEXT");
    let output = fixture.control(&["replace_box_content", "note", "OTHER WORDS HERE"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    app.wait_for_text("OTHER WORDS HERE");
    // Everything boxmux wrote, read one byte at a time: the smallest pieces a PTY can split it
    // into, each looked at as it reaches the screen.
    let mut screen = Screen::new(WIDTH as usize, HEIGHT as usize);
    for (at, byte) in app.raw.iter().enumerate() {
        screen.apply(&[*byte]);
        if let Some(torn) = torn_frame(&screen) {
            panic!(
                "after byte {} of {}, the terminal shows a torn frame: {}\n{}",
                at + 1,
                app.raw.len(),
                torn,
                screen.text()
            );
        }
    }
    assert!(
        screen.find("OTHER WORDS HERE").is_some(),
        "{}",
        screen.text()
    );
}

#[test]
#[cfg(unix)]
fn ctrl_z_suspends_with_the_terminal_restored_and_continuing_redraws() {
    let fixture = Fixture::new();
    let config = fixture.write_config(&menu());
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("READY");
    let before = app.raw.len();
    app.send("\x1a");
    // The process stops after giving the terminal back.
    let start = Instant::now();
    loop {
        app.pump(Duration::from_millis(20));
        let status = fs::read_to_string(format!("/proc/{}/stat", app.pid()))
            .ok()
            .and_then(|stat| stat.rsplit(')').next().map(|rest| rest.trim().to_string()));
        let stopped = match status {
            Some(rest) => rest.starts_with('T'),
            // Without /proc (FreeBSD, macOS), ask ps.
            None => {
                let output = Command::new("ps")
                    .args(["-o", "stat=", "-p", &app.pid().to_string()])
                    .output()
                    .unwrap();
                String::from_utf8_lossy(&output.stdout)
                    .trim()
                    .starts_with('T')
            }
        };
        if stopped {
            break;
        }
        assert!(start.elapsed() < TIMEOUT, "boxmux did not stop");
    }
    // Everything boxmux wrote before it stopped is in the PTY, though the reader may not have
    // passed all of it on yet; stopped, boxmux writes nothing more.
    let start = Instant::now();
    while !String::from_utf8_lossy(&app.raw[before..]).contains("\x1b[?1049l") {
        assert!(
            start.elapsed() < TIMEOUT,
            "boxmux stopped without leaving the alternate screen: {:?}",
            String::from_utf8_lossy(&app.raw[before..])
        );
        app.pump(Duration::from_millis(20));
    }
    assert!(app.screen.autowrap, "boxmux stopped with autowrap off");
    let before = app.raw.len();
    assert_eq!(unsafe { libc::kill(app.pid(), libc::SIGCONT) }, 0);
    let start = Instant::now();
    while !String::from_utf8_lossy(&app.raw[before..]).contains("\x1b[?1049h") {
        assert!(start.elapsed() < TIMEOUT, "boxmux did not come back");
        app.pump(Duration::from_millis(20));
    }
    app.wait_for_text("READY");
    assert!(!app.screen.autowrap, "boxmux came back with autowrap on");
}

/// Text widths as BoxMux draws them, read through the test terminal. Unix only: on Windows
/// the console host (ConPTY) between boxmux and the test keeps its own screen and writes that
/// out instead of boxmux's output, without combining marks or skin tones and with its own
/// cursor placement, so a test there reads ConPTY's rendering rather than boxmux's.
#[cfg(unix)]
mod display_width {
    use super::*;

    /// Boxes whose titles, tab labels, menu entries and text hold wide characters (CJK, emoji, an
    /// emoji with a variation selector, an emoji with a skin tone), ambiguous ones (→, ·) and a
    /// combining mark (e + U+0301), at widths that make them wrap and be cut off.
    const WIDTHS: &str = r#"
app:
  layouts:
    - id: main
      title: Widths
      root: true
      children:
        - id: dawn
          title: "🌅 Dawn → dusk · 日本 ☀️ éclair 👍🏽 and more"
          title_position: left
          position: {x1: "0", y1: "0", x2: "29", y2: "6"}
          overflow_behavior: truncate
          content: "日本語のテキストですね、長い行です\n🌅 sun → moon · star ☀️ été 👍🏽 end of line\nplain ascii line that is long enough"
        - id: wrap
          title: Wrap
          position: {x1: "30", y1: "0", x2: "49", y2: "9"}
          overflow_behavior: wrap
          content: "🌅 dawn → 日本語のテキスト · ☀️ éclair 👍🏽 a b c 日本"
        - id: tabs
          title: Tabs
          position: {x1: "50", y1: "0", x2: "99", y2: "9"}
          content: TAB_HOME
        - id: sunrise
          title: "🌅 Sunrise → east"
          position: {x1: "0", y1: "10", x2: "29", y2: "14"}
          script: ["echo FEED_ONE"]
          redirect_output: tabs
        - id: nihon
          title: "日本 · ☀️ é"
          position: {x1: "30", y1: "10", x2: "49", y2: "14"}
          script: ["echo FEED_TWO"]
          redirect_output: tabs
        - id: menu
          title: "Menu → 🌅"
          position: {x1: "50", y1: "10", x2: "69", y2: "19"}
          overflow_behavior: wrap
          choices:
            - id: one
              content: "🌅 Rise → shine 日本語"
            - id: two
              content: "éclair · ☀️ 👍🏽 ok"
"#;

    /// The cells a box covers: `(x1, y1, x2, y2)`.
    type Cells = (usize, usize, usize, usize);

    /// The boxes of [`WIDTHS`]: id and the cells it covers.
    const WIDTH_BOXES: [(&str, Cells); 6] = [
        ("dawn", (0, 0, 29, 6)),
        ("wrap", (30, 0, 49, 9)),
        ("tabs", (50, 0, 99, 9)),
        ("sunrise", (0, 10, 29, 14)),
        ("nihon", (30, 10, 49, 14)),
        ("menu", (50, 10, 69, 19)),
    ];

    /// What keeps an edge of a box of [`WIDTH_BOXES`] out of its own column and row: none when
    /// every edge of every box is in its place.
    fn edge_problems(screen: &Screen) -> Vec<String> {
        let mut problems = Vec::new();
        for (id, (x1, y1, x2, y2)) in WIDTH_BOXES {
            let mut fail = |what: String| problems.push(format!("box {}: {}", id, what));
            let at = |x: usize, y: usize| screen.cell(x, y);
            if (at(x1, y1), at(x2, y1), at(x1, y2)) != ("┌", "┐", "└")
                || !["┘", "⋱"].contains(&at(x2, y2))
            {
                fail("a corner is not where the box's corners are".to_string());
            }
            for y in y1 + 1..y2 {
                if at(x1, y) != "│" || !["│", "█"].contains(&at(x2, y)) {
                    fail(format!("row {} is not closed by its borders", y));
                }
            }
            for x in x1 + 1..x2 {
                if !["─", "■"].contains(&at(x, y2)) {
                    fail(format!(
                        "column {} of the bottom border is {:?}",
                        x,
                        at(x, y2)
                    ));
                }
            }
            if at(x1 + 1, y1) != "─" || at(x2 - 1, y1) != "─" {
                fail("the title or tabs reach the corners".to_string());
            }
        }
        problems
    }

    /// The cells inside the box's border, row by row.
    fn interior(screen: &Screen, (x1, y1, x2, y2): Cells) -> Vec<String> {
        (y1 + 1..y2)
            .map(|y| screen.span(y, x1 + 1, x2 - 1))
            .collect()
    }

    /// Rows of [`WIDTHS`] as drawn, `(y, x1, x2, cells x1..=x2)`, in a terminal that gives every
    /// cluster the width BoxMux does. Titles are cut at 24 columns (dawn) and centered; text is cut
    /// with `…` at 26 columns (dawn) and wrapped at 16 (wrap) and at 14 after a menu marker.
    const WIDTH_ROWS: [(usize, usize, usize, &str); 15] = [
        (0, 0, 29, "┌─ 🌅 Dawn → dusk · 日本 … ──┐"),
        (2, 0, 29, "│ 日本語のテキストですね、…  │"),
        (3, 0, 29, "│ 🌅 sun → moon · star ☀\u{fe0f} e\u{301}… │"),
        (4, 0, 29, "│ plain ascii line that is … │"),
        (3, 30, 49, "│ 🌅 dawn →        │"),
        (4, 30, 49, "│ 日本語のテキスト │"),
        (5, 30, 49, "│ · ☀\u{fe0f} e\u{301}clair 👍🏽 a │"),
        (6, 30, 49, "│ b c 日本         │"),
        (10, 0, 29, "┌──── 🌅 Sunrise → east ─────┐"),
        (10, 30, 49, "┌── 日本 · ☀\u{fe0f} e\u{301} ───┐"),
        (10, 50, 69, "┌─── Menu → 🌅 ────┐"),
        (11, 50, 69, "│ ► 🌅 Rise →      │"),
        (12, 50, 69, "│   shine 日本語   │"),
        (13, 50, 69, "│   e\u{301}clair · ☀\u{fe0f} 👍🏽 │"),
        (14, 50, 69, "│   ok             │"),
    ];

    /// The tabs of `tabs` in its top border; the two scripts' tabs come in the order they ended.
    const WIDTH_TABS: [&str; 2] = [" 日本 · ☀\u{fe0f} e\u{301} × ", " 🌅 Sunrise → ea… × "];

    /// What a row of [`WIDTH_ROWS`] reads in a terminal that draws 🌅 and ☀️ one column wide: each
    /// followed by the blank BoxMux put in its second column.
    fn drawn_narrow(row: &str) -> String {
        row.replace('🌅', "🌅 ").replace("☀\u{fe0f}", "☀\u{fe0f} ")
    }

    /// What differs from the rows of [`WIDTH_ROWS`] and the tabs; `drawn` maps a row to what
    /// the terminal shows for it.
    fn width_row_problems(screen: &Screen, drawn: fn(&str) -> String) -> Vec<String> {
        let mut problems = Vec::new();
        for (y, x1, x2, row) in WIDTH_ROWS {
            let shown = screen.span(y, x1, x2);
            if shown != drawn(row) {
                problems.push(format!(
                    "row {} from column {} is {:?}, not {:?}",
                    y,
                    x1,
                    shown,
                    drawn(row)
                ));
            }
        }
        let [a, b] = WIDTH_TABS;
        let tabs = screen.span(0, 50, 99);
        if ![a, b]
            .iter()
            .zip([b, a])
            .any(|(first, second)| tabs == drawn(&format!("┌── Tabs ─{}─{}───┐", first, second)))
        {
            problems.push(format!("the tabs are {:?}", tabs));
        }
        problems
    }

    /// Runs boxmux on [`WIDTHS`] in `screen`, and checks that every box's edges stay in their
    /// columns and its rows read as `drawn` makes [`WIDTH_ROWS`]. Returns the running app.
    fn draw_widths(screen: Screen, drawn: fn(&str) -> String) -> (Fixture, LiveBoxMux) {
        let fixture = Fixture::new();
        let config = fixture.write_config(WIDTHS);
        let mut app = LiveBoxMux::spawn_in(&fixture, &config, &[], screen);
        // Both scripts ran: their output opened a tab each in `tabs`, after its own.
        app.wait_for_screen("every box drawn in its columns", |screen| {
            let mut problems = edge_problems(screen);
            problems.extend(width_row_problems(screen, drawn));
            problems
        });
        (fixture, app)
    }

    /// Replaces the text of `wrap` and `dawn` over the control socket.
    fn replace_widths_text(fixture: &Fixture, wrap: &str, dawn: &str) {
        for (id, text) in [("wrap", wrap), ("dawn", dawn)] {
            let output = fixture.control(&["replace_box_content", id, text]);
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }

    /// Waits until every box keeps its edges, and inside its border `wrap` shows only `wrap` on
    /// its fourth row and `dawn` only `dawn` on its third: the cells of what they showed before
    /// are blank. `wrap` and `dawn` are the rows as a terminal giving every cluster the width
    /// BoxMux does reads them; `drawn` maps them to what this terminal shows.
    fn wait_for_replaced(app: &mut LiveBoxMux, drawn: fn(&str) -> String, wrap: &str, dawn: &str) {
        // wrap: 18 cells inside the border, 8 rows; dawn: 28 cells, 5 rows.
        let interiors: Vec<_> = [
            ("wrap", WIDTH_BOXES[1].1, 8, 3, wrap),
            ("dawn", WIDTH_BOXES[0].1, 5, 2, dawn),
        ]
        .into_iter()
        .map(|(id, cells, rows, at, row)| {
            let mut expected = vec![drawn(&" ".repeat(cells.2 - cells.0 - 1)); rows];
            expected[at] = drawn(row);
            (id, cells, expected)
        })
        .collect();
        app.wait_for_screen("the new text alone in wrap and dawn", |screen| {
            let mut problems = edge_problems(screen);
            for (id, cells, expected) in &interiors {
                let shown = interior(screen, *cells);
                if shown != *expected {
                    problems.push(format!("inside {} is {:?}, not {:?}", id, shown, expected));
                }
            }
            problems
        });
    }

    /// Replaces the text of `wrap` and `dawn` twice and checks that nothing of what they showed
    /// before is left: first wide, ambiguous and combining text with plain text, then plain text
    /// with wide text over the same cells. Each box shows its new line, centered, and blanks.
    fn replace_widths(fixture: &Fixture, app: &mut LiveBoxMux, drawn: fn(&str) -> String) {
        replace_widths_text(fixture, "short", "ok");
        // Centered in the 16 columns of wrap's content area and the 26 of dawn's, each one column
        // in from the border.
        wait_for_replaced(
            app,
            drawn,
            &format!("{}short{}", " ".repeat(6), " ".repeat(7)),
            &format!("{}ok{}", " ".repeat(13), " ".repeat(13)),
        );

        replace_widths_text(fixture, "🌅🌅🌅", "→ 日本 ☀\u{fe0f}");
        wait_for_replaced(
            app,
            drawn,
            &format!("{}🌅🌅🌅{}", " ".repeat(6), " ".repeat(6)),
            &format!("{}→ 日本 ☀\u{fe0f}{}", " ".repeat(9), " ".repeat(10)),
        );
    }

    #[test]
    fn wide_ambiguous_and_combining_characters_keep_every_box_in_its_columns() {
        let (fixture, mut app) =
            draw_widths(Screen::new(WIDTH as usize, HEIGHT as usize), str::to_string);
        replace_widths(&fixture, &mut app, str::to_string);
    }

    #[test]
    fn a_terminal_that_draws_wide_characters_narrower_still_shows_every_box_in_its_columns() {
        // Like a terminal with Unicode 6 tables, where 🌅 takes one column, and WezTerm's default,
        // where ☀️ takes one.
        let (fixture, mut app) = draw_widths(
            Screen::narrow(WIDTH as usize, HEIGHT as usize, &['🌅']),
            drawn_narrow,
        );
        replace_widths(&fixture, &mut app, drawn_narrow);
    }
}

/// OSC 8 hyperlinks in a box's text, a menu entry and a terminal box, read through the test
/// terminal. Unix only, for the reason [`display_width`] gives: on Windows the test would read
/// ConPTY's rendering rather than boxmux's.
#[cfg(unix)]
mod hyperlinks {
    use super::*;

    /// A box of text, a menu and a terminal box, each with a link whose text holds a wide
    /// word (日本) and wraps onto a second line. The text's link has an id, the menu entry's has
    /// none, and the text ends with a malformed OSC 8 (a parameter that is not `key=value`).
    const LINKS: &str = r#"
app:
  layouts:
    - id: main
      title: Links
      root: true
      children:
        - id: doc
          title: Doc
          position: {x1: "0", y1: "0", x2: "21", y2: "6"}
          overflow_behavior: wrap
          content: "see \e]8;id=manual;https://example.com/manual\e\\the 日本 manual pages\e]8;;\e\\ now\e]8;broken;https://example.com/x\e\\!"
        - id: menu
          title: Menu
          position: {x1: "30", y1: "0", x2: "51", y2: "8"}
          overflow_behavior: wrap
          choices:
            - id: guide
              content: "open \e]8;;https://example.com/guide\e\\our 日本 guide pages\e]8;;\e\\"
        - id: term
          title: Term
          position: {x1: "0", y1: "10", x2: "21", y2: "16"}
          execution_mode: Pty
          script: ['printf "go \033]8;id=term;https://example.com/term\033\\\\to 日本 terminals link text\033]8;;\033\\\\ end\n"; sleep 600']
"#;

    /// The link of each column `text` takes where it is on the screen, and of the column on
    /// each side of it.
    fn links_around(
        screen: &Screen,
        text: &str,
    ) -> (Option<Link>, Vec<Option<Link>>, Option<Link>) {
        let (x, y) = screen
            .find(text)
            .unwrap_or_else(|| panic!("{:?} is not shown\n{}", text, screen.with_links()));
        let columns: usize = text
            .graphemes(true)
            .map(|cluster| screen.cluster_width(cluster))
            .sum();
        let at = |x: usize| screen.links[y].get(x).cloned().flatten();
        (
            x.checked_sub(1).and_then(at),
            (x..x + columns).map(at).collect(),
            at(x + columns),
        )
    }

    /// Checks that every column of `text` is a link to `uri`, the columns beside it are not, and
    /// returns the link.
    fn assert_linked(screen: &Screen, text: &str, uri: &str) -> Link {
        let (before, cells, after) = links_around(screen, text);
        let link = cells[0]
            .clone()
            .unwrap_or_else(|| panic!("{:?} is not a link\n{}", text, screen.with_links()));
        assert_eq!(link.uri, uri, "the link of {:?}", text);
        assert!(
            cells.iter().all(|cell| cell.as_ref() == Some(&link)),
            "not every column of {:?} is the link: {:?}",
            text,
            cells
        );
        assert_ne!(
            before.as_ref(),
            Some(&link),
            "the link starts before {:?}",
            text
        );
        assert_ne!(
            after.as_ref(),
            Some(&link),
            "the link goes on after {:?}",
            text
        );
        link
    }

    /// Checks that no escape sequence of a link was drawn as text and every OSC 8 boxmux wrote
    /// was well formed.
    fn assert_no_raw_links(app: &mut LiveBoxMux) {
        let text = app.screen.text();
        assert!(
            !text.contains("8;") && !text.contains("]8") && !text.contains('\\'),
            "a link's escape sequence is drawn as text\n{}",
            app.report()
        );
        assert!(
            app.screen.malformed_links.is_empty(),
            "boxmux wrote malformed OSC 8 sequences: {:?}",
            app.screen.malformed_links
        );
    }

    /// Runs boxmux on [`LINKS`] until `shown` is on the screen, in a whole frame: the rest of
    /// the box it is in is drawn too.
    fn start(shown: &str) -> (Fixture, LiveBoxMux) {
        let fixture = Fixture::new();
        let config = fixture.write_config(LINKS);
        let mut app = LiveBoxMux::spawn(&fixture, &config);
        app.wait_for_text(shown);
        (fixture, app)
    }

    #[test]
    fn a_link_in_a_box_s_text_reaches_the_terminal_on_each_line_it_wraps_onto() {
        let (_fixture, mut app) = start("manual pages now!");
        let first = assert_linked(&app.screen, "the 日本", "https://example.com/manual");
        let second = assert_linked(&app.screen, "manual pages", "https://example.com/manual");
        assert_eq!(first.id.as_deref(), Some("manual"), "the link keeps its id");
        assert_eq!(first, second, "both lines are the same link");
        assert!(links_around(&app.screen, "see ")
            .1
            .iter()
            .all(Option::is_none));
        // The malformed OSC 8 is dropped: nothing of it is drawn, and `!` is no link.
        let (_, cells, _) = links_around(&app.screen, " now!");
        assert!(cells.iter().all(Option::is_none), "{:?}", cells);
        assert_no_raw_links(&mut app);
    }

    #[test]
    fn a_link_in_a_menu_entry_reaches_the_terminal_on_each_line_it_wraps_onto() {
        let (_fixture, mut app) = start("guide pages");
        let first = assert_linked(&app.screen, "our 日本", "https://example.com/guide");
        let second = assert_linked(&app.screen, "guide pages", "https://example.com/guide");
        assert!(
            first.id.as_deref().is_some_and(|id| !id.is_empty()),
            "a link given no id is written with one, so its lines are one link: {:?}",
            first
        );
        assert_eq!(first, second, "both lines are the same link");
        assert_no_raw_links(&mut app);
    }

    #[test]
    fn a_link_a_program_prints_in_a_terminal_box_reaches_the_terminal_on_each_line() {
        let (_fixture, mut app) = start("link text end");
        let first = assert_linked(&app.screen, "to 日本 terminals", "https://example.com/term");
        let second = assert_linked(&app.screen, " link text", "https://example.com/term");
        assert_eq!(first.id.as_deref(), Some("term"), "the link keeps its id");
        assert_eq!(first, second, "both lines are the same link");
        assert!(links_around(&app.screen, "go ")
            .1
            .iter()
            .all(Option::is_none));
        assert_no_raw_links(&mut app);
    }
}

/// The rows Needs you takes in the handlebar the musher saw: its borders and fourteen rows of
/// buttons, the most wsb-gangline gives it.
const NEEDS: usize = 16;

/// The last line of the swing's log; it is the log box's last row once the log is scrolled to
/// its end.
const LAST_LOG_LINE: &str = "LAST LOG LINE";

/// The handlebar's arrangement as wsb-gangline lays it out: Needs you at the top, `needs` rows
/// tall, counted from the top, with its buttons; the swing's log in every row below it to the
/// screen's bottom, wrapped and scrolled to its last line.
fn handlebar(needs: usize) -> String {
    let log_script = if cfg!(windows) {
        "Get-Content log.txt"
    } else {
        "cat log.txt"
    };
    r#"
app:
  layouts:
    - id: main
      root: true
      title: swing
      children:
        - id: needs
          title: "Needs you: 5 (click to act)"
          position: {x1: "0%", y1: "0", x2: "100%", y2: "NEEDS_LAST"}
          border_color: bright_yellow
          tab_order: 1
          selected: true
          overflow_behavior: wrap
          choices:
            - id: refresh
              content: "⟳ Refresh"
              script: ["echo refresh"]
            - id: plan
              content: "? capdag plan: 4 design questions before building the save-as caps (answer tomorrow)"
              script: ["echo plan"]
            - id: fix
              content: "▸ Before the process-fix lands: bring pve2 to today's sdx 1.863 / sdx-mf 0.91.601 (pve is done)"
              script: ["echo fix"]
            - id: rerun
              content: "● Mac: rerun the fixed monitor proof of the process-stop fix (asks for sudo once) — running: its terminal"
              script: ["echo rerun"]
            - id: question
              content: "⚑ Question (product): lungo 1.120.164 is on the way"
              script: ["echo question"]
        - id: log
          title: What the swing is doing
          position: {x1: "0%", y1: "LOG_FIRST", x2: "100%", y2: "100%"}
          border_color: green
          overflow_behavior: wrap
          auto_scroll_bottom: true
          script: ["LOG_SCRIPT"]
"#
    .replace("NEEDS_LAST", &(needs - 1).to_string())
    .replace("LOG_FIRST", &needs.to_string())
    .replace("LOG_SCRIPT", log_script)
}

/// A fixture holding the handlebar's configuration and the swing's log it shows.
fn handlebar_fixture() -> (Fixture, PathBuf) {
    let fixture = Fixture::new();
    let mut log: Vec<String> = (1..=30)
        .map(|n| {
            format!(
                "11:{:02}:04 worker {} moved to a fresh worktree from main and runs QuickRust on a guest",
                n, n
            )
        })
        .collect();
    log.push(LAST_LOG_LINE.to_string());
    fs::write(fixture.path("log.txt"), log.join("\n") + "\n").unwrap();
    let config = fixture.write_config(&handlebar(NEEDS));
    (fixture, config)
}

/// What is wrong with the handlebar `screen` shows, Needs you `needs` rows tall: each box's
/// border must run down its sides and end on its own last row, Needs you's top border alone on
/// row 0, the log's bottom border on the screen's last row with the log's last line just above
/// it, and the screen never scrolled.
fn handlebar_problems(screen: &Screen, needs: usize) -> Vec<String> {
    let last = screen.height - 1;
    let mut problems = Vec::new();
    if screen.scrolled > 0 {
        problems.push(format!(
            "the screen scrolled {} times: something was written past its last row or column",
            screen.scrolled
        ));
    }
    let mut expect = |y: usize, left: char, right: &[char], holds: &str, what: &str| {
        let row = screen.row(y);
        let (first, end) = (row.chars().next(), row.chars().last());
        if first != Some(left) || !end.is_some_and(|c| right.contains(&c)) || !row.contains(holds) {
            problems.push(format!("row {} is not {}: {:?}", y, what, row));
        }
    };
    let side = ['│', '█'];
    let corner = ['┘', '⋱'];
    expect(0, '┌', &['┐'], "Needs you", "Needs you's top border");
    for y in 1..needs - 1 {
        expect(y, '│', &side, "", "inside Needs you");
    }
    expect(needs - 1, '└', &corner, "", "Needs you's bottom border");
    expect(
        needs,
        '┌',
        &['┐'],
        "What the swing is doing",
        "the log's top border",
    );
    for y in needs + 1..last {
        expect(y, '│', &side, "", "inside the log");
    }
    expect(last - 1, '│', &side, LAST_LOG_LINE, "the log's last line");
    expect(last, '└', &corner, "", "the log's bottom border");
    problems
}

impl LiveBoxMux {
    /// Waits until the screen shows the handlebar laid out for the terminal's size, Needs you
    /// `needs` rows tall (see [`handlebar_problems`]).
    fn wait_for_handlebar(&mut self, needs: usize) {
        let what = format!(
            "the handlebar laid out for {}x{}",
            self.screen.width, self.screen.height
        );
        self.wait_for_screen(&what, |screen| handlebar_problems(screen, needs));
    }
}

#[test]
fn every_box_of_the_handlebar_ends_on_its_own_last_row_at_every_height() {
    // 55x26 is the pane the musher saw the log's bottom border on row 0 in; 19 rows leave the
    // log one row between its borders.
    for (cols, rows) in [(55, 26), (55, 19), (55, 33), (48, 50), (60, 26)] {
        let (fixture, config) = handlebar_fixture();
        let mut app = LiveBoxMux::spawn_in(&fixture, &config, &[], Screen::new(cols, rows));
        app.wait_for_handlebar(NEEDS);
    }
}

#[test]
#[cfg(unix)]
fn a_terminal_resized_as_boxmux_starts_and_back_and_forth_is_drawn_at_each_size_it_has() {
    // The handlebar opens 60 columns wide and its pane is sized after: here while boxmux asks the
    // terminal for its background color, which it does after reading the terminal's size and
    // before it draws anything.
    for (from, to) in [
        ((60, 30), (55, 26)),
        ((55, 20), (55, 26)),
        ((55, 26), (48, 33)),
    ] {
        let (fixture, config) = handlebar_fixture();
        let mut app =
            LiveBoxMux::spawn_asking_background(&fixture, &config, Screen::new(from.0, from.1));
        let start = Instant::now();
        while !String::from_utf8_lossy(&app.raw).contains("\x1b]11;?") {
            assert!(
                start.elapsed() < TIMEOUT,
                "boxmux never asked for the background color\n{}",
                app.report()
            );
            app.pump(Duration::from_millis(10));
        }
        app.resize(to.0, to.1);
        // The device attributes end the question: the terminal does not say its background.
        app.send("\x1b[?62;22c");
        app.wait_for_handlebar(NEEDS);
        // A shrink and a grow back before boxmux draws, then a size of its own.
        app.resize(to.0, to.1 - 1);
        app.resize(to.0, to.1);
        app.wait_for_handlebar(NEEDS);
        app.resize(to.0 + 7, to.1 + 4);
        app.wait_for_handlebar(NEEDS);
    }
}

/// One layout of empty boxes, each with its title and its position: `(title, x1, y1, x2, y2)`.
fn boxes(positions: &[(&str, &str, &str, &str, &str)]) -> String {
    let mut yaml =
        "app:\n  layouts:\n    - id: main\n      root: true\n      children:\n".to_string();
    for (title, x1, y1, x2, y2) in positions {
        yaml.push_str(&format!(
            "        - id: {}\n          title: {}\n          position: {{x1: \"{}\", y1: \"{}\", x2: \"{}\", y2: \"{}\"}}\n",
            title.to_lowercase(),
            title,
            x1,
            y1,
            x2,
            y2
        ));
    }
    yaml
}

/// A box over the whole screen.
fn whole_box() -> String {
    boxes(&[("Whole", "0%", "0%", "100%", "100%")])
}

/// Boxes stacked from the top of the screen to its bottom, split at `splits`, the percentages
/// where each box ends and the next begins.
fn stacked_rows(splits: &[&str]) -> String {
    let edges: Vec<&str> = ["0%"]
        .into_iter()
        .chain(splits.iter().copied())
        .chain(["100%"])
        .collect();
    let titles = ["A", "B", "C", "D"];
    let positions: Vec<_> = edges
        .windows(2)
        .zip(titles)
        .map(|(edge, title)| (title, "0%", edge[0], "100%", edge[1]))
        .collect();
    boxes(&positions)
}

/// Boxes side by side from the left of the screen to its right, split at `splits`, the
/// percentages where each box ends and the next begins.
fn stacked_columns(splits: &[&str]) -> String {
    let edges: Vec<&str> = ["0%"]
        .into_iter()
        .chain(splits.iter().copied())
        .chain(["100%"])
        .collect();
    let titles = ["A", "B", "C", "D"];
    let positions: Vec<_> = edges
        .windows(2)
        .zip(titles)
        .map(|(edge, title)| (title, edge[0], "0%", edge[1], "100%"))
        .collect();
    boxes(&positions)
}

/// The character in cell (`x`, `y`) of `screen`; a space for the right half of a wide one.
fn char_at(screen: &Screen, x: usize, y: usize) -> char {
    screen.cells[y][x].chars().next().unwrap_or(' ')
}

/// A side of a box, or its scrollbar.
const SIDE: [char; 2] = ['│', '█'];
/// The bottom right corner of a box, or the resize handle drawn in it.
const BOTTOM_RIGHT: [char; 2] = ['┘', '⋱'];

/// What is wrong with empty boxes laid out on `screen` whose borders should be exactly at
/// `columns` (each box's left border, the last box's right border last) and `rows` (likewise
/// top to bottom): every border where it belongs, each box's title on its top border, nothing
/// inside any box, and the screen never scrolled. Boxes that share an edge share its border,
/// the later box's drawn over the earlier's.
fn grid_problems(
    screen: &Screen,
    columns: &[usize],
    rows: &[usize],
    titles: &[&str],
) -> Vec<String> {
    let mut problems = Vec::new();
    if screen.scrolled > 0 {
        problems.push(format!(
            "the screen scrolled {} times: something was written past its last row or column",
            screen.scrolled
        ));
    }
    let (right, bottom) = (screen.width - 1, screen.height - 1);
    if columns.first() != Some(&0) || columns.last() != Some(&right) {
        problems.push(format!(
            "the borders expected at columns {:?} do not span the screen's {} columns",
            columns, screen.width
        ));
    }
    if rows.first() != Some(&0) || rows.last() != Some(&bottom) {
        problems.push(format!(
            "the borders expected at rows {:?} do not span the screen's {} rows",
            rows, screen.height
        ));
    }
    for y in 0..screen.height {
        let top = rows[..rows.len() - 1].contains(&y);
        for x in 0..screen.width {
            let left = columns[..columns.len() - 1].contains(&x);
            let expected: &[char] = match (left, x == right, top, y == bottom) {
                (true, _, true, _) => &['┌'],
                (_, true, true, _) => &['┐'],
                (true, _, _, true) => &['└'],
                (_, true, _, true) => &BOTTOM_RIGHT,
                (true, _, _, _) | (_, true, _, _) => &SIDE,
                // A top border holds its box's title.
                (_, _, true, _) | (_, _, _, true) => continue,
                _ => &[' '],
            };
            let found = char_at(screen, x, y);
            if !expected.contains(&found) {
                problems.push(format!(
                    "cell ({}, {}) is {:?}, not {:?}: {:?}",
                    x,
                    y,
                    found,
                    expected,
                    screen.row(y)
                ));
            }
        }
    }
    // Boxes stacked top to bottom each have their own top border; side by side, they share one.
    for (i, title) in titles.iter().enumerate() {
        let y = if rows.len() > 2 { rows[i] } else { 0 };
        let row = screen.row(y);
        if !row.contains(title) {
            problems.push(format!("row {} lacks the title {}: {:?}", y, title, row));
        }
    }
    problems
}

/// A terminal size, and the columns and rows the borders of a layout drawn at that size are on.
struct Borders {
    cols: u16,
    rows: u16,
    /// The column of each box's left border, the last box's right border last.
    at_columns: Vec<usize>,
    /// The row of each box's top border, the last box's bottom border last.
    at_rows: Vec<usize>,
}

/// One way of laying out boxes to the edges of the screen, and where their borders are at
/// each size the tests draw it.
struct EdgeLayout {
    what: &'static str,
    yaml: String,
    titles: &'static [&'static str],
    sizes: Vec<Borders>,
}

/// Boxes laid out to the edges of the screen: one box over all of it, boxes stacked top to
/// bottom split at a third and two thirds and at odd percentages, and the same side by side.
/// Each at 30, 48 and 70 rows (or columns), its borders where 0% is the first row and 100% the
/// last, a box's bottom border on the row the next box's top border shares.
fn edge_layouts() -> Vec<EdgeLayout> {
    let full = |cols: u16, rows: u16| Borders {
        cols,
        rows,
        at_columns: vec![0, usize::from(cols) - 1],
        at_rows: vec![0, usize::from(rows) - 1],
    };
    let across = |cols: u16, between: [usize; 2]| Borders {
        cols,
        rows: 20,
        at_columns: vec![0, between[0], between[1], usize::from(cols) - 1],
        at_rows: vec![0, 19],
    };
    let down = |rows: u16, between: [usize; 2]| Borders {
        cols: 48,
        rows,
        at_columns: vec![0, 47],
        at_rows: vec![0, between[0], between[1], usize::from(rows) - 1],
    };
    vec![
        EdgeLayout {
            what: "a box from 0% to 100% both ways",
            yaml: whole_box(),
            titles: &["Whole"],
            sizes: vec![
                full(48, 30),
                full(48, 48),
                full(48, 70),
                full(30, 20),
                full(70, 20),
            ],
        },
        EdgeLayout {
            what: "boxes stacked at 33% and 66%",
            yaml: stacked_rows(&["33%", "66%"]),
            titles: &["A", "B", "C"],
            // 33% of 29, 47 and 69 rows past the first is 9.57, 15.51 and 22.77; 66% is 19.14,
            // 31.02 and 45.54.
            sizes: vec![down(30, [10, 19]), down(48, [16, 31]), down(70, [23, 46])],
        },
        EdgeLayout {
            what: "boxes stacked at 12.5% and 57%",
            yaml: stacked_rows(&["12.5%", "57%"]),
            titles: &["A", "B", "C"],
            // 12.5% of 29, 47 and 69 is 3.625, 5.875 and 8.625; 57% is 16.53, 26.79 and 39.33.
            sizes: vec![down(30, [4, 17]), down(48, [6, 27]), down(70, [9, 39])],
        },
        EdgeLayout {
            what: "boxes side by side at 33% and 66%",
            yaml: stacked_columns(&["33%", "66%"]),
            titles: &["A", "B", "C"],
            sizes: vec![
                across(30, [10, 19]),
                across(48, [16, 31]),
                across(70, [23, 46]),
            ],
        },
        EdgeLayout {
            what: "boxes side by side at 21% and 57%",
            yaml: stacked_columns(&["21%", "57%"]),
            titles: &["A", "B", "C"],
            // 21% of 29, 47 and 69 columns past the first is 6.09, 9.87 and 14.49.
            sizes: vec![
                across(30, [6, 17]),
                across(48, [10, 27]),
                across(70, [14, 39]),
            ],
        },
    ]
}

impl LiveBoxMux {
    /// Waits until the screen shows `layout` with its borders where `borders` has them (see
    /// [`grid_problems`]).
    fn wait_for_edges(&mut self, layout: &EdgeLayout, borders: &Borders) {
        let what = format!(
            "{} at {}x{}",
            layout.what, self.screen.width, self.screen.height
        );
        self.wait_for_screen(&what, |screen| {
            grid_problems(screen, &borders.at_columns, &borders.at_rows, layout.titles)
        });
    }
}

#[test]
fn boxes_laid_out_to_the_screen_s_edges_end_on_its_last_row_and_column() {
    for layout in edge_layouts() {
        for borders in &layout.sizes {
            let fixture = Fixture::new();
            let config = fixture.write_config(&layout.yaml);
            let screen = Screen::new(usize::from(borders.cols), usize::from(borders.rows));
            let mut app = LiveBoxMux::spawn_in(&fixture, &config, &[], screen);
            app.wait_for_edges(&layout, borders);
        }
    }
}

#[test]
#[cfg(unix)]
fn a_frame_drawn_for_a_larger_terminal_than_it_reaches_never_scrolls_the_screen() {
    // The frame is drawn for 7 more columns and 5 more rows than the terminal has once it
    // arrives: each of its rows reaches past the last column, the last ones past the last row.
    for layout in edge_layouts() {
        for borders in &layout.sizes {
            let fixture = Fixture::new();
            let config = fixture.write_config(&layout.yaml);
            let screen = Screen::new(usize::from(borders.cols + 7), usize::from(borders.rows + 5));
            let mut app = LiveBoxMux::spawn_in(&fixture, &config, &[], screen);
            app.resize_with_first_frame_in_flight(borders.cols, borders.rows);
            app.wait_for_edges(&layout, borders);
        }
    }
}

/// Two layouts: `main`, shown, with a box saying HOME at its right, and `runs`, hidden, whose
/// box `run1_box` takes a share of the screen at its top left.
#[cfg(unix)]
const HIDDEN_RUNS: &str = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: home
          title: Home
          position: {x1: "70%", y1: "0%", x2: "100%", y2: "100%"}
          content: HOME
    - id: runs
      children:
        - id: run1_box
          title: Run one
          position: {x1: "0%", y1: "0%", x2: "60%", y2: "50%"}
          content: idle
"#;

/// A terminal program that writes the size it has, as `stty size` says it, to `markers/size` when
/// it starts and each time its terminal is resized, then runs until it is stopped.
#[cfg(unix)]
const SIZE_REPORTER: &str = "report() { stty size > markers/size.new && mv markers/size.new markers/size; }; trap report WINCH; report; while :; do sleep 0.05; done";

#[cfg(unix)]
impl Fixture {
    /// Waits until `markers/<name>` holds something other than `not`, and returns it.
    fn wait_for_marker_other_than(&self, name: &str, not: &str) -> String {
        let start = Instant::now();
        loop {
            let text = fs::read_to_string(self.marker(name)).unwrap_or_default();
            if !text.is_empty() && text != not {
                return text;
            }
            assert!(
                start.elapsed() < TIMEOUT,
                "marker {} still holds {:?}\nlog:\n{}",
                name,
                not,
                self.log()
            );
            thread::sleep(Duration::from_millis(20));
        }
    }
}

/// The size, as `stty size` writes it, of the inside of the box whose top left corner is the
/// screen's: from its top right and bottom left corners.
#[cfg(unix)]
fn top_left_box_inside(screen: &Screen) -> String {
    let right = (1..screen.width)
        .find(|&x| screen.cell(x, 0) == "┐")
        .expect("the box has a top right corner");
    let bottom = (1..screen.height)
        .find(|&y| screen.cell(0, y) == "└")
        .expect("the box has a bottom left corner");
    format!("{} {}\n", bottom - 1, right - 1)
}

#[cfg(unix)]
fn control_ok(fixture: &Fixture, args: &[&str]) {
    let output = fixture.control(args);
    assert!(
        output.status.success(),
        "`boxmux {}` failed: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
#[cfg(unix)]
fn a_terminal_started_in_a_hidden_layout_s_box_has_its_size_there_and_follows_the_screen() {
    let fixture = Fixture::new();
    let config = fixture.write_config(HIDDEN_RUNS);
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("HOME");

    // Started over the socket while its layout is hidden, the program has a size at once.
    control_ok(&fixture, &["spawn_pty_process", "run1_box", SIZE_REPORTER]);
    let hidden = fixture.wait_for_marker("size");
    assert!(app.screen.find("HOME").is_some(), "{}", app.report());

    // Shown, the box is the size the program was given.
    control_ok(&fixture, &["switch_active_layout", "runs"]);
    app.wait_until("the run's box to be shown", |screen| {
        screen.find("Run one").is_some() && screen.cell(0, 0) == "┌"
    });
    assert_eq!(top_left_box_inside(&app.screen), hidden, "{}", app.report());
    thread::sleep(Duration::from_millis(300));
    assert_eq!(fs::read_to_string(fixture.marker("size")).unwrap(), hidden);

    // The screen resized while the layout is hidden reaches the program, at the size its box has
    // when shown again.
    control_ok(&fixture, &["switch_active_layout", "main"]);
    app.wait_for_text("HOME");
    app.resize(80, 24);
    let resized = fixture.wait_for_marker_other_than("size", &hidden);
    control_ok(&fixture, &["switch_active_layout", "runs"]);
    app.wait_until("the run's box to be shown at the new size", |screen| {
        screen.find("Run one").is_some() && screen.cell(0, 0) == "┌"
    });
    assert_eq!(
        top_left_box_inside(&app.screen),
        resized,
        "{}",
        app.report()
    );
}

/// Script output that overwrites part of a styled line after a carriage return, read through the
/// test terminal's cells. Unix only, for the reason [`display_width`] gives.
#[cfg(unix)]
mod overwritten_styles {
    use super::*;

    /// A box whose script prints a red line, a line on a blue background and a bold one, each
    /// with its style reset and its first column overwritten after a carriage return.
    const OVERWRITES: &str = r#"
app:
  layouts:
    - id: main
      title: Overwrites
      root: true
      children:
        - id: out
          title: Out
          position: {x1: "0", y1: "0", x2: "40", y2: "8"}
          script: ['printf "\033[31mred line\033[0m\rX\n\033[44mblue line\033[0m\rY\n\033[1mbold line\033[0m\rZ\n"']
"#;

    /// The pens of the first cell of `shown` and of the rest of it, where it is on the screen,
    /// and of the cell after it: the box's own.
    fn pens(app: &LiveBoxMux, shown: &str) -> (Pen, Vec<Pen>, Pen) {
        let (x, y) = app
            .screen
            .find(shown)
            .unwrap_or_else(|| panic!("{:?} is not shown\n{}", shown, app.screen.text()));
        let end = x + shown.chars().count();
        let row = &app.screen.pens[y];
        (row[x], row[x + 1..end].to_vec(), row[end])
    }

    #[test]
    fn text_overwritten_after_a_carriage_return_keeps_the_style_of_each_column_it_does_not_cover() {
        let fixture = Fixture::new();
        let config = fixture.write_config(OVERWRITES);
        let mut app = LiveBoxMux::spawn(&fixture, &config);
        app.wait_for_text("Zold line");
        let red = Some(Ink::Indexed(1));
        let blue = Some(Ink::Indexed(4));

        let (first, rest, body) = pens(&app, "Xed line");
        assert!(
            body.fg != red && body.bg != blue && !body.bold,
            "the box's own style is none of the script's: {:?}",
            body
        );
        assert_eq!(
            first,
            body,
            "X is written after the reset\n{}",
            app.report()
        );
        let colored = Pen { fg: red, ..body };
        assert!(
            rest.iter().all(|pen| *pen == colored),
            "\"ed line\" stays red: {:?}\n{}",
            rest,
            app.report()
        );

        let (first, rest, body) = pens(&app, "Ylue line");
        assert_eq!(
            first,
            body,
            "Y is written after the reset\n{}",
            app.report()
        );
        let backed = Pen { bg: blue, ..body };
        assert!(
            rest.iter().all(|pen| *pen == backed),
            "\"lue line\" stays on blue: {:?}\n{}",
            rest,
            app.report()
        );

        let (first, rest, body) = pens(&app, "Zold line");
        assert_eq!(
            first,
            body,
            "Z is written after the reset\n{}",
            app.report()
        );
        let bold = Pen { bold: true, ..body };
        assert!(
            rest.iter().all(|pen| *pen == bold),
            "\"old line\" stays bold: {:?}\n{}",
            rest,
            app.report()
        );
    }
}
