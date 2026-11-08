use boxmux_lib::model::yaml_edit::ConfigLock;
use boxmux_lib::platform;
use boxmux_lib::runtime::display::TerminalDisplay;
use boxmux_lib::runtime::event::Event;
use boxmux_lib::runtime::Runtime;
use boxmux_lib::socket::protocol::{BoxDefinition, Reply, SocketFunction};
use boxmux_lib::socket::{client, server::SocketServer};
use boxmux_lib::{AppContext, Config};
use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use std::path::PathBuf;
use std::process::ExitCode;

fn box_id_arg() -> Arg {
    Arg::new("box_id").required(true).help("The id of the box")
}

fn cli() -> Command {
    Command::new("boxmux")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Terminal dashboards and tools described in YAML")
        .subcommand_negates_reqs(true)
        .override_usage(
            "boxmux [OPTIONS] <config>\n       boxmux [--socket <NAME>] <COMMAND> [ARGS]",
        )
        .arg(
            Arg::new("config")
                .required(true)
                .value_parser(value_parser!(PathBuf))
                .help("The YAML file describing the interface"),
        )
        .arg(
            Arg::new("frame_delay")
                .short('d')
                .long("frame-delay")
                .value_name("MS")
                .value_parser(value_parser!(u64).range(1..))
                .default_value("30")
                .help("Minimum time between two redraws, in milliseconds"),
        )
        .arg(
            Arg::new("log_file")
                .long("log-file")
                .value_name("FILE")
                .value_parser(value_parser!(PathBuf))
                .help("Write a log to FILE"),
        )
        .arg(
            Arg::new("log_level")
                .long("log-level")
                .value_name("LEVEL")
                .value_parser(["error", "warn", "info", "debug", "trace"])
                .default_value("info")
                .requires("log_file")
                .help("How much to log"),
        )
        .arg(
            Arg::new("lock")
                .long("lock")
                .action(ArgAction::SetTrue)
                .help("Disable moving and resizing boxes; the YAML file is never written"),
        )
        .arg(
            Arg::new("calibrate")
                .long("calibrate")
                .action(ArgAction::SetTrue)
                .help("Highlight the cell under the mouse pointer"),
        )
        .arg(
            Arg::new("dark")
                .long("dark")
                .action(ArgAction::SetTrue)
                .conflicts_with("light")
                .help("Use the dark color theme regardless of the terminal"),
        )
        .arg(
            Arg::new("light")
                .long("light")
                .action(ArgAction::SetTrue)
                .help("Use the light color theme regardless of the terminal"),
        )
        .arg(
            Arg::new("socket")
                .long("socket")
                .value_name("NAME")
                .global(true)
                .help("The control socket (default: $BOXMUX_SOCKET, or a per-user default)"),
        )
        .subcommand(
            Command::new("replace_box_content")
                .about("Show text in a box's socket tab")
                .arg(box_id_arg())
                .arg(Arg::new("content").required(true))
                .arg(
                    Arg::new("failed")
                        .long("failed")
                        .action(ArgAction::SetTrue)
                        .help("Show the content in the box's error colors"),
                ),
        )
        .subcommand(
            Command::new("replace_box_script")
                .about("Replace a box's script and run it")
                .arg(box_id_arg())
                .arg(
                    Arg::new("script")
                        .required(true)
                        .num_args(1..)
                        .help("Script lines"),
                ),
        )
        .subcommand(
            Command::new("stop_box_refresh")
                .about("Pause a box's periodic refresh")
                .arg(box_id_arg()),
        )
        .subcommand(
            Command::new("start_box_refresh")
                .about("Resume a box's periodic refresh")
                .arg(box_id_arg()),
        )
        .subcommand(
            Command::new("replace_box")
                .about("Replace a box with the one described in JSON")
                .arg(box_id_arg())
                .arg(Arg::new("box_json").required(true)),
        )
        .subcommand(
            Command::new("add_box")
                .about("Add a box, described in JSON, to a layout")
                .arg(Arg::new("layout_id").required(true))
                .arg(Arg::new("box_json").required(true))
                .arg(
                    Arg::new("parent")
                        .long("parent")
                        .value_name("BOX_ID")
                        .help("Nest the new box in this box"),
                ),
        )
        .subcommand(
            Command::new("remove_box")
                .about("Remove a box")
                .arg(box_id_arg()),
        )
        .subcommand(
            Command::new("switch_active_layout")
                .about("Show another layout")
                .arg(Arg::new("layout_id").required(true)),
        )
        .subcommand(
            Command::new("spawn_pty_process")
                .about("Run a script in a terminal shown in a box")
                .arg(box_id_arg())
                .arg(
                    Arg::new("script")
                        .required(true)
                        .num_args(1..)
                        .help("Script lines"),
                )
                .arg(
                    Arg::new("redirect_output")
                        .long("redirect-output")
                        .value_name("BOX_ID")
                        .help("Show the terminal in this box instead"),
                ),
        )
        .subcommand(
            Command::new("kill_pty_process")
                .about("Stop the process in a box's visible tab")
                .arg(box_id_arg()),
        )
        .subcommand(
            Command::new("restart_pty_process")
                .about("Restart the process in a box's visible tab")
                .arg(box_id_arg()),
        )
        .subcommand(
            Command::new("query_pty_status")
                .about("Report on the process in a box's visible tab")
                .arg(box_id_arg()),
        )
        .subcommand(
            Command::new("send_pty_input")
                .about("Type input into the terminal in a box's visible tab")
                .arg(box_id_arg())
                .arg(Arg::new("input").required(true)),
        )
        .subcommand(
            Command::new("send_signal")
                .about("Send a signal to the process in a box's visible tab")
                .arg(box_id_arg())
                .arg(
                    Arg::new("signal")
                        .required(true)
                        .value_parser(value_parser!(i32).range(1..)),
                ),
        )
}

