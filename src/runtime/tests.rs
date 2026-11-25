//! The runtime driven end to end without a terminal: configuration in, terminal events in,
//! frames out. Scripts run as real processes.

use super::display::MemoryDisplay;
use super::event::Event;
use super::Runtime;
use crate::model::app::{load_app_from_str, AppContext};
use crate::tests::HANG_BOUND;
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

/// Handles events and ticks until `done` holds, failing as hung after the hang bound.
fn pump_until(
    runtime: &mut Runtime<MemoryDisplay>,
    what: &str,
    done: impl Fn(&Runtime<MemoryDisplay>) -> bool,
) {
    let deadline = Instant::now() + HANG_BOUND;
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

/// Prints `started`, then waits on two child processes. Scripts and terminal programs that a
/// test needs running wait for 600 seconds, longer than any wait of a test ([`HANG_BOUND`]),
/// so only being stopped ends them while the test looks; the executor stops them when the
/// test ends.
#[cfg(unix)]
const LONG_RUNNING: &str = "['echo started', 'sleep 600 & sleep 600']";
#[cfg(windows)]
const LONG_RUNNING: &str = "['echo started', 'Start-Sleep 600']";

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
const TERMINAL_UP: &str = "['echo term up', 'sleep 600']";
#[cfg(windows)]
const TERMINAL_UP: &str = "['echo \"term up\"', 'Start-Sleep 600']";

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

fn focused(runtime: &Runtime<MemoryDisplay>) -> Vec<String> {
    runtime
        .ctx
        .app
        .get_active_layout()
        .unwrap()
        .get_selected_muxboxes()
        .iter()
        .map(|b| b.id.clone())
        .collect()
}

#[test]
fn the_escape_chord_moves_the_focus_from_a_terminal_to_the_next_box_and_tab_comes_back_to_stay() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: term
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          tab_order: 1
          execution_mode: Pty
          script: [LONG_TERMINAL]
        - id: notes
          position: {x1: "40", y1: "0", x2: "79", y2: "9"}
          tab_order: 2
          content: notes
"#
    .replace("LONG_TERMINAL", LONG_TERMINAL);
    let mut runtime = runtime(&yaml, 80, 10);
    pump_until(&mut runtime, "the terminal to start", |r| {
        r.executor.runs().any(|(_, run)| run.pty().is_some())
    });
    assert_eq!(focused(&runtime), vec!["term"]);
    // Ctrl+], as the terminal input reader reports the GS character a terminal sends for it.
    runtime.handle(key(KeyCode::Char('5'), KeyModifiers::CONTROL));
    assert_eq!(focused(&runtime), vec!["notes"]);
    assert!(
        runtime
            .executor
            .runs()
            .any(|(_, run)| run.pty().is_some() && !run.stop_requested),
        "the terminal's program runs on"
    );
    runtime.handle(key(KeyCode::Tab, KeyModifiers::NONE));
    assert_eq!(focused(&runtime), vec!["term"]);
    // In the terminal, Tab and Shift+Tab are the program's.
    runtime.handle(key(KeyCode::Tab, KeyModifiers::NONE));
    runtime.handle(key(KeyCode::BackTab, KeyModifiers::SHIFT));
    assert_eq!(focused(&runtime), vec!["term"]);
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
    runtime.display.set_size(100, 20);
    runtime.handle(Event::Input(TermEvent::Resize(100, 20)));
    runtime.render();
    assert_eq!(runtime.display.rows().len(), 20);
    assert_eq!(find(&runtime, "here"), (73, 9));
}

/// The handlebar's arrangement: a box at the top, `needs` rows counted from the top, then one in
/// every row below it to the screen's bottom.
const HANDLEBAR: &str = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: needs
          title: Needs you
          position: {x1: "0%", y1: "0", x2: "100%", y2: "5"}
          content: "⟳ Refresh"
        - id: log
          title: Log
          position: {x1: "0%", y1: "6", x2: "100%", y2: "100%"}
          content: "11:40:04 a worker moved to a fresh worktree"
"#;

/// What is wrong with the handlebar on screen, Needs you six rows tall: row 0 holds Needs you's
/// top border and nothing else, and each box's bottom border is on its own last row, the log's
/// on the screen's last row.
fn handlebar_problems(rows: &[String]) -> Vec<String> {
    let last = rows.len() - 1;
    let edges = |y: usize| (rows[y].chars().next(), rows[y].chars().last());
    let mut problems = Vec::new();
    let mut expect = |y: usize, first: char, end: char, holds: &str| {
        if edges(y) != (Some(first), Some(end)) || !rows[y].contains(holds) {
            problems.push(format!("row {}: {:?}", y, rows[y]));
        }
    };
    expect(0, '┌', '┐', "Needs you");
    (1..5).for_each(|y| expect(y, '│', '│', ""));
    expect(5, '└', '⋱', "");
    expect(6, '┌', '┐', "Log");
    (7..last).for_each(|y| expect(y, '│', '│', ""));
    expect(last, '└', '⋱', "");
    problems
}

