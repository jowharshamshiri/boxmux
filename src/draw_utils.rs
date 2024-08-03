use termion::color;

use crate::{
    set_terminal_title, AppContext, AppGraph, Bounds, Choice, Layout, Panel, ScreenBuffer,
};
use std::collections::HashMap;

use crate::model::common::Cell;
use crate::utils::screen_bounds;

pub fn content_size(text: &str) -> (usize, usize) {
    let mut width = 0;
    let mut height = 0;
    for line in text.lines() {
        width = width.max(line.len());
        height += 1;
    }
    (width, height)
}

pub fn get_fg_color(color: &str) -> String {
    match color {
        "red" => format!("{}", color::Fg(color::Red)),
        "green" => format!("{}", color::Fg(color::Green)),
        "yellow" => format!("{}", color::Fg(color::Yellow)),
        "blue" => format!("{}", color::Fg(color::Blue)),
        "magenta" => format!("{}", color::Fg(color::Magenta)),
        "cyan" => format!("{}", color::Fg(color::Cyan)),
        "white" => format!("{}", color::Fg(color::White)),
        "black" => format!("{}", color::Fg(color::Black)),
        "reset" => format!("{}", color::Fg(color::Reset)),
        "bright_black" => format!("{}", color::Fg(color::AnsiValue(8))),
        "bright_red" => format!("{}", color::Fg(color::AnsiValue(9))),
        "bright_green" => format!("{}", color::Fg(color::AnsiValue(10))),
        "bright_yellow" => format!("{}", color::Fg(color::AnsiValue(11))),
        "bright_blue" => format!("{}", color::Fg(color::AnsiValue(12))),
        "bright_magenta" => format!("{}", color::Fg(color::AnsiValue(13))),
        "bright_cyan" => format!("{}", color::Fg(color::AnsiValue(14))),
        "bright_white" => format!("{}", color::Fg(color::AnsiValue(15))),
        _ => format!("{}", color::Fg(color::Reset)),
    }
}

pub fn get_bg_color(color: &str) -> String {
    match color {
        "red" => format!("{}", color::Bg(color::Red)),
        "green" => format!("{}", color::Bg(color::Green)),
        "yellow" => format!("{}", color::Bg(color::Yellow)),
        "blue" => format!("{}", color::Bg(color::Blue)),
        "magenta" => format!("{}", color::Bg(color::Magenta)),
        "cyan" => format!("{}", color::Bg(color::Cyan)),
        "white" => format!("{}", color::Bg(color::White)),
        "black" => format!("{}", color::Bg(color::Black)),
        "reset" => format!("{}", color::Bg(color::Reset)),
        "bright_black" => format!("{}", color::Bg(color::AnsiValue(8))),
        "bright_red" => format!("{}", color::Bg(color::AnsiValue(9))),
        "bright_green" => format!("{}", color::Bg(color::AnsiValue(10))),
        "bright_yellow" => format!("{}", color::Bg(color::AnsiValue(11))),
        "bright_blue" => format!("{}", color::Bg(color::AnsiValue(12))),
        "bright_magenta" => format!("{}", color::Bg(color::AnsiValue(13))),
        "bright_cyan" => format!("{}", color::Bg(color::AnsiValue(14))),
        "bright_white" => format!("{}", color::Bg(color::AnsiValue(15))),
        _ => format!("{}", color::Bg(color::Reset)),
    }
}

pub fn draw_app(
    app_context: &AppContext,
    app_graph: &AppGraph,
    adjusted_bounds: &HashMap<String, HashMap<String, Bounds>>,
    buffer: &mut ScreenBuffer,
) {
    let active_layout = app_context
        .app
        .get_active_layout()
        .expect("No active layout found!")
        .clone();
    draw_layout(
        app_context,
        app_graph,
        adjusted_bounds,
        &active_layout,
        buffer,
    );
}

pub fn draw_layout(
    app_context: &AppContext,
    app_graph: &AppGraph,
    adjusted_bounds: &HashMap<String, HashMap<String, Bounds>>,
    layout: &Layout,
    buffer: &mut ScreenBuffer,
) {
    let cloned_layout = layout.clone();
    let bg_color = cloned_layout.bg_color.unwrap_or("black".to_string());
    let fill_char = cloned_layout.fill_char.unwrap_or(' ');

    // Set the background for the layout
    fill_panel(&screen_bounds(), false, &bg_color, fill_char, buffer);

    if let Some(layout_title) = &cloned_layout.title {
        if !layout_title.trim().is_empty() {
            set_terminal_title(layout_title);
        }
    }

    if let Some(children) = &cloned_layout.children {
        for panel in children.iter() {
            draw_panel(
                app_context,
                app_graph,
                adjusted_bounds,
                layout,
                &panel,
                buffer,
            );
        }
    }
}

