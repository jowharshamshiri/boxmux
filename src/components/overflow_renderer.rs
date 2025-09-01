use crate::model::common::{Bounds, ScreenBuffer, Cell};
use crate::model::muxbox::Choice;
use crate::draw_utils::{
    get_bg_color, get_fg_color, fill_muxbox, wrap_text_to_width, wrap_choices_to_width,
    render_wrapped_content, render_wrapped_choices, print_with_color_and_background_at
};
use crate::components::vertical_scrollbar::VerticalScrollbar;
use crate::components::horizontal_scrollbar::HorizontalScrollbar;

/// Overflow behavior types for content rendering
#[derive(Debug, Clone, PartialEq)]
pub enum OverflowBehavior {
    /// Standard scrolling with scrollbars
    Scroll,
    /// Text wrapping with line breaks
    Wrap,
    /// Fill entire box with solid pattern
    Fill(char),
    /// Cross out content with X pattern
    CrossOut,
    /// Remove/hide content completely
    Removed,
    /// Clip content without scrollbars (default)
    Clip,
}

/// Configuration for overflow rendering behavior
#[derive(Debug, Clone)]
pub struct OverflowConfig {
    /// Primary overflow behavior
    pub behavior: OverflowBehavior,
    /// Whether to draw borders around overflow content
    pub draw_border: bool,
    /// Custom fill character for Fill behavior
    pub fill_char: char,
    /// Cross-out pattern character
    pub cross_char: char,
}

impl Default for OverflowConfig {
    fn default() -> Self {
        Self {
            behavior: OverflowBehavior::Clip,
            draw_border: true,
            fill_char: '█',
            cross_char: 'X',
        }
    }
}

impl OverflowConfig {
    /// Create config for scroll overflow behavior
    pub fn scroll() -> Self {
        Self {
            behavior: OverflowBehavior::Scroll,
            ..Default::default()
        }
    }

    /// Create config for wrap overflow behavior  
    pub fn wrap() -> Self {
        Self {
            behavior: OverflowBehavior::Wrap,
            ..Default::default()
        }
    }

    /// Create config for fill overflow behavior with custom character
    pub fn fill(fill_char: char) -> Self {
        Self {
            behavior: OverflowBehavior::Fill(fill_char),
            fill_char,
            ..Default::default()
        }
    }

    /// Create config for cross-out overflow behavior
    pub fn cross_out() -> Self {
        Self {
            behavior: OverflowBehavior::CrossOut,
            ..Default::default()
        }
    }

    /// Create config for removed overflow behavior
    pub fn removed() -> Self {
        Self {
            behavior: OverflowBehavior::Removed,
            ..Default::default()
        }
    }
}

/// Specialized component for overflow behavior visualization and handling
pub struct OverflowRenderer {
    /// Unique identifier for this overflow renderer instance
    pub id: String,
    /// Configuration for overflow behavior
    pub config: OverflowConfig,
}

impl OverflowRenderer {
    /// Create a new overflow renderer with specified ID and config
    pub fn new(id: String, config: OverflowConfig) -> Self {
        Self { id, config }
    }

    /// Create overflow renderer with default configuration
    pub fn with_defaults(id: String) -> Self {
        Self::new(id, OverflowConfig::default())
    }

    /// Create overflow renderer for scroll behavior
    pub fn with_scroll(id: String) -> Self {
        Self::new(id, OverflowConfig::scroll())
    }

    /// Create overflow renderer for wrap behavior
    pub fn with_wrap(id: String) -> Self {
        Self::new(id, OverflowConfig::wrap())
    }

    /// Determine if content overflows the available bounds
    pub fn content_overflows(
        &self,
        content_width: usize,
        content_height: usize,
        viewable_width: usize,
        viewable_height: usize,
    ) -> bool {
        content_width > viewable_width || content_height > viewable_height
    }

    /// Render overflow behavior for text content
    pub fn render_text_overflow(
        &self,
        content: &str,
        bounds: &Bounds,
        vertical_scroll: f64,
        horizontal_scroll: f64,
        fg_color: &str,
        bg_color: &str,
        border_color: &str,
        parent_bg_color: &str,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let content_lines: Vec<&str> = content.lines().collect();
        let content_height = content_lines.len();
        let content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        let overflows = self.content_overflows(content_width, content_height, viewable_width, viewable_height);

        if !overflows {
            return false;
        }

        match &self.config.behavior {
            OverflowBehavior::Fill(fill_char) => {
                fill_muxbox(bounds, true, bg_color, *fill_char, buffer);
                true
            }
            OverflowBehavior::CrossOut => {
                self.render_cross_out_pattern(bounds, border_color, parent_bg_color, buffer);
                true
            }
            OverflowBehavior::Removed => {
                fill_muxbox(bounds, false, parent_bg_color, ' ', buffer);
                true
            }
            OverflowBehavior::Scroll => {
                self.render_scrollable_text_content(
                    content,
                    bounds,
                    vertical_scroll,
                    horizontal_scroll,
                    fg_color,
                    bg_color,
                    border_color,
                    buffer,
                )
            }
            OverflowBehavior::Wrap => {
                self.render_wrapped_text_content(
                    content,
                    bounds,
                    vertical_scroll,
                    fg_color,
                    bg_color,
                    border_color,
                    buffer,
                )
            }
            OverflowBehavior::Clip => false, // No special handling for clipping
        }
    }

