use crate::model::common::Cell;
use crossterm::style::Color;

/// ANSI color processor for rendering text with embedded ANSI escape sequences
/// Converts ANSI escape sequences to BoxMux Cell structures with proper colors
#[derive(Debug, Clone)]
pub struct AnsiColorProcessor {
    current_fg: Color,
    current_bg: Color,
    current_bold: bool,
    current_italic: bool,
    current_underline: bool,
    current_reverse: bool,
}

impl Default for AnsiColorProcessor {
    fn default() -> Self {
        Self {
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            current_bold: false,
            current_italic: false,
            current_underline: false,
            current_reverse: false,
        }
    }
}

impl AnsiColorProcessor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Reset all attributes to default
    pub fn reset(&mut self) {
        self.current_fg = Color::Reset;
        self.current_bg = Color::Reset;
        self.current_bold = false;
        self.current_italic = false;
        self.current_underline = false;
        self.current_reverse = false;
    }

    /// Process text containing ANSI escape sequences and return vector of Cells
    pub fn process_text(&mut self, text: &str) -> Vec<Cell> {
        let mut cells = Vec::new();
        let mut chars = text.chars().peekable();
        
        while let Some(ch) = chars.next() {
            if ch == '\x1b' && chars.peek() == Some(&'[') {
                // Consume the '['
                chars.next();
                
                // Parse ANSI escape sequence
                let mut sequence = String::new();
                while let Some(&next_ch) = chars.peek() {
                    if next_ch.is_ascii_alphabetic() {
                        sequence.push(chars.next().unwrap());
                        break;
                    } else {
                        sequence.push(chars.next().unwrap());
                    }
                }
                
                self.process_ansi_sequence(&sequence);
            } else if ch != '\r' && ch != '\x1b' {
                // Regular character - create cell with current attributes
                let fg_color = self.color_to_string(&self.current_fg);
                let bg_color = self.bg_color_to_string(&self.current_bg);
                
                cells.push(Cell {
                    fg_color,
                    bg_color,
                    ch,
                });
            }
        }
        
        cells
    }

    /// Process ANSI escape sequence and update current attributes
    fn process_ansi_sequence(&mut self, sequence: &str) {
        if sequence.ends_with('m') {
            // SGR (Select Graphic Rendition) sequence
            let params = sequence.trim_end_matches('m');
            
            if params.is_empty() {
                // ESC[m is equivalent to ESC[0m (reset)
                self.reset();
                return;
            }
            
            for param in params.split(';') {
                if let Ok(code) = param.parse::<u8>() {
                    self.process_sgr_code(code);
                }
            }
        }
    }

    /// Process SGR (Select Graphic Rendition) codes
    fn process_sgr_code(&mut self, code: u8) {
        match code {
            0 => self.reset(), // Reset all attributes
            1 => self.current_bold = true, // Bold
            3 => self.current_italic = true, // Italic
            4 => self.current_underline = true, // Underline
            7 => self.current_reverse = true, // Reverse
            22 => self.current_bold = false, // Normal intensity
            23 => self.current_italic = false, // Not italic
            24 => self.current_underline = false, // Not underlined
            27 => self.current_reverse = false, // Not reversed
            
            // Standard foreground colors (30-37) - use AnsiValue to get exact codes
            30 => self.current_fg = Color::AnsiValue(0), // Black
            31 => self.current_fg = Color::AnsiValue(1), // Red
            32 => self.current_fg = Color::AnsiValue(2), // Green
            33 => self.current_fg = Color::AnsiValue(3), // Yellow
            34 => self.current_fg = Color::AnsiValue(4), // Blue
            35 => self.current_fg = Color::AnsiValue(5), // Magenta
            36 => self.current_fg = Color::AnsiValue(6), // Cyan
            37 => self.current_fg = Color::AnsiValue(7), // White
            39 => self.current_fg = Color::Reset, // Default foreground
            
            // Standard background colors (40-47) - use AnsiValue to get exact codes
            40 => self.current_bg = Color::AnsiValue(0), // Black
            41 => self.current_bg = Color::AnsiValue(1), // Red
            42 => self.current_bg = Color::AnsiValue(2), // Green
            43 => self.current_bg = Color::AnsiValue(3), // Yellow
            44 => self.current_bg = Color::AnsiValue(4), // Blue
            45 => self.current_bg = Color::AnsiValue(5), // Magenta
            46 => self.current_bg = Color::AnsiValue(6), // Cyan
            47 => self.current_bg = Color::AnsiValue(7), // White
            49 => self.current_bg = Color::Reset, // Default background
            
            // Bright foreground colors (90-97)
            90 => self.current_fg = Color::AnsiValue(8),  // Bright black
            91 => self.current_fg = Color::AnsiValue(9),  // Bright red
            92 => self.current_fg = Color::AnsiValue(10), // Bright green
            93 => self.current_fg = Color::AnsiValue(11), // Bright yellow
            94 => self.current_fg = Color::AnsiValue(12), // Bright blue
            95 => self.current_fg = Color::AnsiValue(13), // Bright magenta
            96 => self.current_fg = Color::AnsiValue(14), // Bright cyan
            97 => self.current_fg = Color::AnsiValue(15), // Bright white
            
            // Bright background colors (100-107)
            100 => self.current_bg = Color::AnsiValue(8),  // Bright black background
            101 => self.current_bg = Color::AnsiValue(9),  // Bright red background
            102 => self.current_bg = Color::AnsiValue(10), // Bright green background
            103 => self.current_bg = Color::AnsiValue(11), // Bright yellow background
            104 => self.current_bg = Color::AnsiValue(12), // Bright blue background
            105 => self.current_bg = Color::AnsiValue(13), // Bright magenta background
            106 => self.current_bg = Color::AnsiValue(14), // Bright cyan background
            107 => self.current_bg = Color::AnsiValue(15), // Bright white background
            
            _ => {} // Ignore unsupported codes for now
        }
    }

    /// Convert Color enum to string format used by BoxMux (using SetForegroundColor format)
    fn color_to_string(&self, color: &Color) -> String {
        use crossterm::style::SetForegroundColor;
        format!("{}", SetForegroundColor(*color))
    }

    /// Convert Color enum to background color string format used by BoxMux
    fn bg_color_to_string(&self, color: &Color) -> String {
        use crossterm::style::SetBackgroundColor;
        format!("{}", SetBackgroundColor(*color))
    }
}

