use std::env;

use colored::{ColoredString, Colorize};
use config::Config;
use serde::Deserialize;

// This is a hack to get the color deserializaton working
// Essentially it uses my own enum, and to print it you need to call cfcolor_to_terminal_color
#[derive(Deserialize, Debug)]
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
    BrightWhite
}
pub fn color_string(string: &str, color: &CrabFetchColor) -> ColoredString {
    match color {
        CrabFetchColor::Black => string.black(),
        CrabFetchColor::Red => string.red(),
        CrabFetchColor::Green => string.green(),
        CrabFetchColor::Yellow => string.yellow(),
        CrabFetchColor::Blue => todo!(),
        CrabFetchColor::Magenta => todo!(),
        CrabFetchColor::Cyan => todo!(),
        CrabFetchColor::White => todo!(),
        CrabFetchColor::BrightBlack => todo!(),
        CrabFetchColor::BrightRed => todo!(),
        CrabFetchColor::BrightGreen => todo!(),
        CrabFetchColor::BrightYellow => todo!(),
        CrabFetchColor::BrightBlue => todo!(),
        CrabFetchColor::BrightMagenta => todo!(),
        CrabFetchColor::BrightCyan => todo!(),
        CrabFetchColor::BrightWhite => todo!(),
    }
}


#[derive(Deserialize)]
pub struct Configuration {
    pub seperator: String,
    pub title_color: CrabFetchColor,

    pub enable_cpu: bool,
    pub cpu_title: String,
    pub cpu_format: String,

    pub enable_memory: bool,
    pub memory_title: String,
    pub memory_format: String
}

pub fn parse() -> Configuration {
    // Find the config path
    // Tries $XDG_CONFIG_HOME/CrabFetch before backing up to $HOME/.config/CrabFetch
    let config_path_str: String = match env::var("XDG_CONFIG_HOME") {
        Ok(mut r) => {
            r.push_str("/CrabFetch/config.toml");
            r
        }
        Err(_) => {
            // Let's try the home directory
            let mut home_dir: String = match env::var("HOME") {
                Ok(r) => {
                    r
                },
                Err(e) => {
                    // why tf would you unset home lmao
                    panic!("Unable to find config folder; {}", e);
                }
            };
            home_dir.push_str("/.config/CrabFetch/config.toml");
            home_dir
        }
    };

    let mut builder = Config::builder();
    builder = builder.add_source(config::File::with_name(&config_path_str).required(false));
    // Set the defaults here
    builder = builder.set_default("seperator", " > ").unwrap();

    builder = builder.set_default("enable_cpu", true).unwrap();
    builder = builder.set_default("cpu_title", "Processor").unwrap();
    builder = builder.set_default("cpu_format", "Processor > {name} @ {max_clock_ghz} GHz (currently {current_clock_ghz} GHz)").unwrap();

    builder = builder.set_default("enable_memory", true).unwrap();
    builder = builder.set_default("memory_title", "Memory").unwrap();
    builder = builder.set_default("memory_format", "Memory -> {phys_used_gib}GiB / {phys_max_gib}GiB").unwrap();
    // Now stop.
    let config = match builder.build() {
        Ok(r) => r,
        Err(e) => panic!("Unable to parse config.toml: {}", e),
    };

    let deserialized = match config.try_deserialize::<Configuration>() {
        Ok(r) => r,
        Err(e) => panic!("Unable to parse config.toml: {}", e),
    };
    deserialized
}
