#[cfg(test)]
mod tests {
    use super::super::choice_content::ChoiceContent;
    use super::super::renderable_content::*;
    use super::super::text_content::TextContent;
    use crate::model::muxbox::Choice;
    use crate::{Bounds, ScreenBuffer};
    use std::time::Duration;
    use std::time::SystemTime;

    #[test]
    fn test_content_event_creation() {
        // Test basic event creation
        let click_event = ContentEvent::new_click(Some((10, 5)), Some("choice_1".to_string()));
        assert_eq!(click_event.event_type, EventType::Click);
        assert_eq!(click_event.position, Some((10, 5)));
        assert_eq!(click_event.zone_id, Some("choice_1".to_string()));
        assert_eq!(click_event.data.mouse_button, Some(MouseButton::Left));

        // Test hover event
        let hover_event = ContentEvent::new_hover((15, 8), Some("text_area".to_string()));
        assert_eq!(hover_event.event_type, EventType::Hover);
        assert_eq!(hover_event.position, Some((15, 8)));
        assert_eq!(hover_event.zone_id, Some("text_area".to_string()));

        // Test key press event
        let key_event = ContentEvent::new_keypress(
            "ArrowDown".to_string(),
            vec![KeyModifier::Ctrl],
            Some("menu".to_string()),
        );
        assert_eq!(key_event.event_type, EventType::KeyPress);
        assert!(key_event.key_info().is_some());
        assert_eq!(key_event.key_info().unwrap().key, "ArrowDown");
        assert_eq!(
            key_event.key_info().unwrap().modifiers,
            vec![KeyModifier::Ctrl]
        );
    }

    #[test]
    fn test_content_event_helper_methods() {
        let click_event = ContentEvent::new_click(None, None);
        assert!(click_event.is_click());
        assert!(!click_event.is_double_click());
        assert!(!click_event.is_keyboard());

        let key_event = ContentEvent::new_keypress("Enter".to_string(), vec![], None);
        assert!(!key_event.is_click());
        assert!(key_event.is_keyboard());

        let scroll_event = ContentEvent::new_scroll(ScrollDirection::Down, 3, None);
        assert!(scroll_event.scroll_info().is_some());
        assert_eq!(
            scroll_event.scroll_info().unwrap().direction,
            ScrollDirection::Down
        );
        assert_eq!(scroll_event.scroll_info().unwrap().amount, 3);
    }

    #[test]
    fn test_choice_content_event_handling() {
        let mut choices = vec![
            Choice {
                content: Some("Option 1".to_string()),
                selected: false,
                waiting: false,
                ..Default::default()
            },
            Choice {
                content: Some("Option 2".to_string()),
                selected: true,
                waiting: false,
                ..Default::default()
            },
        ];

        let white_color = Some("white".to_string());
        let yellow_color = Some("yellow".to_string());
        let blue_color = Some("blue".to_string());

        let mut choice_content =
            ChoiceContent::new(&choices, &white_color, &None, &yellow_color, &blue_color);

        // Test valid choice click
        let click_event = ContentEvent::new_click(Some((5, 0)), Some("choice_0".to_string()));
        let result = choice_content.handle_event(&click_event);
        assert_eq!(result, EventResult::Handled);

        // Test invalid choice click
        let invalid_click = ContentEvent::new_click(Some((5, 0)), Some("choice_5".to_string()));
        let result = choice_content.handle_event(&invalid_click);
        assert_eq!(result, EventResult::NotHandled);

        // Test hover event
        let hover_event = ContentEvent::new_hover((5, 1), Some("choice_1".to_string()));
        let result = choice_content.handle_event(&hover_event);
        assert_eq!(result, EventResult::HandledContinue);

        // Test keyboard navigation
        let key_event = ContentEvent::new_keypress(
            "ArrowDown".to_string(),
            vec![],
            Some("choice_0".to_string()),
        );
        let result = choice_content.handle_event(&key_event);
        assert_eq!(result, EventResult::Handled);

        // Test non-navigation key
        let other_key = ContentEvent::new_keypress("x".to_string(), vec![], None);
        let result = choice_content.handle_event(&other_key);
        assert_eq!(result, EventResult::NotHandled);
    }

