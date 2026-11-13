//! Everything a terminal remembers between bytes: both screens, the cursor, modes, tab stops,
//! character sets and the replies owed to the program.

use super::*;

/// How many lines that scrolled off the primary screen are kept.
pub const MAX_SCROLLBACK: usize = 10_000;

/// What DECSC saves and DECRC restores.
#[derive(Debug, Clone, PartialEq)]
pub struct SavedCursor {
    pub x: usize,
    pub y: usize,
    pub attributes: TerminalCell,
    pub origin_mode: bool,
    pub auto_wrap_mode: bool,
    pub charsets: [Charset; 2],
    pub active_charset: usize,
}

/// A device control string being received, by what it asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DcsRequest {
    /// `DCS $ q ... ST` (DECRQSS): the setting named by the data.
    StatusString,
    /// `DCS + q ... ST` (XTGETTCAP): the termcap entries named by the data.
    Termcap,
    /// Anything else, which is read and dropped.
    Other,
}

#[derive(Debug, Clone)]
pub struct TerminalState {
    pub screen_width: usize,
    pub screen_height: usize,
    pub primary_buffer: TerminalScreenBuffer,
    /// The screen full-screen programs draw on; it has no scrollback.
    pub alternate_buffer: TerminalScreenBuffer,
    pub use_alternate_screen: bool,
    /// DECSC's saved state, one per screen.
    pub saved_cursor: [Option<SavedCursor>; 2],
    /// The attributes printed characters get.
    pub current_attributes: TerminalCell,
    /// The hyperlink printed characters are part of (OSC 8). It is not an attribute: SGR resets
    /// and DECSC/DECRC leave it as it is.
    pub hyperlink: Option<Arc<Link>>,
    pub mode: TerminalMode,
    pub scroll_region: ScrollRegion,
    pub tab_stops: Vec<bool>,
    /// The G0 and G1 character sets, and which of them is in use (shift in / shift out).
    pub charsets: [Charset; 2],
    pub active_charset: usize,
    /// The last character printed, for REP.
    pub last_printed: Option<char>,
    /// The cell (column, row) holding the cluster printed last, while nothing but printing has
    /// happened since: a character that continues that cluster (a combining mark, a variation
    /// selector, a skin tone, a joiner and what it joins) goes into that cell.
    pub joinable: Option<(usize, usize)>,
    pub terminal_title: Option<String>,
    pub icon_name: Option<String>,
    pub dcs_request: Option<DcsRequest>,
    pub dcs_data: Vec<u8>,
    /// Replies to queries (DSR, DA, DECRQM...) waiting to be written back to the program.
    pub pending_responses: Vec<String>,
    /// A character was written to the last column; the next one goes to the next line.
    pub wrap_pending: bool,
    /// Text the program put on the clipboard (OSC 52), waiting to be copied.
    pub clipboard_writes: Vec<String>,
}

impl TerminalState {
    pub fn new(width: usize, height: usize) -> Self {
        let (width, height) = (width.max(1), height.max(1));
        TerminalState {
            screen_width: width,
            screen_height: height,
            primary_buffer: TerminalScreenBuffer::new(width, height, MAX_SCROLLBACK),
            alternate_buffer: TerminalScreenBuffer::new(width, height, 0),
            use_alternate_screen: false,
            saved_cursor: [None, None],
            current_attributes: TerminalCell::default(),
            hyperlink: None,
            mode: TerminalMode::default(),
            scroll_region: ScrollRegion {
                top: 0,
                bottom: height - 1,
            },
            tab_stops: default_tab_stops(width),
            charsets: [Charset::Ascii; 2],
            active_charset: 0,
            last_printed: None,
            joinable: None,
            terminal_title: None,
            icon_name: None,
            dcs_request: None,
            dcs_data: Vec::new(),
            pending_responses: Vec::new(),
            wrap_pending: false,
            clipboard_writes: Vec::new(),
        }
    }

    pub fn get_active_buffer(&mut self) -> &mut TerminalScreenBuffer {
        if self.use_alternate_screen {
            &mut self.alternate_buffer
        } else {
            &mut self.primary_buffer
        }
    }

    pub fn get_active_buffer_ref(&self) -> &TerminalScreenBuffer {
        if self.use_alternate_screen {
            &self.alternate_buffer
        } else {
            &self.primary_buffer
        }
    }

    pub fn get_active_screen(&mut self) -> &mut Vec<Vec<TerminalCell>> {
        &mut self.get_active_buffer().content
    }

    pub fn get_active_screen_ref(&self) -> &Vec<Vec<TerminalCell>> {
        &self.get_active_buffer_ref().content
    }

    pub fn get_cursor(&self) -> &CursorState {
        &self.get_active_buffer_ref().cursor
    }

