#[cfg(test)]
mod conditional_stream_creation_tests {
    use crate::model::common::StreamType;
    use crate::model::muxbox::Choice;
    use crate::tests::test_utils::TestDataFactory;

    #[test]
    fn test_no_content_no_choices_creates_no_streams() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = None; // no title
        muxbox.content = None; // no content field set
        muxbox.choices = None; // no choices field set

        muxbox.initialize_streams();

        assert!(
            muxbox.streams.is_empty(),
            "Box with no content and no choices should have no streams"
        );
        assert!(
            muxbox.get_selected_stream().is_none(),
            "Box with no streams should have no active stream"
        );
        assert!(
            muxbox.get_tab_labels().is_empty(),
            "Box with no streams should have no tabs"
        );
    }

    #[test]
    fn test_empty_content_no_choices_creates_no_streams() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = None;
        muxbox.content = Some("   \n\t  \n   ".to_string()); // whitespace-only content
        muxbox.choices = None; // no choices field set

        muxbox.initialize_streams();

        assert!(
            muxbox.streams.is_empty(),
            "Box with empty/whitespace content should have no streams"
        );
        assert!(
            muxbox.get_selected_stream().is_none(),
            "Box with no streams should have no active stream"
        );
        assert!(
            muxbox.get_tab_labels().is_empty(),
            "Box with no streams should have no tabs"
        );
    }

    #[test]
    fn test_content_only_creates_content_stream_with_default_title() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = None; // no title
        muxbox.content = Some("Test content".to_string());
        muxbox.choices = None;

        muxbox.initialize_streams();

        assert_eq!(
            muxbox.streams.len(),
            1,
            "Box with only content should have exactly one stream"
        );

        let stream = muxbox.get_selected_stream().unwrap();
        assert_eq!(stream.stream_type, StreamType::Content);
        assert_eq!(
            stream.label, "Content",
            "Content-only stream should get 'Content' label"
        );
        assert!(stream.id.len() > 0); // Stream exists and has ID

        let tab_labels = muxbox.get_tab_labels();
        assert_eq!(tab_labels.len(), 1);
        assert_eq!(tab_labels[0], "Content");
    }

    #[test]
    fn test_content_only_with_title_creates_content_stream_with_content_title() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = Some("My Box".to_string()); // has title
        muxbox.content = Some("Test content".to_string());
        muxbox.choices = None;

        muxbox.initialize_streams();

        assert_eq!(
            muxbox.streams.len(),
            1,
            "Box with only content should have exactly one stream"
        );

        let stream = muxbox.get_selected_stream().unwrap();
        assert_eq!(stream.stream_type, StreamType::Content);
        assert_eq!(
            stream.label, "Content",
            "Content-only stream should get 'Content' label when no choices exist"
        );

        let tab_labels = muxbox.get_tab_labels();
        assert_eq!(tab_labels.len(), 1);
        assert_eq!(tab_labels[0], "Content");
    }

    #[test]
    fn test_choices_only_creates_choices_stream_with_box_id() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = None; // no title
        muxbox.content = None;
        muxbox.choices = Some(vec![Choice {
            id: "choice1".to_string(),
            content: Some("Test Choice".to_string()),
            script: None,
            redirect_output: None,
            append_output: None,
            execution_mode: crate::model::common::ExecutionMode::default(),
            selected: false,
			hovered: false,
            waiting: false,
        }]);

        muxbox.initialize_streams();

        assert_eq!(
            muxbox.streams.len(),
            1,
            "Box with only choices should have exactly one stream"
        );

        let stream = muxbox.get_selected_stream().unwrap();
        assert_eq!(stream.stream_type, StreamType::Choices);
        assert_eq!(
            stream.label, "test_box",
            "Choices-only stream should use box ID as label"
        );
        assert!(stream.id.len() > 0); // Stream exists and has ID

        let tab_labels = muxbox.get_tab_labels();
        assert_eq!(tab_labels.len(), 1);
        assert_eq!(tab_labels[0], "test_box");
    }

    #[test]
    fn test_choices_only_with_title_creates_choices_stream_with_box_title() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = Some("My Box".to_string()); // has title
        muxbox.content = None;
        muxbox.choices = Some(vec![Choice {
            id: "choice1".to_string(),
            content: Some("Test Choice".to_string()),
            script: None,
            redirect_output: None,
            append_output: None,
            execution_mode: crate::model::common::ExecutionMode::default(),
            selected: false,
			hovered: false,
            waiting: false,
        }]);

        muxbox.initialize_streams();

        assert_eq!(
            muxbox.streams.len(),
            1,
            "Box with only choices should have exactly one stream"
        );

        let stream = muxbox.get_selected_stream().unwrap();
        assert_eq!(stream.stream_type, StreamType::Choices);
        assert_eq!(
            stream.label, "My Box",
            "Choices-only stream should use box title as label"
        );

        let tab_labels = muxbox.get_tab_labels();
        assert_eq!(tab_labels.len(), 1);
        assert_eq!(tab_labels[0], "My Box");
    }

    #[test]
    fn test_both_content_and_choices_creates_both_streams_with_proper_titles() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = Some("My Box".to_string());
        muxbox.content = Some("Test content".to_string());
        muxbox.choices = Some(vec![Choice {
            id: "choice1".to_string(),
            content: Some("Test Choice".to_string()),
            script: None,
            redirect_output: None,
            append_output: None,
            execution_mode: crate::model::common::ExecutionMode::default(),
            selected: false,
			hovered: false,
            waiting: false,
        }]);

        muxbox.initialize_streams();

        assert_eq!(
            muxbox.streams.len(),
            2,
            "Box with both content and choices should have exactly two streams"
        );

        let tab_labels = muxbox.get_tab_labels();
        assert_eq!(tab_labels.len(), 2);

        // Find content and choices streams
        let content_stream = muxbox
            .streams
            .values()
            .find(|s| s.stream_type == StreamType::Content)
            .expect("Should have content stream");
        let choices_stream = muxbox
            .streams
            .values()
            .find(|s| s.stream_type == StreamType::Choices)
            .expect("Should have choices stream");

        assert_eq!(
            content_stream.label, "My Box",
            "Content stream should use box title when both streams exist"
        );
        assert_eq!(
            choices_stream.label, "Choices",
            "Choices stream should use 'Choices' label when both streams exist"
        );
        assert!(
            content_stream.stream_type == crate::model::common::StreamType::Content,
            "Content stream should be active by default"
        );
        assert!(
            choices_stream.choices.is_some(),
            "Choices stream should not be active when content exists"
        );

        // Verify tab labels match stream labels
        assert!(tab_labels.contains(&"My Box".to_string()));
        assert!(tab_labels.contains(&"Choices".to_string()));
    }

    #[test]
    fn test_both_content_and_choices_no_title_uses_box_id() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = None; // no title
        muxbox.content = Some("Test content".to_string());
        muxbox.choices = Some(vec![Choice {
            id: "choice1".to_string(),
            content: Some("Test Choice".to_string()),
            script: None,
            redirect_output: None,
            append_output: None,
            execution_mode: crate::model::common::ExecutionMode::default(),
            selected: false,
			hovered: false,
            waiting: false,
        }]);

        muxbox.initialize_streams();

        let content_stream = muxbox
            .streams
            .values()
            .find(|s| s.stream_type == StreamType::Content)
            .expect("Should have content stream");
        let choices_stream = muxbox
            .streams
            .values()
            .find(|s| s.stream_type == StreamType::Choices)
            .expect("Should have choices stream");

        assert_eq!(
            content_stream.label, "test_box",
            "Content stream should use box ID when no title and both streams exist"
        );
        assert_eq!(
            choices_stream.label, "Choices",
            "Choices stream should use 'Choices' label when both streams exist"
        );

        let tab_labels = muxbox.get_tab_labels();
        assert!(tab_labels.contains(&"test_box".to_string()));
        assert!(tab_labels.contains(&"Choices".to_string()));
    }

    #[test]
    fn test_empty_choices_array_creates_no_choices_stream() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = None;
        muxbox.content = Some("Test content".to_string());
        muxbox.choices = Some(vec![]); // empty choices array

        muxbox.initialize_streams();

        assert_eq!(
            muxbox.streams.len(),
            1,
            "Box with empty choices array should only have content stream"
        );

        let stream = muxbox.get_selected_stream().unwrap();
        assert_eq!(stream.stream_type, StreamType::Content);
        assert_eq!(stream.label, "Content");

        let tab_labels = muxbox.get_tab_labels();
        assert_eq!(tab_labels.len(), 1);
        assert_eq!(tab_labels[0], "Content");
    }

    #[test]
    fn test_active_stream_content_empty_when_no_streams() {
        let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
        muxbox.title = None;
        muxbox.content = None; // No content
        muxbox.choices = None; // No choices

        muxbox.initialize_streams();

        let content = muxbox
            .get_selected_stream()
            .map_or(Vec::new(), |s| s.content.clone());
        assert!(
            content.is_empty(),
            "Box with no streams should return empty content"
        );

        let choices = muxbox.get_selected_stream_choices();
        assert!(
            choices.is_none(),
            "Box with no streams should return None for choices"
        );
    }
}
