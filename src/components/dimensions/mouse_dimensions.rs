use crate::Bounds;

/// MouseDimensions - Centralizes ALL mouse coordinate translation and hit testing
///
/// Eliminates ad hoc coordinate math like:
/// - column as usize, row as usize conversions from crossterm
/// - inbox_x = screen_x.saturating_sub(content_start_x.saturating_sub(horizontal_offset))
/// - Complex coordinate system translations between screen, content, and scrolled spaces
/// - Hit testing scattered across multiple files
///
/// Replaces scattered coordinate translation from box_renderer.rs, draw_loop.rs, input_loop.rs
#[derive(Debug, Clone)]
pub struct MouseDimensions {
    /// Screen bounds (outer component bounds)
    screen_bounds: Bounds,
    /// Content bounds (inner drawable area)
    content_bounds: Bounds,
    /// Current scroll offset (horizontal, vertical) in pixels
    scroll_offset: (usize, usize),
    /// Content size (total scrollable area)
    content_size: (usize, usize),
}

impl MouseDimensions {
    /// Create new mouse dimensions
    pub fn new(
        screen_bounds: Bounds,
        content_bounds: Bounds,
        scroll_offset: (usize, usize),
        content_size: (usize, usize),
    ) -> Self {
        Self {
            screen_bounds,
            content_bounds,
            scroll_offset,
            content_size,
        }
    }
    
    /// Update scroll offset
    pub fn set_scroll_offset(&mut self, horizontal: usize, vertical: usize) {
        self.scroll_offset = (horizontal, vertical);
    }
    
    /// Convert crossterm coordinates (u16) to internal coordinates (usize)
    /// Centralizes: column as usize, row as usize conversions
    pub fn crossterm_to_internal(&self, column: u16, row: u16) -> (usize, usize) {
        (column as usize, row as usize)
    }
    
    /// Convert internal coordinates back to crossterm (for event generation)
    pub fn internal_to_crossterm(&self, x: usize, y: usize) -> (u16, u16) {
        (x as u16, y as u16)
    }
    
    /// Test if screen coordinates are within component bounds
    pub fn is_within_screen_bounds(&self, x: usize, y: usize) -> bool {
        self.screen_bounds.contains_point(x, y)
    }
    
    /// Test if screen coordinates are within content area
    pub fn is_within_content_bounds(&self, x: usize, y: usize) -> bool {
        self.content_bounds.contains_point(x, y)
    }
    
    /// Convert screen coordinates to content coordinates (accounting for scroll)
    /// Centralizes complex translation logic from box_renderer.rs
    pub fn screen_to_content(&self, screen_x: usize, screen_y: usize) -> Option<(usize, usize)> {
        // First check if within content bounds
        if !self.content_bounds.contains_point(screen_x, screen_y) {
            return None;
        }
        
        // Convert to content-relative coordinates
        let relative_x = screen_x.saturating_sub(self.content_bounds.x1);
        let relative_y = screen_y.saturating_sub(self.content_bounds.y1);
        
        // Apply scroll offset to get actual content coordinates
        let content_x = relative_x + self.scroll_offset.0;
        let content_y = relative_y + self.scroll_offset.1;
        
        // Validate within content size
        if content_x >= self.content_size.0 || content_y >= self.content_size.1 {
            return None;
        }
        
        Some((content_x, content_y))
    }
    
    /// Convert content coordinates to screen coordinates (reverse translation)
    pub fn content_to_screen(&self, content_x: usize, content_y: usize) -> Option<(usize, usize)> {
        // Check if content coordinates are valid
        if content_x >= self.content_size.0 || content_y >= self.content_size.1 {
            return None;
        }
        
        // Check if content coordinates are within the currently visible range
        // Visible range is from scroll_offset to scroll_offset + viewable_size
        let viewable_width = self.content_bounds.width();
        let viewable_height = self.content_bounds.height();
        
        if content_x < self.scroll_offset.0 || content_y < self.scroll_offset.1 {
            return None; // Before visible area (scrolled past)
        }
        
        if content_x >= self.scroll_offset.0 + viewable_width || 
           content_y >= self.scroll_offset.1 + viewable_height {
            return None; // After visible area
        }
        
        // Apply scroll offset to get screen-relative position
        let screen_relative_x = content_x - self.scroll_offset.0;
        let screen_relative_y = content_y - self.scroll_offset.1;
        
        // Convert to absolute screen coordinates
        let screen_x = self.content_bounds.x1 + screen_relative_x;
        let screen_y = self.content_bounds.y1 + screen_relative_y;
        
        Some((screen_x, screen_y))
    }
    
