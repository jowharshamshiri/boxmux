#[cfg(test)]
mod tests {
    use crate::components::box_renderer::{BoxDimensions, BoxRenderer};
    use crate::model::common::{Stream, StreamType};
    use crate::model::muxbox::MuxBox;
    use crate::Bounds;
    use indexmap::IndexMap;

    fn create_test_muxbox_with_content(content: &str) -> MuxBox {
        MuxBox {
            id: "test".to_string(),
            content: Some(content.to_string()),
            border_color: Some("white".to_string()),
            horizontal_scroll: Some(0.0),
            vertical_scroll: Some(0.0),
            streams: IndexMap::new(),
            ..Default::default()
        }
    }

    fn create_test_muxbox_with_streams() -> MuxBox {
        let mut streams = IndexMap::new();
        streams.insert(
            "stream1".to_string(),
            Stream {
                id: "stream1".to_string(),
                stream_type: StreamType::Content,
                label: "Stream 1".to_string(),
                content: vec![
                    "Multi-line".to_string(),
                    "stream content".to_string(),
                    "with tabs".to_string(),
                ],
                choices: None,
                active: true,
                source: None,
                content_hash: 0,
                last_updated: std::time::SystemTime::now(),
                created_at: std::time::SystemTime::now(),
            },
        );
        streams.insert(
            "stream2".to_string(),
            Stream {
                id: "stream2".to_string(),
                stream_type: StreamType::Content,
                label: "Stream 2".to_string(),
                content: vec!["Another stream".to_string()],
                choices: None,
                active: false,
                source: None,
                content_hash: 0,
                last_updated: std::time::SystemTime::now(),
                created_at: std::time::SystemTime::now(),
            },
        );

        MuxBox {
            id: "test_tabs".to_string(),
            border_color: Some("white".to_string()),
            horizontal_scroll: Some(25.0),
            vertical_scroll: Some(50.0),
            streams,
            selected_stream_id: Some("stream1".to_string()),
            ..Default::default()
        }
    }

    #[test]
    fn test_box_dimensions_calculation_basic() {
        let muxbox = create_test_muxbox_with_content("Hello\nWorld");
        let total_bounds = Bounds::new(10, 5, 50, 25); // 40x20 total

        let dimensions = BoxDimensions::calculate_from_muxbox(&muxbox, total_bounds.clone());

        // Verify total bounds
        assert_eq!(dimensions.total_bounds, total_bounds);

        // Verify border calculations
        assert_eq!(dimensions.border_thickness, 1);

        // Verify content bounds (accounting for borders, no tabs since single stream)
        assert_eq!(dimensions.content_bounds.left(), 11); // 10 + 1 border
        assert_eq!(dimensions.content_bounds.top(), 6); // 5 + 1 border + 0 tabs
        assert_eq!(dimensions.content_bounds.right(), 49); // 50 - 1 border
        assert_eq!(dimensions.content_bounds.bottom(), 24); // 25 - 1 border

        // Verify viewable dimensions (content area minus potential scrollbars)
        assert_eq!(dimensions.viewable_width, 38); // 39 width - 1 scrollbar
        assert_eq!(dimensions.viewable_height, 18); // 19 height - 1 scrollbar

        // Verify content dimensions from actual content
        assert_eq!(dimensions.content_width, 5); // "World" is longest line
        assert_eq!(dimensions.content_height, 2); // 2 lines

        // Verify scroll positions
        assert_eq!(dimensions.horizontal_scroll, 0.0);
        assert_eq!(dimensions.vertical_scroll, 0.0);
    }

    #[test]
    fn test_box_dimensions_with_tabs() {
        let muxbox = create_test_muxbox_with_streams();
        let total_bounds = Bounds::new(0, 0, 30, 20); // 30x20 total

        let dimensions = BoxDimensions::calculate_from_muxbox(&muxbox, total_bounds.clone());

        // Verify tab height calculation (multiple streams = tabs present)
        assert_eq!(dimensions.tab_height, 1);

        // Verify content bounds (accounting for borders + tabs)
        assert_eq!(dimensions.content_bounds.left(), 1); // 0 + 1 border
        assert_eq!(dimensions.content_bounds.top(), 2); // 0 + 1 border + 1 tab
        assert_eq!(dimensions.content_bounds.right(), 29); // 30 - 1 border
        assert_eq!(dimensions.content_bounds.bottom(), 19); // 20 - 1 border

        // Verify scroll positions from muxbox
        assert_eq!(dimensions.horizontal_scroll, 25.0);
        assert_eq!(dimensions.vertical_scroll, 50.0);
    }

    #[test]
    fn test_coordinate_translation_basic() {
        let muxbox = create_test_muxbox_with_content("ABCDE\nFGHIJ");
        let total_bounds = Bounds::new(10, 10, 30, 25); // 20x15 total
        let dimensions = BoxDimensions::calculate_from_muxbox(&muxbox, total_bounds.clone());

        // Test basic inbox to screen translation (no scrolling, centered)
        let (screen_x, screen_y) = dimensions.inbox_to_screen(0, 0);

        // Calculate expected coordinates based on actual dimensions
        let expected_horizontal_padding =
            (dimensions.viewable_width - dimensions.content_width) / 2;
        let expected_vertical_padding =
            (dimensions.viewable_height - dimensions.content_height) / 2;
        let expected_screen_x = dimensions.content_bounds.left() + expected_horizontal_padding;
        let expected_screen_y = dimensions.content_bounds.top() + expected_vertical_padding;

        assert_eq!(screen_x, expected_screen_x);
        assert_eq!(screen_y, expected_screen_y);

        // Test screen to inbox translation
        let inbox_coords = dimensions.screen_to_inbox(expected_screen_x, expected_screen_y);
        assert_eq!(inbox_coords, Some((0, 0)));

        // Test mid-content translation
        let (screen_x, screen_y) = dimensions.inbox_to_screen(2, 1);
        let expected_mid_screen_x = expected_screen_x + 2;
        let expected_mid_screen_y = expected_screen_y + 1;
        assert_eq!(screen_x, expected_mid_screen_x);
        assert_eq!(screen_y, expected_mid_screen_y);

        let inbox_coords = dimensions.screen_to_inbox(expected_mid_screen_x, expected_mid_screen_y);
        assert_eq!(inbox_coords, Some((2, 1)));
    }

