#[cfg(test)]
mod text_wrapping_tests {
    use crate::draw_utils::{wrap_choices_to_width, wrap_text_to_width, WrappedChoice};
    use crate::model::muxbox::Choice;

    #[test]
    fn test_wrap_text_short_line() {
        let text = "Short line";
        let wrapped = wrap_text_to_width(text, 20);
        assert_eq!(wrapped, vec!["Short line"]);
    }

    #[test]
    fn test_wrap_text_exact_width() {
        let text = "Exactly twenty chars";
        let wrapped = wrap_text_to_width(text, 20);
        assert_eq!(wrapped, vec!["Exactly twenty chars"]);
    }

    #[test]
    fn test_wrap_text_word_boundary() {
        let text = "This is a long line that needs to be wrapped";
        let wrapped = wrap_text_to_width(text, 20);
        assert_eq!(
            wrapped,
            vec!["This is a long line", "that needs to be", "wrapped"]
        );
    }

    #[test]
    fn test_wrap_text_long_word() {
        let text = "Supercalifragilisticexpialidocious word";
        let wrapped = wrap_text_to_width(text, 10);
        assert_eq!(
            wrapped,
            vec!["Supercalif", "ragilistic", "expialidoc", "ious word"]
        );
    }

    #[test]
    fn test_wrap_text_multiple_lines() {
        let text = "Line one\nLine two is very long and needs wrapping\nLine three";
        let wrapped = wrap_text_to_width(text, 15);
        assert_eq!(
            wrapped,
            vec![
                "Line one",
                "Line two is",
                "very long and",
                "needs wrapping",
                "Line three"
            ]
        );
    }

    #[test]
    fn test_wrap_text_zero_width() {
        let text = "Any text";
        let wrapped = wrap_text_to_width(text, 0);
        assert_eq!(wrapped, vec!["Any text"]);
    }

    #[test]
    fn test_wrap_choices_basic() {
        let choices = vec![
            Choice {
                id: "1".to_string(),
                content: Some("Short choice".to_string()),
                script: None,
                thread: None,
                redirect_output: None,
                append_output: None,
                pty: None,
                execution_mode: crate::model::common::ExecutionMode::default(),
                selected: false,
                waiting: false,
            },
            Choice {
                id: "2".to_string(),
                content: Some("This is a very long choice that needs wrapping".to_string()),
                script: None,
                thread: None,
                redirect_output: None,
                append_output: None,
                pty: None,
                execution_mode: crate::model::common::ExecutionMode::default(),
                selected: true,
                waiting: false,
            },
        ];

        let wrapped = wrap_choices_to_width(&choices, 20);
        assert_eq!(wrapped.len(), 4); // 1 line + 3 wrapped lines

        // First choice - single line
        assert_eq!(wrapped[0].original_index, 0);
        assert_eq!(wrapped[0].line_index, 0);
        assert_eq!(wrapped[0].content, "Short choice");
        assert!(!wrapped[0].is_selected);

        // Second choice - wrapped over 3 lines
        assert_eq!(wrapped[1].original_index, 1);
        assert_eq!(wrapped[1].line_index, 0);
        assert_eq!(wrapped[1].content, "This is a very long");
        assert!(wrapped[1].is_selected);

        assert_eq!(wrapped[2].original_index, 1);
        assert_eq!(wrapped[2].line_index, 1);
        assert_eq!(wrapped[2].content, "choice that needs");
        assert!(wrapped[2].is_selected);

        assert_eq!(wrapped[3].original_index, 1);
        assert_eq!(wrapped[3].line_index, 2);
        assert_eq!(wrapped[3].content, "wrapping");
        assert!(wrapped[3].is_selected);
    }

    #[test]
    fn test_wrap_choices_waiting() {
        let choices = vec![Choice {
            id: "1".to_string(),
            content: Some("Processing long task".to_string()),
            script: None,
            thread: None,
            redirect_output: None,
            append_output: None,
            pty: None,
            execution_mode: crate::model::common::ExecutionMode::default(),
            selected: false,
            waiting: true,
        }];

        let wrapped = wrap_choices_to_width(&choices, 15);
        assert_eq!(wrapped.len(), 2); // Should wrap "Processing long task..."

        assert_eq!(wrapped[0].original_index, 0);
        assert_eq!(wrapped[0].line_index, 0);
        assert_eq!(wrapped[0].content, "Processing long");
        assert!(wrapped[0].is_waiting);

        assert_eq!(wrapped[1].original_index, 0);
        assert_eq!(wrapped[1].line_index, 1);
        assert_eq!(wrapped[1].content, "task...");
        assert!(wrapped[1].is_waiting);
    }
}
