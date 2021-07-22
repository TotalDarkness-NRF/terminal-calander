use std::{fs::File, io::Write};

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
    }

    pub fn write_background(&mut self, pos: Position, message: String, color: &AnsiValue) {
        self.restore_cursor_write(pos, format!("{}{}", color::Bg(*color), message));
    }

    pub fn draw_large_box(&mut self, start: Position, end: Position, color: &AnsiValue) {
        if start.get_x() <= end.get_x() && start.get_y() <= end.get_y() {
            let mut format = Formatter::new().bg_color(color);
            let mut cursor = Position::new_origin();
            for y in start.get_y()..=end.get_y() {
                cursor.set(start.get_x(), y);
                format = format
                .go_to(cursor)
                .text(format!("{:width$}", " ", width = (end.get_x() - start.get_x() + 1).into()));
            }
            self.write_format(format);
        }
    }

    pub fn restore_cursor_write(&mut self, pos: Position, message: String) {
        if pos.is_in_boundary() {
            self.write(format!(
                "{}{}{}",
                cursor::Save,
                cursor::Goto(pos.get_x(), pos.get_y()),
                message
            ));
            self.write(cursor::Restore.to_string());
        }
    }

    pub fn clear_all(&mut self) {
        self.write(format!("{}{}", clear::All, cursor::Goto::default()));
    }

    pub fn reset(&mut self) {
        self.write(format!("{}{}{}{}", cursor::Goto::default(), color::Bg(color::Reset), color::Fg(color::Reset), style::Reset));
        self.clear_all();
    }

    pub fn flush(&mut self) {
        self.terminal.flush().unwrap();
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
        self.write(format.string);
    }
}

pub struct Formatter {
    string: String,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter { string: String::new() }
    }

    pub fn bg_color(mut self, color: &AnsiValue) -> Self {
        self.string += color::Bg(*color).to_string().as_str();
        self
    }

    pub fn fg_color(mut self, color: &AnsiValue) -> Self {
        self.string += color::Fg(*color).to_string().as_str();
        self
    }

    pub fn go_to(mut self, position: Position) -> Self {
        self.string += cursor::Goto(position.get_x(), position.get_y()).to_string().as_str();
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.string += text.as_str();
        self
    }
}