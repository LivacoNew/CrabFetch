use std::{cmp::max, env, process::exit};

use config_manager::CrabFetchColor;
use lazy_static::lazy_static;
use clap::{ArgAction, Parser};
use colored::{ColoredString, Colorize};

use crate::{config_manager::{color_string, Configuration}, displays::DisplayInfo, mounts::MountInfo, os::OSInfo};

mod cpu;
mod memory;
mod config_manager;
mod ascii;
mod hostname;
mod os;
mod uptime;
mod desktop;
mod mounts;
mod shell;
mod swap;
mod gpu;
mod terminal;
mod host;
mod packages;
mod displays;
mod battery;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    /// Sets a custom config file. This file MUST be a .toml file.
    config: Option<String>,

    #[arg(short, long)]
    /// Ignores a config file if present, and sticks to the default configuration.
    ignore_config_file: bool,

    #[arg(short, long)]
    /// Generates a default config file
    generate_config_file: bool,

    #[arg(long)]
    /// Ignores the GPU Info cache at /tmp/crabfetch-gpu
    ignore_cache: bool,

    #[arg(long)]
    /// Sets the GPU method to use. Can either be "glxinfo" or "pcisysfile"
    gpu_method: Option<String>,

    #[arg(short, long)]
    /// Overrides the distro ASCII to another distro.
    distro_override: Option<String>,

    #[arg(short, long, require_equals(true), default_missing_value("true"), default_value("true"), action=ArgAction::Set)]
    /// Whether to suppress any errors or not.
    suppress_errors: bool,

    #[arg(long)]
    /// Overrides the modules set in your config file. This should be a comma seperated list of
    /// modules. E.g cpu,gpu,underline:16,title
    module_override: Option<String>
}

fn calc_max_title_length() -> u64 {
    let mut res: u64 = 0;
    // this kinda sucks
    for module in &CONFIG.modules {
        match module.as_str() {
            "hostname" => res = max(res, CONFIG.hostname.title.len() as u64),
            "cpu" => res = max(res, CONFIG.cpu.title.len() as u64),
            "gpu" => res = max(res, CONFIG.gpu.title.len() as u64),
            "memory" => res = max(res, CONFIG.memory.title.len() as u64),
            "swap" => res = max(res, CONFIG.swap.title.len() as u64),
            "mounts" => res = max(res, CONFIG.mounts.title.len() as u64),
            "host" => res = max(res, CONFIG.host.title.len() as u64),
            "displays" => res = max(res, CONFIG.displays.title.len() as u64),
            "os" => res = max(res, CONFIG.os.title.len() as u64),
            "packages" => res = max(res, CONFIG.packages.title.len() as u64),
            "desktop" => res = max(res, CONFIG.desktop.title.len() as u64),
            "terminal" => res = max(res, CONFIG.terminal.title.len() as u64),
            "shell" => res = max(res, CONFIG.shell.title.len() as u64),
            "battery" => res = max(res, CONFIG.battery.title.len() as u64),
            "uptime" => res = max(res, CONFIG.uptime.title.len() as u64),
            _ => {}
        }
    }

    res
}
lazy_static! {
    pub static ref ARGS: Args = Args::parse();
    pub static ref CONFIG: Configuration = config_manager::parse(&ARGS.config, &ARGS.ignore_config_file);
    pub static ref MAX_TITLE_LENGTH: u64 = calc_max_title_length();
}

trait Module {
    fn new() -> Self;
    fn style(&self) -> String;
    fn replace_placeholders(&self) -> String;

    // This helps the format function lol
    fn round(number: f32, places: u32) -> f32 {
        let power: f32 = 10_u32.pow(places) as f32;
        (number * power).round() / power
    }
    fn default_style(&self, title: &str, title_color: &CrabFetchColor, title_bold: bool, title_italic: bool, seperator: &str) -> String {
        let mut str: String = String::new();

        // Title
        if !title.trim().is_empty() {
            let mut title: ColoredString = config_manager::color_string(title, title_color);
            if title_bold {
                title = title.bold();
            }
            if title_italic {
                title = title.italic();
            }

            str.push_str(&title.to_string());
            // Inline value stuff
            if CONFIG.inline_values {
                for _ in 0..(*MAX_TITLE_LENGTH - (title.len() as u64)) {
                    str.push_str(" ");
                }
            }
            str.push_str(seperator);
        }

        let mut value: String = self.replace_placeholders();
        value = self.replace_color_placeholders(&value);
        str.push_str(&value.to_string());

        str
    }
    fn replace_color_placeholders(&self, str: &String) -> String {
        config_manager::replace_color_placeholders(str)
    }
}

fn log_error(module: &str, message: String) {
    if CONFIG.suppress_errors && ARGS.suppress_errors {
        return
    }

    println!("Module {}: {}", module, message);
}


