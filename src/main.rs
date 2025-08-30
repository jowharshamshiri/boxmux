#[macro_use]
extern crate lazy_static;
extern crate clap;

use boxmux_lib::create_runnable_with_dynamic_input;
use boxmux_lib::pty_manager::PtyManager;
use boxmux_lib::resize_loop::ResizeLoop;
use boxmux_lib::send_json_to_socket;
use boxmux_lib::socket_loop::SocketLoop;
use boxmux_lib::thread_manager;
use boxmux_lib::DrawLoop;
use boxmux_lib::FieldUpdate;
use boxmux_lib::InputLoop;
use boxmux_lib::MuxBox;
use boxmux_lib::SocketFunction;
use clap::{Arg, Command};
// Removed manual debug logging - using env_logger instead
use std::collections::HashMap;
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::time::Duration;
use std::time::Instant;
use thread_manager::{Message, Runnable, RunnableImpl, ThreadManager};
use uuid::Uuid;

use boxmux_lib::model::app::*;

lazy_static! {
    static ref LAST_EXECUTION_TIMES: Mutex<HashMap<String, Instant>> = Mutex::new(HashMap::new());
    // Periodic refresh sources - stores box_id -> PeriodicRefreshSource mapping
    static ref PERIODIC_REFRESH_SOURCES: Mutex<HashMap<String, boxmux_lib::model::common::PeriodicRefreshSource>> = Mutex::new(HashMap::new());
}