pub fn draw_panel(
    app_context: &AppContext,
    app_graph: &AppGraph,
    adjusted_bounds: &HashMap<String, HashMap<String, Bounds>>,
    layout: &Layout,
    panel: &Panel,
    buffer: &mut ScreenBuffer,
) {
    let panel_parent = app_graph.get_parent(&layout.id, &panel.id);

    let layout_adjusted_bounds = adjusted_bounds.get(&layout.id);

    let mut panel_adjusted_bounds = None;
    match layout_adjusted_bounds {
        Some(value) => panel_adjusted_bounds = value.get(&panel.id),
        None => println!("Calculated bounds for layout {} not found", &layout.id),
    }

    match panel_adjusted_bounds {
        Some(value) => {
            let bg_color = panel.calc_bg_color(app_context, app_graph).to_string();
            let parent_bg_color = if panel_parent.is_none() {
                layout.bg_color.clone().unwrap_or("black".to_string())
            } else {
                panel_parent
                    .unwrap()
                    .calc_bg_color(app_context, app_graph)
                    .to_string()
            };
            let fg_color = panel.calc_fg_color(app_context, app_graph).to_string();

            let title_bg_color = panel
                .calc_title_bg_color(app_context, app_graph)
                .to_string();
            let title_fg_color = panel
                .calc_title_fg_color(app_context, app_graph)
                .to_string();
            let border = panel.calc_border(app_context, app_graph);
            let border_color = panel.calc_border_color(app_context, app_graph).to_string();
            let fill_char = panel.calc_fill_char(app_context, app_graph);

            // Draw fill
            fill_panel(&value, border, &bg_color, fill_char, buffer);

            let mut content = panel.content.as_deref();
            // check output is not null or empty
            if !panel.output.is_empty() {
                content = Some(&panel.output);
            }

            render_panel(
                &value,
                &border_color,
                &bg_color,
                &parent_bg_color,
                panel.title.as_deref(),
                &title_fg_color,
                &title_bg_color,
                &panel.calc_title_position(app_context, app_graph),
                panel.choices.clone(),
                &panel.calc_menu_fg_color(app_context, app_graph),
                &panel.calc_menu_bg_color(app_context, app_graph),
                &panel.calc_selected_menu_fg_color(app_context, app_graph),
                &panel.calc_selected_menu_bg_color(app_context, app_graph),
                content,
                &fg_color,
                &panel.calc_overflow_behavior(app_context, app_graph),
                Some(&panel.calc_border(app_context, app_graph)),
                panel.current_horizontal_scroll(),
                panel.current_vertical_scroll(),
                buffer,
            );

            // Draw children
            if let Some(children) = &panel.children {
                for child in children.iter() {
                    draw_panel(
                        app_context,
                        app_graph,
                        adjusted_bounds,
                        layout,
                        child,
                        buffer,
                    );
                }
            }
        }
        None => println!("Calculated bounds for panel {} not found", &panel.id),
    }
}

pub fn print_with_color_and_background_at(
    y: usize,
    x: usize,
    fg_color: &str,
    bg_color: &str,
    text: &str,
    buffer: &mut ScreenBuffer,
) {
    let fg_color_code = get_fg_color(fg_color);
    let bg_color_code = get_bg_color(bg_color);
    for (i, mut ch) in text.chars().enumerate() {
        let cell = Cell {
            fg_color: fg_color_code.clone(),
            bg_color: bg_color_code.clone(),
            ch,
        };
        buffer.update(x + i, y, cell);
    }
}

pub fn draw_horizontal_line(
    y: usize,
    x1: usize,
    x2: usize,
    border_color: &str,
    bg_color: &str,
    buffer: &mut ScreenBuffer,
) {
    let border_color_code = get_fg_color(border_color);
    let bg_color_code = get_bg_color(bg_color);
    for x in x1..=x2 {
        let cell = Cell {
            fg_color: border_color_code.clone(),
            bg_color: bg_color_code.clone(),
            ch: '─',
        };
        buffer.update(x, y, cell);
    }
}

pub fn draw_vertical_line(
    x: usize,
    y1: usize,
    y2: usize,
    border_color: &str,
    bg_color: &str,
    buffer: &mut ScreenBuffer,
) {
    let border_color_code = get_fg_color(border_color);
    let bg_color_code = get_bg_color(bg_color);
    for y in y1..=y2 {
        let cell = Cell {
            fg_color: border_color_code.clone(),
            bg_color: bg_color_code.clone(),
            ch: '│',
        };
        buffer.update(x, y, cell);
    }
}

