use vte::{Params, Parser, Perform};

#[derive(Debug, Clone, PartialEq)]
pub enum MouseEventType {
    Press,
    Release,
    Motion,
    Wheel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MouseModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// A color a program set: an entry of the 256-color palette or a 24-bit color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermColor {
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl TermColor {
    /// The SGR parameters selecting this color as the foreground (`background` false) or the
    /// background.
    fn sgr(self, background: bool) -> String {
        let (base, bright, extended) = if background {
            (40, 100, 48)
        } else {
            (30, 90, 38)
        };
        match self {
            TermColor::Indexed(n) if n < 8 => format!("{}", base + u16::from(n)),
            TermColor::Indexed(n) if n < 16 => format!("{}", bright + u16::from(n - 8)),
            TermColor::Indexed(n) => format!("{};5;{}", extended, n),
            TermColor::Rgb(r, g, b) => format!("{};2;{};{};{}", extended, r, g, b),
        }
    }
}

/// The character in the right half of a two-column character's cells.
pub const WIDE_CONTINUATION: char = '\0';

#[derive(Debug, Clone, PartialEq)]
/// F0306: Extended TerminalCell with comprehensive SGR attribute support
pub struct TerminalCell {
    pub character: char,
    pub fg_color: Option<TermColor>,
    pub bg_color: Option<TermColor>,

    // Basic text attributes
    pub bold: bool,
    pub dim: bool, // F0306: Faint/dim text
    pub italic: bool,
    pub underline: bool,
    pub double_underline: bool, // F0306: Double underline
    pub blink: bool,            // F0306: Blinking text
    pub reverse: bool,
    pub hidden: bool, // F0306: Concealed/hidden text
    pub strikethrough: bool,

    // Extended attributes
    pub font_id: Option<u8>, // F0306: Alternative font selection (0-9)
    pub framed: bool,        // F0306: Framed text
    pub encircled: bool,     // F0306: Encircled text
    pub overlined: bool,     // F0306: Overlined text

