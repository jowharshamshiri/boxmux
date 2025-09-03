use crate::components::{
    VerticalScrollbar, HorizontalScrollbar,
    ChartComponent, ChartConfig, ChartType
};
use crate::components::choice_menu::ChoiceMenu;
use crate::components::renderable_content::RenderableContent;
use crate::color_utils::{
    get_fg_color_transparent, get_bg_color_transparent, 
    should_draw_color
};
use crate::draw_utils::{
    fill_muxbox, draw_horizontal_line, draw_vertical_line,
    print_with_color_and_background_at, content_size, draw_horizontal_line_with_tabs,
    wrap_text_to_width, render_wrapped_content
};
use crate::model::common::{Cell, ContentStreamTrait, ChoicesStreamTrait, StreamType};
use crate::{AppContext, AppGraph, MuxBox, ScreenBuffer, Bounds};
use crate::components::renderable_content::ClickableZone;
use std::collections::HashMap;

/// Overflow behavior types for unified content rendering
#[derive(Debug, Clone, PartialEq)]
pub enum UnifiedOverflowBehavior {
    /// Standard scrolling with scrollbars
    Scroll,
    /// Text wrapping with line breaks
    Wrap,
    /// Fill entire box with solid pattern
    Fill(char),
    /// Cross out content with X pattern
    CrossOut,
    /// Remove/hide content completely
    Removed,
    /// Clip content without scrollbars (default)
    Clip,
}

impl UnifiedOverflowBehavior {
    /// Parse overflow behavior from string
    pub fn from_str(behavior: &str) -> Self {
        match behavior {
            "scroll" => Self::Scroll,
            "wrap" => Self::Wrap,
            "fill" => Self::Fill('█'),
            "cross_out" => Self::CrossOut,
            "removed" => Self::Removed,
            _ => Self::Clip,
        }
    }
}

/// BoxRenderer - Unified rendering component with integrated overflow handling
/// 
/// This component consolidates ALL overflow logic from OverflowRenderer and provides
/// unified content rendering for text, choices, and charts using existing scrollbar components.
/// 
/// **IMPORTANT**: This is a VISUAL COMPONENT ONLY - it does not replace the
/// logical MuxBox struct. It queries the MuxBox for state and renders accordingly.
pub struct BoxRenderer<'a> {
    /// Reference to the logical MuxBox this renderer represents
    muxbox: &'a MuxBox,
    /// Component ID for this renderer instance
    component_id: String,
    /// Translated clickable zones in absolute screen coordinates
    clickable_zones: Vec<ClickableZone>,
}

impl<'a> BoxRenderer<'a> {
    /// Create a new BoxRenderer for the given MuxBox
    pub fn new(muxbox: &'a MuxBox, component_id: String) -> Self {
        Self {
            muxbox,
            component_id,
            clickable_zones: Vec::new(),
        }
    }
    
    /// Generate chart content if the muxbox has chart configuration
    /// This moves chart rendering responsibility from MuxBox to BoxRenderer
    fn generate_chart_content(&self, bounds: &Bounds) -> Option<String> {
        if let (Some(chart_type_str), Some(chart_data)) = (&self.muxbox.chart_type, &self.muxbox.chart_data) {
            let data = ChartComponent::parse_chart_data(chart_data);
            
            // Parse chart type with support for all variants including pie and scatter
            let chart_type = chart_type_str.parse::<ChartType>().unwrap_or(ChartType::Bar);
            
            let config = ChartConfig {
                chart_type,
                width: bounds.width().saturating_sub(4),
                height: bounds.height().saturating_sub(4),
                title: None, // Don't show chart title since muxbox already has the title
                color: "blue".to_string(),
                show_title: false, // Muxbox already shows title
                show_values: true,
                show_grid: false,
            };
            
            let chart = ChartComponent::with_data_and_config(
                format!("muxbox_chart_{}", self.muxbox.id),
                data,
                config
            );
            
            // Generate with muxbox title context to avoid duplication
            Some(chart.generate_with_muxbox_title(self.muxbox.title.as_deref()))
        } else {
            None
        }
    }

