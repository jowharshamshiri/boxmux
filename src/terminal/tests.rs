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

/// An emulator of `width` by `height` that has processed `input`.
fn after(width: usize, height: usize, input: &str) -> Emulator {
    let mut emulator = Emulator::with_screen_size(width, height);
    emulator.process_string(input);
    emulator
}

fn text(width: usize, height: usize, input: &str) -> String {
    after(width, height, input).get_processed_text()
}

fn replies(input: &str) -> Vec<String> {
    after(20, 5, input).take_responses()
}

#[test]
fn plain_text_and_colors_show_as_text() {
    assert_eq!(text(80, 24, "Hello World"), "Hello World");
    assert_eq!(text(80, 24, "\x1b[31mRed Text\x1b[0m"), "Red Text");
}

#[test]
fn line_feed_moves_down_and_carriage_return_moves_to_the_first_column() {
    // A PTY turns each newline a program prints into CR LF.
    let emulator = after(80, 24, "Line 1\r\nLine 2\r\n");
    assert_eq!(emulator.get_processed_text(), "Line 1\nLine 2");
    assert_eq!((emulator.get_cursor_x(), emulator.get_cursor_y()), (0, 2));
    // A bare line feed keeps the column...
    assert_eq!(text(10, 3, "ab\ncd"), "ab\n  cd");
    // ...except in newline mode (LNM).
    assert_eq!(text(10, 3, "\x1b[20hab\ncd"), "ab\ncd");
}

#[test]
fn erasing_the_display_keeps_the_cursor_where_it_is() {
    assert_eq!(text(20, 3, "Some text\x1b[2JCleared"), "         Cleared");
    assert_eq!(text(20, 3, "Some text\x1b[2J\x1b[HCleared"), "Cleared");
    assert_eq!(
        text(4, 3, "abcd\r\nefgh\r\nijkl\x1b[2;2H\x1b[0J"),
        "abcd\ne"
    );
}

#[test]
fn erasing_in_a_line() {
    assert_eq!(text(6, 1, "abcdef\x1b[3G\x1b[K"), "ab");
    assert_eq!(text(6, 1, "abcdef\x1b[3G\x1b[1K"), "   def");
    assert_eq!(text(6, 1, "abcdef\x1b[3G\x1b[2K"), "");
}

#[test]
fn erased_cells_take_the_current_background() {
    let emulator = after(4, 1, "\x1b[44m\x1b[2K");
    let cell = &emulator.terminal_state.get_active_screen_ref()[0][3];
    assert_eq!(cell.bg_color, Some(TermColor::Indexed(4)));
}

#[test]
fn lines_that_scroll_off_the_top_are_kept_in_the_scrollback() {
    let emulator = after(10, 2, "one\r\ntwo\r\nthree\r\nfour");
    assert_eq!(emulator.get_processed_text(), "three\nfour");
    let lines: Vec<String> = emulator
        .display_lines()
        .iter()
        .map(|l| crate::utils::strip_ansi_codes(l).trim_end().to_string())
        .collect();
    assert_eq!(lines, vec!["one", "two", "three", "four"]);
    // The alternate screen has no scrollback.
    let emulator = after(10, 2, "\x1b[?1049hone\r\ntwo\r\nthree");
    assert_eq!(emulator.display_lines().len(), 2);
}

#[test]
fn the_cursor_is_shown_as_a_reverse_cell_unless_hidden() {
    let emulator = after(5, 1, "ab");
    assert!(
        emulator.display_lines()[0].contains("\x1b[0;7m "),
        "{:?}",
        emulator.display_lines()
    );
    let emulator = after(5, 1, "ab\x1b[?25l");
    assert!(!emulator.display_lines()[0].contains(";7m"));
}

