//! Colors of the built-in light and dark themes, and the resolution of every color a box is
//! drawn with.
//!
//! A color setting is looked up on the box, then on each enclosing box from the nearest out,
//! then on the layout; the theme supplies the color when nobody sets it. A focused box uses the
//! `selected_*` settings, a box under the mouse pointer the `highlighted_*` ones, and a box
//! whose visible tab failed the `error_*` ones (`error_selected_*` when it is also focused).
//! A state-specific setting falls back to the plain setting, except where the theme marks the
//! state itself: the focused border and the error border and title.

use super::screen::{Color, Style};
use crate::model::appearance::{ColorName, Overflow, TitlePosition};
use crate::model::layout::Layout;
use crate::model::muxbox::MuxBox;
use std::sync::atomic::{AtomicU8, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalTheme {
    Light,
    Dark,
}

// Set from the command line (`--dark`/`--light`): 0 detects, 1 is light, 2 is dark.
static THEME_OVERRIDE: AtomicU8 = AtomicU8::new(0);

/// Forces the theme, or with `None` goes back to detecting it.
pub fn set_theme_override(theme: Option<TerminalTheme>) {
    let value = match theme {
        None => 0,
        Some(TerminalTheme::Light) => 1,
        Some(TerminalTheme::Dark) => 2,
    };
    THEME_OVERRIDE.store(value, Ordering::Relaxed);
}

/// The forced theme, or the one `COLORFGBG` reports (a light background is color 7 or 15).
/// Dark when nothing tells.
pub fn terminal_theme() -> TerminalTheme {
    match THEME_OVERRIDE.load(Ordering::Relaxed) {
        1 => return TerminalTheme::Light,
        2 => return TerminalTheme::Dark,
        _ => {}
    }
    std::env::var("COLORFGBG")
        .ok()
        .and_then(|value| value.rsplit(';').next()?.parse::<u8>().ok())
        .map(|background| {
            if matches!(background, 7 | 15) {
                TerminalTheme::Light
            } else {
                TerminalTheme::Dark
            }
        })
        .unwrap_or(TerminalTheme::Dark)
}

// Fixed entries of the 256-color cube. Unlike the 16 base colors, terminal palettes do not
// remap them, so the theme's accents look the same in every terminal.
const DIM_GRAY: Color = Color::AnsiValue(238);
const FOCUS_BLUE: Color = Color::AnsiValue(25);

/// The colors of the theme in use.
#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub fg: Color,
    pub bg: Color,
    pub border: Color,
    pub focused_border: Color,
    pub error_border: Color,
    pub error_focused_border: Color,
    /// The border of a terminal whose program has ended.
    pub ended_border: Color,
    pub error_title: Style,
    pub active_tab: Style,
    pub focused_active_tab: Style,
    pub menu_selected: Style,
    pub menu_hovered: Style,
    pub scrollbar_track: Color,
    pub notice: Style,
    pub error_notice: Style,
}

pub fn palette() -> Palette {
    let (fg, bg, border) = match terminal_theme() {
        TerminalTheme::Light => (ColorName::Black.color(), ColorName::White.color(), DIM_GRAY),
        TerminalTheme::Dark => (
            ColorName::White.color(),
            ColorName::Black.color(),
            ColorName::White.color(),
        ),
    };
    let bright_white = ColorName::BrightWhite.color();
    Palette {
        fg,
        bg,
        border,
        focused_border: FOCUS_BLUE,
        error_border: ColorName::Red.color(),
        error_focused_border: ColorName::BrightRed.color(),
        ended_border: ColorName::Yellow.color(),
        error_title: Style::new(bright_white, ColorName::Red.color()),
        active_tab: Style::new(bright_white, DIM_GRAY),
        focused_active_tab: Style::new(bright_white, FOCUS_BLUE),
        menu_selected: Style::new(bright_white, DIM_GRAY),
        menu_hovered: Style::new(ColorName::Black.color(), ColorName::BrightYellow.color()),
        scrollbar_track: ColorName::BrightBlack.color(),
        notice: Style::new(bright_white, FOCUS_BLUE),
        error_notice: Style::new(bright_white, ColorName::Red.color()),
    }
}

