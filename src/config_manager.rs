use std::{env, fmt::{Debug, Display}, fs::{self, File}, io::{BufRead, BufReader, Read, Write}, path::Path, str::FromStr, time::Instant};

use colored::{ColoredString, Colorize};
use serde::Deserialize;

use crate::{ascii::AsciiConfiguration, battery::BatteryConfiguration, cpu::CPUConfiguration, desktop::DesktopConfiguration, displays::DisplayConfiguration, editor::EditorConfiguration, gpu::GPUConfiguration, host::HostConfiguration, hostname::HostnameConfiguration, locale::LocaleConfiguration, memory::MemoryConfiguration, mounts::MountConfiguration, os::OSConfiguration, packages::PackagesConfiguration, shell::ShellConfiguration, swap::SwapConfiguration, terminal::TerminalConfiguration, uptime::UptimeConfiguration};

// This is a hack to get the color deserializaton working
// Essentially it uses my own enum, and to print it you need to call color_string
#[derive(Deserialize, Debug, Clone)]
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
            "bright_black" => Ok(CrabFetchColor::BrightBlack),
            "bright_red" => Ok(CrabFetchColor::BrightRed),
            "bright_green" => Ok(CrabFetchColor::BrightGreen),
            "bright_yellow" => Ok(CrabFetchColor::BrightYellow),
            "bright_blue" => Ok(CrabFetchColor::BrightBlue),
            "bright_magenta" => Ok(CrabFetchColor::BrightMagenta),
            "bright_cyan" => Ok(CrabFetchColor::BrightCyan),
            "bright_white" => Ok(CrabFetchColor::BrightWhite),
            _ => Err(())
        }
    }
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
pub fn replace_color_placeholders(str: &String) -> String { // out of place here?
    let mut new_string = String::new();
    let split: Vec<&str> = str.split("{color-").collect();
    if split.len() <= 1 {
        return str.clone();
    }
    for s in split {
        // println!("Parsing: {}", s);
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
                // log_erro("Color Placeholders", format!("Unable to parse color {}", color_str));
                new_string.push_str(&s[len + 1..].to_string());
                continue;
            },
        };
        new_string.push_str(&color_string(&s[len + 1..], &color).to_string());
    }

    new_string
}


pub trait ModuleConfiguration {
    fn apply_toml_line(&mut self, key: &str, value: &str) -> Result<(), TOMLParseError>;
}

#[derive(Deserialize)]
pub struct Configuration {
    pub modules: Vec<String>,
    pub seperator: String,
    pub title_color: CrabFetchColor,
    pub title_bold: bool,
    pub title_italic: bool,
    pub decimal_places: u32,
    pub inline_values: bool,
    pub underline_character: char,
    pub segment_top: String,
    pub suppress_errors: bool,

    pub ascii: AsciiConfiguration,

