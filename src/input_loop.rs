use crate::thread_manager::Runnable;
use crate::utils::should_use_pty;
use crate::{handle_keypress, AppContext, FieldUpdate};
use crossterm::event::{
    poll, read, Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use std::sync::mpsc;
use std::time::Duration;

use crate::thread_manager::*;

use uuid::Uuid;

/// Convert crossterm KeyEvent to appropriate PTY input string
/// F0309: Enhanced input translation system with terminal mode awareness
pub fn format_key_for_pty_with_modes(
    code: KeyCode,
    modifiers: KeyModifiers,
    cursor_key_mode: bool,
    keypad_mode: bool,
) -> String {
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
        // F0309: Numeric keypad support with application keypad mode (must come before general Char case)
        KeyCode::Char('0') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 0 (handled specially when keypad mode matters)
            if keypad_mode {
                "\x1bOp".to_string()
            } else {
                "0".to_string()
            }
        }
        KeyCode::Char('1') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 1
            if keypad_mode {
                "\x1bOq".to_string()
            } else {
                "1".to_string()
            }
        }
        KeyCode::Char('2') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 2
            if keypad_mode {
                "\x1bOr".to_string()
            } else {
                "2".to_string()
            }
        }
        KeyCode::Char('3') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 3
            if keypad_mode {
                "\x1bOs".to_string()
            } else {
                "3".to_string()
            }
        }
        KeyCode::Char('4') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 4
            if keypad_mode {
                "\x1bOt".to_string()
            } else {
                "4".to_string()
            }
        }
        KeyCode::Char('5') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 5
            if keypad_mode {
                "\x1bOu".to_string()
            } else {
                "5".to_string()
            }
        }
        KeyCode::Char('6') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 6
            if keypad_mode {
                "\x1bOv".to_string()
            } else {
                "6".to_string()
            }
        }
        KeyCode::Char('7') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 7
            if keypad_mode {
                "\x1bOw".to_string()
            } else {
                "7".to_string()
            }
        }
        KeyCode::Char('8') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 8
            if keypad_mode {
                "\x1bOx".to_string()
            } else {
                "8".to_string()
            }
        }
        KeyCode::Char('9') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad 9
            if keypad_mode {
                "\x1bOy".to_string()
            } else {
                "9".to_string()
            }
        }
        KeyCode::Char('.') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad decimal point
            if keypad_mode {
                "\x1bOn".to_string()
            } else {
                ".".to_string()
            }
        }
        KeyCode::Char('+') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad plus
            if keypad_mode {
                "\x1bOk".to_string()
            } else {
                "+".to_string()
            }
        }
        KeyCode::Char('-') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad minus
            if keypad_mode {
                "\x1bOm".to_string()
            } else {
                "-".to_string()
            }
        }
        KeyCode::Char('*') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad multiply
            if keypad_mode {
                "\x1bOj".to_string()
            } else {
                "*".to_string()
            }
        }
        KeyCode::Char('/') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad divide
            if keypad_mode {
                "\x1bOo".to_string()
            } else {
                "/".to_string()
            }
        }
        KeyCode::Char('=') if modifiers.contains(KeyModifiers::SHIFT) => {
            // Numpad equals
            if keypad_mode {
                "\x1bOX".to_string()
            } else {
                "=".to_string()
            }
        }

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

        // F0309: Arrow keys with terminal mode awareness
        KeyCode::Up => {
            let base_seq = if cursor_key_mode { "\x1bOA" } else { "\x1b[A" };
            format_modified_key(base_seq, modifiers)
        }
        KeyCode::Down => {
            let base_seq = if cursor_key_mode { "\x1bOB" } else { "\x1b[B" };
            format_modified_key(base_seq, modifiers)
        }
        KeyCode::Right => {
            let base_seq = if cursor_key_mode { "\x1bOC" } else { "\x1b[C" };
            format_modified_key(base_seq, modifiers)
        }
        KeyCode::Left => {
            let base_seq = if cursor_key_mode { "\x1bOD" } else { "\x1b[D" };
            format_modified_key(base_seq, modifiers)
        }

        // F0309: Home/End with terminal mode awareness
        KeyCode::Home => {
            let base_seq = if cursor_key_mode { "\x1bOH" } else { "\x1b[H" };
            format_modified_key(base_seq, modifiers)
        }
        KeyCode::End => {
            let base_seq = if cursor_key_mode { "\x1bOF" } else { "\x1b[F" };
            format_modified_key(base_seq, modifiers)
        }

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

