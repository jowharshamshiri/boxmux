#[cfg(test)]
mod hierarchical_variable_tests {
    use crate::model::app::load_app_from_yaml;
    use std::env;
    use std::fs;
    use tempfile::NamedTempFile;

    /// Test hierarchical variable precedence: child panel overrides parent panel
    #[test]
    fn test_child_panel_variables_override_parent_panel() {
        let yaml_content = r#"
app:
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'parent_panel'
          title: 'Parent Panel'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          variables:
            SHARED_VAR: "parent_value"
            PARENT_ONLY: "parent_specific"
          content: 'Parent sees: ${SHARED_VAR}'
          children:
            - id: 'child_panel'
              title: 'Child Panel'
              position: {x1: 20%, y1: 20%, x2: 80%, y2: 80%}
              variables:
                SHARED_VAR: "child_value"  # Should override parent
                CHILD_ONLY: "child_specific"
              content: 'Child sees: ${SHARED_VAR}, Parent var: ${PARENT_ONLY}'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let parent_panel = &app.layouts[0].children.as_ref().unwrap()[0];
        let child_panel = &parent_panel.children.as_ref().unwrap()[0];

        // Parent should see its own variable value
        let parent_content = parent_panel.content.as_ref().unwrap();
        assert!(
            parent_content.contains("parent_value"),
            "FAILED: Parent panel should see its own variable value. Got: '{}'",
            parent_content
        );

        // Child should see its own override value + inherit parent-only value
        let child_content = child_panel.content.as_ref().unwrap();
        assert!(
            child_content.contains("child_value"),
            "FAILED: Child should override parent variable. Got: '{}'",
            child_content
        );
        assert!(
            child_content.contains("parent_specific"),
            "FAILED: Child should inherit parent-only variables. Got: '{}'",
            child_content
        );
    }

    /// Test that environment variables override all levels of hierarchy
    #[test]
    fn test_environment_overrides_all_hierarchy_levels() {
        env::set_var("HIERARCHY_TEST_VAR", "env_wins");

        let yaml_content = r#"
app:
  variables:
    HIERARCHY_TEST_VAR: "app_level"
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'parent_panel'
          title: 'Parent Panel'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          variables:
            HIERARCHY_TEST_VAR: "parent_level"
          content: 'Parent: ${HIERARCHY_TEST_VAR}'
          children:
            - id: 'child_panel'
              title: 'Child Panel' 
              position: {x1: 20%, y1: 20%, x2: 80%, y2: 80%}
              variables:
                HIERARCHY_TEST_VAR: "child_level"
              content: 'Child: ${HIERARCHY_TEST_VAR}'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let parent_panel = &app.layouts[0].children.as_ref().unwrap()[0];
        let child_panel = &parent_panel.children.as_ref().unwrap()[0];

        // YAML-defined variables should override environment for granular control
        let parent_content = parent_panel.content.as_ref().unwrap();
        assert!(
            parent_content.contains("parent_level"),
            "FAILED: Parent YAML variable should override environment. Got: '{}'",
            parent_content
        );

        let child_content = child_panel.content.as_ref().unwrap();
        assert!(
            child_content.contains("child_level"),
            "FAILED: Child YAML variable should override environment and parent. Got: '{}'",
            child_content
        );

        env::remove_var("HIERARCHY_TEST_VAR");
    }

    /// Test multi-level hierarchy (grandparent -> parent -> child)
    #[test]
    fn test_three_level_variable_hierarchy() {
        let yaml_content = r#"
app:
  variables:
    APP_VAR: "app_level"
  layouts:
    - id: 'main'
      root: true
      title: 'Test'  
      children:
        - id: 'grandparent'
          title: 'Grandparent'
          position: {x1: 5%, y1: 5%, x2: 95%, y2: 95%}
          variables:
            HIERARCHY_VAR: "grandparent_value"
            GRANDPARENT_ONLY: "grandparent_specific"
          children:
            - id: 'parent'
              title: 'Parent'
              position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
              variables:
                HIERARCHY_VAR: "parent_value"  # Overrides grandparent
                PARENT_ONLY: "parent_specific"
              children:
                - id: 'child'
                  title: 'Child'
                  position: {x1: 20%, y1: 20%, x2: 80%, y2: 80%}
                  variables:
                    HIERARCHY_VAR: "child_value"  # Overrides parent & grandparent
                    CHILD_ONLY: "child_specific"
                  content: 'Child: ${HIERARCHY_VAR}, Parent: ${PARENT_ONLY}, Grandparent: ${GRANDPARENT_ONLY}, App: ${APP_VAR}'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let grandparent = &app.layouts[0].children.as_ref().unwrap()[0];
        let parent = &grandparent.children.as_ref().unwrap()[0];
        let child = &parent.children.as_ref().unwrap()[0];

        let child_content = child.content.as_ref().unwrap();

        // Child should see its own override + inherit up the hierarchy + app level
        assert!(
            child_content.contains("child_value"),
            "FAILED: Child should use its own variable override. Got: '{}'",
            child_content
        );
        assert!(
            child_content.contains("parent_specific"),
            "FAILED: Child should inherit parent-only variable. Got: '{}'",
            child_content
        );
        assert!(
            child_content.contains("grandparent_specific"),
            "FAILED: Child should inherit grandparent-only variable. Got: '{}'",
            child_content
        );
        assert!(
            child_content.contains("app_level"),
            "FAILED: Child should inherit app-level variable. Got: '{}'",
            child_content
        );
    }

