//! A choice's confirmation: the modal window that asks before the choice's script runs.
//!
//! While it is open the window takes every key and mouse press. Left and Right (Up and Down,
//! Tab and Shift+Tab) move between the answers, Enter or Space presses the one marked, Escape
//! cancels, and so does a click outside the window. The first answer is marked when it opens,
//! as in a desktop dialog. An answer other than the one that cancels runs the choice's script
//! with the answer's id in `BOXMUX_OPTION`.

use super::Runtime;
use crate::model::choice::{Choice, Confirm};
use crate::render::dialog::{Dialog, DialogHit, DialogView};
use crate::runtime::display::Display;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};

/// The environment variable that carries the chosen answer's id to the choice's script.
pub const OPTION_VARIABLE: &str = "BOXMUX_OPTION";

/// An open confirmation.
#[derive(Debug, Clone)]
pub(crate) struct Confirmation {
    pub box_id: String,
    pub choice_id: String,
    /// The confirmation as the choice declared it when the window opened.
    pub confirm: Confirm,
    pub dialog: Dialog,
    /// The answers' ids, in the order of the dialog's buttons.
    pub option_ids: Vec<String>,
    /// The id of the answer that cancels, if one does.
    pub cancel: Option<String>,
    /// The answer Enter presses.
    pub focused: usize,
}

impl Confirmation {
    fn new(box_id: &str, choice: &Choice, confirm: &Confirm) -> Confirmation {
        let options = confirm.options();
        Confirmation {
            box_id: box_id.to_string(),
            choice_id: choice.id.clone(),
            confirm: confirm.clone(),
            dialog: Dialog {
                title: choice.content.clone().unwrap_or_else(|| choice.id.clone()),
                text: confirm.text.clone(),
                buttons: options.iter().map(|o| o.label().to_string()).collect(),
            },
            option_ids: options.into_iter().map(|o| o.id).collect(),
            cancel: confirm.cancel_id().map(str::to_string),
            focused: 0,
        }
    }
}

impl<D: Display> Runtime<D> {
    /// Opens the confirmation of `choice` of `box_id`.
    pub(super) fn open_confirmation(&mut self, box_id: &str, choice: &Choice, confirm: &Confirm) {
        self.confirmation = Some(Confirmation::new(box_id, choice, confirm));
        // A drag in progress ends: the window takes the mouse.
        self.drag = None;
        self.dirty = true;
    }

    /// The open confirmation's window, as laid out on the screen now.
    pub(super) fn confirmation_view(&self) -> Option<DialogView> {
        let confirmation = self.confirmation.as_ref()?;
        let screen = self.ctx.app.screen();
        DialogView::new(&confirmation.dialog, screen.width(), screen.height())
    }

    /// Handles a key while a confirmation is open; every key goes to it.
    pub(super) fn confirmation_key(&mut self, key: KeyEvent) {
        let Some(confirmation) = self.confirmation.as_mut() else {
            return;
        };
        let count = confirmation.option_ids.len();
        let plain = key.modifiers.difference(KeyModifiers::SHIFT).is_empty();
        match key.code {
            KeyCode::Esc => self.answer_confirmation(None),
            KeyCode::Enter | KeyCode::Char(' ') if plain => {
                let focused = confirmation.focused;
                self.answer_confirmation(Some(focused));
            }
            KeyCode::Right | KeyCode::Down | KeyCode::Tab if plain => {
                confirmation.focused = (confirmation.focused + 1) % count;
                self.dirty = true;
            }
            KeyCode::Left | KeyCode::Up | KeyCode::BackTab if plain => {
                confirmation.focused = (confirmation.focused + count - 1) % count;
                self.dirty = true;
            }
            _ => {}
        }
    }

    /// Handles a mouse event while a confirmation is open: a left press on an answer presses
    /// it, one outside the window cancels; nothing reaches the layout under the window.
    pub(super) fn confirmation_mouse(&mut self, mouse: MouseEvent) {
        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {}
            MouseEventKind::Up(_) => {
                self.button_down = false;
                return;
            }
            _ => return,
        }
        let Some(view) = self.confirmation_view() else {
            // No room for the window on this screen: nothing to press but Escape.
            return;
        };
        match view.hit(usize::from(mouse.column), usize::from(mouse.row)) {
            DialogHit::Button(index) => self.answer_confirmation(Some(index)),
            DialogHit::Outside => self.answer_confirmation(None),
            DialogHit::Window => {}
        }
    }

    /// The answer under the mouse pointer, while a confirmation is open.
    pub(super) fn hovered_answer(&self) -> Option<usize> {
        let (x, y) = self.pointer?;
        match self
            .confirmation_view()?
            .hit(usize::from(x), usize::from(y))
        {
            DialogHit::Button(index) => Some(index),
            DialogHit::Window | DialogHit::Outside => None,
        }
    }

    /// Closes the confirmation with answer `index`, or cancelled with `None`. An answer other
    /// than the one that cancels runs the choice with that answer's id.
    fn answer_confirmation(&mut self, index: Option<usize>) {
        let Some(confirmation) = self.confirmation.take() else {
            return;
        };
        self.dirty = true;
        let Some(option) = index.and_then(|i| confirmation.option_ids.get(i)) else {
            return;
        };
        if confirmation.cancel.as_ref() == Some(option) {
            return;
        }
        let choice = self
            .shown_choice(&confirmation.box_id, &confirmation.choice_id)
            .cloned();
        match choice {
            Some(choice) => self.start_choice(&confirmation.box_id, &choice, Some(option)),
            None => self.notify(
                format!(
                    "choice '{}' of box '{}' is no longer shown; nothing ran",
                    confirmation.choice_id, confirmation.box_id
                ),
                true,
            ),
        }
    }

    /// Choice `choice_id` of `box_id`, when the box is in the shown layout and still has it.
    fn shown_choice(&self, box_id: &str, choice_id: &str) -> Option<&Choice> {
        self.ctx
            .app
            .get_active_layout()?
            .get_all_muxboxes()
            .into_iter()
            .find(|muxbox| muxbox.id == box_id)?
            .choices
            .iter()
            .flatten()
            .find(|choice| choice.id == choice_id)
    }

    /// Closes the open confirmation when its choice left the shown layout (the layout was
    /// switched, or its box replaced or removed over the control socket), saying so.
    pub(super) fn close_stale_confirmation(&mut self) {
        let Some(confirmation) = &self.confirmation else {
            return;
        };
        let still_asks = self
            .shown_choice(&confirmation.box_id, &confirmation.choice_id)
            .is_some_and(|choice| choice.confirm.as_ref() == Some(&confirmation.confirm));
        if still_asks {
            return;
        }
        let text = format!(
            "the confirmation of choice '{}' closed: the choice is no longer shown, or asks something else now",
            confirmation.choice_id
        );
        self.confirmation = None;
        self.notify(text, false);
    }
}
