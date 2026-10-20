//! The terminal emulator behind PTY boxes: bytes a program writes go in, a screen of styled
//! cells comes out, along with the replies the program asked for (cursor position, device
//! attributes) and clipboard writes.

mod buffer;
mod cell;
mod perform;
mod state;
#[cfg(test)]
mod tests;

pub use buffer::*;
pub use cell::*;
pub use state::*;

use vte::{Params, Parser};

/// A terminal emulator: bytes a program writes go in, a screen of styled cells comes out.
pub struct Emulator {
    pub terminal_state: TerminalState,
    /// Kept across calls, so an escape sequence split between two reads is still understood.
    parser: Parser,
}

impl Emulator {
    pub fn new() -> Self {
        Self::with_screen_size(80, 24)
    }

    pub fn with_screen_size(width: usize, height: usize) -> Self {
        Emulator {
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

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
