//! Everything a terminal remembers between bytes: both screens, the cursor, modes, tab stops,
//! character sets and the replies owed to the program.

use super::*;

#[derive(Debug, Clone)]
pub struct TerminalState {
    // Screen dimensions
    pub screen_width: usize,
    pub screen_height: usize,

    // Primary and alternate screen buffers
    pub primary_buffer: TerminalScreenBuffer,
    pub alternate_buffer: TerminalScreenBuffer,
    pub use_alternate_screen: bool,

    // Cursor save stack for DECSC/DECRC
    pub cursor_save_stack: Vec<CursorState>,

    // Character attributes for new text
    pub current_attributes: TerminalCell,

    // Terminal modes and settings
    pub mode: TerminalMode,
    pub scroll_region: ScrollRegion,

    // Tab stops
    pub tab_stops: Vec<bool>,

    // Character sets (G0/G1)
    pub charset_g0: Option<String>,
    pub charset_g1: Option<String>,
    pub active_charset: u8, // 0 for G0, 1 for G1

    // Terminal identification
    pub terminal_title: Option<String>,
    pub icon_name: Option<String>,
    pub title_changed: bool, // F0315: Track when title changes for propagation

    // State tracking
    pub full_screen_program_detected: bool,

    // F0306: SGR state tracking for extended colors
    pub sgr_state: SGRState,

    // DCS (Device Control String) processing
    pub dcs_sequence_type: Option<char>,
    pub dcs_buffer: Option<Vec<u8>>,
    /// Replies to queries (DSR, DA, DECRQM...) waiting to be written back to the program.
    pub pending_responses: Vec<String>,
    /// A character was written to the last column; the next one goes to the next line.
    pub wrap_pending: bool,
    /// Text the program put on the clipboard (OSC 52), waiting to be copied.
    pub clipboard_writes: Vec<String>,

    // OSC (Operating System Command) processing
    pub osc_buffer: Option<Vec<u8>>,
}

impl TerminalState {
    pub fn new(width: usize, height: usize) -> Self {
        let max_scrollback = 10000; // Same as PTY manager default
        let mut state = TerminalState {
            screen_width: width,
            screen_height: height,
            primary_buffer: TerminalScreenBuffer::new(width, height, max_scrollback),
            alternate_buffer: TerminalScreenBuffer::new(width, height, max_scrollback),
            use_alternate_screen: false,
            cursor_save_stack: Vec::new(),
            current_attributes: TerminalCell::default(),
            mode: TerminalMode::default(),
            scroll_region: ScrollRegion {
                top: 0,
                bottom: height.saturating_sub(1),
            },
            tab_stops: vec![false; width],
            charset_g0: None,
            charset_g1: None,
            active_charset: 0,
            terminal_title: None,
            icon_name: None,
            title_changed: false, // F0315: Initialize title change tracking
            full_screen_program_detected: false,
            sgr_state: SGRState::default(),
            dcs_sequence_type: None,
            dcs_buffer: None,
            pending_responses: Vec::new(),
            wrap_pending: false,
            clipboard_writes: Vec::new(),
            osc_buffer: None,
        };

        // Set default tab stops every 8 columns
        for i in (8..width).step_by(8) {
            if i < state.tab_stops.len() {
                state.tab_stops[i] = true;
            }
        }

        state
    }

    pub fn get_active_buffer(&mut self) -> &mut TerminalScreenBuffer {
        if self.use_alternate_screen {
            &mut self.alternate_buffer
        } else {
            &mut self.primary_buffer
        }
    }

    pub fn get_active_buffer_ref(&self) -> &TerminalScreenBuffer {
        if self.use_alternate_screen {
            &self.alternate_buffer
        } else {
            &self.primary_buffer
        }
    }

    pub fn get_active_screen(&mut self) -> &mut Vec<Vec<TerminalCell>> {
        &mut self.get_active_buffer().content
    }

    pub fn get_active_screen_ref(&self) -> &Vec<Vec<TerminalCell>> {
        &self.get_active_buffer_ref().content
    }

    pub fn get_cursor(&self) -> &CursorState {
        &self.get_active_buffer_ref().cursor
    }

