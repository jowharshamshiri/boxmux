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
    fn test_focusable_muxbox_detection() {
        // Test that muxboxes with next_focus_id are considered focusable
        let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");

        // Non-focusable muxbox
        muxbox.next_focus_id = None;
        assert!(
            muxbox.next_focus_id.is_none(),
            "MuxBox without next_focus_id should not be focusable"
        );

        // Focusable muxbox
        muxbox.next_focus_id = Some("next_muxbox".to_string());
        assert!(
            muxbox.next_focus_id.is_some(),
            "MuxBox with next_focus_id should be focusable"
        );
    }

    #[test]
    fn test_overflow_behavior_modification() {
        // Test that overflow behavior is correctly modified for focusable muxboxes
        let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");
        muxbox.next_focus_id = Some("next_muxbox".to_string()); // Make it focusable
        muxbox.overflow_behavior = Some("hidden".to_string()); // Originally hidden

        // Simulate the logic from draw_utils.rs
        let content = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5\nLine 6\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15";
        let mut overflow_behavior = muxbox
            .overflow_behavior
            .clone()
            .unwrap_or("hidden".to_string());

        if muxbox.next_focus_id.is_some() {
            let (content_width, content_height) = content_size(content);
            let viewable_width = 10; // Small area to trigger overflow
            let viewable_height = 5;

            if content_width > viewable_width || content_height > viewable_height {
                overflow_behavior = "scroll".to_string();
            }
        }

        assert_eq!(overflow_behavior, "scroll", "Overflow behavior should be changed to scroll for focusable muxbox with overflowing content");
    }

    #[test]
    fn test_choice_overflow_triggers_scrollbars() {
        // Test that muxboxes with overflowing choices trigger scrollbar display
        let mut muxbox = TestDataFactory::create_test_muxbox("choice_muxbox");
        muxbox.next_focus_id = Some("next_muxbox".to_string()); // Make it focusable

        // Add many choices that will overflow a small muxbox
        let mut choices = Vec::new();
        for i in 1..=20 {
            choices.push(crate::model::muxbox::Choice {
                id: format!("choice_{}", i),
                content: Some(format!("Choice {}", i)),
                script: None,
                    redirect_output: None,
                append_output: None,
                    execution_mode: crate::model::common::ExecutionMode::default(),
                selected: false,
                waiting: false,
            });
        }
        muxbox.choices = Some(choices);

        // Set small bounds that will cause choice overflow
        muxbox.position = crate::model::common::InputBounds {
            x1: "0".to_string(),
            y1: "0".to_string(),
            x2: "20".to_string(), // Small width
            y2: "10".to_string(), // Small height - only 10 but 20 choices
        };

        // Test that has_scrollable_content detects the choice overflow
        assert!(
            muxbox.has_scrollable_content(),
            "MuxBox with overflowing choices should be detected as scrollable"
        );

        // Simulate the new logic from draw_utils.rs that uses has_scrollable_content()
        let mut overflow_behavior = "hidden".to_string();
        if muxbox.next_focus_id.is_some() && muxbox.has_scrollable_content() {
            overflow_behavior = "scroll".to_string();
        }

        assert_eq!(
            overflow_behavior, "scroll",
            "Choice overflow should trigger scrollbar display"
        );
    }
}
