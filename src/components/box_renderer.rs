use crate::color_utils::{get_bg_color_transparent, get_fg_color_transparent, should_draw_color};
use crate::components::choice_menu::ChoiceMenu;
use crate::components::renderable_content::{ClickableZone, ContentEvent, EventType, EventData, RenderableContent};
use crate::components::{
    ChartComponent, ChartConfig, ChartType, HorizontalScrollbar, VerticalScrollbar,
};
use crate::draw_utils::{
    content_size, draw_horizontal_line, draw_horizontal_line_with_tabs, draw_vertical_line,
    fill_muxbox, print_with_color_and_background_at, render_wrapped_content, wrap_text_to_width,
};
use crate::model::common::{Cell, ChoicesStreamTrait, ContentStreamTrait, StreamType};
use crate::{AppContext, AppGraph, Bounds, MuxBox, ScreenBuffer};
use std::collections::HashMap;

/// Box dimensions and coordinate system definitions
///
/// This module formalizes the coordinate systems and size measurements for boxes:
/// - **Box Span**: Total space including borders, tabs, scrollbars, all attachments
/// - **Content Area**: Space available for renderable content (inside borders/scrollbars)
/// - **Screen Coordinates**: Absolute terminal coordinates (0,0 = top-left terminal)
/// - **Inbox Coordinates**: Content-local coordinates (0,0 = top-left content cell)
#[derive(Debug, Clone)]
pub struct BoxDimensions {
    /// Total box span including all attachments (borders, scrollbars, tabs)
    pub total_bounds: Bounds,

    /// Content area bounds (where renderable content is placed)
    pub content_bounds: Bounds,

    /// Dimensions of the viewable content area
    pub viewable_width: usize,
    pub viewable_height: usize,

    /// Actual content dimensions (may exceed viewable area)
    pub content_width: usize,
    pub content_height: usize,

    /// Current scroll positions (0-100 percentage)
    pub horizontal_scroll: f64,
    pub vertical_scroll: f64,

    /// Border thickness
    pub border_thickness: usize,

    /// Tab bar height (if tabs are present)
    pub tab_height: usize,

    /// Scrollbar dimensions
    pub vertical_scrollbar_width: usize,
    pub horizontal_scrollbar_height: usize,
}

impl BoxDimensions {
    /// Create new BoxDimensions from MuxBox state
    pub fn new(
        muxbox: &crate::model::muxbox::MuxBox,
        bounds: &Bounds,
        content_width: usize,
        content_height: usize,
    ) -> Self {
        let border_thickness = 2; // Standard border is 2 chars wide
        let tab_height = if muxbox.streams.len() > 1 { 1 } else { 0 };
        let vertical_scrollbar_width = if content_height > bounds.height().saturating_sub(border_thickness + tab_height) { 1 } else { 0 };
        let horizontal_scrollbar_height = if content_width > bounds.width().saturating_sub(border_thickness + vertical_scrollbar_width) { 1 } else { 0 };
        
        let viewable_width = bounds.width().saturating_sub(border_thickness + vertical_scrollbar_width);
        let viewable_height = bounds.height().saturating_sub(border_thickness + tab_height + horizontal_scrollbar_height);
        
        let content_bounds = Bounds::new(
            bounds.left() + (border_thickness / 2),
            bounds.top() + (border_thickness / 2) + tab_height,
            bounds.left() + (border_thickness / 2) + viewable_width,
            bounds.top() + (border_thickness / 2) + tab_height + viewable_height,
        );
        
        Self {
            total_bounds: bounds.clone(),
            content_bounds,
            viewable_width,
            viewable_height,
            content_width,
            content_height,
            horizontal_scroll: muxbox.current_horizontal_scroll(),
            vertical_scroll: muxbox.current_vertical_scroll(),
            border_thickness,
            tab_height,
            vertical_scrollbar_width,
            horizontal_scrollbar_height,
        }
    }
    
    /// Convert screen coordinates to box span coordinates
    /// Box span includes borders, tabs, scrollbars - the total area the box occupies
    pub fn screen_to_box_span(&self, screen_x: usize, screen_y: usize) -> Option<(usize, usize)> {
        if screen_x < self.total_bounds.left() || screen_x >= self.total_bounds.right() ||
           screen_y < self.total_bounds.top() || screen_y >= self.total_bounds.bottom() {
            return None;
        }
        
        let box_x = screen_x - self.total_bounds.left();
        let box_y = screen_y - self.total_bounds.top();
        Some((box_x, box_y))
    }
    