/// The state a box is drawn in.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BoxState {
    pub focused: bool,
    pub hovered: bool,
    pub error: bool,
    /// The visible tab is a terminal whose program ended (without failing).
    pub ended: bool,
}

/// Everything about a box's look that its configuration and state decide.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoxStyle {
    /// Text and background of the content area.
    pub body: Style,
    pub border: Style,
    /// The title, and tabs that are not the visible one.
    pub title: Style,
    pub active_tab: Style,
    pub menu: Style,
    pub menu_selected: Style,
    pub menu_hovered: Style,
    pub scrollbar_track: Style,
    pub scrollbar_knob: Style,
    pub fill_char: char,
    /// Whether the box paints its background; a box that does not shows what is beneath it.
    pub filled: bool,
    pub overflow: Overflow,
    pub title_position: TitlePosition,
}

/// The box, its enclosing boxes from the nearest out, and its layout: where settings are
/// looked up, in that order.
pub struct Scope<'a> {
    pub muxbox: &'a MuxBox,
    pub ancestors: &'a [&'a MuxBox],
    pub layout: &'a Layout,
}

macro_rules! setting {
    ($scope:expr, $field:ident) => {{
        let scope: &Scope = $scope;
        std::iter::once(scope.muxbox)
            .chain(scope.ancestors.iter().copied())
            .find_map(|b| b.$field)
            .or(scope.layout.$field)
    }};
}

/// The first setting that is present.
fn first(candidates: &[Option<ColorName>]) -> Option<Color> {
    candidates.iter().flatten().next().map(|name| name.color())
}

