#[cfg(test)]
mod line_attribute_tests {
    use crate::ansi_processor::{AnsiProcessor, LineAttribute, TerminalState};

    /// F0314: Test line attribute initialization (all normal)
    #[test]
    fn test_line_attribute_initialization() {
        let terminal_state = TerminalState::new(80, 24);

        // Check that all lines start with Normal attribute
        for line in 0..24 {
            assert_eq!(
                terminal_state.get_line_attribute(line),
                LineAttribute::Normal
            );
        }
    }

    /// F0314: Test setting double-width line with DECDWL (ESC # 6)
    #[test]
    fn test_decdwl_double_width_line() {
        let mut processor = AnsiProcessor::new();

        // Move cursor to line 5
        processor.terminal_state.get_cursor_mut().y = 5;

        // Set double-width line using ESC # 6
        processor.process_bytes(b"\x1b#6");

        // Line 5 should now be double-width
        assert_eq!(
            processor.terminal_state.get_line_attribute(5),
            LineAttribute::DoubleWidth
        );

        // Other lines should remain normal
        assert_eq!(
            processor.terminal_state.get_line_attribute(4),
            LineAttribute::Normal
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(6),
            LineAttribute::Normal
        );
    }

    /// F0314: Test setting double-height line top half with DECDHL (ESC # 3)
    #[test]
    fn test_decdhl_double_height_line_top() {
        let mut processor = AnsiProcessor::new();

        // Move cursor to line 8
        processor.terminal_state.get_cursor_mut().y = 8;

        // Set double-height line top half using ESC # 3
        processor.process_bytes(b"\x1b#3");

        // Line 8 should now be double-height top
        assert_eq!(
            processor.terminal_state.get_line_attribute(8),
            LineAttribute::DoubleHeightTop
        );

        // Other lines should remain normal
        assert_eq!(
            processor.terminal_state.get_line_attribute(7),
            LineAttribute::Normal
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(9),
            LineAttribute::Normal
        );
    }

    /// F0314: Test setting double-height line bottom half with DECDHL (ESC # 4)
    #[test]
    fn test_decdhl_double_height_line_bottom() {
        let mut processor = AnsiProcessor::new();

        // Move cursor to line 12
        processor.terminal_state.get_cursor_mut().y = 12;

        // Set double-height line bottom half using ESC # 4
        processor.process_bytes(b"\x1b#4");

        // Line 12 should now be double-height bottom
        assert_eq!(
            processor.terminal_state.get_line_attribute(12),
            LineAttribute::DoubleHeightBottom
        );

        // Other lines should remain normal
        assert_eq!(
            processor.terminal_state.get_line_attribute(11),
            LineAttribute::Normal
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(13),
            LineAttribute::Normal
        );
    }

    /// F0314: Test resetting line to normal with DECSWL (ESC # 5)
    #[test]
    fn test_decswl_reset_to_normal() {
        let mut processor = AnsiProcessor::new();

        // Move cursor to line 10
        processor.terminal_state.get_cursor_mut().y = 10;

        // Set double-width first
        processor.process_bytes(b"\x1b#6");
        assert_eq!(
            processor.terminal_state.get_line_attribute(10),
            LineAttribute::DoubleWidth
        );

        // Reset to normal using ESC # 5
        processor.process_bytes(b"\x1b#5");

        // Line 10 should now be normal
        assert_eq!(
            processor.terminal_state.get_line_attribute(10),
            LineAttribute::Normal
        );
    }

    /// F0314: Test line attribute sequence on double-height line pair
    #[test]
    fn test_double_height_line_pair() {
        let mut processor = AnsiProcessor::new();

        // Set up a double-height line pair on lines 15 and 16
        processor.terminal_state.get_cursor_mut().y = 15;
        processor.process_bytes(b"\x1b#3"); // Top half

        processor.terminal_state.get_cursor_mut().y = 16;
        processor.process_bytes(b"\x1b#4"); // Bottom half

        // Check both lines have correct attributes
        assert_eq!(
            processor.terminal_state.get_line_attribute(15),
            LineAttribute::DoubleHeightTop
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(16),
            LineAttribute::DoubleHeightBottom
        );

        // Surrounding lines should be normal
        assert_eq!(
            processor.terminal_state.get_line_attribute(14),
            LineAttribute::Normal
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(17),
            LineAttribute::Normal
        );
    }

