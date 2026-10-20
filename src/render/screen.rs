//! The frame being drawn: a grid of cells, each a character with its colors and attributes.
//!
//! A character that takes two columns (CJK, most emoji) occupies its cell and the next one; the
//! next cell holds [`Cell::CONTINUATION`] so every column of the grid is one terminal column.
//! Writing over either half of a wide character blanks the other half, so a frame never shows
//! half a character.

pub use crossterm::style::Color;
use unicode_width::UnicodeWidthChar;

/// Text attributes of a cell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Attrs(u8);

impl Attrs {
    pub const BOLD: Attrs = Attrs(1);
    pub const DIM: Attrs = Attrs(1 << 1);
    pub const ITALIC: Attrs = Attrs(1 << 2);
    pub const UNDERLINE: Attrs = Attrs(1 << 3);
    pub const BLINK: Attrs = Attrs(1 << 4);
    pub const REVERSE: Attrs = Attrs(1 << 5);
    pub const HIDDEN: Attrs = Attrs(1 << 6);
    pub const STRIKETHROUGH: Attrs = Attrs(1 << 7);

    pub const fn empty() -> Attrs {
        Attrs(0)
    }

    pub fn contains(self, other: Attrs) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn insert(&mut self, other: Attrs) {
        self.0 |= other.0;
    }

    pub fn remove(&mut self, other: Attrs) {
        self.0 &= !other.0;
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }
}

/// Colors and attributes of a cell. `Color::Reset` is the terminal's own default color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub attrs: Attrs,
}

impl Style {
    pub const fn new(fg: Color, bg: Color) -> Style {
        Style {
            fg,
            bg,
            attrs: Attrs::empty(),
        }
    }
}

