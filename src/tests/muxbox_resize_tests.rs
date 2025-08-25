// F0189/F0190: MuxBox Border Dragging and YAML Persistence Tests

#[cfg(test)]
mod tests {
    use crate::draw_loop::{calculate_new_bounds, detect_resize_edge, ResizeEdge};
    use crate::model::app::{save_muxbox_bounds_to_yaml, update_muxbox_bounds_recursive};
    use crate::model::common::InputBounds;
    use crate::tests::test_utils::TestDataFactory;
    use std::fs;
    use tempfile::NamedTempFile;

    #[test]
    fn test_resize_edge_detection() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test");
        muxbox.position = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "50%".to_string(),
            y2: "50%".to_string(),
        };

        // Debug the actual bounds
        let bounds = muxbox.bounds();
        println!(
            "MuxBox bounds: ({}, {}) to ({}, {})",
            bounds.x1, bounds.y1, bounds.x2, bounds.y2
        );

        // Test corner detection (only resize method supported)
        let corner = detect_resize_edge(&muxbox, bounds.x2 as u16, bounds.y2 as u16);
        assert_eq!(corner, Some(ResizeEdge::BottomRight));

        // Test corner tolerance - should work 1 pixel before the exact corner
        if bounds.x2 > 0 && bounds.y2 > 0 {
            let corner_near =
                detect_resize_edge(&muxbox, (bounds.x2 - 1) as u16, (bounds.y2 - 1) as u16);
            assert_eq!(corner_near, Some(ResizeEdge::BottomRight));
        }

        // Test that right edge no longer supports resize
        let right_edge = detect_resize_edge(
            &muxbox,
            bounds.x2 as u16,
            (bounds.y1 + bounds.height() / 2) as u16,
        );
        assert_eq!(right_edge, None); // Should be None - right edge resize removed

        // Test that bottom edge no longer supports resize
        let bottom_edge = detect_resize_edge(
            &muxbox,
            (bounds.x1 + bounds.width() / 2) as u16,
            bounds.y2 as u16,
        );
        assert_eq!(bottom_edge, None); // Should be None - bottom edge resize removed

        // Test no resize area (muxbox interior)
        let no_edge = detect_resize_edge(
            &muxbox,
            (bounds.x1 + bounds.width() / 4) as u16,
            (bounds.y1 + bounds.height() / 4) as u16,
        );
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
            50,
            50, // start position (corner)
            60,
            60, // current position (10 pixels right and down)
            100,
            100, // terminal size
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
      - id: "muxbox1"
        x1: "10%"
        y1: "10%"
        x2: "50%"
        y2: "50%"
        content: "Test muxbox"
      - id: "muxbox2"
        x1: "60%"
        y1: "10%"
        x2: "90%"
        y2: "50%"
        content: "Another muxbox"
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
        let result =
            save_muxbox_bounds_to_yaml(temp_file.path().to_str().unwrap(), "muxbox1", &new_bounds);
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
      - id: "parent_muxbox"
        x1: "0%"
        y1: "0%"
        x2: "100%"
        y2: "100%"
        children:
          - id: "nested_muxbox"
            x1: "20%"
            y1: "20%"
            x2: "80%"
            y2: "80%"
            content: "Nested muxbox"
