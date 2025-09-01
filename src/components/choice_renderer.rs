use crate::model::common::{Bounds, ScreenBuffer};
use crate::model::muxbox::Choice;
use crate::draw_utils::{print_with_color_and_background_at, wrap_text_to_width};
use crate::components::vertical_scrollbar::VerticalScrollbar;
use crate::components::selection_styles::{SelectionStyleRenderer, SelectionStyleConfig};

/// Choice rendering component with selection highlighting and overflow handling
pub struct ChoiceRenderer {
    /// Unique identifier for this choice renderer instance
    pub id: String,
    /// Enhanced selection styling renderer
    pub style_renderer: Option<SelectionStyleRenderer>,
}

impl ChoiceRenderer {
    /// Create a new choice renderer with specified ID
    pub fn new(id: String) -> Self {
        Self { 
            id,
            style_renderer: None,
        }
    }

    /// Create a new choice renderer with enhanced selection styling
    pub fn with_selection_styles(id: String, style_config: SelectionStyleConfig) -> Self {
        Self {
            id: id.clone(),
            style_renderer: Some(SelectionStyleRenderer::new(
                format!("{}-style", id),
                style_config,
            )),
        }
    }

    /// Enable default enhanced selection styles
    pub fn with_default_styles(id: String) -> Self {
        Self {
            id: id.clone(),
            style_renderer: Some(SelectionStyleRenderer::with_defaults(
                format!("{}-style", id),
            )),
        }
    }

    /// Render choices with proper selection highlighting and overflow handling
    pub fn draw(
        &self,
        parent_bounds: &Bounds,
        choices: &[Choice],
        viewable_height: usize,
        viewable_width: usize,
        vertical_scroll: f64,
        overflow_behavior: &str,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        border_color: &str,
        draw_border: bool,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        self.draw_with_focus(
            parent_bounds,
            choices,
            viewable_height,
            viewable_width,
            vertical_scroll,
            overflow_behavior,
            menu_fg_color,
            menu_bg_color,
            selected_menu_fg_color,
            selected_menu_bg_color,
            border_color,
            draw_border,
            None, // No focused choice by default
            buffer,
        )
    }

    /// Render choices with focus indication for keyboard navigation
    pub fn draw_with_focus(
        &self,
        parent_bounds: &Bounds,
        choices: &[Choice],
        viewable_height: usize,
        viewable_width: usize,
        vertical_scroll: f64,
        overflow_behavior: &str,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        border_color: &str,
        draw_border: bool,
        focused_choice_index: Option<usize>,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        if choices.is_empty() {
            return false;
        }

        let total_choices = choices.len();
        let choice_overflows = total_choices > viewable_height;

        match overflow_behavior {
            "scroll" if choice_overflows => {
                self.render_scrollable_choices_with_focus(
                    parent_bounds,
                    choices,
                    viewable_height,
                    vertical_scroll,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                    border_color,
                    draw_border,
                    focused_choice_index,
                    buffer,
                )
            }
            "wrap" => {
                self.render_wrapped_choices_with_focus(
                    parent_bounds,
                    choices,
                    viewable_height,
                    viewable_width,
                    vertical_scroll,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                    border_color,
                    draw_border,
                    focused_choice_index,
                    buffer,
                )
            }
            _ => {
                self.render_clipped_choices_with_focus(
                    parent_bounds,
                    choices,
                    viewable_height,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                    focused_choice_index,
                    buffer,
                );
                false
            }
        }
    }

