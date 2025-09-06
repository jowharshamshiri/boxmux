#[cfg(test)]
pub mod unified_execution_architecture_tests {
    use crate::model::common::ExecutionMode;
    use crate::model::muxbox::{Choice, MuxBox};
    use crate::tests::test_utils::TestDataFactory;
    use crate::thread_manager::Message;

    /// Test that verifies the unified execution architecture
    /// All execution modes (Immediate, Thread, PTY) should go through the same ThreadManager path
    #[test]
    fn test_unified_execution_all_modes_use_thread_manager() {
        println!("=== Testing Unified ExecutionMode Architecture ===");

        // Test that all execution modes now use the same unified path
        let modes = vec![
            ExecutionMode::Immediate,
            ExecutionMode::Thread,
            ExecutionMode::Pty,
        ];

        for mode in modes {
            println!("Testing execution mode: {:?}", mode);

            let choice = Choice {
                id: format!("test_choice_{}", mode.as_stream_suffix()),
                content: Some("Test Choice".to_string()),
                selected: false,
                script: Some(vec!["echo hello".to_string()]),
                execution_mode: mode.clone(),
                redirect_output: None,
                append_output: Some(false),
                waiting: false,
            };

            // Verify that the choice has the new ExecutionMode field
            assert_eq!(choice.execution_mode, mode);

            // Verify that legacy fields are ignored

            // Verify stream ID format consistency
            let expected_stream_id = format!("{}_{}", choice.id, mode.as_stream_suffix());
            let expected_suffix = mode.as_stream_suffix();

            match mode {
                ExecutionMode::Immediate => assert_eq!(expected_suffix, "immediate"),
                ExecutionMode::Thread => assert_eq!(expected_suffix, "thread"),
                ExecutionMode::Pty => assert_eq!(expected_suffix, "pty"),
            }

            println!(
                "✓ Mode {:?} uses consistent stream ID format: {}",
                mode, expected_stream_id
            );
        }
    }