    /// Main rendering function that orchestrates all box drawing
    /// 
    /// This replaces the logic that was in draw_muxbox() and render_muxbox()
    /// but preserves ALL existing functionality and behavior.
    pub fn render(
        &mut self,
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
        let bg_color = self.muxbox.calc_bg_color(app_context, app_graph);
        let parent_bg_color = if muxbox_parent.is_none() {
            layout.bg_color.clone().map(|s| s.to_string())
        } else {
            muxbox_parent.unwrap().calc_bg_color(app_context, app_graph).map(|s| s.to_string())
        };
        let fg_color = self.muxbox.calc_fg_color(app_context, app_graph);
        let title_bg_color = self.muxbox.calc_title_bg_color(app_context, app_graph);
        let title_fg_color = self.muxbox.calc_title_fg_color(app_context, app_graph);
        let border = self.muxbox.calc_border(app_context, app_graph);

        // F0135: PTY Error States - Use different colors based on PTY status
        let border_color = if self.muxbox.execution_mode.is_pty() {
            if let Some(pty_manager) = &app_context.pty_manager {
                if pty_manager.is_pty_dead(&self.muxbox.id) {
                    Some("red".to_string())
                } else if pty_manager.is_pty_in_error_state(&self.muxbox.id) {
                    Some("yellow".to_string())
                } else {
                    Some("bright_cyan".to_string())
                }
            } else {
                Some("bright_cyan".to_string())
            }
        } else {
            self.muxbox.calc_border_color(app_context, app_graph).map(|s| s.to_string())
        };

        let fill_char = self.muxbox.calc_fill_char(app_context, app_graph);

        // Draw fill (same logic as draw_muxbox)
        fill_muxbox(bounds, border, &bg_color, &None, fill_char, buffer);

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
        &mut self,
        bounds: &Bounds,
        border_color: &Option<String>,
        bg_color: &Option<String>,
        parent_bg_color: &Option<String>,
        streams: &indexmap::IndexMap<String, crate::model::common::Stream>,
        active_tab_index: usize,
        tab_scroll_offset: usize,
        title_fg_color: &Option<String>,
        title_bg_color: &Option<String>,
        title_position: &str,
        menu_fg_color: &Option<String>,
        menu_bg_color: &Option<String>,
        selected_menu_fg_color: &Option<String>,
        selected_menu_bg_color: &Option<String>,
        fg_color: &Option<String>,
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

        // Check for chart content first - charts take priority over streams
        let chart_content = self.generate_chart_content(bounds);
        
        // F0217: Extract content from streams using trait-based approach
        let (should_render_choices, content_str) = if chart_content.is_some() {
            // Chart content overrides stream content
            (false, chart_content)
        } else if !streams.is_empty() {
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

        // Border visibility now determined by transparent color support
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
                tab_labels,
                tab_close_buttons,
                active_tab_index,
                tab_scroll_offset,
                buffer,
            );
        } else if should_draw_color(border_color) || should_draw_color(bg_color) {
            draw_horizontal_line(
                bounds.top(),
                bounds.left(),
                bounds.right(),
                &border_color,
                bg_color,
                buffer,
            );
        }

