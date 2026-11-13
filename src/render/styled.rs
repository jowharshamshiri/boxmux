//! Text with SGR color sequences and OSC 8 hyperlinks turned into styled grapheme clusters.
//!
//! Lines in streams keep the SGR sequences (`ESC [ ... m`) scripts print and their OSC 8
//! hyperlinks (see [`crate::link`]); every other escape sequence was removed when the output was
//! stored (see [`crate::text`]). Colors apply on top of the box's own style: resetting (`0`,
//! `39`, `49`) returns to the box colors, not to the terminal's default, so colored output never
//! leaves holes in a box's background. A hyperlink is kept with each cluster of its text; a
//! color reset does not end it.

use super::screen::{Attrs, Color, Style};
use crate::link::{self, Link, LinkChange};
use crate::text::{read_escape, Escape};
use crate::width::{clusters, width, Glyph};
use std::sync::Arc;

/// A grapheme cluster, the style it is drawn in and the hyperlink it is part of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyledGlyph {
    pub glyph: Glyph,
    pub style: Style,
    pub link: Option<Arc<Link>>,
}

impl StyledGlyph {
    /// The single character `ch` in `style`, part of no link.
    pub const fn from_char(ch: char, style: Style) -> StyledGlyph {
        StyledGlyph {
            glyph: Glyph::from_char(ch),
            style,
            link: None,
        }
    }

    /// The single character `ch` standing in for `self`: in its style and part of its link.
    pub fn replaced_by(&self, ch: char) -> StyledGlyph {
        StyledGlyph {
            glyph: Glyph::from_char(ch),
            style: self.style,
            link: self.link.clone(),
        }
    }

    pub fn width(&self) -> usize {
        self.glyph.width()
    }
}

/// The grapheme clusters of `line` with the style each is drawn in, starting from `base`, and
/// the hyperlink each is part of, starting from none.
/// A cluster that takes no column of its own (a combining mark or selector a color change cut
/// off from its character) joins the cluster before it; with none before it, or when it is a
/// control character, it is dropped: the screen has no place for it. Every escape sequence other
/// than SGR and a well-formed OSC 8 is dropped, and so is an OSC sequence that the end of the
/// line or another escape cuts off before its terminator, with what it held.
pub fn styled_glyphs(line: &str, base: Style) -> Vec<StyledGlyph> {
    let mut out = Vec::with_capacity(line.len());
    let mut style = base;
    let mut link: Option<Arc<Link>> = None;
    // The text since the last escape sequence, all in `style` and `link`.
    let mut run = String::new();
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '\x1b' {
            run.push(ch);
            continue;
        }
        push_run(&mut out, &run, style, &link);
        run.clear();
        match read_escape(&mut chars) {
            Escape::Csi {
                params,
                final_char: 'm',
            } => apply_sgr(&params, &mut style, base),
            Escape::Osc(body) => match link::parse_osc(&body) {
                Some(LinkChange::Open(opened)) => link = Some(opened),
                Some(LinkChange::Close) => link = None,
                None => {}
            },
            Escape::Csi { .. } | Escape::Other => {}
        }
    }
    push_run(&mut out, &run, style, &link);
    out
}

fn push_run(out: &mut Vec<StyledGlyph>, run: &str, style: Style, link: &Option<Arc<Link>>) {
    for cluster in clusters(run) {
        if width(cluster) > 0 {
            out.push(StyledGlyph {
                glyph: Glyph::new(cluster),
                style,
                link: link.clone(),
            });
        } else if !cluster.starts_with(char::is_control) {
            if let Some(last) = out.last_mut() {
                last.glyph = last.glyph.joined(cluster);
            }
        }
    }
}

/// Columns the styled clusters take.
pub fn width_of(glyphs: &[StyledGlyph]) -> usize {
    glyphs.iter().map(StyledGlyph::width).sum()
}

