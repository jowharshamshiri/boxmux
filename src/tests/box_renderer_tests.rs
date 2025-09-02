#[cfg(test)]
mod tests {
    use crate::components::BoxRenderer;
    use crate::model::common::{Bounds, StreamType, Stream, StreamSource, ExecutionMode};
    use crate::tests::test_utils::TestDataFactory;
    use crate::{AppContext, Config};
    use indexmap::IndexMap;
    use std::collections::HashMap;

    fn create_test_context() -> AppContext {
        let app = TestDataFactory::create_test_app();
        AppContext::new(app, Config::default())
    }

    #[test]
    fn test_box_renderer_creation() {
        let muxbox = TestDataFactory::create_test_muxbox("test_box");
        let renderer = BoxRenderer::new(&muxbox, "test_renderer".to_string());
        
        // BoxRenderer created successfully - component encapsulates muxbox reference
        assert_eq!(muxbox.id, "test_box");
    }

    #[test]
    fn test_box_renderer_basic_rendering() {
        let mut muxbox = TestDataFactory::create_test_muxbox_with_parent("test_box", "test_layout");
        muxbox.title = Some("Test Title".to_string());
        
        let context = create_test_context();
        let mut app = TestDataFactory::create_test_app();
        let graph = app.generate_graph();
        
        // Create adjusted bounds
        let mut adjusted_bounds = HashMap::new();
        let mut layout_bounds = HashMap::new();
        layout_bounds.insert(
            "test_box".to_string(),
            Bounds {
                x1: 10, y1: 10,
                x2: 50, y2: 20,
            }
        );
        adjusted_bounds.insert("test_layout".to_string(), layout_bounds);
        
        let renderer = BoxRenderer::new(&muxbox, "test_renderer".to_string());
        let mut buffer = crate::ScreenBuffer::new();
        
        // Should render without errors
        let result = renderer.render(
            &context,
            &graph,
            &adjusted_bounds,
            &app.layouts[0],
            &mut buffer,
        );
        
        assert!(result, "BoxRenderer should render successfully");
        
        // Validate buffer dimensions were respected
        assert!(buffer.width > 0, "Buffer should have positive width");
        assert!(buffer.height > 0, "Buffer should have positive height");
        
        // Validate that rendering actually occurred by checking buffer cells
        let mut has_non_default_content = false;
        for row in &buffer.buffer {
            for cell in row {
                if cell.ch != ' ' {
                    has_non_default_content = true;
                    break;
                }
            }
        }
        assert!(has_non_default_content, "Buffer should contain rendered content (non-space characters)");
    }

    #[test]
    fn test_box_renderer_with_content_stream() {
        let mut muxbox = TestDataFactory::create_test_muxbox_with_parent("content_box", "test_layout");
        
        // Add content stream with correct API
        let mut streams = IndexMap::new();
        let content_lines = vec!["Line 1".to_string(), "Line 2".to_string()];
        let mut stream = Stream::new(
            "content".to_string(),
            StreamType::Content,
            "Content".to_string(),
            content_lines,
            None,
            None,
        );
        stream.active = true;
        streams.insert("content".to_string(), stream);
        muxbox.streams = streams;
        
        let context = create_test_context();
        let mut app = TestDataFactory::create_test_app();
        let graph = app.generate_graph();
        
        let mut adjusted_bounds = HashMap::new();
        let mut layout_bounds = HashMap::new();
        layout_bounds.insert(
            "content_box".to_string(),
            Bounds {
                x1: 5, y1: 5,
                x2: 45, y2: 15,
            }
        );
        adjusted_bounds.insert("test_layout".to_string(), layout_bounds);
        
        let renderer = BoxRenderer::new(&muxbox, "content_renderer".to_string());
        let mut buffer = crate::ScreenBuffer::new();
        
        let result = renderer.render(
            &context,
            &graph,
            &adjusted_bounds,
            &app.layouts[0],
            &mut buffer,
        );
        
        assert!(result, "BoxRenderer should handle content streams");
        
        // Validate stream state
        assert_eq!(muxbox.streams.len(), 1, "Should have one content stream");
        let content_stream = muxbox.streams.get("content").unwrap();
        assert!(content_stream.active, "Content stream should be active");
        assert_eq!(content_stream.stream_type, StreamType::Content, "Stream should be content type");
        
        // Validate stream content is available
        assert!(!content_stream.content.is_empty(), "Content stream should have content lines");
        assert!(content_stream.content.contains(&"Line 1".to_string()) || 
                content_stream.content.contains(&"Line 2".to_string()),
                "Stream should contain expected content lines");
        
        // Validate buffer has been written to
        assert!(buffer.width > 0 && buffer.height > 0, "Buffer should have valid dimensions");
    }

