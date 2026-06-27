use crate::draw_utils::{
    draw_horizontal_line, fill_horizontal_background, print_with_color_and_background_at,
};
use crate::model::common::ScreenBuffer;

/// Tab bar component for displaying stream tabs with scrolling support
pub struct TabBar;

/// Navigation action for tab scrolling
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TabNavigationAction {
    ScrollLeft,
    ScrollRight,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TabHoverTarget {
    Tab(usize),
    CloseButton(usize),
    NavigationLeft,
    NavigationRight,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TabHitTarget {
    Tab(usize),
    CloseButton(usize),
    Navigation(TabNavigationAction),
}

#[derive(Debug, Clone)]
struct TabLayout {
    left_arrow_x: usize,
    right_arrow_x: usize,
    tab_area_start: usize,
    tab_area_end: usize,
    trailing_border_x: usize,
    needs_scrolling: bool,
    max_scroll_offset: usize,
    centering_offset: usize,
    total_tabs_width: usize,
    tabs: Vec<TabRect>,
}

#[derive(Debug, Clone)]
struct TabRect {
    index: usize,
    x: usize,
    width: usize,
    close_glyph_x: Option<usize>,
}

impl TabBar {
    fn layout(
        x1: usize,
        x2: usize,
        tab_labels: &[String],
        tab_close_buttons: &[bool],
        tab_scroll_offset: usize,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
    ) -> TabLayout {
        let total_width = x2.saturating_sub(x1).saturating_add(1);
        let left_arrow_space = 2;
        let right_arrow_space = 2;
        let has_border_color = crate::color_utils::should_draw_color(fg_color)
            || crate::color_utils::should_draw_color(bg_color);
        let border_space = if has_border_color { 4 } else { 0 };
        let reserved_space = left_arrow_space + right_arrow_space + border_space;
        let tab_area_width = total_width.saturating_sub(reserved_space);
        let left_arrow_x = x1 + if has_border_color { 2 } else { 0 };
        let tab_area_start = left_arrow_x + left_arrow_space;
        let tab_area_end = tab_area_start + tab_area_width;
        let right_arrow_x = tab_area_end;
        let trailing_border_x = right_arrow_x + right_arrow_space;

        let max_tab_width = 16;
        let min_tab_width = 6;
        let ideal_tab_width = if !tab_labels.is_empty() {
            tab_area_width / tab_labels.len()
        } else {
            max_tab_width
        };
        let tab_width = ideal_tab_width.clamp(min_tab_width, max_tab_width);
        let natural_tabs_width = tab_labels.len() * tab_width;
        let needs_scrolling = (natural_tabs_width as f32) > (tab_area_width as f32 * 0.85);

        let separator_width = 1;
        let (visible_tabs, adjusted_tab_width) = if needs_scrolling {
            let min_tab_with_separator = min_tab_width + separator_width;
            let max_visible_tabs = tab_area_width / min_tab_with_separator.max(1);
            let remaining_tabs = tab_labels.len().saturating_sub(tab_scroll_offset);
            let visible_tabs = max_visible_tabs.min(remaining_tabs).max(1);
            let adjusted_tab_width = if visible_tabs > 0 {
                let total_separator_space = (visible_tabs.saturating_sub(1)) * separator_width;
                (tab_area_width.saturating_sub(total_separator_space) / visible_tabs)
                    .max(min_tab_width)
            } else {
                min_tab_width
            };
            (visible_tabs, adjusted_tab_width)
        } else {
            let visible_tabs = tab_labels.len();
            let adjusted_tab_width = if visible_tabs > 0 {
                let total_separator_space = (visible_tabs.saturating_sub(1)) * separator_width;
                tab_area_width.saturating_sub(total_separator_space) / visible_tabs
            } else {
                tab_width
            };
            (visible_tabs, adjusted_tab_width)
        };

        let total_tabs_width = if visible_tabs > 0 {
            visible_tabs * adjusted_tab_width + (visible_tabs.saturating_sub(1)) * separator_width
        } else {
            0
        };
        let centering_offset = if needs_scrolling {
            0
        } else {
            tab_area_width.saturating_sub(total_tabs_width) / 2
        };
        let start_offset = if needs_scrolling {
            tab_scroll_offset
        } else {
            0
        };

        let mut tabs = Vec::with_capacity(visible_tabs);
        let mut tab_x = tab_area_start + centering_offset;
        for i in 0..visible_tabs {
            let tab_index = start_offset + i;
            if tab_index >= tab_labels.len() {
                break;
            }

            let close_glyph_x = if tab_close_buttons.get(tab_index).copied().unwrap_or(false) {
                Self::close_glyph_x(tab_x, adjusted_tab_width)
            } else {
                None
            };

            tabs.push(TabRect {
                index: tab_index,
                x: tab_x,
                width: adjusted_tab_width,
                close_glyph_x,
            });

            tab_x += adjusted_tab_width;
            if i < visible_tabs.saturating_sub(1) {
                tab_x += separator_width;
            }
        }

        let max_scroll_offset = if !tab_labels.is_empty() {
            tab_labels
                .len()
                .saturating_sub(tab_area_width / tab_width.max(1))
        } else {
            0
        };

        TabLayout {
            left_arrow_x,
            right_arrow_x,
            tab_area_start,
            tab_area_end,
            trailing_border_x,
            needs_scrolling,
            max_scroll_offset,
            centering_offset,
            total_tabs_width,
            tabs,
        }
    }

    fn close_glyph_x(tab_x: usize, tab_width: usize) -> Option<usize> {
        if tab_width < 2 {
            return None;
        }

        let min_close_area = 2;
        let preferred_close_area = if tab_width >= 8 { 3 } else { min_close_area };
        let close_area_width = preferred_close_area.min(tab_width.saturating_sub(2));

        Some(if close_area_width >= 3 {
            tab_x + tab_width - close_area_width + 1
        } else {
            tab_x + tab_width - 2
        })
    }

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
        hovered_target: Option<&TabHoverTarget>,
        buffer: &mut ScreenBuffer,
    ) {
        let layout = Self::layout(
            x1,
            x2,
            tab_labels,
            tab_close_buttons,
            tab_scroll_offset,
            fg_color,
            bg_color,
        );
        let left_arrow_space = 2;
        let right_arrow_space = 2;

        // Draw leading border if border colors are not transparent
        if (crate::color_utils::should_draw_color(fg_color)
            || crate::color_utils::should_draw_color(bg_color))
            && x1 < x2
        {
            draw_horizontal_line(y, x1, x1 + 1, fg_color, bg_color, buffer);
        }

        // Draw left scroll arrow or border fill in reserved space
        if layout.needs_scrolling && tab_scroll_offset > 0 {
            // Show left arrow when can scroll left
            let (arrow_fg, arrow_bg) = Self::hover_colors(
                title_fg_color,
                bg_color,
                matches!(hovered_target, Some(TabHoverTarget::NavigationLeft)),
            );
            print_with_color_and_background_at(
                y,
                layout.left_arrow_x,
                &arrow_fg,
                &arrow_bg,
                "◀",
                buffer,
            );
            print_with_color_and_background_at(
                y,
                layout.left_arrow_x + 1,
                &arrow_fg,
                &arrow_bg,
                " ",
                buffer,
            );
        } else {
            // Fill reserved space with border when arrow not needed
            draw_horizontal_line(
                y,
                layout.left_arrow_x,
                layout.left_arrow_x + left_arrow_space - 1,
                fg_color,
                bg_color,
                buffer,
            );
        }

        // Draw tabs with proper spacing and centering
        if !layout.tabs.is_empty() {
            for (visible_index, tab) in layout.tabs.iter().enumerate() {
                Self::draw_single_tab(
                    y,
                    tab.x,
                    tab.width,
                    &tab_labels[tab.index],
                    tab.index == active_tab_index,
                    tab.close_glyph_x.is_some(),
                    matches!(hovered_target, Some(TabHoverTarget::Tab(index)) if *index == tab.index),
                    matches!(hovered_target, Some(TabHoverTarget::CloseButton(index)) if *index == tab.index),
                    title_fg_color,
                    title_bg_color,
                    buffer,
                );

                // Draw separator after tab (except for last tab)
                if visible_index < layout.tabs.len().saturating_sub(1) {
                    print_with_color_and_background_at(
                        y,
                        tab.x + tab.width,
                        fg_color,
                        bg_color,
                        "│",
                        buffer,
                    );
                }
            }
        }

        // Fill remaining tab area with border (before and after tab group)
        if layout.tab_area_start < layout.tab_area_start + layout.centering_offset {
            draw_horizontal_line(
                y,
                layout.tab_area_start,
                layout.tab_area_start + layout.centering_offset - 1,
                fg_color,
                bg_color,
                buffer,
            );
        }

        let tabs_end = layout.tab_area_start + layout.centering_offset + layout.total_tabs_width;
        if tabs_end < layout.tab_area_end {
            draw_horizontal_line(
                y,
                tabs_end,
                layout.tab_area_end - 1,
                fg_color,
                bg_color,
                buffer,
            );
        }

        if layout.needs_scrolling && tab_scroll_offset < layout.max_scroll_offset {
            // Show right arrow when can scroll right
            let (arrow_fg, arrow_bg) = Self::hover_colors(
                title_fg_color,
                bg_color,
                matches!(hovered_target, Some(TabHoverTarget::NavigationRight)),
            );
            print_with_color_and_background_at(
                y,
                layout.right_arrow_x,
                &arrow_fg,
                &arrow_bg,
                " ",
                buffer,
            );
            print_with_color_and_background_at(
                y,
                layout.right_arrow_x + 1,
                &arrow_fg,
                &arrow_bg,
                "▶",
                buffer,
            );
        } else {
            // Fill reserved space with border when arrow not needed
            draw_horizontal_line(
                y,
                layout.right_arrow_x,
                layout.right_arrow_x + right_arrow_space - 1,
                fg_color,
                bg_color,
                buffer,
            );
        }

        // Draw trailing border if border colors are not transparent
        if (crate::color_utils::should_draw_color(fg_color)
            || crate::color_utils::should_draw_color(bg_color))
            && layout.trailing_border_x < x2
        {
            draw_horizontal_line(y, layout.trailing_border_x, x2, fg_color, bg_color, buffer);
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
        is_hovered: bool,
        is_close_hovered: bool,
        title_fg_color: &Option<String>,
        title_bg_color: &Option<String>,
        buffer: &mut ScreenBuffer,
    ) {
        let (base_tab_fg, base_tab_bg) = if is_active {
            (title_bg_color, title_fg_color) // Inverted colors for active tab
        } else {
            (title_fg_color, title_bg_color)
        };
        let (tab_fg, tab_bg) = Self::hover_colors(base_tab_fg, base_tab_bg, is_hovered);

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
        fill_horizontal_background(y, x, x + width - 1, &tab_fg, &tab_bg, buffer);

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
            print_with_color_and_background_at(y, x, &tab_fg, &tab_bg, &label_content, buffer);

            // Draw close button with aesthetic spacing
            let (close_fg, close_bg) = Self::hover_colors(&tab_fg, &tab_bg, is_close_hovered);
            if close_area_width >= 3 {
                // Enough room for " ×" pattern
                print_with_color_and_background_at(
                    y,
                    close_button_x,
                    &close_fg,
                    &close_bg,
                    if is_close_hovered { " X" } else { " ×" },
                    buffer,
                );
            } else {
                // Minimal space - just "×" at edge-1 position
                print_with_color_and_background_at(
                    y,
                    x + width - 2,
                    &close_fg,
                    &close_bg,
                    if is_close_hovered { "X" } else { "×" },
                    buffer,
                );
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
            print_with_color_and_background_at(y, x, &tab_fg, &tab_bg, &display_text, buffer);
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
        match Self::calculate_tab_hit_target(
            click_x,
            x1,
            x2,
            tab_labels,
            &[],
            tab_scroll_offset,
            fg_color,
            bg_color,
        ) {
            Some(TabHitTarget::Tab(index)) => Some(index),
            _ => None,
        }
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
        match Self::calculate_tab_hit_target(
            click_x,
            x1,
            x2,
            tab_labels,
            &[],
            tab_scroll_offset,
            fg_color,
            bg_color,
        ) {
            Some(TabHitTarget::Navigation(action)) => Some(action),
            _ => None,
        }
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
        match Self::calculate_tab_hit_target(
            click_x,
            x1,
            x2,
            tab_labels,
            tab_close_buttons,
            tab_scroll_offset,
            fg_color,
            bg_color,
        ) {
            Some(TabHitTarget::CloseButton(index)) => Some(index),
            _ => None,
        }
    }

    pub fn calculate_tab_hit_target(
        x: usize,
        x1: usize,
        x2: usize,
        tab_labels: &[String],
        tab_close_buttons: &[bool],
        tab_scroll_offset: usize,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
    ) -> Option<TabHitTarget> {
        if tab_labels.is_empty() {
            return None;
        }

        let layout = Self::layout(
            x1,
            x2,
            tab_labels,
            tab_close_buttons,
            tab_scroll_offset,
            fg_color,
            bg_color,
        );

        if layout.needs_scrolling {
            if tab_scroll_offset > 0 && x == layout.left_arrow_x {
                return Some(TabHitTarget::Navigation(TabNavigationAction::ScrollLeft));
            }
            if tab_scroll_offset < layout.max_scroll_offset && x == layout.right_arrow_x + 1 {
                return Some(TabHitTarget::Navigation(TabNavigationAction::ScrollRight));
            }
        }

        if x < layout.tab_area_start || x >= layout.tab_area_end {
            return None;
        }

        for tab in &layout.tabs {
            if tab.close_glyph_x == Some(x) {
                return Some(TabHitTarget::CloseButton(tab.index));
            }
            if x >= tab.x && x < tab.x + tab.width {
                return Some(TabHitTarget::Tab(tab.index));
            }
        }

        None
    }

    pub fn calculate_tab_hover_target(
        hover_x: usize,
        x1: usize,
        x2: usize,
        tab_labels: &[String],
        tab_close_buttons: &[bool],
        tab_scroll_offset: usize,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
    ) -> Option<TabHoverTarget> {
        Self::calculate_tab_hit_target(
            hover_x,
            x1,
            x2,
            tab_labels,
            tab_close_buttons,
            tab_scroll_offset,
            fg_color,
            bg_color,
        )
        .map(|target| match target {
            TabHitTarget::Tab(index) => TabHoverTarget::Tab(index),
            TabHitTarget::CloseButton(index) => TabHoverTarget::CloseButton(index),
            TabHitTarget::Navigation(TabNavigationAction::ScrollLeft) => {
                TabHoverTarget::NavigationLeft
            }
            TabHitTarget::Navigation(TabNavigationAction::ScrollRight) => {
                TabHoverTarget::NavigationRight
            }
        })
    }

    fn hover_colors(
        fg_color: &Option<String>,
        bg_color: &Option<String>,
        is_hovered: bool,
    ) -> (Option<String>, Option<String>) {
        if is_hovered {
            (
                Some(crate::color_utils::default_hover_fg_color().to_string()),
                Some(crate::color_utils::default_hover_bg_color().to_string()),
            )
        } else {
            (fg_color.clone(), bg_color.clone())
        }
    }
}
