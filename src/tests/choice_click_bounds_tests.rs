//! T0258 - Choice Click Detection with Text Bounds Only Tests
//! Test that choice clicks only trigger on actual choice text, not empty space after text

#[cfg(test)]
mod choice_click_bounds_tests {
    use crate::draw_loop::calculate_clicked_choice_index;
    use crate::model::muxbox::{Choice, MuxBox};
    use crate::tests::test_utils::TestDataFactory;

    /// Create a test choice with specific content
    fn create_test_choice(id: &str, content: &str) -> Choice {
        Choice {
            id: id.to_string(),
            content: Some(content.to_string()),
            script: Some(vec!["echo test".to_string()]),
            redirect_output: None,
            append_output: None,
            execution_mode: crate::model::common::ExecutionMode::default(),
            selected: false,
            waiting: false,
        }
    }

    /// Create a test muxbox with choices at fixed position
    fn create_test_muxbox_with_choices(choices: Vec<Choice>) -> MuxBox {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");
        muxbox.choices = Some(choices);

        // Set fixed position bounds for predictable testing: 10x10 muxbox at (10,10)
        muxbox.position = crate::model::common::InputBounds {
            x1: "10".to_string(),
            y1: "10".to_string(),
            x2: "20".to_string(),
            y2: "20".to_string(),
        };

        muxbox
    }

    /// Test clicking directly on choice text triggers choice
    #[test]
    fn test_click_on_choice_text_triggers_choice() {
        let choice1 = create_test_choice("choice1", "Build Project");
        let choice2 = create_test_choice("choice2", "Run Tests");
        let choices = vec![choice1.clone(), choice2.clone()];
        let muxbox = create_test_muxbox_with_choices(choices.clone());

        // Choice text starts at bounds.left() + 2 = 12
        // Choice lines start at bounds.top() + 1 = 11

        // Click on "B" of "Build Project" (first character)
        let result = calculate_clicked_choice_index(&muxbox, 12, 11, &choices);
        assert_eq!(
            result,
            Some(0),
            "Should trigger first choice when clicking on first character"
        );

        // Click on "t" of "Build Project" (last character at position 12 + "Build Project".len() - 1 = 24)
        let result = calculate_clicked_choice_index(&muxbox, 24, 11, &choices);
        assert_eq!(
            result,
            Some(0),
            "Should trigger first choice when clicking on last character"
        );

        // Click on "R" of "Run Tests" (first character of second choice)
        let result = calculate_clicked_choice_index(&muxbox, 12, 12, &choices);
        assert_eq!(
            result,
            Some(1),
            "Should trigger second choice when clicking on first character"
        );
    }

    /// Test clicking after choice text (empty space) does not trigger choice
    #[test]
    fn test_click_after_choice_text_does_not_trigger() {
        let choice1 = create_test_choice("choice1", "Build"); // 5 characters
        let choices = vec![choice1.clone()];
        let muxbox = create_test_muxbox_with_choices(choices.clone());

        // Choice text spans from 12 to 16 (12 + 5 - 1)
        // Click at position 17 (after the text)
        let result = calculate_clicked_choice_index(&muxbox, 17, 11, &choices);
        assert_eq!(
            result, None,
            "Should not trigger choice when clicking after text"
        );

        // Click far to the right on same line
        let result = calculate_clicked_choice_index(&muxbox, 19, 11, &choices);
        assert_eq!(
            result, None,
            "Should not trigger choice when clicking far after text"
        );
    }

    /// Test clicking before choice text does not trigger choice
    #[test]
    fn test_click_before_choice_text_does_not_trigger() {
        let choice1 = create_test_choice("choice1", "Build");
        let choices = vec![choice1.clone()];
        let muxbox = create_test_muxbox_with_choices(choices.clone());

        // Choice text starts at 12, click at 11 (before text)
        let result = calculate_clicked_choice_index(&muxbox, 11, 11, &choices);
        assert_eq!(
            result, None,
            "Should not trigger choice when clicking before text"
        );

        // Click at muxbox left border
        let result = calculate_clicked_choice_index(&muxbox, 10, 11, &choices);
        assert_eq!(
            result, None,
            "Should not trigger choice when clicking on muxbox border"
        );
    }

