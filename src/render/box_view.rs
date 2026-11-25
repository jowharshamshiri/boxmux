//! One box as it appears on screen: its border, title or tabs, the rows of content in view
//! and its scrollbars. The same view is used to draw the box and to find what a mouse event
//! at a cell refers to, so what is drawn and what is clicked always agree.

use super::screen::{ScreenBuffer, Style};
use super::scrollbar::{Scrollbar, ScrollbarHit};
use super::styled::{styled_glyphs, width_of, wrap, StyledGlyph};
use super::tab_bar::{TabBar, TabHit, TabSpec, TabStyles};
use super::theme::BoxStyle;
use crate::model::appearance::{ContentAlign, Overflow, Sides, TitlePosition};
use crate::model::bounds::Bounds;
use crate::model::choice::Choice;

/// What a box's content area shows.
pub enum Content<'a> {
    /// Text lines that may contain SGR color sequences and OSC 8 hyperlinks. Text that fits is
    /// centered unless the box's `content_align` says otherwise.
    Text(&'a [String]),
    /// Generated text (charts, tables, plugin output) laid out from the top-left.
    Generated(Vec<String>),
    /// A choice menu; `selected` is the entry Enter activates.
    Menu(&'a [Choice]),
    /// The screen of a terminal program, filling the whole inside of the border.
    Terminal(&'a [String]),
    Nothing,
}

/// How a box shows its streams in the top border.
pub enum Header<'a> {
    /// Tabs, with the visible one.
    Tabs {
        tabs: Vec<TabSpec>,
        active: usize,
        offset: usize,
    },
    Title(&'a str),
    None,
}

/// What a cell of a box is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxHit {
    /// The bottom-right corner, when the box can be resized.
    ResizeHandle,
    Tab(TabHit),
    /// The top border outside any tab: where the box is dragged from.
    TopBorder,
    VerticalScrollbar(ScrollbarHit),
    HorizontalScrollbar(ScrollbarHit),
    /// A menu entry, by index.
    Choice(usize),
    /// A cell of the content area, relative to its top-left corner.
    Content {
        column: usize,
        row: usize,
    },
    /// Any other part of the box.
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Text,
    Menu,
    Terminal,
}

/// A row of content in view.
#[derive(Debug, Clone)]
struct Row {
    y: usize,
    x: usize,
    chars: Vec<StyledGlyph>,
    choice: Option<usize>,
}

/// The content area of a box and how its content is scrolled.
#[derive(Debug, Clone)]
pub struct Body {
    pub area: Bounds,
    kind: Kind,
    rows: Vec<Row>,
    pub content_width: usize,
    pub content_height: usize,
    pub offset_x: usize,
    pub offset_y: usize,
    /// For each row of content (scrolled or not), the menu entry it belongs to.
    choice_rows: Vec<Option<usize>>,
}

impl Body {
    /// The rows of content, as a range, that show menu entry `choice`.
    pub fn rows_of_choice(&self, choice: usize) -> Option<std::ops::Range<usize>> {
        let first = self.choice_rows.iter().position(|c| *c == Some(choice))?;
        let last = self.choice_rows.iter().rposition(|c| *c == Some(choice))?;
        Some(first..last + 1)
    }

    pub fn max_offset_x(&self) -> usize {
        self.content_width.saturating_sub(self.area.width())
    }

    pub fn max_offset_y(&self) -> usize {
        self.content_height.saturating_sub(self.area.height())
    }
}

/// The scroll percentage that shows offset `offset` of at most `max`.
pub fn percent_for_offset(offset: usize, max: usize) -> f64 {
    if max == 0 {
        0.0
    } else {
        offset.min(max) as f64 / max as f64 * 100.0
    }
}

/// The offset a scroll percentage stands for, given the largest offset `max`.
pub fn offset_for_percent(percent: f64, max: usize) -> usize {
    ((percent.clamp(0.0, 100.0) / 100.0) * max as f64).round() as usize
}

enum Top {
    Tabs { bar: TabBar, active: usize },
    Title { x: usize, text: String },
    Border,
}

pub struct BoxView {
    pub id: String,
    pub outer: Bounds,
    pub style: BoxStyle,
    has_border: bool,
    top: Top,
    pub body: Option<Body>,
    pub vertical_scrollbar: Option<Scrollbar>,
    pub horizontal_scrollbar: Option<Scrollbar>,
    resizable: bool,
}

/// Placement of the box, its content and how it is scrolled.
pub struct BoxSpec<'a> {
    pub id: &'a str,
    pub bounds: Bounds,
    pub style: BoxStyle,
    pub header: Header<'a>,
    pub content: Content<'a>,
    pub selected_choice: Option<usize>,
    pub horizontal_scroll: f64,
    pub vertical_scroll: f64,
    pub resizable: bool,
}

/// `chars` cut to `width` columns, the last one replaced by `…` when anything was cut.
fn truncate(chars: &[StyledGlyph], width: usize) -> Vec<StyledGlyph> {
    if width_of(chars) <= width {
        return chars.to_vec();
    }
    let mut out = Vec::new();
    let mut used = 0;
    for c in chars {
        if used + c.width() > width.saturating_sub(1) {
            break;
        }
        used += c.width();
        out.push(c.clone());
    }
    if width > 0 {
        // The ellipsis stands for what was cut: in its style, and part of its link.
        out.push(chars.get(out.len()).unwrap_or(&chars[0]).replaced_by('…'));
    }
    out
}

