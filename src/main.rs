use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write as IoWrite;
use std::io::{stdin, stdout, Read};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

fn print_with_color_at(
    y: usize,
    x: usize,
    color: &str,
    text: &str,
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
) {
    let color_code = get_color(color);
    write!(
        screen,
        "{}{}{}",
        cursor::Goto((x + 1).try_into().unwrap(), (y + 1).try_into().unwrap()),
        color_code,
        text
    )
    .unwrap();
}

fn parse_percentage(value: &str, total: usize) -> usize {
    if value.ends_with('%') {
        let percentage = value.trim_end_matches('%').parse::<f64>().unwrap() / 100.0;
        (percentage * total as f64).round() as usize
    } else {
        value.parse::<usize>().unwrap()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct InputBounds {
    x1: String,
    y1: String,
    x2: String,
    y2: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Bounds {
    x1: usize,
    y1: usize,
    x2: usize,
    y2: usize,
}

fn input_bounds_to_bounds(input_bounds: &InputBounds, parent_bounds: &Bounds) -> Bounds {
    let bx1 = parse_percentage(&input_bounds.x1, parent_bounds.x2 - parent_bounds.x1);
    let by1 = parse_percentage(&input_bounds.y1, parent_bounds.y2 - parent_bounds.y1);
    let bx2 = parse_percentage(&input_bounds.x2, parent_bounds.x2 - parent_bounds.x1);
    let by2 = parse_percentage(&input_bounds.y2, parent_bounds.y2 - parent_bounds.y1);
    let abs_x1 = parent_bounds.x1 + bx1;
    let abs_y1 = parent_bounds.y1 + by1;
    let abs_x2 = parent_bounds.x1 + bx2;
    let abs_y2 = parent_bounds.y1 + by2;
    Bounds {
        x1: abs_x1,
        y1: abs_y1,
        x2: abs_x2,
        y2: abs_y2,
    }
}

fn screen_width() -> usize {
    termion::terminal_size().unwrap().0 as usize
}

fn screen_height() -> usize {
    termion::terminal_size().unwrap().1 as usize
}

fn screen_bounds() -> Bounds {
    Bounds {
        x1: 0,
        y1: 0,
        x2: screen_width(),
        y2: screen_height(),
    }
}

fn get_color(color: &str) -> String {
    match color {
        "red" => format!("{}", color::Fg(color::Red)),
        "green" => format!("{}", color::Fg(color::Green)),
        "yellow" => format!("{}", color::Fg(color::Yellow)),
        "blue" => format!("{}", color::Fg(color::Blue)),
        "magenta" => format!("{}", color::Fg(color::Magenta)),
        "cyan" => format!("{}", color::Fg(color::Cyan)),
        "white" => format!("{}", color::Fg(color::White)),
        "black" => format!("{}", color::Fg(color::Black)),
        _ => format!("{}", color::Fg(color::Reset)),
    }
}

fn box_draw(
    bounds: &Bounds,
    border_color: &str,
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
) {
    let color_code = get_color(border_color);

    // Draw top and bottom borders
    write!(
        screen,
        "{}{}{}",
        cursor::Goto(
            (bounds.x1 + 1).try_into().unwrap(),
            (bounds.y1 + 1).try_into().unwrap()
        ),
        color_code,
        "─".repeat((bounds.x2 - bounds.x1 + 1) as usize)
    )
    .unwrap();
    write!(
        screen,
        "{}{}{}",
        cursor::Goto(
            (bounds.x1 + 1).try_into().unwrap(),
            (bounds.y2 + 1).try_into().unwrap()
        ),
        color_code,
        "─".repeat((bounds.x2 - bounds.x1 + 1) as usize)
    )
    .unwrap();

    // Draw left and right borders
    for y in bounds.y1 + 1..bounds.y2 {
        write!(
            screen,
            "{}{}│",
            cursor::Goto(
                (bounds.x1 + 1).try_into().unwrap(),
                (y + 1).try_into().unwrap()
            ),
            color_code
        )
        .unwrap();
        write!(
            screen,
            "{}{}│",
            cursor::Goto(
                (bounds.x2 + 1).try_into().unwrap(),
                (y + 1).try_into().unwrap()
            ),
            color_code
        )
        .unwrap();
    }

    // Draw corners
    write!(
        screen,
        "{}{}┌",
        cursor::Goto(
            (bounds.x1 + 1).try_into().unwrap(),
            (bounds.y1 + 1).try_into().unwrap()
        ),
        color_code
    )
    .unwrap();
    write!(
        screen,
        "{}{}┐",
        cursor::Goto(
            (bounds.x2 + 1).try_into().unwrap(),
            (bounds.y1 + 1).try_into().unwrap()
        ),
        color_code
    )
    .unwrap();
    write!(
        screen,
        "{}{}└",
        cursor::Goto(
            (bounds.x1 + 1).try_into().unwrap(),
            (bounds.y2 + 1).try_into().unwrap()
        ),
        color_code
    )
    .unwrap();
    write!(
        screen,
        "{}{}┘",
        cursor::Goto(
            (bounds.x2 + 1).try_into().unwrap(),
            (bounds.y2 + 1).try_into().unwrap()
        ),
        color_code
    )
    .unwrap();
}

fn fill_box(
    bounds: &Bounds,
    fill_color: &str,
    fill_char: char,
    screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>,
) {
    let color_code = get_color(fill_color);
    for y in bounds.y1..=bounds.y2 - 1 {
        write!(
            screen,
            "{}",
            cursor::Goto(
                (bounds.x1 + 1).try_into().unwrap(),
                (y + 1).try_into().unwrap()
            )
        )
        .unwrap();
        write!(
            screen,
            "{}{}",
            color_code,
            fill_char
                .to_string()
                .repeat((bounds.x2 - bounds.x1) as usize)
        )
        .unwrap();
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct BoxEntity {
    id: String,
    title: Option<String>,
    position: InputBounds,
    min_width: Option<usize>,
    min_height: Option<usize>,
    max_width: Option<usize>,
    max_height: Option<usize>,
    overflow_behavior: Option<String>,
    content: Option<String>,
    scroll: bool,
    refresh_interval: Option<u64>,
    tab_order: Option<String>,
    on_error: Option<String>,
    on_enter: Option<Vec<String>>,
    on_leave: Option<Vec<String>>,
    next_focus_id: Option<String>,
    children: Option<Vec<BoxEntity>>,
    fill: Option<bool>,
    fill_color: Option<String>,
    fill_char: Option<char>,
    border: Option<bool>,
    border_color: Option<String>,
    text_color: Option<String>,
    title_color: Option<String>,
    on_refresh: Option<Vec<String>>,
    #[serde(skip)]
    output: String,
    parent: Option<Box<BoxEntity>>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct App {
    layouts: Vec<Layout>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct Layout {
    id: String,
    title: String,
    refresh_interval: Option<u64>,
    children: Vec<BoxEntity>,
}

impl BoxEntity {
    fn bounds(&self) -> Bounds {
        input_bounds_to_bounds(&self.position, &screen_bounds())
    }

    fn absolute_bounds(&self, parent_bounds: Option<&Bounds>) -> Bounds {
        let screen_bounds_value = screen_bounds();
        let actual_parent_bounds = parent_bounds.unwrap_or(&screen_bounds_value);
        input_bounds_to_bounds(&self.position, actual_parent_bounds)
    }

    fn set_output(&mut self, output: &str) {
        self.output = output.to_string();
    }

    fn draw(&mut self, screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>) {
        let parent_bounds = if self.parent.is_none() {
            Some(screen_bounds())
        } else {
            Some(self.parent.as_ref().unwrap().bounds())
        };

        let bounds = self.absolute_bounds(parent_bounds.as_ref());

        // Draw fill
        if self.fill.unwrap_or(false) {
            fill_box(
                &bounds,
                self.fill_color.as_deref().unwrap_or("white"),
                self.fill_char.unwrap_or('█'),
                screen,
            );
        }

        // Draw border
        if self.border.unwrap_or(false) {
            box_draw(
                &bounds,
                self.border_color.as_deref().unwrap_or("white"),
                screen,
            );
        }

        if let Some(title) = &self.title {
            // Print title
            print_with_color_at(
                bounds.y1,
                bounds.x1,
                self.title_color.as_deref().unwrap_or("white"),
                title,
                screen,
            );
        }

        // Print content
        if let Some(content) = &self.content {
            print_with_color_at(
                bounds.y1 + 2,
                bounds.x1 + 2,
                self.text_color.as_deref().unwrap_or("white"),
                content,
                screen,
            );
        }

        // Draw children
        if let Some(children) = &mut self.children {
            // Use &mut here
            for child in children {
                child.draw(screen);
            }
        }
    }
}

impl Layout {
    fn draw(&mut self, screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>) {
        for child in &mut self.children {
            // Use &mut here
            child.draw(screen);
        }
    }
}

impl App {
    fn draw(&mut self, screen: &mut AlternateScreen<RawTerminal<std::io::Stdout>>) {
        for layout in &mut self.layouts {
            // Use &mut here
            layout.draw(screen);
        }
    }
}
fn load_app_from_yaml(file_path: &str) -> Result<App, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut app: App = serde_yaml::from_str(&contents)?;

    // Populate parent fields
    for layout in &mut app.layouts {
        let mut updated_children = Vec::new();

        for mut child in layout.children.drain(..) {
            child.parent = None;

            if let Some(mut children) = child.children.take() {
                let parent_clone = Box::new(child.clone());
                for grandchild in children.iter_mut() {
                    grandchild.parent = Some(parent_clone.clone());
                }
                child.children = Some(children);
            }

            updated_children.push(child);
        }

        layout.children = updated_children;
    }

    Ok(app)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode()?);

    // Load layout from YAML
    let mut app =
        load_app_from_yaml("/Users/bahram/ws/prj/machinegenesis/crossbash/layouts/dashboard.yaml")?;

    // Channel to communicate between input handling and drawing
    let (tx, rx) = mpsc::channel();

    // Handle input in a separate thread
    let input_tx = tx.clone();
    thread::spawn(move || {
        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => {
                    input_tx.send("exit").unwrap();
                    break;
                }
                _ => {}
            }
        }
    });

    // Main drawing loop
    loop {
        app.draw(&mut stdout);
        // Check for input
        if let Ok(msg) = rx.try_recv() {
            if msg == "exit" {
                break;
            }
        }
        thread::sleep(Duration::from_millis(100));
    }

    Ok(())
}
