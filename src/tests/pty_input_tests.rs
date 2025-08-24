use crate::model::app::AppContext;
use crate::model::common::InputBounds;
use crate::model::panel::Panel;
use crate::tests::test_utils::TestDataFactory;
use crate::utils::should_use_pty;

#[cfg(test)]
mod pty_input_tests {
    use super::*;

    #[test]
    fn test_should_use_pty_enabled() {
        let mut panel = TestDataFactory::create_test_panel("pty_panel");
        panel.pty = Some(true);

        assert!(
            should_use_pty(&panel),
            "Panel with pty: true should use PTY"
        );
    }

    #[test]
    fn test_should_use_pty_disabled() {
        let mut panel = TestDataFactory::create_test_panel("regular_panel");
        panel.pty = Some(false);

        assert!(
            !should_use_pty(&panel),
            "Panel with pty: false should not use PTY"
        );
    }

    #[test]
    fn test_should_use_pty_default() {
        let panel = TestDataFactory::create_test_panel("default_panel");
        // pty field is None by default

        assert!(
            !should_use_pty(&panel),
            "Panel with no pty field should default to false"
        );
    }

    #[test]
    fn test_pty_panel_creation() {
        let panel = Panel {
            id: "pty_test".to_string(),
            title: Some("PTY Test Panel".to_string()),
            position: InputBounds {
                x1: "0%".to_string(),
                y1: "0%".to_string(),
                x2: "100%".to_string(),
                y2: "100%".to_string(),
            },
            border: Some(true),
            pty: Some(true),
            script: Some(vec!["echo 'PTY test'".to_string()]),
            ..Default::default()
        };

        assert_eq!(panel.id, "pty_test");
        assert_eq!(panel.pty, Some(true));
        assert!(should_use_pty(&panel));
        assert_eq!(panel.script.as_ref().unwrap()[0], "echo 'PTY test'");
    }
}
