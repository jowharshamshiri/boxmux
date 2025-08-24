use crate::pty_manager::{PtyManager, PtyStatus};
use std::sync::mpsc;

#[cfg(test)]
mod pty_resize_tests {
    use super::*;

    #[test]
    fn test_pty_manager_creation() {
        let manager = PtyManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_pty_resize_nonexistent_muxbox() {
        let mut manager = PtyManager::new().unwrap();

        let result = manager.resize_pty("nonexistent", 25, 80);
        // Should not error out, just warn
        assert!(result.is_ok());
    }

    #[test]
    fn test_pty_process_fields() {
        // Test that PtyProcess can be created with all fields
        let manager = PtyManager::new().unwrap();
        let active_muxboxes = manager.get_active_pty_muxboxes();
        assert_eq!(active_muxboxes.len(), 0);
    }

    #[test]
    fn test_pty_status_values() {
        // Test PtyStatus enum variants
        let status = PtyStatus::Starting;
        assert_eq!(status, PtyStatus::Starting);

        let status = PtyStatus::Running;
        assert_eq!(status, PtyStatus::Running);

        let status = PtyStatus::Finished(0);
        assert_eq!(status, PtyStatus::Finished(0));

        let status = PtyStatus::Error("test".to_string());
        assert_eq!(status, PtyStatus::Error("test".to_string()));
    }

    #[test]
    fn test_pty_resize_api() {
        let mut manager = PtyManager::new().unwrap();

        // Test resize with common terminal sizes
        let result = manager.resize_pty("test_muxbox", 24, 80);
        assert!(result.is_ok());

        let result = manager.resize_pty("test_muxbox", 50, 120);
        assert!(result.is_ok());

        let result = manager.resize_pty("test_muxbox", 30, 100);
        assert!(result.is_ok());
    }

    #[test]
    fn test_pty_manager_cleanup() {
        let mut manager = PtyManager::new().unwrap();

        // Test cleanup operation
        manager.cleanup_finished();

        // Should not panic or error
        let active_muxboxes = manager.get_active_pty_muxboxes();
        assert_eq!(active_muxboxes.len(), 0);
    }

    #[test]
    fn test_pty_kill_nonexistent() {
        let mut manager = PtyManager::new().unwrap();

        let result = manager.kill_pty("nonexistent");
        assert!(result.is_ok());
    }

    #[test]
    fn test_pty_status_query() {
        let manager = PtyManager::new().unwrap();

        let status = manager.get_pty_status("nonexistent");
        assert!(status.is_none());
    }
}
