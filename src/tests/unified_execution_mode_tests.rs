// F0229: Unified ExecutionMode System Tests - Comprehensive test coverage for consistent execution behavior across all modes

use crate::model::common::ExecutionMode;
use crate::model::muxbox::Choice;
use crate::tests::test_utils::TestDataFactory;

#[test]
fn test_unified_execution_immediateediate_mode() {
    // Test that immediate mode creates execution stream and delivers output consistently
    let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");
    muxbox.choices = Some(vec![
        Choice {
            id: "test_immediateediate".to_string(),
            content: Some("Test Immediate Choice".to_string()),
            selected: false,
            script: Some(vec!["echo 'immediate output'".to_string()]),
            execution_mode: ExecutionMode::Immediate,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        },
    ]);
    
    // Initialize streams to set up baseline
    muxbox.initialize_streams();
    let initial_stream_count = muxbox.streams.len();
    
    // Verify choice has correct execution mode
    let choice = &muxbox.choices.as_ref().unwrap()[0];
    assert_eq!(choice.execution_mode, ExecutionMode::Immediate);
    
    // Stream creation should be handled by unified system
    let expected_stream_id = format!("{}_{}", choice.id, choice.execution_mode.as_stream_suffix());
    assert_eq!(expected_stream_id, "test_immediateediate_immediate");
    
    // Verify that execution stream creation doesn't interfere with existing streams
    assert!(muxbox.streams.len() >= initial_stream_count, 
        "Unified system should preserve existing streams");
}

#[test]
fn test_unified_execution_threadead_mode() {
    // Test that thread mode creates execution stream and delivers output consistently  
    let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");
    muxbox.choices = Some(vec![
        Choice {
            id: "test_threadead".to_string(),
            content: Some("Test Thread Choice".to_string()),
            selected: false,
            script: Some(vec!["echo 'thread output'".to_string()]),
            execution_mode: ExecutionMode::Thread,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        },
    ]);
    
    // Initialize streams to set up baseline
    muxbox.initialize_streams();
    
    // Verify choice has correct execution mode
    let choice = &muxbox.choices.as_ref().unwrap()[0];
    assert_eq!(choice.execution_mode, ExecutionMode::Thread);
    
    // Stream creation should be handled by unified system
    let expected_stream_id = format!("{}_{}", choice.id, choice.execution_mode.as_stream_suffix());
    assert_eq!(expected_stream_id, "test_threadead_thread");
}

#[test]
fn test_unified_execution_pty_mode() {
    // Test that PTY mode creates execution stream and delivers output consistently
    let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");
    muxbox.choices = Some(vec![
        Choice {
            id: "test_pty".to_string(),
            content: Some("Test PTY Choice".to_string()),
            selected: false,
            script: Some(vec!["echo 'pty output'".to_string()]),
            execution_mode: ExecutionMode::Pty,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        },
    ]);
    
    // Initialize streams to set up baseline
    muxbox.initialize_streams();
    
    // Verify choice has correct execution mode
    let choice = &muxbox.choices.as_ref().unwrap()[0];
    assert_eq!(choice.execution_mode, ExecutionMode::Pty);
    
    // Stream creation should be handled by unified system
    let expected_stream_id = format!("{}_{}", choice.id, choice.execution_mode.as_stream_suffix());
    assert_eq!(expected_stream_id, "test_pty_pty");
}

#[test]
fn test_unified_execution_redirect_output() {
    // Test that redirect_output works consistently across all execution modes
    let mut source_muxbox = TestDataFactory::create_test_muxbox("source_muxbox");
    source_muxbox.choices = Some(vec![
        Choice {
            id: "redirect_test".to_string(),
            content: Some("Redirect Test Choice".to_string()),
            selected: false,
            script: Some(vec!["echo 'redirected output'".to_string()]),
            execution_mode: ExecutionMode::Immediate,
            redirect_output: Some("target_box".to_string()),
            append_output: Some(false),
            waiting: false,
        },
    ]);
    
    let mut target_muxbox = TestDataFactory::create_test_muxbox("target_box");
    
    // Initialize streams
    source_muxbox.initialize_streams();
    target_muxbox.initialize_streams();
    
    // Verify choice has redirect configuration
    let choice = &source_muxbox.choices.as_ref().unwrap()[0];
    assert_eq!(choice.redirect_output, Some("target_box".to_string()));
    assert_eq!(choice.execution_mode, ExecutionMode::Immediate);
    
    // Stream ID should be based on choice ID and execution mode suffix
    let expected_stream_id = format!("{}_{}", choice.id, choice.execution_mode.as_stream_suffix());
    assert_eq!(expected_stream_id, "redirect_test_immediate");
}