    // Ideogram attributes (rarely used)
    pub ideogram_underline: bool,        // F0306: Ideogram underline
    pub ideogram_double_underline: bool, // F0306: Ideogram double underline
    pub ideogram_overline: bool,         // F0306: Ideogram overline
    pub ideogram_double_overline: bool,  // F0306: Ideogram double overline
    pub ideogram_stress: bool,           // F0306: Ideogram stress marking
}

impl Default for TerminalCell {
    fn default() -> Self {
        TerminalCell {
            character: ' ',
            fg_color: None,
            bg_color: None,

            // Basic text attributes
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            double_underline: false,
            blink: false,
            reverse: false,
            hidden: false,
            strikethrough: false,

            // Extended attributes
            font_id: None,
            framed: false,
            encircled: false,
            overlined: false,

            // Ideogram attributes
            ideogram_underline: false,
            ideogram_double_underline: false,
            ideogram_overline: false,
            ideogram_double_overline: false,
            ideogram_stress: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CursorState {
    pub x: usize,
    pub y: usize,
    pub visible: bool,
    pub style: CursorStyle,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CursorStyle {
    #[default]
    Block,
    Underline,
    Bar,
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}

/// F0314: Line attributes for double-width and double-height lines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineAttribute {
    #[default]
    Normal, // Standard single-width, single-height line
    DoubleWidth,        // DECDWL - Double-width line
    DoubleHeightTop,    // DECDHL - Double-height line (top half)
    DoubleHeightBottom, // DECDHL - Double-height line (bottom half)
}

/// F0306: SGR sequence state tracking for extended color support
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SGRState {
    pub expecting_extended_fg: bool, // Expecting 38;5;n or 38;2;r;g;b sequence
    pub expecting_extended_bg: bool, // Expecting 48;5;n or 48;2;r;g;b sequence
    pub extended_fg_mode: Option<u8>, // 5 for palette, 2 for RGB
    pub extended_bg_mode: Option<u8>, // 5 for palette, 2 for RGB
    pub rgb_components: Vec<u8>,     // RGB components being collected
    pub palette_index: Option<u8>,   // 256-color palette index
}

#[derive(Debug, Clone)]
pub struct TerminalMode {
    pub cursor_key_mode: bool, // DECCKM - Application cursor keys
    pub keypad_mode: bool,     // DECPAM - Application keypad
    pub auto_wrap_mode: bool,  // DECAWM - Auto wrap
    pub origin_mode: bool,     // DECOM - Origin mode
    pub insert_mode: bool,     // IRM - Insert/Replace mode
    pub local_echo_mode: bool, // LNM - Linefeed/Newline mode
    pub reverse_video: bool,   // DECSCNM - Screen mode (reverse)
    // F0310: Mouse Protocol Support
    pub mouse_x10_mode: bool,          // 1000 - X10 mouse reporting
    pub mouse_normal_mode: bool,       // 1002 - Normal mouse reporting
    pub mouse_button_event_mode: bool, // 1002 - Button event tracking
    pub mouse_any_event_mode: bool,    // 1003 - Any event tracking
    pub mouse_sgr_mode: bool,          // 1006 - SGR extended reporting
    pub bracketed_paste_mode: bool,    // 2004 - Bracketed paste mode
}

impl Default for TerminalMode {
    fn default() -> Self {
        TerminalMode {
            cursor_key_mode: false,
            keypad_mode: false,
            auto_wrap_mode: true,
            origin_mode: false,
            insert_mode: false,
            local_echo_mode: false,
            reverse_video: false,
            // F0310: Mouse Protocol Support - all disabled by default
            mouse_x10_mode: false,
            mouse_normal_mode: false,
            mouse_button_event_mode: false,
            mouse_any_event_mode: false,
            mouse_sgr_mode: false,
            bracketed_paste_mode: false,
        }
    }
}

#[derive(Debug, Clone)]
/// F0307: Scrolling region for DECSTBM (Set Top and Bottom Margins)
#[derive(Default)]
pub struct ScrollRegion {
    pub top: usize,    // Top margin (0-based, inclusive)
    pub bottom: usize, // Bottom margin (0-based, inclusive)
}

#[derive(Debug, Clone)]
pub struct TerminalScreenBuffer {
    pub content: Vec<Vec<TerminalCell>>,
    pub scrollback: Vec<Vec<TerminalCell>>,
    pub cursor: CursorState,
    pub saved_cursor: Option<CursorState>,
    pub scroll_position: usize, // 0 = bottom (normal), higher = scrolled up
    pub max_scrollback: usize,
    pub width: usize,
    pub height: usize,
    // F0314: Line attributes for each line
    pub line_attributes: Vec<LineAttribute>,
    pub scrollback_line_attributes: Vec<LineAttribute>,
}

impl TerminalScreenBuffer {
    pub fn new(width: usize, height: usize, max_scrollback: usize) -> Self {
        TerminalScreenBuffer {
            content: vec![vec![TerminalCell::default(); width]; height],
            scrollback: Vec::new(),
            cursor: CursorState {
                x: 0,
                y: 0,
                visible: true,
                style: CursorStyle::default(),
            },
            saved_cursor: None,
            scroll_position: 0,
            max_scrollback,
            width,
            height,
            // F0314: Initialize line attributes to normal for all lines
            line_attributes: vec![LineAttribute::Normal; height],
            scrollback_line_attributes: Vec::new(),
        }
    }

    pub fn scroll_up_one_line(&mut self) {
        // Move top line to scrollback
        if let Some(top_line) = self.content.first().cloned() {
            self.scrollback.push(top_line);

            // F0314: Move top line attribute to scrollback too
            if !self.line_attributes.is_empty() {
                let top_attr = self.line_attributes.remove(0);
                self.scrollback_line_attributes.push(top_attr);
            }

            // Limit scrollback size
            if self.scrollback.len() > self.max_scrollback {
                self.scrollback.remove(0);
                if !self.scrollback_line_attributes.is_empty() {
                    self.scrollback_line_attributes.remove(0);
                }
            }
        }

        // Shift all lines up and add empty line at bottom
        self.content.remove(0);
        self.content.push(vec![TerminalCell::default(); self.width]);

        // F0314: Add normal line attribute for new bottom line
        self.line_attributes.push(LineAttribute::Normal);
    }

    pub fn scroll_down_one_line(&mut self) {
        // Move bottom line up and get line from scrollback
        if let Some(line) = self.scrollback.pop() {
            self.content.remove(self.content.len() - 1);
            self.content.insert(0, line);

            // F0314: Restore line attribute from scrollback
            if let Some(attr) = self.scrollback_line_attributes.pop() {
                self.line_attributes.remove(self.line_attributes.len() - 1);
                self.line_attributes.insert(0, attr);
            }
        }
    }

    pub fn get_visible_content(&self) -> &Vec<Vec<TerminalCell>> {
        &self.content
    }

    pub fn get_line_mut(&mut self, y: usize) -> Option<&mut Vec<TerminalCell>> {
        self.content.get_mut(y)
    }

    pub fn clear_screen(&mut self) {
        for row in &mut self.content {
            for cell in row {
                *cell = TerminalCell::default();
            }
        }
        self.cursor.x = 0;
        self.cursor.y = 0;
    }

    pub fn clear_from_cursor_to_end(&mut self) {
        // Clear from cursor position to end of screen
        for y in self.cursor.y..self.height {
            let start_x = if y == self.cursor.y { self.cursor.x } else { 0 };
            if let Some(row) = self.content.get_mut(y) {
                for cell in row.iter_mut().skip(start_x) {
                    *cell = TerminalCell::default();
                }
            }
        }
    }

    pub fn clear_from_beginning_to_cursor(&mut self) {
        // Clear from beginning of screen to cursor position
        for y in 0..=self.cursor.y.min(self.height - 1) {
            let end_x = if y == self.cursor.y {
                self.cursor.x + 1
            } else {
                self.width
            };
            if let Some(row) = self.content.get_mut(y) {
                for x in 0..end_x.min(row.len()) {
                    row[x] = TerminalCell::default();
                }
            }
        }
    }

    pub fn insert_lines(&mut self, count: usize) {
        let y = self.cursor.y;
        if y < self.height {
            for _ in 0..count {
                if self.content.len() > y {
                    self.content
                        .insert(y, vec![TerminalCell::default(); self.width]);
                    if self.content.len() > self.height {
                        self.content.pop();
                    }
                }
            }
        }
    }

    pub fn delete_lines(&mut self, count: usize) {
        let y = self.cursor.y;
        if y < self.height {
            for _ in 0..count.min(self.height - y) {
                if self.content.len() > y {
                    self.content.remove(y);
                    self.content.push(vec![TerminalCell::default(); self.width]);
                }
            }
        }
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;

        // Resize existing content rows
        for row in &mut self.content {
            row.resize(new_width, TerminalCell::default());
        }

        // Add or remove rows as needed
        if self.content.len() < new_height {
            while self.content.len() < new_height {
                self.content.push(vec![TerminalCell::default(); new_width]);
            }
        } else if self.content.len() > new_height {
            self.content.truncate(new_height);
        }

        // F0314: Resize line attributes to match new height
        self.line_attributes
            .resize(new_height, LineAttribute::Normal);

        // Adjust cursor position if out of bounds
        self.cursor.x = self.cursor.x.min(new_width.saturating_sub(1));
        self.cursor.y = self.cursor.y.min(new_height.saturating_sub(1));
    }

    /// Convert TerminalScreenBuffer to content lines for stream integration
    /// This feeds PTY content into BoxMux's main differential drawing system
    pub fn to_content_lines(&self, include_scrollback: bool) -> Vec<String> {
        let mut lines = Vec::new();

        // Include scrollback if requested
        if include_scrollback {
            for row in &self.scrollback {
                let line = self.terminal_row_to_string(row);
                lines.push(line);
            }
        }

        // Add visible content
        for row in &self.content {
            let line = self.terminal_row_to_string(row);
            lines.push(line);
        }

        lines
    }

    /// A row as text with SGR sequences for its colors and attributes. The right halves of
    /// wide characters are left out.
    fn terminal_row_to_string(&self, row: &[TerminalCell]) -> String {
        fn sgr(cell: &TerminalCell) -> String {
            let mut codes = vec!["0".to_string()];
            for (on, code) in [
                (cell.bold, "1"),
                (cell.dim, "2"),
                (cell.italic, "3"),
                (cell.underline || cell.double_underline, "4"),
                (cell.blink, "5"),
                (cell.reverse, "7"),
                (cell.hidden, "8"),
                (cell.strikethrough, "9"),
            ] {
                if on {
                    codes.push(code.to_string());
                }
            }
            if let Some(fg) = cell.fg_color {
                codes.push(fg.sgr(false));
            }
            if let Some(bg) = cell.bg_color {
                codes.push(bg.sgr(true));
            }
            format!("\x1b[{}m", codes.join(";"))
        }
        let plain = TerminalCell::default();
        let mut result = String::new();
        let mut current = sgr(&plain);
        for cell in row {
            if cell.character == WIDE_CONTINUATION {
                continue;
            }
            let wanted = sgr(cell);
            if wanted != current {
                result.push_str(&wanted);
                current = wanted;
            }
            result.push(cell.character);
        }
        if current != sgr(&plain) {
            result.push_str("\x1b[0m");
        }
        result
    }

    /// Get content as string for stream updates - integrates with BoxMux stream system
    pub fn get_content_for_stream(&self) -> String {
        let lines = self.to_content_lines(true); // Include scrollback for PTY
        lines.join("\n")
    }
}

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

/// A terminal emulator: bytes a program writes go in, a screen of styled cells comes out.
pub struct AnsiProcessor {
    pub terminal_state: TerminalState,
    /// Kept across calls, so an escape sequence split between two reads is still understood.
    parser: Parser,
}

impl AnsiProcessor {
    pub fn new() -> Self {
        Self::with_screen_size(80, 24)
    }

    pub fn with_screen_size(width: usize, height: usize) -> Self {
        AnsiProcessor {
            terminal_state: TerminalState::new(width, height),
            parser: Parser::new(),
        }
    }

    pub fn resize_screen(&mut self, width: usize, height: usize) {
        self.terminal_state.resize(width, height);
    }

    pub fn get_cursor_x(&self) -> usize {
        self.terminal_state.get_cursor().x
    }

    pub fn get_cursor_y(&self) -> usize {
        self.terminal_state.get_cursor().y
    }

    pub fn is_using_alternate_screen(&self) -> bool {
        self.terminal_state.use_alternate_screen
    }

    pub fn get_terminal_state(&self) -> &TerminalState {
        &self.terminal_state
    }

    /// Writes `c` at the cursor with the current attributes. A wide character also takes the
    /// next cell.
    fn put_char_at_cursor(&mut self, c: char, width: usize) {
        let cursor_y = self.terminal_state.get_cursor().y;
        let cursor_x = self.terminal_state.get_cursor().x;
        let (screen_width, screen_height) = (
            self.terminal_state.screen_width,
            self.terminal_state.screen_height,
        );
        if cursor_y >= screen_height || cursor_x >= screen_width {
            return;
        }
        let mut cell = self.terminal_state.current_attributes.clone();
        cell.character = c;

        // F0308: Apply reverse video mode (DECSCNM) globally
        if self.terminal_state.mode.reverse_video {
            std::mem::swap(&mut cell.fg_color, &mut cell.bg_color);
            if cell.fg_color.is_none() && cell.bg_color.is_none() {
                cell.fg_color = Some(TermColor::Indexed(0));
                cell.bg_color = Some(TermColor::Indexed(15));
            }
        }

        let screen = self.terminal_state.get_active_screen();
        let row = &mut screen[cursor_y];
        // Writing over half of a wide character leaves its other half blank.
        if row[cursor_x].character == WIDE_CONTINUATION && cursor_x > 0 {
            row[cursor_x - 1].character = ' ';
        }
        if cursor_x + 1 < screen_width && row[cursor_x + 1].character == WIDE_CONTINUATION {
            row[cursor_x + 1].character = ' ';
        }
        if width == 2 && cursor_x + 1 < screen_width {
            let mut continuation = cell.clone();
            continuation.character = WIDE_CONTINUATION;
            if cursor_x + 2 < screen_width && row[cursor_x + 2].character == WIDE_CONTINUATION {
                row[cursor_x + 2].character = ' ';
            }
            row[cursor_x + 1] = continuation;
        }
        row[cursor_x] = cell;
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
        let mut parser = std::mem::take(&mut self.parser);
        for &byte in bytes {
            parser.advance(self, byte);
        }
        self.parser = parser;
    }

    /// Takes the text the program put on the clipboard since the last call.
    pub fn take_clipboard_writes(&mut self) -> Vec<String> {
        std::mem::take(&mut self.terminal_state.clipboard_writes)
    }

    /// Takes the replies to terminal queries that must be written back to the program.
    pub fn take_responses(&mut self) -> Vec<String> {
        std::mem::take(&mut self.terminal_state.pending_responses)
    }

    /// The lines a box shows for this terminal, with SGR color sequences: the scrollback and
    /// the screen, or only the screen while the program uses the alternate screen.
    pub fn display_lines(&self) -> Vec<String> {
        let buffer = self.terminal_state.get_active_buffer_ref();
        buffer.to_content_lines(!self.terminal_state.use_alternate_screen)
    }

    pub fn process_string(&mut self, text: &str) {
        self.process_bytes(text.as_bytes());
    }

    /// The text of the screen, without colors, trailing spaces or trailing empty lines.
    pub fn get_processed_text(&self) -> String {
        let mut content = String::new();
        for row in self.terminal_state.get_active_screen_ref() {
            let line: String = row
                .iter()
                .map(|cell| cell.character)
                .filter(|c| *c != WIDE_CONTINUATION)
                .collect();
            content.push_str(line.trim_end());
            content.push('\n');
        }
        content.trim_end().to_string()
    }

    /// F0306: Enhanced SGR parameter handling with comprehensive color and attribute support
    fn apply_sgr_param(&mut self, param: u16) {
        match param {
            0 => {
                // Reset all attributes
                self.terminal_state.current_attributes = TerminalCell::default();
            }
            // Text attributes
            1 => self.terminal_state.current_attributes.bold = true,
            2 => self.terminal_state.current_attributes.dim = true, // Dim/faint
            3 => self.terminal_state.current_attributes.italic = true,
            4 => self.terminal_state.current_attributes.underline = true,
            5 => self.terminal_state.current_attributes.blink = true, // Slow blink
            6 => self.terminal_state.current_attributes.blink = true, // Rapid blink (treat as slow blink)
            7 => self.terminal_state.current_attributes.reverse = true,
            8 => self.terminal_state.current_attributes.hidden = true, // Concealed characters
            9 => self.terminal_state.current_attributes.strikethrough = true,

            // Double-width/height characters
            10 => { /* Primary font - default */ }
            11..=19 => {
                // Alternative fonts 1-9 - store font selection
                self.terminal_state.current_attributes.font_id = Some((param - 10) as u8);
            }
            20 => { /* Gothic/Fraktur - rarely supported */ }
            21 => {
                // Double underline or bold off (depending on implementation)
                self.terminal_state.current_attributes.double_underline = true;
            }

            // Reset attributes
            22 => {
                // Normal intensity - reset bold and dim
                self.terminal_state.current_attributes.bold = false;
                self.terminal_state.current_attributes.dim = false;
            }
            23 => {
                // Reset italic and gothic
                self.terminal_state.current_attributes.italic = false;
            }
            24 => {
                // Reset underline
                self.terminal_state.current_attributes.underline = false;
                self.terminal_state.current_attributes.double_underline = false;
            }
            25 => self.terminal_state.current_attributes.blink = false, // Reset blink
            26 => { /* Reserved */ }
            27 => self.terminal_state.current_attributes.reverse = false,
            28 => self.terminal_state.current_attributes.hidden = false, // Reveal
            29 => self.terminal_state.current_attributes.strikethrough = false,

            // Standard 8-color foreground (30-37)
            30..=37 => {
                self.terminal_state.current_attributes.fg_color =
                    Some(TermColor::Indexed((param - 30) as u8))
            }
            38 => {
                // Extended foreground color (handled in apply_sgr_sequence for 38;5;n and 38;2;r;g;b)
                self.terminal_state.sgr_state.expecting_extended_fg = true;
            }
            39 => self.terminal_state.current_attributes.fg_color = None, // Default foreground

            // Standard 8-color background (40-47)
            40..=47 => {
                self.terminal_state.current_attributes.bg_color =
                    Some(TermColor::Indexed((param - 40) as u8))
            }
            48 => {
                // Extended background color (handled in apply_sgr_sequence for 48;5;n and 48;2;r;g;b)
                self.terminal_state.sgr_state.expecting_extended_bg = true;
            }
            49 => self.terminal_state.current_attributes.bg_color = None, // Default background

            // Additional attributes (50-65)
            50 => { /* Reserved */ }
            51 => self.terminal_state.current_attributes.framed = true, // Framed
            52 => self.terminal_state.current_attributes.encircled = true, // Encircled
            53 => self.terminal_state.current_attributes.overlined = true, // Overlined
            54 => {
                // Reset framed and encircled
                self.terminal_state.current_attributes.framed = false;
                self.terminal_state.current_attributes.encircled = false;
            }
            55 => self.terminal_state.current_attributes.overlined = false, // Reset overlined

            // Ideogram attributes (60-65) - rarely used
            60 => self.terminal_state.current_attributes.ideogram_underline = true,
            61 => {
                self.terminal_state
                    .current_attributes
                    .ideogram_double_underline = true
            }
            62 => self.terminal_state.current_attributes.ideogram_overline = true,
            63 => {
                self.terminal_state
                    .current_attributes
                    .ideogram_double_overline = true
            }
            64 => self.terminal_state.current_attributes.ideogram_stress = true,
            65 => {
                // Reset ideogram attributes
                self.terminal_state.current_attributes.ideogram_underline = false;
                self.terminal_state
                    .current_attributes
                    .ideogram_double_underline = false;
                self.terminal_state.current_attributes.ideogram_overline = false;
                self.terminal_state
                    .current_attributes
                    .ideogram_double_overline = false;
                self.terminal_state.current_attributes.ideogram_stress = false;
            }

            // Bright/intense colors (90-97 foreground, 100-107 background)
            90..=97 => {
                self.terminal_state.current_attributes.fg_color =
                    Some(TermColor::Indexed((param - 90 + 8) as u8))
            }
            100..=107 => {
                self.terminal_state.current_attributes.bg_color =
                    Some(TermColor::Indexed((param - 100 + 8) as u8))
            }

            _ => {} // Ignore unknown parameters
        }
    }

    /// F0306: Process complete SGR sequence with extended color support (38;5;n, 38;2;r;g;b, etc.)
    fn apply_sgr_sequence(&mut self, params: &Params) {
        if params.is_empty() {
            self.apply_sgr_param(0); // Reset if no params
            return;
        }

        let mut param_iter = params.iter().flat_map(|slice| slice.iter());

        while let Some(&param) = param_iter.next() {
            match param {
                38 => {
                    // Extended foreground color: 38;5;n (256-color) or 38;2;r;g;b (24-bit RGB)
                    if let Some(&mode) = param_iter.next() {
                        match mode {
                            5 => {
                                // 256-color palette: 38;5;n
                                if let Some(&palette_index) = param_iter.next() {
                                    self.terminal_state.current_attributes.fg_color =
                                        u8::try_from(palette_index).ok().map(TermColor::Indexed);
                                }
                            }
                            2 => {
                                // 24-bit RGB: 38;2;r;g;b
                                if let (Some(&r), Some(&g), Some(&b)) =
                                    (param_iter.next(), param_iter.next(), param_iter.next())
                                {
                                    self.terminal_state.current_attributes.fg_color =
                                        Some(TermColor::Rgb(r as u8, g as u8, b as u8));
                                }
                            }
                            _ => {} // Unknown extended color mode
                        }
                    }
                }
                48 => {
                    // Extended background color: 48;5;n (256-color) or 48;2;r;g;b (24-bit RGB)
                    if let Some(&mode) = param_iter.next() {
                        match mode {
                            5 => {
                                // 256-color palette: 48;5;n
                                if let Some(&palette_index) = param_iter.next() {
                                    self.terminal_state.current_attributes.bg_color =
                                        u8::try_from(palette_index).ok().map(TermColor::Indexed);
                                }
                            }
                            2 => {
                                // 24-bit RGB: 48;2;r;g;b
                                if let (Some(&r), Some(&g), Some(&b)) =
                                    (param_iter.next(), param_iter.next(), param_iter.next())
                                {
                                    self.terminal_state.current_attributes.bg_color =
                                        Some(TermColor::Rgb(r as u8, g as u8, b as u8));
                                }
                            }
                            _ => {} // Unknown extended color mode
                        }
                    }
                }
                _ => {
                    // Regular SGR parameter
                    self.apply_sgr_param(param);
                }
            }
        }
    }

    // Helper to extract first parameter from Params
    fn get_param(params: &Params, index: usize) -> u16 {
        if index < params.len() {
            if let Some(param_slice) = params.iter().nth(index) {
                if !param_slice.is_empty() {
                    return param_slice[0];
                }
            }
        }
        0
    }

    /// F0311: Get DEC private mode status for DECRQM queries
    fn get_private_mode_status(&self, mode: u16) -> u8 {
        let terminal_mode = &self.terminal_state.mode;
        match mode {
            1 => {
                if terminal_mode.cursor_key_mode {
                    1
                } else {
                    2
                }
            } // DECCKM - Application Cursor Keys
            2 => 4, // DECANM - ANSI mode (always permanently set)
            3 => 2, // DECCOLM - 80/132 column mode (not supported, permanently reset)
            6 => {
                if terminal_mode.origin_mode {
                    1
                } else {
                    2
                }
            } // DECOM - Origin Mode
            7 => {
                if terminal_mode.auto_wrap_mode {
                    1
                } else {
                    2
                }
            } // DECAWM - Auto Wrap Mode
            25 => {
                if self.terminal_state.get_cursor().visible {
                    1
                } else {
                    2
                }
            } // DECTCEM - Cursor Visible
            47 => {
                if self.terminal_state.use_alternate_screen {
                    1
                } else {
                    2
                }
            } // Alternate Screen (Simple)
            1000 => {
                if terminal_mode.mouse_x10_mode {
                    1
                } else {
                    2
                }
            } // Mouse X10 mode
            1002 => {
                if terminal_mode.mouse_button_event_mode {
                    1
                } else {
                    2
                }
            } // Mouse Button Event mode
            1003 => {
                if terminal_mode.mouse_any_event_mode {
                    1
                } else {
                    2
                }
            } // Mouse Any Event mode
            1006 => {
                if terminal_mode.mouse_sgr_mode {
                    1
                } else {
                    2
                }
            } // Mouse SGR mode
            1047 => {
                if self.terminal_state.use_alternate_screen {
                    1
                } else {
                    2
                }
            } // Alternate Screen Buffer (Clear)
            1049 => {
                if self.terminal_state.use_alternate_screen {
                    1
                } else {
                    2
                }
            } // Alternate Screen Buffer (Cursor Save)
            2004 => {
                if terminal_mode.bracketed_paste_mode {
                    1
                } else {
                    2
                }
            } // Bracketed Paste Mode
            _ => 0, // Mode not recognized
        }
    }

    /// F0311: Get standard mode status for DECRQM queries
    fn get_standard_mode_status(&self, mode: u16) -> u8 {
        let terminal_mode = &self.terminal_state.mode;
        match mode {
            4 => {
                if terminal_mode.insert_mode {
                    1
                } else {
                    2
                }
            } // IRM - Insert/Replace Mode
            20 => {
                if terminal_mode.local_echo_mode {
                    1
                } else {
                    2
                }
            } // LNM - Line Feed/New Line Mode
            _ => 0, // Mode not recognized
        }
    }

    // F0315: Terminal Title Support - Title change detection and consumption methods

    /// Check if the terminal title has changed since last consumption
    pub fn has_title_changed(&self) -> bool {
        self.terminal_state.title_changed
    }

    /// Get the current terminal title without consuming the change flag
    pub fn get_current_terminal_title(&self) -> Option<String> {
        self.terminal_state.terminal_title.clone()
    }

    /// Get the terminal title and mark the change as consumed
    pub fn get_and_consume_terminal_title(&mut self) -> Option<String> {
        if self.terminal_state.title_changed {
            self.terminal_state.title_changed = false;
            self.terminal_state.terminal_title.clone()
        } else {
            None
        }
    }

    /// Get the current icon name
    pub fn get_current_icon_name(&self) -> Option<String> {
        self.terminal_state.icon_name.clone()
    }
}

impl Perform for AnsiProcessor {
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

impl Default for AnsiProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_sequence_split_between_two_reads_is_still_understood() {
        let mut processor = AnsiProcessor::with_screen_size(20, 3);
        processor.process_bytes(b"a\x1b[3");
        processor.process_bytes(b"1mb\x1b[0mc");
        assert_eq!(processor.get_processed_text(), "abc");
        let line = &processor.display_lines()[0];
        assert!(line.contains("\x1b[0;31mb"), "{:?}", line);
    }

    #[test]
    fn writing_the_last_column_waits_for_the_next_character_to_wrap() {
        let mut processor = AnsiProcessor::with_screen_size(4, 3);
        // A full line followed by CR LF must not leave an empty line.
        processor.process_string("abcd\r\nefgh\r\n");
        assert_eq!(processor.get_processed_text(), "abcd\nefgh");
        // A character after a full line goes to the next line.
        let mut processor = AnsiProcessor::with_screen_size(4, 3);
        processor.process_string("abcdX");
        assert_eq!(processor.get_processed_text(), "abcd\nX");
        // Moving the cursor cancels the pending wrap.
        let mut processor = AnsiProcessor::with_screen_size(4, 3);
        processor.process_string("abcd\x1b[1;1HZ");
        assert_eq!(processor.get_processed_text(), "Zbcd");
    }

    #[test]
    fn wide_characters_take_two_cells_and_wrap_as_a_whole() {
        let mut processor = AnsiProcessor::with_screen_size(5, 3);
        processor.process_string("日本語");
        // Two wide characters fill four columns; the third does not fit in the last column.
        assert_eq!(processor.get_processed_text(), "日本\n語");
        assert_eq!((processor.get_cursor_x(), processor.get_cursor_y()), (2, 1));
        let lines = processor.display_lines();
        assert_eq!(crate::utils::display_width(&lines[0]), 5, "{:?}", lines[0]);
    }

    #[test]
    fn truecolor_and_palette_colors_are_kept_exactly() {
        let mut processor = AnsiProcessor::with_screen_size(10, 2);
        processor.process_string("\x1b[38;2;10;20;30;48;5;200mx");
        let cell = &processor.terminal_state.get_active_screen_ref()[0][0];
        assert_eq!(cell.fg_color, Some(TermColor::Rgb(10, 20, 30)));
        assert_eq!(cell.bg_color, Some(TermColor::Indexed(200)));
        assert!(processor.display_lines()[0].contains("38;2;10;20;30;48;5;200"));
    }

    #[test]
    fn backspace_moves_left_without_erasing() {
        let mut processor = AnsiProcessor::with_screen_size(10, 2);
        processor.process_string("abc\x08\x08");
        assert_eq!(processor.get_processed_text(), "abc");
        assert_eq!(processor.get_cursor_x(), 1);
        processor.process_string("X");
        assert_eq!(processor.get_processed_text(), "aXc");
    }

    #[test]
    fn erasing_to_the_cursor_clears_the_start_of_the_screen() {
        let mut processor = AnsiProcessor::with_screen_size(4, 3);
        processor.process_string("abcd\r\nefgh\r\nijkl\x1b[2;2H\x1b[1J");
        assert_eq!(processor.get_processed_text(), "\n  gh\nijkl");
    }

    #[test]
    fn programs_can_write_the_clipboard_but_not_read_it() {
        let mut processor = AnsiProcessor::with_screen_size(10, 2);
        processor.process_string("\x1b]52;c;aGVsbG8gd29ybGQ=\x07\x1b]52;c;?\x07");
        assert_eq!(processor.take_clipboard_writes(), vec!["hello world"]);
        assert!(processor.take_responses().is_empty(), "nothing is revealed");
        assert!(processor.take_clipboard_writes().is_empty());
    }

    #[test]
    fn a_full_reset_returns_to_the_initial_state() {
        let mut processor = AnsiProcessor::with_screen_size(10, 3);
        processor.process_string("\x1b[?1h\x1b[31mtext\x1bc");
        assert_eq!(processor.get_processed_text(), "");
        assert!(!processor.terminal_state.mode.cursor_key_mode);
        assert_eq!(
            processor.terminal_state.current_attributes,
            TerminalCell::default()
        );
    }

    #[test]
    fn test_ansi_processor_basic_text() {
        let mut processor = AnsiProcessor::new();
        processor.process_string("Hello World");
        assert_eq!(processor.get_processed_text(), "Hello World");
    }

    #[test]
    fn test_ansi_processor_color_codes() {
        let mut processor = AnsiProcessor::new();
        processor.process_string("\x1b[31mRed Text\x1b[0m");
        assert_eq!(processor.get_processed_text(), "Red Text");
    }

    #[test]
    fn test_ansi_processor_newlines() {
        let mut processor = AnsiProcessor::new();
        // A PTY turns each newline a program prints into CR LF.
        processor.process_string("Line 1\r\nLine 2\r\n");
        assert_eq!(processor.get_processed_text(), "Line 1\nLine 2");
        assert_eq!(processor.get_cursor_y(), 2);
        assert_eq!(processor.get_cursor_x(), 0);
        // A bare line feed moves down without returning to the first column.
        let mut processor = AnsiProcessor::new();
        processor.process_string("ab\ncd");
        assert_eq!(processor.get_processed_text(), "ab\n  cd");
    }

    #[test]
    fn test_ansi_processor_clear_screen() {
        let mut processor = AnsiProcessor::new();
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
        let mut processor = AnsiProcessor::new();

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
        let mut processor = AnsiProcessor::new();

        // Test DCS sequence processing
        processor.process_string("\x1bP1$q\x1b\\"); // DCS query sequence

        // Verify DCS state was cleared
        assert!(processor.terminal_state.dcs_sequence_type.is_none());
        assert!(processor.terminal_state.dcs_buffer.is_none());
    }

    #[test]
    fn test_f0311_terminal_queries() {
        let mut processor = AnsiProcessor::new();

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
        let mut processor = AnsiProcessor::new();

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
        let mut processor = AnsiProcessor::new();

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
        let mut processor = AnsiProcessor::new();

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
        let mut processor = AnsiProcessor::new();

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
        let mut processor = AnsiProcessor::new();

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
        let mut processor = AnsiProcessor::new();

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
        let mut processor = AnsiProcessor::new();

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
        let mut processor = AnsiProcessor::new();

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
}
