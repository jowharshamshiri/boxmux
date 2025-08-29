// F0228: ExecutionMode Message System Tests - Comprehensive test coverage for ExecutionMode-based message handling

use crate::model::common::ExecutionMode;
use crate::model::muxbox::Choice;
use crate::tests::test_utils::TestDataFactory;
use crate::thread_manager::Message;
use crate::utils::{run_script_with_pty, run_script_with_pty_and_redirect};

#[test]
fn test_execution_mode_in_utils_run_script_with_pty() {
    // Test that run_script_with_pty correctly handles ExecutionMode instead of boolean
    let libs = Some(vec!["test".to_string()]);
    let script = vec!["echo test".to_string()];
    
    // Test Immediate mode
    let result = run_script_with_pty(
        libs.clone(),
        &script,
        &ExecutionMode::Immediate,
        None,
        None,
        None,
    );
    assert!(result.is_ok(), "Immediate mode should execute successfully");

    // Test Thread mode (without PTY manager, should fallback to regular execution)
    let result = run_script_with_pty(
        libs.clone(),
        &script,
        &ExecutionMode::Thread,
        None,
        None,
        None,
    );
    assert!(result.is_ok(), "Thread mode should execute successfully");

    // Test PTY mode (without PTY manager, should fallback to regular execution)
    let result = run_script_with_pty(
        libs,
        &script,
        &ExecutionMode::Pty,
        None,
        None,
        None,
    );
    assert!(result.is_ok(), "PTY mode should fallback to regular execution without PTY manager");
}

#[test]
fn test_execution_mode_in_utils_run_script_with_pty_and_redirect() {
    // Test that run_script_with_pty_and_redirect correctly handles ExecutionMode
    let libs = Some(vec!["test".to_string()]);
    let script = vec!["echo test".to_string()];
    
    // Test Immediate mode with redirect
    let result = run_script_with_pty_and_redirect(
        libs.clone(),
        &script,
        &ExecutionMode::Immediate,
        None,
        None,
        None,
        Some("test_redirect".to_string()),
    );
    assert!(result.is_ok(), "Immediate mode with redirect should execute successfully");

    // Test Thread mode with redirect
    let result = run_script_with_pty_and_redirect(
        libs.clone(),
        &script,
        &ExecutionMode::Thread,
        None,
        None,
        None,
        Some("test_redirect".to_string()),
    );
    assert!(result.is_ok(), "Thread mode with redirect should execute successfully");

    // Test PTY mode with redirect (should fallback without PTY manager)
    let result = run_script_with_pty_and_redirect(
        libs,
        &script,
        &ExecutionMode::Pty,
        None,
        None,
        None,
        Some("test_redirect".to_string()),
    );
    assert!(result.is_ok(), "PTY mode with redirect should fallback successfully");
}

#[test]
fn test_execution_mode_is_pty_method() {
    // Test ExecutionMode.is_pty() method used in utils functions
    assert!(!ExecutionMode::Immediate.is_pty(), "Immediate mode should not be PTY");
    assert!(!ExecutionMode::Thread.is_pty(), "Thread mode should not be PTY");
    assert!(ExecutionMode::Pty.is_pty(), "PTY mode should be PTY");
}

#[test]
fn test_execute_choice_message_structure() {
    // Test that ExecuteChoice message structure works with ExecutionMode
    let choice = Choice {
        id: "test_choice".to_string(),
        content: Some("Test Choice".to_string()),
        script: Some(vec!["echo test".to_string()]),
        thread: None,
        pty: None,
        execution_mode: ExecutionMode::Pty, // F0228: ExecutionMode field
        redirect_output: None,
        append_output: None,
        selected: false,
        waiting: false,
    };

    let msg = Message::ExecuteChoice(
        choice.clone(),
        "test_muxbox".to_string(),
        Some(vec!["lib1".to_string()]),
    );

    // Verify message structure
    if let Message::ExecuteChoice(choice_in_msg, muxbox_id, libs) = msg {
        assert_eq!(choice_in_msg.id, "test_choice");
        assert_eq!(choice_in_msg.execution_mode, ExecutionMode::Pty);
        assert_eq!(muxbox_id, "test_muxbox");
        assert_eq!(libs, Some(vec!["lib1".to_string()]));
    } else {
        panic!("Message should be ExecuteChoice variant");
    }
}

