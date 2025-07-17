use crate::{handle_keypress, AppContext, FieldUpdate};
use crate::{run_script, thread_manager::Runnable};
use std::sync::mpsc;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, poll, read};
use std::time::Duration;

use crate::thread_manager::*;

use uuid::Uuid;
create_runnable!(
    InputLoop,
    |inner: &mut RunnableImpl, app_context: AppContext, messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl,
     app_context: AppContext,
     messages: Vec<Message>|
     -> (bool, AppContext) {
        let mut should_continue = true;

        let active_layout = app_context.app.get_active_layout().unwrap();

        if poll(Duration::from_millis(10)).unwrap() {
            if let Ok(event) = read() {
                let key_str = match event {
                    Event::Key(KeyEvent { code, modifiers, .. }) => {
                        match code {
                            KeyCode::Char('q') => {
                                inner.send_message(Message::Exit);
                                should_continue = false; // Stop running
                                "q".to_string()
                            }
                            KeyCode::Tab => {
                                inner.send_message(Message::NextPanel());
                                "Tab".to_string()
                            }
                            KeyCode::BackTab => {
                                inner.send_message(Message::PreviousPanel());
                                "BackTab".to_string()
                            }
                            KeyCode::Enter => "Enter".to_string(),
                            KeyCode::Down => {
                                inner.send_message(Message::ScrollPanelDown());
                                "Down".to_string()
                            }
                            KeyCode::Up => {
                                inner.send_message(Message::ScrollPanelUp());
                                "Up".to_string()
                            }
                            KeyCode::Left => {
                                inner.send_message(Message::ScrollPanelLeft());
                                "Left".to_string()
                            }
                            KeyCode::Right => {
                                inner.send_message(Message::ScrollPanelRight());
                                "Right".to_string()
                            }
                            KeyCode::Char(c) => {
                                if modifiers.contains(KeyModifiers::CONTROL) {
                                    format!("Ctrl+{}", c)
                                } else if modifiers.contains(KeyModifiers::ALT) {
                                    format!("Alt+{}", c)
                                } else {
                                    c.to_string()
                                }
                            }
                            KeyCode::Backspace => "Backspace".to_string(),
                            KeyCode::Delete => "Delete".to_string(),
                            KeyCode::Esc => "Esc".to_string(),
                            KeyCode::Home => "Home".to_string(),
                            KeyCode::End => "End".to_string(),
                            KeyCode::PageUp => "PageUp".to_string(),
                            KeyCode::PageDown => "PageDown".to_string(),
                            KeyCode::F(n) => format!("F{}", n),
                            KeyCode::Insert => "Insert".to_string(),
                            _ => return (true, app_context),
                        }
                    }
                    _ => return (true, app_context),
                };

                if let Some(app_key_mappings) = &app_context.app.on_keypress {
                    if let Some(actions) = handle_keypress(&key_str, app_key_mappings) {
                        let libs = app_context.app.libs.clone();
                        let _result = run_script(libs, &actions);
                    }
                }
                if let Some(layout_key_mappings) = &active_layout.on_keypress {
                    if let Some(actions) = handle_keypress(&key_str, layout_key_mappings) {
                        let libs = app_context.app.libs.clone();
                        let _ = run_script(libs, &actions);
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