#[test]
fn each_frame_is_drawn_for_the_size_the_terminal_has_when_it_is_drawn() {
    // A terminal can change size without the runtime being told: one resized before boxmux
    // listens for resizes never says so. Each frame is still laid out for the terminal as it
    // is, every box's bottom border on its own last row.
    let mut runtime = runtime(HANDLEBAR, 55, 30);
    assert_eq!(
        handlebar_problems(&runtime.display.rows()),
        Vec::<String>::new()
    );
    for (cols, rows) in [(55, 26), (55, 20), (55, 33), (60, 26), (48, 9), (55, 26)] {
        runtime.display.set_size(cols, rows);
        runtime.render();
        let shown = runtime.display.rows();
        assert_eq!(shown.len(), usize::from(rows));
        assert_eq!(
            handlebar_problems(&shown),
            Vec::<String>::new(),
            "at {}x{}:\n{}",
            cols,
            rows,
            shown.join("\n")
        );
    }
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
          script: ['echo slow-started', 'sleep 600']
        - id: waiting
          position: {x1: "0", y1: "5", x2: "39", y2: "9"}
          execution_mode: Immediate
          script: ['sleep 600']
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
        let deadline = Instant::now() + HANG_BOUND;
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
const COPY_HI: &str = r#"["printf '\\033]52;c;aGk=\\007copied'", "sleep 600"]"#;
#[cfg(windows)]
const COPY_HI: &str =
    r#"["Write-Host -NoNewline \"$([char]27)]52;c;aGk=$([char]7)copied\"", "Start-Sleep 600"]"#;

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
            - sleep 600
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

use crate::socket::protocol::{ContentTarget, SocketFunction};

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
        target: ContentTarget::Socket,
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

/// A choice that prints, takes a moment — long enough for a program to replace its box meanwhile —, then prints again.
#[cfg(unix)]
const SLOW_CHOICE: &str = r#"["echo 'started'", "sleep 1", "echo 'done after the replace'"]"#;
#[cfg(windows)]
const SLOW_CHOICE: &str =
    r#"["Write-Output 'started'", "Start-Sleep 1", "Write-Output 'done after the replace'"]"#;

/// A box with the slow choice, as a program draws it and draws it again with `replace_box`.
fn slow_menu(title: &str) -> String {
    format!(
        r#"{{"id": "menu", "title": "{}", "position": {{"x1": "0", "y1": "0", "x2": "39", "y2": "9"}},
            "tab_order": 1, "choices": [{{"id": "slow", "content": "Take a moment", "execution_mode": "Thread", "script": {}}},
            {{"id": "other", "content": "Another", "script": ["echo other"]}}]}}"#,
        title, SLOW_CHOICE
    )
}

fn slow_waiting(runtime: &Runtime<MemoryDisplay>) -> bool {
    runtime
        .ctx
        .app
        .get_muxbox_by_id("menu")
        .and_then(|b| b.choices_stream())
        .and_then(|s| s.choices.as_ref())
        .and_then(|choices| choices.iter().find(|c| c.id == "slow"))
        .is_some_and(|c| c.waiting)
}

#[test]
fn replacing_a_box_keeps_the_runs_its_choices_started_and_their_output_reaches_the_replacement() {
    // A program redraws a menu — its title, its choices — while a choice the user pressed is under way: the
    // press is the user's act, not the box's definition, so it goes on, its tab and its waiting mark carried into
    // the replacement, and the rest of its output reaches it there.
    let yaml = format!(
        "app:\n  layouts:\n    - id: main\n      root: true\n      children:\n        - {}\n",
        slow_menu("Menu")
    );
    let mut runtime = runtime(&yaml, 40, 10);
    let at = find(&runtime, "Take a moment");
    click(&mut runtime, at);
    pump_until(&mut runtime, "the choice's first output", |r| {
        tab_content(r, "menu", "menu/choice:menu:slow") == vec!["started".to_string()]
    });
    assert!(slow_waiting(&runtime));

    let reply = runtime.handle_socket(SocketFunction::ReplaceBox {
        box_id: "menu".into(),
        new_box: slow_menu("Menu, redrawn").parse().unwrap(),
    });
    assert!(reply.ok, "{:?}", reply.error);
    assert_eq!(
        runtime
            .ctx
            .app
            .get_muxbox_by_id("menu")
            .unwrap()
            .title
            .as_deref(),
        Some("Menu, redrawn")
    );
    assert!(
        slow_waiting(&runtime),
        "the replacement shows the choice still running"
    );
    assert_eq!(
        tab_content(&runtime, "menu", "menu/choice:menu:slow"),
        vec!["started".to_string()],
        "the choice's tab is carried into the replacement"
    );

    pump_until(
        &mut runtime,
        "the choice to finish into the replacement",
        |r| !slow_waiting(r),
    );
    assert_eq!(
        tab_content(&runtime, "menu", "menu/choice:menu:slow"),
        vec!["started".to_string(), "done after the replace".to_string()]
    );
    let stream = runtime
        .ctx
        .app
        .get_muxbox_by_id("menu")
        .unwrap()
        .streams
        .get("menu/choice:menu:slow")
        .unwrap();
    assert_eq!(stream.status, crate::model::stream::StreamStatus::Exited(0));
}