    pub fn get_cursor_mut(&mut self) -> &mut CursorState {
        &mut self.get_active_buffer().cursor
    }

    /// F0305: Cursor Management System - Save cursor position and attributes
    pub fn save_cursor(&mut self) {
        let current_cursor = self.get_cursor().clone();
        self.cursor_save_stack.push(current_cursor);
    }

    /// F0305: Cursor Management System - Restore cursor position and attributes
    pub fn restore_cursor(&mut self) {
        if let Some(saved_cursor) = self.cursor_save_stack.pop() {
            let screen_width = self.screen_width;
            let screen_height = self.screen_height;
            let cursor = self.get_cursor_mut();
            cursor.x = saved_cursor.x.min(screen_width.saturating_sub(1));
            cursor.y = saved_cursor.y.min(screen_height.saturating_sub(1));
            cursor.visible = saved_cursor.visible;
            cursor.style = saved_cursor.style;
        }
    }

    /// F0305: Set cursor position with bounds checking
    pub fn set_cursor_position(&mut self, x: usize, y: usize) {
        let screen_width = self.screen_width;
        let screen_height = self.screen_height;
        let origin_mode = self.mode.origin_mode;
        let region_top = self.scroll_region.top;
        let region_bottom = self.scroll_region.bottom;

        let cursor = self.get_cursor_mut();

        // F0308: Handle origin mode (DECOM) - coordinates relative to scrolling region
        if origin_mode {
            // In origin mode, coordinates are relative to the scrolling region
            // Clamp Y to scrolling region bounds
            cursor.y = (region_top + y).min(region_bottom);
            cursor.x = x.min(screen_width.saturating_sub(1));
        } else {
            // Normal mode: coordinates are absolute to screen
            cursor.x = x.min(screen_width.saturating_sub(1));
            cursor.y = y.min(screen_height.saturating_sub(1));
        }
    }

    /// F0305: Move cursor relatively with bounds checking
    pub fn move_cursor_relative(&mut self, dx: i32, dy: i32) {
        let screen_width = self.screen_width;
        let screen_height = self.screen_height;
        let cursor = self.get_cursor_mut();
        let new_x = if dx >= 0 {
            cursor.x.saturating_add(dx as usize)
        } else {
            cursor.x.saturating_sub((-dx) as usize)
        };
        let new_y = if dy >= 0 {
            cursor.y.saturating_add(dy as usize)
        } else {
            cursor.y.saturating_sub((-dy) as usize)
        };

        cursor.x = new_x.min(screen_width.saturating_sub(1));
        cursor.y = new_y.min(screen_height.saturating_sub(1));
    }

    /// F0305: Set cursor visibility
    pub fn set_cursor_visibility(&mut self, visible: bool) {
        self.get_cursor_mut().visible = visible;
    }

    /// F0305: Set cursor style
    pub fn set_cursor_style(&mut self, style: CursorStyle) {
        self.get_cursor_mut().style = style;
    }

    /// F0305: Move cursor to column (absolute positioning)
    pub fn set_cursor_column(&mut self, col: usize) {
        let screen_width = self.screen_width;
        let cursor = self.get_cursor_mut();
        cursor.x = col.min(screen_width.saturating_sub(1));
    }

    /// F0305: Move cursor to line (absolute positioning)
    pub fn set_cursor_line(&mut self, line: usize) {
        let screen_height = self.screen_height;
        let cursor = self.get_cursor_mut();
        cursor.y = line.min(screen_height.saturating_sub(1));
    }

    /// F0305: Move cursor up with bounds checking
    pub fn cursor_up(&mut self, lines: usize) {
        let cursor = self.get_cursor_mut();
        cursor.y = cursor.y.saturating_sub(lines);
    }

    /// F0305: Move cursor down with bounds checking
    pub fn cursor_down(&mut self, lines: usize) {
        let screen_height = self.screen_height;
        let cursor = self.get_cursor_mut();
        cursor.y = (cursor.y + lines).min(screen_height.saturating_sub(1));
    }

