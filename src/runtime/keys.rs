//! Keyboard handling: the names keys have in configuration (`hot_keys`, `on_keypress`), and the
//! byte sequences a program in a PTY expects for each key.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// The name of a key press as written in `hot_keys` and `on_keypress`: `a`, `A`, `Enter`, `Tab`,
/// `Up`, `F5`, `Ctrl+r`, `Alt+x`, `Space`... `None` for keys that have no name.
pub fn key_name(event: &KeyEvent) -> Option<String> {
    let base = match event.code {
        KeyCode::Char(' ') => "Space".to_string(),
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".to_string(),
        KeyCode::Tab => "Tab".to_string(),
        KeyCode::BackTab => "BackTab".to_string(),
        KeyCode::Backspace => "Backspace".to_string(),
        KeyCode::Delete => "Delete".to_string(),
        KeyCode::Insert => "Insert".to_string(),
        KeyCode::Esc => "Esc".to_string(),
        KeyCode::Home => "Home".to_string(),
        KeyCode::End => "End".to_string(),
        KeyCode::PageUp => "PageUp".to_string(),
        KeyCode::PageDown => "PageDown".to_string(),
        KeyCode::Up => "Up".to_string(),
        KeyCode::Down => "Down".to_string(),
        KeyCode::Left => "Left".to_string(),
        KeyCode::Right => "Right".to_string(),
        KeyCode::F(n) => format!("F{}", n),
        _ => return None,
    };
    let mut name = String::new();
    if event.modifiers.contains(KeyModifiers::CONTROL) {
        name.push_str("Ctrl+");
    }
    if event.modifiers.contains(KeyModifiers::ALT) {
        name.push_str("Alt+");
    }
    if event.modifiers.contains(KeyModifiers::SUPER) {
        name.push_str("Cmd+");
    }
    // Shift is part of the character for printable keys (`A` vs `a`), and a modifier for the
    // others.
    if event.modifiers.contains(KeyModifiers::SHIFT)
        && !matches!(event.code, KeyCode::Char(_) | KeyCode::BackTab)
    {
        name.push_str("Shift+");
    }
    name.push_str(&base);
    Some(name)
}

/// Whether a configured key name (`ctrl+R`, `enter`) refers to the pressed key `name`. Modifier
/// and special key names are matched case-insensitively; a single printable character must
/// match exactly, so `a` and `A` stay different keys.
pub fn key_matches(configured: &str, name: &str) -> bool {
    let split = |key: &str| -> (Vec<String>, String) {
        let mut parts: Vec<&str> = key.split('+').collect();
        // A trailing '+' is the plus key itself: "Ctrl++".
        let last = if key.ends_with("++") {
            parts.truncate(parts.len().saturating_sub(2));
            "+".to_string()
        } else {
            parts.pop().unwrap_or_default().to_string()
        };
        let mut modifiers: Vec<String> = parts.iter().map(|m| m.to_ascii_lowercase()).collect();
        modifiers.sort();
        (modifiers, last)
    };
    let (configured_mods, configured_key) = split(configured.trim());
    let (pressed_mods, pressed_key) = split(name);
    let keys_equal = if configured_key.chars().count() == 1 && pressed_key.chars().count() == 1 {
        configured_key == pressed_key
    } else {
        configured_key.eq_ignore_ascii_case(&pressed_key)
    };
    configured_mods == pressed_mods && keys_equal
}

