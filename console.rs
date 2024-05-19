use std::thread::sleep;
use std::time::Duration;
use std::collections::HashMap;

// Character representations
fn get_char_map() -> HashMap<char, [&'static str; 4]> {
    let mut char_map = HashMap::new();

    char_map.insert('A', ["╔═╗", "╠═╣", "╩ ╩", ""]);
    char_map.insert('B', ["╔╗ ", "╠╩╗", "╚═╝", ""]);
    char_map.insert('C', ["╔═╗", "║  ", "╚═╝", ""]);
    char_map.insert('D', ["╔╦╗", " ║║", "═╩╝", ""]);
    char_map.insert('E', ["╔═╗", "║╣ ", "╚═╝", ""]);
    char_map.insert('F', ["╔═╗", "╠╣ ", "╚  ", ""]);
    char_map.insert('G', ["╔═╗", "║ ╦", "╚═╝", ""]);
    char_map.insert('H', ["╦ ╦", "╠═╣", "╩ ╩", ""]);
    char_map.insert('I', [" ╦ ", " ║ ", " ╩ ", ""]);
    char_map.insert('J', [" ╦ ", " ║ ", "╚╝ ", ""]);
    char_map.insert('K', ["╦╔═", "╠╩╗", "╩ ╩", ""]);
    char_map.insert('L', ["╦  ", "║  ", "╩═╝", ""]);
    char_map.insert('M', ["╔╦╗", "║║║", "╩ ╩", ""]);
    char_map.insert('N', ["╔╗╔", "║║║", "╝╚╝", ""]);
    char_map.insert('O', ["╔═╗", "║ ║", "╚═╝", ""]);
    char_map.insert('P', ["╔═╗", "╠═╝", "╩  ", ""]);
    char_map.insert('Q', ["╔═╗", "║═╬", "╚═╝", ""]);
    char_map.insert('R', ["╦═╗", "╠╦╝", "╩╚═", ""]);
    char_map.insert('S', ["╔═╗", "╚═╗", "╚═╝", ""]);
    char_map.insert('T', ["╔╦╗", " ║ ", " ╩ ", ""]);
    char_map.insert('U', ["╦ ╦", "║ ║", "╚═╝", ""]);
    char_map.insert('V', ["╦  ", "╦╚╗", "╔╝╚", ""]);
    char_map.insert('W', ["╦ ╦", "║║║", "╚╩╝", ""]);
    char_map.insert('X', ["═╗ ╦", "╔╩╦╝", "╩ ╚═", ""]);
    char_map.insert('Y', ["╦ ╦", "╚╦╝", " ╩ ", ""]);
    char_map.insert('Z', ["╔═╗", "╔═╝", "╚═╝", ""]);

    char_map.insert('a', ["┌─┐", "├─┤", "┴ ┴", ""]);
    char_map.insert('b', ["┌┐ ", "├┴┐", "└─┘", ""]);
    char_map.insert('c', ["┌─┐", "│  ", "└─┘", ""]);
    char_map.insert('d', ["┌┬┐", " ││", "─┴┘", ""]);
    char_map.insert('e', ["┌─┐", "├┤ ", "└─┘", ""]);
    char_map.insert('f', ["┌─┐", "├┤ ", "└  ", ""]);
    char_map.insert('g', ["┌─┐", "│ ┬", "└─┘", ""]);
    char_map.insert('h', ["┬ ┬", "├─┤", "┴ ┴", ""]);
    char_map.insert('i', [" ┬ ", " │ ", " ┴ ", ""]);
    char_map.insert('j', [" ┬ ", " │ ", " └┘", ""]);
    char_map.insert('k', ["┬┌─", "├┴┐", "┴ ┴", ""]);
    char_map.insert('l', ["┬  ", "│  ", "┴─┘", ""]);
    char_map.insert('m', ["┌┬┐", "│││", "┴ ┴", ""]);
    char_map.insert('n', ["┌┐┌", "│││", "┘└┘", ""]);
    char_map.insert('o', ["┌─┐", "│ │", "└─┘", ""]);
    char_map.insert('p', ["┌─┐", "├─┘", "┴  ", ""]);
    char_map.insert('q', ["┌─┐", "│─┼", "└─┘", ""]);
    char_map.insert('r', ["┬─┐", "├┬┘", "┴└─", ""]);
    char_map.insert('s', ["┌─┐", "└─┐", "└─┘", ""]);
    char_map.insert('t', ["┌┬┐", " │ ", " ┴ ", ""]);
    char_map.insert('u', ["┬  ", "┬└┐", "└┘ ", ""]);
    char_map.insert('v', ["┬  ", "└┐┌", " └┘", ""]);
    char_map.insert('w', ["┬ ┬", "│││", "└┴┘", ""]);
    char_map.insert('x', ["─┐ ", "┌┴┬", "┴ └", ""]);
    char_map.insert('y', ["┬ ┬", "└┬┘", " ┴ ", ""]);
    char_map.insert('z', ["┌─┐", "┌─┘", "└─┘", ""]);

    // Special characters
    char_map.insert(' ', ["         ", "", "", ""]);
    char_map.insert('!', [" ┬  ", " │  ", " o ", ""]);
    char_map.insert('?', ["┌─┐ ", " ┌┘ ", " o ", ""]);
    char_map.insert('_', ["      ", "     ", "     ", "───"]);
    char_map.insert('-', ["     ", " ─── ", "     ", ""]);
    char_map.insert(',', ["     ", "     ", "     ", "┘ "]);
    char_map.insert('.', ["     ", "     ", "     ", "o "]);
    char_map.insert('[', ["┌─ ", "│  ", "└─ ", ""]);
    char_map.insert(']', [" ─┐", "  │", " ─┘", ""]);
    char_map.insert('@', ["┌─┐", "│└┘", "└──", ""]);
    char_map.insert('$', ["┌┼┐", "└┼┐", "└┼┘", ""]);
    char_map.insert('%', ["O┬ ", " ┌┘ ", " ┴O", ""]);
    char_map.insert('^', [" /\ ", "     ", "     ", ""]);
    char_map.insert('&', [" ┬ ", "┌┼─", "└┘ ", ""]);
    char_map.insert('*', [" \\│/", "─ ─", "/│\\", ""]);
    char_map.insert('#', ["┼─┼", "│ │", "┼─┼", ""]);
    char_map.insert('\'', [" ┴  ", "     ", "     ", ""]);
    char_map.insert('"', [" ┴┴", "     ", "     ", ""]);
    char_map.insert('/', ["  / ", " / ", " / ", ""]);
    char_map.insert('\\', ["\\  ", " \\ ", "  \\", ""]);
    char_map.insert('+', [" │ ", " -+- ", " | ", ""]);
    char_map.insert('=', ["     ", "___", "   ───", ""]);
    char_map.insert(':', ["    ", " o  ", "  o ", ""]);
    char_map.insert(';', ["    ", " o  ", "  ┘ ", ""]);
    char_map.insert('~', ["   ", " /\\", "    ", ""]);
    char_map.insert('{', [" ┌ ", " <  ", " └  ", ""]);
    char_map.insert('}', [" ┐ ", "   >", " ┘ ", ""]);
    char_map.insert('|', [" | ", "  |", " | ", ""]);

    char_map
}