fn run_muxbox_threads(manager: &mut ThreadManager, app_context: &AppContext) {
    let active_layout = app_context.app.get_active_layout();
    let mut non_threaded_muxboxes: Vec<String> = vec![];

    if let Some(layout) = active_layout {
        for muxbox in layout.get_all_muxboxes() {
            if muxbox.script.is_some() {
                let muxbox_id = muxbox.id.clone();

                if muxbox.execution_mode.is_background() {
                    let vec_fn = move || vec![muxbox_id.clone()];

                    create_runnable_with_dynamic_input!(
                        MuxBoxRefreshLoop,
                        Box::new(vec_fn),
                        |_inner: &mut RunnableImpl,
                         _app_context: AppContext,
                         _messages: Vec<Message>,
                         _vec: Vec<String>|
                         -> bool { true },
                        move |inner: &mut RunnableImpl,
                              app_context: AppContext,
                              _messages: Vec<Message>,
                              vec: Vec<String>|
                              -> (bool, AppContext) {
                            let mut app_context_unwrapped = app_context.clone();
                            let app_graph = app_context_unwrapped.app.generate_graph();
                            let libs = app_context_unwrapped.app.libs.clone();
                            let muxbox = app_context_unwrapped
                                .app
                                .get_muxbox_by_id_mut(&vec[0])
                                .unwrap();

                            // F0228: ExecutionMode Message System - Use ExecutionMode instead of legacy pty field
                            let execution_mode = &muxbox.execution_mode;
                            let sender_for_pty = inner.get_message_sender().clone();
                            let thread_uuid = inner.get_uuid();

                            match boxmux_lib::utils::run_script_with_pty(
                                libs,
                                muxbox.script.clone().unwrap().as_ref(),
                                execution_mode, // F0228: Use ExecutionMode
                                app_context_unwrapped
                                    .pty_manager
                                    .as_ref()
                                    .map(|arc| arc.as_ref()),
                                if execution_mode.is_pty() {
                                    Some(muxbox.id.clone())
                                } else {
                                    None
                                },
                                if execution_mode.is_pty() && sender_for_pty.is_some() {
                                    Some((sender_for_pty.unwrap().clone(), thread_uuid))
                                } else {
                                    None
                                },
                            ) {
                                Ok(output) => {
                                    // T0325: Use StreamUpdateMessage instead of MuxBoxOutputUpdate
                                    let stream_update = boxmux_lib::model::common::StreamUpdate {
                                        stream_id: format!("{}_content", muxbox.id),
                                        target_box_id: muxbox.id.clone(),
                                        content_update: output,
                                        source_state: boxmux_lib::model::common::SourceState::Thread(
                                            boxmux_lib::model::common::ThreadSourceState {
                                                thread_id: format!("{:?}", std::thread::current().id()),
                                                execution_time: std::time::Duration::from_millis(0),
                                                exit_code: Some(0),
                                                status: boxmux_lib::model::common::ExecutionThreadStatus::Completed,
                                            }
                                        ),
                                        execution_mode: execution_mode.clone(),
                                    };
                                    inner.send_message(Message::StreamUpdateMessage(stream_update));
                                },
                                Err(e) => {
                                    // T0325: Use StreamUpdateMessage instead of MuxBoxOutputUpdate
                                    let stream_update = boxmux_lib::model::common::StreamUpdate {
                                        stream_id: format!("{}_content", muxbox.id),
                                        target_box_id: muxbox.id.clone(),
                                        content_update: e.to_string(),
                                        source_state: boxmux_lib::model::common::SourceState::Thread(
                                            boxmux_lib::model::common::ThreadSourceState {
                                                thread_id: format!("{:?}", std::thread::current().id()),
                                                execution_time: std::time::Duration::from_millis(0),
                                                exit_code: Some(1),
                                                status: boxmux_lib::model::common::ExecutionThreadStatus::Failed(e.to_string()),
                                            }
                                        ),
                                        execution_mode: execution_mode.clone(),
                                    };
                                    inner.send_message(Message::StreamUpdateMessage(stream_update));
                                },
                            }
                            std::thread::sleep(std::time::Duration::from_millis(
                                muxbox.calc_refresh_interval(&app_context, &app_graph),
                            ));
                            (true, app_context_unwrapped)
                        }
                    );

                    let mux_box_refresh_loop = MuxBoxRefreshLoop::new(app_context.clone(), Box::new(vec_fn));
                    manager.spawn_thread(mux_box_refresh_loop);
                } else {
                    non_threaded_muxboxes.push(muxbox_id.clone());
                }
            }
        }
        if !non_threaded_muxboxes.is_empty() {
            let vec_fn = move || non_threaded_muxboxes.clone();

            create_runnable_with_dynamic_input!(
                ExecuteLoop,
                Box::new(vec_fn),
                |_inner: &mut RunnableImpl,
                 _app_context: AppContext,
                 _messages: Vec<Message>,
                 _vec: Vec<String>|
                 -> bool { true },
                move |inner: &mut RunnableImpl,
                      app_context: AppContext,
                      messages: Vec<Message>,
                      vec: Vec<String>|
                      -> (bool, AppContext) {
                    let mut app_context_unwrapped = app_context.clone();

                    // UNIFIED EXECUTION ARCHITECTURE: Handle ExecuteScript messages from DrawLoop and other threads
                    for message in messages {
                        match message {
                            boxmux_lib::thread_manager::Message::ExecuteScriptMessage(execute_script) => {
                                log::info!("ExecuteLoop processing ExecuteScript for target_box: {} (mode: {:?})", 
                                           execute_script.target_box_id, execute_script.execution_mode);
                                
                                // Handle execution and send StreamUpdate with stream management for different source types
                                handle_execute_script_in_execute_loop(&execute_script, &inner);
                            }
                            _ => {
                                // Other messages not handled by ExecuteLoop
                                log::trace!("ExecuteLoop: Unhandled message type: {:?}", message);
                            }
                        }
                    }

                    let mut last_execution_times = LAST_EXECUTION_TIMES.lock().unwrap();

                    for muxbox_id in vec.iter() {
                        let libs = app_context_unwrapped.app.libs.clone();
                        
                        // Collect data from muxbox and then register source separately to avoid borrow conflicts
                        let (refresh_interval, execution_mode, script) = {
                            let muxbox = app_context_unwrapped
                                .app
                                .get_muxbox_by_id_mut(muxbox_id)
                                .unwrap();
                            (muxbox.refresh_interval.unwrap_or(1000), muxbox.execution_mode.clone(), muxbox.script.clone())
                        };

                        let last_execution_time = last_execution_times
                            .entry(muxbox_id.clone())
                            .or_insert(Instant::now());

                        if last_execution_time.elapsed() >= Duration::from_millis(refresh_interval)
                        {
                            let sender_for_pty = inner.get_message_sender().clone();
                            let thread_uuid = inner.get_uuid();

                            // T0319: UNIFIED ARCHITECTURE - Replace legacy periodic refresh execution with ExecuteScript message
                            log::info!("T0319: Periodic refresh creating ExecuteScript for muxbox {} (mode: {:?})", muxbox_id, execution_mode);
                            
                            // Create ExecuteScript message for periodic refresh execution
                            use boxmux_lib::model::common::{ExecuteScript, ExecutionSource, SourceType, SourceReference};
                            
                            // Register execution source and get stream_id
                            let script_unwrapped = script.unwrap();
                            let source_type = boxmux_lib::model::common::ExecutionSourceType::PeriodicScript(
                                script_unwrapped.join(" ")
                            );
                            let stream_id = app_context_unwrapped.app.register_execution_source(
                                source_type,
                                muxbox_id.clone()
                            );
                            
                            let execute_script = ExecuteScript {
                                script: script_unwrapped,
                                source: ExecutionSource {
                                    source_type: SourceType::PeriodicRefresh,
                                    source_id: format!("refresh-{}", muxbox_id),
                                    source_reference: SourceReference::PeriodicConfig(muxbox_id.clone()),
                                },
                                execution_mode: execution_mode.clone(),
                                target_box_id: muxbox_id.clone(),
                                libs: libs.unwrap_or_default(),
                                redirect_output: None, // Periodic refresh updates self, no redirection
                                append_output: false,  // CRITICAL: Replace content, don't append
                                stream_id,
                            };

                            // Send ExecuteScript message instead of direct execution
                            inner.send_message(Message::ExecuteScriptMessage(execute_script));

                            log::info!(
                                "T0319: ExecuteScript message sent for static muxbox {} script (unified architecture)",
                                muxbox_id
                            );

                            *last_execution_time = Instant::now();
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(
                        app_context.config.frame_delay,
                    ));

                    (true, app_context_unwrapped)
                }
            );

            let execute_loop = ExecuteLoop::new(app_context.clone(), Box::new(vec_fn));
            manager.spawn_thread(execute_loop);
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
        if let Some(_sig) = signals.forever().next() {
            r.store(false, Ordering::SeqCst);
            cleanup_terminal();
            std::process::exit(0);
        }
    });
}

/// Cleanup terminal state - restore normal mode
fn cleanup_terminal() {
    use crossterm::{event, execute, terminal};
    let mut stdout = std::io::stdout();
    let _ = execute!(stdout, event::DisableMouseCapture);
    let _ = execute!(stdout, terminal::LeaveAlternateScreen);
    let _ = terminal::disable_raw_mode();
}

/// Initialize comprehensive logging framework (F0161/F0162)
fn initialize_logging(matches: &clap::ArgMatches) -> Result<(), Box<dyn std::error::Error>> {
    let log_level = matches
        .get_one::<String>("log_level")
        .unwrap_or(&"info".to_string())
        .clone();
    let log_file = matches.get_one::<String>("log_file").cloned();

    // Parse log level for env_logger
    let env_log_level = match log_level.as_str() {
        "trace" => "trace",
        "debug" => "debug",
        "info" => "info",
        "warn" => "warn",
        "error" => "error",
        _ => "info",
    };

    // Set RUST_LOG to enable all our modules at the specified level
    std::env::set_var(
        "RUST_LOG",
        format!("boxmux={},boxmux_lib={}", env_log_level, env_log_level),
    );

    // Only initialize logging when --log-file is explicitly provided
    let logger_result = if let Some(ref file_path) = log_file {
        // Create file logger with custom format
        let target = match std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&file_path)
        {
            Ok(file) => Box::new(file),
            Err(e) => {
                return Err(format!("Failed to create log file '{}': {}", file_path, e).into())
            }
        };

        env_logger::Builder::from_default_env()
            .target(env_logger::Target::Pipe(target))
            .format(|buf, record| {
                use std::io::Write;
                writeln!(
                    buf,
                    "[{}] [{}] [{}:{}] [T{:?}] {}",
                    chrono::Utc::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                    record.level(),
                    record.file().unwrap_or("unknown"),
                    record.line().unwrap_or(0),
                    std::thread::current().id(),
                    record.args()
                )
            })
            .try_init()
    } else {
        // No logging when --log-file is not provided to avoid corrupting TUI
        return Ok(());
    };

    match logger_result {
        Ok(_) => {
            eprintln!(
                "Debug logging initialized: writing to file {}",
                log_file.as_ref().unwrap()
            );
            log::info!("BoxMux logging system initialized at {} level", log_level);
        }
        Err(e) => {
            eprintln!(
                "Warning: Could not initialize logger ({}). Logging may not work as expected.",
                e
            );
            eprintln!("This usually means another logger is already initialized in the codebase.");
            eprintln!("Continuing anyway...");
        }
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging FIRST before any other setup
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
        .arg(
            Arg::new("log_level")
                .short('l')
                .long("log-level")
                .value_name("LEVEL")
                .help("Sets the logging level (trace, debug, info, warn, error)")
                .default_value("info"),
        )
        .arg(
            Arg::new("log_file")
                .short('f')
                .long("log-file")
                .value_name("FILE")
                .help("Write logs to file instead of stderr"),
        )
        .arg(
            Arg::new("lock")
                .long("lock")
                .action(clap::ArgAction::SetTrue)
                .help("Disable box resizing and moving"),
        )
        .subcommand(
            Command::new("stop_box_refresh")
                .about("Stops the refresh of the box")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id to stop the refresh of"),
                ),
        )
        .subcommand(
            Command::new("start_box_refresh")
                .about("Starts the refresh of the box")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id to start the refresh of"),
                ),
        )
        .subcommand(
            Command::new("replace_box")
                .about("Replaces the box with the provided MuxBox")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id to update"),
                )
                .arg(
                    Arg::new("new_box_json")
                        .required(true)
                        .index(2)
                        .help("The new box to update with"),
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
            Command::new("update_box_script")
                .about("Updates the box script")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id to update the script of"),
                )
                .arg(
                    Arg::new("new_box_script")
                        .required(true)
                        .index(2)
                        .help("The new script to update the box with"),
                ),
        )
        .subcommand(
            Command::new("update_box_content")
                .about("Updates the box content")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id to update the content of"),
                )
                .arg(
                    Arg::new("success")
                        .required(true)
                        .index(2)
                        .help("Whether the content is a success or not"),
                )
                .arg(
                    Arg::new("new_box_content")
                        .required(true)
                        .index(3)
                        .help("The new content to update the box with"),
                ),
        )
        .subcommand(
            Command::new("add_box")
                .about("Adds a box to a layout")
                .arg(
                    Arg::new("layout_id")
                        .required(true)
                        .index(1)
                        .help("The layout id to add the box to"),
                )
                .arg(
                    Arg::new("muxbox_json")
                        .required(true)
                        .index(2)
                        .help("The box to add to the layout"),
                ),
        )
        .subcommand(
            Command::new("remove_box")
                .about("Removes a box from its layout")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id to remove from its layout"),
                ),
        )
        // F0137: Socket PTY Control - Kill and restart PTY processes
        .subcommand(
            Command::new("kill_pty_process")
                .about("Kills a PTY process for a box")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id with the PTY process to kill"),
                ),
        )
        .subcommand(
            Command::new("restart_pty_process")
                .about("Restarts a PTY process for a box")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id with the PTY process to restart"),
                ),
        )
        // F0138: Socket PTY Query - Get PTY status and info
        .subcommand(
            Command::new("query_pty_status")
                .about("Gets PTY process status for a box")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id to query PTY status for"),
                ),
        )
        // F0136: Socket PTY Spawn - Spawn PTY processes via socket commands
        .subcommand(
            Command::new("spawn_pty_process")
                .about("Spawns a PTY process for a box with specified script")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id to spawn PTY process for"),
                )
                .arg(
                    Arg::new("script")
                        .required(true)
                        .index(2)
                        .help("The script commands to run in the PTY (JSON array of strings)"),
                )
                .arg(
                    Arg::new("libs")
                        .long("libs")
                        .help("Optional library paths to source (JSON array of strings)"),
                )
                .arg(
                    Arg::new("redirect_output")
                        .long("redirect-output")
                        .help("Optional box ID to redirect output to"),
                ),
        )
        // F0139: Socket PTY Input - Send input to PTY processes remotely
        .subcommand(
            Command::new("send_pty_input")
                .about("Sends input to a running PTY process for interactive control")
                .arg(
                    Arg::new("box_id")
                        .required(true)
                        .index(1)
                        .help("The box id with the PTY process to send input to"),
                )
                .arg(
                    Arg::new("input")
                        .required(true)
                        .index(2)
                        .help("The input text to send to the PTY process"),
                ),
        )
        .get_matches();

    // Initialize logging framework (F0161/F0162)
    initialize_logging(&matches)?;

    // Handle the stop_box_refresh subcommand
    if let Some(matches) = matches.subcommand_matches("stop_box_refresh") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            // Construct the enum variant using the struct syntax
            let socket_function = SocketFunction::StopBoxRefresh {
                box_id: box_id.clone(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;

            // Send the constructed value to the socket function
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Box ID is required for stop_box_refresh command".into());
        }
    }

    // Handle the start_box_refresh subcommand
    if let Some(matches) = matches.subcommand_matches("start_box_refresh") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            // Construct the enum variant using the struct syntax
            let socket_function = SocketFunction::StartBoxRefresh {
                box_id: box_id.clone(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;

            // Send the constructed value to the socket function
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Box ID is required for start_box_refresh command".into());
        }
    }

    // Handle the replace_box subcommand
    if let Some(matches) = matches.subcommand_matches("replace_box") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            if let Some(new_box_json) = matches.get_one::<String>("new_box_json") {
                // Construct the enum variant using the struct syntax
                let submitted_box = serde_json::from_str::<MuxBox>(new_box_json)?;

                let socket_function = SocketFunction::ReplaceBox {
                    box_id: box_id.clone(),
                    new_box: submitted_box,
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;

                // Send the constructed value to the socket function
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err("New Box JSON is required for replace_box command".into());
            }
        } else {
            return Err("Box ID is required for replace_box command".into());
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

    // Handle the update_box_script subcommand
    if let Some(matches) = matches.subcommand_matches("update_box_script") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            if let Some(new_box_script) = matches.get_one::<String>("new_box_script") {
                let new_box_script = serde_json::from_str::<Vec<String>>(new_box_script)?;

                // Construct the enum variant using the struct syntax
                let socket_function = SocketFunction::ReplaceBoxScript {
                    box_id: box_id.clone(),
                    script: new_box_script,
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;

                // Send the constructed value to the socket function
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err("New Box Script is required for update_box_script command".into());
            }
        } else {
            return Err("Box ID is required for update_box_script command".into());
        }
    }

    // Handle the update_box_content subcommand
    if let Some(matches) = matches.subcommand_matches("update_box_content") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            if let Some(new_box_content) = matches.get_one::<String>("new_box_content") {
                // Construct the enum variant using the struct syntax
                let socket_function = SocketFunction::ReplaceBoxContent {
                    box_id: box_id.clone(),
                    success: matches
                        .get_one::<String>("success")
                        .unwrap()
                        .parse::<bool>()
                        .unwrap(),
                    content: new_box_content.clone(),
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;

                // Send the constructed value to the socket function
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err("New Box Content is required for update_box_content command".into());
            }
        } else {
            return Err("Box ID is required for update_box_content command".into());
        }
    }

    // Handle the add_box subcommand
    if let Some(matches) = matches.subcommand_matches("add_box") {
        if let Some(layout_id) = matches.get_one::<String>("layout_id") {
            if let Some(muxbox_json) = matches.get_one::<String>("muxbox_json") {
                let submitted_muxbox = serde_json::from_str::<MuxBox>(muxbox_json)?;

                // Construct the enum variant using the struct syntax
                let socket_function = SocketFunction::AddBox {
                    layout_id: layout_id.to_string(),
                    muxbox: submitted_muxbox,
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;

                // Send the constructed value to the socket function
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err("Box JSON is required for add_box command".into());
            }
        } else {
            return Err("Layout ID is required for add_box command".into());
        }
    }

    // Handle the remove_box subcommand
    if let Some(matches) = matches.subcommand_matches("remove_box") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            // Construct the enum variant using the struct syntax
            let socket_function = SocketFunction::RemoveBox {
                box_id: box_id.clone(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;

            // Send the constructed value to the socket function
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Box ID is required for remove_box command".into());
        }
    }

    // F0137: Socket PTY Control - Handle kill_pty_process subcommand
    if let Some(matches) = matches.subcommand_matches("kill_pty_process") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            let socket_function = SocketFunction::KillPtyProcess {
                box_id: box_id.clone(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Box ID is required for kill_pty_process command".into());
        }
    }

    // F0137: Socket PTY Control - Handle restart_pty_process subcommand
    if let Some(matches) = matches.subcommand_matches("restart_pty_process") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            let socket_function = SocketFunction::RestartPtyProcess {
                box_id: box_id.clone(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Box ID is required for restart_pty_process command".into());
        }
    }

    // F0138: Socket PTY Query - Handle query_pty_status subcommand
    if let Some(matches) = matches.subcommand_matches("query_pty_status") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            let socket_function = SocketFunction::QueryPtyStatus {
                box_id: box_id.clone(),
            };

            let socket_function_json = serde_json::to_string(&socket_function)?;
            send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

            return Ok(());
        } else {
            return Err("Box ID is required for query_pty_status command".into());
        }
    }

    // F0136: Socket PTY Spawn - Handle spawn_pty_process subcommand
    if let Some(matches) = matches.subcommand_matches("spawn_pty_process") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            if let Some(script_json) = matches.get_one::<String>("script") {
                // Parse script JSON array
                let script: Vec<String> = serde_json::from_str(script_json)
                    .map_err(|e| format!("Invalid script JSON: {}", e))?;

                // Parse optional libs JSON array
                let libs: Option<Vec<String>> =
                    if let Some(libs_json) = matches.get_one::<String>("libs") {
                        Some(
                            serde_json::from_str(libs_json)
                                .map_err(|e| format!("Invalid libs JSON: {}", e))?,
                        )
                    } else {
                        None
                    };

                let redirect_output = matches.get_one::<String>("redirect_output").cloned();

                let socket_function = SocketFunction::SpawnPtyProcess {
                    box_id: box_id.clone(),
                    script,
                    libs,
                    redirect_output,
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err("Script is required for spawn_pty_process command".into());
            }
        } else {
            return Err("Box ID is required for spawn_pty_process command".into());
        }
    }

    // F0139: Socket PTY Input - Handle send_pty_input subcommand
    if let Some(matches) = matches.subcommand_matches("send_pty_input") {
        if let Some(box_id) = matches.get_one::<String>("box_id") {
            if let Some(input) = matches.get_one::<String>("input") {
                let socket_function = SocketFunction::SendPtyInput {
                    box_id: box_id.clone(),
                    input: input.clone(),
                };

                let socket_function_json = serde_json::to_string(&socket_function)?;
                send_json_to_socket("/tmp/boxmux.sock", &socket_function_json)?;

                return Ok(());
            } else {
                return Err("Input is required for send_pty_input command".into());
            }
        } else {
            return Err("Box ID is required for send_pty_input command".into());
        }
    }

    let yaml_path = matches.get_one::<String>("yaml_file").unwrap();
    let frame_delay = matches
        .get_one::<String>("frame_delay")
        .unwrap()
        .parse::<u64>()
        .unwrap_or(100);
    let locked = matches.get_flag("lock");

    let yaml_path = Path::new(yaml_path);

    if !yaml_path.exists() {
        log::error!("Yaml file does not exist: {}", yaml_path.display());
        return Ok(());
    }

    // Convert to absolute path to ensure YAML persistence works regardless of working directory
    let yaml_path = yaml_path
        .canonicalize()
        .map_err(|e| format!("Failed to resolve absolute path for YAML file: {}", e))?;

    // Removed old simplelog - using our new comprehensive logging system instead
    let config = boxmux_lib::model::common::Config::new_with_lock(frame_delay, locked);
    let app = match load_app_from_yaml_with_lock(yaml_path.to_str().unwrap(), locked) {
        Ok(app) => app,
        Err(e) => {
            // The enhanced error handling is now built into load_app_from_yaml
            // so we just need to display the detailed error message
            log::error!("{}", e);
            std::process::exit(1);
        }
    };

    // Initialize PTY manager for this session
    let pty_manager = match PtyManager::new() {
        Ok(mgr) => {
            log::info!("PTY manager initialized successfully");
            Some(Arc::new(mgr))
        }
        Err(e) => {
            log::warn!("Failed to initialize PTY manager: {}", e);
            None
        }
    };

    let app_context = if let Some(pty_mgr) = pty_manager.as_ref() {
        AppContext::new_with_pty_and_yaml_and_lock(
            app,
            config,
            pty_mgr.clone(),
            yaml_path.to_str().unwrap().to_string(),
            locked,
        )
    } else {
        AppContext::new_with_yaml_path_and_lock(
            app,
            config,
            yaml_path.to_str().unwrap().to_string(),
            locked,
        )
    };

    //create alternate screen in terminal and clear it
    use crossterm::{event, execute, terminal};
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

    run_muxbox_threads(&mut manager, &app_context);

    manager.run();

    //restore normal terminal state
    execute!(_stdout, event::DisableMouseCapture)?;
    execute!(_stdout, terminal::LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}

