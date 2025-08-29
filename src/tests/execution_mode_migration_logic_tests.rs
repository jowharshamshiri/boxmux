/// F0229: ExecutionMode Migration Logic Tests
/// Comprehensive tests for automatic migration of legacy thread/pty fields to ExecutionMode
/// Validates backward compatibility with existing YAML configurations

#[cfg(test)]
mod execution_mode_migration_logic_tests {
    use crate::model::app::load_app_from_yaml;
    use crate::model::common::ExecutionMode;
    use crate::model::muxbox::{Choice, MuxBox};
    use tempfile::NamedTempFile;
    use std::io::Write;

    /// Test migration from legacy thread=true to ExecutionMode::Thread
    #[test]
    fn test_migration_legacy_thread_true() {
        let yaml_content = r#"
app:
  layouts:
    - id: test_layout
      root: true
      children:
        - id: thread_box
          title: "Thread Test"
          position: {x1: "0%", y1: "0%", x2: "100%", y2: "100%"}
          thread: true
          script: ["echo 'test'"]
        "#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let app = load_app_from_yaml(file_path).expect("Failed to load app from YAML");
        
        let thread_box = app.get_muxbox_by_id("thread_box").expect("thread_box not found");
        assert_eq!(thread_box.execution_mode, ExecutionMode::Thread);
        assert_eq!(thread_box.thread, Some(true)); // Legacy field preserved
    }

    /// Test migration from legacy pty=true to ExecutionMode::Pty
    #[test]
    fn test_migration_legacy_pty_true() {
        let yaml_content = r#"
app:
  layouts:
    - id: test_layout
      root: true
      children:
        - id: pty_box
          title: "PTY Test"
          position: {x1: "0%", y1: "0%", x2: "100%", y2: "100%"}
          pty: true
          script: ["echo 'test'"]
        "#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let app = load_app_from_yaml(file_path).expect("Failed to load app from YAML");
        
        let pty_box = app.get_muxbox_by_id("pty_box").expect("pty_box not found");
        assert_eq!(pty_box.execution_mode, ExecutionMode::Pty);
        assert_eq!(pty_box.pty, Some(true)); // Legacy field preserved
    }

    /// Test migration with both thread=true and pty=true - PTY should take precedence
    #[test]
    fn test_migration_pty_precedence_over_thread() {
        let yaml_content = r#"
app:
  layouts:
    - id: test_layout
      root: true
      children:
        - id: mixed_box
          title: "Mixed Test"
          position: {x1: "0%", y1: "0%", x2: "100%", y2: "100%"}
          thread: true
          pty: true
          script: ["echo 'test'"]
        "#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let app = load_app_from_yaml(file_path).expect("Failed to load app from YAML");
        
        let mixed_box = app.get_muxbox_by_id("mixed_box").expect("mixed_box not found");
        // PTY takes precedence over thread
        assert_eq!(mixed_box.execution_mode, ExecutionMode::Pty);
        assert_eq!(mixed_box.thread, Some(true));
        assert_eq!(mixed_box.pty, Some(true));
    }

    /// Test migration when no legacy fields present - should remain Immediate
    #[test]
    fn test_migration_no_legacy_fields() {
        let yaml_content = r#"
app:
  layouts:
    - id: test_layout
      root: true
      children:
        - id: default_box
          title: "Default Test"
          position: {x1: "0%", y1: "0%", x2: "100%", y2: "100%"}
          script: ["echo 'test'"]
        "#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let app = load_app_from_yaml(file_path).expect("Failed to load app from YAML");
        
        let default_box = app.get_muxbox_by_id("default_box").expect("default_box not found");
        assert_eq!(default_box.execution_mode, ExecutionMode::Immediate);
    }

    /// Test migration when execution_mode is explicitly set - should not override (unit test version)
    #[test]
    fn test_migration_explicit_execution_mode_not_overridden_unit() {
        let mut muxbox = MuxBox {
            id: "explicit_box".to_string(),
            execution_mode: ExecutionMode::Thread, // Explicitly set to Thread
            thread: Some(false), // Legacy would suggest Immediate
            pty: Some(true),     // Legacy would suggest PTY
            ..Default::default()
        };
        
        muxbox.migrate_execution_mode();
        // Should preserve explicit setting, not migrate from legacy fields
        assert_eq!(muxbox.execution_mode, ExecutionMode::Thread);
    }

    /// Test choice migration from legacy thread=true to ExecutionMode::Thread
    #[test]
    fn test_choice_migration_legacy_thread_true() {
        let yaml_content = r#"
app:
  layouts:
    - id: test_layout
      root: true
      children:
        - id: choice_box
          title: "Choice Test"
          position: {x1: "0%", y1: "0%", x2: "100%", y2: "100%"}
          choices:
            - id: thread_choice
              content: "Thread Choice"
              script: ["echo 'thread choice'"]
              thread: true
        "#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let app = load_app_from_yaml(file_path).expect("Failed to load app from YAML");
        
        let choice_box = app.get_muxbox_by_id("choice_box").expect("choice_box not found");
        let choices = choice_box.choices.as_ref().expect("No choices found");
        let thread_choice = &choices[0];
        
        assert_eq!(thread_choice.execution_mode, ExecutionMode::Thread);
        assert_eq!(thread_choice.thread, Some(true));
    }

