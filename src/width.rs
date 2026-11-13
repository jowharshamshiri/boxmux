//! How many terminal columns text takes, and the cells it is drawn in.
//!
//! BoxMux lays text out in grapheme clusters (Unicode Standard Annex #29, extended grapheme
//! clusters): what a reader takes for one character, such as a letter with its combining
//! accents, an emoji with its variation selector or skin tone, emoji joined by zero-width
//! joiners, or a flag. A cluster is drawn in one cell of the screen and takes the columns
//! [`width`] gives it, by Unicode's East Asian Width (Annex #11) and emoji presentation:
//!
//! - wide and fullwidth characters (CJK) and emoji presented as emoji take two columns: 🌅, 日,
//!   an emoji with the emoji variation selector U+FE0F (☀️), a skin tone or joiner sequence
//!   (👍🏽), a flag;
//! - East Asian *ambiguous* characters take one column, as they do in Western terminal setups
//!   (WezTerm, kitty, xterm, macOS Terminal, Windows Terminal by default): →, ·, ×, …, ►, and
//!   the box-drawing characters BoxMux draws borders with. A terminal set to draw them two
//!   columns wide (a CJK locale, WezTerm's `treat_east_asian_ambiguous_width_as_wide`) does not
//!   match BoxMux's layout;
//! - combining marks, variation selectors and joiners take no column of their own: they belong
//!   to the cluster before them (e + U+0301 is one column, é);
//! - every other character takes one column; control characters take none and are never drawn.
//!
//! A cluster takes at most two columns. Every width and offset BoxMux computes (wrapping,
//! truncation, padding, centering, titles, tabs, menus, tables, charts, the terminal emulator's
//! cursor and what a redraw replaces) is a sum of these widths.
//!
//! Terminals do not all agree on clusters: some draw an emoji variation sequence one column
//! wide (WezTerm's default `unicode_version = 9`), and terminals with old Unicode tables draw
//! some emoji in one column. BoxMux draws each two-column or many-character cluster into
//! cells it blanked first and places the cursor after it itself
//! ([`crate::runtime::display`]), so such a terminal shows the cluster followed by a blank
//! and every column after it where BoxMux put it.

use std::fmt;
use std::sync::Arc;
use unicode_segmentation::{Graphemes, UnicodeSegmentation};
use unicode_width::UnicodeWidthStr;

/// Columns the grapheme cluster `cluster` takes: 0, 1 or 2, by the rules of this module.
pub fn width(cluster: &str) -> usize {
    match cluster.chars().next() {
        None => 0,
        Some(first) if first.is_control() => 0,
        Some(_) => cluster.width().min(2),
    }
}

/// The grapheme clusters of `text`, in order.
pub fn clusters(text: &str) -> Graphemes<'_> {
    text.graphemes(true)
}

/// Columns `text` takes: the sum of its clusters' widths. `text` holds no escape sequences
/// (see [`crate::utils::display_width`] for text that may).
pub fn text_width(text: &str) -> usize {
    clusters(text).map(width).sum()
}

/// The longest start of `text`, in whole clusters, that takes at most `columns` columns.
pub fn cut(text: &str, columns: usize) -> &str {
    let mut used = 0;
    let mut end = 0;
    for (start, cluster) in text.grapheme_indices(true) {
        let w = width(cluster);
        if used + w > columns {
            break;
        }
        used += w;
        end = start + cluster.len();
    }
    &text[..end]
}

/// `text` when it takes at most `columns` columns; otherwise its start cut so that `…` after
/// it fits in `columns`.
pub fn ellipsize(text: &str, columns: usize) -> String {
    if text_width(text) <= columns {
        return text.to_string();
    }
    if columns == 0 {
        return String::new();
    }
    format!("{}…", cut(text, columns - 1))
}

/// `text` cut to at most `columns` columns and padded with spaces to exactly `columns`.
pub fn pad(text: &str, columns: usize) -> String {
    let kept = cut(text, columns);
    format!("{}{}", kept, " ".repeat(columns - text_width(kept)))
}

/// What one cell of the screen shows: a grapheme cluster.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Glyph(Repr);

#[derive(Clone, PartialEq, Eq, Hash)]
enum Repr {
    /// A cluster of one character, which most are.
    One(char),
    Many(Arc<str>),
}

impl Glyph {
    pub const SPACE: Glyph = Glyph::from_char(' ');

    /// The glyph of a single character.
    pub const fn from_char(ch: char) -> Glyph {
        Glyph(Repr::One(ch))
    }

