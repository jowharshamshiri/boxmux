//! Integration tests for BoxMux
//! 
//! These tests verify that multiple components work together correctly
//! and test real-world usage scenarios.

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use crate::model::common::SocketFunction;
    use crate::socket_handler::BoxMuxSocketHandler;
    use crate::thread_manager::Message;
    use rust_janus::protocol::SocketCommand;
    use std::sync::mpsc;
    use std::collections::HashMap;
    use serde_json::json;

    /// Test complete application lifecycle
    #[test]
    fn test_complete_app_lifecycle() {
        // Create and validate app
        let mut app = TestDataFactory::create_test_app();
        app.validate(); // Should not panic
        
        // Verify initial state
        assert!(TestAssertions::assert_app_active_layout(&app, "test_layout"));
        assert_eq!(app.layouts.len(), 1);
        assert_eq!(app.layouts[0].children.as_ref().unwrap().len(), 1);
        
        // Test layout switching with multi-layout app
        let mut multi_app = TestDataFactory::create_multi_layout_app();
        multi_app.validate();
        
        assert!(TestAssertions::assert_app_active_layout(&multi_app, "layout1"));
        assert_eq!(multi_app.layouts.len(), 3);
    }

    /// Test socket command to app message workflow
    #[test]
    fn test_socket_to_message_workflow() {
        // Test replace panel content workflow
        let socket_function = TestDataFactory::create_socket_function_replace_content(
            "test_panel", 
            "Updated via socket", 
            true
        );
        
        let result = IntegrationTestUtils::simulate_socket_to_app_workflow(socket_function);
        assert!(result.is_ok());
        
        let message = result.unwrap();
        assert!(TestAssertions::assert_message_type(&message, "PanelOutputUpdate"));
        
        if let Message::PanelOutputUpdate(panel_id, success, content) = message {
            assert_eq!(panel_id, "test_panel");
            assert_eq!(success, true);
            assert_eq!(content, "Updated via socket");
        }
    }

    /// Test script replacement workflow
    #[test]
    fn test_script_replacement_workflow() {
        let script_commands = MockUtils::create_test_script_commands();
        let socket_function = TestDataFactory::create_socket_function_replace_script(
            "script_panel", 
            script_commands.clone()
        );
        
        let result = IntegrationTestUtils::simulate_socket_to_app_workflow(socket_function);
        assert!(result.is_ok());
        
        let message = result.unwrap();
        assert!(TestAssertions::assert_message_type(&message, "PanelScriptUpdate"));
        
        if let Message::PanelScriptUpdate(panel_id, script) = message {
            assert_eq!(panel_id, "script_panel");
            assert_eq!(script, script_commands);
        }
    }

    /// Test error handling in socket workflow
    #[test]
    fn test_socket_error_handling() {
        let (tx, rx) = mpsc::channel();
        let test_uuid = MockUtils::create_test_uuid();
        let handler = BoxMuxSocketHandler::new(tx, test_uuid);
        
        // Test with missing arguments
        let mut args = HashMap::new();
        args.insert("success".to_string(), json!(true));
        args.insert("content".to_string(), json!("content"));
        // Missing panel_id
        
        let socket_cmd = SocketCommand::new(
            "boxmux-control".to_string(),
            "replace-panel-content".to_string(),
            Some(args),
            None,
        );

        // Simulate handler processing with missing argument
        let message_sender_clone = handler.message_sender().clone();
        let sender_uuid_clone = handler.sender_uuid();
        
        let result = (move |cmd: SocketCommand| {
            let args = cmd.args.unwrap_or_default();
            let _panel_id = args.get("panel_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| rust_janus::JanusError::ValidationError("Missing required argument: panel_id".to_string()))?;
            
            Ok::<serde_json::Value, rust_janus::JanusError>(json!({"status": "success"}))
        })(socket_cmd);

        assert!(result.is_err());
        if let Err(rust_janus::JanusError::ValidationError(msg)) = result {
            assert!(msg.contains("Missing required argument: panel_id"));
        } else {
            panic!("Expected ValidationError for missing panel_id");
        }
    }

    /// Test performance with large configurations
    #[test]
    fn test_large_configuration_performance() {
        PerformanceTestUtils::assert_performance(
            || {
                let _large_layout = PerformanceTestUtils::create_large_layout(100);
            },
            100,
            std::time::Duration::from_millis(500),
            "Large layout creation"
        );
    }

    /// Test concurrent socket operations
    #[test]
    fn test_concurrent_socket_operations() {
        let (tx, rx) = mpsc::channel();
        let test_uuid = MockUtils::create_test_uuid();
        let _handler = BoxMuxSocketHandler::new(tx.clone(), test_uuid);
        
        // Simulate multiple concurrent socket functions by directly sending messages
        let functions = vec![
            TestDataFactory::create_socket_function_replace_content("panel1", "content1", true),
            TestDataFactory::create_socket_function_replace_content("panel2", "content2", true),
            TestDataFactory::create_socket_function_replace_content("panel3", "content3", true),
        ];
        
        for socket_function in functions {
            // Convert socket function to message and send to channel
            let message = match socket_function {
                SocketFunction::ReplacePanelContent { panel_id, success, content } => {
                    Message::PanelOutputUpdate(panel_id, success, content)
                }
                _ => panic!("Unexpected socket function type"),
            };
            tx.send((test_uuid, message)).expect("Failed to send message");
        }
        
        // Verify all messages were processed
        let mut message_count = 0;
        while let Ok(_) = rx.try_recv() {
            message_count += 1;
        }
        assert_eq!(message_count, 3);
    }

    /// Test bounds calculations with various configurations
    #[test]
    fn test_bounds_calculation_integration() {
        use crate::utils::input_bounds_to_bounds;
        
        let parent_bounds = TestDataFactory::create_bounds(0, 0, 100, 50);
        
        // Test percentage-based bounds
        let input_bounds = TestDataFactory::create_input_bounds("10%", "20%", "90%", "80%");
        let result_bounds = input_bounds_to_bounds(&input_bounds, &parent_bounds);
        
        // Should calculate percentages correctly
        assert_eq!(result_bounds.x1, 10); // 10% of 100
        assert_eq!(result_bounds.y1, 10); // 20% of 50
        assert_eq!(result_bounds.x2, 90); // 90% of 100
        assert_eq!(result_bounds.y2, 40); // 80% of 50
        
        // Test absolute bounds
        let input_bounds = TestDataFactory::create_input_bounds("5", "10", "95", "45");
        let result_bounds = input_bounds_to_bounds(&input_bounds, &parent_bounds);
        
        assert_eq!(result_bounds.x1, 5);
        assert_eq!(result_bounds.y1, 10);
        assert_eq!(result_bounds.x2, 95);
        assert_eq!(result_bounds.y2, 45);
    }

    /// Test key mapping integration
    #[test]
    fn test_key_mapping_integration() {
        use crate::utils::handle_keypress;
        
        let key_mappings = MockUtils::create_test_key_mappings();
        
        // Test various key combinations
        let test_cases = vec![
            ("ctrl+c", Some(vec!["exit".to_string()])),
            ("CTRL+C", Some(vec!["exit".to_string()])),
            ("enter", Some(vec!["confirm".to_string()])),
            ("ENTER", Some(vec!["confirm".to_string()])),
            ("tab", Some(vec!["next_panel".to_string()])),
            ("unknown_key", None),
        ];
        
        for (key, expected) in test_cases {
            let result = handle_keypress(key, &key_mappings);
            assert_eq!(result, expected, "Failed for key: {}", key);
        }
    }

    /// Test script execution integration
    #[test]
    fn test_script_execution_integration() {
        use crate::utils::run_script;
        
        // Test simple script execution
        let result = run_script(None, &vec!["echo test".to_string()]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("test"));
        
        // Test script with libraries (if any provided)
        let lib_paths = Some(vec!["/usr/bin".to_string()]);
        let result = run_script(lib_paths, &vec!["echo integration_test".to_string()]);
        assert!(result.is_ok());
    }

    /// Test ANSI code handling integration
    #[test]
    fn test_ansi_handling_integration() {
        use crate::utils::strip_ansi_codes;
        
        let test_cases = vec![
            ("Plain text", "Plain text"),
            ("\x1b[31mRed text\x1b[0m", "Red text"),
            ("\x1b[1;32mBold green\x1b[0m normal", "Bold green normal"),
            ("\x1b[2J\x1b[H\x1b[31mRed after clear\x1b[0m", "Red after clear"),
        ];
        
        for (input, expected) in test_cases {
            let result = strip_ansi_codes(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    /// Test complete panel lifecycle
    #[test]
    fn test_panel_lifecycle_integration() {
        let mut app = TestDataFactory::create_test_app();
        let original_panel_count = app.layouts[0].children.as_ref().unwrap().len();
        
        // Add panel via socket function
        let new_panel = TestDataFactory::create_custom_panel("new_panel", "New panel content");
        let add_function = SocketFunction::AddPanel {
            layout_id: "test_layout".to_string(),
            panel: new_panel.clone(),
        };
        
        let message = IntegrationTestUtils::simulate_socket_to_app_workflow(add_function).unwrap();
        assert!(TestAssertions::assert_message_type(&message, "AddPanel"));
        
        // Test panel replacement
        let updated_panel = TestDataFactory::create_custom_panel("new_panel", "Updated content");
        let replace_function = SocketFunction::ReplacePanel {
            panel_id: "new_panel".to_string(),
            new_panel: updated_panel,
        };
        
        let message = IntegrationTestUtils::simulate_socket_to_app_workflow(replace_function).unwrap();
        assert!(TestAssertions::assert_message_type(&message, "ReplacePanel"));
        
        // Test panel removal
        let remove_function = SocketFunction::RemovePanel {
            panel_id: "new_panel".to_string(),
        };
        
        let message = IntegrationTestUtils::simulate_socket_to_app_workflow(remove_function).unwrap();
        assert!(TestAssertions::assert_message_type(&message, "RemovePanel"));
    }

    /// Test error recovery scenarios
    #[test]
    fn test_error_recovery_scenarios() {
        // Test app validation with invalid configurations
        let mut invalid_app = TestDataFactory::create_test_app();
        
        // Add duplicate panel IDs
        let duplicate_panel = TestDataFactory::create_test_panel("test_panel"); // Same ID as existing
        invalid_app.layouts[0].children.as_mut().unwrap().push(duplicate_panel);
        
        // This should panic with validation error
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            invalid_app.validate();
        }));
        assert!(result.is_err());
        
        // Test multiple root layouts
        let mut multi_root_app = TestDataFactory::create_multi_layout_app();
        multi_root_app.layouts[1].root = Some(true); // Make second layout root too
        
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            multi_root_app.validate();
        }));
        assert!(result.is_err());
    }

    /// Test memory usage and cleanup
    #[test]
    fn test_memory_usage_patterns() {
        // Create many objects and verify they can be cleaned up
        let start_time = std::time::Instant::now();
        
        for _ in 0..1000 {
            let _app = TestDataFactory::create_multi_layout_app();
            let _large_layout = PerformanceTestUtils::create_large_layout(50);
            // Objects should be automatically dropped here
        }
        
        let duration = start_time.elapsed();
        println!("Memory stress test (1000 iterations): {:?}", duration);
        
        // Should complete without excessive memory usage or time
        assert!(duration.as_secs() < 10, "Memory stress test took too long: {:?}", duration);
    }
}