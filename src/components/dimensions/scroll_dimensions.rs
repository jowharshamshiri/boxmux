use super::Orientation;
use crate::Bounds;

/// ScrollDimensions - Centralizes ALL scrolling and scrollbar mathematical operations
///
/// Eliminates ad hoc scrollbar math like:
/// - knob_size = std::cmp::max(1, (track_height as f64 * content_ratio).round() as usize)
/// - available_track = track_height.saturating_sub(knob_size)
/// - knob_position = ((scroll / 100.0) * available_track as f64).round() as usize
/// - All coordinate translation between scroll percentages and pixel positions
///
/// Replaces scattered scrollbar logic from vertical_scrollbar.rs, horizontal_scrollbar.rs, 
/// draw_loop.rs scrollbar detection, and box_renderer.rs scroll calculations.
#[derive(Debug, Clone)]
pub struct ScrollDimensions {
    /// Total content size (width, height)
    content_size: (usize, usize),
    /// Viewable area size (width, height)
    viewable_size: (usize, usize),
    /// Current scroll position as percentage (0.0-100.0) for (horizontal, vertical)
    scroll_position: (f64, f64),
    /// Parent bounds for track positioning
    parent_bounds: Bounds,
}

impl ScrollDimensions {
    /// Create new scroll dimensions
    pub fn new(
        content_size: (usize, usize),
        viewable_size: (usize, usize),
        scroll_position: (f64, f64),
        parent_bounds: Bounds,
    ) -> Self {
        Self {
            content_size,
            viewable_size,
            scroll_position: (
                scroll_position.0.clamp(0.0, 100.0),
                scroll_position.1.clamp(0.0, 100.0),
            ),
            parent_bounds,
        }
    }
    
    /// Update scroll position (with automatic clamping)
    pub fn set_scroll_position(&mut self, horizontal: f64, vertical: f64) {
        self.scroll_position = (
            horizontal.clamp(0.0, 100.0),
            vertical.clamp(0.0, 100.0),
        );
    }
    
    /// Check if scrollbar is needed for given orientation
    /// Replaces: content_height > viewable_height checks
    pub fn is_scrollbar_needed(&self, orientation: Orientation) -> bool {
        match orientation {
            Orientation::Vertical => self.content_size.1 > self.viewable_size.1,
            Orientation::Horizontal => self.content_size.0 > self.viewable_size.0,
        }
    }
    
    /// Get scrollbar track bounds (where scrollbar can be drawn)
    /// Centralizes track positioning logic from scrollbar components
    pub fn get_track_bounds(&self, orientation: Orientation) -> TrackBounds {
        match orientation {
            Orientation::Vertical => {
                let x = self.parent_bounds.right();
                let start_y = self.parent_bounds.top() + 1;
                let end_y = self.parent_bounds.bottom().saturating_sub(1);
                TrackBounds {
                    position: x,
                    start: start_y,
                    end: end_y,
                    length: end_y.saturating_sub(start_y),
                }
            }
            Orientation::Horizontal => {
                let y = self.parent_bounds.bottom();
                let start_x = self.parent_bounds.left() + 1;
                let end_x = self.parent_bounds.right().saturating_sub(1);
                TrackBounds {
                    position: y,
                    start: start_x,
                    end: end_x,
                    length: end_x.saturating_sub(start_x),
                }
            }
        }
    }
    
    /// Calculate scrollbar knob size
    /// Centralizes: knob_size = std::cmp::max(1, (track_length as f64 * content_ratio).round() as usize)
    pub fn calculate_knob_size(&self, orientation: Orientation) -> usize {
        let track = self.get_track_bounds(orientation);
        let track_length = track.length;
        
        if track_length == 0 {
            return 1;
        }
        
        let content_ratio = match orientation {
            Orientation::Vertical => {
                if self.content_size.1 == 0 {
                    1.0
                } else {
                    self.viewable_size.1 as f64 / self.content_size.1 as f64
                }
            }
            Orientation::Horizontal => {
                if self.content_size.0 == 0 {
                    1.0
                } else {
                    self.viewable_size.0 as f64 / self.content_size.0 as f64
                }
            }
        };
        
        std::cmp::max(1, (track_length as f64 * content_ratio).round() as usize)
    }
    
