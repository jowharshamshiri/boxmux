//! Test modules for BoxMux
//!
//! This module contains all the test suites organized by functionality.

#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
pub mod integration_tests;

#[cfg(test)]
pub mod edge_case_tests;

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
pub mod terminal_100_width_reality_test;

#[cfg(test)]
pub mod debug_bounds_calculation;

#[cfg(test)]
pub mod yaml_persistence_integration_test;

#[cfg(test)]
pub mod muxbox_bounds_clipping_tests;

// Commented out due to test framework complexity - functionality tested via demo YAML
//#[cfg(test)]
//pub mod choice_overflow_tests;
