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
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError};
use std::time::{Duration, Instant};

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
    /// Identifies the tree in [`LIVE_TREES`]; unlike the pid, it is never reused.
    id: u64,
    pid: u32,
    reaped: Arc<Mutex<bool>>,
}

/// Every process tree that has not been reaped yet. Release builds abort on a panic without
/// running destructors, so the panic hook kills these through this list.
static LIVE_TREES: Mutex<Vec<ProcessTree>> = Mutex::new(Vec::new());
static NEXT_TREE_ID: AtomicU64 = AtomicU64::new(0);

impl ProcessTree {
    pub fn new(pid: u32) -> Self {
        let tree = ProcessTree {
            id: NEXT_TREE_ID.fetch_add(1, Ordering::Relaxed),
            pid,
            reaped: Arc::new(Mutex::new(false)),
        };
        lock(&LIVE_TREES).push(tree.clone());
        tree
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
        let result = if cfg!(unix) {
            wait_until_exited(self.pid).and_then(|()| {
                let mut reaped = lock(&self.reaped);
                let result = wait();
                *reaped = true;
                result
            })
        } else {
            let result = wait();
            *lock(&self.reaped) = true;
            result
        };
        lock(&LIVE_TREES).retain(|tree| tree.id != self.id);
        result
    }

    /// Asks the process tree to stop, and forces it after `grace` if it is still running.
    pub fn terminate(&self, grace: Duration) {
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

/// Makes a panic kill every process tree that is still running before the process ends.
/// Hooks installed later (the terminal restore) run first.
pub fn kill_process_trees_on_panic() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        kill_live_trees();
        previous(info);
    }));
}

/// Kills every tree in [`LIVE_TREES`]. The panicking thread may hold one of the locks involved,
/// so each is tried only briefly instead of waited for.
fn kill_live_trees() {
    let Some(trees) = lock_briefly(&LIVE_TREES).map(|trees| trees.clone()) else {
        eprintln!("boxmux: cannot stop the running scripts: their list is locked");
        return;
    };
    for tree in trees {
        match lock_briefly(&tree.reaped) {
            Some(reaped) if !*reaped => signal_tree(tree.pid, TreeSignal::Kill),
            Some(_) => {}
            None => eprintln!("boxmux: cannot stop process {}: it is locked", tree.pid),
        }
    }
}

/// Locks a mutex whose data stays consistent even if a holder panicked (a flag, or the list
/// of trees), so a poisoned lock is recovered rather than propagated.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Like [`lock`], but gives up after 100 ms.
fn lock_briefly<T>(mutex: &Mutex<T>) -> Option<MutexGuard<'_, T>> {
    let deadline = Instant::now() + Duration::from_millis(100);
    loop {
        match mutex.try_lock() {
            Ok(guard) => return Some(guard),
            Err(TryLockError::Poisoned(poisoned)) => return Some(poisoned.into_inner()),
            Err(TryLockError::WouldBlock) if Instant::now() < deadline => {
                std::thread::sleep(Duration::from_millis(1))
            }
            Err(TryLockError::WouldBlock) => return None,
        }
    }
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
