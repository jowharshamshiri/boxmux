use crate::ansi_processor::AnsiProcessor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osc_terminal_title_sequence() {
        let mut processor = AnsiProcessor::new();

        // Test OSC 0 - Set both icon and title
        processor.process_bytes("\x1b]0;Test Title\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("Test Title".to_string())
        );

        // Test OSC 2 - Set window title only
        processor.process_bytes("\x1b]2;Window Title\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("Window Title".to_string())
        );
    }

    #[test]
    fn test_osc_icon_name_sequence() {
        let mut processor = AnsiProcessor::new();

        // Test OSC 1 - Set icon name
        processor.process_bytes("\x1b]1;Icon Name\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.icon_name,
            Some("Icon Name".to_string())
        );

        // Test OSC 0 also sets icon name
        processor.process_bytes("\x1b]0;Both Title\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("Both Title".to_string())
        );
        // Icon name should remain from previous OSC 1
        assert_eq!(
            processor.terminal_state.icon_name,
            Some("Icon Name".to_string())
        );
    }

    #[test]
    fn test_osc_sequence_termination_methods() {
        let mut processor = AnsiProcessor::new();

        // Test BEL termination (\x07)
        processor.process_bytes("\x1b]2;BEL Terminated\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("BEL Terminated".to_string())
        );

        // Test ST termination (\x1b\\)
        processor.process_bytes("\x1b]2;ST Terminated\x1b\\".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("ST Terminated".to_string())
        );
    }

    #[test]
    fn test_osc_sequence_with_special_characters() {
        let mut processor = AnsiProcessor::new();

        // Test title with spaces and special characters
        processor.process_bytes("\x1b]2;Title with spaces & symbols!\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("Title with spaces & symbols!".to_string())
        );

        // Test empty title
        processor.process_bytes("\x1b]2;\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("".to_string())
        );
    }

    #[test]
    fn test_osc_sequence_with_unicode() {
        let mut processor = AnsiProcessor::new();

        // Test Unicode characters in title
        processor.process_bytes("\x1b]2;测试标题 🚀 Test\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("测试标题 🚀 Test".to_string())
        );
    }

    #[test]
    fn test_osc_unknown_commands() {
        let mut processor = AnsiProcessor::new();

        // Test unknown OSC command (should be logged but not crash)
        let original_title = processor.terminal_state.terminal_title.clone();
        processor.process_bytes("\x1b]999;Unknown Command\x07".as_bytes());

        // Title should remain unchanged
        assert_eq!(processor.terminal_state.terminal_title, original_title);
    }

    #[test]
    fn test_osc_malformed_sequences() {
        let mut processor = AnsiProcessor::new();

        // Test OSC without semicolon
        let original_title = processor.terminal_state.terminal_title.clone();
        processor.process_bytes("\x1b]2NoSemicolon\x07".as_bytes());

        // Should handle gracefully without changing title
        assert_eq!(processor.terminal_state.terminal_title, original_title);

        // Test OSC with multiple semicolons (should use first part)
        processor.process_bytes("\x1b]2;First;Second;Third\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("First;Second;Third".to_string())
        );
    }

    #[test]
    fn test_osc_color_palette_sequences() {
        let mut processor = AnsiProcessor::new();

        // Test OSC 4 - Set color palette (should be logged, not stored)
        let original_title = processor.terminal_state.terminal_title.clone();
        processor.process_bytes("\x1b]4;0;rgb:00/00/00\x07".as_bytes());

        // Title should remain unchanged
        assert_eq!(processor.terminal_state.terminal_title, original_title);
    }

    #[test]
    fn test_osc_clipboard_sequences() {
        let mut processor = AnsiProcessor::new();

        // Test OSC 52 - Clipboard operations (should be logged for security)
        let original_title = processor.terminal_state.terminal_title.clone();
        processor.process_bytes("\x1b]52;c;dGVzdCBkYXRh\x07".as_bytes());

        // Title should remain unchanged
        assert_eq!(processor.terminal_state.terminal_title, original_title);
    }

    #[test]
    fn test_multiple_osc_sequences() {
        let mut processor = AnsiProcessor::new();

        // Test multiple OSC sequences in sequence
        processor.process_bytes("\x1b]1;First Icon\x07".as_bytes());
        processor.process_bytes("\x1b]2;First Title\x07".as_bytes());

        assert_eq!(
            processor.terminal_state.icon_name,
            Some("First Icon".to_string())
        );
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("First Title".to_string())
        );

        // Update both
        processor.process_bytes("\x1b]0;Updated Both\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("Updated Both".to_string())
        );
    }

    #[test]
    fn test_osc_long_titles() {
        let mut processor = AnsiProcessor::new();

        // Test very long title (should handle gracefully)
        let long_title = "A".repeat(1000);
        let osc_sequence = format!("\x1b]2;{}\x07", long_title);
        processor.process_bytes(osc_sequence.as_bytes());

        assert_eq!(processor.terminal_state.terminal_title, Some(long_title));
    }

    #[test]
    fn test_osc_with_control_characters() {
        let mut processor = AnsiProcessor::new();

        // Test title with embedded control characters (should be filtered out for security)
        processor.process_bytes("\x1b]2;Title\nwith\tcontrols\x07".as_bytes());
        assert_eq!(
            processor.terminal_state.terminal_title,
            Some("Titlewithcontrols".to_string())
        );
    }

    #[test]
    fn test_title_propagation_detection() {
        let mut processor = AnsiProcessor::new();

        // Initially no title change detected
        assert!(!processor.has_title_changed());

        // Set a terminal title
        processor.process_bytes("\x1b]2;New Title\x07".as_bytes());

        // Should detect title change
        assert!(processor.has_title_changed());

        // Get the title and mark as consumed
        let title = processor.get_and_consume_terminal_title();
        assert_eq!(title, Some("New Title".to_string()));

        // After consuming, should no longer detect change
        assert!(!processor.has_title_changed());
    }

    #[test]
    fn test_multiple_title_changes() {
        let mut processor = AnsiProcessor::new();

        // Set initial title
        processor.process_bytes("\x1b]2;Title 1\x07".as_bytes());
        assert!(processor.has_title_changed());
        assert_eq!(
            processor.get_current_terminal_title(),
            Some("Title 1".to_string())
        );

        // Consume the change
        let _ = processor.get_and_consume_terminal_title();
        assert!(!processor.has_title_changed());

        // Change title again
        processor.process_bytes("\x1b]0;Title 2\x07".as_bytes());
        assert!(processor.has_title_changed());
        assert_eq!(
            processor.get_current_terminal_title(),
            Some("Title 2".to_string())
        );

        // Consume the change
        let title = processor.get_and_consume_terminal_title();
        assert_eq!(title, Some("Title 2".to_string()));
        assert!(!processor.has_title_changed());
    }
}
