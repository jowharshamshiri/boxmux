use crate::circular_buffer::CircularBuffer;
use crate::pty_manager::{PtyManager, PtyStatus};
use std::sync::{Arc, Mutex};

/// F0132: PTY Process Info Tests
/// Comprehensive test suite for PTY process information display functionality

#[cfg(test)]
mod pty_process_info_tests {
    use super::*;

    #[test]
    fn test_get_detailed_process_info() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add test PTY process
        pty_manager.add_test_pty_process("test_panel".to_string(), buffer.clone());

        // Test detailed process info retrieval
        let info = pty_manager.get_detailed_process_info("test_panel");
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.panel_id, "test_panel");
        assert_eq!(info.process_id, Some(12345));
        assert!(matches!(info.status, PtyStatus::Running));
        assert_eq!(info.can_kill, false);
        assert_eq!(info.buffer_lines, 0);
        assert_eq!(info.is_running, true);
    }

    #[test]
    fn test_get_process_status_summary() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add test PTY process
        pty_manager.add_test_pty_process("test_panel".to_string(), buffer.clone());

        // Test status summary
        let summary = pty_manager.get_process_status_summary("test_panel");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Running");
    }

    #[test]
    fn test_process_status_summary_finished_success() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with finished status (success)
        pty_manager.add_test_pty_process_with_status(
            "test_panel".to_string(),
            buffer,
            PtyStatus::Finished(0),
            12345,
        );

        let summary = pty_manager.get_process_status_summary("test_panel");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Done");
    }

    #[test]
    fn test_process_status_summary_finished_failure() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with finished status (failure)
        pty_manager.add_test_pty_process_with_status(
            "test_panel".to_string(),
            buffer,
            PtyStatus::Finished(1),
            12345,
        );

        let summary = pty_manager.get_process_status_summary("test_panel");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Exit:1");
    }

    #[test]
    fn test_process_status_summary_error_short() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with short error message
        pty_manager.add_test_pty_process_with_status(
            "test_panel".to_string(),
            buffer,
            PtyStatus::Error("Failed".to_string()),
            12345,
        );

        let summary = pty_manager.get_process_status_summary("test_panel");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Error:Failed");
    }

    #[test]
    fn test_process_status_summary_error_long() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with long error message (should be truncated)
        pty_manager.add_test_pty_process_with_status(
            "test_panel".to_string(),
            buffer,
            PtyStatus::Error("This is a very long error message".to_string()),
            12345,
        );

        let summary = pty_manager.get_process_status_summary("test_panel");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Error:This is");
    }

    #[test]
    fn test_process_status_summary_fallback() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with fallback status
        pty_manager.add_test_pty_process_with_status(
            "test_panel".to_string(),
            buffer,
            PtyStatus::FailedFallback,
            12345,
        );

        let summary = pty_manager.get_process_status_summary("test_panel");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Fallback");
    }

    #[test]
    fn test_process_status_summary_starting() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process with starting status
        pty_manager.add_test_pty_process_with_status(
            "test_panel".to_string(),
            buffer,
            PtyStatus::Starting,
            12345,
        );

        let summary = pty_manager.get_process_status_summary("test_panel");
        assert!(summary.is_some());
        assert_eq!(summary.unwrap(), "PID:12345 Starting");
    }

    #[test]
    fn test_detailed_process_info_with_buffer_content() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add content to buffer
        {
            let mut buf = buffer.lock().unwrap();
            buf.push("Line 1".to_string());
            buf.push("Line 2".to_string());
            buf.push("Line 3".to_string());
        }

        // Add test PTY process
        pty_manager.add_test_pty_process("test_panel".to_string(), buffer.clone());

        let info = pty_manager.get_detailed_process_info("test_panel");
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.buffer_lines, 3);
    }

    #[test]
    fn test_process_info_non_existent_panel() {
        let pty_manager = PtyManager::new().unwrap();

        // Test non-existent panel
        let info = pty_manager.get_detailed_process_info("non_existent");
        assert!(info.is_none());

        let summary = pty_manager.get_process_status_summary("non_existent");
        assert!(summary.is_none());
    }

    #[test]
    fn test_process_info_is_running_logic() {
        let pty_manager = PtyManager::new().unwrap();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Test Running status
        pty_manager.add_test_pty_process_with_status(
            "running_panel".to_string(),
            buffer.clone(),
            PtyStatus::Running,
            12345,
        );
        let info = pty_manager
            .get_detailed_process_info("running_panel")
            .unwrap();
        assert_eq!(info.is_running, true);

        // Test Starting status
        pty_manager.add_test_pty_process_with_status(
            "starting_panel".to_string(),
            buffer.clone(),
            PtyStatus::Starting,
            12346,
        );
        let info = pty_manager
            .get_detailed_process_info("starting_panel")
            .unwrap();
        assert_eq!(info.is_running, true);

        // Test Finished status
        pty_manager.add_test_pty_process_with_status(
            "finished_panel".to_string(),
            buffer.clone(),
            PtyStatus::Finished(0),
            12347,
        );
        let info = pty_manager
            .get_detailed_process_info("finished_panel")
            .unwrap();
        assert_eq!(info.is_running, false);
    }
}