#[test]
fn replacing_a_box_stops_its_own_script_the_terminal_shown_in_it_and_what_runs_in_a_child_it_drops()
{
    // What the box's definition runs — its own script, and a terminal shown in it — ends with the definition; and a
    // child the replacement no longer has is gone, with everything running for it.
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: sleeper
          position: {x1: "0", y1: "0", x2: "39", y2: "4"}
          execution_mode: Thread
          script: LONG_RUNNING
        - id: term
          position: {x1: "0", y1: "5", x2: "39", y2: "9"}
          content: idle
        - id: outer
          position: {x1: "40", y1: "0", x2: "79", y2: "9"}
          children:
            - id: inner
              position: {x1: "0", y1: "0", x2: "100%", y2: "100%"}
              execution_mode: Thread
              script: LONG_RUNNING
"#
    .replace("LONG_RUNNING", LONG_RUNNING);
    let mut runtime = runtime(&yaml, 80, 10);
    pump_until(&mut runtime, "both scripts to start", |r| {
        tab_content(r, "sleeper", "sleeper/script") == vec!["started".to_string()]
            && tab_content(r, "inner", "inner/script") == vec!["started".to_string()]
    });
    let reply = runtime.handle_socket(SocketFunction::SpawnPtyProcess {
        box_id: "term".into(),
        script: vec![LONG_TERMINAL.into()],
        redirect_output: None,
    });
    assert!(reply.ok, "{:?}", reply.error);
    let pids: Vec<u32> = runtime.executor.runs().map(|(_, run)| run.pid()).collect();
    assert_eq!(pids.len(), 3, "the two boxes' scripts and the terminal run");

    for (id, rest) in [
        (
            "sleeper",
            r#""position": {"x1": "0", "y1": "0", "x2": "39", "y2": "4"}, "content": "replaced""#,
        ),
        (
            "term",
            r#""position": {"x1": "0", "y1": "5", "x2": "39", "y2": "9"}, "content": "replaced""#,
        ),
        (
            "outer",
            r#""position": {"x1": "40", "y1": "0", "x2": "79", "y2": "9"}, "content": "no child now""#,
        ),
    ] {
        let reply = runtime.handle_socket(SocketFunction::ReplaceBox {
            box_id: id.into(),
            new_box: format!(r#"{{"id": "{}", {}}}"#, id, rest).parse().unwrap(),
        });
        assert!(reply.ok, "{:?}", reply.error);
    }
    let not_stopped: Vec<_> = runtime
        .executor
        .runs()
        .filter(|(_, run)| !run.stop_requested)
        .map(|(_, run)| run.origin.clone())
        .collect();
    assert!(
        not_stopped.is_empty(),
        "runs not stopped by replacing their boxes: {:?}",
        not_stopped
    );
    // Stopped, they end well within their sleep.
    let deadline = Instant::now() + HANG_BOUND;
    while runtime.executor.runs().count() > 0 {
        assert!(
            Instant::now() < deadline,
            "runs still under way {:?} after their boxes were replaced",
            HANG_BOUND
        );
        while let Ok(event) = runtime.events.recv_timeout(Duration::from_millis(5)) {
            runtime.handle(event);
        }
    }
    #[cfg(unix)]
    for pid in pids {
        // Nothing is left in either run's process group.
        let deadline = Instant::now() + HANG_BOUND;
        while unsafe { libc::kill(-(pid as i32), 0) } == 0 {
            assert!(
                Instant::now() < deadline,
                "process group {} survived its box's replacement",
                pid
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

/// A terminal program that runs until it is stopped.
#[cfg(unix)]
const LONG_TERMINAL: &str = "sleep 600";
#[cfg(windows)]
const LONG_TERMINAL: &str = "Start-Sleep 600";

#[test]
fn a_terminal_for_a_box_in_no_layout_is_refused_naming_the_box() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: shown
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          content: here
    - id: other
      children:
        - id: hidden
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          content: there
"#;
    let mut runtime = runtime(yaml, 40, 10);
    for (box_id, redirect_output) in [("nowhere", None), ("hidden", Some("nowhere".to_string()))] {
        let reply = runtime.handle_socket(SocketFunction::SpawnPtyProcess {
            box_id: box_id.into(),
            script: vec![LONG_TERMINAL.into()],
            redirect_output,
        });
        assert!(!reply.ok, "a terminal for box 'nowhere' was started");
        assert_eq!(reply.error.as_deref(), Some("there is no box 'nowhere'"));
    }
    assert_eq!(runtime.executor.runs().count(), 0);
}

#[test]
fn a_restarted_terminal_takes_its_box_s_size_as_it_is_then() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: term
          position: {x1: "0%", y1: "0%", x2: "50%", y2: "100%"}
          content: idle
"#;
    let mut runtime = runtime(yaml, 40, 10);
    let origin = crate::model::stream::RunOrigin::SocketSpawn {
        box_id: "term".into(),
    };
    let size_of = |runtime: &Runtime<MemoryDisplay>, run| {
        runtime.executor.get(run).unwrap().pty().unwrap().size()
    };
    let reply = runtime.handle_socket(SocketFunction::SpawnPtyProcess {
        box_id: "term".into(),
        script: vec![LONG_TERMINAL.into()],
        redirect_output: None,
    });
    assert!(reply.ok, "{:?}", reply.error);
    let first = runtime.executor.run_for_origin(&origin).unwrap();
    let started = size_of(&runtime, first);

    runtime.display.set_size(100, 30);
    runtime.handle(Event::Input(TermEvent::Resize(100, 30)));
    runtime.render();
    let resized = size_of(&runtime, first);
    assert_ne!(resized, started, "the terminal follows its box");

    let reply = runtime.handle_socket(SocketFunction::RestartPtyProcess {
        box_id: "term".into(),
    });
    assert!(reply.ok, "{:?}", reply.error);
    pump_until(&mut runtime, "the terminal to start again", |r| {
        r.executor
            .run_for_origin(&origin)
            .is_some_and(|run| run != first)
    });
    let second = runtime.executor.run_for_origin(&origin).unwrap();
    assert_eq!(size_of(&runtime, second), resized);
}

/// Prints the answer the choice's confirmation was given.
#[cfg(unix)]
const PRINT_OPTION: &str = r#"['echo "picked $BOXMUX_OPTION"']"#;
#[cfg(windows)]
const PRINT_OPTION: &str = r#"['echo "picked $env:BOXMUX_OPTION"']"#;

/// A menu whose choices ask before they run: `restart` with the default answers, `deploy` with
/// its own, of which `abort` cancels.
fn confirm_menu() -> String {
    r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: menu
          title: Menu
          position: {x1: "0", y1: "0", x2: "39", y2: "5"}
          tab_order: 1
          choices:
            - id: restart
              content: Restart the service
              script: PRINT_OPTION
              confirm:
                text: Restart the service now? Everyone using it is disconnected.
            - id: deploy
              content: Deploy
              script: PRINT_OPTION
              confirm:
                text: Deploy where?
                options:
                  - {id: staging, content: Staging}
                  - {id: prod, content: Production}
                  - {id: abort, content: Not now}
                cancel: abort
    - id: other
      children:
        - id: elsewhere
          position: {x1: "0", y1: "0", x2: "79", y2: "19"}
          content: Another layout
"#
    .replace("PRINT_OPTION", PRINT_OPTION)
}

/// The output of the choice `choice_id` of the menu, once its run printed.
fn picked(runtime: &mut Runtime<MemoryDisplay>, choice_id: &str) -> Vec<String> {
    let stream = format!("menu/choice:menu:{choice_id}");
    pump_until(runtime, "the choice's output", |r| {
        !tab_content(r, "menu", &stream).is_empty() && r.executor.runs().next().is_none()
    });
    tab_content(runtime, "menu", &stream)
}

/// Nothing ran, nothing is running and no tab opened: the menu has only its choices.
fn assert_nothing_ran(runtime: &Runtime<MemoryDisplay>, what: &str) {
    assert!(
        runtime.executor.runs().next().is_none(),
        "{what}: a run was started"
    );
    assert_eq!(
        runtime
            .ctx
            .app
            .get_muxbox_by_id("menu")
            .unwrap()
            .get_tab_stream_ids(),
        vec!["menu/choices".to_string()],
        "{what}: a tab opened"
    );
}

fn press(runtime: &mut Runtime<MemoryDisplay>, code: KeyCode) {
    runtime.handle(key(code, KeyModifiers::NONE));
    runtime.render();
}

#[test]
fn a_choice_with_a_confirmation_asks_first_and_runs_on_yes_with_the_answer_s_id() {
    let mut runtime = runtime(&confirm_menu(), 80, 20);
    // Enter activates the first choice, which is selected.
    press(&mut runtime, KeyCode::Enter);
    assert!(
        screen_contains(&runtime, "Restart the service now? Everyone using it is"),
        "the window shows the question"
    );
    assert!(
        screen_contains(&runtime, " Yes    Cancel "),
        "and the default answers"
    );
    assert_nothing_ran(&runtime, "asked");
    // Yes is marked; Enter presses it.
    press(&mut runtime, KeyCode::Enter);
    assert!(
        !screen_contains(&runtime, " Yes    Cancel "),
        "the window closed"
    );
    assert_eq!(
        picked(&mut runtime, "restart"),
        vec!["picked yes".to_string()]
    );
}

#[test]
fn clicking_one_of_a_choice_s_own_answers_runs_it_with_that_answer_s_id() {
    let mut runtime = runtime(&confirm_menu(), 80, 20);
    let at = find(&runtime, "Deploy");
    click(&mut runtime, at);
    assert!(screen_contains(&runtime, "Deploy where?"));
    let at = find(&runtime, "Production");
    click(&mut runtime, at);
    assert_eq!(
        picked(&mut runtime, "deploy"),
        vec!["picked prod".to_string()]
    );
}

#[test]
fn the_arrow_keys_move_between_the_answers_and_enter_presses_the_one_marked() {
    let mut runtime = runtime(&confirm_menu(), 80, 20);
    let at = find(&runtime, "Deploy");
    click(&mut runtime, at);
    // Staging is marked: Right, Right reach Not now, and Left goes back to Production.
    press(&mut runtime, KeyCode::Right);
    press(&mut runtime, KeyCode::Right);
    press(&mut runtime, KeyCode::Left);
    press(&mut runtime, KeyCode::Enter);
    assert_eq!(
        picked(&mut runtime, "deploy"),
        vec!["picked prod".to_string()]
    );
}

/// Something done to a running runtime.
type Act = dyn Fn(&mut Runtime<MemoryDisplay>);

#[test]
fn the_answer_that_cancels_escape_and_a_click_outside_close_the_window_and_run_nothing() {
    let cancel_with = |cancel: &Act| {
        let mut runtime = runtime(&confirm_menu(), 80, 20);
        let at = find(&runtime, "Deploy");
        click(&mut runtime, at);
        assert!(screen_contains(&runtime, "Deploy where?"));
        cancel(&mut runtime);
        runtime
    };
    let cases: [(&str, &Act); 4] = [
        ("the cancel answer", &|r| {
            let at = find(r, "Not now");
            click(r, at);
        }),
        ("Escape", &|r| press(r, KeyCode::Esc)),
        ("a click outside", &|r| click(r, (1, 18))),
        ("the cancel answer by keyboard", &|r| {
            press(r, KeyCode::Left);
            press(r, KeyCode::Enter);
        }),
    ];
    for (what, cancel) in cases {
        let mut runtime = cancel_with(cancel);
        assert!(
            !screen_contains(&runtime, "Deploy where?"),
            "{what}: the window closed"
        );
        assert_nothing_ran(&runtime, what);
        // Nothing starts later either.
        let start = Instant::now();
        pump_until(&mut runtime, "a moment", |_| {
            start.elapsed() > Duration::from_millis(300)
        });
        assert_nothing_ran(&runtime, what);
    }

    // The default Cancel cancels too.
    let mut runtime = runtime(&confirm_menu(), 80, 20);
    press(&mut runtime, KeyCode::Enter);
    let at = find(&runtime, "Cancel");
    click(&mut runtime, at);
    assert!(!screen_contains(&runtime, " Yes    Cancel "));
    assert_nothing_ran(&runtime, "Cancel");
}

#[test]
fn an_open_confirmation_takes_every_key_and_click() {
    let mut runtime = runtime(&confirm_menu(), 80, 20);
    let at = find(&runtime, "Deploy");
    click(&mut runtime, at);
    // q would quit, Tab would move the focus, and a click on the menu would run a choice.
    press(&mut runtime, KeyCode::Char('q'));
    assert!(!runtime.quit, "q went to the layout");
    let restart = find(&runtime, "Restart the service");
    click(&mut runtime, (restart.0, restart.1));
    // The click was outside the window, so it cancelled, and started nothing.
    assert!(!screen_contains(&runtime, "Deploy where?"));
    assert_nothing_ran(&runtime, "a click on the menu under the window");
}

#[test]
fn a_confirmation_whose_choice_leaves_the_shown_layout_closes_and_says_so() {
    let mut runtime = runtime(&confirm_menu(), 80, 20);
    press(&mut runtime, KeyCode::Enter);
    assert!(screen_contains(&runtime, " Yes    Cancel "));
    let (reply, answer) = crossbeam_channel::unbounded();
    runtime.handle(Event::Socket {
        function: SocketFunction::SwitchActiveLayout {
            layout_id: "other".into(),
        },
        reply,
    });
    assert!(answer.recv().unwrap().ok);
    runtime.render();
    assert!(
        !screen_contains(&runtime, " Yes    Cancel "),
        "the window closed"
    );
    let notice = runtime
        .notice
        .as_ref()
        .map(|n| n.text.clone())
        .unwrap_or_default();
    assert!(
        notice.contains("confirmation of choice 'restart' closed"),
        "{notice}"
    );
    // Enter in the other layout presses nothing.
    press(&mut runtime, KeyCode::Enter);
    assert!(runtime.executor.runs().next().is_none());
}

fn tab_labels(runtime: &Runtime<MemoryDisplay>, box_id: &str) -> Vec<String> {
    runtime
        .ctx
        .app
        .get_muxbox_by_id(box_id)
        .unwrap()
        .get_tab_labels()
}

#[test]
fn replace_box_content_targeting_content_replaces_the_box_s_own_tab() {
    let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: status
          title: Status
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          content: starting
        - id: both
          position: {x1: "40", y1: "0", x2: "79", y2: "9"}
          content: static
          script: ["echo scripted"]
        - id: bare
          title: Bare
          position: {x1: "0", y1: "10", x2: "39", y2: "19"}
"#;
    let mut runtime = runtime(yaml, 80, 20);
    let replace = |box_id: &str, content: &str, success: bool| SocketFunction::ReplaceBoxContent {
        box_id: box_id.into(),
        content: content.into(),
        success,
        target: ContentTarget::Content,
    };

    let reply = runtime.handle_socket(replace("status", "deployed", true));
    assert!(reply.ok, "{:?}", reply.error);
    runtime.render();
    assert!(screen_contains(&runtime, "deployed"));
    assert!(!screen_contains(&runtime, "starting"));
    assert_eq!(
        tab_labels(&runtime, "status"),
        vec!["Status"],
        "still one tab"
    );
    let status = runtime.ctx.app.get_muxbox_by_id("status").unwrap();
    assert_eq!(status.content.as_deref(), Some("deployed"));
    assert!(!status.in_error_state());

    // A failure shows the box's own content in its error colors.
    let reply = runtime.handle_socket(replace("status", "deploy failed", false));
    assert!(reply.ok, "{:?}", reply.error);
    assert_eq!(tab_labels(&runtime, "status"), vec!["Status"]);
    assert!(runtime
        .ctx
        .app
        .get_muxbox_by_id("status")
        .unwrap()
        .in_error_state());

    // The content tab of a box with a script is replaced and shown; no tab is added.
    pump_until(&mut runtime, "the script's output", |r| {
        r.ctx.app.get_muxbox_by_id("both").unwrap().streams["both/script"].content
            == vec!["scripted"]
    });
    runtime
        .ctx
        .app
        .get_muxbox_by_id_mut("both")
        .unwrap()
        .switch_to_stream("both/script");
    let reply = runtime.handle_socket(replace("both", "new static", true));
    assert!(reply.ok, "{:?}", reply.error);
    runtime.render();
    assert_eq!(tab_labels(&runtime, "both"), vec!["Content", "Output"]);
    assert!(screen_contains(&runtime, "new static"));

    // A box with no content tab gets one, first, and it is the box's only tab.
    let reply = runtime.handle_socket(replace("bare", "now with text", true));
    assert!(reply.ok, "{:?}", reply.error);
    runtime.render();
    assert_eq!(tab_labels(&runtime, "bare"), vec!["Bare"]);
    assert!(screen_contains(&runtime, "now with text"));

    // The socket target is still a tab of its own.
    let reply = runtime.handle_socket(SocketFunction::ReplaceBoxContent {
        box_id: "status".into(),
        content: "pushed".into(),
        success: true,
        target: ContentTarget::Socket,
    });
    assert!(reply.ok, "{:?}", reply.error);
    assert_eq!(tab_labels(&runtime, "status"), vec!["Status", "Socket"]);
}

/// A temporary folder for one file box test, with the configuration of a box `log` showing
/// `file` (relative to the configuration file) with `extra` fields, in a 60x12 screen.
fn file_box_runtime(dir: &std::path::Path, file: &str, extra: &str) -> Runtime<MemoryDisplay> {
    let yaml = format!(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: log
          title: Log
          position: {{x1: "0", y1: "0", x2: "59", y2: "11"}}
          file: {file}
{extra}
"#,
        file = serde_json::to_string(file).unwrap(),
        extra = extra
    );
    let config = dir.join("app.yaml");
    std::fs::write(&config, &yaml).unwrap();
    let app = load_app_from_str(&yaml, config.to_str().unwrap()).expect("the configuration loads");
    let mut ctx = AppContext::new(app, Config::new(1, false, false).unwrap());
    ctx.yaml_file_path = Some(config);
    let (sender, events) = crossbeam_channel::unbounded();
    let mut runtime = Runtime::new(ctx, MemoryDisplay::new(60, 12), sender, events, None);
    runtime.activate_layout_runs();
    runtime.render();
    runtime
}

/// Waits for the next events, handles them and draws the frame they make, until `done` holds.
/// Only events wake it, never a timer, so what it waits for must come from the platform's file
/// watching; it fails when the hang bound passes first.
fn pump_events_until(
    runtime: &mut Runtime<MemoryDisplay>,
    what: &str,
    done: impl Fn(&Runtime<MemoryDisplay>) -> bool,
) {
    let deadline = Instant::now() + HANG_BOUND;
    while !done(runtime) {
        let event = runtime.events.recv_deadline(deadline).unwrap_or_else(|_| {
            panic!(
                "nothing was reported for {:?} while waiting for {}; screen:\n{}",
                HANG_BOUND,
                what,
                runtime.display.rows().join("\n")
            )
        });
        runtime.handle(event);
        while let Ok(event) = runtime.events.try_recv() {
            runtime.handle(event);
        }
        // What they changed is drawn in the next frame, as the event loop draws it: at the
        // first moment it wakes for, which is no later than the frame delay after the last.
        runtime.tick(Instant::now());
        let frame_by = Instant::now() + Duration::from_millis(runtime.ctx.config.frame_delay);
        while runtime.dirty {
            let wake = runtime.next_wakeup();
            assert!(wake <= frame_by, "the change waits for more than one frame");
            std::thread::sleep(wake.saturating_duration_since(Instant::now()));
            runtime.tick(Instant::now());
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for {}; screen:\n{}",
            what,
            runtime.display.rows().join("\n")
        );
    }
}

#[test]
fn a_file_box_redraws_in_the_frame_after_its_file_changes_with_no_refresh() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("status.txt"), "first version\n").unwrap();
    let mut runtime = file_box_runtime(dir.path(), "status.txt", "");
    assert!(screen_contains(&runtime, "first version"));
    assert_eq!(tab_labels(&runtime, "log"), vec!["Log"]);
    assert!(runtime.refresh.is_empty(), "nothing is scheduled to poll");

    std::fs::write(dir.path().join("status.txt"), "second version\n").unwrap();
    pump_events_until(&mut runtime, "the second version", |r| {
        screen_contains(r, "second version")
    });
    assert!(!screen_contains(&runtime, "first version"));

    // Written to a new file and renamed over the old one, as editors save.
    std::fs::write(dir.path().join("status.txt.new"), "third version\n").unwrap();
    std::fs::rename(
        dir.path().join("status.txt.new"),
        dir.path().join("status.txt"),
    )
    .unwrap();
    pump_events_until(&mut runtime, "the replacing file", |r| {
        screen_contains(r, "third version")
    });
    // The new file is watched, not the one it replaced.
    std::fs::write(dir.path().join("status.txt"), "fourth version\n").unwrap();
    pump_events_until(&mut runtime, "a write to the replacing file", |r| {
        screen_contains(r, "fourth version")
    });
}

#[test]
fn a_missing_file_shows_a_placeholder_until_it_appears_in_folders_made_later() {
    let dir = tempfile::tempdir().unwrap();
    let mut runtime = file_box_runtime(dir.path(), "out/deploy/status.txt", "");
    assert!(
        screen_contains(&runtime, "Waiting for this file to appear:"),
        "{}",
        runtime.display.rows().join("\n")
    );
    assert!(screen_contains(&runtime, "out/deploy/status.txt"));
    assert!(!runtime
        .ctx
        .app
        .get_muxbox_by_id("log")
        .unwrap()
        .in_error_state());

    std::fs::create_dir(dir.path().join("out")).unwrap();
    std::fs::create_dir(dir.path().join("out/deploy")).unwrap();
    std::fs::write(dir.path().join("out/deploy/status.txt"), "deployed\n").unwrap();
    pump_events_until(&mut runtime, "the file made in new folders", |r| {
        screen_contains(r, "deployed")
    });

    std::fs::remove_dir_all(dir.path().join("out")).unwrap();
    pump_events_until(&mut runtime, "the placeholder again", |r| {
        screen_contains(r, "Waiting for this file to appear:")
    });
    std::fs::create_dir_all(dir.path().join("out/deploy")).unwrap();
    std::fs::write(dir.path().join("out/deploy/status.txt"), "redeployed\n").unwrap();
    pump_events_until(&mut runtime, "the file made again", |r| {
        screen_contains(r, "redeployed")
    });
}

#[test]
fn a_file_box_whose_path_is_a_folder_says_so_in_its_error_colors() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join("logs")).unwrap();
    let runtime = file_box_runtime(dir.path(), "logs", "");
    assert!(screen_contains(&runtime, "This is a folder, not a file"));
    assert!(runtime
        .ctx
        .app
        .get_muxbox_by_id("log")
        .unwrap()
        .in_error_state());
}

