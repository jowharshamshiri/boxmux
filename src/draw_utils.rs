use crate::color_utils::{get_bg_color_transparent, get_fg_color_transparent, should_draw_color};
use crate::{set_terminal_title, AppContext, AppGraph, Bounds, Layout, MuxBox, ScreenBuffer};
use std::collections::HashMap;

use crate::ansi_color_processor::{contains_ansi_sequences, process_ansi_text};
use crate::components::{BoxRenderer, ComponentDimensions};
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
    let fill_char = cloned_layout.fill_char.unwrap_or(' ');

    // Set the background for the layout - None means transparent (no background drawing)
    fill_muxbox(
        &screen_bounds(),
        false,
        &cloned_layout.bg_color,
        &cloned_layout.fg_color,
        fill_char,
        buffer,
    );

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
    // Create BoxRenderer component and delegate rendering to it
    // This preserves ALL existing functionality while using the component system
    let mut box_renderer = BoxRenderer::new(muxbox, format!("muxbox_{}", muxbox.id));

    if box_renderer.render(app_context, app_graph, adjusted_bounds, layout, buffer) {
        // Draw children sorted by z_index (same logic as before)
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
}

pub fn print_with_color_and_background_at(
    y: usize,
    x: usize,
    fg_color: &Option<String>,
    bg_color: &Option<String>,
    text: &str,
    buffer: &mut ScreenBuffer,
) {
    // Check if text contains ANSI sequences
    if contains_ansi_sequences(text) {
        // Process ANSI sequences and render directly
        let cells = process_ansi_text(text);
        for (i, cell) in cells.iter().enumerate() {
            buffer.update(x + i, y, cell.clone());
        }
    } else {
        // Original behavior for plain text
        let fg_color_code = get_fg_color_transparent(fg_color);
        let bg_color_code = get_bg_color_transparent(bg_color);
        for (i, ch) in text.chars().enumerate() {
            let cell = Cell {
                fg_color: fg_color_code.clone(),
                bg_color: bg_color_code.clone(),
                ch,
            };
            buffer.update(x + i, y, cell);
        }
    }
}

/// Render ANSI text with embedded color codes at specific position
pub fn print_ansi_text_at(y: usize, x: usize, text: &str, buffer: &mut ScreenBuffer) {
    let cells = process_ansi_text(text);
    for (i, cell) in cells.iter().enumerate() {
        buffer.update(x + i, y, cell.clone());
    }
}

pub fn draw_horizontal_line(
    y: usize,
    x1: usize,
    x2: usize,
    border_color: &Option<String>,
    bg_color: &Option<String>,
    buffer: &mut ScreenBuffer,
) {
    let border_color_code = get_fg_color_transparent(border_color);
    let bg_color_code = get_bg_color_transparent(bg_color);
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
    fg_color: &Option<String>,
    bg_color: &Option<String>,
    buffer: &mut ScreenBuffer,
) {
    let fg_color_code = get_fg_color_transparent(fg_color);
    let bg_color_code = get_bg_color_transparent(bg_color);
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
    border_color: &Option<String>,
    bg_color: &Option<String>,
    buffer: &mut ScreenBuffer,
) {
    let border_color_code = get_fg_color_transparent(border_color);
    let bg_color_code = get_bg_color_transparent(bg_color);
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
    bg_color: &Option<String>,
    buffer: &mut ScreenBuffer,
) {
    let bg_color_code = get_bg_color_transparent(bg_color);
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
    fg_color: &Option<String>,
    bg_color: &Option<String>,
    _title: Option<&str>,
    title_fg_color: &Option<String>,
    title_bg_color: &Option<String>,
    title_position: &str,
    buffer: &mut ScreenBuffer,
) {
    let width = x2.saturating_sub(x1);
    let title_padding = 2; // Adjust padding if needed

    if let Some(title) = _title {
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
                title_start_position + title_length.saturating_sub(1),
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

            // Draw borders if border colors are not transparent
            if should_draw_color(fg_color) || should_draw_color(bg_color) {
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
        } else if should_draw_color(fg_color) || should_draw_color(bg_color) {
            // If the title is too long, just draw a line without the title
            draw_horizontal_line(y, x1, x2, fg_color, bg_color, buffer);
        }
    } else if should_draw_color(fg_color) || should_draw_color(bg_color) {
        // If there is no title, just draw a full horizontal line
        draw_horizontal_line(y, x1, x2, fg_color, bg_color, buffer);
    }
    // If there is no title and no border, do nothing
}

