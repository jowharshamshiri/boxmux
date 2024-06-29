#[macro_use]
extern crate lazy_static;
extern crate clap;

use clap::{App, Arg};
use draw_loop::DrawLoop;
use input_loop::InputLoop;
use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
use simplelog::*;
use uuid::Uuid;
use std::fs::File;
use std::io::Write as IoWrite;
use std::io::{stdin, stdout, Read};
use std::path::Path;
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
use thread_manager::{Message, Runnable, RunnableImpl, ThreadManager};

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
mod utils;

use crate::model::app::*;
use crate::model::common::*;
use crate::model::layout::*;
use crate::model::panel::*;
use crate::utils::*;

fn run_panel_threads(manager: &mut ThreadManager, app_context: &AppContext) {
	let active_layout = app_context.app.get_active_layout();

	let mut non_threaded_panels: Vec<String> = vec![];

	if let Some(layout) = active_layout {
		for panel in layout.get_all_panels() {
			if panel.has_refresh() {
				let panel_id = panel.id.clone();

				if panel.thread.unwrap_or(false){
					let vec_fn = move || vec![panel_id.clone()];

					create_runnable_with_dynamic_input!(
						PanelRefreshLoop,
						Box::new(vec_fn),
						|inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>, vec: Vec<String>| -> bool {
							let mut state_unwrapped = state.deep_clone();
							let panel = state_unwrapped.app.get_panel_by_id_mut(&vec[0]).unwrap();
							let output = execute_commands(panel.on_refresh.clone().unwrap().as_ref());
							inner.send_message(Message::PanelOutputUpdate(vec[0].clone(), output));
							true
						},
						|inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>, vec: Vec<String>| -> bool {
							std::thread::sleep(std::time::Duration::from_millis(100));
							true
						}
					);

					let panel_refresh_loop = PanelRefreshLoop::new(app_context.deep_clone(), Box::new(vec_fn));
					manager.spawn_thread(panel_refresh_loop);
				}else{
					non_threaded_panels.push(panel_id.clone());
				}
			}
		}
		if !non_threaded_panels.is_empty() {
			let vec_fn = move || non_threaded_panels.clone();

			create_runnable_with_dynamic_input!(
				PanelRefreshLoop,
				Box::new(vec_fn),
				|inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>, vec: Vec<String>| -> bool {
					let mut state_unwrapped = state.deep_clone();

					for panel_id in vec.iter() {
						let panel = state_unwrapped.app.get_panel_by_id_mut(&panel_id).unwrap();
						let output = execute_commands(panel.on_refresh.clone().unwrap().as_ref());
						inner.send_message(Message::PanelOutputUpdate(panel_id.clone(), output));
					}

					true
				},
				|inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>, vec: Vec<String>| -> bool {
					std::thread::sleep(std::time::Duration::from_millis(100));
					true
				}
			);

			let panel_refresh_loop = PanelRefreshLoop::new(app_context.deep_clone(), Box::new(vec_fn));
			manager.spawn_thread(panel_refresh_loop);
		}
	}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Crossbash")
        .version("1.0")
        .author("jowharshamshiri@gmail.com")
        .about("TUI based on the provided YAML configuration.")
        .arg(
            Arg::new("yaml_file")
                .required(true)
                .index(1)
                .help("Sets the yaml_file file to use"),
        )
        .get_matches();

    let yaml_path = matches.value_of("yaml_file").unwrap();
    let yaml_path = Path::new(yaml_path);

    if !yaml_path.exists() {
        eprintln!("Yaml file does not exist: {}", yaml_path.display());
        return Ok(());
    }

    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("app.log")?,
    )])?;

    let app_context = load_app_from_yaml(yaml_path.to_str().unwrap())
        .expect("Failed to load app");

    let mut manager = ThreadManager::new(app_context.deep_clone());

    let input_loop_uuid = manager.spawn_thread(InputLoop::new(app_context.deep_clone()));
    let draw_loop_uuid = manager.spawn_thread(DrawLoop::new(app_context.deep_clone()));
	
	run_panel_threads(&mut manager, &app_context);
	
    manager.run();

    Ok(())
}