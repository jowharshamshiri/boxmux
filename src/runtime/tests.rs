//! The runtime driven end to end without a terminal: configuration in, terminal events in,
//! frames out. Scripts run as real processes.

use super::display::MemoryDisplay;
use super::event::Event;
use super::Runtime;
use crate::model::app::{load_app_from_str, AppContext};
use crate::Config;
use crossterm::event::{
    Event as TermEvent, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::time::{Duration, Instant};

fn runtime(yaml: &str, cols: u16, rows: u16) -> Runtime<MemoryDisplay> {
    let app = load_app_from_str(yaml, "test.yaml").expect("test configuration loads");
    let ctx = AppContext::new(app, Config::new(1, false, false).unwrap());
    let (sender, events) = crossbeam_channel::unbounded();
    let mut runtime = Runtime::new(ctx, MemoryDisplay::new(cols, rows), sender, events, None);
    runtime.activate_layout_runs();
    runtime.render();
    runtime
}

/// Handles events and ticks until `done` holds, failing after five seconds.
fn pump_until(
    runtime: &mut Runtime<MemoryDisplay>,
    what: &str,
    done: impl Fn(&Runtime<MemoryDisplay>) -> bool,
) {
    // PowerShell takes seconds to start on a cold Windows machine.
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        while let Ok(event) = runtime.events.recv_timeout(Duration::from_millis(5)) {
            runtime.handle(event);
        }
        runtime.tick(Instant::now());
        if runtime.dirty {
            runtime.render();
        }
        if done(runtime) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}; screen:\n{}",
            what,
            runtime.display.rows().join("\n")
        );
    }
}

fn screen_contains(runtime: &Runtime<MemoryDisplay>, text: &str) -> bool {
    runtime.display.rows().iter().any(|row| row.contains(text))
}

/// The cell where `text` starts on screen.
fn find(runtime: &Runtime<MemoryDisplay>, text: &str) -> (u16, u16) {
    runtime
        .display
        .rows()
        .iter()
        .enumerate()
        .find_map(|(y, row)| {
            row.find(text)
                .map(|byte| (row[..byte].chars().count() as u16, y as u16))
        })
        .unwrap_or_else(|| {
            panic!(
                "'{}' is not on screen:\n{}",
                text,
                runtime.display.rows().join("\n")
            )
        })
}

fn mouse(kind: MouseEventKind, column: u16, row: u16) -> Event {
    Event::Input(TermEvent::Mouse(MouseEvent {
        kind,
        column,
        row,
        modifiers: KeyModifiers::NONE,
    }))
}

fn click(runtime: &mut Runtime<MemoryDisplay>, (x, y): (u16, u16)) {
    runtime.handle(mouse(MouseEventKind::Down(MouseButton::Left), x, y));
    runtime.handle(mouse(MouseEventKind::Up(MouseButton::Left), x, y));
    runtime.render();
}

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Input(TermEvent::Key(KeyEvent::new(code, modifiers)))
}

/// Writes `text` to standard error and exits with status 3, in the platform's default shell.
#[cfg(unix)]
const FAIL_LOUDLY: &str = "['echo broken >&2', 'exit 3']";
#[cfg(windows)]
const FAIL_LOUDLY: &str = "['[Console]::Error.WriteLine(\"broken\")', 'exit 3']";

/// Prints `started`, then waits on two child processes for 30 seconds.
#[cfg(unix)]
const LONG_RUNNING: &str = "['echo started', 'sleep 30 & sleep 30']";
#[cfg(windows)]
const LONG_RUNNING: &str = "['echo started', 'Start-Sleep 30']";

fn menu() -> String {
    MENU.replace("FAIL_LOUDLY", FAIL_LOUDLY)
}

const MENU: &str = r#"
app:
  layouts:
    - id: main
      root: true
      title: Menu test
      children:
        - id: menu
          title: Menu
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          tab_order: 1
          choices:
            - id: greet
              content: Say hello
              script: ["echo 'hello from greet'"]
            - id: fail
              content: Fail loudly
              script: FAIL_LOUDLY
        - id: log
          title: Log
          position: {x1: "40", y1: "0", x2: "79", y2: "9"}
          tab_order: 2
          content: |
            line 1
            line 2
            line 3
            line 4
            line 5
            line 6
            line 7
            line 8
            line 9
            line 10
            line 11
            line 12
