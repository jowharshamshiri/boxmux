//! Where boxes are: the positions written in configuration, the cells they cover on screen,
//! and the layout pass that turns one into the other.

use crate::model::layout::Layout;
use crate::model::muxbox::MuxBox;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

/// A coordinate of a box edge, relative to the enclosing box (or the screen for top-level
/// boxes). Written in configuration as a percentage with up to two decimals (`"50%"`,
/// `"70.25%"`) or as a cell count (`"12"`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Position {
    /// A percentage of the enclosing extent, in hundredths of a percent (`7025` is 70.25%):
    /// `0` is its first cell, `10000` its last.
    Percent(u16),
    /// A fixed number of cells from the first cell of the enclosing extent. Clamped to the
    /// last cell when the enclosing extent is smaller.
    Cells(u16),
}

/// Hundredths of a percent in 100%.
const FULL_PERCENT: u16 = 10_000;

impl Position {
    /// The percentage position closest to `percent`, at the two-decimal precision positions
    /// are written with.
    pub fn percent(percent: f64) -> Position {
        let hundredths = (percent * 100.0)
            .round()
            .clamp(0.0, f64::from(FULL_PERCENT));
        Position::Percent(hundredths as u16)
    }

    /// The percentage position of cell `offset` within an extent of `extent` cells.
    pub fn percent_of(offset: usize, extent: usize) -> Position {
        let last = extent.saturating_sub(1);
        if last == 0 {
            Position::Percent(0)
        } else {
            Position::percent(offset as f64 / last as f64 * 100.0)
        }
    }

    /// The cell offset this position denotes within an extent of `extent` cells.
    pub fn resolve(&self, extent: usize) -> usize {
        let last = extent.saturating_sub(1);
        match *self {
            Position::Percent(hundredths) => {
                (f64::from(hundredths) / f64::from(FULL_PERCENT) * last as f64).round() as usize
            }
            Position::Cells(cells) => usize::from(cells).min(last),
        }
    }

    /// The position of cell `offset` within an extent of `extent` cells, in the same unit as
    /// `self`.
    pub fn same_unit(&self, offset: usize, extent: usize) -> Position {
        match self {
            Position::Percent(_) => Position::percent_of(offset, extent),
            Position::Cells(_) => Position::Cells(u16::try_from(offset).unwrap_or(u16::MAX)),
        }
    }
}

impl std::str::FromStr for Position {
    type Err = String;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let invalid = || {
            format!(
                "'{}' is not a position: use a percentage like '50%' or '33.25%', or a cell count like '12'",
                text
            )
        };
        let digits_only = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
        if let Some(percent) = text.strip_suffix('%') {
            let (whole, fraction) = percent.split_once('.').unwrap_or((percent, ""));
            if !digits_only(whole)
                || whole.len() > 3
                || (percent.contains('.') && !digits_only(fraction))
                || fraction.len() > 2
            {
                return Err(invalid());
            }
            let whole: u16 = whole.parse().map_err(|_| invalid())?;
            let fraction: u16 = format!("{:0<2}", fraction).parse().map_err(|_| invalid())?;
            let hundredths = whole * 100 + fraction;
            if hundredths > FULL_PERCENT {
                return Err(format!("'{}' is more than 100%", text));
            }
            Ok(Position::Percent(hundredths))
        } else if digits_only(text) {
            text.parse::<u16>()
                .map(Position::Cells)
                .map_err(|_| format!("'{}' is too many cells", text))
        } else {
            Err(invalid())
        }
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Position::Percent(hundredths) => {
                let (whole, fraction) = (hundredths / 100, hundredths % 100);
                match fraction {
                    0 => write!(f, "{}%", whole),
                    f2 if f2 % 10 == 0 => write!(f, "{}.{}%", whole, f2 / 10),
                    f2 => write!(f, "{}.{:02}%", whole, f2),
                }
            }
            Position::Cells(c) => write!(f, "{}", c),
        }
    }
}

impl Serialize for Position {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Position {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// The edges of a box as written in configuration.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Hash, Eq)]
#[serde(deny_unknown_fields)]
pub struct InputBounds {
    pub x1: Position,
    pub y1: Position,
    pub x2: Position,
    pub y2: Position,
}

impl InputBounds {
    /// Parses four edge positions, e.g. `InputBounds::parse("0%", "0%", "50%", "100%")`.
    pub fn parse(x1: &str, y1: &str, x2: &str, y2: &str) -> Result<Self, String> {
        Ok(InputBounds {
            x1: x1.parse()?,
            y1: y1.parse()?,
            x2: x2.parse()?,
            y2: y2.parse()?,
        })
    }

