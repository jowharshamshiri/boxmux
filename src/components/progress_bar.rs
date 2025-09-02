//! Progress Bar Component - Interactive progress bar component for loading states
//!
//! This component provides comprehensive progress bar rendering with customizable
//! styles, animations, and progress tracking. Supports horizontal and vertical bars,
//! percentage display, and integration with long-running operations.

use crate::model::common::Bounds;
use crossterm::style::Color;
use std::time::{Duration, Instant};

/// Configuration for progress bar styling and behavior
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressBarConfig {
    /// Progress bar orientation
    pub orientation: ProgressBarOrientation,
    /// Fill character for completed portion
    pub fill_char: char,
    /// Background character for uncompleted portion
    pub background_char: char,
    /// Color for completed portion
    pub fill_color: Color,
    /// Color for uncompleted portion
    pub background_color: Color,
    /// Color for percentage text
    pub text_color: Color,
    /// Show percentage text
    pub show_percentage: bool,
    /// Show progress text (e.g., "Loading...")
    pub show_progress_text: bool,
    /// Progress text template
    pub progress_text_template: String,
    /// Show animation
    pub animated: bool,
    /// Animation speed (frames per second)
    pub animation_speed: f64,
    /// Border characters for styling
    pub border_chars: Option<ProgressBarBorders>,
    /// Minimum width for horizontal bars
    pub min_width: usize,
    /// Minimum height for vertical bars
    pub min_height: usize,
}

/// Progress bar orientation
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressBarOrientation {
    Horizontal,
    Vertical,
}

/// Border characters for progress bar styling
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressBarBorders {
    pub left: char,
    pub right: char,
    pub top: char,
    pub bottom: char,
}

impl Default for ProgressBarConfig {
    fn default() -> Self {
        Self {
            orientation: ProgressBarOrientation::Horizontal,
            fill_char: '█',
            background_char: '░',
            fill_color: Color::Green,
            background_color: Color::DarkGrey,
            text_color: Color::White,
            show_percentage: true,
            show_progress_text: false,
            progress_text_template: "{text} {percentage}%".to_string(),
            animated: false,
            animation_speed: 2.0,
            border_chars: Some(ProgressBarBorders {
                left: '[',
                right: ']',
                top: '─',
                bottom: '─',
            }),
            min_width: 10,
            min_height: 3,
        }
    }
}

/// Progress state for tracking current progress
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressState {
    /// Current progress value (0.0 - 1.0)
    pub progress: f64,
    /// Optional progress text
    pub text: Option<String>,
    /// Start time for calculating duration
    pub start_time: Instant,
    /// Last update time for animation
    pub last_update: Instant,
    /// Animation frame counter
    pub animation_frame: usize,
}

impl Default for ProgressState {
    fn default() -> Self {
        let now = Instant::now();
        Self {
            progress: 0.0,
            text: None,
            start_time: now,
            last_update: now,
            animation_frame: 0,
        }
    }
}

impl ProgressState {
    /// Create new progress state with current time
    pub fn new() -> Self {
        Self::default()
    }

    /// Update progress value (0.0 - 1.0)
    pub fn set_progress(&mut self, progress: f64) {
        self.progress = progress.clamp(0.0, 1.0);
        self.last_update = Instant::now();
    }

    /// Set progress text
    pub fn set_text(&mut self, text: String) {
        self.text = Some(text);
        self.last_update = Instant::now();
    }

    /// Clear progress text
    pub fn clear_text(&mut self) {
        self.text = None;
        self.last_update = Instant::now();
    }

    /// Get elapsed time since start
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Update animation frame
    pub fn update_animation(&mut self) {
        self.animation_frame = self.animation_frame.wrapping_add(1);
        self.last_update = Instant::now();
    }
}

/// Progress Bar Component for interactive progress display
pub struct ProgressBar {
    config: ProgressBarConfig,
    state: ProgressState,
}

impl ProgressBar {
    /// Create new progress bar with default configuration
    pub fn new() -> Self {
        Self {
            config: ProgressBarConfig::default(),
            state: ProgressState::new(),
        }
    }

    /// Create progress bar with custom configuration
    pub fn with_config(config: ProgressBarConfig) -> Self {
        Self {
            config,
            state: ProgressState::new(),
        }
    }

    /// Create horizontal progress bar with custom styling
    pub fn horizontal(fill_color: Color, background_color: Color) -> Self {
        Self {
            config: ProgressBarConfig {
                orientation: ProgressBarOrientation::Horizontal,
                fill_color,
                background_color,
                ..Default::default()
            },
            state: ProgressState::new(),
        }
    }

