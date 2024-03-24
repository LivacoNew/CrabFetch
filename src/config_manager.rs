use std::{env, fs::{self, File}, io::{Read, Write}, path::Path};

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

    pub gpu_title: String,
    pub gpu_format: String,

    pub memory_title: String,
    pub memory_format: String,

    pub swap_title: String,
    pub swap_format: String,

    pub mount_title: String,
    pub mount_format: String,
    pub mount_ignored: Vec<String>,

    pub host_title: String,
    pub host_format: String,

    pub display_title: String,
    pub display_format: String,


    pub os_title: String,
    pub os_format: String,

    pub packages_title: String,
    pub packages_format: String,

    pub desktop_title: String,
    pub desktop_format: String,

    pub terminal_title: String,
    pub terminal_format: String,

    pub shell_title: String,
    pub shell_format: String,

    pub uptime_title: String,
    pub uptime_format: String,
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
        "underline".to_string(),

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

    // ASCII
    builder = builder.set_default("ascii_display", true).unwrap();
    builder = builder.set_default("ascii_colors", vec!["bright_magenta"]).unwrap();
    builder = builder.set_default("ascii_margin", 4).unwrap();

    // Hostname
    builder = builder.set_default("hostname_title", "").unwrap();
    builder = builder.set_default("hostname_format", "{username}@{hostname}").unwrap();
    builder = builder.set_default("hostname_color", true).unwrap();

    // Underline
    builder = builder.set_default("underline_length", 24).unwrap();
    builder = builder.set_default("underline_format", true).unwrap();

    // CPU
    builder = builder.set_default("cpu_title", "CPU").unwrap();
    builder = builder.set_default("cpu_format", "{name} ({core_count}c {thread_count}t) @ {max_clock_ghz} GHz").unwrap();

    // GPU
    builder = builder.set_default("gpu_title", "GPU").unwrap();
    builder = builder.set_default("gpu_format", "{vendor} {model} ({vram_gb} GB)").unwrap();

    // Memory
    builder = builder.set_default("memory_title", "Memory").unwrap();
    builder = builder.set_default("memory_format", "{phys_used_gib} GiB / {phys_max_gib} GiB ({percent}%)").unwrap();

    // Swap
    builder = builder.set_default("swap_title", "Swap").unwrap();
    builder = builder.set_default("swap_format", "{used_gib} GiB / {total_gib} GiB ({percent}%)").unwrap();

    // Mounts
    builder = builder.set_default("mount_title", "Disk {mount}").unwrap();
    builder = builder.set_default("mount_format", "{space_used_gb} GB used of {space_total_gb} GB @ ({percent}%)").unwrap();
    builder = builder.set_default("mount_ignored", vec!["/boot"]).unwrap();

    // Host
    builder = builder.set_default("host_title", "Host").unwrap();
    builder = builder.set_default("host_format", "{host}").unwrap();

    // Displays
    builder = builder.set_default("display_title", "Display {name}").unwrap();
    builder = builder.set_default("display_format", "{width}x{height} @ {refresh_rate}Hz").unwrap();


    // OS
    builder = builder.set_default("os_title", "Operating System").unwrap();
    builder = builder.set_default("os_format", "{distro} ({kernel})").unwrap();

    // Packages
    builder = builder.set_default("packages_title", "Packages").unwrap();
    builder = builder.set_default("packages_format", "{count} ({manager})").unwrap();

    // Desktop
    builder = builder.set_default("desktop_title", "Desktop").unwrap();
    builder = builder.set_default("desktop_format", "{desktop} ({display_type})").unwrap();

    // Terminal
    builder = builder.set_default("terminal_title", "Terminal").unwrap();
    builder = builder.set_default("terminal_format", "{terminal_name}").unwrap();

    // Shell
    builder = builder.set_default("shell_title", "Shell").unwrap();
    builder = builder.set_default("shell_format", "{shell}").unwrap();

    // And finally, uptime
    builder = builder.set_default("uptime_title", "Uptime").unwrap();
    builder = builder.set_default("uptime_format", "{hours}h {minutes}m {seconds}s").unwrap();


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
# vim:foldmethod=marker
# NOTE: This file is best used with Vim's folds.
# Apolgies if your using something else.