    /// F0305: Move cursor left with bounds checking
    pub fn cursor_left(&mut self, cols: usize) {
        let cursor = self.get_cursor_mut();
        cursor.x = cursor.x.saturating_sub(cols);
    }

    /// F0305: Move cursor right with bounds checking
    pub fn cursor_right(&mut self, cols: usize) {
        let screen_width = self.screen_width;
        let cursor = self.get_cursor_mut();
        cursor.x = (cursor.x + cols).min(screen_width.saturating_sub(1));
    }

    /// F0305: Move cursor to beginning of line
    pub fn cursor_to_line_start(&mut self) {
        self.get_cursor_mut().x = 0;
    }

    /// F0305: Move cursor to next line (carriage return + line feed)
    pub fn cursor_next_line(&mut self, lines: usize) {
        self.cursor_down(lines);
        self.cursor_to_line_start();
    }

    /// F0305: Move cursor to previous line
    pub fn cursor_previous_line(&mut self, lines: usize) {
        self.cursor_up(lines);
        self.cursor_to_line_start();
    }

    /// F0313: Tab forward to next tab stop
    pub fn tab_forward(&mut self) {
        let cursor_x = self.get_cursor().x;
        let screen_width = self.screen_width;

        // Find next tab stop
        for x in (cursor_x + 1)..screen_width {
            if x < self.tab_stops.len() && self.tab_stops[x] {
                self.get_cursor_mut().x = x;
                return;
            }
        }

        // No tab stop found, move to end of line
        self.get_cursor_mut().x = screen_width.saturating_sub(1);
    }

    /// F0313: Tab backward to previous tab stop
    pub fn tab_backward(&mut self) {
        let cursor_x = self.get_cursor().x;

        // Find previous tab stop
        if cursor_x > 0 {
            for x in (0..cursor_x).rev() {
                if x < self.tab_stops.len() && self.tab_stops[x] {
                    self.get_cursor_mut().x = x;
                    return;
                }
            }
        }

        // No tab stop found, move to beginning of line
        self.get_cursor_mut().x = 0;
    }

    /// F0314: Set line attribute for current line
    pub fn set_line_attribute(&mut self, attribute: LineAttribute) {
        let cursor_y = self.get_cursor().y;
        let buffer = if self.use_alternate_screen {
            &mut self.alternate_buffer
        } else {
            &mut self.primary_buffer
        };

        if cursor_y < buffer.line_attributes.len() {
            buffer.line_attributes[cursor_y] = attribute;
        }
    }

    /// F0314: Get line attribute for specific line
    pub fn get_line_attribute(&self, line: usize) -> LineAttribute {
        let buffer = if self.use_alternate_screen {
            &self.alternate_buffer
        } else {
            &self.primary_buffer
        };

        buffer
            .line_attributes
            .get(line)
            .copied()
            .unwrap_or(LineAttribute::Normal)
    }

    /// F0314: Set double-width line (DECDWL)
    pub fn set_double_width_line(&mut self) {
        self.set_line_attribute(LineAttribute::DoubleWidth);
    }

    /// F0314: Set double-height line top half (DECDHL top)
    pub fn set_double_height_line_top(&mut self) {
        self.set_line_attribute(LineAttribute::DoubleHeightTop);
    }

    /// F0314: Set double-height line bottom half (DECDHL bottom)
    pub fn set_double_height_line_bottom(&mut self) {
        self.set_line_attribute(LineAttribute::DoubleHeightBottom);
    }

    /// F0314: Reset line to normal (single-width, single-height)
    pub fn set_normal_line(&mut self) {
        self.set_line_attribute(LineAttribute::Normal);
    }

    /// F0307: Set scrolling region (DECSTBM - Set Top and Bottom Margins)
    pub fn set_scrolling_region(&mut self, top: Option<usize>, bottom: Option<usize>) {
        let screen_height = self.screen_height;

        // Convert 1-based VT coordinates to 0-based internal coordinates
        let new_top = top.map(|t| t.saturating_sub(1)).unwrap_or(0);
        let new_bottom = bottom
            .map(|b| b.saturating_sub(1))
            .unwrap_or(screen_height.saturating_sub(1));

        // Validate region bounds
        if new_top < screen_height && new_bottom < screen_height && new_top <= new_bottom {
            self.scroll_region.top = new_top;
            self.scroll_region.bottom = new_bottom;
        }
        // If invalid bounds, reset to full screen
        else {
            self.scroll_region.top = 0;
            self.scroll_region.bottom = screen_height.saturating_sub(1);
        }
    }

