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
pub mod pty_process_info_tests;

#[cfg(test)]
pub mod pty_error_states_tests;

#[cfg(test)]
pub mod socket_pty_control_tests;
