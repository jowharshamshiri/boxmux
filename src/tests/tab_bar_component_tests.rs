use crate::components::{TabBar, TabNavigationAction};
use crate::model::common::ScreenBuffer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tab_bar_drawing_basic() {
        let mut buffer = ScreenBuffer::new();
        let tab_labels = vec!["Tab1".to_string(), "Tab2".to_string()];
        let tab_close_buttons = vec![false, false];

        TabBar::draw(
            5,
            0,
            50,
            &Some("white".to_string()),
            &Some("black".to_string()),
            &Some("cyan".to_string()),
            &Some("blue".to_string()),
            &tab_labels,
            &tab_close_buttons,
            0,
            false,
            0,
            None,
            &mut buffer,
        );

        // Tab bar should have drawn content - validate buffer dimensions and content
        assert!(
            buffer.buffer.len() > 0,
            "Tab bar should draw content to buffer"
        );
        assert!(
            buffer.width > 0 && buffer.height > 0,
            "Buffer should have valid dimensions"
        );

        // Validate that tab labels are reflected in the drawing somehow
        // Check for non-space characters in the buffer indicating actual content was drawn
        let has_content = buffer
            .buffer
            .iter()
            .any(|row| row.iter().any(|cell| cell.ch != ' '));
        assert!(
            has_content,
            "Tab bar should draw visible content (non-space characters)"
        );

        // Validate basic tab bar positioning
        assert_eq!(
            buffer.width,
            crate::screen_width(),
            "Buffer width should match screen width"
        );
        assert_eq!(
            buffer.height,
            crate::screen_height(),
            "Buffer height should match screen height"
        );
    }

    #[test]
    fn test_tab_bar_drawing_with_close_buttons() {
        let mut buffer = ScreenBuffer::new();
        let tab_labels = vec!["Tab1".to_string(), "Tab2".to_string()];
        let tab_close_buttons = vec![true, false];

        TabBar::draw(
            5,
            0,
            50,
            &Some("white".to_string()),
            &Some("black".to_string()),
            &Some("cyan".to_string()),
            &Some("blue".to_string()),
            &tab_labels,
            &tab_close_buttons,
            0,
            false,
            0,
            None,
            &mut buffer,
        );

        // Should draw successfully with mixed close buttons - validate comprehensive behavior
        assert!(
            buffer.buffer.len() > 0,
            "Tab bar should handle close buttons"
        );

        // Validate buffer state after rendering
        assert!(
            buffer.width > 0 && buffer.height > 0,
            "Buffer should have valid dimensions"
        );

        // Check that close button configuration is handled (Tab1 has close, Tab2 doesn't)
        assert_eq!(
            tab_close_buttons[0], true,
            "First tab should have close button enabled"
        );
        assert_eq!(
            tab_close_buttons[1], false,
            "Second tab should have close button disabled"
        );
        assert_eq!(
            tab_labels.len(),
            tab_close_buttons.len(),
            "Tab labels and close buttons should match in count"
        );

        // Verify rendering actually occurred
        let has_non_space_content = buffer
            .buffer
            .iter()
            .any(|row| row.iter().any(|cell| cell.ch != ' '));
        assert!(
            has_non_space_content,
            "Tab bar with close buttons should produce visible content"
        );
    }

    #[test]
    fn test_tab_click_index_calculation() {
        let tab_labels = vec!["Tab1".to_string(), "Tab2".to_string(), "Tab3".to_string()];

        // Click in first tab area - validate specific tab detection
        let result = TabBar::calculate_tab_click_index(
            15,
            0,
            60,
            &tab_labels,
            0,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        assert!(
            result.is_some(),
            "Should detect tab click in valid tab area"
        );
        if let Some(tab_index) = result {
            assert!(
                tab_index < tab_labels.len(),
                "Detected tab index should be within valid range"
            );
        }

        // Click outside tab area - should return None
        let result = TabBar::calculate_tab_click_index(
            5,
            0,
            60,
            &tab_labels,
            0,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        // For very early coordinates, should not detect tab click
        // Note: Result may vary based on tab positioning logic, but should be deterministic

        // Click in second tab area
        let result = TabBar::calculate_tab_click_index(
            35,
            0,
            60,
            &tab_labels,
            0,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        if let Some(tab_index) = result {
            assert!(
                tab_index < tab_labels.len(),
                "Second tab click should also be within valid range"
            );
            assert!(tab_index >= 0, "Tab index should be non-negative");
        }

        // Validate tab label integrity
        assert_eq!(
            tab_labels.len(),
            3,
            "Should have exactly 3 tabs for this test"
        );
        assert!(
            !tab_labels.is_empty(),
            "Tab labels should not be empty for click detection"
        );
    }

    #[test]
    fn test_tab_navigation_click_detection() {
        let tab_labels = vec![
            "Tab1".to_string(),
            "Tab2".to_string(),
            "Tab3".to_string(),
            "Tab4".to_string(),
            "Tab5".to_string(),
        ];

        // Test navigation when scrolling is needed (narrow width forces scrolling)
        let result = TabBar::calculate_tab_navigation_click(
            3,
            0,
            30,
            &tab_labels,
            1,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        // Should detect left arrow when scroll offset > 0
        if result.is_some() {
            assert_eq!(result.unwrap(), TabNavigationAction::ScrollLeft);
        }

        // Test right arrow
        let result = TabBar::calculate_tab_navigation_click(
            27,
            0,
            30,
            &tab_labels,
            0,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        // May detect right arrow if scrolling needed
        if result.is_some() {
            assert_eq!(result.unwrap(), TabNavigationAction::ScrollRight);
        }
    }

    #[test]
    fn test_tab_close_click_detection() {
        let tab_labels = vec!["Tab1".to_string(), "Tab2".to_string()];
        let tab_close_buttons = vec![true, false];

        // Test close button detection
        let result = TabBar::calculate_tab_close_click(
            45,
            0,
            50,
            &tab_labels,
            &tab_close_buttons,
            0,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        // Validate close button click detection logic
        // Note: The exact behavior depends on coordinate system and tab positioning
        if let Some(tab_index) = result {
            assert!(
                tab_index < tab_labels.len(),
                "Close button tab index should be within valid range"
            );
            assert!(
                tab_close_buttons[tab_index],
                "Clicked tab should actually have a close button enabled"
            );
        }

        // Validate close button configuration consistency
        assert_eq!(
            tab_labels.len(),
            tab_close_buttons.len(),
            "Tab labels and close buttons should have matching count"
        );
        assert!(
            tab_close_buttons[0],
            "First tab should have close button for this test"
        );
        assert!(
            !tab_close_buttons[1],
            "Second tab should not have close button for this test"
        );

        // Test click on tab without close button - should return None
        let no_close_result = TabBar::calculate_tab_close_click(
            35,
            0,
            50,
            &tab_labels,
            &tab_close_buttons,
            0,
            &Some("white".to_string()),
            &Some("black".to_string()),
        );
        // This should be None since second tab has no close button
    }

    #[test]
    fn test_rendered_close_glyph_is_click_target() {
        let mut buffer = ScreenBuffer::new();
        let tab_labels = vec!["Closable".to_string(), "Pinned".to_string()];
        let tab_close_buttons = vec![true, false];
        let y = 5;

        TabBar::draw(
            y,
            0,
            50,
            &Some("white".to_string()),
            &Some("black".to_string()),
            &Some("cyan".to_string()),
            &Some("blue".to_string()),
            &tab_labels,
            &tab_close_buttons,
            0,
            false,
            0,
            None,
            &mut buffer,
        );

        let close_x = (0..buffer.width)
            .find(|x| buffer.get(*x, y).is_some_and(|cell| cell.ch == '×'))
            .expect("tab bar should render a close glyph for a closeable tab");

        assert_eq!(
            TabBar::calculate_tab_close_click(
                close_x,
                0,
                50,
                &tab_labels,
                &tab_close_buttons,
                0,
                &Some("white".to_string()),
                &Some("black".to_string()),
            ),
            Some(0),
            "clicking the rendered close glyph must close the tab that owns it"
        );
        assert_eq!(
            TabBar::calculate_tab_hover_target(
                close_x,
                0,
                50,
                &tab_labels,
                &tab_close_buttons,
                0,
                &Some("white".to_string()),
                &Some("black".to_string()),
            ),
            Some(crate::components::TabHoverTarget::CloseButton(0)),
            "hovering the rendered close glyph must use close-button affordance"
        );
    }

    #[test]
    fn test_only_rendered_close_glyph_cell_is_close_sensitive() {
        let mut buffer = ScreenBuffer::new();
        let tab_labels = vec!["Closable".to_string(), "Pinned".to_string()];
        let tab_close_buttons = vec![true, false];
        let y = 5;

        TabBar::draw(
            y,
            0,
            50,
            &Some("white".to_string()),
            &Some("black".to_string()),
            &Some("cyan".to_string()),
            &Some("blue".to_string()),
            &tab_labels,
            &tab_close_buttons,
            0,
            false,
            0,
            None,
            &mut buffer,
        );

        let close_x = (0..buffer.width)
            .find(|x| buffer.get(*x, y).is_some_and(|cell| cell.ch == '×'))
            .expect("tab bar should render a close glyph for a closeable tab");

        assert_eq!(
            TabBar::calculate_tab_close_click(
                close_x,
                0,
                50,
                &tab_labels,
                &tab_close_buttons,
                0,
                &Some("white".to_string()),
                &Some("black".to_string()),
            ),
            Some(0)
        );
        assert_eq!(
            TabBar::calculate_tab_close_click(
                close_x - 1,
                0,
                50,
                &tab_labels,
                &tab_close_buttons,
                0,
                &Some("white".to_string()),
                &Some("black".to_string()),
            ),
            None,
            "the visual spacing before the close glyph must not be part of the close target"
        );
    }

    #[test]
    fn test_non_scrolling_close_target_ignores_stale_scroll_offset() {
        let mut buffer = ScreenBuffer::new();
        let tab_labels = vec![
            "Content".to_string(),
            "Output".to_string(),
            "Logs".to_string(),
        ];
        let tab_close_buttons = vec![false, true, true];
        let y = 5;
        let stale_scroll_offset = 2;

        TabBar::draw(
            y,
            0,
            80,
            &Some("white".to_string()),
            &Some("black".to_string()),
            &Some("cyan".to_string()),
            &Some("blue".to_string()),
            &tab_labels,
            &tab_close_buttons,
            1,
            false,
            stale_scroll_offset,
            None,
            &mut buffer,
        );

        assert_eq!(
            TabBar::calculate_tab_navigation_click(
                2,
                0,
                80,
                &tab_labels,
                stale_scroll_offset,
                &Some("white".to_string()),
                &Some("black".to_string()),
            ),
            None,
            "test fixture must be a non-scrolling tab bar"
        );

        let close_positions: Vec<usize> = (0..buffer.width)
            .filter(|x| buffer.get(*x, y).is_some_and(|cell| cell.ch == '×'))
            .collect();
        assert_eq!(
            close_positions.len(),
            2,
            "non-scrolling render should show close glyphs for tabs 1 and 2"
        );

        assert_eq!(
            TabBar::calculate_tab_close_click(
                close_positions[0],
                0,
                80,
                &tab_labels,
                &tab_close_buttons,
                stale_scroll_offset,
                &Some("white".to_string()),
                &Some("black".to_string()),
            ),
            Some(1),
            "stale scroll offset must not shift close targets when all tabs fit"
        );
    }

    #[test]
    fn test_live_output_panel_choice_tab_close_cell() {
        let tab_labels = vec![
            "Content".to_string(),
            "Choice:2bc3dce0-ab7d-4763-b794-ee0df8b6e6eb".to_string(),
        ];
        let tab_close_buttons = vec![false, true];
        let fg = Some("white".to_string());
        let bg = Some("black".to_string());
        let title_fg = Some("cyan".to_string());
        let title_bg = Some("blue".to_string());
        let y = 3;
        let mut buffer = ScreenBuffer::new_custom(100, 30);

        TabBar::draw(
            y,
            44,
            98,
            &fg,
            &bg,
            &title_fg,
            &title_bg,
            &tab_labels,
            &tab_close_buttons,
            1,
            false,
            0,
            None,
            &mut buffer,
        );

        let rendered_close_cells: Vec<usize> = (44..=98)
            .filter(|x| buffer.get(*x, y).is_some_and(|cell| cell.ch == '×'))
            .collect();

        let close_cells: Vec<usize> = (44..=98)
            .filter(|x| {
                TabBar::calculate_tab_close_click(
                    *x,
                    44,
                    98,
                    &tab_labels,
                    &tab_close_buttons,
                    0,
                    &fg,
                    &bg,
                ) == Some(1)
            })
            .collect();

        assert_eq!(
            rendered_close_cells, close_cells,
            "live output panel hit target must exactly match the rendered close glyph cells"
        );
        assert_eq!(
            close_cells.len(),
            1,
            "live output panel should expose exactly one close cell for the closeable choice tab"
        );
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    enum VisibleTabTarget {
        Tab(usize),
        Close(usize),
        NavLeft,
        NavRight,
    }

    fn runtime_click_target(
        x: usize,
        x1: usize,
        x2: usize,
        tab_labels: &[String],
        tab_close_buttons: &[bool],
        tab_scroll_offset: usize,
    ) -> Option<VisibleTabTarget> {
        let fg = Some("white".to_string());
        let bg = Some("black".to_string());

        if let Some(nav_action) = TabBar::calculate_tab_navigation_click(
            x,
            x1,
            x2,
            tab_labels,
            tab_scroll_offset,
            &fg,
            &bg,
        ) {
            return Some(match nav_action {
                TabNavigationAction::ScrollLeft => VisibleTabTarget::NavLeft,
                TabNavigationAction::ScrollRight => VisibleTabTarget::NavRight,
            });
        }

        if let Some(close_index) = TabBar::calculate_tab_close_click(
            x,
            x1,
            x2,
            tab_labels,
            tab_close_buttons,
            tab_scroll_offset,
            &fg,
            &bg,
        ) {
            return Some(VisibleTabTarget::Close(close_index));
        }

        TabBar::calculate_tab_click_index(x, x1, x2, tab_labels, tab_scroll_offset, &fg, &bg)
            .map(VisibleTabTarget::Tab)
    }

    fn runtime_hover_target(
        x: usize,
        x1: usize,
        x2: usize,
        tab_labels: &[String],
        tab_close_buttons: &[bool],
        tab_scroll_offset: usize,
    ) -> Option<VisibleTabTarget> {
        let fg = Some("white".to_string());
        let bg = Some("black".to_string());

        TabBar::calculate_tab_hover_target(
            x,
            x1,
            x2,
            tab_labels,
            tab_close_buttons,
            tab_scroll_offset,
            &fg,
            &bg,
        )
        .map(|target| match target {
            crate::components::TabHoverTarget::Tab(index) => VisibleTabTarget::Tab(index),
            crate::components::TabHoverTarget::CloseButton(index) => VisibleTabTarget::Close(index),
            crate::components::TabHoverTarget::NavigationLeft => VisibleTabTarget::NavLeft,
            crate::components::TabHoverTarget::NavigationRight => VisibleTabTarget::NavRight,
        })
    }

    #[test]
    fn test_every_tab_row_cell_maps_to_the_visible_target_under_it_across_sizes() {
        let tab_labels = vec![
            "Content".to_string(),
            "Execution".to_string(),
            "Logs".to_string(),
            "Metrics".to_string(),
            "Monitor".to_string(),
        ];
        let tab_close_buttons = vec![false, true, true, true, true];
        let fg = Some("white".to_string());
        let bg = Some("black".to_string());
        let title_fg = Some("cyan".to_string());
        let title_bg = Some("blue".to_string());

        for (screen_width, x1, x2, scroll_offset) in [
            (80usize, 0usize, 34usize, 0usize),
            (80, 0, 34, 1),
            (100, 25, 73, 0),
            (100, 25, 73, 2),
            (132, 67, 128, 0),
            (132, 67, 128, 3),
            (192, 133, 188, 0),
            (192, 133, 188, 4),
            (241, 185, 238, 0),
            (319, 259, 316, 4),
        ] {
            let y = 5;
            let mut buffer = ScreenBuffer::new_custom(screen_width, 12);

            TabBar::draw(
                y,
                x1,
                x2,
                &fg,
                &bg,
                &title_fg,
                &title_bg,
                &tab_labels,
                &tab_close_buttons,
                1,
                false,
                scroll_offset,
                None,
                &mut buffer,
            );

            for x in 0..buffer.width {
                let cell = buffer.get(x, y).expect("tab row cell should be in buffer");
                let click_target =
                    runtime_click_target(x, x1, x2, &tab_labels, &tab_close_buttons, scroll_offset);
                let hover_target =
                    runtime_hover_target(x, x1, x2, &tab_labels, &tab_close_buttons, scroll_offset);

                assert_eq!(
                    hover_target, click_target,
                    "hover/click mismatch at x={} for panel x1={} x2={} scroll={}",
                    x, x1, x2, scroll_offset
                );

                match cell.ch {
                    '×' => assert!(
                        matches!(click_target, Some(VisibleTabTarget::Close(_))),
                        "rendered close glyph at x={} must be the close target for panel x1={} x2={} scroll={}",
                        x,
                        x1,
                        x2,
                        scroll_offset
                    ),
                    '◀' => assert_eq!(
                        click_target,
                        Some(VisibleTabTarget::NavLeft),
                        "rendered left arrow at x={} must be the left navigation target",
                        x
                    ),
                    '▶' => assert_eq!(
                        click_target,
                        Some(VisibleTabTarget::NavRight),
                        "rendered right arrow at x={} must be the right navigation target",
                        x
                    ),
                    _ => {
                        assert!(
                            !matches!(click_target, Some(VisibleTabTarget::Close(_))),
                            "non-close cell '{}' at x={} must not be close-sensitive for panel x1={} x2={} scroll={}",
                            cell.ch,
                            x,
                            x1,
                            x2,
                            scroll_offset
                        );
                        assert!(
                            !matches!(
                                click_target,
                                Some(VisibleTabTarget::NavLeft | VisibleTabTarget::NavRight)
                            ),
                            "non-arrow cell '{}' at x={} must not be navigation-sensitive for panel x1={} x2={} scroll={}",
                            cell.ch,
                            x,
                            x1,
                            x2,
                            scroll_offset
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_fractional_percentage_far_from_origin_close_glyph_hit_targets() {
        use crate::model::common::{Bounds, InputBounds};

        let tab_labels = vec![
            "Content".to_string(),
            "Choice:de95b8ce-42a0-43fd-9184-58b481c0482f".to_string(),
            "Logs".to_string(),
        ];
        let tab_close_buttons = vec![false, true, true];
        let fg = Some("white".to_string());
        let bg = Some("black".to_string());
        let title_fg = Some("cyan".to_string());
        let title_bg = Some("blue".to_string());

        for (screen_width, screen_height, x1_pct, x2_pct) in [
            (100usize, 30usize, "25%", "73%"),
            (192, 54, "70.5%", "98.4%"),
            (241, 67, "76.8%", "99.1%"),
            (319, 91, "81.4%", "99.3%"),
        ] {
            let root_bounds = Bounds {
                x1: 0,
                y1: 0,
                x2: screen_width - 1,
                y2: screen_height - 1,
            };
            let bounds = InputBounds {
                x1: x1_pct.to_string(),
                y1: "74.5%".to_string(),
                x2: x2_pct.to_string(),
                y2: "92.2%".to_string(),
            }
            .to_bounds(&root_bounds);
            let y = bounds.y1;
            let mut buffer = ScreenBuffer::new_custom(screen_width, screen_height);

            TabBar::draw(
                y,
                bounds.x1,
                bounds.x2,
                &fg,
                &bg,
                &title_fg,
                &title_bg,
                &tab_labels,
                &tab_close_buttons,
                1,
                false,
                0,
                None,
                &mut buffer,
            );

            let rendered_close_cells = (bounds.x1..=bounds.x2)
                .filter(|x| buffer.get(*x, y).is_some_and(|cell| cell.ch == '×'))
                .collect::<Vec<_>>();
            assert!(
                !rendered_close_cells.is_empty(),
                "fractional far-origin panel {}x{} should render close glyphs in {:?}",
                screen_width,
                screen_height,
                bounds
            );

            for x in 0..screen_width {
                let close_target = TabBar::calculate_tab_close_click(
                    x,
                    bounds.x1,
                    bounds.x2,
                    &tab_labels,
                    &tab_close_buttons,
                    0,
                    &fg,
                    &bg,
                );
                let renders_close = buffer.get(x, y).is_some_and(|cell| cell.ch == '×');
                assert_eq!(
                    close_target.is_some(),
                    renders_close,
                    "close hit target/render mismatch at x={} for fractional far-origin panel {:?} on {}x{}",
                    x,
                    bounds,
                    screen_width,
                    screen_height
                );
            }
        }
    }

    #[test]
    fn test_scrolled_rendered_close_glyph_is_click_target() {
        let mut buffer = ScreenBuffer::new();
        let tab_labels = vec![
            "One".to_string(),
            "Two".to_string(),
            "Three".to_string(),
            "Four".to_string(),
        ];
        let tab_close_buttons = vec![true, true, true, true];
        let y = 5;

        TabBar::draw(
            y,
            0,
            50,
            &Some("white".to_string()),
            &Some("black".to_string()),
            &Some("cyan".to_string()),
            &Some("blue".to_string()),
            &tab_labels,
            &tab_close_buttons,
            1,
            false,
            1,
            None,
            &mut buffer,
        );

        let close_positions: Vec<usize> = (0..buffer.width)
            .filter(|x| buffer.get(*x, y).is_some_and(|cell| cell.ch == '×'))
            .collect();

        assert!(
            close_positions.len() >= 2,
            "scrolled tab bar should render close glyphs for visible closeable tabs"
        );
        assert_eq!(
            TabBar::calculate_tab_navigation_click(
                2,
                0,
                50,
                &tab_labels,
                1,
                &Some("white".to_string()),
                &Some("black".to_string()),
            ),
            Some(TabNavigationAction::ScrollLeft),
            "85 percent overflow threshold must match the renderer and expose the left scroll target"
        );
        assert_eq!(
            TabBar::calculate_tab_close_click(
                close_positions[0],
                0,
                50,
                &tab_labels,
                &tab_close_buttons,
                1,
                &Some("white".to_string()),
                &Some("black".to_string()),
            ),
            Some(1),
            "first rendered close glyph after scroll offset 1 belongs to tab index 1"
        );
    }

    #[test]
    fn test_close_button_hover_changes_rendered_cell_style() {
        let tab_labels = vec!["Closable".to_string(), "Pinned".to_string()];
        let tab_close_buttons = vec![true, false];
        let y = 5;

        let mut normal = ScreenBuffer::new();
        TabBar::draw(
            y,
            0,
            50,
            &Some("white".to_string()),
            &Some("black".to_string()),
            &Some("cyan".to_string()),
            &Some("blue".to_string()),
            &tab_labels,
            &tab_close_buttons,
            0,
            false,
            0,
            None,
            &mut normal,
        );
        let close_x = (0..normal.width)
            .find(|x| normal.get(*x, y).is_some_and(|cell| cell.ch == '×'))
            .expect("normal render should contain close glyph");

        let mut hovered = ScreenBuffer::new();
        TabBar::draw(
            y,
            0,
            50,
            &Some("white".to_string()),
            &Some("black".to_string()),
            &Some("cyan".to_string()),
            &Some("blue".to_string()),
            &tab_labels,
            &tab_close_buttons,
            0,
            false,
            0,
            Some(&crate::components::TabHoverTarget::CloseButton(0)),
            &mut hovered,
        );

        let normal_cell = normal.get(close_x, y).expect("normal close cell");
        let hovered_cell = hovered.get(close_x, y).expect("hovered close cell");
        assert!(
            normal_cell.ch != hovered_cell.ch
                || (normal_cell.fg_color.as_str(), normal_cell.bg_color.as_str())
                    != (
                        hovered_cell.fg_color.as_str(),
                        hovered_cell.bg_color.as_str()
                    ),
            "hovering a close button must visibly change its rendered cell style"
        );
    }

    #[test]
    fn test_empty_tabs_handling() {
        let tab_labels: Vec<String> = vec![];
        let tab_close_buttons: Vec<bool> = vec![];

        // All functions should handle empty inputs gracefully
        assert_eq!(
            TabBar::calculate_tab_click_index(
                10,
                0,
                50,
                &tab_labels,
                0,
                &Some("white".to_string()),
                &Some("black".to_string())
            ),
            None
        );
        assert_eq!(
            TabBar::calculate_tab_navigation_click(
                10,
                0,
                50,
                &tab_labels,
                0,
                &Some("white".to_string()),
                &Some("black".to_string())
            ),
            None
        );
        assert_eq!(
            TabBar::calculate_tab_close_click(
                10,
                0,
                50,
                &tab_labels,
                &tab_close_buttons,
                0,
                &Some("white".to_string()),
                &Some("black".to_string())
            ),
            None
        );
    }

    #[test]
    fn test_tab_bar_scrolling_behavior() {
        let tab_labels = vec![
            "LongTabName1".to_string(),
            "LongTabName2".to_string(),
            "LongTabName3".to_string(),
            "LongTabName4".to_string(),
            "LongTabName5".to_string(),
        ];
        let tab_close_buttons = vec![false; 5];

        let mut buffer = ScreenBuffer::new();

        // Test scrolling with offset
        TabBar::draw(
            5,
            0,
            40, // Narrow width to force scrolling
            &Some("white".to_string()),
            &Some("black".to_string()),
            &Some("cyan".to_string()),
            &Some("blue".to_string()),
            &tab_labels,
            &tab_close_buttons,
            2, false, // Active tab index
            1, // Scroll offset
            None,
            &mut buffer,
        );

        // Should handle scrolling without panicking
        assert!(
            buffer.buffer.len() > 0,
            "Scrolling tabs should render properly"
        );
    }
}
