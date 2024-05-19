use std::io::{self, Write};
use std::process::Command;

fn random_prefix() -> String {
    let output = Command::new("date")
        .arg("+%s")
        .output()
        .expect("Failed to execute command");

    let sha256sum = Command::new("sha256sum")
        .arg("-b")
        .stdin(output.stdout.as_slice())
        .output()
        .expect("Failed to execute command");

    let base64 = Command::new("base64")
        .stdin(sha256sum.stdout.as_slice())
        .output()
        .expect("Failed to execute command");

    let head = Command::new("head")
        .arg("-c")
        .arg("6")
        .stdin(base64.stdout.as_slice())
        .output()
        .expect("Failed to execute command");

    String::from_utf8(head.stdout).expect("Invalid UTF-8 sequence")
}

fn setup_terminal(max_items: i32) {
    // Setup the terminal for the TUI.
    // '\x1B[?1049h': Use alternative screen buffer.
    // '\x1B[?7l':    Disable line wrapping.
    // '\x1B[?25l':   Hide the cursor.
    // '\x1B[2J':     Clear the screen.
    // '\x1B[1;Nr':   Limit scrolling to scrolling area.
    //                Also sets cursor to (0,0).
    print!(
        "\x1B[?1049h\x1B[?7l\x1B[?25l\x1B[2J\x1B[1;{}r",
        max_items
    );
    io::stdout().flush().unwrap();

    // Hide echoing of user input
    Command::new("stty")
        .arg("-echo")
        .output()
        .expect("Failed to execute command");
}

fn get_term_size() -> (i32, i32) {
    let output = Command::new("stty")
        .arg("size")
        .output()
        .expect("Failed to execute command");
    let size = String::from_utf8(output.stdout).expect("Invalid UTF-8 sequence");
    let mut iter = size.split_whitespace();
    let lines = iter.next().unwrap().parse::<i32>().unwrap();
    let columns = iter.next().unwrap().parse::<i32>().unwrap();
    (lines, columns)
}

fn get_os() -> (&'static str, &'static str) {
    let os_type = std::env::consts::OS;
    match os_type {
        "macos" => ("open", "bIL"),
        "haiku" => {
            let trash_cmd = "trash"; // Change as needed
            let trash_dir = Command::new("finddir")
                .arg("-v")
                .arg("$PWD")
                .arg("B_TRASH_DIRECTORY")
                .output()
                .expect("Failed to execute command");

            let trash_dir_str = String::from_utf8(trash_dir.stdout).expect("Invalid UTF-8 sequence");
            let trash_dir_str = trash_dir_str.trim();
            Command::new("mkdir")
                .arg("-p")
                .arg(trash_dir_str)
                .output()
                .expect("Failed to execute command");
            ("open", "trash")
        }
        _ => ("xdg-open", ""),
    }
}

fn get_parent_box_path(prefix: &str, path: &str, separator: &str) -> Result<String, String> {
    if prefix.is_empty() || path.is_empty() {
        return Err("Usage: get_parent_box_path <prefix> <path>".to_string());
    }

    let count = path.matches(separator).count();
    if count == 0 {
        return Err(format!("Path '{}' is not a valid box path", path));
    }

    if count == 1 {
        return Ok(path.to_string());
    }

    if count == 2 {
        return Ok(format!("{}root_elem", prefix));
    } else {
        return Ok(path.split(separator).take(2).collect::<Vec<&str>>().join(separator));
    }
}

fn redraw_box(box_instance_id: &str, scroll_value: Option<i32>) -> Result<(), String> {
    let index_in_refreshing_boxes = REFRESHING_BOXES.iter().position(|x| x == box_instance_id);
    let scroll_value = match scroll_value {
        Some(value) => value,
        None => {
            if let Some(index) = index_in_refreshing_boxes {
                REFRESHING_BOXES_SCROLL_VALUES[index]
            } else {
                return Err("Usage: redraw_box <box_instance_id> [scroll_value]".to_string());
            }
        }
    };

    if box_instance_id.is_empty() {
        return Err("Usage: redraw_box <box_instance_id> [scroll_value]".to_string());
    }

    let is_root = instance_get_property(BOX_CLS_ID, box_instance_id, BOX_PROP_IS_ROOT);

    if let Some(is_root) = is_root {
        if is_root == "true" {
            clear_screen();
            draw_boxes(
                box_instance_id,
                0,
                0,
                screen_width(),
                screen_height(),
                scroll_value,
            );
        } else {
            let box_path;
            let box_parent_id = get_box_parent_id(box_instance_id)?;
            let parent_box_instance_id = get_box_instance_id(box_parent_id)?;
            let parent_abs_x1 = get_box_abs_x1(parent_box_instance_id)?;
            let parent_abs_y1 = get_box_abs_y1(parent_box_instance_id)?;
            let parent_abs_x2 = get_box_abs_x2(parent_box_instance_id)?;
            let parent_abs_y2 = get_box_abs_y2(parent_box_instance_id)?;
            let (box_absolute_x1, box_absolute_y1, box_absolute_x2, box_absolute_y2) =
                calculate_absolute_position(
                    box_instance_id,
                    parent_abs_x1,
                    parent_abs_y1,
                    parent_abs_x2,
                    parent_abs_y2,
                )?;

            draw_box(
                box_instance_id,
                box_absolute_x1,
                box_absolute_y1,
                box_absolute_x2,
                box_absolute_y2,
                scroll_value,
            );
        }
    }

    Ok(())
}

