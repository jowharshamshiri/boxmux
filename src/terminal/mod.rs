//! The terminal emulator behind PTY boxes: bytes a program writes go in, a screen of styled
//! cells with their hyperlinks comes out, along with the replies the program asked for (cursor position, device
//! attributes) and clipboard writes.

mod buffer;
mod cell;
mod perform;
mod state;
#[cfg(test)]
mod tests;

pub use buffer::*;
pub use cell::*;
pub use state::*;

use crate::link::Link;
use crate::width::Glyph;
use std::sync::Arc;
use vte::{Params, Parser};

/// A terminal emulator: bytes a program writes go in, a screen of styled cells comes out.
pub struct Emulator {
    pub terminal_state: TerminalState,
    /// Kept across calls, so an escape sequence split between two reads is still understood.
    parser: Parser,
}

impl Emulator {
    pub fn new() -> Self {
        Self::with_screen_size(80, 24)
    }

    pub fn with_screen_size(width: usize, height: usize) -> Self {
        Emulator {
            terminal_state: TerminalState::new(width, height),
            parser: Parser::new(),
        }
    }

    pub fn resize_screen(&mut self, width: usize, height: usize) {
        self.terminal_state.resize(width, height);
    }

    pub fn get_cursor_x(&self) -> usize {
        self.terminal_state.get_cursor().x
    }

    pub fn get_cursor_y(&self) -> usize {
        self.terminal_state.get_cursor().y
    }

    pub fn is_using_alternate_screen(&self) -> bool {
        self.terminal_state.use_alternate_screen
    }

    pub fn get_terminal_state(&self) -> &TerminalState {
        &self.terminal_state
    }

    pub fn process_bytes(&mut self, bytes: &[u8]) {
        let mut parser = std::mem::take(&mut self.parser);
        parser.advance(self, bytes);
        self.parser = parser;
    }

    pub fn process_string(&mut self, text: &str) {
        self.process_bytes(text.as_bytes());
    }

    /// Takes the program's clipboard writes since the last call: the text it put on the
    /// clipboard, or why a write cannot be copied.
    pub fn take_clipboard_writes(&mut self) -> Vec<Result<String, String>> {
        std::mem::take(&mut self.terminal_state.clipboard_writes)
    }

    /// Takes the replies to terminal queries that must be written back to the program.
    pub fn take_responses(&mut self) -> Vec<String> {
        std::mem::take(&mut self.terminal_state.pending_responses)
    }

    /// The lines a box shows for this terminal, with SGR color sequences and OSC 8 hyperlinks
    /// ([`crate::link`]): the scrollback and the screen, or only the screen while the program
    /// uses the alternate screen. The cursor, unless the program hid it, is the cell shown in
    /// reverse video.
    pub fn display_lines(&self) -> Vec<String> {
        let state = &self.terminal_state;
        let buffer = state.get_active_buffer_ref();
        buffer.to_content_lines(!state.use_alternate_screen, buffer.cursor.visible)
    }

    /// The text of the screen, without colors, trailing spaces or trailing empty lines.
    pub fn get_processed_text(&self) -> String {
        let mut content = String::new();
        for row in self.terminal_state.get_active_screen_ref() {
            let mut line = String::new();
            for cell in row.iter().filter(|cell| cell.glyph != WIDE_CONTINUATION) {
                cell.glyph.push_to(&mut line);
            }
            content.push_str(line.trim_end());
            content.push('\n');
        }
        content.trim_end().to_string()
    }

    /// The window title the program set (OSC 0 or 2).
    pub fn get_current_terminal_title(&self) -> Option<String> {
        self.terminal_state.terminal_title.clone()
    }

    /// Puts `more` into the cell of the cluster printed last when it continues that cluster,
    /// and returns whether it did. A cluster that grows to two columns (an emoji variation
    /// selector after a one-column emoji) takes the cell after it, and the cursor moves past
    /// that cell; at the last column it cannot grow, so `more` is dropped and it shows in its
    /// one-column form. A cluster that shrinks to one column leaves its right half blank.
    fn join_printed(&mut self, more: &Glyph) -> bool {
        let state = &mut self.terminal_state;
        let Some((x, y)) = state.joinable else {
            return false;
        };
        let screen_width = state.screen_width;
        let more = more.to_string();
        let before = state.get_active_screen()[y][x].glyph.clone();
        if !before.continues_with(&more) {
            return false;
        }
        let joined = before.joined(&more);
        let grows = joined.width() > before.width();
        if grows && x + 1 >= screen_width {
            return true;
        }
        let row = &mut state.get_active_screen()[y];
        if grows {
            if row[x + 1].glyph.width() == 2 {
                if let Some(right) = row.get_mut(x + 2) {
                    right.glyph = Glyph::SPACE;
                }
            }
            let mut continuation = row[x].clone();
            continuation.glyph = WIDE_CONTINUATION;
            row[x + 1] = continuation;
        } else if joined.width() < before.width() {
            row[x + 1].glyph = Glyph::SPACE;
        }
        row[x].glyph = joined;
        if grows {
            if x + 2 >= screen_width {
                state.get_cursor_mut().x = screen_width - 1;
                state.wrap_pending = state.mode.auto_wrap_mode;
            } else {
                state.get_cursor_mut().x = x + 2;
            }
        }
        true
    }

