//! Output to the terminal: entering and leaving full-screen mode, drawing frames as the
//! difference to the previous frame, the window title, and the clipboard.

use crate::render::screen::{Attrs, ScreenBuffer, Style};
use crossterm::{cursor, event, queue, terminal};
use std::io::{self, BufWriter, Stdout, Write};

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
            event::EnableBracketedPaste,
            cursor::Hide,
            terminal::Clear(terminal::ClearType::All)
        )?;
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
    let _ = crossterm::execute!(
        out,
        event::DisableMouseCapture,
        event::DisableBracketedPaste,
        cursor::Show,
        terminal::LeaveAlternateScreen
    );
    let _ = terminal::disable_raw_mode();
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

impl Display for TerminalDisplay {
    fn size(&self) -> (u16, u16) {
        self.size
    }

    fn set_size(&mut self, cols: u16, rows: u16) {
        self.size = (cols, rows);
    }

    fn present(&mut self, frame: &ScreenBuffer, previous: Option<&ScreenBuffer>) -> io::Result<()> {
        let previous = previous.filter(|p| p.width == frame.width && p.height == frame.height);
        // The style the terminal is in; unknown at the start of each frame.
        let mut style: Option<Style> = None;
        for y in 0..frame.height {
            let changed = |x: usize| previous.is_none_or(|p| p.get(x, y) != frame.get(x, y));
            let mut x = 0;
            while x < frame.width {
                if !changed(x) {
                    x += 1;
                    continue;
                }
                // A changed right half of a wide character is redrawn with its left half.
                if frame.get(x, y).is_some_and(|c| c.is_continuation()) && x > 0 {
                    x -= 1;
                }
                queue!(self.out, cursor::MoveTo(x as u16, y as u16))?;
                while x < frame.width
                    && (changed(x) || frame.get(x, y).is_some_and(|c| c.is_continuation()))
                {
                    let cell = frame.get(x, y).expect("x and y are inside the frame");
                    if !cell.is_continuation() {
                        set_style(&mut self.out, style, cell.style)?;
                        style = Some(cell.style);
                        write!(self.out, "{}", cell.ch)?;
                    }
                    x += 1;
                }
            }
        }
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

    fn copy_to_clipboard(&mut self, text: &str) -> io::Result<()> {
        write!(
            self.out,
            "\x1b]52;c;{}\x07",
            crate::base64::encode(text.as_bytes())
        )?;
        self.out.flush()
    }

    fn reassert_modes(&mut self) -> io::Result<()> {
        crate::platform::ensure_raw_mode();
        queue!(
            self.out,
            event::EnableMouseCapture,
            event::EnableBracketedPaste
        )?;
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
