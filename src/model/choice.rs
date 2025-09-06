use std::hash::{Hash,Hasher};

use serde::{Deserialize, Serialize};

use crate::ExecutionMode;
use crate::model::common::deserialize_script;


#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Choice {
    pub id: String,
    pub content: Option<String>,
    #[serde(deserialize_with = "deserialize_script", default)]
    pub script: Option<Vec<String>>,
    pub redirect_output: Option<String>,
    pub append_output: Option<bool>,
    // F0222: Choice ExecutionMode Field - Replace thread+pty boolean flags with single execution_mode enum
    #[serde(default)]
    pub execution_mode: ExecutionMode,
    #[serde(skip, default)]
    pub selected: bool,
    #[serde(skip, default)]
    pub waiting: bool,
    #[serde(skip, default)]
    pub hovered: bool,
}

impl Choice {}

impl Hash for Choice {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.content.hash(state);
        self.script.hash(state);
        self.redirect_output.hash(state);
        self.append_output.hash(state);
        // F0222: Hash ExecutionMode field
        self.execution_mode.hash(state);
        self.selected.hash(state);
        self.waiting.hash(state);
        self.hovered.hash(state);
    }
}

impl PartialEq for Choice {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.content == other.content
            && self.script == other.script
            && self.redirect_output == other.redirect_output
            && self.append_output == other.append_output
            // F0222: Compare ExecutionMode field
            && self.execution_mode == other.execution_mode
            && self.selected == other.selected
            && self.waiting == other.waiting
            && self.hovered == other.hovered
    }
}

impl Eq for Choice {}

impl Clone for Choice {
    fn clone(&self) -> Self {
        Choice {
            id: self.id.clone(),
            content: self.content.clone(),
            script: self.script.clone(),
            redirect_output: self.redirect_output.clone(),
            append_output: self.append_output,
            // F0222: Clone ExecutionMode field
            execution_mode: self.execution_mode.clone(),
            selected: self.selected,
            waiting: self.waiting,
            hovered: self.hovered,
        }
    }
}
