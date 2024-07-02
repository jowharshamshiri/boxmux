// use crate::{execute_commands, thread_manager::Runnable};
// use crate::{handle_keypress, AppContext, Config};
// use std::io::stdin;
// use std::os::unix::net::{UnixListener, UnixStream};
// use std::sync::{mpsc, Arc, Mutex};
// use termion::event::Key;
// use termion::input::TermRead;

// use crate::thread_manager::*;

// use crate::model::app::*;

// use uuid::Uuid;

// use termion::event::Event;
// create_runnable!(
//     InputLoop,
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool { 
// 		let socket_path = "/tmp/app_socket";
//     	let listener = UnixListener::bind(socket_path);

// 		let config_clone = app_context.config.clone();
// 		std::thread::spawn(move || {
// 			for stream in listener.incoming() {
// 				match stream {
// 					Ok(stream) => {
// 						let config_clone = config_clone.clone();
// 						std::thread::spawn(move || {
// 							handle_client(stream, config_clone);
// 						});
// 					}
// 					Err(err) => {
// 						eprintln!("Error: {:?}", err);
// 					}
// 				}
// 			}
// 		});
		
// 		true },
//     |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool {
//         let config = Arc::new(Mutex::new(Config::default()));

    

//     // Your main app logic here, using the `config` as needed
//     loop {
//         let config_guard = config.lock().unwrap();
//         println!("Current config: {:?}", *config_guard);
//         std::thread::sleep(std::time::Duration::from_millis(config_guard.frame_delay));
//     }
// 		std::thread::sleep(std::time::Duration::from_millis(app_context.config.frame_delay));

//         should_continue
//     }
// );


// fn handle_client(mut stream: UnixStream, config: Arc<Mutex<Config>>) {
//     let mut buffer = [0; 1024];
//     stream.read(&mut buffer).unwrap();
//     let new_config: Config = serde_json::from_slice(&buffer).unwrap();

//     let mut config_guard = config.lock().unwrap();
//     *config_guard = new_config;

//     stream.write_all(b"Config updated").unwrap();
// }