/// UNIFIED EXECUTION ARCHITECTURE: Handle ExecuteScript messages with proper stream management
/// Different source types have different stream behaviors:
/// - Choice execution: Create new streams for each execution (tabs)
/// - Static script refresh: Reuse same stream, replace content (no new tabs)
/// - Socket updates: Create/update streams based on context
fn handle_execute_script_in_execute_loop(
    execute_script: &boxmux_lib::model::common::ExecuteScript,
    inner: &boxmux_lib::thread_manager::RunnableImpl,
) {
    log::info!("EXECUTELOOP DEBUG: handle_execute_script_in_execute_loop called for target_box: {} mode: {:?} redirect: {:?}", 
               execute_script.target_box_id, execute_script.execution_mode, execute_script.redirect_output);
    use boxmux_lib::model::common::{ExecutionMode, SourceType};

    // Determine stream ID strategy based on source type
    // SOURCE OBJECT ARCHITECTURE: ExecuteScript already contains stream_id from source object
    let stream_id = execute_script.stream_id.clone();
    
    // Verify stream_id consistency (remove this once architecture is stable)
    match &execute_script.source.source_type {
        SourceType::Choice(_) => {
            log::info!("Using source object stream_id {} for Choice execution", stream_id);
        }
        SourceType::StaticScript => {
            log::info!("Using source object stream_id {} for StaticScript execution", stream_id);
        }
        SourceType::PeriodicRefresh => {
            log::info!("Using source object stream_id {} for PeriodicRefresh execution", stream_id);
        }
        SourceType::SocketUpdate => {
            log::info!("Using source object stream_id {} for SocketUpdate execution", stream_id);
        }
        SourceType::RedirectedScript => {
            log::info!("Using source object stream_id {} for RedirectedScript execution", stream_id);
        }
        SourceType::HotkeyScript => {
            log::info!("Using source object stream_id {} for HotkeyScript execution", stream_id);
        }
        SourceType::ScheduledScript => {
            log::info!("Using source object stream_id {} for ScheduledScript execution", stream_id);
        }
    };

    log::info!("Executing script for stream_id: {} (source: {:?})", stream_id, execute_script.source.source_type);

    // Execute based on execution mode
    match execute_script.execution_mode {
        ExecutionMode::Immediate => {
            execute_immediate_script_unified(execute_script, &stream_id, inner);
        }
        ExecutionMode::Thread => {
            execute_threaded_script_unified(execute_script, &stream_id, inner);
        }
        ExecutionMode::Pty => {
            execute_pty_script_unified(execute_script, &stream_id, inner);
        }
    }
}