    pub hostname: HostnameConfiguration,
    pub cpu: CPUConfiguration,
    pub gpu: GPUConfiguration,
    pub memory: MemoryConfiguration,
    pub swap: SwapConfiguration,
    pub mounts: MountConfiguration,
    pub host: HostConfiguration,
    pub displays: DisplayConfiguration,
    pub os: OSConfiguration,
    pub packages: PackagesConfiguration,
    pub desktop: DesktopConfiguration,
    pub terminal: TerminalConfiguration,
    pub shell: ShellConfiguration,
    pub uptime: UptimeConfiguration,
    pub battery: BatteryConfiguration,
    pub locale: LocaleConfiguration,
    pub editor: EditorConfiguration
}
impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            modules: vec![
                "hostname".to_string(),
                "underline:16".to_string(),

                "cpu".to_string(),
                "gpu".to_string(),
                "memory".to_string(),
                "swap".to_string(),
                "mounts".to_string(),
                "host".to_string(),
                "displays".to_string(),

                "os".to_string(),
                "packages".to_string(),
                "desktop".to_string(),
                "terminal".to_string(),
                "shell".to_string(),
                "uptime".to_string(),

                "space".to_string(),
                "colors".to_string(),
                "bright_colors".to_string(),
            ],
            seperator: " > ".to_string(),
            title_color: CrabFetchColor::BrightMagenta,
            title_bold: true,
            title_italic: true,
            decimal_places: 2,
            inline_values: false,
            underline_character: '-',
            segment_top: "".to_string(),
            suppress_errors: false,
            ascii: AsciiConfiguration::default(),
            hostname: HostnameConfiguration::default(),
            cpu: CPUConfiguration::default(),
            gpu: GPUConfiguration::default(),
            memory: MemoryConfiguration::default(),
            swap: SwapConfiguration::default(),
            mounts: MountConfiguration::default(),
            host: HostConfiguration::default(),
            displays: DisplayConfiguration::default(),
            os: OSConfiguration::default(),
            packages: PackagesConfiguration::default(),
            desktop: DesktopConfiguration::default(),
            terminal: TerminalConfiguration::default(),
            shell: ShellConfiguration::default(),
            uptime: UptimeConfiguration::default(),
            battery: BatteryConfiguration::default()
        }
    }
}
impl Configuration {
    fn apply_toml_line(&mut self, table: &Option<String>, key: &str, value: &str) -> Result<(), TOMLParseError> {
        // this fuckin SUCKS
        // println!("Parsing: {:?}.{} -> {}", table, key, value);
        if table.is_some() {
            // TODO
            match table.as_ref().unwrap().as_str() {
                "ascii" => self.ascii.apply_toml_line(key, value)?,
                "hostname" => self.hostname.apply_toml_line(key, value)?,
                "cpu" => self.cpu.apply_toml_line(key, value)?,
                "gpu" => self.gpu.apply_toml_line(key, value)?,
                "memory" => self.memory.apply_toml_line(key, value)?,
                "swap" => self.swap.apply_toml_line(key, value)?,
                "mounts" => self.mounts.apply_toml_line(key, value)?,
                "host" => self.host.apply_toml_line(key, value)?,
                "displays" => self.displays.apply_toml_line(key, value)?,
                "os" => self.os.apply_toml_line(key, value)?,
                "packages" => self.packages.apply_toml_line(key, value)?,
                "desktop" => self.desktop.apply_toml_line(key, value)?,
                "terminal" => self.terminal.apply_toml_line(key, value)?,
                "shell" => self.shell.apply_toml_line(key, value)?,
                "uptime" => self.uptime.apply_toml_line(key, value)?,
                "battery" => self.battery.apply_toml_line(key, value)?,
                _ => return Err(TOMLParseError::new("Unknown table.".to_string(), table.clone(), Some(key.to_string()), value.to_string()))
            }
        } else {
            match key {
                "modules" => self.modules = toml_parse_str_array(value)?,
                "seperator" => self.seperator = toml_parse_string(value)?,
                "title_color" => self.title_color = toml_parse_string_to_color(value)?,
                "title_bold" => self.title_bold = toml_parse_bool(value)?,
                "title_italic" => self.title_italic = toml_parse_bool(value)?,
                "decimal_places" => self.decimal_places = toml_parse_u32(value)?,
                "inline_values" => self.inline_values = toml_parse_bool(value)?,
                "underline_character" => self.underline_character = toml_parse_char(value)?,
                "segment_top" => self.segment_top = toml_parse_string(value)?,
                "suppress_errors" => self.suppress_errors = toml_parse_bool(value)?,
                _ => return Err(TOMLParseError::new("Unknown key.".to_string(), table.clone(), Some(key.to_string()), value.to_string()))
            }
        }

        Ok(())
    }
}


