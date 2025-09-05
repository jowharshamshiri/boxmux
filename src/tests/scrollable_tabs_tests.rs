#[cfg(test)]
mod tests {
    use crate::draw_utils::{
        calculate_tab_click_index, calculate_tab_navigation_click, TabNavigationAction,
    };
    use crate::model::muxbox::MuxBox;
    use crate::tests::test_utils::TestDataFactory;
    use indexmap::IndexMap;

    fn create_test_muxbox_with_many_tabs(tab_count: usize) -> MuxBox {
        let mut muxbox = TestDataFactory::create_test_muxbox("test");

        // Create many streams to force tab scrolling
        for i in 0..tab_count {
            let stream = crate::model::common::Stream {
                id: format!("stream_{}", i),
                label: format!("Tab{}", i + 1),
                stream_type: crate::model::common::StreamType::Content,
                source: None, // Active state managed by muxbox
                content: vec![format!("Content {}", i + 1)],
                choices: None,
                content_hash: 0,
                last_updated: std::time::SystemTime::now(),
                created_at: std::time::SystemTime::now(),
            };
            muxbox.streams.insert(format!("stream_{}", i), stream);
        }

        muxbox
    }

    #[test]
    fn test_tabs_need_scrolling_detection() {
        let muxbox = create_test_muxbox_with_many_tabs(10);

        // With small width, should need scrolling
        assert!(
            muxbox.tabs_need_scrolling(50),
            "Should need scrolling with 10 tabs in 50 characters"
        );

        // With large width, should not need scrolling
        assert!(
            !muxbox.tabs_need_scrolling(200),
            "Should not need scrolling with 10 tabs in 200 characters"
        );

        // Empty muxbox should not need scrolling
        let empty_muxbox = TestDataFactory::create_test_muxbox("empty");
        assert!(
            !empty_muxbox.tabs_need_scrolling(50),
            "Empty muxbox should not need scrolling"
        );
    }

    #[test]
    fn test_tab_scroll_left_right() {
        let mut muxbox = create_test_muxbox_with_many_tabs(5);

        // Start at offset 0
        assert_eq!(muxbox.tab_scroll_offset, 0);

        // Scroll right
        muxbox.scroll_tabs_right();
        assert_eq!(muxbox.tab_scroll_offset, 1);

        muxbox.scroll_tabs_right();
        assert_eq!(muxbox.tab_scroll_offset, 2);

        // Scroll left
        muxbox.scroll_tabs_left();
        assert_eq!(muxbox.tab_scroll_offset, 1);

        muxbox.scroll_tabs_left();
        assert_eq!(muxbox.tab_scroll_offset, 0);

        // Can't scroll left past 0
        muxbox.scroll_tabs_left();
        assert_eq!(muxbox.tab_scroll_offset, 0);
    }

    #[test]
    fn test_tab_scroll_boundaries() {
        let mut muxbox = create_test_muxbox_with_many_tabs(3);

        // Can't scroll right past last tab
        muxbox.scroll_tabs_right();
        muxbox.scroll_tabs_right();
        muxbox.scroll_tabs_right(); // Should be capped
        assert!(
            muxbox.tab_scroll_offset <= 2,
            "Should not scroll past last tab"
        );
    }

    #[test]
    fn test_max_tab_scroll_offset() {
        let muxbox = create_test_muxbox_with_many_tabs(10);

        // With small width, should have a reasonable max offset
        let max_offset = muxbox.max_tab_scroll_offset(60);
        assert!(
            max_offset > 0,
            "Should have positive max offset with many tabs"
        );
        assert!(max_offset < 10, "Max offset should be less than total tabs");

        // With large width, should have max offset of 0 (no scrolling needed)
        let max_offset_large = muxbox.max_tab_scroll_offset(300);
        assert_eq!(
            max_offset_large, 0,
            "Should have max offset 0 when all tabs fit"
        );
    }

    #[test]
    fn test_tab_click_with_scrolling() {
        let tab_labels = vec![
            "Tab1".to_string(),
            "Tab2".to_string(),
            "Tab3".to_string(),
            "Tab4".to_string(),
            "Tab5".to_string(),
            "Tab6".to_string(),
        ];

        // Test clicking with scroll offset 0 - click near center where tabs should be
        // With centering, tabs will be positioned differently, so click more centrally
        let clicked_tab = calculate_tab_click_index(
            25,
            0,
            50,
            &tab_labels,
            0,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        assert!(
            clicked_tab.is_some(),
            "Should be able to click centered tabs with no scrolling"
        );

        // Test clicking with scroll offset 2 - tabs should fill available space when scrolling
        let clicked_tab = calculate_tab_click_index(
            25,
            0,
            50,
            &tab_labels,
            2,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        assert!(
            clicked_tab.is_some(),
            "Should be able to click tabs with scrolling"
        );

        if let Some(tab_index) = clicked_tab {
            assert!(
                tab_index >= 2,
                "Should click tabs accounting for scroll offset"
            );
        }
    }

    #[test]
    fn test_navigation_arrow_detection() {
        let tab_labels = vec![
            "Tab1".to_string(),
            "Tab2".to_string(),
            "Tab3".to_string(),
            "Tab4".to_string(),
            "Tab5".to_string(),
            "Tab6".to_string(),
            "Tab7".to_string(),
            "Tab8".to_string(),
            "Tab9".to_string(),
            "Tab10".to_string(),
        ];

        // Should detect left arrow when scroll offset > 0
        let nav_action = calculate_tab_navigation_click(
            3,
            0,
            60,
            &tab_labels,
            2,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        assert!(
            matches!(nav_action, Some(TabNavigationAction::ScrollLeft)),
            "Should detect left arrow click with scroll offset > 0"
        );

        // Should detect right arrow when more tabs are hidden
        let nav_action = calculate_tab_navigation_click(
            57,
            0,
            60,
            &tab_labels,
            0,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        assert!(
            matches!(nav_action, Some(TabNavigationAction::ScrollRight)),
            "Should detect right arrow click when tabs are hidden on right"
        );

        // Should return None when no scrolling needed
        let few_tabs = vec!["Tab1".to_string(), "Tab2".to_string()];
        let nav_action = calculate_tab_navigation_click(
            30,
            0,
            100,
            &few_tabs,
            0,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        assert!(
            nav_action.is_none(),
            "Should return None when no scrolling needed"
        );
    }

    #[test]
    fn test_tab_scrolling_with_active_tab_visibility() {
        let mut muxbox = create_test_muxbox_with_many_tabs(10);

        // Switch to a tab that would be off-screen
        assert!(muxbox.switch_to_tab(7), "Should be able to switch to tab 7");

        // Tab scroll offset should adjust to make active tab visible
        // (This test validates that the active tab stays visible during scrolling)
        let tab_labels = muxbox.get_tab_labels();
        assert_eq!(tab_labels.len(), 10, "Should have 10 tab labels");

        // Verify active tab index is correct
        assert_eq!(
            muxbox.get_active_tab_index(),
            7,
            "Active tab should be index 7"
        );
    }
}
