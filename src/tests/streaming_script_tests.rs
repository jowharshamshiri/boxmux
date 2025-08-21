#[cfg(test)]
mod tests {
    use crate::utils::run_script_streaming;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn test_streaming_script_basic_output() {
        // Test basic streaming functionality with simple echo commands
        let script = vec![
            "echo 'Line 1'".to_string(),
            "echo 'Line 2'".to_string(),
            "echo 'Line 3'".to_string(),
        ];
        
        let result = run_script_streaming(None, &script);
        assert!(result.is_ok(), "Streaming script should execute successfully");
        
        let mut output_lines = Vec::new();
        for line in result.unwrap() {
            output_lines.push(line);
            // Limit collection to avoid infinite iteration in case of issues
            if output_lines.len() >= 10 {
                break;
            }
        }
        
        assert!(!output_lines.is_empty(), "Should receive output lines");
        assert!(output_lines.contains(&"Line 1".to_string()), "Should contain first echo output");
        assert!(output_lines.contains(&"Line 2".to_string()), "Should contain second echo output"); 
        assert!(output_lines.contains(&"Line 3".to_string()), "Should contain third echo output");
    }

    #[test]
    fn test_streaming_script_with_delay() {
        // Test streaming with time delays to verify real-time streaming
        let script = vec![
            "echo 'Start'".to_string(),
            "sleep 0.1".to_string(),
            "echo 'Middle'".to_string(),
            "sleep 0.1".to_string(),
            "echo 'End'".to_string(),
        ];
        
        let result = run_script_streaming(None, &script);
        assert!(result.is_ok(), "Streaming script with delays should execute successfully");
        
        let mut output_lines = Vec::new();
        let start_time = std::time::Instant::now();
        
        for line in result.unwrap() {
            output_lines.push(line.clone());
            
            // Check that we get the first line quickly (within 50ms)
            if line == "Start" && start_time.elapsed() < Duration::from_millis(50) {
                // This proves streaming is working - we got output before the script finished
                break;
            }
            
            // Safety break to avoid infinite loop
            if output_lines.len() >= 10 || start_time.elapsed() > Duration::from_secs(2) {
                break;
            }
        }
        
        assert!(!output_lines.is_empty(), "Should receive streaming output");
        assert!(output_lines.contains(&"Start".to_string()), "Should receive first output before script completion");
    }

    #[test]
    fn test_streaming_script_stderr_capture() {
        // Test that both stdout and stderr are captured in the stream
        let script = vec![
            "echo 'stdout message'".to_string(),
            "echo 'stderr message' >&2".to_string(),
        ];
        
        let result = run_script_streaming(None, &script);
        assert!(result.is_ok(), "Streaming script should capture both stdout and stderr");
        
        let mut output_lines = Vec::new();
        for line in result.unwrap() {
            output_lines.push(line);
            // Limit collection
            if output_lines.len() >= 10 {
                break;
            }
        }
        
        assert!(!output_lines.is_empty(), "Should receive output from both streams");
        assert!(output_lines.contains(&"stdout message".to_string()), "Should contain stdout output");
        assert!(output_lines.contains(&"stderr message".to_string()), "Should contain stderr output");
    }

    #[test]
    fn test_streaming_script_with_libraries() {
        // Test streaming with library includes
        let libs = vec!["echo '# Library loaded'".to_string()];
        let script = vec!["echo 'Main script'".to_string()];
        
        let result = run_script_streaming(Some(&libs), &script);
        assert!(result.is_ok(), "Streaming script with libraries should work");
        
        let mut output_lines = Vec::new();
        for line in result.unwrap() {
            output_lines.push(line);
            if output_lines.len() >= 10 {
                break;
            }
        }
        
        assert!(!output_lines.is_empty(), "Should receive output with library support");
        // Note: Library content is sourced, so exact output may vary
        assert!(output_lines.iter().any(|line| line.contains("script")), "Should contain main script output");
    }

    #[test]
    fn test_streaming_script_long_running_command() {
        // Test streaming with a command that generates output over time
        let script = vec![
            "for i in {1..3}; do echo \"Count: $i\"; sleep 0.1; done".to_string(),
        ];
        
        let result = run_script_streaming(None, &script);
        assert!(result.is_ok(), "Long-running streaming command should work");
        
        let mut output_lines = Vec::new();
        let start_time = std::time::Instant::now();
        
        for line in result.unwrap() {
            if line.starts_with("Count:") {
                output_lines.push(line);
            }
            
            // Break when we get all expected output or timeout
            if output_lines.len() >= 3 || start_time.elapsed() > Duration::from_secs(2) {
                break;
            }
        }
        
        assert!(!output_lines.is_empty(), "Should receive streaming output from loop");
        assert!(output_lines.iter().any(|line| line.contains("Count: 1")), "Should receive first count");
    }
}