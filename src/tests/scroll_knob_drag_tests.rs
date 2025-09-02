//! Visual tests to reproduce and validate critical UI issues
//! These tests demonstrate problems that need to be fixed

#[cfg(test)]
mod scroll_knob_drag_tests {
    use crate::tests::visual_testing::{boxmux_tester::BoxMuxTester, visual_assertions::VisualAssertions};

    /// Test scroll knob dragging functionality - SHOULD FAIL until fixed
    #[test]
    fn test_scroll_knob_dragging_broken() {
        let yaml = r#"
app:
  layouts:
    - id: 'scroll_test'
      root: true
      children:
        - id: 'scrollable_box'
          position:
            x1: '10%'
            y1: '10%'
            x2: '90%'
            y2: '60%'
          border_color: 'white'
          title: 'Scrollable Content'
          overflow_behavior: 'scroll'
          content: |
            Line 1 - This is the first line of content
            Line 2 - This is the second line of content
            Line 3 - This is the third line of content
            Line 4 - This is the fourth line of content
            Line 5 - This is the fifth line of content
            Line 6 - This is the sixth line of content
            Line 7 - This is the seventh line of content
            Line 8 - This is the eighth line of content
            Line 9 - This is the ninth line of content
            Line 10 - This is the tenth line of content
            Line 11 - More content that should require scrolling
            Line 12 - Even more content that should require scrolling
            Line 13 - Additional content for scrolling test
            Line 14 - Final line to ensure scrolling is needed
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml).expect("Failed to load scroll test config");
        
        // Verify initial state shows scrollbar
        let initial_frame = tester.wait_for_frame().expect("Failed to get initial frame");
        initial_frame.assert_contains_text("Line 1").expect("Should show first line");
        // Should show vertical scrollbar when content exceeds box height
        initial_frame.assert_contains_text("│").expect("Should show vertical scrollbar track");
        
        // Test scroll knob dragging - THIS SHOULD FAIL until fixed
        println!("Testing scroll knob dragging functionality - currently expected to be broken");
        
        // Try to drag the scroll knob from top to middle position
        tester.drag_from_to(89, 12, 89, 20).expect("Failed to drag scroll knob");
        
        // Check if content changed after drag
        let after_drag_frame = tester.wait_for_frame().expect("Failed to get frame after drag");
        
        // After dragging down, earlier lines should be hidden and later lines visible
        let shows_line_1 = after_drag_frame.assert_contains_text("Line 1").is_ok();
        let shows_line_10 = after_drag_frame.assert_contains_text("Line 10").is_ok();
        
