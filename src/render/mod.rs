//! Drawing the shown layout into a frame, and finding what is at a screen cell.
//!
//! Boxes are drawn in a fixed order: siblings by ascending `z_index` (configuration order for
//! equal values), each box before its children. The box at a cell is the last one in that
//! order that covers it, which is the one visible there.

pub mod box_view;
pub mod screen;
pub mod scrollbar;
pub mod styled;
pub mod tab_bar;
pub mod theme;

use crate::model::app::AppContext;
use crate::model::bounds::Bounds;
use crate::model::layout::Layout;
use crate::model::muxbox::MuxBox;
use crate::model::stream::StreamStatus;
use box_view::{BoxHit, BoxSpec, BoxView, Content, Header};
use screen::{ScreenBuffer, Style};
use std::collections::HashMap;
use tab_bar::TabSpec;
use theme::{BoxState, BoxStyle, Scope};

/// A box of a layout with the boxes enclosing it, nearest first.
pub struct Placed<'a> {
    pub muxbox: &'a MuxBox,
    pub ancestors: Vec<&'a MuxBox>,
}

/// The boxes of `layout` in drawing order.
pub fn draw_order(layout: &Layout) -> Vec<Placed<'_>> {
    fn visit<'a>(muxboxes: &'a [MuxBox], ancestors: &[&'a MuxBox], out: &mut Vec<Placed<'a>>) {
        let mut ordered: Vec<&MuxBox> = muxboxes.iter().collect();
        ordered.sort_by_key(|muxbox| muxbox.effective_z_index());
        for muxbox in ordered {
            out.push(Placed {
                muxbox,
                ancestors: ancestors.iter().rev().copied().collect(),
            });
            let mut path = ancestors.to_vec();
            path.push(muxbox);
            visit(muxbox.children.as_deref().unwrap_or_default(), &path, out);
        }
    }
    let mut out = Vec::new();
    visit(
        layout.children.as_deref().unwrap_or_default(),
        &[],
        &mut out,
    );
    out
}

/// The id of the box visible at (`x`, `y`) in `layout`.
pub fn box_at(
    layout: &Layout,
    bounds: &HashMap<String, Bounds>,
    x: usize,
    y: usize,
) -> Option<String> {
    draw_order(layout)
        .iter()
        .rev()
        .find(|placed| {
            bounds
                .get(&placed.muxbox.id)
                .is_some_and(|b| b.contains_point(x, y))
        })
        .map(|placed| placed.muxbox.id.clone())
}

/// Where a box's generated content (chart, table, plugin output) goes: inside the border,
/// with a one-column margin.
fn generated_area(bounds: &Bounds) -> Option<Bounds> {
    bounds.shrink(1, 1)?.shrink(1, 0)
}

/// The data a chart or table is drawn from: the configured data, or the latest output of the
/// box's script.
fn data_text(configured: Option<&String>, muxbox: &MuxBox) -> Option<String> {
    configured.cloned().or_else(|| {
        muxbox
            .get_selected_stream()
            .filter(|stream| stream.choices.is_none())
            .map(|stream| crate::utils::strip_ansi_codes(&stream.content.join("\n")))
    })
}

fn generated_content(ctx: &AppContext, muxbox: &MuxBox, bounds: &Bounds) -> Option<Vec<String>> {
    let wants = muxbox.plugin_component.is_some()
        || muxbox.chart_type.is_some()
        || muxbox.table_data.is_some()
        || muxbox.table_config.is_some();
    if !wants {
        return None;
    }
    let Some(area) = generated_area(bounds) else {
        return Some(Vec::new());
    };
    let (width, height) = (area.width(), area.height());
    let text = if let Some(component) = &muxbox.plugin_component {
        match ctx
            .plugins
            .render(component, muxbox.plugin_config.as_ref(), width, height)
        {
            Ok(text) => text,
            Err(e) => format!("plugin '{}': {}", component, e),
        }
    } else if let Some(chart_type) = muxbox.chart_type {
        let data = data_text(muxbox.chart_data.as_ref(), muxbox).unwrap_or_default();
        match crate::chart::parse_chart_data(&data) {
            Ok(points) => crate::chart::chart_text(chart_type, &points, width, height),
            Err(e) => e,
        }
    } else {
        let data = data_text(muxbox.table_data.as_ref(), muxbox).unwrap_or_default();
        crate::table::table_text(&data, &muxbox.table_config.clone().unwrap_or_default())
    };
    Some(crate::text::sanitize_output(&text))
}