#[test]
fn cursor_movement_sequences() {
    // CUP, CUU, CUD, CUF, CUB
    assert_eq!(
        text(10, 4, "\x1b[3;4HX\x1b[2AY\x1b[1BZ\x1b[2CW\x1b[4DV"),
        "    Y\n     V  W\n   X"
    );
    // CHA and HPA set the column, VPA the line, HPR and VPR move relative.
    assert_eq!(text(10, 3, "abc\x1b[6GX\x1b[2`Y\x1b[3dZ"), "aYc  X\n\n  Z");
    assert_eq!(text(10, 3, "\x1b[2aX\x1b[1eY"), "  X\n   Y");
    // CNL and CPL move whole lines and return to the first column.
    assert_eq!(text(10, 3, "abc\x1b[2EX\x1b[1FY"), "abc\nY\nX");
    // Parameters of 0 count as 1.
    assert_eq!(text(10, 3, "\x1b[0;0HA\x1b[0CB"), "A B");
}

#[test]
fn cursor_movement_stays_inside_the_scrolling_region() {
    let emulator = after(10, 6, "\x1b[2;4r\x1b[3;1H\x1b[10A");
    assert_eq!(emulator.get_cursor_y(), 1);
    let emulator = after(10, 6, "\x1b[2;4r\x1b[3;1H\x1b[10B");
    assert_eq!(emulator.get_cursor_y(), 3);
    // Below the region, moving up still stops at its top (as in xterm) and moving down
    // reaches the last line.
    let emulator = after(10, 6, "\x1b[2;4r\x1b[6;1H\x1b[10A");
    assert_eq!(emulator.get_cursor_y(), 1);
    let emulator = after(10, 6, "\x1b[2;4r\x1b[5;1H\x1b[10B");
    assert_eq!(emulator.get_cursor_y(), 5);
    // Above the region, moving up reaches the first line.
    let emulator = after(10, 6, "\x1b[3;4r\x1b[2;1H\x1b[10A");
    assert_eq!(emulator.get_cursor_y(), 0);
}

#[test]
fn saving_the_cursor_also_saves_attributes_and_restores_them() {
    for (save, restore) in [("\x1b7", "\x1b8"), ("\x1b[s", "\x1b[u")] {
        let emulator = after(
            10,
            3,
            &format!("\x1b[2;3H\x1b[31m{}\x1b[0m\x1b[H{}x", save, restore),
        );
        assert_eq!(emulator.get_processed_text(), "\n  x");
        let cell = &emulator.terminal_state.get_active_screen_ref()[1][2];
        assert_eq!(cell.fg_color, Some(TermColor::Indexed(1)));
    }
}

#[test]
fn inserting_deleting_and_erasing_characters() {
    // ICH pushes the rest of the line right, losing what passes the edge.
    assert_eq!(text(6, 1, "abcdef\x1b[2G\x1b[2@"), "a  bcd");
    // DCH pulls the rest of the line left.
    assert_eq!(text(6, 1, "abcdef\x1b[2G\x1b[2P"), "adef");
    // ECH blanks without moving anything.
    assert_eq!(text(6, 1, "abcdef\x1b[2G\x1b[2X"), "a  def");
    // Insert mode pushes text right as it is printed.
    assert_eq!(text(6, 1, "abc\x1b[1G\x1b[4hXY"), "XYabc");
    // A wide character cut by a shift is blanked, not left half.
    assert_eq!(text(4, 1, "ab日\x1b[1G\x1b[1@"), " ab");
}

#[test]
fn repeating_the_last_character() {
    assert_eq!(text(10, 1, "ab\x1b[3b"), "abbbb");
    // Nothing was printed yet: nothing to repeat.
    assert_eq!(text(10, 1, "\x1b[3b"), "");
}

