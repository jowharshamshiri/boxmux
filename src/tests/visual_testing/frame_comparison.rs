// F0330: Visual Diff Detection - Detect and validate specific changes between terminal frames
// Frame comparison, highlight changed regions, count character differences

use super::terminal_capture::{TerminalCell, TerminalFrame};
use std::collections::HashMap;

/// F0330: Represents difference between two terminal frames
#[derive(Debug, Clone, PartialEq)]
pub struct FrameDiff {
    /// Changes between frames
    pub changes: Vec<CellChange>,
    /// Total number of changed cells
    pub change_count: usize,
    /// Change regions (grouped changes)
    pub regions: Vec<ChangeRegion>,
    /// Frame dimensions
    pub dimensions: (u16, u16),
}

/// F0330: Individual cell change
#[derive(Debug, Clone, PartialEq)]
pub struct CellChange {
    /// Position of change
    pub position: (u16, u16),
    /// Change type
    pub change_type: ChangeType,
    /// Old cell state
    pub old_cell: TerminalCell,
    /// New cell state
    pub new_cell: TerminalCell,
}

/// F0330: Type of change detected
#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    /// Character changed
    CharacterChange,
    /// Foreground color changed
    ForegroundColor,
    /// Background color changed
    BackgroundColor,
    /// Text attributes changed
    AttributeChange,
    /// Multiple properties changed
    MultipleChanges(Vec<ChangeType>),
}

/// F0330: Grouped change region
#[derive(Debug, Clone, PartialEq)]
pub struct ChangeRegion {
    /// Top-left corner of region
    pub top_left: (u16, u16),
    /// Bottom-right corner of region
    pub bottom_right: (u16, u16),
    /// Changes in this region
    pub changes: Vec<CellChange>,
    /// Region classification
    pub region_type: RegionType,
}

/// F0330: Classification of change regions
#[derive(Debug, Clone, PartialEq)]
pub enum RegionType {
    /// Text content update
    TextUpdate,
    /// Border modification
    BorderChange,
    /// Background fill
    BackgroundFill,
    /// Cursor movement
    CursorChange,
    /// Unknown/mixed changes
    Mixed,
}

impl FrameDiff {
    /// Create new empty frame diff
    pub fn new(dimensions: (u16, u16)) -> Self {
        Self {
            changes: Vec::new(),
            change_count: 0,
            regions: Vec::new(),
            dimensions,
        }
    }

    /// F0330: Compare two frames and generate diff
    pub fn compare(frame1: &TerminalFrame, frame2: &TerminalFrame) -> Self {
        let mut diff = FrameDiff::new(frame1.dimensions);

        if frame1.dimensions != frame2.dimensions {
            // Handle dimension mismatch - for now, use smaller dimensions
            let width = frame1.dimensions.0.min(frame2.dimensions.0);
            let height = frame1.dimensions.1.min(frame2.dimensions.1);
            diff.dimensions = (width, height);
        }

        // Compare each cell
        for y in 0..diff.dimensions.1 {
            if y as usize >= frame1.buffer.len() || y as usize >= frame2.buffer.len() {
                continue;
            }

            let row1 = &frame1.buffer[y as usize];
            let row2 = &frame2.buffer[y as usize];

            for x in 0..diff.dimensions.0 {
                if x as usize >= row1.len() || x as usize >= row2.len() {
                    continue;
                }

                let cell1 = &row1[x as usize];
                let cell2 = &row2[x as usize];

                if let Some(change) = Self::compare_cells(cell1, cell2, (x, y)) {
                    diff.changes.push(change);
                    diff.change_count += 1;
                }
            }
        }

        // Group changes into regions
        diff.regions = Self::group_changes_into_regions(&diff.changes);

        diff
    }

