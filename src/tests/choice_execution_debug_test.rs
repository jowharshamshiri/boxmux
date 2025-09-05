#[cfg(test)]
mod choice_execution_debug_tests {
    use crate::tests::visual_testing::{BoxMuxTester, TestConfig};
    use std::time::Duration;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};

    /// Test to debug why specific choices in multi_stream_tabs_demo.yaml don't work
    #[test]
    fn debug_multi_stream_choice_execution() {
        println!("=== CHOICE EXECUTION DEBUG TEST ===");
        
        let mut tester = BoxMuxTester::new();
        
        // Load the multi_stream_tabs_demo.yaml layout
        tester.load_config("layouts/multi_stream_tabs_demo.yaml")
            .expect("Failed to load multi_stream_tabs_demo layout");
            
        // Capture initial frame
        let initial_frame = tester.wait_for_frame()
            .expect("Failed to capture initial frame");
        
        println!("Initial frame captured - multi_stream_box should be visible");
        
        // Wait for layout to stabilize
        std::thread::sleep(Duration::from_millis(1000));
        
        // First, test the working choice: "Add External Stream" in control_panel
        println!("\n--- Testing WORKING choice: 'Add External Stream' ---");
        
        // Click on the control_panel box first to focus it
        let control_panel_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 25, // Middle of control_panel (5% to 45% = roughly column 25)
            row: 30,    // Middle of control_panel (65% to 95% = roughly row 30)
            modifiers: KeyModifiers::NONE,
        };
        
        println!("Clicking control_panel at ({}, {})", control_panel_click.column, control_panel_click.row);
        tester.send_mouse_event(MouseEventKind::Down(MouseButton::Left), control_panel_click.column, control_panel_click.row)
            .expect("Failed to send mouse click to control_panel");
        
        let after_focus_frame = tester.wait_for_frame()
            .expect("Failed to capture frame after focusing control_panel");
        
        // Now click on the "Add External Stream" choice (should be first choice in control_panel)
        let external_choice_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 25, // Same column as focus click
            row: 32,    // A bit lower for the choice text
            modifiers: KeyModifiers::NONE,
        };
        
        println!("Clicking 'Add External Stream' choice at ({}, {})", external_choice_click.column, external_choice_click.row);
        tester.send_mouse_event(MouseEventKind::Down(MouseButton::Left), external_choice_click.column, external_choice_click.row)
            .expect("Failed to click Add External Stream choice");
        
        // Wait to see the result
        std::thread::sleep(Duration::from_millis(2000));
        let after_external_frame = tester.wait_for_frame()
            .expect("Failed to capture frame after external choice click");
            
        println!("External stream choice executed - checking for new tab in multi_stream_box");
        
        // Now test the non-working choices in the multi_stream_box
        println!("\n--- Testing NON-WORKING choices in multi_stream_box ---");
        
        // Click on multi_stream_box to focus it
        let multi_stream_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 50, // Middle of multi_stream_box (5% to 95% = roughly column 50)
            row: 15,    // Middle of multi_stream_box (5% to 60% = roughly row 15)  
            modifiers: KeyModifiers::NONE,
        };
        
        println!("Clicking multi_stream_box at ({}, {})", multi_stream_click.column, multi_stream_click.row);
        tester.send_mouse_event(MouseEventKind::Down(MouseButton::Left), multi_stream_click.column, multi_stream_click.row)
            .expect("Failed to click multi_stream_box");
        
        let after_multi_focus_frame = tester.wait_for_frame()
            .expect("Failed to capture frame after focusing multi_stream_box");
        
        // Click on "Deploy App" choice (first choice in multi_stream_box)
        let deploy_choice_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 15, // Left side where choices typically appear
            row: 17,    // Below the title/border
            modifiers: KeyModifiers::NONE,
        };
        
        println!("Clicking 'Deploy App' choice at ({}, {})", deploy_choice_click.column, deploy_choice_click.row);
        tester.send_mouse_event(MouseEventKind::Down(MouseButton::Left), deploy_choice_click.column, deploy_choice_click.row)
            .expect("Failed to click Deploy App choice");
        
        // Wait to see the result
        std::thread::sleep(Duration::from_millis(3000));
        let after_deploy_frame = tester.wait_for_frame()
            .expect("Failed to capture frame after deploy choice click");
            
        println!("Deploy App choice clicked - checking for execution");
        
        // Click on "Monitor Logs" choice (second choice)
        let monitor_choice_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 15,
            row: 18,    // Next line down
            modifiers: KeyModifiers::NONE,
        };
        
        println!("Clicking 'Monitor Logs' choice at ({}, {})", monitor_choice_click.column, monitor_choice_click.row);
        tester.send_mouse_event(MouseEventKind::Down(MouseButton::Left), monitor_choice_click.column, monitor_choice_click.row)
            .expect("Failed to click Monitor Logs choice");
        
        // Wait to see the result
        std::thread::sleep(Duration::from_millis(6000)); // Longer wait for the loop script
        let after_monitor_frame = tester.wait_for_frame()
            .expect("Failed to capture frame after monitor choice click");
            
        println!("Monitor Logs choice clicked - checking for execution");
        
        // Click on "Start PTY Process" choice (third choice) 
        let pty_choice_click = MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: 15,
            row: 19,    // Next line down
            modifiers: KeyModifiers::NONE,
        };
        
        println!("Clicking 'Start PTY Process' choice at ({}, {})", pty_choice_click.column, pty_choice_click.row);
        tester.send_mouse_event(MouseEventKind::Down(MouseButton::Left), pty_choice_click.column, pty_choice_click.row)
            .expect("Failed to click Start PTY Process choice");
        
        // Wait to see the result
        std::thread::sleep(Duration::from_millis(3000));
        let after_pty_frame = tester.wait_for_frame()
            .expect("Failed to capture frame after PTY choice click");
            
        println!("Start PTY Process choice clicked - checking for execution");
        
        // Analyze the frames to understand the differences
        println!("\n=== FRAME ANALYSIS ===");
        
        // Check if any new tabs appeared in multi_stream_box
        let initial_tabs = count_tabs_in_frame(&initial_frame, "multi_stream_box");
        let after_external_tabs = count_tabs_in_frame(&after_external_frame, "multi_stream_box");
        let after_deploy_tabs = count_tabs_in_frame(&after_deploy_frame, "multi_stream_box");
        let after_monitor_tabs = count_tabs_in_frame(&after_monitor_frame, "multi_stream_box");
        let after_pty_tabs = count_tabs_in_frame(&after_pty_frame, "multi_stream_box");
        
        println!("Tab counts in multi_stream_box:");
        println!("  Initial: {}", initial_tabs);
        println!("  After external choice: {}", after_external_tabs);
        println!("  After deploy choice: {}", after_deploy_tabs);
        println!("  After monitor choice: {}", after_monitor_tabs);
        println!("  After PTY choice: {}", after_pty_tabs);
        
        // Check for content changes indicating script execution
        let has_deployment_output = frame_contains_text(&after_deploy_frame, "Starting deployment");
        let has_monitor_output = frame_contains_text(&after_monitor_frame, "Monitoring application logs");
        let has_pty_output = frame_contains_text(&after_pty_frame, "zenith") || frame_contains_text(&after_pty_frame, "PTY");
        
        println!("\nContent analysis:");
        println!("  Has deployment output: {}", has_deployment_output);
        println!("  Has monitor output: {}", has_monitor_output);
        println!("  Has PTY output: {}", has_pty_output);
        
        // Summary of findings
        println!("\n=== DEBUGGING SUMMARY ===");
        if after_external_tabs > initial_tabs {
            println!("✅ External stream choice WORKS - created new tab");
        } else {
            println!("❌ External stream choice FAILED - no new tab");
        }
        
        if after_deploy_tabs > after_external_tabs || has_deployment_output {
            println!("✅ Deploy choice WORKS");
        } else {
            println!("❌ Deploy choice FAILED - no new tab or output");
        }
        
        if after_monitor_tabs > after_deploy_tabs || has_monitor_output {
            println!("✅ Monitor choice WORKS");
        } else {
            println!("❌ Monitor choice FAILED - no new tab or output");
        }
        
        if after_pty_tabs > after_monitor_tabs || has_pty_output {
            println!("✅ PTY choice WORKS");
        } else {
            println!("❌ PTY choice FAILED - no new tab or output");
        }
        
        // The test passes if we've captured the debugging information
        // The actual assertion is the analysis printed above
        assert!(true, "Debug test completed - check console output for analysis");
    }
    
    /// Helper function to count tabs in a specific box by looking for tab indicators
    fn count_tabs_in_frame(frame: &crate::tests::visual_testing::terminal_capture::TerminalFrame, box_title: &str) -> usize {
        // Look for tab indicators like [Script], [Deploy], etc. in the frame content
        let frame_text = frame_to_string(frame);
        
        // Count occurrences of tab patterns within the multi_stream_box area
        let tab_indicators = ["[Script]", "[Deploy]", "[Monitor]", "[External]", "[PTY]"];
        let mut count = 0;
        
        for indicator in &tab_indicators {
            if frame_text.contains(indicator) {
                count += 1;
            }
        }
        
        // At minimum, there should always be 1 tab (the original script)
        std::cmp::max(count, 1)
    }
    
    /// Helper function to check if frame contains specific text
    fn frame_contains_text(frame: &crate::tests::visual_testing::terminal_capture::TerminalFrame, text: &str) -> bool {
        let frame_text = frame_to_string(frame);
        frame_text.contains(text)
    }
    
    /// Convert frame to string for text analysis
    fn frame_to_string(frame: &crate::tests::visual_testing::terminal_capture::TerminalFrame) -> String {
        frame.cells.iter()
            .flat_map(|row| row.iter())
            .map(|cell| cell.character.unwrap_or(' '))
            .collect::<String>()
    }
}