/// F0310: Convert crossterm mouse event to xterm mouse protocol sequence
/// Returns None if mouse reporting is disabled or event should not be reported
pub fn format_mouse_for_pty(
    kind: MouseEventKind,
    column: u16,
    row: u16,
    modifiers: KeyModifiers,
) -> Option<String> {
    use crate::ansi_processor::{MouseButton as AnsiMouseButton, MouseEventType, MouseModifiers};

    // Convert crossterm event to our mouse event types
    let (event_type, button) = match kind {
        MouseEventKind::Down(crossterm_button) => {
            let button = match crossterm_button {
                MouseButton::Left => Some(AnsiMouseButton::Left),
                MouseButton::Right => Some(AnsiMouseButton::Right),
                MouseButton::Middle => Some(AnsiMouseButton::Middle),
            };
            (MouseEventType::Press, button)
        }
        MouseEventKind::Up(crossterm_button) => {
            let button = match crossterm_button {
                MouseButton::Left => Some(AnsiMouseButton::Left),
                MouseButton::Right => Some(AnsiMouseButton::Right),
                MouseButton::Middle => Some(AnsiMouseButton::Middle),
            };
            (MouseEventType::Release, button)
        }
        MouseEventKind::Drag(_) => {
            // Motion events - button info not available in crossterm drag events
            (MouseEventType::Motion, None)
        }
        MouseEventKind::ScrollUp => (MouseEventType::Wheel, Some(AnsiMouseButton::WheelUp)),
        MouseEventKind::ScrollDown => (MouseEventType::Wheel, Some(AnsiMouseButton::WheelDown)),
        MouseEventKind::ScrollLeft => (MouseEventType::Wheel, Some(AnsiMouseButton::WheelLeft)),
        MouseEventKind::ScrollRight => (MouseEventType::Wheel, Some(AnsiMouseButton::WheelRight)),
        _ => return None, // Ignore other event types
    };

    // Convert crossterm modifiers to our modifier struct
    let mouse_modifiers = MouseModifiers {
        shift: modifiers.contains(KeyModifiers::SHIFT),
        ctrl: modifiers.contains(KeyModifiers::CONTROL),
        alt: modifiers.contains(KeyModifiers::ALT),
        meta: modifiers.contains(KeyModifiers::SUPER),
    };

    // TODO: For now, generate a basic X10-style mouse report
    // In a full implementation, we would check the actual terminal state
    // to determine which mouse protocol mode is active
    generate_basic_mouse_report(
        event_type,
        column as usize,
        row as usize,
        button,
        mouse_modifiers,
    )
}

/// F0310: Generate basic mouse report for testing (simulates X10 mode)
/// This is a simplified version - full implementation should check terminal state
fn generate_basic_mouse_report(
    event_type: crate::ansi_processor::MouseEventType,
    x: usize,
    y: usize,
    button: Option<crate::ansi_processor::MouseButton>,
    modifiers: crate::ansi_processor::MouseModifiers,
) -> Option<String> {
    use crate::ansi_processor::{MouseButton as AnsiMouseButton, MouseEventType};

    // Only report press events for basic X10 compatibility
    if event_type != MouseEventType::Press && event_type != MouseEventType::Wheel {
        return None;
    }

    let mut cb = 0u8; // Control byte

    // Set button bits
    match button {
        Some(AnsiMouseButton::Left) => cb |= 0,
        Some(AnsiMouseButton::Middle) => cb |= 1,
        Some(AnsiMouseButton::Right) => cb |= 2,
        Some(AnsiMouseButton::WheelUp) => cb |= 64,
        Some(AnsiMouseButton::WheelDown) => cb |= 65,
        Some(AnsiMouseButton::WheelLeft) => cb |= 66,
        Some(AnsiMouseButton::WheelRight) => cb |= 67,
        None => return None,
    }

    // Set modifier bits
    if modifiers.shift {
        cb |= 4;
    }
    if modifiers.meta {
        cb |= 8;
    }
    if modifiers.ctrl {
        cb |= 16;
    }

    // Add base offset
    cb += 32;

    // Convert coordinates to 1-based and clamp to valid range
    let x = (x + 1).min(255);
    let y = (y + 1).min(255);

    // Generate X10-style escape sequence
    Some(format!(
        "\x1b[M{}{}{}",
        cb as char,
        (x + 32) as u8 as char,
        (y + 32) as u8 as char
    ))
}

