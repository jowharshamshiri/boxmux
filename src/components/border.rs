use crate::model::muxbox::MuxBox;
use crate::pty_manager::PtyManager;
use crate::{Bounds, Cell, ScreenBuffer};
use crate::components::ComponentDimensions;

/// Border component for rendering box borders with various styles and states
pub struct Border {
    pub style: BorderStyle,
    pub color: Option<u8>,
    pub bg_color: Option<u8>,
    pub resize_enabled: bool,
    pub pty_enabled: bool,
    pub error_state: bool,
    pub dead_state: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum BorderStyle {
    #[default]
    Single,
    Double,
    Thick,
    Rounded,
    Custom(BorderCharSet),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BorderCharSet {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub resize_knob: char,
}

impl BorderStyle {
    pub fn get_charset(&self) -> BorderCharSet {
        match self {
            BorderStyle::Single => BorderCharSet {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
                horizontal: '─',
                vertical: '│',
                resize_knob: '⋱',
            },
            BorderStyle::Double => BorderCharSet {
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
                horizontal: '═',
                vertical: '║',
                resize_knob: '⋱',
            },
            BorderStyle::Thick => BorderCharSet {
                top_left: '┏',
                top_right: '┓',
                bottom_left: '┗',
                bottom_right: '┛',
                horizontal: '━',
                vertical: '┃',
                resize_knob: '⋱',
            },
            BorderStyle::Rounded => BorderCharSet {
                top_left: '╭',
                top_right: '╮',
                bottom_left: '╰',
                bottom_right: '╯',
                horizontal: '─',
                vertical: '│',
                resize_knob: '⋱',
            },
            BorderStyle::Custom(charset) => charset.clone(),
        }
    }
}

impl Border {
    /// Create a new Border component from MuxBox configuration
    pub fn from_muxbox(muxbox: &MuxBox, pty_manager: &PtyManager, locked: bool) -> Self {
        let error_state = pty_manager.is_pty_in_error_state(&muxbox.id);
        let dead_state = pty_manager.is_pty_dead(&muxbox.id);

        // Parse color strings to u8 values if present
        let border_color_u8 = muxbox
            .border_color
            .as_ref()
            .and_then(|s| s.parse::<u8>().ok());
        let bg_color_u8 = muxbox.bg_color.as_ref().and_then(|s| s.parse::<u8>().ok());

        Self {
            style: BorderStyle::Single, // TODO: Make configurable from muxbox
            color: border_color_u8,
            bg_color: bg_color_u8,
            resize_enabled: !locked,
            pty_enabled: matches!(muxbox.execution_mode, crate::ExecutionMode::Pty),
            error_state,
            dead_state,
        }
    }

    /// Create a Border with custom configuration
    pub fn new(style: BorderStyle, color: Option<u8>, bg_color: Option<u8>) -> Self {
        Self {
            style,
            color,
            bg_color,
            resize_enabled: false,
            pty_enabled: false,
            error_state: false,
            dead_state: false,
        }
    }

    /// Set resize knob visibility
    pub fn with_resize_enabled(mut self, enabled: bool) -> Self {
        self.resize_enabled = enabled;
        self
    }

    /// Set PTY state for color determination
    pub fn with_pty_state(
        mut self,
        pty_enabled: bool,
        error_state: bool,
        dead_state: bool,
    ) -> Self {
        self.pty_enabled = pty_enabled;
        self.error_state = error_state;
        self.dead_state = dead_state;
        self
    }

    /// Draw the border to the screen buffer
    pub fn draw(&self, bounds: &Bounds, buffer: &mut ScreenBuffer) {
        let charset = self.style.get_charset();
        let border_color = self.calculate_border_color();
        let bg_color = self
            .bg_color
            .map_or_else(|| "0".to_string(), |c| c.to_string());

        // Draw corners
        buffer.update(
            bounds.left(),
            bounds.top(),
            Cell {
                ch: charset.top_left,
                fg_color: border_color.clone(),
                bg_color: bg_color.clone(),
            },
        );

        buffer.update(
            bounds.right(),
            bounds.top(),
            Cell {
                ch: charset.top_right,
                fg_color: border_color.clone(),
                bg_color: bg_color.clone(),
            },
        );

        buffer.update(
            bounds.left(),
            bounds.bottom(),
            Cell {
                ch: charset.bottom_left,
                fg_color: border_color.clone(),
                bg_color: bg_color.clone(),
            },
        );

        // Bottom right corner - resize knob or normal corner
        buffer.update(
            bounds.right(),
            bounds.bottom(),
            Cell {
                ch: if self.resize_enabled {
                    charset.resize_knob
                } else {
                    charset.bottom_right
                },
                fg_color: border_color.clone(),
                bg_color: bg_color.clone(),
            },
        );

        // Draw horizontal edges
        let component_dims = ComponentDimensions::new(*bounds);
        let inside_border = component_dims.inside_border_bounds();
        for x in inside_border.left()..bounds.right() {
            buffer.update(
                x,
                bounds.top(),
                Cell {
                    ch: charset.horizontal,
                    fg_color: border_color.clone(),
                    bg_color: bg_color.clone(),
                },
            );
            buffer.update(
                x,
                bounds.bottom(),
                Cell {
                    ch: charset.horizontal,
                    fg_color: border_color.clone(),
                    bg_color: bg_color.clone(),
                },
            );
        }

        // Draw vertical edges
        for y in inside_border.top()..bounds.bottom() {
            buffer.update(
                bounds.left(),
                y,
                Cell {
                    ch: charset.vertical,
                    fg_color: border_color.clone(),
                    bg_color: bg_color.clone(),
                },
            );
            buffer.update(
                bounds.right(),
                y,
                Cell {
                    ch: charset.vertical,
                    fg_color: border_color.clone(),
                    bg_color: bg_color.clone(),
                },
            );
        }
    }

    /// Calculate border color based on state - returns color string
    pub fn calculate_border_color(&self) -> String {
        if self.pty_enabled {
            "14".to_string() // Bright Cyan for PTY
        } else if self.dead_state || self.error_state {
            "9".to_string() // Bright Red for errors
        } else {
            self.color
                .map_or_else(|| "7".to_string(), |c| c.to_string()) // Default or configured color
        }
    }

    /// Check if coordinates are within the resize knob area
    pub fn is_resize_knob_area(&self, bounds: &Bounds, x: u16, y: u16) -> bool {
        if !self.resize_enabled {
            return false;
        }

        // Resize knob is at bottom-right corner
        x as usize == bounds.right() && y as usize == bounds.bottom()
    }

    /// Check if coordinates are on the border
    pub fn is_border_area(&self, bounds: &Bounds, x: u16, y: u16) -> bool {
        let on_top = y as usize == bounds.top()
            && x as usize >= bounds.left()
            && x as usize <= bounds.right();
        let on_bottom = y as usize == bounds.bottom()
            && x as usize >= bounds.left()
            && x as usize <= bounds.right();
        let on_left = x as usize == bounds.left()
            && y as usize >= bounds.top()
            && y as usize <= bounds.bottom();
        let on_right = x as usize == bounds.right()
            && y as usize >= bounds.top()
            && y as usize <= bounds.bottom();

        on_top || on_bottom || on_left || on_right
    }

    /// Check if coordinates are on the top border (for dragging)
    pub fn is_top_border_area(&self, bounds: &Bounds, x: u16, y: u16) -> bool {
        y as usize == bounds.top() && x as usize >= bounds.left() && x as usize <= bounds.right()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests use direct crate references

    #[test]
    fn test_border_creation() {
        let border = Border::new(BorderStyle::Single, Some(7), Some(0));
        assert_eq!(border.style, BorderStyle::Single);
        assert_eq!(border.color, Some(7));
        assert_eq!(border.bg_color, Some(0));
    }

    #[test]
    fn test_border_style_charset() {
        let single = BorderStyle::Single.get_charset();
        assert_eq!(single.top_left, '┌');
        assert_eq!(single.horizontal, '─');

        let double = BorderStyle::Double.get_charset();
        assert_eq!(double.top_left, '╔');
        assert_eq!(double.horizontal, '═');
    }

    #[test]
    fn test_resize_knob_detection() {
        let border = Border::new(BorderStyle::Single, None, None).with_resize_enabled(true);

        let bounds = Bounds::new(0, 0, 10, 5);

        assert!(border.is_resize_knob_area(&bounds, 10, 5));
        assert!(!border.is_resize_knob_area(&bounds, 0, 0));
        assert!(!border.is_resize_knob_area(&bounds, 5, 2));
    }

    #[test]
    fn test_border_area_detection() {
        let border = Border::new(BorderStyle::Single, None, None);
        let bounds = Bounds::new(2, 1, 8, 4);

        // Corners
        assert!(border.is_border_area(&bounds, 2, 1)); // top-left
        assert!(border.is_border_area(&bounds, 8, 1)); // top-right
        assert!(border.is_border_area(&bounds, 2, 4)); // bottom-left
        assert!(border.is_border_area(&bounds, 8, 4)); // bottom-right

        // Edges
        assert!(border.is_border_area(&bounds, 5, 1)); // top edge
        assert!(border.is_border_area(&bounds, 5, 4)); // bottom edge
        assert!(border.is_border_area(&bounds, 2, 2)); // left edge
        assert!(border.is_border_area(&bounds, 8, 2)); // right edge

        // Interior (should not be border)
        assert!(!border.is_border_area(&bounds, 5, 2));
    }
}
