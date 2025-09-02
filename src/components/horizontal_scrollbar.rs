use crate::{Bounds, ScreenBuffer};
use crate::draw_utils::print_with_color_and_background_at;

/// Horizontal Scrollbar Component
/// 
/// Extracted from existing static content scrollbar implementation (F0357)
/// Provides unified horizontal scrollbar functionality with:
/// - Context awareness of parent box  
/// - Automatic visibility detection
/// - Proportional knob sizing
/// - Click-to-jump and drag-to-scroll support
/// - Transparent behavior matching existing implementation
#[derive(Debug, Clone)]
pub struct HorizontalScrollbar {
    /// Parent muxbox ID for context awareness
    pub parent_id: String,
    /// Scrollbar track character (from existing implementation)
    track_char: &'static str,
    /// Scrollbar knob character (from existing implementation)
    knob_char: &'static str,
}

impl HorizontalScrollbar {
    /// Create new horizontal scrollbar component for a parent muxbox
    pub fn new(parent_id: String) -> Self {
        Self {
            parent_id,
            track_char: "─",  // H_SCROLL_TRACK from existing implementation
            knob_char: "■",   // H_SCROLL_CHAR from existing implementation
        }
    }
    
    /// Determine if scrollbar should be visible based on content dimensions
    pub fn should_draw(&self, content_width: usize, viewable_width: usize) -> bool {
        content_width > viewable_width
    }
    
    /// Get the bounds where this scrollbar should be drawn
    pub fn get_bounds(&self, parent_bounds: &Bounds) -> (usize, usize, usize) {
        let y = parent_bounds.bottom(); // Draw ON bottom border, replacing it
        let start_x = parent_bounds.left() + 1;
        let end_x = parent_bounds.right().saturating_sub(1); // Match border drawing bounds
        (y, start_x, end_x)
    }
    
    /// Calculate knob position and size based on scroll state and content dimensions
    pub fn calculate_knob_metrics(
        &self,
        content_width: usize,
        viewable_width: usize,
        horizontal_scroll: f64,
        track_width: usize
    ) -> (usize, usize) {
        if track_width == 0 {
            return (0, 0);
        }
        
        // Calculate proportional knob size (from existing implementation)
        let content_ratio = viewable_width as f64 / content_width as f64;
        let knob_size = std::cmp::max(1, (track_width as f64 * content_ratio).round() as usize);
        let available_track = track_width.saturating_sub(knob_size);
        
        // Calculate knob position (from existing implementation)
        let knob_position = if available_track > 0 {
            ((horizontal_scroll / 100.0) * available_track as f64).round() as usize
        } else {
            0
        };
        
        (knob_position, knob_size)
    }
    
