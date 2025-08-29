/// F0221-F0222: ExecutionMode Migration Tests
/// Tests for MuxBox and Choice ExecutionMode field integration and legacy migration
/// Validates ExecutionMode field functionality in MuxBox and Choice structs

#[cfg(test)]
mod execution_mode_migration_tests {
    use crate::model::common::ExecutionMode;
    use crate::model::muxbox::{Choice, MuxBox};

    // === MuxBox ExecutionMode Integration Tests ===

    /// Test MuxBox default creation includes ExecutionMode::Immediate
    #[test]
    fn test_muxbox_default_execution_mode() {
        let muxbox = MuxBox::default();
        assert_eq!(muxbox.execution_mode, ExecutionMode::Immediate);
    }

    /// Test MuxBox hash includes ExecutionMode field
    #[test]
    fn test_muxbox_hash_includes_execution_mode() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut muxbox1 = MuxBox::default();
        muxbox1.execution_mode = ExecutionMode::Thread;

        let mut muxbox2 = MuxBox::default();
        muxbox2.execution_mode = ExecutionMode::Pty;

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        muxbox1.hash(&mut hasher1);
        muxbox2.hash(&mut hasher2);

        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    /// Test MuxBox equality considers ExecutionMode field
    #[test]
    fn test_muxbox_equality_includes_execution_mode() {
        let mut muxbox1 = MuxBox::default();
        muxbox1.id = "test".to_string();
        muxbox1.execution_mode = ExecutionMode::Thread;

        let mut muxbox2 = MuxBox::default();
        muxbox2.id = "test".to_string();
        muxbox2.execution_mode = ExecutionMode::Pty;

        assert_ne!(muxbox1, muxbox2);

        muxbox2.execution_mode = ExecutionMode::Thread;
        assert_eq!(muxbox1, muxbox2);
    }

    /// Test MuxBox clone preserves ExecutionMode field
    #[test]
    fn test_muxbox_clone_preserves_execution_mode() {
        let mut muxbox = MuxBox::default();
        muxbox.execution_mode = ExecutionMode::Pty;

        let cloned = muxbox.clone();
        assert_eq!(cloned.execution_mode, ExecutionMode::Pty);
    }

    /// Test MuxBox serialization includes ExecutionMode field
    #[test]
    fn test_muxbox_serialization_includes_execution_mode() {
        let mut muxbox = MuxBox::default();
        muxbox.id = "test".to_string();
        muxbox.execution_mode = ExecutionMode::Thread;

        let serialized = serde_json::to_string(&muxbox).expect("Serialization failed");
        assert!(serialized.contains("execution_mode"));
        assert!(serialized.contains("Thread"));

        let deserialized: MuxBox = serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(deserialized.execution_mode, ExecutionMode::Thread);
    }

    // === Choice ExecutionMode Integration Tests ===

    /// Test Choice creation with ExecutionMode field
    #[test]
    fn test_choice_execution_mode_creation() {
        let choice = Choice {
            id: "test".to_string(),
            content: None,
            script: None,
            thread: None,
            redirect_output: None,
            append_output: None,
            pty: None,
            execution_mode: ExecutionMode::Thread,
            selected: false,
            waiting: false,
        };

        assert_eq!(choice.execution_mode, ExecutionMode::Thread);
    }

    /// Test Choice hash includes ExecutionMode field
    #[test]
    fn test_choice_hash_includes_execution_mode() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut choice1 = Choice {
            id: "test".to_string(),
            content: None,
            script: None,
            thread: None,
            redirect_output: None,
            append_output: None,
            pty: None,
            execution_mode: ExecutionMode::Thread,
            selected: false,
            waiting: false,
        };

        let mut choice2 = choice1.clone();
        choice2.execution_mode = ExecutionMode::Pty;

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();

        choice1.hash(&mut hasher1);
        choice2.hash(&mut hasher2);

