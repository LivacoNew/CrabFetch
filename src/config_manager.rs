use std::{collections::HashMap, env, fs::{self, File}, io::{Read, Write}, path::Path};

use config::{builder::DefaultState, Config, ConfigBuilder};
use serde::Deserialize;

use crate::{ascii::AsciiConfiguration, battery::BatteryConfiguration, cpu::CPUConfiguration, desktop::DesktopConfiguration, displays::DisplayConfiguration, editor::EditorConfiguration, formatter::CrabFetchColor, gpu::GPUConfiguration, host::HostConfiguration, hostname::HostnameConfiguration, initsys::InitSystemConfiguration, locale::LocaleConfiguration, memory::MemoryConfiguration, mounts::MountConfiguration, os::OSConfiguration, packages::PackagesConfiguration, processes::ProcessesConfiguration, shell::ShellConfiguration, swap::SwapConfiguration, terminal::TerminalConfiguration, uptime::UptimeConfiguration};
#[cfg(feature = "music")]
use crate::music::MusicConfiguration;


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
    pub segment_bottom: String,
    pub progress_left_border: String,
    pub progress_right_border: String,
    pub progress_progress: String,
    pub progress_empty: String,
    pub progress_target_length: u8,
    pub percentage_color_thresholds: HashMap<u8, String>,
    pub use_ibis: bool,
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
    #[cfg(feature = "music")]
    pub music: MusicConfiguration,
    pub editor: EditorConfiguration,
    pub initsys: InitSystemConfiguration,
    pub processes: ProcessesConfiguration
}

pub fn parse(location_override: &Option<String>, module_override: &Option<String>, ignore_file: &bool) -> Configuration {
    let mut builder: ConfigBuilder<DefaultState> = Config::builder();
    if !ignore_file {
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
                        Ok(r) => r,
                        Err(e) => panic!("Unable to find config folder; {}", e) // WHYYYY???
                    };
                    home_dir.push_str("/.config/CrabFetch/config.toml");
                    home_dir
                }
            };
        }

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
                                  "editor".to_string(),
                                  "uptime".to_string(),
                                  "locale".to_string(),
                                  "music".to_string(),
                                  "initsys".to_string(),
                                  "processes".to_string(),

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
    builder = builder.set_default("segment_bottom", "{color-white}[======------{color-brightmagenta} {name_sized_gap} {color-white}------======]").unwrap();
    builder = builder.set_default("progress_left_border", "[").unwrap();
    builder = builder.set_default("progress_right_border", "]").unwrap();
    builder = builder.set_default("progress_progress", "=").unwrap();
    builder = builder.set_default("progress_empty", " ").unwrap();
    builder = builder.set_default("progress_target_length", 20).unwrap();
    builder = builder.set_default("use_ibis", false).unwrap();
    builder = builder.set_default("suppress_errors", true).unwrap();

    builder = builder.set_default("percentage_color_thresholds.75", "brightgreen").unwrap();
    builder = builder.set_default("percentage_color_thresholds.85", "brightyellow").unwrap();
    builder = builder.set_default("percentage_color_thresholds.90", "brightred").unwrap();

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
    builder = builder.set_default("gpu.amd_accuracy", true).unwrap();
    builder = builder.set_default("gpu.ignore_disabled_gpus", true).unwrap();
    builder = builder.set_default("gpu.title", "GPU").unwrap();
    builder = builder.set_default("gpu.format", "{vendor} {model} ({vram})").unwrap();

    builder = builder.set_default("memory.title", "Memory").unwrap();
    builder = builder.set_default("memory.format", "{used} / {max} ({percent})").unwrap();

    builder = builder.set_default("swap.title", "Swap").unwrap();
    builder = builder.set_default("swap.format", "{used} / {total} ({percent})").unwrap();

    builder = builder.set_default("mounts.title", "Disk ({mount})").unwrap();
    builder = builder.set_default("mounts.format", "{space_used} used of {space_total} ({percent}) [{filesystem}]").unwrap();
    builder = builder.set_default("mounts.ignore", vec!["/boot", "/snap"]).unwrap();

    builder = builder.set_default("host.title", "Host").unwrap();

    builder = builder.set_default("displays.title", "Display ({name})").unwrap();
    builder = builder.set_default("displays.format", "{width}x{height} @ {refresh_rate}Hz").unwrap();

    builder = builder.set_default("os.title", "Operating System").unwrap();
    builder = builder.set_default("os.format", "{distro} ({kernel})").unwrap();

    builder = builder.set_default("packages.title", "Packages").unwrap();
    builder = builder.set_default("packages.format", "{count} ({manager})").unwrap();

    builder = builder.set_default("desktop.title", "Desktop").unwrap();
    builder = builder.set_default("desktop.format", "{desktop} ({display_type})").unwrap();

    builder = builder.set_default("terminal.title", "Terminal").unwrap();
    builder = builder.set_default("terminal.chase_ssh_pts", false).unwrap();

    builder = builder.set_default("shell.title", "Shell").unwrap();
    builder = builder.set_default("shell.format", "{shell}").unwrap();
    builder = builder.set_default("shell.show_default_shell", "false").unwrap();

    builder = builder.set_default("uptime.title", "Uptime").unwrap();

    builder = builder.set_default("battery.title", "Battery").unwrap();
    builder = builder.set_default("battery.format", "{percentage}%").unwrap();
    builder = builder.set_default("battery.path", "BAT0").unwrap();

    builder = builder.set_default("editor.title", "Editor").unwrap();
    builder = builder.set_default("editor.format", "{name}").unwrap();
    builder = builder.set_default("editor.fancy", true).unwrap();

    builder = builder.set_default("locale.title", "Locale").unwrap();
    builder = builder.set_default("locale.format", "{language} ({encoding})").unwrap();

    builder = builder.set_default("music.title", "Music").unwrap();
    builder = builder.set_default("music.format", "{track} by {track_artists} ({album})").unwrap();
    builder = builder.set_default("music.player", "spotify").unwrap();

    builder = builder.set_default("initsys.title", "Init System").unwrap();

    builder = builder.set_default("processes.title", "Total Processes").unwrap();

    // Check for any module overrides
    if module_override.is_some() {
        let module_override: String = module_override.clone().unwrap();
        builder = builder.set_override("modules", module_override.split(',').collect::<Vec<&str>>()).unwrap();
    }

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
const DEFAULT_CONFIG_CONTENTS: &str = r#"# For more in-depth configuration docs, please view https://github.com/LivacoNew/CrabFetch/wiki


