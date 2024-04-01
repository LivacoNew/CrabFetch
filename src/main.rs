use std::{cmp::max, env, process::exit};

use lazy_static::lazy_static;
use clap::Parser;
use colored::{ColoredString, Colorize};
use hostname::HostnameInfo;

use crate::{config_manager::{color_string, Configuration}, cpu::CPUInfo, desktop::DesktopInfo, displays::DisplayInfo, gpu::GPUInfo, host::HostInfo, memory::MemoryInfo, mounts::MountInfo, os::OSInfo, packages::PackagesInfo, shell::ShellInfo, swap::SwapInfo, terminal::TerminalInfo, uptime::UptimeInfo};

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
    /// Ignores the GPU Info cache at /tmp/crabfetch-gpu - This will make CrabFetch a bit slower as
    /// glxinfo is slow!
    ignore_cache: bool,

    #[arg(short, long)]
    /// Overrides the distro ASCII to another distro.
    distro_override: Option<String>,
}

trait Module {
    fn new() -> Self;
    fn format(&self, format: &str, float_decmials: u32) -> String;
    // This helps the format function lol
    fn round(number: f32, places: u32) -> f32 {
        let power: f32 = 10_u32.pow(places) as f32;
        (number * power).round() / power
    }
}

fn style_entry(title: &str, format: &str, module: &impl Module) -> String {
    let mut str: String = String::new();
    let mut title: ColoredString = config_manager::color_string(title, &CONFIG.title_color);
    if title.trim() != "" {
        if CONFIG.title_bold {
            title = title.bold();
        }
        if CONFIG.title_italic {
            title = title.italic();
        }
        str.push_str(&title.to_string());
        str.push_str(&CONFIG.seperator);
    }
    str.push_str(&module.format(format, CONFIG.decimal_places));
    str
}