"#;

/// The text of the tab `stream_id` of `box_id`, once it exists.
fn tab_content(runtime: &Runtime<MemoryDisplay>, box_id: &str, stream_id: &str) -> Vec<String> {
    runtime
        .ctx
        .app
        .get_muxbox_by_id(box_id)
        .and_then(|b| b.streams.get(stream_id))
        .map(|s| s.content.clone())
        .unwrap_or_default()
}

#[test]
fn clicking_a_choice_runs_its_script_into_a_tab_shown_when_clicked() {
    let mut runtime = runtime(&menu(), 80, 12);
    let at = find(&runtime, "Say hello");
    click(&mut runtime, at);
    pump_until(&mut runtime, "the choice to finish", |r| {
        let menu = r.ctx.app.get_muxbox_by_id("menu").unwrap();
        !menu.choices_stream().unwrap().choices.as_ref().unwrap()[0].waiting
    });
    assert_eq!(
        tab_content(&runtime, "menu", "menu/choice:menu:greet"),
        vec!["hello from greet".to_string()]
    );
    let menu = runtime.ctx.app.get_muxbox_by_id("menu").unwrap();
    let choices = menu.choices_stream().unwrap().choices.as_ref().unwrap();
    assert!(choices[0].selected);
    assert!(
        screen_contains(&runtime, "Fail loudly"),
        "the menu stays in view"
    );

    // The output's tab, in the tab bar, shows it.
    let at = find(&runtime, "Say hello ×");
    click(&mut runtime, at);
    assert!(screen_contains(&runtime, "hello from greet"));
}

#[test]
fn clicking_beside_a_choice_does_not_run_it() {
    let mut runtime = runtime(&menu(), 80, 12);
    let (x, y) = find(&runtime, "Say hello");
    // Past the end of the label, inside the box.
    click(&mut runtime, (x + "Say hello".len() as u16 + 5, y));
    std::thread::sleep(Duration::from_millis(200));
    pump_until(&mut runtime, "a frame", |_| true);
    assert!(
        runtime.executor.runs().next().is_none(),
        "no run was started"
    );
    assert!(!screen_contains(&runtime, "hello from greet"));
}

#[test]
fn a_failing_choice_shows_its_error_output_in_error_colors() {
    let mut runtime = runtime(&menu(), 80, 12);
    let at = find(&runtime, "Fail loudly");
    click(&mut runtime, at);
    pump_until(&mut runtime, "the error output's tab", |r| {
        r.ctx
            .app
            .get_muxbox_by_id("menu")
            .and_then(|b| b.streams.get("menu/choice:menu:fail"))
            .is_some_and(|s| s.status.is_error())
    });
    let at = find(&runtime, "Fail loudly ×");
    click(&mut runtime, at);
    assert!(screen_contains(&runtime, "broken"));
    let menu = runtime.ctx.app.get_muxbox_by_id("menu").unwrap();
    assert!(menu.in_error_state(), "exit code 3 is shown as an error");
}

/// Prints nothing and exits with status 0.
#[cfg(unix)]
const SILENT: &str = "[':']";
#[cfg(windows)]
const SILENT: &str = "['$null = 0']";

/// Prints nothing and exits with status 4.
const SILENT_FAILURE: &str = "['exit 4']";

/// Waits a second, then prints `late output`.
#[cfg(unix)]
const LATE: &str = "['sleep 1', 'echo late output']";
#[cfg(windows)]
const LATE: &str = "['Start-Sleep 1', 'echo \"late output\"']";

/// A menu box whose first choice runs `script` in `mode`.
fn quiet_menu(script: &str, mode: &str) -> String {
    format!(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: menu
          title: Menu
          position: {{x1: "0", y1: "0", x2: "39", y2: "9"}}
          tab_order: 1
          choices:
            - id: quiet
              content: Write a file
              execution_mode: {mode}
              script: {script}
            - id: other
              content: Something else
"#
    )
}

/// The menu box's tabs and visible tab, with the screen as drawn.
fn menu_view(runtime: &Runtime<MemoryDisplay>) -> (Vec<String>, Option<String>, Vec<String>) {
    let menu = runtime.ctx.app.get_muxbox_by_id("menu").unwrap();
    (
        menu.get_tab_stream_ids(),
        menu.get_selected_stream().map(|s| s.id.clone()),
        runtime.display.rows(),
    )
}

