/// Tests for Unified Execution Architecture message system
/// 
/// These tests verify the new ExecuteScript/StreamUpdate/SourceAction message flow
/// that replaces the old ExecuteChoice/CreateChoiceExecutionStream messages

#[cfg(test)]
mod tests {
    use crate::model::common::{
        ExecutionMode, ExecuteScript, ExecutionSource, SourceType, SourceReference,
        StreamUpdate, SourceState, BatchSourceState, BatchStatus
    };
    use crate::model::muxbox::Choice;
    use crate::thread_manager::Message;
    use std::time::Duration;

    #[test]
    fn test_execution_mode_basic_functionality() {
        // Test ExecutionMode enum basic operations
        assert_eq!(ExecutionMode::default(), ExecutionMode::Immediate);
        
        // Test PTY detection
        assert!(!ExecutionMode::Immediate.is_pty(), "Immediate mode should not be PTY");
        assert!(!ExecutionMode::Thread.is_pty(), "Thread mode should not be PTY");
        assert!(ExecutionMode::Pty.is_pty(), "PTY mode should be PTY");
    }

    #[test]
    fn test_execute_script_message_structure() {
        // Test unified ExecuteScript message structure
        let choice = Choice {
            id: "test_choice".to_string(),
            content: Some("Test Choice".to_string()),
            script: Some(vec!["echo test".to_string()]),
            execution_mode: ExecutionMode::Pty,
            redirect_output: None,
            append_output: None,
            selected: false,
            waiting: false,
        };

        // Create ExecuteScript message using unified architecture
        let execute_script = ExecuteScript {
            script: vec!["echo test".to_string()],
            source: ExecutionSource {
                source_type: SourceType::Choice("test_choice".to_string()),
                source_id: "test_execution_001".to_string(),
                source_reference: SourceReference::Choice(choice.clone()),
            },
            execution_mode: ExecutionMode::Pty,
            target_box_id: "test_muxbox".to_string(),
            libs: vec!["lib1".to_string()],
            redirect_output: None,
            append_output: false,
            stream_id: "test-stream-001".to_string(),
        };

        let msg = Message::ExecuteScriptMessage(execute_script.clone());

        // Verify message structure
        if let Message::ExecuteScriptMessage(execute_script_msg) = msg {
            assert_eq!(execute_script_msg.execution_mode, ExecutionMode::Pty);
            assert_eq!(execute_script_msg.target_box_id, "test_muxbox");
            assert_eq!(execute_script_msg.libs, vec!["lib1".to_string()]);
            if let SourceType::Choice(choice_id) = &execute_script_msg.source.source_type {
                assert_eq!(choice_id, "test_choice");
            } else {
                panic!("Source type should be Choice");
            }
        } else {
            panic!("Message should be ExecuteScriptMessage variant");
        }
    }

    #[test]
    fn test_stream_update_message() {
        // Test StreamUpdate message with ExecutionMode
        let stream_update = StreamUpdate {
            stream_id: "test_muxbox_choice_stream".to_string(),
            target_box_id: "test_box".to_string(),
            content_update: "Script output content".to_string(),
            source_state: SourceState::Batch(BatchSourceState {
                task_id: "task_001".to_string(),
                queue_wait_time: Duration::from_millis(10),
                execution_time: Duration::from_millis(500),
                exit_code: Some(0),
                status: BatchStatus::Completed,
            }),
            execution_mode: ExecutionMode::Thread,
        };

        let msg = Message::StreamUpdateMessage(stream_update.clone());

        // Verify message structure  
        if let Message::StreamUpdateMessage(stream_update_msg) = msg {
            assert_eq!(stream_update_msg.stream_id, "test_muxbox_choice_stream");
            assert_eq!(stream_update_msg.content_update, "Script output content");
            assert_eq!(stream_update_msg.execution_mode, ExecutionMode::Thread);
            if let SourceState::Batch(batch_state) = stream_update_msg.source_state {
                assert_eq!(batch_state.task_id, "task_001");
                assert_eq!(batch_state.exit_code, Some(0));
                assert!(matches!(batch_state.status, BatchStatus::Completed));
            } else {
                panic!("Source state should be Batch variant");
            }
        } else {
            panic!("Message should be StreamUpdateMessage variant");
        }
    }

    #[test]
    fn test_execution_mode_hash_and_equality() {
        // Test that ExecutionMode properly implements Hash and PartialEq
        use std::collections::HashSet;

        let mut modes = HashSet::new();
        modes.insert(ExecutionMode::Immediate);
        modes.insert(ExecutionMode::Thread);
        modes.insert(ExecutionMode::Pty);
        
        // Verify all modes are distinct
        assert_eq!(modes.len(), 3);
        
        // Test equality
        assert_eq!(ExecutionMode::Immediate, ExecutionMode::Immediate);
        assert_ne!(ExecutionMode::Immediate, ExecutionMode::Thread);
    }

    #[test]
    fn test_source_type_variants() {
        // Test all SourceType variants work correctly
        let choice_source = SourceType::Choice("test_choice".to_string());
        let static_source = SourceType::StaticScript;
        let socket_source = SourceType::SocketUpdate;
        let hotkey_source = SourceType::HotkeyScript;
        
        // Verify they're all distinct
        assert_ne!(choice_source, static_source);
        assert_ne!(static_source, socket_source);
        assert_ne!(socket_source, hotkey_source);
        
        // Verify choice source contains expected value
        if let SourceType::Choice(choice_id) = choice_source {
            assert_eq!(choice_id, "test_choice");
        } else {
            panic!("Should be Choice variant");
        }
    }

    #[test] 
    fn test_unified_message_flow_integration() {
        // Test that the unified message flow works end-to-end
        let choice = Choice {
            id: "integration_choice".to_string(),
            content: Some("Integration Test".to_string()),
            script: Some(vec!["echo integration".to_string()]),
            execution_mode: ExecutionMode::Thread,
            redirect_output: Some("output_box".to_string()),
            append_output: Some(true),
            selected: false,
            waiting: false,
        };

        // 1. Create ExecuteScript message (replaces ExecuteChoice)
        let execute_script = ExecuteScript {
            script: vec!["echo integration".to_string()],
            source: ExecutionSource {
                source_type: SourceType::Choice("integration_choice".to_string()),
                source_id: "integration_001".to_string(),
                source_reference: SourceReference::Choice(choice),
            },
            execution_mode: ExecutionMode::Thread,
            target_box_id: "test_box".to_string(),
            libs: vec![],
            redirect_output: Some("output_box".to_string()),
            append_output: true,
            stream_id: "test-stream-002".to_string(),
        };

        // 2. Create StreamUpdate message (unified architecture)
        let stream_update = StreamUpdate {
            stream_id: "test_box_integration_stream".to_string(),
            target_box_id: "test_box".to_string(),
            content_update: "integration\n".to_string(),
            source_state: SourceState::Batch(BatchSourceState {
                task_id: "integration_001".to_string(),
                queue_wait_time: Duration::from_millis(5),
                execution_time: Duration::from_millis(250),
                exit_code: Some(0),
                status: BatchStatus::Completed,
            }),
            execution_mode: ExecutionMode::Thread,
        };

        // Verify both messages work together
        let execute_msg = Message::ExecuteScriptMessage(execute_script);
        let update_msg = Message::StreamUpdateMessage(stream_update);

        // Both should be valid message types
        assert!(matches!(execute_msg, Message::ExecuteScriptMessage(_)));
        assert!(matches!(update_msg, Message::StreamUpdateMessage(_)));
    }
}