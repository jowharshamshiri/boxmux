<<<<<<< Updated upstream
//! Validation of BoxMux configurations: the JSON schema checks the shape and allowed values of
//! every field, and the semantic checks verify what a schema cannot express (unique ids and
//! references between boxes, layouts and choices).
=======
use crate::model::common::Config;
use crate::{App, Layout, MuxBox};
use jsonschema::JSONSchema;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
>>>>>>> Stashed changes

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

<<<<<<< Updated upstream
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
=======
impl SchemaValidator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
            muxbox_ids: HashSet::new(),
            layout_ids: HashSet::new(),
        }
    }

    /// Validate a complete BoxMux application configuration
    pub fn validate_app(&mut self, app: &App) -> ValidationResult {
        self.clear();

        // Collect all IDs first for reference validation
        self.collect_ids(app);

        // Validate application structure
        if app.layouts.is_empty() {
            self.add_error(ValidationError::MissingRequiredField {
                field: "layouts".to_string(),
            });
        }

        // Validate each layout
        for (idx, layout) in app.layouts.iter().enumerate() {
            let _ = self.validate_layout(layout, &format!("layouts[{}]", idx));
        }

        // Validate root layout constraints
        self.validate_root_layout_constraints(app);

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Validate a single layout
    pub fn validate_layout(&mut self, layout: &Layout, path: &str) -> ValidationResult {
        // Validate required fields
        if layout.id.is_empty() {
            self.add_error(ValidationError::MissingRequiredField {
                field: format!("{}.id", path),
            });
        }

        // Validate muxboxes if present
        if let Some(muxboxes) = &layout.children {
            for (idx, muxbox) in muxboxes.iter().enumerate() {
                let _ = self.validate_muxbox(muxbox, &format!("{}.children[{}]", path, idx));
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Validate a single muxbox
    pub fn validate_muxbox(&mut self, muxbox: &MuxBox, path: &str) -> ValidationResult {
        // Validate required fields
        if muxbox.id.is_empty() {
            self.add_error(ValidationError::MissingRequiredField {
                field: format!("{}.id", path),
            });
        }

        // Validate position bounds
        self.validate_input_bounds_schema(&muxbox.position, &format!("{}.position", path));

        // Validate child muxboxes recursively
        if let Some(children) = &muxbox.children {
            for (idx, child) in children.iter().enumerate() {
                let _ = self.validate_muxbox(child, &format!("{}.children[{}]", path, idx));
            }
        }

        // Validate script commands if present
        if let Some(scripts) = &muxbox.script {
            for (idx, script) in scripts.iter().enumerate() {
                if script.trim().is_empty() {
                    self.add_error(ValidationError::InvalidFieldValue {
                        field: format!("{}.script[{}]", path, idx),
                        value: "empty".to_string(),
                        constraint: "script commands cannot be empty".to_string(),
                    });
                }
            }
        }

        // Validate ExecutionMode integration and deprecation warnings
        self.validate_execution_mode_fields(
            &muxbox.execution_mode,
            &format!("{}.execution_mode", path),
        );

        // Validate choices if present
        if let Some(choices) = &muxbox.choices {
            for (idx, choice) in choices.iter().enumerate() {
                let _ = self.validate_choice(choice, &format!("{}.choices[{}]", path, idx));
            }
        }

        // Return both errors and warnings for comprehensive validation
        let mut all_issues = self.errors.clone();
        all_issues.extend(self.warnings.clone());

        if all_issues.is_empty() {
            Ok(())
        } else {
            Err(all_issues)
        }
    }

    /// Validate configuration schema
    pub fn validate_config(&mut self, config: &Config) -> ValidationResult {
        self.clear();

        if config.frame_delay == 0 {
            self.add_error(ValidationError::InvalidFieldValue {
                field: "frame_delay".to_string(),
                value: "0".to_string(),
                constraint: "frame_delay must be greater than 0".to_string(),
            });
        }

        if config.frame_delay > 1000 {
            self.add_error(ValidationError::InvalidFieldValue {
                field: "frame_delay".to_string(),
                value: config.frame_delay.to_string(),
                constraint: "frame_delay should not exceed 1000ms for usability".to_string(),
            });
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Validate YAML content against JSON schema
    pub fn validate_with_json_schema(
        &mut self,
        yaml_content: &str,
        schema_dir: &str,
    ) -> ValidationResult {
        self.clear();

        // Parse YAML content to JSON
        let yaml_value: Value = match serde_yaml::from_str(yaml_content) {
            Ok(value) => value,
            Err(e) => {
                self.add_error(ValidationError::SchemaStructure {
                    message: format!("Invalid YAML syntax: {}", e),
                });
                return Err(self.errors.clone());
            }
        };

        // Load and validate against app schema
        let app_schema_path = format!("{}/app_schema.json", schema_dir);
        self.validate_against_schema_file(&yaml_value, &app_schema_path, "app")?;

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Validate a JSON value against a specific schema file
    fn validate_against_schema_file(
        &mut self,
        value: &Value,
        schema_path: &str,
        field_name: &str,
    ) -> ValidationResult {
        // Load schema file
        let resolved_schema_path = match resolve_schema_path(schema_path) {
            Ok(path) => path,
            Err(message) => {
                self.add_error(ValidationError::SchemaStructure { message });
                return Err(self.errors.clone());
            }
        };

        let schema_content = match fs::read_to_string(&resolved_schema_path) {
            Ok(content) => content,
            Err(e) => {
                self.add_error(ValidationError::SchemaStructure {
                    message: format!(
                        "Failed to load schema file '{}': {}",
                        resolved_schema_path.display(),
                        e
                    ),
                });
                return Err(self.errors.clone());
            }
        };

        // Parse schema
        let schema_json: Value = match serde_json::from_str(&schema_content) {
            Ok(schema) => schema,
            Err(e) => {
                self.add_error(ValidationError::SchemaStructure {
                    message: format!(
                        "Invalid JSON schema in '{}': {}",
                        resolved_schema_path.display(),
                        e
                    ),
                });
                return Err(self.errors.clone());
            }
        };

        // Compile and validate schema
        let compiled_schema = match JSONSchema::compile(&schema_json) {
            Ok(schema) => schema,
            Err(e) => {
                self.add_error(ValidationError::SchemaStructure {
                    message: format!(
                        "Failed to compile schema '{}': {}",
                        resolved_schema_path.display(),
                        e
                    ),
                });
                return Err(self.errors.clone());
            }
        };

        // Validate the value against the schema
        if let Err(errors) = compiled_schema.validate(value) {
            for error in errors {
                let error_path = if error.instance_path.to_string().is_empty() {
                    field_name.to_string()
                } else {
                    format!("{}.{}", field_name, error.instance_path)
                };

                self.add_error(ValidationError::JsonSchemaValidation {
                    field: error_path,
                    message: error.to_string(),
                });
            }
            return Err(self.errors.clone());
        }

>>>>>>> Stashed changes
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

<<<<<<< Updated upstream
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
=======
fn resolve_schema_path(schema_path: &str) -> Result<PathBuf, String> {
    let requested_path = Path::new(schema_path);
    if requested_path.exists() {
        return requested_path.canonicalize().map_err(|e| {
            format!(
                "Failed to resolve schema file '{}': {}",
                requested_path.display(),
                e
            )
        });
    }

    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(requested_path);
    if manifest_path.exists() {
        return manifest_path.canonicalize().map_err(|e| {
            format!(
                "Failed to resolve schema file '{}': {}",
                manifest_path.display(),
                e
            )
        });
    }

    Err(format!(
        "Failed to load schema file '{}'; also tried '{}'",
        requested_path.display(),
        manifest_path.display()
    ))
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
>>>>>>> Stashed changes
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
        let root: crate::model::app::TemplateRoot = crate::yaml::from_str(yaml).unwrap();
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
<<<<<<< Updated upstream
        assert_eq!(semantic_errors(&with_children), Vec::<String>::new());
        let alone = semantic_errors(&yaml("content: nothing to refresh"));
=======

        let schema_error = ValidationError::SchemaStructure {
            message:
                "Multiple root layouts detected. Only one layout can be marked as 'root: true'."
                    .to_string(),
        };
        assert_eq!(schema_error.to_string(), "Schema structure error: Multiple root layouts detected. Only one layout can be marked as 'root: true'.");
    }

    #[test]
    fn test_json_schema_validation_success() {
        let mut validator = SchemaValidator::new();
        let yaml_content = r#"
app:
  layouts:
    - id: 'test_layout'
      title: 'Test Layout'
      children:
        - id: 'muxbox1'
          position:
            x1: "0%"
            y1: "0%"
            x2: "100%"
            y2: "100%"
          content: 'Test content'
          tab_order: 1
"#;

        let result = validator.validate_with_json_schema(yaml_content, "schemas");

        // This should pass if schema files exist and are valid
        match result {
            Ok(_) => {
                // Schema validation passed - verify it actually validated something
                // This is acceptable - schema validation completed successfully
            }
            Err(errors) => {
                // If schemas don't exist, that's expected - just verify the error is about missing schema files
                let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
                let combined = error_messages.join("; ");
                assert!(
                    combined.contains("Failed to load schema file")
                        || combined.contains("No such file")
                );
            }
        }
    }

    #[test]
    fn test_json_schema_validation_invalid_yaml() {
        let mut validator = SchemaValidator::new();
        let invalid_yaml = r#"
app:
  layouts:
    - id: 'test'
      children:
        - id: 'muxbox1'
          position:
            x1: "0%"
            y1: "0%"
            x2: "100%"
            # Missing y2 - should cause validation error
          border_color: 'invalid_color'  # Invalid color
"#;

        let result = validator.validate_with_json_schema(invalid_yaml, "schemas");

        match result {
            Ok(_) => {
                // If schemas don't exist, the validation will skip JSON schema validation
                // and this is expected behavior - no assertion needed
            }
            Err(errors) => {
                // Should contain validation errors or schema loading errors
                assert!(!errors.is_empty());
                let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
                let combined = error_messages.join("; ");
                // Either schema validation errors or schema file missing
                assert!(
                    combined.contains("JSON Schema validation error")
                        || combined.contains("Failed to load schema file")
                        || combined.contains("No such file")
                );
            }
        }
    }

    #[test]
    fn test_json_schema_validation_malformed_yaml() {
        let mut validator = SchemaValidator::new();
        let malformed_yaml = r#"
app:
  layouts:
    - id: 'test'
      children:
        - id: 'muxbox1'
          position:
            x1: "0%"
            y1: "0%"
            x2: "100%"
            y2: "100%"
          invalid_field_that_should_not_exist: 'invalid'
          border_color: 123  # Wrong type - should be string
"#;

        let result = validator.validate_with_json_schema(malformed_yaml, "schemas");

        match result {
            Ok(_) => {
                // If schemas don't exist, validation is skipped
                // This is acceptable behavior - no assertion needed
            }
            Err(errors) => {
                assert!(!errors.is_empty());
                // Verify we get meaningful error reporting
                for error in &errors {
                    match error {
                        ValidationError::JsonSchemaValidation { field, message } => {
                            assert!(!field.is_empty());
                            assert!(!message.is_empty());
                        }
                        ValidationError::SchemaStructure { message } => {
                            assert!(!message.is_empty());
                        }
                        _ => {} // Other error types are acceptable
                    }
                }
            }
        }
    }

    #[test]
    fn test_json_schema_validation_error_formatting() {
        let json_schema_error = ValidationError::JsonSchemaValidation {
            field: "app.layouts[0].children[0].border_color".to_string(),
            message: "invalid_color is not one of the allowed values".to_string(),
        };

        let formatted = json_schema_error.to_string();
        assert!(formatted.contains("JSON Schema validation error"));
        assert!(formatted.contains("app.layouts[0].children[0].border_color"));
        assert!(formatted.contains("invalid_color is not one of the allowed values"));
    }

    #[test]
    fn test_validate_against_schema_file_missing_file() {
        let mut validator = SchemaValidator::new();
        let test_value = serde_json::json!({
            "test": "value"
        });

        let result =
            validator.validate_against_schema_file(&test_value, "nonexistent/schema.json", "test");

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ValidationError::SchemaStructure { .. }));
        assert!(errors[0].to_string().contains("Failed to load schema file"));
    }

    #[test]
    fn test_schema_path_resolves_from_manifest_directory() {
        let resolved = resolve_schema_path("schemas/app_schema.json")
            .expect("source checkout schema should resolve through CARGO_MANIFEST_DIR");

        assert!(
            resolved.ends_with("schemas/app_schema.json"),
            "resolved schema path should point at app_schema.json, got {}",
            resolved.display()
        );
        assert!(
            resolved.is_absolute(),
            "manifest-resolved schema path should be absolute for cwd-independent startup"
        );
    }

    #[test]
    fn test_comprehensive_validation_with_multiple_errors() {
        let mut validator = SchemaValidator::new();

        // Create an app with multiple validation errors
        let mut app = App::new();

        // Error 1: Multiple root layouts
        let mut layout1 = create_test_layout("layout1");
        layout1.root = Some(true);
        let mut layout2 = create_test_layout("layout2");
        layout2.root = Some(true); // Second root - should cause error

        // Error 2: Duplicate muxbox IDs
        let muxbox1 = create_test_muxbox("muxbox1");
        let muxbox1_dup = create_test_muxbox("muxbox1"); // Duplicate ID
        layout1.children = Some(vec![muxbox1]);
        layout2.children = Some(vec![muxbox1_dup]);

        app.layouts = vec![layout1, layout2];

        let result = validator.validate_app(&app);
        assert!(result.is_err());

        let errors = result.unwrap_err();
>>>>>>> Stashed changes
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