    /// Create vertical progress bar with custom styling
    pub fn vertical(fill_color: Color, background_color: Color) -> Self {
        Self {
            config: ProgressBarConfig {
                orientation: ProgressBarOrientation::Vertical,
                fill_color,
                background_color,
                ..Default::default()
            },
            state: ProgressState::new(),
        }
    }

    /// Render progress bar within specified bounds
    pub fn render(&mut self, bounds: &Bounds) -> Vec<String> {
        // Update animation if enabled
        if self.config.animated {
            let frame_duration = Duration::from_secs_f64(1.0 / self.config.animation_speed);
            if self.state.last_update.elapsed() > frame_duration {
                self.state.update_animation();
            }
        }

        match self.config.orientation {
            ProgressBarOrientation::Horizontal => self.render_horizontal(bounds),
            ProgressBarOrientation::Vertical => self.render_vertical(bounds),
        }
    }

    /// Render horizontal progress bar
    fn render_horizontal(&self, bounds: &Bounds) -> Vec<String> {
        let mut lines = Vec::new();
        let width = bounds.width().max(self.config.min_width);
        let height = bounds.height();

        // Calculate progress bar dimensions
        let border_width = if self.config.border_chars.is_some() { 2 } else { 0 };
        let bar_width = width.saturating_sub(border_width);
        let filled_width = ((bar_width as f64) * self.state.progress).round() as usize;
        let remaining_width = bar_width.saturating_sub(filled_width);

        // Generate progress bar lines
        for row in 0..height {
            let line = if row == 0 && self.config.show_progress_text {
                self.generate_progress_text_line(width)
            } else if row == height.saturating_sub(1) && self.config.show_percentage {
                self.generate_percentage_line(width)
            } else {
                self.generate_bar_line(filled_width, remaining_width, &border_width)
            };
            lines.push(line);
        }

        // Ensure we have at least one line for the progress bar
        if lines.is_empty() {
            lines.push(self.generate_bar_line(filled_width, remaining_width, &border_width));
        }

        lines
    }

    /// Render vertical progress bar
    fn render_vertical(&self, bounds: &Bounds) -> Vec<String> {
        let mut lines = Vec::new();
        let width = bounds.width();
        let height = bounds.height().max(self.config.min_height);

        // Calculate progress bar dimensions
        let border_height = if self.config.border_chars.is_some() { 2 } else { 0 };
        let bar_height = height.saturating_sub(border_height);
        let filled_height = ((bar_height as f64) * self.state.progress).round() as usize;

        // Generate vertical progress bar lines
        for row in 0..height {
            let line = if row == 0 && self.config.show_progress_text {
                self.generate_progress_text_line(width)
            } else if row == height.saturating_sub(1) && self.config.show_percentage {
                self.generate_percentage_line(width)
            } else {
                self.generate_vertical_bar_line(row, filled_height, bar_height, width, &border_height)
            };
            lines.push(line);
        }

        lines
    }

    /// Generate progress text line
    fn generate_progress_text_line(&self, width: usize) -> String {
        let text = if let Some(ref progress_text) = self.state.text {
            let percentage = (self.state.progress * 100.0).round() as u32;
            self.config
                .progress_text_template
                .replace("{text}", progress_text)
                .replace("{percentage}", &percentage.to_string())
        } else if self.config.show_percentage {
            format!("{}%", (self.state.progress * 100.0).round() as u32)
        } else {
            "Progress".to_string()
        };

        // Center the text within the width
        self.center_text(&text, width)
    }

    /// Generate percentage line
    fn generate_percentage_line(&self, width: usize) -> String {
        let percentage = (self.state.progress * 100.0).round() as u32;
        let percentage_text = format!("{}%", percentage);
        self.center_text(&percentage_text, width)
    }

    /// Generate horizontal bar line
    fn generate_bar_line(&self, filled_width: usize, remaining_width: usize, border_width: &usize) -> String {
        let mut line = String::new();

        // Add left border
        if let Some(ref borders) = self.config.border_chars {
            line.push(borders.left);
        }

        // Add filled portion with animation
        let fill_char = if self.config.animated {
            self.get_animated_fill_char()
        } else {
            self.config.fill_char
        };

        for _ in 0..filled_width {
            line.push(fill_char);
        }

        // Add remaining portion
        for _ in 0..remaining_width {
            line.push(self.config.background_char);
        }

        // Add right border
        if let Some(ref borders) = self.config.border_chars {
            line.push(borders.right);
        }

        line
    }

