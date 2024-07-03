use crate::{execute_commands, thread_manager::Runnable};
use crate::{handle_keypress, AppContext};
use std::io::stdin;
use std::sync::mpsc;
use termion::event::Key;
use termion::input::TermRead;

use crate::thread_manager::*;

use uuid::Uuid;

use termion::event::Event;
create_runnable!(
    InputLoop,
    |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl,
     app_context: AppContext,
     messages: Vec<Message>|
     -> (bool, AppContext) {
        let stdin = stdin();
        let mut should_continue = true;

        let active_layout = app_context.app.get_active_layout().unwrap();

        for c in stdin.events() {
            if let Ok(event) = c {
                let key_str = match event {
                    Event::Key(key) => {
                        match key {
                            Key::Char('q') => {
                                inner.send_message(Message::Exit);
                                should_continue = false; // Stop running
                                "q".to_string()
                            }
                            Key::Char('\t') => {
                                inner.send_message(Message::NextPanel());
                                "Tab".to_string()
                            }
                            Key::BackTab => {
                                inner.send_message(Message::PreviousPanel());
                                "BackTab".to_string()
                            }
                            Key::Down => {
                                inner.send_message(Message::ScrollPanelDown());
                                "Down".to_string()
                            }
                            Key::Up => {
                                inner.send_message(Message::ScrollPanelUp());
                                "Up".to_string()
                            }
                            Key::Left => {
                                inner.send_message(Message::ScrollPanelLeft());
                                "Left".to_string()
                            }
                            Key::Right => {
                                inner.send_message(Message::ScrollPanelRight());
                                "Right".to_string()
                            }
                            Key::Char(c) => c.to_string(),
                            Key::Ctrl(c) => format!("Ctrl+{}", c),
                            Key::Alt(c) => format!("Alt+{}", c),
                            Key::Backspace => "Backspace".to_string(),
                            Key::Delete => "Delete".to_string(),
                            Key::Esc => "Esc".to_string(),
                            Key::Home => "Home".to_string(),
                            Key::End => "End".to_string(),
                            Key::PageUp => "PageUp".to_string(),
                            Key::PageDown => "PageDown".to_string(),
                            Key::F(n) => format!("F{}", n),
                            Key::Insert => "Insert".to_string(),
                            _ => return (true, app_context),
                        }
                    }
                    _ => return (true, app_context),
                };

                if let Some(app_key_mappings) = &app_context.app.on_keypress {
                    if let Some(actions) = handle_keypress(&key_str, app_key_mappings) {
                        execute_commands(&actions);
                    }
                }
                if let Some(layout_key_mappings) = &active_layout.on_keypress {
                    if let Some(actions) = handle_keypress(&key_str, layout_key_mappings) {
                        execute_commands(&actions);
                    }
                }

                inner.send_message(Message::KeyPress(key_str.clone()));
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(
            app_context.config.frame_delay,
        ));

        (should_continue, app_context)
    }
);