    /// Calculate scrollbar knob position within track
    /// Centralizes: knob_position = ((scroll / 100.0) * available_track as f64).round() as usize
    pub fn calculate_knob_position(&self, orientation: Orientation) -> usize {
        let track = self.get_track_bounds(orientation);
        let knob_size = self.calculate_knob_size(orientation);
        let available_track = track.length.saturating_sub(knob_size);
        
        if available_track == 0 {
            return 0;
        }
        
        let scroll_percent = match orientation {
            Orientation::Vertical => self.scroll_position.1,
            Orientation::Horizontal => self.scroll_position.0,
        };
        
        ((scroll_percent / 100.0) * available_track as f64).round() as usize
    }
    
    /// Get absolute knob bounds for drawing
    /// Centralizes knob positioning from scrollbar components
    pub fn get_knob_bounds(&self, orientation: Orientation) -> KnobBounds {
        let track = self.get_track_bounds(orientation);
        let knob_size = self.calculate_knob_size(orientation);
        let knob_position = self.calculate_knob_position(orientation);
        
        match orientation {
            Orientation::Vertical => KnobBounds {
                x: track.position,
                y_start: track.start + knob_position,
                y_end: track.start + knob_position + knob_size,
                size: knob_size,
            },
            Orientation::Horizontal => KnobBounds {
                x: track.start + knob_position,
                y_start: track.position,
                y_end: track.position,
                size: knob_size,
            },
        }
    }
    
    /// Calculate scroll offset in content coordinates
    /// Centralizes: offset = ((scroll / 100.0) * max_offset as f64).floor() as usize
    pub fn calculate_scroll_offset(&self, orientation: Orientation) -> usize {
        let max_offset = match orientation {
            Orientation::Vertical => {
                if self.content_size.1 <= self.viewable_size.1 {
                    0
                } else {
                    self.content_size.1 - self.viewable_size.1
                }
            }
            Orientation::Horizontal => {
                if self.content_size.0 <= self.viewable_size.0 {
                    0
                } else {
                    self.content_size.0 - self.viewable_size.0
                }
            }
        };
        
        if max_offset == 0 {
            return 0;
        }
        
        let scroll_percent = match orientation {
            Orientation::Vertical => self.scroll_position.1,
            Orientation::Horizontal => self.scroll_position.0,
        };
        
        ((scroll_percent / 100.0) * max_offset as f64).floor() as usize
    }
    
    /// Convert pixel position within track to scroll percentage
    /// Centralizes click-to-scroll calculations from draw_loop.rs
    pub fn pixel_to_scroll_percent(
        &self,
        pixel_coordinate: usize,
        orientation: Orientation,
    ) -> f64 {
        let track = self.get_track_bounds(orientation);
        
        if track.length == 0 {
            return 0.0;
        }
        
        let relative_position = pixel_coordinate.saturating_sub(track.start);
        let click_position = relative_position as f64 / track.length as f64;
        
        (click_position * 100.0).clamp(0.0, 100.0)
    }
    
    /// Test if coordinate hits the scrollbar knob
    /// Centralizes knob hit testing from draw_loop.rs
    pub fn hits_knob(&self, x: usize, y: usize, orientation: Orientation) -> bool {
        let knob = self.get_knob_bounds(orientation);
        
        match orientation {
            Orientation::Vertical => {
                x == knob.x && y >= knob.y_start && y < knob.y_end
            }
            Orientation::Horizontal => {
                y == knob.y_start && x >= knob.x && x < (knob.x + knob.size)
            }
        }
    }
    
