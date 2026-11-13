//! Output to the terminal: entering and leaving full-screen mode, drawing frames as the
//! difference to the previous frame with their hyperlinks, the window title, and the clipboard.

use crate::link::{Link, CLOSE};
use crate::render::screen::{Attrs, ScreenBuffer, Style};
use crossterm::{cursor, event, queue, terminal};
use std::io::{self, BufWriter, Stdout, Write};
use std::sync::Arc;

/// Where the runtime draws. The terminal implementation is [`TerminalDisplay`]; tests use an
/// in-memory one.
pub trait Display {
    /// Columns and rows.
    fn size(&self) -> (u16, u16);
    /// Records a new terminal size reported by a resize event.
    fn set_size(&mut self, cols: u16, rows: u16);
    /// Shows `frame`. `previous` is what is currently on screen, or `None` when the screen must
    /// be redrawn completely.
    fn present(&mut self, frame: &ScreenBuffer, previous: Option<&ScreenBuffer>) -> io::Result<()>;
    fn set_title(&mut self, title: &str) -> io::Result<()>;
    /// Puts text on the system clipboard of the terminal (OSC 52).
    fn copy_to_clipboard(&mut self, text: &str) -> io::Result<()>;
    /// Re-establishes raw mode and mouse capture after a child process may have reset them.
    fn reassert_modes(&mut self) -> io::Result<()>;
    /// Returns the terminal to normal mode before the process stops (Ctrl+Z).
    fn suspend(&mut self) -> io::Result<()>;
    /// Enters full-screen mode again after the process resumed.
    fn resume(&mut self) -> io::Result<()>;
}

pub struct TerminalDisplay {
    out: BufWriter<Stdout>,
    size: (u16, u16),
}

impl TerminalDisplay {
    /// Switches the terminal to the alternate screen, raw mode and mouse capture. The terminal
    /// is restored when the display is dropped, and by the panic hook installed here.
    pub fn enter() -> io::Result<TerminalDisplay> {
        install_panic_hook();
        let size = terminal::size()?;
        let mut display = TerminalDisplay {
            out: BufWriter::with_capacity(1 << 16, io::stdout()),
            size,
        };
        display.enter_full_screen()?;
        Ok(display)
    }

    fn enter_full_screen(&mut self) -> io::Result<()> {
        terminal::enable_raw_mode()?;
        queue!(
            self.out,
            terminal::EnterAlternateScreen,
            event::EnableMouseCapture,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )?;
        enable_bracketed_paste(&mut self.out)?;
        self.out.flush()
    }
}

impl Drop for TerminalDisplay {
    fn drop(&mut self) {
        let _ = self.out.flush();
        restore_terminal();
    }
}

/// Leaves full-screen mode: shows the cursor, stops mouse capture, leaves the alternate screen
/// and raw mode. Safe to call more than once.
pub fn restore_terminal() {
    let mut out = io::stdout();
    #[cfg(unix)]
    let _ = crossterm::execute!(out, event::DisableBracketedPaste);
    let _ = crossterm::execute!(
        out,
        event::DisableMouseCapture,
        cursor::Show,
        terminal::LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
}

/// Asks the terminal to deliver pastes as one piece (Unix). The Windows console has no such
/// mode: pasted text arrives as key presses there.
fn enable_bracketed_paste(out: &mut impl Write) -> io::Result<()> {
    #[cfg(unix)]
    queue!(out, event::EnableBracketedPaste)?;
    #[cfg(not(unix))]
    let _ = out;
    Ok(())
}

fn install_panic_hook() {
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // Restore first so the panic message is readable in a normal terminal.
        restore_terminal();
        default_hook(info);
    }));
}

/// Switches the terminal from style `from` (`None` when unknown) to `to`.
fn set_style(out: &mut impl Write, from: Option<Style>, to: Style) -> io::Result<()> {
    use crossterm::style::{Attribute, SetAttribute, SetBackgroundColor, SetForegroundColor};
    if from.map(|f| f.attrs) != Some(to.attrs) {
        // Attributes cannot be turned off one by one portably: reset, then set the wanted ones,
        // which also resets the colors.
        queue!(out, SetAttribute(Attribute::Reset))?;
        for (flag, attribute) in [
            (Attrs::BOLD, Attribute::Bold),
            (Attrs::DIM, Attribute::Dim),
            (Attrs::ITALIC, Attribute::Italic),
            (Attrs::UNDERLINE, Attribute::Underlined),
            (Attrs::BLINK, Attribute::SlowBlink),
            (Attrs::REVERSE, Attribute::Reverse),
            (Attrs::HIDDEN, Attribute::Hidden),
            (Attrs::STRIKETHROUGH, Attribute::CrossedOut),
        ] {
            if to.attrs.contains(flag) {
                queue!(out, SetAttribute(attribute))?;
            }
        }
        queue!(out, SetForegroundColor(to.fg), SetBackgroundColor(to.bg))?;
        return Ok(());
    }
    if from.map(|f| f.fg) != Some(to.fg) {
        queue!(out, SetForegroundColor(to.fg))?;
    }
    if from.map(|f| f.bg) != Some(to.bg) {
        queue!(out, SetBackgroundColor(to.bg))?;
    }
    Ok(())
}

