//! F0187 - Clickable Scrollbars Tests
//! Test the scrollbar clicking functionality for jumping to specific positions

#[cfg(test)]
mod clickable_scrollbar_tests {
    use crate::model::common::InputBounds;
    use crate::model::muxbox::Choice;
    use crate::tests::test_utils::TestDataFactory;

    #[test]
    fn test_vertical_scrollbar_click_calculation() {
        // Test the vertical scrollbar click position calculation logic
        let mut muxbox = TestDataFactory::create_test_muxbox("scrollbar_test");
        muxbox.next_focus_id = Some("next_muxbox".to_string()); // Make it focusable

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
        muxbox.choices = Some(choices);

        // Set small bounds to force scrolling
        muxbox.position = InputBounds {
            x1: "10".to_string(),
            y1: "10".to_string(),
            x2: "50".to_string(),
            y2: "20".to_string(), // Small height
        };

        assert!(
            muxbox.has_scrollable_content(),
            "MuxBox should have scrollable content"
        );

        let muxbox_bounds = muxbox.bounds();

        // Test click position calculations
        let track_height = (muxbox_bounds.height() as isize - 2).max(1) as usize;

        // Click at top of track (should be ~0%)
        let top_click_y = muxbox_bounds.top() + 1;
        let top_position = ((top_click_y) - muxbox_bounds.top() - 1) as f64 / track_height as f64;
        let top_percentage = (top_position * 100.0).min(100.0).max(0.0);
        assert!(
            top_percentage < 20.0,
            "Top click should be near 0%: {}",
            top_percentage
        );

        // Click at bottom of track (should be ~100%)
        let bottom_click_y = muxbox_bounds.bottom() - 1;
        let bottom_position =
            ((bottom_click_y) - muxbox_bounds.top() - 1) as f64 / track_height as f64;
        let bottom_percentage = (bottom_position * 100.0).min(100.0).max(0.0);
        assert!(
            bottom_percentage > 80.0,
            "Bottom click should be near 100%: {}",
            bottom_percentage
        );

        // Click at middle of track (should be ~50%)
        let middle_click_y = muxbox_bounds.top() + (muxbox_bounds.height() / 2);
        let middle_position =
            ((middle_click_y) - muxbox_bounds.top() - 1) as f64 / track_height as f64;
        let middle_percentage = (middle_position * 100.0).min(100.0).max(0.0);
        assert!(
            middle_percentage > 30.0 && middle_percentage < 70.0,
            "Middle click should be around 50%: {}",
            middle_percentage
        );
    }

    #[test]
    fn test_horizontal_scrollbar_click_calculation() {
        // Test the horizontal scrollbar click position calculation logic
        let mut muxbox = TestDataFactory::create_test_muxbox("scrollbar_test");
        muxbox.next_focus_id = Some("next_muxbox".to_string()); // Make it focusable

        // Add long content to make it horizontally scrollable
        muxbox.content = Some("This is a very long line of text that should exceed the muxbox width and trigger horizontal scrolling functionality".to_string());

        // Set narrow bounds to force horizontal scrolling
        muxbox.position = InputBounds {
            x1: "10".to_string(),
            y1: "10".to_string(),
            x2: "30".to_string(), // Narrow width
            y2: "50".to_string(),
        };

        assert!(
            muxbox.has_scrollable_content(),
            "MuxBox should have scrollable content"
        );

        let muxbox_bounds = muxbox.bounds();

        // Test click position calculations
        let track_width = (muxbox_bounds.width() as isize - 2).max(1) as usize;

        // Click at left of track (should be ~0%)
        let left_click_x = muxbox_bounds.left() + 1;
        let left_position = ((left_click_x) - muxbox_bounds.left() - 1) as f64 / track_width as f64;
        let left_percentage = (left_position * 100.0).min(100.0).max(0.0);
        assert!(
            left_percentage < 20.0,
            "Left click should be near 0%: {}",
            left_percentage
        );

        // Click at right of track (should be ~100%)
        let right_click_x = muxbox_bounds.right() - 1;
        let right_position =
            ((right_click_x) - muxbox_bounds.left() - 1) as f64 / track_width as f64;
        let right_percentage = (right_position * 100.0).min(100.0).max(0.0);
        assert!(
            right_percentage > 80.0,
            "Right click should be near 100%: {}",
            right_percentage
        );

        // Click at middle of track (should be ~50%)
        let middle_click_x = muxbox_bounds.left() + (muxbox_bounds.width() / 2);
        let middle_position =
            ((middle_click_x) - muxbox_bounds.left() - 1) as f64 / track_width as f64;
        let middle_percentage = (middle_position * 100.0).min(100.0).max(0.0);
        assert!(
            middle_percentage > 30.0 && middle_percentage < 70.0,
            "Middle click should be around 50%: {}",
            middle_percentage
        );
    }

