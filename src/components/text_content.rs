use crate::components::renderable_content::{RenderableContent, ClickableZone, ContentType, ClickableMetadata, ContentDimensions};
use crate::{ScreenBuffer, Bounds};
use crate::draw_utils::{print_with_color_and_background_at, wrap_text_to_width, content_size};

/// TextContent implementation of RenderableContent trait
/// Preserves all existing text rendering logic from draw_utils.rs
pub struct TextContent<'a> {
    /// The raw text content to render
    text: &'a str,
    /// Text foreground color
    fg_color: &'a Option<String>,
    /// Text background color  
    bg_color: &'a Option<String>,
}

impl<'a> TextContent<'a> {
    /// Create new TextContent
    pub fn new(text: &'a str, fg_color: &'a Option<String>, bg_color: &'a Option<String>) -> Self {
        Self {
            text,
            fg_color,
            bg_color,
        }
    }

    /// Calculate content dimensions using existing logic
    fn calculate_dimensions(&self) -> (usize, usize) {
        content_size(self.text)
    }

    /// Get text lines for rendering
    fn get_text_lines(&self) -> Vec<&str> {
        self.text.lines().collect()
    }

    /// Calculate scroll offsets using existing logic (from box_renderer.rs)
    fn calculate_scroll_offset(&self, vertical_scroll: f64, viewable_height: usize, content_height: usize) -> usize {
        let max_vertical_offset = content_height.saturating_sub(viewable_height);
        ((vertical_scroll / 100.0) * max_vertical_offset as f64).floor() as usize
    }

    /// Calculate horizontal scroll offset
    fn calculate_horizontal_scroll_offset(&self, horizontal_scroll: f64, viewable_width: usize, content_width: usize) -> usize {
        let max_horizontal_offset = content_width.saturating_sub(viewable_width);
        ((horizontal_scroll / 100.0) * max_horizontal_offset as f64).floor() as usize
    }

    /// Render scrollable content using existing logic from box_renderer.rs:render_scrollable_content
    fn render_scrollable_content(
        &self,
        bounds: &Bounds,
        x_offset: usize,
        y_offset: usize,
        buffer: &mut ScreenBuffer,
    ) {
        let content_lines = self.get_text_lines();
        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        // Apply vertical scroll offset
        let visible_lines = content_lines
            .iter()
            .skip(y_offset)
            .take(viewable_height);

        let content_start_x = bounds.left() + 2;
        let content_start_y = bounds.top() + 2;

        for (i, line) in visible_lines.enumerate() {
            let render_y = content_start_y + i;
            if render_y >= bounds.bottom() {
                break;
            }

            // Apply horizontal scroll offset
            let visible_line = if x_offset < line.len() {
                &line[x_offset..]
            } else {
                ""
            };

            // Truncate to viewable width
            let truncated_line = if visible_line.len() > viewable_width {
                &visible_line[..viewable_width]
            } else {
                visible_line
            };

            // Use existing print function - preserves all text rendering logic
            print_with_color_and_background_at(
                render_y,
                content_start_x,
                self.fg_color,
                self.bg_color,
                truncated_line,
                buffer,
            );
        }
    }

    /// Render normal (non-scrollable) content using existing logic from box_renderer.rs:render_normal_content
    fn render_normal_content(
        &self,
        bounds: &Bounds,
        buffer: &mut ScreenBuffer,
    ) {
        let content_lines = self.get_text_lines();
        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        let max_content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        // Use existing centering logic
        let total_lines = content_lines.len();
        let vertical_padding = (viewable_height.saturating_sub(total_lines)) / 2;
        let horizontal_padding = (viewable_width.saturating_sub(max_content_width)) / 2;

        for (i, line) in content_lines.iter().enumerate().take(viewable_height) {
            let visible_line = if line.len() > viewable_width {
                &line[..viewable_width]
            } else {
                line
            };

            let render_x = bounds.left() + 2 + horizontal_padding;
            let render_y = bounds.top() + 2 + vertical_padding + i;

            if render_y >= bounds.bottom() {
                break;
            }

            // Use existing print function - preserves all text rendering logic
            print_with_color_and_background_at(
                render_y,
                render_x,
                self.fg_color,
                self.bg_color,
                visible_line,
                buffer,
            );
        }
    }
}

impl<'a> RenderableContent for TextContent<'a> {
    /// Get raw content dimensions using existing content_size logic
    fn get_dimensions(&self) -> (usize, usize) {
        self.calculate_dimensions()
    }

    /// Render text content within viewport using existing rendering logic
    fn render_viewport(&self, bounds: &Bounds, x_offset: usize, y_offset: usize, buffer: &mut ScreenBuffer) {
        if x_offset > 0 || y_offset > 0 {
            // Scrollable content - use existing scrollable rendering logic
            self.render_scrollable_content(bounds, x_offset, y_offset, buffer);
        } else {
            // Normal content - use existing normal rendering logic
            self.render_normal_content(bounds, buffer);
        }
    }

    /// Get clickable zones for text content
    /// Text content typically doesn't have clickable zones unless it contains links
    fn get_clickable_zones(&self, _bounds: &Bounds, _x_offset: usize, _y_offset: usize) -> Vec<ClickableZone> {
        // Text content doesn't have clickable zones by default
        // This could be extended in the future to detect URLs/links
        Vec::new()
    }

    /// Get dimensions after wrapping using existing wrap_text_to_width logic
    fn get_wrapped_dimensions(&self, max_width: usize) -> (usize, usize) {
        let wrapped_lines = wrap_text_to_width(self.text, max_width);
        let width = wrapped_lines.iter().map(|line| line.len()).max().unwrap_or(0);
        let height = wrapped_lines.len();
        (width, height)
    }

    /// Render wrapped text content using existing render_wrapped_content logic from draw_utils.rs
    fn render_wrapped_viewport(&self, bounds: &Bounds, max_width: usize, y_offset: usize, buffer: &mut ScreenBuffer) {
        let wrapped_lines = wrap_text_to_width(self.text, max_width);
        
        // Use existing render_wrapped_content logic but inline to preserve behavior
        let viewable_height = bounds.height().saturating_sub(4);
        let content_start_x = bounds.left() + 2;
        let content_start_y = bounds.top() + 2;

        // Render visible lines using existing logic from render_wrapped_content
        let visible_lines = wrapped_lines
            .iter()
            .skip(y_offset)
            .take(viewable_height);

        for (i, line) in visible_lines.enumerate() {
            let render_y = content_start_y + i;
            if render_y >= bounds.bottom() {
                break;
            }

            // Use existing print function - preserves all text rendering logic
            print_with_color_and_background_at(
                render_y,
                content_start_x,
                self.fg_color,
                self.bg_color,
                line,
                buffer,
            );
        }
    }

    /// Get clickable zones for wrapped text content
    fn get_wrapped_clickable_zones(&self, _bounds: &Bounds, _max_width: usize, _y_offset: usize) -> Vec<ClickableZone> {
        // Wrapped text content doesn't have clickable zones by default
        Vec::new()
    }
}