    #[test]
    fn test_text_content_event_handling() {
        let white_text_color = Some("white".to_string());

        let mut text_content =
            TextContent::new("Sample text content\nLine 2", &white_text_color, &None);

        // Test click - text generally doesn't handle clicks
        let click_event = ContentEvent::new_click(Some((5, 0)), None);
        let result = text_content.handle_event(&click_event);
        assert_eq!(result, EventResult::NotHandled);

        // Test Ctrl+C for copy
        let copy_event = ContentEvent::new_keypress("c".to_string(), vec![KeyModifier::Ctrl], None);
        let result = text_content.handle_event(&copy_event);
        assert_eq!(result, EventResult::NotHandled); // Could be handled at higher level

        // Test scroll event
        let scroll_event = ContentEvent::new_scroll(ScrollDirection::Down, 1, None);
        let result = text_content.handle_event(&scroll_event);
        assert_eq!(result, EventResult::HandledContinue); // Let scrollbars handle
    }

    #[test]
    fn test_event_backward_compatibility() {
        let mut choices = vec![Choice {
            content: Some("Test Choice".to_string()),
            selected: false,
            waiting: false,
            ..Default::default()
        }];

        let white_dep_color = Some("white".to_string());
        let yellow_dep_color = Some("yellow".to_string());
        let blue_dep_color = Some("blue".to_string());

        let mut choice_content = ChoiceContent::new(
            &choices,
            &white_dep_color,
            &None,
            &yellow_dep_color,
            &blue_dep_color,
        );

        // Test event system with proper click events
        let click_event = ContentEvent::new_click(Some((5, 0)), Some("choice_0".to_string()));
        let result = choice_content.handle_event(&click_event);
        assert!(matches!(result, EventResult::Handled));

        let invalid_event = ContentEvent::new_click(Some((5, 0)), Some("choice_5".to_string()));
        let invalid_result = choice_content.handle_event(&invalid_event);
        assert!(matches!(invalid_result, EventResult::NotHandled));

        // Test with text content
        let mut text_content = TextContent::new("test", &None, &None);
        let text_event = ContentEvent::new_click(Some((0, 0)), None);
        let result = text_content.handle_event(&text_event);
        assert!(matches!(result, EventResult::NotHandled)); // Text doesn't handle clicks
    }

    #[test]
    fn test_custom_events() {
        let mut choices = vec![Choice {
            content: Some("Custom Choice".to_string()),
            ..Default::default()
        }];

        let mut choice_content = ChoiceContent::new(&choices, &None, &None, &None, &None);

        // Test custom event
        let custom_event = ContentEvent::new_custom(
            "selection_changed".to_string(),
            Some("choice_0".to_string()),
            Some("choice_0".to_string()),
        );

        let result = choice_content.handle_event(&custom_event);
        assert_eq!(result, EventResult::NotHandled); // Custom events not handled by default
    }

    #[test]
    fn test_event_data_defaults() {
        let event_data = EventData::default();
        assert!(event_data.mouse_button.is_none());
        assert!(event_data.key.is_none());
        assert!(event_data.scroll.is_none());
        assert!(event_data.size.is_none());
        assert!(event_data.custom_data.is_none());
    }

    #[test]
    fn test_multiple_event_types() {
        let mut text_content = TextContent::new("test content", &None, &None);

        // Test focus event
        let focus_event = ContentEvent::new_focus(None);
        let result = text_content.handle_event(&focus_event);
        assert_eq!(result, EventResult::NotHandled);

        // Test blur event
        let blur_event = ContentEvent::new_blur(None);
        let result = text_content.handle_event(&blur_event);
        assert_eq!(result, EventResult::NotHandled);

        // Test resize event
        let resize_event = ContentEvent::new_resize((100, 50));
        let result = text_content.handle_event(&resize_event);
        assert_eq!(result, EventResult::NotHandled);

        // Test double click
        let mut event = ContentEvent::new_click(None, None);
        event.event_type = EventType::DoubleClick;
        let result = text_content.handle_event(&event);
        assert_eq!(result, EventResult::NotHandled);
    }

