use crate::model::common::{Bounds, ScreenBuffer};
use crate::model::muxbox::Choice;
use crate::draw_utils::print_with_color_and_background_at;

/// Selection highlighting styles for choice menus
#[derive(Debug, Clone, PartialEq)]
pub enum SelectionStyle {
    /// Standard color-based highlighting (current default behavior)
    ColorHighlight,
    /// Inverted colors for selection
    InvertColors,
    /// Border/frame around selected item
    BorderHighlight,
    /// Arrow or pointer indicator
    PointerIndicator,
    /// Underline the selected text
    UnderlineHighlight,
    /// Bold text for selected item
    BoldHighlight,
    /// Combination of multiple styles
    Combined(Vec<SelectionStyle>),
}

/// Focus indication styles for keyboard navigation
#[derive(Debug, Clone, PartialEq)]
pub enum FocusStyle {
    /// No special focus indication beyond selection
    None,
    /// Blinking or animated focus indicator
    Blinking,
    /// Extra bright/intense colors when focused
    Intensity,
    /// Dotted border around focused area
    DottedBorder,
    /// Animated arrow or cursor
    AnimatedCursor,
}

/// Visual feedback styles for user interactions
#[derive(Debug, Clone, PartialEq)]
pub enum FeedbackStyle {
    /// Brief color flash on selection change
    ColorFlash,
    /// Smooth color transition animation
    ColorTransition,
    /// Visual ripple effect from interaction point
    RippleEffect,
    /// Slide/movement animation
    SlideAnimation,
    /// No visual feedback
    None,
}

/// Configuration for choice selection styling
#[derive(Debug, Clone)]
pub struct SelectionStyleConfig {
    pub selection_style: SelectionStyle,
    pub focus_style: FocusStyle,
    pub feedback_style: FeedbackStyle,
    pub animation_duration_ms: u32,
    pub intensity_factor: f32,
    pub custom_indicators: Option<SelectionIndicators>,
}

/// Custom selection indicators and symbols
#[derive(Debug, Clone)]
pub struct SelectionIndicators {
    pub pointer_symbol: String,
    pub selection_prefix: String,
    pub selection_suffix: String,
    pub focus_indicator: String,
    pub border_chars: BorderChars,
}

/// Custom border characters for selection highlighting
#[derive(Debug, Clone)]
pub struct BorderChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

impl Default for SelectionStyleConfig {
    fn default() -> Self {
        Self {
            selection_style: SelectionStyle::ColorHighlight,
            focus_style: FocusStyle::None,
            feedback_style: FeedbackStyle::None,
            animation_duration_ms: 200,
            intensity_factor: 1.5,
            custom_indicators: None,
        }
    }
}

impl Default for SelectionIndicators {
    fn default() -> Self {
        Self {
            pointer_symbol: "▶".to_string(),
            selection_prefix: "".to_string(),
            selection_suffix: "".to_string(),
            focus_indicator: "◀".to_string(),
            border_chars: BorderChars::default(),
        }
    }
}

impl Default for BorderChars {
    fn default() -> Self {
        Self {
            top_left: '┌',
            top_right: '┐',
            bottom_left: '└',
            bottom_right: '┘',
            horizontal: '─',
            vertical: '│',
        }
    }
}

/// Enhanced selection styling component
pub struct SelectionStyleRenderer {
    pub id: String,
    pub config: SelectionStyleConfig,
}

impl SelectionStyleRenderer {
    /// Create new selection style renderer with configuration
    pub fn new(id: String, config: SelectionStyleConfig) -> Self {
        Self { id, config }
    }

    /// Create with default configuration
    pub fn with_defaults(id: String) -> Self {
        Self {
            id,
            config: SelectionStyleConfig::default(),
        }
    }

