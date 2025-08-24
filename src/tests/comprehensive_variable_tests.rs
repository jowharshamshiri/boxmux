#[cfg(test)]
mod comprehensive_variable_tests {
    use crate::model::app::load_app_from_yaml;
    use std::env;
    use std::fs;
    use tempfile::NamedTempFile;

    /// Test that would have caught the "variables ignored" issue
    #[test]
    fn test_app_variables_actually_substituted_in_content() {
        let yaml_content = r#"
app:
  variables:
    TEST_APP_VAR: "app_variable_value"
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'muxbox1'
          title: 'MuxBox'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'App var should resolve to: ${TEST_APP_VAR}'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let muxbox = &app.layouts[0].children.as_ref().unwrap()[0];

        // This test SHOULD FAIL with current implementation
        // MuxBox content should have substituted variable, not literal text
        let content = muxbox.content.as_ref().unwrap();
        assert!(
            content.contains("app_variable_value"),
            "FAILED: App variable not substituted in muxbox content. Got: '{}'",
            content
        );
        assert!(
            !content.contains("${TEST_APP_VAR}"),
            "FAILED: Variable placeholder still present. Got: '{}'",
            content
        );
    }

    /// Test that would have caught the "muxbox variables ignored" issue  
    #[test]
    fn test_muxbox_variables_actually_substituted_in_scripts() {
        let yaml_content = r#"
app:
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'muxbox1'
          title: 'MuxBox'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          variables:
            MUXBOX_LOCAL_VAR: "muxbox_specific_value"
          script:
            - "echo 'MuxBox var: ${MUXBOX_LOCAL_VAR}'"
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let muxbox = &app.layouts[0].children.as_ref().unwrap()[0];

        // This test SHOULD FAIL with current implementation
        let script_command = &muxbox.script.as_ref().unwrap()[0];
        assert!(
            script_command.contains("muxbox_specific_value"),
            "FAILED: MuxBox variable not substituted in script. Got: '{}'",
            script_command
        );
        assert!(
            !script_command.contains("${MUXBOX_LOCAL_VAR}"),
            "FAILED: Variable placeholder still present. Got: '{}'",
            script_command
        );
    }

    /// Test that would have caught the "variable precedence" issue
    #[test]
    fn test_variable_precedence_order() {
        // Set environment variable
        env::set_var("PRECEDENCE_TEST_VAR", "env_value");

        let yaml_content = r#"
app:
  variables:
    PRECEDENCE_TEST_VAR: "app_value"
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'muxbox1'
          title: 'MuxBox'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          variables:
            PRECEDENCE_TEST_VAR: "muxbox_value"
          content: 'Value: ${PRECEDENCE_TEST_VAR}'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let muxbox = &app.layouts[0].children.as_ref().unwrap()[0];

        // MuxBox-local should win over app and environment
        let content = muxbox.content.as_ref().unwrap();
        assert!(
            content.contains("muxbox_value"),
            "FAILED: MuxBox-local variable should have highest precedence. Got: '{}'",
            content
        );
        assert!(
            !content.contains("app_value") && !content.contains("env_value"),
            "FAILED: Wrong precedence - should use muxbox value. Got: '{}'",
            content
        );

        env::remove_var("PRECEDENCE_TEST_VAR");
    }

    /// Test that would have caught "nested variables" issue
    #[test]
    fn test_nested_variables_not_supported() {
        let yaml_content = r#"
app:
  variables:
    DEFAULT_USER: "fallback_user"
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'muxbox1'
          title: 'MuxBox'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'User: ${USER:${DEFAULT_USER}}'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());

        // This should either:
        // 1. Fail gracefully with a clear error message, OR
        // 2. Handle nested variables correctly
        // Currently it produces malformed output

