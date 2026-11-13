use regex::Regex;

/// Columns `text` takes in a terminal, as [`crate::width`] measures them: CSI escape sequences
/// (colors, cursor movement) take none.
pub fn display_width(text: &str) -> usize {
    if text.contains('\x1b') {
        crate::width::text_width(&strip_ansi_codes(text))
    } else {
        crate::width::text_width(text)
    }
}

/// Removes CSI escape sequences (colors, cursor movement) from text.
pub fn strip_ansi_codes(input: &str) -> String {
    static CSI: std::sync::LazyLock<Regex> = std::sync::LazyLock::new(|| {
        Regex::new(r"\x1B\[[0-?]*[ -/]*[@-~]").expect("the CSI pattern is valid")
    });
    let re = &*CSI;
    re.replace_all(input, "").to_string()
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
    fn wide_characters_take_two_cells() {
        assert_eq!(display_width("日本"), 4);
        assert_eq!(display_width("ab"), 2);
        assert_eq!(display_width("é"), 1);
    }
}
