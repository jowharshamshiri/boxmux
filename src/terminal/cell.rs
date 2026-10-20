//! The pieces a terminal screen is made of: cells, colors, cursor, modes and mouse events.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseEventType {
    Press,
    Release,
    /// The pointer moved; with a button when it is a drag.
    Motion,
    Wheel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
    WheelUp,
    WheelDown,
    WheelLeft,
    WheelRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MouseModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// Which mouse events a program asked for (DEC private modes 9, 1000, 1002 and 1003). Each
/// mode replaces the previous one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseTracking {
    #[default]
    Off,
    /// Mode 9: button presses only, without modifiers.
    X10,
    /// Mode 1000: presses and releases.
    Normal,
    /// Mode 1002: presses, releases and motion while a button is held.
    ButtonEvent,
    /// Mode 1003: presses, releases and all motion.
    AnyEvent,
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

/// One cell of the screen: a character and the attributes it is drawn with.
#[derive(Debug, Clone, PartialEq)]
pub struct TerminalCell {
    pub character: char,
    pub fg_color: Option<TermColor>,
    pub bg_color: Option<TermColor>,
    pub bold: bool,
    pub dim: bool,
    pub italic: bool,
    /// Any underline style: single, double or curly.
    pub underline: bool,
    pub blink: bool,
    pub reverse: bool,
    pub hidden: bool,
    pub strikethrough: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        TerminalCell {
            character: ' ',
            fg_color: None,
            bg_color: None,
            bold: false,
            dim: false,
            italic: false,
            underline: false,
            blink: false,
            reverse: false,
            hidden: false,
            strikethrough: false,
        }
    }
}

impl TerminalCell {
    /// A blank cell with the background of `attributes`: what erasing leaves behind.
    pub fn blank(attributes: &TerminalCell) -> Self {
        TerminalCell {
            bg_color: attributes.bg_color,
            ..TerminalCell::default()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorState {
    pub x: usize,
    pub y: usize,
    pub visible: bool,
}

/// The characters a G0 or G1 slot holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Charset {
    #[default]
    Ascii,
    /// `#` is the pound sign.
    Uk,
    /// The DEC special graphics set: `q` is a horizontal line, `x` a vertical one, `l` `k` `m`
    /// `j` the corners, and so on.
    DecLineDrawing,
}

impl Charset {
    pub fn translate(self, c: char) -> char {
        match self {
            Charset::Ascii => c,
            Charset::Uk if c == '#' => '£',
            Charset::Uk => c,
            Charset::DecLineDrawing => match c {
                '`' => '◆',
                'a' => '▒',
                'b' => '␉',
                'c' => '␌',
                'd' => '␍',
                'e' => '␊',
                'f' => '°',
                'g' => '±',
                'h' => '␤',
                'i' => '␋',
                'j' => '┘',
                'k' => '┐',
                'l' => '┌',
                'm' => '└',
                'n' => '┼',
                'o' => '⎺',
                'p' => '⎻',
                'q' => '─',
                'r' => '⎼',
                's' => '⎽',
                't' => '├',
                'u' => '┤',
                'v' => '┴',
                'w' => '┬',
                'x' => '│',
                'y' => '≤',
                'z' => '≥',
                '{' => 'π',
                '|' => '≠',
                '}' => '£',
                '~' => '·',
                other => other,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalMode {
    /// DECCKM: arrow keys send application sequences.
    pub cursor_key_mode: bool,
    /// DECKPAM: the keypad sends application sequences.
    pub keypad_mode: bool,
    /// DECAWM: text continues on the next line at the right edge.
    pub auto_wrap_mode: bool,
    /// DECOM: cursor positions are relative to the scrolling region.
    pub origin_mode: bool,
    /// IRM: printed characters push the rest of the line right.
    pub insert_mode: bool,
    /// LNM: line feed also returns to the first column.
    pub newline_mode: bool,
    /// DECSCNM: the whole screen is shown in reverse video.
    pub reverse_video: bool,
    pub mouse_tracking: MouseTracking,
    /// Mode 1006: mouse reports use the SGR encoding.
    pub mouse_sgr_mode: bool,
    /// Mode 2004: pasted text is wrapped in `ESC [ 200 ~` and `ESC [ 201 ~`.
    pub bracketed_paste_mode: bool,
}

impl Default for TerminalMode {
    fn default() -> Self {
        TerminalMode {
            cursor_key_mode: false,
            keypad_mode: false,
            auto_wrap_mode: true,
            origin_mode: false,
            insert_mode: false,
            newline_mode: false,
            reverse_video: false,
            mouse_tracking: MouseTracking::Off,
            mouse_sgr_mode: false,
            bracketed_paste_mode: false,
        }
    }
}

/// The lines scrolling happens in (DECSTBM), 0-based and inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ScrollRegion {
    pub top: usize,
    pub bottom: usize,
}
