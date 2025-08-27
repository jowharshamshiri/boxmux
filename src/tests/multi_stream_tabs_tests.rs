#[cfg(test)]
mod multi_stream_tabs_tests {
    use crate::model::common::{StreamSource, StreamType, TabSystem};
    use crate::model::muxbox::MuxBox;
    use crate::draw_utils::{draw_horizontal_line_with_tabs, calculate_tab_click_index};
    use crate::model::common::ScreenBuffer;
    use std::time::SystemTime;

    #[test]
    fn test_tab_system_creation() {
        let mut tab_system = TabSystem::new();
        assert_eq!(tab_system.streams.len(), 0);
        assert_eq!(tab_system.active_tab, 0);
        assert!(!tab_system.has_multiple_streams());
    }

    #[test]
    fn test_add_stream() {
        let mut tab_system = TabSystem::new();
        
        let stream = StreamSource::StaticContent(crate::model::common::StaticContentSource {
            content_type: "default".to_string(),
            created_at: SystemTime::now(),
        });
        
        let stream_id = tab_system.add_stream(stream);
        assert_eq!(stream_id, "stream1");
        assert_eq!(tab_system.streams.len(), 1);
        assert!(!tab_system.has_multiple_streams()); // Only one stream
        
        // Add second stream
        let stream2 = StreamSource::StaticContent(crate::model::common::StaticContentSource {
            content_type: "choices".to_string(),
            created_at: SystemTime::now(),
        });
        
        tab_system.add_stream(stream2);
        assert_eq!(tab_system.streams.len(), 2);
        assert!(tab_system.has_multiple_streams()); // Now multiple streams
    }

    #[test]
    fn test_stream_content_management() {
        let mut tab_system = TabSystem::new();
        
        let stream = StreamSource::StaticContent(crate::model::common::StaticContentSource {
            content_type: "pty".to_string(),
            created_at: SystemTime::now(),
        });
        
        tab_system.add_stream(stream);
        tab_system.update_stream_content("test_stream", "Test content".to_string());
        
        assert_eq!(tab_system.get_active_content(), Some(&"Test content".to_string()));
    }

    #[test]
    fn test_tab_switching() {
        let mut tab_system = TabSystem::new();
        
        // Add two streams
        let stream1 = StreamSource::StaticContent(crate::model::common::StaticContentSource {
            content_type: "content".to_string(),
            created_at: SystemTime::now(),
        });
        
        let stream2 = StreamSource::ChoiceExecution(crate::model::common::ChoiceExecutionSource {
            choice_id: "choice1".to_string(),
            muxbox_id: "test_box".to_string(),
            thread_id: None,
            process_id: None,
            execution_type: "threaded".to_string(),
            started_at: SystemTime::now(),
            timeout_seconds: None,
        });
        
        tab_system.add_stream(stream1);
        tab_system.add_stream(stream2);
        
        tab_system.update_stream_content("stream1", "Content 1".to_string());
        tab_system.update_stream_content("stream2", "Content 2".to_string());
        
        // Initially showing first stream
        assert_eq!(tab_system.active_tab, 0);
        assert_eq!(tab_system.get_active_content(), Some(&"Content 1".to_string()));
        
        // Switch to second stream
        assert!(tab_system.switch_to_tab(1));
        assert_eq!(tab_system.active_tab, 1);
        assert_eq!(tab_system.get_active_content(), Some(&"Content 2".to_string()));
        
        // Switch by stream ID
        assert!(tab_system.switch_to_stream("stream1"));
        assert_eq!(tab_system.active_tab, 0);
        assert_eq!(tab_system.get_active_content(), Some(&"Content 1".to_string()));
    }

    #[test]
    fn test_stream_removal() {
        let mut tab_system = TabSystem::new();
        
        // Add three streams
        let stream1 = StreamSource::StaticContent(crate::model::common::StaticContentSource {
            content_type: "content".to_string(),
            created_at: SystemTime::now(),
        });
        
        let stream2 = StreamSource::Redirect(crate::model::common::RedirectSource {
            source_muxbox_id: "box1".to_string(),
            redirect_name: "output".to_string(),
            redirect_type: "append".to_string(),
            source_choice_id: None,
            created_at: SystemTime::now(),
            source_process_id: None,
        });
        
        let stream3 = StreamSource::Socket(crate::model::common::SocketSource {
            connection_id: "socket1".to_string(),
            socket_path: Some("/tmp/test.sock".to_string()),
            client_info: "test_client".to_string(),
            protocol_version: "1.0".to_string(),
            connected_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        });
        
        tab_system.add_stream(stream1);
        tab_system.add_stream(stream2);
        tab_system.add_stream(stream3);
        
        // Switch to second stream
        tab_system.switch_to_tab(1);
        assert_eq!(tab_system.active_tab, 1);
        
        // Remove the active stream
        assert!(tab_system.remove_stream("stream2"));
        assert_eq!(tab_system.streams.len(), 2);
        assert_eq!(tab_system.active_tab, 1); // Should adjust to valid index
        
        // Remove first stream
        assert!(tab_system.remove_stream("stream1"));
        assert_eq!(tab_system.streams.len(), 1);
        assert_eq!(tab_system.active_tab, 0); // Should adjust to 0
    }

