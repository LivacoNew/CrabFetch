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
    // Ignore the LSP warnings here - needs to be _ so that serde can serialize it
    Bright_Black,
    Bright_Red,
    Bright_Green,
    Bright_Yellow,
    Bright_Blue,
    Bright_Magenta,
    Bright_Cyan,
    Bright_White
}
pub fn color_string(string: &str, color: &CrabFetchColor) -> ColoredString {
    match color {
        CrabFetchColor::Black => string.black(),
        CrabFetchColor::Red => string.red(),
        CrabFetchColor::Green => string.green(),
        CrabFetchColor::Yellow => string.yellow(),
        CrabFetchColor::Blue => string.blue(),
        CrabFetchColor::Magenta => string.magenta(),
        CrabFetchColor::Cyan => string.cyan(),
        CrabFetchColor::White => string.white(),
        CrabFetchColor::Bright_Black => string.bright_black(),
        CrabFetchColor::Bright_Red => string.bright_red(),
        CrabFetchColor::Bright_Green => string.bright_green(),
        CrabFetchColor::Bright_Yellow => string.bright_yellow(),
        CrabFetchColor::Bright_Blue => string.bright_blue(),
        CrabFetchColor::Bright_Magenta => string.bright_magenta(),
        CrabFetchColor::Bright_Cyan => string.bright_cyan(),
        CrabFetchColor::Bright_White => string.bright_white(),
    }
}


#[derive(Deserialize)]
pub struct Configuration {
    pub modules: Vec<String>,
    pub seperator: String,
    pub title_color: CrabFetchColor,
    pub title_bold: bool,
    pub title_italic: bool,

    pub cpu_title: String,
    pub cpu_format: String,

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
    println!("{}", config_path_str);

    let mut builder = Config::builder();
    builder = builder.add_source(config::File::with_name(&config_path_str).required(false));
    // Set the defaults here
    builder = builder.set_default("modules", vec!["cpu".to_string(), "memory".to_string()]).unwrap();
    builder = builder.set_default("seperator", " > ").unwrap();
    builder = builder.set_default("title_color", "bright_magenta").unwrap();
    builder = builder.set_default("title_bold", true).unwrap();
    builder = builder.set_default("title_italic", true).unwrap();

    builder = builder.set_default("cpu_title", "Processor").unwrap();
    builder = builder.set_default("cpu_format", "Processor > {name} @ {max_clock_ghz} GHz (currently {current_clock_ghz} GHz)").unwrap();

    builder = builder.set_default("memory_title", "Memory").unwrap();
    builder = builder.set_default("memory_format", "Memory > {phys_used_gib}GiB / {phys_max_gib}GiB").unwrap();
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