/// The options that only apply when showing an interface, not to control commands.
const INTERFACE_OPTIONS: [&str; 8] = [
    "config",
    "frame_delay",
    "log_file",
    "log_level",
    "lock",
    "calibrate",
    "dark",
    "light",
];

fn main() -> ExitCode {
    let matches = cli().get_matches();
    if let Some((name, _)) = matches.subcommand() {
        let misplaced: Vec<String> = INTERFACE_OPTIONS
            .iter()
            .filter(|id| matches.value_source(id) == Some(clap::parser::ValueSource::CommandLine))
            .map(|id| id.replace('_', "-"))
            .collect();
        if !misplaced.is_empty() {
            cli()
                .error(
                    clap::error::ErrorKind::ArgumentConflict,
                    format!(
                        "{} cannot be used with the control command '{}'",
                        misplaced.join(", "),
                        name
                    ),
                )
                .exit();
        }
    }
    let socket = platform::socket_name(matches.get_one::<String>("socket").map(String::as_str));
    let result = match matches.subcommand() {
        Some((name, args)) => control(&socket, name, args),
        None => run(&matches, &socket),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("boxmux: {}", message);
            ExitCode::FAILURE
        }
    }
}

fn arg(args: &ArgMatches, name: &str) -> String {
    args.get_one::<String>(name)
        .cloned()
        .expect("clap enforces required arguments")
}

fn lines(args: &ArgMatches, name: &str) -> Vec<String> {
    args.get_many::<String>(name)
        .expect("clap enforces required arguments")
        .cloned()
        .collect()
}

fn parse_box(json: &str) -> Result<BoxDefinition, String> {
    json.parse().map_err(|e| format!("invalid box: {}", e))
}

/// Sends a control command to a running instance and prints its reply.
fn control(socket: &str, name: &str, args: &ArgMatches) -> Result<(), String> {
    let function = match name {
        "replace_box_content" => SocketFunction::ReplaceBoxContent {
            box_id: arg(args, "box_id"),
            content: arg(args, "content"),
            success: !args.get_flag("failed"),
        },
        "replace_box_script" => SocketFunction::ReplaceBoxScript {
            box_id: arg(args, "box_id"),
            script: lines(args, "script"),
        },
        "stop_box_refresh" => SocketFunction::StopBoxRefresh {
            box_id: arg(args, "box_id"),
        },
        "start_box_refresh" => SocketFunction::StartBoxRefresh {
            box_id: arg(args, "box_id"),
        },
        "replace_box" => SocketFunction::ReplaceBox {
            box_id: arg(args, "box_id"),
            new_box: parse_box(&arg(args, "box_json"))?,
        },
        "add_box" => SocketFunction::AddBox {
            layout_id: arg(args, "layout_id"),
            parent_id: args.get_one::<String>("parent").cloned(),
            muxbox: parse_box(&arg(args, "box_json"))?,
        },
        "remove_box" => SocketFunction::RemoveBox {
            box_id: arg(args, "box_id"),
        },
        "switch_active_layout" => SocketFunction::SwitchActiveLayout {
            layout_id: arg(args, "layout_id"),
        },
        "spawn_pty_process" => SocketFunction::SpawnPtyProcess {
            box_id: arg(args, "box_id"),
            script: lines(args, "script"),
            redirect_output: args.get_one::<String>("redirect_output").cloned(),
        },
        "kill_pty_process" => SocketFunction::KillPtyProcess {
            box_id: arg(args, "box_id"),
        },
        "restart_pty_process" => SocketFunction::RestartPtyProcess {
            box_id: arg(args, "box_id"),
        },
        "query_pty_status" => SocketFunction::QueryPtyStatus {
            box_id: arg(args, "box_id"),
        },
        "send_pty_input" => SocketFunction::SendPtyInput {
            box_id: arg(args, "box_id"),
            input: arg(args, "input"),
        },
        "send_signal" => SocketFunction::SendSignal {
            box_id: arg(args, "box_id"),
            signal: *args.get_one::<i32>("signal").expect("required"),
        },
        other => unreachable!("clap only accepts declared subcommands, got {}", other),
    };
    match client::send(socket, &function)? {
        Reply {
            ok: true,
            result: Some(result),
            ..
        } => {
            println!(
                "{}",
                serde_json::to_string_pretty(&result).expect("replies are valid JSON")
            );
            Ok(())
        }
        Reply { ok: true, .. } => Ok(()),
        Reply { error, .. } => Err(error.unwrap_or_else(|| "the command failed".to_string())),
    }
}

