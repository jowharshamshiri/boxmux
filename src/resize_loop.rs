// use crate::model::app::App;
// use crate::thread_manager::{self, Runnable, RunnableExt};
// use crate::{screen_height, screen_width, InputMessage, ScreenBuffer};
// use log::{error, info};
// use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
// use simplelog::*;
// use std::collections::hash_map::DefaultHasher;
// use std::fs::File;
// use std::hash::{Hash, Hasher};
// use std::io::Write as IoWrite;
// use std::io::{stdin, stdout, Read};
// use std::process::Command;
// use std::sync::{mpsc, Arc, Mutex};
// use std::thread;
// use std::time::Duration;
// use termion::color;
// use termion::cursor;
// use termion::event::Key;
// use termion::input::TermRead;
// use termion::raw::{IntoRawMode, RawTerminal};
// use termion::screen::AlternateScreen;

// use serde::{
//     de::MapAccess, de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize, Serializer,
// };
// use std::fmt;

// pub struct ResizeLoop {
//     pub app: App,
//     pub sender: Option<mpsc::Sender<App>>,
//     pub stdout: AlternateScreen<RawTerminal<std::io::Stdout>>,
//     pub rx: mpsc::Receiver<InputMessage>,
//     pub tx: mpsc::Sender<InputMessage>,
// }

// impl Runnable for ResizeLoop {
//     fn run(&mut self) -> Result<(), Box<dyn std::error::Error>> {
//         // loop {
//         //     // Perform processing with the current app data
//         //     info!("MyRunnableTwo running with data: {:?}", self.app);

//         //     // Simulate processing time
//         //     thread::sleep(std::time::Duration::from_secs(1));

//         // Send updated app back to main thread
//         self.send_update();
//         // }
// 		let mut signals = Signals::new(&[SIGWINCH]).unwrap();
//         for _ in signals.forever() {
//             resize_tx.send(InputMessage::Resize).unwrap();
//         }
//         Ok(())
//     }

//     fn update_app(&mut self, app: App) {
//         self.app = app;
//     }

//     fn set_sender(&mut self, sender: mpsc::Sender<App>) {
//         self.sender = Some(sender);
//     }

//     fn get_app(&self) -> &App {
//         &self.app
//     }

//     fn get_sender(&self) -> &Option<mpsc::Sender<App>> {
//         &self.sender
//     }
// }
