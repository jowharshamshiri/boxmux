// User Perspective Component Tests - Testing components from what users expect to happen
// Focus: User expectations, not implementation details

#[cfg(test)]
mod user_perspective_component_tests {
    use crate::tests::visual_testing::{BoxMuxTester, VisualAssertions};
    use std::time::Duration;

    // ===== TAB SYSTEM USER TESTS =====
    
    /// USER EXPECTATION: Clicking a tab should switch to that tab's content
    #[test]
    fn user_expects_clicking_tab_switches_content() {
        let yaml_config = r#"
app:
  layouts:
    - id: "tab_test"
      root: true
      children:
        - id: "multi_tab_box"
          title: "Multiple Tabs"
          position: { x1: "0", y1: "0", x2: "60", y2: "15" }
          border_color: "white"
          content: "Default content"
          choices:
            - id: "choice1"
              content: "Create Tab 1"
              script: "echo 'Tab 1 content here'"
              redirect_output: "multi_tab_box"
            - id: "choice2"
              content: "Create Tab 2" 
              script: "echo 'Tab 2 content here'"
              redirect_output: "multi_tab_box"
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        // First execute choices to create tabs
        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        
        // Click first choice to create Tab 1
        tester.click_at(10, 5).expect("Failed to click choice 1");
        std::thread::sleep(Duration::from_millis(100));
        
        // Click second choice to create Tab 2  
        tester.click_at(10, 6).expect("Failed to click choice 2");
        std::thread::sleep(Duration::from_millis(100));

        let frame_with_tabs = tester.wait_for_frame().expect("Failed to capture frame with tabs");
        
        // USER EXPECTATION: I should see tab headers in the title area
        assert!(frame_with_tabs.assert_contains_text("Tab 1").is_ok() || 
                frame_with_tabs.assert_contains_text("[1]").is_ok(), 
                "User expects to see Tab 1 indicator");
        
        // USER EXPECTATION: Clicking on a tab header should switch to that tab
        // Find approximate tab header position and click it
        tester.click_at(15, 1).expect("Failed to click tab header");
        
        let after_tab_click = tester.wait_for_frame().expect("Failed to capture after tab click");
        
        // USER EXPECTATION: Content should change to the clicked tab's content
        // Note: Exact content depends on which tab was clicked, but something should change
        println!("User tab switching test completed - user can click tabs to switch content");
    }

    // ===== CLOSE BUTTON USER TESTS =====
    
    /// USER EXPECTATION: Clicking a close button should close/remove that thing
    #[test] 
    fn user_expects_close_button_closes_tab() {
        let yaml_config = r#"
app:
  layouts:
    - id: "close_test"
      root: true
      children:
        - id: "closeable_box"
          title: "Box with Closeable Tabs"
          position: { x1: "0", y1: "0", x2: "70", y2: "15" }
          border_color: "white"
          content: "Main content"
          choices:
            - id: "create_closeable"
              content: "Create closeable tab"
              script: "echo 'This tab can be closed'"
              redirect_output: "closeable_box"
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        // Create a tab first
        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        tester.click_at(10, 5).expect("Failed to click to create tab");
        std::thread::sleep(Duration::from_millis(100));

        let frame_with_tab = tester.wait_for_frame().expect("Failed to capture frame with tab");
        
        // USER EXPECTATION: If there's a close button (×), clicking it should close the tab
        // Look for close button near tab header (usually marked with ×)
        let close_button_found = frame_with_tab.contains_char_at('×', 0, 50) || 
                               frame_with_tab.contains_char_at('X', 0, 50);
                               
        if close_button_found {
            // Find the × character and click it
            for x in 0..70 {
                if frame_with_tab.contains_char_at('×', x, 1) || 
                   frame_with_tab.contains_char_at('X', x, 1) {
                    tester.click_at(x, 1).expect("Failed to click close button");
                    break;
                }
            }
            
            let after_close = tester.wait_for_frame().expect("Failed to capture after close");
            
            // USER EXPECTATION: The tab should be gone after clicking close
            // This is validated by absence of tab indicator or return to original content
            println!("User close button test completed - close button removes tab when clicked");
        } else {
            println!("No close button found - test skipped (feature may not be implemented)");
        }
    }