fn numbered(from: usize, to: usize) -> String {
    (from..=to).map(|n| format!("line {}\n", n)).collect()
}

fn append(path: &std::path::Path, text: &str) {
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().append(true).open(path).unwrap();
    file.write_all(text.as_bytes()).unwrap();
}

#[test]
fn following_a_growing_log_keeps_to_its_bottom_until_the_reader_scrolls_up() {
    let dir = tempfile::tempdir().unwrap();
    let log = dir.path().join("build.log");
    std::fs::write(&log, numbered(1, 30)).unwrap();
    let mut runtime = file_box_runtime(dir.path(), "build.log", "          follow: true");
    assert!(screen_contains(&runtime, "line 30"), "it opens at the end");
    assert!(!screen_contains(&runtime, "line 1 "));

    append(&log, &numbered(31, 40));
    pump_events_until(&mut runtime, "the appended lines", |r| {
        screen_contains(r, "line 40")
    });
    assert!(!screen_contains(&runtime, "line 25"), "kept to the bottom");

    // A line written in two parts shows as one line, once.
    append(&log, "half of line");
    pump_events_until(&mut runtime, "the line's first half", |r| {
        screen_contains(r, "half of line")
    });
    append(&log, " 41\n");
    pump_events_until(&mut runtime, "the whole line", |r| {
        screen_contains(r, "half of line 41")
    });
    let lines = &runtime.ctx.app.get_muxbox_by_id("log").unwrap().streams["log/file"].content;
    assert_eq!(lines.len(), 41, "{:?}", lines);
    assert_eq!(lines[40], "half of line 41");

    // Scrolled up to read, the reader stays where they are.
    runtime
        .ctx
        .app
        .get_muxbox_by_id_mut("log")
        .unwrap()
        .vertical_scroll = Some(0.0);
    append(&log, &numbered(42, 42));
    pump_events_until(&mut runtime, "line 42 read", |r| {
        r.ctx.app.get_muxbox_by_id("log").unwrap().streams["log/file"]
            .content
            .len()
            == 42
    });
    assert!(screen_contains(&runtime, "line 1 "));
    assert!(!screen_contains(&runtime, "line 42"));

    // Back at the bottom, it follows again.
    runtime
        .ctx
        .app
        .get_muxbox_by_id_mut("log")
        .unwrap()
        .vertical_scroll = Some(100.0);
    append(&log, &numbered(43, 43));
    pump_events_until(&mut runtime, "line 43", |r| screen_contains(r, "line 43"));

    // A truncated log starts again from its beginning.
    std::fs::write(&log, "rotated\n").unwrap();
    pump_events_until(&mut runtime, "the truncated log", |r| {
        r.ctx.app.get_muxbox_by_id("log").unwrap().streams["log/file"].content == vec!["rotated"]
    });
}

