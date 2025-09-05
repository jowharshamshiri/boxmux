use crate::components::renderable_content::{
    ClickableZone, ContentEvent, EventResult, EventType, HoverState, RenderableContent,
};
use crate::draw_utils::content_size;

/// TextContent implementation of RenderableContent trait
/// Preserves all existing text rendering logic from draw_utils.rs
pub struct TextContent<'a> {
    /// The raw text content to render
    text: &'a str,
    /// Text foreground color
    _fg_color: &'a Option<String>,
    /// Text background color  
    _bg_color: &'a Option<String>,
}

impl<'a> TextContent<'a> {
    /// Create new TextContent
    pub fn new(text: &'a str, fg_color: &'a Option<String>, bg_color: &'a Option<String>) -> Self {
        Self {
            text,
            _fg_color: fg_color,
            _bg_color: bg_color,
        }
    }

    /// Calculate content dimensions using existing logic
    fn calculate_dimensions(&self) -> (usize, usize) {
        content_size(self.text)
    }
}

impl<'a> RenderableContent for TextContent<'a> {
    /// Get raw content dimensions using existing content_size logic
    fn get_dimensions(&self) -> (usize, usize) {
        self.calculate_dimensions()
    }

    /// Get raw content string
    fn get_raw_content(&self) -> String {
        self.text.to_string()
    }

    /// Get box-relative clickable zones
    fn get_box_relative_clickable_zones(&self) -> Vec<ClickableZone> {
        Vec::new() // Text content typically doesn't have clickable zones
    }

    /// Handle content events on text
    fn handle_event(&mut self, event: &ContentEvent) -> EventResult {
        match event.event_type {
            EventType::Click => {
                // Text content generally doesn't handle clicks unless it has links
                EventResult::NotHandled
            }
            EventType::KeyPress => {
                // Text could handle copy operations, search, etc.
                if let Some(key_info) = event.key_info() {
                    match key_info.key.as_str() {
                        "c" if key_info.modifiers.contains(
                            &crate::components::renderable_content::KeyModifier::Ctrl,
                        ) =>
                        {
                            // Ctrl+C for copy - could be handled here or at higher level
                            EventResult::NotHandled
                        }
                        "/" | "f"
                            if key_info.modifiers.contains(
                                &crate::components::renderable_content::KeyModifier::Ctrl,
                            ) =>
                        {
                            // Search functionality
                            EventResult::NotHandled
                        }
                        _ => EventResult::NotHandled,
                    }
                } else {
                    EventResult::NotHandled
                }
            }
            EventType::Scroll => {
                // Scroll events could be handled for text navigation
                EventResult::HandledContinue // Let scrollbars handle but continue propagation
            }
            EventType::MouseMove => {
                // Handle mouse movement over text for cursor changes, selection
                if let Some(mouse_move) = event.mouse_move_info() {
                    if mouse_move.is_dragging {
                        // Text selection via drag
                        EventResult::NotHandled // Could be handled for text selection
                    } else {
                        // Normal movement - change cursor, show position
                        EventResult::NotHandled
                    }
                } else {
                    EventResult::NotHandled
                }
            }
            EventType::Hover => {
                // Handle hover over text for tooltips, word highlighting
                if let Some(hover_info) = event.hover_info() {
                    match hover_info.state {
                        HoverState::Enter | HoverState::Move => {
                            // Could show word definitions, tooltips
                            EventResult::NotHandled
                        }
                        HoverState::Leave => {
                            // Hide tooltips
                            EventResult::NotHandled
                        }
                    }
                } else {
                    EventResult::NotHandled
                }
            }
            EventType::BoxResize => {
                // Text content needs to reflow on resize
                EventResult::StateChanged // Trigger re-render with new dimensions
            }
            EventType::TitleChange => {
                // Text content doesn't typically handle title changes
                EventResult::NotHandled
            }
            _ => EventResult::NotHandled,
        }
    }
}
