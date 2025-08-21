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

#[cfg(test)]
pub mod automatic_scrollbar_tests;

#[cfg(test)]
pub mod streaming_script_tests;

#[cfg(test)]
pub mod clipboard_tests;