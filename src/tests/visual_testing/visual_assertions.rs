// F0327: Character-Exact Assertions - Precise coordinate validation
// Assert specific characters at exact terminal coordinates

use super::terminal_capture::{TerminalCell, TerminalFrame};
use std::fmt;

/// F0327: Visual assertion trait for precise validation
pub trait VisualAssertions {
    /// Assert character at specific coordinate
    fn assert_char_at(&self, x: u16, y: u16, expected: char) -> Result<(), AssertionError>;

    /// Assert line contains specific text
    fn assert_line_contains(&self, y: u16, expected: &str) -> Result<(), AssertionError>;

    /// Assert specific region matches expected content
    fn assert_screen_region(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        expected: &str,
    ) -> Result<(), AssertionError>;

    /// Assert cursor is at expected position
    fn assert_cursor_at(&self, x: u16, y: u16) -> Result<(), AssertionError>;

    /// Assert cursor visibility state
    fn assert_cursor_visible(&self, visible: bool) -> Result<(), AssertionError>;

    /// Assert line is empty (all spaces)
    fn assert_line_empty(&self, y: u16) -> Result<(), AssertionError>;

    /// Assert character has specific attributes
    fn assert_char_attributes(
        &self,
        x: u16,
        y: u16,
        fg_color: Option<u8>,
        bg_color: Option<u8>,
    ) -> Result<(), AssertionError>;

    /// Assert screen contains specific text anywhere
    fn assert_contains_text(&self, expected: &str) -> Result<(), AssertionError>;

    /// Assert screen has a border (corners and edges)
    fn assert_has_border(&self) -> Result<(), AssertionError>;
}

/// F0327: Assertion error with detailed context
#[derive(Debug, Clone)]
pub struct AssertionError {
    pub message: String,
    pub expected: String,
    pub actual: String,
    pub position: Option<(u16, u16)>,
    pub context: Vec<String>,
}

impl fmt::Display for AssertionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Visual Assertion Failed: {}\nExpected: {}\nActual: {}",
            self.message, self.expected, self.actual
        )?;

        if let Some((x, y)) = self.position {
            write!(f, "\nPosition: ({}, {})", x, y)?;
        }

        if !self.context.is_empty() {
            write!(f, "\nContext:\n{}", self.context.join("\n"))?;
        }

        Ok(())
    }
}

impl std::error::Error for AssertionError {}

/// F0327: Implementation of visual assertions for TerminalFrame
impl VisualAssertions for TerminalFrame {
    fn assert_char_at(&self, x: u16, y: u16, expected: char) -> Result<(), AssertionError> {
        if y >= self.buffer.len() as u16 {
            return Err(AssertionError {
                message: "Y coordinate out of bounds".to_string(),
                expected: format!("char '{}' at ({}, {})", expected, x, y),
                actual: format!("y={} >= height={}", y, self.buffer.len()),
                position: Some((x, y)),
                context: vec![format!(
                    "Terminal dimensions: {}x{}",
                    self.dimensions.0, self.dimensions.1
                )],
            });
        }

        let row = &self.buffer[y as usize];
        if x >= row.len() as u16 {
            return Err(AssertionError {
                message: "X coordinate out of bounds".to_string(),
                expected: format!("char '{}' at ({}, {})", expected, x, y),
                actual: format!("x={} >= width={}", x, row.len()),
                position: Some((x, y)),
                context: vec![format!("Row {} length: {}", y, row.len())],
            });
        }

        let actual_char = row[x as usize].ch;
        if actual_char != expected {
            return Err(AssertionError {
                message: format!("Character mismatch at ({}, {})", x, y),
                expected: format!("'{}'", expected),
                actual: format!("'{}'", actual_char),
                position: Some((x, y)),
                context: vec![
                    format!("Expected: '{}' (U+{:04X})", expected, expected as u32),
                    format!("Actual:   '{}' (U+{:04X})", actual_char, actual_char as u32),
                    self.get_surrounding_context(x, y, 3),
                ],
            });
        }

        Ok(())
    }

    fn assert_line_contains(&self, y: u16, expected: &str) -> Result<(), AssertionError> {
        if y >= self.buffer.len() as u16 {
            return Err(AssertionError {
                message: "Y coordinate out of bounds".to_string(),
                expected: format!("line {} contains '{}'", y, expected),
                actual: format!("y={} >= height={}", y, self.buffer.len()),
                position: Some((0, y)),
                context: vec![format!("Terminal height: {}", self.buffer.len())],
            });
        }

        let row = &self.buffer[y as usize];
        let line_content: String = row.iter().map(|cell| cell.ch).collect();

        if !line_content.contains(expected) {
            return Err(AssertionError {
                message: format!("Line {} does not contain expected text", y),
                expected: format!("contains '{}'", expected),
                actual: format!("'{}'", line_content.trim()),
                position: Some((0, y)),
                context: vec![
                    format!("Full line content: '{}'", line_content),
                    format!("Looking for: '{}'", expected),
                ],
            });
        }

        Ok(())
    }