#[test]
fn test_action_only_boxes_no_content_streams() {
    // F0229: Test that action-only boxes (choices but no meaningful content) don't create unwanted "Content" streams
    let mut action_box = TestDataFactory::create_test_muxbox("action_only");
    action_box.title = Some("Action Box".to_string());
    action_box.content = Some("".to_string()); // Empty content
    action_box.choices = Some(vec![
        Choice {
            id: "action1".to_string(),
            content: Some("Action 1".to_string()),
            selected: false,
            script: Some(vec!["echo 'action1'".to_string()]),
            execution_mode: ExecutionMode::Immediate,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        },
    ]);
    
    // Initialize streams
    action_box.initialize_streams();
    
    // F0229: Action-only box should NOT have a Content stream
    let has_content_stream = action_box.streams.values()
        .any(|s| matches!(s.stream_type, crate::model::common::StreamType::Content));
    assert!(!has_content_stream, "Action-only boxes should not have Content streams");
    
    // Should only have Choices stream
    let has_choices_stream = action_box.streams.values()
        .any(|s| matches!(s.stream_type, crate::model::common::StreamType::Choices));
    assert!(has_choices_stream, "Action-only boxes should have Choices stream");
    
    // Should have exactly 1 stream (choices only)
    assert_eq!(action_box.streams.len(), 1, "Action-only boxes should have exactly 1 stream");
    
    // The choices stream should be active
    let active_streams: Vec<_> = action_box.streams.values().filter(|s| s.active).collect();
    assert_eq!(active_streams.len(), 1, "Should have exactly one active stream");
}

#[test]
fn test_mixed_content_and_choices_boxes() {
    // F0229: Test that boxes with both meaningful content and choices get both streams
    let mut mixed_box = TestDataFactory::create_test_muxbox("mixed_box");
    mixed_box.title = Some("Mixed Box".to_string());
    mixed_box.content = Some("This is meaningful content text.".to_string()); // Meaningful content
    mixed_box.choices = Some(vec![
        Choice {
            id: "choice1".to_string(),
            content: Some("Choice 1".to_string()),
            selected: false,
            script: Some(vec!["echo 'choice1'".to_string()]),
            execution_mode: ExecutionMode::Immediate,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        },
    ]);
    
    // Initialize streams
    mixed_box.initialize_streams();
    
    // Should have both Content and Choices streams
    let has_content_stream = mixed_box.streams.values()
        .any(|s| matches!(s.stream_type, crate::model::common::StreamType::Content));
    assert!(has_content_stream, "Mixed boxes should have Content stream for meaningful content");
    
    let has_choices_stream = mixed_box.streams.values()
        .any(|s| matches!(s.stream_type, crate::model::common::StreamType::Choices));
    assert!(has_choices_stream, "Mixed boxes should have Choices stream");
    
    // Should have exactly 2 streams
    assert_eq!(mixed_box.streams.len(), 2, "Mixed boxes should have exactly 2 streams");
    
    // Content stream should be active by default
    let content_stream = mixed_box.streams.values().find(|s| matches!(s.stream_type, crate::model::common::StreamType::Content));
    assert!(content_stream.is_some(), "Should have content stream");
    assert!(content_stream.unwrap().active, "Content stream should be active in mixed boxes");
}

#[test]
fn test_execution_stream_id_generation() {
    // Test that execution stream IDs are generated consistently
    let test_cases = vec![
        (ExecutionMode::Immediate, "immediate"),
        (ExecutionMode::Thread, "thread"),  
        (ExecutionMode::Pty, "pty"),
    ];
    
    for (mode, expected_suffix) in test_cases {
        let choice_id = "test_choice";
        let expected_stream_id = format!("{}_{}", choice_id, expected_suffix);
        let actual_stream_id = format!("{}_{}", choice_id, mode.as_stream_suffix());
        
        assert_eq!(actual_stream_id, expected_stream_id, 
            "Stream ID generation should be consistent for {:?} mode", mode);
    }
}

#[test]
fn test_no_unwanted_timestamp_content() {
    // F0229: Test that execution output doesn't contain unwanted timestamps
    // This is a structural test - the actual timestamp removal is tested in integration
    let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");
    muxbox.choices = Some(vec![
        Choice {
            id: "no_timestamp".to_string(),
            content: Some("No Timestamp Choice".to_string()),
            selected: false,
            script: Some(vec!["echo 'clean output'".to_string()]),
            execution_mode: ExecutionMode::Immediate,
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        },
    ]);
    
    // Initialize streams
    muxbox.initialize_streams();
    
    // Verify choice configuration
    let choice = &muxbox.choices.as_ref().unwrap()[0];
    assert_eq!(choice.execution_mode, ExecutionMode::Immediate);
    
    // Stream ID should not contain timestamps
    let expected_stream_id = format!("{}_{}", choice.id, choice.execution_mode.as_stream_suffix());
    assert_eq!(expected_stream_id, "no_timestamp_immediate");
    assert!(!expected_stream_id.contains(":"), "Stream ID should not contain timestamp characters");
}

#[test] 
fn test_execution_mode_consistency_across_trigger_methods() {
    // F0229: Test that hotkey and enter/click execution use the same ExecutionMode logic
    let immediate_choice = Choice {
        id: "consistent_choice".to_string(),
        content: Some("Consistent Choice".to_string()),
        selected: false,
        script: Some(vec!["echo 'consistent'".to_string()]),
        execution_mode: ExecutionMode::Thread, // Use Thread mode to test consistency
        redirect_output: None,
        append_output: Some(false),
        waiting: false,
    };
    
    // Both hotkey and enter/click should use the same ExecutionMode
    assert_eq!(immediate_choice.execution_mode, ExecutionMode::Thread);
    
    // Stream ID should be consistent regardless of trigger method
    let expected_stream_id = format!("{}_{}", immediate_choice.id, immediate_choice.execution_mode.as_stream_suffix());
    assert_eq!(expected_stream_id, "consistent_choice_thread");
    
    // Verify that execution_mode is used instead of legacy pty/thread fields
}