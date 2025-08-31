use crate::ansi_processor::AnsiProcessor;

#[test]
fn test_clear_screen_detection() {
    let mut processor = AnsiProcessor::new();

    // Before clear screen command
    assert!(!processor.should_replace_content());
    assert!(!processor.detect_full_screen_program());

    // Send clear screen command
    processor.process_string("\x1b[2J");

    // Should now detect full-screen program
    assert!(processor.should_replace_content());
    assert!(processor.detect_full_screen_program());
    assert!(processor.use_screen_buffer);
}

#[test]
fn test_cursor_home_detection() {
    let mut processor = AnsiProcessor::new();

    // Before cursor home command
    assert!(!processor.should_replace_content());

    // Send cursor home command (common in full-screen apps)
    processor.process_string("\x1b[1;1H");

    // Should detect full-screen program
    assert!(processor.should_replace_content());
    assert!(processor.terminal_state.full_screen_program_detected);
}

#[test]
fn test_alternate_screen_detection() {
    let mut processor = AnsiProcessor::new();

    // Before alternate screen buffer command
    assert!(!processor.alternate_screen_mode);
    assert!(!processor.should_replace_content());

    // Send alternate screen buffer command (definitive full-screen indicator)
    processor.process_string("\x1b[?1049h");

    // Should definitely detect full-screen program
    assert!(processor.alternate_screen_mode);
    assert!(processor.should_replace_content());
    assert!(processor.terminal_state.full_screen_program_detected);
}

#[test]
fn test_line_based_program() {
    let mut processor = AnsiProcessor::new();

    // Send normal text without screen control commands
    processor.process_string("Line 1\nLine 2\nLine 3\n");

    // Should remain in line-based mode
    assert!(!processor.should_replace_content());
    assert!(!processor.terminal_state.full_screen_program_detected);
    assert!(!processor.use_screen_buffer);

    // Content should be in processed text
    assert_eq!(processor.get_processed_text(), "Line 1\nLine 2\nLine 3\n");
}

#[test]
fn test_screen_content_vs_processed_text() {
    let mut processor = AnsiProcessor::with_screen_size(10, 5);

    // Send clear screen and cursor positioning (full-screen mode)
    processor.process_string("\x1b[2J\x1b[1;1HTop\x1b[3;1HMiddle\x1b[5;1HBottom");

    // Should be in screen buffer mode
    assert!(processor.should_replace_content());

    // Screen content should show positioned text
    let screen_content = processor.get_screen_content();
    assert!(screen_content.contains("Top"));
    assert!(screen_content.contains("Middle"));
    assert!(screen_content.contains("Bottom"));

    // Lines should be at correct positions (newlines separate rows)
    let lines: Vec<&str> = screen_content.lines().collect();
    assert_eq!(lines[0].trim(), "Top"); // Row 1
    assert_eq!(lines[2].trim(), "Middle"); // Row 3
    assert_eq!(lines[4].trim(), "Bottom"); // Row 5
}