    /// Get the content coordinate range that's currently visible
    pub fn get_visible_content_range(&self) -> ((usize, usize), (usize, usize)) {
        let start_x = self.scroll_offset.0;
        let start_y = self.scroll_offset.1;
        let end_x = (start_x + self.content_bounds.width()).min(self.content_size.0);
        let end_y = (start_y + self.content_bounds.height()).min(self.content_size.1);
        
        ((start_x, start_y), (end_x, end_y))
    }
    
    /// Test if content coordinates are currently visible on screen
    pub fn is_content_coordinate_visible(&self, content_x: usize, content_y: usize) -> bool {
        let ((start_x, start_y), (end_x, end_y)) = self.get_visible_content_range();
        
        content_x >= start_x && content_x < end_x && content_y >= start_y && content_y < end_y
    }
    
    /// Calculate which content line a screen Y coordinate corresponds to
    /// Useful for text selection and choice navigation
    pub fn screen_y_to_content_line(&self, screen_y: usize) -> Option<usize> {
        if !self.is_within_content_bounds(0, screen_y) {
            return None;
        }
        
        let relative_y = screen_y.saturating_sub(self.content_bounds.y1);
        let content_line = relative_y + self.scroll_offset.1;
        
        if content_line >= self.content_size.1 {
            None
        } else {
            Some(content_line)
        }
    }
    
    /// Calculate which content column a screen X coordinate corresponds to  
    /// Useful for text cursor positioning
    pub fn screen_x_to_content_column(&self, screen_x: usize) -> Option<usize> {
        if !self.is_within_content_bounds(screen_x, 0) {
            return None;
        }
        
        let relative_x = screen_x.saturating_sub(self.content_bounds.x1);
        let content_column = relative_x + self.scroll_offset.0;
        
        if content_column >= self.content_size.0 {
            None
        } else {
            Some(content_column)
        }
    }
    
    /// Calculate click regions for UI components
    /// Centralizes region detection logic
    pub fn detect_click_region(&self, x: usize, y: usize) -> ClickRegion {
        if !self.is_within_screen_bounds(x, y) {
            return ClickRegion::Outside;
        }
        
        // Check if in content area
        if self.is_within_content_bounds(x, y) {
            return ClickRegion::Content;
        }
        
        // Check specific UI regions
        let bounds = &self.screen_bounds;
        
        // Scrollbar regions
        if x == bounds.x2 && y >= bounds.y1 && y <= bounds.y2 {
            return ClickRegion::VerticalScrollbar;
        }
        
        if y == bounds.y2 && x >= bounds.x1 && x <= bounds.x2 {
            return ClickRegion::HorizontalScrollbar;
        }
        
        // Tab bar region (top inside border)
        if y == bounds.y1 + 1 && x > bounds.x1 && x < bounds.x2 {
            return ClickRegion::TabBar;
        }
        
        // Title/border regions
        if y == bounds.y1 || y == bounds.y2 || x == bounds.x1 || x == bounds.x2 {
            return ClickRegion::Border;
        }
        
        ClickRegion::Content // Fallback
    }
    
