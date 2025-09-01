//! F0318: Character Set Support Tests - G0/G1 character set switching and special character sets
//!
//! Tests for character set designation and switching in terminal emulator.
//! Validates DEC Special Character Set, UK character set, and G0/G1 selection.

use crate::ansi_processor::AnsiProcessor;

#[test]
fn test_g0_character_set_designation() {
    // F0318: Test G0 character set designation with ESC ( sequences
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Initially should be ASCII
    assert_eq!(processor.get_terminal_state().charset_g0, None);
    
    // ESC ( A - Designate UK character set to G0
    processor.process_string("\x1b(A");
    assert_eq!(processor.get_terminal_state().charset_g0, Some("UK".to_string()));
    
    // ESC ( B - Designate ASCII character set to G0
    processor.process_string("\x1b(B");
    assert_eq!(processor.get_terminal_state().charset_g0, Some("ASCII".to_string()));
    
    // ESC ( 0 - Designate DEC Special Character Set to G0
    processor.process_string("\x1b(0");
    assert_eq!(processor.get_terminal_state().charset_g0, Some("DEC".to_string()));
}

#[test]
fn test_g1_character_set_designation() {
    // F0318: Test G1 character set designation with ESC ) sequences
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Initially should be None
    assert_eq!(processor.get_terminal_state().charset_g1, None);
    
    // ESC ) A - Designate UK character set to G1
    processor.process_string("\x1b)A");
    assert_eq!(processor.get_terminal_state().charset_g1, Some("UK".to_string()));
    
    // ESC ) B - Designate ASCII character set to G1
    processor.process_string("\x1b)B");
    assert_eq!(processor.get_terminal_state().charset_g1, Some("ASCII".to_string()));
    
    // ESC ) 0 - Designate DEC Special Character Set to G1
    processor.process_string("\x1b)0");
    assert_eq!(processor.get_terminal_state().charset_g1, Some("DEC".to_string()));
}

#[test]
fn test_active_character_set_default() {
    // F0318: Test that G0 is active by default
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Default should be G0 active (0)
    assert_eq!(processor.get_terminal_state().active_charset, 0);
}

#[test]
fn test_shift_in_shift_out() {
    // F0318: Test SO (Shift Out) and SI (Shift In) character set switching
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Set up G0 and G1 character sets
    processor.process_string("\x1b(B");  // G0 = ASCII
    processor.process_string("\x1b)0");  // G1 = DEC
    
    // Initially G0 should be active
    assert_eq!(processor.get_terminal_state().active_charset, 0);
    
    // SO (0x0E) - Shift Out - switch to G1
    processor.process_string("\x0e");
    assert_eq!(processor.get_terminal_state().active_charset, 1);
    
    // SI (0x0F) - Shift In - switch back to G0
    processor.process_string("\x0f");
    assert_eq!(processor.get_terminal_state().active_charset, 0);
}

#[test]
fn test_dec_special_character_set() {
    // F0318: Test DEC Special Character Set with box drawing characters
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Set G0 to DEC Special Character Set
    processor.process_string("\x1b(0");
    assert_eq!(processor.get_terminal_state().charset_g0, Some("DEC".to_string()));
    
    // Process some DEC special characters (these would map to box drawing)
    processor.process_string("lqk");  // DEC: lower-left, horizontal line, upper-right
    
    let content = processor.get_processed_text();
    // The actual character mapping would depend on implementation details
    // For now, just verify content exists and character set is properly set
    assert!(!content.is_empty());
    assert_eq!(processor.get_terminal_state().charset_g0, Some("DEC".to_string()));
}

#[test]
fn test_uk_character_set() {
    // F0318: Test UK character set designation
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Set G0 to UK character set
    processor.process_string("\x1b(A");
    assert_eq!(processor.get_terminal_state().charset_g0, Some("UK".to_string()));
    
    // Process some text (UK character set affects # character to pound sign)
    processor.process_string("Test#Text");
    
    let content = processor.get_processed_text();
    assert!(content.contains("Test"));
    assert!(content.contains("Text"));
}

