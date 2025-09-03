// F0336: BoxMux Visual Tester - Core testing harness for visual validation workflows
// Orchestrates config loading, input simulation, and output capture

use super::terminal_capture::{TerminalCapture, TerminalFrame};
use crate::model::app::load_app_from_yaml;
use crate::{App, AppContext};
use std::path::Path;
use std::time::{Duration, Instant};

/// F0336: Core testing harness for visual validation workflows
pub struct BoxMuxTester {
    /// Terminal capture system
    capture: TerminalCapture,
    /// Current app instance
    app: Option<App>,
    /// App context for message handling
    app_context: Option<AppContext>,
    /// Simulated input events for processing
    pending_events: Vec<SimulatedEvent>,
    /// Terminal dimensions
    dimensions: (u16, u16),
    /// Test configuration
    config: TestConfig,
}

/// Simulated input events for testing
#[derive(Debug, Clone)]
enum SimulatedEvent {
    KeyPress(crossterm::event::KeyEvent),
    MouseEvent(crossterm::event::MouseEvent),
}

/// F0337: Test configuration management
#[derive(Debug, Clone)]
pub struct TestConfig {
    /// Terminal dimensions for testing
    pub terminal_size: (u16, u16),
    /// Timeout for operations
    pub operation_timeout: Duration,
    /// Whether to capture frame history
    pub capture_history: bool,
    /// Maximum frames to keep
    pub max_frames: usize,
    /// Frame capture interval for animations
    pub frame_interval: Duration,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            terminal_size: (80, 24),
            operation_timeout: Duration::from_secs(5),
            capture_history: true,
            max_frames: 100,
            frame_interval: Duration::from_millis(16), // ~60 FPS
        }
    }
}

impl BoxMuxTester {
    /// Create new BoxMux tester with default configuration
    pub fn new() -> Self {
        let config = TestConfig::default();
        let dimensions = config.terminal_size;

        Self {
            capture: TerminalCapture::new(dimensions.0, dimensions.1),
            app: None,
            app_context: None,
            pending_events: Vec::new(),
            dimensions,
            config,
        }
    }

    /// Create new BoxMux tester with custom configuration
    pub fn with_config(config: TestConfig) -> Self {
        let dimensions = config.terminal_size;

        Self {
            capture: TerminalCapture::new(dimensions.0, dimensions.1),
            app: None,
            app_context: None,
            pending_events: Vec::new(),
            dimensions,
            config,
        }
    }

    /// F0337: Load configuration from YAML file
    pub fn load_config<P: AsRef<Path>>(&mut self, yaml_path: P) -> Result<&mut Self, TestError> {
        let yaml_content = std::fs::read_to_string(yaml_path.as_ref())
            .map_err(|e| TestError::ConfigLoad(format!("Failed to read YAML: {}", e)))?;

        self.load_config_from_string(&yaml_content)
    }

    /// F0337: Load configuration from YAML string
    pub fn load_config_from_string(&mut self, yaml_content: &str) -> Result<&mut Self, TestError> {
        // Write YAML content to temporary file
        use std::io::Write;
        use tempfile::NamedTempFile;

        let mut temp_file = NamedTempFile::new()
            .map_err(|e| TestError::ConfigLoad(format!("Failed to create temp file: {}", e)))?;

        temp_file
            .write_all(yaml_content.as_bytes())
            .map_err(|e| TestError::ConfigLoad(format!("Failed to write temp file: {}", e)))?;

        // Load app from temporary file
        let app = load_app_from_yaml(temp_file.path().to_str().unwrap())
            .map_err(|e| TestError::ConfigLoad(format!("Failed to parse YAML: {}", e)))?;

        // Create app context with default config
        let config = crate::Config::default();
        let app_context = AppContext::new(app.clone(), config);

        self.app = Some(app);
        self.app_context = Some(app_context);

        Ok(self)
    }


    /// F0336: Simulate keyboard input
    pub fn send_key(&mut self, key: crossterm::event::KeyCode) -> Result<&mut Self, TestError> {
        self.send_key_with_modifiers(key, crossterm::event::KeyModifiers::empty())
    }