/// The part of `chars` from column `skip` that fits in `width` columns. A wide character cut
/// by either edge is replaced by a space so columns stay aligned.
fn window(chars: &[StyledGlyph], skip: usize, width: usize) -> Vec<StyledGlyph> {
    let mut out = Vec::new();
    let mut column = 0;
    for c in chars {
        let w = c.width();
        let end = column + w;
        if end <= skip {
            column = end;
            continue;
        }
        if column >= skip + width {
            break;
        }
        if column < skip || end > skip + width {
            // Straddles an edge: show the visible half as a space, still part of its link.
            out.push(c.replaced_by(' '));
        } else {
            out.push(c.clone());
        }
        column = end;
    }
    out
}

/// Where text, menus and generated content go in a box at `outer`: inside its border, less its
/// padding. `None` when nothing fits.
pub fn content_area(outer: &Bounds, padding: Sides) -> Option<Bounds> {
    outer.shrink(1, 1)?.inset(padding)
}

impl BoxView {
    pub fn new(spec: BoxSpec) -> BoxView {
        let outer = spec.bounds;
        let has_border = outer.width() >= 2 && outer.height() >= 2;
        let inner = if has_border { outer.shrink(1, 1) } else { None };

        let top = match (&spec.header, has_border) {
            (_, false) => Top::Border,
            (
                Header::Tabs {
                    tabs,
                    active,
                    offset,
                },
                true,
            ) => Top::Tabs {
                bar: TabBar::layout(outer.top(), outer.left(), outer.right(), tabs, *offset),
                active: *active,
            },
            (Header::Title(title), true) => {
                Self::place_title(title, &outer, spec.style.title_position)
            }
            (Header::None, true) => Top::Border,
        };

        let mut view = BoxView {
            id: spec.id.to_string(),
            outer,
            style: spec.style,
            has_border,
            top,
            body: None,
            vertical_scrollbar: None,
            horizontal_scrollbar: None,
            resizable: spec.resizable && has_border,
        };
        if let Some(inner) = inner {
            view.lay_out_body(inner, &spec);
        }
        view
    }

    fn place_title(title: &str, outer: &Bounds, position: TitlePosition) -> Top {
        // Room between the corners, keeping one border cell on each side.
        let room = outer.width().saturating_sub(4);
        if title.trim().is_empty() || room < 3 {
            return Top::Border;
        }
        let chars = truncate(&styled_glyphs(title, Style::default()), room - 2);
        let text: String = format!(
            " {} ",
            chars
                .iter()
                .map(|c| c.glyph.to_string())
                .collect::<String>()
        );
        let width = width_of(&chars) + 2;
        let first = outer.left() + 2;
        let last = outer.right() - 2;
        let x = match position {
            TitlePosition::Left => first,
            TitlePosition::Center => first + (last + 1 - first - width) / 2,
            TitlePosition::Right => last + 1 - width,
        };
        Top::Title { x, text }
    }