# The modules to display and in what order.
# All modules; space, underline:{length}, segment:{name}, end_segment, hostname, cpu, gpu, memory, swap, mounts, host, displays, os, packages, desktop, terminal, shell, battery, uptime, locale, editor, colors, bright_colors
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
    "locale",
    "music",
    "initsys",
    "processes",

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
underline_character = '―'

# Format of segments
# Segments can be defined in the modules array
segment_top = "{color-white}[======------{color-brightmagenta} {name} {color-white}------======]"
segment_bottom = "{color-white}[======------{color-brightmagenta} {name_sized_gap} {color-white}------======]"

# Formatting characters used in progress bars 
progress_left_border = '['
progress_right_border = ']'
progress_progress = '='
progress_empty = ' '
# The target length of the progress bar
progress_target_length = 20

# Whether to use 'ibibytes opposed to 'gabytes 
# E.g use Gibibytes (GiB) opposed to Gigabytes (GB)
use_ibis = false

# Whether to supress any errors that come or not
suppress_errors = true

# Percentage coloring thresholds 
# Empty this section to make it not color 
[percentage_color_thresholds]
75 = "brightgreen"
85 = "brightyellow"
90 = "brightred"


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
# Also remember that you can override some stuff on these, e.g the title formatting. Again check the wiki.

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

# Whether to try to search a seperate AMD specific file to try to improve accuracy on AMD GPU's 
# Only does anything with pcisysfile method 
amd_accuracy = true

# Ignore any GPU's that are marked as "disabled" by Linux
ignore_disabled_gpus = true


title = "GPU"
# Placeholders;
# {vendor} -> The vendor of the GPU, e.g AMD
# {model} -> The model of the GPU, e.g Radeon RX 7800XT
# {vram} -> The total memory of the GPU.
format = "{vendor} {model} ({vram})"


[memory]
title = "Memory"
# Placeholders;
# {used} -> The currently in-use memory.
# {max} -> The maximum total memory.
# {bar} -> A progress bar representing the total space available/taken.
# {percent} -> Percentage of memory used
format = "{used} / {max} ({percent})"


[swap]
title = "Swap"
# Placeholders;
# {used} -> The currently used swap.
# {max} -> The maximum total swap.
# {bar} -> A progress bar representing the total space available/taken.
# {percent} -> Percentage of swap used
format = "{used} / {total} ({percent})"


[mounts]
# Each mount has it's own entry. Title Placeholders;
# {device} -> Device, e.g /dev/sda
# {mount} -> The mount point, e.g /home
# {filesystem} -> The filesystem running on that mount.
title = "Disk ({mount})"

# Placeholders;
# {device} -> Device, e.g /dev/sda
# {mount} -> The mount point, e.g /home
# {space_used} -> The space used.
# {space_avail} -> The space available.
# {space_total} -> The total space.
# {filesystem} -> The filesystem running on that mount.
# {bar} -> A progress bar representing the total space available/taken.
# {percent} -> The percentage of the disk used.
format = "{space_used} used of {space_total} ({percent}) [{filesystem}]"

# Mounts that shouldn't be included
# The mounts only need to start with these
ignore = ["/boot", "/snap"]


[host]
title = "Host"


[displays]
# Same as mounts. Placeholders;
# {name} -> The monitor name, e.g eDP-2
title = "Display ({name})"

# The format each display should be in. Placeholders;
# {name} -> The monitor "name", e.g eDP-2 for Wayland and 412 for x11.
# {width} -> The monitor's width
# {height} -> The monitor's height
# {refresh_rate} -> The monitor's refresh rate. This won't work in x11!
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
# Whether to find the name of the current PTS if SSH is being used. This is a togglable option as most people probably won't care to go hunting for it.
chase_ssh_pts = false


[shell]
title = "Shell"
# Placeholders;
# {shell} -> The name of the shell, e.g zsh
format = "{shell}"

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
# Placeholders;
# {language} - The selected language
# {encoding} - The encoding selected, most likely UTF-8
format = "{language} ({encoding})"


[music]
title = "Music"
# Placeholders;
# {track} - The name of the track
# {album} - The name of the album
# {track_artists} - The names of all track artists
# {album_artists} - The names of all album artists
format = "{track} by {track_artists} ({album})"

# The player to get music data from
# The player must support the MPRIS standard (most do.)
# Known good selections; (make a GitHub issue if a known good one needs added here)
# - spotify
player = "spotify"


[battery]
title = "Battery"
# Placeholders;
# {percentage} -> The battery percentage
format = "{percentage}%"


[initsys]
title = "Init System"


[processes]
title = "Total Processes"



# You've reached the end! Congrats, have a muffin :)"#;
