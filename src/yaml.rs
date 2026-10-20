//! Reading YAML. A configuration is parsed into a JSON value first, where variables are
//! substituted and the schema is checked, and is then deserialized into the model types.

use serde::de::DeserializeOwned;
use serde_saphyr::MessageFormatter;

/// Only `true` and `false` are booleans, so `yes`, `on` and `no` stay text, as in YAML 1.2.
/// A tag boxmux does not know is an error instead of being dropped.
fn options() -> serde_saphyr::Options {
    serde_saphyr::options! {
        strict_booleans: true,
        reject_unsupported_tags: true,
        with_snippet: false,
    }
}

/// Parses YAML text, expanding anchors, aliases and merge keys (`<<`).
pub fn parse(text: &str) -> Result<serde_json::Value, serde_saphyr::Error> {
    serde_saphyr::from_str_with_options(text, options())
}

/// Parses YAML text and deserializes it into `T` the way a configuration is.
pub fn from_str<T: DeserializeOwned>(text: &str) -> Result<T, String> {
    let value = parse(text).map_err(|e| e.to_string())?;
    serde_json::from_value(value).map_err(|e| e.to_string())
}

/// A parse error of `contents` (named `source`), with the offending line and a caret under the
/// column when the error has a location.
pub fn describe_error(error: &serde_saphyr::Error, contents: &str, source: &str) -> String {
    let message = serde_saphyr::UserMessageFormatter.format_message(error);
    let Some(location) = error.location() else {
        return format!("{}: {}", source, message);
    };
    let (line, column) = (location.line() as usize, location.column() as usize);
    let mut text = format!("{}:{}:{}: {}", source, line, column, message);
    if let Some(row) = contents.lines().nth(line.saturating_sub(1)) {
        let number = line.to_string();
        text.push_str(&format!(
            "\n{} | {}\n{} | {}^",
            number,
            row,
            " ".repeat(number.len()),
            " ".repeat(column.saturating_sub(1))
        ));
    }
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn only_true_and_false_are_booleans() {
        assert_eq!(
            parse("a: yes\nb: on\nc: no\nd: true\ne: false\n").unwrap(),
            json!({"a": "yes", "b": "on", "c": "no", "d": true, "e": false})
        );
    }

    #[test]
    fn anchors_and_merge_keys_are_expanded() {
        let value = parse("base: &base {x: 1, y: two}\nbox:\n  <<: *base\n  y: three\n").unwrap();
        assert_eq!(value["box"], json!({"x": 1, "y": "three"}));
    }

    #[test]
    fn unknown_tags_and_duplicate_keys_are_errors() {
        assert!(parse("a: !custom text\n").is_err());
        assert!(parse("a: 1\na: 2\n").is_err());
    }

    #[test]
    fn an_error_shows_the_line_and_points_at_the_column() {
        let contents = "app:\n  layouts: [\n  x: y\n";
        let error = parse(contents).unwrap_err();
        assert_eq!(
            describe_error(&error, contents, "demo.yaml"),
            "demo.yaml:2:12: unclosed bracket '['\n2 |   layouts: [\n  |            ^"
        );
    }
}