<<<<<<< Updated upstream
fn init_logging(matches: &ArgMatches) -> Result<(), String> {
    let Some(path) = matches.get_one::<PathBuf>("log_file") else {
        return Ok(());
    };
    let level = matches
        .get_one::<String>("log_level")
        .expect("has a default")
        .parse::<log::LevelFilter>()
        .expect("clap restricts the values");
    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("cannot open the log file '{}': {}", path.display(), e))?;
    env_logger::Builder::new()
        .filter_level(level)
        .target(env_logger::Target::Pipe(Box::new(file)))
        .format(|buf, record| {
            use std::io::Write;
            writeln!(
                buf,
                "{} {:5} [{}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                std::thread::current().name().unwrap_or("-"),
                record.args()
            )
        })
        .try_init()
        .map_err(|e| format!("cannot start logging: {}", e))
}
=======
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
    let calibrate = matches.get_flag("calibrate");

    let yaml_path = Path::new(yaml_path);

    if !yaml_path.exists() {
        return Err(format!("YAML file does not exist: {}", yaml_path.display()).into());
    }
>>>>>>> Stashed changes

/// Shows the interface described in the configuration file until the user quits.
fn run(matches: &ArgMatches, socket: &str) -> Result<(), String> {
    use boxmux_lib::render::theme;
    init_logging(matches)?;
    if matches.get_flag("dark") {
        theme::set_theme_override(Some(theme::TerminalTheme::Dark));
    } else if matches.get_flag("light") {
        theme::set_theme_override(Some(theme::TerminalTheme::Light));
    }
    let path = matches.get_one::<PathBuf>("config").expect("required");
    let path = path
        .canonicalize()
        .map_err(|e| format!("cannot open '{}': {}", path.display(), e))?;
    let locked = matches.get_flag("lock");
    let config = Config::new(
        *matches
            .get_one::<u64>("frame_delay")
            .expect("has a default"),
        locked,
<<<<<<< Updated upstream
        matches.get_flag("calibrate"),
    )?;
    let app = boxmux_lib::load_app_from_yaml(&path)?;
    // Only an instance that may write the file needs to own it.
    let _config_lock = if locked {
        None
=======
        calibrate,
    );
    let app = match load_app_from_yaml_with_lock(yaml_path.to_str().unwrap(), locked) {
        Ok(app) => app,
        Err(e) => {
            // The enhanced error handling is now built into load_app_from_yaml
            // so we just need to display the detailed error message
            eprintln!("{}", e);
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
>>>>>>> Stashed changes
    } else {
        Some(ConfigLock::acquire(&path)?)
    };
    let mut ctx = AppContext::new(app, config).with_yaml_path(path);
    ctx.load_plugins()?;

    // Release builds abort on a panic, so the runtime cannot stop its scripts on the way out.
    platform::kill_process_trees_on_panic();
    let (sender, events) = crossbeam_channel::unbounded();
    let _socket_server = SocketServer::start(socket, sender.clone())?;
    let signal_sender = sender.clone();
    platform::listen_for_signals(move |signal| signal_sender.send(Event::Signal(signal)).is_ok())
        .map_err(|e| format!("cannot listen for signals: {}", e))?;

    let display =
        TerminalDisplay::enter().map_err(|e| format!("cannot use the terminal: {}", e))?;
    // Asked before the input thread starts, so the answer is not taken for typed keys.
    if !theme::theme_is_forced() {
        if let Some(color) = platform::query_background_color(std::time::Duration::from_secs(1)) {
            theme::set_background_color(color);
        }
    }
    let input_sender = sender.clone();
    std::thread::Builder::new()
        .name("input".into())
        .spawn(move || loop {
            let event = match crossterm::event::read() {
                Ok(event) => Event::Input(event),
                Err(e) => Event::InputFailed(e.to_string()),
            };
            let failed = matches!(event, Event::InputFailed(_));
            if input_sender.send(event).is_err() || failed {
                break;
            }
        })
        .map_err(|e| format!("cannot start the input thread: {}", e))?;

    Runtime::new(ctx, display, sender, events, Some(socket.to_string())).run()
}
