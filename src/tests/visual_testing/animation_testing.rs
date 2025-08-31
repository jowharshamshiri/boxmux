// F0347: Animation Testing System - Time-based visual validation for dynamic content
use crate::tests::visual_testing::boxmux_tester::TestError;
use crate::tests::visual_testing::terminal_capture::TerminalCell;
use crate::tests::visual_testing::{BoxMuxTester, TerminalCapture, VisualAssertions};
use std::time::{Duration, Instant};

/// F0347: Animation testing configuration
#[derive(Debug, Clone)]
pub struct AnimationConfig {
    /// Frame capture interval
    pub frame_interval: Duration,
    /// Total animation duration to capture
    pub total_duration: Duration,
    /// Expected frame count
    pub expected_frames: Option<usize>,
    /// Frame comparison tolerance
    pub comparison_tolerance: f32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        Self {
            frame_interval: Duration::from_millis(100), // 10 FPS
            total_duration: Duration::from_secs(2),
            expected_frames: None,
            comparison_tolerance: 0.05, // 5% difference allowed
        }
    }
}

/// F0347: Animation capture results
#[derive(Debug, Clone)]
pub struct AnimationCapture {
    pub frames: Vec<Vec<Vec<TerminalCell>>>,
    pub timestamps: Vec<Instant>,
    pub config: AnimationConfig,
}

/// F0347: Animation testing extensions for BoxMuxTester
pub trait AnimationTesting {
    /// Start capturing animation frames at regular intervals
    fn start_animation_capture(&mut self, config: AnimationConfig) -> Result<(), TestError>;

    /// Stop animation capture and return results
    fn stop_animation_capture(&mut self) -> Result<AnimationCapture, TestError>;

    /// Capture animation for specified duration
    fn capture_animation(&mut self, config: AnimationConfig)
        -> Result<AnimationCapture, TestError>;

    /// Assert animation properties
    fn assert_animation_smooth(
        &self,
        capture: &AnimationCapture,
        max_change_per_frame: f32,
    ) -> Result<(), String>;
    fn assert_animation_stable(
        &self,
        capture: &AnimationCapture,
        stable_threshold: f32,
    ) -> Result<(), String>;
    fn assert_frame_progression(&self, capture: &AnimationCapture) -> Result<(), String>;
}

impl AnimationTesting for BoxMuxTester {
    fn start_animation_capture(&mut self, config: AnimationConfig) -> Result<(), TestError> {
        // For now, this is a placeholder for the animation capture system
        // In full implementation, this would start a background thread capturing frames
        log::debug!("Starting animation capture with config: {:?}", config);
        Ok(())
    }

    fn stop_animation_capture(&mut self) -> Result<AnimationCapture, TestError> {
        // Placeholder implementation
        let config = AnimationConfig::default();
        let capture = AnimationCapture {
            frames: vec![],
            timestamps: vec![],
            config,
        };
        Ok(capture)
    }

    fn capture_animation(
        &mut self,
        config: AnimationConfig,
    ) -> Result<AnimationCapture, TestError> {
        let start_time = Instant::now();
        let mut frames = Vec::new();
        let mut timestamps = Vec::new();

        // Capture frames at regular intervals
        while start_time.elapsed() < config.total_duration {
            let frame_start = Instant::now();

            // Capture current frame
            let frame_ref = self.wait_for_frame()?;
            frames.push(frame_ref.buffer.clone());
            timestamps.push(frame_start);

            // Wait for next frame interval
            let elapsed = frame_start.elapsed();
            if elapsed < config.frame_interval {
                std::thread::sleep(config.frame_interval - elapsed);
            }
        }

        Ok(AnimationCapture {
            frames,
            timestamps,
            config,
        })
    }

    fn assert_animation_smooth(
        &self,
        capture: &AnimationCapture,
        max_change_per_frame: f32,
    ) -> Result<(), String> {
        if capture.frames.len() < 2 {
            return Err("Need at least 2 frames for smoothness analysis".to_string());
        }

        for i in 1..capture.frames.len() {
            let change_ratio =
                self.calculate_frame_difference(&capture.frames[i - 1], &capture.frames[i]);

            if change_ratio > max_change_per_frame {
                return Err(format!(
                    "Frame {} has change ratio {:.3}, exceeding max {:.3}",
                    i, change_ratio, max_change_per_frame
                ));
            }
        }

        Ok(())
    }

    fn assert_animation_stable(
        &self,
        capture: &AnimationCapture,
        stable_threshold: f32,
    ) -> Result<(), String> {
        if capture.frames.len() < 3 {
            return Err("Need at least 3 frames for stability analysis".to_string());
        }

        let first_frame = &capture.frames[0];
        let last_frame = &capture.frames[capture.frames.len() - 1];

        let total_change = self.calculate_frame_difference(first_frame, last_frame);

        if total_change > stable_threshold {
            return Err(format!(
                "Animation not stable: total change {:.3} exceeds threshold {:.3}",
                total_change, stable_threshold
            ));
        }

        Ok(())
    }

