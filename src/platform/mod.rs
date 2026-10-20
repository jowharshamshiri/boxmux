//! Everything that differs between Unix and Windows: how scripts are started, waited for and
//! killed (with the whole process tree they spawn), where the control socket lives, how
//! termination signals arrive, and how the terminal is kept in raw mode.

#[cfg(unix)]
mod unix;
#[cfg(windows)]
mod windows;

#[cfg(unix)]
pub use unix::*;
#[cfg(windows)]
pub use windows::*;

use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

/// The shell scripts run in, chosen with the app-level `shell` setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Bash,
    Sh,
    Zsh,
    Powershell,
    Pwsh,
}

impl Default for Shell {
    fn default() -> Self {
        if cfg!(windows) {
            Shell::Powershell
        } else {
            Shell::Bash
        }
    }
}

impl Shell {
    /// The program and the arguments that precede the script text. `interactive` is set for
    /// scripts in a PTY, which read what is typed into their box; other scripts have no input,
    /// so PowerShell is told to fail on a prompt rather than wait for input that never comes.
    pub fn invocation(&self, interactive: bool) -> (&'static str, &'static [&'static str]) {
        const POWERSHELL: &[&str] = &["-NoLogo", "-NoProfile", "-Command"];
        const POWERSHELL_BATCH: &[&str] = &["-NoLogo", "-NoProfile", "-NonInteractive", "-Command"];
        let powershell_args = if interactive {
            POWERSHELL
        } else {
            POWERSHELL_BATCH
        };
        match self {
            Shell::Bash => ("bash", &["-c"]),
            Shell::Sh => ("sh", &["-c"]),
            Shell::Zsh => ("zsh", &["-c"]),
            Shell::Powershell => ("powershell", powershell_args),
            Shell::Pwsh => ("pwsh", powershell_args),
        }
    }

    /// The full script text: `libs` sourced first, then the script lines.
    pub fn script_text(&self, libs: &[std::path::PathBuf], lines: &[String]) -> String {
        let mut text = String::new();
        for lib in libs {
            let path = lib.to_string_lossy();
            match self {
                // Single quotes keep the path literal; an embedded quote is closed, escaped and
                // reopened.
                Shell::Bash | Shell::Sh | Shell::Zsh => {
                    text.push_str(&format!(". '{}'\n", path.replace('\'', r"'\''")))
                }
                Shell::Powershell | Shell::Pwsh => {
                    text.push_str(&format!(". '{}'\n", path.replace('\'', "''")))
                }
            }
        }
        text.push_str(&lines.join("\n"));
        text
    }
}

/// A started process and the tree of processes it spawns, which can be terminated together.
///
/// The process stays reserved (not reaped) until its exit has been observed and recorded under
/// the lock, so termination can never signal an unrelated process that reused its id.
#[derive(Clone)]
pub struct ProcessTree {
    pid: u32,
    reaped: Arc<Mutex<bool>>,
}

impl ProcessTree {
    pub fn new(pid: u32) -> Self {
        ProcessTree {
            pid,
            reaped: Arc::new(Mutex::new(false)),
        }
    }

    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Waits for the process to exit using `wait`, which must block until it has exited and
    /// reap it. The caller must keep the process handle alive until this returns.
    ///
    /// On Unix the exit is first observed without reaping and the reap happens under the lock,
    /// so a concurrent [`terminate`](Self::terminate) never targets a reused pid. On Windows the
    /// open process handle keeps the id reserved, so the blocking wait happens outside the lock.
    pub fn wait_with<T>(&self, wait: impl FnOnce() -> std::io::Result<T>) -> std::io::Result<T> {
        if cfg!(unix) {
            wait_until_exited(self.pid)?;
            let mut reaped = lock(&self.reaped);
            let result = wait();
            *reaped = true;
            result
        } else {
            let result = wait();
            *lock(&self.reaped) = true;
            result
        }
    }

    /// Asks the process tree to stop, and forces it after `grace` if it is still running.
    pub fn terminate(&self, grace: std::time::Duration) {
        {
            let reaped = lock(&self.reaped);
            if *reaped {
                return;
            }
            signal_tree(self.pid, TreeSignal::Terminate);
        }
        let tree = self.clone();
        std::thread::Builder::new()
            .name(format!("kill-{}", self.pid))
            .spawn(move || {
                std::thread::sleep(grace);
                let reaped = lock(&tree.reaped);
                if !*reaped {
                    signal_tree(tree.pid, TreeSignal::Kill);
                }
            })
            .expect("spawning a thread must succeed");
    }

    /// Stops the process tree immediately.
    pub fn kill(&self) {
        let reaped = lock(&self.reaped);
        if !*reaped {
            signal_tree(self.pid, TreeSignal::Kill);
        }
    }
}

/// Locks a mutex whose data stays consistent even if a holder panicked (it is a plain flag),
/// so a poisoned lock is recovered rather than propagated.
fn lock(mutex: &Mutex<bool>) -> std::sync::MutexGuard<'_, bool> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeSignal {
    /// Polite stop: SIGTERM on Unix.
    Terminate,
    /// Forced stop: SIGKILL on Unix, terminating the tree on Windows.
    Kill,
}

/// The control socket name, as given with `--socket`, `BOXMUX_SOCKET`, or the per-user default.
pub fn socket_name(explicit: Option<&str>) -> String {
    if let Some(name) = explicit {
        return name.to_string();
    }
    if let Ok(name) = std::env::var("BOXMUX_SOCKET") {
        if !name.is_empty() {
            return name;
        }
    }
    default_socket_name()
}
