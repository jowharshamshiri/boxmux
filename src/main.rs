use signal_hook::{consts::signal::SIGWINCH, iterator::Signals};
use simplelog::*;
use std::fs::File;
use std::io::Write as IoWrite;
use std::io::{stdin, stdout, Read};
use std::process::Command;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use termion::color;
use termion::cursor;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::{IntoRawMode, RawTerminal};
use termion::screen::AlternateScreen;

use serde::{
    de::MapAccess, de::SeqAccess, de::Visitor, Deserialize, Deserializer, Serialize, Serializer,
};
use std::fmt;

mod entities;
mod state;
mod utils;

use crate::entities::*;
use crate::state::*;
use crate::utils::*;

// BOX_EVENTS! {
//     "on_error",
//     "on_enter",
//     "on_leave",
//     "on_refresh",
// }

fn get_box_list_in_tab_order(box_list: &Vec<BoxEntity>, tab_order: &str) -> Vec<BoxEntity> {
    let mut tab_order_list = tab_order
        .split(',')
        .map(|id| {
            box_list
                .iter()
                .find(|b| b.id == id)
                .expect(&format!("Box with id {} not found", id))
                .clone()
        })
        .collect::<Vec<BoxEntity>>();

    // Sort by tab order
    tab_order_list.sort_by(|a, b| {
        let a_index = tab_order.split(',').position(|id| id == a.id).unwrap();
        let b_index = tab_order.split(',').position(|id| id == b.id).unwrap();
        a_index.cmp(&b_index)
    });

    tab_order_list
}

fn load_app_from_yaml(file_path: &str) -> Result<App, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let mut app: App = serde_yaml::from_str(&contents)?;

    // Populate parent fields recursively
    for layout in &mut app.layouts {
        let layout_clone = layout.clone();
        load_layout(&mut layout.children, &layout_clone, None);
    }

    Ok(app)
}

fn load_layout(
    children: &mut Vec<BoxEntity>,
    parent_layout: &Layout,
    parent: Option<Box<BoxEntity>>,
) {
    for child in children.iter_mut() {
        child.parent_layout = Some(Box::new(parent_layout.clone()));
        child.parent = parent.clone();

        if let Some(mut child_children) = child.children.take() {
            let parent_clone = Box::new(child.clone());
            load_layout(&mut child_children, parent_layout, Some(parent_clone));
            child.children = Some(child_children);
        }
    }
}

fn select_next_box(layout: &Layout) {
    let mut selected_box_guard = SELECTED_BOX.lock().unwrap();
    let tab_order_list = &layout.box_list_in_tab_order;

    if tab_order_list.is_empty() {
        return;
    }

    let next_index = match selected_box_guard.as_ref() {
        Some(selected_box) => {
            let selected_index = tab_order_list
                .iter()
                .position(|b| b == selected_box)
                .unwrap_or(0);
            (selected_index + 1) % tab_order_list.len()
        }
        None => 0,
    };

    *selected_box_guard = Some(tab_order_list[next_index].clone());
}