    /// Render overflow behavior for choice content
    pub fn render_choice_overflow(
        &self,
        choices: &[Choice],
        bounds: &Bounds,
        vertical_scroll: f64,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        border_color: &str,
        parent_bg_color: &str,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        // Calculate choice content dimensions
        let choice_count = choices.len();
        let max_choice_width = choices
            .iter()
            .map(|choice| {
                // Use same logic as ChoiceRenderer for consistent width calculation
                match &choice.content {
                    Some(content) => {
                        if choice.waiting {
                            content.len() + 3 // Add "..." for waiting state
                        } else {
                            content.len()
                        }
                    }
                    None => 0,
                }
            })
            .max()
            .unwrap_or(0);

        let overflows = self.content_overflows(max_choice_width, choice_count, viewable_width, viewable_height);

        if !overflows {
            return false;
        }

        match &self.config.behavior {
            OverflowBehavior::Fill(fill_char) => {
                fill_muxbox(bounds, true, menu_bg_color, *fill_char, buffer);
                true
            }
            OverflowBehavior::CrossOut => {
                self.render_cross_out_pattern(bounds, border_color, parent_bg_color, buffer);
                true
            }
            OverflowBehavior::Removed => {
                fill_muxbox(bounds, false, parent_bg_color, ' ', buffer);
                true
            }
            OverflowBehavior::Wrap => {
                self.render_wrapped_choice_content(
                    choices,
                    bounds,
                    vertical_scroll,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                    border_color,
                    buffer,
                )
            }
            OverflowBehavior::Scroll | OverflowBehavior::Clip => false, // Handled elsewhere
        }
    }

