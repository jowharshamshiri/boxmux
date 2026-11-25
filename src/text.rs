//! Text cleanup for script output shown in a box.
//!
//! Script output is a stream of bytes meant for a terminal. A box shows it as lines, so each
//! line is reduced to what a terminal would leave on screen for that line: carriage returns go
//! back to the start of the line and later text overwrites earlier text (progress bars),
//! backspace steps back one column, tabs move to the next 8-column stop without writing the
//! columns they pass, each column keeps the style its SGR sequences gave it ([`apply_sgr`]) and
//! the OSC 8 hyperlink ([`crate::link`]) it was written in, and every other control character or
//! escape sequence is dropped. Overwriting a column replaces its text, style and link and leaves
//! every other column as it was. Columns are counted as [`crate::width`] says: a wide cluster
//! takes two, a combining mark joins the cluster before it.
//!
//! The line comes back as text with SGR sequences where its columns' style changes, starting
//! from no style: a reset returns to it, as it returns to a box's own colors when drawn.

use crate::link::{self, Link, LinkChange};
use crate::render::screen::{Attrs, Color, Style};
use crate::render::styled::apply_sgr;
use crate::terminal::TermColor;
use crate::width::{clusters, width, Glyph};
use std::iter::Peekable;
use std::str::Chars;
use std::sync::Arc;

/// An escape sequence, as [`read_escape`] reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Escape {
    /// `ESC [ params final`: a control sequence (CSI), such as SGR (`m`).
    Csi { params: String, final_char: char },
    /// `ESC ] body` ended by BEL or ST (`ESC \`): an operating system command.
    Osc(String),
    /// Any other escape (`ESC ( B`, `ESC 7`), read to its final character, and a CSI or OSC
    /// that the end of the text or, for an OSC, another escape cuts off: it means nothing.
    Other,
}

/// Reads the escape sequence that follows an ESC from `chars`. An OSC cut off by another ESC
/// leaves that ESC unread, as a terminal starts a new sequence there.
pub fn read_escape(chars: &mut Peekable<Chars>) -> Escape {
    match chars.peek() {
        Some('[') => {
            chars.next();
            let mut params = String::new();
            for next in chars.by_ref() {
                if ('\u{40}'..='\u{7e}').contains(&next) {
                    return Escape::Csi {
                        params,
                        final_char: next,
                    };
                }
                params.push(next);
            }
            Escape::Other
        }
        Some(']') => {
            chars.next();
            let mut body = String::new();
            while let Some(&next) = chars.peek() {
                if next == '\u{7}' {
                    chars.next();
                    return Escape::Osc(body);
                }
                if next == '\u{1b}' {
                    let mut ahead = chars.clone();
                    ahead.next();
                    if ahead.next() == Some('\\') {
                        chars.next();
                        chars.next();
                        return Escape::Osc(body);
                    }
                    return Escape::Other;
                }
                body.push(next);
                chars.next();
            }
            Escape::Other
        }
        _ => {
            // Intermediate characters, then the final one (ECMA-48: ESC I* F).
            while chars
                .peek()
                .is_some_and(|c| ('\u{20}'..='\u{2f}').contains(c))
            {
                chars.next();
            }
            if chars
                .peek()
                .is_some_and(|c| ('\u{30}'..='\u{7e}').contains(c))
            {
                chars.next();
            }
            Escape::Other
        }
    }
}

/// One column of a line being rebuilt: what it shows (`None` for the right half of a two-column
/// cluster), the style it was written in and the hyperlink it was written in.
#[derive(Clone)]
struct Column {
    glyph: Option<Glyph>,
    style: Style,
    link: Option<Arc<Link>>,
}

impl Column {
    /// A column nothing was written to.
    fn blank() -> Column {
        Column {
            glyph: Some(Glyph::SPACE),
            style: Style::default(),
            link: None,
        }
    }
}

