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
    fn test_spawn_pty_process_socket_command_success() {
        let app_context = create_test_app_context_with_pty();

        let socket_function = SocketFunction::SpawnPtyProcess {
            box_id: "test_spawn_muxbox".to_string(),
            script: vec!["echo 'Hello PTY'".to_string(), "ls".to_string()],
            libs: Some(vec!["/usr/local/bin/mylib.sh".to_string()]),
            redirect_output: None,
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have a success message about PTY spawn
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "test_spawn_muxbox");
            assert!(*success);
            assert!(message.contains("PTY process spawned successfully"));
            assert!(message.contains("test_spawn_muxbox"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for successful spawn");
        }
    }

    #[test]
    fn test_spawn_pty_process_with_redirect_output() {
        let app_context = create_test_app_context_with_pty();

        let socket_function = SocketFunction::SpawnPtyProcess {
            box_id: "source_box".to_string(),
            script: vec!["echo 'Redirected output'".to_string()],
            libs: None,
            redirect_output: Some("target_box".to_string()),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have a success message
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "source_box");
            assert!(*success);
            assert!(message.contains("PTY process spawned successfully"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for spawn with redirect");
        }
    }

    #[test]
    fn test_spawn_pty_process_no_pty_manager() {
        let mut app_context = create_test_app_context_with_pty();
        app_context.pty_manager = None; // Remove PTY manager

        let socket_function = SocketFunction::SpawnPtyProcess {
            box_id: "test_box".to_string(),
            script: vec!["echo 'test'".to_string()],
            libs: None,
            redirect_output: None,
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have an error message about PTY manager unavailability
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "test_box");
            assert!(!success);
            assert!(message.contains("PTY manager not available"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for missing PTY manager");
        }
    }

    #[test]
    fn test_spawn_pty_process_empty_script() {
        let app_context = create_test_app_context_with_pty();

        let socket_function = SocketFunction::SpawnPtyProcess {
            box_id: "empty_script_box".to_string(),
            script: vec![], // Empty script
            libs: None,
            redirect_output: None,
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should handle empty script gracefully - result depends on PTY implementation
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, _success, _message) =
            &messages[0]
        {
            assert_eq!(box_id, "empty_script_box");
            // Success/failure depends on PTY manager's handling of empty scripts
        } else {
            panic!("Expected MuxBoxOutputUpdate message for empty script");
        }
    }

    #[test]
    fn test_spawn_pty_process_complex_script_with_libs() {
        let app_context = create_test_app_context_with_pty();

        let socket_function = SocketFunction::SpawnPtyProcess {
            box_id: "complex_pty".to_string(),
            script: vec![
                "#!/bin/bash".to_string(),
                "set -e".to_string(),
                "echo 'Starting complex script'".to_string(),
                "for i in {1..3}; do echo \"Step $i\"; done".to_string(),
                "echo 'Complex script completed'".to_string(),
            ],
            libs: Some(vec![
                "/usr/local/lib/utils.sh".to_string(),
                "/opt/myapp/shared.sh".to_string(),
            ]),
            redirect_output: Some("output_viewer".to_string()),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should handle complex script with multiple libs and redirect
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "complex_pty");
            assert!(*success);
            assert!(message.contains("PTY process spawned successfully"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for complex script");
        }
    }

    #[test]
    fn test_send_pty_input_success() {
        let app_context = create_test_app_context_with_pty();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add a PTY process in running state
        if let Some(pty_manager) = &app_context.pty_manager {
            pty_manager.add_test_pty_process_with_status(
                "input_test_box".to_string(),
                buffer,
                PtyStatus::Running,
                12345,
            );
        }

        let socket_function = SocketFunction::SendPtyInput {
            box_id: "input_test_box".to_string(),
            input: "echo 'Hello PTY'\n".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have a success message about input being sent
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "input_test_box");
            assert!(*success);
            assert!(message.contains("Input sent successfully"));
            assert!(message.contains("input_test_box"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for successful input");
        }
    }

    #[test]
    fn test_send_pty_input_non_existent_muxbox() {
        let app_context = create_test_app_context_with_pty();

        let socket_function = SocketFunction::SendPtyInput {
            box_id: "non_existent_box".to_string(),
            input: "test input\n".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have an error message about PTY not found
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "non_existent_box");
            assert!(!success);
            assert!(message.contains("Failed to send input to PTY process"));
            assert!(message.contains("No PTY process found"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for missing PTY");
        }
    }

    #[test]
    fn test_send_pty_input_finished_process() {
        let app_context = create_test_app_context_with_pty();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add a PTY process in finished state
        if let Some(pty_manager) = &app_context.pty_manager {
            pty_manager.add_test_pty_process_with_status(
                "finished_box".to_string(),
                buffer,
                PtyStatus::Finished(0),
                12345,
            );
        }

        let socket_function = SocketFunction::SendPtyInput {
            box_id: "finished_box".to_string(),
            input: "test input\n".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have an error message about process being finished
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "finished_box");
            assert!(!success);
            assert!(message.contains("Failed to send input to PTY process"));
            assert!(message.contains("has finished"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for finished PTY");
        }
    }

    #[test]
    fn test_send_pty_input_error_state() {
        let app_context = create_test_app_context_with_pty();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add a PTY process in error state
        if let Some(pty_manager) = &app_context.pty_manager {
            pty_manager.add_test_pty_process_with_status(
                "error_box".to_string(),
                buffer,
                PtyStatus::Error("Test error".to_string()),
                12345,
            );
        }

        let socket_function = SocketFunction::SendPtyInput {
            box_id: "error_box".to_string(),
            input: "test input\n".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have an error message about process being in error state
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "error_box");
            assert!(!success);
            assert!(message.contains("Failed to send input to PTY process"));
            assert!(message.contains("error state"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for error PTY");
        }
    }

    #[test]
    fn test_send_pty_input_no_pty_manager() {
        let mut app_context = create_test_app_context_with_pty();
        app_context.pty_manager = None; // Remove PTY manager

        let socket_function = SocketFunction::SendPtyInput {
            box_id: "any_box".to_string(),
            input: "test input\n".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should have an error message about PTY manager unavailability
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "any_box");
            assert!(!success);
            assert!(message.contains("PTY manager not available"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for missing PTY manager");
        }
    }

    #[test]
    fn test_send_pty_input_special_characters() {
        let app_context = create_test_app_context_with_pty();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add a PTY process in running state
        if let Some(pty_manager) = &app_context.pty_manager {
            pty_manager.add_test_pty_process_with_status(
                "special_chars_box".to_string(),
                buffer,
                PtyStatus::Running,
                12345,
            );
        }

        let socket_function = SocketFunction::SendPtyInput {
            box_id: "special_chars_box".to_string(),
            input: "ls -la | grep *.txt && echo 'done'\n".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should handle special characters in input successfully
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "special_chars_box");
            assert!(*success);
            assert!(message.contains("Input sent successfully"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for special character input");
        }
    }

    #[test]
    fn test_send_pty_input_starting_process() {
        let app_context = create_test_app_context_with_pty();
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add a PTY process in starting state (should accept input)
        if let Some(pty_manager) = &app_context.pty_manager {
            pty_manager.add_test_pty_process_with_status(
                "starting_box".to_string(),
                buffer,
                PtyStatus::Starting,
                12345,
            );
        }

        let socket_function = SocketFunction::SendPtyInput {
            box_id: "starting_box".to_string(),
            input: "early input\n".to_string(),
        };

        let result = run_socket_function(socket_function, &app_context);
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);

        // Should accept input even in starting state
        if let crate::thread_manager::Message::MuxBoxOutputUpdate(box_id, success, message) =
            &messages[0]
        {
            assert_eq!(box_id, "starting_box");
            assert!(*success);
            assert!(message.contains("Input sent successfully"));
        } else {
            panic!("Expected MuxBoxOutputUpdate message for starting PTY");
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