    #[test]
    fn test_mouse_button_variants() {
        // Test different mouse buttons
        let left_click = ContentEvent::new_click_with_button(None, None, MouseButton::Left);
        assert_eq!(left_click.mouse_button(), Some(&MouseButton::Left));

        let right_click = ContentEvent::new_click_with_button(None, None, MouseButton::Right);
        assert_eq!(right_click.mouse_button(), Some(&MouseButton::Right));

        let middle_click = ContentEvent::new_click_with_button(None, None, MouseButton::Middle);
        assert_eq!(middle_click.mouse_button(), Some(&MouseButton::Middle));

        let wheel_up = ContentEvent::new_click_with_button(None, None, MouseButton::WheelUp);
        assert_eq!(wheel_up.mouse_button(), Some(&MouseButton::WheelUp));
    }

    #[test]
    fn test_key_modifiers() {
        let event = ContentEvent::new_keypress(
            "s".to_string(),
            vec![KeyModifier::Ctrl, KeyModifier::Shift],
            None,
        );

        let key_info = event.key_info().unwrap();
        assert_eq!(key_info.key, "s");
        assert!(key_info.modifiers.contains(&KeyModifier::Ctrl));
        assert!(key_info.modifiers.contains(&KeyModifier::Shift));
        assert!(!key_info.modifiers.contains(&KeyModifier::Alt));
    }

    #[test]
    fn test_scroll_directions() {
        let down_scroll = ContentEvent::new_scroll(ScrollDirection::Down, 5, None);
        assert_eq!(
            down_scroll.scroll_info().unwrap().direction,
            ScrollDirection::Down
        );

        let up_scroll = ContentEvent::new_scroll(ScrollDirection::Up, 3, None);
        assert_eq!(
            up_scroll.scroll_info().unwrap().direction,
            ScrollDirection::Up
        );

        let left_scroll = ContentEvent::new_scroll(ScrollDirection::Left, 2, None);
        assert_eq!(
            left_scroll.scroll_info().unwrap().direction,
            ScrollDirection::Left
        );

        let right_scroll = ContentEvent::new_scroll(ScrollDirection::Right, 1, None);
        assert_eq!(
            right_scroll.scroll_info().unwrap().direction,
            ScrollDirection::Right
        );
    }

    #[test]
    fn test_mouse_move_events() {
        // Test basic mouse move
        let move_event =
            ContentEvent::new_mouse_move(Some((5, 5)), (10, 8), Some("text_area".to_string()));
        assert!(move_event.is_mouse_move());
        assert_eq!(move_event.position, Some((10, 8)));

        let move_info = move_event.mouse_move_info().unwrap();
        assert_eq!(move_info.from_position, Some((5, 5)));
        assert_eq!(move_info.to_position, (10, 8));
        assert_eq!(move_info.delta, (5, 3));
        assert!(!move_info.is_dragging);

        // Test mouse drag
        let drag_event = ContentEvent::new_mouse_drag(
            (5, 5),
            (15, 10),
            MouseButton::Left,
            Some("choice_1".to_string()),
        );
        assert!(drag_event.is_mouse_move());
        assert!(drag_event.is_drag());

        let drag_info = drag_event.mouse_move_info().unwrap();
        assert!(drag_info.is_dragging);
        assert_eq!(drag_info.drag_button, Some(MouseButton::Left));
        assert_eq!(drag_info.delta, (10, 5));
    }