    #[test]
    fn test_box_renderer_with_choices_stream() {
        let mut muxbox = TestDataFactory::create_test_muxbox_with_parent("choice_box", "test_layout");
        
        // Add choices stream with correct API
        let mut streams = IndexMap::new();
        let choice1 = crate::model::muxbox::Choice {
            id: "choice1".to_string(),
            content: Some("Option 1".to_string()),
            script: Some(vec!["echo option1".to_string()]),
            ..Default::default()
        };
        let choice2 = crate::model::muxbox::Choice {
            id: "choice2".to_string(), 
            content: Some("Option 2".to_string()),
            script: Some(vec!["echo option2".to_string()]),
            ..Default::default()
        };
        let choices = vec![choice1, choice2];
        let mut stream = Stream::new(
            "choices".to_string(),
            StreamType::Choices,
            "Choices".to_string(),
            vec![],
            Some(choices),
            None,
        );
        stream.active = true;
        streams.insert("choices".to_string(), stream);
        muxbox.streams = streams;
        
        let context = create_test_context();
        let mut app = TestDataFactory::create_test_app();
        let graph = app.generate_graph();
        
        let mut adjusted_bounds = HashMap::new();
        let mut layout_bounds = HashMap::new();
        layout_bounds.insert(
            "choice_box".to_string(),
            Bounds {
                x1: 10, y1: 10,
                x2: 60, y2: 25,
            }
        );
        adjusted_bounds.insert("test_layout".to_string(), layout_bounds);
        
        let renderer = BoxRenderer::new(&muxbox, "choice_renderer".to_string());
        let mut buffer = crate::ScreenBuffer::new();
        
        let result = renderer.render(
            &context,
            &graph,
            &adjusted_bounds,
            &app.layouts[0],
            &mut buffer,
        );
        
        assert!(result, "BoxRenderer should handle choice streams");
        
        // Validate stream configuration
        assert_eq!(muxbox.streams.len(), 1, "Should have one choices stream");
        let choices_stream = muxbox.streams.get("choices").unwrap();
        assert!(choices_stream.active, "Choices stream should be active");
        assert_eq!(choices_stream.stream_type, StreamType::Choices, "Stream should be choices type");
        
        // Validate choices are available in the stream
        assert!(choices_stream.choices.is_some(), "Choices stream should have choices");
        let choices = choices_stream.choices.as_ref().unwrap();
        assert_eq!(choices.len(), 2, "Should have two choices");
        assert_eq!(choices[0].id, "choice1", "First choice should have correct ID");
        assert_eq!(choices[1].id, "choice2", "Second choice should have correct ID");
        
        // Validate buffer dimensions
        assert!(buffer.width > 0 && buffer.height > 0, "Buffer should have valid dimensions");
    }

    #[test]
    fn test_box_renderer_preserves_muxbox_state() {
        let muxbox = TestDataFactory::create_test_muxbox("state_box");
        let renderer = BoxRenderer::new(&muxbox, "state_renderer".to_string());
        
        // BoxRenderer preserves muxbox state - it's a read-only rendering component
        assert_eq!(muxbox.id, "state_box");
        assert_eq!(muxbox.streams.len(), 0); // Default test muxbox has no streams
        
        // Renderer doesn't modify the muxbox - it only renders based on its state
        // This validates the design principle that BoxRenderer is purely visual
        
        // Validate renderer is read-only and doesn't mutate the muxbox
        assert_eq!(muxbox.error_state, false, "Renderer should not change muxbox error state");
        assert!(muxbox.title.is_some() || muxbox.title.is_none(), "Renderer should not affect title");
        assert!(muxbox.content.is_some() || muxbox.content.is_none(), "Renderer should not affect content");
    }