    fn assert_screen_region(
        &self,
        x: u16,
        y: u16,
        width: u16,
        height: u16,
        expected: &str,
    ) -> Result<(), AssertionError> {
        let mut actual_region = Vec::new();

        for row_offset in 0..height {
            let row_y = y + row_offset;
            if row_y >= self.buffer.len() as u16 {
                break;
            }

            let mut row_content = String::new();
            let row = &self.buffer[row_y as usize];

            for col_offset in 0..width {
                let col_x = x + col_offset;
                if col_x >= row.len() as u16 {
                    break;
                }
                row_content.push(row[col_x as usize].ch);
            }

            actual_region.push(row_content);
        }

        let actual = actual_region.join("\n");
        let expected_trimmed = expected.trim();

        if actual.trim() != expected_trimmed {
            return Err(AssertionError {
                message: format!("Region mismatch at ({}, {}) {}x{}", x, y, width, height),
                expected: format!("'{}'", expected_trimmed),
                actual: format!("'{}'", actual.trim()),
                position: Some((x, y)),
                context: vec![
                    format!("Region dimensions: {}x{}", width, height),
                    format!("Expected:\n{}", expected_trimmed),
                    format!("Actual:\n{}", actual.trim()),
                ],
            });
        }

        Ok(())
    }

    fn assert_cursor_at(&self, x: u16, y: u16) -> Result<(), AssertionError> {
        let (actual_x, actual_y) = self.cursor;

        if actual_x != x || actual_y != y {
            return Err(AssertionError {
                message: "Cursor position mismatch".to_string(),
                expected: format!("cursor at ({}, {})", x, y),
                actual: format!("cursor at ({}, {})", actual_x, actual_y),
                position: Some((actual_x, actual_y)),
                context: vec![format!("Cursor visible: {}", self.cursor_visible)],
            });
        }

        Ok(())
    }

    fn assert_cursor_visible(&self, visible: bool) -> Result<(), AssertionError> {
        if self.cursor_visible != visible {
            return Err(AssertionError {
                message: "Cursor visibility mismatch".to_string(),
                expected: format!("cursor visible: {}", visible),
                actual: format!("cursor visible: {}", self.cursor_visible),
                position: Some(self.cursor),
                context: vec![format!(
                    "Cursor position: ({}, {})",
                    self.cursor.0, self.cursor.1
                )],
            });
        }

        Ok(())
    }

    fn assert_line_empty(&self, y: u16) -> Result<(), AssertionError> {
        if y >= self.buffer.len() as u16 {
            return Err(AssertionError {
                message: "Y coordinate out of bounds".to_string(),
                expected: format!("empty line at y={}", y),
                actual: format!("y={} >= height={}", y, self.buffer.len()),
                position: Some((0, y)),
                context: vec![format!("Terminal height: {}", self.buffer.len())],
            });
        }

        let row = &self.buffer[y as usize];
        let line_content: String = row.iter().map(|cell| cell.ch).collect();
        let trimmed = line_content.trim();

        if !trimmed.is_empty() {
            return Err(AssertionError {
                message: format!("Line {} is not empty", y),
                expected: "empty line".to_string(),
                actual: format!("'{}'", trimmed),
                position: Some((0, y)),
                context: vec![
                    format!("Full line: '{}'", line_content),
                    format!("Non-whitespace characters: {}", trimmed.len()),
                ],
            });
        }

        Ok(())
    }

    fn assert_char_attributes(
        &self,
        x: u16,
        y: u16,
        fg_color: Option<u8>,
        bg_color: Option<u8>,
    ) -> Result<(), AssertionError> {
        if y >= self.buffer.len() as u16 || x >= self.buffer[y as usize].len() as u16 {
            return Err(AssertionError {
                message: "Coordinates out of bounds".to_string(),
                expected: format!("attributes at ({}, {})", x, y),
                actual: "out of bounds".to_string(),
                position: Some((x, y)),
                context: vec![format!(
                    "Terminal dimensions: {}x{}",
                    self.dimensions.0, self.dimensions.1
                )],
            });
        }

        let cell = &self.buffer[y as usize][x as usize];

        if cell.fg_color != fg_color || cell.bg_color != bg_color {
            return Err(AssertionError {
                message: format!("Color attributes mismatch at ({}, {})", x, y),
                expected: format!("fg={:?}, bg={:?}", fg_color, bg_color),
                actual: format!("fg={:?}, bg={:?}", cell.fg_color, cell.bg_color),
                position: Some((x, y)),
                context: vec![
                    format!("Character: '{}'", cell.ch),
                    format!("Attributes: {:?}", cell.attributes),
                ],
            });
        }

        Ok(())
    }