/// Applies one SGR parameter list (the text between `ESC [` and `m`) to `style`.
pub fn apply_sgr(params: &str, style: &mut Style, base: Style) {
    // Both `;` and `:` separate parameters (`38:5:196` is the ITU form of `38;5;196`).
    let values: Vec<Option<u16>> = if params.is_empty() {
        vec![Some(0)]
    } else {
        params
            .split([';', ':'])
            .map(|p| {
                if p.is_empty() {
                    Some(0)
                } else {
                    p.parse().ok()
                }
            })
            .collect()
    };
    let mut i = 0;
    while i < values.len() {
        let Some(code) = values[i] else {
            // An unparsable parameter makes the rest of the sequence meaningless.
            return;
        };
        match code {
            0 => *style = base,
            1 => style.attrs.insert(Attrs::BOLD),
            2 => style.attrs.insert(Attrs::DIM),
            3 => style.attrs.insert(Attrs::ITALIC),
            4 => style.attrs.insert(Attrs::UNDERLINE),
            5 | 6 => style.attrs.insert(Attrs::BLINK),
            7 => style.attrs.insert(Attrs::REVERSE),
            8 => style.attrs.insert(Attrs::HIDDEN),
            9 => style.attrs.insert(Attrs::STRIKETHROUGH),
            21 => style.attrs.insert(Attrs::UNDERLINE),
            22 => {
                style.attrs.remove(Attrs::BOLD);
                style.attrs.remove(Attrs::DIM);
            }
            23 => style.attrs.remove(Attrs::ITALIC),
            24 => style.attrs.remove(Attrs::UNDERLINE),
            25 => style.attrs.remove(Attrs::BLINK),
            27 => style.attrs.remove(Attrs::REVERSE),
            28 => style.attrs.remove(Attrs::HIDDEN),
            29 => style.attrs.remove(Attrs::STRIKETHROUGH),
            30..=37 => style.fg = Color::AnsiValue((code - 30) as u8),
            39 => style.fg = base.fg,
            40..=47 => style.bg = Color::AnsiValue((code - 40) as u8),
            49 => style.bg = base.bg,
            90..=97 => style.fg = Color::AnsiValue((code - 90 + 8) as u8),
            100..=107 => style.bg = Color::AnsiValue((code - 100 + 8) as u8),
            38 | 48 => {
                let (color, used) = extended_color(&values[i + 1..]);
                match color {
                    Some(color) if code == 38 => style.fg = color,
                    Some(color) => style.bg = color,
                    None => return,
                }
                i += used;
            }
            _ => {}
        }
        i += 1;
    }
}

/// Parses the color after `38` or `48`: `5;N` (256 colors) or `2;R;G;B` (truecolor).
/// Returns the color and how many parameters it used.
fn extended_color(rest: &[Option<u16>]) -> (Option<Color>, usize) {
    let byte = |i: usize| {
        rest.get(i)
            .copied()
            .flatten()
            .and_then(|v| u8::try_from(v).ok())
    };
    match rest.first().copied().flatten() {
        Some(5) => (byte(1).map(Color::AnsiValue), 2),
        Some(2) => match (byte(1), byte(2), byte(3)) {
            (Some(r), Some(g), Some(b)) => (Some(Color::Rgb { r, g, b }), 4),
            _ => (None, 0),
        },
        _ => (None, 0),
    }
}