#[test]
fn file_boxes_added_replaced_and_removed_through_the_socket_watch_their_files() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.txt"), "file a\n").unwrap();
    std::fs::write(dir.path().join("b.txt"), "file b\n").unwrap();
    let mut runtime = file_box_runtime(dir.path(), "a.txt", "");
    let file_box = |id: &str, file: &std::path::Path, y: u16| {
        serde_json::json!({
            "id": id,
            "position": {"x1": "0", "y1": y.to_string(), "x2": "59", "y2": (y + 4).to_string()},
            "file": file.to_str().unwrap(),
        })
        .try_into()
        .unwrap()
    };

    let reply = runtime.handle_socket(SocketFunction::ReplaceBox {
        box_id: "log".into(),
        new_box: file_box("log", &dir.path().join("b.txt"), 0),
    });
    assert!(reply.ok, "{:?}", reply.error);
    runtime.render();
    assert!(screen_contains(&runtime, "file b"));
    std::fs::write(dir.path().join("a.txt"), "a changed\n").unwrap();
    std::fs::write(dir.path().join("b.txt"), "b changed\n").unwrap();
    pump_events_until(&mut runtime, "the replacement's file", |r| {
        screen_contains(r, "b changed")
    });

    let reply = runtime.handle_socket(SocketFunction::AddBox {
        layout_id: "main".into(),
        parent_id: None,
        muxbox: file_box("added", &dir.path().join("a.txt"), 6),
    });
    assert!(reply.ok, "{:?}", reply.error);
    runtime.render();
    assert!(screen_contains(&runtime, "a changed"));

    let reply = runtime.handle_socket(SocketFunction::RemoveBox {
        box_id: "added".into(),
    });
    assert!(reply.ok, "{:?}", reply.error);
    let reply = runtime.handle_socket(SocketFunction::RemoveBox {
        box_id: "log".into(),
    });
    assert!(reply.ok, "{:?}", reply.error);
    assert!(
        runtime.files.is_idle(),
        "the removed boxes' watches are ended"
    );
}

