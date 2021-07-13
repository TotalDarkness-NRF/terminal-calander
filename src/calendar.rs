use std::process::exit;

use chrono::{Date, Datelike, Local, Weekday};
use termion::color::{self, Color};

use crate::{config::Config, position::Position, terminal::Terminal};

pub struct Tui {
    max_x: u16,
    max_y: u16,
    config: Config,
    terminal: Terminal,
    widgets: Vec<WidgetType>,
}

impl Tui {
    pub fn new() -> Self {
        let boundary = Terminal::get_boundaries();
        Tui {
            max_x: boundary.get_x(),
            max_y: boundary.get_y(),
            config: Config::get_config(),
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
        self.draw_calendar();
        for key in Terminal::get_keys() {
            let key = key.unwrap();
            if key == self.config.quit {
                self.quit();
            }
            self.terminal.flush();
        }
    }

    fn draw_calendar(&mut self) {
        // Ok this works now make it more generic for any month
        // TODO draw a proper calander
        // Draw background of calander
        self.terminal.draw_large_box(
            Position::new_origin(),
            Position::new(22, 11),
            self.config.calendar_bg_color.as_ref(),
        );

        let mut date = Local::now().date();

        // Make first day sunday
        // TODO maybe config should have a what is the first day
        while date.weekday() != Weekday::Sun {
            date = date.pred();
        }
        // Draw calander weekdays
        let mut temp_date = date.clone();
        let mut position = Position::new(1, 1);
        for _ in 1..=7 {
            self.widgets.push(WidgetType::TextBox(position, temp_date.format("%a").to_string(), Box::new(color::Red))); // Todo use weekend config
            temp_date = temp_date.succ();
            position.set_x(position.get_x() + 3);
        }

        position.set(2, 2);
        // Draw calander dates
        for _ in 1..=30 {
            let button = Button {
                button_data: ButtonType::CalanderDate(date),
                start_position: position,
                end_position: Position::new(position.get_x() + 3, position.get_y()),
                color: Box::new(color::Black),
            };
            self.widgets.push(WidgetType::Button(button));
            date = date.succ();
            if let Weekday::Sun = date.weekday() {
                position.set(2, position.get_y() + 2);
            } else {
                position.set_x(position.get_x() + 3);
            }
            
        }

        for widget in self.widgets.iter_mut() {
            match widget {
                WidgetType::Button(button) => button.draw(&mut self.terminal),
                WidgetType::TextBox(pos, text, color) => 
                    self.terminal.write_background(*pos, text.to_string(), color.as_ref()),
            }
        }
    }

    fn draw_background(&mut self) {
        self.terminal.draw_large_box(
            Position::new_origin(),
            Position::new(self.max_x, self.max_y),
            self.config.bg_color.as_ref(),
        );
    }
}

trait Widget {
    fn is_pressed(&self, position: Position) -> bool {
        self.get_start() >= position && self.get_end() <= position
    }

    fn draw(&mut self, terminal: &mut Terminal);
    fn action(&mut self);
    fn get_start(&self) -> Position;
    fn get_end(&self) -> Position;
}

enum WidgetType {
    Button(Button),
    TextBox(Position, String, Box<dyn Color>),
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
            ButtonType::CalanderDate(date) => self.draw_calendar_date(terminal, date),
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
        let center_x: u16 = (self.end_position.get_x() + self.start_position.get_x()) / 2;
        let length: u16 = text.chars().count() as u16;
        let center_x: u16 = if center_x >= length {
            center_x - text.chars().count() as u16 / 2
        } else {
            center_x
        };
        let center_y: u16 = (self.end_position.get_y() + self.start_position.get_y()) / 2;
        terminal.draw_large_box(self.start_position, self.end_position, self.color.as_ref());
        terminal.write_background(Position::new(center_x, center_y), text, self.color.as_ref());
    }

    fn draw_calendar_date(&self, terminal: &mut Terminal, date: &Date<Local>) {
        let date = if date.day() < 10 {
            format!(" {}", date.day().to_string())
        } else {
            date.day().to_string()
        };
        terminal.write_background(self.start_position, date, self.color.as_ref());
    }
}

enum ButtonType {
    TextButton(String),
    CalanderDate(Date<Local>),
}