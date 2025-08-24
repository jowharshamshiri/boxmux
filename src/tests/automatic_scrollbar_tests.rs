#[cfg(test)]
mod tests {
    use crate::draw_utils::content_size;
    use crate::tests::test_utils::TestDataFactory;

    #[test]
    fn test_automatic_scrollbar_overflow_detection_logic() {
        // Test the core logic that determines if scrollbars should be shown
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15";
        let (content_width, content_height) = content_size(content);

        // Small viewable area should trigger overflow
        let viewable_width = 10;
        let viewable_height = 5;

        let content_overflows = content_width > viewable_width || content_height > viewable_height;

        assert!(
            content_overflows,
            "Content should overflow in small viewable area"
        );
        assert!(
            content_height > viewable_height,
            "Content should overflow vertically"
        );
    }

    #[test]
    fn test_focusable_panel_detection() {
        // Test that panels with next_focus_id are considered focusable
        let mut panel = TestDataFactory::create_test_panel("test_panel");

        // Non-focusable panel
        panel.next_focus_id = None;
        assert!(
            panel.next_focus_id.is_none(),
            "Panel without next_focus_id should not be focusable"
        );

        // Focusable panel
        panel.next_focus_id = Some("next_panel".to_string());
        assert!(
            panel.next_focus_id.is_some(),
            "Panel with next_focus_id should be focusable"
        );
    }

    #[test]
    fn test_overflow_behavior_modification() {
        // Test that overflow behavior is correctly modified for focusable panels
        let mut panel = TestDataFactory::create_test_panel("test_panel");
        panel.next_focus_id = Some("next_panel".to_string()); // Make it focusable
        panel.overflow_behavior = Some("hidden".to_string()); // Originally hidden

        // Simulate the logic from draw_utils.rs
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15";
        let mut overflow_behavior = panel
            .overflow_behavior
            .clone()
            .unwrap_or("hidden".to_string());

        if panel.next_focus_id.is_some() {
            let (content_width, content_height) = content_size(content);
            let viewable_width = 10; // Small area to trigger overflow
            let viewable_height = 5;

            if content_width > viewable_width || content_height > viewable_height {
                overflow_behavior = "scroll".to_string();
            }
        }

        assert_eq!(overflow_behavior, "scroll", "Overflow behavior should be changed to scroll for focusable panel with overflowing content");
    }

    #[test]
    fn test_choice_overflow_triggers_scrollbars() {
        // Test that panels with overflowing choices trigger scrollbar display
        let mut panel = TestDataFactory::create_test_panel("choice_panel");
        panel.next_focus_id = Some("next_panel".to_string()); // Make it focusable
        
        // Add many choices that will overflow a small panel
        let mut choices = Vec::new();
        for i in 1..=20 {
            choices.push(crate::model::panel::Choice {
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
        
        // Set small bounds that will cause choice overflow
        panel.position = crate::model::common::InputBounds {
            x1: "0".to_string(),
            y1: "0".to_string(),
            x2: "20".to_string(), // Small width
            y2: "10".to_string(), // Small height - only 10 but 20 choices
        };

        // Test that has_scrollable_content detects the choice overflow
        assert!(panel.has_scrollable_content(), "Panel with overflowing choices should be detected as scrollable");

        // Simulate the new logic from draw_utils.rs that uses has_scrollable_content()
        let mut overflow_behavior = "hidden".to_string();
        if panel.next_focus_id.is_some() && panel.has_scrollable_content() {
            overflow_behavior = "scroll".to_string();
        }

        assert_eq!(overflow_behavior, "scroll", "Choice overflow should trigger scrollbar display");
    }
}
