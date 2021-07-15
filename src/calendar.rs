use chrono::{Date, Datelike, Local, Weekday};

use crate::{config::Config, position::Position, terminal::Terminal, tui::{Button, ButtonType, Widget, WidgetType}};

pub struct Calendar {
    start_date: Date<Local>,
    position: Position,
    widgets: Vec<WidgetType>,
    pub cursor: usize,
}

impl Calendar {
    pub fn new(start_date: Date<Local>, position: Position) -> Self {
        Calendar { start_date, position, widgets: Vec::new() , cursor: 0 }
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
        let button = Button {
            button_data: ButtonType::TextButton(date.format("%B %Y").to_string()),
            start_position: self.position,
            end_position: Position::new(self.position.get_x() + 21, self.position.get_y()),
            color: config.date_bg_color,
        };
        self.widgets.push(WidgetType::Button(button));

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
        self.widgets.push(WidgetType::TextBox(position, weekdays, config.weekday_bg_color));
        self.widgets.reverse(); // Make the button widget first (too lazy to fix issue if it isn't)
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
            let button = Button {
                button_data: ButtonType::CalanderDate(date),
                start_position: position,
                end_position: Position::new(position.get_x() + 3, position.get_y()),
                color: config.date_bg_color,
            };
            self.widgets.push(WidgetType::Button(button));
            date = date.succ();
            if let Weekday::Sun = date.weekday() {
                position.set(self.position.get_x() + 1, position.get_y() + 2);
            } else {
                position.set_x(self.position.get_x() + 1 + 3 * (date.weekday().number_from_sunday() as u16 - 1));
            }
            
        }

        for widget in self.widgets.iter_mut() {
            match widget {
                WidgetType::Button(button) => button.draw(terminal),
                WidgetType::TextBox(pos, text, color) => 
                    terminal.write_background(*pos, text.to_string(), color),
            }
        }
    }

    pub fn select_button(&mut self, config: &mut Config, terminal: &mut Terminal, index_to: usize) {
        // TODO fix not being able to reselct calendar text buttons cause a non button is blocking
        if index_to >= self.widgets.len() { return; } // TODO maybe use directions later
        self.unselect_button(config, terminal);
        let iter = self.widgets.iter_mut().skip(index_to);
        for (i, widget) in iter.enumerate() {
            if let WidgetType::Button(button) = widget {
                button.color = 
                match button.button_data {
                    ButtonType::TextButton(_) => config.select_bg_text_button_color,
                    ButtonType::CalanderDate(_) => config.select_bg_date_color,
                };
                button.draw(terminal);
                self.cursor = i + index_to;
                break;
            }
        }
    }

    pub fn unselect_button(&mut self, config: &mut Config, terminal: &mut Terminal) {
        match self.widgets.get_mut(self.cursor) {
            Some(widget) => 
            match widget {
                WidgetType::Button(button) => {
                    button.color =
                    match button.button_data {
                        ButtonType::TextButton(_) => config.date_bg_color, // todo
                        ButtonType::CalanderDate(_) => config.date_bg_color,
                    };
                    button.draw(terminal);
                }
               
                WidgetType::TextBox(_, _, _) => (),
            },
            None => return,
        }
    }
}