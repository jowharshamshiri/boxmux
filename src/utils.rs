use shutil::pipe;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Write};
use std::process::{Command, Stdio};

use crate::{
    model::common::{Bounds, InputBounds, ScreenBuffer},
    Layout,
};
use termion::event::{Event, Key};
use termion::{raw::RawTerminal, screen::AlternateScreen};

pub fn screen_width() -> usize {
    termion::terminal_size().unwrap().0 as usize
}

pub fn screen_height() -> usize {
    termion::terminal_size().unwrap().1 as usize
}

pub fn screen_bounds() -> Bounds {
    Bounds {
        x1: 0,
        y1: 0,
        x2: screen_width(),
        y2: screen_height(),
    }
}

pub fn input_bounds_to_bounds(input_bounds: &InputBounds, parent_bounds: &Bounds) -> Bounds {
    let bx1 = parse_percentage(&input_bounds.x1, parent_bounds.width());
    let by1 = parse_percentage(&input_bounds.y1, parent_bounds.height());
    let bx2 = parse_percentage(&input_bounds.x2, parent_bounds.width());
    let by2 = parse_percentage(&input_bounds.y2, parent_bounds.height());
    let abs_x1 = parent_bounds.x1 + bx1;
    let abs_y1 = parent_bounds.y1 + by1;
    let abs_x2 = parent_bounds.x1 + bx2;
    let abs_y2 = parent_bounds.y1 + by2;
    Bounds {
        x1: abs_x1,
        y1: abs_y1,
        x2: abs_x2,
        y2: abs_y2,
    }
}

pub fn bounds_to_input_bounds(abs_bounds: &Bounds, parent_bounds: &Bounds) -> InputBounds {
    let width = parent_bounds.width();
    let height = parent_bounds.height();

    let ix1 = (abs_bounds.x1 - parent_bounds.x1) / width;
    let iy1 = (abs_bounds.y1 - parent_bounds.y1) / height;
    let ix2 = (abs_bounds.x2 - parent_bounds.x1) / width;
    let iy2 = (abs_bounds.y2 - parent_bounds.y1) / height;

    InputBounds {
        x1: format!("{}%", ix1 as f64 * 100.0),
        y1: format!("{}%", iy1 as f64 * 100.0),
        x2: format!("{}%", ix2 as f64 * 100.0),
        y2: format!("{}%", iy2 as f64 * 100.0),
    }
}

pub fn parse_percentage(value: &str, total: usize) -> usize {
    if value.ends_with('%') {
        let percentage = value.trim_end_matches('%').parse::<f64>().unwrap() / 100.0;
        (percentage * total as f64).round() as usize
    } else {
        value.parse::<usize>().unwrap()
    }
}

pub fn inherit_string(
    child_value: Option<&String>,
    parent_value: Option<&String>,
    parent_layout_value: Option<&String>,
    default_value: &str,
) -> String {
    if let Some(value) = child_value {
        if !value.is_empty() {
            return value.clone();
        }
    }
    if let Some(value) = parent_value {
        if !value.is_empty() {
            return value.clone();
        }
    }
    if let Some(value) = parent_layout_value {
        if !value.is_empty() {
            return value.clone();
        }
    }
    default_value.to_string()
}

pub fn inherit_char(
    child_char: Option<&char>,
    parent_char: Option<&char>,
    parent_layout_char: Option<&char>,
    default_char: char,
) -> char {
    if let Some(&char) = child_char {
        return char;
    }
    if let Some(&char) = parent_char {
        return char;
    }
    if let Some(&char) = parent_layout_char {
        return char;
    }
    default_char
}

pub fn inherit_bool(
    child_bool: Option<&bool>,
    parent_bool: Option<&bool>,
    parent_layout_bool: Option<&bool>,
    default_bool: bool,
) -> bool {
    if let Some(bool) = child_bool {
        return *bool;
    }
    if let Some(bool) = parent_bool {
        return *bool;
    }
    if let Some(bool) = parent_layout_bool {
        return *bool;
    }
    default_bool
}

pub fn inherit_u64(
    child_value: Option<&u64>,
    parent_value: Option<&u64>,
    parent_layout_value: Option<&u64>,
    default_value: u64,
) -> u64 {
    if let Some(value) = child_value {
        return *value;
    }
    if let Some(value) = parent_value {
        return *value;
    }
    if let Some(value) = parent_layout_value {
        return *value;
    }
    default_value
}

pub fn inherit_i64(
    child_value: Option<&i64>,
    parent_value: Option<&i64>,
    parent_layout_value: Option<&i64>,
    default_value: i64,
) -> i64 {
    if let Some(value) = child_value {
        return *value;
    }
    if let Some(value) = parent_value {
        return *value;
    }
    if let Some(value) = parent_layout_value {
        return *value;
    }
    default_value
}