    /// Compare individual cells
    fn compare_cells(
        cell1: &TerminalCell,
        cell2: &TerminalCell,
        position: (u16, u16),
    ) -> Option<CellChange> {
        let mut change_types = Vec::new();

        if cell1.ch != cell2.ch {
            change_types.push(ChangeType::CharacterChange);
        }

        if cell1.fg_color != cell2.fg_color {
            change_types.push(ChangeType::ForegroundColor);
        }

        if cell1.bg_color != cell2.bg_color {
            change_types.push(ChangeType::BackgroundColor);
        }

        if cell1.attributes != cell2.attributes {
            change_types.push(ChangeType::AttributeChange);
        }

        if change_types.is_empty() {
            return None;
        }

        let change_type = if change_types.len() == 1 {
            change_types.into_iter().next().unwrap()
        } else {
            ChangeType::MultipleChanges(change_types)
        };

        Some(CellChange {
            position,
            change_type,
            old_cell: cell1.clone(),
            new_cell: cell2.clone(),
        })
    }

    /// Group individual changes into logical regions
    fn group_changes_into_regions(changes: &[CellChange]) -> Vec<ChangeRegion> {
        if changes.is_empty() {
            return Vec::new();
        }

        let mut regions = Vec::new();
        let mut processed = vec![false; changes.len()];

        for i in 0..changes.len() {
            if processed[i] {
                continue;
            }

            let mut region_changes = vec![changes[i].clone()];
            processed[i] = true;

            let (mut min_x, mut min_y) = changes[i].position;
            let (mut max_x, mut max_y) = changes[i].position;

            // Find adjacent changes to group together
            for j in (i + 1)..changes.len() {
                if processed[j] {
                    continue;
                }

                let (x, y) = changes[j].position;

                // Check if this change is adjacent to the current region
                if Self::is_adjacent_to_region((x, y), (min_x, min_y), (max_x, max_y)) {
                    region_changes.push(changes[j].clone());
                    processed[j] = true;

                    min_x = min_x.min(x);
                    min_y = min_y.min(y);
                    max_x = max_x.max(x);
                    max_y = max_y.max(y);
                }
            }

            let region_type = Self::classify_region(&region_changes);

            regions.push(ChangeRegion {
                top_left: (min_x, min_y),
                bottom_right: (max_x, max_y),
                changes: region_changes,
                region_type,
            });
        }

        regions
    }

    /// Check if position is adjacent to region bounds
    fn is_adjacent_to_region(
        pos: (u16, u16),
        region_min: (u16, u16),
        region_max: (u16, u16),
    ) -> bool {
        let (x, y) = pos;
        let (min_x, min_y) = region_min;
        let (max_x, max_y) = region_max;

        // Adjacent if within 1 cell of region bounds
        x >= min_x.saturating_sub(1)
            && x <= max_x + 1
            && y >= min_y.saturating_sub(1)
            && y <= max_y + 1
    }