// All the toml parsing functions
// Yell at me all you want, this is how I'm doing it.
pub fn toml_parse_str_array(value: &str) -> Result<Vec<String>, TOMLParseError> {
    if !value.starts_with("[") || !value.ends_with("]") {
        return Err(TOMLParseError::new("Invalid array; does not start/end with [...]".to_string(), None, None, value.to_string()))
    }
    let inner: String = value[1..value.len() - 1].to_string();
    let values: Vec<String> = inner.split(",")
        .map(|x| x.trim().trim_matches('"').to_string())
        .filter(|x| !x.is_empty())
        .collect();

    // println!("{:?}", values);
    Ok(values)
}
pub fn toml_parse_color_array(value: &str) -> Result<Vec<CrabFetchColor>, TOMLParseError> {
    let v: Vec<String> = toml_parse_str_array(value)?;
    Ok(v.iter()
        .filter_map(|x| match CrabFetchColor::from_str(x) {
            Ok(r) => Some(r),
            Err(_) => None
        })
        .collect())
}
pub fn toml_parse_string(value: &str) -> Result<String, TOMLParseError> {
    if (!value.starts_with('"') || !value.ends_with('"')) && (!value.starts_with("'") || !value.ends_with("'")) {
        return Err(TOMLParseError::new("Invalid String; does not start/end with single nor double quotes.".to_string(), None, None, value.to_string()))
    }

    let inner: String = value[1..value.len() - 1].to_string();
    Ok(inner)
}
pub fn toml_parse_string_to_color(value: &str) -> Result<CrabFetchColor, TOMLParseError> {
    let str: String = toml_parse_string(value)?;
    match CrabFetchColor::from_str(&str) {
        Ok(r) => return Ok(r),
        Err(_) => return Err(TOMLParseError::new(format!("Unknown color: {}", str), None, None, value.to_string())),
    };
}
pub fn toml_parse_bool(value: &str) -> Result<bool, TOMLParseError> {
    match value.to_lowercase().trim() {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(TOMLParseError::new("Invalid boolean: not true or false.".to_string(), None, None, value.to_string())),
    }
}
// TODO: Convert these two to generics
pub fn toml_parse_u16(value: &str) -> Result<u16, TOMLParseError> {
    match value.parse::<u16>() {
        Ok(r) => Ok(r),
        Err(e) => Err(TOMLParseError::new(format!("Invalid number: {}", e), None, None, value.to_string())),
    }
}
pub fn toml_parse_u32(value: &str) -> Result<u32, TOMLParseError> {
    match value.parse::<u32>() {
        Ok(r) => Ok(r),
        Err(e) => Err(TOMLParseError::new(format!("Invalid number: {}", e), None, None, value.to_string())),
    }
}
pub fn toml_parse_char(value: &str) -> Result<char, TOMLParseError> {
    let str: String = toml_parse_string(value)?;
    if str.len() > 1 {
        return Err(TOMLParseError::new("Invalid char: cannot be more than 1 character long.".to_string(), None, None, value.to_string()))
    }
    match str.parse::<char>() {
        Ok(r) => Ok(r),
        Err(e) => Err(TOMLParseError::new(format!("Invalid char: {}", e), None, None, value.to_string())),
    }
}