use crate::model::appearance::ColorName;
use crate::render::screen::Color;
use crate::render::theme::palette;

/// A queue of items whose choices carry colors of their own, by priority, and one that does not.
const COLORED_MENU: &str = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: queue
          title: Queue
          position: {x1: "0", y1: "0", x2: "39", y2: "9"}
          tab_order: 1
          choices:
            - id: urgent
              content: Urgent item
              fg_color: bright_red
              bg_color: black
              selected_fg_color: black
              selected_bg_color: bright_red
            - id: review
              content: Review item
              fg_color: yellow
              bg_color: blue
              selected_fg_color: blue
              selected_bg_color: yellow
            - id: plain
              content: Plain item
"#;

/// The text and background colors of the cells of `text` on screen and of the two cells of the
/// entry's marker before it, which every cell of them must share.
fn colors_of(runtime: &Runtime<MemoryDisplay>, text: &str) -> (Color, Color) {
    let (x, y) = find(runtime, text);
    let frame = runtime.display.frame.as_ref().expect("a frame was drawn");
    let start = x as usize - 2;
    let cells: Vec<(Color, Color)> = (start..x as usize + text.chars().count())
        .map(|column| {
            let cell = frame
                .get(column, y as usize)
                .expect("the cell is on screen");
            (cell.style.fg, cell.style.bg)
        })
        .collect();
    assert!(
        cells.iter().all(|colors| *colors == cells[0]),
        "every cell of '{}' and its marker has the same colors: {:?}",
        text,
        cells
    );
    cells[0]
}

