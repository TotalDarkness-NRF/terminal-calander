use chrono::{Date, Datelike, Local, Weekday};
use termion::color::AnsiValue;

use crate::{config::Config, position::{Direction, Position}, terminal::Terminal, tui::{Button, ButtonType, TextBox, Widget}};

#[derive(Clone)]
pub struct Calendar {
    start_date: Date<Local>,
    start: Position,
    end: Position,
    weekdays: TextBox, // TODO no point in saving this in each calendar
    pub buttons: Vec<Button>,
    pub cursor: usize,
    bg_color: AnsiValue
}

impl Calendar {
    pub fn new(start_date: Date<Local>, start: Position, config: &Config) -> Self {
        let mut calendar = Calendar {
            start_date,
            start,
            end: Position::new(start.get_x() + 21, start.get_y() + 12),
            weekdays: TextBox::new(get_weekdays(start_date), Position::new(start.get_x(), start.get_y() + 1), config.weekday_bg_color),
            buttons: Vec::new(),
            cursor: 0,
            bg_color: config.calendar_bg_color,
        };
        calendar.setup(config);
        calendar
    }

    pub fn dummy(config: &Config) -> Self {
        Calendar {
            start_date: Local::today(),
            start: Position::new_origin(),
            end: Position::new_origin(),
            weekdays: TextBox::new("".to_string(), Position::new_origin(), config.weekday_bg_color),
            buttons: Vec::new(),
            cursor: 0,
            bg_color: config.calendar_bg_color,
        }
    }

    fn setup(&mut self, config: &Config) {
        let mut  date = self.start_date.clone();
        // Make title bar button
        let button = Button {
            button_data: ButtonType::TextButton(date.format("%B %Y").to_string()),
            start_position: self.start,
            end_position: Position::new(self.start.get_x() + 21, self.start.get_y()),
            bg_color: config.text_button_bg_color,
            fg_color: config.month_text_color,
        };
        self.buttons.push(button);
        let mut position = self.start.clone();
        position.set(
            position.get_x() + 1 + 3 * (date.weekday().number_from_sunday() as u16 - 1),
            position.get_y() + 2,
        );

        // Count days in a month
        let days = {
            let mut date = date.clone();
            let mut days = 0;
            while self.start_date.month() == date.month() {
                days += 1;
                date = date.succ();
            }
            days
        };

        // Add days in a month
        for _ in 1..=days {
            let button = Button {
                button_data: ButtonType::CalanderDate(date),
                start_position: position,
                end_position: Position::new(position.get_x() + 1, position.get_y()),
                bg_color: config.date_bg_color,
                fg_color: config.date_num_color,
            };
            self.buttons.push(button);
            date = date.succ();
            let days_from_max = date.weekday().num_days_from_sunday() as u16;
            position.set(
                self.start.get_x() + 1 + 3 * days_from_max,
                position.get_y() + if days_from_max == 0 { 2 } else { 0 }
            );
        }
    }

    pub fn move_cursor(&mut self, config: &mut Config, terminal: &mut Terminal, direction: Direction) {
        let index_to = match direction {
            Direction::Up => 
                if self.cursor <= 7 { 0 } else { self.cursor - 7 },
            Direction::Down => 
                if self.cursor + 7 >= self.buttons.len() { self.buttons.len() - 1 } else { self.cursor + 7 },
            Direction::Left => 
                if self.cursor == 0 { 0 } else { self.cursor - 1 },
            Direction::Right =>
                if self.cursor + 1 >= self.buttons.len() { self.buttons.len() - 1 } else { self.cursor + 1 },
        };
        self.select_button(config, terminal, index_to);
    }

    pub fn select_button(&mut self, config: &mut Config, terminal: &mut Terminal, index_to: usize) {
        if index_to >= self.buttons.len() { return; }
        if self.cursor != index_to { self.unselect_button(config, terminal) };
        let button = self.buttons.iter_mut().skip(index_to).next();
        match button {
            Some(button) => {
                button.bg_color = match button.button_data {
                    ButtonType::TextButton(_) => config.select_bg_text_button_color,
                    ButtonType::CalanderDate(_) => config.select_bg_date_color,
                };
                button.draw(terminal);
                self.cursor = index_to;
            }
            None => (),
        }
    }

    pub fn unselect_button(&mut self, config: &mut Config, terminal: &mut Terminal) {
        match self.buttons.get_mut(self.cursor) {
            Some(button) => {
                button.bg_color = match button.button_data {
                    ButtonType::TextButton(_) => config.text_button_bg_color,
                    ButtonType::CalanderDate(_) => config.date_bg_color,
                };
                button.draw(terminal);
            }
            None => (),
        }
    }
}

impl Widget for Calendar {
    fn draw(&mut self, terminal: &mut Terminal) {
        terminal.draw_large_box(self.start, self.end, &self.bg_color);
        for button in self.buttons.iter_mut() {
            button.draw(terminal);
        }
        self.weekdays.draw(terminal);
    }

    fn action(&mut self) {
        todo!()
    }

    fn get_start(&self) -> Position {
        self.start
    }

    fn get_end(&self) -> Position {
        self.end
    }
}

fn get_weekdays(mut date: Date<Local>) -> String {
    while date.weekday() != Weekday::Sun { date = date.succ(); }
    let mut text = String::new();
    for _ in 1..=7 {
        text.push_str(date.format("%a").to_string().as_str());
        date = date.succ();
    }
    text.push(' ');
    text
}