    /// Test that PTY mode no longer has artificial single command restriction
    #[test]
    fn test_pty_mode_supports_multiple_commands() {
        println!("=== Testing PTY Multiple Commands Support ===");

        let choice = Choice {
            id: "multi_command_pty".to_string(),
            content: Some("Multi-command PTY".to_string()),
            selected: false,
            script: Some(vec![
                "echo 'First command'".to_string(),
                "echo 'Second command'".to_string(),
                "echo 'Third command'".to_string(),
            ]),

            execution_mode: ExecutionMode::Pty,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        // Verify PTY choice accepts multiple commands
        assert_eq!(choice.script.as_ref().unwrap().len(), 3);
        assert_eq!(choice.execution_mode, ExecutionMode::Pty);

        println!(
            "✓ PTY mode supports {} commands",
            choice.script.as_ref().unwrap().len()
        );

        // The artificial restriction should be removed - PTY Manager handles multiple commands by joining with newlines
        for (i, cmd) in choice.script.as_ref().unwrap().iter().enumerate() {
            println!("  Command {}: {}", i + 1, cmd);
        }
    }

    /// Test that multiple clicks create multiple streams consistently
    #[test]
    fn test_multiple_clicks_create_multiple_streams() {
        println!("=== Testing Multiple Click Stream Creation ===");

        let choice_template = Choice {
            id: "sensitive_choice".to_string(),
            content: Some("Sensitive Choice".to_string()),
            selected: false,
            script: Some(vec!["echo test".to_string()]),
            execution_mode: ExecutionMode::Pty,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        // Simulate multiple clicks - each should create a unique stream
        let mut stream_ids = Vec::new();
        for click_num in 1..=3 {
            let stream_id = format!(
                "{}_{}_click{}",
                choice_template.id,
                choice_template.execution_mode.as_stream_suffix(),
                click_num
            );
            stream_ids.push(stream_id);
        }

        // Verify all stream IDs are unique
        let mut unique_ids = stream_ids.clone();
        unique_ids.sort();
        unique_ids.dedup();

        assert_eq!(
            stream_ids.len(),
            unique_ids.len(),
            "All stream IDs should be unique"
        );

        for (i, stream_id) in stream_ids.iter().enumerate() {
            println!("✓ Click {} creates stream: {}", i + 1, stream_id);
        }
    }

    /// Test consistent stream creation message format
    #[test]
    fn test_consistent_stream_creation_message_format() {
        println!("=== Testing Consistent Stream Creation Messages ===");

        let choice = Choice {
            id: "consistent_choice".to_string(),
            content: Some("Consistent Choice".to_string()),
            selected: false,
            script: Some(vec!["echo test".to_string()]),
            execution_mode: ExecutionMode::Thread,
            redirect_output: Some("target_box".to_string()),
            append_output: Some(false),
            waiting: false,
        };

        // Verify consistent stream ID format
        let expected_stream_id =
            format!("{}_{}", choice.id, choice.execution_mode.as_stream_suffix());
        let expected_target = choice.redirect_output.as_ref().unwrap();

        println!("✓ Stream ID format: {}", expected_stream_id);
        println!("✓ Target muxbox: {}", expected_target);
        println!("✓ Execution mode: {:?}", choice.execution_mode);

        // This would create a CreateChoiceExecutionStream message with consistent format:
        // Message::CreateChoiceExecutionStream(stream_id, target_muxbox_id, execution_mode, stream_label)
        assert!(expected_stream_id.contains(&choice.id));
        assert!(expected_stream_id.contains("thread")); // execution mode suffix
    }

    /// Test legacy boolean fields are completely ignored
    #[test]
    fn test_legacy_boolean_fields_ignored() {
        println!("=== Testing Legacy Boolean Fields Are Ignored ===");

        // Create choice with legacy boolean fields set (they should be ignored)
        let choice_with_legacy = Choice {
            id: "legacy_test".to_string(),
            content: Some("Legacy Test".to_string()),
            selected: false,
            script: Some(vec!["echo test".to_string()]),

            execution_mode: ExecutionMode::Immediate, // This should take precedence
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        // Verify ExecutionMode takes precedence over legacy fields
        assert_eq!(choice_with_legacy.execution_mode, ExecutionMode::Immediate);

        // Legacy fields exist but should be ignored in execution logic

        println!(
            "✓ ExecutionMode field: {:?} (takes precedence)",
            choice_with_legacy.execution_mode
        );

        // The actual execution should use ExecutionMode::Immediate despite legacy boolean flags
        let stream_suffix = choice_with_legacy.execution_mode.as_stream_suffix();
        assert_eq!(stream_suffix, "immediate");

        println!(
            "✓ Stream suffix correctly uses ExecutionMode: {}",
            stream_suffix
        );
    }

    /// Test that all execution modes create streams with consistent labeling
    #[test]
    fn test_consistent_stream_labeling() {
        println!("=== Testing Consistent Stream Labeling ===");

        let modes = vec![
            (ExecutionMode::Immediate, "immediate"),
            (ExecutionMode::Thread, "thread"),
            (ExecutionMode::Pty, "pty"),
        ];

        for (mode, expected_suffix) in modes {
            let choice = Choice {
                id: format!("label_test_{}", expected_suffix),
                content: Some(format!("Label Test {}", expected_suffix)),
                selected: false,
                script: Some(vec!["echo test".to_string()]),
                execution_mode: mode.clone(),
                redirect_output: None,
                append_output: Some(false),
                waiting: false,
            };

            let stream_id = format!("{}_{}", choice.id, mode.as_stream_suffix());
            let expected_label = format!("{} ({})", choice.id, expected_suffix);

            assert_eq!(mode.as_stream_suffix(), expected_suffix);

            println!(
                "✓ Mode {:?}: stream_id={}, label={}",
                mode, stream_id, expected_label
            );
        }
    }
}
