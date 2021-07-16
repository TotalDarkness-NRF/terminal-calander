use std::{process::exit, sync::mpsc::{Receiver, channel}, thread};

use chrono::{Date, Datelike, Local};
use termion::{color::AnsiValue, event::{Event, Key, MouseButton, MouseEvent}};

use crate::{calendar::Calendar, config::Config, position::{Direction, Position}, terminal::{Formatter, Terminal}};

pub struct Tui {
    bounds: Position,
    config: Config,
    terminal: Terminal,
    calendars: Vec<Calendar>,
    events: Receiver<Event>,
}

impl Tui {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        thread::spawn(move || {
            for key in Terminal::get_events() {
                tx.send(key.unwrap()).unwrap();
            }
        });
        Tui {
            bounds: Terminal::get_boundaries(),
            config: Config::get_config(),
            terminal: Terminal::get_raw(),
            calendars: Vec::new(),
            events: rx,
        }
    }

    pub fn start(&mut self) {
        self.terminal.begin();
        self.tui_loop();
    }

    fn quit(&mut self) {
        self.terminal.exit();
        exit(0);
    }

    fn tui_loop(&mut self) {
        self.draw_background();
        self.create_calendars();
        self.draw_calendars();
        let mut calendar_index: usize = 0;
        loop {
            self.handle_event(&mut calendar_index);
            self.terminal.flush();
            if self.bounds != Terminal::get_boundaries() {
                calendar_index = 0;
                self.reset();
            }
        }
    }

    fn handle_event(&mut self, index: &mut usize) {
        if let Ok(event) = self.events.try_recv() {
            match event {
                Event::Key(key) => self.handle_key(key, index),
                Event::Mouse(mouse) => self.handle_mouse(mouse, index),
                Event::Unsupported(_) => (),
            }
        }
    }

    fn handle_key(&mut self, key: Key, index: &mut usize) {
        if key == self.config.quit {
            self.quit();
         } else if key == self.config.left {
             self.calendars.get_mut(*index).unwrap()  // TODO make sure we check for incorrect index
             .move_cursor(&mut self.config, &mut self.terminal, Direction::Left);
         } else if key == self.config.right {
             self.calendars.get_mut(*index).unwrap()
             .move_cursor(&mut self.config, &mut self.terminal, Direction::Right);
         } else if key == self.config.up {
             self.calendars.get_mut(*index).unwrap()
             .move_cursor(&mut self.config, &mut self.terminal, Direction::Up);
         } else if key == self.config.down {
             self.calendars.get_mut(*index).unwrap()
             .move_cursor(&mut self.config, &mut self.terminal, Direction::Down);
         } else if key == self.config.calendar_left {
             self.move_calendar(index, Direction::Left);
         } else if key == self.config.calendar_right {
             self.move_calendar(index, Direction::Right);
         } else if key == self.config.calendar_up {
             self.move_calendar(index, Direction::Up);
         } else if key == self.config.calendar_down {
             self.move_calendar(index, Direction::Down);            
         }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, index: &mut usize) {
        if let MouseEvent::Press(mouse, x, y) = mouse {
            if !matches!(mouse, MouseButton::Left) { return }
            let mut calendar_change = false;
            let mut future_index = self.calendars.len() + 1;
            let mouse_pos = Position::new(x, y);
            for (calendar_index, calendar) in self.calendars.iter_mut().enumerate() {
                if calendar.is_hovered(mouse_pos) {
                    for (i, button) in calendar.buttons.iter_mut().enumerate() {
                        if button.is_hovered(mouse_pos) {
                            if *index != calendar_index {
                                calendar_change = true;
                                future_index = calendar_index;
                            }
                            calendar.select_button(&mut self.config, &mut self.terminal, i);
                            break;
                        }
                    }
                }
            }

            if !calendar_change { return; }
            let last_calendar = self.calendars.get_mut(*index).unwrap();
            if self.config.unselect_change_calendar_cursor || self.config.change_calendar_reset_cursor {
                last_calendar.unselect_button(&mut self.config, &mut self.terminal);
            }
            if self.config.change_calendar_reset_cursor { last_calendar.cursor = 0; }  
            *index = future_index;  
        }
    }

    fn get_num_columns(&self) -> usize {
        if self.calendars.is_empty() { return 0; }
        let first_y = self.calendars.first().unwrap().get_start().get_y();
        for (i, calendar) in self.calendars.iter().enumerate() {
            if calendar.get_start().get_y() != first_y { return i };
        }
        self.calendars.len()
    }

    fn draw_background(&mut self) {
        self.terminal.draw_large_box(
            Position::new_origin(),
            Position::new(self.bounds.get_x(), self.bounds.get_y()),
            &self.config.bg_color,
        );
    }

    fn create_calendars(&mut self) {
        let mut date = Local::today().with_day(1).unwrap();
        let mut position = Position::new_origin();
        loop { // TODO config option for max amount of calendars
            let calendar = Calendar::new(date, position, &self.config);
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
            calendar.draw(&mut self.terminal);
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

    fn reset(&mut self) {
        self.terminal.reset();
        self.bounds = Terminal::get_boundaries();
        self.config = Config::get_config();
        self.calendars.clear();
        self.draw_background();
        self.create_calendars();
        self.draw_calendars();
    }
}

pub trait Widget {
    fn is_hovered(&self, position: Position) -> bool {
        position.get_x() >= self.get_start().get_x() && position.get_y() >= self.get_start().get_y()
        && position.get_x() <= self.get_end().get_x() && position.get_y() <= self.get_end().get_y()
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