    /// Calculate distance between two points (for drag detection)
    pub fn calculate_distance(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> f64 {
        let dx = (x2 as f64) - (x1 as f64);
        let dy = (y2 as f64) - (y1 as f64);
        (dx * dx + dy * dy).sqrt()
    }
    
    /// Test if mouse movement constitutes a drag (beyond threshold)
    pub fn is_drag_operation(&self, start_x: usize, start_y: usize, current_x: usize, current_y: usize, threshold: f64) -> bool {
        self.calculate_distance(start_x, start_y, current_x, current_y) > threshold
    }
    
    /// Calculate bounding box for a selection region
    pub fn calculate_selection_bounds(&self, start_x: usize, start_y: usize, end_x: usize, end_y: usize) -> SelectionBounds {
        let min_x = start_x.min(end_x);
        let max_x = start_x.max(end_x);
        let min_y = start_y.min(end_y);
        let max_y = start_y.max(end_y);
        
        SelectionBounds {
            start_x: min_x,
            start_y: min_y,
            end_x: max_x,
            end_y: max_y,
            width: max_x.saturating_sub(min_x) + 1,
            height: max_y.saturating_sub(min_y) + 1,
        }
    }
    
    /// Clamp coordinates to valid screen bounds
    /// Prevents out-of-bounds coordinate issues
    pub fn clamp_to_screen_bounds(&self, x: usize, y: usize) -> (usize, usize) {
        let clamped_x = x.clamp(self.screen_bounds.x1, self.screen_bounds.x2);
        let clamped_y = y.clamp(self.screen_bounds.y1, self.screen_bounds.y2);
        (clamped_x, clamped_y)
    }
    
    /// Clamp coordinates to valid content bounds
    pub fn clamp_to_content_bounds(&self, x: usize, y: usize) -> (usize, usize) {
        let clamped_x = x.clamp(self.content_bounds.x1, self.content_bounds.x2);
        let clamped_y = y.clamp(self.content_bounds.y1, self.content_bounds.y2);
        (clamped_x, clamped_y)
    }
    
    /// Calculate scroll delta needed to make content coordinate visible
    pub fn calculate_scroll_to_visible(&self, content_x: usize, content_y: usize) -> (i32, i32) {
        let mut delta_x = 0i32;
        let mut delta_y = 0i32;
        
        let viewable_width = self.content_bounds.width();
        let viewable_height = self.content_bounds.height();
        
        // Horizontal scrolling
        if content_x < self.scroll_offset.0 {
            // Scroll left
            delta_x = -((self.scroll_offset.0 - content_x) as i32);
        } else if content_x >= self.scroll_offset.0 + viewable_width {
            // Scroll right
            delta_x = (content_x - (self.scroll_offset.0 + viewable_width - 1)) as i32;
        }
        
        // Vertical scrolling
        if content_y < self.scroll_offset.1 {
            // Scroll up
            delta_y = -((self.scroll_offset.1 - content_y) as i32);
        } else if content_y >= self.scroll_offset.1 + viewable_height {
            // Scroll down
            delta_y = (content_y - (self.scroll_offset.1 + viewable_height - 1)) as i32;
        }
        
        (delta_x, delta_y)
    }
    
    /// Update mouse dimensions for window/content resize
    pub fn update_bounds(&mut self, screen_bounds: Bounds, content_bounds: Bounds) {
        self.screen_bounds = screen_bounds;
        self.content_bounds = content_bounds;
    }
    
    /// Update content size (when content changes)
    pub fn update_content_size(&mut self, width: usize, height: usize) {
        self.content_size = (width, height);
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ClickRegion {
    Content,
    Border,
    TabBar,
    VerticalScrollbar,
    HorizontalScrollbar,
    CloseButton,
    ResizeHandle,
    Outside,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SelectionBounds {
    pub start_x: usize,
    pub start_y: usize,
    pub end_x: usize,
    pub end_y: usize,
    pub width: usize,
    pub height: usize,
}

impl SelectionBounds {
    /// Test if a point is within the selection
    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.start_x && x <= self.end_x && y >= self.start_y && y <= self.end_y
    }
    
    /// Get all coordinates within the selection
    pub fn get_coordinates(&self) -> Vec<(usize, usize)> {
        let mut coords = Vec::new();
        for y in self.start_y..=self.end_y {
            for x in self.start_x..=self.end_x {
                coords.push((x, y));
            }
        }
        coords
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_mouse_dims() -> MouseDimensions {
        let screen_bounds = Bounds::new(0, 0, 20, 10); // 21x11
        let content_bounds = Bounds::new(2, 2, 18, 8); // 17x7 content area
        let scroll_offset = (5, 3); // Scrolled 5 right, 3 down
        let content_size = (50, 30); // Large content area
        
        MouseDimensions::new(screen_bounds, content_bounds, scroll_offset, content_size)
    }
    
    #[test]
    fn test_crossterm_coordinate_conversion() {
        let mouse_dims = create_test_mouse_dims();
        
        let (x, y) = mouse_dims.crossterm_to_internal(15, 7);
        assert_eq!(x, 15);
        assert_eq!(y, 7);
        
        let (col, row) = mouse_dims.internal_to_crossterm(15, 7);
        assert_eq!(col, 15);
        assert_eq!(row, 7);
    }
    
    #[test]
    fn test_bounds_checking() {
        let mouse_dims = create_test_mouse_dims();
        
        // Within screen bounds
        assert!(mouse_dims.is_within_screen_bounds(10, 5));
        
        // Outside screen bounds
        assert!(!mouse_dims.is_within_screen_bounds(25, 5));
        assert!(!mouse_dims.is_within_screen_bounds(10, 15));
        
        // Within content bounds
        assert!(mouse_dims.is_within_content_bounds(10, 5));
        
        // Outside content bounds (but within screen)
        assert!(!mouse_dims.is_within_content_bounds(1, 1)); // Border area
    }
    
    #[test]
    fn test_screen_to_content_translation() {
        let mouse_dims = create_test_mouse_dims();
        
        // Click at content area (10, 5) should translate to content coordinates
        if let Some((content_x, content_y)) = mouse_dims.screen_to_content(10, 5) {
            // Screen (10,5) - content_bounds.x1 (2) = 8, + scroll_offset.0 (5) = 13
            // Screen (5) - content_bounds.y1 (2) = 3, + scroll_offset.1 (3) = 6
            assert_eq!(content_x, 13);
            assert_eq!(content_y, 6);
        } else {
            panic!("Should successfully translate coordinates");
        }
        
        // Click outside content area should return None
        assert!(mouse_dims.screen_to_content(1, 1).is_none());
    }
    
    #[test]
    fn test_content_to_screen_translation() {
        let mouse_dims = create_test_mouse_dims();
        
        // Content coordinates (13, 6) should translate back to screen
        if let Some((screen_x, screen_y)) = mouse_dims.content_to_screen(13, 6) {
            // Content (13) - scroll_offset.0 (5) = 8, + content_bounds.x1 (2) = 10
            // Content (6) - scroll_offset.1 (3) = 3, + content_bounds.y1 (2) = 5
            assert_eq!(screen_x, 10);
            assert_eq!(screen_y, 5);
        } else {
            panic!("Should successfully translate back to screen coordinates");
        }
        
        // Content coordinates not currently visible should return None
        assert!(mouse_dims.content_to_screen(0, 0).is_none()); // Before scroll area
    }
    
    #[test]
    fn test_visible_content_range() {
        let mouse_dims = create_test_mouse_dims();
        
        let ((start_x, start_y), (end_x, end_y)) = mouse_dims.get_visible_content_range();
        
        // Should match scroll offset and content bounds size
        assert_eq!(start_x, 5); // scroll_offset.0
        assert_eq!(start_y, 3); // scroll_offset.1
        assert_eq!(end_x, 5 + 17); // start + content_bounds.width()
        assert_eq!(end_y, 3 + 7);  // start + content_bounds.height()
    }
    
    #[test]
    fn test_click_region_detection() {
        let mouse_dims = create_test_mouse_dims();
        
        // Content area
        assert_eq!(mouse_dims.detect_click_region(10, 5), ClickRegion::Content);
        
        // Border
        assert_eq!(mouse_dims.detect_click_region(0, 0), ClickRegion::Border);
        
        // Vertical scrollbar (right edge)
        assert_eq!(mouse_dims.detect_click_region(20, 5), ClickRegion::VerticalScrollbar);
        
        // Horizontal scrollbar (bottom edge)
        assert_eq!(mouse_dims.detect_click_region(10, 10), ClickRegion::HorizontalScrollbar);
        
        // Outside
        assert_eq!(mouse_dims.detect_click_region(25, 15), ClickRegion::Outside);
    }
    
    #[test]
    fn test_drag_detection() {
        let mouse_dims = create_test_mouse_dims();
        
        // Small movement - not a drag
        assert!(!mouse_dims.is_drag_operation(10, 5, 11, 6, 3.0));
        
        // Large movement - is a drag
        assert!(mouse_dims.is_drag_operation(10, 5, 15, 10, 3.0));
    }
    
    #[test]
    fn test_selection_bounds() {
        let mouse_dims = create_test_mouse_dims();
        
        let selection = mouse_dims.calculate_selection_bounds(5, 3, 10, 7);
        
        assert_eq!(selection.start_x, 5);
        assert_eq!(selection.start_y, 3);
        assert_eq!(selection.end_x, 10);
        assert_eq!(selection.end_y, 7);
        assert_eq!(selection.width, 6); // 10 - 5 + 1
        assert_eq!(selection.height, 5); // 7 - 3 + 1
        
        // Test contains
        assert!(selection.contains(7, 5));
        assert!(!selection.contains(2, 2));
        assert!(!selection.contains(12, 8));
    }
    
    #[test]
    fn test_coordinate_clamping() {
        let mouse_dims = create_test_mouse_dims();
        
        // Clamp to screen bounds
        let (x, y) = mouse_dims.clamp_to_screen_bounds(25, 15);
        assert_eq!(x, 20); // screen_bounds.x2
        assert_eq!(y, 10); // screen_bounds.y2
        
        // Clamp to content bounds
        let (x, y) = mouse_dims.clamp_to_content_bounds(25, 15);
        assert_eq!(x, 18); // content_bounds.x2
        assert_eq!(y, 8);  // content_bounds.y2
    }
    
    #[test]
    fn test_scroll_to_visible_calculation() {
        let mouse_dims = create_test_mouse_dims();
        
        // Content coordinate already visible - no scroll needed
        let (dx, dy) = mouse_dims.calculate_scroll_to_visible(10, 5);
        assert_eq!(dx, 0);
        assert_eq!(dy, 0);
        
        // Content coordinate to the left - need to scroll left
        let (dx, dy) = mouse_dims.calculate_scroll_to_visible(2, 5);
        assert!(dx < 0); // Scroll left
        
        // Content coordinate to the right - need to scroll right
        let (dx, dy) = mouse_dims.calculate_scroll_to_visible(25, 5);
        assert!(dx > 0); // Scroll right
    }
}