"#;

        let mut yaml_value: serde_yaml::Value = serde_yaml::from_str(yaml_content).unwrap();
        let new_bounds = InputBounds {
            x1: "20%".to_string(),
            y1: "20%".to_string(),
            x2: "90%".to_string(),
            y2: "90%".to_string(),
        };

        let result = update_muxbox_bounds_recursive(&mut yaml_value, "nested_muxbox", &new_bounds);
        assert!(result.is_ok());
        assert!(result.unwrap());

        // Convert back to string and verify
        let updated_yaml = serde_yaml::to_string(&yaml_value).unwrap();
        assert!(updated_yaml.contains("x2: 90%"));
        assert!(updated_yaml.contains("y2: 90%"));
    }

    #[test]
    fn test_nonexistent_muxbox_bounds_update() {
        let yaml_content = r#"
layouts:
  - id: "test_layout"
    children:
      - id: "muxbox1"
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

        let result = update_muxbox_bounds_recursive(&mut yaml_value, "nonexistent", &new_bounds);
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false (not found)
    }

    // F0197: Minimum Box Resize Constraints Tests
    #[test]
    fn test_minimum_width_constraint() {
        let original_bounds = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "20%".to_string(),
            y2: "30%".to_string(),
        };

        // Try to resize to make box too narrow (less than 2 characters wide)
        let new_bounds = calculate_new_bounds(
            &original_bounds,
            &ResizeEdge::BottomRight,
            20,
            30, // start position
            12,
            40, // current position (8 pixels left, 10 pixels down)
            100,
            100, // terminal size 100x100
        );

        // With 100x100 terminal, 2 characters = 2% minimum width
        // Original x1=10%, so minimum x2 should be 12%
        let x2_percent: f32 = new_bounds.x2.replace('%', "").parse().unwrap();
        assert!(
            x2_percent >= 12.0,
            "Width constraint should prevent x2 < 12% (got {}%)",
            x2_percent
        );
    }

    #[test]
    fn test_minimum_height_constraint() {
        let original_bounds = InputBounds {
            x1: "10%".to_string(),
            y1: "20%".to_string(),
            x2: "50%".to_string(),
            y2: "30%".to_string(),
        };

        // Try to resize to make box too short (less than 2 characters tall)
        let new_bounds = calculate_new_bounds(
            &original_bounds,
            &ResizeEdge::BottomRight,
            50,
            30, // start position
            60,
            22, // current position (10 pixels right, 8 pixels up)
            100,
            100, // terminal size 100x100
        );

        // With 100x100 terminal, 2 characters = 2% minimum height
        // Original y1=20%, so minimum y2 should be 22%
        let y2_percent: f32 = new_bounds.y2.replace('%', "").parse().unwrap();
        assert!(
            y2_percent >= 22.0,
            "Height constraint should prevent y2 < 22% (got {}%)",
            y2_percent
        );
    }

    #[test]
    fn test_normal_resize_unaffected() {
        let original_bounds = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "50%".to_string(),
            y2: "50%".to_string(),
        };

        // Normal resize that doesn't violate constraints
        let new_bounds = calculate_new_bounds(
            &original_bounds,
            &ResizeEdge::BottomRight,
            50,
            50, // start position
            60,
            60, // current position (10 pixels right and down)
            100,
            100, // terminal size
        );

        // Should increase by 10% in both dimensions
        assert_eq!(new_bounds.x2, "60%");
        assert_eq!(new_bounds.y2, "60%");
        assert_eq!(new_bounds.x1, "10%"); // unchanged
        assert_eq!(new_bounds.y1, "10%"); // unchanged
    }

    #[test]
    fn test_both_constraints_simultaneously() {
        let original_bounds = InputBounds {
            x1: "40%".to_string(),
            y1: "30%".to_string(),
            x2: "45%".to_string(),
            y2: "35%".to_string(),
        };

        // Try to resize to violate both width and height constraints
        let new_bounds = calculate_new_bounds(
            &original_bounds,
            &ResizeEdge::BottomRight,
            45,
            35, // start position
            42,
            31, // current position (3 pixels left, 4 pixels up - shrinking)
            100,
            100, // terminal size
        );

        // Both dimensions should be constrained to minimums
        let x2_percent: f32 = new_bounds.x2.replace('%', "").parse().unwrap();
        let y2_percent: f32 = new_bounds.y2.replace('%', "").parse().unwrap();

        // Minimum x2 = x1 + 2% = 40% + 2% = 42%
        // Minimum y2 = y1 + 2% = 30% + 2% = 32%
        assert!(
            x2_percent >= 42.0,
            "Width should be constrained to minimum (got {}%)",
            x2_percent
        );
        assert!(
            y2_percent >= 32.0,
            "Height should be constrained to minimum (got {}%)",
            y2_percent
        );
    }

    #[test]
    fn test_different_terminal_sizes() {
        let original_bounds = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "15%".to_string(),
            y2: "15%".to_string(),
        };

        // Test with larger terminal (200x200) - minimum percentages should be smaller
        let new_bounds_large = calculate_new_bounds(
            &original_bounds,
            &ResizeEdge::BottomRight,
            30,
            30, // start position
            25,
            25, // shrink by 5 pixels in each direction
            200,
            200, // large terminal
        );

        // With 200x200 terminal, 2 characters = 1% minimum
        let x2_percent: f32 = new_bounds_large.x2.replace('%', "").parse().unwrap();
        let y2_percent: f32 = new_bounds_large.y2.replace('%', "").parse().unwrap();

        // Minimum should be x1 + 1% = 11%, y1 + 1% = 11%
        assert!(
            x2_percent >= 11.0,
            "Large terminal width constraint failed (got {}%)",
            x2_percent
        );
        assert!(
            y2_percent >= 11.0,
            "Large terminal height constraint failed (got {}%)",
            y2_percent
        );
    }
}
