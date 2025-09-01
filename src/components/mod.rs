// BoxMux UI Components
// 
// This module contains reusable UI components that can be used across different parts of BoxMux.
// Components are designed to be context-aware and handle their own rendering and interactions.

pub mod vertical_scrollbar;
pub mod horizontal_scrollbar;

pub use vertical_scrollbar::VerticalScrollbar;
pub use horizontal_scrollbar::HorizontalScrollbar;