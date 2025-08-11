use crate::{App, Panel, Layout};
use crate::model::common::Config;
use serde_json::{Value, Map};
use std::collections::HashSet;

/// Configuration schema validation errors
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidFieldType { field: String, expected: String, actual: String },
    MissingRequiredField { field: String },
    InvalidFieldValue { field: String, value: String, constraint: String },
    DuplicateId { id: String, location: String },
    InvalidReference { field: String, reference: String, target_type: String },
    SchemaStructure { message: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidFieldType { field, expected, actual } => {
                write!(f, "Field '{}' expected type '{}' but got '{}'", field, expected, actual)
            }
            ValidationError::MissingRequiredField { field } => {
                write!(f, "Required field '{}' is missing", field)
            }
            ValidationError::InvalidFieldValue { field, value, constraint } => {
                write!(f, "Field '{}' has invalid value '{}' (constraint: {})", field, value, constraint)
            }
            ValidationError::DuplicateId { id, location } => {
                write!(f, "Duplicate ID '{}' found in {}", id, location)
            }
            ValidationError::InvalidReference { field, reference, target_type } => {
                write!(f, "Field '{}' references unknown {} '{}'", field, target_type, reference)
            }
            ValidationError::SchemaStructure { message } => {
                write!(f, "Schema structure error: {}", message)
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
    panel_ids: HashSet<String>,
    layout_ids: HashSet<String>,
}

impl SchemaValidator {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            panel_ids: HashSet::new(),
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

        // Validate panels if present
        if let Some(panels) = &layout.children {
            for (idx, panel) in panels.iter().enumerate() {
                let _ = self.validate_panel(panel, &format!("{}.children[{}]", path, idx));
            }
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    /// Validate a single panel
    pub fn validate_panel(&mut self, panel: &Panel, path: &str) -> ValidationResult {
        // Validate required fields
        if panel.id.is_empty() {
            self.add_error(ValidationError::MissingRequiredField {
                field: format!("{}.id", path),
            });
        }

        // Validate position bounds
        self.validate_input_bounds_schema(&panel.position, &format!("{}.position", path));

        // Validate child panels recursively
        if let Some(children) = &panel.children {
            for (idx, child) in children.iter().enumerate() {
                let _ = self.validate_panel(child, &format!("{}.children[{}]", path, idx));
            }
        }

        // Validate script commands if present
        if let Some(scripts) = &panel.script {
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
        self.panel_ids.clear();
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
            self.collect_panel_ids_recursive(&layout.children, "panels");
        }
    }

    fn collect_panel_ids_recursive(&mut self, panels: &Option<Vec<Panel>>, location: &str) {
        if let Some(panel_list) = panels {
            for panel in panel_list {
                if !self.panel_ids.insert(panel.id.clone()) {
                    self.add_error(ValidationError::DuplicateId {
                        id: panel.id.clone(),
                        location: location.to_string(),
                    });
                }
                self.collect_panel_ids_recursive(&panel.children, location);
                
                // Check choice IDs if they exist
                if let Some(choices) = &panel.choices {
                    for choice in choices {
                        if !self.panel_ids.insert(choice.id.clone()) {
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
                message: "Multiple root layouts detected. Only one layout can be marked as 'root: true'.".to_string(),
            });
        }
    }

    fn validate_input_bounds_schema(&mut self, bounds: &crate::model::common::InputBounds, path: &str) {
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
    use crate::{App, Layout, Panel};

    fn create_test_panel(id: &str) -> Panel {
        Panel {
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
            children: Some(vec![create_test_panel("panel1")]),
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
        assert!(matches!(errors[0], ValidationError::MissingRequiredField { .. }));
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
        let config = Config { frame_delay: 0 };
        
        let result = validator.validate_config(&config);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ValidationError::InvalidFieldValue { .. }));
    }

    #[test]
    fn test_validate_config_excessive_frame_delay() {
        let mut validator = SchemaValidator::new();
        let config = Config { frame_delay: 2000 };
        
        let result = validator.validate_config(&config);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(matches!(errors[0], ValidationError::InvalidFieldValue { .. }));
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
                            "id": "panel1",
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
        assert!(matches!(errors[0], ValidationError::MissingRequiredField { .. }));
    }

    #[test]
    fn test_validate_empty_bounds() {
        let mut validator = SchemaValidator::new();
        let panel = Panel {
            id: "test_panel".to_string(),
            position: InputBounds {
                x1: "".to_string(),  // Empty bound
                y1: "20".to_string(),
                x2: "90".to_string(),
                y2: "80".to_string(),
            },
            ..Default::default()
        };
        
        let result = validator.validate_panel(&panel, "test_panel");
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| matches!(e, ValidationError::InvalidFieldValue { .. })));
    }

    #[test]
    fn test_validation_error_formatting() {
        // Test that ValidationError instances format correctly for user display
        let duplicate_error = ValidationError::DuplicateId {
            id: "panel1".to_string(),
            location: "panels".to_string(),
        };
        assert_eq!(duplicate_error.to_string(), "Duplicate ID 'panel1' found in panels");
        
        let missing_field_error = ValidationError::MissingRequiredField {
            field: "layouts".to_string(),
        };
        assert_eq!(missing_field_error.to_string(), "Required field 'layouts' is missing");
        
        let schema_error = ValidationError::SchemaStructure {
            message: "Multiple root layouts detected. Only one layout can be marked as 'root: true'.".to_string(),
        };
        assert_eq!(schema_error.to_string(), "Schema structure error: Multiple root layouts detected. Only one layout can be marked as 'root: true'.");
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
        layout2.root = Some(true);  // Second root - should cause error
        
        // Error 2: Duplicate panel IDs
        let panel1 = create_test_panel("panel1");
        let panel1_dup = create_test_panel("panel1");  // Duplicate ID
        layout1.children = Some(vec![panel1]);
        layout2.children = Some(vec![panel1_dup]);
        
        app.layouts = vec![layout1, layout2];
        
        let result = validator.validate_app(&app);
        assert!(result.is_err());
        
        let errors = result.unwrap_err();
        assert!(errors.len() >= 2, "Should have at least 2 validation errors");
        
        // Check for multiple root layouts error
        assert!(errors.iter().any(|e| matches!(e, ValidationError::SchemaStructure { .. })));
        
        // Check for duplicate ID error
        assert!(errors.iter().any(|e| matches!(e, ValidationError::DuplicateId { .. })));
    }
}