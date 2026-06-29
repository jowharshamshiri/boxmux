use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalTheme {
    Light,
    Dark,
}

// Explicit theme override set from the CLI (`--dark`/`--light`). 0 = auto-detect,
// 1 = light, 2 = dark. An override takes precedence over COLORFGBG detection.
static THEME_OVERRIDE: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

/// Force the terminal theme, or pass `None` to fall back to auto-detection.
pub fn set_theme_override(theme: Option<TerminalTheme>) {
    let value = match theme {
        None => 0,
        Some(TerminalTheme::Light) => 1,
        Some(TerminalTheme::Dark) => 2,
    };
    THEME_OVERRIDE.store(value, std::sync::atomic::Ordering::Relaxed);
}

pub fn detect_terminal_theme() -> TerminalTheme {
    match THEME_OVERRIDE.load(std::sync::atomic::Ordering::Relaxed) {
        1 => return TerminalTheme::Light,
        2 => return TerminalTheme::Dark,
        _ => {}
    }

    std::env::var("COLORFGBG")
        .ok()
        .and_then(|value| {
            value
                .split(';')
                .next_back()
                .and_then(|part| part.parse::<u8>().ok())
        })
        .map(|background| {
            if matches!(background, 7 | 15) {
                TerminalTheme::Light
            } else {
                TerminalTheme::Dark
            }
        })
        .unwrap_or(TerminalTheme::Dark)
}

// A selected box uses the SAME text/background as an unselected one — selection is
// shown by the border (default_border_color) instead. The old code inverted the
// selected box (bright_white on bright_black), but bright_black (ANSI 8) is remapped
// to a light tint (cyan) by some palettes, making the selected box unreadable.
pub fn default_fg_color(selected: bool) -> &'static str {
    let _ = selected;
    match detect_terminal_theme() {
        TerminalTheme::Light => "black",
        TerminalTheme::Dark => "white",
    }
}

pub fn default_bg_color(selected: bool) -> &'static str {
    let _ = selected;
    match detect_terminal_theme() {
        TerminalTheme::Light => "white",
        TerminalTheme::Dark => "black",
    }
}

pub fn default_border_color(selected: bool) -> &'static str {
    match (detect_terminal_theme(), selected) {
        // Unselected light border is a fixed cube gray (not bright_black=ANSI 8,
        // which palettes remap to cyan). The FOCUSED box gets a distinct blue border
        // (focus_blue) so it's clearly indicated even for menu boxes whose choices
        // render over the tab row and hide the focused tab color.
        (TerminalTheme::Light, false) => "dim_gray",
        (TerminalTheme::Light, true) => "focus_blue",
        (TerminalTheme::Dark, false) => "white",
        (TerminalTheme::Dark, true) => "focus_blue",
    }
}

pub fn default_title_fg_color(selected: bool) -> &'static str {
    let _ = selected;
    match detect_terminal_theme() {
        TerminalTheme::Light => "black",
        TerminalTheme::Dark => "white",
    }
}

pub fn default_title_bg_color(selected: bool) -> &'static str {
    let _ = selected;
    match (detect_terminal_theme(), selected) {
        (TerminalTheme::Light, _) => "bright_white",
        // Dark: inactive title/tab bars blend with the dark panel; the active tab
        // (selected-title color) is the only highlighted one.
        (TerminalTheme::Dark, false) => "black",
        (TerminalTheme::Dark, true) => "black",
    }
}

/// Colors for the ACTIVE tab / selected title bar. The active tab must stand out
/// from inactive tabs without becoming a light bar on a dark page (the old code
/// inverted title_fg/title_bg, which made the active tab's background = title_fg =
/// white in dark mode). Light keeps its liked dark bar; dark uses the same blue
/// accent as the selection so it's distinct yet still dark.
pub fn default_selected_title_bg_color() -> &'static str {
    // Fixed dark gray (cube index 238) in both themes — same reasoning as the
    // selected-menu color: avoid base ANSI colors that palettes remap to a light
    // tint, so the active tab is always a dark bar with readable white text.
    "dim_gray"
}

pub fn default_selected_title_fg_color() -> &'static str {
    "bright_white"
}

