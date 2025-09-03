use crate::components::renderable_content::{
    RenderableContent, ClickableZone, ContentType, ClickableMetadata, 
    ContentEvent, EventResult, EventType, HoverState
};
use crate::{ScreenBuffer, Bounds};
use crate::draw_utils::{print_with_color_and_background_at, wrap_text_to_width};
use crate::model::muxbox::Choice;

/// ChoiceContent implementation of RenderableContent trait
/// FIXES the broken choice rendering that renders outside bounds and doesn't trigger horizontal scrollbars
/// PRESERVES the working wrap mode functionality
pub struct ChoiceContent<'a> {
    /// The choices to render
    choices: &'a [Choice],
    /// Menu foreground color
    menu_fg_color: &'a Option<String>,
    /// Menu background color
    menu_bg_color: &'a Option<String>,
    /// Selected choice foreground color
    selected_menu_fg_color: &'a Option<String>,
    /// Selected choice background color
    selected_menu_bg_color: &'a Option<String>,
}

impl<'a> ChoiceContent<'a> {
    /// Create new ChoiceContent
    pub fn new(
        choices: &'a [Choice],
        menu_fg_color: &'a Option<String>,
        menu_bg_color: &'a Option<String>,
        selected_menu_fg_color: &'a Option<String>,
        selected_menu_bg_color: &'a Option<String>,
    ) -> Self {
        Self {
            choices,
            menu_fg_color,
            menu_bg_color,
            selected_menu_fg_color,
            selected_menu_bg_color,
        }
    }

    /// Format choice content (from choice_renderer.rs)
    fn format_choice_content(&self, choice: &Choice) -> String {
        if let Some(content) = &choice.content {
            if choice.waiting {
                format!("{}...", content)
            } else {
                content.clone()
            }
        } else {
            String::new()
        }
    }

    /// Get choice colors (from choice_renderer.rs)
    fn get_choice_colors(&self, choice: &Choice) -> (&Option<String>, &Option<String>) {
        if choice.selected {
            (self.selected_menu_fg_color, self.selected_menu_bg_color)
        } else {
            (self.menu_fg_color, self.menu_bg_color)
        }
    }

    /// Calculate maximum choice width for horizontal scrolling
    fn get_max_choice_width(&self) -> usize {
        self.choices
            .iter()
            .map(|choice| self.format_choice_content(choice).len())
            .max()
            .unwrap_or(0)
    }

    /// FIXED: Render scrollable choices with proper horizontal bounds checking
    /// This fixes the broken logic that renders outside the box
    fn render_scrollable_choices_fixed(
        &self,
        bounds: &Bounds,
        x_offset: usize,
        y_offset: usize,
        buffer: &mut ScreenBuffer,
    ) {
        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        // Apply vertical scroll offset like working text rendering
        let visible_choices = self.choices.iter().skip(y_offset).take(viewable_height);

        let choice_start_x = bounds.left() + 2;
        let choice_start_y = bounds.top() + 2;

        for (i, choice) in visible_choices.enumerate() {
            let render_y = choice_start_y + i;
            if render_y >= bounds.bottom() {
                break;
            }

            let (fg_color, bg_color) = self.get_choice_colors(choice);
            let formatted_content = self.format_choice_content(choice);

            // FIX: Apply horizontal scrolling like working text rendering
            let visible_part = if x_offset < formatted_content.len() {
                formatted_content.chars()
                    .skip(x_offset)
                    .take(viewable_width)
                    .collect::<String>()
            } else {
                String::new()
            };

            print_with_color_and_background_at(
                render_y,
                choice_start_x,
                fg_color,
                bg_color,
                &visible_part,
                buffer,
            );
        }
    }

