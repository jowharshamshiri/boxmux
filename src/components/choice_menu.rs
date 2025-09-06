use crate::components::renderable_content::{
    SensitiveMetadata, SensitiveZone, ContentEvent, ContentType, EventResult, EventType,
    RenderableContent,
};
use crate::model::choice::Choice;
use crate::Bounds;

/// ChoiceMenu component - generates choice content and sensitive zones for BoxRenderer
///
/// This component handles choice logic (selection, waiting states, content generation)
/// and outputs content strings and sensitive zones that BoxRenderer treats like any other content.
pub struct ChoiceMenu<'a> {
    /// Reference to the choices data
    choices: &'a [Choice],
    /// Selected choice index (if any)
    selected_index: Option<usize>,
    /// Focused choice index for keyboard navigation
    focused_index: Option<usize>,
    /// Component identifier
    _id: String,
}

impl<'a> ChoiceMenu<'a> {
    /// Create a new choice menu component
    pub fn new(id: String, choices: &'a [Choice]) -> Self {
        Self {
            choices,
            selected_index: None,
            focused_index: None,
            _id: id,
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
        self.generate_choice_lines().join("\n")
    }

    /// Get content as raw string for BoxRenderer to handle like text content
    pub fn get_raw_content(&self) -> String {
        self.generate_choice_content()
    }

    /// Generate sensitive zones accounting for text wrapping at specified width
    pub fn get_box_relative_sensitive_zones_with_width(&self, available_width: usize) -> Vec<SensitiveZone> {
        let mut zones = Vec::new();
        let mut current_line = 0;

        // Generate the actual content that will be rendered (with indicators and waiting states)
        let content_lines = self.generate_choice_lines();

        for (index, choice) in self.choices.iter().enumerate() {
            if let Some(choice_content) = &choice.content {
                // Get the actual formatted line for this choice (with indicators)
                let formatted_line = &content_lines[index];
                
                if available_width == usize::MAX || formatted_line.len() <= available_width {
                    // Single line case - no wrapping needed
                    let zone_bounds = Bounds::new(
                        0,                                          // Start at x=0 (left edge of content area)
                        current_line,                              // y position based on actual rendered line
                        formatted_line.len().saturating_sub(1),    // End x coordinate based on formatted content
                        current_line,                              // Single line height, so y2 == y1
                    );

                    let metadata = SensitiveMetadata {
                        display_text: Some(formatted_line.clone()),
                        tooltip: Some(format!("Choice {}: {}", index, choice_content)),
                        selected: choice.selected,
                        enabled: !choice.waiting,
                        original_line: Some(index),
                        char_range: Some((0, formatted_line.len())),
                    };

                    zones.push(SensitiveZone::with_metadata(
                        zone_bounds,
                        format!("choice_{}", index),
                        ContentType::Choice,
                        metadata,
                    ));
                    
                    current_line += 1;
                } else {
                    // Multi-line case - choice wraps across multiple lines
                    let mut remaining_text = formatted_line.clone();
                    let mut char_offset = 0;
                    
                    while !remaining_text.is_empty() {
                        let line_width = remaining_text.len().min(available_width);
                        let line_text = remaining_text[..line_width].to_string();
                        
                        let zone_bounds = Bounds::new(
                            0,                              // Start at x=0 (left edge of content area)
                            current_line,                   // y position for this wrapped segment
                            line_width.saturating_sub(1),   // End x coordinate for this segment
                            current_line,                   // Single line height, so y2 == y1
                        );

                        let metadata = SensitiveMetadata {
                            display_text: Some(line_text),
                            tooltip: Some(format!("Choice {}: {} (part)", index, choice_content)),
                            selected: choice.selected,
                            enabled: !choice.waiting,
                            original_line: Some(index),
                            char_range: Some((char_offset, char_offset + line_width)),
                        };

                        zones.push(SensitiveZone::with_metadata(
                            zone_bounds,
                            format!("choice_{}", index),
                            ContentType::Choice,
                            metadata,
                        ));
                        
                        remaining_text = remaining_text[line_width..].to_string();
                        char_offset += line_width;
                        current_line += 1;
                    }
                }
            }
        }

        zones
    }

    /// Generate the formatted choice lines as they will actually be rendered
    pub fn generate_choice_lines(&self) -> Vec<String> {
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

        content_lines
    }
}

impl<'a> RenderableContent for ChoiceMenu<'a> {
    /// Get the raw content as a string
    fn get_raw_content(&self) -> String {
        self.generate_choice_content()
    }

    /// Get sensitive zones in box-relative coordinates (0,0 = top-left of content area)
    fn get_box_relative_sensitive_zones(&self) -> Vec<SensitiveZone> {
        self.get_box_relative_sensitive_zones_with_width(usize::MAX)
    }

    /// Handle content events on choice menu
    /// Note: This doesn't mutate the choices directly - that's handled at the MuxBox level
    fn handle_event(&mut self, event: &ContentEvent) -> EventResult {
        match event.event_type {
            EventType::Click => {
                if let Some(zone_id) = &event.zone_id {
                    log::info!(
                        "CHOICE CLICK: ChoiceMenu handling click on zone '{}'",
                        zone_id
                    );

                    if let Some(index_str) = zone_id.strip_prefix("choice_") {
                        if let Ok(choice_index) = index_str.parse::<usize>() {
                            if choice_index < self.choices.len() {
                                // Store the clicked choice index for external handling
                                self.selected_index = Some(choice_index);

                                log::info!(
                                    "CHOICE CLICK: Registered click on choice index {}",
                                    choice_index
                                );

                                // Actual choice mutation happens at the MuxBox level
                                return EventResult::Handled;
                            }
                        }
                    }
                }
                EventResult::NotHandled
            }
            EventType::Hover => {
                // Handle hover for choice highlighting
                if event.zone_id.is_some() {
                    EventResult::HandledContinue
                } else {
                    EventResult::NotHandled
                }
            }
            EventType::KeyPress => {
                // Handle keyboard navigation
                if let Some(key_info) = event.key_info() {
                    match key_info.key.as_str() {
                        "ArrowUp" | "ArrowDown" | "Enter" | "Space" => EventResult::Handled,
                        _ => EventResult::NotHandled,
                    }
                } else {
                    EventResult::NotHandled
                }
            }
            _ => EventResult::NotHandled,
        }
    }

    /// Get the raw content dimensions before any transformations
    fn get_dimensions(&self) -> (usize, usize) {
        let content = self.generate_choice_content();
        let lines: Vec<&str> = content.lines().collect();
        let height = lines.len();
        let width = lines.iter().map(|line| line.len()).max().unwrap_or(0);
        (width, height)
    }
}