    /// Test that variables work in all fields at all hierarchy levels
    #[test]
    fn test_hierarchical_variables_in_all_fields() {
        let yaml_content = r#"
app:
  variables:
    GLOBAL_PREFIX: "APP"
  layouts:
    - id: 'main'
      root: true
      title: '${GLOBAL_PREFIX} Layout'
      children:
        - id: 'parent_panel'
          title: '${GLOBAL_PREFIX} Parent'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          variables:
            LEVEL: "PARENT"
            OUTPUT_TARGET: "child_panel"
          content: 'Parent Level: ${LEVEL}'
          script:
            - "echo 'Parent script: ${LEVEL}'"
          redirect_output: '${OUTPUT_TARGET}'
          children:
            - id: 'child_panel'
              title: '${GLOBAL_PREFIX} Child'
              position: {x1: 20%, y1: 20%, x2: 80%, y2: 80%}
              variables:
                LEVEL: "CHILD"  # Overrides parent LEVEL
              content: 'Child Level: ${LEVEL}, Global: ${GLOBAL_PREFIX}'
              script:
                - "echo 'Child script: ${LEVEL} under ${GLOBAL_PREFIX}'"
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let layout = &app.layouts[0];
        let parent = &layout.children.as_ref().unwrap()[0];
        let child = &parent.children.as_ref().unwrap()[0];

        // Test layout title uses app variable
        assert_eq!(
            layout.title.as_ref().unwrap(),
            "APP Layout",
            "FAILED: Layout title should use app variable"
        );

        // Test parent uses its own LEVEL variable
        assert!(
            parent.content.as_ref().unwrap().contains("PARENT"),
            "FAILED: Parent should use its own LEVEL variable"
        );
        assert!(
            parent.script.as_ref().unwrap()[0].contains("PARENT"),
            "FAILED: Parent script should use its own variable"
        );
        assert_eq!(
            parent.redirect_output.as_ref().unwrap(),
            "child_panel",
            "FAILED: Parent redirect_output should use variable substitution"
        );

        // Test child overrides LEVEL but inherits GLOBAL_PREFIX
        let child_content = child.content.as_ref().unwrap();
        assert!(
            child_content.contains("CHILD") && child_content.contains("APP"),
            "FAILED: Child should override LEVEL but inherit GLOBAL_PREFIX. Got: '{}'",
            child_content
        );

        let child_script = &child.script.as_ref().unwrap()[0];
        assert!(
            child_script.contains("CHILD") && child_script.contains("APP"),
            "FAILED: Child script should use overridden and inherited variables. Got: '{}'",
            child_script
        );
    }

    /// Test variable inheritance with missing intermediate levels
    #[test]
    fn test_variable_inheritance_skips_missing_levels() {
        let yaml_content = r#"
app:
  variables:
    APP_VAR: "from_app"
    SHARED_VAR: "app_shared"
  layouts:
    - id: 'main'
      root: true
      title: 'Test'
      children:
        - id: 'parent_no_vars'
          title: 'Parent Without Variables'
          position: {x1: 10%, y1: 10%, x2: 90%, y2: 90%}
          # No variables defined at this level
          children:
            - id: 'child_with_vars'
              title: 'Child With Variables'
              position: {x1: 20%, y1: 20%, x2: 80%, y2: 80%}
              variables:
                SHARED_VAR: "child_override"
                CHILD_ONLY: "child_specific"
              content: 'App: ${APP_VAR}, Shared: ${SHARED_VAR}, Child: ${CHILD_ONLY}'
"#;

        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let result = load_app_from_yaml(temp_file.path().to_str().unwrap());
        assert!(result.is_ok(), "YAML loading failed: {:?}", result.err());

        let app = result.unwrap();
        let parent = &app.layouts[0].children.as_ref().unwrap()[0];
        let child = &parent.children.as_ref().unwrap()[0];

        let child_content = child.content.as_ref().unwrap();

        // Child should inherit app variable (skipping parent level with no variables)
        assert!(
            child_content.contains("from_app"),
            "FAILED: Child should inherit app variable despite parent having no variables. Got: '{}'", child_content
        );

        // Child should override shared variable
        assert!(
            child_content.contains("child_override"),
            "FAILED: Child should override app-level shared variable. Got: '{}'",
            child_content
        );

        // Child should have its own variable
        assert!(
            child_content.contains("child_specific"),
            "FAILED: Child should have access to its own variables. Got: '{}'",
            child_content
        );
    }
}