    /// Render choice with enhanced selection styling
    pub fn render_choice(
        &self,
        choice: &Choice,
        bounds: &Bounds,
        y_position: usize,
        x_position: usize,
        base_fg_color: &str,
        base_bg_color: &str,
        selected_fg_color: &str,
        selected_bg_color: &str,
        is_focused: bool,
        buffer: &mut ScreenBuffer,
    ) {
        let (final_fg_color, final_bg_color, display_text) = self.calculate_style_colors_and_text(
            choice,
            base_fg_color,
            base_bg_color,
            selected_fg_color,
            selected_bg_color,
            is_focused,
        );

        // Render the main choice content
        print_with_color_and_background_at(
            y_position,
            x_position,
            &final_fg_color,
            &final_bg_color,
            &display_text,
            buffer,
        );

        // Apply additional styling based on selection style
        if choice.selected {
            self.apply_selection_style(choice, bounds, y_position, x_position, &display_text, buffer);
        }

        // Apply focus indicators if focused
        if is_focused {
            self.apply_focus_style(bounds, y_position, x_position, &display_text, buffer);
        }
    }

    /// Calculate final colors and text based on style configuration
    pub fn calculate_style_colors_and_text(
        &self,
        choice: &Choice,
        base_fg_color: &str,
        base_bg_color: &str,
        selected_fg_color: &str,
        selected_bg_color: &str,
        is_focused: bool,
    ) -> (String, String, String) {
        let mut fg_color = if choice.selected {
            selected_fg_color.to_string()
        } else {
            base_fg_color.to_string()
        };

        let mut bg_color = if choice.selected {
            selected_bg_color.to_string()
        } else {
            base_bg_color.to_string()
        };

        let mut display_text = choice.content.as_ref().unwrap_or(&String::new()).clone();

        // Apply selection style modifications
        if choice.selected {
            match &self.config.selection_style {
                SelectionStyle::InvertColors => {
                    // Swap foreground and background colors
                    std::mem::swap(&mut fg_color, &mut bg_color);
                }
                SelectionStyle::BoldHighlight => {
                    // Use bright color variants for bold effect
                    fg_color = self.make_color_bright(&fg_color);
                }
                SelectionStyle::PointerIndicator => {
                    // Add pointer symbol prefix
                    let default_indicators = SelectionIndicators::default();
                    let indicators = self.config.custom_indicators.as_ref().unwrap_or(&default_indicators);
                    display_text = format!("{} {}", indicators.pointer_symbol, display_text);
                }
                SelectionStyle::Combined(styles) => {
                    // Apply multiple styles
                    for style in styles {
                        match style {
                            SelectionStyle::InvertColors => {
                                std::mem::swap(&mut fg_color, &mut bg_color);
                            }
                            SelectionStyle::BoldHighlight => {
                                fg_color = self.make_color_bright(&fg_color);
                            }
                            SelectionStyle::PointerIndicator => {
                                let default_indicators = SelectionIndicators::default();
                                let indicators = self.config.custom_indicators.as_ref().unwrap_or(&default_indicators);
                                display_text = format!("{} {}", indicators.pointer_symbol, display_text);
                            }
                            _ => {} // Other styles handled separately
                        }
                    }
                }
                _ => {} // ColorHighlight and other styles use base colors
            }
        }

        // Apply focus intensity if focused
        if is_focused && matches!(self.config.focus_style, FocusStyle::Intensity) {
            fg_color = self.apply_intensity(&fg_color);
        }

        // Add waiting state indicator if needed
        if choice.waiting {
            display_text = format!("{}...", display_text);
        }

        (fg_color, bg_color, display_text)
    }

    /// Apply additional selection styling (borders, underlines, etc.)
    fn apply_selection_style(
        &self,
        _choice: &Choice,
        bounds: &Bounds,
        y_position: usize,
        x_position: usize,
        display_text: &str,
        buffer: &mut ScreenBuffer,
    ) {
        match &self.config.selection_style {
            SelectionStyle::BorderHighlight => {
                self.draw_selection_border(bounds, y_position, x_position, display_text.len(), buffer);
            }
            SelectionStyle::UnderlineHighlight => {
                self.draw_selection_underline(bounds, y_position, x_position, display_text.len(), buffer);
            }
            SelectionStyle::Combined(styles) => {
                if styles.contains(&SelectionStyle::BorderHighlight) {
                    self.draw_selection_border(bounds, y_position, x_position, display_text.len(), buffer);
                }
                if styles.contains(&SelectionStyle::UnderlineHighlight) {
                    self.draw_selection_underline(bounds, y_position, x_position, display_text.len(), buffer);
                }
            }
            _ => {}
        }
    }