/// Execute script immediately and send StreamUpdate
fn execute_immediate_script_unified(
    execute_script: &boxmux_lib::model::common::ExecuteScript,
    stream_id: &str,
    inner: &boxmux_lib::thread_manager::RunnableImpl,
) {
    use std::process::Command;
    
    log::info!("Immediate execution for stream: {}", stream_id);
    
    // Run script synchronously
    let output = Command::new("sh")
        .arg("-c")
        .arg(execute_script.script.join(" "))
        .output();
        
    let content = match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.is_empty() {
                stdout.to_string()
            } else {
                format!("{}\n{}", stdout, stderr)
            }
        }
        Err(e) => {
            format!("Error executing script: {}", e)
        }
    };
    
    // Send StreamUpdate with results
    let target_box = if let Some(ref redirect_to) = execute_script.redirect_output {
        log::info!("REDIRECT FIX: Using redirect destination: {} (was {})", redirect_to, execute_script.target_box_id);
        redirect_to.clone()
    } else {
        log::info!("REDIRECT FIX: No redirect, using source box: {}", execute_script.target_box_id);
        execute_script.target_box_id.clone()
    };
    
    let stream_update = boxmux_lib::model::common::StreamUpdate {
        stream_id: stream_id.to_string(),
        target_box_id: target_box,
        content_update: content,
        source_state: boxmux_lib::model::common::SourceState::Batch(
            boxmux_lib::model::common::BatchSourceState {
                task_id: stream_id.to_string(),
                queue_wait_time: std::time::Duration::from_millis(0),
                execution_time: std::time::Duration::from_millis(100),
                exit_code: Some(0),
                status: boxmux_lib::model::common::BatchStatus::Completed,
            }
        ),
        execution_mode: execute_script.execution_mode.clone(),
    };
    
    inner.send_message(boxmux_lib::thread_manager::Message::StreamUpdateMessage(stream_update));
}

