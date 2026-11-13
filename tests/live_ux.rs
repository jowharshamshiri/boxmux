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
use unicode_width::UnicodeWidthChar;

const WIDTH: u16 = 100;
const HEIGHT: u16 = 30;
// PowerShell can take seconds to start on a cold Windows machine.
const TIMEOUT: Duration = Duration::from_secs(30);

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
        let pair = native_pty_system()
            .openpty(PtySize {
                rows: HEIGHT,
                cols: WIDTH,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("a PTY");
        let mut command = CommandBuilder::new(binary());
        command.arg(config);
        command.args(extra);
        // A fixed theme: boxmux does not ask the terminal for its colors, which this test
        // terminal would not answer.
        command.args([
            "--dark",
            "--frame-delay",
            "10",
            "--log-level",
            "debug",
            "--log-file",
        ]);
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
        // Keep the master open for as long as the reader runs.
        std::mem::forget(pair.master);
        LiveBoxMux {
            child,
            writer,
            output: rx,
            raw: Vec::new(),
            screen: Screen::new(WIDTH as usize, HEIGHT as usize),
            log: fixture.path("boxmux.log"),
            _reader: reader_thread,
        }
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
            "screen:\n{}\nlast output ({} bytes in all): {}\nexited: {:?}\nlog:\n{}",
            self.screen.text(),
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

    /// Waits for boxmux to exit and returns whether it exited successfully.
    fn wait_for_exit(&mut self) -> bool {
        let start = Instant::now();
        loop {
            self.pump(Duration::from_millis(20));
            if let Some(status) = self.child.try_wait().unwrap() {
                // Collect what was written just before exiting.
                self.pump(Duration::from_millis(100));
                return status.success();
            }
            assert!(start.elapsed() < TIMEOUT, "boxmux did not exit");
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
                r#"'$line = Read-Host "ready>"; Write-Output "got:$line"; Start-Sleep 30'"#
            } else {
                r#"'printf "ready> "; read line; echo "got:$line"; sleep 30'"#
            },
        ),
    );
    let mut app = LiveBoxMux::spawn(&fixture, &config);
    app.wait_for_text("ready>");
    app.send("abc q\r");
    app.wait_for_text("got:abc q");
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
          script: ['sleep 300 & echo $! > markers/child', 'echo started', 'wait']
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
            start.elapsed() < Duration::from_secs(3),
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

/// Enough of a terminal to read what boxmux draws: cursor movement, erasing and UTF-8 text.
struct Screen {
    width: usize,
    height: usize,
    cells: Vec<Vec<char>>,
    x: usize,
    y: usize,
    wrap_pending: bool,
    pending: Vec<u8>,
}

impl Screen {
    fn new(width: usize, height: usize) -> Screen {
        Screen {
            width,
            height,
            cells: vec![vec![' '; width]; height],
            x: 0,
            y: 0,
            wrap_pending: false,
            pending: Vec::new(),
        }
    }

    fn apply(&mut self, bytes: &[u8]) {
        self.pending.extend_from_slice(bytes);
        let data = std::mem::take(&mut self.pending);
        let mut i = 0;
        while i < data.len() {
            match data[i] {
                0x1b => match self.escape(&data, i) {
                    Some(next) => i = next,
                    None => {
                        self.pending = data[i..].to_vec();
                        return;
                    }
                },
                b'\r' => {
                    self.x = 0;
                    self.wrap_pending = false;
                    i += 1;
                }
                b'\n' => {
                    self.y = (self.y + 1).min(self.height - 1);
                    self.wrap_pending = false;
                    i += 1;
                }
                byte if byte < 0x20 => i += 1,
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
                if !private {
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
                                row.fill(' ');
                            }
                        }
                        b'K' => {
                            let (from, to) = match params.first() {
                                Some(1) => (0, self.x + 1),
                                Some(2) => (0, self.width),
                                _ => (self.x, self.width),
                            };
                            for x in from..to.min(self.width) {
                                self.cells[self.y][x] = ' ';
                            }
                        }
                        b'X' => {
                            for x in self.x..(self.x + n(0)).min(self.width) {
                                self.cells[self.y][x] = ' ';
                            }
                        }
                        _ => {}
                    }
                }
                Some(j + 1)
            }
            b']' => {
                let mut j = i + 2;
                loop {
                    match *data.get(j)? {
                        0x07 => return Some(j + 1),
                        0x1b if data.get(j + 1) == Some(&b'\\') => return Some(j + 2),
                        _ => j += 1,
                    }
                }
            }
            _ => Some(i + 2),
        }
    }

    fn put(&mut self, ch: char) {
        let width = ch.width().unwrap_or(0);
        if width == 0 {
            return;
        }
        // Like a real terminal, a character after the last column goes to the next line.
        if self.wrap_pending || self.x + width > self.width {
            self.x = 0;
            self.y = (self.y + 1).min(self.height - 1);
            self.wrap_pending = false;
        }
        self.cells[self.y][self.x] = ch;
        if width == 2 && self.x + 1 < self.width {
            self.cells[self.y][self.x + 1] = '\0';
        }
        if self.x + width >= self.width {
            self.x = self.width - 1;
            self.wrap_pending = true;
        } else {
            self.x += width;
        }
    }

    fn row(&self, y: usize) -> String {
        self.cells[y].iter().filter(|c| **c != '\0').collect()
    }

    /// The cell where `needle` starts.
    fn find(&self, needle: &str) -> Option<(usize, usize)> {
        (0..self.height).find_map(|y| {
            let row = self.row(y);
            let byte = row.find(needle)?;
            Some((row[..byte].chars().map(|c| c.width().unwrap_or(0)).sum(), y))
        })
    }

    fn text(&self) -> String {
        (0..self.height)
            .map(|y| self.row(y))
            .collect::<Vec<_>>()
            .join("\n")
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
    app.send("q");
    assert!(app.wait_for_exit(), "q exits successfully");
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
    let suspended = String::from_utf8_lossy(&app.raw[before..]).to_string();
    assert!(
        suspended.contains("\x1b[?1049l"),
        "the alternate screen is left: {:?}",
        suspended
    );
    let before = app.raw.len();
    assert_eq!(unsafe { libc::kill(app.pid(), libc::SIGCONT) }, 0);
    let start = Instant::now();
    while !String::from_utf8_lossy(&app.raw[before..]).contains("\x1b[?1049h") {
        assert!(start.elapsed() < TIMEOUT, "boxmux did not come back");
        app.pump(Duration::from_millis(20));
    }
    app.wait_for_text("READY");
}
