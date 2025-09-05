use crate::components::renderable_content::{
    ClickableZone, ContentEvent, EventResult, EventType, HoverState, RenderableContent,
};
use crate::model::muxbox::Choice;
use crate::Bounds;

/// ChoiceContent implementation of RenderableContent trait
/// FIXES the broken choice rendering that renders outside bounds and doesn't trigger horizontal scrollbars
/// PRESERVES the working wrap mode functionality
pub struct ChoiceContent<'a> {
    /// The choices to render
    choices: &'a [Choice],
    /// Menu foreground color
    _menu_fg_color: &'a Option<String>,
    /// Menu background color
    _menu_bg_color: &'a Option<String>,
    /// Selected choice foreground color
    _selected_menu_fg_color: &'a Option<String>,
    /// Selected choice background color
    _selected_menu_bg_color: &'a Option<String>,
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
            _menu_fg_color: menu_fg_color,
            _menu_bg_color: menu_bg_color,
            _selected_menu_fg_color: selected_menu_fg_color,
            _selected_menu_bg_color: selected_menu_bg_color,
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

    /// Calculate maximum choice width for horizontal scrolling
    fn get_max_choice_width(&self) -> usize {
        self.choices
            .iter()
            .map(|choice| self.format_choice_content(choice).len())
            .max()
            .unwrap_or(0)
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
        self.choices
            .iter()
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
                            // Mouse moving within choice - always continue to allow for tooltip handling
                            EventResult::HandledContinue
                        }
                    }
                } else {
                    // Basic hover event (no hover info) - still handled for choice highlighting
                    EventResult::HandledContinue
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
                        _ => EventResult::NotHandled,
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
            _ => EventResult::NotHandled,
        }
    }
}
