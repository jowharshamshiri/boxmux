// F0326: Terminal Frame Capture - Core visual testing infrastructure
// Captures exact terminal output character-by-character for validation

use crate::color_utils::should_draw_color;
use crate::{App, Message};
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::time::{Duration, Instant};

/// F0326: Represents a single terminal frame capture
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalFrame {
    /// 2D character buffer representing terminal screen
    pub buffer: Vec<Vec<TerminalCell>>,
    /// Cursor position (x, y)
    pub cursor: (u16, u16),
    /// Whether cursor is visible
    pub cursor_visible: bool,
    /// Frame capture timestamp
    pub timestamp: Instant,
    /// Terminal dimensions
    pub dimensions: (u16, u16),
}

/// F0326: Terminal cell with character and attributes
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalCell {
    /// Character at this position
    pub ch: char,
    /// Foreground color (ANSI color code)
    pub fg_color: Option<u8>,
    /// Background color (ANSI color code)  
    pub bg_color: Option<u8>,
    /// Text attributes (bold, italic, underline, etc.)
    pub attributes: CellAttributes,
}

/// F0333: Terminal cell attributes for color/style testing
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CellAttributes {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub dim: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg_color: None,
            bg_color: None,
            attributes: CellAttributes::default(),
        }
    }
}

/// F0326: Core terminal frame capture system
pub struct TerminalCapture {
    /// Frame history for animation testing
    frames: VecDeque<TerminalFrame>,
    /// Maximum frames to keep in history
    max_history: usize,
    /// Current terminal dimensions
    dimensions: (u16, u16),
}

