use super::*;

#[test]
fn a_sequence_split_between_two_reads_is_still_understood() {
    let mut processor = Emulator::with_screen_size(20, 3);
    processor.process_bytes(b"a\x1b[3");
    processor.process_bytes(b"1mb\x1b[0mc");
    assert_eq!(processor.get_processed_text(), "abc");
    let line = &processor.display_lines()[0];
    assert!(line.contains("\x1b[0;31mb"), "{:?}", line);
}

#[test]
fn writing_the_last_column_waits_for_the_next_character_to_wrap() {
    let mut processor = Emulator::with_screen_size(4, 3);
    // A full line followed by CR LF must not leave an empty line.
    processor.process_string("abcd\r\nefgh\r\n");
    assert_eq!(processor.get_processed_text(), "abcd\nefgh");
    // A character after a full line goes to the next line.
    let mut processor = Emulator::with_screen_size(4, 3);
    processor.process_string("abcdX");
    assert_eq!(processor.get_processed_text(), "abcd\nX");
    // Moving the cursor cancels the pending wrap.
    let mut processor = Emulator::with_screen_size(4, 3);
    processor.process_string("abcd\x1b[1;1HZ");
    assert_eq!(processor.get_processed_text(), "Zbcd");
}

#[test]
fn wide_characters_take_two_cells_and_wrap_as_a_whole() {
    let mut processor = Emulator::with_screen_size(5, 3);
    processor.process_string("日本語");
    // Two wide characters fill four columns; the third does not fit in the last column.
    assert_eq!(processor.get_processed_text(), "日本\n語");
    assert_eq!((processor.get_cursor_x(), processor.get_cursor_y()), (2, 1));
    let lines = processor.display_lines();
    assert_eq!(crate::utils::display_width(&lines[0]), 5, "{:?}", lines[0]);
}

#[test]
fn truecolor_and_palette_colors_are_kept_exactly() {
    let mut processor = Emulator::with_screen_size(10, 2);
    processor.process_string("\x1b[38;2;10;20;30;48;5;200mx");
    let cell = &processor.terminal_state.get_active_screen_ref()[0][0];
    assert_eq!(cell.fg_color, Some(TermColor::Rgb(10, 20, 30)));
    assert_eq!(cell.bg_color, Some(TermColor::Indexed(200)));
    assert!(processor.display_lines()[0].contains("38;2;10;20;30;48;5;200"));
}

#[test]
fn backspace_moves_left_without_erasing() {
    let mut processor = Emulator::with_screen_size(10, 2);
    processor.process_string("abc\x08\x08");
    assert_eq!(processor.get_processed_text(), "abc");
    assert_eq!(processor.get_cursor_x(), 1);
    processor.process_string("X");
    assert_eq!(processor.get_processed_text(), "aXc");
}

#[test]
fn erasing_to_the_cursor_clears_the_start_of_the_screen() {
    let mut processor = Emulator::with_screen_size(4, 3);
    processor.process_string("abcd\r\nefgh\r\nijkl\x1b[2;2H\x1b[1J");
    assert_eq!(processor.get_processed_text(), "\n  gh\nijkl");
}

#[test]
fn programs_can_write_the_clipboard_but_not_read_it() {
    let mut processor = Emulator::with_screen_size(10, 2);
    processor.process_string("\x1b]52;c;aGVsbG8gd29ybGQ=\x07\x1b]52;c;?\x07");
    assert_eq!(processor.take_clipboard_writes(), vec!["hello world"]);
    assert!(processor.take_responses().is_empty(), "nothing is revealed");
    assert!(processor.take_clipboard_writes().is_empty());
}

#[test]
fn a_full_reset_returns_to_the_initial_state() {
    let mut processor = Emulator::with_screen_size(10, 3);
    processor.process_string("\x1b[?1h\x1b[31mtext\x1bc");
    assert_eq!(processor.get_processed_text(), "");
    assert!(!processor.terminal_state.mode.cursor_key_mode);
    assert_eq!(
        processor.terminal_state.current_attributes,
        TerminalCell::default()
    );
}

#[test]
fn test_emulator_basic_text() {
    let mut processor = Emulator::new();
    processor.process_string("Hello World");
    assert_eq!(processor.get_processed_text(), "Hello World");
}

#[test]
fn test_emulator_color_codes() {
    let mut processor = Emulator::new();
    processor.process_string("\x1b[31mRed Text\x1b[0m");
    assert_eq!(processor.get_processed_text(), "Red Text");
}

#[test]
fn test_emulator_newlines() {
    let mut processor = Emulator::new();
    // A PTY turns each newline a program prints into CR LF.
    processor.process_string("Line 1\r\nLine 2\r\n");
    assert_eq!(processor.get_processed_text(), "Line 1\nLine 2");
    assert_eq!(processor.get_cursor_y(), 2);
    assert_eq!(processor.get_cursor_x(), 0);
    // A bare line feed moves down without returning to the first column.
    let mut processor = Emulator::new();
    processor.process_string("ab\ncd");
    assert_eq!(processor.get_processed_text(), "ab\n  cd");
}

