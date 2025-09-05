#[cfg(test)]
mod choice_redirect_debug {
    use crate::model::app::load_app_from_yaml;
    use crate::model::muxbox::Choice;
    use crate::{AppContext, Config};
    use std::path::Path;

    /// Test to analyze the differences between working and non-working choices
    #[test]
    fn analyze_choice_configurations() {
        println!("=== CHOICE CONFIGURATION ANALYSIS ===");
        
        // Load the multi_stream_tabs_demo layout
        let app = load_app_from_yaml("layouts/multi_stream_tabs_demo.yaml")
            .expect("Failed to load multi_stream_tabs_demo.yaml");
            
        let config = Config::default();
        let app_context = AppContext::new(app, config);
        
        // Find the relevant muxboxes
        let multi_stream_box = app_context.app.get_muxbox_by_id("multi_stream_box")
            .expect("multi_stream_box not found");
        let control_panel = app_context.app.get_muxbox_by_id("control_panel")
            .expect("control_panel not found");
            
        println!("\n--- WORKING CHOICE (control_panel) ---");
        if let Some(choices) = &control_panel.choices {
            for choice in choices {
                if choice.id == "external_stream" {
                    print_choice_analysis(choice, "WORKING");
                    break;
                }
            }
        }
        
        println!("\n--- NON-WORKING CHOICES (multi_stream_box) ---");
        if let Some(choices) = &multi_stream_box.choices {
            for choice in choices {
                match choice.id.as_str() {
                    "deploy" => print_choice_analysis(choice, "NON-WORKING"),
                    "monitor" => print_choice_analysis(choice, "NON-WORKING"), 
                    "pty_process" => print_choice_analysis(choice, "NON-WORKING"),
                    _ => {}
                }
            }
        }
        
        println!("\n--- MUXBOX COMPARISON ---");
        println!("multi_stream_box properties:");
        println!("  ID: {}", multi_stream_box.id);
        println!("  Has script: {}", multi_stream_box.script.is_some());
        println!("  Execution mode: {:?}", multi_stream_box.execution_mode);
        println!("  Choice count: {}", multi_stream_box.choices.as_ref().map_or(0, |c| c.len()));
        println!("  Has streams: {}", !multi_stream_box.streams.is_empty());
        println!("  Stream count: {}", multi_stream_box.streams.len());
        
        println!("\ncontrol_panel properties:");
        println!("  ID: {}", control_panel.id);
        println!("  Has script: {}", control_panel.script.is_some());
        println!("  Execution mode: {:?}", control_panel.execution_mode);
        println!("  Choice count: {}", control_panel.choices.as_ref().map_or(0, |c| c.len()));
        println!("  Has streams: {}", !control_panel.streams.is_empty());
        println!("  Stream count: {}", control_panel.streams.len());
        
        // Key insight: Check if both boxes can accept redirected output
        println!("\n--- REDIRECT TARGET VALIDATION ---");
        let target_box_id = "multi_stream_box";
        let target_exists = app_context.app.get_muxbox_by_id(target_box_id).is_some();
        println!("Target box '{}' exists: {}", target_box_id, target_exists);
        
        // The test itself - check for specific patterns that might cause issues
        println!("\n=== DEBUGGING CONCLUSIONS ===");
        
        // Check 1: Do all choices have the same redirect_output target?
        let mut all_redirect_to_same = true;
        let expected_target = "multi_stream_box";
        
        if let Some(choices) = &control_panel.choices {
            for choice in choices {
                if let Some(ref redirect) = choice.redirect_output {
                    if redirect != expected_target {
                        all_redirect_to_same = false;
                        println!("❌ control_panel choice '{}' redirects to '{}', expected '{}'", 
                                choice.id, redirect, expected_target);
                    }
                } else {
                    println!("❌ control_panel choice '{}' has no redirect_output", choice.id);
                }
            }
        }
        
        if let Some(choices) = &multi_stream_box.choices {
            for choice in choices {
                if let Some(ref redirect) = choice.redirect_output {
                    if redirect != expected_target {
                        all_redirect_to_same = false;
                        println!("❌ multi_stream_box choice '{}' redirects to '{}', expected '{}'", 
                                choice.id, redirect, expected_target);
                    }
                } else {
                    println!("❌ multi_stream_box choice '{}' has no redirect_output", choice.id);
                }
            }
        }
        
        if all_redirect_to_same {
            println!("✅ All choices redirect to the same target: '{}'", expected_target);
        }
        
        // Check 2: Are there differences in script complexity?
        println!("\n--- SCRIPT COMPLEXITY ANALYSIS ---");
        analyze_script_complexity();
        
        // This test always passes - it's for analysis only
        assert!(true, "Analysis complete - check console output for findings");
    }
    
    fn print_choice_analysis(choice: &Choice, status: &str) {
        println!("Choice '{}' ({}):", choice.id, status);
        println!("  Content: {:?}", choice.content);
        println!("  Script: {:?}", choice.script);
        println!("  Execution mode: {:?}", choice.execution_mode);
        println!("  Redirect output: {:?}", choice.redirect_output);
        println!("  Append output: {:?}", choice.append_output);
        println!("  Waiting: {}", choice.waiting);
    }
    
    fn analyze_script_complexity() {
        println!("Script complexity analysis:");
        
        // Working choice script:
        let working_script = vec![
            "echo 'This creates another tab in the multi-stream box'".to_string(),
            "echo 'External process output flowing to multi-stream box'".to_string()
        ];
        
        // Non-working choice scripts:
        let deploy_script = vec![
            "echo 'Starting deployment...'".to_string(),
            "sleep 2".to_string(), 
            "echo 'Deployment complete!'".to_string()
        ];
        
        let monitor_script = vec![
            "echo 'Monitoring application logs...'".to_string(),
            "for i in {1..5}; do echo \"Log entry $i\"; sleep 1; done".to_string()
        ];
        
        let pty_script = vec![
            "zenith".to_string()
        ];
        
        println!("  Working (external_stream): {} commands, simple echo", working_script.len());
        println!("  Deploy: {} commands, includes sleep", deploy_script.len());
        println!("  Monitor: {} commands, includes loop and sleep", monitor_script.len()); 
        println!("  PTY: {} commands, single command 'zenith'", pty_script.len());
        
        // Key insight: The non-working choices have more complex scripts
        println!("  💡 Hypothesis: Scripts with sleep/loops/external commands may be failing");
        println!("  💡 Hypothesis: PTY execution mode may have different routing issues");
    }
}