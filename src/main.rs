mod terminal;
mod position;
mod calendar;
mod config;

use crate::calendar::Tui;

fn main() {
    // TODO learn to use chrono
    // TODO make a tui using termion to make a calander
    Tui::new().start();
}