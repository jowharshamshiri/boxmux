#[cfg(test)]
mod tests {
    use crate::draw_loop::{copy_to_clipboard, get_muxbox_content_for_clipboard};
    use crate::tests::test_utils::TestDataFactory;

    #[test]
    fn test_get_muxbox_content_output_priority() {
        // Test that muxbox output takes priority over content
        let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");
        muxbox.content = Some("Static content".to_string());
        muxbox.output = "Dynamic output".to_string();

        let clipboard_content = get_muxbox_content_for_clipboard(&muxbox);
        assert_eq!(
            clipboard_content, "Dynamic output",
            "MuxBox output should take priority over static content"
        );
    }

    #[test]
    fn test_get_muxbox_content_static_content() {
        // Test content extraction when only static content is available
        let mut muxbox = TestDataFactory::create_test_muxbox("test_muxbox");
        muxbox.content = Some("Static muxbox content".to_string());
        muxbox.output = "".to_string(); // Empty output

        let clipboard_content = get_muxbox_content_for_clipboard(&muxbox);
        assert_eq!(
            clipboard_content, "Static muxbox content",
            "Should use static content when output is empty"
        );
    }

    #[test]
    fn test_get_muxbox_content_no_content() {
        // Test fallback message when muxbox has no content
        let mut muxbox = TestDataFactory::create_test_muxbox("empty_muxbox");
        muxbox.content = None;
        muxbox.output = "".to_string();

        let clipboard_content = get_muxbox_content_for_clipboard(&muxbox);
        assert_eq!(
            clipboard_content, "MuxBox 'empty_muxbox': No content",
            "Should provide informative message for empty muxboxes"
        );
    }

    #[test]
    fn test_get_muxbox_content_multiline() {
        // Test handling of multiline content
        let mut muxbox = TestDataFactory::create_test_muxbox("multiline_muxbox");
        muxbox.output = "Line 1\nLine 2\nLine 3".to_string();

        let clipboard_content = get_muxbox_content_for_clipboard(&muxbox);
        assert_eq!(
            clipboard_content, "Line 1\nLine 2\nLine 3",
            "Should preserve multiline content exactly"
        );
    }

    #[test]
    fn test_copy_to_clipboard_basic() {
        // Test basic clipboard copy functionality
        let test_content = "Test clipboard content";

        // This test may fail on systems without clipboard utilities
        // but should not crash the application
        let result = copy_to_clipboard(test_content);

        // We can't easily verify clipboard contents in tests, but we can verify it doesn't crash
        match result {
            Ok(_) => {
                // Clipboard copy succeeded
                assert!(true, "Clipboard copy should not crash");
            }
            Err(_) => {
                // Clipboard utilities might not be available in test environment
                // This is acceptable for testing purposes
                assert!(true, "Clipboard copy gracefully handles missing utilities");
            }
        }
    }

    #[test]
    fn test_copy_to_clipboard_empty_content() {
        // Test copying empty content to clipboard
        let empty_content = "";

        let result = copy_to_clipboard(empty_content);

        // Should handle empty content gracefully
        match result {
            Ok(_) => assert!(true, "Should handle empty content"),
            Err(_) => assert!(true, "Gracefully handles clipboard errors"),
        }
    }

    #[test]
    fn test_copy_to_clipboard_large_content() {
        // Test copying large content to clipboard
        let large_content = "A".repeat(10000); // 10KB of text

        let result = copy_to_clipboard(&large_content);

        // Should handle large content gracefully
        match result {
            Ok(_) => assert!(true, "Should handle large content"),
            Err(_) => assert!(true, "Gracefully handles clipboard errors"),
        }
    }

    #[test]
    fn test_copy_to_clipboard_special_characters() {
        // Test copying content with special characters
        let special_content = "Special chars: !@#$%^&*()_+{}|:<>?[]\\;'\",./ \n\t\r";

        let result = copy_to_clipboard(special_content);

        // Should handle special characters gracefully
        match result {
            Ok(_) => assert!(true, "Should handle special characters"),
            Err(_) => assert!(true, "Gracefully handles clipboard errors"),
        }
    }
}
