use std::{process::exit, sync::{Arc, Mutex, mpsc::{Receiver, channel}}, thread, time::Duration};

use chrono::{Date, Datelike, Local};
use termion::{color::AnsiValue, event::{Event, Key, MouseButton, MouseEvent}};

use crate::{calendar::Calendar, config::Config, position::{Direction, Position}, terminal::{Formatter, Terminal}};

pub struct Tui {
    bounds: Position,
    config: Config,
    terminal: Terminal,
    calendars: Vec<Calendar>,
}

impl Tui {
    pub fn new() -> Self {
        Tui {
            bounds: Terminal::get_boundaries(),
            config: Config::get_config(),
            terminal: Terminal::get_raw(),
            calendars: Vec::new(),
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

    fn init(&mut self) {
        self.draw_background();
        self.create_calendars_threads();
    }

    fn tui_loop(&mut self) {
        self.init();
        let (tx, rx) = channel();
        thread::spawn(move || {
            for key in Terminal::get_events() {
                tx.send(key.unwrap()).unwrap();
            }
        });
        let mut calendar_index: usize = 0;
        loop {
            self.handle_event(&mut calendar_index, &rx);
            self.terminal.flush();
            if self.bounds != Terminal::get_boundaries() {
                calendar_index = 0;
                self.reset();
                thread::sleep(Duration::from_secs(1));
            }
        }
    }

    fn handle_event(&mut self, index: &mut usize, rx: &Receiver<Event>) {
        if let Ok(event) = rx.try_recv() {
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

    pub fn create_calendars_threads(&mut self) {
        let columns = {
            let mut position = Position::new_origin();
            loop {
                if !position.set_x(position.get_x() + 24) {
                    break;
                }
            }
            (position.get_x() as usize) / 24
        };
        let rows = {
            let mut position = Position::new_origin();
            loop {
                if !position.set_y(position.get_y() + 14) {
                    break;
                } 
            }
            (position.get_y() as usize) / 14
        };
        let threads = 
        if rows > 10 || columns > 10 { 10 } 
        else if rows >= columns { rows }
        else { columns };
        let mut handles = Vec::new();
        let date = Local::today().with_day(1).unwrap();
        let position = Position::new_origin();
        let vec = vec![Calendar::dummy(&self.config); rows*columns];
        let mutex = Arc::new(Mutex::new((date, position, 0, vec)));
        for _ in 0..threads {
            let config = self.config; // Auto cloned config
            let mutex = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                Tui::thread_create_calendar(mutex, config);
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
        
        if let Ok(lock) = Arc::try_unwrap(mutex) {
            self.calendars = lock.into_inner().unwrap().3;
        }
    }

    fn thread_create_calendar(mutex: Arc<Mutex<(Date<Local>, Position, usize, Vec<Calendar>)>>, config: Config) {
        loop {
            let date;
            let position;
            let index;
            { // Put in a new scope to force the lock drop and unlock for other threads
                let mut lock  = mutex.lock().unwrap();
                if lock.2 >= lock.3.len() { break; }
                date = lock.0.clone();
                position = lock.1.clone();
                index = lock.2.clone();
                lock.0 = (date + chrono::Duration::days(32)).with_day(1).unwrap();
                lock.1.set_x(position.get_x() + 24);
                if !lock.1.clone().set_x(lock.1.get_x() + 24) { lock.1.set(1, position.get_y() + 14); }
                lock.2 += 1;
            }
            let mut calendar = Calendar::new(date, position, &config);
            calendar.draw(&mut Terminal::get_raw());
            { *mutex.lock().unwrap().3.get_mut(index).unwrap() = calendar; }
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
        self.calendars.clear(); // TODO maybe dont clear but see how many we should add or remove and do so
        self.init();
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

#[derive(Clone)]
pub struct Button {
    pub button_data: ButtonType,
    pub start_position: Position,
    pub end_position: Position,
    pub bg_color: AnsiValue,
    pub fg_color: AnsiValue,
}

#[derive(Clone)]
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

#[derive(Clone)]
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