    #[test]
    fn test_hover_events() {
        // Test hover enter
        let hover_enter = ContentEvent::new_hover_enter(
            (10, 5),
            "choice_0".to_string(),
            Some("previous_zone".to_string()),
        );
        assert!(hover_enter.is_hover_enter());
        assert!(!hover_enter.is_hover_leave());

        let hover_info = hover_enter.hover_info().unwrap();
        assert_eq!(hover_info.state, HoverState::Enter);
        assert_eq!(hover_info.current_zone, Some("choice_0".to_string()));
        assert_eq!(hover_info.previous_zone, Some("previous_zone".to_string()));

        // Test hover leave
        let hover_leave = ContentEvent::new_hover_leave(
            (12, 6),
            "choice_0".to_string(),
            Some("new_zone".to_string()),
        );
        assert!(hover_leave.is_hover_leave());
        assert!(!hover_leave.is_hover_enter());

        // Test hover move with duration
        let hover_move = ContentEvent::new_hover_move(
            (11, 5),
            "choice_0".to_string(),
            Duration::from_millis(1500),
        );
        let hover_info = hover_move.hover_info().unwrap();
        assert_eq!(hover_info.state, HoverState::Move);
        assert!(hover_info.hover_duration.is_some());
        assert_eq!(
            hover_info.hover_duration.unwrap(),
            Duration::from_millis(1500)
        );
    }

    #[test]
    fn test_box_resize_events() {
        let resize_event = ContentEvent::new_box_resize(
            BoxResizeType::Interactive,
            (10, 10, 50, 30), // original bounds
            (10, 10, 60, 35), // new bounds
            ResizeAnchor::BottomRight,
            ResizeState::InProgress,
        );

        assert!(resize_event.is_box_resize());

        let resize_info = resize_event.box_resize_info().unwrap();
        assert_eq!(resize_info.resize_type, BoxResizeType::Interactive);
        assert_eq!(resize_info.original_bounds, (10, 10, 50, 30));
        assert_eq!(resize_info.new_bounds, (10, 10, 60, 35));
        assert_eq!(resize_info.anchor, ResizeAnchor::BottomRight);
        assert_eq!(resize_info.state, ResizeState::InProgress);
    }

    #[test]
    fn test_title_change_events() {
        let title_event = ContentEvent::new_title_change(
            Some("Old Title".to_string()),
            "New Title".to_string(),
            TitleChangeSource::User,
            true,
        );

        assert!(title_event.is_title_change());

        let title_info = title_event.title_change_info().unwrap();
        assert_eq!(title_info.old_title, Some("Old Title".to_string()));
        assert_eq!(title_info.new_title, "New Title");
        assert_eq!(title_info.source, TitleChangeSource::User);
        assert!(title_info.persist);
    }

    #[test]
    fn test_extended_choice_event_handling() {
        let mut choices = vec![Choice {
            content: Some("Hover Choice".to_string()),
            selected: false,
            waiting: false,
            ..Default::default()
        }];

        let white_hover_color = Some("white".to_string());
        let yellow_hover_color = Some("yellow".to_string());
        let blue_hover_color = Some("blue".to_string());

        let mut choice_content = ChoiceContent::new(
            &choices,
            &white_hover_color,
            &None,
            &yellow_hover_color,
            &blue_hover_color,
        );

        // Test hover enter
        let hover_enter = ContentEvent::new_hover_enter((5, 0), "choice_0".to_string(), None);
        let result = choice_content.handle_event(&hover_enter);
        assert_eq!(result, EventResult::HandledContinue);

        // Test mouse move (non-drag)
        let mouse_move =
            ContentEvent::new_mouse_move(Some((5, 0)), (6, 0), Some("choice_0".to_string()));
        let result = choice_content.handle_event(&mouse_move);
        assert_eq!(result, EventResult::HandledContinue);

        // Test mouse drag
        let mouse_drag = ContentEvent::new_mouse_drag(
            (5, 0),
            (10, 0),
            MouseButton::Left,
            Some("choice_0".to_string()),
        );
        let result = choice_content.handle_event(&mouse_drag);
        assert_eq!(result, EventResult::NotHandled); // Let higher level handle
    }

