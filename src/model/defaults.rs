//! Default implementations for model structs
//! 
//! This module centralizes all Default trait implementations for model types,
//! providing easy visibility into system default values and configurations.
//!
//! Note: This module contains reference documentation - actual implementations 
//! remain in their original files (common.rs, muxbox.rs, etc.) due to serde dependencies.

// ============================================================================
// EXECUTION AND CONFIGURATION DEFAULTS (from common.rs)
// ============================================================================

/*
ExecutionMode::default() -> ExecutionMode::Immediate
  - Default execution mode for scripts and commands
  - Immediate execution on UI thread for simple operations

Config::default() -> Config { frame_delay: 30, locked: false }
  - frame_delay: 30ms default refresh rate
  - locked: false - allows UI resizing and movement by default
*/

// ============================================================================
// MUXBOX DEFAULTS (from muxbox.rs)  
// ============================================================================

/*
MuxBox::default() provides these key defaults:
  - border: Some(true) - boxes have borders by default
  - scrollable: Some(true) - content can be scrolled
  - focusable: Some(true) - can receive keyboard focus
  - append_output: Some(false) - output replacement by default
  - streaming: Some(false) - batch output by default  
  - auto_scroll_to_bottom: Some(false) - manual scroll control
  - position: InputBounds all "0" - positioned at origin
  - anchor: Anchor::Center - centered positioning
  - streams: empty HashMap - no streams initially

InputBounds::default() -> all coordinates "0"
  - x1: "0", y1: "0", x2: "0", y2: "0"
  - Represents uninitialized position bounds

Bounds::default() -> all coordinates 0
  - x1: 0, y1: 0, x2: 0, y2: 0  
  - Computed bounds initialized to origin

Choice::default() provides:
  - text: "" - empty choice text
  - append_output: Some(false) - replace output by default
  - streaming: Some(false) - batch execution by default
  - All optional fields None - minimal configuration
*/

// ============================================================================
// STREAM AND LAYOUT DEFAULTS
// ============================================================================

/*
Stream::default() provides:
  - stream_type: StreamType::DefaultContent
  - label: "Content" - generic content label
  - content: "" - empty content initially
  - choices: [] - no choices initially
  - closeable: false - cannot be closed by default
  - content_hash: 0 - no content hash initially

Layout::default() provides:
  - id: "default" - default layout identifier
  - children: None - no child muxboxes initially

App::default() provides:
  - active_layout_id: "" - no active layout initially
  - layouts: [] - empty layout list
  - config: Config::default() - default configuration
  - All optional fields None - minimal setup

StreamType::default() -> StreamType::DefaultContent
OverflowBehavior::default() -> OverflowBehavior::Scroll  
Anchor::default() -> Anchor::TopLeft
TitlePosition::default() -> TitlePosition::Top
*/