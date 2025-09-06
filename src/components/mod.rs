// BoxMux UI Components
//
// This module contains reusable UI components that can be used across different parts of BoxMux.
// Components are designed to be context-aware and handle their own rendering and interactions.

pub mod border;
pub mod box_renderer;
pub mod dimensions;
pub mod chart_component;
pub mod choice_content;
pub mod choice_menu;
pub mod defaults;
pub mod error_display;
pub mod horizontal_scrollbar;
pub mod progress_bar;
pub mod renderable_content;
pub mod selection_styles;
pub mod status_indicator;
pub mod tab_bar;
pub mod table_component;
pub mod text_content;
pub mod vertical_scrollbar;

#[cfg(test)]
pub mod event_system_tests;

pub use border::{Border, BorderCharSet, BorderStyle};
pub use box_renderer::{BoxRenderer, UnifiedOverflowBehavior};
pub use chart_component::{ChartComponent, ChartConfig, ChartType, DataPoint};
pub use choice_content::ChoiceContent;
pub use choice_menu::ChoiceMenu;
pub use error_display::{
    CaretPositioning, ErrorDisplay, ErrorDisplayConfig, ErrorInfo, ErrorSeverity, ErrorSpan,
};
pub use horizontal_scrollbar::HorizontalScrollbar;
pub use progress_bar::{ProgressBar, ProgressBarConfig, ProgressBarOrientation, ProgressState};
pub use renderable_content::{
    BoxResizeInfo, BoxResizeType, SensitiveMetadata, SensitiveZone, ContentDimensions,
    ContentEvent, ContentType, EventData, EventResult, EventType, HoverInfo, HoverState, KeyInfo,
    KeyModifier, MouseButton, MouseMoveInfo, RenderableContent, ResizeAnchor, ResizeState,
    ScrollDirection, ScrollInfo, TitleChangeInfo, TitleChangeSource,
};
pub use selection_styles::{
    BorderChars, FeedbackStyle, FocusStyle, SelectionIndicators, SelectionStyle,
    SelectionStyleConfig, SelectionStyleRenderer,
};
pub use status_indicator::{StatusIndicator, StatusType};
pub use tab_bar::{TabBar, TabNavigationAction};
pub use table_component::{TableComponent, TableComponentConfig};
pub use text_content::TextContent;
pub use vertical_scrollbar::VerticalScrollbar;

// Dimension classes for centralized mathematical operations
pub use dimensions::{
    ComponentDimensions, LayoutDimensions, MouseDimensions, ProgressDimensions, ScrollDimensions,
    TextDimensions,
};
