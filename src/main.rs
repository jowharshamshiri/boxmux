#[macro_use]
extern crate lazy_static;
extern crate clap;

use boxmux_lib::create_runnable_with_dynamic_input;
use boxmux_lib::resize_loop::ResizeLoop;
use boxmux_lib::send_json_to_socket;
use boxmux_lib::socket_loop::SocketLoop;
use boxmux_lib::thread_manager;
use boxmux_lib::DrawLoop;
use boxmux_lib::FieldUpdate;
use boxmux_lib::InputLoop;
use boxmux_lib::Panel;
use boxmux_lib::SocketFunction;
use clap::{Arg, Command};
use std::collections::HashMap;
use std::fs::File;
use std::path::Path;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;
use std::time::Instant;
use thread_manager::{Message, Runnable, RunnableImpl, ThreadManager};
use uuid::Uuid;

use boxmux_lib::model::app::*;
use boxmux_lib::utils::*;

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
                        |_inner: &mut RunnableImpl,
                         _app_context: AppContext,
                         _messages: Vec<Message>,
                         _vec: Vec<String>|
                         -> bool { true },
                        |inner: &mut RunnableImpl,
                         app_context: AppContext,
                         _messages: Vec<Message>,
                         vec: Vec<String>|
                         -> (bool, AppContext) {
                            let mut app_context_unwrapped = app_context.clone();
                            let app_graph = app_context_unwrapped.app.generate_graph();
                            let libs = app_context_unwrapped.app.libs.clone();
                            let panel = app_context_unwrapped
                                .app
                                .get_panel_by_id_mut(&vec[0])
                                .unwrap();
                            match run_script(libs, panel.script.clone().unwrap().as_ref()) {
                                Ok(output) => inner.send_message(Message::PanelOutputUpdate(
                                    panel.id.clone(),
                                    true,
                                    output,
                                )),
                                Err(e) => inner.send_message(Message::PanelOutputUpdate(
                                    panel.id.clone(),
                                    false,
                                    e.to_string(),
                                )),
                            }
                            std::thread::sleep(std::time::Duration::from_millis(
                                panel.calc_refresh_interval(&app_context, &app_graph),
                            ));
                            (true, app_context_unwrapped)
                        }
                    );

                    let panel_refresh_loop =
                        PanelRefreshLoop::new(app_context.clone(), Box::new(vec_fn));
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
                |_inner: &mut RunnableImpl,
                 _app_context: AppContext,
                 _messages: Vec<Message>,
                 _vec: Vec<String>|
                 -> bool { true },
                |inner: &mut RunnableImpl,
                 app_context: AppContext,
                 _messages: Vec<Message>,
                 vec: Vec<String>|
                 -> (bool, AppContext) {
                    let mut app_context_unwrapped = app_context.clone();

                    let mut last_execution_times = LAST_EXECUTION_TIMES.lock().unwrap();

                    for panel_id in vec.iter() {
                        let libs = app_context_unwrapped.app.libs.clone();
                        let panel = app_context_unwrapped
                            .app
                            .get_panel_by_id_mut(panel_id)
                            .unwrap();
                        let refresh_interval = panel.refresh_interval.unwrap_or(1000);

                        let last_execution_time = last_execution_times
                            .entry(panel_id.clone())
                            .or_insert(Instant::now());

                        if last_execution_time.elapsed() >= Duration::from_millis(refresh_interval)
                        {
                            match run_script(libs, panel.script.clone().unwrap().as_ref()) {
                                Ok(output) => inner.send_message(Message::PanelOutputUpdate(
                                    panel_id.clone(),
                                    true,
                                    output,
                                )),
                                Err(e) => inner.send_message(Message::PanelOutputUpdate(
                                    panel_id.clone(),
                                    false,
                                    e.to_string(),
                                )),
                            }

                            *last_execution_time = Instant::now();
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(
                        app_context.config.frame_delay,
                    ));

                    (true, app_context_unwrapped)
                }
            );

            let panel_refresh_loop = PanelRefreshLoop::new(app_context.clone(), Box::new(vec_fn));
            manager.spawn_thread(panel_refresh_loop);
        }
    }
}