/// The bytes a terminal program receives for a key press. `application_cursor_keys` is the
/// program's DECCKM mode (arrows send `ESC O A` instead of `ESC [ A`).
pub fn pty_key_bytes(event: &KeyEvent, application_cursor_keys: bool) -> Vec<u8> {
    let modifiers = event.modifiers;
    // xterm modifier parameter: 1 + shift(1) + alt(2) + ctrl(4).
    let modifier_code = 1
        + u8::from(modifiers.contains(KeyModifiers::SHIFT))
        + 2 * u8::from(modifiers.contains(KeyModifiers::ALT))
        + 4 * u8::from(modifiers.contains(KeyModifiers::CONTROL));
    // Cursor-style keys: `ESC [ X` (or `ESC O X` in application mode), `ESC [ 1 ; m X` with
    // modifiers.
    let cursor = |letter: char| -> Vec<u8> {
        if modifier_code > 1 {
            format!("\x1b[1;{}{}", modifier_code, letter).into_bytes()
        } else if application_cursor_keys {
            format!("\x1bO{}", letter).into_bytes()
        } else {
            format!("\x1b[{}", letter).into_bytes()
        }
    };
    // Tilde keys: `ESC [ n ~`, `ESC [ n ; m ~` with modifiers.
    let tilde = |number: u8| -> Vec<u8> {
        if modifier_code > 1 {
            format!("\x1b[{};{}~", number, modifier_code).into_bytes()
        } else {
            format!("\x1b[{}~", number).into_bytes()
        }
    };
    match event.code {
        KeyCode::Char(c) => {
            let mut bytes = Vec::new();
            if modifiers.contains(KeyModifiers::ALT) {
                bytes.push(0x1b);
            }
            if modifiers.contains(KeyModifiers::CONTROL) {
                let control = match c {
                    'a'..='z' => Some(c as u8 - b'a' + 1),
                    'A'..='Z' => Some(c as u8 - b'A' + 1),
                    '@' | ' ' | '2' => Some(0),
                    '[' | '3' => Some(0x1b),
                    '\\' | '4' => Some(0x1c),
                    ']' | '5' => Some(0x1d),
                    '^' | '6' => Some(0x1e),
                    '_' | '7' | '-' => Some(0x1f),
                    '?' | '8' => Some(0x7f),
                    _ => None,
                };
                match control {
                    Some(byte) => bytes.push(byte),
                    None => bytes.extend(c.to_string().as_bytes()),
                }
            } else {
                bytes.extend(c.to_string().as_bytes());
            }
            bytes
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::BackTab => b"\x1b[Z".to_vec(),
        KeyCode::Backspace => {
            if modifiers.contains(KeyModifiers::ALT) {
                b"\x1b\x7f".to_vec()
            } else if modifiers.contains(KeyModifiers::CONTROL) {
                vec![0x08]
            } else {
                vec![0x7f]
            }
        }
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => cursor('A'),
        KeyCode::Down => cursor('B'),
        KeyCode::Right => cursor('C'),
        KeyCode::Left => cursor('D'),
        KeyCode::Home => cursor('H'),
        KeyCode::End => cursor('F'),
        KeyCode::Insert => tilde(2),
        KeyCode::Delete => tilde(3),
        KeyCode::PageUp => tilde(5),
        KeyCode::PageDown => tilde(6),
        KeyCode::F(n @ 1..=4) => {
            let letter = (b'P' + (n - 1)) as char;
            if modifier_code > 1 {
                format!("\x1b[1;{}{}", modifier_code, letter).into_bytes()
            } else {
                format!("\x1bO{}", letter).into_bytes()
            }
        }
        KeyCode::F(n @ 5..=12) => {
            let number = [15, 17, 18, 19, 20, 21, 23, 24][usize::from(n - 5)];
            tilde(number)
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn names_include_modifiers_but_not_shift_for_characters() {
        assert_eq!(
            key_name(&key(KeyCode::Char('A'), KeyModifiers::SHIFT)).unwrap(),
            "A"
        );
        assert_eq!(
            key_name(&key(KeyCode::Char('r'), KeyModifiers::CONTROL)).unwrap(),
            "Ctrl+r"
        );
        assert_eq!(
            key_name(&key(KeyCode::Up, KeyModifiers::SHIFT)).unwrap(),
            "Shift+Up"
        );
        assert_eq!(
            key_name(&key(KeyCode::Char(' '), KeyModifiers::NONE)).unwrap(),
            "Space"
        );
        assert_eq!(
            key_name(&key(KeyCode::F(5), KeyModifiers::NONE)).unwrap(),
            "F5"
        );
    }

    #[test]
    fn configured_names_match_case_insensitively_except_single_characters() {
        assert!(key_matches("ctrl+R", "Ctrl+R"));
        assert!(key_matches("enter", "Enter"));
        assert!(key_matches("F5", "F5"));
        assert!(!key_matches("a", "A"));
        assert!(!key_matches("Ctrl+a", "a"));
        assert!(key_matches("Alt+Ctrl+x", "Ctrl+Alt+x"));
        assert!(key_matches("Ctrl++", "Ctrl++"));
    }

    #[test]
    fn arrows_follow_the_programs_cursor_key_mode() {
        let up = key(KeyCode::Up, KeyModifiers::NONE);
        assert_eq!(pty_key_bytes(&up, false), b"\x1b[A");
        assert_eq!(pty_key_bytes(&up, true), b"\x1bOA");
        let ctrl_right = key(KeyCode::Right, KeyModifiers::CONTROL);
        assert_eq!(pty_key_bytes(&ctrl_right, true), b"\x1b[1;5C");
    }

    #[test]
    fn control_and_alt_characters_become_terminal_bytes() {
        assert_eq!(
            pty_key_bytes(&key(KeyCode::Char('c'), KeyModifiers::CONTROL), false),
            vec![3]
        );
        assert_eq!(
            pty_key_bytes(&key(KeyCode::Char('x'), KeyModifiers::ALT), false),
            b"\x1bx"
        );
        assert_eq!(
            pty_key_bytes(&key(KeyCode::Char('é'), KeyModifiers::NONE), false),
            "é".as_bytes()
        );
    }

    #[test]
    fn function_and_editing_keys_use_xterm_sequences() {
        assert_eq!(
            pty_key_bytes(&key(KeyCode::F(1), KeyModifiers::NONE), false),
            b"\x1bOP"
        );
        assert_eq!(
            pty_key_bytes(&key(KeyCode::F(12), KeyModifiers::NONE), false),
            b"\x1b[24~"
        );
        assert_eq!(
            pty_key_bytes(&key(KeyCode::Delete, KeyModifiers::SHIFT), false),
            b"\x1b[3;2~"
        );
        assert_eq!(
            pty_key_bytes(&key(KeyCode::Enter, KeyModifiers::NONE), false),
            b"\r"
        );
    }
}
