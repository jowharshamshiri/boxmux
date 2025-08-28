#[cfg(test)]
mod tests {
    use crate::model::common::{ChoiceExecutionSource, Stream, StreamSource, StreamType};
    use crate::model::muxbox::MuxBox;
    use crate::tests::test_utils::TestDataFactory;
    use indexmap::IndexMap;
    use std::time::SystemTime;

    #[test]
    fn test_redirected_tab_close_button_functionality() {
        // Create a muxbox with redirected output stream
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        let mut streams = IndexMap::new();

        // Add content stream (not closeable)
        let content_stream = Stream {
            id: "content".to_string(),
            stream_type: StreamType::Content,
            label: "Content".to_string(),
            content: vec!["Original content".to_string()],
            choices: None,
            active: true,
            source: None,
            content_hash: 0,
            last_updated: SystemTime::now(),
            created_at: SystemTime::now(),
        };
        streams.insert("content".to_string(), content_stream);

        // Add redirected output stream (closeable)
        let redirect_stream = Stream {
            id: "redirect_output_1".to_string(),
            stream_type: StreamType::RedirectedOutput("choice_deploy".to_string()),
            label: "→Deploy".to_string(),
            content: vec!["Deployment output line 1".to_string(), "Deployment output line 2".to_string()],
            choices: None,
            active: false,
            source: Some(StreamSource::ChoiceExecution(ChoiceExecutionSource {
                choice_id: "choice_deploy".to_string(),
                thread_id: None,
                process_id: None,
                execution_type: "thread".to_string(),
                started_at: SystemTime::now(),
                muxbox_id: "test_box".to_string(),
                timeout_seconds: None,
            })),
            content_hash: 0,
            last_updated: SystemTime::now(),
            created_at: SystemTime::now(),
        };
        streams.insert("redirect_output_1".to_string(), redirect_stream);

        muxbox.streams = streams;

        // Test 1: Verify muxbox has correct number of streams
        assert_eq!(muxbox.streams.len(), 2, "Muxbox should have 2 streams");

        // Test 2: Verify close buttons are correctly identified
        let close_buttons = muxbox.get_tab_close_buttons();
        assert_eq!(
            close_buttons,
            vec![false, true], // content=false, redirect=true
            "Close buttons should be [false, true]"
        );

        // Test 3: Verify stream IDs are correctly ordered
        let stream_ids = muxbox.get_tab_stream_ids();
        assert_eq!(
            stream_ids,
            vec!["content".to_string(), "redirect_output_1".to_string()],
            "Stream IDs should match insertion order"
        );

        // Test 4: Verify redirected stream is closeable
        let redirect_stream = muxbox.streams.get("redirect_output_1").unwrap();
        assert!(
            redirect_stream.is_closeable(),
            "Redirected output stream should be closeable"
        );

        // Test 5: Verify content stream is not closeable
        let content_stream = muxbox.streams.get("content").unwrap();
        assert!(
            !content_stream.is_closeable(),
            "Content stream should not be closeable"
        );

        // Test 6: Simulate stream removal (what happens when close button is clicked)
        let removed_source = muxbox.remove_stream("redirect_output_1");
        assert!(
            removed_source.is_some(),
            "Should successfully remove redirected stream"
        );

        // Test 7: Verify stream was actually removed
        assert_eq!(muxbox.streams.len(), 1, "Should have 1 stream after removal");
        assert!(
            !muxbox.streams.contains_key("redirect_output_1"),
            "Redirected stream should be removed"
        );
        assert!(
            muxbox.streams.contains_key("content"),
            "Content stream should remain"
        );

        // Test 8: Verify close buttons are updated after removal
        let updated_close_buttons = muxbox.get_tab_close_buttons();
        assert_eq!(
            updated_close_buttons,
            vec![false], // Only content stream remains
            "Only content stream should remain (not closeable)"
        );
    }