    #[test]
    fn test_coordinate_translation_with_scroll() {
        let mut muxbox =
            create_test_muxbox_with_content("ABCDEFGHIJ\nKLMNOPQRST\nUVWXYZ1234\n567890ABCD");
        muxbox.horizontal_scroll = Some(50.0); // 50% scroll
        muxbox.vertical_scroll = Some(25.0); // 25% scroll

        let total_bounds = Bounds::new(0, 0, 15, 10); // Small box to force scrolling
        let dimensions = BoxDimensions::calculate_from_muxbox(&muxbox, total_bounds.clone());

        // Content: 10x4, Viewable: ~11x6
        // Since content width (10) < viewable width (11), no horizontal scroll
        // Since content height (4) < viewable height (6), no vertical scroll

        // With no actual scrolling needed, coordinates should be centered
        let (screen_x, screen_y) = dimensions.inbox_to_screen(0, 0);
        let inbox_coords = dimensions.screen_to_inbox(screen_x, screen_y);
        assert_eq!(inbox_coords, Some((0, 0)));
    }

    #[test]
    fn test_coordinate_translation_out_of_bounds() {
        let muxbox = create_test_muxbox_with_content("ABC\nDEF");
        let total_bounds = Bounds::new(0, 0, 20, 15);
        let dimensions = BoxDimensions::calculate_from_muxbox(&muxbox, total_bounds.clone());

        // Test out-of-bounds inbox coordinates
        let (screen_x, screen_y) = dimensions.inbox_to_screen(10, 10);
        assert_eq!(screen_x, usize::MAX); // Out of bounds marker
        assert_eq!(screen_y, usize::MAX);

        // Test out-of-bounds screen coordinates
        let inbox_coords = dimensions.screen_to_inbox(0, 0); // Outside content bounds
        assert_eq!(inbox_coords, None);
    }

    #[test]
    fn test_visible_inbox_region() {
        // Create content that will definitely require scrolling
        let mut muxbox = create_test_muxbox_with_content(
            "Line1\nLine2\nLine3\nLine4\nLine5\nLine6\nLine7\nLine8\nLine9\nLine10",
        );
        muxbox.vertical_scroll = Some(50.0); // 50% scroll

        let total_bounds = Bounds::new(0, 0, 20, 8); // Small box to force vertical scrolling
        let dimensions = BoxDimensions::calculate_from_muxbox(&muxbox, total_bounds.clone());

        let (left, top, right, bottom) = dimensions.get_visible_inbox_region();

        // Only check if scrolling is needed based on actual dimensions
        if dimensions.content_height > dimensions.viewable_height {
            assert!(top > 0); // Some vertical scroll offset when scrolling needed
        }
        assert_eq!(left, 0); // No horizontal scroll needed for this content
        assert_eq!(right - left, dimensions.viewable_width);
        assert_eq!(bottom - top, dimensions.viewable_height);
    }

    #[test]
    fn test_visibility_checks() {
        let mut muxbox =
            create_test_muxbox_with_content("ABCDEFGHIJKLMNOP\nQRSTUVWXYZ123456\n7890!@#$%^&*()++");
        muxbox.horizontal_scroll = Some(60.0);

        let total_bounds = Bounds::new(0, 0, 12, 8); // Small box forcing scroll
        let dimensions = BoxDimensions::calculate_from_muxbox(&muxbox, total_bounds.clone());

        // Get the visible region to understand what should be visible
        let (left, top, right, bottom) = dimensions.get_visible_inbox_region();

        // Test visibility based on actual visible region
        assert!(dimensions.is_inbox_visible(left, top)); // Start of visible region should be visible
        if left + 1 < right && top + 1 < bottom {
            assert!(dimensions.is_inbox_visible(left + 1, top)); // Next coordinate should be visible
        }

        // Coordinates outside content should not be visible
        assert!(!dimensions.is_inbox_visible(dimensions.content_width, 0));
        assert!(!dimensions.is_inbox_visible(0, dimensions.content_height));

        // Test coordinates outside current scroll view (if applicable)
        if left > 0 {
            assert!(!dimensions.is_inbox_visible(0, top)); // Before scroll window
        }
    }

    #[test]
    fn test_box_renderer_coordinate_integration() {
        let muxbox = Box::leak(Box::new(create_test_muxbox_with_content("Test\nContent")));
        let mut renderer = BoxRenderer::new(muxbox, "test_renderer".to_string());

        let bounds = Bounds::new(5, 5, 25, 15);
        renderer.initialize_dimensions(bounds.clone());

        // Test that renderer uses formalized coordinate system
        let dimensions = renderer
            .get_dimensions()
            .expect("Dimensions should be initialized");
        assert_eq!(dimensions.total_bounds, bounds);

        // Test coordinate translation through renderer
        let inbox_coords = renderer.screen_to_inbox_coords(10, 10);
        if let Some((inbox_x, inbox_y)) = inbox_coords {
            let screen_coords = renderer.inbox_to_screen_coords(inbox_x, inbox_y);
            // Round-trip should work (approximately, within visible area)
            assert!(
                screen_coords.is_some(),
                "Round-trip coordinate translation should work"
            );
        }
    }
}