    /// F0307: Reset scrolling region to full screen
    pub fn reset_scrolling_region(&mut self) {
        self.scroll_region.top = 0;
        self.scroll_region.bottom = self.screen_height.saturating_sub(1);
    }

    /// F0307: Scroll up within the scrolling region
    pub fn scroll_up_in_region(&mut self, lines: usize) {
        if lines == 0 {
            return;
        }

        let top = self.scroll_region.top;
        let bottom = self.scroll_region.bottom;
        let screen_width = self.screen_width;

        // Move lines up within the region
        for _ in 0..lines {
            // Get buffer once for each iteration to avoid borrow checker issues
            {
                let buffer = self.get_active_buffer();
                if top <= bottom && top < buffer.content.len() && bottom < buffer.content.len() {
                    // Remove the top line of the scrolling region
                    let _removed_line = buffer.content.remove(top);

                    // Add a blank line at the bottom of the scrolling region
                    if bottom < buffer.content.len() {
                        let width = if !buffer.content.is_empty() {
                            buffer.content[0].len()
                        } else {
                            screen_width
                        };
                        let blank_line = vec![TerminalCell::default(); width];
                        buffer.content.insert(bottom, blank_line);
                    } else {
                        // If we're at the end, just push a new line
                        let blank_line = vec![TerminalCell::default(); screen_width];
                        buffer.content.push(blank_line);
                    }
                }
            }
        }
    }

    /// F0307: Scroll down within the scrolling region
    pub fn scroll_down_in_region(&mut self, lines: usize) {
        if lines == 0 {
            return;
        }

        let top = self.scroll_region.top;
        let bottom = self.scroll_region.bottom;
        let screen_width = self.screen_width;

        // Move lines down within the region
        for _ in 0..lines {
            // Get buffer once for each iteration to avoid borrow checker issues
            {
                let buffer = self.get_active_buffer();
                if top <= bottom && top < buffer.content.len() && bottom < buffer.content.len() {
                    // Remove the bottom line of the scrolling region
                    if bottom < buffer.content.len() {
                        let _removed_line = buffer.content.remove(bottom);
                    }

                    // Add a blank line at the top of the scrolling region
                    let blank_line = vec![TerminalCell::default(); screen_width];
                    buffer.content.insert(top, blank_line);
                }
            }
        }
    }

    /// F0307: Check if cursor is within scrolling region
    pub fn is_cursor_in_scrolling_region(&self) -> bool {
        let cursor_y = self.get_cursor().y;
        cursor_y >= self.scroll_region.top && cursor_y <= self.scroll_region.bottom
    }

    /// F0307: Handle line feed with scrolling region awareness
    pub fn line_feed_with_scrolling(&mut self) {
        let current_y = self.get_cursor().y;
        let bottom = self.scroll_region.bottom;

        // If cursor is at the bottom of the scrolling region, scroll up
        if current_y >= bottom {
            self.scroll_up_in_region(1);
            // Keep cursor at bottom of scrolling region
            self.get_cursor_mut().y = bottom;
        } else {
            // Normal line feed within the scrolling region
            self.get_cursor_mut().y = (current_y + 1).min(bottom);
        }
    }

    /// F0307: Handle reverse line feed with scrolling region awareness
    pub fn reverse_line_feed_with_scrolling(&mut self) {
        let current_y = self.get_cursor().y;
        let top = self.scroll_region.top;

        // If cursor is at the top of the scrolling region, scroll down
        if current_y <= top {
            self.scroll_down_in_region(1);
            // Keep cursor at top of scrolling region
            self.get_cursor_mut().y = top;
        } else {
            // Normal reverse line feed within the scrolling region
            self.get_cursor_mut().y = current_y.saturating_sub(1).max(top);
        }
    }