/// Writes what turns a terminal showing `previous` (`None` when unknown) into one showing
/// `frame`: each run of changed cells, from its first cell.
///
/// A cell that is part of a hyperlink is written inside that link's OSC 8 sequence
/// ([`crate::link`]): the link is opened before the first such cell written and closed before a
/// cell that is not part of it, and at the end of the frame, so text BoxMux writes later is never
/// a link by mistake. Every cell of a link, on every line, is written with the same id, so the
/// terminal takes them for one link.
///
/// Terminals do not all give a cluster the width [`crate::width`] does: some draw an emoji
/// variation sequence or an emoji their Unicode tables do not know in one column. So a cell
/// holding a two-column cluster or a cluster of several characters is blanked first, the
/// cluster written over the blanks, and the cursor placed after it by position: a terminal
/// that draws it narrower shows a blank where the rest of it would be, and every cell after it
/// stays in its column.
pub fn write_frame(
    out: &mut impl Write,
    frame: &ScreenBuffer,
    previous: Option<&ScreenBuffer>,
) -> io::Result<()> {
    let previous = previous.filter(|p| p.width == frame.width && p.height == frame.height);
    // The style the terminal is in; unknown at the start of each frame.
    let mut style: Option<Style> = None;
    // The link the terminal is in; none at the start of each frame, which ends with none.
    let mut link: Option<&Arc<Link>> = None;
    for y in 0..frame.height {
        let changed = |x: usize| previous.is_none_or(|p| p.get(x, y) != frame.get(x, y));
        let continuation = |x: usize| frame.get(x, y).is_some_and(|c| c.is_continuation());
        // The terminal's cursor column on this row, when it is known.
        let mut cursor: Option<usize> = None;
        let mut x = 0;
        while x < frame.width {
            if !changed(x) {
                x += 1;
                continue;
            }
            // A changed right half of a wide cluster is redrawn with its left half.
            if continuation(x) && x > 0 {
                x -= 1;
            }
            while x < frame.width && (changed(x) || continuation(x)) {
                let cell = frame.get(x, y).expect("x and y are inside the frame");
                if cell.is_continuation() {
                    x += 1;
                    continue;
                }
                if cursor != Some(x) {
                    queue!(out, cursor::MoveTo(x as u16, y as u16))?;
                }
                set_style(out, style, cell.style)?;
                style = Some(cell.style);
                if link != cell.link.as_ref() {
                    if link.is_some() {
                        write!(out, "{}", CLOSE)?;
                    }
                    if let Some(opened) = &cell.link {
                        write!(out, "{}", opened.open_sequence())?;
                    }
                    link = cell.link.as_ref();
                }
                let width = cell.glyph.width();
                if width == 2 || cell.glyph.is_sequence() {
                    write!(out, "{}", " ".repeat(width))?;
                    queue!(out, cursor::MoveTo(x as u16, y as u16))?;
                    write!(out, "{}", cell.glyph)?;
                    cursor = None;
                } else {
                    write!(out, "{}", cell.glyph)?;
                    cursor = Some(x + 1);
                }
                x += 1;
            }
        }
    }
    if link.is_some() {
        write!(out, "{}", CLOSE)?;
    }
    Ok(())
}

impl Display for TerminalDisplay {
    fn size(&self) -> (u16, u16) {
        self.size
    }

    fn set_size(&mut self, cols: u16, rows: u16) {
        self.size = (cols, rows);
    }

    fn present(&mut self, frame: &ScreenBuffer, previous: Option<&ScreenBuffer>) -> io::Result<()> {
        write_frame(&mut self.out, frame, previous)?;
        queue!(
            self.out,
            crossterm::style::SetAttribute(crossterm::style::Attribute::Reset)
        )?;
        self.out.flush()
    }

    fn set_title(&mut self, title: &str) -> io::Result<()> {
        queue!(self.out, terminal::SetTitle(title))?;
        self.out.flush()
    }

    /// Writes `ESC ] 52 ; c ; <base64 of the UTF-8 text> ST`, ended with ST as OSC 8 links
    /// are ([`crate::link`]).
    fn copy_to_clipboard(&mut self, text: &str) -> io::Result<()> {
        write!(
            self.out,
            "\x1b]52;c;{}\x1b\\",
            crate::base64::encode(text.as_bytes())
        )?;
        self.out.flush()
    }

    fn reassert_modes(&mut self) -> io::Result<()> {
        crate::platform::ensure_raw_mode();
        queue!(self.out, event::EnableMouseCapture)?;
        enable_bracketed_paste(&mut self.out)?;
        self.out.flush()
    }

    fn suspend(&mut self) -> io::Result<()> {
        self.out.flush()?;
        restore_terminal();
        Ok(())
    }