    /// Test clicking on waiting choice (with "..." suffix)
    #[test]
    fn test_click_on_waiting_choice_text() {
        let mut choice1 = create_test_choice("choice1", "Deploy");
        choice1.waiting = true; // This adds "..." to the displayed text
        let choices = vec![choice1.clone()];
        let muxbox = create_test_muxbox_with_choices(choices.clone());

        // Waiting choice displays as "Deploy..." (9 characters total)
        // Click on last character of "Deploy..."
        let result = calculate_clicked_choice_index(&muxbox, 20, 11, &choices); // 12 + 9 - 1 = 20
        assert_eq!(
            result,
            Some(0),
            "Should trigger waiting choice when clicking within expanded text"
        );

        // Click after the "..." should not trigger
        let result = calculate_clicked_choice_index(&muxbox, 21, 11, &choices);
        assert_eq!(
            result, None,
            "Should not trigger waiting choice when clicking after expanded text"
        );
    }

    /// Test clicking on empty choice (no content) does not trigger
    #[test]
    fn test_click_on_empty_choice_does_not_trigger() {
        let mut choice1 = create_test_choice("choice1", "Build");
        choice1.content = None; // Remove content
        let choices = vec![choice1.clone()];
        let muxbox = create_test_muxbox_with_choices(choices.clone());

        // Click where text would normally be
        let result = calculate_clicked_choice_index(&muxbox, 12, 11, &choices);
        assert_eq!(result, None, "Should not trigger choice with no content");
    }

    /// Test clicking on different choice lines
    #[test]
    fn test_click_on_different_choice_lines() {
        let choice1 = create_test_choice("choice1", "First");
        let choice2 = create_test_choice("choice2", "Second");
        let choice3 = create_test_choice("choice3", "Third");
        let choices = vec![choice1.clone(), choice2.clone(), choice3.clone()];
        let muxbox = create_test_muxbox_with_choices(choices.clone());

        // First choice at y=11, second at y=12, third at y=13

        // Click on first choice text
        let result = calculate_clicked_choice_index(&muxbox, 12, 11, &choices);
        assert_eq!(result, Some(0), "Should trigger first choice");

        // Click on second choice text
        let result = calculate_clicked_choice_index(&muxbox, 12, 12, &choices);
        assert_eq!(result, Some(1), "Should trigger second choice");

        // Click on third choice text
        let result = calculate_clicked_choice_index(&muxbox, 12, 13, &choices);
        assert_eq!(result, Some(2), "Should trigger third choice");

        // Click after third choice text should not trigger
        let result = calculate_clicked_choice_index(&muxbox, 18, 13, &choices); // 12 + "Third".len() = 17, so 18 is after
        assert_eq!(
            result, None,
            "Should not trigger when clicking after third choice text"
        );
    }

    /// Test coordinate boundary conditions
    #[test]
    fn test_coordinate_boundary_conditions() {
        let choice1 = create_test_choice("choice1", "X"); // Single character
        let choices = vec![choice1.clone()];
        let muxbox = create_test_muxbox_with_choices(choices.clone());

        // Single character choice at position 12

        // Click exactly on the character
        let result = calculate_clicked_choice_index(&muxbox, 12, 11, &choices);
        assert_eq!(
            result,
            Some(0),
            "Should trigger choice when clicking exactly on single character"
        );

        // Click one position after
        let result = calculate_clicked_choice_index(&muxbox, 13, 11, &choices);
        assert_eq!(
            result, None,
            "Should not trigger choice when clicking one position after single character"
        );
    }
}
