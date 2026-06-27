use crate::Bounds;

/// Enhanced sensitive zone for standalone draw loop management
/// Combines interaction zones with immediate visual state and action handling
#[derive(Debug, Clone)]
pub struct SensitiveZone {
    /// Unique identifier for this sensitive zone
    pub id: String,
    /// Screen rectangle bounds for this sensitive area
    pub bounds: Bounds,
    /// Display text for immediate redrawing on hover
    pub display_text: String,
    /// Action string to be sent when this zone is triggered
    pub action: String,
    /// Current visual state of this zone
    pub state: SensitiveZoneState,
    /// Normal visual styling when not interacting
    pub normal_style: ZoneStyle,
    /// Hover visual styling when mouse is over the zone
    pub hover_style: ZoneStyle,
    /// Selected visual styling when zone is selected/focused
    pub selected_style: ZoneStyle,
    /// Type of content this zone represents
    pub content_type: ContentType,
    /// Additional metadata for this sensitive zone
    pub metadata: SensitiveMetadata,
}

/// Current state of a sensitive zone
#[derive(Debug, Clone, PartialEq)]
pub enum SensitiveZoneState {
    /// Normal state - no interaction
    Normal,
    /// Mouse is hovering over this zone
    Hovered,
    /// Zone is selected/focused
    Selected,
    /// Zone is both selected and hovered
    SelectedHovered,
    /// Zone is disabled
    Disabled,
}

/// Visual styling for a sensitive zone in different states
#[derive(Debug, Clone)]
pub struct ZoneStyle {
    /// Foreground color (text color)
    pub fg_color: Option<(u8, u8, u8)>,
    /// Background color
    pub bg_color: Option<(u8, u8, u8)>,
    /// Whether text should be bold
    pub bold: bool,
    /// Whether text should be underlined
    pub underline: bool,
    /// Whether text should be italic
    pub italic: bool,
}

/// Type of sensitive content
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
    /// Scrollbar component
    Scrollbar,
    /// Generic interactive element
    Interactive,
}

/// Additional metadata for sensitive zones
#[derive(Debug, Clone, Default)]
pub struct SensitiveMetadata {
    /// Display text for this zone
    pub display_text: Option<String>,
    /// Tooltip or help text
    pub tooltip: Option<String>,
    /// Whether this zone is enabled for interaction
    pub enabled: bool,
    /// Line number within the original content (for wrapped content)
    pub original_line: Option<usize>,
    /// Character range within the original line
    pub char_range: Option<(usize, usize)>,
    /// Additional zone-specific data
    pub custom_data: Option<String>,
}

impl SensitiveZone {
    /// Create a new sensitive zone with default styling
    pub fn new(
        id: String,
        bounds: Bounds,
        display_text: String,
        action: String,
        content_type: ContentType,
    ) -> Self {
        Self {
            id,
            bounds,
            display_text,
            action,
            state: SensitiveZoneState::Normal,
            normal_style: ZoneStyle::default(),
            hover_style: ZoneStyle::default_hover(),
            selected_style: ZoneStyle::default_selected(),
            content_type,
            metadata: SensitiveMetadata::default(),
        }
    }

    /// Create a choice zone with custom styling
    pub fn new_choice(
        id: String,
        bounds: Bounds,
        display_text: String,
        action: String,
        normal_fg: Option<(u8, u8, u8)>,
        normal_bg: Option<(u8, u8, u8)>,
        hover_fg: Option<(u8, u8, u8)>,
        hover_bg: Option<(u8, u8, u8)>,
    ) -> Self {
        let mut zone = Self::new(id, bounds, display_text.clone(), action, ContentType::Choice);
        zone.normal_style.fg_color = normal_fg;
        zone.normal_style.bg_color = normal_bg;
        zone.hover_style.fg_color = hover_fg;
        zone.hover_style.bg_color = hover_bg;
        zone.metadata.display_text = Some(display_text);
        zone
    }

    /// Check if a screen position is within this zone
    pub fn contains_point(&self, x: usize, y: usize) -> bool {
        x >= self.bounds.left() && 
        x <= self.bounds.right() &&
        y >= self.bounds.top() &&
        y <= self.bounds.bottom()
    }

    /// Update the zone's state
    pub fn set_state(&mut self, new_state: SensitiveZoneState) {
        self.state = new_state;
    }

    /// Get the current style based on state
    pub fn current_style(&self) -> &ZoneStyle {
        match self.state {
            SensitiveZoneState::Normal => &self.normal_style,
            SensitiveZoneState::Hovered => &self.hover_style,
            SensitiveZoneState::Selected => &self.selected_style,
            SensitiveZoneState::SelectedHovered => &self.hover_style, // Hover takes precedence
            SensitiveZoneState::Disabled => &self.normal_style, // Use normal style for disabled
        }
    }

    /// Check if this zone is interactive (enabled and not disabled)
    pub fn is_interactive(&self) -> bool {
        self.metadata.enabled && self.state != SensitiveZoneState::Disabled
    }

    /// Update the display text and trigger a redraw flag
    pub fn update_display_text(&mut self, new_text: String) {
        self.display_text = new_text.clone();
        self.metadata.display_text = Some(new_text);
    }

    /// Update the bounds (for dynamic layouts or resizing)
    pub fn update_bounds(&mut self, new_bounds: Bounds) {
        self.bounds = new_bounds;
    }

    /// Check if this zone needs immediate redraw (state changed)
    pub fn needs_redraw(&self) -> bool {
        // For now, always assume redraw needed when state is hovered or selected
        matches!(self.state, SensitiveZoneState::Hovered | SensitiveZoneState::SelectedHovered)
    }

    /// Get the visual character attributes for current state
    pub fn get_display_attributes(&self) -> (Option<(u8, u8, u8)>, Option<(u8, u8, u8)>, bool, bool, bool) {
        let style = self.current_style();
        (
            style.fg_color,
            style.bg_color,
            style.bold,
            style.underline,
            style.italic,
        )
    }
}

impl ZoneStyle {
    /// Create default normal styling
    pub fn default() -> Self {
        Self {
            fg_color: None,
            bg_color: None,
            bold: false,
            underline: false,
            italic: false,
        }
    }

    /// Create default hover styling (typically highlighted)
    pub fn default_hover() -> Self {
        Self {
            fg_color: Some((255, 255, 255)), // White text
            bg_color: Some((100, 149, 237)),  // Cornflower blue background
            bold: false,
            underline: false,
            italic: false,
        }
    }

    /// Create default selected styling
    pub fn default_selected() -> Self {
        Self {
            fg_color: Some((255, 255, 255)), // White text
            bg_color: Some((70, 130, 180)),   // Steel blue background
            bold: true,
            underline: false,
            italic: false,
        }
    }
}

impl Default for SensitiveMetadata {
    fn default() -> Self {
        Self {
            display_text: None,
            tooltip: None,
            enabled: true,
            original_line: None,
            char_range: None,
            custom_data: None,
        }
    }
}