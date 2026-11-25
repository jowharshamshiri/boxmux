//! The values appearance settings can take in configuration.

use serde::{Deserialize, Serialize};

/// A color as written in configuration: one of the 16 ANSI colors, or the terminal's default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ColorName {
    Default,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

impl ColorName {
    pub const ALL: [ColorName; 17] = [
        ColorName::Default,
        ColorName::Black,
        ColorName::Red,
        ColorName::Green,
        ColorName::Yellow,
        ColorName::Blue,
        ColorName::Magenta,
        ColorName::Cyan,
        ColorName::White,
        ColorName::BrightBlack,
        ColorName::BrightRed,
        ColorName::BrightGreen,
        ColorName::BrightYellow,
        ColorName::BrightBlue,
        ColorName::BrightMagenta,
        ColorName::BrightCyan,
        ColorName::BrightWhite,
    ];

    /// The terminal color: base colors are ANSI 0-7, bright colors ANSI 8-15.
    pub fn color(self) -> crossterm::style::Color {
        use crossterm::style::Color;
        match self {
            ColorName::Default => Color::Reset,
            other => {
                let index = ColorName::ALL
                    .iter()
                    .position(|c| *c == other)
                    .expect("every color is in ALL");
                Color::AnsiValue(index as u8 - 1)
            }
        }
    }

    /// The name used in configuration.
    pub fn name(self) -> String {
        serde_json::to_value(self)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .expect("a color serializes to its name")
    }
}

/// What a box does with content larger than its content area.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Overflow {
    /// The content scrolls; scrollbars show on the border.
    #[default]
    Scroll,
    /// Long lines break into several rows; the rows scroll vertically.
    Wrap,
    /// What does not fit is cut off and marked with `…`.
    Truncate,
}

/// Where the title sits in the top border.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TitlePosition {
    Left,
    #[default]
    Center,
    Right,
}

/// Where text that fits in a box is placed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContentAlign {
    /// Centered in each direction that has room.
    #[default]
    Center,
    /// From the top-left corner of the content area.
    TopLeft,
}

/// Empty cells between a box's border and its content, as written in configuration. A side left
/// out takes the enclosing box's or the layout's, else its default.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Padding {
    pub top: Option<usize>,
    pub bottom: Option<usize>,
    pub left: Option<usize>,
    pub right: Option<usize>,
}

/// The padding of each side of a box, every side decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sides {
    pub top: usize,
    pub bottom: usize,
    pub left: usize,
    pub right: usize,
}

impl Sides {
    /// The padding of a side no configuration sets: no rows above or below the content, one
    /// column on each side of it.
    pub const DEFAULT: Sides = Sides {
        top: 0,
        bottom: 0,
        left: 1,
        right: 1,
    };

    /// Each side from the first of `paddings`, nearest first, that sets it, else its default.
    pub fn resolve<'a>(paddings: impl Iterator<Item = &'a Padding> + Clone) -> Sides {
        let side = |pick: fn(&Padding) -> Option<usize>, default: usize| {
            paddings.clone().find_map(pick).unwrap_or(default)
        };
        Sides {
            top: side(|p| p.top, Sides::DEFAULT.top),
            bottom: side(|p| p.bottom, Sides::DEFAULT.bottom),
            left: side(|p| p.left, Sides::DEFAULT.left),
            right: side(|p| p.right, Sides::DEFAULT.right),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::style::Color;

    #[test]
    fn each_side_of_the_padding_comes_from_the_nearest_setting_of_it() {
        let own = Padding {
            top: Some(0),
            ..Padding::default()
        };
        let enclosing = Padding {
            top: Some(3),
            left: Some(2),
            ..Padding::default()
        };
        let layout = Padding {
            left: Some(4),
            bottom: Some(1),
            ..Padding::default()
        };
        assert_eq!(
            Sides::resolve([own, enclosing, layout].iter()),
            Sides {
                top: 0,
                bottom: 1,
                left: 2,
                right: 1
            }
        );
        assert_eq!(Sides::resolve([].iter()), Sides::DEFAULT);
    }

    #[test]
    fn base_and_bright_colors_are_distinct_ansi_colors() {
        assert_eq!(ColorName::Black.color(), Color::AnsiValue(0));
        assert_eq!(ColorName::Red.color(), Color::AnsiValue(1));
        assert_eq!(ColorName::White.color(), Color::AnsiValue(7));
        assert_eq!(ColorName::BrightBlack.color(), Color::AnsiValue(8));
        assert_eq!(ColorName::BrightRed.color(), Color::AnsiValue(9));
        assert_eq!(ColorName::BrightWhite.color(), Color::AnsiValue(15));
        assert_eq!(ColorName::Default.color(), Color::Reset);
    }

    #[test]
    fn the_schema_lists_exactly_the_known_colors() {
        let schema: serde_json::Value =
            serde_json::from_str(crate::validation::APP_SCHEMA).unwrap();
        let names: Vec<String> = schema["definitions"]["color_name"]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        let known: Vec<String> = ColorName::ALL.iter().map(|c| c.name()).collect();
        assert_eq!(names, known);
    }

    #[test]
    fn the_schema_lists_exactly_the_overflow_title_positions_and_content_alignments() {
        let schema: serde_json::Value =
            serde_json::from_str(crate::validation::APP_SCHEMA).unwrap();
        let list = |def: &str| -> Vec<String> {
            schema["definitions"][def]["enum"]
                .as_array()
                .unwrap_or_else(|| panic!("{} has an enum", def))
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect()
        };
        let names = |values: &[serde_json::Value]| -> Vec<String> {
            values
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect()
        };
        assert_eq!(
            list("overflow_behavior"),
            names(&[
                serde_json::to_value(Overflow::Scroll).unwrap(),
                serde_json::to_value(Overflow::Wrap).unwrap(),
                serde_json::to_value(Overflow::Truncate).unwrap(),
            ])
        );
        assert_eq!(
            list("title_position"),
            names(&[
                serde_json::to_value(TitlePosition::Left).unwrap(),
                serde_json::to_value(TitlePosition::Center).unwrap(),
                serde_json::to_value(TitlePosition::Right).unwrap(),
            ])
        );
        assert_eq!(
            list("content_align"),
            names(&[
                serde_json::to_value(ContentAlign::Center).unwrap(),
                serde_json::to_value(ContentAlign::TopLeft).unwrap(),
            ])
        );
        let padding: Vec<&String> = schema["definitions"]["padding"]["properties"]
            .as_object()
            .expect("padding has properties")
            .keys()
            .collect();
        let fields = serde_json::to_value(Padding::default()).unwrap();
        let fields: Vec<&String> = fields.as_object().unwrap().keys().collect();
        assert_eq!(padding, fields);
    }
}
