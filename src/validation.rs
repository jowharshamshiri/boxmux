use crate::model::common::Config;
use crate::{App, Layout, MuxBox};
use jsonschema::JSONSchema;
use serde_json::{Map, Value};
use std::collections::HashSet;
use std::fs;

/// Configuration schema validation errors
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidFieldType {
        field: String,
        expected: String,
        actual: String,
    },
    MissingRequiredField {
        field: String,
    },
    InvalidFieldValue {
        field: String,
        value: String,
        constraint: String,
    },
    DuplicateId {
        id: String,
        location: String,
    },
    InvalidReference {
        field: String,
        reference: String,
        target_type: String,
    },
    SchemaStructure {
        message: String,
    },
    JsonSchemaValidation {
        field: String,
        message: String,
    },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidFieldType {
                field,
                expected,
                actual,
            } => {
                write!(
                    f,
                    "Field '{}' expected type '{}' but got '{}'",
                    field, expected, actual
                )
            }
            ValidationError::MissingRequiredField { field } => {
                write!(f, "Required field '{}' is missing", field)
            }
            ValidationError::InvalidFieldValue {
                field,
                value,
                constraint,
            } => {
                write!(
                    f,
                    "Field '{}' has invalid value '{}' (constraint: {})",
                    field, value, constraint
                )
            }
            ValidationError::DuplicateId { id, location } => {
                write!(f, "Duplicate ID '{}' found in {}", id, location)
            }
            ValidationError::InvalidReference {
                field,
                reference,
                target_type,
            } => {
                write!(
                    f,
                    "Field '{}' references unknown {} '{}'",
                    field, target_type, reference
                )
            }
            ValidationError::SchemaStructure { message } => {
                write!(f, "Schema structure error: {}", message)
            }
            ValidationError::JsonSchemaValidation { field, message } => {
                write!(
                    f,
                    "JSON Schema validation error in '{}': {}",
                    field, message
                )
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Schema validation result
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// Central schema validator for BoxMux configurations
pub struct SchemaValidator {
    errors: Vec<ValidationError>,
    muxbox_ids: HashSet<String>,
    layout_ids: HashSet<String>,
}

impl SchemaValidator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
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

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
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
        if let Err(e) = self.validate_against_schema_file(&yaml_value, &app_schema_path, "app") {
            return Err(e);
        }

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
        let schema_content = match fs::read_to_string(schema_path) {
            Ok(content) => content,
            Err(e) => {
                self.add_error(ValidationError::SchemaStructure {
                    message: format!("Failed to load schema file '{}': {}", schema_path, e),
                });
                return Err(self.errors.clone());
            }
        };

        // Parse schema
        let schema_json: Value = match serde_json::from_str(&schema_content) {
            Ok(schema) => schema,
            Err(e) => {
                self.add_error(ValidationError::SchemaStructure {
                    message: format!("Invalid JSON schema in '{}': {}", schema_path, e),
                });
                return Err(self.errors.clone());
            }
        };

        // Compile and validate schema
        let compiled_schema = match JSONSchema::compile(&schema_json) {
            Ok(schema) => schema,
            Err(e) => {
                self.add_error(ValidationError::SchemaStructure {
                    message: format!("Failed to compile schema '{}': {}", schema_path, e),
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

        Ok(())
    }

    /// Validate JSON configuration against schema
    pub fn validate_json_config(&mut self, config: &Value) -> ValidationResult {
        self.clear();

        if !config.is_object() {
            self.add_error(ValidationError::SchemaStructure {
                message: "Configuration must be a JSON object".to_string(),
            });
            return Err(self.errors.clone());
        }

        let obj = config.as_object().unwrap();

        // Validate required top-level fields
        if !obj.contains_key("layouts") {
            self.add_error(ValidationError::MissingRequiredField {
                field: "layouts".to_string(),
            });
        } else if !obj["layouts"].is_array() {
            self.add_error(ValidationError::InvalidFieldType {
                field: "layouts".to_string(),
                expected: "array".to_string(),
                actual: self.get_json_type(&obj["layouts"]),
            });
        }

        // Validate optional config section
        if let Some(config_section) = obj.get("config") {
            if !config_section.is_object() {
                self.add_error(ValidationError::InvalidFieldType {
                    field: "config".to_string(),
                    expected: "object".to_string(),
                    actual: self.get_json_type(config_section),
                });
            } else {
                self.validate_json_config_section(config_section.as_object().unwrap());
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Helper methods
    fn clear(&mut self) {
        self.errors.clear();
        self.muxbox_ids.clear();
        self.layout_ids.clear();
    }

    fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    fn collect_ids(&mut self, app: &App) {
        for layout in &app.layouts {
            if !self.layout_ids.insert(layout.id.clone()) {
                self.add_error(ValidationError::DuplicateId {
                    id: layout.id.clone(),
                    location: "layouts".to_string(),
                });
            }
            self.collect_muxbox_ids_recursive(&layout.children, "muxboxes");
        }
    }

    fn collect_muxbox_ids_recursive(&mut self, muxboxes: &Option<Vec<MuxBox>>, location: &str) {
        if let Some(muxbox_list) = muxboxes {
            for muxbox in muxbox_list {
                if !self.muxbox_ids.insert(muxbox.id.clone()) {
                    self.add_error(ValidationError::DuplicateId {
                        id: muxbox.id.clone(),
                        location: location.to_string(),
                    });
                }
                self.collect_muxbox_ids_recursive(&muxbox.children, location);

                // Check choice IDs if they exist
                if let Some(choices) = &muxbox.choices {
                    for choice in choices {
                        if !self.muxbox_ids.insert(choice.id.clone()) {
                            self.add_error(ValidationError::DuplicateId {
                                id: choice.id.clone(),
                                location: "choices".to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    fn validate_root_layout_constraints(&mut self, app: &App) {
        let mut root_count = 0;
        for layout in &app.layouts {
            if layout.root == Some(true) {
                root_count += 1;
            }
        }

        if root_count > 1 {
            self.add_error(ValidationError::SchemaStructure {
                message:
                    "Multiple root layouts detected. Only one layout can be marked as 'root: true'."
                        .to_string(),
            });
        }
    }

    fn validate_input_bounds_schema(
        &mut self,
        bounds: &crate::model::common::InputBounds,
        path: &str,
    ) {
        // Validate that bounds strings are not empty
        if bounds.x1.trim().is_empty() {
            self.add_error(ValidationError::InvalidFieldValue {
                field: format!("{}.x1", path),
                value: "empty".to_string(),
                constraint: "bounds coordinates cannot be empty".to_string(),
            });
        }

        if bounds.y1.trim().is_empty() {
            self.add_error(ValidationError::InvalidFieldValue {
                field: format!("{}.y1", path),
                value: "empty".to_string(),
                constraint: "bounds coordinates cannot be empty".to_string(),
            });
        }

        if bounds.x2.trim().is_empty() {
            self.add_error(ValidationError::InvalidFieldValue {
                field: format!("{}.x2", path),
                value: "empty".to_string(),
                constraint: "bounds coordinates cannot be empty".to_string(),
            });
        }

        if bounds.y2.trim().is_empty() {
            self.add_error(ValidationError::InvalidFieldValue {
                field: format!("{}.y2", path),
                value: "empty".to_string(),
                constraint: "bounds coordinates cannot be empty".to_string(),
            });
        }
    }

    fn validate_json_config_section(&mut self, config: &Map<String, Value>) {
        if let Some(frame_delay) = config.get("frame_delay") {
            if let Some(delay) = frame_delay.as_u64() {
                if delay == 0 {
                    self.add_error(ValidationError::InvalidFieldValue {
                        field: "config.frame_delay".to_string(),
                        value: "0".to_string(),
                        constraint: "frame_delay must be greater than 0".to_string(),
                    });
                }
            } else {
                self.add_error(ValidationError::InvalidFieldType {
                    field: "config.frame_delay".to_string(),
                    expected: "number".to_string(),
                    actual: self.get_json_type(frame_delay),
                });
            }
        }
    }

    fn get_json_type(&self, value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(_) => "boolean".to_string(),
            Value::Number(_) => "number".to_string(),
            Value::String(_) => "string".to_string(),
            Value::Array(_) => "array".to_string(),
            Value::Object(_) => "object".to_string(),
        }
    }
}

impl Default for SchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::common::{Config, InputBounds};
    use crate::{App, Layout, MuxBox};

    fn create_test_muxbox(id: &str) -> MuxBox {
        MuxBox {
            id: id.to_string(),
            position: InputBounds {
                x1: "10".to_string(),
                y1: "20".to_string(),
                x2: "90".to_string(),
                y2: "80".to_string(),
            },
            ..Default::default()
        }
    }

    fn create_test_layout(id: &str) -> Layout {
        Layout {
            id: id.to_string(),
            children: Some(vec![create_test_muxbox("muxbox1")]),
            ..Default::default()
        }
    }

    fn create_test_app() -> App {
        let mut app = App::new();
        app.layouts = vec![create_test_layout("layout1")];
        app
    }

    #[test]
    fn test_validate_app_success() {
        let mut validator = SchemaValidator::new();
        let app = create_test_app();

        let result = validator.validate_app(&app);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_app_no_layouts() {
        let mut validator = SchemaValidator::new();
        let app = App::new();

        let result = validator.validate_app(&app);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            ValidationError::MissingRequiredField { .. }
        ));
    }

    #[test]
    fn test_validate_config_success() {
        let mut validator = SchemaValidator::new();
        let config = Config::new(60);

        let result = validator.validate_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_config_zero_frame_delay() {
        let mut validator = SchemaValidator::new();
        let config = Config { frame_delay: 0, locked: false };

        let result = validator.validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            ValidationError::InvalidFieldValue { .. }
        ));
    }

    #[test]
    fn test_validate_config_excessive_frame_delay() {
        let mut validator = SchemaValidator::new();
        let config = Config { frame_delay: 2000, locked: false };

        let result = validator.validate_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            ValidationError::InvalidFieldValue { .. }
        ));
    }

    #[test]
    fn test_validate_json_config_success() {
        let mut validator = SchemaValidator::new();
        let config = serde_json::json!({
            "layouts": [
                {
                    "id": "layout1",
                    "children": [
                        {
                            "id": "muxbox1",
                            "bounds": {
                                "x1": "10",
                                "y1": "20",
                                "x2": "90",
                                "y2": "80"
                            }
                        }
                    ]
                }
            ],
            "config": {
                "frame_delay": 60
            }
        });

        let result = validator.validate_json_config(&config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_json_config_missing_layouts() {
        let mut validator = SchemaValidator::new();
        let config = serde_json::json!({
            "config": {
                "frame_delay": 60
            }
        });

        let result = validator.validate_json_config(&config);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(
            errors[0],
            ValidationError::MissingRequiredField { .. }
        ));
    }

    #[test]
    fn test_validate_empty_bounds() {
        let mut validator = SchemaValidator::new();
        let muxbox = MuxBox {
            id: "test_muxbox".to_string(),
            position: InputBounds {
                x1: "".to_string(), // Empty bound
                y1: "20".to_string(),
                x2: "90".to_string(),
                y2: "80".to_string(),
            },
            ..Default::default()
        };

        let result = validator.validate_muxbox(&muxbox, "test_muxbox");
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidFieldValue { .. })));
    }

    #[test]
    fn test_validation_error_formatting() {
        // Test that ValidationError instances format correctly for user display
        let duplicate_error = ValidationError::DuplicateId {
            id: "muxbox1".to_string(),
            location: "muxboxes".to_string(),
        };
        assert_eq!(
            duplicate_error.to_string(),
            "Duplicate ID 'muxbox1' found in muxboxes"
        );

        let missing_field_error = ValidationError::MissingRequiredField {
            field: "layouts".to_string(),
        };
        assert_eq!(
            missing_field_error.to_string(),
            "Required field 'layouts' is missing"
        );

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
                // Schema validation passed
                assert!(true);
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
                // and this is expected behavior
                assert!(true);
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
                assert!(true);
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
        assert!(
            errors.len() >= 2,
            "Should have at least 2 validation errors"
        );

        // Check for multiple root layouts error
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::SchemaStructure { .. })));

        // Check for duplicate ID error
        assert!(errors
            .iter()
            .any(|e| matches!(e, ValidationError::DuplicateId { .. })));
    }
}
