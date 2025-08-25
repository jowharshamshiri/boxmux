use crate::circular_buffer::CircularBuffer;
use crate::model::muxbox::MuxBox;
use crate::pty_manager::PtyManager;
use crate::AppContext;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// F0120: PTY Scrollback Tests
/// Comprehensive test suite for PTY scrollback functionality

#[cfg(test)]
mod pty_scrollback_tests {
    use super::*;

    #[test]
    fn test_pty_muxbox_scrollback_content_retrieval() {
        // Create PTY manager and muxbox
        let pty_manager = PtyManager::new().unwrap();
        let mut muxbox = create_test_pty_muxbox();

        // Add content to circular buffer
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));
        {
            let mut buf = buffer.lock().unwrap();
            buf.push("Line 1".to_string());
            buf.push("Line 2".to_string());
            buf.push("Line 3".to_string());
        }

        // Add PTY process with buffer using test helper
        pty_manager.add_test_pty_process(muxbox.id.clone(), buffer.clone());

        // Test scrollback content retrieval
        let content = muxbox.get_scrollback_content(&pty_manager);
        assert!(content.is_some());
        assert_eq!(content.unwrap(), "Line 1\nLine 2\nLine 3");
    }

    #[test]
    fn test_pty_muxbox_scrollback_line_range() {
        let pty_manager = PtyManager::new().unwrap();
        let muxbox = create_test_pty_muxbox();

        // Create buffer with multiple lines
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));
        {
            let mut buf = buffer.lock().unwrap();
            for i in 1..=10 {
                buf.push(format!("Line {}", i));
            }
        }

        // Add PTY process using test helper
        pty_manager.add_test_pty_process(muxbox.id.clone(), buffer.clone());

        // Test line range retrieval
        let lines = muxbox.get_scrollback_lines(&pty_manager, 2, 3);
        assert!(lines.is_some());
        let lines = lines.unwrap();
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0], "Line 3");
        assert_eq!(lines[1], "Line 4");
        assert_eq!(lines[2], "Line 5");
    }

    #[test]
    fn test_pty_muxbox_scrollback_line_count() {
        let pty_manager = PtyManager::new().unwrap();
        let muxbox = create_test_pty_muxbox();

        // Create buffer with known number of lines
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));
        {
            let mut buf = buffer.lock().unwrap();
            for i in 1..=15 {
                buf.push(format!("Line {}", i));
            }
        }

        // Add PTY process using test helper
        pty_manager.add_test_pty_process(muxbox.id.clone(), buffer.clone());

        // Test line count
        let count = muxbox.get_scrollback_line_count(&pty_manager);
        assert_eq!(count, 15);
    }

    #[test]
    fn test_regular_muxbox_scrollback_fallback() {
        let pty_manager = PtyManager::new().unwrap();
        let mut muxbox = create_test_regular_muxbox();
        muxbox.content = Some("Line 1\nLine 2\nLine 3\nLine 4".to_string());

        // Test fallback to regular content for non-PTY muxboxes
        let content = muxbox.get_scrollback_content(&pty_manager);
        assert!(content.is_some());
        assert_eq!(content.unwrap(), "Line 1\nLine 2\nLine 3\nLine 4");

        // Test line range for regular muxboxes
        let lines = muxbox.get_scrollback_lines(&pty_manager, 1, 2);
        assert!(lines.is_some());
        let lines = lines.unwrap();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "Line 2");
        assert_eq!(lines[1], "Line 3");

        // Test line count for regular muxboxes
        let count = muxbox.get_scrollback_line_count(&pty_manager);
        assert_eq!(count, 4);
    }

    #[test]
    fn test_pty_scrollback_with_circular_buffer_overflow() {
        let pty_manager = PtyManager::new().unwrap();
        let muxbox = create_test_pty_muxbox();

        // Create small buffer to test overflow behavior
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(5)));
        {
            let mut buf = buffer.lock().unwrap();
            // Add more lines than buffer capacity
            for i in 1..=10 {
                buf.push(format!("Line {}", i));
            }
        }

        // Add PTY process using test helper
        pty_manager.add_test_pty_process(muxbox.id.clone(), buffer.clone());

        // Should only have last 5 lines due to circular buffer limit
        let count = muxbox.get_scrollback_line_count(&pty_manager);
        assert_eq!(count, 5);

        let content = muxbox.get_scrollback_content(&pty_manager);
        assert!(content.is_some());
        // Should contain only the last 5 lines (6-10)
        assert_eq!(content.unwrap(), "Line 6\nLine 7\nLine 8\nLine 9\nLine 10");
    }

    #[test]
    fn test_pty_scrollback_empty_buffer() {
        let pty_manager = PtyManager::new().unwrap();
        let muxbox = create_test_pty_muxbox();

        // Create empty buffer
        let buffer = Arc::new(Mutex::new(CircularBuffer::new(100)));

        // Add PTY process using test helper
        pty_manager.add_test_pty_process(muxbox.id.clone(), buffer.clone());

        // Test empty buffer handling
        let content = muxbox.get_scrollback_content(&pty_manager);
        assert!(content.is_some());
        assert_eq!(content.unwrap(), "");

        let count = muxbox.get_scrollback_line_count(&pty_manager);
        assert_eq!(count, 0);

        let lines = muxbox.get_scrollback_lines(&pty_manager, 0, 5);
        assert!(lines.is_none());
    }

    #[test]
    fn test_pty_scrollback_non_existent_muxbox() {
        let pty_manager = PtyManager::new().unwrap();
        let muxbox = create_test_pty_muxbox();

        // Don't add the muxbox to PTY manager

        // Test handling of non-existent PTY muxbox
        let content = muxbox.get_scrollback_content(&pty_manager);
        assert!(content.is_none());

        let count = muxbox.get_scrollback_line_count(&pty_manager);
        assert_eq!(count, 0);

        let lines = muxbox.get_scrollback_lines(&pty_manager, 0, 5);
        assert!(lines.is_none());
    }

    // Helper functions
    fn create_test_pty_muxbox() -> MuxBox {
        MuxBox {
            id: "test_pty_muxbox".to_string(),
            title: Some("Test PTY MuxBox".to_string()),
            position: crate::model::common::InputBounds {
                x1: "0%".to_string(),
                y1: "0%".to_string(),
                x2: "100%".to_string(),
                y2: "100%".to_string(),
            },
            anchor: crate::model::common::Anchor::TopLeft,
            min_width: None,
            min_height: None,
            max_width: None,
            max_height: None,
            overflow_behavior: None,
            refresh_interval: None,
            tab_order: None,
            next_focus_id: None,
            children: None,
            fill: None,
            fill_char: None,
            selected_fill_char: None,
            border: None,
            border_color: None,
            selected_border_color: None,
            bg_color: None,
            selected_bg_color: None,
            fg_color: None,
            selected_fg_color: None,
            title_fg_color: None,
            title_bg_color: None,
            selected_title_bg_color: None,
            selected_title_fg_color: None,
            title_position: None,
            error_border_color: None,
            error_bg_color: None,
            error_fg_color: None,
            error_title_bg_color: None,
            error_title_fg_color: None,
            error_selected_border_color: None,
            error_selected_bg_color: None,
            error_selected_fg_color: None,
            error_selected_title_bg_color: None,
            error_selected_title_fg_color: None,
            choices: None,
            menu_fg_color: None,
            menu_bg_color: None,
            selected_menu_fg_color: None,
            selected_menu_bg_color: None,
            redirect_output: None,
            append_output: None,
            script: None,
            thread: None,
            on_keypress: None,
            variables: None,
            horizontal_scroll: None,
            vertical_scroll: None,
            selected: None,
            content: None,
            save_in_file: None,
            chart_type: None,
            chart_data: None,
            plugin_component: None,
            plugin_config: None,
            table_data: None,
            table_config: None,
            auto_scroll_bottom: None,
            pty: Some(true), // This is a PTY muxbox
            z_index: None,
            output: String::new(),
            parent_id: None,
            parent_layout_id: None,
            error_state: false,
            scroll_x: 0,
            scroll_y: 0,
        }
    }

    fn create_test_regular_muxbox() -> MuxBox {
        let mut muxbox = create_test_pty_muxbox();
        muxbox.id = "test_regular_muxbox".to_string();
        muxbox.pty = Some(false); // This is not a PTY muxbox
        muxbox
    }
}
