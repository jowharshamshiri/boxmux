//! A modal window over the layout that asks a question and offers answers as buttons: a
//! choice's confirmation.
//!
//! The window is centered on the screen, as wide as its text and buttons need up to
//! [`MAX_WIDTH`] columns, and wraps its text to that width. The buttons sit in a row under the
//! text, aligned right; buttons that do not fit one row continue on the next. When the screen is
//! too short for all of the text, the text is cut and its last row ends with `…`; the buttons are
//! always shown.

use super::screen::{ScreenBuffer, Style};
use super::styled::{styled_glyphs, wrap, StyledGlyph};
use super::theme;
use crate::model::bounds::Bounds;
use crate::width::{ellipsize, text_width};

/// The widest the window gets, borders included.
pub const MAX_WIDTH: usize = 64;
/// Columns between the border and the text, on each side.
const PADDING: usize = 2;
/// Columns between two buttons.
const BUTTON_GAP: usize = 2;

/// What a dialog shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dialog {
    /// Shown in the top border.
    pub title: String,
    /// The question; may hold several lines and SGR colors.
    pub text: String,
    /// The buttons' labels, in order.
    pub buttons: Vec<String>,
}

/// What is at a cell of the screen while a dialog is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogHit {
    /// Button `index`.
    Button(usize),
    /// The window, beside its buttons.
    Window,
    /// Outside the window.
    Outside,
}

/// A dialog laid out on a screen of a given size: where its window, text and buttons are.
#[derive(Debug, Clone)]
pub struct DialogView {
    pub outer: Bounds,
    title: String,
    /// The text's rows as drawn, each starting at the text's left edge.
    text: Vec<Vec<StyledGlyph>>,
    text_top: usize,
    text_left: usize,
    /// Each button's label as drawn (cut to fit) and the row and first column of the button.
    buttons: Vec<(String, usize, usize)>,
}

impl DialogView {
    /// Lays `dialog` out on a `width` x `height` screen. `None` when the screen has no room for
    /// a window: fewer than 5 columns or 3 rows plus a row per row of buttons.
    pub fn new(dialog: &Dialog, width: usize, height: usize) -> Option<DialogView> {
        let palette = theme::palette();
        let outer_limit = width.min(MAX_WIDTH);
        if outer_limit < 5 {
            return None;
        }
        // The widest the text and the buttons may be inside the borders and the padding.
        let inner_limit = outer_limit.saturating_sub(2 + 2 * PADDING).max(1);
        let labels: Vec<String> = dialog
            .buttons
            .iter()
            .map(|label| {
                let label = crate::utils::strip_ansi_codes(label);
                // A button is its label with a space on each side.
                ellipsize(&label, inner_limit.saturating_sub(2).max(1))
            })
            .collect();
        let paragraphs: Vec<Vec<StyledGlyph>> = crate::text::sanitize_output(&dialog.text)
            .iter()
            .map(|line| styled_glyphs(line, palette.dialog))
            .collect();
        let natural_text = paragraphs
            .iter()
            .map(|line| super::styled::width_of(line))
            .max()
            .unwrap_or(0);
        let natural_buttons = labels
            .iter()
            .map(|label| text_width(label) + 2)
            .sum::<usize>()
            + BUTTON_GAP * labels.len().saturating_sub(1);
        let title = crate::utils::strip_ansi_codes(&dialog.title);
        let natural_title = text_width(&title) + 2;
        let inner = natural_text
            .max(natural_buttons)
            .max(natural_title.saturating_sub(2 * PADDING))
            .clamp(1, inner_limit);

        let button_rows = button_rows(&labels, inner);
        let mut text: Vec<Vec<StyledGlyph>> = paragraphs
            .iter()
            .flat_map(|line| wrap(line, inner))
            .collect();
        // Rows: the borders, the buttons, and as room allows a blank row above the text, a blank
        // row between the text and the buttons, and the text.
        let fixed = 2 + button_rows.len();
        if height < fixed + 1 {
            return None;
        }
        let spare = height - fixed;
        let (blank_above, blank_between, text_room) = if spare >= text.len() + 2 {
            (1, 1, text.len())
        } else if spare >= 3 {
            (1, 1, spare - 2)
        } else {
            (0, 0, spare)
        };
        if text.len() > text_room {
            text.truncate(text_room);
            if let Some(last) = text.last_mut() {
                end_with_ellipsis(last, inner, palette.dialog);
            }
        }
        let outer_width = inner + 2 + 2 * PADDING;
        let outer_height = fixed + blank_above + text.len() + blank_between;
        let left = (width - outer_width) / 2;
        let top = (height - outer_height) / 2;
        let outer = Bounds::new(left, top, left + outer_width - 1, top + outer_height - 1);
        let text_left = left + 1 + PADDING;
        let text_top = top + 1 + blank_above;
        let buttons_top = text_top + text.len() + blank_between;
        let right_edge = text_left + inner;
        let mut buttons = Vec::new();
        for (row_index, row) in button_rows.iter().enumerate() {
            let row_width = row
                .iter()
                .map(|&i| text_width(&labels[i]) + 2)
                .sum::<usize>()
                + BUTTON_GAP * row.len().saturating_sub(1);
            let mut x = right_edge - row_width;
            for &i in row {
                buttons.push((labels[i].clone(), buttons_top + row_index, x));
                x += text_width(&labels[i]) + 2 + BUTTON_GAP;
            }
        }
        Some(DialogView {
            outer,
            title,
            text,
            text_top,
            text_left,
            buttons,
        })
    }

