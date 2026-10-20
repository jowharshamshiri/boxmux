//! Keyboard and mouse handling.

use super::keys::{key_matches, key_name, pty_key_bytes};
use super::{Drag, Runtime};
use crate::model::bounds::{Bounds, InputBounds};
use crate::model::stream::RunId;
use crate::render::box_view::{percent_for_offset, BoxHit, BoxView};
use crate::render::scrollbar::ScrollbarHit;
use crate::render::tab_bar::TabHit;
use crate::runtime::display::Display;
use crate::terminal::{MouseButton as TermButton, MouseEventType, MouseModifiers};
use crate::App;
use crossterm::event::{
    Event as TermEvent, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
    MouseEventKind,
};

/// Lines (or columns) one mouse wheel step scrolls.
const WHEEL_STEP: usize = 3;
/// The smallest a box can be resized to, in cells (border plus one content cell).
const MIN_BOX_EXTENT: usize = 3;

impl<D: Display> Runtime<D> {
    pub(super) fn handle_input(&mut self, event: TermEvent) {
        match event {
            // Windows reports releases and repeats too; only presses act.
            TermEvent::Key(key) if key.kind != KeyEventKind::Release => self.handle_key(key),
            TermEvent::Key(_) => {}
            TermEvent::Mouse(mouse) => self.handle_mouse(mouse),
            TermEvent::Resize(cols, rows) => {
                self.display.set_size(cols, rows);
                self.screen_resized();
            }
            // Pasted text goes to the focused terminal; nothing else takes text.
            TermEvent::Paste(text) => {
                if let Some(run) = self.focused_terminal_run() {
                    let bytes = self
                        .executor
                        .get(run)
                        .and_then(|r| r.pty())
                        .map(|session| session.paste_bytes(&text));
                    if let Some(bytes) = bytes {
                        self.write_to_terminal(run, &bytes);
                    }
                }
            }
            TermEvent::FocusGained | TermEvent::FocusLost => {}
        }
    }

    /// The focused box of the shown layout.
    fn focused_box_id(&self) -> Option<String> {
        self.ctx
            .app
            .get_active_layout()?
            .get_selected_muxboxes()
            .first()
            .map(|muxbox| muxbox.id.clone())
    }

    /// The live PTY run shown in the visible tab of `box_id`.
    fn terminal_run_of(&self, box_id: &str) -> Option<RunId> {
        let run = self
            .ctx
            .app
            .get_muxbox_by_id(box_id)?
            .get_selected_stream()?
            .run?;
        self.executor.get(run)?.pty().map(|_| run)
    }

    fn focused_terminal_run(&self) -> Option<RunId> {
        self.terminal_run_of(&self.focused_box_id()?)
    }

