use crate::components::{
    VerticalScrollbar, HorizontalScrollbar, ChoiceRenderer, OverflowRenderer, OverflowConfig
};
use crate::draw_utils::{
    get_fg_color, get_bg_color, fill_muxbox, draw_horizontal_line, draw_vertical_line,
    print_with_color_and_background_at, content_size, draw_horizontal_line_with_tabs
};
use crate::model::common::{Cell, ContentStreamTrait, ChoicesStreamTrait, StreamType};
use crate::{AppContext, AppGraph, MuxBox, ScreenBuffer, Bounds};
use std::collections::HashMap;

/// BoxRenderer - Visual rendering component for MuxBox
/// 
/// This component consolidates all box drawing logic that was scattered across
/// draw_utils.rs and draw_loop.rs into a unified rendering orchestrator.
/// 
/// **IMPORTANT**: This is a VISUAL COMPONENT ONLY - it does not replace the
/// logical MuxBox struct. It queries the MuxBox for state and renders accordingly.
pub struct BoxRenderer<'a> {
    /// Reference to the logical MuxBox this renderer represents
    muxbox: &'a MuxBox,
    /// Component ID for this renderer instance
    component_id: String,
}

impl<'a> BoxRenderer<'a> {
    /// Create a new BoxRenderer for the given MuxBox
    pub fn new(muxbox: &'a MuxBox, component_id: String) -> Self {
        Self {
            muxbox,
            component_id,
        }
    }

    /// Main rendering function that orchestrates all box drawing
    /// 
    /// This replaces the logic that was in draw_muxbox() and render_muxbox()
    /// but preserves ALL existing functionality and behavior.
    pub fn render(
        &self,
        app_context: &AppContext,
        app_graph: &AppGraph,
        adjusted_bounds: &HashMap<String, HashMap<String, Bounds>>,
        layout: &crate::Layout,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        // Get bounds for this muxbox (same logic as draw_muxbox)
        let layout_adjusted_bounds = adjusted_bounds.get(&layout.id);
        let muxbox_adjusted_bounds = layout_adjusted_bounds.and_then(|bounds| bounds.get(&self.muxbox.id));

        let Some(bounds) = muxbox_adjusted_bounds else {
            log::error!("Calculated bounds for muxbox {} not found", &self.muxbox.id);
            return false;
        };

        // Calculate all colors and properties (same logic as draw_muxbox)
        let muxbox_parent = app_graph.get_parent(&layout.id, &self.muxbox.id);
        let bg_color = self.muxbox.calc_bg_color(app_context, app_graph).to_string();
        let parent_bg_color = if muxbox_parent.is_none() {
            layout.bg_color.clone().unwrap_or("black".to_string())
        } else {
            muxbox_parent.unwrap().calc_bg_color(app_context, app_graph).to_string()
        };
        let fg_color = self.muxbox.calc_fg_color(app_context, app_graph).to_string();
        let title_bg_color = self.muxbox.calc_title_bg_color(app_context, app_graph).to_string();
        let title_fg_color = self.muxbox.calc_title_fg_color(app_context, app_graph).to_string();
        let border = self.muxbox.calc_border(app_context, app_graph);

        // F0135: PTY Error States - Use different colors based on PTY status
        let border_color = if self.muxbox.execution_mode.is_pty() {
            if let Some(pty_manager) = &app_context.pty_manager {
                if pty_manager.is_pty_dead(&self.muxbox.id) {
                    "red".to_string()
                } else if pty_manager.is_pty_in_error_state(&self.muxbox.id) {
                    "yellow".to_string()
                } else {
                    "bright_cyan".to_string()
                }
            } else {
                "bright_cyan".to_string()
            }
        } else {
            self.muxbox.calc_border_color(app_context, app_graph).to_string()
        };

        let fill_char = self.muxbox.calc_fill_char(app_context, app_graph);

        // Draw fill (same logic as draw_muxbox)
        fill_muxbox(bounds, border, &bg_color, fill_char, buffer);

        // Calculate overflow behavior (same logic as draw_muxbox)
        let mut overflow_behavior = self.muxbox.calc_overflow_behavior(app_context, app_graph);
        if self.muxbox.next_focus_id.is_some() && self.muxbox.has_scrollable_content() {
            overflow_behavior = "scroll".to_string();
        }

        // Generate tab labels and close buttons (same logic as draw_muxbox)
        let tab_labels = self.muxbox.get_tab_labels();
        let tab_close_buttons = self.muxbox.get_tab_close_buttons();

        // Call the main rendering function with all calculated parameters
        self.render_box_contents(
            bounds,
            &border_color,
            &bg_color,
            &parent_bg_color,
            &self.muxbox.streams,
            self.muxbox.get_active_tab_index(),
            self.muxbox.tab_scroll_offset,
            &title_fg_color,
            &title_bg_color,
            &self.muxbox.calc_title_position(app_context, app_graph),
            &self.muxbox.calc_menu_fg_color(app_context, app_graph),
            &self.muxbox.calc_menu_bg_color(app_context, app_graph),
            &self.muxbox.calc_selected_menu_fg_color(app_context, app_graph),
            &self.muxbox.calc_selected_menu_bg_color(app_context, app_graph),
            &fg_color,
            &overflow_behavior,
            Some(&self.muxbox.calc_border(app_context, app_graph)),
            self.muxbox.current_horizontal_scroll(),
            self.muxbox.current_vertical_scroll(),
            app_context.config.locked,
            &tab_labels,
            &tab_close_buttons,
            buffer,
        );

        true
    }