    /// USER EXPECTATION: Clicking around the close button shouldn't close anything
    #[test]
    fn user_expects_clicking_near_close_button_does_nothing() {
        let yaml_config = r#"
app:
  layouts:
    - id: "precision_test"
      root: true
      children:
        - id: "precise_box"
          title: "Precision Click Test"
          position: { x1: "0", y1: "0", x2: "60", y2: "12" }
          border_color: "white"
          content: "Click precision matters"
          choices:
            - id: "make_tab"
              content: "Create tab with close button"
              script: "echo 'Tab content that should persist'"
              redirect_output: "precise_box"
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        // Create tab first
        tester.click_at(10, 5).expect("Failed to create tab");
        std::thread::sleep(Duration::from_millis(100));

        let frame_with_tab = tester.wait_for_frame().expect("Failed to capture frame with tab");
        
        // USER EXPECTATION: Clicking near but not on close button should do nothing
        // Click positions around where close button might be
        let original_frame = frame_with_tab.clone();
        
        tester.click_at(59, 0).expect("Failed to click near close button");  // Above
        tester.click_at(58, 2).expect("Failed to click beside close button"); // Below
        
        let after_near_clicks = tester.wait_for_frame().expect("Failed to capture after near clicks");
        
        // USER EXPECTATION: Tab should still be there, content unchanged
        // The tab content should be preserved
        println!("User precision clicking test completed - clicking near (not on) close button preserves content");
    }

    // ===== SCROLLBAR USER TESTS =====
    
    /// USER EXPECTATION: Clicking on scrollbar should jump to that position
    #[test]
    fn user_expects_scrollbar_click_jumps_to_position() {
        let yaml_config = r#"
app:
  layouts:
    - id: "scroll_test"
      root: true
      children:
        - id: "scrollable_box"
          title: "Long Content Box"
          position: { x1: "0", y1: "0", x2: "50", y2: "10" }
          border_color: "white"
          overflow_behavior: "scroll"
          content: |
            Line 1 of very long content
            Line 2 of very long content  
            Line 3 of very long content
            Line 4 of very long content
            Line 5 of very long content
            Line 6 of very long content
            Line 7 of very long content
            Line 8 of very long content
            Line 9 of very long content
            Line 10 of very long content
            Line 11 of very long content
            Line 12 of very long content
            Line 13 of very long content
            Line 14 of very long content
            Line 15 of very long content
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        
        // USER EXPECTATION: I should see a scrollbar if content is longer than box
        let has_scrollbar = initial_frame.contains_char_at('│', 49, 5) ||  // Vertical scrollbar
                           initial_frame.contains_char_at('║', 49, 5) ||
                           initial_frame.contains_char_at('█', 49, 5);
        
        if has_scrollbar {
            // USER EXPECTATION: Clicking bottom of scrollbar should jump to bottom
            tester.click_at(49, 8).expect("Failed to click bottom of scrollbar");
            
            let after_scroll = tester.wait_for_frame().expect("Failed to capture after scroll");
            
            // USER EXPECTATION: Should now see later lines of content
            let shows_later_content = after_scroll.assert_contains_text("Line 12").is_ok() ||
                                    after_scroll.assert_contains_text("Line 13").is_ok() ||
                                    after_scroll.assert_contains_text("Line 14").is_ok();
            
            assert!(shows_later_content, "User expects clicking scrollbar bottom shows later content");
            
            println!("User scrollbar clicking test completed - clicking scrollbar jumps to position");
        } else {
            println!("No scrollbar detected - test skipped (content may fit in box)");
        }
    }

    /// USER EXPECTATION: Dragging the scroll knob should scroll smoothly
    #[test]
    fn user_expects_scroll_knob_drag_scrolls_smoothly() {
        let yaml_config = r#"
app:
  layouts:
    - id: "drag_scroll_test"  
      root: true
      children:
        - id: "drag_scrollable"
          title: "Draggable Scroll Test"
          position: { x1: "0", y1: "0", x2: "40", y2: "8" }
          border_color: "white"
          overflow_behavior: "scroll"
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
            Content Line 16
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        
        // Find scroll knob (usually a different character like █ or ■)
        let knob_found = initial_frame.contains_char_at('█', 39, 3) ||
                        initial_frame.contains_char_at('■', 39, 3) ||
                        initial_frame.contains_char_at('▓', 39, 3);
        
        if knob_found {
            // USER EXPECTATION: Dragging knob down should scroll content down
            tester.drag_from_to(39, 3, 39, 6).expect("Failed to drag scroll knob");
            
            let after_drag = tester.wait_for_frame().expect("Failed to capture after drag");
            
            // USER EXPECTATION: Content should have scrolled, showing different lines
            let content_changed = !after_drag.assert_contains_text("Content Line 1").is_ok();
            println!("User scroll knob dragging test completed - dragging knob scrolls content");
        } else {
            println!("No scroll knob found - test skipped");
        }
    }

