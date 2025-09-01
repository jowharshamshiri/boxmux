// Re-export the necessary modules
pub mod model {
    pub mod app;
    pub mod common;
    pub mod layout;
    pub mod muxbox;
}

#[macro_use]
pub mod thread_manager;
// pub mod choice_threads; // T311: Removed with ChoiceThreadManager unification
pub mod ansi_processor;
pub mod ansi_color_processor;
pub mod chart;
pub mod circular_buffer;
pub mod draw_loop;
pub mod draw_utils;
pub mod input_loop;
pub mod live_yaml_sync;
pub mod plugin;
pub mod pty_manager;
pub mod resize_loop;
pub mod socket_loop;
pub mod table;
pub mod utils;
pub mod validation;

#[cfg(test)]
pub mod tests;

pub use ansi_processor::*;
pub use ansi_color_processor::*;
pub use chart::*;
pub use draw_loop::*;
pub use input_loop::*;
pub use model::app::*;
pub use model::common::*;
pub use model::layout::*;
pub use model::muxbox::*;
pub use plugin::*;
pub use pty_manager::*;
pub use table::*;
pub use thread_manager::*;
pub use utils::*;
pub use validation::*;

#[macro_use]
pub extern crate lazy_static;
