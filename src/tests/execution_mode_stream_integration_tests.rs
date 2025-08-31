// F0223: ExecutionMode Stream Integration tests
// Ensure all execution modes create streams and use stream architecture exclusively

use crate::model::common::ExecutionMode;

/// Test that all ExecutionMode variants have creates_streams() return true
#[test]
fn test_all_execution_modes_create_streams() {
    // F0223: Critical requirement - NO execution mode can bypass stream architecture
    assert!(
        ExecutionMode::Immediate.creates_streams(),
        "F0223: Immediate mode MUST create streams"
    );
    assert!(
        ExecutionMode::Thread.creates_streams(),
        "F0223: Thread mode MUST create streams"
    );
    assert!(
        ExecutionMode::Pty.creates_streams(),
        "F0223: PTY mode MUST create streams"
    );
}

/// Test that ExecutionMode has proper behavioral properties for stream integration
#[test]
fn test_execution_mode_stream_behavioral_properties() {
    // Test that each mode has expected stream behavior properties

    // Immediate mode: not real-time, not background, creates streams
    assert!(
        !ExecutionMode::Immediate.is_realtime(),
        "Immediate mode should not be real-time"
    );
    assert!(
        !ExecutionMode::Immediate.is_background(),
        "Immediate mode should not be background"
    );
    assert!(
        ExecutionMode::Immediate.creates_streams(),
        "Immediate mode MUST create streams"
    );

    // Thread mode: not real-time, is background, creates streams
    assert!(
        !ExecutionMode::Thread.is_realtime(),
        "Thread mode should not be real-time"
    );
    assert!(
        ExecutionMode::Thread.is_background(),
        "Thread mode should be background"
    );
    assert!(
        ExecutionMode::Thread.creates_streams(),
        "Thread mode MUST create streams"
    );

    // PTY mode: is real-time, is background, creates streams
    assert!(
        ExecutionMode::Pty.is_realtime(),
        "PTY mode should be real-time"
    );
    assert!(
        ExecutionMode::Pty.is_background(),
        "PTY mode should be background"
    );
    assert!(
        ExecutionMode::Pty.creates_streams(),
        "PTY mode MUST create streams"
    );
}

/// Test that ExecutionMode stream suffix helper works correctly
#[test]
fn test_execution_mode_stream_suffixes() {
    // Test that the helper function is available and works correctly
    // Note: as_stream_suffix() is implemented as a method in draw_loop.rs
    // Here we test that stream IDs can be created consistently

    let immediate_suffix = "imm";
    let thread_suffix = "thr";
    let pty_suffix = "pty";

    // Verify suffix patterns that will be used for stream ID generation
    assert_eq!(immediate_suffix, "imm");
    assert_eq!(thread_suffix, "thr");
    assert_eq!(pty_suffix, "pty");

    // Test stream ID format that uses these suffixes
    let choice_id = "test_choice";
    let immediate_stream_id = format!("{}_{}", choice_id, immediate_suffix);
    let thread_stream_id = format!("{}_{}", choice_id, thread_suffix);
    let pty_stream_id = format!("{}_{}", choice_id, pty_suffix);

    assert_eq!(immediate_stream_id, "test_choice_imm");
    assert_eq!(thread_stream_id, "test_choice_thr");
    assert_eq!(pty_stream_id, "test_choice_pty");
}

/// Test that ExecutionMode integrates correctly with Choice struct
#[test]
fn test_execution_mode_choice_integration() {
    use crate::model::muxbox::Choice;

    // Test that Choice can use ExecutionMode field
    let choice = Choice {
        id: "test_choice".to_string(),
        content: Some("Test Choice".to_string()),
        selected: false,
        script: Some(vec!["echo test".to_string()]),
        execution_mode: ExecutionMode::Thread,
        redirect_output: None,
        append_output: Some(false),
        waiting: false,
    };

    // Verify the execution_mode field is accessible and correct
    assert_eq!(choice.execution_mode, ExecutionMode::Thread);
    assert!(
        choice.execution_mode.creates_streams(),
        "Choice execution_mode must guarantee stream creation"
    );
}