    /// Apply focus styling indicators
    fn apply_focus_style(
        &self,
        bounds: &Bounds,
        y_position: usize,
        x_position: usize,
        display_text: &str,
        buffer: &mut ScreenBuffer,
    ) {
        match &self.config.focus_style {
            FocusStyle::DottedBorder => {
                self.draw_dotted_focus_border(bounds, y_position, x_position, display_text.len(), buffer);
            }
            FocusStyle::AnimatedCursor => {
                self.draw_animated_cursor(bounds, y_position, x_position, buffer);
            }
            FocusStyle::Blinking => {
                // Blinking effect would need frame timing - simplified here
                self.draw_focus_indicator(bounds, y_position, x_position, buffer);
            }
            _ => {}
        }
    }

    /// Draw border around selected choice
    fn draw_selection_border(
        &self,
        bounds: &Bounds,
        y_position: usize,
        x_position: usize,
        text_length: usize,
        buffer: &mut ScreenBuffer,
    ) {
        let default_indicators = SelectionIndicators::default();
        let indicators = self.config.custom_indicators.as_ref().unwrap_or(&default_indicators);
        let border = &indicators.border_chars;

        // Only draw border if we have space and are within bounds
        if y_position > bounds.top() && y_position < bounds.bottom() && x_position > bounds.left() {
            // Draw corners and sides
            if x_position > 0 {
                print_with_color_and_background_at(
                    y_position,
                    x_position - 1,
                    "bright_yellow",
                    "black",
                    &border.vertical.to_string(),
                    buffer,
                );
            }
            
            if x_position + text_length + 1 <= bounds.right() {
                print_with_color_and_background_at(
                    y_position,
                    x_position + text_length,
                    "bright_yellow",
                    "black",
                    &border.vertical.to_string(),
                    buffer,
                );
            }
        }
    }

    /// Draw underline under selected choice
    fn draw_selection_underline(
        &self,
        bounds: &Bounds,
        y_position: usize,
        x_position: usize,
        text_length: usize,
        buffer: &mut ScreenBuffer,
    ) {
        if y_position + 1 <= bounds.bottom() {
            let underline = "─".repeat(text_length);
            print_with_color_and_background_at(
                y_position + 1,
                x_position,
                "bright_yellow",
                "black",
                &underline,
                buffer,
            );
        }
    }

    /// Draw dotted border for focus indication
    fn draw_dotted_focus_border(
        &self,
        bounds: &Bounds,
        y_position: usize,
        x_position: usize,
        text_length: usize,
        buffer: &mut ScreenBuffer,
    ) {
        if y_position >= bounds.top() && y_position <= bounds.bottom() && x_position >= bounds.left() {
            // Draw dotted indicators
            if x_position > 0 {
                print_with_color_and_background_at(
                    y_position,
                    x_position - 1,
                    "bright_cyan",
                    "black",
                    ":",
                    buffer,
                );
            }
            
            if x_position + text_length + 1 <= bounds.right() {
                print_with_color_and_background_at(
                    y_position,
                    x_position + text_length,
                    "bright_cyan",
                    "black",
                    ":",
                    buffer,
                );
            }
        }
    }

    /// Draw animated cursor for focus
    fn draw_animated_cursor(
        &self,
        bounds: &Bounds,
        y_position: usize,
        x_position: usize,
        buffer: &mut ScreenBuffer,
    ) {
        if y_position >= bounds.top() && y_position <= bounds.bottom() && x_position > bounds.left() {
            let default_indicators = SelectionIndicators::default();
            let indicators = self.config.custom_indicators.as_ref().unwrap_or(&default_indicators);
            print_with_color_and_background_at(
                y_position,
                x_position - 1,
                "bright_magenta",
                "black",
                &indicators.focus_indicator,
                buffer,
            );
        }
    }

