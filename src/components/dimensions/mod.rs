pub mod component_dimensions;
pub mod scroll_dimensions;
pub mod layout_dimensions;
pub mod text_dimensions;
pub mod mouse_dimensions;
pub mod progress_dimensions;

pub use component_dimensions::ComponentDimensions;
pub use scroll_dimensions::ScrollDimensions;
pub use layout_dimensions::LayoutDimensions;
pub use text_dimensions::TextDimensions;
pub use mouse_dimensions::MouseDimensions;
pub use progress_dimensions::ProgressDimensions;

/// Common types used across dimension classes
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Padding {
    pub top: usize,
    pub right: usize,
    pub bottom: usize,
    pub left: usize,
}

impl Padding {
    pub fn new(top: usize, right: usize, bottom: usize, left: usize) -> Self {
        Self { top, right, bottom, left }
    }
    
    pub fn uniform(padding: usize) -> Self {
        Self::new(padding, padding, padding, padding)
    }
    
    pub fn zero() -> Self {
        Self::uniform(0)
    }
    
    pub fn horizontal_total(&self) -> usize {
        self.left + self.right
    }
    
    pub fn vertical_total(&self) -> usize {
        self.top + self.bottom
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HitRegion {
    Content,
    Border,
    ScrollbarVertical,
    ScrollbarHorizontal,
    TabBar,
    CloseButton,
    Outside,
}

/// Font metrics for text calculations (expandable for future use)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FontMetrics {
    pub char_width: usize,
    pub line_height: usize,
}

impl Default for FontMetrics {
    fn default() -> Self {
        Self {
            char_width: 1, // Terminal characters
            line_height: 1,
        }
    }
}