    pub fn get_cursor_mut(&mut self) -> &mut CursorState {
        &mut self.get_active_buffer().cursor
    }

    /// A blank cell as erasing leaves it: with the current background color.
    fn blank(&self) -> TerminalCell {
        TerminalCell::blank(&self.current_attributes)
    }

    /// DECSC: saves the cursor position, attributes, origin and wrap modes and character sets.
    pub fn save_cursor(&mut self) {
        let cursor = *self.get_cursor();
        let saved = SavedCursor {
            x: cursor.x,
            y: cursor.y,
            attributes: self.current_attributes.clone(),
            origin_mode: self.mode.origin_mode,
            auto_wrap_mode: self.mode.auto_wrap_mode,
            charsets: self.charsets,
            active_charset: self.active_charset,
        };
        self.saved_cursor[usize::from(self.use_alternate_screen)] = Some(saved);
    }

    /// DECRC: restores what DECSC saved, or, when nothing was saved, homes the cursor and resets
    /// the attributes and modes, like xterm.
    pub fn restore_cursor(&mut self) {
        let saved = self.saved_cursor[usize::from(self.use_alternate_screen)]
            .clone()
            .unwrap_or(SavedCursor {
                x: 0,
                y: 0,
                attributes: TerminalCell::default(),
                origin_mode: false,
                auto_wrap_mode: true,
                charsets: [Charset::Ascii; 2],
                active_charset: 0,
            });
        self.current_attributes = saved.attributes;
        self.mode.origin_mode = saved.origin_mode;
        self.mode.auto_wrap_mode = saved.auto_wrap_mode;
        self.charsets = saved.charsets;
        self.active_charset = saved.active_charset;
        let (width, height) = (self.screen_width, self.screen_height);
        let cursor = self.get_cursor_mut();
        cursor.x = saved.x.min(width - 1);
        cursor.y = saved.y.min(height - 1);
        self.wrap_pending = false;
    }

    /// Moves the cursor to column `x`, line `y` (0-based). In origin mode the line counts from
    /// the top of the scrolling region and the cursor stays inside it.
    pub fn set_cursor_position(&mut self, x: usize, y: usize) {
        let (top, bottom) = (self.scroll_region.top, self.scroll_region.bottom);
        let y = if self.mode.origin_mode {
            (top + y).min(bottom)
        } else {
            y.min(self.screen_height - 1)
        };
        let x = x.min(self.screen_width - 1);
        let cursor = self.get_cursor_mut();
        cursor.x = x;
        cursor.y = y;
    }

    /// The cursor line relative to the origin: the scrolling region's top in origin mode.
    pub fn cursor_line_from_origin(&self) -> usize {
        let y = self.get_cursor().y;
        if self.mode.origin_mode {
            y.saturating_sub(self.scroll_region.top)
        } else {
            y
        }
    }

    /// Moves the cursor up; it stops at the top of the scrolling region when it starts inside.
    pub fn cursor_up(&mut self, lines: usize) {
        let top = self.scroll_region.top;
        let cursor = self.get_cursor_mut();
        let limit = if cursor.y >= top { top } else { 0 };
        cursor.y = cursor.y.saturating_sub(lines).max(limit);
    }

    /// Moves the cursor down; it stops at the bottom of the scrolling region when it starts
    /// inside.
    pub fn cursor_down(&mut self, lines: usize) {
        let bottom = self.scroll_region.bottom;
        let last = self.screen_height - 1;
        let cursor = self.get_cursor_mut();
        let limit = if cursor.y <= bottom { bottom } else { last };
        cursor.y = (cursor.y + lines).min(limit);
    }

    pub fn cursor_left(&mut self, columns: usize) {
        let cursor = self.get_cursor_mut();
        cursor.x = cursor.x.saturating_sub(columns);
    }

    pub fn cursor_right(&mut self, columns: usize) {
        let last = self.screen_width - 1;
        let cursor = self.get_cursor_mut();
        cursor.x = (cursor.x + columns).min(last);
    }

    /// Moves to the next tab stop, or the last column when there is none.
    pub fn tab_forward(&mut self) {
        let x = self.get_cursor().x;
        let next = (x + 1..self.screen_width)
            .find(|&column| self.tab_stops[column])
            .unwrap_or(self.screen_width - 1);
        self.get_cursor_mut().x = next;
    }

    /// Moves to the previous tab stop, or the first column when there is none.
    pub fn tab_backward(&mut self) {
        let x = self.get_cursor().x;
        let previous = (0..x)
            .rev()
            .find(|&column| self.tab_stops[column])
            .unwrap_or(0);
        self.get_cursor_mut().x = previous;
    }

