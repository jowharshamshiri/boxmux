// BoxMux UI Components
// 
// This module contains reusable UI components that can be used across different parts of BoxMux.
// Components are designed to be context-aware and handle their own rendering and interactions.

pub mod defaults;
pub mod vertical_scrollbar;
pub mod horizontal_scrollbar;
pub mod tab_bar;
pub mod border;
pub mod status_indicator;
pub mod choice_renderer;
pub mod selection_styles;
pub mod overflow_renderer;
pub mod error_display;
pub mod box_renderer;
pub mod chart_component;
pub mod table_component;
pub mod progress_bar;

pub use vertical_scrollbar::VerticalScrollbar;
pub use horizontal_scrollbar::HorizontalScrollbar;
pub use tab_bar::{TabBar, TabNavigationAction};
pub use border::{Border, BorderStyle, BorderCharSet};
pub use status_indicator::{StatusIndicator, StatusType};
pub use choice_renderer::ChoiceRenderer;
pub use selection_styles::{
    SelectionStyleRenderer, SelectionStyleConfig, SelectionStyle, FocusStyle, 
    FeedbackStyle, SelectionIndicators, BorderChars
};
pub use overflow_renderer::{OverflowRenderer, OverflowConfig, OverflowBehavior};
pub use error_display::{ErrorDisplay, ErrorDisplayConfig, ErrorInfo, ErrorSeverity, ErrorSpan, CaretPositioning};
pub use box_renderer::BoxRenderer;
pub use chart_component::{ChartComponent, ChartConfig, ChartType, DataPoint};
pub use table_component::{TableComponent, TableComponentConfig};
pub use progress_bar::{ProgressBar, ProgressBarConfig, ProgressState, ProgressBarOrientation};