pub fn inherit_f64(
    child_value: Option<&f64>,
    parent_value: Option<&f64>,
    parent_layout_value: Option<&f64>,
    default_value: f64,
) -> f64 {
    if let Some(value) = child_value {
        return *value;
    }
    if let Some(value) = parent_value {
        return *value;
    }
    if let Some(value) = parent_layout_value {
        return *value;
    }
    default_value
}

pub fn inherit_optional_string(
    child_value: Option<&String>,
    parent_value: Option<&String>,
    parent_layout_value: Option<&String>,
    default_value: Option<String>,
) -> Option<String> {
    if let Some(value) = child_value {
        if !value.is_empty() {
            return Some(value.clone());
        }
    }
    if let Some(value) = parent_value {
        if !value.is_empty() {
            return Some(value.clone());
        }
    }
    if let Some(value) = parent_layout_value {
        if !value.is_empty() {
            return Some(value.clone());
        }
    }
    default_value
}

pub fn inherit_optional_char(
    child_char: Option<&char>,
    parent_char: Option<&char>,
    parent_layout_char: Option<&char>,
    default_char: Option<char>,
) -> Option<char> {
    if let Some(&char) = child_char {
        return Some(char);
    }
    if let Some(&char) = parent_char {
        return Some(char);
    }
    if let Some(&char) = parent_layout_char {
        return Some(char);
    }
    default_char
}

pub fn inherit_optional_bool(
    child_bool: Option<&bool>,
    parent_bool: Option<&bool>,
    parent_layout_bool: Option<&bool>,
    default_bool: Option<bool>,
) -> Option<bool> {
    if let Some(bool) = child_bool {
        return Some(*bool);
    }
    if let Some(bool) = parent_bool {
        return Some(*bool);
    }
    if let Some(bool) = parent_layout_bool {
        return Some(*bool);
    }
    default_bool
}

pub fn inherit_optional_u64(
    child_value: Option<&u64>,
    parent_value: Option<&u64>,
    parent_layout_value: Option<&u64>,
    default_value: Option<u64>,
) -> Option<u64> {
    if let Some(value) = child_value {
        return Some(*value);
    }
    if let Some(value) = parent_value {
        return Some(*value);
    }
    if let Some(value) = parent_layout_value {
        return Some(*value);
    }
    default_value
}

pub fn inherit_optional_i64(
    child_value: Option<&i64>,
    parent_value: Option<&i64>,
    parent_layout_value: Option<&i64>,
    default_value: Option<i64>,
) -> Option<i64> {
    if let Some(value) = child_value {
        return Some(*value);
    }
    if let Some(value) = parent_value {
        return Some(*value);
    }
    if let Some(value) = parent_layout_value {
        return Some(*value);
    }
    default_value
}

pub fn inherit_optional_f64(
    child_value: Option<&f64>,
    parent_value: Option<&f64>,
    parent_layout_value: Option<&f64>,
    default_value: Option<f64>,
) -> Option<f64> {
    if let Some(value) = child_value {
        return Some(*value);
    }
    if let Some(value) = parent_value {
        return Some(*value);
    }
    if let Some(value) = parent_layout_value {
        return Some(*value);
    }
    default_value
}

pub fn apply_buffer(
    screen_buffer: &mut ScreenBuffer,
    alternate_screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
) {
    for y in 0..screen_buffer.height {
        for x in 0..screen_buffer.width {
            if let Some(cell) = screen_buffer.get(x, y) {
                write!(
                    alternate_screen,
                    "{}{}{}{}",
                    termion::cursor::Goto((x + 1) as u16, (y + 1) as u16),
                    cell.bg_color,
                    cell.fg_color,
                    cell.ch
                )
                .unwrap();
            }
        }
    }
    alternate_screen.flush().unwrap();
}

