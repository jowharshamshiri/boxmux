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
        Self::spawn_in(
            fixture,
            config,
            extra,
            Screen::new(WIDTH as usize, HEIGHT as usize),
        )
    }

    /// Starts boxmux in a terminal that draws like `screen`.
    fn spawn_in(fixture: &Fixture, config: &Path, extra: &[&str], screen: Screen) -> LiveBoxMux {
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
            screen,
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

/// An OSC 8 hyperlink as a terminal holds it: its URI, and its `id` parameter when it has one.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Link {
    id: Option<String>,
    uri: String,
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
struct Screen {
    width: usize,
    height: usize,
    /// Each cell's cluster; the empty string is the right half of a two-column cluster.
    cells: Vec<Vec<String>>,
    /// The hyperlink of each cell.
    links: Vec<Vec<Option<Link>>>,
    /// The hyperlink text printed now goes in.
    link: Option<Link>,
    /// Every OSC 8 sequence boxmux wrote that is not a well-formed opening or closing one.
    malformed_links: Vec<String>,
    x: usize,
    y: usize,
    wrap_pending: bool,
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
            malformed_links: Vec::new(),
            x: 0,
            y: 0,
            wrap_pending: false,
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
        self.pending.extend_from_slice(bytes);
        let data = std::mem::take(&mut self.pending);
        let mut i = 0;
        while i < data.len() {
            match data[i] {
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
                    self.y = (self.y + 1).min(self.height - 1);
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
                                row.fill(" ".to_string());
                            }
                            for row in &mut self.links {
                                row.fill(None);
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
        // Like a real terminal, a character after the last column goes to the next line.
        if self.wrap_pending || self.x + width > self.width {
            self.x = 0;
            self.y = (self.y + 1).min(self.height - 1);
            self.wrap_pending = false;
        }
        let (x, y) = (self.x, self.y);
        self.unsplit(x, y);
        self.cells[y][x] = ch.to_string();
        self.links[y][x] = self.link.clone();
        if width == 2 && x + 1 < self.width {
            if x + 2 < self.width && self.cells[y][x + 2].is_empty() {
                self.cells[y][x + 2] = " ".to_string();
            }
            self.cells[y][x + 1] = String::new();
            self.links[y][x + 1] = self.link.clone();
        }
        self.last = Some((x, y));
        self.advance(x, width);
    }

    /// Moves the cursor past a cluster of `width` columns at column `x`.
    fn advance(&mut self, x: usize, width: usize) {
        if x + width >= self.width {
            self.x = self.width - 1;
            self.wrap_pending = true;
        } else {
            self.x = x + width;
        }
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

    /// Every edge of the box covering `(x1, y1, x2, y2)` is in its own column and row.
    fn assert_edges(screen: &Screen, id: &str, (x1, y1, x2, y2): Cells) {
        let fail = |what: &str| -> ! { panic!("box {}: {}\nscreen:\n{}", id, what, screen.text()) };
        let at = |x: usize, y: usize| screen.cell(x, y);
        if (at(x1, y1), at(x2, y1), at(x1, y2)) != ("┌", "┐", "└")
            || !["┘", "⋱"].contains(&at(x2, y2))
        {
            fail("a corner is not where the box's corners are");
        }
        for y in y1 + 1..y2 {
            if at(x1, y) != "│" || !["│", "█"].contains(&at(x2, y)) {
                fail(&format!("row {} is not closed by its borders", y));
            }
        }
        for x in x1 + 1..x2 {
            if !["─", "■"].contains(&at(x, y2)) {
                fail(&format!(
                    "column {} of the bottom border is {:?}",
                    x,
                    at(x, y2)
                ));
            }
        }
        if at(x1 + 1, y1) != "─" || at(x2 - 1, y1) != "─" {
            fail("the title or tabs reach the corners");
        }
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

    /// Checks the rows of [`WIDTH_ROWS`] and the tabs; `drawn` maps a row to what the terminal
    /// shows for it.
    fn assert_width_rows(screen: &Screen, drawn: fn(&str) -> String) {
        for (y, x1, x2, row) in WIDTH_ROWS {
            assert_eq!(
                screen.span(y, x1, x2),
                drawn(row),
                "row {} from column {}\n{}",
                y,
                x1,
                screen.text()
            );
        }
        let [a, b] = WIDTH_TABS;
        let tabs = screen.span(0, 50, 99);
        assert!(
            [a, b].iter().zip([b, a]).any(
                |(first, second)| tabs == drawn(&format!("┌── Tabs ─{}─{}───┐", first, second))
            ),
            "the tabs are {:?}\n{}",
            tabs,
            screen.text()
        );
    }

    /// Runs boxmux on [`WIDTHS`] in `screen`, and checks that every box's edges stay in their
    /// columns and its rows read as `drawn` makes [`WIDTH_ROWS`]. Returns the running app.
    fn draw_widths(screen: Screen, drawn: fn(&str) -> String) -> (Fixture, LiveBoxMux) {
        let fixture = Fixture::new();
        let config = fixture.write_config(WIDTHS);
        let mut app = LiveBoxMux::spawn_in(&fixture, &config, &[], screen);
        // Both scripts ran: their output opened a tab each in `tabs`, after its own.
        app.wait_until("both tabs", |screen| {
            let top = screen.row(0);
            top.contains("Sunrise") && top.contains("日本 ·")
        });
        app.wait_for_text("Rise");
        // Let the last frames arrive.
        app.pump(Duration::from_millis(300));
        for (id, cells) in WIDTH_BOXES {
            assert_edges(&app.screen, id, cells);
        }
        assert_width_rows(&app.screen, drawn);
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

    /// Checks that every box keeps its edges, and that inside its border `wrap` shows only `wrap`
    /// on its fourth row and `dawn` only `dawn` on its third: the cells of what they showed
    /// before are blank. `wrap` and `dawn` are the rows as a terminal giving every cluster the
    /// width BoxMux does reads them; `drawn` maps them to what this terminal shows.
    fn assert_replaced(app: &LiveBoxMux, drawn: fn(&str) -> String, wrap: &str, dawn: &str) {
        for (id, cells) in WIDTH_BOXES {
            assert_edges(&app.screen, id, cells);
        }
        // wrap: 18 cells inside the border, 8 rows; dawn: 28 cells, 5 rows.
        for ((cells, rows, at, row), id) in [
            ((WIDTH_BOXES[1].1), 8, 3, wrap),
            ((WIDTH_BOXES[0].1), 5, 2, dawn),
        ]
        .into_iter()
        .zip(["wrap", "dawn"])
        {
            let mut expected = vec![drawn(&" ".repeat(cells.2 - cells.0 - 1)); rows];
            expected[at] = drawn(row);
            assert_eq!(
                interior(&app.screen, cells),
                expected,
                "inside {}\n{}",
                id,
                app.screen.text()
            );
        }
    }

    /// Replaces the text of `wrap` and `dawn` twice and checks that nothing of what they showed
    /// before is left: first wide, ambiguous and combining text with plain text, then plain text
    /// with wide text over the same cells. Each box shows its new line, centered, and blanks.
    fn replace_widths(fixture: &Fixture, app: &mut LiveBoxMux, drawn: fn(&str) -> String) {
        replace_widths_text(fixture, "short", "ok");
        app.wait_until_gone("dawn →");
        app.wait_until_gone("sun →");
        app.pump(Duration::from_millis(300));
        // Centered in the 16 columns of wrap's content area and the 26 of dawn's, each one column
        // in from the border.
        assert_replaced(
            app,
            drawn,
            &format!("{}short{}", " ".repeat(6), " ".repeat(7)),
            &format!("{}ok{}", " ".repeat(13), " ".repeat(13)),
        );

        replace_widths_text(fixture, "🌅🌅🌅", "→ 日本 ☀\u{fe0f}");
        app.wait_until_gone("short");
        app.wait_for_text("→ 日本 ☀");
        app.pump(Duration::from_millis(300));
        assert_replaced(
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
          script: ['printf "go \033]8;id=term;https://example.com/term\033\\\\to 日本 terminals link text\033]8;;\033\\\\ end\n"; sleep 30']
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

    /// Runs boxmux on [`LINKS`] until `shown` is on the screen.
    fn start(shown: &str) -> (Fixture, LiveBoxMux) {
        let fixture = Fixture::new();
        let config = fixture.write_config(LINKS);
        let mut app = LiveBoxMux::spawn(&fixture, &config);
        app.wait_for_text(shown);
        // Let the last frames arrive.
        app.pump(Duration::from_millis(300));
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