    /// Writes `glyph` at the cursor with the current attributes. A wide cluster also takes the
    /// next cell.
    fn put_glyph_at_cursor(&mut self, glyph: Glyph, width: usize) {
        let CursorState { x, y, .. } = *self.terminal_state.get_cursor();
        let screen_width = self.terminal_state.screen_width;
        let mut cell = self.terminal_state.current_attributes.clone();
        cell.glyph = glyph;
        cell.link = self.terminal_state.hyperlink.clone();
        let row = &mut self.terminal_state.get_active_screen()[y];
        // Writing over half of a wide character leaves its other half blank.
        if row[x].glyph == WIDE_CONTINUATION && x > 0 {
            row[x - 1].glyph = Glyph::SPACE;
        }
        let end = x + width;
        if end < screen_width && row[end].glyph == WIDE_CONTINUATION {
            row[end].glyph = Glyph::SPACE;
        }
        if width == 2 && x + 1 < screen_width {
            let mut continuation = cell.clone();
            continuation.glyph = WIDE_CONTINUATION;
            row[x + 1] = continuation;
        }
        row[x] = cell;
    }

    /// Applies one SGR sequence. Parameters come either separated by semicolons
    /// (`38;2;r;g;b`) or as colon sub-parameters of one parameter (`38:2::r:g:b`, `4:3`).
    fn apply_sgr(&mut self, params: &Params) {
        let groups: Vec<&[u16]> = params.iter().collect();
        if groups.is_empty() {
            self.terminal_state.current_attributes = TerminalCell::default();
            return;
        }
        let mut i = 0;
        while i < groups.len() {
            let group = groups[i];
            i += 1;
            let attributes = &mut self.terminal_state.current_attributes;
            match group[0] {
                code @ (38 | 48 | 58) => {
                    let color = if group.len() > 1 {
                        extended_color(&group[1..], true)
                    } else {
                        // The color's parameters follow as separate parameters.
                        let rest: Vec<u16> = groups[i..].iter().map(|g| g[0]).collect();
                        let (color, used) = match rest.first() {
                            Some(5) => (
                                rest.get(1)
                                    .and_then(|&n| u8::try_from(n).ok())
                                    .map(TermColor::Indexed),
                                2,
                            ),
                            Some(2) => (extended_color(&rest, false), 4),
                            _ => (None, rest.len().min(1)),
                        };
                        i += used.min(rest.len());
                        color
                    };
                    match code {
                        38 => attributes.fg_color = color,
                        48 => attributes.bg_color = color,
                        // Underline color: underlines take the text color here.
                        _ => {}
                    }
                }
                0 => *attributes = TerminalCell::default(),
                1 => attributes.bold = true,
                2 => attributes.dim = true,
                3 => attributes.italic = true,
                // `4:0` is "no underline"; every other style is an underline.
                4 => attributes.underline = group.get(1) != Some(&0),
                5 | 6 => attributes.blink = true,
                7 => attributes.reverse = true,
                8 => attributes.hidden = true,
                9 => attributes.strikethrough = true,
                21 => attributes.underline = true,
                22 => {
                    attributes.bold = false;
                    attributes.dim = false;
                }
                23 => attributes.italic = false,
                24 => attributes.underline = false,
                25 => attributes.blink = false,
                27 => attributes.reverse = false,
                28 => attributes.hidden = false,
                29 => attributes.strikethrough = false,
                n @ 30..=37 => attributes.fg_color = Some(TermColor::Indexed((n - 30) as u8)),
                39 => attributes.fg_color = None,
                n @ 40..=47 => attributes.bg_color = Some(TermColor::Indexed((n - 40) as u8)),
                49 => attributes.bg_color = None,
                n @ 90..=97 => attributes.fg_color = Some(TermColor::Indexed((n - 90 + 8) as u8)),
                n @ 100..=107 => {
                    attributes.bg_color = Some(TermColor::Indexed((n - 100 + 8) as u8))
                }
                // Fonts, frames, overlines and ideogram marks are not shown.
                other => log::debug!("SGR {} is not shown", other),
            }
        }
    }