    /// Test choice migration from legacy pty=true to ExecutionMode::Pty
    #[test]
    fn test_choice_migration_legacy_pty_true() {
        let yaml_content = r#"
app:
  layouts:
    - id: test_layout
      root: true
      children:
        - id: choice_box
          title: "Choice Test"
          position: {x1: "0%", y1: "0%", x2: "100%", y2: "100%"}
          choices:
            - id: pty_choice
              content: "PTY Choice"
              script: ["echo 'pty choice'"]
              pty: true
        "#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let app = load_app_from_yaml(file_path).expect("Failed to load app from YAML");
        
        let choice_box = app.get_muxbox_by_id("choice_box").expect("choice_box not found");
        let choices = choice_box.choices.as_ref().expect("No choices found");
        let pty_choice = &choices[0];
        
        assert_eq!(pty_choice.execution_mode, ExecutionMode::Pty);
        assert_eq!(pty_choice.pty, Some(true));
    }

    /// Test nested muxbox migration - parent and child should both migrate
    #[test]
    fn test_nested_muxbox_migration() {
        let yaml_content = r#"
app:
  layouts:
    - id: test_layout
      root: true
      children:
        - id: parent_box
          title: "Parent Test"
          position: {x1: "0%", y1: "0%", x2: "100%", y2: "50%"}
          thread: true
          script: ["echo 'parent'"]
          children:
            - id: child_box
              title: "Child Test"
              position: {x1: "0%", y1: "0%", x2: "100%", y2: "100%"}
              pty: true
              script: ["echo 'child'"]
        "#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let app = load_app_from_yaml(file_path).expect("Failed to load app from YAML");
        
        let parent_box = app.get_muxbox_by_id("parent_box").expect("parent_box not found");
        let child_box = app.get_muxbox_by_id("child_box").expect("child_box not found");
        
        assert_eq!(parent_box.execution_mode, ExecutionMode::Thread);
        assert_eq!(child_box.execution_mode, ExecutionMode::Pty);
    }

    /// Test complex YAML with mixed configurations
    #[test]
    fn test_complex_migration_scenario() {
        let yaml_content = r#"
app:
  layouts:
    - id: test_layout
      root: true
      children:
        - id: immediate_box
          title: "Immediate Box"
          position: {x1: "0%", y1: "0%", x2: "33%", y2: "100%"}
          script: ["echo 'immediate'"]
        - id: thread_box  
          title: "Thread Box"
          position: {x1: "33%", y1: "0%", x2: "66%", y2: "100%"}
          thread: true
          script: ["echo 'thread'"]
          choices:
            - id: immediate_choice
              content: "Immediate Choice"
              script: ["echo 'immediate choice'"]
            - id: thread_choice
              content: "Thread Choice"
              script: ["echo 'thread choice'"]
              thread: true
        - id: pty_box
          title: "PTY Box"
          position: {x1: "66%", y1: "0%", x2: "100%", y2: "100%"}
          pty: true
          script: ["echo 'pty'"]
          choices:
            - id: pty_choice
              content: "PTY Choice"
              script: ["echo 'pty choice'"]
              pty: true
        "#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write to temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let app = load_app_from_yaml(file_path).expect("Failed to load app from YAML");
        
        // Verify muxbox migrations
        let immediate_box = app.get_muxbox_by_id("immediate_box").expect("immediate_box not found");
        let thread_box = app.get_muxbox_by_id("thread_box").expect("thread_box not found");
        let pty_box = app.get_muxbox_by_id("pty_box").expect("pty_box not found");
        
        assert_eq!(immediate_box.execution_mode, ExecutionMode::Immediate);
        assert_eq!(thread_box.execution_mode, ExecutionMode::Thread);
        assert_eq!(pty_box.execution_mode, ExecutionMode::Pty);
        
        // Verify choice migrations
        let thread_choices = thread_box.choices.as_ref().expect("thread_box has no choices");
        assert_eq!(thread_choices[0].execution_mode, ExecutionMode::Immediate); // immediate_choice
        assert_eq!(thread_choices[1].execution_mode, ExecutionMode::Thread);    // thread_choice
        
        let pty_choices = pty_box.choices.as_ref().expect("pty_box has no choices");
        assert_eq!(pty_choices[0].execution_mode, ExecutionMode::Pty); // pty_choice
    }

    /// Test unit-level migration methods directly  
    #[test]
    fn test_muxbox_migrate_execution_mode_unit() {
        let mut muxbox = MuxBox {
            id: "test_box".to_string(),
            execution_mode: ExecutionMode::default(), // Immediate
            thread: Some(true),
            pty: Some(false),
            ..Default::default()
        };
        
        muxbox.migrate_execution_mode();
        assert_eq!(muxbox.execution_mode, ExecutionMode::Thread);
    }

    /// Test unit-level choice migration method directly
    #[test]
    fn test_choice_migrate_execution_mode_unit() {
        let mut choice = Choice {
            id: "test_choice".to_string(),
            execution_mode: ExecutionMode::default(), // Immediate
            thread: Some(false),
            pty: Some(true),
            ..Default::default()
        };
        
        choice.migrate_execution_mode();
        assert_eq!(choice.execution_mode, ExecutionMode::Pty);
    }

    /// Test that migration doesn't override already set execution_mode
    #[test]
    fn test_migration_preserves_explicit_execution_mode() {
        let mut muxbox = MuxBox {
            id: "test_box".to_string(),
            execution_mode: ExecutionMode::Thread, // Explicitly set
            thread: Some(false), // Would suggest Immediate
            pty: Some(true),     // Would suggest Pty
            ..Default::default()
        };
        
        muxbox.migrate_execution_mode();
        // Should preserve explicit setting, not migrate from legacy fields
        assert_eq!(muxbox.execution_mode, ExecutionMode::Thread);
    }
}