fn choice_waiting(runtime: &Runtime<MemoryDisplay>) -> bool {
    runtime
        .ctx
        .app
        .get_muxbox_by_id("menu")
        .and_then(|b| b.choices_stream())
        .and_then(|s| s.choices.as_ref())
        .is_some_and(|choices| choices[0].waiting)
}

#[test]
fn a_choice_that_prints_nothing_opens_no_tab_and_leaves_the_view_as_it_was() {
    for mode in ["Immediate", "Thread"] {
        let mut runtime = runtime(&quiet_menu(SILENT, mode), 80, 12);
        let before = menu_view(&runtime);
        let at = find(&runtime, "Write a file");
        click(&mut runtime, at);
        assert!(choice_waiting(&runtime), "{mode}: the choice started");
        pump_until(&mut runtime, "the silent run to finish", |r| {
            !choice_waiting(r) && r.executor.runs().next().is_none()
        });
        assert_eq!(
            menu_view(&runtime),
            before,
            "{mode}: the box keeps its tabs, its visible tab and its screen"
        );
        assert!(
            runtime.notice.is_none(),
            "{mode}: a clean exit says nothing"
        );
    }
}

#[test]
fn a_choice_that_fails_silently_says_so_in_a_notice_and_opens_no_tab() {
    let mut runtime = runtime(&quiet_menu(SILENT_FAILURE, "Immediate"), 80, 12);
    let before = menu_view(&runtime);
    let at = find(&runtime, "Write a file");
    click(&mut runtime, at);
    pump_until(&mut runtime, "the failure notice", |r| {
        r.notice.as_ref().is_some_and(|n| n.is_error)
    });
    let notice = runtime.notice.as_ref().unwrap().text.clone();
    assert!(
        notice.contains("quiet") && notice.contains('4'),
        "the notice names the choice and its exit code: {notice}"
    );
    let (tabs, visible, _) = menu_view(&runtime);
    assert_eq!((tabs, visible), (before.0, before.1));
}

#[test]
fn a_choice_that_prints_late_opens_its_tab_then_without_taking_focus() {
    for mode in ["Immediate", "Thread"] {
        let mut runtime = runtime(&quiet_menu(LATE, mode), 80, 12);
        let before = menu_view(&runtime);
        let at = find(&runtime, "Write a file");
        click(&mut runtime, at);
        // While the script waits it has printed nothing, and no tab is open.
        let quiet_until = Instant::now() + Duration::from_millis(300);
        pump_until(&mut runtime, "a moment of the run", |_| {
            Instant::now() >= quiet_until
        });
        assert!(choice_waiting(&runtime), "{mode}: still running");
        let (tabs, visible, _) = menu_view(&runtime);
        assert_eq!(
            (tabs, visible),
            (before.0.clone(), before.1.clone()),
            "{mode}: no tab before output"
        );
        assert!(
            screen_contains(&runtime, "Write a file …"),
            "{mode}: the menu stays in view, marking the running choice"
        );

        pump_until(&mut runtime, "the late output's tab", |r| {
            menu_view(r).0.len() > before.0.len()
        });
        let (tabs, visible, _) = menu_view(&runtime);
        let output_tab = tabs.last().unwrap().clone();
        assert_eq!(output_tab, "menu/choice:menu:quiet");
        assert_eq!(visible, before.1, "{mode}: the new tab does not take focus");
        assert!(
            screen_contains(&runtime, "Something else"),
            "{mode}: the menu stays in view"
        );
        pump_until(&mut runtime, "the run to finish", |r| !choice_waiting(r));
        let menu = runtime.ctx.app.get_muxbox_by_id("menu").unwrap();
        assert_eq!(
            menu.streams[&output_tab].content,
            vec!["late output".to_string()],
            "{mode}: the tab holds the output"
        );
        assert_eq!(menu.get_selected_stream().map(|s| s.id.clone()), before.1);
    }
}

/// Prints `term up`, then waits.
#[cfg(unix)]
const TERMINAL_UP: &str = "['echo term up', 'sleep 5']";
#[cfg(windows)]
const TERMINAL_UP: &str = "['echo \"term up\"', 'Start-Sleep 5']";

