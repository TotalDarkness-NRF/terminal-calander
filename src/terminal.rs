use std::{fs::File, io::Write};

use termion::{
    clear,
    color::{self, Color},
    cursor,
    input::{Keys, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen,
};

use crate::position::Position;

pub struct Terminal {
    terminal: RawTerminal<File>,
}

impl Terminal {
    pub fn get_raw() -> Self {
        Terminal {
            terminal: termion::get_tty().unwrap().into_raw_mode().unwrap(),
        }
    }

    pub fn get_keys() -> Keys<File> {
        termion::get_tty().unwrap().keys()
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
        if start < end {
            let mut cursor = Position::new_origin();
            for y in 1..=end.get_y() {
                for x in 1..=end.get_x() {
                    cursor.set(x, y);
                    self.draw_box(cursor, color);
                }
            }
        }
    }

    pub fn draw_char(&mut self, pos: Position, character: char) {
        self.restore_cursor_write(pos, String::from(character));
    }

    pub fn erase_box(&mut self, position: Position) {
        self.restore_cursor_write(position, " ".to_string());
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
}