    /// The cells these positions cover inside `parent`. An edge given before its opposite
    /// edge (`x2` left of `x1`) is swapped, so the result is never inverted.
    pub fn to_bounds(&self, parent: &Bounds) -> Bounds {
        let (width, height) = (parent.width(), parent.height());
        let (x1, x2) = (self.x1.resolve(width), self.x2.resolve(width));
        let (y1, y2) = (self.y1.resolve(height), self.y2.resolve(height));
        Bounds::new(
            parent.x1 + x1.min(x2),
            parent.y1 + y1.min(y2),
            parent.x1 + x1.max(x2),
            parent.y1 + y1.max(y2),
        )
    }

    /// The positions, in the units of `self`, that place a box at `bounds` inside `parent`.
    pub fn same_units_for(&self, bounds: &Bounds, parent: &Bounds) -> InputBounds {
        let (width, height) = (parent.width(), parent.height());
        InputBounds {
            x1: self
                .x1
                .same_unit(bounds.x1.saturating_sub(parent.x1), width),
            y1: self
                .y1
                .same_unit(bounds.y1.saturating_sub(parent.y1), height),
            x2: self
                .x2
                .same_unit(bounds.x2.saturating_sub(parent.x1), width),
            y2: self
                .y2
                .same_unit(bounds.y2.saturating_sub(parent.y1), height),
        }
    }
}

impl Default for InputBounds {
    fn default() -> Self {
        InputBounds {
            x1: Position::Cells(0),
            y1: Position::Cells(0),
            x2: Position::Cells(0),
            y2: Position::Cells(0),
        }
    }
}

/// A rectangle of screen cells. Both corners are included: a box from x 2 to x 5 is four
/// cells wide. `x1 <= x2` and `y1 <= y2` always hold.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bounds {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl Bounds {
    /// The rectangle with corners (`x1`, `y1`) and (`x2`, `y2`), in either order.
    pub fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Bounds {
        Bounds {
            x1: x1.min(x2),
            y1: y1.min(y2),
            x2: x1.max(x2),
            y2: y1.max(y2),
        }
    }

    pub fn width(&self) -> usize {
        self.x2 - self.x1 + 1
    }

    pub fn height(&self) -> usize {
        self.y2 - self.y1 + 1
    }

    pub fn left(&self) -> usize {
        self.x1
    }

    pub fn right(&self) -> usize {
        self.x2
    }

    pub fn top(&self) -> usize {
        self.y1
    }

    pub fn bottom(&self) -> usize {
        self.y2
    }

    pub fn contains_point(&self, x: usize, y: usize) -> bool {
        (self.x1..=self.x2).contains(&x) && (self.y1..=self.y2).contains(&y)
    }

    /// The cells both rectangles cover, if any.
    pub fn intersection(&self, other: &Bounds) -> Option<Bounds> {
        let x1 = self.x1.max(other.x1);
        let y1 = self.y1.max(other.y1);
        let x2 = self.x2.min(other.x2);
        let y2 = self.y2.min(other.y2);
        (x1 <= x2 && y1 <= y2).then_some(Bounds { x1, y1, x2, y2 })
    }

    /// The rectangle `dx` columns and `dy` rows smaller on each side; `None` when nothing is
    /// left.
    pub fn shrink(&self, dx: usize, dy: usize) -> Option<Bounds> {
        (self.width() > 2 * dx && self.height() > 2 * dy).then(|| Bounds {
            x1: self.x1 + dx,
            y1: self.y1 + dy,
            x2: self.x2 - dx,
            y2: self.y2 - dy,
        })
    }
}

impl std::fmt::Display for Bounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {}), ({}, {})", self.x1, self.y1, self.x2, self.y2)
    }
}

/// The point of a box that stays in place when `min_*`/`max_*` change its size.
#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Hash, Eq, Default)]
pub enum Anchor {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    #[default]
    Center,
    CenterTop,
    CenterBottom,
    CenterLeft,
    CenterRight,
}

#[derive(Clone, Copy)]
enum Side {
    Start,
    Middle,
    End,
}

impl Anchor {
    /// The horizontal and vertical side the anchor holds fixed.
    fn sides(self) -> (Side, Side) {
        use Side::*;
        match self {
            Anchor::TopLeft => (Start, Start),
            Anchor::TopRight => (End, Start),
            Anchor::BottomLeft => (Start, End),
            Anchor::BottomRight => (End, End),
            Anchor::Center => (Middle, Middle),
            Anchor::CenterTop => (Middle, Start),
            Anchor::CenterBottom => (Middle, End),
            Anchor::CenterLeft => (Start, Middle),
            Anchor::CenterRight => (End, Middle),
        }
    }
}