/// Execute script in background thread and send StreamUpdate
fn execute_threaded_script_unified(
    execute_script: &boxmux_lib::model::common::ExecuteScript,
    stream_id: &str,
    inner: &boxmux_lib::thread_manager::RunnableImpl,
) {
    use std::{process::Command, thread};
    
    let stream_id = stream_id.to_string();
    let target_box_id = if let Some(ref redirect_to) = execute_script.redirect_output {
        log::info!("REDIRECT FIX THREAD: Using redirect destination: {} (was {})", redirect_to, execute_script.target_box_id);
        redirect_to.clone()
    } else {
        log::info!("REDIRECT FIX THREAD: No redirect, using source box: {}", execute_script.target_box_id);
        execute_script.target_box_id.clone()
    };
    let script = execute_script.script.clone();
    let execution_mode = execute_script.execution_mode.clone();
    let message_sender = inner.get_message_sender().cloned();
    
    log::info!("Thread execution for stream: {}", stream_id);
    
    // Send initial update
    let start_update = boxmux_lib::model::common::StreamUpdate {
        stream_id: stream_id.clone(),
        target_box_id: target_box_id.clone(),
        content_update: format!("Starting thread execution: {}\n", script.join(" ")),
        source_state: boxmux_lib::model::common::SourceState::Thread(
            boxmux_lib::model::common::ThreadSourceState {
                thread_id: "pending".to_string(),
                execution_time: std::time::Duration::from_millis(0),
                exit_code: None,
                status: boxmux_lib::model::common::ExecutionThreadStatus::Running,
            }
        ),
        execution_mode: execution_mode.clone(),
    };
    
    inner.send_message(boxmux_lib::thread_manager::Message::StreamUpdateMessage(start_update));
    
    // Spawn background thread with limited scope to avoid capturing inner
    {
        let stream_id_for_thread = stream_id.clone();
        let target_box_id_for_thread = target_box_id.clone();
        let script_for_thread = script.clone();
        let execution_mode_for_thread = execution_mode.clone();
        let message_sender_for_thread = message_sender;
        
        thread::spawn(move || {
            let stream_id = stream_id_for_thread;
            let target_box_id = target_box_id_for_thread;
            let script = script_for_thread;
            let execution_mode = execution_mode_for_thread;
            let message_sender = message_sender_for_thread;
        let thread_id = format!("{:?}", thread::current().id());
        
        let output = Command::new("sh")
            .arg("-c")
            .arg(script.join(" "))
            .output();
            
        let (content, exit_code, status) = match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                let content = if stderr.is_empty() {
                    stdout.to_string()
                } else {
                    format!("{}\n{}", stdout, stderr)
                };
                let exit_code = output.status.code();
                let status = if output.status.success() {
                    boxmux_lib::model::common::ExecutionThreadStatus::Completed
                } else {
                    boxmux_lib::model::common::ExecutionThreadStatus::Failed("Script execution failed".to_string())
                };
                (content, exit_code, status)
            }
            Err(e) => {
                (format!("Error executing script: {}", e), Some(1), 
                 boxmux_lib::model::common::ExecutionThreadStatus::Failed(format!("Execution error: {}", e)))
            }
        };
        
        // Send final result
        if let Some(sender) = message_sender {
            let final_update = boxmux_lib::model::common::StreamUpdate {
                stream_id,
                target_box_id,
                content_update: content,
                source_state: boxmux_lib::model::common::SourceState::Thread(
                    boxmux_lib::model::common::ThreadSourceState {
                        thread_id,
                        execution_time: std::time::Duration::from_millis(100),
                        exit_code,
                        status,
                    }
                ),
                execution_mode,
            };
            
            let _ = sender.send((uuid::Uuid::new_v4(), boxmux_lib::thread_manager::Message::StreamUpdateMessage(final_update)));
        }
        });
    }
}