    fn write_to_terminal(&mut self, run: RunId, bytes: &[u8]) {
        let result = match self.executor.get(run).and_then(|r| r.pty()) {
            Some(session) => session.write_input(bytes),
            None => return,
        };
        if let Err(e) = result {
            self.notify(e, true);
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        let focus_keys = matches!(key.code, KeyCode::Tab | KeyCode::BackTab);
        // A focused terminal receives every key except the focus keys, which always reach
        // boxmux so focus can leave the terminal.
        if !focus_keys {
            if let Some(run) = self.focused_terminal_run() {
                let application_keys = self
                    .executor
                    .get(run)
                    .and_then(|r| r.pty())
                    .is_some_and(|s| s.application_cursor_keys());
                let bytes = pty_key_bytes(&key, application_keys);
                if !bytes.is_empty() {
                    self.write_to_terminal(run, &bytes);
                }
                return;
            }
        }

        let Some(name) = key_name(&key) else {
            return;
        };
        let plain = key.modifiers.difference(KeyModifiers::SHIFT).is_empty();
        let focused = self.focused_box_id();
        match key.code {
            KeyCode::Char('q') if key.modifiers.is_empty() => {
                self.quit = true;
                return;
            }
            // In raw mode the terminal does not turn Ctrl+Z into a stop signal; boxmux does.
            KeyCode::Char('z') if cfg!(unix) && key.modifiers == KeyModifiers::CONTROL => {
                self.suspend();
                return;
            }
            KeyCode::Char('c')
                if key.modifiers == KeyModifiers::CONTROL
                    || key.modifiers == KeyModifiers::SUPER =>
            {
                self.copy_focused_box();
            }
            KeyCode::Tab => self.move_focus(true),
            KeyCode::BackTab => self.move_focus(false),
            KeyCode::Enter if plain => {
                if let Some(box_id) = &focused {
                    self.activate_selected_choice(box_id);
                }
            }
            KeyCode::Up | KeyCode::Down if plain => {
                if let Some(box_id) = &focused {
                    let down = key.code == KeyCode::Down;
                    if !self.move_choice_selection(box_id, down) {
                        self.scroll_box(box_id, if down { 1 } else { -1 }, 0);
                    }
                }
            }
            KeyCode::Left | KeyCode::Right if plain => {
                if let Some(box_id) = &focused {
                    self.scroll_box(box_id, 0, if key.code == KeyCode::Right { 1 } else { -1 });
                }
            }
            KeyCode::PageUp | KeyCode::PageDown => {
                if let Some(box_id) = &focused {
                    let sign = if key.code == KeyCode::PageDown { 1 } else { -1 };
                    if let Some(body) = self.view_of(box_id).and_then(|v| v.body) {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            let page = body.area.width().saturating_sub(1).max(1) as isize;
                            self.scroll_box(box_id, 0, sign * page);
                        } else {
                            let page = body.area.height().saturating_sub(1).max(1) as isize;
                            self.scroll_box(box_id, sign * page, 0);
                        }
                    }
                }
            }
            KeyCode::Home | KeyCode::End => {
                if let Some(muxbox) = focused
                    .as_deref()
                    .and_then(|id| self.ctx.app.get_muxbox_by_id_mut(id))
                {
                    let edge = if key.code == KeyCode::End { 100.0 } else { 0.0 };
                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                        muxbox.vertical_scroll = Some(edge);
                    } else {
                        muxbox.horizontal_scroll = Some(edge);
                    }
                    self.dirty = true;
                }
            }
            _ => {}
        }

