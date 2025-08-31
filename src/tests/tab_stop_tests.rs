#[cfg(test)]
mod tab_stop_tests {
    use crate::ansi_processor::{AnsiProcessor, TerminalState};

    /// F0313: Test default tab stops initialization (every 8 columns)
    #[test]
    fn test_default_tab_stops() {
        let terminal_state = TerminalState::new(80, 24);

        // Check default tab stops at 8, 16, 24, 32, 40, 48, 56, 64, 72
        assert!(!terminal_state.tab_stops[0]); // Column 0 should not have tab stop
        assert!(terminal_state.tab_stops[8]); // Column 8 should have tab stop
        assert!(terminal_state.tab_stops[16]); // Column 16 should have tab stop
        assert!(terminal_state.tab_stops[24]); // Column 24 should have tab stop
        assert!(terminal_state.tab_stops[32]); // Column 32 should have tab stop
        assert!(terminal_state.tab_stops[40]); // Column 40 should have tab stop
        assert!(terminal_state.tab_stops[48]); // Column 48 should have tab stop
        assert!(terminal_state.tab_stops[56]); // Column 56 should have tab stop
        assert!(terminal_state.tab_stops[64]); // Column 64 should have tab stop
        assert!(terminal_state.tab_stops[72]); // Column 72 should have tab stop

        // Check non-tab columns
        assert!(!terminal_state.tab_stops[5]); // Column 5 should not have tab stop
        assert!(!terminal_state.tab_stops[10]); // Column 10 should not have tab stop
    }

    /// F0313: Test tab forward navigation
    #[test]
    fn test_tab_forward() {
        let mut terminal_state = TerminalState::new(80, 24);

        // Start at column 0
        terminal_state.get_cursor_mut().x = 0;

        // Tab forward should go to column 8
        terminal_state.tab_forward();
        assert_eq!(terminal_state.get_cursor().x, 8);

        // Tab forward again should go to column 16
        terminal_state.tab_forward();
        assert_eq!(terminal_state.get_cursor().x, 16);

        // Test from middle of tab section
        terminal_state.get_cursor_mut().x = 5;
        terminal_state.tab_forward();
        assert_eq!(terminal_state.get_cursor().x, 8);
    }

    /// F0313: Test tab backward navigation
    #[test]
    fn test_tab_backward() {
        let mut terminal_state = TerminalState::new(80, 24);

        // Start at column 20
        terminal_state.get_cursor_mut().x = 20;

        // Tab backward should go to column 16
        terminal_state.tab_backward();
        assert_eq!(terminal_state.get_cursor().x, 16);

        // Tab backward again should go to column 8
        terminal_state.tab_backward();
        assert_eq!(terminal_state.get_cursor().x, 8);

        // Tab backward again should go to column 0 (beginning of line)
        terminal_state.tab_backward();
        assert_eq!(terminal_state.get_cursor().x, 0);
    }

    /// F0313: Test tab forward when no more tab stops (should go to end of line)
    #[test]
    fn test_tab_forward_at_end() {
        let mut terminal_state = TerminalState::new(80, 24);

        // Start at column 75 (past last default tab stop at 72)
        terminal_state.get_cursor_mut().x = 75;

        // Tab forward should go to end of line (column 79)
        terminal_state.tab_forward();
        assert_eq!(terminal_state.get_cursor().x, 79);
    }

    /// F0313: Test tab backward when no previous tab stops (should go to beginning)
    #[test]
    fn test_tab_backward_at_beginning() {
        let mut terminal_state = TerminalState::new(80, 24);

        // Start at column 3 (before first tab stop)
        terminal_state.get_cursor_mut().x = 3;

        // Tab backward should go to beginning of line (column 0)
        terminal_state.tab_backward();
        assert_eq!(terminal_state.get_cursor().x, 0);
    }

    /// F0313: Test HTS (ESC H) - Set tab stop at current cursor position
    #[test]
    fn test_set_tab_stop_esc_h() {
        let mut processor = AnsiProcessor::new();

        // Move cursor to column 10
        processor.terminal_state.get_cursor_mut().x = 10;

        // Initially column 10 should not have tab stop
        assert!(!processor.terminal_state.tab_stops[10]);

        // Set tab stop using ESC H
        processor.process_bytes(b"\x1bH");

        // Now column 10 should have tab stop
        assert!(processor.terminal_state.tab_stops[10]);
    }

    /// F0313: Test TBC g (CSI 0 g) - Clear tab stop at current cursor position  
    #[test]
    fn test_clear_tab_stop_current() {
        let mut processor = AnsiProcessor::new();

        // Column 8 should have default tab stop
        assert!(processor.terminal_state.tab_stops[8]);

        // Move cursor to column 8
        processor.terminal_state.get_cursor_mut().x = 8;

        // Clear tab stop at current position (CSI 0 g)
        processor.process_bytes(b"\x1b[0g");

        // Column 8 should no longer have tab stop
        assert!(!processor.terminal_state.tab_stops[8]);

        // Other default tab stops should remain
        assert!(processor.terminal_state.tab_stops[16]);
        assert!(processor.terminal_state.tab_stops[24]);
    }

    /// F0313: Test TBC g (CSI 3 g) - Clear all tab stops
    #[test]
    fn test_clear_all_tab_stops() {
        let mut processor = AnsiProcessor::new();

        // Verify default tab stops exist
        assert!(processor.terminal_state.tab_stops[8]);
        assert!(processor.terminal_state.tab_stops[16]);
        assert!(processor.terminal_state.tab_stops[24]);

        // Clear all tab stops (CSI 3 g)
        processor.process_bytes(b"\x1b[3g");

        // All tab stops should be cleared
        for &stop in &processor.terminal_state.tab_stops {
            assert!(!stop);
        }
    }