    /// Classify what type of region this represents
    fn classify_region(changes: &[CellChange]) -> RegionType {
        if changes.is_empty() {
            return RegionType::Mixed;
        }

        // Count different types of changes
        let mut char_changes = 0;
        let mut color_changes = 0;
        let mut border_chars = 0;

        for change in changes {
            match &change.change_type {
                ChangeType::CharacterChange => {
                    char_changes += 1;
                    if Self::is_border_character(change.new_cell.ch) {
                        border_chars += 1;
                    }
                }
                ChangeType::ForegroundColor | ChangeType::BackgroundColor => {
                    color_changes += 1;
                }
                ChangeType::MultipleChanges(types) => {
                    for change_type in types {
                        match change_type {
                            ChangeType::CharacterChange => char_changes += 1,
                            ChangeType::ForegroundColor | ChangeType::BackgroundColor => {
                                color_changes += 1
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        // Classify based on change patterns
        if border_chars > 0 && border_chars >= char_changes / 2 {
            RegionType::BorderChange
        } else if char_changes > 0 && color_changes == 0 {
            RegionType::TextUpdate
        } else if color_changes > char_changes {
            RegionType::BackgroundFill
        } else {
            RegionType::Mixed
        }
    }

    /// Check if character is likely a border character
    fn is_border_character(ch: char) -> bool {
        matches!(
            ch,
            '┌' | '┐' | '└' | '┘' | '─' | '│' | '├' | '┤' | '┬' | '┴' | '┼'
        )
    }

    /// Get changes in specific region
    pub fn get_changes_in_region(
        &self,
        top_left: (u16, u16),
        bottom_right: (u16, u16),
    ) -> Vec<&CellChange> {
        self.changes
            .iter()
            .filter(|change| {
                let (x, y) = change.position;
                x >= top_left.0 && x <= bottom_right.0 && y >= top_left.1 && y <= bottom_right.1
            })
            .collect()
    }

    /// Count changes of specific type
    pub fn count_changes_of_type(&self, change_type: &ChangeType) -> usize {
        self.changes
            .iter()
            .filter(|change| Self::change_matches_type(&change.change_type, change_type))
            .count()
    }

    /// Check if change type matches filter
    fn change_matches_type(actual: &ChangeType, filter: &ChangeType) -> bool {
        match (actual, filter) {
            (a, b) if a == b => true,
            (ChangeType::MultipleChanges(types), filter) => types.iter().any(|t| t == filter),
            _ => false,
        }
    }

    /// Get summary of changes
    pub fn summary(&self) -> String {
        format!(
            "{} total changes, {} regions",
            self.change_count,
            self.regions.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::visual_testing::terminal_capture::{TerminalCell, TerminalFrame};
    use std::time::Instant;

    fn create_test_cell(ch: char) -> TerminalCell {
        TerminalCell {
            ch,
            ..Default::default()
        }
    }

    fn create_test_frame(content: &[&str]) -> TerminalFrame {
        let height = content.len();
        let width = content.iter().map(|line| line.len()).max().unwrap_or(0);

        let mut buffer = Vec::new();
        for line in content {
            let mut row = Vec::new();
            for ch in line.chars() {
                row.push(create_test_cell(ch));
            }
            // Pad row to width
            while row.len() < width {
                row.push(create_test_cell(' '));
            }
            buffer.push(row);
        }

        TerminalFrame {
            buffer,
            cursor: (0, 0),
            cursor_visible: true,
            timestamp: Instant::now(),
            dimensions: (width as u16, height as u16),
        }
    }

    #[test]
    fn test_no_changes() {
        let frame1 = create_test_frame(&["Hello", "World"]);
        let frame2 = create_test_frame(&["Hello", "World"]);

        let diff = FrameDiff::compare(&frame1, &frame2);
        assert_eq!(diff.change_count, 0);
        assert!(diff.changes.is_empty());
    }

    #[test]
    fn test_character_changes() {
        let frame1 = create_test_frame(&["Hello"]);
        let frame2 = create_test_frame(&["World"]);

        let diff = FrameDiff::compare(&frame1, &frame2);
        assert_eq!(diff.change_count, 4); // H->W, e->o, l->r, o->d (l->l no change)

        // Check specific changes
        let changes: HashMap<(u16, u16), char> = diff
            .changes
            .iter()
            .map(|c| (c.position, c.new_cell.ch))
            .collect();

        assert_eq!(changes.get(&(0, 0)), Some(&'W'));
        assert_eq!(changes.get(&(1, 0)), Some(&'o'));
        assert_eq!(changes.get(&(2, 0)), Some(&'r'));
        assert_eq!(changes.get(&(4, 0)), Some(&'d'));

        // Position 3 should have no change (l->l)
        assert_eq!(changes.get(&(3, 0)), None);
    }

    #[test]
    fn test_region_grouping() {
        let frame1 = create_test_frame(&["     ", " ABC ", "     "]);
        let frame2 = create_test_frame(&["     ", " XYZ ", "     "]);

        let diff = FrameDiff::compare(&frame1, &frame2);
        assert_eq!(diff.change_count, 3);
        assert_eq!(diff.regions.len(), 1); // Should be grouped into one region

        let region = &diff.regions[0];
        assert_eq!(region.top_left, (1, 1));
        assert_eq!(region.bottom_right, (3, 1));
        assert_eq!(region.changes.len(), 3);
    }
}
