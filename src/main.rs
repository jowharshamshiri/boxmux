#[macro_use]
extern crate lazy_static;

use draw_loop::DrawLoop;
use input_loop::InputLoop;
use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
use simplelog::*;
use std::fs::File;
use std::io::Write as IoWrite;
use std::io::{stdin, stdout, Read};
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;
use thread_manager::{Runnable, ThreadManager};

use serde::{
    de::MapAccess, de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

use std::sync::atomic::{AtomicBool, Ordering};

mod model {
    pub mod app;
    pub mod common;
    pub mod layout;
    pub mod panel;
}

#[macro_use]
pub mod thread_manager;
mod draw_loop;
mod input_loop;
mod panel_loop;
mod resize_loop;
mod utils;

use crate::model::app::*;
use crate::model::common::*;
use crate::model::layout::*;
use crate::model::panel::*;
use crate::utils::*;

// fn main() -> Result<(), Box<dyn std::error::Error>> {
//     env_logger::init();

//     CombinedLogger::init(vec![WriteLogger::new(
//         LevelFilter::Debug,
//         Config::default(),
//         File::create("app.log")?,
//     )])?;

//     let mut manager = ThreadManager::new();
//     manager.spawn_thread(MyRunnable);
//     manager.spawn_thread(MyRunnable);

//     let data = App::new();
//     manager.send_data_to_thread(data.clone(), 0);
//     manager.send_data_to_thread(data.clone(), 1);

//     if let Some(received_data) = manager.receive_data_from_thread() {
//         let initial_hash = manager.get_hash(&data);
//         let received_hash = manager.get_hash(&received_data);

//         if initial_hash != received_hash {
//             manager.send_update_to_all_threads(received_data);
//         }
//     }

//     manager.join_threads();
// }

// fn mai3n() {
//     let (app, app_graph) =
//         load_app_from_yaml("/Users/bahram/ws/prj/machinegenesis/crossbash2/layouts/dashboard.yaml")
//             .expect("Failed to load app");

//     if let Some(layout) = app.get_layout_by_id("dashboard") {
//         if let Some(panel) = app_graph.get_panel_by_id(&layout.id, "header") {
//             println!("Found panel: {:?}", panel);
//         }

//         if let Some(parent) = app_graph.get_parent(&layout.id, "time") {
//             println!("Parent panel: {:?}", parent);
//         }

//         let children = app_graph.get_children(&layout.id, "footer");
//         for child in children {
//             println!("Child panel: {:?}", child);
//         }
//     }
// }

// fn start_panel_event_threads(app: &mut App, app_graph: &AppGraph,thread_manager: &mut ThreadManager) {
// 	for layout in &app.layouts {
// 		for panel in &layout.children {
// 			if panel.has_refresh() {
// 				let app = Arc::new(Mutex::new(app.clone()));
// 				let app_clone = app.clone();
// 				let panel_clone = panel.clone();

// 				thread_manager.spawn_thread(move || {
// 					let mut app = app_clone.lock().unwrap();
// 					let mut panel = app.get_panel_by_id(&panel_clone.id).unwrap();

// 					loop {
// 						let mut app = app.clone();
// 						let mut panel = app.get_panel_by_id(&panel.id).unwrap();

// 						if let Some(refresh) = &panel.refresh {
// 							match refresh {
// 								Refresh::Command(command) => {
// 									let output = Command::new("sh")
// 										.arg("-c")
// 										.arg(command)
// 										.output()
// 										.expect("Failed to execute command");

// 									let output = String::from_utf8_lossy(&output.stdout);
// 									panel.content = output.to_string();
// 								}
// 								Refresh::Interval(interval) => {
// 									thread::sleep(Duration::from_secs(*interval));
// 								}
// 							}
// 						}

// 						thread::sleep(Duration::from_secs(1));
// 					}
// 				});
// 			}
// 		}
// 	}

// 	for layout in &app.layouts {
// 		for panel in &layout.children {
// 			if panel.has_refresh() {
// 				let app = Arc::new(Mutex::new(app.clone()));
// 				let app_clone = app.clone();
// 				let panel_clone = panel.clone();

// 				thread::spawn(move || {
// 					let mut app = app_clone.lock().unwrap();
// 					let mut panel = app.get_panel_by_id(&panel_clone.id).unwrap();

// 					loop {
// 						let mut app = app.clone();
// 						let mut panel = app.get_panel_by_id(&panel.id).unwrap();

// 						if let Some(refresh) = &panel.refresh {
// 							match refresh {
// 								Refresh::Command(command) => {
// 									let output = Command::new("sh")
// 										.arg("-c")
// 										.arg(command)
// 										.output()
// 										.expect("Failed to execute command");

// 									let output = String::from_utf8_lossy(&output.stdout);
// 									panel.content = output.to_string();
// 								}
// 								Refresh::Interval(interval) => {
// 									thread::sleep(Duration::from_secs(*interval));
// 								}
// 							}
// 						}

// 						thread::sleep(Duration::from_secs(1));
// 					}
// 				});
// 			}
// 		}
// 	}
// }

fn main() -> Result<(), Box<dyn std::error::Error>> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("app.log")?,
    )])?;

    log::info!("Starting app");
    // let mut stdout = AlternateScreen::from(stdout().into_raw_mode()?);
    let app_context =
        load_app_from_yaml("/Users/bahram/ws/prj/machinegenesis/crossbash/layouts/dashboard.yaml")
            .expect("Failed to load app");

    let mut manager = ThreadManager::new(app_context.deep_clone());

    let input_loop_uuid = manager.spawn_thread(InputLoop::new(app_context.deep_clone()));
    let draw_loop_uuid = manager.spawn_thread(DrawLoop::new(app_context.deep_clone()));

    manager.run();

    Ok(())
}

// fn main() {
//     env_logger::init();
//     let app_context = AppContext::new(App::new());
//     let mut manager = ThreadManager::new(app_context.clone());
//     let uuid1 = manager.spawn_thread(TestRunnableOne::new(app_context.clone()));
//     let uuid2 = manager.spawn_thread(TestRunnableTwo::new(app_context.clone()));
//     let uuid3 = manager.spawn_thread(TestRunnableThree::new(app_context.clone()));

//     let data = AppContext::new(App::new());
//     manager.send_data_to_thread(data.clone(), uuid1);
//     manager.send_data_to_thread(data.clone(), uuid2);
//     manager.send_data_to_thread(data.clone(), uuid3);

//     // Run the manager's loop in a separate thread
//     let manager = Arc::new(manager);
//     let manager_clone = Arc::clone(&manager);

//     let handle = thread::spawn(move || {
//         manager_clone.run();
//     });

//     handle.join().unwrap();
// }