fn main() {
    // Are we defo in Linux?
    if env::consts::OS != "linux" {
        println!("CrabFetch only supports Linux! If you want to go through and add support for your own OS, make a pull request :)");
        exit(-1);
    }

    if ARGS.generate_config_file {
        config_manager::generate_config_file(ARGS.config.clone());
        exit(0);
    }

    // Since we parse the os-release file in OS anyway, this is always called to get the
    // ascii we want.
    let os: OSInfo = os::get_os();
    let mut ascii: (String, u16) = (String::new(), 0);
    if CONFIG.ascii.display {
        if ARGS.distro_override.is_some() {
            ascii = ascii::get_ascii(&ARGS.distro_override.clone().unwrap());
        } else {
            ascii = ascii::get_ascii(&os.distro_id);
        }
    }

    let mut line_number: u8 = 0;
    let mut ascii_line_number: u8 = 0;
    let target_length: u16 = ascii.1 + CONFIG.ascii.margin;

    let split: Vec<&str> = ascii.0.split("\n").filter(|x| x.trim() != "").collect();

    // Figure out how many total lines we have
    let mut modules = CONFIG.modules.clone();
    let mut module_count = modules.len();

    // Drives also need to be treated specially since they need to be on a seperate line
    // So we parse them already up here too, and just increase the index each time the module is
    // called.
    let mut mounts: Option<Vec<MountInfo>> = None;
    let mut mount_index: u32 = 0;
    if modules.contains(&"mounts".to_string()) {
        mounts = Some(mounts::get_mounted_drives());
        mounts.as_mut().unwrap().retain(|x| !x.is_ignored());
        module_count += mounts.as_ref().unwrap().len() - 1
    }

    // AND displays
    let mut displays: Option<Vec<DisplayInfo>> = None;
    let mut display_index: u32 = 0;
    if modules.contains(&"displays".to_string()) {
        displays = Some(displays::get_displays());
        module_count += 1;
    }

    let line_count = max(split.len(), module_count);

    for x in 0..line_count {
        let mut line = "";
        if split.len() > x {
            line = split[x]
        }

        // Figure out the color first
        let percentage: f32 = (ascii_line_number as f32 / split.len() as f32) as f32;
        // https://stackoverflow.com/a/68457573
        let index: u8 = (((CONFIG.ascii.colors.len() - 1) as f32) * percentage).round() as u8;
        let colored: ColoredString = color_string(line, CONFIG.ascii.colors.get(index as usize).unwrap());

        // Print the actual ASCII
        print!("{}", colored);
        if colored.trim().len() != 0 {
            // We're still going
            ascii_line_number = ascii_line_number + 1;
        }
        let remainder: u16 = target_length - (line.chars().collect::<Vec<char>>().len() as u16);
        for _ in 0..remainder {
            print!(" ");
        }

        if modules.len() > line_number as usize {
            let module_split: Vec<&str> = modules[line_number as usize].split(":").collect();
            let module: String = module_split[0].to_string();
            // print!("{}", module);
            match module.as_str() {
                "space" => print!(""),
                "underline" => {
                    let underline_length: u16 = module_split[1].parse().unwrap();
                    for _ in 0..underline_length {
                        print!("{}", CONFIG.underline_character);
                    }
                }

                // Segments
                // This is very crudely done for now but I'll expand it at a later date
                "segment" => {
                    let segment_name: &str = module_split[1];
                    let mut str: String = CONFIG.segment_top.replace("{name}", segment_name);
                    str = config_manager::replace_color_placeholders(&str);
                    print!("{}", str);
                }

                "hostname" => print!("{}", hostname::get_hostname().style()),
                "cpu" => print!("{}", cpu::get_cpu().style()),
                "gpu" => print!("{}", gpu::get_gpu().style()),
                "memory" => print!("{}", memory::get_memory().style()),
                "host" => print!("{}", host::get_host().style()),
                "swap" => print!("{}", swap::get_swap().style()),
                "mounts" => {
                    let mounts: &Vec<MountInfo> = mounts.as_ref().unwrap();
                    if mounts.len() > mount_index as usize {
                        let mount: &MountInfo = mounts.get(mount_index as usize).unwrap();
                        print!("{}", mount.style());
                        mount_index += 1;
                        // sketchy - this is what makes it go through them all
                        if mounts.len() > mount_index as usize {
                            modules.insert(line_number as usize, "mounts".to_string());
                        }
                    }
                }
                "os" => print!("{}", os.style()),
                "packages" => print!("{}", packages::get_packages().style()),
                "desktop" => print!("{}", desktop::get_desktop().style()),
                "terminal" => print!("{}", terminal::get_terminal().style()),
                "shell" => print!("{}", shell::get_shell().style()),
                "uptime" => print!("{}", uptime::get_uptime().style()),
                "displays" => {
                    let displays: &Vec<DisplayInfo> = displays.as_ref().unwrap();
                    if displays.len() > display_index as usize {
                        let display: &DisplayInfo = displays.get(display_index as usize).unwrap();
                        print!("{}", display.style());
                        display_index += 1;
                        // once again, sketchy
                        if displays.len() > display_index as usize {
                            modules.insert(line_number as usize, "displays".to_string());
                        }
                    }
                }
                "battery" => print!("{}", battery::get_battery().style()),
                "colors" => {
                    let str = "   ";
                    print!("{}", str.on_black());
                    print!("{}", str.on_red());
                    print!("{}", str.on_green());
                    print!("{}", str.on_yellow());
                    print!("{}", str.on_blue());
                    print!("{}", str.on_magenta());
                    print!("{}", str.on_cyan());
                    print!("{}", str.on_white());
                }
                "bright_colors" => {
                    let str = "   ";
                    print!("{}", str.on_bright_black());
                    print!("{}", str.on_bright_red());
                    print!("{}", str.on_bright_green());
                    print!("{}", str.on_bright_yellow());
                    print!("{}", str.on_bright_blue());
                    print!("{}", str.on_bright_magenta());
                    print!("{}", str.on_bright_cyan());
                    print!("{}", str.on_bright_white());
                }
                _ => print!("Unknown module: {}", module),
            }
        }
        line_number = line_number + 1;

        if line_number != (line_count + 1) as u8 {
            print!("\n");
        }
    }
}