#[test]
fn test_emulator_clear_screen() {
    let mut processor = Emulator::new();
    processor.process_string("Some text\x1b[2JCleared");
    assert_eq!(processor.get_processed_text(), "Cleared");
}

#[test]
fn test_terminal_screen_buffer_to_content_lines() {
    let mut buffer = TerminalScreenBuffer::new(80, 24, 1000);

    // Add some content with formatting
    buffer.content[0][0] = TerminalCell {
        character: 'H',
        fg_color: Some(TermColor::Indexed(1)),
        bold: true,
        ..Default::default()
    };
    buffer.content[0][1] = TerminalCell {
        character: 'i',
        fg_color: Some(TermColor::Indexed(1)),
        bold: true,
        ..Default::default()
    };

    let lines = buffer.to_content_lines(false);
    assert!(!lines.is_empty());
    let first_line = &lines[0];

    // Should contain ANSI escape codes for formatting
    assert!(first_line.contains("Hi"));
    assert!(first_line.contains("\x1b[")); // ANSI escape sequence
}

#[test]
fn test_enhanced_vte_parser_integration() {
    let mut processor = Emulator::new();

    // Test application keypad mode (ESC sequences)
    processor.process_string("\x1b="); // ESC =
    assert!(processor.terminal_state.mode.keypad_mode);

    processor.process_string("\x1b>"); // ESC >
    assert!(!processor.terminal_state.mode.keypad_mode);

    // Test device status report (CSI sequence)
    processor.process_string("\x1b[6n");
    let responses = processor.take_responses();
    assert_eq!(responses.len(), 1, "one reply per query");
    let response = &responses[0];
    assert!(response.contains("R")); // Cursor position report

    // Test scrolling region (CSI sequence)
    processor.process_string("\x1b[5;20r"); // Set scroll region lines 5-20
    assert_eq!(processor.terminal_state.scroll_region.top, 4); // 0-indexed
    assert_eq!(processor.terminal_state.scroll_region.bottom, 19); // 0-indexed

    // Test enhanced cursor movements
    processor.process_string("\x1b[10A"); // Cursor up 10 lines
    processor.process_string("\x1b[5B"); // Cursor down 5 lines
    processor.process_string("\x1b[3C"); // Cursor forward 3 columns
    processor.process_string("\x1b[2D"); // Cursor backward 2 columns

    // Test that processor handles enhanced sequences without crashing
    assert_eq!(processor.terminal_state.scroll_region.top, 4);
    assert_eq!(processor.terminal_state.scroll_region.bottom, 19);
}

#[test]
fn test_vte_parser_dcs_sequences() {
    let mut processor = Emulator::new();

    // Test DCS sequence processing
    processor.process_string("\x1bP1$q\x1b\\"); // DCS query sequence

    // Verify DCS state was cleared
    assert!(processor.terminal_state.dcs_sequence_type.is_none());
    assert!(processor.terminal_state.dcs_buffer.is_none());
}

#[test]
fn test_f0311_terminal_queries() {
    let mut processor = Emulator::new();

    // Test Primary Device Attributes via VTE parser
    processor.process_bytes("\x1b[c".as_bytes());
    let mut responses = processor.take_responses();
    assert_eq!(responses.len(), 1, "one reply per query");
    let response = responses.remove(0);
    assert!(response.contains("[?62")); // VT220 compatible terminal

    // Test Device Status Report - cursor position
    processor.process_bytes("\x1b[6n".as_bytes());
    let mut responses = processor.take_responses();
    assert_eq!(responses.len(), 1, "one reply per query");
    let response = responses.remove(0);
    assert!(response.ends_with("R")); // Cursor Position Report format

    // Test Secondary Device Attributes
    processor.process_bytes("\x1b[1c".as_bytes());
    let mut responses = processor.take_responses();
    assert_eq!(responses.len(), 1, "one reply per query");
    let response = responses.remove(0);
    assert_eq!(response, "\x1b[>0;276;0c"); // Secondary DA format

    // Test Terminal Status Report
    processor.process_bytes("\x1b[5n".as_bytes());
    let mut responses = processor.take_responses();
    assert_eq!(responses.len(), 1, "one reply per query");
    let response = responses.remove(0);
    assert_eq!(response, "\x1b[0n"); // Terminal OK
}

