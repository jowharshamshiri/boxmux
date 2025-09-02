use crate::model::common::ScreenBuffer;
use crate::draw_utils::{
    draw_horizontal_line, print_with_color_and_background_at, fill_horizontal_background
};

/// Tab bar component for displaying stream tabs with scrolling support
pub struct TabBar;

/// Navigation action for tab scrolling
#[derive(Debug, Clone, PartialEq)]
pub enum TabNavigationAction {
    ScrollLeft,
    ScrollRight,
}

impl TabBar {
    /// Draw tab bar with all functionality preserved from original implementation
    pub fn draw(
        y: usize,
        x1: usize,
        x2: usize,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
        title_fg_color: &Option<String>,
        title_bg_color: &Option<String>,
        tab_labels: &[String],
        tab_close_buttons: &[bool],
        active_tab_index: usize,
        tab_scroll_offset: usize,
        buffer: &mut ScreenBuffer,
    ) {
        let total_width = x2.saturating_sub(x1);

        // FIXED SPACE RESERVATION ARCHITECTURE
        // Always reserve 2 chars on left + 2 chars on right for scroll arrows
        // Space always reserved regardless of whether arrows are shown
        let left_arrow_space = 2;
        let right_arrow_space = 2;
        let border_space = if crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color) { 4 } else { 0 }; // Leading + trailing borders

        // Calculate tab area boundaries with fixed reservations
        let reserved_space = left_arrow_space + right_arrow_space + border_space;
        let tab_area_width = total_width.saturating_sub(reserved_space);

