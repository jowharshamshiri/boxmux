#[cfg(test)]
mod integration_yaml_tests {
    use crate::model::app::load_app_from_yaml;
    use std::env;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_yaml_loading_with_variables() {
        let yaml_content = r#"
app:
  variables:
    TEST_VAR: "test_value"
  layouts:
    - id: 'main'
      root: true
      title: '${TEST_VAR} Dashboard'
      children:
        - id: 'panel1'
          title: 'Panel'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          variables:
            LOCAL_VAR: "local_value"
          content: 'Global: ${TEST_VAR}, Local: ${LOCAL_VAR}'
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());

        assert!(
            result.is_ok(),
            "YAML loading should succeed with variables: {:?}",
            result.err()
        );

        let app = result.unwrap();
        assert!(app.variables.is_some());
        assert_eq!(
            app.variables.as_ref().unwrap().get("TEST_VAR"),
            Some(&"test_value".to_string())
        );

        let panel = &app.layouts[0].children.as_ref().unwrap()[0];
        assert!(panel.variables.is_some());
        assert_eq!(
            panel.variables.as_ref().unwrap().get("LOCAL_VAR"),
            Some(&"local_value".to_string())
        );
    }

    #[test]
    fn test_yaml_loading_with_env_variables() {
        env::set_var("YAML_TEST_VAR", "env_test_value");

        let yaml_content = r#"
app:
  layouts:
    - id: 'main'
      root: true
      title: '$YAML_TEST_VAR Dashboard'
      children:
        - id: 'panel1'
          title: 'Panel'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'Env var: $YAML_TEST_VAR'
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(
            result.is_ok(),
            "YAML loading should succeed with env variables: {:?}",
            result.err()
        );

        env::remove_var("YAML_TEST_VAR");
    }

    #[test]
    fn test_yaml_loading_with_defaults() {
        let yaml_content = r#"
app:
  layouts:
    - id: 'main'
      root: true
      title: '${MISSING_VAR:Default Title}'
      children:
        - id: 'panel1'
          title: 'Panel'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'Value: ${MISSING_VAR:fallback_value}'
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(
            result.is_ok(),
            "YAML loading should succeed with default values: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_schema_validation_accepts_variables() {
        let yaml_content = r#"
app:
  variables:
    APP_NAME: "Test App"
    VERSION: "1.0"
  layouts:
    - id: 'main'
      root: true
      title: '${APP_NAME}'
      children:
        - id: 'panel1'
          title: 'Panel'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          variables:
            PANEL_VAR: "panel_value"
          content: 'App: ${APP_NAME}, Panel: ${PANEL_VAR}'
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        // This test specifically verifies that JSON schema validation passes
        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(
            result.is_ok(),
            "Schema validation should accept variables field: {:?}",
            result.err()
        );
    }

    #[test]
    fn test_schema_validation_rejects_invalid_variables() {
        let yaml_content = r#"
app:
  variables: "invalid_not_object"
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'panel1'
          title: 'Panel'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'Test'
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(
            result.is_err(),
            "Schema validation should reject invalid variables field"
        );

        let error_msg = result.err().unwrap().to_string();
        assert!(
            error_msg.contains("JSON Schema validation failed"),
            "Error should mention schema validation failure: {}",
            error_msg
        );
    }

    #[test]
    fn test_full_yaml_pipeline_integration() {
        env::set_var("INTEGRATION_TEST_VAR", "integration_value");

        let yaml_content = r#"
app:
  variables:
    GLOBAL_VAR: "global_value"
  layouts:
    - id: 'integration_test'
      root: true  
      title: 'Integration Test: ${GLOBAL_VAR}'
      bg_color: 'black'
      fg_color: 'white'
      children:
        - id: 'test_panel'
          title: 'Test Panel'
          position: {x1: 5%, y1: 5%, x2: 95%, y2: 95%}
          border: true
          variables:
            LOCAL_VAR: "local_value"
          script:
            - "echo Global: ${GLOBAL_VAR}"
            - "echo Local: ${LOCAL_VAR}"
            - "echo Env: $INTEGRATION_TEST_VAR"
            - "echo Default: ${MISSING:default_val}"
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        // Test complete pipeline: variable substitution + schema validation + app loading
        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(
            result.is_ok(),
            "Full YAML pipeline should work: {:?}",
            result.err()
        );

        let app = result.unwrap();

        // Verify structure is correct
        assert_eq!(app.layouts.len(), 1);
        assert_eq!(app.layouts[0].id, "integration_test");
        assert!(app.layouts[0].children.is_some());
        assert_eq!(app.layouts[0].children.as_ref().unwrap().len(), 1);

        // Verify variables are preserved
        assert!(app.variables.is_some());
        let panel = &app.layouts[0].children.as_ref().unwrap()[0];
        assert!(panel.variables.is_some());

        env::remove_var("INTEGRATION_TEST_VAR");
    }
}
