mod terminal;
mod position;
mod tui;
mod config;
mod calendar;

use crate::tui::Tui;

fn main() {
    Tui::new().start();
}