impl TerminalCapture {
    /// Create new terminal capture system
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            frames: VecDeque::new(),
            max_history: super::MAX_FRAME_HISTORY,
            dimensions: (width, height),
        }
    }

    /// F0326: Capture current frame from App state
    pub fn capture_frame(&mut self, app: &App) -> &TerminalFrame {
        let frame = self.extract_visual_state(app);

        // Add to history, maintaining size limit
        if self.frames.len() >= self.max_history {
            self.frames.pop_front();
        }
        self.frames.push_back(frame);

        self.frames.back().unwrap()
    }

    /// F0326: Extract actual visual state from App
    fn extract_visual_state(&self, app: &App) -> TerminalFrame {
        let (width, height) = self.dimensions;
        let mut buffer = vec![vec![TerminalCell::default(); width as usize]; height as usize];

        // Extract visual state by simulating the actual rendering process
        self.render_app_to_buffer(app, &mut buffer);

        TerminalFrame {
            buffer,
            cursor: self.extract_cursor_position(app),
            cursor_visible: self.extract_cursor_visibility(app),
            timestamp: Instant::now(),
            dimensions: self.dimensions,
        }
    }

    /// F0326: Render App to character buffer (simulates crossterm output)
    fn render_app_to_buffer(&self, app: &App, buffer: &mut Vec<Vec<TerminalCell>>) {
        // This is the core innovation: instead of testing internal state,
        // we capture what would actually be displayed to the user

        // Clear buffer with background
        for row in buffer.iter_mut() {
            for cell in row.iter_mut() {
                *cell = TerminalCell::default();
            }
        }

        // Get active layout if available
        if let Some(active_layout) = app.get_active_layout() {
            // Render each muxbox in the active layout
            if let Some(ref children) = active_layout.children {
                for muxbox in children {
                    self.render_muxbox_to_buffer(muxbox, buffer, app);
                }
            }
        }
    }

    /// F0332: Render individual muxbox to buffer
    fn render_muxbox_to_buffer(
        &self,
        muxbox: &crate::model::muxbox::MuxBox,
        buffer: &mut Vec<Vec<TerminalCell>>,
        app: &App,
    ) {
        // Calculate bounds from position (InputBounds)
        let bounds = self.calculate_muxbox_bounds(muxbox);

        // Render border if present
        if should_draw_color(&muxbox.border_color) {
            self.render_border_to_buffer(&bounds, buffer, muxbox);
        }

        // Render title if present
        if let Some(title) = &muxbox.title {
            self.render_title_to_buffer(title, &bounds, buffer);
        }

        // Render content
        self.render_content_to_buffer(muxbox, &bounds, buffer, app);
    }

    /// Calculate bounds for muxbox from InputBounds
    fn calculate_muxbox_bounds(
        &self,
        muxbox: &crate::model::muxbox::MuxBox,
    ) -> (u16, u16, u16, u16) {
        let (term_width, term_height) = self.dimensions;

        // Parse the InputBounds from the muxbox position
        let position = &muxbox.position;

        // Simple parsing of position strings - in real implementation this would use proper bounds calculation
        let x1 = position.x1.parse::<u16>().unwrap_or(0).min(term_width - 1);
        let y1 = position.y1.parse::<u16>().unwrap_or(0).min(term_height - 1);
        let x2 = position
            .x2
            .parse::<u16>()
            .unwrap_or(x1 + 30)
            .min(term_width);
        let y2 = position
            .y2
            .parse::<u16>()
            .unwrap_or(y1 + 10)
            .min(term_height);

        // Ensure we have at least minimal size
        let width = (x2.saturating_sub(x1)).max(2);
        let height = (y2.saturating_sub(y1)).max(2);

        (x1, y1, width, height)
    }

    /// F0332: Render border characters to buffer
    fn render_border_to_buffer(
        &self,
        bounds: &(u16, u16, u16, u16),
        buffer: &mut Vec<Vec<TerminalCell>>,
        muxbox: &crate::model::muxbox::MuxBox,
    ) {
        let (x, y, width, height) = *bounds;

        if width < 2 || height < 2 {
            return; // Too small for border
        }

        // Draw corners and edges with proper box drawing characters
        let top_left = '┌';
        let top_right = '┐';
        let bottom_left = '└';
        let bottom_right = '┘';
        let horizontal = '─';
        let vertical = '│';

        // Top border
        if y < buffer.len() as u16 {
            let row = &mut buffer[y as usize];
            if x < row.len() as u16 {
                row[x as usize].ch = top_left;
            }
            for col in (x + 1)..(x + width - 1) {
                if col < row.len() as u16 {
                    row[col as usize].ch = horizontal;
                }
            }
            if (x + width - 1) < row.len() as u16 {
                row[(x + width - 1) as usize].ch = top_right;
            }
        }

        // Side borders
        for row_idx in (y + 1)..(y + height - 1) {
            if row_idx < buffer.len() as u16 {
                let row = &mut buffer[row_idx as usize];
                if x < row.len() as u16 {
                    row[x as usize].ch = vertical;
                }
                if (x + width - 1) < row.len() as u16 {
                    row[(x + width - 1) as usize].ch = vertical;
                }
            }
        }

        // Bottom border
        if (y + height - 1) < buffer.len() as u16 {
            let row = &mut buffer[(y + height - 1) as usize];
            if x < row.len() as u16 {
                row[x as usize].ch = bottom_left;
            }
            for col in (x + 1)..(x + width - 1) {
                if col < row.len() as u16 {
                    row[col as usize].ch = horizontal;
                }
            }
            if (x + width - 1) < row.len() as u16 {
                row[(x + width - 1) as usize].ch = bottom_right;
            }
        }
    }

    /// F0332: Render title to buffer
    fn render_title_to_buffer(
        &self,
        title: &str,
        bounds: &(u16, u16, u16, u16),
        buffer: &mut Vec<Vec<TerminalCell>>,
    ) {
        let (x, y, width, _) = *bounds;

        if y >= buffer.len() as u16 {
            return;
        }

        let row = &mut buffer[y as usize];
        let title_start = x + 2; // After border and space

        for (i, ch) in title.chars().enumerate() {
            let col = title_start + i as u16;
            if col >= x + width - 2 {
                // Before right border
                break;
            }
            if col < row.len() as u16 {
                row[col as usize].ch = ch;
            }
        }
    }

    /// F0347: Render content to buffer
    fn render_content_to_buffer(
        &self,
        muxbox: &crate::model::muxbox::MuxBox,
        bounds: &(u16, u16, u16, u16),
        buffer: &mut Vec<Vec<TerminalCell>>,
        app: &App,
    ) {
        let (x, y, width, height) = *bounds;

        // Content area is inside border
        let content_x = x + 1;
        let content_y = y + 1;
        let content_width = width.saturating_sub(2);
        let content_height = height.saturating_sub(2);

        if content_width == 0 || content_height == 0 {
            return;
        }

        // Get actual content from muxbox
        let content = self.extract_muxbox_content(muxbox, app);

        // Render content lines
        for (line_idx, line) in content.lines().enumerate() {
            let row_idx = content_y + line_idx as u16;
            if row_idx >= buffer.len() as u16 || line_idx >= content_height as usize {
                break;
            }

            let row = &mut buffer[row_idx as usize];
            for (char_idx, ch) in line.chars().enumerate() {
                let col_idx = content_x + char_idx as u16;
                if col_idx >= content_x + content_width || col_idx >= row.len() as u16 {
                    break;
                }
                row[col_idx as usize].ch = ch;
            }
        }
    }

    /// Extract actual content from muxbox streams/tabs
    fn extract_muxbox_content(&self, muxbox: &crate::model::muxbox::MuxBox, app: &App) -> String {
        // First check if there are any active streams
        if !muxbox.streams.is_empty() {
            // Look for active stream first, or fall back to first stream
            let stream = muxbox
                .streams
                .iter()
                .find(|(_, s)| s.stream_type == crate::model::common::StreamType::Content)
                .or_else(|| muxbox.streams.iter().next())
                .map(|(_, s)| s);

            if let Some(stream) = stream {
                // Handle different stream types
                match &stream.stream_type {
                    crate::model::common::StreamType::Choices => {
                        // For choice streams, extract the choice content
                        if let Some(choices) = &stream.choices {
                            let mut content = Vec::new();
                            for choice in choices {
                                let display_text = choice.content.as_deref().unwrap_or(&choice.id);
                                content.push(display_text.to_string());
                            }
                            return content.join("\n");
                        }
                    }
                    crate::model::common::StreamType::Content => {
                        // For content streams, use the content field
                        if !stream.content.is_empty() {
                            return stream.content.join("\n");
                        }
                    }
                    _ => {
                        // For other stream types, use content field
                        return stream.content.join("\n");
                    }
                }
            }
        }

        // Fallback: check for direct choices on muxbox (legacy support)
        if let Some(choices) = &muxbox.choices {
            let mut content = Vec::new();
            for choice in choices {
                let display_text = choice.content.as_deref().unwrap_or(&choice.id);
                content.push(display_text.to_string());
            }
            return content.join("\n");
        }

        // Static content fallback
        muxbox.content.as_deref().unwrap_or("").to_string()
    }

    /// F0331: Extract cursor position from App state
    fn extract_cursor_position(&self, app: &App) -> (u16, u16) {
        // For now, return (0,0) - this would need to be enhanced
        // to extract actual cursor position from focused muxbox
        (0, 0)
    }

    /// F0331: Extract cursor visibility from App state  
    fn extract_cursor_visibility(&self, app: &App) -> bool {
        // For now, return true - this would need to be enhanced
        // to extract actual cursor visibility state
        true
    }

    /// Get current frame
    pub fn current_frame(&self) -> Option<&TerminalFrame> {
        self.frames.back()
    }

    /// Get frame by index (0 = oldest, -1 = newest)
    pub fn get_frame(&self, index: isize) -> Option<&TerminalFrame> {
        if index >= 0 {
            self.frames.get(index as usize)
        } else {
            let len = self.frames.len() as isize;
            self.frames.get((len + index) as usize)
        }
    }

    /// Get number of captured frames
    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    /// Clear frame history
    pub fn clear_frames(&mut self) {
        self.frames.clear();
    }

    /// Set terminal dimensions
    pub fn set_dimensions(&mut self, width: u16, height: u16) {
        self.dimensions = (width, height);
    }
}

impl Display for TerminalFrame {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut result = String::new();
        for row in &self.buffer {
            for cell in row {
                result.push(cell.ch);
            }
            result.push('\n');
        }
        // Remove trailing newline
        if result.ends_with('\n') {
            result.pop();
        }
        write!(f, "{}", result)
    }
}
