use crate::utils::{
    draw_panel, fill_panel, get_bg_color, get_fg_color, input_bounds_to_bounds, screen_bounds,
};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::raw::RawTerminal;
use termion::screen::AlternateScreen;

use serde::{de, ser};

use crate::model::panel::*;

#[derive(Clone, PartialEq, Debug)]
pub struct Cell {
    pub fg_color: String,
    pub bg_color: String,
    pub ch: char,
}

#[derive(Debug,Clone)]
pub struct ScreenBuffer {
    pub width: usize,
    pub height: usize,
    pub buffer: Vec<Vec<Cell>>,
}

impl ScreenBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let default_cell = Cell {
            fg_color: get_fg_color("default"),
            bg_color: get_bg_color("default"),
            ch: ' ',
        };
        let buffer = vec![vec![default_cell; width]; height];
        ScreenBuffer {
            width,
            height,
            buffer,
        }
    }

	pub fn new_filled(width: usize, height: usize, default_cell: Cell) -> Self {
		let buffer = vec![vec![default_cell; width]; height];
		ScreenBuffer {
			width,
			height,
			buffer,
		}
	}

    pub fn clear(&mut self) {
        let default_cell = Cell {
            fg_color: get_fg_color("default"),
            bg_color: get_bg_color("default"),
            ch: ' ',
        };
        self.buffer = vec![vec![default_cell; self.width]; self.height];
    }

    pub fn update(&mut self, x: usize, y: usize, cell: Cell) {
        if x < self.width && y < self.height {
            self.buffer[y][x] = cell;
        }
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x < self.width && y < self.height {
            Some(&self.buffer[y][x])
        } else {
            None
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Hash,Eq)]
pub struct InputBounds {
    pub x1: String,
    pub y1: String,
    pub x2: String,
    pub y2: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Bounds {
    pub x1: usize,
    pub y1: usize,
    pub x2: usize,
    pub y2: usize,
}

impl InputBounds {
    pub fn to_bounds(&self, parent_bounds: &Bounds) -> Bounds {
        input_bounds_to_bounds(self, parent_bounds)
    }
}

impl Bounds {
	pub fn new(x1: usize, y1: usize, x2: usize, y2: usize) -> Self {
		Bounds {
			x1,
			y1,
			x2,
			y2,
		}
	}

    pub fn width(&self) -> usize {
        self.x2.saturating_sub(self.x1)
    }

    pub fn height(&self) -> usize {
        self.y2.saturating_sub(self.y1)
    }

    pub fn contains(&self, x: usize, y: usize) -> bool {
        x >= self.x1 && x < self.x2 && y >= self.y1 && y < self.y2
    }

    pub fn contains_bounds(&self, other: &Bounds) -> bool {
        self.contains(other.x1, other.y1) && self.contains(other.x2, other.y2)
    }

    pub fn intersects(&self, other: &Bounds) -> bool {
        self.contains(other.x1, other.y1)
            || self.contains(other.x2, other.y2)
            || self.contains(other.x1, other.y2)
            || self.contains(other.x2, other.y1)
    }

    pub fn intersection(&self, other: &Bounds) -> Option<Bounds> {
        if self.intersects(other) {
            Some(Bounds {
                x1: self.x1.max(other.x1),
                y1: self.y1.max(other.y1),
                x2: self.x2.min(other.x2),
                y2: self.y2.min(other.y2),
            })
        } else {
            None
        }
    }

    pub fn union(&self, other: &Bounds) -> Bounds {
        Bounds {
            x1: self.x1.min(other.x1),
            y1: self.y1.min(other.y1),
            x2: self.x2.max(other.x2),
            y2: self.y2.max(other.y2),
        }
    }

    pub fn translate(&self, dx: isize, dy: isize) -> Bounds {
        Bounds {
            x1: (self.x1 as isize + dx) as usize,
            y1: (self.y1 as isize + dy) as usize,
            x2: (self.x2 as isize + dx) as usize,
            y2: (self.y2 as isize + dy) as usize,
        }
    }

    pub fn center(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, (self.y1 + self.y2) / 2)
    }

    pub fn center_x(&self) -> usize {
        (self.x1 + self.x2) / 2
    }

    pub fn center_y(&self) -> usize {
        (self.y1 + self.y2) / 2
    }

    pub fn top_left(&self) -> (usize, usize) {
        (self.x1, self.y1)
    }

    pub fn top_right(&self) -> (usize, usize) {
        (self.x2, self.y1)
    }

    pub fn bottom_left(&self) -> (usize, usize) {
        (self.x1, self.y2)
    }

    pub fn bottom_right(&self) -> (usize, usize) {
        (self.x2, self.y2)
    }

    pub fn top(&self) -> usize {
        self.y1
    }

    pub fn bottom(&self) -> usize {
        self.y2
    }

    pub fn left(&self) -> usize {
        self.x1
    }

    pub fn right(&self) -> usize {
        self.x2
    }

    pub fn center_top(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, self.y1)
    }

    pub fn center_bottom(&self) -> (usize, usize) {
        ((self.x1 + self.x2) / 2, self.y2)
    }

    pub fn center_left(&self) -> (usize, usize) {
        (self.x1, (self.y1 + self.y2) / 2)
    }

    pub fn center_right(&self) -> (usize, usize) {
        (self.x2, (self.y1 + self.y2) / 2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_screenbuffer_new() {
        let screen_buffer = ScreenBuffer::new(5, 5);
        assert_eq!(screen_buffer.width, 5);
        assert_eq!(screen_buffer.height, 5);
        assert_eq!(screen_buffer.buffer.len(), 5);
        assert_eq!(screen_buffer.buffer[0].len(), 5);
    }

    #[test]
    fn test_screenbuffer_clear() {
        let mut screen_buffer = ScreenBuffer::new(5, 5);
        let test_cell = Cell {
            fg_color: String::from("red"),
            bg_color: String::from("blue"),
            ch: 'X',
        };
        screen_buffer.update(2, 2, test_cell.clone());
        screen_buffer.clear();
        for row in screen_buffer.buffer.iter() {
            for cell in row.iter() {
                assert_eq!(cell.fg_color, get_fg_color("default"));
                assert_eq!(cell.bg_color, get_bg_color("default"));
                assert_eq!(cell.ch, ' ');
            }
        }
    }

    #[test]
    fn test_screenbuffer_update() {
        let mut screen_buffer = ScreenBuffer::new(5, 5);
        let test_cell = Cell {
            fg_color: String::from("red"),
            bg_color: String::from("blue"),
            ch: 'X',
        };
        screen_buffer.update(2, 2, test_cell.clone());
        assert_eq!(screen_buffer.get(2, 2).unwrap(), &test_cell);
    }

    #[test]
    fn test_screenbuffer_get() {
        let screen_buffer = ScreenBuffer::new(5, 5);
        assert!(screen_buffer.get(6, 6).is_none());
        assert!(screen_buffer.get(3, 3).is_some());
    }
}