    /// F0314: Test line attributes in alternate screen buffer
    #[test]
    fn test_line_attributes_alternate_screen() {
        let mut processor = AnsiProcessor::new();

        // Switch to alternate screen
        processor.terminal_state.use_alternate_screen = true;

        // Move cursor and set double-width in alternate screen
        processor.terminal_state.get_cursor_mut().y = 7;
        processor.process_bytes(b"\x1b#6");

        // Alternate screen should have the attribute
        assert_eq!(
            processor.terminal_state.alternate_buffer.line_attributes[7],
            LineAttribute::DoubleWidth
        );

        // Primary screen should still be normal
        assert_eq!(
            processor.terminal_state.primary_buffer.line_attributes[7],
            LineAttribute::Normal
        );
    }

    /// F0314: Test line attributes persistence during scrolling
    #[test]
    fn test_line_attributes_with_scrolling() {
        let mut processor = AnsiProcessor::new();

        // Set double-width on line 0
        processor.terminal_state.get_cursor_mut().y = 0;
        processor.process_bytes(b"\x1b#6");
        assert_eq!(
            processor.terminal_state.get_line_attribute(0),
            LineAttribute::DoubleWidth
        );

        // Scroll up - this should move line 0 to scrollback
        processor.terminal_state.primary_buffer.scroll_up_one_line();

        // Line 0 should now be normal (new empty line)
        assert_eq!(
            processor.terminal_state.get_line_attribute(0),
            LineAttribute::Normal
        );

        // The scrollback should have the double-width attribute
        assert_eq!(
            processor
                .terminal_state
                .primary_buffer
                .scrollback_line_attributes
                .last(),
            Some(&LineAttribute::DoubleWidth)
        );
    }

    /// F0314: Test line attribute resize behavior
    #[test]
    fn test_line_attributes_after_resize() {
        let mut processor = AnsiProcessor::new();

        // Set some line attributes
        processor.terminal_state.get_cursor_mut().y = 5;
        processor.process_bytes(b"\x1b#6"); // Double width

        processor.terminal_state.get_cursor_mut().y = 10;
        processor.process_bytes(b"\x1b#3"); // Double height top

        // Resize to larger height
        processor.terminal_state.resize(80, 30);

        // Existing attributes should be preserved
        assert_eq!(
            processor.terminal_state.get_line_attribute(5),
            LineAttribute::DoubleWidth
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(10),
            LineAttribute::DoubleHeightTop
        );

        // New lines should be normal
        assert_eq!(
            processor.terminal_state.get_line_attribute(25),
            LineAttribute::Normal
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(29),
            LineAttribute::Normal
        );

        // Resize to smaller height
        processor.terminal_state.resize(80, 15);

        // Only preserved attributes should remain
        assert_eq!(
            processor.terminal_state.get_line_attribute(5),
            LineAttribute::DoubleWidth
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(10),
            LineAttribute::DoubleHeightTop
        );

        // Should have exactly 15 line attributes
        assert_eq!(
            processor
                .terminal_state
                .primary_buffer
                .line_attributes
                .len(),
            15
        );
    }

    /// F0314: Test multiple line attribute changes on same line
    #[test]
    fn test_multiple_attribute_changes() {
        let mut processor = AnsiProcessor::new();

        // Move cursor to line 3
        processor.terminal_state.get_cursor_mut().y = 3;

        // Start with double-width
        processor.process_bytes(b"\x1b#6");
        assert_eq!(
            processor.terminal_state.get_line_attribute(3),
            LineAttribute::DoubleWidth
        );

        // Change to double-height top
        processor.process_bytes(b"\x1b#3");
        assert_eq!(
            processor.terminal_state.get_line_attribute(3),
            LineAttribute::DoubleHeightTop
        );

        // Change to double-height bottom
        processor.process_bytes(b"\x1b#4");
        assert_eq!(
            processor.terminal_state.get_line_attribute(3),
            LineAttribute::DoubleHeightBottom
        );

        // Reset to normal
        processor.process_bytes(b"\x1b#5");
        assert_eq!(
            processor.terminal_state.get_line_attribute(3),
            LineAttribute::Normal
        );
    }

