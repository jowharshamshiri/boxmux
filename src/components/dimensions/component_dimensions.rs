use crate::Bounds;
use super::{HitRegion, Padding};

/// ComponentDimensions - Centralizes all UI component mathematical operations
/// 
/// Eliminates ad hoc math like:
/// - bounds.left() + 1, bounds.right() - 1 
/// - width.saturating_sub(2)
/// - (x1 + x2) / 2 for center calculations
/// - Hit testing and coordinate validation
///
/// All component positioning, sizing, and geometric operations centralized here.
#[derive(Debug, Clone)]
pub struct ComponentDimensions {
    bounds: Bounds,
    border_thickness: usize,
    padding: Padding,
    has_scrollbar_vertical: bool,
    has_scrollbar_horizontal: bool,
}

impl ComponentDimensions {
    /// Create new component dimensions
    pub fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            border_thickness: 1, // Default border thickness
            padding: Padding::uniform(1), // Default padding
            has_scrollbar_vertical: false,
            has_scrollbar_horizontal: false,
        }
    }
    
    /// Builder pattern for configuration
    pub fn with_border_thickness(mut self, thickness: usize) -> Self {
        self.border_thickness = thickness;
        self
    }
    
    pub fn with_padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }
    
    pub fn with_scrollbars(mut self, vertical: bool, horizontal: bool) -> Self {
        self.has_scrollbar_vertical = vertical;
        self.has_scrollbar_horizontal = horizontal;
        self
    }
    
    /// Get outer bounds (full component including border)
    pub fn outer_bounds(&self) -> Bounds {
        self.bounds
    }
    
    /// Get border area bounds (just the border region)
    pub fn border_bounds(&self) -> Bounds {
        self.bounds
    }
    
    /// Get content area bounds (inside border and padding)
    /// Replaces: bounds.left() + 2, bounds.right() - 2, etc.
    pub fn content_bounds(&self) -> Bounds {
        let border = self.border_thickness;
        let left_offset = border + self.padding.left;
        let right_offset = border + self.padding.right;
        let top_offset = border + self.padding.top;
        let bottom_offset = border + self.padding.bottom;
        
        // Handle scrollbar space
        let scrollbar_vertical_space = if self.has_scrollbar_vertical { 1 } else { 0 };
        let scrollbar_horizontal_space = if self.has_scrollbar_horizontal { 1 } else { 0 };
        
        Bounds::new(
            self.bounds.x1 + left_offset,
            self.bounds.y1 + top_offset,
            self.bounds.x2.saturating_sub(right_offset + scrollbar_vertical_space),
            self.bounds.y2.saturating_sub(bottom_offset + scrollbar_horizontal_space),
        )
    }
    
    /// Get center point of component
    /// Replaces: (x1 + x2) / 2, (y1 + y2) / 2
    pub fn center(&self) -> (usize, usize) {
        (
            (self.bounds.x1 + self.bounds.x2) / 2,
            (self.bounds.y1 + self.bounds.y2) / 2,
        )
    }
    
    /// Get content area center
    pub fn content_center(&self) -> (usize, usize) {
        let content = self.content_bounds();
        (
            (content.x1 + content.x2) / 2,
            (content.y1 + content.y2) / 2,
        )
    }
    
    /// Calculate available content width
    /// Replaces: width.saturating_sub(4), bounds.width() - borders
    pub fn content_width(&self) -> usize {
        let content = self.content_bounds();
        content.width()
    }
    
    /// Calculate available content height
    /// Replaces: height.saturating_sub(4), bounds.height() - borders
    pub fn content_height(&self) -> usize {
        let content = self.content_bounds();
        content.height()
    }
    
    /// Get border drawing coordinates for each side
    pub fn border_coordinates(&self) -> BorderCoordinates {
        BorderCoordinates {
            top: (self.bounds.x1, self.bounds.y1, self.bounds.x2, self.bounds.y1),
            bottom: (self.bounds.x1, self.bounds.y2, self.bounds.x2, self.bounds.y2),
            left: (self.bounds.x1, self.bounds.y1, self.bounds.x1, self.bounds.y2),
            right: (self.bounds.x2, self.bounds.y1, self.bounds.x2, self.bounds.y2),
        }
    }
    
    /// Calculate inside border coordinates (border + 1)
    /// Replaces: bounds.left() + 1, bounds.right() - 1, etc.
    pub fn inside_border_bounds(&self) -> Bounds {
        let thickness = self.border_thickness;
        Bounds::new(
            self.bounds.x1 + thickness,
            self.bounds.y1 + thickness,
            self.bounds.x2.saturating_sub(thickness),
            self.bounds.y2.saturating_sub(thickness),
        )
    }
    
    /// Hit test a point against component regions
    /// Centralizes all coordinate hit testing logic
    pub fn hit_test(&self, x: usize, y: usize) -> HitRegion {
        if !self.bounds.contains_point(x, y) {
            return HitRegion::Outside;
        }
        
        let content = self.content_bounds();
        if content.contains_point(x, y) {
            return HitRegion::Content;
        }
        
        // Check scrollbar regions
        if self.has_scrollbar_vertical && x == self.bounds.x2 {
            return HitRegion::ScrollbarVertical;
        }
        
        if self.has_scrollbar_horizontal && y == self.bounds.y2 {
            return HitRegion::ScrollbarHorizontal;
        }
        
        // Check tab bar region (top inside border)
        let inside = self.inside_border_bounds();
        if y == inside.y1 && x >= inside.x1 && x <= inside.x2 {
            return HitRegion::TabBar;
        }
        
        HitRegion::Border
    }
    
    /// Calculate title positioning within component
    /// Centralizes title positioning math from draw_utils.rs
    pub fn calculate_title_position(
        &self,
        title: &str,
        position: &str,
        padding: usize,
    ) -> TitleLayout {
        let width = self.bounds.width();
        let title_length = title.chars().count();
        let x1 = self.bounds.x1;
        let x2 = self.bounds.x2;
        
        let (title_start, line_before_length, line_after_length) = match position {
            "left" | "start" => {
                let title_start = x1 + padding;
                let line_before = title_start.saturating_sub(x1);
                let line_after = width.saturating_sub(line_before + title_length);
                (title_start, line_before, line_after)
            }
            "center" => {
                let title_start = x1 + (width.saturating_sub(title_length)) / 2;
                let line_before = title_start.saturating_sub(x1);
                let line_after = width.saturating_sub(line_before + title_length);
                (title_start, line_before, line_after)
            }
            "right" | "end" => {
                let title_start = x2.saturating_sub(title_length + padding);
                let line_before = title_start.saturating_sub(x1);
                let line_after = x2.saturating_sub(title_start + title_length);
                (title_start, line_before, line_after)
            }
            _ => {
                // Default to left
                let title_start = x1 + padding;
                let line_before = title_start.saturating_sub(x1);
                let line_after = width.saturating_sub(line_before + title_length);
                (title_start, line_before, line_after)
            }
        };
        
        TitleLayout {
            title_x: title_start,
            title_end_x: title_start + title_length - 1,
            line_before_length,
            line_after_length,
        }
    }
    
    /// Check if point is near corner (for resize detection)
    /// Replaces: x >= bounds.x2.saturating_sub(corner_tolerance) && x <= bounds.x2
    pub fn is_near_corner(&self, x: usize, y: usize, tolerance: usize) -> Option<Corner> {
        let bounds = &self.bounds;
        
        // Bottom-right corner
        if (x >= bounds.x2.saturating_sub(tolerance) && x <= bounds.x2)
            && (y >= bounds.y2.saturating_sub(tolerance) && y <= bounds.y2)
        {
            return Some(Corner::BottomRight);
        }
        
        // Top-right corner  
        if (x >= bounds.x2.saturating_sub(tolerance) && x <= bounds.x2)
            && (y >= bounds.y1 && y <= bounds.y1 + tolerance)
        {
            return Some(Corner::TopRight);
        }
        
        // Bottom-left corner
        if (x >= bounds.x1 && x <= bounds.x1 + tolerance)
            && (y >= bounds.y2.saturating_sub(tolerance) && y <= bounds.y2)
        {
            return Some(Corner::BottomLeft);
        }
        
        // Top-left corner
        if (x >= bounds.x1 && x <= bounds.x1 + tolerance)
            && (y >= bounds.y1 && y <= bounds.y1 + tolerance)
        {
            return Some(Corner::TopLeft);
        }
        
        None
    }
    
    /// Calculate space available for child components
    pub fn available_child_space(&self) -> (usize, usize) {
        let content = self.content_bounds();
        (content.width(), content.height())
    }
    
    /// Validate that dimensions are reasonable
    pub fn validate(&self) -> Result<(), ComponentDimensionError> {
        if self.bounds.width() < self.minimum_width() {
            return Err(ComponentDimensionError::TooNarrow {
                actual: self.bounds.width(),
                minimum: self.minimum_width(),
            });
        }
        
        if self.bounds.height() < self.minimum_height() {
            return Err(ComponentDimensionError::TooShort {
                actual: self.bounds.height(),
                minimum: self.minimum_height(),
            });
        }
        
        Ok(())
    }
    
    fn minimum_width(&self) -> usize {
        2 * self.border_thickness + self.padding.horizontal_total() + 1
    }
    
    fn minimum_height(&self) -> usize {
        2 * self.border_thickness + self.padding.vertical_total() + 1
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorderCoordinates {
    pub top: (usize, usize, usize, usize),    // (x1, y1, x2, y2)
    pub bottom: (usize, usize, usize, usize),
    pub left: (usize, usize, usize, usize),
    pub right: (usize, usize, usize, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TitleLayout {
    pub title_x: usize,
    pub title_end_x: usize,
    pub line_before_length: usize,
    pub line_after_length: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Corner {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentDimensionError {
    TooNarrow { actual: usize, minimum: usize },
    TooShort { actual: usize, minimum: usize },
    InvalidPadding,
    InvalidBorder,
}

impl ComponentDimensions {
    /// Get vertical scrollbar track bounds
    /// Replaces: bounds.top() + 1, bounds.bottom() - 1 
    pub fn vertical_scrollbar_track_bounds(&self) -> Bounds {
        let x = self.bounds.right();
        let start_y = self.bounds.top() + 1;
        let end_y = self.bounds.bottom().saturating_sub(1);
        Bounds::new(x, start_y, x, end_y)
    }

    /// Get horizontal scrollbar track bounds  
    /// Replaces: bounds.left() + 1, bounds.right() - 1
    pub fn horizontal_scrollbar_track_bounds(&self) -> Bounds {
        let y = self.bounds.bottom();
        let start_x = self.bounds.left() + 1;
        let end_x = self.bounds.right().saturating_sub(1);
        Bounds::new(start_x, y, end_x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_content_bounds_calculation() {
        let bounds = Bounds::new(0, 0, 10, 5);
        let dims = ComponentDimensions::new(bounds)
            .with_border_thickness(1)
            .with_padding(Padding::uniform(1));
        
        let content = dims.content_bounds();
        assert_eq!(content.x1, 2); // border + padding
        assert_eq!(content.y1, 2);
        assert_eq!(content.x2, 8); // 10 - border - padding
        assert_eq!(content.y2, 3); // 5 - border - padding
    }
    
    #[test]
    fn test_center_calculation() {
        let bounds = Bounds::new(0, 0, 10, 4);
        let dims = ComponentDimensions::new(bounds);
        
        let (cx, cy) = dims.center();
        assert_eq!(cx, 5); // (0 + 10) / 2
        assert_eq!(cy, 2); // (0 + 4) / 2
    }
    
    #[test]
    fn test_hit_testing() {
        let bounds = Bounds::new(0, 0, 10, 5);
        let dims = ComponentDimensions::new(bounds);
        
        // Outside
        assert_eq!(dims.hit_test(15, 15), HitRegion::Outside);
        
        // Content area
        let content = dims.content_bounds();
        assert_eq!(dims.hit_test(content.x1, content.y1), HitRegion::Content);
        
        // Border
        assert_eq!(dims.hit_test(0, 0), HitRegion::Border);
    }
    
    #[test] 
    fn test_title_positioning() {
        let bounds = Bounds::new(0, 0, 20, 5);
        let dims = ComponentDimensions::new(bounds);
        
        let layout = dims.calculate_title_position("Test", "center", 2);
        
        // Title should be centered: width=21, title=4, so start at (21-4)/2 = 8.5 ≈ 8
        assert_eq!(layout.title_x, 8);
        assert_eq!(layout.title_end_x, 11); // 8 + 4 - 1
    }

    #[test]
    fn test_scrollbar_track_bounds() {
        let bounds = Bounds::new(0, 0, 20, 10);
        let dims = ComponentDimensions::new(bounds);
        
        // Vertical scrollbar should be at right edge
        let v_track = dims.vertical_scrollbar_track_bounds();
        assert_eq!(v_track.x1, 20); // right edge
        assert_eq!(v_track.y1, 1);  // top + 1
        assert_eq!(v_track.y2, 9);  // bottom - 1
        
        // Horizontal scrollbar should be at bottom edge  
        let h_track = dims.horizontal_scrollbar_track_bounds();
        assert_eq!(h_track.y1, 10); // bottom edge
        assert_eq!(h_track.x1, 1);  // left + 1
        assert_eq!(h_track.x2, 19); // right - 1
    }
}