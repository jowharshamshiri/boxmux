use crate::model::app::AppContext;
use crate::model::common::InputBounds;
use crate::model::muxbox::MuxBox;
use crate::tests::test_utils::TestDataFactory;
use crate::utils::should_use_pty;

#[cfg(test)]
mod pty_input_tests {
    use super::*;

    #[test]
    fn test_should_use_pty_enabled() {
        let mut muxbox = TestDataFactory::create_test_muxbox("pty_muxbox");
        muxbox.pty = Some(true);

        assert!(
            should_use_pty(&muxbox),
            "MuxBox with pty: true should use PTY"
        );
    }

    #[test]
    fn test_should_use_pty_disabled() {
        let mut muxbox = TestDataFactory::create_test_muxbox("regular_muxbox");
        muxbox.pty = Some(false);

        assert!(
            !should_use_pty(&muxbox),
            "MuxBox with pty: false should not use PTY"
        );
    }

    #[test]
    fn test_should_use_pty_default() {
        let muxbox = TestDataFactory::create_test_muxbox("default_muxbox");
        // pty field is None by default

        assert!(
            !should_use_pty(&muxbox),
            "MuxBox with no pty field should default to false"
        );
    }

    #[test]
    fn test_pty_muxbox_creation() {
        let muxbox = MuxBox {
            id: "pty_test".to_string(),
            title: Some("PTY Test MuxBox".to_string()),
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

        assert_eq!(muxbox.id, "pty_test");
        assert_eq!(muxbox.pty, Some(true));
        assert!(should_use_pty(&muxbox));
        assert_eq!(muxbox.script.as_ref().unwrap()[0], "echo 'PTY test'");
    }
}
