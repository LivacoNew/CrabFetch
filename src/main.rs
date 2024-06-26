use std::{cmp::max, env, fmt::{Debug, Display}, process::exit, time::Instant};

use battery::BatteryInfo;
use config_manager::{color_string, CrabFetchColor};
use cpu::CPUInfo;
use clap::{ArgAction, Parser};
use colored::{ColoredString, Colorize};
use desktop::DesktopInfo;
use displays::DisplayInfo;
use editor::EditorInfo;
use gpu::{GPUInfo, GPUMethod};
use host::HostInfo;
use locale::LocaleInfo;
use memory::MemoryInfo;
use mounts::MountInfo;
use os::OSInfo;
use packages::PackagesInfo;
use shell::ShellInfo;
use swap::SwapInfo;
use terminal::TerminalInfo;
use uptime::UptimeInfo;

use crate::{config_manager::Configuration, hostname::HostnameInfo};

mod cpu;
mod config_manager;
mod ascii;
mod os;
mod hostname;
mod gpu;
mod memory;
mod swap;
mod mounts;
mod host;
mod displays;
mod packages;
mod desktop;
mod terminal;
mod shell;
mod uptime;
mod editor;
mod locale;
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

    #[arg(short, long, require_equals(true), default_missing_value("false"), default_value("false"), action=ArgAction::Set)]
    /// Whether to suppress any errors or not.
    suppress_errors: bool,

    #[arg(long)]
    /// Overrides the modules set in your config file. This should be a comma seperated list of
    /// modules. E.g cpu,gpu,underline:16,title
    module_override: Option<String>,

    #[arg(long)]
    /// Runs CrabFetch in a "benchmark" mode, showing the total times it takes between each stage
    /// and module detection times.
    benchmark: bool
}

