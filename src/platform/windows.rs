use super::TreeSignal;
use std::io;
use std::os::windows::process::CommandExt;
use std::process::Command;

const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

/// Detaches a script's process from boxmux's console: stdin is null, it gets its own process
/// group and no console window.
pub fn detach(command: &mut Command) {
    command.stdin(std::process::Stdio::null());
    command.creation_flags(CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);
}

/// On Windows the process handle held by the caller keeps the process id reserved until the
/// handle is closed, so there is nothing to wait for before reaping.
pub fn wait_until_exited(_pid: u32) -> io::Result<()> {
    Ok(())
}

/// Ends process `pid` and every process it started.
pub fn signal_tree(pid: u32, signal: TreeSignal) {
    let mut command = Command::new("taskkill");
    command.args(["/T", "/PID", &pid.to_string()]);
    if signal == TreeSignal::Kill {
        command.arg("/F");
    }
    command
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .creation_flags(CREATE_NO_WINDOW);
    // taskkill failing means the tree is already gone; there is nothing further to do.
    let _ = command.status();
}

/// Windows has no signals to send to a process tree.
pub fn send_signal(_pid: u32, signal: i32) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        format!(
            "signal {} cannot be sent on Windows; use kill_pty_process",
            signal
        ),
    ))
}

/// The per-user default control pipe, `\\.\pipe\boxmux-<user>`.
pub fn default_socket_name() -> String {
    let user = std::env::var("USERNAME").unwrap_or_else(|_| "user".to_string());
    format!("boxmux-{}", user)
}

/// The Windows console has no cooked mode a child could switch it back to.
pub fn ensure_raw_mode() {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSignal {
    Terminate,
    Suspend,
    Resume,
}

/// Turns Ctrl+C, Ctrl+Break and closing the console window into `Terminate`.
pub fn listen_for_signals(
    deliver: impl Fn(ProcessSignal) -> bool + Send + 'static,
) -> io::Result<()> {
    ctrlc::set_handler(move || {
        deliver(ProcessSignal::Terminate);
    })
    .map_err(|e| io::Error::other(e.to_string()))
}

/// Windows consoles cannot suspend a process.
pub fn suspend_self() {}

/// The console's background color is not queried on Windows; the theme comes from the
/// command line or `COLORFGBG`.
pub fn query_background_color(_timeout: std::time::Duration) -> Option<(u8, u8, u8)> {
    None
}
