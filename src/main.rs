mod terminal;
mod position;
mod tui;
mod config;

use chrono::{Date, DateTime, Local, Utc};

use crate::tui::Tui;

fn main() {
    // TODO learn to use chrono
    // TODO make a tui using termion to make a calander
    let utc: DateTime<Utc> = Utc::now();
    let local: DateTime<Local> = Local::now();
    let date: Date<Local> = local.date();
    println!("{}", utc);
    println!("{}", local);
    println!("{}", date);
    let tommorow = date.succ();
    println!("{}", tommorow);

    let mut tui = Tui::new();
    tui.start();
}