impl BoxStyle {
    pub fn resolve(scope: &Scope, state: BoxState) -> BoxStyle {
        let theme = palette();
        let BoxState {
            focused,
            hovered,
            error,
            ended,
        } = state;
        // Only the settings of the states the box is in take part.
        let when = |on: bool, value: Option<ColorName>| if on { value } else { None };

        let fg = first(&[
            when(error && focused, setting!(scope, error_selected_fg_color)),
            when(error, setting!(scope, error_fg_color)),
            when(focused, setting!(scope, selected_fg_color)),
            when(hovered, setting!(scope, highlighted_fg_color)),
            setting!(scope, fg_color),
        ])
        .unwrap_or(theme.fg);
        let bg = first(&[
            when(error && focused, setting!(scope, error_selected_bg_color)),
            when(error, setting!(scope, error_bg_color)),
            when(focused, setting!(scope, selected_bg_color)),
            when(hovered, setting!(scope, highlighted_bg_color)),
            setting!(scope, bg_color),
        ])
        .unwrap_or(theme.bg);

        let border = if error {
            first(&[
                when(focused, setting!(scope, error_selected_border_color)),
                setting!(scope, error_border_color),
            ])
            .unwrap_or(if focused {
                theme.error_focused_border
            } else {
                theme.error_border
            })
        } else if ended {
            theme.ended_border
        } else if focused {
            first(&[setting!(scope, selected_border_color)]).unwrap_or(theme.focused_border)
        } else {
            first(&[
                when(hovered, setting!(scope, highlighted_border_color)),
                setting!(scope, border_color),
            ])
            .unwrap_or(theme.border)
        };

        let title = if error {
            Style::new(
                first(&[
                    when(focused, setting!(scope, error_selected_title_fg_color)),
                    setting!(scope, error_title_fg_color),
                ])
                .unwrap_or(theme.error_title.fg),
                first(&[
                    when(focused, setting!(scope, error_selected_title_bg_color)),
                    setting!(scope, error_title_bg_color),
                ])
                .unwrap_or(theme.error_title.bg),
            )
        } else {
            Style::new(
                first(&[
                    when(focused, setting!(scope, selected_title_fg_color)),
                    when(hovered, setting!(scope, highlighted_title_fg_color)),
                    setting!(scope, title_fg_color),
                ])
                .unwrap_or(fg),
                first(&[
                    when(focused, setting!(scope, selected_title_bg_color)),
                    when(hovered, setting!(scope, highlighted_title_bg_color)),
                    setting!(scope, title_bg_color),
                ])
                .unwrap_or(bg),
            )
        };
        let active_tab = if focused {
            theme.focused_active_tab
        } else {
            theme.active_tab
        };

        let menu = Style::new(
            first(&[setting!(scope, menu_fg_color)]).unwrap_or(fg),
            first(&[setting!(scope, menu_bg_color)]).unwrap_or(bg),
        );
        let menu_selected = Style::new(
            first(&[setting!(scope, selected_menu_fg_color)]).unwrap_or(theme.menu_selected.fg),
            first(&[setting!(scope, selected_menu_bg_color)]).unwrap_or(theme.menu_selected.bg),
        );
        let menu_hovered = Style::new(
            first(&[setting!(scope, highlighted_menu_fg_color)]).unwrap_or(theme.menu_hovered.fg),
            first(&[setting!(scope, highlighted_menu_bg_color)]).unwrap_or(theme.menu_hovered.bg),
        );

        let fill_char = [
            when_char(focused, setting!(scope, selected_fill_char)),
            when_char(hovered, setting!(scope, highlighted_fill_char)),
            setting!(scope, fill_char),
        ]
        .into_iter()
        .flatten()
        .next()
        .unwrap_or(' ');

        BoxStyle {
            body: Style::new(fg, bg),
            border: Style::new(border, bg),
            title,
            active_tab,
            menu,
            menu_selected,
            menu_hovered,
            scrollbar_track: Style::new(theme.scrollbar_track, bg),
            scrollbar_knob: Style::new(border, bg),
            fill_char,
            filled: setting!(scope, fill).unwrap_or(true),
            overflow: setting!(scope, overflow_behavior).unwrap_or_default(),
            title_position: setting!(scope, title_position).unwrap_or_default(),
        }
    }
}

fn when_char(on: bool, value: Option<char>) -> Option<char> {
    if on {
        value
    } else {
        None
    }
}