        // Position calculations with fixed layout
        let left_arrow_x = x1 + (if crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color) { 2 } else { 0 });
        let tab_area_start = left_arrow_x + left_arrow_space;
        let tab_area_end = tab_area_start + tab_area_width;
        let right_arrow_x = tab_area_end;
        let trailing_border_x = right_arrow_x + right_arrow_space;

        // Determine if scrolling is needed (85% threshold for smooth transitions)
        let max_tab_width = 16;
        let min_tab_width = 6;
        let ideal_tab_width = if tab_labels.len() > 0 {
            tab_area_width / tab_labels.len()
        } else {
            max_tab_width
        };
        let tab_width = ideal_tab_width.clamp(min_tab_width, max_tab_width);
        let total_tabs_width = tab_labels.len() * tab_width;
        let needs_scrolling = (total_tabs_width as f32) > (tab_area_width as f32 * 0.85);

        // Draw leading border if border colors are not transparent
        if (crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color)) && x1 < x2 {
            draw_horizontal_line(y, x1, x1 + 2, fg_color, bg_color, buffer);
        }

        // Draw left scroll arrow or border fill in reserved space
        if needs_scrolling && tab_scroll_offset > 0 {
            // Show left arrow when can scroll left
            print_with_color_and_background_at(y, left_arrow_x, title_fg_color, bg_color, "◀", buffer);
            print_with_color_and_background_at(
                y,
                left_arrow_x + 1,
                title_fg_color,
                bg_color,
                " ",
                buffer,
            );
        } else {
            // Fill reserved space with border when arrow not needed
            draw_horizontal_line(
                y,
                left_arrow_x,
                left_arrow_x + left_arrow_space,
                fg_color,
                bg_color,
                buffer,
            );
        }

        // Calculate tab dimensions with mandatory separators
        let separator_width = 1; // Always 1 character between tabs
        let visible_tabs;
        let mut adjusted_tab_width;

        if needs_scrolling {
            // When scrolling, calculate maximum tabs that fit with minimum width
            let min_tab_with_separator = min_tab_width + separator_width;
            let max_visible_tabs = tab_area_width / min_tab_with_separator.max(1);
            let remaining_tabs = tab_labels.len().saturating_sub(tab_scroll_offset);
            visible_tabs = max_visible_tabs.min(remaining_tabs).max(1);

            // Use ALL available space - expand tabs to fill completely
            if visible_tabs > 0 {
                let total_separator_space = (visible_tabs.saturating_sub(1)) * separator_width;
                adjusted_tab_width =
                    (tab_area_width.saturating_sub(total_separator_space)) / visible_tabs;
                // Ensure minimum width is respected even when expanding
                adjusted_tab_width = adjusted_tab_width.max(min_tab_width);
            } else {
                adjusted_tab_width = min_tab_width;
            }
        } else {
            visible_tabs = tab_labels.len();

            // Calculate tab width to fit all tabs with separators
            if visible_tabs > 0 {
                let total_separator_space = (visible_tabs.saturating_sub(1)) * separator_width;
                adjusted_tab_width =
                    (tab_area_width.saturating_sub(total_separator_space)) / visible_tabs;
            } else {
                adjusted_tab_width = tab_width;
            }
        }

        // Calculate total width needed for visible tabs
        let total_tabs_width = if visible_tabs > 0 {
            visible_tabs * adjusted_tab_width + (visible_tabs.saturating_sub(1)) * separator_width
        } else {
            0
        };

        // Calculate centering offset for tab group
        let centering_offset = if needs_scrolling {
            0 // No centering when scrolling - fill available space
        } else {
            (tab_area_width.saturating_sub(total_tabs_width)) / 2
        };

        // Draw tabs with proper spacing and centering
        if visible_tabs > 0 {
            let start_offset = if needs_scrolling {
                tab_scroll_offset
            } else {
                0
            };
            let mut tab_x = tab_area_start + centering_offset;

            for i in 0..visible_tabs {
                let tab_index = start_offset + i;
                if tab_index >= tab_labels.len() {
                    break;
                }

                // Draw the tab
                let has_close_button = tab_close_buttons.get(tab_index).copied().unwrap_or(false);
                Self::draw_single_tab(
                    y,
                    tab_x,
                    adjusted_tab_width,
                    &tab_labels[tab_index],
                    tab_index == active_tab_index,
                    has_close_button,
                    title_fg_color,
                    title_bg_color,
                    buffer,
                );
                tab_x += adjusted_tab_width;

                // Draw separator after tab (except for last tab)
                if i < visible_tabs.saturating_sub(1) {
                    print_with_color_and_background_at(y, tab_x, fg_color, bg_color, "│", buffer);
                    tab_x += separator_width;
                }
            }
        }

        // Fill remaining tab area with border (before and after tab group)
        if tab_area_start < tab_area_start + centering_offset {
            draw_horizontal_line(
                y,
                tab_area_start,
                tab_area_start + centering_offset,
                fg_color,
                bg_color,
                buffer,
            );
        }

        let tabs_end = tab_area_start + centering_offset + total_tabs_width;
        if tabs_end < tab_area_end {
            draw_horizontal_line(y, tabs_end, tab_area_end, fg_color, bg_color, buffer);
        }

        // Draw right scroll arrow or border fill in reserved space
        let max_scroll_offset = if tab_labels.len() > 0 {
            tab_labels
                .len()
                .saturating_sub(tab_area_width / tab_width.max(1))
        } else {
            0
        };

        if needs_scrolling && tab_scroll_offset < max_scroll_offset {
            // Show right arrow when can scroll right
            print_with_color_and_background_at(y, right_arrow_x, title_fg_color, bg_color, " ", buffer);
            print_with_color_and_background_at(
                y,
                right_arrow_x + 1,
                title_fg_color,
                bg_color,
                "▶",
                buffer,
            );
        } else {
            // Fill reserved space with border when arrow not needed
            draw_horizontal_line(
                y,
                right_arrow_x,
                right_arrow_x + right_arrow_space,
                fg_color,
                bg_color,
                buffer,
            );
        }

        // Draw trailing border if border colors are not transparent
        if (crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color)) && trailing_border_x < x2 {
            draw_horizontal_line(y, trailing_border_x, x2, fg_color, bg_color, buffer);
        }
    }

    /// Helper function to draw a single tab with consistent styling
    fn draw_single_tab(
        y: usize,
        x: usize,
        width: usize,
        label: &str,
        is_active: bool,
        has_close_button: bool,
        title_fg_color: &Option<String>,
        title_bg_color: &Option<String>,
        buffer: &mut ScreenBuffer,
    ) {
        let (tab_fg, tab_bg) = if is_active {
            (title_bg_color, title_fg_color) // Inverted colors for active tab
        } else {
            (title_fg_color, title_bg_color)
        };

        // F0219: Reserve space for close button if needed
        let close_button_space = if has_close_button { 2 } else { 0 }; // "×" + space
        let available_label_width = width.saturating_sub(close_button_space + 2); // 2 for padding

        // Truncate label to fit available space (character-aware)
        let mut display_label = label.to_string();
        let max_label_chars = available_label_width;

        if display_label.chars().count() > max_label_chars {
            let truncate_chars = max_label_chars.saturating_sub(1);
            display_label = display_label
                .chars()
                .take(truncate_chars)
                .collect::<String>();
            display_label.push('…');
        }

        // Draw tab background first
        fill_horizontal_background(y, x, x + width - 1, tab_fg, tab_bg, buffer);

        if has_close_button {
            // F0219: Position close button with aesthetic spacing when possible
            // Reserve 2 chars minimum for "×" (with space if room), more if width allows
            let min_close_area = 2; // Minimum: "×" + space
            let preferred_close_area = if width >= 8 { 3 } else { min_close_area }; // Preferred: " ×" + space

            let close_area_width = preferred_close_area.min(width.saturating_sub(2)); // Don't take all width
            let close_button_x = x + width - close_area_width;
            let label_space = width.saturating_sub(close_area_width + 1); // 1 for left padding

            // Truncate label if needed to leave room for close button area
            let mut final_label = display_label.clone();
            if final_label.chars().count() > label_space {
                let truncate_chars = label_space.saturating_sub(1); // Leave room for "…"
                if truncate_chars > 0 {
                    final_label = final_label.chars().take(truncate_chars).collect::<String>();
                    final_label.push('…');
                } else {
                    final_label = "…".to_string();
                }
            }

            // Draw label with left padding
            let label_content = format!(" {}", final_label);
            print_with_color_and_background_at(y, x, tab_fg, tab_bg, &label_content, buffer);

            // Draw close button with aesthetic spacing
            if close_area_width >= 3 {
                // Enough room for " ×" pattern
                print_with_color_and_background_at(y, close_button_x, tab_fg, tab_bg, " ×", buffer);
            } else {
                // Minimal space - just "×" at edge-1 position
                print_with_color_and_background_at(y, x + width - 2, tab_fg, tab_bg, "×", buffer);
            }
        } else {
            // Regular tab without close button - center the label
            let tab_content = format!(" {} ", display_label);
            let tab_char_count = tab_content.chars().count();
            let display_chars = tab_char_count.min(width);

            let display_text = if display_chars < tab_char_count {
                tab_content.chars().take(display_chars).collect::<String>()
            } else {
                tab_content
            };

            // Draw tab text
            print_with_color_and_background_at(y, x, tab_fg, tab_bg, &display_text, buffer);
        }
    }

    /// Calculate which tab was clicked based on mouse position
    pub fn calculate_tab_click_index(
        click_x: usize,
        x1: usize,
        x2: usize,
        tab_labels: &[String],
        tab_scroll_offset: usize,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
    ) -> Option<usize> {
        if tab_labels.is_empty() {
            return None;
        }

        let total_width = x2.saturating_sub(x1);

        // FIXED SPACE RESERVATION ARCHITECTURE - matches draw_tab_bar
        let left_arrow_space = 2;
        let right_arrow_space = 2;
        let border_space = if crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color) { 4 } else { 0 };
        let reserved_space = left_arrow_space + right_arrow_space + border_space;
        let tab_area_width = total_width.saturating_sub(reserved_space);

        // Position calculations matching draw_tab_bar
        let left_arrow_x = x1 + (if crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color) { 2 } else { 0 });
        let tab_area_start = left_arrow_x + left_arrow_space;
        let tab_area_end = tab_area_start + tab_area_width;
        let right_arrow_x = tab_area_end;

        // Determine if scrolling is needed (85% threshold)
        let max_tab_width = 16;
        let min_tab_width = 6;
        let ideal_tab_width = if tab_labels.len() > 0 {
            tab_area_width / tab_labels.len()
        } else {
            max_tab_width
        };
        let tab_width = ideal_tab_width.clamp(min_tab_width, max_tab_width);
        let total_tabs_width = tab_labels.len() * tab_width;
        let needs_scrolling = (total_tabs_width as f32) > (tab_area_width as f32 * 0.85);

        // Check for navigation arrow clicks first
        if needs_scrolling {
            // Left arrow click check
            if tab_scroll_offset > 0
                && click_x >= left_arrow_x
                && click_x < left_arrow_x + left_arrow_space
            {
                return None; // Signal scroll left
            }

            // Right arrow click check
            let max_scroll_offset = if tab_labels.len() > 0 {
                tab_labels
                    .len()
                    .saturating_sub(tab_area_width / tab_width.max(1))
            } else {
                0
            };

            if tab_scroll_offset < max_scroll_offset
                && click_x >= right_arrow_x
                && click_x < right_arrow_x + right_arrow_space
            {
                return None; // Signal scroll right
            }
        }

        // Calculate tab dimensions with separators (matching draw_tab_bar logic)
        let separator_width = 1;
        let visible_tabs;
        let mut adjusted_tab_width;

        if needs_scrolling {
            // When scrolling, calculate maximum tabs that fit with minimum width
            let min_tab_with_separator = min_tab_width + separator_width;
            let max_visible_tabs = tab_area_width / min_tab_with_separator.max(1);
            let remaining_tabs = tab_labels.len().saturating_sub(tab_scroll_offset);
            visible_tabs = max_visible_tabs.min(remaining_tabs).max(1);

            // Use ALL available space - expand tabs to fill completely
            if visible_tabs > 0 {
                let total_separator_space = (visible_tabs.saturating_sub(1)) * separator_width;
                adjusted_tab_width =
                    (tab_area_width.saturating_sub(total_separator_space)) / visible_tabs;
                // Ensure minimum width is respected even when expanding
                adjusted_tab_width = adjusted_tab_width.max(min_tab_width);
            } else {
                adjusted_tab_width = min_tab_width;
            }
        } else {
            visible_tabs = tab_labels.len();

            if visible_tabs > 0 {
                let total_separator_space = (visible_tabs.saturating_sub(1)) * separator_width;
                adjusted_tab_width =
                    (tab_area_width.saturating_sub(total_separator_space)) / visible_tabs;
            } else {
                adjusted_tab_width = tab_width;
            }
        }

        let total_tabs_width = if visible_tabs > 0 {
            visible_tabs * adjusted_tab_width + (visible_tabs.saturating_sub(1)) * separator_width
        } else {
            0
        };

        let centering_offset = if needs_scrolling {
            0
        } else {
            (tab_area_width.saturating_sub(total_tabs_width)) / 2
        };

        // Check for tab clicks in tab area
        if click_x >= tab_area_start && click_x < tab_area_end && visible_tabs > 0 {
            let start_offset = if needs_scrolling {
                tab_scroll_offset
            } else {
                0
            };
            let mut tab_x = tab_area_start + centering_offset;

            for i in 0..visible_tabs {
                let tab_index = start_offset + i;
                if tab_index >= tab_labels.len() {
                    break;
                }

                // Check if click is within this tab
                if click_x >= tab_x && click_x < tab_x + adjusted_tab_width {
                    return Some(tab_index);
                }

                tab_x += adjusted_tab_width;

                // Skip separator space (don't register clicks on separators)
                if i < visible_tabs.saturating_sub(1) {
                    tab_x += separator_width;
                }
            }
        }

        None
    }

    /// Calculate if click was on navigation arrows
    pub fn calculate_tab_navigation_click(
        click_x: usize,
        x1: usize,
        x2: usize,
        tab_labels: &[String],
        tab_scroll_offset: usize,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
    ) -> Option<TabNavigationAction> {
        if tab_labels.is_empty() {
            return None;
        }

        let total_width = x2.saturating_sub(x1);

        // FIXED SPACE RESERVATION ARCHITECTURE - matches draw_tab_bar
        let left_arrow_space = 2;
        let right_arrow_space = 2;
        let border_space = if crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color) { 4 } else { 0 };
        let reserved_space = left_arrow_space + right_arrow_space + border_space;
        let tab_area_width = total_width.saturating_sub(reserved_space);

        // Position calculations matching draw_tab_bar
        let left_arrow_x = x1 + (if crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color) { 2 } else { 0 });
        let tab_area_start = left_arrow_x + left_arrow_space;
        let tab_area_end = tab_area_start + tab_area_width;
        let right_arrow_x = tab_area_end;

        // Determine if scrolling is needed (85% threshold)
        let max_tab_width = 16;
        let min_tab_width = 6;
        let ideal_tab_width = if tab_labels.len() > 0 {
            tab_area_width / tab_labels.len()
        } else {
            max_tab_width
        };
        let tab_width = ideal_tab_width.clamp(min_tab_width, max_tab_width);
        let total_tabs_width = tab_labels.len() * tab_width;
        let needs_scrolling = (total_tabs_width as f32) > (tab_area_width as f32 * 0.85);

        if !needs_scrolling {
            return None; // No navigation needed
        }

        // Check left arrow click
        if tab_scroll_offset > 0 && click_x >= left_arrow_x && click_x < left_arrow_x + left_arrow_space
        {
            return Some(TabNavigationAction::ScrollLeft);
        }

        // Check right arrow click
        let max_scroll_offset = if tab_labels.len() > 0 {
            tab_labels
                .len()
                .saturating_sub(tab_area_width / tab_width.max(1))
        } else {
            0
        };

        if tab_scroll_offset < max_scroll_offset
            && click_x >= right_arrow_x
            && click_x < right_arrow_x + right_arrow_space
        {
            return Some(TabNavigationAction::ScrollRight);
        }

        None
    }

    /// Calculate if click was on a close button within a tab
    pub fn calculate_tab_close_click(
        click_x: usize,
        x1: usize,
        x2: usize,
        tab_labels: &[String],
        tab_close_buttons: &[bool],
        tab_scroll_offset: usize,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
    ) -> Option<usize> {
        if tab_labels.is_empty() || tab_close_buttons.is_empty() {
            return None;
        }

        let total_width = x2.saturating_sub(x1);

        // Same space calculations as draw_tab_bar and calculate_tab_click_index
        let left_arrow_space = 2;
        let right_arrow_space = 2;
        let border_space = if crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color) { 4 } else { 0 };
        let tab_area_width =
            total_width.saturating_sub(left_arrow_space + right_arrow_space + border_space);

        let tab_area_start = x1 + if crate::color_utils::should_draw_color(fg_color) || crate::color_utils::should_draw_color(bg_color) { 2 } else { 0 } + left_arrow_space;

        // Calculate tab dimensions
        let max_tab_width = 16;
        let min_tab_width = 6;
        let ideal_tab_width = if tab_labels.len() > 0 {
            tab_area_width / tab_labels.len()
        } else {
            max_tab_width
        };
        let tab_width = ideal_tab_width.clamp(min_tab_width, max_tab_width);
        let total_tabs_width = tab_labels.len() * tab_width;
        let needs_scrolling = total_tabs_width > tab_area_width;

        let separator_width = 1;
        let visible_tabs;
        let mut adjusted_tab_width;

        if needs_scrolling {
            let min_tab_with_separator = min_tab_width + separator_width;
            let max_visible_tabs = tab_area_width / min_tab_with_separator.max(1);
            let remaining_tabs = tab_labels.len().saturating_sub(tab_scroll_offset);
            visible_tabs = max_visible_tabs.min(remaining_tabs).max(1);

            if visible_tabs > 0 {
                let total_separator_space = (visible_tabs.saturating_sub(1)) * separator_width;
                adjusted_tab_width =
                    (tab_area_width.saturating_sub(total_separator_space)) / visible_tabs;
                adjusted_tab_width = adjusted_tab_width.max(min_tab_width);
            } else {
                adjusted_tab_width = min_tab_width;
            }
        } else {
            visible_tabs = tab_labels.len();

            if visible_tabs > 0 {
                let total_separator_space = (visible_tabs.saturating_sub(1)) * separator_width;
                adjusted_tab_width =
                    (tab_area_width.saturating_sub(total_separator_space)) / visible_tabs;
            } else {
                adjusted_tab_width = tab_width;
            }
        }

        // Add centering offset to match the tab drawing positioning
        let total_tabs_width = if visible_tabs > 0 {
            visible_tabs * adjusted_tab_width + (visible_tabs.saturating_sub(1)) * separator_width
        } else {
            0
        };

        let centering_offset = if needs_scrolling {
            0
        } else {
            (tab_area_width.saturating_sub(total_tabs_width)) / 2
        };

        // Check each visible tab for close button clicks
        let mut tab_x = tab_area_start + centering_offset;

        for i in 0..visible_tabs {
            let tab_index = tab_scroll_offset + i;
            if tab_index >= tab_labels.len() || tab_index >= tab_close_buttons.len() {
                break;
            }

            // Check if this tab has a close button
            if tab_close_buttons[tab_index] {
                // Close button click detection - account for aesthetic spacing
                let min_close_area = 2;
                let preferred_close_area = if adjusted_tab_width >= 8 {
                    3
                } else {
                    min_close_area
                };
                let close_area_width = preferred_close_area.min(adjusted_tab_width.saturating_sub(2));

                if close_area_width >= 3 {
                    // " ×" pattern - check both space and × positions
                    let close_start_x = tab_x + adjusted_tab_width - close_area_width;
                    let close_end_x = tab_x + adjusted_tab_width - 1;
                    if click_x >= close_start_x && click_x <= close_end_x {
                        return Some(tab_index);
                    }
                } else {
                    // Minimal "×" at edge-1 position
                    let close_button_x = tab_x + adjusted_tab_width - 2;
                    if click_x == close_button_x {
                        return Some(tab_index);
                    }
                }
            }

            tab_x += adjusted_tab_width;

            // Skip separator
            if i < visible_tabs.saturating_sub(1) {
                tab_x += separator_width;
            }
        }

        None
    }
}