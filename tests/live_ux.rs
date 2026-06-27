use boxmux_lib::components::TabBar;
use boxmux_lib::{Bounds, InputBounds};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::fd::AsRawFd;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver};
use std::sync::Mutex;
use std::thread;
use std::time::{Duration, Instant};
use tempfile::TempDir;

const WIDTH: u16 = 100;
const HEIGHT: u16 = 30;
const TIMEOUT: Duration = Duration::from_secs(8);
static LIVE_UX_TEST_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn live_choice_text_click_executes_exact_choice() {
    let _lock = LIVE_UX_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _process_lock = LiveUxProcessLock::acquire();
    let fixture = Fixture::new();
    let yaml = fixture.write_yaml();
    let mut app = LiveBoxMux::spawn(&yaml, fixture.log_path());

    app.wait_for_text("Build Now");
    app.wait_for_text("Run Tests");
    app.click_text("Build Now");

    fixture.wait_for_marker("build", "build");
    fixture.assert_marker_missing("test");
}

#[test]
fn live_click_after_choice_text_does_not_execute_choice() {
    let _lock = LIVE_UX_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _process_lock = LiveUxProcessLock::acquire();
    let fixture = Fixture::new();
    let yaml = fixture.write_yaml();
    let mut app = LiveBoxMux::spawn(&yaml, fixture.log_path());

    let (x, y) = app.wait_for_text("Build Now");
    app.click_at(x + "Build Now".len() + 3, y);

    fixture.assert_marker_missing_after("build", Duration::from_millis(700));
    fixture.assert_marker_missing("test");
}

#[test]
fn live_close_glyph_click_closes_redirected_tab() {
    let _lock = LIVE_UX_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _process_lock = LiveUxProcessLock::acquire();
    let fixture = Fixture::new();
    let yaml = fixture.write_close_tab_yaml();
    let mut app = LiveBoxMux::spawn(&yaml, fixture.log_path());

    app.wait_for_text("Open Closeable");
    app.click_text("Open Closeable");
    app.wait_for_text("CLOSE_ME_PAYLOAD");

    let close_x = close_cell_for_output_fixture();
    app.click_at(close_x, 2);
    app.wait_until_text_absent("CLOSE_ME_PAYLOAD");
}

#[test]
fn live_machinefabric_shape_central_close_glyph_click_closes_redirected_tab() {
    let _lock = LIVE_UX_TEST_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let _process_lock = LiveUxProcessLock::acquire();
    let fixture = Fixture::new();
    let yaml = fixture.write_machinefabric_shape_yaml();
    let mut app = LiveBoxMux::spawn(&yaml, fixture.log_path());

    app.wait_for_text("Dev website run");
    app.click_text("Dev website run");
    app.wait_for_text("MACHINEFABRIC_CENTRAL_PAYLOAD");

    let (close_x, close_y) = close_cell_for_machinefabric_central_fixture();
    app.click_at(close_x, close_y);
    app.wait_until_text_absent("MACHINEFABRIC_CENTRAL_PAYLOAD");
}

fn close_cell_for_output_fixture() -> usize {
    let tab_labels = vec!["Content".to_string(), "Choice".to_string()];
    let tab_close_buttons = vec![false, true];
    let fg = Some("white".to_string());
    let bg = Some("black".to_string());
    let close_cells = (0..WIDTH as usize)
        .filter(|x| {
            TabBar::calculate_tab_close_click(
                *x,
                44,
                98,
                &tab_labels,
                &tab_close_buttons,
                0,
                &fg,
                &bg,
            ) == Some(1)
        })
        .collect::<Vec<_>>();

    assert_eq!(
        close_cells.len(),
        1,
        "fixture should expose exactly one close cell"
    );
    close_cells[0]
}

fn close_cell_for_machinefabric_central_fixture() -> (usize, usize) {
    let bounds = machinefabric_central_bounds();
    let tab_labels = vec!["Content".to_string(), "Choice".to_string()];
    let tab_close_buttons = vec![false, true];
    let fg = Some("white".to_string());
    let bg = Some("black".to_string());
    let close_cells = (0..WIDTH as usize)
        .filter(|x| {
            TabBar::calculate_tab_close_click(
                *x,
                bounds.x1,
                bounds.x2,
                &tab_labels,
                &tab_close_buttons,
                0,
                &fg,
                &bg,
            ) == Some(1)
        })
        .collect::<Vec<_>>();

    assert_eq!(
        close_cells.len(),
        1,
        "MachineFabric-shaped central panel should expose exactly one close cell"
    );
    (close_cells[0], bounds.y1)
}

