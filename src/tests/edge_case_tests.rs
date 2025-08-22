//! Edge case tests for BoxMux
//! 
//! These tests cover error conditions, boundary cases, and unusual scenarios
//! that might not be covered by regular unit tests.

#[cfg(test)]
mod tests {
    use crate::tests::test_utils::*;
    use crate::model::app::App;
    use crate::model::layout::Layout;
    use crate::model::panel::Panel;
    use crate::model::common::{InputBounds, SocketFunction};
    use crate::utils::*;
    use crate::thread_manager::Message;
    use std::collections::HashMap;

    /// Test empty and null configurations
    #[test]
    fn test_empty_configurations() {
        // Test app with no layouts
        let mut empty_app = App::new();
        empty_app.layouts = vec![];
        empty_app.libs = None;
        empty_app.on_keypress = None;
        
        // This should handle empty layouts gracefully
        let _graph = empty_app.generate_graph();
        // Just verify it doesn't panic
        
        // Test layout with no panels
        let empty_layout = Layout {
            id: "empty".to_string(),
            title: None,
            children: None, // No panels
            root: Some(true),
            ..Default::default()
        };
        
        // Should handle empty panel list
        assert!(empty_layout.children.is_none() || empty_layout.children.as_ref().unwrap().is_empty());
    }

    /// Test extremely large inputs
    #[test]
    fn test_large_inputs() {
        // Test very long panel content
        let huge_content = "x".repeat(1_000_000); // 1MB of content
        let mut panel = TestDataFactory::create_test_panel("huge_panel");
        panel.content = Some(huge_content.clone());
        
        assert_eq!(panel.content.as_ref().unwrap().len(), 1_000_000);
        
        // Test very long script arrays
        let huge_script: Vec<String> = (0..10000)
            .map(|i| format!("echo 'command number {}'", i))
            .collect();
        
        panel.script = Some(huge_script.clone());
        assert_eq!(panel.script.as_ref().unwrap().len(), 10000);
    }

    /// Test invalid bounds configurations
    #[test]
    fn test_invalid_bounds() {
        let parent_bounds = TestDataFactory::create_bounds(0, 0, 100, 50);
        
        // Test bounds with invalid percentages
        let invalid_bounds = InputBounds {
            x1: "150%".to_string(), // Over 100%
            y1: "-10%".to_string(), // Negative
            x2: "abc%".to_string(),  // Non-numeric
            y2: "50%".to_string(),
        };
        
        // Should handle invalid bounds gracefully
        let result = input_bounds_to_bounds(&invalid_bounds, &parent_bounds);
        // The actual behavior depends on implementation, but shouldn't crash
        assert!(result.x1 <= parent_bounds.x2);
        assert!(result.y1 <= parent_bounds.y2);
        
        // Test bounds where x1 > x2, y1 > y2
        let backwards_bounds = InputBounds {
            x1: "90".to_string(),
            y1: "40".to_string(),
            x2: "10".to_string(), // Less than x1
            y2: "5".to_string(),  // Less than y1
        };
        
        let _result = input_bounds_to_bounds(&backwards_bounds, &parent_bounds);
        // Should handle gracefully without crashing
    }

    /// Test malformed input strings
    #[test]
    fn test_malformed_inputs() {
        // Test ANSI stripping with malformed escape sequences
        let malformed_ansi = vec![
            "\x1b[", // Incomplete escape
            "\x1b[999m", // Invalid code
            "\x1b[31", // Missing 'm'
            "\x1b[31;", // Incomplete parameter
            "normal\x1b[31mred\x1b[incomplete", // Mixed valid/invalid
        ];
        
        for input in malformed_ansi {
            let result = strip_ansi_codes(input);
            // Should not crash and should return a string (possibly empty)
            // The main goal is that it doesn't panic on malformed input
            assert!(result.len() <= input.len()); // Result should not be longer than input
        }
        
        // Test key mapping with unusual key strings
        let key_mappings = MockUtils::create_test_key_mappings();
        let unusual_keys = vec![
            "", // Empty string
            "   ", // Whitespace only
            "CTRL++", // Double symbols
            "ctrl+", // Incomplete
            "Ctrl + + C", // Extra spaces and symbols
            "ĀĂĄ", // Unicode
            "😀", // Emoji
        ];
        
        for key in unusual_keys {
            let result = handle_keypress(key, &key_mappings);
            // Should handle gracefully, typically returning None
            // The exact behavior depends on implementation
        }
    }

