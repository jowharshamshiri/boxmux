use regex::Regex;

/// Columns `text` takes in a terminal, as [`crate::width`] measures them: CSI escape sequences
/// (colors, cursor movement) and OSC sequences (hyperlinks) take none.
pub fn display_width(text: &str) -> usize {
    if text.contains('\x1b') {
        crate::width::text_width(&strip_ansi_codes(text))
    } else {
        crate::width::text_width(text)
    }
}

/// Removes CSI escape sequences (colors, cursor movement) and terminated OSC sequences
/// (hyperlinks, titles), ended by BEL or ST, from text: what is left is the text a terminal
/// shows.
pub fn strip_ansi_codes(input: &str) -> String {
    static ESCAPES: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]|\x1B\][^\x07\x1B]*(?:\x07|\x1B\\)")
            .expect("the escape pattern is valid")
    });
    ESCAPES.replace_all(input, "").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ansi_sequences_are_stripped_and_take_no_width() {
        let colored = "\x1b[1;31merror\x1b[0m: x";
        assert_eq!(strip_ansi_codes(colored), "error: x");
        assert_eq!(display_width(colored), 8);
    }

    #[test]
    fn hyperlinks_are_stripped_and_take_no_width() {
        let linked = "see \x1b]8;id=a;https://example.com\x1b\\the 日本 docs\x1b]8;;\x1b\\!";
        assert_eq!(strip_ansi_codes(linked), "see the 日本 docs!");
        assert_eq!(display_width(linked), 18);
        assert_eq!(
            strip_ansi_codes("\x1b]8;;https://example.com\x07x\x1b]8;;\x07"),
            "x"
        );
    }

    #[test]
    fn wide_characters_take_two_cells() {
        assert_eq!(display_width("日本"), 4);
        assert_eq!(display_width("ab"), 2);
        assert_eq!(display_width("é"), 1);
    }
}