        assert_ne!(hasher1.finish(), hasher2.finish());
    }

    /// Test Choice equality considers ExecutionMode field
    #[test]
    fn test_choice_equality_includes_execution_mode() {
        let choice1 = Choice {
            id: "test".to_string(),
            content: None,
            script: None,
            thread: None,
            redirect_output: None,
            append_output: None,
            pty: None,
            execution_mode: ExecutionMode::Thread,
            selected: false,
            waiting: false,
        };

        let mut choice2 = choice1.clone();
        choice2.execution_mode = ExecutionMode::Pty;

        assert_ne!(choice1, choice2);

        choice2.execution_mode = ExecutionMode::Thread;
        assert_eq!(choice1, choice2);
    }

    /// Test Choice clone preserves ExecutionMode field
    #[test]
    fn test_choice_clone_preserves_execution_mode() {
        let choice = Choice {
            id: "test".to_string(),
            content: None,
            script: None,
            thread: None,
            redirect_output: None,
            append_output: None,
            pty: None,
            execution_mode: ExecutionMode::Pty,
            selected: false,
            waiting: false,
        };

        let cloned = choice.clone();
        assert_eq!(cloned.execution_mode, ExecutionMode::Pty);
    }

    /// Test Choice serialization includes ExecutionMode field
    #[test]
    fn test_choice_serialization_includes_execution_mode() {
        let choice = Choice {
            id: "test".to_string(),
            content: None,
            script: None,
            thread: None,
            redirect_output: None,
            append_output: None,
            pty: None,
            execution_mode: ExecutionMode::Thread,
            selected: false,
            waiting: false,
        };

        let serialized = serde_json::to_string(&choice).expect("Serialization failed");
        assert!(serialized.contains("execution_mode"));
        assert!(serialized.contains("Thread"));

        let deserialized: Choice = serde_json::from_str(&serialized).expect("Deserialization failed");
        assert_eq!(deserialized.execution_mode, ExecutionMode::Thread);
    }

    // === Legacy Migration Integration Tests ===

    /// Test legacy boolean migration in MuxBox context
    #[test]
    fn test_legacy_migration_integration() {
        // Simulate legacy behavior where thread and pty booleans are converted
        let test_cases = vec![
            (false, false, ExecutionMode::Immediate),
            (true, false, ExecutionMode::Thread),
            (false, true, ExecutionMode::Pty),
            (true, true, ExecutionMode::Pty), // PTY takes precedence
        ];

        for (thread, pty, expected_mode) in test_cases {
            let mut muxbox = MuxBox::default();
            // F0229: Migration logic - convert legacy booleans to ExecutionMode
            muxbox.execution_mode = ExecutionMode::from_legacy(thread, pty);
            assert_eq!(muxbox.execution_mode, expected_mode,
                "Migration failed for thread={}, pty={}", thread, pty);
        }
    }

    /// Test ExecutionMode field coexistence with legacy fields
    #[test]
    fn test_execution_mode_coexistence_with_legacy() {
        let mut muxbox = MuxBox::default();
        muxbox.thread = Some(true);
        muxbox.pty = Some(false);
        muxbox.execution_mode = ExecutionMode::Thread;

        // Both legacy and new fields should coexist during transition period
        assert_eq!(muxbox.thread, Some(true));
        assert_eq!(muxbox.pty, Some(false));
        assert_eq!(muxbox.execution_mode, ExecutionMode::Thread);

        let mut choice = Choice {
            id: "test".to_string(),
            content: None,
            script: None,
            thread: Some(false),
            redirect_output: None,
            append_output: None,
            pty: Some(true),
            execution_mode: ExecutionMode::Pty,
            selected: false,
            waiting: false,
        };

        assert_eq!(choice.thread, Some(false));
        assert_eq!(choice.pty, Some(true));
        assert_eq!(choice.execution_mode, ExecutionMode::Pty);
    }

    /// Test ExecutionMode default behavior in YAML serialization
    #[test]
    fn test_execution_mode_yaml_default() {
        let yaml_without_execution_mode = r#"
id: "test_box"
position:
  x1: "0%"
  y1: "0%"
  x2: "100%"
  y2: "100%"
"#;

        let muxbox: Result<MuxBox, _> = serde_yaml::from_str(yaml_without_execution_mode);
        assert!(muxbox.is_ok());
        let muxbox = muxbox.unwrap();
        assert_eq!(muxbox.execution_mode, ExecutionMode::Immediate);
    }

    /// Test ExecutionMode explicit YAML serialization
    #[test]
    fn test_execution_mode_yaml_explicit() {
        let yaml_with_execution_mode = r#"
id: "test_box"
position:
  x1: "0%"
  y1: "0%"
  x2: "100%"
  y2: "100%"
execution_mode: "Thread"
"#;

        let muxbox: Result<MuxBox, _> = serde_yaml::from_str(yaml_with_execution_mode);
        assert!(muxbox.is_ok());
        let muxbox = muxbox.unwrap();
        assert_eq!(muxbox.execution_mode, ExecutionMode::Thread);
    }
}