#[test]
fn a_terminal_choice_opens_and_shows_its_terminal_when_started() {
    let mut runtime = runtime(&quiet_menu(TERMINAL_UP, "Pty"), 80, 12);
    let at = find(&runtime, "Write a file");
    click(&mut runtime, at);
    // A terminal is a session to type into: its tab is open and shown before it draws.
    let (tabs, visible, _) = menu_view(&runtime);
    assert_eq!(tabs, vec!["menu/choices", "menu/choice:menu:quiet"]);
    assert_eq!(visible.as_deref(), Some("menu/choice:menu:quiet"));
    pump_until(&mut runtime, "the terminal's output", |r| {
        screen_contains(r, "term up")
    });
}

/// Prints `pressed` when it gets the `p` key and nothing for `s`.
#[cfg(unix)]
const KEY_HANDLERS: &str = "{p: ['echo pressed'], s: [':']}";
#[cfg(windows)]
const KEY_HANDLERS: &str = "{p: ['echo pressed'], s: ['$null = 0']}";

#[test]
fn a_key_handler_opens_its_tab_only_when_it_prints_and_shows_it_then() {
    let yaml = format!(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: notes
          title: Notes
          position: {{x1: "0", y1: "0", x2: "39", y2: "9"}}
          tab_order: 1
          content: some notes
          on_keypress: {KEY_HANDLERS}
"#
    );
    let mut runtime = runtime(&yaml, 80, 12);
    runtime.handle(key(KeyCode::Tab, KeyModifiers::NONE));
    let tabs = |r: &Runtime<MemoryDisplay>| {
        r.ctx
            .app
            .get_muxbox_by_id("notes")
            .unwrap()
            .get_tab_stream_ids()
    };
    runtime.handle(key(KeyCode::Char('s'), KeyModifiers::NONE));
    pump_until(&mut runtime, "the silent handler to finish", |r| {
        r.executor.runs().next().is_none()
    });
    assert_eq!(
        tabs(&runtime),
        vec!["notes/content"],
        "nothing printed, no tab"
    );
    assert!(screen_contains(&runtime, "some notes"));

    runtime.handle(key(KeyCode::Char('p'), KeyModifiers::NONE));
    pump_until(&mut runtime, "the handler's output", |r| {
        screen_contains(r, "pressed")
    });
    assert_eq!(tabs(&runtime), vec!["notes/content", "notes/key:notes:p"]);
}

#[test]
fn enter_runs_the_choice_selected_with_the_arrow_keys() {
    let mut runtime = runtime(&menu(), 80, 12);
    runtime.handle(key(KeyCode::Down, KeyModifiers::NONE));
    runtime.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    pump_until(&mut runtime, "the second choice's output", |r| {
        tab_content(r, "menu", "menu/choice:menu:fail") == vec!["broken".to_string()]
    });
}

#[test]
fn the_mouse_wheel_scrolls_the_box_under_the_pointer_by_lines() {
    let mut runtime = runtime(&menu(), 80, 12);
    assert!(screen_contains(&runtime, "line 1 "));
    let (x, y) = find(&runtime, "line 3");
    runtime.handle(mouse(MouseEventKind::ScrollDown, x, y));
    runtime.render();
    // The box shows 8 of 12 lines; one wheel step moves 3 lines.
    assert!(
        !screen_contains(&runtime, "line 3 "),
        "{}",
        runtime.display.rows().join("\n")
    );
    assert!(screen_contains(&runtime, "line 4 "));
    assert!(screen_contains(&runtime, "line 11"));
    runtime.handle(mouse(MouseEventKind::ScrollDown, x, y));
    runtime.render();
    assert!(
        screen_contains(&runtime, "line 12"),
        "scrolling stops at the last line"
    );
    assert!(screen_contains(&runtime, "line 5 "));
}