pub struct TOMLParseError {
    message: String,
    table: Option<String>,
    key: Option<String>,
    value: String
}
impl TOMLParseError {
    pub fn new(message: String, table: Option<String>, key: Option<String>, value: String) -> TOMLParseError {
        TOMLParseError {
            message,
            table,
            key,
            value
        }
    }
}
impl Display for TOMLParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.key.is_some() {
            let table: String = match &self.table {
                Some(r) => format!("{}.", r),
                None => "".to_string(),
            };
            write!(f, "Failed to parse key {}{}: {}", table, self.key.as_ref().unwrap(), self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}
impl Debug for TOMLParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.key.is_some() {
            let table: String = match &self.table {
                Some(r) => format!("{}.", r),
                None => "".to_string(),
            };
            write!(f, "Failed to parse TOML key {}{}: {} (attempted {})", table, self.key.as_ref().unwrap(), self.message, self.value)
        } else {
            write!(f, "{} (attempted {})", self.message, self.value)
        }
    }
}
pub fn parse(location_override: &Option<String>, module_override: Option<String>, ignore_file: &bool) -> Configuration {
    let t = Instant::now();
    let mut config: Configuration = Configuration::default();
    if *ignore_file {
        if module_override.is_some() {
            config.modules = module_override.unwrap()
                .split(",")
                .map(|x| x.to_string())
                .collect();
        }
        return config;
    }
    // Find the config path
    // Tries $XDG_CONFIG_HOME/CrabFetch before backing up to $HOME/.config/CrabFetch
    let config_path_str: String;
    if location_override.is_none() {
        config_path_str = match env::var("XDG_CONFIG_HOME") {
            Ok(mut r) => {
                r.push_str("/CrabFetch/config.toml");
                r
            }
            Err(_) => {
                // Let's try the home directory
                let mut home_dir: String = match env::var("HOME") {
                    Ok(r) => r,
                    Err(e) => panic!("Unable to find config folder; {}", e) // WHYYYY???
                };
                home_dir.push_str("/.config/CrabFetch/config.toml");
                home_dir
            }
        };
    } else {
        config_path_str = location_override.clone().unwrap();
    }

    // An attempt at my own TOML parser
    // This tries to follow as close to https://toml.io/en/v1.0.0 as possible
    // Done because all the rust TOML parsers are too slow lol
    // Note this is extremely hacky - I have zero confidence in this being quite right yet
    // Some stuff that isn't quite right yet;
    //  - Quoted/Dotted keys are not supported, and will break things
    //  - Duplicate keys are not checked - Whichever one comes last overrides
    //  - Literal strings aren't really handled right
    //  - Tons of data types aren't done, because CrabFetch doesn't use them.
    // Time to beat from toml crate: 120-130us
    let file: File = match File::open(config_path_str) {
        Ok(r) => r,
        Err(_) => {
            if module_override.is_some() {
                config.modules = module_override.unwrap()
                    .split(",")
                    .map(|x| x.to_string())
                    .collect();
            }
            return config
        },
    };
    let buffer: BufReader<File> = BufReader::new(file);
    let mut current_table: Option<String> = None;
    let mut current_array_str: Option<String> = None;
    for line in buffer.lines() {
        if line.is_err() {
            continue
        }
        let line: String = line.unwrap();
        let line = line.trim();
        if line.starts_with("#") || line.is_empty() {
            continue
        }

        if line.starts_with("[") && line.ends_with("]") {
            // Table
            current_table = Some(line[1..line.len() - 1].to_string());
            continue
        }
        if current_array_str.is_some() {
            current_array_str.as_mut().unwrap().push_str(line);
            if line.contains("]") {
                let array_str = current_array_str.unwrap();
                let split: Vec<&str> = array_str.splitn(2, "=")
                    .map(|x| x.trim())
                    .collect();
                let (left, right) = (split[0], split[1]);
                match config.apply_toml_line(&current_table, left, right) {
                    Ok(_) => {},
                    Err(e) => {
                        println!("Config error: {}", e);
                    },
                };
                current_array_str = None;
            }
            continue
        }
        if line.contains("=") {
            let split: Vec<&str> = line.splitn(2, "=")
                .map(|x| x.trim())
                .collect();

            let (left, right) = (split[0], split[1]);
            if right.starts_with("[") {
                if !right.contains("]") {
                    current_array_str = Some(line.to_string());
                    continue
                }
            }

            match config.apply_toml_line(&current_table, left, right) {
                Ok(_) => {},
                Err(e) => {
                    println!("Config error: {}", e);
                },
            };
        }
    }
    if module_override.is_some() {
        config.modules = module_override.unwrap()
            .split(",")
            .map(|x| x.to_string())
            .collect();
    }

    println!("Total time: {:2?}", t.elapsed());
    config
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
                Ok(r) => r,
                Err(e) => panic!("Unable to find config folder; {}", e) // bruh
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
        Err(e) => panic!("Can't read from ASCII override - {}", e),
    };
    let mut contents: String = String::new();
    match file.read_to_string(&mut contents) {
        Ok(_) => {},
        Err(e) => panic!("Can't read from ASCII override - {}", e),
    }

    Some(contents)
}