/// Convenience function to process ANSI text and return cells
pub fn process_ansi_text(text: &str) -> Vec<Cell> {
    let mut processor = AnsiColorProcessor::new();
    processor.process_text(text)
}

/// Check if text contains ANSI escape sequences
pub fn contains_ansi_sequences(text: &str) -> bool {
    text.contains('\x1b')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_ansi_processing() {
        let mut processor = AnsiColorProcessor::new();
        let text = "\x1b[31mRed text\x1b[0m normal";
        let cells = processor.process_text(text);
        
        // Should have cells for "Red text normal"
        assert_eq!(cells.len(), 15); // "Red text" (8) + " normal" (7) = 15 chars
        
        // First few cells should have red color (ANSI code 1)
        assert!(cells[0].fg_color.contains("38;5;1")); // Red color code
    }

    #[test]
    fn test_color_reset() {
        let mut processor = AnsiColorProcessor::new();
        let text = "\x1b[31mRed\x1b[0mNormal";
        let cells = processor.process_text(text);
        
        assert_eq!(cells.len(), 9); // "RedNormal" = 9 chars
        
        // First 3 chars should be red
        assert!(cells[0].fg_color.contains("38;5;1"));
        assert!(cells[1].fg_color.contains("38;5;1"));
        assert!(cells[2].fg_color.contains("38;5;1"));
        
        // Next chars should be reset
        assert!(cells[3].fg_color.contains("39")); // Reset code
    }

    #[test]
    fn test_multiple_colors() {
        let mut processor = AnsiColorProcessor::new();
        let text = "\x1b[31mR\x1b[32mG\x1b[34mB";
        let cells = processor.process_text(text);
        
        assert_eq!(cells.len(), 3);
        assert!(cells[0].fg_color.contains("38;5;1")); // Red
        assert!(cells[1].fg_color.contains("38;5;2")); // Green
        assert!(cells[2].fg_color.contains("38;5;4")); // Blue
    }

    #[test]
    fn test_background_colors() {
        let mut processor = AnsiColorProcessor::new();
        let text = "\x1b[41mRed bg\x1b[0m";
        let cells = processor.process_text(text);
        
        assert_eq!(cells.len(), 6); // "Red bg"
        assert!(cells[0].bg_color.contains("48;5;1")); // Red background
    }

    #[test]
    fn test_no_ansi_sequences() {
        let mut processor = AnsiColorProcessor::new();
        let text = "Plain text";
        let cells = processor.process_text(text);
        
        assert_eq!(cells.len(), 10);
        // All cells should have default colors
        for cell in &cells {
            assert!(cell.fg_color.contains("39")); // Reset/default
        }
    }

    #[test]
    fn test_contains_ansi_sequences() {
        assert!(contains_ansi_sequences("\x1b[31mRed"));
        assert!(contains_ansi_sequences("Text \x1b[0m more"));
        assert!(!contains_ansi_sequences("Plain text"));
    }
}