impl Default for Style {
    fn default() -> Style {
        Style::new(Color::Reset, Color::Reset)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Cell {
    /// The right half of a two-column character.
    pub const CONTINUATION: char = '\0';

    pub const fn new(ch: char, style: Style) -> Cell {
        Cell { ch, style }
    }

    pub fn is_continuation(&self) -> bool {
        self.ch == Cell::CONTINUATION
    }
}

/// Columns `ch` takes on screen: 0 for combining marks and other zero-width characters, 2 for
/// wide characters, otherwise 1. Control characters never reach the screen; they count as 0.
pub fn char_width(ch: char) -> usize {
    ch.width().unwrap_or(0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    cells: Vec<Cell>,
}

impl ScreenBuffer {
    /// A buffer of `width` x `height` cells, each a space in `style`.
    pub fn new(width: usize, height: usize, style: Style) -> ScreenBuffer {
        ScreenBuffer {
            width,
            height,
            cells: vec![Cell::new(' ', style); width * height],
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        (x < self.width && y < self.height).then(|| &self.cells[y * self.width + x])
    }

    fn index(&self, x: usize, y: usize) -> Option<usize> {
        (x < self.width && y < self.height).then_some(y * self.width + x)
    }

    /// Blanks the halves of a wide character that a write to (`x`, `y`) would split.
    fn unsplit(&mut self, x: usize, y: usize) {
        let Some(i) = self.index(x, y) else {
            return;
        };
        if self.cells[i].is_continuation() && x > 0 {
            self.cells[i - 1].ch = ' ';
        }
        if char_width(self.cells[i].ch) == 2 {
            if let Some(next) = self.index(x + 1, y) {
                self.cells[next].ch = ' ';
            }
        }
    }

    /// Puts `ch` at (`x`, `y`) and returns the columns it took (0 when it does not fit). A wide
    /// character that would stick out past `limit` (the last usable column) is drawn as a
    /// space instead. Positions outside the buffer are ignored.
    pub fn put(&mut self, x: usize, y: usize, ch: char, style: Style, limit: usize) -> usize {
        debug_assert!(
            !ch.is_control(),
            "control character {:?} drawn to the screen",
            ch
        );
        let width = char_width(ch);
        if width == 0 || x > limit {
            return 0;
        }
        let Some(i) = self.index(x, y) else {
            return 0;
        };
        self.unsplit(x, y);
        if width == 2 {
            if x + 1 > limit || self.index(x + 1, y).is_none() {
                self.cells[i] = Cell::new(' ', style);
                return 1;
            }
            self.unsplit(x + 1, y);
            self.cells[i] = Cell::new(ch, style);
            self.cells[i + 1] = Cell::new(Cell::CONTINUATION, style);
            return 2;
        }
        self.cells[i] = Cell::new(ch, style);
        1
    }

    /// Draws `text` from (`x`, `y`) without passing column `limit`; returns the columns used.
    pub fn put_str(&mut self, x: usize, y: usize, text: &str, style: Style, limit: usize) -> usize {
        let mut column = x;
        for ch in text.chars() {
            if column > limit {
                break;
            }
            column += self.put(column, y, ch, style, limit);
        }
        column - x
    }

    /// Changes only the colors of the cell at (`x`, `y`).
    pub fn restyle(&mut self, x: usize, y: usize, style: Style) {
        if let Some(i) = self.index(x, y) {
            self.cells[i].style = style;
        }
    }

    /// Fills the cells from (`x1`, `y1`) to (`x2`, `y2`), inclusive, with `ch`.
    pub fn fill(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, ch: char, style: Style) {
        for y in y1..=y2.min(self.height.saturating_sub(1)) {
            for x in x1..=x2.min(self.width.saturating_sub(1)) {
                self.put(x, y, ch, style, x2);
            }
        }
    }

    /// The characters of each row, without colors. The right half of a wide character is
    /// left out, so each row reads as the text a terminal shows.
    pub fn text_rows(&self) -> Vec<String> {
        self.cells
            .chunks(self.width.max(1))
            .take(self.height)
            .map(|row| {
                row.iter()
                    .filter(|cell| !cell.is_continuation())
                    .map(|cell| cell.ch)
                    .collect()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain() -> Style {
        Style::default()
    }

    #[test]
    fn wide_characters_take_two_cells() {
        let mut buffer = ScreenBuffer::new(6, 1, plain());
        assert_eq!(buffer.put_str(0, 0, "日本x", plain(), 5), 5);
        assert_eq!(buffer.text_rows(), vec!["日本x "]);
        assert!(buffer.get(1, 0).unwrap().is_continuation());
    }

    #[test]
    fn a_wide_character_that_does_not_fit_becomes_a_space() {
        let mut buffer = ScreenBuffer::new(4, 1, plain());
        assert_eq!(buffer.put_str(0, 0, "ab日", plain(), 2), 3);
        assert_eq!(buffer.text_rows(), vec!["ab  "]);
    }

    #[test]
    fn overwriting_half_of_a_wide_character_blanks_the_other_half() {
        let mut buffer = ScreenBuffer::new(4, 1, plain());
        buffer.put_str(0, 0, "日本", plain(), 3);
        buffer.put(1, 0, '|', plain(), 3);
        assert_eq!(buffer.text_rows(), vec![" |本"]);
        buffer.put(2, 0, '-', plain(), 3);
        assert_eq!(buffer.text_rows(), vec![" |- "]);
    }

    #[test]
    fn writes_outside_the_buffer_are_ignored() {
        let mut buffer = ScreenBuffer::new(2, 1, plain());
        assert_eq!(buffer.put(5, 0, 'x', plain(), 10), 0);
        assert_eq!(buffer.put(0, 3, 'x', plain(), 10), 0);
        buffer.fill(0, 0, 10, 10, '#', plain());
        assert_eq!(buffer.text_rows(), vec!["##"]);
    }

    #[test]
    fn zero_width_characters_take_no_cell() {
        let mut buffer = ScreenBuffer::new(3, 1, plain());
        assert_eq!(buffer.put_str(0, 0, "e\u{301}x", plain(), 2), 2);
        assert_eq!(buffer.text_rows(), vec!["ex "]);
    }
}
