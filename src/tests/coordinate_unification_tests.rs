//! Coordinate-system unification oracle.
//!
//! These tests pin the invariant the whole UI relies on: the mapping a box uses
//! to *draw* a content cell is the exact inverse of the mapping used to *hit-test*
//! a click, at any terminal size, box position (including far from the origin and
//! at fractional-percentage boundaries), border/tab configuration and scroll
//! offset. If render and hit ever diverge again, or an edge cell becomes
//! unreachable, or adjacent boxes leave an unclickable gap, one of these fails.

#[cfg(test)]
mod tests {
    use crate::components::box_renderer::{BoxDimensions, BoxRenderer};
    use crate::components::choice_menu::ChoiceMenu;
    use crate::components::renderable_content::RenderableContent;
    use crate::model::choice::Choice;
    use crate::model::common::{Stream, StreamType};
    use crate::model::muxbox::MuxBox;
    use crate::utils::input_bounds_to_bounds;
    use crate::Bounds;
    use indexmap::IndexMap;

    fn muxbox_with(border: bool, tabs: bool, h_scroll: f64, v_scroll: f64) -> MuxBox {
        let mut muxbox = MuxBox {
            id: "coord_test".to_string(),
            horizontal_scroll: Some(h_scroll),
            vertical_scroll: Some(v_scroll),
            streams: IndexMap::new(),
            ..Default::default()
        };
        if border {
            muxbox.border_color = Some("white".to_string());
        }
        if tabs {
            for i in 0..2 {
                muxbox.streams.insert(
                    format!("s{}", i),
                    Stream {
                        id: format!("s{}", i),
                        stream_type: StreamType::Content,
                        label: format!("S{}", i),
                        content: vec![],
                        choices: None,
                        source: None,
                        content_hash: 0,
                        last_updated: std::time::SystemTime::now(),
                        created_at: std::time::SystemTime::now(),
                    },
                );
            }
        }
        muxbox
    }

