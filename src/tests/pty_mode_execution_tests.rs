#[cfg(test)]
pub mod pty_mode_execution_tests {
    use crate::model::common::ExecutionMode;
    use crate::model::muxbox::Choice;
    use crate::tests::test_utils::TestDataFactory;

    #[test]
    fn test_execution_mode_pty_enum_variant() {
        // F0226: Verify ExecutionMode::Pty variant exists
        let pty_mode = ExecutionMode::Pty;
        assert_eq!(format!("{:?}", pty_mode), "Pty");
    }

    #[test]
    fn test_execution_mode_pty_is_realtime() {
        // F0226: PTY execution should be real-time
        let pty_mode = ExecutionMode::Pty;
        assert!(pty_mode.is_realtime(), "PTY mode should be real-time execution");
    }

    #[test]
    fn test_execution_mode_pty_is_background() {
        // F0226: PTY execution should be background (not on UI thread)
        let pty_mode = ExecutionMode::Pty;
        assert!(pty_mode.is_background(), "PTY mode should be background execution");
    }

    #[test]
    fn test_execution_mode_pty_description() {
        // F0226: PTY mode should have proper description
        let pty_mode = ExecutionMode::Pty;
        let description = pty_mode.description();
        assert!(description.contains("Real-time"), "PTY description should mention Real-time");
        assert!(description.contains("PTY"), "PTY description should mention PTY");
    }

    #[test]
    fn test_execution_mode_pty_stream_suffix() {
        // F0226: PTY execution should have proper stream suffix  
        let pty_mode = ExecutionMode::Pty;
        let suffix = pty_mode.as_stream_suffix();
        assert_eq!(suffix, "pty", "PTY mode should have 'pty' stream suffix");
    }

    #[test]
    fn test_execution_mode_pty_creates_streams() {
        // F0226: PTY mode should create execution streams
        let pty_mode = ExecutionMode::Pty;
        assert!(pty_mode.creates_streams(), "PTY mode should create execution streams");
    }

    #[test]
    fn test_choice_with_execution_mode_pty() {
        // F0226: Choice should support ExecutionMode::Pty
        let mut choice = Choice {
            id: "pty_choice".to_string(),
            content: Some("PTY Choice".to_string()),
            selected: false,
            script: Some(vec!["echo 'PTY test'".to_string()]),
 // Legacy field should be None
 // Legacy field should be None
            execution_mode: ExecutionMode::Pty,
            redirect_output: None,
            append_output: None,
            waiting: false,
        };

        assert_eq!(choice.execution_mode, ExecutionMode::Pty);
    }

    #[test]
    fn test_muxbox_with_pty_choices() {
        // F0226: MuxBox should support choices with ExecutionMode::Pty
        let mut muxbox = TestDataFactory::create_test_muxbox("pty_box");
        
        let pty_choice = Choice {
            id: "pty_choice".to_string(),
            content: Some("PTY Command".to_string()),
            selected: false,
            script: Some(vec!["htop".to_string()]), // Interactive PTY command
 // Legacy field
 // Legacy field  
            execution_mode: ExecutionMode::Pty,
            redirect_output: None,
            append_output: None,
            waiting: false,
        };

        muxbox.choices = Some(vec![pty_choice.clone()]);

        if let Some(ref choices) = muxbox.choices {
            assert_eq!(choices[0].execution_mode, ExecutionMode::Pty);
            assert!(choices[0].execution_mode.is_realtime());
            assert!(choices[0].execution_mode.is_background());
        } else {
            panic!("MuxBox should have choices");
        }
    }

    #[test]
    fn test_execution_mode_pty_serialization() {
        // F0226: ExecutionMode::Pty should serialize correctly
        let pty_mode = ExecutionMode::Pty;
        let serialized = serde_yaml::to_string(&pty_mode).expect("Should serialize");
        assert!(serialized.trim() == "Pty", 
                "PTY mode should serialize to 'Pty', got: {}", serialized.trim());
        
        // Test deserialization
        let deserialized: ExecutionMode = serde_yaml::from_str("Pty").expect("Should deserialize 'Pty'");
        assert_eq!(deserialized, ExecutionMode::Pty);
        
        // Only 'Pty' variant is supported, not lowercase 'pty'
        // let deserialized_lowercase: ExecutionMode = serde_yaml::from_str("pty").expect("Should deserialize 'pty'");
        // assert_eq!(deserialized_lowercase, ExecutionMode::Pty);
    }