    /// Test if coordinate hits the scrollbar track (but not knob)
    /// Centralizes track click detection for jump-to-position
    pub fn hits_track(&self, x: usize, y: usize, orientation: Orientation) -> bool {
        let track = self.get_track_bounds(orientation);
        
        match orientation {
            Orientation::Vertical => {
                x == track.position
                    && y >= track.start
                    && y <= track.end
                    && !self.hits_knob(x, y, orientation)
            }
            Orientation::Horizontal => {
                y == track.position
                    && x >= track.start
                    && x <= track.end
                    && !self.hits_knob(x, y, orientation)
            }
        }
    }
    
    /// Calculate new scroll percentage from knob drag operation
    /// Centralizes drag-to-scroll calculations
    pub fn calculate_drag_scroll(
        &self,
        start_pixel: usize,
        current_pixel: usize,
        orientation: Orientation,
    ) -> f64 {
        let track = self.get_track_bounds(orientation);
        let knob_size = self.calculate_knob_size(orientation);
        let available_track = track.length.saturating_sub(knob_size);
        
        if available_track == 0 {
            return match orientation {
                Orientation::Vertical => self.scroll_position.1,
                Orientation::Horizontal => self.scroll_position.0,
            };
        }
        
        let pixel_delta = current_pixel as i32 - start_pixel as i32;
        let percentage_delta = (pixel_delta as f64 / available_track as f64) * 100.0;
        
        let current_scroll = match orientation {
            Orientation::Vertical => self.scroll_position.1,
            Orientation::Horizontal => self.scroll_position.0,
        };
        
        (current_scroll + percentage_delta).clamp(0.0, 100.0)
    }
    
    /// Get visible content range (start, end) for given orientation  
    /// Centralizes visible range calculations for content clipping
    pub fn get_visible_range(&self, orientation: Orientation) -> (usize, usize) {
        let scroll_offset = self.calculate_scroll_offset(orientation);
        let viewable_size = match orientation {
            Orientation::Vertical => self.viewable_size.1,
            Orientation::Horizontal => self.viewable_size.0,
        };
        
        let start = scroll_offset;
        let end = scroll_offset + viewable_size;
        
        (start, end)
    }
    