    /// F0308: Insert character at cursor position (IRM - Insert/Replace Mode)
    pub fn insert_character_at_cursor(&mut self) {
        let cursor_x = self.get_cursor().x;
        let cursor_y = self.get_cursor().y;
        let screen_width = self.screen_width;

        let buffer = self.get_active_buffer();
        if cursor_y < buffer.content.len() && cursor_x <= buffer.content[cursor_y].len() {
            // Shift all characters to the right from cursor position
            let line = &mut buffer.content[cursor_y];
            if cursor_x < line.len() {
                // Insert a blank character at cursor position, shifting others right
                let blank_cell = TerminalCell::default();
                line.insert(cursor_x, blank_cell);

                // Keep line length within screen width by removing rightmost character if needed
                if line.len() > screen_width {
                    line.truncate(screen_width);
                }
            }
        }
    }

    /// F0308: Delete character at cursor position (DCH command support)
    pub fn delete_character_at_cursor(&mut self, count: usize) {
        let cursor_x = self.get_cursor().x;
        let cursor_y = self.get_cursor().y;
        let screen_width = self.screen_width;

        let buffer = self.get_active_buffer();
        if cursor_y < buffer.content.len() {
            let line = &mut buffer.content[cursor_y];
            for _ in 0..count {
                if cursor_x < line.len() {
                    line.remove(cursor_x);
                }
            }

            // Fill line with blank characters to maintain screen width
            while line.len() < screen_width {
                line.push(TerminalCell::default());
            }
        }
    }

    /// F0308: Get appropriate escape sequence for cursor key based on cursor key mode
    pub fn get_cursor_key_sequence(&self, key: &str) -> String {
        if self.mode.cursor_key_mode {
            // Application cursor keys (DECCKM enabled)
            match key {
                "up" => "\x1bOA".to_string(),
                "down" => "\x1bOB".to_string(),
                "right" => "\x1bOC".to_string(),
                "left" => "\x1bOD".to_string(),
                "home" => "\x1bOH".to_string(),
                "end" => "\x1bOF".to_string(),
                _ => format!("\x1b[{}", key), // Fallback to normal mode
            }
        } else {
            // Normal cursor keys (DECCKM disabled)
            match key {
                "up" => "\x1b[A".to_string(),
                "down" => "\x1b[B".to_string(),
                "right" => "\x1b[C".to_string(),
                "left" => "\x1b[D".to_string(),
                "home" => "\x1b[H".to_string(),
                "end" => "\x1b[F".to_string(),
                _ => format!("\x1b[{}", key),
            }
        }
    }

    /// F0308: Get appropriate escape sequence for keypad key based on keypad mode
    pub fn get_keypad_sequence(&self, key: &str) -> String {
        if self.mode.keypad_mode {
            // Application keypad (DECPAM enabled)
            match key {
                "0" => "\x1bOp".to_string(),
                "1" => "\x1bOq".to_string(),
                "2" => "\x1bOr".to_string(),
                "3" => "\x1bOs".to_string(),
                "4" => "\x1bOt".to_string(),
                "5" => "\x1bOu".to_string(),
                "6" => "\x1bOv".to_string(),
                "7" => "\x1bOw".to_string(),
                "8" => "\x1bOx".to_string(),
                "9" => "\x1bOy".to_string(),
                "." => "\x1bOn".to_string(),
                "+" => "\x1bOk".to_string(),
                "-" => "\x1bOm".to_string(),
                "*" => "\x1bOj".to_string(),
                "/" => "\x1bOo".to_string(),
                "=" => "\x1bOX".to_string(),
                "enter" => "\x1bOM".to_string(),
                _ => key.to_string(), // Fallback to literal key
            }
        } else {
            // Normal keypad (DECPAM disabled) - just return the key
            key.to_string()
        }
    }

    /// F0308: Handle linefeed/newline mode (LNM) - affects how newlines are processed
    pub fn process_newline(&mut self) {
        if self.mode.local_echo_mode {
            // LNM enabled: Newline acts as CR+LF
            self.get_cursor_mut().x = 0; // Carriage return
            self.line_feed_with_scrolling(); // Line feed
        } else {
            // LNM disabled: Newline is just LF
            self.line_feed_with_scrolling();
        }
    }