    /// Generate vertical bar line
    fn generate_vertical_bar_line(
        &self,
        row: usize,
        filled_height: usize,
        bar_height: usize,
        width: usize,
        border_height: &usize,
    ) -> String {
        let mut line = String::new();

        // Determine if this row should be filled (fill from bottom)
        let effective_row = if *border_height > 0 { row.saturating_sub(1) } else { row };
        let is_filled = effective_row >= bar_height.saturating_sub(filled_height);

        // Generate vertical bar character
        let bar_char = if is_filled {
            if self.config.animated {
                self.get_animated_fill_char()
            } else {
                self.config.fill_char
            }
        } else {
            self.config.background_char
        };

        // Add borders and fill
        if let Some(ref borders) = self.config.border_chars {
            if row == 0 || row == bar_height + border_height - 1 {
                // Top or bottom border
                line = borders.top.to_string().repeat(width);
            } else {
                // Side borders with fill
                line.push(borders.left);
                for _ in 1..width.saturating_sub(1) {
                    line.push(bar_char);
                }
                line.push(borders.right);
            }
        } else {
            // No borders, just fill
            line = bar_char.to_string().repeat(width);
        }

        line
    }

    /// Get animated fill character
    fn get_animated_fill_char(&self) -> char {
        const ANIMATION_CHARS: &[char] = &['█', '▉', '▊', '▋', '▌', '▍', '▎', '▏'];
        let index = self.state.animation_frame % ANIMATION_CHARS.len();
        ANIMATION_CHARS[index]
    }

    /// Center text within specified width
    fn center_text(&self, text: &str, width: usize) -> String {
        if text.len() >= width {
            text.chars().take(width).collect()
        } else {
            let padding = (width - text.len()) / 2;
            let right_padding = width - text.len() - padding;
            format!(
                "{}{}{}",
                " ".repeat(padding),
                text,
                " ".repeat(right_padding)
            )
        }
    }

    /// Update progress value (0.0 - 1.0)
    pub fn set_progress(&mut self, progress: f64) {
        self.state.set_progress(progress);
    }

    /// Get current progress value
    pub fn get_progress(&self) -> f64 {
        self.state.progress
    }

    /// Set progress text
    pub fn set_text(&mut self, text: String) {
        self.state.set_text(text);
    }

    /// Clear progress text
    pub fn clear_text(&mut self) {
        self.state.clear_text();
    }

    /// Get elapsed time since creation
    pub fn elapsed(&self) -> Duration {
        self.state.elapsed()
    }

    /// Check if progress bar is complete
    pub fn is_complete(&self) -> bool {
        self.state.progress >= 1.0
    }

    /// Reset progress bar to initial state
    pub fn reset(&mut self) {
        self.state = ProgressState::new();
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ProgressBarConfig {
        &self.config
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ProgressBarConfig) {
        self.config = config;
    }

    /// Get current state
    pub fn get_state(&self) -> &ProgressState {
        &self.state
    }
}

impl Default for ProgressBar {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bounds() -> Bounds {
        Bounds::new(0, 0, 20, 3)
    }

    #[test]
    fn test_progress_bar_creation() {
        let progress_bar = ProgressBar::new();
        assert_eq!(progress_bar.config.orientation, ProgressBarOrientation::Horizontal);
        assert_eq!(progress_bar.config.fill_char, '█');
        assert_eq!(progress_bar.state.progress, 0.0);
    }

    #[test]
    fn test_progress_bar_with_config() {
        let config = ProgressBarConfig {
            fill_color: Color::Blue,
            background_color: Color::Red,
            show_percentage: false,
            ..Default::default()
        };

        let progress_bar = ProgressBar::with_config(config.clone());
        assert_eq!(progress_bar.config.fill_color, Color::Blue);
        assert_eq!(progress_bar.config.background_color, Color::Red);
        assert!(!progress_bar.config.show_percentage);
    }

    #[test]
    fn test_horizontal_progress_bar() {
        let progress_bar = ProgressBar::horizontal(Color::Green, Color::DarkGrey);
        assert_eq!(progress_bar.config.orientation, ProgressBarOrientation::Horizontal);
        assert_eq!(progress_bar.config.fill_color, Color::Green);
        assert_eq!(progress_bar.config.background_color, Color::DarkGrey);
    }

