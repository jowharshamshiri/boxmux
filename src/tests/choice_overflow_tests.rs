#[cfg(test)]
mod choice_overflow_tests {
    use crate::draw_utils::render_muxbox;
    use crate::model::muxbox::Choice;
    use crate::model::common::ScreenBuffer;
    use crate::model::common::Bounds;

    fn create_test_choice(id: &str, content: &str, selected: bool) -> Choice {
        Choice {
            id: id.to_string(),
            content: Some(content.to_string()),
            script: None,
            redirect_output: None,
            append_output: None,
            selected,
            waiting: false,
        }
    }

    #[test]
    fn test_choice_scroll_overflow_behavior() {
        // Test that choices respect overflow_behavior: "scroll" and generate scrollbars
        let mut buffer = ScreenBuffer::new(20, 10);
        let bounds = Bounds::new(0, 0, 19, 9); // 20x10 box
        
        // Create many choices that will overflow the box height (7 visible rows)
        let choices = vec![
            create_test_choice("c1", "Choice 1", false),
            create_test_choice("c2", "Choice 2", true),
            create_test_choice("c3", "Choice 3", false),
            create_test_choice("c4", "Choice 4", false),
            create_test_choice("c5", "Choice 5", false),
            create_test_choice("c6", "Choice 6", false),
            create_test_choice("c7", "Choice 7", false),
            create_test_choice("c8", "Choice 8", false),
            create_test_choice("c9", "Choice 9", false),
            create_test_choice("c10", "Choice 10", false),
        ];
        
        // Create streams for test
        let mut streams = indexmap::IndexMap::new();
        let choices_stream = crate::model::common::Stream {
            id: "choices".to_string(),
            label: "Test Menu".to_string(),
            stream_type: crate::model::common::StreamType::Choices,
            active: true,
            data: crate::model::common::StreamData::Choices(choices),
            source: None,
        };
        streams.insert("choices".to_string(), choices_stream);

        render_muxbox(
            &bounds,
            "white",
            "black",
            "black",
            &streams,
            0, // active_tab_index
            0, // tab_scroll_offset
            "white",
            "black",
            "center",
            "white",
            "black",
            "black",
            "white",
            "white",
            "scroll",
            Some(&true),
            0.0, // horizontal_scroll
            0.0, // vertical_scroll (top)
            false, // locked
            &mut buffer,
        );
        
        // Verify that scrollbar characters appear in the buffer
        let right_column = bounds.right();
        let mut scrollbar_found = false;
        for y in (bounds.top() + 1)..bounds.bottom() {
            let cell = buffer.get_cell(right_column, y);
            if cell.ch == '│' || cell.ch == '█' { // Track or knob character
                scrollbar_found = true;
                break;
            }
        }
        
        assert!(scrollbar_found, "Scrollbar should be drawn for overflowing choices in scroll mode");
        
        // Verify that only the first choices are visible (not all 10)
        let choice_1_found = buffer.content_contains("Choice 1");
        let choice_10_found = buffer.content_contains("Choice 10");
        
        assert!(choice_1_found, "First choice should be visible at scroll position 0");
        assert!(!choice_10_found, "Last choice should not be visible at scroll position 0");
    }

    #[test] 
    fn test_choice_scroll_with_vertical_offset() {
        // Test scrolling down in choices shows different content
        let mut buffer = ScreenBuffer::new(20, 8);
        let bounds = Bounds::new(0, 0, 19, 7); // 20x8 box (5 visible choices)
        
        let choices = vec![
            create_test_choice("c1", "Choice 1", false),
            create_test_choice("c2", "Choice 2", false),
            create_test_choice("c3", "Choice 3", false),
            create_test_choice("c4", "Choice 4", false),
            create_test_choice("c5", "Choice 5", false),
            create_test_choice("c6", "Choice 6", false),
            create_test_choice("c7", "Choice 7", false),
            create_test_choice("c8", "Choice 8", false),
        ];
        
        // Create streams for test
        let mut streams = indexmap::IndexMap::new();
        let choices_stream = crate::model::common::Stream {
            id: "choices".to_string(),
            label: "Scrolled Menu".to_string(),
            stream_type: crate::model::common::StreamType::Choices,
            active: true,
            data: crate::model::common::StreamData::Choices(choices),
            source: None,
        };
        streams.insert("choices".to_string(), choices_stream);

        render_muxbox(
            &bounds,
            "white",
            "black",
            "black",
            &streams,
            0, // active_tab_index
            0, // tab_scroll_offset
            "white",
            "black",
            "center",
            "white",
            "black",
            "black",
            "white",
            "white",
            "scroll",
            Some(&true),
            0.0, // horizontal_scroll
            100.0, // vertical_scroll (bottom) - should show later choices
            false, // locked
            &mut buffer,
        );
        
        // At 100% scroll, should show the last choices, not the first ones
        let choice_1_found = buffer.content_contains("Choice 1");
        let choice_8_found = buffer.content_contains("Choice 8");
        
        assert!(!choice_1_found, "First choice should not be visible at bottom scroll");
        assert!(choice_8_found, "Last choice should be visible at bottom scroll");
    }

