//! The runtime: one thread owns the application state and reacts to events from the input
//! reader, signal listener, control socket and running scripts; it starts and stops runs and
//! draws frames.

pub mod display;
pub mod event;
pub mod executor;
pub mod keys;
pub mod pty;

mod confirm;
mod control;
mod input;
mod runs;
#[cfg(test)]
mod tests;

use crate::model::bounds::{Bounds, InputBounds};
use crate::model::common::ExecutionMode;
use crate::model::stream::{RunId, RunOrigin};
use crate::platform::ProcessSignal;
use crate::render::screen::ScreenBuffer;
use crate::runtime::display::Display;
use crate::runtime::event::Event;
use crate::runtime::executor::{Executor, RunSpec};
use crate::runtime::runs::PendingTab;
use crate::AppContext;
use crossbeam_channel::{Receiver, Sender};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, Instant};

/// How long a notice stays at the bottom of the screen.
const NOTICE_DURATION: Duration = Duration::from_secs(5);

/// A message shown on the last screen row for a few seconds.
#[derive(Debug, Clone)]
pub struct Notice {
    pub text: String,
    pub is_error: bool,
    pub until: Instant,
}

/// A mouse drag in progress.
#[derive(Debug, Clone)]
enum Drag {
    /// Resizing a box from its bottom-right corner.
    Resize {
        box_id: String,
        start: (u16, u16),
        original: InputBounds,
        parent: Bounds,
    },
    /// Moving a box by its title row.
    Move {
        box_id: String,
        start: (u16, u16),
        original: InputBounds,
        parent: Bounds,
    },
    /// Dragging a scrollbar knob.
    Knob {
        box_id: String,
        vertical: bool,
        start: (u16, u16),
        /// The content offset when the drag started.
        start_offset: usize,
    },
    /// The press went to a PTY program; motion and release go there too.
    Terminal { box_id: String },
}

/// What a periodic refresh does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RefreshAction {
    /// Runs the box's script.
    Script,
    /// Redraws the box (plugin content).
    Redraw,
}

/// Settings of the periodic refresh of one box.
#[derive(Debug, Clone)]
struct Refresh {
    action: RefreshAction,
    interval: Option<Duration>,
    next_due: Option<Instant>,
    paused: bool,
}

pub struct Runtime<D: Display> {
    ctx: AppContext,
    display: D,
    events: Receiver<Event>,
    executor: Executor,
    /// Periodic refresh state of box scripts, by box id.
    refresh: HashMap<String, Refresh>,
    /// What to run again when a run that is being restarted finishes, by stream id.
    restart_after_stop: HashMap<String, RunSpec>,
    /// The last spec started for each stream, so `restart_pty_process` can run it again.
    last_spec: HashMap<String, RunSpec>,
    /// Text runs that have printed nothing yet and so have no tab: the tab each opens when its
    /// first output arrives.
    untabbed: HashMap<RunId, PendingTab>,
    pointer: Option<(u16, u16)>,
    button_down: bool,
    drag: Option<Drag>,
    notice: Option<Notice>,
    /// The confirmation shown over the layout, which takes the keyboard and the mouse.
    confirmation: Option<confirm::Confirmation>,
    /// The frame currently on screen; `None` forces a complete redraw.
    frame: Option<ScreenBuffer>,
    dirty: bool,
    last_present: Option<Instant>,
    title: Option<String>,
    quit: bool,
}

impl<D: Display> Runtime<D> {
    /// `events` and `sender` are the two ends of the channel every background thread reports
    /// to. `socket_name`, when set, is exported to scripts as `BOXMUX_SOCKET`.
    pub fn new(
        ctx: AppContext,
        display: D,
        sender: Sender<Event>,
        events: Receiver<Event>,
        socket_name: Option<String>,
    ) -> Self {
        let base_dir = ctx
            .yaml_file_path
            .as_ref()
            .and_then(|p| p.parent().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("."));
        let libs = ctx
            .app
            .libs
            .iter()
            .flatten()
            .map(|lib| {
                let path = PathBuf::from(lib);
                if path.is_absolute() {
                    path
                } else {
                    base_dir.join(path)
                }
            })
            .collect();
        let mut environment = Vec::new();
        if let Some(socket) = socket_name {
            environment.push(("BOXMUX_SOCKET".to_string(), socket));
        }
        let executor = Executor::new(sender.clone(), ctx.app.shell, libs, environment);
        let mut runtime = Runtime {
            ctx,
            display,
            events,
            executor,
            refresh: HashMap::new(),
            restart_after_stop: HashMap::new(),
            last_spec: HashMap::new(),
            untabbed: HashMap::new(),
            pointer: None,
            button_down: false,
            drag: None,
            notice: None,
            confirmation: None,
            frame: None,
            dirty: true,
            last_present: None,
            title: None,
            quit: false,
        };
        // Laid out before anything runs, so each terminal box's program starts at its size. A
        // terminal that cannot say its size fails the first frame, which asks again.
        if let Err(e) = runtime.fit_screen() {
            log::warn!("cannot read the terminal's size: {}", e);
        }
        runtime
    }

