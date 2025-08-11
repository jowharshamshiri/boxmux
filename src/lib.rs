// Re-export the necessary modules
pub mod model {
    pub mod app;
    pub mod common;
    pub mod layout;
    pub mod panel;
}

#[macro_use]
pub mod thread_manager;
pub mod choice_threads;
pub mod draw_loop;
pub mod draw_utils;
pub mod input_loop;
pub mod resize_loop;
pub mod socket_handler;
pub mod socket_server;
pub mod socket_service;
pub mod utils;
pub mod validation;

#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
pub mod integration_tests;

#[cfg(test)]
pub mod edge_case_tests;

pub use draw_loop::*;
pub use input_loop::*;
pub use model::app::*;
pub use model::common::*;
pub use model::layout::*;
pub use model::panel::*;
pub use thread_manager::*;
pub use utils::*;
pub use validation::*;

#[macro_use]
pub extern crate lazy_static;
