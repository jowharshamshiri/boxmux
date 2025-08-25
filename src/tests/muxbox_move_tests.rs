#[cfg(test)]
mod tests {
    use crate::draw_loop::{calculate_new_position, detect_move_area};
    use crate::model::common::InputBounds;
    use crate::tests::test_utils::TestDataFactory;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_detect_move_area_title_top_border() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");

        // Set muxbox bounds: x1=10%, y1=10%, x2=50%, y2=40%
        muxbox.position = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "50%".to_string(),
            y2: "40%".to_string(),
        };

        // Convert percentages to actual coordinates
        let bounds = muxbox.bounds();
        println!(
            "Actual bounds: x1={}, y1={}, x2={}, y2={}",
            bounds.x1, bounds.y1, bounds.x2, bounds.y2
        );
        // Use the actual calculated bounds for testing
        let y1 = bounds.y1;
        let x1 = bounds.x1;
        let x2 = bounds.x2;

        // Test clicking on title/top border (y1 coordinate)
        assert!(detect_move_area(
            &muxbox,
            (x1 + (x2 - x1) / 2) as u16,
            y1 as u16
        )); // Middle of top border
        assert!(detect_move_area(&muxbox, x1 as u16, y1 as u16)); // Left edge of top border
        assert!(detect_move_area(&muxbox, x2 as u16, y1 as u16)); // Right edge of top border

        // Test clicking outside move area
        assert!(!detect_move_area(
            &muxbox,
            (x1 + (x2 - x1) / 2) as u16,
            (y1 + 1) as u16
        )); // Below top border
        assert!(!detect_move_area(&muxbox, (x1 - 1) as u16, y1 as u16)); // Left of muxbox
        assert!(!detect_move_area(&muxbox, (x2 + 1) as u16, y1 as u16)); // Right of muxbox
        assert!(!detect_move_area(
            &muxbox,
            (x1 + (x2 - x1) / 2) as u16,
            (y1 - 1) as u16
        )); // Above top border
    }

    #[test]
    fn test_calculate_new_position_movement() {
        let original_bounds = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "50%".to_string(),
            y2: "40%".to_string(),
        };

        // Test moving right by 10 pixels (10% in 100-width terminal)
        let new_position = calculate_new_position(
            &original_bounds,
            20,
            10, // start_x, start_y
            30,
            15,  // current_x (+10), current_y (+5)
            100, // terminal_width
            50,  // terminal_height
        );

        // Should move right by 10% and down by 10% (5/50 * 100 = 10%)
        assert_eq!(new_position.x1, "20%"); // 10% + 10% = 20%
        assert_eq!(new_position.x2, "60%"); // 50% + 10% = 60%
        assert_eq!(new_position.y1, "20%"); // 10% + 10% = 20%
        assert_eq!(new_position.y2, "50%"); // 40% + 10% = 50%
    }

    #[test]
    fn test_calculate_new_position_boundary_constraints() {
        let original_bounds = InputBounds {
            x1: "80%".to_string(),
            y1: "80%".to_string(),
            x2: "95%".to_string(),
            y2: "95%".to_string(),
        };

        // Test moving beyond right boundary
        let new_position = calculate_new_position(
            &original_bounds,
            90,
            40, // start position
            120,
            45,  // try to move way right and down
            100, // terminal_width
            50,  // terminal_height
        );

        // Should constrain to not go beyond 100%
        assert_eq!(new_position.x2, "100%"); // Capped at 100%
        assert_eq!(new_position.y2, "100%"); // Capped at 100%

        // x1 and y1 should maintain muxbox size
        let x1_val: f32 = new_position.x1.replace('%', "").parse().unwrap();
        let y1_val: f32 = new_position.y1.replace('%', "").parse().unwrap();
        assert!(x1_val >= 0.0); // Should not go negative
        assert!(y1_val >= 0.0); // Should not go negative
    }

    #[test]
    fn test_yaml_position_persistence() {
        let mut temp_file = NamedTempFile::new().expect("Failed to create temp file");

        let yaml_content = r#"
layouts:
  - name: test_layout
    children:
      - id: test_muxbox
        position:
          x1: 10%
          y1: 10%
          x2: 50%
          y2: 40%
        content: Test content
"#;

        temp_file
            .write_all(yaml_content.as_bytes())
            .expect("Failed to write YAML");
        temp_file.flush().expect("Failed to flush temp file");

        let yaml_path = temp_file.path().to_str().unwrap();

        // Test the YAML persistence function (reuse from muxbox resize)
        let new_bounds = InputBounds {
            x1: "20%".to_string(),
            y1: "15%".to_string(),
            x2: "60%".to_string(),
            y2: "45%".to_string(),
        };

        let result =
            crate::model::app::save_muxbox_bounds_to_yaml(yaml_path, "test_muxbox", &new_bounds);
        assert!(
            result.is_ok(),
            "Failed to save muxbox position: {:?}",
            result
        );

        // Verify the file was updated
        let updated_content =
            std::fs::read_to_string(yaml_path).expect("Failed to read updated file");
        assert!(updated_content.contains("x1: 20%"));
        assert!(updated_content.contains("y1: 15%"));
        assert!(updated_content.contains("x2: 60%"));
        assert!(updated_content.contains("y2: 45%"));
    }

    #[test]
    fn test_move_area_vs_resize_area_separation() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");

        muxbox.position = InputBounds {
            x1: "10%".to_string(),
            y1: "10%".to_string(),
            x2: "50%".to_string(),
            y2: "40%".to_string(),
        };

        let bounds = muxbox.bounds();

        // Test that move area (top border) doesn't conflict with resize area (bottom-right corner)
        assert!(detect_move_area(&muxbox, 30, bounds.y1 as u16)); // Top border = move area
        assert!(!detect_move_area(
            &muxbox,
            bounds.x2 as u16,
            bounds.y2 as u16
        )); // Bottom-right corner = not move area

        // Test resize area detection (corner-only now)
        use crate::draw_loop::detect_resize_edge;
        assert!(detect_resize_edge(&muxbox, bounds.x2 as u16, bounds.y2 as u16).is_some()); // Bottom-right corner
        assert!(detect_resize_edge(&muxbox, 30, bounds.y1 as u16).is_none()); // Top border = not resize area
    }
}
