//! Default implementations for core system structs
//!
//! This module centralizes all Default trait implementations for core system types
//! including ANSI processing, PTY management, and other infrastructure components.
//!
//! Note: This module contains reference implementations - actual implementations
//! remain in their original files to avoid circular dependencies.

// ============================================================================
// ANSI PROCESSING DEFAULTS (from ansi_processor.rs)
// ============================================================================

/*
AnsiProcessor::default() provides:
  - use_screen_buffer: false - use line-based processing by default
  - terminal_state: TerminalState::default() - default terminal emulator state
  - raw_buffer: [] - empty input buffer
  - processed_lines: [] - empty processed output
  - dirty_regions: [] - no dirty regions initially
  - last_processed_index: 0 - start from beginning

TerminalState::default() provides complete terminal emulation:
  - primary_screen: 80x24 character buffer - standard terminal size
  - alternate_screen: 80x24 character buffer - for fullscreen apps
  - use_alternate_screen: false - start with primary screen
  - cursor: CursorState::default() - cursor at origin, visible
  - terminal_modes: TerminalMode::default() - standard terminal behavior
  - scrolling_region: full screen (0-23) - entire terminal scrollable
  - tab_stops: [8, 16, 24, 32, 40, 48, 56, 64, 72] - standard tab positions
  - character sets: US ASCII for G0/G1 - standard character encoding

CursorState::default() provides:
  - col: 0, row: 0 - cursor at top-left origin
  - visible: true - cursor visible by default
  - style: Block - standard block cursor style
  - saved positions: (0, 0) - no saved cursor position

TerminalMode::default() provides:
  - application_cursor_keys: false - normal cursor key mode
  - application_keypad: false - normal keypad mode
  - auto_wrap: true - wrap long lines automatically
  - origin_mode: false - absolute cursor positioning
  - insert_mode: false - overwrite mode by default
  - local_echo: true - echo input locally

TerminalCell::default() provides:
  - character: ' ' - space character (empty cell)
  - All colors: None - use terminal default colors
  - All attributes: false - no text formatting
  - font_index: 0 - default font selection
*/

// ============================================================================
// PTY MANAGEMENT DEFAULTS (from pty_manager.rs)
// ============================================================================

/*
PtyConfig::default() provides:
  - rows: 24, cols: 80 - standard terminal dimensions
  - shell: "/bin/bash" - default Unix shell
  - working_dir: None - inherit from parent process
  - env_vars: {} - empty environment override

PtyState::default() -> PtyState::NotStarted
  - PTY processes start in uninitialized state
  - Other states: Starting, Running, Finished, Error

PtyProcessInfo::default() provides:
  - pid: None - no process ID initially
  - status: NotStarted - process not launched
  - exit_code: None - no exit status yet
  - command: "" - empty command string
  - working_dir: None - no working directory set
  - start_time/end_time: None - no timing information
*/

// ============================================================================
// CIRCULAR BUFFER DEFAULTS (from circular_buffer.rs)
// ============================================================================

/*
CircularBuffer<T>::default() -> CircularBuffer::new(10000)
  - Default capacity: 10,000 items - sufficient for most use cases
  - Provides efficient scrollback buffer for terminal output
  - Automatically overwrites oldest data when capacity exceeded
*/

// ============================================================================
// TABLE PROCESSING DEFAULTS (from table.rs)
// ============================================================================

/*
TableFormat::default() -> TableFormat::Csv
  - CSV format most commonly used for data tables
  - Other formats: Json, Custom - specialized formats

TableSortOrder::default() -> TableSortOrder::Ascending
  - Natural ascending sort order for most data
  - Alternative: Descending for reverse ordering

TableBorderStyle::default() -> TableBorderStyle::Single
  - Clean single-line borders for terminal display
  - Matches BorderStyle::Single for UI consistency
*/

// ============================================================================
// VALIDATION DEFAULTS (from validation.rs)
// ============================================================================

/*
ValidationLevel::default() -> ValidationLevel::Strict
  - Strict validation catches configuration errors early
  - Other levels: Lenient - allows more flexibility

ValidationConfig::default() provides:
  - level: Strict - comprehensive validation rules
  - allow_unknown_fields: false - reject unrecognized configuration
  - require_all_fields: true - ensure complete configuration
  - validate_references: true - check cross-references between components
*/

// ============================================================================
// COLOR PROCESSING DEFAULTS (from ansi_color_processor.rs)
// ============================================================================

/*
AnsiColorProcessor::default() provides:
  - use_bright_colors: true - enhanced color support
  - color_support_level: TrueColor - full 24-bit color support

ColorSupportLevel::default() -> ColorSupportLevel::TrueColor
  - Assumes modern terminal with full color support
  - Fallback levels: Colors256, Colors16, Monochrome available
*/