/// F0309: Backwards-compatible wrapper for existing code
/// F0127: Enhanced special key handling with modifiers and extended key support
pub fn format_key_for_pty(code: KeyCode, modifiers: KeyModifiers) -> String {
    // Use normal mode defaults for backwards compatibility
    format_key_for_pty_with_modes(code, modifiers, false, false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_translation_system_cursor_keys() {
        // F0309: Test cursor key translation with terminal modes

        // Normal mode (DECCKM disabled) - default escape sequences
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Up, KeyModifiers::NONE, false, false),
            "\x1b[A"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Down, KeyModifiers::NONE, false, false),
            "\x1b[B"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Right, KeyModifiers::NONE, false, false),
            "\x1b[C"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Left, KeyModifiers::NONE, false, false),
            "\x1b[D"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Home, KeyModifiers::NONE, false, false),
            "\x1b[H"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::End, KeyModifiers::NONE, false, false),
            "\x1b[F"
        );

        // Application cursor key mode (DECCKM enabled) - SS3 sequences
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Up, KeyModifiers::NONE, true, false),
            "\x1bOA"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Down, KeyModifiers::NONE, true, false),
            "\x1bOB"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Right, KeyModifiers::NONE, true, false),
            "\x1bOC"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Left, KeyModifiers::NONE, true, false),
            "\x1bOD"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Home, KeyModifiers::NONE, true, false),
            "\x1bOH"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::End, KeyModifiers::NONE, true, false),
            "\x1bOF"
        );
    }

    #[test]
    fn test_input_translation_system_keypad_keys() {
        // F0309: Test keypad translation with application keypad mode (DECPAM)

        // Note: KeyCode::Char with SHIFT modifier simulates keypad keys for testing
        // Normal keypad mode (DECPAM disabled) - literal characters
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('0'), KeyModifiers::SHIFT, false, false),
            "0"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('1'), KeyModifiers::SHIFT, false, false),
            "1"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('5'), KeyModifiers::SHIFT, false, false),
            "5"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('9'), KeyModifiers::SHIFT, false, false),
            "9"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('.'), KeyModifiers::SHIFT, false, false),
            "."
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('+'), KeyModifiers::SHIFT, false, false),
            "+"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('-'), KeyModifiers::SHIFT, false, false),
            "-"
        );

        // Application keypad mode (DECPAM enabled) - SS3 sequences
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('0'), KeyModifiers::SHIFT, false, true),
            "\x1bOp"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('1'), KeyModifiers::SHIFT, false, true),
            "\x1bOq"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('2'), KeyModifiers::SHIFT, false, true),
            "\x1bOr"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('5'), KeyModifiers::SHIFT, false, true),
            "\x1bOu"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('9'), KeyModifiers::SHIFT, false, true),
            "\x1bOy"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('.'), KeyModifiers::SHIFT, false, true),
            "\x1bOn"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('+'), KeyModifiers::SHIFT, false, true),
            "\x1bOk"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('-'), KeyModifiers::SHIFT, false, true),
            "\x1bOm"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('*'), KeyModifiers::SHIFT, false, true),
            "\x1bOj"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('/'), KeyModifiers::SHIFT, false, true),
            "\x1bOo"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('='), KeyModifiers::SHIFT, false, true),
            "\x1bOX"
        );
    }

    #[test]
    fn test_input_translation_system_mode_combinations() {
        // F0309: Test various combinations of terminal modes

        // Both modes enabled
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Up, KeyModifiers::NONE, true, true),
            "\x1bOA"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('5'), KeyModifiers::SHIFT, true, true),
            "\x1bOu"
        );

        // Only cursor key mode enabled
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Left, KeyModifiers::NONE, true, false),
            "\x1bOD"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('3'), KeyModifiers::SHIFT, true, false),
            "3"
        );

        // Only keypad mode enabled
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Right, KeyModifiers::NONE, false, true),
            "\x1b[C"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Char('7'), KeyModifiers::SHIFT, false, true),
            "\x1bOw"
        );
    }

    #[test]
    fn test_input_translation_system_modifiers() {
        // F0309: Test that modified keys work correctly with terminal modes

        // Modified cursor keys in normal mode
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Up, KeyModifiers::SHIFT, false, false),
            "\x1b[1;2A"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Right, KeyModifiers::CONTROL, false, false),
            "\x1b[1;5C"
        );

        // Modified cursor keys in application mode
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Down, KeyModifiers::ALT, true, false),
            "\x1b[1;3B"
        );
        assert_eq!(
            format_key_for_pty_with_modes(
                KeyCode::Left,
                KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                true,
                false
            ),
            "\x1b[1;6D"
        );
    }

    #[test]
    fn test_input_translation_system_special_keys() {
        // F0309: Test that non-cursor/keypad keys are unaffected by terminal modes

        // Function keys should be the same regardless of modes
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::F(1), KeyModifiers::NONE, false, false),
            "\x1bOP"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::F(1), KeyModifiers::NONE, true, true),
            "\x1bOP"
        );

        // Tab, Enter, Escape should be unaffected
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Tab, KeyModifiers::NONE, false, false),
            "\t"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Tab, KeyModifiers::NONE, true, true),
            "\t"
        );

        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Enter, KeyModifiers::NONE, false, false),
            "\r"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Enter, KeyModifiers::NONE, true, true),
            "\r"
        );

        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Esc, KeyModifiers::NONE, false, false),
            "\x1b"
        );
        assert_eq!(
            format_key_for_pty_with_modes(KeyCode::Esc, KeyModifiers::NONE, true, true),
            "\x1b"
        );
    }

    #[test]
    fn test_backwards_compatibility() {
        // F0309: Test that the backwards-compatible function works correctly

        // Should use normal mode defaults (false, false)
        assert_eq!(
            format_key_for_pty(KeyCode::Up, KeyModifiers::NONE),
            "\x1b[A"
        );
        assert_eq!(
            format_key_for_pty(KeyCode::Char('5'), KeyModifiers::SHIFT),
            "5"
        );

        // Should match the explicit call with false modes
        assert_eq!(
            format_key_for_pty(KeyCode::Left, KeyModifiers::CONTROL),
            format_key_for_pty_with_modes(KeyCode::Left, KeyModifiers::CONTROL, false, false)
        );
    }

    #[test]
    fn test_input_translation_comprehensive_sequences() {
        // F0309: Comprehensive test of all key sequences

        // Test all arrow keys in both modes
        let arrow_keys = [
            (KeyCode::Up, "\x1b[A", "\x1bOA"),
            (KeyCode::Down, "\x1b[B", "\x1bOB"),
            (KeyCode::Right, "\x1b[C", "\x1bOC"),
            (KeyCode::Left, "\x1b[D", "\x1bOD"),
        ];

        for (key, normal_seq, app_seq) in arrow_keys.iter() {
            assert_eq!(
                format_key_for_pty_with_modes(*key, KeyModifiers::NONE, false, false),
                *normal_seq
            );
            assert_eq!(
                format_key_for_pty_with_modes(*key, KeyModifiers::NONE, true, false),
                *app_seq
            );
        }

        // Test all numeric keypad keys in both modes
        let keypad_keys = [
            ('0', "\x1bOp"),
            ('1', "\x1bOq"),
            ('2', "\x1bOr"),
            ('3', "\x1bOs"),
            ('4', "\x1bOt"),
            ('5', "\x1bOu"),
            ('6', "\x1bOv"),
            ('7', "\x1bOw"),
            ('8', "\x1bOx"),
            ('9', "\x1bOy"),
            ('.', "\x1bOn"),
            ('+', "\x1bOk"),
            ('-', "\x1bOm"),
            ('*', "\x1bOj"),
            ('/', "\x1bOo"),
            ('=', "\x1bOX"),
        ];

        for (key_char, app_seq) in keypad_keys.iter() {
            // Normal mode - literal character
            assert_eq!(
                format_key_for_pty_with_modes(
                    KeyCode::Char(*key_char),
                    KeyModifiers::SHIFT,
                    false,
                    false
                ),
                key_char.to_string()
            );
            // Application keypad mode - escape sequence
            assert_eq!(
                format_key_for_pty_with_modes(
                    KeyCode::Char(*key_char),
                    KeyModifiers::SHIFT,
                    false,
                    true
                ),
                *app_seq
            );
        }
    }
}
create_runnable!(
    InputLoop,
    |_inner: &mut RunnableImpl, _app_context: AppContext, _messages: Vec<Message>| -> bool { true },
    |inner: &mut RunnableImpl,
     mut app_context: AppContext,
     _messages: Vec<Message>|
     -> (bool, AppContext) {
        let mut should_continue = true;

        let active_layout = app_context.app.get_active_layout().unwrap().clone();

        if poll(Duration::from_millis(10)).unwrap() {
            if let Ok(event) = read() {
                let key_str = match event {
                    Event::Mouse(MouseEvent {
                        kind,
                        column,
                        row,
                        modifiers,
                    }) => {
                        // F0310: Check if focused muxbox has PTY with mouse reporting enabled
                        let selected_muxboxes = active_layout.get_selected_muxboxes();
                        let focused_muxbox_has_pty = selected_muxboxes
                            .first()
                            .map(|muxbox| should_use_pty(muxbox))
                            .unwrap_or(false);

                        if focused_muxbox_has_pty {
                            // F0310: Send mouse event to PTY for proper terminal state-aware processing
                            if let Some(focused_muxbox) = selected_muxboxes.first() {
                                inner.send_message(Message::PTYMouseEvent(
                                    focused_muxbox.id.clone(),
                                    kind,
                                    column,
                                    row,
                                    modifiers,
                                ));
                                return (true, app_context);
                            }
                        }

                        // Fall back to BoxMux UI mouse handling
                        match kind {
                            MouseEventKind::ScrollUp => {
                                inner.send_message(Message::ScrollMuxBoxUp());
                                "ScrollUp".to_string()
                            }
                            MouseEventKind::ScrollDown => {
                                inner.send_message(Message::ScrollMuxBoxDown());
                                "ScrollDown".to_string()
                            }
                            MouseEventKind::ScrollLeft => {
                                inner.send_message(Message::ScrollMuxBoxLeft());
                                "ScrollLeft".to_string()
                            }
                            MouseEventKind::ScrollRight => {
                                inner.send_message(Message::ScrollMuxBoxRight());
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
                        // Check if focused muxbox has PTY enabled - if so, route input to PTY
                        let selected_muxboxes = active_layout.get_selected_muxboxes();
                        let focused_muxbox_has_pty = selected_muxboxes
                            .first()
                            .map(|muxbox| should_use_pty(muxbox))
                            .unwrap_or(false);

                        if focused_muxbox_has_pty {
                            // F0309: Convert key event to string with terminal mode awareness
                            // TODO: Get actual terminal modes from focused muxbox's terminal state
                            let cursor_key_mode = false; // Default: normal cursor keys
                            let keypad_mode = false; // Default: normal keypad

                            let key_str = format_key_for_pty_with_modes(
                                code,
                                modifiers,
                                cursor_key_mode,
                                keypad_mode,
                            );

                            if let Some(focused_muxbox) = selected_muxboxes.first() {
                                inner.send_message(Message::PTYInput(
                                    focused_muxbox.id.clone(),
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
                                inner.send_message(Message::NextMuxBox());
                                "Tab".to_string()
                            }
                            KeyCode::BackTab => {
                                inner.send_message(Message::PreviousMuxBox());
                                "BackTab".to_string()
                            }
                            KeyCode::Enter => "Enter".to_string(),
                            KeyCode::Down => {
                                inner.send_message(Message::ScrollMuxBoxDown());
                                "Down".to_string()
                            }
                            KeyCode::Up => {
                                inner.send_message(Message::ScrollMuxBoxUp());
                                "Up".to_string()
                            }
                            KeyCode::Left => {
                                inner.send_message(Message::ScrollMuxBoxLeft());
                                "Left".to_string()
                            }
                            KeyCode::Right => {
                                inner.send_message(Message::ScrollMuxBoxRight());
                                "Right".to_string()
                            }
                            KeyCode::PageUp => {
                                if modifiers.contains(KeyModifiers::SHIFT) {
                                    inner.send_message(Message::ScrollMuxBoxPageLeft());
                                    "Shift+PageUp".to_string()
                                } else {
                                    inner.send_message(Message::ScrollMuxBoxPageUp());
                                    "PageUp".to_string()
                                }
                            }
                            KeyCode::PageDown => {
                                if modifiers.contains(KeyModifiers::SHIFT) {
                                    inner.send_message(Message::ScrollMuxBoxPageRight());
                                    "Shift+PageDown".to_string()
                                } else {
                                    inner.send_message(Message::ScrollMuxBoxPageDown());
                                    "PageDown".to_string()
                                }
                            }
                            KeyCode::Char(c) => {
                                if modifiers.contains(KeyModifiers::CONTROL) {
                                    match c {
                                        'c' => {
                                            inner.send_message(Message::CopyFocusedMuxBoxContent());
                                            "Ctrl+c".to_string()
                                        }
                                        _ => format!("Ctrl+{}", c),
                                    }
                                } else if modifiers.contains(KeyModifiers::SUPER) {
                                    match c {
                                        'c' => {
                                            inner.send_message(Message::CopyFocusedMuxBoxContent());
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
                                    inner.send_message(Message::ScrollMuxBoxToTop());
                                    "Ctrl+Home".to_string()
                                } else {
                                    // Home: scroll to beginning horizontally
                                    inner.send_message(Message::ScrollMuxBoxToBeginning());
                                    "Home".to_string()
                                }
                            }
                            KeyCode::End => {
                                if modifiers.contains(KeyModifiers::CONTROL) {
                                    // Ctrl+End: scroll to bottom vertically
                                    inner.send_message(Message::ScrollMuxBoxToBottom());
                                    "Ctrl+End".to_string()
                                } else {
                                    // End: scroll to end horizontally
                                    inner.send_message(Message::ScrollMuxBoxToEnd());
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

                // T0317: UNIFIED ARCHITECTURE - Replace direct run_script calls with ExecuteScript messages
                if let Some(app_key_mappings) = &app_context.app.on_keypress {
                    if let Some(actions) = handle_keypress(&key_str, app_key_mappings) {
                        let libs = app_context.app.libs.clone();

                        // Create ExecuteScript message for app-level keypress handlers
                        use crate::model::common::{
                            ExecuteScript, ExecutionMode, ExecutionSource, SourceReference,
                            SourceType,
                        };

                        // Register execution source and get stream_id
                        let target_box_id = "app_level".to_string(); // App level doesn't target a specific box
                        let source_type = crate::model::common::ExecutionSourceType::SocketUpdate {
                            command_type: format!("app_keypress_{}", key_str),
                        };
                        let stream_id = app_context
                            .app
                            .register_execution_source(source_type, target_box_id.clone());

                        let execute_script = ExecuteScript {
                            script: actions,
                            source: ExecutionSource {
                                source_type: SourceType::SocketUpdate,
                                source_id: format!("app_keypress_{}", key_str),
                                source_reference: SourceReference::SocketCommand(format!(
                                    "app keypress: {}",
                                    key_str
                                )),
                            },
                            execution_mode: ExecutionMode::Immediate, // App-level handlers use immediate execution
                            target_box_id,
                            libs: libs.unwrap_or_default(),
                            redirect_output: None,
                            append_output: false,
                            stream_id,
                            target_bounds: None, // App-level commands don't target specific muxboxes
                        };

                        inner.send_message(Message::ExecuteScriptMessage(execute_script));
                        log::info!(
                            "T0317: ExecuteScript message sent for app keypress handler ({})",
                            key_str
                        );
                    }
                }
                if let Some(layout_key_mappings) = &active_layout.on_keypress {
                    if let Some(actions) = handle_keypress(&key_str, layout_key_mappings) {
                        let libs = app_context.app.libs.clone();

                        // Create ExecuteScript message for layout-level keypress handlers
                        use crate::model::common::{
                            ExecuteScript, ExecutionMode, ExecutionSource, SourceReference,
                            SourceType,
                        };

                        // Register execution source and get stream_id
                        let target_box_id = format!("layout_{}", active_layout.id);
                        let source_type = crate::model::common::ExecutionSourceType::SocketUpdate {
                            command_type: format!("layout_keypress_{}", key_str),
                        };
                        let stream_id = app_context
                            .app
                            .register_execution_source(source_type, target_box_id.clone());

                        let execute_script = ExecuteScript {
                            script: actions,
                            source: ExecutionSource {
                                source_type: SourceType::SocketUpdate,
                                source_id: format!("layout_keypress_{}", key_str),
                                source_reference: SourceReference::SocketCommand(format!(
                                    "layout keypress: {}",
                                    key_str
                                )),
                            },
                            execution_mode: ExecutionMode::Immediate, // Layout-level handlers use immediate execution
                            target_box_id,
                            libs: libs.unwrap_or_default(),
                            redirect_output: None,
                            append_output: false,
                            stream_id,
                            target_bounds: None, // App-level commands don't target specific muxboxes
                        };

                        inner.send_message(Message::ExecuteScriptMessage(execute_script));
                        log::info!(
                            "T0317: ExecuteScript message sent for layout keypress handler ({})",
                            key_str
                        );
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