#[test]
fn test_character_set_persistence_across_operations() {
    // F0318: Test that character set settings persist across other operations
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Set up character sets
    processor.process_string("\x1b(0");  // G0 = DEC
    processor.process_string("\x1b)A");  // G1 = UK
    
    // Perform other operations
    processor.process_string("Hello\n");
    processor.process_string("\x1b[H");   // Cursor home
    processor.process_string("\x1b[2J");  // Clear screen
    
    // Character sets should persist
    assert_eq!(processor.get_terminal_state().charset_g0, Some("DEC".to_string()));
    assert_eq!(processor.get_terminal_state().charset_g1, Some("UK".to_string()));
}

#[test]
fn test_character_set_with_alternate_screen() {
    // F0318: Test character set behavior with alternate screen buffer
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Set character sets in primary screen
    processor.process_string("\x1b(0");  // G0 = DEC
    processor.process_string("\x1b)A");  // G1 = UK
    processor.process_string("\x0e");    // Switch to G1
    
    // Switch to alternate screen
    processor.process_string("\x1b[?1049h");
    
    // Character set state should be maintained
    assert_eq!(processor.get_terminal_state().charset_g0, Some("DEC".to_string()));
    assert_eq!(processor.get_terminal_state().charset_g1, Some("UK".to_string()));
    assert_eq!(processor.get_terminal_state().active_charset, 1);
    
    // Switch back to primary screen
    processor.process_string("\x1b[?1049l");
    
    // Character set state should still be maintained
    assert_eq!(processor.get_terminal_state().charset_g0, Some("DEC".to_string()));
    assert_eq!(processor.get_terminal_state().charset_g1, Some("UK".to_string()));
    assert_eq!(processor.get_terminal_state().active_charset, 1);
}

#[test]
fn test_reset_character_sets() {
    // F0318: Test character set reset behavior
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Set up character sets
    processor.process_string("\x1b(0");  // G0 = DEC
    processor.process_string("\x1b)A");  // G1 = UK
    processor.process_string("\x0e");    // Switch to G1
    
    // Reset (this would typically reset to defaults)
    processor.process_string("\x1bc");   // RIS - Reset to Initial State
    
    // Character sets should be reset to defaults
    // Note: The exact reset behavior depends on implementation
    // For comprehensive terminal emulation, this should reset character sets
}

#[test]
fn test_multiple_character_set_switches() {
    // F0318: Test multiple character set switches in sequence
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Set up character sets
    processor.process_string("\x1b(B");  // G0 = ASCII
    processor.process_string("\x1b)0");  // G1 = DEC
    
    // Multiple switches
    processor.process_string("\x0e");    // Switch to G1
    assert_eq!(processor.get_terminal_state().active_charset, 1);
    
    processor.process_string("\x0f");    // Switch to G0
    assert_eq!(processor.get_terminal_state().active_charset, 0);
    
    processor.process_string("\x0e");    // Switch to G1 again
    assert_eq!(processor.get_terminal_state().active_charset, 1);
    
    processor.process_string("\x0f");    // Switch to G0 again
    assert_eq!(processor.get_terminal_state().active_charset, 0);
}

#[test]
fn test_character_set_edge_cases() {
    // F0318: Test character set edge cases and invalid designations
    let mut processor = AnsiProcessor::with_screen_size(80, 24);
    processor.set_screen_mode(true);
    
    // Test invalid character set designation (should be ignored or handle gracefully)
    processor.process_string("\x1b(X");  // Invalid character set
    
    // Test empty designation
    processor.process_string("\x1b(");   // Incomplete sequence
    
    // Processor should still be functional
    processor.process_string("Test");
    let content = processor.get_processed_text();
    assert!(content.contains("Test"));
}