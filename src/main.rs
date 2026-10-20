use boxmux_lib::model::yaml_edit::ConfigLock;
use boxmux_lib::platform;
use boxmux_lib::runtime::display::TerminalDisplay;
use boxmux_lib::runtime::event::Event;
use boxmux_lib::runtime::Runtime;
use boxmux_lib::socket::protocol::{Reply, SocketFunction};
use boxmux_lib::socket::{client, server::SocketServer};
use boxmux_lib::{AppContext, Config, MuxBox};
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

fn parse_box(json: &str) -> Result<Box<MuxBox>, String> {
    serde_json::from_str(json).map_err(|e| format!("invalid box JSON: {}", e))
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
        matches.get_flag("calibrate"),
    )?;
    let app = boxmux_lib::load_app_from_yaml(&path)?;
    // Only an instance that may write the file needs to own it.
    let _config_lock = if locked {
        None
    } else {
        Some(ConfigLock::acquire(&path)?)
    };
    let mut ctx = AppContext::new(app, config).with_yaml_path(path);
    ctx.load_plugins()?;

    let (sender, events) = crossbeam_channel::unbounded();
    let _socket_server = SocketServer::start(socket, sender.clone())?;
    let signal_sender = sender.clone();
    platform::listen_for_signals(move |signal| signal_sender.send(Event::Signal(signal)).is_ok())
        .map_err(|e| format!("cannot listen for signals: {}", e))?;

    let display =
        TerminalDisplay::enter().map_err(|e| format!("cannot use the terminal: {}", e))?;
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
