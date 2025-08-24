#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::panel::Panel;

    #[test]
    fn test_auto_scroll_bottom_field_creation() {
        let mut panel = Panel::default();
        panel.auto_scroll_bottom = Some(true);

        assert_eq!(panel.auto_scroll_bottom, Some(true));
    }

    #[test]
    fn test_auto_scroll_bottom_content_update() {
        let mut panel = Panel::default();
        panel.auto_scroll_bottom = Some(true);
        panel.vertical_scroll = Some(25.0); // Start at 25%

        // Simulate content update - should auto-scroll to bottom
        panel.update_content("New content", false, true);

        // Verify auto-scroll set vertical scroll to 100%
        assert_eq!(panel.vertical_scroll, Some(100.0));
    }

    #[test]
    fn test_auto_scroll_bottom_disabled_preserves_scroll() {
        let mut panel = Panel::default();
        panel.auto_scroll_bottom = Some(false);
        panel.vertical_scroll = Some(50.0); // Start at 50%

        // Simulate content update - should preserve scroll position
        panel.update_content("New content", false, true);

        // Verify scroll position preserved
        assert_eq!(panel.vertical_scroll, Some(50.0));
    }

    #[test]
    fn test_auto_scroll_bottom_none_preserves_scroll() {
        let mut panel = Panel::default();
        panel.auto_scroll_bottom = None;
        panel.vertical_scroll = Some(75.0); // Start at 75%

        // Simulate content update - should preserve scroll position
        panel.update_content("New content", false, true);

        // Verify scroll position preserved
        assert_eq!(panel.vertical_scroll, Some(75.0));
    }

    #[test]
    fn test_auto_scroll_bottom_preserves_horizontal_scroll() {
        let mut panel = Panel::default();
        panel.auto_scroll_bottom = Some(true);
        panel.horizontal_scroll = Some(30.0);
        panel.vertical_scroll = Some(20.0);

        // Simulate content update - should preserve horizontal, update vertical
        panel.update_content("New content", false, true);

        // Verify horizontal preserved, vertical set to bottom
        assert_eq!(panel.horizontal_scroll, Some(30.0));
        assert_eq!(panel.vertical_scroll, Some(100.0));
    }

    #[test]
    fn test_auto_scroll_bottom_field_serialization() {
        let panel = Panel {
            id: "test".to_string(),
            auto_scroll_bottom: Some(true),
            ..Panel::default()
        };

        // Verify field is included in clone
        let cloned = panel.clone();
        assert_eq!(cloned.auto_scroll_bottom, Some(true));
    }

    #[test]
    fn test_auto_scroll_bottom_field_equality() {
        let panel1 = Panel {
            id: "test".to_string(),
            auto_scroll_bottom: Some(true),
            ..Panel::default()
        };

        let panel2 = Panel {
            id: "test".to_string(),
            auto_scroll_bottom: Some(true),
            ..Panel::default()
        };

        let panel3 = Panel {
            id: "test".to_string(),
            auto_scroll_bottom: Some(false),
            ..Panel::default()
        };

        assert_eq!(panel1, panel2);
        assert_ne!(panel1, panel3);
    }
}
