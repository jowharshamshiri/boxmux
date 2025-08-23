use crossterm::event::{KeyCode, KeyModifiers};
use crate::input_loop::format_key_for_pty;

#[cfg(test)]
mod special_key_tests {
    use super::*;

    #[test]
    fn test_basic_character_keys() {
        // Basic character input
        assert_eq!(format_key_for_pty(KeyCode::Char('a'), KeyModifiers::NONE), "a");
        assert_eq!(format_key_for_pty(KeyCode::Char('Z'), KeyModifiers::NONE), "Z");
        assert_eq!(format_key_for_pty(KeyCode::Char('5'), KeyModifiers::NONE), "5");
        assert_eq!(format_key_for_pty(KeyCode::Char(' '), KeyModifiers::NONE), " ");
    }

    #[test]
    fn test_control_characters() {
        // Basic control characters
        assert_eq!(format_key_for_pty(KeyCode::Char('a'), KeyModifiers::CONTROL), "\x01");
        assert_eq!(format_key_for_pty(KeyCode::Char('z'), KeyModifiers::CONTROL), "\x1a");
        assert_eq!(format_key_for_pty(KeyCode::Char('A'), KeyModifiers::CONTROL), "\x01");
        assert_eq!(format_key_for_pty(KeyCode::Char('Z'), KeyModifiers::CONTROL), "\x1a");
        
        // Special control characters
        assert_eq!(format_key_for_pty(KeyCode::Char('@'), KeyModifiers::CONTROL), "\x00");
        assert_eq!(format_key_for_pty(KeyCode::Char('['), KeyModifiers::CONTROL), "\x1b");
        assert_eq!(format_key_for_pty(KeyCode::Char('\\'), KeyModifiers::CONTROL), "\x1c");
        assert_eq!(format_key_for_pty(KeyCode::Char(']'), KeyModifiers::CONTROL), "\x1d");
        assert_eq!(format_key_for_pty(KeyCode::Char('^'), KeyModifiers::CONTROL), "\x1e");
        assert_eq!(format_key_for_pty(KeyCode::Char('_'), KeyModifiers::CONTROL), "\x1f");
        assert_eq!(format_key_for_pty(KeyCode::Char(' '), KeyModifiers::CONTROL), "\x00");
    }

    #[test]
    fn test_alt_characters() {
        // Alt + character combinations
        assert_eq!(format_key_for_pty(KeyCode::Char('a'), KeyModifiers::ALT), "\x1ba");
        assert_eq!(format_key_for_pty(KeyCode::Char('Z'), KeyModifiers::ALT), "\x1bZ");
        assert_eq!(format_key_for_pty(KeyCode::Char('5'), KeyModifiers::ALT), "\x1b5");
    }

    #[test]
    fn test_special_keys() {
        // Basic special keys
        assert_eq!(format_key_for_pty(KeyCode::Enter, KeyModifiers::NONE), "\r");
        assert_eq!(format_key_for_pty(KeyCode::Tab, KeyModifiers::NONE), "\t");
        assert_eq!(format_key_for_pty(KeyCode::Backspace, KeyModifiers::NONE), "\x7f");
        assert_eq!(format_key_for_pty(KeyCode::Delete, KeyModifiers::NONE), "\x1b[3~");
        assert_eq!(format_key_for_pty(KeyCode::Insert, KeyModifiers::NONE), "\x1b[2~");
        assert_eq!(format_key_for_pty(KeyCode::Esc, KeyModifiers::NONE), "\x1b");
    }

    #[test]
    fn test_arrow_keys() {
        // Basic arrow keys
        assert_eq!(format_key_for_pty(KeyCode::Up, KeyModifiers::NONE), "\x1b[A");
        assert_eq!(format_key_for_pty(KeyCode::Down, KeyModifiers::NONE), "\x1b[B");
        assert_eq!(format_key_for_pty(KeyCode::Right, KeyModifiers::NONE), "\x1b[C");
        assert_eq!(format_key_for_pty(KeyCode::Left, KeyModifiers::NONE), "\x1b[D");
    }

    #[test]
    fn test_navigation_keys() {
        // Navigation keys
        assert_eq!(format_key_for_pty(KeyCode::Home, KeyModifiers::NONE), "\x1b[H");
        assert_eq!(format_key_for_pty(KeyCode::End, KeyModifiers::NONE), "\x1b[F");
        assert_eq!(format_key_for_pty(KeyCode::PageUp, KeyModifiers::NONE), "\x1b[5~");
        assert_eq!(format_key_for_pty(KeyCode::PageDown, KeyModifiers::NONE), "\x1b[6~");
    }