    /// The cells button `index` takes on its row: its first and last column.
    fn button_span(&self, index: usize) -> Option<(usize, usize, usize)> {
        let (label, y, x) = self.buttons.get(index)?;
        Some((y.to_owned(), *x, x + text_width(label) + 1))
    }

    /// What is at (`x`, `y`).
    pub fn hit(&self, x: usize, y: usize) -> DialogHit {
        if !self.outer.contains_point(x, y) {
            return DialogHit::Outside;
        }
        (0..self.buttons.len())
            .find(|&i| {
                self.button_span(i)
                    .is_some_and(|(row, first, last)| row == y && (first..=last).contains(&x))
            })
            .map_or(DialogHit::Window, DialogHit::Button)
    }

    /// Draws the window over what `buffer` holds, with button `focused` marked as the one Enter
    /// presses and button `hovered` as the one under the pointer.
    pub fn draw(&self, buffer: &mut ScreenBuffer, focused: usize, hovered: Option<usize>) {
        let palette = theme::palette();
        let o = self.outer;
        let border = Style::new(palette.focused_border, palette.bg);
        let limit = o.right();
        buffer.fill(
            o.left(),
            o.top(),
            o.right(),
            o.bottom(),
            ' ',
            palette.dialog,
        );
        for x in o.left() + 1..o.right() {
            buffer.put_char(x, o.top(), '─', border, limit);
            buffer.put_char(x, o.bottom(), '─', border, limit);
        }
        for y in o.top() + 1..o.bottom() {
            buffer.put_char(o.left(), y, '│', border, limit);
            buffer.put_char(o.right(), y, '│', border, limit);
        }
        buffer.put_char(o.left(), o.top(), '┌', border, limit);
        buffer.put_char(o.right(), o.top(), '┐', border, limit);
        buffer.put_char(o.left(), o.bottom(), '└', border, limit);
        buffer.put_char(o.right(), o.bottom(), '┘', border, limit);
        // The title sits in the top border, as a box's does.
        let room = o.width().saturating_sub(4);
        if room > 2 && !self.title.is_empty() {
            let title = format!(" {} ", ellipsize(&self.title, room - 2));
            buffer.put_str(o.left() + 2, o.top(), &title, palette.dialog, limit - 1);
        }
        let text_limit = o.right() - 1 - PADDING;
        for (row, glyphs) in self.text.iter().enumerate() {
            let mut x = self.text_left;
            for glyph in glyphs {
                x += buffer.put(
                    x,
                    self.text_top + row,
                    &glyph.glyph,
                    glyph.style,
                    glyph.link.as_ref(),
                    text_limit,
                );
            }
        }
        for (index, (label, y, x)) in self.buttons.iter().enumerate() {
            let style = if hovered == Some(index) {
                palette.menu_hovered
            } else if index == focused {
                palette.dialog_button_focused
            } else {
                palette.dialog_button
            };
            buffer.put_str(*x, *y, &format!(" {} ", label), style, text_limit);
        }
    }
}

/// The buttons on each row: as many on a row as fit in `inner` columns, in order.
fn button_rows(labels: &[String], inner: usize) -> Vec<Vec<usize>> {
    let mut rows: Vec<Vec<usize>> = Vec::new();
    let mut used = 0;
    for (i, label) in labels.iter().enumerate() {
        let width = text_width(label) + 2;
        match rows.last_mut() {
            Some(row) if used + BUTTON_GAP + width <= inner => {
                row.push(i);
                used += BUTTON_GAP + width;
            }
            _ => {
                rows.push(vec![i]);
                used = width;
            }
        }
    }
    rows
}