#[test]
fn inserting_and_deleting_lines_within_the_scrolling_region() {
    let screen = "1\r\n2\r\n3\r\n4\r\n5";
    // IL at line 2 of a region 2..4: line 4 falls out of the region; line 5 stays.
    assert_eq!(
        text(5, 5, &format!("{}\x1b[2;4r\x1b[2;3H\x1b[L", screen)),
        "1\n\n2\n3\n5"
    );
    // DL pulls the region up and leaves a blank at its bottom.
    assert_eq!(
        text(5, 5, &format!("{}\x1b[2;4r\x1b[2;1H\x1b[M", screen)),
        "1\n3\n4\n\n5"
    );
    // Outside the region nothing happens.
    assert_eq!(
        text(5, 5, &format!("{}\x1b[2;4r\x1b[5;1H\x1b[L", screen)),
        "1\n2\n3\n4\n5"
    );
}

#[test]
fn scrolling_the_region_up_and_down() {
    let screen = "1\r\n2\r\n3\r\n4";
    assert_eq!(
        text(5, 4, &format!("{}\x1b[2;3r\x1b[S", screen)),
        "1\n3\n\n4"
    );
    assert_eq!(
        text(5, 4, &format!("{}\x1b[2;3r\x1b[T", screen)),
        "1\n\n2\n4"
    );
    // A line feed on the region's bottom line scrolls only the region.
    assert_eq!(
        text(5, 4, &format!("{}\x1b[2;3r\x1b[3;1H\nX", screen)),
        "1\n3\nX\n4"
    );
    // Index, next line and reverse index.
    assert_eq!(text(5, 3, "a\x1bDb\x1bEc"), "a\n b\nc");
    assert_eq!(text(5, 3, "a\x1bMb"), " b\na");
}

#[test]
fn origin_mode_counts_lines_from_the_scrolling_region() {
    let emulator = after(10, 6, "\x1b[3;5r\x1b[?6h\x1b[2;1HX\x1b[6n");
    assert_eq!(emulator.get_processed_text(), "\n\n\nX");
    assert_eq!(emulator.terminal_state.pending_responses, vec!["\x1b[2;2R"]);
}

#[test]
fn several_modes_in_one_sequence_all_apply() {
    let emulator = after(10, 2, "\x1b[?1;2004;1006h");
    let mode = &emulator.terminal_state.mode;
    assert!(mode.cursor_key_mode && mode.bracketed_paste_mode && mode.mouse_sgr_mode);
    let emulator = after(10, 2, "\x1b[?1;2004;1006h\x1b[?1;2004l");
    let mode = &emulator.terminal_state.mode;
    assert!(!mode.cursor_key_mode && !mode.bracketed_paste_mode && mode.mouse_sgr_mode);
}

#[test]
fn line_drawing_characters_are_translated() {
    assert_eq!(text(10, 2, "\x1b(0lqqk\x1b(B lqk"), "┌──┐ lqk");
    // Through G1 with shift out and shift in.
    assert_eq!(text(10, 2, "\x1b)0x\x0ex\x0fx"), "x│x");
    // The UK set has a pound sign for #.
    assert_eq!(text(10, 2, "\x1b(A#"), "£");
}

#[test]
fn alternate_screen_keeps_the_primary_screen_and_cursor() {
    let mut emulator = after(10, 3, "main\x1b[?1049h");
    assert_eq!(emulator.get_processed_text(), "");
    emulator.process_string("\x1b[2;2Hfull");
    assert!(emulator.is_using_alternate_screen());
    emulator.process_string("\x1b[?1049l");
    assert_eq!(emulator.get_processed_text(), "main");
    assert_eq!((emulator.get_cursor_x(), emulator.get_cursor_y()), (4, 0));
    // Entering again starts from a clear screen.
    emulator.process_string("\x1b[?1049h");
    assert_eq!(emulator.get_processed_text(), "");
}