    /// The glyph of `cluster`, one grapheme cluster.
    pub fn new(cluster: &str) -> Glyph {
        let mut chars = cluster.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Glyph::from_char(ch),
            (Some(_), Some(_)) => Glyph(Repr::Many(Arc::from(cluster))),
            (None, _) => panic!("a glyph is made of at least one character"),
        }
    }

    /// Columns the glyph takes; see [`width`].
    pub fn width(&self) -> usize {
        match &self.0 {
            Repr::One(ch) => width(ch.encode_utf8(&mut [0; 4])),
            Repr::Many(cluster) => width(cluster),
        }
    }

    /// Whether the glyph is exactly the character `ch`.
    pub fn is(&self, ch: char) -> bool {
        self.0 == Repr::One(ch)
    }

    /// Whether the glyph is made of more than one character.
    pub fn is_sequence(&self) -> bool {
        matches!(self.0, Repr::Many(_))
    }

    /// This glyph's cluster followed by `more`.
    pub fn joined(&self, more: &str) -> Glyph {
        Glyph::new(&format!("{}{}", self, more))
    }

    /// Whether `more` continues this glyph's cluster rather than starting a new one.
    pub fn continues_with(&self, more: &str) -> bool {
        let joined = format!("{}{}", self, more);
        clusters(&joined).nth(1).is_none()
    }

    pub fn push_to(&self, out: &mut String) {
        match &self.0 {
            Repr::One(ch) => out.push(*ch),
            Repr::Many(cluster) => out.push_str(cluster),
        }
    }
}

impl fmt::Display for Glyph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0 {
            Repr::One(ch) => write!(f, "{}", ch),
            Repr::Many(cluster) => f.write_str(cluster),
        }
    }
}

impl fmt::Debug for Glyph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_characters_and_emoji_take_two_columns() {
        for cluster in [
            "日",
            "🌅",
            "☀\u{fe0f}",
            "👍🏽",
            "👨\u{200d}👩\u{200d}👧",
            "🇩🇪",
        ] {
            assert_eq!(width(cluster), 2, "{:?}", cluster);
        }
    }

    #[test]
    fn ambiguous_characters_take_one_column() {
        for cluster in ["→", "·", "×", "…", "►", "─", "│", "┌"] {
            assert_eq!(width(cluster), 1, "{:?}", cluster);
        }
    }

    #[test]
    fn combining_marks_belong_to_the_character_before_them() {
        let text = "e\u{301}te\u{301}";
        assert_eq!(
            clusters(text).collect::<Vec<_>>(),
            ["e\u{301}", "t", "e\u{301}"]
        );
        assert_eq!(text_width(text), 3);
        assert_eq!(
            width("\u{301}"),
            0,
            "a combining mark alone takes no column"
        );
    }

    #[test]
    fn a_text_presentation_selector_keeps_a_character_narrow() {
        assert_eq!(width("☀\u{fe0e}"), 1);
        assert_eq!(width("☀"), 1);
    }

    #[test]
    fn control_characters_take_no_column() {
        assert_eq!(width("\u{7}"), 0);
        assert_eq!(width("\r\n"), 0);
    }

    #[test]
    fn cutting_keeps_whole_clusters() {
        assert_eq!(cut("日本語", 5), "日本");
        assert_eq!(cut("a👍🏽b", 2), "a");
        assert_eq!(cut("a👍🏽b", 3), "a👍🏽");
        assert_eq!(cut("e\u{301}x", 1), "e\u{301}");
    }

    #[test]
    fn ellipsizing_and_padding_take_exactly_the_columns_given() {
        assert_eq!(ellipsize("日本語です", 6), "日本…");
        assert_eq!(text_width(&ellipsize("日本語です", 6)), 5);
        assert_eq!(ellipsize("short", 6), "short");
        assert_eq!(pad("日本語", 5), "日本 ");
        assert_eq!(pad("→·", 4), "→·  ");
    }

    #[test]
    fn a_glyph_is_its_cluster() {
        let glyph = Glyph::new("e\u{301}");
        assert!(glyph.is_sequence());
        assert_eq!(glyph.width(), 1);
        assert_eq!(glyph.to_string(), "e\u{301}");
        assert!(Glyph::new("x").is('x'));
        assert!(Glyph::from_char('☀').continues_with("\u{fe0f}"));
        assert_eq!(Glyph::from_char('☀').joined("\u{fe0f}").width(), 2);
        assert!(!Glyph::from_char('a').continues_with("b"));
    }
}