/// The background of the screen behind a layout's boxes, or `None` when the layout leaves the
/// terminal's own background (`fill: false`).
pub fn layout_background(layout: &Layout) -> Option<Style> {
    if layout.fill == Some(false) {
        return None;
    }
    let theme = palette();
    let fg = layout.fg_color.map_or(theme.fg, ColorName::color);
    let bg = layout.bg_color.map_or(theme.bg, ColorName::color);
    Some(Style::new(fg, bg))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve(
        muxbox: &MuxBox,
        ancestors: &[&MuxBox],
        layout: &Layout,
        state: BoxState,
    ) -> BoxStyle {
        BoxStyle::resolve(
            &Scope {
                muxbox,
                ancestors,
                layout,
            },
            state,
        )
    }

    fn focused() -> BoxState {
        BoxState {
            focused: true,
            ..BoxState::default()
        }
    }

    #[test]
    fn settings_come_from_the_box_then_enclosing_boxes_then_the_layout() {
        let layout = Layout {
            fg_color: Some(ColorName::Green),
            bg_color: Some(ColorName::Blue),
            border_color: Some(ColorName::Cyan),
            ..Layout::default()
        };
        let outer = MuxBox {
            bg_color: Some(ColorName::Magenta),
            ..MuxBox::default()
        };
        let inner = MuxBox {
            bg_color: Some(ColorName::Yellow),
            ..MuxBox::default()
        };
        let muxbox = MuxBox {
            fg_color: Some(ColorName::Red),
            ..MuxBox::default()
        };
        let style = resolve(&muxbox, &[&inner, &outer], &layout, BoxState::default());
        assert_eq!(style.body.fg, ColorName::Red.color());
        assert_eq!(
            style.body.bg,
            ColorName::Yellow.color(),
            "the nearest box wins"
        );
        assert_eq!(style.border.fg, ColorName::Cyan.color());
    }

    #[test]
    fn focus_keeps_the_box_colors_and_marks_the_border() {
        let layout = Layout::default();
        let muxbox = MuxBox {
            bg_color: Some(ColorName::Green),
            border_color: Some(ColorName::Yellow),
            ..MuxBox::default()
        };
        let style = resolve(&muxbox, &[], &layout, focused());
        assert_eq!(
            style.body.bg,
            ColorName::Green.color(),
            "focus must not change the background"
        );
        assert_eq!(style.border.fg, FOCUS_BLUE);

        let muxbox = MuxBox {
            selected_border_color: Some(ColorName::BrightGreen),
            ..muxbox
        };
        let style = resolve(&muxbox, &[], &layout, focused());
        assert_eq!(style.border.fg, ColorName::BrightGreen.color());
    }

    #[test]
    fn a_failed_box_shows_error_colors_and_its_own_error_settings() {
        let layout = Layout::default();
        let error = BoxState {
            error: true,
            ..BoxState::default()
        };
        let muxbox = MuxBox::default();
        let style = resolve(&muxbox, &[], &layout, error);
        assert_eq!(style.border.fg, ColorName::Red.color());
        assert_eq!(style.title.bg, ColorName::Red.color());

        let muxbox = MuxBox {
            error_border_color: Some(ColorName::Magenta),
            error_selected_border_color: Some(ColorName::BrightMagenta),
            ..MuxBox::default()
        };
        assert_eq!(
            resolve(&muxbox, &[], &layout, error).border.fg,
            ColorName::Magenta.color()
        );
        let both = BoxState {
            error: true,
            focused: true,
            ..BoxState::default()
        };
        assert_eq!(
            resolve(&muxbox, &[], &layout, both).border.fg,
            ColorName::BrightMagenta.color()
        );
    }

    #[test]
    fn hover_settings_apply_only_under_the_pointer() {
        let layout = Layout::default();
        let muxbox = MuxBox {
            highlighted_bg_color: Some(ColorName::BrightBlue),
            highlighted_fill_char: Some('.'),
            ..MuxBox::default()
        };
        let hovered = BoxState {
            hovered: true,
            ..BoxState::default()
        };
        let style = resolve(&muxbox, &[], &layout, hovered);
        assert_eq!(style.body.bg, ColorName::BrightBlue.color());
        assert_eq!(style.fill_char, '.');
        let style = resolve(&muxbox, &[], &layout, BoxState::default());
        assert_ne!(style.body.bg, ColorName::BrightBlue.color());
        assert_eq!(style.fill_char, ' ');
    }

    #[test]
    fn the_menu_follows_the_box_text_unless_set() {
        let layout = Layout::default();
        let muxbox = MuxBox {
            fg_color: Some(ColorName::Cyan),
            menu_bg_color: Some(ColorName::Blue),
            ..MuxBox::default()
        };
        let style = resolve(&muxbox, &[], &layout, BoxState::default());
        assert_eq!(style.menu.fg, ColorName::Cyan.color());
        assert_eq!(style.menu.bg, ColorName::Blue.color());
    }

    #[test]
    fn overflow_title_position_and_fill_are_inherited() {
        let layout = Layout {
            overflow_behavior: Some(Overflow::Wrap),
            title_position: Some(TitlePosition::Left),
            fill: Some(false),
            ..Layout::default()
        };
        let style = resolve(&MuxBox::default(), &[], &layout, BoxState::default());
        assert_eq!(style.overflow, Overflow::Wrap);
        assert_eq!(style.title_position, TitlePosition::Left);
        assert!(!style.filled);
    }
}
