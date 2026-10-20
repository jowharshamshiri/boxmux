//! A scrollbar drawn on a box border: a track along the content area and a knob whose size and
//! place show which part of the content is visible.

/// A scrollbar for content of `content` lines (or columns) seen through a view of `view`,
/// scrolled to `offset`, on a track of `track` cells starting at cell `start`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scrollbar {
    pub start: usize,
    pub track: usize,
    pub content: usize,
    pub view: usize,
    pub offset: usize,
}

/// Where a click on a scrollbar landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollbarHit {
    Knob,
    /// The track before or after the knob.
    Track,
}

impl Scrollbar {
    pub fn max_offset(&self) -> usize {
        self.content.saturating_sub(self.view)
    }

    /// The knob's first cell and length, relative to the track. The knob is at least one cell
    /// and at most the track; it touches the start of the track exactly at offset 0 and the end
    /// exactly at the last offset.
    pub fn knob(&self) -> (usize, usize) {
        if self.track == 0 || self.content == 0 {
            return (0, 0);
        }
        let length = ((self.view as f64 / self.content as f64) * self.track as f64)
            .round()
            .clamp(1.0, self.track as f64) as usize;
        let free = self.track - length;
        let max = self.max_offset();
        let position = if max == 0 || free == 0 {
            0
        } else if self.offset >= max {
            free
        } else {
            // Every offset short of the last keeps the knob off the end of the track.
            ((self.offset as f64 / max as f64) * free as f64)
                .round()
                .min(free.saturating_sub(1) as f64) as usize
        };
        (position, length)
    }

    pub fn hit(&self, cell: usize) -> Option<ScrollbarHit> {
        let relative = cell.checked_sub(self.start).filter(|r| *r < self.track)?;
        let (position, length) = self.knob();
        Some(if (position..position + length).contains(&relative) {
            ScrollbarHit::Knob
        } else {
            ScrollbarHit::Track
        })
    }

    /// The offset a click on the track at `cell` scrolls to: the one that centers the knob
    /// there.
    pub fn offset_for_click(&self, cell: usize) -> usize {
        let (_, length) = self.knob();
        let free = self.track.saturating_sub(length);
        if free == 0 {
            return 0;
        }
        let relative = cell.saturating_sub(self.start).min(self.track - 1);
        let knob_start = relative.saturating_sub(length / 2).min(free);
        ((knob_start as f64 / free as f64) * self.max_offset() as f64).round() as usize
    }

    /// The offset after dragging the knob by `cells` from where it was at `start_offset`.
    pub fn offset_for_drag(&self, start_offset: usize, cells: isize) -> usize {
        let (_, length) = self.knob();
        let free = self.track.saturating_sub(length);
        if free == 0 {
            return start_offset.min(self.max_offset());
        }
        let per_cell = self.max_offset() as f64 / free as f64;
        let target = start_offset as f64 + cells as f64 * per_cell;
        target.round().clamp(0.0, self.max_offset() as f64) as usize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bar(offset: usize) -> Scrollbar {
        // 100 lines seen 10 at a time on a 20-cell track.
        Scrollbar {
            start: 5,
            track: 20,
            content: 100,
            view: 10,
            offset,
        }
    }

    #[test]
    fn the_knob_shows_the_visible_share_and_reaches_both_ends_only_at_the_ends() {
        assert_eq!(bar(0).knob(), (0, 2));
        assert_eq!(bar(90).knob(), (18, 2));
        let (position, _) = bar(89).knob();
        assert!(
            position < 18,
            "one line before the end must not look like the end"
        );
        assert!(bar(45).knob().0 > 0, "the middle is away from the start");
    }

    #[test]
    fn a_tiny_view_still_gets_a_one_cell_knob() {
        let bar = Scrollbar {
            start: 0,
            track: 10,
            content: 10_000,
            view: 5,
            offset: 0,
        };
        assert_eq!(bar.knob(), (0, 1));
    }

    #[test]
    fn clicks_tell_the_knob_from_the_track() {
        assert_eq!(bar(0).hit(5), Some(ScrollbarHit::Knob));
        assert_eq!(bar(0).hit(6), Some(ScrollbarHit::Knob));
        assert_eq!(bar(0).hit(7), Some(ScrollbarHit::Track));
        assert_eq!(bar(0).hit(4), None);
        assert_eq!(bar(0).hit(25), None);
    }

    #[test]
    fn clicking_the_track_moves_the_knob_there() {
        assert_eq!(bar(0).offset_for_click(5), 0);
        assert_eq!(bar(0).offset_for_click(24), 90);
        let middle = bar(0).offset_for_click(15);
        assert!((40..=50).contains(&middle), "{}", middle);
    }

    #[test]
    fn dragging_the_knob_moves_the_content_proportionally_and_stops_at_the_ends() {
        // 18 free cells for 90 offsets: 5 lines per cell.
        assert_eq!(bar(0).offset_for_drag(0, 2), 10);
        assert_eq!(bar(0).offset_for_drag(10, -5), 0);
        assert_eq!(bar(0).offset_for_drag(80, 50), 90);
    }
}