/// Resizes the span `start..=end` to `size` cells, keeping `side` in place, then moves it
/// back inside `low..=high` if it left. A span larger than `low..=high` is cut to fit.
fn resize_span(
    start: usize,
    end: usize,
    size: usize,
    side: Side,
    low: usize,
    high: usize,
) -> (usize, usize) {
    let size = size.clamp(1, high - low + 1);
    let current = end - start + 1;
    let new_start = match side {
        Side::Start => start as isize,
        Side::End => end as isize - size as isize + 1,
        Side::Middle => start as isize + (current as isize - size as isize) / 2,
    };
    let new_start = new_start.clamp(low as isize, (high + 1 - size) as isize) as usize;
    (new_start, new_start + size - 1)
}

/// The cells `muxbox` covers inside `parent`: its position, then its `min_*`/`max_*` sizes
/// applied around its anchor, kept inside `parent`.
pub fn box_bounds(muxbox: &MuxBox, parent: &Bounds) -> Bounds {
    let placed = muxbox.position.to_bounds(parent);
    let clamp = |size: usize, min: Option<usize>, max: Option<usize>| {
        let size = min.map_or(size, |min| size.max(min));
        max.map_or(size, |max| size.min(max))
    };
    let width = clamp(placed.width(), muxbox.min_width, muxbox.max_width);
    let height = clamp(placed.height(), muxbox.min_height, muxbox.max_height);
    let (horizontal, vertical) = muxbox.anchor.sides();
    let (x1, x2) = resize_span(
        placed.x1, placed.x2, width, horizontal, parent.x1, parent.x2,
    );
    let (y1, y2) = resize_span(placed.y1, placed.y2, height, vertical, parent.y1, parent.y2);
    Bounds { x1, y1, x2, y2 }
}