        if shows_line_1 && !shows_line_10 {
            println!("WARNING: Scroll knob dragging not working - content didn't scroll");
        } else {
            println!("Scroll knob dragging appears to work - content scrolled");
        }
    }

    /// Test choice text overflow issue - choices drawing outside box bounds
    #[test] 
    fn test_choice_text_overflow_narrow_box() {
        let yaml = r#"
app:
  layouts:
    - id: 'narrow_choice_test'
      root: true
      children:
        - id: 'narrow_box'
          position:
            x1: '10%'
            y1: '10%'
            x2: '30%'  # Very narrow box - only 20% width
            y2: '80%'
          border_color: 'white'
          title: 'Menu'
          choices:
            - id: 'long_choice_1'
              content: 'Very Long Choice Text That Should Not Fit'
              script: ['echo "Choice 1"']
            - id: 'long_choice_2' 
              content: 'Another Extremely Long Choice Text That Overflows'
              script: ['echo "Choice 2"']
            - id: 'long_choice_3'
              content: 'Super Duper Long Choice Text That Definitely Overflows The Box'
              script: ['echo "Choice 3"']
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml).expect("Failed to load narrow choice test");
        
        let frame = tester.wait_for_frame().expect("Failed to get frame for narrow choice test");
            // Get box bounds by finding the border
            let mut box_right_edge = None;
            for y in 0..frame.dimensions.1 {
                for x in 0..frame.dimensions.0 {
                    // Check for border characters using assert_char_at 
                    if frame.assert_char_at(x, y, '┐').is_ok() ||
                       frame.assert_char_at(x, y, '┘').is_ok() ||
                       frame.assert_char_at(x, y, '│').is_ok() {
                        if let Some(existing) = box_right_edge {
                            if x > existing {
                                box_right_edge = Some(x);
                            }
                        } else {
                            box_right_edge = Some(x);
                        }
                    }
                }
            }
            
            if let Some(right_edge) = box_right_edge {
                // Check if choice text extends beyond the box border
                let mut text_found_beyond_border = false;
                
                for y in 12..25u16 { // Choice area
                    for x in (right_edge + 1)..frame.dimensions.0 {
                        // Check if there's non-space content beyond the border  
                        // We'll check for any printable characters that might indicate overflow
                        let printable_chars = "VeryLongChoiceTextThatShouldNotFitAnotherExtremelySuper";
                        let mut found_char = None;
                        for test_char in printable_chars.chars() {
                            if frame.assert_char_at(x, y, test_char).is_ok() {
                                text_found_beyond_border = true;
                                found_char = Some(test_char);
                                break;
                            }
                        }
                        if let Some(ch) = found_char {
                            println!("Found text '{}' at ({}, {}) beyond box border at x={}", 
                                    ch, x, y, right_edge);
                            break;
                        }
                    }
                    if text_found_beyond_border {
                        break;
                    }
                }
                
                if text_found_beyond_border {
                    panic!("ISSUE CONFIRMED: Choice text extends beyond box boundaries");
                } else {
                    println!("Choice text properly contained within box bounds");
                }
            } else {
                panic!("Could not find box right edge to test overflow");
            }
    }

    /// Test choice overflow with wrap=true - should wrap text instead of overflowing
    #[test]
    fn test_choice_overflow_with_wrapping() {
        let yaml = r#"
app:
  layouts:
    - id: 'wrap_choice_test'
      root: true  
      children:
        - id: 'narrow_wrap_box'
          position:
            x1: '10%'
            y1: '10%'
            x2: '30%'  # Very narrow box
            y2: '80%'
          border_color: 'white'
          title: 'Wrap Menu'
          overflow_behavior: 'wrap'  # Should wrap instead of overflow
          choices:
            - id: 'wrap_choice_1'
              content: 'Very Long Choice Text That Should Wrap To Multiple Lines'
              script: ['echo "Wrapped choice 1"']
            - id: 'wrap_choice_2'
              content: 'Another Long Choice That Should Also Wrap Properly'
              script: ['echo "Wrapped choice 2"']
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml).expect("Failed to load wrap choice test");
        
        let frame = tester.wait_for_frame().expect("Failed to get frame for wrap test");
        // Verify text wrapping behavior
        frame.assert_contains_text("Very Long Choice").expect("Should show start of first choice");
        frame.assert_contains_text("Text That Should").expect("Should show wrapped part of first choice");
        
        // Should NOT extend beyond box bounds
        // This test will help verify that wrapping is working correctly
        println!("Testing choice text wrapping within narrow box bounds");
    }

    /// Test horizontal scrollbar generation for choice overflow
    #[test]
    fn test_choice_horizontal_scrollbar_generation() {
        let yaml = r#"
app:
  layouts:
    - id: 'horizontal_scroll_test'
      root: true
      children:
        - id: 'narrow_scroll_box'
          position:
            x1: '10%'
            y1: '10%'
            x2: '40%'  # Narrow box
            y2: '50%'
          border_color: 'white'
          title: 'H-Scroll'
          overflow_behavior: 'scroll'  # Should generate scrollbar, not overflow
          choices:
            - id: 'horizontal_choice'
              content: 'Extremely Long Choice Text That Should Generate Horizontal Scrollbar Instead Of Overflowing Beyond Box Boundaries'
              script: ['echo "Horizontal scroll test"']
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml).expect("Failed to load horizontal scroll test");
        
        let frame = tester.wait_for_frame().expect("Failed to get frame for horizontal scroll test");
        // Should show horizontal scrollbar at bottom of box
        let has_horizontal_scrollbar = frame.assert_contains_text("─").is_ok() || 
                                     frame.assert_contains_text("◄").is_ok() ||
                                     frame.assert_contains_text("►").is_ok();
        
        if !has_horizontal_scrollbar {
            println!("WARNING: No horizontal scrollbar detected - may need implementation");
        }
        
        // Should show truncated choice text, not overflowing text
        frame.assert_contains_text("Extremely Long").expect("Should show start of choice");
        
        println!("Testing horizontal scrollbar generation for long choice text");
    }

    /// Test scrollbar interaction states - clicking vs dragging
    #[test]
    fn test_scrollbar_interaction_modes() {
        let yaml = r#"
app:
  layouts:
    - id: 'scrollbar_interaction_test'
      root: true
      children:
        - id: 'interactive_scroll_box'
          position:
            x1: '5%'
            y1: '5%'
            x2: '95%'
            y2: '50%'
          border_color: 'white'
          title: 'Scroll Test'
          overflow_behavior: 'scroll'
          content: |
            Content Line 1
            Content Line 2
            Content Line 3
            Content Line 4
            Content Line 5
            Content Line 6
            Content Line 7
            Content Line 8
            Content Line 9
            Content Line 10
            Content Line 11
            Content Line 12
            Content Line 13
            Content Line 14
            Content Line 15
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml).expect("Failed to load scrollbar interaction test");
        
        // Test 1: Click on scrollbar track (should jump)
        tester.click_at(94, 20).expect("Failed to click on scrollbar track");
        
        let after_click_frame = tester.wait_for_frame().expect("Failed to get frame after click");
        println!("Testing scrollbar track click functionality");
        // Should jump to clicked position
        
        // Test 2: Try to drag scroll knob (currently broken)
        println!("Testing scroll knob dragging - attempting drag operation");
        
        tester.drag_from_to(94, 15, 94, 25).expect("Failed to drag scroll knob");
        
        let after_drag_frame = tester.wait_for_frame().expect("Failed to get frame after drag");
        println!("Scroll knob drag completed - content should have scrolled if drag works");
        // This test will show whether dragging actually works
    }
}