/// Active-tab background for the FOCUSED (selected) box — a distinct blue so the
/// focused box stands out from the dim-gray active tabs of the other boxes.
pub fn default_focused_title_bg_color() -> &'static str {
    "focus_blue"
}

pub fn default_menu_fg_color() -> &'static str {
    default_fg_color(false)
}

pub fn default_menu_bg_color() -> &'static str {
    default_bg_color(false)
}

pub fn default_selected_menu_fg_color() -> &'static str {
    "bright_white"
}

pub fn default_selected_menu_bg_color() -> &'static str {
    // Fixed dark gray (cube index 238) + white text reads reliably in both themes.
    // "blue" resolves to bright-blue (ANSI 12), which terminal palettes can render
    // as a light cyan — giving unreadable white-on-cyan selected items.
    "dim_gray"
}

pub fn default_hover_fg_color() -> &'static str {
    // Dark text on the yellow hover background reads well in both themes.
    "black"
}

pub fn default_hover_bg_color() -> &'static str {
    "bright_yellow"
}

/// Get foreground color code, returning empty string for transparent (None) colors
pub fn get_fg_color_transparent(color: &Option<String>) -> String {
    match color {
        Some(color_str) => get_fg_color(color_str),
        None => String::new(), // Transparent - no color change
    }
}

/// Get background color code, returning empty string for transparent (None) colors  
pub fn get_bg_color_transparent(color: &Option<String>) -> String {
    match color {
        Some(color_str) => get_bg_color(color_str),
        None => String::new(), // Transparent - no color change
    }
}

/// Check if color should be drawn (not transparent)
pub fn should_draw_color(color: &Option<String>) -> bool {
    color.is_some()
}

pub fn get_fg_color(color: &str) -> String {
    match color {
        "red" => format!("{}", SetForegroundColor(Color::Red)),
        "green" => format!("{}", SetForegroundColor(Color::Green)),
        "yellow" => format!("{}", SetForegroundColor(Color::Yellow)),
        "blue" => format!("{}", SetForegroundColor(Color::Blue)),
        "magenta" => format!("{}", SetForegroundColor(Color::Magenta)),
        "cyan" => format!("{}", SetForegroundColor(Color::Cyan)),
        "white" => format!("{}", SetForegroundColor(Color::White)),
        "black" => format!("{}", SetForegroundColor(Color::Black)),
        "dark_gray" | "gray" | "grey" => format!("{}", SetForegroundColor(Color::AnsiValue(8))),
        // Fixed 256-color-cube grays (indices 16-255 are NOT remapped by terminal
        // palettes, unlike the 16 base ANSI colors), so they render predictably.
        "dim_gray" => format!("{}", SetForegroundColor(Color::AnsiValue(238))),
        // Fixed cube blue for the focused box's active tab (palette-independent).
        "focus_blue" => format!("{}", SetForegroundColor(Color::AnsiValue(25))),
        "reset" => format!("{}", SetForegroundColor(Color::Reset)),
        "bright_black" => format!("{}", SetForegroundColor(Color::AnsiValue(8))),
        "bright_red" => format!("{}", SetForegroundColor(Color::AnsiValue(9))),
        "bright_green" => format!("{}", SetForegroundColor(Color::AnsiValue(10))),
        "bright_yellow" => format!("{}", SetForegroundColor(Color::AnsiValue(11))),
        "bright_blue" => format!("{}", SetForegroundColor(Color::AnsiValue(12))),
        "bright_magenta" => format!("{}", SetForegroundColor(Color::AnsiValue(13))),
        "bright_cyan" => format!("{}", SetForegroundColor(Color::AnsiValue(14))),
        "bright_white" => format!("{}", SetForegroundColor(Color::AnsiValue(15))),
        _ => format!("{}", SetForegroundColor(Color::Reset)),
    }
}