fn machinefabric_central_bounds() -> Bounds {
    InputBounds {
        x1: "25%".to_string(),
        y1: "8%".to_string(),
        x2: "73%".to_string(),
        y2: "70%".to_string(),
    }
    .to_bounds(&Bounds {
        x1: 0,
        y1: 0,
        x2: WIDTH as usize - 1,
        y2: HEIGHT as usize - 1,
    })
}

struct Fixture {
    tmp: TempDir,
}

struct LiveUxProcessLock {
    file: File,
}

impl LiveUxProcessLock {
    fn acquire() -> Self {
        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open("/tmp/boxmux-live-ux.lock")
            .expect("failed to open live UX process lock");
        let rc = unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) };
        if rc != 0 {
            panic!(
                "failed to acquire live UX process lock: {}",
                std::io::Error::last_os_error()
            );
        }
        Self { file }
    }
}

impl Drop for LiveUxProcessLock {
    fn drop(&mut self) {
        let rc = unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
        if rc != 0 {
            panic!(
                "failed to release live UX process lock: {}",
                std::io::Error::last_os_error()
            );
        }
    }
}

impl Fixture {
    fn new() -> Self {
        Self {
            tmp: tempfile::tempdir().expect("failed to create live ux tempdir"),
        }
    }

    fn marker_path(&self, name: &str) -> PathBuf {
        self.tmp.path().join("markers").join(name)
    }

    fn log_path(&self) -> PathBuf {
        self.tmp.path().join("boxmux.log")
    }

    fn write_yaml(&self) -> PathBuf {
        let marker_dir = self.tmp.path().join("markers");
        fs::create_dir(&marker_dir).expect("failed to create marker dir");

        let yaml_path = self.tmp.path().join("live_ux.yaml");
        let build_marker = shell_quote(&self.marker_path("build"));
        let test_marker = shell_quote(&self.marker_path("test"));
        let yaml = format!(
            r#"app:
  layouts:
    - id: "live_ux"
      title: "Live UX"
      root: true
      children:
        - id: "menu"
          title: "Commands"
          position:
            x1: "2"
            y1: "2"
            x2: "42"
            y2: "14"
          tab_order: "1"
          choices:
            - id: "build"
              content: "Build Now"
              execution_mode: "Immediate"
              script: "printf build > {build_marker}"
            - id: "test"
              content: "Run Tests"
              execution_mode: "Immediate"
              script: "printf test > {test_marker}"
        - id: "output"
          title: "Output"
          position:
            x1: "44"
            y1: "2"
            x2: "98"
            y2: "14"
          content: "Waiting"
"#
        );
        fs::write(&yaml_path, yaml).expect("failed to write live ux yaml");
        yaml_path
    }

    fn write_close_tab_yaml(&self) -> PathBuf {
        let yaml_path = self.tmp.path().join("live_close_tab.yaml");
        let yaml = r#"app:
  layouts:
    - id: "live_close_tab"
      title: "Live Close Tab"
      root: true
      children:
        - id: "menu"
          title: "Commands"
          position:
            x1: "2"
            y1: "2"
            x2: "42"
            y2: "14"
          tab_order: "1"
          choices:
            - id: "close_me"
              content: "Open Closeable"
              execution_mode: "Immediate"
              redirect_output: "output"
              script: "printf CLOSE_ME_PAYLOAD"
        - id: "output"
          title: "Output"
          position:
            x1: "44"
            y1: "2"
            x2: "98"
            y2: "14"
          content: "READY"
"#;
        fs::write(&yaml_path, yaml).expect("failed to write live close tab yaml");
        yaml_path
    }

