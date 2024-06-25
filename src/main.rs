use std::{cmp::max, env, fmt::{Debug, Display}, process::exit};

use config_manager::{color_string, CrabFetchColor};
use cpu::CPUInfo;
use clap::{ArgAction, Parser};
use colored::{ColoredString, Colorize};
use os::OSInfo;

use crate::{config_manager::Configuration, hostname::HostnameInfo};

mod cpu;
mod config_manager;
mod ascii;
mod os;
mod hostname;

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

    #[arg(short, long, require_equals(true), default_missing_value("false"), default_value("false"), action=ArgAction::Set)]
    /// Whether to suppress any errors or not.
    suppress_errors: bool,

    #[arg(long)]
    /// Overrides the modules set in your config file. This should be a comma seperated list of
    /// modules. E.g cpu,gpu,underline:16,title
    module_override: Option<String>
}

// Figures out the max title length for when we're using inline value display
fn calc_max_title_length(config: &Configuration) -> u64 {
    let mut res: u64 = 0;
    // this kinda sucks
    for module in &config.modules {
        match module.as_str() {
            "hostname" => res = max(res, config.hostname.title.len() as u64),
            "cpu" => res = max(res, config.cpu.title.len() as u64),
            // "gpu" => res = max(res, config.gpu.title.len() as u64),
            // "memory" => res = max(res, config.memory.title.len() as u64),
            // "swap" => res = max(res, config.swap.title.len() as u64),
            // "mounts" => res = max(res, config.mounts.title.len() as u64),
            // "host" => res = max(res, config.host.title.len() as u64),
            // "displays" => res = max(res, config.displays.title.len() as u64),
            "os" => res = max(res, config.os.title.len() as u64),
            // "packages" => res = max(res, config.packages.title.len() as u64),
            // "desktop" => res = max(res, config.desktop.title.len() as u64),
            // "terminal" => res = max(res, config.terminal.title.len() as u64),
            // "shell" => res = max(res, config.shell.title.len() as u64),
            // "battery" => res = max(res, config.battery.title.len() as u64),
            // "uptime" => res = max(res, config.uptime.title.len() as u64),
            // "locale" => res = max(res, config.locale.title.len() as u64),
            // "editor" => res = max(res, config.editor.title.len() as u64),
            _ => {}
        }
    }

    res
}

trait Module {
    fn new() -> Self;
    fn style(&self, config: &Configuration, max_title_length: u64) -> String;
    fn unknown_output(config: &Configuration, max_title_length: u64) -> String;
    fn replace_placeholders(&self, config: &Configuration) -> String;

    // This helps the format function lol
    fn round(number: f32, places: u32) -> f32 {
        let power: f32 = 10_u32.pow(places) as f32;
        (number * power).round() / power
    }
    // TODO: Move these params into some kinda struct or some shit idk, cus it just sucks
    fn default_style(config: &Configuration, max_title_len: u64, title: &str, title_color: &CrabFetchColor, title_bold: bool, title_italic: bool, seperator: &str, value: &str) -> String {
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
            if config.inline_values {
                for _ in 0..(max_title_len - (title.len() as u64)) {
                    str.push_str(" ");
                }
            }
            str.push_str(seperator);
        }

        str.push_str(value);

        str
    }
    fn replace_color_placeholders(&self, str: &String) -> String {
        config_manager::replace_color_placeholders(str)
    }
}

// A generic module error
struct ModuleError {
    module_name: String,
    message: String
}
impl ModuleError {
    pub fn new(module: &str, message: String) -> ModuleError {
        ModuleError {
            module_name: module.to_string(),
            message
        }
    }
}
impl Display for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module {} failed: {}", self.module_name, self.message)
    }
}
impl Debug for ModuleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Module {} failed: {}", self.module_name, self.message)
    }
}


// Stores all the module's outputs as we know them
// This is to prevent us doing additional work when we don't need to, when modules need shared data
struct ModuleOutputs {
    hostname: Option<Result<HostnameInfo, ModuleError>>,
    cpu: Option<Result<CPUInfo, ModuleError>>,
    os: Option<Result<OSInfo, ModuleError>>
}
impl ModuleOutputs {
    fn new() -> Self {
        Self {
            hostname: None,
            cpu: None,
            os: None
        }
    }
}