pub fn generate_config_file(location_override: Option<String>) {
    let path: String;
    if location_override.is_some() {
        path = shellexpand::tilde(&location_override.unwrap()).to_string();
        // Config won't be happy unless it ends with .toml
        if !path.ends_with(".toml") {
            // Simply crash, to avoid confusing the user as to why the default config is being used
            // instead of their custom one.
            panic!("Config path must end with '.toml'");
        }
    } else {
        // Find the config path
        // Tries $XDG_CONFIG_HOME/CrabFetch before backing up to $HOME/.config/CrabFetch
        path = match env::var("XDG_CONFIG_HOME") {
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
                        panic!("Unable to find suitable config folder; {}", e);
                    }
                };
                home_dir.push_str("/.config/CrabFetch/config.toml");
                home_dir
            }
        };
    }
    let config_path: &Path = Path::new(&path);

    if config_path.exists() {
        panic!("Path already exists: {}", config_path.display());
    }
    match fs::create_dir_all(config_path.parent().unwrap()) {
        Ok(_) => {},
        Err(e) => panic!("Unable to create directory: {}", e),
    };

    let mut file: File = match File::create(config_path) {
        Ok(r) => r,
        Err(e) => panic!("Unable to create file; {}", e),
    };
    match file.write_all(DEFAULT_CONFIG_CONTENTS.as_bytes()) {
        Ok(_) => {},
        Err(e) => panic!("Unable to write to file; {}", e),
    };
    println!("Created default config file at {}", path);
}



// The default config, stored so that it can be written
const DEFAULT_CONFIG_CONTENTS: &str = r#"
# The modules to display and in what order.
# All modules; hostname, space, cpu, gpu, memory, swap, mounts, host, displays, os, packages, desktop, terminal, shell, uptime, colors, bright_colors
modules = [
    "hostname",
    "underline:16",

    "cpu",
    "gpu",
    "memory",
    "swap",
    "mounts",
    "host",
    "displays",

    "os",
    "packages",
    "desktop",
    "terminal",
    "shell",
    "editor",
    "uptime",

    "space",
    "colors",
    "bright_colors"
]

# The default seperator between a modules title and it's value
seperator = " > "
# The default color of a modules title
# Can be; black, red, green, yellow, blue, magenta, cyan, white
# All of these can be prefixed with "bright_" to be lighter versions, e.g bright_red
title_color = "bright_magenta"
# Whether to bold/italic the title by default too
title_bold = false
title_italic = false

# The default decimal places to provide in a module
decimal_places = 2

# Whether to have all module values as inline, e.g; https://i.imgur.com/UNyq2zj.png
# To add padding use the "seperator" and add some spaces
inline_values = false

# The character to use in the underline module
underline_character = '-'

# Format of segments
# Segments can be defined in the modules array
segment_top = "{color-white}[======------{color-brightmagenta} {name} {color-white}------======]"

# Whether to supress any errors that come or not
suppress_errors = false


[ascii]
# If to display the ASCII distro art or not
display = true

# The colors to render the ASCII in
# This array can be as long as the actual ASCII. Each entry represents the color at a certain %
# E.g ["red", "green"] would render the top half as red and the bottom half as green.
# ["yellow", "blue", "magenta"] would render 33.33% as yellow, then blue, than magenta.
colors = ["bright_magenta"]

# The amount of space to put between the ASCII and the info
margin = 4




# Below here is the actual modules
# Refer to the wiki for any module-specific parameters or hidden parameters
# Also remember that you can override some stuff on these, e.g the title formatting.

[hostname]
title = ""
# Placeholders;
# {hostname} -> The hostname
# {username} -> The username of the current user
format = "{color-bright_magenta}{username}{color-white}@{color-bright_magenta}{hostname}"


[cpu]
title = "CPU"
# Placeholders;
# {name} -> The name of the cpu.
# {core_count} -> The number of cores.
# {thread_count} -> The number of threads.
# {current_clock_mhz} -> The current clock speed, in MHz.
# {current_clock_ghz} -> The current clock speed, in GHz.
# {max_clock_mhz} -> The maximum clock speed, in MHz.
# {max_clock_ghz} -> The maximum clock speed, in GHz.
format = "{name} ({core_count}c {thread_count}t) @ {max_clock_ghz} GHz"


[gpu]
# The method for getting GPU info
# Getting accurate GPU info can be really slow. Because of this CrabFetch gives you two options
# - "pcisysfile" which searches the /sys/bus/pci/devices directory to find your GPU. This is fast but can be inaccurate due to decoding the vendor/product IDs
# - "glxinfo" which uses the glxinfo command to get the primary GPU. This is more accurate but REALLY slow!
# These methods may give be the exact same info, but if not you can swap to one or the other.
method = "pcisysfile"

# On top of the above, this allows you to choose to cache the GPU info.
# It's reccomended to use this with "glxinfo" to give you full speed while retaining accurate GPU info.
cache = false

