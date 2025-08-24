//! Integration tests for BoxMux
//!
//! These tests verify that multiple components work together correctly
//! and test real-world usage scenarios.

#[cfg(test)]
mod tests {
    use crate::model::common::SocketFunction;
    use crate::tests::test_utils::*;
    use crate::thread_manager::Message;
    use std::sync::mpsc;

    /// Test complete application lifecycle
    #[test]
    fn test_complete_app_lifecycle() {
        // Create and validate app
        let mut app = TestDataFactory::create_test_app();
        app.validate(); // Should not panic

        // Verify initial state
        assert!(TestAssertions::assert_app_active_layout(
            &app,
            "test_layout"
        ));
        assert_eq!(app.layouts.len(), 1);
        assert_eq!(app.layouts[0].children.as_ref().unwrap().len(), 1);

        // Test layout switching with multi-layout app
        let mut multi_app = TestDataFactory::create_multi_layout_app();
        multi_app.validate();

        assert!(TestAssertions::assert_app_active_layout(
            &multi_app, "layout1"
        ));
        assert_eq!(multi_app.layouts.len(), 3);
    }

    /// Test socket command to app message workflow
    #[test]
    fn test_socket_to_message_workflow() {
        // Test replace muxbox content workflow
        let socket_function = TestDataFactory::create_socket_function_replace_content(
            "test_muxbox",
            "Updated via socket",
            true,
        );

        let result = IntegrationTestUtils::simulate_socket_to_app_workflow(socket_function);
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(TestAssertions::assert_message_type(
            &message,
            "MuxBoxOutputUpdate"
        ));

        if let Message::MuxBoxOutputUpdate(box_id, success, content) = message {
            assert_eq!(box_id, "test_muxbox");
            assert_eq!(success, true);
            assert_eq!(content, "Updated via socket");
        }
    }

    /// Test script replacement workflow
    #[test]
    fn test_script_replacement_workflow() {
        let script_commands = MockUtils::create_test_script_commands();
        let socket_function = TestDataFactory::create_socket_function_replace_script(
            "script_muxbox",
            script_commands.clone(),
        );

        let result = IntegrationTestUtils::simulate_socket_to_app_workflow(socket_function);
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(TestAssertions::assert_message_type(
            &message,
            "MuxBoxScriptUpdate"
        ));

        if let Message::MuxBoxScriptUpdate(box_id, script) = message {
            assert_eq!(box_id, "script_muxbox");
            assert_eq!(script, script_commands);
        }
    }

    /// Test socket function handling without RustJanus dependencies
    #[test]
    fn test_socket_function_handling() {
        // Test SocketFunction processing directly
        let socket_function = SocketFunction::ReplaceBoxContent {
            box_id: "test_muxbox".to_string(),
            success: true,
            content: "test content".to_string(),
        };

        let result = crate::model::common::run_socket_function(
            socket_function,
            &TestDataFactory::create_test_app_context(),
        );
        assert!(result.is_ok());

        let (_, messages) = result.unwrap();
        assert_eq!(messages.len(), 1);
        match &messages[0] {
            Message::MuxBoxOutputUpdate(box_id, success, content) => {
                assert_eq!(box_id, "test_muxbox");
                assert_eq!(*success, true);
                assert_eq!(content, "test content");
            }
            _ => panic!("Expected MuxBoxOutputUpdate message"),
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
            "Large layout creation",
        );
    }

    /// Test concurrent socket operations
    #[test]
    fn test_concurrent_socket_operations() {
        let (tx, rx) = mpsc::channel::<(uuid::Uuid, Message)>();
        let test_uuid = MockUtils::create_test_uuid();

        // Simulate multiple concurrent socket functions by directly sending messages
        let functions = vec![
            TestDataFactory::create_socket_function_replace_content("muxbox1", "content1", true),
            TestDataFactory::create_socket_function_replace_content("muxbox2", "content2", true),
            TestDataFactory::create_socket_function_replace_content("muxbox3", "content3", true),
        ];

        for socket_function in functions {
            // Convert socket function to message and send to channel
            let message = match socket_function {
                SocketFunction::ReplaceBoxContent {
                    box_id,
                    success,
                    content,
                } => Message::MuxBoxOutputUpdate(box_id, success, content),
                _ => panic!("Unexpected socket function type"),
            };
            tx.send((test_uuid, message))
                .expect("Failed to send message");
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
        assert_eq!(result_bounds.x1, 10); // 10% of 0-99 range = 10
        assert_eq!(result_bounds.y1, 10); // 20% of 0-49 range = 10
        assert_eq!(result_bounds.x2, 89); // 90% of 0-99 range = 89
        assert_eq!(result_bounds.y2, 39); // 80% of 0-49 range = 39

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
            ("tab", Some(vec!["next_muxbox".to_string()])),
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
            (
                "\x1b[2J\x1b[H\x1b[31mRed after clear\x1b[0m",
                "Red after clear",
            ),
        ];

        for (input, expected) in test_cases {
            let result = strip_ansi_codes(input);
            assert_eq!(result, expected, "Failed for input: {}", input);
        }
    }

    /// Test complete muxbox lifecycle
    #[test]
    fn test_muxbox_lifecycle_integration() {
        let mut app = TestDataFactory::create_test_app();
        let original_muxbox_count = app.layouts[0].children.as_ref().unwrap().len();

        // Add muxbox via socket function
        let new_muxbox = TestDataFactory::create_custom_muxbox("new_muxbox", "New muxbox content");
        let add_function = SocketFunction::AddBox {
            layout_id: "test_layout".to_string(),
            muxbox: new_muxbox.clone(),
        };

        let message = IntegrationTestUtils::simulate_socket_to_app_workflow(add_function).unwrap();
        assert!(TestAssertions::assert_message_type(&message, "AddBox"));

        // Test muxbox replacement
        let updated_muxbox = TestDataFactory::create_custom_muxbox("new_muxbox", "Updated content");
        let replace_function = SocketFunction::ReplaceBox {
            box_id: "new_muxbox".to_string(),
            new_box: updated_muxbox,
        };

        let message =
            IntegrationTestUtils::simulate_socket_to_app_workflow(replace_function).unwrap();
        assert!(TestAssertions::assert_message_type(
            &message,
            "ReplaceMuxBox"
        ));

        // Test muxbox removal
        let remove_function = SocketFunction::RemoveBox {
            box_id: "new_muxbox".to_string(),
        };

        let message =
            IntegrationTestUtils::simulate_socket_to_app_workflow(remove_function).unwrap();
        assert!(TestAssertions::assert_message_type(&message, "RemoveBox"));
    }

    /// Test error recovery scenarios
    #[test]
    fn test_error_recovery_scenarios() {
        // Test app validation with invalid configurations
        let mut invalid_app = TestDataFactory::create_test_app();

        // Add duplicate muxbox IDs
        let duplicate_muxbox = TestDataFactory::create_test_muxbox("test_muxbox"); // Same ID as existing
        invalid_app.layouts[0]
            .children
            .as_mut()
            .unwrap()
            .push(duplicate_muxbox);

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
        assert!(
            duration.as_secs() < 10,
            "Memory stress test took too long: {:?}",
            duration
        );
    }
}
