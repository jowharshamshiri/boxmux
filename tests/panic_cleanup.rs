//! A panic must not leave scripts running. This test runs itself again as a child process that
//! starts a piped script and a PTY script, then panics; once the child has ended, neither
//! script may still be running.
//!
//! It has its own `main` (no test harness) so the child can be this same executable.

use boxmux_lib::platform::{self, Shell, TreeSignal};
use boxmux_lib::runtime::executor::{Executor, RunSpec};
use boxmux_lib::runtime::pty::TerminalSize;
use boxmux_lib::{ExecutionMode, RunOrigin};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// How long the test waits for something a script does (getting ready, going away) before it
/// calls it hung and fails, naming what it waited for. It bounds a hang, not the work: on a
/// machine loaded well past its cores starting one process can take seconds. The scripts sleep
/// longer than this, so one still running when the bound passes was not killed.
const HANG_BOUND: Duration = Duration::from_secs(120);

/// The scripts sleep longer than the hang bound. The PTY script ignores the hangup it gets when
/// its terminal closes with the child, which would otherwise end it without any cleanup; it says
/// when it is ready.
#[cfg(unix)]
const PTY_SCRIPT: &str = "trap '' HUP; echo ready; sleep 600";
#[cfg(windows)]
const PTY_SCRIPT: &str = "echo ready; Start-Sleep 600";
const PIPED_SCRIPT: &str = "sleep 600";

fn main() {
    if std::env::args().nth(1).as_deref() == Some("child") {
        child();
    }
    // Reported like the standard test harness does, so test runners can read the outcome.
    match std::panic::catch_unwind(parent) {
        Ok(()) => println!("test panic_cleanup ... ok"),
        Err(_) => {
            println!("test panic_cleanup ... FAILED");
            std::process::exit(101);
        }
    }
}

/// Starts both scripts, prints their process ids and panics.
fn child() {
    platform::kill_process_trees_on_panic();
    let (sender, _events) = crossbeam_channel::unbounded();
    let mut executor = Executor::new(sender, Shell::default(), Vec::new(), Vec::new());
    for (box_id, mode, script) in [
        ("piped", ExecutionMode::Thread, PIPED_SCRIPT),
        ("terminal", ExecutionMode::Pty, PTY_SCRIPT),
    ] {
        let spec = RunSpec {
            origin: RunOrigin::BoxScript {
                box_id: box_id.to_string(),
            },
            target_box: box_id.to_string(),
            script: vec![script.to_string()],
            mode,
            terminal_size: Some(TerminalSize { rows: 10, cols: 40 }),
            environment: Vec::new(),
        };
        executor
            .start(spec, "out".to_string())
            .expect("the script starts");
    }
    let deadline = Instant::now() + HANG_BOUND;
    while !executor.runs().any(|(_, run)| {
        run.pty()
            .is_some_and(|pty| pty.snapshot().lines.iter().any(|l| l.contains("ready")))
    }) {
        assert!(Instant::now() < deadline, "the PTY script never got ready");
        std::thread::sleep(Duration::from_millis(20));
    }
    let pids: Vec<String> = executor
        .runs()
        .map(|(_, run)| run.pid().to_string())
        .collect();
    let mut out = std::io::stdout();
    writeln!(out, "{}", pids.join(" ")).unwrap();
    out.flush().unwrap();
    // Release builds abort on a panic, so the executor's destructor never runs there; it must
    // not run here either, or it would do the cleanup under test.
    std::mem::forget(executor);
    panic!("a panic on purpose");
}

fn parent() {
    let mut child = Command::new(std::env::current_exe().unwrap())
        .arg("child")
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the child starts");
    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let pids: Vec<u32> = line
        .split_whitespace()
        .map(|pid| pid.parse().unwrap())
        .collect();
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(101), "the child ends with a panic");
    assert_eq!(pids.len(), 2, "the child reports both scripts: {:?}", line);

    let deadline = Instant::now() + HANG_BOUND;
    let survivors = loop {
        let survivors: Vec<u32> = pids.iter().copied().filter(|&pid| running(pid)).collect();
        if survivors.is_empty() || Instant::now() > deadline {
            break survivors;
        }
        std::thread::sleep(Duration::from_millis(50));
    };
    for &pid in &survivors {
        platform::signal_tree(pid, TreeSignal::Kill);
    }
    assert!(
        survivors.is_empty(),
        "scripts {:?} outlived the panic",
        survivors
    );
}

/// Whether any process of the tree led by `pid` is running.
#[cfg(unix)]
fn running(pid: u32) -> bool {
    // SAFETY: signal 0 only checks that the process group exists.
    unsafe { libc::kill(-(pid as libc::pid_t), 0) == 0 }
}

#[cfg(windows)]
fn running(pid: u32) -> bool {
    let output = Command::new("tasklist")
        .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
        .output()
        .expect("tasklist runs");
    String::from_utf8_lossy(&output.stdout).contains(&format!("\"{}\"", pid))
}