fn main() {
    // Are we defo in Linux?
    if env::consts::OS != "linux" {
        println!("CrabFetch only supports Linux! If you want to go through and add support for your own OS, make a pull request :)");
        exit(-1);
    }

    // 
    //  Parse 
    //

    // Get the args/config stuff out of the way
    let args: Args = Args::parse();
    if args.generate_config_file {
        config_manager::generate_config_file(args.config.clone());
        exit(0);
    }
    let config: Configuration = config_manager::parse(&args.config, &args.module_override, &args.ignore_config_file);
    let log_errors: bool = !(config.suppress_errors || args.suppress_errors);
    let max_title_length: u64 = calc_max_title_length(&config);

    // Define our module outputs, and figure out which modules need pre-calculated 
    let mut known_outputs: ModuleOutputs = ModuleOutputs::new();
    let mut ascii: (String, u16) = (String::new(), 0);
    if config.ascii.display {
        // OS needs done 
        let os: Result<OSInfo, ModuleError> = os::get_os();
        known_outputs.os = Some(os);
        if known_outputs.os.as_ref().unwrap().is_ok() {
            // Calculate the ASCII stuff while we're here
            if args.distro_override.is_some() {
                ascii = ascii::get_ascii(&args.distro_override.clone().unwrap());
            } else {
                ascii = ascii::get_ascii(&known_outputs.os.as_ref().unwrap().as_ref().unwrap().distro_id);
            }
        }
    }

    // 
    //  Detect
    //
    let mut output: Vec<String> = Vec::new();
    for module in &config.modules {
        let module_split: Vec<&str> = module.split(":").collect();
        let module_name: &str = module_split[0];
        match module_name {
            "space" => output.push("".to_string()),
            "underline" => {
                let underline_length: usize = module_split[1].parse().unwrap();
                output.push(config.underline_character.to_string().repeat(underline_length));
            },
            "hostname" => {
                if known_outputs.hostname.is_none() {
                    known_outputs.hostname = Some(hostname::get_hostname());
                }
                match known_outputs.hostname.as_ref().unwrap() {
                    Ok(hostname) => {
                        output.push(hostname.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(HostnameInfo::unknown_output(&config, max_title_length));
                        }
                    },
                };
            },
            "cpu" => {
                if known_outputs.cpu.is_none() {
                    known_outputs.cpu = Some(cpu::get_cpu());
                }
                match known_outputs.cpu.as_ref().unwrap() {
                    Ok(cpu) => {
                        output.push(cpu.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(CPUInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
            },
            "os" => {
                if known_outputs.os.is_none() {
                    known_outputs.os = Some(os::get_os());
                }
                match known_outputs.os.as_ref().unwrap() {
                    Ok(os) => {
                        output.push(os.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(OSInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
            },
            _ => {

            }
        }
    }


    // 
    //  Display
    //
    let mut ascii_split: Vec<&str> = Vec::new();
    let mut ascii_length: usize = 0;
    let mut ascii_target_length: u16 = 0;
    if config.ascii.display {
        ascii_split = ascii.0.split("\n").filter(|x| x.trim() != "").collect();
        ascii_length = ascii_split.len();
        ascii_target_length = ascii.1 + config.ascii.margin;
    }

    let mut current_line: usize = 0;
    for out in output {
        if config.ascii.display {
            // Figure out the color first
            print!("{}", get_ascii_line(current_line, &ascii_split, &ascii_target_length, &config));
        }

        print!("{}", out);
        current_line += 1;
        println!();
    }
    if current_line < ascii_length && config.ascii.display {
        for _ in current_line..ascii_length {
            print!("{}", get_ascii_line(current_line, &ascii_split, &ascii_target_length, &config));
            current_line += 1;
            println!();
        }
    }
}

fn get_ascii_line(current_line: usize, ascii_split: &Vec<&str>, target_length: &u16, config: &Configuration) -> String {
    let percentage: f32 = (current_line as f32 / ascii_split.len() as f32) as f32;
    let index: u8 = (((config.ascii.colors.len() - 1) as f32) * percentage).round() as u8;

    let mut line = String::new();
    if ascii_split.len() > current_line {
        line = ascii_split[current_line].to_string();
    }
    let remainder: u16 = target_length - (line.chars().collect::<Vec<char>>().len() as u16);
    for _ in 0..remainder {
        line.push_str(" ");
    }
    let colored: ColoredString = color_string(&line, config.ascii.colors.get(index as usize).unwrap());

    return colored.to_string();
}
