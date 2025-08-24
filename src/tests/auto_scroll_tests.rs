#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::muxbox::MuxBox;

    #[test]
    fn test_auto_scroll_bottom_field_creation() {
        let mut muxbox = MuxBox::default();
        muxbox.auto_scroll_bottom = Some(true);

        assert_eq!(muxbox.auto_scroll_bottom, Some(true));
    }

    #[test]
    fn test_auto_scroll_bottom_content_update() {
        let mut muxbox = MuxBox::default();
        muxbox.auto_scroll_bottom = Some(true);
        muxbox.vertical_scroll = Some(25.0); // Start at 25%

        // Simulate content update - should auto-scroll to bottom
        muxbox.update_content("New content", false, true);

        // Verify auto-scroll set vertical scroll to 100%
        assert_eq!(muxbox.vertical_scroll, Some(100.0));
    }

    #[test]
    fn test_auto_scroll_bottom_disabled_preserves_scroll() {
        let mut muxbox = MuxBox::default();
        muxbox.auto_scroll_bottom = Some(false);
        muxbox.vertical_scroll = Some(50.0); // Start at 50%

        // Simulate content update - should preserve scroll position
        muxbox.update_content("New content", false, true);

        // Verify scroll position preserved
        assert_eq!(muxbox.vertical_scroll, Some(50.0));
    }

    #[test]
    fn test_auto_scroll_bottom_none_preserves_scroll() {
        let mut muxbox = MuxBox::default();
        muxbox.auto_scroll_bottom = None;
        muxbox.vertical_scroll = Some(75.0); // Start at 75%

        // Simulate content update - should preserve scroll position
        muxbox.update_content("New content", false, true);

        // Verify scroll position preserved
        assert_eq!(muxbox.vertical_scroll, Some(75.0));
    }

    #[test]
    fn test_auto_scroll_bottom_preserves_horizontal_scroll() {
        let mut muxbox = MuxBox::default();
        muxbox.auto_scroll_bottom = Some(true);
        muxbox.horizontal_scroll = Some(30.0);
        muxbox.vertical_scroll = Some(20.0);

        // Simulate content update - should preserve horizontal, update vertical
        muxbox.update_content("New content", false, true);

        // Verify horizontal preserved, vertical set to bottom
        assert_eq!(muxbox.horizontal_scroll, Some(30.0));
        assert_eq!(muxbox.vertical_scroll, Some(100.0));
    }

    #[test]
    fn test_auto_scroll_bottom_field_serialization() {
        let muxbox = MuxBox {
            id: "test".to_string(),
            auto_scroll_bottom: Some(true),
            ..MuxBox::default()
        };

        // Verify field is included in clone
        let cloned = muxbox.clone();
        assert_eq!(cloned.auto_scroll_bottom, Some(true));
    }

    #[test]
    fn test_auto_scroll_bottom_field_equality() {
        let muxbox1 = MuxBox {
            id: "test".to_string(),
            auto_scroll_bottom: Some(true),
            ..MuxBox::default()
        };

        let muxbox2 = MuxBox {
            id: "test".to_string(),
            auto_scroll_bottom: Some(true),
            ..MuxBox::default()
        };

        let muxbox3 = MuxBox {
            id: "test".to_string(),
            auto_scroll_bottom: Some(false),
            ..MuxBox::default()
        };

        assert_eq!(muxbox1, muxbox2);
        assert_ne!(muxbox1, muxbox3);
    }
}