pub fn erase_to_background_color(
    x: usize,
    y: usize,
    width: usize,
    height: usize,
    bg_color: &str,
    buffer: &mut ScreenBuffer,
) {
    let bg_color_code = get_bg_color(bg_color);
    for y in y..y + height {
        for x in x..x + width {
            let cell = Cell {
                fg_color: "default".to_string(),
                bg_color: bg_color_code.clone(),
                ch: ' ',
            };
            buffer.update(x, y, cell);
        }
    }
}

pub fn draw_horizontal_line_with_title(
    y: usize,
    x1: usize,
    x2: usize,
    fg_color: &str,
    bg_color: &str,
    title: Option<&str>,
    title_fg_color: &str,
    title_bg_color: &str,
    title_position: &str,
    draw_border: bool,
    buffer: &mut ScreenBuffer,
) {
    let width = x2.saturating_sub(x1);
    let title_padding = 2; // Adjust padding if needed

    if let Some(title) = title {
        let formatted_title = format!(" {} ", title);
        let title_length = formatted_title.len();

        if title_length <= width {
            let (title_start_position, line_before_title_length, line_after_title_length) =
                match title_position {
                    "start" => {
                        let title_start_position = x1 + title_padding;
                        let line_before_title_length = title_start_position.saturating_sub(x1);
                        let line_after_title_length =
                            width.saturating_sub(line_before_title_length + title_length);
                        (
                            title_start_position,
                            line_before_title_length,
                            line_after_title_length,
                        )
                    }
                    "center" => {
                        let title_start_position = x1 + (width.saturating_sub(title_length)) / 2;
                        let line_before_title_length = title_start_position.saturating_sub(x1);
                        let line_after_title_length =
                            width.saturating_sub(line_before_title_length + title_length);
                        (
                            title_start_position,
                            line_before_title_length,
                            line_after_title_length,
                        )
                    }
                    "end" => {
                        let title_start_position = x2.saturating_sub(title_length + title_padding);
                        let line_before_title_length = title_start_position.saturating_sub(x1);
                        let line_after_title_length =
                            x2.saturating_sub(title_start_position + title_length);
                        (
                            title_start_position,
                            line_before_title_length,
                            line_after_title_length,
                        )
                    }
                    _ => (x1, width, 0), // Default to no title
                };

            if draw_border {
                draw_horizontal_line(
                    y,
                    x1,
                    x1 + line_before_title_length,
                    fg_color,
                    bg_color,
                    buffer,
                );
            }

            print_with_color_and_background_at(
                y,
                title_start_position,
                title_fg_color,
                title_bg_color,
                &formatted_title,
                buffer,
            );

            if draw_border {
                draw_horizontal_line(
                    y,
                    x1 + line_before_title_length + title_length,
                    x2,
                    fg_color,
                    bg_color,
                    buffer,
                );
            }
        } else {
            if draw_border {
                // If the title is too long, just draw a line without the title
                draw_horizontal_line(y, x1, x2, fg_color, bg_color, buffer);
            }
        }
    } else {
        if draw_border {
            // If there is no title, just draw a full horizontal line
            draw_horizontal_line(y, x1, x2, fg_color, bg_color, buffer);
        }
    }
}

static H_SCROLL_CHAR: &str = "|";
static V_SCROLL_CHAR: &str = "-";

