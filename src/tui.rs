use std::process::exit;

use termion::{color, event::Key};

use crate::{position::Position, terminal::Terminal};

pub struct Tui {
    max_x: u16,
    max_y: u16,
    terminal: Terminal,
    widgets: Vec<Widget>,
}

impl Tui {
    pub fn new() -> Self {
        let boundary = Terminal::get_boundaries();
        Tui {
            max_x: boundary.get_x(),
            max_y: boundary.get_y(),
            terminal: Terminal::get_raw(),
            widgets: Vec::new(),
        }
    }

    pub fn start(&mut self) {
        self.terminal.begin();
        self.draw_background();
        self.tui_loop();
    }

    fn quit(&mut self) {
        self.terminal.exit();
        exit(0);
    }

    fn tui_loop(&mut self) {
        let mut center = Position::new_center();
        let message = String::from("Hello, world!");
        center.set_x(center.get_x() - message.chars().count() as u16 / 2);
        self.terminal.write_background(&center, message, &color::Blue);
        for key in Terminal::get_keys() {
            match key.unwrap() {
                Key::Char('q') => self.quit(),
                _ => {}
            }
        }
    }

    fn draw_background(&mut self) {
        let mut cursor = Position::new_origin();
        for x in 1..=self.max_x {
            for y in 1..=self.max_y {
                cursor.set(x, y);
                self.terminal
                    .draw_box(&cursor, &termion::color::LightMagenta);
            }
        }
    }
}

struct Widget {}