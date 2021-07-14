use chrono::{Date, Datelike, Local, Weekday};

use crate::{config::Config, position::Position, terminal::Terminal, tui::{Button, ButtonType, Widget, WidgetType}};

pub struct Calendar {
    start_date: Date<Local>,
    position: Position,
    widgets: Vec<WidgetType>,
    cursor: usize,
}

impl Calendar {
    pub fn new(start_date: Date<Local>, position: Position) -> Self {
        Calendar { start_date, position, widgets: Vec::new() , cursor: 0 }
    }

    pub fn draw(&mut self, terminal: &mut Terminal, config: &mut Config) {
        // TODO draw a proper calendar
        // Draw background of calendar
        terminal.draw_large_box(
            self.position,
            Position::new(self.position.get_x() + 21, self.position.get_y() + 11),
            &config.calendar_bg_color,
        );

        let mut date = Local::today();
        while date.weekday() != Weekday::Sun {
            date = date.pred();
        }
        // TODO maybe config should have a what is the first day
        
        // Draw calander weekdays
        let mut position = self.position;
        let mut weekdays = String::new();
        for _ in 1..=7 {
            weekdays.push_str(date.format("%a").to_string().as_str());
            date = date.succ();
        }
        weekdays.push(' ');
        self.widgets.push(WidgetType::TextBox(position, weekdays, config.weekday_bg_color));
        date = self.start_date.clone();
        position.set(self.position.get_x() + 1 + 3 * (date.weekday().number_from_sunday() as u16 - 1), self.position.get_y() + 1); // TODO make the x calc a function
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

    fn select_button(&mut self, config: &mut Config, index_from: usize, index_to: usize) -> Option<usize> {
        // TODO change cursor value here and then change the buttons colors here as well
        // Or we can return the previous button or change buttons later?
        // TODO draw buttons here maybe. Also would have to pass terminal here
        match self.widgets.get_mut(index_from) {
            Some(widget) => 
            match widget {
                WidgetType::Button(button) => 
                match button.button_data {
                    ButtonType::_TextButton(_) => todo!("Set config for text buttons"),
                    ButtonType::CalanderDate(_) => button.color = config.date_bg_color,
                }
                WidgetType::TextBox(_, _, _) => (),
            },
            None => return None,
        }
        let iter = self.widgets.iter_mut();
        for (i, widget) in iter.skip(index_to).enumerate() {
            if let WidgetType::Button(button) = widget {
                button.color = config.select_bg_color;
                return Some(i + index_to);
            }
        }
        None
    }

    fn get_select_index(&mut self, config: &mut Config, index_from: usize, index_to: usize) -> usize {
        // TODO we might not need this we could change directly in select button the values
        match self.select_button(config, index_from, index_to) {
            Some(value) => value,
            None => self.select_button(config, 0, 0).unwrap_or(0),
        }
    }
}