pub fn get_bg_color(color: &str) -> String {
    match color {
        "red" => format!("{}", SetBackgroundColor(Color::Red)),
        "green" => format!("{}", SetBackgroundColor(Color::Green)),
        "yellow" => format!("{}", SetBackgroundColor(Color::Yellow)),
        "blue" => format!("{}", SetBackgroundColor(Color::Blue)),
        "magenta" => format!("{}", SetBackgroundColor(Color::Magenta)),
        "cyan" => format!("{}", SetBackgroundColor(Color::Cyan)),
        "white" => format!("{}", SetBackgroundColor(Color::White)),
        "black" => format!("{}", SetBackgroundColor(Color::Black)),
        "dark_gray" | "gray" | "grey" => format!("{}", SetBackgroundColor(Color::AnsiValue(8))),
        // Fixed 256-color-cube grays (indices 16-255 are NOT remapped by terminal
        // palettes, unlike the 16 base ANSI colors), so they render predictably.
        "dim_gray" => format!("{}", SetBackgroundColor(Color::AnsiValue(238))),
        // Fixed cube blue for the focused box's active tab (palette-independent).
        "focus_blue" => format!("{}", SetBackgroundColor(Color::AnsiValue(25))),
        "reset" => format!("{}", SetBackgroundColor(Color::Reset)),
        "bright_black" => format!("{}", SetBackgroundColor(Color::AnsiValue(8))),
        "bright_red" => format!("{}", SetBackgroundColor(Color::AnsiValue(9))),
        "bright_green" => format!("{}", SetBackgroundColor(Color::AnsiValue(10))),
        "bright_yellow" => format!("{}", SetBackgroundColor(Color::AnsiValue(11))),
        "bright_blue" => format!("{}", SetBackgroundColor(Color::AnsiValue(12))),
        "bright_magenta" => format!("{}", SetBackgroundColor(Color::AnsiValue(13))),
        "bright_cyan" => format!("{}", SetBackgroundColor(Color::AnsiValue(14))),
        "bright_white" => format!("{}", SetBackgroundColor(Color::AnsiValue(15))),
        _ => format!("{}", SetBackgroundColor(Color::Reset)),
    }
}

#[cfg(test)]
mod theme_override_tests {
    use super::*;

    // Restores auto-detect on drop so the global override never leaks to other
    // tests, even if an assertion panics.
    struct ResetGuard;
    impl Drop for ResetGuard {
        fn drop(&mut self) {
            set_theme_override(None);
        }
    }

    // NOTE: these assertions live in a single test on purpose — they mutate the
    // process-global theme override, so splitting them into separate #[test]s
    // would let cargo's parallel runner race them against each other.
    #[test]
    fn test_theme_override_and_faithful_dark_inverse() {
        let _guard = ResetGuard;
        let dark = ["black", "bright_black"];
        let light = ["white", "bright_white"];

        set_theme_override(Some(TerminalTheme::Light));
        assert_eq!(detect_terminal_theme(), TerminalTheme::Light);
        // Defaults follow the forced theme regardless of COLORFGBG.
        assert_eq!(default_bg_color(false), "white");

        set_theme_override(Some(TerminalTheme::Dark));
        assert_eq!(detect_terminal_theme(), TerminalTheme::Dark);
        assert_eq!(default_bg_color(false), "black");

        // The dark theme is the dark-paged version of the light theme: every panel/
        // title BACKGROUND must be dark, every text FOREGROUND light, in both states.
        for selected in [false, true] {
            assert!(
                dark.contains(&default_bg_color(selected)),
                "dark panel bg must be dark (selected={selected}), got {}",
                default_bg_color(selected)
            );
            assert!(
                light.contains(&default_fg_color(selected)),
                "dark text must be light (selected={selected})"
            );
            assert!(
                dark.contains(&default_title_bg_color(selected)),
                "dark title bar bg must be dark (selected={selected}), got {}",
                default_title_bg_color(selected)
            );
        }
        // Normal menu rows share the panel bg (dark).
        assert!(dark.contains(&default_menu_bg_color()));

        // Accents match the light theme exactly (same design language).
        set_theme_override(Some(TerminalTheme::Light));
        let light_sel_bg = default_selected_menu_bg_color();
        let light_hover_bg = default_hover_bg_color();
        let light_hover_fg = default_hover_fg_color();
        set_theme_override(Some(TerminalTheme::Dark));
        assert_eq!(default_selected_menu_bg_color(), light_sel_bg);
        assert_eq!(default_hover_bg_color(), light_hover_bg);
        assert_eq!(default_hover_fg_color(), light_hover_fg);
    }
}