    /// Convert box span coordinates to screen coordinates
    pub fn box_span_to_screen(&self, box_x: usize, box_y: usize) -> (usize, usize) {
        (
            self.total_bounds.left() + box_x,
            self.total_bounds.top() + box_y,
        )
    }
    
    /// Convert screen coordinates to inbox (content-local) coordinates
    /// Inbox coordinates treat content area as (0,0) origin
    pub fn screen_to_inbox(&self, screen_x: usize, screen_y: usize) -> Option<(usize, usize)> {
        // Check if point is within content area
        if screen_x < self.content_bounds.left() || screen_x >= self.content_bounds.right() ||
           screen_y < self.content_bounds.top() || screen_y >= self.content_bounds.bottom() {
            return None;
        }
        
        // Calculate content positioning and padding
        let vertical_padding = (self.viewable_height.saturating_sub(self.content_height)) / 2;
        let horizontal_padding = (self.viewable_width.saturating_sub(self.content_width)) / 2;
        
        // Calculate scroll offsets
        let horizontal_offset = if self.content_width > self.viewable_width {
            ((self.content_width - self.viewable_width) as f64 * self.horizontal_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        let vertical_offset = if self.content_height > self.viewable_height {
            ((self.content_height - self.viewable_height) as f64 * self.vertical_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        // Calculate actual content start position
        let content_start_x = self.content_bounds.left() + horizontal_padding;
        let content_start_y = self.content_bounds.top() + vertical_padding;
        
        // Calculate expanded bounds to account for scrolling
        let expanded_left = content_start_x.saturating_sub(horizontal_offset);
        let expanded_top = content_start_y.saturating_sub(vertical_offset);
        let expanded_right = content_start_x + self.viewable_width + horizontal_offset;
        let expanded_bottom = content_start_y + self.viewable_height + vertical_offset;
        
        // Check if within scrolled content area
        if screen_x < expanded_left || screen_x >= expanded_right ||
           screen_y < expanded_top || screen_y >= expanded_bottom {
            return None;
        }
        
        // Convert to inbox coordinates
        let inbox_x = screen_x.saturating_sub(content_start_x.saturating_sub(horizontal_offset));
        let inbox_y = screen_y.saturating_sub(content_start_y.saturating_sub(vertical_offset));
        
        Some((inbox_x, inbox_y))
    }
    
    /// Convert inbox coordinates to screen coordinates
    pub fn inbox_to_screen(&self, inbox_x: usize, inbox_y: usize) -> (usize, usize) {
        // Calculate content positioning and padding
        let vertical_padding = (self.viewable_height.saturating_sub(self.content_height)) / 2;
        let horizontal_padding = (self.viewable_width.saturating_sub(self.content_width)) / 2;
        
        // Calculate scroll offsets
        let horizontal_offset = if self.content_width > self.viewable_width {
            ((self.content_width - self.viewable_width) as f64 * self.horizontal_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        let vertical_offset = if self.content_height > self.viewable_height {
            ((self.content_height - self.viewable_height) as f64 * self.vertical_scroll / 100.0).round() as usize
        } else {
            0
        };
        
        // Calculate screen position
        let screen_x = self.content_bounds.left() + horizontal_padding + inbox_x - horizontal_offset;
        let screen_y = self.content_bounds.top() + vertical_padding + inbox_y - vertical_offset;
        
        (screen_x, screen_y)
    }
    
    /// Check if screen coordinates are within the content area
    pub fn contains_screen_point(&self, screen_x: usize, screen_y: usize) -> bool {
        self.screen_to_inbox(screen_x, screen_y).is_some()
    }
    
    /// Check if screen coordinates are within the total box span
    pub fn contains_screen_point_in_span(&self, screen_x: usize, screen_y: usize) -> bool {
        screen_x >= self.total_bounds.left() && screen_x < self.total_bounds.right() &&
        screen_y >= self.total_bounds.top() && screen_y < self.total_bounds.bottom()
    }
    /// Calculate BoxDimensions from MuxBox and bounds
    pub fn calculate_from_muxbox(muxbox: &MuxBox, total_bounds: Bounds) -> Self {
        // Border thickness determined by presence of border_color
        let border_thickness = if muxbox.border_color.is_some() { 1 } else { 0 };

        // Tab bar height (calculated based on stream presence)
        let tab_height = if muxbox.streams.len() > 1 { 1 } else { 0 };

        // Scrollbar dimensions (standard sizes)
        let vertical_scrollbar_width = 1;
        let horizontal_scrollbar_height = 1;

        // Calculate content bounds accounting for borders and tabs
        let content_left = total_bounds.left() + border_thickness;
        let content_top = total_bounds.top() + border_thickness + tab_height;
        let content_right = total_bounds.right().saturating_sub(border_thickness);
        let content_bottom = total_bounds.bottom().saturating_sub(border_thickness);

        let content_bounds = Bounds::new(content_left, content_top, content_right, content_bottom);

        // Calculate viewable dimensions (content area minus scrollbars if needed)
        let viewable_width = content_bounds
            .width()
            .saturating_sub(vertical_scrollbar_width);
        let viewable_height = content_bounds
            .height()
            .saturating_sub(horizontal_scrollbar_height);

        // Content dimensions from muxbox (if available) or default to viewable
        let (content_width, content_height) =
            Self::get_content_dimensions(muxbox, viewable_width, viewable_height);

        Self {
            total_bounds,
            content_bounds,
            viewable_width,
            viewable_height,
            content_width,
            content_height,
            horizontal_scroll: muxbox.horizontal_scroll.unwrap_or(0.0),
            vertical_scroll: muxbox.vertical_scroll.unwrap_or(0.0),
            border_thickness,
            tab_height,
            vertical_scrollbar_width,
            horizontal_scrollbar_height,
        }
    }

    /// Get content dimensions from muxbox content
    fn get_content_dimensions(
        muxbox: &MuxBox,
        default_width: usize,
        default_height: usize,
    ) -> (usize, usize) {
        // Get content from selected stream or static content
        let content_lines = if let Some(selected_stream) = muxbox.get_selected_stream() {
            selected_stream.content.clone()
        } else {
            // Convert static content string to Vec<String>
            let static_content = muxbox.content.clone().unwrap_or_default();
            if static_content.is_empty() {
                Vec::new()
            } else {
                static_content
                    .lines()
                    .map(|line| line.to_string())
                    .collect()
            }
        };

        if content_lines.is_empty() {
            return (default_width, default_height);
        }

        let content_height = content_lines.len();
        let content_width = content_lines
            .iter()
            .map(|line| line.len())
            .max()
            .unwrap_or(0);

        (content_width, content_height)
    }

    // REMOVED: Duplicate screen_to_inbox and inbox_to_screen methods - using BoxDimensions implementation instead

    /// Check if inbox coordinates are currently visible in the viewable area
    pub fn is_inbox_visible(&self, inbox_x: usize, inbox_y: usize) -> bool {
        // Calculate scroll offsets
        let horizontal_offset = if self.content_width > self.viewable_width {
            ((self.content_width - self.viewable_width) as f64 * self.horizontal_scroll / 100.0)
                .round() as usize
        } else {
            0
        };

        let vertical_offset = if self.content_height > self.viewable_height {
            ((self.content_height - self.viewable_height) as f64 * self.vertical_scroll / 100.0)
                .round() as usize
        } else {
            0
        };

        // Check if coordinates are within visible scroll range
        inbox_x >= horizontal_offset
            && inbox_x < horizontal_offset + self.viewable_width
            && inbox_y >= vertical_offset
            && inbox_y < vertical_offset + self.viewable_height
    }

    /// Get the visible inbox region (scroll window)
    pub fn get_visible_inbox_region(&self) -> (usize, usize, usize, usize) {
        // Calculate scroll offsets
        let horizontal_offset = if self.content_width > self.viewable_width {
            ((self.content_width - self.viewable_width) as f64 * self.horizontal_scroll / 100.0)
                .round() as usize
        } else {
            0
        };

        let vertical_offset = if self.content_height > self.viewable_height {
            ((self.content_height - self.viewable_height) as f64 * self.vertical_scroll / 100.0)
                .round() as usize
        } else {
            0
        };

        (
            horizontal_offset,                       // left
            vertical_offset,                         // top
            horizontal_offset + self.viewable_width, // right
            vertical_offset + self.viewable_height,  // bottom
        )
    }
}

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
    pub fn from_behavior_str(behavior: &str) -> Self {
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
    /// Formalized dimensions and coordinate system
    dimensions: Option<BoxDimensions>,
    /// Translated clickable zones in absolute screen coordinates
    clickable_zones: Vec<ClickableZone>,
}

impl<'a> BoxRenderer<'a> {
    /// Create a new BoxRenderer for the given MuxBox
    pub fn new(muxbox: &'a MuxBox, component_id: String) -> Self {
        Self {
            muxbox,
            component_id,
            dimensions: None,
            clickable_zones: Vec::new(),
        }
    }

    /// Initialize dimensions for this box renderer
    pub fn initialize_dimensions(&mut self, bounds: Bounds) {
        self.dimensions = Some(BoxDimensions::calculate_from_muxbox(self.muxbox, bounds));
    }

    /// Get the formalized dimensions for this box
    pub fn get_dimensions(&self) -> Option<&BoxDimensions> {
        self.dimensions.as_ref()
    }

    /// Translate screen coordinates to inbox coordinates using formalized system
    /// This is the primary method all coordinate translation should use
    pub fn screen_to_inbox_coords(
        &self,
        screen_x: usize,
        screen_y: usize,
    ) -> Option<(usize, usize)> {
        self.dimensions
            .as_ref()?
            .screen_to_inbox(screen_x, screen_y)
    }

    /// Translate inbox coordinates to screen coordinates using formalized system
    /// This is the primary method all coordinate translation should use
    pub fn inbox_to_screen_coords(&self, inbox_x: usize, inbox_y: usize) -> Option<(usize, usize)> {
        if let Some(dimensions) = &self.dimensions {
            let (screen_x, screen_y) = dimensions.inbox_to_screen(inbox_x, inbox_y);
            // Return None if coordinates are out of visible area
            if screen_x == usize::MAX || screen_y == usize::MAX {
                None
            } else {
                Some((screen_x, screen_y))
            }
        } else {
            None
        }
    }

    /// Check if inbox coordinates are currently visible
    pub fn is_inbox_coords_visible(&self, inbox_x: usize, inbox_y: usize) -> bool {
        self.dimensions
            .as_ref()
            .map(|d| d.is_inbox_visible(inbox_x, inbox_y))
            .unwrap_or(false)
    }

    /// Generate chart content if the muxbox has chart configuration
    /// This moves chart rendering responsibility from MuxBox to BoxRenderer
    fn generate_chart_content(&self, bounds: &Bounds) -> Option<String> {
        if let (Some(chart_type_str), Some(chart_data)) =
            (&self.muxbox.chart_type, &self.muxbox.chart_data)
        {
            let data = ChartComponent::parse_chart_data(chart_data);

            // Parse chart type with support for all variants including pie and scatter
            let chart_type = chart_type_str
                .parse::<ChartType>()
                .unwrap_or(ChartType::Bar);

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
                config,
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
        let muxbox_adjusted_bounds =
            layout_adjusted_bounds.and_then(|bounds| bounds.get(&self.muxbox.id));

        let Some(bounds) = muxbox_adjusted_bounds else {
            log::error!("Calculated bounds for muxbox {} not found", &self.muxbox.id);
            return false;
        };

        // Calculate all colors and properties (same logic as draw_muxbox)
        let muxbox_parent = app_graph.get_parent(&layout.id, &self.muxbox.id);
        let bg_color = self.muxbox.calc_bg_color(app_context, app_graph);
        let parent_bg_color = if let Some(parent) = muxbox_parent {
            parent
                .calc_bg_color(app_context, app_graph)
                .map(|s| s.to_string())
        } else {
            layout.bg_color.clone().map(|s| s.to_string())
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
            self.muxbox
                .calc_border_color(app_context, app_graph)
                .map(|s| s.to_string())
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
            &self
                .muxbox
                .calc_selected_menu_fg_color(app_context, app_graph),
            &self
                .muxbox
                .calc_selected_menu_bg_color(app_context, app_graph),
            &fg_color,
            &overflow_behavior,
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
        _selected_menu_fg_color: &Option<String>,
        _selected_menu_bg_color: &Option<String>,
        fg_color: &Option<String>,
        overflow_behavior: &str,
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
            let selected_stream = self.muxbox.get_selected_stream();
            let should_render_choices = if let Some(stream) = selected_stream {
                matches!(stream.stream_type, StreamType::Choices)
            } else {
                false
            };

            let content_str = if should_render_choices {
                None
            } else if let Some(stream) = selected_stream {
                match stream.stream_type {
                    StreamType::Content
                    | StreamType::RedirectedOutput(_)
                    | StreamType::PTY
                    | StreamType::Plugin(_)
                    | StreamType::ChoiceExecution(_)
                    | StreamType::PtySession(_)
                    | StreamType::OwnScript => Some(stream.get_content_lines().join("\n")),
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
        let _choices = if should_render_choices {
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
        let bounds = bounds
            .intersection(&screen_bounds)
            .unwrap_or_else(|| bounds.clone());

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
                border_color,
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
                    let choice_menu =
                        ChoiceMenu::new(format!("{}_choice_menu", self.component_id), choices)
                            .with_selection(self.muxbox.selected_choice_index())
                            .with_focus(self.muxbox.focused_choice_index());

                    log::info!("CHOICE RENDER: BoxRenderer creating ChoiceMenu for muxbox '{}' with {} choices, dimensions: {:?}", 
                             self.muxbox.id, choices.len(), choice_menu.get_dimensions());

                    // Get choice content as string - treat it like any other content
                    let choice_content = choice_menu.get_raw_content();
                    log::info!(
                        "CHOICE RENDER: Generated choice content length: {} chars",
                        choice_content.len()
                    );

                    // Use unified content rendering (same path as text content)
                    let (content_width, content_height) = content_size(&choice_content);
                    let viewable_width = bounds.width().saturating_sub(4);
                    let viewable_height = bounds.height().saturating_sub(4);
                    let _choices_overflow =
                        content_width > viewable_width || content_height > viewable_height;

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
                        false, // not wrapped
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
                    let choice_menu =
                        ChoiceMenu::new(format!("{}_choice_menu", self.component_id), choices)
                            .with_selection(self.muxbox.selected_choice_index())
                            .with_focus(self.muxbox.focused_choice_index());

                    log::info!(
                        "CHOICE RENDER: Secondary ChoiceMenu created for content only, muxbox '{}'",
                        self.muxbox.id
                    );
                    choice_menu.get_raw_content()
                })
        } else {
            content.map(|s| s.to_string())
        };

        self.render_borders(
            &bounds,
            border_color,
            bg_color,
            scrollbars_drawn,
            locked,
            rendered_content.as_deref(),
            buffer,
        );
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
        _viewable_height: usize,
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

        let horizontal_offset =
            ((horizontal_scroll / 100.0) * max_horizontal_offset as f64).floor() as usize;
        let vertical_offset =
            ((vertical_scroll / 100.0) * max_vertical_offset as f64).floor() as usize;

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
            let vertical_scrollbar =
                VerticalScrollbar::new(format!("{}_wrapped_text", self.component_id));
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
            let visible_line = &line.chars().take(viewable_width).collect::<String>();

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
        _scrollbars_drawn: bool,
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

    /// Translate box-relative clickable zones to absolute screen coordinates
    /// This handles centering, scroll offsets, and coordinate translation
    /// CRITICAL FIX: Updated to use BoxDimensions for proper coordinate translation
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
        _is_wrapped: bool,
    ) -> Vec<ClickableZone> {
        // CRITICAL FIX: Use BoxDimensions for consistent coordinate translation
        let dimensions = BoxDimensions {
            total_bounds: bounds.clone(),
            content_bounds: Bounds::new(
                bounds.left() + 1,
                bounds.top() + 1,
                bounds.right() - 1,
                bounds.bottom() - 1,
            ),
            viewable_width,
            viewable_height,
            content_width,
            content_height,
            horizontal_scroll,
            vertical_scroll,
            border_thickness: 1,
            tab_height: 0,
            vertical_scrollbar_width: 1,
            horizontal_scrollbar_height: 1,
        };

        let mut translated_zones = Vec::new();

        let (visible_left, visible_top, visible_right, visible_bottom) = dimensions.get_visible_inbox_region();

        log::info!(
            "TRANSLATE ZONES: content={}x{}, viewable={}x{}, scroll={}%x{}%, visible_region=({},{} to {},{})",
            content_width,
            content_height,
            viewable_width,
            viewable_height,
            horizontal_scroll,
            vertical_scroll,
            visible_left,
            visible_top,
            visible_right,
            visible_bottom
        );

        for zone in box_relative_zones {
            // Skip zones that are scrolled out of view using BoxDimensions visibility check
            if !dimensions.is_inbox_visible(zone.bounds.x1, zone.bounds.y1) {
                continue;
            }

            // CRITICAL FIX: Use BoxDimensions for consistent coordinate translation
            let (absolute_x1, absolute_y1) = dimensions.inbox_to_screen(
                zone.bounds.x1,
                zone.bounds.y1,
            );
            let (absolute_x2, absolute_y2) = dimensions.inbox_to_screen(
                zone.bounds.x2,
                zone.bounds.y2,
            );

            let translated_bounds = Bounds::new(absolute_x1, absolute_y1, absolute_x2, absolute_y2);

            log::info!(
                "TRANSLATE ZONE: '{}' inbox=({},{} to {},{}) -> screen=({},{} to {},{})",
                zone.content_id,
                zone.bounds.x1,
                zone.bounds.y1,
                zone.bounds.x2,
                zone.bounds.y2,
                absolute_x1,
                absolute_y1,
                absolute_x2,
                absolute_y2
            );

            let mut translated_zone = zone.clone();
            translated_zone.bounds = translated_bounds;
            translated_zones.push(translated_zone);
        }

        translated_zones
    }

    /// Store clickable zones for click detection
    /// CRITICAL FIX: Now stores zones in screen coordinates after proper translation
    pub fn store_translated_clickable_zones(&mut self, zones: Vec<ClickableZone>) {
        self.clickable_zones = zones;
        log::info!(
            "STORE ZONES: Stored {} screen-coordinate clickable zones for muxbox '{}'",
            self.clickable_zones.len(),
            self.muxbox.id
        );
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
            ((content_width - viewable_width + 3) as f64 * horizontal_scroll / 100.0).round()
                as usize
        } else {
            0
        };

        let vertical_offset = if content_height > viewable_height {
            ((content_height - viewable_height) as f64 * vertical_scroll / 100.0).round() as usize
        } else {
            0
        };

        // Forward translate: inbox coordinates -> screen coordinates
        let screen_x =
            (bounds.left() + 2 + horizontal_padding + inbox_x).wrapping_sub(horizontal_offset);
        let screen_y =
            (bounds.top() + 2 + vertical_padding + inbox_y).wrapping_sub(vertical_offset);

        (screen_x, screen_y)
    }

    /// Translate screen coordinates to inbox coordinates
    /// Screen coordinates: absolute terminal coordinates
    /// Inbox coordinates: content-local (0,0 = top-left of content area)

    /// Handle click using formalized coordinate system
    /// Returns true if click was handled by a renderable content
    pub fn handle_click_with_dimensions(
        &mut self,
        screen_x: usize,
        screen_y: usize,
        dimensions: &BoxDimensions,
    ) -> bool {
        // Use formalized coordinate translation
        if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(screen_x, screen_y) {
            // Check clickable zones in screen coordinates
            for zone in &self.clickable_zones {
                if zone.bounds.contains_point(screen_x, screen_y) {
                    log::info!(
                        "CLICK: Screen ({},{}) -> Inbox ({},{}) on zone '{}'",
                        screen_x, screen_y, inbox_x, inbox_y, zone.content_id
                    );
                    
                    // TODO: Pass inbox coordinates to renderable content
                    // let event = ContentEvent::new_click((inbox_x, inbox_y), MouseButton::Left, Some(zone.content_id.clone()));
                    // let result = renderable_content.handle_event(&event);
                    
                    return true;
                }
            }
        }
        false
    }
    
    /// Handle click using formalized coordinate system
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
        // Create BoxDimensions and delegate to new method
        let mut temp_muxbox = crate::model::muxbox::MuxBox::default();
        temp_muxbox.set_horizontal_scroll(horizontal_scroll);
        temp_muxbox.set_vertical_scroll(vertical_scroll);
        let dimensions = BoxDimensions::new(&temp_muxbox, bounds, content_width, content_height);
        
        self.handle_click_with_dimensions(click_x, click_y, &dimensions)
    }

    /// Handle mouse move using formalized coordinate system
    pub fn handle_mouse_move_with_dimensions(
        &mut self,
        screen_x: usize,
        screen_y: usize,
        dimensions: &BoxDimensions,
    ) -> bool {
        if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(screen_x, screen_y) {
            log::info!(
                "MOUSE_MOVE: Screen ({},{}) -> Inbox ({},{})",
                screen_x, screen_y, inbox_x, inbox_y
            );
            
            // TODO: Create ContentEvent with inbox coordinates and pass to RenderableContent
            // let event = ContentEvent::new_mouse_move(None, (inbox_x, inbox_y), None);
            // let result = renderable_content.handle_event(&event);
            
            return true;
        }
        false
    }
    
    /// Handle mouse move using formalized coordinate system
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
        // Create BoxDimensions and delegate to new method
        let mut temp_muxbox = crate::model::muxbox::MuxBox::default();
        temp_muxbox.set_horizontal_scroll(horizontal_scroll);
        temp_muxbox.set_vertical_scroll(vertical_scroll);
        let dimensions = BoxDimensions::new(&temp_muxbox, bounds, content_width, content_height);
        
        self.handle_mouse_move_with_dimensions(screen_x, screen_y, &dimensions)
    }

    /// Handle mouse hover using formalized coordinate system
    pub fn handle_mouse_hover_with_dimensions(
        &mut self,
        screen_x: usize,
        screen_y: usize,
        dimensions: &BoxDimensions,
    ) -> bool {
        // Check clickable zones using screen coordinates
        for zone in &self.clickable_zones {
            if zone.bounds.contains_point(screen_x, screen_y) {
                if let Some((inbox_x, inbox_y)) = dimensions.screen_to_inbox(screen_x, screen_y) {
                    log::info!(
                        "HOVER: Screen ({},{}) -> Inbox ({},{}) on zone '{}'",
                        screen_x, screen_y, inbox_x, inbox_y, zone.content_id
                    );
                    
                    // TODO: Create ContentEvent with inbox coordinates
                    // let event = ContentEvent::new_hover((inbox_x, inbox_y), Some(zone.content_id.clone()));
                    // let result = renderable_content.handle_event(&event);
                    
                    return true;
                }
            }
        }
        false
    }
    
    /// Handle mouse hover using formalized coordinate system
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
        // Create BoxDimensions and delegate to new method
        let mut temp_muxbox = crate::model::muxbox::MuxBox::default();
        temp_muxbox.set_horizontal_scroll(horizontal_scroll);
        temp_muxbox.set_vertical_scroll(vertical_scroll);
        let dimensions = BoxDimensions::new(&temp_muxbox, bounds, content_width, content_height);
        
        self.handle_mouse_hover_with_dimensions(screen_x, screen_y, &dimensions)
    }

    /// Handle mouse drag using formalized coordinate system
    pub fn handle_mouse_drag_with_dimensions(
        &mut self,
        from_screen_x: usize,
        from_screen_y: usize,
        to_screen_x: usize,
        to_screen_y: usize,
        dimensions: &BoxDimensions,
    ) -> bool {
        // Translate both coordinates to inbox coordinates
        if let (Some((from_inbox_x, from_inbox_y)), Some((to_inbox_x, to_inbox_y))) = (
            dimensions.screen_to_inbox(from_screen_x, from_screen_y),
            dimensions.screen_to_inbox(to_screen_x, to_screen_y),
        ) {
            log::info!(
                "DRAG: Screen ({},{}) -> ({},{}) = Inbox ({},{}) -> ({},{})",
                from_screen_x, from_screen_y, to_screen_x, to_screen_y,
                from_inbox_x, from_inbox_y, to_inbox_x, to_inbox_y
            );
            
            // TODO: Create ContentEvent with inbox coordinates
            // let event = ContentEvent::new_mouse_drag((from_inbox_x, from_inbox_y), (to_inbox_x, to_inbox_y), MouseButton::Left, None);
            // let result = renderable_content.handle_event(&event);
            
            return true;
        }
        false
    }
    
    /// Handle mouse drag using formalized coordinate system
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
        // Create BoxDimensions and delegate to new method
        let mut temp_muxbox = crate::model::muxbox::MuxBox::default();
        temp_muxbox.set_horizontal_scroll(horizontal_scroll);
        temp_muxbox.set_vertical_scroll(vertical_scroll);
        let dimensions = BoxDimensions::new(&temp_muxbox, bounds, content_width, content_height);
        
        self.handle_mouse_drag_with_dimensions(from_screen_x, from_screen_y, to_screen_x, to_screen_y, &dimensions)
    }
}
