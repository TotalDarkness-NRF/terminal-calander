use std::process::exit;

use chrono::{Date, Datelike, Local};
use termion::{color::{self, Color}, event::Key};

use crate::{position::Position, terminal::Terminal};

pub struct Tui {
    max_x: u16,
    max_y: u16,
    terminal: Terminal,
    widgets: Vec<WidgetType>,
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
        self.terminal.write_background(center, message, &color::Blue);
        for key in Terminal::get_keys() {
            self.draw_calander();
            match key.unwrap() {
                Key::Char('q') => self.quit(),
                _ => {}
            }
        }
    }

    fn draw_calander(&mut self) {
        self.terminal.draw_large_box(Position::new_origin(), Position::new(22, 11), &color::LightMagenta);
        let mut date = Local::now().date();
        let mut position = Position::new(2, 2);
        for i in 1..=7 {
            let button = Button {
                button_data: ButtonType::CalanderDate(date),
                start_position: position,
                end_position: Position::new(position.get_x() + 3, position.get_y()),
                color: Box::new(color::Black),
            };
            self.widgets.push(WidgetType::Button(button));
            date = date.succ();
            position.set_x(position.get_x() + 3);
        }
        
        for widget in self.widgets.iter_mut() {
            match widget {
                WidgetType::Button(button) => button.draw(&mut self.terminal),
                WidgetType::TextBox => todo!(),
            }
        }
    }

    fn draw_background(&mut self) {
        self.terminal.draw_large_box(Position::new_origin(), Position::new(self.max_x, self.max_y), &color::LightBlue);
    }
}

trait Widget {
    fn is_pressed(&self, x: u16, y: u16) -> bool {
        self.get_start().get_x() >= x && self.get_start().get_y() >= y
        && self.get_end().get_x()<= x && self.get_end().get_y() <= y
    }

    fn draw(&mut self, terminal: &mut Terminal);
    fn action(&mut self);
    fn get_start(&self) -> Position;
    fn get_end(&self) -> Position;
}

enum WidgetType {
    Button(Button),
    TextBox,
}

struct Button {
    button_data: ButtonType,
    start_position: Position,
    end_position: Position,
    color: Box<dyn Color>,
}

impl Widget for Button {
    fn draw(&mut self, terminal: &mut Terminal) {
        match &self.button_data {
            ButtonType::TextButton(text) => self.draw_text_button(terminal, text.to_string()),
            ButtonType::CalanderDate(date) => self.draw_calander_date(terminal, date),
        }
    }

    fn action(&mut self) {
        todo!()
    }

    fn get_start(&self) -> Position {
        self.start_position
    }

    fn get_end(&self) -> Position {
        self.end_position
    }    
}

impl Button {
    fn draw_text_button(&self, terminal: &mut Terminal, text: String) {
        let center_x: u16 = (self.end_position.get_x() + self.start_position.get_x()) / 2 + 1;
        let length: u16 = text.chars().count() as u16;
        let center_x: u16 = 
        if center_x >= length {
            center_x - text.chars().count() as u16 / 2
        } else {
            center_x
        };
        let center_y: u16 = (self.end_position.get_y() + self.start_position.get_y()) / 2;
        let color = self.color.as_ref();
        for mut y in self.start_position.get_y()..=self.end_position.get_y() {
            for x in self.start_position.get_x()..= self.end_position.get_x() {
                if y == center_y && x == center_x {
                    terminal.write_background(Position::new(center_x, center_y), text.to_string(), color);
                    // finish it
                    for x in x + length..=self.end_position.get_x() {
                        terminal.draw_box(Position::new(x, y), color);
                    }
                    y += 1;
                } else {
                    terminal.draw_box(Position::new(x, y), color);
                }
            }
        }
    }

    fn draw_calander_date(&self, terminal: &mut Terminal, date: &Date<Local>) {
        let date = if date.day() < 10 { format!(" {}", date) } else { date.day().to_string() };
        terminal.write_background(self.start_position, date, self.color.as_ref());
    }
}

enum ButtonType {
    TextButton(String),
    CalanderDate(Date<Local>),
}