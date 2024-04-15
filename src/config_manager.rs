use std::{env, fs::{self, File}, io::{Read, Write}, path::Path, str::FromStr};

use colored::{ColoredString, Colorize};
use config::{builder::DefaultState, Config, ConfigBuilder};
use serde::Deserialize;

use crate::{ascii::AsciiConfiguration, battery::BatteryConfiguration, cpu::CPUConfiguration, desktop::DesktopConfiguration, displays::DisplayConfiguration, gpu::GPUConfiguration, host::HostConfiguration, hostname::HostnameConfiguration, log_error, memory::MemoryConfiguration, mounts::MountConfiguration, os::OSConfiguration, packages::PackagesConfiguration, shell::ShellConfiguration, swap::SwapConfiguration, terminal::TerminalConfiguration, uptime::UptimeConfiguration};

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
            "brightblack" => Ok(CrabFetchColor::BrightBlack),
            "brightred" => Ok(CrabFetchColor::BrightRed),
            "brightgreen" => Ok(CrabFetchColor::BrightGreen),
            "brightyellow" => Ok(CrabFetchColor::BrightYellow),
            "brightblue" => Ok(CrabFetchColor::BrightBlue),
            "brightmagenta" => Ok(CrabFetchColor::BrightMagenta),
            "brightcyan" => Ok(CrabFetchColor::BrightCyan),
            "brightwhite" => Ok(CrabFetchColor::BrightWhite),
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
                log_error("Color Placeholders", format!("Unable to parse color {}", color_str));
                continue;
            },
        };
        new_string.push_str(&color_string(&s[len + 1..], &color).to_string());
    }

    new_string
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
    pub battery: BatteryConfiguration
}

pub fn parse(location_override: &Option<String>, ignore_file: &bool) -> Configuration {
    let config_path_str: String;
    if location_override.is_some() {
        config_path_str = shellexpand::tilde(&location_override.clone().unwrap()).to_string();
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
    if !ignore_file {
        builder = builder.add_source(config::File::with_name(&config_path_str).required(false));
    }
    // Set the defaults here
    // General
    builder = builder.set_default("modules", vec![
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
    ]).unwrap();
    builder = builder.set_default("seperator", " > ").unwrap();
    builder = builder.set_default("title_color", "bright_magenta").unwrap();
    builder = builder.set_default("title_bold", true).unwrap();
    builder = builder.set_default("title_italic", true).unwrap();
    builder = builder.set_default("decimal_places", 2).unwrap();
    builder = builder.set_default("inline_values", false).unwrap();
    builder = builder.set_default("underline_character", "―").unwrap();
    builder = builder.set_default("segment_top", "{color-white}[======------{color-brightmagenta} {name} {color-white}------======]").unwrap();
    builder = builder.set_default("suppress_errors", true).unwrap();

    // ASCII
    builder = builder.set_default("ascii.display", true).unwrap();
    builder = builder.set_default("ascii.colors", vec!["bright_magenta"]).unwrap();
    builder = builder.set_default("ascii.margin", 4).unwrap();

    // Modules
    builder = builder.set_default("hostname.title", "").unwrap();
    builder = builder.set_default("hostname.format", "{color-brightmagenta}{username}{color-white}@{color-brightmagenta}{hostname}").unwrap();

    builder = builder.set_default("cpu.title", "CPU").unwrap();
    builder = builder.set_default("cpu.format", "{name} ({core_count}c {thread_count}t) @ {max_clock_ghz} GHz").unwrap();

    builder = builder.set_default("gpu.method", "pcisysfile").unwrap();
    builder = builder.set_default("gpu.cache", false).unwrap();
    builder = builder.set_default("gpu.title", "GPU").unwrap();
    builder = builder.set_default("gpu.format", "{vendor} {model} ({vram_gb} GB)").unwrap();

    builder = builder.set_default("memory.title", "Memory").unwrap();
    builder = builder.set_default("memory.format", "{phys_used_gib} GiB / {phys_max_gib} GiB ({percent}%)").unwrap();

    builder = builder.set_default("swap.title", "Swap").unwrap();
    builder = builder.set_default("swap.format", "{used_gib} GiB / {total_gib} GiB ({percent}%)").unwrap();

    builder = builder.set_default("mounts.title", "Disk {mount}").unwrap();
    builder = builder.set_default("mounts.format", "{space_used_gb} GB used of {space_total_gb} GB ({percent}%)").unwrap();
    builder = builder.set_default("mounts.ignore", vec!["/boot", "/snap"]).unwrap();

    builder = builder.set_default("host.title", "host").unwrap();

    builder = builder.set_default("displays.title", "Display {name}").unwrap();
    builder = builder.set_default("displays.format", "{width}x{height} @ {refresh_rate}Hz").unwrap();

    builder = builder.set_default("os.title", "Operating System").unwrap();
    builder = builder.set_default("os.format", "{distro} ({kernel})").unwrap();

    builder = builder.set_default("packages.title", "Packages").unwrap();
    builder = builder.set_default("packages.format", "{count} ({manager})").unwrap();

    builder = builder.set_default("desktop.title", "Desktop").unwrap();
    builder = builder.set_default("desktop.format", "{desktop} ({display_type})").unwrap();

    builder = builder.set_default("terminal.title", "Terminal").unwrap();

    builder = builder.set_default("shell.title", "Shell").unwrap();
    builder = builder.set_default("shell.format", "{shell}").unwrap();
    builder = builder.set_default("shell.show_default_shell", "false").unwrap();

    builder = builder.set_default("uptime.title", "Uptime").unwrap();
    builder = builder.set_default("uptime.format", "{hours}h {minutes}m {seconds}s").unwrap();

    builder = builder.set_default("battery.title", "Battery").unwrap();
    builder = builder.set_default("battery.format", "{percentage}%").unwrap();
    builder = builder.set_default("battery.path", "BAT0").unwrap();



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
suppress_errors = true


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
format = "{color-brightmagenta}{username}{color-white}@{color-brightmagenta}{hostname}"


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
# {refresh_rate} -> The monitor's refresh rate
format = "{width}x{height} @ {refresh_rate}Hz"


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
# Placeholders;
# {hours} -> The hours
# {minutes} -> The minutes
# {seconds} -> The seconds
# NOTE: These are expected to be used in order. E.g Using only {seconds} will not give you the proper system uptime
format = "{hours}h {minutes}m {seconds}s"


[battery]
title = "Battery"
# Placeholders;
# {percentage} -> The battery percentage
format = "{percentage}%"



# You've reached the end! Congrats, have a muffin :)"#;
