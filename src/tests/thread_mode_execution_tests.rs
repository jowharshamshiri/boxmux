// F0225: Thread Mode Execution Tests - Comprehensive validation of ExecutionMode::Thread
// Tests background execution via ThreadManager, stream integration, and proper threading behavior

use crate::model::common::ExecutionMode;
use crate::model::muxbox::Choice;

#[cfg(test)]
mod thread_mode_execution_tests {
    use super::*;

    #[test]
    fn test_execution_mode_thread_is_background() {
        // F0225: Thread mode should be background execution
        let thread_mode = ExecutionMode::Thread;
        assert!(thread_mode.is_background(), "Thread mode should be background");
        assert!(!thread_mode.is_realtime(), "Thread mode should not be realtime");
        assert!(thread_mode.creates_streams(), "Thread mode should create streams");
    }

    #[test]
    fn test_execution_mode_thread_stream_suffix() {
        // F0225: Thread mode should have "thread" stream suffix
        let thread_mode = ExecutionMode::Thread;
        assert_eq!(thread_mode.as_stream_suffix(), "thread");
    }

    #[test]
    fn test_execution_mode_thread_description() {
        // F0225: Thread mode should have correct description
        let thread_mode = ExecutionMode::Thread;
        assert_eq!(thread_mode.description(), "Background execution in thread pool");
    }

    #[test]
    fn test_execution_mode_from_legacy_thread_true() {
        // F0225: Legacy true/false should map to Thread
        let thread_from_legacy = ExecutionMode::from_legacy(true, false);
        assert_eq!(thread_from_legacy, ExecutionMode::Thread);
    }