// Function to get character representation
fn get_char_representation(ch: char) -> Option<[&'static str; 4]> {
    let char_map = get_char_map();
    char_map.get(&ch).cloned()
}

fn is_all_space(segment: &str) -> bool {
    segment.chars().all(|c| c.is_whitespace())
}

fn print_title(input: &str, compact: bool) {
    let mut line1 = String::new();
    let mut line2 = String::new();
    let mut line3 = String::new();

    for ch in input.chars() {
        if let Some(char) = get_char_representation(ch) {
            let segment_length = char[0].len();

            let segment1 = char[0];
            let segment2 = char[1];
            let segment3 = char[2];

            if is_all_space(segment1) && is_all_space(segment2) && is_all_space(segment3) {
                line1.push(' ');
                line2.push(' ');
                line3.push(' ');
            } else {
                line1.push_str(segment1);
                line2.push_str(segment2);
                line3.push_str(segment3);
            }

            if !compact {
                line1.push(' ');
                line2.push(' ');
                line3.push(' ');
            }
        }
    }

    println!("{}", line1);
    println!("{}", line2);
    println!("{}", line3);
}

fn clear_lines(num_lines: usize) {
    for _ in 0..num_lines {
        print!("\x1B[1A\x1B[2K");
    }
    io::stdout().flush().unwrap();
}