#[test]
fn test_f0312_alternate_screen_buffer() {
    let mut processor = Emulator::new();

    // Test initial state - should be on primary screen
    assert!(!processor.is_using_alternate_screen());
    assert!(!processor.is_using_alternate_screen());

    // Add some content to primary screen
    processor.process_string("Primary screen content");
    let primary_content = processor.get_processed_text();
    assert!(primary_content.contains("Primary"));

    // Test Mode 47 - Simple Alternate Screen
    processor.process_bytes("\x1b[?47h".as_bytes());
    assert!(processor.is_using_alternate_screen());
    assert!(processor.is_using_alternate_screen());

    // Add content to alternate screen
    processor.process_string("Alternate screen content");
    assert!(processor
        .get_processed_text()
        .contains("Alternate screen content"));

    // Exit Mode 47
    processor.process_bytes("\x1b[?47l".as_bytes());
    assert!(!processor.is_using_alternate_screen());

    // The primary screen shows again, without the alternate screen's text.
    assert!(!processor
        .get_processed_text()
        .contains("Alternate screen content"));

    // Test Mode 1047 - Alternate Screen with Clear
    processor.process_bytes("\x1b[?1047h".as_bytes());
    assert!(processor.is_using_alternate_screen());

    // Exit Mode 1047
    processor.process_bytes("\x1b[?1047l".as_bytes());
    assert!(!processor.is_using_alternate_screen());

    // Test Mode 1049 - Alternate Screen with Cursor Save/Restore
    // Set cursor position first
    processor.terminal_state.set_cursor_position(10, 5);
    let saved_x = processor.terminal_state.get_cursor().x;
    let saved_y = processor.terminal_state.get_cursor().y;

    processor.process_bytes("\x1b[?1049h".as_bytes());
    assert!(processor.is_using_alternate_screen());

    // Move cursor in alternate screen
    processor.terminal_state.set_cursor_position(20, 15);
    assert_ne!(processor.terminal_state.get_cursor().x, saved_x);
    assert_ne!(processor.terminal_state.get_cursor().y, saved_y);

    // Exit Mode 1049 - should restore cursor position
    processor.process_bytes("\x1b[?1049l".as_bytes());
    assert!(!processor.is_using_alternate_screen());

    // Verify cursor was restored
    assert_eq!(processor.terminal_state.get_cursor().x, saved_x);
    assert_eq!(processor.terminal_state.get_cursor().y, saved_y);
}

#[test]
fn test_f0312_buffer_isolation() {
    let mut processor = Emulator::new();

    // Enable screen buffer mode for proper isolation testing

    // Add content to primary screen
    processor.process_string("Primary line 1\n");
    processor.process_string("Primary line 2\n");

    // Get primary screen content
    let primary_lines = processor.display_lines();
    assert!(primary_lines.len() >= 2);
    assert!(primary_lines[0].contains("Primary line 1"));

    // Switch to alternate screen with clear
    processor.process_bytes("\x1b[?1047h".as_bytes());
    assert!(processor.is_using_alternate_screen());

    // Add different content to alternate screen
    processor.process_string("Alternate line 1\n");
    processor.process_string("Alternate line 2\n");

    // Get alternate screen content
    let alt_lines = processor.display_lines();
    assert!(alt_lines.len() >= 2);
    assert!(alt_lines[0].contains("Alternate line 1"));

    // Switch back to primary screen
    processor.process_bytes("\x1b[?1047l".as_bytes());
    assert!(!processor.is_using_alternate_screen());

    // The primary screen's content comes back without the alternate screen's.
    let restored_lines = processor.display_lines().join("\n");
    assert!(
        !restored_lines.contains("Alternate line 1"),
        "{}",
        restored_lines
    );

    // Verify buffer isolation is working
    assert!(processor.terminal_state.verify_buffer_isolation());
}

#[test]
fn test_cursor_management_system() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Test basic cursor positioning
    terminal_state.set_cursor_position(10, 5);
    assert_eq!(terminal_state.get_cursor().x, 10);
    assert_eq!(terminal_state.get_cursor().y, 5);

    // Test bounds checking
    terminal_state.set_cursor_position(100, 30); // Beyond screen bounds
    assert_eq!(terminal_state.get_cursor().x, 79); // Clamped to screen width - 1
    assert_eq!(terminal_state.get_cursor().y, 23); // Clamped to screen height - 1

    // Test relative movement
    terminal_state.set_cursor_position(10, 10);
    terminal_state.move_cursor_relative(5, -3);
    assert_eq!(terminal_state.get_cursor().x, 15);
    assert_eq!(terminal_state.get_cursor().y, 7);

    // Test negative relative movement with bounds
    terminal_state.move_cursor_relative(-20, -10);
    assert_eq!(terminal_state.get_cursor().x, 0); // Saturated at 0
    assert_eq!(terminal_state.get_cursor().y, 0); // Saturated at 0
}

#[test]
fn test_cursor_save_restore() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Set initial position and style
    terminal_state.set_cursor_position(15, 10);
    terminal_state.set_cursor_visibility(false);
    terminal_state.set_cursor_style(CursorStyle::Underline);

    // Save cursor state
    terminal_state.save_cursor();

    // Change cursor state
    terminal_state.set_cursor_position(30, 20);
    terminal_state.set_cursor_visibility(true);
    terminal_state.set_cursor_style(CursorStyle::Block);

    // Verify changed state
    assert_eq!(terminal_state.get_cursor().x, 30);
    assert_eq!(terminal_state.get_cursor().y, 20);
    assert!(terminal_state.get_cursor().visible);
    assert_eq!(terminal_state.get_cursor().style, CursorStyle::Block);

    // Restore cursor state
    terminal_state.restore_cursor();

    // Verify restored state
    assert_eq!(terminal_state.get_cursor().x, 15);
    assert_eq!(terminal_state.get_cursor().y, 10);
    assert!(!terminal_state.get_cursor().visible);
    assert_eq!(terminal_state.get_cursor().style, CursorStyle::Underline);
}

