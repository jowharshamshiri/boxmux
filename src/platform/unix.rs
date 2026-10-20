use super::TreeSignal;
use std::io;
use std::os::unix::process::CommandExt;
use std::process::Command;

/// Detaches a script's process from boxmux's terminal: stdin is null and the process runs in a
/// new session, so it has no controlling terminal (it cannot reset raw mode or mouse tracking)
/// and is the leader of a process group that can be signalled as a whole.
pub fn detach(command: &mut Command) {
    command.stdin(std::process::Stdio::null());
    // SAFETY: setsid is async-signal-safe and only affects the freshly forked child.
    unsafe {
        command.pre_exec(|| {
            if libc::setsid() == -1 {
                return Err(io::Error::last_os_error());
            }
            Ok(())
        });
    }
}

/// Blocks until process `pid` (a child of this process) has exited, without reaping it.
pub fn wait_until_exited(pid: u32) -> io::Result<()> {
    loop {
        // SAFETY: zeroed siginfo_t is a valid out-parameter for waitid.
        let mut info: libc::siginfo_t = unsafe { std::mem::zeroed() };
        let result = unsafe {
            libc::waitid(
                libc::P_PID,
                pid as libc::id_t,
                &mut info,
                libc::WEXITED | libc::WNOWAIT,
            )
        };
        if result == 0 {
            return Ok(());
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::Interrupted {
            return Err(error);
        }
    }
}

/// Signals the process group led by `pid` (scripts and PTY sessions are group leaders).
pub fn signal_tree(pid: u32, signal: TreeSignal) {
    let signal = match signal {
        TreeSignal::Terminate => libc::SIGTERM,
        TreeSignal::Kill => libc::SIGKILL,
    };
    // SAFETY: killpg only sends a signal. ESRCH (group already gone) is the expected outcome
    // when the tree finished on its own and needs no handling.
    unsafe {
        libc::killpg(pid as libc::pid_t, signal);
    }
}

/// Sends `signal` (a Unix signal number) to the process group led by `pid`.
pub fn send_signal(pid: u32, signal: i32) -> io::Result<()> {
    // SAFETY: killpg only sends a signal.
    if unsafe { libc::killpg(pid as libc::pid_t, signal) } == -1 {
        return Err(io::Error::last_os_error());
    }
    Ok(())
}

/// The per-user default control socket: `$XDG_RUNTIME_DIR/boxmux.sock`, or
/// `/tmp/boxmux-<uid>.sock` when there is no runtime directory.
pub fn default_socket_name() -> String {
    match std::env::var("XDG_RUNTIME_DIR") {
        Ok(dir) if !dir.is_empty() => format!("{}/boxmux.sock", dir.trim_end_matches('/')),
        // SAFETY: getuid cannot fail.
        _ => format!("/tmp/boxmux-{}.sock", unsafe { libc::getuid() }),
    }
}

/// Puts the terminal back into raw mode when a child process (for example one that ran `stty`
/// on `/dev/tty`) switched it to cooked mode. crossterm cannot notice such an external reset, so
/// the real terminal attributes are inspected. A no-op when stdin is not a terminal or is raw.
pub fn ensure_raw_mode() {
    // SAFETY: tcgetattr/tcsetattr only read and write the termios struct.
    unsafe {
        let mut termios: libc::termios = std::mem::zeroed();
        if libc::tcgetattr(libc::STDIN_FILENO, &mut termios) != 0 {
            return;
        }
        if termios.c_lflag & (libc::ICANON | libc::ECHO) != 0 {
            libc::cfmakeraw(&mut termios);
            libc::tcsetattr(libc::STDIN_FILENO, libc::TCSANOW, &termios);
        }
    }
}

/// Signals that end or pause boxmux, delivered as runtime events.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessSignal {
    Terminate,
    Suspend,
    Resume,
}

/// Starts a thread that turns SIGTERM, SIGHUP, SIGINT and SIGQUIT into `Terminate`, SIGTSTP
/// into `Suspend` and SIGCONT into `Resume`.
pub fn listen_for_signals(
    deliver: impl Fn(ProcessSignal) -> bool + Send + 'static,
) -> io::Result<()> {
    use signal_hook::consts::{SIGCONT, SIGHUP, SIGINT, SIGQUIT, SIGTERM, SIGTSTP};
    let mut signals =
        signal_hook::iterator::Signals::new([SIGTERM, SIGHUP, SIGINT, SIGQUIT, SIGTSTP, SIGCONT])?;
    std::thread::Builder::new()
        .name("signals".into())
        .spawn(move || {
            for signal in signals.forever() {
                let event = match signal {
                    SIGTSTP => ProcessSignal::Suspend,
                    SIGCONT => ProcessSignal::Resume,
                    _ => ProcessSignal::Terminate,
                };
                if !deliver(event) {
                    break;
                }
            }
        })?;
    Ok(())
}

/// Stops this process the way the default SIGTSTP action would, after boxmux has restored the
/// terminal. Execution continues here when the process receives SIGCONT.
pub fn suspend_self() {
    // SAFETY: raising SIGSTOP on ourselves is always valid.
    unsafe {
        libc::raise(libc::SIGSTOP);
    }
}