    /// Lays the app out for the size the terminal has now, asked of the terminal itself. When
    /// that size differs from the one the app was laid out for, the boxes are placed anew, each
    /// terminal box's program is given its box's new size, and the next frame is drawn whole.
    /// Whether the terminal has cells to draw on.
    fn fit_screen(&mut self) -> std::io::Result<bool> {
        let (cols, rows) = self.display.size()?;
        if cols == 0 || rows == 0 {
            return Ok(false);
        }
        let resized = self
            .ctx
            .app
            .set_screen_size(cols, rows)
            .map_err(std::io::Error::other)?;
        if resized {
            self.frame = None;
            self.resize_terminals();
        }
        Ok(true)
    }

    pub fn context(&self) -> &AppContext {
        &self.ctx
    }

    pub fn display(&self) -> &D {
        &self.display
    }

    /// Runs until the user quits or a termination signal arrives. Every process started by
    /// boxmux is stopped before this returns.
    pub fn run(mut self) -> Result<(), String> {
        self.activate_layout_runs();
        while !self.quit {
            let timeout = self.next_wakeup().saturating_duration_since(Instant::now());
            match self.events.recv_timeout(timeout) {
                Ok(event) => {
                    self.handle(event);
                    while let Ok(event) = self.events.try_recv() {
                        self.handle(event);
                        if self.quit {
                            break;
                        }
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {}
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    return Err("all event sources stopped".to_string());
                }
            }
            self.tick(Instant::now());
        }
        Ok(())
    }

    /// The next moment the loop must wake up without an event: a due refresh, a notice to
    /// remove, or a frame to draw once the frame delay has passed.
    fn next_wakeup(&self) -> Instant {
        let now = Instant::now();
        let mut wake = now + Duration::from_secs(1);
        let shown = self.shown_box_ids();
        for (box_id, refresh) in &self.refresh {
            if let Some(due) = refresh.next_due.filter(|_| !refresh.paused) {
                if shown.contains(box_id) {
                    wake = wake.min(due);
                }
            }
        }
        if let Some(notice) = &self.notice {
            wake = wake.min(notice.until);
        }
        if self.dirty {
            let frame_ready = self
                .last_present
                .map(|t| t + Duration::from_millis(self.ctx.config.frame_delay))
                .unwrap_or(now);
            wake = wake.min(frame_ready);
        }
        wake
    }

    /// Work that happens on time rather than on events: due refreshes, expired notices, and
    /// drawing a frame when something changed.
    pub fn tick(&mut self, now: Instant) {
        self.run_due_refreshes(now);
        if self.notice.as_ref().is_some_and(|n| n.until <= now) {
            self.notice = None;
            self.dirty = true;
        }
        let frame_due = self
            .last_present
            .is_none_or(|t| now >= t + Duration::from_millis(self.ctx.config.frame_delay));
        if self.dirty && frame_due {
            self.render();
        }
    }

    pub fn handle(&mut self, event: Event) {
        match event {
            Event::Input(input) => self.handle_input(input),
            Event::InputFailed(reason) => {
                log::error!("reading the terminal failed: {}", reason);
                self.quit = true;
            }
            Event::Signal(ProcessSignal::Terminate) => self.quit = true,
            Event::Signal(ProcessSignal::Suspend) => self.suspend(),
            Event::Signal(ProcessSignal::Resume) => {}
            Event::Socket { function, reply } => {
                let answer = self.handle_socket(function);
                // The client may have given up waiting; there is nobody left to tell.
                let _ = reply.send(answer);
                self.close_stale_confirmation();
            }
            Event::Run(run_event) => self.handle_run_event(run_event),
        }
    }

    pub(super) fn suspend(&mut self) {
        if let Err(e) = self.display.suspend() {
            log::error!("cannot restore the terminal before suspending: {}", e);
        }
        crate::platform::suspend_self();
        // Execution continues here after SIGCONT.
        if let Err(e) = self.display.resume() {
            log::error!("cannot re-enter full-screen mode: {}", e);
            self.quit = true;
            return;
        }
        self.redraw_whole();
    }

    pub fn notify(&mut self, text: impl Into<String>, is_error: bool) {
        let text = text.into();
        if is_error {
            log::warn!("{}", text);
        }
        self.notice = Some(Notice {
            text,
            is_error,
            until: Instant::now() + NOTICE_DURATION,
        });
        self.dirty = true;
    }

    /// Draws the next frame whole, every cell of it. After the terminal is resized, even back to
    /// the size of the last frame, or given back by another program, what it shows is not that
    /// frame: a terminal may move, keep or blank rows as it resizes.
    fn redraw_whole(&mut self) {
        self.frame = None;
        self.dirty = true;
    }

    /// Matches every PTY's size to the content area of the box it is shown in, in that box's own
    /// layout, shown or not: a terminal in a hidden layout is the size it will be shown at.
    fn resize_terminals(&mut self) {
        let runs: Vec<_> = self
            .executor
            .runs()
            .filter(|(_, run)| run.pty().is_some())
            .map(|(id, run)| (*id, run.target_box.clone()))
            .collect();
        for (run, box_id) in runs {
            let Some(size) = self
                .ctx
                .app
                .box_bounds(&box_id)
                .as_ref()
                .and_then(terminal_size)
            else {
                continue;
            };
            if let Some(session) = self.executor.get_mut(run).and_then(|r| r.pty_mut()) {
                if let Err(e) = session.resize(size) {
                    log::warn!("{}", e);
                }
            }
        }
    }

    /// The views of the shown layout's boxes as they would be drawn now. Under an open
    /// confirmation no box is hovered.
    pub(crate) fn views(&mut self) -> Vec<crate::render::box_view::BoxView> {
        let bounds = self.ctx.app.active_bounds();
        let pointer = self.pointer.filter(|_| self.confirmation.is_none());
        let hovered_box = pointer.and_then(|(x, y)| {
            let layout = self.ctx.app.get_active_layout()?;
            crate::render::box_at(layout, &bounds, usize::from(x), usize::from(y))
        });
        crate::render::views(&self.ctx, &bounds, hovered_box.as_deref())
    }

    /// Draws a frame if anything changed since the last one.
    fn render(&mut self) {
        match self.fit_screen() {
            Ok(true) => {}
            // Nothing can be drawn until the terminal has cells again.
            Ok(false) => return,
            Err(e) => {
                log::error!("cannot read the terminal's size: {}", e);
                self.quit = true;
                return;
            }
        }
        let screen = self.ctx.app.screen();
        if let Err(e) = self.display.reassert_modes() {
            log::warn!("cannot re-establish terminal modes: {}", e);
        }
        let views = self.views();
        let hover = self
            .pointer
            .filter(|_| self.confirmation.is_none())
            .and_then(|(x, y)| {
                let (x, y) = (usize::from(x), usize::from(y));
                let view = crate::render::view_at(&views, x, y)?;
                Some((view.id.as_str(), view.hit(x, y)?))
            });
        let mut buffer =
            crate::render::draw(&self.ctx, &views, screen.width(), screen.height(), hover);
        if self.ctx.config.calibrate {
            if let Some((x, y)) = self.pointer {
                if let Some(cell) = buffer.get(usize::from(x), usize::from(y)) {
                    let mut style = cell.style;
                    style.bg = crate::model::appearance::ColorName::Red.color();
                    buffer.restyle(usize::from(x), usize::from(y), style);
                }
            }
        }
        if let Some(confirmation) = &self.confirmation {
            if let Some(view) = self.confirmation_view() {
                view.draw(&mut buffer, confirmation.focused, self.hovered_answer());
            }
        }
        if let Some(notice) = &self.notice {
            crate::render::draw_notice(&mut buffer, &notice.text, notice.is_error);
        }
        if let Err(e) = self.display.present(&buffer, self.frame.as_ref()) {
            log::error!("cannot draw to the terminal: {}", e);
            self.quit = true;
            return;
        }
        let title = self
            .ctx
            .app
            .get_active_layout()
            .and_then(|layout| layout.title.clone())
            .filter(|title| !title.trim().is_empty());
        if title != self.title {
            if let Some(title) = &title {
                if let Err(e) = self.display.set_title(title) {
                    log::warn!("cannot set the terminal title: {}", e);
                }
            }
            self.title = title;
        }
        self.frame = Some(buffer);
        self.last_present = Some(Instant::now());
        self.dirty = false;
    }

    /// What the periodic refresh of `box_id` does and how often, from the configuration: a box
    /// with a script runs it (once, when there is no interval, and always once for a PTY
    /// script, which keeps running); a plugin box with an interval is redrawn. `None` for a box
    /// with neither.
    fn refresh_settings(&self, box_id: &str) -> Option<(RefreshAction, Option<Duration>)> {
        let muxbox = self.ctx.app.get_muxbox_by_id(box_id)?;
        let interval = self
            .ctx
            .app
            .refresh_interval(box_id)
            .map(Duration::from_millis);
        if muxbox.script.is_some() {
            let interval = interval.filter(|_| muxbox.execution_mode != ExecutionMode::Pty);
            Some((RefreshAction::Script, interval))
        } else if muxbox.plugin_component.is_some() && interval.is_some() {
            Some((RefreshAction::Redraw, interval))
        } else {
            None
        }
    }

    /// The ids of the boxes of the shown layout, in configuration order. Only their refreshes
    /// run; the others wait until their layout is shown.
    fn shown_box_ids(&self) -> Vec<String> {
        self.ctx
            .app
            .get_active_layout()
            .map(|layout| {
                layout
                    .get_all_muxboxes()
                    .iter()
                    .map(|muxbox| muxbox.id.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Schedules the scripts and plugin redraws of the shown layout that are not scheduled yet,
    /// and runs what is due.
    fn activate_layout_runs(&mut self) {
        let now = Instant::now();
        for box_id in self.shown_box_ids() {
            if self.refresh.contains_key(&box_id) {
                continue;
            }
            if let Some((action, interval)) = self.refresh_settings(&box_id) {
                self.refresh.insert(
                    box_id,
                    Refresh {
                        action,
                        interval,
                        next_due: Some(now),
                        paused: false,
                    },
                );
            }
        }
        self.run_due_refreshes(now);
    }

    /// Starts every box script whose refresh is due and that is not still running from the
    /// previous refresh.
    fn run_due_refreshes(&mut self, now: Instant) {
        let shown = self.shown_box_ids();
        let due: Vec<String> = self
            .refresh
            .iter()
            .filter(|(id, r)| {
                shown.contains(*id) && !r.paused && r.next_due.is_some_and(|due| due <= now)
            })
            .map(|(id, _)| id.clone())
            .collect();
        for box_id in due {
            let action = self.refresh.get(&box_id).map(|r| r.action);
            match action {
                Some(RefreshAction::Script) => {
                    let origin = RunOrigin::BoxScript {
                        box_id: box_id.clone(),
                    };
                    let busy = self.executor.run_for_origin(&origin).is_some();
                    if !busy {
                        if let Err(e) = self.start_box_script(&box_id, false) {
                            self.notify(format!("box '{}': {}", box_id, e), true);
                        }
                    }
                }
                Some(RefreshAction::Redraw) => self.dirty = true,
                None => continue,
            }
            if let Some(refresh) = self.refresh.get_mut(&box_id) {
                refresh.next_due = refresh.interval.map(|interval| {
                    // Keep the cadence; skip intervals missed while the script was busy.
                    let mut next = refresh.next_due.unwrap_or(now) + interval;
                    while next <= now {
                        next += interval;
                    }
                    next
                });
            }
        }
    }
}

/// The terminal size of a PTY shown in a box at `bounds`: the whole inside of the border.
fn terminal_size(bounds: &Bounds) -> Option<crate::runtime::pty::TerminalSize> {
    let inner = bounds.shrink(1, 1)?;
    Some(crate::runtime::pty::TerminalSize {
        rows: u16::try_from(inner.height()).unwrap_or(u16::MAX),
        cols: u16::try_from(inner.width()).unwrap_or(u16::MAX),
    })
}