        self.run_key_bindings(&name, focused.as_deref());
    }

    /// Runs the hot keys and `on_keypress` handlers bound to the key `name`.
    fn run_key_bindings(&mut self, name: &str, focused: Option<&str>) {
        let hot_choices: Vec<String> = self
            .ctx
            .app
            .hot_keys
            .iter()
            .flatten()
            .filter(|(configured, _)| key_matches(configured, name))
            .map(|(_, choice)| choice.clone())
            .collect();
        for choice_id in hot_choices {
            let owner = self.ctx.app.get_active_layout().and_then(|layout| {
                layout
                    .get_all_muxboxes()
                    .into_iter()
                    .find(|muxbox| {
                        muxbox
                            .choices
                            .iter()
                            .flatten()
                            .any(|choice| choice.id == choice_id)
                    })
                    .map(|muxbox| muxbox.id.clone())
            });
            match owner {
                Some(box_id) => self.activate_choice(&box_id, &choice_id),
                None => self.notify(
                    format!(
                        "hot key {}: choice '{}' is not in the shown layout",
                        name, choice_id
                    ),
                    true,
                ),
            }
        }

        let app_handlers = matching_handlers(self.ctx.app.on_keypress.as_ref(), name);
        for script in app_handlers {
            self.start_global_keypress("app", name, script);
        }
        let layout_handlers = self.ctx.app.get_active_layout().map(|layout| {
            (
                layout.id.clone(),
                matching_handlers(layout.on_keypress.as_ref(), name),
            )
        });
        if let Some((layout_id, handlers)) = layout_handlers {
            for script in handlers {
                self.start_global_keypress(&format!("layout:{}", layout_id), name, script);
            }
        }
        if let Some(box_id) = focused {
            let handlers = self
                .ctx
                .app
                .get_muxbox_by_id(box_id)
                .filter(|muxbox| muxbox.get_selected_stream_choices().is_none())
                .map(|muxbox| matching_handlers(muxbox.on_keypress.as_ref(), name))
                .unwrap_or_default();
            for script in handlers {
                self.start_box_keypress(box_id, name, script);
            }
        }
    }

    fn move_focus(&mut self, forward: bool) {
        if let Some(layout) = self.ctx.app.get_active_layout_mut() {
            if forward {
                layout.select_next_muxbox();
            } else {
                layout.select_previous_muxbox();
            }
            self.dirty = true;
        }
    }

    fn activate_selected_choice(&mut self, box_id: &str) {
        let selected = self.ctx.app.get_muxbox_by_id(box_id).and_then(|muxbox| {
            let choices = muxbox.get_selected_stream_choices()?;
            choices.iter().find(|c| c.selected).map(|c| c.id.clone())
        });
        if let Some(choice_id) = selected {
            self.activate_choice(box_id, &choice_id);
        }
    }

    /// Moves the menu selection of `box_id` up or down (wrapping) and keeps it in view.
    /// Returns `false` when the visible tab is not a menu.
    fn move_choice_selection(&mut self, box_id: &str, down: bool) -> bool {
        let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(box_id) else {
            return false;
        };
        let Some(choices) = muxbox.get_selected_stream_choices_mut() else {
            return false;
        };
        if choices.is_empty() {
            return true;
        }
        let count = choices.len();
        let current = choices.iter().position(|c| c.selected);
        let next = match (current, down) {
            (Some(i), true) => (i + 1) % count,
            (Some(i), false) => (i + count - 1) % count,
            (None, true) => 0,
            (None, false) => count - 1,
        };
        for (i, choice) in choices.iter_mut().enumerate() {
            choice.selected = i == next;
        }
        if let Some(body) = self.view_of(box_id).and_then(|view| view.body) {
            if let Some(rows) = body.rows_of_choice(next) {
                let height = body.area.height();
                let offset = if rows.start < body.offset_y {
                    rows.start
                } else if rows.end > body.offset_y + height {
                    rows.end.saturating_sub(height)
                } else {
                    body.offset_y
                };
                if let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(box_id) {
                    muxbox.vertical_scroll = Some(percent_for_offset(offset, body.max_offset_y()));
                }
            }
        }
        self.dirty = true;
        true
    }

    /// The view of `box_id` in the shown layout, as it would be drawn now.
    fn view_of(&mut self, box_id: &str) -> Option<BoxView> {
        self.views().into_iter().find(|view| view.id == box_id)
    }

    /// Scrolls the content of `box_id` by whole lines and columns.
    fn scroll_box(&mut self, box_id: &str, lines: isize, columns: isize) {
        let Some(body) = self.view_of(box_id).and_then(|view| view.body) else {
            return;
        };
        let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(box_id) else {
            return;
        };
        if lines != 0 {
            let target = body
                .offset_y
                .saturating_add_signed(lines)
                .min(body.max_offset_y());
            muxbox.vertical_scroll = Some(percent_for_offset(target, body.max_offset_y()));
        }
        if columns != 0 {
            let target = body
                .offset_x
                .saturating_add_signed(columns)
                .min(body.max_offset_x());
            muxbox.horizontal_scroll = Some(percent_for_offset(target, body.max_offset_x()));
        }
        self.dirty = true;
    }

    fn copy_focused_box(&mut self) {
        let Some(text) = self.focused_box_id().and_then(|id| {
            let stream = self.ctx.app.get_muxbox_by_id(&id)?.get_selected_stream()?;
            Some(match &stream.choices {
                Some(choices) => choices
                    .iter()
                    .filter_map(|c| c.content.clone())
                    .collect::<Vec<_>>()
                    .join("\n"),
                None => crate::utils::strip_ansi_codes(&stream.content.join("\n")),
            })
        }) else {
            return;
        };
        match self.display.copy_to_clipboard(&text) {
            Ok(()) => self.notify(format!("copied {} lines", text.lines().count()), false),
            Err(e) => self.notify(format!("cannot copy to the clipboard: {}", e), true),
        }
    }

    /// The PTY run that should receive a mouse event at (`x`, `y`) and the cell of its screen
    /// there: the focused box shows a running terminal whose program asked for mouse events,
    /// and the point is on that terminal and not covered by another box.
    fn terminal_under_pointer(&mut self, x: u16, y: u16) -> Option<(RunId, usize, usize)> {
        let box_id = self.focused_box_id()?;
        let run = self.terminal_run_of(&box_id)?;
        if !self.executor.get(run)?.pty()?.mouse_reporting() {
            return None;
        }
        let views = self.views();
        let view = crate::render::view_at(&views, usize::from(x), usize::from(y))?;
        if view.id != box_id {
            return None;
        }
        match view.hit(usize::from(x), usize::from(y))? {
            BoxHit::Content { column, row } => Some((run, column, row)),
            _ => None,
        }
    }

    /// Sends a mouse event at `column`, `row` of the terminal screen to the program of `run`.
    fn forward_mouse(&mut self, run: RunId, column: usize, row: usize, mouse: &MouseEvent) {
        let (event_type, button) = match mouse.kind {
            MouseEventKind::Down(b) => (MouseEventType::Press, Some(term_button(b))),
            MouseEventKind::Up(b) => (MouseEventType::Release, Some(term_button(b))),
            MouseEventKind::Drag(b) => (MouseEventType::Motion, Some(term_button(b))),
            MouseEventKind::Moved => (MouseEventType::Motion, None),
            MouseEventKind::ScrollUp => (MouseEventType::Wheel, Some(TermButton::WheelUp)),
            MouseEventKind::ScrollDown => (MouseEventType::Wheel, Some(TermButton::WheelDown)),
            MouseEventKind::ScrollLeft => (MouseEventType::Wheel, Some(TermButton::WheelLeft)),
            MouseEventKind::ScrollRight => (MouseEventType::Wheel, Some(TermButton::WheelRight)),
        };
        let modifiers = MouseModifiers {
            shift: mouse.modifiers.contains(KeyModifiers::SHIFT),
            ctrl: mouse.modifiers.contains(KeyModifiers::CONTROL),
            alt: mouse.modifiers.contains(KeyModifiers::ALT),
            meta: mouse.modifiers.contains(KeyModifiers::SUPER),
        };
        let report = self
            .executor
            .get(run)
            .and_then(|r| r.pty())
            .and_then(|session| session.mouse_report(event_type, button, column, row, modifiers));
        if let Some(report) = report {
            self.write_to_terminal(run, report.as_bytes());
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        let (x, y) = (mouse.column, mouse.row);
        if self.pointer != Some((x, y)) {
            self.pointer = Some((x, y));
            self.dirty = true;
        }
        // A drag that started in a terminal stays with the terminal until the button is up,
        // with positions clamped to its screen.
        if let Some(Drag::Terminal { box_id }) = &self.drag {
            let box_id = box_id.clone();
            if let Some(run) = self.terminal_run_of(&box_id) {
                if let Some(area) = self.view_of(&box_id).and_then(|v| v.body).map(|b| b.area) {
                    let column = usize::from(x).clamp(area.left(), area.right()) - area.left();
                    let row = usize::from(y).clamp(area.top(), area.bottom()) - area.top();
                    self.forward_mouse(run, column, row, &mouse);
                }
            }
            if matches!(mouse.kind, MouseEventKind::Up(_)) {
                self.drag = None;
                self.button_down = false;
            }
            return;
        }

        match mouse.kind {
            MouseEventKind::Down(button) => {
                self.button_down = true;
                if let Some((run, column, row)) = self.terminal_under_pointer(x, y) {
                    self.forward_mouse(run, column, row, &mouse);
                    if let Some(box_id) = self.focused_box_id() {
                        self.drag = Some(Drag::Terminal { box_id });
                    }
                    return;
                }
                if button == MouseButton::Left {
                    self.click(x, y);
                }
            }
            MouseEventKind::Up(_) => {
                self.button_down = false;
                self.finish_drag();
            }
            // Terminals label button-less motion inconsistently (`Moved`, or `Drag` with a
            // phantom button), so the button state tracked here decides.
            MouseEventKind::Drag(_) | MouseEventKind::Moved => {
                if self.button_down {
                    self.continue_drag(x, y);
                } else if let Some((run, column, row)) = self.terminal_under_pointer(x, y) {
                    self.forward_mouse(run, column, row, &mouse);
                }
            }
            MouseEventKind::ScrollUp
            | MouseEventKind::ScrollDown
            | MouseEventKind::ScrollLeft
            | MouseEventKind::ScrollRight => {
                if let Some((run, column, row)) = self.terminal_under_pointer(x, y) {
                    self.forward_mouse(run, column, row, &mouse);
                    return;
                }
                let views = self.views();
                let Some(box_id) = crate::render::view_at(&views, usize::from(x), usize::from(y))
                    .map(|view| view.id.clone())
                else {
                    return;
                };
                let step = WHEEL_STEP as isize;
                match mouse.kind {
                    MouseEventKind::ScrollUp => self.scroll_box(&box_id, -step, 0),
                    MouseEventKind::ScrollDown => self.scroll_box(&box_id, step, 0),
                    MouseEventKind::ScrollLeft => self.scroll_box(&box_id, 0, -step),
                    _ => self.scroll_box(&box_id, 0, step),
                }
            }
        }
    }

    /// A left click: focuses the box under the pointer and acts on what was clicked (resize
    /// corner, scrollbar, tab bar, top border or a choice).
    fn click(&mut self, x: u16, y: u16) {
        let (cx, cy) = (usize::from(x), usize::from(y));
        let views = self.views();
        let Some(view) = crate::render::view_at(&views, cx, cy) else {
            return;
        };
        let Some(hit) = view.hit(cx, cy) else {
            return;
        };
        let box_id = view.id.clone();
        let Some(muxbox) = self.ctx.app.get_muxbox_by_id(&box_id) else {
            return;
        };
        let original = muxbox.position;
        let parent = self.parent_bounds(&box_id);

        if let Some(layout) = self.ctx.app.get_active_layout_mut() {
            layout.select_only_muxbox(&box_id);
        }
        self.dirty = true;

        match hit {
            BoxHit::ResizeHandle => {
                self.drag = Some(Drag::Resize {
                    box_id,
                    start: (x, y),
                    original,
                    parent,
                });
            }
            BoxHit::TopBorder => {
                if !self.ctx.config.locked {
                    self.drag = Some(Drag::Move {
                        box_id,
                        start: (x, y),
                        original,
                        parent,
                    });
                }
            }
            BoxHit::VerticalScrollbar(part) | BoxHit::HorizontalScrollbar(part) => {
                let vertical = matches!(hit, BoxHit::VerticalScrollbar(_));
                let bar = if vertical {
                    view.vertical_scrollbar
                } else {
                    view.horizontal_scrollbar
                };
                let Some(bar) = bar else {
                    return;
                };
                match part {
                    ScrollbarHit::Knob => {
                        self.drag = Some(Drag::Knob {
                            box_id,
                            vertical,
                            start: (x, y),
                            start_offset: bar.offset,
                        });
                    }
                    ScrollbarHit::Track => {
                        let cell = if vertical { cy } else { cx };
                        let percent =
                            percent_for_offset(bar.offset_for_click(cell), bar.max_offset());
                        if let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(&box_id) {
                            if vertical {
                                muxbox.vertical_scroll = Some(percent);
                            } else {
                                muxbox.horizontal_scroll = Some(percent);
                            }
                        }
                    }
                }
            }
            BoxHit::Tab(tab) => {
                let offset = view.tab_offsets().map_or(0, |(offset, _)| offset);
                let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(&box_id) else {
                    return;
                };
                match tab {
                    TabHit::ScrollLeft => muxbox.tab_scroll_offset = offset.saturating_sub(1),
                    TabHit::ScrollRight => muxbox.tab_scroll_offset = offset + 1,
                    TabHit::Tab(index) => {
                        muxbox.switch_to_tab(index);
                    }
                    TabHit::Close(index) => self.close_tab(&box_id, index),
                }
            }
            BoxHit::Choice(index) => {
                let choice_id = muxbox_choice_id(&self.ctx.app, &box_id, index);
                if let Some(choice_id) = choice_id {
                    self.activate_choice(&box_id, &choice_id);
                }
            }
            BoxHit::Content { .. } | BoxHit::Other => {}
        }
    }

    /// The bounds the position of `box_id` is relative to: its parent box, or the screen.
    fn parent_bounds(&mut self, box_id: &str) -> Bounds {
        let parent_id = self
            .ctx
            .app
            .get_muxbox_by_id(box_id)
            .and_then(|muxbox| muxbox.parent_id.clone());
        let bounds = self.ctx.app.active_bounds();
        parent_id
            .and_then(|parent| bounds.get(&parent).copied())
            .unwrap_or_else(|| self.ctx.app.screen())
    }

    /// Closes tab `index` of `box_id`, stopping the run that feeds it.
    fn close_tab(&mut self, box_id: &str, index: usize) {
        let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(box_id) else {
            return;
        };
        let Some(stream_id) = muxbox.get_tab_stream_ids().get(index).cloned() else {
            return;
        };
        let closeable = muxbox
            .streams
            .get(&stream_id)
            .is_some_and(|s| s.is_closeable(box_id));
        if !closeable {
            return;
        }
        if let Some(stream) = muxbox.remove_stream(&stream_id) {
            if let Some(run) = stream.run {
                self.executor.stop(run);
            }
        }
        self.restart_after_stop.remove(&stream_id);
        self.last_spec.remove(&stream_id);
        self.dirty = true;
    }

    fn continue_drag(&mut self, x: u16, y: u16) {
        let Some(drag) = self.drag.clone() else {
            return;
        };
        match drag {
            Drag::Resize {
                box_id,
                start,
                original,
                parent,
            } => {
                let new = resized(&original, &parent, delta(start.0, x), delta(start.1, y));
                self.set_position(&box_id, new);
            }
            Drag::Move {
                box_id,
                start,
                original,
                parent,
            } => {
                let new = moved(&original, &parent, delta(start.0, x), delta(start.1, y));
                self.set_position(&box_id, new);
            }
            Drag::Knob {
                box_id,
                vertical,
                start,
                start_offset,
            } => {
                let Some(view) = self.view_of(&box_id) else {
                    return;
                };
                let (bar, cells) = if vertical {
                    (view.vertical_scrollbar, delta(start.1, y))
                } else {
                    (view.horizontal_scrollbar, delta(start.0, x))
                };
                let Some(bar) = bar else {
                    return;
                };
                let percent =
                    percent_for_offset(bar.offset_for_drag(start_offset, cells), bar.max_offset());
                if let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(&box_id) {
                    if vertical {
                        muxbox.vertical_scroll = Some(percent);
                    } else {
                        muxbox.horizontal_scroll = Some(percent);
                    }
                }
                self.dirty = true;
            }
            Drag::Terminal { .. } => {}
        }
    }

    fn set_position(&mut self, box_id: &str, position: InputBounds) {
        if let Some(muxbox) = self.ctx.app.get_muxbox_by_id_mut(box_id) {
            if muxbox.position != position {
                muxbox.position = position;
                self.ctx.app.invalidate_layout_cache();
                self.dirty = true;
            }
        }
    }

    /// Ends a drag. A box that was moved or resized has its new position saved to the
    /// configuration file.
    fn finish_drag(&mut self) {
        let Some(drag) = self.drag.take() else {
            return;
        };
        let (box_id, original) = match drag {
            Drag::Resize {
                box_id, original, ..
            }
            | Drag::Move {
                box_id, original, ..
            } => (box_id, original),
            Drag::Knob { .. } | Drag::Terminal { .. } => return,
        };
        let Some(position) = self.ctx.app.get_muxbox_by_id(&box_id).map(|b| b.position) else {
            return;
        };
        if position == original {
            return;
        }
        self.resize_terminals();
        if let Some(path) = self.ctx.yaml_file_path.clone() {
            if let Err(e) = crate::model::yaml_edit::save_box_position(&path, &box_id, &position) {
                self.notify(e, true);
            }
        }
    }
}

