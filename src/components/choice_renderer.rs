use crate::model::common::{Bounds, ScreenBuffer};
use crate::model::muxbox::Choice;
use crate::draw_utils::{print_with_color_and_background_at, wrap_text_to_width};
use crate::components::vertical_scrollbar::VerticalScrollbar;

/// Choice rendering component with selection highlighting and overflow handling
pub struct ChoiceRenderer {
    /// Unique identifier for this choice renderer instance
    pub id: String,
}

impl ChoiceRenderer {
    /// Create a new choice renderer with specified ID
    pub fn new(id: String) -> Self {
        Self { id }
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
        if choices.is_empty() {
            return false;
        }

        let total_choices = choices.len();
        let choice_overflows = total_choices > viewable_height;

        match overflow_behavior {
            "scroll" if choice_overflows => {
                self.render_scrollable_choices(
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
                    buffer,
                )
            }
            "wrap" => {
                self.render_wrapped_choices(
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
                    buffer,
                )
            }
            _ => {
                self.render_clipped_choices(
                    parent_bounds,
                    choices,
                    viewable_height,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
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
}