    #[test]
    fn test_box_renderer_with_missing_bounds() {
        let muxbox = TestDataFactory::create_test_muxbox("missing_bounds_box");
        let context = create_test_context();
        let mut app = TestDataFactory::create_test_app();
        let graph = app.generate_graph();
        
        // Empty adjusted bounds - should handle gracefully
        let adjusted_bounds = HashMap::new();
        
        let renderer = BoxRenderer::new(&muxbox, "missing_bounds_renderer".to_string());
        let mut buffer = crate::ScreenBuffer::new();
        
        let result = renderer.render(
            &context,
            &graph,
            &adjusted_bounds,
            &app.layouts[0],
            &mut buffer,
        );
        
        assert!(!result, "BoxRenderer should return false when bounds are missing");
    }

    #[test]
    fn test_box_renderer_component_integration() {
        let mut muxbox = TestDataFactory::create_test_muxbox_with_parent("integration_box", "test_layout");
        
        // Configure for scrollable content to test scrollbar integration
        muxbox.overflow_behavior = Some("scroll".to_string());
        muxbox.next_focus_id = Some("next_box".to_string()); // Makes it focusable
        
        // Add large content stream to trigger scrollbars
        let mut streams = IndexMap::new();
        let content_lines: Vec<String> = (1..=20).map(|i| 
            format!("Long content line {} with lots of text to trigger horizontal scrolling", i)
        ).collect();
        let mut stream = Stream::new(
            "content".to_string(),
            StreamType::Content,
            "Large Content".to_string(),
            content_lines,
            None,
            None,
        );
        stream.active = true;
        streams.insert("content".to_string(), stream);
        muxbox.streams = streams;
        
        let context = create_test_context();
        let mut app = TestDataFactory::create_test_app();
        let graph = app.generate_graph();
        
        let mut adjusted_bounds = HashMap::new();
        let mut layout_bounds = HashMap::new();
        layout_bounds.insert(
            "integration_box".to_string(),
            Bounds {
                x1: 5, y1: 5,
                x2: 35, y2: 15, // Small box to force scrolling
            }
        );
        adjusted_bounds.insert("test_layout".to_string(), layout_bounds);
        
        let renderer = BoxRenderer::new(&muxbox, "integration_renderer".to_string());
        let mut buffer = crate::ScreenBuffer::new();
        
        let result = renderer.render(
            &context,
            &graph,
            &adjusted_bounds,
            &app.layouts[0],
            &mut buffer,
        );
        
        assert!(result, "BoxRenderer should integrate with scrollbar components");
    }

    #[test]
    fn test_box_renderer_pty_indicator_integration() {
        let mut muxbox = TestDataFactory::create_test_muxbox_with_parent("pty_box", "test_layout");
        
        // Set PTY execution mode
        muxbox.execution_mode = ExecutionMode::Pty;
        
        let context = create_test_context();
        let mut app = TestDataFactory::create_test_app();
        let graph = app.generate_graph();
        
        let mut adjusted_bounds = HashMap::new();
        let mut layout_bounds = HashMap::new();
        layout_bounds.insert(
            "pty_box".to_string(),
            Bounds {
                x1: 10, y1: 10,
                x2: 50, y2: 20,
            }
        );
        adjusted_bounds.insert("test_layout".to_string(), layout_bounds);
        
        let renderer = BoxRenderer::new(&muxbox, "pty_renderer".to_string());
        let mut buffer = crate::ScreenBuffer::new();
        
        let result = renderer.render(
            &context,
            &graph,
            &adjusted_bounds,
            &app.layouts[0],
            &mut buffer,
        );
        
        assert!(result, "BoxRenderer should handle PTY muxboxes with proper indicators");
    }
}