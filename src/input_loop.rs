use crate::model::app::App;
use crate::thread_manager::{self, Runnable};
use crate::{screen_height, screen_width, AppContext, ScreenBuffer};
use log::{error, info};
use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
use simplelog::*;
use std::collections::hash_map::DefaultHasher;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::io::Write as IoWrite;
use std::io::{stdin, stdout, Read};
use std::process::Command;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use serde::{
    de::MapAccess, de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

use crate::thread_manager::*;

use std::sync::atomic::{AtomicBool, Ordering};

use crate::utils::{draw_panel as util_draw_panel, fill_panel, screen_bounds};
use uuid::Uuid;

create_runnable!(
    InputLoop,
    |inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>| -> bool {
        // Initialization block
        // info!("Initializing InputLoop with state: {:?}", state);

        true // Initialization complete, continue running
    },
    |inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>| -> bool {
        // Processing block
        // info!(
        //     "Processing in InputLoop with state: {:?} and messages: {:?}",
        //     state, messages
        // );

        let stdin = stdin();
        let mut should_continue = true;
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => {
                    log::info!("Q Exiting...");
                    inner.send_message(Message::Exit);
                    should_continue = false; // Stop running
                    break;
                }
                Key::Char('\t') => {
                    inner.send_message(Message::NextPanel("".to_string()));
                }
                Key::BackTab => {
                    inner.send_message(Message::PreviousPanel("".to_string()));
                }
                Key::Down => {
                    // let mut selected_box_guard = SELECTED_BOX.lock().unwrap();
                    // if let Some(ref selected_box) = *selected_box_guard {
                    //     let mut box_guard = selected_box.0.lock().unwrap();
                    //     box_guard.scroll_down(None);
                    //     input_tx
                    //         .send(Message::RedrawPanel(BoxEntityWrapper::clone(
                    //             selected_box,
                    //         )))
                    //         .unwrap();
                    // }
                }
                Key::Up => {
                    // let mut selected_box_guard = SELECTED_BOX.lock().unwrap();
                    // if let Some(ref selected_box) = *selected_box_guard {
                    //     let mut box_guard = selected_box.0.lock().unwrap();
                    //     box_guard.scroll_up(None);
                    //     input_tx
                    //         .send(Message::RedrawPanel(BoxEntityWrapper::clone(
                    //             selected_box,
                    //         )))
                    //         .unwrap();
                    // }
                }
                _ => {}
            }
        }

		std::thread::sleep(std::time::Duration::from_millis(10));

        should_continue
    }
);