/// Splits `chars` into rows of at most `width` columns, breaking after spaces where possible
/// and inside words that are longer than a row.
pub fn wrap(chars: &[StyledGlyph], width: usize) -> Vec<Vec<StyledGlyph>> {
    if width == 0 {
        return Vec::new();
    }
    let mut rows: Vec<Vec<StyledGlyph>> = Vec::new();
    let mut row: Vec<StyledGlyph> = Vec::new();
    let mut row_width = 0;
    // Index in `row` just after the last space, where the row can be broken.
    let mut break_at: Option<usize> = None;
    for c in chars {
        let w = c.width();
        if row_width + w > width && !row.is_empty() {
            if c.glyph.is(' ') {
                // The row ends exactly here; the space is not carried over.
                rows.push(std::mem::take(&mut row));
                row_width = 0;
                break_at = None;
                continue;
            }
            match break_at.filter(|&at| at < row.len()) {
                Some(at) => {
                    let rest = row.split_off(at);
                    while row.last().is_some_and(|c| c.glyph.is(' ')) {
                        row.pop();
                    }
                    rows.push(std::mem::take(&mut row));
                    row = rest;
                }
                None => rows.push(std::mem::take(&mut row)),
            }
            row_width = width_of(&row);
            break_at = None;
        }
        if c.glyph.is(' ') && row.is_empty() && !rows.is_empty() {
            continue;
        }
        row.push(c.clone());
        row_width += w;
        if c.glyph.is(' ') {
            break_at = Some(row.len());
        }
    }
    if !row.is_empty() || rows.is_empty() {
        rows.push(row);
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: Style = Style::new(Color::AnsiValue(7), Color::AnsiValue(4));

    fn text(chars: &[StyledGlyph]) -> String {
        chars.iter().map(|c| c.glyph.to_string()).collect()
    }

    #[test]
    fn colors_apply_until_reset_and_reset_returns_to_the_box_colors() {
        let chars = styled_glyphs("a\x1b[31mb\x1b[0mc", BASE);
        assert_eq!(text(&chars), "abc");
        assert_eq!(chars[0].style, BASE);
        assert_eq!(chars[1].style.fg, Color::AnsiValue(1));
        assert_eq!(
            chars[1].style.bg, BASE.bg,
            "the box background shows through"
        );
        assert_eq!(chars[2].style, BASE);
    }

    #[test]
    fn default_color_codes_mean_the_box_colors() {
        let chars = styled_glyphs("\x1b[32;41mx\x1b[39;49my", BASE);
        assert_eq!(chars[0].style.fg, Color::AnsiValue(2));
        assert_eq!(chars[0].style.bg, Color::AnsiValue(1));
        assert_eq!(chars[1].style, BASE);
    }

    #[test]
    fn extended_colors_consume_their_parameters() {
        // 256-color foreground followed by a plain bold; the 5 and 196 are not codes of
        // their own (5 would be blink).
        let chars = styled_glyphs("\x1b[38;5;196;1mx", BASE);
        assert_eq!(chars[0].style.fg, Color::AnsiValue(196));
        assert!(chars[0].style.attrs.contains(Attrs::BOLD));
        assert!(!chars[0].style.attrs.contains(Attrs::BLINK));

        let chars = styled_glyphs("\x1b[48;2;10;20;30;4mx", BASE);
        assert_eq!(
            chars[0].style.bg,
            Color::Rgb {
                r: 10,
                g: 20,
                b: 30
            }
        );
        assert!(chars[0].style.attrs.contains(Attrs::UNDERLINE));

        let chars = styled_glyphs("\x1b[38:5:33mx", BASE);
        assert_eq!(chars[0].style.fg, Color::AnsiValue(33));
    }

    #[test]
    fn bright_colors_differ_from_base_colors() {
        let chars = styled_glyphs("\x1b[31ma\x1b[91mb\x1b[101mc", BASE);
        assert_eq!(chars[0].style.fg, Color::AnsiValue(1));
        assert_eq!(chars[1].style.fg, Color::AnsiValue(9));
        assert_eq!(chars[2].style.bg, Color::AnsiValue(9));
    }

    #[test]
    fn attributes_turn_off_individually() {
        let chars = styled_glyphs("\x1b[1;4mA\x1b[22mB\x1b[24mC", BASE);
        assert!(chars[0].style.attrs.contains(Attrs::BOLD));
        assert!(!chars[1].style.attrs.contains(Attrs::BOLD));
        assert!(chars[1].style.attrs.contains(Attrs::UNDERLINE));
        assert!(chars[2].style.attrs.is_empty());
    }

    #[test]
    fn non_color_sequences_are_dropped() {
        assert_eq!(text(&styled_glyphs("a\x1b[2Kb\x1b[1;1Hc", BASE)), "abc");
    }

    #[test]
    fn wrapping_breaks_at_spaces_and_inside_long_words() {
        let rows = wrap(&styled_glyphs("one two three", BASE), 7);
        let rows: Vec<String> = rows.iter().map(|r| text(r)).collect();
        assert_eq!(rows, vec!["one two", "three"]);

        let rows = wrap(&styled_glyphs("abcdefghij", BASE), 4);
        let rows: Vec<String> = rows.iter().map(|r| text(r)).collect();
        assert_eq!(rows, vec!["abcd", "efgh", "ij"]);
    }

    #[test]
    fn wrapping_counts_wide_characters_as_two_columns() {
        let rows = wrap(&styled_glyphs("日本語です", BASE), 4);
        let rows: Vec<String> = rows.iter().map(|r| text(r)).collect();
        assert_eq!(rows, vec!["日本", "語で", "す"]);
        for row in ["日本", "語で"] {
            assert_eq!(width_of(&styled_glyphs(row, BASE)), 4);
        }
    }

    #[test]
    fn wrapping_an_empty_line_keeps_one_empty_row() {
        assert_eq!(wrap(&[], 5).len(), 1);
    }

    #[test]
    fn clusters_stay_whole_and_a_color_change_does_not_split_one() {
        let glyphs = styled_glyphs("e\u{301}☀\u{fe0f}\x1b[31m\u{301}x", BASE);
        assert_eq!(text(&glyphs), "e\u{301}☀\u{fe0f}\u{301}x");
        assert_eq!(glyphs.len(), 3);
        assert_eq!(glyphs[1].glyph.to_string(), "☀\u{fe0f}\u{301}");
        assert_eq!(width_of(&glyphs), 4);
    }

    const DOCS: &str = "\x1b]8;id=docs;https://example.com/docs\x1b\\";

    fn uris(glyphs: &[StyledGlyph]) -> Vec<Option<&str>> {
        glyphs
            .iter()
            .map(|g| g.link.as_deref().map(Link::uri))
            .collect()
    }

    #[test]
    fn a_link_is_kept_with_each_cluster_until_it_is_closed_and_a_color_reset_does_not_end_it() {
        let docs = Some("https://example.com/docs");
        let glyphs = styled_glyphs(
            &format!("a{}b\x1b[31m日\x1b[0mc{}d", DOCS, link::CLOSE),
            BASE,
        );
        assert_eq!(text(&glyphs), "ab日cd");
        assert_eq!(uris(&glyphs), vec![None, docs, docs, docs, None]);
        assert_eq!(glyphs[1].link.as_ref().unwrap().id(), "docs");
        assert_eq!(glyphs[2].style.fg, Color::AnsiValue(1));
        // BEL ends an OSC as ST does.
        let glyphs = styled_glyphs("\x1b]8;;https://example.com\x07x\x1b]8;;\x07y", BASE);
        assert_eq!(uris(&glyphs), vec![Some("https://example.com"), None]);
    }

    #[test]
    fn malformed_and_cut_off_osc_sequences_are_dropped_and_never_shown() {
        // A parameter that is not key=value: no link, no text.
        let glyphs = styled_glyphs("a\x1b]8;oops;https://example.com\x1b\\b", BASE);
        assert_eq!(text(&glyphs), "ab");
        assert_eq!(uris(&glyphs), vec![None, None]);
        // A malformed sequence leaves the link that is open as it was.
        let glyphs = styled_glyphs(&format!("{}a\x1b]8;;bad\x7f\x1b\\b", DOCS), BASE);
        assert!(glyphs.iter().all(|g| g.link.is_some()));
        // Cut off by the end of the line: nothing after it is text.
        assert_eq!(
            text(&styled_glyphs("a\x1b]8;;https://example.com b", BASE)),
            "a"
        );
        // Cut off by another escape, which takes effect.
        let glyphs = styled_glyphs("a\x1b]8;;https://example.com\x1b[31mb", BASE);
        assert_eq!(text(&glyphs), "ab");
        assert_eq!(glyphs[1].style.fg, Color::AnsiValue(1));
        assert_eq!(uris(&glyphs), vec![None, None]);
        // Other OSC commands and two-character escapes are dropped whole.
        assert_eq!(
            text(&styled_glyphs("a\x1b]0;title\x07b\x1b(Bc", BASE)),
            "abc"
        );
    }

    #[test]
    fn a_wrapped_link_is_the_same_link_on_each_row() {
        let rows = wrap(
            &styled_glyphs(
                &format!("see {}the 日本 manual{} now", DOCS, link::CLOSE),
                BASE,
            ),
            10,
        );
        let rows_text: Vec<String> = rows.iter().map(|r| text(r)).collect();
        assert_eq!(rows_text, vec!["see the", "日本", "manual now"]);
        let docs = Some("https://example.com/docs");
        assert_eq!(
            uris(&rows[0]),
            vec![None, None, None, None, docs, docs, docs]
        );
        assert_eq!(uris(&rows[1]), vec![docs, docs]);
        assert_eq!(
            &uris(&rows[2])[..7],
            &[docs, docs, docs, docs, docs, docs, None]
        );
        assert_eq!(rows[0][4].link, rows[2][0].link);
    }

    #[test]
    fn wrapping_never_splits_a_cluster() {
        let rows = wrap(&styled_glyphs("ab👍🏽cd", BASE), 3);
        let rows: Vec<String> = rows.iter().map(|r| text(r)).collect();
        assert_eq!(rows, vec!["ab", "👍🏽c", "d"]);
    }
}