    fn lay_out_body(&mut self, inner: Bounds, spec: &BoxSpec) {
        let style = self.style;
        let (kind, area) = match spec.content {
            Content::Nothing => return,
            Content::Terminal(_) => (Kind::Terminal, Some(inner)),
            Content::Menu(_) => (Kind::Menu, inner.inset(style.padding)),
            Content::Text(_) | Content::Generated(_) => (Kind::Text, inner.inset(style.padding)),
        };
        let Some(area) = area else {
            return;
        };
        let overflow = if kind == Kind::Terminal {
            Overflow::Scroll
        } else {
            style.overflow
        };

        // Every row of content, before scrolling: its characters and the choice it belongs to.
        // A menu entry is a marker followed by its label; wrapped labels continue under the
        // label, not under the marker.
        let entries: Vec<(Vec<StyledGlyph>, Vec<StyledGlyph>, Option<usize>)> = match &spec.content
        {
            Content::Text(lines) | Content::Terminal(lines) => lines
                .iter()
                .map(|line| (Vec::new(), styled_glyphs(line, style.body), None))
                .collect(),
            Content::Generated(lines) => lines
                .iter()
                .map(|line| (Vec::new(), styled_glyphs(line, style.body), None))
                .collect(),
            Content::Menu(choices) => choices
                .iter()
                .enumerate()
                .map(|(index, choice)| {
                    let selected = spec.selected_choice == Some(index);
                    let item_style = style.choice(choice, selected);
                    let marker = if selected { "► " } else { "  " };
                    let label = choice.content.as_deref().unwrap_or(&choice.id);
                    let waiting = if choice.waiting { " …" } else { "" };
                    (
                        styled_glyphs(marker, item_style),
                        styled_glyphs(&format!("{}{}", label, waiting), item_style),
                        Some(index),
                    )
                })
                .collect(),
            Content::Nothing => unreachable!("handled above"),
        };
        let lines: Vec<(Vec<StyledGlyph>, Option<usize>)> = if overflow == Overflow::Wrap {
            entries
                .into_iter()
                .flat_map(|(prefix, body, choice)| {
                    let indent = width_of(&prefix);
                    let rows = wrap(&body, area.width().saturating_sub(indent).max(1));
                    rows.into_iter().enumerate().map(move |(i, row)| {
                        let lead = if i == 0 {
                            prefix.clone()
                        } else {
                            prefix
                                .iter()
                                .map(|c| StyledGlyph::from_char(' ', c.style))
                                .collect()
                        };
                        ([lead, row].concat(), choice)
                    })
                })
                .collect()
        } else {
            entries
                .into_iter()
                .map(|(prefix, body, choice)| ([prefix, body].concat(), choice))
                .collect()
        };

        let content_height = lines.len();
        let content_width = lines
            .iter()
            .map(|(chars, _)| width_of(chars))
            .max()
            .unwrap_or(0);
        let mut body = Body {
            area,
            kind,
            rows: Vec::new(),
            content_width,
            content_height,
            offset_x: 0,
            offset_y: 0,
            choice_rows: lines.iter().map(|(_, choice)| *choice).collect(),
        };
        if overflow != Overflow::Truncate {
            body.offset_y = offset_for_percent(spec.vertical_scroll, body.max_offset_y());
        }
        if overflow == Overflow::Scroll && kind != Kind::Terminal {
            body.offset_x = offset_for_percent(spec.horizontal_scroll, body.max_offset_x());
        }

        // Text that fits is centered in each direction that has room, unless it is aligned to the
        // top-left corner.
        let centered =
            style.content_align == ContentAlign::Center && matches!(spec.content, Content::Text(_));
        let (pad_x, pad_y) = if centered {
            (
                area.width().saturating_sub(content_width) / 2,
                area.height().saturating_sub(content_height) / 2,
            )
        } else {
            (0, 0)
        };

        for (i, (chars, choice)) in lines
            .iter()
            .skip(body.offset_y)
            .take(area.height() - pad_y)
            .enumerate()
        {
            let visible = match overflow {
                Overflow::Truncate => truncate(chars, area.width()),
                Overflow::Scroll | Overflow::Wrap => {
                    window(chars, body.offset_x, area.width() - pad_x)
                }
            };
            body.rows.push(Row {
                y: area.top() + pad_y + i,
                x: area.left() + pad_x,
                chars: visible,
                choice: *choice,
            });
        }

        let scrollable = overflow != Overflow::Truncate;
        if scrollable && body.max_offset_y() > 0 {
            self.vertical_scrollbar = Some(Scrollbar {
                start: inner.top(),
                track: inner.height(),
                content: content_height,
                view: area.height(),
                offset: body.offset_y,
            });
        }
        if overflow == Overflow::Scroll && kind != Kind::Terminal && body.max_offset_x() > 0 {
            self.horizontal_scrollbar = Some(Scrollbar {
                start: inner.left(),
                track: inner.width(),
                content: content_width,
                view: area.width(),
                offset: body.offset_x,
            });
        }
        self.body = Some(body);
    }

    /// The first visible tab and the last one it can be, when the box has tabs.
    pub fn tab_offsets(&self) -> Option<(usize, usize)> {
        match &self.top {
            Top::Tabs { bar, .. } => Some((bar.offset, bar.max_offset)),
            _ => None,
        }
    }

    /// What is at cell (`x`, `y`); `None` outside the box.
    pub fn hit(&self, x: usize, y: usize) -> Option<BoxHit> {
        if !self.outer.contains_point(x, y) {
            return None;
        }
        if self.resizable && x == self.outer.right() && y == self.outer.bottom() {
            return Some(BoxHit::ResizeHandle);
        }
        if self.has_border && y == self.outer.top() {
            if let Top::Tabs { bar, .. } = &self.top {
                if let Some(hit) = bar.hit(x) {
                    return Some(BoxHit::Tab(hit));
                }
            }
            return Some(BoxHit::TopBorder);
        }
        if let Some(bar) = &self.vertical_scrollbar {
            if x == self.outer.right() {
                if let Some(hit) = bar.hit(y) {
                    return Some(BoxHit::VerticalScrollbar(hit));
                }
            }
        }
        if let Some(bar) = &self.horizontal_scrollbar {
            if y == self.outer.bottom() {
                if let Some(hit) = bar.hit(x) {
                    return Some(BoxHit::HorizontalScrollbar(hit));
                }
            }
        }
        if let Some(body) = &self.body {
            if body.kind == Kind::Menu {
                let on_choice = body.rows.iter().find_map(|row| {
                    let span = row.x..row.x + width_of(&row.chars);
                    (row.y == y && span.contains(&x))
                        .then_some(row.choice)
                        .flatten()
                });
                if let Some(choice) = on_choice {
                    return Some(BoxHit::Choice(choice));
                }
            } else if body.area.contains_point(x, y) {
                return Some(BoxHit::Content {
                    column: x - body.area.left(),
                    row: y - body.area.top(),
                });
            }
        }
        Some(BoxHit::Other)
    }