fn title_marquee(input: &str, width: usize, speed: f64, compact: bool, separation: usize, prefix: &str, suffix: &str) {
    let text_width = width - prefix.len() - suffix.len();
    let separator: String = std::iter::repeat(' ').take(separation).collect();
    let repeat_factor = width / (input.len() + separation) + 2;
    let mut marquee_text = String::new();

    for _ in 0..repeat_factor {
        marquee_text.push_str(input);
        marquee_text.push_str(&separator);
    }

    let buffer = format!("{}{}", marquee_text, marquee_text);
    let mut offset = 0;

    loop {
        clear_lines(3);
        print_title(&format!("{}{}{}", prefix, &buffer[offset..offset + text_width], suffix), compact);
        offset = (offset + 1) % buffer.len();
        sleep(Duration::from_secs_f64(speed));
    }
}

fn console_height() -> usize {
    termion::terminal_size().unwrap().1 as usize
}

fn console_width() -> usize {
    termion::terminal_size().unwrap().0 as usize
}

fn progress_bar(percentage: usize, width: usize, prefix: &str, suffix: &str, empty_char: char, filled_char: char) {
    let percentage_width = 5;
    let bar_width = width - prefix.len() - suffix.len() - percentage_width;

    if percentage > 100 {
        println!("Error: Percentage must be between 0 and 100.");
        return;
    }

    let filled_length = percentage * bar_width / 100;
    let bar_filled: String = std::iter::repeat(filled_char).take(filled_length).collect();
    let bar_empty: String = std::iter::repeat(empty_char).take(bar_width - filled_length).collect();

    let suffix = format!("{}%{}", suffix, percentage);

    clear_lines(1);
    println!("{}{}{}{}", prefix, bar_filled, bar_empty, suffix);
}

fn marquee_progress_bar_percentage(percentage: usize) {
    if percentage > 100 {
        println!("Error: Percentage must be between 0 and 100.");
        return;
    }

    let suffix = if percentage < 10 {
        format!("  ]   %{}", percentage)
    } else if percentage < 100 {
        format!("  ]  %{}", percentage)
    } else {
        format!("  ] %{}", percentage)
    };

    print_marquee("--", 40, 0.1, 2, 3, percentage != 0, "[  ", &suffix);
}

fn marquee_wait() {
    loop {
        print_marquee("----", 80, 0.1, 5, 0, true, "", "");
    }
}