/// How the box's streams appear in its top border: tabs when there are several or one that
/// can be closed, otherwise the box title (or the label of its only stream).
fn header(muxbox: &MuxBox) -> Header<'_> {
    let closeable = muxbox.get_tab_close_buttons();
    if muxbox.streams.len() > 1 || closeable.iter().any(|c| *c) {
        return Header::Tabs {
            tabs: muxbox
                .get_tab_labels()
                .into_iter()
                .zip(closeable)
                .map(|(label, closeable)| TabSpec { label, closeable })
                .collect(),
            active: muxbox.get_active_tab_index(),
            offset: muxbox.tab_scroll_offset,
        };
    }
    match muxbox
        .title
        .as_deref()
        .or_else(|| muxbox.get_selected_stream().map(|s| s.label.as_str()))
    {
        Some(title) => Header::Title(title),
        None => Header::None,
    }
}

/// The view of every box of the shown layout, in drawing order. `hovered_box` is the box under
/// the mouse pointer.
pub fn views(
    ctx: &AppContext,
    bounds: &HashMap<String, Bounds>,
    hovered_box: Option<&str>,
) -> Vec<BoxView> {
    let Some(layout) = ctx.app.get_active_layout() else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for placed in draw_order(layout) {
        let muxbox = placed.muxbox;
        let Some(&box_bounds) = bounds.get(&muxbox.id) else {
            continue;
        };
        let visible = muxbox.get_selected_stream();
        let state = BoxState {
            focused: muxbox.selected == Some(true),
            hovered: hovered_box == Some(muxbox.id.as_str()),
            error: muxbox.in_error_state(),
            ended: visible.is_some_and(|stream| {
                stream.terminal
                    && matches!(
                        stream.status,
                        StreamStatus::Exited(_) | StreamStatus::Killed
                    )
            }),
        };
        let style = BoxStyle::resolve(
            &Scope {
                muxbox,
                ancestors: &placed.ancestors,
                layout,
            },
            state,
        );
        let generated = generated_content(ctx, muxbox, &box_bounds);
        let stream = muxbox.get_selected_stream();
        let content = match (&generated, stream) {
            (Some(lines), _) => Content::Generated(lines.clone()),
            (None, Some(stream)) => match &stream.choices {
                Some(choices) => Content::Menu(choices),
                None if stream.terminal => Content::Terminal(&stream.content),
                None => Content::Text(&stream.content),
            },
            (None, None) => Content::Nothing,
        };
        out.push(BoxView::new(BoxSpec {
            id: &muxbox.id,
            bounds: box_bounds,
            style,
            header: header(muxbox),
            content,
            selected_choice: muxbox
                .get_selected_stream_choices()
                .and_then(|choices| choices.iter().position(|choice| choice.selected)),
            horizontal_scroll: muxbox.current_horizontal_scroll(),
            vertical_scroll: muxbox.current_vertical_scroll(),
            resizable: !ctx.config.locked,
        }));
    }
    out
}

/// The view visible at (`x`, `y`).
pub fn view_at(views: &[BoxView], x: usize, y: usize) -> Option<&BoxView> {
    views
        .iter()
        .rev()
        .find(|view| view.outer.contains_point(x, y))
}

/// Draws the shown layout on a `width` x `height` screen. `hover` is the box under the
/// pointer and what in it the pointer is on.
pub fn draw(
    ctx: &AppContext,
    views: &[BoxView],
    width: usize,
    height: usize,
    hover: Option<(&str, BoxHit)>,
) -> ScreenBuffer {
    let background = ctx
        .app
        .get_active_layout()
        .and_then(theme::layout_background);
    let mut buffer = ScreenBuffer::new(width, height, background.unwrap_or_default());
    if let Some(style) = background {
        let fill_char = ctx
            .app
            .get_active_layout()
            .and_then(|layout| layout.fill_char)
            .unwrap_or(' ');
        if width > 0 && height > 0 {
            buffer.fill(0, 0, width - 1, height - 1, fill_char, style);
        }
    }
    for view in views {
        let hovered = hover.and_then(|(id, hit)| (id == view.id).then_some(hit));
        view.draw(&mut buffer, hovered);
    }
    buffer
}

/// A message bar along the bottom row of the screen.
pub fn draw_notice(buffer: &mut ScreenBuffer, text: &str, is_error: bool) {
    if buffer.width == 0 || buffer.height == 0 {
        return;
    }
    let theme = theme::palette();
    let style: Style = if is_error {
        theme.error_notice
    } else {
        theme.notice
    };
    let y = buffer.height - 1;
    let right = buffer.width - 1;
    buffer.fill(0, y, right, y, ' ', style);
    let line = text.lines().next().unwrap_or_default();
    buffer.put_str(1, y, line, style, right);
}
