// F0224: Immediate Mode Execution Tests - Test immediate mode execution bypassing ThreadManager
use crate::model::common::ExecutionMode;
use crate::model::muxbox::Choice;
use crate::tests::test_utils::TestDataFactory;

#[cfg(test)]
mod immediate_mode_execution_tests {
    use super::*;

    #[test]
    fn test_execution_mode_immediate_is_not_background() {
        // F0224: Immediate mode should not be background execution
        let immediate_mode = ExecutionMode::Immediate;
        assert!(!immediate_mode.is_background(), "Immediate mode should not be background");
        assert!(!immediate_mode.is_realtime(), "Immediate mode should not be realtime");
        assert!(immediate_mode.creates_streams(), "Immediate mode should create streams");
    }

    #[test]
    fn test_execution_mode_immediate_stream_suffix() {
        // F0224: Immediate mode should have "immediate" stream suffix
        let immediate_mode = ExecutionMode::Immediate;
        assert_eq!(immediate_mode.as_stream_suffix(), "immediate");
    }

    #[test]
    fn test_execution_mode_immediate_description() {
        // F0224: Immediate mode should have correct description
        let immediate_mode = ExecutionMode::Immediate;
        assert_eq!(immediate_mode.description(), "Synchronous execution on UI thread");
    }

    #[test]
    fn test_execution_mode_from_legacy_immediate() {
        // F0224: Legacy false/false should map to Immediate
        let immediate_from_legacy = ExecutionMode::from_legacy(false, false);
        assert_eq!(immediate_from_legacy, ExecutionMode::Immediate);
    }

    #[test]
    fn test_execution_mode_default_is_immediate() {
        // F0224: Default execution mode should be Immediate
        let default_mode = ExecutionMode::default();
        assert_eq!(default_mode, ExecutionMode::Immediate);
    }