// Scrollbar constants moved to components module

// F0203: Multi-Stream Input Tabs - Tab rendering functions
pub fn draw_horizontal_line_with_tabs(
    y: usize,
    x1: usize,
    x2: usize,
    fg_color: &Option<String>,
    bg_color: &Option<String>,
    _title: Option<&str>,
    title_fg_color: &Option<String>,
    title_bg_color: &Option<String>,
    title_position: &str,
    tab_labels: &[String],
    tab_close_buttons: &[bool], // F0219: Close button info for each tab
    active_tab_index: usize,
    tab_scroll_offset: usize,
    buffer: &mut ScreenBuffer,
) {
    let _width = x2.saturating_sub(x1);

    // All boxes show tabs - render tab bar if tabs exist, otherwise empty bar
    if !tab_labels.is_empty() {
        crate::components::TabBar::draw(
            y,
            x1,
            x2,
            fg_color,
            bg_color,
            title_fg_color,
            title_bg_color,
            tab_labels,
            tab_close_buttons,
            active_tab_index,
            tab_scroll_offset,
            buffer,
        );
    } else {
        // No tabs initialized - fall back to empty border line
        draw_horizontal_line_with_title(
            y,
            x1,
            x2,
            fg_color,
            bg_color,
            None,
            title_fg_color,
            title_bg_color,
            title_position,
            buffer,
        );
    }
}

// Tab bar implementation moved to components::tab_bar module

pub fn calculate_tab_click_index(
    click_x: usize,
    x1: usize,
    x2: usize,
    tab_labels: &[String],
    tab_scroll_offset: usize,
    fg_color: &Option<String>,
    bg_color: &Option<String>,
) -> Option<usize> {
    crate::components::TabBar::calculate_tab_click_index(
        click_x,
        x1,
        x2,
        tab_labels,
        tab_scroll_offset,
        fg_color,
        bg_color,
    )
}

pub fn calculate_tab_navigation_click(
    click_x: usize,
    x1: usize,
    x2: usize,
    tab_labels: &[String],
    tab_scroll_offset: usize,
    fg_color: &Option<String>,
    bg_color: &Option<String>,
) -> Option<TabNavigationAction> {
    crate::components::TabBar::calculate_tab_navigation_click(
        click_x,
        x1,
        x2,
        tab_labels,
        tab_scroll_offset,
        fg_color,
        bg_color,
    )
}

/// F0219: Calculate if click was on a close button within a tab
pub fn calculate_tab_close_click(
    click_x: usize,
    x1: usize,
    x2: usize,
    tab_labels: &[String],
    tab_close_buttons: &[bool],
    tab_scroll_offset: usize,
    fg_color: &Option<String>,
    bg_color: &Option<String>,
) -> Option<usize> {
    crate::components::TabBar::calculate_tab_close_click(
        click_x,
        x1,
        x2,
        tab_labels,
        tab_close_buttons,
        tab_scroll_offset,
        fg_color,
        bg_color,
    )
}

// TabNavigationAction moved to components::tab_bar module
pub use crate::components::TabNavigationAction;

// render_muxbox function removed - replaced by BoxRenderer component
// All box rendering logic is now encapsulated in src/components/box_renderer.rs

// Old manual scrollbar function removed - now using unified components