#[test]
fn tab_moves_the_focus_in_tab_order() {
    let mut runtime = runtime(&menu(), 80, 12);
    let focused = |r: &Runtime<MemoryDisplay>| {
        r.ctx
            .app
            .get_active_layout()
            .unwrap()
            .get_selected_muxboxes()
            .iter()
            .map(|b| b.id.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        focused(&runtime),
        vec!["menu"],
        "the lowest tab_order starts focused"
    );
    runtime.handle(key(KeyCode::Tab, KeyModifiers::NONE));
    assert_eq!(focused(&runtime), vec!["log"]);
    runtime.handle(key(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(focused(&runtime), vec!["menu"]);
}

#[test]
fn q_quits_but_q_with_a_modifier_does_not() {
    let mut runtime = runtime(&menu(), 80, 12);
    runtime.handle(key(KeyCode::Char('q'), KeyModifiers::CONTROL));
    runtime.handle(key(KeyCode::Char('q'), KeyModifiers::ALT));
    assert!(!runtime.quit);
    runtime.handle(key(KeyCode::Char('q'), KeyModifiers::NONE));
    assert!(runtime.quit);
}

#[test]
fn a_resize_lays_the_app_out_for_the_new_size() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: right
          title: Right half
          position: {x1: 50%, y1: 0%, x2: 100%, y2: 100%}
          content: here
"#;
    // The box covers the right half; its text is centered in it.
    let mut runtime = runtime(yaml, 40, 10);
    assert_eq!(find(&runtime, "here"), (28, 4));
    runtime.handle(Event::Input(TermEvent::Resize(100, 20)));
    runtime.render();
    assert_eq!(runtime.display.rows().len(), 20);
    assert_eq!(find(&runtime, "here"), (73, 9));
}

#[test]
fn a_higher_z_index_box_is_drawn_over_an_overlapping_one() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: top
          position: {x1: "0", y1: "0", x2: "19", y2: "4"}
          z_index: 5
          content: TOPTOPTOP
        - id: bottom
          position: {x1: "0", y1: "0", x2: "19", y2: "4"}
          content: BOTTOMBOT
"#;
    let mut runtime = runtime(yaml, 30, 8);
    assert!(screen_contains(&runtime, "TOPTOPTOP"));
    assert!(!screen_contains(&runtime, "BOTTOMBOT"));
    // Clicks go to the box that is drawn on top.
    click(&mut runtime, (3, 1));
    let selected = runtime.ctx.app.get_muxbox_by_id("top").unwrap().selected;
    assert_eq!(selected, Some(true));
}

#[test]
fn a_refreshing_script_runs_at_start_and_again_each_interval() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: counter
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          refresh_interval: 50
          append_output: true
          script: ['echo tick']
"#;
    let mut runtime = runtime(yaml, 40, 10);
    pump_until(&mut runtime, "three refreshes", |r| {
        r.ctx
            .app
            .get_muxbox_by_id("counter")
            .unwrap()
            .get_selected_stream()
            .is_some_and(|s| s.content.iter().filter(|l| l.as_str() == "tick").count() >= 3)
    });
}

#[test]
fn a_long_running_script_does_not_hold_up_other_boxes() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: streaming
          position: {x1: "0", y1: "0", x2: "39", y2: "4"}
          execution_mode: Thread
          script: ['echo slow-started', 'sleep 30']
        - id: waiting
          position: {x1: "0", y1: "5", x2: "39", y2: "9"}
          execution_mode: Immediate
          script: ['sleep 30']
        - id: counter
          position: {x1: "40", y1: "0", x2: "79", y2: "9"}
          refresh_interval: 50
          append_output: true
          script: ['echo tick']
"#;
    let mut runtime = runtime(yaml, 80, 10);
    pump_until(&mut runtime, "the other boxes to keep working", |r| {
        let ticks = r
            .ctx
            .app
            .get_muxbox_by_id("counter")
            .unwrap()
            .get_selected_stream()
            .is_some_and(|s| s.content.iter().filter(|l| l.as_str() == "tick").count() >= 3);
        ticks && screen_contains(r, "slow-started")
    });
    // Both slow scripts are still running while the counter refreshed and the streaming box
    // showed its first line.
    for box_id in ["streaming", "waiting"] {
        let origin = crate::model::stream::RunOrigin::BoxScript {
            box_id: box_id.into(),
        };
        assert!(
            runtime.executor.run_for_origin(&origin).is_some(),
            "box '{}' finished early",
            box_id
        );
    }
}

#[test]
fn a_box_without_refresh_interval_runs_its_script_once() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: once
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          append_output: true
          script: ['echo ran']