    /// Draw simple focus indicator
    fn draw_focus_indicator(
        &self,
        bounds: &Bounds,
        y_position: usize,
        x_position: usize,
        buffer: &mut ScreenBuffer,
    ) {
        if y_position >= bounds.top() && y_position <= bounds.bottom() && x_position > bounds.left() {
            print_with_color_and_background_at(
                y_position,
                x_position - 1,
                "bright_white",
                "black",
                "●",
                buffer,
            );
        }
    }

    /// Make color brighter/more intense
    fn make_color_bright(&self, color: &str) -> String {
        match color {
            "red" => "bright_red".to_string(),
            "green" => "bright_green".to_string(),
            "blue" => "bright_blue".to_string(),
            "yellow" => "bright_yellow".to_string(),
            "magenta" => "bright_magenta".to_string(),
            "cyan" => "bright_cyan".to_string(),
            "white" => "bright_white".to_string(),
            "black" => "bright_black".to_string(),
            _ => color.to_string(), // Return as-is if already bright or unknown
        }
    }

    /// Apply intensity factor to color
    fn apply_intensity(&self, color: &str) -> String {
        // For now, just make it bright - could be enhanced with RGB manipulation
        self.make_color_bright(color)
    }

    /// Get default selection style for migration from existing choice renderer
    pub fn get_legacy_style_config() -> SelectionStyleConfig {
        SelectionStyleConfig {
            selection_style: SelectionStyle::ColorHighlight,
            focus_style: FocusStyle::None,
            feedback_style: FeedbackStyle::None,
            animation_duration_ms: 0,
            intensity_factor: 1.0,
            custom_indicators: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::common::{Bounds, ScreenBuffer};

    #[test]
    fn test_selection_style_renderer_creation() {
        let renderer = SelectionStyleRenderer::with_defaults("test".to_string());
        assert_eq!(renderer.id, "test");
        assert_eq!(renderer.config.selection_style, SelectionStyle::ColorHighlight);
    }

    #[test]
    fn test_custom_selection_style_config() {
        let config = SelectionStyleConfig {
            selection_style: SelectionStyle::PointerIndicator,
            focus_style: FocusStyle::Intensity,
            feedback_style: FeedbackStyle::ColorFlash,
            animation_duration_ms: 300,
            intensity_factor: 2.0,
            custom_indicators: Some(SelectionIndicators::default()),
        };
        
        let renderer = SelectionStyleRenderer::new("custom".to_string(), config);
        assert!(matches!(renderer.config.selection_style, SelectionStyle::PointerIndicator));
        assert!(matches!(renderer.config.focus_style, FocusStyle::Intensity));
    }

    #[test]
    fn test_color_bright_conversion() {
        let renderer = SelectionStyleRenderer::with_defaults("test".to_string());
        assert_eq!(renderer.make_color_bright("red"), "bright_red");
        assert_eq!(renderer.make_color_bright("green"), "bright_green");
        assert_eq!(renderer.make_color_bright("bright_blue"), "bright_blue"); // Already bright
    }

    #[test]
    fn test_selection_indicators_default() {
        let indicators = SelectionIndicators::default();
        assert_eq!(indicators.pointer_symbol, "▶");
        assert_eq!(indicators.focus_indicator, "◀");
        assert_eq!(indicators.border_chars.top_left, '┌');
    }

    #[test]
    fn test_combined_selection_styles() {
        let combined = SelectionStyle::Combined(vec![
            SelectionStyle::PointerIndicator,
            SelectionStyle::BoldHighlight,
            SelectionStyle::BorderHighlight,
        ]);
        
        if let SelectionStyle::Combined(styles) = combined {
            assert_eq!(styles.len(), 3);
            assert!(styles.contains(&SelectionStyle::PointerIndicator));
            assert!(styles.contains(&SelectionStyle::BoldHighlight));
            assert!(styles.contains(&SelectionStyle::BorderHighlight));
        } else {
            panic!("Expected Combined selection style");
        }
    }

    #[test]
    fn test_calculate_style_colors_and_text_basic() {
        let renderer = SelectionStyleRenderer::with_defaults("test".to_string());
        let mut choice = Choice {
            content: Some("Test Choice".to_string()),
            selected: true,
            waiting: false,
            script: None,
            libs: None,
            redirect_output: None,
            append_output: None,
        };

        let (fg, bg, text) = renderer.calculate_style_colors_and_text(
            &choice,
            "white",
            "black",
            "bright_white",
            "blue",
            false,
        );

        assert_eq!(fg, "bright_white");
        assert_eq!(bg, "blue");
        assert_eq!(text, "Test Choice");
    }

    #[test]
    fn test_calculate_style_colors_and_text_with_pointer() {
        let config = SelectionStyleConfig {
            selection_style: SelectionStyle::PointerIndicator,
            focus_style: FocusStyle::None,
            feedback_style: FeedbackStyle::None,
            animation_duration_ms: 0,
            intensity_factor: 1.0,
            custom_indicators: Some(SelectionIndicators::default()),
        };
        
        let renderer = SelectionStyleRenderer::new("test".to_string(), config);
        let choice = Choice {
            content: Some("Menu Item".to_string()),
            selected: true,
            waiting: false,
            script: None,
            libs: None,
            redirect_output: None,
            append_output: None,
        };

        let (fg, bg, text) = renderer.calculate_style_colors_and_text(
            &choice,
            "white",
            "black",
            "bright_white",
            "blue",
            false,
        );

        assert_eq!(fg, "bright_white");
        assert_eq!(bg, "blue");
        assert_eq!(text, "▶ Menu Item");
    }

    #[test]
    fn test_inverted_colors() {
        let config = SelectionStyleConfig {
            selection_style: SelectionStyle::InvertColors,
            ..SelectionStyleConfig::default()
        };
        
        let renderer = SelectionStyleRenderer::new("test".to_string(), config);
        let choice = Choice {
            content: Some("Inverted".to_string()),
            selected: true,
            waiting: false,
            script: None,
            libs: None,
            redirect_output: None,
            append_output: None,
        };

        let (fg, bg, text) = renderer.calculate_style_colors_and_text(
            &choice,
            "white",
            "black",
            "bright_white",
            "blue",
            false,
        );

        // Colors should be swapped
        assert_eq!(fg, "blue");
        assert_eq!(bg, "bright_white");
        assert_eq!(text, "Inverted");
    }

    #[test]
    fn test_waiting_state_indicator() {
        let renderer = SelectionStyleRenderer::with_defaults("test".to_string());
        let choice = Choice {
            content: Some("Loading".to_string()),
            selected: false,
            waiting: true,
            script: None,
            libs: None,
            redirect_output: None,
            append_output: None,
        };

        let (fg, bg, text) = renderer.calculate_style_colors_and_text(
            &choice,
            "white",
            "black",
            "bright_white",
            "blue",
            false,
        );

        assert_eq!(text, "Loading...");
    }

    #[test]
    fn test_focus_intensity() {
        let config = SelectionStyleConfig {
            selection_style: SelectionStyle::ColorHighlight,
            focus_style: FocusStyle::Intensity,
            ..SelectionStyleConfig::default()
        };
        
        let renderer = SelectionStyleRenderer::new("test".to_string(), config);
        let choice = Choice {
            content: Some("Focused".to_string()),
            selected: true,
            waiting: false,
            script: None,
            libs: None,
            redirect_output: None,
            append_output: None,
        };

        let (fg, bg, text) = renderer.calculate_style_colors_and_text(
            &choice,
            "white",
            "black",
            "red",
            "blue",
            true, // is_focused = true
        );

        // Should apply intensity to red -> bright_red
        assert_eq!(fg, "bright_red");
        assert_eq!(bg, "blue");
    }
}