    /// Draws the box. `hover` is what the mouse pointer is on in this box.
    pub fn draw(&self, buffer: &mut ScreenBuffer, hover: Option<BoxHit>) {
        let o = self.outer;
        let style = self.style;
        let limit = o.right();
        if style.filled {
            buffer.fill(
                o.left(),
                o.top(),
                o.right(),
                o.bottom(),
                style.fill_char,
                style.body,
            );
        }
        if self.has_border {
            self.draw_border(buffer);
        }
        if let Some(body) = &self.body {
            for row in &body.rows {
                let mut x = row.x;
                let hovered = matches!(hover, Some(BoxHit::Choice(c)) if Some(c) == row.choice);
                for c in &row.chars {
                    let char_style = if hovered { style.menu_hovered } else { c.style };
                    x += buffer.put(
                        x,
                        row.y,
                        &c.glyph,
                        char_style,
                        c.link.as_ref(),
                        body.area.right(),
                    );
                }
            }
        }
        match &self.top {
            Top::Tabs { bar, active } => {
                let hovered = match hover {
                    Some(BoxHit::Tab(hit)) => Some(hit),
                    _ => None,
                };
                bar.draw(
                    buffer,
                    *active,
                    &TabStyles {
                        active: style.active_tab,
                        inactive: style.title,
                        hovered: style.menu_hovered,
                    },
                    hovered,
                    limit - 1,
                );
            }
            Top::Title { x, text } => {
                buffer.put_str(*x, o.top(), text, style.title, limit - 1);
            }
            Top::Border => {}
        }
        if let Some(bar) = &self.vertical_scrollbar {
            let (position, length) = bar.knob();
            for i in 0..bar.track {
                let knob = (position..position + length).contains(&i);
                let (ch, cell_style) = if knob {
                    ('█', style.scrollbar_knob)
                } else {
                    ('│', style.scrollbar_track)
                };
                buffer.put_char(o.right(), bar.start + i, ch, cell_style, limit);
            }
        }
        if let Some(bar) = &self.horizontal_scrollbar {
            let (position, length) = bar.knob();
            for i in 0..bar.track {
                let knob = (position..position + length).contains(&i);
                let (ch, cell_style) = if knob {
                    ('■', style.scrollbar_knob)
                } else {
                    ('─', style.scrollbar_track)
                };
                buffer.put_char(bar.start + i, o.bottom(), ch, cell_style, limit);
            }
        }
    }