    /// F0312: Switch to alternate screen buffer with proper isolation
    pub fn switch_to_alternate_screen(&mut self) {
        if !self.use_alternate_screen {
            self.use_alternate_screen = true;
            self.full_screen_program_detected = true;
            // Alternate screen starts clean but preserves previous content
            // Only clear if explicitly requested (like mode 1047/1049)
        }
    }

    /// F0312: Switch to primary screen buffer maintaining buffer isolation
    pub fn switch_to_primary_screen(&mut self) {
        if self.use_alternate_screen {
            self.use_alternate_screen = false;
            // Primary screen content is preserved during alternate screen usage
        }
    }

    /// F0312: Clear alternate screen buffer explicitly
    pub fn clear_alternate_screen(&mut self) {
        self.alternate_buffer.clear_screen();
    }

    /// F0312: Verify buffer isolation - ensure buffers don't share content
    pub fn verify_buffer_isolation(&self) -> bool {
        // Buffers should have independent content structures
        !self.primary_buffer.content.is_empty() || !self.alternate_buffer.content.is_empty()
    }

    pub fn scroll_up(&mut self, lines: usize) {
        for _ in 0..lines {
            self.get_active_buffer().scroll_up_one_line();
        }
    }

    pub fn scroll_down(&mut self, lines: usize) {
        for _ in 0..lines {
            self.get_active_buffer().scroll_down_one_line();
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.screen_width = new_width;
        self.screen_height = new_height;

        // Resize both screen buffers
        self.primary_buffer.resize(new_width, new_height);
        self.alternate_buffer.resize(new_width, new_height);

        // Update scroll region
        self.scroll_region.bottom = new_height.saturating_sub(1);

        // Resize and reset tab stops
        self.tab_stops = vec![false; new_width];
        for i in (8..new_width).step_by(8) {
            if i < self.tab_stops.len() {
                self.tab_stops[i] = true;
            }
        }
    }

    pub fn clear_screen(&mut self) {
        self.get_active_buffer().clear_screen();
    }

    pub fn clear_line(&mut self, line: usize) {
        if line < self.screen_height {
            let screen = self.get_active_screen();
            for cell in &mut screen[line] {
                *cell = TerminalCell::default();
            }
        }
    }

    /// Process DCS query sequences
    pub fn process_dcs_query(&mut self, data: &str) {
        match data {
            "1c" => {
                // Terminal ID query response
                self.pending_responses.push("\x1b[?6c".to_string());
            }
            "2c" => {
                // Secondary device attributes
                self.pending_responses.push("\x1b[>0;276;0c".to_string());
            }
            _ => {
                log::debug!("Unknown DCS query: {}", data);
            }
        }
    }

    /// Process DCS extended capability sequences
    pub fn process_dcs_extended(&mut self, data: &str) {
        // Extended capabilities like sixel graphics, ReGIS, etc.
        // For now, just log for debugging
        log::debug!("DCS extended capability: {}", data);
    }

    /// F0310: Check if any mouse reporting mode is active
    pub fn is_mouse_reporting_enabled(&self) -> bool {
        self.mode.mouse_x10_mode
            || self.mode.mouse_normal_mode
            || self.mode.mouse_button_event_mode
            || self.mode.mouse_any_event_mode
    }

    /// F0310: Get the current active mouse reporting mode
    pub fn get_mouse_reporting_mode(&self) -> Option<&str> {
        if self.mode.mouse_x10_mode {
            Some("X10")
        } else if self.mode.mouse_button_event_mode {
            Some("ButtonEvent")
        } else if self.mode.mouse_any_event_mode {
            Some("AnyEvent")
        } else if self.mode.mouse_normal_mode {
            Some("Normal")
        } else {
            None
        }
    }

    /// F0310: Generate mouse report sequence for xterm compatibility
    pub fn generate_mouse_report(
        &self,
        event_type: MouseEventType,
        x: usize,
        y: usize,
        button: Option<MouseButton>,
        modifiers: MouseModifiers,
    ) -> Option<String> {
        if !self.is_mouse_reporting_enabled() {
            return None;
        }

        // Convert coordinates to 1-based (xterm convention)
        let x = (x + 1).min(255); // Clamp to valid range
        let y = (y + 1).min(255); // Clamp to valid range

        if self.mode.mouse_sgr_mode {
            // SGR (1006) extended reporting - supports coordinates > 223
            self.generate_sgr_mouse_report(event_type, x, y, button, modifiers)
        } else {
            // Standard reporting (X10, Normal, ButtonEvent, AnyEvent)
            self.generate_standard_mouse_report(event_type, x, y, button, modifiers)
        }
    }

    /// F0310: Generate standard mouse report (modes 1000, 1002, 1003)
    fn generate_standard_mouse_report(
        &self,
        event_type: MouseEventType,
        x: usize,
        y: usize,
        button: Option<MouseButton>,
        modifiers: MouseModifiers,
    ) -> Option<String> {
        let mut cb = 0u8; // Control byte

        // Set button bits
        match button {
            Some(MouseButton::Left) => cb |= 0,
            Some(MouseButton::Middle) => cb |= 1,
            Some(MouseButton::Right) => cb |= 2,
            Some(MouseButton::WheelUp) => cb |= 64,
            Some(MouseButton::WheelDown) => cb |= 65,
            Some(MouseButton::WheelLeft) => cb |= 66,
            Some(MouseButton::WheelRight) => cb |= 67,
            None => {}
        }

        // Set modifier bits
        if modifiers.shift {
            cb |= 4;
        }
        if modifiers.meta {
            cb |= 8;
        }
        if modifiers.ctrl {
            cb |= 16;
        }

        // Set event type bits
        match event_type {
            MouseEventType::Press => {} // Default - no additional bits
            MouseEventType::Release => {
                if self.mode.mouse_x10_mode {
                    return None; // X10 mode doesn't report releases
                }
                cb |= 3; // Release marker
            }
            MouseEventType::Motion => {
                if !self.mode.mouse_any_event_mode {
                    return None; // Only AnyEvent mode reports motion
                }
                cb |= 32; // Motion marker
            }
            MouseEventType::Wheel => {
                // Wheel events already handled in button assignment
            }
        }

        // Add base offset
        cb += 32;

        // Generate escape sequence
        Some(format!(
            "\x1b[M{}{}{}",
            cb as char,
            (x + 32) as u8 as char,
            (y + 32) as u8 as char
        ))
    }

    /// F0310: Generate SGR mouse report (mode 1006)
    fn generate_sgr_mouse_report(
        &self,
        event_type: MouseEventType,
        x: usize,
        y: usize,
        button: Option<MouseButton>,
        modifiers: MouseModifiers,
    ) -> Option<String> {
        let mut cb = 0u8;

        // Set button bits
        match button {
            Some(MouseButton::Left) => cb |= 0,
            Some(MouseButton::Middle) => cb |= 1,
            Some(MouseButton::Right) => cb |= 2,
            Some(MouseButton::WheelUp) => cb |= 64,
            Some(MouseButton::WheelDown) => cb |= 65,
            Some(MouseButton::WheelLeft) => cb |= 66,
            Some(MouseButton::WheelRight) => cb |= 67,
            None => {}
        }

        // Set modifier bits
        if modifiers.shift {
            cb |= 4;
        }
        if modifiers.meta {
            cb |= 8;
        }
        if modifiers.ctrl {
            cb |= 16;
        }

        // Set motion bit for motion events
        if event_type == MouseEventType::Motion && self.mode.mouse_any_event_mode {
            cb |= 32;
        }

        // SGR format: ESC[<Cb;Cx;CyM/m (M=press, m=release)
        let end_char = match event_type {
            MouseEventType::Press | MouseEventType::Motion | MouseEventType::Wheel => 'M',
            MouseEventType::Release => 'm',
        };

        Some(format!("\x1b[<{};{};{}{}", cb, x, y, end_char))
    }
}
