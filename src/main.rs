use std::{cmp::max, env, process::exit};

use config_manager::CrabFetchColor;
use lazy_static::lazy_static;
use clap::{ArgAction, Parser};
use colored::{ColoredString, Colorize};

use crate::{battery::BatteryInfo, config_manager::{color_string, Configuration}, cpu::CPUInfo, desktop::DesktopInfo, displays::DisplayInfo, gpu::GPUInfo, host::HostInfo, memory::MemoryInfo, mounts::MountInfo, os::OSInfo, packages::PackagesInfo, shell::ShellInfo, swap::SwapInfo, terminal::TerminalInfo, uptime::UptimeInfo};

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
}

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
    pub static ref CONFIG: Configuration = config_manager::parse(&ARGS.config, &ARGS.ignore_config_file);
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
            str.push_str(seperator);
        }

        let mut value: String = self.replace_placeholders();
        value = self.replace_color_placeholders(&value); // TODO
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

    let split: Vec<&str> = ascii.0.split("\n").collect();

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
        module_count += 1;
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
                "space" => {
                    print!("");
                },
                "underline" => {
                    let underline_length: u16 = module_split[1].parse().unwrap();
                    for _ in 0..underline_length {
                        print!("-");
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

                "hostname" => {
                    let str: String = hostname::get_hostname().style();
                    print!("{}", str);
                },
                "cpu" => {
                    let cpu: CPUInfo = cpu::get_cpu();
                    print!("{}", cpu.style());
                },
                "gpu" => {
                    let gpu: GPUInfo = gpu::get_gpu();
                    print!("{}", gpu.style());
                },
                "memory" => {
                    let memory: MemoryInfo = memory::get_memory();
                    print!("{}", memory.style());
                }
                "host" => {
                    let host: HostInfo = host::get_host();
                    print!("{}", host.style());
                },
                "swap" => {
                    let swap: SwapInfo = swap::get_swap();
                    print!("{}", swap.style());
                }
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
                "os" => {
                    print!("{}", os.style());
                }
                "packages" => {
                    let packages: PackagesInfo = packages::get_packages();
                    print!("{}", packages.style());
                }
                "desktop" => {
                    let desktop: DesktopInfo = desktop::get_desktop();
                    print!("{}", desktop.style());
                }
                "terminal" => {
                    let terminal: TerminalInfo = terminal::get_terminal();
                    print!("{}", terminal.style());
                },
                "shell" => {
                    let shell: ShellInfo = shell::get_shell();
                    print!("{}", shell.style());
                }
                "uptime" => {
                    let uptime: UptimeInfo = uptime::get_uptime();
                    print!("{}", uptime.style());
                }
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
                "battery" => {
                    let battery: BatteryInfo = battery::get_battery();
                    print!("{}", battery.style());
                }
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
                _ => {
                    print!("Unknown module: {}", module);
                }
            }
        }
        line_number = line_number + 1;

        if line_number != (line_count + 1) as u8 {
            print!("\n");
        }
    }
}