    /// F0313: Test CHT (CSI I) - Cursor Horizontal Tabulation (tab forward)
    #[test]
    fn test_csi_tab_forward() {
        let mut processor = AnsiProcessor::new();

        // Start at column 0
        processor.terminal_state.get_cursor_mut().x = 0;

        // Tab forward using CSI I
        processor.process_bytes(b"\x1b[I");
        assert_eq!(processor.terminal_state.get_cursor().x, 8);

        // Tab forward 2 positions using CSI 2 I
        processor.process_bytes(b"\x1b[2I");
        assert_eq!(processor.terminal_state.get_cursor().x, 24);
    }

    /// F0313: Test CBT (CSI Z) - Cursor Backward Tabulation (tab backward)
    #[test]
    fn test_csi_tab_backward() {
        let mut processor = AnsiProcessor::new();

        // Start at column 20
        processor.terminal_state.get_cursor_mut().x = 20;

        // Tab backward using CSI Z
        processor.process_bytes(b"\x1b[Z");
        assert_eq!(processor.terminal_state.get_cursor().x, 16);

        // Tab backward 2 positions using CSI 2 Z
        processor.process_bytes(b"\x1b[2Z");
        assert_eq!(processor.terminal_state.get_cursor().x, 0);
    }

    /// F0313: Test horizontal tab character (\t) uses proper tab stops
    #[test]
    fn test_horizontal_tab_character() {
        let mut processor = AnsiProcessor::new();

        // Start at column 0
        processor.terminal_state.get_cursor_mut().x = 0;

        // Send horizontal tab character
        processor.process_bytes(b"\t");
        assert_eq!(processor.terminal_state.get_cursor().x, 8);

        // Another tab should go to column 16
        processor.process_bytes(b"\t");
        assert_eq!(processor.terminal_state.get_cursor().x, 16);
    }

    /// F0313: Test tab stops with custom positions
    #[test]
    fn test_custom_tab_stops() {
        let mut processor = AnsiProcessor::new();

        // Clear all default tab stops
        processor.process_bytes(b"\x1b[3g");

        // Set custom tab stops at columns 5, 15, 25
        processor.terminal_state.get_cursor_mut().x = 5;
        processor.process_bytes(b"\x1bH");

        processor.terminal_state.get_cursor_mut().x = 15;
        processor.process_bytes(b"\x1bH");

        processor.terminal_state.get_cursor_mut().x = 25;
        processor.process_bytes(b"\x1bH");

        // Test navigation with custom tab stops
        processor.terminal_state.get_cursor_mut().x = 0;
        processor.process_bytes(b"\t"); // Should go to 5
        assert_eq!(processor.terminal_state.get_cursor().x, 5);

        processor.process_bytes(b"\t"); // Should go to 15
        assert_eq!(processor.terminal_state.get_cursor().x, 15);

        processor.process_bytes(b"\t"); // Should go to 25
        assert_eq!(processor.terminal_state.get_cursor().x, 25);
    }

    /// F0313: Test tab stops after terminal resize
    #[test]
    fn test_tab_stops_after_resize() {
        let mut terminal_state = TerminalState::new(40, 24);

        // Check initial tab stops for 40-column terminal
        assert!(terminal_state.tab_stops[8]);
        assert!(terminal_state.tab_stops[16]);
        assert!(terminal_state.tab_stops[24]);
        assert!(terminal_state.tab_stops[32]);

        // Resize to 120 columns
        terminal_state.resize(120, 24);

        // Check that tab stops were reset with new width
        assert_eq!(terminal_state.tab_stops.len(), 120);
        assert!(terminal_state.tab_stops[8]);
        assert!(terminal_state.tab_stops[16]);
        assert!(terminal_state.tab_stops[24]);
        assert!(terminal_state.tab_stops[32]);
        assert!(terminal_state.tab_stops[40]);
        assert!(terminal_state.tab_stops[48]);
        assert!(terminal_state.tab_stops[56]);
        assert!(terminal_state.tab_stops[64]);
        assert!(terminal_state.tab_stops[72]);
        assert!(terminal_state.tab_stops[80]);
        assert!(terminal_state.tab_stops[88]);
        assert!(terminal_state.tab_stops[96]);
        assert!(terminal_state.tab_stops[104]);
        assert!(terminal_state.tab_stops[112]);
    }

    /// F0313: Test tab navigation edge cases
    #[test]
    fn test_tab_navigation_edge_cases() {
        let mut terminal_state = TerminalState::new(80, 24);

        // Test tab forward at exact tab stop position
        terminal_state.get_cursor_mut().x = 8;
        terminal_state.tab_forward();
        assert_eq!(terminal_state.get_cursor().x, 16);

        // Test tab backward at exact tab stop position
        terminal_state.get_cursor_mut().x = 16;
        terminal_state.tab_backward();
        assert_eq!(terminal_state.get_cursor().x, 8);

        // Test tab forward at last column
        terminal_state.get_cursor_mut().x = 79;
        terminal_state.tab_forward();
        assert_eq!(terminal_state.get_cursor().x, 79); // Should stay at end

        // Test tab backward at first column
        terminal_state.get_cursor_mut().x = 0;
        terminal_state.tab_backward();
        assert_eq!(terminal_state.get_cursor().x, 0); // Should stay at beginning
    }
}