fn named(fg: ColorName, bg: ColorName) -> (Color, Color) {
    (fg.color(), bg.color())
}

#[test]
fn each_choice_is_drawn_in_its_own_colors_selected_and_not() {
    let mut runtime = runtime(COLORED_MENU, 40, 10);
    let theme = palette();
    let selected = (theme.menu_selected.fg, theme.menu_selected.bg);
    let plain = (theme.fg, theme.bg);

    assert_eq!(
        colors_of(&runtime, "Urgent item"),
        named(ColorName::Black, ColorName::BrightRed),
        "the selected entry in its selected colors"
    );
    assert_eq!(
        colors_of(&runtime, "Review item"),
        named(ColorName::Yellow, ColorName::Blue)
    );
    assert_eq!(
        colors_of(&runtime, "Plain item"),
        plain,
        "an entry without colors of its own in the box's"
    );

    runtime.handle(key(KeyCode::Down, KeyModifiers::NONE));
    runtime.render();
    assert_eq!(
        colors_of(&runtime, "Urgent item"),
        named(ColorName::BrightRed, ColorName::Black)
    );
    assert_eq!(
        colors_of(&runtime, "Review item"),
        named(ColorName::Blue, ColorName::Yellow)
    );

    runtime.handle(key(KeyCode::Down, KeyModifiers::NONE));
    runtime.render();
    assert_eq!(
        colors_of(&runtime, "Review item"),
        named(ColorName::Yellow, ColorName::Blue)
    );
    assert_eq!(
        colors_of(&runtime, "Plain item"),
        selected,
        "the selected entry without colors of its own in the box's selected colors"
    );

    // The entry under the pointer shows the box's hover colors, as every entry does.
    let at = find(&runtime, "Urgent item");
    runtime.handle(mouse(MouseEventKind::Moved, at.0, at.1));
    runtime.render();
    assert_eq!(
        colors_of(&runtime, "Urgent item"),
        (theme.menu_hovered.fg, theme.menu_hovered.bg)
    );
}