lazy_static! {
    pub static ref ARGS: Args = Args::parse();
    pub static ref CONFIG: Configuration = config_manager::parse(&ARGS.config, &ARGS.ignore_config_file);
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
    if CONFIG.ascii_display {
        if ARGS.distro_override.is_some() {
            ascii = ascii::get_ascii(&ARGS.distro_override.clone().unwrap());
        } else {
            ascii = ascii::get_ascii(&os.distro_id);
        }
    }

    let mut line_number: u8 = 0;
    let mut ascii_line_number: u8 = 0;
    let target_length: u16 = ascii.1 + CONFIG.ascii_margin;

    let split: Vec<&str> = ascii.0.split("\n").collect();

    // Figure out how many total lines we have
    let mut modules = CONFIG.modules.clone();
    let mut line_count = max(split.len(), modules.len());


    // Drives also need to be treated specially since they need to be on a seperate line
    // So we parse them already up here too, and just increase the index each time the module is
    // called.
    let mut mounts: Option<Vec<MountInfo>> = None;
    let mut mount_index: u32 = 0;
    if modules.contains(&"mounts".to_string()) {
        mounts = Some(mounts::get_mounted_drives());
        mounts.as_mut().unwrap().retain(|x| !x.is_ignored(&CONFIG));
        line_count += mounts.as_ref().unwrap().len() - 1; // TODO: And me!
    }

    // AND displays
    let mut displays: Option<Vec<DisplayInfo>> = None;
    let mut display_index: u32 = 0;
    if modules.contains(&"displays".to_string()) {
        displays = Some(displays::get_displays());
        line_count += displays.as_ref().unwrap().len(); // TODO: Investigate me!
    }

    for x in 0..line_count {
        let mut line = "";
        if split.len() > x {
            line = split[x]
        }

        // Figure out the color first
        let percentage: f32 = (ascii_line_number as f32 / split.len() as f32) as f32;
        // https://stackoverflow.com/a/68457573
        let index: u8 = (((CONFIG.ascii_colors.len() - 1) as f32) * percentage).round() as u8;
        let colored: ColoredString = color_string(line, CONFIG.ascii_colors.get(index as usize).unwrap());

        // Print the actual ASCII
        print!("{}", colored);
        if colored.trim().len() != 0 {
            // We're still going
            ascii_line_number = ascii_line_number + 1;
        }
        let remainder: u16 = target_length - (line.len() as u16);
        for _ in 0..remainder {
            print!(" ");
        }

        if modules.len() > line_number as usize {
            let module: String = modules[line_number as usize].to_owned();
            // print!("{}", module);
            match module.as_str() {
                "space" => {
                    print!("");
                },
                "hostname" => {
                    // Pretty much reimplements style_entry
                    // Sorry DRY enthusiasts
                    let mut str: String = String::new();
                    let mut title: ColoredString = config_manager::color_string(&CONFIG.hostname_title, &CONFIG.title_color);
                    if title.trim() != "" {
                        if CONFIG.title_bold {
                            title = title.bold();
                        }
                        if CONFIG.title_italic {
                            title = title.italic();
                        }
                        str.push_str(&title.to_string());
                        str.push_str(&CONFIG.seperator);
                    }

                    let hostname: HostnameInfo = hostname::get_hostname();
                    if CONFIG.hostname_color {
                        str.push_str(&hostname.format_colored(&CONFIG.hostname_format, CONFIG.decimal_places, &CONFIG.title_color));
                    } else {
                        str.push_str(&hostname.format(&CONFIG.hostname_format, CONFIG.decimal_places));
                    }

                    print!("{}", str);
                },
                "underline" => {
                    for _ in 0..CONFIG.underline_length {
                        print!("-");
                    }
                }
                "cpu" => {
                    let cpu: CPUInfo = cpu::get_cpu();
                    print!("{}", style_entry(&CONFIG.cpu_title, &CONFIG.cpu_format, &cpu));
                },
                "memory" => {
                    let memory: MemoryInfo = memory::get_memory();
                    print!("{}", style_entry(&CONFIG.memory_title, &CONFIG.memory_format, &memory));
                }
                "swap" => {
                    let swap: SwapInfo = swap::get_swap();
                    print!("{}", style_entry(&CONFIG.swap_title, &CONFIG.swap_format, &swap));
                }
                "gpu" => {
                    let gpu: GPUInfo = gpu::get_gpu(ARGS.ignore_cache);
                    print!("{}", style_entry(&CONFIG.gpu_title, &CONFIG.gpu_format, &gpu));
                },
                "os" => {
                    print!("{}", style_entry(&CONFIG.os_title, &CONFIG.os_format, &os));
                }
                "terminal" => {
                    let terminal: TerminalInfo = terminal::get_terminal();
                    print!("{}", style_entry(&CONFIG.terminal_title, &CONFIG.terminal_format, &terminal));
                },
                "host" => {
                    let host: HostInfo = host::get_host();
                    print!("{}", style_entry(&CONFIG.host_title, &CONFIG.host_format, &host));
                },
                "uptime" => {
                    let uptime: UptimeInfo = uptime::get_uptime();
                    print!("{}", style_entry(&CONFIG.uptime_title, &CONFIG.uptime_format, &uptime));
                }
                "desktop" => {
                    let desktop: DesktopInfo = desktop::get_desktop();
                    print!("{}", style_entry(&CONFIG.desktop_title, &CONFIG.desktop_format, &desktop));
                }
                "shell" => {
                    let shell: ShellInfo = shell::get_shell();
                    print!("{}", style_entry(&CONFIG.shell_title, &CONFIG.shell_format, &shell));
                }
                "packages" => {
                    let packages: PackagesInfo = packages::get_packages();
                    print!("{}", style_entry(&CONFIG.packages_title, &CONFIG.packages_format, &packages));
                }
                "mounts" => {
                    let mounts: &Vec<MountInfo> = mounts.as_ref().unwrap();
                    if mounts.len() > mount_index as usize {
                        let mount: &MountInfo = mounts.get(mount_index as usize).unwrap();
                        let title: String = mount.format(&CONFIG.mount_title, 0);
                        print!("{}", style_entry(&title, &CONFIG.mount_format, mount));
                        mount_index += 1;
                        // sketchy - this is what makes it go through them all
                        if mounts.len() > mount_index as usize {
                            modules.insert(line_number as usize, "mounts".to_string());
                        }
                    }
                }
                "displays" => {
                    let displays: &Vec<DisplayInfo> = displays.as_ref().unwrap();
                    if displays.len() > display_index as usize {
                        let display: &DisplayInfo = displays.get(display_index as usize).unwrap();
                        let title: String = display.format(&CONFIG.display_title, 0);
                        print!("{}", style_entry(&title, &CONFIG.display_format, display));
                        display_index += 1;
                        // once again, sketchy
                        if displays.len() > display_index as usize {
                            modules.insert(line_number as usize, "displays".to_string());
                        }
                    }
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

        if line_number != (line_count -1) as u8 {
            print!("\n");
        }
    }
}
