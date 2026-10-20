//! The pieces a terminal screen is made of: cells, colors, cursor, modes and mouse events.

#[derive(Debug, Clone, PartialEq)]
pub enum MouseEventType {
    Press,
    Release,
    Motion,
    Wheel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MouseModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// A color a program set: an entry of the 256-color palette or a 24-bit color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TermColor {
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl TermColor {
    /// The SGR parameters selecting this color as the foreground (`background` false) or the
    /// background.
    pub(crate) fn sgr(self, background: bool) -> String {
        let (base, bright, extended) = if background {
            (40, 100, 48)
        } else {
            (30, 90, 38)
        };
        match self {
            TermColor::Indexed(n) if n < 8 => format!("{}", base + u16::from(n)),
            TermColor::Indexed(n) if n < 16 => format!("{}", bright + u16::from(n - 8)),
            TermColor::Indexed(n) => format!("{};5;{}", extended, n),
            TermColor::Rgb(r, g, b) => format!("{};2;{};{};{}", extended, r, g, b),
        }
    }
}

/// The character in the right half of a two-column character's cells.
pub const WIDE_CONTINUATION: char = '\0';

#[derive(Debug, Clone, PartialEq)]
/// F0306: Extended TerminalCell with comprehensive SGR attribute support
pub struct TerminalCell {
    pub character: char,
    pub fg_color: Option<TermColor>,
    pub bg_color: Option<TermColor>,

    // Basic text attributes
    pub bold: bool,
    pub dim: bool, // F0306: Faint/dim text
    pub italic: bool,
    pub underline: bool,
    pub double_underline: bool, // F0306: Double underline
    pub blink: bool,            // F0306: Blinking text
    pub reverse: bool,
    pub hidden: bool, // F0306: Concealed/hidden text
    pub strikethrough: bool,

    // Extended attributes
    pub font_id: Option<u8>, // F0306: Alternative font selection (0-9)
    pub framed: bool,        // F0306: Framed text
    pub encircled: bool,     // F0306: Encircled text
    pub overlined: bool,     // F0306: Overlined text

    // Ideogram attributes (rarely used)
    pub ideogram_underline: bool,        // F0306: Ideogram underline
    pub ideogram_double_underline: bool, // F0306: Ideogram double underline
    pub ideogram_overline: bool,         // F0306: Ideogram overline
    pub ideogram_double_overline: bool,  // F0306: Ideogram double overline
    pub ideogram_stress: bool,           // F0306: Ideogram stress marking
}

impl Default for TerminalCell {
    fn default() -> Self {
        TerminalCell {
            character: ' ',
            fg_color: None,
            bg_color: None,

            // Basic text attributes
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            double_underline: false,
            blink: false,
            reverse: false,
            hidden: false,
            strikethrough: false,

            // Extended attributes
            font_id: None,
            framed: false,
            encircled: false,
            overlined: false,

            // Ideogram attributes
            ideogram_underline: false,
            ideogram_double_underline: false,
            ideogram_overline: false,
            ideogram_double_overline: false,
            ideogram_stress: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CursorState {
    pub x: usize,
    pub y: usize,
    pub visible: bool,
    pub style: CursorStyle,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum CursorStyle {
    #[default]
    Block,
    Underline,
    Bar,
    BlinkingBlock,
    BlinkingUnderline,
    BlinkingBar,
}

/// F0314: Line attributes for double-width and double-height lines
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LineAttribute {
    #[default]
    Normal, // Standard single-width, single-height line
    DoubleWidth,        // DECDWL - Double-width line
    DoubleHeightTop,    // DECDHL - Double-height line (top half)
    DoubleHeightBottom, // DECDHL - Double-height line (bottom half)
}

/// F0306: SGR sequence state tracking for extended color support
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SGRState {
    pub expecting_extended_fg: bool, // Expecting 38;5;n or 38;2;r;g;b sequence
    pub expecting_extended_bg: bool, // Expecting 48;5;n or 48;2;r;g;b sequence
    pub extended_fg_mode: Option<u8>, // 5 for palette, 2 for RGB
    pub extended_bg_mode: Option<u8>, // 5 for palette, 2 for RGB
    pub rgb_components: Vec<u8>,     // RGB components being collected
    pub palette_index: Option<u8>,   // 256-color palette index
}

#[derive(Debug, Clone)]
pub struct TerminalMode {
    pub cursor_key_mode: bool, // DECCKM - Application cursor keys
    pub keypad_mode: bool,     // DECPAM - Application keypad
    pub auto_wrap_mode: bool,  // DECAWM - Auto wrap
    pub origin_mode: bool,     // DECOM - Origin mode
    pub insert_mode: bool,     // IRM - Insert/Replace mode
    pub local_echo_mode: bool, // LNM - Linefeed/Newline mode
    pub reverse_video: bool,   // DECSCNM - Screen mode (reverse)
    // F0310: Mouse Protocol Support
    pub mouse_x10_mode: bool,          // 1000 - X10 mouse reporting
    pub mouse_normal_mode: bool,       // 1002 - Normal mouse reporting
    pub mouse_button_event_mode: bool, // 1002 - Button event tracking
    pub mouse_any_event_mode: bool,    // 1003 - Any event tracking
    pub mouse_sgr_mode: bool,          // 1006 - SGR extended reporting
    pub bracketed_paste_mode: bool,    // 2004 - Bracketed paste mode
}

impl Default for TerminalMode {
    fn default() -> Self {
        TerminalMode {
            cursor_key_mode: false,
            keypad_mode: false,
            auto_wrap_mode: true,
            origin_mode: false,
            insert_mode: false,
            local_echo_mode: false,
            reverse_video: false,
            // F0310: Mouse Protocol Support - all disabled by default
            mouse_x10_mode: false,
            mouse_normal_mode: false,
            mouse_button_event_mode: false,
            mouse_any_event_mode: false,
            mouse_sgr_mode: false,
            bracketed_paste_mode: false,
        }
    }
}

#[derive(Debug, Clone)]
/// F0307: Scrolling region for DECSTBM (Set Top and Bottom Margins)
#[derive(Default)]
pub struct ScrollRegion {
    pub top: usize,    // Top margin (0-based, inclusive)
    pub bottom: usize, // Bottom margin (0-based, inclusive)
}
