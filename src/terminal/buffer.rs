//! A screen of cells, with its scrollback.

use super::*;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct TerminalScreenBuffer {
    pub content: Vec<Vec<TerminalCell>>,
    /// Lines that scrolled off the top of the screen, oldest first.
    pub scrollback: VecDeque<Vec<TerminalCell>>,
    pub cursor: CursorState,
    pub max_scrollback: usize,
    pub width: usize,
    pub height: usize,
}

impl TerminalScreenBuffer {
    pub fn new(width: usize, height: usize, max_scrollback: usize) -> Self {
        TerminalScreenBuffer {
            content: vec![vec![TerminalCell::default(); width]; height],
            scrollback: VecDeque::new(),
            cursor: CursorState {
                x: 0,
                y: 0,
                visible: true,
            },
            max_scrollback,
            width,
            height,
        }
    }

    /// Keeps `line`, which scrolled off the top, in the scrollback.
    pub fn push_scrollback(&mut self, line: Vec<TerminalCell>) {
        if self.max_scrollback == 0 {
            return;
        }
        if self.scrollback.len() == self.max_scrollback {
            self.scrollback.pop_front();
        }
        self.scrollback.push_back(line);
    }

    /// Blanks every cell and puts the cursor at the top left.
    pub fn clear_screen(&mut self) {
        for row in &mut self.content {
            row.fill(TerminalCell::default());
        }
        self.cursor.x = 0;
        self.cursor.y = 0;
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.width = new_width;
        self.height = new_height;
        for row in &mut self.content {
            row.resize(new_width, TerminalCell::default());
            // A wide character cut in half at the new edge cannot be shown.
            if let Some(last) = row.last_mut() {
                if last.glyph.width() == 2 {
                    last.glyph = Glyph::SPACE;
                }
            }
        }
        // A shorter screen keeps its bottom lines, like a terminal window being made smaller;
        // the lines above go to the scrollback.
        while self.content.len() > new_height && self.cursor.y > 0 {
            let line = self.content.remove(0);
            self.push_scrollback(line);
            self.cursor.y -= 1;
        }
        self.content.truncate(new_height);
        while self.content.len() < new_height {
            self.content.push(vec![TerminalCell::default(); new_width]);
        }
        self.cursor.x = self.cursor.x.min(new_width.saturating_sub(1));
        self.cursor.y = self.cursor.y.min(new_height.saturating_sub(1));
    }

    /// The lines with SGR sequences for their colors: the scrollback (when
    /// `include_scrollback`) and the screen. When `cursor` is set, the cell under the cursor is
    /// shown in reverse video.
    pub fn to_content_lines(&self, include_scrollback: bool, cursor: bool) -> Vec<String> {
        let mut lines = Vec::new();
        if include_scrollback {
            lines.extend(self.scrollback.iter().map(|row| row_to_string(row, None)));
        }
        for (y, row) in self.content.iter().enumerate() {
            let cursor_x = (cursor && y == self.cursor.y).then_some(self.cursor.x);
            lines.push(row_to_string(row, cursor_x));
        }
        lines
    }
}

/// A row as text with SGR sequences for its colors and attributes and OSC 8 sequences around
/// the cells of each hyperlink, the cell at `cursor_x` in reverse video. The right halves of
/// wide characters are left out.
fn row_to_string(row: &[TerminalCell], cursor_x: Option<usize>) -> String {
    fn sgr(cell: &TerminalCell) -> String {
        let mut codes = vec!["0".to_string()];
        for (on, code) in [
            (cell.bold, "1"),
            (cell.dim, "2"),
            (cell.italic, "3"),
            (cell.underline, "4"),
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
    // The cursor on the right half of a wide character is shown on its left half.
    let cursor_x = cursor_x.map(|x| {
        if x > 0 && row.get(x).is_some_and(|c| c.glyph == WIDE_CONTINUATION) {
            x - 1
        } else {
            x
        }
    });
    let plain = sgr(&TerminalCell::default());
    let mut result = String::new();
    let mut current = plain.clone();
    let mut link: Option<&Arc<Link>> = None;
    for (x, cell) in row.iter().enumerate() {
        if cell.glyph == WIDE_CONTINUATION {
            continue;
        }
        if link != cell.link.as_ref() {
            if link.is_some() {
                result.push_str(crate::link::CLOSE);
            }
            if let Some(opened) = &cell.link {
                result.push_str(&opened.open_sequence());
            }
            link = cell.link.as_ref();
        }
        let wanted = if cursor_x == Some(x) {
            let mut shown = cell.clone();
            shown.reverse = !shown.reverse;
            sgr(&shown)
        } else {
            sgr(cell)
        };
        if wanted != current {
            result.push_str(&wanted);
            current = wanted;
        }
        cell.glyph.push_to(&mut result);
    }
    if link.is_some() {
        result.push_str(crate::link::CLOSE);
    }
    if current != plain {
        result.push_str("\x1b[0m");
    }
    result
}
