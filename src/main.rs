use std::{cmp::max, env, process::exit};

use clap::Parser;
use colored::{ColoredString, Colorize};
use hostname::HostnameInfo;
use shell::ShellInfo;

use crate::{config_manager::{color_string, Configuration}, cpu::CPUInfo, desktop::DesktopInfo, gpu::GPUInfo, host::HostInfo, memory::MemoryInfo, mounts::MountInfo, os::OSInfo, packages::PackagesInfo, swap::SwapInfo, terminal::TerminalInfo, uptime::UptimeInfo};

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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    /// Sets a custom config file. This file MUST be a .toml file.
    config: Option<String>,
    #[arg(short, long)]
    /// Ignores the GPU Info cache at /tmp/crabfetch-gpu - This will make CrabFetch a bit slower as
    /// glxinfo is slow!
    ignore_cache: bool
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

fn style_entry(title: &str, format: &str, config: &Configuration, module: &impl Module) -> String {
    let mut str: String = String::new();
    let mut title: ColoredString = config_manager::color_string(title, &config.title_color);
    if title.trim() != "" {
        if config.title_bold {
            title = title.bold();
        }
        if config.title_italic {
            title = title.italic();
        }
        str.push_str(&title.to_string());
        str.push_str(&config.seperator);
    }
    str.push_str(&module.format(format, config.decimal_places));
    str
}

fn main() {
    // Are we defo in Linux?
    if env::consts::OS != "linux" {
        println!("CrabFetch only supports Linux! If you want to go through and add support for your own OS, make a pull request :)");
        exit(-1);
    }

    let args = Args::parse();
    let mut config: Configuration = config_manager::parse(args.config);

    // Since we parse the os-release file in OS anyway, this is always called to get the
    // ascii we want.
    let os: OSInfo = os::get_os();
    let mut ascii: (String, u16) = (String::new(), 0);
    if config.ascii_display {
        ascii = ascii::get_ascii(&os.distro_id);
    }

    let mut line_number: u8 = 0;
    let target_length: u16 = ascii.1 + config.ascii_margin;

    let mut split: Vec<&str> = ascii.0.split("\n").collect();
    if split.len() < config.modules.len() {
        // Artificially add length so that all the modules get in
        for _ in 0..(config.modules.len() - split.len()) {
            split.insert(split.len(), "");
        }
    }

    // Figure out how many total lines we have
    let line_count = max(split.len(), config.modules.len());

    // Drives also need to be treated specially since they need to be on a seperate line
    // So we parse them already up here too, and just increase the index each time the module is
    // called.
    let mut mounts: Vec<MountInfo> = mounts::get_mounted_drives();
    mounts.retain(|x| !x.is_ignored(&config));
    let mut mount_index: u32 = 0;
    for x in 0..line_count + 1 {
        let mut line = "";
        if split.len() > x {
            line = split[x]
        }

        // Figure out the color first
        let percentage: f32 = (line_number as f32 / split.len() as f32) as f32;
        // https://stackoverflow.com/a/68457573
        let index: u8 = (((config.ascii_colors.len() - 1) as f32) * percentage).round() as u8;
        let colored: ColoredString = color_string(line, config.ascii_colors.get(index as usize).unwrap());

        // Print the actual ASCII
        print!("{}", colored);
        let remainder: u16 = target_length - (line.len() as u16);
        for _ in 0..remainder {
            print!(" ");
        }

        if config.modules.len() > line_number as usize {
            let module: String = config.modules[line_number as usize].to_owned();
            // print!("{}", module);
            match module.as_str() {
                "space" => {
                    print!("");
                },
                "hostname" => {
                    // Pretty much reimplements style_entry
                    // Sorry DRY enthusiasts
                    let mut str: String = String::new();
                    let mut title: ColoredString = config_manager::color_string(&config.hostname_title, &config.title_color);
                    if title.trim() != "" {
                        if config.title_bold {
                            title = title.bold();
                        }
                        if config.title_italic {
                            title = title.italic();
                        }
                        str.push_str(&title.to_string());
                        str.push_str(&config.seperator);
                    }

                    let hostname: HostnameInfo = hostname::get_hostname();
                    if config.hostname_color {
                        str.push_str(&hostname.format_colored(&config.hostname_format, config.decimal_places, &config.title_color));
                    } else {
                        str.push_str(&hostname.format(&config.hostname_format, config.decimal_places));
                    }

                    print!("{}", str);
                },
                "underline" => {
                    for _ in 0..config.underline_length {
                        print!("-");
                    }
                }
                "cpu" => {
                    let cpu: CPUInfo = cpu::get_cpu();
                    print!("{}", style_entry(&config.cpu_title, &config.cpu_format, &config, &cpu));
                },
                "memory" => {
                    let memory: MemoryInfo = memory::get_memory();
                    print!("{}", style_entry(&config.memory_title, &config.memory_format, &config, &memory));
                }
                "swap" => {
                    let swap: SwapInfo = swap::get_swap();
                    print!("{}", style_entry(&config.swap_title, &config.swap_format, &config, &swap));
                }
                "gpu" => {
                    let gpu: GPUInfo = gpu::get_gpu(args.ignore_cache);
                    print!("{}", style_entry(&config.gpu_title, &config.gpu_format, &config, &gpu));
                },
                "os" => {
                    print!("{}", style_entry(&config.os_title, &config.os_format, &config, &os));
                }
                "terminal" => {
                    let terminal: TerminalInfo = terminal::get_terminal();
                    print!("{}", style_entry(&config.terminal_title, &config.terminal_format, &config, &terminal));
                },
                "host" => {
                    let host: HostInfo = host::get_host();
                    print!("{}", style_entry(&config.host_title, &config.host_format, &config, &host));
                },
                "uptime" => {
                    let uptime: UptimeInfo = uptime::get_uptime();
                    print!("{}", style_entry(&config.uptime_title, &config.uptime_format, &config, &uptime));
                }
                "desktop" => {
                    let desktop: DesktopInfo = desktop::get_desktop();
                    print!("{}", style_entry(&config.desktop_title, &config.desktop_format, &config, &desktop));
                }
                "shell" => {
                    let shell: ShellInfo = shell::get_shell();
                    print!("{}", style_entry(&config.shell_title, &config.shell_format, &config, &shell));
                }
                "packages" => {
                    let packages: PackagesInfo = packages::get_packages();
                    print!("{}", style_entry(&config.packages_title, &config.packages_format, &config, &packages));
                }
                "mounts" => {
                    if mounts.len() > mount_index as usize {
                        let mount: &MountInfo = mounts.get(mount_index as usize).unwrap();
                        let title: String = mount.format(&config.mount_title, 0);
                        print!("{}", style_entry(&title, &config.mount_format, &config, mount));
                        mount_index += 1;
                        // sketchy - this is what makes it go through them all
                        if mounts.len() > mount_index as usize {
                            config.modules.insert(line_number as usize, "mounts".to_string());
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
        println!();
    }
}