    /// PRESERVED: Working wrap mode from draw_utils.rs render_wrapped_choices
    fn render_wrapped_choices_preserved(
        &self,
        bounds: &Bounds,
        max_width: usize,
        y_offset: usize,
        buffer: &mut ScreenBuffer,
    ) {
        let viewable_height = bounds.height().saturating_sub(4);
        
        // Use existing working wrap logic from draw_utils.rs
        let mut all_wrapped_lines = Vec::new();
        for choice in self.choices.iter() {
            let (fg_color, bg_color) = self.get_choice_colors(choice);
            let formatted_content = self.format_choice_content(choice);
            let wrapped_lines = wrap_text_to_width(&formatted_content, max_width);

            for wrapped_line in wrapped_lines {
                all_wrapped_lines.push((wrapped_line, fg_color, bg_color));
            }
        }

        // Render visible wrapped lines using preserved logic
        let choice_start_x = bounds.left() + 2;
        let choice_start_y = bounds.top() + 2;

        for (i, (wrapped_line, fg_color, bg_color)) in all_wrapped_lines
            .iter()
            .skip(y_offset)
            .take(viewable_height)
            .enumerate()
        {
            let render_y = choice_start_y + i;
            if render_y >= bounds.bottom() {
                break;
            }

            print_with_color_and_background_at(
                render_y,
                choice_start_x,
                fg_color,
                bg_color,
                wrapped_line,
                buffer,
            );
        }
    }

    /// Calculate wrapped choice dimensions
    fn calculate_wrapped_dimensions(&self, max_width: usize) -> (usize, usize) {
        let mut total_wrapped_lines = 0;
        let mut max_wrapped_width = 0;

        for choice in self.choices.iter() {
            let formatted_content = self.format_choice_content(choice);
            let wrapped_lines = wrap_text_to_width(&formatted_content, max_width);
            total_wrapped_lines += wrapped_lines.len();
            
            for line in &wrapped_lines {
                max_wrapped_width = max_wrapped_width.max(line.len());
            }
        }

        (max_wrapped_width, total_wrapped_lines)
    }

    /// Generate clickable zones for choices accounting for scroll offsets
    fn generate_clickable_zones(
        &self,
        bounds: &Bounds,
        x_offset: usize,
        y_offset: usize,
        is_wrapped: bool,
        max_width: Option<usize>,
    ) -> Vec<ClickableZone> {
        let mut zones = Vec::new();
        let viewable_height = bounds.height().saturating_sub(4);
        let choice_start_x = bounds.left() + 2;
        let choice_start_y = bounds.top() + 2;

        if is_wrapped {
            // Wrapped choices - multiple zones per choice
            let mut all_wrapped_lines = Vec::new();
            for (choice_idx, choice) in self.choices.iter().enumerate() {
                let formatted_content = self.format_choice_content(choice);
                let wrapped_lines = wrap_text_to_width(&formatted_content, max_width.unwrap_or(80));

                for (line_idx, wrapped_line) in wrapped_lines.iter().enumerate() {
                    all_wrapped_lines.push((choice_idx, line_idx, wrapped_line.clone()));
                }
            }

            // Create zones for visible wrapped lines
            for (i, (choice_idx, line_idx, wrapped_line)) in all_wrapped_lines
                .iter()
                .skip(y_offset)
                .take(viewable_height)
                .enumerate()
            {
                let render_y = choice_start_y + i;
                if render_y >= bounds.bottom() {
                    break;
                }

                let zone_bounds = Bounds::new(
                    choice_start_x,
                    render_y,
                    choice_start_x + wrapped_line.len(),
                    render_y,
                );

                let metadata = ClickableMetadata {
                    display_text: Some(wrapped_line.clone()),
                    selected: self.choices[*choice_idx].selected,
                    enabled: true,
                    original_line: Some(*line_idx),
                    ..Default::default()
                };

                zones.push(ClickableZone::with_metadata(
                    zone_bounds,
                    format!("choice_{}", choice_idx),
                    ContentType::Choice,
                    metadata,
                ));
            }
        } else {
            // Regular choices - one zone per choice
            for (i, (choice_idx, choice)) in self.choices
                .iter()
                .enumerate()
                .skip(y_offset)
                .take(viewable_height)
                .enumerate()
            {
                let render_y = choice_start_y + i;
                if render_y >= bounds.bottom() {
                    break;
                }

                let formatted_content = self.format_choice_content(choice);
                let viewable_width = bounds.width().saturating_sub(4);
                
                // Account for horizontal scroll offset
                let visible_part = if x_offset < formatted_content.len() {
                    formatted_content.chars()
                        .skip(x_offset)
                        .take(viewable_width)
                        .collect::<String>()
                } else {
                    String::new()
                };

                let zone_bounds = Bounds::new(
                    choice_start_x,
                    render_y,
                    choice_start_x + visible_part.len(),
                    render_y,
                );

                let metadata = ClickableMetadata {
                    display_text: Some(visible_part),
                    selected: choice.selected,
                    enabled: true,
                    char_range: Some((x_offset, x_offset + viewable_width)),
                    ..Default::default()
                };

                zones.push(ClickableZone::with_metadata(
                    zone_bounds,
                    format!("choice_{}", choice_idx),
                    ContentType::Choice,
                    metadata,
                ));
            }
        }

        zones
    }
}