# General {{{
# The modules to display and in what order.
# All modules; hostname, underline, space, cpu, gpu, memory, swap, mounts, host, displays, os, packages, desktop, terminal, shell, uptime, colors, bright_colors
modules = [
    "hostname",
    "underline",

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


# The seperator between a modules title and it's value
seperator = " > "

# The color of a modules title
# Can be; black, red, green, yellow, blue, magenta, cyan, white
# All of these can be prefixed with "bright_" to be lighter versions, e.g bright_red
title_color = "bright_magenta"
# Whether to bold/italic the title
title_bold = true
title_italic = true

# How many decimal places to use on float values. This is kinda inconsistent right now.
decimal_places = 2

# }}}

# ASCII {{{

# If to display the ASCII distro art or not
ascii_display = true

# The colors to render the ASCII in
# This array can be as long as the actual ASCII. Each entry represents the color at a certain %
# E.g ["red", "green"] would render the top half as red and the bottom half as green.
# ["yellow", "blue", "magenta"] would render 33.33% as yellow, then blue, than magenta.
ascii_colors = ["bright_magenta", "bright_magenta", "bright_blue", "bright_magenta", "bright_magenta"]

# The amount of space to put between the ASCII and the info
ascii_margin = 4

# }}}

# Hostname {{{

# Title for the module
hostname_title = ""

# The format the hostname should be in. Placeholders;
# {hostname}            -> The hostname of the system.
# {username}            -> The username of the current user.
hostname_format = "{username}@{hostname}"

# Whether to color the hostname's placeholders the title color
# Mainly for if you remove the title and want to use it like a header, Neofetch style
hostname_color = true

# }}}
# Underline {{{

# Length of the underline
underline_length = 24

# Whether to format the underline to the title formatting
underline_format = true

# }}}
# CPU {{{

# Title for the module
cpu_title = "CPU"

# The format the module should be in. Placeholders;
# {name}                -> The name of the cpu.
# {core_count}          -> The number of cores.
# {thread_count}        -> The number of threads.
# {current_clock_mhz}   -> The current clock speed, in MHz.
# {current_clock_ghz}   -> The current clock speed, in GHz.
# {max_clock_mhz}       -> The maximum clock speed, in MHz.
# {max_clock_ghz}       -> The maximum clock speed, in GHz.
cpu_format = "{name} ({core_count}c {thread_count}t) @ {max_clock_ghz} GHz"

# }}}
# GPU {{{

# Title for the module
gpu_title = "GPU"

# The format the module should be in. Placeholders;
# {vendor}             -> The vendor of the GPU, e.g AMD
# {model}              -> The model of the GPU, e.g Radeon RX 7800XT
# {vram_mb}            -> The total memory of the GPU in mb
# {vram_gb}            -> The total memory of the GPU in gb
gpu_format = "{vendor} {model} ({vram_gb} GB)"

#}}}
# Memory {{{

# Title for the module
memory_title = "Memory"

# The format the memory should be in. Placeholders;
# {phys_used_kib}       -> The currently used memory in KiB.
# {phys_used_mib}       -> The currently used memory in MiB.
# {phys_used_gib}       -> The currently used memory in GiB.
# {phys_max_kib}        -> The maximum total memory in KiB.
# {phys_max_mib}        -> The maximum total memory in MiB.
# {phys_max_gib}        -> The maximum total memory in GiB.
# {percent}             -> Percentage of memory used
memory_format = "{phys_used_gib} GiB / {phys_max_gib} GiB ({percent}%)"

#}}}
# Swap {{{

# Title for the module
swap_title = "Swap"

# The format the module should be in. Placeholders;
# {used_kib}       -> The currently used swap in KiB.
# {used_mib}       -> The currently used swap in MiB.
# {used_gib}       -> The currently used swap in GiB.
# {max_kib}        -> The maximum total swap  in KiB.
# {max_mib}        -> The maximum total swap in MiB.
# {max_gib}        -> The maximum total swap in GiB.
# {percent}             -> Percentage of swap used
swap_format = "{used_gib} GiB / {total_gib} GiB ({percent}%)"

#}}}
# Mounts {{{

# Title for each mount. Placeholders;
# {device}              -> Device, e.g /dev/sda
# {mount}               -> The mount point, e.g /home
mount_title = "Disk {mount}"

# The format each mount should be in. Placeholders;
# {device}              -> Device, e.g /dev/sda
# {mount}               -> The mount point, e.g /home
# {space_used_mb}       -> The space used in megabytes.
# {space_avail_mb}      -> The space available in metabytes.
# {space_total_mb}      -> The total space in metabytes.
# {space_used_gb}       -> The space used in gigabytes.
# {space_avail_gb}      -> The space available in gigabytes.
# {space_total_gb}      -> The total space in gigabytes.
# {percent}             -> The percentage of the disk used.
mount_format = "{space_used_gb} GB used of {space_total_gb} GB @ ({percent}%)"

# Mounts that shouldn't be included
mount_ignored = ["/boot"]

# }}}
# Host {{{

# Title for the module
host_title = "Host"

# The format the Host should be in. Placeholders;
# {host}              -> The name of the host
host_format = "{host}"

# }}}
# Displays {{{

# Title for each display. Placeholders;
# {name}                -> The monitor name, e.g eDP-2
display_title = "Display {name}"

# The format each display should be in. Placeholders;
# {name}                -> The monitor name, e.g eDP-2
# {width}               -> The monitor's width
# {height}              -> The monitor's height
# {refresh_rate}        -> The monitor's refresh rate
display_format = "{width}x{height} @ {refresh_rate}Hz"

# }}}

# OS {{{

# Title for the module
os_title = "Operating System"

# The format the OS should be in. Placeholders;
# {distro}              -> The distro name
# {kernel}              -> The kernel version
os_format = "{distro} ({kernel})"

# }}}
# Packages {{{

# Title for the module
packages_title = "Packages"

# The format the Packages should be in. This format is for each entry, with all entries being combined into a single string seperated by a comma.
# Placeholders;
# {manager}              -> The name of the manager
# {count}                -> The amount of packages that manager reports
packages_format = "{count} ({manager})"

# }}}
# Desktop {{{

# Title for the module
desktop_title = "Desktop"

# The format the desktop should be in. Placeholders;
# {desktop}             -> The name of the desktop
# {display_type}        -> The type of display server, aka x11 or wayland.
desktop_format = "{desktop} ({display_type})"

# }}}
# Terminal {{{

# Title for the module
terminal_title = "Terminal"

# The format the Terminal should be in. Placeholders;
# {terminal_name}              -> The terminal name
terminal_format = "{terminal_name}"

# }}}
# Shell {{{

# Title for the module
shell_title = "Shell"

# The format the shell should be in. Placeholders;
# {shell}               -> The name of the shell, e.g zsh
# {path}                -> The path of the shell, e.g /bin/zsh
shell_format = "{shell}"

# }}}
# Uptime {{{

# Title for the module
uptime_title = "Uptime"

# The format the uptime should be in. Placeholders;
# {hours}               -> The hours
# {minutes}             -> The minutes
# {seconds}             -> The seconds
#
# NOTE: These are expected to be used in order. E.g Using only {seconds} will not give you the proper system uptime
uptime_format = "{hours}h {minutes}m {seconds}s"

# }}}
"#;
