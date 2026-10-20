//! `execution_mode` is the only way to choose how a script runs; the old `thread` and `pty`
//! flags are rejected, not silently ignored.

use crate::model::app::load_app_from_str;
use crate::model::common::ExecutionMode;

fn app_yaml(box_fields: &str, choice_fields: &str) -> String {
    format!(
        r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: worker
          position: {{x1: 0%, y1: 0%, x2: 100%, y2: 100%}}
          script: ['echo box']
{box_fields}
          choices:
            - id: run
              content: Run
              script: ['echo choice']
{choice_fields}
"#
    )
}

fn load(yaml: &str) -> Result<crate::App, String> {
    load_app_from_str(yaml, "test.yaml")
}

#[test]
fn every_mode_is_accepted_for_boxes_and_choices() {
    for (name, mode) in [
        ("Immediate", ExecutionMode::Immediate),
        ("Thread", ExecutionMode::Thread),
        ("Pty", ExecutionMode::Pty),
    ] {
        let app = load(&app_yaml(
            &format!("          execution_mode: {}", name),
            &format!("              execution_mode: {}", name),
        ))
        .unwrap_or_else(|e| panic!("{}: {}", name, e));
        let worker = app.layouts[0].get_muxbox_by_id("worker").unwrap();
        assert_eq!(worker.execution_mode, mode);
        assert_eq!(worker.choices.as_ref().unwrap()[0].execution_mode, mode);
    }
}

#[test]
fn the_default_mode_runs_in_the_background_and_shows_output_when_done() {
    let app = load(&app_yaml("", "")).unwrap();
    let worker = app.layouts[0].get_muxbox_by_id("worker").unwrap();
    assert_eq!(worker.execution_mode, ExecutionMode::Immediate);
}

#[test]
fn unknown_modes_are_rejected() {
    let error = load(&app_yaml("          execution_mode: Background", "")).unwrap_err();
    assert!(error.contains("execution_mode"), "{}", error);
}

#[test]
fn the_removed_thread_and_pty_flags_are_rejected() {
    for flag in ["thread: true", "pty: true"] {
        let key = flag.split(':').next().unwrap();
        let error = load(&app_yaml(&format!("          {}", flag), "")).unwrap_err();
        assert!(error.contains(key), "`{}` gave: {}", flag, error);
        let error = load(&app_yaml("", &format!("              {}", flag))).unwrap_err();
        assert!(error.contains(key), "choice `{}` gave: {}", flag, error);
    }
}

#[test]
fn a_pty_box_cannot_refresh_on_an_interval() {
    let error = load(&app_yaml(
        "          execution_mode: Pty\n          refresh_interval: 1000",
        "",
    ))
    .unwrap_err();
    assert!(error.contains("refresh_interval"), "{}", error);
}
