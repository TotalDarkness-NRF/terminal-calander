use std::{env::current_exe,fs::File,io::{BufRead, BufReader}};

use termion::{color::{self, Color},event::Key};

pub struct Config { // TODO switch to Ansi Value or u8 (0 to 15)
    pub bg_color: Box<dyn Color>,
    pub calendar_bg_color: Box<dyn Color>,
    pub date_bg_color: Box<dyn Color>,
    pub date_num_color: Box<dyn Color>,
    pub weekday_bg_color: Box<dyn Color>,
    pub quit: Key,
}

impl Config {
    pub fn get_config() -> Self {
        // TODO handle errors
        let mut config = Config::get_default_config();
        let config_file = current_exe().unwrap().parent().unwrap().join("config.txt");
        //println!("{}", config.to_str().unwrap().to_string());
        let file = File::open(config_file).unwrap();
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap(); // Ignore errors.
            let line = line.trim().to_ascii_lowercase();
            if line.starts_with('#') {
                continue;
            }
            let mut split_index = 0;
            match line.find('=') {
                Some(index) => split_index = index,
                None => continue,
            }
            let (config_var, value) = line.split_at(split_index);
            let value = value.trim();
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
            bg_color: Box::new(color::LightBlue),
            calendar_bg_color: Box::new(color::White),
            date_bg_color: Box::new(color::Black),
            date_num_color: Box::new(color::White),
            weekday_bg_color: Box::new(color::Red),
            quit: Key::Char('q'),
        }
    }
}

fn parse_color(mut color_string: &str) -> Result<Box<dyn Color>, ()> {
    if color_string.starts_with("high-intensity") {
        color_string = color_string.split_at("high-intensity".len()).1;
    }

    match color_string {
        "black" | "0" => Ok(Box::new(color::Black)),
        "red" | "1" => Ok(Box::new(color::Red)),
        "green" | "2" => Ok(Box::new(color::Green)),
        "yellow" | "3" => Ok(Box::new(color::Yellow)),
        "blue" | "4" => Ok(Box::new(color::Blue)),
        "magenta" | "5" => Ok(Box::new(color::Magenta)),
        "cyan" | "6" => Ok(Box::new(color::Cyan)),
        "white" | "7" => Ok(Box::new(color::White)),
        "lightblack" | "8" => Ok(Box::new(color::Green)),
        "lightred" | "9" => Ok(Box::new(color::LightRed)),
        "lightgreen" | "10" => Ok(Box::new(color::LightGreen)),
        "lightyellow" | "11" => Ok(Box::new(color::LightYellow)),
        "lightblue" | "12" => Ok(Box::new(color::LightBlue)),
        "lightmagenta" | "13" => Ok(Box::new(color::LightMagenta)),
        "lightcyan" | "14" => Ok(Box::new(color::LightCyan)),
        "lightwhite" | "15" => Ok(Box::new(color::LightWhite)),
        _ => Err(()),
    }
}