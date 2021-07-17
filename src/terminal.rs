use std::{fs::File, io::Write};

use termion::{clear, color::{self, Color}, cursor, input::{Events, MouseTerminal, TermRead}, raw::{IntoRawMode, RawTerminal}, screen, style};

use crate::position::Position;

pub struct Terminal {
    terminal: MouseTerminal<RawTerminal<File>>,
}

impl Terminal {
    pub fn get_raw() -> Self {
        Terminal {
            terminal: MouseTerminal::from(termion::get_tty().unwrap().into_raw_mode().unwrap()),
        }
    }

    pub fn get_events() -> Events<File>{
        termion::get_tty().unwrap().events()
    }

    pub fn write(&mut self, message: String) {
        write!(self.terminal, "{}", message).unwrap();
    }

    pub fn write_background(&mut self, pos: Position, message: String, color: &dyn Color) {
        self.restore_cursor_write(pos, format!("{}{}", color::Bg(color), message));
    }

    pub fn draw_box(&mut self, pos: Position, color: &dyn Color) {
        self.restore_cursor_write(pos, format!("{} ", color::Bg(color)));
    }

    pub fn draw_large_box(&mut self, start: Position, end: Position, color: &dyn Color) {
        if start.get_x() <= end.get_x() && start.get_y() <= end.get_y() {
            let mut cursor = Position::new_origin();
            for y in start.get_y()..=end.get_y() {
                for x in start.get_x()..=end.get_x() {
                    cursor.set(x, y);
                    self.draw_box(cursor, color);
                }
            }
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

    pub fn _clear_all(&mut self) {
        self.write(format!("{}{}", clear::All, cursor::Goto::default()));
    }

    pub fn reset(&mut self) {
        self.write(format!("{}{}{}{}", cursor::Goto::default(), color::Bg(color::Reset), color::Fg(color::Reset), style::Reset));
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
        self.terminal.suspend_raw_mode().unwrap();
        drop(self);
    }

    pub fn get_boundaries() -> Position {
        let (x, y) = termion::terminal_size().unwrap();
        Position::new(x, y)
    }

    pub fn write_format(&mut self, format: Formatter) {
        self.write(format.get_string_format());
    }
}

#[derive(Clone)]
pub struct Formatter {
    string: String,
}

impl Formatter {
    pub fn new() -> Self {
        Formatter { string: String::new() }
    }

    pub fn bg_color(mut self, color: &dyn Color) -> Self {
        self.string.push_str(color::Bg(color).to_string().as_str());
        self
    }

    pub fn fg_color(mut self, color: &dyn Color) -> Self {
        self.string.push_str(color::Fg(color).to_string().as_str());
        self
    }

    pub fn text(mut self, text: String) -> Self {
        self.string.push_str(text.as_str());
        self
    }

    pub fn go_to(mut self, position: Position) -> Self {
        self.string.push_str(cursor::Goto(position.get_x(), position.get_y()).to_string().as_str());
        self
    }

    pub fn get_string_format(&self) -> String {
        self.string.clone()
    }
}