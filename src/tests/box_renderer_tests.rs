//! What a box looks like on screen: border, title, the visible tab's content, clipping and
//! status colors. Each test draws a whole app from a configuration into a screen buffer.

#[cfg(test)]
mod tests {
    use crate::model::app::{load_app_from_str, AppContext};
    use crate::model::appearance::ColorName;
    use crate::model::stream::StreamStatus;
    use crate::render::screen::{Color, ScreenBuffer};
    use crate::Config;

    fn context(yaml: &str, cols: u16, rows: u16) -> AppContext {
        let mut app = load_app_from_str(yaml, "test.yaml").expect("test configuration loads");
        app.set_screen_size(cols, rows).unwrap();
        AppContext::new(app, Config::new(1, false, false).unwrap())
    }

    fn draw(ctx: &mut AppContext) -> ScreenBuffer {
        let screen = ctx.app.screen();
        let bounds = ctx.app.active_bounds();
        let views = crate::render::views(ctx, &bounds, None);
        crate::render::draw(ctx, &views, screen.width(), screen.height(), None)
    }

    fn fg(name: ColorName) -> Color {
        name.color()
    }

    fn rows(buffer: &ScreenBuffer) -> Vec<String> {
        buffer.text_rows()
    }

    const BOX: &str = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: b
          title: Status
          position: {x1: "2", y1: "1", x2: "21", y2: "5"}
          border_color: green
          content: |
            first line of content that is far too long for the box
            second
            third
            fourth
            fifth
"#;

    #[test]
    fn the_title_sits_in_the_top_border_and_content_inside_it() {
        let mut ctx = context(BOX, 30, 8);
        let rows = rows(&draw(&mut ctx));
        assert!(rows[1].contains(" Status "), "{:?}", rows);
        assert!(
            rows[2].starts_with("  │ first line"),
            "one column of margin: {:?}",
            rows
        );
        // Every border corner is drawn.
        for (x, y) in [(2, 1), (21, 1), (2, 5), (21, 5)] {
            assert_ne!(
                rows[y].chars().nth(x),
                Some(' '),
                "corner ({}, {}) in {:?}",
                x,
                y,
                rows
            );
        }
    }

    #[test]
    fn content_is_clipped_to_the_box() {
        let mut ctx = context(BOX, 30, 8);
        let rows = rows(&draw(&mut ctx));
        // Nothing is drawn right of the box or below it.
        for row in &rows {
            let right_of_box: String = row.chars().skip(22).collect();
            assert!(right_of_box.trim().is_empty(), "{:?}", rows);
        }
        assert!(
            rows[6].trim().is_empty() && rows[7].trim().is_empty(),
            "{:?}",
            rows
        );
        // Three lines fit inside the border; the rest needs scrolling.
        let text = rows.join("\n");
        assert!(text.contains("third"));
        assert!(!text.contains("fourth"), "{:?}", rows);
    }

    #[test]
    fn only_the_visible_tab_is_drawn() {
        let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: b
          position: {x1: "0", y1: "0", x2: "39", y2: "5"}
          content: static text
          choices:
            - id: c
              content: menu entry
"#;
        let mut ctx = context(yaml, 40, 6);
        let text = rows(&draw(&mut ctx)).join("\n");
        assert!(
            text.contains("static text") && !text.contains("menu entry"),
            "{}",
            text
        );
        ctx.app.get_muxbox_by_id_mut("b").unwrap().switch_to_tab(1);
        let text = rows(&draw(&mut ctx)).join("\n");
        assert!(
            !text.contains("static text") && text.contains("menu entry"),
            "{}",
            text
        );
    }

    #[test]
    fn a_terminal_box_border_shows_how_its_process_ended() {
        let yaml = r#"
app:
  layouts:
    - id: main
      root: true
      children:
        - id: t
          position: {x1: "0", y1: "0", x2: "19", y2: "4"}
          border_color: green
          execution_mode: Pty
          script: ['true']
"#;
        let mut ctx = context(yaml, 20, 5);
        let border_color = |ctx: &mut AppContext| draw(ctx).get(0, 0).unwrap().style.fg;
        let set_status = |ctx: &mut AppContext, status: StreamStatus| {
            let muxbox = ctx.app.get_muxbox_by_id_mut("t").unwrap();
            muxbox.get_selected_stream_mut().unwrap().status = status;
        };
        set_status(&mut ctx, StreamStatus::Running);
        assert_eq!(border_color(&mut ctx), fg(ColorName::Green));
        set_status(&mut ctx, StreamStatus::Exited(1));
        assert_eq!(border_color(&mut ctx), fg(ColorName::Red));
        set_status(&mut ctx, StreamStatus::Exited(0));
        assert_eq!(border_color(&mut ctx), fg(ColorName::Yellow));
        set_status(&mut ctx, StreamStatus::Killed);
        assert_eq!(border_color(&mut ctx), fg(ColorName::Yellow));
    }
}
