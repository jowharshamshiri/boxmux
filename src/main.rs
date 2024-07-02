#[macro_use]
extern crate lazy_static;
extern crate clap;

use clap::{App, Arg};
use crossbash_lib::create_runnable_with_dynamic_input;
use crossbash_lib::resize_loop::ResizeLoop;
use crossbash_lib::thread_manager;
use crossbash_lib::DrawLoop;
use crossbash_lib::InputLoop;
use simplelog::*;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use thread_manager::{Message, Runnable, RunnableImpl, ThreadManager};
use uuid::Uuid;

use crossbash_lib::model::app::*;
use crossbash_lib::utils::*;

lazy_static! {
    static ref LAST_EXECUTION_TIMES: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
}

fn run_panel_threads(manager: &mut ThreadManager, app_context: &AppContext) {
    let active_layout = app_context.app.get_active_layout();

    let mut non_threaded_panels: Vec<String> = vec![];

    if let Some(layout) = active_layout {
        for panel in layout.get_all_panels() {
            if panel.script.is_some() {
                let panel_id = panel.id.clone();

                if panel.thread.unwrap_or(false) {
                    let vec_fn = move || vec![panel_id.clone()];

                    create_runnable_with_dynamic_input!(
                        PanelRefreshLoop,
                        Box::new(vec_fn),
                        |inner: &mut RunnableImpl,
                         app_context: AppContext,
                         messages: Vec<Message>,
                         vec: Vec<String>|
                         -> bool { true },
                        |inner: &mut RunnableImpl,
                         app_context: AppContext,
                         messages: Vec<Message>,
                         vec: Vec<String>|
                         -> bool {
                            let mut state_unwrapped = app_context.deep_clone();
                            let app_graph = state_unwrapped.app.generate_graph();
                            let panel = state_unwrapped.app.get_panel_by_id_mut(&vec[0]).unwrap();
                            let output = execute_commands(panel.script.clone().unwrap().as_ref());
                            inner.send_message(Message::PanelOutputUpdate(vec[0].clone(), output));
                            std::thread::sleep(std::time::Duration::from_millis(
                                panel.calc_refresh_interval(&app_context, &app_graph),
                            ));
                            true
                        }
                    );

                    let panel_refresh_loop =
                        PanelRefreshLoop::new(app_context.deep_clone(), Box::new(vec_fn));
                    manager.spawn_thread(panel_refresh_loop);
                } else {
                    non_threaded_panels.push(panel_id.clone());
                }
            }
        }
        if !non_threaded_panels.is_empty() {
            let vec_fn = move || non_threaded_panels.clone();

            create_runnable_with_dynamic_input!(
                PanelRefreshLoop,
                Box::new(vec_fn),
                |inner: &mut RunnableImpl,
                 app_context: AppContext,
                 messages: Vec<Message>,
                 vec: Vec<String>|
                 -> bool { true },
                |inner: &mut RunnableImpl,
                 app_context: AppContext,
                 messages: Vec<Message>,
                 vec: Vec<String>|
                 -> bool {
                    let mut state_unwrapped = app_context.deep_clone();

                    let mut last_execution_times = LAST_EXECUTION_TIMES.lock().unwrap();

                    for panel_id in vec.iter() {
                        let panel = state_unwrapped.app.get_panel_by_id_mut(panel_id).unwrap();
                        let refresh_interval = panel.refresh_interval.unwrap_or(1000);

                        let last_execution_time = last_execution_times
                            .entry(panel_id.clone())
                            .or_insert(Instant::now());

                        if last_execution_time.elapsed() >= Duration::from_millis(refresh_interval)
                        {
                            let output = execute_commands(panel.script.clone().unwrap().as_ref());
                            inner
                                .send_message(Message::PanelOutputUpdate(panel_id.clone(), output));

                            *last_execution_time = Instant::now();
                        }
                    }

					std::thread::sleep(std::time::Duration::from_millis(app_context.config.frame_delay));

                    true
                }
            );

            let panel_refresh_loop =
                PanelRefreshLoop::new(app_context.deep_clone(), Box::new(vec_fn));
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
        .arg(
            Arg::new("frame_delay")
                .short('d')
                .long("frame_delay")
                .takes_value(true)
                .default_value("100")
                .help("Sets the frame delay in milliseconds"),
        )
        .get_matches();

    let yaml_path = matches.value_of("yaml_file").unwrap();
    let frame_delay = matches
        .value_of("frame_delay")
        .unwrap()
        .parse::<u64>()
        .unwrap_or(100);

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
    let config = crossbash_lib::model::common::Config::new(frame_delay);
    let app = load_app_from_yaml(yaml_path.to_str().unwrap()).expect("Failed to load app");

    let app_context = AppContext::new(app, config);

    //create alternate screen in terminal and clear it
    let mut _stdout = std::io::stdout();
    write!(_stdout, "{}", termion::screen::ToAlternateScreen)?;
    write!(_stdout, "{}", termion::clear::All)?;

    // let panel_ids_to_modify: Vec<String> = vec!["log_input".to_string()];

    // for panel_id in panel_ids_to_modify {
    //     if let Some(panel) = app_context.app.get_panel_by_id_mut(&panel_id) {
    //         let data = vec![("A", 5), ("B", 3), ("C", 8), ("D", 2), ("E", 6), ("F", 7), ("G", 4), ("H", 5)];
    //         let chart = generate_bar_chart(data, 10, 50);
    //         panel.content = Some(chart);
    //     }
    // }

    let mut manager = ThreadManager::new(app_context.deep_clone());

    let input_loop_uuid = manager.spawn_thread(InputLoop::new(app_context.deep_clone()));
    let draw_loop_uuid = manager.spawn_thread(DrawLoop::new(app_context.deep_clone()));
    let resize_loop_uuid = manager.spawn_thread(ResizeLoop::new(app_context.deep_clone()));
    // let periodical_resize_loop_uuid = manager.spawn_thread(PeriodicalResizeLoop::new(app_context.deep_clone()));

    run_panel_threads(&mut manager, &app_context);

    manager.run();

    //restore normal screen
    write!(_stdout, "{}", termion::screen::ToMainScreen)?;

    Ok(())
}
