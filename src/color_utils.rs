use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalTheme {
    Light,
    Dark,
}

pub fn detect_terminal_theme() -> TerminalTheme {
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

pub fn default_fg_color(selected: bool) -> &'static str {
    match (detect_terminal_theme(), selected) {
        (TerminalTheme::Light, false) => "black",
        (TerminalTheme::Light, true) => "bright_white",
        (TerminalTheme::Dark, false) => "white",
        (TerminalTheme::Dark, true) => "bright_white",
    }
}

pub fn default_bg_color(selected: bool) -> &'static str {
    match (detect_terminal_theme(), selected) {
        (TerminalTheme::Light, false) => "white",
        (TerminalTheme::Light, true) => "bright_black",
        (TerminalTheme::Dark, false) => "black",
        (TerminalTheme::Dark, true) => "bright_black",
    }
}

pub fn default_border_color(selected: bool) -> &'static str {
    match (detect_terminal_theme(), selected) {
        (TerminalTheme::Light, false) => "bright_black",
        (TerminalTheme::Light, true) => "black",
        (TerminalTheme::Dark, false) => "white",
        (TerminalTheme::Dark, true) => "bright_white",
    }
}

pub fn default_title_fg_color(selected: bool) -> &'static str {
    match (detect_terminal_theme(), selected) {
        (TerminalTheme::Light, false) => "black",
        (TerminalTheme::Light, true) => "bright_white",
        (TerminalTheme::Dark, false) => "white",
        (TerminalTheme::Dark, true) => "bright_white",
    }
}

pub fn default_title_bg_color(selected: bool) -> &'static str {
    match (detect_terminal_theme(), selected) {
        (TerminalTheme::Light, false) => "bright_white",
        (TerminalTheme::Light, true) => "bright_black",
        (TerminalTheme::Dark, false) => "bright_black",
        (TerminalTheme::Dark, true) => "black",
    }
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
    match detect_terminal_theme() {
        TerminalTheme::Light => "blue",
        TerminalTheme::Dark => "red",
    }
}

pub fn default_hover_fg_color() -> &'static str {
    match detect_terminal_theme() {
        TerminalTheme::Light => "black",
        TerminalTheme::Dark => "bright_white",
    }
}

pub fn default_hover_bg_color() -> &'static str {
    match detect_terminal_theme() {
        TerminalTheme::Light => "bright_yellow",
        TerminalTheme::Dark => "bright_blue",
    }
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