        // F0206: Render choices from streams as content using ChoiceMenu component
        if should_render_choices {
            let choices_stream = streams
                .values()
                .find(|s| matches!(s.stream_type, StreamType::Choices));
            if let Some(stream) = choices_stream {
                let choices = stream.get_choices();
                if !choices.is_empty() {
                    // Create ChoiceMenu component to generate content and clickable zones
                    let choice_menu = ChoiceMenu::new(
                        format!("{}_choice_menu", self.component_id),
                        &choices
                    )
                    .with_selection(self.muxbox.selected_choice_index())
                    .with_focus(self.muxbox.focused_choice_index());
                    
                    log::info!("CHOICE RENDER: BoxRenderer creating ChoiceMenu for muxbox '{}' with {} choices, dimensions: {:?}", 
                             self.muxbox.id, choices.len(), choice_menu.get_dimensions());
                    
                    // Get choice content as string - treat it like any other content
                    let choice_content = choice_menu.get_raw_content();
                    log::info!("CHOICE RENDER: Generated choice content length: {} chars", choice_content.len());
                    
                    // Use unified content rendering (same path as text content)
                    let (content_width, content_height) = content_size(&choice_content);
                    let viewable_width = bounds.width().saturating_sub(4);
                    let viewable_height = bounds.height().saturating_sub(4);
                    let choices_overflow = content_width > viewable_width || content_height > viewable_height;
                    
                    scrollbars_drawn = self.render_content(
                        &bounds,
                        &choice_content,
                        menu_fg_color,
                        menu_bg_color,
                        border_color,
                        parent_bg_color,
                        overflow_behavior,
                        horizontal_scroll,
                        vertical_scroll,
                        buffer,
                    );
                    
                    // Get box-relative clickable zones and translate to absolute coordinates
                    let box_relative_zones = choice_menu.get_box_relative_clickable_zones();
                    let translated_zones = self.translate_box_relative_zones_to_absolute(
                        &box_relative_zones,
                        &bounds,
                        content_width,
                        content_height,
                        viewable_width,
                        viewable_height,
                        horizontal_scroll,
                        vertical_scroll,
                        false // not wrapped
                    );
                    
                    // Store translated zones for click detection
                    self.store_translated_clickable_zones(translated_zones);
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
                buffer,
            );
        }

        // Special overflow behaviors are now handled within the unified content rendering path

        // Pass the actual rendered content for proper scrollbar detection
        let rendered_content = if should_render_choices {
            // For choices, use the choice content that was actually rendered
            streams
                .values()
                .find(|s| matches!(s.stream_type, StreamType::Choices))
                .map(|stream| {
                    let choices = stream.get_choices();
                    let choice_menu = ChoiceMenu::new(
                        format!("{}_choice_menu", self.component_id),
                        &choices
                    )
                    .with_selection(self.muxbox.selected_choice_index())
                    .with_focus(self.muxbox.focused_choice_index());
                    
                    log::info!("CHOICE RENDER: Secondary ChoiceMenu created for content only, muxbox '{}'", self.muxbox.id);
                    choice_menu.get_raw_content()
                })
        } else {
            content.map(|s| s.to_string())
        };

        self.render_borders(&bounds, &border_color, &bg_color, scrollbars_drawn, locked, rendered_content.as_deref(), buffer);
    }

