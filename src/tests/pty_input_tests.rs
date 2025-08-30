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
        muxbox.execution_mode = crate::model::common::ExecutionMode::Pty; // F0226: ExecutionMode determines PTY usage

        assert!(
            should_use_pty(&muxbox),
            "MuxBox with ExecutionMode::Pty should use PTY"
        );
    }

    #[test]
    fn test_should_use_pty_disabled() {
        let mut muxbox = TestDataFactory::create_test_muxbox("regular_muxbox");
        muxbox.execution_mode = crate::model::common::ExecutionMode::Thread; // F0226: ExecutionMode overrides legacy pty field

        assert!(
            !should_use_pty(&muxbox),
            "MuxBox with ExecutionMode::Thread should not use PTY even if legacy pty=true"
        );
    }

    #[test]
    fn test_should_use_pty_default() {
        let muxbox = TestDataFactory::create_test_muxbox("default_muxbox");
        // F0226: ExecutionMode defaults to Immediate, pty field is None by default

        assert_eq!(muxbox.execution_mode, crate::model::common::ExecutionMode::Immediate);
        assert!(
            !should_use_pty(&muxbox),
            "MuxBox with default ExecutionMode::Immediate should not use PTY"
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
            execution_mode: crate::model::common::ExecutionMode::Pty, // F0226: ExecutionMode determines PTY usage
            script: Some(vec!["echo 'PTY test'".to_string()]),
            ..Default::default()
        };

        assert_eq!(muxbox.id, "pty_test");
        assert!(should_use_pty(&muxbox));
        assert_eq!(muxbox.script.as_ref().unwrap()[0], "echo 'PTY test'");
    }
}