fn draw_box(
    box_instance_id: &str,
    absolute_x1: i32,
    absolute_y1: i32,
    absolute_x2: i32,
    absolute_y2: i32,
    scroll_value: Option<i32>,
) -> Result<(), String> {
    if box_instance_id.is_empty()
        || absolute_x1 < 0
        || absolute_y1 < 0
        || absolute_x2 < 0
        || absolute_y2 < 0
    {
        return Err(
            "Usage: draw_box <box_instance_id> <absolute_x1> <absolute_y1> <absolute_x2> <absolute_y2> [scroll_value]"
                .to_string(),
        );
    }

    let index_in_refreshing_boxes = REFRESHING_BOXES.iter().position(|x| x == box_instance_id);

    let scroll_value = match scroll_value {
        Some(value) => value,
        None => {
            if let Some(index) = index_in_refreshing_boxes {
                REFRESHING_BOXES_SCROLL_VALUES[index]
            } else {
                return Err("Usage: draw_box <box_instance_id> [scroll_value]".to_string());
            }
        }
    };

    let fill = get_box_fill(box_instance_id);

    if let Some(fill) = fill {
        if fill == "true" {
            let fill_color = get_box_fill_color(box_instance_id)?;
            let fill_char = get_box_fill_char(box_instance_id)?;
            fill_box(absolute_x1, absolute_y1, absolute_x2, absolute_y2, fill_color, fill_char);
        }
    }

    let border_color = get_box_border_color(box_instance_id);

    if let Some(border_color) = border_color {
        if box_instance_id == LAYOUT_SELECTED_BOX_INSTANCE_ID {
            border_color = LAYOUT_SELECTED_BOX_BORDER_COLOR.to_string();
        }
        box(
            absolute_x1,
            absolute_y1,
            absolute_x2,
            absolute_y2,
            border_color,
        );
    }

    let title = get_box_title(box_instance_id);

    if let Some(title) = title {
        let title_color = get_box_title_color(box_instance_id)?;
        print_with_color_at(
            absolute_y1 + 1,
            absolute_x1 + 1,
            title_color,
            title,
        );
    }

    let output = get_box_output(box_instance_id);

    if let Some(output) = output {
        let text_color = get_box_text_color(box_instance_id)?;
        let split_output = replace_with_newlines(&output);
        print_in_boxes(
            absolute_x1 + 3,
            absolute_y1 + 3,
            absolute_x2 - 2,
            absolute_y2 - 1,
            scroll_value,
            text_color,
            &split_output,
        );
    }

    if let Some(max_scroll_value) = REFRESHING_BOXES_MAX_SCROLL_VALUES[index_in_refreshing_boxes] {
        if max_scroll_value > 0 {
            let scroll_indicator = format!("[{}/{}]", scroll_value, max_scroll_value);
            print_with_color_at(
                absolute_y2 - 1,
                absolute_x2 - scroll_indicator.len() as i32 - 1,
                border_color,
                scroll_indicator,
            );
        }
    }

    Ok(())
}

fn replace_with_newlines(text: &str) -> String {
    text.replace("____", "\n")
}