    /// The first parameter of `params` or `default` when it is missing or 0.
    fn param_or(params: &Params, index: usize, default: u16) -> u16 {
        match params.iter().nth(index).and_then(|p| p.first().copied()) {
            Some(0) | None => default,
            Some(n) => n,
        }
    }

    /// The state of DEC private mode `mode`, or `None` when it is not one this terminal knows.
    fn private_mode(&self, mode: u16) -> Option<bool> {
        let state = &self.terminal_state;
        let m = &state.mode;
        Some(match mode {
            1 => m.cursor_key_mode,
            5 => m.reverse_video,
            6 => m.origin_mode,
            7 => m.auto_wrap_mode,
            9 => m.mouse_tracking == MouseTracking::X10,
            25 => state.get_cursor().visible,
            47 | 1047 | 1049 => state.use_alternate_screen,
            66 => m.keypad_mode,
            1000 => m.mouse_tracking == MouseTracking::Normal,
            1002 => m.mouse_tracking == MouseTracking::ButtonEvent,
            1003 => m.mouse_tracking == MouseTracking::AnyEvent,
            1006 => m.mouse_sgr_mode,
            2004 => m.bracketed_paste_mode,
            _ => return None,
        })
    }

    /// Sets (`on`) or resets DEC private mode `mode`.
    fn set_private_mode(&mut self, mode: u16, on: bool) {
        let state = &mut self.terminal_state;
        let tracking = |kind: MouseTracking| if on { kind } else { MouseTracking::Off };
        match mode {
            1 => state.mode.cursor_key_mode = on,
            5 => state.mode.reverse_video = on,
            6 => {
                state.mode.origin_mode = on;
                state.set_cursor_position(0, 0);
            }
            7 => state.mode.auto_wrap_mode = on,
            9 => state.mode.mouse_tracking = tracking(MouseTracking::X10),
            25 => state.get_cursor_mut().visible = on,
            66 => state.mode.keypad_mode = on,
            1000 => state.mode.mouse_tracking = tracking(MouseTracking::Normal),
            1002 => state.mode.mouse_tracking = tracking(MouseTracking::ButtonEvent),
            1003 => state.mode.mouse_tracking = tracking(MouseTracking::AnyEvent),
            1006 => state.mode.mouse_sgr_mode = on,
            2004 => state.mode.bracketed_paste_mode = on,
            47 => {
                if on {
                    state.switch_to_alternate_screen();
                } else {
                    state.switch_to_primary_screen();
                }
            }
            1047 => {
                if on {
                    state.switch_to_alternate_screen();
                } else {
                    // Leaving clears the alternate screen for the next program.
                    state.clear_alternate_screen();
                    state.switch_to_primary_screen();
                }
            }
            1048 => {
                if on {
                    state.save_cursor();
                } else {
                    state.restore_cursor();
                }
            }
            1049 => {
                if on && !state.use_alternate_screen {
                    state.save_cursor();
                    state.switch_to_alternate_screen();
                    state.clear_alternate_screen();
                } else if !on && state.use_alternate_screen {
                    state.switch_to_primary_screen();
                    state.restore_cursor();
                }
            }
            // Cursor blinking, focus events and the alternative mouse encodings change
            // nothing that is drawn or reported here.
            other => log::debug!("DEC private mode {} is not supported", other),
        }
    }

    /// The state of ANSI mode `mode`, or `None` when it is not one this terminal knows.
    fn ansi_mode(&self, mode: u16) -> Option<bool> {
        match mode {
            4 => Some(self.terminal_state.mode.insert_mode),
            20 => Some(self.terminal_state.mode.newline_mode),
            _ => None,
        }
    }

    fn set_ansi_mode(&mut self, mode: u16, on: bool) {
        match mode {
            4 => self.terminal_state.mode.insert_mode = on,
            20 => self.terminal_state.mode.newline_mode = on,
            other => log::debug!("ANSI mode {} is not supported", other),
        }
    }
}

/// A color from the parameters after 38, 48 or 58: `5;n` for a palette entry, `2;r;g;b` for
/// a 24-bit color. In the colon form (`colon`), `2` may be followed by a color space id before
/// the components.
fn extended_color(params: &[u16], colon: bool) -> Option<TermColor> {
    let byte = |i: usize| params.get(i).and_then(|&n| u8::try_from(n).ok());
    match params.first()? {
        5 => byte(1).map(TermColor::Indexed),
        2 => {
            let start = if colon && params.len() >= 5 { 2 } else { 1 };
            Some(TermColor::Rgb(
                byte(start)?,
                byte(start + 1)?,
                byte(start + 2)?,
            ))
        }
        _ => None,
    }
}

impl Default for Emulator {
    fn default() -> Self {
        Self::new()
    }
}
