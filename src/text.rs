//! Text cleanup for script output shown in a box.
//!
//! Script output is a stream of bytes meant for a terminal. A box shows it as lines, so each
//! line is reduced to what a terminal would leave on screen for that line: carriage returns go
//! back to the start of the line and later characters overwrite earlier ones (progress bars),
//! backspace steps back, tabs advance to the next 8-column stop, SGR color sequences are kept,
//! and every other control character or escape sequence is dropped.

use unicode_width::UnicodeWidthChar;

/// One cell of a line being rebuilt: the SGR sequences that precede it, then its character.
#[derive(Clone)]
struct Cell {
    styles: String,
    ch: char,
}

/// Reduces one line of terminal output (without its newline) to displayable text.
pub fn sanitize_line(line: &str) -> String {
    let mut cells: Vec<Cell> = Vec::new();
    let mut cursor = 0usize;
    let mut pending_styles = String::new();
    let mut chars = line.chars().peekable();

    let put = |cells: &mut Vec<Cell>, cursor: &mut usize, styles: &mut String, ch: char| {
        let cell = Cell {
            styles: std::mem::take(styles),
            ch,
        };
        if *cursor < cells.len() {
            // Overwriting keeps styles attached to the old cell so earlier color changes still
            // take effect.
            let previous = std::mem::replace(&mut cells[*cursor], cell);
            cells[*cursor].styles.insert_str(0, &previous.styles);
        } else {
            while cells.len() < *cursor {
                cells.push(Cell {
                    styles: String::new(),
                    ch: ' ',
                });
            }
            cells.push(cell);
        }
        *cursor += 1;
    };

    while let Some(ch) = chars.next() {
        match ch {
            '\r' => cursor = 0,
            '\u{8}' => cursor = cursor.saturating_sub(1),
            '\t' => {
                let column: usize = cells
                    .iter()
                    .take(cursor)
                    .map(|cell| cell.ch.width().unwrap_or(0))
                    .sum();
                let spaces = 8 - column % 8;
                for _ in 0..spaces {
                    put(&mut cells, &mut cursor, &mut pending_styles, ' ');
                }
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
                        pending_styles.push_str(&sequence);
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
            c if c.is_control() => {}
            c => put(&mut cells, &mut cursor, &mut pending_styles, c),
        }
    }

    let mut out = String::with_capacity(line.len());
    for cell in &cells {
        out.push_str(&cell.styles);
        out.push(cell.ch);
    }
    out.push_str(&pending_styles);
    out
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
}