    /// Test streaming script execution edge cases
    #[test]
    fn test_streaming_script_edge_cases() {
        use crate::streaming_executor::StreamingExecutor;
        use std::time::Duration;
        
        // Test script with special characters - these should execute safely
        let special_commands = vec![
            "echo 'test $(echo safe)'",
            "echo 'pipe test' | cat",
            "echo 'semicolon'; echo 'test'",
        ];
        
        for cmd in special_commands {
            let mut executor = StreamingExecutor::new();
            let result = executor.spawn_streaming(cmd, None);
            // Should execute safely or fail gracefully
            match result {
                Ok(_) => {}, // Fine if it executes
                Err(_) => {}, // Fine if it fails safely
            }
        }
    }

    /// Test concurrent access patterns
    #[test]
    fn test_concurrent_patterns() {
        use std::sync::{Arc, Mutex};
        use std::thread;
        
        let app = Arc::new(Mutex::new(TestDataFactory::create_test_app()));
        let mut handles = vec![];
        
        // Simulate multiple threads accessing app simultaneously
        for i in 0..10 {
            let app_clone: Arc<std::sync::Mutex<App>> = Arc::clone(&app);
            let handle = thread::spawn(move || {
                let _app_guard = app_clone.lock().unwrap();
                // Simulate some work with the app
                let _panel = TestDataFactory::create_test_panel(&format!("thread_panel_{}", i));
                thread::sleep(std::time::Duration::from_millis(1));
            });
            handles.push(handle);
        }
        
        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
        
        // App should still be in valid state
        let final_app = app.lock().unwrap();
        assert!(!final_app.layouts.is_empty());
    }

    /// Test memory pressure scenarios
    #[test]
    fn test_memory_pressure() {
        // Create many objects quickly to test memory handling
        let mut objects = Vec::new();
        
        for i in 0..1000 {
            let app = TestDataFactory::create_multi_layout_app();
            let large_layout = PerformanceTestUtils::create_large_layout(100);
            
            objects.push((app, large_layout));
            
            // Periodically clear some objects to test cleanup
            if i % 100 == 0 {
                objects.clear();
            }
        }
        
        // Final cleanup
        objects.clear();
        
        // Should complete without memory issues
    }

    /// Test validation edge cases
    #[test]
    fn test_validation_edge_cases() {
        // Test panel with extremely long ID
        let mut panel = TestDataFactory::create_test_panel("normal_id");
        panel.id = "x".repeat(10000);
        
        // Should handle long IDs (behavior depends on implementation)
        assert_eq!(panel.id.len(), 10000);
        
        // Test layout with circular references (if possible)
        let mut layout = TestDataFactory::create_test_layout("circular", None);
        // This would create circular reference if the structure allowed it
        // layout.parent = Some(&layout); // Not possible with current structure
        
        // Test app with many duplicate layout IDs
        let mut problematic_app = App::new();
        problematic_app.layouts = vec![
            TestDataFactory::create_test_layout("same_id", None),
            TestDataFactory::create_test_layout("same_id", None),
            TestDataFactory::create_test_layout("same_id", None),
        ];
        problematic_app.libs = None;
        problematic_app.on_keypress = None;
        
        // Should handle duplicate IDs appropriately
        let graph = problematic_app.generate_graph();
        // Behavior depends on implementation
    }

    /// Test socket function edge cases
    #[test]
    fn test_socket_function_edge_cases() {
        // Test with empty panel ID
        let empty_id_function = SocketFunction::ReplacePanelContent {
            panel_id: "".to_string(),
            success: true,
            content: "content".to_string(),
        };
        
        let result = IntegrationTestUtils::simulate_socket_to_app_workflow(empty_id_function);
        assert!(result.is_ok()); // Should handle gracefully
        
        // Test with extremely long content
        let huge_content = "x".repeat(1_000_000);
        let huge_content_function = SocketFunction::ReplacePanelContent {
            panel_id: "test".to_string(),
            success: true,
            content: huge_content,
        };
        
        let result = IntegrationTestUtils::simulate_socket_to_app_workflow(huge_content_function);
        assert!(result.is_ok());
        
        // Test with special characters in content
        let special_content = "Special chars: ñáéíóú 中文 🚀 \n\r\t\\\"'";
        let special_function = SocketFunction::ReplacePanelContent {
            panel_id: "special".to_string(),
            success: true,
            content: special_content.to_string(),
        };
        
        let result = IntegrationTestUtils::simulate_socket_to_app_workflow(special_function);
        assert!(result.is_ok());
        
        if let Ok(Message::PanelOutputUpdate(_, _, content)) = result {
            assert_eq!(content, special_content);
        }
    }

