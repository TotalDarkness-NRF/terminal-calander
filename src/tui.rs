use std::process::exit;

use chrono::{Date, Datelike, Local, Weekday};
use termion::color::AnsiValue;

use crate::{config::Config, position::Position, terminal::Terminal};

pub struct Tui {
    max_x: u16,
    max_y: u16,
    config: Config,
    terminal: Terminal,
    widgets: Vec<WidgetType>, // TODO this might have to be for each individual calendar, instead have a vec of calendars
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
        let mut cursor_index = self.select_button(0, 0).unwrap_or(0);
        for key in Terminal::get_keys() {
            let key = key.unwrap();
            let prev_cursor_pos = cursor_index;
            // TODO do selecting of dates on buttons!
            if key == self.config.quit {
                self.quit();
            }
            else if key == self.config.left {
                // allow for looping back left like right?
                cursor_index = self.get_select_index(cursor_index, cursor_index - 1);
            }
            else if key == self.config.right {
                cursor_index = self.get_select_index(cursor_index, cursor_index + 1);
            }

            if let WidgetType::Button(button) = self.widgets.get_mut(prev_cursor_pos).unwrap() {
                button.draw(&mut self.terminal);
            }
            if let WidgetType::Button(button) = self.widgets.get_mut(cursor_index).unwrap() {
                button.draw(&mut self.terminal);
            }
            self.terminal.flush();
        }
    }

    fn draw_calendar(&mut self) {
        // Ok this works now make it more generic for any month
        // TODO draw a proper calendar
        // Draw background of calendar
        self.terminal.draw_large_box(
            Position::new_origin(),
            Position::new(22, 11),
            &self.config.calendar_bg_color,
        );

        let mut date = Local::today();
        while date.weekday() != Weekday::Sun {
            date = date.pred();
        }
        // TODO maybe config should have a what is the first day
        
        // Draw calander weekdays
        let mut position = Position::new(1, 1);
        for _ in 1..=7 {
            self.widgets.push(WidgetType::TextBox(position, date.format("%a").to_string(), self.config.weekday_bg_color));
            date = date.succ();
            position.set_x(position.get_x() + 3);
        }
        date = date.with_day(1).unwrap();
        position.set(2 + 3 * (date.weekday().number_from_sunday() as u16 - 1), 2); // TODO make the x calc a function
        // Count days in a month
        let days = {
            let mut date = date.clone();
            let mut days = 0;
            let month = date.month();
            while month == date.month() {
                days += 1;
                date = date.succ();
            }
            days
        };
        // Draw calendar dates
        for _ in 1..=days {
            let button = Button {
                button_data: ButtonType::CalanderDate(date),
                start_position: position,
                end_position: Position::new(position.get_x() + 3, position.get_y()),
                color: self.config.date_bg_color,
            };
            self.widgets.push(WidgetType::Button(button));
            date = date.succ();
            if let Weekday::Sun = date.weekday() {
                position.set(2, position.get_y() + 2);
            } else {
                position.set_x(2 + 3 * (date.weekday().number_from_sunday() as u16 - 1));
            }
            
        }

        for widget in self.widgets.iter_mut() {
            match widget {
                WidgetType::Button(button) => button.draw(&mut self.terminal),
                WidgetType::TextBox(pos, text, color) => 
                    self.terminal.write_background(*pos, text.to_string(), color),
            }
        }
    }

    fn draw_background(&mut self) {
        self.terminal.draw_large_box(
            Position::new_origin(),
            Position::new(self.max_x, self.max_y),
            &self.config.bg_color,
        );
    }

    fn select_button(&mut self, index_from: usize, index_to: usize) -> Option<usize> {
        // TODO draw buttons here maybe. Also would have to pass terminal here
        match self.widgets.get_mut(index_from) {
            Some(widget) => 
            match widget {
                WidgetType::Button(button) => 
                match button.button_data {
                    ButtonType::_TextButton(_) => todo!("Set config for text buttons"),
                    ButtonType::CalanderDate(_) => button.color = self.config.date_bg_color,
                }
                WidgetType::TextBox(_, _, _) => (),
            },
            None => return None,
        }
        let iter = self.widgets.iter_mut();
        for (i, widget) in iter.skip(index_to).enumerate() {
            if let WidgetType::Button(button) = widget {
                button.color = self.config.select_bg_color;
                return Some(i + index_to);
            }
        }
        None
    }

    fn get_select_index(&mut self, index_from: usize, index_to: usize) -> usize {
        match self.select_button(index_from, index_to) {
            Some(value) => value,
            None => self.select_button(0, 0).unwrap_or(0),
        }
    }
}

trait Widget {
    fn is_hovered(&self, position: Position) -> bool {
        self.get_start() >= position && self.get_end() <= position
    }

    fn draw(&mut self, terminal: &mut Terminal);
    fn action(&mut self);
    fn get_start(&self) -> Position;
    fn get_end(&self) -> Position;
}

enum WidgetType {
    Button(Button),
    TextBox(Position, String, AnsiValue),
}

struct Button {
    button_data: ButtonType,
    start_position: Position,
    end_position: Position,
    color: AnsiValue,
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

enum ButtonType {
    _TextButton(String), // TODO
    CalanderDate(Date<Local>),
}