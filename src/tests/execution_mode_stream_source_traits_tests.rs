// F0227: ExecutionMode Stream Source Traits tests
// Comprehensive test suite for execution-mode-specific source traits and implementations

use crate::model::common::{
    ExecutionMode, ImmediateExecutionSource, ImmediateSource, PtySessionExecutionSource,
    PtySessionSource, StreamSource, StreamSourceTrait, ThreadPoolExecutionSource, ThreadPoolSource,
    ThreadStatus,
};
use std::time::{Duration, SystemTime};

#[cfg(test)]
mod tests {
    use super::*;

    // ===== ImmediateExecutionSource Tests =====

    /// Test that ImmediateExecutionSource implements basic StreamSourceTrait correctly
    #[test]
    fn test_immediate_execution_source_basic_traits() {
        let source = ImmediateExecutionSource {
            choice_id: "test_choice".to_string(),
            muxbox_id: "test_box".to_string(),
            script: vec!["echo test".to_string()],
            started_at: SystemTime::now(),
            completed_at: None,
            execution_result: None,
            execution_duration: None,
        };

        assert_eq!(source.source_type(), "immediate_execution");
        assert_eq!(source.source_id(), "immediate_test_choice");
        assert!(!source.can_terminate()); // Immediate execution cannot be terminated
        assert!(source.cleanup().is_ok());

        let metadata = source.get_metadata();
        assert_eq!(metadata.get("choice_id"), Some(&"test_choice".to_string()));
        assert_eq!(metadata.get("muxbox_id"), Some(&"test_box".to_string()));
        assert_eq!(metadata.get("script_lines"), Some(&"1".to_string()));
        assert_eq!(metadata.get("is_complete"), Some(&"false".to_string()));
    }

    /// Test that ImmediateExecutionSource implements ImmediateSource trait correctly
    #[test]
    fn test_immediate_execution_source_immediate_trait() {
        let mut source = ImmediateExecutionSource {
            choice_id: "test_choice".to_string(),
            muxbox_id: "test_box".to_string(),
            script: vec!["echo test".to_string()],
            started_at: SystemTime::now(),
            completed_at: None,
            execution_result: None,
            execution_duration: None,
        };

        // Test initial state
        assert!(!source.is_complete());
        assert!(source.get_execution_result().is_none());
        assert!(source.get_execution_duration().is_none());

        // Test after setting result
        source.execution_result = Some(Ok("test output".to_string()));
        source.execution_duration = Some(Duration::from_millis(100));
        source.completed_at = Some(SystemTime::now());

        assert!(source.is_complete());
        assert_eq!(
            source.get_execution_result(),
            Some(Ok("test output".to_string()))
        );
        assert_eq!(
            source.get_execution_duration(),
            Some(Duration::from_millis(100))
        );
    }

    // ===== ThreadPoolExecutionSource Tests =====

    /// Test that ThreadPoolExecutionSource implements basic StreamSourceTrait correctly
    #[test]
    fn test_thread_pool_execution_source_basic_traits() {
        let source = ThreadPoolExecutionSource {
            choice_id: "thread_choice".to_string(),
            muxbox_id: "thread_box".to_string(),
            script: vec!["sleep 5".to_string(), "echo done".to_string()],
            thread_id: Some("thread-123".to_string()),
            started_at: SystemTime::now(),
            timeout_seconds: Some(30),
            thread_status: ThreadStatus::Running,
            completion_result: None,
            execution_duration: None,
        };

        assert_eq!(source.source_type(), "thread_pool_execution");
        assert_eq!(source.source_id(), "thread_thread_choice");
        assert!(source.can_terminate()); // Running thread can be terminated
        assert!(source.cleanup().is_ok());

        let metadata = source.get_metadata();
        assert_eq!(
            metadata.get("choice_id"),
            Some(&"thread_choice".to_string())
        );
        assert_eq!(metadata.get("thread_id"), Some(&"thread-123".to_string()));
        assert_eq!(metadata.get("timeout_seconds"), Some(&"30".to_string()));
        assert_eq!(metadata.get("script_lines"), Some(&"2".to_string()));
        assert_eq!(metadata.get("thread_status"), Some(&"Running".to_string()));
    }

