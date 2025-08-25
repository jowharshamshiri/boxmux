use crossterm::style::{Color, SetBackgroundColor, SetForegroundColor};

use crate::{
    set_terminal_title, AppContext, AppGraph, Bounds, Choice, Layout, MuxBox, ScreenBuffer,
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
    fill_muxbox(&screen_bounds(), false, &bg_color, fill_char, buffer);

    if let Some(layout_title) = &cloned_layout.title {
        if !layout_title.trim().is_empty() {
            set_terminal_title(layout_title);
        }
    }

    if let Some(children) = &cloned_layout.children {
        // Sort children by z_index (lower z_index first, higher z_index on top)
        let mut sorted_children: Vec<&MuxBox> = children.iter().collect();
        sorted_children.sort_by_key(|muxbox| muxbox.effective_z_index());
        
        for muxbox in sorted_children.iter() {
            draw_muxbox(
                app_context,
                app_graph,
                adjusted_bounds,
                layout,
                muxbox,
                buffer,
            );
        }
    }
}

pub fn draw_muxbox(
    app_context: &AppContext,
    app_graph: &AppGraph,
    adjusted_bounds: &HashMap<String, HashMap<String, Bounds>>,
    layout: &Layout,
    muxbox: &MuxBox,
    buffer: &mut ScreenBuffer,
) {
    let muxbox_parent = app_graph.get_parent(&layout.id, &muxbox.id);

    let layout_adjusted_bounds = adjusted_bounds.get(&layout.id);

    let mut muxbox_adjusted_bounds = None;
    match layout_adjusted_bounds {
        Some(value) => muxbox_adjusted_bounds = value.get(&muxbox.id),
        None => println!("Calculated bounds for layout {} not found", &layout.id),
    }

    match muxbox_adjusted_bounds {
        Some(value) => {
            let bg_color = muxbox.calc_bg_color(app_context, app_graph).to_string();
            let parent_bg_color = if muxbox_parent.is_none() {
                layout.bg_color.clone().unwrap_or("black".to_string())
            } else {
                muxbox_parent
                    .unwrap()
                    .calc_bg_color(app_context, app_graph)
                    .to_string()
            };
            let fg_color = muxbox.calc_fg_color(app_context, app_graph).to_string();

            let title_bg_color = muxbox
                .calc_title_bg_color(app_context, app_graph)
                .to_string();
            let title_fg_color = muxbox
                .calc_title_fg_color(app_context, app_graph)
                .to_string();
            let border = muxbox.calc_border(app_context, app_graph);
            // F0135: PTY Error States - Use different colors based on PTY status
            let border_color = if muxbox.pty.unwrap_or(false) {
                // Check for error states and use appropriate colors
                if let Some(pty_manager) = &app_context.pty_manager {
                    if pty_manager.is_pty_dead(&muxbox.id) {
                        "red".to_string() // Dead PTY processes get red borders
                    } else if pty_manager.is_pty_in_error_state(&muxbox.id) {
                        "yellow".to_string() // Error states get yellow borders
                    } else {
                        "bright_cyan".to_string() // Normal PTY muxboxes get bright cyan borders
                    }
                } else {
                    "bright_cyan".to_string() // Default PTY color if no manager
                }
            } else {
                muxbox.calc_border_color(app_context, app_graph).to_string()
            };
            let fill_char = muxbox.calc_fill_char(app_context, app_graph);

            // Draw fill
            fill_muxbox(value, border, &bg_color, fill_char, buffer);

            let mut content = muxbox.content.as_deref();
            let mut chart_content = None;
            let mut plugin_content = None;
            let mut table_content = None;

            // Generate plugin content if muxbox has plugin configuration
            if let Some(generated_plugin) = muxbox.generate_plugin_content(app_context, value) {
                plugin_content = Some(generated_plugin);
                content = plugin_content.as_deref();
            }

            // Generate chart content if muxbox has chart configuration (charts override plugins)
            if let Some(generated_chart) = muxbox.generate_chart_content(value) {
                chart_content = Some(generated_chart);
                content = chart_content.as_deref();
            }

            // Generate table content if muxbox has table configuration (tables override charts)
            if let Some(generated_table) = muxbox.generate_table_content(value) {
                table_content = Some(generated_table);
                content = table_content.as_deref();
            }

            // F0120: PTY Scrollback - Use scrollback content for PTY muxboxes
            let mut pty_scrollback_content = None;
            if muxbox.pty.unwrap_or(false) {
                if let Some(pty_manager) = &app_context.pty_manager {
                    if let Some(scrollback) = muxbox.get_scrollback_content(pty_manager) {
                        pty_scrollback_content = Some(scrollback);
                        content = pty_scrollback_content.as_deref();
                    }
                }
            }

            // check output is not null or empty - output overrides everything (including scrollback)
            if !muxbox.output.is_empty() {
                content = Some(&muxbox.output);
            }

            // Automatic scrollbar logic for focusable muxboxes
            let mut overflow_behavior = muxbox.calc_overflow_behavior(app_context, app_graph);

            // If muxbox is focusable (has next_focus_id) and has scrollable content, enable scrolling
            if muxbox.next_focus_id.is_some() && muxbox.has_scrollable_content() {
                overflow_behavior = "scroll".to_string();
            }

            // Add PTY indicator and process info to title if muxbox has PTY enabled
            let title_with_pty_indicator = if muxbox.pty.unwrap_or(false) {
                // F0135: PTY Error States - Use different indicators for error states
                let indicator = if let Some(pty_manager) = &app_context.pty_manager {
                    if pty_manager.is_pty_dead(&muxbox.id) {
                        "💀" // Skull for dead processes
                    } else if pty_manager.is_pty_in_error_state(&muxbox.id) {
                        "⚠️" // Warning for error states
                    } else {
                        "⚡" // Lightning bolt for normal PTY
                    }
                } else {
                    "⚡" // Default lightning bolt
                };

                let mut title_parts = vec![indicator.to_string()];

                // F0132: PTY Process Info - Add process info if available
                if let Some(pty_manager) = &app_context.pty_manager {
                    if let Some(status_summary) = pty_manager.get_process_status_summary(&muxbox.id)
                    {
                        title_parts.push(format!("[{}]", status_summary));
                    }
                }

                // Add original title if it exists
                if let Some(title) = muxbox.title.as_deref() {
                    title_parts.push(title.to_string());
                } else {
                    title_parts.push("PTY".to_string());
                }

                Some(title_parts.join(" "))
            } else {
                muxbox.title.clone()
            };

            render_muxbox(
                value,
                &border_color,
                &bg_color,
                &parent_bg_color,
                title_with_pty_indicator.as_deref(),
                &title_fg_color,
                &title_bg_color,
                &muxbox.calc_title_position(app_context, app_graph),
                muxbox.choices.clone(),
                &muxbox.calc_menu_fg_color(app_context, app_graph),
                &muxbox.calc_menu_bg_color(app_context, app_graph),
                &muxbox.calc_selected_menu_fg_color(app_context, app_graph),
                &muxbox.calc_selected_menu_bg_color(app_context, app_graph),
                content,
                &fg_color,
                &overflow_behavior,
                Some(&muxbox.calc_border(app_context, app_graph)),
                muxbox.current_horizontal_scroll(),
                muxbox.current_vertical_scroll(),
                app_context.config.locked, // Pass locked state from config
                buffer,
            );

            // Draw children sorted by z_index
            if let Some(children) = &muxbox.children {
                let mut sorted_children: Vec<&MuxBox> = children.iter().collect();
                sorted_children.sort_by_key(|child| child.effective_z_index());
                
                for child in sorted_children.iter() {
                    draw_muxbox(
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
        None => println!("Calculated bounds for muxbox {} not found", &muxbox.id),
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
    for (i, ch) in text.chars().enumerate() {
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

pub fn fill_horizontal_background(
    y: usize,
    x1: usize,
    x2: usize,
    fg_color: &str,
    bg_color: &str,
    buffer: &mut ScreenBuffer,
) {
    let fg_color_code = get_fg_color(fg_color);
    let bg_color_code = get_bg_color(bg_color);
    for x in x1..=x2 {
        let cell = Cell {
            fg_color: fg_color_code.clone(),
            bg_color: bg_color_code.clone(),
            ch: ' ',
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
            let (title_start_position, line_before_title_length, _line_after_title_length) =
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

            // First, fill the entire title area with title background
            fill_horizontal_background(
                y,
                title_start_position,
                title_start_position + title_length - 1,
                title_fg_color,
                title_bg_color,
                buffer,
            );

            // Then print the title text on top
            print_with_color_and_background_at(
                y,
                title_start_position,
                title_fg_color,
                title_bg_color,
                &formatted_title,
                buffer,
            );

            // Draw borders if needed
            if draw_border {
                draw_horizontal_line(
                    y,
                    x1,
                    x1 + line_before_title_length,
                    fg_color,
                    bg_color,
                    buffer,
                );
                draw_horizontal_line(
                    y,
                    x1 + line_before_title_length + title_length,
                    x2,
                    fg_color,
                    bg_color,
                    buffer,
                );
            }
        } else if draw_border {
            // If the title is too long, just draw a line without the title
            draw_horizontal_line(y, x1, x2, fg_color, bg_color, buffer);
        }
    } else if draw_border {
        // If there is no title, just draw a full horizontal line
        draw_horizontal_line(y, x1, x2, fg_color, bg_color, buffer);
    }
    // If there is no title and no border, do nothing
}

static H_SCROLL_CHAR: &str = "■";
static V_SCROLL_CHAR: &str = "█";
static H_SCROLL_TRACK: &str = "─";
static V_SCROLL_TRACK: &str = "│";

pub fn render_muxbox(
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
    locked: bool, // Whether muxboxes are locked (disable resize/move and hide corner knob)
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
    } else if *draw_border {
        draw_horizontal_line(
            bounds.top(),
            bounds.left(),
            bounds.right(),
            border_color,
            bg_color,
            buffer,
        );
    }

    if let Some(ref choices) = choices {
        let viewable_height = bounds.height().saturating_sub(2); // Account for borders
        let total_choices = choices.len();
        let choice_overflows = total_choices > viewable_height;
        
        // Calculate scroll offset for choices
        let vertical_offset = if choice_overflows {
            ((vertical_scroll / 100.0) * (total_choices - viewable_height) as f64).floor() as usize
        } else {
            0
        };
        
        // Respect overflow_behavior for choices
        if choice_overflows && overflow_behavior == "scroll" {
            // Render visible choices with scrolling
            let visible_choices = choices.iter().skip(vertical_offset).take(viewable_height);
            let mut y_position = bounds.top() + 1;
            
            for choice in visible_choices {
                let fg_color = if choice.selected { selected_menu_fg_color } else { menu_fg_color };
                let bg_color = if choice.selected { selected_menu_bg_color } else { menu_bg_color };

                let formatted_content = if choice.waiting {
                    format!("{}...", choice.content.as_ref().unwrap())
                } else {
                    choice.content.as_ref().unwrap().to_string()
                };

                print_with_color_and_background_at(
                    y_position,
                    bounds.left() + 2,
                    fg_color,
                    bg_color,
                    &formatted_content,
                    buffer,
                );
                y_position += 1;
            }
            
            // Draw vertical scrollbar for choices
            if *draw_border {
                draw_vertical_scrollbar(
                    &bounds,
                    total_choices,
                    viewable_height,
                    vertical_scroll,
                    border_color,
                    bg_color,
                    buffer,
                );
                scrollbars_drawn = true;
            }
        } else {
            // Handle other overflow behaviors (wrap, fill, cross_out, removed) and clipping
            let mut y_position = bounds.top() + 1;
            let viewable_width = bounds.width().saturating_sub(4);
            let viewable_height = bounds.height().saturating_sub(2);
            
            if overflow_behavior == "wrap" {
                // Create all wrapped lines first
                let mut all_wrapped_lines = Vec::new();
                for choice in choices {
                    let fg_color = if choice.selected { selected_menu_fg_color } else { menu_fg_color };
                    let bg_color = if choice.selected { selected_menu_bg_color } else { menu_bg_color };

                    let formatted_content = if choice.waiting {
                        format!("{}...", choice.content.as_ref().unwrap())
                    } else {
                        choice.content.as_ref().unwrap().to_string()
                    };

                    // Wrap the choice text
                    let wrapped_lines = wrap_text_to_width(&formatted_content, viewable_width);
                    
                    for wrapped_line in wrapped_lines {
                        all_wrapped_lines.push((wrapped_line, fg_color, bg_color));
                    }
                }
                
                // Calculate scroll offset for wrapped lines
                let total_wrapped_lines = all_wrapped_lines.len();
                let vertical_offset = if total_wrapped_lines > viewable_height {
                    ((vertical_scroll / 100.0) * (total_wrapped_lines - viewable_height) as f64).floor() as usize
                } else {
                    0
                };
                
                // Render visible wrapped lines with scroll offset
                let visible_lines = all_wrapped_lines.iter().skip(vertical_offset).take(viewable_height);
                for (wrapped_line, fg_color, bg_color) in visible_lines {
                    if y_position > bounds.bottom() - 1 {
                        break; // Don't draw outside the bounds
                    }
                    
                    print_with_color_and_background_at(
                        y_position,
                        bounds.left() + 2,
                        *fg_color,
                        *bg_color,
                        wrapped_line,
                        buffer,
                    );
                    y_position += 1;
                }
                
                // Check if wrapped choices overflow vertically and need scrollbar
                let total_wrapped_lines: usize = choices.iter().map(|choice| {
                    let formatted_content = if choice.waiting {
                        format!("{}...", choice.content.as_ref().unwrap())
                    } else {
                        choice.content.as_ref().unwrap().to_string()
                    };
                    wrap_text_to_width(&formatted_content, viewable_width).len()
                }).sum();
                
                if total_wrapped_lines > viewable_height && *draw_border {
                    draw_vertical_scrollbar(
                        &bounds,
                        total_wrapped_lines,
                        viewable_height,
                        vertical_scroll,
                        border_color,
                        bg_color,
                        buffer,
                    );
                    scrollbars_drawn = true;
                }
            } else {
                // Original choice rendering (simple clipping for other overflow behaviors)
                for choice in choices {
                    if y_position > bounds.bottom() - 1 {
                        break; // Don't draw outside the bounds
                    }

                    let fg_color = if choice.selected { selected_menu_fg_color } else { menu_fg_color };
                    let bg_color = if choice.selected { selected_menu_bg_color } else { menu_bg_color };

                    let formatted_content = if choice.waiting {
                        format!("{}...", choice.content.as_ref().unwrap())
                    } else {
                        choice.content.as_ref().unwrap().to_string()
                    };

                    // Apply overflow behavior to choice text
                    let processed_content = match overflow_behavior {
                        "fill" => {
                            let mut filled = formatted_content.clone();
                            while filled.len() < viewable_width {
                                filled.push(' ');
                            }
                            filled.truncate(viewable_width);
                            filled
                        },
                        "cross_out" => {
                            let mut crossed = String::new();
                            for ch in formatted_content.chars().take(viewable_width) {
                                if ch == ' ' {
                                    crossed.push(' ');
                                } else {
                                    crossed.push('X');
                                }
                            }
                            crossed
                        },
                        "removed" => {
                            continue; // Don't draw removed choices
                        },
                        _ => {
                            // Default clipping behavior
                            if formatted_content.len() > viewable_width {
                                formatted_content.chars().take(viewable_width).collect()
                            } else {
                                formatted_content
                            }
                        }
                    };

                    print_with_color_and_background_at(
                        y_position,
                        bounds.left() + 2,
                        fg_color,
                        bg_color,
                        &processed_content,
                        buffer,
                    );
                    y_position += 1;
                }
            }
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

            // Drawing scroll indicators with track and position
            if max_content_height > viewable_height {
                let track_height = bounds.bottom().saturating_sub(bounds.top() + 1); // Actual track space

                // Draw vertical scroll track
                for y in (bounds.top() + 1)..bounds.bottom() {
                    print_with_color_and_background_at(
                        y,
                        bounds.right(),
                        "bright_black",
                        bg_color,
                        V_SCROLL_TRACK,
                        buffer,
                    );
                }

                if track_height > 0 {
                    // Calculate proportional knob size and position
                    let content_ratio = viewable_height as f64 / max_content_height as f64;
                    let knob_size =
                        std::cmp::max(1, (track_height as f64 * content_ratio).round() as usize);
                    let available_track = track_height.saturating_sub(knob_size);

                    let knob_position = if available_track > 0 {
                        ((vertical_scroll / 100.0) * available_track as f64).round() as usize
                    } else {
                        0
                    };

                    // Draw proportional vertical scroll knob
                    for i in 0..knob_size {
                        let knob_y = bounds.top() + 1 + knob_position + i;
                        if knob_y < bounds.bottom() {
                            print_with_color_and_background_at(
                                knob_y,
                                bounds.right(),
                                border_color,
                                bg_color,
                                V_SCROLL_CHAR,
                                buffer,
                            );
                        }
                    }
                }
            }

            if max_content_width > viewable_width {
                let track_width = bounds.right().saturating_sub(bounds.left() + 1); // Actual track space

                // Draw horizontal scroll track
                for x in (bounds.left() + 1)..bounds.right() {
                    print_with_color_and_background_at(
                        bounds.bottom(),
                        x,
                        "bright_black",
                        bg_color,
                        H_SCROLL_TRACK,
                        buffer,
                    );
                }

                if track_width > 0 {
                    // Calculate proportional knob size and position
                    let content_ratio = viewable_width as f64 / max_content_width as f64;
                    let knob_size =
                        std::cmp::max(1, (track_width as f64 * content_ratio).round() as usize);
                    let available_track = track_width.saturating_sub(knob_size);

                    let knob_position = if available_track > 0 {
                        ((horizontal_scroll / 100.0) * available_track as f64).round() as usize
                    } else {
                        0
                    };

                    // Draw proportional horizontal scroll knob
                    for i in 0..knob_size {
                        let knob_x = bounds.left() + 1 + knob_position + i;
                        if knob_x < bounds.right() {
                            print_with_color_and_background_at(
                                bounds.bottom(),
                                knob_x,
                                border_color,
                                bg_color,
                                H_SCROLL_CHAR,
                                buffer,
                            );
                        }
                    }
                }
            }

            // Scroll position percentage indicator removed - visual scrollbars provide sufficient feedback

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

    // Handle special overflow behaviors that completely replace content
    if _overflowing && overflow_behavior != "scroll" && overflow_behavior != "wrap" {
        if overflow_behavior == "fill" {
            fill_muxbox(&bounds, true, bg_color, '█', buffer);
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
            fill_muxbox(&bounds, false, parent_bg_color, ' ', buffer);
        }
        return; // These behaviors completely replace the muxbox, no borders needed
    }
    
    // Handle text wrapping - render wrapped content but still draw borders and scrollbars
    if _overflowing && overflow_behavior == "wrap" {
        if let Some(content) = content {
            let viewable_width = bounds.width().saturating_sub(4);
            let wrapped_content = wrap_text_to_width(content, viewable_width);
            
            // Check if wrapped content still overflows vertically
            let viewable_height = bounds.height().saturating_sub(4);
            let wrapped_overflows_vertically = wrapped_content.len() > viewable_height;
            
            render_wrapped_content(
                &wrapped_content,
                &bounds,
                vertical_scroll,
                fg_color,
                bg_color,
                buffer,
            );
            
            // Draw vertical scrollbar if wrapped content overflows
            if wrapped_overflows_vertically && *draw_border {
                draw_vertical_scrollbar(
                    &bounds,
                    wrapped_content.len(),
                    viewable_height,
                    vertical_scroll,
                    border_color,
                    bg_color,
                    buffer,
                );
                scrollbars_drawn = true;
            }
        } else if let Some(ref choices) = choices {
            let viewable_width = bounds.width().saturating_sub(4);
            let wrapped_choices = wrap_choices_to_width(choices, viewable_width);
            
            // Check if wrapped choices overflow vertically
            let viewable_height = bounds.height().saturating_sub(4);
            let wrapped_overflows_vertically = wrapped_choices.len() > viewable_height;
            
            render_wrapped_choices(
                &choices,
                &bounds,
                vertical_scroll,
                fg_color,
                bg_color,
                selected_menu_fg_color,
                selected_menu_bg_color,
                buffer,
            );
            
            // Draw vertical scrollbar if wrapped choices overflow
            if wrapped_overflows_vertically && *draw_border {
                draw_vertical_scrollbar(
                    &bounds,
                    wrapped_choices.len(),
                    viewable_height,
                    vertical_scroll,
                    border_color,
                    bg_color,
                    buffer,
                );
                scrollbars_drawn = true;
            }
        }
    }

    // Draw borders for all cases (normal, scroll, wrap) - but not for special behaviors (fill, cross_out, removed)
    if *draw_border {
        // Draw bottom border - always draw this
        draw_horizontal_line(
            bounds.bottom(),
            bounds.left(),
            bounds.right(),
            border_color,
            bg_color,
            buffer,
        );

        // Draw left border - always draw this
        draw_vertical_line(
            bounds.left(),
            bounds.top() + 1,
            bounds.bottom().saturating_sub(1),
            border_color,
            bg_color,
            buffer,
        );

        // Draw right border - only skip if vertical scrollbars are drawn
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
        // Bottom-right corner: show resize knob when unlocked, regular corner when locked
        buffer.update(
            bounds.right(),
            bounds.bottom(),
            Cell {
                fg_color: border_color_code.clone(),
                bg_color: bg_color_code.clone(),
                ch: if locked { '┘' } else { '⋱' }, // Use diagonal dots for resize knob when unlocked
            },
        );
    }
}

// Helper function to draw vertical scrollbars - unified for choices and content
fn draw_vertical_scrollbar(
    bounds: &Bounds,
    content_height: usize,
    viewable_height: usize,
    vertical_scroll: f64,
    border_color: &str,
    bg_color: &str,
    buffer: &mut ScreenBuffer,
) {
    let track_height = bounds.bottom().saturating_sub(bounds.top() + 1);
    
    // Draw vertical scroll track
    for y in (bounds.top() + 1)..bounds.bottom() {
        print_with_color_and_background_at(
            y,
            bounds.right(),
            "bright_black",
            bg_color,
            V_SCROLL_TRACK,
            buffer,
        );
    }
    
    if track_height > 0 {
        // Calculate proportional knob size and position
        let content_ratio = viewable_height as f64 / content_height as f64;
        let knob_size = std::cmp::max(1, (track_height as f64 * content_ratio).round() as usize);
        let available_track = track_height.saturating_sub(knob_size);
        
        let knob_position = if available_track > 0 {
            ((vertical_scroll / 100.0) * available_track as f64).round() as usize
        } else {
            0
        };
        
        // Draw proportional vertical scroll knob
        for i in 0..knob_size {
            let knob_y = bounds.top() + 1 + knob_position + i;
            if knob_y < bounds.bottom() {
                print_with_color_and_background_at(
                    knob_y,
                    bounds.right(),
                    border_color,
                    bg_color,
                    V_SCROLL_CHAR,
                    buffer,
                );
            }
        }
    }
}

pub fn fill_muxbox(
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

/// Wrap text to fit within specified width, preserving word boundaries
pub fn wrap_text_to_width(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut wrapped_lines = Vec::new();
    
    for line in text.lines() {
        if line.len() <= width {
            wrapped_lines.push(line.to_string());
            continue;
        }

        // Split long line into multiple wrapped lines
        let mut current_line = String::new();
        let mut current_width = 0;

        for word in line.split_whitespace() {
            let word_len = word.len();
            
            // If word itself is longer than width, break it
            if word_len > width {
                // Finish current line if it has content
                if !current_line.is_empty() {
                    wrapped_lines.push(current_line.clone());
                    current_line.clear();
                    current_width = 0;
                }
                
                // Break the long word across multiple lines
                let mut remaining_word = word;
                while remaining_word.len() > width {
                    let (chunk, rest) = remaining_word.split_at(width);
                    wrapped_lines.push(chunk.to_string());
                    remaining_word = rest;
                }
                
                if !remaining_word.is_empty() {
                    current_line = remaining_word.to_string();
                    current_width = remaining_word.len();
                }
                continue;
            }

            // Check if adding this word would exceed width
            let space_needed = if current_line.is_empty() { 0 } else { 1 }; // Space before word
            if current_width + space_needed + word_len > width {
                // Start new line with this word
                if !current_line.is_empty() {
                    wrapped_lines.push(current_line.clone());
                }
                current_line = word.to_string();
                current_width = word_len;
            } else {
                // Add word to current line
                if !current_line.is_empty() {
                    current_line.push(' ');
                    current_width += 1;
                }
                current_line.push_str(word);
                current_width += word_len;
            }
        }

        // Add final line if it has content
        if !current_line.is_empty() {
            wrapped_lines.push(current_line);
        }
    }

    if wrapped_lines.is_empty() {
        wrapped_lines.push(String::new());
    }

    wrapped_lines
}

/// Render wrapped text content within bounds
fn render_wrapped_content(
    wrapped_lines: &[String],
    bounds: &Bounds,
    vertical_scroll: f64,
    fg_color: &str,
    bg_color: &str,
    buffer: &mut ScreenBuffer,
) {
    let viewable_height = bounds.height().saturating_sub(4);
    let content_start_x = bounds.left() + 2;
    let content_start_y = bounds.top() + 2;
    
    // Calculate scroll offset
    let max_vertical_offset = wrapped_lines.len().saturating_sub(viewable_height);
    let vertical_offset = ((vertical_scroll / 100.0) * max_vertical_offset as f64).floor() as usize;
    
    // Render visible lines
    let visible_lines = wrapped_lines
        .iter()
        .skip(vertical_offset)
        .take(viewable_height);

    for (i, line) in visible_lines.enumerate() {
        let render_y = content_start_y + i;
        if render_y >= bounds.bottom() {
            break;
        }
        
        print_with_color_and_background_at(
            render_y,
            content_start_x,
            fg_color,
            bg_color,
            line,
            buffer,
        );
    }
}

/// Wrap choice content and store wrapped lines with original choice index
#[derive(Clone)]
pub struct WrappedChoice {
    pub original_index: usize,
    pub line_index: usize,  // Which line of the wrapped choice this is (0-based)
    pub content: String,
    pub is_selected: bool,
    pub is_waiting: bool,
}

/// Wrap choices to fit within specified width, handling multi-line choices
pub fn wrap_choices_to_width(choices: &[crate::model::muxbox::Choice], width: usize) -> Vec<WrappedChoice> {
    let mut wrapped_choices = Vec::new();
    
    for (choice_idx, choice) in choices.iter().enumerate() {
        if let Some(content) = &choice.content {
            let formatted_content = if choice.waiting {
                format!("{}...", content)
            } else {
                content.clone()
            };
            
            let wrapped_lines = wrap_text_to_width(&formatted_content, width);
            
            for (line_idx, wrapped_line) in wrapped_lines.iter().enumerate() {
                wrapped_choices.push(WrappedChoice {
                    original_index: choice_idx,
                    line_index: line_idx,
                    content: wrapped_line.clone(),
                    is_selected: choice.selected,
                    is_waiting: choice.waiting,
                });
            }
        }
    }
    
    wrapped_choices
}

/// Render wrapped choices within bounds
fn render_wrapped_choices(
    choices: &[crate::model::muxbox::Choice],
    bounds: &Bounds,
    vertical_scroll: f64,
    fg_color: &str,
    bg_color: &str,
    selected_choice_fg_color: &str,
    selected_choice_bg_color: &str,
    buffer: &mut ScreenBuffer,
) {
    let viewable_width = bounds.width().saturating_sub(4);
    let viewable_height = bounds.height().saturating_sub(4);
    let wrapped_choices = wrap_choices_to_width(choices, viewable_width);
    
    if wrapped_choices.is_empty() {
        return;
    }
    
    // Calculate scroll offset
    let max_vertical_offset = wrapped_choices.len().saturating_sub(viewable_height);
    let vertical_offset = ((vertical_scroll / 100.0) * max_vertical_offset as f64).floor() as usize;
    
    // Render visible wrapped choice lines
    let visible_wrapped_choices = wrapped_choices
        .iter()
        .skip(vertical_offset)
        .take(viewable_height);

    let choice_start_x = bounds.left() + 2;
    let choice_start_y = bounds.top() + 1;

    for (display_idx, wrapped_choice) in visible_wrapped_choices.enumerate() {
        let render_y = choice_start_y + display_idx;
        if render_y >= bounds.bottom() {
            break;
        }

        let (choice_fg, choice_bg) = if wrapped_choice.is_selected {
            (selected_choice_fg_color, selected_choice_bg_color)
        } else {
            (fg_color, bg_color)
        };

        print_with_color_and_background_at(
            render_y,
            choice_start_x,
            choice_fg,
            choice_bg,
            &wrapped_choice.content,
            buffer,
        );
    }
}
