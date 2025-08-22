use crate::{handle_keypress, AppContext, FieldUpdate};
use crate::{thread_manager::Runnable};
use crate::streaming_executor::{StreamingExecutor, OutputLine};
use std::sync::mpsc;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, poll, read};
use std::time::Duration;

use crate::thread_manager::*;

use uuid::Uuid;
create_runnable!(
    InputLoop,
    |_inner: &mut RunnableImpl, _app_context: AppContext, _messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl,
     app_context: AppContext,
     _messages: Vec<Message>|
     -> (bool, AppContext) {
        let mut should_continue = true;

        let active_layout = app_context.app.get_active_layout().unwrap();

        if poll(Duration::from_millis(10)).unwrap() {
            if let Ok(event) = read() {
                let key_str = match event {
                    Event::Mouse(MouseEvent { kind, column, row, modifiers: _ }) => {
                        match kind {
                            MouseEventKind::ScrollUp => {
                                inner.send_message(Message::ScrollPanelUp());
                                "ScrollUp".to_string()
                            }
                            MouseEventKind::ScrollDown => {
                                inner.send_message(Message::ScrollPanelDown());
                                "ScrollDown".to_string()
                            }
                            MouseEventKind::ScrollLeft => {
                                inner.send_message(Message::ScrollPanelLeft());
                                "ScrollLeft".to_string()
                            }
                            MouseEventKind::ScrollRight => {
                                inner.send_message(Message::ScrollPanelRight());
                                "ScrollRight".to_string()
                            }
                            MouseEventKind::Down(_button) => {
                                // F0091: Handle mouse clicks
                                inner.send_message(Message::MouseClick(column, row));
                                format!("MouseClick({}, {})", column, row)
                            }
                            _ => return (true, app_context), // Ignore other mouse events
                        }
                    }
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
                            KeyCode::PageUp => {
                                if modifiers.contains(KeyModifiers::SHIFT) {
                                    inner.send_message(Message::ScrollPanelPageLeft());
                                    "Shift+PageUp".to_string()
                                } else {
                                    inner.send_message(Message::ScrollPanelPageUp());
                                    "PageUp".to_string()
                                }
                            }
                            KeyCode::PageDown => {
                                if modifiers.contains(KeyModifiers::SHIFT) {
                                    inner.send_message(Message::ScrollPanelPageRight());
                                    "Shift+PageDown".to_string()
                                } else {
                                    inner.send_message(Message::ScrollPanelPageDown());
                                    "PageDown".to_string()
                                }
                            }
                            KeyCode::Char(c) => {
                                if modifiers.contains(KeyModifiers::CONTROL) {
                                    match c {
                                        'c' => {
                                            inner.send_message(Message::CopyFocusedPanelContent());
                                            "Ctrl+c".to_string()
                                        }
                                        _ => format!("Ctrl+{}", c)
                                    }
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
                            KeyCode::F(n) => format!("F{}", n),
                            KeyCode::Insert => "Insert".to_string(),
                            _ => return (true, app_context),
                        }
                    }
                    _ => return (true, app_context),
                };

                // F0081: Hot Key Actions - Direct choice execution
                if let Some(hot_keys) = &app_context.app.hot_keys {
                    if let Some(choice_id) = hot_keys.get(&key_str) {
                        inner.send_message(Message::ExecuteHotKeyChoice(choice_id.clone()));
                    }
                }
                
                if let Some(app_key_mappings) = &app_context.app.on_keypress {
                    if let Some(actions) = handle_keypress(&key_str, app_key_mappings) {
                        let libs = app_context.app.libs.clone();
                        
                        // Use streaming execution for app-level keypress actions
                        let mut executor = StreamingExecutor::new();
                        let combined_command = if let Some(ref libs) = libs {
                            let mut full_script = libs.join(" && ");
                            full_script.push_str(" && ");
                            full_script.push_str(&actions.join(" && "));
                            full_script
                        } else {
                            actions.join(" && ")
                        };
                        
                        match executor.spawn_streaming(&combined_command, None) {
                            Ok((mut child, receiver)) => {
                                // For global app actions, we collect output but don't display it
                                // to avoid interfering with the current UI state
                                let mut _output_buffer = String::new();
                                while let Ok(line) = receiver.recv_timeout(Duration::from_millis(50)) {
                                    _output_buffer.push_str(&line.content);
                                }
                                
                                // Wait for completion and log results
                                match child.wait() {
                                    Ok(status) => {
                                        if status.success() {
                                            log::trace!("App keypress action executed successfully");
                                        } else {
                                            log::warn!("App keypress action failed with exit code: {:?}", status.code());
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("App keypress action process error: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to start app keypress streaming: {}", e);
                            }
                        }
                    }
                }
                if let Some(layout_key_mappings) = &active_layout.on_keypress {
                    if let Some(actions) = handle_keypress(&key_str, layout_key_mappings) {
                        let libs = app_context.app.libs.clone();
                        
                        // Use streaming execution for layout-level keypress actions
                        let mut executor = StreamingExecutor::new();
                        let combined_command = if let Some(ref libs) = libs {
                            let mut full_script = libs.join(" && ");
                            full_script.push_str(" && ");
                            full_script.push_str(&actions.join(" && "));
                            full_script
                        } else {
                            actions.join(" && ")
                        };
                        
                        match executor.spawn_streaming(&combined_command, None) {
                            Ok((mut child, receiver)) => {
                                // For layout actions, collect output but don't display
                                // as these are typically control/navigation commands
                                let mut _output_buffer = String::new();
                                while let Ok(line) = receiver.recv_timeout(Duration::from_millis(50)) {
                                    _output_buffer.push_str(&line.content);
                                }
                                
                                // Wait for completion and log results
                                match child.wait() {
                                    Ok(status) => {
                                        if status.success() {
                                            log::trace!("Layout keypress action executed successfully");
                                        } else {
                                            log::warn!("Layout keypress action failed with exit code: {:?}", status.code());
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Layout keypress action process error: {}", e);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("Failed to start layout keypress streaming: {}", e);
                            }
                        }
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
