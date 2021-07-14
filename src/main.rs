mod terminal;
mod position;
mod tui;
mod config;

use crate::tui::Tui;

fn main() {
    Tui::new().start();
}