#[test]
fn tab_stops() {
    // Every 8 columns by default.
    assert_eq!(text(30, 1, "a\tb\tc"), "a       b       c");
    // HTS sets one, TBC 0 clears the one at the cursor, TBC 3 clears all.
    assert_eq!(text(30, 1, "\x1b[4G\x1bH\x1b[1Ga\tb"), "a  b");
    assert_eq!(text(30, 1, "\x1b[9G\x1b[g\x1b[1Ga\tb"), "a               b");
    assert_eq!(text(12, 1, "\x1b[3ga\tb"), "a          b");
    // CHT and CBT.
    assert_eq!(text(30, 1, "\x1b[2Ia\x1b[ZXb"), "                Xb");
}

#[test]
fn queries_are_answered() {
    assert_eq!(replies("\x1b[5n"), vec!["\x1b[0n"]);
    assert_eq!(replies("ab\x1b[6n"), vec!["\x1b[1;3R"]);
    assert_eq!(replies("\x1b[c"), vec![PRIMARY]);
    assert_eq!(replies("\x1b[0c"), vec![PRIMARY]);
    assert_eq!(replies("\x1bZ"), vec![PRIMARY]);
    assert_eq!(replies("\x1b[>c"), vec!["\x1b[>0;0;0c"]);
    assert_eq!(replies("\x1b[18t"), vec!["\x1b[8;5;20t"]);
    // DECRQM: set, reset, unknown.
    assert_eq!(replies("\x1b[?2004h\x1b[?2004$p"), vec!["\x1b[?2004;1$y"]);
    assert_eq!(replies("\x1b[?1$p"), vec!["\x1b[?1;2$y"]);
    assert_eq!(replies("\x1b[?4242$p"), vec!["\x1b[?4242;0$y"]);
    assert_eq!(replies("\x1b[4$p"), vec!["\x1b[4;2$y"]);
    // Settings and termcap entries are reported as not valid, so programs do not wait.
    assert_eq!(replies("\x1bP$qm\x1b\\"), vec!["\x1bP0$r\x1b\\"]);
    assert_eq!(replies("\x1bP+q436f\x1b\\"), vec!["\x1bP0+r436f\x1b\\"]);
}

const PRIMARY: &str = "\x1b[?62;22c";

#[test]
fn colon_separated_colors_and_underline_styles() {
    let emulator = after(10, 1, "\x1b[38:2::1:2:3;48:5:99;4:3mx");
    let cell = &emulator.terminal_state.get_active_screen_ref()[0][0];
    assert_eq!(cell.fg_color, Some(TermColor::Rgb(1, 2, 3)));
    assert_eq!(cell.bg_color, Some(TermColor::Indexed(99)));
    assert!(cell.underline);
    let emulator = after(10, 1, "\x1b[4m\x1b[4:0mx");
    assert!(!emulator.terminal_state.get_active_screen_ref()[0][0].underline);
}

#[test]
fn attributes_turn_on_and_off() {
    let emulator = after(
        10,
        1,
        "\x1b[1;2;3;4;5;7;8;9ma\x1b[22;23;24;25;27;28;29mb\x1b[91;102mc\x1b[39;49md",
    );
    let row = &emulator.terminal_state.get_active_screen_ref()[0];
    let a = &row[0];
    assert!(a.bold && a.dim && a.italic && a.underline && a.blink);
    assert!(a.reverse && a.hidden && a.strikethrough);
    let b = TerminalCell {
        character: 'b',
        ..TerminalCell::default()
    };
    assert_eq!(row[1], b);
    assert_eq!(row[2].fg_color, Some(TermColor::Indexed(9)));
    assert_eq!(row[2].bg_color, Some(TermColor::Indexed(10)));
    assert_eq!((row[3].fg_color, row[3].bg_color), (None, None));
}

fn report(
    modes: &str,
    event: MouseEventType,
    button: Option<MouseButton>,
    x: usize,
    y: usize,
) -> Option<String> {
    after(300, 300, modes).terminal_state.generate_mouse_report(
        event,
        x,
        y,
        button,
        MouseModifiers::default(),
    )
}