#[test]
fn a_choice_without_its_selected_colors_is_drawn_selected_in_the_box_selected_colors() {
    // A choice's selected colors stand for the selection mark, as the box's do: its plain colors
    // never mark it selected.
    let yaml = COLORED_MENU
        .replace("              selected_fg_color: black\n", "")
        .replace("              selected_bg_color: bright_red\n", "");
    let runtime = runtime(&yaml, 40, 10);
    let theme = palette();
    assert_eq!(
        colors_of(&runtime, "Urgent item"),
        (theme.menu_selected.fg, theme.menu_selected.bg)
    );
}

#[test]
fn the_colors_of_a_choice_change_on_a_running_layout_through_the_socket() {
    let mut runtime = runtime(COLORED_MENU, 40, 10);
    // A request as a client writes it on the socket, read as the server reads it.
    let request = serde_json::json!({
        "command": "replace_box",
        "box_id": "queue",
        "new_box": {
            "id": "queue",
            "title": "Queue",
            "position": {"x1": "0", "y1": "0", "x2": "39", "y2": "9"},
            "tab_order": 1,
            "choices": [
                {"id": "urgent", "content": "Urgent item",
                 "fg_color": "magenta", "bg_color": "white",
                 "selected_fg_color": "bright_white", "selected_bg_color": "magenta"},
                {"id": "review", "content": "Review item",
                 "fg_color": "green", "bg_color": "black"},
                {"id": "plain", "content": "Plain item", "fg_color": "cyan"}
            ]
        }
    })
    .to_string();
    let function: SocketFunction = serde_json::from_str(&request).unwrap();
    let reply = runtime.handle_socket(function);
    assert!(reply.ok, "{:?}", reply.error);
    runtime.render();

    let theme = palette();
    assert_eq!(
        colors_of(&runtime, "Urgent item"),
        named(ColorName::BrightWhite, ColorName::Magenta)
    );
    assert_eq!(
        colors_of(&runtime, "Review item"),
        named(ColorName::Green, ColorName::Black)
    );
    assert_eq!(
        colors_of(&runtime, "Plain item"),
        (ColorName::Cyan.color(), theme.bg),
        "a color the choice leaves out comes from the box"
    );

    // A color that is not a color name is refused, naming the field, and nothing changes.
    let request = request.replace("\"green\"", "\"crimson\"");
    let error = serde_json::from_str::<SocketFunction>(&request)
        .unwrap_err()
        .to_string();
    assert!(error.contains("fg_color"), "{error}");
    assert_eq!(
        colors_of(&runtime, "Review item"),
        named(ColorName::Green, ColorName::Black)
    );
}
