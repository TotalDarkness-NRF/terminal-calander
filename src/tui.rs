use std::{sync::{Arc, Mutex, mpsc::{Receiver, Sender, channel}}, thread};

use chrono::{Date, Datelike, Local};
use termion::{color::AnsiValue, event::{Event, Key, MouseButton, MouseEvent}};

use crate::{calendar::Calendar, config::Config, position::{Direction, Position}, terminal::{Formatter, Terminal}};

pub struct Tui {
    bounds: Position,
    config: Config,
    terminal: Terminal,
    calendars: Vec<Calendar>,
    tx_mut: Arc<Mutex<Sender<Event>>>,
    rx: Receiver<Event>,
    quit: bool,
}

impl Tui {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        Tui {
            bounds: Terminal::get_boundaries(),
            config: Config::get_config(),
            terminal: Terminal::new_raw(),
            calendars: Vec::new(),
            tx_mut: Arc::new(Mutex::new(tx)), 
            rx,
            quit: false,
        }
    }

    pub fn start(&mut self) {
        self.terminal.begin();
        self.tui_loop();
    }

    fn init(&mut self, date: Date<Local>) {
        self.create_calendars(date);
        let format = self.draw_calendars();
        self.terminal.write_format(format);
    }

    fn tui_loop(&mut self) {
        self.init(Local::today().with_day(1).unwrap());
        self.terminal.mouse_terminal();
        let tx = self.tx_mut.clone();
        thread::spawn(move || {
            for key in Terminal::get_events() {
                // I use lock to stop this thread when I need to edit
                // If I let it keep sending it causes lag for editor input
                tx.lock().unwrap().send(key.unwrap()).unwrap();
            }
        });
        let mut calendar_index: usize = 0;
        while !self.quit {
            self.handle_event(&mut calendar_index);
            if self.bounds != Terminal::get_boundaries() {
                calendar_index = 0;
                self.reset(Local::today().with_day(1).unwrap());
            }
        }
        self.terminal.exit();
    }

    fn handle_event(&mut self, index: &mut usize) {
        if let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Key(key) => self.handle_key(key, index),
                Event::Mouse(mouse) => self.handle_mouse(mouse, index),
                Event::Unsupported(_) => (),
            };
        }
    }

    fn handle_key(&mut self, key: Key, index: &mut usize) {
        let config = &self.config;
        if key == config.quit {
            self.quit = true;
        } else if key == config.edit {
           self.edit();
        } else if key == config.go_back_time {
            self.reset(self.time_travel(Direction::Left));
        } else if key == config.go_forward_time {
            self.reset(self.time_travel(Direction::Right));
        } else if key == config.go_back_calendar {
            self.reset(self.time_travel(Direction::Down));
        } else if key == config.go_forward_calendar {
            self.reset(self.time_travel(Direction::Up));
        } else {
            let format =
            if key == config.left {
                self.calendars.get_mut(*index).unwrap()
                .move_cursor(&config, Direction::Left)
            } else if key == config.right {
                self.calendars.get_mut(*index).unwrap()
                .move_cursor(&config, Direction::Right)
            } else if key == config.up {
                self.calendars.get_mut(*index).unwrap()
                .move_cursor(&config, Direction::Up)
            } else if key == config.down {
                self.calendars.get_mut(*index).unwrap()
                .move_cursor(&config, Direction::Down)
            } else if key == config.calendar_left {
                self.move_calendar(index, Direction::Left)
            } else if key == config.calendar_right {
                self.move_calendar(index, Direction::Right)
            } else if key == config.calendar_up {
                self.move_calendar(index, Direction::Up)
            } else if key == config.calendar_down {
                self.move_calendar(index, Direction::Down)
            } else { return };
            self.terminal.write_format(format);
        }
    }

    fn edit(&mut self) {
        // TODO save to a file and the date
        self.terminal.reset();
        self.terminal.exit();
        self.terminal = Terminal::new(); // Drop the raw mode and mouse terminal
        // I decided to use the lock here to stop the other thread (causes lag to editor input)
        let lock = self.tx_mut.lock().unwrap();
        match edit::edit("") {
            Ok(edit) => edit,
            Err(_) => "".to_string(),
        };
        drop(lock); // Drop lock and allow use of another self mut refrence
        self.terminal = Terminal::new_raw();
        self.terminal.mouse_terminal();
        self.terminal.begin();
        let draw = self.draw_calendars();
        self.terminal.write_format(draw);
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
            let mut format = Formatter::new();
            for (calendar_index, calendar) in self.calendars.iter_mut().enumerate() {
                if !calendar.is_hovered(mouse_pos) { continue }
                for (i, button) in calendar.buttons.iter_mut().enumerate() {
                    if !button.is_hovered(mouse_pos) { continue }
                    if *index != calendar_index {
                        calendar_change = true;
                        future_index = calendar_index;
                    }
                    format += &calendar.select_button(&mut self.config, i);
                    break;
                }
            }

            if calendar_change { 
                let last_calendar = self.calendars.get_mut(*index).unwrap();
                if self.config.unselect_change_calendar_cursor || self.config.change_calendar_reset_cursor {
                    format += &last_calendar.unselect_button(&mut self.config);
                }
                if self.config.change_calendar_reset_cursor { last_calendar.cursor = 0; }  
                *index = future_index;  
            }
            self.terminal.write_format(format);
        }
    }

    fn draw_background(&mut self) -> Formatter {
        Formatter::new().
        create_box(&Position::new_origin(), &Position::new(self.bounds.get_x(), self.bounds.get_y()), &self.config.bg_color)
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
        let max = rows * columns;
        let mut vec = vec![Calendar::dummy(); max]; //fill up space
        let (tx, rx) = channel();
        let handles: Vec<_> = (0..threads).map(|_| {
            let config = self.config; // Auto cloned config
            let mutex = mutex.clone();
            let tx = tx.clone();
            thread::spawn(move || {
                loop {
                    let date;
                    let position;
                    let index;
                    { // Put in a new scope to force the lock drop and unlock for other threads
                        let mut lock  = mutex.lock().unwrap();
                        if lock.2 >= max { break; }
                        date = lock.0.clone();
                        position = lock.1.clone();
                        index = lock.2.clone();
                        lock.0 = (date + chrono::Duration::days(32)).with_day(1).unwrap();
                        lock.1.set_x(position.get_x() + 24);
                        lock.2 += 1;
                        if lock.2 % Tui::get_columns() == 0 { lock.1.set(1, position.get_y() + 14); }
                    }
                    tx.send((Calendar::new(date, position, &config), index)).unwrap();
                }
            })
        }).collect();

        for handle in handles {
            handle.join().unwrap();
        }

        for _ in 0..max {
            let (calendar, index) = rx.recv().unwrap();
            *vec.get_mut(index).unwrap() = calendar;
        }

        self.calendars = vec;
    }

    fn draw_calendars(&mut self) -> Formatter {
        let threads = self.config.max_threads;
        let threads: usize = if self.calendars.len() > threads { threads } else { self.calendars.len() };
        let (tx, rx) = channel();
        let mutex = Arc::new(Mutex::new(self.calendars.clone()));
        let handles: Vec<_> = (0..threads).map(|_| {
            let mutex = Arc::clone(&mutex);
            let tx = tx.clone();
            thread::spawn(move || {
                loop {
                    let mut calendar: Calendar;
                    {// Introduce scope to remove lock fast
                        match mutex.lock().unwrap().pop() {
                            Some(value) => calendar = value,
                            None => break,
                        };
                    }
                    tx.send(calendar.draw_format()).unwrap();
                }
            })
        }).collect();

        // Wait for all threads to finish
        for handle in handles {
            handle.join().unwrap();
        }

        // Collect thread messages
        let mut format = Formatter::new();
        for _ in 0..self.calendars.len() {
            format += &rx.recv().unwrap();
        }
        
        self.draw_background() + &format
    }

    fn move_calendar(&mut self, index: &mut usize, direction: Direction) -> Formatter {
        let change;
        let mut format = Formatter::new();
        match direction {
            Direction::Up | Direction::Down => {
                let x = Tui::get_columns();
                if let Direction::Up = direction {
                    if *index == 0 || *index < x { return format; }
                } else if *index + x >= self.calendars.len() { return format; }
                change = x;
            },
            _ => {
                if let Direction::Left = direction {
                    if *index == 0 { return format; }
                } else if *index + 1 >= self.calendars.len() { return format; }
                change = 1;
            },
        }
        let calendar = self.calendars.get_mut(*index).unwrap();
        if self.config.unselect_change_calendar_cursor || self.config.change_calendar_reset_cursor {
            format += &calendar.unselect_button(&mut self.config);
        }
        if matches!(direction, Direction::Down) || matches!(direction, Direction::Right) { 
            *index += change; 
        } else { *index -= change; }
        if self.config.change_calendar_reset_cursor { calendar.cursor = 0; }
        let calendar = self.calendars.get_mut(*index).unwrap();
        format + &calendar.select_button(&mut self.config, calendar.cursor)
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
    fn draw_format(&mut self) -> Formatter;
    fn action(&mut self);
    fn get_start(&self) -> Position;
    fn get_end(&self) -> Position;
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
    fn draw_format(&mut self) -> Formatter {
        match &self.button_data {
            ButtonType::TextButton(text) => self.draw_text_button(text),
            ButtonType::CalanderDate(date) => self.draw_calendar_date(date),
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
    fn draw_text_button(&self, text: &String) -> Formatter {
        let mut center_x: u16 = (self.end_position.get_x() + self.start_position.get_x()) / 2;
        let length: u16 = text.chars().count() as u16 / 2;
        if center_x >= length { center_x -= length; }
        let center_y: u16 = (self.end_position.get_y() + self.start_position.get_y()) / 2;
        Formatter::new()
        .create_box(&self.start_position, &self.end_position, &self.bg_color)
        .go_to(Position::new(center_x, center_y))
        .bg_color(&self.bg_color)
        .fg_color(&self.fg_color)
        .text(text.clone())
    }

    fn draw_calendar_date(&self, date: &Date<Local>) -> Formatter {
        Formatter::new()
        .go_to(self.start_position)
        .bg_color(&self.bg_color)
        .fg_color(&self.fg_color)
        .text(date.format("%e").to_string())
    }
}

pub struct TextBox {
    text: String,
    pub position: Position,
    color: AnsiValue,
}

impl TextBox {
    pub fn new(text: String, position: Position, color: AnsiValue) -> Self{
        TextBox { text, position, color }
    }
    pub fn draw_format(&self) -> Formatter {
        Formatter::new().go_to(self.position).bg_color(&self.color).text(self.text.clone())
    }
}