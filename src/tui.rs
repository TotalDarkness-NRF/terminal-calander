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
        self.create_calendars();
        self.draw_calendars();
        let mut index: usize = 0;
        for key in Terminal::get_keys() {
            let key = key.unwrap();
            //let prev_cursor_pos = cursor_index;
            // TODO do selecting of dates on buttons!
            if key == self.config.quit {
               self.quit();
            }
            else if key == self.config.left {
                let calendar =  self.calendars.get_mut(index).unwrap(); // TODO make sure we check for incorrect index
                if calendar.cursor > 0 {
                    calendar.select_button(&mut self.config, &mut self.terminal, calendar.cursor - 1);
                }
            }
            else if key == self.config.right {
                let calendar = self.calendars.get_mut(index).unwrap();
                calendar.select_button(&mut self.config, &mut self.terminal, calendar.cursor + 1);
            }
            else if key == self.config.calendar_left {
                if index > 0 {
                    let calendar = self.calendars.get_mut(index).unwrap();
                    calendar.unselect_button(&mut self.config, &mut self.terminal);
                    index -= 1;
                    // TODO maybe config option to remove curosr on that calendar?
                    if self.config.change_calendar_reset_cursor { calendar.cursor = 0 };
                    let calendar = self.calendars.get_mut(index).unwrap();
                    calendar.select_button(&mut self.config, &mut self.terminal, calendar.cursor);
                }
            }
            else if key == self.config.calendar_right {
                if index + 1 < self.calendars.len() {
                    let calendar = self.calendars.get_mut(index).unwrap();
                    calendar.unselect_button(&mut self.config, &mut self.terminal);
                    index += 1;
                    if self.config.change_calendar_reset_cursor { calendar.cursor = 0 };
                    let calendar = self.calendars.get_mut(index).unwrap();
                    calendar.select_button(&mut self.config, &mut self.terminal, calendar.cursor);
                }
            }
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
        let mut date = Local::today().with_day(1).unwrap();
        let mut position = Position::new_origin();
        for _ in 0..=3 {
        let calendar = Calendar::new(date, position);
        self.calendars.push(calendar);
        date = date.with_month(date.month() + 1).unwrap(); // TODO wont work if above 12, just for testing now
        position.set_x(position.get_x() + 24);
        }
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
    WriteBox(Position, AnsiValue),
}

pub struct Button {
    pub button_data: ButtonType,
    pub start_position: Position,
    pub end_position: Position,
    pub color: AnsiValue,
}

pub enum ButtonType {
    TextButton(String),
    CalanderDate(Date<Local>),
}

impl Widget for Button {
    fn draw(&mut self, terminal: &mut Terminal) {
        match &self.button_data {
            ButtonType::TextButton(text) => self.draw_text_button(terminal, text.to_string()),
            ButtonType::CalanderDate(date) => self.draw_calendar_date(terminal, date),
        }
    }

    fn action(&mut self) {
        todo!("Make button actions for date buttons and text buttons. Date buttons allow you to insert text, while text button give overview")
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
        let center_x: u16 = 
        if center_x >= length {
            center_x - text.chars().count() as u16 / 2
        } else {
            center_x
        };
        let center_y: u16 = (self.end_position.get_y() + self.start_position.get_y()) / 2;
        terminal.draw_large_box(self.start_position, self.end_position, &self.color);
        terminal.write_background(Position::new(center_x, center_y), text, &self.color);
    }

    fn draw_calendar_date(&self, terminal: &mut Terminal, date: &Date<Local>) {
        terminal.write_background(self.start_position, date.format("%e").to_string(), &self.color);
    }
}

pub struct TextBox {
    text: String,
    position: Position,
    color: AnsiValue,
}

impl TextBox {
    pub fn new(text: String, position: Position, color: AnsiValue) -> Self{
        TextBox { text, position, color }
    }
    pub fn draw(&self, terminal: &mut Terminal) {
        terminal.write_background(self.position, self.text.clone(), &self.color);
    }
}