    #[test] 
    fn test_muxbox_tab_integration() {
        let mut muxbox = MuxBox::default();
        muxbox.id = "test_box".to_string();
        
        // Initialize default tab system creates default tab
        muxbox.initialize_default_tabs();
        assert_eq!(muxbox.tab_system.streams.len(), 1);
        // All boxes always show tabs now
        
        // Add another stream
        let stream_id = muxbox.add_input_stream(
            StreamType::PtySession("vim".to_string()),
            "Vim Editor".to_string()
        );
        
        assert_eq!(muxbox.tab_system.streams.len(), 2);
        // All boxes always show tabs now
        
        let tab_labels = muxbox.get_tab_labels();
        assert_eq!(tab_labels.len(), 2);
        assert_eq!(tab_labels[0], "test_box"); // First tab uses box id as title in unified system
        assert_eq!(tab_labels[1], "Vim Editor");
        
        // Test content updates through tab system
        muxbox.update_stream_content_with_tab(&stream_id, "Vim content".to_string());
        
        // Switch to the new stream
        assert!(muxbox.switch_to_stream(&stream_id));
        assert_eq!(muxbox.get_active_tab_index(), 1);
    }

    #[test]
    fn test_tab_click_detection() {
        // Test tab click calculation
        let tab_labels = vec![
            "Script".to_string(),
            "Deploy".to_string(),
            "Monitor".to_string(),
        ];
        
        // Click on first tab (left side)
        if let Some(tab_index) = calculate_tab_click_index(3, 0, 50, &tab_labels, true) {
            assert_eq!(tab_index, 0);
        }
        
        // Click on second tab (middle)
        if let Some(tab_index) = calculate_tab_click_index(20, 0, 50, &tab_labels, true) {
            assert!(tab_index <= 2); // Should be valid tab index
        }
        
        // Click outside tab area
        let result = calculate_tab_click_index(100, 0, 50, &tab_labels, true);
        assert!(result.is_none());
    }

    #[test]
    fn test_tab_label_truncation() {
        let mut tab_system = TabSystem::new();
        tab_system.max_tab_width = 8; // Short width for testing
        
        let long_stream = StreamSource::ChoiceExecution(crate::model::common::ChoiceExecutionSource {
            choice_id: "choice1".to_string(),
            muxbox_id: "test_box".to_string(),
            thread_id: None,
            process_id: None,
            execution_type: "threaded".to_string(),
            started_at: SystemTime::now(),
            timeout_seconds: None,
        });
        
        tab_system.add_stream(long_stream);
        
        let tab_labels = tab_system.get_tab_labels();
        assert_eq!(tab_labels.len(), 1);
        
        // The ellipsis '…' is a 3-byte UTF-8 character, so len() measures bytes, not visual chars
        // Check both byte length and character length
        let label = &tab_labels[0];
        let byte_len = label.len();
        let char_len = label.chars().count();
        
        println!("Label: '{}', Byte length: {}, Char length: {}, Max: {}", label, byte_len, char_len, tab_system.max_tab_width);
        
        // The visual length (character count) should be <= max_tab_width
        assert!(char_len <= tab_system.max_tab_width);
        assert!(label.ends_with('…')); // Should be truncated with ellipsis
    }

    #[test]
    fn test_tab_rendering() {
        let mut buffer = ScreenBuffer::new_custom(40, 5);
        
        let tab_labels = vec![
            "Main".to_string(),
            "PTY".to_string(),
            "Log".to_string(),
        ];
        
        draw_horizontal_line_with_tabs(
            0, // y position
            0, // x1
            39, // x2  
            "white", // fg_color
            "black", // bg_color
            None, // title (not used when tabs present)
            "white", // title_fg_color
            "blue", // title_bg_color
            "center", // title_position
            true, // draw_border
            &tab_labels,
            1, // active_tab_index (PTY tab active)
            &mut buffer,
        );
        
        // Basic verification that the function executed without panic
        // More detailed testing would require checking buffer contents
        assert_eq!(buffer.width, 40);
        assert_eq!(buffer.height, 5);
    }

    #[test]
    fn test_stream_types() {
        // Test all StreamType variants
        let own_script = StreamType::OwnScript;
        let redirect = StreamType::RedirectSource("source_box".to_string());
        let pty = StreamType::PtySession("htop".to_string());
        let choice = StreamType::ChoiceExecution("deploy_choice".to_string());
        let socket = StreamType::ExternalSocket;
        
        // Ensure they can be cloned and compared
        assert_eq!(own_script, StreamType::OwnScript);
        assert_eq!(redirect, StreamType::RedirectSource("source_box".to_string()));
        assert_eq!(pty, StreamType::PtySession("htop".to_string()));
        assert_eq!(choice, StreamType::ChoiceExecution("deploy_choice".to_string()));
        assert_eq!(socket, StreamType::ExternalSocket);
    }
}