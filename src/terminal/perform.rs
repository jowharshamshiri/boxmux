//! What each control sequence does to the terminal.

use super::*;
use vte::{Params, Perform};

impl Perform for Emulator {
    fn print(&mut self, c: char) {
        let width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if width == 0 {
            // Combining marks and other zero-width characters have no cell of their own.
            return;
        }
        let screen_width = self.terminal_state.screen_width;
        let auto_wrap = self.terminal_state.mode.auto_wrap_mode;
        // A character that does not fit on the rest of the line wraps first, like on a real
        // terminal: either the previous one filled the line, or this one is wide.
        let fits = self.terminal_state.get_cursor().x + width <= screen_width;
        if self.terminal_state.wrap_pending || !fits {
            if auto_wrap {
                self.terminal_state.get_cursor_mut().x = 0;
                self.terminal_state.line_feed_with_scrolling();
            } else {
                self.terminal_state.get_cursor_mut().x = screen_width.saturating_sub(width);
            }
            self.terminal_state.wrap_pending = false;
        }

        // F0308: Handle insert mode - shift characters right before inserting
        if self.terminal_state.mode.insert_mode {
            for _ in 0..width {
                self.terminal_state.insert_character_at_cursor();
            }
        }

        self.put_char_at_cursor(c, width);
        let next = self.terminal_state.get_cursor().x + width;
        if next >= screen_width {
            // The cursor stays on the last column until the next character arrives.
            self.terminal_state.get_cursor_mut().x = screen_width.saturating_sub(1);
            self.terminal_state.wrap_pending = auto_wrap;
        } else {
            self.terminal_state.get_cursor_mut().x = next;
        }
    }

    fn execute(&mut self, byte: u8) {
        if matches!(byte, b'\n' | 0x0B | 0x0C | b'\r' | b'\t' | 0x08) {
            self.terminal_state.wrap_pending = false;
        }
        match byte {
            // Line feed, vertical tab and form feed all move down one line; returning to the
            // first column is the carriage return's job.
            b'\n' | 0x0B | 0x0C => self.terminal_state.line_feed_with_scrolling(),
            b'\r' => {
                self.terminal_state.get_cursor_mut().x = 0;
            }
            b'\t' => {
                // F0313: Use proper tab stops for horizontal tab
                self.terminal_state.tab_forward();
            }
            0x08 => {
                // Backspace moves left; it does not erase.
                let cursor = self.terminal_state.get_cursor_mut();
                cursor.x = cursor.x.saturating_sub(1);
            }
            0x0E => {
                // F0318: SO (Shift Out) - Switch to G1 character set
                self.terminal_state.active_charset = 1;
            }
            0x0F => {
                // F0318: SI (Shift In) - Switch to G0 character set
                self.terminal_state.active_charset = 0;
            }
            _ => {
                // Other control characters - for now just ignore
            }
        }
    }

    fn hook(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, c: char) {
        // Device Control String (DCS) - Enhanced implementation
        match c {
            'q' => {
                // DCS sequences starting with 'q' - terminal queries
                if intermediates.is_empty() && !params.is_empty() {
                    let query_type = Self::get_param(params, 0);
                    if query_type == 1 {
                        // Request terminal ID
                        self.terminal_state
                            .pending_responses
                            .push("\x1b[>0;276;0c".to_string());
                    }
                }
            }
            '+' => {
                // DCS + sequences - extended capabilities
                // Store for processing in put() method
                self.terminal_state.dcs_sequence_type = Some(c);
            }
            _ => {
                // Store DCS sequence type for processing
                self.terminal_state.dcs_sequence_type = Some(c);
            }
        }
    }

    fn put(&mut self, byte: u8) {
        // Put character in DCS - Enhanced implementation
        if self.terminal_state.dcs_buffer.is_none() {
            self.terminal_state.dcs_buffer = Some(Vec::new());
        }
        if let Some(ref mut buffer) = self.terminal_state.dcs_buffer {
            buffer.push(byte);
            // Limit DCS buffer size to prevent memory issues
            if buffer.len() > 4096 {
                buffer.clear();
                self.terminal_state.dcs_buffer = None;
            }
        }
    }

