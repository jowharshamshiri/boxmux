use crate::terminal::Emulator;

#[cfg(test)]
mod emulator_integration_tests {
    use super::*;

    #[test]
    fn test_emulator_with_ls_output() {
        let mut processor = Emulator::new();

        // Simulate colorized ls output
        let ls_output = "\x1b[0m\x1b[01;34mfolder\x1b[0m\n\x1b[01;32mscript.sh\x1b[0m\n\x1b[00mfile.txt\x1b[0m\n";
        processor.process_string(ls_output);

        let result = processor.get_processed_text();
        assert!(result.contains("folder"));
        assert!(result.contains("script.sh"));
        assert!(result.contains("file.txt"));

        // Should have processed color codes without including them in text
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn test_emulator_with_grep_output() {
        let mut processor = Emulator::new();

        // Simulate grep output with highlighting
        let grep_output = "file.txt:\x1b[01;31m\x1b[Kpattern\x1b[m\x1b[K found here\n";
        processor.process_string(grep_output);

        let result = processor.get_processed_text();
        assert!(result.contains("file.txt:pattern found here"));
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn test_emulator_with_vim_output() {
        let mut processor = Emulator::new();

        // Simulate vim-like editor output with cursor positioning
        let vim_output = "\x1b[2J\x1b[H\x1b[7mWelcome to editor\x1b[0m\n\x1b[2;1HLine 2 content";
        processor.process_string(vim_output);

        let result = processor.get_processed_text();
        assert!(result.contains("Welcome to editor"));
        assert!(result.contains("Line 2 content"));
    }

    #[test]
    fn test_emulator_with_htop_like_output() {
        let mut processor = Emulator::new();

        // Simulate system monitor output with colors and positioning
        let htop_output = "\x1b[1;1H\x1b[44m\x1b[37mCPU: \x1b[32m15%\x1b[0m\x1b[49m\n\x1b[2;1H\x1b[44m\x1b[37mMEM: \x1b[31m85%\x1b[0m\x1b[49m";
        processor.process_string(htop_output);

        let result = processor.get_processed_text();
        assert!(result.contains("CPU: 15%"));
        assert!(result.contains("MEM: 85%"));
    }

    #[test]
    fn test_emulator_cursor_tracking() {
        let mut processor = Emulator::new();

        processor.process_string("Start\x1b[5;10HMiddle\x1b[1;1HBegin");

        // The cursor ends after "Begin" on the first row.
        assert_eq!((processor.get_cursor_x(), processor.get_cursor_y()), (5, 0));
    }

    #[test]
    fn test_emulator_color_state_tracking() {
        let mut processor = Emulator::new();

        processor.process_string("\x1b[31mRed\x1b[32mGreen\x1b[0mReset");

        // After reset, colors should be cleared
        assert_eq!(processor.terminal_state.current_attributes.fg_color, None);
        assert_eq!(processor.terminal_state.current_attributes.bg_color, None);
        assert!(!processor.terminal_state.current_attributes.bold);
    }

    #[test]
    fn test_emulator_incremental_processing() {
        let mut processor = Emulator::new();

        // Process in chunks like PTY would - this tests the state machine
        processor.process_string("\x1b[31mRed ");
        processor.process_string("Text\x1b[0m");

        let result = processor.get_processed_text();
        assert_eq!(result, "Red Text");
    }

    #[test]
    fn test_emulator_mixed_content() {
        let mut processor = Emulator::new();

        // Mix of control chars, ANSI codes, and regular text
        let mixed = "Normal\n\x1b[1mBold\x1b[0m\tTabbed\r\nNew line\x1b[2JCleared";
        processor.process_string(mixed);

        let result = processor.get_processed_text();
        assert!(result.contains("Cleared")); // Should be last after clear screen
        assert!(!result.contains("Normal")); // Should be cleared by \x1b[2J
    }

    #[test]
    fn test_emulator_performance_large_output() {
        fn coloured_lines(count: usize) -> String {
            (0..count)
                .map(|i| format!("\x1b[{}m Line {}\x1b[0m\r\n", 31 + (i % 7), i))
                .collect()
        }
        // The fastest of a few runs, so a stall of the machine does not count.
        fn processing_time(output: &str) -> std::time::Duration {
            (0..3)
                .map(|_| {
                    let mut processor = Emulator::new();
                    let start = std::time::Instant::now();
                    processor.process_string(output);
                    start.elapsed()
                })
                .min()
                .unwrap()
        }

        // Each line costs the same however much scrolled off before it: ten times the output
        // takes about ten times as long, where work that grows with the history would take a
        // hundred. Comparing the two keeps the check independent of how fast the machine is.
        let short = processing_time(&coloured_lines(2_000));
        let long = processing_time(&coloured_lines(20_000));
        assert!(
            long < short * 30,
            "2000 lines took {:?} but 20000 took {:?}",
            short,
            long
        );

        // The screen keeps the last lines; the rest scrolled off the top.
        let mut processor = Emulator::new();
        processor.process_string(&coloured_lines(1_000));
        let text = processor.get_processed_text();
        assert!(text.contains("Line 999"), "{}", text);
        assert!(!text.contains("Line 0\n"), "{}", text);
    }

    #[test]
    fn test_emulator_malformed_sequences() {
        let mut processor = Emulator::new();

        // Test malformed or incomplete ANSI sequences - focus on not panicking
        processor.process_string("Text\x1b[999;999mMore");

        // Should handle gracefully without panicking
        let result = processor.get_processed_text();
        assert!(result.contains("Text"));
        assert!(result.contains("More"));
    }
}