    /// DECSTBM with 1-based lines; a missing or invalid region means the whole screen.
    pub fn set_scrolling_region(&mut self, top: Option<usize>, bottom: Option<usize>) {
        let last = self.screen_height - 1;
        let top = top.filter(|&t| t > 0).map_or(0, |t| t - 1);
        let bottom = bottom
            .filter(|&b| b > 0)
            .map_or(last, |b| (b - 1).min(last));
        self.scroll_region = if top < bottom {
            ScrollRegion { top, bottom }
        } else {
            ScrollRegion {
                top: 0,
                bottom: last,
            }
        };
    }

    /// Scrolls the scrolling region up by `lines`: its top lines go away (into the scrollback
    /// when the region starts at the top of the primary screen) and blank lines appear at its
    /// bottom.
    pub fn scroll_up_in_region(&mut self, lines: usize) {
        let ScrollRegion { top, bottom } = self.scroll_region;
        let keep = top == 0 && !self.use_alternate_screen;
        let blank = vec![self.blank(); self.screen_width];
        let buffer = self.get_active_buffer();
        for _ in 0..lines.min(bottom - top + 1) {
            let line = buffer.content.remove(top);
            if keep {
                buffer.push_scrollback(line);
            }
            buffer.content.insert(bottom, blank.clone());
        }
    }

    /// Scrolls the scrolling region down by `lines`: its bottom lines go away and blank lines
    /// appear at its top.
    pub fn scroll_down_in_region(&mut self, lines: usize) {
        let ScrollRegion { top, bottom } = self.scroll_region;
        let blank = vec![self.blank(); self.screen_width];
        let buffer = self.get_active_buffer();
        for _ in 0..lines.min(bottom - top + 1) {
            buffer.content.remove(bottom);
            buffer.content.insert(top, blank.clone());
        }
    }

    /// Line feed: down one line, scrolling the region when the cursor is on its bottom line.
    pub fn line_feed_with_scrolling(&mut self) {
        let y = self.get_cursor().y;
        if y == self.scroll_region.bottom {
            self.scroll_up_in_region(1);
        } else if y + 1 < self.screen_height {
            self.get_cursor_mut().y = y + 1;
        }
    }

    /// Reverse index: up one line, scrolling the region down when the cursor is on its top
    /// line.
    pub fn reverse_line_feed_with_scrolling(&mut self) {
        let y = self.get_cursor().y;
        if y == self.scroll_region.top {
            self.scroll_down_in_region(1);
        } else if y > 0 {
            self.get_cursor_mut().y = y - 1;
        }
    }

    /// IL: inserts `count` blank lines at the cursor line, pushing the lines below it down
    /// within the scrolling region. Nothing happens outside the region.
    pub fn insert_lines(&mut self, count: usize) {
        let y = self.get_cursor().y;
        let ScrollRegion { top, bottom } = self.scroll_region;
        if y < top || y > bottom {
            return;
        }
        let blank = vec![self.blank(); self.screen_width];
        let buffer = self.get_active_buffer();
        for _ in 0..count.min(bottom - y + 1) {
            buffer.content.remove(bottom);
            buffer.content.insert(y, blank.clone());
        }
        buffer.cursor.x = 0;
    }

    /// DL: deletes `count` lines from the cursor line, pulling the lines below it up within
    /// the scrolling region. Nothing happens outside the region.
    pub fn delete_lines(&mut self, count: usize) {
        let y = self.get_cursor().y;
        let ScrollRegion { top, bottom } = self.scroll_region;
        if y < top || y > bottom {
            return;
        }
        let blank = vec![self.blank(); self.screen_width];
        let buffer = self.get_active_buffer();
        for _ in 0..count.min(bottom - y + 1) {
            buffer.content.remove(y);
            buffer.content.insert(bottom, blank.clone());
        }
        buffer.cursor.x = 0;
    }

    /// ICH: inserts `count` blanks at the cursor, pushing the rest of the line right; what
    /// passes the right edge is lost.
    pub fn insert_blank_characters(&mut self, count: usize) {
        let blank = self.blank();
        let width = self.screen_width;
        let CursorState { x, y, .. } = *self.get_cursor();
        let row = &mut self.get_active_screen()[y];
        let count = count.min(width - x);
        row.truncate(width - count);
        for _ in 0..count {
            row.insert(x, blank.clone());
        }
        repair_wide_characters(row);
    }

    /// DCH: deletes `count` characters at the cursor, pulling the rest of the line left and
    /// filling the right end with blanks.
    pub fn delete_characters(&mut self, count: usize) {
        let blank = self.blank();
        let width = self.screen_width;
        let CursorState { x, y, .. } = *self.get_cursor();
        let row = &mut self.get_active_screen()[y];
        let count = count.min(width - x);
        row.drain(x..x + count);
        row.resize(width, blank);
        repair_wide_characters(row);
    }