    fn resume(&mut self) -> io::Result<()> {
        self.size = terminal::size()?;
        self.enter_full_screen()
    }
}

/// A display that keeps what would be shown in memory: the last frame, the title and the
/// clipboard. Used to run the app without a terminal, for example in tests.
#[derive(Debug)]
pub struct MemoryDisplay {
    size: (u16, u16),
    pub frame: Option<ScreenBuffer>,
    pub frames_presented: usize,
    pub title: Option<String>,
    pub clipboard: Option<String>,
}

impl MemoryDisplay {
    pub fn new(cols: u16, rows: u16) -> MemoryDisplay {
        MemoryDisplay {
            size: (cols, rows),
            frame: None,
            frames_presented: 0,
            title: None,
            clipboard: None,
        }
    }

    /// The text of each row of the last frame, without colors.
    pub fn rows(&self) -> Vec<String> {
        self.frame
            .as_ref()
            .map(ScreenBuffer::text_rows)
            .unwrap_or_default()
    }
}

impl Display for MemoryDisplay {
    fn size(&self) -> (u16, u16) {
        self.size
    }

    fn set_size(&mut self, cols: u16, rows: u16) {
        self.size = (cols, rows);
    }

    fn present(
        &mut self,
        frame: &ScreenBuffer,
        _previous: Option<&ScreenBuffer>,
    ) -> io::Result<()> {
        self.frame = Some(frame.clone());
        self.frames_presented += 1;
        Ok(())
    }

    fn set_title(&mut self, title: &str) -> io::Result<()> {
        self.title = Some(title.to_string());
        Ok(())
    }

    fn copy_to_clipboard(&mut self, text: &str) -> io::Result<()> {
        self.clipboard = Some(text.to_string());
        Ok(())
    }

    fn reassert_modes(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn suspend(&mut self) -> io::Result<()> {
        Ok(())
    }

    fn resume(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::Emulator;
    use crate::width::Glyph;

    /// The link of each cell of row `y` of a terminal that was sent `bytes`.
    fn links_shown(bytes: &[u8], width: usize, height: usize, y: usize) -> Vec<Option<String>> {
        let mut terminal = Emulator::with_screen_size(width, height);
        terminal.process_bytes(bytes);
        assert_eq!(
            terminal.terminal_state.hyperlink, None,
            "the frame leaves the terminal in a link"
        );
        terminal.terminal_state.primary_buffer.content[y]
            .iter()
            .map(|cell| cell.link.as_ref().map(|link| link.id().to_string()))
            .collect()
    }

    /// A 6 x 2 frame: `ab日` at the end of the first row and `cd` at the start of the second
    /// are the link `x`, the rest is no link.
    fn linked_frame() -> ScreenBuffer {
        let link = Arc::new(Link::new("https://example.com", Some("x")));
        let mut frame = ScreenBuffer::new(6, 2, Style::default());
        frame.put_str(0, 0, "on", Style::default(), 5);
        for (x, text) in [(2, "a"), (3, "b"), (4, "日")] {
            frame.put(x, 0, &Glyph::new(text), Style::default(), Some(&link), 5);
        }
        for (x, text) in [(0, "c"), (1, "d")] {
            frame.put(x, 1, &Glyph::new(text), Style::default(), Some(&link), 5);
        }
        frame.put_str(2, 1, "off", Style::default(), 5);
        frame
    }

    #[test]
    fn linked_cells_are_written_inside_their_link_and_the_frame_ends_outside_it() {
        let frame = linked_frame();
        let mut out = Vec::new();
        write_frame(&mut out, &frame, None).unwrap();
        let x = Some("x".to_string());
        assert_eq!(
            links_shown(&out, 6, 2, 0),
            vec![None, None, x.clone(), x.clone(), x.clone(), x.clone()]
        );
        assert_eq!(
            links_shown(&out, 6, 2, 1),
            vec![x.clone(), x, None, None, None, None]
        );
    }

    #[test]
    fn a_redraw_writes_a_link_over_the_same_text_drawn_without_it() {
        let frame = linked_frame();
        let mut plain = ScreenBuffer::new(6, 2, Style::default());
        plain.put_str(0, 0, "onab日", Style::default(), 5);
        plain.put_str(0, 1, "cdoff", Style::default(), 5);
        let mut out = Vec::new();
        write_frame(&mut out, &plain, None).unwrap();
        write_frame(&mut out, &frame, Some(&plain)).unwrap();
        // Only the cells that became a link are written, the last of them `d`: the frame still
        // ends outside the link (`links_shown` checks).
        let x = Some("x".to_string());
        assert_eq!(
            links_shown(&out, 6, 2, 0)[2..],
            [x.clone(), x.clone(), x.clone(), x.clone()]
        );
        assert_eq!(links_shown(&out, 6, 2, 1)[..3], [x.clone(), x, None]);
        // And taken away again by redrawing the plain frame.
        write_frame(&mut out, &plain, Some(&frame)).unwrap();
        assert!(links_shown(&out, 6, 2, 0).iter().all(Option::is_none));
    }
}