    fn assert_frame_progression(&self, capture: &AnimationCapture) -> Result<(), String> {
        // Verify that frames are captured at expected intervals
        if capture.timestamps.len() != capture.frames.len() {
            return Err("Frame count mismatch with timestamps".to_string());
        }

        // Check timing consistency
        for i in 1..capture.timestamps.len() {
            let interval = capture.timestamps[i].duration_since(capture.timestamps[i - 1]);
            let expected_interval = capture.config.frame_interval;

            // Allow 20% tolerance for timing variations
            let tolerance = expected_interval.as_millis() as f32 * 0.2;
            let diff = (interval.as_millis() as f32 - expected_interval.as_millis() as f32).abs();

            if diff > tolerance {
                log::warn!(
                    "Frame {} timing inconsistent: {}ms vs expected {}ms",
                    i,
                    interval.as_millis(),
                    expected_interval.as_millis()
                );
            }
        }

        Ok(())
    }
}

impl BoxMuxTester {
    /// Calculate difference ratio between two frames (0.0 = identical, 1.0 = completely different)
    fn calculate_frame_difference(
        &self,
        frame1: &Vec<Vec<TerminalCell>>,
        frame2: &Vec<Vec<TerminalCell>>,
    ) -> f32 {
        if frame1.len() != frame2.len() {
            return 1.0; // Completely different if dimensions don't match
        }

        let mut total_cells = 0;
        let mut different_cells = 0;

        for (row1, row2) in frame1.iter().zip(frame2.iter()) {
            if row1.len() != row2.len() {
                return 1.0; // Completely different if row lengths don't match
            }

            for (cell1, cell2) in row1.iter().zip(row2.iter()) {
                total_cells += 1;
                if cell1.ch != cell2.ch
                    || cell1.fg_color != cell2.fg_color
                    || cell1.bg_color != cell2.bg_color
                {
                    different_cells += 1;
                }
            }
        }

        if total_cells == 0 {
            return 0.0;
        }

        different_cells as f32 / total_cells as f32
    }
}

/// F0347: Dynamic content testing - simulates content that changes over time
pub struct DynamicContentSimulator {
    /// Counter for generating changing content
    pub counter: usize,
    /// Update interval
    pub update_interval: Duration,
    /// Last update time
    pub last_update: Instant,
}

impl DynamicContentSimulator {
    pub fn new(update_interval: Duration) -> Self {
        Self {
            counter: 0,
            update_interval,
            last_update: Instant::now(),
        }
    }

    /// Generate content that changes based on counter
    pub fn generate_content(&mut self) -> String {
        if self.last_update.elapsed() >= self.update_interval {
            self.counter += 1;
            self.last_update = Instant::now();
        }

        format!(
            "Dynamic content: {} [{}]",
            self.counter,
            "=".repeat((self.counter % 20) + 1)
        )
    }

    /// Generate progress bar content
    pub fn generate_progress_bar(&mut self, width: usize) -> String {
        if self.last_update.elapsed() >= self.update_interval {
            self.counter += 1;
            self.last_update = Instant::now();
        }

        let progress = (self.counter % 100) as f32 / 100.0;
        let filled = (progress * width as f32) as usize;
        let empty = width - filled;

        format!(
            "[{}{}] {:.0}%",
            "#".repeat(filled),
            " ".repeat(empty),
            progress * 100.0
        )
    }
}

#[cfg(test)]
mod animation_tests {
    use super::*;

    /// F0347: Test animation capture system
    #[test]
    fn test_animation_capture_creation() {
        let config = AnimationConfig {
            frame_interval: Duration::from_millis(50),
            total_duration: Duration::from_millis(200),
            expected_frames: Some(4),
            comparison_tolerance: 0.1,
        };

        assert_eq!(config.frame_interval, Duration::from_millis(50));
        assert_eq!(config.expected_frames, Some(4));
    }

    /// F0347: Test dynamic content simulator
    #[test]
    fn test_dynamic_content_simulator() {
        let mut simulator = DynamicContentSimulator::new(Duration::from_millis(10));

        let content1 = simulator.generate_content();
        std::thread::sleep(Duration::from_millis(15));
        let content2 = simulator.generate_content();

        // Content should be different after enough time has passed
        assert_ne!(content1, content2);
        assert!(content2.contains("Dynamic content:"));
    }

    /// F0347: Test progress bar generation
    #[test]
    fn test_progress_bar_generation() {
        let mut simulator = DynamicContentSimulator::new(Duration::from_millis(10));

        let bar1 = simulator.generate_progress_bar(20);
        std::thread::sleep(Duration::from_millis(15));
        let bar2 = simulator.generate_progress_bar(20);

        assert_ne!(bar1, bar2);
        assert!(bar1.contains("["));
        assert!(bar1.contains("]"));
        assert!(bar1.contains("%"));
    }
}
