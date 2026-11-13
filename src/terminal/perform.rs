//! What each control sequence does to the terminal.

use super::*;
use vte::{Params, Perform};

/// The primary device attributes: a VT220 with ANSI colors.
const PRIMARY_DEVICE_ATTRIBUTES: &str = "\x1b[?62;22c";
/// The secondary device attributes: terminal type 0 (VT100 family), version 0.
const SECONDARY_DEVICE_ATTRIBUTES: &str = "\x1b[>0;0;0c";

impl Perform for Emulator {
    fn print(&mut self, c: char) {
        let c = self.terminal_state.charsets[self.terminal_state.active_charset].translate(c);
        let glyph = Glyph::from_char(c);
        if self.join_printed(&glyph) {
            return;
        }
        self.terminal_state.joinable = None;
        let width = glyph.width();
        if width == 0 {
            // A combining mark or selector with no cluster before it has no cell to go in.
            return;
        }
        self.terminal_state.last_printed = Some(c);
        let state = &mut self.terminal_state;
        let screen_width = state.screen_width;
        let auto_wrap = state.mode.auto_wrap_mode;
        // A character that does not fit on the rest of the line wraps first, like on a real
        // terminal: either the previous one filled the line, or this one is wide.
        let fits = state.get_cursor().x + width <= screen_width;
        if state.wrap_pending || !fits {
            if auto_wrap {
                state.get_cursor_mut().x = 0;
                state.line_feed_with_scrolling();
            } else {
                state.get_cursor_mut().x = screen_width.saturating_sub(width);
            }
            state.wrap_pending = false;
        }
        if width > screen_width {
            return;
        }
        if state.mode.insert_mode {
            state.insert_blank_characters(width);
        }
        self.put_glyph_at_cursor(glyph, width);
        let state = &mut self.terminal_state;
        let CursorState { x, y, .. } = *state.get_cursor();
        state.joinable = Some((x, y));
        let next = x + width;
        if next >= screen_width {
            // The cursor stays on the last column until the next character arrives.
            state.get_cursor_mut().x = screen_width - 1;
            state.wrap_pending = auto_wrap;
        } else {
            state.get_cursor_mut().x = next;
        }
    }