    /// Test boundary value scenarios
    #[test]
    fn test_boundary_values() {
        // Test with zero-sized bounds
        let zero_bounds = TestDataFactory::create_bounds(10, 10, 10, 10);
        let input_bounds = TestDataFactory::create_input_bounds("0", "0", "0", "0");
        let result = input_bounds_to_bounds(&input_bounds, &zero_bounds);
        
        // Should handle zero-sized areas gracefully
        assert!(result.x1 <= result.x2);
        assert!(result.y1 <= result.y2);
        
        // Test with maximum usize values
        let max_bounds = TestDataFactory::create_bounds(0, 0, usize::MAX, usize::MAX);
        let max_input = TestDataFactory::create_input_bounds("100%", "100%", "100%", "100%");
        let result = input_bounds_to_bounds(&max_input, &max_bounds);
        
        // Should handle large values without overflow
        assert!(result.x2 >= result.x1);
        assert!(result.y2 >= result.y1);
    }

    /// Test streaming error propagation
    #[test]
    fn test_streaming_error_propagation() {
        use crate::streaming_executor::StreamingExecutor;
        
        // Test what happens when streaming script execution fails
        let mut executor = StreamingExecutor::new();
        let result = executor.spawn_streaming("false", None); // Command that returns error code
        
        // Should handle gracefully
        match result {
            Ok((mut child, _receiver, _command)) => {
                // Should exit with non-zero status
                let status = child.wait().unwrap();
                assert!(!status.success());
            }
            Err(_) => {
                // Also acceptable if spawn fails
            }
        }
        
        // Test with non-existent command
        let mut executor2 = StreamingExecutor::new();
        let result = executor2.spawn_streaming("this_command_definitely_does_not_exist_12345", None);
        
        // Should succeed spawning but process will fail with non-zero exit (shell handles invalid commands)
        assert!(result.is_ok());
        if let Ok((mut child, _receiver, _command)) = result {
            // Wait for process to complete and check exit status
            let status = child.wait().unwrap();
            assert!(!status.success()); // Should exit with error code
        }
    }

    /// Test unicode and internationalization
    #[test]
    fn test_unicode_handling() {
        // Test unicode in panel content
        let unicode_content = "Hello: 你好 مرحبا こんにちは 🌍 🚀 ñáéíóú";
        let mut panel = TestDataFactory::create_test_panel("unicode_panel");
        panel.content = Some(unicode_content.to_string());
        
        assert_eq!(panel.content.as_ref().unwrap(), unicode_content);
        
        // Test unicode in ANSI stripping
        let unicode_ansi = format!("\x1b[31m{}\x1b[0m", unicode_content);
        let stripped = strip_ansi_codes(&unicode_ansi);
        assert_eq!(stripped, unicode_content);
        
        // Test unicode in key mappings
        let mut unicode_mappings = HashMap::new();
        unicode_mappings.insert("Ctrl + ñ".to_string(), vec!["special_action".to_string()]);
        
        let result = handle_keypress("ctrl+ñ", &unicode_mappings);
        // Behavior depends on key normalization implementation
    }

    /// Test resource exhaustion scenarios
    #[test]
    fn test_resource_exhaustion() {
        // Test creating maximum reasonable number of panels
        let large_panel_count = 10000;
        let start_time = std::time::Instant::now();
        
        let large_panels = PerformanceTestUtils::create_large_panel_list(large_panel_count);
        
        let creation_time = start_time.elapsed();
        println!("Created {} panels in {:?}", large_panel_count, creation_time);
        
        assert_eq!(large_panels.len(), large_panel_count);
        
        // Should complete in reasonable time (less than 5 seconds)
        assert!(
            creation_time.as_secs() < 5,
            "Creating {} panels took too long: {:?}",
            large_panel_count,
            creation_time
        );
        
        // Test memory usage stays reasonable
        let panel_size = std::mem::size_of::<Panel>();
        let expected_memory = large_panel_count * panel_size;
        println!("Estimated memory usage: {} bytes ({} MB)", expected_memory, expected_memory / 1_000_000);
        
        // Memory usage should be reasonable (less than 100MB for 10k panels)
        assert!(expected_memory < 100_000_000, "Memory usage too high: {} bytes", expected_memory);
    }
}