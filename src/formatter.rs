// Works in conjunction with ColoredString crate, for now...
use std::str::FromStr;

use colored::{ColoredString, Colorize};
use serde::Deserialize;

use crate::config_manager::Configuration;

// This is a hack to get the color deserializaton working
// Essentially it uses my own enum, and to print it you need to call color_string
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum CrabFetchColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Clear
}
impl FromStr for CrabFetchColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "black" => Ok(CrabFetchColor::Black),
            "red" => Ok(CrabFetchColor::Red),
            "green" => Ok(CrabFetchColor::Green),
            "yellow" => Ok(CrabFetchColor::Yellow),
            "blue" => Ok(CrabFetchColor::Blue),
            "magenta" => Ok(CrabFetchColor::Magenta),
            "cyan" => Ok(CrabFetchColor::Cyan),
            "white" => Ok(CrabFetchColor::White),
            "brightblack" => Ok(CrabFetchColor::BrightBlack),
            "brightred" => Ok(CrabFetchColor::BrightRed),
            "brightgreen" => Ok(CrabFetchColor::BrightGreen),
            "brightyellow" => Ok(CrabFetchColor::BrightYellow),
            "brightblue" => Ok(CrabFetchColor::BrightBlue),
            "brightmagenta" => Ok(CrabFetchColor::BrightMagenta),
            "brightcyan" => Ok(CrabFetchColor::BrightCyan),
            "brightwhite" => Ok(CrabFetchColor::BrightWhite),
            "clear" => Ok(CrabFetchColor::Clear),
            _ => Err(())
        }
    }
}
impl CrabFetchColor {
    pub fn color_string(&self, string: &str) -> ColoredString {
        match self {
            CrabFetchColor::Black => string.black(),
            CrabFetchColor::Red => string.red(),
            CrabFetchColor::Green => string.green(),
            CrabFetchColor::Yellow => string.yellow(),
            CrabFetchColor::Blue => string.blue(),
            CrabFetchColor::Magenta => string.magenta(),
            CrabFetchColor::Cyan => string.cyan(),
            CrabFetchColor::White => string.white(),
            CrabFetchColor::BrightBlack => string.bright_black(),
            CrabFetchColor::BrightRed => string.bright_red(),
            CrabFetchColor::BrightGreen => string.bright_green(),
            CrabFetchColor::BrightYellow => string.bright_yellow(),
            CrabFetchColor::BrightBlue => string.bright_blue(),
            CrabFetchColor::BrightMagenta => string.bright_magenta(),
            CrabFetchColor::BrightCyan => string.bright_cyan(),
            CrabFetchColor::BrightWhite => string.bright_white(),
            CrabFetchColor::Clear => string.clear(),
        }
    }
}

pub fn replace_color_placeholders(str: &String) -> String { // out of place here?
    let mut new_string = String::new();
    let split: Vec<&str> = str.split("{color-").collect();
    if split.len() <= 1 {
        return str.clone();
    }
    for s in split {
        let len: usize = match s.find("}") {
            Some(r) => r,
            None => {
                new_string.push_str(s);
                continue;
            },
        };
        let color_str: String = s[..len].to_string();
        let color: CrabFetchColor = match CrabFetchColor::from_str(&color_str) {
            Ok(r) => r,
            Err(_) => {
                continue;
            },
        };
        new_string.push_str(&color.color_string(&s[len + 1..]).to_string());
    }

    new_string
}

pub fn process_percentage_placeholder(text: &str, percentage: f32, config: &Configuration) -> String {
    let mut percent_str: String = percentage.to_string();
    percent_str.push_str("%");
    if config.percentage_color_thresholds.len() <= 0 {
        return text.replace("{percent}", &percent_str).to_string();
    }

    let mut cur_threshold: u8 = *config.percentage_color_thresholds.keys().min().unwrap();
    for x in &config.percentage_color_thresholds {
        let threshold: u8 = *x.0;
        if (percentage as u8) > threshold && threshold > cur_threshold {
            cur_threshold = threshold;
        }
    }

    let color = match CrabFetchColor::from_str(&config.percentage_color_thresholds[&cur_threshold]) {
        Ok(r) => r,
        Err(_) => CrabFetchColor::Clear,
    };

    percent_str = color.color_string(&percent_str.to_string()).to_string();
    text.replace("{percent}", &percent_str).to_string()
}

pub fn auto_format_bytes(kilobytes: u64, ibis: bool, dec_places: u32) -> String {
    let mut result: f64 = kilobytes as f64;
    let mut steps: u8 = 0; // 0 - Kilo, 1 - Mega, 2 - Giga, 3 - Tera 
    let divider = if ibis {1024} else {1000};
    if ibis {
        result = result / 1.024;
    }

    for _ in 0..3 {
        let cur_step: f64 = result as f64 / divider as f64;
        if cur_step <= 1.0 {
            break; // Use current 
        }

        result = cur_step;
        steps += 1;
    }
    result = round(result, dec_places);

    let dec_places: usize = dec_places as usize;
    let mut res: String = format!("{:.dec_places$}", result).to_string();
    res.push_str(match steps {
        0 => if ibis {" KiB"} else {" KB"},
        1 => if ibis {" MiB"} else {" MB"},
        2 => if ibis {" GiB"} else {" GB"},
        3 => if ibis {" TiB"} else {" TB"},
        _ => " ?"
    });
    return res;
}

// Rust is a great language, but when I need to start re-implementing the most basic of functions
// into your language, you know you've fucked up specing your language... badly.
pub fn round(number: f64, places: u32) -> f64 {
    let power: f64 = 10_u32.pow(places) as f64;
    (number * power).round() / power
}
