use std::{env, path::Path, fs::File, io::Read};

use colored::{ColoredString, Colorize};
use config::{builder::DefaultState, Config, ConfigBuilder};
use serde::Deserialize;

// This is a hack to get the color deserializaton working
// Essentially it uses my own enum, and to print it you need to call color_string
#[derive(Deserialize, Debug)]
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
    BrightWhite
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
        CrabFetchColor::BrightBlack => string.bright_black(),
        CrabFetchColor::BrightRed => string.bright_red(),
        CrabFetchColor::BrightGreen => string.bright_green(),
        CrabFetchColor::BrightYellow => string.bright_yellow(),
        CrabFetchColor::BrightBlue => string.bright_blue(),
        CrabFetchColor::BrightMagenta => string.bright_magenta(),
        CrabFetchColor::BrightCyan => string.bright_cyan(),
        CrabFetchColor::BrightWhite => string.bright_white(),
    }
}


#[derive(Deserialize)]
pub struct Configuration {
    pub modules: Vec<String>,
    pub seperator: String,
    pub title_color: CrabFetchColor,
    pub title_bold: bool,
    pub title_italic: bool,
    pub decimal_places: u32,

    pub ascii_display: bool,
    pub ascii_colors: Vec<CrabFetchColor>,
    pub ascii_margin: u16,

    pub hostname_title: String,
    pub hostname_format: String,
    pub hostname_color: bool,

    pub underline_length: u16,
    pub underline_format: bool,

    pub cpu_title: String,
    pub cpu_format: String,

    pub memory_title: String,
    pub memory_format: String,

    pub swap_title: String,
    pub swap_format: String,

    pub gpu_title: String,
    pub gpu_format: String,

    pub os_title: String,
    pub os_format: String,

    pub terminal_title: String,
    pub terminal_format: String,

    pub uptime_title: String,
    pub uptime_format: String,

    pub desktop_title: String,
    pub desktop_format: String,

    pub shell_title: String,
    pub shell_format: String,

    pub mount_title: String,
    pub mount_format: String,
    pub mount_ignored: Vec<String>
}

pub fn parse(location_override: Option<String>) -> Configuration {
    let config_path_str: String;
    if location_override.is_some() {
        config_path_str = shellexpand::tilde(&location_override.unwrap()).to_string();
        // Config won't be happy unless it ends with .toml
        if !config_path_str.ends_with(".toml") {
            // Simply crash, to avoid confusing the user as to why the default config is being used
            // instead of their custom one.
            panic!("Config path must end with '.toml'");
        }

        // Verify it exists
        let path: &Path = Path::new(&config_path_str);
        if !path.exists() {
            // Simply crash, to avoid confusing the user as to why the default config is being used
            // instead of their custom one.
            panic!("Unable to find config: {}", config_path_str);
        }
    } else {
        // Find the config path
        // Tries $XDG_CONFIG_HOME/CrabFetch before backing up to $HOME/.config/CrabFetch
        config_path_str = match env::var("XDG_CONFIG_HOME") {
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
    }

    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    builder = builder.add_source(config::File::with_name(&config_path_str).required(false));
    // Set the defaults here
    builder = builder.set_default("modules", vec!["cpu".to_string(), "memory".to_string()]).unwrap();
    builder = builder.set_default("seperator", " > ").unwrap();
    builder = builder.set_default("title_color", "bright_magenta").unwrap();
    builder = builder.set_default("title_bold", true).unwrap();
    builder = builder.set_default("title_italic", true).unwrap();

    builder = builder.set_default("ascii_display", true).unwrap();
    builder = builder.set_default("ascii_colors", vec!["red"]).unwrap();

    builder = builder.set_default("cpu_title", "Processor").unwrap();
    builder = builder.set_default("cpu_format", "Processor > {name} @ {max_clock_ghz} GHz (currently {current_clock_ghz} GHz)").unwrap();

    builder = builder.set_default("memory_title", "Memory").unwrap();
    builder = builder.set_default("memory_format", "Memory > {phys_used_gib}GiB / {phys_max_gib}GiB").unwrap();

    builder = builder.set_default("os_title", "Operating System").unwrap();
    builder = builder.set_default("os_format", "{distro} ({kernel})").unwrap();

    builder = builder.set_default("uptime_title", "System Uptime").unwrap();

    builder = builder.set_default("desktop_title", "Desktop").unwrap();
    builder = builder.set_default("desktop_format", "{desktop}").unwrap();
    // Now stop.
    let config: Config = match builder.build() {
        Ok(r) => r,
        Err(e) => panic!("Unable to parse config.toml: {}", e),
    };

    let deserialized: Configuration = match config.try_deserialize::<Configuration>() {
        Ok(r) => r,
        Err(e) => panic!("Unable to parse config.toml: {}", e),
    };
    deserialized
}

pub fn check_for_ascii_override() -> Option<String> {
    let ascii_path_str: String = match env::var("XDG_CONFIG_HOME") {
        Ok(mut r) => {
            r.push_str("/CrabFetch/ascii");
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
            home_dir.push_str("/.config/CrabFetch/ascii");
            home_dir
        }
    };

    let path: &Path = Path::new(&ascii_path_str);
    if !path.exists() {
        return None;
    }

    let mut file: File = match File::open(path) {
        Ok(r) => r,
        Err(e) => {
            panic!("Can't read from ASCII override - {}", e);
        },
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => {
            panic!("Can't read from ASCII override - {}", e);
        },
    }

    Some(contents)
}