impl<'a> RenderableContent for ChoiceContent<'a> {
    /// Get raw choice dimensions - maximum choice width and total choice count
    fn get_dimensions(&self) -> (usize, usize) {
        let max_width = self.get_max_choice_width();
        let height = self.choices.len();
        (max_width, height)
    }

    /// Get raw content string for choices
    fn get_raw_content(&self) -> String {
        self.choices.iter()
            .map(|choice| choice.id.clone())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Get box-relative clickable zones for choices - raw row/col positions
    fn get_box_relative_clickable_zones(&self) -> Vec<ClickableZone> {
        let mut zones = Vec::new();
        
        for (idx, choice) in self.choices.iter().enumerate() {
            zones.push(ClickableZone {
                bounds: Bounds::new(0, idx, choice.id.len(), 1), // Raw content: col 0, row idx, width=choice length, height=1
                content_id: format!("choice_{}", idx),
                content_type: crate::components::renderable_content::ContentType::Choice,
                metadata: Default::default(),
            });
        }
        
        zones
    }


    /// Handle content events on choices (click, hover, keypress, etc.)
    /// Note: Choice mutation happens at MuxBox level, this validates and processes events
    fn handle_event(&mut self, event: &ContentEvent) -> EventResult {
        match event.event_type {
            EventType::Click => {
                if let Some(zone_id) = &event.zone_id {
                    if let Some(idx_str) = zone_id.strip_prefix("choice_") {
                        if let Ok(choice_idx) = idx_str.parse::<usize>() {
                            if choice_idx < self.choices.len() {
                                // Valid choice click - actual mutation happens at MuxBox level
                                return EventResult::Handled;
                            }
                        }
                    }
                }
                EventResult::NotHandled
            }
            EventType::Hover => {
                // Handle hover events for choice highlighting
                if let Some(hover_info) = event.hover_info() {
                    match hover_info.state {
                        HoverState::Enter => {
                            // Choice gained hover - could trigger visual feedback
                            EventResult::HandledContinue // Allow tooltips
                        }
                        HoverState::Leave => {
                            // Choice lost hover - remove visual feedback
                            EventResult::HandledContinue
                        }
                        HoverState::Move => {
                            // Mouse moving within choice - check for tooltip timing
                            if hover_info.hover_duration.is_some() {
                                EventResult::HandledContinue // Show tooltip
                            } else {
                                EventResult::NotHandled
                            }
                        }
                    }
                } else {
                    EventResult::NotHandled
                }
            }
            EventType::MouseMove => {
                // Handle mouse movement over choices
                if let Some(mouse_move) = event.mouse_move_info() {
                    if mouse_move.is_dragging {
                        // Dragging over choices - could be selection
                        EventResult::NotHandled // Let higher level handle drag selection
                    } else {
                        // Normal mouse movement - update hover state
                        EventResult::HandledContinue
                    }
                } else {
                    EventResult::NotHandled
                }
            }
            EventType::KeyPress => {
                // Handle keyboard navigation within choices
                if let Some(key_info) = event.key_info() {
                    match key_info.key.as_str() {
                        "ArrowUp" | "ArrowDown" | "Enter" | "Space" => {
                            EventResult::Handled // Navigation keys are handled
                        }
                        _ => EventResult::NotHandled
                    }
                } else {
                    EventResult::NotHandled
                }
            }
            EventType::Focus => {
                // Choice gained focus
                EventResult::StateChanged
            }
            EventType::Blur => {
                // Choice lost focus
                EventResult::StateChanged  
            }
            _ => EventResult::NotHandled
        }
    }

}