    fn execute(&mut self, byte: u8) {
        self.terminal_state.joinable = None;
        let state = &mut self.terminal_state;
        if matches!(byte, b'\n' | 0x0B | 0x0C | b'\r' | b'\t' | 0x08) {
            state.wrap_pending = false;
        }
        match byte {
            // Line feed, vertical tab and form feed move down one line; they also return to
            // the first column in newline mode.
            b'\n' | 0x0B | 0x0C => {
                if state.mode.newline_mode {
                    state.get_cursor_mut().x = 0;
                }
                state.line_feed_with_scrolling();
            }
            b'\r' => state.get_cursor_mut().x = 0,
            b'\t' => state.tab_forward(),
            // Backspace moves left; it does not erase.
            0x08 => state.cursor_left(1),
            // Shift out and shift in: print from G1 or G0.
            0x0E => state.active_charset = 1,
            0x0F => state.active_charset = 0,
            // The bell, the enquiry (whose answerback is empty) and the rest do nothing here.
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, intermediates: &[u8], _ignore: bool, c: char) {
        self.terminal_state.joinable = None;
        let request = match (intermediates, c) {
            (b"$", 'q') => DcsRequest::StatusString,
            (b"+", 'q') => DcsRequest::Termcap,
            _ => DcsRequest::Other,
        };
        self.terminal_state.dcs_request = Some(request);
        self.terminal_state.dcs_data.clear();
    }

    fn put(&mut self, byte: u8) {
        self.terminal_state.joinable = None;
        // Queries are short; anything longer (sixel images and the like) is not kept.
        if self.terminal_state.dcs_data.len() < 1024 {
            self.terminal_state.dcs_data.push(byte);
        }
    }

    fn unhook(&mut self) {
        self.terminal_state.joinable = None;
        let state = &mut self.terminal_state;
        let data = String::from_utf8_lossy(&state.dcs_data).into_owned();
        state.dcs_data.clear();
        // Settings and termcap entries are not reported: the answer is "not valid", which
        // lets the program go on with its defaults instead of waiting.
        match state.dcs_request.take() {
            Some(DcsRequest::StatusString) => {
                state.pending_responses.push("\x1bP0$r\x1b\\".to_string())
            }
            Some(DcsRequest::Termcap) => state
                .pending_responses
                .push(format!("\x1bP0+r{}\x1b\\", data)),
            Some(DcsRequest::Other) | None => log::debug!("ignored a device control string"),
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        self.terminal_state.joinable = None;
        let text = |i: usize| {
            params
                .get(i)
                .map(|p| String::from_utf8_lossy(p).into_owned())
        };
        let Some(command) = text(0) else {
            return;
        };
        // The data may itself contain semicolons, which the parser split on.
        let data = || {
            params
                .get(1..)
                .unwrap_or_default()
                .iter()
                .map(|p| String::from_utf8_lossy(p).into_owned())
                .collect::<Vec<_>>()
                .join(";")
        };
        match command.as_str() {
            "0" | "2" => {
                self.terminal_state.terminal_title = Some(data());
            }
            "1" => self.terminal_state.icon_name = Some(data()),
            "52" => {
                // `52 ; targets ; base64` sets the clipboard. A `?` asks for its contents,
                // which programs are not given.
                let Some(payload) = text(2) else {
                    return;
                };
                if payload == "?" {
                    log::debug!("refused a clipboard read request");
                    return;
                }
                match crate::base64::decode(&payload)
                    .and_then(|bytes| String::from_utf8(bytes).ok())
                {
                    Some(copied) => self.terminal_state.clipboard_writes.push(copied),
                    None => log::debug!("ignored a clipboard write that is not base64 UTF-8 text"),
                }
            }
            other => log::debug!("unhandled OSC {}", other),
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, c: char) {
        self.terminal_state.joinable = None;
        let count = |i: usize| usize::from(Self::param_or(params, i, 1));
        let private = intermediates.first() == Some(&b'?');
        if intermediates.is_empty()
            && matches!(
                c,
                'H' | 'f' | 'A' | 'B' | 'C' | 'D' | 'E' | 'F' | 'G' | 'd' | '`' | 'a' | 'e' | 'r'
            )
        {
            // Moving the cursor cancels a pending wrap at the last column.
            self.terminal_state.wrap_pending = false;
        }
        match (intermediates, c) {
            (b"", 'm') => self.apply_sgr(params),
            (b"", 'H' | 'f') => self
                .terminal_state
                .set_cursor_position(count(1) - 1, count(0) - 1),
            (b"", 'A') => self.terminal_state.cursor_up(count(0)),
            (b"", 'B' | 'e') => self.terminal_state.cursor_down(count(0)),
            (b"", 'C' | 'a') => self.terminal_state.cursor_right(count(0)),
            (b"", 'D') => self.terminal_state.cursor_left(count(0)),
            (b"", 'E') => {
                self.terminal_state.cursor_down(count(0));
                self.terminal_state.get_cursor_mut().x = 0;
            }
            (b"", 'F') => {
                self.terminal_state.cursor_up(count(0));
                self.terminal_state.get_cursor_mut().x = 0;
            }
            (b"", 'G' | '`') => {
                let last = self.terminal_state.screen_width - 1;
                self.terminal_state.get_cursor_mut().x = (count(0) - 1).min(last);
            }
            (b"", 'd') => {
                let x = self.terminal_state.get_cursor().x;
                self.terminal_state.set_cursor_position(x, count(0) - 1);
            }
            (b"" | b"?", 'J') => self.erase_in_display(Self::param_or(params, 0, 0)),
            (b"" | b"?", 'K') => self.erase_in_line(Self::param_or(params, 0, 0)),
            (b"", '@') => self.terminal_state.insert_blank_characters(count(0)),
            (b"", 'P') => self.terminal_state.delete_characters(count(0)),
            (b"", 'X') => {
                let CursorState { x, y, .. } = *self.terminal_state.get_cursor();
                self.terminal_state.erase_cells(y, x, x + count(0));
            }
            (b"", 'L') => self.terminal_state.insert_lines(count(0)),
            (b"", 'M') => self.terminal_state.delete_lines(count(0)),
            (b"", 'S') => self.terminal_state.scroll_up_in_region(count(0)),
            (b"", 'T') => self.terminal_state.scroll_down_in_region(count(0)),
            (b"", 'b') => {
                if let Some(last) = self.terminal_state.last_printed {
                    // Bounded by the screen: more repeats only overwrite the same cells.
                    let screen =
                        self.terminal_state.screen_width * self.terminal_state.screen_height;
                    for _ in 0..count(0).min(screen) {
                        self.print(last);
                    }
                }
            }
            (b"", 'r') => {
                let param = |i: usize| match Self::param_or(params, i, 0) {
                    0 => None,
                    n => Some(usize::from(n)),
                };
                self.terminal_state.set_scrolling_region(param(0), param(1));
                self.terminal_state.set_cursor_position(0, 0);
            }
            (b"", 's') => self.terminal_state.save_cursor(),
            (b"", 'u') => self.terminal_state.restore_cursor(),
            (b"" | b"?", 'h' | 'l') => {
                let on = c == 'h';
                for mode in params.iter().filter_map(|p| p.first().copied()) {
                    if private {
                        self.set_private_mode(mode, on);
                    } else {
                        self.set_ansi_mode(mode, on);
                    }
                }
            }
            (b"" | b"?", 'n') => self.device_status_report(Self::param_or(params, 0, 0), private),
            (b"", 'c') if Self::param_or(params, 0, 0) == 0 => self
                .terminal_state
                .pending_responses
                .push(PRIMARY_DEVICE_ATTRIBUTES.to_string()),
            (b">", 'c') if Self::param_or(params, 0, 0) == 0 => self
                .terminal_state
                .pending_responses
                .push(SECONDARY_DEVICE_ATTRIBUTES.to_string()),
            (b"$" | b"?$", 'p') => {
                // DECRQM: 1 set, 2 reset, 0 not recognized.
                let mode = Self::param_or(params, 0, 0);
                let state = if private {
                    self.private_mode(mode)
                } else {
                    self.ansi_mode(mode)
                };
                let value = match state {
                    Some(true) => 1,
                    Some(false) => 2,
                    None => 0,
                };
                let prefix = if private { "?" } else { "" };
                self.terminal_state
                    .pending_responses
                    .push(format!("\x1b[{}{};{}$y", prefix, mode, value));
            }
            (b"", 't') => self.window_report(Self::param_or(params, 0, 0)),
            (b"", 'g') => match Self::param_or(params, 0, 0) {
                0 => {
                    let x = self.terminal_state.get_cursor().x;
                    self.terminal_state.tab_stops[x] = false;
                }
                3 => self.terminal_state.tab_stops.fill(false),
                _ => {}
            },
            (b"", 'I') => {
                for _ in 0..count(0) {
                    self.terminal_state.tab_forward();
                }
            }
            (b"", 'Z') => {
                for _ in 0..count(0) {
                    self.terminal_state.tab_backward();
                }
            }
            // The cursor shape (DECSCUSR) is not drawn; the cursor is always a reverse cell.
            (b" ", 'q') => {}
            _ => log::debug!("unhandled CSI {:?} {:?} {}", intermediates, params, c),
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        self.terminal_state.joinable = None;
        let state = &mut self.terminal_state;
        if intermediates.is_empty() && matches!(byte, b'D' | b'E' | b'M') {
            state.wrap_pending = false;
        }
        match (intermediates, byte) {
            (b"", b'7') => state.save_cursor(),
            (b"", b'8') => state.restore_cursor(),
            (b"", b'c') => {
                // RIS: back to the power-on state, keeping the size and the scrollback.
                let (width, height) = (state.screen_width, state.screen_height);
                let scrollback = std::mem::take(&mut state.primary_buffer.scrollback);
                *state = TerminalState::new(width, height);
                state.primary_buffer.scrollback = scrollback;
            }
            (b"", b'D') => state.line_feed_with_scrolling(),
            (b"", b'E') => {
                state.get_cursor_mut().x = 0;
                state.line_feed_with_scrolling();
            }
            (b"", b'M') => state.reverse_line_feed_with_scrolling(),
            (b"", b'H') => {
                let x = state.get_cursor().x;
                state.tab_stops[x] = true;
            }
            (b"", b'=') => state.mode.keypad_mode = true,
            (b"", b'>') => state.mode.keypad_mode = false,
            (b"", b'Z') => state
                .pending_responses
                .push(PRIMARY_DEVICE_ATTRIBUTES.to_string()),
            (b"(" | b")", designator) => {
                let slot = usize::from(intermediates == b")");
                state.charsets[slot] = match designator {
                    b'0' => Charset::DecLineDrawing,
                    b'A' => Charset::Uk,
                    _ => Charset::Ascii,
                };
            }
            (b"#", b'8') => {
                // DECALN: fills the screen with E, for aligning the display.
                let filled = TerminalCell {
                    glyph: Glyph::from_char('E'),
                    ..TerminalCell::default()
                };
                for row in state.get_active_screen() {
                    row.fill(filled.clone());
                }
                state.scroll_region = ScrollRegion {
                    top: 0,
                    bottom: state.screen_height - 1,
                };
                state.set_cursor_position(0, 0);
            }
            // Double-width and double-height lines are shown at normal size.
            (b"#", b'3' | b'4' | b'5' | b'6') => {}
            _ => log::debug!("unhandled ESC {:?} {}", intermediates, byte as char),
        }
    }
}

impl Emulator {
    /// ED: 0 erases from the cursor to the end of the screen, 1 from the start to the cursor,
    /// 2 the whole screen, 3 the scrollback.
    fn erase_in_display(&mut self, mode: u16) {
        let state = &mut self.terminal_state;
        let CursorState { x, y, .. } = *state.get_cursor();
        let (width, height) = (state.screen_width, state.screen_height);
        match mode {
            0 => {
                state.erase_cells(y, x, width);
                for line in y + 1..height {
                    state.erase_cells(line, 0, width);
                }
            }
            1 => {
                for line in 0..y {
                    state.erase_cells(line, 0, width);
                }
                state.erase_cells(y, 0, x + 1);
            }
            2 => state.clear_screen(),
            3 => state.get_active_buffer().scrollback.clear(),
            _ => {}
        }
    }

    /// EL: 0 erases from the cursor to the end of the line, 1 from the start to the cursor,
    /// 2 the whole line.
    fn erase_in_line(&mut self, mode: u16) {
        let state = &mut self.terminal_state;
        let CursorState { x, y, .. } = *state.get_cursor();
        let width = state.screen_width;
        match mode {
            0 => state.erase_cells(y, x, width),
            1 => state.erase_cells(y, 0, x + 1),
            2 => state.erase_cells(y, 0, width),
            _ => {}
        }
    }

    /// DSR: 5 asks whether the terminal is working, 6 where the cursor is (relative to the
    /// origin in origin mode).
    fn device_status_report(&mut self, request: u16, private: bool) {
        let state = &mut self.terminal_state;
        match request {
            5 => state.pending_responses.push("\x1b[0n".to_string()),
            6 => {
                let line = state.cursor_line_from_origin() + 1;
                let column = state.get_cursor().x + 1;
                let prefix = if private { "?" } else { "" };
                state
                    .pending_responses
                    .push(format!("\x1b[{}{};{}R", prefix, line, column));
            }
            other => log::debug!("unhandled device status request {}", other),
        }
    }

    /// Window reports (XTWINOPS): 18 is the screen size in cells, 14 in pixels, counted as 8
    /// by 16 pixel cells. Window changes are not done.
    fn window_report(&mut self, request: u16) {
        let state = &mut self.terminal_state;
        let (columns, lines) = (state.screen_width, state.screen_height);
        match request {
            18 => state
                .pending_responses
                .push(format!("\x1b[8;{};{}t", lines, columns)),
            14 => state
                .pending_responses
                .push(format!("\x1b[4;{};{}t", lines * 16, columns * 8)),
            other => log::debug!("unhandled window operation {}", other),
        }
    }
}