    #[test]
    fn test_choice_no_overflow_no_scrollbar() {
        // Test that choices without overflow don't show scrollbars
        let mut buffer = ScreenBuffer::new(20, 10);
        let bounds = Bounds::new(0, 0, 19, 9);
        
        let choices = vec![
            create_test_choice("c1", "Choice 1", false),
            create_test_choice("c2", "Choice 2", true),
        ];
        
        // Create streams for test
        let mut streams = indexmap::IndexMap::new();
        let choices_stream = crate::model::common::Stream {
            id: "choices".to_string(),
            label: "Small Menu".to_string(),
            stream_type: crate::model::common::StreamType::Choices,
            active: true,
            data: crate::model::common::StreamData::Choices(choices),
            source: None,
        };
        streams.insert("choices".to_string(), choices_stream);

        render_muxbox(
            &bounds,
            "white",
            "black",
            "black",
            &streams,
            0, // active_tab_index
            0, // tab_scroll_offset
            "white",
            "black",
            "center",
            "white",
            "black",
            "black",
            "white",
            "white",
            "scroll",
            Some(&true),
            0.0, // horizontal_scroll
            0.0, // vertical_scroll
            false, // locked
            &mut buffer,
        );
        
        // Should NOT have scrollbar since choices fit
        let right_column = bounds.right();
        let mut scrollbar_found = false;
        for y in (bounds.top() + 1)..bounds.bottom() {
            let cell = buffer.get_cell(right_column, y);
            if cell.ch == '│' || cell.ch == '█' {
                scrollbar_found = true;
                break;
            }
        }
        
        assert!(!scrollbar_found, "Scrollbar should NOT be drawn when choices fit in container");
        
        // But right border should be drawn  
        let cell = buffer.get_cell(right_column, bounds.top() + 1);
        assert_eq!(cell.ch, '│', "Right border should be drawn when no scrollbar needed");
    }

    #[test]
    fn test_choice_overflow_detection() {
        // Test that overflow detection works properly for choices
        let mut buffer = ScreenBuffer::new(20, 6); // Very short box
        let bounds = Bounds::new(0, 0, 19, 5);
        
        let many_choices: Vec<Choice> = (1..=10).map(|i| {
            create_test_choice(&format!("c{}", i), &format!("Choice {}", i), false)
        }).collect();
        
        // Create streams for test
        let mut streams = indexmap::IndexMap::new();
        let choices_stream = crate::model::common::Stream {
            id: "choices".to_string(),
            label: "Overflow Test".to_string(),
            stream_type: crate::model::common::StreamType::Choices,
            active: true,
            data: crate::model::common::StreamData::Choices(many_choices),
            source: None,
        };
        streams.insert("choices".to_string(), choices_stream);

        render_muxbox(
            &bounds,
            "white",
            "black",
            "black",
            &streams,
            0, // active_tab_index
            0, // tab_scroll_offset
            "white",
            "black",
            "center",
            "white",
            "black",
            "black",
            "white",
            "white",
            "scroll",
            Some(&true),
            0.0, // horizontal_scroll
            50.0, // Middle scroll position
            false, // locked
            &mut buffer,
        );
        
        // Should show choices from the middle (around 50% scroll position)
        // With 10 choices and ~3 visible slots, 50% should show choices 4-6 approximately
        let early_choice_found = buffer.content_contains("Choice 1");
        let late_choice_found = buffer.content_contains("Choice 10");
        let middle_choice_found = buffer.content_contains("Choice 5") || buffer.content_contains("Choice 6");
        
        assert!(!early_choice_found, "Early choices should not be visible at middle scroll");
        assert!(!late_choice_found, "Late choices should not be visible at middle scroll");
        assert!(middle_choice_found, "Middle choices should be visible at middle scroll");
    }
}