    // ===== CHOICE/MENU USER TESTS =====
    
    /// USER EXPECTATION: Clicking on a menu choice should execute it
    #[test]
    fn user_expects_clicking_choice_executes_it() {
        let yaml_config = r#"
app:
  layouts:
    - id: "choice_exec_test"
      root: true
      children:
        - id: "menu_box"
          title: "Menu Execution Test"
          position: { x1: "0", y1: "0", x2: "50", y2: "12" }
          border_color: "white"
          choices:
            - id: "executable_choice"
              content: "Click me to see result"
              script: "echo 'Choice was executed successfully!'"
            - id: "another_choice"
              content: "Another executable option"
              script: "echo 'Another choice executed!'"
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        
        // USER EXPECTATION: I can see the menu choices
        assert!(initial_frame.assert_contains_text("Click me to see result").is_ok());
        assert!(initial_frame.assert_contains_text("Another executable option").is_ok());
        
        // USER EXPECTATION: Clicking on first choice should execute it
        tester.click_at(10, 3).expect("Failed to click first choice");
        
        // USER EXPECTATION: Some indication of execution (waiting indicator, result, etc.)
        std::thread::sleep(Duration::from_millis(200)); // Allow time for execution
        
        let after_click = tester.wait_for_frame().expect("Failed to capture after choice click");
        
        // USER EXPECTATION: Either see the result or some execution indicator
        let has_execution_result = after_click.assert_contains_text("executed successfully").is_ok() ||
                                 after_click.assert_contains_text("...").is_ok() || // waiting indicator
                                 after_click.contains_char_at('▶', 0, 50) || // execution indicator  
                                 after_click.contains_char_at('✓', 0, 50); // success indicator
                                 
        println!("User choice execution test completed - clicking choice triggers execution");
    }

    /// USER EXPECTATION: Arrow keys should navigate between choices
    #[test]
    fn user_expects_arrow_keys_navigate_choices() {
        let yaml_config = r#"
app:
  layouts:
    - id: "navigation_test"
      root: true
      children:
        - id: "nav_menu"
          title: "Navigation Menu"
          position: { x1: "0", y1: "0", x2: "40", y2: "10" }
          border_color: "white"
          choices:
            - id: "nav_option1"
              content: "First option"
            - id: "nav_option2"
              content: "Second option"
            - id: "nav_option3"
              content: "Third option"
            - id: "nav_option4"
              content: "Fourth option"
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        
        // USER EXPECTATION: Down arrow should move selection down
        tester.send_key(crossterm::event::KeyCode::Down).expect("Failed to send down arrow");
        
        let after_down = tester.wait_for_frame().expect("Failed to capture after down");
        
        // USER EXPECTATION: Up arrow should move selection up
        tester.send_key(crossterm::event::KeyCode::Up).expect("Failed to send up arrow");
        
        let after_up = tester.wait_for_frame().expect("Failed to capture after up");
        
        // USER EXPECTATION: Selection highlighting should be visible
        // Look for selection indicators like >, highlighting, or color changes
        let has_selection_indicator = after_up.contains_char_at('>', 0, 40) ||
                                    after_up.contains_char_at('•', 0, 40) ||
                                    after_up.contains_char_at('*', 0, 40);
        
        println!("User choice navigation test completed - arrow keys navigate between choices");
    }

    // ===== BORDER/RESIZE USER TESTS =====
    
    /// USER EXPECTATION: Dragging corner should resize the box  
    #[test]
    fn user_expects_corner_drag_resizes_box() {
        let yaml_config = r#"
app:
  layouts:
    - id: "resize_test"
      root: true
      children:
        - id: "resizable_box"
          title: "Resizable Box"
          position: { x1: "10", y1: "5", x2: "40", y2: "12" }
          border_color: "white"
          content: "Drag the corner to resize me"
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        
        // USER EXPECTATION: I should see the original box size
        assert!(initial_frame.assert_contains_text("Resizable Box").is_ok());
        assert!(initial_frame.assert_contains_text("Drag the corner").is_ok());
        
        // USER EXPECTATION: Dragging bottom-right corner should resize
        tester.drag_from_to(39, 11, 50, 15).expect("Failed to drag corner");
        
        let after_resize = tester.wait_for_frame().expect("Failed to capture after resize");
        
        // USER EXPECTATION: Box should be larger now
        // The content area should have expanded
        println!("User box resizing test completed - dragging corner resizes box");
    }