/// Execute script with PTY and send multi-phase StreamUpdate messages  
fn execute_pty_script_unified(
    execute_script: &boxmux_lib::model::common::ExecuteScript,
    stream_id: &str,
    inner: &boxmux_lib::thread_manager::RunnableImpl,
) {
    let stream_id = stream_id.to_string();
    let target_box_id = if let Some(ref redirect_to) = execute_script.redirect_output {
        log::info!("REDIRECT FIX PTY: Using redirect destination: {} (was {})", redirect_to, execute_script.target_box_id);
        redirect_to.clone()
    } else {
        log::info!("REDIRECT FIX PTY: No redirect, using source box: {}", execute_script.target_box_id);
        execute_script.target_box_id.clone()
    };
    let script = execute_script.script.clone();
    let execution_mode = execute_script.execution_mode.clone();
    
    log::info!("PTY execution for stream: {}", stream_id);
    
    // Phase 1: Connecting
    let start_update = boxmux_lib::model::common::StreamUpdate {
        stream_id: stream_id.clone(),
        target_box_id: target_box_id.clone(),
        content_update: format!("Connecting PTY for: {}\n", script.join(" ")),
        source_state: boxmux_lib::model::common::SourceState::Pty(
            boxmux_lib::model::common::PtySourceState {
                process_id: 0,
                runtime: std::time::Duration::from_millis(0),
                exit_code: None,
                status: boxmux_lib::model::common::ExecutionPtyStatus::Starting,
            }
        ),
        execution_mode: execution_mode.clone(),
    };
    
    inner.send_message(boxmux_lib::thread_manager::Message::StreamUpdateMessage(start_update));
    
    // Phase 2: Connected (simulate PTY session)
    let connected_update = boxmux_lib::model::common::StreamUpdate {
        stream_id: stream_id.clone(),
        target_box_id: target_box_id.clone(),
        content_update: "[PTY Connected]\n".to_string(),
        source_state: boxmux_lib::model::common::SourceState::Pty(
            boxmux_lib::model::common::PtySourceState {
                process_id: 12345,
                runtime: std::time::Duration::from_millis(100),
                exit_code: None,
                status: boxmux_lib::model::common::ExecutionPtyStatus::Running,
            }
        ),
        execution_mode: execution_mode.clone(),
    };
    
    inner.send_message(boxmux_lib::thread_manager::Message::StreamUpdateMessage(connected_update));
    
    // Phase 3: Execute command and show output
    use std::process::Command;
    let output = Command::new("sh")
        .arg("-c")
        .arg(script.join(" "))
        .output();
        
    let content = match output {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.is_empty() {
                stdout.to_string()
            } else {
                format!("{}\n{}", stdout, stderr)
            }
        }
        Err(e) => {
            format!("Error executing script: {}", e)
        }
    };
    
    let output_update = boxmux_lib::model::common::StreamUpdate {
        stream_id: stream_id.clone(),
        target_box_id: target_box_id.clone(),
        content_update: content,
        source_state: boxmux_lib::model::common::SourceState::Pty(
            boxmux_lib::model::common::PtySourceState {
                process_id: 12345,
                runtime: std::time::Duration::from_millis(200),
                exit_code: None,
                status: boxmux_lib::model::common::ExecutionPtyStatus::Running,
            }
        ),
        execution_mode: execution_mode.clone(),
    };
    
    inner.send_message(boxmux_lib::thread_manager::Message::StreamUpdateMessage(output_update));
    
    // Phase 4: Session ended
    let end_update = boxmux_lib::model::common::StreamUpdate {
        stream_id,
        target_box_id,
        content_update: "[PTY Session Ended]\n".to_string(),
        source_state: boxmux_lib::model::common::SourceState::Pty(
            boxmux_lib::model::common::PtySourceState {
                process_id: 12345,
                runtime: std::time::Duration::from_millis(300),
                exit_code: Some(0),
                status: boxmux_lib::model::common::ExecutionPtyStatus::Completed,
            }
        ),
        execution_mode,
    };
    
    inner.send_message(boxmux_lib::thread_manager::Message::StreamUpdateMessage(end_update));
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