fn select_previous_box(layout: &Layout) {
    let mut selected_box_guard = SELECTED_BOX.lock().unwrap();
    let tab_order_list = &layout.box_list_in_tab_order;

    if tab_order_list.is_empty() {
        return;
    }

    let prev_index = match selected_box_guard.as_ref() {
        Some(selected_box) => {
            let selected_index = tab_order_list
                .iter()
                .position(|b| b == selected_box)
                .unwrap_or(0);
            if selected_index == 0 {
                tab_order_list.len() - 1
            } else {
                selected_index - 1
            }
        }
        None => tab_order_list.len() - 1,
    };

    *selected_box_guard = Some(tab_order_list[prev_index].clone());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Debug,
        Config::default(),
        File::create("/Users/bahram/ws/prj/machinegenesis/crossbash/trash/app.log").unwrap(),
    )])
    .unwrap();

    log::info!("Starting app");
    let mut stdout = AlternateScreen::from(stdout().into_raw_mode()?);

    // Load layout from YAML
    let mut app =
        load_app_from_yaml("/Users/bahram/ws/prj/machinegenesis/crossbash/layouts/dashboard.yaml")?;

    for layout in &mut app.layouts {
        layout.populate_tab_order();
        if !layout.box_list_in_tab_order.is_empty() {
            let first_box = layout.box_list_in_tab_order[0].clone();
            println!("Initially selecting box: {}", first_box.id); // Debug print
            *SELECTED_BOX.lock().unwrap() = Some(first_box);
        }
    }

    // Channel to communicate between input handling and drawing
    let (tx, rx) = mpsc::channel();

    // Handle input in a separate thread
    let input_tx = tx.clone();
    thread::spawn(move || {
        let stdin = stdin();
        for c in stdin.keys() {
            match c.unwrap() {
                Key::Char('q') => {
                    input_tx.send(InputMessage::Exit).unwrap();
                    break;
                }
                // tab
                Key::Char('\t') => {
                    input_tx.send(InputMessage::NextBox).unwrap();
                }
                // shift-tab
                Key::BackTab => {
                    input_tx.send(InputMessage::PreviousBox).unwrap();
                }
                // down arrow
                Key::Down => {
                    let mut selected_box_guard = SELECTED_BOX.lock().unwrap();
                    if let Some(ref mut selected_box) = *selected_box_guard {
                        selected_box.scroll_down(None);
                        input_tx
                            .send(InputMessage::RedrawBox(selected_box.clone()))
                            .unwrap();
                        log::info!("scrolled down, {}", selected_box.vertical_scroll.unwrap());
                    }
                }
                // up arrow
                Key::Up => {
                    let mut selected_box_guard = SELECTED_BOX.lock().unwrap();
                    if let Some(ref mut selected_box) = *selected_box_guard {
                        selected_box.scroll_up(None);
                        input_tx
                            .send(InputMessage::RedrawBox(selected_box.clone()))
                            .unwrap();
                        log::info!("scrolled up, {}", selected_box.vertical_scroll.unwrap());
                    }
                }
                _ => {}
            }
        }
    });

    // Start refresh threads for each box with a refresh interval
    app.start_event_threads();

    // Handle terminal resize
    let resize_tx = tx.clone();
    thread::spawn(move || {
        let mut signals = Signals::new(&[SIGWINCH]).unwrap();
        for _ in signals.forever() {
            resize_tx.send(InputMessage::Resize).unwrap();
        }
    });

    // Main drawing loop
    let mut screen_buffer = ScreenBuffer::new(screen_width(), screen_height());
    let mut prev_screen_buffer = screen_buffer.clone();

    app.draw(&mut stdout, &mut screen_buffer);
    loop {
        // Check for input
        if let Ok(msg) = rx.try_recv() {
            match msg {
                InputMessage::Exit => break,
                InputMessage::Resize => {
                    write!(stdout, "{}", termion::clear::All).unwrap();
                    screen_buffer = ScreenBuffer::new(screen_width(), screen_height());
                    prev_screen_buffer = screen_buffer.clone();
                    app.draw(&mut stdout, &mut screen_buffer);
                }
                InputMessage::NextBox => {
                    for layout in &app.layouts {
                        select_next_box(layout);
                    }
                    tx.send(InputMessage::RedrawApp).unwrap();
                }
                InputMessage::PreviousBox => {
                    for layout in &app.layouts {
                        select_previous_box(layout);
                    }
                    tx.send(InputMessage::RedrawApp).unwrap();
                }
                InputMessage::RedrawBox(mut box_entity) => {
                    log::info!(
                        "Redrawing box '{}' with vertical scroll '{}'",
                        box_entity.id,
                        box_entity.current_vertical_scroll()
                    );
                    box_entity.draw(&mut stdout, &mut screen_buffer);
                    // tx.send(InputMessage::RedrawApp).unwrap();
                }
                InputMessage::RedrawApp => {
                    app.draw(&mut stdout, &mut screen_buffer);
                    // Compare screen buffers and update only the differences
                    for y in 0..screen_buffer.height {
                        for x in 0..screen_buffer.width {
                            if screen_buffer.get(x, y) != prev_screen_buffer.get(x, y) {
                                if let Some(cell) = screen_buffer.get(x, y) {
                                    write!(
                                        stdout,
                                        "{}{}{}{}",
                                        cursor::Goto((x + 1) as u16, (y + 1) as u16),
                                        cell.bg_color,
                                        cell.fg_color,
                                        cell.ch
                                    )
                                    .unwrap();
                                }
                            }
                        }
                    }
                    stdout.flush().unwrap();
                    prev_screen_buffer = screen_buffer.clone();
                }
            }
        }
        thread::sleep(Duration::from_millis(10));
    }

    Ok(())
}

// the instances of box_entity that the app.draw function reaches are not the same instances that are in selected_box, and i think might be their clones, so when i scroll them up or down the result is not reflected in app redraw. how do i fix that. this is the current state of my code, just give me the updated sections.