#[test]
fn test_cursor_movement_operations() {
    let mut terminal_state = TerminalState::new(80, 24);
    terminal_state.set_cursor_position(10, 10);

    // Test directional movements
    terminal_state.cursor_up(3);
    assert_eq!(terminal_state.get_cursor().y, 7);

    terminal_state.cursor_down(5);
    assert_eq!(terminal_state.get_cursor().y, 12);

    terminal_state.cursor_left(4);
    assert_eq!(terminal_state.get_cursor().x, 6);

    terminal_state.cursor_right(8);
    assert_eq!(terminal_state.get_cursor().x, 14);

    // Test line operations
    terminal_state.cursor_to_line_start();
    assert_eq!(terminal_state.get_cursor().x, 0);
    assert_eq!(terminal_state.get_cursor().y, 12);

    terminal_state.cursor_next_line(2);
    assert_eq!(terminal_state.get_cursor().x, 0);
    assert_eq!(terminal_state.get_cursor().y, 14);

    terminal_state.set_cursor_position(20, 10);
    terminal_state.cursor_previous_line(3);
    assert_eq!(terminal_state.get_cursor().x, 0);
    assert_eq!(terminal_state.get_cursor().y, 7);
}

#[test]
fn test_cursor_column_line_positioning() {
    let mut terminal_state = TerminalState::new(80, 24);

    terminal_state.set_cursor_position(10, 10);

    // Test column positioning
    terminal_state.set_cursor_column(25);
    assert_eq!(terminal_state.get_cursor().x, 25);
    assert_eq!(terminal_state.get_cursor().y, 10); // Y unchanged

    // Test line positioning
    terminal_state.set_cursor_line(15);
    assert_eq!(terminal_state.get_cursor().x, 25); // X unchanged
    assert_eq!(terminal_state.get_cursor().y, 15);

    // Test bounds checking
    terminal_state.set_cursor_column(100);
    assert_eq!(terminal_state.get_cursor().x, 79); // Clamped

    terminal_state.set_cursor_line(50);
    assert_eq!(terminal_state.get_cursor().y, 23); // Clamped
}

#[test]
fn test_cursor_styles() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Test all cursor styles
    terminal_state.set_cursor_style(CursorStyle::Block);
    assert_eq!(terminal_state.get_cursor().style, CursorStyle::Block);

    terminal_state.set_cursor_style(CursorStyle::Underline);
    assert_eq!(terminal_state.get_cursor().style, CursorStyle::Underline);

    terminal_state.set_cursor_style(CursorStyle::Bar);
    assert_eq!(terminal_state.get_cursor().style, CursorStyle::Bar);

    terminal_state.set_cursor_style(CursorStyle::BlinkingBlock);
    assert_eq!(
        terminal_state.get_cursor().style,
        CursorStyle::BlinkingBlock
    );

    terminal_state.set_cursor_style(CursorStyle::BlinkingUnderline);
    assert_eq!(
        terminal_state.get_cursor().style,
        CursorStyle::BlinkingUnderline
    );

    terminal_state.set_cursor_style(CursorStyle::BlinkingBar);
    assert_eq!(terminal_state.get_cursor().style, CursorStyle::BlinkingBar);
}

#[test]
fn test_character_attribute_system() {
    let mut processor = Emulator::new();

    // Test basic text attributes
    processor.apply_sgr_param(1); // Bold
    assert!(processor.terminal_state.current_attributes.bold);

    processor.apply_sgr_param(3); // Italic
    assert!(processor.terminal_state.current_attributes.italic);

    processor.apply_sgr_param(4); // Underline
    assert!(processor.terminal_state.current_attributes.underline);

    processor.apply_sgr_param(7); // Reverse
    assert!(processor.terminal_state.current_attributes.reverse);

    processor.apply_sgr_param(9); // Strikethrough
    assert!(processor.terminal_state.current_attributes.strikethrough);

    // Test extended attributes
    processor.apply_sgr_param(2); // Dim
    assert!(processor.terminal_state.current_attributes.dim);

    processor.apply_sgr_param(5); // Blink
    assert!(processor.terminal_state.current_attributes.blink);

    processor.apply_sgr_param(8); // Hidden
    assert!(processor.terminal_state.current_attributes.hidden);

    // Test reset attributes
    processor.apply_sgr_param(22); // Reset bold and dim
    assert!(!processor.terminal_state.current_attributes.bold);
    assert!(!processor.terminal_state.current_attributes.dim);

    processor.apply_sgr_param(23); // Reset italic
    assert!(!processor.terminal_state.current_attributes.italic);

    processor.apply_sgr_param(24); // Reset underline
    assert!(!processor.terminal_state.current_attributes.underline);

    // Test complete reset
    processor.apply_sgr_param(0); // Reset all
    let default_cell = TerminalCell::default();
    assert_eq!(
        processor.terminal_state.current_attributes.bold,
        default_cell.bold
    );
    assert_eq!(
        processor.terminal_state.current_attributes.italic,
        default_cell.italic
    );
    assert_eq!(
        processor.terminal_state.current_attributes.underline,
        default_cell.underline
    );
    assert_eq!(
        processor.terminal_state.current_attributes.reverse,
        default_cell.reverse
    );
    assert_eq!(
        processor.terminal_state.current_attributes.strikethrough,
        default_cell.strikethrough
    );
}