fn calculate_absolute_position(
    box_instance_id: &str,
    parent_abs_x1: i32,
    parent_abs_y1: i32,
    parent_abs_x2: i32,
    parent_abs_y2: i32,
) -> Result<(i32, i32, i32, i32), String> {
    if box_instance_id.is_empty()
        || parent_abs_x1 < 0
        || parent_abs_y1 < 0
        || parent_abs_x2 < 0
        || parent_abs_y2 < 0
    {
        return Err("Usage: calculate_absolute_position <box_instance_id> <parent_abs_x1> <parent_abs_y1> <parent_abs_x2> <parent_abs_y2>".to_string());
    }

    let is_root = instance_get_property(BOX_CLS_ID, box_instance_id, BOX_PROP_IS_ROOT);

    let (abs_x1, abs_y1, abs_x2, abs_y2) = if let Some(is_root) = is_root {
        if is_root == "true" {
            (0, 0, screen_width(), screen_height())
        } else {
            let box_x1 = get_box_x1(box_instance_id)?;
            let box_y1 = get_box_y1(box_instance_id)?;
            let box_x2 = get_box_x2(box_instance_id)?;
            let box_y2 = get_box_y2(box_instance_id)?;

            let abs_x1 = parent_abs_x1 + (parent_abs_x2 - parent_abs_x1) * box_x1 / 100;
            let abs_y1 = parent_abs_y1 + (parent_abs_y2 - parent_abs_y1) * box_y1 / 100;
            let abs_x2 = parent_abs_x1 + (parent_abs_x2 - parent_abs_x1) * box_x2 / 100;
            let abs_y2 = parent_abs_y1 + (parent_abs_y2 - parent_abs_y1) * box_y2 / 100;

            (abs_x1, abs_y1, abs_x2, abs_y2)
        }
    } else {
        return Err("Instance property not found".to_string());
    };

    instance_set_property(BOX_CLS_ID, box_instance_id, BOX_PROP_ABS_X1, &abs_x1.to_string());
    instance_set_property(BOX_CLS_ID, box_instance_id, BOX_PROP_ABS_Y1, &abs_y1.to_string());
    instance_set_property(BOX_CLS_ID, box_instance_id, BOX_PROP_ABS_X2, &abs_x2.to_string());
    instance_set_property(BOX_CLS_ID, box_instance_id, BOX_PROP_ABS_Y2, &abs_y2.to_string());

    Ok((abs_x1, abs_y1, abs_x2, abs_y2))
}

fn print_in_boxes(
    x1: i32,
    y1: i32,
    x2: i32,
    y2: i32,
    start_line: i32,
    text_color: &str,
    text_lines: &str,
) {
    let box_width = x2 - x1 + 1;
    let box_height = y2 - y1 + 1;

    let lines: Vec<&str> = text_lines.split('\n').collect();
    let mut wrapped_lines = Vec::new();

    for i in (start_line as usize)..lines.len().min((start_line + box_height) as usize) {
        let mut text_line = lines[i];
        while text_line.len() > box_width as usize {
            wrapped_lines.push(&text_line[..box_width as usize]);
            text_line = &text_line[box_width as usize..];
        }
        wrapped_lines.push(text_line);
    }

    let mut y_pos = y1;
    for line in wrapped_lines {
        if y_pos > y2 {
            break;
        }
        print_with_color_at(y_pos, x1, text_color, line);
        y_pos += 1;
    }
}

fn draw_boxes(
    box_instance_id: &str,
    parent_abs_x1: i32,
    parent_abs_y1: i32,
    parent_abs_x2: i32,
    parent_abs_y2: i32,
    scroll_value: i32,
) -> Result<(), String> {
    if box_instance_id.is_empty() {
        return Err("Usage: draw_boxes <box_instance_id> [parent_abs_x1] [parent_abs_y1] [parent_abs_x2] [parent_abs_y2] [scroll_value]".to_string());
    }

    let index_in_refreshing_boxes = REFRESHING_BOXES.iter().position(|x| x == box_instance_id);

    let scroll_value = if let Some(index) = index_in_refreshing_boxes {
        REFRESHING_BOXES_SCROLL_VALUES[index]
    } else {
        return Err("Usage: draw_boxes <box_instance_id> [scroll_value]".to_string());
    };

    let (box_absolute_x1, box_absolute_y1, box_absolute_x2, box_absolute_y2) =
        calculate_absolute_position(
            box_instance_id,
            parent_abs_x1,
            parent_abs_y1,
            parent_abs_x2,
            parent_abs_y2,
        )?;

    draw_box(
        box_instance_id,
        box_absolute_x1,
        box_absolute_y1,
        box_absolute_x2,
        box_absolute_y2,
        Some(scroll_value),
    )?;

    let box_id = instance_get_property(BOX_CLS_ID, box_instance_id, BOX_PROP_ID);

    if let Some(box_id) = box_id {
        let childrens_instance_ids =
            instance_list_by_property(BOX_CLS_ID, BOX_PROP_PARENT_ID, &box_id);
        for child_instance_id in childrens_instance_ids {
            draw_boxes(
                &child_instance_id,
                box_absolute_x1,
                box_absolute_y1,
                box_absolute_x2,
                box_absolute_y2,
                scroll_value,
            )?;
        }
    }

    Ok(())
}
