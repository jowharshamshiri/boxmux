use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};

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