#[test]
fn test_standard_color_support() {
    let mut processor = Emulator::new();

    // Test standard 8 colors (foreground)
    processor.apply_sgr_param(30); // Black
    assert_eq!(
        processor.terminal_state.current_attributes.fg_color,
        Some(TermColor::Indexed(0))
    );

    processor.apply_sgr_param(31); // Red
    assert_eq!(
        processor.terminal_state.current_attributes.fg_color,
        Some(TermColor::Indexed(1))
    );

    processor.apply_sgr_param(37); // White
    assert_eq!(
        processor.terminal_state.current_attributes.fg_color,
        Some(TermColor::Indexed(7))
    );

    // Test bright colors (foreground)
    processor.apply_sgr_param(90); // Bright black
    assert_eq!(
        processor.terminal_state.current_attributes.fg_color,
        Some(TermColor::Indexed(8))
    );

    processor.apply_sgr_param(97); // Bright white
    assert_eq!(
        processor.terminal_state.current_attributes.fg_color,
        Some(TermColor::Indexed(15))
    );

    // Test standard 8 colors (background)
    processor.apply_sgr_param(40); // Black background
    assert_eq!(
        processor.terminal_state.current_attributes.bg_color,
        Some(TermColor::Indexed(0))
    );

    processor.apply_sgr_param(47); // White background
    assert_eq!(
        processor.terminal_state.current_attributes.bg_color,
        Some(TermColor::Indexed(7))
    );

    // Test bright colors (background)
    processor.apply_sgr_param(100); // Bright black background
    assert_eq!(
        processor.terminal_state.current_attributes.bg_color,
        Some(TermColor::Indexed(8))
    );

    processor.apply_sgr_param(107); // Bright white background
    assert_eq!(
        processor.terminal_state.current_attributes.bg_color,
        Some(TermColor::Indexed(15))
    );

    // Test color reset
    processor.apply_sgr_param(39); // Default foreground
    assert_eq!(processor.terminal_state.current_attributes.fg_color, None);

    processor.apply_sgr_param(49); // Default background
    assert_eq!(processor.terminal_state.current_attributes.bg_color, None);
}

#[test]
fn test_extended_attributes() {
    let mut processor = Emulator::new();

    // Test double underline
    processor.apply_sgr_param(21);
    assert!(processor.terminal_state.current_attributes.double_underline);

    // Test framed and encircled
    processor.apply_sgr_param(51);
    assert!(processor.terminal_state.current_attributes.framed);

    processor.apply_sgr_param(52);
    assert!(processor.terminal_state.current_attributes.encircled);

    processor.apply_sgr_param(53);
    assert!(processor.terminal_state.current_attributes.overlined);

    // Test frame/encircle reset
    processor.apply_sgr_param(54);
    assert!(!processor.terminal_state.current_attributes.framed);
    assert!(!processor.terminal_state.current_attributes.encircled);

    processor.apply_sgr_param(55);
    assert!(!processor.terminal_state.current_attributes.overlined);

    // Test font selection
    processor.apply_sgr_param(11); // Alternative font 1
    assert_eq!(processor.terminal_state.current_attributes.font_id, Some(1));

    processor.apply_sgr_param(19); // Alternative font 9
    assert_eq!(processor.terminal_state.current_attributes.font_id, Some(9));
}

#[test]
fn test_ideogram_attributes() {
    let mut processor = Emulator::new();

    // Test ideogram attributes
    processor.apply_sgr_param(60); // Ideogram underline
    assert!(
        processor
            .terminal_state
            .current_attributes
            .ideogram_underline
    );

    processor.apply_sgr_param(61); // Ideogram double underline
    assert!(
        processor
            .terminal_state
            .current_attributes
            .ideogram_double_underline
    );

    processor.apply_sgr_param(62); // Ideogram overline
    assert!(
        processor
            .terminal_state
            .current_attributes
            .ideogram_overline
    );

    processor.apply_sgr_param(63); // Ideogram double overline
    assert!(
        processor
            .terminal_state
            .current_attributes
            .ideogram_double_overline
    );

    processor.apply_sgr_param(64); // Ideogram stress
    assert!(processor.terminal_state.current_attributes.ideogram_stress);

    // Test ideogram reset
    processor.apply_sgr_param(65);
    assert!(
        !processor
            .terminal_state
            .current_attributes
            .ideogram_underline
    );
    assert!(
        !processor
            .terminal_state
            .current_attributes
            .ideogram_double_underline
    );
    assert!(
        !processor
            .terminal_state
            .current_attributes
            .ideogram_overline
    );
    assert!(
        !processor
            .terminal_state
            .current_attributes
            .ideogram_double_overline
    );
    assert!(!processor.terminal_state.current_attributes.ideogram_stress);
}