    fn write_machinefabric_shape_yaml(&self) -> PathBuf {
        let yaml_path = self.tmp.path().join("machinefabric_shape.yaml");
        let yaml = r#"app:
  layouts:
    - id: "machinefabric_shape"
      title: "MachineFabric Shape"
      root: true
      children:
        - id: "run_menu"
          title: "Run"
          position:
            x1: "1%"
            y1: "8%"
            x2: "24%"
            y2: "45%"
          tab_order: "1"
          choices:
            - id: "dev_run"
              content: "Dev website run"
              execution_mode: "Immediate"
              redirect_output: "output"
              script: "printf MACHINEFABRIC_CENTRAL_PAYLOAD"
        - id: "tests"
          title: "Test"
          position:
            x1: "1%"
            y1: "46%"
            x2: "24%"
            y2: "91%"
          content: "Website dev tests"
        - id: "output"
          title: "Content"
          position:
            x1: "25%"
            y1: "8%"
            x2: "73%"
            y2: "70%"
          content: "READY"
          scroll: true
          auto_scroll_bottom: true
        - id: "inspector"
          title: "Inspector"
          position:
            x1: "25%"
            y1: "71%"
            x2: "73%"
            y2: "91%"
          content: "Git status"
        - id: "right"
          title: "Fabric / Release / Models / Mon"
          position:
            x1: "74%"
            y1: "46%"
            x2: "99%"
            y2: "91%"
          content: "Fabric staging update"
        - id: "footer"
          title: "Content"
          position:
            x1: "0%"
            y1: "92%"
            x2: "100%"
            y2: "99%"
          content: "Run from repo root"
"#;
        fs::write(&yaml_path, yaml).expect("failed to write MachineFabric-shaped live yaml");
        yaml_path
    }

    fn wait_for_marker(&self, name: &str, expected: &str) {
        let path = self.marker_path(name);
        let start = Instant::now();
        loop {
            match fs::read_to_string(&path) {
                Ok(actual) if actual == expected => return,
                Ok(actual) => panic!(
                    "marker {} had unexpected content {:?}, expected {:?}",
                    path.display(),
                    actual,
                    expected
                ),
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    if start.elapsed() > TIMEOUT {
                        panic!(
                            "marker {} was not created within {:?}\nboxmux log:\n{}",
                            path.display(),
                            TIMEOUT,
                            self.boxmux_log()
                        );
                    }
                    thread::sleep(Duration::from_millis(25));
                }
                Err(err) => panic!("failed to read marker {}: {}", path.display(), err),
            }
        }
    }

    fn assert_marker_missing(&self, name: &str) {
        let path = self.marker_path(name);
        assert!(
            !path.exists(),
            "marker {} exists with content {:?}",
            path.display(),
            fs::read_to_string(&path).ok()
        );
    }

    fn assert_marker_missing_after(&self, name: &str, duration: Duration) {
        thread::sleep(duration);
        self.assert_marker_missing(name);
    }

    fn boxmux_log(&self) -> String {
        fs::read_to_string(self.log_path())
            .unwrap_or_else(|err| format!("failed to read {}: {}", self.log_path().display(), err))
    }
}

struct LiveBoxMux {
    child: Box<dyn portable_pty::Child + Send + Sync>,
    writer: Box<dyn Write + Send>,
    output: Receiver<Vec<u8>>,
    screen: TerminalScreen,
    log_path: PathBuf,
    _reader: thread::JoinHandle<()>,
}