    #[test]
    fn test_multiple_redirected_streams_close_functionality() {
        let mut muxbox = TestDataFactory::create_test_muxbox("multi_redirect_box");
        let mut streams = IndexMap::new();

        // Add content stream
        streams.insert("content".to_string(), Stream {
            id: "content".to_string(),
            stream_type: StreamType::Content,
            label: "Content".to_string(),
            content: vec!["Base content".to_string()],
            choices: None,
            active: true,
            source: None,
            content_hash: 0,
            last_updated: SystemTime::now(),
            created_at: SystemTime::now(),
        });

        // Add first redirected stream
        streams.insert("redirect1".to_string(), Stream {
            id: "redirect1".to_string(),
            stream_type: StreamType::RedirectedOutput("deploy".to_string()),
            label: "→Deploy".to_string(),
            content: vec!["Deploy output".to_string()],
            choices: None,
            active: false,
            source: Some(StreamSource::ChoiceExecution(ChoiceExecutionSource {
                choice_id: "deploy".to_string(),
                thread_id: None,
                process_id: None,
                execution_type: "thread".to_string(),
                started_at: SystemTime::now(),
                muxbox_id: "multi_redirect_box".to_string(),
                timeout_seconds: None,
            })),
            content_hash: 0,
            last_updated: SystemTime::now(),
            created_at: SystemTime::now(),
        });

        // Add second redirected stream
        streams.insert("redirect2".to_string(), Stream {
            id: "redirect2".to_string(),
            stream_type: StreamType::RedirectedOutput("monitor".to_string()),
            label: "→Monitor".to_string(),
            content: vec!["Monitor output".to_string()],
            choices: None,
            active: false,
            source: Some(StreamSource::ChoiceExecution(ChoiceExecutionSource {
                choice_id: "monitor".to_string(),
                thread_id: None,
                process_id: None,
                execution_type: "thread".to_string(),
                started_at: SystemTime::now(),
                muxbox_id: "multi_redirect_box".to_string(),
                timeout_seconds: None,
            })),
            content_hash: 0,
            last_updated: SystemTime::now(),
            created_at: SystemTime::now(),
        });

        muxbox.streams = streams;

        // Verify initial state
        assert_eq!(muxbox.streams.len(), 3, "Should have 3 streams initially");
        
        let close_buttons = muxbox.get_tab_close_buttons();
        assert_eq!(
            close_buttons,
            vec![false, true, true], // content=false, both redirects=true
            "Should have close buttons for redirected streams only"
        );

        // Remove first redirected stream
        let removed = muxbox.remove_stream("redirect1");
        assert!(removed.is_some(), "Should remove first redirected stream");
        
        // Verify state after first removal
        assert_eq!(muxbox.streams.len(), 2, "Should have 2 streams after first removal");
        
        let updated_close_buttons = muxbox.get_tab_close_buttons();
        assert_eq!(
            updated_close_buttons,
            vec![false, true], // content=false, redirect2=true
            "Should have correct close buttons after first removal"
        );

        // Remove second redirected stream
        let removed2 = muxbox.remove_stream("redirect2");
        assert!(removed2.is_some(), "Should remove second redirected stream");
        
        // Verify final state
        assert_eq!(muxbox.streams.len(), 1, "Should have 1 stream after all removals");
        
        let final_close_buttons = muxbox.get_tab_close_buttons();
        assert_eq!(
            final_close_buttons,
            vec![false], // Only content stream remains
            "Only content stream should remain (not closeable)"
        );
    }

    #[test]
    fn test_active_stream_switching_on_close() {
        let mut muxbox = TestDataFactory::create_test_muxbox("active_switch_box");
        let mut streams = IndexMap::new();

        // Add content stream (not active)
        streams.insert("content".to_string(), Stream {
            id: "content".to_string(),
            stream_type: StreamType::Content,
            label: "Content".to_string(),
            content: vec!["Base content".to_string()],
            choices: None,
            active: false, // Not active initially
            source: None,
            content_hash: 0,
            last_updated: SystemTime::now(),
            created_at: SystemTime::now(),
        });

        // Add redirected stream (active)
        streams.insert("redirect_active".to_string(), Stream {
            id: "redirect_active".to_string(),
            stream_type: StreamType::RedirectedOutput("deploy".to_string()),
            label: "→Deploy".to_string(),
            content: vec!["Deploy output".to_string()],
            choices: None,
            active: true, // This is the active stream
            source: Some(StreamSource::ChoiceExecution(ChoiceExecutionSource {
                choice_id: "deploy".to_string(),
                thread_id: None,
                process_id: None,
                execution_type: "thread".to_string(),
                started_at: SystemTime::now(),
                muxbox_id: "active_switch_box".to_string(),
                timeout_seconds: None,
            })),
            content_hash: 0,
            last_updated: SystemTime::now(),
            created_at: SystemTime::now(),
        });

        muxbox.streams = streams;

        // Verify initial state - redirected stream is active
        let active_stream = muxbox.get_active_stream();
        assert!(active_stream.is_some(), "Should have an active stream");
        assert_eq!(
            active_stream.unwrap().id,
            "redirect_active",
            "Redirected stream should be active initially"
        );

        // Remove the active redirected stream
        let removed = muxbox.remove_stream("redirect_active");
        assert!(removed.is_some(), "Should remove active redirected stream");

        // Verify that the content stream is now active
        let new_active_stream = muxbox.get_active_stream();
        assert!(new_active_stream.is_some(), "Should have a new active stream");
        assert_eq!(
            new_active_stream.unwrap().id,
            "content",
            "Content stream should become active after removing active redirected stream"
        );

        // Verify content stream's active flag is set to true
        let content_stream = muxbox.streams.get("content").unwrap();
        assert!(
            content_stream.active,
            "Content stream should have active=true after switching"
        );
    }
}