fn add_wait_marquee() {
    let handle = std::thread::spawn(marquee_wait);
    let wait_marquee_pid = handle.thread().id();
    std::thread::sleep(Duration::from_secs(2)); // Example sleep time

    // Trap interrupt signal (Ctrl+C) to stop the marquee
    ctrlc::set_handler(move || {
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");

    // Wait for the marquee thread to finish (it won't, so this is an infinite wait)
    handle.join().expect("Marquee thread has panicked");
}

fn remove_wait_marquee(wait_marquee_pid: std::thread::ThreadId) {
    // This function will stop the marquee thread by killing it (not really possible in Rust directly)
    // Instead, you would use a shared flag or message passing to signal the thread to stop
    // For the sake of this example, assume a shared AtomicBool is used
}

fn scrolling_marquee(message: &str, width: usize, speed: f64, lines_above: usize) {
    let message_length = message.len();
    let buffer = format!("{}{}", message, " ".repeat(width));

    loop {
        clear_lines(lines_above + 1);
        for i in 0..message_length + width {
            print!("\x1B[1A\x1B[2K");
            println!("{}", &buffer[i..i + width]);
            sleep(Duration::from_secs_f64(speed));
        }

        for _ in 0..lines_above {
            print!("\x1B[1A");
        }
    }
}

fn test(message: &str, width: usize, speed: f64, lines_above: usize) {
    scrolling_marquee(message, width, speed, lines_above);
}

fn change_color(color: &str) {
    print!("{}", color);
}

fn reset_color() {
    print!("{}", "\x1B[0m");
}

fn clear_screen() {
    print!("\x1B[2J");
}

fn screen_height() -> usize {
    termion::terminal_size().unwrap().1 as usize
}

fn screen_width() -> usize {
    termion::terminal_size().unwrap().0 as usize
}

fn move_cursor(row: usize, col: usize) {
    print!("\x1B[{};{}H", row, col);
}

fn print_at(y: usize, x: usize, text: &str) {
    move_cursor(y, x);
    print!("{}", text);
}

fn print_with_color_at(y: usize, x: usize, color: &str, text: &str) {
    let ansi_color = get_color(color);
    move_cursor(y, x);
    change_color(ansi_color);
    println!("{}", text);
    reset_color();
}

fn vertical_line(x: usize, y1: usize, y2: usize, color: &str, character: Option<char>) {
    let character = character.unwrap_or('|');
    change_color(color);

    for i in y1..=y2 {
        move_cursor(i, x);
        print!("{}", character);
    }
    reset_color();
}

fn horizontal_line(y: usize, x1: usize, x2: usize, color: &str, character: Option<char>) {
    let character = character.unwrap_or('-');
    change_color(color);

    for i in x1..=x2 {
        move_cursor(y, i);
        print!("{}", character);
    }
    reset_color();
}

fn box_draw(x1: usize, y1: usize, x2: usize, y2: usize, color: &str) {
    let ansi_color = get_color(color);
    change_color(ansi_color);

    vertical_line(x1, y1 + 1, y2 - 1, color, Some('|'));
    vertical_line(x2, y1 + 1, y2 - 1, color, Some('|'));

    horizontal_line(y1, x1 + 1, x2 - 1, color, Some('-'));
    horizontal_line(y2, x1 + 1, x2 - 1, color, Some('-'));

    print_at(y1, x1, "+");
    print_at(y1, x2, "+");
    print_at(y2, x1, "+");
    print_at(y2, x2, "+");

    reset_color();
}

fn get_color(color: &str) -> &'static str {
    match color {
        "red" => "\x1B[31m",
        "green" => "\x1B[32m",
        "yellow" => "\x1B[33m",
        "blue" => "\x1B[34m",
        "magenta" => "\x1B[35m",
        "cyan" => "\x1B[36m",
        "white" => "\x1B[37m",
        "black" => "\x1B[30m",
        _ => color,
    }
}

fn fill_box(x1: usize, y1: usize, x2: usize, y2: usize, fill_color: &str, fill_char: char) {
    let ansi_fill_color = get_color(fill_color);
    change_color(ansi_fill_color);

    for i in y1..=y2 {
        for j in x1..=x2 {
            move_cursor(i, j);
            print!("{}", fill_char);
        }
    }

    reset_color();
}

fn reset_terminal() {
    print!("\x1B[?7h\x1B[?25h\x1B[2J\x1B[;r\x1B[?1049l");
    Command::new("stty")
        .arg("echo")
        .output()
        .expect("Failed to execute command");
}