    /// F0336: Simulate keyboard input with modifiers
    pub fn send_key_with_modifiers(
        &mut self,
        key: crossterm::event::KeyCode,
        modifiers: crossterm::event::KeyModifiers,
    ) -> Result<&mut Self, TestError> {
        // Create KeyEvent and add to pending events
        let key_event = crossterm::event::KeyEvent {
            code: key,
            modifiers,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::empty(),
        };
        
        self.pending_events.push(SimulatedEvent::KeyPress(key_event));
        
        // Process the event immediately for testing
        self.process_pending_events()?;

        Ok(self)
    }

    /// F0336: Simulate mouse click
    pub fn click_at(&mut self, x: u16, y: u16) -> Result<&mut Self, TestError> {
        // Create MouseEvent and add to pending events
        let mouse_event = crossterm::event::MouseEvent {
            kind: crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left),
            column: x,
            row: y,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        
        self.pending_events.push(SimulatedEvent::MouseEvent(mouse_event));
        
        // Process the event immediately for testing
        self.process_pending_events()?;

        Ok(self)
    }

    /// F0336: Type text string
    pub fn type_text(&mut self, text: &str) -> Result<&mut Self, TestError> {
        for ch in text.chars() {
            let key = match ch {
                '\n' => crossterm::event::KeyCode::Enter,
                '\t' => crossterm::event::KeyCode::Tab,
                ' ' => crossterm::event::KeyCode::Char(' '),
                c => crossterm::event::KeyCode::Char(c),
            };

            self.send_key(key)?;
        }

        Ok(self)
    }

    /// F0336: Simulate mouse drag operation
    pub fn drag_from_to(&mut self, from_x: u16, from_y: u16, to_x: u16, to_y: u16) -> Result<&mut Self, TestError> {
        // Send mouse down
        self.send_mouse_event(crossterm::event::MouseEventKind::Down(crossterm::event::MouseButton::Left), from_x, from_y)?;
        
        // Send drag event
        self.send_mouse_event(crossterm::event::MouseEventKind::Drag(crossterm::event::MouseButton::Left), to_x, to_y)?;
        
        // Send mouse up
        self.send_mouse_event(crossterm::event::MouseEventKind::Up(crossterm::event::MouseButton::Left), to_x, to_y)?;
        
        Ok(self)
    }

    /// F0336: Send specific mouse event
    pub fn send_mouse_event(&mut self, kind: crossterm::event::MouseEventKind, x: u16, y: u16) -> Result<&mut Self, TestError> {
        let mouse_event = crossterm::event::MouseEvent {
            kind,
            column: x,
            row: y,
            modifiers: crossterm::event::KeyModifiers::empty(),
        };
        
        self.pending_events.push(SimulatedEvent::MouseEvent(mouse_event));
        
        // Process the event immediately for testing
        self.process_pending_events()?;

        Ok(self)
    }

    /// F0336: Wait for specific condition to be met
    pub fn wait_until<F>(&mut self, mut condition: F, timeout: Duration) -> Result<&mut Self, TestError> 
    where 
        F: FnMut(&TerminalFrame) -> bool,
    {
        let start_time = Instant::now();
        
        while start_time.elapsed() < timeout {
            if let Ok(frame) = self.wait_for_frame_with_timeout(Duration::from_millis(100)) {
                if condition(frame) {
                    return Ok(self);
                }
            }
            std::thread::sleep(Duration::from_millis(50));
        }
        
        Err(TestError::Timeout(format!("Condition not met within {:?}", timeout)))
    }

    /// F0336: Assert that frame matches expected state after interaction
    pub fn assert_interaction_result<F>(&mut self, interaction: F, assertion: Box<dyn Fn(&TerminalFrame) -> Result<(), String>>) -> Result<&mut Self, TestError>
    where
        F: FnOnce(&mut Self) -> Result<&mut Self, TestError>,
    {
        // Perform interaction
        interaction(self)?;
        
        // Capture frame
        let frame = self.wait_for_frame()?;
        
        // Assert result
        assertion(frame).map_err(|e| TestError::MessageSend(format!("Assertion failed: {}", e)))?;
        
        Ok(self)
    }

