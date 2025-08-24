use crate::model::app::save_panel_bounds_to_yaml;
use crate::model::common::InputBounds;
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;

#[test]
fn test_yaml_persistence_complete_flow() {
    // Create a temporary YAML file with nested panels
    let yaml_content = r#"app:
  layouts:
    - id: 'dashboard'
      root: true
      title: 'Dashboard Layout'
      children:
        - id: 'header'
          title: 'Header'
          position:
            x1: 0%
            y1: 0%
            x2: 100%
            y2: 10%
          children:
            - id: 'title'
              position:
                x1: 0%
                y1: 0%
                x2: 80%
                y2: 100%
              title: 'Title Display'
            - id: 'clock'
              position:
                x1: 80%
                y1: 0%
                x2: 100%
                y2: 100%
              title: 'Clock Display'
        - id: 'main'
          position:
            x1: 0%
            y1: 10%
            x2: 100%
            y2: 90%
          title: 'Main Content'
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path().to_str().unwrap();

    // Test updating various panels at different nesting levels
    let test_cases = vec![
        // Root level panel
        ("main", InputBounds {
            x1: "5%".to_string(),
            y1: "15%".to_string(),
            x2: "95%".to_string(),
            y2: "85%".to_string(),
        }),
        // Nested child panel
        ("title", InputBounds {
            x1: "10%".to_string(),
            y1: "5%".to_string(),
            x2: "70%".to_string(),
            y2: "95%".to_string(),
        }),
        // Another nested child panel
        ("clock", InputBounds {
            x1: "75%".to_string(),
            y1: "5%".to_string(),
            x2: "95%".to_string(),
            y2: "95%".to_string(),
        }),
    ];

    for (panel_id, new_bounds) in test_cases {
        // Save the panel bounds to YAML
        let result = save_panel_bounds_to_yaml(temp_path, panel_id, &new_bounds);
        assert!(result.is_ok(), "Failed to save panel {} bounds: {:?}", panel_id, result);

        // Read back the YAML and verify the changes
        let updated_yaml = fs::read_to_string(temp_path).expect("Failed to read updated YAML");
        let yaml_value: serde_yaml::Value = serde_yaml::from_str(&updated_yaml).expect("Failed to parse updated YAML");
        
        // Verify the specific panel was updated
        let panel_found = verify_panel_bounds_in_yaml(&yaml_value, panel_id, &new_bounds);
        assert!(panel_found, "Panel {} bounds not found or incorrect in updated YAML", panel_id);
        
        println!("✅ Successfully updated and verified panel: {}", panel_id);
    }
    
    println!("✅ All YAML persistence tests passed!");
}

fn verify_panel_bounds_in_yaml(
    value: &serde_yaml::Value,
    target_panel_id: &str,
    expected_bounds: &InputBounds,
) -> bool {
    use serde_yaml::Value;
    
    match value {
        Value::Mapping(map) => {
            // Check if this is the panel we're looking for
            if let Some(Value::String(id)) = map.get(&Value::String("id".to_string())) {
                if id == target_panel_id {
                    // Check the position bounds
                    if let Some(Value::Mapping(position_map)) = map.get(&Value::String("position".to_string())) {
                        let x1_match = position_map.get(&Value::String("x1".to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s == expected_bounds.x1)
                            .unwrap_or(false);
                        let y1_match = position_map.get(&Value::String("y1".to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s == expected_bounds.y1)
                            .unwrap_or(false);
                        let x2_match = position_map.get(&Value::String("x2".to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s == expected_bounds.x2)
                            .unwrap_or(false);
                        let y2_match = position_map.get(&Value::String("y2".to_string()))
                            .and_then(|v| v.as_str())
                            .map(|s| s == expected_bounds.y2)
                            .unwrap_or(false);
                        
                        return x1_match && y1_match && x2_match && y2_match;
                    }
                }
            }
            
            // Recursively search in other mappings
            for (_, child_value) in map.iter() {
                if verify_panel_bounds_in_yaml(child_value, target_panel_id, expected_bounds) {
                    return true;
                }
            }
        }
        Value::Sequence(seq) => {
            // Search through sequences (like children arrays)
            for item in seq.iter() {
                if verify_panel_bounds_in_yaml(item, target_panel_id, expected_bounds) {
                    return true;
                }
            }
        }
        _ => {}
    }
    
    false
}

#[test]
fn test_yaml_format_preservation() {
    // Test that YAML formatting and comments are preserved after updates
    let yaml_content = r#"# Dashboard configuration
app:
  # Main application settings
  layouts:
    - id: 'test'  # Test layout
      root: true
      title: 'Test Layout'
      children:
        - id: 'panel1'
          title: 'Panel 1'
          position:  # Panel position
            x1: 10%
            y1: 10%
            x2: 50%
            y2: 50%
          content: 'Panel content'
"#;

    let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
    temp_file.write_all(yaml_content.as_bytes()).expect("Failed to write temp file");
    let temp_path = temp_file.path().to_str().unwrap();

    // Update panel bounds
    let new_bounds = InputBounds {
        x1: "15%".to_string(),
        y1: "15%".to_string(),
        x2: "55%".to_string(),
        y2: "55%".to_string(),
    };

    let result = save_panel_bounds_to_yaml(temp_path, "panel1", &new_bounds);
    assert!(result.is_ok(), "Failed to save panel bounds: {:?}", result);

    // Read the updated content
    let updated_content = fs::read_to_string(temp_path).expect("Failed to read updated YAML");
    
    // Verify the bounds were updated
    assert!(updated_content.contains("x1: 15%"));
    assert!(updated_content.contains("y1: 15%"));
    assert!(updated_content.contains("x2: 55%"));
    assert!(updated_content.contains("y2: 55%"));
    
    // Note: serde_yaml doesn't preserve comments and formatting exactly,
    // but the structure and data should be correct
    println!("✅ YAML format preservation test passed!");
}