#[test]
fn test_sgr_sequence_processing() {
    let mut processor = Emulator::new();

    // Test basic bold parameter directly
    processor.apply_sgr_param(1);
    assert!(processor.terminal_state.current_attributes.bold);

    // Test that apply_sgr_param works correctly for various parameters
    processor.apply_sgr_param(0); // Reset
    assert!(!processor.terminal_state.current_attributes.bold);

    processor.apply_sgr_param(31); // Red foreground
    assert_eq!(
        processor.terminal_state.current_attributes.fg_color,
        Some(TermColor::Indexed(1))
    );
}

#[test]
fn test_terminal_cell_extended_attributes() {
    let mut cell = TerminalCell::default();

    // Test all default values
    assert!(!cell.bold);
    assert!(!cell.dim);
    assert!(!cell.italic);
    assert!(!cell.underline);
    assert!(!cell.double_underline);
    assert!(!cell.blink);
    assert!(!cell.reverse);
    assert!(!cell.hidden);
    assert!(!cell.strikethrough);
    assert!(!cell.framed);
    assert!(!cell.encircled);
    assert!(!cell.overlined);
    assert!(cell.font_id.is_none());

    // Test setting attributes
    cell.bold = true;
    cell.dim = true;
    cell.double_underline = true;
    cell.font_id = Some(5);

    assert!(cell.bold);
    assert!(cell.dim);
    assert!(cell.double_underline);
    assert_eq!(cell.font_id, Some(5));
}

#[test]
fn test_scrolling_region_support() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Test setting scrolling region
    terminal_state.set_scrolling_region(Some(5), Some(20));
    assert_eq!(terminal_state.scroll_region.top, 4); // 5-1 (1-based to 0-based)
    assert_eq!(terminal_state.scroll_region.bottom, 19); // 20-1

    // Test invalid regions reset to full screen
    terminal_state.set_scrolling_region(Some(25), Some(30)); // Beyond screen bounds
    assert_eq!(terminal_state.scroll_region.top, 0);
    assert_eq!(terminal_state.scroll_region.bottom, 23);

    // Test invalid order resets to full screen
    terminal_state.set_scrolling_region(Some(15), Some(10)); // Bottom < top
    assert_eq!(terminal_state.scroll_region.top, 0);
    assert_eq!(terminal_state.scroll_region.bottom, 23);

    // Test reset scrolling region
    terminal_state.set_scrolling_region(Some(5), Some(15));
    terminal_state.reset_scrolling_region();
    assert_eq!(terminal_state.scroll_region.top, 0);
    assert_eq!(terminal_state.scroll_region.bottom, 23);
}

#[test]
fn test_scrolling_within_region() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Set scrolling region from lines 5-15 (1-based input)
    terminal_state.set_scrolling_region(Some(5), Some(15));
    assert_eq!(terminal_state.scroll_region.top, 4); // 0-based
    assert_eq!(terminal_state.scroll_region.bottom, 14); // 0-based

    // Test basic scrolling operations (without complex content verification)
    // These operations should not panic and should complete successfully
    terminal_state.scroll_up_in_region(1);
    terminal_state.scroll_down_in_region(1);

    // Verify the scrolling region is still properly set after operations
    assert_eq!(terminal_state.scroll_region.top, 4);
    assert_eq!(terminal_state.scroll_region.bottom, 14);
}

#[test]
fn test_cursor_in_scrolling_region() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Set scrolling region
    terminal_state.set_scrolling_region(Some(10), Some(20));

    // Test cursor outside region
    terminal_state.set_cursor_position(0, 5);
    assert!(!terminal_state.is_cursor_in_scrolling_region());

    // Test cursor inside region
    terminal_state.set_cursor_position(0, 15);
    assert!(terminal_state.is_cursor_in_scrolling_region());

    // Test cursor at boundaries
    terminal_state.set_cursor_position(0, 9); // Top boundary (0-based)
    assert!(terminal_state.is_cursor_in_scrolling_region());

    terminal_state.set_cursor_position(0, 19); // Bottom boundary (0-based)
    assert!(terminal_state.is_cursor_in_scrolling_region());
}

#[test]
fn test_line_feed_with_scrolling_region() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Set scrolling region from lines 10-15
    terminal_state.set_scrolling_region(Some(10), Some(15));

    // Position cursor at bottom of scrolling region
    terminal_state.set_cursor_position(5, 14); // Line 15 (1-based)

    // Line feed should trigger scroll and keep cursor at bottom
    let original_y = terminal_state.get_cursor().y;
    terminal_state.line_feed_with_scrolling();
    assert_eq!(terminal_state.get_cursor().y, original_y); // Should stay at bottom
    assert_eq!(terminal_state.get_cursor().x, 5); // X position unchanged

    // Position cursor within scrolling region
    terminal_state.set_cursor_position(10, 12); // Line 13 (1-based)
    terminal_state.line_feed_with_scrolling();
    assert_eq!(terminal_state.get_cursor().y, 13); // Should move down one line
    assert_eq!(terminal_state.get_cursor().x, 10); // X position unchanged
}