    fn unhook(&mut self) {
        // End of DCS - Process accumulated DCS sequence
        // Extract data first to avoid borrow checker issues
        let dcs_data = self
            .terminal_state
            .dcs_buffer
            .as_ref()
            .map(|buffer| String::from_utf8_lossy(buffer).to_string());

        if let Some(dcs_type) = self.terminal_state.dcs_sequence_type {
            if let Some(data) = dcs_data {
                match dcs_type {
                    'q' => {
                        // Process terminal queries
                        self.terminal_state.process_dcs_query(&data);
                    }
                    '+' => {
                        // Process extended capabilities
                        self.terminal_state.process_dcs_extended(&data);
                    }
                    _ => {
                        // Log unknown DCS sequence for debugging
                        log::debug!("Unknown DCS sequence type: {}", dcs_type);
                    }
                }
            }
        }
        // Clean up DCS state
        self.terminal_state.dcs_sequence_type = None;
        self.terminal_state.dcs_buffer = None;
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        let text = |i: usize| {
            params
                .get(i)
                .map(|p| String::from_utf8_lossy(p).into_owned())
        };
        let Some(command) = text(0) else {
            return;
        };
        // The data may itself contain semicolons, which the parser split on.
        let data = || {
            params
                .get(1..)
                .unwrap_or_default()
                .iter()
                .map(|p| String::from_utf8_lossy(p).into_owned())
                .collect::<Vec<_>>()
                .join(";")
        };
        match command.as_str() {
            "0" | "2" => {
                self.terminal_state.terminal_title = Some(data());
                self.terminal_state.title_changed = true;
            }
            "1" => self.terminal_state.icon_name = Some(data()),
            "52" => {
                // `52 ; targets ; base64` sets the clipboard. A `?` asks for its contents,
                // which programs are not given.
                let Some(payload) = text(2) else {
                    return;
                };
                if payload == "?" {
                    log::debug!("refused a clipboard read request");
                    return;
                }
                match crate::base64::decode(&payload)
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                {
                    Some(copied) => self.terminal_state.clipboard_writes.push(copied),
                    None => log::debug!("ignored a clipboard write that is not base64 UTF-8 text"),
                }
            }
            other => log::debug!("unhandled OSC {}", other),
        }
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        // Moving the cursor cancels a pending wrap at the last column.
        if matches!(
            c,
            'H' | 'f' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'd' | '`' | 'a' | 'e' | 'r' | 'u'
        ) {
            self.terminal_state.wrap_pending = false;
        }
        match c {
            'm' => {
                // SGR - Set Graphics Rendition (colors, bold, etc.)
                // F0306: Use enhanced SGR sequence processing
                self.apply_sgr_sequence(params);
            }
            'H' | 'f' => {
                // Cursor position - CUP or HVP
                let row = if !params.is_empty() {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                let col = if params.len() > 1 {
                    Self::get_param(params, 1).max(1)
                } else {
                    1
                };

                // Direct cursor positioning often indicates full-screen programs
                if row == 1 && col == 1 {
                    // Home cursor position - very common in full-screen apps
                    self.terminal_state.full_screen_program_detected = true;
                }

                // F0305: Use cursor management system
                self.terminal_state.set_cursor_position(
                    (col.saturating_sub(1)) as usize,
                    (row.saturating_sub(1)) as usize,
                );
            }
            'A' => {
                // Cursor up - CUU
                let n = if !params.is_empty() {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                // F0305: Use cursor management system
                self.terminal_state.cursor_up(n as usize);
            }
            'B' => {
                // Cursor down - CUD
                let n = if !params.is_empty() {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                // F0305: Use cursor management system
                self.terminal_state.cursor_down(n as usize);
            }
            'C' => {
                // Cursor forward - CUF
                let n = if !params.is_empty() {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                // F0305: Use cursor management system
                self.terminal_state.cursor_right(n as usize);
            }
            'D' => {
                // Cursor backward - CUB
                let n = if !params.is_empty() {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                // F0305: Use cursor management system
                self.terminal_state.cursor_left(n as usize);
            }
            'J' => {
                // Erase in Display
                let n = if !params.is_empty() {
                    Self::get_param(params, 0)
                } else {
                    0
                };

                // Clear screen commands often indicate full-screen programs
                if n == 2 || n == 1 {
                    self.terminal_state.full_screen_program_detected = true;
                }

                let cursor_x = self.terminal_state.get_cursor().x;
                let cursor_y = self.terminal_state.get_cursor().y;
                let screen_width = self.terminal_state.screen_width;
                let screen_height = self.terminal_state.screen_height;
                match n {
                    0 => {
                        // From the cursor to the end of the screen.
                        let screen = self.terminal_state.get_active_screen();
                        for (y, row) in screen.iter_mut().enumerate().skip(cursor_y) {
                            let start_x = if y == cursor_y { cursor_x } else { 0 };
                            for cell in row.iter_mut().take(screen_width).skip(start_x) {
                                *cell = TerminalCell::default();
                            }
                        }
                    }
                    1 => {
                        // From the start of the screen to the cursor, inclusive.
                        let screen = self.terminal_state.get_active_screen();
                        for (y, row) in screen
                            .iter_mut()
                            .enumerate()
                            .take(cursor_y.min(screen_height - 1) + 1)
                        {
                            let end_x = if y == cursor_y {
                                cursor_x + 1
                            } else {
                                screen_width
                            };
                            for cell in row.iter_mut().take(end_x.min(screen_width)) {
                                *cell = TerminalCell::default();
                            }
                        }
                    }
                    2 => self.terminal_state.clear_screen(),
                    3 => self.terminal_state.get_active_buffer().scrollback.clear(),
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line
                let n = if !params.is_empty() {
                    Self::get_param(params, 0)
                } else {
                    0
                };

                if self.terminal_state.get_cursor().y < self.terminal_state.screen_height {
                    // Extract values to avoid borrow checker issues
                    let cursor_x = self.terminal_state.get_cursor().x;
                    let cursor_y = self.terminal_state.get_cursor().y;
                    let screen_width = self.terminal_state.screen_width;

                    let row = &mut self.terminal_state.get_active_screen()[cursor_y];
                    let cleared = match n {
                        // From the cursor to the end of the line.
                        0 => cursor_x..screen_width,
                        // From the start of the line to the cursor, inclusive.
                        1 => 0..(cursor_x + 1).min(screen_width),
                        // The whole line.
                        2 => 0..screen_width,
                        _ => 0..0,
                    };
                    for cell in &mut row[cleared] {
                        *cell = TerminalCell::default();
                    }
                }
            }
            'h' => {
                // Set Mode
                if _intermediates.contains(&b'?') && !params.is_empty() {
                    // DEC private mode set
                    let mode = Self::get_param(params, 0);
                    match mode {
                        1 => {
                            // DECCKM - Application Cursor Keys
                            self.terminal_state.mode.cursor_key_mode = true;
                        }
                        6 => {
                            // DECOM - Origin Mode
                            self.terminal_state.mode.origin_mode = true;
                            // Move cursor to origin
                            self.terminal_state.get_cursor_mut().x = 0;
                            self.terminal_state.get_cursor_mut().y =
                                self.terminal_state.scroll_region.top;
                        }
                        7 => {
                            // DECAWM - Auto Wrap Mode
                            self.terminal_state.mode.auto_wrap_mode = true;
                        }
                        25 => {
                            // DECTCEM - Cursor Visibility
                            self.terminal_state.get_cursor_mut().visible = true;
                        }
                        1000 => {
                            // F0310: Mouse reporting - X10 mode
                            self.terminal_state.mode.mouse_x10_mode = true;
                            // Clear other mouse modes (mutually exclusive)
                            self.terminal_state.mode.mouse_normal_mode = false;
                            self.terminal_state.mode.mouse_button_event_mode = false;
                            self.terminal_state.mode.mouse_any_event_mode = false;
                            log::debug!("F0310: Mouse X10 mode enabled");
                        }
                        1002 => {
                            // F0310: Mouse reporting - Button Event mode
                            self.terminal_state.mode.mouse_button_event_mode = true;
                            // Clear other mouse modes (mutually exclusive)
                            self.terminal_state.mode.mouse_x10_mode = false;
                            self.terminal_state.mode.mouse_normal_mode = false;
                            self.terminal_state.mode.mouse_any_event_mode = false;
                            log::debug!("F0310: Mouse Button Event mode enabled");
                        }
                        1003 => {
                            // F0310: Mouse reporting - Any Event mode
                            self.terminal_state.mode.mouse_any_event_mode = true;
                            // Clear other mouse modes (mutually exclusive)
                            self.terminal_state.mode.mouse_x10_mode = false;
                            self.terminal_state.mode.mouse_normal_mode = false;
                            self.terminal_state.mode.mouse_button_event_mode = false;
                            log::debug!("F0310: Mouse Any Event mode enabled");
                        }
                        1006 => {
                            // F0310: Mouse reporting - SGR mode
                            self.terminal_state.mode.mouse_sgr_mode = true;
                            log::debug!("F0310: Mouse SGR extended reporting enabled");
                        }
                        47 => {
                            // F0312: Simple Alternate Screen (no cursor save, no clear)
                            self.terminal_state.full_screen_program_detected = true;
                            self.terminal_state.switch_to_alternate_screen();
                        }
                        1047 => {
                            // F0312: Alternate Screen Buffer (with clear, no cursor save)
                            self.terminal_state.full_screen_program_detected = true;
                            self.terminal_state.switch_to_alternate_screen();
                            self.terminal_state.clear_alternate_screen();
                        }
                        1049 => {
                            // F0312: Alternate Screen Buffer (with cursor save/restore and clear)
                            self.terminal_state.full_screen_program_detected = true;
                            // Save cursor position before switching
                            self.terminal_state.save_cursor();
                            self.terminal_state.switch_to_alternate_screen();
                            self.terminal_state.clear_alternate_screen();
                        }
                        2004 => {
                            // F0310: Bracketed paste mode
                            self.terminal_state.mode.bracketed_paste_mode = true;
                            log::debug!("F0310: Bracketed paste mode enabled");
                        }
                        _ => {
                            log::debug!("Unknown DEC private mode set: {}", mode);
                        }
                    }
                } else if !params.is_empty() {
                    // Standard mode set
                    let mode = Self::get_param(params, 0);
                    match mode {
                        4 => {
                            // IRM - Insert/Replace Mode
                            self.terminal_state.mode.insert_mode = true;
                        }
                        20 => {
                            // LNM - Linefeed/Newline Mode
                            self.terminal_state.mode.local_echo_mode = true;
                        }
                        _ => {
                            log::debug!("Unknown standard mode set: {}", mode);
                        }
                    }
                }
            }
            'l' => {
                // Reset Mode
                if _intermediates.contains(&b'?') && !params.is_empty() {
                    // DEC private mode reset
                    let mode = Self::get_param(params, 0);
                    match mode {
                        1 => {
                            // DECCKM - Normal Cursor Keys
                            self.terminal_state.mode.cursor_key_mode = false;
                        }
                        6 => {
                            // DECOM - Absolute Origin Mode
                            self.terminal_state.mode.origin_mode = false;
                            // Move cursor to absolute origin
                            self.terminal_state.get_cursor_mut().x = 0;
                            self.terminal_state.get_cursor_mut().y = 0;
                        }
                        7 => {
                            // DECAWM - No Auto Wrap
                            self.terminal_state.mode.auto_wrap_mode = false;
                        }
                        25 => {
                            // DECTCEM - Cursor Invisible
                            self.terminal_state.get_cursor_mut().visible = false;
                        }
                        1000 => {
                            // F0310: Mouse reporting - X10 mode disable
                            self.terminal_state.mode.mouse_x10_mode = false;
                            log::debug!("F0310: Mouse X10 mode disabled");
                        }
                        1002 => {
                            // F0310: Mouse reporting - Button Event mode disable
                            self.terminal_state.mode.mouse_button_event_mode = false;
                            log::debug!("F0310: Mouse Button Event mode disabled");
                        }
                        1003 => {
                            // F0310: Mouse reporting - Any Event mode disable
                            self.terminal_state.mode.mouse_any_event_mode = false;
                            log::debug!("F0310: Mouse Any Event mode disabled");
                        }
                        1006 => {
                            // F0310: Mouse reporting - SGR mode disable
                            self.terminal_state.mode.mouse_sgr_mode = false;
                            log::debug!("F0310: Mouse SGR extended reporting disabled");
                        }
                        47 => {
                            // F0312: Exit Simple Alternate Screen (no cursor restore)
                            self.terminal_state.switch_to_primary_screen();
                            // Keep full_screen_program_detected true once detected
                        }
                        1047 => {
                            // F0312: Exit Alternate Screen Buffer (no cursor restore)
                            self.terminal_state.switch_to_primary_screen();
                            // Keep full_screen_program_detected true once detected
                        }
                        1049 => {
                            // F0312: Exit Alternate Screen Buffer (with cursor restore)
                            self.terminal_state.switch_to_primary_screen();
                            // Restore cursor position after switching back
                            self.terminal_state.restore_cursor();
                            // Keep full_screen_program_detected true once detected
                        }
                        2004 => {
                            // F0310: Bracketed paste mode disable
                            self.terminal_state.mode.bracketed_paste_mode = false;
                            log::debug!("F0310: Bracketed paste mode disabled");
                        }
                        _ => {
                            log::debug!("Unknown DEC private mode reset: {}", mode);
                        }
                    }
                } else if !params.is_empty() {
                    // Standard mode reset
                    let mode = Self::get_param(params, 0);
                    match mode {
                        4 => {
                            // IRM - Replace Mode
                            self.terminal_state.mode.insert_mode = false;
                        }
                        20 => {
                            // LNM - Linefeed Mode
                            self.terminal_state.mode.local_echo_mode = false;
                        }
                        _ => {
                            log::debug!("Unknown standard mode reset: {}", mode);
                        }
                    }
                }
            }
            'L' => {
                // Insert Lines (IL)
                let count = if !params.is_empty() {
                    Self::get_param(params, 0).max(1) as usize
                } else {
                    1
                };
                self.terminal_state.get_active_buffer().insert_lines(count);
            }
            'M' => {
                // Delete Lines (DL)
                let count = if !params.is_empty() {
                    Self::get_param(params, 0).max(1) as usize
                } else {
                    1
                };
                self.terminal_state.get_active_buffer().delete_lines(count);
            }
            'P' => {
                // Delete Characters (DCH)
                let count = if !params.is_empty() {
                    Self::get_param(params, 0).max(1) as usize
                } else {
                    1
                };
                // Simplified - just clear characters for now
                let cursor_x = self.terminal_state.get_cursor().x;
                let cursor_y = self.terminal_state.get_cursor().y;
                let screen_width = self.terminal_state.screen_width;
                let screen_height = self.terminal_state.screen_height;
                if cursor_y < screen_height {
                    let screen = self.terminal_state.get_active_screen();
                    for i in 0..count {
                        if cursor_x + i < screen_width {
                            screen[cursor_y][cursor_x + i] = TerminalCell::default();
                        }
                    }
                }
            }
            'S' => {
                // SU - Scroll Up
                // F0307: Scroll up within scrolling region
                let lines = if !params.is_empty() {
                    Self::get_param(params, 0).max(1) as usize
                } else {
                    1
                };
                self.terminal_state.scroll_up_in_region(lines);
            }
            'T' => {
                // SD - Scroll Down
                // F0307: Scroll down within scrolling region
                let lines = if !params.is_empty() {
                    Self::get_param(params, 0).max(1) as usize
                } else {
                    1
                };
                self.terminal_state.scroll_down_in_region(lines);
            }
            'r' => {
                // DECSTBM - Set Top and Bottom Margins (Set Scrolling Region)
                // F0307: Scrolling Region Support
                let top = if !params.is_empty() {
                    Some(Self::get_param(params, 0) as usize)
                } else {
                    None
                };
                let bottom = if params.len() > 1 {
                    Some(Self::get_param(params, 1) as usize)
                } else {
                    None
                };

                // Use F0307 scrolling region methods
                self.terminal_state.set_scrolling_region(top, bottom);

                // DECSTBM also moves cursor to home position within the scrolling region
                let region_top = self.terminal_state.scroll_region.top;
                self.terminal_state.set_cursor_position(0, region_top);
            }
            'n' => {
                // F0311: Device Status Report (DSR) - Enhanced with all standard reports
                if !params.is_empty() {
                    let report_type = Self::get_param(params, 0);
                    match report_type {
                        5 => {
                            // Terminal status - always ready
                            self.terminal_state
                                .pending_responses
                                .push("\x1b[0n".to_string());
                        }
                        6 => {
                            // Cursor Position Report (CPR)
                            let cursor_x = self.terminal_state.get_cursor().x + 1;
                            let cursor_y = self.terminal_state.get_cursor().y + 1;
                            self.terminal_state
                                .pending_responses
                                .push(format!("\x1b[{};{}R", cursor_y, cursor_x));
                        }
                        14 => {
                            // F0311: Terminal Size Report (TSR) - Window size in characters
                            let cols = self.terminal_state.screen_width;
                            let rows = self.terminal_state.screen_height;
                            self.terminal_state
                                .pending_responses
                                .push(format!("\x1b[8;{};{}t", rows, cols));
                        }
                        15 => {
                            // F0311: Terminal Size Report in pixels (if supported)
                            // Report approximate pixel size (assume 8x16 character cell)
                            let pixel_width = self.terminal_state.screen_width * 8;
                            let pixel_height = self.terminal_state.screen_height * 16;
                            self.terminal_state
                                .pending_responses
                                .push(format!("\x1b[4;{};{}t", pixel_height, pixel_width));
                        }
                        _ => {
                            log::debug!("Unknown DSR request: {}", report_type);
                        }
                    }
                } else if params.is_empty() {
                    // Default DSR - cursor position report
                    let cursor_x = self.terminal_state.get_cursor().x + 1;
                    let cursor_y = self.terminal_state.get_cursor().y + 1;
                    self.terminal_state
                        .pending_responses
                        .push(format!("\x1b[{};{}R", cursor_y, cursor_x));
                }
            }
            'c' => {
                // F0311: Device Attributes (DA) - Enhanced terminal identification
                if params.is_empty() || Self::get_param(params, 0) == 0 {
                    // Primary Device Attributes - VT102/VT220 compatible terminal
                    // ?1 = VT101/VT102; ?6 = VT102; ?62 = VT220; ?63 = VT320; ?64 = VT420; ?65 = VT520
                    self.terminal_state
                        .pending_responses
                        .push("\x1b[?62;1;2;6;7;8;9c".to_string());
                } else if Self::get_param(params, 0) == 1 {
                    // Secondary Device Attributes - Terminal version and capabilities
                    // Format: ESC[>Pp;Pv;Pc where:
                    // Pp = Terminal type (0 = VT100-series)
                    // Pv = Firmware version (276 = common version number)
                    // Pc = ROM cartridge registration number (0 = no cartridge)
                    self.terminal_state
                        .pending_responses
                        .push("\x1b[>0;276;0c".to_string());
                } else if Self::get_param(params, 0) == 2 {
                    // Tertiary Device Attributes - Unit ID (rarely supported)
                    self.terminal_state
                        .pending_responses
                        .push("\x1b[P!|00000000".to_string());
                }
            }
            'x' => {
                // F0311: Request Terminal Parameters (DECREQTPARM)
                if params.is_empty() || Self::get_param(params, 0) == 0 {
                    // Solicit terminal parameters report
                    self.terminal_state
                        .pending_responses
                        .push("\x1b[2;1;1;128;128;1;0x".to_string());
                } else if Self::get_param(params, 0) == 1 {
                    // Request terminal parameters report
                    self.terminal_state
                        .pending_responses
                        .push("\x1b[3;1;1;128;128;1;0x".to_string());
                }
            }
            '$' => {
                // F0311: Request Mode (DECRQM) - Query terminal mode status
                if let Some(intermediate) = _intermediates.first() {
                    if *intermediate == b'?' {
                        // DEC private mode query
                        let mode = Self::get_param(params, 0);
                        let status = self.get_private_mode_status(mode);
                        self.terminal_state
                            .pending_responses
                            .push(format!("\x1b[?{};{}$y", mode, status));
                    } else {
                        // Standard mode query
                        let mode = Self::get_param(params, 0);
                        let status = self.get_standard_mode_status(mode);
                        self.terminal_state
                            .pending_responses
                            .push(format!("\x1b[{};{}$y", mode, status));
                    }
                } else {
                    // Standard mode query (no intermediate)
                    let mode = Self::get_param(params, 0);
                    let status = self.get_standard_mode_status(mode);
                    self.terminal_state
                        .pending_responses
                        .push(format!("\x1b[{};{}$y", mode, status));
                }
            }
            'g' => {
                // F0313: TBC - Tab Clear
                let mode = if !params.is_empty() {
                    Self::get_param(params, 0)
                } else {
                    0
                };

                match mode {
                    0 => {
                        // Clear tab stop at current column
                        let cursor_x = self.terminal_state.get_cursor().x;
                        if cursor_x < self.terminal_state.tab_stops.len() {
                            self.terminal_state.tab_stops[cursor_x] = false;
                        }
                    }
                    3 => {
                        // Clear all tab stops
                        self.terminal_state.tab_stops.fill(false);
                    }
                    _ => {
                        log::debug!("Unknown TBC mode: {}", mode);
                    }
                }
            }
            'I' => {
                // F0313: CHT - Cursor Horizontal Tabulation (Tab Forward)
                let count = if !params.is_empty() {
                    Self::get_param(params, 0).max(1) as usize
                } else {
                    1
                };

                for _ in 0..count {
                    self.terminal_state.tab_forward();
                }
            }
            'Z' => {
                // F0313: CBT - Cursor Backward Tabulation (Tab Backward)
                let count = if !params.is_empty() {
                    Self::get_param(params, 0).max(1) as usize
                } else {
                    1
                };

                for _ in 0..count {
                    self.terminal_state.tab_backward();
                }
            }
            _ => {
                log::debug!("Unknown CSI sequence: {:?} '{}'", params, { c });
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        if intermediates.is_empty() && matches!(byte, b'8' | b'D' | b'E' | b'M') {
            self.terminal_state.wrap_pending = false;
        }
        match byte {
            b'7' => {
                // Save cursor position (DECSC)
                self.terminal_state.save_cursor();
            }
            b'8' => {
                // Restore cursor position (DECRC)
                self.terminal_state.restore_cursor();
            }
            b'c' => {
                // RIS: back to the power-on state, keeping the size.
                let (width, height) = (
                    self.terminal_state.screen_width,
                    self.terminal_state.screen_height,
                );
                self.terminal_state = TerminalState::new(width, height);
            }
            b'Z' => {
                // F0311: Identify Terminal (DECID) - respond with device attributes
                self.terminal_state
                    .pending_responses
                    .push("\x1b[?62;1;2;6;7;8;9c".to_string());
            }
            5 => {
                // F0311: ENQ (Enquiry) - respond with ANSWERBACK string
                self.terminal_state
                    .pending_responses
                    .push("BoxMux Terminal\r".to_string());
            }
            b'D' => {
                // Index (IND) - cursor down with scroll
                if self.terminal_state.get_cursor().y
                    >= self.terminal_state.screen_height.saturating_sub(1)
                {
                    self.terminal_state.scroll_up(1);
                } else {
                    self.terminal_state.get_cursor_mut().y += 1;
                }
            }
            b'M' => {
                // Reverse Index (RI) - cursor up with scroll
                if self.terminal_state.get_cursor().y == 0 {
                    self.terminal_state.scroll_down(1);
                } else {
                    self.terminal_state.get_cursor_mut().y =
                        self.terminal_state.get_cursor().y.saturating_sub(1);
                }
            }
            b'E' => {
                // Next Line (NEL) - cursor to start of next line
                self.terminal_state.get_cursor_mut().x = 0;
                if self.terminal_state.get_cursor().y
                    >= self.terminal_state.screen_height.saturating_sub(1)
                {
                    self.terminal_state.scroll_up(1);
                } else {
                    self.terminal_state.get_cursor_mut().y += 1;
                }
            }
            b'H' => {
                // Tab Set (HTS)
                let cursor_x = self.terminal_state.get_cursor().x;
                if cursor_x < self.terminal_state.tab_stops.len() {
                    self.terminal_state.tab_stops[cursor_x] = true;
                }
            }
            b'=' => {
                // Application Keypad (DECKPAM)
                self.terminal_state.mode.keypad_mode = true;
            }
            b'>' => {
                // Normal Keypad (DECKPNM)
                self.terminal_state.mode.keypad_mode = false;
            }
            _ => {
                // Handle intermediate-dependent sequences
                if intermediates.len() == 1 {
                    match intermediates[0] {
                        b'(' => {
                            // Designate G0 Character Set
                            match byte {
                                b'A' => self.terminal_state.charset_g0 = Some("UK".to_string()),
                                b'B' => self.terminal_state.charset_g0 = Some("ASCII".to_string()),
                                b'0' => self.terminal_state.charset_g0 = Some("DEC".to_string()),
                                _ => {}
                            }
                        }
                        b')' => {
                            // Designate G1 Character Set
                            match byte {
                                b'A' => self.terminal_state.charset_g1 = Some("UK".to_string()),
                                b'B' => self.terminal_state.charset_g1 = Some("ASCII".to_string()),
                                b'0' => self.terminal_state.charset_g1 = Some("DEC".to_string()),
                                _ => {}
                            }
                        }
                        b'#' => {
                            // F0314: Line attribute sequences
                            match byte {
                                b'3' => {
                                    // DECDHL - Double Height Line (top half)
                                    self.terminal_state.set_double_height_line_top();
                                }
                                b'4' => {
                                    // DECDHL - Double Height Line (bottom half)
                                    self.terminal_state.set_double_height_line_bottom();
                                }
                                b'5' => {
                                    // DECSWL - Single Width Line (normal)
                                    self.terminal_state.set_normal_line();
                                }
                                b'6' => {
                                    // DECDWL - Double Width Line
                                    self.terminal_state.set_double_width_line();
                                }
                                _ => {
                                    log::debug!("Unknown ESC # sequence: {}", byte as char);
                                }
                            }
                        }
                        _ => {
                            log::debug!(
                                "Unknown ESC intermediate sequence: {:?} {}",
                                intermediates,
                                byte as char
                            );
                        }
                    }
                } else {
                    log::debug!("Unknown ESC sequence: {:?} {}", intermediates, byte as char);
                }
            }
        }
    }
}
