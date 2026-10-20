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
        // Shift+Tab: named like every other shifted special key.
        KeyCode::BackTab => "Tab".to_string(),
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
    let shifted = event.modifiers.contains(KeyModifiers::SHIFT) || event.code == KeyCode::BackTab;
    if shifted && !matches!(event.code, KeyCode::Char(_)) {
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

/// The keys besides single characters that bindings can name.
const SPECIAL_KEYS: [&str; 15] = [
    "Space",
    "Enter",
    "Tab",
    "Backspace",
    "Delete",
    "Insert",
    "Esc",
    "Home",
    "End",
    "PageUp",
    "PageDown",
    "Up",
    "Down",
    "Left",
    "Right",
];

/// Checks a key name written in `hot_keys` or `on_keypress`: modifiers (`Ctrl`, `Alt`, `Cmd`,
/// `Shift`, any case) joined by `+` to a single character, a special key (`Enter`, `PageUp`,
/// `F1` to `F24`...) or `+` itself. Names that could never match a key press are errors, and
/// so are the keys boxmux keeps for itself: `q` quits and, on Unix, `Ctrl+z` suspends.
pub fn check_key_name(configured: &str) -> Result<(), String> {
    let configured = configured.trim();
    let (modifiers, base) = match configured.strip_suffix("++") {
        Some(rest) => (rest, "+"),
        None if configured == "+" => ("", "+"),
        None => match configured.rsplit_once('+') {
            Some((modifiers, base)) => (modifiers, base),
            None => ("", configured),
        },
    };
    let mut seen = Vec::new();
    let named_modifiers = if modifiers.is_empty() {
        Vec::new()
    } else {
        modifiers.split('+').collect()
    };
    for modifier in named_modifiers {
        let known = ["ctrl", "alt", "cmd", "shift"];
        let lower = modifier.to_ascii_lowercase();
        if !known.contains(&lower.as_str()) {
            return Err(format!(
                "'{}' is not a modifier; use Ctrl, Alt, Cmd or Shift",
                modifier
            ));
        }
        if seen.contains(&lower) {
            return Err(format!("{} is named twice", modifier));
        }
        seen.push(lower);
    }
    let character = base.chars().count() == 1;
    let function_key = base
        .strip_prefix(['F', 'f'])
        .and_then(|n| n.parse::<u8>().ok())
        .is_some_and(|n| (1..=24).contains(&n));
    let special = SPECIAL_KEYS.iter().any(|k| k.eq_ignore_ascii_case(base));
    if base.is_empty() || !(character || function_key || special) {
        return Err(format!(
            "'{}' is not a key; use a single character, F1 to F24, or one of {}",
            base,
            SPECIAL_KEYS.join(", ")
        ));
    }
    if character && base.trim().is_empty() {
        return Err("write the space bar as Space".to_string());
    }
    if character && seen.contains(&"shift".to_string()) {
        return Err(format!(
            "Shift is part of the character: write the shifted character itself (for example A, not Shift+a), not Shift+{}",
            base
        ));
    }
    if seen.is_empty() && base == "q" {
        return Err("q always quits boxmux and cannot be bound".to_string());
    }
    if cfg!(unix) && seen == ["ctrl"] && base == "z" {
        return Err("Ctrl+z suspends boxmux and cannot be bound".to_string());
    }
    Ok(())
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
    #[test]
    fn key_names_in_bindings_are_checked() {
        for good in [
            "a",
            "A",
            "Ctrl+r",
            "ctrl+alt+Delete",
            "F12",
            "f1",
            "Shift+Tab",
            "Space",
            "+",
            "Ctrl++",
            "Cmd+PageUp",
        ] {
            assert_eq!(check_key_name(good), Ok(()), "{}", good);
        }
        for (bad, why) in [
            ("Ctrl+Enterr", "not a key"),
            ("F25", "not a key"),
            ("Hyper+a", "not a modifier"),
            ("Ctrl+Ctrl+a", "named twice"),
            ("Shift+a", "shifted character"),
            ("q", "quits"),
            ("", "not a key"),
            ("Ctrl+", "not a key"),
        ] {
            let error = check_key_name(bad).unwrap_err();
            assert!(error.contains(why), "{}: {}", bad, error);
        }
        // Only q itself quits; with a modifier it is an ordinary key.
        assert_eq!(check_key_name("Alt+q"), Ok(()));
        assert_eq!(check_key_name("Ctrl+z").is_err(), cfg!(unix));
    }

    #[test]
    fn every_name_a_key_press_gets_passes_the_check() {
        let codes = [
            KeyCode::Char('x'),
            KeyCode::Char('X'),
            KeyCode::Char(' '),
            KeyCode::Char('+'),
            KeyCode::Enter,
            KeyCode::BackTab,
            KeyCode::PageDown,
            KeyCode::F(24),
        ];
        let modifiers = [
            KeyModifiers::NONE,
            KeyModifiers::CONTROL,
            KeyModifiers::ALT | KeyModifiers::SUPER,
            KeyModifiers::SHIFT,
        ];
        for code in codes {
            for modifier in modifiers {
                let name = key_name(&KeyEvent::new(code, modifier)).unwrap();
                assert_eq!(check_key_name(&name), Ok(()), "{}", name);
            }
        }
    }

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
        // Terminals report Shift+Tab as BackTab, with or without the Shift flag.
        for modifiers in [KeyModifiers::NONE, KeyModifiers::SHIFT] {
            assert_eq!(
                key_name(&key(KeyCode::BackTab, modifiers)).unwrap(),
                "Shift+Tab"
            );
        }
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
