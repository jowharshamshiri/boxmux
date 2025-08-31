/// F0220: ExecutionMode Enum Definition Tests
/// Comprehensive tests for ExecutionMode enum functionality and legacy migration
/// Tests enum variants, default behavior, legacy migration, and integration properties

#[cfg(test)]
mod execution_mode_tests {
    use crate::model::common::ExecutionMode;

    // === Basic Enum Functionality Tests ===

    /// Test ExecutionMode enum variants and basic functionality
    #[test]
    fn test_execution_mode_variants() {
        let immediate = ExecutionMode::Immediate;
        let thread = ExecutionMode::Thread;
        let pty = ExecutionMode::Pty;

        assert_eq!(format!("{:?}", immediate), "Immediate");
        assert_eq!(format!("{:?}", thread), "Thread");
        assert_eq!(format!("{:?}", pty), "Pty");
    }

    /// Test ExecutionMode default implementation
    #[test]
    fn test_execution_mode_default() {
        let default_mode = ExecutionMode::default();
        assert_eq!(default_mode, ExecutionMode::Immediate);
    }

    /// Test ExecutionMode clone and equality
    #[test]
    fn test_execution_mode_clone_eq() {
        let mode1 = ExecutionMode::Thread;
        let mode2 = mode1.clone();
        assert_eq!(mode1, mode2);

        let mode3 = ExecutionMode::Pty;
        assert_ne!(mode1, mode3);
    }

    /// Test ExecutionMode hash consistency
    #[test]
    fn test_execution_mode_hash() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mode1 = ExecutionMode::Thread;
        let mode2 = ExecutionMode::Thread;
        let mode3 = ExecutionMode::Pty;

        let mut hasher1 = DefaultHasher::new();
        let mut hasher2 = DefaultHasher::new();
        let mut hasher3 = DefaultHasher::new();

        mode1.hash(&mut hasher1);
        mode2.hash(&mut hasher2);
        mode3.hash(&mut hasher3);