/// Ends `row` with `…`, cutting it so that it stays within `width` columns.
fn end_with_ellipsis(row: &mut Vec<StyledGlyph>, width: usize, style: Style) {
    let ellipsis = StyledGlyph::from_char('…', style);
    while !row.is_empty() && super::styled::width_of(row) + 1 > width {
        row.pop();
    }
    row.push(ellipsis);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dialog(text: &str, buttons: &[&str]) -> Dialog {
        Dialog {
            title: "Deploy".into(),
            text: text.into(),
            buttons: buttons.iter().map(|b| b.to_string()).collect(),
        }
    }

    fn drawn(view: &DialogView, width: usize, height: usize) -> Vec<String> {
        let mut buffer = ScreenBuffer::new(width, height, Style::default());
        buffer.fill(0, 0, width - 1, height - 1, '.', Style::default());
        view.draw(&mut buffer, 0, None);
        buffer.text_rows()
    }

    #[test]
    fn a_short_question_is_centered_with_its_buttons_aligned_right() {
        let view = DialogView::new(&dialog("Ship it?", &["Yes", "Cancel"]), 40, 11).unwrap();
        assert_eq!(
            drawn(&view, 40, 11),
            vec![
                "........................................",
                "........................................",
                ".........┌─ Deploy ──────────┐..........",
                ".........│                   │..........",
                ".........│  Ship it?         │..........",
                ".........│                   │..........",
                ".........│   Yes    Cancel   │..........",
                ".........└───────────────────┘..........",
                "........................................",
                "........................................",
                "........................................",
            ]
        );
        assert_eq!(view.hit(12, 6), DialogHit::Button(0));
        assert_eq!(view.hit(16, 6), DialogHit::Button(0));
        assert_eq!(view.hit(17, 6), DialogHit::Window);
        assert_eq!(view.hit(19, 6), DialogHit::Button(1));
        assert_eq!(view.hit(26, 6), DialogHit::Button(1));
        assert_eq!(view.hit(27, 6), DialogHit::Window);
        assert_eq!(view.hit(10, 4), DialogHit::Window);
        assert_eq!(view.hit(8, 4), DialogHit::Outside);
        assert_eq!(view.hit(20, 8), DialogHit::Outside);
    }

    #[test]
    fn long_text_wraps_at_the_widest_the_window_gets() {
        let text = "Restart every service of the production cluster now, dropping the \
                    connections of everyone who is using it at the moment?";
        let view = DialogView::new(&dialog(text, &["Restart", "Cancel"]), 100, 20).unwrap();
        assert_eq!(view.outer.width(), MAX_WIDTH);
        let rows = drawn(&view, 100, 20);
        let text_rows: Vec<String> = rows
            .iter()
            .filter(|row| row.contains("│  ") && !row.contains(" Cancel "))
            .map(|row| row.trim_matches('.').trim_matches('│').trim().to_string())
            .filter(|row| !row.is_empty())
            .collect();
        assert_eq!(
            text_rows,
            vec![
                "Restart every service of the production cluster now,",
                "dropping the connections of everyone who is using it at",
                "the moment?",
            ]
        );
        for row in &rows {
            assert_eq!(crate::width::text_width(row), 100, "{row}");
        }
    }

    #[test]
    fn on_a_short_screen_the_text_is_cut_and_the_buttons_stay() {
        let text = "one\ntwo\nthree\nfour\nfive";
        let view = DialogView::new(&dialog(text, &["Yes", "Cancel"]), 30, 6).unwrap();
        let rows = drawn(&view, 30, 6);
        let joined = rows.join("\n");
        assert!(joined.contains("one"), "{joined}");
        assert!(joined.contains("…"), "{joined}");
        assert!(!joined.contains("five"), "{joined}");
        assert!(joined.contains(" Yes "), "{joined}");
        assert!(joined.contains(" Cancel "), "{joined}");
    }

    #[test]
    fn buttons_that_do_not_fit_one_row_go_on_to_the_next() {
        let view = DialogView::new(
            &dialog(
                "Pick",
                &["Staging environment", "Production environment", "Cancel"],
            ),
            34,
            12,
        )
        .unwrap();
        let rows = drawn(&view, 34, 12);
        let row_of = |label: &str| rows.iter().position(|row| row.contains(label)).unwrap();
        assert!(row_of("Staging") < row_of("Production"), "{rows:#?}");
        assert!(row_of("Production") <= row_of("Cancel"), "{rows:#?}");
        for (i, label) in ["Staging", "Production", "Cancel"].iter().enumerate() {
            let y = row_of(label);
            let x = rows[y].find(label).unwrap();
            let x = rows[y][..x].chars().count();
            assert_eq!(view.hit(x, y), DialogHit::Button(i), "{label}");
        }
    }

    #[test]
    fn a_screen_with_no_room_for_a_window_shows_none() {
        assert!(DialogView::new(&dialog("Sure?", &["Yes"]), 4, 10).is_none());
        assert!(DialogView::new(&dialog("Sure?", &["Yes"]), 40, 3).is_none());
    }
}