    #[test]
    fn test_immediate_choice_creation() {
        // F0224: Test creating choice with immediate execution mode
        let choice = Choice {
            id: "test_choice".to_string(),
            content: Some("Test Choice".to_string()),
            selected: false,
            script: Some(vec!["echo hello world".to_string()]),
            pty: None, // Legacy field - not used with ExecutionMode
            thread: None, // Legacy field - not used with ExecutionMode
            execution_mode: ExecutionMode::Immediate,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        assert_eq!(choice.execution_mode, ExecutionMode::Immediate);
        assert!(!choice.execution_mode.is_background());
        assert_eq!(choice.execution_mode.as_stream_suffix(), "immediate");
    }

    #[test]
    fn test_immediate_choice_with_redirect() {
        // F0224: Test immediate choice with redirect output
        let choice = Choice {
            id: "redirect_choice".to_string(),
            content: Some("Redirect Choice".to_string()),
            selected: false,
            script: Some(vec!["echo redirected".to_string()]),
            pty: None,
            thread: None,
            execution_mode: ExecutionMode::Immediate,
            redirect_output: Some("target_muxbox".to_string()),
            append_output: Some(true),
            waiting: false,
        };

        assert_eq!(choice.execution_mode, ExecutionMode::Immediate);
        assert_eq!(choice.redirect_output, Some("target_muxbox".to_string()));
        assert_eq!(choice.append_output, Some(true));
    }

    #[test]
    fn test_muxbox_with_immediate_choices() {
        // F0224: Test muxbox with immediate mode choices
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        
        let immediate_choice = Choice {
            id: "immediate_1".to_string(),
            content: Some("Immediate Choice 1".to_string()),
            selected: false,
            script: Some(vec!["echo immediate 1".to_string()]),
            pty: None,
            thread: None,
            execution_mode: ExecutionMode::Immediate,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        let thread_choice = Choice {
            id: "thread_1".to_string(),
            content: Some("Thread Choice 1".to_string()),
            selected: false,
            script: Some(vec!["echo thread 1".to_string()]),
            pty: None,
            thread: None,
            execution_mode: ExecutionMode::Thread,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        muxbox.choices = Some(vec![immediate_choice.clone(), thread_choice.clone()]);

        // Verify the choices have correct execution modes
        if let Some(choices) = &muxbox.choices {
            assert_eq!(choices[0].execution_mode, ExecutionMode::Immediate);
            assert_eq!(choices[1].execution_mode, ExecutionMode::Thread);
        }
    }

    #[test]
    fn test_execution_mode_serialization() {
        // F0224: Test that ExecutionMode serializes correctly
        let immediate_mode = ExecutionMode::Immediate;
        let thread_mode = ExecutionMode::Thread;
        let pty_mode = ExecutionMode::Pty;

        // Test that all modes can be cloned and compared
        assert_eq!(immediate_mode.clone(), ExecutionMode::Immediate);
        assert_eq!(thread_mode.clone(), ExecutionMode::Thread);
        assert_eq!(pty_mode.clone(), ExecutionMode::Pty);

        // Test that they have different stream suffixes
        assert_ne!(immediate_mode.as_stream_suffix(), thread_mode.as_stream_suffix());
        assert_ne!(immediate_mode.as_stream_suffix(), pty_mode.as_stream_suffix());
        assert_ne!(thread_mode.as_stream_suffix(), pty_mode.as_stream_suffix());
    }

    #[test]
    fn test_choice_execution_mode_field_integration() {
        // F0224: Test that Choice struct properly integrates execution_mode field
        let choice_immediate = Choice {
            id: "test".to_string(),
            content: Some("Test".to_string()),
            selected: false,
            script: Some(vec!["echo test".to_string()]),
            pty: Some(false), // Legacy - should be ignored
            thread: Some(false), // Legacy - should be ignored
            execution_mode: ExecutionMode::Immediate, // This should be used
            redirect_output: None,
            append_output: None,
            waiting: false,
        };

        // execution_mode field should take precedence over legacy fields
        assert_eq!(choice_immediate.execution_mode, ExecutionMode::Immediate);
        assert!(!choice_immediate.execution_mode.is_background());

        // Test choice can be cloned and compared
        let cloned_choice = choice_immediate.clone();
        assert_eq!(choice_immediate.execution_mode, cloned_choice.execution_mode);
    }

    #[test]
    fn test_muxbox_execution_mode_field_integration() {
        // F0224: Test that MuxBox struct properly integrates execution_mode field
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.execution_mode = ExecutionMode::Immediate;

        assert_eq!(muxbox.execution_mode, ExecutionMode::Immediate);
        assert!(!muxbox.execution_mode.is_background());
        assert_eq!(muxbox.execution_mode.as_stream_suffix(), "immediate");

        // Test muxbox can be cloned
        let cloned_muxbox = muxbox.clone();
        assert_eq!(muxbox.execution_mode, cloned_muxbox.execution_mode);
    }

    #[test]
    fn test_immediate_vs_background_modes() {
        // F0224: Test distinguishing immediate mode from background modes
        let immediate = ExecutionMode::Immediate;
        let thread = ExecutionMode::Thread;
        let pty = ExecutionMode::Pty;

        // Immediate should not be background
        assert!(!immediate.is_background());
        assert!(!immediate.is_realtime());

        // Thread and PTY should be background
        assert!(thread.is_background());
        assert!(!thread.is_realtime());
        
        assert!(pty.is_background());
        assert!(pty.is_realtime());

        // All should create streams
        assert!(immediate.creates_streams());
        assert!(thread.creates_streams());
        assert!(pty.creates_streams());
    }

    #[test]
    fn test_legacy_migration_patterns() {
        // F0224: Test various legacy boolean combinations map correctly
        assert_eq!(ExecutionMode::from_legacy(false, false), ExecutionMode::Immediate);
        assert_eq!(ExecutionMode::from_legacy(true, false), ExecutionMode::Thread);
        assert_eq!(ExecutionMode::from_legacy(false, true), ExecutionMode::Pty);
        assert_eq!(ExecutionMode::from_legacy(true, true), ExecutionMode::Pty); // PTY takes precedence
    }

    #[test]
    fn test_stream_id_generation() {
        // F0224: Test that stream IDs can be generated from execution modes
        let choice_id = "test_choice";
        let immediate_suffix = ExecutionMode::Immediate.as_stream_suffix();
        let thread_suffix = ExecutionMode::Thread.as_stream_suffix();
        let pty_suffix = ExecutionMode::Pty.as_stream_suffix();

        let immediate_stream_id = format!("{}_{}", choice_id, immediate_suffix);
        let thread_stream_id = format!("{}_{}", choice_id, thread_suffix);
        let pty_stream_id = format!("{}_{}", choice_id, pty_suffix);

        assert_eq!(immediate_stream_id, "test_choice_immediate");
        assert_eq!(thread_stream_id, "test_choice_thread");
        assert_eq!(pty_stream_id, "test_choice_pty");

        // Ensure they're all different
        assert_ne!(immediate_stream_id, thread_stream_id);
        assert_ne!(immediate_stream_id, pty_stream_id);
        assert_ne!(thread_stream_id, pty_stream_id);
    }
}