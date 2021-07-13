mod terminal;
mod position;
mod calendar;
mod config;

use crate::calendar::Tui;

fn main() {
    Tui::new().start();
}