    /// Render content with scrolling and overflow handling
    fn render_content(
        &self,
        bounds: &Bounds,
        content: &str,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
        border_color: &Option<String>,
        parent_bg_color: &Option<String>,
        overflow_behavior: &str,
        horizontal_scroll: f64,
        vertical_scroll: f64,
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
                buffer,
            );
            // Return true if any scrollbar is drawn
            max_content_height > viewable_height || max_content_width > viewable_width
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
        fg_color: &Option<String>,
        bg_color: &Option<String>,
        border_color: &Option<String>,
        buffer: &mut ScreenBuffer,
    ) {
        let viewable_height = bounds.height().saturating_sub(1);
        
        let max_horizontal_offset = max_content_width.saturating_sub(viewable_width);
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
                .take(viewable_width)
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

        if should_draw_color(border_color) || should_draw_color(bg_color) {
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

        // Draw scrollbars using components only when needed
        if max_content_height > viewable_height {
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
        }

        if max_content_width > viewable_width {
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
    }

    /// Render wrapped content using integrated overflow logic
    fn render_wrapped_content(
        &self,
        bounds: &Bounds,
        content: &str,
        vertical_scroll: f64,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
        border_color: &Option<String>,
        _parent_bg_color: &Option<String>,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let viewable_width = bounds.width().saturating_sub(4);
        let wrapped_content = wrap_text_to_width(content, viewable_width);

        let viewable_height = bounds.height().saturating_sub(4);
        let wrapped_overflows_vertically = wrapped_content.len() > viewable_height;

        render_wrapped_content(
            &wrapped_content,
            bounds,
            vertical_scroll,
            fg_color,
            bg_color,
            buffer,
        );

        // Draw vertical scrollbar if wrapped content overflows
        if wrapped_overflows_vertically && should_draw_color(border_color) {
            let vertical_scrollbar = VerticalScrollbar::new(format!("{}_wrapped_text", self.component_id));
            vertical_scrollbar.draw(
                bounds,
                wrapped_content.len(),
                viewable_height,
                vertical_scroll,
                border_color,
                bg_color,
                buffer,
            );
            true
        } else {
            false
        }
    }

    /// Render normal (non-overflowing) content
    fn render_normal_content(
        &self,
        bounds: &Bounds,
        content_lines: &[&str],
        max_content_width: usize,
        viewable_width: usize,
        viewable_height: usize,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
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

    /// Render borders with proper corner handling - supports transparent colors
    fn render_borders(
        &self,
        bounds: &Bounds,
        border_color: &Option<String>,
        bg_color: &Option<String>,
        scrollbars_drawn: bool,
        locked: bool,
        content: Option<&str>,
        buffer: &mut ScreenBuffer,
    ) {
        // Skip border drawing if border color is transparent
        if !should_draw_color(border_color) {
            return;
        }

        let border_color_code = get_fg_color_transparent(border_color);
        let bg_color_code = get_bg_color_transparent(bg_color);

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
        let has_vertical_scrollbar = content.is_some() && {
            if let Some(content_str) = content {
                let content_lines: Vec<&str> = content_str.lines().collect();
                let content_height = content_lines.len();
                let viewable_height = bounds.height().saturating_sub(4);
                content_height > viewable_height
            } else {
                false
            }
        };

        if !has_vertical_scrollbar {
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

    /// Unified text overflow rendering with integrated overflow behaviors
    fn render_unified_text_overflow(
        &self,
        content: &str,
        bounds: &Bounds,
        vertical_scroll: f64,
        horizontal_scroll: f64,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
        border_color: &Option<String>,
        parent_bg_color: &Option<String>,
        behavior: &UnifiedOverflowBehavior,
        buffer: &mut ScreenBuffer,
    ) -> bool {
        let content_lines: Vec<&str> = content.lines().collect();
        let content_height = content_lines.len();
        let content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        let overflows = content_width > viewable_width || content_height > viewable_height;

        if !overflows {
            return false;
        }

        match behavior {
            UnifiedOverflowBehavior::Fill(fill_char) => {
                fill_muxbox(bounds, true, bg_color, &None, *fill_char, buffer);
                true
            }
            UnifiedOverflowBehavior::CrossOut => {
                self.render_cross_out_pattern(bounds, border_color, parent_bg_color, buffer);
                true
            }
            UnifiedOverflowBehavior::Removed => {
                fill_muxbox(bounds, false, parent_bg_color, &None, ' ', buffer);
                true
            }
            UnifiedOverflowBehavior::Scroll => {
                self.render_unified_scrollable_text(
                    content,
                    bounds,
                    vertical_scroll,
                    horizontal_scroll,
                    fg_color,
                    bg_color,
                    border_color,
                    buffer,
                );
                true
            }
            UnifiedOverflowBehavior::Wrap => {
                self.render_wrapped_content(
                    bounds,
                    content,
                    vertical_scroll,
                    fg_color,
                    bg_color,
                    border_color,
                    parent_bg_color,
                    buffer,
                )
            }
            UnifiedOverflowBehavior::Clip => false, // No special handling for clipping
        }
    }

    /// Render cross-out pattern for disabled/removed content
    fn render_cross_out_pattern(
        &self,
        bounds: &Bounds,
        border_color: &Option<String>,
        parent_bg_color: &Option<String>,
        buffer: &mut ScreenBuffer,
    ) {
        let border_color_code = get_fg_color_transparent(border_color);
        let parent_bg_color_code = get_bg_color_transparent(parent_bg_color);
        let cross_char = 'X';

        // Draw diagonal cross pattern
        let width = bounds.width();
        let height = bounds.height();

        for i in 0..width.min(height) {
            // Draw main diagonal (top-left to bottom-right)
            if i < height {
                let cell = crate::model::common::Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: parent_bg_color_code.clone(),
                    ch: cross_char,
                };
                buffer.update(bounds.left() + i, bounds.top() + i, cell);
            }

            // Draw anti-diagonal (top-right to bottom-left)
            if i < height && (width.saturating_sub(1).saturating_sub(i)) < width {
                let cell = crate::model::common::Cell {
                    fg_color: border_color_code.clone(),
                    bg_color: parent_bg_color_code.clone(),
                    ch: cross_char,
                };
                buffer.update(
                    bounds.left() + width.saturating_sub(1).saturating_sub(i),
                    bounds.top() + i,
                    cell,
                );
            }
        }
    }

    /// Render scrollable text content using existing scrollbar components
    fn render_unified_scrollable_text(
        &self,
        content: &str,
        bounds: &Bounds,
        vertical_scroll: f64,
        horizontal_scroll: f64,
        fg_color: &Option<String>,
        bg_color: &Option<String>,
        border_color: &Option<String>,
        buffer: &mut ScreenBuffer,
    ) {
        let content_lines: Vec<&str> = content.lines().collect();
        let content_height = content_lines.len();
        let max_content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        let viewable_width = bounds.width().saturating_sub(4);
        let viewable_height = bounds.height().saturating_sub(4);

        // Calculate offsets
        let y_offset = if content_height > viewable_height {
            ((content_height - viewable_height) as f64 * vertical_scroll / 100.0).round() as usize
        } else {
            0
        };

        let x_offset = if max_content_width > viewable_width {
            ((max_content_width - viewable_width + 3) as f64 * horizontal_scroll / 100.0).round() as usize
        } else {
            0
        };

        // Render visible content lines
        for (display_y, &line) in content_lines
            .iter()
            .skip(y_offset)
            .take(viewable_height)
            .enumerate()
        {
            let visible_line = if line.len() > x_offset {
                line.chars()
                    .skip(x_offset)
                    .take(viewable_width)
                    .collect::<String>()
            } else {
                String::new()
            };

            print_with_color_and_background_at(
                bounds.top() + 1 + display_y,
                bounds.left() + 2,
                fg_color,
                bg_color,
                &visible_line,
                buffer,
            );
        }

        // Draw scrollbars if needed using existing scrollbar components
        if should_draw_color(border_color) {
            if content_height > viewable_height {
                let vertical_scrollbar = VerticalScrollbar::new(format!("{}_unified_vertical", self.component_id));
                vertical_scrollbar.draw(
                    bounds,
                    content_height,
                    viewable_height,
                    vertical_scroll,
                    border_color,
                    bg_color,
                    buffer,
                );
            }

            if max_content_width > viewable_width {
                let horizontal_scrollbar = HorizontalScrollbar::new(format!("{}_unified_horizontal", self.component_id));
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
        }
    }
    
    /// Translate box-relative clickable zones to absolute screen coordinates
    /// This handles centering, scroll offsets, and coordinate translation
    pub fn translate_box_relative_zones_to_absolute(
        &self,
        box_relative_zones: &[ClickableZone],
        bounds: &Bounds,
        content_width: usize,
        content_height: usize,
        viewable_width: usize,
        viewable_height: usize,
        horizontal_scroll: f64,
        vertical_scroll: f64,
        is_wrapped: bool,
    ) -> Vec<ClickableZone> {
        let mut translated_zones = Vec::new();
        
        // Calculate same offsets as render_normal_content
        let vertical_padding = (viewable_height.saturating_sub(content_height)) / 2;
        let horizontal_padding = (viewable_width.saturating_sub(content_width)) / 2;
        
        // Calculate scroll offsets
        let horizontal_offset = if content_width > viewable_width {
            ((content_width - viewable_width + 3) as f64 * horizontal_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        let vertical_offset = if content_height > viewable_height {
            ((content_height - viewable_height) as f64 * vertical_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        log::info!("TRANSLATE ZONES: content={}x{}, viewable={}x{}, padding={}x{}, scroll_offset={}x{}", 
                   content_width, content_height, viewable_width, viewable_height, 
                   horizontal_padding, vertical_padding, horizontal_offset, vertical_offset);
        
        for zone in box_relative_zones {
            // Skip zones that are scrolled out of view
            if zone.bounds.y1 < vertical_offset || zone.bounds.y1 >= vertical_offset + viewable_height {
                continue;
            }
            if zone.bounds.x1 < horizontal_offset || zone.bounds.x1 >= horizontal_offset + viewable_width {
                continue;
            }
            
            // Use isolated coordinate translation methods
            let (absolute_x1, absolute_y1) = self.translate_inbox_to_screen_coordinates(
                zone.bounds.x1, zone.bounds.y1, bounds, content_width, content_height, 
                viewable_width, viewable_height, horizontal_scroll, vertical_scroll
            );
            let (absolute_x2, absolute_y2) = self.translate_inbox_to_screen_coordinates(
                zone.bounds.x2, zone.bounds.y2, bounds, content_width, content_height, 
                viewable_width, viewable_height, horizontal_scroll, vertical_scroll
            );
            
            let translated_bounds = Bounds::new(absolute_x1, absolute_y1, absolute_x2, absolute_y2);
            
            log::info!("TRANSLATE ZONE: '{}' inbox=({},{} to {},{}) -> screen=({},{} to {},{})",
                       zone.content_id, zone.bounds.x1, zone.bounds.y1, zone.bounds.x2, zone.bounds.y2,
                       absolute_x1, absolute_y1, absolute_x2, absolute_y2);
            
            let mut translated_zone = zone.clone();
            translated_zone.bounds = translated_bounds;
            translated_zones.push(translated_zone);
        }
        
        translated_zones
    }
    
    /// Store translated clickable zones for click detection
    pub fn store_translated_clickable_zones(&mut self, zones: Vec<ClickableZone>) {
        self.clickable_zones = zones;
        log::info!("STORE ZONES: Stored {} translated clickable zones for muxbox '{}'", 
                   self.clickable_zones.len(), self.muxbox.id);
    }
    
    /// Get clickable zones in absolute screen coordinates
    pub fn get_clickable_zones(&self) -> &[ClickableZone] {
        &self.clickable_zones
    }
    
    /// Translate inbox coordinates to screen coordinates
    /// Inbox coordinates: content-local (0,0 = top-left of content area)
    /// Screen coordinates: absolute terminal coordinates
    pub fn translate_inbox_to_screen_coordinates(
        &self,
        inbox_x: usize,
        inbox_y: usize,
        bounds: &Bounds,
        content_width: usize,
        content_height: usize,
        viewable_width: usize,
        viewable_height: usize,
        horizontal_scroll: f64,
        vertical_scroll: f64,
    ) -> (usize, usize) {
        // Calculate padding offsets
        let vertical_padding = (viewable_height.saturating_sub(content_height)) / 2;
        let horizontal_padding = (viewable_width.saturating_sub(content_width)) / 2;
        
        // Calculate scroll offsets
        let horizontal_offset = if content_width > viewable_width {
            ((content_width - viewable_width + 3) as f64 * horizontal_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        let vertical_offset = if content_height > viewable_height {
            ((content_height - viewable_height) as f64 * vertical_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        // Forward translate: inbox coordinates -> screen coordinates
        let screen_x = bounds.left() + 2 + horizontal_padding + inbox_x - horizontal_offset;
        let screen_y = bounds.top() + 2 + vertical_padding + inbox_y - vertical_offset;
        
        (screen_x, screen_y)
    }

    /// Translate screen coordinates to inbox coordinates
    /// Screen coordinates: absolute terminal coordinates
    /// Inbox coordinates: content-local (0,0 = top-left of content area)
    pub fn translate_screen_to_inbox_coordinates(
        &self,
        screen_x: usize,
        screen_y: usize,
        bounds: &Bounds,
        content_width: usize,
        content_height: usize,
        viewable_width: usize,
        viewable_height: usize,
        horizontal_scroll: f64,
        vertical_scroll: f64,
    ) -> Option<(usize, usize)> {
        // Calculate padding offsets
        let vertical_padding = (viewable_height.saturating_sub(content_height)) / 2;
        let horizontal_padding = (viewable_width.saturating_sub(content_width)) / 2;
        
        // Calculate scroll offsets
        let horizontal_offset = if content_width > viewable_width {
            ((content_width - viewable_width + 3) as f64 * horizontal_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        let vertical_offset = if content_height > viewable_height {
            ((content_height - viewable_height) as f64 * vertical_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        // Calculate content area bounds
        let content_left = bounds.left() + 2 + horizontal_padding;
        let content_top = bounds.top() + 2 + vertical_padding;
        let content_right = content_left + viewable_width;
        let content_bottom = content_top + viewable_height;
        
        // Check if click is within content area
        if screen_x < content_left || screen_x >= content_right || 
           screen_y < content_top || screen_y >= content_bottom {
            return None;
        }
        
        // Reverse translate: screen coordinates -> inbox coordinates
        let inbox_x = (screen_x - content_left) + horizontal_offset;
        let inbox_y = (screen_y - content_top) + vertical_offset;
        
        Some((inbox_x, inbox_y))
    }

    /// Handle click at absolute screen coordinates
    /// Returns true if click was handled by a renderable content
    pub fn handle_click(
        &mut self, 
        click_x: usize, 
        click_y: usize,
        bounds: &Bounds,
        content_width: usize,
        content_height: usize,
        viewable_width: usize,
        viewable_height: usize,
        horizontal_scroll: f64,
        vertical_scroll: f64,
    ) -> bool {
        log::info!("BOX CLICK: Checking click at ({}, {}) against {} zones for muxbox '{}'", 
                   click_x, click_y, self.clickable_zones.len(), self.muxbox.id);
        
        for zone in &self.clickable_zones {
            if zone.bounds.contains_point(click_x, click_y) {
                log::info!("BOX CLICK: Found zone '{}' contains click ({}, {})", zone.content_id, click_x, click_y);
                
                // Translate screen coordinates to inbox coordinates before passing to renderable content
                if let Some((inbox_x, inbox_y)) = self.translate_screen_to_inbox_coordinates(
                    click_x, click_y, bounds, content_width, content_height, 
                    viewable_width, viewable_height, horizontal_scroll, vertical_scroll
                ) {
                    log::info!("BOX CLICK: Translated screen coords ({}, {}) to inbox coords ({}, {}) for zone '{}'", 
                               click_x, click_y, inbox_x, inbox_y, zone.content_id);
                    
                    // TODO: Create ContentEvent with inbox coordinates and pass to RenderableContent
                    // let event = ContentEvent::new_click(Some((inbox_x, inbox_y)), Some(zone.content_id.clone()));
                    // let result = renderable_content.handle_event(&event);
                    
                    log::info!("BOX CLICK: Would handle click on zone '{}' with inbox coords ({}, {}) - implementation needed", 
                               zone.content_id, inbox_x, inbox_y);
                    return true;
                } else {
                    log::warn!("BOX CLICK: Failed to translate screen coordinates to inbox coordinates");
                    return false;
                }
            }
        }
        
        log::info!("BOX CLICK: No zone found for click ({}, {})", click_x, click_y);
        false
    }

    /// Handle mouse move at absolute screen coordinates
    /// Returns true if move was handled by a renderable content
    pub fn handle_mouse_move(
        &mut self, 
        screen_x: usize, 
        screen_y: usize,
        bounds: &Bounds,
        content_width: usize,
        content_height: usize,
        viewable_width: usize,
        viewable_height: usize,
        horizontal_scroll: f64,
        vertical_scroll: f64,
    ) -> bool {
        // Translate screen coordinates to inbox coordinates before passing to renderable content
        if let Some((inbox_x, inbox_y)) = self.translate_screen_to_inbox_coordinates(
            screen_x, screen_y, bounds, content_width, content_height, 
            viewable_width, viewable_height, horizontal_scroll, vertical_scroll
        ) {
            log::info!("BOX MOUSE_MOVE: Translated screen coords ({}, {}) to inbox coords ({}, {})", 
                       screen_x, screen_y, inbox_x, inbox_y);
            
            // TODO: Create ContentEvent with inbox coordinates and pass to RenderableContent
            // let event = ContentEvent::new_mouse_move(None, (inbox_x, inbox_y), None);
            // let result = renderable_content.handle_event(&event);
            
            log::info!("BOX MOUSE_MOVE: Would handle mouse move with inbox coords ({}, {}) - implementation needed", 
                       inbox_x, inbox_y);
            return true;
        }
        
        false
    }

    /// Handle mouse hover at absolute screen coordinates
    /// Returns true if hover was handled by a renderable content
    pub fn handle_mouse_hover(
        &mut self, 
        screen_x: usize, 
        screen_y: usize,
        bounds: &Bounds,
        content_width: usize,
        content_height: usize,
        viewable_width: usize,
        viewable_height: usize,
        horizontal_scroll: f64,
        vertical_scroll: f64,
    ) -> bool {
        for zone in &self.clickable_zones {
            if zone.bounds.contains_point(screen_x, screen_y) {
                // Translate screen coordinates to inbox coordinates before passing to renderable content
                if let Some((inbox_x, inbox_y)) = self.translate_screen_to_inbox_coordinates(
                    screen_x, screen_y, bounds, content_width, content_height, 
                    viewable_width, viewable_height, horizontal_scroll, vertical_scroll
                ) {
                    log::info!("BOX HOVER: Translated screen coords ({}, {}) to inbox coords ({}, {}) for zone '{}'", 
                               screen_x, screen_y, inbox_x, inbox_y, zone.content_id);
                    
                    // TODO: Create ContentEvent with inbox coordinates and pass to RenderableContent
                    // let event = ContentEvent::new_hover((inbox_x, inbox_y), Some(zone.content_id.clone()));
                    // let result = renderable_content.handle_event(&event);
                    
                    log::info!("BOX HOVER: Would handle hover on zone '{}' with inbox coords ({}, {}) - implementation needed", 
                               zone.content_id, inbox_x, inbox_y);
                    return true;
                }
            }
        }
        
        false
    }

    /// Handle mouse drag from screen coordinates to screen coordinates
    /// Returns true if drag was handled by a renderable content
    pub fn handle_mouse_drag(
        &mut self, 
        from_screen_x: usize, 
        from_screen_y: usize, 
        to_screen_x: usize, 
        to_screen_y: usize,
        bounds: &Bounds,
        content_width: usize,
        content_height: usize,
        viewable_width: usize,
        viewable_height: usize,
        horizontal_scroll: f64,
        vertical_scroll: f64,
    ) -> bool {
        // Translate both from and to coordinates to inbox coordinates
        if let (Some((from_inbox_x, from_inbox_y)), Some((to_inbox_x, to_inbox_y))) = (
            self.translate_screen_to_inbox_coordinates(
                from_screen_x, from_screen_y, bounds, content_width, content_height, 
                viewable_width, viewable_height, horizontal_scroll, vertical_scroll
            ),
            self.translate_screen_to_inbox_coordinates(
                to_screen_x, to_screen_y, bounds, content_width, content_height, 
                viewable_width, viewable_height, horizontal_scroll, vertical_scroll
            ),
        ) {
            log::info!("BOX DRAG: Translated screen coords ({}, {}) -> ({}, {}) to inbox coords ({}, {}) -> ({}, {})", 
                       from_screen_x, from_screen_y, to_screen_x, to_screen_y,
                       from_inbox_x, from_inbox_y, to_inbox_x, to_inbox_y);
            
            // TODO: Create ContentEvent with inbox coordinates and pass to RenderableContent
            // let event = ContentEvent::new_mouse_drag((from_inbox_x, from_inbox_y), (to_inbox_x, to_inbox_y), MouseButton::Left, None);
            // let result = renderable_content.handle_event(&event);
            
            log::info!("BOX DRAG: Would handle drag with inbox coords ({}, {}) -> ({}, {}) - implementation needed", 
                       from_inbox_x, from_inbox_y, to_inbox_x, to_inbox_y);
            return true;
        }
        
        false
    }
}