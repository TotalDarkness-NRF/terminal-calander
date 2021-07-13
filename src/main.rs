mod terminal;
mod position;
mod tui;
mod config;

use crate::tui::Tui;

fn main() {
    // TODO learn to use chrono
    // TODO make a tui using termion to make a calander
    Tui::new().start();
}