#[test]
fn test_create_choice_execution_stream_message() {
    // Test CreateChoiceExecutionStream message with ExecutionMode
    let msg = Message::CreateChoiceExecutionStream(
        "test_choice".to_string(),
        "test_muxbox".to_string(),
        ExecutionMode::Thread,
        "Test Stream Label".to_string(),
    );

    // Verify message structure
    if let Message::CreateChoiceExecutionStream(choice_id, muxbox_id, execution_mode, label) = msg {
        assert_eq!(choice_id, "test_choice");
        assert_eq!(muxbox_id, "test_muxbox");
        assert_eq!(execution_mode, ExecutionMode::Thread);
        assert_eq!(label, "Test Stream Label");
    } else {
        panic!("Message should be CreateChoiceExecutionStream variant");
    }
}

#[test]
fn test_execution_mode_message_hash_consistency() {
    // Test that messages with ExecutionMode hash consistently
    let choice1 = Choice {
        id: "test_choice".to_string(),
        content: Some("Test Choice".to_string()),
        script: Some(vec!["echo test".to_string()]),
        thread: None,
        pty: None,
        execution_mode: ExecutionMode::Immediate,
        redirect_output: None,
        append_output: None,
        selected: false,
        waiting: false,
    };

    let choice2 = Choice {
        id: "test_choice".to_string(),
        content: Some("Test Choice".to_string()),
        script: Some(vec!["echo test".to_string()]),
        thread: None,
        pty: None,
        execution_mode: ExecutionMode::Immediate, // Same ExecutionMode
        redirect_output: None,
        append_output: None,
        selected: false,
        waiting: false,
    };

    let msg1 = Message::ExecuteChoice(choice1, "test_muxbox".to_string(), None);
    let msg2 = Message::ExecuteChoice(choice2, "test_muxbox".to_string(), None);

    // Messages with same ExecutionMode should hash equally
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();
    
    msg1.hash(&mut hasher1);
    msg2.hash(&mut hasher2);
    
    assert_eq!(hasher1.finish(), hasher2.finish(), "Messages with same ExecutionMode should hash equally");
}

#[test]
fn test_execution_mode_message_different_modes() {
    // Test that messages with different ExecutionModes hash differently
    let choice_immediate = Choice {
        id: "test_choice".to_string(),
        content: Some("Test Choice".to_string()),
        script: Some(vec!["echo test".to_string()]),
        thread: None,
        pty: None,
        execution_mode: ExecutionMode::Immediate,
        redirect_output: None,
        append_output: None,
        selected: false,
        waiting: false,
    };

    let choice_pty = Choice {
        id: "test_choice".to_string(),
        content: Some("Test Choice".to_string()),
        script: Some(vec!["echo test".to_string()]),
        thread: None,
        pty: None,
        execution_mode: ExecutionMode::Pty, // Different ExecutionMode
        redirect_output: None,
        append_output: None,
        selected: false,
        waiting: false,
    };

    let msg1 = Message::ExecuteChoice(choice_immediate, "test_muxbox".to_string(), None);
    let msg2 = Message::ExecuteChoice(choice_pty, "test_muxbox".to_string(), None);

    // Messages with different ExecutionMode should not be equal
    assert_ne!(msg1, msg2, "Messages with different ExecutionModes should not be equal");
}

#[test]
fn test_execution_mode_stream_creation_message() {
    // Test all ExecutionMode variants in stream creation messages
    let modes = vec![
        ExecutionMode::Immediate,
        ExecutionMode::Thread,
        ExecutionMode::Pty,
    ];

    for mode in modes {
        let msg = Message::CreateChoiceExecutionStream(
            "test_choice".to_string(),
            "test_muxbox".to_string(),
            mode.clone(),
            format!("Stream for {:?}", mode),
        );

        // Verify each message can be created and structured correctly
        if let Message::CreateChoiceExecutionStream(choice_id, muxbox_id, execution_mode, label) = msg {
            assert_eq!(choice_id, "test_choice");
            assert_eq!(muxbox_id, "test_muxbox");
            assert_eq!(execution_mode, mode);
            assert!(label.contains(&format!("{:?}", mode)));
        } else {
            panic!("Message should be CreateChoiceExecutionStream variant");
        }
    }
}

