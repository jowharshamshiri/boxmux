use serde::{Deserialize, Deserializer, Serialize};

/// How a script runs.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Hash, Eq, Default)]
pub enum ExecutionMode {
    /// Runs in the background; the output replaces (or is appended to) the stream when the
    /// script finishes.
    #[default]
    Immediate,
    /// Runs in the background; output is streamed into the stream line by line as it arrives.
    Thread,
    /// Runs in a pseudo-terminal shown live in the box; keyboard and mouse input go to it while
    /// the box has focus.
    Pty,
}

impl ExecutionMode {
    pub fn is_pty(&self) -> bool {
        matches!(self, ExecutionMode::Pty)
    }
}

/// Runtime settings that come from the command line rather than the YAML file.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Config {
    /// Minimum time between two frames, in milliseconds. Bounds the redraw rate.
    pub frame_delay: u64,
    /// Disables moving and resizing boxes with the mouse; nothing is written to the YAML file.
    pub locked: bool,
    /// Highlights the cell under the mouse pointer, for checking click coordinates.
    pub calibrate: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            frame_delay: 30,
            locked: false,
            calibrate: false,
        }
    }
}

impl Config {
    pub fn new(frame_delay: u64, locked: bool, calibrate: bool) -> Result<Self, String> {
        if frame_delay == 0 {
            return Err("frame delay must be at least 1 millisecond".to_string());
        }
        Ok(Config {
            frame_delay,
            locked,
            calibrate,
        })
    }
}

/// Deserializes a `script` field. A single string is one script, kept verbatim (indentation,
/// blank lines and heredocs intact). A sequence holds one command per element; an element that
/// YAML parsed as a mapping (`- echo key: value` written without quotes) is turned back into the
/// text the user wrote. Elements are joined with newlines when the script runs.
pub fn deserialize_script<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum ScriptFormat {
        Single(String),
        Sequence(Vec<serde_json::Value>),
    }

    fn element_text<E: Error>(value: serde_json::Value) -> Result<String, E> {
        match value {
            serde_json::Value::String(s) => Ok(s),
            serde_json::Value::Number(n) => Ok(n.to_string()),
            serde_json::Value::Bool(b) => Ok(b.to_string()),
            serde_json::Value::Object(_) => serde_saphyr::to_string(&value)
                .map(|text| text.trim_end().to_string())
                .map_err(|e| E::custom(format!("script element cannot be shown as text: {}", e))),
            serde_json::Value::Null => Err(E::custom("script element is empty")),
            serde_json::Value::Array(_) => Err(E::custom(
                "script elements must be strings, not nested lists",
            )),
        }
    }

    match Option::<ScriptFormat>::deserialize(deserializer)? {
        None => Ok(None),
        Some(ScriptFormat::Single(script)) => Ok(Some(vec![script])),
        Some(ScriptFormat::Sequence(elements)) => elements
            .into_iter()
            .map(element_text::<D::Error>)
            .collect::<Result<Vec<_>, _>>()
            .map(Some),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // === Config Tests ===

    #[test]
    fn config_rejects_a_zero_frame_delay() {
        assert!(Config::new(0, false, false).is_err());
        let config = Config::new(16, true, false).unwrap();
        assert_eq!((config.frame_delay, config.locked), (16, true));
    }

    // === Position Tests ===

    #[test]
    fn script_strings_keep_lines_blank_lines_and_indentation() {
        #[derive(Deserialize)]
        struct Holder {
            #[serde(deserialize_with = "deserialize_script")]
            script: Option<Vec<String>>,
        }
        let holder: Holder =
            crate::yaml::from_str("script: |\n  cat <<EOF\n    indented\n\n  EOF\n").unwrap();
        assert_eq!(
            holder.script.unwrap(),
            vec!["cat <<EOF\n  indented\n\nEOF\n".to_string()]
        );
        let holder: Holder = crate::yaml::from_str("script:\n  - echo a: b\n  - 42\n").unwrap();
        assert_eq!(holder.script.unwrap(), vec!["echo a: b", "42"]);
    }

    // === Bounds Tests ===
}
