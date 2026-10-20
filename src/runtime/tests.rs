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
    let deadline = Instant::now() + Duration::from_secs(5);
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
              script: ['echo hello from greet']
            - id: fail
              content: Fail loudly
              script: ['echo broken >&2', 'exit 3']
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

#[test]
fn clicking_a_choice_runs_its_script_and_shows_the_output() {
    let mut runtime = runtime(MENU, 80, 12);
    let at = find(&runtime, "Say hello");
    click(&mut runtime, at);
    pump_until(&mut runtime, "the choice output", |r| {
        screen_contains(r, "hello from greet")
    });
    let menu = runtime.ctx.app.get_muxbox_by_id("menu").unwrap();
    let choices = menu.choices_stream().unwrap().choices.as_ref().unwrap();
    assert!(choices[0].selected);
    assert!(
        !choices[0].waiting,
        "the spinner stops once the run finished"
    );
}

#[test]
fn clicking_beside_a_choice_does_not_run_it() {
    let mut runtime = runtime(MENU, 80, 12);
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
    let mut runtime = runtime(MENU, 80, 12);
    let at = find(&runtime, "Fail loudly");
    click(&mut runtime, at);
    pump_until(&mut runtime, "the error output", |r| {
        screen_contains(r, "broken")
    });
    let menu = runtime.ctx.app.get_muxbox_by_id("menu").unwrap();
    assert!(menu.in_error_state(), "exit code 3 is shown as an error");
}

#[test]
fn enter_runs_the_choice_selected_with_the_arrow_keys() {
    let mut runtime = runtime(MENU, 80, 12);
    runtime.handle(key(KeyCode::Down, KeyModifiers::NONE));
    runtime.handle(key(KeyCode::Enter, KeyModifiers::NONE));
    pump_until(&mut runtime, "the second choice's output", |r| {
        screen_contains(r, "broken")
    });
}

#[test]
fn the_mouse_wheel_scrolls_the_box_under_the_pointer_by_lines() {
    let mut runtime = runtime(MENU, 80, 12);
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
    let mut runtime = runtime(MENU, 80, 12);
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
    let mut runtime = runtime(MENU, 80, 12);
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
          script: ['echo started', 'sleep 30 & sleep 30']
"#;
    let app = load_app_from_str(yaml, "test.yaml").unwrap();
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
