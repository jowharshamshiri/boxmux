use vte::{Params, Parser, Perform};

pub struct AnsiProcessor {
    pub processed_text: String,
    pub cursor_x: usize,
    pub cursor_y: usize,
    pub current_fg_color: Option<u8>,
    pub current_bg_color: Option<u8>,
    pub bold: bool,
    pub underline: bool,
    pub italic: bool,
}

impl AnsiProcessor {
    pub fn new() -> Self {
        AnsiProcessor {
            processed_text: String::new(),
            cursor_x: 0,
            cursor_y: 0,
            current_fg_color: None,
            current_bg_color: None,
            bold: false,
            underline: false,
            italic: false,
        }
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
        let mut parser = Parser::new();
        for &byte in bytes {
            parser.advance(self, byte);
        }
    }

    pub fn process_string(&mut self, text: &str) {
        self.process_bytes(text.as_bytes());
    }

    pub fn get_processed_text(&self) -> &str {
        &self.processed_text
    }

    pub fn clear_processed_text(&mut self) {
        self.processed_text.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
    }

    fn apply_sgr_param(&mut self, param: u16) {
        match param {
            0 => {
                // Reset all
                self.current_fg_color = None;
                self.current_bg_color = None;
                self.bold = false;
                self.underline = false;
                self.italic = false;
            }
            1 => self.bold = true,
            3 => self.italic = true,
            4 => self.underline = true,
            22 => self.bold = false,
            23 => self.italic = false,
            24 => self.underline = false,
            30..=37 => self.current_fg_color = Some((param - 30) as u8),
            38 => {
                // 38;5;n for 256-color foreground - simplified handling
            }
            39 => self.current_fg_color = None,
            40..=47 => self.current_bg_color = Some((param - 40) as u8),
            48 => {
                // 48;5;n for 256-color background - simplified handling
            }
            49 => self.current_bg_color = None,
            90..=97 => self.current_fg_color = Some((param - 90 + 8) as u8), // Bright colors
            100..=107 => self.current_bg_color = Some((param - 100 + 8) as u8), // Bright bg colors
            _ => {} // Ignore unknown parameters
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
}

impl Perform for AnsiProcessor {
    fn print(&mut self, c: char) {
        self.processed_text.push(c);
        self.cursor_x += 1;
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.processed_text.push('\n');
                self.cursor_y += 1;
                self.cursor_x = 0;
            }
            b'\r' => {
                self.cursor_x = 0;
            }
            b'\t' => {
                let spaces = 8 - (self.cursor_x % 8);
                for _ in 0..spaces {
                    self.processed_text.push(' ');
                }
                self.cursor_x += spaces;
            }
            b'\x08' => {
                // Backspace
                if self.cursor_x > 0 {
                    self.processed_text.pop();
                    self.cursor_x -= 1;
                }
            }
            _ => {
                // Other control characters - for now just ignore
            }
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {
        // Device Control String - not implemented for basic ANSI processing
    }

    fn put(&mut self, _byte: u8) {
        // Put character in DCS - not implemented for basic ANSI processing
    }

    fn unhook(&mut self) {
        // End of DCS - not implemented for basic ANSI processing
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {
        // Operating System Command - could be used for terminal title, etc.
        // For basic implementation, we'll skip this
    }

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'm' => {
                // SGR - Set Graphics Rendition (colors, bold, etc.)
                if params.len() == 0 {
                    self.apply_sgr_param(0); // Reset if no params
                } else {
                    for param_slice in params.iter() {
                        for &param in param_slice {
                            self.apply_sgr_param(param);
                        }
                    }
                }
            }
            'H' | 'f' => {
                // Cursor position - CUP or HVP
                let row = if params.len() > 0 {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                let col = if params.len() > 1 {
                    Self::get_param(params, 1).max(1)
                } else {
                    1
                };
                self.cursor_y = (row.saturating_sub(1)) as usize;
                self.cursor_x = (col.saturating_sub(1)) as usize;
            }
            'A' => {
                // Cursor up
                let n = if params.len() > 0 {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                self.cursor_y = self.cursor_y.saturating_sub(n as usize);
            }
            'B' => {
                // Cursor down
                let n = if params.len() > 0 {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                self.cursor_y += n as usize;
            }
            'C' => {
                // Cursor forward
                let n = if params.len() > 0 {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                self.cursor_x += n as usize;
            }
            'D' => {
                // Cursor backward
                let n = if params.len() > 0 {
                    Self::get_param(params, 0).max(1)
                } else {
                    1
                };
                self.cursor_x = self.cursor_x.saturating_sub(n as usize);
            }
            'J' => {
                // Erase in Display
                let n = if params.len() > 0 {
                    Self::get_param(params, 0)
                } else {
                    0
                };
                match n {
                    0 => {} // Clear from cursor to end of display
                    1 => {} // Clear from start of display to cursor
                    2 => {
                        // Clear entire display
                        self.processed_text.clear();
                        self.cursor_x = 0;
                        self.cursor_y = 0;
                    }
                    _ => {}
                }
            }
            'K' => {
                // Erase in Line - for simplified implementation, we'll skip this
            }
            _ => {
                // Other CSI sequences - ignore for basic implementation
            }
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {
        // ESC sequences - for basic implementation, we'll skip most of these
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
        processor.process_string("Line 1\nLine 2\n");
        assert_eq!(processor.get_processed_text(), "Line 1\nLine 2\n");
        assert_eq!(processor.cursor_y, 2);
        assert_eq!(processor.cursor_x, 0);
    }

    #[test]
    fn test_ansi_processor_clear_screen() {
        let mut processor = AnsiProcessor::new();
        processor.process_string("Some text\x1b[2JCleared");
        assert_eq!(processor.get_processed_text(), "Cleared");
    }
}
