use crate::draw_utils::print_with_color_and_background_at;
use crate::{Bounds, ScreenBuffer};
use crate::components::ComponentDimensions;

/// Vertical Scrollbar Component
///
/// Extracted from existing static content scrollbar implementation (F0356)
/// Provides unified vertical scrollbar functionality with:
/// - Context awareness of parent box
/// - Automatic visibility detection
/// - Proportional knob sizing
/// - Click-to-jump and drag-to-scroll support
/// - Transparent behavior matching existing implementation
#[derive(Debug, Clone)]
pub struct VerticalScrollbar {
    /// Parent muxbox ID for context awareness
    pub parent_id: String,
    /// Scrollbar track character (from existing implementation)
    track_char: &'static str,
    /// Scrollbar knob character (from existing implementation)
    knob_char: &'static str,
}

impl VerticalScrollbar {
    /// Create new vertical scrollbar component for a parent muxbox
    pub fn new(parent_id: String) -> Self {
        Self {
            parent_id,
            track_char: "│", // V_SCROLL_TRACK from existing implementation
            knob_char: "█",  // V_SCROLL_CHAR from existing implementation
        }
    }

    /// Determine if scrollbar should be visible based on content dimensions
    pub fn should_draw(&self, content_height: usize, viewable_height: usize) -> bool {
        content_height > viewable_height
    }

    /// Get the bounds where this scrollbar should be drawn
    pub fn get_bounds(&self, parent_bounds: &Bounds) -> (usize, usize, usize) {
        let component_dims = ComponentDimensions::new(*parent_bounds);
        let track_bounds = component_dims.vertical_scrollbar_track_bounds();
        (track_bounds.x1, track_bounds.y1, track_bounds.y2)
    }

    /// Calculate knob position and size using ScrollDimensions (eliminates ad-hoc math)
    pub fn calculate_knob_metrics(
        &self,
        content_height: usize,
        viewable_height: usize,
        vertical_scroll: f64,
        track_height: usize,
    ) -> (usize, usize) {
        if track_height == 0 {
            return (0, 0);
        }

        // Use ScrollDimensions for proper calculations
        use crate::components::dimensions::{ScrollDimensions, Orientation};
        use crate::Bounds;
        
        let scroll_dims = ScrollDimensions::new(
            (1, content_height), // Use 1 for width since this is vertical scrollbar
            (1, viewable_height),
            (0.0, vertical_scroll), // Only vertical scroll matters
            Bounds::new(0, 0, 2, track_height + 1), // Adjust bounds so track_length = track_height
        );
        
        let knob_size = scroll_dims.calculate_knob_size(Orientation::Vertical);
        let knob_position = scroll_dims.calculate_knob_position(Orientation::Vertical);
        
        (knob_position, knob_size)
    }

    /// Draw the complete vertical scrollbar (track + knob)
    /// This is extracted directly from the existing draw_vertical_scrollbar function
    pub fn draw(
        &self,
        parent_bounds: &Bounds,
        content_height: usize,
        viewable_height: usize,
        vertical_scroll: f64,
        border_color: &Option<String>,
        bg_color: &Option<String>,
        buffer: &mut ScreenBuffer,
    ) {
        if !self.should_draw(content_height, viewable_height) {
            return;
        }

        let (x, start_y, end_y) = self.get_bounds(parent_bounds);
        let track_height = end_y.saturating_sub(start_y);

        // Draw vertical scroll track (exact copy from existing implementation)
        for y in start_y..end_y {
            print_with_color_and_background_at(
                y,
                x,
                &Some("bright_black".to_string()),
                bg_color,
                self.track_char,
                buffer,
            );
        }

        if track_height > 0 {
            let (knob_position, knob_size) = self.calculate_knob_metrics(
                content_height,
                viewable_height,
                vertical_scroll,
                track_height,
            );

            // Draw proportional vertical scroll knob (exact copy from existing implementation)
            for i in 0..knob_size {
                let knob_y = start_y + knob_position + i;
                if knob_y < end_y {
                    print_with_color_and_background_at(
                        knob_y,
                        x,
                        border_color,
                        bg_color,
                        self.knob_char,
                        buffer,
                    );
                }
            }
        }
    }

    /// Check if a click position is on this scrollbar
    pub fn is_click_on_scrollbar(
        &self,
        click_x: usize,
        click_y: usize,
        parent_bounds: &Bounds,
    ) -> bool {
        let (x, _, _) = self.get_bounds(parent_bounds);
        click_x == x && click_y > parent_bounds.top() && click_y < parent_bounds.bottom()
    }