    #[test]
    fn test_vertical_progress_bar() {
        let progress_bar = ProgressBar::vertical(Color::Yellow, Color::Black);
        assert_eq!(progress_bar.config.orientation, ProgressBarOrientation::Vertical);
        assert_eq!(progress_bar.config.fill_color, Color::Yellow);
        assert_eq!(progress_bar.config.background_color, Color::Black);
    }

    #[test]
    fn test_progress_updates() {
        let mut progress_bar = ProgressBar::new();
        
        assert_eq!(progress_bar.get_progress(), 0.0);
        
        progress_bar.set_progress(0.5);
        assert_eq!(progress_bar.get_progress(), 0.5);
        
        progress_bar.set_progress(1.0);
        assert_eq!(progress_bar.get_progress(), 1.0);
        assert!(progress_bar.is_complete());
    }

    #[test]
    fn test_progress_clamping() {
        let mut progress_bar = ProgressBar::new();
        
        // Test values beyond valid range
        progress_bar.set_progress(-0.5);
        assert_eq!(progress_bar.get_progress(), 0.0);
        
        progress_bar.set_progress(1.5);
        assert_eq!(progress_bar.get_progress(), 1.0);
    }

    #[test]
    fn test_progress_text() {
        let mut progress_bar = ProgressBar::new();
        
        progress_bar.set_text("Loading files...".to_string());
        assert_eq!(progress_bar.state.text, Some("Loading files...".to_string()));
        
        progress_bar.clear_text();
        assert_eq!(progress_bar.state.text, None);
    }

    #[test]
    fn test_horizontal_rendering() {
        let mut progress_bar = ProgressBar::new();
        progress_bar.set_progress(0.5);
        
        let bounds = create_test_bounds();
        let lines = progress_bar.render(&bounds);
        
        assert!(!lines.is_empty());
        
        // Check that rendered lines fit within bounds
        for line in &lines {
            assert!(line.chars().count() <= bounds.width());
        }
        assert!(lines.len() <= bounds.height());
    }

    #[test]
    fn test_vertical_rendering() {
        let mut progress_bar = ProgressBar::vertical(Color::Blue, Color::DarkGrey);
        progress_bar.set_progress(0.3);
        
        let bounds = create_test_bounds();
        let lines = progress_bar.render(&bounds);
        
        assert!(!lines.is_empty());
        
        // Check that rendered lines fit within bounds
        for line in &lines {
            assert!(line.chars().count() <= bounds.width());
        }
        assert!(lines.len() <= bounds.height());
    }

    #[test]
    fn test_progress_bar_reset() {
        let mut progress_bar = ProgressBar::new();
        progress_bar.set_progress(0.7);
        progress_bar.set_text("Almost done...".to_string());
        
        assert_eq!(progress_bar.get_progress(), 0.7);
        assert!(progress_bar.state.text.is_some());
        
        progress_bar.reset();
        
        assert_eq!(progress_bar.get_progress(), 0.0);
        assert!(progress_bar.state.text.is_none());
        assert!(!progress_bar.is_complete());
    }

    #[test]
    fn test_text_centering() {
        let progress_bar = ProgressBar::new();
        
        let centered = progress_bar.center_text("Test", 10);
        assert_eq!(centered.len(), 10);
        assert!(centered.contains("Test"));
        
        let long_text = progress_bar.center_text("This is a very long text", 10);
        assert_eq!(long_text.len(), 10);
        assert_eq!(&long_text, "This is a ");
    }

    #[test]
    fn test_animated_fill_char() {
        let progress_bar = ProgressBar::new();
        
        // Test that animation characters are valid
        let anim_char = progress_bar.get_animated_fill_char();
        assert!(anim_char != '\0');
    }

    #[test]
    fn test_config_update() {
        let mut progress_bar = ProgressBar::new();
        let original_color = progress_bar.config.fill_color;
        
        let mut new_config = progress_bar.config.clone();
        new_config.fill_color = Color::Red;
        
        progress_bar.update_config(new_config);
        
        assert_ne!(progress_bar.config.fill_color, original_color);
        assert_eq!(progress_bar.config.fill_color, Color::Red);
    }

    #[test]
    fn test_progress_state_methods() {
        let mut state = ProgressState::new();
        
        assert_eq!(state.progress, 0.0);
        assert!(state.text.is_none());
        
        state.set_progress(0.8);
        assert_eq!(state.progress, 0.8);
        
        state.set_text("Processing...".to_string());
        assert_eq!(state.text, Some("Processing...".to_string()));
        
        state.clear_text();
        assert!(state.text.is_none());
        
        // Test elapsed time
        let elapsed = state.elapsed();
        assert!(elapsed.as_millis() >= 0);
    }
}