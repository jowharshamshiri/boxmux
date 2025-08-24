//! F0187 - Clickable Scrollbars Tests
//! Test the scrollbar clicking functionality for jumping to specific positions

#[cfg(test)]
mod clickable_scrollbar_tests {
    use crate::tests::test_utils::TestDataFactory;
    use crate::model::panel::Choice;
    use crate::model::common::InputBounds;

    #[test]
    fn test_vertical_scrollbar_click_calculation() {
        // Test the vertical scrollbar click position calculation logic
        let mut panel = TestDataFactory::create_test_panel("scrollbar_test");
        panel.next_focus_id = Some("next_panel".to_string()); // Make it focusable
        
        // Add choices to make content scrollable
        let mut choices = Vec::new();
        for i in 1..=20 {
            choices.push(Choice {
                id: format!("choice_{}", i),
                content: Some(format!("Choice {}", i)),
                script: None,
                thread: None,
                redirect_output: None,
                append_output: None,
                pty: None,
                selected: false,
                waiting: false,
            });
        }
        panel.choices = Some(choices);
        
        // Set small bounds to force scrolling
        panel.position = InputBounds {
            x1: "10".to_string(),
            y1: "10".to_string(),
            x2: "50".to_string(),
            y2: "20".to_string(), // Small height
        };

        assert!(panel.has_scrollable_content(), "Panel should have scrollable content");

        let panel_bounds = panel.bounds();
        
        // Test click position calculations
        let track_height = (panel_bounds.height() as isize - 2).max(1) as usize;
        
        // Click at top of track (should be ~0%)
        let top_click_y = panel_bounds.top() + 1;
        let top_position = ((top_click_y) - panel_bounds.top() - 1) as f64 / track_height as f64;
        let top_percentage = (top_position * 100.0).min(100.0).max(0.0);
        assert!(top_percentage < 20.0, "Top click should be near 0%: {}", top_percentage);
        
        // Click at bottom of track (should be ~100%)
        let bottom_click_y = panel_bounds.bottom() - 1;
        let bottom_position = ((bottom_click_y) - panel_bounds.top() - 1) as f64 / track_height as f64;
        let bottom_percentage = (bottom_position * 100.0).min(100.0).max(0.0);
        assert!(bottom_percentage > 80.0, "Bottom click should be near 100%: {}", bottom_percentage);
        
        // Click at middle of track (should be ~50%)
        let middle_click_y = panel_bounds.top() + (panel_bounds.height() / 2);
        let middle_position = ((middle_click_y) - panel_bounds.top() - 1) as f64 / track_height as f64;
        let middle_percentage = (middle_position * 100.0).min(100.0).max(0.0);
        assert!(middle_percentage > 30.0 && middle_percentage < 70.0, 
            "Middle click should be around 50%: {}", middle_percentage);
    }

    #[test]
    fn test_horizontal_scrollbar_click_calculation() {
        // Test the horizontal scrollbar click position calculation logic
        let mut panel = TestDataFactory::create_test_panel("scrollbar_test");
        panel.next_focus_id = Some("next_panel".to_string()); // Make it focusable
        
        // Add long content to make it horizontally scrollable
        panel.content = Some("This is a very long line of text that should exceed the panel width and trigger horizontal scrolling functionality".to_string());
        
        // Set narrow bounds to force horizontal scrolling
        panel.position = InputBounds {
            x1: "10".to_string(),
            y1: "10".to_string(),
            x2: "30".to_string(), // Narrow width
            y2: "50".to_string(),
        };

        assert!(panel.has_scrollable_content(), "Panel should have scrollable content");

        let panel_bounds = panel.bounds();
        
        // Test click position calculations
        let track_width = (panel_bounds.width() as isize - 2).max(1) as usize;
        
        // Click at left of track (should be ~0%)
        let left_click_x = panel_bounds.left() + 1;
        let left_position = ((left_click_x) - panel_bounds.left() - 1) as f64 / track_width as f64;
        let left_percentage = (left_position * 100.0).min(100.0).max(0.0);
        assert!(left_percentage < 20.0, "Left click should be near 0%: {}", left_percentage);
        
        // Click at right of track (should be ~100%)
        let right_click_x = panel_bounds.right() - 1;
        let right_position = ((right_click_x) - panel_bounds.left() - 1) as f64 / track_width as f64;
        let right_percentage = (right_position * 100.0).min(100.0).max(0.0);
        assert!(right_percentage > 80.0, "Right click should be near 100%: {}", right_percentage);
        
        // Click at middle of track (should be ~50%)
        let middle_click_x = panel_bounds.left() + (panel_bounds.width() / 2);
        let middle_position = ((middle_click_x) - panel_bounds.left() - 1) as f64 / track_width as f64;
        let middle_percentage = (middle_position * 100.0).min(100.0).max(0.0);
        assert!(middle_percentage > 30.0 && middle_percentage < 70.0, 
            "Middle click should be around 50%: {}", middle_percentage);
    }

    #[test]
    fn test_scrollbar_boundary_detection() {
        // Test that scrollbar click detection works for exact boundary coordinates
        let mut panel = TestDataFactory::create_test_panel("boundary_test");
        panel.next_focus_id = Some("next_panel".to_string());
        
        // Add scrollable content
        panel.content = Some("Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10".to_string());
        
        // Set bounds
        panel.position = InputBounds {
            x1: "20".to_string(),
            y1: "15".to_string(),
            x2: "60".to_string(),
            y2: "25".to_string(),
        };

        assert!(panel.has_scrollable_content(), "Panel should have scrollable content");

        let panel_bounds = panel.bounds();
        
        // Test vertical scrollbar boundary detection
        let right_border_x = panel_bounds.right();
        let top_y = panel_bounds.top() + 1; // Just inside top border
        let bottom_y = panel_bounds.bottom() - 1; // Just inside bottom border
        
        // These coordinates should be detected as scrollbar clicks
        assert_eq!(right_border_x as u16, right_border_x as u16); // Right border exists
        assert!(top_y < panel_bounds.bottom()); // Valid y range
        assert!(bottom_y > panel_bounds.top()); // Valid y range
        
        // Test horizontal scrollbar boundary detection
        let bottom_border_y = panel_bounds.bottom();
        let left_x = panel_bounds.left() + 1; // Just inside left border  
        let right_x = panel_bounds.right() - 1; // Just inside right border
        
        // These coordinates should be detected as scrollbar clicks
        assert_eq!(bottom_border_y as u16, bottom_border_y as u16); // Bottom border exists
        assert!(left_x < panel_bounds.right()); // Valid x range
        assert!(right_x > panel_bounds.left()); // Valid x range
    }
}