    /// Calculate auto-scroll adjustment to keep target line visible
    /// Centralizes auto-scroll logic from draw_loop.rs choice navigation
    pub fn calculate_auto_scroll_to_line(
        &self,
        target_line: usize,
        orientation: Orientation,
    ) -> f64 {
        let (visible_start, visible_end) = self.get_visible_range(orientation);
        let viewable_size = match orientation {
            Orientation::Vertical => self.viewable_size.1,
            Orientation::Horizontal => self.viewable_size.0,
        };
        let content_size = match orientation {
            Orientation::Vertical => self.content_size.1,
            Orientation::Horizontal => self.content_size.0,
        };
        
        // Already visible
        if target_line >= visible_start && target_line < visible_end {
            return match orientation {
                Orientation::Vertical => self.scroll_position.1,
                Orientation::Horizontal => self.scroll_position.0,
            };
        }
        
        let new_offset = if target_line < visible_start {
            // Scroll up to show target line at top
            target_line
        } else {
            // Scroll down to show target line at bottom
            target_line.saturating_sub(viewable_size - 1)
        };
        
        if content_size <= viewable_size {
            return 0.0;
        }
        
        let max_offset = content_size - viewable_size;
        let scroll_percent = (new_offset as f64 / max_offset as f64) * 100.0;
        
        scroll_percent.clamp(0.0, 100.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrackBounds {
    /// Position of the track (x for vertical, y for horizontal)
    pub position: usize,
    /// Start coordinate along track direction
    pub start: usize,
    /// End coordinate along track direction
    pub end: usize,
    /// Total track length
    pub length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct KnobBounds {
    /// X coordinate of knob
    pub x: usize,
    /// Start Y coordinate of knob
    pub y_start: usize,
    /// End Y coordinate of knob
    pub y_end: usize,
    /// Knob size
    pub size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_scrollbar_needed() {
        let bounds = Bounds::new(0, 0, 10, 5);
        let scroll_dims = ScrollDimensions::new(
            (20, 15), // content larger than viewable
            (10, 5),  // viewable size
            (0.0, 0.0),
            bounds,
        );
        
        assert!(scroll_dims.is_scrollbar_needed(Orientation::Horizontal));
        assert!(scroll_dims.is_scrollbar_needed(Orientation::Vertical));
    }
    
    #[test]
    fn test_knob_size_calculation() {
        let bounds = Bounds::new(0, 0, 10, 10);
        let scroll_dims = ScrollDimensions::new(
            (20, 20), // content is 2x viewable size
            (10, 10), // viewable size
            (0.0, 0.0),
            bounds,
        );
        
        // Track length is bounds.height() - 2 = 8 (top+1, bottom-1)
        // Content ratio is 10/20 = 0.5
        // Knob size should be 8 * 0.5 = 4
        let knob_size = scroll_dims.calculate_knob_size(Orientation::Vertical);
        assert_eq!(knob_size, 4);
    }
    
    #[test]
    fn test_scroll_offset_calculation() {
        let bounds = Bounds::new(0, 0, 10, 10);
        let mut scroll_dims = ScrollDimensions::new(
            (20, 20), // content size
            (10, 10), // viewable size  
            (50.0, 50.0), // 50% scroll
            bounds,
        );
        
        // Max offset = 20 - 10 = 10
        // 50% of 10 = 5
        let offset = scroll_dims.calculate_scroll_offset(Orientation::Vertical);
        assert_eq!(offset, 5);
        
        // Test 100% scroll
        scroll_dims.set_scroll_position(100.0, 100.0);
        let offset = scroll_dims.calculate_scroll_offset(Orientation::Vertical);
        assert_eq!(offset, 10);
    }
    
    #[test]
    fn test_pixel_to_scroll_percent() {
        let bounds = Bounds::new(0, 0, 10, 10);
        let scroll_dims = ScrollDimensions::new(
            (20, 20),
            (10, 10),
            (0.0, 0.0),
            bounds,
        );
        
        // Vertical track: start=1, end=9, length=8
        // Click at pixel 5 (middle of track) should be 50%
        let track = scroll_dims.get_track_bounds(Orientation::Vertical);
        let middle_pixel = track.start + track.length / 2;
        let percent = scroll_dims.pixel_to_scroll_percent(middle_pixel, Orientation::Vertical);
        
        assert!((percent - 50.0).abs() < 1.0); // Allow small floating point error
    }
    
    #[test]
    fn test_knob_hit_testing() {
        let bounds = Bounds::new(0, 0, 10, 10);
        let scroll_dims = ScrollDimensions::new(
            (20, 20),
            (10, 10),
            (0.0, 0.0), // No scroll - knob at top
            bounds,
        );
        
        let knob = scroll_dims.get_knob_bounds(Orientation::Vertical);
        
        // Should hit knob at its position
        assert!(scroll_dims.hits_knob(knob.x, knob.y_start, Orientation::Vertical));
        
        // Should not hit outside knob
        assert!(!scroll_dims.hits_knob(knob.x, knob.y_end + 1, Orientation::Vertical));
    }
    
    #[test]
    fn test_auto_scroll_to_line() {
        let bounds = Bounds::new(0, 0, 10, 10);
        let scroll_dims = ScrollDimensions::new(
            (10, 100), // 100 lines of content
            (10, 10),  // 10 lines visible
            (0.0, 0.0), // Start at top
            bounds,
        );
        
        // Target line 50 should scroll to show it
        let scroll_percent = scroll_dims.calculate_auto_scroll_to_line(50, Orientation::Vertical);
        
        // Should be around 50% scroll to center line 50
        assert!(scroll_percent > 40.0 && scroll_percent < 60.0);
    }
}