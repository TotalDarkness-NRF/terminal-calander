use std::{sync::{Arc, Mutex, mpsc::{Receiver, channel}}, thread};

use chrono::{Date, Datelike, Local};
use termion::{color::AnsiValue, event::{Event, Key, MouseButton, MouseEvent}};

use crate::{calendar::Calendar, config::Config, position::{Direction, Position}, terminal::{Formatter, Terminal}};

pub struct Tui {
    bounds: Position,
    config: Config,
    terminal: Terminal,
    calendars: Vec<Calendar>,
    quit: bool,
}

impl Tui {
    pub fn new() -> Self {
        Tui {
            bounds: Terminal::get_boundaries(),
            config: Config::get_config(),
            terminal: Terminal::new_raw(),
            calendars: Vec::new(),
            quit: false,
        }
    }

    pub fn start(&mut self) {
        self.terminal.begin();
        self.tui_loop();
    }

    fn init(&mut self, date: Date<Local>) {
        self.draw_background();
        self.create_calendars(date);
        self.draw_calendars();
    }

    fn tui_loop(&mut self) {
        self.init(Local::today().with_day(1).unwrap());
        let (tx, rx) = channel();
        thread::spawn(move || {
            for key in Terminal::get_events() {
                tx.send(key.unwrap()).unwrap();
            }
        });
        // Below forces the capture of a mouse terminal and makes sure we dont drop it
        let mouse_terminal_hold = self.terminal.get_mouse_terminal();
        let mut calendar_index: usize = 0;
        while !self.quit {
            self.handle_event(&mut calendar_index, &rx);
            if self.bounds != Terminal::get_boundaries() {
                calendar_index = 0;
                self.reset(Local::today().with_day(1).unwrap());
            }
        }
        self.terminal.exit();
        drop(mouse_terminal_hold.unwrap()); // Force mouse terminal to stop
    }

    fn handle_event(&mut self, index: &mut usize, rx: &Receiver<Event>) {
        if let Ok(event) = rx.try_recv() {
            match event {
                Event::Key(key) => self.handle_key(key, index),
                Event::Mouse(mouse) => self.handle_mouse(mouse, index),
                Event::Unsupported(_) => (),
            }
            self.terminal.flush();
        }
    }

    fn handle_key(&mut self, key: Key, index: &mut usize) {
        let config = &self.config;
        if key == config.quit {
            self.quit = true;
         } else if key == config.left {
             self.calendars.get_mut(*index).unwrap()
             .move_cursor(&config, &mut self.terminal, Direction::Left);
         } else if key == config.right {
             self.calendars.get_mut(*index).unwrap()
             .move_cursor(&config, &mut self.terminal, Direction::Right);
         } else if key == config.up {
             self.calendars.get_mut(*index).unwrap()
             .move_cursor(&config, &mut self.terminal, Direction::Up);
         } else if key == config.down {
             self.calendars.get_mut(*index).unwrap()
             .move_cursor(&config, &mut self.terminal, Direction::Down);
         } else if key == config.calendar_left {
             self.move_calendar(index, Direction::Left);
         } else if key == config.calendar_right {
             self.move_calendar(index, Direction::Right);
         } else if key == config.calendar_up {
             self.move_calendar(index, Direction::Up);
         } else if key == config.calendar_down {
             self.move_calendar(index, Direction::Down);            
         } else if key == config.go_back_time {
             self.reset(self.time_travel(Direction::Left));
         } else if key == config.go_forward_time {
             self.reset(self.time_travel(Direction::Right));
         } else if key == config.go_back_calendar {
             self.reset(self.time_travel(Direction::Down));
         } else if key == config.go_forward_calendar {
             self.reset(self.time_travel(Direction::Up));
         }
    }

