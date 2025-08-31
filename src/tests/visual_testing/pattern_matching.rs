// F0328: Visual Pattern Matching - Match terminal output against ASCII art patterns
// Pattern parsing, whitespace handling, flexible matching for borders/boxes/content

use super::terminal_capture::TerminalFrame;

/// F0328: Pattern matcher for ASCII art and visual layouts
pub struct PatternMatcher {
    /// Whether to ignore whitespace differences
    pub ignore_whitespace: bool,
    /// Whether to allow partial matches
    pub allow_partial: bool,
    /// Character used for wildcards in patterns
    pub wildcard_char: char,
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self {
            ignore_whitespace: false,
            allow_partial: false,
            wildcard_char: '?',
        }
    }
}

impl PatternMatcher {
    /// Create new pattern matcher with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Create pattern matcher that ignores whitespace
    pub fn ignore_whitespace() -> Self {
        Self {
            ignore_whitespace: true,
            ..Default::default()
        }
    }

    /// F0328: Match frame against ASCII art pattern
    pub fn matches(&self, frame: &TerminalFrame, pattern: &str) -> bool {
        self.matches_at_position(frame, pattern, 0, 0).is_ok()
    }

    /// F0328: Match pattern at specific position
    pub fn matches_at_position(
        &self,
        frame: &TerminalFrame,
        pattern: &str,
        start_x: u16,
        start_y: u16,
    ) -> Result<(), PatternMatchError> {
        let pattern_lines: Vec<&str> = pattern.lines().collect();

        if pattern_lines.is_empty() {
            return Ok(());
        }

        for (line_idx, pattern_line) in pattern_lines.iter().enumerate() {
            let frame_y = start_y + line_idx as u16;

            if frame_y >= frame.buffer.len() as u16 {
                if !self.allow_partial {
                    return Err(PatternMatchError::OutOfBounds(format!(
                        "Pattern line {} extends beyond frame height",
                        line_idx
                    )));
                }
                break;
            }

            self.match_line(
                &frame.buffer[frame_y as usize],
                pattern_line,
                start_x,
                frame_y,
            )?;
        }

        Ok(())
    }

    /// Match single line against pattern
    fn match_line(
        &self,
        frame_row: &[super::terminal_capture::TerminalCell],
        pattern_line: &str,
        start_x: u16,
        y: u16,
    ) -> Result<(), PatternMatchError> {
        let frame_line: String = frame_row.iter().map(|cell| cell.ch).collect();

        if self.ignore_whitespace {
            let frame_trimmed = frame_line.trim();
            let pattern_trimmed = pattern_line.trim();

            if self.matches_with_wildcards(frame_trimmed, pattern_trimmed) {
                return Ok(());
            }
        } else {
            let pattern_chars: Vec<char> = pattern_line.chars().collect();

            for (char_idx, &pattern_char) in pattern_chars.iter().enumerate() {
                let frame_x = start_x + char_idx as u16;

                if frame_x >= frame_row.len() as u16 {
                    if !self.allow_partial {
                        return Err(PatternMatchError::OutOfBounds(format!(
                            "Pattern extends beyond frame width at line {}",
                            y
                        )));
                    }
                    break;
                }

                let frame_char = frame_row[frame_x as usize].ch;

                if pattern_char != self.wildcard_char && pattern_char != frame_char {
                    return Err(PatternMatchError::Mismatch(format!(
                        "Character mismatch at ({}, {}): expected '{}', got '{}'",
                        frame_x, y, pattern_char, frame_char
                    )));
                }
            }
        }

        Ok(())
    }

    /// Match strings with wildcard support
    fn matches_with_wildcards(&self, frame_text: &str, pattern: &str) -> bool {
        let frame_chars: Vec<char> = frame_text.chars().collect();
        let pattern_chars: Vec<char> = pattern.chars().collect();

        if pattern_chars.is_empty() {
            return frame_chars.is_empty();
        }

        self.wildcard_match(&frame_chars, &pattern_chars, 0, 0)
    }