"#;
    let mut runtime = runtime(yaml, 40, 10);
    pump_until(&mut runtime, "the first run", |r| screen_contains(r, "ran"));
    let started = Instant::now();
    while started.elapsed() < Duration::from_millis(300) {
        pump_until(&mut runtime, "a tick", |_| true);
    }
    let stream = runtime
        .ctx
        .app
        .get_muxbox_by_id("once")
        .unwrap()
        .get_selected_stream()
        .unwrap()
        .clone();
    assert_eq!(stream.content, vec!["ran"], "the script ran exactly once");
}

#[test]
fn quitting_stops_every_running_process() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: sleeper
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          execution_mode: Thread
          script: LONG_RUNNING
"#
    .replace("LONG_RUNNING", LONG_RUNNING);
    let app = load_app_from_str(&yaml, "test.yaml").unwrap();
    let ctx = AppContext::new(app, Config::new(1, false, false).unwrap());
    let (sender, events) = crossbeam_channel::unbounded();
    let mut runtime = Runtime::new(
        ctx,
        MemoryDisplay::new(40, 10),
        sender.clone(),
        events,
        None,
    );
    runtime.activate_layout_runs();
    pump_until(&mut runtime, "the script to start", |r| {
        screen_contains(r, "started")
    });
    let pids: Vec<u32> = runtime.executor.runs().map(|(_, run)| run.pid()).collect();
    assert_eq!(pids.len(), 1);

    sender
        .send(key(KeyCode::Char('q'), KeyModifiers::NONE))
        .unwrap();
    runtime.run().expect("the loop ends on q");

    #[cfg(unix)]
    {
        // The script and both of its background children are killed and reaped: nothing is
        // left in the process group.
        let group = pids[0] as i32;
        let deadline = Instant::now() + Duration::from_secs(2);
        while unsafe { libc::kill(-group, 0) } == 0 {
            assert!(
                Instant::now() < deadline,
                "process group {} survived quitting",
                group
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

#[test]
fn every_shipped_layout_renders_at_any_screen_size() {
    let mut paths: Vec<std::path::PathBuf> = ["layouts", "examples"]
        .iter()
        .flat_map(|dir| std::fs::read_dir(dir).unwrap())
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    paths.sort();
    for path in paths {
        let app = crate::load_app_from_yaml(&path).unwrap();
        for (cols, rows) in [(1, 1), (7, 3), (40, 12), (120, 40), (300, 90)] {
            let ctx = AppContext::new(app.clone(), Config::new(1, false, false).unwrap());
            let (sender, events) = crossbeam_channel::unbounded();
            // Scripts are not started: this checks drawing only.
            let mut runtime =
                Runtime::new(ctx, MemoryDisplay::new(cols, rows), sender, events, None);
            runtime.render();
            let screen = runtime.display.rows();
            assert_eq!(screen.len(), usize::from(rows), "{}", path.display());
            if std::env::var("BOXMUX_SHOW_LAYOUTS").is_ok() && cols == 120 {
                println!("=== {}\n{}", path.display(), screen.join("\n"));
            }
        }
    }
}

/// Prints an OSC 52 clipboard write of "hi" (base64 `aGk=`), then waits.
#[cfg(unix)]
const COPY_HI: &str = r#"["printf '\\033]52;c;aGk=\\007copied'", "sleep 5"]"#;
#[cfg(windows)]
const COPY_HI: &str =
    r#"["Write-Host -NoNewline \"$([char]27)]52;c;aGk=$([char]7)copied\"", "Start-Sleep 5"]"#;

#[test]
fn a_terminal_program_can_copy_to_the_clipboard() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: term
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          execution_mode: Pty
          script: COPY_HI
"#
    .replace("COPY_HI", COPY_HI);
    let mut runtime = runtime(&yaml, 40, 10);
    pump_until(&mut runtime, "the copy", |r| r.display.clipboard.is_some());
    assert_eq!(runtime.display.clipboard.as_deref(), Some("hi"));
}

#[test]
fn a_script_that_cannot_start_is_reported() {
    for mode in ["Immediate", "Thread", "Pty"] {
        // A NUL byte cannot be passed to a process.
        let yaml = format!(
            r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: broken
          position: {{x1: "0", y1: "0", x2: "39", y2: "9"}}
          execution_mode: {}
          script: ["echo \0 x"]
"#,
            mode
        );
        let mut runtime = runtime(&yaml, 40, 10);
        pump_until(&mut runtime, "the failure", |r| {
            r.notice.as_ref().is_some_and(|n| n.is_error)
                || r.ctx
                    .app
                    .get_muxbox_by_id("broken")
                    .unwrap()
                    .in_error_state()
        });
        let shown = runtime.display.rows().join("\n");
        let notice = runtime
            .notice
            .as_ref()
            .map(|n| n.text.clone())
            .unwrap_or_default();
        assert!(
            runtime
                .ctx
                .app
                .get_muxbox_by_id("broken")
                .unwrap()
                .in_error_state()
                || notice.contains("broken"),
            "{}: {}\n{}",
            mode,
            notice,
            shown
        );
    }
}

/// Pastes "hello" into a terminal program that turns bracketed paste on or off first, and
/// returns what the program read.
#[cfg(unix)]
fn paste_into_terminal(mode: char) -> String {
    let yaml = format!(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: term
          position: {{x1: "0", y1: "0", x2: "59", y2: "9"}}
          tab_order: 1
          execution_mode: Pty
          script:
            - printf '\033[?2004{}ready\n'
            - read -r line
            - printf 'got[%s]\n' "$line" | cat -v
            - sleep 5
"#,
        mode
    );
    let mut runtime = runtime(&yaml, 60, 10);
    pump_until(&mut runtime, "the program", |r| screen_contains(r, "ready"));
    runtime.handle(Event::Input(TermEvent::Paste("hello\r".to_string())));
    pump_until(&mut runtime, "the paste", |r| screen_contains(r, "got["));
    let rows = runtime.display.rows();
    rows.into_iter()
        .find(|row| row.contains("got["))
        .unwrap()
        .trim()
        .trim_matches('│')
        .trim()
        .to_string()
}

#[cfg(unix)]
#[test]
fn a_paste_is_bracketed_only_for_programs_that_asked() {
    let bracketed = paste_into_terminal('h');
    assert!(bracketed.contains("got[^[[200~hello]"), "{}", bracketed);
    let plain = paste_into_terminal('l');
    assert!(plain.contains("got[hello]"), "{}", plain);
}

use crate::socket::protocol::SocketFunction;

/// A file in the temporary directory that scripts append to, removed when dropped.
#[cfg(unix)]
struct RunLog(std::path::PathBuf);

#[cfg(unix)]
impl RunLog {
    fn new(name: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("boxmux-runlog-{}-{}", name, std::process::id()));
        let _ = std::fs::remove_file(&path);
        RunLog(path)
    }

    fn runs(&self) -> usize {
        std::fs::read_to_string(&self.0)
            .map(|text| text.lines().count())
            .unwrap_or(0)
    }
}

#[cfg(unix)]
impl Drop for RunLog {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[cfg(unix)]
#[test]
fn boxes_of_a_hidden_layout_do_not_refresh() {
    let log = RunLog::new("hidden");
    let yaml = format!(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: a
          position: {{x1: "0", y1: "0", x2: "39", y2: "9"}}
          content: main
    - id: other
      children:
        - id: b
          position: {{x1: "0", y1: "0", x2: "39", y2: "9"}}
          refresh_interval: 50
          script: ["echo x >> '{}'"]
"#,
        log.0.display()
    );
    let mut runtime = runtime(&yaml, 40, 10);
    // Nothing of the hidden layout runs at start.
    let start = Instant::now();
    pump_until(&mut runtime, "a moment", |_| {
        start.elapsed() > Duration::from_millis(300)
    });
    assert_eq!(log.runs(), 0);
    // Shown, it refreshes.
    assert!(
        runtime
            .handle_socket(SocketFunction::SwitchActiveLayout {
                layout_id: "other".into()
            })
            .ok
    );
    pump_until(&mut runtime, "refreshes", |_| log.runs() >= 2);
    // Hidden again, it stops.
    assert!(
        runtime
            .handle_socket(SocketFunction::SwitchActiveLayout {
                layout_id: "main".into()
            })
            .ok
    );
    pump_until(&mut runtime, "the last run", |r| {
        r.executor
            .run_for_origin(&crate::model::stream::RunOrigin::BoxScript { box_id: "b".into() })
            .is_none()
    });
    let hidden = log.runs();
    let start = Instant::now();
    pump_until(&mut runtime, "a moment", |_| {
        start.elapsed() > Duration::from_millis(300)
    });
    assert_eq!(log.runs(), hidden);
}

#[test]
fn replacing_a_script_keeps_the_other_tabs_and_starts_refreshing() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      refresh_interval: 100
      children:
        - id: a
          position: {x1: "0", y1: "0", x2: "59", y2: "9"}
          content: static
"#;
    let mut runtime = runtime(yaml, 60, 10);
    let socket = runtime.handle_socket(SocketFunction::ReplaceBoxContent {
        box_id: "a".into(),
        content: "from socket".into(),
        success: true,
    });
    assert!(socket.ok);
    let reply = runtime.handle_socket(SocketFunction::ReplaceBoxScript {
        box_id: "a".into(),
        script: vec!["echo replaced".into()],
    });
    assert!(reply.ok, "{:?}", reply.error);
    pump_until(&mut runtime, "the script output", |r| {
        screen_contains(r, "replaced")
    });
    let labels: Vec<String> = runtime
        .ctx
        .app
        .get_muxbox_by_id("a")
        .unwrap()
        .streams
        .values()
        .map(|s| s.label.clone())
        .collect();
    assert_eq!(labels, vec!["Content", "Socket", "Output"]);
    // The layout's refresh_interval applies to the new script.
    let refresh = runtime.refresh.get("a").expect("the script is scheduled");
    assert_eq!(refresh.interval, Some(Duration::from_millis(100)));
}

#[test]
fn refresh_control_names_what_is_wrong() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: once
          position: {x1: "0", y1: "0", x2: "39", y2: "4"}
          script: ["echo once"]
        - id: plain
          position: {x1: "0", y1: "5", x2: "39", y2: "9"}
          content: text
    - id: later
      children:
        - id: periodic
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          refresh_interval: 1000
          script: ["echo periodic"]
"#;
    let mut runtime = runtime(yaml, 40, 10);
    let error = |reply: crate::socket::protocol::Reply| reply.error.unwrap_or_default();
    let stop = |id: &str| SocketFunction::StopBoxRefresh { box_id: id.into() };
    assert!(error(runtime.handle_socket(stop("missing"))).contains("there is no box 'missing'"));
    assert!(error(runtime.handle_socket(stop("plain"))).contains("has no script"));
    assert!(error(runtime.handle_socket(stop("once"))).contains("does not refresh"));
    // A box of a layout that was never shown can be paused before it is shown.
    assert!(runtime.handle_socket(stop("periodic")).ok);
    assert!(runtime.refresh["periodic"].paused);
    assert!(
        runtime
            .handle_socket(SocketFunction::StartBoxRefresh {
                box_id: "periodic".into()
            })
            .ok
    );
    assert!(!runtime.refresh["periodic"].paused);
}

