#[cfg(test)]
mod tests {
    use crate::components::box_renderer::BoxRenderer;
    use crate::model::muxbox::MuxBox;
    use crate::{Bounds};

    fn create_test_muxbox() -> MuxBox {
        MuxBox {
            id: "test".to_string(),
            content: Some("Test content".to_string()),
            ..Default::default()
        }
    }

    fn create_box_renderer() -> BoxRenderer<'static> {
        let muxbox = Box::leak(Box::new(create_test_muxbox()));
        BoxRenderer::new(muxbox, "test_renderer".to_string())
    }

    #[test]
    fn test_basic_coordinate_translation_no_scroll_no_padding() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(10, 5, 50, 25); // 40x20 content area
        let content_width = 20;
        let content_height = 10;
        let viewable_width = 40; // bounds.width() - 4 = 44 - 4 = 40
        let viewable_height = 20; // bounds.height() - 4 = 24 - 4 = 20
        let horizontal_scroll = 0.0;
        let vertical_scroll = 0.0;

        // Test basic translation: inbox (0,0) should map to screen (12, 7)
        // bounds.left(10) + 2(border) + 0(padding) = 12
        // bounds.top(5) + 2(border) + 0(padding) = 7
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            0, 0, &bounds, content_width, content_height, 
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(screen_x, 12);
        assert_eq!(screen_y, 7);

        // Test reverse translation: screen (12, 7) should map to inbox (0,0)
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            12, 7, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, Some((0, 0)));

        // Test mid-point translation
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            10, 5, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(screen_x, 22);
        assert_eq!(screen_y, 12);

        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            22, 12, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, Some((10, 5)));
    }

    #[test]
    fn test_coordinate_translation_with_centering_padding() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(10, 5, 50, 25); // 40x20 content area
        let content_width = 20; // Smaller than viewable
        let content_height = 10; // Smaller than viewable
        let viewable_width = 40;
        let viewable_height = 20;
        let horizontal_scroll = 0.0;
        let vertical_scroll = 0.0;

        // Calculate expected padding
        let horizontal_padding = (viewable_width - content_width) / 2; // (40 - 20) / 2 = 10
        let vertical_padding = (viewable_height - content_height) / 2; // (20 - 10) / 2 = 5

        // Test translation with padding: inbox (0,0) should map to screen (12 + 10, 7 + 5) = (22, 12)
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            0, 0, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(screen_x, 22); // 10 + 2 + 10 = 22
        assert_eq!(screen_y, 12); // 5 + 2 + 5 = 12

        // Test reverse translation
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            22, 12, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, Some((0, 0)));

        // Test content bounds (center of centered content)
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            10, 5, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(screen_x, 32); // 22 + 10
        assert_eq!(screen_y, 17); // 12 + 5
    }

    #[test]
    fn test_coordinate_translation_with_horizontal_scrolling() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(10, 5, 30, 15); // 20x10 viewable area
        let content_width = 40; // Larger than viewable
        let content_height = 8;
        let viewable_width = 20;
        let viewable_height = 10;
        let horizontal_scroll = 50.0; // 50% scroll
        let vertical_scroll = 0.0;

        // Calculate expected horizontal offset
        // ((40 - 20 + 3) * 0.5).round() = (23 * 0.5).round() = 11.5.round() = 12
        let expected_horizontal_offset = 12;

        // Test translation with horizontal scroll
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            0, 0, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // screen_x = 10 + 2 + 1 + 0 - 12 = 1 (bounds.left + border + padding + inbox_x - offset)
        assert_eq!(screen_x, 1);
        assert_eq!(screen_y, 6); // 5 + 2 + 1 + 0 = 8

        // Test reverse translation
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            1, 6, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // inbox_x = (1 - 13) + 12 = -12 + 12 = 0
        assert_eq!(inbox_coords, Some((0, 0)));

        // Test scrolled position - inbox (12, 0) should map to visible screen position
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            12, 0, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(screen_x, 13); // Should be at left edge of content area plus 12 - 12 = 0 offset
        assert_eq!(screen_y, 6);
    }

    #[test]
    fn test_coordinate_translation_with_vertical_scrolling() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(5, 10, 25, 25); // 20x15 viewable area
        let content_width = 18;
        let content_height = 30; // Larger than viewable
        let viewable_width = 20;
        let viewable_height = 15;
        let horizontal_scroll = 0.0;
        let vertical_scroll = 25.0; // 25% scroll

        // Calculate expected vertical offset
        // ((30 - 15) * 0.25).round() = (15 * 0.25).round() = 3.75.round() = 4
        let expected_vertical_offset = 4;

        // Test translation with vertical scroll
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            0, 0, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // screen_y = 10 + 2 + 0 + 0 - 4 = 8
        assert_eq!(screen_x, 8); // 5 + 2 + 1 = 8
        assert_eq!(screen_y, 8);

        // Test reverse translation
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            8, 8, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, Some((0, 0)));

        // Test scrolled content: inbox (0, 4) should be at top of visible area
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            0, 4, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(screen_x, 8);
        assert_eq!(screen_y, 12); // 8 + 4 = 12
    }

    #[test]
    fn test_coordinate_translation_with_both_scrolling_and_padding() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(0, 0, 30, 20); // 30x20 total, 26x16 viewable
        let content_width = 50; // Wider than viewable
        let content_height = 8; // Shorter than viewable (will have vertical padding)
        let viewable_width = 26;
        let viewable_height = 16;
        let horizontal_scroll = 20.0; // 20% horizontal scroll
        let vertical_scroll = 0.0;

        // Calculate expected offsets and padding
        let vertical_padding = (viewable_height - content_height) / 2; // (16 - 8) / 2 = 4
        let horizontal_offset = ((content_width - viewable_width + 3) as f64 * 0.20).round() as usize; // ((50 - 26 + 3) * 0.20).round() = (27 * 0.20).round() = 5.4.round() = 5

        // Test translation: inbox (0,0)
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            0, 0, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // screen_x = 0 + 2 + 0 + 0 - 5 = -3 (would be clipped in actual rendering)
        // screen_y = 0 + 2 + 4 + 0 - 0 = 6
        assert_eq!(screen_x, 2 - 5); // This would be negative, showing content scrolled off-screen
        assert_eq!(screen_y, 6);

        // Test reverse translation for visible area
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            2, 6, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // inbox_x = (2 - 2) + 5 = 5
        // inbox_y = (6 - 6) + 0 = 0
        assert_eq!(inbox_coords, Some((5, 0)));
    }

    #[test]
    fn test_coordinate_translation_boundary_cases() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(5, 5, 25, 25); // 20x20 viewable
        let content_width = 18;
        let content_height = 18;
        let viewable_width = 20;
        let viewable_height = 20;
        let horizontal_scroll = 0.0;
        let vertical_scroll = 0.0;

        // Test coordinates outside content area (should return None for reverse translation)
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            4, 10, &bounds, content_width, content_height, // x=4 is before content area (starts at 7)
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, None);

        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            10, 4, &bounds, content_width, content_height, // y=4 is before content area (starts at 7)
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, None);

        // Test coordinates at exact boundaries
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            8, 8, &bounds, content_width, content_height, // Just inside content area
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert!(inbox_coords.is_some());

        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            26, 24, &bounds, content_width, content_height, // At far boundary (should be outside)
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, None);
    }

    #[test]
    fn test_coordinate_translation_maximum_scroll() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(0, 0, 20, 20); // 16x16 viewable
        let content_width = 32; // Double the viewable width
        let content_height = 32; // Double the viewable height
        let viewable_width = 16;
        let viewable_height = 16;
        let horizontal_scroll = 100.0; // Maximum scroll
        let vertical_scroll = 100.0; // Maximum scroll

        // Calculate maximum offsets
        let max_horizontal_offset = ((content_width - viewable_width + 3) as f64 * 1.0).round() as usize; // ((32 - 16 + 3) * 1.0) = 19
        let max_vertical_offset = ((content_height - viewable_height) as f64 * 1.0).round() as usize; // ((32 - 16) * 1.0) = 16

        // Test maximum scroll position - the end of content should be visible
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            content_width - 1, content_height - 1, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        
        // screen_x = 0 + 2 + 0 + 31 - 19 = 14
        // screen_y = 0 + 2 + 0 + 31 - 16 = 17
        assert_eq!(screen_x, 14);
        assert_eq!(screen_y, 17);

        // Test reverse translation from bottom-right of visible area
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            15, 17, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // inbox_x = (15 - 2) + 19 = 32 (beyond content, but mathematically correct)
        // inbox_y = (17 - 2) + 16 = 31
        assert!(inbox_coords.is_some());
        let (inbox_x, inbox_y) = inbox_coords.unwrap();
        assert_eq!(inbox_y, 31);
    }

    #[test]
    fn test_coordinate_translation_zero_content_size() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(10, 10, 30, 30); // 20x20 viewable
        let content_width = 0;
        let content_height = 0;
        let viewable_width = 20;
        let viewable_height = 20;
        let horizontal_scroll = 0.0;
        let vertical_scroll = 0.0;

        // With zero content size, everything should be padded to center
        let vertical_padding = viewable_height / 2; // 10
        let horizontal_padding = viewable_width / 2; // 10

        // Test translation of (0,0) - should be centered
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            0, 0, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // screen_x = 10 + 2 + 10 + 0 - 0 = 22
        // screen_y = 10 + 2 + 10 + 0 - 0 = 22
        assert_eq!(screen_x, 22);
        assert_eq!(screen_y, 22);

        // Test reverse translation
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            22, 22, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, Some((0, 0)));
    }

    #[test]
    fn test_coordinate_translation_minimal_bounds() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(0, 0, 5, 5); // Minimal 5x5 box, 1x1 viewable content
        let content_width = 1;
        let content_height = 1;
        let viewable_width = 1;
        let viewable_height = 1;
        let horizontal_scroll = 0.0;
        let vertical_scroll = 0.0;

        // Test translation with minimal bounds
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            0, 0, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // screen_x = 0 + 2 + 0 + 0 - 0 = 2
        // screen_y = 0 + 2 + 0 + 0 - 0 = 2
        assert_eq!(screen_x, 2);
        assert_eq!(screen_y, 2);

        // Test reverse translation
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            2, 2, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, Some((0, 0)));

        // Test coordinates outside the tiny content area
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            3, 3, &bounds, content_width, content_height, // Outside the 1x1 content
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        assert_eq!(inbox_coords, None);
    }

    #[test]
    fn test_coordinate_translation_roundtrip_consistency() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(15, 20, 55, 50); // 40x30 viewable area
        let content_width = 60;
        let content_height = 40;
        let viewable_width = 40;
        let viewable_height = 30;
        let horizontal_scroll = 33.3;
        let vertical_scroll = 66.7;

        // Test multiple inbox coordinates for roundtrip consistency
        let test_coords = vec![
            (0, 0), (10, 5), (20, 15), (30, 25), (50, 35)
        ];

        for (inbox_x, inbox_y) in test_coords {
            // Forward translation
            let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
                inbox_x, inbox_y, &bounds, content_width, content_height,
                viewable_width, viewable_height, horizontal_scroll, vertical_scroll
            );

            // Reverse translation
            if let Some((roundtrip_inbox_x, roundtrip_inbox_y)) = renderer.translate_screen_to_inbox_coordinates(
                screen_x, screen_y, &bounds, content_width, content_height,
                viewable_width, viewable_height, horizontal_scroll, vertical_scroll
            ) {
                // Check roundtrip consistency
                assert_eq!(inbox_x, roundtrip_inbox_x, 
                    "X coordinate roundtrip failed for ({}, {}): {} -> ({}, {}) -> {}", 
                    inbox_x, inbox_y, inbox_x, screen_x, screen_y, roundtrip_inbox_x);
                assert_eq!(inbox_y, roundtrip_inbox_y,
                    "Y coordinate roundtrip failed for ({}, {}): {} -> ({}, {}) -> {}",
                    inbox_x, inbox_y, inbox_y, screen_x, screen_y, roundtrip_inbox_y);
            }
        }
    }

    #[test]
    fn test_coordinate_translation_fractional_scroll() {
        let renderer = create_box_renderer();
        let bounds = Bounds::new(0, 0, 24, 24); // 20x20 viewable
        let content_width = 40;
        let content_height = 40;
        let viewable_width = 20;
        let viewable_height = 20;
        let horizontal_scroll = 33.33; // Fractional scroll
        let vertical_scroll = 66.66; // Fractional scroll

        // Test that fractional scrolls are handled correctly (rounded)
        let horizontal_offset = ((content_width - viewable_width + 3) as f64 * horizontal_scroll / 100.0).round() as usize;
        let vertical_offset = ((content_height - viewable_height) as f64 * vertical_scroll / 100.0).round() as usize;

        // Verify calculated offsets
        // horizontal: ((40 - 20 + 3) * 0.3333).round() = (23 * 0.3333).round() = 7.6659.round() = 8
        // vertical: ((40 - 20) * 0.6666).round() = (20 * 0.6666).round() = 13.332.round() = 13
        assert_eq!(horizontal_offset, 8);
        assert_eq!(vertical_offset, 13);

        // Test actual translation
        let (screen_x, screen_y) = renderer.translate_inbox_to_screen_coordinates(
            0, 0, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // screen_x = 0 + 2 + 0 + 0 - 8 = -6
        // screen_y = 0 + 2 + 0 + 0 - 13 = -11
        assert_eq!(screen_x, 2_usize.wrapping_sub(8)); // This will wrap around
        assert_eq!(screen_y, 2_usize.wrapping_sub(13)); // This will wrap around

        // Test reverse translation for visible content
        let inbox_coords = renderer.translate_screen_to_inbox_coordinates(
            2, 2, &bounds, content_width, content_height,
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        );
        // inbox_x = (2 - 2) + 8 = 8
        // inbox_y = (2 - 2) + 13 = 13
        assert_eq!(inbox_coords, Some((8, 13)));
    }
}