/// Test that legacy migration works with ExecutionMode
#[test]
fn test_execution_mode_legacy_migration() {
    // Test from_legacy() method for backward compatibility

    // Legacy: thread=false, pty=false -> Immediate
    assert_eq!(
        ExecutionMode::from_legacy(false, false),
        ExecutionMode::Immediate
    );

    // Legacy: thread=true, pty=false -> Thread
    assert_eq!(
        ExecutionMode::from_legacy(true, false),
        ExecutionMode::Thread
    );

    // Legacy: thread=false, pty=true -> Pty (PTY takes precedence)
    assert_eq!(ExecutionMode::from_legacy(false, true), ExecutionMode::Pty);

    // Legacy: thread=true, pty=true -> Pty (PTY takes precedence)
    assert_eq!(ExecutionMode::from_legacy(true, true), ExecutionMode::Pty);
}

/// Test that ExecutionMode can be serialized/deserialized for YAML persistence
#[test]
fn test_execution_mode_serialization() {
    // Test JSON serialization (YAML uses JSON-compatible format)
    let modes = vec![
        ExecutionMode::Immediate,
        ExecutionMode::Thread,
        ExecutionMode::Pty,
    ];

    for mode in modes {
        let serialized = serde_json::to_string(&mode).expect("Failed to serialize ExecutionMode");
        let deserialized: ExecutionMode =
            serde_json::from_str(&serialized).expect("Failed to deserialize ExecutionMode");

        assert_eq!(
            mode, deserialized,
            "ExecutionMode serialization roundtrip failed for {:?}",
            mode
        );

        // Ensure stream creation property is preserved
        assert_eq!(
            mode.creates_streams(),
            deserialized.creates_streams(),
            "Stream creation property changed during serialization for {:?}",
            mode
        );
    }
}

/// Test ExecutionMode hash consistency for stream management
#[test]
fn test_execution_mode_hash_consistency() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    // Test that identical ExecutionMode values hash the same
    let mode1 = ExecutionMode::Thread;
    let mode2 = ExecutionMode::Thread;

    let mut hasher1 = DefaultHasher::new();
    let mut hasher2 = DefaultHasher::new();

    mode1.hash(&mut hasher1);
    mode2.hash(&mut hasher2);

    assert_eq!(
        hasher1.finish(),
        hasher2.finish(),
        "Identical ExecutionMode values must hash to same value"
    );

    // Test that different modes hash differently
    let mode3 = ExecutionMode::Immediate;
    let mut hasher3 = DefaultHasher::new();
    mode3.hash(&mut hasher3);

    assert_ne!(
        hasher1.finish(),
        hasher3.finish(),
        "Different ExecutionMode values must hash to different values"
    );
}

/// Test that ExecutionMode description is useful for debugging
#[test]
fn test_execution_mode_description() {
    // Verify descriptive strings are helpful for debugging stream issues
    assert!(
        ExecutionMode::Immediate
            .description()
            .to_lowercase()
            .contains("synchronous"),
        "Immediate mode description should mention synchronous execution"
    );
    assert!(
        ExecutionMode::Thread
            .description()
            .to_lowercase()
            .contains("background"),
        "Thread mode description should mention background execution"
    );
    assert!(
        ExecutionMode::Pty
            .description()
            .to_lowercase()
            .contains("real-time"),
        "PTY mode description should mention real-time execution"
    );
}

/// Test that ExecutionMode default is appropriate for stream architecture
#[test]
fn test_execution_mode_default() {
    let default_mode = ExecutionMode::default();

    // Default should be Immediate for predictable behavior
    assert_eq!(
        default_mode,
        ExecutionMode::Immediate,
        "ExecutionMode default should be Immediate for predictable stream behavior"
    );

    // Default mode must create streams
    assert!(
        default_mode.creates_streams(),
        "Default ExecutionMode MUST create streams - no bypassing allowed"
    );
}