// Figures out the max title length for when we're using inline value display
fn calc_max_title_length(config: &Configuration) -> u64 {
    let mut res: u64 = 0;
    // this kinda sucks
    for module in &config.modules {
        match module.as_str() {
            "hostname" => res = max(res, config.hostname.title.len() as u64),
            "cpu" => res = max(res, config.cpu.title.len() as u64),
            "gpu" => res = max(res, config.gpu.title.len() as u64),
            "memory" => res = max(res, config.memory.title.len() as u64),
            "swap" => res = max(res, config.swap.title.len() as u64),
            "mounts" => res = max(res, config.mounts.title.len() as u64),
            "host" => res = max(res, config.host.title.len() as u64),
            "displays" => res = max(res, config.displays.title.len() as u64),
            "os" => res = max(res, config.os.title.len() as u64),
            "packages" => res = max(res, config.packages.title.len() as u64),
            "desktop" => res = max(res, config.desktop.title.len() as u64),
            "terminal" => res = max(res, config.terminal.title.len() as u64),
            "shell" => res = max(res, config.shell.title.len() as u64),
            "battery" => res = max(res, config.battery.title.len() as u64),
            "uptime" => res = max(res, config.uptime.title.len() as u64),
            "locale" => res = max(res, config.locale.title.len() as u64),
            "editor" => res = max(res, config.editor.title.len() as u64),
            _ => {}
        }
    }

    res
}
// This is done here simply to make the main function not as indented of a mess, it's abstracted into here
fn benchmark_point(benchmarking: bool) -> Option<Instant> {
    if !benchmarking {return None;}
    return Some(Instant::now());
}
fn print_bench_time(benchmarking: bool, name: &str, time: Option<Instant>) {
    if !benchmarking {
        return;
    }
    if time.is_none() {
        return;
    }

    // This is different to module bench times fyi
    println!("[Benchmark] {}: {:2?}", name, time.unwrap().elapsed());
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
    gpu: Option<Result<GPUInfo, ModuleError>>,
    memory: Option<Result<MemoryInfo, ModuleError>>,
    swap: Option<Result<SwapInfo, ModuleError>>,
    mounts: Option<Result<Vec<MountInfo>, ModuleError>>,
    host: Option<Result<HostInfo, ModuleError>>,
    displays: Option<Result<Vec<DisplayInfo>, ModuleError>>,
    packages: Option<PackagesInfo>,
    desktop: Option<Result<DesktopInfo, ModuleError>>,
    terminal: Option<Result<TerminalInfo, ModuleError>>,
    shell: Option<Result<ShellInfo, ModuleError>>,
    battery: Option<Result<BatteryInfo, ModuleError>>,
    uptime: Option<Result<UptimeInfo, ModuleError>>,
    locale: Option<Result<LocaleInfo, ModuleError>>,
    editor: Option<Result<EditorInfo, ModuleError>>,
    os: Option<Result<OSInfo, ModuleError>>
}
impl ModuleOutputs {
    fn new() -> Self {
        Self {
            hostname: None,
            cpu: None,
            gpu: None,
            memory: None,
            swap: None,
            mounts: None,
            host: None,
            displays: None,
            packages: None,
            desktop: None,
            terminal: None,
            shell: None,
            battery: None,
            uptime: None,
            locale: None,
            editor: None,
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
    let args_bench: Option<Instant> = benchmark_point(true); // Just true as it's before we parse it
    let args: Args = Args::parse();
    print_bench_time(args.benchmark, "Args Parsing", args_bench);

    if args.generate_config_file {
        let bench: Option<Instant> = benchmark_point(args.benchmark); 
        config_manager::generate_config_file(args.config.clone());
        print_bench_time(args.benchmark, "Generating Config File", bench);
        exit(0);
    }
    let bench: Option<Instant> = benchmark_point(args.benchmark); 
    let config: Configuration = config_manager::parse(&args.config, &args.module_override, &args.ignore_config_file);
    print_bench_time(args.benchmark, "Parsing Config", bench);
    let log_errors: bool = !(config.suppress_errors || args.suppress_errors);
    let max_title_length: u64 = calc_max_title_length(&config);

    // Define our module outputs, and figure out which modules need pre-calculated 
    let bench: Option<Instant> = benchmark_point(args.benchmark); 
    let mut known_outputs: ModuleOutputs = ModuleOutputs::new();
    let mut ascii: (String, u16) = (String::new(), 0);
    if config.ascii.display {
        // OS needs done 
        let os_bench: Option<Instant> = benchmark_point(args.benchmark); 
        let os: Result<OSInfo, ModuleError> = os::get_os();
        print_bench_time(args.benchmark, "OS Module", os_bench);
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
    print_bench_time(args.benchmark, "ASCII (Potentially includes OS parse)", bench);

    // 
    //  Detect
    //
    let detect_bench: Option<Instant> = benchmark_point(args.benchmark); 
    let mut output: Vec<String> = Vec::new();
    for module in &config.modules {
        let module_parse_bench: Option<Instant> = benchmark_point(args.benchmark); 
        let module_split: Vec<&str> = module.split(":").collect();
        let module_name: &str = module_split[0];
        match module_name {
            "space" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                output.push("".to_string());
                print_bench_time(args.benchmark, "Space Module", bench);
            },
            "underline" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let underline_length: usize = module_split[1].parse().unwrap();
                output.push(config.underline_character.to_string().repeat(underline_length));
                print_bench_time(args.benchmark, "Underline Module", bench);
            },
            "segment" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let segment_name: &str = module_split[1];  
                let segment_string: String = config.segment_top.replace("{name}", segment_name);
                output.push(config_manager::replace_color_placeholders(&segment_string));
                print_bench_time(args.benchmark, "Segment Module", bench);
            },
            "hostname" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
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
                print_bench_time(args.benchmark, "Hostname Module", bench);
            },
            "cpu" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
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
                print_bench_time(args.benchmark, "CPU Module", bench);
            },
            "gpu" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.gpu.is_none() {
                    let mut method: GPUMethod = config.gpu.method.clone();
                    if args.gpu_method.is_some() {
                        method = match args.gpu_method.clone().unwrap().as_str() {
                            "pcisysfile" => GPUMethod::PCISysFile,
                            "glxinfo" => GPUMethod::GLXInfo,
                            _ => GPUMethod::PCISysFile
                        }
                    }
                    let use_cache: bool = !args.ignore_cache && config.gpu.cache;
                    known_outputs.gpu = Some(gpu::get_gpu(method, use_cache));
                }
                match known_outputs.gpu.as_ref().unwrap() {
                    Ok(gpu) => {
                        output.push(gpu.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(GPUInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "GPU Module", bench);
            },
            "memory" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.memory.is_none() {
                    known_outputs.memory = Some(memory::get_memory());
                }
                match known_outputs.memory.as_ref().unwrap() {
                    Ok(memory) => {
                        output.push(memory.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(MemoryInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Memory Module", bench);
            },
            "swap" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.swap.is_none() {
                    known_outputs.swap = Some(swap::get_swap());
                }
                match known_outputs.swap.as_ref().unwrap() {
                    Ok(swap) => {
                        output.push(swap.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(SwapInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Swap Module", bench);
            },
            "mounts" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.mounts.is_none() {
                    known_outputs.mounts = Some(mounts::get_mounted_drives());
                }
                match known_outputs.mounts.as_ref().unwrap() {
                    Ok(mounts) => {
                        for mount in mounts {
                            if mount.is_ignored(&config) {
                                continue;
                            }
                            output.push(mount.style(&config, max_title_length))
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(SwapInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Mounts Module", bench);
            },
            "host" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.host.is_none() {
                    known_outputs.host = Some(host::get_host());
                }
                match known_outputs.host.as_ref().unwrap() {
                    Ok(host) => {
                        output.push(host.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(HostInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Host Module", bench);
            },
            "displays" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.displays.is_none() {
                    known_outputs.displays = Some(displays::get_displays());
                }
                match known_outputs.displays.as_ref().unwrap() {
                    Ok(displays) => {
                        for display in displays {
                            output.push(display.style(&config, max_title_length))
                        }
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(DisplayInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Displays Module", bench);
            },
            "os" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
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
                print_bench_time(args.benchmark, "OS Module", bench);
            },
            "packages" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.packages.is_none() {
                    known_outputs.packages = Some(packages::get_packages());
                }
                output.push(known_outputs.packages.as_ref().unwrap().style(&config, max_title_length));
                print_bench_time(args.benchmark, "Packages Module", bench);
            },
            "desktop" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.desktop.is_none() {
                    known_outputs.desktop = Some(desktop::get_desktop());
                }
                match known_outputs.desktop.as_ref().unwrap() {
                    Ok(desktop) => {
                        output.push(desktop.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(DesktopInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Desktop Module", bench);
            },
            "terminal" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.terminal.is_none() {
                    known_outputs.terminal = Some(terminal::get_terminal(config.terminal.chase_ssh_pts));
                }
                match known_outputs.terminal.as_ref().unwrap() {
                    Ok(terminal) => {
                        output.push(terminal.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(TerminalInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Terminal Module", bench);
            },
            "shell" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.shell.is_none() {
                    known_outputs.shell = Some(shell::get_shell(config.shell.show_default_shell));
                }
                match known_outputs.shell.as_ref().unwrap() {
                    Ok(shell) => {
                        output.push(shell.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(ShellInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Shell Module", bench);
            },
            "battery" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.battery.is_none() {
                    known_outputs.battery = Some(battery::get_battery(&config.battery.path));
                }
                match known_outputs.battery.as_ref().unwrap() {
                    Ok(battery) => {
                        output.push(battery.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(BatteryInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Battery Module", bench);
            },
            "uptime" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.uptime.is_none() {
                    known_outputs.uptime = Some(uptime::get_uptime());
                }
                match known_outputs.uptime.as_ref().unwrap() {
                    Ok(uptime) => {
                        output.push(uptime.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(UptimeInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Uptime Module", bench);
            },
            "locale" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.locale.is_none() {
                    known_outputs.locale = Some(locale::get_locale());
                }
                match known_outputs.locale.as_ref().unwrap() {
                    Ok(locale) => {
                        output.push(locale.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(LocaleInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Locale Module", bench);
            },
            "editor" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                if known_outputs.editor.is_none() {
                    known_outputs.editor = Some(editor::get_editor());
                }
                match known_outputs.editor.as_ref().unwrap() {
                    Ok(editor) => {
                        output.push(editor.style(&config, max_title_length))
                    },
                    Err(e) => {
                        if log_errors {
                            output.push(e.to_string());
                        } else {
                            output.push(EditorInfo::unknown_output(&config, max_title_length));
                        }
                    },
                }; 
                print_bench_time(args.benchmark, "Editor Module", bench);
            },
            "colors" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let gap: &str = "   ";
                let mut str: String = String::new();
                str.push_str(&gap.on_black().to_string());
                str.push_str(&gap.on_red().to_string());
                str.push_str(&gap.on_green().to_string());
                str.push_str(&gap.on_yellow().to_string());
                str.push_str(&gap.on_blue().to_string());
                str.push_str(&gap.on_magenta().to_string());
                str.push_str(&gap.on_cyan().to_string());
                str.push_str(&gap.on_white().to_string());
                output.push(str);
                print_bench_time(args.benchmark, "Colors Module", bench);
            }
            "bright_colors" => {
                let bench: Option<Instant> = benchmark_point(args.benchmark); 
                let gap: &str = "   ";
                let mut str: String = String::new();
                str.push_str(&gap.on_bright_black().to_string());
                str.push_str(&gap.on_bright_red().to_string());
                str.push_str(&gap.on_bright_green().to_string());
                str.push_str(&gap.on_bright_yellow().to_string());
                str.push_str(&gap.on_bright_blue().to_string());
                str.push_str(&gap.on_bright_magenta().to_string());
                str.push_str(&gap.on_bright_cyan().to_string());
                str.push_str(&gap.on_bright_white().to_string());
                output.push(str);
                print_bench_time(args.benchmark, "Bright Colors Module", bench);
            }
            _ => {
                output.push(format!("Unknown module: {}", module_name));
            }
        }
        print_bench_time(args.benchmark, "  Entire Module Parse/Detection", module_parse_bench);
    }
    print_bench_time(args.benchmark, "Entire detection step", detect_bench);


    // 
    //  Display
    //
    let bench: Option<Instant> = benchmark_point(args.benchmark); 
    let mut ascii_split: Vec<&str> = Vec::new();
    let mut ascii_length: usize = 0;
    let mut ascii_target_length: u16 = 0;
    if config.ascii.display {
        ascii_split = ascii.0.split("\n").filter(|x| x.trim() != "").collect();
        ascii_length = ascii_split.len();
        ascii_target_length = ascii.1 + config.ascii.margin;
    }
    print_bench_time(args.benchmark, "Display ASCII Pre-Calc", bench);

    let bench: Option<Instant> = benchmark_point(args.benchmark); 
    let mut current_line: usize = 0;
    for out in output {
        if config.ascii.display {
            print!("{}", get_ascii_line(current_line, &ascii_split, &ascii_target_length, &config));
        }

        print!("{}", out);
        current_line += 1;
        println!();
    }
    print_bench_time(args.benchmark, "Module + ASCII Output", bench);
    let bench: Option<Instant> = benchmark_point(args.benchmark); 
    if current_line < ascii_length && config.ascii.display {
        for _ in current_line..ascii_length {
            print!("{}", get_ascii_line(current_line, &ascii_split, &ascii_target_length, &config));
            current_line += 1;
            println!();
        }
    }
    print_bench_time(args.benchmark, "Remaining ASCII Output", bench);
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

    if current_line < ascii_split.len() {
        let colored: ColoredString = color_string(&line, config.ascii.colors.get(index as usize).unwrap());
        return colored.to_string();
    }
    line
}