// Old manual scrollbar function removed - now using unified components

pub fn fill_muxbox(
    bounds: &Bounds,
    _inside: bool,
    bg_color: &Option<String>,
    fg_color: &Option<String>,
    fill_char: char,
    buffer: &mut ScreenBuffer,
) {
    // Skip drawing entirely if both colors are transparent
    if !should_draw_color(bg_color) && !should_draw_color(fg_color) {
        return;
    }

    let fg_color_code = get_fg_color_transparent(fg_color);
    let bg_color_code = get_bg_color_transparent(bg_color);

    let (top, bottom) = (bounds.top(), bounds.bottom());
    let (left, right) = (bounds.left(), bounds.right());

    for y in top..bottom {
        for x in left..right {
            // Only update cells where we have colors to draw
            if should_draw_color(bg_color) || should_draw_color(fg_color) {
                let default_cell = Cell {
                    fg_color: get_fg_color_transparent(&None),
                    bg_color: get_bg_color_transparent(&None),
                    ch: ' ',
                };
                let existing_cell = buffer.get(x, y).unwrap_or(&default_cell);
                let cell = Cell {
                    fg_color: if should_draw_color(fg_color) {
                        fg_color_code.clone()
                    } else {
                        existing_cell.fg_color.clone()
                    },
                    bg_color: if should_draw_color(bg_color) {
                        bg_color_code.clone()
                    } else {
                        existing_cell.bg_color.clone()
                    },
                    ch: fill_char,
                };
                buffer.update(x, y, cell);
            }
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
pub fn render_wrapped_content(
    wrapped_lines: &[String],
    bounds: &Bounds,
    vertical_scroll: f64,
    fg_color: &Option<String>,
    bg_color: &Option<String>,
    buffer: &mut ScreenBuffer,
) {
    let component_dims = ComponentDimensions::new(*bounds);
    let content_bounds = component_dims.content_bounds(); 
    let viewable_height = content_bounds.height();
    let content_start_x = content_bounds.left() + 1;
    let content_start_y = content_bounds.top();

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

        // Use the updated function that handles ANSI sequences
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
    pub line_index: usize, // Which line of the wrapped choice this is (0-based)
    pub content: String,
    pub is_selected: bool,
    pub is_waiting: bool,
    pub is_hovered: bool,
}

/// Wrap choices to fit within specified width, handling multi-line choices
pub fn wrap_choices_to_width(
    choices: &[crate::model::muxbox::Choice],
    width: usize,
) -> Vec<WrappedChoice> {
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
                    is_hovered: choice.hovered,
                });
            }
        }
    }

    wrapped_choices
}

/// Render wrapped choices within bounds
pub fn render_wrapped_choices(
    choices: &[crate::model::muxbox::Choice],
    bounds: &Bounds,
    vertical_scroll: f64,
    fg_color: &Option<String>,
    bg_color: &Option<String>,
    selected_choice_fg_color: &Option<String>,
    selected_choice_bg_color: &Option<String>,
    highlighted_choice_fg_color: &Option<String>,
    highlighted_choice_bg_color: &Option<String>,
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

    let component_dims = ComponentDimensions::new(*bounds);
    let content_bounds = component_dims.content_bounds();
    let choice_start_x = content_bounds.left() + 1;
    let choice_start_y = content_bounds.top();

    for (display_idx, wrapped_choice) in visible_wrapped_choices.enumerate() {
        let render_y = choice_start_y + display_idx;
        if render_y >= bounds.bottom() {
            break;
        }

        let (choice_fg, choice_bg) = if wrapped_choice.is_selected {
            // Selected state takes highest priority
            (selected_choice_fg_color, selected_choice_bg_color)
        } else if wrapped_choice.is_hovered {
            // Hovered state takes priority over normal state
            (highlighted_choice_fg_color, highlighted_choice_bg_color)
        } else {
            // Normal state
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
