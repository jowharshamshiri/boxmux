use serde::{Deserialize, Serialize};

use crate::model::common::{deserialize_script, ExecutionMode};

/// An entry of a box's menu: a label and the script it runs when activated.
#[derive(Debug, Clone, PartialEq, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub id: String,
    pub content: Option<String>,
    #[serde(deserialize_with = "deserialize_script", default)]
    pub script: Option<Vec<String>>,
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