/// The id of choice `index` of the visible menu of `box_id`.
fn muxbox_choice_id(app: &App, box_id: &str, index: usize) -> Option<String> {
    app.get_muxbox_by_id(box_id)?
        .get_selected_stream_choices()?
        .get(index)
        .map(|choice| choice.id.clone())
}

fn matching_handlers(
    handlers: Option<&std::collections::HashMap<String, Vec<String>>>,
    name: &str,
) -> Vec<Vec<String>> {
    handlers
        .into_iter()
        .flatten()
        .filter(|(configured, _)| key_matches(configured, name))
        .map(|(_, script)| script.clone())
        .collect()
}

fn term_button(button: MouseButton) -> TermButton {
    match button {
        MouseButton::Left => TermButton::Left,
        MouseButton::Right => TermButton::Right,
        MouseButton::Middle => TermButton::Middle,
    }
}

fn delta(from: u16, to: u16) -> isize {
    to as isize - from as isize
}

/// `value` limited to `[low, high]`; when the range is empty (the box is larger than its
/// parent) nothing can move, so the result is 0.
fn clamp_shift(value: isize, low: isize, high: isize) -> isize {
    if low > high {
        0
    } else {
        value.clamp(low, high)
    }
}

/// `original` moved by (`dx`, `dy`) cells, kept inside `parent` without changing its size.
/// Each edge keeps its unit (percent or cells).
pub(crate) fn moved(original: &InputBounds, parent: &Bounds, dx: isize, dy: isize) -> InputBounds {
    let current = original.to_bounds(parent);
    let dx = clamp_shift(
        dx,
        parent.left() as isize - current.left() as isize,
        parent.right() as isize - current.right() as isize,
    );
    let dy = clamp_shift(
        dy,
        parent.top() as isize - current.top() as isize,
        parent.bottom() as isize - current.bottom() as isize,
    );
    let shift = |value: usize, by: isize| value.saturating_add_signed(by);
    InputBounds {
        x1: original
            .x1
            .same_unit(shift(current.x1, dx) - parent.x1, parent.width()),
        y1: original
            .y1
            .same_unit(shift(current.y1, dy) - parent.y1, parent.height()),
        x2: original
            .x2
            .same_unit(shift(current.x2, dx) - parent.x1, parent.width()),
        y2: original
            .y2
            .same_unit(shift(current.y2, dy) - parent.y1, parent.height()),
    }
}

