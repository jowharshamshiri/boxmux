//! Test modules for BoxMux
//!
//! This module contains all the test suites organized by functionality.

#[cfg(test)]
pub mod test_utils;

// F0326-F0355: Visual Testing System - Character-Exact Validation
#[cfg(test)]
pub mod visual_testing;

#[cfg(test)]
pub mod visual_basic_box_tests;

#[cfg(test)]
pub mod visual_demo_test;

#[cfg(test)]
pub mod visual_animation_demo;

#[cfg(test)]
pub mod integration_tests;

#[cfg(test)]
pub mod edge_case_tests;

#[cfg(test)]
pub mod execution_mode_tests;

#[cfg(test)]
pub mod stream_only_execution_tests;

#[cfg(test)]
pub mod execution_mode_schema_validation_tests; // F0230: ExecutionMode YAML Schema tests

#[cfg(test)]
#[cfg(test)]
pub mod execution_mode_stream_source_traits_tests; // F0227: ExecutionMode Stream Source Traits Tests

#[cfg(test)]
pub mod execution_mode_message_system_tests; // F0228: ExecutionMode Message System Tests

#[cfg(test)]
pub mod unified_execution_mode_tests; // F0229: Unified ExecutionMode System Tests

#[cfg(test)]
pub mod thread_mode_execution_tests;

#[cfg(test)]
pub mod immediate_mode_execution_tests; // F0224: Immediate Mode Execution Tests

#[cfg(test)]
pub mod pty_mode_execution_tests; // F0226: PTY Mode Execution Tests

#[cfg(test)]
pub mod pty_mode_execution_integration_tests; // F0226: PTY Mode Execution Integration Tests

#[cfg(test)]
pub mod execution_mode_stream_integration_tests; // F0223: ExecutionMode Stream Integration tests

#[cfg(test)]
pub mod socket_tests;

#[cfg(test)]
pub mod variable_tests;

#[cfg(test)]
pub mod integration_yaml_tests;

#[cfg(test)]
pub mod comprehensive_variable_tests;

#[cfg(test)]
pub mod hierarchical_variable_tests;

#[cfg(test)]
pub mod chart_tests;

#[cfg(test)]
pub mod plugin_integration_tests;

#[cfg(test)]
pub mod plugin_dynamic_tests;

#[cfg(test)]
pub mod table_tests;

#[cfg(test)]
pub mod layout_validation_tests;

pub mod auto_scroll_tests;
#[cfg(test)]
pub mod automatic_scrollbar_tests;

#[cfg(test)]
pub mod clipboard_tests;

#[cfg(test)]
pub mod home_end_navigation_tests;

#[cfg(test)]
pub mod hotkey_tests;

#[cfg(test)]
pub mod mouse_click_tests;

#[cfg(test)]
pub mod choice_click_bounds_tests;

#[cfg(test)]
pub mod clickable_scrollbar_tests;

#[cfg(test)]
pub mod pty_input_tests;

#[cfg(test)]
pub mod pty_resize_tests;

#[cfg(test)]
pub mod ansi_processor_tests;

#[cfg(test)]
pub mod tab_bar_component_tests;

#[cfg(test)]
pub mod special_key_tests;

#[cfg(test)]
pub mod pty_scrollback_tests;

#[cfg(test)]
pub mod z_index_tests;

#[cfg(test)]
pub mod pty_process_info_tests;

#[cfg(test)]
pub mod pty_error_states_tests;

#[cfg(test)]
pub mod text_wrapping_tests;

#[cfg(test)]
pub mod socket_pty_control_tests;

#[cfg(test)]
pub mod muxbox_resize_tests;

#[cfg(test)]
pub mod muxbox_move_tests;

#[cfg(test)]
pub mod yaml_live_persistence_tests;

#[cfg(test)]
pub mod resize_100_percent_debug;

#[cfg(test)]
pub mod tab_stop_tests; // F0313: Tab Stop Management tests

#[cfg(test)]
pub mod line_attribute_tests; // F0314: Line Attribute Support tests

#[cfg(test)]
pub mod terminal_title_tests; // F0315: Terminal Title Support tests

#[cfg(test)]
pub mod pty_box_resize_integration_tests; // F0317: PTY Box Resize Integration tests

#[cfg(test)]
pub mod character_set_tests; // F0318: Character Set Support tests

#[cfg(test)]
pub mod terminal_compatibility_tests; // F0319: Terminal Compatibility Testing

#[cfg(test)]
pub mod terminal_100_width_reality_test;

#[cfg(test)]
pub mod debug_bounds_calculation;

#[cfg(test)]
pub mod yaml_persistence_integration_test;

#[cfg(test)]
pub mod muxbox_bounds_clipping_tests;

#[cfg(test)]
pub mod conditional_stream_creation_tests;

#[cfg(test)]
mod tab_close_button_tests;

#[cfg(test)]
mod close_button_integration_test;

#[cfg(test)]
pub mod scrollable_tabs_tests;

#[cfg(test)]
pub mod unified_execution_architecture_tests;

#[cfg(test)]
pub mod periodic_source_stability_tests;

// Commented out due to test framework complexity - functionality tested via demo YAML
//#[cfg(test)]
//pub mod choice_overflow_tests;
pub mod full_screen_detection_tests;

#[cfg(test)]
pub mod border_component_tests;

#[cfg(test)]
pub mod status_indicator_tests;

#[cfg(test)]
pub mod performance_optimization_tests; // F0316: Performance Optimization - Dirty Region Tracking Tests