    /// inbox_to_screen and screen_to_inbox must be exact inverses over the
    /// viewable window, every viewable cell must be reachable, and no screen cell
    /// outside the viewable window may decode to an inbox cell.
    #[test]
    fn test_inbox_screen_mapping_is_bijective_across_configs() {
        // A spread of box rectangles: small, large, and deliberately far from the
        // origin / at odd coordinates so any accumulated rounding shows up.
        let rects = [
            Bounds::new(0, 0, 9, 5),
            Bounds::new(10, 5, 50, 25),
            Bounds::new(133, 41, 188, 53),
            Bounds::new(201, 67, 318, 119),
            Bounds::new(7, 3, 8, 4), // degenerate-ish tiny box
        ];
        let content_dims = [(3usize, 2usize), (40, 12), (400, 200)];
        let scrolls = [0.0f64, 25.0, 50.0, 100.0];

        for &border in &[false, true] {
            for &tabs in &[false, true] {
                for rect in &rects {
                    for &(cw, ch) in &content_dims {
                        for &hs in &scrolls {
                            for &vs in &scrolls {
                                let muxbox = muxbox_with(border, tabs, hs, vs);
                                let dims = BoxDimensions::new(&muxbox, rect, cw, ch);

                                let cb = dims.content_bounds;
                                let vw = dims.viewable_width;
                                let vh = dims.viewable_height;
                                if vw == 0 || vh == 0 {
                                    continue;
                                }

                                // 1. Every screen cell that decodes to an inbox cell
                                //    must round-trip back to the same screen cell,
                                //    only cells inside the viewable window may decode
                                //    at all, and when content fills the window every
                                //    window cell must be clickable (no holes).
                                for sy in rect.y1..=rect.y2 {
                                    for sx in rect.x1..=rect.x2 {
                                        let in_window = sx >= cb.left()
                                            && sx < cb.left() + vw
                                            && sy >= cb.top()
                                            && sy < cb.top() + vh;
                                        let content_fills_window = cw >= vw && ch >= vh;
                                        match dims.screen_to_inbox(sx, sy) {
                                            Some((ix, iy)) => {
                                                assert!(
                                                    in_window,
                                                    "decoded a cell outside the viewable window: \
                                                     screen=({sx},{sy}) border={border} tabs={tabs} \
                                                     rect={rect} cw={cw} ch={ch} hs={hs} vs={vs}"
                                                );
                                                assert!(ix < cw && iy < ch);
                                                assert_eq!(
                                                    dims.inbox_to_screen(ix, iy),
                                                    (sx, sy),
                                                    "screen->inbox->screen not identity at \
                                                     ({sx},{sy}) border={border} tabs={tabs} rect={rect}"
                                                );
                                            }
                                            None => assert!(
                                                !(in_window && content_fills_window),
                                                "viewable cell ({sx},{sy}) was not clickable \
                                                 border={border} tabs={tabs} rect={rect} cw={cw} ch={ch} \
                                                 hs={hs} vs={vs}"
                                            ),
                                        }
                                    }
                                }

                                // 2. The four corners of the viewable window are
                                //    reachable when content fills them (no edge is
                                //    silently unclickable).
                                if cw >= vw && ch >= vh {
                                    let corners = [
                                        (cb.left(), cb.top()),
                                        (cb.left() + vw - 1, cb.top()),
                                        (cb.left(), cb.top() + vh - 1),
                                        (cb.left() + vw - 1, cb.top() + vh - 1),
                                    ];
                                    for (sx, sy) in corners {
                                        assert!(
                                            dims.screen_to_inbox(sx, sy).is_some(),
                                            "viewable corner ({sx},{sy}) must be clickable \
                                             border={border} tabs={tabs} rect={rect} cw={cw} ch={ch} \
                                             hs={hs} vs={vs}"
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// For a box of choices, the stored screen-space sensitive zone of each visible
    /// choice must cover exactly the cells the renderer draws that choice on — same
    /// row, same horizontal extent — so a click on any visible glyph of a choice
    /// activates that choice and nothing else.
    #[test]
    fn test_choice_zone_matches_rendered_span() {
        let choices: Vec<Choice> = (0..8)
            .map(|i| Choice {
                id: format!("c{}", i),
                content: Some(format!("Choice number {}", i)),
                ..Default::default()
            })
            .collect();

        for rect in &[
            Bounds::new(0, 0, 30, 10),
            Bounds::new(25, 6, 73, 20),
            Bounds::new(133, 41, 188, 53),
        ] {
            for &vs in &[0.0f64, 50.0, 100.0] {
                let mut muxbox = muxbox_with(true, false, 0.0, vs);
                muxbox.id = "cbox".to_string();

                let menu = ChoiceMenu::new("cbox_choice_menu".to_string(), &choices);
                let lines = menu.generate_choice_lines();
                let (cw, ch) = menu.get_dimensions();
                let dims = BoxDimensions::new(&muxbox, rect, cw, ch);

                let renderer = BoxRenderer::new(&muxbox, "cbox_click_renderer".to_string());
                let zones = renderer.translate_box_relative_zones_to_absolute(
                    &menu.get_box_relative_sensitive_zones(),
                    rect,
                    cw,
                    ch,
                    dims.viewable_width,
                    dims.viewable_height,
                    dims.horizontal_scroll,
                    dims.vertical_scroll,
                    false,
                );

                let (vis_left, vis_top, _r, vis_bottom) = dims.get_visible_inbox_region();
                let last = vis_bottom.min(lines.len());

                for choice_index in vis_top..last {
                    // Expected rendered span for this choice row.
                    let (sx, sy) = dims.inbox_to_screen(vis_left, choice_index);
                    assert_ne!(
                        sx,
                        usize::MAX,
                        "visible choice {choice_index} must render (rect={rect} vs={vs})"
                    );
                    let visible_chars = lines[choice_index]
                        .chars()
                        .skip(vis_left)
                        .take(dims.viewable_width)
                        .count();
                    assert!(visible_chars > 0);
                    let expected = Bounds::new(sx, sy, sx + visible_chars - 1, sy);

                    let zone = zones
                        .iter()
                        .find(|z| z.content_id == format!("choice_{}", choice_index))
                        .unwrap_or_else(|| {
                            panic!("missing zone for visible choice {choice_index} (rect={rect} vs={vs})")
                        });

                    assert_eq!(
                        zone.bounds, expected,
                        "choice {choice_index} hit zone must equal its rendered span \
                         (rect={rect} vs={vs})"
                    );
                }
            }
        }
    }

    /// Adjacent percentage-positioned boxes must tile the screen with no
    /// unclickable column/row, at every terminal size — the fractional-rounding
    /// guarantee. With inclusive bounds adjacent boxes share their boundary cell,
    /// so every screen cell is owned by some box.
    #[test]
    fn test_percentage_boxes_leave_no_unclickable_gaps() {
        use crate::model::common::InputBounds;

        let split = |x1: &str, y1: &str, x2: &str, y2: &str| InputBounds {
            x1: x1.to_string(),
            y1: y1.to_string(),
            x2: x2.to_string(),
            y2: y2.to_string(),
        };

        // A few representative splits, including thirds which round unevenly.
        let layouts = [
            vec![
                split("0%", "0%", "50%", "100%"),
                split("50%", "0%", "100%", "100%"),
            ],
            vec![
                split("0%", "0%", "33%", "100%"),
                split("33%", "0%", "66%", "100%"),
                split("66%", "0%", "100%", "100%"),
            ],
            vec![
                split("0%", "0%", "100%", "50%"),
                split("0%", "50%", "100%", "100%"),
            ],
        ];

        for width in [40usize, 80, 100, 133, 191, 240, 317] {
            for height in [12usize, 24, 30, 51] {
                let root = Bounds::new(0, 0, width - 1, height - 1);
                for layout in &layouts {
                    let rects: Vec<Bounds> = layout
                        .iter()
                        .map(|ib| input_bounds_to_bounds(ib, &root))
                        .collect();

                    for sy in 0..height {
                        for sx in 0..width {
                            let owned = rects.iter().any(|r| r.contains_point(sx, sy));
                            assert!(
                                owned,
                                "screen cell ({sx},{sy}) is not owned by any box at \
                                 {width}x{height} (fractional-rounding gap)"
                            );
                        }
                    }
                }
            }
        }
    }
}
