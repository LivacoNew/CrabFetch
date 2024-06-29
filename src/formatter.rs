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

// This is placed in color as it's likely going to coloring the percentages
pub fn process_percentage_placeholder(text: &str, percentage: f32, config: &Configuration) -> String {
    if config.percentage_color_thresholds.len() <= 0 {
        return text.replace("{percent}", &percentage.to_string()).to_string();
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

    let mut percent_str: String = percentage.to_string();
    percent_str.push_str("%");
    percent_str = color.color_string(&percent_str.to_string()).to_string();
    text.replace("{percent}", &percent_str).to_string()
}