#[test]
fn test_no_hardcoded_boolean_in_message_system() {
    // Test that ExecutionMode methods work correctly for message system decisions
    
    // PTY detection should work through ExecutionMode
    let pty_mode = ExecutionMode::Pty;
    assert!(pty_mode.is_pty(), "PTY mode should be detected correctly");
    assert!(pty_mode.is_background(), "PTY mode should be background");
    assert!(pty_mode.is_realtime(), "PTY mode should be realtime");

    // Thread detection
    let thread_mode = ExecutionMode::Thread;
    assert!(!thread_mode.is_pty(), "Thread mode should not be PTY");
    assert!(thread_mode.is_background(), "Thread mode should be background");
    assert!(!thread_mode.is_realtime(), "Thread mode should not be realtime");

    // Immediate detection
    let immediate_mode = ExecutionMode::Immediate;
    assert!(!immediate_mode.is_pty(), "Immediate mode should not be PTY");
    assert!(!immediate_mode.is_background(), "Immediate mode should not be background");
    assert!(!immediate_mode.is_realtime(), "Immediate mode should not be realtime");
}

#[test]
fn test_execution_mode_legacy_migration_in_messages() {
    // Test that legacy boolean conversion still works for backward compatibility
    
    // PTY precedence: pty=true, thread=false -> Pty
    let mode1 = ExecutionMode::from_legacy(false, true);
    assert_eq!(mode1, ExecutionMode::Pty, "PTY should have precedence over thread");

    // Thread mode: pty=false, thread=true -> Thread
    let mode2 = ExecutionMode::from_legacy(true, false);
    assert_eq!(mode2, ExecutionMode::Thread, "Thread mode should be detected");

    // Immediate mode: pty=false, thread=false -> Immediate
    let mode3 = ExecutionMode::from_legacy(false, false);
    assert_eq!(mode3, ExecutionMode::Immediate, "Default should be Immediate");

    // Both true: PTY precedence
    let mode4 = ExecutionMode::from_legacy(true, true);
    assert_eq!(mode4, ExecutionMode::Pty, "PTY should have precedence when both are true");
}

#[test]
fn test_muxbox_execution_mode_message_integration() {
    // Test that MuxBox ExecutionMode integrates properly with message system
    let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
    muxbox.execution_mode = ExecutionMode::Pty;

    // Test that ExecutionMode can be read from MuxBox for message decisions
    assert!(muxbox.execution_mode.is_pty(), "MuxBox should have PTY execution mode");
    assert!(muxbox.execution_mode.creates_streams(), "All execution modes should create streams");

    // Simulate message system decision
    let should_use_pty = muxbox.execution_mode.is_pty();
    assert!(should_use_pty, "Message system should detect PTY mode correctly");
}

#[test]
fn test_choice_execution_mode_message_integration() {
    // Test that Choice ExecutionMode integrates properly with message system
    let choice = Choice {
        id: "test_choice".to_string(),
        content: Some("Test Choice".to_string()),
        script: Some(vec!["echo test".to_string()]),
        thread: None,
        pty: None,
        execution_mode: ExecutionMode::Thread, // F0228: Use ExecutionMode
        redirect_output: Some("redirect_target".to_string()),
        append_output: Some(true),
        selected: false,
        waiting: false,
    };

    // Test message creation with choice ExecutionMode
    let msg = Message::ExecuteChoice(
        choice.clone(),
        "test_muxbox".to_string(),
        Some(vec!["lib1".to_string(), "lib2".to_string()]),
    );

    // Verify message preserves ExecutionMode
    if let Message::ExecuteChoice(choice_in_msg, _, _) = msg {
        assert_eq!(choice_in_msg.execution_mode, ExecutionMode::Thread);
        assert!(choice_in_msg.execution_mode.is_background(), "Thread mode should be background");
        assert!(!choice_in_msg.execution_mode.is_pty(), "Thread mode should not be PTY");
    } else {
        panic!("Message should be ExecuteChoice variant");
    }
}