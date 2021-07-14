use std::process::exit;

use chrono::{Date, Datelike, Local};
use termion::color::AnsiValue;

use crate::{calendar::Calendar, config::Config, position::Position, terminal::Terminal};

pub struct Tui {
    max_x: u16,
    max_y: u16,
    config: Config,
    terminal: Terminal,
    calendars: Vec<Calendar>, // TODO this might have to be for each individual calendar, instead have a vec of calendars
}

impl Tui {
    pub fn new() -> Self {
        let boundary = Terminal::get_boundaries();
        Tui {
            max_x: boundary.get_x(),
            max_y: boundary.get_y(),
            config: Config::get_config(),
            terminal: Terminal::get_raw(),
            calendars: Vec::new(),
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
        // TODO make calanders and add to our list. Then draw and keep track of the selected one
        //self.draw_calendar();
        //let mut cursor_index = self.select_button(0, 0).unwrap_or(0);
        self.create_calendars();
        self.draw_calendars();
        for key in Terminal::get_keys() {
            let key = key.unwrap();
            //let prev_cursor_pos = cursor_index;
            // TODO do selecting of dates on buttons!
            if key == self.config.quit {
                self.quit();
            }
            else if key == self.config.left {
                // allow for looping back left like right?
                //cursor_index = self.get_select_index(cursor_index, cursor_index - 1);
            }
            else if key == self.config.right {
                //cursor_index = self.get_select_index(cursor_index, cursor_index + 1);
            }

            /* 
            if let WidgetType::Button(button) = self.widgets.get_mut(prev_cursor_pos).unwrap() {
                button.draw(&mut self.terminal);
            }
            if let WidgetType::Button(button) = self.widgets.get_mut(cursor_index).unwrap() {
                button.draw(&mut self.terminal);
            }
            */
            self.terminal.flush();
        }
    }

    fn draw_background(&mut self) {
        self.terminal.draw_large_box(
            Position::new_origin(),
            Position::new(self.max_x, self.max_y),
            &self.config.bg_color,
        );
    }

    fn create_calendars(&mut self) {
        let calendar = Calendar::new(Local::today().with_day(1).unwrap(), Position::new_origin());
        self.calendars.push(calendar);
    }

    fn draw_calendars(&mut self) {
        for calendar in self.calendars.iter_mut() {
            calendar.draw(&mut self.terminal, &mut self.config);
        }
    }
}

pub trait Widget {
    fn is_hovered(&self, position: Position) -> bool {
        self.get_start() >= position && self.get_end() <= position
    }

    fn draw(&mut self, terminal: &mut Terminal);
    fn action(&mut self);
    fn get_start(&self) -> Position;
    fn get_end(&self) -> Position;
}

pub enum WidgetType {
    Button(Button),
    TextBox(Position, String, AnsiValue),
}

pub struct Button {
    pub button_data: ButtonType,
    pub start_position: Position,
    pub end_position: Position,
    pub color: AnsiValue,
}

impl Widget for Button {
    fn draw(&mut self, terminal: &mut Terminal) {
        match &self.button_data {
            ButtonType::_TextButton(text) => self.draw_text_button(terminal, text.to_string()),
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
        terminal.draw_large_box(self.start_position, self.end_position, &self.color);
        terminal.write_background(Position::new(center_x, center_y), text, &self.color);
    }

    fn draw_calendar_date(&self, terminal: &mut Terminal, date: &Date<Local>) {
        let date = if date.day() < 10 {
            format!(" {}", date.day().to_string())
        } else {
            date.day().to_string()
        };
        terminal.write_background(self.start_position, date, &self.color);
    }
}

pub enum ButtonType {
    _TextButton(String), // TODO
    CalanderDate(Date<Local>),
}