pub fn apply_buffer_if_changed(
    previous_buffer: &ScreenBuffer,
    current_buffer: &ScreenBuffer,
    alternate_screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
) {
    for y in 0..current_buffer.height {
        let mut last_changed_index: Option<u16> = None;
        let mut changes = Vec::new(); // Store changes for each line

        for x in 0..current_buffer.width {
            let current_cell = current_buffer.get(x, y);
            let previous_cell = previous_buffer.get(x, y);

            if current_cell != previous_cell {
                if last_changed_index.is_none() {
                    last_changed_index = Some(x as u16); // Mark the start of a change sequence
                }
                if let Some(cell) = current_cell {
                    changes.push(cell.clone()); // Accumulate changes
                }
            } else {
                // When encountering the end of a sequence of changes, flush them
                if let Some(start) = last_changed_index {
                    write!(
                        alternate_screen,
                        "{}",
                        termion::cursor::Goto(start + 1, y as u16 + 1)
                    )
                    .unwrap();
                    for cell in &changes {
                        write!(
                            alternate_screen,
                            "{}{}{}",
                            cell.bg_color, cell.fg_color, cell.ch
                        )
                        .unwrap();
                    }
                    changes.clear();
                    last_changed_index = None;
                }
            }
        }

        // Check if there's a pending sequence at the end of the line
        if let Some(start) = last_changed_index {
            write!(
                alternate_screen,
                "{}",
                termion::cursor::Goto(start + 1, y as u16 + 1)
            )
            .unwrap();
            for cell in changes {
                write!(
                    alternate_screen,
                    "{}{}{}",
                    cell.bg_color, cell.fg_color, cell.ch
                )
                .unwrap();
            }
        }
    }
    alternate_screen.flush().unwrap(); // Make sure to flush only once after all changes
}

pub fn find_selected_panel_uuid(layout: &Layout) -> Option<String> {
    if let Some(children) = &layout.children {
        for panel in children {
            if let Some(selected) = panel.selected {
                if selected {
                    return Some(panel.id.clone());
                }
            }
        }
    }

    None
}

pub fn calculate_tab_order(layout: &Layout) -> Vec<String> {
    let mut result: HashMap<String, i32> = HashMap::new();

    if let Some(children) = &layout.children {
        for panel in children {
            let tab_order = panel.tab_order.clone();
            if tab_order.is_some() {
                result.insert(
                    panel.id.clone(),
                    tab_order
                        .unwrap()
                        .parse::<i32>()
                        .expect("Invalid tab order"),
                );
            }
        }
    }

    // Sort the hashmap by value
    let mut sorted_result: Vec<(String, i32)> = result.into_iter().collect();
    sorted_result.sort_by(|a, b| a.1.cmp(&b.1));

    let mut tab_order: Vec<String> = Vec::new();
    for (key, _) in sorted_result {
        tab_order.push(key);
    }

    tab_order
}

pub fn find_next_panel_uuid(layout: &Layout, current_panel_uuid: &str) -> Option<String> {
    let tab_order = calculate_tab_order(layout);
    let mut found_current_panel = false;

    for panel_uuid in tab_order {
        if found_current_panel {
            return Some(panel_uuid);
        }

        if panel_uuid == current_panel_uuid {
            found_current_panel = true;
        }
    }

    None
}

pub fn find_previous_panel_uuid(layout: &Layout, current_panel_uuid: &str) -> Option<String> {
    let tab_order = calculate_tab_order(layout);
    let mut previous_panel_uuid: Option<String> = None;

    for panel_uuid in tab_order {
        if panel_uuid == current_panel_uuid {
            return previous_panel_uuid;
        }

        previous_panel_uuid = Some(panel_uuid);
    }

    None
}
pub fn run_script(libs_paths: Option<Vec<String>>, script: &Vec<String>) -> io::Result<String> {
    // Create the script content in-memory
    let mut script_content = String::new();
    if let Some(paths) = libs_paths {
        for lib in paths {
            script_content.push_str(&format!("source {}\n", lib));
        }
    }

    // Add the script commands to the script content
    for command in script {
        script_content.push_str(&format!("{}\n", command));
    }

    // println!("Script content:\n{}", script_content); // Debugging output

    // Create the command sequence
    let command_sequence = vec![vec!["bash", "-c", &script_content]];

    // Execute the command sequence
    let output = pipe(command_sequence).unwrap_or_else(|e| format!("Error: {:?}", e));

    Ok(output)
}
// pub fn run_script3(libs_paths: Option<Vec<String>>, script: &Vec<String>) -> io::Result<String> {
//     // Create the script content in-memory
//     let mut script_content = String::new();
//     if let Some(paths) = libs_paths {
//         for lib in paths {
//             script_content.push_str(&format!("source {}\n", lib));
//         }
//     }

//     // Add the script commands to the script content
//     for command in script {
//         script_content.push_str(&format!("{}\n", command));
//     }

//     // Save the script content to a temporary file for debugging
//     // let mut file = File::create("script.sh").expect("Failed to create script file");
//     // file.write_all(script_content.as_bytes())
//     //     .expect("Failed to write to script file");

//     // let output = pipe! {
//     //     script_content.as_bytes() => |cmd| {
//     //         let mut output = String::new();
//     //         cmd.stdout.as_mut().unwrap().read_to_string(&mut output).unwrap();
//     //         output
//     //     }
//     // };
//     // let output = pipe(script_content)?;