/// `original` with its bottom-right corner moved by (`dx`, `dy`) cells, at least
/// [`MIN_BOX_EXTENT`] cells in each direction and inside `parent`.
pub(crate) fn resized(
    original: &InputBounds,
    parent: &Bounds,
    dx: isize,
    dy: isize,
) -> InputBounds {
    let current = original.to_bounds(parent);
    let x2 = current.x2.saturating_add_signed(dx).clamp(
        (current.x1 + MIN_BOX_EXTENT - 1).min(parent.right()),
        parent.right(),
    );
    let y2 = current.y2.saturating_add_signed(dy).clamp(
        (current.y1 + MIN_BOX_EXTENT - 1).min(parent.bottom()),
        parent.bottom(),
    );
    InputBounds {
        x1: original.x1,
        y1: original.y1,
        x2: original.x2.same_unit(x2 - parent.x1, parent.width()),
        y2: original.y2.same_unit(y2 - parent.y1, parent.height()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bounds(x1: &str, y1: &str, x2: &str, y2: &str) -> InputBounds {
        InputBounds::parse(x1, y1, x2, y2).unwrap()
    }

    #[test]
    fn moving_keeps_units_and_size_and_stops_at_the_parent_edge() {
        let parent = Bounds::new(0, 0, 100, 50);
        let original = bounds("10%", "10", "30%", "20");
        let new = moved(&original, &parent, 5, 3);
        assert_eq!(new, bounds("15%", "13", "35%", "23"));
        // Dragging far past the right edge stops with the box against it, same width.
        let new = moved(&original, &parent, 500, 0);
        let b = new.to_bounds(&parent);
        let o = original.to_bounds(&parent);
        assert_eq!(b.right(), 100);
        assert_eq!(b.width(), o.width());
    }

    #[test]
    fn moving_with_cell_positions_never_converts_them_to_percentages() {
        let parent = Bounds::new(0, 0, 79, 23);
        let new = moved(&bounds("40", "5", "60", "10"), &parent, -3, 2);
        assert_eq!(new, bounds("37", "7", "57", "12"));
    }

    #[test]
    fn resizing_cannot_make_a_box_smaller_than_three_cells_or_leave_the_parent() {
        let parent = Bounds::new(0, 0, 79, 23);
        let original = bounds("10", "5", "20", "10");
        assert_eq!(
            resized(&original, &parent, -50, -50),
            bounds("10", "5", "12", "7")
        );
        assert_eq!(
            resized(&original, &parent, 500, 500),
            bounds("10", "5", "79", "23")
        );
    }

    #[test]
    fn a_box_larger_than_its_parent_does_not_move() {
        let parent = Bounds::new(0, 0, 9, 9);
        let original = bounds("0", "0", "20", "20");
        assert_eq!(moved(&original, &parent, 3, 3), bounds("0", "0", "9", "9"));
    }

    #[test]
    fn resizing_a_small_percentage_box_is_not_snapped_to_ten_percent() {
        let parent = Bounds::new(0, 0, 200, 100);
        let original = bounds("0%", "0%", "5%", "5%");
        let new = resized(&original, &parent, 2, 0);
        assert_eq!(new.x2.to_string(), "6%");
    }
}