    fn time_travel(&self, direction: Direction) -> Date<Local> {
        if self.calendars.is_empty() { return Local::today().with_day(1).unwrap() }
        let date = self.calendars.first().unwrap().get_start_date();
        let other_date = self.calendars.last().unwrap().get_start_date();
        let result = 
        match direction {
            Direction::Left => {
                let other_date = other_date + chrono::Duration::days(5); // Skip a few days into that month.
                date + date.signed_duration_since(other_date)
            }, 
            Direction::Right => {
                let other_date = other_date + chrono::Duration::days(32); // Skip a month and a bit
                date + other_date.signed_duration_since(date)
            },
            Direction::Up => date + chrono::Duration::days(32),
            Direction::Down => date - chrono::Duration::days(5),
        }.with_day(1);

        match result {
            Some(value) => value,
            None => {
                if matches!(direction, Direction::Left) || matches!(direction, Direction::Down){
                    chrono::MIN_DATE.with_timezone(&Local)
                } else {
                    chrono::MAX_DATE.with_timezone(&Local)
                }
            },
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent, index: &mut usize) {
        if let MouseEvent::Press(mouse, x, y) = mouse {
            if !matches!(mouse, MouseButton::Left) { return; }
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

    fn draw_background(&mut self) {
        self.terminal.draw_large_box(
            Position::new_origin(),
            Position::new(self.bounds.get_x(), self.bounds.get_y()),
            &self.config.bg_color,
        );
    }

    pub fn create_calendars(&mut self, date: Date<Local>) {
        let columns = Tui::get_columns();
        let rows = Tui::get_rows();
        let threads = self.config.max_threads;
        let threads = 
        if rows > threads || columns > threads { threads } 
        else if rows >= columns { rows }
        else { columns };
        // Put all this here because they all relate and need to be in sync
        let mutex = Arc::new(Mutex::new((date, Position::new_origin(), 0)));
        let vec = vec![Calendar::dummy(&self.config); rows*columns];
        let mutex_vec = Arc::new(Mutex::new(vec));
        for _ in 0..threads {
            let config = self.config; // Auto cloned config
            let mutex = mutex.clone();
            let mutex_vec = mutex_vec.clone();
            thread::spawn(move || {
                Tui::thread_create_calendar(mutex, mutex_vec, config);
            }).join().unwrap();
        }

        if let Ok(lock) = Arc::try_unwrap(mutex_vec) {
            self.calendars = lock.into_inner().unwrap();
        }
    }

    fn get_columns() -> usize {
        let mut position = Position::new_origin();
        let mut count:  usize = 0;
        loop {
            if !position.set_x(position.get_x() + 23) {
                break;
            } else { count += 1 } 
            
            if !position.set_x(position.get_x() + 1) {
                break;
            }
        }
        count
    }

    fn get_rows() -> usize {
        let mut position = Position::new_origin();
        let mut count: usize = 0;
        loop {
            if !position.set_y(position.get_y() + 13) {
                break;
            } else { count += 1; }
            
            if !position.set_y(position.get_y() + 1) {
                break;
            }
        }
        count
    }

    fn thread_create_calendar(mutex: Arc<Mutex<(Date<Local>, Position, usize)>>, mutex_vec: Arc<Mutex<Vec<Calendar>>>, config: Config) {
        loop {
            let date;
            let position;
            let index;
            { // Put in a new scope to force the lock drop and unlock for other threads
                let mut lock  = mutex.lock().unwrap();
                if lock.2 >= mutex_vec.lock().unwrap().len() { break; } // Quick check then drop vec lock
                date = lock.0.clone();
                position = lock.1.clone();
                index = lock.2.clone();
                lock.0 = (date + chrono::Duration::days(32)).with_day(1).unwrap();
                lock.1.set_x(position.get_x() + 24);
                if (index + 1) % Tui::get_columns() == 0 { lock.1.set(1, position.get_y() + 14); }
                lock.2 += 1;
            }
            let calendar = Calendar::new(date, position, &config);
            { *mutex_vec.lock().unwrap().get_mut(index).unwrap() = calendar; }
        }
    }

    fn draw_calendars(&mut self) {
        let mut handles = Vec::new();
        let mutex = Arc::new(Mutex::new(self.calendars.clone()));
        let threads = self.config.max_threads;
        let threads: usize = if self.calendars.len() > threads { threads } else { self.calendars.len() };
        for _ in 0..threads {
            let mutex = Arc::clone(&mutex);
            let handle = thread::spawn(move || {
                loop {
                    let mut calendar: Calendar;
                    {// Introduce scope to remove lock fast
                        match mutex.lock().unwrap().pop() {
                            Some(value) => calendar = value,
                            None => break,
                        };
                    }
                    calendar.draw(&mut Terminal::new());
                }
            });
            handles.push(handle);
        }
        for handle in handles {
            handle.join().unwrap();
        }
    }

    fn move_calendar(&mut self, index: &mut usize, direction: Direction) {
        let change;
        match direction {
            Direction::Up | Direction::Down => {
                let x = Tui::get_columns();
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

    fn reset(&mut self, date: Date<Local>) {
        self.terminal.reset();
        self.bounds = Terminal::get_boundaries();
        self.config = Config::get_config();
        self.calendars.clear();
        self.init(date);
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