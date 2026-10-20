//! Validation of BoxMux configurations: the JSON schema checks the shape and allowed values of
//! every field, and the semantic checks verify what a schema cannot express (unique ids and
//! references between boxes, layouts and choices).

use crate::model::common::ExecutionMode;
use crate::{App, MuxBox};
use jsonschema::Validator;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

/// App JSON schema, embedded at compile time so validation works regardless of the current
/// working directory or where the binary is installed.
pub const APP_SCHEMA: &str = include_str!("../schemas/app_schema.json");

static APP_SCHEMA_COMPILED: LazyLock<Validator> = LazyLock::new(|| {
    let schema: Value =
        serde_json::from_str(APP_SCHEMA).expect("embedded app schema must be valid JSON");
    jsonschema::validator_for(&schema).expect("embedded app schema must compile")
});

/// The schema of one box: the box definition of the app schema.
static BOX_SCHEMA_COMPILED: LazyLock<Validator> = LazyLock::new(|| {
    let mut schema: Value =
        serde_json::from_str(APP_SCHEMA).expect("embedded app schema must be valid JSON");
    let definitions = schema["definitions"].take();
    let box_schema = serde_json::json!({
        "$ref": "#/definitions/box",
        "definitions": definitions,
    });
    jsonschema::validator_for(&box_schema).expect("embedded box schema must compile")
});

/// One problem in a configuration, with the place it was found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationError {
    /// Where the problem is, e.g. `app.layouts[0].children[2] (box 'logs').border_color`.
    pub location: String,
    pub message: String,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.location, self.message)
    }
}

impl std::error::Error for ValidationError {}

