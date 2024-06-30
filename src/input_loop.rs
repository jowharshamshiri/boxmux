use crate::thread_manager::Runnable;
use crate::AppContext;
use std::io::stdin;
use std::sync::mpsc;
use termion::event::Key;
use termion::input::TermRead;

use crate::thread_manager::*;

use uuid::Uuid;

create_runnable!(
    InputLoop,
    |inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>| -> bool {
        true 
    },
    |inner: &mut RunnableImpl, state: AppContext, messages: Vec<Message>| -> bool {
        let stdin = stdin();
        let mut should_continue = true;
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => {
                    inner.send_message(Message::Exit);
                    should_continue = false; // Stop running
                    break;
                }
                Key::Char('\t') => {
                    inner.send_message(Message::NextPanel());
                }
                Key::BackTab => {
                    inner.send_message(Message::PreviousPanel());
                }
                Key::Down => {
					inner.send_message(Message::ScrollPanelDown());
                }
                Key::Up => {
					inner.send_message(Message::ScrollPanelUp());
                }
				Key::Left => {
					inner.send_message(Message::ScrollPanelLeft());
				}
				Key::Right => {
					inner.send_message(Message::ScrollPanelRight());
				}
                _ => {}
            }
        }

		std::thread::sleep(std::time::Duration::from_millis(10));

        should_continue
    }
);
