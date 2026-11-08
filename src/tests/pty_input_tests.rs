use crate::model::common::InputBounds;
use crate::model::muxbox::MuxBox;
use crate::tests::test_utils::TestDataFactory;
use crate::utils::should_use_pty;

#[cfg(test)]
mod pty_input_tests {
    use super::*;
    use crate::ansi_processor::TerminalMode;
    use crate::circular_buffer::CircularBuffer;
    use crate::pty_manager::{PtyManager, PtyStatus};
    use crossterm::event::{KeyModifiers, MouseButton, MouseEventKind};
    use std::sync::{Arc, Mutex};

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

        assert_eq!(
            muxbox.execution_mode,
            crate::model::common::ExecutionMode::Immediate
        );
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
            execution_mode: crate::model::common::ExecutionMode::Pty, // F0226: ExecutionMode determines PTY usage
            script: Some(vec!["echo 'PTY test'".to_string()]),
            ..Default::default()
        };

        assert_eq!(muxbox.id, "pty_test");
        assert!(should_use_pty(&muxbox));
        assert_eq!(muxbox.script.as_ref().unwrap()[0], "echo 'PTY test'");
    }

    #[test]
    fn test_pty_mouse_sequence_requires_active_mouse_reporting() {
        let pty_manager = PtyManager::new().expect("PTY manager");
        pty_manager.add_test_pty_process(
            "pty_test".to_string(),
            Arc::new(Mutex::new(CircularBuffer::new(16))),
        );

        let event = MouseEventKind::Down(MouseButton::Left);
        assert_eq!(
            pty_manager.generate_mouse_sequence("pty_test", event, 10, 5, KeyModifiers::empty()),
            None,
            "default PTY mode must not receive mouse escape sequences"
        );

        let mut terminal_mode = TerminalMode::default();
        terminal_mode.mouse_sgr_mode = true;
        terminal_mode.mouse_normal_mode = true;
        pty_manager.set_test_pty_terminal_mode("pty_test", terminal_mode);

        let sequence = pty_manager
            .generate_mouse_sequence("pty_test", event, 10, 5, KeyModifiers::empty())
            .expect("mouse sequence after explicit terminal opt-in");
        assert!(
            sequence.starts_with("\x1b[<"),
            "SGR mouse reporting should generate an SGR mouse sequence"
        );

        pty_manager.add_test_pty_process_with_status(
            "finished_pty".to_string(),
            Arc::new(Mutex::new(CircularBuffer::new(16))),
            PtyStatus::Finished(1),
            123,
        );
        let mut finished_mode = TerminalMode::default();
        finished_mode.mouse_sgr_mode = true;
        finished_mode.mouse_normal_mode = true;
        pty_manager.set_test_pty_terminal_mode("finished_pty", finished_mode);

        assert_eq!(
            pty_manager.generate_mouse_sequence(
                "finished_pty",
                event,
                10,
                5,
                KeyModifiers::empty()
            ),
            None,
            "finished PTYs must not receive stale mouse escape sequences"
        );
    }
}
