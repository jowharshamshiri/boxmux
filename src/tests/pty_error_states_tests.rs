use crate::circular_buffer::CircularBuffer;
use crate::pty_manager::{PtyManager, PtyStatus};
use std::sync::{Arc, Mutex};

/// F0135: PTY Error States Tests
/// Comprehensive test suite for PTY error state detection and visual indication

#[cfg(test)]
mod pty_error_states_tests {
    use super::*;

    #[test]
    fn test_is_pty_in_error_state_with_error_status() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with error status
        pty_manager.add_test_pty_process_with_status(
            "error_muxbox".to_string(),
            buffer,
            PtyStatus::Error("Connection failed".to_string()),
            12345,
        );

        assert!(pty_manager.is_pty_in_error_state("error_muxbox"));
        assert!(!pty_manager.is_pty_dead("error_muxbox"));
    }

    #[test]
    fn test_is_pty_in_error_state_with_dead_status() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with dead status
        pty_manager.add_test_pty_process_with_status(
            "dead_muxbox".to_string(),
            buffer,
            PtyStatus::Dead("Process crashed".to_string()),
            12345,
        );

        assert!(pty_manager.is_pty_in_error_state("dead_muxbox"));
        assert!(pty_manager.is_pty_dead("dead_muxbox"));
    }

    #[test]
    fn test_is_pty_in_error_state_with_fallback_status() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with fallback status
        pty_manager.add_test_pty_process_with_status(
            "fallback_muxbox".to_string(),
            buffer,
            PtyStatus::FailedFallback,
            12345,
        );

        assert!(pty_manager.is_pty_in_error_state("fallback_muxbox"));
        assert!(!pty_manager.is_pty_dead("fallback_muxbox"));
    }

    #[test]
    fn test_is_pty_in_error_state_with_normal_status() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with normal running status
        pty_manager.add_test_pty_process_with_status(
            "normal_muxbox".to_string(),
            buffer,
            PtyStatus::Running,
            12345,
        );

        assert!(!pty_manager.is_pty_in_error_state("normal_muxbox"));
        assert!(!pty_manager.is_pty_dead("normal_muxbox"));
    }

    #[test]
    fn test_get_error_state_info_for_error_status() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with error status
        pty_manager.add_test_pty_process_with_status(
            "error_muxbox".to_string(),
            buffer,
            PtyStatus::Error("Network timeout".to_string()),
            12345,
        );

        let error_info = pty_manager.get_error_state_info("error_muxbox");
        assert!(error_info.is_some());

        let info = error_info.unwrap();
        assert_eq!(info.muxbox_id, "error_muxbox");
        assert_eq!(
            info.error_type,
            crate::pty_manager::ErrorType::ExecutionError
        );
        assert_eq!(info.message, "Network timeout");
        assert_eq!(info.pid, Some(12345));
        assert!(info.can_retry);
        assert_eq!(info.suggested_action, "Retry PTY execution");
    }

    #[test]
    fn test_get_error_state_info_for_dead_status() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with dead status
        pty_manager.add_test_pty_process_with_status(
            "dead_muxbox".to_string(),
            buffer,
            PtyStatus::Dead("Segmentation fault".to_string()),
            12345,
        );

        let error_info = pty_manager.get_error_state_info("dead_muxbox");
        assert!(error_info.is_some());

        let info = error_info.unwrap();
        assert_eq!(info.muxbox_id, "dead_muxbox");
        assert_eq!(info.error_type, crate::pty_manager::ErrorType::ProcessDied);
        assert_eq!(info.message, "Segmentation fault");
        assert_eq!(info.pid, Some(12345));
        assert!(info.can_retry);
        assert_eq!(info.suggested_action, "Restart PTY process");
    }

    #[test]
    fn test_get_error_state_info_for_fallback_status() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with fallback status
        pty_manager.add_test_pty_process_with_status(
            "fallback_muxbox".to_string(),
            buffer,
            PtyStatus::FailedFallback,
            12345,
        );

        let error_info = pty_manager.get_error_state_info("fallback_muxbox");
        assert!(error_info.is_some());

        let info = error_info.unwrap();
        assert_eq!(info.muxbox_id, "fallback_muxbox");
        assert_eq!(info.error_type, crate::pty_manager::ErrorType::FallbackUsed);
        assert_eq!(info.message, "PTY failed, using regular execution");
        assert_eq!(info.pid, Some(12345));
        assert!(info.can_retry);
        assert_eq!(info.suggested_action, "Reset PTY and retry");
    }

    #[test]
    fn test_get_error_state_info_for_normal_status() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with normal status
        pty_manager.add_test_pty_process_with_status(
            "normal_muxbox".to_string(),
            buffer,
            PtyStatus::Running,
            12345,
        );

        let error_info = pty_manager.get_error_state_info("normal_muxbox");
        assert!(error_info.is_none());
    }

    #[test]
    fn test_mark_pty_dead() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add normal PTY process
        pty_manager.add_test_pty_process("test_muxbox".to_string(), buffer.clone());

        // Initially should not be dead
        assert!(!pty_manager.is_pty_dead("test_muxbox"));

        // Mark as dead
        let result = pty_manager.mark_pty_dead("test_muxbox", "Process killed".to_string());
        assert!(result.is_ok());

        // Should now be dead
        assert!(pty_manager.is_pty_dead("test_muxbox"));
        assert!(pty_manager.is_pty_in_error_state("test_muxbox"));

        // Check status summary includes dead reason
        let summary = pty_manager.get_process_status_summary("test_muxbox");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Dead:Proce");
    }

    #[test]
    fn test_mark_pty_dead_non_existent() {
        let pty_manager = PtyManager::new().unwrap();

        // Try to mark non-existent PTY as dead
        let result = pty_manager.mark_pty_dead("non_existent", "Test reason".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_dead_status_summary_truncation() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with long dead reason
        pty_manager.add_test_pty_process_with_status(
            "dead_muxbox".to_string(),
            buffer,
            PtyStatus::Dead("Very long error message that should be truncated".to_string()),
            12345,
        );

        let summary = pty_manager.get_process_status_summary("dead_muxbox");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Dead:Very ");
    }

    #[test]
    fn test_dead_status_summary_short() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with short dead reason
        pty_manager.add_test_pty_process_with_status(
            "dead_muxbox".to_string(),
            buffer,
            PtyStatus::Dead("SIGKILL".to_string()),
            12345,
        );

        let summary = pty_manager.get_process_status_summary("dead_muxbox");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Dead:SIGKILL");
    }

    #[test]
    fn test_error_states_for_non_existent_muxbox() {
        let pty_manager = PtyManager::new().unwrap();

        // Test all error state methods with non-existent muxbox
        assert!(!pty_manager.is_pty_in_error_state("non_existent"));
        assert!(!pty_manager.is_pty_dead("non_existent"));
        assert!(pty_manager.get_error_state_info("non_existent").is_none());
    }

    #[test]
    fn test_finished_status_not_error_state() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with finished status (both success and failure)
        pty_manager.add_test_pty_process_with_status(
            "success_muxbox".to_string(),
            buffer.clone(),
            PtyStatus::Finished(0),
            12345,
        );

        pty_manager.add_test_pty_process_with_status(
            "failure_muxbox".to_string(),
            buffer.clone(),
            PtyStatus::Finished(1),
            12346,
        );

        // Finished processes should not be considered error states
        assert!(!pty_manager.is_pty_in_error_state("success_muxbox"));
        assert!(!pty_manager.is_pty_in_error_state("failure_muxbox"));
        assert!(!pty_manager.is_pty_dead("success_muxbox"));
        assert!(!pty_manager.is_pty_dead("failure_muxbox"));
    }
}