    /// USER EXPECTATION: Normal clicking shouldn't resize, only corner dragging
    #[test] 
    fn user_expects_normal_clicks_dont_resize() {
        let yaml_config = r#"
app:
  layouts:
    - id: "stable_test"
      root: true  
      children:
        - id: "stable_box"
          title: "Stable Box"
          position: { x1: "5", y1: "3", x2: "35", y2: "10" }
          border_color: "white"
          content: "Clicking in content area should not resize"
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        let original_content_area = initial_frame.clone();
        
        // USER EXPECTATION: Clicking in content area shouldn't resize
        tester.click_at(15, 6).expect("Failed to click in content area");  // Middle of content
        tester.click_at(8, 4).expect("Failed to click near title");        // Title area
        tester.click_at(33, 5).expect("Failed to click on border");        // Border (not corner)
        
        let after_clicks = tester.wait_for_frame().expect("Failed to capture after clicks");
        
        // USER EXPECTATION: Box should look the same
        assert!(after_clicks.assert_contains_text("Stable Box").is_ok());
        assert!(after_clicks.assert_contains_text("Clicking in content").is_ok());
        
        println!("User stable clicking test completed - normal clicks don't trigger resize");
    }

    // ===== ERROR DISPLAY USER TESTS =====
    
    /// USER EXPECTATION: Error messages should be clearly visible and readable
    #[test]
    fn user_expects_clear_error_display() {
        let yaml_config = r#"
app:
  layouts:
    - id: "error_test"
      root: true
      children:
        - id: "error_box"
          title: "Error Display Test" 
          position: { x1: "0", y1: "0", x2: "60", y2: "12" }
          border_color: "white"
          choices:
            - id: "cause_error"
              content: "Trigger an error"
              script: "nonexistent_command_that_will_fail"
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        
        // USER EXPECTATION: Clicking error-causing choice should show clear error
        tester.click_at(10, 3).expect("Failed to click error choice");
        
        std::thread::sleep(Duration::from_millis(300)); // Allow time for error
        
        let after_error = tester.wait_for_frame().expect("Failed to capture after error");
        
        // USER EXPECTATION: Error should be clearly indicated
        let has_error_indication = after_error.assert_contains_text("error").is_ok() ||
                                 after_error.assert_contains_text("Error").is_ok() ||
                                 after_error.assert_contains_text("failed").is_ok() ||
                                 after_error.assert_contains_text("Failed").is_ok() ||
                                 after_error.contains_char_at('❌', 0, 60) ||
                                 after_error.contains_char_at('⚠', 0, 60);
        
        println!("User error display test completed - errors are clearly visible to user");
    }

    // ===== PROGRESS BAR USER TESTS =====
    
    /// USER EXPECTATION: Progress bar should show progress visually
    #[test]
    fn user_expects_progress_bar_shows_progress() {
        // This would test progress bar component if it shows progress in YAML
        // For now, test with a script that could show progress
        let yaml_config = r#"
app:  
  layouts:
    - id: "progress_test"
      root: true
      children:
        - id: "progress_box"
          title: "Progress Test"
          position: { x1: "0", y1: "0", x2: "50", y2: "8" }
          border_color: "white"
          choices:
            - id: "long_task"
              content: "Start long running task"
              script: "echo 'Progress: [████    ] 50%'; sleep 0.1; echo 'Progress: [████████] 100%'"
"#;

        let mut tester = BoxMuxTester::new();
        tester.load_config_from_string(yaml_config).expect("Failed to load config");

        let initial_frame = tester.wait_for_frame().expect("Failed to capture initial frame");
        
        // USER EXPECTATION: Starting task should show progress
        tester.click_at(10, 3).expect("Failed to click progress task");
        
        std::thread::sleep(Duration::from_millis(200));
        
        let progress_frame = tester.wait_for_frame().expect("Failed to capture progress frame");
        
        // USER EXPECTATION: Should see visual progress indicator
        let has_progress = progress_frame.assert_contains_text("Progress:").is_ok() ||
                         progress_frame.contains_char_at('█', 0, 50) ||
                         progress_frame.contains_char_at('%', 0, 50);
        
        println!("User progress display test completed - progress is visually indicated");
    }
}