    #[test]
    fn test_extended_text_event_handling() {
        let white_interaction_color = Some("white".to_string());

        let mut text_content = TextContent::new(
            "Sample text for interaction testing",
            &white_interaction_color,
            &None,
        );

        // Test mouse move over text
        let mouse_move = ContentEvent::new_mouse_move(Some((5, 0)), (10, 0), None);
        let result = text_content.handle_event(&mouse_move);
        assert_eq!(result, EventResult::NotHandled);

        // Test text selection drag
        let drag_event = ContentEvent::new_mouse_drag((5, 0), (15, 0), MouseButton::Left, None);
        let result = text_content.handle_event(&drag_event);
        assert_eq!(result, EventResult::NotHandled); // Could be implemented for text selection

        // Test hover for tooltips
        let hover_enter = ContentEvent::new_hover_enter((10, 0), "word_zone".to_string(), None);
        let result = text_content.handle_event(&hover_enter);
        assert_eq!(result, EventResult::NotHandled); // Could show word tooltips

        // Test resize triggering reflow
        let resize_event = ContentEvent::new_box_resize(
            BoxResizeType::Interactive,
            (0, 0, 50, 20),
            (0, 0, 60, 20),
            ResizeAnchor::Right,
            ResizeState::Completed,
        );
        let result = text_content.handle_event(&resize_event);
        assert_eq!(result, EventResult::StateChanged); // Text needs reflow
    }

    #[test]
    fn test_resize_anchor_types() {
        // Test all resize anchor types
        let anchors = vec![
            ResizeAnchor::TopLeft,
            ResizeAnchor::TopRight,
            ResizeAnchor::BottomLeft,
            ResizeAnchor::BottomRight,
            ResizeAnchor::Top,
            ResizeAnchor::Bottom,
            ResizeAnchor::Left,
            ResizeAnchor::Right,
        ];

        for anchor in anchors {
            let resize_event = ContentEvent::new_box_resize(
                BoxResizeType::Interactive,
                (10, 10, 50, 30),
                (15, 15, 55, 35),
                anchor.clone(),
                ResizeState::Started,
            );

            assert!(resize_event.is_box_resize());
            assert_eq!(resize_event.box_resize_info().unwrap().anchor, anchor);
        }
    }

    #[test]
    fn test_title_change_sources() {
        let sources = vec![
            TitleChangeSource::User,
            TitleChangeSource::PTY,
            TitleChangeSource::Script,
            TitleChangeSource::API,
            TitleChangeSource::System,
        ];

        for source in sources {
            let title_event = ContentEvent::new_title_change(
                None,
                "Test Title".to_string(),
                source.clone(),
                false,
            );

            assert!(title_event.is_title_change());
            assert_eq!(title_event.title_change_info().unwrap().source, source);
        }
    }

    #[test]
    fn test_movement_delta_calculation() {
        // Test positive movement
        let move_event = ContentEvent::new_mouse_move(Some((5, 10)), (15, 20), None);
        assert_eq!(move_event.movement_delta(), Some((10, 10)));

        // Test negative movement
        let move_back = ContentEvent::new_mouse_move(Some((20, 25)), (10, 15), None);
        assert_eq!(move_back.movement_delta(), Some((-10, -10)));

        // Test no previous position
        let move_start = ContentEvent::new_mouse_move(None, (10, 10), None);
        assert_eq!(move_start.movement_delta(), Some((0, 0)));

        // Test non-move event
        let click_event = ContentEvent::new_click(None, None);
        assert_eq!(click_event.movement_delta(), None);
    }

    #[test]
    fn test_extended_event_data_defaults() {
        let event_data = EventData::default();
        assert!(event_data.mouse_button.is_none());
        assert!(event_data.key.is_none());
        assert!(event_data.scroll.is_none());
        assert!(event_data.size.is_none());
        assert!(event_data.mouse_move.is_none());
        assert!(event_data.hover.is_none());
        assert!(event_data.box_resize.is_none());
        assert!(event_data.title_change.is_none());
        assert!(event_data.custom_data.is_none());
    }
}
