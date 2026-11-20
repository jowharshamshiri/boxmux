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
    /// Activating the choice asks this first, and runs the script only on an answer other than
    /// cancelling.
    #[serde(default)]
    pub confirm: Option<Confirm>,
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

/// The question a choice asks before it runs: its text and the answers offered.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Confirm {
    pub text: String,
    /// The answers, in the order they are shown; "Yes" and "Cancel" when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<ConfirmOption>>,
    /// The id of the answer that cancels, one of `options`. Only with `options`: the default
    /// answers cancel with "Cancel".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cancel: Option<String>,
}

/// One answer a confirmation offers.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfirmOption {
    pub id: String,
    /// The label on its button; the id when left out.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
}

impl ConfirmOption {
    pub fn label(&self) -> &str {
        self.content.as_deref().unwrap_or(&self.id)
    }
}

/// The id of the default answer that runs the choice.
pub const DEFAULT_CONFIRM_OPTION: &str = "yes";
/// The id of the default answer that cancels.
pub const DEFAULT_CANCEL_OPTION: &str = "cancel";

impl Confirm {
    /// The answers offered, in order: the declared ones, or "Yes" and "Cancel".
    pub fn options(&self) -> Vec<ConfirmOption> {
        match &self.options {
            Some(options) => options.clone(),
            None => vec![
                ConfirmOption {
                    id: DEFAULT_CONFIRM_OPTION.to_string(),
                    content: Some("Yes".to_string()),
                },
                ConfirmOption {
                    id: DEFAULT_CANCEL_OPTION.to_string(),
                    content: Some("Cancel".to_string()),
                },
            ],
        }
    }

    /// The id of the answer that cancels, if one of the answers does.
    pub fn cancel_id(&self) -> Option<&str> {
        match &self.options {
            Some(_) => self.cancel.as_deref(),
            None => Some(DEFAULT_CANCEL_OPTION),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn choice(yaml: &str) -> Choice {
        crate::yaml::from_str::<Choice>(yaml).unwrap()
    }

    #[test]
    fn a_confirmation_without_options_offers_yes_and_cancel_and_cancel_cancels() {
        let confirm = choice("id: go\nscript: [true]\nconfirm: {text: Sure?}\n")
            .confirm
            .unwrap();
        let options = confirm.options();
        let labels: Vec<(&str, &str)> =
            options.iter().map(|o| (o.id.as_str(), o.label())).collect();
        assert_eq!(labels, vec![("yes", "Yes"), ("cancel", "Cancel")]);
        assert_eq!(confirm.cancel_id(), Some("cancel"));
    }

    #[test]
    fn declared_options_are_offered_as_written_and_cancel_only_when_named() {
        let confirm = choice(
            "id: go\nscript: [true]\nconfirm:\n  text: Where?\n  options: [{id: staging}, {id: prod, content: Production}]\n",
        )
        .confirm
        .unwrap();
        let options = confirm.options();
        let labels: Vec<(&str, &str)> =
            options.iter().map(|o| (o.id.as_str(), o.label())).collect();
        assert_eq!(labels, vec![("staging", "staging"), ("prod", "Production")]);
        assert_eq!(confirm.cancel_id(), None);
    }

    #[test]
    fn an_unknown_field_of_a_confirmation_is_refused() {
        let error = crate::yaml::from_str::<Choice>(
            "id: go\nscript: [true]\nconfirm: {text: Sure?, title: Hm}\n",
        )
        .unwrap_err()
        .to_string();
        assert!(error.contains("title"), "{error}");
    }
}