    #[test]
    fn test_function_keys() {
        // F1-F4 (using different escape sequences)
        assert_eq!(format_key_for_pty(KeyCode::F(1), KeyModifiers::NONE), "\x1bOP");
        assert_eq!(format_key_for_pty(KeyCode::F(2), KeyModifiers::NONE), "\x1bOQ");
        assert_eq!(format_key_for_pty(KeyCode::F(3), KeyModifiers::NONE), "\x1bOR");
        assert_eq!(format_key_for_pty(KeyCode::F(4), KeyModifiers::NONE), "\x1bOS");
        
        // F5-F12
        assert_eq!(format_key_for_pty(KeyCode::F(5), KeyModifiers::NONE), "\x1b[15~");
        assert_eq!(format_key_for_pty(KeyCode::F(6), KeyModifiers::NONE), "\x1b[17~");
        assert_eq!(format_key_for_pty(KeyCode::F(12), KeyModifiers::NONE), "\x1b[24~");
        
        // Extended function keys F13-F24
        assert_eq!(format_key_for_pty(KeyCode::F(13), KeyModifiers::NONE), "\x1b[25~");
        assert_eq!(format_key_for_pty(KeyCode::F(24), KeyModifiers::NONE), "\x1b[38~");
        
        // Invalid function key
        assert_eq!(format_key_for_pty(KeyCode::F(25), KeyModifiers::NONE), "");
    }

    #[test]
    fn test_modified_special_keys() {
        // Ctrl+Enter
        assert_eq!(format_key_for_pty(KeyCode::Enter, KeyModifiers::CONTROL), "\n");
        
        // Shift+Tab (Back Tab)
        assert_eq!(format_key_for_pty(KeyCode::Tab, KeyModifiers::SHIFT), "\x1b[Z");
        
        // Ctrl+Backspace
        assert_eq!(format_key_for_pty(KeyCode::Backspace, KeyModifiers::CONTROL), "\x08");
        
        // Alt+Backspace
        assert_eq!(format_key_for_pty(KeyCode::Backspace, KeyModifiers::ALT), "\x1b\x7f");
    }

    #[test]
    fn test_modifier_combinations() {
        let shift_alt = KeyModifiers::SHIFT | KeyModifiers::ALT;
        let ctrl_alt = KeyModifiers::CONTROL | KeyModifiers::ALT;
        let shift_ctrl = KeyModifiers::SHIFT | KeyModifiers::CONTROL;
        let all_mods = KeyModifiers::SHIFT | KeyModifiers::ALT | KeyModifiers::CONTROL;
        
        // Test that modifier combinations are handled (exact sequences depend on terminal)
        let result_shift_alt = format_key_for_pty(KeyCode::Up, shift_alt);
        let result_ctrl_alt = format_key_for_pty(KeyCode::Up, ctrl_alt);
        let result_shift_ctrl = format_key_for_pty(KeyCode::Up, shift_ctrl);
        let result_all = format_key_for_pty(KeyCode::Up, all_mods);
        
        // These should all produce different escape sequences
        assert_ne!(result_shift_alt, "\x1b[A");
        assert_ne!(result_ctrl_alt, "\x1b[A");
        assert_ne!(result_shift_ctrl, "\x1b[A");
        assert_ne!(result_all, "\x1b[A");
        
        // And they should be different from each other
        assert_ne!(result_shift_alt, result_ctrl_alt);
        assert_ne!(result_shift_alt, result_shift_ctrl);
        assert_ne!(result_ctrl_alt, result_all);
    }

    #[test]
    fn test_system_keys() {
        // System keys that should return empty strings
        assert_eq!(format_key_for_pty(KeyCode::CapsLock, KeyModifiers::NONE), "");
        assert_eq!(format_key_for_pty(KeyCode::ScrollLock, KeyModifiers::NONE), "");
        assert_eq!(format_key_for_pty(KeyCode::NumLock, KeyModifiers::NONE), "");
        assert_eq!(format_key_for_pty(KeyCode::PrintScreen, KeyModifiers::NONE), "");
        assert_eq!(format_key_for_pty(KeyCode::Pause, KeyModifiers::NONE), "");
        
        // Menu key
        assert_eq!(format_key_for_pty(KeyCode::Menu, KeyModifiers::NONE), "\x1b[29~");
    }

    #[test]
    fn test_modifier_code_calculation() {
        // Test the modifier code calculation logic by testing known combinations
        
        // Shift = +1, so mod_code = 2
        let shift_up = format_key_for_pty(KeyCode::Up, KeyModifiers::SHIFT);
        assert!(shift_up.contains("2"));
        
        // Alt = +2, so mod_code = 3  
        let alt_up = format_key_for_pty(KeyCode::Up, KeyModifiers::ALT);
        assert!(alt_up.contains("3"));
        
        // Ctrl = +4, so mod_code = 5
        let ctrl_up = format_key_for_pty(KeyCode::Up, KeyModifiers::CONTROL);
        assert!(ctrl_up.contains("5"));
        
        // Shift+Alt = +1+2 = +3, so mod_code = 4
        let shift_alt_up = format_key_for_pty(KeyCode::Up, KeyModifiers::SHIFT | KeyModifiers::ALT);
        assert!(shift_alt_up.contains("4"));
    }
}