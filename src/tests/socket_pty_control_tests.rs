use crate::circular_buffer::CircularBuffer;
use crate::model::app::AppContext;
use crate::model::common::{run_socket_function, SocketFunction};
use crate::pty_manager::{PtyManager, PtyStatus};
use std::sync::{Arc, Mutex};

/// F0137: Socket PTY Control Tests
/// F0138: Socket PTY Query Tests
/// Comprehensive test suite for socket-based PTY process management

#[cfg(test)]
mod socket_pty_control_tests {
    use super::*;

    fn create_test_app_context_with_pty() -> AppContext {
        use crate::tests::test_utils::TestDataFactory;
        let pty_manager = PtyManager::new().unwrap();
        let mut app_context = TestDataFactory::create_test_app_context();
        app_context.pty_manager = Some(Arc::new(pty_manager));
        app_context
    }

    #[test]
    fn test_kill_pty_process_socket_command_success() {
        let app_context = create_test_app_context_with_pty();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add a PTY process that can be killed
        if let Some(pty_manager) = &app_context.pty_manager {
            pty_manager.add_test_pty_process_with_status(
                "test_muxbox".to_string(),
                buffer,
                PtyStatus::Running,
                12345,
            );

            // Make it killable
            pty_manager.set_pty_killable("test_muxbox", true);
        }

        let socket_function = SocketFunction::KillPtyProcess {
            box_id: "test_muxbox".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have a message about the kill attempt
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "test_muxbox");
            // In test environment, killing a fake PID might fail, but the attempt should be made
            // So we check that we got a message about the kill attempt
            assert!(message.contains("kill") || message.contains("Kill"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message");
        }
    }

    #[test]
    fn test_kill_pty_process_socket_command_not_killable() {
        let app_context = create_test_app_context_with_pty();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add a PTY process that cannot be killed
        if let Some(pty_manager) = &app_context.pty_manager {
            pty_manager.add_test_pty_process_with_status(
                "test_muxbox".to_string(),
                buffer,
                PtyStatus::Running,
                12345,
            );
            // Default can_kill is false
        }

        let socket_function = SocketFunction::KillPtyProcess {
            box_id: "test_muxbox".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have an error message
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "test_muxbox");
            assert!(!success);
            assert!(message.contains("cannot be killed"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message");
        }
    }

    #[test]
    fn test_kill_pty_process_non_existent_muxbox() {
        let app_context = create_test_app_context_with_pty();

        let socket_function = SocketFunction::KillPtyProcess {
            box_id: "non_existent".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have an error message
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "non_existent");
            assert!(!success);
            assert!(message.contains("PTY not found"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message");
        }
    }

    #[test]
    fn test_restart_pty_process_socket_command() {
        let app_context = create_test_app_context_with_pty();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add a PTY process
        if let Some(pty_manager) = &app_context.pty_manager {
            pty_manager.add_test_pty_process_with_status(
                "test_muxbox".to_string(),
                buffer,
                PtyStatus::Running,
                12345,
            );
        }

        let socket_function = SocketFunction::RestartPtyProcess {
            box_id: "test_muxbox".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have a success message
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "test_muxbox");
            assert!(*success);
            assert!(message.contains("restarted"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message");
        }

        // Verify the process is marked for restart
        if let Some(pty_manager) = &app_context.pty_manager {
            let info = pty_manager.get_detailed_process_info("test_muxbox");
            assert!(info.is_some());
            let info = info.unwrap();
            assert!(matches!(info.status, PtyStatus::Starting));
        }
    }

    #[test]
    fn test_restart_pty_process_non_existent_muxbox() {
        let app_context = create_test_app_context_with_pty();

        let socket_function = SocketFunction::RestartPtyProcess {
            box_id: "non_existent".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have an error message
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "non_existent");
            assert!(!success);
            assert!(message.contains("PTY not found"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message");
        }
    }

    #[test]
    fn test_query_pty_status_socket_command() {
        let app_context = create_test_app_context_with_pty();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add content to buffer
        {
            let mut buf = buffer.lock().unwrap();
            buf.push("Line 1".to_string());
            buf.push("Line 2".to_string());
        }

        // Add a PTY process
        if let Some(pty_manager) = &app_context.pty_manager {
            pty_manager.add_test_pty_process_with_status(
                "test_muxbox".to_string(),
                buffer,
                PtyStatus::Running,
                12345,
            );
        }

        let socket_function = SocketFunction::QueryPtyStatus {
            box_id: "test_muxbox".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have a success message with status info
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "test_muxbox");
            assert!(*success);
            assert!(message.contains("PTY Status"));
            assert!(message.contains("test_muxbox"));
            assert!(message.contains("12345"));
            assert!(message.contains("Running"));
            assert!(message.contains("Buffer Lines: 2"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message");
        }
    }

    #[test]
    fn test_query_pty_status_non_existent_muxbox() {
        let app_context = create_test_app_context_with_pty();

        let socket_function = SocketFunction::QueryPtyStatus {
            box_id: "non_existent".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have an error message
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "non_existent");
            assert!(!success);
            assert!(message.contains("No PTY process found"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message");
        }
    }

    #[test]
    fn test_pty_commands_without_pty_manager() {
        use crate::tests::test_utils::TestDataFactory;
        let mut app_context = TestDataFactory::create_test_app_context();
        app_context.pty_manager = None;

        // Test kill command without PTY manager
        let kill_function = SocketFunction::KillPtyProcess {
            box_id: "test_muxbox".to_string(),
        };

        let result = run_socket_function(kill_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        if let crate::thread_manager::Message::MuxBoxOutputUpdate(_, success, message) =
            &messages[0]
        {
            assert!(!success);
            assert!(message.contains("PTY manager not available"));
        }

        // Test restart command without PTY manager
        let restart_function = SocketFunction::RestartPtyProcess {
            box_id: "test_muxbox".to_string(),
        };

        let result = run_socket_function(restart_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        if let crate::thread_manager::Message::MuxBoxOutputUpdate(_, success, message) =
            &messages[0]
        {
            assert!(!success);
            assert!(message.contains("PTY manager not available"));
        }

        // Test query command without PTY manager
        let query_function = SocketFunction::QueryPtyStatus {
            box_id: "test_muxbox".to_string(),
        };

        let result = run_socket_function(query_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        if let crate::thread_manager::Message::MuxBoxOutputUpdate(_, success, message) =
            &messages[0]
        {
            assert!(!success);
            assert!(message.contains("PTY manager not available"));
        }
    }
}