    /// F0314: Test line attribute edge cases
    #[test]
    fn test_line_attribute_edge_cases() {
        let mut processor = AnsiProcessor::new();

        // Test at first line (y=0)
        processor.terminal_state.get_cursor_mut().y = 0;
        processor.process_bytes(b"\x1b#6");
        assert_eq!(
            processor.terminal_state.get_line_attribute(0),
            LineAttribute::DoubleWidth
        );

        // Test at last line (y=23)
        processor.terminal_state.get_cursor_mut().y = 23;
        processor.process_bytes(b"\x1b#3");
        assert_eq!(
            processor.terminal_state.get_line_attribute(23),
            LineAttribute::DoubleHeightTop
        );

        // Test invalid line requests return Normal
        assert_eq!(
            processor.terminal_state.get_line_attribute(100),
            LineAttribute::Normal
        );
    }

    /// F0314: Test line attributes with terminal modes
    #[test]
    fn test_line_attributes_with_terminal_modes() {
        let mut processor = AnsiProcessor::new();

        // Set line attribute in origin mode
        processor.terminal_state.mode.origin_mode = true;
        processor.terminal_state.get_cursor_mut().y = 5;
        processor.process_bytes(b"\x1b#6");

        // Should work regardless of terminal mode
        assert_eq!(
            processor.terminal_state.get_line_attribute(5),
            LineAttribute::DoubleWidth
        );

        // Reset origin mode
        processor.terminal_state.mode.origin_mode = false;

        // Line attribute should still be there
        assert_eq!(
            processor.terminal_state.get_line_attribute(5),
            LineAttribute::DoubleWidth
        );
    }

    /// F0314: Test comprehensive line attribute integration
    #[test]
    fn test_comprehensive_line_attribute_integration() {
        let mut processor = AnsiProcessor::new();

        // Create a pattern of different line attributes
        let test_patterns = [
            (2, b"\x1b#6", LineAttribute::DoubleWidth),
            (4, b"\x1b#3", LineAttribute::DoubleHeightTop),
            (5, b"\x1b#4", LineAttribute::DoubleHeightBottom),
            (7, b"\x1b#6", LineAttribute::DoubleWidth),
            (10, b"\x1b#3", LineAttribute::DoubleHeightTop),
            (11, b"\x1b#4", LineAttribute::DoubleHeightBottom),
        ];

        // Set all patterns
        for &(line, sequence, expected) in &test_patterns {
            processor.terminal_state.get_cursor_mut().y = line;
            processor.process_bytes(sequence);
            assert_eq!(processor.terminal_state.get_line_attribute(line), expected);
        }

        // Verify all other lines are normal
        for line in 0..24 {
            if !test_patterns
                .iter()
                .any(|(test_line, _, _)| *test_line == line)
            {
                assert_eq!(
                    processor.terminal_state.get_line_attribute(line),
                    LineAttribute::Normal
                );
            }
        }

        // Test reset of one line
        processor.terminal_state.get_cursor_mut().y = 7;
        processor.process_bytes(b"\x1b#5"); // Reset to normal
        assert_eq!(
            processor.terminal_state.get_line_attribute(7),
            LineAttribute::Normal
        );

        // Other attributes should remain unchanged
        assert_eq!(
            processor.terminal_state.get_line_attribute(2),
            LineAttribute::DoubleWidth
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(4),
            LineAttribute::DoubleHeightTop
        );
        assert_eq!(
            processor.terminal_state.get_line_attribute(5),
            LineAttribute::DoubleHeightBottom
        );
    }
}