    /// Test that ThreadPoolExecutionSource implements ThreadPoolSource trait correctly
    #[test]
    fn test_thread_pool_execution_source_thread_pool_trait() {
        let mut source = ThreadPoolExecutionSource {
            choice_id: "thread_choice".to_string(),
            muxbox_id: "thread_box".to_string(),
            script: vec!["echo test".to_string()],
            thread_id: Some("thread-456".to_string()),
            started_at: SystemTime::now(),
            timeout_seconds: None,
            thread_status: ThreadStatus::Running,
            completion_result: None,
            execution_duration: None,
        };

        // Test thread operations
        assert_eq!(source.get_thread_id(), Some("thread-456".to_string()));
        assert!(source.is_thread_running());
        assert_eq!(source.get_thread_status(), ThreadStatus::Running);
        assert!(source.cancel_thread().is_ok());

        // Test timeout setting
        source.set_timeout(60);
        assert_eq!(source.timeout_seconds, Some(60));

        // Test termination state
        source.thread_status = ThreadStatus::Completed;
        assert!(!source.is_thread_running());
        assert!(!source.can_terminate()); // Completed thread cannot be terminated
    }

    /// Test ThreadStatus transitions
    #[test]
    fn test_thread_status_transitions() {
        let mut source = ThreadPoolExecutionSource {
            choice_id: "status_test".to_string(),
            muxbox_id: "status_box".to_string(),
            script: vec!["echo test".to_string()],
            thread_id: Some("thread-789".to_string()),
            started_at: SystemTime::now(),
            timeout_seconds: None,
            thread_status: ThreadStatus::NotStarted,
            completion_result: None,
            execution_duration: None,
        };

        // Test all status states
        for status in [
            ThreadStatus::NotStarted,
            ThreadStatus::Running,
            ThreadStatus::Completed,
            ThreadStatus::Failed,
            ThreadStatus::Cancelled,
            ThreadStatus::TimedOut,
        ] {
            source.thread_status = status.clone();
            assert_eq!(source.get_thread_status(), status);

            // Only running and not-started can be cancelled
            let can_cancel = matches!(status, ThreadStatus::Running | ThreadStatus::NotStarted);
            assert_eq!(source.cancel_thread().is_ok(), can_cancel);
        }
    }

    // ===== PtySessionExecutionSource Tests =====

    /// Test that PtySessionExecutionSource implements basic StreamSourceTrait correctly
    #[test]
    fn test_pty_session_execution_source_basic_traits() {
        let source = PtySessionExecutionSource {
            choice_id: "pty_choice".to_string(),
            muxbox_id: "pty_box".to_string(),
            command: "bash".to_string(),
            args: vec!["-c".to_string(), "echo hello".to_string()],
            working_dir: Some("/tmp".to_string()),
            process_id: Some(12345),
            terminal_size: (24, 80),
            started_at: SystemTime::now(),
            reader_thread_id: Some("pty-reader-123".to_string()),
            is_process_running: true,
        };

        assert_eq!(source.source_type(), "pty_session_execution");
        assert_eq!(source.source_id(), "pty_pty_choice");
        assert!(source.can_terminate()); // Running PTY process can be terminated

        let metadata = source.get_metadata();
        assert_eq!(metadata.get("choice_id"), Some(&"pty_choice".to_string()));
        assert_eq!(metadata.get("command"), Some(&"bash".to_string()));
        assert_eq!(metadata.get("args"), Some(&"-c echo hello".to_string()));
        assert_eq!(metadata.get("working_dir"), Some(&"/tmp".to_string()));
        assert_eq!(metadata.get("process_id"), Some(&"12345".to_string()));
        assert_eq!(metadata.get("terminal_size"), Some(&"24x80".to_string()));
        assert_eq!(metadata.get("is_running"), Some(&"true".to_string()));
    }