    /// Draw the complete horizontal scrollbar (track + knob)
    /// This is extracted directly from the existing inline horizontal scrollbar drawing
    pub fn draw(
        &self,
        parent_bounds: &Bounds,
        content_width: usize,
        viewable_width: usize,
        horizontal_scroll: f64,
        border_color: &Option<String>,
        bg_color: &Option<String>,
        buffer: &mut ScreenBuffer,
    ) {
        if !self.should_draw(content_width, viewable_width) {
            return;
        }
        
        let (y, start_x, end_x) = self.get_bounds(parent_bounds);
        let track_width = end_x.saturating_sub(start_x);
        
        // Draw horizontal scroll track (exact copy from existing implementation)
        for x in start_x..end_x {
            print_with_color_and_background_at(
                y,
                x,
                &Some("bright_black".to_string()),
                bg_color,
                self.track_char,
                buffer,
            );
        }
        
        if track_width > 0 {
            let (knob_position, knob_size) = self.calculate_knob_metrics(
                content_width,
                viewable_width,
                horizontal_scroll,
                track_width
            );
            
            // Draw proportional horizontal scroll knob (exact copy from existing implementation)
            for i in 0..knob_size {
                let knob_x = start_x + knob_position + i;
                if knob_x < end_x {
                    print_with_color_and_background_at(
                        y,
                        knob_x,
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
    pub fn is_click_on_scrollbar(&self, click_x: usize, click_y: usize, parent_bounds: &Bounds) -> bool {
        let (y, start_x, end_x) = self.get_bounds(parent_bounds);
        click_y == y && click_x > parent_bounds.left() && click_x < parent_bounds.right()
    }
    
    /// Check if a click position is specifically on the scrollbar knob
    pub fn is_click_on_knob(
        &self,
        click_x: usize,
        click_y: usize,
        parent_bounds: &Bounds,
        content_width: usize,
        viewable_width: usize,
        horizontal_scroll: f64
    ) -> bool {
        if !self.is_click_on_scrollbar(click_x, click_y, parent_bounds) {
            return false;
        }
        
        let (_, start_x, end_x) = self.get_bounds(parent_bounds);
        let track_width = end_x.saturating_sub(start_x);
        
        if track_width == 0 {
            return false;
        }
        
        let (knob_position, knob_size) = self.calculate_knob_metrics(
            content_width,
            viewable_width,
            horizontal_scroll,
            track_width
        );
        
        let knob_start_x = start_x + knob_position;
        let knob_end_x = knob_start_x + knob_size;
        
        click_x >= knob_start_x && click_x < knob_end_x
    }
    
    /// Convert click position to scroll percentage
    pub fn click_position_to_scroll_percentage(
        &self,
        click_x: usize,
        parent_bounds: &Bounds
    ) -> f64 {
        let (_, start_x, end_x) = self.get_bounds(parent_bounds);
        let track_width = end_x.saturating_sub(start_x);
        
        if track_width == 0 {
            return 0.0;
        }
        
        let click_position = (click_x.saturating_sub(start_x)) as f64 / track_width as f64;
        (click_position * 100.0).min(100.0).max(0.0)
    }
    
    /// Calculate new scroll percentage based on drag movement
    pub fn drag_to_scroll_percentage(
        &self,
        start_x: u16,
        current_x: u16,
        start_scroll_percentage: f64,
        parent_bounds: &Bounds
    ) -> f64 {
        let (_, track_start_x, track_end_x) = self.get_bounds(parent_bounds);
        let track_width = track_end_x.saturating_sub(track_start_x);
        
        if track_width == 0 {
            return start_scroll_percentage;
        }
        
        let drag_delta = (current_x as isize) - (start_x as isize);
        let percentage_delta = (drag_delta as f64 / track_width as f64) * 100.0;
        
        (start_scroll_percentage + percentage_delta).min(100.0).max(0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_horizontal_scrollbar_creation() {
        let scrollbar = HorizontalScrollbar::new("test_muxbox".to_string());
        assert_eq!(scrollbar.parent_id, "test_muxbox");
        assert_eq!(scrollbar.track_char, "─");
        assert_eq!(scrollbar.knob_char, "■");
    }
    
    #[test]
    fn test_should_draw_logic() {
        let scrollbar = HorizontalScrollbar::new("test".to_string());
        
        // Should not draw when content fits
        assert!(!scrollbar.should_draw(10, 20));
        assert!(!scrollbar.should_draw(10, 10));
        
        // Should draw when content overflows
        assert!(scrollbar.should_draw(20, 10));
    }
    
    #[test]
    fn test_knob_metrics_calculation() {
        let scrollbar = HorizontalScrollbar::new("test".to_string());
        
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
        let scrollbar = HorizontalScrollbar::new("test".to_string());
        let bounds = Bounds::new(10, 10, 50, 30);
        
        // Click at left of track should be 0%
        let scroll_pct = scrollbar.click_position_to_scroll_percentage(11, &bounds);
        assert!((scroll_pct - 0.0).abs() < 0.01);
        
        // Click at right of track should be close to 100%
        let scroll_pct = scrollbar.click_position_to_scroll_percentage(48, &bounds);
        assert!((scroll_pct - 97.37).abs() < 1.0); // Right edge of track (now at position 48)
        
        // Click in middle should be ~50%
        let scroll_pct = scrollbar.click_position_to_scroll_percentage(30, &bounds);
        assert!((scroll_pct - 50.0).abs() < 1.0); // Approximately 50%
    }
}