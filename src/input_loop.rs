use crate::utils::should_use_pty;
use crate::{handle_keypress, AppContext, FieldUpdate};
use crate::{run_script, thread_manager::Runnable};
use crossterm::event::{
    poll, read, Event, KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind,
};
use std::sync::mpsc;
use std::time::Duration;

use crate::thread_manager::*;

use uuid::Uuid;

/// Convert crossterm KeyEvent to appropriate PTY input string
/// F0127: Enhanced special key handling with modifiers and extended key support
pub fn format_key_for_pty(code: KeyCode, modifiers: KeyModifiers) -> String {
    // Helper function to generate modified key sequences
    let format_modified_key = |base_seq: &str, modifiers: KeyModifiers| -> String {
        let mut mod_code = 1;
        if modifiers.contains(KeyModifiers::SHIFT) {
            mod_code += 1;
        }
        if modifiers.contains(KeyModifiers::ALT) {
            mod_code += 2;
        }
        if modifiers.contains(KeyModifiers::CONTROL) {
            mod_code += 4;
        }

        if mod_code == 1 {
            base_seq.to_string()
        } else {
            // Format: ESC[1;{mod}{key}
            format!("\x1b[1;{}{}", mod_code, &base_seq[2..])
        }
    };

    match code {
        KeyCode::Char(c) => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                // Enhanced control character handling
                match c {
                    'a'..='z' => {
                        let ctrl_code = (c as u8) - b'a' + 1;
                        format!("{}", ctrl_code as char)
                    }
                    'A'..='Z' => {
                        let ctrl_code = (c as u8) - b'A' + 1;
                        format!("{}", ctrl_code as char)
                    }
                    '@' => "\x00".to_string(),  // Ctrl+@
                    '[' => "\x1b".to_string(),  // Ctrl+[
                    '\\' => "\x1c".to_string(), // Ctrl+\
                    ']' => "\x1d".to_string(),  // Ctrl+]
                    '^' => "\x1e".to_string(),  // Ctrl+^
                    '_' => "\x1f".to_string(),  // Ctrl+_
                    ' ' => "\x00".to_string(),  // Ctrl+Space
                    _ => c.to_string(),
                }
            } else if modifiers.contains(KeyModifiers::ALT) {
                // Alt+character combinations
                format!("\x1b{}", c)
            } else {
                c.to_string()
            }
        }
        KeyCode::Enter => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                "\n".to_string() // Ctrl+Enter
            } else {
                "\r".to_string()
            }
        }
        KeyCode::Tab => {
            if modifiers.contains(KeyModifiers::SHIFT) {
                "\x1b[Z".to_string() // Shift+Tab (Back Tab)
            } else {
                "\t".to_string()
            }
        }
        KeyCode::Backspace => {
            if modifiers.contains(KeyModifiers::CONTROL) {
                "\x08".to_string() // Ctrl+Backspace
            } else if modifiers.contains(KeyModifiers::ALT) {
                "\x1b\x7f".to_string() // Alt+Backspace
            } else {
                "\x7f".to_string()
            }
        }
        KeyCode::Delete => format_modified_key("\x1b[3~", modifiers),
        KeyCode::Insert => format_modified_key("\x1b[2~", modifiers),

        // Arrow keys with modifier support
        KeyCode::Up => format_modified_key("\x1b[A", modifiers),
        KeyCode::Down => format_modified_key("\x1b[B", modifiers),
        KeyCode::Right => format_modified_key("\x1b[C", modifiers),
        KeyCode::Left => format_modified_key("\x1b[D", modifiers),

        // Home/End with modifier support
        KeyCode::Home => format_modified_key("\x1b[H", modifiers),
        KeyCode::End => format_modified_key("\x1b[F", modifiers),

        // Page Up/Down with modifier support
        KeyCode::PageUp => format_modified_key("\x1b[5~", modifiers),
        KeyCode::PageDown => format_modified_key("\x1b[6~", modifiers),

        KeyCode::Esc => "\x1b".to_string(),

        KeyCode::F(n) => {
            // F1-F24 keys with modifier support
            let base_seq = match n {
                1 => "\x1bOP",
                2 => "\x1bOQ",
                3 => "\x1bOR",
                4 => "\x1bOS",
                5 => "\x1b[15~",
                6 => "\x1b[17~",
                7 => "\x1b[18~",
                8 => "\x1b[19~",
                9 => "\x1b[20~",
                10 => "\x1b[21~",
                11 => "\x1b[23~",
                12 => "\x1b[24~",
                13 => "\x1b[25~",
                14 => "\x1b[26~",
                15 => "\x1b[28~",
                16 => "\x1b[29~",
                17 => "\x1b[31~",
                18 => "\x1b[32~",
                19 => "\x1b[33~",
                20 => "\x1b[34~",
                21 => "\x1b[35~",
                22 => "\x1b[36~",
                23 => "\x1b[37~",
                24 => "\x1b[38~",
                _ => return "".to_string(),
            };

            if modifiers.is_empty() {
                base_seq.to_string()
            } else {
                format_modified_key(base_seq, modifiers)
            }
        }

        // Additional special keys
        KeyCode::CapsLock => "".to_string(), // Usually handled by system
        KeyCode::ScrollLock => "".to_string(),
        KeyCode::NumLock => "".to_string(),
        KeyCode::PrintScreen => "".to_string(),
        KeyCode::Pause => "".to_string(),
        KeyCode::Menu => "\x1b[29~".to_string(),

        _ => "".to_string(),
    }
}
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
                    Event::Mouse(MouseEvent {
                        kind,
                        column,
                        row,
                        modifiers: _,
                    }) => {
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
                                // F0091 & F0188: Handle mouse clicks and start of drag
                                inner.send_message(Message::MouseClick(column, row));
                                inner.send_message(Message::MouseDragStart(column, row));
                                format!("MouseClick({}, {})", column, row)
                            }
                            MouseEventKind::Drag(_button) => {
                                // F0188: Handle mouse drag for scroll knob dragging
                                inner.send_message(Message::MouseDrag(column, row));
                                format!("MouseDrag({}, {})", column, row)
                            }
                            MouseEventKind::Up(_button) => {
                                // F0188: Handle end of drag
                                inner.send_message(Message::MouseDragEnd(column, row));
                                format!("MouseDragEnd({}, {})", column, row)
                            }
                            _ => return (true, app_context), // Ignore other mouse events
                        }
                    }
                    Event::Key(KeyEvent {
                        code, modifiers, ..
                    }) => {
                        // Check if focused panel has PTY enabled - if so, route input to PTY
                        let selected_panels = active_layout.get_selected_panels();
                        let focused_panel_has_pty = selected_panels
                            .first()
                            .map(|panel| should_use_pty(panel))
                            .unwrap_or(false);

                        if focused_panel_has_pty {
                            // Convert key event to string and send to PTY
                            let key_str = format_key_for_pty(code, modifiers);
                            if let Some(focused_panel) = selected_panels.first() {
                                inner.send_message(Message::PTYInput(
                                    focused_panel.id.clone(),
                                    key_str.clone(),
                                ));
                                return (true, app_context);
                            }
                        }

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
                                        _ => format!("Ctrl+{}", c),
                                    }
                                } else if modifiers.contains(KeyModifiers::SUPER) {
                                    match c {
                                        'c' => {
                                            inner.send_message(Message::CopyFocusedPanelContent());
                                            "Cmd+c".to_string()
                                        }
                                        _ => format!("Cmd+{}", c),
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
                            KeyCode::Home => {
                                if modifiers.contains(KeyModifiers::CONTROL) {
                                    // Ctrl+Home: scroll to top vertically
                                    inner.send_message(Message::ScrollPanelToTop());
                                    "Ctrl+Home".to_string()
                                } else {
                                    // Home: scroll to beginning horizontally
                                    inner.send_message(Message::ScrollPanelToBeginning());
                                    "Home".to_string()
                                }
                            }
                            KeyCode::End => {
                                if modifiers.contains(KeyModifiers::CONTROL) {
                                    // Ctrl+End: scroll to bottom vertically
                                    inner.send_message(Message::ScrollPanelToBottom());
                                    "Ctrl+End".to_string()
                                } else {
                                    // End: scroll to end horizontally
                                    inner.send_message(Message::ScrollPanelToEnd());
                                    "End".to_string()
                                }
                            }
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