    fn draw_border(&self, buffer: &mut ScreenBuffer) {
        let o = self.outer;
        let border = self.style.border;
        let limit = o.right();
        for x in o.left() + 1..o.right() {
            buffer.put_char(x, o.top(), '─', border, limit);
            buffer.put_char(x, o.bottom(), '─', border, limit);
        }
        for y in o.top() + 1..o.bottom() {
            buffer.put_char(o.left(), y, '│', border, limit);
            buffer.put_char(o.right(), y, '│', border, limit);
        }
        buffer.put_char(o.left(), o.top(), '┌', border, limit);
        buffer.put_char(o.right(), o.top(), '┐', border, limit);
        buffer.put_char(o.left(), o.bottom(), '└', border, limit);
        let corner = if self.resizable { '⋱' } else { '┘' };
        buffer.put_char(o.right(), o.bottom(), corner, border, limit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::layout::Layout;
    use crate::model::muxbox::MuxBox;
    use crate::render::theme::{BoxState, Scope};

    fn style(overflow: Overflow) -> BoxStyle {
        let layout = Layout {
            overflow_behavior: Some(overflow),
            ..Layout::default()
        };
        BoxStyle::resolve(
            &Scope {
                muxbox: &MuxBox::default(),
                ancestors: &[],
                layout: &layout,
            },
            BoxState::default(),
        )
    }

    fn view(bounds: Bounds, overflow: Overflow, content: Content, scroll: (f64, f64)) -> BoxView {
        BoxView::new(BoxSpec {
            id: "b",
            bounds,
            style: style(overflow),
            header: Header::Title("Box"),
            content,
            selected_choice: None,
            horizontal_scroll: scroll.0,
            vertical_scroll: scroll.1,
            resizable: true,
        })
    }

    fn render(view: &BoxView, width: usize, height: usize) -> Vec<String> {
        let mut buffer = ScreenBuffer::new(width, height, Style::default());
        view.draw(&mut buffer, None);
        buffer.text_rows()
    }

    fn lines(text: &[&str]) -> Vec<String> {
        text.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn content_sits_inside_the_border_with_a_one_column_margin() {
        let content = lines(&["0123456789"]);
        let v = view(
            Bounds::new(0, 0, 13, 2),
            Overflow::Scroll,
            Content::Text(&content),
            (0.0, 0.0),
        );
        let rows = render(&v, 14, 3);
        assert_eq!(rows[0], "┌─── Box ────┐");
        assert_eq!(rows[1], "│ 0123456789 │");
        assert_eq!(rows[2], "└────────────⋱");
    }

    /// The link URI of each cell of row `y` from column `from` to `to`.
    fn link_uris(
        view: &BoxView,
        size: (usize, usize),
        y: usize,
        from: usize,
        to: usize,
    ) -> Vec<Option<String>> {
        let mut buffer = ScreenBuffer::new(size.0, size.1, Style::default());
        view.draw(&mut buffer, None);
        (from..=to)
            .map(|x| {
                let cell = buffer.get(x, y).unwrap();
                cell.link.as_ref().map(|link| link.uri().to_string())
            })
            .collect()
    }

    #[test]
    fn a_link_cut_at_the_box_s_edge_keeps_its_cut_part_in_the_link() {
        let content = lines(&["ab\x1b]8;;https://e.com\x1b\\日本xy\x1b]8;;\x1b\\cd"]);
        let e = Some("https://e.com".to_string());
        // Truncated: the ellipsis stands for linked text, so it is part of the link.
        let v = view(
            Bounds::new(0, 0, 9, 2),
            Overflow::Truncate,
            Content::Text(&content),
            (0.0, 0.0),
        );
        assert_eq!(render(&v, 10, 3)[1], "│ ab日…  │");
        assert_eq!(
            link_uris(&v, (10, 3), 1, 2, 7),
            vec![None, None, e.clone(), e.clone(), e.clone(), None]
        );
        // Scrolled so that 日 straddles the left edge: its visible half is a linked blank.
        let v = view(
            Bounds::new(0, 0, 9, 2),
            Overflow::Scroll,
            Content::Text(&content),
            (75.0, 0.0),
        );
        assert_eq!(render(&v, 10, 3)[1], "│  本xyc │");
        assert_eq!(
            link_uris(&v, (10, 3), 1, 2, 7),
            vec![e.clone(), e.clone(), e.clone(), e.clone(), e, None]
        );
    }

    #[test]
    fn text_that_fits_is_centered() {
        let content = lines(&["ab"]);
        let v = view(
            Bounds::new(0, 0, 9, 4),
            Overflow::Scroll,
            Content::Text(&content),
            (0.0, 0.0),
        );
        let rows = render(&v, 10, 5);
        assert_eq!(rows[2], "│   ab   │");
    }

    #[test]
    fn scrolled_text_shows_the_scrolled_window_and_scrollbars() {
        let content: Vec<String> = (1..=10)
            .map(|i| format!("line {:02} and more text", i))
            .collect();
        let v = view(
            Bounds::new(0, 0, 11, 4),
            Overflow::Scroll,
            Content::Text(&content),
            (0.0, 100.0),
        );
        let body = v.body.as_ref().unwrap();
        assert_eq!((body.area.width(), body.area.height()), (8, 3));
        assert_eq!(body.offset_y, 7);
        let rows = render(&v, 12, 5);
        assert!(rows[1].starts_with("│ line 08"), "{:?}", rows);
        assert_eq!(
            rows[3].chars().nth(11),
            Some('█'),
            "the knob is at the bottom: {:?}",
            rows
        );
        assert!(rows[4].contains('■'), "horizontal knob: {:?}", rows);
        assert!(v.horizontal_scrollbar.is_some() && v.vertical_scrollbar.is_some());
    }

    #[test]
    fn wrapped_text_breaks_lines_and_only_scrolls_vertically() {
        let content = lines(&["one two three four"]);
        let v = view(
            Bounds::new(0, 0, 11, 6),
            Overflow::Wrap,
            Content::Text(&content),
            (0.0, 0.0),
        );
        let rows = render(&v, 12, 7);
        let text: Vec<String> = rows[1..6]
            .iter()
            .map(|r| r[3..11].trim().to_string())
            .collect();
        assert!(text.contains(&"one two".to_string()), "{:?}", rows);
        assert!(text.contains(&"three".to_string()) && text.contains(&"four".to_string()));
        assert!(v.horizontal_scrollbar.is_none());
    }

    #[test]
    fn truncated_text_ends_with_an_ellipsis_and_has_no_scrollbars() {
        let content = lines(&["abcdefghijklmnop", "x", "y", "z", "w"]);
        let v = view(
            Bounds::new(0, 0, 9, 3),
            Overflow::Truncate,
            Content::Text(&content),
            (0.0, 50.0),
        );
        let rows = render(&v, 10, 4);
        assert_eq!(rows[1], "│ abcde… │");
        assert_eq!(rows[2], "│ x      │");
        assert!(v.vertical_scrollbar.is_none() && v.horizontal_scrollbar.is_none());
    }

    #[test]
    fn colored_and_wide_text_keeps_columns_aligned() {
        let content = lines(&["\x1b[31m日本\x1b[0mab"]);
        let v = view(
            Bounds::new(0, 0, 9, 2),
            Overflow::Scroll,
            Content::Text(&content),
            (0.0, 0.0),
        );
        let rows = render(&v, 10, 3);
        assert_eq!(rows[1], "│ 日本ab │");
        // Scrolled by one column, the first wide character is cut in half and shows as a space.
        let content = lines(&["日本語の文章です"]);
        let v = view(
            Bounds::new(0, 0, 9, 2),
            Overflow::Scroll,
            Content::Text(&content),
            (12.5, 0.0),
        );
        let body = v.body.as_ref().unwrap();
        assert_eq!(body.offset_x, 1);
        let rows = render(&v, 10, 3);
        assert_eq!(
            rows[1], "│  本語  │",
            "half-visible characters become spaces"
        );
    }

    #[test]
    fn clicks_find_choices_only_on_their_labels() {
        let choices = vec![
            Choice {
                id: "a".into(),
                content: Some("Alpha".into()),
                ..Choice::default()
            },
            Choice {
                id: "b".into(),
                content: Some("Bravo".into()),
                ..Choice::default()
            },
        ];
        let v = BoxView::new(BoxSpec {
            id: "m",
            bounds: Bounds::new(0, 0, 19, 4),
            style: style(Overflow::Scroll),
            header: Header::None,
            content: Content::Menu(&choices),
            selected_choice: Some(1),
            horizontal_scroll: 0.0,
            vertical_scroll: 0.0,
            resizable: false,
        });
        let rows = render(&v, 20, 5);
        assert!(rows[1].starts_with("│   Alpha "), "{:?}", rows);
        assert!(rows[2].contains("► Bravo"), "{:?}", rows);
        assert_eq!(v.hit(4, 1), Some(BoxHit::Choice(0)));
        assert_eq!(v.hit(8, 1), Some(BoxHit::Choice(0)));
        assert_eq!(v.hit(9, 1), Some(BoxHit::Other), "past the label");
        assert_eq!(
            v.hit(2, 2),
            Some(BoxHit::Choice(1)),
            "the marker belongs to the entry"
        );
        assert_eq!(v.hit(19, 4), Some(BoxHit::Other), "not resizable");
    }

    #[test]
    fn wrapped_menu_entries_continue_under_their_label() {
        let choices = vec![Choice {
            id: "a".into(),
            content: Some("one two three four".into()),
            ..Choice::default()
        }];
        let v = BoxView::new(BoxSpec {
            id: "m",
            bounds: Bounds::new(0, 0, 13, 4),
            style: style(Overflow::Wrap),
            header: Header::None,
            content: Content::Menu(&choices),
            selected_choice: Some(0),
            horizontal_scroll: 0.0,
            vertical_scroll: 0.0,
            resizable: false,
        });
        let rows = render(&v, 14, 5);
        assert_eq!(rows[1], "│ ► one two  │");
        assert_eq!(rows[2], "│   three    │");
        assert_eq!(rows[3], "│   four     │");
        // Every row of the entry is part of it.
        assert_eq!(v.hit(4, 3), Some(BoxHit::Choice(0)));
    }

    #[test]
    fn scrollbars_and_corners_are_hit_targets() {
        let content: Vec<String> = (0..20).map(|i| format!("{}", i)).collect();
        let v = view(
            Bounds::new(0, 0, 9, 6),
            Overflow::Scroll,
            Content::Text(&content),
            (0.0, 0.0),
        );
        assert_eq!(v.hit(9, 6), Some(BoxHit::ResizeHandle));
        assert_eq!(
            v.hit(9, 1),
            Some(BoxHit::VerticalScrollbar(ScrollbarHit::Knob))
        );
        assert_eq!(
            v.hit(9, 5),
            Some(BoxHit::VerticalScrollbar(ScrollbarHit::Track))
        );
        assert_eq!(v.hit(4, 0), Some(BoxHit::TopBorder));
        assert_eq!(v.hit(3, 2), Some(BoxHit::Content { column: 1, row: 1 }));
        assert_eq!(v.hit(10, 2), None);
    }

    #[test]
    fn a_terminal_fills_the_whole_inside_of_the_border() {
        let screen = lines(&["$ ls", "file"]);
        let v = view(
            Bounds::new(0, 0, 9, 3),
            Overflow::Wrap,
            Content::Terminal(&screen),
            (0.0, 0.0),
        );
        let body = v.body.as_ref().unwrap();
        assert_eq!(body.area, Bounds::new(1, 1, 8, 2));
        let rows = render(&v, 10, 4);
        assert_eq!(rows[1], "│$ ls    │");
        assert_eq!(v.hit(1, 2), Some(BoxHit::Content { column: 0, row: 1 }));
    }

    #[test]
    fn tiny_boxes_draw_what_fits_and_nothing_outside() {
        let content = lines(&["text"]);
        for (w, h) in [(1, 1), (2, 2), (3, 2), (2, 3), (3, 3)] {
            let v = view(
                Bounds::new(1, 1, w, h),
                Overflow::Scroll,
                Content::Text(&content),
                (0.0, 0.0),
            );
            let rows = render(&v, 6, 6);
            assert!(rows[0].trim().is_empty(), "{}x{}: {:?}", w, h, rows);
            for row in &rows[h + 1..] {
                assert!(row.trim().is_empty(), "{}x{}: {:?}", w, h, rows);
            }
            for row in &rows {
                assert!(
                    row.chars().skip(w + 1).all(|c| c == ' '),
                    "{}x{}: {:?}",
                    w,
                    h,
                    rows
                );
            }
        }
    }

    #[test]
    fn titles_follow_their_position_setting() {
        let mut s = style(Overflow::Scroll);
        for (position, expected) in [
            (TitlePosition::Left, "┌─ Title ──────────┐"),
            (TitlePosition::Center, "┌───── Title ──────┐"),
            (TitlePosition::Right, "┌────────── Title ─┐"),
        ] {
            s.title_position = position;
            let v = BoxView::new(BoxSpec {
                id: "t",
                bounds: Bounds::new(0, 0, 19, 2),
                style: s,
                header: Header::Title("Title"),
                content: Content::Nothing,
                selected_choice: None,
                horizontal_scroll: 0.0,
                vertical_scroll: 0.0,
                resizable: false,
            });
            assert_eq!(render(&v, 20, 3)[0], expected);
        }
    }

    /// A box of `bounds` with `padding` and `align`, showing `content` under `header`.
    fn padded(
        bounds: Bounds,
        padding: Sides,
        align: ContentAlign,
        header: Header,
        content: Content,
    ) -> BoxView {
        let mut s = style(Overflow::Scroll);
        s.padding = padding;
        s.content_align = align;
        BoxView::new(BoxSpec {
            id: "p",
            bounds,
            style: s,
            header,
            content,
            selected_choice: Some(0),
            horizontal_scroll: 0.0,
            vertical_scroll: 0.0,
            resizable: false,
        })
    }

    #[test]
    fn text_aligned_top_left_with_no_top_padding_starts_on_the_row_under_the_border() {
        let content = lines(&["ab"]);
        let top_left = padded(
            Bounds::new(0, 0, 9, 4),
            Sides {
                top: 0,
                ..Sides::DEFAULT
            },
            ContentAlign::TopLeft,
            Header::Title("Box"),
            Content::Text(&content),
        );
        let rows = render(&top_left, 10, 5);
        assert_eq!(rows[0], "┌─ Box ──┐");
        assert_eq!(rows[1], "│ ab     │", "{:?}", rows);
        assert_eq!(rows[2], "│        │", "{:?}", rows);
        // With a row of padding it starts on the row below that.
        let below = padded(
            Bounds::new(0, 0, 9, 4),
            Sides {
                top: 1,
                left: 0,
                ..Sides::DEFAULT
            },
            ContentAlign::TopLeft,
            Header::Title("Box"),
            Content::Text(&content),
        );
        let rows = render(&below, 10, 5);
        assert_eq!(rows[1], "│        │", "{:?}", rows);
        assert_eq!(rows[2], "│ab      │", "{:?}", rows);
    }

    #[test]
    fn the_default_padding_and_alignment_center_text_inside_a_one_column_margin() {
        let content = lines(&["ab"]);
        let v = padded(
            Bounds::new(0, 0, 9, 4),
            Sides::DEFAULT,
            ContentAlign::Center,
            Header::Title("Box"),
            Content::Text(&content),
        );
        assert_eq!(v.body.as_ref().unwrap().area, Bounds::new(2, 1, 7, 3));
        let rows = render(&v, 10, 5);
        assert_eq!(rows[1], "│        │", "{:?}", rows);
        assert_eq!(rows[2], "│   ab   │", "{:?}", rows);
    }

    #[test]
    fn centered_text_is_centered_in_the_padded_area() {
        let content = lines(&["ab"]);
        let v = padded(
            Bounds::new(0, 0, 11, 6),
            Sides {
                top: 2,
                bottom: 0,
                left: 4,
                right: 0,
            },
            ContentAlign::Center,
            Header::None,
            Content::Text(&content),
        );
        // The padded area is columns 5 to 10 and rows 3 to 5.
        assert_eq!(v.body.as_ref().unwrap().area, Bounds::new(5, 3, 10, 5));
        let rows = render(&v, 12, 7);
        assert_eq!(rows[4], "│      ab  │", "{:?}", rows);
    }

    #[test]
    fn menus_start_at_the_padded_corner_and_their_entries_are_hit_there() {
        let choices = vec![
            Choice {
                id: "a".into(),
                content: Some("Alpha".into()),
                ..Choice::default()
            },
            Choice {
                id: "b".into(),
                content: Some("Bravo".into()),
                ..Choice::default()
            },
        ];
        let v = padded(
            Bounds::new(0, 0, 13, 5),
            Sides {
                top: 1,
                bottom: 0,
                left: 2,
                right: 1,
            },
            ContentAlign::TopLeft,
            Header::None,
            Content::Menu(&choices),
        );
        let rows = render(&v, 14, 6);
        assert_eq!(rows[1], "│            │", "{:?}", rows);
        assert_eq!(rows[2], "│  ► Alpha   │", "{:?}", rows);
        assert_eq!(rows[3], "│    Bravo   │", "{:?}", rows);
        assert_eq!(v.hit(3, 2), Some(BoxHit::Choice(0)), "the marker");
        assert_eq!(v.hit(5, 3), Some(BoxHit::Choice(1)));
        assert_eq!(v.hit(5, 1), Some(BoxHit::Other), "the padding above");
        assert_eq!(v.hit(2, 2), Some(BoxHit::Other), "the padding beside");
    }

    #[test]
    fn content_hits_are_counted_from_the_padded_corner_and_padding_is_not_content() {
        let content = lines(&["one", "two"]);
        let v = padded(
            Bounds::new(0, 0, 9, 5),
            Sides {
                top: 1,
                bottom: 1,
                left: 2,
                right: 0,
            },
            ContentAlign::TopLeft,
            Header::None,
            Content::Text(&content),
        );
        assert_eq!(v.hit(3, 2), Some(BoxHit::Content { column: 0, row: 0 }));
        assert_eq!(v.hit(4, 3), Some(BoxHit::Content { column: 1, row: 1 }));
        assert_eq!(v.hit(3, 1), Some(BoxHit::Other), "top padding");
        assert_eq!(v.hit(3, 4), Some(BoxHit::Other), "bottom padding");
        assert_eq!(v.hit(2, 2), Some(BoxHit::Other), "left padding");
    }

    #[test]
    fn scrolling_measures_the_padded_view_and_the_scrollbars_stay_on_the_border() {
        let content: Vec<String> = (0..10).map(|i| format!("line {} of the text", i)).collect();
        let v = padded(
            Bounds::new(0, 0, 11, 6),
            Sides {
                top: 1,
                bottom: 1,
                left: 1,
                right: 1,
            },
            ContentAlign::TopLeft,
            Header::None,
            Content::Text(&content),
        );
        let body = v.body.as_ref().unwrap();
        // Five rows inside the border, three of them for the text; eight columns, six for it.
        assert_eq!((body.area.width(), body.area.height()), (8, 3));
        assert_eq!(body.max_offset_y(), 7);
        assert_eq!(body.max_offset_x(), 10);
        let vertical = v.vertical_scrollbar.unwrap();
        assert_eq!((vertical.start, vertical.track), (1, 5));
        assert_eq!((vertical.content, vertical.view), (10, 3));
        let horizontal = v.horizontal_scrollbar.unwrap();
        assert_eq!((horizontal.start, horizontal.track), (1, 10));
        assert_eq!((horizontal.content, horizontal.view), (18, 8));
        let rows = render(&v, 12, 7);
        let start = |row: &str| row.chars().take(11).collect::<String>();
        assert_eq!(start(&rows[1]), "│          ", "{:?}", rows);
        assert_eq!(start(&rows[2]), "│ line 0 o ", "{:?}", rows);
        assert_eq!(start(&rows[4]), "│ line 2 o ", "{:?}", rows);
        assert_eq!(start(&rows[5]), "│          ", "{:?}", rows);
        assert_eq!(rows[6].chars().nth(1), Some('■'), "{:?}", rows);
    }

    #[test]
    fn a_tab_s_content_takes_the_padding_and_the_tabs_stay_in_the_border() {
        let content = lines(&["out"]);
        let v = padded(
            Bounds::new(0, 0, 19, 3),
            Sides {
                top: 0,
                bottom: 0,
                left: 0,
                right: 0,
            },
            ContentAlign::TopLeft,
            Header::Tabs {
                tabs: vec![
                    TabSpec {
                        label: "One".into(),
                        closeable: false,
                    },
                    TabSpec {
                        label: "Two".into(),
                        closeable: false,
                    },
                ],
                active: 1,
                offset: 0,
            },
            Content::Text(&content),
        );
        let rows = render(&v, 20, 4);
        assert!(
            rows[0].contains("One") && rows[0].contains("Two"),
            "{:?}",
            rows
        );
        assert_eq!(rows[1], "│out               │", "{:?}", rows);
        assert!(matches!(
            v.hit(rows[0].chars().position(|c| c == 'T').unwrap(), 0),
            Some(BoxHit::Tab(_))
        ));
    }

    #[test]
    fn a_terminal_fills_the_inside_of_the_border_whatever_the_padding() {
        let screen = lines(&["$ ls"]);
        let v = padded(
            Bounds::new(0, 0, 9, 3),
            Sides {
                top: 1,
                bottom: 1,
                left: 2,
                right: 2,
            },
            ContentAlign::Center,
            Header::None,
            Content::Terminal(&screen),
        );
        assert_eq!(v.body.as_ref().unwrap().area, Bounds::new(1, 1, 8, 2));
        assert_eq!(render(&v, 10, 4)[1], "│$ ls    │");
    }

    #[test]
    fn padding_larger_than_the_inside_leaves_no_content_area() {
        let content = lines(&["ab"]);
        let v = padded(
            Bounds::new(0, 0, 9, 3),
            Sides {
                top: 1,
                bottom: 1,
                ..Sides::DEFAULT
            },
            ContentAlign::Center,
            Header::Title("Box"),
            Content::Text(&content),
        );
        assert!(v.body.is_none());
        let rows = render(&v, 10, 4);
        assert_eq!(rows[1], "│        │");
        assert_eq!(rows[2], "│        │");
        assert_eq!(
            content_area(&Bounds::new(0, 0, 9, 3), Sides::DEFAULT),
            Some(Bounds::new(2, 1, 7, 2))
        );
        assert_eq!(
            content_area(
                &Bounds::new(0, 0, 9, 3),
                Sides {
                    top: 1,
                    bottom: 1,
                    ..Sides::DEFAULT
                }
            ),
            None
        );
    }

    #[test]
    fn scroll_percentages_and_offsets_convert_both_ways() {
        for max in [1, 3, 7, 100] {
            for offset in 0..=max {
                assert_eq!(
                    offset_for_percent(percent_for_offset(offset, max), max),
                    offset
                );
            }
        }
    }
}
