//! Text cleanup for script output shown in a box.
//!
//! Script output is a stream of bytes meant for a terminal. A box shows it as lines, so each
//! line is reduced to what a terminal would leave on screen for that line: carriage returns go
//! back to the start of the line and later text overwrites earlier text (progress bars),
//! backspace steps back one column, tabs advance to the next 8-column stop, SGR color sequences
//! are kept, and every other control character or escape sequence is dropped. Columns are
//! counted as [`crate::width`] says: a wide cluster takes two, a combining mark joins the
//! cluster before it.

use crate::width::{clusters, width, Glyph};

/// One column of a line being rebuilt: the SGR sequences that precede it, then what it shows;
/// `None` for the right half of a two-column cluster.
#[derive(Clone)]
struct Column {
    styles: String,
    glyph: Option<Glyph>,
}

impl Column {
    fn blank() -> Column {
        Column {
            styles: String::new(),
            glyph: Some(Glyph::SPACE),
        }
    }
}

/// A line being rebuilt, with its cursor.
#[derive(Default)]
struct Line {
    columns: Vec<Column>,
    cursor: usize,
    /// SGR sequences not yet attached to a column.
    pending_styles: String,
    /// The column of the cluster written last, while the cursor has not been moved since: a
    /// combining mark or selector that follows belongs to it.
    last: Option<usize>,
}

impl Line {
    /// Blanks the other half of a two-column cluster that writing column `x` cuts.
    fn unsplit(&mut self, x: usize) {
        let Some(column) = self.columns.get(x) else {
            return;
        };
        match &column.glyph {
            None if x > 0 => self.columns[x - 1].glyph = Some(Glyph::SPACE),
            Some(glyph) if glyph.width() == 2 => {
                if let Some(next) = self.columns.get_mut(x + 1) {
                    next.glyph = Some(Glyph::SPACE);
                }
            }
            _ => {}
        }
    }

    /// Makes column `x` the right half of the cluster before it.
    fn continue_into(&mut self, x: usize) {
        while self.columns.len() <= x {
            self.columns.push(Column::blank());
        }
        self.unsplit(x);
        self.columns[x].glyph = None;
    }

    fn put(&mut self, cluster: &str) {
        let w = width(cluster);
        if w == 0 {
            if let Some(at) = self.last {
                let before = self.columns[at]
                    .glyph
                    .clone()
                    .expect("a cluster was written there");
                let joined = before.joined(cluster);
                if joined.width() > before.width() {
                    self.continue_into(at + 1);
                    self.cursor = at + 2;
                } else if joined.width() < before.width() {
                    self.columns[at + 1].glyph = Some(Glyph::SPACE);
                }
                self.columns[at].glyph = Some(joined);
            }
            return;
        }
        while self.columns.len() < self.cursor {
            self.columns.push(Column::blank());
        }
        let at = self.cursor;
        let styles = std::mem::take(&mut self.pending_styles);
        let column = Column {
            styles,
            glyph: Some(Glyph::new(cluster)),
        };
        if at < self.columns.len() {
            self.unsplit(at);
            // Overwriting keeps styles attached to the old column so earlier color changes
            // still take effect.
            let previous = std::mem::replace(&mut self.columns[at], column);
            self.columns[at].styles.insert_str(0, &previous.styles);
        } else {
            self.columns.push(column);
        }
        if w == 2 {
            self.continue_into(at + 1);
        }
        self.last = Some(at);
        self.cursor = at + w;
    }

    /// Writes each cluster of `run`, text without control characters.
    fn put_run(&mut self, run: &mut String) {
        for cluster in clusters(run) {
            self.put(cluster);
        }
        run.clear();
    }

    fn move_to(&mut self, column: usize) {
        self.cursor = column;
        self.last = None;
    }

    fn into_text(self, capacity: usize) -> String {
        let mut out = String::with_capacity(capacity);
        for column in &self.columns {
            out.push_str(&column.styles);
            if let Some(glyph) = &column.glyph {
                glyph.push_to(&mut out);
            }
        }
        out.push_str(&self.pending_styles);
        out
    }
}

