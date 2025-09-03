use crate::{ScreenBuffer, Bounds};

/// Universal event system for RenderableContent interactions
/// Replaces direct method calls with flexible event-based architecture
#[derive(Debug, Clone)]
pub struct ContentEvent {
    /// Type of event that occurred
    pub event_type: EventType,
    /// Screen coordinates where event occurred (relative to content area)
    pub position: Option<(usize, usize)>,
    /// Identifier of the content zone that was targeted
    pub zone_id: Option<String>,
    /// Additional event-specific data
    pub data: EventData,
    /// Timestamp when event occurred
    pub timestamp: std::time::SystemTime,
}

/// Types of events that can occur on content
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    /// Mouse click event
    Click,
    /// Mouse double-click event
    DoubleClick,
    /// Mouse hover event (entering/leaving clickable zones)
    Hover,
    /// Mouse move event (continuous movement tracking)
    MouseMove,
    /// Keyboard key press event
    KeyPress,
    /// Focus gained event
    Focus,
    /// Focus lost event
    Blur,
    /// Content scroll event
    Scroll,
    /// Content area resize event
    Resize,
    /// Box resize event (interactive resizing)
    BoxResize,
    /// Box title change event
    TitleChange,
    /// Custom application-specific event
    Custom(String),
}

/// Event-specific data payload
#[derive(Debug, Clone)]
pub struct EventData {
    /// Mouse button for click events
    pub mouse_button: Option<MouseButton>,
    /// Key information for keyboard events
    pub key: Option<KeyInfo>,
    /// Scroll direction and amount
    pub scroll: Option<ScrollInfo>,
    /// Size information for resize events
    pub size: Option<(usize, usize)>,
    /// Mouse movement information
    pub mouse_move: Option<MouseMoveInfo>,
    /// Hover state information
    pub hover: Option<HoverInfo>,
    /// Box resize information
    pub box_resize: Option<BoxResizeInfo>,
    /// Title change information
    pub title_change: Option<TitleChangeInfo>,
    /// Custom event data
    pub custom_data: Option<String>,
}

/// Mouse button types
#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    WheelUp,
    WheelDown,
}

/// Keyboard key information
#[derive(Debug, Clone)]
pub struct KeyInfo {
    /// Key code or character
    pub key: String,
    /// Modifier keys pressed
    pub modifiers: Vec<KeyModifier>,
}

/// Keyboard modifier keys
#[derive(Debug, Clone, PartialEq)]
pub enum KeyModifier {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

/// Scroll event information
#[derive(Debug, Clone)]
pub struct ScrollInfo {
    /// Scroll direction
    pub direction: ScrollDirection,
    /// Amount to scroll
    pub amount: i32,
}

/// Scroll directions
#[derive(Debug, Clone, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Mouse movement information
#[derive(Debug, Clone)]
pub struct MouseMoveInfo {
    /// Previous mouse position (if available)
    pub from_position: Option<(usize, usize)>,
    /// Current mouse position
    pub to_position: (usize, usize),
    /// Movement delta (dx, dy)
    pub delta: (i32, i32),
    /// Whether mouse is being dragged (button held)
    pub is_dragging: bool,
    /// Button being held during drag (if dragging)
    pub drag_button: Option<MouseButton>,
}

/// Hover state information
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// Whether entering or leaving hover state
    pub state: HoverState,
    /// Previous zone that was hovered (for leave events)
    pub previous_zone: Option<String>,
    /// Current zone being hovered (for enter events)
    pub current_zone: Option<String>,
    /// Duration mouse has been hovering (for tooltip timing)
    pub hover_duration: Option<std::time::Duration>,
}

/// Hover state transitions
#[derive(Debug, Clone, PartialEq)]
pub enum HoverState {
    /// Mouse entered clickable zone
    Enter,
    /// Mouse left clickable zone  
    Leave,
    /// Mouse moved within same zone
    Move,
}

