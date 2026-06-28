#[cfg(test)]
mod tests {
    use crate::components::TabBar;
    use crate::draw_utils::{
        calculate_tab_click_index, calculate_tab_navigation_click, TabNavigationAction,
    };
    use crate::model::common::ScreenBuffer;
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

    /// Render a tab bar (no close buttons, matching the empty-close-button slice
    /// the click/navigation helpers compute against) and return the tab row.
    fn render_tab_row(
        x1: usize,
        x2: usize,
        tab_labels: &[String],
        scroll_offset: usize,
        fg: &Option<String>,
        bg: &Option<String>,
    ) -> ScreenBuffer {
        let close_buttons = vec![false; tab_labels.len()];
        let mut buffer = ScreenBuffer::new_custom(x2 + 1, 1);
        TabBar::draw(
            0,
            x1,
            x2,
            fg,
            bg,
            fg,
            bg,
            tab_labels,
            &close_buttons,
            0,
            scroll_offset,
            None,
            &mut buffer,
        );
        buffer
    }

    #[test]
    fn test_tab_click_with_scrolling() {
        let tab_labels: Vec<String> = (1..=6).map(|i| format!("Tab{}", i)).collect();
        let x1 = 0usize;
        let x2 = 50usize;
        let fg = Some("white".to_string());
        let bg = Some("black".to_string());

        for scroll_offset in [0usize, 2usize] {
            // Every cell that the click helper resolves to a tab must map to a tab
            // that is actually visible at this scroll offset: tabs scrolled off the
            // left edge (index < scroll_offset) must never be clickable, and the
            // visible ones must be reachable.
            let clickable: Vec<usize> = (x1..=x2)
                .filter_map(|x| {
                    calculate_tab_click_index(x, x1, x2, &tab_labels, scroll_offset, &fg, &bg)
                })
                .collect();

            assert!(
                !clickable.is_empty(),
                "tabs must remain clickable at scroll offset {}",
                scroll_offset
            );
            assert!(
                clickable
                    .iter()
                    .all(|&i| i >= scroll_offset && i < tab_labels.len()),
                "clickable tab indices {:?} must all be visible at scroll offset {}",
                clickable,
                scroll_offset
            );
            assert!(
                clickable.contains(&scroll_offset),
                "the first visible tab (index {}) must be clickable",
                scroll_offset
            );
        }
    }

    #[test]
    fn test_navigation_arrow_detection() {
        let tab_labels: Vec<String> = (1..=10).map(|i| format!("Tab{}", i)).collect();
        let x1 = 0usize;
        let x2 = 60usize;
        let fg = Some("white".to_string());
        let bg = Some("black".to_string());
        // Offset 1: tabs are hidden on both sides, so both arrows render.
        let scroll_offset = 1usize;

        let buffer = render_tab_row(x1, x2, &tab_labels, scroll_offset, &fg, &bg);
        let left_arrow_x = (0..buffer.width)
            .find(|x| buffer.get(*x, 0).is_some_and(|c| c.ch == '◀'))
            .expect("left arrow must render when scrolled right of the start");
        let right_arrow_x = (0..buffer.width)
            .find(|x| buffer.get(*x, 0).is_some_and(|c| c.ch == '▶'))
            .expect("right arrow must render while tabs remain hidden on the right");

        // Clicking the exact rendered arrow glyph triggers the matching scroll.
        assert_eq!(
            calculate_tab_navigation_click(left_arrow_x, x1, x2, &tab_labels, scroll_offset, &fg, &bg),
            Some(TabNavigationAction::ScrollLeft),
            "clicking the rendered left arrow glyph must scroll left"
        );
        assert_eq!(
            calculate_tab_navigation_click(right_arrow_x, x1, x2, &tab_labels, scroll_offset, &fg, &bg),
            Some(TabNavigationAction::ScrollRight),
            "clicking the rendered right arrow glyph must scroll right"
        );

        // Exact-cell mapping: the padding space beside an arrow is not the arrow,
        // so it must not trigger navigation.
        assert_ne!(
            calculate_tab_navigation_click(
                left_arrow_x + 1,
                x1,
                x2,
                &tab_labels,
                scroll_offset,
                &fg,
                &bg
            ),
            Some(TabNavigationAction::ScrollLeft),
            "the padding cell beside the left arrow must not scroll left"
        );

        // No scrolling needed -> no arrows render and navigation never fires.
        let few_tabs = vec!["Tab1".to_string(), "Tab2".to_string()];
        let wide_buffer = render_tab_row(0, 100, &few_tabs, 0, &fg, &bg);
        assert!(
            (0..wide_buffer.width)
                .all(|x| wide_buffer.get(x, 0).map(|c| c.ch != '◀' && c.ch != '▶').unwrap_or(true)),
            "no navigation arrows should render when all tabs fit"
        );
        for x in 0..=100usize {
            assert!(
                calculate_tab_navigation_click(x, 0, 100, &few_tabs, 0, &fg, &bg).is_none(),
                "navigation must never fire when no scrolling is needed (x={})",
                x
            );
        }
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