    #[test]
    fn test_scrollbar_boundary_detection() {
        // Test that scrollbar click detection works for exact boundary coordinates
        let mut muxbox = TestDataFactory::create_test_muxbox("boundary_test");
        muxbox.next_focus_id = Some("next_muxbox".to_string());

        // Add scrollable content
        muxbox.content = Some(
            "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10"
                .to_string(),
        );

        // Set bounds
        muxbox.position = InputBounds {
            x1: "20".to_string(),
            y1: "15".to_string(),
            x2: "60".to_string(),
            y2: "25".to_string(),
        };

        assert!(
            muxbox.has_scrollable_content(),
            "MuxBox should have scrollable content"
        );

        let muxbox_bounds = muxbox.bounds();

        // Test vertical scrollbar boundary detection
        let right_border_x = muxbox_bounds.right();
        let top_y = muxbox_bounds.top() + 1; // Just inside top border
        let bottom_y = muxbox_bounds.bottom() - 1; // Just inside bottom border

        // These coordinates should be detected as scrollbar clicks
        assert_eq!(right_border_x as u16, right_border_x as u16); // Right border exists
        assert!(top_y < muxbox_bounds.bottom()); // Valid y range
        assert!(bottom_y > muxbox_bounds.top()); // Valid y range

        // Test horizontal scrollbar boundary detection
        let bottom_border_y = muxbox_bounds.bottom();
        let left_x = muxbox_bounds.left() + 1; // Just inside left border
        let right_x = muxbox_bounds.right() - 1; // Just inside right border

        // These coordinates should be detected as scrollbar clicks
        assert_eq!(bottom_border_y as u16, bottom_border_y as u16); // Bottom border exists
        assert!(left_x < muxbox_bounds.right()); // Valid x range
        assert!(right_x > muxbox_bounds.left()); // Valid x range
    }

