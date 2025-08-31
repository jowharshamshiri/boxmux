// F0326-F0355: Visual Testing System - Character-Exact Validation
// Core module for visual testing infrastructure

pub mod animation_testing;
pub mod boxmux_tester;
pub mod frame_comparison;
pub mod pattern_matching;
pub mod terminal_capture;
pub mod visual_assertions;

// Re-export key types for easy access
pub use animation_testing::{
    AnimationCapture, AnimationConfig, AnimationTesting, DynamicContentSimulator,
};
pub use boxmux_tester::{BoxMuxTester, TestConfig};
pub use frame_comparison::FrameDiff;
pub use pattern_matching::PatternMatcher;
pub use terminal_capture::{TerminalCapture, TerminalFrame};
pub use visual_assertions::VisualAssertions;

// F0326: Terminal Frame Capture constants
pub const DEFAULT_TERMINAL_WIDTH: u16 = 80;
pub const DEFAULT_TERMINAL_HEIGHT: u16 = 24;
pub const MAX_FRAME_HISTORY: usize = 1000;

// F0330: Visual diff detection constants
pub const MAX_ACCEPTABLE_CHANGES: usize = 100;
pub const FRAME_CAPTURE_TIMEOUT_MS: u64 = 1000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visual_testing_module_constants() {
        assert_eq!(DEFAULT_TERMINAL_WIDTH, 80);
        assert_eq!(DEFAULT_TERMINAL_HEIGHT, 24);
        assert!(MAX_FRAME_HISTORY > 0);
    }
}