/// Box resize information
#[derive(Debug, Clone)]
pub struct BoxResizeInfo {
    /// Type of resize operation
    pub resize_type: BoxResizeType,
    /// Original box bounds before resize
    pub original_bounds: (usize, usize, usize, usize), // x, y, width, height
    /// New box bounds after resize
    pub new_bounds: (usize, usize, usize, usize), // x, y, width, height
    /// Resize anchor point (corner/edge being dragged)
    pub anchor: ResizeAnchor,
    /// Whether resize is in progress or completed
    pub state: ResizeState,
}

/// Types of box resize operations
#[derive(Debug, Clone, PartialEq)]
pub enum BoxResizeType {
    /// Interactive resize via mouse drag
    Interactive,
    /// Programmatic resize via API
    Programmatic,
    /// Resize due to terminal window changes
    Terminal,
}

/// Resize anchor points
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeAnchor {
    /// Top-left corner
    TopLeft,
    /// Top-right corner
    TopRight,
    /// Bottom-left corner
    BottomLeft,
    /// Bottom-right corner
    BottomRight,
    /// Top edge
    Top,
    /// Bottom edge
    Bottom,
    /// Left edge
    Left,
    /// Right edge
    Right,
}

/// Resize operation state
#[derive(Debug, Clone, PartialEq)]
pub enum ResizeState {
    /// Resize operation started
    Started,
    /// Resize in progress
    InProgress,
    /// Resize completed
    Completed,
    /// Resize cancelled
    Cancelled,
}

/// Title change information
#[derive(Debug, Clone)]
pub struct TitleChangeInfo {
    /// Previous title (if any)
    pub old_title: Option<String>,
    /// New title
    pub new_title: String,
    /// Source of title change
    pub source: TitleChangeSource,
    /// Whether change should be persisted to YAML
    pub persist: bool,
}

/// Sources of title changes
#[derive(Debug, Clone, PartialEq)]
pub enum TitleChangeSource {
    /// User interaction (editing)
    User,
    /// PTY program (terminal title sequences)
    PTY,
    /// Script execution result
    Script,
    /// API call
    API,
    /// System event
    System,
}

/// Result of event handling
#[derive(Debug, Clone, PartialEq)]
pub enum EventResult {
    /// Event was handled and should not propagate
    Handled,
    /// Event was not handled, continue propagation
    NotHandled,
    /// Event was handled but should continue propagation
    HandledContinue,
    /// Event caused a state change that requires re-rendering
    StateChanged,
}