title = "GPU"
# Placeholders;
# {vendor} -> The vendor of the GPU, e.g AMD
# {model} -> The model of the GPU, e.g Radeon RX 7800XT
# {vram_mb} -> The total memory of the GPU in mb
# {vram_gb} -> The total memory of the GPU in gb
format = "{vendor} {model} ({vram_gb} GB)"


[memory]
title = "Memory"
# Placeholders;
# {phys_used_kib} -> The currently used memory in KiB.
# {phys_used_mib} -> The currently used memory in MiB.
# {phys_used_gib} -> The currently used memory in GiB.
# {phys_max_kib} -> The maximum total memory in KiB.
# {phys_max_mib} -> The maximum total memory in MiB.
# {phys_max_gib} -> The maximum total memory in GiB.
# {percent} -> Percentage of memory used
format = "{phys_used_gib} GiB / {phys_max_gib} GiB ({percent}%)"


[swap]
title = "Swap"
# Placeholders;
# {used_kib} -> The currently used swap in KiB.
# {used_mib} -> The currently used swap in MiB.
# {used_gib} -> The currently used swap in GiB.
# {max_kib} -> The maximum total swap  in KiB.
# {max_mib} -> The maximum total swap in MiB.
# {max_gib} -> The maximum total swap in GiB.
# {percent} -> Percentage of swap used
format = "{used_gib} GiB / {total_gib} GiB ({percent}%)"


[mounts]
# Each mount has it's own entry. Title Placeholders;
# {device}              -> Device, e.g /dev/sda
# {mount}               -> The mount point, e.g /home
title = "Disk {mount}"

# Placeholders;
# {device} -> Device, e.g /dev/sda
# {mount} -> The mount point, e.g /home
# {space_used_mb} -> The space used in megabytes.
# {space_avail_mb} -> The space available in metabytes.
# {space_total_mb} -> The total space in metabytes.
# {space_used_gb} -> The space used in gigabytes.
# {space_avail_gb} -> The space available in gigabytes.
# {space_total_gb} -> The total space in gigabytes.
# {percent} -> The percentage of the disk used.
format = "{space_used_gb} GB used of {space_total_gb} GB ({percent}%)"

# Mounts that shouldn't be included
# The mounts only need to start with these
ignore = ["/boot", "/snap"]


[host]
title = "Host"


[displays]
# Same as mounts. Placeholders;
# {name} -> The monitor name, e.g eDP-2
title = "Display {name}"

# The format each display should be in. Placeholders;
# {name} -> The monitor name, e.g eDP-2
# {width} -> The monitor's width
# {height} -> The monitor's height
format = "{width}x{height}"


[os]
title = "Operating System"
# Placeholders;
# {distro} -> The distro name
# {kernel} -> The kernel version
format = "{distro} ({kernel})"


[packages]
title = "Packages"
# This format is for each entry, with all entries being combined into a single string seperated by a comma. Placeholders;
# {manager} -> The name of the manager
# {count} -> The amount of packages that manager reports
format = "{count} ({manager})"


[desktop]
title = "Desktop"
# Placeholders;
# {desktop} -> The name of the desktop
# {display_type} -> The type of display server, aka x11 or wayland.
format = "{desktop} ({display_type})"


[terminal]
title = "Terminal"
# Whether to find the name of the current PTS if SSH is being used. This is a togglable option as most people probably won't care to go hunting for it.
chase_ssh_pts = false


[shell]
title = "Shell"
# Placeholders;
# {shell} -> The name of the shell, e.g zsh
# {path} -> The path of the shell, e.g /bin/zsh
format = "{shell} ({path})"

# Whether to show your default shell, instead of your current shell.
show_default_shell = false


[uptime]
title = "Uptime"


[editor]
title = "Editor"
# Placeholders;
# {name} - The name of the editor
# {path} - The path the editor is at
format = "{name}"

# Whether to turn the name into a "fancy" variant. E.g "nvim" gets turned into "NeoVim"
fancy = true


[locale]
title = "Locale"


[battery]
title = "Battery"
# Placeholders;
# {percentage} -> The battery percentage
format = "{percentage}%"



# You've reached the end! Congrats, have a muffin :)"#;
