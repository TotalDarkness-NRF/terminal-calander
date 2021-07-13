use std::{env::current_exe,fs::File,io::{BufRead, BufReader}};

use termion::{color::AnsiValue, event::Key};

/*
Ansi value for color
Black, 0
Red, 1
Green, 2
Yellow, 3
Blue, 4
Magenta, 5
Cyan, 6
White, 7
LightBlack, 8
LightRed, 9
LightGreen, 10
LightYellow, 11
LightBlue, 12
LightMagenta, 13
LightCyan, 14
LightWhite, 15
*/

pub struct Config {
    pub bg_color: AnsiValue,
    pub calendar_bg_color: AnsiValue,
    pub date_bg_color: AnsiValue,
    pub date_num_color: AnsiValue,
    pub weekday_bg_color: AnsiValue,
    pub quit: Key,
}

impl Config {
    pub fn get_config() -> Self {
        // TODO handle errors
        let mut config = Config::get_default_config();
        let config_file = current_exe().unwrap().parent().unwrap().join("config.txt");
        let file = File::open(config_file).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap(); // TODO Ignore errors.
            let line = line.trim().to_ascii_lowercase();
            if line.starts_with('#') { continue }
            let split_index;
            match line.find('=') {
                Some(index) => split_index = index,
                None => continue,
            }
            let (config_var, value) = line.split_at(split_index);
            let value = value.trim().split_at(1).1.trim();
            match config_var.trim() {
                "bg_color" =>
                    config.bg_color =
                    parse_color(value)
                    .unwrap_or(config.bg_color),
                "calendar_bg_color" => 
                    config.calendar_bg_color =
                        parse_color(value)
                        .unwrap_or(config.calendar_bg_color),
                "date_bg_color" => 
                    config.date_bg_color =
                        parse_color(value)
                        .unwrap_or(config.date_bg_color),
                "date_num_color" => 
                    config.date_num_color =
                        parse_color(value)
                        .unwrap_or(config.date_num_color),
                "weekday_bg_color" => 
                    config.weekday_bg_color =
                        parse_color(value)
                        .unwrap_or(config.weekday_bg_color),
                        // TODO Keys
                _ => continue,
            }
        }
        config
    }

    fn get_default_config() -> Self {
        Config {
            bg_color: AnsiValue(12),
            calendar_bg_color: AnsiValue(7),
            date_bg_color: AnsiValue(0),
            date_num_color: AnsiValue(7),
            weekday_bg_color: AnsiValue(9),
            quit: Key::Char('q'),
        }
    }
}

fn parse_color(mut color_string: &str) -> Result<AnsiValue, ()> {
    if color_string.starts_with("high-intensity") {
        color_string = color_string.split_at("high-intensity".len()).1;
    }
    match color_string {
        "black" | "0" => Ok(AnsiValue(0)),
        "red" | "1" => Ok(AnsiValue(1)),
        "green" | "2" => Ok(AnsiValue(2)),
        "yellow" | "3" => Ok(AnsiValue(3)),
        "blue" | "4" => Ok(AnsiValue(4)),
        "magenta" | "5" => Ok(AnsiValue(5)),
        "cyan" | "6" => Ok(AnsiValue(6)),
        "white" | "7" => Ok(AnsiValue(7)),
        "lightblack" | "8" => Ok(AnsiValue(8)),
        "lightred" | "9" => Ok(AnsiValue(9)),
        "lightgreen" | "10" => Ok(AnsiValue(10)),
        "lightyellow" | "11" => Ok(AnsiValue(11)),
        "lightblue" | "12" => Ok(AnsiValue(12)),
        "lightmagenta" | "13" => Ok(AnsiValue(13)),
        "lightcyan" | "14" => Ok(AnsiValue(14)),
        "lightwhite" | "15" => Ok(AnsiValue(15)),
        _ => Err(()),
    }
}