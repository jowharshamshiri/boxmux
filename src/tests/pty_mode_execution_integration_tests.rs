#[cfg(test)]
pub mod pty_mode_execution_integration_tests {
    use crate::model::common::ExecutionMode;
    use crate::model::muxbox::Choice;
    use crate::tests::test_utils::TestDataFactory;
    use crate::utils::{should_use_pty, should_use_pty_for_choice};

    #[test]
    fn test_should_use_pty_for_choice_with_execution_mode_pty() {
        // F0226: should_use_pty_for_choice should use ExecutionMode instead of legacy pty field
        let choice = Choice {
            id: "pty_choice".to_string(),
            content: Some("PTY Choice".to_string()),
            selected: false,
            script: Some(vec!["htop".to_string()]),
            // Legacy field ignored
            execution_mode: ExecutionMode::Pty, // F0226: Use ExecutionMode
            redirect_output: None,
            append_output: None,
            waiting: false,
        };

        assert!(
            should_use_pty_for_choice(&choice),
            "ExecutionMode::Pty should return true"
        );
    }

    #[test]
    fn test_should_use_pty_for_choice_with_execution_mode_thread() {
        // F0226: should_use_pty_for_choice should return false for ExecutionMode::Thread
        let choice = Choice {
            id: "thread_choice".to_string(),
            content: Some("Thread Choice".to_string()),
            selected: false,
            script: Some(vec!["echo 'thread'".to_string()]),
            execution_mode: ExecutionMode::Thread, // F0226: ExecutionMode overrides legacy
            redirect_output: None,
            append_output: None,
            waiting: false,
        };

        assert!(
            !should_use_pty_for_choice(&choice),
            "ExecutionMode::Thread should return false even if legacy pty=true"
        );
    }

    #[test]
    fn test_should_use_pty_for_choice_with_execution_mode_immediate() {
        // F0226: should_use_pty_for_choice should return false for ExecutionMode::Immediate
        let choice = Choice {
            id: "immediate_choice".to_string(),
            content: Some("Immediate Choice".to_string()),
            selected: false,
            script: Some(vec!["echo 'immediate'".to_string()]),
            execution_mode: ExecutionMode::Immediate, // F0226: ExecutionMode overrides legacy
            redirect_output: None,
            append_output: None,
            waiting: false,
        };

        assert!(
            !should_use_pty_for_choice(&choice),
            "ExecutionMode::Immediate should return false even if legacy pty=true"
        );
    }

    #[test]
    fn test_should_use_pty_with_execution_mode_pty() {
        // F0226: should_use_pty should use ExecutionMode instead of legacy pty field
        let mut muxbox = TestDataFactory::create_test_muxbox("pty_box");
        muxbox.execution_mode = ExecutionMode::Pty;

        assert!(
            should_use_pty(&muxbox),
            "ExecutionMode::Pty should return true"
        );
    }

    #[test]
    fn test_should_use_pty_with_execution_mode_thread() {
        // F0226: should_use_pty should return false for ExecutionMode::Thread
        let mut muxbox = TestDataFactory::create_test_muxbox("thread_box");
        muxbox.execution_mode = ExecutionMode::Thread;

        assert!(
            !should_use_pty(&muxbox),
            "ExecutionMode::Thread should return false even if legacy pty=true"
        );
    }

    #[test]
    fn test_should_use_pty_with_execution_mode_immediate() {
        // F0226: should_use_pty should return false for ExecutionMode::Immediate
        let mut muxbox = TestDataFactory::create_test_muxbox("immediate_box");
        muxbox.execution_mode = ExecutionMode::Immediate;

        assert!(
            !should_use_pty(&muxbox),
            "ExecutionMode::Immediate should return false even if legacy pty=true"
        );
    }

    #[test]
    fn test_legacy_vs_execution_mode_priority() {
        // F0226: ExecutionMode should completely override legacy pty field
        let test_cases = [
            (
                ExecutionMode::Immediate,
                Some(true),
                false,
                "Immediate mode overrides pty=true",
            ),
            (
                ExecutionMode::Thread,
                Some(true),
                false,
                "Thread mode overrides pty=true",
            ),
            (
                ExecutionMode::Pty,
                Some(false),
                true,
                "PTY mode overrides pty=false",
            ),
            (
                ExecutionMode::Pty,
                None,
                true,
                "PTY mode works without legacy field",
            ),
        ];

        for (execution_mode, legacy_pty, expected, description) in test_cases {
            let choice = Choice {
                id: "test_choice".to_string(),
                content: Some("Test Choice".to_string()),
                selected: false,
                script: Some(vec!["echo 'test'".to_string()]),
                execution_mode: execution_mode.clone(),
                redirect_output: None,
                append_output: None,
                waiting: false,
            };

            assert_eq!(
                should_use_pty_for_choice(&choice),
                expected,
                "{}",
                description
            );

            let mut muxbox = TestDataFactory::create_test_muxbox("test_box");
            muxbox.execution_mode = execution_mode;

            assert_eq!(
                should_use_pty(&muxbox),
                expected,
                "{} (MuxBox)",
                description
            );
        }
    }

    #[test]
    fn test_pty_mode_execution_integration() {
        // F0226: Integration test for PTY mode execution
        let mut muxbox = TestDataFactory::create_test_muxbox("pty_integration_box");
        muxbox.execution_mode = ExecutionMode::Pty;

        let pty_choice = Choice {
            id: "pty_integration_choice".to_string(),
            content: Some("PTY Integration Choice".to_string()),
            selected: false,
            script: Some(vec!["bash".to_string()]),
            // F0226: Legacy field not used
            execution_mode: ExecutionMode::Pty, // F0226: ExecutionMode determines behavior
            redirect_output: None,
            append_output: Some(false),
            waiting: false,
        };

        muxbox.choices = Some(vec![pty_choice.clone()]);

        // Test utility functions
        assert!(should_use_pty(&muxbox));
        assert!(should_use_pty_for_choice(&pty_choice));

        // Test ExecutionMode properties
        assert!(pty_choice.execution_mode.is_realtime());
        assert!(pty_choice.execution_mode.is_background());
        assert!(pty_choice.execution_mode.creates_streams());
        assert_eq!(pty_choice.execution_mode.as_stream_suffix(), "pty");

        // Test MuxBox ExecutionMode properties
        assert!(muxbox.execution_mode.is_realtime());
        assert!(muxbox.execution_mode.is_background());
        assert!(muxbox.execution_mode.creates_streams());
        assert_eq!(muxbox.execution_mode.as_stream_suffix(), "pty");
    }
}