pub fn render_panel(
    bounds: &Bounds,
    border_color: &str,
    bg_color: &str,
    parent_bg_color: &str,
    title: Option<&str>,
    title_fg_color: &str,
    title_bg_color: &str,
    title_position: &str,
    choices: Option<Vec<Choice>>,
    menu_fg_color: &str,
    menu_bg_color: &str,
    selected_menu_fg_color: &str,
    selected_menu_bg_color: &str,
    content: Option<&str>,
    fg_color: &str,
    overflow_behavior: &str,
    border: Option<&bool>,
    horizontal_scroll: f64,
    vertical_scroll: f64,
    buffer: &mut ScreenBuffer,
) {
    let draw_border = border.unwrap_or(&true);
    let border_color_code = get_fg_color(border_color);
    // let fg_color_code = get_fg_color(fg_color);
    let bg_color_code = get_bg_color(bg_color);
    let parent_bg_color_code = get_bg_color(parent_bg_color);
    let mut _title_overflowing = false;
    let mut _x_offset = 0;
    let mut _y_offset = 0;
    let mut _overflowing = false;
    let mut scrollbars_drawn = false;

    // Ensure bounds stay within screen limits
    let screen_bounds = screen_bounds();
    let bounds = bounds
        .intersection(&screen_bounds)
        .unwrap_or_else(|| bounds.clone());

    // Draw top border with title
    let top_border_length = bounds.width();
    if let Some(title) = title {
        let mut formatted_title = format!(" {} ", title);
        let title_length = formatted_title.len();
        let mut max_title_length = title_length;

        if title_length < top_border_length {
            if top_border_length < 5 {
                _title_overflowing = true;
            } else {
                if max_title_length > top_border_length.saturating_sub(3) {
                    max_title_length = top_border_length.saturating_sub(3);
                }

                if title_length > max_title_length {
                    formatted_title = format!("{}...", &formatted_title[..max_title_length]);
                }
            }
        }

        if !_title_overflowing {
            draw_horizontal_line_with_title(
                bounds.top(),
                bounds.left(),
                bounds.right(),
                border_color,
                bg_color,
                Some(&formatted_title),
                title_fg_color,
                title_bg_color,
                title_position,
                *draw_border,
                buffer,
            );
        }
    } else {
        if *draw_border {
            draw_horizontal_line(
                bounds.top(),
                bounds.left(),
                bounds.right(),
                border_color,
                bg_color,
                buffer,
            );
        }
    }

    if let Some(choices) = choices {
        let mut y_position = bounds.top() + 1; // Start drawing menu items below the border
        for choice in choices {
            if y_position > bounds.bottom() - 1 {
                break; // Don't draw outside the bounds
            }

            let fg_color = if choice.selected {
                selected_menu_fg_color
            } else {
                menu_fg_color
            };
            let bg_color = if choice.selected {
                selected_menu_bg_color
            } else {
                menu_bg_color
            };

            print_with_color_and_background_at(
                y_position,
                bounds.left() + 2,
                fg_color,
                bg_color,
                &choice.content.unwrap(),
                buffer,
            );
            y_position += 1;
        }
    } else if let Some(content) = content {
        let (content_width, content_height) = content_size(content);
        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        let content_lines: Vec<&str> = content.lines().collect();
        let max_content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        let max_content_height = content_lines.len();

        if content_width > viewable_width {
            _x_offset = (max_content_width as f64 * horizontal_scroll / 100.0).round() as usize;
        }

        if content_height > viewable_height {
            _y_offset = (max_content_height as f64 * vertical_scroll / 100.0).round() as usize;
        }

        _overflowing = content_width > viewable_width || content_height > viewable_height;

        if _overflowing && overflow_behavior == "scroll" {
            let viewable_width = bounds.width();
            let viewable_height = bounds.height().saturating_sub(1); // minus one to avoid overwriting the bottom border

            // Calculate the maximum allowable offsets based on content size
            let max_horizontal_offset = max_content_width.saturating_sub(viewable_width) + 3;
            let max_vertical_offset = max_content_height.saturating_sub(viewable_height);

            // Calculate the current offsets based on scroll percentages
            let horizontal_offset =
                ((horizontal_scroll / 100.0) * max_horizontal_offset as f64).floor() as usize;
            let vertical_offset =
                ((vertical_scroll / 100.0) * max_vertical_offset as f64).floor() as usize;

            // Ensure the offsets are within the allowable range
            let horizontal_offset = horizontal_offset.min(max_horizontal_offset);
            let vertical_offset = vertical_offset.min(max_vertical_offset);

            let visible_lines = content_lines
                .iter()
                .skip(vertical_offset)
                .take(viewable_height);

            let total_lines = content_lines.len();
            let vertical_padding = (viewable_height.saturating_sub(total_lines)) / 2;
            let horizontal_padding = (viewable_width.saturating_sub(max_content_width)) / 2;

            for (line_idx, line) in visible_lines.enumerate() {
                let visible_part = line
                    .chars()
                    .skip(horizontal_offset)
                    .take(viewable_width.saturating_sub(3))
                    .collect::<String>();

                print_with_color_and_background_at(
                    bounds.top() + 1 + line_idx + vertical_padding,
                    bounds.left() + 2 + horizontal_padding,
                    fg_color,
                    bg_color,
                    &visible_part,
                    buffer,
                );
            }

            if *draw_border {
                // Draw bottom border
                draw_horizontal_line(
                    bounds.bottom(),
                    bounds.left(),
                    bounds.right(),
                    border_color,
                    bg_color,
                    buffer,
                );

                // Draw right border
                draw_vertical_line(
                    bounds.right(),
                    bounds.top() + 1,
                    bounds.bottom().saturating_sub(1),
                    border_color,
                    bg_color,
                    buffer,
                );
            }

            // Drawing the scroll nobs within the borders
            if max_content_height > viewable_height {
                let scrollbar_position = if viewable_height > 1 {
                    ((vertical_scroll as f64 / 100.0) * (viewable_height.saturating_sub(1)) as f64)
                        .floor() as usize
                } else {
                    0
                };
                print_with_color_and_background_at(
                    bounds.top() + 1 + scrollbar_position,
                    bounds.right(),
                    border_color,
                    bg_color,
                    V_SCROLL_CHAR,
                    buffer,
                );
            }

            if max_content_width > viewable_width {
                let scrollbar_position = if viewable_width > 2 {
                    ((horizontal_scroll as f64 / 100.0) * (viewable_width.saturating_sub(2)) as f64)
                        .floor() as usize
                } else {
                    0
                };
                print_with_color_and_background_at(
                    bounds.bottom(),
                    bounds.left() + 1 + scrollbar_position,
                    border_color,
                    bg_color,
                    H_SCROLL_CHAR,
                    buffer,
                );
            }

            scrollbars_drawn = true;
        } else if !_overflowing {
            // Calculate total height of the content block
            let total_lines = content_lines.len();
            let vertical_padding = (viewable_height.saturating_sub(total_lines)) / 2;
            let horizontal_padding = (viewable_width.saturating_sub(max_content_width)) / 2;

            // Iterate through the content lines and print them
            for (i, line) in content_lines.iter().enumerate().take(viewable_height) {
                let visible_line = &line
                    .chars()
                    .skip(_x_offset)
                    .take(viewable_width)
                    .collect::<String>();

                print_with_color_and_background_at(
                    bounds.top() + 2 + vertical_padding + i,
                    bounds.left() + 2 + horizontal_padding,
                    fg_color,
                    bg_color,
                    visible_line,
                    buffer,
                );
            }
        }
    }

    if _overflowing && overflow_behavior != "scroll" {
        if overflow_behavior == "fill" {
            fill_panel(&bounds, true, bg_color, '█', buffer);
        } else if overflow_behavior == "cross_out" {
            for i in 0..bounds.width() {
                let cell = Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: parent_bg_color_code.clone(),
                    ch: 'X',
                };
                buffer.update(bounds.left() + i, bounds.top() + i, cell);
            }
        } else if overflow_behavior == "removed" {
            fill_panel(&bounds, false, parent_bg_color, ' ', buffer);
        }
    } else {
        if *draw_border {
            // Draw bottom border
            if !scrollbars_drawn {
                draw_horizontal_line(
                    bounds.bottom(),
                    bounds.left(),
                    bounds.right(),
                    border_color,
                    bg_color,
                    buffer,
                );
            }

            // Draw left border
            draw_vertical_line(
                bounds.left(),
                bounds.top() + 1,
                bounds.bottom().saturating_sub(1),
                border_color,
                bg_color,
                buffer,
            );

            // Draw right border
            if !scrollbars_drawn {
                draw_vertical_line(
                    bounds.right(),
                    bounds.top() + 1,
                    bounds.bottom().saturating_sub(1),
                    border_color,
                    bg_color,
                    buffer,
                );
            }

            // Draw corners
            buffer.update(
                bounds.left(),
                bounds.top(),
                Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: bg_color_code.clone(),
                    ch: '┌',
                },
            );
            buffer.update(
                bounds.right(),
                bounds.top(),
                Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: bg_color_code.clone(),
                    ch: '┐',
                },
            );
            buffer.update(
                bounds.left(),
                bounds.bottom(),
                Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: bg_color_code.clone(),
                    ch: '└',
                },
            );
            buffer.update(
                bounds.right(),
                bounds.bottom(),
                Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: bg_color_code.clone(),
                    ch: '┘',
                },
            );
        }
    }
}

pub fn fill_panel(
    bounds: &Bounds,
    inside: bool,
    bg_color: &str,
    fill_char: char,
    buffer: &mut ScreenBuffer,
) {
    let fg_color_code = get_fg_color(bg_color);
    let bg_color_code = get_bg_color(bg_color);

    let (top, bottom) = if inside {
        (bounds.top(), bounds.bottom())
    } else {
        (bounds.top(), bounds.bottom())
    };

    let (left, right) = if inside {
        (bounds.left(), bounds.right())
    } else {
        (bounds.left(), bounds.right())
    };

    for y in top..bottom {
        for x in left..right {
            let cell = Cell {
                fg_color: fg_color_code.clone(),
                bg_color: bg_color_code.clone(),
                ch: fill_char,
            };
            buffer.update(x, y, cell);
        }
    }
}