impl ContentEvent {
    /// Create a new content event with current timestamp
    pub fn new(event_type: EventType, position: Option<(usize, usize)>, zone_id: Option<String>) -> Self {
        Self {
            event_type,
            position,
            zone_id,
            data: EventData::default(),
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Create a click event
    pub fn new_click(position: Option<(usize, usize)>, zone_id: Option<String>) -> Self {
        let mut event = Self::new(EventType::Click, position, zone_id);
        event.data.mouse_button = Some(MouseButton::Left);
        event
    }

    /// Create a click event with specific mouse button
    pub fn new_click_with_button(position: Option<(usize, usize)>, zone_id: Option<String>, button: MouseButton) -> Self {
        let mut event = Self::new(EventType::Click, position, zone_id);
        event.data.mouse_button = Some(button);
        event
    }

    /// Create a hover event
    pub fn new_hover(position: (usize, usize), zone_id: Option<String>) -> Self {
        Self::new(EventType::Hover, Some(position), zone_id)
    }

    /// Create a key press event
    pub fn new_keypress(key: String, modifiers: Vec<KeyModifier>, zone_id: Option<String>) -> Self {
        let mut event = Self::new(EventType::KeyPress, None, zone_id);
        event.data.key = Some(KeyInfo { key, modifiers });
        event
    }

    /// Create a scroll event
    pub fn new_scroll(direction: ScrollDirection, amount: i32, zone_id: Option<String>) -> Self {
        let mut event = Self::new(EventType::Scroll, None, zone_id);
        event.data.scroll = Some(ScrollInfo { direction, amount });
        event
    }

    /// Create a focus event
    pub fn new_focus(zone_id: Option<String>) -> Self {
        Self::new(EventType::Focus, None, zone_id)
    }

    /// Create a blur event
    pub fn new_blur(zone_id: Option<String>) -> Self {
        Self::new(EventType::Blur, None, zone_id)
    }

    /// Create a resize event
    pub fn new_resize(new_size: (usize, usize)) -> Self {
        let mut event = Self::new(EventType::Resize, None, None);
        event.data.size = Some(new_size);
        event
    }

    /// Create a custom event
    pub fn new_custom(name: String, data: Option<String>, zone_id: Option<String>) -> Self {
        let mut event = Self::new(EventType::Custom(name), None, zone_id);
        event.data.custom_data = data;
        event
    }

    /// Create a mouse move event
    pub fn new_mouse_move(from_pos: Option<(usize, usize)>, to_pos: (usize, usize), zone_id: Option<String>) -> Self {
        let mut event = Self::new(EventType::MouseMove, Some(to_pos), zone_id);
        let delta = if let Some(from) = from_pos {
            (to_pos.0 as i32 - from.0 as i32, to_pos.1 as i32 - from.1 as i32)
        } else {
            (0, 0)
        };
        event.data.mouse_move = Some(MouseMoveInfo {
            from_position: from_pos,
            to_position: to_pos,
            delta,
            is_dragging: false,
            drag_button: None,
        });
        event
    }

    /// Create a mouse drag event
    pub fn new_mouse_drag(from_pos: (usize, usize), to_pos: (usize, usize), button: MouseButton, zone_id: Option<String>) -> Self {
        let mut event = Self::new(EventType::MouseMove, Some(to_pos), zone_id);
        let delta = (to_pos.0 as i32 - from_pos.0 as i32, to_pos.1 as i32 - from_pos.1 as i32);
        event.data.mouse_move = Some(MouseMoveInfo {
            from_position: Some(from_pos),
            to_position: to_pos,
            delta,
            is_dragging: true,
            drag_button: Some(button),
        });
        event
    }

    /// Create a hover enter event
    pub fn new_hover_enter(position: (usize, usize), zone_id: String, previous_zone: Option<String>) -> Self {
        let mut event = Self::new(EventType::Hover, Some(position), Some(zone_id.clone()));
        event.data.hover = Some(HoverInfo {
            state: HoverState::Enter,
            previous_zone,
            current_zone: Some(zone_id),
            hover_duration: None,
        });
        event
    }

    /// Create a hover leave event
    pub fn new_hover_leave(position: (usize, usize), zone_id: String, new_zone: Option<String>) -> Self {
        let mut event = Self::new(EventType::Hover, Some(position), Some(zone_id.clone()));
        event.data.hover = Some(HoverInfo {
            state: HoverState::Leave,
            previous_zone: Some(zone_id),
            current_zone: new_zone,
            hover_duration: None,
        });
        event
    }

    /// Create a hover move event (within same zone)
    pub fn new_hover_move(position: (usize, usize), zone_id: String, duration: std::time::Duration) -> Self {
        let mut event = Self::new(EventType::Hover, Some(position), Some(zone_id.clone()));
        event.data.hover = Some(HoverInfo {
            state: HoverState::Move,
            previous_zone: None,
            current_zone: Some(zone_id),
            hover_duration: Some(duration),
        });
        event
    }

    /// Create a box resize event
    pub fn new_box_resize(
        resize_type: BoxResizeType,
        original_bounds: (usize, usize, usize, usize),
        new_bounds: (usize, usize, usize, usize),
        anchor: ResizeAnchor,
        state: ResizeState,
    ) -> Self {
        let mut event = Self::new(EventType::BoxResize, None, None);
        event.data.box_resize = Some(BoxResizeInfo {
            resize_type,
            original_bounds,
            new_bounds,
            anchor,
            state,
        });
        event
    }

    /// Create a title change event
    pub fn new_title_change(
        old_title: Option<String>,
        new_title: String,
        source: TitleChangeSource,
        persist: bool,
    ) -> Self {
        let mut event = Self::new(EventType::TitleChange, None, None);
        event.data.title_change = Some(TitleChangeInfo {
            old_title,
            new_title,
            source,
            persist,
        });
        event
    }

    /// Get the mouse button for click events
    pub fn mouse_button(&self) -> Option<&MouseButton> {
        self.data.mouse_button.as_ref()
    }

    /// Get the key information for key events
    pub fn key_info(&self) -> Option<&KeyInfo> {
        self.data.key.as_ref()
    }

    /// Get the scroll information for scroll events
    pub fn scroll_info(&self) -> Option<&ScrollInfo> {
        self.data.scroll.as_ref()
    }

    /// Check if this is a click event
    pub fn is_click(&self) -> bool {
        self.event_type == EventType::Click
    }

    /// Check if this is a double-click event
    pub fn is_double_click(&self) -> bool {
        self.event_type == EventType::DoubleClick
    }

    /// Check if this is a keyboard event
    pub fn is_keyboard(&self) -> bool {
        self.event_type == EventType::KeyPress
    }

    /// Check if this is a mouse move event
    pub fn is_mouse_move(&self) -> bool {
        self.event_type == EventType::MouseMove
    }

    /// Check if this is a box resize event
    pub fn is_box_resize(&self) -> bool {
        self.event_type == EventType::BoxResize
    }

    /// Check if this is a title change event
    pub fn is_title_change(&self) -> bool {
        self.event_type == EventType::TitleChange
    }

    /// Get the mouse move information
    pub fn mouse_move_info(&self) -> Option<&MouseMoveInfo> {
        self.data.mouse_move.as_ref()
    }

    /// Get the hover information
    pub fn hover_info(&self) -> Option<&HoverInfo> {
        self.data.hover.as_ref()
    }

    /// Get the box resize information
    pub fn box_resize_info(&self) -> Option<&BoxResizeInfo> {
        self.data.box_resize.as_ref()
    }

    /// Get the title change information
    pub fn title_change_info(&self) -> Option<&TitleChangeInfo> {
        self.data.title_change.as_ref()
    }

    /// Check if this is a drag event
    pub fn is_drag(&self) -> bool {
        if let Some(mouse_move) = &self.data.mouse_move {
            mouse_move.is_dragging
        } else {
            false
        }
    }

    /// Check if this is a hover enter event
    pub fn is_hover_enter(&self) -> bool {
        if let Some(hover) = &self.data.hover {
            hover.state == HoverState::Enter
        } else {
            false
        }
    }

    /// Check if this is a hover leave event
    pub fn is_hover_leave(&self) -> bool {
        if let Some(hover) = &self.data.hover {
            hover.state == HoverState::Leave
        } else {
            false
        }
    }

    /// Get the movement delta for mouse move/drag events
    pub fn movement_delta(&self) -> Option<(i32, i32)> {
        self.data.mouse_move.as_ref().map(|m| m.delta)
    }
}

impl Default for EventData {
    fn default() -> Self {
        Self {
            mouse_button: None,
            key: None,
            scroll: None,
            size: None,
            mouse_move: None,
            hover: None,
            box_resize: None,
            title_change: None,
            custom_data: None,
        }
    }
}

/// Universal content interface for all content types in BoxMux
/// Enables unified overflow handling for text, choices, charts, and any future content types
pub trait RenderableContent {
    /// Get the raw content dimensions before any transformations
    /// Returns (width, height) in character cells
    fn get_dimensions(&self) -> (usize, usize);

    /// Get the raw content as a string - for backward compatibility with existing text rendering
    fn get_raw_content(&self) -> String;
    
    /// Get clickable zones in box-relative coordinates (0,0 = top-left of content area, ignoring borders)
    /// BoxRenderer will translate these to absolute screen coordinates accounting for scroll/wrap/centering
    fn get_box_relative_clickable_zones(&self) -> Vec<ClickableZone>;
    
    /// Handle a content event (click, hover, keypress, etc.)
    /// Event contains zone_id, position, and event-specific data
    /// Returns EventResult indicating how the event was processed
    fn handle_event(&mut self, event: &ContentEvent) -> EventResult;


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