impl LiveBoxMux {
    fn spawn(yaml_path: &Path, log_path: PathBuf) -> Self {
        let binary = option_env!("CARGO_BIN_EXE_boxmux")
            .expect("CARGO_BIN_EXE_boxmux must be set for live ux integration tests");
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let command = format!(
            "cd {} && exec {} {} --frame_delay 20 --log-level error --log-file {}",
            shell_quote(&repo_root),
            shell_quote(Path::new(binary)),
            shell_quote(yaml_path),
            shell_quote(&log_path)
        );

        let pty_system = native_pty_system();
        let pair = pty_system
            .openpty(PtySize {
                rows: HEIGHT,
                cols: WIDTH,
                pixel_width: 0,
                pixel_height: 0,
            })
            .expect("failed to open PTY for live boxmux test");

        let mut cmd = CommandBuilder::new("bash");
        cmd.arg("-lc");
        cmd.arg(command);

        let child = pair
            .slave
            .spawn_command(cmd)
            .expect("failed to spawn boxmux in PTY");
        let writer = pair
            .master
            .take_writer()
            .expect("failed to take PTY writer");
        let mut reader = pair
            .master
            .try_clone_reader()
            .expect("failed to clone PTY reader");
        let (tx, rx) = mpsc::channel();
        let reader_thread = thread::spawn(move || {
            let mut buf = [0_u8; 4096];
            loop {
                match reader.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if tx.send(buf[..n].to_vec()).is_err() {
                            break;
                        }
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            child,
            writer,
            output: rx,
            screen: TerminalScreen::new(WIDTH as usize, HEIGHT as usize),
            log_path,
            _reader: reader_thread,
        }
    }

    fn wait_for_text(&mut self, needle: &str) -> (usize, usize) {
        let start = Instant::now();
        loop {
            self.drain_available_output(Duration::from_millis(50));
            if let Some(pos) = self.screen.find_text(needle) {
                return pos;
            }
            if start.elapsed() > TIMEOUT {
                panic!(
                    "text {:?} was not rendered within {:?}\n{}\nboxmux log:\n{}",
                    needle,
                    TIMEOUT,
                    self.screen.text(),
                    self.boxmux_log()
                );
            }
        }
    }

    fn wait_until_text_absent(&mut self, needle: &str) {
        let start = Instant::now();
        loop {
            self.drain_available_output(Duration::from_millis(50));
            if self.screen.find_text(needle).is_none() {
                return;
            }
            if start.elapsed() > TIMEOUT {
                panic!(
                    "text {:?} was still rendered after {:?}\n{}\nboxmux log:\n{}",
                    needle,
                    TIMEOUT,
                    self.screen.text(),
                    self.boxmux_log()
                );
            }
        }
    }

    fn click_text(&mut self, text: &str) {
        let (x, y) = self.wait_for_text(text);
        self.click_at(x, y);
    }

    fn click_at(&mut self, zero_based_x: usize, zero_based_y: usize) {
        let x = zero_based_x + 1;
        let y = zero_based_y + 1;
        write!(self.writer, "\x1b[<0;{};{}M\x1b[<0;{};{}m", x, y, x, y)
            .expect("failed to write SGR mouse click to PTY");
        self.writer
            .flush()
            .expect("failed to flush PTY mouse click");
    }

    fn drain_available_output(&mut self, first_wait: Duration) {
        match self.output.recv_timeout(first_wait) {
            Ok(bytes) => self.screen.apply(&bytes),
            Err(mpsc::RecvTimeoutError::Timeout) => return,
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                panic!("boxmux PTY output disconnected\n{}", self.screen.text())
            }
        }

        while let Ok(bytes) = self.output.try_recv() {
            self.screen.apply(&bytes);
        }
    }

    fn boxmux_log(&self) -> String {
        fs::read_to_string(&self.log_path)
            .unwrap_or_else(|err| format!("failed to read {}: {}", self.log_path.display(), err))
    }
}

impl Drop for LiveBoxMux {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

struct TerminalScreen {
    width: usize,
    height: usize,
    cells: Vec<Vec<char>>,
    cursor_x: usize,
    cursor_y: usize,
    saved_cursor: Option<(usize, usize)>,
}

impl TerminalScreen {
    fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            cells: vec![vec![' '; width]; height],
            cursor_x: 0,
            cursor_y: 0,
            saved_cursor: None,
        }
    }

    fn apply(&mut self, bytes: &[u8]) {
        let mut i = 0;
        while i < bytes.len() {
            match bytes[i] {
                b'\x1b' => i = self.apply_escape(bytes, i + 1),
                b'\r' => {
                    self.cursor_x = 0;
                    i += 1;
                }
                b'\n' => {
                    self.cursor_y = (self.cursor_y + 1).min(self.height.saturating_sub(1));
                    i += 1;
                }
                b'\x08' => {
                    self.cursor_x = self.cursor_x.saturating_sub(1);
                    i += 1;
                }
                byte if byte.is_ascii_graphic() || byte == b' ' => {
                    self.put(byte as char);
                    i += 1;
                }
                byte if byte >= 0x80 => {
                    i += self.skip_utf8_width(bytes, i);
                }
                _ => i += 1,
            }
        }
    }