        assert_eq!(hasher1.finish(), hasher2.finish());
        assert_ne!(hasher1.finish(), hasher3.finish());
    }

    // === Legacy Migration Tests ===

    /// Test from_legacy conversion for all boolean combinations
    #[test]
    fn test_from_legacy_conversions() {
        // thread=false, pty=false -> Immediate
        assert_eq!(
            ExecutionMode::from_legacy(false, false),
            ExecutionMode::Immediate
        );

        // thread=true, pty=false -> Thread
        assert_eq!(
            ExecutionMode::from_legacy(true, false),
            ExecutionMode::Thread
        );

        // thread=false, pty=true -> Pty
        assert_eq!(ExecutionMode::from_legacy(false, true), ExecutionMode::Pty);

        // thread=true, pty=true -> Pty (pty takes precedence)
        assert_eq!(ExecutionMode::from_legacy(true, true), ExecutionMode::Pty);
    }

    /// Test from_legacy prioritizes PTY over thread when both true
    #[test]
    fn test_from_legacy_pty_precedence() {
        let mode = ExecutionMode::from_legacy(true, true);
        assert_eq!(mode, ExecutionMode::Pty);
        assert_ne!(mode, ExecutionMode::Thread);
    }

    // === Description Tests ===

    /// Test description() method returns correct strings
    #[test]
    fn test_execution_mode_descriptions() {
        assert_eq!(
            ExecutionMode::Immediate.description(),
            "Synchronous execution on UI thread"
        );
        assert_eq!(
            ExecutionMode::Thread.description(),
            "Background execution in thread pool"
        );
        assert_eq!(
            ExecutionMode::Pty.description(),
            "Real-time PTY execution with continuous output"
        );
    }

    // === Stream Architecture Integration Tests ===

    /// Test creates_streams() returns true for all modes - no bypassing stream architecture
    #[test]
    fn test_creates_streams_always_true() {
        assert!(ExecutionMode::Immediate.creates_streams());
        assert!(ExecutionMode::Thread.creates_streams());
        assert!(ExecutionMode::Pty.creates_streams());
    }

    /// Test is_realtime() correctly identifies real-time modes
    #[test]
    fn test_is_realtime() {
        assert!(!ExecutionMode::Immediate.is_realtime());
        assert!(!ExecutionMode::Thread.is_realtime());
        assert!(ExecutionMode::Pty.is_realtime());
    }

    /// Test is_background() correctly identifies background modes
    #[test]
    fn test_is_background() {
        assert!(!ExecutionMode::Immediate.is_background());
        assert!(ExecutionMode::Thread.is_background());
        assert!(ExecutionMode::Pty.is_background());
    }

    // === Serialization Tests ===

    /// Test ExecutionMode serialization/deserialization
    #[test]
    fn test_execution_mode_serialization() {
        let modes = vec![
            ExecutionMode::Immediate,
            ExecutionMode::Thread,
            ExecutionMode::Pty,
        ];

        for mode in modes {
            let serialized = serde_json::to_string(&mode).expect("Serialization failed");
            let deserialized: ExecutionMode =
                serde_json::from_str(&serialized).expect("Deserialization failed");
            assert_eq!(mode, deserialized);
        }
    }

    /// Test ExecutionMode deserialization from string variants
    #[test]
    fn test_execution_mode_string_deserialization() {
        assert_eq!(
            serde_json::from_str::<ExecutionMode>("\"Immediate\"").unwrap(),
            ExecutionMode::Immediate
        );
        assert_eq!(
            serde_json::from_str::<ExecutionMode>("\"Thread\"").unwrap(),
            ExecutionMode::Thread
        );
        assert_eq!(
            serde_json::from_str::<ExecutionMode>("\"Pty\"").unwrap(),
            ExecutionMode::Pty
        );
    }

    // === Integration Pattern Tests ===

    /// Test ExecutionMode can be used in collections and data structures
    #[test]
    fn test_execution_mode_collections() {
        use std::collections::{HashMap, HashSet};

        let mut set = HashSet::new();
        set.insert(ExecutionMode::Immediate);
        set.insert(ExecutionMode::Thread);
        set.insert(ExecutionMode::Pty);
        assert_eq!(set.len(), 3);

        let mut map = HashMap::new();
        map.insert(ExecutionMode::Immediate, "sync");
        map.insert(ExecutionMode::Thread, "background");
        map.insert(ExecutionMode::Pty, "realtime");
        assert_eq!(map.len(), 3);
    }

    /// Test ExecutionMode method chaining and fluent usage patterns
    #[test]
    fn test_execution_mode_fluent_patterns() {
        let mode = ExecutionMode::default();

        // Test method chaining patterns
        let is_sync = !mode.is_background() && !mode.is_realtime();
        assert!(is_sync);

        let creates_and_syncs = mode.creates_streams() && !mode.is_background();
        assert!(creates_and_syncs);
    }

    /// Test ExecutionMode covers all expected behavioral combinations
    #[test]
    fn test_execution_mode_behavior_matrix() {
        let behaviors = vec![
            (ExecutionMode::Immediate, false, false, true), // (realtime, background, creates_streams)
            (ExecutionMode::Thread, false, true, true), // (realtime, background, creates_streams)
            (ExecutionMode::Pty, true, true, true),     // (realtime, background, creates_streams)
        ];

        for (mode, expected_realtime, expected_background, expected_creates_streams) in behaviors {
            assert_eq!(
                mode.is_realtime(),
                expected_realtime,
                "Realtime mismatch for {:?}",
                mode
            );
            assert_eq!(
                mode.is_background(),
                expected_background,
                "Background mismatch for {:?}",
                mode
            );
            assert_eq!(
                mode.creates_streams(),
                expected_creates_streams,
                "Creates streams mismatch for {:?}",
                mode
            );
        }
    }
}