    /// Test that PtySessionExecutionSource implements PtySessionSource trait correctly
    #[test]
    fn test_pty_session_execution_source_pty_trait() {
        let source = PtySessionExecutionSource {
            choice_id: "pty_choice".to_string(),
            muxbox_id: "pty_box".to_string(),
            command: "bash".to_string(),
            args: vec!["-i".to_string()],
            working_dir: Some("/home/user".to_string()),
            process_id: Some(54321),
            terminal_size: (30, 120),
            started_at: SystemTime::now(),
            reader_thread_id: Some("pty-reader-456".to_string()),
            is_process_running: true,
        };

        // Test PTY operations
        assert_eq!(source.get_process_id(), Some(54321));
        assert!(source.is_process_running());
        assert_eq!(source.get_terminal_size(), (30, 120));
        assert_eq!(source.get_command(), "bash -i");
        assert_eq!(
            source.get_working_directory(),
            Some("/home/user".to_string())
        );

        // Test process operations (should succeed for running process)
        assert!(source.send_input("echo test\n").is_ok());
        assert!(source.resize_terminal(25, 100).is_ok());

        // Note: kill_process() test would be platform-specific and potentially disruptive
    }

    /// Test PTY command formatting with different argument scenarios
    #[test]
    fn test_pty_command_formatting() {
        // Command with no args
        let source1 = PtySessionExecutionSource {
            choice_id: "cmd1".to_string(),
            muxbox_id: "box1".to_string(),
            command: "vim".to_string(),
            args: vec![],
            working_dir: None,
            process_id: None,
            terminal_size: (24, 80),
            started_at: SystemTime::now(),
            reader_thread_id: None,
            is_process_running: false,
        };
        assert_eq!(source1.get_command(), "vim");

        // Command with single arg
        let source2 = PtySessionExecutionSource {
            choice_id: "cmd2".to_string(),
            muxbox_id: "box2".to_string(),
            command: "ls".to_string(),
            args: vec!["-la".to_string()],
            working_dir: None,
            process_id: None,
            terminal_size: (24, 80),
            started_at: SystemTime::now(),
            reader_thread_id: None,
            is_process_running: false,
        };
        assert_eq!(source2.get_command(), "ls -la");

        // Command with multiple args
        let source3 = PtySessionExecutionSource {
            choice_id: "cmd3".to_string(),
            muxbox_id: "box3".to_string(),
            command: "git".to_string(),
            args: vec![
                "commit".to_string(),
                "-m".to_string(),
                "test message".to_string(),
            ],
            working_dir: None,
            process_id: None,
            terminal_size: (24, 80),
            started_at: SystemTime::now(),
            reader_thread_id: None,
            is_process_running: false,
        };
        assert_eq!(source3.get_command(), "git commit -m test message");
    }

    // ===== StreamSource Factory Tests =====

    /// Test StreamSource factory functions create correct source types
    #[test]
    fn test_stream_source_factory_functions() {
        // Test immediate execution source creation
        let immediate = StreamSource::create_immediate_execution_source(
            "imm_choice".to_string(),
            "imm_box".to_string(),
            vec!["echo immediate".to_string()],
        );
        assert!(immediate.supports_immediate_source());
        assert!(!immediate.supports_thread_pool_source());
        assert!(!immediate.supports_pty_session_source());

        // Test thread pool execution source creation
        let thread = StreamSource::create_thread_pool_execution_source(
            "thread_choice".to_string(),
            "thread_box".to_string(),
            vec!["sleep 1".to_string()],
            Some("thread-123".to_string()),
            Some(30),
        );
        assert!(!thread.supports_immediate_source());
        assert!(thread.supports_thread_pool_source());
        assert!(!thread.supports_pty_session_source());

        // Test PTY session execution source creation
        let pty = StreamSource::create_pty_session_execution_source(
            "pty_choice".to_string(),
            "pty_box".to_string(),
            "bash".to_string(),
            vec!["-i".to_string()],
            Some("/tmp".to_string()),
            (24, 80),
        );
        assert!(!pty.supports_immediate_source());
        assert!(!pty.supports_thread_pool_source());
        assert!(pty.supports_pty_session_source());
    }