/// A line being rebuilt, with its cursor.
#[derive(Default)]
struct Line {
    columns: Vec<Column>,
    cursor: usize,
    /// The style text written now is in.
    style: Style,
    /// The hyperlink text written now goes in.
    link: Option<Arc<Link>>,
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
        self.columns[x] = Column {
            glyph: None,
            ..self.columns[x - 1].clone()
        };
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
        let column = Column {
            glyph: Some(Glyph::new(cluster)),
            style: self.style,
            link: self.link.clone(),
        };
        if at < self.columns.len() {
            self.unsplit(at);
            self.columns[at] = column;
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

    /// The line as text: an SGR sequence before each column whose style differs from the one
    /// before it (the first from no style), and each run of columns of one hyperlink inside that
    /// link's OSC 8 sequences.
    fn into_text(self, capacity: usize) -> String {
        let mut out = String::with_capacity(capacity);
        let mut open: Option<&Arc<Link>> = None;
        let mut style = Style::default();
        for column in &self.columns {
            let Some(glyph) = &column.glyph else {
                continue;
            };
            if open != column.link.as_ref() {
                if open.is_some() {
                    out.push_str(link::CLOSE);
                }
                if let Some(link) = &column.link {
                    out.push_str(&link.open_sequence());
                }
                open = column.link.as_ref();
            }
            if column.style != style {
                push_style_change(&mut out, style, column.style);
                style = column.style;
            }
            glyph.push_to(&mut out);
        }
        if open.is_some() {
            out.push_str(link::CLOSE);
        }
        out
    }
}

/// Writes the SGR sequence that changes text in style `from` to style `to`: a reset alone back
/// to no style, the parameters of `to` alone from no style, and a reset followed by them
/// otherwise.
fn push_style_change(out: &mut String, from: Style, to: Style) {
    let mut params: Vec<String> = Vec::new();
    if from != Style::default() {
        params.push("0".into());
    }
    for (flag, code) in [
        (Attrs::BOLD, "1"),
        (Attrs::DIM, "2"),
        (Attrs::ITALIC, "3"),
        (Attrs::UNDERLINE, "4"),
        (Attrs::BLINK, "5"),
        (Attrs::REVERSE, "7"),
        (Attrs::HIDDEN, "8"),
        (Attrs::STRIKETHROUGH, "9"),
    ] {
        if to.attrs.contains(flag) {
            params.push(code.into());
        }
    }
    params.extend(term_color(to.fg).map(|color| color.sgr(false)));
    params.extend(term_color(to.bg).map(|color| color.sgr(true)));
    out.push_str("\u{1b}[");
    out.push_str(&params.join(";"));
    out.push('m');
}

/// The color an SGR sequence selects for `color`; `None` for [`Color::Reset`], no color.
fn term_color(color: Color) -> Option<TermColor> {
    let index = match color {
        Color::Reset => return None,
        Color::AnsiValue(n) => n,
        Color::Rgb { r, g, b } => return Some(TermColor::Rgb(r, g, b)),
        Color::Black => 0,
        Color::DarkRed => 1,
        Color::DarkGreen => 2,
        Color::DarkYellow => 3,
        Color::DarkBlue => 4,
        Color::DarkMagenta => 5,
        Color::DarkCyan => 6,
        Color::Grey => 7,
        Color::DarkGrey => 8,
        Color::Red => 9,
        Color::Green => 10,
        Color::Yellow => 11,
        Color::Blue => 12,
        Color::Magenta => 13,
        Color::Cyan => 14,
        Color::White => 15,
    };
    Some(TermColor::Indexed(index))
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
            '\t' => line.move_to(line.cursor + 8 - line.cursor % 8),
            '\u{1b}' => match read_escape(&mut chars) {
                Escape::Csi {
                    params,
                    final_char: 'm',
                } => apply_sgr(&params, &mut line.style, Style::default()),
                Escape::Osc(body) => match link::parse_osc(&body) {
                    Some(LinkChange::Open(opened)) => line.link = Some(opened),
                    Some(LinkChange::Close) => line.link = None,
                    None => {}
                },
                Escape::Csi { .. } | Escape::Other => {}
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
    use crate::render::screen::{Attrs, Color, Style};
    use crate::render::styled::styled_glyphs;

    /// The text of `line` once sanitized, cut where its style changes: each piece with the style
    /// it is drawn in on a box whose own style is the default.
    fn styled_runs(line: &str) -> Vec<(String, Style)> {
        let mut runs: Vec<(String, Style)> = Vec::new();
        for glyph in styled_glyphs(&sanitize_line(line), Style::default()) {
            match runs.last_mut() {
                Some((text, style)) if *style == glyph.style => glyph.glyph.push_to(text),
                _ => {
                    let mut text = String::new();
                    glyph.glyph.push_to(&mut text);
                    runs.push((text, glyph.style));
                }
            }
        }
        runs
    }

    fn fg(fg: Color) -> Style {
        Style::new(fg, Color::Reset)
    }

    fn bg(bg: Color) -> Style {
        Style::new(Color::Reset, bg)
    }

    fn bold() -> Style {
        let mut style = Style::default();
        style.attrs.insert(Attrs::BOLD);
        style
    }

    #[test]
    fn overwriting_after_a_carriage_return_keeps_the_color_of_each_column_it_does_not_write() {
        assert_eq!(sanitize_line("\u{1b}[31mabc\u{1b}[0m\rX"), "X\u{1b}[31mbc");
        assert_eq!(
            styled_runs("\u{1b}[31mabc\u{1b}[0m\rX"),
            vec![
                ("X".into(), Style::default()),
                ("bc".into(), fg(Color::AnsiValue(1)))
            ]
        );
    }

    #[test]
    fn overwriting_after_a_carriage_return_keeps_the_background_of_each_column_it_does_not_write() {
        assert_eq!(
            styled_runs("\u{1b}[44mabc\u{1b}[0m\rX"),
            vec![
                ("X".into(), Style::default()),
                ("bc".into(), bg(Color::AnsiValue(4)))
            ]
        );
    }

    #[test]
    fn overwriting_after_a_carriage_return_keeps_the_bold_of_each_column_it_does_not_write() {
        assert_eq!(
            styled_runs("\u{1b}[1mabc\u{1b}[0m\rX"),
            vec![("X".into(), Style::default()), ("bc".into(), bold())]
        );
    }

    #[test]
    fn overwriting_text_is_in_the_style_in_force_as_each_column_is_written() {
        let red = Color::AnsiValue(1);
        assert_eq!(
            sanitize_line("abcd\r\u{1b}[31mX\u{1b}[42mY\u{1b}[0m"),
            "\u{1b}[31mX\u{1b}[0;31;42mY\u{1b}[0mcd"
        );
        assert_eq!(
            styled_runs("abcd\r\u{1b}[31mX\u{1b}[0mY"),
            vec![("X".into(), fg(red)), ("Ycd".into(), Style::default())]
        );
        // Backspace steps back over one column, and overwrites that one alone.
        assert_eq!(
            styled_runs("\u{1b}[31mab\u{1b}[0m\u{8}Z"),
            vec![("a".into(), fg(red)), ("Z".into(), Style::default())]
        );
    }

    #[test]
    fn a_tab_moves_the_cursor_over_columns_without_writing_them() {
        let red = Color::AnsiValue(1);
        assert_eq!(
            styled_runs("\u{1b}[31mabcdefghij\u{1b}[0m\r\tX"),
            vec![
                ("abcdefgh".into(), fg(red)),
                ("X".into(), Style::default()),
                ("j".into(), fg(red))
            ]
        );
        // Columns a tab moves over past the end of the line are blank, in no style.
        assert_eq!(
            styled_runs("a\u{1b}[41m\tb"),
            vec![
                ("a       ".into(), Style::default()),
                ("b".into(), bg(Color::AnsiValue(1)))
            ]
        );
    }

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
    fn hyperlinks_are_kept_in_one_form_and_malformed_ones_dropped() {
        // BEL or ST, with or without an id: written back with ST and an id.
        let line = sanitize_line(
            "see \u{1b}]8;;https://example.com\u{7}the docs\u{1b}]8;;\u{7} \u{1b}]8;id=x;https://e.com\u{1b}\\y\u{1b}]8;;\u{1b}\\",
        );
        let implicit = Link::new("https://example.com", None);
        assert_eq!(
            line,
            format!(
                "see {}the docs{} \x1b]8;id=x;https://e.com\x1b\\y{}",
                implicit.open_sequence(),
                link::CLOSE,
                link::CLOSE
            )
        );
        // Malformed, cut off by the end of the line, or by another escape: dropped, and the
        // escape that cut it off still counts.
        assert_eq!(sanitize_line("a\u{1b}]8;oops;https://e.com\u{1b}\\b"), "ab");
        assert_eq!(sanitize_line("a\u{1b}]8;;https://e.com b"), "a");
        assert_eq!(
            sanitize_line("a\u{1b}]8;;https://e.com\u{1b}[31mb"),
            "a\u{1b}[31mb"
        );
    }

    #[test]
    fn each_column_keeps_the_link_it_was_written_in() {
        let open = Link::new("https://e.com", Some("x")).open_sequence();
        // Overwritten after a carriage return outside the link: only the rest stays a link.
        assert_eq!(
            sanitize_line(&format!("{}abc{}\rX", open, link::CLOSE)),
            format!("X{}bc{}", open, link::CLOSE)
        );
        // A wide cluster's link covers it whole; a tab's blanks are no link.
        assert_eq!(
            sanitize_line(&format!("{}日\tz", open)),
            format!("{}日{}      {}z{}", open, link::CLOSE, open, link::CLOSE)
        );
    }

    #[test]
    fn escapes_with_intermediate_characters_are_dropped_whole() {
        assert_eq!(sanitize_line("a\u{1b}(Bb\u{1b}#8c\u{1b}7d"), "abcd");
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