#[test]
fn test_reverse_line_feed_with_scrolling_region() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Set scrolling region from lines 10-15
    terminal_state.set_scrolling_region(Some(10), Some(15));

    // Position cursor at top of scrolling region
    terminal_state.set_cursor_position(5, 9); // Line 10 (1-based)

    // Reverse line feed should trigger scroll and keep cursor at top
    let original_y = terminal_state.get_cursor().y;
    terminal_state.reverse_line_feed_with_scrolling();
    assert_eq!(terminal_state.get_cursor().y, original_y); // Should stay at top
    assert_eq!(terminal_state.get_cursor().x, 5); // X position unchanged

    // Position cursor within scrolling region
    terminal_state.set_cursor_position(10, 12); // Line 13 (1-based)
    terminal_state.reverse_line_feed_with_scrolling();
    assert_eq!(terminal_state.get_cursor().y, 11); // Should move up one line
    assert_eq!(terminal_state.get_cursor().x, 10); // X position unchanged
}

#[test]
fn test_decstbm_command_processing() {
    let mut processor = Emulator::new();

    // Test DECSTBM command with parameters
    processor.process_string("\x1b[10;20r"); // Set margins 10-20
    assert_eq!(processor.terminal_state.scroll_region.top, 9); // 10-1 (1-based to 0-based)
    assert_eq!(processor.terminal_state.scroll_region.bottom, 19); // 20-1

    // DECSTBM should move cursor to home position of scrolling region
    assert_eq!(processor.terminal_state.get_cursor().x, 0);
    assert_eq!(processor.terminal_state.get_cursor().y, 9); // Top of scrolling region

    // Test DECSTBM without parameters (reset to full screen)
    processor.process_string("\x1b[r");
    assert_eq!(processor.terminal_state.scroll_region.top, 0);
    assert_eq!(processor.terminal_state.scroll_region.bottom, 23);

    // Test SU (Scroll Up) command
    processor.process_string("\x1b[10;15r"); // Set region first
    processor.process_string("\x1b[3S"); // Scroll up 3 lines
                                         // Scrolling would occur within the region

    // Test SD (Scroll Down) command
    processor.process_string("\x1b[2T"); // Scroll down 2 lines
                                         // Scrolling would occur within the region
}

#[test]
fn test_terminal_mode_management() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Test cursor key mode (DECCKM)
    assert!(!terminal_state.mode.cursor_key_mode); // Default is disabled
    let normal_up = terminal_state.get_cursor_key_sequence("up");
    assert_eq!(normal_up, "\x1b[A");

    terminal_state.mode.cursor_key_mode = true;
    let app_up = terminal_state.get_cursor_key_sequence("up");
    assert_eq!(app_up, "\x1bOA");

    // Test keypad mode (DECPAM)
    assert!(!terminal_state.mode.keypad_mode); // Default is disabled
    let normal_keypad = terminal_state.get_keypad_sequence("5");
    assert_eq!(normal_keypad, "5");

    terminal_state.mode.keypad_mode = true;
    let app_keypad = terminal_state.get_keypad_sequence("5");
    assert_eq!(app_keypad, "\x1bOu");

    // Test origin mode (DECOM)
    terminal_state.set_scrolling_region(Some(10), Some(20));
    terminal_state.mode.origin_mode = false;
    terminal_state.set_cursor_position(5, 15);
    assert_eq!(terminal_state.get_cursor().y, 15); // Absolute positioning

    terminal_state.mode.origin_mode = true;
    terminal_state.set_cursor_position(5, 5);
    assert_eq!(terminal_state.get_cursor().y, 14); // 9 + 5 (relative to scrolling region)
}

#[test]
fn test_insert_and_auto_wrap_modes() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Test insert mode effects
    terminal_state.set_cursor_position(0, 0);
    terminal_state.mode.insert_mode = true;

    // Insert mode should shift characters right
    terminal_state.insert_character_at_cursor();
    // Verify the method completes without panic

    // Test delete character functionality
    terminal_state.delete_character_at_cursor(1);
    // Verify the method completes without panic

    // Test auto-wrap mode
    terminal_state.mode.auto_wrap_mode = true;
    assert!(terminal_state.mode.auto_wrap_mode);

    terminal_state.mode.auto_wrap_mode = false;
    assert!(!terminal_state.mode.auto_wrap_mode);
}