    fn apply_escape(&mut self, bytes: &[u8], i: usize) -> usize {
        if i >= bytes.len() {
            return i;
        }

        match bytes[i] {
            b'[' => self.apply_csi(bytes, i + 1),
            b']' => self.skip_osc(bytes, i + 1),
            b'7' => {
                self.saved_cursor = Some((self.cursor_x, self.cursor_y));
                i + 1
            }
            b'8' => {
                if let Some((x, y)) = self.saved_cursor {
                    self.cursor_x = x.min(self.width.saturating_sub(1));
                    self.cursor_y = y.min(self.height.saturating_sub(1));
                }
                i + 1
            }
            _ => i + 1,
        }
    }

    fn apply_csi(&mut self, bytes: &[u8], mut i: usize) -> usize {
        let start = i;
        while i < bytes.len() {
            let byte = bytes[i];
            if (0x40..=0x7e).contains(&byte) {
                let params = String::from_utf8_lossy(&bytes[start..i]);
                self.handle_csi(&params, byte as char);
                return i + 1;
            }
            i += 1;
        }
        i
    }

    fn handle_csi(&mut self, params: &str, command: char) {
        let clean_params = params
            .trim_start_matches('?')
            .split(';')
            .map(|part| part.parse::<usize>().unwrap_or(0))
            .collect::<Vec<_>>();
        let n = |index: usize, default: usize| -> usize {
            clean_params
                .get(index)
                .copied()
                .filter(|v| *v > 0)
                .unwrap_or(default)
        };

        match command {
            'H' | 'f' => {
                self.cursor_y = n(0, 1).saturating_sub(1).min(self.height.saturating_sub(1));
                self.cursor_x = n(1, 1).saturating_sub(1).min(self.width.saturating_sub(1));
            }
            'A' => self.cursor_y = self.cursor_y.saturating_sub(n(0, 1)),
            'B' => self.cursor_y = (self.cursor_y + n(0, 1)).min(self.height.saturating_sub(1)),
            'C' => self.cursor_x = (self.cursor_x + n(0, 1)).min(self.width.saturating_sub(1)),
            'D' => self.cursor_x = self.cursor_x.saturating_sub(n(0, 1)),
            'J' if clean_params.first().copied().unwrap_or(0) == 2 => self.clear(),
            'K' => {
                for x in self.cursor_x..self.width {
                    self.cells[self.cursor_y][x] = ' ';
                }
            }
            's' => self.saved_cursor = Some((self.cursor_x, self.cursor_y)),
            'u' => {
                if let Some((x, y)) = self.saved_cursor {
                    self.cursor_x = x.min(self.width.saturating_sub(1));
                    self.cursor_y = y.min(self.height.saturating_sub(1));
                }
            }
            _ => {}
        }
    }

    fn skip_osc(&self, bytes: &[u8], mut i: usize) -> usize {
        while i < bytes.len() {
            if bytes[i] == b'\x07' {
                return i + 1;
            }
            if bytes[i] == b'\x1b' && bytes.get(i + 1) == Some(&b'\\') {
                return i + 2;
            }
            i += 1;
        }
        i
    }

    fn skip_utf8_width(&mut self, bytes: &[u8], i: usize) -> usize {
        let width = if bytes[i] & 0b1110_0000 == 0b1100_0000 {
            2
        } else if bytes[i] & 0b1111_0000 == 0b1110_0000 {
            3
        } else if bytes[i] & 0b1111_1000 == 0b1111_0000 {
            4
        } else {
            1
        };
        self.put(' ');
        width
    }

    fn put(&mut self, ch: char) {
        if self.cursor_y < self.height && self.cursor_x < self.width {
            self.cells[self.cursor_y][self.cursor_x] = ch;
        }
        self.cursor_x += 1;
        if self.cursor_x >= self.width {
            self.cursor_x = 0;
            self.cursor_y = (self.cursor_y + 1).min(self.height.saturating_sub(1));
        }
    }

    fn clear(&mut self) {
        for row in &mut self.cells {
            row.fill(' ');
        }
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn find_text(&self, needle: &str) -> Option<(usize, usize)> {
        self.cells.iter().enumerate().find_map(|(y, row)| {
            let line = row.iter().collect::<String>();
            line.find(needle).map(|x| (x, y))
        })
    }

    fn text(&self) -> String {
        self.cells
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn shell_quote(path: &Path) -> String {
    let value = path
        .to_str()
        .unwrap_or_else(|| panic!("path is not valid UTF-8: {}", path.display()));
    format!("'{}'", value.replace('\'', "'\\''"))
}