    /// Test ExecutionMode to StreamSource conversion
    #[test]
    fn test_execution_mode_to_stream_source_conversion() {
        let script = vec!["echo test".to_string()];

        // Test Immediate mode conversion
        let immediate_source = StreamSource::from_execution_mode(
            &ExecutionMode::Immediate,
            "test_choice".to_string(),
            "test_box".to_string(),
            script.clone(),
            None,
        );
        assert!(immediate_source.supports_immediate_source());

        // Test Thread mode conversion
        let mut thread_params = std::collections::HashMap::new();
        thread_params.insert("thread_id".to_string(), "thread-456".to_string());
        thread_params.insert("timeout_seconds".to_string(), "60".to_string());

        let thread_source = StreamSource::from_execution_mode(
            &ExecutionMode::Thread,
            "thread_choice".to_string(),
            "thread_box".to_string(),
            script.clone(),
            Some(thread_params),
        );
        assert!(thread_source.supports_thread_pool_source());

        // Test PTY mode conversion
        let mut pty_params = std::collections::HashMap::new();
        pty_params.insert("working_dir".to_string(), "/home/user".to_string());
        pty_params.insert("terminal_rows".to_string(), "30".to_string());
        pty_params.insert("terminal_cols".to_string(), "120".to_string());

        let pty_source = StreamSource::from_execution_mode(
            &ExecutionMode::Pty,
            "pty_choice".to_string(),
            "pty_box".to_string(),
            vec!["bash".to_string(), "-i".to_string()],
            Some(pty_params),
        );
        assert!(pty_source.supports_pty_session_source());
    }

    /// Test trait object access for execution-mode-specific sources
    #[test]
    fn test_trait_object_access() {
        // Create different source types
        let immediate = StreamSource::create_immediate_execution_source(
            "imm".to_string(),
            "box".to_string(),
            vec!["echo test".to_string()],
        );
        let thread = StreamSource::create_thread_pool_execution_source(
            "thread".to_string(),
            "box".to_string(),
            vec!["sleep 1".to_string()],
            None,
            None,
        );
        let pty = StreamSource::create_pty_session_execution_source(
            "pty".to_string(),
            "box".to_string(),
            "bash".to_string(),
            vec![],
            None,
            (24, 80),
        );

        // Test trait object access
        assert!(immediate.as_immediate_source().is_some());
        assert!(immediate.as_thread_pool_source().is_none());
        assert!(immediate.as_pty_session_source().is_none());

        assert!(thread.as_immediate_source().is_none());
        assert!(thread.as_thread_pool_source().is_some());
        assert!(thread.as_pty_session_source().is_none());

        assert!(pty.as_immediate_source().is_none());
        assert!(pty.as_thread_pool_source().is_none());
        assert!(pty.as_pty_session_source().is_some());
    }

    /// Test enhanced metadata for execution-mode-specific sources
    #[test]
    fn test_enhanced_metadata() {
        // Test immediate execution metadata with completion
        let mut immediate = match StreamSource::create_immediate_execution_source(
            "meta_test".to_string(),
            "meta_box".to_string(),
            vec!["echo metadata".to_string()],
        ) {
            StreamSource::ImmediateExecution(mut source) => {
                source.execution_result = Some(Ok("output".to_string()));
                source.execution_duration = Some(Duration::from_millis(150));
                source.completed_at = Some(SystemTime::now());
                StreamSource::ImmediateExecution(source)
            }
            _ => panic!("Expected ImmediateExecution source"),
        };

        let metadata = immediate.get_metadata();
        assert_eq!(metadata.get("is_complete"), Some(&"true".to_string()));
        assert_eq!(metadata.get("duration_ms"), Some(&"150".to_string()));

        // Test thread execution metadata
        let thread = StreamSource::create_thread_pool_execution_source(
            "thread_meta".to_string(),
            "thread_meta_box".to_string(),
            vec![
                "long".to_string(),
                "running".to_string(),
                "script".to_string(),
            ],
            Some("thread-meta-123".to_string()),
            Some(120),
        );

        let thread_metadata = thread.get_metadata();
        assert_eq!(thread_metadata.get("script_lines"), Some(&"3".to_string()));
        assert_eq!(
            thread_metadata.get("timeout_seconds"),
            Some(&"120".to_string())
        );
        assert_eq!(
            thread_metadata.get("thread_id"),
            Some(&"thread-meta-123".to_string())
        );

        // Test PTY execution metadata
        let pty = StreamSource::create_pty_session_execution_source(
            "pty_meta".to_string(),
            "pty_meta_box".to_string(),
            "complex_command".to_string(),
            vec!["--arg1".to_string(), "--arg2=value".to_string()],
            Some("/project/root".to_string()),
            (40, 160),
        );

        let pty_metadata = pty.get_metadata();
        assert_eq!(
            pty_metadata.get("command"),
            Some(&"complex_command".to_string())
        );
        assert_eq!(
            pty_metadata.get("args"),
            Some(&"--arg1 --arg2=value".to_string())
        );
        assert_eq!(
            pty_metadata.get("working_dir"),
            Some(&"/project/root".to_string())
        );
        assert_eq!(
            pty_metadata.get("terminal_size"),
            Some(&"40x160".to_string())
        );
    }

