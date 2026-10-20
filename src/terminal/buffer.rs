//! A screen of cells, with its scrollback.

use super::*;

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
