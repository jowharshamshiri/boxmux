// F0189/F0190: Panel Border Dragging and YAML Persistence Tests

#[cfg(test)]
mod tests {
    use crate::draw_loop::{detect_resize_edge, calculate_new_bounds, ResizeEdge};
    use crate::model::app::{save_panel_bounds_to_yaml, update_panel_bounds_recursive};
    use crate::model::common::InputBounds;
    use crate::tests::test_utils::TestDataFactory;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_resize_edge_detection() {
        let mut panel = TestDataFactory::create_test_panel("test");
        panel.position = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "50%".to_string(),
            y2: "50%".to_string(),
        };

        // Debug the actual bounds
        let bounds = panel.bounds();
        println!("Panel bounds: ({}, {}) to ({}, {})", bounds.x1, bounds.y1, bounds.x2, bounds.y2);
        
        // Test corner detection (only resize method supported)
        let corner = detect_resize_edge(&panel, bounds.x2 as u16, bounds.y2 as u16);
        assert_eq!(corner, Some(ResizeEdge::BottomRight));

        // Test that right edge no longer supports resize
        let right_edge = detect_resize_edge(&panel, bounds.x2 as u16, (bounds.y1 + bounds.height()/2) as u16);
        assert_eq!(right_edge, None); // Should be None - right edge resize removed

        // Test that bottom edge no longer supports resize
        let bottom_edge = detect_resize_edge(&panel, (bounds.x1 + bounds.width()/2) as u16, bounds.y2 as u16);
        assert_eq!(bottom_edge, None); // Should be None - bottom edge resize removed

        // Test no resize area (panel interior)
        let no_edge = detect_resize_edge(&panel, (bounds.x1 + bounds.width()/4) as u16, (bounds.y1 + bounds.height()/4) as u16);
        assert_eq!(no_edge, None);
    }

    #[test]
    fn test_bounds_calculation() {
        let original_bounds = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "50%".to_string(),
            y2: "50%".to_string(),
        };

        // Test corner resize (only resize method supported)
        let new_bounds = calculate_new_bounds(
            &original_bounds,
            &ResizeEdge::BottomRight,
            50, 50, // start position (corner)
            60, 60, // current position (10 pixels right and down)
            100, 100, // terminal size
        );
        
        // Should increase both x2 and y2 by 10% (10 pixels out of 100)
        assert_eq!(new_bounds.x2, "60%"); // x2 increased by 10%
        assert_eq!(new_bounds.y2, "60%"); // y2 increased by 10%
        assert_eq!(new_bounds.x1, "10%"); // x1 unchanged
        assert_eq!(new_bounds.y1, "10%"); // y1 unchanged
    }

    #[test]
    fn test_yaml_bounds_update() {
        let yaml_content = r#"
layouts:
  - id: "test_layout"
    children:
      - id: "panel1"
        x1: "10%"
        y1: "10%"
        x2: "50%"
        y2: "50%"
        content: "Test panel"
      - id: "panel2"
        x1: "60%"
        y1: "10%"
        x2: "90%"
        y2: "50%"
        content: "Another panel"
"#;

        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");
        fs::write(&temp_file, yaml_content).expect("Failed to write temp file");

        let new_bounds = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "60%".to_string(),
            y2: "60%".to_string(),
        };

        // Test saving bounds
        let result = save_panel_bounds_to_yaml(
            temp_file.path().to_str().unwrap(),
            "panel1",
            &new_bounds,
        );
        assert!(result.is_ok());

        // Read back and verify
        let updated_content = fs::read_to_string(&temp_file).expect("Failed to read updated file");
        assert!(updated_content.contains("x2: 60%"));
        assert!(updated_content.contains("y2: 60%"));
    }

    #[test] 
    fn test_recursive_bounds_update() {
        let yaml_content = r#"
layouts:
  - id: "test_layout"
    children:
      - id: "parent_panel"
        x1: "0%"
        y1: "0%"
        x2: "100%"
        y2: "100%"
        children:
          - id: "nested_panel"
            x1: "20%"
            y1: "20%"
            x2: "80%"
            y2: "80%"
            content: "Nested panel"
"#;

        let mut yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content).unwrap();
        let new_bounds = InputBounds {
            x1: "20%".to_string(),
            y1: "20%".to_string(),
            x2: "90%".to_string(),
            y2: "90%".to_string(),
        };

        let result = update_panel_bounds_recursive(&mut yaml_value, "nested_panel", &new_bounds);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Convert back to string and verify
        let updated_yaml = serde_yaml::to_string(&yaml_value).unwrap();
        assert!(updated_yaml.contains("x2: 90%"));
        assert!(updated_yaml.contains("y2: 90%"));
    }

    #[test]
    fn test_nonexistent_panel_bounds_update() {
        let yaml_content = r#"
layouts:
  - id: "test_layout"
    children:
      - id: "panel1"
        x1: "10%"
        y1: "10%"
        x2: "50%"
        y2: "50%"
"#;

        let mut yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content).unwrap();
        let new_bounds = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "60%".to_string(),
            y2: "60%".to_string(),
        };

        let result = update_panel_bounds_recursive(&mut yaml_value, "nonexistent", &new_bounds);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false (not found)
    }
}