    fn assert_contains_text(&self, expected: &str) -> Result<(), AssertionError> {
        // Search through all screen content for the expected text
        for (y, row) in self.buffer.iter().enumerate() {
            let line: String = row.iter().map(|cell| cell.ch).collect();
            if line.contains(expected) {
                return Ok(());
            }
        }

        // If not found, create error with context
        let mut context = Vec::new();
        context.push("Screen content:".to_string());
        for (y, row) in self.buffer.iter().enumerate().take(10) {
            let line: String = row.iter().map(|cell| cell.ch).collect();
            context.push(format!("{:2}: '{}'", y, line.trim_end()));
        }

        Err(AssertionError {
            message: "Text not found in screen content".to_string(),
            expected: format!("text containing '{}'", expected),
            actual: "text not found".to_string(),
            position: None,
            context,
        })
    }

    fn assert_has_border(&self) -> Result<(), AssertionError> {
        // Check for basic border characters at expected positions
        // This is a simplified check for demonstration
        
        if self.buffer.is_empty() || self.buffer[0].is_empty() {
            return Err(AssertionError {
                message: "Empty screen buffer".to_string(),
                expected: "border characters".to_string(),
                actual: "empty buffer".to_string(),
                position: None,
                context: vec!["Screen is empty".to_string()],
            });
        }

        // Look for border-like characters in the first row
        let first_row: String = self.buffer[0].iter().map(|cell| cell.ch).collect();
        let has_border_chars = first_row.chars().any(|c| {
            matches!(c, '┌' | '┐' | '└' | '┘' | '─' | '│' | '┬' | '┴' | '├' | '┤' | '┼')
        });

        if has_border_chars {
            Ok(())
        } else {
            Err(AssertionError {
                message: "No border characters detected".to_string(),
                expected: "border characters (┌┐└┘─│)".to_string(),
                actual: format!("first row: '{}'", first_row.trim()),
                position: Some((0, 0)),
                context: vec!["Expected box drawing characters".to_string()],
            })
        }
    }
}

impl TerminalFrame {
    /// F0327: Get surrounding context for better error messages
    fn get_surrounding_context(&self, x: u16, y: u16, radius: u16) -> String {
        let mut context = Vec::new();

        let start_y = y.saturating_sub(radius);
        let end_y = (y + radius + 1).min(self.buffer.len() as u16);

        for row_idx in start_y..end_y {
            if row_idx >= self.buffer.len() as u16 {
                break;
            }

            let row = &self.buffer[row_idx as usize];
            let line: String = row.iter().map(|cell| cell.ch).collect();

            let marker = if row_idx == y {
                format!("{:2}→", row_idx)
            } else {
                format!("{:2} ", row_idx)
            };

            context.push(format!("{} {}", marker, line));

            // Add cursor indicator for the target row
            if row_idx == y {
                let mut pointer = " ".repeat(4 + x as usize);
                pointer.push('^');
                context.push(pointer);
            }
        }

        context.join("\n")
    }
}

/// F0327: Convenience macros for visual assertions
#[macro_export]
macro_rules! assert_char {
    ($frame:expr, $x:expr, $y:expr, $expected:expr) => {
        use $crate::tests::visual_testing::visual_assertions::VisualAssertions;
        $frame.assert_char_at($x, $y, $expected).unwrap();
    };
}

#[macro_export]
macro_rules! assert_line_contains {
    ($frame:expr, $y:expr, $expected:expr) => {
        use $crate::tests::visual_testing::visual_assertions::VisualAssertions;
        $frame.assert_line_contains($y, $expected).unwrap();
    };
}

#[macro_export]
macro_rules! assert_region {
    ($frame:expr, $x:expr, $y:expr, $width:expr, $height:expr, $expected:expr) => {
        use $crate::tests::visual_testing::visual_assertions::VisualAssertions;
        $frame
            .assert_screen_region($x, $y, $width, $height, $expected)
            .unwrap();
    };
}