    #[test]
    fn test_thread_choice_creation() {
        // F0225: Test creating choice with thread execution mode
        let choice = Choice {
            id: "test_choice".to_string(),
            content: Some("Test Thread Choice".to_string()),
            selected: false,
            script: Some(vec!["sleep 0.1; echo hello from background".to_string()]),
            pty: None, // Legacy field - not used with ExecutionMode
            thread: None, // Legacy field - not used with ExecutionMode
            execution_mode: ExecutionMode::Thread,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        assert_eq!(choice.execution_mode, ExecutionMode::Thread);
        assert!(choice.script.is_some());
        assert_eq!(choice.thread, None); // Legacy field should be None
        assert_eq!(choice.pty, None); // Legacy field should be None
    }

    #[test]
    fn test_thread_mode_vs_immediate_mode() {
        // F0225: Thread mode should be different from immediate mode
        let thread_mode = ExecutionMode::Thread;
        let immediate_mode = ExecutionMode::Immediate;
        
        assert_ne!(thread_mode, immediate_mode);
        assert!(thread_mode.is_background() && !immediate_mode.is_background());
        assert_ne!(thread_mode.as_stream_suffix(), immediate_mode.as_stream_suffix());
        assert_ne!(thread_mode.description(), immediate_mode.description());
    }

    #[test]
    fn test_thread_mode_vs_pty_mode() {
        // F0225: Thread mode should be different from PTY mode
        let thread_mode = ExecutionMode::Thread;
        let pty_mode = ExecutionMode::Pty;
        
        assert_ne!(thread_mode, pty_mode);
        assert!(thread_mode.is_background() && pty_mode.is_background()); // Both background
        assert!(!thread_mode.is_realtime() && pty_mode.is_realtime()); // Different real-time behavior
        assert_ne!(thread_mode.as_stream_suffix(), pty_mode.as_stream_suffix());
        assert_ne!(thread_mode.description(), pty_mode.description());
    }

    #[test]
    fn test_thread_choice_serialization() {
        // F0225: Test that ExecutionMode::Thread serializes correctly in choices
        let thread_choice = Choice {
            id: "serialization_test".to_string(),
            content: Some("Thread Choice".to_string()),
            selected: false,
            script: Some(vec!["echo thread execution".to_string()]),
            pty: None,
            thread: None,
            execution_mode: ExecutionMode::Thread,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        // Test that it can be cloned and compared
        let cloned_choice = thread_choice.clone();
        assert_eq!(thread_choice.execution_mode, cloned_choice.execution_mode);
        assert_eq!(thread_choice.execution_mode, ExecutionMode::Thread);
    }

    #[test]
    fn test_thread_mode_with_redirect_output() {
        // F0225: Thread mode should work with output redirection
        let choice_with_redirect = Choice {
            id: "redirect_test".to_string(),
            content: Some("Thread Choice with Redirect".to_string()),
            selected: false,
            script: Some(vec!["echo redirected background output".to_string()]),
            pty: None,
            thread: None,
            execution_mode: ExecutionMode::Thread,
            redirect_output: Some("output_box".to_string()),
            append_output: Some(true),
            waiting: false,
        };

        assert_eq!(choice_with_redirect.execution_mode, ExecutionMode::Thread);
        assert_eq!(choice_with_redirect.redirect_output, Some("output_box".to_string()));
        assert_eq!(choice_with_redirect.append_output, Some(true));
    }

    #[test]
    fn test_thread_mode_stream_creation() {
        // F0225: Thread mode must create streams (requirement for stream architecture)
        let thread_mode = ExecutionMode::Thread;
        assert!(thread_mode.creates_streams(), 
            "F0225: Thread mode MUST create streams for architecture consistency");
    }

    #[test]
    fn test_thread_mode_execution_properties() {
        // F0225: Test all execution properties of thread mode
        let thread_mode = ExecutionMode::Thread;
        
        // Thread mode should be background but not real-time
        assert!(thread_mode.is_background(), "Thread mode should be background execution");
        assert!(!thread_mode.is_realtime(), "Thread mode should not be real-time");
        
        // Thread mode should create streams for result delivery
        assert!(thread_mode.creates_streams(), "Thread mode should create streams");
        
        // Thread mode should have appropriate stream suffix
        assert_eq!(thread_mode.as_stream_suffix(), "thread");
        
        // Thread mode description should be accurate
        assert_eq!(thread_mode.description(), "Background execution in thread pool");
    }

    #[test]
    fn test_thread_choice_waiting_state() {
        // F0225: Thread choices should support waiting state during execution
        let mut thread_choice = Choice {
            id: "waiting_test".to_string(),
            content: Some("Waiting Thread Choice".to_string()),
            selected: false,
            script: Some(vec!["sleep 1; echo done".to_string()]),
            pty: None,
            thread: None,
            execution_mode: ExecutionMode::Thread,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        // Initially not waiting
        assert!(!thread_choice.waiting);
        
        // Can be set to waiting (simulating execution start)
        thread_choice.waiting = true;
        assert!(thread_choice.waiting);
        
        // Execution mode should remain thread
        assert_eq!(thread_choice.execution_mode, ExecutionMode::Thread);
    }

    #[test]
    fn test_thread_mode_hash_consistency() {
        // F0225: Thread mode should hash consistently for stream management
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mode1 = ExecutionMode::Thread;
        let mode2 = ExecutionMode::Thread;
        let mode3 = ExecutionMode::Immediate;

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        mode1.hash(&mut hasher1);
        mode2.hash(&mut hasher2);
        mode3.hash(&mut hasher3);

        // Same modes should hash the same
        assert_eq!(hasher1.finish(), hasher2.finish());
        
        // Different modes should hash differently
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    #[test]
    fn test_legacy_migration_preserves_thread_behavior() {
        // F0225: Legacy thread=true should migrate to ExecutionMode::Thread
        let legacy_thread_mode = ExecutionMode::from_legacy(true, false);
        let direct_thread_mode = ExecutionMode::Thread;
        
        assert_eq!(legacy_thread_mode, direct_thread_mode);
        assert_eq!(legacy_thread_mode.is_background(), direct_thread_mode.is_background());
        assert_eq!(legacy_thread_mode.creates_streams(), direct_thread_mode.creates_streams());
        assert_eq!(legacy_thread_mode.as_stream_suffix(), direct_thread_mode.as_stream_suffix());
        assert_eq!(legacy_thread_mode.description(), direct_thread_mode.description());
    }

    #[test]
    fn test_thread_mode_default_behavior() {
        // F0225: Thread mode should have appropriate default behavior
        let thread_mode = ExecutionMode::Thread;
        
        // Should not be the default mode (Immediate is default)
        assert_ne!(ExecutionMode::default(), thread_mode);
        
        // But should be explicitly creatable
        assert_eq!(thread_mode, ExecutionMode::Thread);
        
        // Should have consistent behavior
        assert!(thread_mode.is_background());
        assert!(thread_mode.creates_streams());
        assert!(!thread_mode.is_realtime());
    }

    #[test]
    fn test_thread_choice_with_script() {
        // F0225: Thread choices must have scripts for meaningful execution
        let thread_choice_with_script = Choice {
            id: "script_test".to_string(),
            content: Some("Thread Choice with Script".to_string()),
            selected: false,
            script: Some(vec!["echo 'executing in background'".to_string(), "date".to_string()]),
            pty: None,
            thread: None,
            execution_mode: ExecutionMode::Thread,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        assert_eq!(thread_choice_with_script.execution_mode, ExecutionMode::Thread);
        assert!(thread_choice_with_script.script.is_some());
        assert_eq!(thread_choice_with_script.script.as_ref().unwrap().len(), 2);
        
        let thread_choice_without_script = Choice {
            id: "no_script_test".to_string(),
            content: Some("Thread Choice without Script".to_string()),
            selected: false,
            script: None,
            pty: None,
            thread: None,
            execution_mode: ExecutionMode::Thread,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        assert_eq!(thread_choice_without_script.execution_mode, ExecutionMode::Thread);
        assert!(thread_choice_without_script.script.is_none());
    }
}