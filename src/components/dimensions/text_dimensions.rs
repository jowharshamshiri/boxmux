use super::FontMetrics;
use crate::Bounds;

/// TextDimensions - Centralizes ALL text layout and wrapping mathematical operations
///
/// Eliminates ad hoc text math like:
/// - width.saturating_sub(border_width)
/// - title_start_position = x1 + (width.saturating_sub(title_length)) / 2
/// - Text wrapping and line counting scattered throughout draw_utils.rs
/// - Character-aware truncation with ellipsis
///
/// Replaces scattered text layout logic from draw_utils.rs, components/text_content.rs
#[derive(Debug, Clone)]
pub struct TextDimensions {
    /// Available area for text rendering
    bounds: Bounds,
    /// Font metrics for calculations
    font_metrics: FontMetrics,
    /// Text alignment preferences
    alignment: TextAlignment,
    /// Whether to wrap text or truncate
    wrap_behavior: WrapBehavior,
}

impl TextDimensions {
    /// Create new text dimensions
    pub fn new(bounds: Bounds) -> Self {
        Self {
            bounds,
            font_metrics: FontMetrics::default(),
            alignment: TextAlignment::Left,
            wrap_behavior: WrapBehavior::Wrap,
        }
    }
    
    /// Builder methods
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }
    
    pub fn with_wrap_behavior(mut self, behavior: WrapBehavior) -> Self {
        self.wrap_behavior = behavior;
        self
    }
    
    pub fn with_font_metrics(mut self, metrics: FontMetrics) -> Self {
        self.font_metrics = metrics;
        self
    }
    
    /// Get available text width (accounting for UI chrome)
    pub fn available_width(&self) -> usize {
        self.bounds.width()
    }
    
    /// Get available text height (accounting for UI chrome)
    pub fn available_height(&self) -> usize {
        self.bounds.height()
    }
    
    /// Wrap text to fit within available width
    /// Centralizes text wrapping logic from draw_utils.rs
    pub fn wrap_text(&self, text: &str) -> Vec<String> {
        let max_width = self.available_width();
        
        if max_width == 0 {
            return vec![];
        }
        
        match self.wrap_behavior {
            WrapBehavior::Wrap => self.wrap_text_to_width(text, max_width),
            WrapBehavior::Truncate => {
                vec![self.truncate_with_ellipsis(text, max_width)]
            }
            WrapBehavior::Clip => {
                vec![text.chars().take(max_width).collect()]
            }
        }
    }
    
    /// Internal text wrapping implementation
    /// Handles word boundaries and preserves formatting
    fn wrap_text_to_width(&self, text: &str, width: usize) -> Vec<String> {
        if width == 0 {
            return vec![];
        }
        
        let mut lines = Vec::new();
        
        for line in text.lines() {
            if line.chars().count() <= width {
                lines.push(line.to_string());
                continue;
            }
            
            // Handle long lines that need wrapping
            let wrapped = self.wrap_long_line(line, width);
            lines.extend(wrapped);
        }
        
        lines
    }
    
    /// Wrap a single long line, respecting word boundaries when possible
    fn wrap_long_line(&self, line: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let mut current_line = String::new();
        let mut current_width = 0;
        
        for word in line.split_whitespace() {
            let word_width = word.chars().count();
            
            // If word alone is too long, break it up
            if word_width > width {
                // Finish current line if it has content
                if !current_line.is_empty() {
                    lines.push(current_line);
                    current_line = String::new();
                    current_width = 0;
                }
                
                // Break up the long word
                let broken_word = self.break_long_word(word, width);
                lines.extend(broken_word);
                continue;
            }
            
            // Check if adding word would exceed width
            let space_needed = if current_line.is_empty() { 0 } else { 1 }; // Space before word
            if current_width + space_needed + word_width > width {
                // Start new line with this word
                lines.push(current_line);
                current_line = word.to_string();
                current_width = word_width;
            } else {
                // Add word to current line
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += 1;
                }
                current_line.push_str(word);
                current_width += word_width;
            }
        }
        
        // Add final line if it has content
        if !current_line.is_empty() {
            lines.push(current_line);
        }
        
        lines
    }
    
    /// Break up a word that's too long for a single line
    fn break_long_word(&self, word: &str, width: usize) -> Vec<String> {
        let mut lines = Vec::new();
        let chars: Vec<char> = word.chars().collect();
        
        for chunk in chars.chunks(width) {
            lines.push(chunk.iter().collect());
        }
        
        lines
    }
    
    /// Truncate text with ellipsis if it exceeds width
    /// Centralizes ellipsis logic from various components
    pub fn truncate_with_ellipsis(&self, text: &str, max_width: usize) -> String {
        let char_count = text.chars().count();
        
        if char_count <= max_width {
            return text.to_string();
        }
        
        if max_width <= 1 {
            return if max_width == 1 { "…".to_string() } else { String::new() };
        }
        
        let truncate_at = max_width - 1; // Leave room for ellipsis
        let mut result: String = text.chars().take(truncate_at).collect();
        result.push('…');
        result
    }
    
    /// Calculate text bounds (width, height) for given text
    pub fn calculate_text_bounds(&self, text: &str) -> (usize, usize) {
        let wrapped_lines = self.wrap_text(text);
        let width = wrapped_lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let height = wrapped_lines.len();
        
        (width, height)
    }
    
    /// Calculate positioning for centered text within bounds
    /// Centralizes center positioning logic from draw_utils.rs
    pub fn center_text_in_bounds(&self, text: &str) -> TextLayout {
        let wrapped_lines = self.wrap_text(text);
        let text_width = wrapped_lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let text_height = wrapped_lines.len();
        
        let available_width = self.available_width();
        let available_height = self.available_height();
        
        // Calculate starting position for centering
        let start_x = if text_width < available_width {
            self.bounds.x1 + (available_width - text_width) / 2
        } else {
            self.bounds.x1
        };
        
        let start_y = if text_height < available_height {
            self.bounds.y1 + (available_height - text_height) / 2
        } else {
            self.bounds.y1
        };
        
        TextLayout {
            lines: wrapped_lines,
            start_x,
            start_y,
            text_width,
            text_height,
            alignment: self.alignment,
        }
    }
    
    /// Calculate text layout based on alignment
    pub fn layout_text(&self, text: &str) -> TextLayout {
        let wrapped_lines = self.wrap_text(text);
        let text_width = wrapped_lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let text_height = wrapped_lines.len();
        
        let (start_x, start_y) = match self.alignment {
            TextAlignment::Left => (self.bounds.x1, self.bounds.y1),
            TextAlignment::Center => {
                let x = self.bounds.x1 + 
                    (self.available_width().saturating_sub(text_width)) / 2;
                let y = self.bounds.y1 + 
                    (self.available_height().saturating_sub(text_height)) / 2;
                (x, y)
            }
            TextAlignment::Right => {
                let x = self.bounds.x2.saturating_sub(text_width);
                (x, self.bounds.y1)
            }
            TextAlignment::Justify => (self.bounds.x1, self.bounds.y1), // TODO: Implement justify
        };
        
        TextLayout {
            lines: wrapped_lines,
            start_x,
            start_y,
            text_width,
            text_height,
            alignment: self.alignment,
        }
    }
    
    /// Calculate line positions for each wrapped line
    pub fn calculate_line_positions(&self, layout: &TextLayout) -> Vec<LinePosition> {
        let mut positions = Vec::new();
        
        for (line_index, line) in layout.lines.iter().enumerate() {
            let line_width = line.chars().count();
            let y = layout.start_y + line_index;
            
            let x = match layout.alignment {
                TextAlignment::Left => layout.start_x,
                TextAlignment::Center => {
                    if line_width < self.available_width() {
                        self.bounds.x1 + (self.available_width() - line_width) / 2
                    } else {
                        self.bounds.x1
                    }
                }
                TextAlignment::Right => {
                    self.bounds.x2.saturating_sub(line_width)
                }
                TextAlignment::Justify => {
                    // TODO: Implement proper justification with word spacing
                    layout.start_x
                }
            };
            
            positions.push(LinePosition {
                x,
                y,
                width: line_width,
                content: line.clone(),
            });
        }
        
        positions
    }
    
    /// Count visible lines that fit within available height
    pub fn count_visible_lines(&self, text: &str) -> usize {
        let wrapped_lines = self.wrap_text(text);
        wrapped_lines.len().min(self.available_height())
    }
    
    /// Get visible text portion for scrolling
    /// Centralizes scrolling text logic
    pub fn get_visible_lines(&self, text: &str, scroll_offset: usize) -> Vec<String> {
        let wrapped_lines = self.wrap_text(text);
        let available_height = self.available_height();
        
        if scroll_offset >= wrapped_lines.len() {
            return vec![];
        }
        
        wrapped_lines
            .into_iter()
            .skip(scroll_offset)
            .take(available_height)
            .collect()
    }
    
    /// Calculate maximum scroll offset for text
    pub fn max_scroll_offset(&self, text: &str) -> usize {
        let wrapped_lines = self.wrap_text(text);
        let available_height = self.available_height();
        
        if wrapped_lines.len() <= available_height {
            0
        } else {
            wrapped_lines.len() - available_height
        }
    }
    
    /// Validate text fits within dimensions
    pub fn validate_text_fit(&self, text: &str) -> TextFitResult {
        let available_width = self.available_width();
        let available_height = self.available_height();
        
        // Calculate unwrapped text dimensions to check if it needs wrapping
        let unwrapped_lines: Vec<&str> = text.lines().collect();
        let unwrapped_width = unwrapped_lines
            .iter()
            .map(|line| line.chars().count())
            .max()
            .unwrap_or(0);
        let unwrapped_height = unwrapped_lines.len();
        
        // Check if original text fits without modification
        let fits_width = unwrapped_width <= available_width;
        let _fits_height = unwrapped_height <= available_height;
        
        // If text doesn't fit width-wise, check wrapped dimensions for height validation
        let final_height = if !fits_width {
            let wrapped_lines = self.wrap_text(text);
            wrapped_lines.len()
        } else {
            unwrapped_height
        };
        
        let fits_height_after_wrap = final_height <= available_height;
        
        TextFitResult {
            fits_width,
            fits_height: fits_height_after_wrap,
            text_width: unwrapped_width,
            text_height: final_height,
            available_width,
            available_height,
            requires_wrapping: !fits_width,
            requires_scrolling: !fits_height_after_wrap,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WrapBehavior {
    /// Wrap text to multiple lines
    Wrap,
    /// Truncate with ellipsis
    Truncate,
    /// Hard clip at boundary
    Clip,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextLayout {
    pub lines: Vec<String>,
    pub start_x: usize,
    pub start_y: usize,
    pub text_width: usize,
    pub text_height: usize,
    pub alignment: TextAlignment,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinePosition {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TextFitResult {
    pub fits_width: bool,
    pub fits_height: bool,
    pub text_width: usize,
    pub text_height: usize,
    pub available_width: usize,
    pub available_height: usize,
    pub requires_wrapping: bool,
    pub requires_scrolling: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_text_wrapping() {
        let bounds = Bounds::new(0, 0, 9, 4); // 10 chars wide, 5 lines high
        let text_dims = TextDimensions::new(bounds);
        
        let text = "This is a long line that should be wrapped";
        let wrapped = text_dims.wrap_text(text);
        
        // Should wrap into multiple lines
        assert!(wrapped.len() > 1);
        
        // Each line should fit within width
        for line in &wrapped {
            assert!(line.chars().count() <= 10);
        }
    }
    
    #[test]
    fn test_text_truncation() {
        let bounds = Bounds::new(0, 0, 9, 4); // 10 chars wide
        let text_dims = TextDimensions::new(bounds)
            .with_wrap_behavior(WrapBehavior::Truncate);
        
        let text = "This is a very long line that should be truncated";
        let wrapped = text_dims.wrap_text(text);
        
        // Should be single truncated line
        assert_eq!(wrapped.len(), 1);
        assert!(wrapped[0].ends_with('…'));
        assert!(wrapped[0].chars().count() <= 10);
    }
    
    #[test]
    fn test_center_alignment() {
        let bounds = Bounds::new(0, 0, 19, 4); // 20 chars wide, 5 lines high  
        let text_dims = TextDimensions::new(bounds)
            .with_alignment(TextAlignment::Center);
        
        let text = "Hello"; // 5 chars
        let layout = text_dims.center_text_in_bounds(text);
        
        // Should be centered: (20 - 5) / 2 = 7.5 ≈ 7
        assert_eq!(layout.start_x, 7);
    }
    
    #[test]
    fn test_text_bounds_calculation() {
        let bounds = Bounds::new(0, 0, 9, 9); // 10x10
        let text_dims = TextDimensions::new(bounds);
        
        let text = "Hello\nWorld\nTest";
        let (width, height) = text_dims.calculate_text_bounds(text);
        
        assert_eq!(width, 5); // "Hello" and "World" are 5 chars
        assert_eq!(height, 3); // 3 lines
    }
    
    #[test]
    fn test_visible_lines_with_scroll() {
        let bounds = Bounds::new(0, 0, 9, 2); // 3 lines visible
        let text_dims = TextDimensions::new(bounds);
        
        let text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5";
        
        // No scroll - should show first 3 lines
        let visible = text_dims.get_visible_lines(text, 0);
        assert_eq!(visible.len(), 3);
        assert_eq!(visible[0], "Line 1");
        assert_eq!(visible[2], "Line 3");
        
        // Scroll by 2 - should show lines 3-5
        let scrolled = text_dims.get_visible_lines(text, 2);
        assert_eq!(scrolled.len(), 3);
        assert_eq!(scrolled[0], "Line 3");
        assert_eq!(scrolled[2], "Line 5");
    }
    
    #[test]
    fn test_max_scroll_offset() {
        let bounds = Bounds::new(0, 0, 9, 2); // 3 lines visible
        let text_dims = TextDimensions::new(bounds);
        
        let text = "Line 1\nLine 2\nLine 3\nLine 4\nLine 5"; // 5 lines total
        let max_offset = text_dims.max_scroll_offset(text);
        
        // Should be 5 - 3 = 2 (can scroll down 2 lines)
        assert_eq!(max_offset, 2);
    }
    
    #[test]
    fn test_text_fit_validation() {
        let bounds = Bounds::new(0, 0, 9, 2); // 10 chars wide, 3 lines high
        let text_dims = TextDimensions::new(bounds);
        
        // Text that fits
        let short_text = "Hello";
        let fit_result = text_dims.validate_text_fit(short_text);
        assert!(fit_result.fits_width);
        assert!(fit_result.fits_height);
        assert!(!fit_result.requires_wrapping);
        assert!(!fit_result.requires_scrolling);
        
        // Text that needs wrapping
        let long_text = "This is a very long line";
        let wrap_result = text_dims.validate_text_fit(long_text);
        assert!(!wrap_result.fits_width);
        assert!(wrap_result.requires_wrapping);
    }
    
    #[test]
    fn test_ellipsis_truncation() {
        let bounds = Bounds::new(0, 0, 4, 0); // 5 chars wide
        let text_dims = TextDimensions::new(bounds);
        
        let text = "Hello World";
        let truncated = text_dims.truncate_with_ellipsis(text, 5);
        
        assert_eq!(truncated, "Hell…");
        assert_eq!(truncated.chars().count(), 5);
        
        // Edge case: width = 1
        let tiny = text_dims.truncate_with_ellipsis(text, 1);
        assert_eq!(tiny, "…");
    }
}