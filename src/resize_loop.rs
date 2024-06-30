use crate::thread_manager::Runnable;
use crate::Message;
use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
use crate::AppContext;

use crate::thread_manager::*;

use uuid::Uuid;
use std::io::stdin;
use std::sync::mpsc;
use termion::event::Key;
use termion::input::TermRead;

use crate::thread_manager::*;

use termion::event::Event;

create_runnable!(
    ResizeLoop,
    |inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>| -> bool {
		let mut signals = Signals::new(&[SIGWINCH]).unwrap();
        for _ in signals.forever() {
            inner.send_message(Message::Resize);
        }
		true
    }
);
