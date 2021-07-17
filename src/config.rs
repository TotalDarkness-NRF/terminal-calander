use std::{env,fs::File,io::{BufRead, BufReader}};

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

#[derive(Clone, Copy)]
pub struct Config {
    pub bg_color: AnsiValue,
    pub calendar_bg_color: AnsiValue,
    pub date_bg_color: AnsiValue,
    pub text_button_bg_color: AnsiValue,
    pub date_num_color: AnsiValue,
    pub month_text_color: AnsiValue,
    pub weekday_bg_color: AnsiValue,
    pub select_bg_date_color: AnsiValue,
    pub select_bg_text_button_color: AnsiValue,
    pub quit: Key,
    pub up: Key,
    pub left: Key,
    pub down: Key,
    pub right: Key,
    pub calendar_up: Key,
    pub calendar_left: Key,
    pub calendar_right: Key,
    pub calendar_down: Key,
    pub change_calendar_reset_cursor: bool,
    pub unselect_change_calendar_cursor: bool,
    // TODO have buttons to move calander right left etc
}

impl Config {
    pub fn get_config() -> Self {
        // TODO handle errors
        let mut config = Config::get_default_config();
        let file = env::current_exe().unwrap().parent().unwrap().join("config.txt"); 
        // TODO this path is dumb. Instead store it either in .cofing, in current_dir() or specified by user
        if !file.exists() { return config };
        let file = File::open(file).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap(); // TODO Ignore errors.
            let line = line.trim().to_lowercase();
            if line.starts_with('#') { continue }
            let split_index;
            match line.find('=') {
                Some(index) => split_index = index,
                None => continue,
            }
            let (config_var, value) = line.split_at(split_index);
            let value = value.replace(" ", "").replace("=", "");
            let config_var = config_var.trim();
            let value = value.trim();
            if config_var.contains("color") {
                match_colors(&mut config, config_var, value);
            }
        }
        config
    }

    fn get_default_config() -> Self {
        Config {
            bg_color: AnsiValue(12),
            calendar_bg_color: AnsiValue(7),
            date_bg_color: AnsiValue(0),
            text_button_bg_color: AnsiValue(6),
            date_num_color: AnsiValue(7),
            month_text_color: AnsiValue(7),
            weekday_bg_color: AnsiValue(9),
            select_bg_date_color: AnsiValue(5),
            select_bg_text_button_color: AnsiValue(13),
            quit: Key::Char('q'),
            up: Key::Char('w'),
            left: Key::Char('a'),
            down: Key::Char('s'),
            right: Key::Char('d'),
            calendar_up: Key::Char('W'),
            calendar_left: Key::Char('A'),
            calendar_down: Key::Char('S'),
            calendar_right: Key::Char('D'),
            change_calendar_reset_cursor: true,
            unselect_change_calendar_cursor: true, 
        }
    }
}

fn match_colors(config: &mut Config, config_var: &str, value: &str) {
    if let Some(value) = parse_color(value) {
        match config_var {
            "bg_color" => config.bg_color = value,
            "calendar_bg_color" => config.calendar_bg_color = value,
            "date_bg_color" => config.date_bg_color = value,
            "text_button_bg_color" => config.text_button_bg_color = value,
            "date_num_color" => config.date_num_color = value,
            "month_text_color" => config.month_text_color = value,
            "weekday_bg_color" => config.weekday_bg_color = value,
            "select_bg_date_color" => config.select_bg_date_color = value,
            "select_bg_text_button_color" => config.select_bg_text_button_color = value,
            _ => (),
        }
    }
}

fn parse_color(mut color_string: &str) -> Option<AnsiValue> {
    if color_string.starts_with("high-intensity") {
        color_string = color_string.split_at("high-intensity".len()).1;
    }
    match color_string {
        "black" | "0" => Some(AnsiValue(0)),
        "red" | "1" => Some(AnsiValue(1)),
        "green" | "2" => Some(AnsiValue(2)),
        "yellow" | "3" => Some(AnsiValue(3)),
        "blue" | "4" => Some(AnsiValue(4)),
        "magenta" | "5" => Some(AnsiValue(5)),
        "cyan" | "6" => Some(AnsiValue(6)),
        "white" | "7" => Some(AnsiValue(7)),
        "lightblack" | "8" => Some(AnsiValue(8)),
        "lightred" | "9" => Some(AnsiValue(9)),
        "lightgreen" | "10" => Some(AnsiValue(10)),
        "lightyellow" | "11" => Some(AnsiValue(11)),
        "lightblue" | "12" => Some(AnsiValue(12)),
        "lightmagenta" | "13" => Some(AnsiValue(13)),
        "lightcyan" | "14" => Some(AnsiValue(14)),
        "lightwhite" | "15" => Some(AnsiValue(15)),
        _ => None,
    }
}