    /// Internal function that contains all the rendering logic from render_muxbox()
    /// 
    /// This is essentially the render_muxbox() function but as a method of BoxRenderer
    /// Preserves ALL existing functionality and behavior.
    fn render_box_contents(
        &self,
        bounds: &Bounds,
        border_color: &str,
        bg_color: &str,
        parent_bg_color: &str,
        streams: &indexmap::IndexMap<String, crate::model::common::Stream>,
        active_tab_index: usize,
        tab_scroll_offset: usize,
        title_fg_color: &str,
        title_bg_color: &str,
        title_position: &str,
        menu_fg_color: &str,
        menu_bg_color: &str,
        selected_menu_fg_color: &str,
        selected_menu_bg_color: &str,
        fg_color: &str,
        overflow_behavior: &str,
        border: Option<&bool>,
        horizontal_scroll: f64,
        vertical_scroll: f64,
        locked: bool,
        tab_labels: &[String],
        tab_close_buttons: &[bool],
        buffer: &mut ScreenBuffer,
    ) {
        // EXACT copy of render_muxbox() logic - preserves ALL functionality

        // F0217: Extract content from streams using trait-based approach
        let (should_render_choices, content_str) = if !streams.is_empty() {
            let active_stream = streams.values().find(|s| s.active);
            let should_render_choices = if let Some(stream) = active_stream {
                matches!(stream.stream_type, StreamType::Choices)
            } else {
                false
            };

            let content_str = if should_render_choices {
                None
            } else if let Some(stream) = active_stream {
                match stream.stream_type {
                    StreamType::Content
                    | StreamType::RedirectedOutput(_)
                    | StreamType::PTY
                    | StreamType::Plugin(_)
                    | StreamType::ChoiceExecution(_)
                    | StreamType::PtySession(_)
                    | StreamType::OwnScript => {
                        Some(stream.get_content_lines().join("\n"))
                    }
                    _ => None,
                }
            } else {
                None
            };

            (should_render_choices, content_str)
        } else {
            (false, None)
        };

        let content = content_str.as_deref();

        // Extract choices from streams for legacy rendering logic compatibility
        let choices = if should_render_choices {
            streams
                .values()
                .find(|s| matches!(s.stream_type, StreamType::Choices))
                .map(|stream| stream.get_choices().clone())
        } else {
            None
        };

        let draw_border = border.unwrap_or(&true);
        let mut _overflowing = false;
        let mut scrollbars_drawn = false;

        // Ensure bounds stay within screen limits
        let screen_bounds = crate::utils::screen_bounds();
        let bounds = bounds.intersection(&screen_bounds).unwrap_or_else(|| bounds.clone());

        // F0208: Draw top border with tabs
        if !tab_labels.is_empty() {
            draw_horizontal_line_with_tabs(
                bounds.top(),
                bounds.left(),
                bounds.right(),
                border_color,
                bg_color,
                None,
                title_fg_color,
                title_bg_color,
                title_position,
                *draw_border,
                tab_labels,
                tab_close_buttons,
                active_tab_index,
                tab_scroll_offset,
                buffer,
            );
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

        // F0206: Render choices from streams if active stream is choices
        if should_render_choices {
            let choices_stream = streams
                .values()
                .find(|s| matches!(s.stream_type, StreamType::Choices));
            if let Some(stream) = choices_stream {
                let choices = stream.get_choices();
                if !choices.is_empty() {
                    let viewable_height = bounds.height().saturating_sub(2);
                    let viewable_width = bounds.width().saturating_sub(4);
                    
                    let choice_renderer = ChoiceRenderer::new("choices".to_string());
                    let component_scrollbars_drawn = choice_renderer.draw(
                        &bounds,
                        &choices,
                        viewable_height,
                        viewable_width,
                        vertical_scroll,
                        overflow_behavior,
                        menu_fg_color,
                        menu_bg_color,
                        selected_menu_fg_color,
                        selected_menu_bg_color,
                        border_color,
                        *draw_border,
                        buffer,
                    );
                    
                    if component_scrollbars_drawn {
                        scrollbars_drawn = true;
                    }
                }
            }
        } else if let Some(content) = content {
            let (content_width, content_height) = content_size(content);
            let viewable_width = bounds.width().saturating_sub(4);
            let viewable_height = bounds.height().saturating_sub(4);
            _overflowing = content_width > viewable_width || content_height > viewable_height;
            
            scrollbars_drawn = self.render_content(
                &bounds,
                content,
                fg_color,
                bg_color,
                border_color,
                parent_bg_color,
                overflow_behavior,
                horizontal_scroll,
                vertical_scroll,
                *draw_border,
                buffer,
            );
        }

        // Handle special overflow behaviors using OverflowRenderer component
        if _overflowing && overflow_behavior != "scroll" && overflow_behavior != "wrap" {
            let overflow_config = match overflow_behavior {
                "fill" => OverflowConfig::fill('█'),
                "cross_out" => OverflowConfig::cross_out(),
                "removed" => OverflowConfig::removed(),
                _ => OverflowConfig::default(),
            };
            
            let overflow_renderer = OverflowRenderer::new(
                format!("muxbox_special_overflow"), 
                overflow_config
            );

            if let Some(content) = content {
                if overflow_renderer.render_text_overflow(
                    content,
                    &bounds,
                    vertical_scroll,
                    0.0,
                    fg_color,
                    bg_color,
                    border_color,
                    parent_bg_color,
                    buffer,
                ) {
                    return;
                }
            } else if let Some(ref choices) = choices {
                if overflow_renderer.render_choice_overflow(
                    choices,
                    &bounds,
                    vertical_scroll,
                    menu_fg_color,
                    menu_bg_color,
                    selected_menu_fg_color,
                    selected_menu_bg_color,
                    border_color,
                    parent_bg_color,
                    buffer,
                ) {
                    return;
                }
            }
        }

        self.render_borders(&bounds, border_color, bg_color, *draw_border, scrollbars_drawn, locked, content, buffer);
    }

    /// Render content with scrolling and overflow handling
    fn render_content(
        &self,
        bounds: &Bounds,
        content: &str,
        fg_color: &str,
        bg_color: &str,
        border_color: &str,
        parent_bg_color: &str,
        overflow_behavior: &str,
        horizontal_scroll: f64,
        vertical_scroll: f64,
        draw_border: bool,
        buffer: &mut ScreenBuffer,
    ) -> bool {
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

        let _overflowing = content_width > viewable_width || content_height > viewable_height;

        if _overflowing && overflow_behavior == "scroll" {
            self.render_scrollable_content(
                bounds,
                &content_lines,
                max_content_width,
                max_content_height,
                viewable_width,
                viewable_height,
                horizontal_scroll,
                vertical_scroll,
                fg_color,
                bg_color,
                border_color,
                draw_border,
                buffer,
            );
            true
        } else if _overflowing && overflow_behavior == "wrap" {
            self.render_wrapped_content(
                bounds,
                content,
                vertical_scroll,
                fg_color,
                bg_color,
                border_color,
                parent_bg_color,
                buffer,
            );
            false
        } else {
            self.render_normal_content(
                bounds,
                &content_lines,
                max_content_width,
                viewable_width,
                viewable_height,
                fg_color,
                bg_color,
                buffer,
            );
            false
        }
    }

    /// Render scrollable content with scrollbars
    fn render_scrollable_content(
        &self,
        bounds: &Bounds,
        content_lines: &[&str],
        max_content_width: usize,
        max_content_height: usize,
        viewable_width: usize,
        viewable_height: usize,
        horizontal_scroll: f64,
        vertical_scroll: f64,
        fg_color: &str,
        bg_color: &str,
        border_color: &str,
        draw_border: bool,
        buffer: &mut ScreenBuffer,
    ) {
        let viewable_height = bounds.height().saturating_sub(1);
        
        let max_horizontal_offset = max_content_width.saturating_sub(viewable_width) + 3;
        let max_vertical_offset = max_content_height.saturating_sub(viewable_height);

        let horizontal_offset = ((horizontal_scroll / 100.0) * max_horizontal_offset as f64).floor() as usize;
        let vertical_offset = ((vertical_scroll / 100.0) * max_vertical_offset as f64).floor() as usize;

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

        if draw_border {
            draw_horizontal_line(
                bounds.bottom(),
                bounds.left(),
                bounds.right(),
                border_color,
                bg_color,
                buffer,
            );

            draw_vertical_line(
                bounds.right(),
                bounds.top() + 1,
                bounds.bottom().saturating_sub(1),
                border_color,
                bg_color,
                buffer,
            );
        }

        // Draw scrollbars using components
        let vertical_scrollbar = VerticalScrollbar::new("content".to_string());
        vertical_scrollbar.draw(
            bounds,
            max_content_height,
            viewable_height,
            vertical_scroll,
            border_color,
            bg_color,
            buffer,
        );

        let horizontal_scrollbar = HorizontalScrollbar::new("content".to_string());
        horizontal_scrollbar.draw(
            bounds,
            max_content_width,
            viewable_width,
            horizontal_scroll,
            border_color,
            bg_color,
            buffer,
        );
    }

    /// Render wrapped content using OverflowRenderer
    fn render_wrapped_content(
        &self,
        bounds: &Bounds,
        content: &str,
        vertical_scroll: f64,
        fg_color: &str,
        bg_color: &str,
        border_color: &str,
        parent_bg_color: &str,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let overflow_config = OverflowConfig::wrap();
        let overflow_renderer = OverflowRenderer::new(
            format!("muxbox_wrap_overflow"), 
            overflow_config
        );

        overflow_renderer.render_text_overflow(
            content,
            bounds,
            vertical_scroll,
            0.0,
            fg_color,
            bg_color,
            border_color,
            parent_bg_color,
            buffer,
        )
    }

    /// Render normal (non-overflowing) content
    fn render_normal_content(
        &self,
        bounds: &Bounds,
        content_lines: &[&str],
        max_content_width: usize,
        viewable_width: usize,
        viewable_height: usize,
        fg_color: &str,
        bg_color: &str,
        buffer: &mut ScreenBuffer,
    ) {
        let total_lines = content_lines.len();
        let vertical_padding = (viewable_height.saturating_sub(total_lines)) / 2;
        let horizontal_padding = (viewable_width.saturating_sub(max_content_width)) / 2;

        for (i, line) in content_lines.iter().enumerate().take(viewable_height) {
            let visible_line = &line
                .chars()
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

    /// Render borders with proper corner handling
    fn render_borders(
        &self,
        bounds: &Bounds,
        border_color: &str,
        bg_color: &str,
        draw_border: bool,
        scrollbars_drawn: bool,
        locked: bool,
        content: Option<&str>,
        buffer: &mut ScreenBuffer,
    ) {
        if !draw_border {
            return;
        }

        let border_color_code = get_fg_color(border_color);
        let bg_color_code = get_bg_color(bg_color);

        // Check if we need horizontal scrollbar
        let has_horizontal_scrollbar = content.is_some() && {
            if let Some(content_str) = content {
                let content_lines: Vec<&str> = content_str.lines().collect();
                let max_content_width = content_lines
                    .iter()
                    .map(|line| line.len())
                    .max()
                    .unwrap_or(0);
                let viewable_width = bounds.width().saturating_sub(4);
                max_content_width > viewable_width
            } else {
                false
            }
        };

        // Draw bottom border
        if has_horizontal_scrollbar {
            // Skip middle section for horizontal scrollbar
            draw_horizontal_line(
                bounds.bottom(),
                bounds.left(),
                bounds.left() + 1,
                border_color,
                bg_color,
                buffer,
            );
            draw_horizontal_line(
                bounds.bottom(),
                bounds.right().saturating_sub(1),
                bounds.right(),
                border_color,
                bg_color,
                buffer,
            );
        } else {
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

        // Draw right border - skip if vertical scrollbars are drawn
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

        // Bottom-right corner: show resize knob when unlocked
        buffer.update(
            bounds.right(),
            bounds.bottom(),
            Cell {
                fg_color: border_color_code.clone(),
                bg_color: bg_color_code.clone(),
                ch: if locked { '┘' } else { '⋱' },
            },
        );
    }
}