    /// Test lifecycle management differences between execution modes
    #[test]
    fn test_execution_mode_lifecycle_differences() {
        // Immediate execution - cannot be terminated
        let immediate = StreamSource::create_immediate_execution_source(
            "lifecycle_imm".to_string(),
            "box".to_string(),
            vec!["echo test".to_string()],
        );
        assert!(!immediate.can_terminate());
        assert!(immediate.cleanup().is_ok());

        // Thread execution - can be terminated when running
        let thread = StreamSource::create_thread_pool_execution_source(
            "lifecycle_thread".to_string(),
            "box".to_string(),
            vec!["long_script".to_string()],
            Some("thread-lifecycle".to_string()),
            None,
        );
        assert!(thread.can_terminate()); // NotStarted status allows termination
        assert!(thread.cleanup().is_ok());

        // PTY execution - can be terminated when process is running
        let mut pty = match StreamSource::create_pty_session_execution_source(
            "lifecycle_pty".to_string(),
            "box".to_string(),
            "bash".to_string(),
            vec![],
            None,
            (24, 80),
        ) {
            StreamSource::PtySessionExecution(mut source) => {
                source.process_id = Some(12345);
                source.is_process_running = true;
                StreamSource::PtySessionExecution(source)
            }
            _ => panic!("Expected PtySessionExecution source"),
        };
        assert!(pty.can_terminate());

        // Test PTY with terminated process
        if let StreamSource::PtySessionExecution(ref mut source) = pty {
            source.is_process_running = false;
        }
        assert!(!pty.can_terminate());
    }

    /// Test error handling in source operations
    #[test]
    fn test_source_error_handling() {
        // Test PTY operations on non-running process
        let pty_not_running = PtySessionExecutionSource {
            choice_id: "error_test".to_string(),
            muxbox_id: "error_box".to_string(),
            command: "bash".to_string(),
            args: vec![],
            working_dir: None,
            process_id: None,
            terminal_size: (24, 80),
            started_at: SystemTime::now(),
            reader_thread_id: None,
            is_process_running: false,
        };

        assert!(pty_not_running.send_input("test\n").is_err());
        assert!(pty_not_running.resize_terminal(25, 90).is_err());
        assert!(pty_not_running.kill_process().is_err());

        // Test thread cancellation on completed thread
        let completed_thread = ThreadPoolExecutionSource {
            choice_id: "completed_thread".to_string(),
            muxbox_id: "error_box".to_string(),
            script: vec!["echo done".to_string()],
            thread_id: Some("completed-thread-123".to_string()),
            started_at: SystemTime::now(),
            timeout_seconds: None,
            thread_status: ThreadStatus::Completed,
            completion_result: Some(Ok("done".to_string())),
            execution_duration: Some(Duration::from_millis(100)),
        };

        assert!(completed_thread.cancel_thread().is_err());
    }
}