    /// Blanks the cells `from..to` of line `y`.
    pub fn erase_cells(&mut self, y: usize, from: usize, to: usize) {
        let blank = self.blank();
        let row = &mut self.get_active_screen()[y];
        let to = to.min(row.len());
        if from < to {
            row[from..to].fill(blank);
            repair_wide_characters(row);
        }
    }

    /// Switches to the alternate screen, which full-screen programs draw on.
    pub fn switch_to_alternate_screen(&mut self) {
        self.use_alternate_screen = true;
    }

    /// Switches back to the primary screen, whose content was kept.
    pub fn switch_to_primary_screen(&mut self) {
        self.use_alternate_screen = false;
    }

    pub fn clear_alternate_screen(&mut self) {
        self.alternate_buffer.clear_screen();
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        self.joinable = None;
        let (new_width, new_height) = (new_width.max(1), new_height.max(1));
        self.screen_width = new_width;
        self.screen_height = new_height;
        self.primary_buffer.resize(new_width, new_height);
        self.alternate_buffer.resize(new_width, new_height);
        self.scroll_region = ScrollRegion {
            top: 0,
            bottom: new_height - 1,
        };
        self.tab_stops = default_tab_stops(new_width);
        self.wrap_pending = false;
    }

    pub fn clear_screen(&mut self) {
        let blank = self.blank();
        for row in self.get_active_screen() {
            row.fill(blank.clone());
        }
    }

    /// Whether the program asked for mouse events.
    pub fn is_mouse_reporting_enabled(&self) -> bool {
        self.mode.mouse_tracking != MouseTracking::Off
    }

    /// The bytes reporting a mouse event to the program, in the tracking mode and encoding it
    /// chose; `None` when it did not ask for this kind of event. `x` and `y` are 0-based cells
    /// of the screen. Without SGR encoding positions past column or line 223 cannot be
    /// reported.
    pub fn generate_mouse_report(
        &self,
        event_type: MouseEventType,
        x: usize,
        y: usize,
        button: Option<MouseButton>,
        modifiers: MouseModifiers,
    ) -> Option<String> {
        let tracking = self.mode.mouse_tracking;
        let wanted = match event_type {
            MouseEventType::Press | MouseEventType::Wheel => tracking != MouseTracking::Off,
            MouseEventType::Release => !matches!(tracking, MouseTracking::Off | MouseTracking::X10),
            MouseEventType::Motion if button.is_some() => matches!(
                tracking,
                MouseTracking::ButtonEvent | MouseTracking::AnyEvent
            ),
            MouseEventType::Motion => tracking == MouseTracking::AnyEvent,
        };
        if !wanted {
            return None;
        }
        let mut code: u32 = match button {
            Some(MouseButton::Left) => 0,
            Some(MouseButton::Middle) => 1,
            Some(MouseButton::Right) => 2,
            Some(MouseButton::WheelUp) => 64,
            Some(MouseButton::WheelDown) => 65,
            Some(MouseButton::WheelLeft) => 66,
            Some(MouseButton::WheelRight) => 67,
            // Motion with no button held.
            None => 3,
        };
        if tracking != MouseTracking::X10 {
            if modifiers.shift {
                code += 4;
            }
            if modifiers.alt || modifiers.meta {
                code += 8;
            }
            if modifiers.ctrl {
                code += 16;
            }
        }
        if event_type == MouseEventType::Motion {
            code += 32;
        }
        let (column, line) = (x + 1, y + 1);
        if self.mode.mouse_sgr_mode {
            let end = if event_type == MouseEventType::Release {
                'm'
            } else {
                'M'
            };
            return Some(format!("\x1b[<{};{};{}{}", code, column, line, end));
        }
        // The legacy encoding has no release per button: a release is button 3.
        if event_type == MouseEventType::Release {
            code = (code & !3) | 3;
        }
        let byte =
            |value: usize| char::from_u32(u32::try_from(value + 32).ok()?).filter(|_| value <= 223);
        Some(format!(
            "\x1b[M{}{}{}",
            char::from_u32(code + 32)?,
            byte(column)?,
            byte(line)?
        ))
    }
}

fn default_tab_stops(width: usize) -> Vec<bool> {
    (0..width).map(|x| x > 0 && x % 8 == 0).collect()
}

/// After cells moved or were overwritten: a wide character whose right half is gone, or a
/// right half whose character is gone, becomes a blank.
fn repair_wide_characters(row: &mut [TerminalCell]) {
    for x in 0..row.len() {
        let wide = row[x].glyph.width() == 2;
        if wide && row.get(x + 1).map(|c| &c.glyph) != Some(&WIDE_CONTINUATION) {
            row[x].glyph = Glyph::SPACE;
        }
        let orphan = row[x].glyph == WIDE_CONTINUATION && (x == 0 || row[x - 1].glyph.width() != 2);
        if orphan {
            row[x].glyph = Glyph::SPACE;
        }
    }
}
