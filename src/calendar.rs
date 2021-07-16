use chrono::{Date, Datelike, Local, Weekday};

use crate::{config::Config, position::{Direction, Position}, terminal::Terminal, tui::{Button, ButtonType, TextBox, Widget}};

pub struct Calendar {
    start_date: Date<Local>,
    position: Position,
    pub buttons: Vec<Button>,
    pub cursor: usize,
}

impl Calendar {
    pub fn new(start_date: Date<Local>, position: Position) -> Self {
        Calendar { start_date, position, buttons: Vec::new() , cursor: 0 }
    }

    pub fn draw(&mut self, terminal: &mut Terminal, config: &mut Config) {
        // Draw background of calendar
        terminal.draw_large_box(
            self.position,
            Position::new(self.position.get_x() + 21, self.position.get_y() + 12),
            &config.calendar_bg_color,
        );

        let mut date = self.start_date.clone();
        // Make title bar button
        let mut button = Button {
            button_data: ButtonType::TextButton(date.format("%B %Y").to_string()),
            start_position: self.position,
            end_position: Position::new(self.position.get_x() + 21, self.position.get_y()),
            bg_color: config.text_button_bg_color,
            fg_color: config.month_text_color,
        };
        button.draw(terminal);
        self.buttons.push(button);

        while date.weekday() != Weekday::Sun {
            date = date.pred();
        }
        // TODO maybe config should have a what is the first day
        
        // Draw calander weekdays
        let mut weekdays = String::new();
        for _ in 1..=7 {
            weekdays.push_str(date.format("%a").to_string().as_str());
            date = date.succ();
        }
        weekdays.push(' ');
        let mut position = self.position.clone();
        position.set_y(position.get_y() + 1);
        TextBox::new(weekdays, position, config.weekday_bg_color).draw(terminal);
        date = self.start_date.clone();
        position.set(self.position.get_x() + 1 + 3 * (date.weekday().number_from_sunday() as u16 - 1), position.get_y() + 1); // TODO make the x calc a function
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
        // Draw calendar dates
        for _ in 1..=days {
            let mut button = Button {
                button_data: ButtonType::CalanderDate(date),
                start_position: position,
                end_position: Position::new(position.get_x() + 1, position.get_y()),
                bg_color: config.date_bg_color,
                fg_color: config.date_num_color,
            };
            button.draw(terminal);
            self.buttons.push(button);
            date = date.succ();
            if let Weekday::Sun = date.weekday() {
                position.set(self.position.get_x() + 1, position.get_y() + 2);
            } else {
                position.set_x(self.position.get_x() + 1 + 3 * (date.weekday().number_from_sunday() as u16 - 1));
            }   
        }
    }

    pub fn move_cursor(&mut self, config: &mut Config, terminal: &mut Terminal, direction: Direction) {
        let index_to = match direction {
            Direction::Up => {
                if self.cursor <= 7 { 0 }
                else { self.cursor - 7 }
            },
            Direction::Down => {
                if self.cursor + 7 >= self.buttons.len() { self.buttons.len() - 1 }
                else { self.cursor + 7 }
            },
            Direction::Left => {
                if self.cursor == 0 { 0 }
                else { self.cursor - 1 }
            },
            Direction::Right => {
                if self.cursor + 1 >= self.buttons.len() { self.buttons.len() - 1 }
                else { self.cursor + 1 }
            },
        };
        self.select_button(config, terminal, index_to);
    }

    pub fn select_button(&mut self, config: &mut Config, terminal: &mut Terminal, index_to: usize) {
        if index_to >= self.buttons.len() { return; }
        if self.cursor != index_to { self.unselect_button(config, terminal) };
        let button = self.buttons.iter_mut().skip(index_to).next();
        match button {
            Some(button) => {
                button.bg_color = 
                match button.button_data {
                    ButtonType::TextButton(_) => config.select_bg_text_button_color,
                    ButtonType::CalanderDate(_) => config.select_bg_date_color,
                };
                button.draw(terminal);
                self.cursor = index_to;
            },
            None => (),
        }
    }

    pub fn unselect_button(&mut self, config: &mut Config, terminal: &mut Terminal) {
        match self.buttons.get_mut(self.cursor) {
            Some(button) => {
                button.bg_color =
                    match button.button_data {
                        ButtonType::TextButton(_) => config.text_button_bg_color,
                        ButtonType::CalanderDate(_) => config.date_bg_color,
                    };
                button.draw(terminal);
            },
            None => (),
        }
    }

    pub fn get_position(&self) -> &Position {
        &self.position
    }
}