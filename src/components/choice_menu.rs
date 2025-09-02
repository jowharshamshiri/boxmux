use crate::model::muxbox::Choice;
use crate::components::renderable_content::{RenderableContent, ClickableZone, ContentType, ClickableMetadata};
use crate::{ScreenBuffer, Bounds};

/// ChoiceMenu component - generates choice content and clickable zones for BoxRenderer
/// 
/// This component handles choice logic (selection, waiting states, content generation)
/// and outputs content strings and clickable zones that BoxRenderer treats like any other content.
pub struct ChoiceMenu<'a> {
    /// Reference to the choices data
    choices: &'a [Choice],
    /// Selected choice index (if any)
    selected_index: Option<usize>,
    /// Focused choice index for keyboard navigation
    focused_index: Option<usize>,
    /// Component identifier
    id: String,
}

impl<'a> ChoiceMenu<'a> {
    /// Create a new choice menu component
    pub fn new(id: String, choices: &'a [Choice]) -> Self {
        Self {
            choices,
            selected_index: None,
            focused_index: None,
            id,
        }
    }

    /// Set the selected choice index
    pub fn with_selection(mut self, selected_index: Option<usize>) -> Self {
        self.selected_index = selected_index;
        self
    }

    /// Set the focused choice index for keyboard navigation
    pub fn with_focus(mut self, focused_index: Option<usize>) -> Self {
        self.focused_index = focused_index;
        self
    }

    /// Generate formatted choice content as a single string
    fn generate_choice_content(&self) -> String {
        let mut content_lines = Vec::new();
        
        for (index, choice) in self.choices.iter().enumerate() {
            let mut line = String::new();
            
            // Add selection indicator
            if self.selected_index == Some(index) {
                line.push_str("► ");
            } else if self.focused_index == Some(index) {
                line.push_str("• ");
            } else {
                line.push_str("  ");
            }
            
            // Add choice content
            if let Some(content) = &choice.content {
                line.push_str(content);
                
                // Add waiting indicator
                if choice.waiting {
                    line.push_str("...");
                }
            }
            
            content_lines.push(line);
        }
        
        content_lines.join("\n")
    }

    /// Get content as raw string for BoxRenderer to handle like text content
    pub fn get_raw_content(&self) -> String {
        self.generate_choice_content()
    }
}

impl<'a> RenderableContent for ChoiceMenu<'a> {
    /// Get the raw content dimensions before any transformations
    fn get_dimensions(&self) -> (usize, usize) {
        let content = self.generate_choice_content();
        let lines: Vec<&str> = content.lines().collect();
        let height = lines.len();
        let width = lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);
        (width, height)
    }

    /// Render content within a viewport with scroll offsets
    fn render_viewport(&self, bounds: &Bounds, x_offset: usize, y_offset: usize, buffer: &mut ScreenBuffer) {
        let content = self.generate_choice_content();
        let lines: Vec<&str> = content.lines().collect();
        
        for (display_y, &line) in lines
            .iter()
            .skip(y_offset)
            .take(bounds.height().saturating_sub(2))
            .enumerate()
        {
            let visible_line = if line.len() > x_offset {
                &line[x_offset..]
            } else {
                ""
            };

            crate::draw_utils::print_with_color_and_background_at(
                bounds.top() + 1 + display_y,
                bounds.left() + 2,
                &None, // Use default colors for now
                &None,
                visible_line,
                buffer,
            );
        }
    }

    /// Get clickable zones for this content accounting for scroll offsets
    fn get_clickable_zones(&self, bounds: &Bounds, x_offset: usize, y_offset: usize) -> Vec<ClickableZone> {
        let mut zones = Vec::new();
        let choices = self.choices;
        let viewport_height = bounds.height().saturating_sub(2);
        let viewport_width = bounds.width().saturating_sub(4);

        for (index, choice) in choices.iter().enumerate() {
            if index < y_offset || index >= y_offset + viewport_height {
                continue; // Skip choices outside viewport
            }

            if let Some(content) = &choice.content {
                let display_row = index - y_offset;
                let choice_width = content.len().min(viewport_width);

                let zone_bounds = Bounds::new(
                    bounds.left() + 2, // Start at column 2 (inside border)
                    bounds.top() + display_row + 1, // Start at row (inside border)
                    choice_width,
                    1, // Single line height
                );

                let metadata = ClickableMetadata {
                    display_text: Some(content.clone()),
                    tooltip: Some(format!("Choice {}: {}", index, content)),
                    selected: choice.selected,
                    enabled: !choice.waiting,
                    original_line: Some(index),
                    char_range: Some((x_offset, x_offset + choice_width)),
                };

                zones.push(ClickableZone::with_metadata(
                    zone_bounds,
                    format!("choice_{}", index),
                    ContentType::Choice,
                    metadata,
                ));
            }
        }

        zones
    }

    /// Get dimensions after applying wrapping transformation
    fn get_wrapped_dimensions(&self, max_width: usize) -> (usize, usize) {
        let content = self.generate_choice_content();
        let wrapped_content = crate::draw_utils::wrap_text_to_width(&content, max_width);
        (max_width, wrapped_content.len())
    }

    /// Render wrapped content within viewport
    fn render_wrapped_viewport(&self, bounds: &Bounds, max_width: usize, y_offset: usize, buffer: &mut ScreenBuffer) {
        let content = self.generate_choice_content();
        let wrapped_content = crate::draw_utils::wrap_text_to_width(&content, max_width);
        
        crate::draw_utils::render_wrapped_content(
            &wrapped_content,
            bounds,
            y_offset as f64,
            &None, // Use default colors for now
            &None,
            buffer,
        );
    }

    /// Get clickable zones for wrapped content
    fn get_wrapped_clickable_zones(&self, bounds: &Bounds, max_width: usize, y_offset: usize) -> Vec<ClickableZone> {
        // For wrapped choices, we need to track which wrapped lines correspond to which original choices
        let mut zones = Vec::new();
        let choices = self.choices;
        let viewport_height = bounds.height().saturating_sub(2);
        let mut current_line = 0;

        for (choice_index, choice) in choices.iter().enumerate() {
            if let Some(content) = &choice.content {
                let wrapped_lines = crate::draw_utils::wrap_text_to_width(content, max_width);
                
                for (wrapped_line_index, wrapped_line) in wrapped_lines.iter().enumerate() {
                    if current_line < y_offset || current_line >= y_offset + viewport_height {
                        current_line += 1;
                        continue;
                    }

                    let display_row = current_line - y_offset;
                    let zone_bounds = Bounds::new(
                        bounds.left() + 2,
                        bounds.top() + display_row + 1,
                        wrapped_line.len(),
                        1,
                    );

                    let metadata = ClickableMetadata {
                        display_text: Some(wrapped_line.clone()),
                        tooltip: Some(format!("Choice {}: {}", choice_index, content)),
                        selected: choice.selected,
                        enabled: !choice.waiting,
                        original_line: Some(choice_index),
                        char_range: Some((wrapped_line_index * max_width, (wrapped_line_index + 1) * max_width)),
                    };

                    zones.push(ClickableZone::with_metadata(
                        zone_bounds,
                        format!("choice_{}_{}", choice_index, wrapped_line_index),
                        ContentType::Choice,
                        metadata,
                    ));

                    current_line += 1;
                }
            }
        }

        zones
    }
}