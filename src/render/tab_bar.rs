//! The tabs in a box's top border: where each tab, close button and scroll arrow is, so the
//! same layout is used for drawing and for finding what the mouse is on.

use super::screen::{ScreenBuffer, Style};
use super::styled::styled_glyphs;
use crate::width::{ellipsize, text_width};

/// Columns of a tab label before it is cut off with `…`.
const MAX_LABEL_WIDTH: usize = 16;
const CLOSE: &str = "×";
const LEFT_ARROW: char = '◀';
const RIGHT_ARROW: char = '▶';

/// One tab as the box shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabSpec {
    pub label: String,
    pub closeable: bool,
}

/// What a cell of the tab bar is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TabHit {
    Tab(usize),
    Close(usize),
    ScrollLeft,
    ScrollRight,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlacedTab {
    index: usize,
    x: usize,
    /// The text drawn for the tab: ` label ` or ` label × `.
    text: String,
    width: usize,
    /// Column of the close button.
    close_x: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabBar {
    y: usize,
    left_arrow: Option<usize>,
    right_arrow: Option<usize>,
    tabs: Vec<PlacedTab>,
    /// The first visible tab, after limiting the requested offset.
    pub offset: usize,
    /// The largest offset that still shows tabs: the one where the last tab just fits.
    pub max_offset: usize,
}

/// What a tab shows: ` label ` or ` label × `, its label without colors and cut to
/// [`MAX_LABEL_WIDTH`] columns.
fn tab_text(spec: &TabSpec) -> String {
    let plain: String = styled_glyphs(&spec.label, Style::default())
        .iter()
        .map(|g| g.glyph.to_string())
        .collect();
    let label = ellipsize(&plain, MAX_LABEL_WIDTH);
    if spec.closeable {
        format!(" {} {} ", label, CLOSE)
    } else {
        format!(" {} ", label)
    }
}

impl TabBar {
    /// Lays out `tabs` on row `y` between columns `x1` and `x2` (the box's top corners),
    /// starting at tab `offset`. Tabs that fit are centered; otherwise arrows at both ends
    /// scroll them.
    pub fn layout(y: usize, x1: usize, x2: usize, tabs: &[TabSpec], offset: usize) -> TabBar {
        let texts: Vec<String> = tabs.iter().map(tab_text).collect();
        let widths: Vec<usize> = texts.iter().map(|t| text_width(t)).collect();
        // One border cell stays visible next to each corner.
        let first = x1 + 2;
        let last = x2.saturating_sub(2);
        let mut bar = TabBar {
            y,
            left_arrow: None,
            right_arrow: None,
            tabs: Vec::new(),
            offset: 0,
            max_offset: 0,
        };
        if last < first || tabs.is_empty() {
            return bar;
        }
        let space = last - first + 1;
        // Tabs are separated by one border cell.
        let total = widths.iter().sum::<usize>() + widths.len() - 1;
        if total <= space {
            let mut x = first + (space - total) / 2;
            for (index, (text, width)) in texts.into_iter().zip(widths).enumerate() {
                bar.place(index, x, text, width, tabs[index].closeable);
                x += width + 1;
            }
            return bar;
        }

        // Scrolling: an arrow and a border cell at each end.
        if space < 5 {
            return bar;
        }
        let inner_first = first + 2;
        let inner_last = last - 2;
        let inner_space = inner_last - inner_first + 1;
        let fits_from = |start: usize| {
            widths[start..].iter().sum::<usize>() + widths.len() - start - 1 <= inner_space
        };
        bar.max_offset = (0..tabs.len())
            .find(|&s| fits_from(s))
            .unwrap_or(tabs.len() - 1);
        bar.offset = offset.min(bar.max_offset);
        let mut x = inner_first;
        for index in bar.offset..tabs.len() {
            let width = widths[index];
            if x + width - 1 > inner_last {
                break;
            }
            bar.place(index, x, texts[index].clone(), width, tabs[index].closeable);
            x += width + 1;
        }
        if bar.offset > 0 {
            bar.left_arrow = Some(first);
        }
        if bar.offset < bar.max_offset {
            bar.right_arrow = Some(last);
        }
        bar
    }

    fn place(&mut self, index: usize, x: usize, text: String, width: usize, closeable: bool) {
        // The close button is the second to last cell: ` label × `.
        let close_x = closeable.then(|| x + width - 2);
        self.tabs.push(PlacedTab {
            index,
            x,
            text,
            width,
            close_x,
        });
    }

    /// What is at column `x` of the bar's row.
    pub fn hit(&self, x: usize) -> Option<TabHit> {
        if self.left_arrow == Some(x) {
            return Some(TabHit::ScrollLeft);
        }
        if self.right_arrow == Some(x) {
            return Some(TabHit::ScrollRight);
        }
        self.tabs.iter().find_map(|tab| {
            if tab.close_x == Some(x) {
                Some(TabHit::Close(tab.index))
            } else if (tab.x..tab.x + tab.width).contains(&x) {
                Some(TabHit::Tab(tab.index))
            } else {
                None
            }
        })
    }

    /// Indexes of the tabs that are shown.
    pub fn visible_tabs(&self) -> Vec<usize> {
        self.tabs.iter().map(|t| t.index).collect()
    }

    /// Draws the bar over the border row. `hovered` is what the pointer is on.
    pub fn draw(
        &self,
        buffer: &mut ScreenBuffer,
        active: usize,
        styles: &TabStyles,
        hovered: Option<TabHit>,
        limit: usize,
    ) {
        for tab in &self.tabs {
            let mut style = if tab.index == active {
                styles.active
            } else {
                styles.inactive
            };
            if hovered == Some(TabHit::Tab(tab.index)) {
                style = styles.hovered;
            }
            buffer.put_str(tab.x, self.y, &tab.text, style, limit);
            if let Some(close_x) = tab.close_x {
                let close_style = if hovered == Some(TabHit::Close(tab.index)) {
                    styles.hovered
                } else {
                    style
                };
                buffer.put_str(close_x, self.y, CLOSE, close_style, limit);
            }
        }
        for (arrow, x, hit) in [
            (LEFT_ARROW, self.left_arrow, TabHit::ScrollLeft),
            (RIGHT_ARROW, self.right_arrow, TabHit::ScrollRight),
        ] {
            if let Some(x) = x {
                let style = if hovered == Some(hit) {
                    styles.hovered
                } else {
                    styles.inactive
                };
                buffer.put_char(x, self.y, arrow, style, limit);
            }
        }
    }
}

pub struct TabStyles {
    pub active: Style,
    pub inactive: Style,
    pub hovered: Style,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tabs(labels: &[&str], closeable: bool) -> Vec<TabSpec> {
        labels
            .iter()
            .map(|label| TabSpec {
                label: label.to_string(),
                closeable,
            })
            .collect()
    }

    #[test]
    fn tabs_that_fit_are_centered_and_each_cell_maps_to_its_tab() {
        let bar = TabBar::layout(0, 0, 39, &tabs(&["one", "two"], false), 0);
        // " one " and " two " with a separator: 11 columns centered in columns 2..=37.
        assert_eq!(bar.visible_tabs(), vec![0, 1]);
        assert_eq!(bar.hit(14), Some(TabHit::Tab(0)));
        assert_eq!(bar.hit(18), Some(TabHit::Tab(0)));
        assert_eq!(bar.hit(19), None, "the separator belongs to no tab");
        assert_eq!(bar.hit(20), Some(TabHit::Tab(1)));
        assert_eq!(bar.hit(2), None);
        assert_eq!(bar.max_offset, 0);
    }

    #[test]
    fn the_close_button_is_its_own_target() {
        let bar = TabBar::layout(0, 0, 39, &tabs(&["log"], true), 0);
        // " log × " is 7 columns, centered at 16..=22.
        assert_eq!(bar.hit(16), Some(TabHit::Tab(0)));
        assert_eq!(bar.hit(21), Some(TabHit::Close(0)));
        assert_eq!(bar.hit(22), Some(TabHit::Tab(0)));
    }

    #[test]
    fn every_tab_can_be_scrolled_into_view_and_no_further() {
        let specs = tabs(&["alpha", "bravo", "charlie", "delta", "echo"], false);
        let bar = TabBar::layout(0, 0, 29, &specs, 0);
        assert!(bar.max_offset > 0);
        assert_eq!(bar.hit(27), Some(TabHit::ScrollRight));
        assert_eq!(bar.hit(2), None, "no left arrow at the start");

        let end = TabBar::layout(0, 0, 29, &specs, 99);
        assert_eq!(end.offset, end.max_offset, "the offset is limited");
        assert!(end.visible_tabs().contains(&4), "the last tab is reachable");
        assert_eq!(end.hit(2), Some(TabHit::ScrollLeft));
        assert_eq!(end.hit(27), None, "no right arrow at the end");
    }

    #[test]
    fn long_labels_are_cut_off() {
        let specs = tabs(&["a very long tab label indeed"], false);
        let bar = TabBar::layout(0, 0, 79, &specs, 0);
        assert_eq!(bar.tabs[0].text, " a very long tab… ");
    }

    #[test]
    fn a_bar_too_narrow_for_anything_shows_nothing() {
        let bar = TabBar::layout(0, 0, 4, &tabs(&["one", "two"], false), 0);
        assert!(bar.visible_tabs().is_empty());
        assert_eq!(bar.hit(2), None);
    }
}