    /// Render cross-out pattern for disabled/removed content
    fn render_cross_out_pattern(
        &self,
        bounds: &Bounds,
        border_color: &str,
        parent_bg_color: &str,
        buffer: &mut ScreenBuffer,
    ) {
        let border_color_code = get_fg_color(border_color);
        let parent_bg_color_code = get_bg_color(parent_bg_color);

        // Draw diagonal cross pattern
        let width = bounds.width();
        let height = bounds.height();

        for i in 0..width.min(height) {
            // Draw main diagonal (top-left to bottom-right)
            if i < height {
                let cell = Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: parent_bg_color_code.clone(),
                    ch: self.config.cross_char,
                };
                buffer.update(bounds.left() + i, bounds.top() + i, cell);
            }

            // Draw anti-diagonal (top-right to bottom-left) 
            if i < height && (width.saturating_sub(1).saturating_sub(i)) < width {
                let cell = Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: parent_bg_color_code.clone(),
                    ch: self.config.cross_char,
                };
                buffer.update(
                    bounds.left() + width.saturating_sub(1).saturating_sub(i),
                    bounds.top() + i,
                    cell,
                );
            }
        }
    }

    /// Render scrollable text content with scrollbars
    fn render_scrollable_text_content(
        &self,
        content: &str,
        bounds: &Bounds,
        vertical_scroll: f64,
        horizontal_scroll: f64,
        fg_color: &str,
        bg_color: &str,
        border_color: &str,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let content_lines: Vec<&str> = content.lines().collect();
        let content_height = content_lines.len();
        let max_content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        // Calculate offsets
        let y_offset = if content_height > viewable_height {
            ((content_height - viewable_height) as f64 * vertical_scroll / 100.0).round() as usize
        } else {
            0
        };

        let x_offset = if max_content_width > viewable_width {
            ((max_content_width - viewable_width + 3) as f64 * horizontal_scroll / 100.0).round() as usize
        } else {
            0
        };

        // Render visible content lines
        for (display_y, &line) in content_lines
            .iter()
            .skip(y_offset)
            .take(viewable_height)
            .enumerate()
        {
            let visible_line = if line.len() > x_offset {
                &line[x_offset..]
            } else {
                ""
            };

            print_with_color_and_background_at(
                bounds.top() + 1 + display_y,
                bounds.left() + 2,
                fg_color,
                bg_color,
                visible_line,
                buffer,
            );
        }

        // Draw scrollbars if needed and borders are enabled
        let mut scrollbars_drawn = false;
        if self.config.draw_border {
            if content_height > viewable_height {
                let vertical_scrollbar = VerticalScrollbar::new(format!("{}_text_vertical", self.id));
                vertical_scrollbar.draw(
                    bounds,
                    content_height,
                    viewable_height,
                    vertical_scroll,
                    border_color,
                    bg_color,
                    buffer,
                );
                scrollbars_drawn = true;
            }

            if max_content_width > viewable_width {
                let horizontal_scrollbar = HorizontalScrollbar::new(format!("{}_text_horizontal", self.id));
                horizontal_scrollbar.draw(
                    bounds,
                    max_content_width,
                    viewable_width,
                    horizontal_scroll,
                    border_color,
                    bg_color,
                    buffer,
                );
                scrollbars_drawn = true;
            }
        }

        scrollbars_drawn
    }

    /// Render wrapped text content with optional scrollbar
    fn render_wrapped_text_content(
        &self,
        content: &str,
        bounds: &Bounds,
        vertical_scroll: f64,
        fg_color: &str,
        bg_color: &str,
        border_color: &str,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let viewable_width = bounds.width().saturating_sub(4);
        let wrapped_content = wrap_text_to_width(content, viewable_width);

        let viewable_height = bounds.height().saturating_sub(4);
        let wrapped_overflows_vertically = wrapped_content.len() > viewable_height;

        render_wrapped_content(
            &wrapped_content,
            bounds,
            vertical_scroll,
            fg_color,
            bg_color,
            buffer,
        );

        // Draw vertical scrollbar if wrapped content overflows
        if wrapped_overflows_vertically && self.config.draw_border {
            let vertical_scrollbar = VerticalScrollbar::new(format!("{}_wrapped_text", self.id));
            vertical_scrollbar.draw(
                bounds,
                wrapped_content.len(),
                viewable_height,
                vertical_scroll,
                border_color,
                bg_color,
                buffer,
            );
            true
        } else {
            false
        }
    }

    /// Render wrapped choice content with optional scrollbar
    fn render_wrapped_choice_content(
        &self,
        choices: &[Choice],
        bounds: &Bounds,
        vertical_scroll: f64,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        border_color: &str,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let viewable_width = bounds.width().saturating_sub(4);
        let wrapped_choices = wrap_choices_to_width(choices, viewable_width);

        let viewable_height = bounds.height().saturating_sub(4);
        let wrapped_overflows_vertically = wrapped_choices.len() > viewable_height;

        render_wrapped_choices(
            choices,
            bounds,
            vertical_scroll,
            menu_fg_color,
            menu_bg_color,
            selected_menu_fg_color,
            selected_menu_bg_color,
            buffer,
        );

        // Draw vertical scrollbar if wrapped choices overflow
        if wrapped_overflows_vertically && self.config.draw_border {
            let vertical_scrollbar = VerticalScrollbar::new(format!("{}_wrapped_choices", self.id));
            vertical_scrollbar.draw(
                bounds,
                wrapped_choices.len(),
                viewable_height,
                vertical_scroll,
                border_color,
                menu_bg_color,
                buffer,
            );
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::common::{Bounds, ScreenBuffer, Cell};
    use crate::model::muxbox::Choice;

    fn create_test_buffer() -> ScreenBuffer {
        ScreenBuffer::new()
    }

    fn create_test_choice(content: &str, selected: bool) -> Choice {
        Choice {
            content: Some(content.to_string()),
            selected,
            waiting: false,
            ..Default::default()
        }
    }

    #[test]
    fn test_overflow_renderer_creation() {
        let renderer = OverflowRenderer::with_defaults("test".to_string());
        assert_eq!(renderer.id, "test");
        assert!(matches!(renderer.config.behavior, OverflowBehavior::Clip));
    }

    #[test]
    fn test_overflow_renderer_with_scroll() {
        let renderer = OverflowRenderer::with_scroll("test".to_string());
        assert!(matches!(renderer.config.behavior, OverflowBehavior::Scroll));
    }

    #[test]
    fn test_overflow_renderer_with_wrap() {
        let renderer = OverflowRenderer::with_wrap("test".to_string());
        assert!(matches!(renderer.config.behavior, OverflowBehavior::Wrap));
    }

    #[test]
    fn test_overflow_config_creation() {
        let config = OverflowConfig::fill('*');
        assert!(matches!(config.behavior, OverflowBehavior::Fill('*')));
        assert_eq!(config.fill_char, '*');

        let config = OverflowConfig::cross_out();
        assert!(matches!(config.behavior, OverflowBehavior::CrossOut));

        let config = OverflowConfig::removed();
        assert!(matches!(config.behavior, OverflowBehavior::Removed));
    }

    #[test]
    fn test_content_overflows_detection() {
        let renderer = OverflowRenderer::with_defaults("test".to_string());
        
        // Content fits
        assert!(!renderer.content_overflows(10, 5, 20, 10));
        
        // Content overflows width
        assert!(renderer.content_overflows(25, 5, 20, 10));
        
        // Content overflows height
        assert!(renderer.content_overflows(10, 15, 20, 10));
        
        // Content overflows both
        assert!(renderer.content_overflows(25, 15, 20, 10));
    }

    #[test]
    fn test_render_text_overflow_no_overflow() {
        let renderer = OverflowRenderer::with_defaults("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();

        let result = renderer.render_text_overflow(
            "Short content",
            &bounds,
            0.0,
            0.0,
            "white",
            "black",
            "grey",
            "black",
            &mut buffer,
        );

        assert!(!result); // No overflow, no special handling
    }

    #[test]
    fn test_render_text_overflow_fill() {
        let config = OverflowConfig::fill('█');
        let renderer = OverflowRenderer::new("test".to_string(), config);
        let bounds = Bounds::new(10, 5, 20, 10);
        let mut buffer = create_test_buffer();

        let content = "This is a very long line that will definitely overflow the small bounds we have set for this test case";

        let result = renderer.render_text_overflow(
            content,
            &bounds,
            0.0,
            0.0,
            "white",
            "black",
            "grey",
            "black",
            &mut buffer,
        );

        assert!(result); // Fill overflow was applied
    }

    #[test]
    fn test_render_text_overflow_cross_out() {
        let config = OverflowConfig::cross_out();
        let renderer = OverflowRenderer::new("test".to_string(), config);
        let bounds = Bounds::new(10, 5, 20, 10);
        let mut buffer = create_test_buffer();

        let content = "This is a very long line that will definitely overflow the small bounds we have set for this test case";

        let result = renderer.render_text_overflow(
            content,
            &bounds,
            0.0,
            0.0,
            "white",
            "black",
            "grey",
            "black",
            &mut buffer,
        );

        assert!(result); // Cross-out overflow was applied
    }

    #[test]
    fn test_render_text_overflow_removed() {
        let config = OverflowConfig::removed();
        let renderer = OverflowRenderer::new("test".to_string(), config);
        let bounds = Bounds::new(10, 5, 20, 10);
        let mut buffer = create_test_buffer();

        let content = "This is a very long line that will definitely overflow the small bounds we have set for this test case";

        let result = renderer.render_text_overflow(
            content,
            &bounds,
            0.0,
            0.0,
            "white",
            "black",
            "grey",
            "black",
            &mut buffer,
        );

        assert!(result); // Removed overflow was applied
    }

    #[test]
    fn test_render_choice_overflow_no_overflow() {
        let renderer = OverflowRenderer::with_defaults("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();

        let choices = vec![
            create_test_choice("Choice 1", false),
            create_test_choice("Choice 2", true),
        ];

        let result = renderer.render_choice_overflow(
            &choices,
            &bounds,
            0.0,
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            "black",
            &mut buffer,
        );

        assert!(!result); // No overflow, no special handling
    }

    #[test]
    fn test_render_choice_overflow_fill() {
        let config = OverflowConfig::fill('▓');
        let renderer = OverflowRenderer::new("test".to_string(), config);
        let bounds = Bounds::new(10, 5, 15, 8);
        let mut buffer = create_test_buffer();

        let choices = vec![
            create_test_choice("This is a very long choice that will overflow", false),
            create_test_choice("Another long choice", true),
            create_test_choice("Yet another choice", false),
            create_test_choice("More choices", false),
            create_test_choice("Even more choices that exceed bounds", false),
            create_test_choice("Final long choice", false),
        ];

        let result = renderer.render_choice_overflow(
            &choices,
            &bounds,
            0.0,
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            "black",
            &mut buffer,
        );

        assert!(result); // Fill overflow was applied
    }

    #[test]
    fn test_cross_out_pattern_rendering() {
        let config = OverflowConfig::cross_out();
        let renderer = OverflowRenderer::new("test".to_string(), config);
        let bounds = Bounds::new(10, 5, 20, 10);
        let mut buffer = create_test_buffer();

        renderer.render_cross_out_pattern(&bounds, "red", "black", &mut buffer);

        // Test verifies no panic and cross-out pattern is applied
        // Detailed buffer inspection would require more complex test setup
    }
}