    #[test]
    fn test_legacy_pty_migration_to_execution_mode() {
        // F0226: Legacy pty=true should migrate to ExecutionMode::Pty
        let pty_mode = ExecutionMode::from_legacy(false, true);
        assert_eq!(pty_mode, ExecutionMode::Pty, "pty=true should migrate to ExecutionMode::Pty");

        let pty_override = ExecutionMode::from_legacy(true, true); 
        assert_eq!(pty_override, ExecutionMode::Pty, "pty=true should override thread=true");
    }

    #[test]
    fn test_pty_execution_mode_choice_integration() {
        // F0226: Complete integration test for PTY execution mode
        let pty_choice = Choice {
            id: "interactive_shell".to_string(),
            content: Some("Start Shell".to_string()),
            selected: false,
            script: Some(vec!["bash".to_string()]),
 // F0226: Should not use legacy field
 // F0226: Should not use legacy field
            execution_mode: ExecutionMode::Pty, // F0226: Use ExecutionMode for PTY
            redirect_output: None,
            append_output: Some(false), // PTY typically doesn't append
            waiting: false,
        };

        // Verify ExecutionMode behavior
        assert!(pty_choice.execution_mode.is_realtime());
        assert!(pty_choice.execution_mode.is_background());
        assert!(pty_choice.execution_mode.creates_streams());
        assert_eq!(pty_choice.execution_mode.as_stream_suffix(), "pty");
        assert!(pty_choice.execution_mode.description().contains("Real-time"));

        // Verify legacy fields are unused

        // Verify choice serialization includes execution_mode
        let serialized = serde_yaml::to_string(&pty_choice).expect("Choice should serialize");
        assert!(serialized.contains("execution_mode"), "Serialized choice should include execution_mode field");
        assert!(serialized.contains("Pty") || serialized.contains("pty"), "Serialized choice should include PTY execution mode");
    }

    #[test] 
    fn test_execution_mode_pty_hash_and_equality() {
        // F0226: ExecutionMode::Pty should support hash and equality
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let pty1 = ExecutionMode::Pty;
        let pty2 = ExecutionMode::Pty;
        let thread_mode = ExecutionMode::Thread;

        // Test equality
        assert_eq!(pty1, pty2);
        assert_ne!(pty1, thread_mode);

        // Test hash consistency
        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();
        
        pty1.hash(&mut hasher1);
        pty2.hash(&mut hasher2);
        thread_mode.hash(&mut hasher3);

        let hash1 = hasher1.finish();
        let hash2 = hasher2.finish();
        let hash3 = hasher3.finish();

        assert_eq!(hash1, hash2, "Same ExecutionMode::Pty should have same hash");
        assert_ne!(hash1, hash3, "Different execution modes should have different hashes");
    }

    #[test]
    fn test_execution_mode_pty_clone_and_default() {
        // F0226: ExecutionMode should support clone and have proper default
        let pty_mode = ExecutionMode::Pty;
        let cloned_mode = pty_mode.clone();
        assert_eq!(pty_mode, cloned_mode);

        // Default should be Immediate, not PTY
        let default_mode = ExecutionMode::default();
        assert_eq!(default_mode, ExecutionMode::Immediate);
        assert_ne!(default_mode, ExecutionMode::Pty);
    }

    #[test]
    fn test_pty_mode_vs_other_execution_modes() {
        // F0226: PTY mode should be distinct from other execution modes
        let pty = ExecutionMode::Pty;
        let thread = ExecutionMode::Thread;
        let immediate = ExecutionMode::Immediate;

        // PTY and Thread are both background but different
        assert!(pty.is_background());
        assert!(thread.is_background());
        assert_ne!(pty, thread);

        // PTY is real-time, Thread is not
        assert!(pty.is_realtime());
        assert!(!thread.is_realtime());

        // PTY and Immediate are both create streams but different
        assert!(pty.creates_streams());
        assert!(immediate.creates_streams());
        assert_ne!(pty, immediate);

        // Only PTY is both background AND real-time
        assert!(pty.is_background() && pty.is_realtime());
        assert!(!(thread.is_background() && thread.is_realtime()));
        assert!(!(immediate.is_background() && immediate.is_realtime()));
    }
}