/// Reduces one line of terminal output (without its newline) to displayable text.
pub fn sanitize_line(text: &str) -> String {
    let mut line = Line::default();
    // Text since the last control character or escape sequence.
    let mut run = String::new();
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if !ch.is_control() {
            run.push(ch);
            continue;
        }
        line.put_run(&mut run);
        match ch {
            '\r' => line.move_to(0),
            '\u{8}' => line.move_to(line.cursor.saturating_sub(1)),
            '\t' => {
                for _ in 0..8 - line.cursor % 8 {
                    line.put(" ");
                }
                line.last = None;
            }
            '\u{1b}' => match chars.peek() {
                Some('[') => {
                    chars.next();
                    let mut sequence = String::from("\u{1b}[");
                    let mut terminated = false;
                    for next in chars.by_ref() {
                        sequence.push(next);
                        if ('\u{40}'..='\u{7e}').contains(&next) {
                            terminated = true;
                            break;
                        }
                    }
                    if terminated && sequence.ends_with('m') {
                        line.pending_styles.push_str(&sequence);
                    }
                }
                Some(']') => {
                    // OSC: skip to BEL or ST.
                    chars.next();
                    while let Some(next) = chars.next() {
                        if next == '\u{7}' {
                            break;
                        }
                        if next == '\u{1b}' && chars.peek() == Some(&'\\') {
                            chars.next();
                            break;
                        }
                    }
                }
                Some(_) => {
                    // Two-character escape such as ESC ( B.
                    chars.next();
                }
                None => {}
            },
            _ => {}
        }
    }
    line.put_run(&mut run);
    line.into_text(text.len())
}

/// Splits output into lines and sanitizes each. A trailing newline does not produce an empty
/// last line.
pub fn sanitize_output(text: &str) -> Vec<String> {
    text.lines().map(sanitize_line).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn carriage_return_overwrites_from_the_line_start() {
        assert_eq!(sanitize_line("progress 10%\rprogress 99%"), "progress 99%");
        assert_eq!(sanitize_line("abcdef\rXY"), "XYcdef");
    }

    #[test]
    fn tabs_expand_to_eight_column_stops() {
        assert_eq!(sanitize_line("a\tb"), "a       b");
        assert_eq!(sanitize_line("12345678\tx"), "12345678        x");
    }

    #[test]
    fn backspace_steps_back_and_is_overwritten() {
        assert_eq!(sanitize_line("ab\u{8}c"), "ac");
    }

    #[test]
    fn color_sequences_are_kept_and_other_escapes_dropped() {
        assert_eq!(
            sanitize_line("\u{1b}[31mred\u{1b}[0m \u{1b}[2Kok\u{1b}]0;title\u{7}"),
            "\u{1b}[31mred\u{1b}[0m ok"
        );
    }

    #[test]
    fn control_characters_are_removed() {
        assert_eq!(sanitize_line("bell\u{7}!"), "bell!");
    }

    #[test]
    fn wide_characters_count_two_columns_for_tab_stops() {
        assert_eq!(sanitize_line("日本\tx"), "日本    x");
    }

    #[test]
    fn crlf_output_loses_the_carriage_return() {
        assert_eq!(sanitize_output("one\r\ntwo\r\n"), vec!["one", "two"]);
    }

    #[test]
    fn overwriting_counts_columns_and_never_leaves_half_a_wide_character() {
        // `a` cuts 日 in half; `b` goes in its right half's column.
        assert_eq!(sanitize_line("日本\rab"), "ab本");
        assert_eq!(sanitize_line("日本\rabc"), "abc ");
        assert_eq!(sanitize_line("ab\u{8}日"), "a日");
    }

    #[test]
    fn combining_marks_and_selectors_stay_with_their_character() {
        assert_eq!(sanitize_line("e\u{301}t\u{8}\u{8}E"), "Et");
        assert_eq!(sanitize_line("☀\u{fe0f}\tx"), "☀\u{fe0f}      x");
        // A color change between a letter and its accent does not part them.
        assert_eq!(
            sanitize_line("\u{1b}[31me\u{1b}[0m\u{301}x"),
            "\u{1b}[31me\u{301}\u{1b}[0mx"
        );
    }
}
