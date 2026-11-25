//! Tests that span several modules. Unit tests live next to the code they test.

use std::time::Duration;

pub mod test_utils;

/// How long a test waits for something a script or terminal program does (its output, its
/// exit, its processes going away) before it calls the run hung and fails, naming what it
/// waited for. It bounds a hang, not the work: every wait ends at the event it waits for, so a
/// passing test never waits this long, while on a machine loaded well past its cores starting
/// one process can take seconds, and PowerShell takes seconds to start on a cold Windows
/// machine.
pub const HANG_BOUND: Duration = Duration::from_secs(120);

mod box_renderer_tests;
mod comprehensive_variable_tests;
mod execution_mode_schema_validation_tests;
mod hierarchical_variable_tests;
mod integration_yaml_tests;
mod layout_validation_tests;
mod terminal_compatibility_tests;
mod terminal_emulator_tests;
mod terminal_title_tests;