#[test]
fn mouse_reports_follow_the_tracking_mode() {
    use MouseButton::*;
    use MouseEventType::*;
    // No tracking: nothing is reported.
    assert_eq!(report("", Press, Some(Left), 0, 0), None);
    // X10: presses only.
    assert_eq!(
        report("\x1b[?9h", Press, Some(Left), 0, 0).as_deref(),
        Some("\x1b[M !!")
    );
    assert_eq!(report("\x1b[?9h", Release, Some(Left), 0, 0), None);
    // 1000: presses and releases (button 3 in the legacy encoding), no motion.
    assert_eq!(
        report("\x1b[?1000h", Release, Some(Left), 1, 2).as_deref(),
        Some("\x1b[M#\"#")
    );
    assert_eq!(report("\x1b[?1000h", Motion, Some(Left), 0, 0), None);
    // 1002: motion while a button is held.
    assert_eq!(
        report("\x1b[?1002h", Motion, Some(Left), 0, 0).as_deref(),
        Some("\x1b[M@!!")
    );
    assert_eq!(report("\x1b[?1002h", Motion, None, 0, 0), None);
    // 1003: all motion.
    assert_eq!(
        report("\x1b[?1003h", Motion, None, 0, 0).as_deref(),
        Some("\x1b[MC!!")
    );
    // Turning the mode off stops the reports.
    assert_eq!(
        report("\x1b[?1000h\x1b[?1000l", Press, Some(Left), 0, 0),
        None
    );
    // Wheel events are buttons 64 and up.
    assert_eq!(
        report("\x1b[?1000h\x1b[?1006h", Wheel, Some(WheelDown), 4, 5).as_deref(),
        Some("\x1b[<65;5;6M")
    );
}

#[test]
fn sgr_mouse_reports_name_the_released_button_and_have_no_position_limit() {
    use MouseButton::*;
    use MouseEventType::*;
    let sgr = "\x1b[?1002h\x1b[?1006h";
    assert_eq!(
        report(sgr, Press, Some(Right), 249, 0).as_deref(),
        Some("\x1b[<2;250;1M")
    );
    assert_eq!(
        report(sgr, Release, Some(Right), 249, 0).as_deref(),
        Some("\x1b[<2;250;1m")
    );
    assert_eq!(
        report(sgr, Motion, Some(Left), 0, 0).as_deref(),
        Some("\x1b[<32;1;1M")
    );
    // The legacy encoding cannot express column 250.
    assert_eq!(report("\x1b[?1000h", Press, Some(Left), 249, 0), None);
}

#[test]
fn mouse_modifiers_are_reported_except_in_x10_mode() {
    let modifiers = MouseModifiers {
        shift: true,
        ctrl: true,
        alt: true,
        meta: false,
    };
    let state = after(10, 2, "\x1b[?1000h\x1b[?1006h").terminal_state;
    let report = state.generate_mouse_report(
        MouseEventType::Press,
        0,
        0,
        Some(MouseButton::Left),
        modifiers,
    );
    assert_eq!(report.as_deref(), Some("\x1b[<28;1;1M"));
    let state = after(10, 2, "\x1b[?9h").terminal_state;
    let report = state.generate_mouse_report(
        MouseEventType::Press,
        0,
        0,
        Some(MouseButton::Left),
        modifiers,
    );
    assert_eq!(report.as_deref(), Some("\x1b[M !!"));
}

#[test]
fn resizing_keeps_the_bottom_lines_and_the_cursor_on_them() {
    let mut emulator = after(10, 4, "1\r\n2\r\n3\r\n4");
    emulator.resize_screen(10, 2);
    assert_eq!(emulator.get_processed_text(), "3\n4");
    assert_eq!(emulator.get_cursor_y(), 1);
    let lines = emulator.display_lines();
    assert_eq!(crate::utils::strip_ansi_codes(&lines[0]).trim_end(), "1");
}

#[test]
fn screen_alignment_fills_the_screen() {
    assert_eq!(text(3, 2, "\x1b#8"), "EEE\nEEE");
}