#[test]
fn test_cursor_key_sequences() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Test all cursor keys in normal mode
    terminal_state.mode.cursor_key_mode = false;
    assert_eq!(terminal_state.get_cursor_key_sequence("up"), "\x1b[A");
    assert_eq!(terminal_state.get_cursor_key_sequence("down"), "\x1b[B");
    assert_eq!(terminal_state.get_cursor_key_sequence("right"), "\x1b[C");
    assert_eq!(terminal_state.get_cursor_key_sequence("left"), "\x1b[D");
    assert_eq!(terminal_state.get_cursor_key_sequence("home"), "\x1b[H");
    assert_eq!(terminal_state.get_cursor_key_sequence("end"), "\x1b[F");

    // Test all cursor keys in application mode
    terminal_state.mode.cursor_key_mode = true;
    assert_eq!(terminal_state.get_cursor_key_sequence("up"), "\x1bOA");
    assert_eq!(terminal_state.get_cursor_key_sequence("down"), "\x1bOB");
    assert_eq!(terminal_state.get_cursor_key_sequence("right"), "\x1bOC");
    assert_eq!(terminal_state.get_cursor_key_sequence("left"), "\x1bOD");
    assert_eq!(terminal_state.get_cursor_key_sequence("home"), "\x1bOH");
    assert_eq!(terminal_state.get_cursor_key_sequence("end"), "\x1bOF");
}

#[test]
fn test_keypad_sequences() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Test keypad keys in normal mode
    terminal_state.mode.keypad_mode = false;
    assert_eq!(terminal_state.get_keypad_sequence("0"), "0");
    assert_eq!(terminal_state.get_keypad_sequence("5"), "5");
    assert_eq!(terminal_state.get_keypad_sequence("+"), "+");
    assert_eq!(terminal_state.get_keypad_sequence("enter"), "enter");

    // Test keypad keys in application mode
    terminal_state.mode.keypad_mode = true;
    assert_eq!(terminal_state.get_keypad_sequence("0"), "\x1bOp");
    assert_eq!(terminal_state.get_keypad_sequence("1"), "\x1bOq");
    assert_eq!(terminal_state.get_keypad_sequence("2"), "\x1bOr");
    assert_eq!(terminal_state.get_keypad_sequence("5"), "\x1bOu");
    assert_eq!(terminal_state.get_keypad_sequence("9"), "\x1bOy");
    assert_eq!(terminal_state.get_keypad_sequence("."), "\x1bOn");
    assert_eq!(terminal_state.get_keypad_sequence("+"), "\x1bOk");
    assert_eq!(terminal_state.get_keypad_sequence("-"), "\x1bOm");
    assert_eq!(terminal_state.get_keypad_sequence("*"), "\x1bOj");
    assert_eq!(terminal_state.get_keypad_sequence("/"), "\x1bOo");
    assert_eq!(terminal_state.get_keypad_sequence("="), "\x1bOX");
    assert_eq!(terminal_state.get_keypad_sequence("enter"), "\x1bOM");
}

#[test]
fn test_newline_mode() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Position cursor away from origin
    terminal_state.set_cursor_position(10, 5);
    let initial_x = terminal_state.get_cursor().x;
    let initial_y = terminal_state.get_cursor().y;

    // Test LNM disabled (default) - newline is just LF
    terminal_state.mode.local_echo_mode = false;
    terminal_state.process_newline();
    assert_eq!(terminal_state.get_cursor().x, initial_x); // X unchanged
    assert_eq!(terminal_state.get_cursor().y, initial_y + 1); // Y incremented

    // Reset position
    terminal_state.set_cursor_position(10, 5);

    // Test LNM enabled - newline is CR+LF
    terminal_state.mode.local_echo_mode = true;
    terminal_state.process_newline();
    assert_eq!(terminal_state.get_cursor().x, 0); // X reset to 0 (CR)
    assert_eq!(terminal_state.get_cursor().y, 6); // Y incremented (LF)
}

#[test]
fn test_origin_mode_with_scrolling_region() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Set scrolling region from lines 10-20 (1-based input)
    terminal_state.set_scrolling_region(Some(10), Some(20));

    // Test absolute positioning (origin mode disabled)
    terminal_state.mode.origin_mode = false;
    terminal_state.set_cursor_position(5, 15);
    assert_eq!(terminal_state.get_cursor().x, 5);
    assert_eq!(terminal_state.get_cursor().y, 15); // Absolute position

    // Test relative positioning (origin mode enabled)
    terminal_state.mode.origin_mode = true;
    terminal_state.set_cursor_position(7, 3);
    assert_eq!(terminal_state.get_cursor().x, 7);
    assert_eq!(terminal_state.get_cursor().y, 12); // 9 (region_top) + 3 = 12

    // Test bounds checking in origin mode
    terminal_state.set_cursor_position(10, 50); // Y beyond region
    assert_eq!(terminal_state.get_cursor().y, 19); // Clamped to region bottom
}

#[test]
fn test_reverse_video_mode() {
    let mut terminal_state = TerminalState::new(80, 24);

    // Test reverse video mode state
    assert!(!terminal_state.mode.reverse_video); // Default is disabled
    terminal_state.mode.reverse_video = true;
    assert!(terminal_state.mode.reverse_video);

    // Reverse video effects are tested implicitly through put_char_at_cursor
    // The color swapping logic is applied when characters are written
}