        if result.is_ok() {
            let app = result.unwrap();
            let muxbox = &app.layouts[0].children.as_ref().unwrap()[0];
            let content = muxbox.content.as_ref().unwrap();

            // Should not contain malformed substitution
            assert!(
                !content.contains("${DEFAULT_USER}}"),
                "FAILED: Malformed nested variable substitution. Got: '{}'",
                content
            );

            // Should either substitute correctly or show clear error
            assert!(
                content.contains("fallback_user")
                    || content.contains("error")
                    || content.contains("invalid"),
                "FAILED: Nested variables should be handled gracefully. Got: '{}'",
                content
            );
        } else {
            // If it fails, error should be descriptive
            let error = result.err().unwrap().to_string();
            assert!(
                error.contains("nested") || error.contains("variable") || error.contains("syntax"),
                "FAILED: Error message should be descriptive for nested variables. Got: '{}'",
                error
            );
        }
    }

    /// Test that would have caught "variables only work via environment" issue
    #[test]
    fn test_yaml_variables_work_without_environment() {
        // Explicitly clear environment to ensure we're not relying on it
        let vars_to_clear = ["TEST_YAML_ONLY", "APP_NAME", "MUXBOX_VAR"];
        for var in &vars_to_clear {
            env::remove_var(var);
        }

        let yaml_content = r#"
app:
  variables:
    TEST_YAML_ONLY: "yaml_only_value"
    APP_NAME: "Pure YAML App"
  layouts:
    - id: 'main'
      root: true
      title: '${APP_NAME}'
      children:
        - id: 'muxbox1'
          title: 'MuxBox'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          variables:
            MUXBOX_VAR: "muxbox_only_value"
          content: 'App: ${TEST_YAML_ONLY}, MuxBox: ${MUXBOX_VAR}'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();

        // Test layout title substitution
        assert_eq!(
            app.layouts[0].title.as_ref().unwrap(),
            "Pure YAML App",
            "FAILED: App variable not substituted in layout title"
        );

        // Test muxbox content substitution
        let muxbox = &app.layouts[0].children.as_ref().unwrap()[0];
        let content = muxbox.content.as_ref().unwrap();

        assert!(
            content.contains("yaml_only_value") && content.contains("muxbox_only_value"),
            "FAILED: YAML variables not substituted without environment. Got: '{}'",
            content
        );
    }

    /// Test that would have caught "defaults not working" issue
    #[test]
    fn test_default_values_actually_used() {
        // Clear environment to ensure defaults are used
        env::remove_var("MISSING_VAR");
        env::remove_var("ANOTHER_MISSING_VAR");

        let yaml_content = r#"
app:
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'muxbox1'
          title: 'MuxBox'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          script:
            - "echo 'Value1: ${MISSING_VAR:default_one}'"
            - "echo 'Value2: ${ANOTHER_MISSING_VAR:default_two}'"
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let muxbox = &app.layouts[0].children.as_ref().unwrap()[0];
        let script = muxbox.script.as_ref().unwrap();

        assert!(
            script[0].contains("default_one"),
            "FAILED: Default value not used for first missing var. Got: '{}'",
            script[0]
        );
        assert!(
            script[1].contains("default_two"),
            "FAILED: Default value not used for second missing var. Got: '{}'",
            script[1]
        );

        // Should not contain variable placeholders
        assert!(
            !script[0].contains("${MISSING_VAR") && !script[1].contains("${ANOTHER_MISSING_VAR"),
            "FAILED: Variable placeholders still present in scripts"
        );
    }

    /// Test that would have caught "script vs content vs title" substitution gaps
    #[test]
    fn test_variables_substituted_in_all_fields() {
        let yaml_content = r#"
app:
  variables:
    GLOBAL_VAR: "global_value"
  layouts:
    - id: 'main'
      root: true
      title: 'Layout: ${GLOBAL_VAR}'
      children:
        - id: 'muxbox1'
          title: 'MuxBox: ${GLOBAL_VAR}'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'Content: ${GLOBAL_VAR}'
          script:
            - "echo 'Script: ${GLOBAL_VAR}'"
          redirect_output: 'muxbox2'
        - id: 'muxbox2'
          title: 'Output MuxBox'
          position: {x1: 10%, y1: 50%, x2: 90%, y2: 90%}
          content: 'Redirect target'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let layout = &app.layouts[0];
        let muxbox1 = &layout.children.as_ref().unwrap()[0];

        // Test substitution in all field types
        assert_eq!(
            layout.title.as_ref().unwrap(),
            "Layout: global_value",
            "FAILED: Variable not substituted in layout title"
        );

        assert_eq!(
            muxbox1.title.as_ref().unwrap(),
            "MuxBox: global_value",
            "FAILED: Variable not substituted in muxbox title"
        );

        assert_eq!(
            muxbox1.content.as_ref().unwrap(),
            "Content: global_value",
            "FAILED: Variable not substituted in muxbox content"
        );

        let script = muxbox1.script.as_ref().unwrap();
        assert!(
            script[0].contains("global_value"),
            "FAILED: Variable not substituted in script. Got: '{}'",
            script[0]
        );
    }

    /// Test that would have caught "empty defaults" edge case
    #[test]
    fn test_empty_default_values() {
        env::remove_var("EMPTY_DEFAULT_VAR");

        let yaml_content = r#"
app:
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'muxbox1'
          title: 'MuxBox'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'Before${EMPTY_DEFAULT_VAR:}After'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let muxbox = &app.layouts[0].children.as_ref().unwrap()[0];
        let content = muxbox.content.as_ref().unwrap();

        assert_eq!(
            content, "BeforeAfter",
            "FAILED: Empty default value not handled correctly. Got: '{}'",
            content
        );
    }

    /// Test that would have caught "malformed regex" or "special characters" issues
    #[test]
    fn test_special_characters_in_variable_values() {
        let yaml_content = r#"
app:
  variables:
    SPECIAL_CHARS: "value with spaces, $pecial chars & symbols!"
    PATH_LIKE: "/usr/bin:/usr/local/bin"
    QUOTED_VALUE: '"quoted string"'
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'muxbox1'
          title: 'MuxBox'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          content: 'Special: ${SPECIAL_CHARS}, Path: ${PATH_LIKE}, Quoted: ${QUOTED_VALUE}'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let muxbox = &app.layouts[0].children.as_ref().unwrap()[0];
        let content = muxbox.content.as_ref().unwrap();

        assert!(
            content.contains("value with spaces, $pecial chars & symbols!"),
            "FAILED: Special characters not preserved. Got: '{}'",
            content
        );
        assert!(
            content.contains("/usr/bin:/usr/local/bin"),
            "FAILED: Path-like value not preserved. Got: '{}'",
            content
        );
        assert!(
            content.contains("\"quoted string\""),
            "FAILED: Quoted value not preserved. Got: '{}'",
            content
        );
    }
}
