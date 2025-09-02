use crate::{ScreenBuffer, Bounds};

/// Universal content interface for all content types in BoxMux
/// Enables unified overflow handling for text, choices, charts, and any future content types
pub trait RenderableContent {
    /// Get the raw content dimensions before any transformations
    /// Returns (width, height) in character cells
    fn get_dimensions(&self) -> (usize, usize);

    /// Render content within a viewport with scroll offsets
    /// - bounds: The available rendering area
    /// - x_offset: Horizontal scroll offset (characters to skip from left)
    /// - y_offset: Vertical scroll offset (lines to skip from top)
    /// - buffer: Target screen buffer for rendering
    fn render_viewport(&self, bounds: &Bounds, x_offset: usize, y_offset: usize, buffer: &mut ScreenBuffer);

    /// Get clickable zones for this content accounting for scroll offsets
    /// Returns all clickable areas mapped to content items
    fn get_clickable_zones(&self, bounds: &Bounds, x_offset: usize, y_offset: usize) -> Vec<ClickableZone>;

    /// Get dimensions after applying wrapping transformation
    /// Used for wrap overflow behavior - content is transformed to fit width
    fn get_wrapped_dimensions(&self, max_width: usize) -> (usize, usize);

    /// Render wrapped content within viewport
    /// Content is pre-wrapped to fit max_width, then rendered with scroll offsets
    fn render_wrapped_viewport(&self, bounds: &Bounds, max_width: usize, y_offset: usize, buffer: &mut ScreenBuffer);

    /// Get clickable zones for wrapped content
    /// Wrapped content may have multiple zones per item (multi-line choices)
    fn get_wrapped_clickable_zones(&self, bounds: &Bounds, max_width: usize, y_offset: usize) -> Vec<ClickableZone>;
}

/// Clickable zone representing an interactive area on screen
#[derive(Debug, Clone)]
pub struct ClickableZone {
    /// Screen rectangle bounds for this clickable area
    pub bounds: Bounds,
    /// Identifier for the content item (choice ID, link target, etc.)
    pub content_id: String,
    /// Type of content this zone represents
    pub content_type: ContentType,
    /// Additional metadata for this clickable zone
    pub metadata: ClickableMetadata,
}

/// Type of clickable content
#[derive(Debug, Clone, PartialEq)]
pub enum ContentType {
    /// Menu choice item
    Choice,
    /// Text link or hyperlink
    Link,
    /// Interactive button
    Button,
    /// Chart element (bar, line, data point)
    ChartElement,
    /// Tab in tab bar
    Tab,
    /// Generic interactive element
    Interactive,
}

/// Additional metadata for clickable zones
#[derive(Debug, Clone, Default)]
pub struct ClickableMetadata {
    /// Display text for this zone
    pub display_text: Option<String>,
    /// Tooltip or help text
    pub tooltip: Option<String>,
    /// Whether this zone is currently selected/focused
    pub selected: bool,
    /// Whether this zone is enabled for interaction
    pub enabled: bool,
    /// Line number within the original content (for wrapped content)
    pub original_line: Option<usize>,
    /// Character range within the original line
    pub char_range: Option<(usize, usize)>,
}

impl ClickableZone {
    /// Create a new clickable zone
    pub fn new(bounds: Bounds, content_id: String, content_type: ContentType) -> Self {
        Self {
            bounds,
            content_id,
            content_type,
            metadata: ClickableMetadata::default(),
        }
    }

    /// Create a clickable zone with metadata
    pub fn with_metadata(bounds: Bounds, content_id: String, content_type: ContentType, metadata: ClickableMetadata) -> Self {
        Self {
            bounds,
            content_id,
            content_type,
            metadata,
        }
    }

    /// Check if a point is within this clickable zone
    pub fn contains(&self, x: usize, y: usize) -> bool {
        self.bounds.contains(x, y)
    }

    /// Get the display text for this zone
    pub fn display_text(&self) -> String {
        self.metadata.display_text.clone()
            .unwrap_or_else(|| self.content_id.clone())
    }
}

/// Content dimensions with scroll requirements
#[derive(Debug, Clone)]
pub struct ContentDimensions {
    /// Total content width in characters
    pub width: usize,
    /// Total content height in lines
    pub height: usize,
    /// Whether horizontal scrolling is needed
    pub needs_horizontal_scroll: bool,
    /// Whether vertical scrolling is needed
    pub needs_vertical_scroll: bool,
}

impl ContentDimensions {
    /// Create content dimensions
    pub fn new(width: usize, height: usize, viewport_width: usize, viewport_height: usize) -> Self {
        Self {
            width,
            height,
            needs_horizontal_scroll: width > viewport_width,
            needs_vertical_scroll: height > viewport_height,
        }
    }

    /// Check if any scrolling is needed
    pub fn needs_scrolling(&self) -> bool {
        self.needs_horizontal_scroll || self.needs_vertical_scroll
    }
}