    /// Process pending events by simulating user interactions
    fn process_pending_events(&mut self) -> Result<(), TestError> {
        // For now, we simulate the effects of user interactions
        // In a full implementation, this would integrate with the input handling system
        
        // Clear pending events (they've been "processed")
        let events = std::mem::take(&mut self.pending_events);
        
        // Log the simulated interactions
        for event in &events {
            match event {
                SimulatedEvent::KeyPress(key_event) => {
                    log::debug!("Simulated key press: {:?}", key_event.code);
                    // Here we could simulate navigation, choice selection, etc.
                    self.simulate_key_effect(key_event)?;
                }
                SimulatedEvent::MouseEvent(mouse_event) => {
                    log::debug!("Simulated mouse event: {:?} at ({}, {})", 
                        mouse_event.kind, mouse_event.column, mouse_event.row);
                    // Here we could simulate mouse clicks on choices, scrollbars, etc.
                    self.simulate_mouse_effect(mouse_event)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Simulate the effect of a key press on the app state
    fn simulate_key_effect(&mut self, _key_event: &crossterm::event::KeyEvent) -> Result<(), TestError> {
        // This would simulate navigation, choice execution, etc.
        // For now, we just allow the visual system to capture the current state
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    }
    
    /// Simulate the effect of a mouse event on the app state
    fn simulate_mouse_effect(&mut self, _mouse_event: &crossterm::event::MouseEvent) -> Result<(), TestError> {
        // This would simulate clicking on choices, dragging scrollbars, etc.
        // For now, we just allow the visual system to capture the current state
        std::thread::sleep(Duration::from_millis(10));
        Ok(())
    }

    /// F0326: Wait for frame and capture current visual state
    pub fn wait_for_frame(&mut self) -> Result<&TerminalFrame, TestError> {
        self.wait_for_frame_with_timeout(self.config.operation_timeout)
    }

    /// F0326: Wait for frame with custom timeout
    pub fn wait_for_frame_with_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<&TerminalFrame, TestError> {
        let _start_time = Instant::now();

        // Process any pending events
        self.process_pending_events()?;

        // Capture current frame from app
        if let Some(app) = &self.app {
            let frame = self.capture.capture_frame(app);
            return Ok(frame);
        }

        Err(TestError::NoApp(
            "No app loaded for frame capture".to_string(),
        ))
    }

    /// F0329: Capture animation sequence
    pub fn capture_animation_sequence(
        &mut self,
        duration: Duration,
        frame_interval: Option<Duration>,
    ) -> Result<Vec<TerminalFrame>, TestError> {
        let interval = frame_interval.unwrap_or(self.config.frame_interval);
        let mut frames = Vec::new();
        let start_time = Instant::now();

        while start_time.elapsed() < duration {
            if let Ok(frame) = self.wait_for_frame_with_timeout(interval) {
                frames.push(frame.clone());
            }

            std::thread::sleep(interval);
        }

        Ok(frames)
    }


    /// F0326: Get current captured frame
    pub fn current_frame(&self) -> Option<&TerminalFrame> {
        self.capture.current_frame()
    }

    /// F0330: Get frame difference count
    pub fn get_frame_diff_count(
        &self,
        frame1_index: isize,
        frame2_index: isize,
    ) -> Result<usize, TestError> {
        let frame1 = self
            .capture
            .get_frame(frame1_index)
            .ok_or_else(|| TestError::FrameAccess(format!("Frame {} not found", frame1_index)))?;
        let frame2 = self
            .capture
            .get_frame(frame2_index)
            .ok_or_else(|| TestError::FrameAccess(format!("Frame {} not found", frame2_index)))?;

        let mut diff_count = 0;

        for (y, (row1, row2)) in frame1.buffer.iter().zip(frame2.buffer.iter()).enumerate() {
            for (x, (cell1, cell2)) in row1.iter().zip(row2.iter()).enumerate() {
                if cell1 != cell2 {
                    diff_count += 1;
                }
            }
        }

        Ok(diff_count)
    }

    /// F0339: Save visual snapshot
    pub fn save_snapshot(&self, name: &str) -> Result<(), TestError> {
        if let Some(frame) = self.current_frame() {
            let snapshot_data = self.serialize_frame(frame);
            let filename = format!("snapshots/{}.snapshot", name);

            // Create snapshots directory if it doesn't exist
            if let Some(parent) = Path::new(&filename).parent() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    TestError::SnapshotSave(format!("Failed to create directory: {}", e))
                })?;
            }

            std::fs::write(&filename, snapshot_data)
                .map_err(|e| TestError::SnapshotSave(format!("Failed to save snapshot: {}", e)))?;

            Ok(())
        } else {
            Err(TestError::SnapshotSave(
                "No frame available for snapshot".to_string(),
            ))
        }
    }

    /// Serialize frame to string format
    fn serialize_frame(&self, frame: &TerminalFrame) -> String {
        let mut output = Vec::new();

        // Add metadata
        output.push(format!("# BoxMux Visual Snapshot"));
        output.push(format!(
            "# Dimensions: {}x{}",
            frame.dimensions.0, frame.dimensions.1
        ));
        output.push(format!(
            "# Cursor: ({}, {}) visible={}",
            frame.cursor.0, frame.cursor.1, frame.cursor_visible
        ));
        output.push(format!("# Timestamp: {:?}", frame.timestamp));
        output.push("".to_string());

        // Add visual content
        for (y, row) in frame.buffer.iter().enumerate() {
            let line: String = row.iter().map(|cell| cell.ch).collect();
            output.push(format!("{:2}|{}", y, line));
        }

        output.join("\n")
    }

    /// Get frame history count
    pub fn frame_count(&self) -> usize {
        self.capture.frame_count()
    }

    /// Clear frame history
    pub fn clear_frames(&mut self) {
        self.capture.clear_frames();
    }

    /// Set terminal dimensions
    pub fn set_dimensions(&mut self, width: u16, height: u16) {
        self.dimensions = (width, height);
        self.capture.set_dimensions(width, height);
        self.config.terminal_size = (width, height);
    }

    /// Get terminal dimensions
    pub fn get_dimensions(&self) -> (u16, u16) {
        self.dimensions
    }
}

impl std::fmt::Debug for BoxMuxTester {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BoxMuxTester")
            .field("dimensions", &self.dimensions)
            .field("config", &self.config)
            .field("app_loaded", &self.app.is_some())
            .field("app_context_loaded", &self.app_context.is_some())
            .field("pending_events_count", &self.pending_events.len())
            .field("frame_count", &self.capture.frame_count())
            .finish()
    }
}

/// F0336: Testing errors
#[derive(Debug, Clone)]
pub enum TestError {
    ConfigLoad(String),
    MessageSend(String),
    FrameAccess(String),
    SnapshotSave(String),
    NoApp(String),
    Timeout(String),
}

impl std::fmt::Display for TestError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TestError::ConfigLoad(msg) => write!(f, "Config load error: {}", msg),
            TestError::MessageSend(msg) => write!(f, "Message send error: {}", msg),
            TestError::FrameAccess(msg) => write!(f, "Frame access error: {}", msg),
            TestError::SnapshotSave(msg) => write!(f, "Snapshot save error: {}", msg),
            TestError::NoApp(msg) => write!(f, "No app error: {}", msg),
            TestError::Timeout(msg) => write!(f, "Timeout error: {}", msg),
        }
    }
}

impl std::error::Error for TestError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boxmux_tester_creation() {
        let tester = BoxMuxTester::new();
        assert_eq!(tester.dimensions, (80, 24));
        assert_eq!(tester.frame_count(), 0);
    }

    #[test]
    fn test_test_config_default() {
        let config = TestConfig::default();
        assert_eq!(config.terminal_size, (80, 24));
        assert_eq!(config.max_frames, 100);
        assert!(config.capture_history);
    }
}
