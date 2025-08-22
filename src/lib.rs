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
pub mod socket_loop;
pub mod real_time_updates;
pub mod streaming_executor;
pub mod streaming_messages;
pub mod streaming_panel_manager;
pub mod unified_thread_pool;
pub mod utils;
pub mod validation;
pub mod chart;
pub mod plugin;
pub mod table;

#[cfg(test)]
pub mod tests;

pub use draw_loop::*;
pub use input_loop::*;
pub use model::app::*;
pub use model::common::*;
pub use model::layout::*;
pub use model::panel::*;
pub use thread_manager::*;
pub use utils::*;
pub use validation::*;
pub use chart::*;
pub use plugin::*;
pub use real_time_updates::*;
pub use streaming_executor::*;
pub use streaming_messages::*;
pub use streaming_panel_manager::*;
pub use table::*;
pub use unified_thread_pool::*;

#[macro_use]
pub extern crate lazy_static;
