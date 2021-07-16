use std::process::exit;

use chrono::{Date, Datelike, Local};
use termion::color::AnsiValue;

use crate::{calendar::Calendar, config::Config, position::{Direction, Position}, terminal::{Formatter, Terminal}};

pub struct Tui {
    max_x: u16,
    max_y: u16,
    config: Config,
    terminal: Terminal,
    calendars: Vec<Calendar>,
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
            if key == self.config.quit {
               self.quit();
            }
            else if key == self.config.left {
                let calendar =  self.calendars.get_mut(index).unwrap(); // TODO make sure we check for incorrect index
                calendar.move_cursor(&mut self.config, &mut self.terminal, Direction::Left);
            }
            else if key == self.config.right {
                let calendar = self.calendars.get_mut(index).unwrap();
                calendar.move_cursor(&mut self.config, &mut self.terminal, Direction::Right);
            }
            else if key == self.config.up {
                let calendar = self.calendars.get_mut(index).unwrap();
                calendar.move_cursor(&mut self.config, &mut self.terminal, Direction::Up);
            }
            else if key == self.config.down {
                let calendar = self.calendars.get_mut(index).unwrap();
                calendar.move_cursor(&mut self.config, &mut self.terminal, Direction::Down);
            }
            else if key == self.config.calendar_left {
                self.move_calendar(&mut index, Direction::Left);
            }
            else if key == self.config.calendar_right {
                self.move_calendar(&mut index, Direction::Right);
            }
            else if key == self.config.calendar_up {
                self.move_calendar(&mut index, Direction::Up);
            }
            else if key == self.config.calendar_down {
                self.move_calendar(&mut index, Direction::Down);            
            }
            self.terminal.flush();
        }
    }

    fn get_num_columns(&self) -> usize {
        if self.calendars.is_empty() { return 0; }
        let first_y = self.calendars.first().unwrap().get_position().get_y();
        for (i, calendar) in self.calendars.iter().enumerate() {
            if calendar.get_position().get_y() != first_y { return i };
        }
        self.calendars.len()
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
        loop {
            let calendar = Calendar::new(date, position);
            self.calendars.push(calendar);
            let month = date.month();
            while month == date.month() { date = date.succ() };
            position.set_x(position.get_x() + 24);
            if !position.clone().set_x(position.get_x() + 24) { // Check if future position will work
                if !position.set(1, position.get_y() + 14) || !position.clone().set_y(position.get_y() + 14) {
                     break;
                }
            }
        }
    }

    fn draw_calendars(&mut self) {
        for calendar in self.calendars.iter_mut() {
            calendar.draw(&mut self.terminal, &mut self.config);
        }
    }

    fn move_calendar(&mut self, index: &mut usize, direction: Direction) {
        let change;
        match direction {
            Direction::Up | Direction::Down => {
                let x = self.get_num_columns();
                if let Direction::Up = direction {
                    if *index == 0 || *index < x { return; }
                } else if *index + x >= self.calendars.len() { return; }
                change = x;
            },
            _ => {
                if let Direction::Left = direction {
                    if *index == 0 { return; }
                } else if *index + 1 >= self.calendars.len() { return; }
                change = 1;
            },
        }
        let calendar = self.calendars.get_mut(*index).unwrap();
        if self.config.unselect_change_calendar_cursor || self.config.change_calendar_reset_cursor {
             calendar.unselect_button(&mut self.config, &mut self.terminal);
        }
        if matches!(direction, Direction::Down) || matches!(direction, Direction::Right) { 
            *index += change; 
        } else { *index -= change; }
        if self.config.change_calendar_reset_cursor { calendar.cursor = 0; }
        let calendar = self.calendars.get_mut(*index).unwrap();
        calendar.select_button(&mut self.config, &mut self.terminal, calendar.cursor);
    }
}

pub trait Widget {
    fn is_hovered(&self, position: Position) -> bool {
        self.get_start().get_x() >= position.get_x()
        && self.get_start().get_y() >= position.get_y()
        && self.get_end().get_x() <= position.get_x()
        && self.get_end().get_y() <= position.get_y()
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
    pub bg_color: AnsiValue,
    pub fg_color: AnsiValue,
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
        let length: u16 = text.chars().count() as u16 / 2;
        let center_x: u16 = 
        if center_x >= length {
            center_x - length
        } else {
            center_x
        };
        let center_y: u16 = (self.end_position.get_y() + self.start_position.get_y()) / 2;
        terminal.draw_large_box(self.start_position, self.end_position, &self.bg_color);
        let format = Formatter::new()
        .go_to(Position::new(center_x, center_y))
        .bg_color(&self.bg_color)
        .fg_color(&self.fg_color)
        .text(text);
        terminal.write_format(format);
    }

    fn draw_calendar_date(&self, terminal: &mut Terminal, date: &Date<Local>) {
        let format = Formatter::new()
        .go_to(self.start_position)
        .bg_color(&self.bg_color)
        .fg_color(&self.fg_color)
        .text(date.format("%e").to_string());
        terminal.write_format(format);
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