//     // // Spawn the bash process
//     // let mut cmd = Command::new("bash")
//     //     .arg("-c")
//     //     .arg("-")
//     //     .stdin(Stdio::piped())
//     //     .stdout(Stdio::piped())
//     //     .stderr(Stdio::piped())
//     //     .spawn()
//     //     .expect("Failed to spawn bash process");

//     // {
//     //     // Write the script content to the stdin of the bash process
//     //     let stdin = cmd.stdin.as_mut().expect("Failed to open stdin");
//     //     stdin
//     //         .write_all(script_content.as_bytes())
//     //         .expect("Failed to write to stdin");
//     // }

//     // // Wait for the command to complete and get the output
//     // let output = cmd.wait_with_output().expect("Failed to read stdout");

//     // // Combine stdout and stderr into a single string
//     // let combined_output = format!(
//     //     "{}{}",
//     //     String::from_utf8_lossy(&output.stdout),
//     //     String::from_utf8_lossy(&output.stderr)
//     // );

//     Ok(output)
// }

pub fn normalize_key_str(key_str: &str) -> String {
    key_str.to_lowercase().replace(" ", "").replace("+", "")
}

pub fn extract_key_str(event: Event) -> Option<String> {
    match event {
        Event::Key(Key::Char(' ')) => Some("Space".to_string()),
        Event::Key(Key::Char('\n')) => Some("Return".to_string()),
        Event::Key(Key::Char('\t')) => Some("Tab".to_string()),
        Event::Key(Key::Char(c)) => Some(c.to_string()),
        Event::Key(Key::Ctrl(c)) => Some(format!("Ctrl+{}", c)),
        Event::Key(Key::Alt(c)) => Some(format!("Alt+{}", c)),
        Event::Key(Key::Left) => Some("Left".to_string()),
        Event::Key(Key::Right) => Some("Right".to_string()),
        Event::Key(Key::Up) => Some("Up".to_string()),
        Event::Key(Key::Down) => Some("Down".to_string()),
        Event::Key(Key::Backspace) => Some("Backspace".to_string()),
        Event::Key(Key::Delete) => Some("Delete".to_string()),
        Event::Key(Key::Esc) => Some("Esc".to_string()),
        Event::Key(Key::BackTab) => Some("BackTab".to_string()),
        Event::Key(Key::Home) => Some("Home".to_string()),
        Event::Key(Key::End) => Some("End".to_string()),
        Event::Key(Key::PageUp) => Some("PageUp".to_string()),
        Event::Key(Key::PageDown) => Some("PageDown".to_string()),
        Event::Key(Key::F(n)) => Some(format!("F{}", n)),
        Event::Key(Key::Insert) => Some("Insert".to_string()),
        _ => None,
    }
}

pub fn key_str_to_translate_whitespace(original_str: &str) -> String {
    let mut replacements = HashMap::new();
    replacements.insert(" ", "Space");
    replacements.insert("\n", "Return");
    replacements.insert("\t", "Tab");
    replacements.insert("\x08", "Backspace");
    replacements.insert("\x7f", "Delete");
    replacements.insert("\x1b", "Esc");
    replacements.insert("\x09", "BackTab");
    replacements.insert("\x1b[H", "Home");
    replacements.insert("\x1b[F", "End");
    replacements.insert("\x1b[5~", "PageUp");
    replacements.insert("\x1b[6~", "PageDown");
    replacements.insert("\x1b[2~", "Insert");
    replacements.insert("\x1bOP", "F1");
    replacements.insert("\x1bOQ", "F2");
    replacements.insert("\x1bOR", "F3");
    replacements.insert("\x1bOS", "F4");
    replacements.insert("\x1b[15~", "F5");
    replacements.insert("\x1b[17~", "F6");
    replacements.insert("\x1b[18~", "F7");
    replacements.insert("\x1b[19~", "F8");
    replacements.insert("\x1b[20~", "F9");
    replacements.insert("\x1b[21~", "F10");
    replacements.insert("\x1b[23~", "F11");
    replacements.insert("\x1b[24~", "F12");
    replacements.insert("\x1b[A", "Up");
    replacements.insert("\x1b[B", "Down");
    replacements.insert("\x1b[C", "Right");
    replacements.insert("\x1b[D", "Left");

    let mut result = original_str.to_string();
    for (key, value) in &replacements {
        result = result.replace(key, value);
    }
    result
}

pub fn handle_keypress(
    key_str: &str,
    key_mappings: &HashMap<String, Vec<String>>,
) -> Option<Vec<String>> {
    let normalized_key_str = normalize_key_str(&key_str_to_translate_whitespace(key_str));

    for (key, actions) in key_mappings.iter() {
        if normalize_key_str(key) == normalized_key_str {
            return Some(actions.clone());
        }
    }
    None
}