    /// Recursive wildcard matching
    fn wildcard_match(
        &self,
        frame_chars: &[char],
        pattern_chars: &[char],
        frame_idx: usize,
        pattern_idx: usize,
    ) -> bool {
        // End of pattern
        if pattern_idx >= pattern_chars.len() {
            return frame_idx >= frame_chars.len();
        }

        // End of frame text
        if frame_idx >= frame_chars.len() {
            return pattern_chars[pattern_idx..]
                .iter()
                .all(|&c| c == self.wildcard_char);
        }

        let pattern_char = pattern_chars[pattern_idx];
        let frame_char = frame_chars[frame_idx];

        if pattern_char == self.wildcard_char {
            // Wildcard matches any single character
            self.wildcard_match(frame_chars, pattern_chars, frame_idx + 1, pattern_idx + 1)
        } else if pattern_char == frame_char {
            // Exact character match
            self.wildcard_match(frame_chars, pattern_chars, frame_idx + 1, pattern_idx + 1)
        } else {
            // No match
            false
        }
    }

    /// F0328: Find pattern in frame and return position
    pub fn find_pattern(&self, frame: &TerminalFrame, pattern: &str) -> Option<(u16, u16)> {
        let pattern_lines: Vec<&str> = pattern.lines().collect();
        if pattern_lines.is_empty() {
            return None;
        }

        let pattern_height = pattern_lines.len() as u16;
        let pattern_width = pattern_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0) as u16;

        for y in 0..=(frame.buffer.len() as u16).saturating_sub(pattern_height) {
            if let Some(row) = frame.buffer.get(y as usize) {
                for x in 0..=(row.len() as u16).saturating_sub(pattern_width) {
                    if self.matches_at_position(frame, pattern, x, y).is_ok() {
                        return Some((x, y));
                    }
                }
            }
        }

        None
    }
}

/// F0328: Pattern matching errors
#[derive(Debug, Clone)]
pub enum PatternMatchError {
    OutOfBounds(String),
    Mismatch(String),
    InvalidPattern(String),
}

impl std::fmt::Display for PatternMatchError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            PatternMatchError::OutOfBounds(msg) => write!(f, "Out of bounds: {}", msg),
            PatternMatchError::Mismatch(msg) => write!(f, "Pattern mismatch: {}", msg),
            PatternMatchError::InvalidPattern(msg) => write!(f, "Invalid pattern: {}", msg),
        }
    }
}

impl std::error::Error for PatternMatchError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::visual_testing::terminal_capture::{TerminalCell, TerminalFrame};
    use std::time::Instant;

    fn create_test_frame(content: &[&str]) -> TerminalFrame {
        let height = content.len();
        let width = content.iter().map(|line| line.len()).max().unwrap_or(0);

        let mut buffer = Vec::new();
        for line in content {
            let mut row = Vec::new();
            for ch in line.chars() {
                row.push(TerminalCell {
                    ch,
                    ..Default::default()
                });
            }
            // Pad row to width
            while row.len() < width {
                row.push(TerminalCell::default());
            }
            buffer.push(row);
        }

        TerminalFrame {
            buffer,
            cursor: (0, 0),
            cursor_visible: true,
            timestamp: Instant::now(),
            dimensions: (width as u16, height as u16),
        }
    }

    #[test]
    fn test_exact_pattern_match() {
        let frame = create_test_frame(&["┌─ Box ─┐", "│ Hello │", "└───────┘"]);

        let pattern = "┌─ Box ─┐\n│ Hello │\n└───────┘";
        let matcher = PatternMatcher::new();

        assert!(matcher.matches(&frame, pattern));
    }

    #[test]
    fn test_wildcard_pattern_match() {
        let frame = create_test_frame(&["┌─ Test ─┐", "│ World  │", "└────────┘"]);

        let pattern = "┌─ ???? ─┐\n│ ?????  │\n└────────┘";
        let matcher = PatternMatcher::new();

        assert!(matcher.matches(&frame, pattern));
    }

    #[test]
    fn test_find_pattern_position() {
        let frame = create_test_frame(&["       ", "  ┌─┐  ", "  │X│  ", "  └─┘  ", "       "]);

        let pattern = "┌─┐\n│X│\n└─┘";
        let matcher = PatternMatcher::new();

        assert_eq!(matcher.find_pattern(&frame, pattern), Some((2, 1)));
    }
}