#[test]
fn process_queries_refuse_tabs_without_a_process_and_invalid_signals() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: a
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          content: text
"#;
    let mut runtime = runtime(yaml, 40, 10);
    let reply = runtime.handle_socket(SocketFunction::QueryPtyStatus { box_id: "a".into() });
    assert!(!reply.ok);
    assert!(reply.error.unwrap().contains("does not show a process"));
    for signal in [0, -9] {
        assert!(runtime
            .executor
            .signal(crate::model::stream::RunId(1), signal)
            .unwrap_err()
            .contains("not a signal number"));
    }
}

#[test]
fn a_box_added_over_the_socket_must_use_a_loaded_plugin() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: a
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          content: text
"#;
    let mut runtime = runtime(yaml, 40, 10);
    let muxbox = r#"{"id": "g", "position": {"x1": "0", "y1": "0", "x2": "9", "y2": "9"}, "plugin_component": "gauge"}"#;
    let reply = runtime.handle_socket(SocketFunction::AddBox {
        layout_id: "main".into(),
        parent_id: None,
        muxbox: muxbox.parse().unwrap(),
    });
    assert!(!reply.ok);
    assert!(reply.error.unwrap().contains("plugin component 'gauge'"));
    assert!(runtime.ctx.app.get_muxbox_by_id("g").is_none());
}