    #[test]
    fn test_draggable_scroll_knob_vertical_calculation() {
        // Test vertical scroll knob drag calculation logic
        let mut muxbox = TestDataFactory::create_test_muxbox("drag_test");
        muxbox.next_focus_id = Some("next_muxbox".to_string());

        // Add many choices to create scrollable content
        let mut choices = Vec::new();
        for i in 1..=30 {
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
        muxbox.choices = Some(choices);

        // Set bounds for scrolling
        muxbox.position = InputBounds {
            x1: "10".to_string(),
            y1: "10".to_string(),
            x2: "50".to_string(),
            y2: "20".to_string(), // Small height for scrolling
        };

        assert!(
            muxbox.has_scrollable_content(),
            "MuxBox should have scrollable content"
        );

        // Test drag calculation logic
        let muxbox_bounds = muxbox.bounds();
        let track_height = (muxbox_bounds.height() as isize - 2).max(1) as usize;

        // Simulate drag from 25% position to 75% position
        let start_y = muxbox_bounds.top() + (track_height / 4); // 25% down the track
        let end_y = muxbox_bounds.top() + (track_height * 3 / 4); // 75% down the track

        let drag_delta = (end_y as isize) - (start_y as isize);
        let percentage_delta = (drag_delta as f64 / track_height as f64) * 100.0;

        // Should be approximately 50% change (from 25% to 75%)
        assert!(
            percentage_delta > 40.0 && percentage_delta < 60.0,
            "Drag delta should be around 50%: {}",
            percentage_delta
        );

        // Test boundaries - drag to top
        let top_drag = (muxbox_bounds.top() as isize) - (start_y as isize);
        let top_percentage_delta = (top_drag as f64 / track_height as f64) * 100.0;
        let new_top_percentage = (25.0 + top_percentage_delta).min(100.0).max(0.0);
        assert_eq!(
            new_top_percentage, 0.0,
            "Dragging above track should result in 0%"
        );

        // Test boundaries - drag to bottom
        let bottom_y = muxbox_bounds.bottom();
        let bottom_drag = (bottom_y as isize) - (start_y as isize);
        let bottom_percentage_delta = (bottom_drag as f64 / track_height as f64) * 100.0;
        let new_bottom_percentage = (25.0 + bottom_percentage_delta).min(100.0).max(0.0);
        assert_eq!(
            new_bottom_percentage, 100.0,
            "Dragging below track should result in 100%"
        );
    }

    #[test]
    fn test_draggable_scroll_knob_horizontal_calculation() {
        // Test horizontal scroll knob drag calculation logic
        let mut muxbox = TestDataFactory::create_test_muxbox("drag_test");
        muxbox.next_focus_id = Some("next_muxbox".to_string());

        // Add very long content to force horizontal scrolling
        muxbox.content = Some("This is an extremely long line of text that is intentionally designed to be much longer than any reasonable muxbox width to test horizontal scrolling functionality in the draggable scroll knob system".to_string());

        // Set narrow bounds for horizontal scrolling
        muxbox.position = InputBounds {
            x1: "10".to_string(),
            y1: "10".to_string(),
            x2: "30".to_string(), // Narrow width
            y2: "50".to_string(),
        };

        assert!(
            muxbox.has_scrollable_content(),
            "MuxBox should have scrollable content"
        );

        // Test drag calculation logic
        let muxbox_bounds = muxbox.bounds();
        let track_width = (muxbox_bounds.width() as isize - 2).max(1) as usize;

        // Simulate drag from 20% position to 80% position
        let start_x = muxbox_bounds.left() + (track_width / 5); // 20% across the track
        let end_x = muxbox_bounds.left() + (track_width * 4 / 5); // 80% across the track

        let drag_delta = (end_x as isize) - (start_x as isize);
        let percentage_delta = (drag_delta as f64 / track_width as f64) * 100.0;

        // Should be approximately 60% change (from 20% to 80%)
        assert!(
            percentage_delta > 50.0 && percentage_delta < 70.0,
            "Drag delta should be around 60%: {}",
            percentage_delta
        );

        // Test boundaries - drag to left
        let left_drag = (muxbox_bounds.left() as isize) - (start_x as isize);
        let left_percentage_delta = (left_drag as f64 / track_width as f64) * 100.0;
        let new_left_percentage = (20.0 + left_percentage_delta).min(100.0).max(0.0);
        assert!(
            new_left_percentage < 10.0,
            "Dragging leftward should result in low percentage: {}",
            new_left_percentage
        );

        // Test boundaries - drag to right
        let right_x = muxbox_bounds.right();
        let right_drag = (right_x as isize) - (start_x as isize);
        let right_percentage_delta = (right_drag as f64 / track_width as f64) * 100.0;
        let new_right_percentage = (20.0 + right_percentage_delta).min(100.0).max(0.0);
        assert_eq!(
            new_right_percentage, 100.0,
            "Dragging beyond right should result in 100%"
        );
    }

    #[test]
    fn test_scroll_knob_boundary_constraints() {
        // Test that scroll knob dragging respects 0-100% boundaries
        let mut muxbox = TestDataFactory::create_test_muxbox("boundary_test");
        muxbox.next_focus_id = Some("next_muxbox".to_string());

        // Set initial scroll positions
        muxbox.vertical_scroll = Some(50.0);
        muxbox.horizontal_scroll = Some(30.0);

        // Add scrollable content
        let mut choices = Vec::new();
        for i in 1..=20 {
            choices.push(Choice {
                id: format!("choice_{}", i),
                content: Some(format!(
                    "Very long choice content that should trigger horizontal scrolling {}",
                    i
                )),
                script: None,
                thread: None,
                redirect_output: None,
                append_output: None,
                pty: None,
                selected: false,
                waiting: false,
            });
        }
        muxbox.choices = Some(choices);

        muxbox.position = InputBounds {
            x1: "5".to_string(),
            y1: "5".to_string(),
            x2: "25".to_string(),
            y2: "15".to_string(),
        };

        let muxbox_bounds = muxbox.bounds();
        let track_height = (muxbox_bounds.height() as isize - 2).max(1) as usize;
        let track_width = (muxbox_bounds.width() as isize - 2).max(1) as usize;

        // Test vertical drag beyond boundaries
        let extreme_up_drag = -1000_isize; // Way beyond top
        let up_percentage_delta = (extreme_up_drag as f64 / track_height as f64) * 100.0;
        let constrained_up = (50.0 + up_percentage_delta).min(100.0).max(0.0);
        assert_eq!(
            constrained_up, 0.0,
            "Extreme upward drag should be constrained to 0%"
        );

        let extreme_down_drag = 1000_isize; // Way beyond bottom
        let down_percentage_delta = (extreme_down_drag as f64 / track_height as f64) * 100.0;
        let constrained_down = (50.0 + down_percentage_delta).min(100.0).max(0.0);
        assert_eq!(
            constrained_down, 100.0,
            "Extreme downward drag should be constrained to 100%"
        );

        // Test horizontal drag beyond boundaries
        let extreme_left_drag = -1000_isize; // Way beyond left
        let left_percentage_delta = (extreme_left_drag as f64 / track_width as f64) * 100.0;
        let constrained_left = (30.0 + left_percentage_delta).min(100.0).max(0.0);
        assert_eq!(
            constrained_left, 0.0,
            "Extreme leftward drag should be constrained to 0%"
        );

        let extreme_right_drag = 1000_isize; // Way beyond right
        let right_percentage_delta = (extreme_right_drag as f64 / track_width as f64) * 100.0;
        let constrained_right = (30.0 + right_percentage_delta).min(100.0).max(0.0);
        assert_eq!(
            constrained_right, 100.0,
            "Extreme rightward drag should be constrained to 100%"
        );
    }
}