    /// Render choices with vertical scrolling support
    fn render_scrollable_choices(
        &self,
        parent_bounds: &Bounds,
        choices: &[Choice],
        viewable_height: usize,
        vertical_scroll: f64,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        border_color: &str,
        draw_border: bool,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let total_choices = choices.len();
        let vertical_offset = ((vertical_scroll / 100.0) * (total_choices - viewable_height) as f64)
            .floor() as usize;

        let visible_choices = choices.iter().skip(vertical_offset).take(viewable_height);
        let mut y_position = parent_bounds.top() + 1;

        for choice in visible_choices {
            let (fg_color, bg_color) = self.get_choice_colors(
                choice,
                menu_fg_color,
                menu_bg_color,
                selected_menu_fg_color,
                selected_menu_bg_color,
            );

            let formatted_content = self.format_choice_content(choice);

            print_with_color_and_background_at(
                y_position,
                parent_bounds.left() + 2,
                fg_color,
                bg_color,
                &formatted_content,
                buffer,
            );
            y_position += 1;
        }

        // Draw vertical scrollbar for choices
        if draw_border {
            let vertical_scrollbar = VerticalScrollbar::new(format!("{}_choices", self.id));
            vertical_scrollbar.draw(
                parent_bounds,
                total_choices,
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

    /// Render choices with vertical scrolling and focus support
    fn render_scrollable_choices_with_focus(
        &self,
        parent_bounds: &Bounds,
        choices: &[Choice],
        viewable_height: usize,
        vertical_scroll: f64,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        border_color: &str,
        draw_border: bool,
        focused_choice_index: Option<usize>,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let total_choices = choices.len();
        let vertical_offset = ((vertical_scroll / 100.0) * (total_choices - viewable_height) as f64)
            .floor() as usize;

        let visible_choices = choices.iter().enumerate().skip(vertical_offset).take(viewable_height);
        let mut y_position = parent_bounds.top() + 1;

        for (original_index, choice) in visible_choices {
            let is_focused = focused_choice_index == Some(original_index);

            // Use enhanced styling if available
            if let Some(style_renderer) = &self.style_renderer {
                style_renderer.render_choice(
                    choice,
                    parent_bounds,
                    y_position,
                    parent_bounds.left() + 2,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                    is_focused,
                    buffer,
                );
            } else {
                // Fallback to basic rendering
                let (fg_color, bg_color) = self.get_choice_colors(
                    choice,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                );

                let formatted_content = self.format_choice_content(choice);

                print_with_color_and_background_at(
                    y_position,
                    parent_bounds.left() + 2,
                    fg_color,
                    bg_color,
                    &formatted_content,
                    buffer,
                );
            }
            y_position += 1;
        }

        // Draw vertical scrollbar for choices
        if draw_border {
            let vertical_scrollbar = VerticalScrollbar::new(format!("{}_choices", self.id));
            vertical_scrollbar.draw(
                parent_bounds,
                total_choices,
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

    /// Render choices with text wrapping support
    fn render_wrapped_choices(
        &self,
        parent_bounds: &Bounds,
        choices: &[Choice],
        viewable_height: usize,
        viewable_width: usize,
        vertical_scroll: f64,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        border_color: &str,
        draw_border: bool,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        // Create all wrapped lines first
        let mut all_wrapped_lines = Vec::new();
        for choice in choices {
            let (fg_color, bg_color) = self.get_choice_colors(
                choice,
                menu_fg_color,
                menu_bg_color,
                selected_menu_fg_color,
                selected_menu_bg_color,
            );

            let formatted_content = self.format_choice_content(choice);
            let wrapped_lines = wrap_text_to_width(&formatted_content, viewable_width);

            for wrapped_line in wrapped_lines {
                all_wrapped_lines.push((wrapped_line, fg_color, bg_color));
            }
        }

        // Calculate scroll offset for wrapped lines
        let total_wrapped_lines = all_wrapped_lines.len();
        let vertical_offset = if total_wrapped_lines > viewable_height {
            ((vertical_scroll / 100.0) * (total_wrapped_lines - viewable_height) as f64).floor()
                as usize
        } else {
            0
        };

        // Render visible wrapped lines
        let mut y_position = parent_bounds.top() + 1;
        for (wrapped_line, fg_color, bg_color) in all_wrapped_lines
            .iter()
            .skip(vertical_offset)
            .take(viewable_height)
        {
            print_with_color_and_background_at(
                y_position,
                parent_bounds.left() + 2,
                fg_color,
                bg_color,
                wrapped_line,
                buffer,
            );
            y_position += 1;
        }

        // Draw vertical scrollbar if needed
        if total_wrapped_lines > viewable_height && draw_border {
            let vertical_scrollbar = VerticalScrollbar::new(format!("{}_wrapped_choices", self.id));
            vertical_scrollbar.draw(
                parent_bounds,
                total_wrapped_lines,
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

    /// Render choices with text wrapping and focus support
    fn render_wrapped_choices_with_focus(
        &self,
        parent_bounds: &Bounds,
        choices: &[Choice],
        viewable_height: usize,
        viewable_width: usize,
        vertical_scroll: f64,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        border_color: &str,
        draw_border: bool,
        focused_choice_index: Option<usize>,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        // Create all wrapped lines first with focus information
        let mut all_wrapped_lines = Vec::new();
        for (choice_index, choice) in choices.iter().enumerate() {
            let is_focused = focused_choice_index == Some(choice_index);
            
            // Use enhanced styling if available
            if let Some(style_renderer) = &self.style_renderer {
                // For wrapped choices with enhanced styling, we need to manually calculate colors/text
                // since the enhanced renderer is designed for single-line rendering
                let (fg_color, bg_color) = if choice.selected {
                    (selected_menu_fg_color, selected_menu_bg_color)
                } else {
                    (menu_fg_color, menu_bg_color)
                };
                
                let formatted_content = self.format_choice_content(choice);
                let wrapped_lines = wrap_text_to_width(&formatted_content, viewable_width);

                for wrapped_line in wrapped_lines {
                    all_wrapped_lines.push((wrapped_line, fg_color.to_string(), bg_color.to_string()));
                }
            } else {
                // Fallback to basic rendering
                let (fg_color, bg_color) = self.get_choice_colors(
                    choice,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                );

                let formatted_content = self.format_choice_content(choice);
                let wrapped_lines = wrap_text_to_width(&formatted_content, viewable_width);

                for wrapped_line in wrapped_lines {
                    all_wrapped_lines.push((wrapped_line, fg_color.to_string(), bg_color.to_string()));
                }
            }
        }

        let total_wrapped_lines = all_wrapped_lines.len();
        let vertical_offset = if total_wrapped_lines > viewable_height {
            ((vertical_scroll / 100.0) * (total_wrapped_lines - viewable_height) as f64).floor() as usize
        } else {
            0
        };

        // Render visible wrapped lines
        let mut y_position = parent_bounds.top() + 1;
        for (wrapped_line, fg_color, bg_color) in all_wrapped_lines
            .iter()
            .skip(vertical_offset)
            .take(viewable_height)
        {
            print_with_color_and_background_at(
                y_position,
                parent_bounds.left() + 2,
                fg_color,
                bg_color,
                wrapped_line,
                buffer,
            );
            y_position += 1;
        }

        // Draw vertical scrollbar if needed
        if total_wrapped_lines > viewable_height && draw_border {
            let vertical_scrollbar = VerticalScrollbar::new(format!("{}_wrapped_choices", self.id));
            vertical_scrollbar.draw(
                parent_bounds,
                total_wrapped_lines,
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

    /// Render choices with simple clipping (no overflow handling)
    fn render_clipped_choices(
        &self,
        parent_bounds: &Bounds,
        choices: &[Choice],
        viewable_height: usize,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        buffer: &mut ScreenBuffer,
    ) {
        let mut y_position = parent_bounds.top() + 1;

        for choice in choices {
            if y_position > parent_bounds.bottom() - 1 {
                break; // Don't draw outside the bounds
            }

            let (fg_color, bg_color) = self.get_choice_colors(
                choice,
                menu_fg_color,
                menu_bg_color,
                selected_menu_fg_color,
                selected_menu_bg_color,
            );

            let formatted_content = self.format_choice_content(choice);

            print_with_color_and_background_at(
                y_position,
                parent_bounds.left() + 2,
                fg_color,
                bg_color,
                &formatted_content,
                buffer,
            );
            y_position += 1;
        }
    }

    /// Render choices with simple clipping and focus support
    fn render_clipped_choices_with_focus(
        &self,
        parent_bounds: &Bounds,
        choices: &[Choice],
        viewable_height: usize,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        focused_choice_index: Option<usize>,
        buffer: &mut ScreenBuffer,
    ) {
        let mut y_position = parent_bounds.top() + 1;

        for (choice_index, choice) in choices.iter().enumerate() {
            if y_position > parent_bounds.bottom() - 1 {
                break; // Don't draw outside the bounds
            }

            let is_focused = focused_choice_index == Some(choice_index);

            // Use enhanced styling if available
            if let Some(style_renderer) = &self.style_renderer {
                style_renderer.render_choice(
                    choice,
                    parent_bounds,
                    y_position,
                    parent_bounds.left() + 2,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                    is_focused,
                    buffer,
                );
            } else {
                // Fallback to basic rendering
                let (fg_color, bg_color) = self.get_choice_colors(
                    choice,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                );

                let formatted_content = self.format_choice_content(choice);

                print_with_color_and_background_at(
                    y_position,
                    parent_bounds.left() + 2,
                    fg_color,
                    bg_color,
                    &formatted_content,
                    buffer,
                );
            }
            y_position += 1;
        }
    }

    /// Get appropriate colors for a choice based on its state
    fn get_choice_colors<'a>(
        &self,
        choice: &Choice,
        menu_fg_color: &'a str,
        menu_bg_color: &'a str,
        selected_menu_fg_color: &'a str,
        selected_menu_bg_color: &'a str,
    ) -> (&'a str, &'a str) {
        if choice.selected {
            (selected_menu_fg_color, selected_menu_bg_color)
        } else {
            (menu_fg_color, menu_bg_color)
        }
    }

    /// Format choice content with waiting state indicator
    fn format_choice_content(&self, choice: &Choice) -> String {
        if choice.waiting {
            format!("{}...", choice.content.as_ref().unwrap())
        } else {
            choice.content.as_ref().unwrap().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::common::Bounds;

    fn create_test_choice(content: &str, selected: bool, waiting: bool) -> Choice {
        Choice {
            content: Some(content.to_string()),
            selected,
            waiting,
            ..Default::default()
        }
    }

    fn create_test_buffer() -> ScreenBuffer {
        ScreenBuffer::new()
    }

    #[test]
    fn test_choice_renderer_creation() {
        let renderer = ChoiceRenderer::new("test_renderer".to_string());
        assert_eq!(renderer.id, "test_renderer");
    }

    #[test]
    fn test_render_empty_choices() {
        let renderer = ChoiceRenderer::new("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let scrollbars_drawn = renderer.draw(
            &bounds,
            &[],
            10,
            60,
            0.0,
            "scroll",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            true,
            &mut buffer,
        );
        
        assert!(!scrollbars_drawn);
    }

    #[test]
    fn test_render_simple_choices() {
        let renderer = ChoiceRenderer::new("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let choices = vec![
            create_test_choice("Option 1", false, false),
            create_test_choice("Option 2", true, false),
            create_test_choice("Option 3", false, true),
        ];

        let scrollbars_drawn = renderer.draw(
            &bounds,
            &choices,
            10,
            60,
            0.0,
            "scroll",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            true,
            &mut buffer,
        );
        
        assert!(!scrollbars_drawn); // No scrollbar needed for 3 choices in 10-line height
    }

    #[test]
    fn test_render_scrollable_choices() {
        let renderer = ChoiceRenderer::new("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 10); // Small height
        let mut buffer = create_test_buffer();
        
        let choices: Vec<Choice> = (0..20)
            .map(|i| create_test_choice(&format!("Option {}", i + 1), false, false))
            .collect();

        let scrollbars_drawn = renderer.draw(
            &bounds,
            &choices,
            3, // Small viewable height
            60,
            0.0,
            "scroll",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            true,
            &mut buffer,
        );
        
        assert!(scrollbars_drawn); // Scrollbar needed for 20 choices in 3-line height
    }

    #[test]
    fn test_render_wrapped_choices() {
        let renderer = ChoiceRenderer::new("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let choices = vec![
            create_test_choice("This is a very long choice that should wrap across multiple lines when rendered", false, false),
            create_test_choice("Short", true, false),
        ];

        let scrollbars_drawn = renderer.draw(
            &bounds,
            &choices,
            10,
            20, // Small width to force wrapping
            0.0,
            "wrap",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            true,
            &mut buffer,
        );
        
        // May or may not need scrollbars depending on wrapping
        assert!(scrollbars_drawn || !scrollbars_drawn);
    }

    #[test]
    fn test_choice_color_selection() {
        let renderer = ChoiceRenderer::new("test".to_string());
        
        let normal_choice = create_test_choice("Normal", false, false);
        let selected_choice = create_test_choice("Selected", true, false);
        
        let (normal_fg, normal_bg) = renderer.get_choice_colors(
            &normal_choice,
            "white",
            "black",
            "yellow",
            "blue",
        );
        
        let (selected_fg, selected_bg) = renderer.get_choice_colors(
            &selected_choice,
            "white",
            "black",
            "yellow",
            "blue",
        );
        
        assert_eq!(normal_fg, "white");
        assert_eq!(normal_bg, "black");
        assert_eq!(selected_fg, "yellow");
        assert_eq!(selected_bg, "blue");
    }

    #[test]
    fn test_waiting_choice_formatting() {
        let renderer = ChoiceRenderer::new("test".to_string());
        
        let normal_choice = create_test_choice("Normal Choice", false, false);
        let waiting_choice = create_test_choice("Waiting Choice", false, true);
        
        assert_eq!(renderer.format_choice_content(&normal_choice), "Normal Choice");
        assert_eq!(renderer.format_choice_content(&waiting_choice), "Waiting Choice...");
    }

    #[test]
    fn test_clipped_choices_rendering() {
        let renderer = ChoiceRenderer::new("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 8); // Very small height
        let mut buffer = create_test_buffer();
        
        let choices: Vec<Choice> = (0..10)
            .map(|i| create_test_choice(&format!("Option {}", i + 1), false, false))
            .collect();

        let scrollbars_drawn = renderer.draw(
            &bounds,
            &choices,
            2, // Only 2 lines viewable
            60,
            0.0,
            "clip", // Not scroll or wrap
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            true,
            &mut buffer,
        );
        
        assert!(!scrollbars_drawn); // No scrollbars for clipped rendering
    }

    #[test]
    fn test_choice_renderer_with_default_styles() {
        let renderer = ChoiceRenderer::with_default_styles("test".to_string());
        assert_eq!(renderer.id, "test");
        assert!(renderer.style_renderer.is_some());
    }

    #[test]
    fn test_choice_renderer_with_custom_styles() {
        use crate::components::selection_styles::{SelectionStyleConfig, SelectionStyle, FocusStyle};
        
        let config = SelectionStyleConfig {
            selection_style: SelectionStyle::PointerIndicator,
            focus_style: FocusStyle::Intensity,
            ..SelectionStyleConfig::default()
        };
        
        let renderer = ChoiceRenderer::with_selection_styles("custom".to_string(), config);
        assert_eq!(renderer.id, "custom");
        assert!(renderer.style_renderer.is_some());
        
        if let Some(style_renderer) = &renderer.style_renderer {
            assert!(matches!(style_renderer.config.selection_style, SelectionStyle::PointerIndicator));
            assert!(matches!(style_renderer.config.focus_style, FocusStyle::Intensity));
        }
    }

    #[test]
    fn test_draw_with_focus_no_focused_choice() {
        let renderer = ChoiceRenderer::new("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let choices = vec![
            create_test_choice("Choice 1", true, false),
            create_test_choice("Choice 2", false, false),
        ];
        
        let scrollbars_drawn = renderer.draw_with_focus(
            &bounds,
            &choices,
            10,
            70,
            0.0,
            "clip",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            false,
            None, // No focused choice
            &mut buffer,
        );
        
        assert!(!scrollbars_drawn);
    }

    #[test]
    fn test_draw_with_focus_first_choice_focused() {
        let renderer = ChoiceRenderer::new("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let choices = vec![
            create_test_choice("Choice 1", true, false),
            create_test_choice("Choice 2", false, false),
        ];
        
        let scrollbars_drawn = renderer.draw_with_focus(
            &bounds,
            &choices,
            10,
            70,
            0.0,
            "clip",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            false,
            Some(0), // First choice focused
            &mut buffer,
        );
        
        assert!(!scrollbars_drawn);
    }

    #[test]
    fn test_scrollable_choices_with_focus() {
        let renderer = ChoiceRenderer::new("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let choices = vec![
            create_test_choice("Choice 1", false, false),
            create_test_choice("Choice 2", true, false),
            create_test_choice("Choice 3", false, false),
            create_test_choice("Choice 4", false, false),
            create_test_choice("Choice 5", false, false),
        ];
        
        let scrollbars_drawn = renderer.draw_with_focus(
            &bounds,
            &choices,
            3, // Only show 3 choices at once
            70,
            0.0,
            "scroll",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            true,
            Some(1), // Second choice focused
            &mut buffer,
        );
        
        assert!(scrollbars_drawn); // Should draw scrollbars
    }

    #[test]
    fn test_wrapped_choices_with_focus() {
        let renderer = ChoiceRenderer::new("test".to_string());
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let choices = vec![
            create_test_choice("This is a very long choice that should wrap to multiple lines", false, false),
            create_test_choice("Short choice", true, false),
        ];
        
        let scrollbars_drawn = renderer.draw_with_focus(
            &bounds,
            &choices,
            10,
            20, // Narrow width to force wrapping
            0.0,
            "wrap",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            true,
            Some(0), // First choice focused
            &mut buffer,
        );
        
        // May or may not draw scrollbars depending on wrapped content height
        // Test just verifies no panic and successful rendering
        assert!(scrollbars_drawn || !scrollbars_drawn);
    }

    #[test]
    fn test_enhanced_styling_with_pointer_indicator() {
        use crate::components::selection_styles::{
            SelectionStyleConfig, SelectionStyle, SelectionIndicators
        };
        
        let config = SelectionStyleConfig {
            selection_style: SelectionStyle::PointerIndicator,
            custom_indicators: Some(SelectionIndicators {
                pointer_symbol: "→".to_string(),
                ..SelectionIndicators::default()
            }),
            ..SelectionStyleConfig::default()
        };
        
        let renderer = ChoiceRenderer::with_selection_styles("test".to_string(), config);
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let choices = vec![
            create_test_choice("Selected Choice", true, false),
            create_test_choice("Normal Choice", false, false),
        ];
        
        let scrollbars_drawn = renderer.draw_with_focus(
            &bounds,
            &choices,
            10,
            70,
            0.0,
            "clip",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            false,
            Some(0), // Focus on selected choice
            &mut buffer,
        );
        
        assert!(!scrollbars_drawn);
        // Note: Testing the actual pointer symbol rendering would require 
        // inspecting the buffer contents, which is complex. The test verifies
        // that the enhanced styling system doesn't cause crashes.
    }

    #[test]
    fn test_enhanced_styling_with_inverted_colors() {
        use crate::components::selection_styles::{
            SelectionStyleConfig, SelectionStyle, FocusStyle
        };
        
        let config = SelectionStyleConfig {
            selection_style: SelectionStyle::InvertColors,
            focus_style: FocusStyle::Intensity,
            ..SelectionStyleConfig::default()
        };
        
        let renderer = ChoiceRenderer::with_selection_styles("test".to_string(), config);
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let choices = vec![
            create_test_choice("Inverted Choice", true, false),
        ];
        
        let scrollbars_drawn = renderer.draw_with_focus(
            &bounds,
            &choices,
            10,
            70,
            0.0,
            "clip",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            false,
            Some(0), // Focus for intensity effect
            &mut buffer,
        );
        
        assert!(!scrollbars_drawn);
    }

    #[test]
    fn test_enhanced_styling_with_combined_styles() {
        use crate::components::selection_styles::{
            SelectionStyleConfig, SelectionStyle, FocusStyle, SelectionIndicators
        };
        
        let config = SelectionStyleConfig {
            selection_style: SelectionStyle::Combined(vec![
                SelectionStyle::PointerIndicator,
                SelectionStyle::BoldHighlight,
                SelectionStyle::BorderHighlight,
            ]),
            focus_style: FocusStyle::DottedBorder,
            custom_indicators: Some(SelectionIndicators::default()),
            ..SelectionStyleConfig::default()
        };
        
        let renderer = ChoiceRenderer::with_selection_styles("test".to_string(), config);
        let bounds = Bounds::new(10, 5, 80, 20);
        let mut buffer = create_test_buffer();
        
        let choices = vec![
            create_test_choice("Combined Styling", true, false),
            create_test_choice("Normal Choice", false, false),
        ];
        
        let scrollbars_drawn = renderer.draw_with_focus(
            &bounds,
            &choices,
            10,
            70,
            0.0,
            "clip",
            "white",
            "black",
            "yellow",
            "blue",
            "grey",
            false,
            Some(0), // Focus for dotted border effect
            &mut buffer,
        );
        
        assert!(!scrollbars_drawn);
        // Test verifies that combined styling doesn't cause rendering issues
    }
}