/// Checks a configuration document against the embedded schema.
pub fn validate_schema(document: &Value) -> Result<(), Vec<ValidationError>> {
    let errors: Vec<ValidationError> = APP_SCHEMA_COMPILED
        .iter_errors(document)
        .map(|error| ValidationError {
            location: describe_path(document, &error.instance_path().to_string()),
            message: error.to_string(),
        })
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Checks one box, as sent over the control socket, against the box definition of the
/// schema. Locations start at the box (`position.x1`, `children[0] ('inner').script`).
pub fn validate_box(muxbox: &Value) -> Result<(), Vec<ValidationError>> {
    let errors: Vec<ValidationError> = BOX_SCHEMA_COMPILED
        .iter_errors(muxbox)
        .map(|error| ValidationError {
            location: describe_path(muxbox, &error.instance_path().to_string())
                .replace("configuration", "box"),
            message: error.to_string(),
        })
        .collect();
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Turns a JSON pointer such as `/app/layouts/0/children/2/border_color` into a readable
/// location that names the layouts and boxes on the way.
fn describe_path(document: &Value, pointer: &str) -> String {
    let mut location = String::new();
    let mut node = Some(document);
    for segment in pointer.split('/').filter(|s| !s.is_empty()) {
        node = node.and_then(|n| match n {
            Value::Array(items) => segment.parse::<usize>().ok().and_then(|i| items.get(i)),
            Value::Object(map) => map.get(segment),
            _ => None,
        });
        match segment.parse::<usize>() {
            Ok(index) => {
                location.push_str(&format!("[{}]", index));
                if let Some(id) = node.and_then(|n| n.get("id")).and_then(Value::as_str) {
                    location.push_str(&format!(" ('{}')", id));
                }
            }
            Err(_) => {
                if !location.is_empty() {
                    location.push('.');
                }
                location.push_str(segment);
            }
        }
    }
    if location.is_empty() {
        "configuration".to_string()
    } else {
        location
    }
}

/// Checks the relationships a schema cannot: ids are unique, references point at things that
/// exist, and combinations of fields make sense.
pub fn validate_semantics(app: &App) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();
    let mut error =
        |location: String, message: String| errors.push(ValidationError { location, message });

    if app.layouts.is_empty() {
        error(
            "app.layouts".into(),
            "at least one layout is required".into(),
        );
    }

    let mut layout_ids = HashSet::new();
    let mut box_locations: HashMap<String, String> = HashMap::new();
    let mut box_layout: HashMap<String, String> = HashMap::new();
    let mut choice_owner: HashMap<String, String> = HashMap::new();
    let mut boxes: Vec<(String, &MuxBox)> = Vec::new();

    for (layout_index, layout) in app.layouts.iter().enumerate() {
        let layout_location = format!("app.layouts[{}] ('{}')", layout_index, layout.id);
        if !layout_ids.insert(layout.id.clone()) {
            error(
                layout_location.clone(),
                format!("layout id '{}' is used more than once", layout.id),
            );
        }
        let mut stack: Vec<(String, &MuxBox)> = layout
            .children
            .iter()
            .flatten()
            .enumerate()
            .map(|(i, b)| {
                (
                    format!("{}.children[{}] ('{}')", layout_location, i, b.id),
                    b,
                )
            })
            .collect();
        let mut selected = Vec::new();
        while let Some((location, muxbox)) = stack.pop() {
            if let Some(previous) = box_locations.insert(muxbox.id.clone(), location.clone()) {
                error(
                    location.clone(),
                    format!("box id '{}' is also used at {}", muxbox.id, previous),
                );
            }
            box_layout.insert(muxbox.id.clone(), layout.id.clone());
            if muxbox.selected == Some(true) {
                selected.push(muxbox.id.clone());
            }
            let mut box_choice_ids = HashSet::new();
            for choice in muxbox.choices.iter().flatten() {
                if !box_choice_ids.insert(choice.id.clone()) {
                    error(
                        format!("{}.choices", location),
                        format!(
                            "choice id '{}' is used more than once in this box",
                            choice.id
                        ),
                    );
                }
                if let Some(owner) = choice_owner.insert(choice.id.clone(), muxbox.id.clone()) {
                    if owner != muxbox.id {
                        error(
                            format!("{}.choices", location),
                            format!(
                                "choice id '{}' is also used by box '{}'; hot keys need choice ids to be unique",
                                choice.id, owner
                            ),
                        );
                    }
                }
            }
            for (i, child) in muxbox.children.iter().flatten().enumerate() {
                stack.push((
                    format!("{}.children[{}] ('{}')", location, i, child.id),
                    child,
                ));
            }
            boxes.push((location, muxbox));
        }
        if selected.len() > 1 {
            error(
                layout_location.clone(),
                format!(
                    "only one box per layout can be `selected`; found {}",
                    selected.join(", ")
                ),
            );
        }
    }

    let roots: Vec<&str> = app
        .layouts
        .iter()
        .filter(|l| l.root == Some(true))
        .map(|l| l.id.as_str())
        .collect();
    if roots.len() > 1 {
        error(
            "app.layouts".into(),
            format!("only one layout can be `root`; found {}", roots.join(", ")),
        );
    }

    for (location, muxbox) in &boxes {
        if let Some(target) = &muxbox.redirect_output {
            if !box_locations.contains_key(target) {
                error(
                    format!("{}.redirect_output", location),
                    format!("there is no box '{}'", target),
                );
            }
        }
        if let Some(target) = &muxbox.next_focus_id {
            match box_layout.get(target) {
                None => error(
                    format!("{}.next_focus_id", location),
                    format!("there is no box '{}'", target),
                ),
                Some(target_layout) if Some(target_layout) != box_layout.get(&muxbox.id) => error(
                    format!("{}.next_focus_id", location),
                    format!(
                        "box '{}' is in layout '{}'; focus can only move within a layout",
                        target, target_layout
                    ),
                ),
                Some(_) => {}
            }
        }
        for (i, choice) in muxbox.choices.iter().flatten().enumerate() {
            if let Some(target) = &choice.redirect_output {
                if !box_locations.contains_key(target) {
                    error(
                        format!("{}.choices[{}].redirect_output", location, i),
                        format!("there is no box '{}'", target),
                    );
                }
            }
            if choice.script.is_none() && choice.redirect_output.is_some() {
                error(
                    format!("{}.choices[{}]", location, i),
                    "redirect_output needs a script whose output it redirects".into(),
                );
            }
        }
        if muxbox.execution_mode == ExecutionMode::Pty && muxbox.refresh_interval.is_some() {
            error(
                format!("{}.refresh_interval", location),
                "a PTY box runs its script once as an interactive session; refresh_interval does not apply".into(),
            );
        }
        // On a box with children the interval is inherited by the children's scripts.
        if muxbox.refresh_interval.is_some()
            && muxbox.script.is_none()
            && muxbox.plugin_component.is_none()
            && muxbox
                .children
                .as_ref()
                .is_none_or(|children| children.is_empty())
        {
            error(
                format!("{}.refresh_interval", location),
                "refresh_interval needs a script to rerun, a plugin_component to redraw, or children to pass it on to".into(),
            );
        }
        if muxbox.refresh_interval == Some(0) {
            error(
                format!("{}.refresh_interval", location),
                "must be at least 1 millisecond".into(),
            );
        }
        if muxbox.chart_data.is_some() && muxbox.chart_type.is_none() {
            error(
                format!("{}.chart_data", location),
                "chart_data needs a chart_type".into(),
            );
        }
        if muxbox.chart_type.is_some() && muxbox.chart_data.is_none() && muxbox.script.is_none() {
            error(
                format!("{}.chart_type", location),
                "a chart needs chart_data, or a script whose output is the data".into(),
            );
        }
        if muxbox.table_config.is_some() && muxbox.table_data.is_none() && muxbox.script.is_none() {
            error(
                format!("{}.table_config", location),
                "a table needs table_data, or a script whose output is the data".into(),
            );
        }
        let sources: Vec<&str> = [
            (muxbox.plugin_component.is_some(), "plugin_component"),
            (muxbox.chart_type.is_some(), "chart_type"),
            (
                muxbox.table_data.is_some() || muxbox.table_config.is_some(),
                "table_data/table_config",
            ),
        ]
        .iter()
        .filter(|(set, _)| *set)
        .map(|(_, name)| *name)
        .collect();
        if sources.len() > 1 {
            error(
                location.clone(),
                format!(
                    "a box shows one kind of generated content; it has {}",
                    sources.join(" and ")
                ),
            );
        }
        for (min, max, name) in [
            (muxbox.min_width, muxbox.max_width, "width"),
            (muxbox.min_height, muxbox.max_height, "height"),
        ] {
            if let (Some(min), Some(max)) = (min, max) {
                if min > max {
                    error(
                        format!("{}.min_{}", location, name),
                        format!("min_{} {} is larger than max_{} {}", name, min, name, max),
                    );
                }
            }
        }
    }

    let mut bindings: Vec<(String, &String)> = Vec::new();
    bindings.extend(
        app.hot_keys
            .iter()
            .flatten()
            .map(|(key, _)| ("app.hot_keys".to_string(), key)),
    );
    bindings.extend(
        app.on_keypress
            .iter()
            .flatten()
            .map(|(key, _)| ("app.on_keypress".to_string(), key)),
    );
    for (i, layout) in app.layouts.iter().enumerate() {
        let location = format!("app.layouts[{}] ('{}').on_keypress", i, layout.id);
        bindings.extend(
            layout
                .on_keypress
                .iter()
                .flatten()
                .map(|(key, _)| (location.clone(), key)),
        );
    }
    for (location, muxbox) in &boxes {
        let location = format!("{}.on_keypress", location);
        bindings.extend(
            muxbox
                .on_keypress
                .iter()
                .flatten()
                .map(|(key, _)| (location.clone(), key)),
        );
    }
    for (location, key) in bindings {
        if let Err(message) = crate::runtime::keys::check_key_name(key) {
            error(format!("{}.{}", location, key), message);
        }
    }

    for (key, choice_id) in app.hot_keys.iter().flatten() {
        if !choice_owner.contains_key(choice_id) {
            error(
                format!("app.hot_keys.{}", key),
                format!("there is no choice '{}'", choice_id),
            );
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_schema_compiles() {
        LazyLock::force(&APP_SCHEMA_COMPILED);
    }

    /// The properties the schema allows for one object definition.
    fn schema_properties(definition: &Value) -> Vec<String> {
        let mut keys: Vec<String> = definition["properties"]
            .as_object()
            .expect("the definition has properties")
            .keys()
            .cloned()
            .collect();
        keys.sort();
        keys
    }

    /// The keys a struct reads and writes, from serializing its default value.
    fn struct_fields<T: serde::Serialize + Default>() -> Vec<String> {
        let mut keys: Vec<String> = serde_json::to_value(T::default())
            .unwrap()
            .as_object()
            .expect("the struct serializes to an object")
            .keys()
            .cloned()
            .collect();
        keys.sort();
        keys
    }

    #[test]
    fn the_schema_and_the_configuration_structs_have_the_same_fields() {
        let schema: Value = serde_json::from_str(APP_SCHEMA).unwrap();
        let definitions = &schema["definitions"];
        assert_eq!(
            schema_properties(definitions.get("box").unwrap()),
            struct_fields::<crate::MuxBox>(),
            "box"
        );
        assert_eq!(
            schema_properties(definitions.get("layout").unwrap()),
            struct_fields::<crate::Layout>(),
            "layout"
        );
        assert_eq!(
            schema_properties(definitions.get("choice").unwrap()),
            struct_fields::<crate::Choice>(),
            "choice"
        );
        assert_eq!(
            schema_properties(&schema["properties"]["app"]),
            struct_fields::<crate::App>(),
            "app"
        );
        assert_eq!(
            schema_properties(&definitions["box"]["properties"]["table_config"]),
            struct_fields::<crate::table::TableSettings>(),
            "table_config"
        );
    }

    fn semantic_errors(yaml: &str) -> Vec<String> {
        let root: crate::model::app::TemplateRoot = serde_yaml::from_str(yaml).unwrap();
        validate_semantics(&root.app)
            .err()
            .unwrap_or_default()
            .iter()
            .map(|e| e.to_string())
            .collect()
    }

    #[test]
    fn binding_key_names_are_checked_everywhere() {
        let errors = semantic_errors(
            r#"
app:
  hot_keys: {Ctrl+Enterr: pick}
  on_keypress: {q: ["echo"]}
  layouts:
    - id: main
      on_keypress: {Shift+a: ["echo"]}
      children:
        - id: menu
          position: {x1: "0", y1: "0", x2: "9", y2: "9"}
          on_keypress: {Hyper+x: ["echo"], Ctrl+x: ["echo"]}
          choices: [{id: pick, content: Pick, script: ["echo"]}]
"#,
        );
        assert_eq!(errors.len(), 4, "{:#?}", errors);
        for expected in [
            "app.hot_keys.Ctrl+Enterr: 'Enterr' is not a key",
            "app.on_keypress.q: q always quits",
            "app.layouts[0] ('main').on_keypress.Shift+a: Shift is part of the character",
            "('menu').on_keypress.Hyper+x: 'Hyper' is not a modifier",
        ] {
            assert!(
                errors.iter().any(|e| e.contains(expected)),
                "{}: {:#?}",
                expected,
                errors
            );
        }
    }

    #[test]
    fn refresh_interval_on_a_box_with_children_is_passed_on() {
        let yaml = |children: &str| {
            format!(
                r#"
app:
  layouts:
    - id: main
      children:
        - id: group
          position: {{x1: "0", y1: "0", x2: "9", y2: "9"}}
          refresh_interval: 500
          {}
"#,
                children
            )
        };
        let with_children = yaml(
            r#"children: [{id: inner, position: {x1: "0", y1: "0", x2: "5", y2: "5"}, script: ["date"]}]"#,
        );
        assert_eq!(semantic_errors(&with_children), Vec::<String>::new());
        let alone = semantic_errors(&yaml("content: nothing to refresh"));
        assert!(
            alone[0].contains("refresh_interval needs a script"),
            "{:?}",
            alone
        );
    }

    #[test]
    fn generated_content_needs_data_and_only_one_kind() {
        let errors = semantic_errors(
            r#"
app:
  layouts:
    - id: l
      children:
        - id: chart_without_data
          position: {x1: "0", y1: "0", x2: "9", y2: "9"}
          chart_type: bar
        - id: both
          position: {x1: "0", y1: "0", x2: "9", y2: "9"}
          chart_type: line
          chart_data: "a,1"
          table_data: "h\n1"
        - id: sizes
          position: {x1: "0", y1: "0", x2: "9", y2: "9"}
          min_width: 20
          max_width: 10
"#,
        );
        let all = errors.join("\n");
        assert!(
            all.contains("chart_without_data") && all.contains("needs chart_data"),
            "{}",
            all
        );
        assert!(
            all.contains("'both'") && all.contains("one kind of generated content"),
            "{}",
            all
        );
        assert!(
            all.contains("min_width 20 is larger than max_width 10"),
            "{}",
            all
        );
        assert_eq!(errors.len(), 3, "{}", all);
    }

    #[test]
    fn schema_errors_name_the_box_they_are_in() {
        let document: Value = serde_json::json!({
            "app": {"layouts": [{"id": "main", "children": [
                {"id": "logs", "position": {"x1": "0", "y1": "0", "x2": "1", "y2": "1"},
                 "border_color": "purple"}
            ]}]}
        });
        let errors = validate_schema(&document).unwrap_err();
        assert_eq!(errors.len(), 1, "{:?}", errors);
        assert_eq!(
            errors[0].location,
            "app.layouts[0] ('main').children[0] ('logs').border_color"
        );
    }
}