/// The bounds of every box of `layout`, nested boxes inside their parent, on a screen
/// covering `screen`.
pub fn layout_bounds(layout: &Layout, screen: &Bounds) -> HashMap<String, Bounds> {
    fn place(muxbox: &MuxBox, parent: &Bounds, out: &mut HashMap<String, Bounds>) {
        let bounds = box_bounds(muxbox, parent);
        out.insert(muxbox.id.clone(), bounds);
        for child in muxbox.children.iter().flatten() {
            place(child, &bounds, out);
        }
    }
    let mut out = HashMap::new();
    for muxbox in layout.children.iter().flatten() {
        place(muxbox, screen, &mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positions_parse_percentages_and_cell_counts_and_reject_everything_else() {
        assert_eq!("50%".parse::<Position>(), Ok(Position::Percent(5000)));
        assert_eq!("70.5%".parse::<Position>(), Ok(Position::Percent(7050)));
        assert_eq!("0.25%".parse::<Position>(), Ok(Position::Percent(25)));
        assert_eq!("100.00%".parse::<Position>(), Ok(Position::Percent(10000)));
        assert_eq!("12".parse::<Position>(), Ok(Position::Cells(12)));
        for invalid in [
            "101%", "100.01%", "-5%", "abc", "", "10px", "5.555%", "5.%", ".5%", "%", "1000%",
            "5.5", "1e2%",
        ] {
            assert!(
                invalid.parse::<Position>().is_err(),
                "accepted {:?}",
                invalid
            );
        }
    }

    #[test]
    fn positions_print_the_way_they_are_written() {
        for text in ["7%", "70.5%", "33.25%", "0.05%", "100%", "0", "12"] {
            assert_eq!(text.parse::<Position>().unwrap().to_string(), text);
        }
    }

    #[test]
    fn fractional_percentages_place_edges_between_whole_percent_cells() {
        // On a 321-cell extent 1% is 3.2 cells; 70.5% lands between 70% and 71%.
        let at = |text: &str| text.parse::<Position>().unwrap().resolve(321);
        assert_eq!(at("70%"), 224);
        assert_eq!(at("70.5%"), 226);
        assert_eq!(at("71%"), 227);
        assert_eq!(Position::percent_of(226, 321).resolve(321), 226);
    }

    #[test]
    fn percentages_map_zero_to_the_first_cell_and_hundred_to_the_last() {
        let parent = Bounds::new(10, 5, 109, 54); // 100 x 50 cells
        let all = InputBounds::parse("0%", "0%", "100%", "100%").unwrap();
        assert_eq!(all.to_bounds(&parent), parent);
        let half = InputBounds::parse("50%", "50%", "100%", "100%").unwrap();
        let bounds = half.to_bounds(&parent);
        assert_eq!((bounds.x1, bounds.y1), (10 + 50, 5 + 25));
    }

    #[test]
    fn cell_positions_are_offsets_clamped_to_the_parent() {
        let parent = Bounds::new(0, 0, 79, 23);
        let bounds = InputBounds::parse("2", "1", "500", "10")
            .unwrap()
            .to_bounds(&parent);
        assert_eq!(bounds, Bounds::new(2, 1, 79, 10));
    }

    #[test]
    fn edges_given_in_the_wrong_order_are_swapped() {
        let parent = Bounds::new(0, 0, 79, 23);
        let bounds = InputBounds::parse("60%", "10", "10%", "2")
            .unwrap()
            .to_bounds(&parent);
        assert_eq!((bounds.x1, bounds.y1, bounds.y2), (8, 2, 10));
        assert!(bounds.x1 <= bounds.x2);
    }

    #[test]
    fn converting_back_keeps_each_edge_in_its_unit() {
        let parent = Bounds::new(0, 0, 100, 50);
        let original = InputBounds::parse("10%", "20", "60%", "100%").unwrap();
        let absolute = original.to_bounds(&parent);
        assert_eq!(original.same_units_for(&absolute, &parent), original);
    }

    #[test]
    fn bounds_include_both_corners() {
        let b = Bounds::new(2, 3, 5, 3);
        assert_eq!((b.width(), b.height()), (4, 1));
        assert!(b.contains_point(2, 3) && b.contains_point(5, 3));
        assert!(!b.contains_point(6, 3) && !b.contains_point(1, 3) && !b.contains_point(2, 4));
        assert_eq!(
            Bounds::new(5, 4, 1, 0),
            Bounds::new(1, 0, 5, 4),
            "corners in any order"
        );
    }

    #[test]
    fn intersection_finds_any_overlap() {
        let a = Bounds::new(0, 0, 10, 10);
        // A cross shape: no corner of either rectangle lies inside the other.
        let b = Bounds::new(4, 0, 6, 20);
        let c = Bounds::new(0, 4, 20, 6);
        assert_eq!(b.intersection(&c), Some(Bounds::new(4, 4, 6, 6)));
        assert_eq!(
            a.intersection(&Bounds::new(10, 10, 12, 12)),
            Some(Bounds::new(10, 10, 10, 10))
        );
        assert_eq!(a.intersection(&Bounds::new(11, 0, 12, 12)), None);
    }

    #[test]
    fn shrinking_too_far_leaves_nothing() {
        let b = Bounds::new(0, 0, 4, 2);
        assert_eq!(b.shrink(1, 1), Some(Bounds::new(1, 1, 3, 1)));
        assert_eq!(b.shrink(1, 2), None);
        assert_eq!(b.shrink(3, 0), None);
    }

    fn boxed(position: &str, anchor: Anchor, min: Option<usize>, max: Option<usize>) -> MuxBox {
        let edges: Vec<&str> = position.split(' ').collect();
        MuxBox {
            id: "b".into(),
            position: InputBounds::parse(edges[0], edges[1], edges[2], edges[3]).unwrap(),
            anchor,
            min_width: min,
            max_width: max,
            ..MuxBox::default()
        }
    }

    #[test]
    fn minimum_and_maximum_sizes_grow_and_shrink_around_the_anchor() {
        let screen = Bounds::new(0, 0, 99, 49);
        // 11 columns wide at x 40..=50.
        let left = boxed("40 0 50 9", Anchor::TopLeft, Some(21), None);
        assert_eq!(box_bounds(&left, &screen), Bounds::new(40, 0, 60, 9));
        let right = boxed("40 0 50 9", Anchor::TopRight, Some(21), None);
        assert_eq!(box_bounds(&right, &screen), Bounds::new(30, 0, 50, 9));
        let center = boxed("40 0 50 9", Anchor::Center, None, Some(5));
        assert_eq!(box_bounds(&center, &screen), Bounds::new(43, 0, 47, 9));
    }

    #[test]
    fn a_grown_box_is_kept_inside_its_parent() {
        let screen = Bounds::new(0, 0, 99, 49);
        let near_edge = boxed("90 0 99 9", Anchor::TopLeft, Some(30), None);
        assert_eq!(box_bounds(&near_edge, &screen), Bounds::new(70, 0, 99, 9));
        let too_big = boxed("0 0 9 9", Anchor::Center, Some(500), None);
        assert_eq!(box_bounds(&too_big, &screen), Bounds::new(0, 0, 99, 9));
    }

    #[test]
    fn nested_boxes_are_placed_inside_their_parent() {
        let mut parent = boxed("50% 0% 100% 100%", Anchor::Center, None, None);
        parent.id = "parent".into();
        let mut child = boxed("0% 50% 100% 100%", Anchor::Center, None, None);
        child.id = "child".into();
        parent.children = Some(vec![child]);
        let layout = Layout {
            id: "l".into(),
            children: Some(vec![parent]),
            ..Layout::default()
        };
        let bounds = layout_bounds(&layout, &Bounds::new(0, 0, 100, 40));
        assert_eq!(bounds["parent"], Bounds::new(50, 0, 100, 40));
        assert_eq!(bounds["child"], Bounds::new(50, 20, 100, 40));
    }
}