    /// Check if a click position is specifically on the scrollbar knob
    pub fn is_click_on_knob(
        &self,
        click_x: usize,
        click_y: usize,
        parent_bounds: &Bounds,
        content_height: usize,
        viewable_height: usize,
        vertical_scroll: f64,
    ) -> bool {
        if !self.is_click_on_scrollbar(click_x, click_y, parent_bounds) {
            return false;
        }

        let (_, start_y, end_y) = self.get_bounds(parent_bounds);
        let track_height = end_y.saturating_sub(start_y);

        if track_height == 0 {
            return false;
        }

        let (knob_position, knob_size) = self.calculate_knob_metrics(
            content_height,
            viewable_height,
            vertical_scroll,
            track_height,
        );

        let knob_start_y = start_y + knob_position;
        let knob_end_y = knob_start_y + knob_size;

        click_y >= knob_start_y && click_y < knob_end_y
    }

    /// Convert click position to scroll percentage
    pub fn click_position_to_scroll_percentage(
        &self,
        click_y: usize,
        parent_bounds: &Bounds,
    ) -> f64 {
        let (_, start_y, end_y) = self.get_bounds(parent_bounds);
        let track_height = end_y.saturating_sub(start_y);

        if track_height == 0 {
            return 0.0;
        }

        let click_position = (click_y.saturating_sub(start_y)) as f64 / track_height as f64;
        (click_position * 100.0).clamp(0.0, 100.0)
    }

    /// Calculate new scroll percentage based on drag movement
    pub fn drag_to_scroll_percentage(
        &self,
        start_y: u16,
        current_y: u16,
        start_scroll_percentage: f64,
        parent_bounds: &Bounds,
    ) -> f64 {
        let (_, track_start_y, track_end_y) = self.get_bounds(parent_bounds);
        let track_height = track_end_y.saturating_sub(track_start_y);

        if track_height == 0 {
            return start_scroll_percentage;
        }

        let drag_delta = (current_y as isize) - (start_y as isize);
        let percentage_delta = (drag_delta as f64 / track_height as f64) * 100.0;

        (start_scroll_percentage + percentage_delta).clamp(0.0, 100.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertical_scrollbar_creation() {
        let scrollbar = VerticalScrollbar::new("test_muxbox".to_string());
        assert_eq!(scrollbar.parent_id, "test_muxbox");
        assert_eq!(scrollbar.track_char, "│");
        assert_eq!(scrollbar.knob_char, "█");
    }

    #[test]
    fn test_should_draw_logic() {
        let scrollbar = VerticalScrollbar::new("test".to_string());

        // Should not draw when content fits
        assert!(!scrollbar.should_draw(10, 20));
        assert!(!scrollbar.should_draw(10, 10));

        // Should draw when content overflows
        assert!(scrollbar.should_draw(20, 10));
    }

    #[test]
    fn test_knob_metrics_calculation() {
        let scrollbar = VerticalScrollbar::new("test".to_string());

        // Test proportional knob sizing
        let (position, size) = scrollbar.calculate_knob_metrics(100, 50, 0.0, 20);
        assert_eq!(size, 10); // 20 * (50/100) = 10
        assert_eq!(position, 0); // 0% scroll = position 0

        // Test knob position at 50% scroll
        let (position, _) = scrollbar.calculate_knob_metrics(100, 50, 50.0, 20);
        assert_eq!(position, 5); // 50% of available track (10)
    }

    #[test]
    fn test_click_position_conversion() {
        let scrollbar = VerticalScrollbar::new("test".to_string());
        let bounds = Bounds::new(10, 10, 50, 30);

        // Click at top of track should be 0%
        let scroll_pct = scrollbar.click_position_to_scroll_percentage(11, &bounds);
        assert!((scroll_pct - 0.0).abs() < 0.01);

        // Click at bottom of track should be close to 100%
        let scroll_pct = scrollbar.click_position_to_scroll_percentage(28, &bounds);
        assert!((scroll_pct - 94.44).abs() < 1.0); // Bottom edge of track (now at position 28)

        // Click in middle should be ~50%
        let scroll_pct = scrollbar.click_position_to_scroll_percentage(20, &bounds);
        assert!((scroll_pct - 50.0).abs() < 1.0); // Approximately 50%
    }
}
