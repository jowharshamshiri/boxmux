use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::model::common::{deserialize_script, ExecutionMode};

/// An entry of a box's menu: a label and what activating it does, run a script or copy text to
/// the clipboard.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub content: Option<String>,
    #[serde(deserialize_with = "deserialize_script", default)]
    pub script: Option<Vec<String>>,
    /// Activating the choice copies this text to the clipboard instead of running a script.
    #[serde(default)]
    pub copy: Option<CopyText>,
    pub redirect_output: Option<String>,
    pub append_output: Option<bool>,
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    /// The entry Enter activates. Runtime state.
    #[serde(skip)]
    pub selected: bool,
    /// The choice's script is running. Runtime state.
    #[serde(skip)]
    pub waiting: bool,
}

impl Choice {
    /// The label as the menu shows it: its content, or its id when it has none.
    pub fn label(&self) -> &str {
        self.content.as_deref().unwrap_or(&self.id)
    }

    /// The text activating the choice copies, if it is a copy choice. A copied label is its
    /// visible text: the escape sequences that color it or make it a link are not copied.
    pub fn copied_text(&self) -> Option<String> {
        self.copy.as_ref().map(|copy| match copy {
            CopyText::Label => crate::utils::strip_ansi_codes(self.label()),
            CopyText::Text(text) => text.clone(),
        })
    }
}

/// What a copy choice copies, written `copy: true` (its label) or `copy: <text>`.
#[derive(Debug, Clone, PartialEq)]
pub enum CopyText {
    Label,
    Text(String),
}

impl Serialize for CopyText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            CopyText::Label => serializer.serialize_bool(true),
            CopyText::Text(text) => serializer.serialize_str(text),
        }
    }
}

impl<'de> Deserialize<'de> for CopyText {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;

        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Written {
            Flag(bool),
            Text(String),
        }

        match Written::deserialize(deserializer).map_err(|_| {
            D::Error::custom("copy must be true (copy the label) or the text to copy")
        })? {
            Written::Flag(true) => Ok(CopyText::Label),
            Written::Flag(false) => Err(D::Error::custom(
                "copy: false copies nothing; leave copy out for a choice that does not copy",
            )),
            Written::Text(text) if text.is_empty() => {
                Err(D::Error::custom("copy is empty: there is no text to copy"))
            }
            Written::Text(text) => Ok(CopyText::Text(text)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice(yaml: &str) -> Result<Choice, String> {
        crate::yaml::from_str::<Choice>(yaml).map_err(|e| e.to_string())
    }

    #[test]
    fn copy_true_copies_the_label_as_shown_and_a_string_copies_itself() {
        let label = choice(
            r#"
id: cmd
content: "\e[1mcargo \e]8;;https://x.test\e\\test\e]8;;\e\\ --workspace\e[0m"
copy: true
"#,
        )
        .unwrap();
        assert_eq!(label.copy, Some(CopyText::Label));
        assert_eq!(
            label.copied_text().as_deref(),
            Some("cargo test --workspace")
        );

        let bare = choice("id: just-the-id\ncopy: true\n").unwrap();
        assert_eq!(bare.copied_text().as_deref(), Some("just-the-id"));

        let text =
            choice("id: cmd\ncontent: Copy it\ncopy: \"git log -1 ; echo 日本 \"\n").unwrap();
        assert_eq!(
            text.copied_text().as_deref(),
            Some("git log -1 ; echo 日本 "),
            "the declared text is copied exactly, its spaces kept"
        );

        let script = choice("id: run\nscript: [echo]\n").unwrap();
        assert_eq!(script.copied_text(), None);
    }

    #[test]
    fn copy_false_an_empty_copy_and_other_values_are_refused_naming_copy() {
        for yaml in [
            "id: c\ncopy: false\n",
            "id: c\ncopy: \"\"\n",
            "id: c\ncopy: [a]\n",
        ] {
            let error = choice(yaml).expect_err(yaml);
            assert!(error.contains("copy"), "{}: {}", yaml, error);
        }
    }

    #[test]
    fn a_copy_choice_serializes_back_to_what_was_written() {
        for (yaml, written) in [
            ("id: c\ncopy: true\n", serde_json::json!(true)),
            ("id: c\ncopy: echo hi\n", serde_json::json!("echo hi")),
            ("id: c\n", serde_json::Value::Null),
        ] {
            let parsed = choice(yaml).unwrap();
            let json = serde_json::to_value(&parsed).unwrap();
            assert_eq!(json["copy"], written, "{}", yaml);
            let again: Choice = serde_json::from_value(json).unwrap();
            assert_eq!(again, parsed);
        }
    }
}
