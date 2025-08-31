use crate::ansi_processor::AnsiProcessor;

#[cfg(test)]
mod ansi_processor_integration_tests {
    use super::*;

    #[test]
    fn test_ansi_processor_with_ls_output() {
        let mut processor = AnsiProcessor::new();

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
    fn test_ansi_processor_with_grep_output() {
        let mut processor = AnsiProcessor::new();

        // Simulate grep output with highlighting
        let grep_output = "file.txt:\x1b[01;31m\x1b[Kpattern\x1b[m\x1b[K found here\n";
        processor.process_string(grep_output);

        let result = processor.get_processed_text();
        assert!(result.contains("file.txt:pattern found here"));
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn test_ansi_processor_with_vim_output() {
        let mut processor = AnsiProcessor::new();

        // Simulate vim-like editor output with cursor positioning
        let vim_output = "\x1b[2J\x1b[H\x1b[7mWelcome to editor\x1b[0m\n\x1b[2;1HLine 2 content";
        processor.process_string(vim_output);

        let result = processor.get_processed_text();
        assert!(result.contains("Welcome to editor"));
        assert!(result.contains("Line 2 content"));
    }

    #[test]
    fn test_ansi_processor_with_htop_like_output() {
        let mut processor = AnsiProcessor::new();

        // Simulate system monitor output with colors and positioning
        let htop_output = "\x1b[1;1H\x1b[44m\x1b[37mCPU: \x1b[32m15%\x1b[0m\x1b[49m\n\x1b[2;1H\x1b[44m\x1b[37mMEM: \x1b[31m85%\x1b[0m\x1b[49m";
        processor.process_string(htop_output);

        let result = processor.get_processed_text();
        assert!(result.contains("CPU: 15%"));
        assert!(result.contains("MEM: 85%"));
    }

    #[test]
    fn test_ansi_processor_cursor_tracking() {
        let mut processor = AnsiProcessor::new();

        processor.process_string("Start\x1b[5;10HMiddle\x1b[1;1HBegin");

        // After positioning cursor and writing, should track position
        assert!(processor.get_cursor_y() <= 10); // Rough bounds check
        assert!(processor.get_cursor_x() >= 0);
    }

    #[test]
    fn test_ansi_processor_color_state_tracking() {
        let mut processor = AnsiProcessor::new();

        processor.process_string("\x1b[31mRed\x1b[32mGreen\x1b[0mReset");

        // After reset, colors should be cleared
        assert_eq!(processor.terminal_state.current_attributes.fg_color, None);
        assert_eq!(processor.terminal_state.current_attributes.bg_color, None);
        assert!(!processor.terminal_state.current_attributes.bold);
    }

    #[test]
    fn test_ansi_processor_incremental_processing() {
        let mut processor = AnsiProcessor::new();

        // Process in chunks like PTY would - this tests the state machine
        processor.process_string("\x1b[31mRed ");
        processor.process_string("Text\x1b[0m");

        let result = processor.get_processed_text();
        assert_eq!(result, "Red Text");
    }

    #[test]
    fn test_ansi_processor_mixed_content() {
        let mut processor = AnsiProcessor::new();

        // Mix of control chars, ANSI codes, and regular text
        let mixed = "Normal\n\x1b[1mBold\x1b[0m\tTabbed\r\nNew line\x1b[2JCleared";
        processor.process_string(mixed);

        let result = processor.get_processed_text();
        assert!(result.contains("Cleared")); // Should be last after clear screen
        assert!(!result.contains("Normal")); // Should be cleared by \x1b[2J
    }

    #[test]
    fn test_ansi_processor_performance_large_output() {
        let mut processor = AnsiProcessor::new();

        // Large output simulation
        let mut large_output = String::new();
        for i in 0..1000 {
            large_output.push_str(&format!("\x1b[{}m Line {}\x1b[0m\n", 31 + (i % 7), i));
        }

        let start = std::time::Instant::now();
        processor.process_string(&large_output);
        let duration = start.elapsed();

        // Should process reasonably quickly (less than 100ms for 1000 lines)
        assert!(duration.as_millis() < 100);
        assert!(processor.get_processed_text().lines().count() >= 1000);
    }

    #[test]
    fn test_ansi_processor_malformed_sequences() {
        let mut processor = AnsiProcessor::new();

        // Test malformed or incomplete ANSI sequences - focus on not panicking
        processor.process_string("Text\x1b[999;999mMore");

        // Should handle gracefully without panicking
        let result = processor.get_processed_text();
        assert!(result.contains("Text"));
        assert!(result.contains("More"));
    }
}
