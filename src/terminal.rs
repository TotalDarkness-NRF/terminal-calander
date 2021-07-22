use std::{fs::File, io::Write, ops::{Add, AddAssign}};

use termion::{clear, color::{self, AnsiValue}, cursor, input::{Events, MouseTerminal, TermRead}, raw::{IntoRawMode, RawTerminal}, screen, style};

use crate::position::Position;

pub struct Terminal {
    terminal: File,
    raw: Option<RawTerminal<File>>,
    mouse_terminals: u8,
}

impl Terminal {
    pub fn new_raw() -> Self {
        let mut terminal = Terminal::new();
        terminal.raw = Some(Terminal::get_terminal().into_raw_mode().unwrap());
        terminal.raw.as_ref().unwrap().activate_raw_mode().unwrap();
        terminal
    }

    pub fn new() -> Self {
        Terminal {
            terminal: Terminal::get_terminal(),
            raw: None,
            mouse_terminals: 0,
        }
    }

    pub fn get_events() -> Events<File>{
        termion::get_tty().unwrap().events()
    }

    fn get_terminal() -> File {
        termion::get_tty().unwrap()
    }

    pub fn get_mouse_terminal(&mut self) -> Result<MouseTerminal<File>, ()> {
        if self.mouse_terminals >= 1 { Err(()) }
        else {
            self.mouse_terminals += 1;
            Ok(MouseTerminal::from(Terminal::get_terminal())) 
        }
    }

    pub fn write(&mut self, message: String) {
        write!(self.terminal, "{}", message).unwrap();
        self.terminal.flush().unwrap();
    }

    pub fn clear_all(&mut self) {
        self.write(format!("{}{}", clear::All, cursor::Goto::default()));
    }

    pub fn reset(&mut self) {
        self.write(format!("{}{}{}{}", cursor::Goto::default(), color::Bg(color::Reset), color::Fg(color::Reset), style::Reset));
        self.clear_all();
    }

    pub fn begin(&mut self) {
        self.write(format!(
            "{}{}{}",
            screen::ToAlternateScreen,
            clear::All,
            cursor::Hide
        ));
    }

    pub fn exit(&mut self) {
        self.write(format!("{}{}", cursor::Show, screen::ToMainScreen));
        if let Some(raw) = &self.raw { raw.suspend_raw_mode().unwrap(); }
    }

    pub fn get_boundaries() -> Position {
        let (x, y) = termion::terminal_size().unwrap();
        Position::new(x, y)
    }

    pub fn write_format(&mut self, format: Formatter) {
        if format.string.is_empty() { return; }
        self.write(format.string);
    }
}

pub struct Formatter {
    string: String,
}

impl Add<&Formatter> for Formatter {
    type Output = Formatter;
    fn add(mut self, other: &Formatter) -> Self {
        self.string += &other.string;
        self
    }
}

impl AddAssign<&Formatter> for Formatter {
    fn add_assign(&mut self, other: &Formatter) {
        self.string += &other.string;
    }
}

impl Formatter {
    pub fn new() -> Self {
        Formatter { string: String::new() }
    }

    pub fn bg_color(mut self, color: &AnsiValue) -> Self {
        self.string += &color::Bg(*color).to_string();
        self
    }

    pub fn fg_color(mut self, color: &AnsiValue) -> Self {
        self.string += &color::Fg(*color).to_string();
        self
    }

    pub fn go_to(mut self, position: Position) -> Self {
        self.string += &cursor::Goto(position.get_x(), position.get_y()).to_string();
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.string += &text;
        self
    }

    pub fn create_box(mut self, start: &Position, end: &Position, color: &AnsiValue) -> Self {
        self = self.bg_color(color);
        if start.get_x() <= end.get_x() && start.get_y() <= end.get_y() {
            let mut cursor = Position::new_origin();
            for y in start.get_y()..=end.get_y() {
                cursor.set(start.get_x(), y);
                self += &Formatter::new()
                .go_to(cursor)
                .text(format!("{:width$}", " ", width = (end.get_x() - start.get_x() + 1).into()));
            }
        }
        self
    }
}