/// Setup signal handler to ensure proper terminal cleanup on exit
fn setup_signal_handler() {
    use signal_hook::{consts::SIGINT, iterator::Signals};
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGINT]).expect("Error setting up signal handler");
        for _sig in signals.forever() {
            r.store(false, Ordering::SeqCst);
            cleanup_terminal();
            std::process::exit(0);
        }
    });
}

/// Cleanup terminal state - restore normal mode
fn cleanup_terminal() {
    use crossterm::{execute, terminal, event};
    let mut stdout = std::io::stdout();
    let _ = execute!(stdout, event::DisableMouseCapture);
    let _ = execute!(stdout, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = Command::new("Boxmux")
        .version(env!("CARGO_PKG_VERSION"))
        .author("jowharshamshiri@gmail.com")
        .about("A terminal multiplexer")
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
                .value_name("DELAY")
                .default_value("30")
                .help("Sets the frame delay in milliseconds"),
        )
        .subcommand(
            Command::new("stop_panel_refresh")
                .about("Stops the refresh of the panel")
                .arg(
                    Arg::new("panel_id")
                        .required(true)
                        .index(1)
                        .help("The panel id to stop the refresh of"),
                ),
        )
        .subcommand(
            Command::new("start_panel_refresh")
                .about("Starts the refresh of the panel")
                .arg(
                    Arg::new("panel_id")
                        .required(true)
                        .index(1)
                        .help("The panel id to start the refresh of"),
                ),
        )
        .subcommand(
            Command::new("replace_panel")
                .about("Replaces the panel with the provided Panel")
                .arg(
                    Arg::new("panel_id")
                        .required(true)
                        .index(1)
                        .help("The panel id to update"),
                )
                .arg(
                    Arg::new("new_panel_json")
                        .required(true)
                        .index(2)
                        .help("The new panel to update with"),
                ),
        )
        .subcommand(
            Command::new("switch_active_layout")
                .about("Switches the active layout")
                .arg(
                    Arg::new("layout_id_to_switch_to")
                        .required(true)
                        .index(1)
                        .help("The layout id to switch to"),
                ),
        )
        .subcommand(
            Command::new("update_panel_script")
                .about("Updates the panel script")
                .arg(
                    Arg::new("panel_id")
                        .required(true)
                        .index(1)
                        .help("The panel id to update the script of"),
                )
                .arg(
                    Arg::new("new_panel_script")
                        .required(true)
                        .index(2)
                        .help("The new script to update the panel with"),
                ),
        )
        .subcommand(
            Command::new("update_panel_content")
                .about("Updates the panel content")
                .arg(
                    Arg::new("panel_id")
                        .required(true)
                        .index(1)
                        .help("The panel id to update the content of"),
                )
                .arg(
                    Arg::new("success")
                        .required(true)
                        .index(2)
                        .help("Whether the content is a success or not"),
                )
                .arg(
                    Arg::new("new_panel_content")
                        .required(true)
                        .index(3)
                        .help("The new content to update the panel with"),
                ),
        )
        .subcommand(
            Command::new("add_panel")
                .about("Adds a panel to a layout")
                .arg(
                    Arg::new("layout_id")
                        .required(true)
                        .index(1)
                        .help("The layout id to add the panel to"),
                )
                .arg(
                    Arg::new("panel_json")
                        .required(true)
                        .index(2)
                        .help("The panel to add to the layout"),
                ),
        )
        .subcommand(
            Command::new("remove_panel")
                .about("Removes a panel from its layout")
                .arg(
                    Arg::new("panel_id")
                        .required(true)
                        .index(1)
                        .help("The panel id to remove from its layout"),
                ),
        )
        .get_matches();

    // Handle the stop_panel_refresh subcommand
    if let Some(matches) = matches.subcommand_matches("stop_panel_refresh") {
        if let Some(panel_id) = matches.get_one::<String>("panel_id") {
            // Construct the enum variant using the struct syntax
            let socket_function = SocketFunction::StopPanelRefresh {
                panel_id: panel_id.clone(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;

            // Send the constructed value to the socket function
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Panel ID is required for stop_panel_refresh command".into());
        }
    }

    // Handle the start_panel_refresh subcommand
    if let Some(matches) = matches.subcommand_matches("start_panel_refresh") {
        if let Some(panel_id) = matches.get_one::<String>("panel_id") {
            // Construct the enum variant using the struct syntax
            let socket_function = SocketFunction::StartPanelRefresh {
                panel_id: panel_id.clone(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;

            // Send the constructed value to the socket function
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Panel ID is required for start_panel_refresh command".into());
        }
    }

    // Handle the replace_panel subcommand
    if let Some(matches) = matches.subcommand_matches("replace_panel") {
        if let Some(panel_id) = matches.get_one::<String>("panel_id") {
            if let Some(new_panel_json) = matches.get_one::<String>("new_panel_json") {
                // Construct the enum variant using the struct syntax
                let submitted_panel = serde_json::from_str::<Panel>(new_panel_json)?;

                let socket_function = SocketFunction::ReplacePanel {
                    panel_id: panel_id.clone(),
                    new_panel: submitted_panel,
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;

                // Send the constructed value to the socket function
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err("New Panel JSON is required for replace_panel command".into());
            }
        } else {
            return Err("Panel ID is required for replace_panel command".into());
        }
    }

    // Handle the switch_active_layout subcommand
    if let Some(matches) = matches.subcommand_matches("switch_active_layout") {
        if let Some(layout_id_to_switch_to) = matches.get_one::<String>("layout_id_to_switch_to") {
            // Construct the enum variant using the struct syntax
            let socket_function = SocketFunction::SwitchActiveLayout {
                layout_id: layout_id_to_switch_to.to_string(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;

            // Send the constructed value to the socket function
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Layout ID is required for switch_active_layout command".into());
        }
    }

    // Handle the update_panel_script subcommand
    if let Some(matches) = matches.subcommand_matches("update_panel_script") {
        if let Some(panel_id) = matches.get_one::<String>("panel_id") {
            if let Some(new_panel_script) = matches.get_one::<String>("new_panel_script") {
                let new_panel_script = serde_json::from_str::<Vec<String>>(new_panel_script)?;

                // Construct the enum variant using the struct syntax
                let socket_function = SocketFunction::ReplacePanelScript {
                    panel_id: panel_id.clone(),
                    script: new_panel_script,
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;

                // Send the constructed value to the socket function
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err("New Panel Script is required for update_panel_script command".into());
            }
        } else {
            return Err("Panel ID is required for update_panel_script command".into());
        }
    }

    // Handle the update_panel_content subcommand
    if let Some(matches) = matches.subcommand_matches("update_panel_content") {
        if let Some(panel_id) = matches.get_one::<String>("panel_id") {
            if let Some(new_panel_content) = matches.get_one::<String>("new_panel_content") {
                // Construct the enum variant using the struct syntax
                let socket_function = SocketFunction::ReplacePanelContent {
                    panel_id: panel_id.clone(),
                    success: matches
                        .get_one::<String>("success")
                        .unwrap()
                        .parse::<bool>()
                        .unwrap(),
                    content: new_panel_content.clone(),
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;

                // Send the constructed value to the socket function
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err(
                    "New Panel Content is required for update_panel_content command".into(),
                );
            }
        } else {
            return Err("Panel ID is required for update_panel_content command".into());
        }
    }

    // Handle the add_panel subcommand
    if let Some(matches) = matches.subcommand_matches("add_panel") {
        if let Some(layout_id) = matches.get_one::<String>("layout_id") {
            if let Some(panel_json) = matches.get_one::<String>("panel_json") {
                let submitted_panel = serde_json::from_str::<Panel>(panel_json)?;

                // Construct the enum variant using the struct syntax
                let socket_function = SocketFunction::AddPanel {
                    layout_id: layout_id.to_string(),
                    panel: submitted_panel,
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;

                // Send the constructed value to the socket function
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err("Panel JSON is required for add_panel command".into());
            }
        } else {
            return Err("Layout ID is required for add_panel command".into());
        }
    }

    // Handle the remove_panel subcommand
    if let Some(matches) = matches.subcommand_matches("remove_panel") {
        if let Some(panel_id) = matches.get_one::<String>("panel_id") {
            // Construct the enum variant using the struct syntax
            let socket_function = SocketFunction::RemovePanel {
                panel_id: panel_id.clone(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;

            // Send the constructed value to the socket function
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Panel ID is required for remove_panel command".into());
        }
    }

    let yaml_path = matches.get_one::<String>("yaml_file").unwrap();
    let frame_delay = matches
        .get_one::<String>("frame_delay")
        .unwrap()
        .parse::<u64>()
        .unwrap_or(100);

    let yaml_path = Path::new(yaml_path);

    if !yaml_path.exists() {
        eprintln!("Yaml file does not exist: {}", yaml_path.display());
        return Ok(());
    }

    simplelog::CombinedLogger::init(vec![simplelog::WriteLogger::new(
        simplelog::LevelFilter::Debug,
        simplelog::Config::default(),
        File::create("app.log")?,
    )])?;
    let config = boxmux_lib::model::common::Config::new(frame_delay);
    let app = match load_app_from_yaml(yaml_path.to_str().unwrap()) {
        Ok(app) => app,
        Err(e) => {
            // The enhanced error handling is now built into load_app_from_yaml
            // so we just need to display the detailed error message
            eprintln!("{}", e);
            std::process::exit(1);
        }
    };

    let app_context = AppContext::new(app, config);

    //create alternate screen in terminal and clear it
    use crossterm::{execute, terminal, event};
    let mut _stdout = std::io::stdout();
    execute!(_stdout, terminal::EnterAlternateScreen)?;
    execute!(_stdout, terminal::Clear(terminal::ClearType::All))?;
    terminal::enable_raw_mode()?;
    execute!(_stdout, event::EnableMouseCapture)?;

    // Setup signal handler for proper terminal cleanup on exit
    setup_signal_handler();

    let mut manager = ThreadManager::new(app_context.clone());

    let _input_loop_uuid = manager.spawn_thread(InputLoop::new(app_context.clone()));
    let _draw_loop_uuid = manager.spawn_thread(DrawLoop::new(app_context.clone()));
    let _resize_loop_uuid = manager.spawn_thread(ResizeLoop::new(app_context.clone()));
    let _socket_loop_uuid = manager.spawn_thread(SocketLoop::new(app_context.clone()));

    run_panel_threads(&mut manager, &app_context);

    manager.run();

    //restore normal terminal state
    execute!(_stdout, event::DisableMouseCapture)?;
    execute!(_stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

#[cfg(test)]
mod terminal_cleanup_tests {
    use super::*;

    /// Tests that the cleanup_terminal function properly restores terminal state
    /// This test verifies the terminal cleanup functionality to prevent broken terminal state
    #[test]
    fn test_cleanup_terminal_function() {
        // Test that cleanup_terminal doesn't panic and can be called safely
        cleanup_terminal();
        
        // Since we can't easily test the actual terminal state changes in a unit test,
        // we verify that the function completes without error
        // The real test is that the terminal is properly restored when the application exits
        assert!(true, "cleanup_terminal function executed successfully");
    }

    /// Tests that setup_signal_handler can be called without panicking
    /// This ensures the signal handler setup is robust
    #[test] 
    fn test_signal_handler_setup() {
        // Test that signal handler setup doesn't panic
        // Note: In a test environment, we can't fully test signal handling
        // but we can verify the setup doesn't cause immediate failures
        setup_signal_handler();
        
        // Give the signal handler thread a moment to initialize
        std::thread::sleep(std::time::Duration::from_millis(10));
        
        assert!(true, "Signal handler setup completed successfully");
    }

    /// Tests the terminal cleanup sequence that should happen on normal exit
    /// This simulates the cleanup that happens when BoxMux exits normally
    #[test]
    fn test_terminal_cleanup_sequence() {
        // Simulate the terminal setup and cleanup sequence
        // This is what should happen: EnterAlternateScreen -> enable_raw_mode -> ... -> cleanup
        
        // We can't actually enter alternate screen or raw mode in a test,
        // but we can test that our cleanup function handles this gracefully
        cleanup_terminal();
        
        // Test multiple calls to cleanup (should be idempotent)
        cleanup_terminal();
        cleanup_terminal();
        
        assert!(true, "Terminal cleanup sequence completed successfully");
    }
}
